//! Every knob the run was given, in one struct that is also what the result
//! file records.
//!
//! A result that cannot be reproduced is a number without a claim attached, so
//! the report carries this whole struct verbatim alongside the build revision,
//! the seed, the machine profile and the server's own environment.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Serialize;

/// signy's whole write surface. All three signals go to it; the header says
/// which.
const COLLECT_PATH: &str = "/signy/api/v1/collect";

/// Which signal a batch carries, for the collect route's header.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Logs,
    Traces,
    Metrics,
}

impl Signal {
    fn collect_headers(self) -> &'static [(&'static str, &'static str)] {
        const ZSTD: (&str, &str) = ("Content-Encoding", "zstd");
        match self {
            Signal::Logs => &[ZSTD, ("x-collecty-signal", "logs")],
            Signal::Traces => &[ZSTD, ("x-collecty-signal", "traces")],
            Signal::Metrics => &[ZSTD, ("x-collecty-signal", "metrics")],
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Targets {
    /// Gated on the **response** percentiles: the service ones describe a rate
    /// that may never have been offered.
    pub push_response_p95_ms: f64,
    pub push_response_p99_ms: f64,
    pub query_response_p95_ms: f64,
    pub rss_max_bytes: u64,
    pub max_error_rate: f64,
    /// A 429 is the engine defending itself, so it is not an error — but a run
    /// that was refused most of what it offered has not measured the rate it
    /// claims to, and must not read as a clean pass.
    pub max_throttled_rate: f64,
    pub wal_backlog_max_bytes: u64,
    pub min_backlog_samples: usize,
}

/// Which system this run is driving.
///
/// The point of the comparison bed is that this is the *only* variable: the
/// corpus, the seed, the offered rate, the queries and the wire format are the
/// same on both sides, because signy's HTTP surface is Loki-compatible by
/// design. What differs is a readiness check that means the same thing in two
/// vocabularies, two sets of `/metrics` names, and which behavioural gates can
/// be asked at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Target {
    Signy,
    Loki,
    /// The design this engine took its logical model from.
    ///
    /// `docs/ARCHITECTURE.md` names VictoriaLogs' `lib/logstorage` as the design
    /// reference, so comparing against it asks a different question from
    /// comparing against Loki: not "are we competitive" but "did the place we
    /// diverged — Parquet instead of their own columnar format — pay".
    ///
    /// It accepts the Loki push API, so ingest is the same bytes. It does not
    /// speak LogQL, so every query has to be translated and every response
    /// parsed differently; that asymmetry is why the two halves of this adapter
    /// look so unalike.
    VictoriaLogs,
    /// The metrics bed's one competitor (issue #8, M14).
    ///
    /// It answers only the metric phases: the log phases refuse it in `main`
    /// before any workload is built, because a log run against a metrics
    /// engine would measure nothing either claim is about.
    VictoriaMetrics,
    /// Grafana Mimir's monolithic OTLP metrics endpoint. The capacity bed
    /// drives this target only for metric-load: Mimir's PromQL query surface
    /// is intentionally outside the existing metric matrix.
    Mimir,
}

impl Target {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "signy" => Ok(Target::Signy),
            "loki" => Ok(Target::Loki),
            "victorialogs" | "victoria-logs" | "vl" => Ok(Target::VictoriaLogs),
            "victoriametrics" | "victoria-metrics" | "vm" => Ok(Target::VictoriaMetrics),
            "mimir" | "grafana-mimir" => Ok(Target::Mimir),
            other => Err(format!(
                "SIGNY_LOAD_TARGET must be signy, loki, victorialogs, victoriametrics or \
mimir, got {other:?}"
            )),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Target::Signy => "signy",
            Target::Loki => "loki",
            Target::VictoriaLogs => "victorialogs",
            Target::VictoriaMetrics => "victoriametrics",
            Target::Mimir => "mimir",
        }
    }

    /// The header a **read** names its tenant with, and its value.
    ///
    /// A query carries no payload, so every target still names its tenant in a
    /// header here. signy's is `X-Tenant-Id`, named after the attribute its
    /// writes use.
    pub fn read_tenant_header(self, tenant: &str) -> (&'static str, String) {
        let name = match self {
            Target::Signy => "X-Tenant-Id",
            _ => "X-Scope-OrgID",
        };
        (name, self.tenant_header(tenant))
    }

    /// The header a **write** names its tenant with, if it has one.
    ///
    /// signy has none: its tenant rides inside the export, as the `tenant.id`
    /// resource attribute, so a push that also sent a header would be naming
    /// it twice and testing neither.
    pub fn push_tenant_header(self, tenant: &str) -> Option<(&'static str, String)> {
        match self {
            Target::Signy => None,
            _ => Some(("X-Scope-OrgID", self.tenant_header(tenant))),
        }
    }

    /// What to put in a tenant header for this system.
    ///
    /// VictoriaLogs reads that header as its numeric `AccountID` and refuses
    /// anything that is not a `uint32` — `verify-tenant-000` comes back as
    /// `cannot parse "verify-tenant-000" as uint32`. Its tenancy is
    /// `AccountID:ProjectID`, not an opaque string, so a name cannot be carried
    /// across. The comparison corpus is single-tenant, so account `0` holds all
    /// of it and nothing is lost; a multi-tenant comparison would need a
    /// name-to-number mapping and would be measuring something else.
    fn tenant_header(self, tenant: &str) -> String {
        match self {
            Target::Signy | Target::Loki => tenant.to_string(),
            // Single-node VictoriaMetrics has no tenancy at all and ignores the
            // header; "0" keeps the request identical to the VictoriaLogs one
            // rather than inventing a third spelling.
            Target::VictoriaLogs | Target::VictoriaMetrics => "0".to_string(),
            Target::Mimir => tenant.to_string(),
        }
    }

    /// Where OTLP logs are POSTed.
    ///
    /// Three targets take the export bare, at three spellings of the
    /// collector path: Loki nests it under `/otlp`, VictoriaLogs under
    /// `/insert/opentelemetry`, VictoriaMetrics under `/opentelemetry` — all
    /// measured accepting the identical protobuf body (2026-08-02, Loki 3.3.2
    /// / VictoriaLogs v1.52.0).
    ///
    /// signy no longer has such an endpoint. Its OTLP push routes were removed
    /// with its gRPC services, so a collecty's batch is the only way in, and
    /// the bed sends what the one intended producer sends: the same export,
    /// wrapped by [`Target::wrap_push`] and headed by [`Target::push_headers`].
    /// The payload is still byte-identical across targets; the framing around
    /// it is not, and `docs/LOAD_VALIDATION.md` records what that costs the
    /// comparison.
    pub fn push_path(self) -> &'static str {
        match self {
            Target::Signy => COLLECT_PATH,
            Target::Loki => "/otlp/v1/logs",
            Target::VictoriaLogs => "/insert/opentelemetry/v1/logs",
            Target::VictoriaMetrics => "/opentelemetry/v1/metrics",
            Target::Mimir => "/otlp/v1/metrics",
        }
    }

    /// Where OTLP metrics are POSTed, for the metric phases. `None` is the
    /// refusal: a target with no metrics ingest cannot join the metrics bed.
    pub fn metric_push_path(self) -> Option<&'static str> {
        match self {
            Target::Signy => Some(COLLECT_PATH),
            Target::VictoriaMetrics => Some("/opentelemetry/v1/metrics"),
            Target::Mimir => Some("/otlp/v1/metrics"),
            Target::Loki | Target::VictoriaLogs => None,
        }
    }

    /// The headers signy's collect route needs and no other target sends: how
    /// the batch is compressed, and which of the three signals it carries.
    ///
    /// No sender or segment header. Those number a collecty's queue, and a
    /// harness has none — signy reads their absence as "nothing to resume"
    /// and stores every record it is given.
    pub fn push_headers(self, signal: Signal) -> &'static [(&'static str, &'static str)] {
        match self {
            Target::Signy => signal.collect_headers(),
            _ => &[],
        }
    }

    /// One OTLP export, framed the way the endpoint takes it.
    ///
    /// For signy that is a one-record collecty batch: the payload behind its
    /// length, zstd over the whole thing. Every other target takes the export
    /// as it stands.
    pub fn wrap_push(self, payload: Vec<u8>) -> Vec<u8> {
        match self {
            Target::Signy => {
                let mut plain = Vec::with_capacity(4 + payload.len());
                plain.extend_from_slice(&(payload.len() as u32).to_le_bytes());
                plain.extend_from_slice(&payload);
                zstd::bulk::compress(&plain, 3).expect("zstd compresses a batch")
            }
            _ => payload,
        }
    }

    /// Where a run waits for the system to answer before it starts.
    ///
    /// Not `/ready` everywhere: the Victoria* line exposes `/health` and has no
    /// separate readiness notion, so asking for one would wait out a timeout on
    /// a system that was up the whole time.
    pub fn ready_path(self) -> &'static str {
        match self {
            Target::Signy | Target::Loki => "/ready",
            Target::VictoriaLogs | Target::VictoriaMetrics => "/health",
            Target::Mimir => "/ready",
        }
    }
}

