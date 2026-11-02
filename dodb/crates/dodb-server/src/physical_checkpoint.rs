use super::*;
use dodb_storage::{BlinkCheckpointOutput, BlinkCheckpointReport, BlinkStore};
use std::collections::{HashSet, VecDeque};
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::SyncSender;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::{JoinError, JoinHandle};

type Store = BlinkStore<ProductionFile, ProductionFile>;
type WorkerResult = Result<Result<BlinkCheckpointOutput, Error>, JoinError>;
type CheckpointProgressJob = Box<dyn FnOnce() + Send>;
type CheckpointProgressSender = SyncSender<CheckpointProgressJob>;

static TASK_SEQUENCE: AtomicU64 = AtomicU64::new(1);

static CHECKPOINT_CAPACITY: OnceLock<Arc<FairCheckpointScheduler>> = OnceLock::new();
static CHECKPOINT_PROGRESS_SENDER: OnceLock<Result<CheckpointProgressSender, String>> =
    OnceLock::new();

pub(super) fn submit_checkpoint_progress<T, F>(job: F) -> Result<oneshot::Receiver<T>, Error>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let sender = CHECKPOINT_PROGRESS_SENDER.get_or_init(|| {
        let (sender, receiver) = std::sync::mpsc::sync_channel::<CheckpointProgressJob>(16);
        std::thread::Builder::new()
            .name("dodb-checkpoint-progress".to_owned())
            .spawn(move || {
                while let Ok(job) = receiver.recv() {
                    job();
                }
            })
            .map_err(|error| error.to_string())?;
        Ok(sender)
    });
    let sender = sender.as_ref().map_err(|error| {
        Error::durability(format!("checkpoint progress worker failed: {error}"))
    })?;
    let (result_sender, result_receiver) = oneshot::channel();
    let progress_job = Box::new(move || {
        if let Ok(result) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job)) {
            let _ = result_sender.send(result);
        }
    });
    sender.try_send(progress_job).map_err(|error| match error {
        std::sync::mpsc::TrySendError::Full(_) => {
            Error::overloaded("checkpoint progress worker queue is full")
        }
        std::sync::mpsc::TrySendError::Disconnected(_) => {
            Error::durability("checkpoint progress worker stopped")
        }
    })?;
    Ok(result_receiver)
}

struct CheckpointWorkerActivity(Arc<AtomicBool>);

impl Drop for CheckpointWorkerActivity {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[derive(Default)]
pub(super) struct FairTenantReadyQueue {
    ready: VecDeque<u64>,
    pending: HashSet<u64>,
}

impl FairTenantReadyQueue {
    pub(super) fn enqueue(&mut self, tenant: u64) -> bool {
        if !self.pending.insert(tenant) {
            return false;
        }
        self.ready.push_back(tenant);
        true
    }

    pub(super) fn take_next(&mut self) -> Option<u64> {
        let tenant = self.ready.pop_front()?;
        self.pending.remove(&tenant);
        Some(tenant)
    }

    pub(super) fn cancel(&mut self, tenant: u64) -> bool {
        if !self.pending.remove(&tenant) {
            return false;
        }
        self.ready.retain(|ready_tenant| *ready_tenant != tenant);
        true
    }

    #[cfg(test)]
    fn is_pending(&self, tenant: u64) -> bool {
        self.pending.contains(&tenant)
    }

    pub(super) fn len(&self) -> usize {
        self.ready.len()
    }

    fn front(&self) -> Option<u64> {
        self.ready.front().copied()
    }
}

#[derive(Default)]
struct FairCheckpointState {
    ready: FairTenantReadyQueue,
    active: Option<u64>,
}

struct FairCheckpointScheduler {
    state: StdMutex<FairCheckpointState>,
    generation: watch::Sender<u64>,
}

impl FairCheckpointScheduler {
    fn new() -> Self {
        let (generation, _) = watch::channel(0);
        Self {
            state: StdMutex::new(FairCheckpointState::default()),
            generation,
        }
    }

    fn signal(&self) {
        self.generation.send_modify(|generation| {
            *generation = generation.wrapping_add(1);
        });
    }

