use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use dodb_core::{
    DocumentKey, Error, Revision, RevisionState, TenantId, TransactionCondition,
    TransactionMutation, TransactionRequest,
};
use dodb_server::{BlinkTenantService, LocalTenantServiceConfig};
use dodb_service::{DodbService, ExecutionBudget, Request, Response};
use dodb_storage::{CoordinatorConfig, DatabaseConfig};
use serde_json::{Value, json};
use tokio::task::JoinSet;

const RESPONSE_BUDGET: usize = 64 * 1024;
const INITIAL_ROWS: usize = 10_000;
const VALUE_SIZE: usize = 512;
const MAX_WRITE_LEDGER: usize = 1_000_000;
const INPUT_FINGERPRINT_PREFIX_WORKFLOWS: u64 = 100;
const HISTOGRAM_EXACT_BUCKETS: usize = 1_024;
const HISTOGRAM_SUB_BUCKETS_PER_EXPONENT: usize = 1_024;
const HISTOGRAM_BUCKETS: usize =
    HISTOGRAM_EXACT_BUCKETS + (u64::BITS as usize - 10) * HISTOGRAM_SUB_BUCKETS_PER_EXPONENT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BackendKind {
    Blink,
    RocksDb,
    RocksDbDirect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClientExecutor {
    Async,
    Dedicated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Profile {
    Rw95,
    Rw80,
    Rw50,
    Rw20,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Scenario {
    Single(Profile),
    Multi,
    Smoke,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReadKind {
    Get,
    Query16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WriteKind {
    Put,
    Delete,
    ConditionalOne,
    TransactionFour,
    TransactionEight,
    TransactionSixteen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MutationWorkload {
    NewKey,
    Overwrite,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LatencyStorageMode {
    ExactAll,
    Log1024,
    Log1024Validation,
}

#[derive(Clone, Debug)]
enum WorkflowKind {
    Read(ReadKind),
    Write {
        kind: WriteKind,
        revision_get: bool,
        intended_rejection: bool,
        condition_kind: usize,
    },
}

#[derive(Clone, Debug)]
struct WorkflowPlan {
    kind: WorkflowKind,
    key_ordinal: u64,
}

enum DriverBackend {
    Service(Box<dyn DodbService>),
    RocksDbService(Box<dodb_rocksdb::RocksDbService>),
    Direct {
        tenants: BTreeMap<TenantId, dodb_rocksdb::research::SynchronousRocksDbTenant>,
        admission: tokio::sync::Semaphore,
    },
}

struct DriverService {
    backend: DriverBackend,
    research_metrics: Option<Arc<dodb_rocksdb::ResearchMetrics>>,
}

impl DriverService {
    fn new(options: &Options, data_dir: PathBuf) -> Result<Self, Error> {
        let (backend, research_metrics) = match options.backend {
            BackendKind::Blink => {
                let mut config = LocalTenantServiceConfig::default();
                config.data_dir = data_dir;
                config.max_open_shards = 16;
                config.database_config = DatabaseConfig::default().with_cache_capacity(128);
                config.coordinator_config = CoordinatorConfig::default();
                (
                    DriverBackend::Service(Box::new(BlinkTenantService::new(config)?)),
                    None,
                )
            }
            BackendKind::RocksDb => {
                let mut config = dodb_rocksdb::RocksDbServiceConfig::new(data_dir);
                config.max_blocking_operations =
                    options.blocking_admission.split_slot_limits().map_or(
                        options.shared_blocking_slots,
                        |(read_slots, write_slots)| read_slots + write_slots,
                    );
                config.blocking_admission = options.blocking_admission;
                config.research_metrics = options.research_metrics;
                config.detailed_write_metrics = options.detailed_write_metrics;
                config.revision_allocator = options.revision_allocator;
                config.condition_key_lock_mode = options.condition_key_lock_mode;
                let service = dodb_rocksdb::RocksDbService::new(config)?;
                let metrics = service.research_metrics_handle();
                (DriverBackend::RocksDbService(Box::new(service)), metrics)
            }
            BackendKind::RocksDbDirect => {
                std::fs::create_dir_all(&data_dir)?;
                let tenant_count = if matches!(options.scenario, Scenario::Multi | Scenario::Smoke)
                {
                    4
                } else {
                    1
                };
                let tenants = (1..=tenant_count)
                    .map(|tenant_number| {
                        let tenant = TenantId::new(tenant_number);
                        let database = dodb_rocksdb::research::SynchronousRocksDbTenant::open_with_condition_key_lock_mode(
                            &data_dir.join(format!("tenant-{tenant_number}")),
                            options.revision_allocator,
                            options.condition_key_lock_mode,
                        )?;
                        Ok((tenant, database))
                    })
                    .collect::<Result<BTreeMap<_, _>, Error>>()?;
                (
                    DriverBackend::Direct {
                        tenants,
                        admission: tokio::sync::Semaphore::new(options.shared_blocking_slots),
                    },
                    None,
                )
            }
        };
        Ok(Self {
            backend,
            research_metrics,
        })
    }

    async fn execute(&self, tenant: TenantId, request: Request) -> Result<Response, Error> {
        let budget = ExecutionBudget::new(RESPONSE_BUDGET);
        match &self.backend {
            DriverBackend::Service(service) => service.execute(tenant, request, budget).await,
            DriverBackend::RocksDbService(service) => {
                service.execute(tenant, request, budget).await
            }
            DriverBackend::Direct { tenants, admission } => {
                let _permit = admission
                    .acquire()
                    .await
                    .map_err(|_| Error::overloaded("direct admission closed"))?;
                tenants
                    .get(&tenant)
                    .ok_or_else(|| Error::invalid_request("tenant was not preopened"))?
                    .execute(request, budget)
            }
        }
    }

    async fn shutdown(&self) {
        match &self.backend {
            DriverBackend::Service(service) => service.shutdown().await,
            DriverBackend::RocksDbService(service) => service.shutdown().await,
            DriverBackend::Direct { .. } => {}
        }
    }

    fn engine_health_observer(&self) -> Option<dodb_rocksdb::EngineHealthObserver> {
        match &self.backend {
            DriverBackend::Service(_) => None,
            DriverBackend::RocksDbService(service) => Some(service.engine_health_observer()),
            DriverBackend::Direct { tenants, .. } => {
                Some(dodb_rocksdb::EngineHealthObserver::combine(
                    tenants.iter().map(|(tenant, database)| {
                        database.engine_health_observer_named(format!("tenant_{tenant}"))
                    }),
                ))
            }
        }
    }

    fn set_research_measurement_phase(&self) {
        if let Some(metrics) = &self.research_metrics {
            metrics.set_measurement_phase();
        }
    }

    fn set_research_warmup_phase(&self) {
        if let Some(metrics) = &self.research_metrics {
            metrics.set_warmup_phase();
        }
    }

    fn research_metrics_snapshot(&self) -> serde_json::Value {
        self.research_metrics.as_ref().map_or_else(
            || json!({"enabled": false, "reason": "disabled"}),
            |metrics| metrics.snapshot(),
        )
    }
}

#[derive(Clone, Debug)]
struct Options {
    backend: BackendKind,
    client_executor: ClientExecutor,
    scenario: Scenario,
    data_dir: PathBuf,
    seed: u64,
    warmup: Duration,
    measurement: Duration,
    clients: usize,
    initial_rows: usize,
    value_size: usize,
    call_cap: u64,
    report_path: Option<PathBuf>,
    research_metrics: bool,
    read_stage_metrics: bool,
    detailed_write_metrics: bool,
    revision_allocator: dodb_rocksdb::RevisionAllocatorMode,
    condition_key_lock_mode: dodb_rocksdb::ConditionKeyLockMode,
    engine_health_interval_ms: u64,
    blocking_admission: dodb_rocksdb::BlockingAdmissionMode,
    shared_blocking_slots: usize,
    mutation_workload: MutationWorkload,
    overwrite_slots_per_client: usize,
    latency_storage: LatencyStorageMode,
    max_write_ledger: usize,
    require_full_window: bool,
    include_width_16: bool,
}

struct Latencies {
    count: u64,
    storage: LatencyData,
}

enum LatencyData {
    ExactAll(Vec<u64>),
    Log1024 {
        buckets: Vec<u64>,
        validation_samples: Option<Vec<u64>>,
    },
}

impl Latencies {
    fn new(storage_mode: LatencyStorageMode) -> Self {
        let storage = match storage_mode {
            LatencyStorageMode::ExactAll => LatencyData::ExactAll(Vec::new()),
            LatencyStorageMode::Log1024 => LatencyData::Log1024 {
                buckets: vec![0; HISTOGRAM_BUCKETS],
                validation_samples: None,
            },
            LatencyStorageMode::Log1024Validation => LatencyData::Log1024 {
                buckets: vec![0; HISTOGRAM_BUCKETS],
                validation_samples: Some(Vec::new()),
            },
        };
        Self { count: 0, storage }
    }

    fn record(&mut self, nanos: u64) {
        self.count += 1;
        match &mut self.storage {
            LatencyData::ExactAll(samples) => samples.push(nanos),
            LatencyData::Log1024 {
                buckets,
                validation_samples,
            } => {
                buckets[histogram_bucket_index(nanos)] += 1;
                if let Some(samples) = validation_samples {
                    samples.push(nanos);
                }
            }
        }
    }

    fn report(&self) -> Value {
        match &self.storage {
            LatencyData::ExactAll(samples) => {
                let mut sorted = samples.clone();
                sorted.sort_unstable();
                let percentile = |numerator: usize| -> Option<f64> {
                    percentile_rank(sorted.len(), numerator)
                        .map(|rank| sorted[rank] as f64 / 1_000_000.0)
                };
                json!({
                    "storage": "exact-all",
                    "count": self.count,
                    "sample_count": sorted.len(),
                    "sample_limit": null,
                    "sampling": "all recorded completions",
                    "p50_ms": percentile(50),
                    "p95_ms": percentile(95),
                    "p99_ms": percentile(99),
                })
            }
            LatencyData::Log1024 {
                buckets,
                validation_samples,
            } => {
                let percentile = |numerator: usize| -> Option<f64> {
                    let rank = percentile_rank(self.count as usize, numerator)?;
                    histogram_percentile_upper_bound(buckets, rank)
                        .map(|nanos| nanos as f64 / 1_000_000.0)
                };
                let validation = validation_samples
                    .as_ref()
                    .map(|samples| histogram_validation_report(samples, buckets, self.count));
                json!({
                    "storage": if validation_samples.is_some() { "log1024-with-exact-validation" } else { "log1024" },
                    "count": self.count,
                    "sample_count": self.count,
                    "bucket_count": HISTOGRAM_BUCKETS,
                    "sampling": "all recorded completions counted in fixed buckets",
                    "percentile_value": "bucket upper bound in nanoseconds",
                    "maximum_relative_bucket_width": 1.0 / 1024.0,
                    "p50_ms": percentile(50),
                    "p95_ms": percentile(95),
                    "p99_ms": percentile(99),
                    "exact_sample_validation": validation,
                })
            }
        }
    }

    fn merge(&mut self, other: &Self) {
        self.count += other.count;
        match (&mut self.storage, &other.storage) {
            (LatencyData::ExactAll(samples), LatencyData::ExactAll(other_samples)) => {
                samples.extend_from_slice(other_samples);
            }
            (
                LatencyData::Log1024 {
                    buckets,
                    validation_samples,
                },
                LatencyData::Log1024 {
                    buckets: other_buckets,
                    validation_samples: other_samples,
                },
            ) => {
                for (bucket_count, other_count) in buckets.iter_mut().zip(other_buckets) {
                    *bucket_count += other_count;
                }
                match (validation_samples, other_samples) {
                    (Some(samples), Some(other_samples)) => {
                        samples.extend_from_slice(other_samples)
                    }
                    (None, None) => {}
                    _ => {
                        unreachable!("histogram validation mode must match while merging counters")
                    }
                }
            }
            _ => unreachable!("latency storage mode must match while merging counters"),
        }
    }

    fn memory_accounting(&self) -> (usize, usize, usize) {
        match &self.storage {
            LatencyData::ExactAll(samples) => (
                samples.capacity() * std::mem::size_of::<u64>(),
                samples.len() * std::mem::size_of::<u64>(),
                0,
            ),
            LatencyData::Log1024 {
                buckets,
                validation_samples,
            } => {
                let sample_capacity_bytes = validation_samples
                    .as_ref()
                    .map_or(0, |samples| samples.capacity() * std::mem::size_of::<u64>());
                (
                    sample_capacity_bytes,
                    validation_samples
                        .as_ref()
                        .map_or(0, |samples| samples.len() * std::mem::size_of::<u64>()),
                    buckets.capacity() * std::mem::size_of::<u64>(),
                )
            }
        }
    }
}

fn percentile_rank(sample_count: usize, percentile: usize) -> Option<usize> {
    if sample_count == 0 {
        return None;
    }
    let last_index = sample_count - 1;
    Some(((last_index as u128 * percentile as u128).div_ceil(100)) as usize)
}

fn histogram_bucket_index(nanos: u64) -> usize {
    if nanos < HISTOGRAM_EXACT_BUCKETS as u64 {
        return nanos as usize;
    }
    let exponent = u64::BITS as usize - 1 - nanos.leading_zeros() as usize;
    let exponent_lower_bound = 1_u64 << exponent;
    let sub_bucket_width = 1_u64 << (exponent - 10);
    let sub_bucket_index = ((nanos - exponent_lower_bound) / sub_bucket_width) as usize;
    HISTOGRAM_EXACT_BUCKETS
        + (exponent - 10) * HISTOGRAM_SUB_BUCKETS_PER_EXPONENT
        + sub_bucket_index
}

fn histogram_bucket_upper_bound(bucket_index: usize) -> Option<u64> {
    if bucket_index < HISTOGRAM_EXACT_BUCKETS {
        return Some(bucket_index as u64);
    }
    if bucket_index >= HISTOGRAM_BUCKETS {
        return None;
    }
    let relative_index = bucket_index - HISTOGRAM_EXACT_BUCKETS;
    let exponent = 10 + relative_index / HISTOGRAM_SUB_BUCKETS_PER_EXPONENT;
    let sub_bucket_index = relative_index % HISTOGRAM_SUB_BUCKETS_PER_EXPONENT;
    let exponent_lower_bound = 1_u64 << exponent;
    let sub_bucket_width = 1_u64 << (exponent - 10);
    exponent_lower_bound.checked_add(sub_bucket_width * (sub_bucket_index as u64 + 1) - 1)
}

fn histogram_bucket_width(bucket_index: usize) -> Option<u64> {
    if bucket_index < HISTOGRAM_EXACT_BUCKETS {
        return Some(1);
    }
    if bucket_index >= HISTOGRAM_BUCKETS {
        return None;
    }
    let relative_index = bucket_index - HISTOGRAM_EXACT_BUCKETS;
    let exponent = 10 + relative_index / HISTOGRAM_SUB_BUCKETS_PER_EXPONENT;
    Some(1_u64 << (exponent - 10))
}

fn histogram_percentile_upper_bound(buckets: &[u64], rank: usize) -> Option<u64> {
    let mut cumulative_count = 0_u64;
    for (bucket_index, bucket_count) in buckets.iter().copied().enumerate() {
        cumulative_count = cumulative_count.saturating_add(bucket_count);
        if cumulative_count > rank as u64 {
            return histogram_bucket_upper_bound(bucket_index);
        }
    }
    None
}

fn histogram_validation_report(samples: &[u64], buckets: &[u64], recorded_count: u64) -> Value {
    let mut sorted_samples = samples.to_vec();
    sorted_samples.sort_unstable();
    let histogram_count = buckets.iter().copied().sum::<u64>();
    let percentile_checks = [50, 95, 99]
        .into_iter()
        .map(|percentile| {
            let rank = percentile_rank(sorted_samples.len(), percentile);
            let exact_nanos = rank.and_then(|rank| sorted_samples.get(rank).copied());
            let histogram_nanos =
                rank.and_then(|rank| histogram_percentile_upper_bound(buckets, rank));
            let bucket_width_nanos = histogram_nanos
                .map(histogram_bucket_index)
                .and_then(histogram_bucket_width);
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
    let percentiles_passed = percentile_checks
        .iter()
        .all(|check| check.get("within_bucket_width").and_then(Value::as_bool) == Some(true));
    let counts_match = samples.len() as u64 == recorded_count && histogram_count == recorded_count;
    json!({
        "passed": counts_match && percentiles_passed,
        "recorded_count": recorded_count,
        "exact_sample_count": samples.len(),
        "histogram_bucket_count_sum": histogram_count,
        "counts_match": counts_match,
        "percentiles": percentile_checks,
    })
}

struct Counters {
    api_calls: u64,
    read_calls: u64,
    preparatory_gets: u64,
    ordinary_gets: u64,
    query_calls: u64,
    query_rows: u64,
    write_requests: u64,
    successful_write_transactions: u64,
    successful_mutations: u64,
    intended_rejections: u64,
    natural_conflicts: u64,
    overloads: u64,
    other_errors: u64,
    conditional_attempts: u64,
    category_attempts: BTreeMap<String, u64>,
    category_successes: BTreeMap<String, u64>,
    condition_kinds: BTreeMap<String, u64>,
    latencies: BTreeMap<String, Latencies>,
    write_ledger: Vec<LedgerEntry>,
    write_ledger_entries: usize,
    max_write_ledger: usize,
    latency_storage: LatencyStorageMode,
    client_input_prefixes: Vec<ClientInputPrefix>,
    verification_errors: Vec<String>,
    request_limit_reached: bool,
}

impl Default for Counters {
    fn default() -> Self {
        Self::new(LatencyStorageMode::ExactAll, MAX_WRITE_LEDGER)
    }
}

impl Counters {
    fn new(latency_storage: LatencyStorageMode, max_write_ledger: usize) -> Self {
        Self {
            max_write_ledger,
            latency_storage,
            ..Self::default_values()
        }
    }

    fn default_values() -> Self {
        Self {
            api_calls: 0,
            read_calls: 0,
            preparatory_gets: 0,
            ordinary_gets: 0,
            query_calls: 0,
            query_rows: 0,
            write_requests: 0,
            successful_write_transactions: 0,
            successful_mutations: 0,
            intended_rejections: 0,
            natural_conflicts: 0,
            overloads: 0,
            other_errors: 0,
            conditional_attempts: 0,
            category_attempts: BTreeMap::new(),
            category_successes: BTreeMap::new(),
            condition_kinds: BTreeMap::new(),
            latencies: BTreeMap::new(),
            write_ledger: Vec::new(),
            write_ledger_entries: 0,
            max_write_ledger: MAX_WRITE_LEDGER,
            latency_storage: LatencyStorageMode::ExactAll,
            client_input_prefixes: Vec::new(),
            verification_errors: Vec::new(),
            request_limit_reached: false,
        }
    }
}

#[derive(Clone)]
struct LedgerEntry {
    commit_identity: u64,
    workflow_id: u64,
    client_index: usize,
    client_sequence: u64,
    kind: WriteKind,
}

#[derive(Clone)]
struct SeedLedgerEntry {
    start: usize,
    end: usize,
    commit_identity: u64,
}

enum ExpectedState {
    Present { value: Vec<u8>, revision: Revision },
    Missing { revision: Revision },
}

#[derive(Clone)]
struct ClientInputPrefix {
    client_index: usize,
    profile: Profile,
    workflow_count: u64,
    admitted_workflows: u64,
    fingerprint: String,
    workflow_plan_capacity_bytes: usize,
}

impl Counters {
    fn latency(&mut self, name: &str, duration: Duration) {
        self.latencies
            .entry(name.to_owned())
            .or_insert_with(|| Latencies::new(self.latency_storage))
            .record(duration.as_nanos().min(u64::MAX as u128) as u64);
    }

    fn report(&self) -> Value {
        let categories = [
            "put",
            "delete",
            "conditional_width_1",
            "conditional_width_4",
            "conditional_width_8",
            "conditional_width_16",
        ];
        let mut category_report = serde_json::Map::new();
        for category in categories {
            category_report.insert(
                category.to_owned(),
                json!({
                    "attempts": self.category_attempts.get(category).copied().unwrap_or(0),
                    "successes": self.category_successes.get(category).copied().unwrap_or(0),
                }),
            );
        }
        json!({
            "api_calls": self.api_calls,
            "read_calls": self.read_calls,
            "read_fraction": self.read_calls as f64 / self.api_calls.max(1) as f64,
            "write_fraction": self.write_requests as f64 / self.api_calls.max(1) as f64,
            "preparatory_gets": self.preparatory_gets,
            "ordinary_gets": self.ordinary_gets,
            "query16_calls": self.query_calls,
            "query_rows_returned": self.query_rows,
            "write_requests": self.write_requests,
            "successful_write_transactions": self.successful_write_transactions,
            "successful_mutations": self.successful_mutations,
            "intentional_condition_rejections": self.intended_rejections,
            "natural_conflicts": self.natural_conflicts,
            "overloads": self.overloads,
            "other_errors": self.other_errors,
            "conditional_attempts": self.conditional_attempts,
            "write_categories": category_report,
            "condition_kinds": self.condition_kinds,
            "latency": self.latencies.iter().map(|(name, data)| (name.clone(), data.report())).collect::<BTreeMap<_, _>>(),
            "write_ledger_entries": self.write_ledger_entries,
            "write_ledger_limit_per_tenant": self.max_write_ledger,
            "client_input_prefixes": self.client_input_prefixes.iter().map(|entry| json!({
                "client_index": entry.client_index,
                "profile": profile_name(entry.profile),
                "workflow_count": entry.workflow_count,
                "admitted_workflows": entry.admitted_workflows,
                "prefix_limit": INPUT_FINGERPRINT_PREFIX_WORKFLOWS,
                "fingerprint": entry.fingerprint,
                "workflow_plan_capacity_bytes": entry.workflow_plan_capacity_bytes,
            })).collect::<Vec<_>>(),
            "verification_errors": self.verification_errors,
            "request_limit_reached": self.request_limit_reached,
            "collector_memory": self.memory_accounting(),
        })
    }

    fn merge(&mut self, other: &Counters) {
        self.api_calls += other.api_calls;
        self.read_calls += other.read_calls;
        self.preparatory_gets += other.preparatory_gets;
        self.ordinary_gets += other.ordinary_gets;
        self.query_calls += other.query_calls;
        self.query_rows += other.query_rows;
        self.write_requests += other.write_requests;
        self.successful_write_transactions += other.successful_write_transactions;
        self.successful_mutations += other.successful_mutations;
        self.intended_rejections += other.intended_rejections;
        self.natural_conflicts += other.natural_conflicts;
        self.overloads += other.overloads;
        self.other_errors += other.other_errors;
        self.conditional_attempts += other.conditional_attempts;
        self.request_limit_reached |= other.request_limit_reached;
        for (name, count) in &other.category_attempts {
            *self.category_attempts.entry(name.clone()).or_default() += count;
        }
        for (name, count) in &other.category_successes {
            *self.category_successes.entry(name.clone()).or_default() += count;
        }
        for (name, count) in &other.condition_kinds {
            *self.condition_kinds.entry(name.clone()).or_default() += count;
        }
        for (name, latency) in &other.latencies {
            self.latencies
                .entry(name.clone())
                .or_insert_with(|| Latencies::new(self.latency_storage))
                .merge(latency);
        }
        self.write_ledger_entries += other.write_ledger_entries;
        self.client_input_prefixes
            .extend(other.client_input_prefixes.iter().cloned());
        self.verification_errors
            .extend(other.verification_errors.iter().cloned());
    }

    fn memory_accounting(&self) -> Value {
        let mut exact_latency_capacity_bytes = 0_usize;
        let mut exact_latency_sample_bytes = 0_usize;
        let mut histogram_capacity_bytes = 0_usize;
        for latency in self.latencies.values() {
            let (capacity_bytes, sample_bytes, histogram_bytes) = latency.memory_accounting();
            exact_latency_capacity_bytes += capacity_bytes;
            exact_latency_sample_bytes += sample_bytes;
            histogram_capacity_bytes += histogram_bytes;
        }
        let write_ledger_capacity_bytes =
            self.write_ledger.capacity() * std::mem::size_of::<LedgerEntry>();
        let write_ledger_used_bytes = self.write_ledger.len() * std::mem::size_of::<LedgerEntry>();
        let client_prefix_capacity_bytes =
            self.client_input_prefixes.capacity() * std::mem::size_of::<ClientInputPrefix>();
        let client_fingerprint_capacity_bytes = self
            .client_input_prefixes
            .iter()
            .map(|entry| entry.fingerprint.capacity())
            .sum::<usize>();
        let latency_percentile_sort_copy_bytes = self
            .latencies
            .values()
            .map(|latency| match &latency.storage {
                LatencyData::ExactAll(samples) => samples.len() * std::mem::size_of::<u64>(),
                LatencyData::Log1024 {
                    validation_samples: Some(samples),
                    ..
                } => samples.len() * std::mem::size_of::<u64>(),
                LatencyData::Log1024 {
                    validation_samples: None,
                    ..
                } => 0,
            })
            .sum::<usize>();
        json!({
            "scope": "actual collector vector capacities and populated sizes; excludes allocator metadata, map node overhead, runtime, service and document state",
            "latency_exact_capacity_bytes": exact_latency_capacity_bytes,
            "latency_exact_sample_bytes": exact_latency_sample_bytes,
            "latency_histogram_capacity_bytes": histogram_capacity_bytes,
            "write_ledger_length": self.write_ledger.len(),
            "write_ledger_capacity": self.write_ledger.capacity(),
            "write_ledger_used_bytes": write_ledger_used_bytes,
            "write_ledger_capacity_bytes": write_ledger_capacity_bytes,
            "client_input_prefix_length": self.client_input_prefixes.len(),
            "client_input_prefix_capacity": self.client_input_prefixes.capacity(),
            "client_input_prefix_vector_bytes": client_prefix_capacity_bytes,
            "client_input_fingerprint_capacity_bytes": client_fingerprint_capacity_bytes,
            "workflow_plan_capacity_bytes": self.client_input_prefixes.iter().map(|entry| entry.workflow_plan_capacity_bytes).sum::<usize>(),
            "latency_percentile_sort_copy_bytes": latency_percentile_sort_copy_bytes,
        })
    }

    fn collector_memory_capacity_bytes(&self) -> usize {
        let mut latency_capacity_bytes = 0_usize;
        let mut histogram_capacity_bytes = 0_usize;
        for latency in self.latencies.values() {
            let (exact_capacity, _, histogram_capacity) = latency.memory_accounting();
            latency_capacity_bytes += exact_capacity;
            histogram_capacity_bytes += histogram_capacity;
        }
        latency_capacity_bytes
            + histogram_capacity_bytes
            + self.write_ledger.capacity() * std::mem::size_of::<LedgerEntry>()
            + self.client_input_prefixes.capacity() * std::mem::size_of::<ClientInputPrefix>()
            + self
                .client_input_prefixes
                .iter()
                .map(|entry| entry.fingerprint.capacity() + entry.workflow_plan_capacity_bytes)
                .sum::<usize>()
    }

    fn latency_sort_copy_capacity_bytes(&self) -> usize {
        self.latencies
            .values()
            .map(|latency| match &latency.storage {
                LatencyData::ExactAll(samples) => samples.len() * std::mem::size_of::<u64>(),
                LatencyData::Log1024 {
                    validation_samples: Some(samples),
                    ..
                } => samples.len() * std::mem::size_of::<u64>(),
                LatencyData::Log1024 {
                    validation_samples: None,
                    ..
                } => 0,
            })
            .sum()
    }
}

fn parse_options() -> Result<Options, String> {
    let mut values = BTreeMap::new();
    let mut args = std::env::args().skip(1);
    while let Some(key) = args.next() {
        if !key.starts_with("--") {
            return Err(format!("unexpected argument {key}"));
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {key}"))?;
        values.insert(key, value);
    }
    let backend = match values.get("--backend").map(String::as_str).unwrap_or("") {
        "blink" => BackendKind::Blink,
        "rocksdb" => BackendKind::RocksDb,
        "rocksdb-direct" => BackendKind::RocksDbDirect,
        other => return Err(format!("unsupported backend {other}")),
    };
    let client_executor = match values
        .get("--client-executor")
        .map(String::as_str)
        .unwrap_or("async")
    {
        "async" => ClientExecutor::Async,
        "dedicated" => ClientExecutor::Dedicated,
        other => return Err(format!("unsupported client executor {other}")),
    };
    if backend == BackendKind::RocksDbDirect && client_executor != ClientExecutor::Dedicated {
        return Err("rocksdb-direct requires dedicated client threads".to_owned());
    }
    let revision_allocator = match values
        .get("--revision-allocator")
        .map(String::as_str)
        .unwrap_or("serialized")
    {
        "serialized" => dodb_rocksdb::RevisionAllocatorMode::Serialized,
        "reserved-ranges" => dodb_rocksdb::RevisionAllocatorMode::ReservedRanges,
        other => return Err(format!("unsupported revision allocator {other}")),
    };
    let condition_key_lock_mode =
        parse_condition_key_lock_mode(values.get("--condition-key-lock-mode").map(String::as_str))?;
    let engine_health_interval_ms = values
        .get("--engine-health-interval-ms")
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|_| "invalid --engine-health-interval-ms".to_owned())
        })
        .transpose()?
        .unwrap_or(0);
    if engine_health_interval_ms > 0 && engine_health_interval_ms < 1_000 {
        return Err("--engine-health-interval-ms must be 0 or at least 1000".to_owned());
    }
    let blocking_admission = match values
        .get("--blocking-admission")
        .map(String::as_str)
        .unwrap_or("shared")
    {
        "shared" => dodb_rocksdb::BlockingAdmissionMode::Shared,
        "split-read-write" => dodb_rocksdb::BlockingAdmissionMode::SplitReadWrite,
        "split-read-2-write-8" => dodb_rocksdb::BlockingAdmissionMode::SplitRead2Write8,
        other => return Err(format!("unsupported blocking admission {other}")),
    };
    let shared_blocking_slots = values
        .get("--shared-blocking-slots")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| "invalid --shared-blocking-slots".to_owned())
        })
        .transpose()?
        .unwrap_or(8);
    if shared_blocking_slots == 0 {
        return Err("--shared-blocking-slots must be positive".to_owned());
    }
    if blocking_admission.split_slot_limits().is_some()
        && values.contains_key("--shared-blocking-slots")
    {
        return Err("--shared-blocking-slots cannot be combined with split admission".to_owned());
    }
    let scenario = match values.get("--scenario").map(String::as_str).unwrap_or("") {
        "rw95" => Scenario::Single(Profile::Rw95),
        "rw80" => Scenario::Single(Profile::Rw80),
        "rw50" => Scenario::Single(Profile::Rw50),
        "rw20" => Scenario::Single(Profile::Rw20),
        "mt-mix" => Scenario::Multi,
        "smoke" => Scenario::Smoke,
        other => return Err(format!("unsupported scenario {other}")),
    };
    if backend == BackendKind::Blink && shared_blocking_slots != 8 {
        return Err("--shared-blocking-slots applies only to RocksDB".to_owned());
    }
    if backend == BackendKind::Blink
        && blocking_admission == dodb_rocksdb::BlockingAdmissionMode::SplitRead2Write8
    {
        return Err("split-read-2-write-8 admission applies only to RocksDB".to_owned());
    }
    let parse = |name: &str, default: u64| -> Result<u64, String> {
        values
            .get(name)
            .map(|value| value.parse().map_err(|_| format!("invalid {name}")))
            .unwrap_or(Ok(default))
    };
    let research_metrics = parse("--research-metrics", 0)? == 1;
    if backend == BackendKind::RocksDbDirect
        && (research_metrics || blocking_admission != dodb_rocksdb::BlockingAdmissionMode::Shared)
    {
        return Err("rocksdb-direct requires metrics OFF and shared admission".to_owned());
    }
    let read_stage_metrics = parse("--read-stage-metrics", 0)? == 1;
    let detailed_write_metrics =
        parse("--detailed-write-metrics", u64::from(!read_stage_metrics))? == 1;
    let mutation_workload = match values
        .get("--mutation-workload")
        .map(String::as_str)
        .unwrap_or("new-key")
    {
        "new-key" => MutationWorkload::NewKey,
        "overwrite" => MutationWorkload::Overwrite,
        other => return Err(format!("unsupported mutation workload {other}")),
    };
    let overwrite_slots_per_client = usize::try_from(parse("--overwrite-slots-per-client", 1_024)?)
        .map_err(|_| "invalid --overwrite-slots-per-client".to_owned())?;
    if overwrite_slots_per_client == 0 {
        return Err("--overwrite-slots-per-client must be positive".to_owned());
    }
    if overwrite_slots_per_client > (1_usize << 32) {
        return Err("--overwrite-slots-per-client exceeds the key-space bound".to_owned());
    }
    let latency_storage = match values
        .get("--latency-storage")
        .map(String::as_str)
        .unwrap_or("exact-all")
    {
        "exact-all" => LatencyStorageMode::ExactAll,
        "log1024" => LatencyStorageMode::Log1024,
        "log1024-validate" => LatencyStorageMode::Log1024Validation,
        other => return Err(format!("unsupported latency storage {other}")),
    };
    let max_write_ledger = usize::try_from(parse("--max-write-ledger", MAX_WRITE_LEDGER as u64)?)
        .map_err(|_| "invalid --max-write-ledger".to_owned())?;
    if max_write_ledger == 0 {
        return Err("--max-write-ledger must be positive".to_owned());
    }
    let parse_switch = |name: &str| -> Result<bool, String> {
        match parse(name, 0)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(format!("{name} must be 0 or 1")),
        }
    };
    Ok(Options {
        backend,
        client_executor,
        scenario,
        data_dir: PathBuf::from(values.get("--data-dir").ok_or("--data-dir is required")?),
        seed: parse("--seed", 2_026_101_3)?,
        warmup: Duration::from_millis(parse("--warmup-ms", 1_000)?),
        measurement: Duration::from_millis(parse("--measurement-ms", 5_000)?),
        clients: parse("--clients", 16)? as usize,
        initial_rows: parse("--rows", INITIAL_ROWS as u64)? as usize,
        value_size: parse("--value-size", VALUE_SIZE as u64)? as usize,
        call_cap: parse("--call-cap", 1_000_000)?,
        report_path: values.get("--report").map(PathBuf::from),
        research_metrics,
        read_stage_metrics,
        detailed_write_metrics,
        revision_allocator,
        condition_key_lock_mode,
        engine_health_interval_ms,
        blocking_admission,
        shared_blocking_slots,
        mutation_workload,
        overwrite_slots_per_client,
        latency_storage,
        max_write_ledger,
        require_full_window: parse_switch("--require-full-window")?,
        include_width_16: parse_switch("--include-width-16")?,
    })
}

fn parse_condition_key_lock_mode(
    value: Option<&str>,
) -> Result<dodb_rocksdb::ConditionKeyLockMode, String> {
    match value.unwrap_or("exclusive") {
        "exclusive" => Ok(dodb_rocksdb::ConditionKeyLockMode::Exclusive),
        "shared" => Ok(dodb_rocksdb::ConditionKeyLockMode::Shared),
        other => Err(format!("unsupported condition key lock mode {other}")),
    }
}

fn profile_name(profile: Profile) -> &'static str {
    match profile {
        Profile::Rw95 => "rw95",
        Profile::Rw80 => "rw80",
        Profile::Rw50 => "rw50",
        Profile::Rw20 => "rw20",
    }
}

fn profile_read_percentage(profile: Profile) -> usize {
    match profile {
        Profile::Rw95 => 95,
        Profile::Rw80 => 80,
        Profile::Rw50 => 50,
        Profile::Rw20 => 20,
    }
}

fn category_name(kind: WriteKind) -> &'static str {
    match kind {
        WriteKind::Put => "put",
        WriteKind::Delete => "delete",
        WriteKind::ConditionalOne => "conditional_width_1",
        WriteKind::TransactionFour => "conditional_width_4",
        WriteKind::TransactionEight => "conditional_width_8",
        WriteKind::TransactionSixteen => "conditional_width_16",
    }
}

fn condition_name(kind: usize) -> &'static str {
    match kind % 4 {
        0 => "RevisionEquals",
        1 | 2 => "Exists",
        _ => "NotExists",
    }
}

