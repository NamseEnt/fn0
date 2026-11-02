#![recursion_limit = "256"]

#[path = "support/blink_checkpoint.rs"]
mod blink_checkpoint;
#[path = "support/pressure_observations.rs"]
mod pressure_observations;
#[path = "support/shutdown_trace.rs"]
mod shutdown_trace;

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
    worker_phase: String,
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
    blink_demand_paging: bool,
    blink_paged_cache_bytes: u64,
    blink_memory_limit_bytes: u64,
    blink_memory_per_tenant_bytes: u64,
    blink_disk_limit_bytes: u64,
    blink_disk_per_tenant_bytes: u64,
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
                #[cfg(feature = "blink-demand-paging")]
                let blink_shared_resources = if arguments.blink_demand_paging {
                    Some(paged_resource_config(arguments)?)
                } else {
                    None
                };
                #[cfg(not(feature = "blink-demand-paging"))]
                if arguments.blink_demand_paging {
                    return Err("worker binary was not built with blink-demand-paging".into());
                }
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
                    #[cfg(feature = "blink-demand-paging")]
                    blink_shared_resources,
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
        "checkpoint_debt_bytes_current": metrics.checkpoint_debt_bytes_current,
        "checkpoint_debt_bytes_max": metrics.checkpoint_debt_bytes_max,
        "threshold_excess_total_nanos": metrics.threshold_excess_total_nanos,
        "threshold_excess_current_nanos": metrics.threshold_excess_current_nanos,
        "threshold_excess_since_unix_nanos": metrics.threshold_excess_since_unix_nanos,
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
        "checkpoint_requests_queued": metrics.checkpoint_requests_queued,
        "checkpoint_requests_pending": metrics.checkpoint_requests_pending,
        "checkpoint_requests_cancelled": metrics.checkpoint_requests_cancelled,
        "checkpoint_request_wait_nanos_total": metrics.checkpoint_request_wait_nanos_total,
        "checkpoint_request_wait_nanos_max": metrics.checkpoint_request_wait_nanos_max,
        "capacity_wait_count": metrics.capacity_wait_count,
        "capacity_wait_nanos": metrics.capacity_wait_nanos,
        "coordinator_pause_nanos": metrics.coordinator_pause_nanos,
        "retained_image_bytes_normalized_page_images": metrics.retained_image_bytes,
        "peak_retained_image_bytes_normalized_page_images": metrics.peak_retained_image_bytes,
        "tasks_running": metrics.tasks_running,
        "peak_tasks_running": metrics.peak_tasks_running,
    })
}

