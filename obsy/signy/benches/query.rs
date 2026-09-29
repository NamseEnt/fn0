//! The executor: a limit over a window far larger than the limit.
//!
//! This is the shape `docs/COMPARISON.md` measured signy losing on — 1.49x
//! slower than Loki cold on `{...} | json | field="v"` — and it had no bench,
//! because it is a property of `log_scan` rather than of any one part scan.
//! `benches/part.rs` measures a single part with no limit; nothing measured what
//! a `limit=100` costs across eight parts holding two hundred thousand rows.
//!
//! Three shapes, one limit, one window:
//!
//! * `label_only` — the limit reaches the storage scan. The shape signy
//!   already won (0.36x).
//! * `line_filter` — a `|=`, which the trigram bloom can prune and which still
//!   runs before any limit can apply.
//! * `json_field` — a parser stage and a field equality. `normal_scan_limit`
//!   was `usize::MAX` for exactly this, so the whole window was materialized to
//!   return a hundred rows.
//!
//! `lines` in the allocation table is `scanned_rows`, which is what
//! `data.stats.totalLinesProcessed` reports. On the comparison bed that number
//! read 16 384 against Loki's 1 251 for the same answer, so it is the figure
//! this bench exists to move and it is printed next to the allocations.

#[path = "corpus/mod.rs"]
#[allow(dead_code)]
mod corpus;

use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use signy::log_scan::LogScan;
use signy::logql::LogQuery;
use signy::memtable::MemTable;
use signy::part::{QueryTimeRange, flush_rows};
use signy::part_registry::PartRegistry;
use signy::tenant::TenantId;

use corpus::{CorpusSpec, Shape, scratch::ScratchDir};

const WARM_UP: Duration = Duration::from_millis(500);
#[global_allocator]
static ALLOCATOR: corpus::alloc::CountingAllocator = corpus::alloc::CountingAllocator;

/// `Config::default().row_group_size`.
const ROW_GROUP_SIZE: usize = 8192;
/// Eight parts, which is the order the comparison bed reaches within a minute.
const PARTS: usize = 8;
const ROWS_PER_PART: usize = 25_000;
/// Grafana's default, and the limit `docs/COMPARISON.md` queried with.
const LIMIT: usize = 100;

/// Parts on disk plus the memtable a live server would also be holding.
///
/// The memtable is not empty: a query answers from both buffers, and a scan
/// that stops early has to stop early in both or the early stop is worth half
/// of what it measures.
struct Bed {
    _dir: ScratchDir,
    parts: PartRegistry,
    memtable: MemTable,
    tenant: TenantId,
    range: QueryTimeRange,
    app: String,
    rows: usize,
}

fn bed() -> Bed {
    let dir = ScratchDir::new("query-bed");
    let parts = PartRegistry::new();
    let memtable = MemTable::new();
    let mut tenant = None;
    let mut app = None;
    let mut min_ts = i64::MAX;
    let mut max_ts = i64::MIN;
    let mut rows = 0usize;
    let base = CorpusSpec::default();
    for index in 0..PARTS {
        let corpus = corpus::generate(
            &CorpusSpec::default()
                .rows(ROWS_PER_PART)
                .streams(128)
                .labels_per_stream(6)
                .only(Shape::Json)
                .out_of_order(true)
                .seed(base.seed + index as u64)
                // Consecutive, non-overlapping windows, which is what a flush
                // interval produces. A part whose range the frontier can reject
                // is a part the scan never opens, and parts that all covered the
                // same window would measure the opposite case as if it were the
                // common one.
                .start_ts_ns(base.start_ts_ns + (index * ROWS_PER_PART) as i64 * base.step_ns),
        );
        min_ts = min_ts.min(corpus.min_ts_ns());
        max_ts = max_ts.max(corpus.max_ts_ns());
        rows += corpus.entry_count();
        tenant.get_or_insert_with(|| corpus.tenant().clone());
        app.get_or_insert_with(|| corpus.label_value("app"));
        let written = flush_rows(corpus.rows(), dir.path(), ROW_GROUP_SIZE)
            .expect("bench corpus flushes to a part");
        parts.register(written).expect("bench part registers");
    }
    // One flush interval's worth still buffered, at the newest end of the
    // window, so a backward query reads the memtable before it reads a part.
    let buffered = corpus::generate(
        &CorpusSpec::default()
            .rows(2_000)
            .streams(128)
            .labels_per_stream(6)
            .only(Shape::Json)
            .seed(base.seed + 1_000)
            .start_ts_ns(max_ts + base.step_ns),
    );
    for (tenant, entries) in buffered.snapshot() {
        memtable.insert(tenant, entries);
    }
    max_ts = max_ts.max(buffered.max_ts_ns());
    rows += buffered.entry_count();

    Bed {
        _dir: dir,
        parts,
        memtable,
        tenant: tenant.expect("one tenant"),
        // Half-open on `end`, which is what a log query asks for; `+1` so the
        // newest row is inside the window rather than on its excluded boundary.
        range: QueryTimeRange::half_open(min_ts, max_ts.saturating_add(1)),
        app: app.expect("one app label value"),
        rows,
    }
}

