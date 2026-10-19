use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use dodb_core::{
    DocumentKey, Error, Revision, RevisionState, TenantId, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use dodb_rocksdb::{
    ConditionKeyLockMode, RevisionAllocatorMode, RocksDbService, RocksDbServiceConfig,
};
use dodb_server::{FaultInjectorFactory, LocalTenantService, LocalTenantServiceConfig};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use dodb_storage::FaultInjector;
use tempfile::TempDir;
use tokio::runtime::Builder;

const TENANT_ID: u64 = 913;
const RESPONSE_BUDGET: ExecutionBudget = ExecutionBudget::new(1 << 20);
const HANDOFF_TIMEOUT: Duration = Duration::from_secs(60);
const CHILD_BACKEND_ENV: &str = "DODB_R1B3B_CHILD_BACKEND";
const CHILD_DATA_DIR_ENV: &str = "DODB_R1B3B_CHILD_DATA_DIR";
const CHILD_CONDITION_LOCK_MODE_ENV: &str = "DODB_R1B3B_CONDITION_LOCK_MODE";
const CHILD_REVISION_ALLOCATOR_ENV: &str = "DODB_R1B3B_REVISION_ALLOCATOR";
const ROCKSDB_HOOK_ENV: &str = "DODB_R1B3B_ENABLE_WAL_SYNC_HOOK";
const HOOK_PREFIX: &str = "R1B3B_HOOK|";

unsafe extern "C" {
    fn dodb_rocksdb_arm_wal_sync_hook();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Backend {
    Native,
    RocksDb,
}

impl Backend {
    fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::RocksDb => "rocksdb",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExpectedDocument {
    value: Vec<u8>,
}

struct Scenario {
    seed_request: TransactionRequest,
    target_request: TransactionRequest,
    previous_documents: BTreeMap<DocumentKey, ExpectedDocument>,
    expected_documents: BTreeMap<DocumentKey, ExpectedDocument>,
    deleted_key: DocumentKey,
    untouched_key: DocumentKey,
}

struct ChildGuard {
    child: Child,
    handoff_lines: mpsc::Receiver<std::io::Result<String>>,
    reaped: bool,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if !self.reaped {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[test]
fn native_wal_sync_interruption_recovers_all_or_none() {
    if std::env::var_os(CHILD_BACKEND_ENV).is_some() {
        return;
    }
    run_backend_case(
        Backend::Native,
        ConditionKeyLockMode::Exclusive,
        RevisionAllocatorMode::Serialized,
    );
}

#[test]
fn rocksdb_wal_sync_interruption_recovers_all_or_none() {
    if std::env::var_os(CHILD_BACKEND_ENV).is_some() {
        return;
    }
    for condition_key_lock_mode in [
        ConditionKeyLockMode::Exclusive,
        ConditionKeyLockMode::Shared,
    ] {
        for revision_allocator in [
            RevisionAllocatorMode::Serialized,
            RevisionAllocatorMode::ReservedRanges,
        ] {
            run_backend_case(
                Backend::RocksDb,
                condition_key_lock_mode,
                revision_allocator,
            );
        }
    }
}

#[test]
fn service_child() {
    let Some(backend) = std::env::var_os(CHILD_BACKEND_ENV) else {
        return;
    };
    let backend = parse_backend(&backend.to_string_lossy());
    let condition_key_lock_mode = parse_condition_lock_mode(
        &std::env::var(CHILD_CONDITION_LOCK_MODE_ENV).unwrap_or_else(|_| "exclusive".to_owned()),
    );
    let revision_allocator = parse_revision_allocator(
        &std::env::var(CHILD_REVISION_ALLOCATOR_ENV).unwrap_or_else(|_| "serialized".to_owned()),
    );
    assert_eq!(
        backend == Backend::RocksDb,
        std::env::var_os(ROCKSDB_HOOK_ENV).is_some(),
        "RocksDB test hook must be enabled only for the RocksDB child"
    );
    let data_dir = PathBuf::from(
        std::env::var_os(CHILD_DATA_DIR_ENV).expect("child data directory is required"),
    );
    let scenario = build_scenario();
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("child runtime should build");
    runtime.block_on(async move {
        match backend {
            Backend::Native => {
                let mut service = LocalTenantService::new(LocalTenantServiceConfig {
                    data_dir: data_dir.clone(),
                    ..LocalTenantServiceConfig::default()
                })
                .expect("native service should open");
                let hook_armed = Arc::new(AtomicBool::new(false));
                let hook_wal_path =
                    data_dir.join(format!("tenant-{TENANT_ID}-shard-{TENANT_ID}.wal"));
                let injector_armed = Arc::clone(&hook_armed);
                let factory: FaultInjectorFactory = Arc::new(move |_tenant_id, _shard_id| {
                    Box::new(NativeWalSyncHook {
                        armed: Arc::clone(&injector_armed),
                        wal_path: hook_wal_path.clone(),
                    })
                });
                service.set_fault_injector_factory(factory);
                execute_seed(&service, &scenario).await;
                hook_armed.store(true, Ordering::Release);
                execute_target(&service, scenario.target_request).await;
                write_target_returned(backend);
                std::future::pending::<()>().await;
                service.shutdown().await;
            }
            Backend::RocksDb => {
                let mut config = RocksDbServiceConfig::new(data_dir);
                config.condition_key_lock_mode = condition_key_lock_mode;
                config.revision_allocator = revision_allocator;
                let service = RocksDbService::new(config).expect("RocksDB service should open");
                execute_seed(&service, &scenario).await;
                unsafe { dodb_rocksdb_arm_wal_sync_hook() };
                execute_target(&service, scenario.target_request).await;
                write_target_returned(backend);
                std::future::pending::<()>().await;
                service.shutdown().await;
            }
        }
    });
}

fn run_backend_case(
    backend: Backend,
    condition_key_lock_mode: ConditionKeyLockMode,
    revision_allocator: RevisionAllocatorMode,
) {
    let scenario = build_scenario();
    let previous_documents = scenario.previous_documents.clone();
    let expected_documents = scenario.expected_documents.clone();
    let deleted_key = scenario.deleted_key.clone();
    let untouched_key = scenario.untouched_key.clone();
    let directory = TempDir::new().expect("new crash test directory should be created");
    let data_dir = directory.path().join("service-data");
    assert!(!data_dir.exists());
    let mut child = spawn_service_child(
        backend,
        &data_dir,
        condition_key_lock_mode,
        revision_allocator,
    );
    let wal_path = data_dir.join(format!("tenant-{TENANT_ID}-shard-{TENANT_ID}.wal"));
    let hook = receive_hook(&mut child, backend);
    if backend == Backend::Native {
        assert!(hook.starts_with(&format!("{HOOK_PREFIX}native|before_wal_sync|")));
        assert!(wal_path.exists(), "native WAL file must exist at the hook");
        assert!(hook.contains(&wal_path.display().to_string()));
    } else {
        assert!(hook.starts_with(&format!("{HOOK_PREFIX}rocksdb|wal_sync_before|")));
        let hooked_path = Path::new(hook.rsplit('|').next().expect("hook has path"));
        assert!(
            hooked_path
                .extension()
                .is_some_and(|extension| extension == "log"),
            "RocksDB hook must name a WAL log file"
        );
        assert!(
            hooked_path.exists(),
            "RocksDB WAL file must exist at the hook"
        );
    }
    println!(
        "R1B3B_HOOK_REACHED child_pid={} message={hook}",
        child.child.id()
    );
    assert!(
        child
            .child
            .try_wait()
            .expect("child status should be readable")
            .is_none(),
        "child must remain blocked at the actual WAL sync hook"
    );

    child
        .child
        .kill()
        .expect("SIGKILL should be sent to the child PID");
    let status = child.child.wait().expect("killed child should be reaped");
    child.reaped = true;
    assert_eq!(
        status.signal(),
        Some(9),
        "child must terminate from SIGKILL"
    );
    println!(
        "R1B3B_CASE backend={} child_pid={} hook=wal_sync_before child_signal=9 child_reaped=true",
        backend.as_str(),
        child.child.id()
    );

    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("parent runtime should build");
    runtime.block_on(async {
        match backend {
            Backend::Native => {
                let service = LocalTenantService::new(LocalTenantServiceConfig {
                    data_dir: data_dir.clone(),
                    ..LocalTenantServiceConfig::default()
                })
                .expect("native service should reopen after child exit");
                let recovered = verify_reopened_state(
                    &service,
                    &previous_documents,
                    &expected_documents,
                    &deleted_key,
                    &untouched_key,
                )
                .await;
                println!("R1B3B_RECOVERY backend=native state={recovered}");
                service.shutdown().await;
            }
            Backend::RocksDb => {
                let mut config = RocksDbServiceConfig::new(&data_dir);
                config.condition_key_lock_mode = condition_key_lock_mode;
                config.revision_allocator = revision_allocator;
                let service = RocksDbService::new(config)
                    .expect("RocksDB service should reopen after child exit");
                let recovered = verify_reopened_state(
                    &service,
                    &previous_documents,
                    &expected_documents,
                    &deleted_key,
                    &untouched_key,
                )
                .await;
                println!("R1B3B_RECOVERY backend=rocksdb state={recovered}");
                service.shutdown().await;
            }
        }
    });
    println!(
        "R1B3B_REOPEN backend={} result=passed expected_documents={}",
        backend.as_str(),
        expected_documents.len()
    );
}

fn spawn_service_child(
    backend: Backend,
    data_dir: &Path,
    condition_key_lock_mode: ConditionKeyLockMode,
    revision_allocator: RevisionAllocatorMode,
) -> ChildGuard {
    let executable = std::env::current_exe().expect("test executable path should be available");
    let mut command = Command::new(executable);
    command
        .arg("--exact")
        .arg("service_child")
        .arg("--nocapture")
        .env(CHILD_BACKEND_ENV, backend.as_str())
        .env(CHILD_DATA_DIR_ENV, data_dir)
        .env(
            CHILD_CONDITION_LOCK_MODE_ENV,
            match condition_key_lock_mode {
                ConditionKeyLockMode::Exclusive => "exclusive",
                ConditionKeyLockMode::Shared => "shared",
            },
        )
        .env(
            CHILD_REVISION_ALLOCATOR_ENV,
            match revision_allocator {
                RevisionAllocatorMode::Serialized => "serialized",
                RevisionAllocatorMode::ReservedRanges => "reserved-ranges",
            },
        )
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    if backend == Backend::RocksDb {
        command.env(ROCKSDB_HOOK_ENV, "1");
    }
    let mut child = command.spawn().expect("service child should start");
    let stdout = child.stdout.take().expect("child stdout should be piped");
    let (line_sender, line_receiver) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if line_sender.send(line).is_err() {
                return;
            }
        }
    });
    ChildGuard {
        child,
        handoff_lines: line_receiver,
        reaped: false,
    }
}

fn receive_hook(child: &mut ChildGuard, backend: Backend) -> String {
    let deadline = Instant::now() + HANDOFF_TIMEOUT;
    loop {
        let line = child.handoff_lines.recv_timeout(Duration::from_millis(25));
        match line {
            Ok(Ok(line)) => {
                if let Some(event_line) = extract_handoff_event(&line) {
                    if event_line.starts_with("R1B3B_WRONG_SYNC_TARGET|") {
                        panic!("wrong_sync_target: {event_line}");
                    }
                    if event_line.starts_with(HOOK_PREFIX) {
                        let expected_backend_prefix = format!("{HOOK_PREFIX}{}|", backend.as_str());
                        assert!(
                            event_line.starts_with(&expected_backend_prefix),
                            "unexpected WAL sync hook backend: {event_line}"
                        );
                        return event_line.to_owned();
                    }
                    if event_line.starts_with("R1B3B_TARGET_RETURNED|") {
                        panic!("target transaction returned before WAL sync hook: {event_line}");
                    }
                }
            }
            Ok(Err(error)) => panic!("handoff_timeout: failed reading child output: {error}"),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let status = child
                    .child
                    .try_wait()
                    .expect("child status should be readable")
                    .unwrap_or_else(|| {
                        panic!("hook_not_reached: child output closed for {backend:?}")
                    });
                panic!("hook_not_reached: child exited before hook: {status}");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(status) = child
                    .child
                    .try_wait()
                    .expect("child status should be readable")
                {
                    child.reaped = true;
                    panic!("hook_not_reached: child exited before hook: {status}");
                }
            }
        }
        if Instant::now() >= deadline {
            panic!("handoff_timeout: WAL sync hook timed out for {backend:?}");
        }
    }
}

fn extract_handoff_event(output_line: &str) -> Option<&str> {
    let marker_offset = [
        "R1B3B_HOOK|",
        "R1B3B_WRONG_SYNC_TARGET|",
        "R1B3B_TARGET_RETURNED|",
    ]
    .iter()
    .filter_map(|marker| output_line.find(marker))
    .min()?;
    Some(&output_line[marker_offset..])
}

#[test]
fn handoff_event_framing_accepts_raw_and_prefixed_lines() {
    let raw_event = "R1B3B_HOOK|native|before_wal_sync|/tmp/tenant.wal";
    let prefixed_event = format!("test service_child ... {raw_event}");

    assert_eq!(extract_handoff_event(raw_event), Some(raw_event));
    assert_eq!(extract_handoff_event(&prefixed_event), Some(raw_event));
    assert_eq!(
        extract_handoff_event("error: child failed before emitting a handoff event"),
        None
    );
    assert_eq!(
        extract_handoff_event("test service_child ... R1B3B_TARGET_RETURNED|native"),
        Some("R1B3B_TARGET_RETURNED|native")
    );
}

async fn execute_seed<S: DodbService>(service: &S, scenario: &Scenario) {
    let tenant = TenantId::new(TENANT_ID);
    let seed_response = service
        .execute(
            tenant,
            Request::Transact {
                request: scenario.seed_request.clone(),
            },
            RESPONSE_BUDGET,
        )
        .await
        .expect("seed transaction should succeed");
    assert!(matches!(seed_response, Response::Transact(outcome) if outcome.commit_lsn.is_some()));
}

async fn execute_target<S: DodbService>(service: &S, target_request: TransactionRequest) {
    let tenant = TenantId::new(TENANT_ID);
    let target_response = service
        .execute(
            tenant,
            Request::Transact {
                request: target_request,
            },
            RESPONSE_BUDGET,
        )
        .await
        .expect("target transaction should succeed");
    assert!(matches!(target_response, Response::Transact(outcome) if outcome.commit_lsn.is_some()));
}

fn write_target_returned(backend: Backend) {
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    writeln!(output, "R1B3B_TARGET_RETURNED|{}", backend.as_str())
        .expect("target-returned marker should be written");
    output
        .flush()
        .expect("target-returned marker should be flushed");
}

struct NativeWalSyncHook {
    armed: Arc<AtomicBool>,
    wal_path: PathBuf,
}

impl FaultInjector for NativeWalSyncHook {
    fn hit(&mut self, point: &str) -> Result<(), Error> {
        if point == "before_wal_sync" && self.armed.swap(false, Ordering::AcqRel) {
            let stdout = std::io::stdout();
            let mut output = stdout.lock();
            writeln!(
                output,
                "{HOOK_PREFIX}native|before_wal_sync|{}",
                self.wal_path.display()
            )
            .map_err(|error| Error::recovery(format!("native hook handoff failed: {error}")))?;
            output
                .flush()
                .map_err(|error| Error::recovery(format!("native hook flush failed: {error}")))?;
            loop {
                thread::park();
            }
        }
        Ok(())
    }
}

async fn verify_reopened_state<S: DodbService>(
    service: &S,
    previous_documents: &BTreeMap<DocumentKey, ExpectedDocument>,
    expected_documents: &BTreeMap<DocumentKey, ExpectedDocument>,
    deleted_key: &DocumentKey,
    untouched_key: &DocumentKey,
) -> &'static str {
    let tenant = TenantId::new(TENANT_ID);
    let mut cursor = None;
    let mut actual_documents = BTreeMap::new();
    let mut revisions = BTreeMap::new();
    loop {
        let response = service
            .execute(
                tenant,
                Request::Scan {
                    exclusive_after_key: cursor.clone(),
                    limit: 16,
                },
                RESPONSE_BUDGET,
            )
            .await
            .expect("reopened full scan should succeed");
        let Response::Scan(documents) = response else {
            panic!("reopened scan returned an unexpected response");
        };
        if documents.is_empty() {
            break;
        }
        for document in documents {
            cursor = Some(document.key.clone());
            assert!(
                actual_documents
                    .insert(
                        document.key.clone(),
                        ExpectedDocument {
                            value: document.value,
                        },
                    )
                    .is_none(),
                "scan returned a duplicate logical document"
            );
            revisions.insert(document.key, document.revision);
        }
    }
    let recovered_state = if actual_documents == *previous_documents {
        "previous"
    } else if actual_documents == *expected_documents {
        "complete_target"
    } else {
        panic!("partial_or_unexpected_state: full scan differs from both complete oracle states")
    };

    let deleted_state = get_state(service, tenant, deleted_key.clone()).await;
    let untouched_state = get_state(service, tenant, untouched_key.clone()).await;
    let RevisionState::Present {
        value: untouched_value,
        revision: untouched_revision,
    } = untouched_state
    else {
        panic!("untouched key must remain present after reopen");
    };
    assert_eq!(untouched_value, b"seed-untouched");
    if recovered_state == "previous" {
        assert_eq!(
            deleted_state,
            RevisionState::Present {
                value: b"seed-delete".to_vec(),
                revision: *revisions
                    .get(deleted_key)
                    .expect("seed delete key is scanned"),
            }
        );
        let seed_revision = *revisions
            .values()
            .next()
            .expect("previous state has documents");
        assert!(
            revisions
                .values()
                .all(|revision| *revision == seed_revision)
        );
        let created_state = get_state(service, tenant, key(b"crash", b"created")).await;
        assert_eq!(
            created_state,
            RevisionState::Missing {
                revision: Revision::ZERO
            }
        );
    } else {
        let RevisionState::Missing {
            revision: deleted_revision,
        } = deleted_state
        else {
            panic!("complete_target state requires a deleted tombstone");
        };
        assert!(deleted_revision > Revision::ZERO);
        let target_revision = *revisions
            .get(&key(b"crash", b"updated-a"))
            .expect("updated key should be present");
        for changed_key in [key(b"crash", b"updated-b"), key(b"crash", b"created")] {
            assert_eq!(revisions.get(&changed_key), Some(&target_revision));
        }
        assert_eq!(deleted_revision, target_revision);
        assert!(deleted_revision > untouched_revision);
        assert!(target_revision > untouched_revision);
    }

    let reopen_key = key(b"crash", b"post-reopen");
    let reopen_request = TransactionRequest::new(
        vec![TransactionCondition::NotExists {
            key: reopen_key.clone(),
        }],
        vec![TransactionMutation::Put {
            key: reopen_key.clone(),
            value: b"post-reopen".to_vec(),
        }],
    );
    let response = service
        .execute(
            tenant,
            Request::Transact {
                request: reopen_request,
            },
            RESPONSE_BUDGET,
        )
        .await
        .expect("post-reopen transaction should succeed");
    assert!(matches!(response, Response::Transact(outcome) if outcome.commit_lsn.is_some()));
    let mut post_reopen_documents = if recovered_state == "previous" {
        previous_documents.clone()
    } else {
        expected_documents.clone()
    };
    post_reopen_documents.insert(
        reopen_key,
        ExpectedDocument {
            value: b"post-reopen".to_vec(),
        },
    );
    let post_reopen_scan = full_scan(service, tenant).await;
    assert_eq!(post_reopen_scan.0, post_reopen_documents);
    assert!(
        post_reopen_scan
            .1
            .values()
            .all(|revision| *revision > Revision::ZERO)
    );
    recovered_state
}

