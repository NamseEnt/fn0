//! Splits the worker's own telemetry between the platform tenant and the
//! tenants of the projects it describes.
//!
//! One OTel resource covers everything the worker exports, but a request span
//! and the request metrics recorded for a project belong to that project's
//! tenant, so its owner can read them beside the spans and metrics its own app
//! exported. Data carrying [`PROJECT_TENANT_ATTRIBUTE`] is regrouped under a
//! copy of the resource whose `tenant.id` is that project, just before it goes
//! on the wire; the rest keeps the platform tenant.
//!
//! A span only carries the attribute if something set it, and a request span's
//! children are opened by code that does not know the project. So
//! [`ProjectTenantSpanProcessor`] copies the attribute from a request's root span
//! onto every span started in the same trace.

use fn0::otlp_hijack::stamp_tenant;
use fn0::telemetry::PROJECT_TENANT_ATTRIBUTE;
use opentelemetry::trace::{Span as _, SpanId, TraceContextExt, TraceId};
use opentelemetry::{Context, KeyValue, Value};
use opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::common::v1::{KeyValue as ProtoKeyValue, any_value};
use opentelemetry_proto::tonic::logs::v1::{ResourceLogs, ScopeLogs};
use opentelemetry_proto::tonic::metrics::v1::{Metric, ResourceMetrics, ScopeMetrics, metric};
use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans};
use opentelemetry_sdk::error::OTelSdkResult;
use opentelemetry_sdk::trace::{Span, SpanData, SpanProcessor};
use prost::Message;
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
use std::time::Duration;

/// Traces whose root has not ended are few — one per sampled request in
/// flight — so a map this large only means roots that never end. Clearing it
/// costs those traces their children's attribution and nothing else.
const MAX_TRACKED_TRACES: usize = 10_000;

#[derive(Debug, Default)]
pub struct ProjectTenantSpanProcessor {
    project_by_trace: Mutex<HashMap<TraceId, String>>,
}

impl SpanProcessor for ProjectTenantSpanProcessor {
    fn on_start(&self, span: &mut Span, parent_context: &Context) {
        let trace_id = span.span_context().trace_id();
        let parent = parent_context.span();
        let parent_span_context = parent.span_context();
        let starts_a_local_trace =
            !parent_span_context.is_valid() || parent_span_context.is_remote();
        let mut project_by_trace = self
            .project_by_trace
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let project_id = if starts_a_local_trace {
            let project_id = span.exported_data().and_then(|data| {
                data.attributes
                    .iter()
                    .find(|attribute| attribute.key.as_str() == PROJECT_TENANT_ATTRIBUTE)
                    .map(|attribute| attribute.value.as_str().into_owned())
            });
            if let Some(project_id) = &project_id {
                if project_by_trace.len() >= MAX_TRACKED_TRACES {
                    project_by_trace.clear();
                }
                project_by_trace.insert(trace_id, project_id.clone());
            }
            project_id
        } else {
            project_by_trace.get(&trace_id).cloned()
        };
        drop(project_by_trace);
        if let Some(project_id) = project_id {
            span.set_attribute(KeyValue::new(
                PROJECT_TENANT_ATTRIBUTE,
                Value::from(project_id),
            ));
        }
    }

    fn on_end(&self, span: SpanData) {
        if span.parent_span_id == SpanId::INVALID || span.parent_span_is_remote {
            self.project_by_trace
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .remove(&span.span_context.trace_id());
        }
    }

    fn force_flush(&self) -> OTelSdkResult {
        Ok(())
    }

    fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult {
        Ok(())
    }
}

/// The export body to send in place of `body`, or `None` to send it unchanged:
/// a signal this module does not split, or bytes that do not decode.
pub fn route_export(path: &str, body: &[u8], platform_tenant_id: &str) -> Option<Vec<u8>> {
    if path.ends_with("/v1/traces") {
        let request = ExportTraceServiceRequest::decode(body).ok()?;
        return Some(route_traces(request, platform_tenant_id).encode_to_vec());
    }
    if path.ends_with("/v1/metrics") {
        let request = ExportMetricsServiceRequest::decode(body).ok()?;
        return Some(route_metrics(request, platform_tenant_id).encode_to_vec());
    }
    if path.ends_with("/v1/logs") {
        let request = ExportLogsServiceRequest::decode(body).ok()?;
        return Some(route_logs(request, platform_tenant_id).encode_to_vec());
    }
    None
}

