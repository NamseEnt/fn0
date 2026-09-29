//! The load harness.
//!
//! The one it replaces could not measure what it claimed to, and every number
//! it produced has been retired (`docs/LOAD_RESULTS.md`, retirement header;
//! `docs/VISION.md`, "The ruler comes before the work"). The defects it had
//! and what is done about each of them:
//!
//! * **One connection, one request in flight, `Connection: close`.** Now N
//!   keep-alive connections per workload, over the hand-rolled client in
//!   `http.rs`.
//! * **Uncorrected coordinated omission.** The pacer keeps a nominal schedule
//!   and the stopwatch now starts at the **intended** send. Both numbers are
//!   reported: service time from the actual send, response time from the
//!   intended one. The gap between them is the signal that the offered rate
//!   was not achievable, and it is exactly the signal the old harness deleted.
//! * **Cardinality 1 and near-zero entropy.** The corpus is
//!   `signy::corpus`, the same generator the benches measure.
//! * **Reads never contended with writes.** Queries are an independent
//!   workload with their own rate and their own connections.
//! * **Percentiles over eight samples.** Every percentile carries its sample
//!   count, and one the count cannot support is `null` with the reason.
//! * **RSS from a coarse `ps` poll.** `VmHWM` out of `/proc/<pid>/status`, and
//!   a sampled `VmRSS` series for the shape. A run that cannot read it fails.
//! * **`std::thread::sleep` inside `#[tokio::main]`.** Every wait is
//!   `tokio::time`.

mod config;
mod http;
mod matrix;
mod metric_leg;
mod metric_matrix;
mod metric_workload;
mod otlp;
mod probe;
mod stats;
mod workload;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use prost014::Message;

use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use opentelemetry_proto::tonic::trace::v1::{ResourceSpans, ScopeSpans, Span};
use serde_json::{Value, json};
use tokio::sync::{Mutex, mpsc};
use tokio::time::Instant;

use config::{Config, Phase, Signal, Target};
use http::{Client, Request};
use stats::{GaugeSeries, LatencyPair, target_row, wal_backlog_drains};
use workload::{ArrivalOrder, PushGenerator, QUERY_SHAPES, QueryGenerator, result_rows};

const PUSH_CONTENT_TYPE: &str = "application/x-protobuf";
/// Row group size the engine itself flushes at, so the measured compression
/// ratio is the ratio this data will actually get on disk.
const ROW_GROUP_SIZE: usize = 8192;
/// Keeps the trace ids of a run distinct from its log identifiers while
/// staying a function of the run seed.
const OTLP_SEED_SALT: u64 = 0x07_1e_50_ed;

struct PushJob {
    intended: Instant,
    body: workload::PushBody,
}

struct QueryJob {
    intended: Instant,
    plan: workload::QueryPlan,
}

#[derive(Default)]
struct PushOutcome {
    steady: LatencyPair,
    warmup: LatencyPair,
    throttled_latency: LatencyPair,
    accepted: u64,
    throttled: u64,
    errors: u64,
    /// Reads nobody answered: the address refused, or the connection went
    /// while the request was out. A soak stops the engine on purpose, so this
    /// is a statement about availability and not about correctness, and
    /// mixing it into `errors` is what made a fault schedule look like a
    /// defect.
    unavailable: u64,
    events_accepted: u64,
    events_offered: u64,
    wire_bytes: u64,
    line_bytes: u64,
    encoded_bytes: u64,
    out_of_order_entries: u64,
    max_lateness_ms: u64,
    streams_sent: u64,
    connects: u64,
    statuses: BTreeMap<u16, u64>,
    first_error: Option<String>,
    first_unavailable: Option<String>,
}

impl PushOutcome {
    fn merge(&mut self, other: PushOutcome) {
        self.steady.extend(&other.steady);
        self.warmup.extend(&other.warmup);
        self.throttled_latency.extend(&other.throttled_latency);
        self.accepted += other.accepted;
        self.throttled += other.throttled;
        self.errors += other.errors;
        self.unavailable += other.unavailable;
        self.events_accepted += other.events_accepted;
        self.events_offered += other.events_offered;
        self.wire_bytes += other.wire_bytes;
        self.line_bytes += other.line_bytes;
        self.encoded_bytes += other.encoded_bytes;
        self.out_of_order_entries += other.out_of_order_entries;
        self.max_lateness_ms = self.max_lateness_ms.max(other.max_lateness_ms);
        self.streams_sent += other.streams_sent;
        self.connects += other.connects;
        for (status, count) in other.statuses {
            *self.statuses.entry(status).or_default() += count;
        }
        self.first_error = self.first_error.take().or(other.first_error);
        self.first_unavailable = self.first_unavailable.take().or(other.first_unavailable);
    }
}

#[derive(Default)]
struct QueryOutcome {
    steady: LatencyPair,
    per_shape: BTreeMap<&'static str, LatencyPair>,
    shape_counts: BTreeMap<&'static str, u64>,
    answered: u64,
    errors: u64,
    throttled: u64,
    /// Reads nobody answered: the address refused, or the connection went
    /// while the request was out. A soak stops the engine on purpose, so this
    /// is a statement about availability and not about correctness, and
    /// mixing it into `errors` is what made a fault schedule look like a
    /// defect.
    unavailable: u64,
    restore_probes: u64,
    restore_rows: u64,
    restore_probes_with_rows: u64,
    rows_returned: u64,
    connects: u64,
    statuses: BTreeMap<u16, u64>,
    first_error: Option<String>,
    first_unavailable: Option<String>,
}

impl QueryOutcome {
    fn merge(&mut self, other: QueryOutcome) {
        self.steady.extend(&other.steady);
        for (shape, latency) in other.per_shape {
            self.per_shape.entry(shape).or_default().extend(&latency);
        }
        for (shape, count) in other.shape_counts {
            *self.shape_counts.entry(shape).or_default() += count;
        }
        self.answered += other.answered;
        self.errors += other.errors;
        self.throttled += other.throttled;
        self.unavailable += other.unavailable;
        self.restore_probes += other.restore_probes;
        self.restore_rows += other.restore_rows;
        self.restore_probes_with_rows += other.restore_probes_with_rows;
        self.rows_returned += other.rows_returned;
        self.connects += other.connects;
        for (status, count) in other.statuses {
            *self.statuses.entry(status).or_default() += count;
        }
        self.first_error = self.first_error.take().or(other.first_error);
    }
}

/// Counters the engine resets when it restarts, banked across the restarts a
/// soak causes on purpose.
///
/// A soak reads `dropped_by_reason` from one scrape at the end, which is only
/// ever the last generation's tally: a day with five restarts reported the
/// drops of its final stretch and called it the run's. The sampler already
/// scrapes `/metrics` on its own interval, so the whole-run figure costs
/// nothing but noticing when a value goes backwards -- which a monotonic
/// counter only does by starting again.
#[derive(Default)]
struct RestartSafeBreakdown {
    /// label -> value seen in the generation running now.
    current: BTreeMap<String, u64>,
    /// label -> everything earlier generations had counted before they went.
    banked: BTreeMap<String, u64>,
    restarts_seen: u64,
}

impl RestartSafeBreakdown {
    fn observe(&mut self, seen: BTreeMap<String, u64>) {
        // One counter going backwards means the process behind all of them
        // did, so the whole tally is banked rather than the single label.
        let restarted = seen
            .iter()
            .any(|(label, value)| *value < self.current.get(label).copied().unwrap_or(0))
            || (!self.current.is_empty() && seen.is_empty());
        if restarted {
            for (label, value) in std::mem::take(&mut self.current) {
                *self.banked.entry(label).or_default() += value;
            }
            self.restarts_seen += 1;
        }
        for (label, value) in seen {
            self.current.insert(label, value);
        }
    }

    fn total(&self) -> BTreeMap<String, u64> {
        let mut out = self.banked.clone();
        for (label, value) in &self.current {
            *out.entry(label.clone()).or_default() += *value;
        }
        out
    }
}

#[derive(Default)]
struct SampleOutcome {
    wal_backlog: GaugeSeries,
    memtable_bytes: GaugeSeries,
    part_count: GaugeSeries,
    /// Capacity-probe-only shape gauges.  The server emits these only when its
    /// own raw-capacity switch is enabled, so normal load runs retain no
    /// structural series and pay only the existing `/metrics` scrape.
    series_states_len: GaugeSeries,
    series_states_capacity: GaugeSeries,
    series_buffers_len: GaugeSeries,
    series_buffers_capacity: GaugeSeries,
    series_buffers_empty: GaugeSeries,
    series_buffers_inline: GaugeSeries,
    series_buffers_stream: GaugeSeries,
    series_flushing_series: GaugeSeries,
    series_flushing_tenants: GaugeSeries,
    rss: GaugeSeries,
    anon: GaugeSeries,
    /// The ingest drop tally, summed over every generation of the engine this
    /// run went through rather than read off the last one.
    dropped_resources: RestartSafeBreakdown,
    health_samples: u64,
    health_healthy: u64,
    scrape_errors: u64,
    vm_hwm_bytes: Option<u64>,
    anon_peak_bytes: Option<u64>,
    file_bytes_end: Option<u64>,
    rss_error: Option<String>,
}

#[derive(Default)]
struct OtlpOutcome {
    latency: LatencyPair,
    sent: u64,
    spans_sent: u64,
    errors: u64,
    connected: bool,
    /// Traces read back by id after they were sent, and what came of it.
    ///
    /// The three ways a readback can end badly are three different statements
    /// and are counted apart, because only one of them is the engine being
    /// wrong:
    ///
    /// * `missing` — the timeline route answered 404 after the readback had
    ///   spent its whole patience. The trace was acked and is not there.
    /// * `short` — it answered with fewer spans than were exported.
    /// * `quota_rejected` — it answered 429. That is the tenant's own
    ///   concurrency limit doing what it is for, not a lost trace, and it used
    ///   to be counted as one.
    /// * `unexpected_status` — anything else. Kept apart from `missing` so a
    ///   new failure mode cannot hide inside a number that means data loss.
    ///
    /// `missing`, `short` and `unexpected_status` fail the run. The other two
    /// do not.
    verify_attempts: u64,
    verified: u64,
    missing: u64,
    short: u64,
    quota_rejected: u64,
    unexpected_status: u64,
    /// Probes that could not be made: the read address did not answer at all.
    /// Not a wrong answer — a soak takes the engine down on purpose — so it is
    /// counted and reported rather than failed.
    unreachable: u64,
    /// Traces asked for again, because a backlog in front of the engine is not
    /// the engine losing a trace. A soak's outage plus the collector's drain
    /// behind it outlasts a single retry, which is why the budget is a number
    /// rather than "once".
    retried: u64,
    /// Traces the readback gave up on with its patience spent, still counted
    /// in `missing`. Reported so a run can say whether the budget was the
    /// binding constraint.
    gave_up_after_budget: u64,
    /// When each trace was called missing, in seconds since the leg started.
    ///
    /// A count says the engine could not produce a trace it acked and nothing
    /// about whether that was a minute the collector had not delivered yet.
    /// Put beside the run's own fault log it says which. Bounded, because a
    /// read path that has stopped answering should not become a list of every
    /// trace it did not answer for.
    missing_at: Vec<f64>,
    /// Set when the run was long enough for a sent trace to come back. A run
    /// that stopped before the first probe was due proves nothing about the
    /// read path, and must not read as if it had.
    verification_expected: bool,
    search_probes: u64,
    search_empty: u64,
    first_verify_error: Option<String>,
}

#[tokio::main]
async fn main() {
    let cfg = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    // Phase and target are validated together before any workload is built:
    // the log phases against a metrics engine, or the metric phases against a
    // logs-only engine, would measure nothing either claim is about — and the
    // refusal here is what lets the per-target code below treat the wrong
    // pairing as unreachable instead of half-supporting it.
    let target_fits = match cfg.phase {
        Phase::Load | Phase::Seed | Phase::Matrix => {
            matches!(
                cfg.target,
                Target::Signy | Target::Loki | Target::VictoriaLogs
            )
        }
        Phase::MetricSeed | Phase::MetricMatrix => {
            matches!(cfg.target, Target::Signy | Target::VictoriaMetrics)
        }
        Phase::MetricLoad => matches!(
            cfg.target,
            Target::Signy | Target::VictoriaMetrics | Target::Mimir
        ),
    };
    if !target_fits {
        eprintln!(
            "target {} does not answer the {:?} phase: the log phases accept signy, loki \
and victorialogs; metric seed/matrix accept signy and victoriametrics; metric-load also \
accepts mimir",
            cfg.target.name(),
            cfg.phase
        );
        std::process::exit(2);
    }
    match cfg.phase {
        Phase::Load => run_load(cfg).await,
        Phase::Seed | Phase::Matrix => run_verify(cfg).await,
        Phase::MetricSeed | Phase::MetricMatrix | Phase::MetricLoad => run_metric_verify(cfg).await,
    }
}

