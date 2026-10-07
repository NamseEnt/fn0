//! Sustained Phase 0 baseline benchmark for the current main B+Tree.
//!
//! This binary intentionally lives beside (rather than inside) the storage
//! implementation. It uses the public AsyncShard/BTreeStore boundary and
//! existing cumulative metrics, so the production hot path does not gain
//! benchmark-only timestamps or counters.

use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write as _;
use std::future::Future;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dodb_core::{
    DocumentKey, Error, PrimaryKey, Result, RevisionState, SortKey, TransactionCondition,
    TransactionMutation, TransactionRequest, TransactionResult,
};
use dodb_storage::{
    AsyncShard, BTreeStore, BatchRequest, BatchResponse, BlinkBatchMetrics, BlinkReadHandle,
    BlinkSplitMetrics, BlinkStore, BlinkVersionedReadMetrics, CoordinatorConfig, DatabaseConfig,
    DurableFile, ProductionFile, StorageMetrics, WalMetrics, WalRedoStats,
};

#[cfg(feature = "churn-counters")]
mod churn_allocator {
    use std::alloc::{GlobalAlloc, Layout};

    struct CountingAllocator;

    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            dodb_storage::churn::record_alloc(layout.size());
            unsafe { mimalloc::MiMalloc.alloc(layout) }
        }

        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            dodb_storage::churn::record_alloc(layout.size());
            unsafe { mimalloc::MiMalloc.alloc_zeroed(layout) }
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            dodb_storage::churn::record_free(layout.size());
            unsafe { mimalloc::MiMalloc.dealloc(pointer, layout) }
        }

        unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
            dodb_storage::churn::record_realloc(new_size);
            unsafe { mimalloc::MiMalloc.realloc(pointer, layout, new_size) }
        }
    }

    #[global_allocator]
    static GLOBAL: CountingAllocator = CountingAllocator;
}

#[cfg(not(feature = "churn-counters"))]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn churn_delta(
    before: &[(
        dodb_storage::churn::ChurnSite,
        dodb_storage::churn::ChurnCounter,
        u64,
    )],
    after: &[(
        dodb_storage::churn::ChurnSite,
        dodb_storage::churn::ChurnCounter,
        u64,
    )],
) -> Vec<(String, u64)> {
    before
        .iter()
        .zip(after)
        .filter_map(|((site, counter, before_value), (_, _, after_value))| {
            let delta = after_value.saturating_sub(*before_value);
            (delta > 0).then(|| (format!("churn_{}_{}", site.name(), counter.name()), delta))
        })
        .collect()
}

fn leaf_sample_summary(samples: &[[u32; 3]]) -> Vec<(String, u64)> {
    let mut summary = vec![("leaf_samples".to_owned(), samples.len() as u64)];
    for (column, name) in ["entries", "key_bytes", "value_bytes", "payload_bytes"]
        .iter()
        .enumerate()
    {
        let mut values = samples
            .iter()
            .map(|sample| match column {
                3 => u64::from(sample[1]) + u64::from(sample[2]),
                _ => u64::from(sample[column]),
            })
            .collect::<Vec<_>>();
        if values.is_empty() {
            continue;
        }
        values.sort_unstable();
        let sum = values.iter().sum::<u64>();
        let at = |fraction: f64| values[((values.len() - 1) as f64 * fraction).round() as usize];
        summary.push((format!("leaf_{name}_sum"), sum));
        summary.push((format!("leaf_{name}_p50"), at(0.5)));
        summary.push((format!("leaf_{name}_p95"), at(0.95)));
        summary.push((format!("leaf_{name}_max"), *values.last().unwrap()));
    }
    summary
}

static CHECKPOINT_WAL_BYTES: AtomicU64 = AtomicU64::new(0);
static CHECKPOINT_EVENTS: Mutex<Vec<CheckpointEvent>> = Mutex::new(Vec::new());

#[derive(Clone, Copy, Debug)]
struct CheckpointEvent {
    finished: Instant,
    duration_nanos: u64,
    wal_bytes_before: u64,
    wal_bytes_reclaimed: u64,
}

fn checkpoint_blink_store(store: &mut BlinkStore<BenchFile, BenchFile>) -> Result<()> {
    let started = Instant::now();
    let wal_bytes_before = store.wal_metrics()?.map_or(0, |metrics| metrics.wal_bytes);
    let report = store.checkpoint()?;
    if let Ok(mut events) = CHECKPOINT_EVENTS.lock() {
        events.push(CheckpointEvent {
            finished: Instant::now(),
            duration_nanos: started.elapsed().as_nanos() as u64,
            wal_bytes_before,
            wal_bytes_reclaimed: report.wal_bytes_reclaimed,
        });
    }
    Ok(())
}

const BASELINE_COMMIT: &str = "1ff96e1b3d205074d4c1b820f5f2680bd3226a8b";
const DEFAULT_OUTPUT: &str = "target/phase0/phase0-results.jsonl";
const DEFAULT_DURATION: Duration = Duration::from_secs(2);
const DEFAULT_WARMUP: Duration = Duration::from_secs(1);
const DEFAULT_REPETITIONS: usize = 3;
const DEFAULT_WORKING_SET: usize = 4_096;
const DEFAULT_KEY_SIZE: usize = 16;
const DEFAULT_VALUE_SIZE: usize = 64;
const DEFAULT_READ_LIMIT: usize = 16;
const DEFAULT_MAX_GROUP_REQUESTS: usize = 64;
const DEFAULT_MAX_GROUP_BYTES: usize = 4 * 1024 * 1024;
const DEFAULT_QUEUE_CAPACITY: usize = 256;
const LATENCY_RESERVOIR_LIMIT: usize = 16_384;

type BenchShard = AsyncShard<BenchFile, BenchFile>;
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Suite {
    Write,
    Read,
    Mixed,
    DelaySweep,
    SyncSweep,
    All,
}

