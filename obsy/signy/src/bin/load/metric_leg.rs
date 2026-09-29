//! The load phase's metric leg: paced metric ingest and paced metric reads,
//! alongside the log workload rather than instead of it.
//!
//! Why it exists separately from `metric_workload`'s phases. Those drive a
//! dataset the comparison bed then checks answers against, over minutes, with
//! the population under the run's control. This one drives *a day*, against a
//! tenant that is also taking log pushes, and asks a different question: does
//! the metric path hold its residents flat while the log path runs at rate,
//! and does what was written hours ago still read back.
//!
//! Two things it deliberately does not do. It does not check answers for
//! equality — that is the bed's job and it needs a fixed corpus, which live
//! wall-clock timestamps are not. And it does not burst cardinality: the
//! explosion phase measures refusal, and refusal for hours would measure the
//! gate rather than the engine behind it. What it does check is weaker and
//! still worth a day: every shape must keep answering, and the shapes that
//! must return rows must keep returning them.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::time::Instant;

use crate::config::{Config, Signal, Target};
use crate::http::{Client, Request};
use crate::metric_workload::{
    COUNTER_NAMES, GAUGE_NAMES, HISTOGRAM_NAME, LivePopulation, PhaseTally, live_scrape_bodies,
    rejected_datapoints, service_names,
};
use crate::stats::LatencyPair;

const PUSH_CONTENT_TYPE: &str = "application/x-protobuf";

/// Keeps the leg's query draws distinct from every other seeded stream in the
/// run while staying a function of the run seed.
const METRIC_LEG_SEED_SALT: u64 = 0x0e_67_1c_a1;

#[derive(Default)]
pub struct MetricIngestOutcome {
    pub tally: PhaseTally,
    /// Distinct series identities the generator minted, which is what the
    /// engine's own `active_series` gauge is read against.
    pub series_offered: u64,
    /// Scrapes that could not be sent before the next one was due. A leg that
    /// falls behind is offering less than the run says it offered, and the
    /// number says so rather than the rate quietly sagging.
    pub scrapes_late: u64,
}

