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
const MAX_LATENCY_SAMPLES: usize = 20_000;
const MAX_WRITE_LEDGER: usize = 1_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BackendKind {
    Blink,
    RocksDb,
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

struct DriverService {
    service: Box<dyn DodbService>,
}

impl DriverService {
    fn new(backend: BackendKind, data_dir: PathBuf) -> Result<Self, Error> {
        let service: Box<dyn DodbService> = match backend {
            BackendKind::Blink => {
                let mut config = LocalTenantServiceConfig::default();
                config.data_dir = data_dir;
                config.max_open_shards = 16;
                config.database_config = DatabaseConfig::default().with_cache_capacity(128);
                config.coordinator_config = CoordinatorConfig::default();
                Box::new(BlinkTenantService::new(config)?)
            }
            BackendKind::RocksDb => {
                let mut config = dodb_rocksdb::RocksDbServiceConfig::new(data_dir);
                config.max_blocking_operations = 8;
                Box::new(dodb_rocksdb::RocksDbService::new(config)?)
            }
        };
        Ok(Self { service })
    }

    async fn execute(&self, tenant: TenantId, request: Request) -> Result<Response, Error> {
        self.service
            .execute(tenant, request, ExecutionBudget::new(RESPONSE_BUDGET))
            .await
    }

    async fn shutdown(&self) {
        self.service.shutdown().await;
    }
}

#[derive(Clone, Debug)]
struct Options {
    backend: BackendKind,
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
}

#[derive(Default)]
struct Latencies {
    count: u64,
    samples: Vec<u64>,
}

impl Latencies {
    fn record(&mut self, nanos: u64) {
        self.count += 1;
        if self.samples.len() < MAX_LATENCY_SAMPLES {
            self.samples.push(nanos);
        } else {
            let replacement = self.count.wrapping_mul(2_654_435_761) % self.count;
            if replacement < MAX_LATENCY_SAMPLES as u64 {
                self.samples[replacement as usize] = nanos;
            }
        }
    }

    fn report(&self) -> Value {
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        let percentile = |numerator: usize| -> Option<f64> {
            if sorted.is_empty() {
                return None;
            }
            let rank = ((sorted.len() - 1) * numerator).div_ceil(100);
            Some(sorted[rank] as f64 / 1_000_000.0)
        };
        json!({
            "count": self.count,
            "sample_count": sorted.len(),
            "sample_limit": MAX_LATENCY_SAMPLES,
            "p50_ms": percentile(50),
            "p95_ms": percentile(95),
            "p99_ms": percentile(99),
        })
    }
}

#[derive(Default)]
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
    verification_errors: Vec<String>,
    request_limit_reached: bool,
}

#[derive(Clone)]
struct LedgerEntry {
    commit_identity: u64,
    workflow_id: u64,
    kind: WriteKind,
}

impl Counters {
    fn latency(&mut self, name: &str, duration: Duration) {
        self.latencies
            .entry(name.to_owned())
            .or_default()
            .record(duration.as_nanos().min(u64::MAX as u128) as u64);
    }

