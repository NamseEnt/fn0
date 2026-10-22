use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use dodb_core::{
    DocumentKey, Error, RevisionState, TenantId, TransactionMutation, TransactionRequest,
};
use dodb_server::{
    BlinkCheckpointExecution, BlinkCheckpointPolicy, BlinkTenantService, BlinkWalLayout,
    LocalTenantServiceConfig,
};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use dodb_storage::FaultInjector;
use tempfile::TempDir;

#[derive(Default)]
struct Gate {
    entered: AtomicBool,
    released: AtomicBool,
    fail: AtomicBool,
    panic: AtomicBool,
}

struct CheckpointGate(Arc<Gate>);

impl FaultInjector for CheckpointGate {
    fn hit(&mut self, point: &str) -> Result<(), Error> {
        if point == "before_checkpoint_data_flush" && !self.0.entered.swap(true, Ordering::SeqCst) {
            let started = std::time::Instant::now();
            while !self.0.released.load(Ordering::SeqCst) {
                if started.elapsed() > Duration::from_secs(20) {
                    return Err(Error::durability("checkpoint barrier timed out"));
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            assert!(
                !self.0.panic.load(Ordering::SeqCst),
                "checkpoint worker panic"
            );
            if self.0.fail.load(Ordering::SeqCst) {
                return Err(Error::durability("checkpoint worker injected failure"));
            }
        }
        Ok(())
    }
}

fn config(directory: &TempDir, execution: BlinkCheckpointExecution) -> LocalTenantServiceConfig {
    LocalTenantServiceConfig {
        data_dir: directory.path().to_path_buf(),
        blink_wal_layout: BlinkWalLayout::Segmented {
            max_segment_bytes: 1,
        },
        blink_checkpoint_policy: BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes: 1 },
        blink_checkpoint_execution: execution,
        ..LocalTenantServiceConfig::default()
    }
}

fn service(config: LocalTenantServiceConfig, gate: &Arc<Gate>) -> Arc<BlinkTenantService> {
    let mut service = BlinkTenantService::new(config).unwrap();
    let gate = Arc::clone(gate);
    service.set_fault_injector_factory(Arc::new(move |_, _| {
        Box::new(CheckpointGate(Arc::clone(&gate)))
    }));
    Arc::new(service)
}

fn key(slot: u64) -> DocumentKey {
    DocumentKey::new("partition", slot.to_be_bytes().to_vec())
}

async fn put(
    service: &BlinkTenantService,
    tenant: u64,
    slot: u64,
    value: Vec<u8>,
) -> Result<Response, Error> {
    service
        .execute(
            TenantId::new(tenant),
            Request::Transact {
                request: TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: key(slot),
                        value,
                    }],
                ),
            },
            ExecutionBudget::new(8 * 1024 * 1024),
        )
        .await
}

async fn get(service: &BlinkTenantService, tenant: u64, slot: u64) -> RevisionState {
    match service
        .execute(
            TenantId::new(tenant),
            Request::Get { key: key(slot) },
            ExecutionBudget::new(8 * 1024 * 1024),
        )
        .await
        .unwrap()
    {
        Response::Get(state) => state,
        response => panic!("unexpected response {response:?}"),
    }
}

