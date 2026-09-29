    use super::*;
    use opentelemetry_proto::tonic::common::v1::{InstrumentationScope, KeyValue};
    use opentelemetry_proto::tonic::logs::v1::{ResourceLogs, ScopeLogs};
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use std::collections::BTreeMap;

    fn string_value(value: &str) -> AnyValue {
        AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }
    }

    fn attribute(key: &str, value: AnyValue) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(value),
            ..Default::default()
        }
    }

    fn request(resource: Vec<KeyValue>, records: Vec<LogRecord>) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: resource,
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        name: "checkout".to_string(),
                        ..Default::default()
                    }),
                    log_records: records,
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        }
    }

    fn record(time_unix_nano: u64, body: &str) -> LogRecord {
        LogRecord {
            time_unix_nano,
            observed_time_unix_nano: 0,
            severity_number: 9,
            severity_text: "INFO".to_string(),
            body: Some(string_value(body)),
            ..Default::default()
        }
    }

    fn metadata_map(entry: &LogEntry) -> BTreeMap<&str, &str> {
        entry
            .structured_metadata
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect()
    }

    /// There is no stream concept: nothing is promoted to a label, and every
    /// resource attribute lands in each record's metadata with a key LogQL can
    /// name (dots become underscores, as Loki's OTLP intake normalizes too).
    #[test]
    fn every_resource_attribute_becomes_metadata_with_a_normalized_key() {
        let normalized = normalize_request(request(
            vec![
                attribute("service.name", string_value("checkout")),
                attribute("k8s.namespace.name", string_value("prod")),
                attribute("process.pid", string_value("41213")),
                attribute("telemetry.sdk.version", string_value("1.28.0")),
            ],
            vec![record(1_700_000_000_000_000_000, "order placed")],
        ))
        .unwrap();

        assert_eq!(normalized.len(), 1);
        let metadata = metadata_map(&normalized[0]);
        assert_eq!(metadata["service_name"], "checkout");
        assert_eq!(metadata["k8s_namespace_name"], "prod");
        assert_eq!(metadata["process_pid"], "41213");
        assert_eq!(metadata["telemetry_sdk_version"], "1.28.0");
        assert_eq!(metadata["severity_text"], "INFO");
        assert_eq!(metadata["severity_number"], "9");
        assert_eq!(metadata["scope_name"], "checkout");
    }

    /// An export is a flat run of records in arrival order; nothing groups
    /// them by resource any more.
    #[test]
    fn records_come_back_flat_in_arrival_order() {
        let normalized = normalize_request(request(
            vec![attribute("service.name", string_value("checkout"))],
            vec![
                record(1_700_000_000_000_000_000, "first"),
                record(1_700_000_001_000_000_000, "second"),
                record(1_700_000_002_000_000_000, "third"),
            ],
        ))
        .unwrap();

        assert_eq!(normalized.len(), 3);
        assert_eq!(normalized[2].line, "third");
        assert_eq!(
            metadata_map(&normalized[2])["service_name"],
            "checkout",
            "every record carries the resource attributes itself"
        );
    }

    /// `time_unix_nano` is optional in OTLP and a collector leaves it at zero
    /// when the source had none. Falling back to the observed time is the
    /// difference between a usable record and one stamped at the epoch.
    #[test]
    fn a_record_without_a_timestamp_falls_back_to_when_it_was_observed() {
        let mut without_time = record(0, "no timestamp at the source");
        without_time.observed_time_unix_nano = 1_700_000_005_000_000_000;
        let normalized = normalize_request(request(
            vec![attribute("service.name", string_value("checkout"))],
            vec![without_time],
        ))
        .unwrap();
        assert_eq!(normalized[0].timestamp_ns, 1_700_000_005_000_000_000);
    }

    /// Trace correlation is the reason logs and traces live in one engine, so
    /// the ids have to arrive in the form the trace side already speaks.
    #[test]
    fn trace_and_span_ids_arrive_as_hex_like_everywhere_else() {
        let mut correlated = record(1_700_000_000_000_000_000, "handled");
        correlated.trace_id = vec![0x0a; 16];
        correlated.span_id = vec![0xff, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let normalized = normalize_request(request(Vec::new(), vec![correlated])).unwrap();
        let metadata = metadata_map(&normalized[0]);
        assert_eq!(metadata["trace_id"], "0a".repeat(16));
        assert_eq!(metadata["span_id"], "ff01020304050607");
    }

    /// A non-scalar body is rendered as JSON rather than dropped, so the
    /// parser filters that already exist for structured lines can reach into
    /// it. Dropping it would lose the record's only content.
    #[test]
    fn a_composite_body_is_kept_as_json() {
        let mut structured = record(1_700_000_000_000_000_000, "");
        structured.body = Some(AnyValue {
            value: Some(any_value::Value::KvlistValue(
                opentelemetry_proto::tonic::common::v1::KeyValueList {
                    values: vec![attribute("event", string_value("checkout.completed"))],
                },
            )),
        });
        let normalized = normalize_request(request(Vec::new(), vec![structured])).unwrap();
        let line = &normalized[0].line;
        assert!(line.contains("checkout.completed"), "{line}");
    }

    #[test]
    fn an_export_with_no_records_is_rejected_rather_than_silently_accepted() {
        let error = normalize_request(ExportLogsServiceRequest::default())
            .err()
            .expect("an export with no records is an error");
        assert_eq!(error, OtlpLogError::EmptyRequest);
    }

    /// The journal stores the export as it arrived and replay re-normalizes,
    /// so what must round-trip is normalization itself — the same bytes must
    /// normalize to the same entries before and after a crash, awkward
    /// attribute values included.
    #[test]
    fn an_awkward_export_normalizes_identically_after_a_wal_round_trip() {
        let export = request(
            vec![attribute(
                "service.name",
                string_value(r#"say "hi"\ then a newline
"#),
            )],
            vec![record(1_700_000_000_000_000_000, "line")],
        );
        let encoded = prost014::Message::encode_to_vec(&export);
        let before = normalize_request(export).unwrap();

        let decoded = <ExportLogsServiceRequest as prost014::Message>::decode(encoded.as_slice())
            .expect("the WAL payload decodes");
        let after = normalize_request(decoded).expect("and normalizes again");
        assert_eq!(before.len(), after.len());
        for (entry_before, entry_after) in before.iter().zip(&after) {
            assert_eq!(entry_before.timestamp_ns, entry_after.timestamp_ns);
            assert_eq!(entry_before.line, entry_after.line);
            assert_eq!(
                entry_before.structured_metadata,
                entry_after.structured_metadata
            );
        }
        assert_eq!(after[0].timestamp_ns, 1_700_000_000_000_000_000);
    }
