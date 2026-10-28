#[path = "support/blink_checkpoint.rs"]
mod blink_checkpoint;

use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use dodb_core::{DocumentKey, Error, Revision, TenantId, TransactionMutation};
use dodb_protocol::ProtocolLimits;
use dodb_rocksdb::raw_research::RawRocksDb;
use dodb_rocksdb::{
    ConditionKeyLockMode, RevisionAllocatorMode, RocksDbService, RocksDbServiceConfig,
    engine_health_disabled_report,
};
use dodb_server::{
    BlinkCheckpointPolicy, BlinkTenantService, DodbServer, DodbServerConfig,
    LocalTenantServiceConfig, ServerTlsConfig,
};
use dodb_service::{
    Document, DodbService, ExecutionBudget, Request, Response, ServiceFuture, ShutdownFuture,
    TransactionOutcome,
};
use dodb_storage::{CoordinatorConfig, DatabaseConfig};
use serde_json::json;
use tokio::sync::mpsc;

type WorkerError = Box<dyn std::error::Error + Send + Sync>;
type WorkerResult<T> = Result<T, WorkerError>;

struct Arguments {
    backend: String,
    shared_resources: Option<dodb_rocksdb::shared_resources::SharedResourceConfig>,
    memory_cgroup_path: Option<PathBuf>,
    pressure_interval_ms: u64,
    data_dir: PathBuf,
    certificate_path: PathBuf,
    private_key_path: PathBuf,
    health_path: PathBuf,
    source_revision: String,
    worker_binary_sha256: String,
    parent_binary_sha256: String,
    parent_pid: u32,
    response_budget: usize,
    max_mutations: usize,
    max_concurrent_requests: usize,
    revision_allocator: RevisionAllocatorMode,
    condition_key_lock_mode: ConditionKeyLockMode,
    blocking_slots: usize,
    blocking_admission: dodb_rocksdb::BlockingAdmissionMode,
    engine_health_interval_ms: u64,
    tenant_count: usize,
    core_tenant_subdirectories: bool,
    blink_max_open_shards: usize,
    blink_cache_pages: Option<usize>,
    blink_checkpoint_policy: BlinkCheckpointPolicy,
    blink_checkpoint: blink_checkpoint::Options,
}

enum BackendService {
    Blink(BlinkTenantService),
    RocksDb(RocksDbService),
    MinimalNative(Arc<dodb_rocksdb::minimal_native::MinimalNativeService>),
    Core(BTreeMap<TenantId, Arc<dodb_rocksdb::research::SynchronousRocksDbTenant>>),
    Raw(Arc<RawRocksDb>),
}

impl BackendService {
    fn open(arguments: &Arguments) -> WorkerResult<Self> {
        match arguments.backend.as_str() {
            "blink" => {
                let config = LocalTenantServiceConfig {
                    data_dir: arguments.data_dir.clone(),
                    max_open_shards: arguments.blink_max_open_shards,
                    database_config: arguments
                        .blink_cache_pages
                        .map_or_else(DatabaseConfig::default, |capacity| {
                            DatabaseConfig::default().with_cache_capacity(capacity)
                        }),
                    coordinator_config: CoordinatorConfig::default(),
                    blink_checkpoint_policy: arguments.blink_checkpoint_policy,
                    blink_wal_layout: arguments.blink_checkpoint.layout,
                    blink_checkpoint_execution: arguments.blink_checkpoint.execution,
                    ..LocalTenantServiceConfig::default()
                };
                Ok(Self::Blink(BlinkTenantService::new(config)?))
            }
            "minimal-native" | "minimal-native-shared" => {
                let mut config = dodb_rocksdb::minimal_native::MinimalNativeConfig::new(
                    arguments.data_dir.clone(),
                );
                config.revision_allocator = arguments.revision_allocator;
                config.shared_resources = arguments.shared_resources;
                Ok(Self::MinimalNative(Arc::new(
                    dodb_rocksdb::minimal_native::MinimalNativeService::new(config)?,
                )))
            }
            "rocksdb-service" => {
                let mut config = RocksDbServiceConfig::new(&arguments.data_dir);
                config.revision_allocator = arguments.revision_allocator;
                config.condition_key_lock_mode = arguments.condition_key_lock_mode;
                config.max_blocking_operations = arguments.blocking_slots;
                config.blocking_admission = arguments.blocking_admission;
                config.detailed_write_metrics = false;
                Ok(Self::RocksDb(RocksDbService::new(config)?))
            }
            "core" => {
                let mut tenants = BTreeMap::new();
                for tenant_number in 1..=arguments.tenant_count {
                    let tenant = TenantId::new(tenant_number as u64);
                    let path = if arguments.core_tenant_subdirectories {
                        arguments.data_dir.join(format!("tenant-{tenant_number}"))
                    } else {
                        arguments.data_dir.clone()
                    };
                    let database = dodb_rocksdb::research::SynchronousRocksDbTenant::open_with_condition_key_lock_mode(
                        &path,
                        arguments.revision_allocator,
                        arguments.condition_key_lock_mode,
                    )?;
                    tenants.insert(tenant, Arc::new(database));
                }
                Ok(Self::Core(tenants))
            }
            "raw" => Ok(Self::Raw(Arc::new(RawRocksDb::open(&arguments.data_dir)?))),
            backend => Err(format!("unsupported diagnostic worker backend {backend}").into()),
        }
    }