async fn full_scan<S: DodbService>(
    service: &S,
    tenant: TenantId,
) -> (
    BTreeMap<DocumentKey, ExpectedDocument>,
    BTreeMap<DocumentKey, Revision>,
) {
    let mut cursor = None;
    let mut actual_documents = BTreeMap::new();
    let mut revisions = BTreeMap::new();
    loop {
        let response = service
            .execute(
                tenant,
                Request::Scan {
                    exclusive_after_key: cursor.clone(),
                    limit: 16,
                },
                RESPONSE_BUDGET,
            )
            .await
            .expect("reopened full scan should succeed");
        let Response::Scan(documents) = response else {
            panic!("reopened scan returned an unexpected response");
        };
        if documents.is_empty() {
            break;
        }
        for document in documents {
            cursor = Some(document.key.clone());
            assert!(
                actual_documents
                    .insert(
                        document.key.clone(),
                        ExpectedDocument {
                            value: document.value,
                        },
                    )
                    .is_none(),
                "full scan returned a duplicate logical document"
            );
            revisions.insert(document.key, document.revision);
        }
    }
    (actual_documents, revisions)
}

async fn get_state<S: DodbService>(
    service: &S,
    tenant: TenantId,
    document_key: DocumentKey,
) -> RevisionState {
    match service
        .execute(tenant, Request::Get { key: document_key }, RESPONSE_BUDGET)
        .await
        .expect("reopened Get should succeed")
    {
        Response::Get(state) => state,
        _ => panic!("reopened Get returned an unexpected response"),
    }
}