/// A guest's own output is the project's log, not the platform's: the lines
/// the worker captures from a guest's stdout and stderr carry the attribute,
/// and everything the worker says about itself does not.
fn route_logs(
    request: ExportLogsServiceRequest,
    platform_tenant_id: &str,
) -> ExportLogsServiceRequest {
    let mut routed = Vec::new();
    for resource_logs in request.resource_logs {
        let mut by_tenant: BTreeMap<String, Vec<ScopeLogs>> = BTreeMap::new();
        for scope_logs in resource_logs.scope_logs {
            let mut records_by_tenant: BTreeMap<String, Vec<_>> = BTreeMap::new();
            for mut record in scope_logs.log_records {
                let tenant_id = take_tenant(&mut record.attributes, platform_tenant_id);
                records_by_tenant
                    .entry(tenant_id)
                    .or_default()
                    .push(record);
            }
            for (tenant_id, log_records) in records_by_tenant {
                by_tenant.entry(tenant_id).or_default().push(ScopeLogs {
                    scope: scope_logs.scope.clone(),
                    log_records,
                    schema_url: scope_logs.schema_url.clone(),
                });
            }
        }
        for (tenant_id, scope_logs) in by_tenant {
            let mut resource = resource_logs.resource.clone();
            stamp_tenant(&mut resource, &tenant_id);
            routed.push(ResourceLogs {
                resource,
                scope_logs,
                schema_url: resource_logs.schema_url.clone(),
            });
        }
    }
    ExportLogsServiceRequest {
        resource_logs: routed,
    }
}

/// Removes the project attribute and says which tenant the item belongs to.
fn take_tenant(attributes: &mut Vec<ProtoKeyValue>, platform_tenant_id: &str) -> String {
    let mut tenant_id = platform_tenant_id.to_string();
    attributes.retain(|attribute| {
        if attribute.key != PROJECT_TENANT_ATTRIBUTE {
            return true;
        }
        if let Some(any_value::Value::StringValue(project_id)) = attribute
            .value
            .as_ref()
            .and_then(|value| value.value.as_ref())
        {
            tenant_id = project_id.clone();
        }
        false
    });
    tenant_id
}

fn route_traces(
    request: ExportTraceServiceRequest,
    platform_tenant_id: &str,
) -> ExportTraceServiceRequest {
    let mut routed = Vec::new();
    for resource_spans in request.resource_spans {
        let mut by_tenant: BTreeMap<String, Vec<ScopeSpans>> = BTreeMap::new();
        for scope_spans in resource_spans.scope_spans {
            let mut spans_by_tenant: BTreeMap<String, Vec<_>> = BTreeMap::new();
            for mut span in scope_spans.spans {
                let tenant_id = take_tenant(&mut span.attributes, platform_tenant_id);
                spans_by_tenant.entry(tenant_id).or_default().push(span);
            }
            for (tenant_id, spans) in spans_by_tenant {
                by_tenant.entry(tenant_id).or_default().push(ScopeSpans {
                    scope: scope_spans.scope.clone(),
                    spans,
                    schema_url: scope_spans.schema_url.clone(),
                });
            }
        }
        for (tenant_id, scope_spans) in by_tenant {
            let mut resource = resource_spans.resource.clone();
            stamp_tenant(&mut resource, &tenant_id);
            routed.push(ResourceSpans {
                resource,
                scope_spans,
                schema_url: resource_spans.schema_url.clone(),
            });
        }
    }
    ExportTraceServiceRequest {
        resource_spans: routed,
    }
}

/// Splits one metric into one copy per tenant, each holding only that
/// tenant's data points.
fn split_metric(metric: Metric, platform_tenant_id: &str) -> BTreeMap<String, Metric> {
    macro_rules! split_points {
        ($data:expr, $variant:ident) => {{
            let mut points_by_tenant = BTreeMap::new();
            for mut point in std::mem::take(&mut $data.data_points) {
                let tenant_id = take_tenant(&mut point.attributes, platform_tenant_id);
                points_by_tenant
                    .entry(tenant_id)
                    .or_insert_with(Vec::new)
                    .push(point);
            }
            points_by_tenant
                .into_iter()
                .map(|(tenant_id, data_points)| {
                    let mut data = $data.clone();
                    data.data_points = data_points;
                    (
                        tenant_id,
                        Metric {
                            data: Some(metric::Data::$variant(data)),
                            ..metric_without_data(&metric)
                        },
                    )
                })
                .collect()
        }};
    }
    match metric.data.clone() {
        Some(metric::Data::Gauge(mut data)) => split_points!(data, Gauge),
        Some(metric::Data::Sum(mut data)) => split_points!(data, Sum),
        Some(metric::Data::Histogram(mut data)) => split_points!(data, Histogram),
        Some(metric::Data::ExponentialHistogram(mut data)) => {
            split_points!(data, ExponentialHistogram)
        }
        Some(metric::Data::Summary(mut data)) => split_points!(data, Summary),
        None => BTreeMap::from([(platform_tenant_id.to_string(), metric)]),
    }
}

