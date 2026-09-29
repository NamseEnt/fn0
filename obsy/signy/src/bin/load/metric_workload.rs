//! The fixed metric dataset the metrics comparison runs on (M14, issue #8).
//!
//! The same rule as the log seed in `matrix.rs`: the query comparison must run
//! over data both systems provably hold, so this corpus is a pure function of
//! the run seed and the anchor — same seed, same series, same values, same
//! timestamps, and the identical OTLP protobuf bytes go to every target.
//!
//! Two decisions the honesty of the bed rests on, made here and stated here:
//!
//! * **Every identity label rides as a datapoint attribute**, and no resource
//!   attributes are sent. Engines disagree by *policy* on what to do with OTLP
//!   resource attributes (promotion lists, `job`/`instance` synthesis,
//!   dropping), and a bed that exercised those defaults would measure schema
//!   policy rather than storage. Datapoint attributes become labels verbatim
//!   on every engine that stores metrics at all, so the comparison stays about
//!   the engines' storage and executors. The engines' resource-attribute
//!   behavior is their own surface, tested engine-side, not bed-side.
//! * **The churn block is data, not load.** The churn *service*'s instances
//!   are replaced generation by generation across the scrape range, so the
//!   seeded dataset itself contains series that begin and end mid-span — the
//!   pod-restart shape. The `churned_selector` query shape must cross those
//!   boundaries at read time. The ingest-time churn phases (an engine
//!   surviving series churn under a memory limit) are a separate, paced
//!   workload and are not this file's business.
//!
//! The instrument vocabulary, the histogram bounds and the churn layout are
//! constants rather than knobs: the dataset's shape is part of the ruler, and
//! a knob per axis is an invitation to tune the corpus toward whichever
//! engine a run is flattering.

use std::collections::BTreeMap;
use std::sync::Arc;

use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
use opentelemetry_proto::tonic::metrics::v1::{
    AggregationTemporality, Gauge, Histogram, HistogramDataPoint, Metric, NumberDataPoint,
    ResourceMetrics, ScopeMetrics, Sum, metric, number_data_point,
};
use prost014::Message;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::config::{Config, MetricVerify, Signal, Target};
use crate::http::{Client, Request};

const PUSH_CONTENT_TYPE: &str = "application/x-protobuf";

/// Keeps the metric dataset's value streams distinct from the log corpus's
/// while staying a function of the run seed.
const METRIC_SEED_SALT: u64 = 0x9e_7f_2c_a5;

pub const GAUGE_NAMES: [&str; 6] = [
    "process_resident_memory_bytes",
    "queue_depth",
    "cpu_usage_ratio",
    "open_connections",
    "heap_inuse_bytes",
    "worker_utilization_ratio",
];

pub const COUNTER_NAMES: [&str; 6] = [
    "http_requests_total",
    "http_errors_total",
    "bytes_sent_total",
    "tasks_completed_total",
    "cache_misses_total",
    "retries_total",
];

pub const HISTOGRAM_NAME: &str = "http_request_duration_seconds";

/// Classic explicit bounds, seconds. Eight bounds means eleven decomposed
/// series per instrument (`_bucket` per bound plus `+Inf`, `_sum`, `_count`) —
/// the decomposition multiplier the workload deliberately carries, because it
/// is the multiplier the engine's own histogram decomposition pays.
pub const HISTOGRAM_BOUNDS: [f64; 8] = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0];

/// The churn service's name. Not in the steady `service` vocabulary, so a
/// query over it selects exactly the replaced-series population.
pub const CHURN_SERVICE: &str = "churnsvc";
pub const CHURN_COUNTER: &str = "churn_requests_total";
pub const CHURN_GAUGE: &str = "churn_queue_depth";

/// The steady `service` label vocabulary — the log corpus's app names, reused
/// so the two beds read as one family of datasets.
pub fn service_names(count: usize) -> Vec<String> {
    (0..count)
        .map(|index| signy::corpus::APPS[index % signy::corpus::APPS.len()].to_string())
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InstrumentKind {
    Gauge,
    Counter,
    Histogram,
}

/// One OTLP instrument: a metric name under one label set, reporting in one
/// contiguous slice of the scrape range.
pub struct Instrument {
    pub metric: String,
    pub kind: InstrumentKind,
    /// Sorted datapoint attributes; the series identity below the name.
    pub labels: Vec<(String, String)>,
    /// Scrape indices `[first, last)` this instrument reports in. Steady
    /// instruments cover the whole range; churn instruments cover their
    /// generation's slice.
    pub first_scrape: usize,
    pub last_scrape: usize,
    seed: u64,
    /// The live path's running cumulative totals. A real exporter reports a
    /// running total, and the paced path used to rebuild one by summing every
    /// increment from scrape zero on every scrape — quadratic, which a bed
    /// measured in minutes never felt and a soak measured in hours does: at a
    /// 10 s interval a day is 8,640 scrapes, and the harness starves the
    /// server of load long before the day ends. The totals are advanced once
    /// per scrape instead; an instrument minted mid-run pays the catch-up sum
    /// once, so the values it reports are the ones it always reported.
    totals: Totals,
}

/// One instrument's cumulative state on the live path.
#[derive(Default)]
struct Totals {
    /// The last scrape folded in, so a repeated or skipped index cannot
    /// double-count or leave a hole.
    advanced_through: Option<usize>,
    counter: u64,
    buckets: Vec<u64>,
    sum: f64,
}

pub struct MetricCorpus {
    pub tenant: String,
    pub anchor_ns: i64,
    pub scrape_interval_ns: i64,
    pub scrapes: usize,
    pub instruments: Vec<Instrument>,
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn hash_str(state: &mut u64, text: &str) {
    for byte in text.as_bytes() {
        *state = (*state ^ *byte as u64).wrapping_mul(0x0000_0100_0000_01b3);
    }
    splitmix64(state);
}

/// One draw for (instrument, scrape, lane), independent of every other draw.
fn draw(seed: u64, scrape: usize, lane: u64) -> u64 {
    let mut state = seed ^ (scrape as u64).wrapping_mul(0xa076_1d64_78bd_642f) ^ lane;
    splitmix64(&mut state)
}

impl Instrument {
    pub fn active_at(&self, scrape: usize) -> bool {
        scrape >= self.first_scrape && scrape < self.last_scrape
    }

    /// Gauge sample at a scrape: two decimal places over a per-series base, so
    /// the values are unremarkable floats both engines store bit-identically.
    pub fn gauge_value(&self, scrape: usize) -> f64 {
        let base = (self.seed % 900) as f64;
        base + (draw(self.seed, scrape, 1) % 10_000) as f64 / 100.0
    }

    /// Counter increment between consecutive scrapes: a small integer, so the
    /// cumulative total stays exactly representable for the whole run.
    pub fn counter_increment(&self, scrape: usize) -> u64 {
        draw(self.seed, scrape, 2) % 50
    }

    /// Per-bucket count increment between consecutive scrapes.
    pub fn bucket_increment(&self, scrape: usize, bucket: usize) -> u64 {
        draw(self.seed, scrape, 3 + bucket as u64) % 20
    }

    /// The histogram sum's increment: hundredths, deterministic.
    pub fn sum_increment(&self, scrape: usize) -> f64 {
        (draw(self.seed, scrape, 99) % 10_000) as f64 / 100.0
    }

    /// Fold every scrape up to and including `scrape` that is not folded in
    /// yet. The steady path folds exactly one; an instrument minted at scrape
    /// N folds N + 1 the first time, which is the same arithmetic the summing
    /// version did and therefore the same reported totals.
    fn advance_to(&mut self, scrape: usize) {
        let from = match self.totals.advanced_through {
            Some(last) if last >= scrape => return,
            Some(last) => last + 1,
            None => 0,
        };
        if self.kind == InstrumentKind::Histogram && self.totals.buckets.is_empty() {
            self.totals.buckets = vec![0; HISTOGRAM_BOUNDS.len() + 1];
        }
        for at in from..=scrape {
            match self.kind {
                InstrumentKind::Gauge => {}
                InstrumentKind::Counter => {
                    self.totals.counter += self.counter_increment(at);
                }
                InstrumentKind::Histogram => {
                    // Taken out and put back so the increment stays defined in
                    // exactly one place: a second copy of the formula here is
                    // a second thing to keep in step with the seeded corpus.
                    let mut buckets = std::mem::take(&mut self.totals.buckets);
                    for (bucket, total) in buckets.iter_mut().enumerate() {
                        *total += self.bucket_increment(at, bucket);
                    }
                    self.totals.buckets = buckets;
                    self.totals.sum += self.sum_increment(at);
                }
            }
        }
        self.totals.advanced_through = Some(scrape);
    }
}

fn instrument(
    spec_seed: u64,
    metric: &str,
    kind: InstrumentKind,
    labels: &[(&str, &str)],
) -> Instrument {
    let mut labels: Vec<(String, String)> = labels
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_string()))
        .collect();
    labels.sort();
    let mut seed = spec_seed ^ METRIC_SEED_SALT;
    hash_str(&mut seed, metric);
    for (name, value) in &labels {
        hash_str(&mut seed, name);
        hash_str(&mut seed, value);
    }
    Instrument {
        metric: metric.to_string(),
        kind,
        labels,
        first_scrape: 0,
        last_scrape: usize::MAX,
        seed,
        totals: Totals::default(),
    }
}