fn build_scenario() -> Scenario {
    let seed_rows = [
        (key(b"crash", b"updated-a"), b"seed-a".to_vec()),
        (key(b"crash", b"updated-b"), b"seed-b".to_vec()),
        (key(b"crash", b"deleted"), b"seed-delete".to_vec()),
        (key(b"crash", b"untouched"), b"seed-untouched".to_vec()),
    ];
    let seed_request = TransactionRequest::new(
        Vec::new(),
        seed_rows
            .iter()
            .map(|(document_key, value)| TransactionMutation::Put {
                key: document_key.clone(),
                value: value.clone(),
            })
            .collect(),
    );
    let updated_a = key(b"crash", b"updated-a");
    let updated_b = key(b"crash", b"updated-b");
    let deleted_key = key(b"crash", b"deleted");
    let created_key = key(b"crash", b"created");
    let untouched_key = key(b"crash", b"untouched");
    let missing_condition_key = key(b"crash", b"missing-condition");
    let target_request = TransactionRequest::new(
        vec![
            TransactionCondition::Exists {
                key: updated_a.clone(),
            },
            TransactionCondition::Exists {
                key: deleted_key.clone(),
            },
            TransactionCondition::NotExists {
                key: created_key.clone(),
            },
            TransactionCondition::Exists {
                key: untouched_key.clone(),
            },
            TransactionCondition::NotExists {
                key: missing_condition_key,
            },
        ],
        vec![
            TransactionMutation::Put {
                key: updated_a.clone(),
                value: b"target-a".to_vec(),
            },
            TransactionMutation::Put {
                key: updated_b.clone(),
                value: b"target-b".to_vec(),
            },
            TransactionMutation::Put {
                key: created_key.clone(),
                value: b"target-created".to_vec(),
            },
            TransactionMutation::Delete {
                key: deleted_key.clone(),
            },
        ],
    );
    let previous_documents = seed_request
        .mutations
        .iter()
        .filter_map(|mutation| match mutation {
            TransactionMutation::Put { key, value } => Some((
                key.clone(),
                ExpectedDocument {
                    value: value.clone(),
                },
            )),
            TransactionMutation::Delete { .. } => None,
        })
        .collect::<BTreeMap<_, _>>();
    let mut expected_documents = previous_documents.clone();
    for mutation in &target_request.mutations {
        match mutation {
            TransactionMutation::Put { key, value } => {
                expected_documents.insert(
                    key.clone(),
                    ExpectedDocument {
                        value: value.clone(),
                    },
                );
            }
            TransactionMutation::Delete { key } => {
                expected_documents.remove(key);
            }
        }
    }
    Scenario {
        seed_request,
        target_request,
        previous_documents,
        expected_documents,
        deleted_key,
        untouched_key,
    }
}

fn key(partition: &[u8], sort: &[u8]) -> DocumentKey {
    DocumentKey::new(partition.to_vec(), sort.to_vec())
}

fn parse_backend(value: &str) -> Backend {
    match value {
        "native" => Backend::Native,
        "rocksdb" => Backend::RocksDb,
        _ => panic!("unknown child backend {value}"),
    }
}

fn parse_condition_lock_mode(value: &str) -> ConditionKeyLockMode {
    match value {
        "exclusive" => ConditionKeyLockMode::Exclusive,
        "shared" => ConditionKeyLockMode::Shared,
        _ => panic!("unknown child condition key lock mode {value}"),
    }
}

fn parse_revision_allocator(value: &str) -> RevisionAllocatorMode {
    match value {
        "serialized" => RevisionAllocatorMode::Serialized,
        "reserved-ranges" => RevisionAllocatorMode::ReservedRanges,
        _ => panic!("unknown child revision allocator {value}"),
    }
}
