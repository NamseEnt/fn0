use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CStr, c_char, c_void};
use std::path::{Path, PathBuf};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use dodb_core::{
    DocumentKey, Error, PrimaryKey, Revision, RevisionState, SortKey, TenantId,
    TransactionCondition, TransactionConflict, TransactionMutation, TransactionRequest,
};
use dodb_protocol::{
    MAX_STORAGE_ENCODED_KEY_SIZE, ProtocolLimits, ResponseEnvelope, encode_response,
};
use dodb_service::{
    DodbService, ExecutionBudget, Request, Response, ServiceFuture, ShutdownFuture,
    TransactionOutcome,
};
use tokio::sync::{Notify, OnceCell, Semaphore};

const USER_KEY_PREFIX: u8 = 1;
const METADATA_COUNTER_KEY: &[u8] = &[0, 1];
const DEFAULT_REVISION_RANGE_SIZE: u64 = 4_096;
const VALUE_FORMAT: u8 = 1;
const PRESENT_TAG: u8 = 1;
const MISSING_TAG: u8 = 2;
const METRIC_COUNT: usize = 16;
const HISTOGRAM_BUCKETS: usize = 64;
const MAX_METRIC_TENANTS: usize = 64;
const READ_METRIC_COUNT: usize = 13;

tokio::task_local! {
    static RESEARCH_READ_KIND: ResearchReadKind;
    static RESEARCH_READ_CONTEXT: Arc<ReadRequestContext>;
}

struct ReadRequestContext {
    kind: ResearchReadKind,
    started: Instant,
    first_permit: std::sync::atomic::AtomicBool,
    semaphore_wait_nanos: std::sync::atomic::AtomicU64,
    blocking_dispatch_nanos: std::sync::atomic::AtomicU64,
    blocking_closure_nanos: std::sync::atomic::AtomicU64,
    completion_handoff_nanos: std::sync::atomic::AtomicU64,
}

impl ReadRequestContext {
    fn add_duration(counter: &std::sync::atomic::AtomicU64, duration: Duration) {
        counter.fetch_add(
            duration.as_nanos().min(u64::MAX as u128) as u64,
            Ordering::Relaxed,
        );
    }
}

pub async fn with_research_read_kind<F>(kind: ResearchReadKind, future: F) -> F::Output
where
    F: std::future::Future,
{
    RESEARCH_READ_KIND.scope(kind, future).await
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum ResearchReadKind {
    Get,
    PreparatoryGet,
    Query16,
}

#[derive(Clone, Copy)]
#[repr(usize)]
enum ReadMetric {
    PrePermit,
    SemaphoreWait,
    BlockingDispatch,
    BlockingClosure,
    CompletionHandoff,
    TenantDatabaseLookup,
    RocksDbGet,
    GetCopyDecode,
    QueryIteratorSetup,
    QueryIteratorCalls,
    QueryCopyDecodeBudget,
    ResponseProcessing,
    ServiceRequestTotal,
}

const READ_METRIC_NAMES: [&str; READ_METRIC_COUNT] = [
    "request_to_permit_acquire_start",
    "semaphore_wait",
    "blocking_dispatch_wait",
    "blocking_closure_duration",
    "blocking_completion_handoff",
    "tenant_database_lookup",
    "rocksdb_get_call",
    "get_copy_decode",
    "query_snapshot_iterator_setup",
    "query_iterator_calls_sum",
    "query_copy_decode_budget",
    "response_processing",
    "service_request_total",
];

const READ_KIND_NAMES: [&str; 3] = ["get", "preparatory_get", "query16"];

#[derive(Clone, Copy)]
#[repr(usize)]
enum Metric {
    RequestDuration,
    ReadSemaphoreWait,
    WriteSemaphoreWait,
    BlockingDispatchWait,
    UserGetForUpdateDuration,
    UserKeyLockWait,
    RevisionCounterGetForUpdateDuration,
    RevisionCounterLockWait,
    ConditionCheck,
    CommitDuration,
    RollbackDuration,
    CounterHeldDuration,
    MutationConstruction,
    AllocatorWait,
    RangeReservationDuration,
    UserCommitDuration,
}

const METRIC_NAMES: [&str; METRIC_COUNT] = [
    "service_request_duration",
    "read_semaphore_wait",
    "write_semaphore_wait",
    "blocking_dispatch_wait",
    "user_key_get_for_update_duration",
    "user_key_lock_wait",
    "revision_counter_get_for_update_duration",
    "revision_counter_lock_wait",
    "condition_check_duration",
    "commit_duration",
    "rollback_duration",
    "counter_held_duration_approx",
    "mutation_construction_duration",
    "revision_allocator_wait",
    "revision_range_reservation_duration",
    "user_commit_duration",
];

#[derive(Default)]
pub struct ResearchMetrics {
    tenants: DashMap<u64, Arc<TenantResearchMetrics>>,
    stage: std::sync::atomic::AtomicUsize,
    tenant_slots: std::sync::atomic::AtomicUsize,
    omitted_tenant_metric_events: std::sync::atomic::AtomicU64,
}

#[derive(Default)]
struct TenantResearchMetrics {
    warmup: [Histogram; METRIC_COUNT],
    measurement: [Histogram; METRIC_COUNT],
    warmup_reads: [[Histogram; READ_METRIC_COUNT]; 3],
    measurement_reads: [[Histogram; READ_METRIC_COUNT]; 3],
    warmup_outcomes: OutcomeCounts,
    measurement_outcomes: OutcomeCounts,
}

#[derive(Default)]
struct OutcomeCounts {
    successful_writes: std::sync::atomic::AtomicU64,
    condition_rejections: std::sync::atomic::AtomicU64,
    other_errors: std::sync::atomic::AtomicU64,
    user_key_lock_wait_count: std::sync::atomic::AtomicU64,
    revision_counter_lock_wait_count: std::sync::atomic::AtomicU64,
    range_reservations: std::sync::atomic::AtomicU64,
    reservation_commits: std::sync::atomic::AtomicU64,
    user_commits: std::sync::atomic::AtomicU64,
    user_transaction_counter_accesses: std::sync::atomic::AtomicU64,
}

struct Histogram {
    count: std::sync::atomic::AtomicU64,
    total_nanos: std::sync::atomic::AtomicU64,
    buckets: [std::sync::atomic::AtomicU64; HISTOGRAM_BUCKETS],
}

impl Default for Histogram {
    fn default() -> Self {
        Self {
            count: std::sync::atomic::AtomicU64::new(0),
            total_nanos: std::sync::atomic::AtomicU64::new(0),
            buckets: std::array::from_fn(|_| std::sync::atomic::AtomicU64::new(0)),
        }
    }
}

impl ResearchMetrics {
    pub fn set_measurement_phase(&self) {
        self.stage.store(2, Ordering::Release);
    }

    pub fn set_warmup_phase(&self) {
        self.stage.store(1, Ordering::Release);
    }

    pub fn snapshot(&self) -> serde_json::Value {
        self.snapshot_inner()
    }

    fn tenant(&self, tenant: TenantId) -> Option<Arc<TenantResearchMetrics>> {
        match self.tenants.entry(tenant.get()) {
            dashmap::mapref::entry::Entry::Occupied(entry) => Some(Arc::clone(entry.get())),
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                let reserved =
                    self.tenant_slots
                        .fetch_update(Ordering::AcqRel, Ordering::Relaxed, |slots| {
                            (slots < MAX_METRIC_TENANTS).then_some(slots + 1)
                        });
                if reserved.is_err() {
                    self.omitted_tenant_metric_events
                        .fetch_add(1, Ordering::Relaxed);
                    return None;
                }
                let metrics = Arc::new(TenantResearchMetrics::default());
                entry.insert(Arc::clone(&metrics));
                Some(metrics)
            }
        }
    }

    fn record(&self, tenant: TenantId, metric: Metric, elapsed: std::time::Duration) {
        let Some(tenant_metrics) = self.tenant(tenant) else {
            return;
        };
        let stage = self.stage.load(Ordering::Acquire);
        if stage > 0 {
            let phase = if stage == 2 {
                &tenant_metrics.measurement
            } else {
                &tenant_metrics.warmup
            };
            phase[metric as usize].record(elapsed.as_nanos().min(u64::MAX as u128) as u64);
        }
    }

    fn record_read(
        &self,
        tenant: TenantId,
        kind: ResearchReadKind,
        metric: ReadMetric,
        elapsed: std::time::Duration,
    ) {
        let Some(tenant_metrics) = self.tenant(tenant) else {
            return;
        };
        let stage = self.stage.load(Ordering::Acquire);
        if stage > 0 {
            let phase = if stage == 2 {
                &tenant_metrics.measurement_reads
            } else {
                &tenant_metrics.warmup_reads
            };
            phase[kind as usize][metric as usize]
                .record(elapsed.as_nanos().min(u64::MAX as u128) as u64);
        }
    }

    fn snapshot_inner(&self) -> serde_json::Value {
        let tenants = self.tenants.iter().map(|entry| {
            let metrics = entry.value();
            let render_phase = |histograms: &[Histogram; METRIC_COUNT]| {
                let mut values = serde_json::Map::new();
                for (index, name) in METRIC_NAMES.iter().enumerate() {
                    values.insert((*name).to_owned(), histograms[index].snapshot());
                }
                serde_json::Value::Object(values)
            };
            let render_reads = |histograms: &[[Histogram; READ_METRIC_COUNT]; 3]| {
                let mut kinds = serde_json::Map::new();
                for (kind_index, kind_name) in READ_KIND_NAMES.iter().enumerate() {
                    let mut values = serde_json::Map::new();
                    for (metric_index, metric_name) in READ_METRIC_NAMES.iter().enumerate() {
                        values.insert(
                            (*metric_name).to_owned(),
                            histograms[kind_index][metric_index].snapshot(),
                        );
                    }
                    kinds.insert((*kind_name).to_owned(), serde_json::Value::Object(values));
                }
                serde_json::Value::Object(kinds)
            };
            let outcomes = |counts: &OutcomeCounts| serde_json::json!({
                "successful_writes": counts.successful_writes.load(Ordering::Relaxed),
                "condition_rejections": counts.condition_rejections.load(Ordering::Relaxed),
                "other_errors": counts.other_errors.load(Ordering::Relaxed),
                "user_key_lock_wait_count": counts.user_key_lock_wait_count.load(Ordering::Relaxed),
                "revision_counter_lock_wait_count": counts.revision_counter_lock_wait_count.load(Ordering::Relaxed),
                "range_reservations": counts.range_reservations.load(Ordering::Relaxed),
                "reservation_commits": counts.reservation_commits.load(Ordering::Relaxed),
                "user_commits": counts.user_commits.load(Ordering::Relaxed),
                "user_transaction_counter_accesses": counts
                    .user_transaction_counter_accesses
                    .load(Ordering::Relaxed),
                "reservation_counter_accesses": counts
                    .range_reservations
                    .load(Ordering::Relaxed),
            });
            (format!("tenant_{}", entry.key()), serde_json::json!({
                "warmup": render_phase(&metrics.warmup),
                "measurement": render_phase(&metrics.measurement),
                "read_stages": {
                    "warmup": render_reads(&metrics.warmup_reads),
                    "measurement": render_reads(&metrics.measurement_reads),
                },
                "outcomes": {
                    "warmup": outcomes(&metrics.warmup_outcomes),
                    "measurement": outcomes(&metrics.measurement_outcomes),
                }
            }))
        }).collect::<serde_json::Map<_, _>>();
        serde_json::json!({
            "enabled": true,
            "histogram_percentiles": "approximate upper bounds of base-2 nanosecond buckets",
            "lock_wait_source": "RocksDB 11.8.1 PerfContext key_lock_wait_time and key_lock_wait_count deltas around each GetForUpdate call",
            "tenant_limit": MAX_METRIC_TENANTS,
            "omitted_tenant_metric_events": self.omitted_tenant_metric_events.load(Ordering::Relaxed),
            "tenants": tenants,
        })
    }
}