/// Where a push goes and how it is framed.
///
/// The bed's default is straight at the system under test. A collecty in front
/// is the other way in, and the one production has: the harness then stands
/// where an application's exporter stands, and the batch signy reads is one a
/// real collecty built, numbered and resumable. What differs is only the
/// framing — a bare export on that signal's own path, with no batch frame and
/// none of the collect route's headers, because collecty never decodes what it
/// forwards and would refuse anything else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PushWire {
    pub target: Target,
    /// Set when pushes go to a collecty rather than to the target itself.
    pub through_collecty: bool,
}

impl PushWire {
    #[cfg(test)]
    pub fn direct(target: Target) -> Self {
        Self {
            target,
            through_collecty: false,
        }
    }

    /// Where this signal's export is POSTed. `None` is the refusal a target
    /// with no ingest for the signal gives.
    ///
    /// collecty serves three paths and nothing else, one per signal, and takes
    /// every signal the same way — so unlike a target it never answers `None`.
    pub fn path(self, signal: Signal) -> Option<&'static str> {
        if self.through_collecty {
            return Some(match signal {
                Signal::Logs => "/v1/logs",
                Signal::Traces => "/v1/traces",
                Signal::Metrics => "/v1/metrics",
            });
        }
        match signal {
            Signal::Metrics => self.target.metric_push_path(),
            Signal::Logs | Signal::Traces => Some(self.target.push_path()),
        }
    }

    /// The headers the endpoint needs beyond the content type.
    ///
    /// None through a collecty: the compression and the signal are collecty's
    /// to declare when it ships the segment, and a `Content-Encoding` on the
    /// way in is refused with `415` — the export has to be plain protobuf for
    /// collecty to store it without decoding it.
    pub fn headers(self, signal: Signal) -> &'static [(&'static str, &'static str)] {
        if self.through_collecty {
            return &[];
        }
        self.target.push_headers(signal)
    }

    /// One OTLP export, framed the way the endpoint takes it.
    pub fn wrap(self, payload: Vec<u8>) -> Vec<u8> {
        if self.through_collecty {
            return payload;
        }
        self.target.wrap_push(payload)
    }

    /// The header a write names its tenant with, if it has one.
    ///
    /// Never one through a collecty. The export names its tenant in the
    /// `tenant.id` resource attribute exactly as it does on the collect route,
    /// and it has to: collecty does not read the attribute and could not move
    /// it into a header, so a run that named the tenant in a header instead
    /// would have every export dropped on arrival at signy.
    pub fn tenant_header(self, tenant: &str) -> Option<(&'static str, String)> {
        if self.through_collecty {
            return None;
        }
        self.target.push_tenant_header(tenant)
    }
}

