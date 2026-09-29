//! Parquet part write and row-group scan.
//!
//! The scan sweep varies the number of **label columns** while the query needs
//! exactly one of them. `ProjectionMask` appears nowhere in the tree
//! (`docs/VISION.md` III), so every label column and the
//! `structured_metadata` blob is decoded regardless — the sweep is shaped so
//! that shows up as a slope, and lands flat once projection pushdown exists.
//!
//! The write sweep runs the same rows as plain text and as JSON. The delta is
//! the parser work in `encode_blooms`, which runs over every line twice
//! (`src/part/format.rs:335-341` vs `:360-366`).

#[path = "corpus/mod.rs"]
#[allow(dead_code)]
mod corpus;

use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use signy::logql::LineFilter;
use signy::part::{ExactFieldPredicate, ExactFieldPruning, Part, PartReader, Row, flush_rows};

use corpus::{CorpusSpec, Shape, scratch::ScratchDir};

/// Criterion warms up for three seconds per bench function by default,
/// which across this suite is minutes of warm-up alone. These are a
/// regression gate, not a distribution.
const WARM_UP: Duration = Duration::from_millis(500);
#[global_allocator]
static ALLOCATOR: corpus::alloc::CountingAllocator = corpus::alloc::CountingAllocator;

/// `Config::default().row_group_size`.
const ROW_GROUP_SIZE: usize = 8192;
const SCAN_ROWS: usize = 40_000;
/// The write path is the expensive one — a full flush is parquet, zstd, both
/// parser passes and the sidecars — so it runs on fewer rows than the scan.
const WRITE_ROWS: usize = 10_000;
const LABEL_COLUMN_SWEEP: [usize; 3] = [2, 6, 10];

struct WrittenPart {
    _dir: ScratchDir,
    reader: PartReader,
}

fn write_part(corpus: &corpus::Corpus, tag: &str) -> WrittenPart {
    let dir = ScratchDir::new(tag);
    let parts = flush_rows(corpus.rows(), dir.path(), ROW_GROUP_SIZE)
        .expect("bench corpus flushes to a part");
    let part: Part = parts
        .into_iter()
        .next()
        .expect("bench corpus produces one part");
    let reader = PartReader::open(part).expect("bench part opens");
    WrittenPart { _dir: dir, reader }
}

fn bench_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("part/write");
    group
        .sample_size(10)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(2));
    for (name, shape) in [("plain", Shape::Plain), ("json", Shape::Json)] {
        let corpus = corpus::generate(
            &CorpusSpec::default()
                .rows(WRITE_ROWS)
                .streams(64)
                .labels_per_stream(6)
                .only(shape)
                .out_of_order(true),
        );
        let rows = corpus.rows();
        group.throughput(Throughput::Bytes(corpus.line_bytes()));
        group.bench_with_input(BenchmarkId::from_parameter(name), &rows, |b, rows| {
            b.iter_batched(
                || (ScratchDir::new("part-write"), rows.clone()),
                |(dir, rows)| flush_rows(rows, dir.path(), ROW_GROUP_SIZE).expect("flush succeeds"),
                BatchSize::PerIteration,
            );
        });
    }
    group.finish();
}

/// The projection sweep: one attribute predicate, N attribute columns, all of
/// them decoded.
fn bench_scan_label_columns(c: &mut Criterion) {
    let mut group = c.benchmark_group("part/scan_label_columns");
    group
        .sample_size(10)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(2));
    for label_columns in LABEL_COLUMN_SWEEP {
        let corpus = corpus::generate(
            &CorpusSpec::default()
                .rows(SCAN_ROWS)
                .streams(128)
                .labels_per_stream(label_columns),
        );
        let written = write_part(&corpus, "part-scan");
        let tenant = corpus.tenant().clone();
        let predicate = ExactFieldPredicate::new("app", corpus.label_value("app"));
        let start = corpus.min_ts_ns();
        let end = corpus.max_ts_ns();
        group.throughput(Throughput::Elements(corpus.entry_count() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(label_columns),
            &label_columns,
            |b, _| {
                b.iter(|| {
                    written
                        .reader
                        .query_with_exact_field_pruning(
                            &tenant,
                            signy::part::ExactFieldPruning::new(
                                &[],
                                std::slice::from_ref(&predicate),
                            ),
                            signy::part::QueryTimeRange::closed(start, end),
                            usize::MAX,
                            true,
                        )
                        .expect("scan succeeds")
                });
            },
        );
    }
    group.finish();
}