    fn try_acquire(self: &Arc<Self>, tenant: u64) -> Option<CheckpointCapacityPermit> {
        let mut state = self.state.lock().ok()?;
        if state.active.is_some() || state.ready.len() > 0 {
            return None;
        }
        state.active = Some(tenant);
        Some(CheckpointCapacityPermit {
            scheduler: Arc::clone(self),
            tenant,
        })
    }

    async fn acquire(self: &Arc<Self>, tenant: u64) -> Result<CheckpointCapacityPermit, Error> {
        let mut generation = self.generation.subscribe();
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| Error::invariant("checkpoint scheduler lock poisoned"))?;
            if !state.ready.enqueue(tenant) {
                return Err(Error::invariant(
                    "tenant already has a pending checkpoint ticket",
                ));
            }
        }
        self.signal();
        let mut ticket = CheckpointWaitTicket {
            scheduler: Arc::clone(self),
            tenant,
            pending: true,
        };
        loop {
            let _ = *generation.borrow_and_update();
            {
                let mut state = self
                    .state
                    .lock()
                    .map_err(|_| Error::invariant("checkpoint scheduler lock poisoned"))?;
                if state.active.is_none() && state.ready.front() == Some(tenant) {
                    let selected = state.ready.take_next();
                    if selected != Some(tenant) {
                        return Err(Error::invariant(
                            "checkpoint scheduler selected a different tenant",
                        ));
                    }
                    state.active = Some(tenant);
                    ticket.pending = false;
                    return Ok(CheckpointCapacityPermit {
                        scheduler: Arc::clone(self),
                        tenant,
                    });
                }
            }
            generation
                .changed()
                .await
                .map_err(|_| Error::durability("checkpoint scheduler notification closed"))?;
        }
    }
}

struct CheckpointWaitTicket {
    scheduler: Arc<FairCheckpointScheduler>,
    tenant: u64,
    pending: bool,
}

impl Drop for CheckpointWaitTicket {
    fn drop(&mut self) {
        if self.pending
            && let Ok(mut state) = self.scheduler.state.lock()
            && state.ready.cancel(self.tenant)
        {
            drop(state);
            self.scheduler.signal();
        }
    }
}

pub(super) struct CheckpointCapacityPermit {
    scheduler: Arc<FairCheckpointScheduler>,
    tenant: u64,
}

#[cfg(feature = "blink-demand-paging")]
pub(super) async fn acquire_fair_checkpoint_slice(
    tenant: u64,
) -> Result<CheckpointCapacityPermit, Error> {
    let scheduler = CHECKPOINT_CAPACITY.get_or_init(|| Arc::new(FairCheckpointScheduler::new()));
    Arc::clone(scheduler).acquire(tenant).await
}

impl Drop for CheckpointCapacityPermit {
    fn drop(&mut self) {
        if let Ok(mut state) = self.scheduler.state.lock()
            && state.active == Some(self.tenant)
        {
            state.active = None;
            drop(state);
            self.scheduler.signal();
        }
    }
}

pub(super) struct PhysicalCheckpointConfig {
    pub execution: BlinkCheckpointExecution,
    pub injector_factory: Option<(FaultInjectorFactory, TenantId, ShardId)>,
}

pub(super) struct CoordinatorSettings {
    pub grouping: CoordinatorConfig,
    pub policy: BlinkCheckpointPolicy,
    pub tenant_id: u64,
    pub physical: PhysicalCheckpointConfig,
    pub metrics: Arc<StdMutex<BlinkWalCheckpointMetrics>>,
    pub activity: Arc<CoordinatorActivity>,
    pub failed_message: Arc<StdMutex<Option<String>>>,
    #[cfg(all(test, feature = "blink-demand-paging"))]
    pub commit_test_gate: Option<Arc<super::OneShotTestGate>>,
}

struct RunningCheckpoint {
    worker: JoinHandle<Result<BlinkCheckpointOutput, Error>>,
    _permit: CheckpointCapacityPermit,
    _activity: CoordinatorActivityGuard,
    started_unix_nanos: u128,
    started: Instant,
    wal_bytes_before: u64,
    retained_bytes: u64,
    task_id: u64,
    target_lsn: u64,
    completed_writes: u64,
    preparation_nanos: u64,
    worker_active: Arc<AtomicBool>,
}