/// What this invocation does.
///
/// `load` is M8's run and is unchanged. `seed` and `matrix` exist because the
/// query comparison has to run on data both systems provably hold: a paced
/// ingest run sends whatever it managed to send, at wall-clock timestamps, so
/// two runs of it produce two different datasets and any row-level comparison
/// between them would be meaningless. `seed` pushes a fixed corpus at fixed
/// timestamps, so both systems end up holding byte-identical entries, and
/// `matrix` then times and compares queries over that.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    Load,
    Seed,
    Matrix,
    /// The metric analogue of `seed`: the fixed metric dataset, identical OTLP
    /// bodies at identical timestamps on every side (M14, issue #8).
    MetricSeed,
    /// The metric analogue of `matrix`: the fn0 metric shapes over the seeded
    /// dataset, cold and warm, with a per-answer record set the report's
    /// agreement check runs on.
    MetricMatrix,
    /// The paced metric ingest: steady, then rolling series churn, then a
    /// cardinality burst. This is the phase the M14 claim's own half is
    /// measured in — what an engine does when active series outgrow the
    /// budget it was given.
    MetricLoad,
}

impl Phase {
    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "load" => Ok(Phase::Load),
            "seed" => Ok(Phase::Seed),
            "matrix" => Ok(Phase::Matrix),
            "metric-seed" => Ok(Phase::MetricSeed),
            "metric-matrix" => Ok(Phase::MetricMatrix),
            "metric-load" => Ok(Phase::MetricLoad),
            other => Err(format!(
                "SIGNY_LOAD_PHASE must be load, seed, matrix, metric-seed, \
metric-matrix or metric-load, got {other:?}"
            )),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Config {
    pub target: Target,
    pub phase: Phase,
    /// Whether this invocation is paired with signy's disposable raw-capacity
    /// probe.  The server uses the same switch to publish structural gauges;
    /// keeping it in the harness lets the sampler retain those gauges only for
    /// probe trials.
    pub capacity_probe: bool,
    pub http_address: String,
    /// Where pushes go, when that is not `http_address`.
    ///
    /// Set to a collecty's OTLP address to drive the whole stack rather than
    /// the engine alone: writes go there and reads stay on `http_address`,
    /// because collecty has no read surface and signy has no other way in.
    pub push_address: Option<String>,
    /// How long to wait, after the workloads stop, for what the collector
    /// still holds to reach the engine. Ignored without a collector in front:
    /// nothing is queued there to wait for.
    pub drain_seconds: u64,
    /// How long a trace is left alone before the leg reads it back. It has to
    /// cover the whole write path — a collecty's segment age, the send, the
    /// journal — because a probe that runs too early reports a miss the engine
    /// never made.
    pub trace_verify_lag_seconds: u64,
    /// How many times a readback may ask before it calls a trace missing.
    ///
    /// One retry was not a budget, it was a coin toss: a soak's outage is
    /// three minutes and the collector's drain behind it took another two and
    /// a half, so a trace caught by one had two chances a lag apart against a
    /// disturbance five times that long, and the ones that lost were counted
    /// as data loss. Five attempts at the default lag outlast the disturbance;
    /// a trace that is still absent after that is absent.
    pub trace_verify_max_attempts: u32,
    /// How long after an unanswered read a metric shape may still answer
    /// empty without that counting as the read path going quiet.
    pub metric_recovery_grace_seconds: u64,
    /// One trace in this many is kept for the read-back probe. Every one would
    /// double the leg's traffic and turn a garnish into a second query
    /// workload.
    pub trace_verify_sample: u64,
    pub tier: String,
    pub seed: u64,
    pub duration_seconds: u64,
    pub warmup_seconds: u64,
    pub target_events: u64,
    pub request_timeout_seconds: u64,
    pub sample_interval_ms: u64,
    pub server_pid: Option<u32>,
    /// cgroup v2 directory of the server's container, when the server is not a
    /// sibling process. Takes precedence over `server_pid`.
    pub cgroup_path: Option<String>,
    pub result_path: Option<String>,

    pub ingest_connections: usize,
    pub target_eps: f64,
    pub entries_per_push: usize,
    pub streams_per_push: usize,

    pub corpus_rows: usize,
    pub tenants: usize,
    /// The retention pushed when onboarding this run's tenants.
    ///
    /// signy has no server-wide retention period any more: a tenant's policy
    /// is the only place one lives, and pushing it is also what onboards the
    /// tenant. `infinite` keeps a run that is not measuring retention from
    /// deleting its own corpus out from under itself.
    pub tenant_retention: String,
    pub streams: usize,
    pub labels_per_stream: usize,
    pub metadata_pairs: usize,
    pub plain_weight: u32,
    pub json_weight: u32,
    pub logfmt_weight: u32,

    pub entry_spread_ms: u64,
    pub late_fraction: f64,
    pub late_max_ms: u64,

    pub query_connections: usize,
    pub query_eps: f64,
    pub query_window_seconds: i64,
    pub query_limit: usize,
    pub heavy_window_seconds: i64,
    pub heavy_limit: usize,
    pub restore_lookback_seconds: i64,
    pub query_weights: [u32; 6],

    pub otlp_eps: f64,

    /// The load phase's metric leg: seconds between scrapes of a live
    /// population, `0` for no metric ingest at all.
    ///
    /// Off by default, so every run recorded before the leg existed means what
    /// it meant. The soak turns it on: metrics are the one signal that has
    /// never been through a long run, and a day is where a per-part cost or a
    /// catalog that does not retract shows up.
    pub metric_leg_scrape_seconds: u64,
    /// Instances replaced per scrape, standing in for a deploy rolling pods.
    /// `0` keeps the population fixed.
    pub metric_leg_churn_per_scrape: usize,
    /// Metric reads per second alongside the log queries. `0` for none.
    pub metric_query_eps: f64,
    /// How far back the leg's range shapes ask, in seconds.
    ///
    /// A knob rather than a constant because retention is one: a window longer
    /// than the tenant keeps reads empty, and the leg's empty-answer gate
    /// would then be firing on the run's own configuration rather than on the
    /// engine.
    pub metric_query_window_seconds: i64,

    pub verify: Verify,
    pub metric_verify: MetricVerify,
    pub targets: Targets,
}

/// The fixed dataset the query comparison runs on, and the query matrix over
/// it.
#[derive(Clone, Debug, Serialize)]
pub struct Verify {
    pub tenant_prefix: String,
    pub rows: usize,
    pub streams: usize,
    pub labels_per_stream: usize,
    /// Nanoseconds between consecutive rows in *log* time. The dataset spans
    /// `rows * step_ns`, which is what the query windows are cut out of.
    pub step_ns: i64,
    /// Unix nanoseconds the dataset starts at. Zero means "derive it from the
    /// clock", which is only correct for a single-system run: the two runs of
    /// a comparison must be given the same anchor explicitly, or they are not
    /// holding the same rows.
    pub anchor_ns: i64,
    pub entries_per_push: usize,
    pub push_connections: usize,
    /// Sub-windows the dataset is cut into. Queries are (app x sub-window), so
    /// this is what makes a cold query cold: a window nothing has asked for
    /// before cannot be answered from either system's result cache.
    pub windows: usize,
    /// Warm repeats of each query after its cold issue.
    pub repeats: usize,
    pub limit: usize,
    /// One knob for both the metric sample grid and the `rate` window. They
    /// are the same number on purpose: LogQL evaluates a sliding window per
    /// step and LogsQL cuts tumbling `_time` buckets, and the two produce the
    /// same set of samples only when the window equals the step and the query
    /// windows are aligned to it — consecutive lookbacks then tile the range
    /// exactly like buckets do.
    pub step_seconds: i64,
}

/// The fixed metric dataset the metrics comparison runs on, and the query
/// matrix over it (M14, issue #8).
///
/// The vocabulary knobs are deliberately few: the dataset's shape is part of
/// the ruler, and a knob per axis would invite tuning the corpus toward
/// whichever engine the run is flattering. What varies is scale (scrapes,
/// services, instances) and the matrix mechanics; the instrument names, the
/// histogram bounds and the churn layout are constants in
/// `metric_workload.rs`.
#[derive(Clone, Debug, Serialize)]
pub struct MetricVerify {
    pub tenant: String,
    /// Unix nanoseconds of the first scrape. Zero is refused for the same
    /// reason the log anchor's zero is: two runs deriving it from their own
    /// clocks would seed different datasets.
    pub anchor_ns: i64,
    /// Scrapes per active series. The dataset spans
    /// `scrapes * scrape_interval_seconds`.
    pub scrapes: usize,
    pub scrape_interval_seconds: i64,
    /// Steady services × instances is the steady series population; every
    /// (service, instance) pair carries the full instrument vocabulary.
    pub services: usize,
    pub instances_per_service: usize,
    pub gauges: usize,
    pub counters: usize,
    /// Generations the churn service's instances are replaced across: each
    /// generation's series report only in its slice of the scrape range, which
    /// is the pod-restart shape the `churned_selector` query must cross.
    pub churn_generations: usize,
    pub churn_instances: usize,
    pub push_connections: usize,
    /// Sub-windows the span is cut into, per shape — what makes a cold query
    /// cold, exactly as in the log matrix.
    pub windows: usize,
    pub repeats: usize,
    /// The range-query step. A multiple of the scrape interval, and the query
    /// windows are aligned to it, so both engines evaluate on the same grid.
    pub step_seconds: i64,
    /// The `rate`/`increase`/quantile window. One knob for every windowed
    /// shape so a ratio difference is never the two engines being asked
    /// different windows.
    pub range_seconds: i64,

