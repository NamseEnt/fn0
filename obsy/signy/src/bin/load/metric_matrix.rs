//! The metric query matrix: the fn0 shapes, timed cold and warm, with the
//! per-answer records the agreement check runs on (M14, issue #8).
//!
//! The same discipline as the log matrix in `matrix.rs`, with one structural
//! difference the float domain forces. Log answers are compared by digest
//! alone, because two engines that agree return byte-comparable rows. Metric
//! answers come in **two declared classes**:
//!
//! * **exact** — raw samples and sum/count aggregations of them. Both engines
//!   return the floats the corpus stored (or sums of a few dozen of them), so
//!   the answers are compared on a digest of every value rounded to nine
//!   significant digits — far finer than any difference that would matter,
//!   coarse enough that summation order cannot flip it in the ordinary case.
//! * **tolerance** — `rate` and quantiles, where each engine's window
//!   arithmetic is its own (the rate definitions agree by decision, the
//!   floating-point path does not have to), so a digest equality would
//!   manufacture disagreement. Every answer therefore carries its full record
//!   set, and the report compares the two sides pointwise under
//!   `|a - b| <= max(1e-9, 0.005 * |b|)`, withholding the shape's ratios when
//!   the comparison fails — the same rule the log bed applies to a digest
//!   mismatch.
//!
//! The shapes are frozen *before* the engine exists to answer them — the
//! signy paths below are the read surface Phase 7 must implement, not a
//! description of one that runs today. That order is the point: the ruler is
//! built first, and the engine grows to fit it rather than the reverse.

use std::collections::BTreeMap;

use serde_json::{Value, json};
use tokio::time::Instant;

use crate::config::{Config, Target};
use crate::http::{Client, Request};
use crate::matrix::ns_to_sample_seconds;
use crate::metric_workload::{
    CHURN_COUNTER, COUNTER_NAMES, GAUGE_NAMES, HISTOGRAM_NAME, service_names,
};
use crate::stats::Series;

/// The six fn0 shapes, each named for the storage design difference it
/// exists to measure.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MetricShape {
    /// One gauge series over a window: pure chunk seek and decode, no
    /// arithmetic. The floor every other shape stands on.
    RawRange,
    /// A label-grouped sum across every steady service's instances: the
    /// dashboard's "by service" panel. Measures series selection fan-in and
    /// the group accumulator, not decode.
    AggSumBy,
    /// A counter `rate` over the window, per instance: the shape every
    /// dashboard's request-rate panel issues. Measures how the chunk layout
    /// serves windowed reads; the rate definition is the
    /// VictoriaMetrics-style positive-delta sum on both sides by decision.
    RateRange,
    /// The alert evaluation: the worst instance's error rate at one instant.
    /// One step, `agg=max` — measures the index-to-latest path a scheduler
    /// hammers, not a scan.
    InstantAlert,
    /// A p99 from bucketed histogram series: the decomposition's bill comes
    /// due here — the engine must select, window and merge `bounds + 1`
    /// bucket series per instrument and interpolate.
    QuantileP99,
    /// The claim's own shape: a selector over the churn service, whose
    /// instances the seeded dataset replaces generation by generation. The
    /// answer's series set must cross the replacement boundaries — an engine
    /// that lost the dead generations' series, or double-counts across a
    /// counter restart, shows here.
    ChurnedSelector,
}

pub const METRIC_SHAPES: [MetricShape; 6] = [
    MetricShape::RawRange,
    MetricShape::AggSumBy,
    MetricShape::RateRange,
    MetricShape::InstantAlert,
    MetricShape::QuantileP99,
    MetricShape::ChurnedSelector,
];

impl MetricShape {
    pub fn name(self) -> &'static str {
        match self {
            MetricShape::RawRange => "raw_range",
            MetricShape::AggSumBy => "agg_sum_by",
            MetricShape::RateRange => "rate_range",
            MetricShape::InstantAlert => "instant_alert",
            MetricShape::QuantileP99 => "quantile_p99",
            MetricShape::ChurnedSelector => "churned_selector",
        }
    }

    /// Which agreement rule the report must apply to this shape's answers.
    ///
    /// The names say what the class is *about*, because the first published
    /// run showed the obvious pair of names to be wrong. `exact` promised
    /// bit-equality on the shapes that only read stored numbers back — and
    /// bit-equality is unsatisfiable against VictoriaMetrics, which returns a
    /// decimal approximation of the double it was given (measured 2026-08-27:
    /// `250.07999999999998` in, `250.08` out, one ULP apart). So the two
    /// classes are now the two *sources* of difference: what an engine stored,
    /// and what an engine computed.
    pub fn digest_class(self) -> &'static str {
        match self {
            // Read back, not computed: the only licensed difference is each
            // engine's storage fidelity.
            MetricShape::RawRange | MetricShape::AggSumBy => "stored",
            // Each engine's own window arithmetic, which the rate definitions
            // agree on by decision but the floating-point path need not.
            MetricShape::RateRange
            | MetricShape::InstantAlert
            | MetricShape::QuantileP99
            | MetricShape::ChurnedSelector => "computed",
        }
    }
}

