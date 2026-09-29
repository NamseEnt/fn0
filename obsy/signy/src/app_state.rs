use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::backpressure::IngestGate;
use crate::clock::Clock;
use crate::config::Config;
use crate::journal::Journal;
use crate::memtable::MemTable;
use crate::metrics::RuntimeMetrics;
use crate::object_storage::RemoteCache;
use crate::part_registry::PartRegistry;
use crate::shutdown::ShutdownState;
use crate::tenant_policy::TenantPolicy;
use crate::tenant_quota::TenantQuota;
use crate::trace_registry::TraceRegistry;

/// Runtime resources shared by HTTP handlers and background workers.
pub struct AppState {
    pub config: Arc<Config>,
    pub query_scan_semaphore: Arc<tokio::sync::Semaphore>,
    /// Trace scans have their own slots: a trace scan decodes whole-span JSON
    /// payloads, so its cost profile is unlike a log scan's, and sharing the
    /// log semaphore would let either surface starve the other.
    pub trace_scan_semaphore: Arc<tokio::sync::Semaphore>,
    /// Metric scans likewise: a metric scan is Gorilla decode plus per-step
    /// folds, and sharing slots with either other surface would let one cost
    /// profile starve the others.
    pub metric_scan_semaphore: Arc<tokio::sync::Semaphore>,
    /// What this process has committed to work in flight, queries and
    /// background writers together. A query prices itself against it before it
    /// scans, so a refusal arrives on the request rather than partway through
    /// one. Shared with `Config`, which is how flush and compaction reach it.
    pub memory_account: Arc<crate::memory_budget::MemoryAccount>,
    /// Bounds live tail connections. Held for the life of a socket, unlike the
    /// scan semaphores, which a tail borrows per poll.
    pub tail_semaphore: Arc<tokio::sync::Semaphore>,
    pub memtable: Arc<MemTable>,
    pub journal: Arc<Journal>,
    /// Free space on the data directory's filesystem, refreshed by a task.
    /// Read by the ingest gate and by `/metrics`.
    pub disk: Arc<crate::disk::DiskSpace>,
    pub parts: Arc<PartRegistry>,
    pub trace_parts: Arc<TraceRegistry>,
    pub series_parts: Arc<crate::series_registry::SeriesRegistry>,
    pub flush_healthy: Arc<AtomicBool>,
    pub merge_healthy: Arc<AtomicBool>,
    pub retention_healthy: Arc<AtomicBool>,
    pub remote_cache: Option<Arc<RemoteCache>>,
    /// The tenant→retention snapshot, read by query handlers to clamp a
    /// requested range to what the tenant is still entitled to see.
    pub tenant_policy: Arc<TenantPolicy>,
    pub metrics: Arc<RuntimeMetrics>,
    pub shutdown: Arc<ShutdownState>,
    /// Shared with the OTLP service so both protocols answer to one set of
    /// thresholds.
    pub ingest_gate: Arc<IngestGate>,
    /// Per-tenant ingest rate, from the pushed policy. Distinct from
    /// `ingest_gate`, which asks whether *this instance* can take more; this
    /// one asks whether *this tenant* may send more.
    pub tenant_quota: Arc<TenantQuota>,
    /// Outstanding deletion requests. Consulted by the one scan every read path
    /// funnels through, so a deleted line stops being readable the moment the
    /// request is accepted rather than when its part is next rewritten.
    pub delete_requests: Arc<crate::delete_requests::DeleteRequests>,
    /// Wall clock. Injected so the boundaries that depend on it — which
    /// timestamps ingest accepts, what range a query defaults to — can be
    /// tested at the edge instead of relative to whatever `now` happened to be.
    pub clock: Arc<Clock>,
}

/// Dependencies that are created during startup or by a test fixture.
pub struct AppStateDependencies {
    pub memtable: Arc<MemTable>,
    pub journal: Arc<Journal>,
    pub parts: Arc<PartRegistry>,
    pub trace_parts: Arc<TraceRegistry>,
    pub series_parts: Arc<crate::series_registry::SeriesRegistry>,
    pub flush_healthy: Arc<AtomicBool>,
    pub merge_healthy: Arc<AtomicBool>,
    pub retention_healthy: Arc<AtomicBool>,
    pub remote_cache: Option<Arc<RemoteCache>>,
    pub tenant_policy: Arc<TenantPolicy>,
    pub metrics: Arc<RuntimeMetrics>,
    pub shutdown: Arc<ShutdownState>,
    pub clock: Arc<Clock>,
    /// Supplied by startup, which has already loaded what previous runs
    /// accepted. A test fixture leaves it out and gets an empty registry.
    pub delete_requests: Option<Arc<crate::delete_requests::DeleteRequests>>,
}

impl AppState {
    /// Construct state and all endpoint semaphores from one configuration.
    /// Keeping this in one place prevents production and test limits from
    /// silently diverging.
    pub fn from_config(config: Arc<Config>, dependencies: AppStateDependencies) -> Self {
        // Sampled once here so the gate never reads an unmeasured disk in
        // production; the sampler task keeps it current from then on.
        let disk = Arc::new(crate::disk::DiskSpace::sampled(&config.data_dir));
        let ingest_gate = Arc::new(IngestGate::new(
            dependencies.journal.clone(),
            config.clone(),
            dependencies.metrics.clone(),
            disk.clone(),
        ));
        let delete_requests = dependencies.delete_requests.unwrap_or_else(|| {
            Arc::new(crate::delete_requests::DeleteRequests::new(
                dependencies
                    .remote_cache
                    .as_ref()
                    .map(|cache| cache.storage.clone()),
            ))
        });
        let tenant_quota = Arc::new(TenantQuota::new(
            config.clone(),
            dependencies.metrics.clone(),
            dependencies.tenant_policy.clone(),
            dependencies.parts.clone(),
            dependencies.trace_parts.clone(),
            dependencies.series_parts.clone(),
        ));
        Self {
            ingest_gate,
            disk,
            tenant_quota,
            delete_requests,
            clock: dependencies.clock,
            query_scan_semaphore: Arc::new(tokio::sync::Semaphore::new(
                config.max_concurrent_query_scans,
            )),
            trace_scan_semaphore: Arc::new(tokio::sync::Semaphore::new(
                config.max_concurrent_trace_scans,
            )),
            metric_scan_semaphore: Arc::new(tokio::sync::Semaphore::new(
                config.max_concurrent_metric_scans,
            )),
            memory_account: config.memory_account.clone(),
            tail_semaphore: Arc::new(tokio::sync::Semaphore::new(config.max_concurrent_tails)),
            config,
            memtable: dependencies.memtable,
            journal: dependencies.journal,
            parts: dependencies.parts,
            trace_parts: dependencies.trace_parts,
            series_parts: dependencies.series_parts,
            flush_healthy: dependencies.flush_healthy,
            merge_healthy: dependencies.merge_healthy,
            retention_healthy: dependencies.retention_healthy,
            remote_cache: dependencies.remote_cache,
            tenant_policy: dependencies.tenant_policy,
            metrics: dependencies.metrics,
            shutdown: dependencies.shutdown,
        }
    }
}