    // The paced ingest phases (`metric-load`), which are a different
    // experiment from the seeded dataset above: these run in wall-clock and
    // measure what an engine does to a *rate* of series, not what it answers
    // about a fixed corpus.
    /// Seconds of the fixed series population, before any churn.
    pub steady_seconds: u64,
    /// Seconds of rolling instance replacement — the pod-restart shape, the
    /// claim's own axis.
    pub churn_seconds: u64,
    /// Instances replaced per scrape during the churn phase. Sized so
    /// `churn_seconds / scrape_interval * this` pushes active + idle series
    /// past what the container's budget can index.
    pub churn_replace_per_scrape: usize,
    /// Seconds after the burst, so the recovery is measured rather than
    /// assumed.
    pub explosion_seconds: u64,
    /// Distinct new series minted in one scrape at the start of the
    /// explosion phase.
    pub explosion_series: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let duration_seconds = env_u64("SIGNY_LOAD_SECONDS", 60).max(1);
        Self {
            target: Target::parse(&env_string("SIGNY_LOAD_TARGET", "signy"))?,
            phase: Phase::parse(&env_string("SIGNY_LOAD_PHASE", "load"))?,
            capacity_probe: env_bool("SIGNY_CAPACITY_PROBE", false),
            http_address: env_string("SIGNY_LOAD_ADDR", "127.0.0.1:3100"),
            push_address: std::env::var("SIGNY_LOAD_PUSH_ADDR")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            drain_seconds: env_u64("SIGNY_LOAD_DRAIN_SECONDS", 180),
            trace_verify_lag_seconds: env_u64("SIGNY_LOAD_TRACE_VERIFY_LAG_SECONDS", 60),
            trace_verify_max_attempts: env_u64("SIGNY_LOAD_TRACE_VERIFY_MAX_ATTEMPTS", 5) as u32,
            metric_recovery_grace_seconds: env_u64("SIGNY_LOAD_METRIC_RECOVERY_GRACE_SECONDS", 300),
            trace_verify_sample: env_u64("SIGNY_LOAD_TRACE_VERIFY_SAMPLE", 10).max(1),
            tier: env_string("SIGNY_LOAD_TIER", "B"),
            seed: env_u64("SIGNY_LOAD_SEED", 0x5eed_2026),
            duration_seconds,
            warmup_seconds: env_u64("SIGNY_LOAD_WARMUP_SECONDS", 10)
                .min(duration_seconds.saturating_sub(1)),
            target_events: env_u64("SIGNY_LOAD_EVENTS", 0),
            request_timeout_seconds: env_u64("SIGNY_LOAD_REQUEST_TIMEOUT_SECONDS", 60).max(1),
            sample_interval_ms: env_u64("SIGNY_LOAD_SAMPLE_INTERVAL_MS", 1000).max(50),
            server_pid: std::env::var("SIGNY_LOAD_SERVER_PID")
                .ok()
                .and_then(|raw| raw.trim().parse::<u32>().ok()),
            cgroup_path: std::env::var("SIGNY_LOAD_CGROUP")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            result_path: std::env::var("SIGNY_LOAD_RESULT_PATH").ok(),

            ingest_connections: env_usize("SIGNY_LOAD_CONNECTIONS", 8).max(1),
            target_eps: env_f64("SIGNY_LOAD_TARGET_EPS", 3000.0).max(0.0),
            entries_per_push: env_usize("SIGNY_LOAD_ENTRIES_PER_PUSH", 100).max(1),
            streams_per_push: env_usize("SIGNY_LOAD_STREAMS_PER_PUSH", 4).max(1),

            corpus_rows: env_usize("SIGNY_LOAD_CORPUS_ROWS", 50_000).max(1),
            tenants: env_usize("SIGNY_LOAD_TENANTS", 4).max(1),
            tenant_retention: env_string("SIGNY_LOAD_TENANT_RETENTION", "infinite"),
            streams: env_usize("SIGNY_LOAD_STREAMS", 256).max(1),
            labels_per_stream: env_usize("SIGNY_LOAD_LABELS_PER_STREAM", 6).clamp(1, 10),
            metadata_pairs: env_usize("SIGNY_LOAD_METADATA_PAIRS", 2),
            plain_weight: env_u64("SIGNY_LOAD_PLAIN_WEIGHT", 3) as u32,
            json_weight: env_u64("SIGNY_LOAD_JSON_WEIGHT", 5) as u32,
            logfmt_weight: env_u64("SIGNY_LOAD_LOGFMT_WEIGHT", 2) as u32,

            entry_spread_ms: env_u64("SIGNY_LOAD_ENTRY_SPREAD_MS", 250),
            late_fraction: env_f64("SIGNY_LOAD_LATE_FRACTION", 0.02).clamp(0.0, 1.0),
            late_max_ms: env_u64("SIGNY_LOAD_LATE_MAX_MS", 30_000),

            query_connections: env_usize("SIGNY_LOAD_QUERY_CONNECTIONS", 4).max(1),
            query_eps: env_f64("SIGNY_LOAD_QUERY_EPS", 5.0).max(0.0),
            query_window_seconds: env_u64("SIGNY_LOAD_QUERY_WINDOW_SECONDS", 60) as i64,
            query_limit: env_usize("SIGNY_LOAD_QUERY_LIMIT", 100).max(1),
            restore_lookback_seconds: env_u64("SIGNY_LOAD_RESTORE_LOOKBACK_SECONDS", 60) as i64,
            query_weights: [
                env_u64("SIGNY_LOAD_QUERY_WEIGHT_LABEL_ONLY", 3) as u32,
                env_u64("SIGNY_LOAD_QUERY_WEIGHT_LINE_FILTER", 3) as u32,
                env_u64("SIGNY_LOAD_QUERY_WEIGHT_JSON_FIELD", 3) as u32,
                env_u64("SIGNY_LOAD_QUERY_WEIGHT_RATE", 2) as u32,
                env_u64("SIGNY_LOAD_QUERY_WEIGHT_RESTORE_PROBE", 1) as u32,
                // Zero: the heavy shape is an instrument for measuring what a
                // slow query does to everyone else, opted into per run.
                env_u64("SIGNY_LOAD_QUERY_WEIGHT_HEAVY", 0) as u32,
            ],
            heavy_window_seconds: env_u64("SIGNY_LOAD_HEAVY_WINDOW_SECONDS", 3600) as i64,
            heavy_limit: env_usize("SIGNY_LOAD_HEAVY_LIMIT", 20000).max(1),

            otlp_eps: env_f64("SIGNY_LOAD_OTLP_EPS", 5.0).max(0.0),

            metric_leg_scrape_seconds: env_u64("SIGNY_LOAD_METRIC_LEG_SCRAPE_SECONDS", 0),
            metric_leg_churn_per_scrape: env_usize("SIGNY_LOAD_METRIC_LEG_CHURN_PER_SCRAPE", 0),
            metric_query_eps: env_f64("SIGNY_LOAD_METRIC_QUERY_EPS", 0.0).max(0.0),
            metric_query_window_seconds: env_u64("SIGNY_LOAD_METRIC_QUERY_WINDOW_SECONDS", 900)
                .max(1) as i64,

            verify: Verify {
                tenant_prefix: env_string("SIGNY_LOAD_VERIFY_TENANT_PREFIX", "verify-tenant"),
                rows: env_usize("SIGNY_LOAD_VERIFY_ROWS", 120_000).max(1),
                streams: env_usize("SIGNY_LOAD_VERIFY_STREAMS", 32).max(1),
                labels_per_stream: env_usize("SIGNY_LOAD_VERIFY_LABELS", 6).clamp(1, 10),
                step_ns: env_u64("SIGNY_LOAD_VERIFY_STEP_NS", 1_000_000).max(1) as i64,
                anchor_ns: env_u64("SIGNY_LOAD_VERIFY_ANCHOR_NS", 0) as i64,
                entries_per_push: env_usize("SIGNY_LOAD_VERIFY_ENTRIES_PER_PUSH", 100).max(1),
                push_connections: env_usize("SIGNY_LOAD_VERIFY_CONNECTIONS", 4).max(1),
                windows: env_usize("SIGNY_LOAD_MATRIX_WINDOWS", 3).max(1),
                repeats: env_usize("SIGNY_LOAD_MATRIX_REPEATS", 5).max(1),
                limit: env_usize("SIGNY_LOAD_MATRIX_LIMIT", 20_000).max(1),
                step_seconds: env_u64("SIGNY_LOAD_MATRIX_STEP_SECONDS", 10).max(1) as i64,
            },

            metric_verify: MetricVerify {
                tenant: env_string("SIGNY_LOAD_METRIC_TENANT", "verify-metrics"),
                anchor_ns: env_u64("SIGNY_LOAD_METRIC_ANCHOR_NS", 0) as i64,
                scrapes: env_usize("SIGNY_LOAD_METRIC_SCRAPES", 360).max(4),
                scrape_interval_seconds: env_u64("SIGNY_LOAD_METRIC_SCRAPE_SECONDS", 10).max(1)
                    as i64,
                services: env_usize("SIGNY_LOAD_METRIC_SERVICES", 8).max(1),
                instances_per_service: env_usize("SIGNY_LOAD_METRIC_INSTANCES", 4).max(1),
                gauges: env_usize("SIGNY_LOAD_METRIC_GAUGES", 4).max(1),
                counters: env_usize("SIGNY_LOAD_METRIC_COUNTERS", 4).max(1),
                churn_generations: env_usize("SIGNY_LOAD_METRIC_CHURN_GENERATIONS", 4).max(1),
                churn_instances: env_usize("SIGNY_LOAD_METRIC_CHURN_INSTANCES", 4).max(1),
                push_connections: env_usize("SIGNY_LOAD_METRIC_CONNECTIONS", 4).max(1),
                windows: env_usize("SIGNY_LOAD_METRIC_WINDOWS", 3).max(1),
                repeats: env_usize("SIGNY_LOAD_METRIC_REPEATS", 5).max(1),
                step_seconds: env_u64("SIGNY_LOAD_METRIC_STEP_SECONDS", 30).max(1) as i64,
                range_seconds: env_u64("SIGNY_LOAD_METRIC_RANGE_SECONDS", 60).max(1) as i64,
                steady_seconds: env_u64("SIGNY_LOAD_METRIC_STEADY_SECONDS", 60),
                churn_seconds: env_u64("SIGNY_LOAD_METRIC_CHURN_SECONDS", 120),
                churn_replace_per_scrape: env_usize("SIGNY_LOAD_METRIC_CHURN_REPLACE", 64),
                explosion_seconds: env_u64("SIGNY_LOAD_METRIC_EXPLOSION_SECONDS", 60),
                explosion_series: env_usize("SIGNY_LOAD_METRIC_EXPLOSION_SERIES", 50_000),
            },

            targets: Targets {
                push_response_p95_ms: env_f64("SIGNY_TARGET_PUSH_RESPONSE_P95_MS", 250.0),
                push_response_p99_ms: env_f64("SIGNY_TARGET_PUSH_RESPONSE_P99_MS", 1000.0),
                query_response_p95_ms: env_f64("SIGNY_TARGET_QUERY_RESPONSE_P95_MS", 2000.0),
                rss_max_bytes: env_u64("SIGNY_TARGET_RSS_MAX_BYTES", 4 * 1024 * 1024 * 1024),
                max_error_rate: env_f64("SIGNY_TARGET_MAX_ERROR_RATE", 0.0),
                max_throttled_rate: env_f64("SIGNY_TARGET_MAX_THROTTLED_RATE", 0.05),
                // The engine's own `max_wal_backlog_bytes` is where
                // backpressure engages, and engaging is the design working, so
                // the ceiling is that rather than a number the harness invented.
                wal_backlog_max_bytes: env_u64(
                    "SIGNY_TARGET_WAL_BACKLOG_MAX_BYTES",
                    1024 * 1024 * 1024,
                ),
                min_backlog_samples: env_usize("SIGNY_TARGET_MIN_BACKLOG_SAMPLES", 8).max(2),
            },
        }
        .validated()
    }

