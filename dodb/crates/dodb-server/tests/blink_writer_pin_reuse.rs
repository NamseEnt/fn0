use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use dodb_core::{DocumentKey, Error, TenantId, TransactionMutation, TransactionRequest};
use dodb_server::{
    BlinkCheckpointPolicy, BlinkTenantService, BlinkWalLayout, LocalTenantServiceConfig,
};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use dodb_storage::FaultInjector;
use tempfile::TempDir;

fn key() -> DocumentKey {
    DocumentKey::new("writer-pin-reuse", "fixed-overflow-key")
}

fn config(directory: &TempDir, layout: BlinkWalLayout) -> LocalTenantServiceConfig {
    LocalTenantServiceConfig {
        data_dir: directory.path().to_path_buf(),
        blink_wal_layout: layout,
        blink_checkpoint_policy: match layout {
            BlinkWalLayout::SegmentedBases { .. } => {
                BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 1 }
            }
            BlinkWalLayout::Legacy | BlinkWalLayout::Segmented { .. } => {
                BlinkCheckpointPolicy::Disabled
            }
        },
        ..LocalTenantServiceConfig::default()
    }
}

fn layout_from_child_environment() -> BlinkWalLayout {
    match std::env::var("DODB_A1_CRASH_LAYOUT").as_deref() {
        Ok("legacy") => BlinkWalLayout::Legacy,
        Ok("segmented-bases") => BlinkWalLayout::SegmentedBases {
            max_segment_bytes: 16 * 1024 * 1024,
            coalesce: true,
        },
        value => panic!("unexpected crash layout: {value:?}"),
    }
}

async fn write_value(service: &BlinkTenantService, value: Vec<u8>) -> u64 {
    match service
        .execute(
            TenantId::new(1),
            Request::Transact {
                request: TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put { key: key(), value }],
                ),
            },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap()
    {
        Response::Transact(_) => {}
        response => panic!("unexpected response: {response:?}"),
    }
    match service
        .execute(
            TenantId::new(1),
            Request::Get { key: key() },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap()
    {
        Response::Get(state) => state.revision().get(),
        response => panic!("unexpected response: {response:?}"),
    }
}

struct HoldBeforeCommit {
    marker_path: PathBuf,
    enabled: Arc<AtomicBool>,
}

impl FaultInjector for HoldBeforeCommit {
    fn hit(&mut self, point: &str) -> Result<(), Error> {
        if point == "before_commit_record" && self.enabled.load(Ordering::SeqCst) {
            fs::write(&self.marker_path, b"paused").unwrap();
            std::thread::sleep(Duration::from_secs(60));
        }
        Ok(())
    }
}

fn install_before_commit_hold(
    service: &mut BlinkTenantService,
    marker_path: PathBuf,
    enabled: Arc<AtomicBool>,
) {
    service.set_fault_injector_factory(std::sync::Arc::new(move |_, _| {
        Box::new(HoldBeforeCommit {
            marker_path: marker_path.clone(),
            enabled: Arc::clone(&enabled),
        })
    }));
}