impl TenantResearchMetrics {
    fn record(&self, metric: Metric, elapsed: std::time::Duration, stage: usize) {
        if stage > 0 {
            let phase = if stage == 2 {
                &self.measurement
            } else {
                &self.warmup
            };
            phase[metric as usize].record(elapsed.as_nanos().min(u64::MAX as u128) as u64);
        }
    }
}

impl Histogram {
    fn record(&self, nanos: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_nanos.fetch_add(nanos, Ordering::Relaxed);
        let bucket = if nanos == 0 {
            0
        } else {
            (u64::BITS - nanos.leading_zeros()) as usize
        };
        self.buckets[bucket.min(HISTOGRAM_BUCKETS - 1)].fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> serde_json::Value {
        let count = self.count.load(Ordering::Relaxed);
        let total_nanos = self.total_nanos.load(Ordering::Relaxed);
        let percentile = |numerator: u64| -> Option<f64> {
            if count == 0 {
                return None;
            }
            let target = (count - 1).saturating_mul(numerator).div_ceil(100);
            let mut seen = 0u64;
            for (bucket_index, bucket) in self.buckets.iter().enumerate() {
                seen = seen.saturating_add(bucket.load(Ordering::Relaxed));
                if seen > target {
                    let upper_bound = if bucket_index == 0 {
                        1
                    } else {
                        1u64.checked_shl(bucket_index as u32).unwrap_or(u64::MAX)
                    };
                    return Some(upper_bound as f64 / 1_000_000.0);
                }
            }
            Some(u64::MAX as f64 / 1_000_000.0)
        };
        serde_json::json!({
            "count": count,
            "total_ms_across_requests": total_nanos as f64 / 1_000_000.0,
            "mean_ms": if count == 0 { None } else { Some(total_nanos as f64 / count as f64 / 1_000_000.0) },
            "p50_ms_approx": percentile(50),
            "p95_ms_approx": percentile(95),
            "p99_ms_approx": percentile(99),
        })
    }
}

unsafe extern "C" {
    fn dodb_rocksdb_open(path: *const c_char, error: *mut *mut c_char) -> *mut c_void;
    fn dodb_rocksdb_close(database: *mut c_void);
    fn dodb_rocksdb_iterator_new(
        database: *mut c_void,
        start_key: *const u8,
        start_key_length: usize,
        error: *mut *mut c_char,
    ) -> *mut c_void;
    fn dodb_rocksdb_iterator_valid(iterator: *mut c_void) -> i32;
    fn dodb_rocksdb_iterator_key(iterator: *mut c_void, key_length: *mut usize) -> *const u8;
    fn dodb_rocksdb_iterator_value(iterator: *mut c_void, value_length: *mut usize) -> *const u8;
    fn dodb_rocksdb_iterator_status(iterator: *mut c_void, error: *mut *mut c_char) -> i32;
    fn dodb_rocksdb_iterator_next(iterator: *mut c_void);
    fn dodb_rocksdb_iterator_destroy(iterator: *mut c_void);
    fn dodb_rocksdb_begin(database: *mut c_void, error: *mut *mut c_char) -> *mut c_void;
    fn dodb_rocksdb_get(
        database: *mut c_void,
        key: *const u8,
        key_length: usize,
        value: *mut *mut u8,
        value_length: *mut usize,
        error: *mut *mut c_char,
    ) -> i32;
    fn dodb_rocksdb_transaction_get_for_update(
        transaction: *mut c_void,
        key: *const u8,
        key_length: usize,
        value: *mut *mut u8,
        value_length: *mut usize,
        collect_lock_wait: i32,
        lock_wait_nanos: *mut u64,
        lock_wait_count: *mut u64,
        error: *mut *mut c_char,
    ) -> i32;
    fn dodb_rocksdb_transaction_put(
        transaction: *mut c_void,
        key: *const u8,
        key_length: usize,
        value: *const u8,
        value_length: usize,
        error: *mut *mut c_char,
    ) -> i32;
    fn dodb_rocksdb_transaction_commit(transaction: *mut c_void, error: *mut *mut c_char) -> i32;
    fn dodb_rocksdb_transaction_rollback(transaction: *mut c_void);
    fn dodb_rocksdb_transaction_destroy(transaction: *mut c_void);
    fn dodb_rocksdb_set_research_metrics(enabled: bool);
    fn dodb_rocksdb_value_free(value: *mut u8);
    fn dodb_rocksdb_error_free(error: *mut c_char);
}

#[derive(Clone, Debug)]
pub struct RocksDbServiceConfig {
    pub data_dir: PathBuf,
    pub max_blocking_operations: usize,
    pub research_metrics: bool,
    pub detailed_write_metrics: bool,
    pub revision_allocator: RevisionAllocatorMode,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RevisionAllocatorMode {
    #[default]
    Serialized,
    ReservedRanges,
}

impl RocksDbServiceConfig {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
            max_blocking_operations: 8,
            research_metrics: false,
            detailed_write_metrics: true,
            revision_allocator: RevisionAllocatorMode::Serialized,
        }
    }
}

pub struct RocksDbService {
    data_dir: PathBuf,
    databases: DashMap<TenantId, Arc<OnceCell<Arc<TenantDatabase>>>>,
    blocking_operations: Arc<Semaphore>,
    blocking_operation_limit: usize,
    research_metrics: Option<Arc<ResearchMetrics>>,
    detailed_write_metrics: bool,
    revision_allocator: RevisionAllocatorMode,
    lifecycle: LifecycleCoordinator,
    #[cfg(test)]
    blocking_operation_hook: std::sync::Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

struct TenantDatabase {
    handle: Arc<RocksDbHandle>,
    revision_allocator: RevisionAllocatorMode,
    reserved_revisions: Mutex<ReservedRevisionRange>,
    revision_range_size: u64,
}

#[derive(Default)]
struct ReservedRevisionRange {
    next: u64,
    end: u64,
}

struct RocksDbHandle(NonNull<c_void>);

unsafe impl Send for RocksDbHandle {}
unsafe impl Sync for RocksDbHandle {}

struct TransactionGuard {
    handle: NonNull<c_void>,
    _database: Arc<RocksDbHandle>,
    active: bool,
}

struct TenantReadIterator {
    handle: NonNull<c_void>,
    _database: Arc<RocksDbHandle>,
}

struct LifecycleCoordinator {
    state: Mutex<LifecycleState>,
    active_requests: AtomicUsize,
    changed: Notify,
    #[cfg(test)]
    shutdown_hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

struct LifecycleState {
    shutting_down: bool,
}

struct RequestLease<'service> {
    lifecycle: &'service LifecycleCoordinator,
}

struct ShutdownLease<'service> {
    lifecycle: &'service LifecycleCoordinator,
}

impl RocksDbService {
    pub fn new(config: RocksDbServiceConfig) -> Result<Self, Error> {
        if config.max_blocking_operations == 0 {
            return Err(Error::invalid_request(
                "max_blocking_operations must be nonzero",
            ));
        }
        std::fs::create_dir_all(&config.data_dir)?;
        Ok(Self {
            data_dir: config.data_dir,
            databases: DashMap::new(),
            blocking_operations: Arc::new(Semaphore::new(config.max_blocking_operations)),
            blocking_operation_limit: config.max_blocking_operations,
            research_metrics: config
                .research_metrics
                .then(|| Arc::new(ResearchMetrics::default())),
            detailed_write_metrics: config.detailed_write_metrics,
            revision_allocator: config.revision_allocator,
            lifecycle: LifecycleCoordinator::new(),
            #[cfg(test)]
            blocking_operation_hook: std::sync::Mutex::new(None),
        })
    }

    pub async fn shutdown(&self) {
        let _shutdown_lease = self.lifecycle.begin_shutdown().await;
        let mut blocking_permits = Vec::with_capacity(self.blocking_operation_limit);
        for _ in 0..self.blocking_operation_limit {
            if let Ok(permit) = Arc::clone(&self.blocking_operations).acquire_owned().await {
                blocking_permits.push(permit);
            }
        }
        let cells = self
            .databases
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect::<Vec<_>>();
        self.databases.clear();
        let _ = tokio::task::spawn_blocking(move || drop(cells)).await;
        drop(blocking_permits);
    }

    pub fn tenant_path(&self, tenant: TenantId) -> PathBuf {
        self.data_dir.join(format!("tenant-{}", tenant.get()))
    }

    pub fn research_metrics_snapshot(&self) -> Option<serde_json::Value> {
        self.research_metrics
            .as_ref()
            .map(|metrics| metrics.snapshot())
    }

    pub fn research_metrics_handle(&self) -> Option<Arc<ResearchMetrics>> {
        self.research_metrics.clone()
    }

    pub fn set_research_measurement_phase(&self) {
        if let Some(metrics) = &self.research_metrics {
            metrics.set_measurement_phase();
        }
    }

    pub fn set_research_warmup_phase(&self) {
        if let Some(metrics) = &self.research_metrics {
            metrics.set_warmup_phase();
        }
    }

    async fn database(
        &self,
        tenant: TenantId,
        create: bool,
    ) -> Result<Option<Arc<TenantDatabase>>, Error> {
        let path = self.tenant_path(tenant);
        if !create {
            let path_exists = self
                .run_blocking(tenant, false, move || Ok(path.exists()))
                .await?;
            if !path_exists {
                return Ok(None);
            }
        }
        let path = self.tenant_path(tenant);
        let cell = self
            .databases
            .entry(tenant)
            .or_insert_with(|| Arc::new(OnceCell::new()))
            .clone();
        let semaphore = Arc::clone(&self.blocking_operations);
        let research_metrics = self.research_metrics.clone();
        let revision_allocator = self.revision_allocator;
        let database = cell
            .get_or_try_init(|| async move {
                let semaphore_started = Instant::now();
                let permit = semaphore
                    .acquire_owned()
                    .await
                    .map_err(|_| Error::overloaded("blocking RocksDB executor is closed"))?;
                if let Some(metrics) = &research_metrics {
                    metrics.record(
                        tenant,
                        Metric::WriteSemaphoreWait,
                        semaphore_started.elapsed(),
                    );
                }
                let dispatch_started = Instant::now();
                tokio::task::spawn_blocking(move || {
                    let _permit = permit;
                    if let Some(metrics) = &research_metrics {
                        metrics.record(
                            tenant,
                            Metric::BlockingDispatchWait,
                            dispatch_started.elapsed(),
                        );
                    }
                    TenantDatabase::open_with_allocator(
                        &path,
                        revision_allocator,
                        DEFAULT_REVISION_RANGE_SIZE,
                    )
                    .map(Arc::new)
                })
                .await
                .map_err(|error| Error::recovery(format!("RocksDB open task failed: {error}")))?
            })
            .await?;
        Ok(Some(Arc::clone(database)))
    }