/// The whole dataset, from the run seed and the metric knobs alone.
pub fn metric_corpus(seed: u64, verify: &MetricVerify) -> MetricCorpus {
    let services = service_names(verify.services);
    let mut instruments = Vec::new();
    for service in &services {
        for index in 0..verify.instances_per_service {
            let instance = format!("instance-{index}");
            let env = if index % 2 == 0 { "prod" } else { "staging" };
            let labels: [(&str, &str); 3] = [
                ("service", service.as_str()),
                ("instance", instance.as_str()),
                ("env", env),
            ];
            for gauge in GAUGE_NAMES.iter().take(verify.gauges) {
                instruments.push(instrument(seed, gauge, InstrumentKind::Gauge, &labels));
            }
            for counter in COUNTER_NAMES.iter().take(verify.counters) {
                instruments.push(instrument(seed, counter, InstrumentKind::Counter, &labels));
            }
            instruments.push(instrument(
                seed,
                HISTOGRAM_NAME,
                InstrumentKind::Histogram,
                &labels,
            ));
        }
    }
    // The churn block: each generation's instances exist only in its slice of
    // the scrape range. Slices tile the range, so at any instant exactly
    // `churn_instances` of them are live and every generation boundary is a
    // replacement event the `churned_selector` query must read across.
    let generations = verify.churn_generations.max(1);
    for generation in 0..generations {
        let first = verify.scrapes * generation / generations;
        let last = verify.scrapes * (generation + 1) / generations;
        for index in 0..verify.churn_instances {
            let instance = format!("churn-{generation}-{index}");
            let labels: [(&str, &str); 3] = [
                ("service", CHURN_SERVICE),
                ("instance", instance.as_str()),
                ("env", "prod"),
            ];
            for (metric, kind) in [
                (CHURN_COUNTER, InstrumentKind::Counter),
                (CHURN_GAUGE, InstrumentKind::Gauge),
            ] {
                let mut instrument = instrument(seed, metric, kind, &labels);
                instrument.first_scrape = first;
                instrument.last_scrape = last;
                instruments.push(instrument);
            }
        }
    }
    MetricCorpus {
        tenant: verify.tenant.clone(),
        anchor_ns: verify.anchor_ns,
        scrape_interval_ns: verify.scrape_interval_seconds * 1_000_000_000,
        scrapes: verify.scrapes,
        instruments,
    }
}

impl MetricCorpus {
    pub fn scrape_ts_ns(&self, scrape: usize) -> i64 {
        self.anchor_ns + scrape as i64 * self.scrape_interval_ns
    }

    /// OTLP datapoints the corpus sends, before any decomposition.
    pub fn datapoint_count(&self) -> u64 {
        self.instruments
            .iter()
            .map(|instrument| {
                (instrument.last_scrape.min(self.scrapes)
                    - instrument.first_scrape.min(self.scrapes)) as u64
            })
            .sum()
    }

    /// Float samples the dataset decomposes into: one per gauge or counter
    /// datapoint, `bounds + 3` per histogram datapoint (`_bucket` per bound
    /// plus `+Inf`, `_sum`, `_count`).
    pub fn decomposed_sample_count(&self) -> u64 {
        self.instruments
            .iter()
            .map(|instrument| {
                let scrapes = (instrument.last_scrape.min(self.scrapes)
                    - instrument.first_scrape.min(self.scrapes))
                    as u64;
                match instrument.kind {
                    InstrumentKind::Gauge | InstrumentKind::Counter => scrapes,
                    InstrumentKind::Histogram => scrapes * (HISTOGRAM_BOUNDS.len() as u64 + 3),
                }
            })
            .sum()
    }

    /// Distinct decomposed series the dataset holds, for the report's series
    /// census.
    pub fn decomposed_series_count(&self) -> u64 {
        self.instruments
            .iter()
            .map(|instrument| match instrument.kind {
                InstrumentKind::Gauge | InstrumentKind::Counter => 1,
                InstrumentKind::Histogram => HISTOGRAM_BOUNDS.len() as u64 + 3,
            })
            .sum()
    }
}

fn string_attribute(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_string())),
        }),
        ..Default::default()
    }
}

struct CounterState {
    total: u64,
}

struct HistogramState {
    bucket_totals: Vec<u64>,
    sum: f64,
    count: u64,
}

/// One `ExportMetricsServiceRequest` per scrape, instruments grouped by metric
/// name the way a real SDK exports them, in a deterministic order.
///
/// Cumulative temporality throughout: each datapoint carries the totals since
/// the instrument's own start scrape, which is what makes the churn block's
/// replacement generations look exactly like restarted pods — new series,
/// totals starting over.
pub struct SeedBody {
    pub bytes: Vec<u8>,
    pub datapoints: u64,
    pub decomposed_samples: u64,
}