async fn run_crash_child(data_dir: PathBuf, stage: &str, marker_directory: &Path) {
    let layout = layout_from_child_environment();
    let config = LocalTenantServiceConfig {
        data_dir: data_dir.clone(),
        blink_wal_layout: layout,
        blink_checkpoint_policy: match layout {
            BlinkWalLayout::SegmentedBases { .. } => {
                BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 1 }
            }
            BlinkWalLayout::Legacy | BlinkWalLayout::Segmented { .. } => {
                BlinkCheckpointPolicy::Disabled
            }
        },
        ..LocalTenantServiceConfig::default()
    };
    let mut service = BlinkTenantService::new(config).unwrap();
    let hold_enabled = Arc::new(AtomicBool::new(false));
    install_before_commit_hold(
        &mut service,
        marker_directory.join(format!("{stage}-paused")),
        Arc::clone(&hold_enabled),
    );
    if stage == "second" {
        let first_revision = fs::read_to_string(marker_directory.join("first-revision"))
            .unwrap()
            .parse()
            .unwrap();
        assert_value(&service, &[3; 4_096], first_revision).await;
        let successful_revision = write_value(&service, vec![7; 4_096]).await;
        assert!(successful_revision > first_revision);
        fs::write(
            marker_directory.join("second-revision"),
            successful_revision.to_string(),
        )
        .unwrap();
        fs::write(
            marker_directory.join("second-write-acknowledged"),
            b"acknowledged",
        )
        .unwrap();
    } else {
        let mut final_revision = 0;
        for value_version in 0..4u8 {
            final_revision = write_value(&service, vec![value_version; 4_096]).await;
        }
        if matches!(layout, BlinkWalLayout::SegmentedBases { .. }) {
            tokio::time::timeout(Duration::from_secs(10), async {
                while service.checkpoint_metrics_snapshot().checkpoints_completed == 0 {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            })
            .await
            .unwrap();
            let bases_directory =
                database_path_from_dir(&data_dir).with_file_name("tenant-1-shard-1.db.segments");
            assert!(fs::read_dir(bases_directory).unwrap().any(|entry| {
                entry
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "pages")
            }));
        }
        fs::write(
            marker_directory.join("first-writes-acknowledged"),
            b"acknowledged",
        )
        .unwrap();
        fs::write(
            marker_directory.join("first-revision"),
            final_revision.to_string(),
        )
        .unwrap();
    }
    hold_enabled.store(true, Ordering::SeqCst);
    let _ = write_value(&service, vec![99; 4_096]).await;
    panic!("uncommitted child transaction unexpectedly completed");
}

fn database_path_from_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("blink/tenant-1-shard-1.db")
}

