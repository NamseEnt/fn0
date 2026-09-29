//! WAL append.
//!
//! Three things are separable here and were not before:
//!
//! * `append/serial` is one record per fsync — the durability floor.
//! * `append/batched` issues N appends before awaiting any, so `writer_loop`
//!   group-commits them behind one fsync. Divided by N it is the per-record
//!   batching cost with the syscall amortized away.
//! * `fsync/*` writes the same bytes to a plain file with and without
//!   `sync_all`, which is the syscall's own cost on this filesystem. The
//!   journal cannot be asked to skip its fsync — that is the point of it — so
//!   the baseline is measured beside it rather than inside it.
//!
//! `encode_push_request` was here too, measuring the second message the JSON
//! and OTLP paths built for the WAL so replay had one decoder. The WAL stores
//! the OTLP export itself now, so the thing that bench measured no longer
//! exists (`todo.md`, "Next — OTLP only").

#[path = "corpus/mod.rs"]
#[allow(dead_code)]
mod corpus;

use std::io::{Seek, SeekFrom, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use signy::config::Config;
use signy::journal::Journal;
use signy::memtable::MemTable;
use signy::tenant::TenantId;

use corpus::{CorpusSpec, scratch::ScratchDir};

/// Criterion warms up for three seconds per bench function by default,
/// which across this suite is minutes of warm-up alone. These are a
/// regression gate, not a distribution.
const WARM_UP: Duration = Duration::from_millis(500);
/// Small enough that a sweep is a plausible push body rather than a bulk load.
const ENTRIES_PER_PUSH: usize = 8;
/// A bench that appends for a second writes gigabytes. Every target here uses
/// `iter_custom` so the reclaim happens between timed iterations rather than
/// inside them — and for the journal the reclaim is `compact_checkpoint`,
/// which is the production path for the same problem.
const RETAINED_BYTES: u64 = 32 * 1024 * 1024;

fn push_payload() -> (Vec<u8>, TenantId) {
    let corpus = corpus::generate(
        &CorpusSpec::default()
            .rows(ENTRIES_PER_PUSH)
            .streams(1)
            .labels_per_stream(6),
    );
    (encode_otlp(&corpus.streams), corpus.tenant().clone())
}

/// The bytes the WAL stores: the export itself, one `ResourceLogs` per
/// generator stream, the attributes riding as resource attributes.
fn encode_otlp(streams: &[signy::corpus::Stream]) -> Vec<u8> {
    use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
    use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use opentelemetry_proto::tonic::resource::v1::Resource;
    let attr = |key: &str, value: &str| KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }),
        ..Default::default()
    };
    let request = ExportLogsServiceRequest {
        resource_logs: streams
            .iter()
            .map(|stream| ResourceLogs {
                resource: Some(Resource {
                    attributes: stream.labels.iter().map(|(k, v)| attr(k, v)).collect(),
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    log_records: stream
                        .entries
                        .iter()
                        .map(|entry| LogRecord {
                            time_unix_nano: entry.timestamp_ns.max(0) as u64,
                            body: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(entry.line.clone())),
                            }),
                            attributes: entry
                                .structured_metadata
                                .iter()
                                .map(|(k, v)| attr(k, v))
                                .collect(),
                            ..Default::default()
                        })
                        .collect(),
                    ..Default::default()
                }],
                ..Default::default()
            })
            .collect(),
    };
    prost014::Message::encode_to_vec(&request)
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("bench runtime builds")
}

fn spawn_journal(dir: &ScratchDir) -> Journal {
    let config = Config {
        data_dir: dir.path().to_path_buf(),
        ..Config::default()
    };
    Journal::spawn(&config, Arc::new(MemTable::new())).expect("bench journal spawns")
}

async fn drop_wal_prefix(journal: &Journal) {
    let length = std::fs::metadata(journal.wal_path())
        .map(|meta| meta.len())
        .unwrap_or(0);
    if length == 0 {
        return;
    }
    journal
        .compact_checkpoint(length)
        .await
        .expect("wal compaction succeeds");
}