pub struct MetricQuery {
    pub id: String,
    pub shape: MetricShape,
    /// Evaluation instants this query's answers must not be compared at, as
    /// the canonical seconds the records carry.
    ///
    /// Only the churn shape has any. Its window deliberately straddles the
    /// points where a generation of series stops reporting, and that is
    /// exactly where the two engines' conventions differ by construction:
    /// this one answers the increase that arrived, VictoriaMetrics scales a
    /// partially-covered window to the full range (`QUERY_API.md`). Measured
    /// 2026-08-27: 6 of 118 steps, every one within `range` of a generation
    /// end, and the series set — what this shape exists to check — agreed on
    /// all 16. Exempting them by name keeps that check in force instead of
    /// widening a tolerance until a 3x difference passes.
    pub exempt_steps: Vec<String>,
    /// The human-readable form of the question — the MetricsQL text for the
    /// VictoriaMetrics side, the flat parameters for the first-party side —
    /// recorded so the report can print what was asked.
    pub description: String,
    pub start_ns: i64,
    pub end_ns: i64,
    pub instant: bool,
    pub path: String,
    pub step_ns: i64,
}

fn align_to_step(ns: i64, step_ns: i64) -> i64 {
    if step_ns <= 0 {
        return ns;
    }
    ns - ns.rem_euclid(step_ns)
}

fn seconds(ns: i64) -> i64 {
    ns.div_euclid(1_000_000_000)
}