    fn report(&self) -> Value {
        let categories = [
            "put",
            "delete",
            "conditional_width_1",
            "conditional_width_4",
            "conditional_width_8",
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
            "write_ledger_entries": self.write_ledger.len(),
            "write_ledger_limit": MAX_WRITE_LEDGER,
            "verification_errors": self.verification_errors,
            "request_limit_reached": self.request_limit_reached,
        })
    }

    fn merge(&mut self, other: Counters) {
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
        for (name, count) in other.category_attempts {
            *self.category_attempts.entry(name).or_default() += count;
        }
        for (name, count) in other.category_successes {
            *self.category_successes.entry(name).or_default() += count;
        }
        for (name, count) in other.condition_kinds {
            *self.condition_kinds.entry(name).or_default() += count;
        }
        for (name, latency) in other.latencies {
            let target = self.latencies.entry(name).or_default();
            target.count += latency.count;
            for sample in latency.samples {
                if target.samples.len() < MAX_LATENCY_SAMPLES {
                    target.samples.push(sample);
                }
            }
        }
        self.write_ledger.extend(other.write_ledger);
        self.verification_errors.extend(other.verification_errors);
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
        other => return Err(format!("unsupported backend {other}")),
    };
    let scenario = match values.get("--scenario").map(String::as_str).unwrap_or("") {
        "rw95" => Scenario::Single(Profile::Rw95),
        "rw80" => Scenario::Single(Profile::Rw80),
        "rw50" => Scenario::Single(Profile::Rw50),
        "rw20" => Scenario::Single(Profile::Rw20),
        "mt-mix" => Scenario::Multi,
        "smoke" => Scenario::Smoke,
        other => return Err(format!("unsupported scenario {other}")),
    };
    let parse = |name: &str, default: u64| -> Result<u64, String> {
        values
            .get(name)
            .map(|value| value.parse().map_err(|_| format!("invalid {name}")))
            .unwrap_or(Ok(default))
    };
    Ok(Options {
        backend,
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
    })
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
    }
}

fn condition_name(kind: usize) -> &'static str {
    match kind % 4 {
        0 => "RevisionEquals",
        1 | 2 => "Exists",
        _ => "NotExists",
    }
}

