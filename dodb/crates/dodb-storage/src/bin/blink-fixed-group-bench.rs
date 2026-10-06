#[path = "../../../../experiments/mixed-harness/harness/common/workload.rs"]
mod workload;

use std::env;
use std::error::Error as StdError;
use std::fmt::Write as _;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use dodb_core::{DocumentKey, RevisionState, TransactionMutation, TransactionRequest};
use dodb_storage::{
    BlinkBatchMetrics, BlinkStore, BlinkVersionedReadMetrics, DatabaseConfig, DurableFile,
    ProductionFile, StorageMetrics, WalMetrics,
};
use workload::{
    Distribution, LatencySamples, MixedValueMode, WorkloadConfig, WorkloadGenerator,
    mixed_operation_seed, mixed_trace_prefix_hash, seed_rows, writer_phase_seed,
};

#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

const DEFAULT_WORKING_SET: usize = 10_000;
const DEFAULT_VALUE_SIZE: usize = 512;
const DEFAULT_WARMUP_MS: u64 = 2_000;
const DEFAULT_DURATION_MS: u64 = 5_000;
const SAMPLE_KEY_COUNT: usize = 16;

type BenchResult<T> = std::result::Result<T, Box<dyn StdError>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Engine {
    Planned,
    Parallel,
}