pub(super) struct PhysicalCoordinator {
    store: Option<Store>,
    settings: CoordinatorSettings,
    requests: mpsc::Receiver<QueuedBlinkTransaction>,
    pending: Option<QueuedBlinkTransaction>,
    queued: Vec<QueuedBlinkTransaction>,
    running: Option<RunningCheckpoint>,
    requested: bool,
    requested_since: Option<Instant>,
    requested_activity: Option<CoordinatorActivityGuard>,
}

impl PhysicalCoordinator {
    pub fn new(
        store: Store,
        settings: CoordinatorSettings,
        requests: mpsc::Receiver<QueuedBlinkTransaction>,
    ) -> Self {
        Self {
            store: Some(store),
            settings,
            requests,
            pending: None,
            queued: Vec::new(),
            running: None,
            requested: false,
            requested_since: None,
            requested_activity: None,
        }
    }

    pub async fn run(mut self) {
        if let Err(error) = self.run_loop().await {
            self.cancel_pending_checkpoint_request();
            let message = error.to_string();
            if let Ok(mut failed) = self.settings.failed_message.lock() {
                *failed = Some(message.clone());
            }
            self.metrics(|metrics| metrics.failed_message = Some(message.clone()));
            self.requests.close();
            if self.running.is_some() {
                let _ = self.finish(None).await;
            }
            for entry in self.queued.drain(..).chain(self.pending.take()) {
                let _ = entry
                    .response_tx
                    .send(Err(Error::durability(message.clone())));
            }
            while let Some(entry) = self.requests.recv().await {
                let _ = entry
                    .response_tx
                    .send(Err(Error::durability(message.clone())));
            }
        }
    }

    fn metrics(&self, update: impl FnOnce(&mut BlinkWalCheckpointMetrics)) {
        if let Ok(mut metrics) = self.settings.metrics.lock() {
            update(&mut metrics);
        }
    }

    fn request_checkpoint(&mut self) {
        if self.requested {
            self.metrics(|metrics| metrics.merged_checkpoint_requests += 1);
        } else {
            self.requested_since = Some(Instant::now());
            self.metrics(|metrics| {
                metrics.tenant_checkpoint_request_queued(self.settings.tenant_id)
            });
        }
        self.requested = true;
        if self.requested_activity.is_none() {
            self.requested_activity = Some(self.settings.activity.enter());
        }
    }

    fn cancel_pending_checkpoint_request(&mut self) {
        if !self.requested {
            return;
        }
        self.requested = false;
        self.requested_activity = None;
        let wait_nanos = self.requested_since.take().map_or(0, nanos);
        self.metrics(|metrics| {
            metrics.tenant_checkpoint_request_cancelled(self.settings.tenant_id, wait_nanos)
        });
    }

    fn scheduler() -> Arc<FairCheckpointScheduler> {
        Arc::clone(CHECKPOINT_CAPACITY.get_or_init(|| Arc::new(FairCheckpointScheduler::new())))
    }