/// `apps x sub-windows` queries per service-scoped shape, one query per
/// window for the corpus-wide shapes, one full-span query for the churn
/// shape — built so every query selects series that exist.
pub fn build_metric_queries(cfg: &Config) -> Vec<MetricQuery> {
    let verify = &cfg.metric_verify;
    let services = service_names(verify.services);
    let span = cfg.metric_span_ns();
    let windows = verify.windows as i64;
    let step_ns = verify.step_seconds * 1_000_000_000;
    let range_ns = verify.range_seconds * 1_000_000_000;
    let range = format!("{}s", verify.range_seconds);
    let step = format!("{}s", verify.step_seconds);
    let gauges: Vec<&str> = GAUGE_NAMES.iter().take(verify.gauges).copied().collect();
    let counters: Vec<&str> = COUNTER_NAMES
        .iter()
        .take(verify.counters)
        .copied()
        .collect();

    // No evaluation point may sit past the last sample.
    //
    // The corpus spans `scrapes * interval`, but its final *sample* is one
    // interval before that end, so a window ending at the span's end holds
    // less data than it asks for — and the two engines disagree there by
    // design: this one answers the increase that arrived, VictoriaMetrics
    // scales it up to the full window. Measured 2026-08-27: 40 of the 41
    // steps in such an answer agreed and only the last differed, by exactly
    // `range / covered`. That is a real semantic difference and it belongs in
    // `QUERY_API.md`, not in a bed that keeps asking a question whose answer
    // depends on which engine's edge convention you prefer. So the matrix
    // stops at the last sample — the same move the log bed makes for its
    // `rate` shape, and for the same reason.
    let last_sample_ns = verify.anchor_ns
        + (verify.scrapes.saturating_sub(1)) as i64
            * verify.scrape_interval_seconds
            * 1_000_000_000;
    let window_bounds = |window: i64| {
        let start = align_to_step(verify.anchor_ns + span * window / windows, step_ns);
        let end = align_to_step(verify.anchor_ns + span * (window + 1) / windows, step_ns);
        (start, end.min(align_to_step(last_sample_ns, step_ns)))
    };

    // Where a churn generation stops reporting. Every step within `range`
    // after one of these is a window the two engines answer differently by
    // construction; the comparison exempts them by name.
    let generation_ends: Vec<i64> = (1..=verify.churn_generations.max(1) as i64)
        .map(|generation| {
            verify.anchor_ns
                + (verify.scrapes as i64 * generation / verify.churn_generations.max(1) as i64)
                    * verify.scrape_interval_seconds
                    * 1_000_000_000
        })
        .collect();

    let mut queries = Vec::new();
    let mut push = |shape: MetricShape,
                    id: String,
                    start_ns: i64,
                    end_ns: i64,
                    instant: bool,
                    description: String,
                    path: String| {
        let exempt_steps = if shape == MetricShape::ChurnedSelector {
            let mut steps = Vec::new();
            let mut at = start_ns;
            while at <= end_ns {
                if generation_ends
                    .iter()
                    .any(|end| at >= *end && at - *end <= range_ns)
                {
                    steps.push(canonical_seconds_ns(at));
                }
                at += step_ns;
            }
            steps
        } else {
            Vec::new()
        };
        queries.push(MetricQuery {
            id,
            shape,
            description,
            start_ns,
            end_ns,
            instant,
            path,
            step_ns,
            exempt_steps,
        });
    };

    for shape in METRIC_SHAPES {
        match shape {
            MetricShape::RawRange => {
                for (index, service) in services.iter().enumerate() {
                    for window in 0..windows {
                        let (start, end) = window_bounds(window);
                        let gauge = gauges[(index + window as usize) % gauges.len()];
                        let selector =
                            format!("{gauge}{{service=\"{service}\",instance=\"instance-0\"}}");
                        let (description, path) = match cfg.target {
                            Target::Signy => first_party_range(
                                gauge,
                                &[
                                    ("attr", format!("service={service}")),
                                    ("attr", "instance=instance-0".to_string()),
                                ],
                                start,
                                end,
                                &step,
                            ),
                            _ => victoriametrics_range(&selector, start, end, verify.step_seconds),
                        };
                        push(
                            shape,
                            format!("raw_range/{service}/w{window}"),
                            start,
                            end,
                            false,
                            description,
                            path,
                        );
                    }
                }
            }
            MetricShape::AggSumBy => {
                for window in 0..windows {
                    let (start, end) = window_bounds(window);
                    let gauge = gauges[window as usize % gauges.len()];
                    let expression = format!("sum by (service) ({gauge})");
                    let (description, path) = match cfg.target {
                        Target::Signy => first_party_range(
                            gauge,
                            &[("agg", "sum".to_string()), ("by", "service".to_string())],
                            start,
                            end,
                            &step,
                        ),
                        _ => victoriametrics_range(&expression, start, end, verify.step_seconds),
                    };
                    push(
                        shape,
                        format!("agg_sum_by/all/w{window}"),
                        start,
                        end,
                        false,
                        description,
                        path,
                    );
                }
            }
            MetricShape::RateRange => {
                for (index, service) in services.iter().enumerate() {
                    for window in 0..windows {
                        // The first evaluation point starts one range in, so no
                        // window reaches before the dataset for its deltas.
                        let (start, end) = window_bounds(window);
                        let start = if window == 0 { start + range_ns } else { start };
                        let counter = counters[(index + window as usize) % counters.len()];
                        let expression =
                            format!("rate({counter}{{service=\"{service}\"}}[{range}])");
                        let (description, path) = match cfg.target {
                            Target::Signy => first_party_range(
                                counter,
                                &[
                                    ("attr", format!("service={service}")),
                                    ("func", "rate".to_string()),
                                    ("range", range.clone()),
                                ],
                                start,
                                end,
                                &step,
                            ),
                            _ => {
                                victoriametrics_range(&expression, start, end, verify.step_seconds)
                            }
                        };
                        push(
                            shape,
                            format!("rate_range/{service}/w{window}"),
                            start,
                            end,
                            false,
                            description,
                            path,
                        );
                    }
                }
            }
            MetricShape::InstantAlert => {
                for (index, service) in services.iter().enumerate() {
                    for window in 0..windows {
                        let (_, end) = window_bounds(window);
                        let counter = counters[index % counters.len()];
                        let expression =
                            format!("max(rate({counter}{{service=\"{service}\"}}[{range}]))");
                        let (description, path) = match cfg.target {
                            Target::Signy => first_party_instant(
                                counter,
                                &[
                                    ("attr", format!("service={service}")),
                                    ("func", "rate".to_string()),
                                    ("range", range.clone()),
                                    ("agg", "max".to_string()),
                                ],
                                end,
                            ),
                            _ => victoriametrics_instant(&expression, end),
                        };
                        push(
                            shape,
                            format!("instant_alert/{service}/w{window}"),
                            end,
                            end,
                            true,
                            description,
                            path,
                        );
                    }
                }
            }
            MetricShape::QuantileP99 => {
                for service in &services {
                    for window in 0..windows {
                        let (start, end) = window_bounds(window);
                        let start = if window == 0 { start + range_ns } else { start };
                        // `sum by` every non-`le` label the bucket series
                        // carry, so the grouping equals the first-party
                        // default: one quantile per underlying instrument.
                        let expression = format!(
                            "histogram_quantile(0.99, sum by (le, service, instance, env) \
(rate({HISTOGRAM_NAME}_bucket{{service=\"{service}\"}}[{range}])))"
                        );
                        let (description, path) = match cfg.target {
                            Target::Signy => first_party_range_at(
                                "/signy/api/v1/metrics/quantile",
                                HISTOGRAM_NAME,
                                &[
                                    ("q", "0.99".to_string()),
                                    ("attr", format!("service={service}")),
                                    ("range", range.clone()),
                                ],
                                start,
                                end,
                                &step,
                            ),
                            _ => {
                                victoriametrics_range(&expression, start, end, verify.step_seconds)
                            }
                        };
                        push(
                            shape,
                            format!("quantile_p99/{service}/w{window}"),
                            start,
                            end,
                            false,
                            description,
                            path,
                        );
                    }
                }
            }
            MetricShape::ChurnedSelector => {
                let start = align_to_step(verify.anchor_ns, step_ns) + range_ns;
                let end = align_to_step(last_sample_ns, step_ns);
                let expression = format!("sum by (instance) (rate({CHURN_COUNTER}[{range}]))");
                let (description, path) = match cfg.target {
                    Target::Signy => first_party_range(
                        CHURN_COUNTER,
                        &[
                            ("func", "rate".to_string()),
                            ("range", range.clone()),
                            ("agg", "sum".to_string()),
                            ("by", "instance".to_string()),
                        ],
                        start,
                        end,
                        &step,
                    ),
                    _ => victoriametrics_range(&expression, start, end, verify.step_seconds),
                };
                push(
                    shape,
                    "churned_selector/all/w0".to_string(),
                    start,
                    end,
                    false,
                    description,
                    path,
                );
            }
        }
    }
    queries
}

fn first_party_range(
    metric: &str,
    params: &[(&str, String)],
    start_ns: i64,
    end_ns: i64,
    step: &str,
) -> (String, String) {
    first_party_range_at(
        "/signy/api/v1/metrics/query",
        metric,
        params,
        start_ns,
        end_ns,
        step,
    )
}

