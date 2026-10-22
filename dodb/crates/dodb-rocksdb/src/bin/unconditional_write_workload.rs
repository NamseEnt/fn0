#[path = "support/blink_checkpoint.rs"]
mod blink_checkpoint;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use dodb_core::{DocumentKey, Revision, TenantId, TransactionMutation, TransactionRequest};
use dodb_rocksdb::raw_research::RawRocksDb;
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use serde_json::{Value, json};

const INITIAL_VALUE_SIZE: usize = 512;
const POOL_SLOTS_PER_CLIENT: usize = 256;
const PREPOPULATE_BATCH_SIZE: usize = 1_024;
const INPUT_PREFIX_BATCH_LIMIT: u64 = 100;
const MEASUREMENT_PAYLOAD_SEQUENCE_BASE: u64 = 1_u64 << 56;
const FNV1A64_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;
const EXACT_HISTOGRAM_BUCKETS: usize = 1_024;
const SUB_BUCKETS_PER_EXPONENT: usize = 1_024;
const HISTOGRAM_BUCKETS: usize =
    EXACT_HISTOGRAM_BUCKETS + (u64::BITS as usize - 10) * SUB_BUCKETS_PER_EXPONENT;
const TENANT_NUMBER: u64 = 1;
const RESPONSE_BUDGET_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BackendKind {
    RawRocksDb,
    SynchronousCore,
    RocksDbService,
    Blink,
    MinimalNative,
}

struct Options {
    backend: BackendKind,
    blink_checkpoint_policy: dodb_server::BlinkCheckpointPolicy,
    blink_checkpoint: blink_checkpoint::Options,
    width: usize,
    clients: usize,
    seed: u64,
    value_size: usize,
    warmup_ms: u64,
    measurement_ms: u64,
    call_cap: u64,
    validate_histogram: bool,
    revision_allocator: dodb_rocksdb::RevisionAllocatorMode,
    condition_key_lock_mode: dodb_rocksdb::ConditionKeyLockMode,
    engine_health_interval_ms: u64,
    shared_blocking_slots: usize,
    data_dir: PathBuf,
    report_path: PathBuf,
    memory_diagnostic: bool,
    source_revision: Option<String>,
    memory_worker_startup_timeout_ms: u64,
}

struct SampleHistogram {
    bucket_counts: Vec<u64>,
    observation_count: u64,
    maximum_nanos: u64,
    exact_validation_samples: Option<Vec<u64>>,
}

enum ProbeBackend {
    Raw(RawRocksDb),
    Core(dodb_rocksdb::research::SynchronousRocksDbTenant),
    RocksDbService(Box<dodb_rocksdb::RocksDbService>, tokio::runtime::Handle),
    Service(
        Box<dyn DodbService>,
        tokio::runtime::Handle,
        Option<dodb_server::BlinkWalCheckpointMetricsHandle>,
    ),
    Diagnostic {
        backend_kind: BackendKind,
        worker: dodb_rocksdb::memory_diagnostic::MemoryDiagnosticWorker,
        service: dodb_rocksdb::memory_diagnostic::RemoteDodbService,
        runtime: tokio::runtime::Handle,
    },
}

struct ClientResult {
    slot_write_counts: Vec<u64>,
    latest_payload_sequences: Vec<u64>,
    latest_ack_revisions: Vec<u64>,
    input_prefix_fingerprint: u64,
    input_prefix_batches: u64,
    warmup_writes: u64,
    measurement_writes: u64,
    width_keys_acknowledged: u64,
    histogram: SampleHistogram,
}

struct PhaseResult {
    client_results: Vec<ClientResult>,
    actual_duration_ms: u64,
}

struct ProcessSample {
    elapsed_ms: u64,
    rss_kb: u64,
    cpu_ticks: u64,
    database_bytes: u64,
    wal_bytes: u64,
}

struct ProcessSampler {
    stopped: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<ProcessSample>>>,
    omitted_samples: Arc<AtomicU64>,
    started: Instant,
    worker: Option<thread::JoinHandle<()>>,
}

impl SampleHistogram {
    fn new(validate_histogram: bool) -> Self {
        Self {
            bucket_counts: vec![0; HISTOGRAM_BUCKETS],
            observation_count: 0,
            maximum_nanos: 0,
            exact_validation_samples: validate_histogram.then(Vec::new),
        }
    }

    fn record(&mut self, elapsed_nanos: u64) {
        let bucket_index = histogram_bucket_index(elapsed_nanos);
        self.bucket_counts[bucket_index] = self.bucket_counts[bucket_index].saturating_add(1);
        self.observation_count = self.observation_count.saturating_add(1);
        self.maximum_nanos = self.maximum_nanos.max(elapsed_nanos);
        if let Some(samples) = &mut self.exact_validation_samples {
            samples.push(elapsed_nanos);
        }
    }

    fn merge(&mut self, other: &Self) {
        for (bucket_count, additional_count) in
            self.bucket_counts.iter_mut().zip(&other.bucket_counts)
        {
            *bucket_count = bucket_count.saturating_add(*additional_count);
        }
        self.observation_count = self
            .observation_count
            .saturating_add(other.observation_count);
        self.maximum_nanos = self.maximum_nanos.max(other.maximum_nanos);
        match (
            &mut self.exact_validation_samples,
            &other.exact_validation_samples,
        ) {
            (Some(samples), Some(other_samples)) => samples.extend_from_slice(other_samples),
            (None, None) => {}
            _ => unreachable!("histogram validation mode must match while merging clients"),
        }
    }

    fn percentile_nanos(&self, percentile: u64) -> u64 {
        if self.observation_count == 0 {
            return 0;
        }
        let target_rank = self
            .observation_count
            .saturating_mul(percentile)
            .div_ceil(100)
            .max(1);
        let mut cumulative_count = 0_u64;
        for (bucket_index, bucket_count) in self.bucket_counts.iter().enumerate() {
            cumulative_count = cumulative_count.saturating_add(*bucket_count);
            if cumulative_count >= target_rank {
                return histogram_bucket_upper_bound(bucket_index as u64);
            }
        }
        self.maximum_nanos
    }

    fn json(&self) -> Value {
        let exact_validation = self.exact_validation_samples.as_ref().map(|samples| {
            sample_histogram_validation_report(samples, &self.bucket_counts, self.observation_count)
        });
        let nonzero_bins = self
            .bucket_counts
            .iter()
            .enumerate()
            .filter(|(_, count)| **count > 0)
            .map(|(bucket_index, count)| json!([bucket_index, count]))
            .collect::<Vec<_>>();
        json!({
            "storage": if exact_validation.is_some() { "log1024-with-exact-validation" } else { "log1024" },
            "sampling": "all acknowledged batch completions counted in fixed buckets",
            "percentile_value": "bucket upper bound in nanoseconds",
            "histogram_schema_version": 1,
            "histogram_binning_schema": "exact integer nanoseconds below 1024; 1024 equal-width sub-buckets per binary exponent thereafter; percentile is the bucket upper bound",
            "nonzero_bins": nonzero_bins,
            "maximum_relative_bucket_width": 0.0009765625,
            "count": self.observation_count,
            "sample_count": self.observation_count,
            "maximum_nanos": self.maximum_nanos,
            "p50_nanos": self.percentile_nanos(50),
            "p95_nanos": self.percentile_nanos(95),
            "p99_nanos": self.percentile_nanos(99),
            "bucket_count": self.bucket_counts.len(),
            "histogram_capacity_bytes": self.bucket_counts.capacity() * std::mem::size_of::<u64>(),
            "exact_validation_sample_count": self.exact_validation_samples.as_ref().map(Vec::len),
            "exact_validation_sample_capacity_bytes": self.exact_validation_samples.as_ref().map_or(0, |samples| samples.capacity() * std::mem::size_of::<u64>()),
            "exact_sample_validation": exact_validation,
        })
    }
}

