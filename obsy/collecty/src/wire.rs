use std::io::Read;

pub const ZSTD_LEVEL: i32 = 3;

pub const RECORD_HEADER_BYTES: usize = 4;

/// A record as it goes into a segment: the payload's length and the payload.
/// Uncompressed — the segment compresses the whole of itself as one zstd
/// stream, so nothing here knows about compression.
///
/// Nothing here names a signal either. A segment holds one signal's records
/// and says so on the wire, so repeating the answer in front of every record
/// would be paying per export to learn what the request already carries.
pub fn frame_record(payload: &[u8]) -> Vec<u8> {
    let mut plain = Vec::with_capacity(RECORD_HEADER_BYTES + payload.len());
    plain.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    plain.extend_from_slice(payload);
    plain
}

pub fn split_records(plain: &[u8]) -> Result<Vec<&[u8]>, String> {
    let mut records = Vec::new();
    let mut at = 0;
    while at < plain.len() {
        let Some(header) = plain.get(at..at + RECORD_HEADER_BYTES) else {
            return Err(format!(
                "a record header needs {RECORD_HEADER_BYTES} bytes and {} are left",
                plain.len() - at
            ));
        };
        let len = u32::from_le_bytes(header.try_into().expect("four bytes")) as usize;
        at += RECORD_HEADER_BYTES;
        let Some(payload) = plain.get(at..at + len) else {
            return Err(format!(
                "a record claims {len} bytes and {} are left",
                plain.len() - at
            ));
        };
        at += len;
        records.push(payload);
    }
    Ok(records)
}

/// How much of `plain` is whole records.
///
/// What a crash leaves in a segment ends wherever the encoder happened to be,
/// which is not a record boundary. signy refuses a batch whose last record is
/// cut short, and refusing means the whole segment is dropped, so this is
/// where recovery cuts instead.
pub fn whole_records_len(plain: &[u8]) -> usize {
    let mut at = 0;
    loop {
        let Some(header) = plain.get(at..at + RECORD_HEADER_BYTES) else {
            return at;
        };
        let len = u32::from_le_bytes(header.try_into().expect("four bytes")) as usize;
        let end = at + RECORD_HEADER_BYTES + len;
        if end > plain.len() {
            return at;
        }
        at = end;
    }
}

