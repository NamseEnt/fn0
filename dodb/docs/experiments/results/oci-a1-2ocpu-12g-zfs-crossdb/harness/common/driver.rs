use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::CString;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Barrier, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

use super::workload::{
    Distribution, LatencySamples, Mutation, QueryRequest, TraceHash, WorkloadConfig,
    WorkloadGenerator, key_for_index, seed_rows, splitmix64, transaction_value_bytes,
    transaction_value_identity, writer_phase_seed,
};

pub const MAX_ATTEMPTS: u32 = 16;
pub const BACKOFF_BASE_MICROS: u64 = 100;
pub const BACKOFF_CAP_MICROS: u64 = 10_000;
pub const SEED_CHUNK_ROWS: usize = 1_000;
pub const SEEDER_WRITER_ID: usize = usize::MAX;

pub fn backoff_after(failed_attempts: u32) -> Duration {
    let shift = failed_attempts.saturating_sub(1).min(16);
    Duration::from_micros((BACKOFF_BASE_MICROS << shift).min(BACKOFF_CAP_MICROS))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Bench,
    Probe,
    Atomicity,
}

#[derive(Clone, Debug)]
pub struct Args {
    pub mode: Mode,
    pub operation: String,
    pub read_percent: u8,
    pub read_limit: usize,
    pub writers: usize,
    pub width: usize,
    pub distribution: Distribution,
    pub working_set: usize,
    pub key_size: usize,
    pub value_size: usize,
    pub warmup: Duration,
    pub duration: Duration,
    pub seed: u64,
    pub scenario_index: u64,
    pub repetition: u64,
    pub data_dir: PathBuf,
    pub output: PathBuf,
    pub window: Duration,
    pub monitor_interval: Duration,
    pub probe_transactions: usize,
    pub engine_options: BTreeMap<String, String>,
}

impl Args {
    pub fn parse() -> Self {
        let mut args = Self {
            mode: Mode::Bench,
            operation: "write".to_owned(),
            read_percent: 0,
            read_limit: 16,
            writers: 16,
            width: 1,
            distribution: Distribution::Uniform,
            working_set: 100_000,
            key_size: 16,
            value_size: 64,
            warmup: Duration::from_secs(2),
            duration: Duration::from_secs(5),
            seed: 979_000_000,
            scenario_index: 0,
            repetition: 1,
            data_dir: PathBuf::from("/bench/zfs/db/crossdb"),
            output: PathBuf::from("crossdb.jsonl"),
            window: Duration::ZERO,
            monitor_interval: Duration::from_secs(1),
            probe_transactions: 200,
            engine_options: BTreeMap::new(),
        };
        let mut values = std::env::args().skip(1);
        while let Some(flag) = values.next() {
            let value = values
                .next()
                .unwrap_or_else(|| panic!("flag {flag} needs a value"));
            match flag.as_str() {
                "--mode" => {
                    args.mode = match value.as_str() {
                        "bench" => Mode::Bench,
                        "probe" => Mode::Probe,
                        "atomicity" => Mode::Atomicity,
                        other => panic!("unknown mode {other}"),
                    }
                }
                "--operation" => args.operation = value,
                "--read-percent" => args.read_percent = value.parse().expect("read percent"),
                "--read-limit" => args.read_limit = value.parse().expect("read limit"),
                "--writers" => args.writers = value.parse().expect("writers"),
                "--width" => args.width = value.parse().expect("width"),
                "--distribution" => args.distribution = Distribution::parse(&value),
                "--working-set" => args.working_set = value.parse().expect("working set"),
                "--key-size" => args.key_size = value.parse().expect("key size"),
                "--value-size" => args.value_size = value.parse().expect("value size"),
                "--warmup-ms" => {
                    args.warmup = Duration::from_millis(value.parse().expect("warmup"))
                }
                "--duration-ms" => {
                    args.duration = Duration::from_millis(value.parse().expect("duration"))
                }
                "--seed" => args.seed = value.parse().expect("seed"),
                "--scenario-index" => args.scenario_index = value.parse().expect("scenario"),
                "--repetition" => args.repetition = value.parse().expect("repetition"),
                "--data-dir" => args.data_dir = PathBuf::from(value),
                "--output" => args.output = PathBuf::from(value),
                "--window-ms" => {
                    args.window = Duration::from_millis(value.parse().expect("window"))
                }
                "--monitor-ms" => {
                    args.monitor_interval = Duration::from_millis(value.parse().expect("monitor"))
                }
                "--probe-transactions" => {
                    args.probe_transactions = value.parse().expect("probe transactions")
                }
                other => {
                    let name = other
                        .strip_prefix("--")
                        .unwrap_or_else(|| panic!("unexpected argument {other}"));
                    args.engine_options.insert(name.to_owned(), value);
                }
            }
        }
        assert_eq!(args.key_size, 16, "the shared workload uses 16-byte keys");
        assert!(matches!(
            args.operation.as_str(),
            "write" | "get" | "query" | "mixed"
        ));
        assert!(args.read_percent <= 100);
        assert!(args.read_limit > 0);
        args
    }

    pub fn option(&self, name: &str) -> Option<&str> {
        self.engine_options.get(name).map(String::as_str)
    }