fn make_plan(profile: Profile, seed: u64, include_width_16: bool) -> Vec<WorkflowPlan> {
    let read_calls = profile_read_percentage(profile) * 10;
    let write_calls = 1_000 - read_calls;
    let put_count = write_calls * 20 / 100;
    let delete_count = write_calls * 10 / 100;
    let conditional_one = write_calls * 30 / 100;
    let transaction_four = write_calls * 30 / 100;
    let mut transaction_eight =
        write_calls - put_count - delete_count - conditional_one - transaction_four;
    let transaction_sixteen = if include_width_16 {
        let moved_count = transaction_eight / 2;
        transaction_eight -= moved_count;
        moved_count
    } else {
        0
    };
    let conditional_count =
        conditional_one + transaction_four + transaction_eight + transaction_sixteen;
    let revision_get_count = (conditional_count + 3) / 4;
    let prep_get_count = revision_get_count;
    let query_count = read_calls / 5;
    let get_count = read_calls - query_count;
    let plain_get_count = get_count - prep_get_count;
    let mut plans = Vec::with_capacity(1_000 - prep_get_count);
    for ordinal in 0..plain_get_count {
        plans.push(WorkflowPlan {
            kind: WorkflowKind::Read(ReadKind::Get),
            key_ordinal: 500 + (ordinal % 8_500) as u64,
        });
    }
    for ordinal in 0..query_count {
        plans.push(WorkflowPlan {
            kind: WorkflowKind::Read(ReadKind::Query16),
            key_ordinal: (ordinal * 37) as u64,
        });
    }
    let mut writes = Vec::with_capacity(write_calls);
    for (kind, count) in [
        (WriteKind::Put, put_count),
        (WriteKind::Delete, delete_count),
        (WriteKind::ConditionalOne, conditional_one),
        (WriteKind::TransactionFour, transaction_four),
        (WriteKind::TransactionEight, transaction_eight),
        (WriteKind::TransactionSixteen, transaction_sixteen),
    ] {
        for _ in 0..count {
            writes.push(kind);
        }
    }
    shuffle(&mut writes, seed ^ profile_read_percentage(profile) as u64);
    let mut conditional_ordinal = 0;
    for (ordinal, kind) in writes.into_iter().enumerate() {
        let conditional = !matches!(kind, WriteKind::Put | WriteKind::Delete);
        let revision_get = conditional && conditional_ordinal % 4 == 0;
        let intended_rejection = conditional && conditional_ordinal % 10 == 0;
        let condition_kind = conditional_ordinal % 4;
        if revision_get {
            plans.push(WorkflowPlan {
                kind: WorkflowKind::Write {
                    kind,
                    revision_get: true,
                    intended_rejection,
                    condition_kind,
                },
                key_ordinal: 9_000 + (seed.rotate_left(11) ^ ordinal as u64) % 1_000,
            });
        } else {
            plans.push(WorkflowPlan {
                kind: WorkflowKind::Write {
                    kind,
                    revision_get: false,
                    intended_rejection,
                    condition_kind,
                },
                key_ordinal: 9_000 + (seed.rotate_left(11) ^ ordinal as u64) % 1_000,
            });
        }
        if conditional {
            conditional_ordinal += 1;
        }
    }
    shuffle(&mut plans, seed ^ 0x9e37_79b9_7f4a_7c15);
    plans
}