#[cfg(feature = "blink-demand-paging")]
fn paged_resource_config(
    arguments: &Arguments,
) -> WorkerResult<dodb_server::BlinkSharedResourceConfig> {
    if arguments.blink_memory_limit_bytes < 32 * 1024 * 1024
        || arguments.blink_memory_per_tenant_bytes < 32 * 1024 * 1024
        || arguments.blink_memory_per_tenant_bytes > arguments.blink_memory_limit_bytes
        || arguments.blink_disk_limit_bytes < 32 * 1024 * 1024
        || arguments.blink_disk_per_tenant_bytes < 32 * 1024 * 1024
        || arguments.blink_disk_per_tenant_bytes > arguments.blink_disk_limit_bytes
        || arguments.blink_paged_cache_bytes < dodb_storage::PAGE_SIZE as u64
    {
        return Err("B-link paging resource limits are invalid".into());
    }
    let progress_pool_bytes = (64 * 1024 * 1024).min(arguments.blink_memory_limit_bytes / 2);
    Ok(dodb_server::BlinkSharedResourceConfig {
        memory: dodb_storage::blink::resource::ResourceLimits {
            global_bytes: arguments.blink_memory_limit_bytes,
            per_tenant_bytes: arguments.blink_memory_per_tenant_bytes,
            progress_pool_bytes,
            max_tenants: arguments.blink_max_open_shards,
            max_pending_waiters: 4096,
            global_category_bytes: [arguments.blink_memory_limit_bytes; 7],
            per_tenant_category_bytes: [arguments.blink_memory_per_tenant_bytes; 7],
        },
        disk: dodb_storage::blink::resource::DiskResourceLimits {
            global_bytes: arguments.blink_disk_limit_bytes,
            per_tenant_bytes: arguments.blink_disk_per_tenant_bytes,
            max_tenants: arguments.blink_max_open_shards,
            global_category_bytes: [arguments.blink_disk_limit_bytes; 5],
            per_tenant_category_bytes: [arguments.blink_disk_per_tenant_bytes; 5],
        },
        open_tenant_metadata_bytes: 4096,
        paged_read_only: false,
        paged_writer_enabled: true,
        maximum_working_pages: 64,
        page_loader: dodb_storage::blink::page_cache::PageLoaderConfig {
            maximum_cache_bytes: arguments.blink_paged_cache_bytes,
            minimum_decoded_page_bytes: dodb_storage::PAGE_SIZE as u64,
            maximum_load_reconstruction_bytes: arguments
                .blink_paged_cache_bytes
                .max(16 * dodb_storage::PAGE_SIZE as u64),
            maximum_concurrent_loads: 8,
            maximum_followers: 128,
        },
        maximum_cached_index_nodes: 4096,
        maximum_open_source_handles: 4096,
        maximum_roots_per_tenant: 4096,
        source_handle_charge_bytes: 256,
        source_gc_reservation_bytes: 4096 + 4096 * 256,
        checkpoint_capacity_wait_timeout: Duration::from_secs(30),
        maximum_checkpoint_debt_bytes_global: arguments.blink_disk_limit_bytes / 2,
        maximum_checkpoint_debt_bytes_per_tenant: arguments.blink_disk_per_tenant_bytes / 2,
    })
}

#[cfg(feature = "blink-demand-paging")]
fn page_loader_metrics_json(
    metrics: &dodb_storage::blink::page_cache::PageLoaderMetrics,
) -> serde_json::Value {
    json!({
        "cache_misses": metrics.cache_misses,
        "evictions": metrics.evictions,
        "source_reads": metrics.source_reads,
        "wal_reconstructions": metrics.wal_reconstructions,
        "coalesced_loads": metrics.coalesced_loads,
        "chain_entries_inspected": metrics.chain_entries_inspected,
        "maximum_chain_entries": metrics.maximum_chain_entries,
        "current_cache_bytes": metrics.current_cache_bytes,
        "peak_cache_bytes": metrics.peak_cache_bytes,
        "active_loads": metrics.active_loads,
        "active_followers": metrics.active_followers,
    })
}

#[cfg(feature = "blink-demand-paging")]
fn source_manager_metrics_json(
    metrics: &dodb_storage::blink::source_manager::SourceManagerMetrics,
) -> serde_json::Value {
    json!({
        "source_opens": metrics.source_opens,
        "active_source_handles": metrics.active_source_handles,
        "active_roots": metrics.active_roots,
        "reserved_roots": metrics.reserved_roots,
        "oldest_root_age_millis": metrics.oldest_root_age_millis,
    })
}