async fn wait_for_child_markers(child: &mut std::process::Child, marker_paths: &[PathBuf]) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline && marker_paths.iter().any(|path| !path.exists()) {
        if child.try_wait().unwrap().is_some() {
            panic!("crash child exited before reaching its pause markers");
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    for marker_path in marker_paths {
        assert!(marker_path.exists(), "child did not create {marker_path:?}");
    }
}

struct ChildGuard {
    child: Child,
}

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self { child }
    }

    async fn wait_for_markers(&mut self, marker_paths: &[PathBuf]) {
        wait_for_child_markers(&mut self.child, marker_paths).await;
    }

    fn kill_and_wait(&mut self) -> ExitStatus {
        self.child.kill().unwrap();
        self.child.wait().unwrap()
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

async fn assert_value(service: &BlinkTenantService, expected: &[u8], expected_revision: u64) {
    match service
        .execute(
            TenantId::new(1),
            Request::Get { key: key() },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap()
    {
        Response::Get(state) => {
            assert_eq!(state.value(), Some(expected));
            assert_eq!(state.revision().get(), expected_revision);
        }
        response => panic!("unexpected response: {response:?}"),
    }
}

fn database_path(directory: &TempDir) -> PathBuf {
    directory.path().join("blink/tenant-1-shard-1.db")
}

fn append_torn_tail(path: &Path, layout: BlinkWalLayout) {
    let active_wal = match layout {
        BlinkWalLayout::Legacy => path.with_extension("wal"),
        BlinkWalLayout::SegmentedBases { .. } | BlinkWalLayout::Segmented { .. } => {
            let segment_directory = path.with_file_name("tenant-1-shard-1.db.segments");
            let mut segment_paths = fs::read_dir(segment_directory)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|entry_path| {
                    entry_path
                        .extension()
                        .is_some_and(|extension| extension == "wal")
                })
                .collect::<Vec<_>>();
            segment_paths.sort();
            segment_paths.pop().expect("active segment WAL exists")
        }
    };
    OpenOptions::new()
        .append(true)
        .open(active_wal)
        .unwrap()
        .write_all(&[1, 2, 3, 4, 5])
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn service_recovers_torn_tail_accepts_write_and_recovers_again_for_legacy_and_bases() {
    for layout in [
        BlinkWalLayout::Legacy,
        BlinkWalLayout::SegmentedBases {
            max_segment_bytes: 16 * 1024 * 1024,
            coalesce: true,
        },
    ] {
        let directory = TempDir::new().unwrap();
        let config = config(&directory, layout);
        let service = BlinkTenantService::new(config.clone()).unwrap();
        let mut last_revision = 0;
        for value_version in 0..4u8 {
            last_revision = write_value(&service, vec![value_version; 4_096]).await;
        }
        service.shutdown_checked().await.unwrap();
        let path = database_path(&directory);
        if matches!(layout, BlinkWalLayout::SegmentedBases { .. }) {
            assert!(
                path.with_file_name("tenant-1-shard-1.db.segments.manifest")
                    .is_file()
            );
        }
        append_torn_tail(&path, layout);

        let recovered = BlinkTenantService::new(config.clone()).unwrap();
        assert_value(&recovered, &[3; 4_096], last_revision).await;
        let acknowledged_revision = write_value(&recovered, vec![7; 4_096]).await;
        recovered.shutdown_checked().await.unwrap();
        append_torn_tail(&path, layout);

        let recovered_again = BlinkTenantService::new(config).unwrap();
        assert_value(&recovered_again, &[7; 4_096], acknowledged_revision).await;
        recovered_again.shutdown_checked().await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_kill_recovers_incomplete_transaction_then_write_then_second_crash() {
    let Some(data_dir) = std::env::var_os("DODB_A1_CRASH_CHILD_DIR") else {
        for (layout_name, layout) in [
            ("legacy", BlinkWalLayout::Legacy),
            (
                "segmented-bases",
                BlinkWalLayout::SegmentedBases {
                    max_segment_bytes: 16 * 1024 * 1024,
                    coalesce: true,
                },
            ),
        ] {
            let directory = TempDir::new().unwrap();
            for stage in ["first", "second"] {
                let marker_directory = directory.path();
                let marker_paths = if stage == "first" {
                    vec![
                        marker_directory.join("first-writes-acknowledged"),
                        marker_directory.join("first-paused"),
                    ]
                } else {
                    vec![
                        marker_directory.join("second-write-acknowledged"),
                        marker_directory.join("second-paused"),
                    ]
                };
                let child = Command::new(std::env::current_exe().unwrap())
                    .arg("--exact")
                    .arg(
                        "process_kill_recovers_incomplete_transaction_then_write_then_second_crash",
                    )
                    .arg("--nocapture")
                    .env("DODB_A1_CRASH_CHILD_DIR", directory.path())
                    .env("DODB_A1_CRASH_LAYOUT", layout_name)
                    .env("DODB_A1_CRASH_STAGE", stage)
                    .spawn()
                    .unwrap();
                let mut child = ChildGuard::new(child);
                child.wait_for_markers(&marker_paths).await;
                assert_eq!(child.kill_and_wait().signal(), Some(9));
            }
            let reopened = BlinkTenantService::new(config(&directory, layout)).unwrap();
            let first_revision = fs::read_to_string(directory.path().join("first-revision"))
                .unwrap()
                .parse::<u64>()
                .unwrap();
            let second_revision = fs::read_to_string(directory.path().join("second-revision"))
                .unwrap()
                .parse()
                .unwrap();
            assert!(second_revision > first_revision);
            assert_value(&reopened, &[7; 4_096], second_revision).await;
            reopened.shutdown_checked().await.unwrap();
        }
        return;
    };
    let stage = std::env::var("DODB_A1_CRASH_STAGE").unwrap();
    run_crash_child(
        PathBuf::from(data_dir),
        &stage,
        Path::new(&std::env::var("DODB_A1_CRASH_CHILD_DIR").unwrap()),
    )
    .await;
}
