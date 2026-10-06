use std::collections::{BTreeMap, HashMap, HashSet};
use std::ffi::CString;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Barrier, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

use super::workload::{
    Distribution, LatencySamples, MixedValueMode, Mutation, TraceHash, WorkloadConfig,
    WorkloadGenerator, component_bytes, key_for_index, mixed_operation_is_read,
    mixed_operation_seed, mixed_trace_prefix_hash, mixed_value_bytes, seed_rows, splitmix64,
    value_bytes, writer_phase_seed,
};

pub const MAX_ATTEMPTS: u32 = 16;
pub const BACKOFF_BASE_MICROS: u64 = 100;
pub const BACKOFF_CAP_MICROS: u64 = 10_000;
pub const SEED_CHUNK_ROWS: usize = 1_000;
pub const SEEDER_WRITER_ID: usize = usize::MAX;

fn query_primary_key() -> Vec<u8> {
    component_bytes(0x33, 0, 8)
}

fn query_rows(value_size: usize, limit: usize) -> Vec<Mutation> {
    (0..limit.min(256))
        .map(|index| {
            let mut key = query_primary_key();
            key.extend_from_slice(&component_bytes(0x43, index as u64, 8));
            Mutation {
                key,
                value: vec![(index & 0xff) as u8; value_size],
            }
        })
        .collect()
}

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
    pub mixed_value_mode: MixedValueMode,
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
            mixed_value_mode: MixedValueMode::Constant,
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
                "--mixed-value-mode" => args.mixed_value_mode = MixedValueMode::parse(&value),
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
    Committed,
    Retryable(RetryKind, String),
    Failed(String),
}

pub trait Engine: Sync {
    type Writer;
    fn open_writer(&self, writer_id: usize) -> Self::Writer;
    fn attempt(&self, writer: &mut Self::Writer, mutations: &[Mutation]) -> Attempt;
    fn seed(&self, writer: &mut Self::Writer, rows: &[Mutation]);
    fn read(&self, writer: &mut Self::Writer, key: &[u8]) -> Option<Vec<u8>>;
    fn query(&self, writer: &mut Self::Writer, primary_key: &[u8], limit: usize) -> Vec<Mutation>;
    fn count_rows(&self, writer: &mut Self::Writer) -> u64;
    fn settings(&self, writer: &mut Self::Writer) -> Value;
    fn metrics(&self) -> Value;
    fn monitor_sample(&self) -> Value;
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
    Committed,
    Abandoned,
    Failed,
}

#[derive(Clone, Debug)]
struct PhaseStats {
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
    successful_read_operations: u64,
    max_attempts_used: u32,
    committed_latency: LatencySamples,
    all_latency: LatencySamples,
    read_latency: LatencySamples,
    write_latency: LatencySamples,
    messages: BTreeMap<String, u64>,
    timeline: Vec<(u64, u64, bool)>,
}