fn shapes(bed: &Bed) -> Vec<(&'static str, LogQuery)> {
    let app = &bed.app;
    vec![
        ("label_only", parse(&format!("attr=app={app}"))),
        (
            "line_filter",
            parse(&format!("attr=app={app}&contains=timeout")),
        ),
        (
            "json_field",
            parse(&format!("parse=json&attr=app={app}&attr=status=500")),
        ),
    ]
}

fn parse(query: &str) -> LogQuery {
    signy::query::parse_filter_params(query, 0, signy::query::LOGS_PARAMS)
        .expect("bench query parses")
        .query
}

fn run(bed: &Bed, query: &LogQuery, forward: bool) -> signy::log_scan::LogScanResult {
    LogScan::new(&bed.tenant, query, bed.range, LIMIT, forward)
        .run(&bed.memtable, &bed.parts)
        .expect("bench scan succeeds")
}

/// Rows returned, lines processed and allocations per shape.
///
/// The three columns are one claim each: the answer did not change, the scan
/// stopped, and the materialization is bounded by the limit rather than by the
/// window.
fn report_allocations(bed: &Bed) {
    eprintln!();
    eprintln!(
        "=== allocation accounting: log query, limit {LIMIT} over {} rows in the window ===",
        bed.rows
    );
    eprintln!(
        "{:<14} {:>9} {:>8} {:>9} {:>13} {:>12} {:>12} {:>11}",
        "shape",
        "direction",
        "rows",
        "lines",
        "bytes alloc",
        "bytes/row",
        "peak live",
        "allocs/row"
    );
    for (name, query) in shapes(bed) {
        for (direction, forward) in [("backward", false), ("forward", true)] {
            let (result, stats) = corpus::alloc::measure(|| run(bed, &query, forward));
            let returned: usize = result
                .results
                .iter()
                .map(|stream| stream.entries.len())
                .sum();
            let per_row = returned.max(1) as f64;
            eprintln!(
                "{:<14} {:>9} {:>8} {:>9} {:>13} {:>12.1} {:>12} {:>11.2}",
                name,
                direction,
                returned,
                result.scanned_rows,
                stats.bytes_allocated,
                stats.bytes_allocated as f64 / per_row,
                stats.peak_live_bytes,
                stats.allocations as f64 / per_row,
            );
        }
    }
    eprintln!();
}

fn bench_shapes(c: &mut Criterion, bed: &Bed) {
    let mut group = c.benchmark_group("query/limit");
    group
        .sample_size(10)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(2));
    for (name, query) in shapes(bed) {
        for (direction, forward) in [("backward", false), ("forward", true)] {
            group.bench_function(format!("{name}/{direction}"), |b| {
                b.iter(|| run(bed, &query, forward));
            });
        }
    }
    group.finish();
}

/// The metric counting path: `sum(rate({app}[10s]))` over the bed, evaluated
/// on a 10-second grid across the whole range. No rows are materialized; what
/// this measures is the scan itself — decode, run detection, and two array
/// updates per row — which is the whole of what stands between this engine and
/// a columnar counter.
fn bench_rate(c: &mut Criterion, bed: &Bed) {
    let mut group = c.benchmark_group("query/rate");
    group
        .sample_size(10)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(2));
    let query = parse(&format!("attr=app={}", bed.app));
    let columns = signy::part::ColumnSet::for_count_query(&query);
    let times: Vec<i64> = (1..=20)
        .map(|step| 1_772_000_000_000_000_000i64 + step * 10_000_000_000)
        .collect();
    group.bench_function("sum_rate", |b| {
        b.iter(|| {
            let mut sink = signy::log_scan::CountingSink {
                query: &query,
                hidden: None,
                cancellation: None,
                times: &times,
                range_ns: 10_000_000_000,
                bytes: false,
                diff: vec![0.0; times.len() + 1],
                rows: 0,
            };
            LogScan::new(&bed.tenant, &query, bed.range, usize::MAX, true)
                .columns(columns.clone())
                .run_into(&bed.memtable, &bed.parts, &mut sink)
                .expect("bench scan succeeds");
            sink.rows
        });
    });
    group.finish();
}

fn benches(c: &mut Criterion) {
    let bed = bed();
    report_allocations(&bed);
    bench_shapes(c, &bed);
    bench_rate(c, &bed);
}

criterion_group!(query, benches);
criterion_main!(query);
