//! Per-project OTLP metrics cardinality gate, attached to the OTLP hijack by
//! the worker (absent in `forte dev`).
//!
//! The gate caps how many distinct series a project may carry so one
//! cardinality explosion (e.g. an unbounded label value) cannot monopolise the
//! shared metrics node, whose RAM is what every project's series compete for.
//! It mirrors [`crate::presign_gate::PresignGate`]: worker-local state,
//! enforced inline on the telemetry path.
//!
//! Self-hosting is what makes this gate load-bearing rather than an early
//! warning. A purchased allowance has a bill at the end of it; a node that
//! runs out of memory takes every other project's metrics down with it, and
//! there is nothing behind this to refuse the write.
//!
//! Semantics are "keep existing, drop new": a series admitted once stays
//! admitted while it keeps reporting, and only newly appearing series are
//! refused once a cap is hit. Series go stale after [`STALE_WINDOW_SECS`]
//! without a sample (matching Alloy's `deltatocumulative.max_stale`), freeing
//! their slot, so the tracked set reflects *active* series the way the
//! backend counts them, not every series ever seen.

use bytes::Bytes;
use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
use opentelemetry_proto::tonic::metrics::v1::{
    ExponentialHistogramDataPoint, HistogramDataPoint, Metric, NumberDataPoint, ResourceMetrics,
    ScopeMetrics, Sum, SummaryDataPoint, metric, number_data_point,
};
use prost::Message;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub const METRIC_ACTIVE_SERIES_PER_PROJECT: usize = 1_000;
pub const METRIC_NAMES_PER_PROJECT: usize = 100;
pub const METRIC_LABEL_VALUES_PER_KEY: usize = 100;

const STALE_WINDOW_SECS: i64 = 300;
pub const DROPPED_METRIC_NAME: &str = "fn0.metrics.dropped";

#[derive(Debug, Default)]
pub struct MetricCardinalityGate {
    state: Mutex<HashMap<String, ProjectState>>,
}

#[derive(Debug, Default)]
struct ProjectState {
    series: HashMap<String, i64>,
    names: HashMap<String, i64>,
    label_values: HashMap<String, HashMap<String, i64>>,
}

impl ProjectState {
    fn evict(&mut self, now: i64) {
        let stale = |t: &i64| now - *t > STALE_WINDOW_SECS;
        self.series.retain(|_, t| !stale(t));
        self.names.retain(|_, t| !stale(t));
        for values in self.label_values.values_mut() {
            values.retain(|_, t| !stale(t));
        }
        self.label_values.retain(|_, values| !values.is_empty());
    }

    fn admit(&mut self, name: &str, pairs: &[(String, String)], now: i64) -> bool {
        let key = series_key(name, pairs);
        let existing = self.series.contains_key(&key);
        let allowed = existing
            || (self.series.len() < METRIC_ACTIVE_SERIES_PER_PROJECT
                && (self.names.contains_key(name) || self.names.len() < METRIC_NAMES_PER_PROJECT)
                && pairs.iter().all(|(k, v)| {
                    self.label_values.get(k).is_none_or(|values| {
                        values.contains_key(v) || values.len() < METRIC_LABEL_VALUES_PER_KEY
                    })
                }));
        if allowed {
            self.series.insert(key, now);
            self.names.insert(name.to_string(), now);
            for (k, v) in pairs {
                self.label_values
                    .entry(k.clone())
                    .or_default()
                    .insert(v.clone(), now);
            }
        }
        allowed
    }
}

trait HasAttributes {
    fn attributes(&self) -> &[KeyValue];
}

impl HasAttributes for NumberDataPoint {
    fn attributes(&self) -> &[KeyValue] {
        &self.attributes
    }
}

impl HasAttributes for HistogramDataPoint {
    fn attributes(&self) -> &[KeyValue] {
        &self.attributes
    }
}

impl HasAttributes for ExponentialHistogramDataPoint {
    fn attributes(&self) -> &[KeyValue] {
        &self.attributes
    }
}

impl HasAttributes for SummaryDataPoint {
    fn attributes(&self) -> &[KeyValue] {
        &self.attributes
    }
}

fn filter_points<T: HasAttributes>(
    state: &mut ProjectState,
    name: &str,
    points: &mut Vec<T>,
    now: i64,
) -> u64 {
    let before = points.len();
    points.retain(|point| {
        let pairs = label_pairs(point.attributes());
        state.admit(name, &pairs, now)
    });
    (before - points.len()) as u64
}

