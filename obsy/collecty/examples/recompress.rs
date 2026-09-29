//! Throwaway: what decompress-and-recompress-at-send actually costs.
use std::io::Write;
use std::time::Instant;

use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use opentelemetry_proto::tonic::resource::v1::Resource;
use prost::Message;

fn attribute(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }),
        ..Default::default()
    }
}

/// One export, distinct from every other: a real collector never sends the
/// same batch twice.
fn export(records: usize, batch: usize) -> Vec<u8> {
    let lines = [
        "GET /v1/checkout 200 in 31ms",
        "connection reset by peer while reading upstream",
        "cache miss for key user:8172:profile, falling back to postgres",
        "retrying publish attempt 2 of 5 after 400ms",
    ];
    let log_records = (0..records)
        .map(|index| {
            let seq = batch * records + index;
            LogRecord {
                time_unix_nano: 1_700_000_000_000_000_000 + seq as u64 * 1_000_000,
                severity_number: 9,
                severity_text: "INFO".to_string(),
                body: Some(AnyValue {
                    value: Some(any_value::Value::StringValue(format!(
                        "{} request_id=req-{seq:08}",
                        lines[seq % lines.len()]
                    ))),
                }),
                attributes: vec![
                    attribute("http.route", "/v1/checkout"),
                    attribute("net.peer.name", &format!("upstream-{}", seq % 8)),
                ],
                ..Default::default()
            }
        })
        .collect();

    ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes: vec![
                    attribute("service.name", "checkout"),
                    attribute("deployment.environment", "production"),
                ],
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                log_records,
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
    .encode_to_vec()
}

fn framed(payload: &[u8]) -> Vec<u8> {
    let mut plain = Vec::with_capacity(5 + payload.len());
    plain.push(1u8);
    plain.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    plain.extend_from_slice(payload);
    plain
}

fn main() {
    for (records, exports) in [(512usize, 300usize), (64, 2400), (8, 19200)] {
        let bodies: Vec<Vec<u8>> = (0..exports)
            .map(|batch| framed(&export(records, batch)))
            .collect();
        let plain_bytes: usize = bodies.iter().map(|b| b.len()).sum();

        // Intake: one frame per export, as it is today.
        let start = Instant::now();
        let frames: Vec<Vec<u8>> = bodies
            .iter()
            .map(|body| zstd::encode_all(body.as_slice(), 3).unwrap())
            .collect();
        let intake = start.elapsed();
        let stored: usize = frames.iter().map(|f| f.len()).sum();

        let segment: Vec<u8> = frames.concat();

        // Send: unwrap every frame and compress the lot as one stream.
        let start = Instant::now();
        let mut encoder = zstd::stream::write::Encoder::new(Vec::new(), 3).unwrap();
        let mut at = 0usize;
        for frame in &frames {
            let plain = zstd::decode_all(&segment[at..at + frame.len()]).unwrap();
            encoder.write_all(&plain).unwrap();
            at += frame.len();
        }
        let wire = encoder.finish().unwrap();
        let resend = start.elapsed();

        let mib = |bytes: usize| bytes as f64 / (1024.0 * 1024.0);
        println!(
            "{records:>4} rec/export x {exports:>5} | plain {:>7.1} MiB | disk {:>6.2} MiB | wire {:>6.2} MiB ({:>+5.1}%) | intake {:>7.1} ms | recompress {:>7.1} ms ({:.2}x)",
            mib(plain_bytes),
            mib(stored),
            mib(wire.len()),
            (wire.len() as f64 / stored as f64 - 1.0) * 100.0,
            intake.as_secs_f64() * 1e3,
            resend.as_secs_f64() * 1e3,
            resend.as_secs_f64() / intake.as_secs_f64(),
        );
    }
}
