//! The OTLP body every target ingests, and the label vocabulary it forces.
//!
//! The bed used to speak Loki push because all three systems accepted it; the
//! engine's decided ingest is OTLP only, so the bed sends what the one
//! intended consumer sends. One consequence is measured rather than styled:
//! OTLP resource attributes have semconv names, and the systems that promote
//! them to stream labels (signy and Loki, the same default list) sanitize
//! the dots to underscores. The corpus's `app` therefore leaves this process
//! as `service.name` and is queried as `service_name` — the mapping lives
//! here, in one place, applied identically to every target.

use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
use opentelemetry_proto::tonic::resource::v1::Resource;
use prost014::Message;
use signy::memtable::{Labels, LogEntry};

/// The corpus label vocabulary as OTel semconv resource attributes.
///
/// Every name on the right is in the promotion list signy and Loki share,
/// so the stream labels a query sees are these names with dots sanitized to
/// underscores. A corpus label with no semconv equivalent would ride along
/// unpromoted and become structured metadata instead — none does today, and
/// the identity arm keeps that failure visible as a query miss rather than a
/// silent rename.
pub fn resource_attribute_name(label: &str) -> String {
    match label {
        "app" => "service.name",
        "env" => "deployment.environment",
        "cluster" => "k8s.cluster.name",
        "namespace" => "k8s.namespace.name",
        "container" => "k8s.container.name",
        "region" => "cloud.region",
        other => other,
    }
    .to_string()
}

/// The same sanitization signy and Loki apply to promoted attribute
/// names. VictoriaLogs keeps the dots, so the reduced digest passes every key
/// through this before comparing — `service.name` and `service_name` are the
/// same key or the cross-system check would disagree with itself.
pub fn sanitize_key(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

/// The resource attribute signy reads a tenant from.
///
/// Only signy gets it: Loki and VictoriaLogs still name their tenant in a
/// header, so the bodies are no longer byte-identical across targets. What is
/// stored still is — signy strips this key before storage — so the digests the
/// comparison compares are unaffected.
pub const TENANT_ATTRIBUTE: &str = "tenant.id";

pub fn tenant_attribute(tenant: &str) -> KeyValue {
    string_attribute(TENANT_ATTRIBUTE.to_string(), tenant)
}

/// The service a span's resource claims to be, which is what the trace search
/// filters and groups on. Without it every trace the leg sends is a trace of
/// nothing in particular, and the search probe cannot ask a question a console
/// would ask.
pub fn service_attribute(service: &str) -> KeyValue {
    string_attribute(resource_attribute_name("app"), service)
}

fn string_attribute(key: String, value: &str) -> KeyValue {
    KeyValue {
        key,
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }),
        ..Default::default()
    }
}

/// One `ExportLogsServiceRequest`: a `ResourceLogs` per stream, the stream
/// labels as mapped resource attributes, each entry's structured metadata as
/// record attributes.
///
/// `tenant` is stamped on every resource when the target reads its tenant out
/// of the payload, which is signy and only signy.
pub fn encode_export_logs(batch: &[(Labels, Vec<LogEntry>)], tenant: Option<&str>) -> Vec<u8> {
    let request = ExportLogsServiceRequest {
        resource_logs: batch
            .iter()
            .map(|(labels, entries)| ResourceLogs {
                resource: Some(Resource {
                    attributes: tenant
                        .map(tenant_attribute)
                        .into_iter()
                        .chain(labels.iter().map(|(name, value)| {
                            string_attribute(resource_attribute_name(name), value)
                        }))
                        .collect(),
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    log_records: entries
                        .iter()
                        .map(|entry| LogRecord {
                            time_unix_nano: entry.timestamp_ns.max(0) as u64,
                            body: Some(AnyValue {
                                value: Some(any_value::Value::StringValue(entry.line.clone())),
                            }),
                            attributes: entry
                                .structured_metadata
                                .iter()
                                .map(|(name, value)| string_attribute(name.clone(), value))
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
    request.encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_corpus_label_maps_to_a_promoted_semconv_name() {
        for (label, promoted) in [
            ("app", "service_name"),
            ("env", "deployment_environment"),
            ("cluster", "k8s_cluster_name"),
            ("namespace", "k8s_namespace_name"),
            ("container", "k8s_container_name"),
            ("region", "cloud_region"),
        ] {
            assert_eq!(sanitize_key(&resource_attribute_name(label)), promoted);
        }
        assert_eq!(resource_attribute_name("pod_ip"), "pod_ip");
    }

    #[test]
    fn the_encoded_request_round_trips_streams_entries_and_metadata() {
        let mut labels = Labels::new();
        labels.insert("app".to_string(), "api-gateway".to_string());
        labels.insert("env".to_string(), "prod".to_string());
        let entries = vec![LogEntry {
            timestamp_ns: 1_772_000_000_000_000_000,
            line: "hello".to_string(),
            structured_metadata: vec![("trace_id".to_string(), "abc123".to_string())],
        }];
        let bytes = encode_export_logs(&[(labels, entries)], None);
        let decoded = ExportLogsServiceRequest::decode(bytes.as_slice()).expect("valid protobuf");
        assert_eq!(decoded.resource_logs.len(), 1);
        let resource = decoded.resource_logs[0]
            .resource
            .as_ref()
            .expect("resource");
        let names: Vec<&str> = resource
            .attributes
            .iter()
            .map(|attribute| attribute.key.as_str())
            .collect();
        assert_eq!(names, vec!["service.name", "deployment.environment"]);
        let record = &decoded.resource_logs[0].scope_logs[0].log_records[0];
        assert_eq!(record.time_unix_nano, 1_772_000_000_000_000_000);
        assert_eq!(record.attributes[0].key, "trace_id");
    }
}