fn metric_without_data(metric: &Metric) -> Metric {
    Metric {
        name: metric.name.clone(),
        description: metric.description.clone(),
        unit: metric.unit.clone(),
        metadata: metric.metadata.clone(),
        data: None,
    }
}

fn route_metrics(
    request: ExportMetricsServiceRequest,
    platform_tenant_id: &str,
) -> ExportMetricsServiceRequest {
    let mut routed = Vec::new();
    for resource_metrics in request.resource_metrics {
        let mut by_tenant: BTreeMap<String, Vec<ScopeMetrics>> = BTreeMap::new();
        for scope_metrics in resource_metrics.scope_metrics {
            let mut metrics_by_tenant: BTreeMap<String, Vec<Metric>> = BTreeMap::new();
            for metric in scope_metrics.metrics {
                for (tenant_id, metric) in split_metric(metric, platform_tenant_id) {
                    metrics_by_tenant.entry(tenant_id).or_default().push(metric);
                }
            }
            for (tenant_id, metrics) in metrics_by_tenant {
                by_tenant.entry(tenant_id).or_default().push(ScopeMetrics {
                    scope: scope_metrics.scope.clone(),
                    metrics,
                    schema_url: scope_metrics.schema_url.clone(),
                });
            }
        }
        for (tenant_id, scope_metrics) in by_tenant {
            let mut resource = resource_metrics.resource.clone();
            stamp_tenant(&mut resource, &tenant_id);
            routed.push(ResourceMetrics {
                resource,
                scope_metrics,
                schema_url: resource_metrics.schema_url.clone(),
            });
        }
    }
    ExportMetricsServiceRequest {
        resource_metrics: routed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fn0::otlp_hijack::TENANT_ATTRIBUTE;
    use opentelemetry::trace::{Tracer, TracerProvider as _};
    use opentelemetry_proto::tonic::common::v1::AnyValue;
    use opentelemetry_proto::tonic::metrics::v1::{Histogram, HistogramDataPoint};
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use opentelemetry_proto::tonic::trace::v1::Span as ProtoSpan;
    use opentelemetry_sdk::trace::{InMemorySpanExporter, SdkTracerProvider};

    fn string_attribute(key: &str, value: &str) -> ProtoKeyValue {
        ProtoKeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_string())),
            }),
        }
    }

    fn tenant_of(resource: &Option<Resource>) -> &str {
        resource
            .as_ref()
            .and_then(|resource| {
                resource
                    .attributes
                    .iter()
                    .find(|attribute| attribute.key == TENANT_ATTRIBUTE)
            })
            .and_then(|attribute| attribute.value.as_ref()?.value.as_ref())
            .and_then(|value| match value {
                any_value::Value::StringValue(tenant_id) => Some(tenant_id.as_str()),
                _ => None,
            })
            .expect("a tenant")
    }

    fn platform_resource() -> Option<Resource> {
        Some(Resource {
            attributes: vec![
                string_attribute(TENANT_ATTRIBUTE, "fn0"),
                string_attribute("service.name", "fn0-worker"),
            ],
            ..Default::default()
        })
    }

    #[test]
    fn spans_marked_with_a_project_move_to_its_tenant_and_lose_the_mark() {
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: platform_resource(),
                scope_spans: vec![ScopeSpans {
                    spans: vec![
                        ProtoSpan {
                            name: "fn0.request".to_string(),
                            attributes: vec![string_attribute(
                                PROJECT_TENANT_ATTRIBUTE,
                                "project-a",
                            )],
                            ..Default::default()
                        },
                        ProtoSpan {
                            name: "unmarked".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let routed = route_traces(request, "fn0");

        let mut tenants: Vec<(&str, Vec<&str>)> = routed
            .resource_spans
            .iter()
            .map(|resource_spans| {
                (
                    tenant_of(&resource_spans.resource),
                    resource_spans.scope_spans[0]
                        .spans
                        .iter()
                        .map(|span| span.name.as_str())
                        .collect(),
                )
            })
            .collect();
        tenants.sort();
        assert_eq!(
            tenants,
            vec![
                ("fn0", vec!["unmarked"]),
                ("project-a", vec!["fn0.request"])
            ]
        );
        for resource_spans in &routed.resource_spans {
            for span in &resource_spans.scope_spans[0].spans {
                assert!(
                    span.attributes
                        .iter()
                        .all(|attribute| attribute.key != PROJECT_TENANT_ATTRIBUTE)
                );
            }
        }
    }

    /// A guest's captured output belongs to the project that printed it; the
    /// worker's own lines stay with the platform, and neither carries the mark
    /// onward.
    #[test]
    fn guest_output_moves_to_the_project_tenant_and_the_workers_own_lines_do_not() {
        use opentelemetry_proto::tonic::logs::v1::LogRecord;

        let request = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: platform_resource(),
                scope_logs: vec![ScopeLogs {
                    log_records: vec![
                        LogRecord {
                            severity_text: "INFO".to_string(),
                            attributes: vec![
                                string_attribute(PROJECT_TENANT_ATTRIBUTE, "project-a"),
                                string_attribute("stream", "stderr"),
                            ],
                            ..Default::default()
                        },
                        LogRecord {
                            severity_text: "ERROR".to_string(),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let routed = route_logs(request, "fn0");

        let mut tenants: Vec<(&str, Vec<&str>)> = routed
            .resource_logs
            .iter()
            .map(|resource_logs| {
                (
                    tenant_of(&resource_logs.resource),
                    resource_logs.scope_logs[0]
                        .log_records
                        .iter()
                        .map(|record| record.severity_text.as_str())
                        .collect(),
                )
            })
            .collect();
        tenants.sort();
        assert_eq!(
            tenants,
            vec![("fn0", vec!["ERROR"]), ("project-a", vec!["INFO"])]
        );
        let guest_record = &routed
            .resource_logs
            .iter()
            .find(|resource_logs| tenant_of(&resource_logs.resource) == "project-a")
            .expect("the project's own resource")
            .scope_logs[0]
            .log_records[0];
        assert!(
            guest_record
                .attributes
                .iter()
                .all(|attribute| attribute.key != PROJECT_TENANT_ATTRIBUTE)
        );
        assert!(
            guest_record
                .attributes
                .iter()
                .any(|attribute| attribute.key == "stream")
        );
    }

    #[test]
    fn one_metric_splits_into_one_copy_per_tenant() {
        let point = |project_id: Option<&str>| HistogramDataPoint {
            attributes: project_id
                .map(|project_id| vec![string_attribute(PROJECT_TENANT_ATTRIBUTE, project_id)])
                .unwrap_or_default(),
            count: 1,
            ..Default::default()
        };
        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: platform_resource(),
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: fn0::telemetry::REQUEST_DURATION_METRIC.to_string(),
                        unit: "s".to_string(),
                        data: Some(metric::Data::Histogram(Histogram {
                            data_points: vec![
                                point(Some("project-a")),
                                point(Some("project-b")),
                                point(Some("project-a")),
                                point(None),
                            ],
                            aggregation_temporality: 2,
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let routed = route_metrics(request, "fn0");

        let mut points_per_tenant: Vec<(&str, usize)> = routed
            .resource_metrics
            .iter()
            .map(|resource_metrics| {
                let metric = &resource_metrics.scope_metrics[0].metrics[0];
                assert_eq!(metric.name, fn0::telemetry::REQUEST_DURATION_METRIC);
                let Some(metric::Data::Histogram(histogram)) = &metric.data else {
                    panic!("expected a histogram");
                };
                assert_eq!(histogram.aggregation_temporality, 2);
                (
                    tenant_of(&resource_metrics.resource),
                    histogram.data_points.len(),
                )
            })
            .collect();
        points_per_tenant.sort();
        assert_eq!(
            points_per_tenant,
            vec![("fn0", 1), ("project-a", 2), ("project-b", 1)]
        );
    }

    #[test]
    fn children_of_a_request_span_inherit_its_project() {
        let exporter = InMemorySpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_span_processor(ProjectTenantSpanProcessor::default())
            .with_simple_exporter(exporter.clone())
            .build();
        let tracer = provider.tracer("test");

        let root = tracer
            .span_builder("fn0.request")
            .with_attributes([KeyValue::new(PROJECT_TENANT_ATTRIBUTE, "project-a")])
            .start(&tracer);
        let root_context = Context::current_with_span(root);
        let child = tracer.start_with_context("get_impl", &root_context);
        drop(child);
        drop(root_context);
        let unrelated = tracer.start("manifest_poll");
        drop(unrelated);

        let project_of = |name: &str| -> Option<String> {
            exporter
                .get_finished_spans()
                .unwrap()
                .into_iter()
                .find(|span| span.name == name)
                .and_then(|span| {
                    span.attributes
                        .into_iter()
                        .find(|attribute| attribute.key.as_str() == PROJECT_TENANT_ATTRIBUTE)
                        .map(|attribute| attribute.value.as_str().into_owned())
                })
        };
        assert_eq!(project_of("fn0.request").as_deref(), Some("project-a"));
        assert_eq!(project_of("get_impl").as_deref(), Some("project-a"));
        assert_eq!(project_of("manifest_poll"), None);
    }
}
