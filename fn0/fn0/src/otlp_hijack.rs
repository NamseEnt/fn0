//! Forwards a guest's OTLP exports to the worker-local collector.
//!
//! The collector stores what it receives without reading it, and the store
//! files every resource under the `tenant.id` it names, one tenant per project.
//! So the worker is the only place that can decide ownership: it decodes each
//! export, overwrites `tenant.id` on every resource with the calling project's
//! id, which the guest cannot choose, and re-encodes it.

use crate::metric_gate::{self, MetricCardinalityGate};
use bytes::Bytes;
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
use opentelemetry_proto::tonic::resource::v1::Resource;
use prost::Message;
use std::sync::Arc;

pub const TENANT_ATTRIBUTE: &str = "tenant.id";
pub const PROTOBUF_CONTENT_TYPE: &str = "application/x-protobuf";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OtlpSignal {
    Traces,
    Metrics,
    Logs,
}

impl OtlpSignal {
    pub fn from_path(path: &str) -> Option<Self> {
        match path {
            "/v1/traces" => Some(Self::Traces),
            "/v1/metrics" => Some(Self::Metrics),
            "/v1/logs" => Some(Self::Logs),
            _ => None,
        }
    }

    pub fn path(self) -> &'static str {
        match self {
            Self::Traces => "/v1/traces",
            Self::Metrics => "/v1/metrics",
            Self::Logs => "/v1/logs",
        }
    }
}

#[derive(Debug)]
pub struct OtlpHijack {
    placeholder_host: String,
    collector_endpoint: String,
    metric_gate: Option<Arc<MetricCardinalityGate>>,
}

impl OtlpHijack {
    pub fn new(placeholder_host: String, collector_endpoint: &str) -> Self {
        Self {
            placeholder_host,
            collector_endpoint: collector_endpoint.trim_end_matches('/').to_string(),
            metric_gate: None,
        }
    }

    pub fn with_metric_gate(mut self, gate: Arc<MetricCardinalityGate>) -> Self {
        self.metric_gate = Some(gate);
        self
    }

    pub fn metric_gate(&self) -> Option<&Arc<MetricCardinalityGate>> {
        self.metric_gate.as_ref()
    }

    pub(crate) fn matches(&self, uri: &hyper::Uri) -> bool {
        uri.host() == Some(self.placeholder_host.as_str())
    }

    pub(crate) fn collector_uri(&self, signal: OtlpSignal) -> String {
        format!("{}{}", self.collector_endpoint, signal.path())
    }

    /// The export to forward, filed under `project_id`'s tenant.
    pub fn stamp_export(
        &self,
        signal: OtlpSignal,
        project_id: &str,
        body: &[u8],
    ) -> Result<Bytes, prost::DecodeError> {
        let encoded = match signal {
            OtlpSignal::Traces => {
                let mut request = ExportTraceServiceRequest::decode(body)?;
                for resource_spans in &mut request.resource_spans {
                    stamp_tenant(&mut resource_spans.resource, project_id);
                }
                request.encode_to_vec()
            }
            OtlpSignal::Logs => {
                let mut request = ExportLogsServiceRequest::decode(body)?;
                for resource_logs in &mut request.resource_logs {
                    stamp_tenant(&mut resource_logs.resource, project_id);
                }
                request.encode_to_vec()
            }
            OtlpSignal::Metrics => {
                let mut request = ExportMetricsServiceRequest::decode(body)?;
                if let Some(gate) = &self.metric_gate {
                    metric_gate::enforce_request(gate, project_id, &mut request);
                }
                for resource_metrics in &mut request.resource_metrics {
                    stamp_tenant(&mut resource_metrics.resource, project_id);
                }
                request.encode_to_vec()
            }
        };
        Ok(Bytes::from(encoded))
    }
}