fn shuffle<T>(values: &mut [T], mut state: u64) {
    if state == 0 {
        state = 0xa076_1d64_78bd_642f;
    }
    for remaining in (1..values.len()).rev() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        values.swap(remaining, state as usize % (remaining + 1));
    }
}

fn profile_for_tenant(tenant_index: usize) -> Profile {
    [Profile::Rw95, Profile::Rw80, Profile::Rw50, Profile::Rw20][tenant_index]
}

fn profile_for_scenario(scenario: Scenario, tenant_index: usize) -> Profile {
    match scenario {
        Scenario::Single(profile) => profile,
        Scenario::Multi | Scenario::Smoke => profile_for_tenant(tenant_index),
    }
}

fn schedule_fingerprint(profile: Profile, seed: u64, options: &Options) -> String {
    let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
    for value in [
        if options.include_width_16 {
            16_u64
        } else {
            8_u64
        },
        options.overwrite_slots_per_client as u64,
        options.mutation_workload as u64,
    ] {
        for byte in value.to_le_bytes() {
            fingerprint ^= byte as u64;
            fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    for plan in make_plan(profile, seed, options.include_width_16) {
        let (tag, flags) = match plan.kind {
            WorkflowKind::Read(ReadKind::Get) => (1_u64, 0_u64),
            WorkflowKind::Read(ReadKind::Query16) => (2, 0),
            WorkflowKind::Write {
                kind,
                revision_get,
                intended_rejection,
                condition_kind,
            } => {
                let tag = match kind {
                    WriteKind::Put => 3,
                    WriteKind::Delete => 4,
                    WriteKind::ConditionalOne => 5,
                    WriteKind::TransactionFour => 6,
                    WriteKind::TransactionEight => 7,
                    WriteKind::TransactionSixteen => 8,
                };
                let flags = (revision_get as u64)
                    | ((intended_rejection as u64) << 1)
                    | ((condition_kind as u64) << 2);
                (tag, flags)
            }
        };
        for byte in tag
            .to_le_bytes()
            .into_iter()
            .chain(flags.to_le_bytes())
            .chain(plan.key_ordinal.to_le_bytes())
        {
            fingerprint ^= byte as u64;
            fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    format!("{fingerprint:016x}")
}

fn input_workflow_fingerprint(
    profile: Profile,
    seed: u64,
    client_index: usize,
    tenant_index: usize,
    workflow_id: u64,
    client_sequence: u64,
    value_size: usize,
    options: &Options,
    plan: &WorkflowPlan,
) -> u64 {
    let (tag, flags) = match &plan.kind {
        WorkflowKind::Read(ReadKind::Get) => (1_u64, 0_u64),
        WorkflowKind::Read(ReadKind::Query16) => (2, 0),
        WorkflowKind::Write {
            kind,
            revision_get,
            intended_rejection,
            condition_kind,
        } => {
            let tag = match *kind {
                WriteKind::Put => 3,
                WriteKind::Delete => 4,
                WriteKind::ConditionalOne => 5,
                WriteKind::TransactionFour => 6,
                WriteKind::TransactionEight => 7,
                WriteKind::TransactionSixteen => 8,
            };
            let flags = (*revision_get as u64)
                | ((*intended_rejection as u64) << 1)
                | ((*condition_kind as u64) << 2);
            (tag, flags)
        }
    };
    let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
    for value in [
        profile_read_percentage(profile) as u64,
        seed,
        client_index as u64,
        tenant_index as u64,
        workflow_id,
        client_sequence,
        value_size as u64,
        options.mutation_workload as u64,
        options.overwrite_slots_per_client as u64,
        if options.include_width_16 { 16 } else { 8 },
        tag,
        flags,
        plan.key_ordinal,
    ] {
        for byte in value.to_le_bytes() {
            fingerprint ^= byte as u64;
            fingerprint = fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    fingerprint
}

fn key_from_ordinal(ordinal: u64) -> DocumentKey {
    DocumentKey::new(
        (ordinal / 10).to_be_bytes().to_vec(),
        (ordinal % 10).to_be_bytes().to_vec(),
    )
}

fn mutation_key(
    options: &Options,
    tenant_index: usize,
    workflow_id: u64,
    client_index: usize,
    client_sequence: u64,
    mutation_index: usize,
    kind: WriteKind,
) -> DocumentKey {
    if options.mutation_workload == MutationWorkload::Overwrite {
        let slot = client_sequence % options.overwrite_slots_per_client as u64;
        let ordinal = 1_u64 << 60
            | (tenant_index as u64) << 52
            | (client_index as u64) << 36
            | slot * 16
            | mutation_index as u64;
        return key_from_ordinal(ordinal);
    }
    if kind == WriteKind::Delete && mutation_index == 0 && workflow_id % 2 == 0 {
        return key_from_ordinal(workflow_id % 500);
    }
    let ordinal = 1_000_000_000_u64
        .wrapping_add((tenant_index as u64) << 52)
        .wrapping_add(workflow_id.wrapping_mul(if options.include_width_16 { 16 } else { 8 }))
        .wrapping_add(mutation_index as u64);
    key_from_ordinal(ordinal)
}

fn generated_value(size: usize, seed: u64, workflow_id: u64, mutation_index: usize) -> Vec<u8> {
    let mut value = vec![0; size];
    let mut state = seed ^ workflow_id.rotate_left(17) ^ mutation_index as u64;
    for (byte_index, byte) in value.iter_mut().enumerate() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *byte = (state as u8).wrapping_add(byte_index as u8);
    }
    value
}

fn initial_value(size: usize, tenant_index: usize, ordinal: usize) -> Vec<u8> {
    let mut value = vec![0; size];
    let marker = format!("initial:{tenant_index}:{ordinal}:");
    let marker_bytes = marker.as_bytes();
    let copy_length = marker_bytes.len().min(size);
    value[..copy_length].copy_from_slice(&marker_bytes[..copy_length]);
    for (byte_index, byte) in value.iter_mut().enumerate().skip(copy_length) {
        *byte = ((ordinal.wrapping_mul(31) + byte_index + tenant_index) % 251) as u8;
    }
    value
}

async fn seed_tenant(
    service: &DriverService,
    tenant: TenantId,
    tenant_index: usize,
    rows: usize,
    value_size: usize,
) -> Result<Vec<SeedLedgerEntry>, Error> {
    let mut start = 0;
    let mut ledger = Vec::new();
    while start < rows {
        let end = (start + 64).min(rows);
        let mutations = (start..end)
            .map(|ordinal| TransactionMutation::Put {
                key: key_from_ordinal(ordinal as u64),
                value: initial_value(value_size, tenant_index, ordinal),
            })
            .collect();
        let response = service
            .execute(
                tenant,
                Request::Transact {
                    request: TransactionRequest::new(Vec::new(), mutations),
                },
            )
            .await?;
        let Response::Transact(outcome) = response else {
            return Err(Error::invalid_request(
                "seed transaction returned an unexpected response",
            ));
        };
        let commit_identity = outcome
            .commit_lsn
            .ok_or_else(|| Error::invalid_request("seed transaction returned no commit identity"))?
            .get();
        ledger.push(SeedLedgerEntry {
            start,
            end,
            commit_identity,
        });
        start = end;
    }
    Ok(ledger)
}

async fn execute_workflow(
    service: &DriverService,
    tenant: TenantId,
    tenant_index: usize,
    plan: WorkflowPlan,
    workflow_id: u64,
    client_index: usize,
    client_sequence: u64,
    options: &Options,
    counters: &Mutex<Counters>,
) {
    let workflow_started = Instant::now();
    match plan.kind {
        WorkflowKind::Read(ReadKind::Get) => {
            let key = key_from_ordinal(plan.key_ordinal % options.initial_rows.max(1) as u64);
            let started = Instant::now();
            let result = service
                .execute(tenant, Request::Get { key: key.clone() })
                .await;
            let elapsed = started.elapsed();
            let mut state = counters.lock().unwrap();
            state.api_calls += 1;
            state.read_calls += 1;
            state.ordinary_gets += 1;
            state.latency("get", elapsed);
            match result {
                Ok(Response::Get(RevisionState::Present { value, .. })) => {
                    let expected = initial_value(
                        options.value_size,
                        tenant_index,
                        plan.key_ordinal as usize % options.initial_rows.max(1),
                    );
                    if value != expected {
                        state
                            .verification_errors
                            .push("Get returned unexpected seeded value".to_owned());
                    }
                }
                Ok(Response::Get(_)) => state
                    .verification_errors
                    .push("Get returned missing seeded key".to_owned()),
                Ok(_) => state.other_errors += 1,
                Err(error) => record_error(&mut state, error),
            }
        }
        WorkflowKind::Read(ReadKind::Query16) => {
            let partition_count = (options.initial_rows / 10).max(1);
            let partition =
                (plan.key_ordinal as usize * 97 + workflow_id as usize) % partition_count;
            let cursor_ordinal = (plan.key_ordinal as usize + workflow_id as usize) % 9;
            let pk_key = key_from_ordinal((partition * 10) as u64);
            let cursor = dodb_core::SortKey::new((cursor_ordinal as u64).to_be_bytes().to_vec());
            let started = Instant::now();
            let result = service
                .execute(
                    tenant,
                    Request::Query {
                        pk: pk_key.pk.clone(),
                        exclusive_after_sk: Some(cursor),
                        limit: 16,
                    },
                )
                .await;
            let elapsed = started.elapsed();
            let mut state = counters.lock().unwrap();
            state.api_calls += 1;
            state.read_calls += 1;
            state.query_calls += 1;
            state.latency("query16", elapsed);
            match result {
                Ok(Response::Query(rows)) => {
                    state.query_rows += rows.len() as u64;
                    if rows.iter().any(|row| row.value.len() != options.value_size)
                        || rows.windows(2).any(|pair| pair[0].key >= pair[1].key)
                        || rows.iter().any(|row| row.key.pk != pk_key.pk)
                    {
                        state
                            .verification_errors
                            .push("Query16 returned invalid ordered rows".to_owned());
                    }
                }
                Ok(_) => state.other_errors += 1,
                Err(error) => record_error(&mut state, error),
            }
        }
        WorkflowKind::Write {
            kind,
            revision_get,
            intended_rejection,
            condition_kind,
        } => {
            let mut observed_revision = None;
            let condition_key =
                key_from_ordinal(plan.key_ordinal % options.initial_rows.max(1) as u64);
            if revision_get {
                let started = Instant::now();
                let result = dodb_rocksdb::with_research_read_kind(
                    dodb_rocksdb::ResearchReadKind::PreparatoryGet,
                    service.execute(
                        tenant,
                        Request::Get {
                            key: condition_key.clone(),
                        },
                    ),
                )
                .await;
                let elapsed = started.elapsed();
                let mut state = counters.lock().unwrap();
                state.api_calls += 1;
                state.read_calls += 1;
                state.preparatory_gets += 1;
                state.latency("preparatory_get", elapsed);
                match result {
                    Ok(Response::Get(RevisionState::Present { value, revision })) => {
                        observed_revision = Some(revision);
                        if value.len() != options.value_size {
                            state
                                .verification_errors
                                .push("preparatory Get returned invalid value length".to_owned());
                        }
                    }
                    Ok(Response::Get(RevisionState::Missing { revision })) => {
                        observed_revision = Some(revision);
                    }
                    Ok(_) => state
                        .verification_errors
                        .push("preparatory Get returned an unexpected response".to_owned()),
                    Err(error) => record_error(&mut state, error),
                }
            }
            let width = match kind {
                WriteKind::Put | WriteKind::Delete | WriteKind::ConditionalOne => 1,
                WriteKind::TransactionFour => 4,
                WriteKind::TransactionEight => 8,
                WriteKind::TransactionSixteen => 16,
            };
            let mut mutations = Vec::with_capacity(width);
            for mutation_index in 0..width {
                let key = mutation_key(
                    options,
                    tenant_index,
                    workflow_id,
                    client_index,
                    client_sequence,
                    mutation_index,
                    kind,
                );
                if kind == WriteKind::Delete {
                    mutations.push(TransactionMutation::Delete { key });
                } else {
                    mutations.push(TransactionMutation::Put {
                        key,
                        value: generated_value(
                            options.value_size,
                            options.seed,
                            workflow_id,
                            mutation_index,
                        ),
                    });
                }
            }
            let mut conditions = Vec::new();
            if !matches!(kind, WriteKind::Put | WriteKind::Delete) {
                let expectation = match condition_kind % 4 {
                    0 => {
                        let revision = observed_revision.unwrap_or(Revision::new(1));
                        let expected_revision = if intended_rejection {
                            Revision::new(if revision.get() == 0 { u64::MAX } else { 0 })
                        } else {
                            revision
                        };
                        TransactionCondition::RevisionEquals {
                            key: condition_key.clone(),
                            expected_revision,
                        }
                    }
                    1 | 2 => TransactionCondition::Exists {
                        key: condition_key.clone(),
                    },
                    _ => TransactionCondition::NotExists {
                        key: condition_key.clone(),
                    },
                };
                let condition = if intended_rejection {
                    match expectation {
                        TransactionCondition::Exists { .. } => TransactionCondition::Exists {
                            key: key_from_ordinal((options.initial_rows + 50_000) as u64),
                        },
                        TransactionCondition::NotExists { .. } => TransactionCondition::NotExists {
                            key: key_from_ordinal(options.initial_rows.saturating_sub(1) as u64),
                        },
                        revision => revision,
                    }
                } else {
                    match expectation {
                        TransactionCondition::NotExists { .. } => TransactionCondition::NotExists {
                            key: key_from_ordinal((options.initial_rows + 75_000) as u64),
                        },
                        condition => condition,
                    }
                };
                conditions.push(condition);
                if width >= 4 && workflow_id % 3 == 0 {
                    conditions.push(TransactionCondition::Exists {
                        key: key_from_ordinal(
                            (plan.key_ordinal + 1) % options.initial_rows.max(1) as u64,
                        ),
                    });
                }
            }
            let request = match kind {
                WriteKind::Put => Request::Put {
                    key: mutations[0].key().clone(),
                    value: generated_value(options.value_size, options.seed, workflow_id, 0),
                },
                WriteKind::Delete => Request::Delete {
                    key: mutations[0].key().clone(),
                },
                _ => Request::Transact {
                    request: TransactionRequest::new(conditions, mutations),
                },
            };
            let started = Instant::now();
            let result = service.execute(tenant, request).await;
            let elapsed = started.elapsed();
            let success = result.is_ok();
            let mut state = counters.lock().unwrap();
            state.api_calls += 1;
            state.write_requests += 1;
            *state
                .category_attempts
                .entry(category_name(kind).to_owned())
                .or_default() += 1;
            state.latency(
                if success {
                    "write_transaction_success"
                } else if intended_rejection {
                    "condition_rejection"
                } else {
                    "write_error"
                },
                elapsed,
            );
            if !matches!(kind, WriteKind::Put | WriteKind::Delete) {
                state.conditional_attempts += 1;
                *state
                    .condition_kinds
                    .entry(condition_name(condition_kind).to_owned())
                    .or_default() += 1;
            }
            match result {
                Ok(Response::Put(revision)) | Ok(Response::Delete(revision)) => {
                    state.successful_write_transactions += 1;
                    state.successful_mutations += 1;
                    *state
                        .category_successes
                        .entry(category_name(kind).to_owned())
                        .or_default() += 1;
                    append_ledger(
                        &mut state,
                        LedgerEntry {
                            commit_identity: revision.get(),
                            workflow_id,
                            client_index,
                            client_sequence,
                            kind,
                        },
                    );
                }
                Ok(Response::Transact(outcome)) if !intended_rejection => {
                    let commit_identity = outcome.commit_lsn.map_or(0, |lsn| lsn.get());
                    if commit_identity == 0 {
                        state
                            .verification_errors
                            .push("mutation transaction returned no commit identity".to_owned());
                    }
                    state.successful_write_transactions += 1;
                    state.successful_mutations += width as u64;
                    *state
                        .category_successes
                        .entry(category_name(kind).to_owned())
                        .or_default() += 1;
                    append_ledger(
                        &mut state,
                        LedgerEntry {
                            commit_identity,
                            workflow_id,
                            client_index,
                            client_sequence,
                            kind,
                        },
                    );
                }
                Ok(Response::Transact(_)) => {
                    state
                        .verification_errors
                        .push("intended rejected condition unexpectedly committed".to_owned());
                    state.other_errors += 1;
                }
                Ok(_) => state.other_errors += 1,
                Err(Error::Conflict(_)) if intended_rejection => state.intended_rejections += 1,
                Err(Error::Conflict(_)) => state.natural_conflicts += 1,
                Err(error) => record_error(&mut state, error),
            }
        }
    }
    counters
        .lock()
        .unwrap()
        .latency("workflow", workflow_started.elapsed());
}

fn append_ledger(counters: &mut Counters, entry: LedgerEntry) {
    if counters.write_ledger.len() < counters.max_write_ledger {
        counters.write_ledger.push(entry);
        counters.write_ledger_entries += 1;
    } else {
        counters.request_limit_reached = true;
    }
}

fn record_error(counters: &mut Counters, error: Error) {
    match error {
        Error::Overloaded(_) => counters.overloads += 1,
        Error::Conflict(_) => counters.natural_conflicts += 1,
        other => {
            counters.other_errors += 1;
            if counters.verification_errors.len() < 128 {
                counters.verification_errors.push(other.to_string());
            }
        }
    }
}

async fn run_clients(
    service: Arc<DriverService>,
    assignments: Vec<(usize, TenantId, usize, Profile)>,
    options: Arc<Options>,
    counters: Arc<Vec<Mutex<Counters>>>,
    run_for: Duration,
    call_cap: u64,
    workflow_id_base: u64,
) -> (Duration, Duration, u64, bool) {
    let admission_start = Instant::now();
    let deadline = admission_start + run_for;
    let accepting = Arc::new(AtomicBool::new(true));
    let tenant_count = assignments
        .iter()
        .map(|(_, _, tenant_index, _)| tenant_index + 1)
        .max()
        .unwrap_or(1);
    let admitted_calls = Arc::new(
        (0..tenant_count)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<_>>(),
    );
    let mut tasks = JoinSet::new();
    for (client_index, tenant, tenant_index, profile) in assignments {
        let service = Arc::clone(&service);
        let options = Arc::clone(&options);
        let counters = Arc::clone(&counters);
        let accepting = Arc::clone(&accepting);
        let admitted_calls = Arc::clone(&admitted_calls);
        let tenant_cap = if matches!(options.scenario, Scenario::Smoke) {
            call_cap
        } else {
            (call_cap / tenant_count as u64).max(1)
        };
        let client_executor = options.client_executor;
        let client_operation = async move {
            let plan = make_plan(
                profile,
                options.seed ^ tenant_index as u64,
                options.include_width_16,
            );
            let mut plan_index = client_index % plan.len();
            let mut client_sequence = 0_u64;
            let mut prefix_workflow_count = 0_u64;
            let mut admitted_workflows = 0_u64;
            let mut input_fingerprint = 0xcbf2_9ce4_8422_2325_u64;
            while accepting.load(Ordering::Relaxed) && Instant::now() < deadline {
                if counters[tenant_index].lock().unwrap().request_limit_reached {
                    break;
                }
                let next = plan[plan_index].clone();
                let required_calls = match next.kind {
                    WorkflowKind::Write {
                        revision_get: true, ..
                    } => 2,
                    _ => 1,
                };
                let mut current = admitted_calls[tenant_index].load(Ordering::Relaxed);
                loop {
                    if current.saturating_add(required_calls) > tenant_cap {
                        break;
                    }
                    match admitted_calls[tenant_index].compare_exchange_weak(
                        current,
                        current + required_calls,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    ) {
                        Ok(_) => break,
                        Err(updated) => current = updated,
                    }
                }
                if admitted_calls[tenant_index].load(Ordering::Relaxed) > tenant_cap {
                    break;
                }
                if current.saturating_add(required_calls) > tenant_cap {
                    break;
                }
                plan_index = (plan_index + 1) % plan.len();
                let next_id = workflow_id_base
                    .wrapping_add((client_index as u64) << 40)
                    .wrapping_add(client_sequence);
                client_sequence += 1;
                if prefix_workflow_count < INPUT_FINGERPRINT_PREFIX_WORKFLOWS {
                    let workflow_fingerprint = input_workflow_fingerprint(
                        profile,
                        options.seed ^ tenant_index as u64,
                        client_index,
                        tenant_index,
                        next_id,
                        client_sequence,
                        options.value_size,
                        &options,
                        &next,
                    );
                    for byte in workflow_fingerprint.to_le_bytes() {
                        input_fingerprint ^= byte as u64;
                        input_fingerprint = input_fingerprint.wrapping_mul(0x0000_0100_0000_01b3);
                    }
                    prefix_workflow_count += 1;
                }
                admitted_workflows += 1;
                execute_workflow(
                    &service,
                    tenant,
                    tenant_index,
                    next,
                    next_id,
                    client_index,
                    client_sequence - 1,
                    &options,
                    &counters[tenant_index],
                )
                .await;
            }
            counters[tenant_index]
                .lock()
                .unwrap()
                .client_input_prefixes
                .push(ClientInputPrefix {
                    client_index,
                    profile,
                    workflow_count: prefix_workflow_count,
                    admitted_workflows,
                    fingerprint: format!("{input_fingerprint:016x}"),
                    workflow_plan_capacity_bytes: plan.capacity()
                        * std::mem::size_of::<WorkflowPlan>(),
                });
        };
        match client_executor {
            ClientExecutor::Async => {
                tasks.spawn(client_operation);
            }
            ClientExecutor::Dedicated => {
                let runtime = tokio::runtime::Handle::current();
                tasks.spawn_blocking(move || runtime.block_on(client_operation));
            }
        }
    }
    while Instant::now() < deadline
        && accepting.load(Ordering::Relaxed)
        && admitted_calls.iter().any(|calls| {
            calls.load(Ordering::Relaxed)
                < if matches!(options.scenario, Scenario::Smoke) {
                    call_cap
                } else {
                    (call_cap / tenant_count as u64).max(1)
                }
        })
    {
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    accepting.store(false, Ordering::Relaxed);
    let admission_elapsed = admission_start.elapsed();
    while tasks.join_next().await.is_some() {}
    let total_elapsed = admission_start.elapsed();
    let drain = total_elapsed.saturating_sub(admission_elapsed);
    let calls = counters
        .iter()
        .map(|counter| counter.lock().unwrap().api_calls)
        .sum();
    let cap = if matches!(options.scenario, Scenario::Smoke) {
        call_cap
    } else {
        (call_cap / tenant_count as u64).max(1)
    };
    let call_cap_reached = admitted_calls
        .iter()
        .any(|tenant_calls| tenant_calls.load(Ordering::Relaxed) >= cap);
    (admission_elapsed, drain, calls, call_cap_reached)
}

fn assignments(scenario: Scenario, clients: usize) -> Vec<(usize, TenantId, usize, Profile)> {
    match scenario {
        Scenario::Single(profile) => (0..clients)
            .map(|client| (client, TenantId::new(1), 0, profile))
            .collect(),
        Scenario::Multi => (0..clients)
            .map(|client| {
                let tenant_index = client * 4 / clients;
                (
                    client,
                    TenantId::new(tenant_index as u64 + 1),
                    tenant_index,
                    profile_for_tenant(tenant_index),
                )
            })
            .collect(),
        Scenario::Smoke => (0..clients)
            .map(|client| {
                let tenant_index = client % 4;
                (
                    client,
                    TenantId::new(tenant_index as u64 + 1),
                    tenant_index,
                    profile_for_tenant(tenant_index),
                )
            })
            .collect(),
    }
}

async fn seed_all(
    service: &DriverService,
    scenario: Scenario,
    rows: usize,
    value_size: usize,
) -> Result<Vec<Vec<SeedLedgerEntry>>, Error> {
    let tenant_count = if matches!(scenario, Scenario::Multi | Scenario::Smoke) {
        4
    } else {
        1
    };
    let mut seed_ledger = Vec::with_capacity(tenant_count);
    for tenant_index in 0..tenant_count {
        seed_ledger.push(
            seed_tenant(
                service,
                TenantId::new(tenant_index as u64 + 1),
                tenant_index,
                rows,
                value_size,
            )
            .await?,
        );
    }
    Ok(seed_ledger)
}

async fn verify_tenant(
    service: &DriverService,
    tenant: TenantId,
    tenant_index: usize,
    options: &Options,
    seed_ledger: &[SeedLedgerEntry],
    counters: &Mutex<Counters>,
) -> Value {
    let mut sorted_ledger = counters.lock().unwrap().write_ledger.clone();
    sorted_ledger.sort_by_key(|entry| entry.commit_identity);
    let commit_identity_unique = sorted_ledger
        .windows(2)
        .all(|pair| pair[0].commit_identity != pair[1].commit_identity);
    let mut expected = BTreeMap::new();
    for batch in seed_ledger {
        for ordinal in batch.start..batch.end {
            expected.insert(
                key_from_ordinal(ordinal as u64),
                ExpectedState::Present {
                    value: initial_value(options.value_size, tenant_index, ordinal),
                    revision: Revision::new(batch.commit_identity),
                },
            );
        }
    }
    for entry in &sorted_ledger {
        let width = match entry.kind {
            WriteKind::Put | WriteKind::Delete | WriteKind::ConditionalOne => 1,
            WriteKind::TransactionFour => 4,
            WriteKind::TransactionEight => 8,
            WriteKind::TransactionSixteen => 16,
        };
        for mutation_index in 0..width {
            let key = mutation_key(
                options,
                tenant_index,
                entry.workflow_id,
                entry.client_index,
                entry.client_sequence,
                mutation_index,
                entry.kind,
            );
            if entry.kind == WriteKind::Delete {
                expected.insert(
                    key,
                    ExpectedState::Missing {
                        revision: Revision::new(entry.commit_identity),
                    },
                );
            } else {
                expected.insert(
                    key,
                    ExpectedState::Present {
                        value: generated_value(
                            options.value_size,
                            options.seed,
                            entry.workflow_id,
                            mutation_index,
                        ),
                        revision: Revision::new(entry.commit_identity),
                    },
                );
            }
        }
    }
    let expected_row_count = expected
        .values()
        .filter(|state| matches!(state, ExpectedState::Present { .. }))
        .count();
    let mut actual_rows = 0_usize;
    let mut missing_rows = 0_usize;
    let mut mismatch_samples = Vec::new();
    let mut seen_keys = BTreeSet::new();
    let mut cursor = None;
    loop {
        match service
            .execute(
                tenant,
                Request::Scan {
                    exclusive_after_key: cursor.clone(),
                    limit: 64,
                },
            )
            .await
        {
            Ok(Response::Scan(page)) => {
                if page.is_empty() {
                    break;
                }
                cursor = page.last().map(|document| document.key.clone());
                for document in page {
                    actual_rows += 1;
                    seen_keys.insert(document.key.clone());
                    match expected.get(&document.key) {
                        Some(ExpectedState::Present { value, revision })
                            if *value == document.value && *revision == document.revision => {}
                        Some(ExpectedState::Present { revision, .. }) => {
                            if mismatch_samples.len() < 16 {
                                mismatch_samples.push(format!(
                                    "value or revision mismatch at {:?}: expected revision {}, received {}",
                                    document.key.encode(), revision.get(), document.revision.get()
                                ));
                            }
                        }
                        Some(ExpectedState::Missing { revision }) => {
                            if mismatch_samples.len() < 16 {
                                mismatch_samples.push(format!(
                                    "deleted key appeared at {:?}, expected revision {}",
                                    document.key.encode(),
                                    revision.get()
                                ));
                            }
                        }
                        None => {
                            if mismatch_samples.len() < 16 {
                                mismatch_samples
                                    .push(format!("unexpected key {:?}", document.key.encode()));
                            }
                        }
                    }
                }
            }
            Ok(_) => return json!({"passed": false, "error": "Scan response type mismatch"}),
            Err(error) => return json!({"passed": false, "error": error.to_string()}),
        }
    }
    for (key, state) in &expected {
        match state {
            ExpectedState::Present { .. } if !seen_keys.contains(key) => {
                missing_rows += 1;
                if mismatch_samples.len() < 16 {
                    mismatch_samples.push(format!("expected row missing at {:?}", key.encode()));
                }
            }
            ExpectedState::Missing { revision } => match service
                .execute(tenant, Request::Get { key: key.clone() })
                .await
            {
                Ok(Response::Get(RevisionState::Missing { revision: actual }))
                    if actual == *revision => {}
                Ok(Response::Get(actual)) => {
                    if mismatch_samples.len() < 16 {
                        mismatch_samples.push(format!(
                            "deleted key {:?} has unexpected state {actual:?}",
                            key.encode()
                        ));
                    }
                }
                Ok(_) => {
                    if mismatch_samples.len() < 16 {
                        mismatch_samples.push(format!(
                            "Get returned unexpected response for deleted key {:?}",
                            key.encode()
                        ));
                    }
                }
                Err(error) => {
                    if mismatch_samples.len() < 16 {
                        mismatch_samples.push(format!(
                            "Get failed for deleted key {:?}: {error}",
                            key.encode()
                        ));
                    }
                }
            },
            _ => {}
        }
    }
    let passed = actual_rows == expected_row_count
        && mismatch_samples.is_empty()
        && missing_rows == 0
        && commit_identity_unique;
    json!({
        "passed": passed,
        "expected_rows": expected_row_count,
        "actual_rows": actual_rows,
        "missing_rows": missing_rows,
        "mismatch_samples": mismatch_samples,
        "replay_order": "acknowledged commit identity reconstructs final per-key state only; it is not a read-history linearizability proof",
        "commit_identity_unique": commit_identity_unique,
        "commit_identity_entries": counters.lock().unwrap().write_ledger.len(),
        "scan_page_limit": 64,
        "scan_materialization_bound": "one response page plus bounded write ledger and mismatch sample",
    })
}

fn process_metrics() -> Value {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
        let rss_kb = status.lines().find_map(|line| {
            line.strip_prefix("VmRSS:")
                .and_then(|value| value.split_whitespace().next())
                .and_then(|value| value.parse::<u64>().ok())
        });
        let stat = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
        let fields = stat.split_whitespace().collect::<Vec<_>>();
        let cpu_ticks = if fields.len() > 15 {
            fields[13].parse::<u64>().unwrap_or(0) + fields[14].parse::<u64>().unwrap_or(0)
        } else {
            0
        };
        return json!({"rss_kb": rss_kb, "cpu_ticks_total": cpu_ticks, "sample_source": "/proc/self/status and /proc/self/stat", "complete": rss_kb.is_some()});
    }
    #[cfg(not(target_os = "linux"))]
    {
        json!({"rss_kb": null, "cpu_ticks_total": null, "sample_source": "unavailable on this host", "complete": false})
    }
}

fn directory_bytes(path: &Path) -> u64 {
    let mut total_bytes = 0_u64;
    let mut pending = vec![path.to_path_buf()];
    while let Some(current_path) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(current_path) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(entry.path());
            } else {
                total_bytes = total_bytes.saturating_add(metadata.len());
            }
        }
    }
    total_bytes
}

fn encoded_key_range(rows: usize) -> Value {
    let lengths = (0..rows.max(1))
        .map(|ordinal| key_from_ordinal(ordinal as u64).encoded_len())
        .collect::<Vec<_>>();
    json!({"minimum": lengths.iter().min(), "maximum": lengths.iter().max()})
}

async fn run(options: Options) -> Result<Value, String> {
    if options.clients == 0 || options.initial_rows == 0 || options.call_cap == 0 {
        return Err("client, row, and call cap values must be nonzero".to_owned());
    }
    if matches!(options.scenario, Scenario::Multi) && options.clients < 4 {
        return Err("mt-mix requires at least four clients".to_owned());
    }
    if options.clients > 255 {
        return Err("client count exceeds the workload key-space bound".to_owned());
    }
    if options.read_stage_metrics && !options.research_metrics {
        return Err("read-stage metrics require research metrics".to_owned());
    }
    if options.data_dir.exists()
        && std::fs::read_dir(&options.data_dir)
            .map_err(|error| error.to_string())?
            .next()
            .is_some()
    {
        return Err("data directory must be empty for an independent run".to_owned());
    }
    std::fs::create_dir_all(&options.data_dir).map_err(|error| error.to_string())?;
    let service = Arc::new(
        DriverService::new(&options, options.data_dir.clone())
            .map_err(|error| error.to_string())?,
    );
    let seed_started = Instant::now();
    let seed_ledger = seed_all(
        &service,
        options.scenario,
        options.initial_rows,
        options.value_size,
    )
    .await
    .map_err(|error| error.to_string())?;
    let seed_elapsed = seed_started.elapsed();
    service.set_research_warmup_phase();
    let assignments = assignments(options.scenario, options.clients);
    let tenant_count = if matches!(options.scenario, Scenario::Multi | Scenario::Smoke) {
        4
    } else {
        1
    };
    let warmup_counts = Arc::new(
        (0..tenant_count)
            .map(|_| {
                Mutex::new(Counters::new(
                    options.latency_storage,
                    options.max_write_ledger,
                ))
            })
            .collect::<Vec<_>>(),
    );
    if options.warmup > Duration::ZERO {
        let _ = run_clients(
            Arc::clone(&service),
            assignments.clone(),
            Arc::new(options.clone()),
            Arc::clone(&warmup_counts),
            options.warmup,
            options.call_cap,
            0,
        )
        .await;
    }
    service.set_research_measurement_phase();
    let counters = Arc::new(
        (0..tenant_count)
            .map(|_| {
                Mutex::new(Counters::new(
                    options.latency_storage,
                    options.max_write_ledger,
                ))
            })
            .collect::<Vec<_>>(),
    );
    let cpu_before = process_metrics();
    let metric_samples = Arc::new(Mutex::new(Vec::new()));
    let stop_metric_sampling = Arc::new(AtomicBool::new(false));
    let sampler_samples = Arc::clone(&metric_samples);
    let sampler_stop = Arc::clone(&stop_metric_sampling);
    let sampler = tokio::spawn(async move {
        while !sampler_stop.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_millis(200)).await;
            if sampler_stop.load(Ordering::Relaxed) {
                break;
            }
            sampler_samples.lock().unwrap().push(process_metrics());
        }
    });
    let engine_health_interval = Duration::from_millis(options.engine_health_interval_ms);
    let engine_health_sampler = if engine_health_interval.is_zero() {
        None
    } else {
        service
            .engine_health_observer()
            .map(|observer| observer.start_sampling(engine_health_interval, options.measurement))
            .transpose()?
    };
    let (admission, drain, total_calls, call_cap_reached) = run_clients(
        Arc::clone(&service),
        assignments,
        Arc::new(options.clone()),
        Arc::clone(&counters),
        options.measurement,
        options.call_cap,
        1 << 56,
    )
    .await;
    let engine_health_metrics = match engine_health_sampler {
        Some(sampler) => sampler.finish()?,
        None if engine_health_interval.is_zero() => dodb_rocksdb::engine_health_disabled_report(),
        None => dodb_rocksdb::engine_health_unsupported_report(engine_health_interval),
    };
    let research_metrics = service.research_metrics_snapshot();
    stop_metric_sampling.store(true, Ordering::Relaxed);
    let _ = sampler.await;
    let total_elapsed = admission + drain;
    let ledger_limit_reached = counters
        .iter()
        .any(|counter| counter.lock().unwrap().request_limit_reached);
    let measurement_complete =
        admission >= options.measurement && !call_cap_reached && !ledger_limit_reached;
    let termination_reason = if ledger_limit_reached {
        "write-ledger-cap"
    } else if call_cap_reached {
        "call-cap"
    } else if admission >= options.measurement {
        "requested-duration"
    } else {
        "early-stop"
    };
    let cpu_after = process_metrics();
    let sample_values = metric_samples.lock().unwrap().clone();
    let peak_rss_kb = sample_values
        .iter()
        .filter_map(|sample| sample["rss_kb"].as_u64())
        .max();
    let cpu_tick_delta = cpu_after["cpu_ticks_total"]
        .as_u64()
        .zip(cpu_before["cpu_ticks_total"].as_u64())
        .map(|(after, before)| after.saturating_sub(before));
    let clock_ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as u64;
    let online_processors = unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) }.max(1) as u64;
    let cpu_seconds = cpu_tick_delta.map(|ticks| ticks as f64 / clock_ticks_per_second as f64);
    let cpu_core_percent = cpu_seconds.map(|seconds| seconds / total_elapsed.as_secs_f64() * 100.0);
    let cpu_machine_percent = cpu_core_percent.map(|percent| percent / online_processors as f64);
    let mut validation = BTreeMap::new();
    let mut per_tenant = BTreeMap::new();
    let mut total_counters = Counters::new(options.latency_storage, options.max_write_ledger);
    let mut largest_replay_ledger_clone_bytes = 0_usize;
    for tenant_index in 0..tenant_count {
        let warmup = warmup_counts[tenant_index].lock().unwrap();
        let mut replay_counters = Counters::default();
        replay_counters
            .write_ledger
            .extend(warmup.write_ledger.clone());
        drop(warmup);
        replay_counters
            .write_ledger
            .extend(counters[tenant_index].lock().unwrap().write_ledger.clone());
        largest_replay_ledger_clone_bytes = largest_replay_ledger_clone_bytes
            .max(replay_counters.write_ledger.capacity() * std::mem::size_of::<LedgerEntry>());
        validation.insert(
            format!("tenant_{}", tenant_index + 1),
            verify_tenant(
                &service,
                TenantId::new(tenant_index as u64 + 1),
                tenant_index,
                &options,
                &seed_ledger[tenant_index],
                &Mutex::new(replay_counters),
            )
            .await,
        );
        let measured = counters[tenant_index].lock().unwrap();
        let tenant_profile = profile_for_scenario(options.scenario, tenant_index);
        per_tenant.insert(
            format!("tenant_{}", tenant_index + 1),
            json!({
                "profile": profile_name(tenant_profile),
                "schedule_fingerprint": schedule_fingerprint(tenant_profile, options.seed ^ tenant_index as u64, &options),
                "throughput_per_second": {
                    "api_calls": measured.api_calls as f64 / total_elapsed.as_secs_f64(),
                    "read_calls": measured.read_calls as f64 / total_elapsed.as_secs_f64(),
                    "successful_write_transactions": measured.successful_write_transactions as f64 / total_elapsed.as_secs_f64(),
                    "successful_mutations": measured.successful_mutations as f64 / total_elapsed.as_secs_f64(),
                },
                "counters": measured.report(),
            }),
        );
        total_counters.merge(&measured);
    }
    let disk_bytes_before_shutdown = directory_bytes(&options.data_dir);
    service.shutdown().await;
    drop(service);
    let mut reopen_options = options.clone();
    reopen_options.research_metrics = false;
    let reopened = DriverService::new(&reopen_options, options.data_dir.clone())
        .map_err(|error| error.to_string())?;
    let mut reopen_validation = BTreeMap::new();
    for tenant_index in 0..tenant_count {
        let mut replay_counters = Counters::default();
        replay_counters.write_ledger.extend(
            warmup_counts[tenant_index]
                .lock()
                .unwrap()
                .write_ledger
                .clone(),
        );
        replay_counters
            .write_ledger
            .extend(counters[tenant_index].lock().unwrap().write_ledger.clone());
        largest_replay_ledger_clone_bytes = largest_replay_ledger_clone_bytes
            .max(replay_counters.write_ledger.capacity() * std::mem::size_of::<LedgerEntry>());
        reopen_validation.insert(
            format!("tenant_{}", tenant_index + 1),
            verify_tenant(
                &reopened,
                TenantId::new(tenant_index as u64 + 1),
                tenant_index,
                &options,
                &seed_ledger[tenant_index],
                &Mutex::new(replay_counters),
            )
            .await,
        );
    }
    reopened.shutdown().await;
    drop(reopened);
    let disk_bytes_after_reopen = directory_bytes(&options.data_dir);
    let warmup_collector_capacity_bytes = warmup_counts
        .iter()
        .map(|counter| counter.lock().unwrap().collector_memory_capacity_bytes())
        .sum::<usize>();
    let measurement_collector_capacity_bytes = counters
        .iter()
        .map(|counter| counter.lock().unwrap().collector_memory_capacity_bytes())
        .sum::<usize>();
    let merged_collector_capacity_bytes = total_counters.collector_memory_capacity_bytes();
    let latency_report_sort_copy_bytes = total_counters.latency_sort_copy_capacity_bytes();
    let simultaneous_replay_ledger_clone_bytes =
        largest_replay_ledger_clone_bytes.saturating_mul(2);
    let estimated_collector_peak_bytes = warmup_collector_capacity_bytes
        + measurement_collector_capacity_bytes
        + merged_collector_capacity_bytes
        + simultaneous_replay_ledger_clone_bytes
        + latency_report_sort_copy_bytes;
    let report = json!({
        "schema": "dodb-service-workload-v1",
        "backend": match options.backend { BackendKind::Blink => "blink", BackendKind::RocksDb => "rocksdb", BackendKind::RocksDbDirect => "rocksdb-direct" },
        "scenario": match options.scenario { Scenario::Single(profile) => profile_name(profile), Scenario::Multi => "mt-mix", Scenario::Smoke => "smoke" },
        "seed": options.seed,
        "api_boundary": if options.backend == BackendKind::RocksDbDirect { "synchronous TenantDatabase core with shared Request/Response and ExecutionBudget; preopened tenant handles; same transaction and revision implementation" } else { "DodbService::execute with shared Request/Response and ExecutionBudget" },
        "configuration": {
            "clients": options.clients,
            "client_executor": match options.client_executor { ClientExecutor::Async => "async", ClientExecutor::Dedicated => "dedicated" },
            "tokio_runtime_worker_threads": 16,
            "tenant_count": tenant_count,
            "initial_rows_per_tenant": options.initial_rows,
            "value_size_bytes": options.value_size,
            "pk_raw_bytes": 8,
            "sk_raw_bytes": 8,
            "combined_raw_pk_sk_bytes": 16,
            "canonical_encoded_key_sample_bytes": key_from_ordinal(123_456).encoded_len(),
            "canonical_encoded_key_initial_dataset_min_max_bytes": encoded_key_range(options.initial_rows),
            "warmup_requested_ms": options.warmup.as_millis(),
            "measurement_requested_ms": options.measurement.as_millis(),
            "call_cap": options.call_cap,
            "mutation_workload": match options.mutation_workload { MutationWorkload::NewKey => "new-key", MutationWorkload::Overwrite => "overwrite" },
            "overwrite_slots_per_client": options.overwrite_slots_per_client,
            "maximum_overwrite_mutation_keys_per_client": options.overwrite_slots_per_client * 16,
            "latency_storage": match options.latency_storage { LatencyStorageMode::ExactAll => "exact-all", LatencyStorageMode::Log1024 => "log1024", LatencyStorageMode::Log1024Validation => "log1024-validate" },
            "max_write_ledger_entries_per_tenant": options.max_write_ledger,
            "require_full_window": options.require_full_window,
            "include_width_16": options.include_width_16,
            "query_limit": 16,
            "blink": {"database_cache_capacity_pages": 128, "coordinator_queue_capacity": 256, "coordinator_group_requests": 64, "coordinator_group_bytes": 4194304, "coordinator_collection_delay_ms": 0},
            "rocksdb": {
                "blocking_operation_limit": if options.backend != BackendKind::Blink {
                    options.blocking_admission.split_slot_limits().map_or(options.shared_blocking_slots, |(read_slots, write_slots)| read_slots + write_slots)
                } else { 0 },
                "blocking_admission": if options.backend == BackendKind::Blink {
                    "not-applicable".to_owned()
                } else {
                    match options.blocking_admission {
                        dodb_rocksdb::BlockingAdmissionMode::Shared => format!("shared-{}", options.shared_blocking_slots),
                        dodb_rocksdb::BlockingAdmissionMode::SplitReadWrite => "split-read-2-write-6".to_owned(),
                        dodb_rocksdb::BlockingAdmissionMode::SplitRead2Write8 => "split-read-2-write-8".to_owned()
                    }
                },
                "read_blocking_slots": if options.backend == BackendKind::Blink { 0 } else { options.blocking_admission.split_slot_limits().map_or(options.shared_blocking_slots, |(read_slots, _)| read_slots) },
                "write_blocking_slots": if options.backend == BackendKind::Blink { 0 } else { options.blocking_admission.split_slot_limits().map_or(options.shared_blocking_slots, |(_, write_slots)| write_slots) },
                "revision_allocator": match options.revision_allocator { dodb_rocksdb::RevisionAllocatorMode::Serialized => "serialized", dodb_rocksdb::RevisionAllocatorMode::ReservedRanges => "reserved-ranges" },
                "revision_range_size": if options.revision_allocator == dodb_rocksdb::RevisionAllocatorMode::ReservedRanges { Some(4_096) } else { None },
                "condition_key_lock_mode": if options.backend == BackendKind::Blink { "not-applicable" } else { match options.condition_key_lock_mode { dodb_rocksdb::ConditionKeyLockMode::Exclusive => "exclusive", dodb_rocksdb::ConditionKeyLockMode::Shared => "shared" } },
                "engine_health_interval_ms": options.engine_health_interval_ms
            },
            "research_metrics_enabled": options.research_metrics,
            "read_stage_metrics_enabled": options.read_stage_metrics,
            "detailed_write_metrics_enabled": options.research_metrics && options.detailed_write_metrics,
        },
        "timing": {
            "seed_setup_ms_excluded": seed_elapsed.as_secs_f64() * 1000.0,
            "warmup_count_by_tenant": warmup_counts.iter().map(|counter| counter.lock().unwrap().api_calls).collect::<Vec<_>>(),
            "admission_ms": admission.as_secs_f64() * 1000.0,
            "drain_ms": drain.as_secs_f64() * 1000.0,
            "total_elapsed_ms": total_elapsed.as_secs_f64() * 1000.0,
            "measurement_deadline_to_last_completion_ms": drain.as_secs_f64() * 1000.0,
            "throughput_denominator_seconds": total_elapsed.as_secs_f64(),
            "completed_api_calls": total_calls,
            "requested_measurement_ms": options.measurement.as_millis(),
            "actual_admission_ms": admission.as_secs_f64() * 1000.0,
            "termination_reason": termination_reason,
            "measurement_complete": measurement_complete,
            "accepted_as_full_window": !options.require_full_window || measurement_complete,
        },
        "throughput_per_second": {
            "api_calls": total_calls as f64 / total_elapsed.as_secs_f64(),
            "read_calls": total_counters.read_calls as f64 / total_elapsed.as_secs_f64(),
            "successful_write_transactions": total_counters.successful_write_transactions as f64 / total_elapsed.as_secs_f64(),
            "successful_mutations": total_counters.successful_mutations as f64 / total_elapsed.as_secs_f64(),
        },
        "counters": total_counters.report(),
        "collector_memory_phases": {
            "accounting_scope": "actual latency, write ledger, client prefix and input plan vector capacities; excludes allocator metadata, map nodes, runtime, service state, document state, expected-state BTreeMap and seen-key set",
            "warmup_capacity_bytes": warmup_collector_capacity_bytes,
            "measurement_capacity_bytes": measurement_collector_capacity_bytes,
            "merged_measurement_capacity_bytes": merged_collector_capacity_bytes,
            "largest_replay_ledger_clone_capacity_bytes": largest_replay_ledger_clone_bytes,
            "simultaneous_replay_ledger_clones": 2,
            "simultaneous_replay_ledger_clone_capacity_bytes": simultaneous_replay_ledger_clone_bytes,
            "latency_report_sort_copy_bytes": latency_report_sort_copy_bytes,
            "estimated_simultaneous_collector_peak_bytes": estimated_collector_peak_bytes,
        },
        "rocksdb_research_metrics": research_metrics,
        "per_tenant": per_tenant,
        "validation": validation,
        "reopen_validation": reopen_validation,
        "process_metrics": {"before": cpu_before, "after": cpu_after, "cpu_ticks_delta": cpu_tick_delta, "cpu_seconds": cpu_seconds, "cpu_percent_of_one_core": cpu_core_percent, "cpu_percent_of_online_machine": cpu_machine_percent, "online_processors": online_processors, "peak_sampled_rss_kb": peak_rss_kb, "rss_sample_count": sample_values.len(), "sampling_interval_ms": 200, "sampling_completeness": if sample_values.len() >= 2 && peak_rss_kb.is_some() { "complete" } else { "incomplete" }},
        "engine_health": engine_health_metrics,
        "disk_usage_bytes": {"before_shutdown": disk_bytes_before_shutdown, "after_reopen_validation": disk_bytes_after_reopen},
        "disk_usage_scope": "all regular files under this run's isolated data directory after measurement and before shutdown",
        "sync_count_and_group_metrics": "unavailable from existing service interfaces",
        "compaction_observation": "not exposed by the service comparison driver",
    });
    Ok(report)
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() {
    let options = match parse_options() {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let report_path = options.report_path.clone();
    let require_full_window = options.require_full_window;
    match run(options).await {
        Ok(report) => {
            let encoded = serde_json::to_string_pretty(&report).unwrap();
            if let Some(path) = report_path {
                if let Err(error) = std::fs::write(path, format!("{encoded}\n")) {
                    eprintln!("failed to write report: {error}");
                    std::process::exit(1);
                }
            }
            println!("{encoded}");
            if require_full_window && report["timing"]["measurement_complete"] != true {
                std::process::exit(3);
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_key_lock_mode_cli_defaults_and_accepts_only_named_modes() {
        assert_eq!(
            parse_condition_key_lock_mode(None).unwrap(),
            dodb_rocksdb::ConditionKeyLockMode::Exclusive
        );
        assert_eq!(
            parse_condition_key_lock_mode(Some("exclusive")).unwrap(),
            dodb_rocksdb::ConditionKeyLockMode::Exclusive
        );
        assert_eq!(
            parse_condition_key_lock_mode(Some("shared")).unwrap(),
            dodb_rocksdb::ConditionKeyLockMode::Shared
        );
        assert!(parse_condition_key_lock_mode(Some("upgrade")).is_err());
    }

    #[test]
    fn schedules_keep_all_categories_and_api_ratio_including_preparatory_gets() {
        for profile in [Profile::Rw95, Profile::Rw80, Profile::Rw50, Profile::Rw20] {
            let plan = make_plan(profile, 2_026_101_3, false);
            let reads = plan
                .iter()
                .filter(|entry| matches!(entry.kind, WorkflowKind::Read(_)))
                .count();
            let writes = plan
                .iter()
                .filter(|entry| matches!(entry.kind, WorkflowKind::Write { .. }))
                .count();
            let preparatory = plan
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.kind,
                        WorkflowKind::Write {
                            revision_get: true,
                            ..
                        }
                    )
                })
                .count();
            assert_eq!(reads + preparatory, profile_read_percentage(profile) * 10);
            assert_eq!(reads + preparatory + writes, 1_000);
            assert_eq!(
                plan.iter()
                    .filter(|entry| matches!(entry.kind, WorkflowKind::Read(ReadKind::Query16)))
                    .count(),
                profile_read_percentage(profile) * 2
            );
            assert_eq!(
                plan.iter()
                    .filter(|entry| matches!(entry.kind, WorkflowKind::Read(ReadKind::Get)))
                    .count()
                    + preparatory,
                profile_read_percentage(profile) * 8
            );
            let conditional = plan
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.kind,
                        WorkflowKind::Write {
                            kind: WriteKind::ConditionalOne
                                | WriteKind::TransactionFour
                                | WriteKind::TransactionEight,
                            ..
                        }
                    )
                })
                .count();
            assert_eq!(
                plan.iter()
                    .filter(|entry| matches!(
                        entry.kind,
                        WorkflowKind::Write {
                            intended_rejection: true,
                            ..
                        }
                    ))
                    .count(),
                conditional.div_ceil(10)
            );
            assert_eq!(preparatory, conditional.div_ceil(4));
            assert!(plan.iter().any(|entry| matches!(
                entry.kind,
                WorkflowKind::Write {
                    kind: WriteKind::TransactionFour,
                    ..
                }
            )));
            assert!(plan.iter().any(|entry| matches!(
                entry.kind,
                WorkflowKind::Write {
                    kind: WriteKind::TransactionEight,
                    ..
                }
            )));
            for kind in [
                WriteKind::Put,
                WriteKind::Delete,
                WriteKind::ConditionalOne,
                WriteKind::TransactionFour,
                WriteKind::TransactionEight,
            ] {
                assert!(plan.iter().any(|entry| matches!(entry.kind, WorkflowKind::Write { kind: entry_kind, .. } if entry_kind == kind)));
            }
            assert!(plan.iter().any(|entry| matches!(
                entry.kind,
                WorkflowKind::Write {
                    intended_rejection: true,
                    ..
                }
            )));
        }
    }

    #[test]
    fn width_four_and_eight_are_single_transaction_workflows() {
        let plan = make_plan(Profile::Rw50, 17, false);
        for target_width in [4, 8] {
            assert!(plan.iter().any(|entry| match entry.kind {
                WorkflowKind::Write {
                    kind: WriteKind::TransactionFour,
                    ..
                } => target_width == 4,
                WorkflowKind::Write {
                    kind: WriteKind::TransactionEight,
                    ..
                } => target_width == 8,
                _ => false,
            }));
        }
    }

    #[test]
    fn width_sixteen_is_opt_in_and_replaces_half_of_width_eight_writes() {
        let default_plan = make_plan(Profile::Rw50, 17, false);
        let expanded_plan = make_plan(Profile::Rw50, 17, true);
        let count_kind = |plan: &[WorkflowPlan], target: WriteKind| {
            plan.iter()
                .filter(|entry| matches!(entry.kind, WorkflowKind::Write { kind, .. } if kind == target))
                .count()
        };
        assert_eq!(
            count_kind(&default_plan, WriteKind::TransactionEight),
            count_kind(&expanded_plan, WriteKind::TransactionEight)
                + count_kind(&expanded_plan, WriteKind::TransactionSixteen)
        );
        assert_eq!(
            count_kind(&expanded_plan, WriteKind::TransactionEight),
            count_kind(&expanded_plan, WriteKind::TransactionSixteen)
        );
    }

    #[test]
    fn expanded_schedule_keeps_preparatory_inclusive_ratios_for_each_profile() {
        for profile in [Profile::Rw95, Profile::Rw80, Profile::Rw50, Profile::Rw20] {
            let plan = make_plan(profile, 2_026_101_3, true);
            let read_workflows = plan
                .iter()
                .filter(|entry| matches!(entry.kind, WorkflowKind::Read(_)))
                .count();
            let write_workflows = plan
                .iter()
                .filter(|entry| matches!(entry.kind, WorkflowKind::Write { .. }))
                .count();
            let preparatory_gets = plan
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.kind,
                        WorkflowKind::Write {
                            revision_get: true,
                            ..
                        }
                    )
                })
                .count();
            let conditional_workflows = plan
                .iter()
                .filter(|entry| {
                    matches!(
                        entry.kind,
                        WorkflowKind::Write {
                            kind: WriteKind::ConditionalOne
                                | WriteKind::TransactionFour
                                | WriteKind::TransactionEight
                                | WriteKind::TransactionSixteen,
                            ..
                        }
                    )
                })
                .count();
            assert_eq!(
                read_workflows + preparatory_gets,
                profile_read_percentage(profile) * 10
            );
            assert_eq!(read_workflows + preparatory_gets + write_workflows, 1_000);
            assert_eq!(preparatory_gets, conditional_workflows.div_ceil(4));
            assert_eq!(
                plan.iter()
                    .filter(|entry| { matches!(entry.kind, WorkflowKind::Read(ReadKind::Query16)) })
                    .count(),
                profile_read_percentage(profile) * 2
            );
        }
    }

    #[test]
    fn overwrite_key_slots_are_per_client_and_preserve_seeded_key_space() {
        let options = Options {
            backend: BackendKind::RocksDb,
            client_executor: ClientExecutor::Async,
            scenario: Scenario::Multi,
            data_dir: PathBuf::new(),
            seed: 1,
            warmup: Duration::ZERO,
            measurement: Duration::ZERO,
            clients: 16,
            initial_rows: 10_000,
            value_size: VALUE_SIZE,
            call_cap: 100,
            report_path: None,
            research_metrics: false,
            read_stage_metrics: false,
            detailed_write_metrics: false,
            revision_allocator: dodb_rocksdb::RevisionAllocatorMode::Serialized,
            condition_key_lock_mode: dodb_rocksdb::ConditionKeyLockMode::Exclusive,
            engine_health_interval_ms: 0,
            blocking_admission: dodb_rocksdb::BlockingAdmissionMode::Shared,
            shared_blocking_slots: 8,
            mutation_workload: MutationWorkload::Overwrite,
            overwrite_slots_per_client: 4,
            latency_storage: LatencyStorageMode::Log1024,
            max_write_ledger: 100,
            require_full_window: false,
            include_width_16: true,
        };
        let first_key = mutation_key(&options, 0, 7, 0, 3, 0, WriteKind::Delete);
        let repeated_key = mutation_key(&options, 0, 9, 0, 7, 0, WriteKind::Put);
        let other_client_key = mutation_key(&options, 0, 7, 1, 3, 0, WriteKind::Delete);
        let other_tenant_key = mutation_key(&options, 1, 7, 0, 3, 0, WriteKind::Delete);
        assert_eq!(first_key, repeated_key);
        assert_ne!(first_key, other_client_key);
        assert_ne!(first_key, other_tenant_key);
        assert!(first_key > key_from_ordinal(options.initial_rows as u64));
        let wide_key = mutation_key(&options, 0, 7, 0, 3, 15, WriteKind::TransactionSixteen);
        assert_ne!(first_key, wide_key);
    }

    #[test]
    fn width_sixteen_new_key_workflows_have_disjoint_adjacent_key_sets() {
        let options = Options {
            backend: BackendKind::RocksDb,
            client_executor: ClientExecutor::Async,
            scenario: Scenario::Single(Profile::Rw50),
            data_dir: PathBuf::new(),
            seed: 1,
            warmup: Duration::ZERO,
            measurement: Duration::ZERO,
            clients: 1,
            initial_rows: 10_000,
            value_size: VALUE_SIZE,
            call_cap: 100,
            report_path: None,
            research_metrics: false,
            read_stage_metrics: false,
            detailed_write_metrics: false,
            revision_allocator: dodb_rocksdb::RevisionAllocatorMode::Serialized,
            condition_key_lock_mode: dodb_rocksdb::ConditionKeyLockMode::Exclusive,
            engine_health_interval_ms: 0,
            blocking_admission: dodb_rocksdb::BlockingAdmissionMode::Shared,
            shared_blocking_slots: 8,
            mutation_workload: MutationWorkload::NewKey,
            overwrite_slots_per_client: 1_024,
            latency_storage: LatencyStorageMode::ExactAll,
            max_write_ledger: MAX_WRITE_LEDGER,
            require_full_window: false,
            include_width_16: true,
        };
        let first_workflow_keys = (0..16)
            .map(|mutation_index| {
                mutation_key(
                    &options,
                    0,
                    100,
                    0,
                    100,
                    mutation_index,
                    WriteKind::TransactionSixteen,
                )
            })
            .collect::<BTreeSet<_>>();
        let second_workflow_keys = (0..16)
            .map(|mutation_index| {
                mutation_key(
                    &options,
                    0,
                    101,
                    0,
                    101,
                    mutation_index,
                    WriteKind::TransactionSixteen,
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(first_workflow_keys.len(), 16);
        assert_eq!(second_workflow_keys.len(), 16);
        assert!(first_workflow_keys.is_disjoint(&second_workflow_keys));
    }

    #[test]
    fn log1024_histogram_counts_boundaries_and_merges_against_exact_samples() {
        let observations = [
            0,
            1,
            1_023,
            1_024,
            1_025,
            2_047,
            2_048,
            4_095,
            4_096,
            u64::MAX,
        ];
        let mut exact = Latencies::new(LatencyStorageMode::ExactAll);
        let mut first_histogram = Latencies::new(LatencyStorageMode::Log1024Validation);
        let mut second_histogram = Latencies::new(LatencyStorageMode::Log1024Validation);
        for (observation_index, value) in observations.into_iter().enumerate() {
            exact.record(value);
            if observation_index % 2 == 0 {
                first_histogram.record(value);
            } else {
                second_histogram.record(value);
            }
        }
        first_histogram.merge(&second_histogram);
        assert_eq!(first_histogram.count, observations.len() as u64);
        let LatencyData::Log1024 { buckets, .. } = &first_histogram.storage else {
            panic!("unexpected latency storage mode");
        };
        assert_eq!(buckets.iter().sum::<u64>(), observations.len() as u64);
        let validation =
            histogram_validation_report(&observations, buckets, observations.len() as u64);
        assert_eq!(validation["passed"], true);
        assert_eq!(
            first_histogram.report()["exact_sample_validation"]["passed"],
            true
        );
        let mut sorted = observations.to_vec();
        sorted.sort_unstable();
        for percentile in [50, 95, 99] {
            let rank = percentile_rank(sorted.len(), percentile).unwrap();
            let exact_value = sorted[rank];
            let upper_bound = histogram_percentile_upper_bound(buckets, rank).unwrap();
            assert!(upper_bound >= exact_value);
            if exact_value != u64::MAX {
                let width = upper_bound - exact_value;
                assert!(width as u128 * 1_024 <= exact_value.max(1) as u128 + 1_024);
            }
        }
        assert_eq!(histogram_bucket_index(1_023), 1_023);
        assert_eq!(histogram_bucket_index(1_024), 1_024);
        assert_eq!(
            histogram_bucket_upper_bound(histogram_bucket_index(u64::MAX)),
            Some(u64::MAX)
        );
        assert!(exact.report()["p50_ms"].is_number());
    }

    #[test]
    fn binary_keys_have_sixteen_raw_bytes_and_canonical_encoded_size_range() {
        let key = key_from_ordinal(987_654_321);
        assert_eq!(key.pk.as_bytes().len(), 8);
        assert_eq!(key.sk.as_bytes().len(), 8);
        assert_eq!(key.pk.as_bytes().len() + key.sk.as_bytes().len(), 16);
        assert!((20..=36).contains(&key.encoded_len()));
    }
}
