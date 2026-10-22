use super::*;
use dodb_storage::{BlinkCheckpointOutput, BlinkCheckpointReport, BlinkStore};
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc;
use tokio::task::{JoinError, JoinHandle};

type Store = BlinkStore<ProductionFile, ProductionFile>;
type WorkerResult = Result<Result<BlinkCheckpointOutput, Error>, JoinError>;

static TASK_SEQUENCE: AtomicU64 = AtomicU64::new(1);

static CHECKPOINT_CAPACITY: OnceLock<Arc<Semaphore>> = OnceLock::new();

pub(super) struct PhysicalCheckpointConfig {
    pub execution: BlinkCheckpointExecution,
    pub injector_factory: Option<(FaultInjectorFactory, TenantId, ShardId)>,
}

pub(super) struct CoordinatorSettings {
    pub grouping: CoordinatorConfig,
    pub policy: BlinkCheckpointPolicy,
    pub physical: PhysicalCheckpointConfig,
    pub metrics: Arc<StdMutex<BlinkWalCheckpointMetrics>>,
    pub activity: Arc<CoordinatorActivity>,
    pub failed_message: Arc<StdMutex<Option<String>>>,
}

struct RunningCheckpoint {
    worker: JoinHandle<Result<BlinkCheckpointOutput, Error>>,
    _permit: OwnedSemaphorePermit,
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
            requested_activity: None,
        }
    }

    pub async fn run(mut self) {
        if let Err(error) = self.run_loop().await {
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
        }
        self.requested = true;
        if self.requested_activity.is_none() {
            self.requested_activity = Some(self.settings.activity.enter());
        }
    }

    fn semaphore() -> Arc<Semaphore> {
        Arc::clone(CHECKPOINT_CAPACITY.get_or_init(|| Arc::new(Semaphore::new(1))))
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
                    let permit = Self::semaphore()
                        .acquire_owned()
                        .await
                        .map_err(|_| Error::durability("checkpoint capacity closed"))?;
                    self.start(permit).await?;
                } else if let Ok(permit) = Self::semaphore().try_acquire_owned() {
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
                tokio::select! {
                    permit = Self::semaphore().acquire_owned() => {
                        self.start(permit.map_err(|_| Error::durability("checkpoint capacity closed"))?).await?;
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
                    let permit = Self::semaphore()
                        .acquire_owned()
                        .await
                        .map_err(|_| Error::durability("checkpoint capacity closed"))?;
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
                            let permit = Self::semaphore()
                                .acquire_owned()
                                .await
                                .map_err(|_| Error::durability("checkpoint capacity closed"))?;
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
        let mut store = self.store.take().unwrap();
        let (store, before, after, results) = tokio::task::spawn_blocking(move || {
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
            metrics.wal_bytes_current = wal_bytes;
            metrics.wal_bytes_at_measurement_end = Some(wal_bytes);
            metrics.wal_bytes_max = metrics.wal_bytes_max.max(wal_bytes);
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

    async fn start(&mut self, permit: OwnedSemaphorePermit) -> Result<(), Error> {
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
        self.requested = false;
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
                metrics.wal_bytes_current = wal.wal_bytes;
                metrics.wal_bytes_at_measurement_end = Some(wal.wal_bytes);
                metrics.wal_bytes_max = metrics.wal_bytes_max.max(wal.wal_bytes);
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
        self.running = Some(RunningCheckpoint {
            worker: tokio::task::spawn_blocking(move || {
                worker_state.store(true, Ordering::Release);
                let result = snapshot.run();
                worker_state.store(false, Ordering::Release);
                result
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
            match tokio::task::spawn_blocking(move || {
                let result = store.complete_segmented_checkpoint(output);
                let bytes = store
                    .wal_metrics()
                    .ok()
                    .flatten()
                    .map_or(0, |wal| wal.wal_bytes);
                (store, result, bytes)
            })
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
            metrics.wal_bytes_current = bytes;
            metrics.wal_bytes_at_measurement_end = Some(bytes);
            metrics.wal_bytes_max = metrics.wal_bytes_max.max(bytes);
            if result.is_ok() {
                metrics.checkpoints_completed += 1;
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
                succeeded: result.is_ok(),
            });
        });
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