    /// Combinations that cannot mean anything, refused before a run starts
    /// rather than measured and reported.
    fn validated(self) -> Result<Self, String> {
        // A collecty forwards to signy and to nothing else, so pushing through
        // one while reading from another engine would put the load in one
        // system and the queries in a different one.
        if self.push_address.is_some() && self.target != Target::Signy {
            return Err(format!(
                "SIGNY_LOAD_PUSH_ADDR names a collecty, which forwards only to signy, but SIGNY_LOAD_TARGET is {}",
                self.target.name()
            ));
        }
        Ok(self)
    }

    /// Where the server's resident memory is read from, or the reason no
    /// reading is possible. An unmeasurable peak is an error the run carries,
    /// never a zero.
    pub fn memory_source(&self) -> Result<crate::probe::MemorySource, String> {
        match (self.cgroup_path.as_ref(), self.server_pid) {
            (Some(dir), _) => Ok(crate::probe::MemorySource::Cgroup(dir.clone())),
            (None, Some(pid)) => Ok(crate::probe::MemorySource::Proc(pid)),
            (None, None) => Err(
                "neither SIGNY_LOAD_CGROUP nor SIGNY_LOAD_SERVER_PID was set, so no \
server memory could be watched"
                    .to_string(),
            ),
        }
    }