    async fn run_loop(&mut self) -> Result<(), Error> {
        loop {
            if self
                .running
                .as_ref()
                .is_some_and(|running| running.worker.is_finished())
            {
                self.finish(None).await?;
            }
            if self.running.is_none() && self.requested {
                if self.settings.physical.execution == BlinkCheckpointExecution::Sync {
                    let permit = Self::scheduler().acquire(self.settings.tenant_id).await?;
                    self.start(permit).await?;
                } else if let Some(permit) = Self::scheduler().try_acquire(self.settings.tenant_id)
                {
                    self.start(permit).await?;
                }
            }
            if self.running.is_some()
                && self.settings.physical.execution == BlinkCheckpointExecution::Sync
            {
                self.finish(None).await?;
                continue;
            }
            let first = if let Some(entry) = self.pending.take() {
                Some(entry)
            } else if let Some(running) = self.running.as_mut() {
                tokio::select! {
                    biased;
                    result = &mut running.worker => {
                        self.finish(Some(result)).await?;
                        continue;
                    }
                    entry = self.requests.recv() => entry,
                }
            } else if self.requested {
                let scheduler = Self::scheduler();
                tokio::select! {
                    permit = scheduler.acquire(self.settings.tenant_id) => {
                        self.start(permit?).await?;
                        continue;
                    }
                    entry = self.requests.recv() => entry,
                }
            } else {
                self.requests.recv().await
            };
            let Some(first) = first else {
                if self.running.is_some() {
                    self.finish(None).await?;
                }
                if self.requested {
                    let permit = Self::scheduler().acquire(self.settings.tenant_id).await?;
                    self.start(permit).await?;
                    if self.running.is_some() {
                        self.finish(None).await?;
                    }
                }
                return Ok(());
            };
            let _activity = self.settings.activity.enter();
            self.collect(first).await;
            let mut retries = 0;
            loop {
                match self.apply_group().await {
                    Ok(()) => break,
                    Err(Error::Overloaded(message))
                        if message == "segmented checkpoint capacity exceeded"
                            && self.settings.policy != BlinkCheckpointPolicy::Disabled
                            && retries < 2 =>
                    {
                        retries += 1;
                        let started = Instant::now();
                        self.metrics(|metrics| metrics.capacity_wait_count += 1);
                        if self.running.is_some() {
                            self.finish(None).await?;
                        } else {
                            let permit = Self::scheduler().acquire(self.settings.tenant_id).await?;
                            self.start(permit).await?;
                            if self.running.is_some() {
                                self.finish(None).await?;
                            }
                        }
                        self.metrics(|metrics| {
                            metrics.capacity_wait_nanos =
                                metrics.capacity_wait_nanos.saturating_add(nanos(started));
                        });
                    }
                    Err(Error::Overloaded(message)) => {
                        for entry in self.queued.drain(..) {
                            let _ = entry
                                .response_tx
                                .send(Err(Error::overloaded(message.clone())));
                        }
                        break;
                    }
                    Err(error) => return Err(error),
                }
            }
        }
    }

    async fn collect(&mut self, first: QueuedBlinkTransaction) {
        let config = self.settings.grouping;
        let mut request_bytes = transaction_size(&first.request);
        self.queued.push(first);
        tokio::task::yield_now().await;
        while self.queued.len() < config.max_group_requests.max(1) {
            let Ok(next) = self.requests.try_recv() else {
                break;
            };
            let next_bytes = transaction_size(&next.request);
            if request_bytes.saturating_add(next_bytes) > config.max_group_bytes.max(1) {
                self.pending = Some(next);
                break;
            }
            request_bytes = request_bytes.saturating_add(next_bytes);
            self.queued.push(next);
        }
        if self.queued.len() > 1
            && self.pending.is_none()
            && config.max_collection_delay > std::time::Duration::ZERO
        {
            let deadline = tokio::time::sleep(config.max_collection_delay);
            tokio::pin!(deadline);
            while self.queued.len() < config.max_group_requests.max(1) {
                tokio::select! {
                    next = self.requests.recv() => {
                        let Some(next) = next else { break };
                        let next_bytes = transaction_size(&next.request);
                        if request_bytes.saturating_add(next_bytes) > config.max_group_bytes.max(1) {
                            self.pending = Some(next);
                            break;
                        }
                        request_bytes = request_bytes.saturating_add(next_bytes);
                        self.queued.push(next);
                    }
                    _ = &mut deadline => break,
                }
            }
        }
    }