impl ProcessSampler {
    fn start(data_dir: PathBuf) -> Result<Self, String> {
        let stopped = Arc::new(AtomicBool::new(false));
        let samples = Arc::new(Mutex::new(Vec::new()));
        let omitted_samples = Arc::new(AtomicU64::new(0));
        let worker_stopped = Arc::clone(&stopped);
        let worker_samples = Arc::clone(&samples);
        let worker_omitted_samples = Arc::clone(&omitted_samples);
        let started = Instant::now();
        let worker = thread::spawn(move || {
            while !worker_stopped.load(Ordering::Relaxed) {
                if let Some((rss_kb, cpu_ticks)) = read_process_sample() {
                    let (database_bytes, wal_bytes) = database_and_wal_bytes(&data_dir);
                    if let Ok(mut sample_values) = worker_samples.lock() {
                        if sample_values.len() < 1_024 {
                            sample_values.push(ProcessSample {
                                elapsed_ms: started.elapsed().as_millis().min(u64::MAX as u128)
                                    as u64,
                                rss_kb,
                                cpu_ticks,
                                database_bytes,
                                wal_bytes,
                            });
                        } else {
                            worker_omitted_samples.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(200));
            }
            if let Some((rss_kb, cpu_ticks)) = read_process_sample() {
                let (database_bytes, wal_bytes) = database_and_wal_bytes(&data_dir);
                if let Ok(mut sample_values) = worker_samples.lock() {
                    if sample_values.len() < 1_024 {
                        sample_values.push(ProcessSample {
                            elapsed_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
                            rss_kb,
                            cpu_ticks,
                            database_bytes,
                            wal_bytes,
                        });
                    } else {
                        worker_omitted_samples.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
        Ok(Self {
            stopped,
            samples,
            omitted_samples,
            started,
            worker: Some(worker),
        })
    }

    fn finish(mut self) -> Result<Value, String> {
        self.stopped.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            worker
                .join()
                .map_err(|_| "process sampler panicked".to_owned())?;
        }
        let sample_values = self
            .samples
            .lock()
            .map_err(|_| "process sample lock poisoned".to_owned())?;
        let peak_rss_kb = sample_values
            .iter()
            .map(|sample| sample.rss_kb)
            .max()
            .unwrap_or(0);
        let first_cpu_ticks = sample_values
            .first()
            .map(|sample| sample.cpu_ticks)
            .unwrap_or(0);
        let last_cpu_ticks = sample_values
            .last()
            .map(|sample| sample.cpu_ticks)
            .unwrap_or(first_cpu_ticks);
        let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as u64;
        let cpu_seconds =
            last_cpu_ticks.saturating_sub(first_cpu_ticks) as f64 / ticks_per_second as f64;
        let samples = sample_values
            .iter()
            .map(|sample| json!({"elapsed_ms": sample.elapsed_ms, "rss_kb": sample.rss_kb, "cpu_ticks": sample.cpu_ticks, "database_file_bytes": sample.database_bytes, "wal_file_bytes": sample.wal_bytes}))
            .collect::<Vec<_>>();
        if sample_values.is_empty() || peak_rss_kb == 0 {
            return Err("process RSS/CPU samples are unavailable".to_owned());
        }
        Ok(json!({
            "sampling_interval_ms": 200,
            "sampling_scope": "measurement phase including admission and drain; warmup and final validation excluded",
            "sampling_completeness": "complete",
            "rss_sample_count": sample_values.len(),
            "omitted_resource_samples": self.omitted_samples.load(Ordering::Relaxed),
            "peak_sampled_rss_kb": peak_rss_kb,
            "cpu_seconds": cpu_seconds,
            "elapsed_ms": self.started.elapsed().as_millis(),
            "samples": samples,
        }))
    }
}

fn read_process_sample() -> Option<(u64, u64)> {
    let status_text = std::fs::read_to_string("/proc/self/status").ok()?;
    let rss_kb = status_text
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmRSS:")
                .and_then(|value| value.split_whitespace().next())
        })?
        .parse::<u64>()
        .ok()?;
    let stat_text = std::fs::read_to_string("/proc/self/stat").ok()?;
    let process_fields = stat_text
        .rsplit_once(')')?
        .1
        .split_whitespace()
        .collect::<Vec<_>>();
    let user_cpu_ticks = process_fields.get(11)?.parse::<u64>().ok()?;
    let system_cpu_ticks = process_fields.get(12)?.parse::<u64>().ok()?;
    Some((rss_kb, user_cpu_ticks.saturating_add(system_cpu_ticks)))
}

impl ProbeBackend {
    fn open(
        options: &Options,
        runtime: tokio::runtime::Handle,
        memory_session: Option<Arc<dodb_rocksdb::memory_diagnostic::MemoryDiagnosticSession>>,
    ) -> Result<Self, String> {
        if let Some(memory_session) = memory_session {
            let backend = match options.backend {
                BackendKind::RawRocksDb => "raw",
                BackendKind::SynchronousCore => "core",
                BackendKind::RocksDbService => "rocksdb-service",
                BackendKind::Blink => "blink",
                BackendKind::MinimalNative => "minimal-native",
            };
            if options.source_revision.is_none() {
                return Err("memory diagnostic requires --source-revision".to_owned());
            }
            let worker_arguments = vec![
                "--backend".to_owned(),
                backend.to_owned(),
                "--revision-allocator".to_owned(),
                match options.revision_allocator {
                    dodb_rocksdb::RevisionAllocatorMode::Serialized => "serialized".to_owned(),
                    dodb_rocksdb::RevisionAllocatorMode::ReservedRanges => {
                        "reserved-ranges".to_owned()
                    }
                },
                "--condition-key-lock-mode".to_owned(),
                match options.condition_key_lock_mode {
                    dodb_rocksdb::ConditionKeyLockMode::Exclusive => "exclusive".to_owned(),
                    dodb_rocksdb::ConditionKeyLockMode::Shared => "shared".to_owned(),
                },
                "--shared-blocking-slots".to_owned(),
                options.shared_blocking_slots.to_string(),
                "--engine-health-interval-ms".to_owned(),
                options.engine_health_interval_ms.to_string(),
                "--memory-tenant-count".to_owned(),
                "1".to_owned(),
                "--memory-core-tenant-subdirectories".to_owned(),
                "0".to_owned(),
            ];
            let worker_executable = std::env::current_exe()
                .map_err(|error| error.to_string())?
                .with_file_name("memory_diagnostic_worker");
            let worker = runtime.block_on(memory_session.spawn_worker(
                &worker_executable,
                &worker_arguments,
                RESPONSE_BUDGET_BYTES,
                PREPOPULATE_BATCH_SIZE,
                options.clients,
                &options.data_dir,
                Duration::from_millis(options.memory_worker_startup_timeout_ms),
            ))?;
            let service = dodb_rocksdb::memory_diagnostic::RemoteDodbService::new(
                worker.connection().clone(),
            );
            return Ok(Self::Diagnostic {
                backend_kind: options.backend,
                worker,
                service,
                runtime,
            });
        }
        match options.backend {
            BackendKind::RawRocksDb => RawRocksDb::open(&options.data_dir)
                .map(Self::Raw)
                .map_err(|error| error.to_string()),
            BackendKind::SynchronousCore => {
                let database = dodb_rocksdb::research::SynchronousRocksDbTenant::open_with_condition_key_lock_mode(
                    &options.data_dir,
                    options.revision_allocator,
                    options.condition_key_lock_mode,
                )
                .map_err(|error| error.to_string())?;
                Ok(Self::Core(database))
            }
            BackendKind::RocksDbService => {
                let mut config = dodb_rocksdb::RocksDbServiceConfig::new(&options.data_dir);
                config.revision_allocator = options.revision_allocator;
                config.condition_key_lock_mode = options.condition_key_lock_mode;
                config.max_blocking_operations = options.shared_blocking_slots;
                config.detailed_write_metrics = false;
                Ok(Self::RocksDbService(
                    Box::new(
                        dodb_rocksdb::RocksDbService::new(config)
                            .map_err(|error| error.to_string())?,
                    ),
                    runtime,
                ))
            }
            BackendKind::Blink => {
                let mut config = dodb_server::LocalTenantServiceConfig::default();
                config.data_dir = options.data_dir.clone();
                config.max_open_shards = 1_024;
                config.blink_checkpoint_policy = options.blink_checkpoint_policy;
                config.blink_wal_layout = options.blink_checkpoint.layout;
                config.blink_checkpoint_execution = options.blink_checkpoint.execution;
                let service = dodb_server::BlinkTenantService::new(config)
                    .map_err(|error| error.to_string())?;
                let metrics = service.checkpoint_metrics_handle();
                Ok(Self::Service(Box::new(service), runtime, Some(metrics)))
            }
            BackendKind::MinimalNative => {
                let service = dodb_rocksdb::minimal_native::MinimalNativeService::new(
                    dodb_rocksdb::minimal_native::MinimalNativeConfig::new(&options.data_dir),
                )
                .map_err(|error| error.to_string())?;
                Ok(Self::Service(Box::new(service), runtime, None))
            }
        }
    }

    fn write(&self, writes: &[(DocumentKey, Vec<u8>)]) -> Result<Option<Revision>, String> {
        match self {
            Self::Raw(database) => {
                let entries = writes
                    .iter()
                    .map(|(key, value)| (physical_key(key), raw_envelope(value)))
                    .collect::<Vec<_>>();
                database
                    .write_batch(&entries)
                    .map_err(|error| error.to_string())?;
                Ok(None)
            }
            Self::Core(database) => {
                let mutations = writes
                    .iter()
                    .map(|(key, value)| TransactionMutation::Put {
                        key: key.clone(),
                        value: value.clone(),
                    })
                    .collect();
                let request = Request::Transact {
                    request: TransactionRequest::new(Vec::new(), mutations),
                };
                match database
                    .execute(request, ExecutionBudget::new(RESPONSE_BUDGET_BYTES))
                    .map_err(|error| error.to_string())?
                {
                    Response::Transact(outcome) if outcome.commit_lsn.is_some() => {
                        Ok(outcome.commit_lsn.map(Revision::from))
                    }
                    response => Err(format!(
                        "unexpected synchronous core response: {response:?}"
                    )),
                }
            }
            Self::Service(service, runtime, _) => {
                let mutations = writes
                    .iter()
                    .map(|(key, value)| TransactionMutation::Put {
                        key: key.clone(),
                        value: value.clone(),
                    })
                    .collect();
                let request = Request::Transact {
                    request: TransactionRequest::new(Vec::new(), mutations),
                };
                let response = runtime
                    .block_on(service.execute(
                        TenantId::new(TENANT_NUMBER),
                        request,
                        ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
                    ))
                    .map_err(|error| error.to_string())?;
                match response {
                    Response::Transact(outcome) if outcome.commit_lsn.is_some() => {
                        Ok(outcome.commit_lsn.map(Revision::from))
                    }
                    other => Err(format!("unexpected service response: {other:?}")),
                }
            }
            Self::RocksDbService(service, runtime) => {
                let mutations = writes
                    .iter()
                    .map(|(key, value)| TransactionMutation::Put {
                        key: key.clone(),
                        value: value.clone(),
                    })
                    .collect();
                let request = Request::Transact {
                    request: TransactionRequest::new(Vec::new(), mutations),
                };
                let response = runtime
                    .block_on(service.execute(
                        TenantId::new(TENANT_NUMBER),
                        request,
                        ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
                    ))
                    .map_err(|error| error.to_string())?;
                match response {
                    Response::Transact(outcome) if outcome.commit_lsn.is_some() => {
                        Ok(outcome.commit_lsn.map(Revision::from))
                    }
                    other => Err(format!("unexpected RocksDB service response: {other:?}")),
                }
            }
            Self::Diagnostic {
                backend_kind,
                service,
                runtime,
                ..
            } => {
                let mutations = writes
                    .iter()
                    .map(|(key, value)| TransactionMutation::Put {
                        key: key.clone(),
                        value: value.clone(),
                    })
                    .collect();
                let request = Request::Transact {
                    request: TransactionRequest::new(Vec::new(), mutations),
                };
                let response = runtime
                    .block_on(service.execute(
                        TenantId::new(TENANT_NUMBER),
                        request,
                        ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
                    ))
                    .map_err(|error| error.to_string())?;
                match (backend_kind, response) {
                    (BackendKind::RawRocksDb, Response::Transact(outcome))
                        if outcome.commit_lsn.is_none() =>
                    {
                        Ok(None)
                    }
                    (_, Response::Transact(outcome)) if outcome.commit_lsn.is_some() => {
                        Ok(outcome.commit_lsn.map(Revision::from))
                    }
                    (_, other) => Err(format!("unexpected diagnostic worker response: {other:?}")),
                }
            }
        }
    }

    fn read_all(&self) -> Result<Vec<(DocumentKey, Vec<u8>, Option<Revision>)>, String> {
        match self {
            Self::Raw(database) => database
                .scan()
                .map_err(|error| error.to_string())?
                .into_iter()
                .map(|(key_bytes, encoded_value)| decode_raw_entry(&key_bytes, &encoded_value))
                .collect(),
            Self::Core(database) => {
                scan_service(|request, budget| database.execute(request, budget))
                    .map_err(|error| error.to_string())
            }
            Self::Service(service, runtime, _) => scan_service(|request, budget| {
                runtime.block_on(service.execute(TenantId::new(TENANT_NUMBER), request, budget))
            })
            .map_err(|error| error.to_string()),
            Self::RocksDbService(service, runtime) => scan_service(|request, budget| {
                runtime.block_on(service.execute(TenantId::new(TENANT_NUMBER), request, budget))
            })
            .map_err(|error| error.to_string()),
            Self::Diagnostic {
                backend_kind,
                service,
                runtime,
                ..
            } => {
                let mut rows = scan_service(|request, budget| {
                    runtime.block_on(service.execute(TenantId::new(TENANT_NUMBER), request, budget))
                })
                .map_err(|error| error.to_string())?;
                if *backend_kind == BackendKind::RawRocksDb {
                    for (_, _, revision) in &mut rows {
                        *revision = None;
                    }
                }
                Ok(rows)
            }
        }
    }

    fn shutdown(&self) -> Result<(), String> {
        match self {
            Self::Service(service, runtime, _) => runtime.block_on(service.shutdown()),
            Self::RocksDbService(service, runtime) => runtime.block_on(service.shutdown()),
            Self::Diagnostic {
                worker, service, ..
            } => {
                service.connection().close();
                worker.shutdown().map_err(|error| error.to_string())?;
            }
            Self::Raw(_) | Self::Core(_) => {}
        }
        Ok(())
    }

    fn engine_health_observer(&self) -> Option<dodb_rocksdb::EngineHealthObserver> {
        match self {
            Self::Raw(database) => Some(database.engine_health_observer()),
            Self::Core(database) => Some(database.engine_health_observer()),
            Self::RocksDbService(service, _) => Some(service.engine_health_observer()),
            Self::Diagnostic { .. } => None,
            Self::Service(_, _, _) => None,
        }
    }

    fn start_measurement(&self) -> Result<(), String> {
        if let Self::Diagnostic { worker, .. } = self {
            worker.start_measurement()?;
        }
        Ok(())
    }

    fn stop_measurement(&self) -> Result<(), String> {
        if let Self::Diagnostic { worker, .. } = self {
            worker.stop_measurement()?;
        }
        Ok(())
    }

    fn reset_blink_checkpoint_metrics(&self) {
        if let Self::Service(_, _, Some(metrics)) = self {
            metrics.reset();
        }
    }

    fn wait_for_blink_coordinators_idle(&self) {
        if let Self::Service(_, runtime, Some(metrics)) = self {
            runtime.block_on(metrics.wait_for_coordinators_idle());
        }
    }

    fn blink_checkpoint_metrics(&self) -> Value {
        let Self::Service(_, _, Some(metrics)) = self else {
            return serde_json::json!({"enabled": false});
        };
        let metrics = metrics.snapshot();
        serde_json::json!({
            "enabled": matches!(metrics.policy, dodb_server::BlinkCheckpointPolicy::WalBytesThreshold { .. }),
            "policy": match metrics.policy {
                dodb_server::BlinkCheckpointPolicy::Disabled => "disabled",
                dodb_server::BlinkCheckpointPolicy::WalBytesThreshold { .. } => "wal_bytes_threshold",
            },
            "threshold_bytes": metrics.threshold_bytes,
            "execution": match metrics.execution { dodb_server::BlinkCheckpointExecution::Sync => "sync", dodb_server::BlinkCheckpointExecution::Background => "background" },
            "writes_during_checkpoint": metrics.writes_during_checkpoint,
            "merged_checkpoint_requests": metrics.merged_checkpoint_requests,
            "capacity_wait_count": metrics.capacity_wait_count,
            "capacity_wait_nanos": metrics.capacity_wait_nanos,
            "coordinator_pause_nanos": metrics.coordinator_pause_nanos,
            "retained_image_bytes": metrics.retained_image_bytes,
            "peak_retained_image_bytes": metrics.peak_retained_image_bytes,
            "tasks_running": metrics.tasks_running,
            "peak_tasks_running": metrics.peak_tasks_running,
            "checkpoints_started": metrics.checkpoints_started,
            "checkpoints_completed": metrics.checkpoints_completed,
            "checkpoints_failed": metrics.checkpoints_failed,
            "checkpoint_total_nanos": metrics.checkpoint_total_nanos,
            "checkpoint_max_nanos": metrics.checkpoint_max_nanos,
            "wal_bytes_current": metrics.wal_bytes_current,
            "wal_bytes_at_measurement_start": metrics.wal_bytes_at_measurement_start,
            "wal_bytes_at_measurement_end": metrics.wal_bytes_at_measurement_end,
            "wal_bytes_max": metrics.wal_bytes_max,
            "threshold_excess_max_bytes": metrics.threshold_excess_max_bytes,
            "successful_mutation_transactions": metrics.successful_mutation_transactions,
            "mutation_commit_groups": metrics.mutation_commit_groups,
            "transaction_group_histogram": metrics.transaction_group_histogram,
            "transaction_group_count": metrics.transaction_group_count,
            "wal_syncs": metrics.wal_syncs,
            "commit_wal_syncs": metrics.commit_wal_syncs,
            "checkpoint_wal_syncs": metrics.checkpoint_wal_syncs,
            "omitted_checkpoint_events": metrics.omitted_checkpoint_events,
            "checkpoint_events": metrics.checkpoint_events.iter().map(|event| serde_json::json!({
                "phase": event.phase,
                    "task_id": event.task_id,
                    "target_lsn": event.target_lsn,
                    "writes_during_checkpoint": event.writes_during_checkpoint,
                    "preparation_nanos": event.preparation_nanos,
                    "page_write_nanos": event.page_write_nanos,
                    "data_sync_nanos": event.data_sync_nanos,
                    "metadata_sync_nanos": event.metadata_sync_nanos,
                    "completion_nanos": event.completion_nanos,
                "started_unix_nanos": event.started_unix_nanos.to_string(),
                "ended_unix_nanos": event.ended_unix_nanos.to_string(),
                "wal_bytes_before": event.wal_bytes_before,
                "wal_bytes_after": event.wal_bytes_after,
                "reclaimed_bytes": event.reclaimed_bytes,
                "duration_nanos": event.duration_nanos,
                "succeeded": event.succeeded,
            })).collect::<Vec<_>>(),
            "failed_message": metrics.failed_message,
        })
    }

    fn residency_scan(&self) -> Result<serde_json::Value, String> {
        match self {
            Self::Diagnostic {
                worker, runtime, ..
            } => runtime.block_on(dodb_rocksdb::memory_diagnostic::bounded_scan_digest(
                worker.connection(),
                TenantId::new(TENANT_NUMBER),
                RESPONSE_BUDGET_BYTES,
            )),
            _ => Err("residency scans require the opt-in diagnostic worker".to_owned()),
        }
    }
}

fn scan_service<F>(mut execute: F) -> Result<Vec<(DocumentKey, Vec<u8>, Option<Revision>)>, String>
where
    F: FnMut(Request, ExecutionBudget) -> Result<Response, dodb_core::Error>,
{
    let mut documents = Vec::new();
    let mut exclusive_after_key = None;
    loop {
        let response = execute(
            Request::Scan {
                exclusive_after_key: exclusive_after_key.clone(),
                limit: 1_024,
            },
            ExecutionBudget::new(RESPONSE_BUDGET_BYTES),
        )
        .map_err(|error| error.to_string())?;
        let Response::Scan(page) = response else {
            return Err(format!("unexpected scan response: {response:?}"));
        };
        if page.is_empty() {
            break;
        }
        exclusive_after_key = page.last().map(|document| document.key.clone());
        documents.extend(
            page.into_iter()
                .map(|document| (document.key, document.value, Some(document.revision))),
        );
    }
    Ok(documents)
}

fn parse_options() -> Result<Options, String> {
    let mut arguments = std::env::args().skip(1);
    let mut backend = None;
    let mut blink_checkpoint_policy = dodb_server::BlinkCheckpointPolicy::Disabled;
    let mut blink_checkpoint_option_seen = false;
    let mut checkpoint_values = BTreeMap::new();
    let mut width = 1;
    let mut clients = 16;
    let mut seed = 20261013;
    let mut value_size = INITIAL_VALUE_SIZE;
    let mut warmup_ms = 1_000;
    let mut measurement_ms = 5_000;
    let mut call_cap = 100_000_000;
    let mut validate_histogram = false;
    let mut revision_allocator = dodb_rocksdb::RevisionAllocatorMode::Serialized;
    let mut condition_key_lock_mode = dodb_rocksdb::ConditionKeyLockMode::Exclusive;
    let mut engine_health_interval_ms = 0_u64;
    let mut shared_blocking_slots = 8;
    let mut data_dir = None;
    let mut report_path = None;
    let mut memory_diagnostic = false;
    let mut source_revision = None;
    let mut memory_worker_startup_timeout_ms =
        dodb_rocksdb::memory_diagnostic::DEFAULT_WORKER_START_TIMEOUT_MS;
    while let Some(argument) = arguments.next() {
        let value = arguments
            .next()
            .ok_or_else(|| format!("missing value for {argument}"))?;
        match argument.as_str() {
            "--backend" => {
                backend = Some(match value.as_str() {
                    "raw-rocksdb" => BackendKind::RawRocksDb,
                    "synchronous-core" => BackendKind::SynchronousCore,
                    "rocksdb-service" => BackendKind::RocksDbService,
                    "blink" => BackendKind::Blink,
                    "minimal-native" => BackendKind::MinimalNative,
                    _ => return Err(format!("unknown backend: {value}")),
                });
            }
            "--blink-wal-layout" | "--blink-checkpoint-execution" | "--blink-wal-segment-bytes" => {
                checkpoint_values.insert(argument, value);
            }
            "--blink-checkpoint-wal-bytes" => {
                blink_checkpoint_option_seen = true;
                blink_checkpoint_policy = if value == "off" {
                    dodb_server::BlinkCheckpointPolicy::Disabled
                } else {
                    let threshold_bytes = value.parse::<u64>().map_err(|_| {
                        "--blink-checkpoint-wal-bytes must be off or a positive byte count"
                            .to_owned()
                    })?;
                    if threshold_bytes == 0 {
                        return Err(
                            "--blink-checkpoint-wal-bytes must be off or a positive byte count"
                                .to_owned(),
                        );
                    }
                    dodb_server::BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes }
                };
            }
            "--width" => {
                width = value
                    .parse()
                    .map_err(|error| format!("invalid width: {error}"))?
            }
            "--clients" => {
                clients = value
                    .parse()
                    .map_err(|error| format!("invalid clients: {error}"))?
            }
            "--seed" => {
                seed = value
                    .parse()
                    .map_err(|error| format!("invalid seed: {error}"))?
            }
            "--value-size" => {
                value_size = value
                    .parse()
                    .map_err(|error| format!("invalid value size: {error}"))?
            }
            "--warmup-ms" => {
                warmup_ms = value
                    .parse()
                    .map_err(|error| format!("invalid warmup: {error}"))?
            }
            "--measurement-ms" => {
                measurement_ms = value
                    .parse()
                    .map_err(|error| format!("invalid measurement: {error}"))?
            }
            "--call-cap" => {
                call_cap = value
                    .parse()
                    .map_err(|error| format!("invalid call cap: {error}"))?
            }
            "--validate-histogram" => {
                validate_histogram = match value.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err("--validate-histogram must be 0 or 1".to_owned()),
                }
            }
            "--revision-allocator" => {
                revision_allocator = match value.as_str() {
                    "serialized" => dodb_rocksdb::RevisionAllocatorMode::Serialized,
                    "reserved-ranges" => dodb_rocksdb::RevisionAllocatorMode::ReservedRanges,
                    _ => return Err(format!("unknown revision allocator: {value}")),
                }
            }
            "--condition-key-lock-mode" => {
                condition_key_lock_mode = match value.as_str() {
                    "exclusive" => dodb_rocksdb::ConditionKeyLockMode::Exclusive,
                    "shared" => dodb_rocksdb::ConditionKeyLockMode::Shared,
                    _ => return Err(format!("unknown condition key lock mode: {value}")),
                }
            }
            "--engine-health-interval-ms" => {
                engine_health_interval_ms = value
                    .parse()
                    .map_err(|error| format!("invalid engine health interval: {error}"))?;
                if engine_health_interval_ms > 0 && engine_health_interval_ms < 1_000 {
                    return Err("--engine-health-interval-ms must be 0 or at least 1000".to_owned());
                }
            }
            "--shared-blocking-slots" => {
                shared_blocking_slots = value
                    .parse()
                    .map_err(|error| format!("invalid shared blocking slots: {error}"))?
            }
            "--memory-diagnostic" => {
                memory_diagnostic = match value.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err("--memory-diagnostic must be 0 or 1".to_owned()),
                }
            }
            "--source-revision" => source_revision = Some(value),
            "--memory-worker-startup-timeout-ms" => {
                memory_worker_startup_timeout_ms = value
                    .parse()
                    .map_err(|_| "invalid --memory-worker-startup-timeout-ms".to_owned())?
            }
            "--data-dir" => data_dir = Some(PathBuf::from(value)),
            "--report" => report_path = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }
    let backend = backend.ok_or_else(|| "--backend is required".to_owned())?;
    let blink_checkpoint = blink_checkpoint::Options::parse(
        checkpoint_values
            .get("--blink-wal-layout")
            .map(String::as_str),
        checkpoint_values
            .get("--blink-checkpoint-execution")
            .map(String::as_str),
        checkpoint_values
            .get("--blink-wal-segment-bytes")
            .map(String::as_str),
        backend == BackendKind::Blink,
    )?;
    let options = Options {
        backend,
        blink_checkpoint_policy,
        blink_checkpoint,
        width,
        clients,
        seed,
        value_size,
        warmup_ms,
        measurement_ms,
        call_cap,
        validate_histogram,
        revision_allocator,
        condition_key_lock_mode,
        engine_health_interval_ms,
        shared_blocking_slots,
        data_dir: data_dir.ok_or_else(|| "--data-dir is required".to_owned())?,
        report_path: report_path.ok_or_else(|| "--report is required".to_owned())?,
        memory_diagnostic,
        source_revision,
        memory_worker_startup_timeout_ms,
    };
    if blink_checkpoint_option_seen && options.backend != BackendKind::Blink {
        return Err(
            "--blink-checkpoint-wal-bytes is supported only with --backend blink".to_owned(),
        );
    }
    if options.memory_diagnostic
        && options.blink_checkpoint.layout != dodb_server::BlinkWalLayout::Legacy
    {
        return Err(
            "segmented checkpoints are unavailable in the memory diagnostic worker".to_owned(),
        );
    }
    if options.memory_diagnostic
        && matches!(
            options.blink_checkpoint_policy,
            dodb_server::BlinkCheckpointPolicy::WalBytesThreshold { .. }
        )
    {
        return Err(
            "B-link WAL checkpoint policy is unavailable through the memory diagnostic worker"
                .to_owned(),
        );
    }
    if ![1, 16].contains(&options.width) {
        return Err("width must be 1 or 16".to_owned());
    }
    if ![16, 64].contains(&options.clients) {
        return Err("clients must be 16 or 64".to_owned());
    }
    if options.backend == BackendKind::MinimalNative
        && options.revision_allocator != dodb_rocksdb::RevisionAllocatorMode::ReservedRanges
    {
        return Err("minimal-native requires reserved-range revisions".to_owned());
    }
    if options.value_size != INITIAL_VALUE_SIZE
        || options.measurement_ms == 0
        || options.call_cap == 0
        || options.shared_blocking_slots == 0
    {
        return Err(
            "value size must be 512; duration, call cap, and shared slots must be nonzero"
                .to_owned(),
        );
    }
    if options.memory_diagnostic && options.source_revision.is_none() {
        return Err("--source-revision is required with --memory-diagnostic 1".to_owned());
    }
    if options.memory_worker_startup_timeout_ms == 0
        || options.memory_worker_startup_timeout_ms
            > dodb_rocksdb::memory_diagnostic::MAX_WORKER_START_TIMEOUT_MS
    {
        return Err("--memory-worker-startup-timeout-ms must be between 1 and 300000".to_owned());
    }
    Ok(options)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("unconditional_write_workload: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_options()?;
    std::fs::create_dir_all(&options.data_dir).map_err(|error| error.to_string())?;
    if options
        .data_dir
        .read_dir()
        .map_err(|error| error.to_string())?
        .next()
        .is_some()
    {
        return Err("data directory must be empty".to_owned());
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(16)
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    let memory_session = if options.memory_diagnostic {
        let _ = quinn::rustls::crypto::aws_lc_rs::default_provider().install_default();
        Some(
            dodb_rocksdb::memory_diagnostic::MemoryDiagnosticSession::new(
                options.source_revision.clone().unwrap_or_default(),
            )
            .map_err(|error| error.to_string())?,
        )
    } else {
        None
    };
    let tenant = Arc::new(ProbeBackend::open(
        &options,
        runtime.handle().clone(),
        memory_session.clone(),
    )?);
    prepopulate_pool(&tenant, &options)?;
    let seeded_disk_bytes = directory_bytes(&options.data_dir)?;
    let baseline_rows = tenant.read_all()?;
    let baseline_revisions = collect_baseline_revisions(&baseline_rows, &options)?;
    drop(baseline_rows);
    if let Some(memory_session) = &memory_session {
        memory_session.set_phase("warmup");
    }
    let warmup_phase = execute_phase(
        Arc::clone(&tenant),
        &options,
        options.warmup_ms,
        true,
        Vec::new(),
        &baseline_revisions,
    )?;
    tenant.wait_for_blink_coordinators_idle();
    let process_sampler = ProcessSampler::start(options.data_dir.clone())?;
    let storage_before_measurement = database_and_wal_bytes(&options.data_dir);
    let engine_health_interval = Duration::from_millis(options.engine_health_interval_ms);
    tenant.reset_blink_checkpoint_metrics();
    tenant.start_measurement()?;
    if let Some(memory_session) = &memory_session {
        memory_session.set_phase("measurement");
    }
    let engine_health_sampler = if engine_health_interval.is_zero() || options.memory_diagnostic {
        None
    } else {
        tenant
            .engine_health_observer()
            .map(|observer| {
                observer.start_sampling(
                    engine_health_interval,
                    Duration::from_millis(options.measurement_ms),
                )
            })
            .transpose()?
    };
    let measurement_window_start = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let measured_phase = execute_phase(
        Arc::clone(&tenant),
        &options,
        options.measurement_ms,
        false,
        warmup_phase.client_results,
        &baseline_revisions,
    )?;
    let measurement_window_end = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let checkpoint_drain_started = Instant::now();
    tenant.wait_for_blink_coordinators_idle();
    let checkpoint_drain_ms = checkpoint_drain_started.elapsed().as_secs_f64() * 1_000.0;
    tenant.stop_measurement()?;
    let storage_after_measurement = database_and_wal_bytes(&options.data_dir);
    let engine_health_metrics = match engine_health_sampler {
        Some(sampler) => sampler.finish()?,
        None if options.memory_diagnostic => serde_json::json!({
            "enabled": options.engine_health_interval_ms > 0,
            "scope": "backend worker process; detailed samples are nested in memory_only_diagnostic.engine_health_reports",
        }),
        None if engine_health_interval.is_zero() => dodb_rocksdb::engine_health_disabled_report(),
        None => dodb_rocksdb::engine_health_unsupported_report(engine_health_interval),
    };
    let process_metrics = process_sampler.finish()?;
    let final_rows = tenant.read_all()?;
    let final_validation = validate_rows(
        &final_rows,
        &measured_phase.client_results,
        &baseline_revisions,
        &options,
    )?;
    if !final_validation
        .get("passed")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(format!("final state oracle failed: {final_validation}"));
    }
    drop(final_rows);
    if let Some(memory_session) = &memory_session {
        memory_session.set_phase("reopen-validation");
    }
    tenant.shutdown()?;
    let blink_checkpoint_metrics = tenant.blink_checkpoint_metrics();
    drop(tenant);
    let reopened_tenant =
        ProbeBackend::open(&options, runtime.handle().clone(), memory_session.clone())?;
    let observed_rows = reopened_tenant.read_all()?;
    let reopen_validation = validate_rows(
        &observed_rows,
        &measured_phase.client_results,
        &baseline_revisions,
        &options,
    )?;
    drop(observed_rows);
    if !reopen_validation
        .get("passed")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(format!("reopen state oracle failed: {reopen_validation}"));
    }
    if let Some(memory_session) = &memory_session {
        memory_session.set_phase("residency-cache-warmup");
        let first_scan = reopened_tenant.residency_scan()?;
        let host_before = memory_session.boundary_snapshot();
        let process_before = memory_session.process_boundary_snapshot();
        memory_session.set_phase("residency-second-scan");
        let second_scan = reopened_tenant.residency_scan()?;
        let process_after = memory_session.process_boundary_snapshot();
        let host_after = memory_session.boundary_snapshot();
        let reopen_oracle_passed = reopen_validation
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        memory_session.record_residency_report(
            dodb_rocksdb::memory_diagnostic::residency_scan_report(
                &first_scan,
                &second_scan,
                &host_before,
                &host_after,
                &process_before,
                &process_after,
                reopen_oracle_passed,
            ),
        );
    }
    reopened_tenant.shutdown()?;
    drop(reopened_tenant);
    let disk_usage_bytes = directory_bytes(&options.data_dir)?;
    let mut report = build_report(
        &options,
        &measured_phase.client_results,
        measured_phase.actual_duration_ms,
        final_validation,
        reopen_validation,
        process_metrics,
        engine_health_metrics,
        disk_usage_bytes,
        seeded_disk_bytes,
        blink_checkpoint_metrics,
        checkpoint_drain_ms,
        storage_before_measurement,
        storage_after_measurement,
    );
    report["git_commit"] = json!(env!("DODB_BUILD_GIT_COMMIT"));
    report["blink_checkpoint_configuration"] = json!(
        (options.backend == BackendKind::Blink).then(|| options.blink_checkpoint.configuration())
    );
    report["measurement_window_unix_nanos"] = json!({"start": measurement_window_start.to_string(), "last_client_completion": measurement_window_end.to_string()});
    if let Some(memory_session) = memory_session {
        report["memory_only_diagnostic"] = memory_session.finish_sampling();
    }
    let report_text = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?;
    std::fs::write(&options.report_path, format!("{report_text}\n"))
        .map_err(|error| error.to_string())?;
    println!("{report_text}");
    if report["blink_checkpoint_metrics"]["checkpoints_failed"]
        .as_u64()
        .unwrap_or(0)
        > 0
    {
        return Err("B-link checkpoint failed during the measurement run".to_owned());
    }
    Ok(())
}

fn execute_phase(
    backend: Arc<ProbeBackend>,
    options: &Options,
    duration_ms: u64,
    warmup_phase: bool,
    prior_results: Vec<ClientResult>,
    baseline_revisions: &[Vec<Vec<u64>>],
) -> Result<PhaseResult, String> {
    let prior_sequences = prior_results
        .into_iter()
        .map(|result| {
            (
                result.slot_write_counts,
                result.latest_payload_sequences,
                result.latest_ack_revisions,
                result.warmup_writes,
            )
        })
        .collect::<Vec<_>>();
    let ready_barrier = Arc::new(Barrier::new(options.clients + 1));
    let start_barrier = Arc::new(Barrier::new(options.clients + 1));
    let shared_start = Arc::new(Mutex::new(None));
    let acknowledged_calls = Arc::new(AtomicU64::new(0));
    let call_limit_reached = Arc::new(AtomicBool::new(false));
    let mut workers = Vec::with_capacity(options.clients);
    for client_index in 0..options.clients {
        let worker_backend = Arc::clone(&backend);
        let worker_options = options.clone_for_worker();
        let worker_ready_barrier = Arc::clone(&ready_barrier);
        let worker_barrier = Arc::clone(&start_barrier);
        let worker_start = Arc::clone(&shared_start);
        let worker_calls = Arc::clone(&acknowledged_calls);
        let worker_limit = Arc::clone(&call_limit_reached);
        let (slot_write_counts, payload_sequences, ack_revisions, prior_writes) = prior_sequences
            .get(client_index)
            .cloned()
            .unwrap_or_else(|| {
                let baseline_for_client = &baseline_revisions[client_index];
                let revision_per_slot = baseline_for_client
                    .iter()
                    .map(|slot_revisions| slot_revisions.first().copied().unwrap_or(0))
                    .collect::<Vec<_>>();
                (
                    vec![0; POOL_SLOTS_PER_CLIENT],
                    vec![0; POOL_SLOTS_PER_CLIENT],
                    revision_per_slot,
                    0,
                )
            });
        workers.push(thread::spawn(move || {
            run_client_phase(
                worker_backend,
                worker_options,
                client_index,
                slot_write_counts,
                payload_sequences,
                ack_revisions,
                duration_ms,
                worker_ready_barrier,
                worker_barrier,
                worker_start,
                worker_calls,
                worker_limit,
                warmup_phase,
                prior_writes,
            )
        }));
    }
    ready_barrier.wait();
    let phase_started = Instant::now();
    *shared_start
        .lock()
        .map_err(|_| "phase start lock poisoned".to_owned())? = Some(phase_started);
    start_barrier.wait();
    let mut client_results = Vec::with_capacity(options.clients);
    for worker in workers {
        client_results.push(
            worker
                .join()
                .map_err(|_| "client worker panicked".to_owned())??,
        );
    }
    let actual_duration = phase_started.elapsed();
    if actual_duration < Duration::from_millis(duration_ms) {
        return Err("measurement window ended before requested duration".to_owned());
    }
    if call_limit_reached.load(Ordering::Relaxed) {
        return Err("request call cap reached before full measurement window".to_owned());
    }
    if acknowledged_calls.load(Ordering::Relaxed) == 0 {
        return Err("measurement completed without acknowledged writes".to_owned());
    }
    Ok(PhaseResult {
        client_results,
        actual_duration_ms: actual_duration.as_millis().min(u64::MAX as u128) as u64,
    })
}

fn run_client_phase(
    backend: Arc<ProbeBackend>,
    options: Options,
    client_index: usize,
    mut slot_write_counts: Vec<u64>,
    mut latest_payload_sequences: Vec<u64>,
    mut latest_ack_revisions: Vec<u64>,
    duration_ms: u64,
    ready_barrier: Arc<Barrier>,
    start_barrier: Arc<Barrier>,
    shared_start: Arc<Mutex<Option<Instant>>>,
    acknowledged_calls: Arc<AtomicU64>,
    call_limit_reached: Arc<AtomicBool>,
    warmup_phase: bool,
    prior_writes: u64,
) -> Result<ClientResult, String> {
    let mut histogram = SampleHistogram::new(options.validate_histogram);
    let mut measurement_writes = 0_u64;
    let mut width_keys_acknowledged = 0_u64;
    let mut phase_operation_index = 0_u64;
    let mut input_prefix_fingerprint = FNV1A64_OFFSET_BASIS;
    ready_barrier.wait();
    start_barrier.wait();
    let phase_started = shared_start
        .lock()
        .map_err(|_| "phase start lock poisoned".to_owned())?
        .ok_or_else(|| "phase start was not published".to_owned())?;
    let deadline = phase_started + Duration::from_millis(duration_ms);
    loop {
        if Instant::now() >= deadline {
            break;
        }
        let current_calls = acknowledged_calls.load(Ordering::Relaxed);
        if current_calls >= options.call_cap {
            call_limit_reached.store(true, Ordering::Relaxed);
            break;
        }
        let slot_index = (phase_operation_index % POOL_SLOTS_PER_CLIENT as u64) as usize;
        let payload_sequence = if warmup_phase {
            phase_operation_index / POOL_SLOTS_PER_CLIENT as u64 + 1
        } else {
            MEASUREMENT_PAYLOAD_SEQUENCE_BASE
                .saturating_add(phase_operation_index / POOL_SLOTS_PER_CLIENT as u64)
                .saturating_add(1)
        };
        let writes = mutation_for_slot(&options, client_index, slot_index, payload_sequence);
        if !warmup_phase && phase_operation_index < INPUT_PREFIX_BATCH_LIMIT {
            input_prefix_fingerprint =
                update_input_prefix_fingerprint(input_prefix_fingerprint, &writes);
        }
        let write_started = Instant::now();
        let acknowledged_revision = backend.write(&writes)?;
        if let Some(acknowledged_revision) = acknowledged_revision {
            if acknowledged_revision.get() <= latest_ack_revisions[slot_index] {
                return Err(format!(
                    "acknowledged revision did not advance for client {client_index}, slot {slot_index}"
                ));
            }
            latest_ack_revisions[slot_index] = acknowledged_revision.get();
        }
        let elapsed_nanos = write_started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        if !warmup_phase {
            histogram.record(elapsed_nanos);
        }
        slot_write_counts[slot_index] = slot_write_counts[slot_index].saturating_add(1);
        latest_payload_sequences[slot_index] = payload_sequence;
        phase_operation_index = phase_operation_index.saturating_add(1);
        measurement_writes = measurement_writes.saturating_add(1);
        width_keys_acknowledged = width_keys_acknowledged.saturating_add(options.width as u64);
        let completed_calls = acknowledged_calls
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        if completed_calls >= options.call_cap {
            call_limit_reached.store(true, Ordering::Relaxed);
        }
    }
    Ok(ClientResult {
        slot_write_counts,
        latest_payload_sequences,
        latest_ack_revisions,
        input_prefix_fingerprint,
        input_prefix_batches: if warmup_phase {
            0
        } else {
            phase_operation_index.min(INPUT_PREFIX_BATCH_LIMIT)
        },
        warmup_writes: if warmup_phase {
            measurement_writes
        } else {
            prior_writes
        },
        measurement_writes: if warmup_phase { 0 } else { measurement_writes },
        width_keys_acknowledged,
        histogram,
    })
}

fn prepopulate_pool(backend: &ProbeBackend, options: &Options) -> Result<(), String> {
    let mut writes = Vec::with_capacity(PREPOPULATE_BATCH_SIZE);
    for client_index in 0..options.clients {
        for slot_index in 0..POOL_SLOTS_PER_CLIENT {
            for width_index in 0..options.width {
                let key = document_key(options.seed, client_index, slot_index, width_index);
                writes.push((key, vec![0; options.value_size]));
                if writes.len() == PREPOPULATE_BATCH_SIZE {
                    backend.write(&writes)?;
                    writes.clear();
                }
            }
        }
    }
    if !writes.is_empty() {
        backend.write(&writes)?;
    }
    Ok(())
}

fn mutation_for_slot(
    options: &Options,
    client_index: usize,
    slot_index: usize,
    sequence: u64,
) -> Vec<(DocumentKey, Vec<u8>)> {
    (0..options.width)
        .map(|width_index| {
            (
                document_key(options.seed, client_index, slot_index, width_index),
                payload(
                    options.seed,
                    client_index,
                    slot_index,
                    width_index,
                    sequence,
                    options.value_size,
                ),
            )
        })
        .collect()
}

fn update_input_prefix_fingerprint(mut fingerprint: u64, writes: &[(DocumentKey, Vec<u8>)]) -> u64 {
    for (key, value) in writes {
        let encoded_key = physical_key(key);
        fingerprint = update_fnv1a64(fingerprint, &(encoded_key.len() as u64).to_be_bytes());
        fingerprint = update_fnv1a64(fingerprint, &encoded_key);
        fingerprint = update_fnv1a64(fingerprint, &(value.len() as u64).to_be_bytes());
        fingerprint = update_fnv1a64(fingerprint, value);
    }
    fingerprint
}

fn update_fnv1a64(mut fingerprint: u64, bytes: &[u8]) -> u64 {
    for byte_value in bytes {
        fingerprint ^= *byte_value as u64;
        fingerprint = fingerprint.wrapping_mul(FNV1A64_PRIME);
    }
    fingerprint
}

fn document_key(
    seed: u64,
    client_index: usize,
    slot_index: usize,
    width_index: usize,
) -> DocumentKey {
    let mut partition = seed.to_be_bytes().to_vec();
    partition.extend_from_slice(&(client_index as u64).to_be_bytes());
    let mut sort = (slot_index as u64).to_be_bytes().to_vec();
    sort.extend_from_slice(&(width_index as u64).to_be_bytes());
    DocumentKey::new(partition, sort)
}

fn physical_key(key: &DocumentKey) -> Vec<u8> {
    let encoded = key.encode();
    let mut physical = Vec::with_capacity(encoded.len() + 1);
    physical.push(1);
    physical.extend_from_slice(&encoded);
    physical
}

fn raw_envelope(payload: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(payload.len() + 10);
    encoded.extend_from_slice(&[1, 1]);
    encoded.extend_from_slice(&0_u64.to_be_bytes());
    encoded.extend_from_slice(payload);
    encoded
}

fn decode_raw_entry(
    key_bytes: &[u8],
    encoded_value: &[u8],
) -> Result<(DocumentKey, Vec<u8>, Option<Revision>), String> {
    if key_bytes.first() != Some(&1)
        || encoded_value.len() < 10
        || encoded_value[0] != 1
        || encoded_value[1] != 1
    {
        return Err("raw RocksDB row has an unexpected key or envelope".to_owned());
    }
    let key = DocumentKey::decode(&key_bytes[1..]).map_err(|error| error.to_string())?;
    Ok((key, encoded_value[10..].to_vec(), None))
}

fn payload(
    seed: u64,
    client_index: usize,
    slot_index: usize,
    width_index: usize,
    sequence: u64,
    value_size: usize,
) -> Vec<u8> {
    let mut value = vec![0; value_size];
    let mut state = seed
        ^ (client_index as u64).rotate_left(11)
        ^ (slot_index as u64).rotate_left(23)
        ^ (width_index as u64).rotate_left(37)
        ^ sequence.rotate_left(47);
    for (byte_index, value_byte) in value.iter_mut().enumerate() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *value_byte = (state ^ byte_index as u64) as u8;
    }
    value
}

fn validate_rows(
    observed_rows: &[(DocumentKey, Vec<u8>, Option<Revision>)],
    client_results: &[ClientResult],
    baseline_revisions: &[Vec<Vec<u64>>],
    options: &Options,
) -> Result<Value, String> {
    let mut expected = BTreeMap::new();
    for (client_index, client_result) in client_results.iter().enumerate() {
        for (slot_index, payload_sequence) in
            client_result.latest_payload_sequences.iter().enumerate()
        {
            for width_index in 0..options.width {
                let key = document_key(options.seed, client_index, slot_index, width_index);
                let value = if *payload_sequence == 0 {
                    vec![0; options.value_size]
                } else {
                    payload(
                        options.seed,
                        client_index,
                        slot_index,
                        width_index,
                        *payload_sequence,
                        options.value_size,
                    )
                };
                expected.insert(physical_key(&key), (key, value));
            }
        }
    }
    let mut observed = BTreeMap::new();
    let mut grouped_revisions = BTreeMap::<(usize, usize), BTreeSet<u64>>::new();
    for (key, value, revision) in observed_rows {
        let physical = physical_key(key);
        let (client_index, slot_index, _) = key_coordinates(key)?;
        if let Some(revision) = revision {
            grouped_revisions
                .entry((client_index, slot_index))
                .or_default()
                .insert(revision.get());
        }
        observed.insert(physical, (value.clone(), *revision));
    }
    let mut missing_count = 0_usize;
    let mut extra_count = 0_usize;
    let mut mismatch_count = 0_usize;
    let mut revision_group_mismatch_count = 0_usize;
    for (key_bytes, (expected_key, expected_value)) in &expected {
        let (client_index, slot_index, width_index) = key_coordinates(expected_key)?;
        match observed.get(key_bytes) {
            Some((observed_value, revision)) => {
                if observed_value != expected_value {
                    mismatch_count += 1;
                }
                if options.backend != BackendKind::RawRocksDb {
                    let expected_revision = client_results
                        .get(client_index)
                        .and_then(|client| client.latest_ack_revisions.get(slot_index))
                        .copied()
                        .or_else(|| {
                            baseline_revisions
                                .get(client_index)?
                                .get(slot_index)?
                                .get(width_index)
                                .copied()
                        });
                    if revision.map(Revision::get) != expected_revision {
                        mismatch_count += 1;
                    }
                }
            }
            None => missing_count += 1,
        }
    }
    for key_bytes in observed.keys() {
        if !expected.contains_key(key_bytes) {
            extra_count += 1;
        }
    }
    if options.backend != BackendKind::RawRocksDb {
        for (client_index, client_result) in client_results.iter().enumerate() {
            for slot_index in 0..POOL_SLOTS_PER_CLIENT {
                let expected_revision = client_result.latest_ack_revisions[slot_index];
                if grouped_revisions
                    .get(&(client_index, slot_index))
                    .is_none_or(|revisions| {
                        revisions.len() != 1 || !revisions.contains(&expected_revision)
                    })
                {
                    revision_group_mismatch_count += 1;
                }
            }
        }
    }
    let passed = missing_count == 0
        && extra_count == 0
        && mismatch_count == 0
        && revision_group_mismatch_count == 0;
    Ok(json!({
        "passed": passed,
        "expected_rows": expected.len(),
        "actual_rows": observed.len(),
        "missing_rows": missing_count,
        "extra_rows": extra_count,
        "value_or_revision_mismatches": mismatch_count,
        "same_transaction_revision_group_mismatches": revision_group_mismatch_count,
        "revision_progression_scope": "each acknowledged write advances the same slot revision; final and reopened per-key revisions exactly match the latest acknowledged revision, while untouched slots retain their baseline revision; keys in each acknowledged batch share one revision",
        "reopened_after_shutdown": true,
        "raw_placeholder_revision_is_not_logical_revision": options.backend == BackendKind::RawRocksDb,
    }))
}

fn collect_baseline_revisions(
    baseline_rows: &[(DocumentKey, Vec<u8>, Option<Revision>)],
    options: &Options,
) -> Result<Vec<Vec<Vec<u64>>>, String> {
    let expected_row_count = options
        .clients
        .saturating_mul(POOL_SLOTS_PER_CLIENT)
        .saturating_mul(options.width);
    if baseline_rows.len() != expected_row_count {
        return Err(format!(
            "prepopulated pool has {} rows, expected {expected_row_count}",
            baseline_rows.len()
        ));
    }
    let mut revisions = vec![vec![vec![0; options.width]; POOL_SLOTS_PER_CLIENT]; options.clients];
    let mut rows_seen = BTreeSet::new();
    let mut grouped_revisions = BTreeMap::<(usize, usize), BTreeSet<u64>>::new();
    for (key, _, revision) in baseline_rows {
        let (client_index, slot_index, width_index) = key_coordinates(key)?;
        if client_index >= options.clients
            || slot_index >= POOL_SLOTS_PER_CLIENT
            || width_index >= options.width
        {
            return Err("prepopulated key is outside the configured overwrite pool".to_owned());
        }
        if !rows_seen.insert((client_index, slot_index, width_index)) {
            return Err("prepopulated pool contains a duplicate logical key".to_owned());
        }
        let revision_value = revision.map_or(0, Revision::get);
        revisions[client_index][slot_index][width_index] = revision_value;
        grouped_revisions
            .entry((client_index, slot_index))
            .or_default()
            .insert(revision_value);
    }
    if rows_seen.len() != expected_row_count
        || grouped_revisions
            .values()
            .any(|revision_values| revision_values.len() != 1)
    {
        return Err(
            "prepopulated pool has missing keys or inconsistent transaction revisions".to_owned(),
        );
    }
    Ok(revisions)
}

fn key_coordinates(key: &DocumentKey) -> Result<(usize, usize, usize), String> {
    if key.pk.as_bytes().len() != 16 || key.sk.as_bytes().len() != 16 {
        return Err("observed key has unexpected canonical components".to_owned());
    }
    let client_index = u64::from_be_bytes(
        key.pk.as_bytes()[8..16]
            .try_into()
            .map_err(|_| "invalid client key")?,
    ) as usize;
    let slot_index = u64::from_be_bytes(
        key.sk.as_bytes()[0..8]
            .try_into()
            .map_err(|_| "invalid slot key")?,
    ) as usize;
    let width_index = u64::from_be_bytes(
        key.sk.as_bytes()[8..16]
            .try_into()
            .map_err(|_| "invalid width key")?,
    ) as usize;
    Ok((client_index, slot_index, width_index))
}

fn measurement_timing(
    measurement_writes: u64,
    requested_measurement_ms: u64,
    actual_measurement_duration_ms: u64,
    checkpoint_drain_ms: f64,
) -> Value {
    let total_elapsed_ms = actual_measurement_duration_ms as f64 + checkpoint_drain_ms;
    let denominator_seconds = total_elapsed_ms / 1_000.0;
    json!({
        "measurement_complete": true,
        "accepted_as_full_window": true,
        "termination_reason": "requested-duration",
        "requested_measurement_ms": requested_measurement_ms,
        "actual_admission_drain_and_last_completion_ms": actual_measurement_duration_ms,
        "final_checkpoint_drain_ms": checkpoint_drain_ms,
        "total_elapsed_ms": total_elapsed_ms,
        "throughput_denominator_seconds": denominator_seconds,
        "actual_write_batches": measurement_writes,
        "write_batches_per_second": measurement_writes as f64 / denominator_seconds,
    })
}

fn build_report(
    options: &Options,
    client_results: &[ClientResult],
    actual_measurement_duration_ms: u64,
    final_validation: Value,
    reopen_validation: Value,
    process_metrics: Value,
    engine_health_metrics: Value,
    disk_usage_bytes: u64,
    seeded_disk_bytes: u64,
    blink_checkpoint_metrics: Value,
    checkpoint_drain_ms: f64,
    storage_before_measurement: (u64, u64),
    storage_after_measurement: (u64, u64),
) -> Value {
    let mut histogram = SampleHistogram::new(options.validate_histogram);
    let measurement_writes = client_results
        .iter()
        .map(|result| result.measurement_writes)
        .sum::<u64>();
    let acknowledged_keys = client_results
        .iter()
        .map(|result| result.width_keys_acknowledged)
        .sum::<u64>();
    let warmup_writes = client_results
        .iter()
        .map(|result| result.warmup_writes)
        .sum::<u64>();
    let mut per_client = Vec::with_capacity(client_results.len());
    let mut histogram_capacity_bytes = 0_usize;
    for (client_index, result) in client_results.iter().enumerate() {
        histogram.merge(&result.histogram);
        histogram_capacity_bytes = histogram_capacity_bytes
            .saturating_add(result.histogram.bucket_counts.capacity() * std::mem::size_of::<u64>());
        per_client.push(json!({
            "client_index": client_index,
            "measurement_write_batches": result.measurement_writes,
            "acknowledged_keys": result.width_keys_acknowledged,
            "completed_write_batches_by_slot": result.slot_write_counts,
            "final_payload_sequence_by_slot": result.latest_payload_sequences,
            "final_ack_revision_by_slot": result.latest_ack_revisions,
            "measurement_input_prefix_fingerprint_fnv1a64": format!("{:016x}", result.input_prefix_fingerprint),
            "measurement_input_prefix_batches": result.input_prefix_batches,
        }));
    }
    let minimum_client_writes = client_results
        .iter()
        .map(|result| result.measurement_writes)
        .min()
        .unwrap_or(0);
    let maximum_client_writes = client_results
        .iter()
        .map(|result| result.measurement_writes)
        .max()
        .unwrap_or(0);
    let merged_histogram_capacity_bytes =
        histogram.bucket_counts.capacity() * std::mem::size_of::<u64>();
    json!({
        "benchmark_kind": "unconditional_write_probe",
        "binary_sha256": std::env::var("DODB_BINARY_SHA256").unwrap_or_else(|_| "not-recorded".to_owned()),
        "run_status": if blink_checkpoint_metrics["checkpoints_failed"].as_u64().unwrap_or(0) > 0 || blink_checkpoint_metrics["failed_message"].is_string() { "failed_checkpoint" } else { "completed" },
        "backend": format!("{:?}", options.backend),
        "source_sha": std::env::var("DODB_SOURCE_SHA").unwrap_or_else(|_| "not-recorded".to_owned()),
        "configuration": {
            "seed": options.seed,
            "clients": options.clients,
            "width": options.width,
            "value_size_bytes": options.value_size,
            "overwrite_slots_per_client": POOL_SLOTS_PER_CLIENT,
            "prepopulated_before_warmup": true,
            "raw_key_prefix_bytes": 1,
            "canonical_document_key_encoding": true,
            "raw_placeholder_envelope_bytes": 10,
            "compression": "none",
            "wal_sync": true,
            "raw_has_logical_revision_occ_aba": false,
            "revision_allocator": if matches!(
                options.backend,
                BackendKind::SynchronousCore | BackendKind::RocksDbService
                    | BackendKind::MinimalNative
            ) {
                Some(format!("{:?}", options.revision_allocator))
            } else {
                None
            },
            "condition_key_lock_mode_requested": match options.condition_key_lock_mode {
                dodb_rocksdb::ConditionKeyLockMode::Exclusive => "exclusive",
                dodb_rocksdb::ConditionKeyLockMode::Shared => "shared",
            },
            "condition_key_lock_mode": match options.backend {
                BackendKind::RawRocksDb => "not-applicable-no-conditional-transactions",
                BackendKind::Blink => "not-applicable",
                BackendKind::SynchronousCore | BackendKind::RocksDbService => {
                    match options.condition_key_lock_mode {
                        dodb_rocksdb::ConditionKeyLockMode::Exclusive => "exclusive",
                        dodb_rocksdb::ConditionKeyLockMode::Shared => "shared",
                    }
                }
                BackendKind::MinimalNative => "exclusive",
            },
            "engine_health_interval_ms": options.engine_health_interval_ms,
            "memory_worker_startup_timeout_ms": if options.memory_diagnostic { Some(options.memory_worker_startup_timeout_ms) } else { None },
            "shared_blocking_slots": if options.backend == BackendKind::RocksDbService {
                Some(options.shared_blocking_slots)
            } else {
                None
            },
            "requested_shared_blocking_slots": options.shared_blocking_slots,
            "warmup_requested_ms": options.warmup_ms,
            "measurement_requested_ms": options.measurement_ms,
            "blink_checkpoint_policy": match options.blink_checkpoint_policy {
                dodb_server::BlinkCheckpointPolicy::Disabled => serde_json::json!({"mode": "disabled"}),
                dodb_server::BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes } => serde_json::json!({"mode": "wal_bytes_threshold", "threshold_bytes": threshold_bytes}),
            },
            "call_cap": options.call_cap,
            "validate_histogram_against_exact_samples": options.validate_histogram,
            "caller_threads": options.clients,
            "executor": "one dedicated caller thread per client",
            "measurement_slot_schedule": "each client starts a local phase operation index at zero; slot is phase index modulo pool size",
            "measurement_payload_sequence_base": MEASUREMENT_PAYLOAD_SEQUENCE_BASE,
            "input_prefix_fingerprint": "FNV-1a 64-bit over the first 100 measurement batches per client, including physical canonical key bytes and payload bytes",
        },
        "timing": measurement_timing(
            measurement_writes, options.measurement_ms,
            actual_measurement_duration_ms, checkpoint_drain_ms,
        ),
        "counters": {
            "measurement_write_batches_acknowledged": measurement_writes,
            "acknowledged_keys": acknowledged_keys,
            "warmup_write_batches_acknowledged": warmup_writes,
            "other_errors": 0,
            "overloads": 0,
            "request_limit_reached": false,
        },
        "latency": histogram.json(),
        "blink_checkpoint_metrics": blink_checkpoint_metrics,
        "fairness": {
            "per_client_write_batch_counts": client_results.iter().map(|result| result.measurement_writes).collect::<Vec<_>>(),
            "minimum_client_write_batches": minimum_client_writes,
            "maximum_client_write_batches": maximum_client_writes,
            "minimum_to_maximum_ratio": if maximum_client_writes == 0 { 0.0 } else { minimum_client_writes as f64 / maximum_client_writes as f64 },
        },
        "memory_accounting": {
            "per_client_latency_histogram_capacity_bytes": histogram_capacity_bytes,
            "merged_latency_histogram_capacity_bytes": merged_histogram_capacity_bytes,
            "latency_histogram_capacity_bytes": histogram_capacity_bytes.saturating_add(merged_histogram_capacity_bytes),
            "exact_validation_samples_capacity_bytes": histogram.exact_validation_samples.as_ref().map_or(0, |samples| samples.capacity() * std::mem::size_of::<u64>()),
            "client_sequence_entries": options.clients * POOL_SLOTS_PER_CLIENT,
            "baseline_revision_entries": options.clients * POOL_SLOTS_PER_CLIENT * options.width,
            "baseline_revision_storage_bytes": options.clients * POOL_SLOTS_PER_CLIENT * options.width * std::mem::size_of::<u64>(),
            "validation_expected_state_btree_and_seen_keys_included": false,
            "baseline_read_all_rows_retained_during_measurement": false,
            "final_and_reopened_read_all_rows_dropped_after_each_oracle": true,
            "validation_expected_state_map_and_key_set_memory_estimate_bytes": null,
            "sampled_process_metrics_include_validation_oracle": false,
            "raw_write_batch_native_memory_included": false,
        },
        "final_validation": final_validation,
        "reopen_validation": reopen_validation,
        "disk_usage_bytes": disk_usage_bytes,
        "seeded_disk_bytes": seeded_disk_bytes,
        "disk_growth_bytes": disk_usage_bytes.saturating_sub(seeded_disk_bytes),
        "process_metrics": process_metrics,
        "storage_file_bytes_at_measurement_boundaries": {
            "start": {"database": storage_before_measurement.0, "wal": storage_before_measurement.1},
            "end": {"database": storage_after_measurement.0, "wal": storage_after_measurement.1},
        },
        "engine_health": engine_health_metrics,
        "per_client": per_client,
    })
}