    fn shared_resource_snapshot(&self) -> serde_json::Value {
        match self {
            Self::MinimalNative(service) => service
                .shared_resource_snapshot()
                .unwrap_or(serde_json::Value::Null),
            _ => serde_json::Value::Null,
        }
    }

    fn engine_health_observer(&self) -> Option<dodb_rocksdb::EngineHealthObserver> {
        match self {
            Self::Blink(_) => None,
            Self::MinimalNative(service) => Some(service.engine_health_observer()),
            Self::RocksDb(service) => Some(service.engine_health_observer()),
            Self::Core(tenants) => Some(dodb_rocksdb::EngineHealthObserver::combine(
                tenants.iter().map(|(tenant, database)| {
                    database.engine_health_observer_named(format!("tenant_{tenant}"))
                }),
            )),
            Self::Raw(database) => Some(database.engine_health_observer()),
        }
    }

    fn execute_raw(database: &RawRocksDb, request: Request) -> Result<Response, Error> {
        match request {
            Request::Transact { request } if request.conditions.is_empty() => {
                let entries = request
                    .mutations
                    .iter()
                    .map(|mutation| match mutation {
                        TransactionMutation::Put { key, value } => {
                            Ok((physical_key(key), raw_envelope(value)))
                        }
                        TransactionMutation::Delete { .. } => Err(Error::invalid_request(
                            "raw unconditional diagnostic supports Put mutations only",
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                database.write_batch(&entries)?;
                Ok(Response::Transact(
                    TransactionOutcome::conditions_satisfied(),
                ))
            }
            Request::Scan {
                exclusive_after_key,
                limit,
            } => {
                let cursor = exclusive_after_key.as_ref().map(physical_key);
                let rows = database.scan_page(cursor.as_deref(), limit)?;
                let documents = rows
                    .into_iter()
                    .map(|(key_bytes, value_bytes)| decode_raw_row(&key_bytes, &value_bytes))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Response::Scan(documents))
            }
            _ => Err(Error::invalid_request(
                "raw diagnostic implements only unconditional Put batches and bounded Scan",
            )),
        }
    }
}

fn blink_checkpoint_metrics_json(
    metrics: &dodb_server::BlinkWalCheckpointMetrics,
) -> serde_json::Value {
    let checkpoint_events = metrics
        .checkpoint_events
        .iter()
        .map(|event| {
            json!({
                "phase": event.phase,
                "task_id": event.task_id,
                "target_lsn": event.target_lsn,
                "writes_during_checkpoint": event.writes_during_checkpoint,
                "preparation_nanos": event.preparation_nanos,
                "page_write_nanos": event.page_write_nanos,
                "data_sync_nanos": event.data_sync_nanos,
                "metadata_sync_nanos": event.metadata_sync_nanos,
                "completion_nanos": event.completion_nanos,
                "started_unix_nanos": event.started_unix_nanos,
                "ended_unix_nanos": event.ended_unix_nanos,
                "wal_bytes_before": event.wal_bytes_before,
                "wal_bytes_after": event.wal_bytes_after,
                "reclaimed_bytes": event.reclaimed_bytes,
                "duration_nanos": event.duration_nanos,
                "succeeded": event.succeeded,
                "checkpoint_bytes_written": event.checkpoint_bytes_written,
                "base_bytes_written": event.base_bytes_written,
                "base_bytes_retained": event.base_bytes_retained,
                "base_bytes_reclaimed": event.base_bytes_reclaimed,
                "full_base_snapshots": event.full_base_snapshots,
                "checkpoint_write_requests": event.checkpoint_write_requests,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "status": "sampled",
        "policy": match metrics.policy {
            BlinkCheckpointPolicy::Disabled => json!({"mode": "disabled"}),
            BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes } => json!({
                "mode": "wal_bytes_threshold",
                "threshold_bytes": threshold_bytes,
            }),
        },
        "execution": match metrics.execution {
            dodb_server::BlinkCheckpointExecution::Sync => "sync",
            dodb_server::BlinkCheckpointExecution::Background => "background",
        },
        "threshold_bytes": metrics.threshold_bytes,
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
        "checkpoint_events": checkpoint_events,
        "failed_message": metrics.failed_message,
        "writes_during_checkpoint": metrics.writes_during_checkpoint,
        "merged_checkpoint_requests": metrics.merged_checkpoint_requests,
        "capacity_wait_count": metrics.capacity_wait_count,
        "capacity_wait_nanos": metrics.capacity_wait_nanos,
        "coordinator_pause_nanos": metrics.coordinator_pause_nanos,
        "retained_image_bytes_normalized_page_images": metrics.retained_image_bytes,
        "peak_retained_image_bytes_normalized_page_images": metrics.peak_retained_image_bytes,
        "tasks_running": metrics.tasks_running,
        "peak_tasks_running": metrics.peak_tasks_running,
    })
}

fn backend_configuration_json(arguments: &Arguments) -> serde_json::Value {
    match arguments.backend.as_str() {
        "blink" => json!({
            "max_open_shards": arguments.blink_max_open_shards,
            "max_concurrent_requests": arguments.max_concurrent_requests,
            "cache_capacity_pages": arguments.blink_cache_pages.unwrap_or_else(|| DatabaseConfig::default().cache_capacity),
            "coordinator_config": "default",
            "wal_layout": arguments.blink_checkpoint.configuration(),
            "checkpoint_policy": match arguments.blink_checkpoint_policy {
                BlinkCheckpointPolicy::Disabled => json!({"mode": "disabled"}),
                BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes } => json!({
                    "mode": "wal_bytes_threshold",
                    "threshold_bytes": threshold_bytes,
                }),
            },
            "cache_capacity_is_blink_memory_limit": false,
        }),
        "minimal-native" | "minimal-native-shared" => json!({
            "shared_resources": arguments.shared_resources.map(|resources| resources.report()),
            "revision_allocator": "reserved-ranges",
            "condition_key_lock_mode": "exclusive",
            "compression": "disabled",
            "request_adapter": "tokio spawn_blocking around synchronous MinimalNativeService execution",
            "max_concurrent_requests": arguments.max_concurrent_requests,
            "blocking_semaphore_added_by_worker": false,
        }),
        _ => serde_json::Value::Null,
    }
}

impl DodbService for BackendService {
    fn execute<'service>(
        &'service self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> ServiceFuture<'service> {
        Box::pin(async move {
            match self {
                Self::Blink(service) => service.execute(tenant, request, budget).await,
                Self::RocksDb(service) => service.execute(tenant, request, budget).await,
                Self::MinimalNative(service) => {
                    let service = Arc::clone(service);
                    tokio::task::spawn_blocking(move || {
                        futures::executor::block_on(service.execute(tenant, request, budget))
                    })
                    .await
                    .map_err(|error| Error::invariant(error.to_string()))?
                }
                Self::Core(tenants) => {
                    let service = Arc::clone(tenants.get(&tenant).ok_or_else(|| {
                        Error::invalid_request(format!("tenant {tenant} was not preopened"))
                    })?);
                    tokio::task::spawn_blocking(move || service.execute(request, budget))
                        .await
                        .map_err(|error| Error::invariant(error.to_string()))?
                }
                Self::Raw(database) => {
                    let database = Arc::clone(database);
                    tokio::task::spawn_blocking(move || Self::execute_raw(&database, request))
                        .await
                        .map_err(|error| Error::invariant(error.to_string()))?
                }
            }
        })
    }

    fn shutdown<'service>(&'service self) -> ShutdownFuture<'service> {
        Box::pin(async move {
            match self {
                Self::Blink(service) => service.shutdown().await,
                Self::RocksDb(service) => service.shutdown().await,
                Self::MinimalNative(service) => service.shutdown().await,
                Self::Core(_) | Self::Raw(_) => {}
            }
        })
    }
}

impl BackendService {
    fn blink_checkpoint_metrics(&self) -> serde_json::Value {
        match self {
            Self::Blink(service) => {
                blink_checkpoint_metrics_json(&service.checkpoint_metrics_snapshot())
            }
            Self::MinimalNative(_) | Self::RocksDb(_) | Self::Core(_) | Self::Raw(_) => {
                serde_json::json!({
                    "value": null,
                    "status": "unsupported",
                    "reason": "backend does not expose B-link checkpoint metrics",
                })
            }
        }
    }