    async fn run_blocking<T, F>(
        &self,
        tenant: TenantId,
        is_write: bool,
        operation: F,
    ) -> Result<T, Error>
    where
        T: Send + 'static,
        F: FnOnce() -> Result<T, Error> + Send + 'static,
    {
        let semaphore_started = Instant::now();
        let read_context = RESEARCH_READ_CONTEXT.try_with(Arc::clone).ok();
        if let (Some(context), Some(metrics)) = (&read_context, &self.research_metrics) {
            if context.first_permit.swap(false, Ordering::AcqRel) {
                metrics.record_read(
                    tenant,
                    context.kind,
                    ReadMetric::PrePermit,
                    semaphore_started.saturating_duration_since(context.started),
                );
            }
        }
        let permit = Arc::clone(&self.blocking_operations)
            .acquire_owned()
            .await
            .map_err(|_| Error::overloaded("blocking RocksDB executor is closed"))?;
        if let Some(metrics) = &self.research_metrics {
            metrics.record(
                tenant,
                if is_write {
                    Metric::WriteSemaphoreWait
                } else {
                    Metric::ReadSemaphoreWait
                },
                semaphore_started.elapsed(),
            );
            if let Some(context) = &read_context {
                ReadRequestContext::add_duration(
                    &context.semaphore_wait_nanos,
                    semaphore_started.elapsed(),
                );
            }
        }
        let metrics = self.research_metrics.clone();
        let read_context_for_task = read_context.clone();
        let collect_metrics = metrics
            .as_ref()
            .is_some_and(|metrics| metrics.stage.load(Ordering::Acquire) > 0)
            && is_write
            && self.detailed_write_metrics;
        #[cfg(test)]
        let operation_hook = self.blocking_operation_hook.lock().unwrap().clone();
        let dispatch_started = Instant::now();
        let (result, closure_finished) = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let started = Instant::now();
            if let Some(metrics) = &metrics {
                metrics.record(
                    tenant,
                    Metric::BlockingDispatchWait,
                    dispatch_started.elapsed(),
                );
                if let Some(context) = &read_context_for_task {
                    ReadRequestContext::add_duration(
                        &context.blocking_dispatch_nanos,
                        dispatch_started.elapsed(),
                    );
                }
            }
            if collect_metrics {
                unsafe { dodb_rocksdb_set_research_metrics(true) };
            }
            #[cfg(test)]
            if let Some(operation_hook) = operation_hook {
                operation_hook();
            }
            let result = operation();
            if let Some(context) = &read_context_for_task {
                ReadRequestContext::add_duration(
                    &context.blocking_closure_nanos,
                    started.elapsed(),
                );
            }
            if collect_metrics {
                unsafe { dodb_rocksdb_set_research_metrics(false) };
            }
            (result, Instant::now())
        })
        .await
        .map_err(|error| Error::recovery(format!("RocksDB operation task failed: {error}")))?;
        if let Some(context) = &read_context {
            ReadRequestContext::add_duration(
                &context.completion_handoff_nanos,
                closure_finished.elapsed(),
            );
        }
        result
    }

    async fn execute_request(
        &self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> Result<Response, Error> {
        let started = Instant::now();
        let default_read_kind = match &request {
            Request::Get { .. } => Some(ResearchReadKind::Get),
            Request::Query { .. } => Some(ResearchReadKind::Query16),
            _ => None,
        };
        let read_kind = RESEARCH_READ_KIND
            .try_with(|kind| *kind)
            .ok()
            .or(default_read_kind);
        let is_mutation = request.is_mutation();
        let read_context = read_kind.map(|kind| {
            Arc::new(ReadRequestContext {
                kind,
                started,
                first_permit: std::sync::atomic::AtomicBool::new(true),
                semaphore_wait_nanos: std::sync::atomic::AtomicU64::new(0),
                blocking_dispatch_nanos: std::sync::atomic::AtomicU64::new(0),
                blocking_closure_nanos: std::sync::atomic::AtomicU64::new(0),
                completion_handoff_nanos: std::sync::atomic::AtomicU64::new(0),
            })
        });
        let result = if let (Some(kind), Some(context)) = (read_kind, read_context.clone()) {
            RESEARCH_READ_KIND
                .scope(
                    kind,
                    RESEARCH_READ_CONTEXT
                        .scope(context, self.execute_request_inner(tenant, request, budget)),
                )
                .await
        } else {
            self.execute_request_inner(tenant, request, budget).await
        };
        if let Some(metrics) = &self.research_metrics {
            if let Some(tenant_metrics) = metrics.tenant(tenant) {
                let stage = metrics.stage.load(Ordering::Acquire);
                metrics.record(tenant, Metric::RequestDuration, started.elapsed());
                if let Some(context) = &read_context {
                    for (metric, nanos) in [
                        (
                            ReadMetric::SemaphoreWait,
                            context.semaphore_wait_nanos.load(Ordering::Relaxed),
                        ),
                        (
                            ReadMetric::BlockingDispatch,
                            context.blocking_dispatch_nanos.load(Ordering::Relaxed),
                        ),
                        (
                            ReadMetric::BlockingClosure,
                            context.blocking_closure_nanos.load(Ordering::Relaxed),
                        ),
                        (
                            ReadMetric::CompletionHandoff,
                            context.completion_handoff_nanos.load(Ordering::Relaxed),
                        ),
                    ] {
                        metrics.record_read(
                            tenant,
                            context.kind,
                            metric,
                            Duration::from_nanos(nanos),
                        );
                    }
                    metrics.record_read(
                        tenant,
                        context.kind,
                        ReadMetric::ServiceRequestTotal,
                        started.elapsed(),
                    );
                }
                let outcomes = if stage == 2 {
                    &tenant_metrics.measurement_outcomes
                } else {
                    &tenant_metrics.warmup_outcomes
                };
                if stage > 0 {
                    match &result {
                        Ok(_) if is_mutation => {
                            outcomes.successful_writes.fetch_add(1, Ordering::Relaxed)
                        }
                        Err(Error::Conflict(_)) => outcomes
                            .condition_rejections
                            .fetch_add(1, Ordering::Relaxed),
                        Err(_) => outcomes.other_errors.fetch_add(1, Ordering::Relaxed),
                        Ok(_) => 0,
                    };
                }
            }
        }
        result
    }

    async fn execute_request_inner(
        &self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> Result<Response, Error> {
        let _request_lease = self.lifecycle.begin_request().await;
        match request {
            Request::Get { key } => {
                let database_started = Instant::now();
                let database = self.database(tenant, false).await?;
                if let Some(metrics) = &self.research_metrics {
                    if let Some(kind) = RESEARCH_READ_CONTEXT.try_with(|context| context.kind).ok()
                    {
                        metrics.record_read(
                            tenant,
                            kind,
                            ReadMetric::TenantDatabaseLookup,
                            database_started.elapsed(),
                        );
                    }
                }
                let state = match database {
                    Some(database) => {
                        let metrics = self.research_metrics.clone();
                        let kind = RESEARCH_READ_CONTEXT.try_with(|context| context.kind).ok();
                        self.run_blocking(tenant, false, move || {
                            database.get_with_metrics(tenant, &key, metrics, kind)
                        })
                        .await?
                    }
                    None => RevisionState::missing(Revision::ZERO),
                };
                let response_started = Instant::now();
                let response = Response::Get(state);
                enforce_response_budget(&response, budget)?;
                if let (Some(metrics), Ok(kind)) = (
                    &self.research_metrics,
                    RESEARCH_READ_CONTEXT.try_with(|context| context.kind),
                ) {
                    metrics.record_read(
                        tenant,
                        kind,
                        ReadMetric::ResponseProcessing,
                        response_started.elapsed(),
                    );
                }
                Ok(response)
            }
            Request::Put { key, value } => {
                let database = self
                    .database(tenant, true)
                    .await?
                    .ok_or_else(|| Error::invariant("created tenant database was absent"))?;
                let request = TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put { key, value }],
                );
                let stage = self
                    .research_metrics
                    .as_ref()
                    .map_or(0, |metrics| metrics.stage.load(Ordering::Acquire));
                let transaction_metrics = self
                    .research_metrics
                    .as_ref()
                    .filter(|_| self.detailed_write_metrics)
                    .and_then(|metrics| metrics.tenant(tenant));
                let result = self
                    .run_blocking(tenant, true, move || {
                        database.transact_with_metrics(request, transaction_metrics, stage)
                    })
                    .await?;
                let revision = result
                    .revision
                    .ok_or_else(|| Error::invariant("put transaction returned no revision"))?;
                let response = Response::Put(revision);
                enforce_response_budget(&response, budget)?;
                Ok(response)
            }
            Request::Delete { key } => {
                let database = self
                    .database(tenant, true)
                    .await?
                    .ok_or_else(|| Error::invariant("created tenant database was absent"))?;
                let request =
                    TransactionRequest::new(Vec::new(), vec![TransactionMutation::Delete { key }]);
                let stage = self
                    .research_metrics
                    .as_ref()
                    .map_or(0, |metrics| metrics.stage.load(Ordering::Acquire));
                let transaction_metrics = self
                    .research_metrics
                    .as_ref()
                    .filter(|_| self.detailed_write_metrics)
                    .and_then(|metrics| metrics.tenant(tenant));
                let result = self
                    .run_blocking(tenant, true, move || {
                        database.transact_with_metrics(request, transaction_metrics, stage)
                    })
                    .await?;
                let revision = result
                    .revision
                    .ok_or_else(|| Error::invariant("delete transaction returned no revision"))?;
                let response = Response::Delete(revision);
                enforce_response_budget(&response, budget)?;
                Ok(response)
            }
            Request::Transact { request } => {
                request.validate()?;
                let has_mutations = !request.mutations.is_empty();
                let database = self.database(tenant, has_mutations).await?;
                let result = match database {
                    Some(database) => {
                        let stage = self
                            .research_metrics
                            .as_ref()
                            .map_or(0, |metrics| metrics.stage.load(Ordering::Acquire));
                        let transaction_metrics = self
                            .research_metrics
                            .as_ref()
                            .filter(|_| self.detailed_write_metrics)
                            .and_then(|metrics| metrics.tenant(tenant));
                        self.run_blocking(tenant, has_mutations, move || {
                            database.transact_with_metrics(request, transaction_metrics, stage)
                        })
                        .await?
                    }
                    None => evaluate_uncreated(&request)?,
                };
                let outcome = match result.commit_lsn {
                    Some(commit_lsn) => TransactionOutcome::committed(commit_lsn),
                    None => TransactionOutcome::conditions_satisfied(),
                };
                let response = Response::Transact(outcome);
                enforce_response_budget(&response, budget)?;
                Ok(response)
            }
            Request::Query {
                pk,
                exclusive_after_sk,
                limit,
            } => {
                let database_started = Instant::now();
                let database = self.database(tenant, false).await?;
                if let Some(metrics) = &self.research_metrics {
                    if let Some(kind) = RESEARCH_READ_CONTEXT.try_with(|context| context.kind).ok()
                    {
                        metrics.record_read(
                            tenant,
                            kind,
                            ReadMetric::TenantDatabaseLookup,
                            database_started.elapsed(),
                        );
                    }
                }
                let rows = match database {
                    Some(database) => {
                        let metrics = self.research_metrics.clone();
                        let kind = RESEARCH_READ_CONTEXT.try_with(|context| context.kind).ok();
                        self.run_blocking(tenant, false, move || {
                            database.query_with_metrics(
                                tenant,
                                &pk,
                                exclusive_after_sk.as_ref(),
                                limit,
                                budget,
                                metrics,
                                kind,
                            )
                        })
                        .await?
                    }
                    None => {
                        let response = Response::Query(Vec::new());
                        enforce_response_budget(&response, budget)?;
                        return Ok(response);
                    }
                };
                let response_started = Instant::now();
                let response = Response::Query(rows);
                if let (Some(metrics), Ok(kind)) = (
                    &self.research_metrics,
                    RESEARCH_READ_CONTEXT.try_with(|context| context.kind),
                ) {
                    metrics.record_read(
                        tenant,
                        kind,
                        ReadMetric::ResponseProcessing,
                        response_started.elapsed(),
                    );
                }
                Ok(response)
            }
            Request::Scan {
                exclusive_after_key,
                limit,
            } => {
                let rows = match self.database(tenant, false).await? {
                    Some(database) => {
                        self.run_blocking(tenant, false, move || {
                            database.scan(exclusive_after_key.as_ref(), limit, budget)
                        })
                        .await?
                    }
                    None => {
                        let response = Response::Scan(Vec::new());
                        enforce_response_budget(&response, budget)?;
                        return Ok(response);
                    }
                };
                Ok(Response::Scan(rows))
            }
        }
    }
}