    /// Log-time span the verification dataset covers, in nanoseconds.
    pub fn verify_span_ns(&self) -> i64 {
        self.verify.rows as i64 * self.verify.step_ns
    }

    /// Metric-time span the metric verification dataset covers, in
    /// nanoseconds: the scrape grid from the anchor to one interval past the
    /// last scrape.
    pub fn metric_span_ns(&self) -> i64 {
        self.metric_verify.scrapes as i64
            * self.metric_verify.scrape_interval_seconds
            * 1_000_000_000
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_seconds)
    }

    /// Where pushes are sent. The read address unless a collecty stands in
    /// front, which is the only thing that separates the two.
    pub fn push_address(&self) -> &str {
        self.push_address.as_deref().unwrap_or(&self.http_address)
    }

    /// How pushes are framed, which follows from where they go.
    pub fn push_wire(&self) -> PushWire {
        PushWire {
            target: self.target,
            through_collecty: self.push_address.is_some(),
        }
    }

    /// Seconds between pushes that hold the offered event rate, or `None` when
    /// the run is unpaced.
    pub fn push_interval(&self) -> Option<Duration> {
        (self.target_eps > 0.0)
            .then(|| Duration::from_secs_f64(self.entries_per_push as f64 / self.target_eps))
    }

    pub fn query_interval(&self) -> Option<Duration> {
        (self.query_eps > 0.0).then(|| Duration::from_secs_f64(1.0 / self.query_eps))
    }

    pub fn otlp_interval(&self) -> Option<Duration> {
        (self.otlp_eps > 0.0).then(|| Duration::from_secs_f64(1.0 / self.otlp_eps))
    }

    /// Seconds between the metric leg's scrapes, or `None` when the leg is off.
    pub fn metric_leg_interval(&self) -> Option<Duration> {
        (self.metric_leg_scrape_seconds > 0)
            .then(|| Duration::from_secs(self.metric_leg_scrape_seconds))
    }

    pub fn metric_query_interval(&self) -> Option<Duration> {
        (self.metric_query_eps > 0.0).then(|| Duration::from_secs_f64(1.0 / self.metric_query_eps))
    }
}

