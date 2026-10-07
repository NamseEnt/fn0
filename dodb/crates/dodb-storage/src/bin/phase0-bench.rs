//! Sustained Phase 0 baseline benchmark for the current main B+Tree.
//!
//! This binary intentionally lives beside (rather than inside) the storage
//! implementation. It uses the public AsyncShard/BTreeStore boundary and
//! existing cumulative metrics, so the production hot path does not gain
//! benchmark-only timestamps or counters.

use std::collections::HashSet;
use std::env;
use std::fmt::Write as _;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dodb_core::{
    DocumentKey, Error, PrimaryKey, Result, RevisionState, TransactionCondition,
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
const READER_WORKER_SEED_MASK: u64 = 0x1000_0000;

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
        for readers in &mixed_readers {
            for writers in &mixed_writers {
                for distribution in &mixed_distributions {
                    for mix in &mixes {
                        let selected_widths = if args.mixed_clients {
                            widths.as_slice()
                        } else {
                            &[1]
                        };
                        for width in selected_widths {
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
        let index = self.next_index(0);
        self.operation = self.operation.wrapping_add(1);
        match kind {
            ReadKind::Get => BatchRequest::Get {
                key: self.key_for_index(index),
            },
            ReadKind::Query => BatchRequest::Query {
                pk: PrimaryKey::new(query_pk(self.config.key_size)),
                exclusive_after_sk: None,
                limit: self.config.read_limit,
            },
            ReadKind::Scan => BatchRequest::Scan {
                exclusive_after_key: None,
                limit: self.config.read_limit,
            },
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

fn query_pk(key_size: usize) -> Vec<u8> {
    let (pk_len, _) = key_component_lengths(key_size);
    component_bytes(0x33, 0, pk_len)
}

fn query_key(key_size: usize, index: usize) -> DocumentKey {
    let (_, sk_len) = key_component_lengths(key_size);
    DocumentKey::new(
        query_pk(key_size),
        component_bytes(0x43, index as u64, sk_len),
    )
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
    mutation_ops: u64,
    conflicts: u64,
    overloads: u64,
    errors: u64,
    e2e_latency: LatencySamples,
    write_latency: LatencySamples,
    read_latency: LatencySamples,
    window_timeline: Vec<(u64, u64)>,
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
            mutation_ops: 0,
            conflicts: 0,
            overloads: 0,
            errors: 0,
            e2e_latency: LatencySamples::with_seed(seed),
            write_latency: LatencySamples::with_seed(seed ^ 0x1111),
            read_latency: LatencySamples::with_seed(seed ^ 0x2222),
            window_timeline: Vec::new(),
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
    }

    fn attempted_operations(&self) -> u64 {
        self.attempted_transactions
            + self.attempted_gets
            + self.attempted_queries
            + self.attempted_scans
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
            let read_slot = (slot % 100) < u64::from(self.read_percent);
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
}

async fn writer_loop(
    adapter: Arc<dyn EngineAdapter>,
    workload: WorkloadConfig,
    seed: u64,
    worker_id: usize,
    deadline: Instant,
    quota: Option<MixQuota>,
    warmup: bool,
    timeline_start: Option<Instant>,
) -> WorkerStats {
    let mut generator = WorkloadGenerator::new(workload, seed, worker_id);
    let mut stats = WorkerStats::new(seed ^ worker_id as u64);
    while Instant::now() < deadline {
        if let Some(quota) = &quota
            && !quota.claim(Role::Writer, deadline).await
        {
            break;
        }
        let request = generator.next_transaction();
        let width = request.mutations.len() as u64;
        let started = Instant::now();
        let result = adapter.execute_transaction(request).await;
        let elapsed = started.elapsed();
        if !warmup {
            stats.attempted_transactions += 1;
            stats.e2e_latency.push(elapsed);
            stats.write_latency.push(elapsed);
            match result {
                Ok(_) => {
                    stats.successful_transactions += 1;
                    stats.mutation_ops += width;
                    if let Some(timeline_start) = timeline_start {
                        stats.window_timeline.push((
                            (started + elapsed - timeline_start).as_nanos() as u64,
                            elapsed.as_nanos() as u64,
                        ));
                    }
                }
                Err(Error::Conflict(_)) => stats.conflicts += 1,
                Err(Error::Overloaded(_)) => stats.overloads += 1,
                Err(_) => stats.errors += 1,
            }
        }
    }
    stats
}

async fn reader_loop(
    adapter: Arc<dyn EngineAdapter>,
    workload: WorkloadConfig,
    read_kind: ReadKind,
    seed: u64,
    worker_id: usize,
    deadline: Instant,
    quota: Option<MixQuota>,
    warmup: bool,
) -> WorkerStats {
    let mut generator = WorkloadGenerator::new(workload, seed, worker_id);
    let mut stats = WorkerStats::new(seed ^ worker_id as u64 ^ 0xfeed);
    while Instant::now() < deadline {
        if let Some(quota) = &quota
            && !quota.claim(Role::Reader, deadline).await
        {
            break;
        }
        let started = Instant::now();
        let request = generator.next_read(read_kind);
        let expected_get_byte = match &request {
            BatchRequest::Get { key } => key.sk.as_bytes().last().copied(),
            _ => None,
        };
        let result = adapter.execute(request).await;
        let elapsed = started.elapsed();
        if warmup {
            continue;
        }
        stats.e2e_latency.push(elapsed);
        stats.read_latency.push(elapsed);
        match read_kind {
            ReadKind::Get => stats.attempted_gets += 1,
            ReadKind::Query => stats.attempted_queries += 1,
            ReadKind::Scan => stats.attempted_scans += 1,
        }
        match (read_kind, result) {
            (ReadKind::Get, Ok(BatchResponse::Get(RevisionState::Present { value, .. })))
                if expected_get_byte.is_some_and(|expected_byte| {
                    value_matches_byte(&value, generator.config.value_size, expected_byte)
                }) =>
            {
                stats.successful_gets += 1;
            }
            (ReadKind::Query, Ok(BatchResponse::Query(rows)))
                if query_result_matches(
                    &rows,
                    generator.config.key_size,
                    generator.config.value_size,
                    generator.config.read_limit,
                ) =>
            {
                stats.successful_queries += 1;
                stats.returned_rows += rows.len() as u64;
            }
            (ReadKind::Scan, Ok(BatchResponse::Scan(rows))) if !rows.is_empty() => {
                stats.successful_scans += 1;
                stats.returned_rows += rows.len() as u64;
            }
            (_, Err(Error::Overloaded(_))) => stats.overloads += 1,
            _ => stats.errors += 1,
        }
    }
    stats
}

fn value_matches_byte(value: &[u8], expected_length: usize, expected_byte: u8) -> bool {
    value.len() == expected_length && value.iter().all(|byte| *byte == expected_byte)
}

fn query_result_matches(
    rows: &[dodb_storage::btree::Document],
    key_size: usize,
    value_size: usize,
    read_limit: usize,
) -> bool {
    rows.len() == read_limit.min(256)
        && rows.iter().enumerate().all(|(row_index, row)| {
            row.key == query_key(key_size, row_index)
                && value_matches_byte(&row.value, value_size, row_index as u8)
        })
}

async fn mixed_client_loop(
    adapter: Arc<dyn EngineAdapter>,
    workload: WorkloadConfig,
    seed: u64,
    worker_id: usize,
    read_percent: u8,
    value_mode: MixedValueMode,
    next_operation: Arc<AtomicU64>,
    deadline: Instant,
    warmup: bool,
    timeline_start: Option<Instant>,
) -> WorkerStats {
    let mut stats = WorkerStats::new(seed ^ worker_id as u64 ^ 0xfeed);
    let phase_seed = seed ^ 0x1000_0000;
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
            let result = adapter.execute(request).await;
            let elapsed = started.elapsed();
            if warmup {
                continue;
            }
            stats.attempted_gets += 1;
            stats.read_latency.push(elapsed);
            match result {
                Ok(BatchResponse::Get(_)) => stats.successful_gets += 1,
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
            let result = adapter.execute_transaction(request).await;
            let elapsed = started.elapsed();
            if warmup {
                continue;
            }
            stats.attempted_transactions += 1;
            stats.write_latency.push(elapsed);
            match result {
                Ok(_) => {
                    stats.successful_transactions += 1;
                    stats.mutation_ops += width;
                }
                Err(Error::Conflict(_)) => stats.conflicts += 1,
                Err(Error::Overloaded(_)) => stats.overloads += 1,
                Err(_) => stats.errors += 1,
            }
        }
    }
    stats
}

async fn run_interval(
    adapter: Arc<dyn EngineAdapter>,
    args: &Args,
    scenario: &Scenario,
    seed: u64,
    duration: Duration,
    warmup: bool,
) -> WorkerStats {
    let interval_start = Instant::now();
    let deadline = interval_start + duration;
    let timeline_start = (!warmup && args.window_seconds.is_some()).then_some(interval_start);
    let workload = WorkloadConfig {
        distribution: scenario.distribution,
        working_set: args.working_set,
        key_size: args.key_size,
        value_size: args.value_size,
        width: scenario.width,
        transaction_mode: args.transaction_mode,
        read_limit: args.read_limit,
    };
    let mut tasks = Vec::with_capacity(scenario.writers + scenario.readers);
    if args.mixed_clients
        && let Some(mix) = scenario.mix
    {
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
                deadline,
                warmup,
                timeline_start,
            )));
        }
    } else {
        let quota = scenario.mix.map(|mix| MixQuota::new(mix.read_percent));
        for worker_id in 0..scenario.writers {
            tasks.push(tokio::spawn(writer_loop(
                Arc::clone(&adapter),
                workload.clone(),
                seed ^ 0x1000_0000,
                worker_id,
                deadline,
                quota.clone(),
                warmup,
                timeline_start,
            )));
        }
        for worker_id in 0..scenario.readers {
            tasks.push(tokio::spawn(reader_loop(
                Arc::clone(&adapter),
                workload.clone(),
                scenario.read_kind.unwrap_or(ReadKind::Get),
                seed ^ READER_WORKER_SEED_MASK,
                worker_id,
                deadline,
                quota.clone(),
                warmup,
            )));
        }
    }
    let mut stats = WorkerStats::new(seed ^ 0xabcd);
    for task in tasks {
        stats.merge(task.await.expect("benchmark worker task should not panic"));
    }
    stats
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

    if scenario.read_kind == Some(ReadKind::Query) {
        let query_rows = args.working_set.min(256);
        let mut query_mutations = Vec::new();
        for index in 0..query_rows {
            query_mutations.push(TransactionMutation::Put {
                key: query_key(args.key_size, index),
                value: value_bytes(args.value_size, index as u64, 0),
            });
            if query_mutations.len() >= 25 {
                requests.push(TransactionRequest::new(
                    Vec::new(),
                    std::mem::take(&mut query_mutations),
                ));
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
    repetition: usize,
    seed: u64,
    seeded_rows: usize,
    measured: &WorkerStats,
    delta: &MetricDelta,
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
    json.string("mixed_value_mode", args.mixed_value_mode.as_str());
    json.string(
        "mixed_value_generator",
        args.mixed_value_mode.generator_name(),
    );
    if args.mixed_clients
        && let Some(mix) = scenario.mix
    {
        json.u64("requested_read_percent", u64::from(mix.read_percent));
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
    json.u64("duration_ms", wall.as_millis() as u64);
    json.u64("requested_duration_ms", args.duration.as_millis() as u64);
    json.u64("warmup_ms", args.warmup.as_millis() as u64);
    json.usize("repetition", repetition);
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
    repetition: usize,
    seed: u64,
    output: &mut std::fs::File,
) -> Result<()> {
    let (adapter, data_path, seeded_rows) = open_adapter(args, scenario, repetition, seed).await?;
    let mut samples = Vec::new();
    if args.window_seconds.is_some() {
        samples.push(ResourceSample::capture(&*adapter, "seeded".to_string()));
    }
    let warmup_stats = run_interval(
        Arc::clone(&adapter),
        args,
        scenario,
        seed ^ 0xaaaa_0000,
        args.warmup,
        true,
    )
    .await;
    let _ = warmup_stats;
    #[cfg(feature = "phase-i-instrumentation")]
    let _ = dodb_storage::blink::take_phase_i_group_locality_samples();
    if args.window_seconds.is_some() {
        samples.push(ResourceSample::capture(
            &*adapter,
            "measurement_start".to_string(),
        ));
    }
    adapter.reset_checkpoint_metrics();
    let before = adapter.snapshot();
    let churn_before = dodb_storage::churn::snapshot();
    let (leaf_sample_start, _) = dodb_storage::churn::leaf_samples_since(usize::MAX);
    let cpu_start = ProcessCpuSample::capture();
    let started = Instant::now();
    let sampler = args.window_seconds.map(|window_seconds| {
        ResourceSampler::start(
            Arc::clone(&adapter),
            started,
            Duration::from_secs(window_seconds),
        )
    });
    let measured = run_interval(
        Arc::clone(&adapter),
        args,
        scenario,
        seed ^ 0xbbbb_0000,
        args.duration,
        false,
    )
    .await;
    let wall = started.elapsed();
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
    print_run_summary(scenario, repetition, &measured, wall, &delta);
    let line = build_record(
        machine,
        args,
        scenario,
        repetition,
        seed,
        seeded_rows,
        &measured,
        &delta,
        wall,
        &cpu_start,
        &cpu_end,
        started,
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
    Ok(())
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
                let seed = args
                    .seed
                    .wrapping_add((scenario_index as u64).wrapping_mul(0x9e37_79b9))
                    .wrapping_add(repetition as u64);
                run_repetition(&args, scenario, &machine, repetition, seed, &mut output).await?;
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
    fn read_validation_checks_point_and_query_value_contents() {
        let mut corrupted_value = vec![7; 512];
        assert!(value_matches_byte(&corrupted_value, 512, 7));
        corrupted_value[255] ^= 1;
        assert!(!value_matches_byte(&corrupted_value, 512, 7));

        let rows: Vec<dodb_storage::btree::Document> = (0..16)
            .map(|row_index| dodb_storage::btree::Document {
                key: query_key(16, row_index),
                value: vec![row_index as u8; 512],
                revision: dodb_core::Revision::ZERO,
            })
            .collect();
        assert!(query_result_matches(&rows, 16, 512, 16));
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