impl LifecycleCoordinator {
    fn new() -> Self {
        Self {
            state: Mutex::new(LifecycleState {
                shutting_down: false,
            }),
            active_requests: AtomicUsize::new(0),
            changed: Notify::new(),
            #[cfg(test)]
            shutdown_hook: Mutex::new(None),
        }
    }

    async fn begin_request(&self) -> RequestLease<'_> {
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let accepted = {
                let state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if state.shutting_down {
                    false
                } else {
                    self.active_requests.fetch_add(1, Ordering::AcqRel);
                    true
                }
            };
            if accepted {
                return RequestLease { lifecycle: self };
            }
            notified.await;
        }
    }

    async fn begin_shutdown(&self) -> ShutdownLease<'_> {
        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let started = {
                let mut state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if state.shutting_down {
                    false
                } else {
                    state.shutting_down = true;
                    true
                }
            };
            if started {
                break;
            }
            notified.await;
        }

        loop {
            let notified = self.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.active_requests.load(Ordering::Acquire) == 0 {
                break;
            }
            notified.await;
        }
        #[cfg(test)]
        if let Some(shutdown_hook) = self
            .shutdown_hook
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            shutdown_hook();
        }
        ShutdownLease { lifecycle: self }
    }
}

impl Drop for RequestLease<'_> {
    fn drop(&mut self) {
        self.lifecycle
            .active_requests
            .fetch_sub(1, Ordering::AcqRel);
        self.lifecycle.changed.notify_waiters();
    }
}

impl Drop for ShutdownLease<'_> {
    fn drop(&mut self) {
        let mut state = self
            .lifecycle
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.shutting_down = false;
        drop(state);
        self.lifecycle.changed.notify_waiters();
    }
}

impl TenantDatabase {
    #[cfg(test)]
    fn open(path: &Path) -> Result<Self, Error> {
        Self::open_with_allocator(
            path,
            RevisionAllocatorMode::Serialized,
            DEFAULT_REVISION_RANGE_SIZE,
        )
    }

    fn open_with_allocator(
        path: &Path,
        revision_allocator: RevisionAllocatorMode,
        revision_range_size: u64,
    ) -> Result<Self, Error> {
        if revision_range_size == 0 {
            return Err(Error::invalid_input("revision range size must be nonzero"));
        }
        let path = path
            .to_str()
            .ok_or_else(|| Error::invalid_input("tenant path is not valid UTF-8"))?;
        let path = std::ffi::CString::new(path)
            .map_err(|_| Error::invalid_input("tenant path contains a NUL byte"))?;
        let mut error = std::ptr::null_mut();
        let handle = unsafe { dodb_rocksdb_open(path.as_ptr(), &mut error) };
        let handle = NonNull::new(handle).ok_or_else(|| ffi_error(error, "RocksDB open failed"))?;
        Ok(Self {
            handle: Arc::new(RocksDbHandle(handle)),
            revision_allocator,
            reserved_revisions: Mutex::new(ReservedRevisionRange::default()),
            revision_range_size,
        })
    }

    #[cfg(test)]
    fn get(&self, key: &DocumentKey) -> Result<RevisionState, Error> {
        self.get_with_metrics(TenantId::new(0), key, None, None)
    }

    fn get_with_metrics(
        &self,
        tenant: TenantId,
        key: &DocumentKey,
        metrics: Option<Arc<ResearchMetrics>>,
        kind: Option<ResearchReadKind>,
    ) -> Result<RevisionState, Error> {
        let encoded_key = user_key(key);
        let mut value = std::ptr::null_mut();
        let mut value_length = 0;
        let mut error = std::ptr::null_mut();
        let native_started = Instant::now();
        let result = unsafe {
            dodb_rocksdb_get(
                self.handle.0.as_ptr(),
                encoded_key.as_ptr(),
                encoded_key.len(),
                &mut value,
                &mut value_length,
                &mut error,
            )
        };
        if let (Some(metrics), Some(kind)) = (&metrics, kind) {
            metrics.record_read(
                tenant,
                kind,
                ReadMetric::RocksDbGet,
                native_started.elapsed(),
            );
        }
        let decode_started = Instant::now();
        let decoded = match result {
            1 => Ok(RevisionState::missing(Revision::ZERO)),
            2 => Err(ffi_error(error, "RocksDB get failed")),
            _ => {
                let bytes = unsafe { copy_and_free_value(value, value_length) };
                decode_state(&bytes)
            }
        };
        if let (Some(metrics), Some(kind)) = (&metrics, kind) {
            metrics.record_read(
                tenant,
                kind,
                ReadMetric::GetCopyDecode,
                decode_started.elapsed(),
            );
        }
        decoded
    }

    fn read_iterator(&self, start_key: &[u8]) -> Result<TenantReadIterator, Error> {
        let mut error = std::ptr::null_mut();
        let iterator = unsafe {
            dodb_rocksdb_iterator_new(
                self.handle.0.as_ptr(),
                start_key.as_ptr(),
                start_key.len(),
                &mut error,
            )
        };
        NonNull::new(iterator)
            .map(|handle| TenantReadIterator {
                handle,
                _database: Arc::clone(&self.handle),
            })
            .ok_or_else(|| ffi_error(error, "RocksDB snapshot iterator creation failed"))
    }

    #[cfg(test)]
    fn query(
        &self,
        pk: &PrimaryKey,
        exclusive_after_sk: Option<&SortKey>,
        limit: usize,
        maximum_response_bytes: ExecutionBudget,
    ) -> Result<Vec<dodb_service::Document>, Error> {
        self.query_with_metrics(
            TenantId::new(0),
            pk,
            exclusive_after_sk,
            limit,
            maximum_response_bytes,
            None,
            None,
        )
    }

    fn query_with_metrics(
        &self,
        tenant: TenantId,
        pk: &PrimaryKey,
        exclusive_after_sk: Option<&SortKey>,
        limit: usize,
        maximum_response_bytes: ExecutionBudget,
        metrics: Option<Arc<ResearchMetrics>>,
        kind: Option<ResearchReadKind>,
    ) -> Result<Vec<dodb_service::Document>, Error> {
        let query_started = Instant::now();
        let mut iterator_setup = Duration::ZERO;
        let mut iterator_calls = Duration::ZERO;
        let result = (|| {
            let mut budget = ReadResponseBudget::new(maximum_response_bytes.max_response_bytes)?;
            if limit == 0 {
                return Ok(Vec::new());
            }
            validate_encoded_key(&DocumentKey::new(pk.as_bytes().to_vec(), Vec::new()))?;
            if let Some(cursor) = exclusive_after_sk {
                validate_encoded_key(&DocumentKey::new(
                    pk.as_bytes().to_vec(),
                    cursor.as_bytes().to_vec(),
                ))?;
            }
            let start_key = exclusive_after_sk
                .map(|sort_key| {
                    user_key(&DocumentKey::new(
                        pk.as_bytes().to_vec(),
                        sort_key.as_bytes().to_vec(),
                    ))
                })
                .unwrap_or_else(|| user_partition_prefix(pk));
            let setup_started = Instant::now();
            let iterator = self.read_iterator(&start_key)?;
            iterator_setup = setup_started.elapsed();
            let mut documents = Vec::new();
            loop {
                let current_started = Instant::now();
                let current = iterator.current();
                iterator_calls += current_started.elapsed();
                let Some((encoded_key, value_pointer, value_length)) = current? else {
                    break;
                };
                if encoded_key.first() != Some(&USER_KEY_PREFIX) {
                    break;
                }
                let document_key = DocumentKey::decode(&encoded_key[1..]).map_err(|error| {
                    Error::corruption(format!("RocksDB document key decode failed: {error}"))
                })?;
                if document_key.pk != *pk {
                    break;
                }
                if exclusive_after_sk.is_some_and(|cursor| document_key.sk <= *cursor) {
                    let next_started = Instant::now();
                    iterator.next();
                    iterator_calls += next_started.elapsed();
                    continue;
                }
                if let Some((revision, value_length)) =
                    decode_present_record(value_pointer, value_length)?
                {
                    budget.reserve_document(&document_key, value_length)?;
                    documents.push(dodb_service::Document {
                        key: document_key,
                        value: unsafe {
                            std::slice::from_raw_parts(value_pointer.add(10), value_length).to_vec()
                        },
                        revision,
                    });
                    if documents.len() == limit {
                        return Ok(documents);
                    }
                }
                let next_started = Instant::now();
                iterator.next();
                iterator_calls += next_started.elapsed();
            }
            Ok(documents)
        })();
        if let (Some(metrics), Some(kind)) = (&metrics, kind) {
            let total = query_started.elapsed();
            metrics.record_read(tenant, kind, ReadMetric::QueryIteratorSetup, iterator_setup);
            metrics.record_read(tenant, kind, ReadMetric::QueryIteratorCalls, iterator_calls);
            metrics.record_read(
                tenant,
                kind,
                ReadMetric::QueryCopyDecodeBudget,
                total.saturating_sub(iterator_setup + iterator_calls),
            );
        }
        result
    }