    pub fn workload(&self) -> WorkloadConfig {
        WorkloadConfig {
            distribution: self.distribution,
            working_set: self.working_set,
            key_size: self.key_size,
            value_size: self.value_size,
            width: self.width,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum RetryKind {
    Busy,
    BusySnapshot,
    Conflict,
}

#[derive(Debug)]
pub enum Attempt {
    Committed(Option<u64>),
    Retryable(RetryKind, String),
    Failed(String),
}

pub trait Engine: Sync {
    type Writer;
    fn open_writer(&self, writer_id: usize) -> Self::Writer;
    fn attempt(&self, writer: &mut Self::Writer, mutations: &[Mutation]) -> Attempt;
    fn seed(&self, writer: &mut Self::Writer, rows: &[Mutation]);
    fn finish_seeding(&self) {}
    fn read(&self, writer: &mut Self::Writer, key: &[u8]) -> Option<Vec<u8>>;
    fn read_observation(&self, writer: &mut Self::Writer, key: &[u8]) -> Option<ReadObservation> {
        self.read(writer, key).map(|value| ReadObservation {
            value,
            revision: None,
        })
    }
    fn checks_revisions(&self) -> bool {
        false
    }
    fn seed_revision(&self, _key: &[u8]) -> Option<u64> {
        None
    }
    fn query(
        &self,
        writer: &mut Self::Writer,
        request: &QueryRequest,
        limit: usize,
    ) -> Vec<Mutation>;
    fn count_rows(&self, writer: &mut Self::Writer) -> u64;
    fn settings(&self, writer: &mut Self::Writer) -> Value;
    fn metrics(&self) -> Value;
    fn monitor_sample(&self) -> Value;
}

pub struct ReadObservation {
    pub value: Vec<u8>,
    pub revision: Option<u64>,
}

pub trait EngineFactory {
    type Engine: Engine;
    fn engine_name(args: &Args) -> String;
    fn build_info() -> Value;
    fn create(args: &Args) -> Self::Engine;
    fn reopen(args: &Args) -> Self::Engine;
    fn close(engine: Self::Engine) -> Value;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Outcome {
    Committed(Option<u64>),
    Abandoned,
    Failed,
}

#[derive(Clone, Debug)]
struct PhaseStats {
    attempted_reads: u64,
    successful_reads: u64,
    failed_reads: u64,
    attempted_write_transactions: u64,
    successful_write_transactions: u64,
    failed_write_transactions: u64,
    revision_checks: u64,
    revision_failures: u64,
    attempted_transactions: u64,
    successful_transactions: u64,
    abandoned_transactions: u64,
    failed_transactions: u64,
    attempts: u64,
    retries: u64,
    busy: u64,
    busy_snapshot: u64,
    conflicts: u64,
    errors: u64,
    mutation_ops: u64,
    read_operations: u64,
    query_rows: u64,
    max_attempts_used: u32,
    committed_latency: LatencySamples,
    all_latency: LatencySamples,
    read_latency: LatencySamples,
    write_latency: LatencySamples,
    messages: BTreeMap<String, u64>,
    timeline: Vec<(u64, u64, bool, bool)>,
}

impl PhaseStats {
    fn new(seed: u64) -> Self {
        Self {
            attempted_reads: 0,
            successful_reads: 0,
            failed_reads: 0,
            attempted_write_transactions: 0,
            successful_write_transactions: 0,
            failed_write_transactions: 0,
            revision_checks: 0,
            revision_failures: 0,
            attempted_transactions: 0,
            successful_transactions: 0,
            abandoned_transactions: 0,
            failed_transactions: 0,
            attempts: 0,
            retries: 0,
            busy: 0,
            busy_snapshot: 0,
            conflicts: 0,
            errors: 0,
            mutation_ops: 0,
            read_operations: 0,
            query_rows: 0,
            max_attempts_used: 0,
            committed_latency: LatencySamples::with_seed(seed),
            all_latency: LatencySamples::with_seed(seed ^ 0x1111),
            read_latency: LatencySamples::with_seed(seed ^ 0x2222),
            write_latency: LatencySamples::with_seed(seed ^ 0x3333),
            messages: BTreeMap::new(),
            timeline: Vec::new(),
        }
    }

    fn merge(&mut self, other: Self) {
        self.attempted_reads += other.attempted_reads;
        self.successful_reads += other.successful_reads;
        self.failed_reads += other.failed_reads;
        self.attempted_write_transactions += other.attempted_write_transactions;
        self.successful_write_transactions += other.successful_write_transactions;
        self.failed_write_transactions += other.failed_write_transactions;
        self.revision_checks += other.revision_checks;
        self.revision_failures += other.revision_failures;
        self.attempted_transactions += other.attempted_transactions;
        self.successful_transactions += other.successful_transactions;
        self.abandoned_transactions += other.abandoned_transactions;
        self.failed_transactions += other.failed_transactions;
        self.attempts += other.attempts;
        self.retries += other.retries;
        self.busy += other.busy;
        self.busy_snapshot += other.busy_snapshot;
        self.conflicts += other.conflicts;
        self.errors += other.errors;
        self.mutation_ops += other.mutation_ops;
        self.read_operations += other.read_operations;
        self.query_rows += other.query_rows;
        self.max_attempts_used = self.max_attempts_used.max(other.max_attempts_used);
        self.committed_latency.merge(other.committed_latency);
        self.all_latency.merge(other.all_latency);
        self.read_latency.merge(other.read_latency);
        self.write_latency.merge(other.write_latency);
        for (message, count) in other.messages {
            *self.messages.entry(message).or_default() += count;
        }
        self.timeline.extend(other.timeline);
    }

    fn counters(&self) -> Value {
        json!({
            "attempted_reads": self.attempted_reads,
            "successful_reads": self.successful_reads,
            "failed_reads": self.failed_reads,
            "attempted_write_transactions": self.attempted_write_transactions,
            "successful_write_transactions": self.successful_write_transactions,
            "failed_write_transactions": self.failed_write_transactions,
            "revision_checks": self.revision_checks,
            "revision_failures": self.revision_failures,
            "attempted_transactions": self.attempted_transactions,
            "successful_transactions": self.successful_transactions,
            "abandoned_transactions": self.abandoned_transactions,
            "failed_transactions": self.failed_transactions,
            "attempts": self.attempts,
            "retries": self.retries,
            "busy": self.busy,
            "busy_snapshot": self.busy_snapshot,
            "conflicts": self.conflicts,
            "errors": self.errors,
            "mutation_ops": self.mutation_ops,
            "read_operations": self.read_operations,
            "query_rows": self.query_rows,
            "max_attempts_used": self.max_attempts_used,
            "messages": self.messages,
        })
    }
}

fn execute_with_retry<E: Engine>(
    engine: &E,
    writer: &mut E::Writer,
    mutations: &[Mutation],
    stats: &mut PhaseStats,
) -> Outcome {
    let mut attempt_number = 0u32;
    loop {
        attempt_number += 1;
        stats.attempts += 1;
        stats.max_attempts_used = stats.max_attempts_used.max(attempt_number);
        match engine.attempt(writer, mutations) {
            Attempt::Committed(revision) => return Outcome::Committed(revision),
            Attempt::Retryable(kind, message) => {
                match kind {
                    RetryKind::Busy => stats.busy += 1,
                    RetryKind::BusySnapshot => stats.busy_snapshot += 1,
                    RetryKind::Conflict => stats.conflicts += 1,
                }
                *stats.messages.entry(message).or_default() += 1;
                if attempt_number >= MAX_ATTEMPTS {
                    return Outcome::Abandoned;
                }
                stats.retries += 1;
                std::thread::sleep(backoff_after(attempt_number));
            }
            Attempt::Failed(message) => {
                stats.errors += 1;
                *stats.messages.entry(message).or_default() += 1;
                return Outcome::Failed;
            }
        }
    }
}

struct PhaseReport {
    warmup: bool,
    stats: PhaseStats,
    trace: TraceHash,
    finished_at: Instant,
    finished_after_start: Duration,
}

struct WriterReport {
    writer_id: usize,
    phases: Vec<PhaseReport>,
    last_writes: HashMap<Vec<u8>, Vec<u8>>,
    ambiguous_writes: HashMap<Vec<u8>, Vec<Vec<u8>>>,
    last_committed: Vec<Vec<u8>>,
    read_events: Vec<ReadEvent>,
    read_events_seen: u64,
    write_events: Vec<WriteEvent>,
    write_history_overflow: bool,
}

struct ReadEvent {
    key: Vec<u8>,
    value: Vec<u8>,
    revision: Option<u64>,
    seed_revision: Option<u64>,
    started_at: Instant,
    finished_at: Instant,
}

struct WriteEvent {
    key: Vec<u8>,
    writer_id: u64,
    operation_id: u64,
    mutation_offset: u64,
    revision: Option<u64>,
    started_at: Instant,
    finished_at: Instant,
}

const MAX_READ_HISTORY_PER_CLIENT: usize = 1_024;
const MAX_WRITE_HISTORY_PER_CLIENT: usize = 65_536;

struct Schedule {
    phase_start: Instant,
    deadline: Instant,
}

fn writer_thread<E: Engine>(
    engine: &E,
    args: &Args,
    writer_id: usize,
    barrier: &Barrier,
    schedule: &Mutex<Schedule>,
) -> WriterReport {
    let mut writer = engine.open_writer(writer_id);
    let mut report = WriterReport {
        writer_id,
        phases: Vec::new(),
        last_writes: HashMap::new(),
        ambiguous_writes: HashMap::new(),
        last_committed: Vec::new(),
        read_events: Vec::new(),
        read_events_seen: 0,
        write_events: Vec::new(),
        write_history_overflow: false,
    };
    let record_timeline = !args.window.is_zero();
    barrier.wait();
    for warmup in [true, false] {
        barrier.wait();
        let (phase_start, deadline) = {
            let guard = schedule.lock().unwrap();
            (guard.phase_start, guard.deadline)
        };
        let phase_seed = writer_phase_seed(args.seed, warmup);
        let mut generator = WorkloadGenerator::new(args.workload(), phase_seed, writer_id);
        let mut stats = PhaseStats::new(phase_seed ^ writer_id as u64);
        let mut trace = TraceHash::new();
        let mut operation_index = 0u64;
        while Instant::now() < deadline {
            let started = Instant::now();
            let choose_read = match args.operation.as_str() {
                "get" | "query" => true,
                "mixed" => {
                    splitmix64(phase_seed ^ (writer_id as u64).rotate_left(17) ^ operation_index)
                        % 100
                        < u64::from(args.read_percent)
                }
                _ => false,
            };
            if choose_read {
                let engine_call_started;
                let (rows, query_request, read_revision) = if args.operation == "query" {
                    let query_request = generator.next_query(args.read_limit);
                    trace.push_query(&query_request, args.read_limit);
                    engine_call_started = Instant::now();
                    (
                        Some(engine.query(&mut writer, &query_request, args.read_limit)),
                        Some(query_request),
                        None,
                    )
                } else {
                    let generated = generator.next_transaction();
                    let key = &generated[0].key;
                    trace.push_get(key);
                    engine_call_started = Instant::now();
                    let observation = engine.read_observation(&mut writer, key);
                    let read_revision = observation.as_ref().and_then(|value| value.revision);
                    let rows = observation.map(|value| {
                        vec![Mutation {
                            key: key.clone(),
                            value: value.value,
                        }]
                    });
                    (rows, None, read_revision)
                };
                let finished = Instant::now();
                let elapsed = finished - started;
                stats.attempted_transactions += 1;
                stats.attempted_reads += 1;
                stats.read_operations += 1;
                let succeeded = rows.as_ref().is_some_and(|result| {
                    if args.operation == "query" {
                        query_request
                            .as_ref()
                            .is_some_and(|request| result == &request.expected_rows)
                    } else {
                        result
                            .first()
                            .is_some_and(|row| row.value.len() == args.value_size)
                            && (args.operation != "get"
                                || rows.as_ref().is_some_and(|result| {
                                    result.first().is_some_and(|row| {
                                        row.value
                                            == super::workload::value_bytes(
                                                args.value_size,
                                                key_index(&row.key),
                                                0,
                                            )
                                            && (!engine.checks_revisions()
                                                || read_revision == engine.seed_revision(&row.key))
                                    })
                                }))
                    }
                });
                if args.operation == "mixed" && !warmup {
                    if let Some(row) = rows.as_ref().and_then(|result| result.first()) {
                        report.read_events_seen += 1;
                        let read_event = ReadEvent {
                            key: row.key.clone(),
                            value: row.value.clone(),
                            revision: read_revision,
                            seed_revision: engine.seed_revision(&row.key),
                            started_at: engine_call_started,
                            finished_at: finished,
                        };
                        if report.read_events.len() < MAX_READ_HISTORY_PER_CLIENT {
                            report.read_events.push(read_event);
                        } else {
                            let replacement =
                                (splitmix64(phase_seed ^ report.read_events_seen.rotate_left(17))
                                    % report.read_events_seen)
                                    as usize;
                            if replacement < MAX_READ_HISTORY_PER_CLIENT {
                                report.read_events[replacement] = read_event;
                            }
                        }
                    }
                }
                if args.operation == "get" && engine.checks_revisions() {
                    stats.revision_checks += 1;
                    if !succeeded {
                        stats.revision_failures += 1;
                    }
                }
                if succeeded {
                    stats.successful_reads += 1;
                    stats.successful_transactions += 1;
                    stats.query_rows += if args.operation == "query" {
                        rows.as_ref().map_or(0, |result| result.len() as u64)
                    } else {
                        0
                    };
                    stats.committed_latency.push(elapsed);
                } else {
                    stats.failed_reads += 1;
                    stats.failed_transactions += 1;
                    stats.errors += 1;
                }
                stats.all_latency.push(elapsed);
                if succeeded {
                    stats.read_latency.push(elapsed);
                }
                if record_timeline && !warmup {
                    stats.timeline.push((
                        (finished - phase_start).as_nanos() as u64,
                        elapsed.as_nanos() as u64,
                        succeeded,
                        true,
                    ));
                }
                operation_index = operation_index.wrapping_add(1);
                continue;
            }
            let mutations = generator.next_transaction();
            trace.push_transaction(
                mutations
                    .iter()
                    .map(|mutation| (mutation.key.as_slice(), mutation.value.as_slice())),
            );
            let engine_call_started = Instant::now();
            let outcome = execute_with_retry(engine, &mut writer, &mutations, &mut stats);
            let finished = Instant::now();
            let elapsed = finished - started;
            stats.attempted_transactions += 1;
            stats.attempted_write_transactions += 1;
            stats.all_latency.push(elapsed);
            match outcome {
                Outcome::Committed(revision) => {
                    stats.successful_write_transactions += 1;
                    stats.successful_transactions += 1;
                    stats.mutation_ops += mutations.len() as u64;
                    stats.committed_latency.push(elapsed);
                    stats.write_latency.push(elapsed);
                    for mutation in &mutations {
                        report
                            .last_writes
                            .insert(mutation.key.clone(), mutation.value.clone());
                        if report.write_events.len() < MAX_WRITE_HISTORY_PER_CLIENT {
                            let (value_writer, operation_id, mutation_offset) =
                                transaction_value_identity(&mutation.value).expect(
                                    "write value should carry its generated input identity",
                                );
                            report.write_events.push(WriteEvent {
                                key: mutation.key.clone(),
                                writer_id: value_writer,
                                operation_id,
                                mutation_offset,
                                revision,
                                started_at: engine_call_started,
                                finished_at: finished,
                            });
                        } else {
                            report.write_history_overflow = true;
                        }
                    }
                    if !warmup {
                        report.last_committed = mutations
                            .iter()
                            .map(|mutation| mutation.key.clone())
                            .collect();
                    }
                }
                Outcome::Abandoned => {
                    stats.abandoned_transactions += 1;
                    stats.failed_write_transactions += 1;
                }
                Outcome::Failed => {
                    stats.failed_transactions += 1;
                    stats.failed_write_transactions += 1;
                    for mutation in &mutations {
                        report
                            .ambiguous_writes
                            .entry(mutation.key.clone())
                            .or_default()
                            .push(mutation.value.clone());
                    }
                }
            }
            if record_timeline && !warmup {
                stats.timeline.push((
                    (finished - phase_start).as_nanos() as u64,
                    elapsed.as_nanos() as u64,
                    matches!(outcome, Outcome::Committed(_)),
                    false,
                ));
            }
            operation_index = operation_index.wrapping_add(1);
        }
        report.phases.push(PhaseReport {
            warmup,
            stats,
            trace,
            finished_at: Instant::now(),
            finished_after_start: phase_start.elapsed(),
        });
        barrier.wait();
    }
    report
}

pub fn process_cpu_ticks() -> Option<u64> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let after_command = stat.rsplit_once(") ")?.1;
    let fields: Vec<_> = after_command.split_whitespace().collect();
    let user_ticks = fields.get(11)?.parse::<u64>().ok()?;
    let system_ticks = fields.get(12)?.parse::<u64>().ok()?;
    Some(user_ticks.saturating_add(system_ticks))
}

pub fn clock_ticks_per_second() -> u64 {
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks > 0 { ticks as u64 } else { 100 }
}

pub fn status_kib(field: &str) -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        line.strip_prefix(field)
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|value| value.parse().ok())
    })
}