/// `tenant` is stamped on the resource when the target reads its tenant out of
/// the payload rather than a header — signy, and only signy.
/// A resource naming nothing but the tenant. The metric corpus promotes no
/// resource attributes of its own, so this is the whole of it.
fn tenant_resource(tenant: &str) -> opentelemetry_proto::tonic::resource::v1::Resource {
    opentelemetry_proto::tonic::resource::v1::Resource {
        attributes: vec![crate::otlp::tenant_attribute(tenant)],
        ..Default::default()
    }
}

pub fn seed_bodies(
    corpus: &MetricCorpus,
    tenant: Option<&str>,
    wire: crate::config::PushWire,
) -> Vec<SeedBody> {
    let mut counters: Vec<CounterState> = corpus
        .instruments
        .iter()
        .map(|_| CounterState { total: 0 })
        .collect();
    let mut histograms: Vec<HistogramState> = corpus
        .instruments
        .iter()
        .map(|_| HistogramState {
            bucket_totals: vec![0; HISTOGRAM_BOUNDS.len() + 1],
            sum: 0.0,
            count: 0,
        })
        .collect();

    let mut bodies = Vec::with_capacity(corpus.scrapes);
    for scrape in 0..corpus.scrapes {
        let ts = corpus.scrape_ts_ns(scrape) as u64;
        // metric name -> the scrape's datapoints, grouped the way OTLP groups
        // them; BTreeMap so the byte stream is deterministic.
        let mut gauges: BTreeMap<&str, Vec<NumberDataPoint>> = BTreeMap::new();
        let mut sums: BTreeMap<&str, Vec<NumberDataPoint>> = BTreeMap::new();
        let mut hists: BTreeMap<&str, Vec<HistogramDataPoint>> = BTreeMap::new();
        let mut datapoints = 0u64;
        let mut decomposed = 0u64;

        for (index, instrument) in corpus.instruments.iter().enumerate() {
            if !instrument.active_at(scrape) {
                continue;
            }
            let start_ts = corpus.scrape_ts_ns(instrument.first_scrape) as u64;
            let attributes: Vec<KeyValue> = instrument
                .labels
                .iter()
                .map(|(name, value)| string_attribute(name, value))
                .collect();
            datapoints += 1;
            match instrument.kind {
                InstrumentKind::Gauge => {
                    decomposed += 1;
                    gauges
                        .entry(instrument.metric.as_str())
                        .or_default()
                        .push(NumberDataPoint {
                            attributes,
                            time_unix_nano: ts,
                            value: Some(number_data_point::Value::AsDouble(
                                instrument.gauge_value(scrape),
                            )),
                            ..Default::default()
                        });
                }
                InstrumentKind::Counter => {
                    decomposed += 1;
                    counters[index].total += instrument.counter_increment(scrape);
                    sums.entry(instrument.metric.as_str())
                        .or_default()
                        .push(NumberDataPoint {
                            attributes,
                            start_time_unix_nano: start_ts,
                            time_unix_nano: ts,
                            value: Some(number_data_point::Value::AsDouble(
                                counters[index].total as f64,
                            )),
                            ..Default::default()
                        });
                }
                InstrumentKind::Histogram => {
                    decomposed += HISTOGRAM_BOUNDS.len() as u64 + 3;
                    let state = &mut histograms[index];
                    for bucket in 0..state.bucket_totals.len() {
                        let increment = instrument.bucket_increment(scrape, bucket);
                        state.bucket_totals[bucket] += increment;
                        state.count += increment;
                    }
                    state.sum += instrument.sum_increment(scrape);
                    hists
                        .entry(instrument.metric.as_str())
                        .or_default()
                        .push(HistogramDataPoint {
                            attributes,
                            start_time_unix_nano: start_ts,
                            time_unix_nano: ts,
                            count: state.count,
                            sum: Some(state.sum),
                            bucket_counts: state.bucket_totals.clone(),
                            explicit_bounds: HISTOGRAM_BOUNDS.to_vec(),
                            ..Default::default()
                        });
                }
            }
        }

        let mut metrics: Vec<Metric> = Vec::new();
        for (name, data_points) in gauges {
            metrics.push(Metric {
                name: name.to_string(),
                data: Some(metric::Data::Gauge(Gauge { data_points })),
                ..Default::default()
            });
        }
        for (name, data_points) in sums {
            metrics.push(Metric {
                name: name.to_string(),
                data: Some(metric::Data::Sum(Sum {
                    data_points,
                    aggregation_temporality: AggregationTemporality::Cumulative as i32,
                    is_monotonic: true,
                })),
                ..Default::default()
            });
        }
        for (name, data_points) in hists {
            metrics.push(Metric {
                name: name.to_string(),
                data: Some(metric::Data::Histogram(Histogram {
                    data_points,
                    aggregation_temporality: AggregationTemporality::Cumulative as i32,
                })),
                ..Default::default()
            });
        }

        let request = ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: tenant.map(tenant_resource),
                scope_metrics: vec![ScopeMetrics {
                    metrics,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        bodies.push(SeedBody {
            bytes: wire.wrap(request.encode_to_vec()),
            datapoints,
            decomposed_samples: decomposed,
        });
    }
    bodies
}

/// Anchors that are not `> 0` are refused for the metric phases too, and for
/// the same reason as the log anchor: a clock-derived default would seed two
/// different datasets on the two runs of a comparison.
pub fn require_metric_anchor(verify: &MetricVerify) -> Result<(), String> {
    if verify.anchor_ns > 0 {
        return Ok(());
    }
    Err(
        "SIGNY_LOAD_METRIC_ANCHOR_NS must be set to the same value for both runs of a \
comparison; without it each run would seed a different dataset"
            .to_string(),
    )
}

pub struct MetricSeedOutcome {
    pub pushes: u64,
    pub datapoints: u64,
    pub decomposed_samples: u64,
    pub wire_bytes: u64,
    pub retries: u64,
    pub errors: u64,
    pub rejected_datapoints: u64,
    pub statuses: BTreeMap<u16, u64>,
    pub first_error: Option<String>,
    pub elapsed_seconds: f64,
}

impl Default for MetricSeedOutcome {
    fn default() -> Self {
        Self {
            pushes: 0,
            datapoints: 0,
            decomposed_samples: 0,
            wire_bytes: 0,
            retries: 0,
            errors: 0,
            rejected_datapoints: 0,
            statuses: BTreeMap::new(),
            first_error: None,
            elapsed_seconds: 0.0,
        }
    }
}

/// A 2xx body is still inspected for what it refused: a seed that was
/// partially rejected has not seeded the dataset, and reading only the status
/// line would report it as complete.
///
/// Two answers say it, because the two endpoints are not the same endpoint.
/// VictoriaMetrics answers the OTLP `ExportMetricsServiceResponse` and names
/// the count in its `partial_success`. signy has no OTLP endpoint left — the
/// collect route is the whole of its ingest — and names the same count in the
/// `rejected` field of its own JSON answer.
pub fn rejected_datapoints(target: Target, body: &[u8]) -> u64 {
    if body.is_empty() {
        return 0;
    }
    match target {
        Target::Signy => json_number(body, "rejected"),
        _ => ExportMetricsServiceResponse::decode(body)
            .ok()
            .and_then(|response| response.partial_success)
            .map(|partial| partial.rejected_data_points.max(0) as u64)
            .unwrap_or(0),
    }
}

/// One unsigned field out of a flat JSON object, without a parser: the answer
/// is written by this repository and is two numbers.
fn json_number(body: &[u8], field: &str) -> u64 {
    let body = String::from_utf8_lossy(body);
    let key = format!("\"{field}\"");
    let Some(at) = body.find(&key) else {
        return 0;
    };
    body[at + key.len()..]
        .trim_start()
        .strip_prefix(':')
        .map(|rest| rest.trim_start())
        .map(|rest| {
            rest.chars()
                .take_while(|character| character.is_ascii_digit())
                .collect::<String>()
        })
        .and_then(|digits| digits.parse().ok())
        .unwrap_or(0)
}

pub async fn run_metric_seed(cfg: &Config, corpus: &MetricCorpus) -> MetricSeedOutcome {
    let wire = cfg.push_wire();
    let push_headers = wire.headers(Signal::Metrics);
    let target = cfg.target;
    let Some(push_path) = wire.path(Signal::Metrics) else {
        return MetricSeedOutcome {
            errors: 1,
            first_error: Some(format!(
                "target {} has no OTLP metrics ingest",
                cfg.target.name()
            )),
            ..MetricSeedOutcome::default()
        };
    };
    let header = wire.tenant_header(&corpus.tenant);
    // signy reads the tenant out of the export, the others out of the header,
    // so exactly one of these two carries it.
    let in_body = header.is_none().then_some(corpus.tenant.as_str());
    let bodies = Arc::new(Mutex::new(seed_bodies(corpus, in_body, wire)));
    let start = Instant::now();

    let workers: Vec<_> = (0..cfg.metric_verify.push_connections)
        .map(|_| {
            let bodies = bodies.clone();
            let header = header.clone();
            let address = cfg.push_address().to_string();
            let timeout = cfg.request_timeout();
            tokio::spawn(async move {
                let mut client = Client::new(&address, timeout);
                let mut outcome = MetricSeedOutcome::default();
                loop {
                    let Some(body) = bodies.lock().await.pop() else {
                        break;
                    };
                    // Like the log seed: a refusal is waited out rather than
                    // recorded, because the dataset has to land in full on
                    // both sides or the comparison has nothing to stand on.
                    for attempt in 0..60 {
                        let result = client
                            .request(&Request {
                                method: "POST",
                                path: push_path,
                                body: &body.bytes,
                                content_type: PUSH_CONTENT_TYPE,
                                headers: push_headers,
                                tenant: header
                                    .as_ref()
                                    .map(|(name, value)| (*name, value.as_str())),
                            })
                            .await;
                        match result {
                            Ok(response) => {
                                *outcome.statuses.entry(response.status).or_default() += 1;
                                if (200..300).contains(&response.status) {
                                    let rejected = rejected_datapoints(target, &response.body);
                                    if rejected > 0 {
                                        outcome.rejected_datapoints += rejected;
                                        outcome.errors += 1;
                                        outcome.first_error.get_or_insert_with(|| {
                                            format!(
                                                "partial_success rejected {rejected} datapoints"
                                            )
                                        });
                                        break;
                                    }
                                    outcome.pushes += 1;
                                    outcome.datapoints += body.datapoints;
                                    outcome.decomposed_samples += body.decomposed_samples;
                                    outcome.wire_bytes += body.bytes.len() as u64;
                                    break;
                                }
                                if response.status == 429 && attempt < 59 {
                                    outcome.retries += 1;
                                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                                    continue;
                                }
                                outcome.errors += 1;
                                outcome.first_error.get_or_insert_with(|| {
                                    format!(
                                        "{}: {}",
                                        response.status,
                                        String::from_utf8_lossy(&response.body)
                                            .chars()
                                            .take(300)
                                            .collect::<String>()
                                    )
                                });
                                break;
                            }
                            Err(error) => {
                                if attempt < 59 {
                                    outcome.retries += 1;
                                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                                    continue;
                                }
                                outcome.errors += 1;
                                outcome.first_error.get_or_insert(error);
                                break;
                            }
                        }
                    }
                }
                outcome
            })
        })
        .collect();

    let mut total = MetricSeedOutcome::default();
    for worker in workers {
        if let Ok(outcome) = worker.await {
            total.pushes += outcome.pushes;
            total.datapoints += outcome.datapoints;
            total.decomposed_samples += outcome.decomposed_samples;
            total.wire_bytes += outcome.wire_bytes;
            total.retries += outcome.retries;
            total.errors += outcome.errors;
            total.rejected_datapoints += outcome.rejected_datapoints;
            for (status, count) in outcome.statuses {
                *total.statuses.entry(status).or_default() += count;
            }
            total.first_error = total.first_error.take().or(outcome.first_error);
        }
    }
    total.elapsed_seconds = start.elapsed().as_secs_f64();
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MetricVerify;

    fn verify_for_test() -> MetricVerify {
        MetricVerify {
            tenant: "verify-metrics".to_string(),
            anchor_ns: 1_772_000_000_000_000_000,
            scrapes: 12,
            scrape_interval_seconds: 10,
            services: 2,
            instances_per_service: 2,
            gauges: 2,
            counters: 2,
            churn_generations: 3,
            churn_instances: 2,
            push_connections: 2,
            windows: 3,
            repeats: 2,
            step_seconds: 30,
            range_seconds: 60,
            steady_seconds: 2,
            churn_seconds: 2,
            churn_replace_per_scrape: 2,
            explosion_seconds: 2,
            explosion_series: 4,
        }
    }

    #[test]
    fn the_corpus_is_a_pure_function_of_seed_and_knobs() {
        let first = seed_bodies_for_test(&metric_corpus(7, &verify_for_test()));
        let second = seed_bodies_for_test(&metric_corpus(7, &verify_for_test()));
        let other_seed = seed_bodies_for_test(&metric_corpus(8, &verify_for_test()));
        assert_eq!(first.len(), second.len());
        for (a, b) in first.iter().zip(&second) {
            assert_eq!(a.bytes, b.bytes, "same seed, same bytes");
        }
        assert!(
            first
                .iter()
                .zip(&other_seed)
                .any(|(a, b)| a.bytes != b.bytes),
            "a different seed is a different dataset"
        );
    }

    #[test]
    fn churn_generations_tile_the_scrape_range_without_overlap() {
        let corpus = metric_corpus(7, &verify_for_test());
        for scrape in 0..corpus.scrapes {
            let live: Vec<&Instrument> = corpus
                .instruments
                .iter()
                .filter(|instrument| {
                    instrument.metric == CHURN_COUNTER && instrument.active_at(scrape)
                })
                .collect();
            assert_eq!(
                live.len(),
                verify_for_test().churn_instances,
                "exactly one generation is live at scrape {scrape}"
            );
        }
        let boundary_crossers: usize = corpus
            .instruments
            .iter()
            .filter(|instrument| instrument.first_scrape > 0)
            .count();
        assert!(
            boundary_crossers > 0,
            "some churn series must begin mid-span or the churned shape reads nothing"
        );
    }

    #[test]
    fn cumulative_counters_restart_at_a_generation_boundary() {
        let corpus = metric_corpus(7, &verify_for_test());
        let later_generation = corpus
            .instruments
            .iter()
            .find(|instrument| instrument.metric == CHURN_COUNTER && instrument.first_scrape > 0)
            .expect("a later churn generation exists");
        // The body encoder starts every counter at zero from its own first
        // scrape; decode the generation's first scrape and check its total is
        // its first increment, not a continuation.
        let bodies = seed_bodies_for_test(&corpus);
        let request = ExportMetricsServiceRequest::decode(
            bodies[later_generation.first_scrape].bytes.as_slice(),
        )
        .expect("valid protobuf");
        let mut found = false;
        for resource in &request.resource_metrics {
            for scope in &resource.scope_metrics {
                for metric in &scope.metrics {
                    if metric.name != CHURN_COUNTER {
                        continue;
                    }
                    let Some(metric::Data::Sum(sum)) = &metric.data else {
                        panic!("churn counter is a sum");
                    };
                    assert_eq!(
                        sum.aggregation_temporality,
                        AggregationTemporality::Cumulative as i32
                    );
                    for point in &sum.data_points {
                        let instance = point
                            .attributes
                            .iter()
                            .find(|attribute| attribute.key == "instance")
                            .and_then(|attribute| attribute.value.as_ref())
                            .and_then(|value| value.value.as_ref())
                            .map(|value| match value {
                                any_value::Value::StringValue(text) => text.clone(),
                                other => format!("{other:?}"),
                            })
                            .unwrap_or_default();
                        let expected_instance = later_generation
                            .labels
                            .iter()
                            .find(|(name, _)| name == "instance")
                            .map(|(_, value)| value.clone())
                            .unwrap();
                        if instance == expected_instance {
                            let Some(number_data_point::Value::AsDouble(value)) = point.value
                            else {
                                panic!("counter value is a double");
                            };
                            assert_eq!(
                                value,
                                later_generation.counter_increment(later_generation.first_scrape)
                                    as f64,
                                "the generation's first total is its own first increment"
                            );
                            found = true;
                        }
                    }
                }
            }
        }
        assert!(
            found,
            "the later generation's first scrape carries its point"
        );
    }

    #[test]
    fn histogram_bucket_counts_are_cumulative_and_consistent_with_count() {
        let corpus = metric_corpus(7, &verify_for_test());
        let bodies = seed_bodies_for_test(&corpus);
        let last = ExportMetricsServiceRequest::decode(
            bodies.last().expect("bodies exist").bytes.as_slice(),
        )
        .expect("valid protobuf");
        let mut seen = 0;
        for resource in &last.resource_metrics {
            for scope in &resource.scope_metrics {
                for metric in &scope.metrics {
                    let Some(metric::Data::Histogram(histogram)) = &metric.data else {
                        continue;
                    };
                    for point in &histogram.data_points {
                        assert_eq!(point.explicit_bounds, HISTOGRAM_BOUNDS.to_vec());
                        assert_eq!(point.bucket_counts.len(), HISTOGRAM_BOUNDS.len() + 1);
                        assert_eq!(
                            point.bucket_counts.iter().sum::<u64>(),
                            point.count,
                            "count is the bucket total"
                        );
                        assert!(point.count > 0, "the last scrape has accumulated counts");
                        seen += 1;
                    }
                }
            }
        }
        assert_eq!(seen, 4, "one histogram per steady (service, instance) pair");
    }

    #[test]
    fn the_sample_census_matches_what_the_bodies_carry() {
        let corpus = metric_corpus(7, &verify_for_test());
        let bodies = seed_bodies_for_test(&corpus);
        let datapoints: u64 = bodies.iter().map(|body| body.datapoints).sum();
        let decomposed: u64 = bodies.iter().map(|body| body.decomposed_samples).sum();
        assert_eq!(datapoints, corpus.datapoint_count());
        assert_eq!(decomposed, corpus.decomposed_sample_count());
        assert!(decomposed > datapoints, "the histogram multiplier is real");
    }

    #[test]
    fn the_phase_boundaries_tile_the_run_and_then_end_it() {
        let verify = verify_for_test();
        assert_eq!(phase_at(0, &verify), Some(LoadPhase::Steady));
        assert_eq!(phase_at(1, &verify), Some(LoadPhase::Steady));
        assert_eq!(phase_at(2, &verify), Some(LoadPhase::Churn));
        assert_eq!(phase_at(3, &verify), Some(LoadPhase::Churn));
        assert_eq!(phase_at(4, &verify), Some(LoadPhase::Explosion));
        assert_eq!(phase_at(5, &verify), Some(LoadPhase::Explosion));
        assert_eq!(
            phase_at(6, &verify),
            None,
            "the run ends rather than looping"
        );
    }

    #[test]
    fn churning_replaces_a_generation_and_the_burst_mints_series_once() {
        let verify = verify_for_test();
        let mut population = LivePopulation::new(7, &verify);
        let steady = population.steady.len();
        assert!(steady > 0);
        let before = population.minted;

        population.churn(7, verify.churn_replace_per_scrape);
        let first_generation: Vec<String> = population
            .churned
            .iter()
            .map(|instrument| format!("{:?}", instrument.labels))
            .collect();
        assert_eq!(
            population.churned.len(),
            verify.churn_replace_per_scrape * 2,
            "a counter and a gauge per replaced instance"
        );
        population.churn(7, verify.churn_replace_per_scrape);
        let second_generation: Vec<String> = population
            .churned
            .iter()
            .map(|instrument| format!("{:?}", instrument.labels))
            .collect();
        assert_ne!(
            first_generation, second_generation,
            "a generation's instances are replaced, not re-reported"
        );
        assert_eq!(
            population.instruments().count(),
            steady + verify.churn_replace_per_scrape * 2,
            "only the live generation reports"
        );

        population.explode(7, verify.explosion_series);
        assert_eq!(population.exploded.len(), verify.explosion_series);
        assert!(
            population.minted > before,
            "the census counts every identity the run offered"
        );
    }

    /// Every fixture names a tenant, because signy reads one out of the body.
    fn seed_bodies_for_test(corpus: &MetricCorpus) -> Vec<SeedBody> {
        seed_bodies(
            corpus,
            Some("test-tenant"),
            crate::config::PushWire::direct(Target::Loki),
        )
    }

    #[test]
    fn a_live_scrape_encodes_every_live_instrument_once() {
        let verify = verify_for_test();
        let mut population = LivePopulation::new(7, &verify);
        population.churn(7, verify.churn_replace_per_scrape);
        let bodies = live_scrape_bodies(
            &mut population,
            3,
            1_772_000_000_000_000_000,
            Some("test-tenant"),
            crate::config::PushWire::direct(Target::Loki),
        );
        let offered: u64 = bodies.iter().map(|(_, datapoints, _)| *datapoints).sum();
        assert_eq!(offered, population.instruments().count() as u64);
        let encoded: u64 = bodies
            .iter()
            .map(|(bytes, _, _)| {
                let decoded =
                    ExportMetricsServiceRequest::decode(bytes.as_slice()).expect("valid protobuf");
                decoded
                    .resource_metrics
                    .iter()
                    .flat_map(|resource| resource.scope_metrics.iter())
                    .flat_map(|scope| scope.metrics.iter())
                    .map(|metric| match &metric.data {
                        Some(metric::Data::Gauge(gauge)) => gauge.data_points.len() as u64,
                        Some(metric::Data::Sum(sum)) => sum.data_points.len() as u64,
                        Some(metric::Data::Histogram(histogram)) => {
                            histogram.data_points.len() as u64
                        }
                        _ => 0,
                    })
                    .sum::<u64>()
            })
            .sum();
        assert_eq!(encoded, offered);
    }

    /// A burst larger than one export may carry is split, because an engine
    /// refusing the *request* is a different refusal from an engine refusing
    /// the new series in it — and the second is the one the claim is about.
    #[test]
    fn a_scrape_past_one_exports_worth_is_split_into_several() {
        let mut verify = verify_for_test();
        verify.explosion_series = SCRAPE_CHUNK_INSTRUMENTS * 2 + 1;
        let mut population = LivePopulation::new(7, &verify);
        population.explode(7, verify.explosion_series);
        let bodies = live_scrape_bodies(
            &mut population,
            0,
            1_772_000_000_000_000_000,
            Some("test-tenant"),
            crate::config::PushWire::direct(Target::Loki),
        );
        assert!(bodies.len() >= 3, "{} bodies", bodies.len());
        for (_, datapoints, _) in &bodies {
            assert!(
                *datapoints as usize <= SCRAPE_CHUNK_INSTRUMENTS,
                "no body carries more instruments than one export's worth"
            );
        }
        assert_eq!(
            bodies
                .iter()
                .map(|(_, datapoints, _)| *datapoints)
                .sum::<u64>(),
            population.instruments().count() as u64,
            "splitting drops nothing"
        );
    }

    #[test]
    fn a_phase_tally_reads_acceptance_from_what_partial_success_refused() {
        let mut tally = PhaseTally::default();
        tally.record(200, 100, 0, 100, 1.0);
        assert_eq!(tally.acceptance(), 1.0);
        tally.record(200, 100, 50, 100, 1.0);
        assert_eq!(tally.datapoints_rejected, 50);
        assert_eq!(tally.acceptance(), 150.0 / 200.0);
        tally.record(429, 100, 0, 100, 1.0);
        assert_eq!(tally.requests_refused, 1);
        assert_eq!(
            tally.acceptance(),
            150.0 / 300.0,
            "a whole-request refusal counts against acceptance like any unaccepted offer"
        );
        assert_eq!(tally.series_offered, 300);
        assert_eq!(tally.series_accepted, 150);
        assert_eq!(tally.series_rejected, 150);
    }

    #[test]
    fn running_totals_report_what_summing_from_zero_reported() {
        // The quadratic version is the specification: whatever a scrape used
        // to report, the accumulator has to report, including for an
        // instrument minted in the middle of a run.
        let labels: [(&str, &str); 2] = [("service", "api"), ("instance", "instance-0")];
        for kind in [InstrumentKind::Counter, InstrumentKind::Histogram] {
            let mut live = instrument(7, "m", kind, &labels);
            for scrape in 0..64 {
                live.advance_to(scrape);
                let expected_counter: u64 = (0..=scrape).map(|at| live.counter_increment(at)).sum();
                let expected_sum: f64 = (0..=scrape).map(|at| live.sum_increment(at)).sum();
                let expected_buckets: Vec<u64> = (0..HISTOGRAM_BOUNDS.len() + 1)
                    .map(|bucket| {
                        (0..=scrape)
                            .map(|at| live.bucket_increment(at, bucket))
                            .sum()
                    })
                    .collect();
                match kind {
                    InstrumentKind::Counter => assert_eq!(live.totals.counter, expected_counter),
                    InstrumentKind::Histogram => {
                        assert_eq!(live.totals.buckets, expected_buckets);
                        assert_eq!(live.totals.sum, expected_sum);
                    }
                    InstrumentKind::Gauge => unreachable!(),
                }
            }
            let mut minted_late = instrument(7, "m", kind, &labels);
            minted_late.advance_to(63);
            assert_eq!(minted_late.totals.counter, live.totals.counter);
            assert_eq!(minted_late.totals.buckets, live.totals.buckets);
            assert_eq!(minted_late.totals.sum, live.totals.sum);
        }
    }

    #[test]
    fn a_repeated_scrape_index_does_not_double_count() {
        let labels: [(&str, &str); 1] = [("service", "api")];
        let mut counter = instrument(11, "m", InstrumentKind::Counter, &labels);
        counter.advance_to(5);
        let after_five = counter.totals.counter;
        counter.advance_to(5);
        assert_eq!(counter.totals.counter, after_five);
    }

    #[test]
    fn an_unset_metric_anchor_is_refused() {
        let mut verify = verify_for_test();
        verify.anchor_ns = 0;
        assert!(require_metric_anchor(&verify).is_err());
        verify.anchor_ns = 1;
        assert!(require_metric_anchor(&verify).is_ok());
    }
}

// --- The paced ingest phases: steady, churn, explosion (M14 Phase 8) ---

/// Which sub-phase a scrape belongs to. The claim's own half lives in the
/// second and third: an engine that sizes its index to the workload and one
/// that sizes the workload to its budget diverge exactly here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoadPhase {
    Steady,
    Churn,
    Explosion,
}

impl LoadPhase {
    pub fn name(self) -> &'static str {
        match self {
            LoadPhase::Steady => "steady",
            LoadPhase::Churn => "churn",
            LoadPhase::Explosion => "explosion",
        }
    }
}