/// One scrape of a live population per interval, on one connection.
///
/// One connection on purpose: this is a garnish beside a 20 k eps log
/// workload, not a throughput measurement. What it must not do is stall the
/// run, so a scrape that overruns its interval is counted and the next one is
/// issued immediately rather than the leg sleeping itself further behind.
pub async fn metric_ingest_leg(
    cfg: Config,
    tenant: String,
    stop: Arc<AtomicBool>,
    deadline: Instant,
) -> MetricIngestOutcome {
    let mut outcome = MetricIngestOutcome::default();
    if cfg.target != Target::Signy {
        return outcome;
    }
    let Some(interval) = cfg.metric_leg_interval() else {
        return outcome;
    };
    let wire = cfg.push_wire();
    let Some(push_path) = wire.path(Signal::Metrics) else {
        outcome.tally.errors = 1;
        outcome.tally.first_error = Some(format!(
            "target {} has no OTLP metrics ingest",
            cfg.target.name()
        ));
        return outcome;
    };
    let push_headers = wire.headers(Signal::Metrics);
    let header = wire.tenant_header(&tenant);
    let in_body = header.is_none().then_some(tenant.as_str());
    let mut client = Client::new(cfg.push_address(), cfg.request_timeout());
    let mut population = LivePopulation::new(cfg.seed, &cfg.metric_verify);
    let mut intended = Instant::now();

    for scrape in 0.. {
        if stop.load(Ordering::Relaxed) || Instant::now() >= deadline {
            break;
        }
        if Instant::now() > intended {
            if scrape > 0 {
                outcome.scrapes_late += 1;
            }
            intended = Instant::now();
        } else {
            tokio::time::sleep_until(intended).await;
        }
        if cfg.metric_leg_churn_per_scrape > 0 {
            population.churn(cfg.seed, cfg.metric_leg_churn_per_scrape);
        }
        let now_ns = crate::unix_nanos().min(i64::MAX as u64) as i64;
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
            match result {
                Ok(response) => {
                    let rejected = rejected_datapoints(cfg.target, &response.body);
                    outcome
                        .tally
                        .record(response.status, datapoints, rejected, series, elapsed_ms);
                    if !(200..300).contains(&response.status) && response.status != 429 {
                        outcome.tally.errors += 1;
                        outcome.tally.first_error.get_or_insert_with(|| {
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
                    outcome.tally.scrapes += 1;
                    outcome.tally.datapoints_offered += datapoints;
                    outcome.tally.series_offered += series;
                    outcome.tally.series_rejected += series;
                    outcome.tally.unavailable += 1;
                    outcome.tally.latency.push(elapsed_ms);
                    outcome.tally.first_unavailable.get_or_insert(error);
                }
            }
        }
        intended += interval;
    }
    outcome.series_offered = population.minted();
    outcome
}

/// The five read shapes the leg cycles, each named for what it would break.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MetricQueryShape {
    /// A gauge's samples on a grid: the stored numbers read straight back.
    RawRange,
    /// A counter `rate`, folded across services — the dashboard panel.
    RateRange,
    /// One value now, the worst instance: the alert evaluation.
    InstantAlert,
    /// A p99 out of the histogram. Since issue #12 a histogram is one stored
    /// series and `_bucket{le=}` is synthesized on read, so this is the shape
    /// that would go empty if the synthesis stopped covering what a day's
    /// worth of parts and merges hold.
    QuantileP99,
    /// The catalog read: which series exist in the window.
    SeriesLookup,
}

pub const METRIC_QUERY_SHAPES: [MetricQueryShape; 5] = [
    MetricQueryShape::RawRange,
    MetricQueryShape::RateRange,
    MetricQueryShape::InstantAlert,
    MetricQueryShape::QuantileP99,
    MetricQueryShape::SeriesLookup,
];

/// Weights, not knobs: the mix is part of what the leg is, and the quantile
/// carries the most because it is the newest read path.
const METRIC_QUERY_WEIGHTS: [u32; 5] = [2, 2, 2, 3, 1];

impl MetricQueryShape {
    pub fn name(self) -> &'static str {
        match self {
            MetricQueryShape::RawRange => "metric_raw_range",
            MetricQueryShape::RateRange => "metric_rate_range",
            MetricQueryShape::InstantAlert => "metric_instant_alert",
            MetricQueryShape::QuantileP99 => "metric_quantile_p99",
            MetricQueryShape::SeriesLookup => "metric_series_lookup",
        }
    }

    /// Whether an empty answer from this shape is evidence of a defect. The
    /// catalog read is the exception only because a window with no series is
    /// a legitimate answer to it; the rest are asked about data the leg
    /// itself has been pushing for the whole window.
    pub fn must_return_rows(self) -> bool {
        self != MetricQueryShape::SeriesLookup
    }
}