pub fn directory_usage(path: &Path) -> (u64, u64, u64) {
    use std::os::unix::fs::MetadataExt;
    let mut apparent = 0u64;
    let mut allocated = 0u64;
    let mut files = 0u64;
    let mut pending = vec![path.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(entry.path());
            } else {
                files += 1;
                apparent += metadata.len();
                allocated += metadata.blocks() * 512;
            }
        }
    }
    (apparent, allocated, files)
}

pub fn directory_listing(path: &Path) -> Value {
    let mut listing = Map::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                listing.insert(
                    entry.file_name().to_string_lossy().into_owned(),
                    json!(metadata.len()),
                );
            }
        }
    }
    Value::Object(listing)
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or_default()
}

fn key_index(key: &[u8]) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&key[key.len() - 8..]);
    u64::from_be_bytes(bytes)
}

fn machine_info() -> Value {
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let os = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|contents| {
            contents.lines().find_map(|line| {
                line.strip_prefix("PRETTY_NAME=")
                    .map(|value| value.trim_matches('"').to_owned())
            })
        })
        .unwrap_or_default();
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    let cpu_model = cpuinfo
        .lines()
        .find_map(|line| {
            line.strip_prefix("model name\t: ")
                .or_else(|| line.strip_prefix("Hardware\t: "))
        })
        .unwrap_or("unknown");
    json!({
        "logical_cpus": std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
        "cpu_model": cpu_model,
        "cpu_part": cpuinfo.lines().find_map(|line| line.strip_prefix("CPU part\t: ")).unwrap_or("unknown"),
        "os": os,
        "kernel": kernel.trim(),
        "process_thread_count": status_value("Threads:"),
    })
}

fn status_value(prefix: &str) -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find_map(|line| line.strip_prefix(prefix)?.trim().parse().ok())
}