fn directory_bytes(directory: &Path) -> Result<u64, String> {
    let mut total_bytes = 0_u64;
    for entry in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let entry_path = entry.path();
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_symlink() {
            return Err(format!(
                "database directory contains symlink: {}",
                entry_path.display()
            ));
        }
        if file_type.is_dir() {
            total_bytes = total_bytes.saturating_add(directory_bytes(&entry_path)?);
        } else if file_type.is_file() {
            total_bytes = total_bytes
                .saturating_add(entry.metadata().map_err(|error| error.to_string())?.len());
        }
    }
    Ok(total_bytes)
}

fn database_and_wal_bytes(directory: &Path) -> (u64, u64) {
    let mut database_bytes = 0_u64;
    let mut wal_bytes = 0_u64;
    let mut pending = vec![directory.to_path_buf()];
    while let Some(current_directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(current_directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if dodb_rocksdb::research::is_wal_file(&entry.path()) {
                wal_bytes = wal_bytes.saturating_add(metadata.len());
            } else {
                database_bytes = database_bytes.saturating_add(metadata.len());
            }
        }
    }
    (database_bytes, wal_bytes)
}

fn histogram_bucket_index(elapsed_nanos: u64) -> usize {
    if elapsed_nanos < EXACT_HISTOGRAM_BUCKETS as u64 {
        return elapsed_nanos as usize;
    }
    let exponent = (u64::BITS - 1 - elapsed_nanos.leading_zeros()) as usize;
    let exponent_base = 1_u64 << exponent;
    let bucket_width = (exponent_base / SUB_BUCKETS_PER_EXPONENT as u64).max(1);
    let sub_bucket = ((elapsed_nanos - exponent_base) / bucket_width)
        .min(SUB_BUCKETS_PER_EXPONENT as u64 - 1) as usize;
    EXACT_HISTOGRAM_BUCKETS + (exponent.saturating_sub(10)) * SUB_BUCKETS_PER_EXPONENT + sub_bucket
}