/// Returns whether the metric still has data — a metric whose `data` is `None`
/// carries only metadata and is kept — and how many points were dropped.
fn filter_metric(state: &mut ProjectState, metric: &mut Metric, now: i64) -> (bool, u64) {
    let name = metric.name.clone();
    let dropped = match &mut metric.data {
        Some(metric::Data::Gauge(g)) => filter_points(state, &name, &mut g.data_points, now),
        Some(metric::Data::Sum(s)) => filter_points(state, &name, &mut s.data_points, now),
        Some(metric::Data::Histogram(h)) => filter_points(state, &name, &mut h.data_points, now),
        Some(metric::Data::ExponentialHistogram(e)) => {
            filter_points(state, &name, &mut e.data_points, now)
        }
        Some(metric::Data::Summary(s)) => filter_points(state, &name, &mut s.data_points, now),
        None => 0,
    };
    let has_data = match &metric.data {
        Some(metric::Data::Gauge(g)) => !g.data_points.is_empty(),
        Some(metric::Data::Sum(s)) => !s.data_points.is_empty(),
        Some(metric::Data::Histogram(h)) => !h.data_points.is_empty(),
        Some(metric::Data::ExponentialHistogram(e)) => !e.data_points.is_empty(),
        Some(metric::Data::Summary(s)) => !s.data_points.is_empty(),
        None => true,
    };
    (has_data, dropped)
}

fn enforce_project(
    state: &mut ProjectState,
    request: &mut ExportMetricsServiceRequest,
    now: i64,
) -> u64 {
    let mut dropped = 0;
    for resource in &mut request.resource_metrics {
        for scope in &mut resource.scope_metrics {
            scope.metrics.retain_mut(|metric| {
                let (has_data, dropped_here) = filter_metric(state, metric, now);
                dropped += dropped_here;
                has_data
            });
        }
        resource
            .scope_metrics
            .retain(|scope| !scope.metrics.is_empty());
    }
    request
        .resource_metrics
        .retain(|resource| !resource.scope_metrics.is_empty());
    dropped
}

fn series_key(name: &str, pairs: &[(String, String)]) -> String {
    let mut key = String::with_capacity(name.len() + pairs.len() * 16);
    key.push_str(name);
    key.push('\x1f');
    for (k, v) in pairs {
        key.push_str(k);
        key.push('=');
        key.push_str(v);
        key.push('\x1e');
    }
    key
}

fn label_pairs(attributes: &[KeyValue]) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = attributes
        .iter()
        .map(|kv| (kv.key.clone(), any_value_to_key(&kv.value)))
        .collect();
    pairs.sort();
    pairs
}

fn any_value_to_key(value: &Option<AnyValue>) -> String {
    match value.as_ref().and_then(|v| v.value.as_ref()) {
        Some(any_value::Value::StringValue(s)) => s.clone(),
        Some(any_value::Value::BoolValue(b)) => b.to_string(),
        Some(any_value::Value::IntValue(i)) => i.to_string(),
        Some(any_value::Value::DoubleValue(d)) => d.to_string(),
        Some(any_value::Value::BytesValue(bytes)) => format!("bytes:{}", bytes.len()),
        Some(_) => "nested".to_string(),
        None => String::new(),
    }
}

/// Appends the drop count to the project's own payload so Alloy stamps
/// `fn0.project_id` on it from the trusted hijack header, making the
/// throttling visible to the project owner rather than only to operators.
pub fn inject_dropped_metric(
    request: &mut ExportMetricsServiceRequest,
    dropped: u64,
    now_nanos: u64,
) {
    let point = NumberDataPoint {
        time_unix_nano: now_nanos,
        value: Some(number_data_point::Value::AsInt(dropped as i64)),
        ..Default::default()
    };
    let metric = Metric {
        name: DROPPED_METRIC_NAME.to_string(),
        unit: "1".to_string(),
        data: Some(metric::Data::Sum(Sum {
            data_points: vec![point],
            aggregation_temporality: 1,
            is_monotonic: true,
        })),
        ..Default::default()
    };
    if request.resource_metrics.is_empty() {
        request.resource_metrics.push(ResourceMetrics::default());
    }
    let resource = &mut request.resource_metrics[0];
    if resource.scope_metrics.is_empty() {
        resource.scope_metrics.push(ScopeMetrics::default());
    }
    resource.scope_metrics[0].metrics.push(metric);
}