    fn scan(
        &self,
        exclusive_after_key: Option<&DocumentKey>,
        limit: usize,
        maximum_response_bytes: ExecutionBudget,
    ) -> Result<Vec<dodb_service::Document>, Error> {
        let mut budget = ReadResponseBudget::new(maximum_response_bytes.max_response_bytes)?;
        if limit == 0 {
            return Ok(Vec::new());
        }
        if let Some(cursor) = exclusive_after_key {
            validate_encoded_key(cursor)?;
        }
        let start_key = exclusive_after_key
            .map(user_key)
            .unwrap_or_else(|| vec![USER_KEY_PREFIX]);
        let iterator = self.read_iterator(&start_key)?;
        let mut documents = Vec::new();
        while let Some((encoded_key, value_pointer, value_length)) = iterator.current()? {
            if encoded_key.first() != Some(&USER_KEY_PREFIX) {
                break;
            }
            let document_key = DocumentKey::decode(&encoded_key[1..]).map_err(|error| {
                Error::corruption(format!("RocksDB document key decode failed: {error}"))
            })?;
            if exclusive_after_key.is_some_and(|cursor| document_key <= *cursor) {
                iterator.next();
                continue;
            }
            if let Some((revision, value_length)) =
                decode_present_record(value_pointer, value_length)?
            {
                budget.reserve_document(&document_key, value_length)?;
                documents.push(dodb_service::Document {
                    key: document_key,
                    value: unsafe {
                        std::slice::from_raw_parts(value_pointer.add(10), value_length).to_vec()
                    },
                    revision,
                });
                if documents.len() == limit {
                    return Ok(documents);
                }
            }
            iterator.next();
        }
        Ok(documents)
    }

    #[cfg(test)]
    fn next_reserved_revision(
        &self,
        metrics: Option<Arc<TenantResearchMetrics>>,
        stage: usize,
    ) -> Result<u64, Error> {
        self.next_reserved_revision_with_hook(metrics, stage, || Ok(()))
    }

    fn next_reserved_revision_with_hook<F>(
        &self,
        metrics: Option<Arc<TenantResearchMetrics>>,
        stage: usize,
        after_reservation_commit: F,
    ) -> Result<u64, Error>
    where
        F: FnOnce() -> Result<(), Error>,
    {
        let wait_started = Instant::now();
        let mut range = self
            .reserved_revisions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(metrics) = &metrics {
            metrics.record(Metric::AllocatorWait, wait_started.elapsed(), stage);
        }
        if range.next == 0 || range.next > range.end {
            let reserve_started = Instant::now();
            let mut error = std::ptr::null_mut();
            let transaction = unsafe { dodb_rocksdb_begin(self.handle.0.as_ptr(), &mut error) };
            let transaction = NonNull::new(transaction)
                .ok_or_else(|| ffi_error(error, "revision range transaction begin failed"))?;
            let mut transaction_guard = TransactionGuard {
                handle: transaction,
                _database: Arc::clone(&self.handle),
                active: true,
            };
            let mut lock_wait_nanos = 0;
            let mut lock_wait_count = 0;
            let counter = transaction_get_for_update(
                &transaction_guard,
                METADATA_COUNTER_KEY,
                metrics.is_some() && stage > 0,
                &mut lock_wait_nanos,
                &mut lock_wait_count,
            )?;
            if let Some(metrics) = &metrics {
                metrics.record(
                    Metric::RevisionCounterGetForUpdateDuration,
                    reserve_started.elapsed(),
                    stage,
                );
                metrics.record(
                    Metric::RevisionCounterLockWait,
                    std::time::Duration::from_nanos(lock_wait_nanos),
                    stage,
                );
            }
            let current_high_water = counter
                .as_deref()
                .map(counter_bytes_to_u64)
                .transpose()?
                .unwrap_or(0);
            let first_revision = current_high_water
                .checked_add(1)
                .filter(|revision| *revision != 0)
                .ok_or_else(|| Error::invariant("tenant revision counter exhausted"))?;
            let reserved_high_water = current_high_water
                .checked_add(self.revision_range_size)
                .filter(|revision| *revision >= first_revision)
                .ok_or_else(|| Error::invariant("tenant revision range exhausted"))?;
            transaction_put(
                &transaction_guard,
                METADATA_COUNTER_KEY,
                &reserved_high_water.to_be_bytes(),
            )?;
            let commit_started = Instant::now();
            let mut commit_error = std::ptr::null_mut();
            let commit_result = unsafe {
                dodb_rocksdb_transaction_commit(
                    transaction_guard.handle.as_ptr(),
                    &mut commit_error,
                )
            };
            if commit_result != 0 {
                return Err(Error::durability(unsafe {
                    take_error(commit_error, "revision range reservation commit failed")
                }));
            }
            transaction_guard.finish();
            after_reservation_commit()?;
            range.next = first_revision;
            range.end = reserved_high_water;
            if let Some(metrics) = &metrics {
                metrics.record(Metric::CommitDuration, commit_started.elapsed(), stage);
                metrics.record(
                    Metric::RangeReservationDuration,
                    reserve_started.elapsed(),
                    stage,
                );
                let outcomes = if stage == 2 {
                    &metrics.measurement_outcomes
                } else {
                    &metrics.warmup_outcomes
                };
                if stage > 0 {
                    outcomes.range_reservations.fetch_add(1, Ordering::Relaxed);
                    outcomes.reservation_commits.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        let revision = range.next;
        range.next = range
            .next
            .checked_add(1)
            .ok_or_else(|| Error::invariant("tenant revision counter exhausted"))?;
        Ok(revision)
    }

    #[cfg(test)]
    fn transact(&self, request: TransactionRequest) -> Result<LogicalTransactionResult, Error> {
        self.transact_with_metrics(request, None, 0)
    }

    fn transact_with_metrics(
        &self,
        request: TransactionRequest,
        metrics: Option<Arc<TenantResearchMetrics>>,
        stage: usize,
    ) -> Result<LogicalTransactionResult, Error> {
        self.transact_with_hooks(request, || {}, || Ok(()), || Ok(()), metrics, stage)
    }

    #[cfg(test)]
    fn transact_with_begin_hook<F>(
        &self,
        request: TransactionRequest,
        on_begin: F,
    ) -> Result<LogicalTransactionResult, Error>
    where
        F: FnOnce(),
    {
        self.transact_with_hooks(request, on_begin, || Ok(()), || Ok(()), None, 0)
    }

    fn transact_with_hooks<B, R, T>(
        &self,
        request: TransactionRequest,
        on_begin: B,
        after_reservation_commit: R,
        after_revision_assignment: T,
        metrics: Option<Arc<TenantResearchMetrics>>,
        stage: usize,
    ) -> Result<LogicalTransactionResult, Error>
    where
        B: FnOnce(),
        R: FnOnce() -> Result<(), Error>,
        T: FnOnce() -> Result<(), Error>,
    {
        request.validate()?;
        let mut error = std::ptr::null_mut();
        let transaction = unsafe { dodb_rocksdb_begin(self.handle.0.as_ptr(), &mut error) };
        let transaction = NonNull::new(transaction)
            .ok_or_else(|| ffi_error(error, "RocksDB transaction begin failed"))?;
        let mut transaction_guard = TransactionGuard {
            handle: transaction,
            _database: Arc::clone(&self.handle),
            active: true,
        };
        on_begin();
        let mut encoded_keys = BTreeSet::new();
        for condition in &request.conditions {
            encoded_keys.insert(user_key(condition.key()));
        }
        for mutation in &request.mutations {
            encoded_keys.insert(user_key(mutation.key()));
        }
        let mut states = BTreeMap::new();
        for encoded_key in encoded_keys {
            let get_started = Instant::now();
            let is_user_key = encoded_key.first() == Some(&USER_KEY_PREFIX);
            let mut lock_wait_nanos = 0;
            let mut lock_wait_count = 0;
            let state = match transaction_get_for_update(
                &transaction_guard,
                &encoded_key,
                metrics.is_some() && stage > 0,
                &mut lock_wait_nanos,
                &mut lock_wait_count,
            )? {
                Some(encoded_state) => decode_state(&encoded_state)?,
                None => RevisionState::missing(Revision::ZERO),
            };
            if let Some(metrics) = &metrics {
                metrics.record(
                    if is_user_key {
                        Metric::UserGetForUpdateDuration
                    } else {
                        Metric::RevisionCounterGetForUpdateDuration
                    },
                    get_started.elapsed(),
                    stage,
                );
                let lock_metric = if is_user_key {
                    Metric::UserKeyLockWait
                } else {
                    Metric::RevisionCounterLockWait
                };
                metrics.record(
                    lock_metric,
                    std::time::Duration::from_nanos(lock_wait_nanos),
                    stage,
                );
                if stage > 0 && is_user_key {
                    let outcomes = if stage == 2 {
                        &metrics.measurement_outcomes
                    } else {
                        &metrics.warmup_outcomes
                    };
                    outcomes
                        .user_key_lock_wait_count
                        .fetch_add(lock_wait_count, Ordering::Relaxed);
                } else if stage > 0 {
                    let outcomes = if stage == 2 {
                        &metrics.measurement_outcomes
                    } else {
                        &metrics.warmup_outcomes
                    };
                    outcomes
                        .revision_counter_lock_wait_count
                        .fetch_add(lock_wait_count, Ordering::Relaxed);
                }
            }
            states.insert(encoded_key, state);
        }
        let condition_started = Instant::now();
        for condition in &request.conditions {
            let actual = states
                .get(&user_key(condition.key()))
                .ok_or_else(|| Error::invariant("transaction condition state was not locked"))?;
            if !condition_matches(condition, actual) {
                if let Some(metrics) = &metrics {
                    metrics.record(Metric::ConditionCheck, condition_started.elapsed(), stage);
                    let rollback_started = Instant::now();
                    transaction_guard.rollback();
                    metrics.record(Metric::RollbackDuration, rollback_started.elapsed(), stage);
                }
                return Err(Error::conflict(TransactionConflict {
                    key: condition.key().clone(),
                    expected: condition.expectation(),
                    actual: actual.observed(),
                }));
            }
        }
        if let Some(metrics) = &metrics {
            metrics.record(Metric::ConditionCheck, condition_started.elapsed(), stage);
        }
        if request.mutations.is_empty() {
            let rollback_started = Instant::now();
            transaction_guard.rollback();
            if let Some(metrics) = &metrics {
                metrics.record(Metric::RollbackDuration, rollback_started.elapsed(), stage);
            }
            return Ok(LogicalTransactionResult {
                commit_lsn: None,
                revision: None,
            });
        }
        let counter_held_started = if self.revision_allocator == RevisionAllocatorMode::Serialized {
            let counter_started = Instant::now();
            let mut counter_lock_wait_nanos = 0;
            let mut counter_lock_wait_count = 0;
            let counter = transaction_get_for_update(
                &transaction_guard,
                METADATA_COUNTER_KEY,
                metrics.is_some() && stage > 0,
                &mut counter_lock_wait_nanos,
                &mut counter_lock_wait_count,
            )?;
            if let Some(metrics) = &metrics {
                if stage > 0 {
                    let outcomes = if stage == 2 {
                        &metrics.measurement_outcomes
                    } else {
                        &metrics.warmup_outcomes
                    };
                    outcomes
                        .user_transaction_counter_accesses
                        .fetch_add(1, Ordering::Relaxed);
                }
            }
            if let Some(metrics) = &metrics {
                metrics.record(
                    Metric::RevisionCounterGetForUpdateDuration,
                    counter_started.elapsed(),
                    stage,
                );
                metrics.record(
                    Metric::RevisionCounterLockWait,
                    std::time::Duration::from_nanos(counter_lock_wait_nanos),
                    stage,
                );
                if stage > 0 {
                    let outcomes = if stage == 2 {
                        &metrics.measurement_outcomes
                    } else {
                        &metrics.warmup_outcomes
                    };
                    outcomes
                        .revision_counter_lock_wait_count
                        .fetch_add(counter_lock_wait_count, Ordering::Relaxed);
                }
            }
            let current_revision = counter
                .as_deref()
                .map(counter_bytes_to_u64)
                .transpose()?
                .unwrap_or(0);
            Some((current_revision, Instant::now()))
        } else {
            None
        };
        let next_revision = match &counter_held_started {
            Some((current_revision, _)) => current_revision
                .checked_add(1)
                .ok_or_else(|| Error::invariant("tenant revision counter exhausted"))?,
            None => self.next_reserved_revision_with_hook(
                metrics.clone(),
                stage,
                after_reservation_commit,
            )?,
        };
        after_revision_assignment()?;
        let mutation_started = Instant::now();
        for mutation in &request.mutations {
            let (key, state) = match mutation {
                TransactionMutation::Put { key, value } => (
                    user_key(key),
                    RevisionState::present(value.clone(), Revision::new(next_revision)),
                ),
                TransactionMutation::Delete { key } => (
                    user_key(key),
                    RevisionState::missing(Revision::new(next_revision)),
                ),
            };
            let encoded_state = encode_state(&state);
            transaction_put(&transaction_guard, &key, &encoded_state)?;
        }
        if self.revision_allocator == RevisionAllocatorMode::Serialized {
            transaction_put(
                &transaction_guard,
                METADATA_COUNTER_KEY,
                &next_revision.to_be_bytes(),
            )?;
        }
        if let Some(metrics) = &metrics {
            metrics.record(
                Metric::MutationConstruction,
                mutation_started.elapsed(),
                stage,
            );
        }
        let commit_started = Instant::now();
        let mut commit_error = std::ptr::null_mut();
        let commit_result = unsafe {
            dodb_rocksdb_transaction_commit(transaction_guard.handle.as_ptr(), &mut commit_error)
        };
        if commit_result != 0 {
            return Err(Error::durability(unsafe {
                take_error(commit_error, "RocksDB transaction commit failed")
            }));
        }
        if let Some(metrics) = &metrics {
            metrics.record(Metric::CommitDuration, commit_started.elapsed(), stage);
            metrics.record(Metric::UserCommitDuration, commit_started.elapsed(), stage);
            if let Some((_, counter_held_started)) = counter_held_started {
                metrics.record(
                    Metric::CounterHeldDuration,
                    counter_held_started.elapsed(),
                    stage,
                );
            }
            if stage > 0 {
                let outcomes = if stage == 2 {
                    &metrics.measurement_outcomes
                } else {
                    &metrics.warmup_outcomes
                };
                outcomes.user_commits.fetch_add(1, Ordering::Relaxed);
            }
        }
        transaction_guard.finish();
        Ok(LogicalTransactionResult {
            commit_lsn: Some(dodb_core::Lsn::new(next_revision)),
            revision: Some(Revision::new(next_revision)),
        })
    }
}

impl Drop for RocksDbHandle {
    fn drop(&mut self) {
        unsafe { dodb_rocksdb_close(self.0.as_ptr()) };
    }
}

impl TransactionGuard {
    fn rollback(&mut self) {
        if self.active {
            unsafe { dodb_rocksdb_transaction_rollback(self.handle.as_ptr()) };
            self.active = false;
        }
    }

    fn finish(&mut self) {
        if self.active {
            unsafe { dodb_rocksdb_transaction_destroy(self.handle.as_ptr()) };
            self.active = false;
        }
    }
}

impl TenantReadIterator {
    fn current(&self) -> Result<Option<(Vec<u8>, *const u8, usize)>, Error> {
        if unsafe { dodb_rocksdb_iterator_valid(self.handle.as_ptr()) } == 0 {
            let mut error = std::ptr::null_mut();
            let result = unsafe { dodb_rocksdb_iterator_status(self.handle.as_ptr(), &mut error) };
            if result != 0 {
                return Err(ffi_error(error, "RocksDB iterator failed"));
            }
            return Ok(None);
        }
        let mut key_length = 0;
        let key = unsafe { dodb_rocksdb_iterator_key(self.handle.as_ptr(), &mut key_length) };
        let mut value_length = 0;
        let value = unsafe { dodb_rocksdb_iterator_value(self.handle.as_ptr(), &mut value_length) };
        let key = unsafe { std::slice::from_raw_parts(key, key_length) }.to_vec();
        Ok(Some((key, value, value_length)))
    }

    fn next(&self) {
        unsafe { dodb_rocksdb_iterator_next(self.handle.as_ptr()) };
    }
}

impl Drop for TenantReadIterator {
    fn drop(&mut self) {
        unsafe { dodb_rocksdb_iterator_destroy(self.handle.as_ptr()) };
    }
}

impl Drop for TransactionGuard {
    fn drop(&mut self) {
        self.rollback();
    }
}

#[derive(Clone, Copy)]
struct LogicalTransactionResult {
    commit_lsn: Option<dodb_core::Lsn>,
    revision: Option<Revision>,
}

impl DodbService for RocksDbService {
    fn execute<'service>(
        &'service self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> ServiceFuture<'service> {
        Box::pin(async move { self.execute_request(tenant, request, budget).await })
    }

    fn shutdown<'service>(&'service self) -> ShutdownFuture<'service> {
        Box::pin(async move { self.shutdown().await })
    }
}

fn user_key(key: &DocumentKey) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(key.encoded_len() + 1);
    encoded.push(USER_KEY_PREFIX);
    encoded.extend_from_slice(&key.encode());
    encoded
}