#[cfg(feature = "blink-demand-paging")]
fn paged_resource_snapshot_json(
    snapshot: dodb_server::BlinkPagedResourceSnapshot,
) -> serde_json::Value {
    let memory_current = snapshot.memory.current_by_category;
    let memory_peak = snapshot.memory.peak_by_category;
    let disk_current = snapshot.disk.current_by_category;
    let disk_peak = snapshot.disk.peak_by_category;
    let page_loader = snapshot
        .tenants
        .iter()
        .map(|tenant| tenant.page_loader)
        .fold(
            dodb_storage::blink::page_cache::PageLoaderMetrics::default(),
            |mut total, metrics| {
                total.cache_misses += metrics.cache_misses;
                total.evictions += metrics.evictions;
                total.source_reads += metrics.source_reads;
                total.wal_reconstructions += metrics.wal_reconstructions;
                total.coalesced_loads += metrics.coalesced_loads;
                total.chain_entries_inspected += metrics.chain_entries_inspected;
                total.maximum_chain_entries = total
                    .maximum_chain_entries
                    .max(metrics.maximum_chain_entries);
                total.current_cache_bytes += metrics.current_cache_bytes;
                total.peak_cache_bytes += metrics.peak_cache_bytes;
                total.active_loads += metrics.active_loads;
                total.active_followers += metrics.active_followers;
                total
            },
        );
    let active_source_handles = snapshot
        .tenants
        .iter()
        .map(|tenant| tenant.source_manager.active_source_handles)
        .sum::<usize>();
    let active_roots = snapshot
        .tenants
        .iter()
        .map(|tenant| tenant.source_manager.active_roots)
        .sum::<usize>();
    let reserved_roots = snapshot
        .tenants
        .iter()
        .map(|tenant| tenant.source_manager.reserved_roots)
        .sum::<usize>();
    let source_opens = snapshot
        .tenants
        .iter()
        .map(|tenant| tenant.source_manager.source_opens)
        .sum::<u64>();
    let oldest_root_age_millis = snapshot
        .tenants
        .iter()
        .filter_map(|tenant| tenant.source_manager.oldest_root_age_millis)
        .max();
    json!({
        "status": "sampled",
        "maximum_open_shards": snapshot.maximum_open_shards,
        "applied_paging_config": {
            "paged_read_only": snapshot.applied_config.paged_read_only,
            "paged_writer_enabled": snapshot.applied_config.paged_writer_enabled,
            "open_tenant_metadata_bytes": snapshot.applied_config.open_tenant_metadata_bytes,
            "maximum_working_pages": snapshot.applied_config.maximum_working_pages,
            "page_loader": {
                "maximum_cache_bytes": snapshot.applied_config.page_loader.maximum_cache_bytes,
                "minimum_decoded_page_bytes": snapshot.applied_config.page_loader.minimum_decoded_page_bytes,
                "maximum_load_reconstruction_bytes": snapshot.applied_config.page_loader.maximum_load_reconstruction_bytes,
                "maximum_concurrent_loads": snapshot.applied_config.page_loader.maximum_concurrent_loads,
                "maximum_followers": snapshot.applied_config.page_loader.maximum_followers,
            },
            "maximum_cached_index_nodes": snapshot.applied_config.maximum_cached_index_nodes,
            "maximum_open_source_handles": snapshot.applied_config.maximum_open_source_handles,
            "maximum_roots_per_tenant": snapshot.applied_config.maximum_roots_per_tenant,
            "source_handle_charge_bytes": snapshot.applied_config.source_handle_charge_bytes,
            "source_gc_reservation_bytes": snapshot.applied_config.source_gc_reservation_bytes,
            "checkpoint_capacity_wait_timeout_ms": snapshot.applied_config.checkpoint_capacity_wait_timeout.as_millis(),
            "maximum_checkpoint_debt_bytes_global": snapshot.applied_config.maximum_checkpoint_debt_bytes_global,
            "maximum_checkpoint_debt_bytes_per_tenant": snapshot.applied_config.maximum_checkpoint_debt_bytes_per_tenant,
        },
        "format_identities": {
            "version_index": dodb_storage::blink::version_index::VERSION_INDEX_FORMAT_ID,
            "delta_chain_sidecar": dodb_storage::blink::version_chain::DELTA_CHAIN_FORMAT_ID,
        },
        "memory_limits": {
            "global_bytes": snapshot.memory_limits.global_bytes,
            "per_tenant_bytes": snapshot.memory_limits.per_tenant_bytes,
            "progress_pool_bytes": snapshot.memory_limits.progress_pool_bytes,
            "max_tenants": snapshot.memory_limits.max_tenants,
            "max_pending_waiters": snapshot.memory_limits.max_pending_waiters,
            "global_category_bytes": snapshot.memory_limits.global_category_bytes,
            "per_tenant_category_bytes": snapshot.memory_limits.per_tenant_category_bytes,
        },
        "memory": {
            "current_by_category_bytes": snapshot.memory.current_by_category,
            "peak_by_category_bytes": snapshot.memory.peak_by_category,
            "categories": {
                "decoded_payload": {"current_bytes": memory_current[0], "peak_bytes": memory_peak[0]},
                "metadata": {"current_bytes": memory_current[1], "peak_bytes": memory_peak[1]},
                "load_reconstruction": {"current_bytes": memory_current[2], "peak_bytes": memory_peak[2]},
                "queued_foreground": {"current_bytes": memory_current[3], "peak_bytes": memory_peak[3]},
                "write_preparation": {"current_bytes": memory_current[4], "peak_bytes": memory_peak[4]},
                "dirty_images": {"current_bytes": memory_current[5], "peak_bytes": memory_peak[5]},
                "checkpoint_validation": {"current_bytes": memory_current[6], "peak_bytes": memory_peak[6]},
            },
            "current_total_bytes": snapshot.memory.current_total,
            "peak_total_bytes": snapshot.memory.peak_total,
            "current_progress_bytes": snapshot.memory.current_progress,
            "peak_progress_bytes": snapshot.memory.peak_progress,
            "active_tenants": snapshot.memory.active_tenants,
            "pending_waiters": snapshot.memory.pending_waiters,
            "rejected_reservations": snapshot.memory.rejected_reservations,
        },
        "disk_limits": {
            "global_bytes": snapshot.disk_limits.global_bytes,
            "per_tenant_bytes": snapshot.disk_limits.per_tenant_bytes,
            "max_tenants": snapshot.disk_limits.max_tenants,
            "global_category_bytes": snapshot.disk_limits.global_category_bytes,
            "per_tenant_category_bytes": snapshot.disk_limits.per_tenant_category_bytes,
        },
        "disk": {
            "current_by_category_bytes": snapshot.disk.current_by_category,
            "peak_by_category_bytes": snapshot.disk.peak_by_category,
            "categories": {
                "wal": {"current_bytes": disk_current[0], "peak_bytes": disk_peak[0]},
                "checkpoint_output": {"current_bytes": disk_current[1], "peak_bytes": disk_peak[1]},
                "version_index": {"current_bytes": disk_current[2], "peak_bytes": disk_peak[2]},
                "scratch": {"current_bytes": disk_current[3], "peak_bytes": disk_peak[3]},
                "retained_source": {"current_bytes": disk_current[4], "peak_bytes": disk_peak[4]},
            },
            "current_total_bytes": snapshot.disk.current_total,
            "peak_total_bytes": snapshot.disk.peak_total,
            "active_tenants": snapshot.disk.active_tenants,
            "pending_waiters": snapshot.disk.pending_waiters,
            "rejected_reservations": snapshot.disk.rejected_reservations,
        },
        "disk_bytes": {
            "reachable_source_bytes": null,
            "reachable_source_bytes_reason": "the service does not expose a bounded reachability walk in periodic snapshots",
            "retained_source_bytes": disk_current[4],
            "temporary_checkpoint_and_scratch_bytes": disk_current[1] + disk_current[3],
        },
        "page_loader_totals": {
            "cache_misses": page_loader.cache_misses,
            "evictions": page_loader.evictions,
            "source_reads": page_loader.source_reads,
            "wal_reconstructions": page_loader.wal_reconstructions,
            "coalesced_loads": page_loader.coalesced_loads,
            "chain_entries_inspected": page_loader.chain_entries_inspected,
            "maximum_chain_entries": page_loader.maximum_chain_entries,
            "current_cache_bytes": page_loader.current_cache_bytes,
            "peak_cache_bytes_sum_of_tenant_peaks": page_loader.peak_cache_bytes,
            "active_loads": page_loader.active_loads,
            "active_followers": page_loader.active_followers,
        },
        "source_manager_totals": {
            "source_opens": source_opens,
            "active_source_handles": active_source_handles,
            "active_roots": active_roots,
            "reserved_roots": reserved_roots,
            "oldest_root_age_millis": oldest_root_age_millis,
            "paged_route_enabled": true,
            "loaded_paged_tenant_count": snapshot.tenants.len(),
        },
        "per_tenant": snapshot.tenants.iter().map(|tenant| json!({
            "tenant_id": tenant.tenant_id,
            "page_loader": page_loader_metrics_json(&tenant.page_loader),
            "source_manager": source_manager_metrics_json(&tenant.source_manager),
        })).collect::<Vec<_>>(),
        "allocator_retained_bytes": null,
        "allocator_retained_bytes_reason": "allocator and representation-specific retained heap are not exposed",
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
            "demand_paging_enabled": arguments.blink_demand_paging,
            "paged_cache_bytes_requested": if arguments.blink_demand_paging { Some(arguments.blink_paged_cache_bytes) } else { None },
            "paged_memory_global_bytes_requested": if arguments.blink_demand_paging { Some(arguments.blink_memory_limit_bytes) } else { None },
            "paged_memory_per_tenant_bytes_requested": if arguments.blink_demand_paging { Some(arguments.blink_memory_per_tenant_bytes) } else { None },
            "paged_disk_global_bytes_requested": if arguments.blink_demand_paging { Some(arguments.blink_disk_limit_bytes) } else { None },
            "paged_disk_per_tenant_bytes_requested": if arguments.blink_demand_paging { Some(arguments.blink_disk_per_tenant_bytes) } else { None },
            "feature_compiled": cfg!(feature = "blink-demand-paging"),
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

    #[cfg(feature = "blink-demand-paging")]
    fn blink_paged_resource_snapshot(&self) -> serde_json::Value {
        match self {
            Self::Blink(service) => match service.paged_resource_snapshot() {
                Ok(Some(snapshot)) => paged_resource_snapshot_json(snapshot),
                Ok(None) => json!({
                    "status": "resident",
                    "reason": "paged resources were not requested",
                }),
                Err(error) => json!({"status": "error", "reason": error.to_string()}),
            },
            _ => json!({
                "status": "unsupported",
                "reason": "backend is not B-link",
            }),
        }
    }

    #[cfg(not(feature = "blink-demand-paging"))]
    fn blink_paged_resource_snapshot(&self) -> serde_json::Value {
        match self {
            Self::Blink(_) => json!({
                "status": "resident",
                "reason": "worker binary was built without blink-demand-paging",
            }),
            _ => json!({
                "status": "unsupported",
                "reason": "backend is not B-link",
            }),
        }
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
    let trace = shutdown_trace::ShutdownTrace::new(
        arguments.pressure_interval_ms > 0,
        &arguments.source_revision,
    );
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let execution = runtime.block_on(run(arguments, memory_membership, &trace));
    trace.emit(if execution.is_ok() {
        "async_run_return_success"
    } else {
        "async_run_return_error"
    });
    trace.emit("runtime_destruction_start");
    drop(runtime);
    trace.emit("runtime_destruction_end");
    execution
}

async fn run(
    arguments: Arguments,
    memory_membership: Option<serde_json::Value>,
    trace: &shutdown_trace::ShutdownTrace,
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
    let mut paged_measurement_start = serde_json::Value::Null;
    let mut paged_measurement_end = serde_json::Value::Null;
    let mut pressure_samples = pressure_observations::PressureObservations::default();
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
                if pressure_samples.can_record() {
                    let sample = json!({
                        "timestamp_unix_ms": dodb_rocksdb::memory_pressure::unix_time_ms(),
                        "phase": pressure_phase,
                        "shared_resources": backend.shared_resource_snapshot(),
                        "rocksdb_properties": backend.engine_health_observer().map(|observer| observer.sample()),
                        "blink_checkpoints": backend.periodic_blink_checkpoint_metrics(),
                        "blink_paged_resources": backend.blink_paged_resource_snapshot(),
                    });
                    pressure_samples.record(&sample)?;
                } else { pressure_samples.mark_truncated(); }
                continue;
            }
        };
        match command.as_str() {
            "start-measurement" => {
                trace.emit("command_received_start_measurement");
                pressure_phase = "measurement-including-drain";
                shared_measurement_start = backend.shared_resource_snapshot();
                paged_measurement_start = backend.blink_paged_resource_snapshot();
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
                trace.emit("command_received_stop_measurement");
                trace.emit("final_sampler_capture_start");
                pressure_phase = "after-measurement";
                shared_measurement_end = backend.shared_resource_snapshot();
                paged_measurement_end = backend.blink_paged_resource_snapshot();
                let final_sampler_capture = match health_sampler.take() {
                    Some(sampler) => sampler.finish(),
                    None if arguments.engine_health_interval_ms == 0 => {
                        Ok(engine_health_disabled_report())
                    }
                    None => Ok(json!({"status": "unsupported"})),
                };
                match final_sampler_capture {
                    Ok(report) => {
                        health_report = report;
                        trace.emit("final_sampler_capture_end");
                    }
                    Err(error) => {
                        trace.emit("final_sampler_capture_error");
                        return Err(error.into());
                    }
                }
            }
            "shutdown" => {
                trace.emit("command_received_shutdown");
                break;
            }
            other => return Err(format!("unknown diagnostic worker command {other}").into()),
        }
    }
    if let Some(sampler) = health_sampler.take() {
        trace.emit("final_sampler_capture_start");
        let final_sampler_capture = sampler.finish();
        match final_sampler_capture {
            Ok(report) => {
                health_report = report;
                trace.emit("final_sampler_capture_end");
            }
            Err(error) => {
                trace.emit("final_sampler_capture_error");
                return Err(error.into());
            }
        }
    }
    trace.emit("final_property_capture_start");
    let shutdown_property_snapshot = backend.engine_health_observer().map_or_else(
        || json!({"status": "unsupported", "reason": "backend exposes no RocksDB properties"}),
        |observer| observer.sample(),
    );
    let shared_before_shutdown = backend.shared_resource_snapshot();
    let paged_resources_before_shutdown = backend.blink_paged_resource_snapshot();
    trace.emit("final_property_capture_end");
    trace.emit("async_server_shutdown_start");
    let shutdown = tokio::time::timeout(Duration::from_secs(15), async {
        server.shutdown().await;
        (&mut server_task).await
    })
    .await;
    match shutdown {
        Ok(Ok(Ok(()))) => trace.emit("async_server_shutdown_end"),
        Ok(Ok(Err(error))) => {
            trace.emit("async_server_shutdown_error");
            return Err(Box::new(error) as WorkerError);
        }
        Ok(Err(error)) => {
            trace.emit("async_server_shutdown_error");
            return Err(Box::new(error) as WorkerError);
        }
        Err(_) => {
            trace.emit("async_server_shutdown_error");
            server_task.abort();
            return Err("diagnostic worker shutdown exceeded 15 seconds".into());
        }
    }
    trace.emit("final_checkpoint_metrics_capture_start");
    let blink_checkpoint_metrics = backend.blink_checkpoint_metrics();
    trace.emit("final_checkpoint_metrics_capture_end");
    trace.emit("server_drop_start");
    drop(server);
    trace.emit("server_drop_end");
    trace.emit("backend_drop_start");
    drop(backend);
    trace.emit("backend_drop_end");
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
    trace.emit("health_report_write_start");
    let health_report_write = pressure_observations::write_health_report(
        &arguments.health_path,
        json!({
            "backend": arguments.backend.clone(),
            "worker_phase": arguments.worker_phase,
            "blink_configuration": if arguments.backend == "blink" {
                backend_configuration_json(&arguments)
            } else {
                serde_json::Value::Null
            },
            "shared_resource_snapshots": {
                "measurement_start": shared_measurement_start,
                "measurement_end": shared_measurement_end,
                "paged_measurement_start": paged_measurement_start,
                "paged_measurement_end": paged_measurement_end,
                "before_shutdown": shared_before_shutdown,
                "scope": "control-command boundaries; end includes drain; not continuous or peak instrumentation",
            },
            "effective_backend_configuration": backend_configuration_json(&arguments),
            "backend_memory_cgroup": memory_membership,
            "periodic_pressure": {
                "enabled": arguments.pressure_interval_ms > 0,
                "interval_ms": arguments.pressure_interval_ms,
                "sample_limit": pressure_observations::SAMPLE_LIMIT,
                "serialized_sample_byte_limit": pressure_observations::SERIALIZED_BYTE_LIMIT,
                "serialized_sample_bytes": pressure_samples.serialized_bytes(),
                "storage_scope": "serialized sample bytes bound retained observation volume, not exact serde_json heap allocation; observations are charged to the worker budget",
                "samples_truncated": pressure_samples.truncated(),
                "scope": "worker lifetime after readiness; non-atomic sampled observations; not instantaneous peaks; measurement phase includes drain; shared cache/WBM counted once in shared_resources; repeated per-DB cache properties must not be summed",
                "unsupported": ["allocator retained heap", "generation pin ages", "request and checkpoint queue representation bytes"],
            },
            "blink_checkpoint_metrics": blink_checkpoint_metrics,
            "blink_paged_resources_before_shutdown": paged_resources_before_shutdown,
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
        }),
        &pressure_samples,
    );
    match health_report_write {
        Ok(()) => trace.emit("health_report_write_end"),
        Err(error) => {
            trace.emit("health_report_write_error");
            return Err(error.into());
        }
    }
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
    let worker_phase =
        option_value(&values, "--memory-worker-phase").unwrap_or_else(|| "unspecified".to_owned());
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
    let blink_demand_paging = match option_value(&values, "--memory-blink-demand-paging").as_deref()
    {
        None | Some("off") => false,
        Some("on") => true,
        Some(value) => return Err(format!("unsupported B-link demand paging mode {value}").into()),
    };
    let blink_paged_cache_bytes = option_value(&values, "--memory-blink-paged-cache-bytes")
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(2 * 1024 * 1024);
    let blink_memory_limit_bytes = option_value(&values, "--memory-blink-memory-limit-bytes")
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(512 * 1024 * 1024);
    let blink_memory_per_tenant_bytes =
        option_value(&values, "--memory-blink-memory-per-tenant-bytes")
            .map(|value| value.parse())
            .transpose()?
            .unwrap_or(64 * 1024 * 1024);
    let blink_disk_limit_bytes = option_value(&values, "--memory-blink-disk-limit-bytes")
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(4 * 1024 * 1024 * 1024);
    let blink_disk_per_tenant_bytes = option_value(&values, "--memory-blink-disk-per-tenant-bytes")
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(512 * 1024 * 1024);
    if blink_max_open_shards == 0 {
        return Err("B-link maximum open shards must be nonzero".into());
    }
    if blink_demand_paging && !cfg!(feature = "blink-demand-paging") {
        return Err("worker binary was not built with blink-demand-paging".into());
    }
    if blink_demand_paging
        && (blink_memory_limit_bytes < 32 * 1024 * 1024
            || blink_memory_per_tenant_bytes < 32 * 1024 * 1024
            || blink_memory_per_tenant_bytes > blink_memory_limit_bytes
            || blink_disk_limit_bytes < 32 * 1024 * 1024
            || blink_disk_per_tenant_bytes < 32 * 1024 * 1024
            || blink_disk_per_tenant_bytes > blink_disk_limit_bytes
            || blink_paged_cache_bytes < dodb_storage::PAGE_SIZE as u64)
    {
        return Err("B-link paging resource limits are invalid".into());
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
    if blink_demand_paging
        && (backend != "blink"
            || !matches!(
                blink_checkpoint.configuration()["layout"].as_str(),
                Some("segmented-bases" | "segmented-bases-coalesced")
            ))
    {
        return Err(
            "B-link demand paging requires the blink backend and a segmented durable-base layout"
                .into(),
        );
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
        worker_phase,
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
        blink_demand_paging,
        blink_paged_cache_bytes,
        blink_memory_limit_bytes,
        blink_memory_per_tenant_bytes,
        blink_disk_limit_bytes,
        blink_disk_per_tenant_bytes,
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