fn make_plan(profile: Profile, seed: u64) -> Vec<WorkflowPlan> {
    let read_calls = profile_read_percentage(profile) * 10;
    let write_calls = 1_000 - read_calls;
    let put_count = write_calls * 20 / 100;
    let delete_count = write_calls * 10 / 100;
    let conditional_one = write_calls * 30 / 100;
    let transaction_four = write_calls * 30 / 100;
    let transaction_eight =
        write_calls - put_count - delete_count - conditional_one - transaction_four;
    let conditional_count = conditional_one + transaction_four + transaction_eight;
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

fn schedule_fingerprint(profile: Profile, seed: u64) -> String {
    let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
    for plan in make_plan(profile, seed) {
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

fn key_from_ordinal(ordinal: u64) -> DocumentKey {
    DocumentKey::new(
        (ordinal / 10).to_be_bytes().to_vec(),
        (ordinal % 10).to_be_bytes().to_vec(),
    )
}

fn mutation_key(
    tenant_index: usize,
    workflow_id: u64,
    mutation_index: usize,
    kind: WriteKind,
) -> DocumentKey {
    if kind == WriteKind::Delete && mutation_index == 0 && workflow_id % 2 == 0 {
        return key_from_ordinal(workflow_id % 500);
    }
    let ordinal = 1_000_000_000_u64
        .wrapping_add((tenant_index as u64) << 52)
        .wrapping_add(workflow_id.wrapping_mul(8))
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
) -> Result<(), Error> {
    let mut start = 0;
    while start < rows {
        let end = (start + 64).min(rows);
        let mutations = (start..end)
            .map(|ordinal| TransactionMutation::Put {
                key: key_from_ordinal(ordinal as u64),
                value: initial_value(value_size, tenant_index, ordinal),
            })
            .collect();
        service
            .execute(
                tenant,
                Request::Transact {
                    request: TransactionRequest::new(Vec::new(), mutations),
                },
            )
            .await?;
        start = end;
    }
    Ok(())
}

async fn execute_workflow(
    service: &DriverService,
    tenant: TenantId,
    tenant_index: usize,
    plan: WorkflowPlan,
    workflow_id: u64,
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
                let result = service
                    .execute(
                        tenant,
                        Request::Get {
                            key: condition_key.clone(),
                        },
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
                    Ok(_) => state
                        .verification_errors
                        .push("preparatory Get returned a missing seeded key".to_owned()),
                    Err(error) => record_error(&mut state, error),
                }
            }
            let width = match kind {
                WriteKind::Put | WriteKind::Delete | WriteKind::ConditionalOne => 1,
                WriteKind::TransactionFour => 4,
                WriteKind::TransactionEight => 8,
            };
            let mut mutations = Vec::with_capacity(width);
            for mutation_index in 0..width {
                let key = mutation_key(tenant_index, workflow_id, mutation_index, kind);
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
                            Revision::new(revision.get().saturating_sub(1))
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
                            key: condition_key.clone(),
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
    if counters.write_ledger.len() < MAX_WRITE_LEDGER {
        counters.write_ledger.push(entry);
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
) -> (Duration, Duration, u64) {
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
        tasks.spawn(async move {
            let plan = make_plan(profile, options.seed ^ tenant_index as u64);
            let mut plan_index = client_index % plan.len();
            let mut client_sequence = 0_u64;
            while accepting.load(Ordering::Relaxed) && Instant::now() < deadline {
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
                execute_workflow(
                    &service,
                    tenant,
                    tenant_index,
                    next,
                    next_id,
                    &options,
                    &counters[tenant_index],
                )
                .await;
            }
        });
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
    (admission_elapsed, drain, calls)
}

fn assignments(scenario: Scenario, clients: usize) -> Vec<(usize, TenantId, usize, Profile)> {
    match scenario {
        Scenario::Single(profile) => (0..clients)
            .map(|client| (client, TenantId::new(1), 0, profile))
            .collect(),
        Scenario::Multi => (0..clients)
            .map(|client| {
                let tenant_index = client / 4;
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
) -> Result<(), Error> {
    let tenant_count = if matches!(scenario, Scenario::Multi | Scenario::Smoke) {
        4
    } else {
        1
    };
    for tenant_index in 0..tenant_count {
        seed_tenant(
            service,
            TenantId::new(tenant_index as u64 + 1),
            tenant_index,
            rows,
            value_size,
        )
        .await?;
    }
    Ok(())
}

async fn verify_tenant(
    service: &DriverService,
    tenant: TenantId,
    tenant_index: usize,
    options: &Options,
    counters: &Mutex<Counters>,
) -> Value {
    let mut sorted_ledger = counters.lock().unwrap().write_ledger.clone();
    sorted_ledger.sort_by_key(|entry| entry.commit_identity);
    let commit_identity_unique = sorted_ledger
        .windows(2)
        .all(|pair| pair[0].commit_identity != pair[1].commit_identity);
    let mut deleted_seed_ordinals = BTreeSet::new();
    for entry in &sorted_ledger {
        if entry.kind == WriteKind::Delete && entry.workflow_id % 2 == 0 {
            deleted_seed_ordinals.insert(entry.workflow_id % 500);
        }
    }
    let put_mutation_count = sorted_ledger
        .iter()
        .filter(|entry| entry.kind != WriteKind::Delete)
        .map(|entry| match entry.kind {
            WriteKind::Put | WriteKind::Delete | WriteKind::ConditionalOne => 1,
            WriteKind::TransactionFour => 4,
            WriteKind::TransactionEight => 8,
        })
        .sum::<usize>();
    let expected_row_count =
        options.initial_rows - deleted_seed_ordinals.len() + put_mutation_count;
    let seed_rows = (0..options.initial_rows)
        .filter(|ordinal| !deleted_seed_ordinals.contains(&(*ordinal as u64)))
        .map(|ordinal| {
            (
                key_from_ordinal(ordinal as u64),
                initial_value(options.value_size, tenant_index, ordinal),
            )
        });
    sorted_ledger.sort_by_key(|entry| entry.workflow_id);
    let dynamic_rows = sorted_ledger.into_iter().flat_map(|entry| {
        let width = match entry.kind {
            WriteKind::Put | WriteKind::Delete | WriteKind::ConditionalOne => 1,
            WriteKind::TransactionFour => 4,
            WriteKind::TransactionEight => 8,
        };
        (0..width).filter_map(move |mutation_index| {
            if entry.kind == WriteKind::Delete {
                return None;
            }
            let key = mutation_key(tenant_index, entry.workflow_id, mutation_index, entry.kind);
            Some((
                key,
                generated_value(
                    options.value_size,
                    options.seed,
                    entry.workflow_id,
                    mutation_index,
                ),
            ))
        })
    });
    let mut expected = seed_rows.chain(dynamic_rows).peekable();
    let mut actual_rows = 0_usize;
    let mut missing_rows = 0_usize;
    let mut mismatch_samples = Vec::new();
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
                    match expected.next() {
                        Some((expected_key, expected_value))
                            if expected_key == document.key && expected_value == document.value => {
                        }
                        Some((expected_key, _)) => {
                            missing_rows += 1;
                            if mismatch_samples.len() < 16 {
                                mismatch_samples.push(format!(
                                    "expected {:?}, received {:?}",
                                    expected_key.encode(),
                                    document.key.encode()
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
    while expected.next().is_some() {
        missing_rows += 1;
        if mismatch_samples.len() < 16 {
            mismatch_samples.push("expected row missing after final Scan row".to_owned());
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
        "replay_order": "commit identity for write history; ordered expected stream for Scan comparison",
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
        DriverService::new(options.backend, options.data_dir.clone())
            .map_err(|error| error.to_string())?,
    );
    let seed_started = Instant::now();
    seed_all(
        &service,
        options.scenario,
        options.initial_rows,
        options.value_size,
    )
    .await
    .map_err(|error| error.to_string())?;
    let seed_elapsed = seed_started.elapsed();
    let assignments = assignments(options.scenario, options.clients);
    let tenant_count = if matches!(options.scenario, Scenario::Multi | Scenario::Smoke) {
        4
    } else {
        1
    };
    let warmup_counts = Arc::new(
        (0..tenant_count)
            .map(|_| Mutex::new(Counters::default()))
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
    let counters = Arc::new(
        (0..tenant_count)
            .map(|_| Mutex::new(Counters::default()))
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
    let (admission, drain, total_calls) = run_clients(
        Arc::clone(&service),
        assignments,
        Arc::new(options.clone()),
        Arc::clone(&counters),
        options.measurement,
        options.call_cap,
        1 << 56,
    )
    .await;
    stop_metric_sampling.store(true, Ordering::Relaxed);
    let _ = sampler.await;
    let total_elapsed = admission + drain;
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
    let mut total_counters = Counters::default();
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
        validation.insert(
            format!("tenant_{}", tenant_index + 1),
            verify_tenant(
                &service,
                TenantId::new(tenant_index as u64 + 1),
                tenant_index,
                &options,
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
                "schedule_fingerprint": schedule_fingerprint(tenant_profile, options.seed ^ tenant_index as u64),
                "throughput_per_second": {
                    "api_calls": measured.api_calls as f64 / total_elapsed.as_secs_f64(),
                    "read_calls": measured.read_calls as f64 / total_elapsed.as_secs_f64(),
                    "successful_write_transactions": measured.successful_write_transactions as f64 / total_elapsed.as_secs_f64(),
                    "successful_mutations": measured.successful_mutations as f64 / total_elapsed.as_secs_f64(),
                },
                "counters": measured.report(),
            }),
        );
        total_counters.merge(Counters {
            api_calls: measured.api_calls,
            read_calls: measured.read_calls,
            preparatory_gets: measured.preparatory_gets,
            ordinary_gets: measured.ordinary_gets,
            query_calls: measured.query_calls,
            query_rows: measured.query_rows,
            write_requests: measured.write_requests,
            successful_write_transactions: measured.successful_write_transactions,
            successful_mutations: measured.successful_mutations,
            intended_rejections: measured.intended_rejections,
            natural_conflicts: measured.natural_conflicts,
            overloads: measured.overloads,
            other_errors: measured.other_errors,
            conditional_attempts: measured.conditional_attempts,
            category_attempts: measured.category_attempts.clone(),
            category_successes: measured.category_successes.clone(),
            condition_kinds: measured.condition_kinds.clone(),
            latencies: measured
                .latencies
                .iter()
                .map(|(name, latency)| {
                    (
                        name.clone(),
                        Latencies {
                            count: latency.count,
                            samples: latency.samples.clone(),
                        },
                    )
                })
                .collect(),
            write_ledger: measured.write_ledger.clone(),
            verification_errors: measured.verification_errors.clone(),
            request_limit_reached: measured.request_limit_reached,
        });
    }
    let disk_bytes_before_shutdown = directory_bytes(&options.data_dir);
    service.shutdown().await;
    let reopened = DriverService::new(options.backend, options.data_dir.clone())
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
        reopen_validation.insert(
            format!("tenant_{}", tenant_index + 1),
            verify_tenant(
                &reopened,
                TenantId::new(tenant_index as u64 + 1),
                tenant_index,
                &options,
                &Mutex::new(replay_counters),
            )
            .await,
        );
    }
    reopened.shutdown().await;
    let disk_bytes_after_reopen = directory_bytes(&options.data_dir);
    let report = json!({
        "schema": "dodb-service-workload-v1",
        "backend": match options.backend { BackendKind::Blink => "blink", BackendKind::RocksDb => "rocksdb" },
        "scenario": match options.scenario { Scenario::Single(profile) => profile_name(profile), Scenario::Multi => "mt-mix", Scenario::Smoke => "smoke" },
        "seed": options.seed,
        "api_boundary": "DodbService::execute with shared Request/Response and ExecutionBudget",
        "configuration": {
            "clients": options.clients,
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
            "query_limit": 16,
            "blink": {"database_cache_capacity_pages": 128, "coordinator_queue_capacity": 256, "coordinator_group_requests": 64, "coordinator_group_bytes": 4194304, "coordinator_collection_delay_ms": 0},
            "rocksdb": {"blocking_operation_limit": 8, "transaction_revision_counter": "tenant serialized"},
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
        },
        "throughput_per_second": {
            "api_calls": total_calls as f64 / total_elapsed.as_secs_f64(),
            "read_calls": total_counters.read_calls as f64 / total_elapsed.as_secs_f64(),
            "successful_write_transactions": total_counters.successful_write_transactions as f64 / total_elapsed.as_secs_f64(),
            "successful_mutations": total_counters.successful_mutations as f64 / total_elapsed.as_secs_f64(),
        },
        "counters": total_counters.report(),
        "per_tenant": per_tenant,
        "validation": validation,
        "reopen_validation": reopen_validation,
        "process_metrics": {"before": cpu_before, "after": cpu_after, "cpu_ticks_delta": cpu_tick_delta, "cpu_seconds": cpu_seconds, "cpu_percent_of_one_core": cpu_core_percent, "cpu_percent_of_online_machine": cpu_machine_percent, "online_processors": online_processors, "peak_sampled_rss_kb": peak_rss_kb, "rss_sample_count": sample_values.len(), "sampling_interval_ms": 200, "sampling_completeness": if sample_values.len() >= 2 && peak_rss_kb.is_some() { "complete" } else { "incomplete" }},
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
    fn schedules_keep_all_categories_and_api_ratio_including_preparatory_gets() {
        for profile in [Profile::Rw95, Profile::Rw80, Profile::Rw50, Profile::Rw20] {
            let plan = make_plan(profile, 2_026_101_3);
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
        let plan = make_plan(Profile::Rw50, 17);
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
    fn binary_keys_have_sixteen_raw_bytes_and_canonical_encoded_size_range() {
        let key = key_from_ordinal(987_654_321);
        assert_eq!(key.pk.as_bytes().len(), 8);
        assert_eq!(key.sk.as_bytes().len(), 8);
        assert_eq!(key.pk.as_bytes().len() + key.sk.as_bytes().len(), 16);
        assert!((20..=36).contains(&key.encoded_len()));
    }
}
