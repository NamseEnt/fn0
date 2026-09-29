//! What a `limit=100` costs as the *window* grows, holding the answer fixed.
//!
//! `benches/query.rs` measures one window size. The ten-times comparison bed
//! measured what happens when it grows, and the number it produced does not fit
//! a scan cost (`docs/COMPARISON_LARGE_CORPUS.md`, todo.md item 3):
//!
//! | `line_filter`, limit 100 | 150 k rows | 1.5 M rows | ratio |
//! |---|---|---|---|
//! | time | 1.55 ms | 13.6 ms | **×8.8** |
//! | rows returned | 2400 | 2400 | ×1.0 |
//! | lines scanned | 426 064 | 288 295 | **×0.68** |
//!
//! Ten times the data, nine times the time, and the scan read *less* to produce
//! the same answer. So the cost is not in the rows scanned and not in the rows
//! returned, and the part count cannot explain it either: the bed held 1.5 M
//! rows in **two** parts, merge having consolidated them.
//!
//! That leaves the structures a query walks whose size follows the window
//! rather than the answer, and the row group is the one with ten times as many
//! of it. Hence two sweeps, each holding constant what the other varies:
//!
//! * `window` — rows grow 150 k → 1.5 M at a fixed `row_group_size`, so the row
//!   group count grows with them. This is the bed's condition, reproduced
//!   without Loki, Docker or a network in the way.
//! * `row_groups` — the row count is **pinned at 1.5 M** and `row_group_size`
//!   grows instead, so the same rows arrive in 183, 46 or 11 groups. If time
//!   falls as the groups get fewer while the rows and the answer stay put, the
//!   cost is per row group and the first sweep is measuring row group count
//!   wearing a row count's clothes.
//!
//! This bench decides between those two and nothing else. It is a diagnosis
//! instrument: `line_filter` growing worse than its input is a scaling property
//! and in scope for production readiness, while making a fast shape faster is
//! the optimization frozen on 2026-08-07 (todo.md).

#[path = "corpus/mod.rs"]
#[allow(dead_code)]
mod corpus;

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use signy::log_scan::LogScan;
use signy::logql::LogQuery;
use signy::memtable::MemTable;
use signy::part::{QueryTimeRange, flush_rows};
use signy::part_registry::PartRegistry;
use signy::tenant::TenantId;

use corpus::{CorpusSpec, Shape, scratch::ScratchDir};

const WARM_UP: Duration = Duration::from_millis(500);

/// `Config::default().row_group_size`, and what the bed wrote with.
const ROW_GROUP_SIZE: usize = 8192;
/// Grafana's default, and the limit both bed runs queried with.
const LIMIT: usize = 100;
/// The bed's own sizes: its published dataset and the ten-times run.
const WINDOW_ROWS: [usize; 3] = [150_000, 500_000, 1_500_000];
/// Two parts, which is what the bed actually held at 1.5 M rows after merge —
/// so the sweep varies the window and not the part count.
const PARTS: usize = 2;

struct Bed {
    _dir: ScratchDir,
    parts: PartRegistry,
    memtable: MemTable,
    tenant: TenantId,
    range: QueryTimeRange,
    app: String,
    rows: usize,
    row_groups: usize,
}

/// `rows` total across `PARTS` parts, each written at `row_group_size`.
fn bed(rows: usize, row_group_size: usize) -> Bed {
    let dir = ScratchDir::new("scan-scaling-bed");
    let parts = PartRegistry::new();
    let memtable = MemTable::new();
    let mut tenant = None;
    let mut app = None;
    let mut min_ts = i64::MAX;
    let mut max_ts = i64::MIN;
    let mut written_rows = 0usize;
    let mut row_groups = 0usize;
    let base = CorpusSpec::default();
    let rows_per_part = rows / PARTS;
    for index in 0..PARTS {
        let corpus = corpus::generate(
            &CorpusSpec::default()
                .rows(rows_per_part)
                .streams(32)
                .labels_per_stream(6)
                .only(Shape::Json)
                .out_of_order(true)
                .seed(base.seed + index as u64)
                // Consecutive and non-overlapping, as a flush interval
                // produces: the window is the union, so it grows with the rows.
                .start_ts_ns(base.start_ts_ns + (index * rows_per_part) as i64 * base.step_ns),
        );
        min_ts = min_ts.min(corpus.min_ts_ns());
        max_ts = max_ts.max(corpus.max_ts_ns());
        written_rows += corpus.entry_count();
        tenant.get_or_insert_with(|| corpus.tenant().clone());
        app.get_or_insert_with(|| corpus.label_value("app"));
        let written = flush_rows(corpus.rows(), dir.path(), row_group_size)
            .expect("bench corpus flushes to a part");
        for part in &written {
            row_groups += part.meta.row_group_count as usize;
        }
        parts.register(written).expect("bench part registers");
    }
    // The same buffered tail `benches/query.rs` holds, and for the same reason:
    // a backward query reads the memtable before it reads a part, and a scan
    // that stops early has to stop early in both.
    let buffered = corpus::generate(
        &CorpusSpec::default()
            .rows(2_000)
            .streams(32)
            .labels_per_stream(6)
            .only(Shape::Json)
            .seed(base.seed + 1_000)
            .start_ts_ns(max_ts + base.step_ns),
    );
    for (tenant, entries) in buffered.snapshot() {
        memtable.insert(tenant, entries);
    }
    max_ts = max_ts.max(buffered.max_ts_ns());
    written_rows += buffered.entry_count();

    Bed {
        _dir: dir,
        parts,
        memtable,
        tenant: tenant.expect("one tenant"),
        range: QueryTimeRange::half_open(min_ts, max_ts.saturating_add(1)),
        app: app.expect("one app label value"),
        rows: written_rows,
        row_groups,
    }
}