impl Engine {
    fn parse(value: &str) -> BenchResult<Self> {
        match value {
            "planned-blink" => Ok(Self::Planned),
            "parallel-blink" => Ok(Self::Parallel),
            other => Err(format!("unknown engine {other:?}").into()),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned-blink",
            Self::Parallel => "parallel-blink",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SyncMode {
    Real,
    Disabled,
}

impl SyncMode {
    fn parse(value: &str) -> BenchResult<Self> {
        match value {
            "real" => Ok(Self::Real),
            "disabled" => Ok(Self::Disabled),
            other => Err(format!("unknown sync mode {other:?}").into()),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Real => "real",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Clone, Debug)]
struct Args {
    engine: Engine,
    workers: usize,
    group_size: usize,
    sync_mode: SyncMode,
    value_mode: MixedValueMode,
    working_set: usize,
    value_size: usize,
    warmup: Duration,
    duration: Duration,
    seed: u64,
    data_dir: PathBuf,
    output: PathBuf,
    repetition: u64,
}

impl Args {
    fn parse(arguments: impl IntoIterator<Item = String>) -> BenchResult<Self> {
        let mut engine = None;
        let mut workers = None;
        let mut group_size = None;
        let mut sync_mode = None;
        let mut value_mode = None;
        let mut working_set = DEFAULT_WORKING_SET;
        let mut value_size = DEFAULT_VALUE_SIZE;
        let mut warmup_ms = DEFAULT_WARMUP_MS;
        let mut duration_ms = DEFAULT_DURATION_MS;
        let mut seed = 0x9790_0000_u64;
        let mut data_dir = None;
        let mut output = None;
        let mut repetition = 0_u64;
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            if argument == "--help" || argument == "-h" {
                return Err(usage().into());
            }
            let value = arguments
                .next()
                .ok_or_else(|| format!("missing value after {argument}"))?;
            match argument.as_str() {
                "--engine" => engine = Some(Engine::parse(&value)?),
                "--workers" => workers = Some(parse_number::<usize>(&argument, &value)?),
                "--group-size" => {
                    let parsed = parse_number::<usize>(&argument, &value)?;
                    if !matches!(parsed, 4 | 16 | 64) {
                        return Err("group size must be 4, 16, or 64".into());
                    }
                    group_size = Some(parsed);
                }
                "--sync-mode" => sync_mode = Some(SyncMode::parse(&value)?),
                "--value-mode" => value_mode = Some(MixedValueMode::parse(&value)),
                "--working-set" => working_set = parse_number(&argument, &value)?,
                "--value-size" => value_size = parse_number(&argument, &value)?,
                "--warmup-ms" => warmup_ms = parse_number(&argument, &value)?,
                "--duration-ms" => duration_ms = parse_number(&argument, &value)?,
                "--seed" => seed = parse_number(&argument, &value)?,
                "--data-dir" => data_dir = Some(PathBuf::from(value)),
                "--output" => output = Some(PathBuf::from(value)),
                "--repetition" => repetition = parse_number(&argument, &value)?,
                _ => return Err(format!("unknown argument {argument:?}\n{}", usage()).into()),
            }
        }

        let engine = engine.ok_or_else(|| "--engine is required".to_owned())?;
        let workers = match (engine, workers) {
            (Engine::Planned, None | Some(0)) => 0,
            (Engine::Planned, Some(_)) => {
                return Err("planned-blink uses zero executor workers".into());
            }
            (Engine::Parallel, None) => 2,
            (Engine::Parallel, Some(1 | 2)) => workers.expect("parallel workers were supplied"),
            (Engine::Parallel, Some(_)) => {
                return Err("parallel-blink workers must be 1 or 2".into());
            }
        };
        let args = Self {
            engine,
            workers,
            group_size: group_size.ok_or_else(|| "--group-size is required".to_owned())?,
            sync_mode: sync_mode.ok_or_else(|| "--sync-mode is required".to_owned())?,
            value_mode: value_mode.ok_or_else(|| "--value-mode is required".to_owned())?,
            working_set,
            value_size,
            warmup: Duration::from_millis(warmup_ms),
            duration: Duration::from_millis(duration_ms),
            seed,
            data_dir: data_dir.ok_or_else(|| "--data-dir is required".to_owned())?,
            output: output.ok_or_else(|| "--output is required".to_owned())?,
            repetition,
        };
        if args.working_set < args.group_size || args.working_set == 0 {
            return Err("working set must be at least the group size".into());
        }
        if args.value_size == 0 {
            return Err("value size must be positive".into());
        }
        if args.duration.is_zero() {
            return Err("duration must be positive".into());
        }
        Ok(args)
    }
}

struct BenchFile {
    inner: ProductionFile,
    sync_mode: SyncMode,
}

impl BenchFile {
    fn open(path: &Path, sync_mode: SyncMode) -> dodb_core::Result<Self> {
        Ok(Self {
            inner: ProductionFile::open(path)?,
            sync_mode,
        })
    }
}

impl DurableFile for BenchFile {
    fn read_at(&mut self, offset: u64, buffer: &mut [u8]) -> dodb_core::Result<usize> {
        self.inner.read_at(offset, buffer)
    }

    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> dodb_core::Result<usize> {
        self.inner.write_at(offset, bytes)
    }

    fn len(&self) -> dodb_core::Result<u64> {
        self.inner.len()
    }

    fn set_len(&mut self, length: u64) -> dodb_core::Result<()> {
        self.inner.set_len(length)
    }

    fn sync_data(&mut self) -> dodb_core::Result<()> {
        if self.sync_mode == SyncMode::Disabled {
            return Ok(());
        }
        self.inner.sync_data()
    }

    fn sync_all(&mut self) -> dodb_core::Result<()> {
        if self.sync_mode == SyncMode::Disabled {
            return Ok(());
        }
        self.inner.sync_all()
    }

    fn try_clone_for_background(&self) -> Option<Box<dyn DurableFile + Send>> {
        self.inner.try_clone().ok().map(|inner| {
            Box::new(Self {
                inner,
                sync_mode: self.sync_mode,
            }) as Box<dyn DurableFile + Send>
        })
    }
}

struct PhaseReport {
    groups: u64,
    transactions: u64,
    elapsed: Duration,
    latencies: LatencySamples,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> BenchResult<()> {
    let args = Args::parse(env::args().skip(1))?;
    let source_commit = git_commit()?;
    let data_dir = create_fresh_data_dir(&args.data_dir)?;
    let data_path = data_dir.join("data.db");
    let wal_path = data_dir.join("data.wal");
    if data_path.exists() || wal_path.exists() {
        return Err(format!(
            "benchmark data files already exist under {}",
            data_dir.display()
        )
        .into());
    }

    let workload_config = WorkloadConfig {
        distribution: Distribution::Uniform,
        working_set: args.working_set,
        key_size: 16,
        value_size: args.value_size,
        width: 1,
    };
    let mut store = BlinkStore::<BenchFile, BenchFile>::open_with_wal(
        BenchFile::open(&data_path, args.sync_mode)?,
        BenchFile::open(&wal_path, args.sync_mode)?,
        DatabaseConfig::default(),
    )?;
    let seeded_keys = seed_existing(&mut store, &workload_config, args.group_size)?;
    match args.engine {
        Engine::Planned => store.enable_planned_execution(),
        Engine::Parallel => store.enable_parallel_execution(args.workers)?,
    }
    let mut expected_samples = initial_sample_values(&workload_config);
    let invocation_seed = args.seed.wrapping_add(args.repetition);
    let warmup_report = run_phase_with_expected(
        &mut store,
        &workload_config,
        invocation_seed,
        true,
        args.group_size,
        args.value_mode,
        args.warmup,
        &mut expected_samples,
        false,
    )?;

    let wal_before = store.wal_metrics()?.unwrap_or_default();
    let blink_before = store.batch_metrics();
    let storage_before = store.storage_metrics();
    let publication_before = store.versioned_read_metrics();
    let cpu_ticks_before = process_cpu_ticks();

    let measured_report = run_phase_with_expected(
        &mut store,
        &workload_config,
        invocation_seed,
        false,
        args.group_size,
        args.value_mode,
        args.duration,
        &mut expected_samples,
        true,
    )?;

    let wal_after = store.wal_metrics()?.unwrap_or_default();
    let blink_after = store.batch_metrics();
    let storage_after = store.storage_metrics();
    let publication_after = store.versioned_read_metrics();
    let cpu_ticks_after = process_cpu_ticks();
    let vm_hwm_kib = process_vm_hwm_kib();
    verify_samples(&mut store, &expected_samples)?;
    let final_source_commit = git_commit()?;
    if source_commit != final_source_commit {
        return Err(format!(
            "source commit changed during the run: {source_commit} -> {final_source_commit}"
        )
        .into());
    }
    let measured_phase_seed = writer_phase_seed(invocation_seed, false);
    let trace_prefix_operations = 1_000;
    let trace_prefix_hash = mixed_trace_prefix_hash(
        workload_config.clone(),
        measured_phase_seed,
        0,
        args.value_mode,
        trace_prefix_operations,
    );

    let mut fields = vec![
        ("record_type", json_string("fixed_group_write_diagnostic")),
        ("source_git_commit", json_string(&source_commit)),
        ("release", cfg!(not(debug_assertions)).to_string()),
        ("engine", json_string(args.engine.as_str())),
        ("workers", args.workers.to_string()),
        ("executor", json_string("direct_apply_transaction_group")),
        ("group_size", args.group_size.to_string()),
        ("sync_mode", json_string(args.sync_mode.as_str())),
        (
            "sync_contract",
            json_string(if args.sync_mode == SyncMode::Real {
                "successful return follows ProductionFile sync"
            } else {
                "diagnostic only; sync_data and sync_all are skipped"
            }),
        ),
        ("value_mode", json_string(args.value_mode.as_str())),
        (
            "value_generator",
            json_string(args.value_mode.generator_name()),
        ),
        ("distribution", json_string("uniform")),
        ("working_set", args.working_set.to_string()),
        ("key_size", "16".to_owned()),
        ("value_size", args.value_size.to_string()),
        ("seed", args.seed.to_string()),
        ("invocation_seed", invocation_seed.to_string()),
        ("repetition", args.repetition.to_string()),
        ("seeded_keys", seeded_keys.to_string()),
        ("warmup_ms_requested", args.warmup.as_millis().to_string()),
        (
            "warmup_ms_actual",
            format!("{:.3}", warmup_report.elapsed.as_secs_f64() * 1_000.0),
        ),
        ("warmup_groups", warmup_report.groups.to_string()),
        (
            "warmup_transactions",
            warmup_report.transactions.to_string(),
        ),
        (
            "duration_ms_requested",
            args.duration.as_millis().to_string(),
        ),
        (
            "duration_ms_actual",
            format!("{:.3}", measured_report.elapsed.as_secs_f64() * 1_000.0),
        ),
        ("measured_groups", measured_report.groups.to_string()),
        (
            "measured_transactions",
            measured_report.transactions.to_string(),
        ),
        (
            "measured_operations",
            measured_report.transactions.to_string(),
        ),
        (
            "transactions_per_second",
            format!(
                "{:.3}",
                measured_report.transactions as f64 / measured_report.elapsed.as_secs_f64()
            ),
        ),
        (
            "groups_per_second",
            format!(
                "{:.3}",
                measured_report.groups as f64 / measured_report.elapsed.as_secs_f64()
            ),
        ),
        (
            "trace_prefix_operations",
            trace_prefix_operations.to_string(),
        ),
        ("trace_prefix_hash", trace_prefix_hash.to_string()),
        (
            "group_latency_samples",
            measured_report.latencies.values.len().to_string(),
        ),
        (
            "group_latency_p50_us",
            format!("{:.3}", measured_report.latencies.percentile_us(0.50)),
        ),
        (
            "group_latency_p95_us",
            format!("{:.3}", measured_report.latencies.percentile_us(0.95)),
        ),
        (
            "group_latency_p99_us",
            format!("{:.3}", measured_report.latencies.percentile_us(0.99)),
        ),
        (
            "process_cpu_ticks_delta",
            option_json(cpu_tick_delta(cpu_ticks_before, cpu_ticks_after)),
        ),
        ("process_vm_hwm_kib", option_json(vm_hwm_kib)),
        ("data_dir", json_string(&data_dir.display().to_string())),
    ];
    fields.extend(wal_metric_fields(&wal_before, &wal_after));
    fields.extend(blink_metric_fields(&blink_before, &blink_after));
    fields.extend(storage_metric_fields(&storage_before, &storage_after));
    fields.extend(publication_metric_fields(
        &publication_before,
        &publication_after,
    ));
    let record = json_object(fields);
    append_output(&args.output, &record)?;
    println!("{record}");
    Ok(())
}

fn run_phase_with_expected(
    store: &mut BlinkStore<BenchFile, BenchFile>,
    config: &WorkloadConfig,
    invocation_seed: u64,
    warmup: bool,
    group_size: usize,
    value_mode: MixedValueMode,
    duration: Duration,
    expected_samples: &mut [Vec<u8>],
    measure_latency: bool,
) -> BenchResult<PhaseReport> {
    if duration.is_zero() {
        return Ok(PhaseReport {
            groups: 0,
            transactions: 0,
            elapsed: Duration::ZERO,
            latencies: LatencySamples::with_seed(writer_phase_seed(invocation_seed, warmup)),
        });
    }
    run_phase_inner(
        store,
        config,
        invocation_seed,
        warmup,
        group_size,
        value_mode,
        duration,
        measure_latency,
        expected_samples,
    )
}

fn run_phase_inner(
    store: &mut BlinkStore<BenchFile, BenchFile>,
    config: &WorkloadConfig,
    invocation_seed: u64,
    warmup: bool,
    group_size: usize,
    value_mode: MixedValueMode,
    duration: Duration,
    measure_latency: bool,
    expected_samples: &mut [Vec<u8>],
) -> BenchResult<PhaseReport> {
    let phase_seed = writer_phase_seed(invocation_seed, warmup);
    let start = Instant::now();
    let deadline = start + duration;
    let mut groups = 0_u64;
    let mut transactions = 0_u64;
    let mut latencies = LatencySamples::with_seed(phase_seed);
    while Instant::now() < deadline || groups == 0 {
        let requests = generate_group(
            config,
            phase_seed,
            groups.saturating_mul(group_size as u64),
            group_size,
            value_mode,
            Some(expected_samples),
        );
        let group_started = Instant::now();
        let outcomes = store.apply_transaction_group(&requests)?;
        if outcomes.len() != group_size {
            return Err(format!(
                "group returned {} outcomes for {} requests",
                outcomes.len(),
                group_size
            )
            .into());
        }
        for outcome in outcomes {
            outcome?;
        }
        let elapsed = group_started.elapsed();
        if measure_latency {
            latencies.push(elapsed);
        }
        groups = groups.saturating_add(1);
        transactions = transactions.saturating_add(group_size as u64);
    }
    Ok(PhaseReport {
        groups,
        transactions,
        elapsed: start.elapsed(),
        latencies,
    })
}

fn generate_group(
    config: &WorkloadConfig,
    phase_seed: u64,
    first_operation: u64,
    group_size: usize,
    value_mode: MixedValueMode,
    mut expected_samples: Option<&mut [Vec<u8>]>,
) -> Vec<TransactionRequest> {
    let mut requests = Vec::with_capacity(group_size);
    for offset in 0..group_size {
        let operation_index = first_operation.saturating_add(offset as u64);
        let operation_seed = mixed_operation_seed(phase_seed, operation_index);
        let mut generator = WorkloadGenerator::new_mixed(
            config.clone(),
            operation_seed,
            phase_seed,
            operation_index,
            value_mode,
        );
        let mut mutations = generator.next_transaction();
        let generated = mutations
            .pop()
            .expect("width-one workload must yield one mutation");
        if let Some(samples) = expected_samples.as_deref_mut() {
            if let Some(sample_index) = sample_index(&generated.key, samples.len()) {
                samples[sample_index] = generated.value.clone();
            }
        }
        requests.push(TransactionRequest::new(
            Vec::new(),
            vec![TransactionMutation::Put {
                key: document_key(&generated.key),
                value: generated.value,
            }],
        ));
    }
    requests
}

fn seed_existing(
    store: &mut BlinkStore<BenchFile, BenchFile>,
    config: &WorkloadConfig,
    group_size: usize,
) -> BenchResult<usize> {
    let mut rows = seed_rows(config);
    let mut seeded_keys = 0_usize;
    loop {
        let group = rows
            .by_ref()
            .take(group_size)
            .map(|row| {
                TransactionRequest::new(
                    Vec::new(),
                    vec![TransactionMutation::Put {
                        key: document_key(&row.key),
                        value: row.value,
                    }],
                )
            })
            .collect::<Vec<_>>();
        if group.is_empty() {
            break;
        }
        let outcomes = store.apply_transaction_group(&group)?;
        if outcomes.len() != group.len() {
            return Err("seed group result count mismatch".into());
        }
        for outcome in outcomes {
            outcome?;
        }
        seeded_keys = seeded_keys.saturating_add(group.len());
    }
    Ok(seeded_keys)
}

fn initial_sample_values(config: &WorkloadConfig) -> Vec<Vec<u8>> {
    seed_rows(config)
        .take(SAMPLE_KEY_COUNT)
        .map(|row| row.value)
        .collect()
}

fn verify_samples(
    store: &mut BlinkStore<BenchFile, BenchFile>,
    expected_samples: &[Vec<u8>],
) -> BenchResult<()> {
    for (sample_index, expected) in expected_samples.iter().enumerate() {
        match store.get(&key_for_index(sample_index))? {
            RevisionState::Present { value, .. } if value == *expected => {}
            actual => {
                return Err(format!(
                    "post-run sample verification failed for key {sample_index}: {actual:?}"
                )
                .into());
            }
        }
    }
    Ok(())
}

fn document_key(raw_key: &[u8]) -> DocumentKey {
    assert_eq!(raw_key.len(), 16);
    DocumentKey::new(raw_key[..8].to_vec(), raw_key[8..].to_vec())
}

fn key_for_index(index: usize) -> DocumentKey {
    let raw_key = workload::key_for_index(Distribution::Uniform, 16, index);
    document_key(&raw_key)
}

fn sample_index(raw_key: &[u8], sample_count: usize) -> Option<usize> {
    if raw_key.len() != 16 {
        return None;
    }
    let decoded = u64::from_be_bytes(raw_key[8..16].try_into().ok()?);
    usize::try_from(decoded)
        .ok()
        .filter(|index| *index < sample_count)
}

fn wal_metric_fields(before: &WalMetrics, after: &WalMetrics) -> Vec<(&'static str, String)> {
    vec![
        (
            "wal_bytes",
            delta(before.wal_bytes, after.wal_bytes).to_string(),
        ),
        (
            "wal_syncs",
            delta(before.wal_syncs, after.wal_syncs).to_string(),
        ),
        (
            "wal_committed_batches",
            delta(
                before.committed_batches as u64,
                after.committed_batches as u64,
            )
            .to_string(),
        ),
        (
            "wal_page_images",
            delta(before.page_images as u64, after.page_images as u64).to_string(),
        ),
        (
            "wal_append_nanos",
            delta(before.append_nanos, after.append_nanos).to_string(),
        ),
        (
            "wal_sync_nanos",
            delta(before.sync_nanos, after.sync_nanos).to_string(),
        ),
        (
            "wal_group_encode_nanos",
            delta(before.group_encode_nanos, after.group_encode_nanos).to_string(),
        ),
        (
            "wal_group_write_nanos",
            delta(before.group_write_nanos, after.group_write_nanos).to_string(),
        ),
        (
            "wal_physical_write_calls",
            delta(before.physical_write_calls, after.physical_write_calls).to_string(),
        ),
        (
            "wal_redo_page_image_records",
            delta(
                before.redo.page_image_records,
                after.redo.page_image_records,
            )
            .to_string(),
        ),
        (
            "wal_redo_page_delta_records",
            delta(
                before.redo.page_delta_records,
                after.redo.page_delta_records,
            )
            .to_string(),
        ),
        (
            "wal_redo_page_delta_payload_bytes",
            delta(
                before.redo.page_delta_payload_bytes,
                after.redo.page_delta_payload_bytes,
            )
            .to_string(),
        ),
        (
            "wal_redo_page_delta_spans",
            delta(before.redo.page_delta_spans, after.redo.page_delta_spans).to_string(),
        ),
        (
            "wal_redo_page_delta_changed_bytes",
            delta(
                before.redo.page_delta_changed_bytes,
                after.redo.page_delta_changed_bytes,
            )
            .to_string(),
        ),
    ]
}

fn blink_metric_fields(
    before: &BlinkBatchMetrics,
    after: &BlinkBatchMetrics,
) -> Vec<(&'static str, String)> {
    vec![
        (
            "blink_logical_groups",
            delta(before.logical_groups, after.logical_groups).to_string(),
        ),
        (
            "blink_logical_transactions",
            delta(before.logical_transactions, after.logical_transactions).to_string(),
        ),
        (
            "blink_admitted_transactions",
            delta(before.admitted_transactions, after.admitted_transactions).to_string(),
        ),
        (
            "blink_logical_admission_nanos",
            delta(
                before.logical_admission_nanos,
                after.logical_admission_nanos,
            )
            .to_string(),
        ),
        (
            "blink_planning_nanos",
            delta(before.planning_nanos, after.planning_nanos).to_string(),
        ),
        (
            "blink_physical_execution_nanos",
            delta(
                before.physical_execution_nanos,
                after.physical_execution_nanos,
            )
            .to_string(),
        ),
        (
            "blink_physical_mutation_nanos",
            delta(
                before.physical_mutation_nanos,
                after.physical_mutation_nanos,
            )
            .to_string(),
        ),
        (
            "blink_physical_page_encode_nanos",
            delta(
                before.physical_page_encode_nanos,
                after.physical_page_encode_nanos,
            )
            .to_string(),
        ),
        (
            "blink_superblock_images_emitted",
            delta(
                before.superblock_images_emitted,
                after.superblock_images_emitted,
            )
            .to_string(),
        ),
        (
            "blink_superblock_images_elided",
            delta(
                before.superblock_images_elided,
                after.superblock_images_elided,
            )
            .to_string(),
        ),
        (
            "blink_page_images",
            delta(before.page_images, after.page_images).to_string(),
        ),
        (
            "blink_wal_bytes",
            delta(before.wal_bytes, after.wal_bytes).to_string(),
        ),
        (
            "blink_catalog_construction_nanos",
            delta(
                before.catalog_construction_nanos,
                after.catalog_construction_nanos,
            )
            .to_string(),
        ),
        (
            "blink_catalog_map_clone_nanos",
            delta(
                before.catalog_map_clone_nanos,
                after.catalog_map_clone_nanos,
            )
            .to_string(),
        ),
        (
            "blink_catalog_directory_clone_nanos",
            delta(
                before.catalog_directory_clone_nanos,
                after.catalog_directory_clone_nanos,
            )
            .to_string(),
        ),
        (
            "blink_catalog_chunk_clone_nanos",
            delta(
                before.catalog_chunk_clone_nanos,
                after.catalog_chunk_clone_nanos,
            )
            .to_string(),
        ),
        (
            "blink_catalog_state_scan_nanos",
            delta(
                before.catalog_state_scan_nanos,
                after.catalog_state_scan_nanos,
            )
            .to_string(),
        ),
        (
            "blink_wal_assembly_nanos",
            delta(before.wal_assembly_nanos, after.wal_assembly_nanos).to_string(),
        ),
        (
            "blink_state_install_nanos",
            delta(before.state_install_nanos, after.state_install_nanos).to_string(),
        ),
        (
            "blink_generation_publication_nanos",
            delta(
                before.generation_publication_nanos,
                after.generation_publication_nanos,
            )
            .to_string(),
        ),
        (
            "blink_parallel_groups",
            delta(before.parallel_groups, after.parallel_groups).to_string(),
        ),
        (
            "blink_parallel_leaf_jobs",
            delta(before.parallel_leaf_jobs, after.parallel_leaf_jobs).to_string(),
        ),
        (
            "blink_parallel_transactions",
            delta(before.parallel_transactions, after.parallel_transactions).to_string(),
        ),
        (
            "blink_parallel_worker_dispatches",
            delta(
                before.parallel_worker_dispatches,
                after.parallel_worker_dispatches,
            )
            .to_string(),
        ),
        (
            "blink_parallel_worker_nanos",
            delta(before.parallel_worker_nanos, after.parallel_worker_nanos).to_string(),
        ),
        (
            "blink_parallel_join_nanos",
            delta(before.parallel_join_nanos, after.parallel_join_nanos).to_string(),
        ),
        (
            "blink_parallel_job_operations",
            delta(
                before.parallel_job_operations,
                after.parallel_job_operations,
            )
            .to_string(),
        ),
        (
            "blink_parallel_dispatch_nanos",
            delta(
                before.parallel_dispatch_nanos,
                after.parallel_dispatch_nanos,
            )
            .to_string(),
        ),
        (
            "blink_parallel_collect_nanos",
            delta(before.parallel_collect_nanos, after.parallel_collect_nanos).to_string(),
        ),
        (
            "blink_parallel_worker_slot_nanos",
            delta(
                before.parallel_worker_slot_nanos,
                after.parallel_worker_slot_nanos,
            )
            .to_string(),
        ),
        (
            "blink_parallel_coordinator_lane_nanos",
            delta(
                before.parallel_coordinator_lane_nanos,
                after.parallel_coordinator_lane_nanos,
            )
            .to_string(),
        ),
        (
            "blink_parallel_worker_base_nanos",
            delta(
                before.parallel_worker_base_nanos,
                after.parallel_worker_base_nanos,
            )
            .to_string(),
        ),
        (
            "blink_parallel_worker_mutation_nanos",
            delta(
                before.parallel_worker_mutation_nanos,
                after.parallel_worker_mutation_nanos,
            )
            .to_string(),
        ),
        (
            "blink_parallel_worker_encode_nanos",
            delta(
                before.parallel_worker_encode_nanos,
                after.parallel_worker_encode_nanos,
            )
            .to_string(),
        ),
        (
            "blink_parallel_worker_delta_nanos",
            delta(
                before.parallel_worker_delta_nanos,
                after.parallel_worker_delta_nanos,
            )
            .to_string(),
        ),
        (
            "blink_parallel_fallback_groups",
            delta(
                before.parallel_fallback_groups,
                after.parallel_fallback_groups,
            )
            .to_string(),
        ),
        (
            "blink_parallel_fallback_after_dispatch",
            delta(
                before.parallel_fallback_after_dispatch,
                after.parallel_fallback_after_dispatch,
            )
            .to_string(),
        ),
        (
            "blink_parallel_fallback_no_delta_wal",
            delta(
                before.parallel_fallback_no_delta_wal,
                after.parallel_fallback_no_delta_wal,
            )
            .to_string(),
        ),
        (
            "blink_parallel_fallback_route",
            delta(
                before.parallel_fallback_route,
                after.parallel_fallback_route,
            )
            .to_string(),
        ),
        (
            "blink_parallel_fallback_overflow",
            delta(
                before.parallel_fallback_overflow,
                after.parallel_fallback_overflow,
            )
            .to_string(),
        ),
        (
            "blink_parallel_fallback_structural",
            delta(
                before.parallel_fallback_structural,
                after.parallel_fallback_structural,
            )
            .to_string(),
        ),
        (
            "blink_parallel_skipped_single_leaf",
            delta(
                before.parallel_skipped_single_leaf,
                after.parallel_skipped_single_leaf,
            )
            .to_string(),
        ),
        (
            "blink_parallel_skipped_small_group",
            delta(
                before.parallel_skipped_small_group,
                after.parallel_skipped_small_group,
            )
            .to_string(),
        ),
    ]
}

fn storage_metric_fields(
    before: &StorageMetrics,
    after: &StorageMetrics,
) -> Vec<(&'static str, String)> {
    vec![
        (
            "storage_validation_nanos",
            delta(before.validation_nanos, after.validation_nanos).to_string(),
        ),
        (
            "storage_btree_preparation_nanos",
            delta(
                before.btree_preparation_nanos,
                after.btree_preparation_nanos,
            )
            .to_string(),
        ),
        (
            "storage_publication_nanos",
            delta(before.publication_nanos, after.publication_nanos).to_string(),
        ),
    ]
}

fn publication_metric_fields(
    before: &BlinkVersionedReadMetrics,
    after: &BlinkVersionedReadMetrics,
) -> Vec<(&'static str, String)> {
    vec![
        (
            "publication_generations",
            delta(before.published_generations, after.published_generations).to_string(),
        ),
        (
            "publication_page_version_installs",
            delta(before.page_version_installs, after.page_version_installs).to_string(),
        ),
        (
            "publication_versions_retained",
            delta(before.versions_retained, after.versions_retained).to_string(),
        ),
        (
            "publication_versions_reclaimed",
            delta(before.versions_reclaimed, after.versions_reclaimed).to_string(),
        ),
    ]
}

fn append_output(path: &Path, record: &str) -> BenchResult<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let mut output = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(output, "{record}")?;
    output.sync_all()?;
    Ok(())
}

fn create_fresh_data_dir(path: &Path) -> BenchResult<PathBuf> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(path)?;
    Ok(path.to_path_buf())
}

fn git_commit() -> BenchResult<String> {
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    if !output.status.success() {
        return Err("git rev-parse HEAD failed".into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn process_cpu_ticks() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/stat").ok()?;
    let (_, counters) = status.rsplit_once(") ")?;
    let fields = counters.split_whitespace().collect::<Vec<_>>();
    let user_ticks = fields.get(11)?.parse::<u64>().ok()?;
    let system_ticks = fields.get(12)?.parse::<u64>().ok()?;
    Some(user_ticks.saturating_add(system_ticks))
}

fn process_vm_hwm_kib() -> Option<u64> {
    let status = fs::read_to_string("/proc/self/status").ok()?;
    status.lines().find_map(|line| {
        let value = line.strip_prefix("VmHWM:")?.split_whitespace().next()?;
        value.parse::<u64>().ok()
    })
}

fn cpu_tick_delta(before: Option<u64>, after: Option<u64>) -> Option<u64> {
    Some(after?.saturating_sub(before?))
}

fn option_json(value: Option<u64>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn delta(before: u64, after: u64) -> u64 {
    after.saturating_sub(before)
}

fn parse_number<T>(argument: &str, value: &str) -> BenchResult<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value
        .parse::<T>()
        .map_err(|error| format!("invalid value for {argument}: {error}").into())
}

fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(escaped, "\\u{:04x}", character as u32);
            }
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

fn json_object(fields: Vec<(&str, String)>) -> String {
    let mut record = String::from("{");
    for (field_index, (name, value)) in fields.into_iter().enumerate() {
        if field_index > 0 {
            record.push(',');
        }
        let _ = write!(record, "{}:{value}", json_string(name));
    }
    record.push('}');
    record
}

fn usage() -> &'static str {
    "Usage: blink-fixed-group-bench --engine planned-blink|parallel-blink --group-size 4|16|64 --sync-mode real|disabled --value-mode changing|constant --data-dir PATH --output PATH [--workers 1|2] [--working-set 10000] [--value-size 512] [--warmup-ms 2000] [--duration-ms 5000] [--seed N] [--repetition N]"
}