fn user_partition_prefix(pk: &PrimaryKey) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(pk.as_bytes().len() + 3);
    encoded.push(USER_KEY_PREFIX);
    let partition = DocumentKey::new(pk.as_bytes().to_vec(), Vec::new()).encode();
    encoded.extend_from_slice(&partition[..partition.len() - 2]);
    encoded
}

fn validate_encoded_key(key: &DocumentKey) -> Result<(), Error> {
    let encoded_length = key.encoded_len();
    if encoded_length > MAX_STORAGE_ENCODED_KEY_SIZE {
        return Err(Error::invalid_input(format!(
            "encoded document key is {encoded_length} bytes, maximum is {MAX_STORAGE_ENCODED_KEY_SIZE}"
        )));
    }
    Ok(())
}

fn decode_present_record(
    encoded_pointer: *const u8,
    encoded_length: usize,
) -> Result<Option<(Revision, usize)>, Error> {
    if encoded_length < 10 {
        return Err(Error::corruption(
            "unsupported RocksDB logical value format",
        ));
    }
    let encoded = unsafe { std::slice::from_raw_parts(encoded_pointer, encoded_length) };
    if encoded[0] != VALUE_FORMAT {
        return Err(Error::corruption(
            "unsupported RocksDB logical value format",
        ));
    }
    let revision = Revision::new(u64::from_be_bytes(
        encoded[2..10]
            .try_into()
            .map_err(|_| Error::corruption("invalid RocksDB revision bytes"))?,
    ));
    match encoded[1] {
        PRESENT_TAG => Ok(Some((revision, encoded_length - 10))),
        MISSING_TAG if encoded.len() == 10 => Ok(None),
        MISSING_TAG => Err(Error::corruption("tombstone contains value bytes")),
        _ => Err(Error::corruption("unknown RocksDB logical value state")),
    }
}

struct ReadResponseBudget {
    maximum_wire: usize,
    maximum_memory: usize,
    used_wire: usize,
    used_memory: usize,
}

impl ReadResponseBudget {
    fn new(maximum: usize) -> Result<Self, Error> {
        if maximum < 17 {
            return Err(Error::response_too_large(format!(
                "response envelope exceeds {maximum} bytes"
            )));
        }
        Ok(Self {
            maximum_wire: maximum,
            maximum_memory: maximum.saturating_add(64 * 1024),
            used_wire: 17,
            used_memory: 0,
        })
    }

    fn reserve_document(&mut self, key: &DocumentKey, value_length: usize) -> Result<(), Error> {
        let materialized = key
            .pk
            .as_bytes()
            .len()
            .saturating_add(key.sk.as_bytes().len())
            .saturating_add(value_length)
            .saturating_add(64);
        let wire = key
            .pk
            .as_bytes()
            .len()
            .saturating_add(key.sk.as_bytes().len())
            .saturating_add(value_length)
            .saturating_add(20);
        let next_wire = self
            .used_wire
            .checked_add(wire)
            .ok_or_else(|| Error::response_too_large("encoded response size overflows"))?;
        let next_memory = self
            .used_memory
            .checked_add(materialized.max(wire))
            .ok_or_else(|| Error::response_too_large("response materialization size overflows"))?;
        if next_wire > self.maximum_wire || next_memory > self.maximum_memory {
            return Err(Error::response_too_large(format!(
                "response materialization exceeds {} bytes",
                self.maximum_wire
            )));
        }
        self.used_wire = next_wire;
        self.used_memory = next_memory;
        Ok(())
    }
}

fn encode_state(state: &RevisionState) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.push(VALUE_FORMAT);
    match state {
        RevisionState::Present { value, revision } => {
            encoded.push(PRESENT_TAG);
            encoded.extend_from_slice(&revision.get().to_be_bytes());
            encoded.extend_from_slice(value);
        }
        RevisionState::Missing { revision } => {
            encoded.push(MISSING_TAG);
            encoded.extend_from_slice(&revision.get().to_be_bytes());
        }
    }
    encoded
}