fn bench_append(c: &mut Criterion) {
    let (payload, tenant) = push_payload();
    let runtime = runtime();
    let dir = ScratchDir::new("wal");
    let journal = runtime.block_on(async { spawn_journal(&dir) });
    let record_bytes = payload.len() as u64;

    let mut group = c.benchmark_group("wal/append");
    group
        .sample_size(10)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(1));

    {
        // Streams are left empty on purpose: the memtable insert that normally
        // rides along has its own bench, and letting it run here would grow
        // the memtable without bound across the sweep.
        let journal = &journal;
        let payload = &payload;
        let tenant = &tenant;
        group.throughput(Throughput::Bytes(record_bytes));
        group.bench_function("serial", |b| {
            b.to_async(&runtime).iter_custom(move |iters| async move {
                let mut elapsed = Duration::ZERO;
                let mut unreclaimed = 0u64;
                for _ in 0..iters {
                    let record = payload.clone();
                    let started = Instant::now();
                    journal
                        .append_otlp_logs(tenant.clone(), record, Vec::new())
                        .await
                        .expect("append succeeds");
                    elapsed += started.elapsed();
                    unreclaimed += record_bytes;
                    if unreclaimed >= RETAINED_BYTES {
                        drop_wal_prefix(journal).await;
                        unreclaimed = 0;
                    }
                }
                elapsed
            });
        });

        for batch in [8usize, 32] {
            group.throughput(Throughput::Bytes(record_bytes * batch as u64));
            group.bench_with_input(BenchmarkId::new("batched", batch), &batch, |b, &batch| {
                b.to_async(&runtime).iter_custom(move |iters| async move {
                    let mut elapsed = Duration::ZERO;
                    let mut unreclaimed = 0u64;
                    for _ in 0..iters {
                        let records: Vec<Vec<u8>> = (0..batch).map(|_| payload.clone()).collect();
                        let started = Instant::now();
                        let appends = records.into_iter().map(|record| {
                            journal.append_otlp_logs(tenant.clone(), record, Vec::new())
                        });
                        for result in futures_util::future::join_all(appends).await {
                            result.expect("append succeeds");
                        }
                        elapsed += started.elapsed();
                        unreclaimed += record_bytes * batch as u64;
                        if unreclaimed >= RETAINED_BYTES {
                            drop_wal_prefix(journal).await;
                            unreclaimed = 0;
                        }
                    }
                    elapsed
                });
            });
        }
    }
    group.finish();
    drop(journal);
}

/// The syscall, on its own, at the same byte size.
fn bench_fsync(c: &mut Criterion) {
    let (payload, _) = push_payload();
    let dir = ScratchDir::new("wal-fsync");

    let mut group = c.benchmark_group("wal/fsync");
    group
        .sample_size(10)
        .warm_up_time(WARM_UP)
        .measurement_time(Duration::from_secs(1))
        .throughput(Throughput::Bytes(payload.len() as u64));

    for (name, sync) in [("write_only", false), ("write_and_sync", true)] {
        let path = dir.path().join(format!("{name}.bin"));
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)
            .expect("bench file opens");
        group.bench_function(name, |b| {
            b.iter_custom(|iters| {
                let mut elapsed = Duration::ZERO;
                let mut unreclaimed = 0u64;
                for _ in 0..iters {
                    let started = Instant::now();
                    file.write_all(&payload).expect("write succeeds");
                    if sync {
                        file.sync_all().expect("sync succeeds");
                    }
                    elapsed += started.elapsed();
                    unreclaimed += payload.len() as u64;
                    if unreclaimed >= RETAINED_BYTES {
                        file.set_len(0).expect("truncate succeeds");
                        file.seek(SeekFrom::Start(0)).expect("rewind succeeds");
                        unreclaimed = 0;
                    }
                }
                elapsed
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_append, bench_fsync);
criterion_main!(benches);