fn first_party_range_at(
    route: &str,
    metric: &str,
    params: &[(&str, String)],
    start_ns: i64,
    end_ns: i64,
    step: &str,
) -> (String, String) {
    let mut encoded = url::form_urlencoded::Serializer::new(String::new());
    encoded.append_pair("metric", metric);
    for (name, value) in params {
        encoded.append_pair(name, value);
    }
    encoded.append_pair("start", &ns_to_sample_seconds(start_ns));
    encoded.append_pair("end", &ns_to_sample_seconds(end_ns));
    encoded.append_pair("step", step);
    let query = encoded.finish();
    (format!("{route}?{query}"), format!("{route}?{query}"))
}

fn first_party_instant(metric: &str, params: &[(&str, String)], at_ns: i64) -> (String, String) {
    let mut encoded = url::form_urlencoded::Serializer::new(String::new());
    encoded.append_pair("metric", metric);
    for (name, value) in params {
        encoded.append_pair(name, value);
    }
    encoded.append_pair("at", &ns_to_sample_seconds(at_ns));
    let query = encoded.finish();
    let path = format!("/signy/api/v1/metrics/instant?{query}");
    (path.clone(), path)
}

fn victoriametrics_range(
    expression: &str,
    start_ns: i64,
    end_ns: i64,
    step_seconds: i64,
) -> (String, String) {
    let encoded = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("query", expression)
        .append_pair("start", &seconds(start_ns).to_string())
        .append_pair("end", &seconds(end_ns).to_string())
        .append_pair("step", &format!("{step_seconds}s"))
        .finish();
    (
        expression.to_string(),
        format!("/api/v1/query_range?{encoded}"),
    )
}

fn victoriametrics_instant(expression: &str, at_ns: i64) -> (String, String) {
    let encoded = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("query", expression)
        .append_pair("time", &seconds(at_ns).to_string())
        .finish();
    (expression.to_string(), format!("/api/v1/query?{encoded}"))
}

/// What a metric response contained, reduced to something two runs can be
/// compared on.
pub struct MetricAnswer {
    pub kind: String,
    pub series: u64,
    pub points: u64,
    /// Whether every series' samples came back in ascending time — a metric
    /// range's one ordering contract on both response schemas.
    pub ordered: bool,
    /// One record per sample as `labels\u{1}seconds\u{1}value`, plus one
    /// `series\u{1}labels` record per series identity, sorted. Values keep
    /// their full shortest-round-trip rendering: this is the material the
    /// report's tolerance comparison reads, so rounding here would launder
    /// the difference it exists to find. `__name__` is dropped — the query
    /// itself named the metric, and the engines disagree by design on whether
    /// a function's result keeps it.
    pub records: Vec<String>,
    /// Digest over the records with values rounded to nine significant
    /// digits — the exact class's comparison, and every answer's warm-repeat
    /// stability check.
    pub digest: String,
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Nine significant digits, sign and scale preserved: the exact class's
/// canonical value form.
fn round_sig(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    if !value.is_finite() {
        return format!("{value}");
    }
    format!("{value:.8e}")
}

/// A canonical seconds rendering for a sample timestamp: whole seconds print
/// bare, fractional ones keep their fraction with trailing zeros trimmed.
fn canonical_seconds_ns(ns: i64) -> String {
    let whole = ns.div_euclid(1_000_000_000);
    let fraction = ns.rem_euclid(1_000_000_000);
    if fraction == 0 {
        return whole.to_string();
    }
    let text = format!("{whole}.{fraction:09}");
    text.trim_end_matches('0').to_string()
}

fn canonical_seconds_value(value: &Value) -> Result<String, String> {
    let seconds = value
        .as_f64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        .ok_or_else(|| format!("a sample timestamp is not a number: {value}"))?;
    let ns = (seconds * 1e9).round() as i64;
    Ok(canonical_seconds_ns(ns))
}

fn canonical_value(value: &Value) -> Result<(String, String), String> {
    let number = value
        .as_f64()
        .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        .ok_or_else(|| format!("a metric sample is not a number: {value}"))?;
    Ok((format!("{number:?}"), round_sig(number)))
}

fn canonical_labels(labels: &BTreeMap<String, String>) -> String {
    labels
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn labels_of(value: &Value) -> Result<BTreeMap<String, String>, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "a result's labels are not an object".to_string())?;
    let mut labels = BTreeMap::new();
    for (name, value) in object {
        if name == "__name__" {
            continue;
        }
        let text = value
            .as_str()
            .ok_or_else(|| format!("label '{name}' is not a string: {value}"))?;
        labels.insert(name.clone(), text.to_string());
    }
    Ok(labels)
}

struct RecordState {
    records: Vec<String>,
    rounded: Vec<String>,
    points: u64,
    ordered: bool,
}

impl RecordState {
    fn new() -> Self {
        Self {
            records: Vec::new(),
            rounded: Vec::new(),
            points: 0,
            ordered: true,
        }
    }

    fn push_series(&mut self, identity: &str) {
        self.records.push(format!("series\u{1}{identity}"));
        self.rounded.push(format!("series\u{1}{identity}"));
    }

    fn push_sample(&mut self, identity: &str, at: &str, full: &str, rounded: &str) {
        self.records.push(format!("{identity}\u{1}{at}\u{1}{full}"));
        self.rounded
            .push(format!("{identity}\u{1}{at}\u{1}{rounded}"));
        self.points += 1;
    }