fn decode_state(encoded: &[u8]) -> Result<RevisionState, Error> {
    if encoded.len() < 10 || encoded[0] != VALUE_FORMAT {
        return Err(Error::corruption(
            "unsupported RocksDB logical value format",
        ));
    }
    let revision = Revision::new(u64::from_be_bytes(
        encoded[2..10]
            .try_into()
            .map_err(|_| Error::corruption("invalid RocksDB revision bytes"))?,
    ));
    match encoded[1] {
        PRESENT_TAG => Ok(RevisionState::present(encoded[10..].to_vec(), revision)),
        MISSING_TAG if encoded.len() == 10 => Ok(RevisionState::missing(revision)),
        MISSING_TAG => Err(Error::corruption("tombstone contains value bytes")),
        _ => Err(Error::corruption("unknown RocksDB logical value state")),
    }
}

fn counter_bytes_to_u64(encoded: &[u8]) -> Result<u64, Error> {
    Ok(u64::from_be_bytes(encoded.try_into().map_err(|_| {
        Error::corruption("RocksDB revision counter has invalid length")
    })?))
}

fn condition_matches(condition: &TransactionCondition, actual: &RevisionState) -> bool {
    match condition {
        TransactionCondition::RevisionEquals {
            expected_revision, ..
        } => actual.revision() == *expected_revision,
        TransactionCondition::Exists { .. } => !actual.is_missing(),
        TransactionCondition::NotExists { .. } => actual.is_missing(),
    }
}

fn evaluate_uncreated(request: &TransactionRequest) -> Result<LogicalTransactionResult, Error> {
    let actual = RevisionState::missing(Revision::ZERO);
    for condition in &request.conditions {
        if !condition_matches(condition, &actual) {
            return Err(Error::conflict(TransactionConflict {
                key: condition.key().clone(),
                expected: condition.expectation(),
                actual: actual.observed(),
            }));
        }
    }
    Ok(LogicalTransactionResult {
        commit_lsn: None,
        revision: None,
    })
}