/// One part shared by N tenants, queried as one of them.
///
/// Row groups are tenant-aligned (`format.rs:221-238`) and a tenant occupies a
/// contiguous run, so a query should cost what its own run costs and not what
/// the part holds. That is the property `docs/VISION.md` III names as already
/// right and not to be regressed, and nothing measured it.
fn bench_scan_tenants(c: &mut Criterion) {
    let mut group = c.benchmark_group("part/scan_tenants");
    group
        .sample_size(10)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(2));
    for tenants in [1usize, 16, 128] {
        let corpus = corpus::generate(
            &CorpusSpec::default()
                .rows(SCAN_ROWS)
                .streams(tenants * 8)
                .tenants(tenants)
                .labels_per_stream(6),
        );
        let written = write_part(&corpus, "part-tenants");
        let tenant = corpus.tenant().clone();
        let start = corpus.min_ts_ns();
        let end = corpus.max_ts_ns();
        group.bench_with_input(BenchmarkId::from_parameter(tenants), &tenants, |b, _| {
            b.iter(|| {
                written
                    .reader
                    .query(
                        &tenant,
                        &[],
                        signy::part::QueryTimeRange::closed(start, end),
                        100,
                        false,
                    )
                    .expect("scan succeeds")
            });
        });
    }
    group.finish();
}

fn bench_scan_filters(c: &mut Criterion) {
    let corpus = corpus::generate(
        &CorpusSpec::default()
            .rows(SCAN_ROWS)
            .streams(128)
            .labels_per_stream(6)
            .only(Shape::Json),
    );
    let written = write_part(&corpus, "part-filters");
    let tenant = corpus.tenant().clone();
    let start = corpus.min_ts_ns();
    let end = corpus.max_ts_ns();

    let mut group = c.benchmark_group("part/scan_filters");
    group
        .sample_size(10)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(2))
        .throughput(Throughput::Elements(corpus.entry_count() as u64));

    group.bench_function("no_filter", |b| {
        b.iter(|| {
            written
                .reader
                .query(
                    &tenant,
                    &[],
                    signy::part::QueryTimeRange::closed(start, end),
                    100,
                    false,
                )
                .expect("scan succeeds")
        });
    });

    // Present in the corpus, so the bloom admits every row group and the row
    // loop runs. `reader.rs:727` allocates the line before this filter rejects
    // it, which is what this measures.
    let hit = vec![LineFilter::Contains("duration_ms".to_string())];
    group.bench_function("line_filter_hit", |b| {
        b.iter(|| {
            written
                .reader
                .query(
                    &tenant,
                    &hit,
                    signy::part::QueryTimeRange::closed(start, end),
                    100,
                    false,
                )
                .expect("scan succeeds")
        });
    });

    // Absent, so trigram pruning rejects every row group before any Parquet
    // page is touched. The gap between this and `line_filter_hit` is what the
    // sidecar buys.
    let miss = vec![LineFilter::Contains("zzqqxx-never-written".to_string())];
    group.bench_function("line_filter_pruned", |b| {
        b.iter(|| {
            written
                .reader
                .query(
                    &tenant,
                    &miss,
                    signy::part::QueryTimeRange::closed(start, end),
                    100,
                    false,
                )
                .expect("scan succeeds")
        });
    });

    // The claim in `docs/VISION.md`: `| json | field="value"` prunes at
    // row-group granularity here where Loki must scan. Both sides are
    // measured — a value the corpus contains, and one it does not.
    let present = corpus.entries()[0]
        .structured_metadata
        .first()
        .map(|(name, value)| {
            ExactFieldPredicate::new_with_extraction(name.clone(), value.clone(), true)
        })
        .expect("the bench corpus carries structured metadata");
    let absent = ExactFieldPredicate::new_with_extraction(
        "trace_id".to_string(),
        "ffffffffffffffffffffffffffffffff".to_string(),
        true,
    );
    for (name, predicate) in [
        ("exact_field_hit", present.clone()),
        ("exact_field_pruned", absent),
    ] {
        let predicates = [predicate];
        group.bench_function(name, |b| {
            b.iter(|| {
                written
                    .reader
                    .query_with_exact_field_pruning(
                        &tenant,
                        ExactFieldPruning::new(&[], &predicates),
                        signy::part::QueryTimeRange::closed(start, end),
                        100,
                        false,
                    )
                    .expect("scan succeeds")
            });
        });
    }

    // The comparison bed's warm pass: the same rare query repeated. The
    // first issue fills the cache under its narrow-pass selection; the
    // repeats replay the decode (criterion's warmup is the fill traffic).
    signy::part::configure_row_group_cache(Some(1 << 30));
    let cached = write_part(&corpus, "part-filters-cached");
    let predicates = [present];
    group.bench_function("exact_field_hit_cached", |b| {
        b.iter(|| {
            cached
                .reader
                .query_with_exact_field_pruning(
                    &tenant,
                    ExactFieldPruning::new(&[], &predicates),
                    signy::part::QueryTimeRange::closed(start, end),
                    100,
                    false,
                )
                .expect("scan succeeds")
        });
    });
    signy::part::configure_row_group_cache(None);
    group.finish();
}