#[derive(Default)]
pub struct PhaseTally {
    pub scrapes: u64,
    pub datapoints_offered: u64,
    pub datapoints_accepted: u64,
    /// What OTLP `partial_success` said was refused — the designed behavior
    /// under cardinality pressure, counted rather than hidden.
    pub datapoints_rejected: u64,
    /// Requests refused whole (429), which is the all-new-series case.
    pub requests_refused: u64,
    /// Series identities carried by this phase. The capacity harness uses a
    /// one-shot scrape, so these are distinct series rather than repeated
    /// reports of the same identity.
    pub series_offered: u64,
    pub series_accepted: u64,
    pub series_rejected: u64,
    pub errors: u64,
    /// Scrapes nobody answered. A soak stops the engine on purpose, so an
    /// unanswered push is availability rather than an ingest error, and the
    /// two are counted apart.
    pub unavailable: u64,
    pub first_error: Option<String>,
    pub first_unavailable: Option<String>,
    pub statuses: BTreeMap<u16, u64>,
    pub latency: crate::stats::Series,
}

impl PhaseTally {
    /// The fraction of offered datapoints the engine took. The gate reads the
    /// steady phase's: a budget met by refusing the load was never exercised.
    pub fn acceptance(&self) -> f64 {
        if self.datapoints_offered == 0 {
            return 0.0;
        }
        self.datapoints_accepted as f64 / self.datapoints_offered as f64
    }

