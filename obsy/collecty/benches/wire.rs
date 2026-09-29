#[path = "corpus.rs"]
mod corpus;

use std::io::Write;

use collecty::wire;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};

/// A segment's worth of exports through one stream, which is how the queue
/// compresses: the encoder is opened once and fed a record at a time.
fn segment(records: usize, exports: usize, level: i32) -> std::io::Result<Vec<u8>> {
    let export = corpus::export_bytes(records);
    let plain = wire::frame_record(&export);
    let mut encoder = zstd::stream::write::Encoder::new(Vec::new(), level)?;
    for _ in 0..exports {
        encoder.write_all(std::hint::black_box(&plain))?;
    }
    encoder.finish()
}

fn compression(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("wire/compress");
    for records in [16usize, 256, 4096] {
        let plain = wire::RECORD_HEADER_BYTES + corpus::export_bytes(records).len();
        group.throughput(Throughput::Bytes((plain * 64) as u64));
        group.bench_function(format!("{records}-records"), |bencher| {
            bencher.iter(|| segment(records, 64, wire::ZSTD_LEVEL).expect("a segment"))
        });
    }
    group.finish();
}

fn levels(criterion: &mut Criterion) {
    let plain = wire::RECORD_HEADER_BYTES + corpus::export_bytes(256).len();
    let mut group = criterion.benchmark_group("wire/level");
    group.throughput(Throughput::Bytes((plain * 64) as u64));
    for level in [1i32, 3, 6, 12] {
        group.bench_function(format!("level-{level}"), |bencher| {
            bencher.iter(|| segment(256, 64, level).expect("a segment"))
        });
    }
    group.finish();
}

fn segment_decompression(criterion: &mut Criterion) {
    let body = segment(64, 64, wire::ZSTD_LEVEL).expect("a segment");
    let plain = (wire::RECORD_HEADER_BYTES + corpus::export_bytes(64).len()) * 64;

    let mut group = criterion.benchmark_group("wire/decompress-segment");
    group.throughput(Throughput::Bytes(plain as u64));
    group.bench_function("64-records", |bencher| {
        bencher.iter(|| {
            let plain = wire::decompress(std::hint::black_box(&body), plain)
                .expect("a decompressed segment");
            wire::split_records(&plain).expect("framed records").len()
        })
    });
    group.finish();
}

criterion_group!(benches, compression, levels, segment_decompression);
criterion_main!(benches);