/// Read-path allocation, the mirror of the write-path table in `rows.rs`.
///
/// The reader used to materialize a fresh `Labels` per row and the registry and
/// executor each cloned it again; it now interns one per distinct stream for the
/// duration of the scan. This measures the first of those three hops, per row
/// returned, so `peak live` is what has to stay flat across the label sweep.
fn report_allocations() {
    corpus::alloc::header("part scan (row-group read, one part)");
    for label_columns in LABEL_COLUMN_SWEEP {
        let corpus = corpus::generate(
            &CorpusSpec::default()
                .rows(SCAN_ROWS)
                .streams(128)
                .labels_per_stream(label_columns),
        );
        let written = write_part(&corpus, "part-alloc");
        let tenant = corpus.tenant().clone();
        let start = corpus.min_ts_ns();
        let end = corpus.max_ts_ns();
        let (result, stats) = corpus::alloc::measure(|| {
            written
                .reader
                .query(
                    &tenant,
                    &[],
                    signy::part::QueryTimeRange::closed(start, end),
                    usize::MAX,
                    true,
                )
                .expect("scan succeeds")
        });
        let returned: usize = result.iter().map(|stream| stream.entries.len()).sum();
        corpus::alloc::row(
            &format!("label_columns={label_columns}"),
            returned,
            &stats,
            "one interned label set per stream, shared by its rows",
        );
    }

    corpus::alloc::header("part write (materialize + sort + parquet + sidecars)");
    for label_columns in LABEL_COLUMN_SWEEP {
        let corpus = corpus::generate(
            &CorpusSpec::default()
                .rows(WRITE_ROWS)
                .streams(64)
                .labels_per_stream(label_columns)
                .only(Shape::Json),
        );
        let rows: Vec<Row> = corpus.rows();
        let entries = rows.len();
        let dir = ScratchDir::new("part-write-alloc");
        let (_parts, stats) = corpus::alloc::measure(|| {
            flush_rows(rows, dir.path(), ROW_GROUP_SIZE).expect("flush succeeds")
        });
        corpus::alloc::row(
            &format!("label_columns={label_columns}"),
            entries,
            &stats,
            "parquet + blooms + stream index",
        );
    }
    corpus::alloc::footer();
    report_on_disk_size();
}

/// Bytes on disk per byte ingested, per line shape.
///
/// This is the corpus calibrating itself. `docs/LOAD_RESULTS.md` §2 records
/// that realistic lines compress 5.9x where the old harness's `"x".repeat(n)`
/// compressed 31.5x; a ratio anywhere near that second number here means the
/// generator has drifted back towards no entropy and every number below it is
/// worth nothing.
fn report_on_disk_size() {
    eprintln!("=== corpus entropy check: on-disk size per shape ===");
    eprintln!(
        "{:<12} {:>8} {:>14} {:>12} {:>12} {:>10}",
        "shape", "rows", "line bytes", "parquet", "index.bin", "ratio"
    );
    for (name, shape) in [
        ("plain", Shape::Plain),
        ("json", Shape::Json),
        ("logfmt", Shape::Logfmt),
    ] {
        let corpus = corpus::generate(
            &CorpusSpec::default()
                .rows(WRITE_ROWS)
                .streams(64)
                .labels_per_stream(6)
                .only(shape),
        );
        let dir = ScratchDir::new("part-size");
        let parts = flush_rows(corpus.rows(), dir.path(), ROW_GROUP_SIZE).expect("flush succeeds");
        let part = parts.first().expect("one part");
        let file_len =
            |path: std::path::PathBuf| std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);
        let parquet_bytes = file_len(part.data_path());
        let index_bytes = file_len(part.index_path());
        let line_bytes = corpus.line_bytes();
        eprintln!(
            "{:<12} {:>8} {:>14} {:>12} {:>12} {:>9.2}x",
            name,
            corpus.entry_count(),
            line_bytes,
            parquet_bytes,
            index_bytes,
            line_bytes as f64 / parquet_bytes.max(1) as f64,
        );
    }
    eprintln!();
}

fn benches(c: &mut Criterion) {
    report_allocations();
    bench_write(c);
    bench_scan_label_columns(c);
    bench_scan_tenants(c);
    bench_scan_filters(c);
}

criterion_group!(part, benches);
criterion_main!(part);