/// Enforces the per-project caps on a raw OTLP metrics payload, returning the
/// bytes to forward. Undecodable payloads pass through untouched rather than
/// failing the export: the gate is a cost guard, not a validator, and the
/// collector is entitled to reject what it cannot parse.
pub fn enforce_request_bytes(
    gate: &MetricCardinalityGate,
    project_id: &str,
    bytes: Bytes,
) -> Bytes {
    let Ok(mut request) = ExportMetricsServiceRequest::decode(bytes.as_ref()) else {
        return bytes;
    };
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let dropped = gate.enforce(project_id, &mut request, now_secs);
    if dropped == 0 {
        return bytes;
    }
    tracing::warn!(
        project_id,
        dropped,
        "otlp metrics dropped by cardinality cap"
    );
    inject_dropped_metric(
        &mut request,
        dropped,
        (now_secs as u64).saturating_mul(1_000_000_000),
    );
    let mut buf = Vec::with_capacity(request.encoded_len());
    if request.encode(&mut buf).is_err() {
        return bytes;
    }
    Bytes::from(buf)
}

impl MetricCardinalityGate {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of data points dropped.
    pub fn enforce(
        &self,
        project_id: &str,
        request: &mut ExportMetricsServiceRequest,
        now_secs: i64,
    ) -> u64 {
        let mut state = self.state.lock().unwrap();
        let project = state.entry(project_id.to_string()).or_default();
        project.evict(now_secs);
        enforce_project(project, request, now_secs)
    }