    async fn apply_group(&mut self) -> Result<(), Error> {
        let requests = self
            .queued
            .iter()
            .map(|entry| entry.request.clone())
            .collect::<Vec<_>>();
        #[cfg(all(test, feature = "blink-demand-paging"))]
        let commit_test_gate = self.settings.commit_test_gate.clone();
        let mut store = self.store.take().unwrap();
        let (store, before, after, results) = tokio::task::spawn_blocking(move || {
            #[cfg(all(test, feature = "blink-demand-paging"))]
            super::wait_for_one_shot_test_gate(commit_test_gate);
            let before = store.wal_metrics().ok().flatten();
            let results = store.apply_transaction_group(&requests);
            let after = store.wal_metrics().ok().flatten();
            (store, before, after, results)
        })
        .await
        .map_err(|error| Error::durability(format!("Blink coordinator worker failed: {error}")))?;
        self.store = Some(store);
        let results = results?;
        let successful = results
            .iter()
            .filter(|result| {
                result
                    .as_ref()
                    .is_ok_and(|transaction| transaction.commit_lsn.is_some())
            })
            .count() as u64;
        let overlap = self.running.as_ref().is_some_and(|running| {
            running.worker_active.load(Ordering::Acquire) && !running.worker.is_finished()
        });
        if overlap && let Some(running) = self.running.as_mut() {
            running.completed_writes += successful;
        }
        let wal_bytes = after.as_ref().map_or(0, |wal| wal.wal_bytes);
        let tenant_id = self.settings.tenant_id;
        self.metrics(|metrics| {
            if metrics.wal_bytes_at_measurement_start.is_none() {
                metrics.wal_bytes_at_measurement_start = before.as_ref().map(|wal| wal.wal_bytes);
            }
            let syncs = after
                .as_ref()
                .map_or(0, |wal| wal.wal_syncs)
                .saturating_sub(before.as_ref().map_or(0, |wal| wal.wal_syncs));
            metrics.commit_wal_syncs = metrics.commit_wal_syncs.saturating_add(syncs);
            metrics.wal_syncs = metrics
                .commit_wal_syncs
                .saturating_add(metrics.checkpoint_wal_syncs);
            metrics.observe_tenant_wal_bytes(tenant_id, wal_bytes, unix_nanos());
            metrics.wal_bytes_at_measurement_end = Some(metrics.wal_bytes_current);
            metrics.successful_mutation_transactions += successful;
            if overlap {
                metrics.writes_during_checkpoint += successful;
            }
            if successful > 0 {
                metrics.mutation_commit_groups += 1;
                metrics.transaction_group_count += 1;
                let bucket = (successful as usize).min(64);
                if metrics.transaction_group_histogram.len() <= bucket {
                    metrics.transaction_group_histogram.resize(bucket + 1, 0);
                }
                metrics.transaction_group_histogram[bucket] += 1;
            }
        });
        if successful > 0
            && matches!(self.settings.policy,
            BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes } if wal_bytes >= threshold_bytes)
        {
            self.request_checkpoint();
        }
        for (entry, result) in self.queued.drain(..).zip(results) {
            let _ = entry.response_tx.send(result);
        }
        Ok(())
    }

    async fn start(&mut self, permit: CheckpointCapacityPermit) -> Result<(), Error> {
        let started = Instant::now();
        let started_unix_nanos = unix_nanos();
        let mut store = self.store.take().unwrap();
        let (store, snapshot, before, after) = tokio::task::spawn_blocking(move || {
            let before = store.wal_metrics().ok().flatten();
            let snapshot = store.prepare_segmented_checkpoint();
            let after = store.wal_metrics().ok().flatten();
            (store, snapshot, before, after)
        })
        .await
        .map_err(|error| Error::durability(format!("checkpoint preparation failed: {error}")))?;
        self.store = Some(store);
        let request_wait_nanos = self.requested_since.take().map_or(0, nanos);
        self.requested = false;
        self.metrics(|metrics| {
            metrics.tenant_checkpoint_request_started(self.settings.tenant_id, request_wait_nanos)
        });
        let snapshot = snapshot?;
        self.metrics(|metrics| {
            metrics.coordinator_pause_nanos = metrics
                .coordinator_pause_nanos
                .saturating_add(nanos(started));
            let syncs = after
                .as_ref()
                .map_or(0, |wal| wal.wal_syncs)
                .saturating_sub(before.as_ref().map_or(0, |wal| wal.wal_syncs));
            metrics.checkpoint_wal_syncs += syncs;
            metrics.wal_syncs = metrics
                .commit_wal_syncs
                .saturating_add(metrics.checkpoint_wal_syncs);
            if let Some(wal) = &after {
                metrics.observe_tenant_wal_bytes(
                    self.settings.tenant_id,
                    wal.wal_bytes,
                    unix_nanos(),
                );
                metrics.wal_bytes_at_measurement_end = Some(metrics.wal_bytes_current);
            }
        });
        let Some(mut snapshot) = snapshot else {
            self.requested_activity = None;
            return Ok(());
        };
        if let Some((factory, tenant, shard)) = &self.settings.physical.injector_factory {
            snapshot.set_boxed_fault_injector(factory(*tenant, *shard));
        }
        let retained_bytes = snapshot.retained_bytes();
        let wal_bytes_before = before.as_ref().map_or(0, |wal| wal.wal_bytes);
        self.metrics(|metrics| {
            metrics.checkpoints_started += 1;
            metrics.tasks_running += 1;
            metrics.peak_tasks_running = metrics.peak_tasks_running.max(metrics.tasks_running);
            metrics.retained_image_bytes += retained_bytes;
            metrics.peak_retained_image_bytes = metrics
                .peak_retained_image_bytes
                .max(metrics.retained_image_bytes);
            if let Some(threshold) = metrics.threshold_bytes {
                metrics.threshold_excess_max_bytes = metrics
                    .threshold_excess_max_bytes
                    .max(wal_bytes_before.saturating_sub(threshold));
            }
        });
        let target_lsn = snapshot.checkpoint_lsn().get();
        let preparation_nanos = nanos(started);
        let worker_active = Arc::new(AtomicBool::new(false));
        let worker_state = Arc::clone(&worker_active);
        let worker = submit_checkpoint_progress(move || {
            worker_state.store(true, Ordering::Release);
            let _worker_activity = CheckpointWorkerActivity(worker_state);
            snapshot.run()
        })?;
        self.running = Some(RunningCheckpoint {
            worker: tokio::spawn(async move {
                worker.await.map_err(|error| {
                    Error::durability(format!(
                        "checkpoint progress worker dropped its result: {error}"
                    ))
                })?
            }),
            _permit: permit,
            _activity: self.settings.activity.enter(),
            started,
            started_unix_nanos,
            wal_bytes_before,
            retained_bytes,
            task_id: TASK_SEQUENCE.fetch_add(1, Ordering::Relaxed),
            target_lsn,
            completed_writes: 0,
            preparation_nanos,
            worker_active,
        });
        self.requested_activity = None;
        Ok(())
    }

    async fn finish(&mut self, result: Option<WorkerResult>) -> Result<(), Error> {
        let running = self.running.take().unwrap();
        let worker_result = match result {
            Some(result) => result,
            None => running.worker.await,
        };
        let output = worker_result.unwrap_or_else(|error| {
            Err(Error::durability(format!(
                "checkpoint worker failed: {error}"
            )))
        });
        let phase_timings = output
            .as_ref()
            .map_or([0; 3], |output| output.phase_timings());
        let completion_started = Instant::now();
        let result = if let Some(mut store) = self.store.take() {
            match submit_checkpoint_progress(move || {
                let result = store.complete_segmented_checkpoint(output);
                let bytes = store
                    .wal_metrics()
                    .ok()
                    .flatten()
                    .map_or(0, |wal| wal.wal_bytes);
                (store, result, bytes)
            })?
            .await
            {
                Ok((store, report, bytes)) => {
                    self.store = Some(store);
                    report.map(|report| (report, bytes))
                }
                Err(error) => Err(Error::durability(format!(
                    "checkpoint completion failed: {error}"
                ))),
            }
        } else {
            Err(Error::durability("checkpoint lost its live store"))
        };
        let duration_nanos = nanos(running.started);
        self.metrics(|metrics| {
            metrics.tasks_running = metrics.tasks_running.saturating_sub(1);
            metrics.retained_image_bytes = metrics
                .retained_image_bytes
                .saturating_sub(running.retained_bytes);
            metrics.coordinator_pause_nanos = metrics
                .coordinator_pause_nanos
                .saturating_add(nanos(completion_started));
            metrics.checkpoint_total_nanos = metrics
                .checkpoint_total_nanos
                .saturating_add(duration_nanos);
            metrics.checkpoint_max_nanos = metrics.checkpoint_max_nanos.max(duration_nanos);
            let (report, bytes) = result
                .as_ref()
                .map_or((None, metrics.wal_bytes_current), |(report, bytes)| {
                    (Some(report), *bytes)
                });
            metrics.observe_tenant_wal_bytes(self.settings.tenant_id, bytes, unix_nanos());
            metrics.wal_bytes_at_measurement_end = Some(metrics.wal_bytes_current);
            if result.is_ok() {
                metrics.checkpoints_completed += 1;
                metrics.tenant_checkpoint_slice_completed(self.settings.tenant_id);
            } else {
                metrics.checkpoints_failed += 1;
            }
            metrics.record_event(BlinkCheckpointEvent {
                phase: "segmented_snapshot_data_sync_coordinator_manifest_reclaim".to_owned(),
                task_id: running.task_id,
                target_lsn: running.target_lsn,
                writes_during_checkpoint: running.completed_writes,
                preparation_nanos: running.preparation_nanos,
                page_write_nanos: phase_timings[0],
                data_sync_nanos: phase_timings[1],
                metadata_sync_nanos: phase_timings[2],
                completion_nanos: nanos(completion_started),
                started_unix_nanos: running.started_unix_nanos,
                ended_unix_nanos: unix_nanos(),
                wal_bytes_before: running.wal_bytes_before,
                wal_bytes_after: bytes,
                reclaimed_bytes: report.map_or(0, |report| report.wal_bytes_reclaimed),
                duration_nanos,
                checkpoint_bytes_written: report.map_or(0, |report| report.bytes_written),
                base_bytes_written: report.map_or(0, |report| report.base_bytes_written),
                base_bytes_retained: report.map_or(0, |report| report.base_bytes_retained),
                base_bytes_reclaimed: report.map_or(0, |report| report.base_bytes_reclaimed),
                full_base_snapshots: report.map_or(0, |report| report.full_base_snapshots),
                checkpoint_write_requests: report
                    .map_or(0, |report| report.checkpoint_write_requests),
                succeeded: result.is_ok(),
            });
        });
        if self.requested
            && let Ok((_, wal_bytes)) = &result
            && matches!(self.settings.policy,
                BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes } if *wal_bytes < threshold_bytes)
        {
            self.cancel_pending_checkpoint_request();
        }
        result.map(|_: (BlinkCheckpointReport, u64)| ())
    }
}