pub fn decompress(body: &[u8], expected_plain_len: usize) -> std::io::Result<Vec<u8>> {
    let mut plain = Vec::with_capacity(expected_plain_len);
    zstd::stream::read::Decoder::new(body)?.read_to_end(&mut plain)?;
    Ok(plain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::collector::logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse,
    };
    use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use opentelemetry_proto::tonic::common::v1::{
        AnyValue, InstrumentationScope, KeyValue, any_value,
    };
    use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use opentelemetry_proto::tonic::metrics::v1::ResourceMetrics;
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use opentelemetry_proto::tonic::trace::v1::ResourceSpans;
    use prost::Message;

    fn request_with(service: &str, body: &str) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "service.name".to_string(),
                        value: Some(AnyValue {
                            value: Some(any_value::Value::StringValue(service.to_string())),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope::default()),
                    log_records: vec![LogRecord {
                        time_unix_nano: 1,
                        body: Some(AnyValue {
                            value: Some(any_value::Value::StringValue(body.to_string())),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    /// A segment is one zstd stream over every record it took, and the file is
    /// what goes on the wire unchanged.
    #[test]
    fn one_stream_over_many_records_decompresses_to_all_of_them() {
        let first = b"the first request's bytes, long enough to be worth compressing";
        let second = b"the second request's bytes, also long enough to be worth it";

        let mut plain = frame_record(first);
        plain.extend_from_slice(&frame_record(second));
        let body = zstd::encode_all(plain.as_slice(), ZSTD_LEVEL).unwrap();

        let decompressed = decompress(&body, plain.len()).unwrap();
        let records = split_records(&decompressed).unwrap();

        assert_eq!(records, vec![first.as_slice(), second.as_slice()]);
    }

    #[test]
    fn concatenated_export_requests_decode_as_one_merged_request() {
        let first = request_with("api", "first line");
        let second = request_with("worker", "second line");

        let mut bytes = first.encode_to_vec();
        bytes.extend_from_slice(&second.encode_to_vec());

        let merged = ExportLogsServiceRequest::decode(bytes.as_slice()).unwrap();

        assert_eq!(merged.resource_logs.len(), 2);
        assert_eq!(merged.resource_logs[0], first.resource_logs[0]);
        assert_eq!(merged.resource_logs[1], second.resource_logs[0]);
    }

    #[test]
    fn a_batch_survives_both_layers_at_once() {
        let requests: Vec<_> = (0..8)
            .map(|index| request_with(&format!("service-{index}"), &format!("line {index}")))
            .collect();

        let mut written = Vec::new();
        for request in &requests {
            written.extend_from_slice(&frame_record(&request.encode_to_vec()));
        }
        let body = zstd::encode_all(written.as_slice(), ZSTD_LEVEL).unwrap();

        let plain = decompress(&body, written.len()).unwrap();
        assert_eq!(plain, written);
        let mut merged_bytes = Vec::new();
        for payload in split_records(&plain).unwrap() {
            merged_bytes.extend_from_slice(payload);
        }
        let merged = ExportLogsServiceRequest::decode(merged_bytes.as_slice()).unwrap();

        assert_eq!(merged.resource_logs.len(), requests.len());
        for (index, request) in requests.iter().enumerate() {
            assert_eq!(merged.resource_logs[index], request.resource_logs[0]);
        }
    }

    /// A crash cuts the stream wherever the encoder happened to be. What is
    /// kept is the last record that arrived whole.
    #[test]
    fn a_recovered_stream_is_cut_at_the_last_whole_record() {
        let mut plain = frame_record(b"first");
        plain.extend_from_slice(&frame_record(b"second"));
        let whole = plain.len();
        plain.extend_from_slice(&frame_record(b"cut short"));
        plain.truncate(whole + RECORD_HEADER_BYTES + 3);

        assert_eq!(whole_records_len(&plain), whole);
        assert_eq!(whole_records_len(&plain[..whole + 2]), whole);
        assert_eq!(whole_records_len(&[]), 0);

        plain.truncate(whole);
        assert_eq!(
            split_records(&plain).unwrap(),
            vec![b"first".as_slice(), b"second".as_slice()]
        );
    }

    #[test]
    fn a_record_that_claims_more_than_it_carries_is_refused() {
        let mut plain = Vec::new();
        plain.extend_from_slice(&64u32.to_le_bytes());
        plain.extend_from_slice(b"only eight");

        let error = split_records(&plain).unwrap_err();
        assert!(error.contains("claims 64 bytes"), "{error}");
    }

    #[test]
    fn traces_and_metrics_carry_the_same_single_repeated_field() {
        let mut spans = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans::default()],
        }
        .encode_to_vec();
        spans.extend_from_slice(
            &ExportTraceServiceRequest {
                resource_spans: vec![ResourceSpans::default()],
            }
            .encode_to_vec(),
        );
        assert_eq!(
            ExportTraceServiceRequest::decode(spans.as_slice())
                .unwrap()
                .resource_spans
                .len(),
            2
        );

        let mut metrics = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics::default()],
        }
        .encode_to_vec();
        metrics.extend_from_slice(
            &ExportMetricsServiceRequest {
                resource_metrics: vec![ResourceMetrics::default()],
            }
            .encode_to_vec(),
        );
        assert_eq!(
            ExportMetricsServiceRequest::decode(metrics.as_slice())
                .unwrap()
                .resource_metrics
                .len(),
            2
        );
    }

    #[test]
    fn an_empty_body_is_a_successful_export_response() {
        let response = ExportLogsServiceResponse::decode([].as_slice()).unwrap();
        assert!(response.partial_success.is_none());
        assert!(response.encode_to_vec().is_empty());
    }
}