/// The server's knobs as this process can see them.
///
/// The run script exports them into the environment the harness inherits, so
/// this is the configuration the server was started with — recorded because a
/// result that does not say what the flush interval was is not reproducible.
/// `SIGNY_LOAD_*` and `SIGNY_TARGET_*` are excluded: those are the
/// harness's own and are already in `Config`.
pub fn server_environment() -> BTreeMap<String, String> {
    std::env::vars()
        .filter(|(name, _)| {
            name.starts_with("SIGNY_")
                && !name.starts_with("SIGNY_LOAD_")
                && !name.starts_with("SIGNY_TARGET_")
                && name != "SIGNY_BUILD_REVISION"
                && name != "SIGNY_MACHINE_PROFILE"
        })
        .collect()
}

pub fn build_revision() -> String {
    std::env::var("SIGNY_BUILD_REVISION")
        .or_else(|_| std::env::var("GIT_COMMIT"))
        .unwrap_or_else(|_| "unknown".to_string())
}

pub fn machine_profile() -> String {
    if let Ok(profile) = std::env::var("SIGNY_MACHINE_PROFILE") {
        return profile;
    }
    // Read rather than left as "unspecified": a latency number whose machine
    // is unknown cannot be compared with anything.
    let cpus = std::thread::available_parallelism()
        .map(|count| count.get().to_string())
        .unwrap_or_else(|_| "?".to_string());
    let memory = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|line| line.strip_prefix("MemTotal:"))
                .and_then(|value| {
                    value
                        .trim()
                        .trim_end_matches(" kB")
                        .trim()
                        .parse::<u64>()
                        .ok()
                })
        })
        .map(|kilobytes| format!("{:.1} GiB RAM", kilobytes as f64 / 1_048_576.0))
        .unwrap_or_else(|| "unknown RAM".to_string());
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|text| text.trim().to_string())
        .unwrap_or_else(|_| "unknown kernel".to_string());
    format!("{kernel}; {cpus} logical CPUs; {memory}")
}