fn nanos(started: Instant) -> u64 {
    started.elapsed().as_nanos().min(u64::MAX as u128) as u64
}

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

#[cfg(test)]
mod tests {
    use super::{FairCheckpointScheduler, FairTenantReadyQueue, submit_checkpoint_progress};
    use std::sync::Arc;

    async fn wait_until_pending(scheduler: &FairCheckpointScheduler, tenant: u64) {
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let pending = scheduler.state.lock().unwrap().ready.is_pending(tenant);
                if pending {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }

    #[test]
    fn fair_ready_queue_deduplicates_tickets_and_bounds_turn_wait() {
        let mut queue = FairTenantReadyQueue::default();
        for tenant in [11, 22, 33, 44] {
            assert!(queue.enqueue(tenant));
        }
        assert!(!queue.enqueue(22));
        assert_eq!(queue.len(), 4);

        for expected_tenant in [11, 22, 33, 44, 11, 22, 33, 44] {
            let tenant = queue.take_next().unwrap();
            assert_eq!(tenant, expected_tenant);
            assert!(queue.enqueue(tenant));
        }
        assert_eq!(queue.len(), 4);
    }

    #[test]
    fn fair_ready_queue_drops_idle_failed_and_cancelled_tickets() {
        let mut queue = FairTenantReadyQueue::default();
        assert!(queue.enqueue(51));
        assert!(queue.enqueue(52));
        assert_eq!(queue.take_next(), Some(51));
        assert!(!queue.is_pending(51));
        assert_eq!(queue.take_next(), Some(52));
        assert!(queue.enqueue(52));
        assert!(queue.cancel(52));
        assert!(!queue.cancel(52));
        assert!(!queue.is_pending(52));
        assert_eq!(queue.len(), 0);
    }

    #[tokio::test]
    async fn shared_checkpoint_capacity_serves_waiters_in_ticket_order() {
        let scheduler = Arc::new(FairCheckpointScheduler::new());
        let running_permit = scheduler.try_acquire(61).unwrap();
        let second_scheduler = Arc::clone(&scheduler);
        let second_waiter =
            tokio::spawn(async move { second_scheduler.acquire(62).await.unwrap() });
        wait_until_pending(&scheduler, 62).await;
        let third_scheduler = Arc::clone(&scheduler);
        let third_waiter = tokio::spawn(async move { third_scheduler.acquire(63).await.unwrap() });
        wait_until_pending(&scheduler, 63).await;

        drop(running_permit);
        let second_permit = tokio::time::timeout(std::time::Duration::from_secs(2), second_waiter)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(second_permit.tenant, 62);
        drop(second_permit);
        let third_permit = tokio::time::timeout(std::time::Duration::from_secs(2), third_waiter)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(third_permit.tenant, 63);
    }

    #[tokio::test]
    async fn cancelled_checkpoint_waiter_releases_its_ready_ticket() {
        let scheduler = Arc::new(FairCheckpointScheduler::new());
        let running_permit = scheduler.try_acquire(71).unwrap();
        let cancelled_scheduler = Arc::clone(&scheduler);
        let cancelled_waiter =
            tokio::spawn(async move { cancelled_scheduler.acquire(72).await.unwrap() });
        wait_until_pending(&scheduler, 72).await;
        cancelled_waiter.abort();
        assert!(
            cancelled_waiter
                .await
                .is_err_and(|error| error.is_cancelled())
        );
        assert_eq!(scheduler.state.lock().unwrap().ready.len(), 0);

        let next_scheduler = Arc::clone(&scheduler);
        let next_waiter = tokio::spawn(async move { next_scheduler.acquire(73).await.unwrap() });
        wait_until_pending(&scheduler, 73).await;
        drop(running_permit);
        let next_permit = tokio::time::timeout(std::time::Duration::from_secs(2), next_waiter)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(next_permit.tenant, 73);
    }

    #[tokio::test]
    async fn checkpoint_continuation_returns_behind_other_ready_tenants() {
        let scheduler = Arc::new(FairCheckpointScheduler::new());
        let initial_permit = scheduler.try_acquire(81).unwrap();
        let first_scheduler = Arc::clone(&scheduler);
        let first_slice = tokio::spawn(async move { first_scheduler.acquire(82).await.unwrap() });
        wait_until_pending(&scheduler, 82).await;
        let second_scheduler = Arc::clone(&scheduler);
        let second_tenant_slice =
            tokio::spawn(async move { second_scheduler.acquire(83).await.unwrap() });
        wait_until_pending(&scheduler, 83).await;

        drop(initial_permit);
        let first_permit = tokio::time::timeout(std::time::Duration::from_secs(2), first_slice)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(first_permit.tenant, 82);
        let continuation_scheduler = Arc::clone(&scheduler);
        let continuation_slice =
            tokio::spawn(async move { continuation_scheduler.acquire(82).await.unwrap() });
        wait_until_pending(&scheduler, 82).await;

        drop(first_permit);
        let second_permit =
            tokio::time::timeout(std::time::Duration::from_secs(2), second_tenant_slice)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(second_permit.tenant, 83);
        drop(second_permit);
        let continuation_permit =
            tokio::time::timeout(std::time::Duration::from_secs(2), continuation_slice)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(continuation_permit.tenant, 82);
    }

    #[tokio::test]
    async fn failed_progress_job_releases_worker_and_checkpoint_capacity() {
        let scheduler = Arc::new(FairCheckpointScheduler::new());
        let first_permit = scheduler.try_acquire(91).unwrap();
        let failed_result = submit_checkpoint_progress(move || {
            drop(first_permit);
            Err::<(), _>(super::Error::durability("injected progress failure"))
        })
        .unwrap();
        assert!(failed_result.await.unwrap().is_err());

        let next_permit = scheduler.try_acquire(92).unwrap();
        assert_eq!(next_permit.tenant, 92);
        let next_result = submit_checkpoint_progress(|| 1u64).unwrap();
        assert_eq!(next_result.await.unwrap(), 1);
    }

    #[tokio::test]
    async fn panicking_progress_job_does_not_stop_the_dedicated_worker() {
        let failed_result =
            submit_checkpoint_progress(|| -> u64 { panic!("injected worker panic") }).unwrap();
        assert!(failed_result.await.is_err());
        let recovered_result = submit_checkpoint_progress(|| 2u64).unwrap();
        assert_eq!(recovered_result.await.unwrap(), 2);
    }
}