fn rss_summary(samples: &[Value], requested_interval: Duration, wall: Duration) -> Value {
    let mut rss_values: Vec<u64> = samples
        .iter()
        .filter_map(|sample| sample.get("rss_kib").and_then(Value::as_u64))
        .collect();
    rss_values.sort_unstable();
    let percentile = |fraction: f64| {
        if rss_values.is_empty() {
            Value::Null
        } else {
            let index = ((rss_values.len() - 1) as f64 * fraction).round() as usize;
            json!(rss_values[index])
        }
    };
    let expected_samples = if requested_interval.is_zero() {
        0
    } else {
        (wall.as_nanos() / requested_interval.as_nanos()) as usize
    };
    let mut thread_counts: Vec<u64> = samples
        .iter()
        .filter_map(|sample| sample.get("thread_count").and_then(Value::as_u64))
        .collect();
    thread_counts.sort_unstable();
    json!({
        "p50_kib": percentile(0.50),
        "p95_kib": percentile(0.95),
        "observed_max_kib": rss_values.last().copied(),
        "sample_count": rss_values.len(),
        "expected_sample_count": expected_samples,
        "collection_failures": samples.len().saturating_sub(rss_values.len()),
        "complete": !rss_values.is_empty() && samples.len() == expected_samples && rss_values.len() == samples.len(),
        "thread_count_min": thread_counts.first().copied(),
        "thread_count_max": thread_counts.last().copied(),
        "thread_count_sample_count": thread_counts.len(),
    })
}

fn window_summary(
    timeline: &mut [(u64, u64, bool, bool)],
    window: Duration,
    wall: Duration,
) -> Value {
    if window.is_zero() {
        return Value::Null;
    }
    timeline.sort_unstable();
    let window_nanos = window.as_nanos() as u64;
    let window_count = wall.as_nanos().div_ceil(window.as_nanos()) as u64;
    let mut windows = Vec::new();
    for window_index in 0..window_count {
        let start = window_index * window_nanos;
        let end = start + window_nanos;
        let mut committed_latencies: Vec<u64> = Vec::new();
        let mut read_latencies: Vec<u64> = Vec::new();
        let mut write_latencies: Vec<u64> = Vec::new();
        let mut attempted_reads = 0u64;
        let mut attempted_writes = 0u64;
        let mut successful_reads = 0u64;
        let mut successful_writes = 0u64;
        for (finish, latency, committed, is_read) in timeline.iter() {
            if *finish >= start && *finish < end {
                if *is_read {
                    attempted_reads += 1;
                } else {
                    attempted_writes += 1;
                }
                if *committed {
                    committed_latencies.push(*latency);
                    if *is_read {
                        successful_reads += 1;
                        read_latencies.push(*latency);
                    } else {
                        successful_writes += 1;
                        write_latencies.push(*latency);
                    }
                }
            }
        }
        committed_latencies.sort_unstable();
        read_latencies.sort_unstable();
        write_latencies.sort_unstable();
        let percentile = |values: &[u64], fraction: f64| -> f64 {
            if values.is_empty() {
                return 0.0;
            }
            let index = ((values.len() - 1) as f64 * fraction).round() as usize;
            values[index] as f64 / 1_000.0
        };
        let span_seconds = (end.min(wall.as_nanos() as u64).saturating_sub(start)) as f64 / 1e9;
        windows.push(json!({
            "window_index": window_index,
            "start_s": start as f64 / 1e9,
            "span_s": span_seconds,
            "attempted_reads": attempted_reads,
            "attempted_write_transactions": attempted_writes,
            "successful_reads": successful_reads,
            "successful_write_transactions": successful_writes,
            "successful_transactions": committed_latencies.len(),
            "aggregate_logical_ops_per_second": if span_seconds > 0.0 { committed_latencies.len() as f64 / span_seconds } else { 0.0 },
            "read_operations_per_second": if span_seconds > 0.0 { successful_reads as f64 / span_seconds } else { 0.0 },
            "write_transactions_per_second": if span_seconds > 0.0 { successful_writes as f64 / span_seconds } else { 0.0 },
            "read_p50_us": percentile(&read_latencies, 0.50),
            "read_p95_us": percentile(&read_latencies, 0.95),
            "read_p99_us": percentile(&read_latencies, 0.99),
            "write_p50_us": percentile(&write_latencies, 0.50),
            "write_p95_us": percentile(&write_latencies, 0.95),
            "write_p99_us": percentile(&write_latencies, 0.99),
        }));
    }
    Value::Array(windows)
}

struct RunOutcome {
    metrics_before: Value,
    measured: PhaseStats,
    warmup: PhaseStats,
    wall: Duration,
    cpu_seconds: Option<f64>,
    traces: Vec<Value>,
    writer_reports: Vec<WriterReport>,
    monitor: Vec<Value>,
    client_completion: Vec<Value>,
    completion_skew_ms: f64,
}

fn run_phases<E: Engine>(engine: &E, args: &Args) -> RunOutcome {
    let barrier = Barrier::new(args.writers + 1);
    let now = Instant::now();
    let schedule = Mutex::new(Schedule {
        phase_start: now,
        deadline: now,
    });
    let stop_monitor = AtomicBool::new(false);
    let monitor_samples = Mutex::new(Vec::new());
    let ticks_per_second = clock_ticks_per_second();
    let (writer_reports, wall, cpu_seconds, metrics_before) = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(args.writers);
        for writer_id in 0..args.writers {
            let barrier = &barrier;
            let schedule = &schedule;
            handles.push(
                std::thread::Builder::new()
                    .name(format!("writer-{writer_id}"))
                    .stack_size(8 * 1024 * 1024)
                    .spawn_scoped(scope, move || {
                        writer_thread(engine, args, writer_id, barrier, schedule)
                    })
                    .expect("writer thread should spawn"),
            );
        }
        barrier.wait();
        {
            let start = Instant::now();
            let mut guard = schedule.lock().unwrap();
            guard.phase_start = start;
            guard.deadline = start + args.warmup;
        }
        barrier.wait();
        barrier.wait();
        let metrics_before = engine.metrics();
        let cpu_start = process_cpu_ticks();
        let started = Instant::now();
        {
            let mut guard = schedule.lock().unwrap();
            guard.phase_start = started;
            guard.deadline = started + args.duration;
        }
        let monitor = if args.monitor_interval.is_zero() {
            None
        } else {
            let stop_monitor = &stop_monitor;
            let monitor_samples = &monitor_samples;
            Some(scope.spawn(move || {
                let mut next_sample = started + args.monitor_interval;
                while !stop_monitor.load(Ordering::Relaxed) {
                    let now = Instant::now();
                    if now < next_sample {
                        std::thread::sleep((next_sample - now).min(Duration::from_millis(50)));
                        continue;
                    }
                    next_sample += args.monitor_interval;
                    let (apparent, allocated, files) = directory_usage(&args.data_dir);
                    let sample = json!({
                        "t_s": started.elapsed().as_secs_f64(),
                        "rss_kib": status_kib("VmRSS:"),
                        "thread_count": status_value("Threads:"),
                        "disk_apparent_bytes": apparent,
                        "disk_allocated_bytes": allocated,
                        "disk_files": files,
                        "engine": engine.monitor_sample(),
                    });
                    monitor_samples.lock().unwrap().push(sample);
                }
            }))
        };
        barrier.wait();
        barrier.wait();
        let wall = started.elapsed();
        let cpu_end = process_cpu_ticks();
        stop_monitor.store(true, Ordering::Relaxed);
        if let Some(monitor) = monitor {
            monitor.join().expect("monitor thread should not panic");
        }
        let reports: Vec<WriterReport> = handles
            .into_iter()
            .map(|handle| handle.join().expect("writer thread should not panic"))
            .collect();
        let measured_finish = reports
            .iter()
            .filter_map(|report| report.phases.iter().find(|phase| !phase.warmup))
            .map(|phase| phase.finished_at)
            .max()
            .unwrap_or(started);
        let wall_from_writers = measured_finish.saturating_duration_since(started);
        let cpu_seconds = match (cpu_start, cpu_end) {
            (Some(start), Some(end)) if end >= start => {
                Some((end - start) as f64 / ticks_per_second as f64)
            }
            _ => None,
        };
        (
            reports,
            wall.max(wall_from_writers),
            cpu_seconds,
            metrics_before,
        )
    });
    let mut measured = PhaseStats::new(args.seed ^ 0xabcd);
    let mut warmup = PhaseStats::new(args.seed ^ 0xabce);
    let mut traces = Vec::new();
    let mut client_completion = Vec::new();
    let mut completion_times = Vec::new();
    let mut writer_reports = writer_reports;
    for report in writer_reports.iter_mut() {
        if let Some(phase) = report.phases.iter().find(|phase| !phase.warmup) {
            completion_times.push(phase.finished_after_start);
            client_completion.push(json!({
                "client": report.writer_id,
                "finished_ms": phase.finished_after_start.as_secs_f64() * 1_000.0,
                "attempted": phase.stats.attempted_transactions,
                "successful": phase.stats.successful_transactions,
                "failed": phase.stats.failed_transactions + phase.stats.abandoned_transactions,
            }));
        }
        for phase in report.phases.drain(..) {
            traces.push(json!({
                "writer": report.writer_id,
                "phase": if phase.warmup { "warmup" } else { "measured" },
                "transactions": phase.trace.transactions,
                "hash": format!("{:016x}", phase.trace.state),
                "checkpoints": phase
                    .trace
                    .checkpoints
                    .iter()
                    .map(|(transactions, hash)| {
                        json!({"transactions": transactions, "hash": format!("{hash:016x}")})
                    })
                    .collect::<Vec<_>>(),
            }));
            if phase.warmup {
                warmup.merge(phase.stats);
            } else {
                measured.merge(phase.stats);
            }
        }
    }
    RunOutcome {
        metrics_before,
        measured,
        warmup,
        wall,
        cpu_seconds,
        traces,
        writer_reports,
        monitor: monitor_samples.into_inner().unwrap(),
        completion_skew_ms: completion_times
            .iter()
            .max()
            .zip(completion_times.iter().min())
            .map_or(0.0, |(latest, earliest)| {
                latest.saturating_sub(*earliest).as_secs_f64() * 1_000.0
            }),
        client_completion,
    }
}