/// The metric analogue of `run_verify`: the fixed metric dataset and the fn0
/// shape matrix over it (M14, issue #8).
async fn run_metric_verify(cfg: Config) {
    if let Err(error) = metric_workload::require_metric_anchor(&cfg.metric_verify) {
        eprintln!("{error}");
        std::process::exit(2);
    }
    if let Err(error) = wait_for_ready(&cfg).await {
        eprintln!("server at {} is not ready: {error}", cfg.http_address);
        std::process::exit(1);
    }
    if let Err(error) = wait_for_collector(&cfg).await {
        eprintln!(
            "collector at {} is not answering: {error}",
            cfg.push_address()
        );
        std::process::exit(1);
    }
    let memory_source = cfg.memory_source();
    let corpus = metric_workload::metric_corpus(cfg.seed, &cfg.metric_verify);
    if let Err(error) = onboard_tenants(&cfg, &[corpus.tenant.as_str()]).await {
        eprintln!("{error}");
        std::process::exit(1);
    }
    eprintln!(
        "{} phase {:?}: {} instruments, {} decomposed series, {} scrapes",
        cfg.target.name(),
        cfg.phase,
        corpus.instruments.len(),
        corpus.decomposed_series_count(),
        corpus.scrapes
    );

    let run_start = Instant::now();
    let stop = Arc::new(AtomicBool::new(false));
    let anon_peak = Arc::new(AtomicU64::new(0));
    let anon_watch = tokio::spawn({
        let stop = stop.clone();
        let anon_peak = anon_peak.clone();
        let source = cfg.memory_source();
        async move {
            while !stop.load(Ordering::Relaxed) {
                if let Ok(source) = source.as_ref()
                    && let Ok(memory) = source.read()
                    && let Some(anon) = memory.anon_bytes
                {
                    anon_peak.fetch_max(anon, Ordering::Relaxed);
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
    });
    // The structural series gauges are emitted only by the server's
    // capacity-probe mode.  Reuse the normal sampler there so the result keeps
    // the same sampled timeline and peak summaries as the long load report;
    // ordinary metric seed/matrix runs do not start this extra scrape loop.
    let series_sampler = (cfg.capacity_probe && cfg.target == Target::Signy)
        .then(|| tokio::spawn(sampler(cfg.clone(), stop.clone(), run_start)));

    let mut report = json!({
        "phase": match cfg.phase {
            Phase::MetricSeed => "metric-seed",
            Phase::MetricLoad => "metric-load",
            _ => "metric-matrix",
        },
        "target": cfg.target.name(),
        "run": {
            "build_revision": config::build_revision(),
            "machine_profile": config::machine_profile(),
            "seed": cfg.seed,
        },
        "metric_verify": {
            "tenant": corpus.tenant,
            "anchor_ns": cfg.metric_verify.anchor_ns,
            "span_ns": cfg.metric_span_ns(),
            "scrapes": corpus.scrapes,
            "instruments": corpus.instruments.len(),
            "datapoints": corpus.datapoint_count(),
            "decomposed_samples": corpus.decomposed_sample_count(),
            "decomposed_series": corpus.decomposed_series_count(),
        },
    });

    let ok;
    match cfg.phase {
        Phase::MetricSeed => {
            let outcome = metric_workload::run_metric_seed(&cfg, &corpus).await;
            ok = outcome.errors == 0 && outcome.datapoints == corpus.datapoint_count();
            report["seed"] = json!({
                "pushes": outcome.pushes,
                "datapoints": outcome.datapoints,
                "decomposed_samples": outcome.decomposed_samples,
                "wire_bytes": outcome.wire_bytes,
                "retries": outcome.retries,
                "errors": outcome.errors,
                "rejected_datapoints": outcome.rejected_datapoints,
                "statuses": outcome
                    .statuses
                    .iter()
                    .map(|(status, count)| (status.to_string(), *count))
                    .collect::<BTreeMap<_, _>>(),
                "first_error": outcome.first_error,
                "elapsed_seconds": outcome.elapsed_seconds,
                "complete": ok,
            });
        }
        Phase::MetricMatrix => {
            let outcome = metric_matrix::run_metric_matrix(&cfg).await;
            ok = outcome["shapes"]
                .as_object()
                .is_some_and(|shapes| shapes.values().all(|shape| shape["errors"] == json!(0)));
            report["matrix"] = outcome;
        }
        Phase::MetricLoad => {
            let outcome = metric_workload::run_metric_load(&cfg).await;
            // The steady phase is the gate: a budget met by refusing the
            // offered load was never exercised. The churn and explosion
            // phases' refusals are the designed behavior and are reported,
            // not gated.
            let steady = outcome
                .phases
                .iter()
                .find(|(phase, _)| *phase == metric_workload::LoadPhase::Steady)
                .map(|(_, tally)| tally);
            ok = outcome.phases.iter().all(|(_, tally)| tally.errors == 0)
                && steady.is_some_and(|tally| tally.acceptance() >= 0.9);
            report["load"] = json!({
                "elapsed_seconds": outcome.elapsed_seconds,
                "series_offered": outcome.series_offered,
                "phases": outcome
                    .phases
                    .iter()
                    .map(|(phase, tally)| {
                        (
                            phase.name().to_string(),
                            json!({
                                "scrapes": tally.scrapes,
                                "datapoints_offered": tally.datapoints_offered,
                                "datapoints_accepted": tally.datapoints_accepted,
                                "datapoints_rejected": tally.datapoints_rejected,
                                "requests_refused": tally.requests_refused,
                                "series_offered": tally.series_offered,
                                "series_accepted": tally.series_accepted,
                                "series_rejected": tally.series_rejected,
                                "acceptance": tally.acceptance(),
                                "errors": tally.errors,
                                "first_error": tally.first_error,
                                "statuses": tally
                                    .statuses
                                    .iter()
                                    .map(|(status, count)| (status.to_string(), *count))
                                    .collect::<BTreeMap<_, _>>(),
                                "latency_ms": tally.latency.clone().summary(),
                            }),
                        )
                    })
                    .collect::<serde_json::Map<_, _>>(),
            });
        }
        Phase::Load | Phase::Seed | Phase::Matrix => {
            unreachable!("run_metric_verify is only reached for the metric phases")
        }
    }

    stop.store(true, Ordering::Relaxed);
    let _ = anon_watch.await;
    let samples = match series_sampler {
        Some(task) => task.await.unwrap_or_default(),
        None => SampleOutcome::default(),
    };
    let end_metrics = if cfg.capacity_probe && cfg.target == Target::Signy {
        let mut client = Client::new(&cfg.http_address, cfg.request_timeout());
        scrape(&mut client).await.unwrap_or_default()
    } else {
        probe::Metrics::new()
    };
    let memory_after = memory_source
        .as_ref()
        .ok()
        .and_then(|source| source.read().ok());
    report["memory"] = json!({
        "source": memory_source.as_ref().map(|source| source.describe()).ok(),
        "error": memory_source.as_ref().err(),
        "peak_bytes": memory_after.as_ref().map(|memory| memory.vm_hwm_bytes),
        "current_bytes": memory_after.as_ref().map(|memory| memory.vm_rss_bytes),
        "anon_peak_bytes": match anon_peak.load(Ordering::Relaxed) {
            0 => Value::Null,
            peak => json!(peak),
        },
        "anon_bytes_end": memory_after.as_ref().and_then(|memory| memory.anon_bytes),
        "file_bytes_end": memory_after.as_ref().and_then(|memory| memory.file_bytes),
    });
    if cfg.capacity_probe && !samples.series_states_len.samples.is_empty() {
        report["series_memory"] = series_memory_report(&samples, &end_metrics);
    }
    report["config"] = serde_json::to_value(&cfg).expect("config serialization");
    report["verdict"] = json!(if ok { "PASS" } else { "FAIL" });

    let rendered = serde_json::to_string_pretty(&report).expect("report serialization");
    if let Some(path) = cfg.result_path.as_ref()
        && let Err(error) = std::fs::write(path, format!("{rendered}\n"))
    {
        eprintln!("could not write {path}: {error}");
    }
    println!("{rendered}");
    std::process::exit(if ok { 0 } else { 1 });
}

/// The seed and matrix phases, which drive the fixed dataset the comparison's
/// query numbers and its row-equality check are both taken over.
async fn run_verify(cfg: Config) {
    if let Err(error) = matrix::require_anchor(&cfg.verify) {
        eprintln!("{error}");
        std::process::exit(2);
    }
    if let Err(error) = wait_for_ready(&cfg).await {
        eprintln!("server at {} is not ready: {error}", cfg.http_address);
        std::process::exit(1);
    }
    if let Err(error) = wait_for_collector(&cfg).await {
        eprintln!(
            "collector at {} is not answering: {error}",
            cfg.push_address()
        );
        std::process::exit(1);
    }
    let memory_source = cfg.memory_source();
    let memory_before = memory_source
        .as_ref()
        .ok()
        .and_then(|source| source.read().ok());

    eprintln!(
        "{} phase {:?}: generating verification corpus of {} rows",
        cfg.target.name(),
        cfg.phase,
        cfg.verify.rows
    );
    let corpus = matrix::verify_corpus(&cfg);
    let tenants: Vec<&str> = corpus.tenant_ids.iter().map(|id| id.as_str()).collect();
    if let Err(error) = onboard_tenants(&cfg, &tenants).await {
        eprintln!("{error}");
        std::process::exit(1);
    }

    // Anonymous memory has to be *sampled*, not read at the ends: the cgroup's
    // own `memory.peak` is a high-water mark but includes reclaimable page
    // cache, and this phase reads large data files, so the page cache alone
    // would carry the peak to the limit and say nothing.
    let stop = Arc::new(AtomicBool::new(false));
    let anon_peak = Arc::new(AtomicU64::new(0));
    let anon_watch = tokio::spawn({
        let stop = stop.clone();
        let anon_peak = anon_peak.clone();
        let source = cfg.memory_source();
        async move {
            while !stop.load(Ordering::Relaxed) {
                if let Ok(source) = source.as_ref()
                    && let Ok(memory) = source.read()
                    && let Some(anon) = memory.anon_bytes
                {
                    anon_peak.fetch_max(anon, Ordering::Relaxed);
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
    });

    let mut report = json!({
        "phase": if cfg.phase == Phase::Seed { "seed" } else { "matrix" },
        "target": cfg.target.name(),
        "run": {
            "build_revision": config::build_revision(),
            "machine_profile": config::machine_profile(),
            "seed": cfg.seed,
        },
        "verify": {
            "tenant": corpus.tenant_ids[0].as_str(),
            "anchor_ns": cfg.verify.anchor_ns,
            "span_ns": cfg.verify_span_ns(),
            "rows": corpus.entry_count(),
            "streams": corpus.streams.len(),
            "line_bytes": corpus.line_bytes(),
        },
    });

    let ok;
    match cfg.phase {
        Phase::Seed => {
            let outcome = matrix::run_seed(&cfg, &corpus).await;
            ok = outcome.errors == 0 && outcome.rows == corpus.entry_count() as u64;
            report["seed"] = json!({
                "pushes": outcome.pushes,
                "rows": outcome.rows,
                "line_bytes": outcome.line_bytes,
                "wire_bytes": outcome.wire_bytes,
                "retries": outcome.retries,
                "errors": outcome.errors,
                "statuses": outcome
                    .statuses
                    .iter()
                    .map(|(status, count)| (status.to_string(), *count))
                    .collect::<BTreeMap<_, _>>(),
                "first_error": outcome.first_error,
                "elapsed_seconds": outcome.elapsed_seconds,
                "complete": ok,
            });
        }
        Phase::Matrix => {
            let outcome = matrix::run_matrix(&cfg, &corpus).await;
            ok = outcome["shapes"]
                .as_object()
                .is_some_and(|shapes| shapes.values().all(|shape| shape["errors"] == json!(0)));
            report["matrix"] = outcome;
        }
        Phase::Load | Phase::MetricSeed | Phase::MetricMatrix | Phase::MetricLoad => {
            unreachable!("run_verify is only reached for the seed and matrix phases")
        }
    }

    stop.store(true, Ordering::Relaxed);
    let _ = anon_watch.await;
    let memory_after = memory_source
        .as_ref()
        .ok()
        .and_then(|source| source.read().ok());
    report["memory"] = json!({
        "source": memory_source.as_ref().map(|source| source.describe()).ok(),
        "error": memory_source.as_ref().err(),
        "peak_bytes_before": memory_before.as_ref().map(|memory| memory.vm_hwm_bytes),
        "peak_bytes": memory_after.as_ref().map(|memory| memory.vm_hwm_bytes),
        "current_bytes": memory_after.as_ref().map(|memory| memory.vm_rss_bytes),
        "anon_peak_bytes": match anon_peak.load(Ordering::Relaxed) {
            0 => Value::Null,
            peak => json!(peak),
        },
        "anon_bytes_end": memory_after.as_ref().and_then(|memory| memory.anon_bytes),
        "file_bytes_end": memory_after.as_ref().and_then(|memory| memory.file_bytes),
    });
    report["config"] = serde_json::to_value(&cfg).expect("config serialization");
    report["verdict"] = json!(if ok { "PASS" } else { "FAIL" });

    let rendered = serde_json::to_string_pretty(&report).expect("report serialization");
    if let Some(path) = cfg.result_path.as_ref()
        && let Err(error) = std::fs::write(path, format!("{rendered}\n"))
    {
        eprintln!("warning: failed to write result file {path}: {error}");
    }
    println!("{rendered}");
    if !ok {
        std::process::exit(2);
    }
}

async fn run_load(cfg: Config) {
    let revision = config::build_revision();
    let machine_profile = config::machine_profile();

    eprintln!("generating corpus: {} rows", cfg.corpus_rows);
    let corpus = Arc::new(signy::corpus::generate(&signy::corpus::CorpusSpec {
        seed: cfg.seed,
        tenants: cfg.tenants,
        streams: cfg.streams,
        labels_per_stream: cfg.labels_per_stream,
        rows: cfg.corpus_rows,
        tenant_prefix: "load-tenant".to_string(),
        plain_weight: cfg.plain_weight,
        json_weight: cfg.json_weight,
        logfmt_weight: cfg.logfmt_weight,
        metadata_pairs: cfg.metadata_pairs,
        ..Default::default()
    }));
    let compression = measure_compression(&corpus);

    if let Err(error) = wait_for_ready(&cfg).await {
        eprintln!("server at {} is not ready: {error}", cfg.http_address);
        std::process::exit(1);
    }
    if let Err(error) = wait_for_collector(&cfg).await {
        eprintln!(
            "collector at {} is not answering: {error}",
            cfg.push_address()
        );
        std::process::exit(1);
    }
    let tenants: Vec<&str> = corpus.tenant_ids.iter().map(|id| id.as_str()).collect();
    if let Err(error) = onboard_tenants(&cfg, &tenants).await {
        eprintln!("{error}");
        std::process::exit(1);
    }

    let mut probe_client = Client::new(&cfg.http_address, cfg.request_timeout());
    let start_metrics = scrape(&mut probe_client).await.unwrap_or_default();

    let run_start = Instant::now();
    let warmup_end = run_start + Duration::from_secs(cfg.warmup_seconds);
    let deadline = run_start + Duration::from_secs(cfg.duration_seconds);
    let stop = Arc::new(AtomicBool::new(false));
    let events_accepted = Arc::new(AtomicU64::new(0));

    let (push_tx, push_rx) = mpsc::channel::<PushJob>(cfg.ingest_connections * 4);
    let push_rx = Arc::new(Mutex::new(push_rx));
    let (query_tx, query_rx) = mpsc::channel::<QueryJob>(cfg.query_connections * 4);
    let query_rx = Arc::new(Mutex::new(query_rx));

    let push_pacer = tokio::spawn(push_pacer(
        cfg.clone(),
        corpus.clone(),
        push_tx,
        stop.clone(),
        deadline,
    ));
    let query_pacer = tokio::spawn(query_pacer(
        cfg.clone(),
        corpus.clone(),
        query_tx,
        stop.clone(),
        deadline,
    ));
    let push_workers: Vec<_> = (0..cfg.ingest_connections)
        .map(|_| {
            tokio::spawn(push_worker(
                cfg.clone(),
                push_rx.clone(),
                warmup_end,
                events_accepted.clone(),
            ))
        })
        .collect();
    let query_workers: Vec<_> = (0..cfg.query_connections)
        .map(|_| tokio::spawn(query_worker(cfg.clone(), query_rx.clone(), warmup_end)))
        .collect();
    let sampler = tokio::spawn(sampler(cfg.clone(), stop.clone(), run_start));
    let otlp = tokio::spawn(otlp_workload(
        cfg.clone(),
        corpus.tenant_ids[0].as_str().to_string(),
        stop.clone(),
        deadline,
    ));
    // The metric leg rides the *log* corpus's first tenant rather than a
    // tenant of its own. That is the production shape — one customer sends
    // three signals — and it is also the only way this run can see the thing
    // the metrics work left open: the memtable byte budget is shared across
    // signals with no per-signal floor, so metric pressure and log pressure
    // meet in one gate or they never meet at all.
    let metric_ingest = tokio::spawn(metric_leg::metric_ingest_leg(
        cfg.clone(),
        corpus.tenant_ids[0].as_str().to_string(),
        stop.clone(),
        deadline,
    ));
    let metric_query = tokio::spawn(metric_leg::metric_query_leg(
        cfg.clone(),
        corpus.tenant_ids[0].as_str().to_string(),
        stop.clone(),
        deadline,
        warmup_end,
    ));

    // The stop condition is checked here rather than in each pacer so that
    // "enough work happened" and "long enough happened" are one decision.
    loop {
        if Instant::now() >= deadline {
            break;
        }
        if cfg.target_events > 0 && events_accepted.load(Ordering::Relaxed) >= cfg.target_events {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    stop.store(true, Ordering::Relaxed);

    let _ = push_pacer.await;
    let _ = query_pacer.await;
    let mut push = PushOutcome::default();
    for worker in push_workers {
        if let Ok(outcome) = worker.await {
            push.merge(outcome);
        }
    }
    let mut query = QueryOutcome::default();
    for worker in query_workers {
        if let Ok(outcome) = worker.await {
            query.merge(outcome);
        }
    }
    let mut samples = sampler.await.unwrap_or_default();
    let otlp = otlp.await.unwrap_or_default();
    let metric_ingest = metric_ingest.await.unwrap_or_default();
    let metric_query = metric_query.await.unwrap_or_default();
    // Read once more after the workload stopped: the sampler exits before the
    // last in-flight requests land, and both `VmHWM` and cgroup `memory.peak`
    // are high-water marks, so the final read is the only one that covers the
    // whole run.
    match cfg.memory_source() {
        Ok(source) => match source.read() {
            Ok(memory) => {
                samples.vm_hwm_bytes =
                    Some(samples.vm_hwm_bytes.unwrap_or(0).max(memory.vm_hwm_bytes));
                if let Some(anon) = memory.anon_bytes {
                    samples.anon_peak_bytes = Some(samples.anon_peak_bytes.unwrap_or(0).max(anon));
                }
                samples.file_bytes_end = memory.file_bytes.or(samples.file_bytes_end);
            }
            Err(error) => {
                samples.rss_error.get_or_insert(error);
            }
        },
        Err(error) => {
            samples.rss_error.get_or_insert(error);
        }
    }

    let elapsed_seconds = run_start.elapsed().as_secs_f64().max(f64::MIN_POSITIVE);
    // Through a collecty, a `200` means the export is on the collector's disk,
    // not that signy holds it. Everything still in the queue arrives after the
    // workloads stop, so the end scrape has to wait for it — otherwise the
    // accounting below reports a shortfall that is only the harness reading
    // too early.
    let collector_drain = drain_backlog(&cfg, &mut probe_client).await;
    let end_metrics = scrape(&mut probe_client).await.unwrap_or_default();

    let report = build_report(ReportInputs {
        cfg: &cfg,
        revision,
        machine_profile,
        compression,
        corpus: &corpus,
        elapsed_seconds,
        push,
        query,
        samples,
        otlp,
        metric_ingest,
        metric_query,
        start_metrics,
        end_metrics,
        collector_drain,
        ended_on: if cfg.target_events > 0
            && events_accepted.load(Ordering::Relaxed) >= cfg.target_events
        {
            "event_target"
        } else {
            "duration_cap"
        },
    });

    let rendered = serde_json::to_string_pretty(&report).expect("report serialization");
    if let Some(path) = cfg.result_path.as_ref()
        && let Err(error) = std::fs::write(path, format!("{rendered}\n"))
    {
        eprintln!("warning: failed to write result file {path}: {error}");
    }
    println!("{rendered}");
    // Two codes, because a caller has to be able to ignore one without ignoring
    // the other. Exit 2 is "the verdict is not PASS", which a run too short to
    // fill its percentiles reaches on the numeric gates alone; a caller that
    // knows its run is short is entitled to wave that through. A run that did
    // not get its load into the system under test is not a short run, and
    // nothing entitles a caller to wave it through, so it leaves by a door of
    // its own.
    if report["load_delivered"] == json!(false) {
        eprintln!(
            "the bed did not deliver its load; nothing under the verdict measures anything. \
             behavioral.delivered in the result file says which half failed -- nothing was \
             accepted, or what was accepted was dropped on arrival"
        );
        std::process::exit(3);
    }
    if report["verdict"] != json!("PASS") {
        std::process::exit(2);
    }
}

async fn wait_for_ready(cfg: &Config) -> Result<(), String> {
    let mut client = Client::new(&cfg.http_address, Duration::from_secs(5));
    let mut last = "no attempt made".to_string();
    for _ in 0..60 {
        match client
            .request(&Request {
                method: "GET",
                path: cfg.target.ready_path(),
                body: &[],
                content_type: "",
                tenant: None,
                headers: &[],
            })
            .await
        {
            Ok(response) if response.status == 200 => return Ok(()),
            Ok(response) => last = format!("/ready answered {}", response.status),
            Err(error) => last = error,
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Err(last)
}

/// Wait for the collector in front, when there is one.
///
/// collecty serves three POST paths and nothing else — no readiness route, and
/// nothing to ask about its queue from outside — so the check is that it
/// answers at all. `GET /v1/logs` is refused with `405`, and a refusal from
/// the process is proof the process is there. Silence is not: a run that
/// started against a collector still binding its socket would count every
/// early push as a connection error.
async fn wait_for_collector(cfg: &Config) -> Result<(), String> {
    let Some(address) = cfg.push_address.as_deref() else {
        return Ok(());
    };
    let mut client = Client::new(address, Duration::from_secs(5));
    let mut last = "no attempt made".to_string();
    for _ in 0..60 {
        match client
            .request(&Request {
                method: "GET",
                path: "/v1/logs",
                body: &[],
                content_type: "",
                tenant: None,
                headers: &[],
            })
            .await
        {
            Ok(_) => return Ok(()),
            Err(error) => last = error,
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Err(last)
}

/// Onboard the tenants this run pushes under.
///
/// signy's tenant registry *is* the set of pushed retention policies: a
/// tenant nobody has pushed a policy for is not served, and a push naming it
/// is dropped on arrival and answered 200 anyway -- the tenant rides in a
/// resource attribute now, so there is no transport left to refuse it. So the
/// harness onboards its own tenants first, the way a control plane would. The
/// comparison targets have no such API and no such gate, which is why this is
/// signy-only.
///
/// A failure here ends the run. A harness offering a rate that nothing
/// accepts still fills in every field of the result, and an idle server's
/// numbers are not distinguishable from a fast one's after the fact.
async fn onboard_tenants(cfg: &Config, tenants: &[&str]) -> Result<(), String> {
    if cfg.target != Target::Signy {
        return Ok(());
    }
    let mut client = Client::new(&cfg.http_address, cfg.request_timeout());
    let body = format!(
        "{{\"revision\": 1, \"retention\": \"{}\"}}",
        cfg.tenant_retention
    );
    for tenant in tenants {
        let path = format!("/signy/api/v1/admin/tenants/{tenant}/retention");
        let response = client
            .request(&Request {
                method: "PUT",
                path: &path,
                body: body.as_bytes(),
                content_type: "application/json",
                tenant: None,
                headers: &[],
            })
            .await
            .map_err(|error| format!("onboarding {tenant}: {error}"))?;
        if response.status != 200 {
            return Err(format!(
                "onboarding {tenant} answered {}: {}",
                response.status,
                String::from_utf8_lossy(&response.body).trim()
            ));
        }
    }
    Ok(())
}

/// Two attempts, because the second is what survives an idle keep-alive.
/// This client sits unused for the whole run between the start and end
/// scrapes, VictoriaLogs' server closes idle connections, and the first
/// request on a dead socket fails before the client notices — measured as a
/// behavioral gate reading zero rows from an engine that ingested millions.
/// What waiting for the collector's backlog cost, and whether it was enough.
#[derive(Default)]
struct CollectorDrain {
    /// False when there was nothing to wait for: a run pushing straight at the
    /// engine has no queue in front of it.
    waited: bool,
    seconds: f64,
    /// True when the arrival counter went quiet before the deadline. False is
    /// the run saying its end-of-run accounting is short by whatever was still
    /// in flight, rather than the accounting silently reporting a loss.
    settled: bool,
    requests_at_end: u64,
}

/// Wait for what a collecty still holds to reach signy.
///
/// A `200` from a collector means the export is on its disk. The queue drains
/// on its own schedule after that, so a run that scraped the moment its
/// workloads stopped would count as missing every record still queued.
///
/// The signal is signy's own request counter: one collect request is one
/// segment, so a counter that has stopped advancing means nothing is arriving
/// any more. Three quiet polls rather than one, because a sender that has just
/// been refused is backing off and a single quiet poll cannot tell that from an
/// empty queue.
async fn drain_backlog(cfg: &Config, client: &mut Client) -> CollectorDrain {
    let mut drain = CollectorDrain::default();
    if cfg.push_address.is_none() || cfg.drain_seconds == 0 {
        return drain;
    }
    drain.waited = true;
    const QUIET_POLLS: u32 = 3;
    let started = Instant::now();
    let deadline = started + Duration::from_secs(cfg.drain_seconds);
    let mut previous = u64::MAX;
    let mut quiet = 0;
    while Instant::now() < deadline {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let Some(metrics) = scrape(client).await else {
            continue;
        };
        let arrived = probe::gauge(&metrics, "signy_ingest_requests_total");
        drain.requests_at_end = arrived;
        if arrived == previous {
            quiet += 1;
            if quiet >= QUIET_POLLS {
                drain.settled = true;
                break;
            }
        } else {
            quiet = 0;
            previous = arrived;
        }
    }
    drain.seconds = started.elapsed().as_secs_f64();
    drain
}

async fn scrape(client: &mut Client) -> Option<probe::Metrics> {
    for _ in 0..2 {
        let request = Request {
            method: "GET",
            path: "/metrics",
            body: &[],
            content_type: "",
            tenant: None,
            headers: &[],
        };
        if let Ok(response) = client.request(&request).await
            && response.status == 200
        {
            return Some(probe::parse_metrics(&response.body));
        }
    }
    None
}

/// Issues on the nominal schedule regardless of how far behind the workers
/// are, so a job's `intended` time carries the delay the offered rate has
/// already accrued. That accrual is what coordinated omission drops.
async fn push_pacer(
    cfg: Config,
    corpus: Arc<signy::corpus::Corpus>,
    tx: mpsc::Sender<PushJob>,
    stop: Arc<AtomicBool>,
    deadline: Instant,
) {
    let mut generator = PushGenerator::new(
        corpus,
        cfg.seed,
        cfg.entries_per_push,
        cfg.streams_per_push,
        ArrivalOrder {
            spread_ms: cfg.entry_spread_ms,
            late_fraction: cfg.late_fraction,
            late_max_ms: cfg.late_max_ms,
        },
        cfg.push_wire(),
    );
    let interval = cfg.push_interval();
    let mut intended = Instant::now();
    while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
        // Built before the sleep so the encode cost lands in the idle gap
        // rather than between the intended send and the actual one.
        let body = generator.next_body(unix_nanos() as i64);
        let intended_at = match interval {
            Some(interval) => {
                let at = intended;
                tokio::time::sleep_until(at).await;
                // Advanced from the schedule, never from `now`: a pacer that
                // resets to the clock after a slow send is a pacer that
                // forgives the server the time it stole, which is coordinated
                // omission written into the pacing itself.
                intended = at + interval;
                at
            }
            // Unpaced: there is no schedule to be behind, so the intended time
            // is the moment the job was handed over and response time equals
            // service time. `run.pacing` says so.
            None => Instant::now(),
        };
        if tx
            .send(PushJob {
                intended: intended_at,
                body,
            })
            .await
            .is_err()
        {
            return;
        }
    }
}

async fn push_worker(
    cfg: Config,
    rx: Arc<Mutex<mpsc::Receiver<PushJob>>>,
    warmup_end: Instant,
    events_accepted: Arc<AtomicU64>,
) -> PushOutcome {
    let wire = cfg.push_wire();
    let push_path = wire.path(Signal::Logs).expect("signy takes logs");
    let mut client = Client::new(cfg.push_address(), cfg.request_timeout());
    let mut outcome = PushOutcome::default();
    loop {
        let job = {
            let mut guard = rx.lock().await;
            guard.recv().await
        };
        let Some(job) = job else { break };
        outcome.events_offered += job.body.entries as u64;
        outcome.out_of_order_entries += job.body.out_of_order_entries as u64;
        outcome.max_lateness_ms = outcome.max_lateness_ms.max(job.body.max_lateness_ms);
        outcome.streams_sent += job.body.streams as u64;

        let sent = Instant::now();
        let result = client
            .request(&Request {
                method: "POST",
                path: push_path,
                body: &job.body.bytes,
                content_type: PUSH_CONTENT_TYPE,
                headers: wire.headers(Signal::Logs),
                tenant: job
                    .body
                    .tenant_header
                    .as_ref()
                    .map(|(name, value)| (*name, value.as_str())),
            })
            .await;
        let done = Instant::now();
        let queueing_ms = duration_ms(sent.saturating_duration_since(job.intended));
        let service_ms = duration_ms(done.saturating_duration_since(sent));
        let steady = job.intended >= warmup_end;

        match result {
            Ok(response) => {
                *outcome.statuses.entry(response.status).or_default() += 1;
                match response.status {
                    // Loki's OTLP endpoint answers 204; signy and
                    // VictoriaLogs answer 200 with a response body, as the
                    // OTLP/HTTP specification prescribes.
                    200 | 204 => {
                        if steady {
                            outcome.steady.record(queueing_ms, service_ms);
                        } else {
                            outcome.warmup.record(queueing_ms, service_ms);
                        }
                        outcome.accepted += 1;
                        outcome.events_accepted += job.body.entries as u64;
                        events_accepted.fetch_add(job.body.entries as u64, Ordering::Relaxed);
                        outcome.wire_bytes += job.body.bytes.len() as u64;
                        outcome.line_bytes += job.body.line_bytes as u64;
                        outcome.encoded_bytes += job.body.encoded_bytes as u64;
                    }
                    // Backpressure, not an error. Kept out of the error rate
                    // and out of the accepted latencies, because a refusal is
                    // not the latency of doing the work.
                    429 => {
                        outcome.throttled += 1;
                        if steady {
                            outcome.throttled_latency.record(queueing_ms, service_ms);
                        }
                    }
                    status => {
                        outcome.errors += 1;
                        outcome.first_error.get_or_insert_with(|| {
                            format!(
                                "{status}: {}",
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
            }
        }
    }
    outcome.connects = client.connects;
    outcome
}

async fn query_pacer(
    cfg: Config,
    corpus: Arc<signy::corpus::Corpus>,
    tx: mpsc::Sender<QueryJob>,
    stop: Arc<AtomicBool>,
    deadline: Instant,
) {
    let Some(interval) = cfg.query_interval() else {
        return;
    };
    let mut generator = QueryGenerator::new(
        corpus,
        cfg.seed,
        cfg.target,
        cfg.query_weights,
        cfg.query_window_seconds,
        cfg.restore_lookback_seconds,
        cfg.query_limit,
        cfg.heavy_window_seconds,
        cfg.heavy_limit,
    );
    let mut intended = Instant::now();
    while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
        let plan = generator.next_plan(unix_seconds());
        tokio::time::sleep_until(intended).await;
        let job = QueryJob { intended, plan };
        intended += interval;
        if tx.send(job).await.is_err() {
            return;
        }
    }
}

async fn query_worker(
    cfg: Config,
    rx: Arc<Mutex<mpsc::Receiver<QueryJob>>>,
    warmup_end: Instant,
) -> QueryOutcome {
    let mut client = Client::new(&cfg.http_address, cfg.request_timeout());
    let mut outcome = QueryOutcome::default();
    loop {
        let job = {
            let mut guard = rx.lock().await;
            guard.recv().await
        };
        let Some(job) = job else { break };
        let shape = job.plan.shape;
        *outcome.shape_counts.entry(shape.name()).or_default() += 1;

        let sent = Instant::now();
        let result = client
            .request(&Request {
                method: "GET",
                path: &job.plan.path,
                body: &[],
                content_type: "",
                tenant: Some((job.plan.tenant.0, job.plan.tenant.1.as_str())),
                headers: &[],
            })
            .await;
        let done = Instant::now();
        let queueing_ms = duration_ms(sent.saturating_duration_since(job.intended));
        let service_ms = duration_ms(done.saturating_duration_since(sent));
        let steady = job.intended >= warmup_end;

        match result {
            Ok(response) => {
                *outcome.statuses.entry(response.status).or_default() += 1;
                match response.status {
                    200 => {
                        outcome.answered += 1;
                        let rows = result_rows(cfg.target, &response.body);
                        outcome.rows_returned += rows;
                        if shape == workload::QueryShape::RestoreProbe {
                            outcome.restore_probes += 1;
                            outcome.restore_rows += rows;
                            outcome.restore_probes_with_rows += u64::from(rows > 0);
                        }
                        if steady {
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
                                "{status} on {}: {}",
                                job.plan.expression,
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
            }
        }
    }
    outcome.connects = client.connects;
    outcome
}

/// Reads the server rather than driving it: `/proc` for memory, `/metrics` for
/// everything the engine publishes about itself.
async fn sampler(cfg: Config, stop: Arc<AtomicBool>, run_start: Instant) -> SampleOutcome {
    let mut outcome = SampleOutcome::default();
    let mut client = Client::new(&cfg.http_address, cfg.request_timeout());
    let interval = Duration::from_millis(cfg.sample_interval_ms);
    let memory_source = cfg.memory_source();
    while !stop.load(Ordering::Relaxed) {
        let elapsed = run_start.elapsed().as_secs_f64();
        match &memory_source {
            Ok(source) => match source.read() {
                Ok(memory) => {
                    outcome.rss.push(elapsed, memory.vm_rss_bytes);
                    outcome.vm_hwm_bytes =
                        Some(outcome.vm_hwm_bytes.unwrap_or(0).max(memory.vm_hwm_bytes));
                    if let Some(anon) = memory.anon_bytes {
                        outcome.anon.push(elapsed, anon);
                        outcome.anon_peak_bytes =
                            Some(outcome.anon_peak_bytes.unwrap_or(0).max(anon));
                    }
                    outcome.file_bytes_end = memory.file_bytes;
                }
                Err(error) => {
                    outcome.rss_error.get_or_insert(error);
                }
            },
            Err(error) => {
                outcome.rss_error.get_or_insert_with(|| error.clone());
            }
        }
        match scrape(&mut client).await {
            Some(metrics) => match cfg.target {
                Target::Signy => {
                    outcome.dropped_resources.observe(probe::breakdown(
                        &metrics,
                        "signy_ingest_dropped_resources_total",
                        "reason",
                    ));
                    outcome
                        .wal_backlog
                        .push(elapsed, probe::gauge(&metrics, "signy_wal_backlog_bytes"));
                    outcome
                        .memtable_bytes
                        .push(elapsed, probe::gauge(&metrics, "signy_memtable_bytes"));
                    outcome
                        .part_count
                        .push(elapsed, probe::gauge(&metrics, "signy_part_count"));
                    if cfg.capacity_probe {
                        // Structural gauges are enabled by the server only in
                        // probe mode.  Keep their complete sampled shape so a
                        // trial can inspect the path to OOM, not just its
                        // terminal population and one peak RSS number.
                        outcome
                            .series_states_len
                            .push(elapsed, probe::gauge(&metrics, "signy_series_states_len"));
                        outcome.series_states_capacity.push(
                            elapsed,
                            probe::gauge(&metrics, "signy_series_states_capacity"),
                        );
                        outcome
                            .series_buffers_len
                            .push(elapsed, probe::gauge(&metrics, "signy_series_buffers_len"));
                        outcome.series_buffers_capacity.push(
                            elapsed,
                            probe::gauge(&metrics, "signy_series_buffers_capacity"),
                        );
                        outcome.series_buffers_empty.push(
                            elapsed,
                            probe::gauge(&metrics, "signy_series_buffers_empty"),
                        );
                        outcome.series_buffers_inline.push(
                            elapsed,
                            probe::gauge(&metrics, "signy_series_buffers_inline"),
                        );
                        outcome.series_buffers_stream.push(
                            elapsed,
                            probe::gauge(&metrics, "signy_series_buffers_stream"),
                        );
                        outcome.series_flushing_series.push(
                            elapsed,
                            probe::gauge(&metrics, "signy_series_flushing_series"),
                        );
                        outcome.series_flushing_tenants.push(
                            elapsed,
                            probe::gauge(&metrics, "signy_series_flushing_tenants"),
                        );
                    }
                    if let Some(healthy) = metrics.get("signy_remote_healthy") {
                        outcome.health_samples += 1;
                        outcome.health_healthy += u64::from(*healthy >= 1.0);
                    }
                }
                // Loki's analogues, so the two runs report the same shape of
                // series: what is buffered in the ingester and how many
                // chunks it holds are its memtable and its part count.
                Target::Loki => {
                    outcome.memtable_bytes.push(
                        elapsed,
                        probe::sum_by_prefix(&metrics, "loki_ingester_memory_streams") as u64,
                    );
                    outcome.part_count.push(
                        elapsed,
                        probe::sum_by_prefix(&metrics, "loki_ingester_memory_chunks") as u64,
                    );
                }
                // VictoriaLogs' analogues: what is not yet on disk, and how
                // many parts hold what is.
                Target::VictoriaLogs => {
                    outcome.memtable_bytes.push(
                        elapsed,
                        probe::sum_by_prefix(&metrics, "vl_storage_inmemory_parts") as u64,
                    );
                    outcome.part_count.push(
                        elapsed,
                        probe::sum_by_prefix(&metrics, "vl_storage_file_parts") as u64,
                    );
                }
                Target::VictoriaMetrics => {
                    unreachable!("main refuses the log load phase for victoriametrics")
                }
                Target::Mimir => {
                    unreachable!("main refuses the log load phase for mimir")
                }
            },
            None => outcome.scrape_errors += 1,
        }
        tokio::time::sleep(interval).await;
    }
    outcome
}

/// Trace ingest, so the trace registry is not idle while the log path is
/// loaded. One request in flight by design — it is a garnish on the workload,
/// not part of the measured rate — but its latency is still taken from the
/// intended send, so a stall shows.
///
/// It goes to the collect route like everything else signy takes. It used to
/// have a listener of its own — `SIGNY_LOAD_OTLP_ADDR`, a gRPC `TraceService`
/// on `:4317` — and both went with the engine's other ingest paths.
async fn otlp_workload(
    cfg: Config,
    tenant: String,
    stop: Arc<AtomicBool>,
    deadline: Instant,
) -> OtlpOutcome {
    let mut outcome = OtlpOutcome::default();
    // Loki has no trace ingest, so a trace workload would be load one side
    // carries and the other does not.
    if cfg.target != Target::Signy {
        return outcome;
    }
    let Some(interval) = cfg.otlp_interval() else {
        return outcome;
    };
    let mut client = Client::new(cfg.push_address(), cfg.request_timeout());
    let mut reader = Client::new(&cfg.http_address, cfg.request_timeout());
    let mut rng = signy::corpus::Rng::new(cfg.seed ^ OTLP_SEED_SALT);
    let read_tenant = Target::Signy.read_tenant_header(&tenant);
    let lag = Duration::from_secs(cfg.trace_verify_lag_seconds);
    // The leg's own clock, so a missing trace can be put beside the fault log.
    let leg_start = Instant::now();
    outcome.verification_expected = deadline.saturating_duration_since(Instant::now()) > lag * 2;
    // Traces waiting out their lag before being read back. Bounded because a
    // read path that stopped answering must not turn into unbounded memory in
    // the harness measuring it.
    let mut pending: std::collections::VecDeque<(Instant, SentTrace)> =
        std::collections::VecDeque::new();
    let mut intended = Instant::now();
    while !stop.load(Ordering::Relaxed) && Instant::now() < deadline {
        tokio::time::sleep_until(intended).await;
        let sent = Instant::now();
        let result = send_otlp(&mut client, &cfg, &tenant, &mut rng).await;
        let done = Instant::now();
        outcome.latency.record(
            duration_ms(sent.saturating_duration_since(intended)),
            duration_ms(done.saturating_duration_since(sent)),
        );
        match result {
            Ok(trace) => {
                outcome.sent += 1;
                outcome.spans_sent += trace.spans;
                outcome.connected = true;
                if outcome.sent % cfg.trace_verify_sample == 0 && pending.len() < 256 {
                    pending.push_back((done, trace));
                }
            }
            Err(_) => outcome.errors += 1,
        }

        // One probe per turn at most: this leg is a garnish on the log
        // workload and reading back faster than that would make it a second
        // query workload competing with the one being measured.
        if let Some((at, _)) = pending.front()
            && at.elapsed() >= lag
        {
            let (_, mut trace) = pending.pop_front().expect("the front was just read");
            if verify_trace(
                &mut reader,
                &read_tenant,
                &trace,
                cfg.trace_verify_max_attempts,
                leg_start.elapsed().as_secs_f64(),
                &mut outcome,
            )
            .await
                == TraceProbe::AskAgain
            {
                outcome.retried += 1;
                trace.attempts += 1;
                pending.push_back((Instant::now(), trace));
            }
            // Every tenth read-back also asks the question a console opens
            // with: what traces are there at all. It shares the probe's pacing
            // so it cannot become a workload of its own.
            if outcome.verify_attempts % 10 == 1 {
                search_traces(&mut reader, &read_tenant, &mut outcome).await;
            }
        }
        intended += interval;
    }
    outcome
}

/// A trace this run exported, kept so it can be asked for again.
struct SentTrace {
    id: String,
    spans: u64,
    /// How many times it has been asked for and answered 404.
    ///
    /// One 404 is not yet a miss when a collector stands in front: the export
    /// is on the collector's disk the moment it is acked, and a backlog — a
    /// restart, a signy that was down for a few minutes — can put it behind
    /// the probe's lag. A second 404 a lag later is the miss.
    attempts: u32,
}

/// What the caller should do with a trace after one probe.
#[derive(PartialEq, Eq)]
enum TraceProbe {
    Done,
    /// A first 404 or an unanswered read: ask once more a lag later, because a
    /// backlog in front of the engine is not the engine losing a trace.
    AskAgain,
}

/// Read one trace back by id and check every span it was sent came back.
///
/// The timeline route answers 404 both for a trace that was never stored and
/// for one retention has taken, and the engine cannot tell those apart — which
/// is why the probe waits only its lag and never longer than a retention
/// period.
async fn verify_trace(
    reader: &mut Client,
    read_tenant: &(&'static str, String),
    trace: &SentTrace,
    max_attempts: u32,
    at_seconds: f64,
    outcome: &mut OtlpOutcome,
) -> TraceProbe {
    outcome.verify_attempts += 1;
    let path = format!("/signy/api/v1/traces/{}", trace.id);
    let response = reader
        .request(&Request {
            method: "GET",
            path: &path,
            body: &[],
            content_type: "",
            tenant: Some((read_tenant.0, read_tenant.1.as_str())),
            headers: &[],
        })
        .await;
    match response {
        Ok(response) if response.status == 200 => {
            let spans = response
                .body
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.is_empty())
                .count() as u64;
            if spans >= trace.spans {
                outcome.verified += 1;
            } else {
                outcome.short += 1;
                outcome.first_verify_error.get_or_insert_with(|| {
                    format!(
                        "trace {} came back with {spans} of {} spans",
                        trace.id, trace.spans
                    )
                });
            }
            TraceProbe::Done
        }
        // The tenant's read concurrency limit. The engine is up and answering;
        // it is saying this tenant already has as many queries in flight as it
        // is sold. Retried like a backlog, never counted as a lost trace.
        Ok(response) if response.status == 429 => {
            outcome.quota_rejected += 1;
            if trace.attempts < max_attempts {
                TraceProbe::AskAgain
            } else {
                TraceProbe::Done
            }
        }
        Ok(response) if response.status == 404 && trace.attempts < max_attempts => {
            TraceProbe::AskAgain
        }
        Ok(response) if response.status == 404 => {
            outcome.missing += 1;
            outcome.gave_up_after_budget += 1;
            if outcome.missing_at.len() < 512 {
                outcome.missing_at.push((at_seconds * 10.0).round() / 10.0);
            }
            outcome.first_verify_error.get_or_insert_with(|| {
                format!(
                    "trace {} was not found in {} attempts",
                    trace.id,
                    trace.attempts + 1
                )
            });
            TraceProbe::Done
        }
        Ok(response) => {
            outcome.unexpected_status += 1;
            outcome
                .first_verify_error
                .get_or_insert_with(|| format!("the timeline route answered {}", response.status));
            TraceProbe::Done
        }
        // The read address did not answer. A soak stops the engine on purpose,
        // so this says nothing about whether the trace is stored, and asking
        // again is the only honest thing to do with it.
        Err(_) if trace.attempts < max_attempts => {
            outcome.unreachable += 1;
            TraceProbe::AskAgain
        }
        Err(_) => {
            outcome.unreachable += 1;
            TraceProbe::Done
        }
    }
}

/// The search route over the window the leg has been writing into.
///
/// An empty answer is recorded rather than failed: the window can legitimately
/// hold nothing at the very start of a run. A search that never answers with
/// anything across a whole soak is what the counts are there to show.
async fn search_traces(
    reader: &mut Client,
    read_tenant: &(&'static str, String),
    outcome: &mut OtlpOutcome,
) {
    outcome.search_probes += 1;
    let response = reader
        .request(&Request {
            method: "GET",
            path: "/signy/api/v1/traces?start=-5m&limit=20",
            body: &[],
            content_type: "",
            tenant: Some((read_tenant.0, read_tenant.1.as_str())),
            headers: &[],
        })
        .await;
    match response {
        Ok(response) if response.status == 200 => {
            if response.body.iter().all(|byte| byte.is_ascii_whitespace()) {
                outcome.search_empty += 1;
            }
        }
        Ok(response) => {
            outcome.search_empty += 1;
            outcome
                .first_verify_error
                .get_or_insert_with(|| format!("the trace search answered {}", response.status));
        }
        Err(error) => {
            outcome.search_empty += 1;
            outcome.first_verify_error.get_or_insert(error);
        }
    }
}

/// The service the leg's spans claim, so the search probe can ask for a
/// service the way a console does.
const TRACE_SERVICE: &str = "load-trace";

/// One trace: a root and two children, rather than a lone span.
///
/// A single span would prove storage and retrieval and nothing else. Three
/// spans with a parent between them are what the timeline route is for, and
/// they are what makes a short answer — a trace that came back missing one of
/// its spans — a thing this run can see at all.
async fn send_otlp(
    client: &mut Client,
    cfg: &Config,
    tenant: &str,
    rng: &mut signy::corpus::Rng,
) -> Result<SentTrace, String> {
    let now = unix_nanos();
    let trace_id = rng.next_u64().to_be_bytes().repeat(2);
    let root_id = rng.next_u64().to_be_bytes().to_vec();
    let spans = vec![
        Span {
            trace_id: trace_id.clone(),
            span_id: root_id.clone(),
            name: "GET /load".to_string(),
            kind: 2,
            start_time_unix_nano: now,
            end_time_unix_nano: now.saturating_add(3_000_000),
            ..Default::default()
        },
        Span {
            trace_id: trace_id.clone(),
            span_id: rng.next_u64().to_be_bytes().to_vec(),
            parent_span_id: root_id.clone(),
            name: "store.write".to_string(),
            kind: 3,
            start_time_unix_nano: now.saturating_add(200_000),
            end_time_unix_nano: now.saturating_add(1_400_000),
            ..Default::default()
        },
        Span {
            trace_id: trace_id.clone(),
            span_id: rng.next_u64().to_be_bytes().to_vec(),
            parent_span_id: root_id,
            name: "encode".to_string(),
            kind: 1,
            start_time_unix_nano: now.saturating_add(1_500_000),
            end_time_unix_nano: now.saturating_add(2_600_000),
            ..Default::default()
        },
    ];
    let sent = SentTrace {
        id: trace_id.iter().map(|byte| format!("{byte:02x}")).collect(),
        spans: spans.len() as u64,
        attempts: 0,
    };
    let request = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            // The same tenancy gate the log path goes through: signy reads the
            // tenant off the resource, and without it every export is dropped
            // and the trace leg records latency for spans the server threw
            // away.
            resource: Some(opentelemetry_proto::tonic::resource::v1::Resource {
                attributes: vec![
                    crate::otlp::tenant_attribute(tenant),
                    crate::otlp::service_attribute(TRACE_SERVICE),
                ],
                ..Default::default()
            }),
            scope_spans: vec![ScopeSpans {
                spans,
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let wire = cfg.push_wire();
    let body = wire.wrap(request.encode_to_vec());
    let response = client
        .request(&Request {
            method: "POST",
            path: wire.path(Signal::Traces).expect("signy takes traces"),
            body: &body,
            content_type: PUSH_CONTENT_TYPE,
            tenant: None,
            headers: wire.headers(Signal::Traces),
        })
        .await?;
    match response.status {
        200 | 204 => Ok(sent),
        status => Err(format!("the ingest route answered {status}")),
    }
}

/// What this corpus actually compresses to, through the engine's own writer.
///
/// The number is in the report because the retired documents recorded a 31.5x
/// ratio for `"x".repeat(n)` beside a 5.9x ratio for realistic lines and drew
/// disk-footprint conclusions from the first. A load result has to state which
/// data it was measuring.
fn measure_compression(corpus: &signy::corpus::Corpus) -> Value {
    let dir = std::env::temp_dir().join(format!("signy-load-corpus-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    if let Err(error) = std::fs::create_dir_all(&dir) {
        return json!({ "measured": false, "error": error.to_string() });
    }
    let line_bytes = corpus.line_bytes();
    let result = signy::part::flush_rows(corpus.rows(), &dir, ROW_GROUP_SIZE);
    let value = match result {
        Ok(parts) => {
            let file_len = |path: std::path::PathBuf| {
                std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
            };
            let parquet_bytes: u64 = parts.iter().map(|part| file_len(part.data_path())).sum();
            let index_bytes: u64 = parts.iter().map(|part| file_len(part.index_path())).sum();
            json!({
                "measured": true,
                "rows": corpus.entry_count(),
                "line_bytes": line_bytes,
                "parquet_bytes": parquet_bytes,
                "index_bytes": index_bytes,
                "parquet_ratio": line_bytes as f64 / parquet_bytes.max(1) as f64,
            })
        }
        Err(error) => json!({ "measured": false, "error": error.to_string() }),
    };
    let _ = std::fs::remove_dir_all(&dir);
    value
}

struct ReportInputs<'a> {
    cfg: &'a Config,
    revision: String,
    machine_profile: String,
    compression: Value,
    corpus: &'a signy::corpus::Corpus,
    elapsed_seconds: f64,
    push: PushOutcome,
    query: QueryOutcome,
    samples: SampleOutcome,
    otlp: OtlpOutcome,
    metric_ingest: metric_leg::MetricIngestOutcome,
    metric_query: metric_leg::MetricQueryOutcome,
    start_metrics: probe::Metrics,
    end_metrics: probe::Metrics,
    collector_drain: CollectorDrain,
    ended_on: &'static str,
}

fn series_memory_report(samples: &SampleOutcome, terminal: &probe::Metrics) -> Value {
    json!({
        "states_len": {
            "summary": samples.series_states_len.summary(),
            "samples": samples.series_states_len.points(),
        },
        "states_capacity": {
            "summary": samples.series_states_capacity.summary(),
            "samples": samples.series_states_capacity.points(),
        },
        "buffers_len": {
            "summary": samples.series_buffers_len.summary(),
            "samples": samples.series_buffers_len.points(),
        },
        "buffers_capacity": {
            "summary": samples.series_buffers_capacity.summary(),
            "samples": samples.series_buffers_capacity.points(),
        },
        "buffers_empty": {
            "summary": samples.series_buffers_empty.summary(),
            "samples": samples.series_buffers_empty.points(),
        },
        "buffers_inline": {
            "summary": samples.series_buffers_inline.summary(),
            "samples": samples.series_buffers_inline.points(),
        },
        "buffers_stream": {
            "summary": samples.series_buffers_stream.summary(),
            "samples": samples.series_buffers_stream.points(),
        },
        "flushing_series": {
            "summary": samples.series_flushing_series.summary(),
            "samples": samples.series_flushing_series.points(),
        },
        "flushing_tenants": {
            "summary": samples.series_flushing_tenants.summary(),
            "samples": samples.series_flushing_tenants.points(),
        },
        "terminal": {
            "states_len": probe::gauge(terminal, "signy_series_states_len"),
            "states_capacity": probe::gauge(terminal, "signy_series_states_capacity"),
            "buffers_len": probe::gauge(terminal, "signy_series_buffers_len"),
            "buffers_capacity": probe::gauge(terminal, "signy_series_buffers_capacity"),
            "buffers_empty": probe::gauge(terminal, "signy_series_buffers_empty"),
            "buffers_inline": probe::gauge(terminal, "signy_series_buffers_inline"),
            "buffers_stream": probe::gauge(terminal, "signy_series_buffers_stream"),
            "flushing_series": probe::gauge(terminal, "signy_series_flushing_series"),
            "flushing_tenants": probe::gauge(terminal, "signy_series_flushing_tenants"),
            "interner_len": probe::gauge(terminal, "signy_series_label_interner_len"),
            "interner_capacity": probe::gauge(terminal, "signy_series_label_interner_capacity"),
        },
    })
}

fn build_report(inputs: ReportInputs<'_>) -> Value {
    let ReportInputs {
        cfg,
        revision,
        machine_profile,
        compression,
        corpus,
        elapsed_seconds,
        mut push,
        mut query,
        samples,
        mut otlp,
        mut metric_ingest,
        mut metric_query,
        start_metrics,
        end_metrics,
        collector_drain,
        ended_on,
    } = inputs;

    let delta = |name: &str| probe::counter_delta(&start_metrics, &end_metrics, name);
    let flush_success = delta("signy_flush_success_total");
    let flush_errors = delta("signy_flush_errors_total");
    let merge_success = delta("signy_merge_success_total");
    let merge_errors = delta("signy_merge_errors_total");
    let ingest_errors = delta("signy_ingest_errors_total");
    let dropped_resources = probe::sum_delta(
        &start_metrics,
        &end_metrics,
        "signy_ingest_dropped_resources_total",
    );
    // Records a batch carried that signy will never accept. Zero on every run:
    // unlike a dropped resource, which is a tenant this instance does not
    // serve, this is a record it could not read at all.
    let collect_dropped_records = delta("signy_collect_dropped_records_total");
    // Records a collecty sent again that signy already had. Not a failure —
    // it is the resume working — so it is reported rather than gated. A run
    // that restarted either side is expected to move it.
    let collect_skipped_records = delta("signy_collect_skipped_records_total");
    // A trace read back by id after it was written is the only end-to-end
    // check the trace path has ever had: every earlier run wrote spans and
    // asked nothing about them afterwards. A run too short for a probe to come
    // due proves nothing either way, and says so rather than passing.
    // A quota refusal is not a lost trace and an outage is not one either, so
    // neither is here. What is here is the engine answering wrongly: a trace it
    // acked and cannot produce, one it produces short, or a status nobody
    // planned for.
    let traces_pass = otlp.missing == 0
        && otlp.short == 0
        && otlp.unexpected_status == 0
        && (!otlp.verification_expected || otlp.verified > 0);
    // Every OTLP export this run got a 2xx for, across the three signals.
    //
    // The one number a collector's own `collecty_records_appended_total` can be
    // read against: both count exports, and neither counts entries. It sits at
    // the top of the report because reconciling the two is a shell's job at the
    // end of a soak, and digging it out of three nested status maps with grep
    // is not.
    let accepted = |statuses: &BTreeMap<u16, u64>| -> u64 {
        statuses
            .iter()
            .filter(|(status, _)| (200..300).contains(*status))
            .map(|(_, count)| *count)
            .sum()
    };
    let exports_accepted =
        accepted(&push.statuses) + accepted(&metric_ingest.tally.statuses) + otlp.sent;
    let signy_delivered = push.events_accepted > 0 && dropped_resources == 0;
    let retention_success = delta("signy_retention_success_total");
    let restore_success = delta("signy_remote_restore_success_total");
    let restore_errors = delta("signy_remote_restore_errors_total");

    // Throttled requests are on neither side of the error rate: a 429 is
    // neither work the server accepted nor work it failed at. It is reported
    // beside it instead, because a run that was refused most of what it
    // offered has not measured what it set out to.
    let attempted = push.accepted + push.errors + push.throttled;
    let error_rate = if push.accepted + push.errors == 0 {
        0.0
    } else {
        push.errors as f64 / (push.accepted + push.errors) as f64
    };
    let throttled_rate = if attempted == 0 {
        0.0
    } else {
        push.throttled as f64 / attempted as f64
    };

    let push_response_p95 = push.steady.response.quantile(0.95);
    let push_response_p99 = push.steady.response.quantile(0.99);
    let push_service_p99 = push.steady.service.quantile(0.99);
    let query_response_p95 = query.steady.response.quantile(0.95);

    let rss_peak = samples.vm_hwm_bytes;
    let rss_measured = rss_peak.is_some();

    let drain = wal_backlog_drains(
        &samples.wal_backlog,
        cfg.targets.wal_backlog_max_bytes,
        cfg.targets.min_backlog_samples,
    );
    // A wedged flush loop emits an error roughly every tick, so its errors
    // outnumber its successes; a healthy loop with injected transient faults
    // does not.
    let flush_progressing = flush_success > 0 && flush_errors <= flush_success && drain.drained;
    let remote_healthy_fraction = if samples.health_samples > 0 {
        samples.health_healthy as f64 / samples.health_samples as f64
    } else {
        0.0
    };
    let cache_healthy_end = probe::gauge(&end_metrics, "signy_cache_healthy") == 1;

    // The metric leg's own verdict, computed here so the gate below can read
    // it and the report can carry the evidence either way.
    let metric_leg_on = cfg.metric_leg_scrape_seconds > 0 || cfg.metric_query_eps > 0.0;
    let metric_ingest_delivered = metric_ingest.tally.datapoints_accepted > 0;
    // A shape that must return rows and returned none is the failure a status
    // gate cannot see: the write path kept accepting and the read path stopped
    // finding. It is reported per shape rather than as one flag so that which
    // shape went quiet is in the artifact.
    let metric_shapes_answered: Vec<String> = metric_leg::METRIC_QUERY_SHAPES
        .iter()
        .filter(|shape| shape.must_return_rows())
        .filter(|shape| {
            let issued = metric_query
                .shape_counts
                .get(shape.name())
                .copied()
                .unwrap_or(0);
            // Empties a shape gave while catching up from an unanswered
            // window are not here: see `shape_empty_recovering`.
            let empty = metric_query
                .shape_empty
                .get(shape.name())
                .copied()
                .unwrap_or(0);
            issued > 0 && empty > 0
        })
        .map(|shape| shape.name().to_string())
        .collect();
    // A read leg that judged nothing has not checked anything, and the run
    // has to say so rather than report the absence as a pass. It is only a
    // failure when the run outlasted the leg's own settling floor: below that
    // there is legitimately nothing to judge, and the report carries both
    // numbers either way.
    let metric_reads_on = cfg.metric_query_eps > 0.0;
    let metric_outlasted_settling =
        elapsed_seconds > metric_query.settling_seconds as f64 && metric_query.answered > 0;
    let metric_reads_judged =
        !metric_reads_on || !metric_outlasted_settling || metric_query.judged_total > 0;
    // Unanswered reads are not here: a soak takes the engine down on purpose,
    // and the availability those windows cost is reported beside this rather
    // than folded into it. What is here is the engine answering wrongly while
    // it was up.
    let metric_leg_pass = !metric_leg_on
        || (metric_ingest_delivered
            && metric_ingest.tally.errors == 0
            && metric_query.errors == 0
            && metric_reads_judged
            && metric_shapes_answered.is_empty());
    let metric_report = if !metric_leg_on {
        json!({ "enabled": false })
    } else {
        let mut per_shape = serde_json::Map::new();
        for shape in metric_leg::METRIC_QUERY_SHAPES {
            let name = shape.name();
            let issued = metric_query.shape_counts.get(name).copied().unwrap_or(0);
            if issued == 0 {
                continue;
            }
            let mut latency = metric_query.per_shape.remove(name).unwrap_or_default();
            per_shape.insert(
                name.to_string(),
                json!({
                    "issued": issued,
                    "judged": metric_query.shape_judged.get(name).copied().unwrap_or(0),
                    "series_returned": metric_query.shape_rows.get(name).copied().unwrap_or(0),
                    "empty_answers": metric_query.shape_empty.get(name).copied().unwrap_or(0),
                    "empty_while_recovering": metric_query
                        .shape_empty_recovering
                        .get(name)
                        .copied()
                        .unwrap_or(0),
                    "must_return_rows": shape.must_return_rows(),
                    "latency_ms": latency.summary(),
                }),
            );
        }
        json!({
            "enabled": true,
            "recovery_grace_seconds": metric_query.recovery_grace_seconds,
            "empty_answers_at": metric_query
                .empty_at
                .iter()
                .map(|(shape, at, catching_up)| json!({
                    "shape": shape,
                    "at_seconds": (at * 10.0).round() / 10.0,
                    "catching_up": catching_up,
                }))
                .collect::<Vec<_>>(),
            "tenant": corpus.tenant_ids.first().map(|id| id.as_str()),
            "scrape_interval_seconds": cfg.metric_leg_scrape_seconds,
            "churn_per_scrape": cfg.metric_leg_churn_per_scrape,
            "query_eps": cfg.metric_query_eps,
            "query_window_seconds": cfg.metric_query_window_seconds,
            "ingest": {
                "scrapes": metric_ingest.tally.scrapes,
                "scrapes_late": metric_ingest.scrapes_late,
                "datapoints_offered": metric_ingest.tally.datapoints_offered,
                "datapoints_accepted": metric_ingest.tally.datapoints_accepted,
                "datapoints_rejected": metric_ingest.tally.datapoints_rejected,
                "requests_refused": metric_ingest.tally.requests_refused,
                "acceptance": metric_ingest.tally.acceptance(),
                "series_offered": metric_ingest.series_offered,
                "errors": metric_ingest.tally.errors,
                "first_error": metric_ingest.tally.first_error.clone(),
                "statuses": metric_ingest.tally.statuses.iter()
                    .map(|(status, count)| (status.to_string(), *count))
                    .collect::<BTreeMap<_, _>>(),
                "latency_ms": metric_ingest.tally.latency.summary(),
            },
            "queries": {
                "answered": metric_query.answered,
                "judged": metric_query.judged_total,
                "settling_seconds": metric_query.settling_seconds,
                "note": if metric_query.judged_total == 0 && metric_reads_on {
                    "no answer was judged: the run did not outlast the settling floor, so the empty-answer check made no assertion"
                } else {
                    "empty answers are counted only past the settling floor, where a rate has two samples and a quantile has a bucket window"
                },
                "errors": metric_query.errors,
                "throttled": metric_query.throttled,
                "unavailable": metric_query.unavailable,
                "first_unavailable": metric_query.first_unavailable,
                "first_error": metric_query.first_error.clone(),
                "statuses": metric_query.statuses.iter()
                    .map(|(status, count)| (status.to_string(), *count))
                    .collect::<BTreeMap<_, _>>(),
                "latency_ms": metric_query.steady.summary(),
                "per_shape": Value::Object(per_shape),
            },
            // What the engine says about its own metric path over the same
            // window. The leg's counts are what was offered; these are what
            // was kept, and a soak is the run where the two stop agreeing.
            "server": {
                "active_series_end": probe::gauge(&end_metrics, "signy_active_series"),
                "series_memtable_bytes_end": probe::gauge(&end_metrics, "signy_series_memtable_bytes"),
                "metric_part_count_end": probe::gauge(&end_metrics, "signy_metric_part_count"),
                "series_created": delta("signy_series_created_total"),
                "series_retired_flushed": delta("signy_series_retired_flushed_total"),
                "series_evicted_idle": delta("signy_series_evicted_idle_total"),
                "series_rejected": delta("signy_series_rejected_total"),
                "datapoints_rejected": delta("signy_metric_datapoints_rejected_total"),
                "samples_rejected": delta("signy_metric_samples_rejected_total"),
                "cardinality_rejected": delta("signy_metric_cardinality_rejected_total"),
                "memory_rejected": delta("signy_metric_memory_rejected_total"),
            },
            "pass": metric_leg_pass,
            "shapes_that_answered_nothing": metric_shapes_answered,
        })
    };

    let targets = json!({
        "push_response_p95_ms": target_row(
            push_response_p95,
            cfg.targets.push_response_p95_ms,
            &format!(
                "from the intended send, over {} ingest connections; the service percentiles are \
    beside it in push_latency_ms. The connection count is part of this target's definition rather than \
    a property of the rig: measured on one unchanged server at one unchanged offered rate, service p95 \
    was 40.3 ms over 8 connections and 106.5 ms over 32 while response p95 went 266.4 to 166.8 \
    (todo.md, 2026-08-12). Neither number is the server alone; response is at least the one a client \
    experiences, and it moves the honest way when the server gets slower",
                cfg.ingest_connections
            ),
        ),
        "push_response_p99_ms": target_row(
            push_response_p99,
            cfg.targets.push_response_p99_ms,
            "",
        ),
        "query_response_p95_ms": target_row(
            query_response_p95,
            cfg.targets.query_response_p95_ms,
            "",
        ),
        "rss_peak_bytes": target_row(
            rss_peak.map(|bytes| bytes as f64),
            cfg.targets.rss_max_bytes as f64,
            "VmHWM of the server process, not a sampled maximum",
        ),
        "error_rate": target_row(Some(error_rate), cfg.targets.max_error_rate, "429 excluded"),
        "throttled_rate": target_row(
            Some(throttled_rate),
            cfg.targets.max_throttled_rate,
            "429 is backpressure working, but a run refused most of its offer measured a rate \
    nobody offered",
        ),
        "wal_backlog_peak_bytes": target_row(
            Some(samples.wal_backlog.peak() as f64),
            cfg.targets.wal_backlog_max_bytes as f64,
            "",
        ),
    });
    // A gate whose measurement is missing fails. `pass: null` is not a pass —
    // the correction in docs/LOAD_RESULTS.md §3 is exactly this rule, arrived
    // at after a peak RSS that had never been measured was written down as an
    // engine result.
    let numeric_pass = targets
        .as_object()
        .expect("targets is an object")
        .values()
        .all(|row| row["pass"] == json!(true));

    // Loki publishes none of the series the signy gates are written
    // against, and a gate over a series that does not exist reads zero and
    // passes. So the Loki run is gated on Loki's own evidence instead — chiefly
    // `loki_discarded_samples_total`, which is where a rate limit, a rejected
    // old sample or an unordered write would show up. Every deviation this bed
    // makes from Loki's defaults exists to keep that counter at zero, and a
    // run where it is not zero is a misconfiguration on our side, not a result.
    // `delivered` is the arrival gate on its own, split out of the pass so the
    // exit code can key on it. Every target has one and it means the same thing
    // in each: the bed put its load into the system under test. A run that did
    // not is not a slow run or a short one, it is not a run.
    let (behavioral, behavioral_pass, delivered) = match cfg.target {
        // The two gates Loki's and VictoriaLogs' arms already lead with, and
        // signy's did not: something arrived, and nothing was thrown away.
        //
        // Neither is spare. `events_accepted` is the harness's own count, taken
        // where a push was answered 200, and it is what a run refused outright
        // leaves at zero -- `signy_ingest_errors_total` counts what ingest
        // failed at rather than what admission turned away, so it stays clean
        // through a run in which nothing was stored. `dropped_resources` is the
        // other half, and the one no client can see: an export naming a tenant
        // this instance does not serve is dropped and still answered 200, so a
        // bed whose tenants were never onboarded reports every push accepted
        // and stores none of them. The counter's own help text says why it has
        // to be read here -- an ingest answers whether the body arrived and
        // nothing about who sent it.
        Target::Signy => (
            json!({
                "delivered": signy_delivered,
                "events_accepted": push.events_accepted,
                "dropped_resources": dropped_resources,
                // Summed over every generation of the engine, not read off
                // the last one: a soak restarts the process on purpose and the
                // counter starts again each time.
                "dropped_by_reason": samples.dropped_resources.total(),
                "dropped_generations_observed": samples.dropped_resources.restarts_seen + 1,
                "no_ingest_errors": ingest_errors == 0,
                "remote_healthy_fraction": remote_healthy_fraction,
                "cache_healthy_end": cache_healthy_end,
                "flush_progressing": flush_progressing,
                "wal_backlog_drains": drain.drained,
                "wal_backlog_verdict_decided": drain.decided,
                "wal_backlog_reason": drain.reason,
                "wal_backlog": samples.wal_backlog.summary(),
                "flush_success_delta": flush_success,
                "recovered_flush_errors": flush_errors,
                "recovered_merge_errors": merge_errors,
                "merge_success_delta": merge_success,
                "restore_observed": restore_success > 0,
                "restore_errors": restore_errors,
                "restore_probe_rows": query.restore_rows,
                "restore_probes_with_rows": query.restore_probes_with_rows,
                "retention_observed": retention_success > 0,
                // The collector in front, when there is one. `settled` is the
                // gate: a run whose backlog had not arrived by the deadline
                // cannot say what reached storage, so its accounting is not a
                // loss report and must not read as one.
                "ingest_path": if cfg.push_address.is_some() { "collecty" } else { "direct" },
                "collector_drain": {
                    "waited": collector_drain.waited,
                    "settled": collector_drain.settled,
                    "seconds": collector_drain.seconds,
                    "ingest_requests_at_end": collector_drain.requests_at_end,
                },
                "collect_dropped_records": collect_dropped_records,
                "collect_skipped_records": collect_skipped_records,
                "metric_leg": metric_report,
            }),
            signy_delivered
                && ingest_errors == 0
                && collect_dropped_records == 0
                && (!collector_drain.waited || collector_drain.settled)
                && remote_healthy_fraction >= 0.95
                && cache_healthy_end
                && flush_progressing
                && traces_pass
                && metric_leg_pass,
            signy_delivered,
        ),
        Target::Loki => {
            let lines_received = probe::sum_delta(
                &start_metrics,
                &end_metrics,
                "loki_distributor_lines_received_total",
            );
            let discarded =
                probe::sum_delta(&start_metrics, &end_metrics, "loki_discarded_samples_total");
            (
                json!({
                    "delivered": lines_received > 0 && discarded == 0,
                    "lines_received": lines_received,
                    "discarded_samples": discarded,
                    "discarded_by_reason": probe::breakdown(
                        &end_metrics,
                        "loki_discarded_samples_total",
                        "reason",
                    ),
                    "chunks_flushed": probe::sum_delta(
                        &start_metrics,
                        &end_metrics,
                        "loki_ingester_chunks_flushed_total",
                    ),
                    "memory_chunks_end": probe::sum_by_prefix(&end_metrics, "loki_ingester_memory_chunks"),
                    "memory_streams_end": probe::sum_by_prefix(&end_metrics, "loki_ingester_memory_streams"),
                    "note": "a non-zero discard count is this bed misconfiguring Loki, not a Loki \
                result; the reason label says which limit did it",
                }),
                lines_received > 0 && discarded == 0,
                lines_received > 0 && discarded == 0,
            )
        }
        // Same rule as Loki's: a rejection here is this bed misconfiguring the
        // system rather than a result about it, so the gate is that nothing was
        // dropped and something arrived.
        Target::VictoriaLogs => {
            let rows = probe::sum_delta(&start_metrics, &end_metrics, "vl_rows_ingested_total");
            let dropped = probe::sum_delta(&start_metrics, &end_metrics, "vl_rows_dropped_total");
            (
                json!({
                    "delivered": rows > 0 && dropped == 0,
                    "rows_ingested": rows,
                    "rows_dropped": dropped,
                    "dropped_by_reason": probe::breakdown(
                        &end_metrics,
                        "vl_rows_dropped_total",
                        "reason",
                    ),
                    "inmemory_parts_end": probe::sum_by_prefix(&end_metrics, "vl_storage_inmemory_parts"),
                    "file_parts_end": probe::sum_by_prefix(&end_metrics, "vl_storage_file_parts"),
                    "note": "a non-zero drop count is this bed misconfiguring VictoriaLogs, not a \
                VictoriaLogs result",
                }),
                rows > 0 && dropped == 0,
                rows > 0 && dropped == 0,
            )
        }
        Target::VictoriaMetrics => {
            unreachable!("main refuses the log load phase for victoriametrics")
        }
        Target::Mimir => unreachable!("main refuses the log load phase for mimir"),
    };

    let mut per_shape = serde_json::Map::new();
    for shape in QUERY_SHAPES {
        let name = shape.name();
        let mut latency = query.per_shape.remove(name).unwrap_or_default();
        per_shape.insert(
            name.to_string(),
            json!({
                "issued": query.shape_counts.get(name).copied().unwrap_or(0),
                "latency_ms": latency.summary(),
            }),
        );
    }

    let mut report = json!({
        "verdict": if behavioral_pass && numeric_pass {
            "PASS"
        } else if behavioral_pass {
            "PASS_BEHAVIORAL_ONLY"
        } else {
            "FAIL"
        },
        "numeric_pass": numeric_pass,
        "behavioral_pass": behavioral_pass,
        "load_delivered": delivered,
        "exports_accepted": exports_accepted,
        "targets": targets,
        "behavioral": behavioral,
        "run": {
            "target": cfg.target.name(),
            "phase": "load",
            "tier": cfg.tier,
            "build_revision": revision,
            "machine_profile": machine_profile,
            "seed": cfg.seed,
            "elapsed_seconds": elapsed_seconds,
            "ended_on": ended_on,
            "pacing": if cfg.push_interval().is_some() { "open_loop" } else { "closed_loop" },
        },
        "ingest": {
            "offered_eps": cfg.target_eps,
            "attempted_eps": push.events_offered as f64 / elapsed_seconds,
            "achieved_eps": push.events_accepted as f64 / elapsed_seconds,
            "events_accepted": push.events_accepted,
            "events_offered": push.events_offered,
            "pushes_accepted": push.accepted,
            "pushes_throttled": push.throttled,
            "pushes_failed": push.errors,
            "throttled_rate": throttled_rate,
            "error_rate": error_rate,
            "statuses": push.statuses.iter().map(|(k, v)| (k.to_string(), *v)).collect::<BTreeMap<_, _>>(),
            "first_error": push.first_error,
            "tcp_connections_opened": push.connects,
            "wire_bytes": push.wire_bytes,
            "line_bytes": push.line_bytes,
            "streams_per_push_mean": if push.accepted + push.throttled + push.errors > 0 {
                json!(push.streams_sent as f64 / (push.accepted + push.throttled + push.errors) as f64)
            } else {
                Value::Null
            },
            "out_of_order_entries": push.out_of_order_entries,
            "max_lateness_ms": push.max_lateness_ms,
        },
        "push_latency_ms": push.steady.summary(),
        "push_warmup_latency_ms": push.warmup.summary(),
        "push_throttled_latency_ms": push.throttled_latency.summary(),
        "queries": {
            "answered": query.answered,
            "errors": query.errors,
            "throttled": query.throttled,
            "unavailable": query.unavailable,
            "first_unavailable": query.first_unavailable,
            "rows_returned": query.rows_returned,
            "restore_probes": query.restore_probes,
            "achieved_qps": query.answered as f64 / elapsed_seconds,
            "statuses": query.statuses.iter().map(|(k, v)| (k.to_string(), *v)).collect::<BTreeMap<_, _>>(),
            "first_error": query.first_error,
            "tcp_connections_opened": query.connects,
        },
        "query_latency_ms": query.steady.summary(),
        "query_by_shape": Value::Object(per_shape),
    });

    // Assembled separately: one `json!` literal holding all of this stops the
    // macro expanding at its recursion limit.
    report["memory"] = json!({
        "vm_hwm_bytes": rss_peak,
        "measured": rss_measured,
        "error": samples.rss_error,
        "vm_rss": samples.rss.summary(),
        "vm_rss_samples": samples.rss.points(),
        // A cgroup peak includes the page cache the cgroup's own writes
        // created, which is reclaimable and is not the process's footprint.
        // Both systems write a WAL and then large data files, so both carry
        // hundreds of megabytes of it. The anonymous series is the part that
        // cannot be reclaimed and is what an OOM kill is decided on.
        "anon_peak_bytes": samples.anon_peak_bytes,
        "anon": samples.anon.summary(),
        "file_bytes_end": samples.file_bytes_end,
    });
    report["gauges"] = json!({
        "memtable_bytes": samples.memtable_bytes.summary(),
        "part_count": samples.part_count.summary(),
        "wal_backlog_bytes": samples.wal_backlog.summary(),
        "scrape_errors": samples.scrape_errors,
        "terminal": {
            "memtable_bytes": probe::gauge(&end_metrics, "signy_memtable_bytes"),
            "memtable_entries": probe::gauge(&end_metrics, "signy_memtable_entries"),
            "part_count": probe::gauge(&end_metrics, "signy_part_count"),
            "part_bytes": probe::gauge(&end_metrics, "signy_part_bytes"),
            "trace_part_count": probe::gauge(&end_metrics, "signy_trace_part_count"),
            "merge_debt_parts": probe::gauge(&end_metrics, "signy_merge_debt_parts"),
            "part_tenant_segments": probe::gauge(&end_metrics, "signy_part_tenant_segments"),
            "part_sidecar_resident_bytes": probe::gauge(
                &end_metrics,
                "signy_part_sidecar_resident_bytes",
            ),
            "part_meta_bytes": probe::gauge(&end_metrics, "signy_part_meta_bytes"),
        },
    });
    if cfg.capacity_probe && !samples.series_states_len.samples.is_empty() {
        report["gauges"]["series_memory"] = series_memory_report(&samples, &end_metrics);
    }
    report["object_store_operations"] = json!({
        "puts": probe::object_store_op_delta(&start_metrics, &end_metrics, "put"),
        "gets": probe::object_store_op_delta(&start_metrics, &end_metrics, "get"),
        "deletes": probe::object_store_op_delta(&start_metrics, &end_metrics, "delete"),
        "lists": probe::object_store_op_delta(&start_metrics, &end_metrics, "list"),
        "copies": probe::object_store_op_delta(&start_metrics, &end_metrics, "copy"),
        "listed_objects": delta("signy_object_store_listed_objects_total"),
        // The byte axis, and the restores that generate most of it. A restore
        // downloads a whole part, and a part is shared by every tenant whose
        // rows landed in it — so `get_bytes / restores` against a tenant's
        // share of a part is the size of "add Parquet range reads", which was
        // an argument from reading the code until this ran.
        "ranged_gets": delta("signy_object_store_ranged_gets_total"),
        "get_bytes": delta("signy_object_store_bytes_total{direction=\"get\"}"),
        "put_bytes": delta("signy_object_store_bytes_total{direction=\"put\"}"),
        "get_bytes_by_kind": {
            "manifest": delta("signy_object_store_bytes_by_kind_total{direction=\"get\",kind=\"manifest\"}"),
            "part": delta("signy_object_store_bytes_by_kind_total{direction=\"get\",kind=\"part\"}"),
            "trace_part": delta("signy_object_store_bytes_by_kind_total{direction=\"get\",kind=\"trace_part\"}"),
            "other": delta("signy_object_store_bytes_by_kind_total{direction=\"get\",kind=\"other\"}"),
        },
        "put_bytes_by_kind": {
            "manifest": delta("signy_object_store_bytes_by_kind_total{direction=\"put\",kind=\"manifest\"}"),
            "part": delta("signy_object_store_bytes_by_kind_total{direction=\"put\",kind=\"part\"}"),
            "trace_part": delta("signy_object_store_bytes_by_kind_total{direction=\"put\",kind=\"trace_part\"}"),
            "other": delta("signy_object_store_bytes_by_kind_total{direction=\"put\",kind=\"other\"}"),
        },
        "restores": delta("signy_remote_restore_latency_ms_count"),
        // The two numbers that decide whether a selective download is worth
        // issuing. `selected_runs + part_scans` is the request count one would
        // cost where a whole restore costs one GET, and
        // `restored_body_scans / restores` is how many scans the whole copy
        // serves before eviction takes it — the amortisation a range read
        // cancels.
        "part_scans": delta("signy_query_part_scans_total"),
        "row_groups": {
            "present": delta("signy_query_row_groups_total{stage=\"present\"}"),
            "tenant": delta("signy_query_row_groups_total{stage=\"tenant\"}"),
            "selected": delta("signy_query_row_groups_total{stage=\"selected\"}"),
        },
        "selected_runs": delta("signy_query_selected_runs_total"),
        "restore_first_scan": {
            "parts": delta("signy_restore_first_scan_total{stage=\"parts\"}"),
            "present": delta("signy_restore_first_scan_total{stage=\"present\"}"),
            "selected": delta("signy_restore_first_scan_total{stage=\"selected\"}"),
            "runs": delta("signy_restore_first_scan_total{stage=\"runs\"}"),
        },
        "restored_body_scans": delta("signy_restored_body_scans_total"),
        "restored_bodies": delta("signy_restored_bodies_total{state=\"restored\"}"),
        "retired_bodies": delta("signy_restored_bodies_total{state=\"retired\"}"),
        "restored_tenant_slices": delta("signy_restored_tenant_slices_total"),
        "cycles": {
            "flush": flush_success,
            "merge": merge_success,
            "retention": retention_success,
        },
    });
    report["traces"] = json!({
        "connected": otlp.connected,
        "sent": otlp.sent,
        "spans_sent": otlp.spans_sent,
        "errors": otlp.errors,
        "latency_ms": otlp.latency.summary(),
        "readback": {
            "pass": traces_pass,
            "expected": otlp.verification_expected,
            "attempts": otlp.verify_attempts,
            "verified": otlp.verified,
            "missing": otlp.missing,
            "gave_up_after_budget": otlp.gave_up_after_budget,
            "missing_at_seconds": otlp.missing_at,
            "short": otlp.short,
            "unexpected_status": otlp.unexpected_status,
            "quota_rejected": otlp.quota_rejected,
            "unreachable": otlp.unreachable,
            "retried": otlp.retried,
            "max_attempts": cfg.trace_verify_max_attempts,
            "search_probes": otlp.search_probes,
            "search_empty": otlp.search_empty,
            "first_error": otlp.first_verify_error,
        },
    });
    report["corpus"] = json!({
        "streams": corpus.streams.len(),
        "tenants": corpus.tenant_ids.len(),
        "rows": corpus.entry_count(),
        "distinct_label_bytes": corpus.distinct_label_bytes(),
        "line_bytes": corpus.line_bytes(),
        "compression": compression,
        "wire_snappy_ratio": if push.wire_bytes > 0 {
            json!(push.line_bytes as f64 / push.wire_bytes as f64)
        } else {
            Value::Null
        },
        "source": "signy::corpus, the generator benches/ measures",
    });
    report["coordinated_omission"] = json!({
        "push_response_p99_ms": push_response_p99,
        "push_service_p99_ms": push_service_p99,
        "gap_p99_ms": match (push_response_p99, push_service_p99) {
            (Some(response), Some(service)) => json!(response - service),
            _ => Value::Null,
        },
        "queueing_delay_ms": push.steady.queueing.summary(),
        "note": "a large gap means the harness could not issue on schedule, so the service \
    percentiles describe a lower rate than the one offered. The floor is the harness's own: the \
    timer wheel and a channel handoff put roughly a millisecond under every queueing delay, so a \
    queueing p50 sitting at that floor is this process, not the server.",
    });
    report["config"] = serde_json::to_value(cfg).expect("config serialization");
    report["server_environment"] = json!(config::server_environment());
    report
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn unix_seconds() -> i64 {
    (unix_nanos() / 1_000_000_000) as i64
}

fn unix_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .min(u64::MAX as u128) as u64
}