impl Suite {
    fn parse(value: &str) -> Self {
        match value {
            "write" | "write-scaling" | "core-write" => Self::Write,
            "read" | "read-scaling" | "core-read" => Self::Read,
            "mixed" | "core-mixed" => Self::Mixed,
            "delay" | "delay-sweep" => Self::DelaySweep,
            "sync" | "sync-sweep" => Self::SyncSweep,
            "all" | "core" => Self::All,
            other => panic!("unknown suite {other:?}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Write => "write-scaling",
            Self::Read => "read-scaling",
            Self::Mixed => "mixed",
            Self::DelaySweep => "delay-sweep",
            Self::SyncSweep => "sync-sweep",
            Self::All => "all",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Distribution {
    Uniform,
    Sequential,
    Hotspot,
    SameLeafHeavy,
    DifferentLeafHeavy,
}

impl Distribution {
    fn parse(value: &str) -> Self {
        match value {
            "uniform" => Self::Uniform,
            "sequential" => Self::Sequential,
            "hotspot" => Self::Hotspot,
            "same-leaf-heavy" | "same_leaf_heavy" | "same" | "compact" => Self::SameLeafHeavy,
            "different-leaf-heavy" | "different_leaf_heavy" | "different" | "spread" => {
                Self::DifferentLeafHeavy
            }
            other => panic!("unknown key distribution {other:?}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Sequential => "sequential",
            Self::Hotspot => "hotspot",
            Self::SameLeafHeavy => "same-leaf-heavy",
            Self::DifferentLeafHeavy => "different-leaf-heavy",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReadKind {
    Get,
    Query,
    Scan,
}

impl ReadKind {
    fn parse(value: &str) -> Self {
        match value {
            "get" => Self::Get,
            "query" => Self::Query,
            "scan" => Self::Scan,
            other => panic!("unknown read kind {other:?}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Query => "query",
            Self::Scan => "scan",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransactionMode {
    Unconditional,
    InsertIfAbsent,
}

impl TransactionMode {
    fn parse(value: &str) -> Self {
        match value {
            "unconditional" | "put" => Self::Unconditional,
            "insert-if-absent" | "insert_if_absent" => Self::InsertIfAbsent,
            other => panic!("unknown transaction mode {other:?}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Unconditional => "unconditional",
            Self::InsertIfAbsent => "insert-if-absent",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SyncMode {
    Real,
    Injected,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum MixedValueMode {
    #[default]
    Constant,
    Changing,
}

impl MixedValueMode {
    fn parse(value: &str) -> Self {
        match value {
            "constant" => Self::Constant,
            "changing" => Self::Changing,
            other => panic!("unknown mixed value mode {other:?}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Constant => "constant",
            Self::Changing => "changing",
        }
    }

    fn generator_name(self) -> &'static str {
        match self {
            Self::Constant => "legacy_constant_v1",
            Self::Changing => "seeded_nonrepeating_v1",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum CollectionPolicy {
    #[default]
    Current,
    MainParity,
}

impl CollectionPolicy {
    fn parse(value: &str) -> Self {
        match value {
            "current" => Self::Current,
            "main-parity" => Self::MainParity,
            other => panic!("unknown Blink collection policy {other:?}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::MainParity => "main-parity",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EngineKind {
    MainBtree,
    SerialBlink,
    VersionedBlink,
    PlannedBlink,
    ParallelBlink,
    LogicalOverlayBlink,
    BackgroundOverlayBlink,
}

impl EngineKind {
    fn parse(value: &str) -> Self {
        match value {
            "main-btree" | "btree" | "main" => Self::MainBtree,
            "serial-blink" | "blink" => Self::SerialBlink,
            "versioned-blink" | "blink-versioned" | "phase2" => Self::VersionedBlink,
            "planned-blink" | "blink-planned" | "phase3" => Self::PlannedBlink,
            "parallel-blink" | "blink-parallel" | "phase4" => Self::ParallelBlink,
            "logical-overlay-blink" | "blink-logical" | "phase-j" | "synchronous-overlay-blink" => {
                Self::LogicalOverlayBlink
            }
            "background-overlay-blink" | "phase-k" => Self::BackgroundOverlayBlink,
            other => panic!("unknown engine {other:?}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::MainBtree => "main-btree",
            Self::SerialBlink => "serial-blink",
            Self::VersionedBlink => "versioned-blink",
            Self::PlannedBlink => "planned-blink",
            Self::ParallelBlink => "parallel-blink",
            Self::LogicalOverlayBlink => "logical-overlay-blink",
            Self::BackgroundOverlayBlink => "background-overlay-blink",
        }
    }
}

impl SyncMode {
    fn parse(value: &str) -> Self {
        match value {
            "real" => Self::Real,
            "delay" | "injected" => Self::Injected,
            "disabled" | "none" | "no-sync" => Self::Disabled,
            other => panic!("unknown sync mode {other:?}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Real => "real",
            Self::Injected => "injected",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Role {
    Reader,
    Writer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Mix {
    read_percent: u8,
}

impl Mix {
    const READ_HEAVY: Self = Self { read_percent: 95 };
    const BALANCED: Self = Self { read_percent: 50 };
    const WRITE_HEAVY: Self = Self { read_percent: 20 };

    fn parse(value: &str) -> Self {
        match value {
            "95/5" | "95-5" | "read-heavy" => Self::READ_HEAVY,
            "50/50" | "50-50" | "balanced" => Self::BALANCED,
            "write-heavy" | "20/80" | "20-80" => Self::WRITE_HEAVY,
            other => panic!("unknown operation mix {other:?}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self.read_percent {
            95 => "95/5",
            50 => "50/50",
            20 => "20/80-write-heavy",
            _ => "custom",
        }
    }
}

#[derive(Clone, Debug)]
struct Args {
    engine: EngineKind,
    suite: Suite,
    writers: Option<Vec<usize>>,
    readers: Option<Vec<usize>>,
    widths: Option<Vec<usize>>,
    distributions: Option<Vec<Distribution>>,
    read_kinds: Option<Vec<ReadKind>>,
    mixes: Option<Vec<Mix>>,
    mixed_clients: bool,
    main_compact_wal: bool,
    mixed_value_mode: MixedValueMode,
    collection_policy: CollectionPolicy,
    duration: Duration,
    warmup: Duration,
    repetitions: usize,
    cache_capacity: usize,
    working_set: usize,
    key_size: usize,
    value_size: usize,
    read_limit: usize,
    max_group_requests: usize,
    max_group_bytes: usize,
    queue_capacity: usize,
    collection_delay: Option<Duration>,
    sync_mode: SyncMode,
    sync_delay: Duration,
    transaction_mode: TransactionMode,
    tokio_workers: usize,
    blink_workers: usize,
    parallel_workers: usize,
    parallel_min_mutations: usize,
    parallel_background_min_operations: usize,
    seed: u64,
    output: PathBuf,
    window_seconds: Option<u64>,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            engine: EngineKind::MainBtree,
            suite: Suite::Write,
            writers: None,
            readers: None,
            widths: None,
            distributions: None,
            read_kinds: None,
            mixes: None,
            mixed_clients: false,
            main_compact_wal: false,
            mixed_value_mode: MixedValueMode::Constant,
            collection_policy: CollectionPolicy::Current,
            duration: DEFAULT_DURATION,
            warmup: DEFAULT_WARMUP,
            repetitions: DEFAULT_REPETITIONS,
            cache_capacity: 256,
            working_set: DEFAULT_WORKING_SET,
            key_size: DEFAULT_KEY_SIZE,
            value_size: DEFAULT_VALUE_SIZE,
            read_limit: DEFAULT_READ_LIMIT,
            max_group_requests: DEFAULT_MAX_GROUP_REQUESTS,
            max_group_bytes: DEFAULT_MAX_GROUP_BYTES,
            queue_capacity: DEFAULT_QUEUE_CAPACITY,
            collection_delay: None,
            sync_mode: SyncMode::Real,
            sync_delay: Duration::ZERO,
            transaction_mode: TransactionMode::Unconditional,
            tokio_workers: std::thread::available_parallelism()
                .map_or(1, std::num::NonZeroUsize::get),
            blink_workers: 2,
            parallel_workers: 0,
            parallel_min_mutations: 0,
            parallel_background_min_operations: 0,
            seed: 0xd0db_2026_0000_0001,
            output: PathBuf::from(DEFAULT_OUTPUT),
            window_seconds: None,
        }
    }
}

impl Args {
    fn parse() -> Self {
        Self::parse_from(env::args().skip(1))
    }

    fn parse_from(arguments: impl IntoIterator<Item = String>) -> Self {
        let mut args = Self::default();
        let mut values = arguments.into_iter();
        while let Some(flag) = values.next() {
            match flag.as_str() {
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                "--suite" => args.suite = Suite::parse(&take_value(&mut values, &flag)),
                "--engine" => args.engine = EngineKind::parse(&take_value(&mut values, &flag)),
                "--writers" => args.writers = Some(parse_list(&take_value(&mut values, &flag))),
                "--readers" => args.readers = Some(parse_list(&take_value(&mut values, &flag))),
                "--widths" | "--transaction-widths" => {
                    args.widths = Some(parse_list(&take_value(&mut values, &flag)))
                }
                "--distribution" | "--distributions" => {
                    args.distributions = Some(
                        take_value(&mut values, &flag)
                            .split(',')
                            .map(Distribution::parse)
                            .collect(),
                    )
                }
                "--read-kind" | "--read-kinds" => {
                    args.read_kinds = Some(
                        take_value(&mut values, &flag)
                            .split(',')
                            .map(ReadKind::parse)
                            .collect(),
                    )
                }
                "--mix" | "--mixes" => {
                    args.mixes = Some(
                        take_value(&mut values, &flag)
                            .split(',')
                            .map(Mix::parse)
                            .collect(),
                    )
                }
                "--mixed-clients" => args.mixed_clients = true,
                "--main-compact-wal" => args.main_compact_wal = true,
                "--mixed-value-mode" => {
                    args.mixed_value_mode = MixedValueMode::parse(&take_value(&mut values, &flag))
                }
                "--blink-collection-policy" => {
                    args.collection_policy =
                        CollectionPolicy::parse(&take_value(&mut values, &flag))
                }
                "--duration" => args.duration = parse_duration(&take_value(&mut values, &flag)),
                "--warmup" => args.warmup = parse_duration(&take_value(&mut values, &flag)),
                "--repetitions" | "--reps" => {
                    args.repetitions = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--cache-capacity" => {
                    args.cache_capacity = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--working-set" => {
                    args.working_set = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--key-size" => args.key_size = parse_usize(&take_value(&mut values, &flag), &flag),
                "--value-size" => {
                    args.value_size = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--read-limit" => {
                    args.read_limit = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--group-limit" | "--max-group-requests" => {
                    args.max_group_requests = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--group-bytes" | "--max-group-bytes" => {
                    args.max_group_bytes = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--queue-capacity" => {
                    args.queue_capacity = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--window-seconds" => {
                    args.window_seconds =
                        Some(parse_usize(&take_value(&mut values, &flag), &flag).max(1) as u64)
                }
                "--checkpoint-wal-bytes" => CHECKPOINT_WAL_BYTES.store(
                    parse_usize(&take_value(&mut values, &flag), &flag) as u64,
                    Ordering::Relaxed,
                ),
                "--collection-delay" => {
                    args.collection_delay = Some(parse_duration(&take_value(&mut values, &flag)))
                }
                "--sync-mode" => args.sync_mode = SyncMode::parse(&take_value(&mut values, &flag)),
                "--sync-delay" => args.sync_delay = parse_duration(&take_value(&mut values, &flag)),
                "--transaction-mode" => {
                    args.transaction_mode = TransactionMode::parse(&take_value(&mut values, &flag))
                }
                "--tokio-workers" => {
                    args.tokio_workers = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--parallel-workers" => {
                    args.parallel_workers = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--parallel-min-mutations" => {
                    args.parallel_min_mutations =
                        parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--parallel-background-min-operations" => {
                    args.parallel_background_min_operations =
                        parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--blink-workers" => {
                    args.blink_workers = parse_usize(&take_value(&mut values, &flag), &flag)
                }
                "--seed" => args.seed = parse_u64(&take_value(&mut values, &flag), &flag),
                "--output" => args.output = PathBuf::from(take_value(&mut values, &flag)),
                other => panic!("unknown argument {other:?}; use --help"),
            }
        }
        args.validate();
        args
    }

    fn validate(&self) {
        assert!(
            !self.main_compact_wal || self.engine == EngineKind::MainBtree,
            "main-compact-wal requires main-btree"
        );
        assert!(self.repetitions > 0, "repetitions must be positive");
        assert!(self.tokio_workers > 0, "tokio-workers must be positive");
        assert!(self.blink_workers > 0, "blink-workers must be positive");
        assert!(self.working_set > 0, "working-set must be positive");
        assert!(self.key_size >= 2, "key-size must be at least 2 bytes");
        assert!(self.read_limit > 0, "read-limit must be positive");
        assert!(self.max_group_requests > 0, "group-limit must be positive");
        assert!(self.max_group_bytes > 0, "group-bytes must be positive");
        assert!(self.queue_capacity > 0, "queue-capacity must be positive");
        if let Some(widths) = &self.widths {
            assert!(
                widths.iter().all(|width| *width > 0),
                "widths must be positive"
            );
            assert!(widths.iter().all(|width| *width <= self.working_set));
        }
    }

    fn writers(&self) -> Vec<usize> {
        self.writers
            .clone()
            .unwrap_or_else(|| vec![1, 4, 16, 32, 64, 128])
    }

    fn readers(&self) -> Vec<usize> {
        self.readers
            .clone()
            .unwrap_or_else(|| vec![1, 4, 16, 32, 64, 128])
    }

    fn widths(&self) -> Vec<usize> {
        self.widths.clone().unwrap_or_else(|| vec![1, 16])
    }

    fn distributions(&self) -> Vec<Distribution> {
        self.distributions.clone().unwrap_or_else(|| {
            vec![
                Distribution::Uniform,
                Distribution::SameLeafHeavy,
                Distribution::DifferentLeafHeavy,
            ]
        })
    }

    fn read_kinds(&self) -> Vec<ReadKind> {
        self.read_kinds
            .clone()
            .unwrap_or_else(|| vec![ReadKind::Get, ReadKind::Query, ReadKind::Scan])
    }

    fn mixes(&self) -> Vec<Mix> {
        self.mixes
            .clone()
            .unwrap_or_else(|| vec![Mix::READ_HEAVY, Mix::BALANCED])
    }
}

fn print_help() {
    println!(
        "phase0-bench sustained baseline\n\n\
         Usage: cargo run --release -p dodb-storage --bin phase0-bench -- [options]\n\n\
         --engine main-btree|serial-blink|versioned-blink|planned-blink|parallel-blink|logical-overlay-blink\n\
         Suites: write, read, mixed, delay-sweep, sync-sweep, all\n\
         Options: --writers 1,4 --readers 1,4 --widths 1,16\n\
         --distributions uniform,same-leaf-heavy,different-leaf-heavy\n\
         --read-kinds get,query,scan --mixes 95/5,50/50\n\
         --duration 2s --warmup 1s --repetitions 3\n\
         --cache-capacity 256 --working-set 4096 --key-size 16 --value-size 64\n\
         --group-limit 64 --group-bytes 4194304 --queue-capacity 256\n\
         --blink-collection-policy current|main-parity --mixed-value-mode constant|changing\n\
         --main-compact-wal (experimental main B-tree compact redo)\n\
         --collection-delay 500us --sync-mode real|injected|disabled --sync-delay 1ms\n\
         --transaction-mode unconditional|insert-if-absent\n\
         --tokio-workers 12 --blink-workers 2 --parallel-workers 0|1|2 (planned-blink leaf workers, 0 = serial)\n\
         --parallel-background-min-operations 0 (minimum planned operations before background worker dispatch)\n\
         --seed 0xd0db2026 --output target/phase0/results.jsonl\n\
         --window-seconds 10 (per-window tx/s and latency, periodic WAL/RSS/dirty-page samples)"
    );
}

fn parse_usize(value: &str, flag: &str) -> usize {
    value
        .parse()
        .unwrap_or_else(|_| panic!("{flag} expects an unsigned integer, got {value:?}"))
}

fn take_value<I>(values: &mut I, flag: &str) -> String
where
    I: Iterator<Item = String>,
{
    values
        .next()
        .unwrap_or_else(|| panic!("{flag} requires a value"))
}

fn parse_u64(value: &str, flag: &str) -> u64 {
    let trimmed = value.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).unwrap_or_else(|_| {
            panic!("{flag} expects a decimal or hexadecimal unsigned integer, got {value:?}")
        })
    } else {
        trimmed.parse().unwrap_or_else(|_| {
            panic!("{flag} expects a decimal or hexadecimal unsigned integer, got {value:?}")
        })
    }
}

fn parse_list(value: &str) -> Vec<usize> {
    value
        .split(',')
        .map(|item| parse_usize(item.trim(), "list"))
        .collect()
}

fn parse_duration(value: &str) -> Duration {
    let value = value.trim();
    let (number, unit) = if let Some(number) = value.strip_suffix("us") {
        (number, "us")
    } else if let Some(number) = value.strip_suffix("ms") {
        (number, "ms")
    } else if let Some(number) = value.strip_suffix('s') {
        (number, "s")
    } else {
        (value, "ms")
    };
    let number: u64 = number
        .parse()
        .unwrap_or_else(|_| panic!("invalid duration {value:?}"));
    match unit {
        "us" => Duration::from_micros(number),
        "ms" => Duration::from_millis(number),
        "s" => Duration::from_secs(number),
        _ => unreachable!(),
    }
}

#[derive(Clone, Debug)]
struct Scenario {
    suite: Suite,
    workload: &'static str,
    writers: usize,
    readers: usize,
    width: usize,
    distribution: Distribution,
    read_kind: Option<ReadKind>,
    mix: Option<Mix>,
    collection_delay: Duration,
    sync_delay: Duration,
}

impl Scenario {
    fn name(&self) -> String {
        let mut name = format!("{}-{}w-{}r", self.workload, self.writers, self.readers);
        if let Some(read_kind) = self.read_kind {
            let _ = write!(name, "-{read_kind}", read_kind = read_kind.as_str());
        }
        if let Some(mix) = self.mix {
            let _ = write!(name, "-{}", mix.as_str());
        }
        let _ = write!(name, "-width{}-{}", self.width, self.distribution.as_str());
        name
    }
}

fn scenarios(args: &Args) -> Vec<Scenario> {
    let delay_values = [
        Duration::ZERO,
        Duration::from_micros(50),
        Duration::from_micros(100),
        Duration::from_micros(250),
        Duration::from_micros(500),
        Duration::from_millis(1),
        Duration::from_millis(2),
        Duration::from_millis(5),
    ];
    let selected_delay = args.collection_delay.unwrap_or(Duration::ZERO);
    let distributions = args.distributions();
    let widths = args.widths();
    let writers = args.writers();
    let readers = args.readers();
    let read_kinds = args.read_kinds();
    let mixes = args.mixes();
    let mut output = Vec::new();

    let include = |suite: Suite, selected: Suite| {
        args.suite == Suite::All || args.suite == selected || args.suite == suite
    };

    if include(Suite::Write, Suite::Write) {
        for writer in &writers {
            for width in &widths {
                for distribution in &distributions {
                    output.push(Scenario {
                        suite: Suite::Write,
                        workload: "100%-write",
                        writers: *writer,
                        readers: 0,
                        width: *width,
                        distribution: *distribution,
                        read_kind: None,
                        mix: None,
                        collection_delay: selected_delay,
                        sync_delay: args.sync_delay,
                    });
                }
            }
        }
    }

    if include(Suite::Read, Suite::Read) {
        for reader in &readers {
            for read_kind in &read_kinds {
                output.push(Scenario {
                    suite: Suite::Read,
                    workload: match read_kind {
                        ReadKind::Get => "100%-read-get",
                        ReadKind::Query => "100%-read-query",
                        ReadKind::Scan => "100%-read-scan",
                    },
                    writers: 0,
                    readers: *reader,
                    width: 1,
                    distribution: Distribution::Uniform,
                    read_kind: Some(*read_kind),
                    mix: None,
                    collection_delay: Duration::ZERO,
                    sync_delay: Duration::ZERO,
                });
            }
        }
    }

    if include(Suite::Mixed, Suite::Mixed) {
        let mixed_readers = args.readers.clone().unwrap_or_else(|| vec![16, 64]);
        let mixed_writers = args.writers.clone().unwrap_or_else(|| vec![16, 64]);
        let mixed_distributions = args
            .distributions
            .clone()
            .unwrap_or_else(|| vec![Distribution::Uniform]);
        let mixed_widths = args.widths.clone().unwrap_or_else(|| vec![1]);
        for readers in &mixed_readers {
            for writers in &mixed_writers {
                for distribution in &mixed_distributions {
                    for width in &mixed_widths {
                        for mix in &mixes {
                            output.push(Scenario {
                                suite: Suite::Mixed,
                                workload: "mixed",
                                writers: *writers,
                                readers: *readers,
                                width: *width,
                                distribution: *distribution,
                                read_kind: Some(ReadKind::Get),
                                mix: Some(*mix),
                                collection_delay: selected_delay,
                                sync_delay: args.sync_delay,
                            });
                        }
                    }
                }
            }
        }
    }

    if args.suite == Suite::DelaySweep || args.suite == Suite::All {
        let delays = if args.collection_delay.is_some() {
            vec![selected_delay]
        } else {
            delay_values.to_vec()
        };
        for delay in delays {
            output.push(Scenario {
                suite: Suite::DelaySweep,
                workload: "delay-write-control",
                writers: 64,
                readers: 0,
                width: 1,
                distribution: Distribution::Uniform,
                read_kind: None,
                mix: None,
                collection_delay: delay,
                sync_delay: args.sync_delay,
            });
            output.push(Scenario {
                suite: Suite::DelaySweep,
                workload: "delay-mixed-control",
                writers: 64,
                readers: 16,
                width: 1,
                distribution: Distribution::Uniform,
                read_kind: Some(ReadKind::Get),
                mix: Some(Mix::BALANCED),
                collection_delay: delay,
                sync_delay: args.sync_delay,
            });
        }
    }

    if args.suite == Suite::SyncSweep || args.suite == Suite::All {
        let sync_values = [
            Duration::ZERO,
            Duration::from_micros(100),
            Duration::from_millis(1),
            Duration::from_millis(5),
            Duration::from_millis(10),
        ];
        for sync_delay in sync_values {
            output.push(Scenario {
                suite: Suite::SyncSweep,
                workload: "sync-write-control",
                writers: 64,
                readers: 0,
                width: 1,
                distribution: Distribution::Uniform,
                read_kind: None,
                mix: None,
                collection_delay: Duration::ZERO,
                sync_delay,
            });
        }
    }

    assert!(!output.is_empty(), "selected suite produced no scenarios");
    output
}

#[derive(Clone, Debug)]
struct WorkloadConfig {
    distribution: Distribution,
    working_set: usize,
    key_size: usize,
    value_size: usize,
    width: usize,
    transaction_mode: TransactionMode,
    read_limit: usize,
}

#[derive(Clone, Debug)]
struct WorkloadGenerator {
    config: WorkloadConfig,
    worker_id: usize,
    operation: u64,
    state: u64,
    mixed_value_context: Option<MixedValueContext>,
    mixed_value_mode: MixedValueMode,
}

#[derive(Clone, Copy, Debug)]
struct MixedValueContext {
    phase_seed: u64,
    operation_seed: u64,
    operation_index: u64,
}

const QUERY_PK_COUNT: usize = 8;
const QUERY_BASE_ROWS_PER_PK: usize = 32;
const QUERY_INPUT_PLAN_REQUESTS: usize = 131_072;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct QueryRange {
    pk_index: usize,
    start_index: usize,
    limit: usize,
}

impl QueryRange {
    fn request(self, key_size: usize) -> BatchRequest {
        BatchRequest::Query {
            pk: PrimaryKey::new(distributed_query_pk(key_size, self.pk_index)),
            exclusive_after_sk: (self.start_index > 0).then(|| {
                SortKey::new(component_bytes(
                    0x43,
                    (self.start_index - 1) as u64,
                    key_component_lengths(key_size).1,
                ))
            }),
            limit: self.limit,
        }
    }
}

fn query_rows_per_pk(read_limit: usize) -> usize {
    QUERY_BASE_ROWS_PER_PK.max(read_limit.saturating_add(16))
}

fn query_range_count(read_limit: usize) -> usize {
    QUERY_PK_COUNT * (query_rows_per_pk(read_limit) - read_limit + 1)
}

fn query_range_for_index(index: usize, read_limit: usize) -> QueryRange {
    let starts_per_pk = query_rows_per_pk(read_limit) - read_limit + 1;
    QueryRange {
        pk_index: index / starts_per_pk,
        start_index: index % starts_per_pk,
        limit: read_limit,
    }
}

fn distributed_query_pk(key_size: usize, pk_index: usize) -> Vec<u8> {
    component_bytes(0x33, pk_index as u64, key_component_lengths(key_size).0)
}

fn distributed_query_key(key_size: usize, pk_index: usize, row_index: usize) -> DocumentKey {
    DocumentKey::new(
        distributed_query_pk(key_size, pk_index),
        component_bytes(0x43, row_index as u64, key_component_lengths(key_size).1),
    )
}

fn distributed_query_value_index(pk_index: usize, row_index: usize, read_limit: usize) -> u64 {
    (pk_index * query_rows_per_pk(read_limit) + row_index) as u64
}

impl WorkloadGenerator {
    fn new(config: WorkloadConfig, seed: u64, worker_id: usize) -> Self {
        Self {
            config,
            worker_id,
            operation: 0,
            state: seed ^ (worker_id as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15),
            mixed_value_context: None,
            mixed_value_mode: MixedValueMode::Constant,
        }
    }

    fn new_mixed(
        config: WorkloadConfig,
        operation_seed: u64,
        phase_seed: u64,
        operation_index: u64,
        value_mode: MixedValueMode,
    ) -> Self {
        let mut generator = Self::new(config, operation_seed, 0);
        generator.mixed_value_mode = value_mode;
        if value_mode == MixedValueMode::Changing {
            generator.mixed_value_context = Some(MixedValueContext {
                phase_seed,
                operation_seed,
                operation_index,
            });
        }
        generator
    }

    fn next_transaction(&mut self) -> TransactionRequest {
        let mut keys = Vec::with_capacity(self.config.width);
        let mut seen = HashSet::with_capacity(self.config.width);
        for offset in 0..self.config.width {
            let index = self.next_index(offset);
            let key = match self.config.transaction_mode {
                TransactionMode::Unconditional => self.key_for_index(index),
                TransactionMode::InsertIfAbsent => self.key_for_index(
                    self.config
                        .working_set
                        .saturating_add(self.worker_id.saturating_mul(1_000_000))
                        .saturating_add((self.operation as usize).saturating_mul(self.config.width))
                        .saturating_add(offset),
                ),
            };
            if seen.insert(key.clone()) {
                keys.push(key);
            } else {
                let fallback = self.key_for_index(
                    self.config
                        .working_set
                        .saturating_add(self.worker_id.saturating_mul(1_000_000))
                        .saturating_add((self.operation as usize).saturating_mul(self.config.width))
                        .saturating_add(offset),
                );
                assert!(
                    seen.insert(fallback.clone()),
                    "transaction key generator duplicated a key"
                );
                keys.push(fallback);
            }
        }
        self.operation = self.operation.wrapping_add(1);
        let mutations = keys
            .iter()
            .enumerate()
            .map(|(offset, key)| TransactionMutation::Put {
                key: key.clone(),
                value: match (self.mixed_value_mode, self.mixed_value_context) {
                    (MixedValueMode::Changing, Some(context)) => mixed_value_bytes(
                        self.config.value_size,
                        context.phase_seed,
                        context.operation_seed,
                        context.operation_index,
                        offset,
                    ),
                    _ => value_bytes(self.config.value_size, self.operation, offset),
                },
            })
            .collect::<Vec<_>>();
        let conditions = match self.config.transaction_mode {
            TransactionMode::Unconditional => Vec::new(),
            TransactionMode::InsertIfAbsent => keys
                .into_iter()
                .map(|key| TransactionCondition::NotExists { key })
                .collect(),
        };
        TransactionRequest::new(conditions, mutations)
    }

    fn next_read(&mut self, kind: ReadKind) -> BatchRequest {
        match kind {
            ReadKind::Get => {
                let index = self.next_index(0);
                self.operation = self.operation.wrapping_add(1);
                BatchRequest::Get {
                    key: self.key_for_index(index),
                }
            }
            ReadKind::Query => {
                let range_index =
                    self.random_query_range_index(query_range_count(self.config.read_limit));
                self.operation = self.operation.wrapping_add(1);
                query_range_for_index(range_index, self.config.read_limit)
                    .request(self.config.key_size)
            }
            ReadKind::Scan => {
                self.operation = self.operation.wrapping_add(1);
                BatchRequest::Scan {
                    exclusive_after_key: None,
                    limit: self.config.read_limit,
                }
            }
        }
    }

    fn next_index(&mut self, offset: usize) -> usize {
        let working_set = self.config.working_set;
        match self.config.distribution {
            Distribution::Uniform => self.random_bounded(working_set),
            Distribution::Sequential => {
                ((self.operation as usize)
                    .saturating_mul(self.config.width)
                    .saturating_add(offset))
                    % working_set
            }
            Distribution::Hotspot => {
                let hotset = (working_set / 100).max(self.config.width);
                if self.random_bounded(100) < 80 {
                    self.random_bounded(hotset.min(working_set))
                } else {
                    self.random_bounded(working_set)
                }
            }
            Distribution::SameLeafHeavy => {
                let span = working_set.min(64).max(self.config.width);
                ((self.operation as usize)
                    .saturating_mul(self.config.width)
                    .saturating_add(offset))
                    % span.min(working_set)
            }
            Distribution::DifferentLeafHeavy => {
                (self
                    .worker_id
                    .saturating_mul(1_009)
                    .saturating_add((self.operation as usize).saturating_mul(self.config.width))
                    .saturating_add(offset))
                    % working_set
            }
        }
    }

    fn random_bounded(&mut self, bound: usize) -> usize {
        assert!(bound > 0);
        self.state = splitmix64(self.state);
        (self.state as usize) % bound
    }

    fn random_query_range_index(&mut self, bound: usize) -> usize {
        let bound = bound as u64;
        let rejection_threshold = bound.wrapping_neg() % bound;
        loop {
            self.state = splitmix64(self.state);
            if self.state >= rejection_threshold {
                return (self.state % bound) as usize;
            }
        }
    }

    fn key_for_index(&self, index: usize) -> DocumentKey {
        let (pk_len, sk_len) = key_component_lengths(self.config.key_size);
        match self.config.distribution {
            Distribution::SameLeafHeavy => DocumentKey::new(
                component_bytes(0x11, 0, pk_len),
                component_bytes(0x21, index as u64, sk_len),
            ),
            Distribution::DifferentLeafHeavy => DocumentKey::new(
                component_bytes(0x31, index as u64, pk_len),
                component_bytes(0x41, index as u64, sk_len),
            ),
            Distribution::Uniform | Distribution::Sequential | Distribution::Hotspot => {
                DocumentKey::new(
                    component_bytes(0x51, (index % 128) as u64, pk_len),
                    component_bytes(0x61, index as u64, sk_len),
                )
            }
        }
    }

    fn seed_keys(&self) -> impl Iterator<Item = DocumentKey> + '_ {
        (0..self.config.working_set).map(|index| self.key_for_index(index))
    }
}

fn splitmix64(mut state: u64) -> u64 {
    state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn mixed_operation_is_read(operation_index: u64, read_percent: u8) -> bool {
    operation_index % 100 < u64::from(read_percent)
}

fn mixed_operation_seed(phase_seed: u64, operation_index: u64) -> u64 {
    splitmix64(phase_seed ^ operation_index.wrapping_mul(0x9e37_79b9_7f4a_7c15))
}

fn mixed_value_bytes(
    length: usize,
    phase_seed: u64,
    operation_seed: u64,
    operation_index: u64,
    mutation_index: usize,
) -> Vec<u8> {
    let mut bytes = vec![0; length];
    let nonce = operation_index.to_be_bytes();
    let nonce_length = length.min(nonce.len());
    bytes[..nonce_length].copy_from_slice(&nonce[nonce.len() - nonce_length..]);

    let mut state = phase_seed
        ^ operation_seed.rotate_left(17)
        ^ operation_index.wrapping_mul(0xd6e8_feb8_6659_fd93)
        ^ (mutation_index as u64).wrapping_mul(0xa076_1d64_78bd_642f);
    let mut position = nonce_length;
    while position < length {
        state = splitmix64(state);
        for byte in state.to_be_bytes() {
            if position >= length {
                break;
            }
            bytes[position] = byte;
            position += 1;
        }
    }
    bytes
}

fn mixed_trace_prefix_hash(
    workload: WorkloadConfig,
    phase_seed: u64,
    read_percent: u8,
    value_mode: MixedValueMode,
    prefix_operations: u64,
) -> u64 {
    let mut trace_state = 0xcbf2_9ce4_8422_2325;
    for operation_index in 0..prefix_operations {
        let is_read = mixed_operation_is_read(operation_index, read_percent);
        let operation_seed = mixed_operation_seed(phase_seed, operation_index);
        let mut generator = WorkloadGenerator::new_mixed(
            workload.clone(),
            operation_seed,
            phase_seed,
            operation_index,
            value_mode,
        );
        let request = generator.next_transaction();
        absorb_trace(&mut trace_state, &operation_index.to_be_bytes());
        absorb_trace(&mut trace_state, &[u8::from(is_read)]);
        if is_read {
            let Some(TransactionMutation::Put { key, .. }) = request.mutations.first() else {
                unreachable!();
            };
            let key = document_key_bytes(key);
            absorb_trace(&mut trace_state, &(key.len() as u32).to_be_bytes());
            absorb_trace(&mut trace_state, &key);
        } else {
            absorb_trace(
                &mut trace_state,
                &(request.mutations.len() as u32).to_be_bytes(),
            );
            for mutation in request.mutations {
                let TransactionMutation::Put { key, value } = mutation else {
                    unreachable!();
                };
                let key = document_key_bytes(&key);
                absorb_trace(&mut trace_state, &(key.len() as u32).to_be_bytes());
                absorb_trace(&mut trace_state, &key);
                absorb_trace(&mut trace_state, &(value.len() as u32).to_be_bytes());
                absorb_trace(&mut trace_state, &value);
            }
        }
    }
    trace_state
}

fn document_key_bytes(key: &DocumentKey) -> Vec<u8> {
    let mut bytes = key.pk.as_bytes().to_vec();
    bytes.extend_from_slice(key.sk.as_bytes());
    bytes
}

fn absorb_trace(trace_state: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *trace_state ^= u64::from(*byte);
        *trace_state = trace_state.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

fn key_component_lengths(key_size: usize) -> (usize, usize) {
    let pk_len = (key_size / 2).max(1);
    let sk_len = key_size.saturating_sub(pk_len).max(1);
    (pk_len, sk_len)
}

fn component_bytes(tag: u8, value: u64, length: usize) -> Vec<u8> {
    let mut bytes = vec![tag; length];
    let encoded = value.to_be_bytes();
    let copy_len = encoded.len().min(length);
    bytes[length - copy_len..].copy_from_slice(&encoded[encoded.len() - copy_len..]);
    bytes
}

fn invocation_seed(base_seed: u64, scenario_index: usize, repetition: usize) -> u64 {
    base_seed
        .wrapping_add((scenario_index as u64).wrapping_mul(0x9e37_79b9))
        .wrapping_add(repetition as u64)
}

fn query_input_plan_fingerprint(args: &Args, scenario: &Scenario, repetition_seed: u64) -> String {
    let workload = WorkloadConfig {
        distribution: scenario.distribution,
        working_set: args.working_set,
        key_size: args.key_size,
        value_size: args.value_size,
        width: scenario.width,
        transaction_mode: args.transaction_mode,
        read_limit: args.read_limit,
    };
    let mut fingerprint = 0xcbf2_9ce4_8422_2325u64;
    for client_index in 0..scenario.readers {
        let mut generator = WorkloadGenerator::new(
            workload.clone(),
            repetition_seed ^ 0xbbbb_0000 ^ 0x2000_0000,
            client_index,
        );
        for _ in 0..QUERY_INPUT_PLAN_REQUESTS {
            let request = generator.next_read(ReadKind::Query);
            let BatchRequest::Query {
                pk,
                exclusive_after_sk,
                limit,
            } = request
            else {
                unreachable!()
            };
            for byte in pk.as_bytes().iter().copied().chain(
                exclusive_after_sk
                    .iter()
                    .flat_map(|key| key.as_bytes().iter().copied()),
            ) {
                fingerprint ^= byte as u64;
                fingerprint = fingerprint.wrapping_mul(0x100_0000_01b3);
            }
            for byte in (limit as u64).to_be_bytes() {
                fingerprint ^= byte as u64;
                fingerprint = fingerprint.wrapping_mul(0x100_0000_01b3);
            }
            fingerprint ^= 0xff;
            fingerprint = fingerprint.wrapping_mul(0x100_0000_01b3);
        }
    }
    format!(
        "fnv1a64:{fingerprint:016x}:clients={}:requests_each={QUERY_INPUT_PLAN_REQUESTS}",
        scenario.readers,
    )
}

fn query_result_is_valid(
    request: &BatchRequest,
    rows: &[dodb_storage::Document],
    key_size: usize,
    value_size: usize,
    read_limit: usize,
) -> bool {
    let Some(expected_range) = query_range_from_request(request, key_size, read_limit) else {
        return false;
    };
    if rows.len() != expected_range.limit {
        return false;
    }
    rows.iter().enumerate().all(|(offset, row)| {
        let row_index = expected_range.start_index + offset;
        row.key == distributed_query_key(key_size, expected_range.pk_index, row_index)
            && row.value
                == value_bytes(
                    value_size,
                    distributed_query_value_index(expected_range.pk_index, row_index, read_limit),
                    0,
                )
    })
}

fn query_range_from_request(
    request: &BatchRequest,
    key_size: usize,
    read_limit: usize,
) -> Option<QueryRange> {
    let BatchRequest::Query {
        pk,
        exclusive_after_sk,
        limit,
    } = request
    else {
        return None;
    };
    let Some(pk_index) = (0..QUERY_PK_COUNT)
        .find(|pk_index| pk.as_bytes() == distributed_query_pk(key_size, *pk_index))
    else {
        return None;
    };
    let start_index = match exclusive_after_sk {
        Some(sk) => {
            let Some(start_index) = (1..query_rows_per_pk(read_limit)).find(|row_index| {
                sk.as_bytes()
                    == component_bytes(
                        0x43,
                        (*row_index - 1) as u64,
                        key_component_lengths(key_size).1,
                    )
            }) else {
                return None;
            };
            start_index
        }
        None => 0,
    };
    if *limit != read_limit || start_index + limit > query_rows_per_pk(read_limit) {
        return None;
    }
    Some(QueryRange {
        pk_index,
        start_index,
        limit: *limit,
    })
}

fn record_query_response(
    request: &BatchRequest,
    response: &BatchResponse,
    workload: &WorkloadConfig,
    stats: &mut WorkerStats,
) -> bool {
    match response {
        BatchResponse::Query(rows) => {
            stats.returned_rows += rows.len() as u64;
            stats.query_checked_requests += 1;
            stats.query_checked_rows += rows.len() as u64;
            let valid = query_result_is_valid(
                request,
                &rows,
                workload.key_size,
                workload.value_size,
                workload.read_limit,
            );
            if valid {
                stats.successful_queries += 1;
            } else {
                stats.query_validation_failures += 1;
                stats.errors += 1;
            }
            valid
        }
        _ => {
            stats.query_checked_requests += 1;
            stats.query_validation_failures += 1;
            stats.errors += 1;
            false
        }
    }
}

fn measured_query_verification_result(stats: &WorkerStats) -> Result<()> {
    if stats.query_checked_requests == 0 || stats.query_validation_failures > 0 {
        Err(Error::invariant(
            "Measured Query response verification failed or was inconclusive",
        ))
    } else {
        Ok(())
    }
}

const MAX_SAMPLED_KEYS: usize = 1_024;
const MAX_SAMPLE_SELECTION_CANDIDATES: usize = 131_072;
const MAX_VERIFICATION_EVENTS: usize = 65_536;
const MAX_VERIFICATION_MEMORY_BYTES: usize = 8 * 1024 * 1024;

fn sampled_read_keys(args: &Args, scenario: &Scenario) -> Arc<HashSet<DocumentKey>> {
    let config = WorkloadConfig {
        distribution: scenario.distribution,
        working_set: args.working_set,
        key_size: args.key_size,
        value_size: args.value_size,
        width: scenario.width,
        transaction_mode: args.transaction_mode,
        read_limit: args.read_limit,
    };
    let generator = WorkloadGenerator::new(config, 0, 0);
    let mut keys = HashSet::new();
    let mut estimated_bytes = 0usize;
    let selected_indexes = if args.working_set <= MAX_SAMPLE_SELECTION_CANDIDATES {
        (0..args.working_set)
            .filter(|index| splitmix64(args.seed ^ 0x5eed_0000 ^ *index as u64) % 100 == 0)
            .collect::<Vec<_>>()
    } else {
        let mut selected_indexes = HashSet::with_capacity(MAX_SAMPLED_KEYS);
        for attempt in 0..MAX_SAMPLED_KEYS * 8 {
            let index = (splitmix64(args.seed ^ 0x5eed_0000 ^ attempt as u64)
                % args.working_set as u64) as usize;
            selected_indexes.insert(index);
            if selected_indexes.len() == MAX_SAMPLED_KEYS {
                break;
            }
        }
        selected_indexes.into_iter().collect::<Vec<_>>()
    };
    for index in selected_indexes {
        let key = generator.key_for_index(index);
        let key_bytes = std::mem::size_of::<DocumentKey>()
            + key.pk.as_bytes().len()
            + key.sk.as_bytes().len()
            + args.value_size
            + 64;
        if keys.len() == MAX_SAMPLED_KEYS
            || estimated_bytes.saturating_add(key_bytes) > MAX_VERIFICATION_MEMORY_BYTES / 4
        {
            break;
        }
        estimated_bytes += key_bytes;
        keys.insert(key);
    }
    if keys.is_empty() && args.working_set > 0 {
        let key = generator.key_for_index(0);
        let key_bytes = std::mem::size_of::<DocumentKey>()
            + key.pk.as_bytes().len()
            + key.sk.as_bytes().len()
            + args.value_size
            + 64;
        if key_bytes <= MAX_VERIFICATION_MEMORY_BYTES / 4 {
            keys.insert(key);
        }
    }
    Arc::new(keys)
}

fn sampled_mutations(
    request: &TransactionRequest,
    sampled_keys: &HashSet<DocumentKey>,
) -> Vec<(DocumentKey, Vec<u8>)> {
    request
        .mutations
        .iter()
        .filter_map(|mutation| match mutation {
            TransactionMutation::Put { key, value } if sampled_keys.contains(key) => {
                Some((key.clone(), value.clone()))
            }
            _ => None,
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VerificationStatus {
    NotApplicable,
    Passed,
    Failed,
    Inconclusive,
}

impl VerificationStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Inconclusive => "inconclusive",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct VerificationSummary {
    status: VerificationStatus,
    sampled_key_count: usize,
    sampled_reads_observed: u64,
    sampled_reads_checked: u64,
    sampled_reads_failed: u64,
    sampled_reads_indeterminate: u64,
    sampled_writes_observed: u64,
    sampled_writes_confirmed: u64,
    sampled_writes_ambiguous: u64,
    sampled_writes_stored: u64,
    changed_key_reads_checked: u64,
    omitted_events: u64,
    estimated_memory_bytes: usize,
    estimated_index_memory_bytes: usize,
    verification_comparisons: u64,
}

impl VerificationSummary {
    fn not_applicable() -> Self {
        Self {
            status: VerificationStatus::NotApplicable,
            sampled_key_count: 0,
            sampled_reads_observed: 0,
            sampled_reads_checked: 0,
            sampled_reads_failed: 0,
            sampled_reads_indeterminate: 0,
            sampled_writes_observed: 0,
            sampled_writes_confirmed: 0,
            sampled_writes_ambiguous: 0,
            sampled_writes_stored: 0,
            changed_key_reads_checked: 0,
            omitted_events: 0,
            estimated_memory_bytes: 0,
            estimated_index_memory_bytes: 0,
            verification_comparisons: 0,
        }
    }
}

#[derive(Clone, Debug)]
struct SampledMutation {
    key: DocumentKey,
    value: Vec<u8>,
    revision: Option<dodb_core::Revision>,
    started_at: Instant,
    finished_at: Instant,
    ambiguous: bool,
}

#[derive(Clone, Debug)]
struct SampledRead {
    key: DocumentKey,
    value: Option<Vec<u8>>,
    revision: Option<dodb_core::Revision>,
    started_at: Instant,
    finished_at: Instant,
}

#[derive(Clone, Debug)]
struct SampledSeed {
    value: Vec<u8>,
    revision: dodb_core::Revision,
}

#[derive(Default)]
struct AmbiguousValueWindowIndex {
    starts: Vec<Instant>,
    prefix_max_resolved_at: Vec<Option<Instant>>,
    prefix_has_unresolved: Vec<bool>,
}

struct SampledReadVerifier {
    sampled_key_count: usize,
    seed_states: HashMap<DocumentKey, SampledSeed>,
    mutations: Vec<SampledMutation>,
    reads: Vec<SampledRead>,
    sampled_reads_observed: u64,
    sampled_writes_observed: u64,
    sampled_writes_confirmed: u64,
    sampled_writes_ambiguous: u64,
    omitted_events: u64,
    estimated_memory_bytes: usize,
}

impl SampledReadVerifier {
    fn new(
        sampled_keys: &HashSet<DocumentKey>,
        seed_states: HashMap<DocumentKey, SampledSeed>,
    ) -> Self {
        let estimated_memory_bytes = seed_states
            .iter()
            .map(|(key, seed)| sampled_event_memory_bytes(key, &seed.value))
            .sum();
        Self {
            sampled_key_count: sampled_keys.len(),
            seed_states,
            mutations: Vec::new(),
            reads: Vec::new(),
            sampled_reads_observed: 0,
            sampled_writes_observed: 0,
            sampled_writes_confirmed: 0,
            sampled_writes_ambiguous: 0,
            omitted_events: 0,
            estimated_memory_bytes,
        }
    }

    fn record_read(&mut self, read: SampledRead) {
        self.sampled_reads_observed = self.sampled_reads_observed.saturating_add(1);
        if !self.reserve_event_memory(&read.key, read.value.as_deref().unwrap_or_default()) {
            return;
        }
        self.reads.push(read);
    }

    fn record_mutations(
        &mut self,
        mutations: Vec<(DocumentKey, Vec<u8>)>,
        revision: Option<dodb_core::Revision>,
        started_at: Instant,
        finished_at: Instant,
        ambiguous: bool,
    ) {
        for (key, value) in mutations {
            self.sampled_writes_observed = self.sampled_writes_observed.saturating_add(1);
            if ambiguous {
                self.sampled_writes_ambiguous = self.sampled_writes_ambiguous.saturating_add(1);
            } else {
                self.sampled_writes_confirmed = self.sampled_writes_confirmed.saturating_add(1);
            }
            if !self.reserve_event_memory(&key, &value) {
                continue;
            }
            self.mutations.push(SampledMutation {
                key,
                value,
                revision,
                started_at,
                finished_at,
                ambiguous,
            });
        }
    }

    fn reserve_event_memory(&mut self, key: &DocumentKey, value: &[u8]) -> bool {
        let event_count = self.reads.len().saturating_add(self.mutations.len());
        let event_bytes = sampled_event_memory_bytes(key, value);
        if event_count >= MAX_VERIFICATION_EVENTS
            || self.estimated_memory_bytes.saturating_add(event_bytes)
                > MAX_VERIFICATION_MEMORY_BYTES
        {
            self.omitted_events = self.omitted_events.saturating_add(1);
            return false;
        }
        self.estimated_memory_bytes += event_bytes;
        true
    }

    fn summarize(&self) -> VerificationSummary {
        let mut successful_by_key = HashMap::<DocumentKey, Vec<&SampledMutation>>::new();
        let mut successful_by_start = HashMap::<DocumentKey, Vec<&SampledMutation>>::new();
        let mut ambiguous_events =
            HashMap::<DocumentKey, HashMap<Vec<u8>, Vec<(Instant, Option<Instant>)>>>::new();
        let mut unresolved_writes = 0u64;
        for mutation in &self.mutations {
            if !mutation.ambiguous {
                successful_by_key
                    .entry(mutation.key.clone())
                    .or_default()
                    .push(mutation);
                successful_by_start
                    .entry(mutation.key.clone())
                    .or_default()
                    .push(mutation);
            }
        }

        for mutation in self.mutations.iter().filter(|mutation| mutation.ambiguous) {
            let resolved_at = successful_by_start
                .get(&mutation.key)
                .and_then(|successes| {
                    successes
                        .iter()
                        .filter(|success| success.started_at > mutation.finished_at)
                        .min_by_key(|success| success.finished_at)
                        .map(|success| success.finished_at)
                });
            if resolved_at.is_none() {
                unresolved_writes = unresolved_writes.saturating_add(1);
            }
            ambiguous_events
                .entry(mutation.key.clone())
                .or_default()
                .entry(mutation.value.clone())
                .or_default()
                .push((mutation.started_at, resolved_at));
        }

        let mut ambiguous_windows =
            HashMap::<DocumentKey, HashMap<Vec<u8>, AmbiguousValueWindowIndex>>::new();
        for (key, values) in ambiguous_events {
            let mut indexed_values = HashMap::with_capacity(values.len());
            for (value, mut events) in values {
                events.sort_by_key(|(started_at, _)| *started_at);
                let mut index = AmbiguousValueWindowIndex::default();
                let mut latest_resolved_at = None;
                let mut has_unresolved = false;
                for (started_at, resolved_at) in events {
                    if let Some(resolved_at) = resolved_at {
                        latest_resolved_at = Some(
                            latest_resolved_at
                                .map_or(resolved_at, |current: Instant| current.max(resolved_at)),
                        );
                    } else {
                        has_unresolved = true;
                    }
                    index.starts.push(started_at);
                    index.prefix_max_resolved_at.push(latest_resolved_at);
                    index.prefix_has_unresolved.push(has_unresolved);
                }
                indexed_values.insert(value, index);
            }
            ambiguous_windows.insert(key, indexed_values);
        }

        let mut completed_frontiers =
            HashMap::<DocumentKey, Vec<(Instant, dodb_core::Revision)>>::new();
        let mut revisions_by_key =
            HashMap::<DocumentKey, HashMap<dodb_core::Revision, &SampledMutation>>::new();
        for (key, mutations) in &mut successful_by_key {
            mutations.sort_by_key(|mutation| mutation.finished_at);
            let mut max_revision = self.seed_states.get(key).map(|seed| seed.revision);
            let mut frontier = Vec::with_capacity(mutations.len());
            let mut revisions = HashMap::with_capacity(mutations.len());
            for mutation in mutations.iter() {
                if let Some(revision) = mutation.revision {
                    max_revision =
                        Some(max_revision.map_or(revision, |current| current.max(revision)));
                    revisions.insert(revision, *mutation);
                    if let Some(max_revision) = max_revision {
                        frontier.push((mutation.finished_at, max_revision));
                    }
                }
            }
            completed_frontiers.insert(key.clone(), frontier);
            revisions_by_key.insert(key.clone(), revisions);
        }

        let mut reads_checked = 0u64;
        let mut reads_failed = 0u64;
        let mut reads_indeterminate = 0u64;
        let mut changed_key_reads_checked = 0u64;
        let mut comparisons = 0u64;
        for read in &self.reads {
            reads_checked = reads_checked.saturating_add(1);
            let Some(actual_value) = read.value.as_deref() else {
                reads_failed = reads_failed.saturating_add(1);
                continue;
            };
            let Some(actual_revision) = read.revision else {
                reads_failed = reads_failed.saturating_add(1);
                continue;
            };
            let Some(seed) = self.seed_states.get(&read.key) else {
                reads_failed = reads_failed.saturating_add(1);
                continue;
            };
            let frontier = completed_frontiers.get(&read.key);
            let latest_completed_revision = frontier.and_then(|entries| {
                let completed_count =
                    entries.partition_point(|(finished_at, _)| *finished_at <= read.started_at);
                completed_count.checked_sub(1).map(|index| entries[index].1)
            });
            let required_revision = latest_completed_revision
                .map_or(seed.revision, |revision| revision.max(seed.revision));
            if actual_revision < required_revision {
                reads_failed = reads_failed.saturating_add(1);
                continue;
            }

            let seed_matches = actual_revision == seed.revision && actual_value == seed.value;
            comparisons = comparisons.saturating_add(1);
            let matching_write = revisions_by_key
                .get(&read.key)
                .and_then(|revisions| revisions.get(&actual_revision))
                .copied()
                .filter(|mutation| {
                    comparisons = comparisons.saturating_add(1);
                    mutation.started_at < read.finished_at
                        && mutation
                            .revision
                            .is_some_and(|revision| revision >= required_revision)
                        && mutation.value == actual_value
                });
            if seed_matches || matching_write.is_some() {
                if matching_write.is_some_and(|mutation| {
                    mutation
                        .revision
                        .is_some_and(|revision| revision > seed.revision)
                }) {
                    changed_key_reads_checked = changed_key_reads_checked.saturating_add(1);
                }
                continue;
            }

            let ambiguous_match = ambiguous_windows
                .get(&read.key)
                .and_then(|values| values.get(actual_value))
                .is_some_and(|window| {
                    comparisons = comparisons.saturating_add(1);
                    let event_count = window
                        .starts
                        .partition_point(|started_at| *started_at < read.finished_at);
                    event_count > 0
                        && (window.prefix_has_unresolved[event_count - 1]
                            || window.prefix_max_resolved_at[event_count - 1]
                                .is_some_and(|resolved_at| resolved_at > read.started_at))
                });
            if ambiguous_match || self.omitted_events > 0 {
                reads_indeterminate = reads_indeterminate.saturating_add(1);
            } else {
                reads_failed = reads_failed.saturating_add(1);
            }
        }

        let status = if reads_failed > 0 {
            VerificationStatus::Failed
        } else if self.sampled_reads_observed == 0
            || reads_checked == 0
            || reads_indeterminate > 0
            || unresolved_writes > 0
            || self.omitted_events > 0
            || self.seed_states.len() != self.sampled_key_count
        {
            VerificationStatus::Inconclusive
        } else {
            VerificationStatus::Passed
        };
        VerificationSummary {
            status,
            sampled_key_count: self.sampled_key_count,
            sampled_reads_observed: self.sampled_reads_observed,
            sampled_reads_checked: reads_checked,
            sampled_reads_failed: reads_failed,
            sampled_reads_indeterminate: reads_indeterminate,
            sampled_writes_observed: self.sampled_writes_observed,
            sampled_writes_confirmed: self.sampled_writes_confirmed,
            sampled_writes_ambiguous: self.sampled_writes_ambiguous,
            sampled_writes_stored: self.mutations.len() as u64,
            changed_key_reads_checked,
            omitted_events: self.omitted_events,
            estimated_memory_bytes: self.estimated_memory_bytes,
            estimated_index_memory_bytes: self.mutations.len().saturating_mul(96),
            verification_comparisons: comparisons,
        }
    }
}

fn sampled_event_memory_bytes(key: &DocumentKey, value: &[u8]) -> usize {
    std::mem::size_of::<SampledRead>().max(std::mem::size_of::<SampledMutation>())
        + std::mem::size_of::<DocumentKey>()
        + key.pk.as_bytes().len()
        + key.sk.as_bytes().len()
        + value.len()
        + 64
}

fn value_bytes(length: usize, operation: u64, offset: usize) -> Vec<u8> {
    let byte = (operation.wrapping_add(offset as u64) & 0xff) as u8;
    vec![byte; length]
}

struct BenchFile {
    inner: ProductionFile,
    sync_mode: SyncMode,
    sync_delay: Duration,
}

impl BenchFile {
    fn open(path: &Path, sync_mode: SyncMode, sync_delay: Duration) -> Result<Self> {
        Ok(Self {
            inner: ProductionFile::open(path)?,
            sync_mode,
            sync_delay,
        })
    }
}

fn perform_benchmark_sync(
    mode: SyncMode,
    delay: Duration,
    real_sync: impl FnOnce() -> Result<()>,
) -> Result<()> {
    match mode {
        SyncMode::Real => real_sync(),
        SyncMode::Injected => {
            if !delay.is_zero() {
                std::thread::sleep(delay);
            }
            real_sync()
        }
        SyncMode::Disabled => Ok(()),
    }
}

impl DurableFile for BenchFile {
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<usize> {
        self.inner.write_at(offset, bytes)
    }

    fn len(&self) -> Result<u64> {
        self.inner.len()
    }

    fn set_len(&mut self, length: u64) -> Result<()> {
        self.inner.set_len(length)
    }

    fn sync_data(&mut self) -> Result<()> {
        let mode = self.sync_mode;
        let delay = self.sync_delay;
        perform_benchmark_sync(mode, delay, || self.inner.sync_data())
    }

    fn sync_all(&mut self) -> Result<()> {
        let mode = self.sync_mode;
        let delay = self.sync_delay;
        perform_benchmark_sync(mode, delay, || self.inner.sync_all())
    }

    fn try_clone_for_background(&self) -> Option<Box<dyn DurableFile + Send>> {
        self.inner.try_clone().ok().map(|inner| {
            Box::new(BenchFile {
                inner,
                sync_mode: self.sync_mode,
                sync_delay: self.sync_delay,
            }) as Box<dyn DurableFile + Send>
        })
    }
}

#[derive(Clone)]
struct EngineSnapshot {
    coordinator: dodb_storage::CoordinatorMetrics,
    storage: Option<StorageMetrics>,
    wal: Option<WalMetrics>,
    blink: Option<BlinkSplitMetrics>,
    batch: Option<BlinkBatchMetrics>,
    versioned: Option<BlinkVersionedReadMetrics>,
    checkpoint: Option<dodb_storage::BlinkCheckpointMetrics>,
    dirty_pages: Option<usize>,
}

trait EngineAdapter: Send + Sync {
    fn execute_transaction<'a>(
        &'a self,
        request: TransactionRequest,
    ) -> BoxFuture<'a, Result<TransactionResult>>;
    fn execute<'a>(&'a self, request: BatchRequest) -> BoxFuture<'a, Result<BatchResponse>>;
    fn snapshot(&self) -> EngineSnapshot;
    fn reset_checkpoint_metrics(&self) {}
    fn shutdown<'a>(&'a self) -> BoxFuture<'a, Result<()>>;
}

struct BaselineAdapter {
    shard: Arc<BenchShard>,
}

impl EngineAdapter for BaselineAdapter {
    fn execute_transaction<'a>(
        &'a self,
        request: TransactionRequest,
    ) -> BoxFuture<'a, Result<TransactionResult>> {
        Box::pin(self.shard.execute_transaction(request))
    }

    fn execute<'a>(&'a self, request: BatchRequest) -> BoxFuture<'a, Result<BatchResponse>> {
        Box::pin(self.shard.execute(request))
    }

    fn snapshot(&self) -> EngineSnapshot {
        EngineSnapshot {
            coordinator: self.shard.coordinator_metrics(),
            storage: self.shard.storage_metrics(),
            wal: self.shard.wal_metrics(),
            blink: None,
            batch: None,
            versioned: None,
            checkpoint: None,
            dirty_pages: None,
        }
    }

    fn shutdown<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(self.shard.shutdown())
    }
}

struct BlinkWork {
    request: TransactionRequest,
    enqueued: Instant,
    response: tokio::sync::oneshot::Sender<Result<TransactionResult>>,
}

fn transaction_request_size(request: &TransactionRequest) -> usize {
    request
        .conditions
        .iter()
        .map(|condition| condition.key().encode().len().saturating_add(32))
        .chain(request.mutations.iter().map(|mutation| {
            match mutation {
                TransactionMutation::Put { key, value } => key
                    .encode()
                    .len()
                    .saturating_add(value.len())
                    .saturating_add(32),
                TransactionMutation::Delete { key } => key.encode().len().saturating_add(32),
            }
        }))
        .fold(0usize, usize::saturating_add)
}

fn can_add_main_parity_request(
    request_count: usize,
    current_bytes: usize,
    request_bytes: usize,
    config: CoordinatorConfig,
) -> bool {
    request_count < config.max_group_requests
        && current_bytes.saturating_add(request_bytes) <= config.max_group_bytes
}

async fn collect_main_parity_group(
    first: BlinkWork,
    receiver: &mut tokio::sync::mpsc::Receiver<BlinkWork>,
    config: CoordinatorConfig,
) -> (Vec<BlinkWork>, Option<BlinkWork>, usize) {
    let mut requests = vec![first];
    let mut bytes = transaction_request_size(&requests[0].request);
    let mut pending = None;

    tokio::task::yield_now().await;
    loop {
        if requests.len() >= config.max_group_requests {
            break;
        }
        match receiver.try_recv() {
            Ok(request) => {
                let request_bytes = transaction_request_size(&request.request);
                if can_add_main_parity_request(requests.len(), bytes, request_bytes, config) {
                    bytes = bytes.saturating_add(request_bytes);
                    requests.push(request);
                } else {
                    pending = Some(request);
                    break;
                }
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty)
            | Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
        }
    }

    if requests.len() > 1
        && pending.is_none()
        && requests.len() < config.max_group_requests
        && config.max_collection_delay > Duration::ZERO
    {
        let deadline = tokio::time::sleep(config.max_collection_delay);
        tokio::pin!(deadline);
        loop {
            tokio::select! {
                request = receiver.recv() => {
                    let Some(request) = request else { break };
                    let request_bytes = transaction_request_size(&request.request);
                    if can_add_main_parity_request(requests.len(), bytes, request_bytes, config) {
                        bytes = bytes.saturating_add(request_bytes);
                        requests.push(request);
                        if requests.len() >= config.max_group_requests {
                            break;
                        }
                    } else {
                        pending = Some(request);
                        break;
                    }
                }
                _ = &mut deadline => break,
            }
        }
    }

    (requests, pending, bytes)
}

struct BlinkAdapter {
    sender: tokio::sync::mpsc::Sender<BlinkWork>,
    store: Arc<Mutex<BlinkStore<BenchFile, BenchFile>>>,
    coordinator_metrics: Arc<Mutex<dodb_storage::CoordinatorMetrics>>,
    read_handle: Option<BlinkReadHandle>,
}

impl BlinkAdapter {
    fn start(
        store: BlinkStore<BenchFile, BenchFile>,
        config: CoordinatorConfig,
        collection_policy: CollectionPolicy,
    ) -> Self {
        Self::start_with_read_handle(store, config, collection_policy, None)
    }

    fn start_versioned(
        store: BlinkStore<BenchFile, BenchFile>,
        config: CoordinatorConfig,
        collection_policy: CollectionPolicy,
    ) -> Self {
        let read_handle = store.versioned_read_handle();
        Self::start_with_read_handle(store, config, collection_policy, Some(read_handle))
    }

    fn start_with_read_handle(
        store: BlinkStore<BenchFile, BenchFile>,
        config: CoordinatorConfig,
        collection_policy: CollectionPolicy,
        read_handle: Option<BlinkReadHandle>,
    ) -> Self {
        let store = Arc::new(Mutex::new(store));
        let coordinator_metrics = Arc::new(Mutex::new(dodb_storage::CoordinatorMetrics::default()));
        let (sender, mut receiver) =
            tokio::sync::mpsc::channel::<BlinkWork>(config.queue_capacity.max(1));
        let worker_store = Arc::clone(&store);
        let worker_metrics = Arc::clone(&coordinator_metrics);
        tokio::spawn(async move {
            let mut pending_work = None;
            loop {
                let first = match pending_work.take() {
                    Some(work) => Some(work),
                    None => receiver.recv().await,
                };
                let Some(first) = first else { break };
                let collection_started = Instant::now();
                let (batch, next_pending, group_bytes) = match collection_policy {
                    CollectionPolicy::Current => {
                        let mut batch = vec![first];
                        if config.max_collection_delay.is_zero() {
                            while batch.len() < config.max_group_requests {
                                match receiver.try_recv() {
                                    Ok(work) => batch.push(work),
                                    Err(tokio::sync::mpsc::error::TryRecvError::Empty)
                                    | Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                                        break;
                                    }
                                }
                            }
                        } else {
                            let deadline = tokio::time::Instant::from_std(
                                collection_started + config.max_collection_delay,
                            );
                            while batch.len() < config.max_group_requests {
                                match tokio::time::timeout_at(deadline, receiver.recv()).await {
                                    Ok(Some(work)) => batch.push(work),
                                    Ok(None) | Err(_) => break,
                                }
                            }
                        }
                        (batch, None, 0)
                    }
                    CollectionPolicy::MainParity => {
                        collect_main_parity_group(first, &mut receiver, config).await
                    }
                };
                pending_work = next_pending;
                let collection_nanos = collection_started.elapsed().as_nanos() as u64;
                let requests = batch
                    .iter()
                    .map(|work| work.request.clone())
                    .collect::<Vec<_>>();
                let processing_started = Instant::now();
                let results = match worker_store.lock() {
                    Ok(mut store) => {
                        let results = store.apply_transaction_group(&requests);
                        let threshold = CHECKPOINT_WAL_BYTES.load(Ordering::Relaxed);
                        let wal_bytes = store
                            .wal_metrics()
                            .ok()
                            .flatten()
                            .map_or(0, |metrics| metrics.wal_bytes);
                        if results.is_ok() && threshold > 0 && wal_bytes >= threshold {
                            results.and_then(|results| {
                                checkpoint_blink_store(&mut store).map(|()| results)
                            })
                        } else {
                            results
                        }
                    }
                    Err(_) => Err(Error::invariant("serial Blink benchmark mutex poisoned")),
                };
                let processing_nanos = processing_started.elapsed().as_nanos() as u64;
                if let Ok(mut metrics) = worker_metrics.lock() {
                    metrics.groups = metrics.groups.saturating_add(1);
                    metrics.queued_requests =
                        metrics.queued_requests.saturating_add(batch.len() as u64);
                    metrics.logical_transactions = metrics
                        .logical_transactions
                        .saturating_add(batch.len() as u64);
                    metrics.batch_collection_nanos = metrics
                        .batch_collection_nanos
                        .saturating_add(collection_nanos);
                    metrics.processing_nanos =
                        metrics.processing_nanos.saturating_add(processing_nanos);
                    metrics.max_group_requests = metrics.max_group_requests.max(batch.len());
                    metrics.max_group_bytes = metrics.max_group_bytes.max(group_bytes);
                }
                let results = match results {
                    Ok(results) => results,
                    Err(error) => batch
                        .iter()
                        .map(|_| Err(Error::durability(error.to_string())))
                        .collect(),
                };
                if let Ok(mut metrics) = worker_metrics.lock() {
                    for work in &batch {
                        metrics.queue_wait_nanos = metrics
                            .queue_wait_nanos
                            .saturating_add(work.enqueued.elapsed().as_nanos() as u64);
                    }
                }
                for (work, result) in batch.into_iter().zip(results) {
                    let _ = work.response.send(result);
                }
            }
        });
        Self {
            sender,
            store,
            coordinator_metrics,
            read_handle,
        }
    }
}

impl EngineAdapter for BlinkAdapter {
    fn execute_transaction<'a>(
        &'a self,
        request: TransactionRequest,
    ) -> BoxFuture<'a, Result<TransactionResult>> {
        let sender = self.sender.clone();
        Box::pin(async move {
            let (response, receiver) = tokio::sync::oneshot::channel();
            sender
                .try_send(BlinkWork {
                    request,
                    enqueued: Instant::now(),
                    response,
                })
                .map_err(|error| match error {
                    tokio::sync::mpsc::error::TrySendError::Full(_) => {
                        Error::overloaded("serial Blink benchmark queue is full")
                    }
                    tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                        Error::invariant("serial Blink benchmark coordinator stopped")
                    }
                })?;
            receiver
                .await
                .map_err(|_| Error::invariant("serial Blink benchmark response dropped"))?
        })
    }

    fn execute<'a>(&'a self, request: BatchRequest) -> BoxFuture<'a, Result<BatchResponse>> {
        if let Some(read_handle) = &self.read_handle {
            let read_handle = read_handle.clone();
            return Box::pin(async move {
                match request {
                    BatchRequest::Get { key } => Ok(BatchResponse::Get(read_handle.get(&key)?)),
                    BatchRequest::Query {
                        pk,
                        exclusive_after_sk,
                        limit,
                    } => Ok(BatchResponse::Query(read_handle.query(
                        &pk,
                        exclusive_after_sk.as_ref(),
                        limit,
                    )?)),
                    BatchRequest::Scan {
                        exclusive_after_key,
                        limit,
                    } => Ok(BatchResponse::Scan(
                        read_handle.scan(exclusive_after_key.as_ref(), limit)?,
                    )),
                    BatchRequest::Put { .. } | BatchRequest::Delete { .. } => {
                        Err(Error::invalid_request(
                            "versioned Blink benchmark read adapter received a mutation",
                        ))
                    }
                }
            });
        }
        let store = Arc::clone(&self.store);
        Box::pin(async move {
            let mut store = store
                .lock()
                .map_err(|_| Error::invariant("serial Blink benchmark mutex poisoned"))?;
            store
                .apply_batch(&[request])?
                .into_iter()
                .next()
                .ok_or_else(|| Error::invariant("serial Blink returned no response"))
        })
    }

    fn snapshot(&self) -> EngineSnapshot {
        let Ok(store) = self.store.lock() else {
            return EngineSnapshot {
                coordinator: dodb_storage::CoordinatorMetrics::default(),
                storage: None,
                wal: None,
                blink: None,
                batch: None,
                versioned: None,
                checkpoint: None,
                dirty_pages: None,
            };
        };
        EngineSnapshot {
            coordinator: self
                .coordinator_metrics
                .lock()
                .map(|metrics| metrics.clone())
                .unwrap_or_default(),
            storage: Some(store.storage_metrics()),
            wal: store.wal_metrics().ok().flatten(),
            blink: Some(store.split_metrics()),
            batch: Some(store.batch_metrics()),
            versioned: self
                .read_handle
                .as_ref()
                .map(|_| store.versioned_read_metrics()),
            checkpoint: Some(store.checkpoint_metrics()),
            dirty_pages: Some(store.dirty_page_count()),
        }
    }

    fn reset_checkpoint_metrics(&self) {
        if let Ok(mut store) = self.store.lock() {
            store.reset_checkpoint_metrics();
        }
    }

    fn shutdown<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        let store = Arc::clone(&self.store);
        Box::pin(async move {
            let mut store = store
                .lock()
                .map_err(|_| Error::invariant("serial Blink benchmark mutex poisoned"))?;
            store.flush()
        })
    }
}

#[derive(Clone, Debug, Default)]
struct LatencySamples {
    values: Vec<Duration>,
    seen: u64,
    state: u64,
}

impl LatencySamples {
    fn with_seed(seed: u64) -> Self {
        Self {
            values: Vec::new(),
            seen: 0,
            state: seed,
        }
    }

    fn push(&mut self, value: Duration) {
        self.seen = self.seen.saturating_add(1);
        if self.values.len() < LATENCY_RESERVOIR_LIMIT {
            self.values.push(value);
            return;
        }
        self.state = splitmix64(self.state);
        let index = (self.state % self.seen) as usize;
        if index < LATENCY_RESERVOIR_LIMIT {
            self.values[index] = value;
        }
    }

    fn percentile_us(&self, fraction: f64) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        let mut values = self.values.clone();
        values.sort_unstable();
        let index = ((values.len().saturating_sub(1)) as f64 * fraction).round() as usize;
        values[index].as_secs_f64() * 1_000_000.0
    }
}

#[derive(Clone, Debug)]
struct WorkerStats {
    attempted_transactions: u64,
    successful_transactions: u64,
    attempted_gets: u64,
    successful_gets: u64,
    attempted_queries: u64,
    successful_queries: u64,
    attempted_scans: u64,
    successful_scans: u64,
    returned_rows: u64,
    query_checked_requests: u64,
    query_checked_rows: u64,
    query_validation_failures: u64,
    query_ranges: HashSet<QueryRange>,
    mutation_ops: u64,
    conflicts: u64,
    overloads: u64,
    errors: u64,
    e2e_latency: LatencySamples,
    write_latency: LatencySamples,
    read_latency: LatencySamples,
    window_timeline: Vec<(u64, u64)>,
    client_completions: Vec<ClientCompletion>,
}

#[derive(Clone, Debug)]
struct ClientCompletion {
    role: &'static str,
    client_index: usize,
    attempted_operations: u64,
    successful_operations: u64,
    successful_queries: u64,
    returned_rows: u64,
    query_checked_requests: u64,
    query_checked_rows: u64,
    query_validation_failures: u64,
}

impl WorkerStats {
    fn new(seed: u64) -> Self {
        Self {
            attempted_transactions: 0,
            successful_transactions: 0,
            attempted_gets: 0,
            successful_gets: 0,
            attempted_queries: 0,
            successful_queries: 0,
            attempted_scans: 0,
            successful_scans: 0,
            returned_rows: 0,
            query_checked_requests: 0,
            query_checked_rows: 0,
            query_validation_failures: 0,
            query_ranges: HashSet::new(),
            mutation_ops: 0,
            conflicts: 0,
            overloads: 0,
            errors: 0,
            e2e_latency: LatencySamples::with_seed(seed),
            write_latency: LatencySamples::with_seed(seed ^ 0x1111),
            read_latency: LatencySamples::with_seed(seed ^ 0x2222),
            window_timeline: Vec::new(),
            client_completions: Vec::new(),
        }
    }

    fn merge(&mut self, other: Self) {
        self.attempted_transactions += other.attempted_transactions;
        self.successful_transactions += other.successful_transactions;
        self.attempted_gets += other.attempted_gets;
        self.successful_gets += other.successful_gets;
        self.attempted_queries += other.attempted_queries;
        self.successful_queries += other.successful_queries;
        self.attempted_scans += other.attempted_scans;
        self.successful_scans += other.successful_scans;
        self.returned_rows += other.returned_rows;
        self.query_checked_requests += other.query_checked_requests;
        self.query_checked_rows += other.query_checked_rows;
        self.query_validation_failures += other.query_validation_failures;
        self.query_ranges.extend(other.query_ranges);
        self.mutation_ops += other.mutation_ops;
        self.conflicts += other.conflicts;
        self.overloads += other.overloads;
        self.errors += other.errors;
        for value in other.e2e_latency.values {
            self.e2e_latency.push(value);
        }
        for value in other.write_latency.values {
            self.write_latency.push(value);
        }
        for value in other.read_latency.values {
            self.read_latency.push(value);
        }
        self.window_timeline.extend(other.window_timeline);
        self.client_completions.extend(other.client_completions);
    }

    fn attempted_operations(&self) -> u64 {
        self.attempted_transactions
            + self.attempted_gets
            + self.attempted_queries
            + self.attempted_scans
    }

    fn attempted_reads(&self) -> u64 {
        self.attempted_gets + self.attempted_queries + self.attempted_scans
    }

    fn successful_operations(&self) -> u64 {
        self.successful_transactions
            + self.successful_gets
            + self.successful_queries
            + self.successful_scans
    }

    fn successful_reads(&self) -> u64 {
        self.successful_gets + self.successful_queries + self.successful_scans
    }

    fn complete_client(&mut self, role: &'static str, client_index: usize) {
        self.client_completions.push(ClientCompletion {
            role,
            client_index,
            attempted_operations: self.attempted_operations(),
            successful_operations: self.successful_operations(),
            successful_queries: self.successful_queries,
            returned_rows: self.returned_rows,
            query_checked_requests: self.query_checked_requests,
            query_checked_rows: self.query_checked_rows,
            query_validation_failures: self.query_validation_failures,
        });
    }
}

fn percent(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        part as f64 * 100.0 / total as f64
    }
}

#[derive(Clone, Debug, Default)]
struct MetricDelta {
    groups: u64,
    queued_requests: u64,
    logical_transactions: u64,
    queue_wait_nanos: u64,
    collection_nanos: u64,
    processing_nanos: u64,
    max_group_requests: usize,
    max_group_bytes: usize,
    validation_nanos: u64,
    btree_preparation_nanos: u64,
    publication_nanos: u64,
    wal_bytes: u64,
    wal_syncs: u64,
    materializations: u64,
    materialization_total_nanos: u64,
    materialization_cpu_nanos: u64,
    materialization_max_nanos: u64,
    materialized_segments: u64,
    materialized_overlay_bytes: u64,
    materialized_data_bytes: u64,
    materialization_data_write_nanos: u64,
    materialization_data_sync_nanos: u64,
    materialization_checkpoint_nanos: u64,
    materialization_checkpoint_sync_nanos: u64,
    publish_pause_nanos: u64,
    writer_blocked_nanos: u64,
    backpressure_events: u64,
    backpressure_nanos: u64,
    materializer_requests: u64,
    materializer_completed: u64,
    materializer_failed: u64,
    materializer_stale: u64,
    overlay_segments_current: u64,
    overlay_segments_peak: u64,
    overlay_bytes_current: u64,
    overlay_bytes_peak: u64,
    wal_bytes_retained: u64,
    wal_bytes_reclaimed: u64,
    materialization_wal_bytes_reclaimed: u64,
    materialization_max_lag_transactions: u64,
    wal_committed_batches: u64,
    page_images: u64,
    wal_redo: WalRedoStats,
    wal_append_nanos: u64,
    wal_sync_nanos: u64,
    wal_group_encode_nanos: u64,
    wal_group_page_lsn_validate_nanos: u64,
    wal_group_page_image_materialize_nanos: u64,
    wal_group_page_image_validate_nanos: u64,
    wal_group_digest_copy_nanos: u64,
    wal_group_page_payload_crc_nanos: u64,
    wal_group_page_header_crc_nanos: u64,
    wal_group_page_frame_materialize_nanos: u64,
    wal_group_page_frame_append_nanos: u64,
    wal_group_page_direct_encode_nanos: u64,
    wal_group_commit_digest_crc_nanos: u64,
    wal_group_commit_payload_crc_nanos: u64,
    wal_group_commit_header_crc_nanos: u64,
    wal_group_commit_frame_materialize_nanos: u64,
    wal_group_commit_frame_append_nanos: u64,
    wal_group_commit_direct_encode_nanos: u64,
    wal_group_page_frames: u64,
    wal_group_commit_frames: u64,
    wal_group_page_validations: u64,
    wal_group_write_nanos: u64,
    wal_physical_write_calls: u64,
    leaf_splits: u64,
    internal_splits: u64,
    root_splits: u64,
    right_link_corrections: u64,
    pages_touched: u64,
    blink_page_images: u64,
    leaf_splits_total: u64,
    internal_splits_total: u64,
    root_splits_total: u64,
    right_link_corrections_total: u64,
    generation_pins: u64,
    read_operations: u64,
    page_version_installs: u64,
    versions_retained: u64,
    versions_reclaimed: u64,
    active_generation_pins: u64,
    max_concurrent_pins: u64,
    retired_page_ids: u64,
    reusable_page_ids: u64,
    versioned_right_link_corrections: u64,
    logical_groups: u64,
    admitted_transactions: u64,
    conflicted_transactions: u64,
    rejected_transactions: u64,
    logical_admission_nanos: u64,
    planning_nanos: u64,
    planner_route_nanos: u64,
    planner_route_calls: u64,
    planner_route_page_visits: u64,
    planner_route_right_link_hops: u64,
    state_clone_nanos: u64,
    physical_execution_nanos: u64,
    physical_mutation_nanos: u64,
    leaf_load_clone_nanos: u64,
    leaf_entries_clone_nanos: u64,
    leaf_install_clone_nanos: u64,
    physical_restamp_nanos: u64,
    physical_cached_refresh_nanos: u64,
    physical_page_encode_nanos: u64,
    physical_superblock_encode_nanos: u64,
    superblock_images_emitted: u64,
    superblock_images_elided: u64,
    leaf_load_clones: u64,
    leaf_entries_clones: u64,
    leaf_install_clones: u64,
    cached_refresh_clones: u64,
    dirty_union_nanos: u64,
    full_state_clones: u64,
    mutations_planned: u64,
    routes_calculated: u64,
    route_reuses: u64,
    route_invalidations: u64,
    reroutes: u64,
    leaf_groups: u64,
    same_leaf_groups: u64,
    mutations_per_leaf_group: u64,
    coalesced_mutations: u64,
    independent_leaf_groups: u64,
    dependency_edges: u64,
    leaf_loads: u64,
    leaf_encodes: u64,
    structural_fallbacks: u64,
    split_triggered_reroutes: u64,
    coalescing_interruptions: u64,
    planner_page_images: u64,
    planner_wal_bytes: u64,
    catalog_construction_nanos: u64,
    catalog_map_clone_nanos: u64,
    catalog_directory_clone_nanos: u64,
    catalog_chunk_clone_nanos: u64,
    catalog_chunk_clones: u64,
    catalog_state_scan_nanos: u64,
    wal_assembly_nanos: u64,
    state_install_nanos: u64,
    generation_publication_nanos: u64,
    publication_swap_nanos: u64,
    retired_generation_drop_nanos: u64,
    dirty_tracking_nanos: u64,
    parallel_groups: u64,
    parallel_leaf_jobs: u64,
    parallel_transactions: u64,
    parallel_mutations: u64,
    parallel_worker_dispatches: u64,
    parallel_background_worker_dispatches: u64,
    parallel_worker_nanos: u64,
    parallel_join_nanos: u64,
    parallel_fallback_groups: u64,
    parallel_job_operations: u64,
    parallel_dispatch_nanos: u64,
    parallel_collect_nanos: u64,
    parallel_worker_slot_nanos: u64,
    parallel_coordinator_lane_nanos: u64,
    parallel_worker_base_nanos: u64,
    parallel_worker_mutation_nanos: u64,
    parallel_worker_encode_nanos: u64,
    parallel_worker_delta_nanos: u64,
    parallel_fallback_after_dispatch: u64,
    parallel_fallback_no_delta_wal: u64,
    parallel_fallback_route: u64,
    parallel_fallback_overflow: u64,
    parallel_fallback_structural: u64,
    parallel_skipped_single_leaf: u64,
    parallel_skipped_small_group: u64,
    structural_transactions: u64,
    transaction_mutation_histogram: Vec<u64>,
    transaction_dirty_page_histogram: Vec<u64>,
    transaction_leaf_page_histogram: Vec<u64>,
    wal_redo_plan_nanos: u64,
}

impl MetricDelta {
    fn from(before: &EngineSnapshot, after: &EngineSnapshot) -> Self {
        let subtraction = |after: u64, before: u64| after.saturating_sub(before);
        let storage_before = before.storage.clone().unwrap_or_default();
        let storage_after = after.storage.clone().unwrap_or_default();
        let wal_before = before.wal.clone().unwrap_or_default();
        let wal_after = after.wal.clone().unwrap_or_default();
        let blink_before = before.blink.clone().unwrap_or_default();
        let blink_after = after.blink.clone().unwrap_or_default();
        let batch_before = before.batch.clone().unwrap_or_default();
        let batch_after = after.batch.clone().unwrap_or_default();
        let versioned_before = before.versioned.clone().unwrap_or_default();
        let versioned_after = after.versioned.clone().unwrap_or_default();
        let checkpoint_before = before.checkpoint.clone().unwrap_or_default();
        let checkpoint_after = after.checkpoint.clone().unwrap_or_default();
        Self {
            groups: subtraction(after.coordinator.groups, before.coordinator.groups),
            queued_requests: subtraction(
                after.coordinator.queued_requests,
                before.coordinator.queued_requests,
            ),
            logical_transactions: subtraction(
                after.coordinator.logical_transactions,
                before.coordinator.logical_transactions,
            ),
            queue_wait_nanos: subtraction(
                after.coordinator.queue_wait_nanos,
                before.coordinator.queue_wait_nanos,
            ),
            collection_nanos: subtraction(
                after.coordinator.batch_collection_nanos,
                before.coordinator.batch_collection_nanos,
            ),
            processing_nanos: subtraction(
                after.coordinator.processing_nanos,
                before.coordinator.processing_nanos,
            ),
            max_group_requests: after.coordinator.max_group_requests,
            max_group_bytes: after.coordinator.max_group_bytes,
            validation_nanos: subtraction(
                storage_after.validation_nanos,
                storage_before.validation_nanos,
            ),
            btree_preparation_nanos: subtraction(
                storage_after.btree_preparation_nanos,
                storage_before.btree_preparation_nanos,
            ),
            publication_nanos: subtraction(
                storage_after.publication_nanos,
                storage_before.publication_nanos,
            ),
            wal_bytes: wal_after.wal_bytes.saturating_sub(wal_before.wal_bytes),
            wal_syncs: subtraction(wal_after.wal_syncs, wal_before.wal_syncs),
            materializations: subtraction(
                checkpoint_after.materializations,
                checkpoint_before.materializations,
            ),
            materialization_total_nanos: subtraction(
                checkpoint_after.total_duration_nanos,
                checkpoint_before.total_duration_nanos,
            ),
            materialization_cpu_nanos: subtraction(
                checkpoint_after.cpu_nanos,
                checkpoint_before.cpu_nanos,
            ),
            materialization_max_nanos: checkpoint_after.max_duration_nanos,
            materialized_segments: subtraction(
                checkpoint_after.segments_materialized,
                checkpoint_before.segments_materialized,
            ),
            materialized_overlay_bytes: subtraction(
                checkpoint_after.overlay_bytes_materialized,
                checkpoint_before.overlay_bytes_materialized,
            ),
            materialized_data_bytes: subtraction(
                checkpoint_after.bytes_written,
                checkpoint_before.bytes_written,
            ),
            materialization_data_write_nanos: subtraction(
                checkpoint_after.data_write_nanos,
                checkpoint_before.data_write_nanos,
            ),
            materialization_data_sync_nanos: subtraction(
                checkpoint_after.data_sync_nanos,
                checkpoint_before.data_sync_nanos,
            ),
            materialization_checkpoint_nanos: subtraction(
                checkpoint_after.checkpoint_nanos,
                checkpoint_before.checkpoint_nanos,
            ),
            materialization_checkpoint_sync_nanos: subtraction(
                checkpoint_after.checkpoint_sync_nanos,
                checkpoint_before.checkpoint_sync_nanos,
            ),
            publish_pause_nanos: subtraction(
                checkpoint_after.publish_pause_nanos,
                checkpoint_before.publish_pause_nanos,
            ),
            writer_blocked_nanos: subtraction(
                checkpoint_after.writer_blocked_nanos,
                checkpoint_before.writer_blocked_nanos,
            ),
            backpressure_events: subtraction(
                checkpoint_after.backpressure_events,
                checkpoint_before.backpressure_events,
            ),
            backpressure_nanos: subtraction(
                checkpoint_after.backpressure_nanos,
                checkpoint_before.backpressure_nanos,
            ),
            materializer_requests: subtraction(
                checkpoint_after.requests,
                checkpoint_before.requests,
            ),
            materializer_completed: subtraction(
                checkpoint_after.completed,
                checkpoint_before.completed,
            ),
            materializer_failed: subtraction(checkpoint_after.failed, checkpoint_before.failed),
            materializer_stale: subtraction(checkpoint_after.stale, checkpoint_before.stale),
            overlay_segments_current: checkpoint_after.overlay_segments_current,
            overlay_segments_peak: checkpoint_after.overlay_segments_peak,
            overlay_bytes_current: checkpoint_after.overlay_bytes_current,
            overlay_bytes_peak: checkpoint_after.overlay_bytes_peak,
            wal_bytes_retained: checkpoint_after.wal_bytes_retained,
            wal_bytes_reclaimed: subtraction(
                checkpoint_after.wal_bytes_reclaimed_total,
                checkpoint_before.wal_bytes_reclaimed_total,
            ),
            materialization_wal_bytes_reclaimed: subtraction(
                checkpoint_after.wal_bytes_reclaimed,
                checkpoint_before.wal_bytes_reclaimed,
            ),
            materialization_max_lag_transactions: checkpoint_after
                .max_materialization_lag_transactions,
            wal_committed_batches: subtraction(
                wal_after.committed_batches as u64,
                wal_before.committed_batches as u64,
            ),
            page_images: subtraction(wal_after.page_images as u64, wal_before.page_images as u64),
            wal_redo: redo_delta(&wal_after.redo, &wal_before.redo),
            wal_redo_plan_nanos: subtraction(wal_after.redo_plan_nanos, wal_before.redo_plan_nanos),
            wal_append_nanos: subtraction(wal_after.append_nanos, wal_before.append_nanos),
            wal_sync_nanos: subtraction(wal_after.sync_nanos, wal_before.sync_nanos),
            wal_group_encode_nanos: subtraction(
                wal_after.group_encode_nanos,
                wal_before.group_encode_nanos,
            ),
            wal_group_page_lsn_validate_nanos: subtraction(
                wal_after.group_page_lsn_validate_nanos,
                wal_before.group_page_lsn_validate_nanos,
            ),
            wal_group_page_image_materialize_nanos: subtraction(
                wal_after.group_page_image_materialize_nanos,
                wal_before.group_page_image_materialize_nanos,
            ),
            wal_group_page_image_validate_nanos: subtraction(
                wal_after.group_page_image_validate_nanos,
                wal_before.group_page_image_validate_nanos,
            ),
            wal_group_digest_copy_nanos: subtraction(
                wal_after.group_digest_copy_nanos,
                wal_before.group_digest_copy_nanos,
            ),
            wal_group_page_payload_crc_nanos: subtraction(
                wal_after.group_page_payload_crc_nanos,
                wal_before.group_page_payload_crc_nanos,
            ),
            wal_group_page_header_crc_nanos: subtraction(
                wal_after.group_page_header_crc_nanos,
                wal_before.group_page_header_crc_nanos,
            ),
            wal_group_page_frame_materialize_nanos: subtraction(
                wal_after.group_page_frame_materialize_nanos,
                wal_before.group_page_frame_materialize_nanos,
            ),
            wal_group_page_frame_append_nanos: subtraction(
                wal_after.group_page_frame_append_nanos,
                wal_before.group_page_frame_append_nanos,
            ),
            wal_group_page_direct_encode_nanos: subtraction(
                wal_after.group_page_direct_encode_nanos,
                wal_before.group_page_direct_encode_nanos,
            ),
            wal_group_commit_digest_crc_nanos: subtraction(
                wal_after.group_commit_digest_crc_nanos,
                wal_before.group_commit_digest_crc_nanos,
            ),
            wal_group_commit_payload_crc_nanos: subtraction(
                wal_after.group_commit_payload_crc_nanos,
                wal_before.group_commit_payload_crc_nanos,
            ),
            wal_group_commit_header_crc_nanos: subtraction(
                wal_after.group_commit_header_crc_nanos,
                wal_before.group_commit_header_crc_nanos,
            ),
            wal_group_commit_frame_materialize_nanos: subtraction(
                wal_after.group_commit_frame_materialize_nanos,
                wal_before.group_commit_frame_materialize_nanos,
            ),
            wal_group_commit_frame_append_nanos: subtraction(
                wal_after.group_commit_frame_append_nanos,
                wal_before.group_commit_frame_append_nanos,
            ),
            wal_group_commit_direct_encode_nanos: subtraction(
                wal_after.group_commit_direct_encode_nanos,
                wal_before.group_commit_direct_encode_nanos,
            ),
            wal_group_page_frames: subtraction(
                wal_after.group_page_frames,
                wal_before.group_page_frames,
            ),
            wal_group_commit_frames: subtraction(
                wal_after.group_commit_frames,
                wal_before.group_commit_frames,
            ),
            wal_group_page_validations: subtraction(
                wal_after.group_page_validations,
                wal_before.group_page_validations,
            ),
            wal_group_write_nanos: subtraction(
                wal_after.group_write_nanos,
                wal_before.group_write_nanos,
            ),
            wal_physical_write_calls: subtraction(
                wal_after.physical_write_calls,
                wal_before.physical_write_calls,
            ),
            leaf_splits: subtraction(blink_after.leaf_splits, blink_before.leaf_splits),
            internal_splits: subtraction(blink_after.internal_splits, blink_before.internal_splits),
            root_splits: subtraction(blink_after.root_splits, blink_before.root_splits),
            right_link_corrections: subtraction(
                blink_after.right_link_corrections,
                blink_before.right_link_corrections,
            ),
            pages_touched: subtraction(blink_after.pages_touched, blink_before.pages_touched),
            blink_page_images: subtraction(blink_after.page_images, blink_before.page_images),
            leaf_splits_total: blink_after.leaf_splits,
            internal_splits_total: blink_after.internal_splits,
            root_splits_total: blink_after.root_splits,
            right_link_corrections_total: blink_after.right_link_corrections,
            generation_pins: subtraction(
                versioned_after.generation_pins,
                versioned_before.generation_pins,
            ),
            read_operations: subtraction(
                versioned_after.read_operations,
                versioned_before.read_operations,
            ),
            page_version_installs: subtraction(
                versioned_after.page_version_installs,
                versioned_before.page_version_installs,
            ),
            versions_retained: versioned_after.versions_retained,
            versions_reclaimed: subtraction(
                versioned_after.versions_reclaimed,
                versioned_before.versions_reclaimed,
            ),
            active_generation_pins: versioned_after.active_generation_pins,
            max_concurrent_pins: versioned_after.max_concurrent_pins,
            retired_page_ids: versioned_after.retired_page_ids,
            reusable_page_ids: versioned_after.reusable_page_ids,
            versioned_right_link_corrections: subtraction(
                versioned_after.right_link_corrections,
                versioned_before.right_link_corrections,
            ),
            logical_groups: subtraction(batch_after.logical_groups, batch_before.logical_groups),
            admitted_transactions: subtraction(
                batch_after.admitted_transactions,
                batch_before.admitted_transactions,
            ),
            conflicted_transactions: subtraction(
                batch_after.conflicted_transactions,
                batch_before.conflicted_transactions,
            ),
            rejected_transactions: subtraction(
                batch_after.rejected_transactions,
                batch_before.rejected_transactions,
            ),
            logical_admission_nanos: subtraction(
                batch_after.logical_admission_nanos,
                batch_before.logical_admission_nanos,
            ),
            planning_nanos: subtraction(batch_after.planning_nanos, batch_before.planning_nanos),
            planner_route_nanos: subtraction(
                batch_after.planner_route_nanos,
                batch_before.planner_route_nanos,
            ),
            planner_route_calls: subtraction(
                batch_after.planner_route_calls,
                batch_before.planner_route_calls,
            ),
            planner_route_page_visits: subtraction(
                batch_after.planner_route_page_visits,
                batch_before.planner_route_page_visits,
            ),
            planner_route_right_link_hops: subtraction(
                batch_after.planner_route_right_link_hops,
                batch_before.planner_route_right_link_hops,
            ),
            state_clone_nanos: subtraction(
                batch_after.state_clone_nanos,
                batch_before.state_clone_nanos,
            ),
            physical_execution_nanos: subtraction(
                batch_after.physical_execution_nanos,
                batch_before.physical_execution_nanos,
            ),
            physical_mutation_nanos: subtraction(
                batch_after.physical_mutation_nanos,
                batch_before.physical_mutation_nanos,
            ),
            leaf_load_clone_nanos: subtraction(
                batch_after.leaf_load_clone_nanos,
                batch_before.leaf_load_clone_nanos,
            ),
            leaf_entries_clone_nanos: subtraction(
                batch_after.leaf_entries_clone_nanos,
                batch_before.leaf_entries_clone_nanos,
            ),
            leaf_install_clone_nanos: subtraction(
                batch_after.leaf_install_clone_nanos,
                batch_before.leaf_install_clone_nanos,
            ),
            physical_restamp_nanos: subtraction(
                batch_after.physical_restamp_nanos,
                batch_before.physical_restamp_nanos,
            ),
            physical_cached_refresh_nanos: subtraction(
                batch_after.physical_cached_refresh_nanos,
                batch_before.physical_cached_refresh_nanos,
            ),
            physical_page_encode_nanos: subtraction(
                batch_after.physical_page_encode_nanos,
                batch_before.physical_page_encode_nanos,
            ),
            physical_superblock_encode_nanos: subtraction(
                batch_after.physical_superblock_encode_nanos,
                batch_before.physical_superblock_encode_nanos,
            ),
            superblock_images_emitted: subtraction(
                batch_after.superblock_images_emitted,
                batch_before.superblock_images_emitted,
            ),
            superblock_images_elided: subtraction(
                batch_after.superblock_images_elided,
                batch_before.superblock_images_elided,
            ),
            leaf_load_clones: subtraction(
                batch_after.leaf_load_clones,
                batch_before.leaf_load_clones,
            ),
            leaf_entries_clones: subtraction(
                batch_after.leaf_entries_clones,
                batch_before.leaf_entries_clones,
            ),
            leaf_install_clones: subtraction(
                batch_after.leaf_install_clones,
                batch_before.leaf_install_clones,
            ),
            cached_refresh_clones: subtraction(
                batch_after.cached_refresh_clones,
                batch_before.cached_refresh_clones,
            ),
            dirty_union_nanos: subtraction(
                batch_after.dirty_union_nanos,
                batch_before.dirty_union_nanos,
            ),
            full_state_clones: subtraction(
                batch_after.full_state_clones,
                batch_before.full_state_clones,
            ),
            mutations_planned: subtraction(
                batch_after.mutations_planned,
                batch_before.mutations_planned,
            ),
            routes_calculated: subtraction(
                batch_after.routes_calculated,
                batch_before.routes_calculated,
            ),
            route_reuses: subtraction(batch_after.route_reuses, batch_before.route_reuses),
            route_invalidations: subtraction(
                batch_after.route_invalidations,
                batch_before.route_invalidations,
            ),
            reroutes: subtraction(batch_after.reroutes, batch_before.reroutes),
            leaf_groups: subtraction(batch_after.leaf_groups, batch_before.leaf_groups),
            same_leaf_groups: subtraction(
                batch_after.same_leaf_groups,
                batch_before.same_leaf_groups,
            ),
            mutations_per_leaf_group: subtraction(
                batch_after.mutations_per_leaf_group,
                batch_before.mutations_per_leaf_group,
            ),
            coalesced_mutations: subtraction(
                batch_after.coalesced_mutations,
                batch_before.coalesced_mutations,
            ),
            independent_leaf_groups: subtraction(
                batch_after.independent_leaf_groups,
                batch_before.independent_leaf_groups,
            ),
            dependency_edges: subtraction(
                batch_after.dependency_edges,
                batch_before.dependency_edges,
            ),
            leaf_loads: subtraction(batch_after.leaf_loads, batch_before.leaf_loads),
            leaf_encodes: subtraction(batch_after.leaf_encodes, batch_before.leaf_encodes),
            structural_fallbacks: subtraction(
                batch_after.structural_fallbacks,
                batch_before.structural_fallbacks,
            ),
            split_triggered_reroutes: subtraction(
                batch_after.split_triggered_reroutes,
                batch_before.split_triggered_reroutes,
            ),
            coalescing_interruptions: subtraction(
                batch_after.coalescing_interruptions,
                batch_before.coalescing_interruptions,
            ),
            planner_page_images: subtraction(batch_after.page_images, batch_before.page_images),
            planner_wal_bytes: subtraction(batch_after.wal_bytes, batch_before.wal_bytes),
            catalog_construction_nanos: subtraction(
                batch_after.catalog_construction_nanos,
                batch_before.catalog_construction_nanos,
            ),
            catalog_map_clone_nanos: subtraction(
                batch_after.catalog_map_clone_nanos,
                batch_before.catalog_map_clone_nanos,
            ),
            catalog_directory_clone_nanos: subtraction(
                batch_after.catalog_directory_clone_nanos,
                batch_before.catalog_directory_clone_nanos,
            ),
            catalog_chunk_clone_nanos: subtraction(
                batch_after.catalog_chunk_clone_nanos,
                batch_before.catalog_chunk_clone_nanos,
            ),
            catalog_chunk_clones: subtraction(
                batch_after.catalog_chunk_clones,
                batch_before.catalog_chunk_clones,
            ),
            catalog_state_scan_nanos: subtraction(
                batch_after.catalog_state_scan_nanos,
                batch_before.catalog_state_scan_nanos,
            ),
            wal_assembly_nanos: subtraction(
                batch_after.wal_assembly_nanos,
                batch_before.wal_assembly_nanos,
            ),
            state_install_nanos: subtraction(
                batch_after.state_install_nanos,
                batch_before.state_install_nanos,
            ),
            generation_publication_nanos: subtraction(
                batch_after.generation_publication_nanos,
                batch_before.generation_publication_nanos,
            ),
            publication_swap_nanos: subtraction(
                batch_after.publication_swap_nanos,
                batch_before.publication_swap_nanos,
            ),
            retired_generation_drop_nanos: subtraction(
                batch_after.retired_generation_drop_nanos,
                batch_before.retired_generation_drop_nanos,
            ),
            dirty_tracking_nanos: subtraction(
                batch_after.dirty_tracking_nanos,
                batch_before.dirty_tracking_nanos,
            ),
            parallel_groups: subtraction(batch_after.parallel_groups, batch_before.parallel_groups),
            parallel_leaf_jobs: subtraction(
                batch_after.parallel_leaf_jobs,
                batch_before.parallel_leaf_jobs,
            ),
            parallel_transactions: subtraction(
                batch_after.parallel_transactions,
                batch_before.parallel_transactions,
            ),
            parallel_mutations: subtraction(
                batch_after.parallel_mutations,
                batch_before.parallel_mutations,
            ),
            parallel_worker_dispatches: subtraction(
                batch_after.parallel_worker_dispatches,
                batch_before.parallel_worker_dispatches,
            ),
            parallel_background_worker_dispatches: subtraction(
                batch_after.parallel_background_worker_dispatches,
                batch_before.parallel_background_worker_dispatches,
            ),
            parallel_worker_nanos: subtraction(
                batch_after.parallel_worker_nanos,
                batch_before.parallel_worker_nanos,
            ),
            parallel_join_nanos: subtraction(
                batch_after.parallel_join_nanos,
                batch_before.parallel_join_nanos,
            ),
            parallel_fallback_groups: subtraction(
                batch_after.parallel_fallback_groups,
                batch_before.parallel_fallback_groups,
            ),
            parallel_job_operations: subtraction(
                batch_after.parallel_job_operations,
                batch_before.parallel_job_operations,
            ),
            parallel_dispatch_nanos: subtraction(
                batch_after.parallel_dispatch_nanos,
                batch_before.parallel_dispatch_nanos,
            ),
            parallel_collect_nanos: subtraction(
                batch_after.parallel_collect_nanos,
                batch_before.parallel_collect_nanos,
            ),
            parallel_worker_slot_nanos: subtraction(
                batch_after.parallel_worker_slot_nanos,
                batch_before.parallel_worker_slot_nanos,
            ),
            parallel_coordinator_lane_nanos: subtraction(
                batch_after.parallel_coordinator_lane_nanos,
                batch_before.parallel_coordinator_lane_nanos,
            ),
            parallel_worker_base_nanos: subtraction(
                batch_after.parallel_worker_base_nanos,
                batch_before.parallel_worker_base_nanos,
            ),
            parallel_worker_mutation_nanos: subtraction(
                batch_after.parallel_worker_mutation_nanos,
                batch_before.parallel_worker_mutation_nanos,
            ),
            parallel_worker_encode_nanos: subtraction(
                batch_after.parallel_worker_encode_nanos,
                batch_before.parallel_worker_encode_nanos,
            ),
            parallel_worker_delta_nanos: subtraction(
                batch_after.parallel_worker_delta_nanos,
                batch_before.parallel_worker_delta_nanos,
            ),
            parallel_fallback_after_dispatch: subtraction(
                batch_after.parallel_fallback_after_dispatch,
                batch_before.parallel_fallback_after_dispatch,
            ),
            parallel_fallback_no_delta_wal: subtraction(
                batch_after.parallel_fallback_no_delta_wal,
                batch_before.parallel_fallback_no_delta_wal,
            ),
            parallel_fallback_route: subtraction(
                batch_after.parallel_fallback_route,
                batch_before.parallel_fallback_route,
            ),
            parallel_fallback_overflow: subtraction(
                batch_after.parallel_fallback_overflow,
                batch_before.parallel_fallback_overflow,
            ),
            parallel_fallback_structural: subtraction(
                batch_after.parallel_fallback_structural,
                batch_before.parallel_fallback_structural,
            ),
            parallel_skipped_small_group: subtraction(
                batch_after.parallel_skipped_small_group,
                batch_before.parallel_skipped_small_group,
            ),
            parallel_skipped_single_leaf: subtraction(
                batch_after.parallel_skipped_single_leaf,
                batch_before.parallel_skipped_single_leaf,
            ),
            structural_transactions: subtraction(
                batch_after.structural_transactions,
                batch_before.structural_transactions,
            ),
            transaction_mutation_histogram: histogram_delta(
                &batch_after.transaction_mutation_histogram,
                &batch_before.transaction_mutation_histogram,
            ),
            transaction_dirty_page_histogram: histogram_delta(
                &batch_after.transaction_dirty_page_histogram,
                &batch_before.transaction_dirty_page_histogram,
            ),
            transaction_leaf_page_histogram: histogram_delta(
                &batch_after.transaction_leaf_page_histogram,
                &batch_before.transaction_leaf_page_histogram,
            ),
        }
    }

    fn avg_group_requests(&self) -> f64 {
        self.queued_requests as f64 / self.groups.max(1) as f64
    }

    fn avg_transactions_per_group(&self) -> f64 {
        self.logical_transactions as f64 / self.groups.max(1) as f64
    }

    fn transactions_per_sync(&self) -> f64 {
        self.wal_committed_batches as f64 / self.wal_syncs.max(1) as f64
    }
}

#[derive(Clone, Debug)]
struct ProcessCpuSample {
    ticks: Option<u64>,
    ticks_per_second: u64,
}

impl ProcessCpuSample {
    fn capture() -> Self {
        Self {
            ticks: process_cpu_ticks(),
            ticks_per_second: clock_ticks_per_second(),
        }
    }

    fn utilization(
        &self,
        end: &Self,
        wall: Duration,
        logical_cpus: usize,
    ) -> (Option<f64>, Option<f64>) {
        let Some(start) = self.ticks else {
            return (None, None);
        };
        let Some(end) = end.ticks else {
            return (None, None);
        };
        if end < start || self.ticks_per_second == 0 || wall.is_zero() {
            return (None, None);
        }
        let cpu_seconds = (end - start) as f64 / self.ticks_per_second as f64;
        let one_core = cpu_seconds / wall.as_secs_f64() * 100.0;
        let machine = one_core / logical_cpus.max(1) as f64;
        (Some(one_core), Some(machine))
    }
}

const RSS_SAMPLE_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug)]
struct RssWindowMeasurement {
    supported: bool,
    status: &'static str,
    start_rss_bytes: Option<u64>,
    end_rss_bytes: Option<u64>,
    peak_rss_bytes: Option<u64>,
    sample_count: u64,
    requested_sample_interval: Duration,
    max_sample_interval: Option<Duration>,
    collection_failures: u64,
    start_sample_offset: Option<Duration>,
    end_sample_offset: Option<Duration>,
    peak_sample_offset: Option<Duration>,
}

#[derive(Clone, Copy, Debug)]
struct RssSample {
    sampled_at: Instant,
    resident_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
enum RssSamplerCommand {
    Start(Instant),
    Stop(Instant),
}

struct RssWindowAccumulator {
    started_at: Instant,
    finished_at: Option<Instant>,
    requested_sample_interval: Duration,
    supported: bool,
    first_sample: Option<RssSample>,
    last_sample: Option<RssSample>,
    peak_sample: Option<RssSample>,
    sample_count: u64,
    max_sample_interval: Option<Duration>,
    previous_attempt_at: Option<Instant>,
    collection_failures: u64,
}

impl RssWindowAccumulator {
    fn new(started_at: Instant, requested_sample_interval: Duration, supported: bool) -> Self {
        Self {
            started_at,
            finished_at: None,
            requested_sample_interval,
            supported,
            first_sample: None,
            last_sample: None,
            peak_sample: None,
            sample_count: 0,
            max_sample_interval: None,
            previous_attempt_at: None,
            collection_failures: 0,
        }
    }

    fn record(
        &mut self,
        sampled_at: Instant,
        resident_bytes: std::result::Result<u64, ()>,
    ) -> bool {
        if sampled_at < self.started_at
            || self
                .finished_at
                .is_some_and(|finished_at| sampled_at > finished_at)
        {
            return false;
        }
        if let Some(previous_attempt_at) = self.previous_attempt_at {
            let sample_interval = sampled_at.saturating_duration_since(previous_attempt_at);
            self.max_sample_interval = Some(
                self.max_sample_interval
                    .map_or(sample_interval, |maximum| maximum.max(sample_interval)),
            );
        }
        self.previous_attempt_at = Some(sampled_at);
        let Ok(resident_bytes) = resident_bytes else {
            self.collection_failures += 1;
            return true;
        };
        if resident_bytes == 0 {
            self.collection_failures += 1;
            return true;
        }
        let sample = RssSample {
            sampled_at,
            resident_bytes,
        };
        self.first_sample.get_or_insert(sample);
        self.last_sample = Some(sample);
        if self
            .peak_sample
            .is_none_or(|peak_sample| resident_bytes > peak_sample.resident_bytes)
        {
            self.peak_sample = Some(sample);
        }
        self.sample_count += 1;
        true
    }

    fn finish(mut self, finished_at: Instant) -> RssWindowMeasurement {
        self.finished_at = Some(finished_at);
        let status = if !self.supported {
            "unsupported"
        } else if self.sample_count == 0 {
            "failed"
        } else if self.collection_failures > 0 {
            "partial"
        } else {
            "complete"
        };
        RssWindowMeasurement {
            supported: self.supported,
            status,
            start_rss_bytes: self.first_sample.map(|sample| sample.resident_bytes),
            end_rss_bytes: self.last_sample.map(|sample| sample.resident_bytes),
            peak_rss_bytes: self.peak_sample.map(|sample| sample.resident_bytes),
            sample_count: self.sample_count,
            requested_sample_interval: self.requested_sample_interval,
            max_sample_interval: self.max_sample_interval,
            collection_failures: self.collection_failures,
            start_sample_offset: self
                .first_sample
                .map(|sample| sample.sampled_at.saturating_duration_since(self.started_at)),
            end_sample_offset: self
                .last_sample
                .map(|sample| sample.sampled_at.saturating_duration_since(self.started_at)),
            peak_sample_offset: self
                .peak_sample
                .map(|sample| sample.sampled_at.saturating_duration_since(self.started_at)),
        }
    }
}

struct ProcessRssSampler {
    sender: SyncSender<RssSamplerCommand>,
    stop_at: Arc<Mutex<Option<Instant>>>,
    worker: Option<JoinHandle<RssWindowAccumulator>>,
    started_at: Instant,
    requested_sample_interval: Duration,
    supported: bool,
    start_sent: bool,
}

impl ProcessRssSampler {
    fn spawn(requested_sample_interval: Duration) -> Self {
        let supported = process_rss_supported();
        let (sender, receiver) = mpsc::sync_channel(1);
        let stop_at = Arc::new(Mutex::new(None));
        let worker_stop_at = Arc::clone(&stop_at);
        let worker = std::thread::spawn(move || {
            collect_process_rss_samples(
                receiver,
                worker_stop_at,
                requested_sample_interval,
                supported,
            )
        });
        Self {
            sender,
            stop_at,
            worker: Some(worker),
            started_at: Instant::now(),
            requested_sample_interval,
            supported,
            start_sent: false,
        }
    }

    fn start(&mut self, started_at: Instant) {
        self.started_at = started_at;
        self.start_sent = self
            .sender
            .send(RssSamplerCommand::Start(started_at))
            .is_ok();
    }

    fn finish(mut self) -> (Instant, RssWindowMeasurement) {
        let finished_at = {
            let mut stop_at = self
                .stop_at
                .lock()
                .expect("RSS sampler stop state should not be poisoned");
            let finished_at = Instant::now();
            *stop_at = Some(finished_at);
            finished_at
        };
        let _ = self.sender.send(RssSamplerCommand::Stop(finished_at));
        if !self.start_sent {
            let mut accumulator = RssWindowAccumulator::new(
                self.started_at,
                self.requested_sample_interval,
                self.supported,
            );
            accumulator.record(finished_at, Err(()));
            return (finished_at, accumulator.finish(finished_at));
        }
        let accumulator = match self.worker.take().unwrap().join() {
            Ok(accumulator) => accumulator,
            Err(_) => {
                let mut accumulator = RssWindowAccumulator::new(
                    self.started_at,
                    self.requested_sample_interval,
                    self.supported,
                );
                accumulator.record(finished_at, Err(()));
                accumulator
            }
        };
        (finished_at, accumulator.finish(finished_at))
    }
}

fn collect_process_rss_samples(
    receiver: Receiver<RssSamplerCommand>,
    stop_at: Arc<Mutex<Option<Instant>>>,
    requested_sample_interval: Duration,
    supported: bool,
) -> RssWindowAccumulator {
    let started_at = match receiver.recv() {
        Ok(RssSamplerCommand::Start(started_at)) => started_at,
        Ok(RssSamplerCommand::Stop(finished_at)) => {
            return RssWindowAccumulator::new(finished_at, requested_sample_interval, supported);
        }
        Err(_) => {
            return RssWindowAccumulator::new(Instant::now(), requested_sample_interval, supported);
        }
    };
    let mut accumulator =
        RssWindowAccumulator::new(started_at, requested_sample_interval, supported);
    if !supported {
        let _ = receiver.recv();
        return accumulator;
    }
    let mut next_sample_at = started_at;
    loop {
        let current_time = Instant::now();
        if stop_at
            .lock()
            .expect("RSS sampler stop state should not be poisoned")
            .is_some_and(|finished_at| current_time >= finished_at)
        {
            break;
        }
        let wait = next_sample_at.saturating_duration_since(current_time);
        if !wait.is_zero() {
            match receiver.recv_timeout(wait) {
                Ok(RssSamplerCommand::Start(_)) => continue,
                Ok(RssSamplerCommand::Stop(_)) => {}
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        let attempt_started_at = Instant::now();
        if stop_at
            .lock()
            .expect("RSS sampler stop state should not be poisoned")
            .is_some_and(|finished_at| attempt_started_at >= finished_at)
        {
            break;
        }
        let rss_result = process_rss_bytes();
        let sampled_at = Instant::now();
        let finished_at = *stop_at
            .lock()
            .expect("RSS sampler stop state should not be poisoned");
        if !finished_at.is_some_and(|finished_at| sampled_at > finished_at) {
            accumulator.record(sampled_at, rss_result);
        }
        if finished_at.is_some_and(|finished_at| sampled_at >= finished_at) {
            break;
        }
        next_sample_at = attempt_started_at + requested_sample_interval;
        if next_sample_at <= sampled_at {
            next_sample_at = sampled_at + requested_sample_interval;
        }
    }
    accumulator
}

fn process_rss_supported() -> bool {
    cfg!(any(target_os = "macos", target_os = "linux"))
}

fn process_rss_method() -> &'static str {
    if cfg!(target_os = "macos") {
        "libc proc_pidinfo PROC_PIDTASKINFO pti_resident_size bytes"
    } else if cfg!(target_os = "linux") {
        "Linux /proc/self/statm resident pages multiplied by sysconf page size"
    } else {
        "unsupported target"
    }
}

fn process_rss_bytes() -> std::result::Result<u64, ()> {
    #[cfg(target_os = "macos")]
    {
        let mut task_info = std::mem::MaybeUninit::<libc::proc_taskinfo>::zeroed();
        let task_info_size = std::mem::size_of::<libc::proc_taskinfo>() as libc::c_int;
        let result = unsafe {
            libc::proc_pidinfo(
                libc::getpid(),
                libc::PROC_PIDTASKINFO,
                0,
                task_info.as_mut_ptr().cast(),
                task_info_size,
            )
        };
        if result != task_info_size {
            return Err(());
        }
        let resident_bytes = unsafe { task_info.assume_init() }.pti_resident_size;
        return (resident_bytes > 0).then_some(resident_bytes).ok_or(());
    }

    #[cfg(target_os = "linux")]
    {
        let statm = std::fs::read_to_string("/proc/self/statm").map_err(|_| ())?;
        let resident_pages = statm
            .split_whitespace()
            .nth(1)
            .ok_or(())?
            .parse::<u64>()
            .map_err(|_| ())?;
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        if page_size <= 0 {
            return Err(());
        }
        let resident_bytes = resident_pages_to_bytes(resident_pages, page_size as u64).ok_or(())?;
        return (resident_bytes > 0).then_some(resident_bytes).ok_or(());
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(())
    }
}

#[cfg(any(target_os = "linux", test))]
fn resident_pages_to_bytes(resident_pages: u64, page_size_bytes: u64) -> Option<u64> {
    resident_pages.checked_mul(page_size_bytes)
}

#[derive(Clone, Debug)]
struct MachineInfo {
    cpu_model: String,
    logical_cpus: usize,
    cpu_cores: Option<usize>,
    os: String,
    kernel: String,
    rust_version: String,
}

impl MachineInfo {
    fn collect() -> Self {
        let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let cpu_model = if cfg!(target_os = "macos") {
            command_output("sysctl", &["-n", "machdep.cpu.brand_string"])
        } else {
            cpuinfo
                .lines()
                .find_map(|line| line.strip_prefix("model name\t: "))
                .or_else(|| {
                    cpuinfo
                        .lines()
                        .find_map(|line| line.strip_prefix("Model\t\t: "))
                })
                .unwrap_or("unknown")
                .to_owned()
        };
        let cpu_cores = if cfg!(target_os = "macos") {
            command_output("sysctl", &["-n", "hw.physicalcpu"])
                .parse()
                .ok()
        } else {
            cpuinfo
                .lines()
                .find_map(|line| line.strip_prefix("cpu cores\t: "))
                .and_then(|value| value.parse().ok())
        };
        let os = if cfg!(target_os = "macos") {
            let product_name = command_output("sw_vers", &["-productName"]);
            let product_version = command_output("sw_vers", &["-productVersion"]);
            format!("{product_name} {product_version}")
        } else {
            std::fs::read_to_string("/etc/os-release")
                .ok()
                .and_then(|contents| {
                    contents
                        .lines()
                        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
                        .map(|value| value.trim_matches('"').to_owned())
                })
                .unwrap_or_else(|| "unknown".to_owned())
        };
        Self {
            cpu_model,
            logical_cpus: std::thread::available_parallelism()
                .map_or(1, std::num::NonZeroUsize::get),
            cpu_cores,
            os,
            kernel: command_output("uname", &["-srvm"]),
            rust_version: command_output("rustc", &["--version"]),
        }
    }
}

fn command_output(command: &str, args: &[&str]) -> String {
    Command::new(command)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|output| !output.is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
}

fn process_cpu_ticks() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
        let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
        if result != 0 {
            return None;
        }
        let usage = unsafe { usage.assume_init() };
        let user_micros = usage.ru_utime.tv_sec as u64 * 1_000_000 + usage.ru_utime.tv_usec as u64;
        let system_micros =
            usage.ru_stime.tv_sec as u64 * 1_000_000 + usage.ru_stime.tv_usec as u64;
        Some(user_micros.saturating_add(system_micros))
    }

    #[cfg(not(target_os = "macos"))]
    {
        let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
        let after_command = stat.rsplit_once(") ")?.1;
        let fields: Vec<_> = after_command.split_whitespace().collect();
        let user_ticks = fields.get(11)?.parse::<u64>().ok()?;
        let system_ticks = fields.get(12)?.parse::<u64>().ok()?;
        Some(user_ticks.saturating_add(system_ticks))
    }
}

fn clock_ticks_per_second() -> u64 {
    if cfg!(target_os = "macos") {
        return 1_000_000;
    }
    command_output("getconf", &["CLK_TCK"])
        .parse()
        .unwrap_or(100)
}

#[derive(Clone)]
struct MixQuota {
    next: Arc<AtomicU64>,
    read_percent: u8,
}

impl MixQuota {
    fn new(read_percent: u8) -> Self {
        Self {
            next: Arc::new(AtomicU64::new(0)),
            read_percent,
        }
    }

    async fn claim(&self, role: Role, deadline: Instant) -> bool {
        loop {
            if Instant::now() >= deadline {
                return false;
            }
            let slot = self.next.load(Ordering::Relaxed);
            let read_slot = is_read_slot(slot, self.read_percent);
            let wanted = matches!(role, Role::Reader) == read_slot;
            if !wanted {
                tokio::task::yield_now().await;
                continue;
            }
            if self
                .next
                .compare_exchange(slot, slot + 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }
        }
    }

    fn slots_claimed(&self) -> u64 {
        self.next.load(Ordering::Relaxed)
    }
}

fn is_read_slot(slot: u64, read_percent: u8) -> bool {
    let reads_before = (u128::from(slot) * u128::from(read_percent)) / 100;
    let reads_after = (u128::from(slot.saturating_add(1)) * u128::from(read_percent)) / 100;
    reads_after > reads_before
}

async fn writer_loop(
    adapter: Arc<dyn EngineAdapter>,
    workload: WorkloadConfig,
    seed: u64,
    worker_id: usize,
    start_gate: tokio::sync::watch::Receiver<Option<(Instant, Instant)>>,
    quota: Option<MixQuota>,
    warmup: bool,
    timeline_start: Option<Instant>,
    sampled_keys: Arc<HashSet<DocumentKey>>,
    verifier: Option<Arc<Mutex<SampledReadVerifier>>>,
) -> WorkerStats {
    let mut generator = WorkloadGenerator::new(workload.clone(), seed, worker_id);
    let mut stats = WorkerStats::new(seed ^ worker_id as u64);
    let (_, deadline) = await_start_window(start_gate).await;
    while Instant::now() < deadline {
        if let Some(quota) = &quota
            && !quota.claim(Role::Writer, deadline).await
        {
            break;
        }
        if Instant::now() >= deadline {
            break;
        }
        let request = generator.next_transaction();
        let width = request.mutations.len() as u64;
        let sampled_mutations = if warmup || verifier.is_none() {
            Vec::new()
        } else {
            sampled_mutations(&request, &sampled_keys)
        };
        let started = Instant::now();
        if started >= deadline {
            break;
        }
        let result = adapter.execute_transaction(request).await;
        let finished = Instant::now();
        let elapsed = finished - started;
        if !warmup {
            stats.attempted_transactions += 1;
            stats.e2e_latency.push(elapsed);
            stats.write_latency.push(elapsed);
            match result {
                Ok(transaction_result) => {
                    stats.successful_transactions += 1;
                    stats.mutation_ops += width;
                    if let Some(timeline_start) = timeline_start {
                        stats.window_timeline.push((
                            (finished - timeline_start).as_nanos() as u64,
                            elapsed.as_nanos() as u64,
                        ));
                    }
                    if let Some(verifier) = &verifier {
                        verifier
                            .lock()
                            .expect("sample verifier lock should not be poisoned")
                            .record_mutations(
                                sampled_mutations,
                                transaction_result.commit_lsn.map(dodb_core::Revision::from),
                                started,
                                finished,
                                false,
                            );
                    }
                }
                Err(Error::Conflict(_)) => stats.conflicts += 1,
                Err(Error::Overloaded(_)) => stats.overloads += 1,
                Err(Error::DurabilityFailure(_)) => {
                    stats.errors += 1;
                    if let Some(verifier) = &verifier {
                        verifier
                            .lock()
                            .expect("sample verifier lock should not be poisoned")
                            .record_mutations(sampled_mutations, None, started, finished, true);
                    }
                }
                Err(_) => stats.errors += 1,
            }
        }
    }
    stats.complete_client("writer", worker_id);
    stats
}

fn value_matches_byte(value: &[u8], expected_length: usize, expected_byte: u8) -> bool {
    value.len() == expected_length && value.iter().all(|byte| *byte == expected_byte)
}

async fn reader_loop(
    adapter: Arc<dyn EngineAdapter>,
    workload: WorkloadConfig,
    read_kind: ReadKind,
    seed: u64,
    worker_id: usize,
    start_gate: tokio::sync::watch::Receiver<Option<(Instant, Instant)>>,
    quota: Option<MixQuota>,
    warmup: bool,
    sampled_keys: Arc<HashSet<DocumentKey>>,
    verifier: Option<Arc<Mutex<SampledReadVerifier>>>,
) -> WorkerStats {
    let mut generator = WorkloadGenerator::new(workload.clone(), seed, worker_id);
    let mut stats = WorkerStats::new(seed ^ worker_id as u64 ^ 0xfeed);
    let (_, deadline) = await_start_window(start_gate).await;
    while Instant::now() < deadline {
        if let Some(quota) = &quota
            && !quota.claim(Role::Reader, deadline).await
        {
            break;
        }
        if Instant::now() >= deadline {
            break;
        }
        let request = generator.next_read(read_kind);
        let expected_get_byte = match &request {
            BatchRequest::Get { key } => key.sk.as_bytes().last().copied(),
            _ => None,
        };
        let requested_query_range =
            query_range_from_request(&request, workload.key_size, workload.read_limit);
        let sampled_read_key = match &request {
            BatchRequest::Get { key } if verifier.is_some() && sampled_keys.contains(key) => {
                Some(key.clone())
            }
            _ => None,
        };
        let started = Instant::now();
        if started >= deadline {
            break;
        }
        let result = adapter.execute(request.clone()).await;
        let mut finished = Instant::now();
        if warmup {
            continue;
        }
        match read_kind {
            ReadKind::Get => stats.attempted_gets += 1,
            ReadKind::Query => stats.attempted_queries += 1,
            ReadKind::Scan => stats.attempted_scans += 1,
        }
        if let Some(query_range) = requested_query_range {
            stats.query_ranges.insert(query_range);
        }
        match result {
            Ok(response) if read_kind == ReadKind::Query => {
                record_query_response(&request, &response, &workload, &mut stats);
                finished = Instant::now();
            }
            Ok(BatchResponse::Get(state)) => {
                let (value, revision, found) = match state {
                    RevisionState::Present { value, revision } => {
                        (Some(value), Some(revision), true)
                    }
                    RevisionState::Missing { revision } => (None, Some(revision), false),
                };
                let valid_value = value.as_ref().is_some_and(|value| {
                    expected_get_byte.is_some_and(|expected_byte| {
                        value_matches_byte(value, workload.value_size, expected_byte)
                    })
                });
                if let Some(key) = sampled_read_key
                    && let Some(verifier) = &verifier
                {
                    verifier
                        .lock()
                        .expect("sample verifier lock should not be poisoned")
                        .record_read(SampledRead {
                            key,
                            value,
                            revision,
                            started_at: started,
                            finished_at: finished,
                        });
                }
                if found && valid_value {
                    stats.successful_gets += 1;
                } else {
                    stats.errors += 1;
                }
            }
            Ok(BatchResponse::Scan(rows)) => {
                stats.successful_scans += 1;
                stats.returned_rows += rows.len() as u64;
            }
            Ok(response) => {
                if matches!(response, BatchResponse::Query(_)) {
                    record_query_response(&request, &response, &workload, &mut stats);
                    finished = Instant::now();
                } else {
                    stats.errors += 1;
                }
            }
            Err(Error::Overloaded(_)) => stats.overloads += 1,
            Err(_) => stats.errors += 1,
        }
        let elapsed = finished - started;
        stats.e2e_latency.push(elapsed);
        stats.read_latency.push(elapsed);
    }
    stats.complete_client("reader", worker_id);
    stats
}

async fn mixed_client_loop(
    adapter: Arc<dyn EngineAdapter>,
    workload: WorkloadConfig,
    seed: u64,
    worker_id: usize,
    read_percent: u8,
    value_mode: MixedValueMode,
    next_operation: Arc<AtomicU64>,
    start_gate: tokio::sync::watch::Receiver<Option<(Instant, Instant)>>,
    warmup: bool,
    timeline_start: Option<Instant>,
    sampled_keys: Arc<HashSet<DocumentKey>>,
    verifier: Option<Arc<Mutex<SampledReadVerifier>>>,
) -> WorkerStats {
    let mut stats = WorkerStats::new(seed ^ worker_id as u64 ^ 0xfeed);
    let phase_seed = seed ^ 0x1000_0000;
    let (_, deadline) = await_start_window(start_gate).await;
    while Instant::now() < deadline {
        let operation_index = next_operation.fetch_add(1, Ordering::Relaxed);
        let operation_seed = mixed_operation_seed(phase_seed, operation_index);
        let mut generator = WorkloadGenerator::new_mixed(
            workload.clone(),
            operation_seed,
            phase_seed,
            operation_index,
            value_mode,
        );
        let started = Instant::now();
        if mixed_operation_is_read(operation_index, read_percent) {
            let request = generator.next_read(ReadKind::Get);
            let expected_get_byte = match &request {
                BatchRequest::Get { key } => key.sk.as_bytes().last().copied(),
                _ => None,
            };
            let sampled_read_key = match &request {
                BatchRequest::Get { key } if verifier.is_some() && sampled_keys.contains(key) => {
                    Some(key.clone())
                }
                _ => None,
            };
            let result = adapter.execute(request).await;
            let finished = Instant::now();
            let elapsed = finished - started;
            if warmup {
                continue;
            }
            stats.attempted_gets += 1;
            stats.read_latency.push(elapsed);
            match result {
                Ok(BatchResponse::Get(state)) => {
                    let (value, revision, found) = match state {
                        RevisionState::Present { value, revision } => {
                            (Some(value), Some(revision), true)
                        }
                        RevisionState::Missing { revision } => (None, Some(revision), false),
                    };
                    let valid_value = value.as_ref().is_some_and(|value| {
                        value_matches_byte(
                            value,
                            workload.value_size,
                            expected_get_byte.unwrap_or_default(),
                        )
                    });
                    if let (Some(key), Some(verifier)) = (sampled_read_key, &verifier) {
                        verifier
                            .lock()
                            .expect("sample verifier lock should not be poisoned")
                            .record_read(SampledRead {
                                key,
                                value,
                                revision,
                                started_at: started,
                                finished_at: finished,
                            });
                    }
                    if found && valid_value {
                        stats.successful_gets += 1;
                    } else {
                        stats.errors += 1;
                    }
                }
                Ok(_) => stats.errors += 1,
                Err(Error::Overloaded(_)) => stats.overloads += 1,
                Err(_) => stats.errors += 1,
            }
            if let Some(timeline_start) = timeline_start {
                stats.window_timeline.push((
                    (started + elapsed - timeline_start).as_nanos() as u64,
                    elapsed.as_nanos() as u64,
                ));
            }
        } else {
            let request = generator.next_transaction();
            let width = request.mutations.len() as u64;
            let sampled_mutations = if warmup || verifier.is_none() {
                Vec::new()
            } else {
                sampled_mutations(&request, &sampled_keys)
            };
            let result = adapter.execute_transaction(request).await;
            let finished = Instant::now();
            let elapsed = finished - started;
            if warmup {
                continue;
            }
            stats.attempted_transactions += 1;
            stats.write_latency.push(elapsed);
            match result {
                Ok(transaction_result) => {
                    stats.successful_transactions += 1;
                    stats.mutation_ops += width;
                    if let Some(verifier) = &verifier {
                        verifier
                            .lock()
                            .expect("sample verifier lock should not be poisoned")
                            .record_mutations(
                                sampled_mutations,
                                transaction_result.commit_lsn.map(dodb_core::Revision::from),
                                started,
                                finished,
                                false,
                            );
                    }
                }
                Err(Error::Conflict(_)) => stats.conflicts += 1,
                Err(Error::Overloaded(_)) => stats.overloads += 1,
                Err(Error::DurabilityFailure(_)) => {
                    stats.errors += 1;
                    if let Some(verifier) = &verifier {
                        verifier
                            .lock()
                            .expect("sample verifier lock should not be poisoned")
                            .record_mutations(sampled_mutations, None, started, finished, true);
                    }
                }
                Err(_) => stats.errors += 1,
            }
        }
    }
    stats.complete_client("mixed", worker_id);
    stats
}

async fn run_interval(
    adapter: Arc<dyn EngineAdapter>,
    args: &Args,
    scenario: &Scenario,
    seed: u64,
    duration: Duration,
    warmup: bool,
    sampled_keys: Arc<HashSet<DocumentKey>>,
    verifier: Option<Arc<Mutex<SampledReadVerifier>>>,
) -> IntervalResult {
    let workload = WorkloadConfig {
        distribution: scenario.distribution,
        working_set: args.working_set,
        key_size: args.key_size,
        value_size: args.value_size,
        width: scenario.width,
        transaction_mode: args.transaction_mode,
        read_limit: args.read_limit,
    };
    let quota = scenario.mix.map(|mix| MixQuota::new(mix.read_percent));
    let (start_sender, start_receiver) = tokio::sync::watch::channel(None);
    let timeline_start = (!warmup && args.window_seconds.is_some()).then_some(Instant::now());
    let use_mixed_clients = args.mixed_clients && scenario.mix.is_some();
    let mut tasks = Vec::with_capacity(scenario.writers + scenario.readers);
    if use_mixed_clients {
        let mix = scenario
            .mix
            .expect("mixed client scenario should have a mix");
        let next_operation = Arc::new(AtomicU64::new(0));
        for worker_id in 0..scenario.writers {
            tasks.push(tokio::spawn(mixed_client_loop(
                Arc::clone(&adapter),
                workload.clone(),
                seed,
                worker_id,
                mix.read_percent,
                args.mixed_value_mode,
                Arc::clone(&next_operation),
                start_receiver.clone(),
                warmup,
                timeline_start,
                Arc::clone(&sampled_keys),
                verifier.clone(),
            )));
        }
    } else {
        for worker_id in 0..scenario.writers {
            tasks.push(tokio::spawn(writer_loop(
                Arc::clone(&adapter),
                workload.clone(),
                seed ^ 0x1000_0000,
                worker_id,
                start_receiver.clone(),
                quota.clone(),
                warmup,
                timeline_start,
                Arc::clone(&sampled_keys),
                verifier.clone(),
            )));
        }
        for worker_id in 0..scenario.readers {
            tasks.push(tokio::spawn(reader_loop(
                Arc::clone(&adapter),
                workload.clone(),
                scenario.read_kind.unwrap_or(ReadKind::Get),
                seed ^ 0x2000_0000,
                worker_id,
                start_receiver.clone(),
                quota.clone(),
                warmup,
                Arc::clone(&sampled_keys),
                verifier.clone(),
            )));
        }
    }
    let mut rss_sampler = ProcessRssSampler::spawn(RSS_SAMPLE_INTERVAL);
    let cpu_start = ProcessCpuSample::capture();
    let started_at = Instant::now();
    let deadline = started_at + duration;
    rss_sampler.start(started_at);
    start_sender.send_replace(Some((started_at, deadline)));
    let mut stats = WorkerStats::new(seed ^ 0xabcd);
    for task in tasks {
        stats.merge(task.await.expect("benchmark worker task should not panic"));
    }
    let (finished_at, process_rss) = rss_sampler.finish();
    IntervalResult {
        stats,
        started_at,
        finished_at,
        cpu_start,
        process_rss,
        quota_slots_claimed: if use_mixed_clients {
            None
        } else {
            quota.as_ref().map(MixQuota::slots_claimed)
        },
    }
}

struct IntervalResult {
    stats: WorkerStats,
    started_at: Instant,
    finished_at: Instant,
    cpu_start: ProcessCpuSample,
    process_rss: RssWindowMeasurement,
    quota_slots_claimed: Option<u64>,
}

async fn await_start_window(
    mut start_receiver: tokio::sync::watch::Receiver<Option<(Instant, Instant)>>,
) -> (Instant, Instant) {
    loop {
        if let Some(window) = *start_receiver.borrow_and_update() {
            return window;
        }
        if start_receiver.changed().await.is_err() {
            panic!("benchmark start gate closed before release");
        }
    }
}

#[derive(Clone, Debug)]
struct ResourceSample {
    label: String,
    taken_at: Instant,
    unix_ms: u128,
    rss_kib: u64,
    wal_bytes: u64,
    wal_committed_batches: u64,
    wal_page_images: u64,
    retained_recovery_batches: u64,
    retained_recovery_page_images: u64,
    wal_page_image_records: u64,
    wal_page_delta_records: u64,
    wal_syncs: u64,
    wal_sync_nanos: u64,
    dirty_pages: u64,
    logical_transactions: u64,
}

impl ResourceSample {
    fn capture(adapter: &dyn EngineAdapter, label: String) -> Self {
        let snapshot = adapter.snapshot();
        let wal = snapshot.wal.unwrap_or_default();
        let sample = Self {
            label,
            taken_at: Instant::now(),
            unix_ms: unix_timestamp_ms(),
            rss_kib: process_rss_kib(),
            wal_bytes: wal.wal_bytes,
            wal_committed_batches: wal.committed_batches as u64,
            wal_page_images: wal.page_images as u64,
            retained_recovery_batches: wal.retained_recovery_batches as u64,
            retained_recovery_page_images: wal.retained_recovery_page_images as u64,
            wal_page_image_records: wal.redo.page_image_records,
            wal_page_delta_records: wal.redo.page_delta_records,
            wal_syncs: wal.wal_syncs,
            wal_sync_nanos: wal.sync_nanos,
            dirty_pages: snapshot.dirty_pages.unwrap_or_default() as u64,
            logical_transactions: snapshot.coordinator.logical_transactions,
        };
        println!(
            "resource_sample label={} unix_ms={} rss_kib={} wal_bytes={} wal_committed_batches={} wal_page_images={} retained_recovery_batches={} retained_recovery_page_images={} wal_page_image_records={} wal_page_delta_records={} wal_syncs={} wal_sync_nanos={} dirty_pages={} logical_transactions={}",
            sample.label,
            sample.unix_ms,
            sample.rss_kib,
            sample.wal_bytes,
            sample.wal_committed_batches,
            sample.wal_page_images,
            sample.retained_recovery_batches,
            sample.retained_recovery_page_images,
            sample.wal_page_image_records,
            sample.wal_page_delta_records,
            sample.wal_syncs,
            sample.wal_sync_nanos,
            sample.dirty_pages,
            sample.logical_transactions,
        );
        sample
    }
}

fn process_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find_map(|status_line| status_line.strip_prefix("VmRSS:"))
                .and_then(|value| value.split_whitespace().next()?.parse().ok())
        })
        .unwrap_or(0)
}

struct ResourceSampler {
    stop: Arc<AtomicBool>,
    handle: std::thread::JoinHandle<Vec<ResourceSample>>,
}

impl ResourceSampler {
    fn start(adapter: Arc<dyn EngineAdapter>, started: Instant, window: Duration) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            let mut samples = Vec::new();
            let mut window_index = 0u32;
            loop {
                let window_end = started + window * (window_index + 1);
                loop {
                    if thread_stop.load(Ordering::Acquire) {
                        return samples;
                    }
                    let now = Instant::now();
                    if now >= window_end {
                        break;
                    }
                    std::thread::sleep((window_end - now).min(Duration::from_millis(50)));
                }
                samples.push(ResourceSample::capture(
                    &*adapter,
                    format!("window_{window_index:02}_end"),
                ));
                window_index += 1;
            }
        });
        Self { stop, handle }
    }

    fn finish(self) -> Vec<ResourceSample> {
        self.stop.store(true, Ordering::Release);
        self.handle
            .join()
            .expect("resource sampler thread should not panic")
    }
}

fn histogram_delta(after: &[u64], before: &[u64]) -> Vec<u64> {
    after
        .iter()
        .enumerate()
        .map(|(bucket, count)| count.saturating_sub(before.get(bucket).copied().unwrap_or(0)))
        .collect()
}

fn histogram_text(histogram: &[u64]) -> String {
    histogram
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0)
        .map(|(bucket, count)| format!("{bucket}:{count}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn redo_delta(after: &WalRedoStats, before: &WalRedoStats) -> WalRedoStats {
    WalRedoStats {
        page_image_records: u64::saturating_sub(
            after.page_image_records,
            before.page_image_records,
        ),
        page_delta_records: u64::saturating_sub(
            after.page_delta_records,
            before.page_delta_records,
        ),
        page_delta_payload_bytes: u64::saturating_sub(
            after.page_delta_payload_bytes,
            before.page_delta_payload_bytes,
        ),
        page_delta_spans: u64::saturating_sub(after.page_delta_spans, before.page_delta_spans),
        page_delta_changed_bytes: u64::saturating_sub(
            after.page_delta_changed_bytes,
            before.page_delta_changed_bytes,
        ),
        image_superblock: u64::saturating_sub(after.image_superblock, before.image_superblock),
        image_not_requested: u64::saturating_sub(
            after.image_not_requested,
            before.image_not_requested,
        ),
        image_page_image_format: u64::saturating_sub(
            after.image_page_image_format,
            before.image_page_image_format,
        ),
        image_ineligible_commit: u64::saturating_sub(
            after.image_ineligible_commit,
            before.image_ineligible_commit,
        ),
        image_first_touch: u64::saturating_sub(after.image_first_touch, before.image_first_touch),
        image_no_base: u64::saturating_sub(after.image_no_base, before.image_no_base),
        image_not_smaller: u64::saturating_sub(after.image_not_smaller, before.image_not_smaller),
    }
}

fn benchmark_config(args: &Args, scenario: &Scenario) -> CoordinatorConfig {
    CoordinatorConfig {
        queue_capacity: args.queue_capacity,
        max_group_requests: args.max_group_requests,
        max_group_bytes: args.max_group_bytes,
        max_collection_delay: scenario.collection_delay,
    }
}

fn effective_sync(args: &Args, scenario: &Scenario) -> (SyncMode, Duration) {
    if scenario.suite == Suite::SyncSweep {
        (SyncMode::Injected, scenario.sync_delay)
    } else {
        match args.sync_mode {
            SyncMode::Real => (SyncMode::Real, Duration::ZERO),
            SyncMode::Injected => (SyncMode::Injected, scenario.sync_delay),
            SyncMode::Disabled => (SyncMode::Disabled, Duration::ZERO),
        }
    }
}

#[cfg(feature = "phase-i-instrumentation")]
fn write_phase_i_locality_samples(args: &Args, scenario: &Scenario, repetition: usize, seed: u64) {
    use std::io::Write;

    let Some(path) = env::var_os("DODB_PHASE_I_LOCALITY_OUTPUT") else {
        let _ = dodb_storage::blink::take_phase_i_group_locality_samples();
        return;
    };
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("locality output directory should be creatable");
    }
    let mut output = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("locality output should open");
    let encode_array = |values: &[usize]| {
        values
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(",")
    };
    let git_commit = current_git_commit();
    for (group_index, sample) in dodb_storage::blink::take_phase_i_group_locality_samples()
        .into_iter()
        .enumerate()
    {
        writeln!(
            output,
            "{{\"record_type\":\"group_locality\",\"git_commit\":{},\"parallel_background_min_operations\":{},\"blink_read_observational_metrics_enabled\":{},\"sync_mode\":{},\"writers\":{},\"width\":{},\"distribution\":{},\"repetition\":{},\"seed\":{},\"group_index\":{},\"requested_transactions\":{},\"successful_transactions\":{},\"failed_transactions\":{},\"logical_mutations\":{},\"unique_keys\":{},\"boundary_materializations\":{},\"page_encodes\":{},\"page_delta_records\":{},\"distinct_touched_leaves\":{},\"mutations_per_leaf\":[{}],\"transactions_per_leaf\":[{}],\"leaves_by_transaction_touch_count\":[{},{},{}]}}",
            json_string(&git_commit),
            args.parallel_background_min_operations,
            !cfg!(feature = "blink-read-metrics-disabled"),
            json_string(effective_sync(args, scenario).0.as_str()),
            scenario.writers,
            scenario.width,
            json_string(scenario.distribution.as_str()),
            repetition,
            seed,
            group_index,
            sample.requested_transactions,
            sample.successful_transactions,
            sample.failed_transactions,
            sample.logical_mutations,
            sample.unique_keys,
            sample.boundary_materializations,
            sample.page_encodes,
            sample.page_delta_records,
            sample.distinct_touched_leaves,
            encode_array(&sample.mutations_per_leaf),
            encode_array(&sample.transactions_per_leaf),
            sample.leaves_by_transaction_touch_count[0],
            sample.leaves_by_transaction_touch_count[1],
            sample.leaves_by_transaction_touch_count[2],
        )
        .expect("locality event should write");
    }
    output.flush().expect("locality output should flush");
}

fn benchmark_path(scenario: &Scenario, repetition: usize, seed: u64) -> PathBuf {
    let scenario_name = scenario.name().replace(['/', '\\'], "_");
    let directory = benchmark_directory();
    std::fs::create_dir_all(&directory).expect("benchmark data directory should be creatable");
    directory.join(format!(
        "dodb-phase0-{}-{}-{}-{seed:016x}.db",
        std::process::id(),
        scenario_name,
        repetition
    ))
}

fn benchmark_directory() -> PathBuf {
    env::var_os("DODB_BENCH_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir)
}

fn seed_store(
    store: &mut BTreeStore<BenchFile, BenchFile>,
    args: &Args,
    scenario: &Scenario,
) -> Result<usize> {
    let requests = seed_requests(args, scenario);
    let seeded = requests.iter().map(|request| request.mutations.len()).sum();
    for chunk in requests.chunks(64) {
        store.apply_transaction_group(chunk)?;
    }
    Ok(seeded)
}

fn seed_requests(args: &Args, scenario: &Scenario) -> Vec<TransactionRequest> {
    let workload = WorkloadConfig {
        distribution: scenario.distribution,
        working_set: args.working_set,
        key_size: args.key_size,
        value_size: args.value_size,
        width: 25.min(args.working_set),
        transaction_mode: TransactionMode::Unconditional,
        read_limit: args.read_limit,
    };
    let generator = WorkloadGenerator::new(workload.clone(), args.seed, 0);
    let mut requests = Vec::new();
    let mut mutations = Vec::new();
    for (index, key) in generator.seed_keys().enumerate() {
        mutations.push(TransactionMutation::Put {
            key,
            value: value_bytes(args.value_size, index as u64, 0),
        });
        if mutations.len() >= 25 {
            requests.push(TransactionRequest::new(
                Vec::new(),
                std::mem::take(&mut mutations),
            ));
        }
    }
    if !mutations.is_empty() {
        requests.push(TransactionRequest::new(Vec::new(), mutations));
    }

    if scenario.read_kind.is_some() || scenario.mix.is_some() {
        let mut query_mutations = Vec::new();
        for pk_index in 0..QUERY_PK_COUNT {
            for row_index in 0..query_rows_per_pk(args.read_limit) {
                query_mutations.push(TransactionMutation::Put {
                    key: distributed_query_key(args.key_size, pk_index, row_index),
                    value: value_bytes(
                        args.value_size,
                        distributed_query_value_index(pk_index, row_index, args.read_limit),
                        0,
                    ),
                });
                if query_mutations.len() >= 25 {
                    requests.push(TransactionRequest::new(
                        Vec::new(),
                        std::mem::take(&mut query_mutations),
                    ));
                }
            }
        }
        if !query_mutations.is_empty() {
            requests.push(TransactionRequest::new(Vec::new(), query_mutations));
        }
    }

    requests
}

async fn open_adapter(
    args: &Args,
    scenario: &Scenario,
    repetition: usize,
    seed: u64,
) -> Result<(Arc<dyn EngineAdapter>, PathBuf, usize)> {
    let data_path = benchmark_path(scenario, repetition, seed);
    let wal_path = data_path.with_extension("wal");
    remove_database_files(&data_path);
    let (sync_mode, sync_delay) = effective_sync(args, scenario);
    let config = DatabaseConfig::default().with_cache_capacity(args.cache_capacity);
    match args.engine {
        EngineKind::MainBtree => {
            let data_file = BenchFile::open(&data_path, sync_mode, sync_delay)?;
            let wal_file = BenchFile::open(&wal_path, sync_mode, sync_delay)?;
            let mut store = if args.main_compact_wal {
                BTreeStore::open_with_compact_wal(data_file, wal_file, config)?
            } else {
                BTreeStore::open_with_wal(data_file, wal_file, config)?
            };
            let seeded = seed_store(&mut store, args, scenario)?;
            let shard = Arc::new(AsyncShard::start_with_config(
                store,
                benchmark_config(args, scenario),
            ));
            Ok((Arc::new(BaselineAdapter { shard }), data_path, seeded))
        }
        EngineKind::SerialBlink => {
            let mut store = BlinkStore::open_with_wal(
                BenchFile::open(&data_path, sync_mode, sync_delay)?,
                BenchFile::open(&wal_path, sync_mode, sync_delay)?,
                config,
            )?;
            let requests = seed_requests(args, scenario);
            let seeded = requests.iter().map(|request| request.mutations.len()).sum();
            for chunk in requests.chunks(64) {
                store.apply_transaction_group(chunk)?;
            }
            let adapter = BlinkAdapter::start(
                store,
                benchmark_config(args, scenario),
                args.collection_policy,
            );
            Ok((Arc::new(adapter), data_path, seeded))
        }
        EngineKind::VersionedBlink => {
            let mut store = BlinkStore::open_with_wal(
                BenchFile::open(&data_path, sync_mode, sync_delay)?,
                BenchFile::open(&wal_path, sync_mode, sync_delay)?,
                config,
            )?;
            let requests = seed_requests(args, scenario);
            let seeded = requests.iter().map(|request| request.mutations.len()).sum();
            for chunk in requests.chunks(64) {
                store.apply_transaction_group(chunk)?;
            }
            let adapter = BlinkAdapter::start_versioned(
                store,
                benchmark_config(args, scenario),
                args.collection_policy,
            );
            Ok((Arc::new(adapter), data_path, seeded))
        }
        EngineKind::PlannedBlink => {
            let mut store = BlinkStore::open_with_wal(
                BenchFile::open(&data_path, sync_mode, sync_delay)?,
                BenchFile::open(&wal_path, sync_mode, sync_delay)?,
                config,
            )?;
            store.enable_planned_execution();
            let requests = seed_requests(args, scenario);
            let seeded = requests.iter().map(|request| request.mutations.len()).sum();
            for chunk in requests.chunks(64) {
                store.apply_transaction_group(chunk)?;
            }
            if args.parallel_workers > 0 {
                store.enable_parallel_execution(args.parallel_workers)?;
                store.set_parallel_min_group_mutations(args.parallel_min_mutations);
                store.set_parallel_min_background_worker_operations(
                    args.parallel_background_min_operations,
                );
            }
            if CHECKPOINT_WAL_BYTES.load(Ordering::Relaxed) > 0 {
                checkpoint_blink_store(&mut store)?;
            }
            let adapter = BlinkAdapter::start_versioned(
                store,
                benchmark_config(args, scenario),
                args.collection_policy,
            );
            Ok((Arc::new(adapter), data_path, seeded))
        }
        EngineKind::ParallelBlink => {
            let mut store = BlinkStore::open_with_wal(
                BenchFile::open(&data_path, sync_mode, sync_delay)?,
                BenchFile::open(&wal_path, sync_mode, sync_delay)?,
                config,
            )?;
            store.enable_planned_execution();
            let requests = seed_requests(args, scenario);
            let seeded = requests.iter().map(|request| request.mutations.len()).sum();
            for chunk in requests.chunks(64) {
                store.apply_transaction_group(chunk)?;
            }
            store.enable_parallel_execution(args.blink_workers)?;
            store.set_parallel_min_background_worker_operations(
                args.parallel_background_min_operations,
            );
            let adapter = BlinkAdapter::start_versioned(
                store,
                benchmark_config(args, scenario),
                args.collection_policy,
            );
            Ok((Arc::new(adapter), data_path, seeded))
        }
        EngineKind::LogicalOverlayBlink | EngineKind::BackgroundOverlayBlink => {
            let mut store = BlinkStore::open_with_logical_wal(
                BenchFile::open(&data_path, sync_mode, sync_delay)?,
                BenchFile::open(&wal_path, sync_mode, sync_delay)?,
                config,
            )?;
            store.set_background_materialization(args.engine == EngineKind::BackgroundOverlayBlink);
            let requests = seed_requests(args, scenario);
            let seeded = requests.iter().map(|request| request.mutations.len()).sum();
            for chunk in requests.chunks(64) {
                store.apply_transaction_group(chunk)?;
            }
            let adapter = BlinkAdapter::start_versioned(
                store,
                benchmark_config(args, scenario),
                args.collection_policy,
            );
            Ok((Arc::new(adapter), data_path, seeded))
        }
    }
}

fn remove_database_files(data_path: &Path) {
    let _ = std::fs::remove_file(data_path);
    let _ = std::fs::remove_file(data_path.with_extension("wal"));
}

fn unix_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn duration_to_nanos(duration: Duration) -> u64 {
    duration.as_nanos().min(u64::MAX as u128) as u64
}

#[derive(Clone, Debug)]
struct JsonObject {
    fields: Vec<(String, String)>,
}

impl JsonObject {
    fn new() -> Self {
        Self { fields: Vec::new() }
    }

    fn string(&mut self, key: &str, value: &str) {
        self.fields.push((key.to_owned(), json_string(value)));
    }

    fn usize(&mut self, key: &str, value: usize) {
        self.fields.push((key.to_owned(), value.to_string()));
    }

    fn u64(&mut self, key: &str, value: u64) {
        self.fields.push((key.to_owned(), value.to_string()));
    }

    fn boolean(&mut self, key: &str, value: bool) {
        self.fields.push((key.to_owned(), value.to_string()));
    }

    fn bool(&mut self, key: &str, value: bool) {
        self.fields.push((key.to_owned(), value.to_string()));
    }

    fn raw(&mut self, key: &str, value: String) {
        self.fields.push((key.to_owned(), value));
    }
    fn f64(&mut self, key: &str, value: f64) {
        self.fields.push((
            key.to_owned(),
            if value.is_finite() {
                format!("{value:.6}")
            } else {
                "null".to_owned()
            },
        ));
    }

    fn optional_f64(&mut self, key: &str, value: Option<f64>) {
        match value {
            Some(value) => self.f64(key, value),
            None => self.fields.push((key.to_owned(), "null".to_owned())),
        }
    }

    fn optional_u64(&mut self, key: &str, value: Option<u64>) {
        match value {
            Some(value) => self.u64(key, value),
            None => self.fields.push((key.to_owned(), "null".to_owned())),
        }
    }

    fn finish(self) -> String {
        let fields = self
            .fields
            .into_iter()
            .map(|(key, value)| format!("{}:{value}", json_string(&key)))
            .collect::<Vec<_>>();
        format!("{{{}}}", fields.join(","))
    }
}

fn add_parallel_metadata(json: &mut JsonObject, args: &Args) {
    json.usize(
        "parallel_background_min_operations",
        args.parallel_background_min_operations,
    );
    json.boolean(
        "blink_read_observational_metrics_enabled",
        !cfg!(feature = "blink-read-metrics-disabled"),
    );
    json.boolean(
        "blink_borrowed_page_views_enabled",
        cfg!(feature = "blink-borrowed-page-views"),
    );
    json.boolean("main_compact_wal_enabled", args.main_compact_wal);
}

fn json_string(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(output, "\\u{:04x}", character as u32);
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

fn metric_field_name(prefix: &str, field: &str) -> String {
    format!("{prefix}_{field}")
}

fn build_record(
    machine: &MachineInfo,
    args: &Args,
    scenario: &Scenario,
    scenario_index: usize,
    repetition: usize,
    seed: u64,
    seeded_rows: usize,
    measured: &WorkerStats,
    delta: &MetricDelta,
    verification: VerificationSummary,
    process_rss: &RssWindowMeasurement,
    query_preflight_passed: Option<bool>,
    quota_slots_claimed: Option<u64>,
    wall: Duration,
    cpu_start: &ProcessCpuSample,
    cpu_end: &ProcessCpuSample,
    measurement_started: Instant,
    samples: &[ResourceSample],
    churn: &[(String, u64)],
) -> String {
    let mut json = JsonObject::new();
    let seconds = wall.as_secs_f64().max(f64::EPSILON);
    let logical_tx_per_second = measured.successful_transactions as f64 / seconds;
    let mutation_ops_per_second = measured.mutation_ops as f64 / seconds;
    let get_ops_per_second = measured.successful_gets as f64 / seconds;
    let query_ops_per_second = measured.successful_queries as f64 / seconds;
    let scan_ops_per_second = measured.successful_scans as f64 / seconds;
    let read_ops_per_second = measured.successful_reads() as f64 / seconds;
    let aggregate_ops_per_second = measured.successful_operations() as f64 / seconds;
    let rows_per_second = measured.returned_rows as f64 / seconds;
    let (cpu_one_core, cpu_machine) = cpu_start.utilization(cpu_end, wall, machine.logical_cpus);
    let max_actual_group_bytes = if delta.max_group_bytes == 0
        && args.engine != EngineKind::MainBtree
        && scenario.suite == Suite::Mixed
        && delta.max_group_requests > 0
    {
        let workload = WorkloadConfig {
            distribution: scenario.distribution,
            working_set: args.working_set,
            key_size: args.key_size,
            value_size: args.value_size,
            width: scenario.width,
            transaction_mode: args.transaction_mode,
            read_limit: args.read_limit,
        };
        let phase_seed = seed ^ 0xbbbb_0000 ^ 0x1000_0000;
        let mut generator = WorkloadGenerator::new_mixed(
            workload,
            mixed_operation_seed(phase_seed, 0),
            phase_seed,
            0,
            args.mixed_value_mode,
        );
        transaction_request_size(&generator.next_transaction())
            .saturating_mul(delta.max_group_requests)
    } else {
        delta.max_group_bytes
    };

    json.string("record_type", "run");
    json.u64("timestamp_unix_ms", unix_timestamp_ms() as u64);
    json.string("git_commit", &current_git_commit());
    json.string(
        "benchmark_data_dir",
        &benchmark_directory().display().to_string(),
    );
    json.usize("scenario_index", scenario_index);
    json.string(
        "seed_formula",
        "base_seed + scenario_index * 0x9e3779b9 + repetition, wrapping u64",
    );
    json.bool("source_dirty", current_git_dirty().unwrap_or(true));
    json.string("source_sha256", &benchmark_source_sha256());
    json.string("cargo_lock_sha256", &cargo_lock_sha256());
    json.string("binary_sha256", &current_binary_sha256());
    json.string(
        "build_target",
        &format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
    );
    json.string(
        "rustflags",
        &std::env::var("RUSTFLAGS").unwrap_or_else(|_| "unset".to_owned()),
    );
    let command_line = std::env::args()
        .map(|argument| json_string(&argument))
        .collect::<Vec<_>>()
        .join(",");
    json.raw("command_line", format!("[{command_line}]"));
    json.string("baseline_commit", BASELINE_COMMIT);
    json.string("engine", args.engine.as_str());
    json.string(
        "collection_policy",
        if args.engine == EngineKind::MainBtree {
            "native-main"
        } else {
            args.collection_policy.as_str()
        },
    );
    json.string(
        "build_mode",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
    );
    json.string("cpu_model", &machine.cpu_model);
    json.usize("logical_cpus", machine.logical_cpus);
    if let Some(cpu_cores) = machine.cpu_cores {
        json.usize("cpu_cores", cpu_cores);
    }
    json.string("os", &machine.os);
    json.string("kernel", &machine.kernel);
    json.string("rust_version", &machine.rust_version);
    json.usize("tokio_workers", args.tokio_workers);
    json.usize("blink_workers", args.blink_workers);
    json.usize("parallel_workers", args.parallel_workers);
    json.usize("parallel_min_mutations", args.parallel_min_mutations);
    add_parallel_metadata(&mut json, args);
    json.string("suite", scenario.suite.as_str());
    json.string("workload", scenario.workload);
    json.usize("writers", scenario.writers);
    json.usize("readers", scenario.readers);
    json.usize(
        "client_workers",
        if args.mixed_clients && scenario.mix.is_some() {
            scenario.writers
        } else {
            scenario.writers + scenario.readers
        },
    );
    json.usize("transaction_width", scenario.width);
    json.string("distribution", scenario.distribution.as_str());
    json.string(
        "read_kind",
        scenario.read_kind.map_or("none", ReadKind::as_str),
    );
    json.string("mix", scenario.mix.map_or("none", Mix::as_str));
    if let Some(mix) = scenario.mix {
        json.string("mixed_value_mode", args.mixed_value_mode.as_str());
        json.string(
            "mixed_value_generator",
            args.mixed_value_mode.generator_name(),
        );
        if args.mixed_clients {
            json.string(
                "mixed_schedule",
                "global_fetch_add; operation_index_mod_100_lt_read_percent; shared_seeded_request_v2",
            );
            json.string("mixed_schedule_version", "shared_seeded_request_v2");
            let workload = WorkloadConfig {
                distribution: scenario.distribution,
                working_set: args.working_set,
                key_size: args.key_size,
                value_size: args.value_size,
                width: scenario.width,
                transaction_mode: args.transaction_mode,
                read_limit: args.read_limit,
            };
            json.u64("logical_trace_prefix_operations", 1_000);
            json.string(
                "logical_trace_prefix_hash",
                &format!(
                    "{:016x}",
                    mixed_trace_prefix_hash(
                        workload,
                        seed ^ 0xbbbb_0000 ^ 0x1000_0000,
                        mix.read_percent,
                        args.mixed_value_mode,
                        1_000,
                    )
                ),
            );
        }
        json.string("mix_basis", "one read request per one write transaction");
        json.u64("requested_read_percent", u64::from(mix.read_percent));
        json.u64("requested_write_percent", u64::from(100 - mix.read_percent));
        let attempted_mix_operations = measured.attempted_transactions + measured.attempted_reads();
        let cancelled_quota_claims = quota_slots_claimed
            .unwrap_or(attempted_mix_operations)
            .saturating_sub(attempted_mix_operations);
        let attempted_share_error = (measured.attempted_reads() as f64
            - attempted_mix_operations as f64 * f64::from(mix.read_percent) / 100.0)
            .abs();
        let attempted_share_error_bound = 1.0 + cancelled_quota_claims as f64;
        json.u64("quota_slots_claimed", quota_slots_claimed.unwrap_or(0));
        json.u64("quota_claims_without_attempt", cancelled_quota_claims);
        json.f64(
            "attempted_read_share_error_operations",
            attempted_share_error,
        );
        json.f64(
            "attempted_read_share_error_bound_operations",
            attempted_share_error_bound,
        );
        json.bool(
            "attempted_mix_within_interleaved_quota_bound",
            attempted_share_error <= attempted_share_error_bound,
        );
        json.usize(
            "max_inflight_operations_at_deadline",
            scenario.writers + scenario.readers,
        );
        let successful_mixed_operations =
            measured.successful_transactions + measured.successful_reads();
        json.u64("attempted_read_operations", measured.attempted_reads());
        json.u64(
            "attempted_write_transactions",
            measured.attempted_transactions,
        );
        json.u64("successful_read_operations", measured.successful_reads());
        json.u64(
            "successful_write_transactions",
            measured.successful_transactions,
        );
        json.f64(
            "attempted_read_percent",
            percent(measured.attempted_reads(), attempted_mix_operations),
        );
        json.f64(
            "attempted_write_percent",
            percent(measured.attempted_transactions, attempted_mix_operations),
        );
        json.f64(
            "successful_read_percent",
            percent(measured.successful_reads(), successful_mixed_operations),
        );
        json.f64(
            "successful_write_percent",
            percent(
                measured.successful_transactions,
                successful_mixed_operations,
            ),
        );
        json.string(
            "mix_boundary_note",
            "quota is assigned per request/transaction with interleaved slots; stop-at-deadline cancels counted quota claims that did not start and drains at most one in-flight request per client",
        );
    }
    json.string("transaction_mode", args.transaction_mode.as_str());
    json.usize("cache_capacity", args.cache_capacity);
    json.usize("working_set", args.working_set);
    json.usize("seeded_rows", seeded_rows);
    json.usize("key_size", args.key_size);
    json.usize("value_size", args.value_size);
    json.usize("read_limit", args.read_limit);
    json.usize("queue_capacity", args.queue_capacity);
    json.usize("max_group_requests", args.max_group_requests);
    json.usize("max_group_bytes", args.max_group_bytes);
    json.u64(
        "collection_delay_us",
        scenario.collection_delay.as_micros() as u64,
    );
    let (sync_mode, sync_delay) = effective_sync(args, scenario);
    json.string("sync_mode", sync_mode.as_str());
    json.string(
        "sync_contract",
        if sync_mode == SyncMode::Real {
            "durable-return"
        } else {
            "not-durable"
        },
    );
    json.u64("sync_delay_us", sync_delay.as_micros() as u64);
    json.u64("seed", seed);
    json.u64("requested_duration_ms", args.duration.as_millis() as u64);
    json.u64("duration_ms", wall.as_millis() as u64);
    json.string(
        "measurement_stop_policy",
        "stop admissions at deadline, drain in-flight operations, and divide counted outcomes by elapsed time through final client completion; request generation after the shared start is included in wall time",
    );
    json.u64("warmup_ms", args.warmup.as_millis() as u64);
    json.usize("repetition", repetition);
    if scenario.read_kind == Some(ReadKind::Query) {
        let rows_per_pk = query_rows_per_pk(args.read_limit);
        let ranges_per_pk = rows_per_pk - args.read_limit + 1;
        json.string("query_workload", "distributed-query16-v1");
        json.string(
            "query_selection",
            "splitmix64 with rejection sampling over flattened legal PK/start ranges",
        );
        json.usize("query_seeded_pk_count", QUERY_PK_COUNT);
        json.usize("query_seeded_rows_per_pk", rows_per_pk);
        json.usize("query_seeded_rows_total", QUERY_PK_COUNT * rows_per_pk);
        json.usize("query_start_index_min", 0);
        json.usize("query_start_index_max", rows_per_pk - args.read_limit);
        json.usize("query_legal_ranges_per_pk", ranges_per_pk);
        json.usize(
            "query_legal_range_count",
            query_range_count(args.read_limit),
        );
        json.string(
            "query_input_plan_fingerprint",
            &query_input_plan_fingerprint(args, scenario, seed),
        );
        json.usize(
            "query_input_plan_requests_per_client",
            QUERY_INPUT_PLAN_REQUESTS,
        );
        json.string(
            "query_validation_scope",
            "every measured Query response; full key/value validation",
        );
        json.string(
            "query_validation_cost_scope",
            "request generation is inside measurement wall but outside per-request latency; per-response verification is inside both measurement wall and per-request latency; input plan fingerprint and preflight are outside measurement; JSONL serialization and flush are after measurement",
        );
        json.u64("query_checked_requests", measured.query_checked_requests);
        json.u64("query_checked_rows", measured.query_checked_rows);
        json.u64(
            "query_validation_failures",
            measured.query_validation_failures,
        );
        json.usize("query_actual_range_count", measured.query_ranges.len());
        json.string(
            "query_validation_status",
            if measured.query_validation_failures > 0 {
                "failed"
            } else if measured.query_checked_requests == 0 {
                "inconclusive"
            } else {
                "passed"
            },
        );
        json.bool(
            "query_client_aggregation_passed",
            measured
                .client_completions
                .iter()
                .map(|client| client.successful_queries)
                .sum::<u64>()
                == measured.successful_queries
                && measured
                    .client_completions
                    .iter()
                    .map(|client| client.returned_rows)
                    .sum::<u64>()
                    == measured.returned_rows
                && measured
                    .client_completions
                    .iter()
                    .map(|client| client.query_checked_requests)
                    .sum::<u64>()
                    == measured.query_checked_requests
                && measured
                    .client_completions
                    .iter()
                    .map(|client| client.query_checked_rows)
                    .sum::<u64>()
                    == measured.query_checked_rows
                && measured
                    .client_completions
                    .iter()
                    .map(|client| client.query_validation_failures)
                    .sum::<u64>()
                    == measured.query_validation_failures,
        );
        json.bool(
            "query_returned_row_count_consistent",
            measured.query_validation_failures == 0
                && measured.query_checked_requests == measured.successful_queries
                && measured.returned_rows == measured.successful_queries * args.read_limit as u64,
        );
    }
    json.u64("attempted_operations", measured.attempted_operations());
    json.u64("successful_operations", measured.successful_operations());
    json.u64("attempted_transactions", measured.attempted_transactions);
    json.u64("successful_transactions", measured.successful_transactions);
    json.u64("attempted_gets", measured.attempted_gets);
    json.u64("successful_gets", measured.successful_gets);
    json.u64("attempted_queries", measured.attempted_queries);
    json.u64("successful_queries", measured.successful_queries);
    json.u64("attempted_scans", measured.attempted_scans);
    json.u64("successful_scans", measured.successful_scans);
    json.u64("conflicts", measured.conflicts);
    json.u64("overloads", measured.overloads);
    json.u64("errors", measured.errors);
    let client_attempted_operations = measured
        .client_completions
        .iter()
        .map(|client| client.attempted_operations)
        .sum::<u64>();
    let client_successful_operations = measured
        .client_completions
        .iter()
        .map(|client| client.successful_operations)
        .sum::<u64>();
    json.bool(
        "client_aggregation_passed",
        client_attempted_operations == measured.attempted_operations()
            && client_successful_operations == measured.successful_operations(),
    );
    let clients = measured
        .client_completions
        .iter()
        .map(|client| {
            format!(
                "{{\"role\":{},\"client_index\":{},\"attempted_operations\":{},\"successful_operations\":{},\"successful_queries\":{},\"returned_rows\":{},\"query_checked_requests\":{},\"query_checked_rows\":{},\"query_validation_failures\":{}}}",
                json_string(client.role),
                client.client_index,
                client.attempted_operations,
                client.successful_operations,
                client.successful_queries,
                client.returned_rows,
                client.query_checked_requests,
                client.query_checked_rows,
                client.query_validation_failures,
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    json.raw("clients", format!("[{clients}]"));
    json.string(
        "sampled_read_verification_status",
        verification.status.as_str(),
    );
    json.usize("sampled_key_count", verification.sampled_key_count);
    json.usize("sampled_key_limit", MAX_SAMPLED_KEYS);
    json.usize(
        "sample_key_selection_scan_limit",
        MAX_SAMPLE_SELECTION_CANDIDATES,
    );
    json.u64(
        "sampled_reads_observed",
        verification.sampled_reads_observed,
    );
    json.u64("sampled_reads_checked", verification.sampled_reads_checked);
    json.u64("sampled_reads_failed", verification.sampled_reads_failed);
    json.u64(
        "sampled_reads_indeterminate",
        verification.sampled_reads_indeterminate,
    );
    json.u64(
        "sampled_writes_observed",
        verification.sampled_writes_observed,
    );
    json.u64(
        "sampled_writes_confirmed",
        verification.sampled_writes_confirmed,
    );
    json.u64(
        "sampled_writes_ambiguous",
        verification.sampled_writes_ambiguous,
    );
    json.u64("sampled_writes_stored", verification.sampled_writes_stored);
    json.u64(
        "changed_key_reads_checked",
        verification.changed_key_reads_checked,
    );
    json.u64("verification_omitted_events", verification.omitted_events);
    json.usize(
        "verification_estimated_memory_bytes",
        verification.estimated_memory_bytes,
    );
    json.usize(
        "verification_estimated_index_memory_bytes",
        verification.estimated_index_memory_bytes,
    );
    json.usize(
        "verification_estimated_peak_memory_bytes",
        verification
            .estimated_memory_bytes
            .saturating_add(verification.estimated_index_memory_bytes)
            .saturating_add(MAX_VERIFICATION_MEMORY_BYTES / 4),
    );
    json.usize(
        "verification_memory_limit_bytes",
        MAX_VERIFICATION_MEMORY_BYTES,
    );
    json.bool("process_rss_supported", process_rss.supported);
    json.string("process_rss_status", process_rss.status);
    json.string("process_rss_unit", "bytes");
    json.string("process_rss_method", process_rss_method());
    json.string(
        "process_rss_sampling_execution",
        "dedicated standard thread using in-process OS APIs; no subprocess per sample; sampling work contributes to process RSS and execution cost",
    );
    json.string(
        "process_rss_boundary_sample_semantics",
        "start and end RSS are the first and last successful in-window samples; offsets identify their observation times relative to common start",
    );
    json.string(
        "process_rss_window_semantics",
        "samples are restricted to common measurement start through final client completion; samples outside the interval are excluded",
    );
    json.optional_u64("process_rss_start_bytes", process_rss.start_rss_bytes);
    json.optional_u64("process_rss_end_bytes", process_rss.end_rss_bytes);
    json.optional_u64(
        "process_rss_peak_observed_bytes",
        process_rss.peak_rss_bytes,
    );
    json.u64("process_rss_sample_count", process_rss.sample_count);
    json.u64(
        "process_rss_requested_sample_interval_ns",
        duration_to_nanos(process_rss.requested_sample_interval),
    );
    json.optional_u64(
        "process_rss_max_sample_interval_ns",
        process_rss.max_sample_interval.map(duration_to_nanos),
    );
    json.u64(
        "process_rss_collection_failures",
        process_rss.collection_failures,
    );
    json.u64("process_rss_window_start_offset_ns", 0);
    json.u64("process_rss_window_end_offset_ns", duration_to_nanos(wall));
    json.optional_u64(
        "process_rss_start_sample_offset_ns",
        process_rss.start_sample_offset.map(duration_to_nanos),
    );
    json.optional_u64(
        "process_rss_end_sample_offset_ns",
        process_rss.end_sample_offset.map(duration_to_nanos),
    );
    json.optional_u64(
        "process_rss_peak_sample_offset_ns",
        process_rss.peak_sample_offset.map(duration_to_nanos),
    );
    json.string(
        "process_rss_peak_semantics",
        "maximum successfully observed RSS sample within the measured interval; unsampled instantaneous peaks may be missed",
    );
    json.usize("verification_event_limit", MAX_VERIFICATION_EVENTS);
    json.u64(
        "verification_work_events",
        verification
            .sampled_reads_checked
            .saturating_add(verification.sampled_writes_stored),
    );
    json.u64(
        "verification_candidate_comparisons",
        verification.verification_comparisons,
    );
    json.string(
        "verification_cost_policy",
        "bounded event history; per-key sort O(W log W), read frontier lookup O(log W), revision lookup O(1), ambiguous-value lookup O(1)",
    );
    if let Some(passed) = query_preflight_passed {
        json.bool("query_preflight_passed", passed);
    }
    json.u64("mutation_ops", measured.mutation_ops);
    json.u64(
        "attempted_reads",
        measured.attempted_gets + measured.attempted_queries + measured.attempted_scans,
    );
    json.u64(
        "successful_reads",
        measured.successful_gets + measured.successful_queries + measured.successful_scans,
    );
    json.u64(
        "attempted_write_transactions",
        measured.attempted_transactions,
    );
    json.u64(
        "successful_write_transactions",
        measured.successful_transactions,
    );
    json.f64(
        "successful_read_percent",
        measured.successful_reads() as f64 * 100.0 / measured.successful_operations().max(1) as f64,
    );
    json.f64(
        "successful_write_percent",
        measured.successful_transactions as f64 * 100.0
            / measured.successful_operations().max(1) as f64,
    );
    json.f64(
        "attempted_read_percent",
        (measured.attempted_gets + measured.attempted_queries + measured.attempted_scans) as f64
            * 100.0
            / measured.attempted_operations().max(1) as f64,
    );
    json.u64("returned_rows", measured.returned_rows);
    json.f64("logical_tx_per_second", logical_tx_per_second);
    json.f64("mutation_ops_per_second", mutation_ops_per_second);
    json.f64("get_ops_per_second", get_ops_per_second);
    json.f64("query_ops_per_second", query_ops_per_second);
    json.f64("scan_ops_per_second", scan_ops_per_second);
    json.f64("read_ops_per_second", read_ops_per_second);
    json.f64("aggregate_ops_per_second", aggregate_ops_per_second);
    json.f64("returned_rows_per_second", rows_per_second);
    for (prefix, samples) in [
        ("e2e", &measured.e2e_latency),
        ("write", &measured.write_latency),
        ("read", &measured.read_latency),
    ] {
        json.f64(
            &metric_field_name(prefix, "p50_us"),
            samples.percentile_us(0.50),
        );
        json.f64(
            &metric_field_name(prefix, "p95_us"),
            samples.percentile_us(0.95),
        );
        json.f64(
            &metric_field_name(prefix, "p99_us"),
            samples.percentile_us(0.99),
        );
        json.u64(&metric_field_name(prefix, "latency_samples"), samples.seen);
    }
    json.optional_f64("cpu_utilization_percent_one_core", cpu_one_core);
    json.optional_f64("cpu_utilization_percent_machine", cpu_machine);

    json.u64("groups", delta.groups);
    json.u64("queued_requests", delta.queued_requests);
    json.u64("logical_transactions_metric", delta.logical_transactions);
    json.f64("avg_group_requests", delta.avg_group_requests());
    json.f64(
        "avg_transactions_per_group",
        delta.avg_transactions_per_group(),
    );
    json.usize("max_actual_group_requests", delta.max_group_requests);
    json.usize("max_actual_group_bytes", max_actual_group_bytes);
    json.u64("queue_wait_nanos_total", delta.queue_wait_nanos);
    json.u64("collection_nanos_total", delta.collection_nanos);
    json.u64("processing_nanos_total", delta.processing_nanos);
    json.u64("validation_nanos_total", delta.validation_nanos);
    json.u64(
        "btree_preparation_nanos_total",
        delta.btree_preparation_nanos,
    );
    json.u64("publication_nanos_total", delta.publication_nanos);
    json.u64("wal_bytes_delta", delta.wal_bytes);
    json.u64("wal_syncs_delta", delta.wal_syncs);
    json.u64("materializations_delta", delta.materializations);
    json.u64(
        "materialization_total_nanos_delta",
        delta.materialization_total_nanos,
    );
    json.u64(
        "materialization_cpu_nanos_delta",
        delta.materialization_cpu_nanos,
    );
    json.u64("materialization_max_nanos", delta.materialization_max_nanos);
    json.u64("materialized_segments_delta", delta.materialized_segments);
    json.u64(
        "materialized_overlay_bytes_delta",
        delta.materialized_overlay_bytes,
    );
    json.u64(
        "materialized_data_bytes_delta",
        delta.materialized_data_bytes,
    );
    json.u64(
        "materialization_data_write_nanos_delta",
        delta.materialization_data_write_nanos,
    );
    json.u64(
        "materialization_data_sync_nanos_delta",
        delta.materialization_data_sync_nanos,
    );
    json.u64(
        "materialization_checkpoint_nanos_delta",
        delta.materialization_checkpoint_nanos,
    );
    json.u64(
        "materialization_checkpoint_sync_nanos_delta",
        delta.materialization_checkpoint_sync_nanos,
    );
    json.u64("publish_pause_nanos_delta", delta.publish_pause_nanos);
    json.u64("writer_blocked_nanos_delta", delta.writer_blocked_nanos);
    json.u64("backpressure_events_delta", delta.backpressure_events);
    json.u64("backpressure_nanos_delta", delta.backpressure_nanos);
    json.u64("materializer_requests_delta", delta.materializer_requests);
    json.u64("materializer_completed_delta", delta.materializer_completed);
    json.u64("materializer_failed_delta", delta.materializer_failed);
    json.u64("materializer_stale_delta", delta.materializer_stale);
    json.u64("overlay_segments_current", delta.overlay_segments_current);
    json.u64("overlay_segments_peak", delta.overlay_segments_peak);
    json.u64("overlay_bytes_current", delta.overlay_bytes_current);
    json.u64("overlay_bytes_peak", delta.overlay_bytes_peak);
    json.u64("wal_bytes_retained", delta.wal_bytes_retained);
    json.u64("wal_bytes_reclaimed_delta", delta.wal_bytes_reclaimed);
    json.u64(
        "materialization_wal_bytes_reclaimed_delta",
        delta.materialization_wal_bytes_reclaimed,
    );
    json.u64(
        "materialization_max_lag_transactions",
        delta.materialization_max_lag_transactions,
    );
    json.u64("wal_committed_batches_delta", delta.wal_committed_batches);
    json.u64("page_images_delta", delta.page_images);
    let redo = &delta.wal_redo;
    json.u64("wal_page_image_records_delta", redo.page_image_records);
    json.u64("wal_page_delta_records_delta", redo.page_delta_records);
    json.u64(
        "wal_page_delta_payload_bytes_delta",
        redo.page_delta_payload_bytes,
    );
    json.u64("wal_page_delta_spans_delta", redo.page_delta_spans);
    json.u64(
        "wal_page_delta_changed_bytes_delta",
        redo.page_delta_changed_bytes,
    );
    json.u64("wal_image_superblock_delta", redo.image_superblock);
    json.u64("wal_image_not_requested_delta", redo.image_not_requested);
    json.u64(
        "wal_image_page_image_format_delta",
        redo.image_page_image_format,
    );
    json.u64(
        "wal_image_ineligible_commit_delta",
        redo.image_ineligible_commit,
    );
    json.u64("wal_image_first_touch_delta", redo.image_first_touch);
    json.u64("wal_image_no_base_delta", redo.image_no_base);
    json.u64("wal_image_not_smaller_delta", redo.image_not_smaller);
    json.u64("wal_append_nanos_total", delta.wal_append_nanos);
    json.u64("wal_sync_nanos_total", delta.wal_sync_nanos);
    json.u64("wal_group_encode_nanos_total", delta.wal_group_encode_nanos);
    json.u64(
        "wal_group_page_lsn_validate_nanos_total",
        delta.wal_group_page_lsn_validate_nanos,
    );
    json.u64(
        "wal_group_page_image_materialize_nanos_total",
        delta.wal_group_page_image_materialize_nanos,
    );
    json.u64(
        "wal_group_page_image_validate_nanos_total",
        delta.wal_group_page_image_validate_nanos,
    );
    json.u64(
        "wal_group_digest_copy_nanos_total",
        delta.wal_group_digest_copy_nanos,
    );
    json.u64(
        "wal_group_page_payload_crc_nanos_total",
        delta.wal_group_page_payload_crc_nanos,
    );
    json.u64(
        "wal_group_page_header_crc_nanos_total",
        delta.wal_group_page_header_crc_nanos,
    );
    json.u64(
        "wal_group_page_frame_materialize_nanos_total",
        delta.wal_group_page_frame_materialize_nanos,
    );
    json.u64(
        "wal_group_page_frame_append_nanos_total",
        delta.wal_group_page_frame_append_nanos,
    );
    json.u64(
        "wal_group_page_direct_encode_nanos_total",
        delta.wal_group_page_direct_encode_nanos,
    );
    json.u64(
        "wal_group_commit_digest_crc_nanos_total",
        delta.wal_group_commit_digest_crc_nanos,
    );
    json.u64(
        "wal_group_commit_payload_crc_nanos_total",
        delta.wal_group_commit_payload_crc_nanos,
    );
    json.u64(
        "wal_group_commit_header_crc_nanos_total",
        delta.wal_group_commit_header_crc_nanos,
    );
    json.u64(
        "wal_group_commit_frame_materialize_nanos_total",
        delta.wal_group_commit_frame_materialize_nanos,
    );
    json.u64(
        "wal_group_commit_frame_append_nanos_total",
        delta.wal_group_commit_frame_append_nanos,
    );
    json.u64(
        "wal_group_commit_direct_encode_nanos_total",
        delta.wal_group_commit_direct_encode_nanos,
    );
    json.u64("wal_group_page_frames_delta", delta.wal_group_page_frames);
    json.u64(
        "wal_group_commit_frames_delta",
        delta.wal_group_commit_frames,
    );
    json.u64(
        "wal_group_page_validations_delta",
        delta.wal_group_page_validations,
    );
    json.u64("wal_group_write_nanos_total", delta.wal_group_write_nanos);
    json.u64(
        "wal_physical_write_calls_delta",
        delta.wal_physical_write_calls,
    );
    json.f64("transactions_per_sync", delta.transactions_per_sync());
    json.u64("leaf_splits", delta.leaf_splits);
    json.u64("internal_splits", delta.internal_splits);
    json.u64("root_splits", delta.root_splits);
    json.u64("right_link_corrections", delta.right_link_corrections);
    json.u64("pages_touched", delta.pages_touched);
    json.u64("blink_page_images", delta.blink_page_images);
    json.u64("leaf_splits_total", delta.leaf_splits_total);
    json.u64("internal_splits_total", delta.internal_splits_total);
    json.u64("root_splits_total", delta.root_splits_total);
    json.u64(
        "right_link_corrections_total",
        delta.right_link_corrections_total,
    );
    json.u64("generation_pins", delta.generation_pins);
    json.u64("read_operations_metric", delta.read_operations);
    json.u64("page_version_installs", delta.page_version_installs);
    json.u64("versions_retained", delta.versions_retained);
    json.u64("versions_reclaimed", delta.versions_reclaimed);
    json.u64("active_generation_pins", delta.active_generation_pins);
    json.u64("max_concurrent_pins", delta.max_concurrent_pins);
    json.u64("retired_page_ids", delta.retired_page_ids);
    json.u64("reusable_page_ids", delta.reusable_page_ids);
    json.u64(
        "versioned_right_link_corrections",
        delta.versioned_right_link_corrections,
    );
    json.u64("logical_groups", delta.logical_groups);
    json.u64("logical_transactions", delta.logical_transactions);
    json.u64("admitted_transactions", delta.admitted_transactions);
    json.u64("conflicted_transactions", delta.conflicted_transactions);
    json.u64("rejected_transactions", delta.rejected_transactions);
    json.u64("logical_admission_nanos", delta.logical_admission_nanos);
    json.u64("planning_nanos", delta.planning_nanos);
    json.u64("planner_route_nanos", delta.planner_route_nanos);
    json.u64("planner_route_calls", delta.planner_route_calls);
    json.u64("planner_route_page_visits", delta.planner_route_page_visits);
    json.u64(
        "planner_route_right_link_hops",
        delta.planner_route_right_link_hops,
    );
    json.u64("state_clone_nanos_total", delta.state_clone_nanos);
    json.u64("physical_execution_nanos", delta.physical_execution_nanos);
    json.u64(
        "physical_mutation_nanos_total",
        delta.physical_mutation_nanos,
    );
    json.u64("leaf_load_clone_nanos_total", delta.leaf_load_clone_nanos);
    json.u64(
        "leaf_entries_clone_nanos_total",
        delta.leaf_entries_clone_nanos,
    );
    json.u64(
        "leaf_install_clone_nanos_total",
        delta.leaf_install_clone_nanos,
    );
    json.u64("physical_restamp_nanos_total", delta.physical_restamp_nanos);
    json.u64(
        "physical_cached_refresh_nanos_total",
        delta.physical_cached_refresh_nanos,
    );
    json.u64(
        "physical_page_encode_nanos_total",
        delta.physical_page_encode_nanos,
    );
    json.u64(
        "physical_superblock_encode_nanos_total",
        delta.physical_superblock_encode_nanos,
    );
    json.u64("superblock_images_emitted", delta.superblock_images_emitted);
    json.u64("superblock_images_elided", delta.superblock_images_elided);
    json.u64("leaf_load_clones_delta", delta.leaf_load_clones);
    json.u64("leaf_entries_clones_delta", delta.leaf_entries_clones);
    json.u64("leaf_install_clones_delta", delta.leaf_install_clones);
    json.u64("cached_refresh_clones_delta", delta.cached_refresh_clones);
    json.u64("dirty_union_nanos_total", delta.dirty_union_nanos);
    json.u64("full_state_clones", delta.full_state_clones);
    json.f64(
        "full_state_clones_per_group",
        delta.full_state_clones as f64 / delta.groups.max(1) as f64,
    );
    json.f64(
        "full_state_clones_per_transaction",
        delta.full_state_clones as f64 / delta.logical_transactions.max(1) as f64,
    );
    json.u64("mutations_planned", delta.mutations_planned);
    json.u64("routes_calculated", delta.routes_calculated);
    json.u64("route_reuses", delta.route_reuses);
    json.u64("route_invalidations", delta.route_invalidations);
    json.u64("reroutes", delta.reroutes);
    json.u64("leaf_groups", delta.leaf_groups);
    json.u64("same_leaf_groups", delta.same_leaf_groups);
    json.u64("mutations_per_leaf_group", delta.mutations_per_leaf_group);
    json.u64("coalesced_mutations", delta.coalesced_mutations);
    json.u64("independent_leaf_groups", delta.independent_leaf_groups);
    json.u64("dependency_edges", delta.dependency_edges);
    json.u64("leaf_loads", delta.leaf_loads);
    json.u64("leaf_encodes", delta.leaf_encodes);
    json.u64("structural_fallbacks", delta.structural_fallbacks);
    json.u64("split_triggered_reroutes", delta.split_triggered_reroutes);
    json.u64("coalescing_interruptions", delta.coalescing_interruptions);
    json.u64("planner_page_images", delta.planner_page_images);
    json.u64("planner_wal_bytes", delta.planner_wal_bytes);
    json.u64(
        "catalog_construction_nanos",
        delta.catalog_construction_nanos,
    );
    json.u64(
        "catalog_map_clone_nanos_total",
        delta.catalog_map_clone_nanos,
    );
    json.u64(
        "catalog_directory_clone_nanos_total",
        delta.catalog_directory_clone_nanos,
    );
    json.u64(
        "catalog_chunk_clone_nanos_total",
        delta.catalog_chunk_clone_nanos,
    );
    json.u64("catalog_chunk_clones_delta", delta.catalog_chunk_clones);
    json.u64("catalog_chunk_size", 64);
    json.u64(
        "catalog_state_scan_nanos_total",
        delta.catalog_state_scan_nanos,
    );
    json.u64("wal_assembly_nanos_total", delta.wal_assembly_nanos);
    json.u64("state_install_nanos_total", delta.state_install_nanos);
    json.u64(
        "generation_publication_nanos",
        delta.generation_publication_nanos,
    );
    json.u64("publication_swap_nanos_total", delta.publication_swap_nanos);
    json.u64(
        "retired_generation_drop_nanos_total",
        delta.retired_generation_drop_nanos,
    );
    json.u64("dirty_tracking_nanos_total", delta.dirty_tracking_nanos);
    json.u64("parallel_groups_delta", delta.parallel_groups);
    json.u64("parallel_leaf_jobs_delta", delta.parallel_leaf_jobs);
    json.u64("parallel_transactions_delta", delta.parallel_transactions);
    json.u64("parallel_mutations_delta", delta.parallel_mutations);
    json.u64(
        "parallel_worker_dispatches_delta",
        delta.parallel_worker_dispatches,
    );
    json.u64(
        "parallel_background_worker_dispatches_delta",
        delta.parallel_background_worker_dispatches,
    );
    json.u64("parallel_worker_nanos_total", delta.parallel_worker_nanos);
    json.u64("parallel_join_nanos_total", delta.parallel_join_nanos);
    if delta.parallel_join_nanos > 0 {
        json.f64(
            "effective_worker_parallelism",
            delta.parallel_worker_nanos as f64 / delta.parallel_join_nanos as f64,
        );
    }
    json.u64(
        "parallel_fallback_groups_delta",
        delta.parallel_fallback_groups,
    );
    json.u64(
        "parallel_job_operations_delta",
        delta.parallel_job_operations,
    );
    json.u64(
        "parallel_dispatch_nanos_total",
        delta.parallel_dispatch_nanos,
    );
    json.u64("parallel_collect_nanos_total", delta.parallel_collect_nanos);
    json.u64(
        "parallel_worker_slot_nanos_total",
        delta.parallel_worker_slot_nanos,
    );
    json.u64(
        "parallel_coordinator_lane_nanos_total",
        delta.parallel_coordinator_lane_nanos,
    );
    json.u64(
        "parallel_worker_base_nanos_total",
        delta.parallel_worker_base_nanos,
    );
    json.u64(
        "parallel_worker_mutation_nanos_total",
        delta.parallel_worker_mutation_nanos,
    );
    json.u64(
        "parallel_worker_encode_nanos_total",
        delta.parallel_worker_encode_nanos,
    );
    json.u64(
        "parallel_worker_delta_nanos_total",
        delta.parallel_worker_delta_nanos,
    );
    json.u64(
        "parallel_fallback_after_dispatch_delta",
        delta.parallel_fallback_after_dispatch,
    );
    json.u64(
        "parallel_fallback_no_delta_wal_delta",
        delta.parallel_fallback_no_delta_wal,
    );
    json.u64(
        "parallel_fallback_route_delta",
        delta.parallel_fallback_route,
    );
    json.u64(
        "parallel_fallback_overflow_delta",
        delta.parallel_fallback_overflow,
    );
    json.u64(
        "parallel_fallback_structural_delta",
        delta.parallel_fallback_structural,
    );
    json.u64(
        "parallel_skipped_single_leaf_delta",
        delta.parallel_skipped_single_leaf,
    );
    json.u64(
        "parallel_skipped_small_group_delta",
        delta.parallel_skipped_small_group,
    );
    json.u64(
        "structural_transactions_delta",
        delta.structural_transactions,
    );
    json.u64("wal_redo_plan_nanos_total", delta.wal_redo_plan_nanos);
    json.string(
        "tx_mutation_histogram",
        &histogram_text(&delta.transaction_mutation_histogram),
    );
    json.string(
        "tx_dirty_page_histogram",
        &histogram_text(&delta.transaction_dirty_page_histogram),
    );
    json.string(
        "tx_leaf_page_histogram",
        &histogram_text(&delta.transaction_leaf_page_histogram),
    );
    json.string(
        "component_timing_scope",
        "existing cumulative coordinator/storage/WAL metrics; per-request component percentiles unavailable without production hot-path instrumentation",
    );
    json.string(
        "churn_counters",
        if dodb_storage::churn::ENABLED {
            "enabled"
        } else {
            "disabled"
        },
    );
    for (name, value) in churn {
        json.u64(name, *value);
    }
    if let Some(window_seconds) = args.window_seconds {
        let mut timeline = measured.window_timeline.clone();
        timeline.sort_unstable();
        let window_nanos = window_seconds * 1_000_000_000;
        let wall_nanos = wall.as_nanos() as u64;
        let window_count = wall_nanos.div_ceil(window_nanos);
        json.u64("window_seconds", window_seconds);
        json.u64("window_count", window_count);
        for window_index in 0..window_count {
            let start = window_index * window_nanos;
            let end = start + window_nanos;
            let mut latencies: Vec<u64> = timeline
                .iter()
                .filter(|(finish, _)| *finish >= start && *finish < end)
                .map(|(_, latency)| *latency)
                .collect();
            latencies.sort_unstable();
            let span = (end.min(wall_nanos) - start) as f64 / 1e9;
            let percentile = |fraction: f64| {
                if latencies.is_empty() {
                    0.0
                } else {
                    latencies[((latencies.len() - 1) as f64 * fraction).round() as usize] as f64
                        / 1_000.0
                }
            };
            json.u64(
                &format!("window_{window_index:02}_successful_transactions"),
                latencies.len() as u64,
            );
            json.f64(&format!("window_{window_index:02}_span_seconds"), span);
            json.f64(
                &format!("window_{window_index:02}_logical_tx_per_second"),
                latencies.len() as f64 / span,
            );
            json.f64(
                &format!("window_{window_index:02}_p50_us"),
                percentile(0.50),
            );
            json.f64(
                &format!("window_{window_index:02}_p95_us"),
                percentile(0.95),
            );
            json.f64(
                &format!("window_{window_index:02}_p99_us"),
                percentile(0.99),
            );
        }
        let checkpoint_events = CHECKPOINT_EVENTS
            .lock()
            .map(|events| {
                events
                    .iter()
                    .filter(|event| event.finished >= measurement_started)
                    .map(|event| {
                        (
                            (event.finished - measurement_started).as_nanos() as u64,
                            *event,
                        )
                    })
                    .filter(|(offset, _)| *offset <= wall_nanos)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        json.u64(
            "checkpoint_wal_bytes_threshold",
            CHECKPOINT_WAL_BYTES.load(Ordering::Relaxed),
        );
        json.u64("checkpoint_count", checkpoint_events.len() as u64);
        json.u64(
            "checkpoint_total_nanos",
            checkpoint_events
                .iter()
                .map(|(_, event)| event.duration_nanos)
                .sum(),
        );
        json.u64(
            "checkpoint_max_nanos",
            checkpoint_events
                .iter()
                .map(|(_, event)| event.duration_nanos)
                .max()
                .unwrap_or(0),
        );
        json.u64(
            "checkpoint_wal_bytes_reclaimed",
            checkpoint_events
                .iter()
                .map(|(_, event)| event.wal_bytes_reclaimed)
                .sum(),
        );
        json.string(
            "checkpoint_events",
            &checkpoint_events
                .iter()
                .map(|(offset, event)| {
                    format!(
                        "{:.3}s:{:.1}ms:{}B",
                        *offset as f64 / 1e9,
                        event.duration_nanos as f64 / 1e6,
                        event.wal_bytes_before
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        );
        for window_index in 0..window_count {
            let start = window_index * window_nanos;
            let end = start + window_nanos;
            json.u64(
                &format!("window_{window_index:02}_checkpoints"),
                checkpoint_events
                    .iter()
                    .filter(|(offset, _)| *offset >= start && *offset < end)
                    .count() as u64,
            );
        }
        json.u64("resource_sample_count", samples.len() as u64);
        for (sample_index, sample) in samples.iter().enumerate() {
            let prefix = format!("resource_sample_{sample_index:02}");
            let since_measurement_start = if sample.taken_at >= measurement_started {
                (sample.taken_at - measurement_started).as_secs_f64()
            } else {
                -(measurement_started - sample.taken_at).as_secs_f64()
            };
            json.string(&format!("{prefix}_label"), &sample.label);
            json.f64(
                &format!("{prefix}_since_measurement_start_seconds"),
                since_measurement_start,
            );
            json.u64(&format!("{prefix}_unix_ms"), sample.unix_ms as u64);
            json.u64(&format!("{prefix}_rss_kib"), sample.rss_kib);
            json.u64(&format!("{prefix}_wal_bytes"), sample.wal_bytes);
            json.u64(
                &format!("{prefix}_wal_committed_batches"),
                sample.wal_committed_batches,
            );
            json.u64(&format!("{prefix}_wal_page_images"), sample.wal_page_images);
            json.u64(
                &format!("{prefix}_retained_recovery_batches"),
                sample.retained_recovery_batches,
            );
            json.u64(
                &format!("{prefix}_retained_recovery_page_images"),
                sample.retained_recovery_page_images,
            );
            json.u64(
                &format!("{prefix}_wal_page_image_records"),
                sample.wal_page_image_records,
            );
            json.u64(
                &format!("{prefix}_wal_page_delta_records"),
                sample.wal_page_delta_records,
            );
            json.u64(&format!("{prefix}_wal_syncs"), sample.wal_syncs);
            json.u64(&format!("{prefix}_wal_sync_nanos"), sample.wal_sync_nanos);
            json.u64(&format!("{prefix}_dirty_pages"), sample.dirty_pages);
            json.u64(
                &format!("{prefix}_logical_transactions"),
                sample.logical_transactions,
            );
        }
    }
    json.finish()
}

fn current_git_commit() -> String {
    command_output("git", &["rev-parse", "HEAD"])
}

fn current_git_dirty() -> Option<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=all"])
        .output()
        .ok()?;
    output.status.success().then(|| !output.stdout.is_empty())
}

fn benchmark_source_sha256() -> String {
    sha256_bytes(include_bytes!("phase0-bench.rs"))
}

fn cargo_lock_sha256() -> String {
    sha256_bytes(include_bytes!("../../../../../Cargo.lock"))
}

fn current_binary_sha256() -> String {
    std::env::current_exe()
        .ok()
        .map(|path| sha256_file(&path))
        .unwrap_or_else(|| "unknown".to_owned())
}

fn sha256_file(path: &Path) -> String {
    let output = Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .or_else(|| {
            Command::new("sha256sum")
                .arg(path)
                .output()
                .ok()
                .filter(|output| output.status.success())
        });
    output
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_owned()
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let hash = [("shasum", vec!["-a", "256"]), ("sha256sum", Vec::new())]
        .into_iter()
        .find_map(|(command, arguments)| {
            let mut child = Command::new(command)
                .args(arguments)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .ok()?;
            child.stdin.take()?.write_all(bytes).ok()?;
            let output = child.wait_with_output().ok()?;
            if !output.status.success() {
                return None;
            }
            String::from_utf8_lossy(&output.stdout)
                .split_whitespace()
                .next()
                .map(str::to_owned)
        });
    hash.unwrap_or_else(|| "unknown".to_owned())
}

fn print_run_summary(
    scenario: &Scenario,
    repetition: usize,
    measured: &WorkerStats,
    wall: Duration,
    delta: &MetricDelta,
) {
    let seconds = wall.as_secs_f64().max(f64::EPSILON);
    println!(
        "{:<58} rep={} tx/s={:>9.0} mut/s={:>9.0} read/s={:>9.0} agg/s={:>9.0} p50={:>8.1}us p95={:>8.1}us p99={:>8.1}us groups={:>5} avg_group={:>5.2}",
        scenario.name(),
        repetition,
        measured.successful_transactions as f64 / seconds,
        measured.mutation_ops as f64 / seconds,
        measured.successful_reads() as f64 / seconds,
        measured.successful_operations() as f64 / seconds,
        measured.e2e_latency.percentile_us(0.50),
        measured.e2e_latency.percentile_us(0.95),
        measured.e2e_latency.percentile_us(0.99),
        delta.groups,
        delta.avg_group_requests(),
    );
}

async fn run_repetition(
    args: &Args,
    scenario: &Scenario,
    machine: &MachineInfo,
    scenario_index: usize,
    repetition: usize,
    seed: u64,
    output: &mut std::fs::File,
) -> Result<()> {
    let (adapter, data_path, seeded_rows) = open_adapter(args, scenario, repetition, seed).await?;
    let mut samples = Vec::new();
    if args.window_seconds.is_some() {
        samples.push(ResourceSample::capture(&*adapter, "seeded".to_string()));
    }
    let query_preflight_passed = if scenario.read_kind == Some(ReadKind::Query) {
        let request = query_range_for_index(0, args.read_limit).request(args.key_size);
        match adapter.execute(request.clone()).await? {
            BatchResponse::Query(rows) => Some(query_result_is_valid(
                &request,
                &rows,
                args.key_size,
                args.value_size,
                args.read_limit,
            )),
            _ => Some(false),
        }
    } else {
        None
    };
    if query_preflight_passed == Some(false) {
        adapter.shutdown().await?;
        remove_database_files(&data_path);
        return Err(Error::invariant(
            "Query result contract verification failed",
        ));
    }
    let verifies_gets = scenario.mix.is_some() || scenario.read_kind == Some(ReadKind::Get);
    let sampled_keys = if verifies_gets && scenario.readers > 0 {
        sampled_read_keys(args, scenario)
    } else {
        Arc::new(HashSet::new())
    };
    if verifies_gets {
        if let Err(error) = capture_sampled_seed_states(&adapter, args, &sampled_keys, true).await {
            let _ = adapter.shutdown().await;
            remove_database_files(&data_path);
            return Err(error);
        }
    }
    let _warmup = run_interval(
        Arc::clone(&adapter),
        args,
        scenario,
        seed ^ 0xaaaa_0000,
        args.warmup,
        true,
        Arc::new(HashSet::new()),
        None,
    )
    .await;
    #[cfg(feature = "phase-i-instrumentation")]
    let _ = dodb_storage::blink::take_phase_i_group_locality_samples();
    if args.window_seconds.is_some() {
        samples.push(ResourceSample::capture(
            &*adapter,
            "measurement_start".to_string(),
        ));
    }
    adapter.reset_checkpoint_metrics();
    let churn_before = dodb_storage::churn::snapshot();
    let (leaf_sample_start, _) = dodb_storage::churn::leaf_samples_since(usize::MAX);
    let sampler = args.window_seconds.map(|window_seconds| {
        ResourceSampler::start(
            Arc::clone(&adapter),
            Instant::now(),
            Duration::from_secs(window_seconds),
        )
    });
    let seed_states = if verifies_gets {
        match capture_sampled_seed_states(&adapter, args, &sampled_keys, false).await {
            Ok(seed_states) => seed_states,
            Err(error) => {
                let _ = adapter.shutdown().await;
                remove_database_files(&data_path);
                return Err(error);
            }
        }
    } else {
        HashMap::new()
    };
    let verifier = verifies_gets.then(|| {
        Arc::new(Mutex::new(SampledReadVerifier::new(
            &sampled_keys,
            seed_states,
        )))
    });
    let before = adapter.snapshot();
    let measured_interval = run_interval(
        Arc::clone(&adapter),
        args,
        scenario,
        seed ^ 0xbbbb_0000,
        args.duration,
        false,
        Arc::clone(&sampled_keys),
        verifier.clone(),
    )
    .await;
    let wall = measured_interval.finished_at - measured_interval.started_at;
    let cpu_end = ProcessCpuSample::capture();
    if let Some(sampler) = sampler {
        samples.extend(sampler.finish());
        samples.push(ResourceSample::capture(
            &*adapter,
            "measurement_end".to_string(),
        ));
    }
    let churn_after = dodb_storage::churn::snapshot();
    let after = adapter.snapshot();
    let delta = MetricDelta::from(&before, &after);
    let mut churn = churn_delta(&churn_before, &churn_after);
    let (_, leaf_samples) = dodb_storage::churn::leaf_samples_since(leaf_sample_start);
    churn.extend(leaf_sample_summary(&leaf_samples));
    let verification = if let Some(verifier) = verifier {
        verifier
            .lock()
            .expect("sample verifier lock should not be poisoned")
            .summarize()
    } else {
        VerificationSummary::not_applicable()
    };
    print_run_summary(scenario, repetition, &measured_interval.stats, wall, &delta);
    let line = build_record(
        machine,
        args,
        scenario,
        scenario_index,
        repetition,
        seed,
        seeded_rows,
        &measured_interval.stats,
        &delta,
        verification,
        &measured_interval.process_rss,
        query_preflight_passed,
        measured_interval.quota_slots_claimed,
        wall,
        &measured_interval.cpu_start,
        &cpu_end,
        measured_interval.started_at,
        &samples,
        &churn,
    );
    #[cfg(feature = "phase-i-instrumentation")]
    write_phase_i_locality_samples(args, scenario, repetition, seed);
    use std::io::Write;
    writeln!(output, "{line}")?;
    output.flush()?;
    adapter.shutdown().await?;
    remove_database_files(&data_path);
    if scenario.read_kind == Some(ReadKind::Query) {
        measured_query_verification_result(&measured_interval.stats)?;
    }
    match verification.status {
        VerificationStatus::Failed => {
            return Err(Error::invariant("Sampled read verification failed"));
        }
        VerificationStatus::Inconclusive => {
            return Err(Error::invariant(
                "Sampled read verification was inconclusive",
            ));
        }
        VerificationStatus::NotApplicable | VerificationStatus::Passed => {}
    }
    Ok(())
}

async fn capture_sampled_seed_states(
    adapter: &Arc<dyn EngineAdapter>,
    args: &Args,
    sampled_keys: &HashSet<DocumentKey>,
    validate_seed_value: bool,
) -> Result<HashMap<DocumentKey, SampledSeed>> {
    let mut seed_states = HashMap::with_capacity(sampled_keys.len());
    for key in sampled_keys {
        let index = key
            .sk
            .as_bytes()
            .get(key.sk.as_bytes().len().saturating_sub(8)..)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_be_bytes)
            .filter(|index| *index < args.working_set as u64)
            .ok_or_else(|| Error::invariant("sampled key has no valid seeded index"))?;
        let expected_value = value_bytes(args.value_size, index, 0);
        match adapter
            .execute(BatchRequest::Get { key: key.clone() })
            .await?
        {
            BatchResponse::Get(RevisionState::Present { value, revision })
                if !validate_seed_value || value == expected_value =>
            {
                seed_states.insert(key.clone(), SampledSeed { value, revision });
            }
            BatchResponse::Get(_) => {
                return Err(Error::invariant(
                    "sampled seed key was missing or contained an unexpected value",
                ));
            }
            _ => {
                return Err(Error::invariant(
                    "sampled seed Get returned an unexpected response",
                ));
            }
        }
    }
    Ok(seed_states)
}

fn open_output(path: &Path) -> std::fs::File {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("benchmark output directory should be creatable");
    }
    std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .unwrap_or_else(|error| panic!("benchmark output should open: {error}"))
}

fn run(args: Args) -> Result<()> {
    let machine = MachineInfo::collect();
    println!(
        "phase0-bench engine={} build={} cpu={:?} logical_cpus={} tokio_workers={} duration={:?} warmup={:?} repetitions={}",
        args.engine.as_str(),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        machine.cpu_model,
        machine.logical_cpus,
        args.tokio_workers,
        args.duration,
        args.warmup,
        args.repetitions,
    );
    println!("raw output: {}", args.output.display());
    println!(
        "scenario                                                       repetition throughput summary"
    );
    let mut output = open_output(&args.output);
    let scenarios = scenarios(&args);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(args.tokio_workers)
        .enable_all()
        .build()
        .map_err(|error| Error::invariant(format!("benchmark runtime should build: {error}")))?;
    runtime.block_on(async {
        for (scenario_index, scenario) in scenarios.iter().enumerate() {
            for repetition in 0..args.repetitions {
                let seed = invocation_seed(args.seed, scenario_index, repetition);
                run_repetition(
                    &args,
                    scenario,
                    &machine,
                    scenario_index,
                    repetition,
                    seed,
                    &mut output,
                )
                .await?;
            }
        }
        Ok::<(), Error>(())
    })?;
    println!(
        "completed {} scenarios x {} repetitions",
        scenarios.len(),
        args.repetitions
    );
    Ok(())
}

fn main() {
    if let Err(error) = run(Args::parse()) {
        eprintln!("phase0-bench failed: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_validation_checks_point_and_distributed_query_value_contents() {
        let mut corrupted_value = vec![7; 512];
        assert!(value_matches_byte(&corrupted_value, 512, 7));
        corrupted_value[255] ^= 1;
        assert!(!value_matches_byte(&corrupted_value, 512, 7));

        let args = Args::default();
        let range = query_range_for_index(7, args.read_limit);
        let request = range.request(args.key_size);
        let rows: Vec<dodb_storage::Document> = (range.start_index
            ..range.start_index + range.limit)
            .map(|row_index| dodb_storage::Document {
                key: distributed_query_key(args.key_size, range.pk_index, row_index),
                value: value_bytes(
                    args.value_size,
                    distributed_query_value_index(range.pk_index, row_index, args.read_limit),
                    0,
                ),
                revision: dodb_core::Revision::ZERO,
            })
            .collect();
        assert!(query_result_is_valid(
            &request,
            &rows,
            args.key_size,
            args.value_size,
            args.read_limit,
        ));
    }

    #[test]
    fn latency_samples_keep_a_bounded_deterministic_reservoir() {
        let mut samples = LatencySamples::with_seed(0x1234_5678);
        for sample_index in 0..(LATENCY_RESERVOIR_LIMIT * 4) {
            samples.push(Duration::from_nanos(sample_index as u64));
        }

        assert_eq!(samples.values.len(), LATENCY_RESERVOIR_LIMIT);
        assert_eq!(samples.seen, (LATENCY_RESERVOIR_LIMIT * 4) as u64);
        assert!(samples.percentile_us(0.99).is_finite());
    }

    #[test]
    fn parallel_background_threshold_defaults_to_zero() {
        let args = Args::parse_from(Vec::<String>::new());
        assert_eq!(args.parallel_background_min_operations, 0);
    }

    #[test]
    fn parallel_background_threshold_cli_value_is_reported_in_json() {
        let args =
            Args::parse_from(["--parallel-background-min-operations", "17"].map(str::to_owned));
        assert_eq!(args.parallel_background_min_operations, 17);
        let mut json = JsonObject::new();
        add_parallel_metadata(&mut json, &args);
        let record = json.finish();
        assert!(record.contains("\"parallel_background_min_operations\":17"));
        let expected_read_metric_flag = format!(
            "\"blink_read_observational_metrics_enabled\":{}",
            !cfg!(feature = "blink-read-metrics-disabled")
        );
        assert!(record.contains(&expected_read_metric_flag));
    }

    #[test]
    fn mixed_schedule_preserves_exact_requested_ratio() {
        for read_percent in [95, 50, 20] {
            let successful_read_slots = (0..10_000)
                .filter(|operation_index| mixed_operation_is_read(*operation_index, read_percent))
                .count();
            assert_eq!(successful_read_slots, 100 * usize::from(read_percent));
        }
    }

    #[test]
    fn mixed_trace_prefix_matches_external_adapter_vector() {
        let trace_hash = mixed_trace_prefix_hash(
            WorkloadConfig {
                distribution: Distribution::Uniform,
                working_set: 10_000,
                key_size: 16,
                value_size: 512,
                width: 4,
                transaction_mode: TransactionMode::Unconditional,
                read_limit: 16,
            },
            0x1234_5678_9abc_def0,
            95,
            MixedValueMode::Constant,
            1_000,
        );
        assert_eq!(trace_hash, 0x3cca_5e07_ae2e_0ae5);
    }

    #[test]
    fn changing_mixed_trace_and_values_are_seeded() {
        let workload_config = WorkloadConfig {
            distribution: Distribution::Uniform,
            working_set: 10_000,
            key_size: 16,
            value_size: 512,
            width: 4,
            transaction_mode: TransactionMode::Unconditional,
            read_limit: 16,
        };
        let phase_seed = 0x1234_5678_9abc_def0;
        let trace_hash = mixed_trace_prefix_hash(
            workload_config.clone(),
            phase_seed,
            95,
            MixedValueMode::Changing,
            1_000,
        );
        assert_eq!(trace_hash, 0xd6ea_8c55_50fc_11e1);
        assert_ne!(
            trace_hash,
            mixed_trace_prefix_hash(
                workload_config.clone(),
                phase_seed,
                95,
                MixedValueMode::Changing,
                999,
            )
        );
        assert_ne!(
            trace_hash,
            mixed_trace_prefix_hash(
                workload_config.clone(),
                phase_seed,
                95,
                MixedValueMode::Constant,
                1_000,
            )
        );

        let value_for = |operation_index| {
            let operation_seed = mixed_operation_seed(phase_seed, operation_index);
            let mut generator = WorkloadGenerator::new_mixed(
                workload_config.clone(),
                operation_seed,
                phase_seed,
                operation_index,
                MixedValueMode::Changing,
            );
            let TransactionMutation::Put { value, .. } =
                generator.next_transaction().mutations.remove(0)
            else {
                unreachable!();
            };
            value
        };
        let first_value = value_for(0);
        assert_eq!(first_value.len(), 512);
        assert_eq!(first_value, value_for(0));
        assert_ne!(first_value, value_for(1));
        assert_eq!(first_value[..8], 0u64.to_be_bytes());
        assert_eq!(value_for(1)[..8], 1u64.to_be_bytes());
    }

    #[test]
    fn constant_mixed_values_keep_the_existing_bytes() {
        let phase_seed = 0x1234_5678_9abc_def0;
        for operation_index in [3, 991] {
            let operation_seed = mixed_operation_seed(phase_seed, operation_index);
            let mut generator = WorkloadGenerator::new_mixed(
                workload(Distribution::Uniform, 1),
                operation_seed,
                phase_seed,
                operation_index,
                MixedValueMode::Constant,
            );
            let TransactionMutation::Put { value, .. } =
                generator.next_transaction().mutations.remove(0)
            else {
                unreachable!();
            };
            assert_eq!(value, vec![1; 64]);
        }
    }

    #[test]
    fn mixed_value_modes_parse_and_keep_current_collection_default() {
        assert_eq!(MixedValueMode::parse("constant"), MixedValueMode::Constant);
        assert_eq!(MixedValueMode::parse("changing"), MixedValueMode::Changing);
        assert_eq!(
            CollectionPolicy::parse("main-parity"),
            CollectionPolicy::MainParity
        );
        assert_eq!(Args::default().collection_policy, CollectionPolicy::Current);
    }

    #[test]
    fn changing_mixed_values_are_deterministic_and_nonconstant() {
        let config = workload(Distribution::Uniform, 1);
        let phase_seed = 0x1234_5678_9abc_def0;
        let value_for = |operation_index| {
            let operation_seed = mixed_operation_seed(phase_seed, operation_index);
            let mut generator = WorkloadGenerator::new_mixed(
                config.clone(),
                operation_seed,
                phase_seed,
                operation_index,
                MixedValueMode::Changing,
            );
            let TransactionMutation::Put { value, .. } =
                generator.next_transaction().mutations.remove(0)
            else {
                unreachable!();
            };
            value
        };
        let first_value = value_for(0);
        let second_value = value_for(1);
        assert_eq!(first_value.len(), 64);
        assert_eq!(first_value, value_for(0));
        assert_ne!(first_value, second_value);
        assert!(first_value[8..].iter().any(|byte| *byte != first_value[8]));
        assert_eq!(first_value[..8], 0u64.to_be_bytes());
        assert_eq!(second_value[..8], 1u64.to_be_bytes());
    }

    fn queued_blink_work(tag: u8, value_size: usize) -> BlinkWork {
        let (response, _receiver) = tokio::sync::oneshot::channel();
        BlinkWork {
            request: TransactionRequest::new(
                Vec::new(),
                vec![TransactionMutation::Put {
                    key: DocumentKey::new(vec![0x51; 8], vec![tag; 8]),
                    value: vec![tag; value_size],
                }],
            ),
            enqueued: Instant::now(),
            response,
        }
    }

    fn queued_blink_work_tag(work: &BlinkWork) -> u8 {
        let TransactionMutation::Put { value, .. } = &work.request.mutations[0] else {
            unreachable!();
        };
        value[0]
    }

    #[tokio::test(flavor = "current_thread")]
    async fn main_parity_collection_preserves_fifo_at_the_byte_boundary() {
        let first = queued_blink_work(1, 16);
        let second = queued_blink_work(2, 16);
        let third = queued_blink_work(3, 16);
        let fourth = queued_blink_work(4, 16);
        let group_bytes =
            transaction_request_size(&first.request) + transaction_request_size(&second.request);
        let (sender, mut receiver) = tokio::sync::mpsc::channel(4);
        sender.send(second).await.unwrap();
        sender.send(third).await.unwrap();
        sender.send(fourth).await.unwrap();
        let config = CoordinatorConfig {
            max_group_requests: 8,
            max_group_bytes: group_bytes,
            max_collection_delay: Duration::ZERO,
            ..CoordinatorConfig::default()
        };

        let (batch, pending, actual_bytes) =
            collect_main_parity_group(first, &mut receiver, config).await;

        assert_eq!(
            batch.iter().map(queued_blink_work_tag).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(actual_bytes, group_bytes);
        assert_eq!(pending.as_ref().map(queued_blink_work_tag), Some(3));
        assert_eq!(queued_blink_work_tag(&receiver.try_recv().unwrap()), 4);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn main_parity_collection_keeps_an_oversized_first_request_alone() {
        let first = queued_blink_work(1, 16);
        let second = queued_blink_work(2, 16);
        let first_bytes = transaction_request_size(&first.request);
        let (sender, mut receiver) = tokio::sync::mpsc::channel(2);
        sender.send(second).await.unwrap();
        let config = CoordinatorConfig {
            max_group_requests: 8,
            max_group_bytes: first_bytes - 1,
            max_collection_delay: Duration::ZERO,
            ..CoordinatorConfig::default()
        };

        let (batch, pending, actual_bytes) =
            collect_main_parity_group(first, &mut receiver, config).await;

        assert_eq!(batch.len(), 1);
        assert_eq!(queued_blink_work_tag(&batch[0]), 1);
        assert_eq!(actual_bytes, first_bytes);
        assert_eq!(pending.as_ref().map(queued_blink_work_tag), Some(2));
    }

    #[test]
    fn rss_window_excludes_outside_samples_and_tracks_growth_and_intervals() {
        let started_at = Instant::now();
        let mut accumulator =
            RssWindowAccumulator::new(started_at, Duration::from_millis(10), true);
        assert!(!accumulator.record(started_at - Duration::from_millis(1), Ok(900),));
        assert!(accumulator.record(started_at + Duration::from_millis(2), Ok(100)));
        assert!(accumulator.record(started_at + Duration::from_millis(12), Ok(250)));
        assert!(accumulator.record(started_at + Duration::from_millis(22), Err(())));
        assert!(accumulator.record(started_at + Duration::from_millis(32), Ok(400)));
        let finished_at = started_at + Duration::from_millis(34);
        accumulator.finished_at = Some(finished_at);
        assert!(!accumulator.record(finished_at + Duration::from_nanos(1), Ok(1_000)));

        let measurement = accumulator.finish(finished_at);
        assert_eq!(measurement.status, "partial");
        assert_eq!(measurement.start_rss_bytes, Some(100));
        assert_eq!(measurement.end_rss_bytes, Some(400));
        assert_eq!(measurement.peak_rss_bytes, Some(400));
        assert_eq!(measurement.sample_count, 3);
        assert_eq!(measurement.collection_failures, 1);
        assert_eq!(
            measurement.max_sample_interval,
            Some(Duration::from_millis(10))
        );
        assert_eq!(
            measurement.start_sample_offset,
            Some(Duration::from_millis(2))
        );
        assert_eq!(
            measurement.end_sample_offset,
            Some(Duration::from_millis(32))
        );
        assert_eq!(
            measurement.peak_sample_offset,
            Some(Duration::from_millis(32))
        );
    }

    #[test]
    fn rss_failures_and_unsupported_status_never_serialize_as_zero_bytes() {
        let started_at = Instant::now();
        let mut failed = RssWindowAccumulator::new(started_at, Duration::from_millis(10), true);
        failed.record(started_at + Duration::from_millis(1), Err(()));
        let failed = failed.finish(started_at + Duration::from_millis(2));
        assert_eq!(failed.status, "failed");
        assert_eq!(failed.start_rss_bytes, None);
        assert_eq!(failed.end_rss_bytes, None);
        assert_eq!(failed.peak_rss_bytes, None);
        assert_eq!(failed.sample_count, 0);
        assert_eq!(failed.collection_failures, 1);

        let unsupported = RssWindowAccumulator::new(started_at, Duration::from_millis(10), false)
            .finish(started_at + Duration::from_millis(2));
        assert_eq!(unsupported.status, "unsupported");
        assert_eq!(unsupported.start_rss_bytes, None);
        assert_eq!(unsupported.sample_count, 0);
        assert_eq!(unsupported.collection_failures, 0);
        assert_eq!(duration_to_nanos(Duration::from_micros(7)), 7_000);
        assert_eq!(resident_pages_to_bytes(5, 4_096), Some(20_480));
        assert_eq!(resident_pages_to_bytes(u64::MAX, 4_096), None);
    }

    #[test]
    fn platform_rss_reader_returns_bytes_for_the_current_process() {
        if process_rss_supported() {
            assert!(process_rss_bytes().unwrap() > 0);
        }
    }

    #[test]
    fn rss_sampler_runs_independently_while_caller_is_busy() {
        let mut sampler = ProcessRssSampler::spawn(Duration::from_millis(5));
        let started_at = Instant::now();
        sampler.start(started_at);
        let deadline = started_at + Duration::from_millis(60);
        let mut work_value = 1u64;
        while Instant::now() < deadline {
            work_value = work_value.wrapping_mul(3).wrapping_add(1);
        }
        std::hint::black_box(work_value);
        let (finished_at, measurement) = sampler.finish();
        assert_eq!(measurement.status, "complete");
        assert!(measurement.sample_count >= 2);
        assert_eq!(measurement.collection_failures, 0);
        assert!(measurement.start_sample_offset.unwrap() <= finished_at - started_at);
        assert!(measurement.end_sample_offset.unwrap() <= finished_at - started_at);
    }

    fn workload(distribution: Distribution, width: usize) -> WorkloadConfig {
        WorkloadConfig {
            distribution,
            working_set: 256,
            key_size: 16,
            value_size: 64,
            width,
            transaction_mode: TransactionMode::Unconditional,
            read_limit: 16,
        }
    }

    fn verifier_with_seed(key: DocumentKey, revision: u64) -> SampledReadVerifier {
        let sampled_keys = HashSet::from([key.clone()]);
        let seed_states = HashMap::from([(
            key,
            SampledSeed {
                value: value_bytes(8, 0, 0),
                revision: dodb_core::Revision::new(revision),
            },
        )]);
        SampledReadVerifier::new(&sampled_keys, seed_states)
    }

    #[test]
    fn parallel_engine_aliases_and_worker_default_are_stable() {
        assert_eq!(
            EngineKind::parse("parallel-blink"),
            EngineKind::ParallelBlink
        );
        assert_eq!(
            EngineKind::parse("blink-parallel"),
            EngineKind::ParallelBlink
        );
        assert_eq!(EngineKind::parse("phase4"), EngineKind::ParallelBlink);
        assert_eq!(EngineKind::ParallelBlink.as_str(), "parallel-blink");
        assert_eq!(
            EngineKind::parse("phase-j"),
            EngineKind::LogicalOverlayBlink
        );
        assert_eq!(
            EngineKind::parse("phase-k"),
            EngineKind::BackgroundOverlayBlink
        );
        assert_eq!(Args::default().blink_workers, 2);
    }

    #[test]
    fn same_seed_produces_same_transaction_sequence() {
        let mut left = WorkloadGenerator::new(workload(Distribution::Uniform, 4), 42, 3);
        let mut right = WorkloadGenerator::new(workload(Distribution::Uniform, 4), 42, 3);
        for _ in 0..32 {
            assert_eq!(left.next_transaction(), right.next_transaction());
        }
    }

    #[test]
    fn query16_uses_many_deterministic_pk_and_start_ranges() {
        let config = workload(Distribution::Uniform, 1);
        let mut first_generator = WorkloadGenerator::new(config.clone(), 0xabc, 2);
        let mut second_generator = WorkloadGenerator::new(config, 0xabc, 2);
        let mut requested_ranges = HashSet::new();
        for _ in 0..QUERY_INPUT_PLAN_REQUESTS {
            let first_request = first_generator.next_read(ReadKind::Query);
            let second_request = second_generator.next_read(ReadKind::Query);
            assert_eq!(first_request, second_request);
            requested_ranges.insert(
                query_range_from_request(&first_request, DEFAULT_KEY_SIZE, DEFAULT_READ_LIMIT)
                    .unwrap(),
            );
        }
        assert!(
            requested_ranges
                .iter()
                .map(|range| range.pk_index)
                .collect::<HashSet<_>>()
                .len()
                > 1
        );
        assert!(
            requested_ranges
                .iter()
                .map(|range| range.start_index)
                .collect::<HashSet<_>>()
                .len()
                > 1
        );
        assert!(requested_ranges.len() > 100);
        assert_eq!(query_range_count(DEFAULT_READ_LIMIT), 8 * 17);
    }

    #[test]
    fn invocation_seed_records_scenario_and_repetition_inputs() {
        assert_eq!(invocation_seed(100, 0, 0), 100);
        assert_eq!(invocation_seed(100, 1, 0), 100 + 0x9e37_79b9);
        assert_eq!(invocation_seed(100, 1, 2), 100 + 0x9e37_79b9 + 2);
        assert_eq!(invocation_seed(u64::MAX, 1, 1), 0x9e37_79b9);
    }

    #[test]
    fn mix_quota_interleaves_slots_at_the_requested_operation_ratio() {
        for (read_percent, expected_reads) in [(95, 95), (50, 50), (20, 20)] {
            let roles = (0..100)
                .map(|slot| is_read_slot(slot, read_percent))
                .collect::<Vec<_>>();
            assert_eq!(
                roles.iter().filter(|is_read| **is_read).count(),
                expected_reads
            );
            let longest_read_run = roles
                .split(|is_read| !*is_read)
                .map(<[bool]>::len)
                .max()
                .unwrap_or(0);
            let longest_write_run = roles
                .split(|is_read| *is_read)
                .map(<[bool]>::len)
                .max()
                .unwrap_or(0);
            match read_percent {
                95 => assert!(longest_read_run <= 19 && longest_write_run == 1),
                50 => assert!(longest_read_run == 1 && longest_write_run == 1),
                20 => assert!(longest_read_run == 1 && longest_write_run <= 4),
                _ => unreachable!(),
            }
        }
    }

    #[test]
    fn mixed_scenarios_keep_default_width_and_accept_explicit_widths() {
        let mut args = Args::default();
        args.suite = Suite::Mixed;
        args.writers = Some(vec![2]);
        args.readers = Some(vec![2]);
        args.mixes = Some(vec![Mix::BALANCED]);
        assert_eq!(
            scenarios(&args)
                .iter()
                .map(|scenario| scenario.width)
                .collect::<Vec<_>>(),
            vec![1]
        );

        args.widths = Some(vec![4, 8]);
        assert_eq!(
            scenarios(&args)
                .iter()
                .map(|scenario| scenario.width)
                .collect::<Vec<_>>(),
            vec![4, 8]
        );
    }

    #[test]
    fn verification_history_is_bounded_and_reports_omitted_reads() {
        let key = WorkloadGenerator::new(workload(Distribution::Uniform, 1), 0, 0).key_for_index(0);
        let mut verifier = verifier_with_seed(key.clone(), 5);
        let timestamp = Instant::now();
        for _ in 0..50_000 {
            verifier.record_read(SampledRead {
                key: key.clone(),
                value: Some(value_bytes(8, 0, 0)),
                revision: Some(dodb_core::Revision::new(5)),
                started_at: timestamp,
                finished_at: timestamp,
            });
        }

        let summary = verifier.summarize();
        assert_eq!(summary.status, VerificationStatus::Inconclusive);
        assert_eq!(summary.sampled_reads_observed, 50_000);
        assert!(summary.sampled_reads_checked < summary.sampled_reads_observed);
        assert!(summary.omitted_events > 0);
        assert!(summary.estimated_memory_bytes <= MAX_VERIFICATION_MEMORY_BYTES);
    }

    #[test]
    fn transaction_width_and_key_uniqueness_are_preserved() {
        for distribution in [
            Distribution::Uniform,
            Distribution::Sequential,
            Distribution::Hotspot,
            Distribution::SameLeafHeavy,
            Distribution::DifferentLeafHeavy,
        ] {
            let mut generator = WorkloadGenerator::new(workload(distribution, 25), 7, 1);
            for _ in 0..64 {
                let request = generator.next_transaction();
                assert_eq!(request.mutations.len(), 25);
                let keys = request
                    .mutations
                    .iter()
                    .map(TransactionMutation::key)
                    .collect::<HashSet<_>>();
                assert_eq!(keys.len(), 25);
            }
        }
    }

    #[test]
    fn locality_generators_have_distinct_pk_shapes() {
        let same = WorkloadGenerator::new(workload(Distribution::SameLeafHeavy, 4), 1, 0);
        let different = WorkloadGenerator::new(workload(Distribution::DifferentLeafHeavy, 4), 1, 0);
        let same_keys = same.seed_keys().take(32).collect::<Vec<_>>();
        let different_keys = different.seed_keys().take(32).collect::<Vec<_>>();
        assert!(same_keys.windows(2).all(|pair| pair[0].pk == pair[1].pk));
        assert!(
            different_keys
                .windows(2)
                .any(|pair| pair[0].pk != pair[1].pk)
        );
    }

    #[test]
    fn query16_response_contract_rejects_wrong_pk_boundary_order_duplicates_count_and_values() {
        let mut args = Args::default();
        args.working_set = 32;
        args.read_limit = 16;
        let range = QueryRange {
            pk_index: 3,
            start_index: 5,
            limit: args.read_limit,
        };
        let request = range.request(args.key_size);
        let mut rows = (range.start_index..range.start_index + args.read_limit)
            .map(|row_index| dodb_storage::Document {
                key: distributed_query_key(args.key_size, range.pk_index, row_index),
                value: value_bytes(
                    args.value_size,
                    distributed_query_value_index(range.pk_index, row_index, args.read_limit),
                    0,
                ),
                revision: dodb_core::Revision::new(row_index as u64 + 1),
            })
            .collect::<Vec<_>>();

        assert!(query_result_is_valid(
            &request,
            &rows,
            args.key_size,
            args.value_size,
            args.read_limit,
        ));

        rows.swap(0, 1);
        assert!(!query_result_is_valid(
            &request,
            &rows,
            args.key_size,
            args.value_size,
            args.read_limit,
        ));
        rows.swap(0, 1);

        rows[1].key = rows[0].key.clone();
        assert!(!query_result_is_valid(
            &request,
            &rows,
            args.key_size,
            args.value_size,
            args.read_limit,
        ));
        rows[1].key = distributed_query_key(args.key_size, range.pk_index, range.start_index + 1);

        rows.pop();
        assert!(!query_result_is_valid(
            &request,
            &rows,
            args.key_size,
            args.value_size,
            args.read_limit,
        ));
        rows.push(dodb_storage::Document {
            key: distributed_query_key(args.key_size, range.pk_index, range.start_index + 15),
            value: value_bytes(
                args.value_size,
                distributed_query_value_index(
                    range.pk_index,
                    range.start_index + 15,
                    args.read_limit,
                ),
                0,
            ),
            revision: dodb_core::Revision::new(16),
        });

        let wrong_pk_request = QueryRange {
            pk_index: QUERY_PK_COUNT,
            ..range
        }
        .request(args.key_size);
        assert!(!query_result_is_valid(
            &wrong_pk_request,
            &rows,
            args.key_size,
            args.value_size,
            args.read_limit,
        ));

        let wrong_boundary_request = BatchRequest::Query {
            pk: PrimaryKey::new(distributed_query_pk(args.key_size, range.pk_index)),
            exclusive_after_sk: Some(SortKey::new(component_bytes(
                0x43,
                (query_rows_per_pk(args.read_limit) - 1) as u64,
                key_component_lengths(args.key_size).1,
            ))),
            limit: args.read_limit,
        };
        assert!(!query_result_is_valid(
            &wrong_boundary_request,
            &rows,
            args.key_size,
            args.value_size,
            args.read_limit,
        ));

        rows[0].value[0] ^= 0xff;
        assert!(!query_result_is_valid(
            &request,
            &rows,
            args.key_size,
            args.value_size,
            args.read_limit,
        ));
    }

    #[test]
    fn damaged_measured_response_is_counted_and_invalidates_the_run() {
        let args = Args::default();
        let workload = WorkloadConfig {
            distribution: Distribution::Uniform,
            working_set: args.working_set,
            key_size: args.key_size,
            value_size: args.value_size,
            width: 1,
            transaction_mode: args.transaction_mode,
            read_limit: args.read_limit,
        };
        let range = QueryRange {
            pk_index: 2,
            start_index: 4,
            limit: args.read_limit,
        };
        let request = range.request(args.key_size);
        let rows = (range.start_index..range.start_index + range.limit)
            .map(|row_index| dodb_storage::Document {
                key: distributed_query_key(args.key_size, range.pk_index, row_index),
                value: value_bytes(
                    args.value_size,
                    distributed_query_value_index(range.pk_index, row_index, args.read_limit),
                    0,
                ),
                revision: dodb_core::Revision::new(row_index as u64 + 1),
            })
            .collect::<Vec<_>>();
        let mut damaged_rows = rows;
        damaged_rows[0].value[0] ^= 1;
        let mut stats = WorkerStats::new(2);
        assert!(!record_query_response(
            &request,
            &BatchResponse::Query(damaged_rows),
            &workload,
            &mut stats,
        ));
        stats.complete_client("reader", 0);
        assert_eq!(stats.query_checked_requests, 1);
        assert_eq!(stats.query_checked_rows, args.read_limit as u64);
        assert_eq!(stats.query_validation_failures, 1);
        assert_eq!(stats.errors, 1);
        assert_eq!(stats.successful_queries, 0);
        assert_eq!(stats.returned_rows, args.read_limit as u64);
        assert_eq!(stats.client_completions[0].successful_queries, 0);
        assert_eq!(
            stats.client_completions[0].returned_rows,
            args.read_limit as u64
        );
        assert!(measured_query_verification_result(&stats).is_err());

        let clean_response = BatchResponse::Query(
            (range.start_index..range.start_index + range.limit)
                .map(|row_index| dodb_storage::Document {
                    key: distributed_query_key(args.key_size, range.pk_index, row_index),
                    value: value_bytes(
                        args.value_size,
                        distributed_query_value_index(range.pk_index, row_index, args.read_limit),
                        0,
                    ),
                    revision: dodb_core::Revision::new(row_index as u64 + 1),
                })
                .collect(),
        );
        let mut clean_stats = WorkerStats::new(3);
        assert!(record_query_response(
            &request,
            &clean_response,
            &workload,
            &mut clean_stats,
        ));
        assert_eq!(
            clean_stats.returned_rows,
            clean_stats.successful_queries * args.read_limit as u64
        );
        assert!(measured_query_verification_result(&clean_stats).is_ok());
    }

    #[test]
    fn sampled_read_verification_accepts_seed_and_revisioned_updates_but_rejects_corruption() {
        let key = WorkloadGenerator::new(workload(Distribution::Uniform, 1), 0, 0).key_for_index(0);
        let interval_start = Instant::now();
        let seed_revision = dodb_core::Revision::new(5);
        let write_revision = dodb_core::Revision::new(10);
        let seed_value = value_bytes(8, 0, 0);
        let write_value = vec![0x91; 8];
        let mut verifier = verifier_with_seed(key.clone(), seed_revision.get());
        verifier.seed_states.insert(
            key.clone(),
            SampledSeed {
                value: seed_value.clone(),
                revision: seed_revision,
            },
        );
        verifier.record_mutations(
            vec![(key.clone(), write_value.clone())],
            Some(write_revision),
            interval_start + Duration::from_millis(2),
            interval_start + Duration::from_millis(3),
            false,
        );
        verifier.record_read(SampledRead {
            key: key.clone(),
            value: Some(seed_value),
            revision: Some(seed_revision),
            started_at: interval_start,
            finished_at: interval_start + Duration::from_millis(1),
        });
        verifier.record_read(SampledRead {
            key: key.clone(),
            value: Some(write_value.clone()),
            revision: Some(write_revision),
            started_at: interval_start + Duration::from_millis(1),
            finished_at: interval_start + Duration::from_millis(4),
        });
        verifier.record_read(SampledRead {
            key: key.clone(),
            value: Some(write_value.clone()),
            revision: Some(write_revision),
            started_at: interval_start + Duration::from_millis(4),
            finished_at: interval_start + Duration::from_millis(5),
        });
        let summary = verifier.summarize();
        assert_eq!(summary.status, VerificationStatus::Passed);
        assert_eq!(summary.sampled_reads_checked, 3);
        assert_eq!(summary.changed_key_reads_checked, 2);

        let mut corrupted_verifier = verifier_with_seed(key.clone(), seed_revision.get());
        corrupted_verifier.seed_states.insert(
            key.clone(),
            SampledSeed {
                value: value_bytes(8, 0, 0),
                revision: seed_revision,
            },
        );
        corrupted_verifier.record_mutations(
            vec![(key.clone(), write_value)],
            Some(write_revision),
            interval_start + Duration::from_millis(2),
            interval_start + Duration::from_millis(3),
            false,
        );
        corrupted_verifier.record_read(SampledRead {
            key,
            value: Some(vec![0x92; 8]),
            revision: Some(write_revision),
            started_at: interval_start + Duration::from_millis(4),
            finished_at: interval_start + Duration::from_millis(5),
        });
        assert_eq!(
            corrupted_verifier.summarize().status,
            VerificationStatus::Failed
        );
    }

    #[test]
    fn empty_samples_are_inconclusive_and_ambiguous_writes_are_not_permanent_candidates() {
        let key = WorkloadGenerator::new(workload(Distribution::Uniform, 1), 0, 0).key_for_index(0);
        let interval_start = Instant::now();
        let empty_verifier = verifier_with_seed(key.clone(), 5);
        assert_eq!(
            empty_verifier.summarize().status,
            VerificationStatus::Inconclusive
        );

        let unknown_value = vec![0xa1; 8];
        let mut verifier = verifier_with_seed(key.clone(), 5);
        verifier.record_mutations(
            vec![(key.clone(), unknown_value.clone())],
            None,
            interval_start + Duration::from_millis(1),
            interval_start + Duration::from_millis(2),
            true,
        );
        verifier.record_read(SampledRead {
            key: key.clone(),
            value: Some(unknown_value),
            revision: Some(dodb_core::Revision::new(6)),
            started_at: interval_start + Duration::from_millis(3),
            finished_at: interval_start + Duration::from_millis(4),
        });
        assert_eq!(
            verifier.summarize().status,
            VerificationStatus::Inconclusive
        );
        verifier.record_mutations(
            vec![(key.clone(), vec![0xb2; 8])],
            Some(dodb_core::Revision::new(7)),
            interval_start + Duration::from_millis(5),
            interval_start + Duration::from_millis(6),
            false,
        );
        verifier.record_read(SampledRead {
            key: key.clone(),
            value: Some(vec![0xa1; 8]),
            revision: Some(dodb_core::Revision::new(6)),
            started_at: interval_start + Duration::from_millis(7),
            finished_at: interval_start + Duration::from_millis(8),
        });
        assert_eq!(verifier.summarize().status, VerificationStatus::Failed);

        let mut resolved_verifier = verifier_with_seed(key.clone(), 5);
        resolved_verifier.record_mutations(
            vec![(key.clone(), vec![0xa1; 8])],
            None,
            interval_start + Duration::from_millis(1),
            interval_start + Duration::from_millis(2),
            true,
        );
        resolved_verifier.record_mutations(
            vec![(key.clone(), vec![0xb2; 8])],
            Some(dodb_core::Revision::new(7)),
            interval_start + Duration::from_millis(3),
            interval_start + Duration::from_millis(4),
            false,
        );
        resolved_verifier.record_read(SampledRead {
            key,
            value: Some(vec![0xb2; 8]),
            revision: Some(dodb_core::Revision::new(7)),
            started_at: interval_start + Duration::from_millis(5),
            finished_at: interval_start + Duration::from_millis(6),
        });
        let resolved_summary = resolved_verifier.summarize();
        assert_eq!(resolved_summary.status, VerificationStatus::Passed);
        assert_eq!(resolved_summary.sampled_reads_indeterminate, 0);
    }

    #[test]
    fn concurrent_write_response_order_does_not_define_latest_value() {
        let key = WorkloadGenerator::new(workload(Distribution::Uniform, 1), 0, 0).key_for_index(0);
        let interval_start = Instant::now();
        let value_a = vec![0xa1; 8];
        let value_b = vec![0xb2; 8];
        let mut verifier = verifier_with_seed(key.clone(), 5);
        verifier.record_mutations(
            vec![(key.clone(), value_b.clone())],
            Some(dodb_core::Revision::new(20)),
            interval_start + Duration::from_millis(2),
            interval_start + Duration::from_millis(7),
            false,
        );
        verifier.record_mutations(
            vec![(key.clone(), value_a.clone())],
            Some(dodb_core::Revision::new(10)),
            interval_start + Duration::from_millis(1),
            interval_start + Duration::from_millis(8),
            false,
        );
        verifier.record_read(SampledRead {
            key: key.clone(),
            value: Some(value_b.clone()),
            revision: Some(dodb_core::Revision::new(20)),
            started_at: interval_start + Duration::from_millis(9),
            finished_at: interval_start + Duration::from_millis(10),
        });
        assert_eq!(verifier.summarize().status, VerificationStatus::Passed);

        let mut stale_verifier = verifier_with_seed(key.clone(), 5);
        stale_verifier.record_mutations(
            vec![(key.clone(), value_b)],
            Some(dodb_core::Revision::new(20)),
            interval_start + Duration::from_millis(2),
            interval_start + Duration::from_millis(7),
            false,
        );
        stale_verifier.record_mutations(
            vec![(key.clone(), value_a.clone())],
            Some(dodb_core::Revision::new(10)),
            interval_start + Duration::from_millis(1),
            interval_start + Duration::from_millis(8),
            false,
        );
        stale_verifier.record_read(SampledRead {
            key,
            value: Some(value_a),
            revision: Some(dodb_core::Revision::new(10)),
            started_at: interval_start + Duration::from_millis(9),
            finished_at: interval_start + Duration::from_millis(10),
        });
        assert_eq!(
            stale_verifier.summarize().status,
            VerificationStatus::Failed
        );
    }

    #[test]
    fn prior_mixed_smoke_selected_no_sampled_gets() {
        let mut args = Args::default();
        args.seed = 15_049_657_927_369_490_433;
        args.working_set = 256;
        let scenario = Scenario {
            suite: Suite::Mixed,
            workload: "mixed",
            writers: 1,
            readers: 1,
            width: 1,
            distribution: Distribution::Uniform,
            read_kind: Some(ReadKind::Get),
            mix: Some(Mix::BALANCED),
            collection_delay: Duration::ZERO,
            sync_delay: Duration::ZERO,
        };
        let sampled_keys = sampled_read_keys(&args, &scenario);
        let mut sampled_indexes = sampled_keys
            .iter()
            .map(|key| {
                u64::from_be_bytes(
                    key.sk.as_bytes()[key.sk.as_bytes().len() - 8..]
                        .try_into()
                        .unwrap(),
                )
            })
            .collect::<Vec<_>>();
        sampled_indexes.sort_unstable();
        assert_eq!(sampled_indexes, vec![105, 213, 214]);

        let measured_seed = invocation_seed(args.seed, 0, 0) ^ 0xbbbb_0000;
        let read_seed = measured_seed ^ 0x2000_0000;
        let mut read_generator = WorkloadGenerator::new(
            WorkloadConfig {
                distribution: scenario.distribution,
                working_set: args.working_set,
                key_size: args.key_size,
                value_size: args.value_size,
                width: scenario.width,
                transaction_mode: args.transaction_mode,
                read_limit: args.read_limit,
            },
            read_seed,
            0,
        );
        let sampled_gets = (0..50)
            .filter(|_| {
                matches!(
                    read_generator.next_read(ReadKind::Get),
                    BatchRequest::Get { ref key } if sampled_keys.contains(key)
                )
            })
            .count();

        assert_eq!(sampled_gets, 0);
    }

    #[test]
    fn duration_parser_supports_required_units() {
        assert_eq!(parse_duration("0"), Duration::ZERO);
        assert_eq!(parse_duration("50us"), Duration::from_micros(50));
        assert_eq!(parse_duration("1ms"), Duration::from_millis(1));
        assert_eq!(parse_duration("2s"), Duration::from_secs(2));
    }

    #[test]
    fn disabled_sync_mode_aliases_parse_and_report_canonical_name() {
        assert_eq!(SyncMode::parse("disabled"), SyncMode::Disabled);
        assert_eq!(SyncMode::parse("none"), SyncMode::Disabled);
        assert_eq!(SyncMode::parse("no-sync"), SyncMode::Disabled);
        assert_eq!(SyncMode::Disabled.as_str(), "disabled");
    }

    #[test]
    fn real_benchmark_sync_calls_real_sync_once() {
        let mut closure_calls = 0;
        let result = perform_benchmark_sync(SyncMode::Real, Duration::ZERO, || {
            closure_calls += 1;
            Ok(())
        });
        assert!(result.is_ok());
        assert_eq!(closure_calls, 1);
    }

    #[test]
    fn injected_benchmark_sync_calls_real_sync_once() {
        let mut closure_calls = 0;
        let result = perform_benchmark_sync(SyncMode::Injected, Duration::ZERO, || {
            closure_calls += 1;
            Ok(())
        });
        assert!(result.is_ok());
        assert_eq!(closure_calls, 1);
    }

    #[test]
    fn disabled_benchmark_sync_does_not_call_real_sync() {
        let mut closure_calls = 0;
        let result = perform_benchmark_sync(SyncMode::Disabled, Duration::ZERO, || {
            closure_calls += 1;
            Ok(())
        });
        assert!(result.is_ok());
        assert_eq!(closure_calls, 0);
    }

    #[test]
    fn json_string_escapes_control_characters() {
        assert_eq!(json_string("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
    }
}
