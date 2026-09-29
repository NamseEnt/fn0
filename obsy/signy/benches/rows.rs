//! Flush-path row materialization: `rows_from_snapshot` and `Row::from_entry`.
//!
//! Attributes live in each entry's own metadata now, so the cost being held
//! down is bytes-allocated per row across the attribute sweep — this binary
//! carries the counting allocator and prints that table before the timings.

#[path = "corpus/mod.rs"]
#[allow(dead_code)]
mod corpus;

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use signy::part::{Row, rows_from_snapshot};

use corpus::CorpusSpec;

/// Criterion warms up for three seconds per bench function by default,
/// which across this suite is minutes of warm-up alone. These are a
/// regression gate, not a distribution.
const WARM_UP: Duration = Duration::from_millis(500);
#[global_allocator]
static ALLOCATOR: corpus::alloc::CountingAllocator = corpus::alloc::CountingAllocator;

const ROWS: usize = 20_000;
const LABEL_SWEEP: [usize; 3] = [2, 5, 10];
const STREAM_SWEEP: [usize; 3] = [1, 256, 8_192];

fn bench_rows_from_snapshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("rows/from_snapshot");
    group
        .sample_size(10)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(3));
    for labels_per_stream in LABEL_SWEEP {
        for streams in STREAM_SWEEP {
            let corpus = corpus::generate(
                &CorpusSpec::default()
                    .rows(ROWS)
                    .streams(streams)
                    .labels_per_stream(labels_per_stream)
                    .out_of_order(true),
            );
            let snapshot = corpus.snapshot();
            group.throughput(Throughput::Elements(corpus.entry_count() as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("labels={labels_per_stream}"), streams),
                &snapshot,
                |b, snapshot| b.iter(|| rows_from_snapshot(snapshot)),
            );
        }
    }
    group.finish();
}

/// One row, without the surrounding sort, so a change in the clone itself is
/// separable from a change in `sort_rows`.
fn bench_row_from_entry(c: &mut Criterion) {
    let mut group = c.benchmark_group("rows/from_entry");
    group
        .sample_size(20)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(2));
    for labels_per_stream in LABEL_SWEEP {
        let corpus = corpus::generate(
            &CorpusSpec::default()
                .rows(4_096)
                .streams(64)
                .labels_per_stream(labels_per_stream),
        );
        let snapshot = corpus.snapshot();
        let tenant = corpus.tenant().clone();
        let entries: &Vec<_> = snapshot.get(&tenant).expect("the corpus fills its tenant");
        group.throughput(Throughput::Elements(entries.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(labels_per_stream),
            entries,
            |b, entries| {
                b.iter(|| {
                    let mut rows = Vec::with_capacity(entries.len());
                    for entry in entries {
                        rows.push(Row::from_entry(&tenant, entry));
                    }
                    rows
                });
            },
        );
    }
    group.finish();
}

/// The memory half of the same sweep: `peak live` is the whole `Vec<Row>`
/// plus the sort's scratch, watched across attribute count and variety.
fn report_allocations() {
    corpus::alloc::header("rows_from_snapshot (flush-path materialization)");
    for labels_per_stream in LABEL_SWEEP {
        for streams in STREAM_SWEEP {
            let corpus = corpus::generate(
                &CorpusSpec::default()
                    .rows(ROWS)
                    .streams(streams)
                    .labels_per_stream(labels_per_stream)
                    .out_of_order(true),
            );
            let snapshot = corpus.snapshot();
            let entries = corpus.entry_count();
            let distinct_label_bytes = corpus.distinct_label_bytes();
            let (rows, stats) = corpus::alloc::measure(|| rows_from_snapshot(&snapshot));
            corpus::alloc::row(
                &format!("labels={labels_per_stream} streams={streams}"),
                entries,
                &stats,
                &format!(
                    "{} rows ({} distinct attribute bytes in the sweep)",
                    rows.len(),
                    distinct_label_bytes,
                ),
            );
        }
    }
    corpus::alloc::footer();
}

fn benches(c: &mut Criterion) {
    report_allocations();
    bench_rows_from_snapshot(c);
    bench_row_from_entry(c);
}

criterion_group!(rows, benches);
criterion_main!(rows);