fn histogram_bucket_upper_bound(bucket_index: u64) -> u64 {
    if bucket_index < EXACT_HISTOGRAM_BUCKETS as u64 {
        return bucket_index;
    }
    let offset = bucket_index - EXACT_HISTOGRAM_BUCKETS as u64;
    let exponent = 10 + offset / SUB_BUCKETS_PER_EXPONENT as u64;
    let sub_bucket = offset % SUB_BUCKETS_PER_EXPONENT as u64;
    let exponent_base = 1_u64.checked_shl(exponent as u32).unwrap_or(u64::MAX);
    let bucket_width = (exponent_base / SUB_BUCKETS_PER_EXPONENT as u64).max(1);
    exponent_base
        .saturating_add((sub_bucket + 1).saturating_mul(bucket_width))
        .saturating_sub(1)
}

fn sample_histogram_validation_report(
    samples: &[u64],
    bucket_counts: &[u64],
    recorded_count: u64,
) -> Value {
    let mut sorted_samples = samples.to_vec();
    sorted_samples.sort_unstable();
    let histogram_count = bucket_counts.iter().copied().sum::<u64>();
    let percentile_checks = [50, 95, 99]
        .into_iter()
        .map(|percentile| {
            let target_rank = (sorted_samples.len() as u64)
                .saturating_mul(percentile)
                .div_ceil(100)
                .max(1);
            let exact_nanos = target_rank
                .checked_sub(1)
                .and_then(|rank| sorted_samples.get(rank as usize).copied());
            let mut cumulative_count = 0_u64;
            let histogram_bucket =
                bucket_counts
                    .iter()
                    .enumerate()
                    .find_map(|(bucket_index, bucket_count)| {
                        cumulative_count = cumulative_count.saturating_add(*bucket_count);
                        (cumulative_count >= target_rank).then_some(bucket_index)
                    });
            let histogram_nanos = histogram_bucket
                .map(|bucket_index| histogram_bucket_upper_bound(bucket_index as u64));
            let bucket_width_nanos = histogram_bucket.map(|bucket_index| {
                if bucket_index < EXACT_HISTOGRAM_BUCKETS {
                    1
                } else {
                    let exponent =
                        10 + (bucket_index - EXACT_HISTOGRAM_BUCKETS) / SUB_BUCKETS_PER_EXPONENT;
                    (1_u64 << exponent) / SUB_BUCKETS_PER_EXPONENT as u64
                }
            });
            let error_nanos = exact_nanos
                .zip(histogram_nanos)
                .map(|(exact, histogram)| histogram.saturating_sub(exact));
            let histogram_is_upper_bound = exact_nanos
                .zip(histogram_nanos)
                .is_some_and(|(exact, histogram)| histogram >= exact);
            let within_bucket_width = histogram_is_upper_bound
                && error_nanos
                    .zip(bucket_width_nanos)
                    .is_some_and(|(error, width)| error <= width);
            json!({
                "percentile": percentile,
                "exact_nanos": exact_nanos,
                "histogram_upper_bound_nanos": histogram_nanos,
                "error_nanos": error_nanos,
                "bucket_width_nanos": bucket_width_nanos,
                "within_bucket_width": within_bucket_width,
            })
        })
        .collect::<Vec<_>>();
    let counts_match = samples.len() as u64 == recorded_count && histogram_count == recorded_count;
    let percentiles_passed = percentile_checks
        .iter()
        .all(|check| check.get("within_bucket_width").and_then(Value::as_bool) == Some(true));
    json!({
        "passed": counts_match && percentiles_passed,
        "recorded_count": recorded_count,
        "exact_sample_count": samples.len(),
        "histogram_bucket_count_sum": histogram_count,
        "counts_match": counts_match,
        "percentiles": percentile_checks,
    })
}