    /// Per-project active-series counts, stale series evicted first.
    pub fn snapshot(&self, now_secs: i64) -> Vec<(String, usize)> {
        let mut state = self.state.lock().unwrap();
        state.retain(|_, project| {
            project.evict(now_secs);
            !project.series.is_empty()
        });
        state
            .iter()
            .map(|(project_id, project)| (project_id.clone(), project.series.len()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::metrics::v1::Gauge;

    fn point_with(attr: &str, value: &str) -> NumberDataPoint {
        NumberDataPoint {
            attributes: vec![KeyValue {
                key: attr.to_string(),
                value: Some(AnyValue {
                    value: Some(any_value::Value::StringValue(value.to_string())),
                }),
            }],
            ..Default::default()
        }
    }

    // Two labels whose per-key value counts stay under the label-value cap, so
    // distinct (a, b) combinations exercise the *series* cap in isolation.
    fn point_combo(i: usize) -> NumberDataPoint {
        let attr = |key: &str, v: usize| KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(v.to_string())),
            }),
        };
        NumberDataPoint {
            attributes: vec![attr("a", i / 40), attr("b", i % 40)],
            ..Default::default()
        }
    }

    fn gauge_request(name: &str, points: Vec<NumberDataPoint>) -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                scope_metrics: vec![ScopeMetrics {
                    metrics: vec![Metric {
                        name: name.to_string(),
                        data: Some(metric::Data::Gauge(Gauge {
                            data_points: points,
                        })),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn count_points(request: &ExportMetricsServiceRequest) -> usize {
        request
            .resource_metrics
            .iter()
            .flat_map(|r| &r.scope_metrics)
            .flat_map(|s| &s.metrics)
            .map(|m| match &m.data {
                Some(metric::Data::Gauge(g)) => g.data_points.len(),
                _ => 0,
            })
            .sum()
    }

    #[test]
    fn keeps_existing_drops_new_over_cap() {
        let gate = MetricCardinalityGate::new();
        let now = 1_000;

        let mut request = gauge_request(
            "m",
            (0..METRIC_ACTIVE_SERIES_PER_PROJECT + 5)
                .map(point_combo)
                .collect(),
        );
        let dropped = gate.enforce("p", &mut request, now);
        assert_eq!(dropped, 5);
        assert_eq!(count_points(&request), METRIC_ACTIVE_SERIES_PER_PROJECT);

        let mut again = gauge_request("m", vec![point_combo(0)]);
        assert_eq!(gate.enforce("p", &mut again, now + 1), 0);
        assert_eq!(count_points(&again), 1);

        let mut overflow = gauge_request("m", vec![point_combo(1599)]);
        assert_eq!(gate.enforce("p", &mut overflow, now + 1), 1);
        assert_eq!(count_points(&overflow), 0);
    }

    #[test]
    fn stale_series_free_slot() {
        let gate = MetricCardinalityGate::new();
        let now = 10_000;
        let mut request = gauge_request(
            "m",
            (0..METRIC_ACTIVE_SERIES_PER_PROJECT)
                .map(point_combo)
                .collect(),
        );
        gate.enforce("p", &mut request, now);

        let later = now + STALE_WINDOW_SECS + 1;
        let mut fresh = gauge_request("m", vec![point_combo(1599)]);
        assert_eq!(gate.enforce("p", &mut fresh, later), 0);
        assert_eq!(count_points(&fresh), 1);
    }

    #[test]
    fn label_value_cap_blocks_new_values_only() {
        let gate = MetricCardinalityGate::new();
        let now = 1_000;
        let mut request = gauge_request(
            "m",
            (0..METRIC_LABEL_VALUES_PER_KEY + 3)
                .map(|i| point_with("user", &format!("u{i}")))
                .collect(),
        );
        let dropped = gate.enforce("p", &mut request, now);
        assert_eq!(dropped, 3);
        assert_eq!(count_points(&request), METRIC_LABEL_VALUES_PER_KEY);
    }

    #[test]
    fn name_cap_blocks_new_names() {
        let gate = MetricCardinalityGate::new();
        let now = 1_000;
        for i in 0..METRIC_NAMES_PER_PROJECT {
            let mut request = gauge_request(&format!("metric{i}"), vec![point_with("k", "v")]);
            assert_eq!(gate.enforce("p", &mut request, now), 0);
        }
        // The name cap counts names, not series: an existing name still admits
        // new series after the cap is reached.
        let mut new_name = gauge_request("one_too_many", vec![point_with("k", "v")]);
        assert_eq!(gate.enforce("p", &mut new_name, now), 1);
        let mut existing_name = gauge_request("metric0", vec![point_with("k", "v2")]);
        assert_eq!(gate.enforce("p", &mut existing_name, now), 0);
    }

    #[test]
    fn dropped_metric_injected() {
        let mut request = ExportMetricsServiceRequest::default();
        inject_dropped_metric(&mut request, 7, 123);
        let metric = &request.resource_metrics[0].scope_metrics[0].metrics[0];
        assert_eq!(metric.name, DROPPED_METRIC_NAME);
        match &metric.data {
            Some(metric::Data::Sum(sum)) => {
                assert_eq!(sum.data_points.len(), 1);
                assert!(matches!(
                    sum.data_points[0].value,
                    Some(number_data_point::Value::AsInt(7))
                ));
            }
            other => panic!("expected sum, got {other:?}"),
        }
    }

    #[test]
    fn enforce_request_bytes_roundtrip() {
        let gate = MetricCardinalityGate::new();
        let request = gauge_request(
            "m",
            (0..METRIC_ACTIVE_SERIES_PER_PROJECT + 10)
                .map(point_combo)
                .collect(),
        );
        let mut buf = Vec::new();
        request.encode(&mut buf).unwrap();

        let out = enforce_request_bytes(&gate, "p", Bytes::from(buf));
        let decoded = ExportMetricsServiceRequest::decode(out.as_ref()).unwrap();

        let total_points: usize = decoded
            .resource_metrics
            .iter()
            .flat_map(|r| &r.scope_metrics)
            .flat_map(|s| &s.metrics)
            .map(|m| match &m.data {
                Some(metric::Data::Gauge(g)) => g.data_points.len(),
                Some(metric::Data::Sum(s)) => s.data_points.len(),
                _ => 0,
            })
            .sum();
        // Capped series plus the injected dropped metric.
        assert_eq!(total_points, METRIC_ACTIVE_SERIES_PER_PROJECT + 1);

        let dropped = decoded
            .resource_metrics
            .iter()
            .flat_map(|r| &r.scope_metrics)
            .flat_map(|s| &s.metrics)
            .find(|m| m.name == DROPPED_METRIC_NAME)
            .expect("dropped metric present");
        match &dropped.data {
            Some(metric::Data::Sum(sum)) => assert!(matches!(
                sum.data_points[0].value,
                Some(number_data_point::Value::AsInt(10))
            )),
            other => panic!("expected sum, got {other:?}"),
        }
    }
}