    pub fn record(
        &mut self,
        status: u16,
        offered: u64,
        rejected: u64,
        offered_series: u64,
        elapsed_ms: f64,
    ) {
        self.scrapes += 1;
        self.datapoints_offered += offered;
        self.series_offered += offered_series;
        *self.statuses.entry(status).or_default() += 1;
        self.latency.push(elapsed_ms);
        if (200..300).contains(&status) {
            self.datapoints_rejected += rejected;
            self.datapoints_accepted += offered.saturating_sub(rejected);
            // OTLP reports partial acceptance by datapoint, not by the
            // decomposed Prometheus series it may create. Capacity trials use
            // gauge-only burst identities, making this exact for the series
            // under test; baseline histogram series are only a fixed prefix.
            self.series_rejected += rejected.min(offered_series);
            self.series_accepted += offered_series.saturating_sub(rejected);
        } else if status == 429 {
            self.requests_refused += 1;
            self.series_rejected += offered_series;
        } else {
            self.series_rejected += offered_series;
        }
    }
}

pub struct MetricLoadOutcome {
    pub phases: Vec<(LoadPhase, PhaseTally)>,
    pub elapsed_seconds: f64,
    /// Distinct series the generator minted across the run, which is what the
    /// engine's own `active_series` gauge is read against.
    pub series_offered: u64,
}