#[derive(Default)]
pub struct MetricQueryOutcome {
    pub steady: LatencyPair,
    pub per_shape: BTreeMap<&'static str, LatencyPair>,
    pub shape_counts: BTreeMap<&'static str, u64>,
    /// Answers issued past the settling floor, which are the only ones the
    /// empty-answer count below is taken over. Reported beside it so the two
    /// numbers share a denominator.
    pub shape_judged: BTreeMap<&'static str, u64>,
    pub shape_rows: BTreeMap<&'static str, u64>,
    /// Answers that were `200` and carried no series. The gate reads this:
    /// a read path that quietly stops finding what was written answers
    /// exactly this way, and a status-code gate would call it a pass.
    pub shape_empty: BTreeMap<&'static str, u64>,
    /// Empty answers a shape gave while it was still catching up from a
    /// window where nobody answered at all.
    ///
    /// An instant query asks about the last minute by definition, and the
    /// collector in front of the engine holds an outage's exports until it has
    /// drained them in order. So the minute after the engine returns is a
    /// minute the engine legitimately has nothing for, and counting it as "the
    /// read path stopped finding what was written" says the opposite of what
    /// happened. A shape enters this state when a read goes unanswered and
    /// leaves it the moment that same shape returns rows again -- so a leg that
    /// never recovers still fails, and no window has to be guessed at.
    pub shape_empty_recovering: BTreeMap<&'static str, u64>,
    /// Shapes waiting to see rows again after an unanswered read.
    pub recovering: std::collections::BTreeSet<&'static str>,
    /// How long a leg is allowed to be catching up after a read nobody
    /// answered, reported so the number the judging used is in the artifact.
    pub recovery_grace_seconds: u64,
    /// When each empty answer happened, in seconds since the leg started, and
    /// whether the shape was catching up at the time.
    ///
    /// A count alone cannot be argued with: it says a shape went quiet and not
    /// whether that was a minute the collector had not delivered yet. Bounded
    /// so a leg that goes quiet for an hour does not turn into a list of every
    /// second of it.
    pub empty_at: Vec<(&'static str, f64, bool)>,
    pub answered: u64,
    pub errors: u64,
    pub throttled: u64,
    /// Reads nobody answered. A soak stops the engine on purpose, so this is
    /// availability, not correctness, and it is counted apart from `errors`.
    pub unavailable: u64,
    pub statuses: BTreeMap<u16, u64>,
    pub first_error: Option<String>,
    pub first_unavailable: Option<String>,
    /// Answers issued past the settling floor, across every shape, and the
    /// floor itself. A run shorter than the floor judges nothing, and a report
    /// that said `pass` without saying that would be claiming a check it never
    /// made.
    pub judged_total: u64,
    pub settling_seconds: u64,
}

/// Metric reads at a paced rate, on one connection, cycling the shapes above.
pub async fn metric_query_leg(
    cfg: Config,
    tenant: String,
    stop: Arc<AtomicBool>,
    deadline: Instant,
    warmup_end: Instant,
) -> MetricQueryOutcome {
    let mut outcome = MetricQueryOutcome::default();
    // The leg's own clock, so an empty answer can be put beside the fault log.
    let leg_start = Instant::now();
    // When a read last went unanswered, and how long after that the leg is
    // still allowed to be catching up.
    //
    // A single non-empty answer is not proof the pipeline has caught up: the
    // collector drains an outage's exports in order, and an instant query
    // asking about the last minute can find rows once and a gap again a
    // moment later. Measured over three faulted runs, every empty answer fell
    // in a burst starting within two seconds of the engine returning and
    // lasting 50 to 111 seconds, against outages of 57 to 208; the 24-hour
    // soak's collector took 157 to 184 seconds to drain a 180-second outage.
    // The grace is set above all of those rather than at them.
    let recovery_grace = std::time::Duration::from_secs(cfg.metric_recovery_grace_seconds);
    outcome.recovery_grace_seconds = cfg.metric_recovery_grace_seconds;
    let mut last_unavailable: Option<Instant> = None;
    if cfg.target != Target::Signy {
        return outcome;
    }
    let Some(interval) = cfg.metric_query_interval() else {
        return outcome;
    };
    let mut client = Client::new(&cfg.http_address, cfg.request_timeout());
    let mut rng = signy::corpus::Rng::new(cfg.seed ^ METRIC_LEG_SEED_SALT);
    let (header_name, header_value) = cfg.target.read_tenant_header(&tenant);
    let services = service_names(cfg.metric_verify.services);
    let mut intended = Instant::now();
    // The leg's own settling floor, on top of the run's warmup. At the first
    // instant there is nothing to find: a `rate` needs two samples inside its
    // range and a quantile needs a bucket window, so a read issued before the
    // leg has pushed that much answers empty *correctly*. Counting those would
    // make the gate fire on the clock rather than on the engine, and a soak
    // that cries wolf in its first minute is a soak nobody reads the verdict
    // of.
    let settling_seconds =
        2 * cfg.metric_leg_scrape_seconds + cfg.metric_verify.range_seconds.max(0) as u64;
    outcome.settling_seconds = settling_seconds;
    let settled_at = Instant::now() + std::time::Duration::from_secs(settling_seconds);

    while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
        tokio::time::sleep_until(intended).await;
        intended += interval;
        let shape = pick_shape(&mut rng);
        let path = query_path(&cfg, shape, &services, &mut rng);
        *outcome.shape_counts.entry(shape.name()).or_default() += 1;

        let sent = Instant::now();
        let result = client
            .request(&Request {
                method: "GET",
                path: &path,
                body: &[],
                content_type: "",
                tenant: Some((header_name, header_value.as_str())),
                headers: &[],
            })
            .await;
        let done = Instant::now();
        let service_ms = done.saturating_duration_since(sent).as_secs_f64() * 1000.0;
        let queueing_ms = sent
            .saturating_duration_since(intended - interval)
            .as_secs_f64()
            * 1000.0;
        let steady = sent >= warmup_end && sent >= settled_at;

        match result {
            Ok(response) => {
                *outcome.statuses.entry(response.status).or_default() += 1;
                match response.status {
                    200 => {
                        outcome.answered += 1;
                        let rows = ndjson_rows(&response.body);
                        *outcome.shape_rows.entry(shape.name()).or_default() += rows;
                        if steady {
                            *outcome.shape_judged.entry(shape.name()).or_default() += 1;
                            outcome.judged_total += 1;
                            if rows == 0 {
                                let catching_up = outcome.recovering.contains(shape.name())
                                    || last_unavailable
                                        .is_some_and(|at| at.elapsed() < recovery_grace);
                                if outcome.empty_at.len() < 512 {
                                    outcome.empty_at.push((
                                        shape.name(),
                                        leg_start.elapsed().as_secs_f64(),
                                        catching_up,
                                    ));
                                }
                                if catching_up {
                                    *outcome
                                        .shape_empty_recovering
                                        .entry(shape.name())
                                        .or_default() += 1;
                                } else {
                                    *outcome.shape_empty.entry(shape.name()).or_default() += 1;
                                }
                            } else {
                                outcome.recovering.remove(shape.name());
                            }
                            outcome.steady.record(queueing_ms, service_ms);
                            outcome
                                .per_shape
                                .entry(shape.name())
                                .or_default()
                                .record(queueing_ms, service_ms);
                        }
                    }
                    429 => outcome.throttled += 1,
                    status => {
                        outcome.errors += 1;
                        outcome.first_error.get_or_insert_with(|| {
                            format!(
                                "{status} on {path}: {}",
                                String::from_utf8_lossy(&response.body)
                                    .chars()
                                    .take(200)
                                    .collect::<String>()
                            )
                        });
                    }
                }
            }
            Err(error) => {
                outcome.unavailable += 1;
                outcome.first_unavailable.get_or_insert(error);
                last_unavailable = Some(Instant::now());
                for shape in METRIC_QUERY_SHAPES {
                    outcome.recovering.insert(shape.name());
                }
            }
        }
    }
    outcome
}

