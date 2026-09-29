//! OTLP metrics ingest: the decomposition of the five OTLP metric types into
//! float series, and the transport-independent admission contract (M14,
//! issue #8).
//!
//! [`normalize_request`] is deliberately a **pure function of the request** —
//! no clock, no config, no per-series state — because the journal stores the
//! raw `ExportMetricsServiceRequest` bytes (kind 3) and replay runs this same
//! function over them. Changing the decomposition changes what old journals
//! replay into; the journal is unversioned by policy, and the
//! replay-equals-live test pins the coupling. The one stateful step — folding
//! delta-temporality increments into running totals — happens at memtable
//! insert (`series::SeriesMemTable::insert`), which replay drives in the same
//! append order live ingest did, so the totals reproduce.

use opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsServiceRequest;
use opentelemetry_proto::tonic::metrics::v1::{
    AggregationTemporality, ExponentialHistogramDataPoint, HistogramDataPoint, NumberDataPoint,
    metric, number_data_point,
};
use prost014::Message;

use crate::backpressure::IngestError;
use crate::config::Config;
use crate::journal::Journal;
use crate::otlp_log::normalize_attribute_key;
use crate::series::{
    HistogramPoint, METRIC_NAME_LABEL, MetricSample, MetricValue, SampleKind, SeriesLabels,
};
use crate::tenant::TenantId;
use crate::trace_ingest::MAX_OTLP_REQUEST_BYTES;
use axum::http::StatusCode;

/// Decomposed-sample cap per request. Counted **after** decomposition, since
/// a histogram datapoint is one sample whatever its resolution, because the
/// instrument is stored whole — the cap bounds what the engine actually
/// stores, not what the wire carried. The same class of constant as
/// `MAX_OTLP_SPANS`.
pub const MAX_OTLP_METRIC_SAMPLES: usize = 100_000;

/// Exponential histograms are downscaled until at most this many finite
/// bucket boundaries remain (plus `+Inf`). The loss is boundary-limited
/// quantile precision — the decision record in
/// `docs/M14_IMPLEMENTATION_PLAN.md` §3. It no longer bounds cardinality:
/// the boundaries are a schema inside one series rather than a series each.
pub const MAX_EXP_HISTOGRAM_BUCKETS: usize = 64;

/// Resource attributes promoted to series labels, keys normalized by
/// [`normalize_attribute_key`]. One schema decision, small on purpose: series
/// identity must not inherit every resource attribute a collector attaches
/// (a per-process `process.pid` would mint a series per restart), so only the
/// names a dashboard actually groups by are promoted and the rest are
/// dropped. A datapoint attribute with the same normalized key wins.
pub const PROMOTED_RESOURCE_ATTRIBUTES: [&str; 8] = [
    "service.name",
    "service.namespace",
    "service.instance.id",
    "deployment.environment",
    "k8s.cluster.name",
    "k8s.namespace.name",
    "k8s.pod.name",
    "k8s.container.name",
];

#[derive(Debug, PartialEq, Eq)]
pub enum MetricIngestError {
    EmptyRequest,
    TimestampOutOfRange,
    TooManySamples,
}

impl std::fmt::Display for MetricIngestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::EmptyRequest => "OTLP metrics request contains no datapoints",
            Self::TimestampOutOfRange => {
                "metric datapoint timestamp is outside the supported range"
            }
            Self::TooManySamples => "OTLP metrics request decomposes into too many samples",
        };
        f.write_str(message)
    }
}

impl std::error::Error for MetricIngestError {}

/// The `le`/`quantile` label rendering: Rust's shortest round-trip float
/// form, which prints `0.005` as `0.005` and infinity as `+Inf` the way the
/// Prometheus ecosystem spells it. Pinned by test because two renderings of
/// one boundary would be two series.
pub fn format_boundary(value: f64) -> String {
    if value == f64::INFINITY {
        return "+Inf".to_string();
    }
    format!("{value}")
}

struct Decomposition {
    samples: Vec<MetricSample>,
    /// The next datapoint's index in the request's deterministic traversal
    /// order — the same order [`filter_request`] walks, which is what lets an
    /// admission verdict made over samples drop the right datapoints from the
    /// WAL bytes.
    next_datapoint: u32,
    datapoint: u32,
}

impl Decomposition {
    /// Every datapoint consumes exactly one index, whether or not it produces
    /// samples, so the two traversals cannot drift on a skipped point.
    fn begin_datapoint(&mut self) {
        self.datapoint = self.next_datapoint;
        self.next_datapoint += 1;
    }

    fn push(
        &mut self,
        tenant: &TenantId,
        labels: SeriesLabels,
        ts_ns: i64,
        value: f64,
        kind: SampleKind,
    ) -> Result<(), MetricIngestError> {
        self.push_value(tenant, labels, ts_ns, MetricValue::Scalar(value), kind)
    }

    fn push_value(
        &mut self,
        tenant: &TenantId,
        labels: SeriesLabels,
        ts_ns: i64,
        value: MetricValue,
        kind: SampleKind,
    ) -> Result<(), MetricIngestError> {
        if self.samples.len() >= MAX_OTLP_METRIC_SAMPLES {
            return Err(MetricIngestError::TooManySamples);
        }
        self.samples.push(MetricSample {
            tenant: tenant.clone(),
            labels,
            ts_ns,
            value,
            kind,
            datapoint_index: self.datapoint,
        });
        Ok(())
    }
}

/// Base pairs for a datapoint: `__name__` first-pushed (wins any collision),
/// then the datapoint attributes, then the promoted resource attributes —
/// `SeriesLabels::from_pairs` keeps the first occurrence of a key, so this
/// push order is the precedence order.
fn base_pairs(
    name: &str,
    attributes: &[opentelemetry_proto::tonic::common::v1::KeyValue],
    promoted: &[(String, String)],
) -> Vec<(String, String)> {
    let mut pairs = Vec::with_capacity(2 + attributes.len() + promoted.len());
    pairs.push((METRIC_NAME_LABEL.to_string(), name.to_string()));
    for attribute in attributes {
        if let Some(value) = &attribute.value {
            pairs.push((
                normalize_attribute_key(&attribute.key),
                attribute_display(value),
            ));
        }
    }
    pairs.extend_from_slice(promoted);
    pairs
}