/// The bed's own `line_filter` phrase, which the corpus plants in every shape.
fn query_of(bed: &Bed) -> LogQuery {
    signy::query::parse_filter_params(
        &format!("attr=app={}&contains=timeout", bed.app),
        0,
        signy::query::LOGS_PARAMS,
    )
    .expect("bench query parses")
    .query
}

fn run(bed: &Bed, query: &LogQuery) -> signy::log_scan::LogScanResult {
    // Backward: the direction a log view asks for, and the one the limit can
    // stop early in.
    LogScan::new(&bed.tenant, query, bed.range, LIMIT, false)
        .run(&bed.memtable, &bed.parts)
        .expect("bench scan succeeds")
}

/// Rows returned and lines scanned per sweep point, printed before the timings.
///
/// Without these the timing table cannot be read: a point that got slower while
/// also returning more rows has not shown anything, and the whole question is
/// whether the work moved.
fn report(label: &str, points: &[(String, &Bed)]) {
    eprintln!();
    eprintln!("=== {label}: limit {LIMIT}, backward, `|= \"timeout\"` ===");
    eprintln!(
        "{:<14} {:>12} {:>12} {:>10} {:>12} {:>16}",
        "point", "window rows", "row groups", "returned", "lines", "lines/rowgroup"
    );
    for (name, bed) in points {
        let result = run(bed, &query_of(bed));
        let returned: usize = result
            .results
            .iter()
            .map(|stream| stream.entries.len())
            .sum();
        eprintln!(
            "{:<14} {:>12} {:>12} {:>10} {:>12} {:>16.1}",
            name,
            bed.rows,
            bed.row_groups,
            returned,
            result.scanned_rows,
            result.scanned_rows as f64 / bed.row_groups.max(1) as f64,
        );
    }
    eprintln!();
}

fn bench(c: &mut Criterion) {
    // Sweep one: the bed's condition. Rows grow, row group size fixed, so the
    // row group count grows with the rows.
    let windows: Vec<(String, Bed)> = WINDOW_ROWS
        .iter()
        .map(|rows| (format!("{}k", rows / 1000), bed(*rows, ROW_GROUP_SIZE)))
        .collect();
    report(
        "window sweep (row_group_size fixed at 8192)",
        &windows
            .iter()
            .map(|(name, bed)| (name.clone(), bed))
            .collect::<Vec<_>>(),
    );

    // Sweep two: the same 1.5 M rows arriving in fewer, larger groups. The
    // largest is 65 536 rather than something rounder because that is the
    // ceiling the format has: a group's bloom windows are `BLOOM_WINDOW_ROWS`
    // = 1024 rows each and the selection mask is a `u64`, so 64 windows is the
    // most a group can carry (`part/reader.rs`, "exceeds the 64-window limit").
    // Eight times the default is therefore the whole room this sweep has, which
    // is worth knowing before anyone proposes bigger groups as a remedy.
    let largest = *WINDOW_ROWS.last().expect("a largest window");
    let groups: Vec<(String, Bed)> = [ROW_GROUP_SIZE, ROW_GROUP_SIZE * 4, ROW_GROUP_SIZE * 8]
        .iter()
        .map(|size| (format!("rg{size}"), bed(largest, *size)))
        .collect();
    report(
        "row group sweep (rows pinned at 1.5 M)",
        &groups
            .iter()
            .map(|(name, bed)| (name.clone(), bed))
            .collect::<Vec<_>>(),
    );

    let mut group = c.benchmark_group("scan_scaling");
    group
        .sample_size(10)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(3));
    for (name, bed) in &windows {
        let query = query_of(bed);
        group.bench_with_input(BenchmarkId::new("window", name), bed, |b, bed| {
            b.iter(|| run(bed, &query))
        });
    }
    for (name, bed) in &groups {
        let query = query_of(bed);
        group.bench_with_input(BenchmarkId::new("row_groups", name), bed, |b, bed| {
            b.iter(|| run(bed, &query))
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