/// One scrape's worth of instruments: the live steady set plus whatever the
/// churn and explosion phases have added.
pub struct LivePopulation {
    steady: Vec<Instrument>,
    churned: Vec<Instrument>,
    exploded: Vec<Instrument>,
    generation: usize,
    minted: u64,
}

impl LivePopulation {
    pub fn new(seed: u64, verify: &MetricVerify) -> Self {
        let services = service_names(verify.services);
        let mut steady = Vec::new();
        for service in &services {
            for index in 0..verify.instances_per_service {
                let instance = format!("instance-{index}");
                let env = if index % 2 == 0 { "prod" } else { "staging" };
                let labels: [(&str, &str); 3] = [
                    ("service", service.as_str()),
                    ("instance", instance.as_str()),
                    ("env", env),
                ];
                for gauge in GAUGE_NAMES.iter().take(verify.gauges) {
                    steady.push(instrument(seed, gauge, InstrumentKind::Gauge, &labels));
                }
                for counter in COUNTER_NAMES.iter().take(verify.counters) {
                    steady.push(instrument(seed, counter, InstrumentKind::Counter, &labels));
                }
                steady.push(instrument(
                    seed,
                    HISTOGRAM_NAME,
                    InstrumentKind::Histogram,
                    &labels,
                ));
            }
        }
        let minted = steady.len() as u64;
        Self {
            steady,
            churned: Vec::new(),
            exploded: Vec::new(),
            generation: 0,
            minted,
        }
    }

