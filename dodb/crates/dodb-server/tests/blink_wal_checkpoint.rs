use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use dodb_core::{
    DocumentKey, Error, TenantId, TransactionCondition, TransactionMutation, TransactionRequest,
};
use dodb_server::{
    BlinkCheckpointPolicy, BlinkTenantService, BlinkWalLayout, LocalTenantServiceConfig,
};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use dodb_storage::FaultInjector;
use tempfile::TempDir;

fn service_config(
    data_dir: std::path::PathBuf,
    policy: BlinkCheckpointPolicy,
) -> LocalTenantServiceConfig {
    LocalTenantServiceConfig {
        data_dir,
        blink_checkpoint_policy: policy,
        ..LocalTenantServiceConfig::default()
    }
}

fn key(slot: u64) -> DocumentKey {
    DocumentKey::new(format!("tenant-{slot}"), "row")
}

async fn write_value(service: &BlinkTenantService, slot: u64, value: Vec<u8>) -> u64 {
    match service
        .execute(
            TenantId::new(1),
            Request::Transact {
                request: TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: key(slot),
                        value,
                    }],
                ),
            },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap()
    {
        Response::Transact(outcome) => outcome.commit_lsn.unwrap().get(),
        response => panic!("unexpected response: {response:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn disabled_policy_does_not_checkpoint_and_default_is_disabled() {
    let directory = TempDir::new().unwrap();
    let service = BlinkTenantService::new(service_config(
        directory.path().to_path_buf(),
        BlinkCheckpointPolicy::Disabled,
    ))
    .unwrap();
    write_value(&service, 1, vec![7; 4_096]).await;
    service.shutdown().await;
    let metrics = service.checkpoint_metrics_snapshot();
    assert_eq!(metrics.policy, BlinkCheckpointPolicy::Disabled);
    assert_eq!(metrics.checkpoints_started, 0);
    assert_eq!(metrics.checkpoints_completed, 0);
    assert_eq!(
        LocalTenantServiceConfig::default().blink_wal_layout,
        BlinkWalLayout::Legacy
    );
    let database_path = directory.path().join("blink").join("tenant-1-shard-1.db");
    assert!(database_path.with_extension("wal").is_file());
    assert!(
        !database_path
            .with_file_name("tenant-1-shard-1.db.segments.manifest")
            .exists()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn explicit_segmented_layout_uses_segmented_wal_through_service_transactions() {
    let directory = TempDir::new().unwrap();
    let mut config = service_config(
        directory.path().to_path_buf(),
        BlinkCheckpointPolicy::Disabled,
    );
    config.blink_wal_layout = BlinkWalLayout::Segmented {
        max_segment_bytes: 1,
    };
    let service = BlinkTenantService::new(config.clone()).unwrap();
    write_value(&service, 1, b"segmented-service-value".to_vec()).await;
    let state = match service
        .execute(
            TenantId::new(1),
            Request::Get { key: key(1) },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap()
    {
        Response::Get(state) => state,
        response => panic!("unexpected response: {response:?}"),
    };
    service.shutdown().await;

    let database_path = directory.path().join("blink").join("tenant-1-shard-1.db");
    assert!(
        database_path
            .with_file_name("tenant-1-shard-1.db.segments.manifest")
            .is_file()
    );
    assert!(!database_path.with_extension("wal").exists());
    assert!(
        database_path
            .with_file_name("tenant-1-shard-1.db.segments")
            .is_dir()
    );

    let reopened = BlinkTenantService::new(config).unwrap();
    match reopened
        .execute(
            TenantId::new(1),
            Request::Get { key: key(1) },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap()
    {
        Response::Get(recovered) => {
            assert_eq!(
                recovered.value(),
                Some(b"segmented-service-value".as_slice())
            );
            assert_eq!(recovered.revision(), state.revision());
        }
        response => panic!("unexpected response: {response:?}"),
    }
    reopened.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn segmented_threshold_checkpoints_reclaim_files_and_reopen() {
    let directory = TempDir::new().unwrap();
    let mut config = service_config(
        directory.path().to_path_buf(),
        BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 1 },
    );
    config.blink_wal_layout = BlinkWalLayout::Segmented {
        max_segment_bytes: u64::MAX,
    };
    let service = BlinkTenantService::new(config.clone()).unwrap();
    for slot in 1..=4 {
        write_value(&service, slot, vec![slot as u8; 8192]).await;
        service.wait_for_coordinators_idle().await;
    }
    let metrics = service.checkpoint_metrics_snapshot();
    assert_eq!(metrics.checkpoints_completed, 4, "{metrics:?}");
    assert_eq!(metrics.checkpoints_failed, 0);
    assert_eq!(metrics.commit_wal_syncs, 4);
    assert_eq!(metrics.checkpoint_wal_syncs, 4);
    let segment_directory = directory.path().join("blink/tenant-1-shard-1.db.segments");
    assert_eq!(std::fs::read_dir(&segment_directory).unwrap().count(), 1);
    assert_eq!(
        metrics.wal_bytes_current,
        std::fs::read_dir(&segment_directory)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .metadata()
            .unwrap()
            .len()
    );
    service.shutdown().await;
    let reopened = BlinkTenantService::new(config).unwrap();
    for slot in 1..=4 {
        match reopened
            .execute(
                TenantId::new(1),
                Request::Get { key: key(slot) },
                ExecutionBudget::new(64 * 1024),
            )
            .await
            .unwrap()
        {
            Response::Get(state) => {
                assert_eq!(state.value(), Some(vec![slot as u8; 8192].as_slice()))
            }
            response => panic!("unexpected response: {response:?}"),
        }
    }
    write_value(&reopened, 5, b"after-reopen".to_vec()).await;
    reopened.wait_for_coordinators_idle().await;
    assert_eq!(
        reopened.checkpoint_metrics_snapshot().checkpoints_completed,
        1
    );
    reopened.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn threshold_checkpoints_repeat_and_reopen_keeps_latest_revision() {
    let directory = TempDir::new().unwrap();
    let config = service_config(
        directory.path().to_path_buf(),
        BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 1 },
    );
    let service = BlinkTenantService::new(config.clone()).unwrap();
    for slot in 1..=3 {
        write_value(&service, slot, vec![slot as u8; 4_096]).await;
        match service
            .execute(
                TenantId::new(1),
                Request::Get { key: key(slot) },
                ExecutionBudget::new(64 * 1024),
            )
            .await
            .unwrap()
        {
            Response::Get(state) => {
                assert_eq!(state.value(), Some(&vec![slot as u8; 4_096][..]));
            }
            response => panic!("unexpected response: {response:?}"),
        }
    }
    service.wait_for_coordinators_idle().await;
    let metrics_before_reset = service.checkpoint_metrics_snapshot();
    assert_eq!(metrics_before_reset.checkpoints_completed, 3);
    service.reset_checkpoint_metrics();
    let latest_revision = write_value(&service, 4, vec![4; 4_096]).await;
    service.wait_for_coordinators_idle().await;
    service.shutdown().await;
    let metrics = service.checkpoint_metrics_snapshot();
    assert_eq!(metrics.checkpoints_started, 1);
    assert_eq!(metrics.checkpoints_completed, 1, "{metrics:?}");
    assert_eq!(metrics.checkpoints_failed, 0);
    assert_eq!(metrics.successful_mutation_transactions, 1);
    assert_eq!(metrics.mutation_commit_groups, 1);
    assert_eq!(metrics.commit_wal_syncs, 1);
    assert_eq!(metrics.checkpoint_wal_syncs, 2);
    assert_eq!(
        metrics.wal_syncs,
        metrics.commit_wal_syncs + metrics.checkpoint_wal_syncs
    );
    assert!(metrics.checkpoint_events.iter().all(|event| {
        event.succeeded
            && event.wal_bytes_before >= 1
            && event.wal_bytes_after < event.wal_bytes_before
    }));
    let reopened = BlinkTenantService::new(config).unwrap();
    match reopened
        .execute(
            TenantId::new(1),
            Request::Get { key: key(4) },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap()
    {
        Response::Get(state) => {
            assert_eq!(state.revision().get(), latest_revision);
            assert_eq!(state.value(), Some(&vec![4; 4_096][..]));
        }
        response => panic!("unexpected response: {response:?}"),
    }
    reopened.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn threshold_below_current_wal_does_not_checkpoint() {
    let directory = TempDir::new().unwrap();
    let service = BlinkTenantService::new(service_config(
        directory.path().to_path_buf(),
        BlinkCheckpointPolicy::WalBytesThreshold {
            threshold_bytes: 16 * 1024 * 1024,
        },
    ))
    .unwrap();
    write_value(&service, 1, vec![3; 128]).await;
    service
        .execute(
            TenantId::new(1),
            Request::Transact {
                request: TransactionRequest::new(
                    vec![TransactionCondition::NotExists { key: key(9) }],
                    Vec::new(),
                ),
            },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap();
    service
        .execute(
            TenantId::new(1),
            Request::Get { key: key(1) },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap();
    let metrics = service.checkpoint_metrics_snapshot();
    assert_eq!(metrics.checkpoints_started, 0);
    assert_eq!(metrics.successful_mutation_transactions, 1);
    service.shutdown().await;
}

#[test]
fn zero_threshold_and_btree_policy_are_rejected() {
    let directory = TempDir::new().unwrap();
    let zero_threshold = service_config(
        directory.path().join("zero"),
        BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 0 },
    );
    assert!(BlinkTenantService::new(zero_threshold).is_err());

    let unsupported_btree_policy = service_config(
        directory.path().join("btree"),
        BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 1 },
    );
    assert!(dodb_server::LocalTenantService::new(unsupported_btree_policy).is_err());
    let zero_segment = LocalTenantServiceConfig {
        data_dir: directory.path().join("zero-segment"),
        blink_wal_layout: BlinkWalLayout::Segmented {
            max_segment_bytes: 0,
        },
        ..LocalTenantServiceConfig::default()
    };
    assert!(BlinkTenantService::new(zero_segment).is_err());
    let unsupported_btree_layout = LocalTenantServiceConfig {
        data_dir: directory.path().join("btree-layout"),
        blink_wal_layout: BlinkWalLayout::Segmented {
            max_segment_bytes: 1024,
        },
        ..LocalTenantServiceConfig::default()
    };
    assert!(dodb_server::LocalTenantService::new(unsupported_btree_layout).is_err());
}

struct FailCheckpointOnce(Arc<AtomicBool>);

impl FaultInjector for FailCheckpointOnce {
    fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
        if point == "before_checkpoint_data_flush" && !self.0.swap(true, Ordering::SeqCst) {
            return Err(Error::durability("injected checkpoint failure"));
        }
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn checkpoint_failure_keeps_acknowledged_commit_and_fails_later_writes() {
    let directory = TempDir::new().unwrap();
    let config = service_config(
        directory.path().to_path_buf(),
        BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 1 },
    );
    let failure_seen = Arc::new(AtomicBool::new(false));
    let mut service = BlinkTenantService::new(config.clone()).unwrap();
    let injector_state = Arc::clone(&failure_seen);
    service.set_fault_injector_factory(Arc::new(move |_, _| {
        Box::new(FailCheckpointOnce(Arc::clone(&injector_state)))
    }));

    write_value(&service, 1, vec![9; 4_096]).await;
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while service.checkpoint_metrics_snapshot().checkpoints_failed == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let later_write = service
        .execute(
            TenantId::new(1),
            Request::Transact {
                request: TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: key(2),
                        value: vec![8; 128],
                    }],
                ),
            },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap_err();
    assert!(
        later_write
            .to_string()
            .contains("injected checkpoint failure")
    );
    service.shutdown().await;

    let reopened = BlinkTenantService::new(config).unwrap();
    match reopened
        .execute(
            TenantId::new(1),
            Request::Get { key: key(1) },
            ExecutionBudget::new(64 * 1024),
        )
        .await
        .unwrap()
    {
        Response::Get(state) => {
            assert_eq!(state.value(), Some(&vec![9; 4_096][..]));
        }
        response => panic!("unexpected response: {response:?}"),
    }
    reopened.shutdown().await;
}

struct HoldCheckpointAtDataFlush {
    marker_path: std::path::PathBuf,
}

impl FaultInjector for HoldCheckpointAtDataFlush {
    fn hit(&mut self, point: &str) -> dodb_core::Result<()> {
        if point == "before_checkpoint_data_flush" {
            std::fs::write(&self.marker_path, b"paused").unwrap();
            std::thread::sleep(Duration::from_secs(60));
        }
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn process_kill_during_checkpoint_replays_acknowledged_commit() {
    let Some(data_dir) = std::env::var_os("DODB_BLINK_CHECKPOINT_CRASH_CHILD_DIR") else {
        let directory = TempDir::new().unwrap();
        let marker_path = directory.path().join("checkpoint-paused");
        let acknowledged_path = directory.path().join("commit-acknowledged");
        let mut child = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("process_kill_during_checkpoint_replays_acknowledged_commit")
            .arg("--nocapture")
            .env("DODB_BLINK_CHECKPOINT_CRASH_CHILD_DIR", directory.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline && (!marker_path.exists() || !acknowledged_path.exists()) {
            if child.try_wait().unwrap().is_some() {
                panic!("checkpoint crash child exited before reaching the pause point");
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert!(marker_path.exists(), "child did not reach checkpoint pause");
        assert!(
            acknowledged_path.exists(),
            "child did not acknowledge the transaction before checkpoint"
        );
        child.kill().unwrap();
        let child_status = child.wait().unwrap();
        assert_eq!(child_status.signal(), Some(9));

        let reopened = BlinkTenantService::new(service_config(
            directory.path().to_path_buf(),
            BlinkCheckpointPolicy::Disabled,
        ))
        .unwrap();
        match reopened
            .execute(
                TenantId::new(1),
                Request::Get { key: key(77) },
                ExecutionBudget::new(64 * 1024),
            )
            .await
            .unwrap()
        {
            Response::Get(state) => {
                assert_eq!(state.value(), Some(&vec![77; 4_096][..]));
            }
            response => panic!("unexpected response after process kill: {response:?}"),
        }
        reopened.shutdown().await;
        return;
    };

    let data_dir = std::path::PathBuf::from(data_dir);
    let mut service = BlinkTenantService::new(service_config(
        data_dir.clone(),
        BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 1 },
    ))
    .unwrap();
    let checkpoint_marker = data_dir.join("checkpoint-paused");
    service.set_fault_injector_factory(Arc::new(move |_, _| {
        Box::new(HoldCheckpointAtDataFlush {
            marker_path: checkpoint_marker.clone(),
        })
    }));
    write_value(&service, 77, vec![77; 4_096]).await;
    std::fs::write(data_dir.join("commit-acknowledged"), b"acknowledged").unwrap();
    std::future::pending::<()>().await;
}