fn pick_shape(rng: &mut signy::corpus::Rng) -> MetricQueryShape {
    let total: u32 = METRIC_QUERY_WEIGHTS.iter().sum();
    let mut roll = (rng.next_u64() % total as u64) as u32;
    for (index, weight) in METRIC_QUERY_WEIGHTS.iter().enumerate() {
        if roll < *weight {
            return METRIC_QUERY_SHAPES[index];
        }
        roll -= weight;
    }
    MetricQueryShape::RawRange
}

/// The window every range shape asks about, relative to now.
///
/// Relative rather than absolute because the leg's data is wall-clock: there
/// is no anchor to cut fixed windows out of. It is a knob because retention
/// is one — a window longer than the tenant keeps would read empty and the
/// empty-answer gate would fire on the run's own configuration.
fn query_path(
    cfg: &Config,
    shape: MetricQueryShape,
    services: &[String],
    rng: &mut signy::corpus::Rng,
) -> String {
    let verify = &cfg.metric_verify;
    let window = format!("-{}s", cfg.metric_query_window_seconds);
    let step = format!("{}s", verify.step_seconds);
    let range = format!("{}s", verify.range_seconds);
    let service = &services[rng.below(services.len())];
    let instance = format!("instance-{}", rng.below(verify.instances_per_service));
    let gauge = GAUGE_NAMES[rng.below(verify.gauges.min(GAUGE_NAMES.len()))];
    let counter = COUNTER_NAMES[rng.below(verify.counters.min(COUNTER_NAMES.len()))];

    let mut encoded = url::form_urlencoded::Serializer::new(String::new());
    let route = match shape {
        MetricQueryShape::RawRange => {
            encoded.append_pair("metric", gauge);
            encoded.append_pair("attr", &format!("service={service}"));
            encoded.append_pair("attr", &format!("instance={instance}"));
            encoded.append_pair("start", &window);
            encoded.append_pair("step", &step);
            "/signy/api/v1/metrics/query"
        }
        MetricQueryShape::RateRange => {
            encoded.append_pair("metric", counter);
            encoded.append_pair("start", &window);
            encoded.append_pair("step", &step);
            encoded.append_pair("func", "rate");
            encoded.append_pair("range", &range);
            encoded.append_pair("agg", "sum");
            encoded.append_pair("by", "service");
            "/signy/api/v1/metrics/query"
        }
        MetricQueryShape::InstantAlert => {
            encoded.append_pair("metric", counter);
            encoded.append_pair("func", "rate");
            encoded.append_pair("range", &range);
            encoded.append_pair("agg", "max");
            "/signy/api/v1/metrics/instant"
        }
        MetricQueryShape::QuantileP99 => {
            encoded.append_pair("metric", HISTOGRAM_NAME);
            encoded.append_pair("q", "0.99");
            encoded.append_pair("attr", &format!("service={service}"));
            encoded.append_pair("start", &window);
            encoded.append_pair("step", &step);
            encoded.append_pair("range", &range);
            "/signy/api/v1/metrics/quantile"
        }
        MetricQueryShape::SeriesLookup => {
            encoded.append_pair("metric", gauge);
            encoded.append_pair("start", &window);
            "/signy/api/v1/metrics/series"
        }
    };
    format!("{route}?{}", encoded.finish())
}