    fn finish(mut self, kind: String, series: u64) -> MetricAnswer {
        self.records.sort();
        self.rounded.sort();
        let mut hash = fnv1a64(b"metric");
        for record in &self.rounded {
            hash ^= fnv1a64(record.as_bytes());
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        MetricAnswer {
            kind,
            series,
            points: self.points,
            ordered: self.ordered,
            records: self.records,
            digest: format!("{hash:016x}"),
        }
    }
}

/// The Prometheus-API JSON envelope VictoriaMetrics answers with: `matrix`
/// results carry `values`, `vector` results carry one `value`.
pub fn digest_prometheus_response(body: &[u8]) -> Result<MetricAnswer, String> {
    let parsed: Value =
        serde_json::from_slice(body).map_err(|error| format!("response is not JSON: {error}"))?;
    let status = parsed["status"].as_str().unwrap_or("missing");
    if status != "success" {
        return Err(format!(
            "response status is '{status}': {}",
            String::from_utf8_lossy(&body[..body.len().min(300)])
        ));
    }
    let kind = parsed["data"]["resultType"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let entries = parsed["data"]["result"]
        .as_array()
        .ok_or_else(|| "response has no data.result array".to_string())?;
    let mut state = RecordState::new();
    for entry in entries {
        let identity = canonical_labels(&labels_of(&entry["metric"])?);
        state.push_series(&identity);
        let values = if let Some(values) = entry["values"].as_array() {
            values.clone()
        } else if entry["value"].is_array() {
            vec![entry["value"].clone()]
        } else {
            return Err("a result carries neither values nor value".to_string());
        };
        let mut previous: Option<f64> = None;
        for pair in values {
            let pair = pair
                .as_array()
                .ok_or_else(|| "a values element is not an array".to_string())?;
            if pair.len() != 2 {
                return Err(format!(
                    "a values element has {} elements, expected [timestamp, value]",
                    pair.len()
                ));
            }
            let at_number = pair[0]
                .as_f64()
                .or_else(|| pair[0].as_str().and_then(|text| text.parse().ok()))
                .ok_or_else(|| format!("a sample timestamp is not a number: {}", pair[0]))?;
            if previous.is_some_and(|previous| at_number < previous) {
                state.ordered = false;
            }
            previous = Some(at_number);
            let at = canonical_seconds_value(&pair[0])?;
            let (full, rounded) = canonical_value(&pair[1])?;
            state.push_sample(&identity, &at, &full, &rounded);
        }
    }
    Ok(state.finish(kind, entries.len() as u64))
}

/// The first-party NDJSON the M14 read surface answers with: one line per
/// series — `{"labels":{...},"samples":[["<ns>",v],...]}` from the range
/// routes, `{"labels":{...},"timestamp":"<ns>","value":v}` from `/instant`.
///
/// Written before the engine exists, which is the ruler doing its job: this
/// parser is the response contract Phase 7 implements against.
pub fn digest_first_party_metric_response(body: &[u8]) -> Result<MetricAnswer, String> {
    let text =
        std::str::from_utf8(body).map_err(|error| format!("response is not UTF-8: {error}"))?;
    let mut state = RecordState::new();
    let mut series = 0u64;
    let mut instant = false;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let row: Value = serde_json::from_str(line)
            .map_err(|error| format!("response line is not JSON: {error}"))?;
        let object = row
            .as_object()
            .ok_or_else(|| "a response line is not a JSON object".to_string())?;
        let identity = canonical_labels(&labels_of(
            object
                .get("labels")
                .ok_or_else(|| "a series line has no labels object".to_string())?,
        )?);
        series += 1;
        state.push_series(&identity);
        if let Some(samples) = object.get("samples") {
            let samples = samples
                .as_array()
                .ok_or_else(|| "samples is not an array".to_string())?;
            let mut previous: Option<i64> = None;
            for pair in samples {
                let pair = pair
                    .as_array()
                    .ok_or_else(|| "a samples element is not an array".to_string())?;
                if pair.len() != 2 {
                    return Err(format!(
                        "a samples element has {} elements, expected [timestamp, value]",
                        pair.len()
                    ));
                }
                let ns: i64 = pair[0]
                    .as_str()
                    .and_then(|text| text.parse().ok())
                    .ok_or_else(|| {
                        format!("a sample timestamp is not a nanosecond string: {}", pair[0])
                    })?;
                if previous.is_some_and(|previous| ns < previous) {
                    state.ordered = false;
                }
                previous = Some(ns);
                let (full, rounded) = canonical_value(&pair[1])?;
                state.push_sample(&identity, &canonical_seconds_ns(ns), &full, &rounded);
            }
        } else {
            instant = true;
            let ns: i64 = object
                .get("timestamp")
                .and_then(Value::as_str)
                .and_then(|text| text.parse().ok())
                .ok_or_else(|| "an instant line has no nanosecond timestamp string".to_string())?;
            let value = object
                .get("value")
                .ok_or_else(|| "an instant line has no value".to_string())?;
            let (full, rounded) = canonical_value(value)?;
            state.push_sample(&identity, &canonical_seconds_ns(ns), &full, &rounded);
        }
    }
    Ok(state.finish(
        if instant { "vector" } else { "matrix" }.to_string(),
        series,
    ))
}

pub fn digest_metric_for(target: Target, body: &[u8]) -> Result<MetricAnswer, String> {
    match target {
        Target::Signy => digest_first_party_metric_response(body),
        Target::VictoriaMetrics => digest_prometheus_response(body),
        Target::Mimir => Err(format!(
            "target {} is ingest-only in the capacity bed; Mimir query comparison is out of scope",
            target.name()
        )),
        Target::Loki | Target::VictoriaLogs => Err(format!(
            "target {} answers the log matrix, not the metric one",
            target.name()
        )),
    }
}

struct Timing {
    cold_ms: f64,
    warm_ms: Vec<f64>,
    answer: Option<MetricAnswer>,
    warm_digests_agree: bool,
    status: u16,
    error: Option<String>,
}

async fn issue(
    client: &mut Client,
    query: &MetricQuery,
    tenant: (&str, &str),
) -> (f64, u16, Vec<u8>, Option<String>) {
    let sent = Instant::now();
    let result = client
        .request(&Request {
            method: "GET",
            path: &query.path,
            body: &[],
            content_type: "",
            tenant: Some(tenant),
            headers: &[],
        })
        .await;
    let elapsed = sent.elapsed().as_secs_f64() * 1000.0;
    match result {
        Ok(response) if response.status == 200 => (elapsed, 200, response.body, None),
        Ok(response) => {
            let detail = String::from_utf8_lossy(&response.body)
                .chars()
                .take(300)
                .collect::<String>();
            (elapsed, response.status, Vec::new(), Some(detail))
        }
        Err(error) => (elapsed, 0, Vec::new(), Some(error)),
    }
}

/// Cold pass over every query, then the warm repeats — the same separation,
/// for the same reason, as the log matrix.
pub async fn run_metric_matrix(cfg: &Config) -> Value {
    let (header, value) = cfg.target.read_tenant_header(&cfg.metric_verify.tenant);
    let tenant = (header, value.as_str());
    let queries = build_metric_queries(cfg);
    let mut client = Client::new(&cfg.http_address, cfg.request_timeout());
    let mut timings: Vec<Timing> = Vec::with_capacity(queries.len());

    for query in &queries {
        let (elapsed, status, body, error) = issue(&mut client, query, tenant).await;
        let answer = if status == 200 {
            match digest_metric_for(cfg.target, &body) {
                Ok(answer) => Some(answer),
                Err(reason) => {
                    timings.push(Timing {
                        cold_ms: elapsed,
                        warm_ms: Vec::new(),
                        answer: None,
                        warm_digests_agree: false,
                        status,
                        error: Some(reason),
                    });
                    continue;
                }
            }
        } else {
            None
        };
        timings.push(Timing {
            cold_ms: elapsed,
            warm_ms: Vec::new(),
            answer,
            warm_digests_agree: true,
            status,
            error,
        });
    }

    for _ in 0..cfg.metric_verify.repeats {
        for (index, query) in queries.iter().enumerate() {
            let (elapsed, status, body, error) = issue(&mut client, query, tenant).await;
            let timing = &mut timings[index];
            timing.warm_ms.push(elapsed);
            if status != 200 {
                timing.error.get_or_insert_with(|| {
                    error.unwrap_or_else(|| format!("warm repeat answered {status}"))
                });
                timing.warm_digests_agree = false;
                continue;
            }
            match (digest_metric_for(cfg.target, &body), timing.answer.as_ref()) {
                (Ok(repeat), Some(first)) if repeat.digest != first.digest => {
                    timing.warm_digests_agree = false;
                }
                (Err(reason), _) => {
                    timing.error.get_or_insert(reason);
                    timing.warm_digests_agree = false;
                }
                _ => {}
            }
        }
    }

    let mut per_shape = serde_json::Map::new();
    for shape in METRIC_SHAPES {
        let mut cold = Series::default();
        let mut warm = Series::default();
        let mut points = 0u64;
        let mut series = 0u64;
        let mut errors = 0u64;
        let mut unstable = 0u64;
        let mut out_of_order = 0u64;
        let mut example = String::new();
        for (query, timing) in queries.iter().zip(&timings) {
            if query.shape != shape {
                continue;
            }
            if example.is_empty() {
                example = query.description.clone();
            }
            if timing.error.is_some() || timing.answer.is_none() {
                errors += 1;
                continue;
            }
            cold.push(timing.cold_ms);
            for value in &timing.warm_ms {
                warm.push(*value);
            }
            points += timing.answer.as_ref().map_or(0, |answer| answer.points);
            series += timing.answer.as_ref().map_or(0, |answer| answer.series);
            unstable += u64::from(!timing.warm_digests_agree);
            out_of_order += u64::from(timing.answer.as_ref().is_some_and(|answer| !answer.ordered));
        }
        per_shape.insert(
            shape.name().to_string(),
            json!({
                "expression_example": example,
                "digest_class": shape.digest_class(),
                "queries": queries.iter().filter(|query| query.shape == shape).count(),
                "errors": errors,
                "points_returned_cold": points,
                "series_returned_cold": series,
                "warm_answers_differed": unstable,
                "answers_out_of_ascending_order": out_of_order,
                "cold_ms": cold.summary(),
                "warm_ms": warm.summary(),
            }),
        );
    }

    let answers: Vec<Value> = queries
        .iter()
        .zip(&timings)
        .map(|(query, timing)| {
            json!({
                "id": query.id,
                "shape": query.shape.name(),
                "digest_class": query.shape.digest_class(),
                "expression": query.description,
                "start_ns": query.start_ns,
                "end_ns": query.end_ns,
                "step_ns": query.step_ns,
                "exempt_steps": query.exempt_steps,
                "instant": query.instant,
                "status": timing.status,
                "error": timing.error,
                "cold_ms": timing.cold_ms,
                "warm_ms": timing.warm_ms,
                "warm_digests_agree": timing.warm_digests_agree,
                "result_type": timing.answer.as_ref().map(|answer| answer.kind.clone()),
                "series": timing.answer.as_ref().map(|answer| answer.series),
                "points": timing.answer.as_ref().map(|answer| answer.points),
                "ordered": timing.answer.as_ref().map(|answer| answer.ordered),
                "digest": timing.answer.as_ref().map(|answer| answer.digest.clone()),
                // The full record set, because the tolerance classes cannot be
                // compared through a digest; see the module doc.
                "records": timing.answer.as_ref().map(|answer| answer.records.clone()),
            })
        })
        .collect();

    json!({
        "shapes": Value::Object(per_shape),
        "answers": answers,
        "queries_issued": queries.len() as u64 * (1 + cfg.metric_verify.repeats as u64),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_for(target: Target) -> Config {
        let mut cfg = Config::from_env().expect("the env defaults build a config");
        cfg.target = target;
        cfg.metric_verify.anchor_ns = 1_772_000_000_000_000_000;
        cfg.metric_verify.scrapes = 36;
        cfg.metric_verify.services = 2;
        cfg
    }

    #[test]
    fn every_first_party_metric_path_is_a_flat_query_under_the_metrics_routes() {
        let queries = build_metric_queries(&config_for(Target::Signy));
        assert!(!queries.is_empty());
        for query in &queries {
            assert!(
                query.path.starts_with("/signy/api/v1/metrics/"),
                "{}",
                query.path
            );
            assert!(!query.path.contains("query="), "{}", query.path);
            assert!(query.path.contains("metric="), "{}", query.path);
            if query.instant {
                assert!(query.path.contains("at="), "{}", query.path);
            } else {
                assert!(query.path.contains("step=30s"), "{}", query.path);
            }
        }
        let quantiles: Vec<&MetricQuery> = queries
            .iter()
            .filter(|query| query.shape == MetricShape::QuantileP99)
            .collect();
        assert!(
            quantiles
                .iter()
                .all(|query| query.path.starts_with("/signy/api/v1/metrics/quantile?")),
            "quantile is its own route"
        );
    }

    #[test]
    fn every_victoriametrics_path_is_the_prometheus_api_with_metricsql() {
        let queries = build_metric_queries(&config_for(Target::VictoriaMetrics));
        for query in &queries {
            if query.instant {
                assert!(query.path.starts_with("/api/v1/query?"), "{}", query.path);
            } else {
                assert!(
                    query.path.starts_with("/api/v1/query_range?"),
                    "{}",
                    query.path
                );
            }
        }
        let churned = queries
            .iter()
            .find(|query| query.shape == MetricShape::ChurnedSelector)
            .expect("the churn shape exists");
        assert!(churned.description.contains("churn_requests_total"));
    }

    #[test]
    fn the_same_answer_in_both_response_shapes_digests_equal() {
        // 1_772_000_000s, one series, two samples.
        let prometheus = br#"{"status":"success","data":{"resultType":"matrix","result":[
            {"metric":{"__name__":"queue_depth","service":"api","env":"prod"},
             "values":[[1772000000,"12.5"],[1772000030,"13"]]}]}}"#;
        let first_party = br#"{"labels":{"service":"api","env":"prod"},"samples":[["1772000000000000000",12.5],["1772000030000000000",13.0]]}"#;
        let prometheus = digest_prometheus_response(prometheus).expect("valid");
        let first_party = digest_first_party_metric_response(first_party).expect("valid");
        assert_eq!(
            prometheus.digest, first_party.digest,
            "__name__ is dropped and the record bases coincide"
        );
        assert_eq!(prometheus.points, 2);
        assert_eq!(first_party.series, 1);
    }