    /// Replace the churn block: the previous generation's instances stop
    /// reporting and an equal number of fresh ones appear. Their history stays
    /// in the engine, which is the point — the cost of churn is what the dead
    /// generations leave behind.
    pub fn churn(&mut self, seed: u64, count: usize) {
        self.generation += 1;
        self.churned.clear();
        for index in 0..count {
            let instance = format!("churn-{}-{index}", self.generation);
            let labels: [(&str, &str); 3] = [
                ("service", CHURN_SERVICE),
                ("instance", instance.as_str()),
                ("env", "prod"),
            ];
            self.churned.push(instrument(
                seed,
                CHURN_COUNTER,
                InstrumentKind::Counter,
                &labels,
            ));
            self.churned.push(instrument(
                seed,
                CHURN_GAUGE,
                InstrumentKind::Gauge,
                &labels,
            ));
        }
        self.minted += self.churned.len() as u64;
    }

    /// The burst: one scrape mints `count` distinct series that never repeat,
    /// which is the cardinality explosion an engine either contains or dies
    /// of.
    fn explode(&mut self, seed: u64, count: usize) {
        self.exploded.clear();
        for index in 0..count {
            let instance = format!("burst-{index}");
            let labels: [(&str, &str); 3] = [
                ("service", "burstsvc"),
                ("instance", instance.as_str()),
                ("env", "prod"),
            ];
            self.exploded.push(instrument(
                seed,
                CHURN_GAUGE,
                InstrumentKind::Gauge,
                &labels,
            ));
        }
        self.minted += self.exploded.len() as u64;
    }

    fn instruments(&self) -> impl Iterator<Item = &Instrument> {
        self.steady
            .iter()
            .chain(self.churned.iter())
            .chain(self.exploded.iter())
    }

    /// Fold this scrape into every live instrument's cumulative totals.
    fn advance(&mut self, scrape: usize) {
        for instrument in self
            .steady
            .iter_mut()
            .chain(self.churned.iter_mut())
            .chain(self.exploded.iter_mut())
        {
            instrument.advance_to(scrape);
        }
    }

    /// Distinct series identities this population has minted, which is what
    /// the engine's own `active_series` gauge is read against.
    pub fn minted(&self) -> u64 {
        self.minted
    }
}

/// Instruments per request. A real collector batches, and so must this: one
/// export past the engine's own decomposed-sample cap is refused *whole*, by
/// the request limit rather than by the cardinality ladder — which would
/// measure the wrong refusal and call it the claim's. Sized well under
/// `MAX_OTLP_METRIC_SAMPLES` because a histogram instrument fans out to
/// `bounds + 3` samples.
const SCRAPE_CHUNK_INSTRUMENTS: usize = 5_000;

/// One scrape as OTLP bodies at wall-clock `ts_ns`, plus the datapoints each
/// carries. Every instrument reports cumulative totals from `scrape`, the same
/// arithmetic the seeded corpus uses.
pub fn live_scrape_bodies(
    population: &mut LivePopulation,
    scrape: usize,
    ts_ns: i64,
    tenant: Option<&str>,
    wire: crate::config::PushWire,
) -> Vec<(Vec<u8>, u64, u64)> {
    population.advance(scrape);
    let instruments: Vec<&Instrument> = population.instruments().collect();
    instruments
        .chunks(SCRAPE_CHUNK_INSTRUMENTS.max(1))
        .map(|chunk| {
            let (payload, datapoints) = live_scrape_body(chunk, scrape, ts_ns, tenant);
            let series = chunk
                .iter()
                .map(|instrument| match instrument.kind {
                    InstrumentKind::Gauge | InstrumentKind::Counter => 1,
                    InstrumentKind::Histogram => HISTOGRAM_BOUNDS.len() as u64 + 3,
                })
                .sum();
            (wire.wrap(payload), datapoints, series)
        })
        .collect()
}