    fn periodic_blink_checkpoint_metrics(&self) -> serde_json::Value {
        let mut metrics = self.blink_checkpoint_metrics();
        if let Some(fields) = metrics.as_object_mut()
            && fields.remove("checkpoint_events").is_some()
        {
            fields.remove("transaction_group_histogram");
            fields.insert(
                "detail_scope".to_owned(),
                json!("cumulative counters only; event details and group histogram are retained in the final worker report"),
            );
        }
        metrics
    }
}

fn main() -> WorkerResult<()> {
    let arguments = parse_arguments()?;
    install_parent_death_signal(arguments.parent_pid)?;
    let memory_membership = arguments
        .memory_cgroup_path
        .as_ref()
        .map(|path| dodb_rocksdb::memory_pressure::attach_worker(path))
        .transpose()
        .map_err(std::io::Error::other)?;
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run(arguments, memory_membership))
}

async fn run(
    arguments: Arguments,
    memory_membership: Option<serde_json::Value>,
) -> WorkerResult<()> {
    let _ = quinn::rustls::crypto::aws_lc_rs::default_provider().install_default();
    let backend = Arc::new(BackendService::open(&arguments)?);
    let protocol_limits = ProtocolLimits {
        max_response_frame_size: arguments.response_budget,
        max_mutations: arguments.max_mutations,
        ..ProtocolLimits::default()
    };
    let certificate = std::fs::read(&arguments.certificate_path)?;
    let private_key = std::fs::read(&arguments.private_key_path)?;
    let tls = ServerTlsConfig::from_der(vec![certificate], private_key)?;
    let server = Arc::new(
        DodbServer::bind(
            Arc::clone(&backend),
            DodbServerConfig {
                listen_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
                tls,
                protocol_limits,
                max_connections: 1,
                max_concurrent_streams: arguments.max_concurrent_requests,
                max_concurrent_requests: arguments.max_concurrent_requests,
            },
        )
        .map_err(|error| Box::new(error) as WorkerError)?,
    );
    let listen_address = server.local_addr()?;
    let mut server_task = tokio::spawn({
        let server = Arc::clone(&server);
        async move { server.run().await }
    });
    tokio::task::yield_now().await;
    let readiness = dodb_rocksdb::memory_diagnostic::worker_readiness(
        listen_address,
        arguments.source_revision.clone(),
        arguments.worker_binary_sha256.clone(),
        arguments.max_concurrent_requests,
    )?;
    println!(
        "{}",
        serde_json::to_string(&json!({
            "ready": readiness.ready,
            "listen_address": readiness.listen_address,
            "pid": readiness.pid,
            "process_group_id": readiness.process_group_id,
            "max_concurrent_requests": readiness.max_concurrent_requests,
            "started_unix_ms": readiness.started_unix_ms,
            "process_start_time_ticks": readiness.process_start_time_ticks,
            "source_revision": readiness.source_revision,
            "compiled_source_revision": env!("DODB_BUILD_GIT_COMMIT"),
            "worker_binary_sha256": readiness.worker_binary_sha256,
            "parent_binary_sha256": arguments.parent_binary_sha256,
            "backend": arguments.backend,
        }))?
    );
    std::io::stdout().flush()?;
    let (command_sender, mut command_receiver) = mpsc::unbounded_channel();
    std::thread::spawn(move || {
        for command_result in std::io::stdin().lock().lines() {
            match command_result {
                Ok(command) => {
                    if command_sender.send(command).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    let mut health_sampler = None;
    let mut health_report = engine_health_disabled_report();
    let mut shared_measurement_start = serde_json::Value::Null;
    let mut shared_measurement_end = serde_json::Value::Null;
    let mut pressure_samples = Vec::new();
    let mut pressure_samples_truncated = false;
    let mut pressure_sample_bytes = 0_usize;
    let mut pressure_phase = "before-measurement";
    let mut pressure_interval =
        tokio::time::interval(Duration::from_millis(arguments.pressure_interval_ms.max(1)));
    pressure_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        let command = tokio::select! {
            command = command_receiver.recv() => match command {
                Some(command) => command,
                None => break,
            },
            _ = pressure_interval.tick(), if arguments.pressure_interval_ms > 0 => {
                if !pressure_samples_truncated && pressure_samples.len() < 4096 && pressure_sample_bytes < 8 * 1024 * 1024 {
                    let sample = json!({
                        "timestamp_unix_ms": dodb_rocksdb::memory_pressure::unix_time_ms(),
                        "phase": pressure_phase,
                        "shared_resources": backend.shared_resource_snapshot(),
                        "rocksdb_properties": backend.engine_health_observer().map(|observer| observer.sample()),
                        "blink_checkpoints": backend.periodic_blink_checkpoint_metrics(),
                    });
                    let serialized_bytes = serde_json::to_vec(&sample)?.len();
                    if pressure_sample_bytes + serialized_bytes <= 8 * 1024 * 1024 {
                        pressure_sample_bytes += serialized_bytes;
                        pressure_samples.push(sample);
                    } else { pressure_samples_truncated = true; }
                } else { pressure_samples_truncated = true; }
                continue;
            }
        };
        match command.as_str() {
            "start-measurement" => {
                pressure_phase = "measurement-including-drain";
                shared_measurement_start = backend.shared_resource_snapshot();
                health_sampler = if arguments.engine_health_interval_ms == 0 {
                    None
                } else {
                    backend
                        .engine_health_observer()
                        .map(|observer| {
                            observer.start_sampling(
                                Duration::from_millis(arguments.engine_health_interval_ms),
                                Duration::from_secs(86_400),
                            )
                        })
                        .transpose()?
                };
            }
            "stop-measurement" => {
                pressure_phase = "after-measurement";
                shared_measurement_end = backend.shared_resource_snapshot();
                health_report = match health_sampler.take() {
                    Some(sampler) => sampler.finish()?,
                    None if arguments.engine_health_interval_ms == 0 => {
                        engine_health_disabled_report()
                    }
                    None => json!({"status": "unsupported"}),
                };
            }
            "shutdown" => break,
            other => return Err(format!("unknown diagnostic worker command {other}").into()),
        }
    }
    if let Some(sampler) = health_sampler.take() {
        health_report = sampler.finish()?;
    }
    let shutdown_property_snapshot = backend.engine_health_observer().map_or_else(
        || json!({"status": "unsupported", "reason": "backend exposes no RocksDB properties"}),
        |observer| observer.sample(),
    );
    let shared_before_shutdown = backend.shared_resource_snapshot();
    let shutdown = tokio::time::timeout(Duration::from_secs(15), async {
        server.shutdown().await;
        (&mut server_task).await
    })
    .await;
    match shutdown {
        Ok(Ok(Ok(()))) => {}
        Ok(Ok(Err(error))) => return Err(Box::new(error) as WorkerError),
        Ok(Err(error)) => return Err(Box::new(error) as WorkerError),
        Err(_) => {
            server_task.abort();
            return Err("diagnostic worker shutdown exceeded 15 seconds".into());
        }
    }
    let blink_checkpoint_metrics = backend.blink_checkpoint_metrics();
    drop(server);
    drop(backend);
    let worker_process_high_water_kb =
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|contents| {
                contents.lines().find_map(|line| {
                    let (field_name, field_value) = line.split_once(':')?;
                    (field_name == "VmHWM")
                        .then(|| field_value.split_whitespace().next()?.parse::<u64>().ok())
                        .flatten()
                })
            });
    std::fs::write(
        &arguments.health_path,
        serde_json::to_vec_pretty(&json!({
            "backend": arguments.backend.clone(),
            "blink_configuration": if arguments.backend == "blink" {
                backend_configuration_json(&arguments)
            } else {
                serde_json::Value::Null
            },
            "shared_resource_snapshots": {
                "measurement_start": shared_measurement_start,
                "measurement_end": shared_measurement_end,
                "before_shutdown": shared_before_shutdown,
                "scope": "control-command boundaries; end includes drain; not continuous or peak instrumentation",
            },
            "effective_backend_configuration": backend_configuration_json(&arguments),
            "backend_memory_cgroup": memory_membership,
            "periodic_pressure": {
                "enabled": arguments.pressure_interval_ms > 0,
                "interval_ms": arguments.pressure_interval_ms,
                "sample_limit": 4096,
                "serialized_sample_byte_limit": 8 * 1024 * 1024,
                "serialized_sample_bytes": pressure_sample_bytes,
                "storage_scope": "serialized sample bytes bound retained observation volume, not exact serde_json heap allocation; observations are charged to the worker budget",
                "samples_truncated": pressure_samples_truncated,
                "samples": pressure_samples,
                "scope": "worker lifetime after readiness; non-atomic sampled observations; not instantaneous peaks; measurement phase includes drain; shared cache/WBM counted once in shared_resources; repeated per-DB cache properties must not be summed",
                "unsupported": ["allocator retained heap", "generation pin ages", "request and checkpoint queue representation bytes"],
            },
            "blink_checkpoint_metrics": blink_checkpoint_metrics,
            "blink_checkpoint_metrics_scope": "worker_lifetime_including setup/preflight, offered load, measurement drain, and shutdown; measurement start/stop controls the RocksDB sampler only and does not reset or snapshot B-link checkpoint counters at offer boundaries",
            "retained_heap_bytes": {
                "value": null,
                "status": "unsupported",
                "reason": "worker does not expose allocator retained heap or representation-specific retained bytes",
            },
            "response_budget_bytes": arguments.response_budget,
            "max_mutations": arguments.max_mutations,
            "max_concurrent_requests": arguments.max_concurrent_requests,
            "engine_health_interval_ms": arguments.engine_health_interval_ms,
            "engine_health": health_report,
            "shutdown_property_snapshot": shutdown_property_snapshot,
            "worker_process_write_io_at_shutdown": blink_checkpoint::process_write_io(),
            "worker_pid": readiness.pid,
            "worker_process_start_time_ticks": readiness.process_start_time_ticks,
            "worker_epoch_id": format!("{}:{}", readiness.pid, readiness.process_start_time_ticks),
            "compiled_source_revision": env!("DODB_BUILD_GIT_COMMIT"),
            "worker_process_high_water_rss_kb_after_backend_shutdown": worker_process_high_water_kb,
            "shutdown_complete": true,
        }))?,
    )?;
    Ok(())
}

fn parse_arguments() -> WorkerResult<Arguments> {
    let values = std::env::args().skip(1).collect::<Vec<_>>();
    let backend = argument_value(&values, "--backend")?;
    if ![
        "raw",
        "core",
        "rocksdb-service",
        "blink",
        "minimal-native",
        "minimal-native-shared",
    ]
    .contains(&backend.as_str())
    {
        return Err(format!("unsupported diagnostic worker backend {backend}").into());
    }
    let shared_resources =
        dodb_rocksdb::shared_resources::SharedResourceConfig::from_arguments(&backend, &values)?;
    let memory_cgroup_path = option_value(&values, "--memory-cgroup-path").map(PathBuf::from);
    let pressure_interval_ms = option_value(&values, "--memory-pressure-interval-ms")
        .map(|value| value.parse::<u64>())
        .transpose()?
        .unwrap_or(0);
    if (memory_cgroup_path.is_some() && !(10..=10_000).contains(&pressure_interval_ms))
        || (memory_cgroup_path.is_none() && pressure_interval_ms != 0)
    {
        return Err(
            "memory pressure sampling requires a cgroup and a 10..10000 ms interval".into(),
        );
    }
    let data_dir = PathBuf::from(argument_value(&values, "--memory-data-dir")?);
    let certificate_path = PathBuf::from(argument_value(&values, "--memory-certificate")?);
    let private_key_path = PathBuf::from(argument_value(&values, "--memory-private-key")?);
    let health_path = PathBuf::from(argument_value(&values, "--memory-health-report")?);
    let source_revision = argument_value(&values, "--memory-source-revision")?;
    if source_revision != env!("DODB_BUILD_GIT_COMMIT") {
        return Err(format!(
            "worker source revision {source_revision} does not match compiled revision {}",
            env!("DODB_BUILD_GIT_COMMIT")
        )
        .into());
    }
    let worker_binary_sha256 = argument_value(&values, "--memory-worker-binary-sha256")?;
    let parent_binary_sha256 = argument_value(&values, "--memory-parent-binary-sha256")?;
    let parent_pid = argument_value(&values, "--memory-parent-pid")?.parse()?;
    let response_budget = argument_value(&values, "--memory-response-budget")?.parse()?;
    let max_mutations = argument_value(&values, "--memory-max-mutations")?.parse()?;
    let max_concurrent_requests =
        argument_value(&values, "--memory-max-concurrent-requests")?.parse()?;
    if max_concurrent_requests == 0 {
        return Err("maximum concurrent requests must be nonzero".into());
    }
    let blocking_slots = option_value(&values, "--shared-blocking-slots")
        .unwrap_or_else(|| "8".to_owned())
        .parse()?;
    let blocking_admission = match option_value(&values, "--blocking-admission")
        .unwrap_or_else(|| "shared".to_owned())
        .as_str()
    {
        "shared" => dodb_rocksdb::BlockingAdmissionMode::Shared,
        "split-read-write" => dodb_rocksdb::BlockingAdmissionMode::SplitReadWrite,
        "split-read-2-write-8" => dodb_rocksdb::BlockingAdmissionMode::SplitRead2Write8,
        other => return Err(format!("unknown blocking admission {other}").into()),
    };
    let blocking_slots = blocking_admission
        .split_slot_limits()
        .map_or(blocking_slots, |(read_slots, write_slots)| {
            read_slots + write_slots
        });
    let engine_health_interval_ms = option_value(&values, "--engine-health-interval-ms")
        .unwrap_or_else(|| "0".to_owned())
        .parse()?;
    let tenant_count = option_value(&values, "--memory-tenant-count")
        .unwrap_or_else(|| "1".to_owned())
        .parse()?;
    let core_tenant_subdirectories = option_value(&values, "--memory-core-tenant-subdirectories")
        .is_some_and(|value| value == "1");
    let blink_max_open_shards = option_value(&values, "--memory-blink-max-open-shards")
        .unwrap_or_else(|| "1024".to_owned())
        .parse()?;
    let blink_cache_pages = option_value(&values, "--memory-blink-cache-pages")
        .map(|value| value.parse())
        .transpose()?;
    if blink_max_open_shards == 0 {
        return Err("B-link maximum open shards must be nonzero".into());
    }
    let blink_checkpoint_policy =
        match option_value(&values, "--blink-checkpoint-wal-bytes").as_deref() {
            None | Some("off") => BlinkCheckpointPolicy::Disabled,
            Some(value) => {
                let threshold_bytes = value.parse::<u64>()?;
                if threshold_bytes == 0 {
                    return Err("B-link checkpoint threshold must be positive".into());
                }
                BlinkCheckpointPolicy::WalBytesThreshold { threshold_bytes }
            }
        };
    let blink_checkpoint = blink_checkpoint::Options::parse(
        option_value(&values, "--blink-wal-layout").as_deref(),
        option_value(&values, "--blink-checkpoint-execution").as_deref(),
        option_value(&values, "--blink-wal-segment-bytes").as_deref(),
        backend == "blink",
    )?;
    if backend != "blink" && option_value(&values, "--blink-checkpoint-wal-bytes").is_some() {
        return Err("B-link checkpoint policy requires the blink backend".into());
    }
    if tenant_count == 0 {
        return Err("memory tenant count must be nonzero".into());
    }
    let revision_allocator = match option_value(&values, "--revision-allocator")
        .unwrap_or_else(|| "serialized".to_owned())
        .as_str()
    {
        "serialized" => RevisionAllocatorMode::Serialized,
        "reserved-ranges" => RevisionAllocatorMode::ReservedRanges,
        other => return Err(format!("unknown revision allocator {other}").into()),
    };
    let condition_key_lock_mode = match option_value(&values, "--condition-key-lock-mode")
        .unwrap_or_else(|| "exclusive".to_owned())
        .as_str()
    {
        "exclusive" => ConditionKeyLockMode::Exclusive,
        "shared" => ConditionKeyLockMode::Shared,
        other => return Err(format!("unknown condition key lock mode {other}").into()),
    };
    if matches!(backend.as_str(), "minimal-native" | "minimal-native-shared")
        && (revision_allocator != RevisionAllocatorMode::ReservedRanges
            || condition_key_lock_mode != ConditionKeyLockMode::Exclusive)
    {
        return Err(
            "minimal-native requires reserved-range revisions and exclusive condition-key locks"
                .into(),
        );
    }
    Ok(Arguments {
        backend,
        shared_resources,
        memory_cgroup_path,
        pressure_interval_ms,
        data_dir,
        certificate_path,
        private_key_path,
        health_path,
        source_revision,
        worker_binary_sha256,
        parent_binary_sha256,
        parent_pid,
        response_budget,
        max_mutations,
        max_concurrent_requests,
        revision_allocator,
        condition_key_lock_mode,
        blocking_slots,
        blocking_admission,
        engine_health_interval_ms,
        tenant_count,
        core_tenant_subdirectories,
        blink_max_open_shards,
        blink_cache_pages,
        blink_checkpoint_policy,
        blink_checkpoint,
    })
}

fn install_parent_death_signal(expected_parent_pid: u32) -> WorkerResult<()> {
    #[cfg(target_os = "linux")]
    let current_parent_pid = unsafe { libc::getppid() as u32 };
    #[cfg(not(target_os = "linux"))]
    let current_parent_pid = 0;
    if current_parent_pid != expected_parent_pid {
        return Err("diagnostic worker parent changed before backend startup".into());
    }
    #[cfg(target_os = "linux")]
    {
        let result = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) };
        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if unsafe { libc::getppid() as u32 } != expected_parent_pid {
            return Err(
                "diagnostic worker parent exited while installing parent-death signal".into(),
            );
        }
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        Err("memory diagnostic workers require Linux parent-death signals".into())
    }
}

fn argument_value(arguments: &[String], name: &str) -> WorkerResult<String> {
    option_value(arguments, name).ok_or_else(|| format!("missing argument {name}").into())
}

fn option_value(arguments: &[String], name: &str) -> Option<String> {
    let position = arguments.iter().position(|argument| argument == name)?;
    arguments.get(position + 1).cloned()
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

fn decode_raw_row(key_bytes: &[u8], value_bytes: &[u8]) -> Result<Document, Error> {
    if key_bytes.first() != Some(&1)
        || value_bytes.len() < 10
        || value_bytes[0] != 1
        || value_bytes[1] != 1
    {
        return Err(Error::corruption("raw row has an unexpected encoding"));
    }
    let key = DocumentKey::decode(&key_bytes[1..])
        .map_err(|error| Error::corruption(error.to_string()))?;
    Ok(Document {
        key,
        value: value_bytes[10..].to_vec(),
        revision: Revision::ZERO,
    })
}