fn transaction_put(transaction: &TransactionGuard, key: &[u8], value: &[u8]) -> Result<(), Error> {
    let mut error = std::ptr::null_mut();
    let result = unsafe {
        dodb_rocksdb_transaction_put(
            transaction.handle.as_ptr(),
            key.as_ptr(),
            key.len(),
            value.as_ptr(),
            value.len(),
            &mut error,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(ffi_error(error, "RocksDB transaction write failed"))
    }
}

fn transaction_get_for_update(
    transaction: &TransactionGuard,
    key: &[u8],
    collect_lock_wait: bool,
    lock_wait_nanos: &mut u64,
    lock_wait_count: &mut u64,
) -> Result<Option<Vec<u8>>, Error> {
    let mut value = std::ptr::null_mut();
    let mut value_length = 0;
    let mut error = std::ptr::null_mut();
    let result = unsafe {
        dodb_rocksdb_transaction_get_for_update(
            transaction.handle.as_ptr(),
            key.as_ptr(),
            key.len(),
            &mut value,
            &mut value_length,
            i32::from(collect_lock_wait),
            lock_wait_nanos,
            lock_wait_count,
            &mut error,
        )
    };
    match result {
        1 => Ok(None),
        2 => Err(ffi_error(error, "RocksDB transaction read failed")),
        _ => Ok(Some(unsafe { copy_and_free_value(value, value_length) })),
    }
}

fn enforce_response_budget(response: &Response, budget: ExecutionBudget) -> Result<(), Error> {
    let limits = ProtocolLimits {
        max_response_frame_size: budget.max_response_bytes,
        ..ProtocolLimits::default()
    };
    encode_response(&ResponseEnvelope::Success(response.clone()), limits)
        .map(|_| ())
        .map_err(|error| Error::response_too_large(format!("response exceeds budget: {error}")))
}

unsafe fn copy_and_free_value(value: *mut u8, value_length: usize) -> Vec<u8> {
    let bytes = unsafe { std::slice::from_raw_parts(value, value_length) }.to_vec();
    unsafe { dodb_rocksdb_value_free(value) };
    bytes
}

fn ffi_error(error: *mut c_char, fallback: &str) -> Error {
    let message = unsafe { take_error(error, fallback) };
    Error::Io(std::io::Error::other(message))
}

unsafe fn take_error(error: *mut c_char, fallback: &str) -> String {
    let Some(error) = NonNull::new(error) else {
        return fallback.to_owned();
    };
    let message = unsafe { CStr::from_ptr(error.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    unsafe { dodb_rocksdb_error_free(error.as_ptr()) };
    message
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;
    use tempfile::TempDir;

    fn temporary_db_directory() -> TempDir {
        match std::env::var_os("DODB_TEST_DATA_DIR") {
            Some(path) => TempDir::new_in(path).unwrap(),
            None => TempDir::new().unwrap(),
        }
    }

    #[test]
    fn get_completes_while_same_tenant_transaction_is_open() {
        let directory = temporary_db_directory();
        let database = Arc::new(TenantDatabase::open(directory.path()).unwrap());
        let key = DocumentKey::new(b"concurrent-read".to_vec(), b"key".to_vec());
        database
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: b"committed".to_vec(),
                }],
            ))
            .unwrap();

        let (transaction_started_sender, transaction_started_receiver) = mpsc::channel();
        let (continue_transaction_sender, continue_transaction_receiver) = mpsc::channel();
        let transaction_database = Arc::clone(&database);
        let transaction_key = key.clone();
        let transaction = std::thread::spawn(move || {
            transaction_database.transact_with_begin_hook(
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: transaction_key,
                        value: b"uncommitted".to_vec(),
                    }],
                ),
                || {
                    transaction_started_sender.send(()).unwrap();
                    continue_transaction_receiver
                        .recv_timeout(Duration::from_secs(10))
                        .expect("transaction continuation timed out");
                },
            )
        });

        transaction_started_receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("transaction did not reach its open-transaction hook");
        let (get_result_sender, get_result_receiver) = mpsc::channel();
        let get_database = Arc::clone(&database);
        let get_key = key.clone();
        let get = std::thread::spawn(move || {
            get_result_sender.send(get_database.get(&get_key)).unwrap();
        });
        let state = get_result_receiver
            .recv_timeout(Duration::from_secs(5))
            .expect("Get waited for the open transaction");
        assert_eq!(
            state.unwrap(),
            RevisionState::present(b"committed".to_vec(), Revision::new(1))
        );

        continue_transaction_sender.send(()).unwrap();
        transaction.join().unwrap().unwrap();
        get.join().unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_waits_for_an_in_flight_request_and_service_reopens() {
        let directory = temporary_db_directory();
        let service =
            Arc::new(RocksDbService::new(RocksDbServiceConfig::new(directory.path())).unwrap());
        let tenant = TenantId::new(41);
        let key = DocumentKey::new(b"lifecycle".to_vec(), b"key".to_vec());
        service
            .execute(
                tenant,
                Request::Put {
                    key: key.clone(),
                    value: b"alive".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();

        let (operation_started_sender, operation_started_receiver) = mpsc::channel();
        let (continue_operation_sender, continue_operation_receiver) = mpsc::channel();
        let continue_operation_receiver = std::sync::Mutex::new(continue_operation_receiver);
        let should_pause = Arc::new(AtomicBool::new(true));
        *service.blocking_operation_hook.lock().unwrap() = Some(Arc::new(move || {
            if should_pause.swap(false, Ordering::SeqCst) {
                operation_started_sender.send(()).unwrap();
                continue_operation_receiver
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(10))
                    .expect("request continuation timed out");
            }
        }));

        let request_service = Arc::clone(&service);
        let request = tokio::spawn(async move {
            request_service
                .execute(tenant, Request::Get { key }, ExecutionBudget::new(1024))
                .await
        });
        tokio::task::spawn_blocking(move || {
            operation_started_receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("request did not reach its blocking operation");
        })
        .await
        .unwrap();

        let shutdown_service = Arc::clone(&service);
        let (shutdown_started_sender, shutdown_started_receiver) = tokio::sync::oneshot::channel();
        let shutdown = tokio::spawn(async move {
            let _ = shutdown_started_sender.send(());
            shutdown_service.shutdown().await;
        });
        shutdown_started_receiver.await.unwrap();
        continue_operation_sender.send(()).unwrap();

        assert_eq!(
            request.await.unwrap().unwrap(),
            Response::Get(RevisionState::present(b"alive".to_vec(), Revision::new(1)))
        );
        shutdown.await.unwrap();
        *service.blocking_operation_hook.lock().unwrap() = None;

        assert_eq!(
            service
                .execute(
                    tenant,
                    Request::Get {
                        key: DocumentKey::new(b"lifecycle".to_vec(), b"key".to_vec())
                    },
                    ExecutionBudget::new(1024),
                )
                .await
                .unwrap(),
            Response::Get(RevisionState::present(b"alive".to_vec(), Revision::new(1)))
        );
        service.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_drains_a_blocking_operation_after_its_request_is_cancelled() {
        let directory = temporary_db_directory();
        let service =
            Arc::new(RocksDbService::new(RocksDbServiceConfig::new(directory.path())).unwrap());
        let tenant = TenantId::new(42);
        let key = DocumentKey::new(b"cancelled-request".to_vec(), b"key".to_vec());
        service
            .execute(
                tenant,
                Request::Put {
                    key: key.clone(),
                    value: b"alive".to_vec(),
                },
                ExecutionBudget::new(1024),
            )
            .await
            .unwrap();

        let (operation_started_sender, operation_started_receiver) = mpsc::channel();
        let (continue_operation_sender, continue_operation_receiver) = mpsc::channel();
        let continue_operation_receiver = std::sync::Mutex::new(continue_operation_receiver);
        let operation_index = Arc::new(AtomicUsize::new(0));
        *service.blocking_operation_hook.lock().unwrap() = Some(Arc::new(move || {
            if operation_index.fetch_add(1, Ordering::SeqCst) == 1 {
                operation_started_sender.send(()).unwrap();
                continue_operation_receiver
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(10))
                    .expect("request continuation timed out");
            }
        }));

        let request_service = Arc::clone(&service);
        let request_key = key.clone();
        let request = tokio::spawn(async move {
            request_service
                .execute(
                    tenant,
                    Request::Get { key: request_key },
                    ExecutionBudget::new(1024),
                )
                .await
        });
        tokio::task::spawn_blocking(move || {
            operation_started_receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("RocksDB Get did not reach its blocking operation");
        })
        .await
        .unwrap();
        request.abort();
        assert!(request.await.unwrap_err().is_cancelled());

        let (shutdown_started_sender, shutdown_started_receiver) = mpsc::channel();
        *service.lifecycle.shutdown_hook.lock().unwrap() = Some(Arc::new(move || {
            shutdown_started_sender.send(()).unwrap();
        }));
        let shutdown_service = Arc::clone(&service);
        let shutdown = tokio::spawn(async move { shutdown_service.shutdown().await });
        tokio::task::spawn_blocking(move || {
            shutdown_started_receiver
                .recv_timeout(Duration::from_secs(5))
                .expect("shutdown did not close request admission");
        })
        .await
        .unwrap();
        continue_operation_sender.send(()).unwrap();
        shutdown.await.unwrap();
        *service.blocking_operation_hook.lock().unwrap() = None;
        *service.lifecycle.shutdown_hook.lock().unwrap() = None;

        assert_eq!(
            service
                .execute(tenant, Request::Get { key }, ExecutionBudget::new(1024),)
                .await
                .unwrap(),
            Response::Get(RevisionState::present(b"alive".to_vec(), Revision::new(1)))
        );
        service.shutdown().await;
    }

    #[test]
    fn read_iterator_keeps_database_alive_after_tenant_owner_drops() {
        let directory = temporary_db_directory();
        let database = TenantDatabase::open(directory.path()).unwrap();
        let key = DocumentKey::new(b"iterator-lifetime".to_vec(), b"key".to_vec());
        database
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: b"retained".to_vec(),
                }],
            ))
            .unwrap();
        let iterator = database
            .read_iterator(&user_partition_prefix(&PrimaryKey::new(
                b"iterator-lifetime".to_vec(),
            )))
            .unwrap();
        drop(database);
        assert_eq!(iterator_value(&iterator), (key, b"retained".to_vec()));
        drop(iterator);
    }

    #[test]
    fn read_iterator_keeps_one_snapshot_across_atomic_transaction_commit() {
        let directory = temporary_db_directory();
        let database = TenantDatabase::open(directory.path()).unwrap();
        let first_key = DocumentKey::new(b"snapshot".to_vec(), b"a".to_vec());
        let second_key = DocumentKey::new(b"snapshot".to_vec(), b"b".to_vec());
        database
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![
                    TransactionMutation::Put {
                        key: first_key.clone(),
                        value: b"old-a".to_vec(),
                    },
                    TransactionMutation::Put {
                        key: second_key.clone(),
                        value: b"old-b".to_vec(),
                    },
                ],
            ))
            .unwrap();

        let iterator = database
            .read_iterator(&user_partition_prefix(&PrimaryKey::new(
                b"snapshot".to_vec(),
            )))
            .unwrap();
        assert_eq!(
            iterator_value(&iterator),
            (first_key.clone(), b"old-a".to_vec())
        );
        database
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![
                    TransactionMutation::Put {
                        key: first_key,
                        value: b"new-a".to_vec(),
                    },
                    TransactionMutation::Put {
                        key: second_key.clone(),
                        value: b"new-b".to_vec(),
                    },
                ],
            ))
            .unwrap();
        iterator.next();
        assert_eq!(
            iterator_value(&iterator),
            (second_key.clone(), b"old-b".to_vec())
        );
        drop(iterator);

        let current = database
            .query(
                &PrimaryKey::new(b"snapshot".to_vec()),
                None,
                10,
                ExecutionBudget::new(1024),
            )
            .unwrap();
        assert_eq!(current[0].value, b"new-a");
        assert_eq!(current[1].value, b"new-b");
    }

    #[test]
    fn reserved_revision_ranges_are_unique_under_concurrent_assignment() {
        let directory = temporary_db_directory();
        let database = Arc::new(
            TenantDatabase::open_with_allocator(
                directory.path(),
                RevisionAllocatorMode::ReservedRanges,
                3,
            )
            .unwrap(),
        );
        let barrier = Arc::new(std::sync::Barrier::new(16));
        let workers = (0..16)
            .map(|_| {
                let database = Arc::clone(&database);
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    database.next_reserved_revision(None, 0).unwrap()
                })
            })
            .collect::<Vec<_>>();
        let mut revisions = workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .collect::<Vec<_>>();
        revisions.sort_unstable();
        assert_eq!(revisions, (1..=16).collect::<Vec<_>>());
    }

    #[test]
    fn partial_range_is_abandoned_after_reopen_and_failed_tokens_are_burned() {
        let directory = temporary_db_directory();
        let first_database = TenantDatabase::open_with_allocator(
            directory.path(),
            RevisionAllocatorMode::ReservedRanges,
            4,
        )
        .unwrap();
        assert_eq!(first_database.next_reserved_revision(None, 0).unwrap(), 1);
        assert_eq!(first_database.next_reserved_revision(None, 0).unwrap(), 2);
        drop(first_database);

        let reopened = TenantDatabase::open_with_allocator(
            directory.path(),
            RevisionAllocatorMode::ReservedRanges,
            4,
        )
        .unwrap();
        assert_eq!(reopened.next_reserved_revision(None, 0).unwrap(), 5);
        let abandoned = reopened.next_reserved_revision(None, 0).unwrap();
        let next = reopened.next_reserved_revision(None, 0).unwrap();
        assert_eq!(next, abandoned + 1);
    }

    #[test]
    fn assigned_revision_is_burned_when_user_transaction_rolls_back() {
        let directory = temporary_db_directory();
        let database = TenantDatabase::open_with_allocator(
            directory.path(),
            RevisionAllocatorMode::ReservedRanges,
            4,
        )
        .unwrap();
        let key = DocumentKey::new(b"rollback".to_vec(), b"row".to_vec());
        let initial = database
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: b"initial".to_vec(),
                }],
            ))
            .unwrap();
        let initial_revision = initial.revision.unwrap();
        let request = || {
            TransactionRequest::new(
                vec![TransactionCondition::RevisionEquals {
                    key: key.clone(),
                    expected_revision: initial_revision,
                }],
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: b"committed".to_vec(),
                }],
            )
        };
        let failed = database.transact_with_hooks(
            request(),
            || {},
            || Ok(()),
            || {
                Err(Error::durability(
                    "injected rollback after token assignment",
                ))
            },
            None,
            0,
        );
        assert!(failed.is_err());
        assert_eq!(
            database.get(&key).unwrap(),
            RevisionState::present(b"initial".to_vec(), initial_revision)
        );
        let committed = database.transact(request()).unwrap();
        assert_eq!(committed.revision.unwrap().get(), 3);
        assert_eq!(
            database.get(&key).unwrap(),
            RevisionState::present(b"committed".to_vec(), Revision::new(3))
        );
        drop(database);

        let reopened = TenantDatabase::open_with_allocator(
            directory.path(),
            RevisionAllocatorMode::ReservedRanges,
            4,
        )
        .unwrap();
        let after_reopen = reopened
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key,
                    value: b"after reopen".to_vec(),
                }],
            ))
            .unwrap();
        assert_eq!(after_reopen.revision.unwrap().get(), 5);
    }

    #[test]
    fn uncertain_reservation_commit_does_not_publish_its_range() {
        let directory = temporary_db_directory();
        let database = TenantDatabase::open_with_allocator(
            directory.path(),
            RevisionAllocatorMode::ReservedRanges,
            4,
        )
        .unwrap();
        let key = DocumentKey::new(b"reservation".to_vec(), b"row".to_vec());
        let request = || {
            TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: key.clone(),
                    value: b"committed".to_vec(),
                }],
            )
        };
        let uncertain = database.transact_with_hooks(
            request(),
            || {},
            || Err(Error::durability("injected uncertain reservation result")),
            || Ok(()),
            None,
            0,
        );
        assert!(uncertain.is_err());
        assert_eq!(
            database.get(&key).unwrap(),
            RevisionState::missing(Revision::ZERO)
        );
        let committed = database.transact(request()).unwrap();
        assert_eq!(committed.revision.unwrap().get(), 5);
        drop(database);

        let reopened = TenantDatabase::open_with_allocator(
            directory.path(),
            RevisionAllocatorMode::ReservedRanges,
            4,
        )
        .unwrap();
        let after_reopen = reopened
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key,
                    value: b"after reopen".to_vec(),
                }],
            ))
            .unwrap();
        assert_eq!(after_reopen.revision.unwrap().get(), 9);
    }

    #[test]
    fn allocator_modes_switch_above_existing_counter_high_water() {
        let directory = temporary_db_directory();
        let serialized = TenantDatabase::open(directory.path()).unwrap();
        let prior = serialized
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: DocumentKey::new(b"legacy".to_vec(), b"row".to_vec()),
                    value: b"value".to_vec(),
                }],
            ))
            .unwrap()
            .revision
            .unwrap()
            .get();
        drop(serialized);

        let ranged = TenantDatabase::open_with_allocator(
            directory.path(),
            RevisionAllocatorMode::ReservedRanges,
            4,
        )
        .unwrap();
        assert_eq!(ranged.next_reserved_revision(None, 0).unwrap(), prior + 1);
        drop(ranged);

        let serialized_again = TenantDatabase::open(directory.path()).unwrap();
        let next = serialized_again
            .transact(TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: DocumentKey::new(b"legacy".to_vec(), b"next".to_vec()),
                    value: b"value".to_vec(),
                }],
            ))
            .unwrap()
            .revision
            .unwrap()
            .get();
        assert_eq!(next, prior + 5);
    }

    #[test]
    fn range_reservation_rejects_overflow_and_corrupt_counter() {
        for (counter_value, range_size) in [(u64::MAX - 1, 4), (0, 4)] {
            let directory = temporary_db_directory();
            let database = TenantDatabase::open_with_allocator(
                directory.path(),
                RevisionAllocatorMode::ReservedRanges,
                range_size,
            )
            .unwrap();
            let transaction = unsafe {
                dodb_rocksdb_begin(database.handle.0.as_ptr(), &mut std::ptr::null_mut())
            };
            let transaction = NonNull::new(transaction).unwrap();
            let mut guard = TransactionGuard {
                handle: transaction,
                _database: Arc::clone(&database.handle),
                active: true,
            };
            let encoded_counter = if counter_value == 0 {
                vec![1, 2, 3]
            } else {
                counter_value.to_be_bytes().to_vec()
            };
            transaction_put(&guard, METADATA_COUNTER_KEY, &encoded_counter).unwrap();
            let mut commit_error = std::ptr::null_mut();
            assert_eq!(
                unsafe {
                    dodb_rocksdb_transaction_commit(guard.handle.as_ptr(), &mut commit_error)
                },
                0
            );
            guard.finish();
            assert!(database.next_reserved_revision(None, 0).is_err());
        }
    }

    fn iterator_value(iterator: &TenantReadIterator) -> (DocumentKey, Vec<u8>) {
        let (encoded_key, value_pointer, value_length) = iterator.current().unwrap().unwrap();
        let document_key = DocumentKey::decode(&encoded_key[1..]).unwrap();
        let (revision, content_length) = decode_present_record(value_pointer, value_length)
            .unwrap()
            .unwrap();
        let content =
            unsafe { std::slice::from_raw_parts(value_pointer.add(10), content_length).to_vec() };
        assert!(revision.get() > 0);
        (document_key, content)
    }
}