fn live_scrape_body(
    instruments: &[&Instrument],
    scrape: usize,
    ts_ns: i64,
    tenant: Option<&str>,
) -> (Vec<u8>, u64) {
    let mut gauges: BTreeMap<&str, Vec<NumberDataPoint>> = BTreeMap::new();
    let mut sums: BTreeMap<&str, Vec<NumberDataPoint>> = BTreeMap::new();
    let mut hists: BTreeMap<&str, Vec<HistogramDataPoint>> = BTreeMap::new();
    let mut datapoints = 0u64;
    let ts = ts_ns.max(0) as u64;

    for instrument in instruments {
        let attributes: Vec<KeyValue> = instrument
            .labels
            .iter()
            .map(|(name, value)| string_attribute(name, value))
            .collect();
        datapoints += 1;
        match instrument.kind {
            InstrumentKind::Gauge => {
                gauges
                    .entry(instrument.metric.as_str())
                    .or_default()
                    .push(NumberDataPoint {
                        attributes,
                        time_unix_nano: ts,
                        value: Some(number_data_point::Value::AsDouble(
                            instrument.gauge_value(scrape),
                        )),
                        ..Default::default()
                    });
            }
            InstrumentKind::Counter => {
                // The running total from the run's own start, so a paced
                // scrape is a cumulative counter like a real exporter's.
                let total = instrument.totals.counter;
                sums.entry(instrument.metric.as_str())
                    .or_default()
                    .push(NumberDataPoint {
                        attributes,
                        time_unix_nano: ts,
                        value: Some(number_data_point::Value::AsDouble(total as f64)),
                        ..Default::default()
                    });
            }
            InstrumentKind::Histogram => {
                let buckets = instrument.totals.buckets.clone();
                let sum = instrument.totals.sum;
                let count = buckets.iter().sum();
                hists
                    .entry(instrument.metric.as_str())
                    .or_default()
                    .push(HistogramDataPoint {
                        attributes,
                        time_unix_nano: ts,
                        count,
                        sum: Some(sum),
                        bucket_counts: buckets,
                        explicit_bounds: HISTOGRAM_BOUNDS.to_vec(),
                        ..Default::default()
                    });
            }
        }
    }

    let mut metrics: Vec<Metric> = Vec::new();
    for (name, data_points) in gauges {
        metrics.push(Metric {
            name: name.to_string(),
            data: Some(metric::Data::Gauge(Gauge { data_points })),
            ..Default::default()
        });
    }
    for (name, data_points) in sums {
        metrics.push(Metric {
            name: name.to_string(),
            data: Some(metric::Data::Sum(Sum {
                data_points,
                aggregation_temporality: AggregationTemporality::Cumulative as i32,
                is_monotonic: true,
            })),
            ..Default::default()
        });
    }
    for (name, data_points) in hists {
        metrics.push(Metric {
            name: name.to_string(),
            data: Some(metric::Data::Histogram(Histogram {
                data_points,
                aggregation_temporality: AggregationTemporality::Cumulative as i32,
            })),
            ..Default::default()
        });
    }
    let request = ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: tenant.map(tenant_resource),
            scope_metrics: vec![ScopeMetrics {
                metrics,
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    (request.encode_to_vec(), datapoints)
}

/// The phase a scrape at `elapsed` seconds belongs to, or `None` past the run.
fn phase_at(elapsed: u64, verify: &MetricVerify) -> Option<LoadPhase> {
    let churn_end = verify.steady_seconds + verify.churn_seconds;
    if elapsed < verify.steady_seconds {
        Some(LoadPhase::Steady)
    } else if elapsed < churn_end {
        Some(LoadPhase::Churn)
    } else if elapsed < churn_end + verify.explosion_seconds {
        Some(LoadPhase::Explosion)
    } else {
        None
    }
}

/// Drive the three phases in wall-clock, one scrape per interval, on one
/// connection: this measures how an engine's *index* behaves under series
/// pressure, not how many connections it can saturate — the throughput axis
/// belongs to the log bed's load phase.
pub async fn run_metric_load(cfg: &Config) -> MetricLoadOutcome {
    let verify = &cfg.metric_verify;
    let mut outcome = MetricLoadOutcome {
        phases: vec![
            (LoadPhase::Steady, PhaseTally::default()),
            (LoadPhase::Churn, PhaseTally::default()),
            (LoadPhase::Explosion, PhaseTally::default()),
        ],
        elapsed_seconds: 0.0,
        series_offered: 0,
    };
    let wire = cfg.push_wire();
    let push_headers = wire.headers(Signal::Metrics);
    let Some(push_path) = wire.path(Signal::Metrics) else {
        outcome.phases[0].1.errors = 1;
        outcome.phases[0].1.first_error = Some(format!(
            "target {} has no OTLP metrics ingest",
            cfg.target.name()
        ));
        return outcome;
    };
    let header = wire.tenant_header(&verify.tenant);
    let in_body = header.is_none().then_some(verify.tenant.as_str());
    let mut client = Client::new(cfg.push_address(), cfg.request_timeout());
    let mut population = LivePopulation::new(cfg.seed, verify);
    let interval = std::time::Duration::from_secs(verify.scrape_interval_seconds.max(1) as u64);
    let started = Instant::now();
    let mut exploded = false;

    for scrape in 0.. {
        let elapsed = started.elapsed().as_secs();
        let Some(phase) = phase_at(elapsed, verify) else {
            break;
        };
        match phase {
            LoadPhase::Steady => {}
            LoadPhase::Churn => population.churn(cfg.seed, verify.churn_replace_per_scrape),
            LoadPhase::Explosion => {
                if !exploded {
                    exploded = true;
                    population.explode(cfg.seed, verify.explosion_series);
                } else {
                    // The burst's series are minted once and never repeat: the
                    // recovery is what the rest of the phase measures.
                    population.exploded.clear();
                }
            }
        }
        let now_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_nanos().min(i64::MAX as u128) as i64)
            .unwrap_or(0);
        for (body, datapoints, series) in
            live_scrape_bodies(&mut population, scrape, now_ns, in_body, wire)
        {
            let sent = Instant::now();
            let result = client
                .request(&Request {
                    method: "POST",
                    path: push_path,
                    body: &body,
                    content_type: PUSH_CONTENT_TYPE,
                    headers: push_headers,
                    tenant: header.as_ref().map(|(name, value)| (*name, value.as_str())),
                })
                .await;
            let elapsed_ms = sent.elapsed().as_secs_f64() * 1000.0;
            let tally = &mut outcome
                .phases
                .iter_mut()
                .find(|(name, _)| *name == phase)
                .expect("every phase has a tally")
                .1;
            match result {
                Ok(response) => {
                    let rejected = rejected_datapoints(cfg.target, &response.body);
                    tally.record(response.status, datapoints, rejected, series, elapsed_ms);
                    if !(200..300).contains(&response.status) && response.status != 429 {
                        tally.errors += 1;
                        tally.first_error.get_or_insert_with(|| {
                            format!(
                                "{}: {}",
                                response.status,
                                String::from_utf8_lossy(&response.body)
                                    .chars()
                                    .take(300)
                                    .collect::<String>()
                            )
                        });
                    }
                }
                Err(error) => {
                    tally.scrapes += 1;
                    tally.datapoints_offered += datapoints;
                    tally.series_offered += series;
                    tally.series_rejected += series;
                    tally.errors += 1;
                    tally.latency.push(elapsed_ms);
                    tally.first_error.get_or_insert(error);
                }
            }
        }
        tokio::time::sleep(interval).await;
    }
    outcome.elapsed_seconds = started.elapsed().as_secs_f64();
    outcome.series_offered = population.minted;
    outcome
}
