use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use opentelemetry_proto::tonic::resource::v1::Resource;
use prost::Message;

pub fn export_bytes(records: usize) -> Vec<u8> {
    let lines = [
        "GET /v1/checkout 200 in 31ms",
        "connection reset by peer while reading upstream",
        "cache miss for key user:8172:profile, falling back to postgres",
        "retrying publish attempt 2 of 5 after 400ms",
    ];
    let log_records = (0..records)
        .map(|index| LogRecord {
            time_unix_nano: 1_700_000_000_000_000_000 + index as u64,
            severity_number: 9,
            severity_text: "INFO".to_string(),
            body: Some(AnyValue {
                value: Some(any_value::Value::StringValue(format!(
                    "{} request_id=req-{index:08}",
                    lines[index % lines.len()]
                ))),
            }),
            attributes: vec![
                attribute("http.route", "/v1/checkout"),
                attribute("net.peer.name", "upstream-7"),
            ],
            ..Default::default()
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

fn attribute(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }),
        ..Default::default()
    }
}