fn sampled_working_set_indices(args: &Args) -> Vec<usize> {
    (0..args.working_set)
        .filter(|index| splitmix64(args.seed ^ 0x5eed_0000 ^ *index as u64) % 100 == 0)
        .collect()
}

fn verify<E: Engine>(engine: &E, args: &Args, reports: &[WriterReport]) -> Value {
    let mut candidates: HashMap<&[u8], HashSet<Vec<u8>>> = HashMap::new();
    let mut ambiguous: HashMap<&[u8], HashSet<Vec<u8>>> = HashMap::new();
    for report in reports {
        for (key, value) in &report.last_writes {
            candidates
                .entry(key.as_slice())
                .or_default()
                .insert(value.clone());
        }
        for (key, values) in &report.ambiguous_writes {
            ambiguous
                .entry(key.as_slice())
                .or_default()
                .extend(values.iter().cloned());
        }
    }
    let working_set = args.working_set as u64;
    let committed_new_keys = candidates
        .keys()
        .filter(|key| key_index(key) >= working_set)
        .count() as u64;
    let ambiguous_new_keys = ambiguous
        .keys()
        .filter(|key| key_index(key) >= working_set && !candidates.contains_key(*key))
        .count() as u64;
    let expected_rows_min = working_set + committed_new_keys;
    let expected_rows_max = expected_rows_min + ambiguous_new_keys;

    let mut sampled: Vec<Vec<u8>> = sampled_working_set_indices(args)
        .into_iter()
        .map(|index| key_for_index(args.distribution, args.key_size, index))
        .collect();
    let mut sampled_seen: HashSet<Vec<u8>> = sampled.iter().cloned().collect();
    for report in reports {
        for key in &report.last_committed {
            if sampled_seen.insert(key.clone()) {
                sampled.push(key.clone());
            }
        }
    }

    let mut reader = engine.open_writer(SEEDER_WRITER_ID);
    let mut mismatches = Vec::new();
    let mut keys_with_committed_writes = 0u64;
    for key in &sampled {
        let actual = engine.read(&mut reader, key);
        let mut allowed: HashSet<Vec<u8>> = HashSet::new();
        let committed = candidates.get(key.as_slice());
        if let Some(bytes) = committed {
            keys_with_committed_writes += 1;
            allowed.extend(bytes.iter().cloned());
        } else if key_index(key) < working_set {
            allowed.insert(super::workload::value_bytes(
                args.value_size,
                key_index(key),
                0,
            ));
        }
        let may_be_absent = committed.is_none() && key_index(key) >= working_set;
        if let Some(bytes) = ambiguous.get(key.as_slice()) {
            allowed.extend(bytes.iter().cloned());
        }
        let valid = match &actual {
            None => may_be_absent,
            Some(value) => value.len() == args.value_size && allowed.contains(value),
        };
        if !valid && mismatches.len() < 20 {
            mismatches.push(json!({
                "key": hex(key),
                "actual": actual.as_ref().map(|value| hex(value)),
                "allowed_value_count": allowed.len(),
            }));
        }
        if !valid && mismatches.len() >= 20 {
            mismatches.push(json!({"truncated": true}));
            break;
        }
    }
    let row_count = engine.count_rows(&mut reader);
    drop(reader);
    let rows_ok = row_count >= expected_rows_min && row_count <= expected_rows_max;
    let passed = mismatches.is_empty() && rows_ok;
    json!({
        "sampled_keys": sampled.len(),
        "sampled_keys_with_committed_writes": keys_with_committed_writes,
        "mismatches": mismatches,
        "row_count": row_count,
        "expected_rows_min": expected_rows_min,
        "expected_rows_max": expected_rows_max,
        "committed_keys_outside_working_set": committed_new_keys,
        "passed": passed,
    })
}

fn validate_mixed_read_history(args: &Args, reports: &[WriterReport]) -> Value {
    let write_history_overflow = reports.iter().any(|report| report.write_history_overflow);
    let sampled_reads = reports
        .iter()
        .map(|report| report.read_events.len())
        .sum::<usize>();
    let observed_reads = reports
        .iter()
        .map(|report| report.read_events_seen)
        .sum::<u64>();
    let mut failed = 0u64;
    let mut indeterminate = 0u64;
    let mut failure_samples = Vec::new();
    for report in reports {
        for read in &report.read_events {
            let seed_revision = read.seed_revision.unwrap_or_default();
            let seed_value = super::workload::value_bytes(args.value_size, key_index(&read.key), 0);
            let mut latest_completed_revision = seed_revision;
            let mut latest_completed_event: Option<&WriteEvent> = None;
            for candidate in reports.iter().flat_map(|entry| entry.write_events.iter()) {
                if candidate.key != read.key || candidate.finished_at > read.started_at {
                    continue;
                }
                match (latest_completed_event, candidate.revision) {
                    (_, Some(revision)) if revision >= latest_completed_revision => {
                        latest_completed_revision = revision;
                        latest_completed_event = Some(candidate);
                    }
                    (None, None) => latest_completed_event = Some(candidate),
                    (Some(previous), None) if candidate.finished_at > previous.finished_at => {
                        latest_completed_event = Some(candidate)
                    }
                    _ => {}
                }
            }
            if let Some(actual_revision) = read.revision {
                if actual_revision < latest_completed_revision {
                    failed += 1;
                    continue;
                }
            } else if latest_completed_event.is_some() && write_history_overflow {
                indeterminate += 1;
                continue;
            }
            let seed_matches = read.value == seed_value
                && read
                    .revision
                    .map_or(latest_completed_event.is_none(), |revision| {
                        revision == seed_revision
                    });
            if seed_matches {
                continue;
            }
            let matching_write = reports
                .iter()
                .flat_map(|entry| entry.write_events.iter())
                .any(|candidate| {
                    let completed_before_read = candidate.finished_at <= read.started_at
                        && latest_completed_event
                            .is_some_and(|latest| std::ptr::eq(latest, candidate));
                    let overlapped_read = candidate.started_at < read.finished_at
                        && candidate.finished_at > read.started_at;
                    candidate.key == read.key
                        && transaction_value_identity(&read.value).is_some_and(
                            |(writer_identity, operation_id, mutation_offset)| {
                                candidate.writer_id == writer_identity
                                    && candidate.operation_id == operation_id
                                    && candidate.mutation_offset == mutation_offset
                                    && read.value
                                        == transaction_value_bytes(
                                            args.value_size,
                                            writer_identity as usize,
                                            operation_id,
                                            mutation_offset as usize,
                                        )
                            },
                        )
                        && (completed_before_read || overlapped_read)
                        && match (read.revision, candidate.revision) {
                            (Some(read_revision), Some(write_revision)) => {
                                read_revision == write_revision
                                    && write_revision >= latest_completed_revision
                            }
                            (None, None) => {
                                overlapped_read
                                    || latest_completed_event
                                        .is_none_or(|latest| std::ptr::eq(latest, candidate))
                            }
                            _ => false,
                        }
                });
            if !matching_write {
                if write_history_overflow {
                    indeterminate += 1;
                } else {
                    failed += 1;
                    if failure_samples.len() < 16 {
                        let candidates: Vec<_> = reports
                            .iter()
                            .flat_map(|entry| entry.write_events.iter())
                            .filter(|candidate| candidate.key == read.key)
                            .map(|candidate| {
                                json!({
                                    "writer": candidate.writer_id,
                                    "operation": candidate.operation_id,
                                    "offset": candidate.mutation_offset,
                                    "revision": candidate.revision,
                                    "started_before_read_finished": candidate.started_at < read.finished_at,
                                    "finished_after_read_started": candidate.finished_at > read.started_at,
                                    "completed_before_read": candidate.finished_at <= read.started_at,
                                })
                            })
                            .collect();
                        let latest_completed = latest_completed_event.map(|candidate| {
                            json!({
                                "writer": candidate.writer_id,
                                "operation": candidate.operation_id,
                                "offset": candidate.mutation_offset,
                                "revision": candidate.revision,
                            })
                        });
                        failure_samples.push(json!({
                            "key": hex(&read.key),
                            "value_identity": transaction_value_identity(&read.value),
                            "read_revision": read.revision,
                            "seed_revision": read.seed_revision,
                            "latest_completed_write": latest_completed,
                            "key_writes": candidates,
                        }));
                    }
                }
            }
        }
    }
    let passed = sampled_reads > 0 && failed == 0 && indeterminate == 0 && !write_history_overflow;
    json!({
        "status": if passed { "passed" } else if failed > 0 { "failed" } else { "inconclusive" },
        "sampled_reads": sampled_reads,
        "observed_reads_with_value": observed_reads,
        "sampled_writes": reports.iter().map(|report| report.write_events.len()).sum::<usize>(),
        "write_history_overflow": write_history_overflow,
        "read_history_limit_per_client": MAX_READ_HISTORY_PER_CLIENT,
        "write_history_limit_per_client": MAX_WRITE_HISTORY_PER_CLIENT,
        "failed": failed,
        "indeterminate": indeterminate,
        "failure_samples": failure_samples,
        "revision_policy": if reports.iter().flat_map(|report| report.read_events.iter()).any(|read| read.revision.is_some()) {
            "dodb_revision_history"
        } else {
            "value_history_without_engine_revision"
        },
        "passed": passed,
    })
}