impl Options {
    fn clone_for_worker(&self) -> Self {
        Self {
            backend: self.backend,
            blink_checkpoint_policy: self.blink_checkpoint_policy,
            blink_checkpoint: self.blink_checkpoint,
            width: self.width,
            clients: self.clients,
            seed: self.seed,
            value_size: self.value_size,
            warmup_ms: self.warmup_ms,
            measurement_ms: self.measurement_ms,
            call_cap: self.call_cap,
            validate_histogram: self.validate_histogram,
            revision_allocator: self.revision_allocator,
            condition_key_lock_mode: self.condition_key_lock_mode,
            engine_health_interval_ms: self.engine_health_interval_ms,
            shared_blocking_slots: self.shared_blocking_slots,
            data_dir: self.data_dir.clone(),
            report_path: self.report_path.clone(),
            memory_diagnostic: self.memory_diagnostic,
            source_revision: self.source_revision.clone(),
            memory_worker_startup_timeout_ms: self.memory_worker_startup_timeout_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BackendKind, ClientResult, DocumentKey, FNV1A64_OFFSET_BASIS,
        MEASUREMENT_PAYLOAD_SEQUENCE_BASE, Options, POOL_SLOTS_PER_CLIENT, SampleHistogram,
        collect_baseline_revisions, document_key, histogram_bucket_index,
        histogram_bucket_upper_bound, payload, physical_key, raw_envelope,
        sample_histogram_validation_report, update_input_prefix_fingerprint, validate_rows,
    };

    #[test]
    fn throughput_includes_final_checkpoint_drain() {
        for (checkpoint_drain_ms, total_elapsed_ms) in
            [(0.0, 30_000.0), (1_250.0, 31_250.0), (0.5, 30_000.5)]
        {
            let timing = super::measurement_timing(31_250, 30_000, 30_000, checkpoint_drain_ms);
            assert_eq!(timing["total_elapsed_ms"], total_elapsed_ms);
            assert_eq!(timing["final_checkpoint_drain_ms"], checkpoint_drain_ms);
            assert_eq!(
                timing["throughput_denominator_seconds"],
                total_elapsed_ms / 1_000.0
            );
            assert_eq!(
                timing["write_batches_per_second"],
                31_250.0 / (total_elapsed_ms / 1_000.0)
            );
        }
    }

    #[test]
    fn bounded_histogram_counts_all_observations_and_merges_buckets() {
        let mut first_histogram = SampleHistogram::new(true);
        first_histogram.record(0);
        first_histogram.record(1_023);
        let mut second_histogram = SampleHistogram::new(true);
        second_histogram.record(1_024);
        second_histogram.record(2_048);
        first_histogram.merge(&second_histogram);

        assert_eq!(first_histogram.observation_count, 4);
        assert_eq!(first_histogram.percentile_nanos(50), 1_023);
        assert_eq!(
            first_histogram.percentile_nanos(99),
            histogram_bucket_upper_bound(histogram_bucket_index(2_048) as u64)
        );
        let exact_samples = first_histogram
            .exact_validation_samples
            .as_ref()
            .expect("validation samples are enabled in this test");
        assert_eq!(
            sample_histogram_validation_report(
                exact_samples,
                &first_histogram.bucket_counts,
                first_histogram.observation_count,
            )["passed"],
            true
        );
        assert_eq!(
            first_histogram.json()["exact_sample_validation"]["passed"],
            true
        );
        let histogram_report = first_histogram.json();
        assert_eq!(histogram_report["histogram_schema_version"], 1);
        assert_eq!(
            histogram_report["nonzero_bins"]
                .as_array()
                .unwrap()
                .iter()
                .map(|bin| bin[1].as_u64().unwrap())
                .sum::<u64>(),
            4
        );
    }

    #[test]
    fn raw_key_and_placeholder_match_full_api_physical_envelope() {
        let key = document_key(20261013, 7, 19, 3);
        let mut expected_physical = vec![1];
        expected_physical.extend_from_slice(&key.encode());
        assert_eq!(physical_key(&key), expected_physical);
        let value = payload(20261013, 7, 19, 3, 42, 512);
        let encoded = raw_envelope(&value);
        assert_eq!(encoded.len(), 522);
        assert_eq!(&encoded[..10], &[1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(&encoded[10..], value);
        assert_eq!(DocumentKey::decode(&expected_physical[1..]).unwrap(), key);
    }

    #[test]
    fn measurement_input_prefix_fingerprint_tracks_canonical_keys_and_payloads() {
        let key = document_key(20261013, 3, 9, 0);
        let first_value = payload(
            20261013,
            3,
            9,
            0,
            MEASUREMENT_PAYLOAD_SEQUENCE_BASE + 1,
            512,
        );
        let first_writes = vec![(key.clone(), first_value.clone())];
        let repeated_writes = vec![(key.clone(), first_value)];
        let changed_payload = vec![(
            key.clone(),
            payload(
                20261013,
                3,
                9,
                0,
                MEASUREMENT_PAYLOAD_SEQUENCE_BASE + 2,
                512,
            ),
        )];
        let changed_key = vec![(
            document_key(20261013, 3, 10, 0),
            payload(
                20261013,
                3,
                9,
                0,
                MEASUREMENT_PAYLOAD_SEQUENCE_BASE + 1,
                512,
            ),
        )];

        let first_fingerprint =
            update_input_prefix_fingerprint(FNV1A64_OFFSET_BASIS, &first_writes);
        assert_eq!(
            first_fingerprint,
            update_input_prefix_fingerprint(FNV1A64_OFFSET_BASIS, &repeated_writes)
        );
        assert_ne!(
            first_fingerprint,
            update_input_prefix_fingerprint(FNV1A64_OFFSET_BASIS, &changed_payload)
        );
        assert_ne!(
            first_fingerprint,
            update_input_prefix_fingerprint(FNV1A64_OFFSET_BASIS, &changed_key)
        );
    }

    #[test]
    fn untouched_slots_must_keep_their_exact_baseline_revision() {
        let options = Options {
            backend: BackendKind::SynchronousCore,
            blink_checkpoint_policy: dodb_server::BlinkCheckpointPolicy::Disabled,
            blink_checkpoint: blink_checkpoint::Options::default(),
            width: 1,
            clients: 1,
            seed: 20261013,
            value_size: 512,
            warmup_ms: 1,
            measurement_ms: 1,
            call_cap: 100,
            validate_histogram: false,
            revision_allocator: dodb_rocksdb::RevisionAllocatorMode::Serialized,
            condition_key_lock_mode: dodb_rocksdb::ConditionKeyLockMode::Exclusive,
            engine_health_interval_ms: 0,
            shared_blocking_slots: 8,
            memory_diagnostic: false,
            source_revision: None,
            memory_worker_startup_timeout_ms: 20_000,
            data_dir: std::path::PathBuf::from("unused-data"),
            report_path: std::path::PathBuf::from("unused-report"),
        };
        let baseline_rows = (0..POOL_SLOTS_PER_CLIENT)
            .map(|slot_index| {
                (
                    document_key(options.seed, 0, slot_index, 0),
                    vec![0; options.value_size],
                    Some(dodb_core::Revision::from(dodb_core::Lsn::new(17))),
                )
            })
            .collect::<Vec<_>>();
        let baseline_revisions = collect_baseline_revisions(&baseline_rows, &options).unwrap();
        let client_results = [ClientResult {
            slot_write_counts: vec![0; POOL_SLOTS_PER_CLIENT],
            latest_payload_sequences: vec![0; POOL_SLOTS_PER_CLIENT],
            latest_ack_revisions: vec![17; POOL_SLOTS_PER_CLIENT],
            input_prefix_fingerprint: FNV1A64_OFFSET_BASIS,
            input_prefix_batches: 0,
            warmup_writes: 0,
            measurement_writes: 0,
            width_keys_acknowledged: 0,
            histogram: SampleHistogram::new(false),
        }];

        let valid_result = validate_rows(
            &baseline_rows,
            &client_results,
            &baseline_revisions,
            &options,
        )
        .unwrap();
        assert_eq!(valid_result["passed"], true);

        let mut changed_rows = baseline_rows;
        changed_rows[0].2 = Some(dodb_core::Revision::from(dodb_core::Lsn::new(16)));
        let changed_result = validate_rows(
            &changed_rows,
            &client_results,
            &baseline_revisions,
            &options,
        )
        .unwrap();
        assert_eq!(changed_result["passed"], false);
    }
}