fn attribute_display(value: &opentelemetry_proto::tonic::common::v1::AnyValue) -> String {
    use opentelemetry_proto::tonic::common::v1::any_value::Value;
    match &value.value {
        Some(Value::StringValue(text)) => text.clone(),
        Some(Value::BoolValue(value)) => value.to_string(),
        Some(Value::IntValue(value)) => value.to_string(),
        Some(Value::DoubleValue(value)) => value.to_string(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn with_extra(base: &[(String, String)], key: &str, value: String) -> SeriesLabels {
    let mut pairs = Vec::with_capacity(base.len() + 1);
    pairs.push((key.to_string(), value));
    pairs.extend_from_slice(base);
    SeriesLabels::from_pairs(pairs)
}

fn datapoint_ts(time_unix_nano: u64) -> Result<i64, MetricIngestError> {
    i64::try_from(time_unix_nano).map_err(|_| MetricIngestError::TimestampOutOfRange)
}

fn number_value(point: &NumberDataPoint) -> Option<f64> {
    match point.value {
        Some(number_data_point::Value::AsDouble(value)) => Some(value),
        Some(number_data_point::Value::AsInt(value)) => Some(value as f64),
        // A datapoint with no value carries nothing to store; OTLP permits it
        // and dropping it loses nothing a query could have read.
        None => None,
    }
}

/// Delta temporality accumulates at insert; everything else — cumulative and
/// the spec's "unspecified" — is stored as sent.
fn temporality_kind(temporality: i32) -> SampleKind {
    if temporality == AggregationTemporality::Delta as i32 {
        SampleKind::Delta
    } else {
        SampleKind::Cumulative
    }
}

/// Turn one OTLP export into float samples, per the decomposition in
/// `docs/M14_IMPLEMENTATION_PLAN.md` §3. Pure — see the module doc.
pub fn normalize_request(
    tenant: &TenantId,
    request: &ExportMetricsServiceRequest,
) -> Result<Vec<MetricSample>, MetricIngestError> {
    let mut out = Decomposition {
        samples: Vec::new(),
        next_datapoint: 0,
        datapoint: 0,
    };
    for resource_metrics in &request.resource_metrics {
        let promoted: Vec<(String, String)> = resource_metrics
            .resource
            .as_ref()
            .map(|resource| {
                resource
                    .attributes
                    .iter()
                    .filter(|attribute| {
                        PROMOTED_RESOURCE_ATTRIBUTES.contains(&attribute.key.as_str())
                    })
                    .filter_map(|attribute| {
                        attribute.value.as_ref().map(|value| {
                            (
                                normalize_attribute_key(&attribute.key),
                                attribute_display(value),
                            )
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        for scope_metrics in &resource_metrics.scope_metrics {
            for metric in &scope_metrics.metrics {
                match &metric.data {
                    Some(metric::Data::Gauge(gauge)) => {
                        for point in &gauge.data_points {
                            out.begin_datapoint();
                            let Some(value) = number_value(point) else {
                                continue;
                            };
                            let pairs = base_pairs(&metric.name, &point.attributes, &promoted);
                            out.push(
                                tenant,
                                SeriesLabels::from_pairs(pairs),
                                datapoint_ts(point.time_unix_nano)?,
                                value,
                                SampleKind::Gauge,
                            )?;
                        }
                    }
                    Some(metric::Data::Sum(sum)) => {
                        let kind = temporality_kind(sum.aggregation_temporality);
                        for point in &sum.data_points {
                            out.begin_datapoint();
                            let Some(value) = number_value(point) else {
                                continue;
                            };
                            let pairs = base_pairs(&metric.name, &point.attributes, &promoted);
                            out.push(
                                tenant,
                                SeriesLabels::from_pairs(pairs),
                                datapoint_ts(point.time_unix_nano)?,
                                value,
                                kind,
                            )?;
                        }
                    }
                    Some(metric::Data::Histogram(histogram)) => {
                        let kind = temporality_kind(histogram.aggregation_temporality);
                        for point in &histogram.data_points {
                            out.begin_datapoint();
                            decompose_histogram(
                                tenant,
                                &metric.name,
                                point,
                                &promoted,
                                kind,
                                &mut out,
                            )?;
                        }
                    }
                    Some(metric::Data::ExponentialHistogram(histogram)) => {
                        let kind = temporality_kind(histogram.aggregation_temporality);
                        for point in &histogram.data_points {
                            out.begin_datapoint();
                            decompose_exponential(
                                tenant,
                                &metric.name,
                                point,
                                &promoted,
                                kind,
                                &mut out,
                            )?;
                        }
                    }
                    Some(metric::Data::Summary(summary)) => {
                        for point in &summary.data_points {
                            out.begin_datapoint();
                            let ts = datapoint_ts(point.time_unix_nano)?;
                            let base = base_pairs(&metric.name, &point.attributes, &promoted);
                            for quantile in &point.quantile_values {
                                // A summary's quantiles cannot be
                                // re-aggregated; they are stored as the gauges
                                // they behave as.
                                out.push(
                                    tenant,
                                    with_extra(
                                        &base,
                                        "quantile",
                                        format_boundary(quantile.quantile),
                                    ),
                                    ts,
                                    quantile.value,
                                    SampleKind::Gauge,
                                )?;
                            }
                            let sum_base = base_pairs(
                                &format!("{}_sum", metric.name),
                                &point.attributes,
                                &promoted,
                            );
                            out.push(
                                tenant,
                                SeriesLabels::from_pairs(sum_base),
                                ts,
                                point.sum,
                                SampleKind::Cumulative,
                            )?;
                            let count_base = base_pairs(
                                &format!("{}_count", metric.name),
                                &point.attributes,
                                &promoted,
                            );
                            out.push(
                                tenant,
                                SeriesLabels::from_pairs(count_base),
                                ts,
                                point.count as f64,
                                SampleKind::Cumulative,
                            )?;
                        }
                    }
                    None => {}
                }
            }
        }
    }
    if out.samples.is_empty() {
        return Err(MetricIngestError::EmptyRequest);
    }
    Ok(out.samples)
}

/// The request minus the datapoints admission refused, walked in the same
/// traversal order [`normalize_request`] assigns indices — every datapoint
/// consumes one index whether kept or not, and the pin test asserts the two
/// walks agree. This is what the WAL stores on a partial acceptance, so
/// replay decomposes into exactly the admitted samples and a refused series
/// cannot be resurrected by a restart.
#[cfg(test)]
fn filter_request(
    request: &ExportMetricsServiceRequest,
    admitted: &std::collections::HashSet<u32>,
) -> ExportMetricsServiceRequest {
    let mut next: u32 = 0;
    let mut keep = move || {
        let index = next;
        next += 1;
        admitted.contains(&index)
    };
    let mut filtered = ExportMetricsServiceRequest::default();
    for resource_metrics in &request.resource_metrics {
        let mut kept_resource = resource_metrics.clone();
        kept_resource.scope_metrics.clear();
        for scope_metrics in &resource_metrics.scope_metrics {
            let mut kept_scope = scope_metrics.clone();
            kept_scope.metrics.clear();
            for metric in &scope_metrics.metrics {
                let mut kept_metric = metric.clone();
                let has_points = match &mut kept_metric.data {
                    Some(metric::Data::Gauge(gauge)) => {
                        gauge.data_points.retain(|_| keep());
                        !gauge.data_points.is_empty()
                    }
                    Some(metric::Data::Sum(sum)) => {
                        sum.data_points.retain(|_| keep());
                        !sum.data_points.is_empty()
                    }
                    Some(metric::Data::Histogram(histogram)) => {
                        histogram.data_points.retain(|_| keep());
                        !histogram.data_points.is_empty()
                    }
                    Some(metric::Data::ExponentialHistogram(histogram)) => {
                        histogram.data_points.retain(|_| keep());
                        !histogram.data_points.is_empty()
                    }
                    Some(metric::Data::Summary(summary)) => {
                        summary.data_points.retain(|_| keep());
                        !summary.data_points.is_empty()
                    }
                    None => false,
                };
                if has_points {
                    kept_scope.metrics.push(kept_metric);
                }
            }
            if !kept_scope.metrics.is_empty() {
                kept_resource.scope_metrics.push(kept_scope);
            }
        }
        if !kept_resource.scope_metrics.is_empty() {
            filtered.resource_metrics.push(kept_resource);
        }
    }
    filtered
}

/// Explicit-bounds histogram → one [`HistogramPoint`].
///
/// OTLP gives per-bucket counts; `le` semantics are cumulative, so a running
/// total converts. The `+Inf` bucket is not one of the boundaries — the
/// point's `count` carries it.
fn explicit_histogram_point(point: &HistogramDataPoint) -> HistogramPoint {
    let mut running = 0u64;
    let cumulative = point
        .explicit_bounds
        .iter()
        .enumerate()
        .map(|(index, _)| {
            running = running.saturating_add(point.bucket_counts.get(index).copied().unwrap_or(0));
            running
        })
        .collect();
    HistogramPoint {
        bounds: point.explicit_bounds.iter().copied().collect(),
        cumulative,
        sum: point.sum,
        count: point.count,
    }
}

/// Exponential histogram → one [`HistogramPoint`], downscaled until at most
/// [`MAX_EXP_HISTOGRAM_BUCKETS`] finite boundaries remain. The zero bucket and
/// any negative buckets fold into the smallest boundary — the accepted loss
/// the plan records; latency histograms have no negative half in practice, and
/// a signed boundary vocabulary would double the surface for data fn0 never
/// charts.
///
/// The downscale is computed per datapoint from the observed index range, so
/// a widening range changes the boundaries. Under the `le` fan-out that minted
/// a fresh set of up to sixty-seven series and abandoned the old one; a point
/// keeps its series and changes its schema instead.
fn exponential_histogram_point(point: &ExponentialHistogramDataPoint) -> HistogramPoint {
    let positive: Vec<(i64, u64)> = point
        .positive
        .as_ref()
        .map(|buckets| {
            buckets
                .bucket_counts
                .iter()
                .enumerate()
                .filter(|(_, count)| **count > 0)
                .map(|(index, count)| (buckets.offset as i64 + index as i64, *count))
                .collect()
        })
        .unwrap_or_default();
    let negative_total: u64 = point
        .negative
        .as_ref()
        .map(|buckets| buckets.bucket_counts.iter().sum())
        .unwrap_or(0);
    let below_smallest = point.zero_count.saturating_add(negative_total);

    let (bounds, cumulative) = if positive.is_empty() {
        // All mass is at or below zero: no finite boundary, and the `+Inf`
        // bucket the count carries answers for everything.
        (Vec::new(), Vec::new())
    } else {
        // Shifting an index right by `d` halves the resolution (scale − d)
        // exactly — arithmetic shift keeps floor semantics for negative
        // indices.
        let min_index = positive.iter().map(|(index, _)| *index).min().unwrap();
        let max_index = positive.iter().map(|(index, _)| *index).max().unwrap();
        let mut downscale = 0u32;
        while ((max_index >> downscale) - (min_index >> downscale) + 1)
            > MAX_EXP_HISTOGRAM_BUCKETS as i64
        {
            downscale += 1;
        }
        let scale = point.scale - downscale as i32;
        let mut merged: std::collections::BTreeMap<i64, u64> = std::collections::BTreeMap::new();
        for (index, count) in positive {
            *merged.entry(index >> downscale).or_default() += count;
        }
        let mut running = below_smallest;
        let mut bounds = Vec::with_capacity(merged.len());
        let mut cumulative = Vec::with_capacity(merged.len());
        for (index, count) in merged {
            running = running.saturating_add(count);
            // The bucket's upper boundary: 2^((index + 1) · 2^−scale).
            bounds.push(2f64.powf((index + 1) as f64 * 2f64.powi(-scale)));
            cumulative.push(running);
        }
        (bounds, cumulative)
    };

    HistogramPoint {
        bounds: bounds.into(),
        cumulative,
        sum: point.sum,
        count: point.count,
    }
}

struct HistogramNaming<'a> {
    name: &'a str,
    attributes: &'a [opentelemetry_proto::tonic::common::v1::KeyValue],
    promoted: &'a [(String, String)],
    ts: i64,
    kind: SampleKind,
}

/// One histogram datapoint becomes one sample under the instrument's own
/// name — no `_bucket` suffix, no `le`, no `_sum` and `_count` beside it.
/// Those series still exist for anything that asks; they are synthesized when
/// read (`series::synthesize_histogram_series`) rather than stored, so an
/// instrument costs one identity in the index, the catalogs and the parts
/// instead of `bounds + 3`.
fn push_histogram_point(
    tenant: &TenantId,
    naming: &HistogramNaming<'_>,
    point: HistogramPoint,
    out: &mut Decomposition,
) -> Result<(), MetricIngestError> {
    let HistogramNaming {
        name,
        attributes,
        promoted,
        ts,
        kind,
    } = *naming;
    out.push_value(
        tenant,
        SeriesLabels::from_pairs(base_pairs(name, attributes, promoted)),
        ts,
        MetricValue::Histogram(point),
        kind,
    )
}

fn decompose_histogram(
    tenant: &TenantId,
    name: &str,
    point: &HistogramDataPoint,
    promoted: &[(String, String)],
    kind: SampleKind,
    out: &mut Decomposition,
) -> Result<(), MetricIngestError> {
    let naming = HistogramNaming {
        name,
        attributes: &point.attributes,
        promoted,
        ts: datapoint_ts(point.time_unix_nano)?,
        kind,
    };
    push_histogram_point(tenant, &naming, explicit_histogram_point(point), out)
}

fn decompose_exponential(
    tenant: &TenantId,
    name: &str,
    point: &ExponentialHistogramDataPoint,
    promoted: &[(String, String)],
    kind: SampleKind,
    out: &mut Decomposition,
) -> Result<(), MetricIngestError> {
    let naming = HistogramNaming {
        name,
        attributes: &point.attributes,
        promoted,
        ts: datapoint_ts(point.time_unix_nano)?,
        kind,
    };
    push_histogram_point(tenant, &naming, exponential_histogram_point(point), out)
}

/// Accepting one OTLP metrics export, independent of how it arrived. The
/// metrics counterpart to [`crate::trace_ingest::OtlpTraceIngest`], split for
/// the same reason: a limit enforced on gRPC and forgotten on HTTP is not a
/// limit.
pub struct OtlpMetricIngest<'a> {
    pub journal: &'a Journal,
    pub config: &'a Config,
    pub tenant_quota: &'a crate::tenant_quota::TenantQuota,
    pub tenant_policy: &'a crate::tenant_policy::TenantPolicy,
    pub metrics: &'a crate::metrics::RuntimeMetrics,
    pub clock: &'a crate::clock::Clock,
}

struct PreparedMetricGroup {
    tenant: TenantId,
    encoded: Vec<u8>,
    samples: Vec<MetricSample>,
}

impl OtlpMetricIngest<'_> {
    /// See [`crate::log_ingest::OtlpLogIngest::admit_size`]: the tenant is no
    /// longer knowable this early, and the size still is.
    pub fn admit_size(&self, encoded_len: usize) -> Result<(), IngestError> {
        if encoded_len > MAX_OTLP_REQUEST_BYTES {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                format!("OTLP request exceeds the maximum of {MAX_OTLP_REQUEST_BYTES} bytes"),
            )
                .into());
        }
        Ok(())
    }

    /// The metric counterpart to
    /// [`crate::log_ingest::OtlpLogIngest::enqueue_request`].  All tenant
    /// groups are normalized and admitted before the first journal append, so
    /// memory/cardinality pressure can only accept or reject the complete
    /// export.  The append itself remains one WAL record per tenant.
    ///
    pub async fn enqueue_request(
        &self,
        request: ExportMetricsServiceRequest,
        mark: Option<crate::journal::CollectMark>,
    ) -> Result<(Vec<crate::journal::PendingAppend>, MetricAcceptOutcome), IngestError> {
        let datapoints = count_datapoints(&request)?;
        if datapoints > MAX_OTLP_METRIC_SAMPLES {
            return Err(too_many_samples());
        }

        let split = crate::otlp_tenant::split_metrics(request, self.tenant_policy);
        split.dropped.record(self.metrics, "metrics");

        // Normalization and re-encoding are synchronous. Keep their temporary
        // protobuf/label allocations in the ingest arena, but end the guard
        // before the admission/journal awaits below.
        let ingest_arena = crate::memprof::enter(crate::memprof::Arena::Ingest);
        let mut groups = Vec::with_capacity(split.groups.len());
        for (tenant, group) in split.groups {
            if let Err(error) = self.tenant_quota.admit_storage(&tenant) {
                tracing::warn!(%tenant, reason = error.message, "dropping metrics for a tenant at its storage limit");
                continue;
            }
            let samples = normalize_request(&tenant, &group).map_err(|error| match error {
                MetricIngestError::TooManySamples => too_many_samples(),
                other => IngestError::from((StatusCode::BAD_REQUEST, other.to_string())),
            })?;
            let window = crate::ingest::TimestampWindow::from_config(self.config, self.clock);
            for sample in &samples {
                window
                    .validate(sample.ts_ns)
                    .map_err(|error| (StatusCode::BAD_REQUEST, error))?;
            }
            let encoded = group.encode_to_vec();
            groups.push(PreparedMetricGroup {
                tenant,
                encoded,
                samples,
            });
        }
        drop(ingest_arena);
        self.enqueue_prepared(groups, mark).await
    }

    async fn enqueue_prepared(
        &self,
        groups: Vec<PreparedMetricGroup>,
        mark: Option<crate::journal::CollectMark>,
    ) -> Result<(Vec<crate::journal::PendingAppend>, MetricAcceptOutcome), IngestError> {
        if groups.is_empty() {
            return Ok((Vec::new(), MetricAcceptOutcome::default()));
        }
        let series_memtable = self.journal.series_memtable();
        let group_refs: Vec<_> = groups
            .iter()
            .map(|group| (&group.tenant, group.samples.as_slice()))
            .collect();
        let idle_cutoff = self
            .clock
            .now_ns()
            .saturating_sub(self.config.metric_series_idle_timeout.as_nanos() as i64);
        let admissions = series_memtable
            .admit_request(
                &group_refs,
                (!self.config.capacity_probe).then_some(self.config.max_active_series).flatten(),
                idle_cutoff,
            )
            .map_err(|error| {
                self.metrics
                    .ingest_throttled
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                IngestError {
                    status: StatusCode::TOO_MANY_REQUESTS,
                    message: format!(
                        "metric export needs {} new series but the process-wide limit of {} live series is already at {}; retry after capacity returns (SIGNY_MAX_ACTIVE_SERIES)",
                        error.new_series, error.limit, error.active_series
                    ),
                    retry_after: Some(self.config.backpressure_retry_after),
                }
            })?;
        let growth: Vec<u64> = groups
            .iter()
            .map(|group| {
                crate::series::SeriesMemTable::estimate_sample_bytes(&[(
                    &group.tenant,
                    group.samples.as_slice(),
                )])
            })
            .collect();
        let total_growth = growth.iter().copied().sum();
        let mut permit = self
            .journal
            .try_reserve_metric_bytes(
                total_growth,
                (!self.config.capacity_probe)
                    .then_some(self.config.max_memtable_bytes)
                    .flatten(),
            )
            .ok_or_else(|| {
                self.metrics
                    .ingest_throttled
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                series_memtable
                    .counters()
                    .metric_memory_rejected_total
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                IngestError {
                    status: StatusCode::TOO_MANY_REQUESTS,
                    message: format!(
                        "metric export needs {total_growth} bytes of memtable space but the process-wide budget is full; flush is not keeping up"
                    ),
                    retry_after: Some(self.config.backpressure_retry_after),
                }
            })?;
        let last = groups.len() - 1;
        let records = groups
            .into_iter()
            .zip(admissions)
            .zip(growth)
            .map(|((group, admission), bytes)| (group, admission, bytes))
            .enumerate()
            .map(
                |(index, (group, admission, bytes))| crate::journal::ReservedMetricAppend {
                    tenant: group.tenant,
                    data: group.encoded,
                    samples: group.samples,
                    mark: (index == last).then_some(mark).flatten(),
                    metric_memory_permit: permit.split(bytes),
                    metric_series_admission: admission,
                },
            )
            .collect();
        let pending = self
            .journal
            .enqueue_metrics_reserved_batch(records)
            .await
            .map_err(crate::log_ingest::journal_write_failed)?;
        Ok((pending, MetricAcceptOutcome::default()))
    }
}

/// What an accepted export answered with.  Memory/cardinality pressure is
/// rejected before an append, so accepted exports currently carry no partial
/// success; the fields remain for compatibility with the collect response.
#[derive(Debug, Default)]
pub struct MetricAcceptOutcome {
    pub rejected_data_points: u64,
    pub rejection: Option<String>,
}

impl MetricAcceptOutcome {
    pub fn partial_success(
        &self,
    ) -> Option<opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsPartialSuccess>
    {
        (self.rejected_data_points > 0).then(|| {
            opentelemetry_proto::tonic::collector::metrics::v1::ExportMetricsPartialSuccess {
                rejected_data_points: self.rejected_data_points.min(i64::MAX as u64) as i64,
                error_message: self.rejection.clone().unwrap_or_default(),
            }
        })
    }
}

/// The cheap pre-check: every datapoint decomposes into at least one sample,
/// so a request past the cap on datapoints alone is refused before any
/// decomposition work. Counted over the whole request and before the split,
/// so N groups cannot multiply it.
fn count_datapoints(request: &ExportMetricsServiceRequest) -> Result<usize, IngestError> {
    request
        .resource_metrics
        .iter()
        .flat_map(|resource| resource.scope_metrics.iter())
        .flat_map(|scope| scope.metrics.iter())
        .map(datapoint_count)
        .try_fold(0usize, |count, points| count.checked_add(points))
        .ok_or_else(|| {
            IngestError::from((
                StatusCode::PAYLOAD_TOO_LARGE,
                "OTLP metric datapoint count overflow".to_string(),
            ))
        })
}

fn too_many_samples() -> IngestError {
    // Non-retryable on purpose, like the span-count cap: the identical bytes
    // cannot become acceptable, so the collector must split the batch.
    (
        StatusCode::PAYLOAD_TOO_LARGE,
        format!(
            "OTLP metrics request decomposes into more than {MAX_OTLP_METRIC_SAMPLES} samples; \
split the batch (histograms count bounds + 3 samples per datapoint)"
        ),
    )
        .into()
}

fn datapoint_count(metric: &opentelemetry_proto::tonic::metrics::v1::Metric) -> usize {
    match &metric.data {
        Some(metric::Data::Gauge(gauge)) => gauge.data_points.len(),
        Some(metric::Data::Sum(sum)) => sum.data_points.len(),
        Some(metric::Data::Histogram(histogram)) => histogram.data_points.len(),
        Some(metric::Data::ExponentialHistogram(histogram)) => histogram.data_points.len(),
        Some(metric::Data::Summary(summary)) => summary.data_points.len(),
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backpressure::IngestGate;
    use crate::series::SeriesMemTable;
    use crate::shutdown::ShutdownState;
    use crate::tenant::test_tenant;
    use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
    use opentelemetry_proto::tonic::metrics::v1::{
        Gauge, Histogram, Metric, ResourceMetrics, ScopeMetrics, Sum, Summary, SummaryDataPoint,
        exponential_histogram_data_point, summary_data_point,
    };
    use opentelemetry_proto::tonic::resource::v1::Resource;
    use std::sync::Arc;

    fn attr(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_string())),
            }),
            ..Default::default()
        }
    }

    fn gauge_point(ts: u64, value: f64, attributes: Vec<KeyValue>) -> NumberDataPoint {
        NumberDataPoint {
            attributes,
            time_unix_nano: ts,
            value: Some(number_data_point::Value::AsDouble(value)),
            ..Default::default()
        }
    }

    /// Every fixture request names the test tenant, because an export that
    /// names none is dropped rather than stored. The attribute is added to
    /// whatever resource the caller wanted, so a test about promoted
    /// attributes still gets exactly the ones it asked for.
    fn request_with(
        metrics: Vec<Metric>,
        resource: Option<Resource>,
    ) -> ExportMetricsServiceRequest {
        let mut resource = resource.unwrap_or_default();
        resource
            .attributes
            .extend(crate::otlp_tenant::test_tenant_resource().attributes);
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(resource),
                scope_metrics: vec![ScopeMetrics {
                    metrics,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn pairs_of(sample: &MetricSample) -> Vec<(String, String)> {
        sample.labels.pairs().unwrap()
    }

    fn label(sample: &MetricSample, key: &str) -> Option<String> {
        pairs_of(sample)
            .into_iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    #[test]
    fn a_gauge_becomes_one_sample_with_promoted_and_datapoint_labels() {
        let request = request_with(
            vec![Metric {
                name: "queue_depth".to_string(),
                data: Some(metric::Data::Gauge(Gauge {
                    data_points: vec![gauge_point(100, 7.5, vec![attr("shard", "3")])],
                })),
                ..Default::default()
            }],
            Some(Resource {
                attributes: vec![attr("service.name", "api"), attr("process.pid", "4242")],
                ..Default::default()
            }),
        );
        let samples = normalize_request(&test_tenant(), &request).unwrap();
        assert_eq!(samples.len(), 1);
        let sample = &samples[0];
        assert_eq!(sample.ts_ns, 100);
        assert_eq!(sample.value, MetricValue::Scalar(7.5));
        assert_eq!(sample.kind, SampleKind::Gauge);
        assert_eq!(
            label(sample, METRIC_NAME_LABEL).as_deref(),
            Some("queue_depth")
        );
        assert_eq!(label(sample, "service_name").as_deref(), Some("api"));
        assert_eq!(label(sample, "shard").as_deref(), Some("3"));
        assert_eq!(
            label(sample, "process_pid"),
            None,
            "unpromoted resource attributes are dropped, not series identity"
        );
    }

    #[test]
    fn a_datapoint_attribute_wins_over_a_promoted_resource_attribute() {
        let request = request_with(
            vec![Metric {
                name: "queue_depth".to_string(),
                data: Some(metric::Data::Gauge(Gauge {
                    data_points: vec![gauge_point(
                        100,
                        1.0,
                        vec![attr("service_name", "from-datapoint")],
                    )],
                })),
                ..Default::default()
            }],
            Some(Resource {
                attributes: vec![attr("service.name", "from-resource")],
                ..Default::default()
            }),
        );
        let samples = normalize_request(&test_tenant(), &request).unwrap();
        assert_eq!(
            label(&samples[0], "service_name").as_deref(),
            Some("from-datapoint")
        );
    }

    #[test]
    fn a_delta_sum_is_tagged_delta_and_a_cumulative_one_is_not() {
        let sum = |temporality: AggregationTemporality| {
            request_with(
                vec![Metric {
                    name: "http_requests_total".to_string(),
                    data: Some(metric::Data::Sum(Sum {
                        data_points: vec![gauge_point(100, 5.0, vec![])],
                        aggregation_temporality: temporality as i32,
                        is_monotonic: true,
                    })),
                    ..Default::default()
                }],
                None,
            )
        };
        let delta = normalize_request(&test_tenant(), &sum(AggregationTemporality::Delta)).unwrap();
        assert_eq!(delta[0].kind, SampleKind::Delta);
        let cumulative =
            normalize_request(&test_tenant(), &sum(AggregationTemporality::Cumulative)).unwrap();
        assert_eq!(cumulative[0].kind, SampleKind::Cumulative);
    }

    #[test]
    fn an_explicit_point_is_cumulative_and_keeps_the_declared_totals() {
        let point = explicit_histogram_point(&HistogramDataPoint {
            count: 10,
            sum: Some(1.25),
            bucket_counts: vec![3, 4, 2, 1],
            explicit_bounds: vec![0.005, 0.01, 0.025],
            ..Default::default()
        });
        assert_eq!(&*point.bounds, &[0.005, 0.01, 0.025]);
        assert_eq!(point.cumulative, vec![3, 7, 9]);
        assert_eq!(point.sum, Some(1.25));
        // The last bucket's count lives past the last finite bound, so it is
        // in `count` and not in `cumulative`.
        assert_eq!(point.count, 10);
    }

    fn exponential_point(
        scale: i32,
        offset: i32,
        counts: Vec<u64>,
        zero: u64,
        negative: Vec<u64>,
    ) -> ExponentialHistogramDataPoint {
        let total: u64 = counts.iter().sum::<u64>() + zero + negative.iter().sum::<u64>();
        ExponentialHistogramDataPoint {
            scale,
            zero_count: zero,
            count: total,
            sum: Some(1.0),
            positive: Some(exponential_histogram_data_point::Buckets {
                offset,
                bucket_counts: counts,
            }),
            negative: (!negative.is_empty()).then_some(exponential_histogram_data_point::Buckets {
                offset: 0,
                bucket_counts: negative.clone(),
            }),
            ..Default::default()
        }
    }

    #[test]
    fn an_exponential_point_downscales_until_it_fits_the_bucket_cap() {
        let point = exponential_point(0, 0, vec![1; 100], 0, Vec::new());
        let point = exponential_histogram_point(&point);
        assert!(
            point.bounds.len() <= MAX_EXP_HISTOGRAM_BUCKETS,
            "a hundred populated buckets must not survive the cap: {}",
            point.bounds.len()
        );
        assert_eq!(
            point.bounds.len(),
            50,
            "one halving is enough for a hundred"
        );
        assert_eq!(point.bounds.len(), point.cumulative.len());
        assert_eq!(
            point.cumulative.last().copied(),
            Some(100),
            "downscaling merges counts, it does not drop them"
        );
        assert_eq!(point.count, 100);
    }

    #[test]
    fn an_exponential_point_folds_the_zero_and_negative_mass_below_its_smallest_bound() {
        let point = exponential_histogram_point(&exponential_point(0, 0, vec![2], 5, vec![3]));
        assert_eq!(point.bounds.len(), 1);
        assert_eq!(
            point.cumulative,
            vec![10],
            "five at zero and three below it are still at or under the first bound"
        );
        assert_eq!(point.count, 10);
    }

    #[test]
    fn an_exponential_point_with_no_positive_mass_has_no_finite_bound() {
        let point =
            exponential_histogram_point(&exponential_point(0, 0, Vec::new(), 4, Vec::new()));
        assert!(point.bounds.is_empty());
        assert!(point.cumulative.is_empty());
        assert_eq!(point.count, 4, "the +Inf bucket answers for all of it");
    }

    #[test]
    fn a_histogram_decomposes_into_cumulative_le_buckets_sum_and_count() {
        let request = request_with(
            vec![Metric {
                name: "http_request_duration_seconds".to_string(),
                data: Some(metric::Data::Histogram(Histogram {
                    data_points: vec![HistogramDataPoint {
                        attributes: vec![attr("instance", "a")],
                        time_unix_nano: 100,
                        count: 10,
                        sum: Some(1.25),
                        bucket_counts: vec![3, 4, 2, 1],
                        explicit_bounds: vec![0.005, 0.01, 0.025],
                        ..Default::default()
                    }],
                    aggregation_temporality: AggregationTemporality::Cumulative as i32,
                })),
                ..Default::default()
            }],
            None,
        );
        let samples = normalize_request(&test_tenant(), &request).unwrap();
        // One instrument, one sample. What used to be `3 bounds + Inf + sum +
        // count` is now the shape inside it, and the read path hands those
        // back to anything that asks in those terms.
        assert_eq!(samples.len(), 1);
        assert_eq!(
            samples[0].labels.metric_name().as_deref(),
            Some("http_request_duration_seconds"),
            "the stored identity is the instrument, without a _bucket suffix"
        );
        assert!(label(&samples[0], "le").is_none());
        let MetricValue::Histogram(point) = &samples[0].value else {
            panic!("a histogram datapoint decomposes to a histogram value");
        };
        assert_eq!(&*point.bounds, &[0.005, 0.01, 0.025]);
        assert_eq!(point.cumulative, vec![3, 7, 9], "le counts are cumulative");
        assert_eq!(point.count, 10);
        assert_eq!(point.sum, Some(1.25));

        // And the series it answers as are exactly what the fan-out wrote.
        let synthesized =
            crate::series::synthesize_histogram_series(&samples[0].labels, &[(100, point.clone())])
                .unwrap();
        let named = |name: &str| {
            synthesized
                .iter()
                .find(|(labels, _)| labels.metric_name().as_deref() == Some(name))
        };
        assert_eq!(synthesized.len(), 6, "3 bounds + Inf + sum + count");
        let bucket = |le: &str| {
            synthesized
                .iter()
                .find(|(labels, _)| {
                    labels
                        .pairs()
                        .unwrap()
                        .iter()
                        .any(|(key, value)| key == "le" && value == le)
                })
                .map(|(_, samples)| samples[0].1)
                .unwrap_or_else(|| panic!("bucket le={le} exists"))
        };
        assert_eq!(bucket("0.005"), 3.0);
        assert_eq!(bucket("0.01"), 7.0);
        assert_eq!(bucket("0.025"), 9.0);
        assert_eq!(bucket("+Inf"), 10.0);
        assert_eq!(
            named("http_request_duration_seconds_sum").unwrap().1[0].1,
            1.25
        );
        assert_eq!(
            named("http_request_duration_seconds_count").unwrap().1[0].1,
            10.0
        );
        assert!(
            samples
                .iter()
                .all(|sample| sample.kind == SampleKind::Cumulative)
        );
    }

    #[test]
    fn an_exponential_histogram_downscales_to_the_bucket_cap() {
        // 200 populated buckets at scale 3 must downscale (200 > 64).
        let request = request_with(
            vec![Metric {
                name: "latency_seconds".to_string(),
                data: Some(metric::Data::ExponentialHistogram(
                    opentelemetry_proto::tonic::metrics::v1::ExponentialHistogram {
                        data_points: vec![ExponentialHistogramDataPoint {
                            time_unix_nano: 100,
                            count: 205,
                            sum: Some(12.5),
                            scale: 3,
                            zero_count: 5,
                            positive: Some(exponential_histogram_data_point::Buckets {
                                offset: -20,
                                bucket_counts: vec![1; 200],
                            }),
                            negative: None,
                            ..Default::default()
                        }],
                        aggregation_temporality: AggregationTemporality::Cumulative as i32,
                    },
                )),
                ..Default::default()
            }],
            None,
        );
        let samples = normalize_request(&test_tenant(), &request).unwrap();
        assert_eq!(samples.len(), 1, "one instrument, whatever its resolution");
        let MetricValue::Histogram(point) = &samples[0].value else {
            panic!("an exponential datapoint decomposes to a histogram value");
        };
        assert!(
            point.bounds.len() <= MAX_EXP_HISTOGRAM_BUCKETS,
            "{} finite boundaries survived the downscale",
            point.bounds.len()
        );
        assert!(
            point.bounds.len() >= MAX_EXP_HISTOGRAM_BUCKETS / 2,
            "one halving, not a collapse"
        );
        let mut last = 0u64;
        for cumulative in &point.cumulative {
            assert!(*cumulative >= last, "le counts stay cumulative");
            last = *cumulative;
        }
        assert!(
            point.cumulative[0] >= 5,
            "the zero count folds into the smallest bound"
        );
        assert_eq!(point.count, 205, "the +Inf bucket is the declared count");
    }

    #[test]
    fn a_summary_becomes_quantile_gauges_plus_sum_and_count() {
        let request = request_with(
            vec![Metric {
                name: "gc_pause_seconds".to_string(),
                data: Some(metric::Data::Summary(Summary {
                    data_points: vec![SummaryDataPoint {
                        time_unix_nano: 100,
                        count: 42,
                        sum: 3.5,
                        quantile_values: vec![
                            summary_data_point::ValueAtQuantile {
                                quantile: 0.5,
                                value: 0.01,
                            },
                            summary_data_point::ValueAtQuantile {
                                quantile: 0.99,
                                value: 0.25,
                            },
                        ],
                        ..Default::default()
                    }],
                })),
                ..Default::default()
            }],
            None,
        );
        let samples = normalize_request(&test_tenant(), &request).unwrap();
        assert_eq!(samples.len(), 4);
        let p99 = samples
            .iter()
            .find(|sample| label(sample, "quantile").as_deref() == Some("0.99"))
            .unwrap();
        assert_eq!(p99.value.as_scalar(), Some(0.25));
        assert_eq!(p99.kind, SampleKind::Gauge);
        assert!(samples.iter().any(|sample| {
            sample.labels.metric_name().as_deref() == Some("gc_pause_seconds_count")
                && sample.value == MetricValue::Scalar(42.0)
                && sample.kind == SampleKind::Cumulative
        }));
    }

    #[test]
    fn an_empty_request_is_refused() {
        assert_eq!(
            normalize_request(&test_tenant(), &ExportMetricsServiceRequest::default()).unwrap_err(),
            MetricIngestError::EmptyRequest
        );
    }

    #[test]
    fn boundary_rendering_is_pinned() {
        assert_eq!(format_boundary(0.005), "0.005");
        assert_eq!(format_boundary(1.0), "1");
        assert_eq!(format_boundary(f64::INFINITY), "+Inf");
        assert_eq!(
            format_boundary(std::f64::consts::SQRT_2),
            "1.4142135623730951",
            "an exponential boundary keeps its shortest round-trip rendering"
        );
    }

    /// The collect route's own sequence over one record, so these tests
    /// refuse what the route refuses and store what it stores. The tenant
    /// rides in the payload, so what makes a request the test tenant's is the
    /// resource `request_with` builds.
    struct Ingest {
        journal: Arc<Journal>,
        shutdown: Arc<ShutdownState>,
        config: Arc<Config>,
        ingest_gate: Arc<IngestGate>,
        tenant_quota: Arc<crate::tenant_quota::TenantQuota>,
        tenant_policy: Arc<crate::tenant_policy::TenantPolicy>,
        metrics: Arc<crate::metrics::RuntimeMetrics>,
        clock: Arc<crate::clock::Clock>,
    }

    impl Ingest {
        async fn accept(
            &self,
            request: ExportMetricsServiceRequest,
        ) -> Result<MetricAcceptOutcome, IngestError> {
            let ingest = OtlpMetricIngest {
                journal: &self.journal,
                config: &self.config,
                tenant_quota: &self.tenant_quota,
                tenant_policy: &self.tenant_policy,
                metrics: &self.metrics,
                clock: &self.clock,
            };
            crate::backpressure::admit_batch(&self.shutdown, &self.ingest_gate)?;
            ingest.admit_size(request.encoded_len())?;
            let (pending, outcome) = ingest.enqueue_request(request, None).await?;
            for pending in pending {
                pending
                    .settle()
                    .await
                    .map_err(crate::log_ingest::journal_write_failed)?;
            }
            Ok(outcome)
        }
    }

    fn ingest_over(config: Config) -> (Ingest, Arc<SeriesMemTable>, Arc<Journal>) {
        std::fs::create_dir_all(&config.data_dir).unwrap();
        let series_memtable = Arc::new(SeriesMemTable::new());
        let journal = Arc::new(
            crate::journal::Journal::spawn_with_signals(
                &config,
                Arc::new(crate::memtable::MemTable::new()),
                Arc::new(crate::trace::TraceMemTable::new()),
                series_memtable.clone(),
            )
            .unwrap(),
        );
        let ingest_gate = IngestGate::for_test(&journal, &config);
        let ingest = Ingest {
            journal: journal.clone(),
            shutdown: Arc::new(crate::shutdown::ShutdownState::new()),
            config: Arc::new(config.clone()),
            ingest_gate,
            tenant_quota: crate::tenant_quota::TenantQuota::for_test(&config),
            tenant_policy: Arc::new(crate::tenant_policy::TenantPolicy::disabled()),
            metrics: Arc::new(crate::metrics::RuntimeMetrics::new()),
            clock: crate::clock::Clock::system(),
        };
        (ingest, series_memtable, journal)
    }

    fn now_ns() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn small_request() -> ExportMetricsServiceRequest {
        request_with(
            vec![Metric {
                name: "queue_depth".to_string(),
                data: Some(metric::Data::Gauge(Gauge {
                    data_points: vec![gauge_point(now_ns(), 7.5, vec![attr("instance", "a")])],
                })),
                ..Default::default()
            }],
            None,
        )
    }

    #[tokio::test]
    async fn export_acknowledges_after_journal_append_and_replay_reproduces_it() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-metric-ingest-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        };
        let (ingest, series_memtable, journal) = ingest_over(config);
        ingest.accept(small_request()).await.unwrap();
        let live = series_memtable.sorted_samples(&test_tenant()).unwrap();
        assert_eq!(live.len(), 1);

        let replayed = SeriesMemTable::new();
        crate::journal::replay_with_signals(
            journal.wal_path(),
            journal.ckpt_path(),
            &crate::memtable::MemTable::new(),
            &crate::trace::TraceMemTable::new(),
            &replayed,
        )
        .unwrap();
        assert_eq!(
            replayed.sorted_samples(&test_tenant()).unwrap(),
            live,
            "replay reproduces exactly what live ingest stored"
        );
    }

    #[tokio::test]
    async fn a_delta_stream_replays_into_the_same_running_totals() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-metric-delta-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        };
        let (ingest, series_memtable, journal) = ingest_over(config);
        let base = now_ns();
        for (offset, value) in [(0u64, 5.0), (1_000_000_000, 3.0), (2_000_000_000, 2.0)] {
            let request = request_with(
                vec![Metric {
                    name: "http_requests_total".to_string(),
                    data: Some(metric::Data::Sum(Sum {
                        data_points: vec![gauge_point(base + offset, value, vec![])],
                        aggregation_temporality: AggregationTemporality::Delta as i32,
                        is_monotonic: true,
                    })),
                    ..Default::default()
                }],
                None,
            );
            ingest.accept(request).await.unwrap();
        }
        let live = series_memtable.sorted_samples(&test_tenant()).unwrap();
        let totals: Vec<f64> = live
            .values()
            .next()
            .unwrap()
            .iter()
            .map(|(_, v)| *v)
            .collect();
        assert_eq!(totals, vec![5.0, 8.0, 10.0]);

        let replayed = SeriesMemTable::new();
        crate::journal::replay_with_signals(
            journal.wal_path(),
            journal.ckpt_path(),
            &crate::memtable::MemTable::new(),
            &crate::trace::TraceMemTable::new(),
            &replayed,
        )
        .unwrap();
        assert_eq!(replayed.sorted_samples(&test_tenant()).unwrap(), live);
    }

    #[tokio::test]
    async fn export_rejects_while_draining_without_inserting() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-metric-drain-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        };
        let (ingest, series_memtable, _journal) = ingest_over(config);
        ingest.shutdown.begin_drain();
        let error = ingest.accept(small_request()).await.unwrap_err();
        assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(series_memtable.is_empty());
    }

    #[tokio::test]
    async fn export_is_refused_once_the_buffers_are_over_their_limit() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-metric-gate-{}", uuid::Uuid::new_v4())),
            flush_max_bytes: 1,
            max_memtable_bytes: Some(1),
            ..Config::default()
        };
        let (ingest, series_memtable, _journal) = ingest_over(config);
        let error = ingest
            .accept(small_request())
            .await
            .expect_err("a request that cannot fit the byte budget is refused");
        assert_eq!(error.status, StatusCode::TOO_MANY_REQUESTS);
        assert!(
            series_memtable.is_empty(),
            "rejection happens before WAL insert"
        );
    }

    #[tokio::test]
    async fn export_rejects_a_request_past_the_decomposed_sample_cap() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-metric-cap-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        };
        let (ingest, series_memtable, _journal) = ingest_over(config);
        // A histogram datapoint is one sample now however many boundaries it
        // has, so the cap is reached by datapoints rather than by resolution:
        // 125 000 of them, over the cap that 25 000 no longer approaches.
        let point = HistogramDataPoint {
            time_unix_nano: now_ns(),
            count: 1,
            sum: Some(0.1),
            bucket_counts: vec![1, 0, 0],
            explicit_bounds: vec![0.1, 0.2],
            ..Default::default()
        };
        let request = request_with(
            vec![Metric {
                name: "h".to_string(),
                data: Some(metric::Data::Histogram(Histogram {
                    data_points: vec![point; 125_000],
                    aggregation_temporality: AggregationTemporality::Cumulative as i32,
                })),
                ..Default::default()
            }],
            None,
        );
        let error = ingest.accept(request).await.unwrap_err();
        assert_eq!(error.status, StatusCode::PAYLOAD_TOO_LARGE);
        assert!(series_memtable.is_empty());
    }

    /// The traversal pin: filtering by an admitted set and re-decomposing
    /// yields exactly the admitted samples. If `filter_request` and
    /// `normalize_request` ever walked datapoints in different orders, this
    /// is the test that catches it before a replay resurrects a refusal.
    #[test]
    fn filtering_then_decomposing_equals_decomposing_then_filtering() {
        let request = request_with(
            vec![
                Metric {
                    name: "queue_depth".to_string(),
                    data: Some(metric::Data::Gauge(Gauge {
                        data_points: vec![
                            gauge_point(100, 1.0, vec![attr("instance", "a")]),
                            gauge_point(100, 2.0, vec![attr("instance", "b")]),
                        ],
                    })),
                    ..Default::default()
                },
                Metric {
                    name: "http_request_duration_seconds".to_string(),
                    data: Some(metric::Data::Histogram(Histogram {
                        data_points: vec![HistogramDataPoint {
                            time_unix_nano: 100,
                            count: 3,
                            sum: Some(0.3),
                            bucket_counts: vec![1, 2],
                            explicit_bounds: vec![0.1],
                            ..Default::default()
                        }],
                        aggregation_temporality: AggregationTemporality::Cumulative as i32,
                    })),
                    ..Default::default()
                },
                Metric {
                    name: "http_requests_total".to_string(),
                    data: Some(metric::Data::Sum(Sum {
                        data_points: vec![gauge_point(100, 5.0, vec![])],
                        aggregation_temporality: AggregationTemporality::Cumulative as i32,
                        is_monotonic: true,
                    })),
                    ..Default::default()
                },
            ],
            None,
        );
        let all = normalize_request(&test_tenant(), &request).unwrap();
        // Admit datapoints 1 (second gauge) and 2 (the histogram); refuse 0 and 3.
        let admitted: std::collections::HashSet<u32> = [1, 2].into_iter().collect();
        let filtered = filter_request(&request, &admitted);
        let refiltered = normalize_request(&test_tenant(), &filtered).unwrap();
        let expected: Vec<_> = all
            .iter()
            .filter(|sample| admitted.contains(&sample.datapoint_index))
            .collect();
        assert_eq!(refiltered.len(), expected.len());
        for (kept, original) in refiltered.iter().zip(expected) {
            assert_eq!(kept.labels, original.labels);
            assert_eq!(kept.ts_ns, original.ts_ns);
            assert_eq!(kept.value, original.value);
        }
        assert!(
            refiltered.iter().any(|sample| {
                sample.labels.metric_name().as_deref() == Some("http_request_duration_seconds")
                    && matches!(sample.value, MetricValue::Histogram(_))
            }),
            "the admitted histogram survives the round trip whole"
        );
    }

    #[tokio::test]
    async fn past_the_cap_refuses_the_whole_export() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-metric-ladder-{}", uuid::Uuid::new_v4())),
            max_active_series: Some(2),
            ..Config::default()
        };
        let (ingest, series_memtable, journal) = ingest_over(config);
        let base = now_ns();
        let known = request_with(
            vec![Metric {
                name: "queue_depth".to_string(),
                data: Some(metric::Data::Gauge(Gauge {
                    data_points: vec![
                        gauge_point(base, 1.0, vec![attr("instance", "a")]),
                        gauge_point(base, 2.0, vec![attr("instance", "b")]),
                    ],
                })),
                ..Default::default()
            }],
            None,
        );
        let outcome = ingest.accept(known).await.unwrap();
        assert!(outcome.partial_success().is_none());

        let mixed = request_with(
            vec![Metric {
                name: "queue_depth".to_string(),
                data: Some(metric::Data::Gauge(Gauge {
                    data_points: vec![
                        gauge_point(base + 1_000_000_000, 3.0, vec![attr("instance", "a")]),
                        gauge_point(base + 1_000_000_000, 9.0, vec![attr("instance", "new-1")]),
                        gauge_point(base + 1_000_000_000, 9.0, vec![attr("instance", "new-2")]),
                    ],
                })),
                ..Default::default()
            }],
            None,
        );
        let error = ingest
            .accept(mixed)
            .await
            .expect_err("memory/cardinality pressure refuses the complete export");
        assert_eq!(error.status, StatusCode::TOO_MANY_REQUESTS);
        assert!(error.retry_after.is_some());

        let live = series_memtable.sorted_samples(&test_tenant()).unwrap();
        assert_eq!(live.len(), 2, "no refused series exists");
        assert!(live.values().all(|samples| samples.len() == 1));

        // The WAL was filtered: replay reproduces the admitted state, not the
        // refusal-inflated one — the budget survives a restart mid-explosion.
        let replayed = SeriesMemTable::new();
        crate::journal::replay_with_signals(
            journal.wal_path(),
            journal.ckpt_path(),
            &crate::memtable::MemTable::new(),
            &crate::trace::TraceMemTable::new(),
            &replayed,
        )
        .unwrap();
        assert_eq!(replayed.sorted_samples(&test_tenant()).unwrap(), live);
        assert_eq!(replayed.active_series(&test_tenant()), 2);
    }

    #[tokio::test]
    async fn capacity_probe_bypasses_metric_cardinality_and_memory_admission() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-metric-probe-{}", uuid::Uuid::new_v4())),
            capacity_probe: true,
            max_active_series: Some(1),
            max_memtable_bytes: Some(1),
            max_wal_backlog_bytes: Some(1),
            max_inflight_push_bytes: Some(1),
            min_free_disk_bytes: None,
            ..Config::default()
        };
        let (ingest, series_memtable, _journal) = ingest_over(config);
        let accepted = ingest
            .accept(request_with(
                vec![Metric {
                    name: "queue_depth".to_string(),
                    data: Some(metric::Data::Gauge(Gauge {
                        data_points: vec![
                            gauge_point(now_ns(), 1.0, vec![attr("instance", "a")]),
                            gauge_point(now_ns(), 2.0, vec![attr("instance", "b")]),
                        ],
                    })),
                    ..Default::default()
                }],
                None,
            ))
            .await
            .expect("probe accepts beyond the configured metric guards");
        assert!(accepted.partial_success().is_none());
        assert_eq!(series_memtable.active_series(&test_tenant()), 2);
    }

    #[tokio::test]
    async fn an_export_of_only_new_series_past_the_cap_is_refused_whole() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-metric-429-{}", uuid::Uuid::new_v4())),
            max_active_series: Some(1),
            ..Config::default()
        };
        let (ingest, series_memtable, _journal) = ingest_over(config);
        let base = now_ns();
        ingest
            .accept(request_with(
                vec![Metric {
                    name: "queue_depth".to_string(),
                    data: Some(metric::Data::Gauge(Gauge {
                        data_points: vec![gauge_point(base, 1.0, vec![attr("instance", "a")])],
                    })),
                    ..Default::default()
                }],
                None,
            ))
            .await
            .unwrap();
        let all_new = request_with(
            vec![Metric {
                name: "queue_depth".to_string(),
                data: Some(metric::Data::Gauge(Gauge {
                    data_points: vec![gauge_point(base, 9.0, vec![attr("instance", "b")])],
                })),
                ..Default::default()
            }],
            None,
        );
        let error = ingest.accept(all_new).await.unwrap_err();
        assert_eq!(
            error.status,
            StatusCode::TOO_MANY_REQUESTS,
            "capacity returns at the idle horizon, so the refusal is retryable"
        );
        assert_eq!(series_memtable.active_series(&test_tenant()), 1);
    }

    #[tokio::test]
    async fn export_flushes_to_a_metric_part_and_reloads_after_restart() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-metric-flush-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        };
        let (ingest, series_memtable, journal) = ingest_over(config.clone());
        ingest.accept(small_request()).await.unwrap();
        let live = series_memtable.sorted_samples(&test_tenant()).unwrap();

        let registry = crate::part_registry::PartRegistry::new();
        let trace_registry = crate::trace_registry::TraceRegistry::new(registry.operation_lock());
        let series_registry =
            crate::series_registry::SeriesRegistry::new(registry.operation_lock());
        let mut pending_checkpoint = None;
        crate::flush::force_flush_pass(crate::flush::ForceFlush {
            memtable: &crate::memtable::MemTable::new(),
            trace_memtable: &journal.trace_memtable(),
            journal: &journal,
            registry: &registry,
            trace_registry: &trace_registry,
            series_registry: &series_registry,
            remote_cache: None,
            config: &config,
            pending_checkpoint: &mut pending_checkpoint,
        })
        .await
        .unwrap();
        assert!(series_memtable.is_empty(), "the samples left for the part");
        assert_eq!(
            series_memtable.active_series(&test_tenant()),
            0,
            "the index entry left with the samples: the part below carries the identity"
        );
        assert_eq!(series_registry.part_count(), 1);
        assert!(
            series_registry.tenant_stored_bytes(&test_tenant()) > 0,
            "the quota census sees the metric bytes"
        );

        // A restart discovers the part from disk and can read the samples back.
        let restored = crate::series_registry::SeriesRegistry::load_from_disk(
            &config.data_dir.join("metrics"),
            registry.operation_lock(),
        )
        .unwrap();
        assert_eq!(restored.part_count(), 1);
        let reader = restored.snapshot().into_iter().next().unwrap();
        let catalog = reader.tenant_catalog(&test_tenant());
        assert_eq!(catalog.len(), 1);
        let stored = reader.read_series(catalog.get(0).unwrap().chunk).unwrap();
        assert_eq!(&stored, live.values().next().unwrap());
        // And the WAL was retired: a replay after the flush puts nothing back.
        let replayed = SeriesMemTable::new();
        crate::journal::replay_with_signals(
            journal.wal_path(),
            journal.ckpt_path(),
            &crate::memtable::MemTable::new(),
            &crate::trace::TraceMemTable::new(),
            &replayed,
        )
        .unwrap();
        assert!(replayed.is_empty());
        std::fs::remove_dir_all(&config.data_dir).ok();
    }

    #[tokio::test]
    async fn export_rejects_a_timestamp_outside_the_window() {
        let config = Config {
            data_dir: std::env::temp_dir()
                .join(format!("signy-metric-window-{}", uuid::Uuid::new_v4())),
            ..Config::default()
        };
        let (ingest, series_memtable, _journal) = ingest_over(config);
        let request = request_with(
            vec![Metric {
                name: "queue_depth".to_string(),
                data: Some(metric::Data::Gauge(Gauge {
                    // Far past: outside any default max_timestamp_age.
                    data_points: vec![gauge_point(1, 7.5, vec![])],
                })),
                ..Default::default()
            }],
            None,
        );
        let error = ingest.accept(request).await.unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert!(series_memtable.is_empty());
    }
}