#[cfg(test)]
mod validation_tests {
    use super::{
        Args, Distribution, MAX_READ_HISTORY_PER_CLIENT, MAX_WRITE_HISTORY_PER_CLIENT, Mode,
        ReadEvent, SEED_CHUNK_ROWS, WriteEvent, WriterReport, validate_mixed_read_history,
    };
    use crate::common::workload::{key_for_index, transaction_value_bytes, value_bytes};
    use std::collections::{BTreeMap, HashMap};
    use std::time::{Duration, Instant};

    fn args() -> Args {
        Args {
            mode: Mode::Bench,
            operation: "mixed".to_owned(),
            read_percent: 50,
            read_limit: 16,
            writers: 1,
            width: 1,
            distribution: Distribution::Uniform,
            working_set: 10_000,
            key_size: 16,
            value_size: 512,
            warmup: Duration::ZERO,
            duration: Duration::from_secs(1),
            seed: 1,
            scenario_index: 1,
            repetition: 1,
            data_dir: "/bench/zfs/db/test".into(),
            output: "/bench/zfs/db/test.jsonl".into(),
            window: Duration::ZERO,
            monitor_interval: Duration::from_secs(1),
            probe_transactions: 1,
            engine_options: BTreeMap::new(),
        }
    }

    fn report(read: ReadEvent, writes: Vec<WriteEvent>, overflow: bool) -> WriterReport {
        WriterReport {
            writer_id: 0,
            phases: Vec::new(),
            last_writes: HashMap::new(),
            ambiguous_writes: HashMap::new(),
            last_committed: Vec::new(),
            read_events: vec![read],
            read_events_seen: 1,
            write_events: writes,
            write_history_overflow: overflow,
        }
    }

    #[test]
    fn mixed_history_accepts_independent_seed_revision_and_full_value() {
        let start = Instant::now();
        let key = key_for_index(Distribution::Uniform, 16, 17);
        let read = ReadEvent {
            key: key.clone(),
            value: value_bytes(512, 17, 0),
            revision: Some(1),
            seed_revision: Some(1),
            started_at: start,
            finished_at: start + Duration::from_micros(5),
        };
        let result = validate_mixed_read_history(&args(), &[report(read, Vec::new(), false)]);
        assert_eq!(result["status"], "passed");
        assert_eq!(result["sampled_reads"], 1);
    }

    #[test]
    fn mixed_history_rejects_wrong_value_and_revision() {
        let start = Instant::now();
        let key = key_for_index(Distribution::Uniform, 16, 17);
        let read = ReadEvent {
            key,
            value: vec![0; 512],
            revision: Some(99),
            seed_revision: Some(1),
            started_at: start,
            finished_at: start + Duration::from_micros(5),
        };
        let result = validate_mixed_read_history(&args(), &[report(read, Vec::new(), false)]);
        assert_eq!(result["status"], "failed");
        assert_eq!(result["failed"], 1);
    }

    #[test]
    fn mixed_history_accepts_a_completed_write_with_its_commit_revision() {
        let started = Instant::now();
        let key = key_for_index(Distribution::Uniform, 16, 17);
        let value = transaction_value_bytes(512, 0, 41, 0);
        let write = WriteEvent {
            key: key.clone(),
            writer_id: 0,
            operation_id: 41,
            mutation_offset: 0,
            revision: Some(42),
            started_at: started,
            finished_at: started + Duration::from_micros(4),
        };
        let read = ReadEvent {
            key,
            value,
            revision: Some(42),
            seed_revision: Some(1),
            started_at: started + Duration::from_micros(5),
            finished_at: started + Duration::from_micros(8),
        };
        let result = validate_mixed_read_history(&args(), &[report(read, vec![write], false)]);
        assert_eq!(result["status"], "passed");
    }

    #[test]
    fn mixed_history_accepts_a_write_overlapping_a_read_after_an_older_write_completed() {
        let started = Instant::now();
        let key = key_for_index(Distribution::Uniform, 16, 17);
        let completed_write = WriteEvent {
            key: key.clone(),
            writer_id: 0,
            operation_id: 40,
            mutation_offset: 0,
            revision: None,
            started_at: started,
            finished_at: started + Duration::from_micros(2),
        };
        let overlapping_write = WriteEvent {
            key: key.clone(),
            writer_id: 1,
            operation_id: 41,
            mutation_offset: 0,
            revision: None,
            started_at: started + Duration::from_micros(3),
            finished_at: started + Duration::from_micros(10),
        };
        let read = ReadEvent {
            key,
            value: transaction_value_bytes(512, 1, 41, 0),
            revision: None,
            seed_revision: Some(1),
            started_at: started + Duration::from_micros(4),
            finished_at: started + Duration::from_micros(5),
        };
        let result = validate_mixed_read_history(
            &args(),
            &[report(
                read,
                vec![completed_write, overlapping_write],
                false,
            )],
        );
        assert_eq!(result["status"], "passed");
    }

    #[test]
    fn mixed_history_does_not_pass_when_write_events_are_missing() {
        let start = Instant::now();
        let key = key_for_index(Distribution::Uniform, 16, 17);
        let read = ReadEvent {
            key,
            value: value_bytes(512, 500, 0),
            revision: Some(500),
            seed_revision: Some(1),
            started_at: start,
            finished_at: start + Duration::from_micros(5),
        };
        let result = validate_mixed_read_history(&args(), &[report(read, Vec::new(), true)]);
        assert_eq!(result["status"], "inconclusive");
        assert_eq!(result["passed"], false);
    }

    #[test]
    fn history_budgets_are_explicit_and_bounded() {
        assert_eq!(MAX_READ_HISTORY_PER_CLIENT, 1_024);
        assert_eq!(MAX_WRITE_HISTORY_PER_CLIENT, 65_536);
        assert_eq!(SEED_CHUNK_ROWS, 1_000);
    }
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn append_record(path: &Path, record: &Value) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("output directory should be creatable");
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("output should open");
    writeln!(file, "{}", serde_json::to_string(record).unwrap()).expect("output write");
}

fn prepare_data_dir(args: &Args) {
    let data_root = std::env::var_os("CROSSDB_DATA_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/bench/zfs/db"));
    assert!(
        args.data_dir.starts_with(&data_root),
        "database files must live under the configured benchmark data root"
    );
    std::fs::create_dir(&args.data_dir).expect("run data directory must be new and unused");
}