impl PhaseStats {
    fn new(seed: u64) -> Self {
        Self {
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
            successful_read_operations: 0,
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
        self.successful_read_operations += other.successful_read_operations;
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
            "successful_read_operations": self.successful_read_operations,
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
            Attempt::Committed => return Outcome::Committed,
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
}

struct WriterReport {
    writer_id: usize,
    phases: Vec<PhaseReport>,
    last_writes: HashMap<Vec<u8>, ValueExpectation>,
    ambiguous_writes: HashMap<Vec<u8>, Vec<ValueExpectation>>,
    last_committed: Vec<Vec<u8>>,
}

#[derive(Clone, Copy, Debug)]
enum ValueExpectation {
    ConstantByte(u8),
    MixedChanging {
        phase_seed: u64,
        operation_seed: u64,
        operation_index: u64,
        mutation_index: usize,
    },
}

impl ValueExpectation {
    fn from_mutation(
        args: &Args,
        phase_seed: u64,
        mixed_index: Option<u64>,
        mutation_index: usize,
        value: &[u8],
    ) -> Self {
        if args.operation == "mixed" && args.mixed_value_mode == MixedValueMode::Changing {
            if let Some(operation_index) = mixed_index {
                return Self::MixedChanging {
                    phase_seed,
                    operation_seed: mixed_operation_seed(phase_seed, operation_index),
                    operation_index,
                    mutation_index,
                };
            }
        }
        Self::ConstantByte(value.first().copied().unwrap_or_default())
    }

    fn expected_bytes(self, value_size: usize) -> Vec<u8> {
        match self {
            Self::ConstantByte(byte) => vec![byte; value_size],
            Self::MixedChanging {
                phase_seed,
                operation_seed,
                operation_index,
                mutation_index,
            } => mixed_value_bytes(
                value_size,
                phase_seed,
                operation_seed,
                operation_index,
                mutation_index,
            ),
        }
    }
}

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
    next_mixed_operation: &AtomicU64,
) -> WriterReport {
    let mut writer = engine.open_writer(writer_id);
    let mut report = WriterReport {
        writer_id,
        phases: Vec::new(),
        last_writes: HashMap::new(),
        ambiguous_writes: HashMap::new(),
        last_committed: Vec::new(),
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
            let mixed_index = (args.operation == "mixed")
                .then(|| next_mixed_operation.fetch_add(1, Ordering::Relaxed));
            let choose_read = match args.operation.as_str() {
                "get" | "query" => true,
                "mixed" => mixed_operation_is_read(
                    mixed_index.expect("mixed workload should have an operation index"),
                    args.read_percent,
                ),
                _ => false,
            };
            let mut mixed_generator = mixed_index.map(|operation_index| {
                WorkloadGenerator::new_mixed(
                    args.workload(),
                    mixed_operation_seed(phase_seed, operation_index),
                    phase_seed,
                    operation_index,
                    args.mixed_value_mode,
                )
            });
            if choose_read {
                let read_key = mixed_generator.as_mut().map_or_else(
                    || generator.next_read_key(),
                    WorkloadGenerator::next_read_key,
                );
                let rows = if args.operation == "query" {
                    Some(engine.query(&mut writer, &query_primary_key(), args.read_limit))
                } else {
                    engine.read(&mut writer, &read_key).map(|value| {
                        vec![Mutation {
                            key: read_key,
                            value,
                        }]
                    })
                };
                let finished = Instant::now();
                let elapsed = finished - started;
                stats.attempted_transactions += 1;
                stats.read_operations += 1;
                let succeeded = rows.as_ref().is_some_and(|result| {
                    if args.operation == "query" {
                        !result.is_empty()
                    } else {
                        result
                            .first()
                            .is_some_and(|row| row.value.len() == args.value_size)
                    }
                });
                if succeeded {
                    stats.successful_transactions += 1;
                    stats.successful_read_operations += 1;
                    stats.query_rows += if args.operation == "query" {
                        rows.as_ref().map_or(0, |result| result.len() as u64)
                    } else {
                        0
                    };
                    stats.committed_latency.push(elapsed);
                } else {
                    stats.failed_transactions += 1;
                    stats.errors += 1;
                }
                stats.all_latency.push(elapsed);
                stats.read_latency.push(elapsed);
                if record_timeline && !warmup {
                    stats.timeline.push((
                        (finished - phase_start).as_nanos() as u64,
                        elapsed.as_nanos() as u64,
                        succeeded,
                    ));
                }
                operation_index = operation_index.wrapping_add(1);
                continue;
            }
            let mutations = mixed_generator.as_mut().map_or_else(
                || generator.next_transaction(),
                WorkloadGenerator::next_transaction,
            );
            trace.push_transaction(
                mutations
                    .iter()
                    .map(|mutation| (mutation.key.as_slice(), mutation.value.as_slice())),
            );
            let outcome = execute_with_retry(engine, &mut writer, &mutations, &mut stats);
            let finished = Instant::now();
            let elapsed = finished - started;
            stats.attempted_transactions += 1;
            stats.all_latency.push(elapsed);
            match outcome {
                Outcome::Committed => {
                    stats.successful_transactions += 1;
                    stats.mutation_ops += mutations.len() as u64;
                    stats.committed_latency.push(elapsed);
                    stats.write_latency.push(elapsed);
                    for (mutation_index, mutation) in mutations.iter().enumerate() {
                        report.last_writes.insert(
                            mutation.key.clone(),
                            ValueExpectation::from_mutation(
                                args,
                                phase_seed,
                                mixed_index,
                                mutation_index,
                                &mutation.value,
                            ),
                        );
                    }
                    if !warmup {
                        report.last_committed = mutations
                            .iter()
                            .map(|mutation| mutation.key.clone())
                            .collect();
                    }
                }
                Outcome::Abandoned => stats.abandoned_transactions += 1,
                Outcome::Failed => {
                    stats.failed_transactions += 1;
                    for (mutation_index, mutation) in mutations.iter().enumerate() {
                        report
                            .ambiguous_writes
                            .entry(mutation.key.clone())
                            .or_default()
                            .push(ValueExpectation::from_mutation(
                                args,
                                phase_seed,
                                mixed_index,
                                mutation_index,
                                &mutation.value,
                            ));
                    }
                }
            }
            if record_timeline && !warmup {
                stats.timeline.push((
                    (finished - phase_start).as_nanos() as u64,
                    elapsed.as_nanos() as u64,
                    outcome == Outcome::Committed,
                ));
            }
            operation_index = operation_index.wrapping_add(1);
        }
        report.phases.push(PhaseReport {
            warmup,
            stats,
            trace,
            finished_at: Instant::now(),
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

fn current_git_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|| "unknown".to_owned())
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
    json!({
        "logical_cpus": std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
        "cpu_part": cpuinfo.lines().find_map(|line| line.strip_prefix("CPU part\t: ")).unwrap_or("unknown"),
        "os": os,
        "kernel": kernel.trim(),
    })
}

fn window_summary(timeline: &mut [(u64, u64, bool)], window: Duration, wall: Duration) -> Value {
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
        let mut attempted = 0u64;
        for (finish, latency, committed) in timeline.iter() {
            if *finish >= start && *finish < end {
                attempted += 1;
                if *committed {
                    committed_latencies.push(*latency);
                }
            }
        }
        committed_latencies.sort_unstable();
        let percentile = |fraction: f64| -> f64 {
            if committed_latencies.is_empty() {
                return 0.0;
            }
            let index = ((committed_latencies.len() - 1) as f64 * fraction).round() as usize;
            committed_latencies[index] as f64 / 1_000.0
        };
        let span_seconds = (end.min(wall.as_nanos() as u64).saturating_sub(start)) as f64 / 1e9;
        windows.push(json!({
            "window_index": window_index,
            "start_s": start as f64 / 1e9,
            "span_s": span_seconds,
            "attempted_transactions": attempted,
            "successful_transactions": committed_latencies.len(),
            "logical_tx_per_second": if span_seconds > 0.0 { committed_latencies.len() as f64 / span_seconds } else { 0.0 },
            "p50_us": percentile(0.50),
            "p95_us": percentile(0.95),
            "p99_us": percentile(0.99),
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
}

fn run_phases<E: Engine>(engine: &E, args: &Args) -> RunOutcome {
    let barrier = Barrier::new(args.writers + 1);
    let next_mixed_operation = AtomicU64::new(0);
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
            let next_mixed_operation = &next_mixed_operation;
            handles.push(
                std::thread::Builder::new()
                    .name(format!("writer-{writer_id}"))
                    .stack_size(8 * 1024 * 1024)
                    .spawn_scoped(scope, move || {
                        writer_thread(
                            engine,
                            args,
                            writer_id,
                            barrier,
                            schedule,
                            next_mixed_operation,
                        )
                    })
                    .expect("writer thread should spawn"),
            );
        }
        barrier.wait();
        next_mixed_operation.store(0, Ordering::Relaxed);
        {
            let start = Instant::now();
            let mut guard = schedule.lock().unwrap();
            guard.phase_start = start;
            guard.deadline = start + args.warmup;
        }
        barrier.wait();
        barrier.wait();
        next_mixed_operation.store(0, Ordering::Relaxed);
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
    let mut writer_reports = writer_reports;
    for report in writer_reports.iter_mut() {
        for phase in report.phases.drain(..) {
            traces.push(json!({
                "writer": report.writer_id,
                "phase": if phase.warmup { "warmup" } else { "measured" },
                "transactions": phase.trace.transactions,
                "hash": format!("{:016x}", phase.trace.state),
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
    }
}

fn sampled_working_set_indices(args: &Args) -> Vec<usize> {
    (0..args.working_set)
        .filter(|index| splitmix64(args.seed ^ 0x5eed_0000 ^ *index as u64) % 100 == 0)
        .collect()
}

fn verify<E: Engine>(engine: &E, args: &Args, reports: &[WriterReport]) -> Value {
    let mut candidates: HashMap<&[u8], Vec<ValueExpectation>> = HashMap::new();
    let mut ambiguous: HashMap<&[u8], Vec<ValueExpectation>> = HashMap::new();
    for report in reports {
        for (key, expectation) in &report.last_writes {
            candidates
                .entry(key.as_slice())
                .or_default()
                .push(*expectation);
        }
        for (key, expectations) in &report.ambiguous_writes {
            ambiguous
                .entry(key.as_slice())
                .or_default()
                .extend(expectations.iter().copied());
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
    let query_seed_rows = if matches!(args.operation.as_str(), "query" | "mixed") {
        args.working_set.min(256)
    } else {
        0usize
    };
    let query_seed_overlaps = (0..query_seed_rows.min(args.working_set))
        .step_by(128)
        .count() as u64;
    let expected_rows_min =
        working_set + query_seed_rows as u64 - query_seed_overlaps + committed_new_keys;
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
        let mut allowed_value_hashes: HashSet<u64> = HashSet::new();
        let committed = candidates.get(key.as_slice());
        if let Some(expectations) = committed {
            keys_with_committed_writes += 1;
            allowed_value_hashes.extend(
                expectations
                    .iter()
                    .map(|expectation| value_hash(&expectation.expected_bytes(args.value_size))),
            );
        } else if key_index(key) < working_set {
            allowed_value_hashes.insert(value_hash(&value_bytes(
                args.value_size,
                key_index(key),
                0,
            )));
        }
        let may_be_absent = committed.is_none() && key_index(key) >= working_set;
        if let Some(expectations) = ambiguous.get(key.as_slice()) {
            allowed_value_hashes.extend(
                expectations
                    .iter()
                    .map(|expectation| value_hash(&expectation.expected_bytes(args.value_size))),
            );
        }
        let valid = match &actual {
            None => may_be_absent,
            Some(value) => {
                value.len() == args.value_size && allowed_value_hashes.contains(&value_hash(value))
            }
        };
        if !valid && mismatches.len() < 20 {
            mismatches.push(json!({
                "key": hex(key),
                "actual_value_hash": actual.as_ref().map(|value| format!("{:016x}", value_hash(value))),
                "allowed_value_hashes": allowed_value_hashes.iter().map(|value| format!("{value:016x}")).collect::<Vec<_>>(),
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
        "value_hash_algorithm": "fnv1a64_v1",
        "comparison": "full_value_hash_and_length",
        "mismatches": mismatches,
        "row_count": row_count,
        "expected_rows_min": expected_rows_min,
        "expected_rows_max": expected_rows_max,
        "committed_keys_outside_working_set": committed_new_keys,
        "passed": passed,
    })
}

fn value_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |state, byte| {
        (state ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod value_verification_tests {
    use super::{ValueExpectation, value_hash};

    #[test]
    fn full_value_hash_detects_changes_after_the_first_byte() {
        let mut expected = vec![0x31; 512];
        let mut changed = expected.clone();
        changed[511] ^= 1;
        assert_eq!(expected[0], changed[0]);
        assert_ne!(value_hash(&expected), value_hash(&changed));

        expected[0] ^= 1;
        let first_operation = ValueExpectation::MixedChanging {
            phase_seed: 0x1234_5678_9abc_def0,
            operation_seed: 0x1020_3040_5060_7080,
            operation_index: 7,
            mutation_index: 0,
        }
        .expected_bytes(512);
        let next_operation = ValueExpectation::MixedChanging {
            phase_seed: 0x1234_5678_9abc_def0,
            operation_seed: 0x1020_3040_5060_7080,
            operation_index: 8,
            mutation_index: 0,
        }
        .expected_bytes(512);
        assert_eq!(first_operation[..8], 7u64.to_be_bytes());
        assert_eq!(next_operation[..8], 8u64.to_be_bytes());
        assert_ne!(value_hash(&first_operation), value_hash(&next_operation));
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
    assert!(
        args.data_dir.starts_with("/bench/zfs/db"),
        "database files must live under /bench/zfs/db"
    );
    let _ = std::fs::remove_dir_all(&args.data_dir);
    std::fs::create_dir_all(&args.data_dir).expect("data dir should be creatable");
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
    let successful_write_transactions = stats
        .successful_transactions
        .saturating_sub(stats.successful_read_operations);
    object.insert(
        "successful_write_transactions".into(),
        json!(successful_write_transactions),
    );
    object.insert(
        "successful_read_operations_per_second".into(),
        json!(stats.successful_read_operations as f64 / seconds),
    );
    object.insert(
        "write_transactions_per_second".into(),
        json!(successful_write_transactions as f64 / seconds),
    );
    object.insert(
        "mutation_ops_per_second".into(),
        json!(stats.mutation_ops as f64 / seconds),
    );
    object.insert(
        "successful_read_percent".into(),
        json!(
            stats.successful_read_operations as f64 * 100.0
                / stats.successful_transactions.max(1) as f64
        ),
    );
    object.insert(
        "successful_write_percent".into(),
        json!(
            successful_write_transactions as f64 * 100.0
                / stats.successful_transactions.max(1) as f64
        ),
    );
    object.insert(
        "attempted_read_percent".into(),
        json!(stats.read_operations as f64 * 100.0 / stats.attempted_transactions.max(1) as f64),
    );
    object.insert(
        "attempted_write_transactions".into(),
        json!(
            stats
                .attempted_transactions
                .saturating_sub(stats.read_operations)
        ),
    );
    object.insert(
        "total_operations_per_second".into(),
        json!(stats.successful_transactions as f64 / seconds),
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
        let mut seeded_rows = rows.len();
        let seed_started = Instant::now();
        for chunk in rows.chunks(SEED_CHUNK_ROWS) {
            engine.seed(&mut seeder, chunk);
        }
        if matches!(args.operation.as_str(), "query" | "mixed") {
            let rows = query_rows(args.value_size, args.working_set);
            let overlaps = (0..rows.len().min(args.working_set)).step_by(128).count();
            seeded_rows += rows.len() - overlaps;
            for chunk in rows.chunks(SEED_CHUNK_ROWS) {
                engine.seed(&mut seeder, chunk);
            }
        }
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
    let verification = verify(&reopened, args, &outcome.writer_reports);
    let reopen_close = F::close(reopened);
    let wall_seconds = outcome.wall.as_secs_f64();
    let windows = window_summary(&mut outcome.measured.timeline, args.window, outcome.wall);
    let mut record = json!({
        "record_type": "crossdb-run",
        "timestamp_unix_ms": unix_ms() as u64,
        "git_commit": current_git_commit(),
        "engine": F::engine_name(args),
        "build": F::build_info(),
        "machine": machine_info(),
        "sync_contract": "durable-return",
        "collection_policy": "not-applicable",
        "writers": args.writers,
        "client_workers": args.writers,
        "transaction_width": args.width,
        "operation": args.operation,
        "read_key_generator_version": matches!(args.operation.as_str(), "get" | "query" | "mixed")
            .then_some("key_only_preserving_schedule_v1"),
        "read_percent": args.read_percent,
        "mixed_value_mode": args.mixed_value_mode.as_str(),
        "mixed_value_generator": args.mixed_value_mode.generator_name(),
        "mixed_schedule": "global_fetch_add; operation_index_mod_100_lt_read_percent; shared_seeded_request_v2",
        "mixed_schedule_version": "shared_seeded_request_v2",
        "logical_trace_prefix_operations": 1000,
        "logical_trace_prefix_hash": format!(
            "{:016x}",
            mixed_trace_prefix_hash(
                args.workload(),
                writer_phase_seed(args.seed, false),
                args.read_percent,
                args.mixed_value_mode,
                1000,
            )
        ),
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
        "errors": outcome.measured.errors,
        "conflicts": outcome.measured.conflicts,
        "warmup": outcome.warmup.counters(),
        "cpu_seconds": outcome.cpu_seconds,
        "cpu_utilization_percent_one_core": outcome.cpu_seconds.map(|seconds| seconds / wall_seconds * 100.0),
        "rss_kib": {
            "start": rss_start,
            "after_seed": rss_after_seed,
            "end": rss_end,
            "hwm": rss_hwm,
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
    let _ = std::fs::remove_dir_all(&args.data_dir);
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
        let committed = matches!(result, Attempt::Committed);
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
    let _ = std::fs::remove_dir_all(&args.data_dir);
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
                            Outcome::Committed => {
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
    let _ = std::fs::remove_dir_all(&args.data_dir);
}