    #[test]
    fn nine_significant_digits_absorb_rendering_and_not_differences() {
        let one = br#"{"status":"success","data":{"resultType":"matrix","result":[
            {"metric":{},"values":[[1772000000,"0.0666666666666667"]]}]}}"#;
        let same = br#"{"status":"success","data":{"resultType":"matrix","result":[
            {"metric":{},"values":[[1772000000,"0.06666666666666671"]]}]}}"#;
        let different = br#"{"status":"success","data":{"resultType":"matrix","result":[
            {"metric":{},"values":[[1772000000,"0.0666667"]]}]}}"#;
        assert_eq!(
            digest_prometheus_response(one).expect("valid").digest,
            digest_prometheus_response(same).expect("valid").digest
        );
        assert_ne!(
            digest_prometheus_response(one).expect("valid").digest,
            digest_prometheus_response(different).expect("valid").digest
        );
    }

    #[test]
    fn an_extra_or_empty_series_cannot_hide() {
        let one = br#"{"status":"success","data":{"resultType":"matrix","result":[
            {"metric":{"service":"api"},"values":[[1772000000,"1"]]}]}}"#;
        let plus_empty = br#"{"status":"success","data":{"resultType":"matrix","result":[
            {"metric":{"service":"api"},"values":[[1772000000,"1"]]},
            {"metric":{"service":"gone"},"values":[]}]}}"#;
        assert_ne!(
            digest_prometheus_response(one).expect("valid").digest,
            digest_prometheus_response(plus_empty)
                .expect("valid")
                .digest,
            "the churn shape's dead generations must be visible as series records"
        );
    }

    #[test]
    fn descending_samples_are_a_finding_not_a_different_answer() {
        let ascending = br#"{"labels":{},"samples":[["1000000000",1.0],["2000000000",2.0]]}"#;
        let descending = br#"{"labels":{},"samples":[["2000000000",2.0],["1000000000",1.0]]}"#;
        let ascending = digest_first_party_metric_response(ascending).expect("valid");
        let descending = digest_first_party_metric_response(descending).expect("valid");
        assert!(ascending.ordered);
        assert!(!descending.ordered);
        assert_eq!(
            ascending.digest, descending.digest,
            "the digest stays order-independent; the ordering has its own flag"
        );
    }

    #[test]
    fn a_failed_envelope_or_malformed_line_is_an_error_not_an_empty_answer() {
        assert!(digest_prometheus_response(b"not json").is_err());
        assert!(digest_prometheus_response(br#"{"status":"error","error":"boom"}"#).is_err());
        assert!(
            digest_first_party_metric_response(br#"{"samples":[["1",1.0]]}"#).is_err(),
            "a series line without labels is malformed"
        );
        assert!(
            digest_first_party_metric_response(br#"{"labels":{},"samples":[[1,1.0]]}"#).is_err(),
            "a numeric timestamp is not the ns-string contract"
        );
    }

    #[test]
    fn an_instant_line_and_a_vector_answer_share_the_record_basis() {
        let prometheus = br#"{"status":"success","data":{"resultType":"vector","result":[
            {"metric":{"service":"api"},"value":[1772000000,"0.97"]}]}}"#;
        let first_party =
            br#"{"labels":{"service":"api"},"timestamp":"1772000000000000000","value":0.97}"#;
        assert_eq!(
            digest_prometheus_response(prometheus)
                .expect("valid")
                .digest,
            digest_first_party_metric_response(first_party)
                .expect("valid")
                .digest
        );
    }

    /// The bed must not ask a question whose window reaches past the last
    /// sample: that is where the two engines' edge conventions differ, and a
    /// difference in conventions is not a difference in engines.
    #[test]
    fn no_evaluation_point_sits_past_the_last_sample() {
        let cfg = config_for(Target::Signy);
        let verify = &cfg.metric_verify;
        let last_sample_ns = verify.anchor_ns
            + (verify.scrapes as i64 - 1) * verify.scrape_interval_seconds * 1_000_000_000;
        let queries = build_metric_queries(&cfg);
        assert!(!queries.is_empty());
        for query in &queries {
            assert!(
                query.end_ns <= last_sample_ns,
                "{} evaluates at {} which is past the last sample at {last_sample_ns}",
                query.id,
                query.end_ns
            );
            assert!(
                query.start_ns <= query.end_ns,
                "{} has an inverted window",
                query.id
            );
        }
    }

    /// The exemption is derived from the workload's generation boundaries, not
    /// from the answers — a comparison that decided what to skip by looking at
    /// what disagreed would license every difference it found.
    #[test]
    fn only_the_churn_shape_exempts_instants_and_only_at_generation_ends() {
        let cfg = config_for(Target::Signy);
        let verify = &cfg.metric_verify;
        let range_ns = verify.range_seconds * 1_000_000_000;
        let ends: Vec<i64> = (1..=verify.churn_generations as i64)
            .map(|generation| {
                verify.anchor_ns
                    + (verify.scrapes as i64 * generation / verify.churn_generations as i64)
                        * verify.scrape_interval_seconds
                        * 1_000_000_000
            })
            .collect();
        let queries = build_metric_queries(&cfg);
        let mut churn_exemptions = 0;
        for query in &queries {
            if query.shape != MetricShape::ChurnedSelector {
                assert!(
                    query.exempt_steps.is_empty(),
                    "{} exempts instants but is not the churn shape",
                    query.id
                );
                continue;
            }
            churn_exemptions += query.exempt_steps.len();
            for step in &query.exempt_steps {
                let seconds: i64 = step.parse().expect("an exempt step is whole seconds");
                let at = seconds * 1_000_000_000;
                assert!(
                    ends.iter().any(|end| at >= *end && at - *end <= range_ns),
                    "{step} is exempted but is not within one range of a generation end"
                );
            }
        }
        assert!(
            churn_exemptions > 0,
            "the churn shape crosses generation ends, so it must exempt some instants"
        );
    }

    #[test]
    fn the_shape_list_and_its_classes_are_frozen() {
        let names: Vec<&str> = METRIC_SHAPES.iter().map(|shape| shape.name()).collect();
        assert_eq!(
            names,
            vec![
                "raw_range",
                "agg_sum_by",
                "rate_range",
                "instant_alert",
                "quantile_p99",
                "churned_selector"
            ]
        );
        assert_eq!(MetricShape::RawRange.digest_class(), "stored");
        assert_eq!(MetricShape::AggSumBy.digest_class(), "stored");
        for shape in [
            MetricShape::RateRange,
            MetricShape::InstantAlert,
            MetricShape::QuantileP99,
            MetricShape::ChurnedSelector,
        ] {
            assert_eq!(shape.digest_class(), "computed");
        }
    }
}