fn stats_record(stats: &PhaseStats, wall: Duration) -> Value {
    let seconds = wall.as_secs_f64();
    let mut record = stats.counters();
    let object = record.as_object_mut().unwrap();
    object.insert(
        "attempted_tx_per_second".into(),
        json!(stats.attempted_transactions as f64 / seconds),
    );
    object.insert(
        "write_transactions_per_second".into(),
        json!(stats.successful_write_transactions as f64 / seconds),
    );
    object.insert(
        "aggregate_logical_ops_per_second".into(),
        json!((stats.successful_reads + stats.successful_write_transactions) as f64 / seconds),
    );
    object.insert(
        "attempts_per_second".into(),
        json!(stats.attempts as f64 / seconds),
    );
    object.insert(
        "logical_tx_per_second".into(),
        json!(stats.successful_transactions as f64 / seconds),
    );
    object.insert(
        "mutation_ops_per_second".into(),
        json!(stats.mutation_ops as f64 / seconds),
    );
    object.insert(
        "read_operations_per_second".into(),
        json!(stats.read_operations as f64 / seconds),
    );
    object.insert(
        "query_rows_per_second".into(),
        json!(stats.query_rows as f64 / seconds),
    );
    for (name, samples) in [
        ("read", &stats.read_latency),
        ("write", &stats.write_latency),
    ] {
        object.insert(format!("{name}_p50_us"), json!(samples.percentile_us(0.50)));
        object.insert(format!("{name}_p95_us"), json!(samples.percentile_us(0.95)));
        object.insert(format!("{name}_p99_us"), json!(samples.percentile_us(0.99)));
    }
    object.insert(
        "read_operations_per_second".into(),
        json!(stats.read_operations as f64 / seconds),
    );
    object.insert(
        "query_rows_per_second".into(),
        json!(stats.query_rows as f64 / seconds),
    );
    object.insert(
        "p50_us".into(),
        json!(stats.committed_latency.percentile_us(0.50)),
    );
    object.insert(
        "p95_us".into(),
        json!(stats.committed_latency.percentile_us(0.95)),
    );
    object.insert(
        "p99_us".into(),
        json!(stats.committed_latency.percentile_us(0.99)),
    );
    object.insert(
        "all_outcomes_p50_us".into(),
        json!(stats.all_latency.percentile_us(0.50)),
    );
    object.insert(
        "all_outcomes_p95_us".into(),
        json!(stats.all_latency.percentile_us(0.95)),
    );
    object.insert(
        "all_outcomes_p99_us".into(),
        json!(stats.all_latency.percentile_us(0.99)),
    );
    object.insert(
        "committed_latency_samples".into(),
        json!(stats.committed_latency.values.len()),
    );
    record
}

pub fn bench_main<F: EngineFactory>() {
    let args = Args::parse();
    match args.mode {
        Mode::Bench => run_bench::<F>(&args),
        Mode::Probe => run_probe::<F>(&args),
        Mode::Atomicity => run_atomicity::<F>(&args),
    }
}

fn run_bench<F: EngineFactory>(args: &Args) {
    prepare_data_dir(args);
    let rss_start = status_kib("VmRSS:");
    let engine = F::create(args);
    let settings = {
        let mut seeder = engine.open_writer(SEEDER_WRITER_ID);
        let settings = engine.settings(&mut seeder);
        let workload = args.workload();
        let rows: Vec<Mutation> = seed_rows(&workload).collect();
        let seeded_rows = rows.len();
        let seed_started = Instant::now();
        for chunk in rows.chunks(SEED_CHUNK_ROWS) {
            engine.seed(&mut seeder, chunk);
        }
        engine.finish_seeding();
        let seed_elapsed = seed_started.elapsed();
        json!({"effective": settings, "seed_rows": seeded_rows, "seed_elapsed_s": seed_elapsed.as_secs_f64()})
    };
    let rss_after_seed = status_kib("VmRSS:");
    let metrics_after_seed = engine.metrics();
    let mut outcome = run_phases(&engine, args);
    let metrics_after = engine.metrics();
    let rss_end = status_kib("VmRSS:");
    let rss_hwm = status_kib("VmHWM:");
    let files_before_close = directory_listing(&args.data_dir);
    let close_info = F::close(engine);
    let reopened = F::reopen(args);
    let mut verification = verify(&reopened, args, &outcome.writer_reports);
    if args.operation == "mixed" {
        let history = validate_mixed_read_history(args, &outcome.writer_reports);
        let passed = verification["passed"].as_bool().unwrap_or(false)
            && history["passed"].as_bool().unwrap_or(false);
        verification["passed"] = json!(passed);
        verification["mixed_read_history"] = history;
    }
    let reopen_close = F::close(reopened);
    let wall_seconds = outcome.wall.as_secs_f64();
    let windows = window_summary(&mut outcome.measured.timeline, args.window, outcome.wall);
    let read_validation = match args.operation.as_str() {
        "get"
            if F::engine_name(args) == "main-btree" || F::engine_name(args) == "planned-blink" =>
        {
            "Every Get checks full bytes, length, and seed commit revision recorded independently of read results"
        }
        "get" => "Every Get checks full bytes and length; engine revisions unavailable",
        "query" => {
            "Every Query checks row count, full keys, ordering, cursor boundary, and value bytes"
        }
        "mixed"
            if F::engine_name(args) == "main-btree" || F::engine_name(args) == "planned-blink" =>
        {
            "Sampled concurrent reads check full value history and dodb revisions; bounded history overflow is inconclusive"
        }
        "mixed" => {
            "Sampled concurrent reads check full value history; engine revisions unavailable"
        }
        _ => "No read validation required",
    };
    let measurement_rss = rss_summary(&outcome.monitor, args.monitor_interval, outcome.wall);
    let mut record = json!({
        "record_type": "crossdb-run",
        "timestamp_unix_ms": unix_ms() as u64,
        "engine": F::engine_name(args),
        "build": F::build_info(),
        "machine": machine_info(),
        "sync_contract": "durable-return",
        "read_validation": read_validation,
        "writers": args.writers,
        "transaction_width": args.width,
        "operation": args.operation,
        "read_percent": args.read_percent,
        "read_limit": args.read_limit,
        "distribution": args.distribution.as_str(),
        "report_distribution": args.distribution.report_name(),
        "working_set": args.working_set,
        "key_size": args.key_size,
        "value_size": args.value_size,
        "seed": args.seed,
        "scenario_index": args.scenario_index,
        "repetition": args.repetition,
        "warmup_ms": args.warmup.as_millis() as u64,
        "requested_duration_ms": args.duration.as_millis() as u64,
        "duration_ms": outcome.wall.as_millis() as u64,
        "wall_seconds": wall_seconds,
        "data_dir": args.data_dir,
        "retry_policy": {
            "max_attempts": MAX_ATTEMPTS,
            "backoff_base_us": BACKOFF_BASE_MICROS,
            "backoff_cap_us": BACKOFF_CAP_MICROS,
            "backoff": "min(base << (failed_attempt - 1), cap)",
        },
        "measured": stats_record(&outcome.measured, outcome.wall),
        "warmup": outcome.warmup.counters(),
        "cpu_seconds": outcome.cpu_seconds,
        "cpu_utilization_percent_one_core": outcome.cpu_seconds.map(|seconds| seconds / wall_seconds * 100.0),
        "rss_kib": {
            "start": rss_start,
            "after_seed": rss_after_seed,
            "end": rss_end,
            "hwm": rss_hwm,
            "measurement_window": measurement_rss,
        },
        "settings": settings,
        "metrics_after_seed": metrics_after_seed,
        "metrics_before": outcome.metrics_before,
        "metrics_after": metrics_after,
        "files_before_close": files_before_close,
        "close": close_info,
        "reopen_close": reopen_close,
        "verification": verification,
        "windows": windows,
        "monitor": outcome.monitor,
        "client_completion": outcome.client_completion,
        "client_completion_skew_ms": outcome.completion_skew_ms,
        "drain_inclusive_wall_seconds": wall_seconds,
        "traces": outcome.traces,
    });
    let logical_cpus = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    if let Some(one_core) = record["cpu_utilization_percent_one_core"].as_f64() {
        record["cpu_utilization_percent_machine"] = json!(one_core / logical_cpus as f64);
    }
    append_record(&args.output, &record);
    let measured = &record["measured"];
    println!(
        "{} w{} width{} {} rep{}: {:.1} tx/s committed, {:.1} attempted/s, busy={} conflicts={} abandoned={} errors={} p99={:.0}us verify={}",
        record["engine"].as_str().unwrap_or(""),
        args.writers,
        args.width,
        args.distribution.as_str(),
        args.repetition,
        measured["logical_tx_per_second"].as_f64().unwrap_or(0.0),
        measured["attempted_tx_per_second"].as_f64().unwrap_or(0.0),
        measured["busy"],
        measured["conflicts"],
        measured["abandoned_transactions"],
        measured["errors"],
        measured["p99_us"].as_f64().unwrap_or(0.0),
        record["verification"]["passed"],
    );
}