pub fn stamp_tenant(resource: &mut Option<Resource>, tenant_id: &str) {
    let resource = resource.get_or_insert_with(Resource::default);
    resource
        .attributes
        .retain(|attribute| attribute.key != TENANT_ATTRIBUTE);
    resource.attributes.push(KeyValue {
        key: TENANT_ATTRIBUTE.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(tenant_id.to_string())),
        }),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use opentelemetry_proto::tonic::metrics::v1::{
        Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics, metric,
    };
    use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};

    fn string_attribute(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_string())),
            }),
        }
    }

    fn values_of<'a>(attributes: &'a [KeyValue], key: &str) -> Vec<&'a str> {
        attributes
            .iter()
            .filter(|attribute| attribute.key == key)
            .filter_map(
                |attribute| match attribute.value.as_ref()?.value.as_ref()? {
                    any_value::Value::StringValue(value) => Some(value.as_str()),
                    _ => None,
                },
            )
            .collect()
    }

    fn hijack() -> OtlpHijack {
        OtlpHijack::new("fn0-otel.fn0.dev".to_string(), "http://127.0.0.1:4318/")
    }

    #[test]
    fn files_traces_under_the_calling_project_whatever_tenant_the_guest_named() {
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![
                        string_attribute(TENANT_ATTRIBUTE, "someone-else"),
                        string_attribute("service.name", "app"),
                    ],
                    ..Default::default()
                }),
                scope_spans: vec![ScopeSpans {
                    spans: vec![Span::default()],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let stamped = hijack()
            .stamp_export(OtlpSignal::Traces, "caller", &request.encode_to_vec())
            .expect("valid export");
        let decoded = ExportTraceServiceRequest::decode(stamped.as_ref()).expect("decodes");

        let resource = decoded.resource_spans[0]
            .resource
            .as_ref()
            .expect("resource");
        assert_eq!(
            values_of(&resource.attributes, TENANT_ATTRIBUTE),
            ["caller"]
        );
        assert_eq!(values_of(&resource.attributes, "service.name"), ["app"]);
    }

    #[test]
    fn adds_a_resource_when_the_guest_sent_none() {
        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: None,
                scope_logs: vec![ScopeLogs {
                    log_records: vec![LogRecord::default()],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let stamped = hijack()
            .stamp_export(OtlpSignal::Logs, "caller", &request.encode_to_vec())
            .expect("valid export");
        let decoded = ExportLogsServiceRequest::decode(stamped.as_ref()).expect("decodes");

        let resource = decoded.resource_logs[0]
            .resource
            .as_ref()
            .expect("resource");
        assert_eq!(
            values_of(&resource.attributes, TENANT_ATTRIBUTE),
            ["caller"]
        );
    }

    #[test]
    fn files_metrics_under_the_calling_project() {
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: "requests".to_string(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint::default()],
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let stamped = hijack()
            .stamp_export(OtlpSignal::Metrics, "caller", &request.encode_to_vec())
            .expect("valid export");
        let decoded = ExportMetricsServiceRequest::decode(stamped.as_ref()).expect("decodes");

        let resource = decoded.resource_metrics[0]
            .resource
            .as_ref()
            .expect("resource");
        assert_eq!(
            values_of(&resource.attributes, TENANT_ATTRIBUTE),
            ["caller"]
        );
    }

    #[test]
    fn refuses_an_export_that_is_not_protobuf() {
        assert!(
            hijack()
                .stamp_export(OtlpSignal::Traces, "caller", &[0xff, 0xff, 0xff])
                .is_err()
        );
    }

    #[test]
    fn builds_collector_uri_per_signal() {
        let hijack = hijack();
        assert_eq!(
            hijack.collector_uri(OtlpSignal::Metrics),
            "http://127.0.0.1:4318/v1/metrics"
        );
        assert_eq!(OtlpSignal::from_path("/v1/logs"), Some(OtlpSignal::Logs));
        assert_eq!(OtlpSignal::from_path("/v1/profiles"), None);
    }
}