/// Series lines an NDJSON metric response carried.
///
/// Tolerant in the same way the log side's row count is: a body this cannot
/// read counts as zero rather than failing the query, because the status code
/// is the error gate and this number describes what came back.
fn ndjson_rows(body: &[u8]) -> u64 {
    body.split(|byte| *byte == b'\n')
        .filter(|line| !line.iter().all(u8::is_ascii_whitespace))
        .count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_for_test() -> Config {
        let mut cfg = Config::from_env().expect("the env defaults build a config");
        cfg.target = Target::Signy;
        cfg.metric_query_window_seconds = 900;
        cfg
    }

    #[test]
    fn every_shape_addresses_a_first_party_metric_route_and_names_its_metric() {
        let cfg = config_for_test();
        let services = service_names(cfg.metric_verify.services);
        let mut rng = signy::corpus::Rng::new(1);
        for shape in METRIC_QUERY_SHAPES {
            let path = query_path(&cfg, shape, &services, &mut rng);
            assert!(
                path.starts_with("/signy/api/v1/metrics/"),
                "{shape:?}: {path}"
            );
            assert!(path.contains("metric="), "{shape:?}: {path}");
        }
    }

    #[test]
    fn the_range_shapes_ask_about_the_configured_window() {
        let mut cfg = config_for_test();
        cfg.metric_query_window_seconds = 1800;
        let services = service_names(cfg.metric_verify.services);
        let mut rng = signy::corpus::Rng::new(2);
        for shape in [
            MetricQueryShape::RawRange,
            MetricQueryShape::RateRange,
            MetricQueryShape::QuantileP99,
        ] {
            let path = query_path(&cfg, shape, &services, &mut rng);
            assert!(path.contains("start=-1800s"), "{shape:?}: {path}");
        }
    }

    #[test]
    fn the_quantile_shape_asks_the_histogram_by_its_base_name() {
        let cfg = config_for_test();
        let services = service_names(cfg.metric_verify.services);
        let mut rng = signy::corpus::Rng::new(3);
        let path = query_path(&cfg, MetricQueryShape::QuantileP99, &services, &mut rng);
        assert!(
            path.starts_with("/signy/api/v1/metrics/quantile?"),
            "{path}"
        );
        assert!(path.contains(&format!("metric={HISTOGRAM_NAME}")), "{path}");
        assert!(path.contains("q=0.99"), "{path}");
        // `range` is required by the route: a bucket count without a window is
        // a lifetime total, and the route refuses rather than answering one.
        assert!(path.contains("range="), "{path}");
    }

    #[test]
    fn only_the_catalog_read_may_answer_nothing() {
        for shape in METRIC_QUERY_SHAPES {
            assert_eq!(
                shape.must_return_rows(),
                shape != MetricQueryShape::SeriesLookup,
                "{shape:?}"
            );
        }
    }

    #[test]
    fn an_ndjson_body_counts_its_series_lines() {
        assert_eq!(ndjson_rows(b""), 0);
        assert_eq!(ndjson_rows(b"\n"), 0);
        assert_eq!(ndjson_rows(b"{\"labels\":{}}\n"), 1);
        assert_eq!(ndjson_rows(b"{\"a\":1}\n{\"b\":2}\n"), 2);
        assert_eq!(ndjson_rows(b"{\"a\":1}\n{\"b\":2}"), 2);
    }

    #[test]
    fn the_shape_mix_reaches_every_shape() {
        let mut rng = signy::corpus::Rng::new(4);
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..1000 {
            seen.insert(pick_shape(&mut rng).name());
        }
        assert_eq!(seen.len(), METRIC_QUERY_SHAPES.len());
    }
}
