use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use dodb_core::{
    DocumentKey, Revision, RevisionState, TenantId, TransactionCondition, TransactionMutation,
    TransactionRequest,
};
use dodb_rocksdb::{RocksDbService, RocksDbServiceConfig};
use dodb_server::{LocalTenantService, LocalTenantServiceConfig};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use tempfile::TempDir;
use tokio::runtime::Builder;

const TENANT_ID: u64 = 913;
const RESPONSE_BUDGET: ExecutionBudget = ExecutionBudget::new(1 << 20);
const HANDOFF_TIMEOUT: Duration = Duration::from_secs(60);
const CHILD_BACKEND_ENV: &str = "DODB_R1B3A_CHILD_BACKEND";
const CHILD_DATA_DIR_ENV: &str = "DODB_R1B3A_CHILD_DATA_DIR";
const HANDOFF_PREFIX: &str = "R1B3A_HANDOFF|success|";

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
fn native_acknowledged_transaction_survives_process_sigkill_and_reopen() {
    if std::env::var_os(CHILD_BACKEND_ENV).is_some() {
        return;
    }
    run_backend_case(Backend::Native);
}

#[test]
fn rocksdb_acknowledged_transaction_survives_process_sigkill_and_reopen() {
    if std::env::var_os(CHILD_BACKEND_ENV).is_some() {
        return;
    }
    run_backend_case(Backend::RocksDb);
}

#[test]
fn service_child() {
    let Some(backend) = std::env::var_os(CHILD_BACKEND_ENV) else {
        return;
    };
    let backend = parse_backend(&backend.to_string_lossy());
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
                let service = LocalTenantService::new(LocalTenantServiceConfig {
                    data_dir,
                    ..LocalTenantServiceConfig::default()
                })
                .expect("native service should open");
                execute_seed_and_target(&service, scenario).await;
                write_success_handoff(backend);
                std::future::pending::<()>().await;
                service.shutdown().await;
            }
            Backend::RocksDb => {
                let service = RocksDbService::new(RocksDbServiceConfig::new(data_dir))
                    .expect("RocksDB service should open");
                execute_seed_and_target(&service, scenario).await;
                write_success_handoff(backend);
                std::future::pending::<()>().await;
                service.shutdown().await;
            }
        }
    });
}

fn run_backend_case(backend: Backend) {
    let scenario = build_scenario();
    let expected_documents = scenario.expected_documents.clone();
    let deleted_key = scenario.deleted_key.clone();
    let untouched_key = scenario.untouched_key.clone();
    let directory = TempDir::new().expect("new crash test directory should be created");
    let data_dir = directory.path().join("service-data");
    assert!(!data_dir.exists());
    let mut child = spawn_service_child(backend, &data_dir);
    let handoff = receive_handoff(&mut child, backend);
    assert_eq!(handoff, format!("{HANDOFF_PREFIX}{}", backend.as_str()));

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
        "R1B3A_CASE backend={} handoff=success child_signal=9 child_reaped=true",
        backend.as_str()
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
                verify_reopened_state(&service, &expected_documents, &deleted_key, &untouched_key)
                    .await;
                service.shutdown().await;
            }
            Backend::RocksDb => {
                let service = RocksDbService::new(RocksDbServiceConfig::new(&data_dir))
                    .expect("RocksDB service should reopen after child exit");
                verify_reopened_state(&service, &expected_documents, &deleted_key, &untouched_key)
                    .await;
                service.shutdown().await;
            }
        }
    });
    println!(
        "R1B3A_REOPEN backend={} result=passed expected_documents={}",
        backend.as_str(),
        expected_documents.len()
    );
}

fn spawn_service_child(backend: Backend, data_dir: &Path) -> ChildGuard {
    let executable = std::env::current_exe().expect("test executable path should be available");
    let mut child = Command::new(executable)
        .arg("--exact")
        .arg("service_child")
        .arg("--nocapture")
        .env(CHILD_BACKEND_ENV, backend.as_str())
        .env(CHILD_DATA_DIR_ENV, data_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("service child should start");
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

fn receive_handoff(child: &mut ChildGuard, backend: Backend) -> String {
    let deadline = Instant::now() + HANDOFF_TIMEOUT;
    loop {
        let line = child.handoff_lines.recv_timeout(Duration::from_millis(25));
        match line {
            Ok(Ok(line)) if line == format!("{HANDOFF_PREFIX}{}", backend.as_str()) => {
                return line;
            }
            Ok(Ok(_)) => {}
            Ok(Err(error)) => panic!("failed reading child handoff: {error}"),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let status = child
                    .child
                    .try_wait()
                    .expect("child status should be readable")
                    .unwrap_or_else(|| {
                        panic!("child output closed before success handoff for {backend:?}")
                    });
                panic!("child exited before success handoff: {status}");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Some(status) = child
                    .child
                    .try_wait()
                    .expect("child status should be readable")
                {
                    child.reaped = true;
                    panic!("child exited before success handoff: {status}");
                }
            }
        }
        if Instant::now() >= deadline {
            panic!("child success handoff timed out for {backend:?}");
        }
    }
}

async fn execute_seed_and_target<S: DodbService>(service: &S, scenario: Scenario) {
    let tenant = TenantId::new(TENANT_ID);
    let seed_response = service
        .execute(
            tenant,
            Request::Transact {
                request: scenario.seed_request,
            },
            RESPONSE_BUDGET,
        )
        .await
        .expect("seed transaction should succeed");
    assert!(matches!(seed_response, Response::Transact(outcome) if outcome.commit_lsn.is_some()));
    let target_response = service
        .execute(
            tenant,
            Request::Transact {
                request: scenario.target_request,
            },
            RESPONSE_BUDGET,
        )
        .await
        .expect("target transaction should succeed");
    assert!(matches!(target_response, Response::Transact(outcome) if outcome.commit_lsn.is_some()));
}

fn write_success_handoff(backend: Backend) {
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    writeln!(output, "{HANDOFF_PREFIX}{}", backend.as_str())
        .expect("success handoff should be written");
    output.flush().expect("success handoff should be flushed");
}

async fn verify_reopened_state<S: DodbService>(
    service: &S,
    expected_documents: &BTreeMap<DocumentKey, ExpectedDocument>,
    deleted_key: &DocumentKey,
    untouched_key: &DocumentKey,
) {
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
    assert_eq!(actual_documents, *expected_documents);

    let deleted_state = get_state(service, tenant, deleted_key.clone()).await;
    let RevisionState::Missing {
        revision: deleted_revision,
    } = deleted_state
    else {
        panic!("deleted key must be logically missing after reopen");
    };
    let untouched_state = get_state(service, tenant, untouched_key.clone()).await;
    let RevisionState::Present {
        value: untouched_value,
        revision: untouched_revision,
    } = untouched_state
    else {
        panic!("untouched key must remain present after reopen");
    };
    assert_eq!(untouched_value, b"seed-untouched");
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
    let mut expected_documents = seed_request
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