async fn entered(gate: &Gate) {
    tokio::time::timeout(Duration::from_secs(10), async {
        while !gate.entered.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
}

async fn idle(service: &BlinkTenantService) {
    tokio::time::timeout(
        Duration::from_secs(10),
        service.wait_for_coordinators_idle(),
    )
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn background_checkpoint_preserves_newer_writes_rotations_tree_growth_and_reopen() {
    let directory = TempDir::new().unwrap();
    let config = config(&directory, BlinkCheckpointExecution::Background);
    let gate = Arc::new(Gate::default());
    let service = service(config.clone(), &gate);
    put(&service, 1, 1, vec![1; 8192]).await.unwrap();
    entered(&gate).await;
    tokio::time::timeout(Duration::from_secs(5), async {
        put(&service, 1, 1, vec![2; 8192]).await.unwrap();
        service
            .execute(
                TenantId::new(1),
                Request::Transact {
                    request: TransactionRequest::new(
                        Vec::new(),
                        vec![TransactionMutation::Delete { key: key(1) }],
                    ),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();
        put(&service, 1, 1, vec![3; 8192]).await.unwrap();
        for batch in 0..8 {
            let mutations = (0..24)
                .map(|offset| {
                    let slot = 10 + batch * 24 + offset;
                    TransactionMutation::Put {
                        key: key(slot),
                        value: vec![slot as u8; 1024],
                    }
                })
                .collect();
            service
                .execute(
                    TenantId::new(1),
                    Request::Transact {
                        request: TransactionRequest::new(Vec::new(), mutations),
                    },
                    ExecutionBudget::new(1024 * 1024),
                )
                .await
                .unwrap();
        }
    })
    .await
    .unwrap();
    let latest = get(&service, 1, 1).await;
    assert_eq!(latest.value(), Some(vec![3; 8192].as_slice()));
    assert!(
        service
            .checkpoint_metrics_snapshot()
            .writes_during_checkpoint
            >= 11
    );
    gate.released.store(true, Ordering::SeqCst);
    idle(&service).await;
    assert_eq!(get(&service, 1, 1).await, latest);
    let metrics = service.checkpoint_metrics_snapshot();
    assert_eq!(metrics.checkpoints_failed, 0, "{metrics:?}");
    assert!(metrics.checkpoints_completed >= 2, "{metrics:?}");
    assert_eq!(metrics.tasks_running, 0);
    assert_eq!(metrics.peak_tasks_running, 1);
    assert_eq!(metrics.retained_image_bytes, 0);
    service.shutdown_checked().await.unwrap();
    let reopened = BlinkTenantService::new(config).unwrap();
    assert_eq!(get(&reopened, 1, 1).await, latest);
    for slot in 10..202 {
        assert_eq!(
            get(&reopened, 1, slot).await.value(),
            Some(vec![slot as u8; 1024].as_slice())
        );
    }
    put(&reopened, 1, 1, vec![4; 8192]).await.unwrap();
    reopened.shutdown_checked().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn synchronous_execution_waits_for_the_identical_snapshot_worker() {
    let directory = TempDir::new().unwrap();
    let gate = Arc::new(Gate::default());
    let service = service(config(&directory, BlinkCheckpointExecution::Sync), &gate);
    put(&service, 1, 1, vec![1; 8192]).await.unwrap();
    entered(&gate).await;
    let next_service = Arc::clone(&service);
    let mut next = tokio::spawn(async move { put(&next_service, 1, 2, vec![2; 8192]).await });
    assert!(
        tokio::time::timeout(Duration::from_millis(30), &mut next)
            .await
            .is_err()
    );
    gate.released.store(true, Ordering::SeqCst);
    next.await.unwrap().unwrap();
    idle(&service).await;
    assert_eq!(
        service
            .checkpoint_metrics_snapshot()
            .writes_during_checkpoint,
        0
    );
    service.shutdown_checked().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn worker_failure_and_panic_fail_closed_preserve_acknowledged_writes_and_resolve_shutdown() {
    for panic_worker in [false, true] {
        let directory = TempDir::new().unwrap();
        let config = config(&directory, BlinkCheckpointExecution::Background);
        let gate = Arc::new(Gate::default());
        let service = service(config.clone(), &gate);
        put(&service, 1, 1, vec![1; 8192]).await.unwrap();
        entered(&gate).await;
        put(&service, 1, 2, vec![2; 8192]).await.unwrap();
        gate.fail.store(!panic_worker, Ordering::SeqCst);
        gate.panic.store(panic_worker, Ordering::SeqCst);
        gate.released.store(true, Ordering::SeqCst);
        idle(&service).await;
        assert!(matches!(
            put(&service, 1, 3, vec![3; 8192]).await,
            Err(Error::DurabilityFailure(_))
        ));
        assert!(service.shutdown_checked().await.is_err());
        let metrics = service.checkpoint_metrics_snapshot();
        assert_eq!(metrics.checkpoints_failed, 1, "{metrics:?}");
        assert_eq!(metrics.tasks_running, 0);
        assert_eq!(metrics.retained_image_bytes, 0);
        let reopened = BlinkTenantService::new(config).unwrap();
        assert_eq!(
            get(&reopened, 1, 1).await.value(),
            Some(vec![1; 8192].as_slice())
        );
        assert_eq!(
            get(&reopened, 1, 2).await.value(),
            Some(vec![2; 8192].as_slice())
        );
        assert!(get(&reopened, 1, 3).await.value().is_none());
        reopened.shutdown_checked().await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn checkpoint_capacity_waits_and_oversized_requests_are_rejected_without_a_commit() {
    let directory = TempDir::new().unwrap();
    let mut config = config(&directory, BlinkCheckpointExecution::Background);
    config.blink_checkpoint_retention_bytes = 96 * 1024;
    let gate = Arc::new(Gate::default());
    let service = service(config, &gate);
    put(&service, 1, 1, vec![1; 8192]).await.unwrap();
    entered(&gate).await;
    let next_service = Arc::clone(&service);
    let mut next = tokio::spawn(async move { put(&next_service, 1, 2, vec![2; 65536]).await });
    assert!(
        tokio::time::timeout(Duration::from_millis(30), &mut next)
            .await
            .is_err()
    );
    gate.released.store(true, Ordering::SeqCst);
    next.await.unwrap().unwrap();
    idle(&service).await;
    assert!(service.checkpoint_metrics_snapshot().capacity_wait_count > 0);
    assert!(matches!(
        put(&service, 1, 3, vec![3; 256 * 1024]).await,
        Err(Error::Overloaded(_))
    ));
    assert!(get(&service, 1, 3).await.value().is_none());
    put(&service, 1, 4, vec![4; 8192]).await.unwrap();
    service.shutdown_checked().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pending_shards_keep_writing_with_only_one_process_checkpoint_worker() {
    let directory = TempDir::new().unwrap();
    let gate = Arc::new(Gate::default());
    let service = service(
        config(&directory, BlinkCheckpointExecution::Background),
        &gate,
    );
    put(&service, 1, 1, vec![1; 8192]).await.unwrap();
    entered(&gate).await;
    tokio::time::timeout(Duration::from_secs(2), async {
        put(&service, 2, 1, vec![2; 8192]).await.unwrap();
        put(&service, 2, 2, vec![3; 8192]).await.unwrap();
    })
    .await
    .unwrap();
    assert_eq!(service.checkpoint_metrics_snapshot().tasks_running, 1);
    gate.released.store(true, Ordering::SeqCst);
    service.shutdown_checked().await.unwrap();
    let metrics = service.checkpoint_metrics_snapshot();
    assert_eq!(metrics.peak_tasks_running, 1, "{metrics:?}");
    assert_eq!(metrics.tasks_running, 0);
    assert!(metrics.checkpoints_completed >= 2, "{metrics:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_joins_an_active_checkpoint_before_returning() {
    let directory = TempDir::new().unwrap();
    let configuration = config(&directory, BlinkCheckpointExecution::Background);
    let gate = Arc::new(Gate::default());
    let service = service(configuration.clone(), &gate);
    put(&service, 1, 1, vec![1; 8192]).await.unwrap();
    entered(&gate).await;
    put(&service, 1, 2, vec![2; 8192]).await.unwrap();
    let closing = Arc::clone(&service);
    let mut shutdown = tokio::spawn(async move { closing.shutdown_checked().await });
    assert!(
        tokio::time::timeout(Duration::from_millis(30), &mut shutdown)
            .await
            .is_err()
    );
    gate.released.store(true, Ordering::SeqCst);
    shutdown.await.unwrap().unwrap();
    assert_eq!(service.checkpoint_metrics_snapshot().tasks_running, 0);
    let reopened = BlinkTenantService::new(configuration).unwrap();
    assert_eq!(
        get(&reopened, 1, 2).await.value(),
        Some(vec![2; 8192].as_slice())
    );
    reopened.shutdown_checked().await.unwrap();
}