pub fn strace_marker(args: &Args, label: &str) {
    let path = CString::new(format!(
        "{}/.strace-marker-{label}",
        args.data_dir.display()
    ))
    .unwrap();
    unsafe {
        libc::access(path.as_ptr(), libc::F_OK);
    }
}

fn tagged_value(value_size: usize, writer_id: usize, operation: u64) -> Vec<u8> {
    let mut tag = Vec::with_capacity(8);
    tag.extend_from_slice(&(writer_id as u32).to_be_bytes());
    tag.extend_from_slice(&(operation as u32).to_be_bytes());
    tag.iter().copied().cycle().take(value_size).collect()
}

fn run_probe<F: EngineFactory>(args: &Args) {
    prepare_data_dir(args);
    let engine = F::create(args);
    let mut writer = engine.open_writer(0);
    let settings = engine.settings(&mut writer);
    let mut expected: Vec<Mutation> = Vec::new();
    let mut transactions = Vec::new();
    let widths = [1usize, 1, 1, 16];
    let mut next_index = 0usize;
    for (transaction_index, width) in widths.iter().enumerate() {
        let mutations: Vec<Mutation> = (0..*width)
            .map(|offset| {
                let key = key_for_index(Distribution::Uniform, args.key_size, next_index + offset);
                Mutation {
                    key,
                    value: tagged_value(args.value_size, 0, transaction_index as u64 + 1),
                }
            })
            .collect();
        next_index += width;
        strace_marker(args, &format!("commit-begin-{transaction_index}"));
        let result = engine.attempt(&mut writer, &mutations);
        strace_marker(args, &format!("commit-returned-{transaction_index}"));
        let committed = matches!(result, Attempt::Committed(_));
        transactions.push(json!({
            "transaction": transaction_index,
            "width": width,
            "committed": committed,
            "result": format!("{result:?}"),
        }));
        if committed {
            expected.extend(mutations);
        }
    }
    drop(writer);
    strace_marker(args, "close-begin");
    let close_info = F::close(engine);
    strace_marker(args, "reopen-begin");
    let reopened = F::reopen(args);
    let mut reader = reopened.open_writer(SEEDER_WRITER_ID);
    let mut verified = 0usize;
    let mut failures = Vec::new();
    for mutation in &expected {
        match reopened.read(&mut reader, &mutation.key) {
            Some(value) if value == mutation.value => verified += 1,
            other => failures.push(json!({
                "key": hex(&mutation.key),
                "actual": other.map(|value| hex(&value)),
            })),
        }
    }
    let row_count = reopened.count_rows(&mut reader);
    drop(reader);
    let reopen_close = F::close(reopened);
    let record = json!({
        "record_type": "durability-probe",
        "engine": F::engine_name(args),
        "build": F::build_info(),
        "settings": settings,
        "transactions": transactions,
        "expected_values": expected.len(),
        "verified_values": verified,
        "row_count": row_count,
        "failures": failures,
        "close": close_info,
        "reopen_close": reopen_close,
        "passed": failures.is_empty() && verified == expected.len() && row_count == expected.len() as u64,
    });
    append_record(&args.output, &record);
    println!("{}", serde_json::to_string_pretty(&record).unwrap());
}

fn run_atomicity<F: EngineFactory>(args: &Args) {
    prepare_data_dir(args);
    let engine = F::create(args);
    let settings = {
        let mut writer = engine.open_writer(SEEDER_WRITER_ID);
        engine.settings(&mut writer)
    };
    let keys: Vec<Vec<u8>> = (0..16)
        .map(|index| key_for_index(Distribution::Uniform, args.key_size, index))
        .collect();
    let barrier = Barrier::new(args.writers);
    let results: Vec<(PhaseStats, Vec<u64>, Vec<u64>, Vec<u64>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..args.writers)
            .map(|writer_id| {
                let engine = &engine;
                let keys = &keys;
                let barrier = &barrier;
                scope.spawn(move || {
                    let mut writer = engine.open_writer(writer_id);
                    let mut stats = PhaseStats::new(writer_id as u64);
                    let mut committed = Vec::new();
                    let mut abandoned = Vec::new();
                    let mut failed = Vec::new();
                    barrier.wait();
                    for operation in 0..args.probe_transactions as u64 {
                        let tag = ((writer_id as u64) << 32) | operation;
                        let mutations: Vec<Mutation> = keys
                            .iter()
                            .map(|key| Mutation {
                                key: key.clone(),
                                value: tagged_value(args.value_size, writer_id, operation),
                            })
                            .collect();
                        stats.attempted_transactions += 1;
                        match execute_with_retry(engine, &mut writer, &mutations, &mut stats) {
                            Outcome::Committed(_) => {
                                stats.successful_transactions += 1;
                                committed.push(tag);
                            }
                            Outcome::Abandoned => {
                                stats.abandoned_transactions += 1;
                                abandoned.push(tag);
                            }
                            Outcome::Failed => {
                                stats.failed_transactions += 1;
                                failed.push(tag);
                            }
                        }
                    }
                    (stats, committed, abandoned, failed)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("atomicity writer should not panic"))
            .collect()
    });
    let mut stats = PhaseStats::new(0);
    let mut committed: HashSet<u64> = HashSet::new();
    let mut abandoned: HashSet<u64> = HashSet::new();
    let mut failed: HashSet<u64> = HashSet::new();
    for (writer_stats, writer_committed, writer_abandoned, writer_failed) in results {
        stats.merge(writer_stats);
        committed.extend(writer_committed);
        abandoned.extend(writer_abandoned);
        failed.extend(writer_failed);
    }
    let close_info = F::close(engine);
    let reopened = F::reopen(args);
    let mut reader = reopened.open_writer(SEEDER_WRITER_ID);
    let mut tags = Vec::new();
    for key in &keys {
        let value = reopened.read(&mut reader, key);
        let tag = value.as_ref().and_then(|bytes| {
            let writer = u32::from_be_bytes(bytes[0..4].try_into().ok()?);
            let operation = u32::from_be_bytes(bytes[4..8].try_into().ok()?);
            let expected = tagged_value(args.value_size, writer as usize, operation as u64);
            (expected == *bytes).then_some(((writer as u64) << 32) | operation as u64)
        });
        tags.push(tag);
    }
    let row_count = reopened.count_rows(&mut reader);
    drop(reader);
    let reopen_close = F::close(reopened);
    let first = tags.first().copied().flatten();
    let all_same = first.is_some() && tags.iter().all(|tag| *tag == first);
    let final_tag_committed = first.is_some_and(|tag| committed.contains(&tag));
    let final_tag_abandoned = first.is_some_and(|tag| abandoned.contains(&tag));
    let passed = all_same && final_tag_committed && !final_tag_abandoned && row_count == 16;
    let record = json!({
        "record_type": "atomicity-probe",
        "engine": F::engine_name(args),
        "build": F::build_info(),
        "settings": settings,
        "writers": args.writers,
        "transactions_per_writer": args.probe_transactions,
        "counters": stats.counters(),
        "committed_transactions": committed.len(),
        "abandoned_transactions": abandoned.len(),
        "failed_transactions": failed.len(),
        "final_tags": tags.iter().map(|tag| tag.map(|value| format!("{:x}", value))).collect::<Vec<_>>(),
        "all_keys_same_transaction": all_same,
        "final_transaction_committed": final_tag_committed,
        "final_transaction_abandoned": final_tag_abandoned,
        "row_count": row_count,
        "close": close_info,
        "reopen_close": reopen_close,
        "passed": passed,
    });
    append_record(&args.output, &record);
    println!("{}", serde_json::to_string(&record).unwrap());
}