fn env_string(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_bool(name: &str, default: bool) -> bool {
    match std::env::var(name)
        .ok()
        .as_deref()
        .map(str::to_ascii_lowercase)
    {
        Some(value) if matches!(value.as_str(), "1" | "true" | "yes") => true,
        Some(value) if matches!(value.as_str(), "0" | "false" | "no") => false,
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Through a collecty the export leaves the harness exactly as an
    /// application's exporter would send it: bare protobuf, on the signal's own
    /// path, with no batch frame and no collect headers. Anything else is
    /// refused at the door — a `Content-Encoding` with `415`, a non-POST with
    /// `405` — so this is not a preference.
    #[test]
    fn a_push_through_a_collecty_is_the_bare_export_on_the_signals_own_path() {
        let wire = PushWire {
            target: Target::Signy,
            through_collecty: true,
        };
        assert_eq!(wire.path(Signal::Logs), Some("/v1/logs"));
        assert_eq!(wire.path(Signal::Traces), Some("/v1/traces"));
        assert_eq!(wire.path(Signal::Metrics), Some("/v1/metrics"));
        assert!(wire.headers(Signal::Logs).is_empty());
        assert!(wire.headers(Signal::Metrics).is_empty());
        let payload = b"an export".to_vec();
        assert_eq!(wire.wrap(payload.clone()), payload);
        // The tenant stays in the body. collecty does not decode what it
        // forwards, so a header here would reach signy as no tenant at all and
        // every record would be dropped on arrival.
        assert_eq!(wire.tenant_header("acme"), None);
    }

    /// The same wire straight at signy is the collect route's, unchanged by
    /// the knob existing.
    #[test]
    fn a_push_straight_at_signy_is_still_a_one_record_collecty_batch() {
        let wire = PushWire::direct(Target::Signy);
        assert_eq!(wire.path(Signal::Logs), Some(COLLECT_PATH));
        assert_eq!(wire.path(Signal::Metrics), Some(COLLECT_PATH));
        assert_eq!(
            wire.headers(Signal::Traces),
            Signal::Traces.collect_headers()
        );
        let framed = wire.wrap(b"an export".to_vec());
        assert_ne!(framed, b"an export".to_vec());
        assert_eq!(wire.tenant_header("acme"), None);
    }

    /// A target with no ingest for a signal still has none through this type:
    /// the collector is the only thing that takes every signal the same way.
    #[test]
    fn a_target_without_metrics_ingest_still_refuses_the_metric_signal() {
        assert_eq!(PushWire::direct(Target::Loki).path(Signal::Metrics), None);
        assert_eq!(
            PushWire::direct(Target::VictoriaLogs).path(Signal::Metrics),
            None
        );
    }

    #[test]
    fn mimir_uses_the_native_otlp_metrics_route_and_readiness_probe() {
        let target = Target::parse("grafana-mimir").expect("Mimir target parses");
        assert_eq!(target, Target::Mimir);
        assert_eq!(target.metric_push_path(), Some("/otlp/v1/metrics"));
        assert_eq!(target.ready_path(), "/ready");
        assert_eq!(
            target.push_tenant_header("tenant"),
            Some(("X-Scope-OrgID", "tenant".into()))
        );
    }

    #[test]
    fn metric_target_aliases_are_stable_for_script_routing() {
        assert_eq!(Target::parse("vm").unwrap(), Target::VictoriaMetrics);
        assert_eq!(Target::parse("mimir").unwrap().name(), "mimir");
        assert_eq!(Target::Mimir.metric_push_path(), Some("/otlp/v1/metrics"));
    }
}
