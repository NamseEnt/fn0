//! The two phases the comparison needs that a load run cannot give it.
//!
//! A paced load run sends whatever it managed to send, at wall-clock
//! timestamps. Two runs of it against two systems therefore produce two
//! *different* datasets, and any row-level comparison between them would be
//! comparing arrival timing rather than engines. So the query comparison runs
//! on its own dataset:
//!
//! * **`seed`** pushes a fixed corpus at fixed log timestamps, driven from an
//!   anchor both runs are given. Same seed, same rows, same bytes, same
//!   timestamps — the two systems provably hold the same entries.
//! * **`matrix`** times the four query shapes over that dataset, cold and
//!   warm, and records a digest of every answer so the two runs can be checked
//!   for returning the same response — every row, the labels each row carried,
//!   and where the response put them. A fast wrong answer is not a win, so the
//!   digest is the part of this file that matters most.
//!
//! Neither phase paces, and neither reports a throughput. They are latency and
//! correctness instruments; the rate axis belongs to `Phase::Load`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use serde_json::{Value, json};
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::config::{Config, Signal, Target, Verify};
use crate::http::{Client, Request};
use crate::stats::Series;
use signy::corpus::{APPS, Corpus, CorpusSpec, LEVELS, PHRASES, STATUSES};
use signy::memtable::LogEntry;

/// Keeps the verification dataset's identifiers distinct from the load
/// corpus's while staying a function of the run seed.
const VERIFY_SEED_SALT: u64 = 0x1e_4f_a1_ce;

const PUSH_CONTENT_TYPE: &str = "application/x-protobuf";

/// The four shapes `docs/VISION.md` names, in the order the claim is argued
/// in: two that both systems index for, then the one the claim rests on, then
/// the aggregation that should read one column.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    LabelOnly,
    LineFilter,
    JsonField,
    /// The same parser stage, on a predicate that is actually rare.
    ///
    /// `json_field` filters on `status` or `level`, which match roughly a fifth
    /// of the rows, so it measures how fast an engine scans. This one filters on
    /// a `trace_id` drawn from a population of `rows / 4` — about four rows in a
    /// hundred and fifty thousand — which is what a per-row-group bloom over a
    /// columnized field exists to answer. Without this axis the comparison never
    /// tests the condition the design is supposed to win.
    ///
    /// It carries no `app` selector on purpose. A trace is drawn independently
    /// of the app, so constraining one would leave the answer empty for seven
    /// apps in eight; and "find this trace across everything" is both the real
    /// query and the one where the field predicate is the only selective thing.
    JsonFieldRare,
    /// The same rare value, reached without a parser stage.
    ///
    /// `json_field_rare` asks for a `trace_id` that the corpus wrote *into the
    /// JSON line*, so answering it needs a parser. The corpus also pushes the
    /// same `trace_id` as **structured metadata**, which is what an OTLP
    /// attribute becomes — and that is the shape `docs/VISION.md`'s claim now
    /// rests on, because the one intended consumer sends OTLP.
    ///
    /// Same value, same expected rows, one fewer stage. The pair is the
    /// measurement: it separates what the parser costs from what the storage
    /// costs, and it is where the three systems differ by design — signy
    /// indexes structured metadata into a per-row-group bloom, Loki stores it
    /// without indexing it, VictoriaLogs turns it into a column.
    MetadataRare,
    /// The same rare value inside the window Grafana's trace-to-logs actually
    /// sends: the occurrence's own time, one second either side. The wide
    /// `metadata_rare` asks "find this trace across everything"; this asks the
    /// question the consumer's click asks — and it is the measurement that
    /// decides whether a server-side trace-to-log join buys anything the
    /// client's window has not already bought.
    TraceWindow,
    Rate,
}

pub const SHAPES: [Shape; 7] = [
    Shape::LabelOnly,
    Shape::LineFilter,
    Shape::JsonField,
    Shape::JsonFieldRare,
    Shape::MetadataRare,
    Shape::TraceWindow,
    Shape::Rate,
];

impl Shape {
    pub fn name(self) -> &'static str {
        match self {
            Shape::LabelOnly => "label_only",
            Shape::LineFilter => "line_filter",
            Shape::JsonField => "json_field",
            Shape::JsonFieldRare => "json_field_rare",
            Shape::MetadataRare => "metadata_rare",
            Shape::TraceWindow => "trace_window",
            Shape::Rate => "rate",
        }
    }
}

/// The dataset both systems are seeded with.
///
/// One tenant, so the query phase reads only these rows however much a load
/// phase left behind; a fixed `start_ts_ns`, so the window a query asks for is
/// an absolute instant rather than "the last minute" of two different runs;
/// and `out_of_order: false`, because the arrival-order axis belongs to the
/// load phase and a jittered duplicate here would be a row-equality failure
/// that says nothing about either engine.
pub fn verify_corpus(cfg: &Config) -> Corpus {
    signy::corpus::generate(&verify_spec(cfg))
}

/// The spec `verify_corpus` generates from, on its own so the query builder can
/// rebuild the seeded vocabulary without rebuilding the rows.
fn verify_spec(cfg: &Config) -> CorpusSpec {
    CorpusSpec {
        seed: cfg.seed ^ VERIFY_SEED_SALT,
        tenants: 1,
        streams: cfg.verify.streams,
        labels_per_stream: cfg.verify.labels_per_stream,
        rows: cfg.verify.rows,
        tenant_prefix: cfg.verify.tenant_prefix.clone(),
        plain_weight: cfg.plain_weight,
        json_weight: cfg.json_weight,
        logfmt_weight: cfg.logfmt_weight,
        metadata_pairs: cfg.metadata_pairs,
        start_ts_ns: cfg.verify.anchor_ns,
        step_ns: cfg.verify.step_ns,
        out_of_order: false,
    }
}

/// Anchors that are not `> 0` are refused rather than defaulted.
///
/// A default would be derived from this process's clock, and the two runs of a
/// comparison start minutes apart — the datasets would differ by exactly that
/// gap, which is the failure this whole phase exists to prevent, and it would
/// show up as a row-equality mismatch nobody could explain.
pub fn require_anchor(verify: &Verify) -> Result<(), String> {
    if verify.anchor_ns > 0 {
        return Ok(());
    }
    Err(
        "SIGNY_LOAD_VERIFY_ANCHOR_NS must be set to the same value for both runs of a \
comparison; without it each run would seed a different dataset"
            .to_string(),
    )
}

pub struct SeedOutcome {
    pub pushes: u64,
    pub rows: u64,
    pub line_bytes: u64,
    pub wire_bytes: u64,
    pub retries: u64,
    pub errors: u64,
    pub statuses: BTreeMap<u16, u64>,
    pub first_error: Option<String>,
    pub elapsed_seconds: f64,
}

struct SeedBody {
    bytes: Vec<u8>,
    rows: usize,
    line_bytes: usize,
}

/// One stream per push, `entries_per_push` entries at a time.
///
/// Chunking by stream rather than mixing them keeps the body an agent-shaped
/// one and, more importantly, keeps the push sequence a pure function of the
/// corpus: the same bytes leave this process in the same order for both
/// systems.
fn seed_bodies(
    corpus: &Corpus,
    entries_per_push: usize,
    tenant: Option<&str>,
    wire: crate::config::PushWire,
) -> Vec<SeedBody> {
    let mut bodies = Vec::new();
    for stream in &corpus.streams {
        for chunk in stream.entries.chunks(entries_per_push) {
            let line_bytes: usize = chunk.iter().map(|entry| entry.line.len()).sum();
            let batch: Vec<(signy::memtable::Labels, Vec<LogEntry>)> =
                vec![((*stream.labels).clone(), chunk.to_vec())];
            bodies.push(SeedBody {
                bytes: wire.wrap(crate::otlp::encode_export_logs(&batch, tenant)),
                rows: chunk.len(),
                line_bytes,
            });
        }
    }
    bodies
}

pub async fn run_seed(cfg: &Config, corpus: &Corpus) -> SeedOutcome {
    let wire = cfg.push_wire();
    let push_path = wire.path(Signal::Logs).expect("signy takes logs");
    let push_headers = wire.headers(Signal::Logs);
    let tenant = corpus.tenant_ids[0].as_str();
    let header = wire.tenant_header(tenant);
    // signy reads the tenant out of the export, the others out of the header,
    // so exactly one of these two carries it.
    let in_body = header.is_none().then_some(tenant);
    let bodies = Arc::new(Mutex::new(seed_bodies(
        corpus,
        cfg.verify.entries_per_push,
        in_body,
        wire,
    )));
    let start = Instant::now();

    let workers: Vec<_> = (0..cfg.verify.push_connections)
        .map(|_| {
            let bodies = bodies.clone();
            let header = header.clone();
            let address = cfg.push_address().to_string();
            let timeout = cfg.request_timeout();
            tokio::spawn(async move {
                let mut client = Client::new(&address, timeout);
                let mut outcome = SeedOutcome {
                    pushes: 0,
                    rows: 0,
                    line_bytes: 0,
                    wire_bytes: 0,
                    retries: 0,
                    errors: 0,
                    statuses: BTreeMap::new(),
                    first_error: None,
                    elapsed_seconds: 0.0,
                };
                loop {
                    let Some(body) = bodies.lock().await.pop() else {
                        break;
                    };
                    // Seeding is not a measurement, so a refusal is waited out
                    // rather than recorded: the dataset has to land in full on
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
                                if response.status == 204 || response.status == 200 {
                                    outcome.pushes += 1;
                                    outcome.rows += body.rows as u64;
                                    outcome.line_bytes += body.line_bytes as u64;
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

    let mut total = SeedOutcome {
        pushes: 0,
        rows: 0,
        line_bytes: 0,
        wire_bytes: 0,
        retries: 0,
        errors: 0,
        statuses: BTreeMap::new(),
        first_error: None,
        elapsed_seconds: 0.0,
    };
    for worker in workers {
        if let Ok(outcome) = worker.await {
            total.pushes += outcome.pushes;
            total.rows += outcome.rows;
            total.line_bytes += outcome.line_bytes;
            total.wire_bytes += outcome.wire_bytes;
            total.retries += outcome.retries;
            total.errors += outcome.errors;
            for (status, count) in outcome.statuses {
                *total.statuses.entry(status).or_default() += count;
            }
            total.first_error = total.first_error.take().or(outcome.first_error);
        }
    }
    total.elapsed_seconds = start.elapsed().as_secs_f64();
    total
}

/// Query window boundaries are snapped down to a whole `step`.
///
/// Measured, not assumed: Loki puts a metric query's samples on a grid aligned
/// to absolute multiples of `step` and will emit a point past the requested
/// `end` to stay on it, while signy steps from `start`. Over an unaligned
/// window the two therefore report the same rates at different instants, and a
/// row-equality check would fail on the *bed's* choice of window rather than on
/// anything either engine got wrong. Aligning the window makes Loki's alignment
/// a no-op and leaves the two grids identical, so what remains for the check to
/// find is a genuine difference in the values.
fn align_to_step(ns: i64, step_ns: i64) -> i64 {
    if step_ns <= 0 {
        return ns;
    }
    ns - ns.rem_euclid(step_ns)
}

pub struct Query {
    pub id: String,
    pub shape: Shape,
    pub expression: String,
    pub start_ns: i64,
    pub end_ns: i64,
    pub path: String,
    /// The fields the reduced digest compares, which are the fields this query
    /// itself names. A digest is computed by each system alone, so its basis
    /// must be something every system returns for the same row — and the only
    /// fields with that property across schema-on-read and schema-on-write are
    /// the ones the query constrained: a system that returns the row at all
    /// returns the value that satisfied the constraint. Everything else
    /// differs *by design*: VictoriaLogs answers `{app="x"}` with every field
    /// it parsed at ingest, and the other two answer with what the pipeline
    /// produced. A basis of "the whole field set" therefore compares the
    /// storage models, not the answers — that basis reported 0/24 everywhere
    /// and the zeros were its own.
    pub basis_fields: Vec<String>,
    /// The metric bucket width, for converting VictoriaLogs' bucket-start
    /// labels to LogQL's bucket-end evaluation points in the digest.
    pub step_ns: i64,
}

/// `apps x sub-windows` queries per shape, built from the corpus so every one
/// of them selects rows that exist.
///
/// The sub-window is what makes a cold measurement cold: a time range nothing
/// has asked for cannot come out of either system's result cache, and both
/// systems have one.
pub fn build_queries(cfg: &Config, corpus: &Corpus) -> Vec<Query> {
    let apps: Vec<String> = {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for stream in &corpus.streams {
            if let Some(app) = stream.labels.get("app") {
                seen.insert(app.clone());
            }
        }
        if seen.is_empty() {
            seen.insert(APPS[0].to_string());
        }
        seen.into_iter().collect()
    };
    let span = cfg.verify_span_ns();
    let windows = cfg.verify.windows as i64;
    let step_ns = cfg.verify.step_seconds * 1_000_000_000;
    // Rebuilt from the seed rather than scanned out of the corpus, so the
    // matrix phase does not have to hold the rows the seed phase wrote.
    let rare = signy::corpus::rare_field_values(&verify_spec(cfg));
    // Every instant the rare trace occurs at, for the narrow-window shape:
    // a real trace-to-logs query is anchored on an occurrence, not on a grid.
    let rare_times: Vec<i64> = {
        let mut times: Vec<i64> = corpus
            .streams
            .iter()
            .flat_map(|stream| {
                stream.entries.iter().filter_map(|entry| {
                    entry
                        .structured_metadata
                        .iter()
                        .any(|(name, value)| name == "trace_id" && value == &rare.trace_id)
                        .then_some(entry.timestamp_ns)
                })
            })
            .collect();
        times.sort_unstable();
        times
    };
    let mut queries = Vec::new();
    for shape in SHAPES {
        for (app_index, app) in apps.iter().enumerate() {
            for window in 0..cfg.verify.windows {
                let start_ns = align_to_step(
                    cfg.verify.anchor_ns + span * window as i64 / windows,
                    step_ns,
                );
                let end_ns = align_to_step(
                    cfg.verify.anchor_ns + span * (window as i64 + 1) / windows,
                    step_ns,
                );
                // The rate shape never lets a bucket close on the dataset's
                // trailing edge. A LogQL window is `(start, end]` and a LogsQL
                // bucket is `[start, end)`, so a boundary row is excluded by
                // one and included by the other — which cancels exactly when
                // the row at the *other* boundary exists, and the one instant
                // where it does not is the end of the data. Measured: the last
                // window's last bucket read 124.9 against 125.0, one boundary
                // row, everything interior equal. So a rate window that would
                // end where the data does is slid one step in.
                // The trace-window shape replaces the grid window with the
                // occurrence's own: one second either side of the variant-th
                // time the trace appears, which is what a click on a span
                // sends. Unaligned on purpose — nothing metric reads it.
                let variant = app_index * cfg.verify.windows + window;
                let (start_ns, end_ns) = if shape == Shape::TraceWindow && !rare_times.is_empty() {
                    let occurrence = rare_times[variant % rare_times.len()];
                    (
                        occurrence.saturating_sub(1_000_000_000),
                        occurrence.saturating_add(1_000_000_000),
                    )
                } else if shape == Shape::Rate {
                    let dataset_end_ns = align_to_step(cfg.verify.anchor_ns + span, step_ns);
                    if end_ns >= dataset_end_ns {
                        let end = end_ns - step_ns;
                        (start_ns.min(end - step_ns), end)
                    } else {
                        (start_ns, end_ns)
                    }
                } else {
                    (start_ns, end_ns)
                };
                // The corpus's `app`, promoted from the `service.name` the
                // OTLP encoder sends it as.
                let selector = format!("{{service_name=\"{app}\"}}");
                let expression = match shape {
                    // No app selector: a trace is drawn independently of the
                    // app, so pinning one would empty the answer for seven apps
                    // in eight, and the point is a predicate that is the only
                    // selective thing in the query.
                    Shape::JsonFieldRare => {
                        format!(
                            "{{service_name=~\".+\"}} | json | trace_id=\"{}\"",
                            rare.trace_id
                        )
                    }
                    // The same value with no parser stage: the corpus pushes
                    // this `trace_id` as structured metadata as well as writing
                    // it into the line, so the pair separates what the parser
                    // costs from what the storage costs.
                    Shape::MetadataRare | Shape::TraceWindow => {
                        format!("{{service_name=~\".+\"}} | trace_id=\"{}\"", rare.trace_id)
                    }
                    Shape::LabelOnly => selector,
                    Shape::LineFilter => {
                        format!("{selector} |= \"{}\"", PHRASES[variant % PHRASES.len()])
                    }
                    Shape::JsonField => {
                        if variant.is_multiple_of(2) {
                            format!(
                                "{selector} | json | status=\"{}\"",
                                STATUSES[variant % STATUSES.len()]
                            )
                        } else {
                            format!(
                                "{selector} | json | level=\"{}\"",
                                LEVELS[variant % LEVELS.len()]
                            )
                        }
                    }
                    // `sum(rate(...))` rather than a bare `rate(...)`, and the
                    // reason is a measured difference rather than a stylistic
                    // one: Loki promotes structured metadata into a metric's
                    // identity, so a bare `rate()` over this corpus returns one
                    // series per `trace_id` on Loki and one per stream on
                    // signy. That is neither the same amount of work nor a
                    // comparable answer. Summed, both systems have to produce
                    // the same number, which is what the row-equality check is
                    // for. The unsummed difference is reported in
                    // `docs/COMPARISON.md` rather than hidden here.
                    //
                    // The window equals the step. LogsQL has no sliding
                    // window: `stats by (_time:...)` cuts tumbling buckets. A
                    // sliding window degenerates into those buckets exactly
                    // when it is one step wide and the query range is aligned
                    // to it, which the matrix guarantees — any other range
                    // asks a question one of the two languages cannot ask.
                    Shape::Rate => format!("sum(rate({selector}[{}s]))", cfg.verify.step_seconds),
                };
                let path = match cfg.target {
                    // The first-party API: the same six questions as flat
                    // filters. `attr` compiles to the same matcher/field-filter
                    // semantics the LogQL forms had, and the histogram's
                    // `[start, end)` buckets are the tumbling buckets the
                    // LogsQL translation already pinned the rate shape to.
                    Target::Signy => {
                        let mut encoded = url::form_urlencoded::Serializer::new(String::new());
                        match shape {
                            Shape::LabelOnly => {
                                encoded.append_pair("attr", &format!("service_name={app}"));
                            }
                            Shape::LineFilter => {
                                encoded.append_pair("attr", &format!("service_name={app}"));
                                encoded.append_pair("contains", PHRASES[variant % PHRASES.len()]);
                            }
                            Shape::JsonField => {
                                encoded.append_pair("parse", "json");
                                encoded.append_pair("attr", &format!("service_name={app}"));
                                if variant.is_multiple_of(2) {
                                    encoded.append_pair(
                                        "attr",
                                        &format!("status={}", STATUSES[variant % STATUSES.len()]),
                                    );
                                } else {
                                    encoded.append_pair(
                                        "attr",
                                        &format!("level={}", LEVELS[variant % LEVELS.len()]),
                                    );
                                }
                            }
                            Shape::JsonFieldRare => {
                                encoded.append_pair("parse", "json");
                                encoded.append_pair("attr", "service_name=~.+");
                                encoded.append_pair("attr", &format!("trace_id={}", rare.trace_id));
                            }
                            Shape::MetadataRare | Shape::TraceWindow => {
                                encoded.append_pair("attr", "service_name=~.+");
                                encoded.append_pair("attr", &format!("trace_id={}", rare.trace_id));
                            }
                            Shape::Rate => {
                                encoded.append_pair("attr", &format!("service_name={app}"));
                                encoded.append_pair(
                                    "bucket",
                                    &format!("{}s", cfg.verify.step_seconds),
                                );
                            }
                        }
                        encoded.append_pair("start", &ns_to_sample_seconds(start_ns));
                        encoded.append_pair("end", &ns_to_sample_seconds(end_ns));
                        if shape == Shape::Rate {
                            format!("/signy/api/v1/logs/histogram?{}", encoded.finish())
                        } else {
                            encoded.append_pair("limit", &cfg.verify.limit.to_string());
                            encoded.append_pair("direction", "backward");
                            format!("/signy/api/v1/logs?{}", encoded.finish())
                        }
                    }
                    Target::Loki => {
                        // A rate evaluation at `t` covers `(t - step, t]`, so
                        // the point at `t = start` reaches before the window —
                        // rows no `_time` bucket of the same window holds. The
                        // first evaluation point is therefore one step in: the
                        // lookbacks then tile `[start, end)` exactly as the
                        // buckets do.
                        let logql_start_ns = match shape {
                            Shape::Rate => start_ns + step_ns,
                            _ => start_ns,
                        };
                        let encoded = url::form_urlencoded::Serializer::new(String::new())
                            .append_pair("query", &expression)
                            .append_pair("start", &logql_start_ns.to_string())
                            .append_pair("end", &end_ns.to_string())
                            .append_pair("step", &cfg.verify.step_seconds.to_string())
                            .append_pair("limit", &cfg.verify.limit.to_string())
                            .append_pair("direction", "backward")
                            .finish();
                        format!("/loki/api/v1/query_range?{encoded}")
                    }
                    Target::VictoriaLogs => {
                        let encoded = url::form_urlencoded::Serializer::new(String::new())
                            .append_pair("query", &logsql(shape, app, cfg, &rare.trace_id, variant))
                            // Milliseconds, not nanoseconds: `/select/logsql/query`
                            // takes the same `start`/`end` names as the Loki API and
                            // means something different by them.
                            .append_pair("start", &(start_ns / 1_000_000).to_string())
                            .append_pair("end", &(end_ns / 1_000_000).to_string())
                            .finish();
                        format!("/select/logsql/query?{encoded}")
                    }
                    Target::VictoriaMetrics => {
                        unreachable!("main refuses the log phases for victoriametrics")
                    }
                    Target::Mimir => unreachable!("main refuses the log phases for mimir"),
                };
                let basis_fields: Vec<String> = match shape {
                    Shape::LabelOnly | Shape::LineFilter => vec!["service_name".to_string()],
                    Shape::JsonField => {
                        let field = if variant.is_multiple_of(2) {
                            "status"
                        } else {
                            "level"
                        };
                        vec!["service_name".to_string(), field.to_string()]
                    }
                    Shape::JsonFieldRare | Shape::MetadataRare | Shape::TraceWindow => {
                        vec!["service_name".to_string(), "trace_id".to_string()]
                    }
                    // `sum()` strips every label, so the series identity is
                    // empty on both sides and the samples are the basis.
                    Shape::Rate => Vec::new(),
                };
                queries.push(Query {
                    id: format!("{}/{app}/w{window}", shape.name()),
                    shape,
                    expression,
                    start_ns,
                    end_ns,
                    path,
                    basis_fields,
                    step_ns,
                });
            }
        }
    }
    queries
}

/// The same question in LogsQL, and where the two languages do not line up.
///
/// Under OTLP ingest VictoriaLogs stores the body as `_msg` without parsing it
/// (measured, v1.52.0 — its ingest-time JSON parse was a property of its Loki
/// push endpoint, not of the engine), so a field the other two reach through
/// `| json` is reached through `| unpack_json` here: the parser stage is paid
/// at query time on all three systems now, and `metadata_rare` against
/// `json_field_rare` separates the attribute column from the parsed line on
/// this side too.
///
/// Two places the languages' defaults differ and the translation has to take a
/// side, stated so a ratio is not mistaken for a like-for-like:
///
/// * `|=` is a raw substring in LogQL; a bare quoted string in LogsQL is a
///   tokenized phrase. The translation uses `~"..."`, which is the substring —
///   measured equal to `|=` on lines built to straddle token boundaries.
/// * `sum(rate(...))` becomes `stats by (_time:step) rate()`. LogsQL has only
///   tumbling buckets where LogQL has a sliding window, so the matrix pins the
///   window to the step and aligns the range — the one configuration in which
///   both languages are asking the same question. The remaining difference is
///   labeling (bucket start there, evaluation point here), which the digest
///   converts rather than exempts.
fn logsql(shape: Shape, app: &str, cfg: &Config, rare_trace_id: &str, variant: usize) -> String {
    // The dotted resource-attribute name: VictoriaLogs keeps it as sent where
    // the other two promote-and-sanitize, and its LogsQL accepts it unquoted
    // (measured, v1.52.0).
    let selector = format!("service.name:\"{app}\"");
    // `sort by (_time) desc` before every `limit`, because the LogQL side asks
    // `direction=backward`: a bound that binds must cut the *newest* N rows.
    // A bare LogsQL `limit` has no order contract and returns whichever rows
    // it reaches first — measured on this corpus as the window's oldest —
    // so without the sort the two engines answer with disjoint row sets that
    // are both "100 rows from the window".
    let newest = format!("sort by (_time) desc | limit {}", cfg.verify.limit);
    match shape {
        Shape::LabelOnly => format!("{selector} | {newest}"),
        // `~"..."` and not `"..."`. A bare quoted string in LogsQL is a
        // tokenized *phrase* filter, which misses a needle that straddles a
        // token boundary; LogQL's `|=` is a raw substring. Measured on five
        // lines built to straddle: the phrase filter returned two and the
        // regexp filter returned the same three `|=` does, case-sensitivity
        // included.
        Shape::LineFilter => format!(
            "{selector} AND ~\"{}\" | {newest}",
            PHRASES[variant % PHRASES.len()],
        ),
        // `| unpack_json` first, and the reason is what OTLP changed at
        // ingest: a Loki-push JSON line used to be parsed into fields by
        // VictoriaLogs at write time, but an OTLP body is a string it stores
        // as `_msg` unparsed (measured, v1.52.0). The field the query filters
        // on now exists only after a query-time unpack — the same stage the
        // LogQL side pays as `| json`, which makes this pair like-for-like
        // for the first time.
        // `fields (...)` and not a bare `unpack_json`, for a reason the first
        // OTLP run measured: the corpus's JSON lines carry an inner `_msg`
        // field, a bare unpack overwrites VictoriaLogs' own `_msg` column with
        // it, and a row whose inner value is empty comes back without `_msg`
        // at all — which the digest correctly refuses to read as a log row.
        // Unpacking only the queried field asks the same question `| json |
        // field="v"` asks while leaving the message column alone.
        //
        // `keep_original_fields` is LogQL's shadowing rule spelled in LogsQL:
        // on the other side a label that already exists — structured metadata
        // included — wins over what `| json` extracts. Without it, an unpack
        // that finds nothing *erases* the attribute the row already carried,
        // and a logfmt row whose attribute matches the predicate answers on
        // signy and Loki but not here — measured as `json_field_rare`
        // 8/24 with every disagreement a `1 against 0 rows` in the windows
        // where the rare trace's row is not a JSON line.
        Shape::JsonField => {
            let (field, value) = if variant.is_multiple_of(2) {
                ("status", STATUSES[variant % STATUSES.len()].to_string())
            } else {
                ("level", LEVELS[variant % LEVELS.len()].to_string())
            };
            format!(
                "{selector} | unpack_json fields ({field}) keep_original_fields | filter {field}:\"{value}\" | {newest}"
            )
        }
        // No app selector, matching the LogQL side: the point is a predicate
        // that is the only selective thing in the query.
        //
        // The rare pair is now genuinely distinguishable here too. Under Loki
        // push, VictoriaLogs parsed the line at ingest and these two shapes
        // collapsed into the same query; under OTLP the line stays unparsed,
        // so `json_field_rare` pays the unpack stage the way `| json` does and
        // `metadata_rare` reads the attribute column without it. The row sets
        // stay identical because every row carries the attribute and the JSON
        // rows carry the same value inside the line.
        Shape::JsonFieldRare => {
            format!(
                "* | unpack_json fields (trace_id) keep_original_fields | filter trace_id:\"{rare_trace_id}\" | {newest}"
            )
        }
        Shape::MetadataRare | Shape::TraceWindow => {
            format!("trace_id:\"{rare_trace_id}\" | {newest}")
        }
        // `rate()` and not `count()`. LogsQL's `rate()` divides by the bucket
        // width, which is what LogQL's `rate()` does — measured at 0.0833 for
        // five rows in a minute — so the two produce the same units. `count()`
        // produced the bucket total and made the numbers incomparable.
        //
        // The bucket is the matrix step, matching the LogQL side's window;
        // see `build_queries`. VictoriaLogs aligns `_time` buckets to epoch
        // multiples of the width and labels each by its *start*, where LogQL
        // labels a sample by its evaluation point — the bucket's *end*. The
        // digest converts starts to ends with this same step, so the two
        // labelings meet; the values need no conversion, since a bucket that
        // the aligned window fully contains is divided by its full width on
        // both sides.
        Shape::Rate => format!(
            "{selector} | stats by (_time:{}s) rate() as value",
            cfg.verify.step_seconds
        ),
    }
}

/// What a response contained, reduced to something two runs can be compared
/// on.
pub struct Answer {
    pub kind: String,
    pub rows: u64,
    pub series: u64,
    /// Order-independent digest of the **whole** response: every row, the
    /// labels that row carried, and where the response put each of them. Two
    /// systems that returned the same rows in a different order agree here;
    /// two that returned the same lines under different labels do not, which
    /// is the half of the response the `(timestamp, line)` digest this
    /// replaces did not look at (`todo.md`, "Open correctness defects").
    pub digest: String,
    /// The placement-tagged label names inside the digest, as
    /// `stream:`/`entry:`/`metric:` names. Recorded so that a disagreement can
    /// be reported as *which* labels each side had rather than only as two
    /// digests that differ.
    pub label_keys: Vec<String>,
    /// Label names this response carried that the comparison declares out of
    /// scope by name (`DERIVED_LABELS`). Recorded per answer so the exemption
    /// is visible in the document instead of being a silently narrower digest.
    pub dropped_label_keys: Vec<String>,
    /// Whether every series' values follow the direction the query asked for:
    /// `direction=backward` for logs, ascending time for a metric. The digest
    /// is deliberately order-independent, so without this the response order
    /// would be unchecked.
    pub ordered: bool,
    /// `data.stats.summary.totalLinesProcessed`, recorded but **not** digested:
    /// it is a statement about how much each engine had to read, which is the
    /// thing they are supposed to differ on.
    pub lines_processed: Option<u64>,
    /// A digest over the basis all three systems can produce: each row's
    /// nanosecond timestamp plus the values of the fields *the query named*
    /// ([`Query::basis_fields`]), without the message, without placement.
    ///
    /// The full [`Answer::digest`] cannot cross the schema boundary.
    /// VictoriaLogs parses JSON at ingest, so for a JSON row it has the
    /// message's *value* and the fields, not the line the other two return —
    /// there is nothing to compare a line against. And the whole field set
    /// cannot cross it either: schema-on-write returns every field it parsed
    /// where schema-on-read returns what the pipeline produced, so a basis of
    /// "all fields" compares the storage models and always disagrees — it
    /// reported 0/24 on every shape, and the zeros were the checker's own.
    /// What the systems *do* all return for the same row is the row's time and
    /// the fields the query constrained, so that is the basis.
    ///
    /// It is deliberately weaker and the report must say which was compared.
    pub reduced_digest: String,
    pub sample: Vec<String>,
}

/// Labels a system derives for itself rather than being given.
///
/// Loki computes `detected_level` from the line at ingest; nothing in this bed
/// pushes one, so it cannot be a row an engine got wrong and it is dropped
/// from the digest. Dropped rather than ignored: every answer records that it
/// carried the label, and the document states the exemption beside the result.
/// `service_name` used to be exempt for the same reason and no longer can be:
/// the OTLP encoder sends the corpus's `app` as `service.name`, so the label
/// is now pushed data the digest must hold every system to.
///
/// `__stream_shard__` joined the list at the ten-times corpus, and the decision
/// deserves its reasoning rather than its convenience. Loki splits a stream that
/// grows past its shard rate and names the pieces with this label; at 150 k rows
/// no stream was large enough and at 1.5 M rows 32 of 168 answers disagreed on
/// nothing else — identical row counts on both sides, no label missing on the
/// signy side, this one name present on Loki's. So it is `detected_level`'s
/// class exactly: derived by the engine from its own internals, never pushed by
/// this bed, and impossible for either engine to have got *wrong*.
///
/// What the exemption cannot be allowed to hide, stated because a widened basis
/// hides things quietly: this label is part of a **stream's identity**, so
/// dropping it would also hide a real difference in an *unaggregated* metric
/// answer's series set. It is not hidden here for two reasons — the matrix asks
/// for `sum(rate(...))`, whose identity is what the query names rather than what
/// the storage sharded, and every other check the digest runs stays in force
/// beside this drop: the row counts, the label names only one side had, and the
/// answer order. A shard label that arrived with a row-count change would still
/// be a disagreement.
///
/// The alternative was to turn Loki's stream sharding off in
/// `compare/loki-config.yaml` and make the difference not happen. Rejected on
/// that file's own rule — every setting there is a compatibility requirement or
/// a removal of a limit signy does not apply either, and everything that is
/// a tuning choice stays at Loki's default. Sharding is Loki's own tuning, and
/// reaching into it to tidy a table is how a bed starts measuring its author.
const DERIVED_LABELS: [&str; 2] = ["detected_level", "__stream_shard__"];

/// Labels whose *presence* is comparable and whose *wording* is not.
///
/// Loki's `__error_details__` carries its JSON library's internal message
/// ("Value looks like object, but can't find closing '}' symbol"); this
/// engine's carries its own. An engine that fails to attach the label at all
/// was a real disagreement — 16 of 24 `json_field_rare` answers — so the name
/// is digested in place and only the value is replaced. The exemption is by
/// name, stated here, not a silent widening of the basis.
const UNMATCHABLE_VALUE_LABELS: [&str; 1] = ["__error_details__"];
const UNMATCHABLE_VALUE: &str = "<engine-specific wording>";

/// The digest tag for a label, or `None` when it is declared out of scope.
///
/// **No placement is exempt.** There used to be one: pushed structured
/// metadata was digested without its placement, on the grounds that Loki
/// promotes it into stream labels while signy returned it in the third
/// element of each `values` tuple, and that this was a shape difference rather
/// than a wrong answer. It was the same defect as `| json`'s extracted fields —
/// the same slot, the same sentence — and it is fixed rather than declared
/// (`todo.md`, "Open correctness defects"). Loki 3.3.2's default JSON encoding
/// never uses the third element at all; the three-element tuple is its opt-in
/// `categorize-labels` shape and even then it is an object of categories rather
/// than a flat map. So placement is comparable, and a regression back into that
/// slot has to be visible here.
fn tag(name: &str, found_in: &'static str) -> Option<&'static str> {
    if DERIVED_LABELS.contains(&name) {
        return None;
    }
    Some(found_in)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Folds a label object into the digested set, tagged by where it was found.
///
/// A non-string label value is an error rather than an empty string: the
/// previous digest read label values with `unwrap_or_default`, so a response
/// that answered `{"level": null}` compared equal to one that answered
/// `{"level": ""}`.
fn collect_labels(
    value: &Value,
    found_in: &'static str,
    digested: &mut BTreeMap<String, String>,
    dropped: &mut BTreeSet<String>,
) -> Result<(), String> {
    let map = value
        .as_object()
        .ok_or_else(|| format!("a result's {found_in} labels are not an object"))?;
    for (name, value) in map {
        let text = value
            .as_str()
            .ok_or_else(|| format!("{found_in} label '{name}' is not a string: {value}"))?;
        match tag(name, found_in) {
            Some(tag) => {
                let value = if UNMATCHABLE_VALUE_LABELS.contains(&name.as_str()) {
                    UNMATCHABLE_VALUE.to_string()
                } else {
                    text.to_string()
                };
                digested.insert(format!("{tag}:{name}"), value);
            }
            None => {
                dropped.insert(format!("{found_in}:{name}"));
            }
        }
    }
    Ok(())
}

/// The same map with `stream:`/`entry:`/`metric:` dropped from the names.
///
/// Placement is exactly what the strict digest exists to check, and exactly
/// what a system with a different storage model cannot be held to.
fn strip_placement(digested: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    digested
        .iter()
        .map(|(name, value)| {
            let bare = name.split_once(':').map_or(name.as_str(), |(_, rest)| rest);
            (bare.to_string(), value.clone())
        })
        .collect()
}

fn canonical_labels(digested: &BTreeMap<String, String>) -> String {
    digested
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// The reduced basis's field set: of everything the row carried, only the
/// fields the query named. See [`Query::basis_fields`] for why the basis is
/// the query's fields and not the row's.
fn basis_projection(fields: &BTreeMap<String, String>, basis: &[String]) -> String {
    let mut parts: Vec<String> = basis
        .iter()
        .filter_map(|name| fields.get(name).map(|value| format!("{name}={value}")))
        .collect();
    parts.sort();
    parts.dedup();
    parts.join(",")
}

/// A LogsQL `_time` (RFC 3339, fractional seconds truncated) as Unix
/// nanoseconds — the log-record timestamp encoding the Loki side already uses.
fn rfc3339_to_ns(text: &str) -> Result<i64, String> {
    chrono::DateTime::parse_from_rfc3339(text)
        .map_err(|error| format!("a _time value is not RFC 3339: '{text}': {error}"))?
        .timestamp_nanos_opt()
        .ok_or_else(|| format!("a _time value does not fit in nanoseconds: '{text}'"))
}

/// Nanoseconds as the six-decimal seconds string `canonical_sample` produces,
/// without a float division that would wobble in the last microsecond.
pub(crate) fn ns_to_sample_seconds(ns: i64) -> String {
    format!(
        "{}.{:06}",
        ns.div_euclid(1_000_000_000),
        ns.rem_euclid(1_000_000_000) / 1_000
    )
}

/// Metric samples are compared at six decimals.
///
/// Both systems compute `rate()` in f64 and print it themselves, so the last
/// bits of the two decimal renderings are not a property either engine
/// promises. Six places is far finer than any difference that would matter and
/// far coarser than the printing.
fn canonical_sample(value: &Value) -> String {
    match value {
        Value::String(text) => text
            .parse::<f64>()
            .map(|number| format!("{number:.6}"))
            .unwrap_or_else(|_| text.clone()),
        Value::Number(number) => number
            .as_f64()
            .map(|number| format!("{number:.6}"))
            .unwrap_or_else(|| number.to_string()),
        other => other.to_string(),
    }
}

/// One digest record per row, plus one per series identity for a metric result.
///
/// A log response's grouping into streams is *not* a record. Both systems now
/// return one stream per distinct structured-metadata combination — thousands
/// of them over this corpus — but neither promises a particular partition of the
/// same rows, and a digest over the partition would fail on a difference nobody
/// could act on. What must agree is which labels each entry carried, which is
/// why the labels are digested per row. A metric response's grouping *is* its
/// identity, so there each series contributes a record of its own and an extra
/// empty series cannot hide.
fn stream_records(
    entries: &[Value],
    state: &mut DigestState,
    basis: &[String],
) -> Result<(), String> {
    for entry in entries {
        let mut stream_labels = BTreeMap::new();
        collect_labels(
            &entry["stream"],
            "stream",
            &mut stream_labels,
            &mut state.dropped,
        )?;
        let mut previous: Option<i64> = None;
        for value in values_of(entry)? {
            let pair = value
                .as_array()
                .ok_or_else(|| "a values element is not an array".to_string())?;
            if !(pair.len() == 2 || pair.len() == 3) {
                return Err(format!(
                    "a log values element has {} elements, expected [timestamp, line] or \
[timestamp, line, metadata]",
                    pair.len()
                ));
            }
            let timestamp = pair[0]
                .as_str()
                .filter(|text| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()))
                .ok_or_else(|| {
                    format!("a log timestamp is not a nanosecond string: {}", pair[0])
                })?;
            let line = pair[1]
                .as_str()
                .ok_or_else(|| format!("a log line is not a string: {}", pair[1]))?;
            let mut attributes = stream_labels.clone();
            if let Some(metadata) = pair.get(2) {
                collect_labels(metadata, "entry", &mut attributes, &mut state.dropped)?;
            }
            let parsed: i64 = timestamp
                .parse()
                .map_err(|_| format!("log timestamp '{timestamp}' does not fit in i64"))?;
            // `direction=backward`, so within a stream time must not increase.
            if previous.is_some_and(|previous| parsed > previous) {
                state.ordered = false;
            }
            previous = Some(parsed);
            state.label_keys.extend(attributes.keys().cloned());
            state.records.push(format!(
                "{}\u{1}{timestamp}\u{1}{line}",
                canonical_labels(&attributes)
            ));
            state.reduced.push(format!(
                "{parsed}\u{1}{}",
                basis_projection(&strip_placement(&attributes), basis)
            ));
            state.rows += 1;
        }
    }
    Ok(())
}

fn metric_records(
    entries: &[Value],
    state: &mut DigestState,
    basis: &[String],
) -> Result<(), String> {
    for entry in entries {
        let mut labels = BTreeMap::new();
        collect_labels(&entry["metric"], "metric", &mut labels, &mut state.dropped)?;
        let identity = canonical_labels(&labels);
        let reduced_identity = basis_projection(&strip_placement(&labels), basis);
        state.label_keys.extend(labels.keys().cloned());
        state.records.push(format!("series\u{1}{identity}"));
        state.reduced.push(format!("series\u{1}{reduced_identity}"));
        let mut previous: Option<f64> = None;
        for value in values_of(entry)? {
            let pair = value
                .as_array()
                .ok_or_else(|| "a values element is not an array".to_string())?;
            if pair.len() != 2 {
                return Err(format!(
                    "a metric values element has {} elements, expected [timestamp, value]",
                    pair.len()
                ));
            }
            let at = pair[0]
                .as_f64()
                .or_else(|| pair[0].as_str().and_then(|text| text.parse().ok()))
                .ok_or_else(|| format!("a metric timestamp is not a number: {}", pair[0]))?;
            let numeric = pair[1].as_f64().is_some()
                || pair[1]
                    .as_str()
                    .is_some_and(|text| text.parse::<f64>().is_ok());
            if !numeric {
                return Err(format!("a metric sample is not a number: {}", pair[1]));
            }
            // A metric range answers in ascending time on both systems.
            if previous.is_some_and(|previous| at < previous) {
                state.ordered = false;
            }
            previous = Some(at);
            state.records.push(format!(
                "{identity}\u{1}{}\u{1}{}",
                canonical_sample(&pair[0]),
                canonical_sample(&pair[1]),
            ));
            state.reduced.push(format!(
                "{reduced_identity}\u{1}{}\u{1}{}",
                canonical_sample(&pair[0]),
                canonical_sample(&pair[1]),
            ));
            state.rows += 1;
        }
    }
    Ok(())
}

/// `query_range` answers with `values`; the instant form answers with a single
/// `value`. Neither is allowed to be absent — the previous digest skipped a
/// result whose values it could not read, so a response that carried none at
/// all digested as an empty agreement.
fn values_of(entry: &Value) -> Result<Vec<Value>, String> {
    if let Some(values) = entry["values"].as_array() {
        return Ok(values.clone());
    }
    if entry["value"].is_array() {
        return Ok(vec![entry["value"].clone()]);
    }
    Err("a result carries neither a values array nor a value pair".to_string())
}

/// VictoriaLogs' answer: newline-delimited JSON objects, one per row.
///
/// No envelope, no `resultType`, no `data.stats` — so `kind` is synthesized and
/// `lines_processed` is `None`, which the report prints as "not reported"
/// rather than as zero.
///
/// Only the shared basis is available. There is no line to compare: a JSON row
/// was parsed at ingest, so `_msg` holds the message's value and the fields are
/// separate, where the other two return the line they were given. So the strict
/// digest is set equal to the reduced one and the report must not compare it
/// against a strict digest from elsewhere.
pub fn digest_for(target: Target, body: &[u8], query: &Query) -> Result<Answer, String> {
    match target {
        Target::Signy => digest_first_party_response(body, &query.basis_fields),
        Target::Loki => digest_response(body, &query.basis_fields),
        Target::VictoriaLogs => digest_logsql_response(body, &query.basis_fields, query.step_ns),
        Target::VictoriaMetrics => Err(format!(
            "target {} answers the metric matrix, not the log one",
            target.name()
        )),
        Target::Mimir => Err(format!(
            "target {} answers the metric matrix, not the log one",
            target.name()
        )),
    }
}

/// signy's first-party answer: NDJSON — log rows
/// (`timestamp`/`line`/`attributes`) from `/logs`, or dense buckets
/// (`bucket_start`/`bucket_end`/`count`) from `/logs/histogram`.
///
/// Like the LogsQL reader, the strict digest is set equal to the reduced one:
/// the full-response digest lost its cross-target meaning when the response
/// schema stopped being Loki's, and the reduced basis is the comparison that
/// makes the timing tables citable. A histogram bucket becomes the sample the
/// other systems label the same instant with: the count divided by the bucket
/// width in seconds (the rate), timestamped at the bucket's end.
pub fn digest_first_party_response(body: &[u8], basis: &[String]) -> Result<Answer, String> {
    let text =
        std::str::from_utf8(body).map_err(|error| format!("response is not UTF-8: {error}"))?;
    let mut state = DigestState {
        ordered: true,
        ..DigestState::default()
    };
    let mut label_keys: BTreeSet<String> = BTreeSet::new();
    let mut series_seen: BTreeSet<String> = BTreeSet::new();
    let mut is_metric = false;
    let mut previous: Option<i64> = None;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let row: Value = serde_json::from_str(line)
            .map_err(|error| format!("response line is not JSON: {error}"))?;
        let object = row
            .as_object()
            .ok_or_else(|| "a response line is not a JSON object".to_string())?;
        if object.contains_key("bucket_start") {
            is_metric = true;
            let bucket_end: i64 = object
                .get("bucket_end")
                .and_then(Value::as_str)
                .and_then(|text| text.parse().ok())
                .ok_or_else(|| "a bucket has no nanosecond bucket_end string".to_string())?;
            let bucket_start: i64 = object
                .get("bucket_start")
                .and_then(Value::as_str)
                .and_then(|text| text.parse().ok())
                .ok_or_else(|| "a bucket has no nanosecond bucket_start string".to_string())?;
            let count = object
                .get("count")
                .and_then(Value::as_u64)
                .ok_or_else(|| "a bucket has no integer count".to_string())?;
            let width_seconds = (bucket_end - bucket_start) as f64 / 1_000_000_000.0;
            let rate = Value::from(count as f64 / width_seconds);
            let identity = basis_projection(&BTreeMap::new(), basis);
            if series_seen.insert(identity.clone()) {
                state.reduced.push(format!("series\u{1}{identity}"));
            }
            state.reduced.push(format!(
                "{identity}\u{1}{}\u{1}{}",
                ns_to_sample_seconds(bucket_end),
                canonical_sample(&rate),
            ));
            state.rows += 1;
            continue;
        }
        let timestamp: i64 = object
            .get("timestamp")
            .and_then(Value::as_str)
            .and_then(|text| text.parse().ok())
            .ok_or_else(|| "a log row has no nanosecond timestamp string".to_string())?;
        // `direction=backward`, so time must not increase across the answer.
        if previous.is_some_and(|previous| timestamp > previous) {
            state.ordered = false;
        }
        previous = Some(timestamp);
        let mut fields: BTreeMap<String, String> = BTreeMap::new();
        if let Some(attributes) = object.get("attributes").and_then(Value::as_object) {
            for (name, value) in attributes {
                let text = match value {
                    Value::String(text) => text.clone(),
                    other => other.to_string(),
                };
                let text = if UNMATCHABLE_VALUE_LABELS.contains(&name.as_str()) {
                    UNMATCHABLE_VALUE.to_string()
                } else {
                    text
                };
                label_keys.insert(name.clone());
                fields.insert(name.clone(), text);
            }
        }
        state.reduced.push(format!(
            "{timestamp}\u{1}{}",
            basis_projection(&fields, basis)
        ));
        state.rows += 1;
    }
    let reduced = reduced_digest(&mut state);
    Ok(Answer {
        kind: if is_metric { "matrix" } else { "streams" }.to_string(),
        rows: state.rows,
        series: if is_metric {
            series_seen.len() as u64
        } else {
            state.rows
        },
        digest: reduced.clone(),
        label_keys: label_keys.into_iter().collect(),
        dropped_label_keys: Vec::new(),
        ordered: state.ordered,
        lines_processed: None,
        reduced_digest: reduced,
        sample: state.reduced.iter().take(3).cloned().collect(),
    })
}

pub fn digest_logsql_response(
    body: &[u8],
    basis: &[String],
    step_ns: i64,
) -> Result<Answer, String> {
    let text =
        std::str::from_utf8(body).map_err(|error| format!("response is not UTF-8: {error}"))?;
    let mut state = DigestState {
        ordered: true,
        ..DigestState::default()
    };
    let mut label_keys: BTreeSet<String> = BTreeSet::new();
    let mut series_seen: BTreeSet<String> = BTreeSet::new();
    let mut is_metric = false;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let row: Value = serde_json::from_str(line)
            .map_err(|error| format!("response line is not JSON: {error}"))?;
        let object = row
            .as_object()
            .ok_or_else(|| "a response line is not a JSON object".to_string())?;
        let mut fields: BTreeMap<String, String> = BTreeMap::new();
        let mut time_text = String::new();
        let mut has_msg = false;
        for (name, value) in object {
            match name.as_str() {
                // `_stream` and `_stream_id` are VictoriaLogs' own rendering of
                // the label set it already returns field by field, so digesting
                // them would count the same information twice.
                "_stream" | "_stream_id" => {}
                "_time" => time_text = value.as_str().unwrap_or_default().to_string(),
                // The line. The reduced basis has no line — that is the schema
                // boundary the basis exists to cross — so it must not reappear
                // as a field. Its *presence* still matters below: a log row
                // carries `_msg` and a `stats` row does not.
                "_msg" => has_msg = true,
                _ => {
                    let text = match value {
                        Value::String(text) => text.clone(),
                        other => other.to_string(),
                    };
                    let text = if UNMATCHABLE_VALUE_LABELS.contains(&name.as_str()) {
                        UNMATCHABLE_VALUE.to_string()
                    } else {
                        text
                    };
                    // The canonical key form. VictoriaLogs returns a resource
                    // attribute under its dotted name where the other two
                    // promote-and-sanitize, so `service.name` here and
                    // `service_name` there are the same key — leaving both
                    // spellings in play would make the reduced digest disagree
                    // over the checker's own naming, not the rows.
                    let name = crate::otlp::sanitize_key(name);
                    label_keys.insert(name.clone());
                    fields.insert(name, text);
                }
            }
        }
        let time_ns = rfc3339_to_ns(&time_text)?;
        if has_msg {
            state.reduced.push(format!(
                "{time_ns}\u{1}{}",
                basis_projection(&fields, basis)
            ));
        } else {
            // A `stats by (_time:step)` row. Its `_time` is the bucket's
            // *start*, aligned to epoch multiples of the width; LogQL labels
            // the same bucket by its evaluation point — the *end*. One step
            // converts the one labeling into the other; the value needs no
            // conversion because an aligned window contains whole buckets and
            // both languages divide by the full width.
            is_metric = true;
            let value = object
                .get("value")
                .ok_or_else(|| "a stats row has no `value` field".to_string())?;
            if canonical_sample(value).parse::<f64>().is_err() {
                return Err(format!("a stats row's value is not a number: {value}"));
            }
            fields.remove("value");
            let identity = basis_projection(&fields, basis);
            if series_seen.insert(identity.clone()) {
                state.reduced.push(format!("series\u{1}{identity}"));
            }
            state.reduced.push(format!(
                "{identity}\u{1}{}\u{1}{}",
                ns_to_sample_seconds(time_ns + step_ns),
                canonical_sample(value),
            ));
        }
        state.rows += 1;
    }
    let reduced = reduced_digest(&mut state);
    Ok(Answer {
        kind: if is_metric { "matrix" } else { "streams" }.to_string(),
        rows: state.rows,
        series: if is_metric {
            series_seen.len() as u64
        } else {
            state.rows
        },
        digest: reduced.clone(),
        label_keys: label_keys.into_iter().collect(),
        dropped_label_keys: Vec::new(),
        // LogsQL returns rows without an ordering contract the way `direction`
        // asks for one, so this is not checked rather than being asserted true.
        ordered: true,
        lines_processed: None,
        reduced_digest: reduced,
        sample: state.reduced.iter().take(3).cloned().collect(),
    })
}

/// Hashes the shared basis, sorted so it is order-independent like the strict
/// one. No envelope: a `streams` answer and a `matrix` answer are already
/// distinguished by their records here, and VictoriaLogs has no envelope to
/// contribute.
fn reduced_digest(state: &mut DigestState) -> String {
    state.reduced.sort();
    let mut hash = fnv1a64(b"reduced");
    for record in &state.reduced {
        hash ^= fnv1a64(record.as_bytes());
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[derive(Default)]
struct DigestState {
    records: Vec<String>,
    /// The same rows on the basis all three systems share: nanosecond
    /// timestamp plus the query-named fields, message and placement excluded.
    reduced: Vec<String>,
    label_keys: BTreeSet<String>,
    dropped: BTreeSet<String>,
    rows: u64,
    ordered: bool,
}

pub fn digest_response(body: &[u8], basis: &[String]) -> Result<Answer, String> {
    let parsed: Value =
        serde_json::from_slice(body).map_err(|error| format!("response is not JSON: {error}"))?;
    let kind = parsed["data"]["resultType"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let status = parsed["status"].as_str().unwrap_or("missing").to_string();
    let entries = parsed["data"]["result"]
        .as_array()
        .ok_or_else(|| "response has no data.result array".to_string())?;

    let mut state = DigestState {
        ordered: true,
        ..DigestState::default()
    };
    if kind == "matrix" || kind == "vector" {
        metric_records(entries, &mut state, basis)?;
    } else {
        stream_records(entries, &mut state, basis)?;
    }
    state.records.sort();
    // The envelope goes in before the rows and outside the sort: a `streams`
    // answer and a `matrix` answer are not the same answer even if their
    // records happened to coincide.
    let mut hash = fnv1a64(format!("{status}\u{1}{kind}").as_bytes());
    for record in &state.records {
        hash ^= fnv1a64(record.as_bytes());
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let reduced = reduced_digest(&mut state);
    let sample = state
        .records
        .iter()
        .take(3)
        .map(|record| record.chars().take(140).collect::<String>())
        .collect();
    Ok(Answer {
        kind,
        rows: state.rows,
        series: entries.len() as u64,
        digest: format!("{hash:016x}"),
        label_keys: state.label_keys.into_iter().collect(),
        dropped_label_keys: state.dropped.into_iter().collect(),
        ordered: state.ordered,
        lines_processed: parsed["data"]["stats"]["summary"]["totalLinesProcessed"].as_u64(),
        reduced_digest: reduced,
        sample,
    })
}

struct Timing {
    cold_ms: f64,
    warm_ms: Vec<f64>,
    answer: Option<Answer>,
    warm_digests_agree: bool,
    status: u16,
    error: Option<String>,
}

async fn issue(
    client: &mut Client,
    query: &Query,
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

/// Cold pass over every query, then the warm repeats.
///
/// The passes are separated rather than interleaved so that "cold" means what
/// it says: by the time a query is repeated, every other query of every shape
/// has already run, so nothing about the second issue is the first one still
/// being resident in a way the first was not.
pub async fn run_matrix(cfg: &Config, corpus: &Corpus) -> Value {
    let (header, value) = cfg.target.read_tenant_header(corpus.tenant_ids[0].as_str());
    let tenant = (header, value.as_str());
    let queries = build_queries(cfg, corpus);
    let mut client = Client::new(&cfg.http_address, cfg.request_timeout());
    let mut timings: Vec<Timing> = Vec::with_capacity(queries.len());

    for query in &queries {
        let (elapsed, status, body, error) = issue(&mut client, query, tenant).await;
        let answer = if status == 200 {
            match digest_for(cfg.target, &body, query) {
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

    for _ in 0..cfg.verify.repeats {
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
            // A system whose repeat of the same query over the same fixed
            // window returns a different answer has a cache or a scan bound
            // that is not deterministic, and the cold/warm split would be
            // measuring that rather than caching.
            match (digest_for(cfg.target, &body, query), timing.answer.as_ref()) {
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
    for shape in SHAPES {
        let mut cold = Series::default();
        let mut warm = Series::default();
        let mut rows = 0u64;
        let mut errors = 0u64;
        let mut unstable = 0u64;
        let mut out_of_order = 0u64;
        let mut example = String::new();
        for (query, timing) in queries.iter().zip(&timings) {
            if query.shape != shape {
                continue;
            }
            if example.is_empty() {
                example = query.expression.clone();
            }
            if timing.error.is_some() || timing.answer.is_none() {
                errors += 1;
                continue;
            }
            cold.push(timing.cold_ms);
            for value in &timing.warm_ms {
                warm.push(*value);
            }
            rows += timing.answer.as_ref().map_or(0, |answer| answer.rows);
            unstable += u64::from(!timing.warm_digests_agree);
            out_of_order += u64::from(timing.answer.as_ref().is_some_and(|answer| !answer.ordered));
        }
        per_shape.insert(
            shape.name().to_string(),
            json!({
                "expression_example": example,
                "queries": queries.iter().filter(|query| query.shape == shape).count(),
                "errors": errors,
                "rows_returned_cold": rows,
                "warm_answers_differed": unstable,
                "answers_out_of_requested_order": out_of_order,
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
                "expression": query.expression,
                "start_ns": query.start_ns,
                "end_ns": query.end_ns,
                "status": timing.status,
                "error": timing.error,
                "cold_ms": timing.cold_ms,
                "warm_ms": timing.warm_ms,
                "warm_digests_agree": timing.warm_digests_agree,
                "result_type": timing.answer.as_ref().map(|answer| answer.kind.clone()),
                "rows": timing.answer.as_ref().map(|answer| answer.rows),
                "series": timing.answer.as_ref().map(|answer| answer.series),
                "digest": timing.answer.as_ref().map(|answer| answer.digest.clone()),
                // The basis all three share. Compared across systems that do not
                // agree on what a stored row is; the strict `digest` above is
                // only comparable between the two that keep the line.
                "reduced_digest": timing
                    .answer
                    .as_ref()
                    .map(|answer| answer.reduced_digest.clone()),
                "label_keys": timing.answer.as_ref().map(|answer| answer.label_keys.clone()),
                "dropped_label_keys": timing
                    .answer
                    .as_ref()
                    .map(|answer| answer.dropped_label_keys.clone()),
                "ordered": timing.answer.as_ref().map(|answer| answer.ordered),
                "lines_processed": timing.answer.as_ref().and_then(|answer| answer.lines_processed),
                "sample": timing.answer.as_ref().map(|answer| answer.sample.clone()),
            })
        })
        .collect();

    json!({
        "shapes": Value::Object(per_shape),
        "answers": answers,
        "queries_issued": queries.len() as u64 * (1 + cfg.verify.repeats as u64),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_row_digest_ignores_stream_grouping_and_order_but_not_content() {
        let first = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a"},"values":[["2","beta"],["1","alpha"]]}]}}"#;
        let regrouped = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a"},"values":[["1","alpha"]]},
            {"stream":{"app":"a"},"values":[["2","beta"]]}]}}"#;
        let changed_line = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a"},"values":[["1","alpha"],["2","BETA"]]}]}}"#;

        let first = digest_response(first, &[]).expect("valid");
        let regrouped = digest_response(regrouped, &[]).expect("valid");
        let changed_line = digest_response(changed_line, &[]).expect("valid");
        assert_eq!(
            first.digest, regrouped.digest,
            "the same rows under the same labels, split into more streams, are the same answer"
        );
        assert_ne!(
            first.digest, changed_line.digest,
            "a changed line is a changed row"
        );
        assert_eq!(first.rows, 2);
        assert_eq!(first.label_keys, vec!["stream:app".to_string()]);
    }

    /// The blind spot this digest was extended to close: the finding in
    /// `todo.md` was reported as 24/24 agreed because the labels were outside
    /// the digest.
    #[test]
    fn the_same_lines_under_different_labels_are_not_the_same_answer() {
        let one_stream = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a"},"values":[["1","alpha"],["2","beta"]]}]}}"#;
        let mislabelled = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a"},"values":[["1","alpha"]]},
            {"stream":{"app":"b"},"values":[["2","beta"]]}]}}"#;
        let extra_label = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a","level":"info"},"values":[["1","alpha"],["2","beta"]]}]}}"#;
        let other_value = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"b"},"values":[["1","alpha"],["2","beta"]]}]}}"#;

        let one_stream = digest_response(one_stream, &[]).expect("valid");
        for (name, body) in [
            ("a line under the wrong stream", mislabelled.as_slice()),
            ("an extra label on every row", extra_label.as_slice()),
            ("a different label value", other_value.as_slice()),
        ] {
            assert_ne!(
                one_stream.digest,
                digest_response(body, &[]).expect("valid").digest,
                "{name} must not digest equal"
            );
        }
    }

    /// `__error_details__` carries each engine's own wording, so two answers
    /// differing only in that text are the same answer — while an answer
    /// missing the label entirely is not, which is what 16 of 24
    /// `json_field_rare` responses were.
    #[test]
    fn error_details_wording_is_exempt_and_its_absence_is_not() {
        let loki_wording = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a","__error__":"JSONParserErr",
              "__error_details__":"Value looks like object, but can't find closing '}' symbol"},
             "values":[["1","alpha"]]}]}}"#;
        let our_wording = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a","__error__":"JSONParserErr",
              "__error_details__":"line is not valid JSON"},
             "values":[["1","alpha"]]}]}}"#;
        let missing = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a","__error__":"JSONParserErr"},
             "values":[["1","alpha"]]}]}}"#;

        let loki_wording = digest_response(loki_wording, &[]).expect("valid");
        let our_wording = digest_response(our_wording, &[]).expect("valid");
        let missing = digest_response(missing, &[]).expect("valid");
        assert_eq!(loki_wording.digest, our_wording.digest);
        assert_ne!(loki_wording.digest, missing.digest);
    }

    /// The reduced basis is a *common* basis: the same logical answer in the
    /// Loki shape and in the LogsQL shape must produce the same digest. The
    /// previous basis could not — the LogsQL side put `_msg` into the field
    /// set and encoded timestamps as RFC 3339 — so 0/24 everywhere was the
    /// checker disagreeing with itself, not the engines disagreeing.
    #[test]
    fn the_same_log_rows_in_both_response_shapes_reduce_to_the_same_digest() {
        let basis = vec!["app".to_string(), "trace_id".to_string()];
        // 1_700_000_000.123 s = 2023-11-14T22:13:20.123Z.
        let loki = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a","trace_id":"t1","env":"prod","pod_ip":"10.0.0.1"},
             "values":[["1700000000123000000","{\"level\":\"info\",\"_msg\":\"boom\"}"]]}]}}"#;
        // Schema-on-write: the line is gone, its parsed fields and more are
        // top-level, `_stream`/`_stream_id` render the label set again.
        let logsql = br#"{"_time":"2023-11-14T22:13:20.123Z","_msg":"boom","app":"a","trace_id":"t1","level":"info","status":200,"_stream":"{app=\"a\"}","_stream_id":"s1"}"#;
        let wrong_value = br#"{"_time":"2023-11-14T22:13:20.123Z","_msg":"boom","app":"a","trace_id":"t2","level":"info","_stream":"{app=\"a\"}"}"#;

        let loki = digest_response(loki, &basis).expect("valid");
        let logsql = digest_logsql_response(logsql, &basis, 0).expect("valid");
        let wrong_value = digest_logsql_response(wrong_value, &basis, 0).expect("valid");
        assert_eq!(loki.reduced_digest, logsql.reduced_digest);
        assert_ne!(loki.reduced_digest, wrong_value.reduced_digest);
        assert_ne!(
            loki.reduced_digest,
            digest_logsql_response(b"", &basis, 0)
                .expect("valid")
                .reduced_digest,
            "a missing row must not reduce equal"
        );
    }

    /// The promoted-name boundary: signy and Loki answer under the
    /// sanitized `service_name` while VictoriaLogs keeps the dotted
    /// `service.name` it was sent. The reduced digest canonicalizes the key,
    /// so the same row agrees — and a genuinely different value still does
    /// not.
    #[test]
    fn a_dotted_field_name_and_its_promoted_form_are_the_same_reduced_key() {
        let basis = vec!["service_name".to_string()];
        let loki = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"service_name":"api-gateway"},
             "values":[["1700000000123000000","boom"]]}]}}"#;
        let logsql = br#"{"_time":"2023-11-14T22:13:20.123Z","_msg":"boom","service.name":"api-gateway","_stream":"{}"}"#;
        let other_app = br#"{"_time":"2023-11-14T22:13:20.123Z","_msg":"boom","service.name":"checkout","_stream":"{}"}"#;

        let loki = digest_response(loki, &basis).expect("valid");
        let logsql = digest_logsql_response(logsql, &basis, 0).expect("valid");
        let other_app = digest_logsql_response(other_app, &basis, 0).expect("valid");
        assert_eq!(loki.reduced_digest, logsql.reduced_digest);
        assert_ne!(loki.reduced_digest, other_app.reduced_digest);
    }

    /// A metric answer crosses too: LogsQL labels a bucket by its start,
    /// LogQL by its evaluation point — the end — and the digest converts with
    /// the query's step. Values meet at six decimals.
    #[test]
    fn the_same_metric_answer_in_both_response_shapes_reduces_to_the_same_digest() {
        let step_ns = 10_000_000_000;
        // Buckets [22:13:20, 22:13:30) and [22:13:30, 22:13:40): LogQL samples
        // at 1_700_000_010 and 1_700_000_020, LogsQL rows at the starts.
        let loki = br#"{"data":{"resultType":"matrix","result":[
            {"metric":{},"values":[[1700000010,"20.833333333333"],[1700000020,"41.666666666666"]]}]}}"#;
        let logsql = concat!(
            r#"{"_time":"2023-11-14T22:13:20Z","value":20.833333333333332}"#,
            "\n",
            r#"{"_time":"2023-11-14T22:13:30Z","value":41.666666666666664}"#,
        )
        .as_bytes();
        let wrong_value = concat!(
            r#"{"_time":"2023-11-14T22:13:20Z","value":20.833333333333332}"#,
            "\n",
            r#"{"_time":"2023-11-14T22:13:30Z","value":41.7}"#,
        )
        .as_bytes();

        let loki = digest_response(loki, &[]).expect("valid");
        let logsql = digest_logsql_response(logsql, &[], step_ns).expect("valid");
        let wrong_value = digest_logsql_response(wrong_value, &[], step_ns).expect("valid");
        assert_eq!(loki.kind, "matrix");
        assert_eq!(logsql.kind, "matrix");
        assert_eq!(loki.reduced_digest, logsql.reduced_digest);
        assert_ne!(loki.reduced_digest, wrong_value.reduced_digest);
    }

    /// Pushed structured metadata used to be exempt from placement here, which
    /// declared as a shape difference the same defect `| json`'s extracted
    /// fields were open as. It is not exempt now, so a regression back into the
    /// `values` triple is a disagreement — which is the only way the digest can
    /// hold the fix.
    #[test]
    fn structured_metadata_in_the_values_triple_is_a_disagreement() {
        let promoted = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a","trace_id":"t1","pod_ip":"10.0.0.1","detected_level":"info",
              "service_name":"a"},"values":[["1","alpha"]]}]}}"#;
        let in_the_triple = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a"},
             "values":[["1","alpha",{"trace_id":"t1","pod_ip":"10.0.0.1"}]]}]}}"#;
        let wrong_trace = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a","trace_id":"t2","pod_ip":"10.0.0.1"},
             "values":[["1","alpha"]]}]}}"#;

        let promoted = digest_response(promoted, &[]).expect("valid");
        let in_the_triple = digest_response(in_the_triple, &[]).expect("valid");
        assert_ne!(promoted.digest, in_the_triple.digest);
        assert_eq!(
            in_the_triple.label_keys,
            vec![
                "entry:pod_ip".to_string(),
                "entry:trace_id".to_string(),
                "stream:app".to_string()
            ]
        );
        assert_ne!(
            promoted.digest,
            digest_response(wrong_trace, &[]).expect("valid").digest,
            "the metadata values are still part of the row"
        );
        assert_eq!(
            promoted.dropped_label_keys,
            vec!["stream:detected_level".to_string()],
            "an exempted label is recorded as dropped, not silently ignored"
        );
        assert!(in_the_triple.dropped_label_keys.is_empty());
    }

    /// The defect this digest was red on: `| json`'s extracted fields reach the
    /// stream labels on Loki and reached the entry's metadata object on
    /// signy. Not the same response, and a Logs panel renders the two
    /// differently. The promoted side is now what signy answers too, so
    /// this is also the fix's regression guard.
    #[test]
    fn json_extracted_fields_in_the_wrong_place_are_a_disagreement() {
        let promoted = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a","level":"info","status":"200","trace_id":"t1"},
             "values":[["1","{\"level\":\"info\"}"]]}]}}"#;
        let in_the_triple = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a"},
             "values":[["1","{\"level\":\"info\"}",
                        {"level":"info","status":"200","trace_id":"t1"}]]}]}}"#;

        let promoted = digest_response(promoted, &[]).expect("valid");
        let in_the_triple = digest_response(in_the_triple, &[]).expect("valid");
        assert_ne!(promoted.digest, in_the_triple.digest);
        assert_eq!(
            promoted.label_keys,
            vec![
                "stream:app".to_string(),
                "stream:level".to_string(),
                "stream:status".to_string(),
                "stream:trace_id".to_string()
            ]
        );
        assert_eq!(
            in_the_triple.label_keys,
            vec![
                "entry:level".to_string(),
                "entry:status".to_string(),
                "entry:trace_id".to_string(),
                "stream:app".to_string()
            ],
            "the report can name which labels each side had, and where"
        );
    }

    #[test]
    fn matrix_samples_compare_at_six_decimals() {
        let loki = br#"{"data":{"resultType":"matrix","result":[
            {"metric":{"app":"a"},"values":[[1772000000,"0.0666666666666667"]]}]}}"#;
        let other = br#"{"data":{"resultType":"matrix","result":[
            {"metric":{"app":"a"},"values":[[1772000000,"0.06666666666666671"]]}]}}"#;
        let different = br#"{"data":{"resultType":"matrix","result":[
            {"metric":{"app":"a"},"values":[[1772000000,"0.07"]]}]}}"#;
        assert_eq!(
            digest_response(loki, &[]).expect("valid").digest,
            digest_response(other, &[]).expect("valid").digest
        );
        assert_ne!(
            digest_response(loki, &[]).expect("valid").digest,
            digest_response(different, &[]).expect("valid").digest
        );
    }

    /// A metric result's grouping *is* its identity, so an extra series has to
    /// show even when it carries no samples.
    #[test]
    fn metric_series_identity_is_digested_even_when_a_series_is_empty() {
        let one = br#"{"data":{"resultType":"matrix","result":[
            {"metric":{},"values":[[1772000000,"1"]]}]}}"#;
        let plus_empty = br#"{"data":{"resultType":"matrix","result":[
            {"metric":{},"values":[[1772000000,"1"]]},
            {"metric":{"app":"a"},"values":[]}]}}"#;
        let regrouped = br#"{"data":{"resultType":"matrix","result":[
            {"metric":{"app":"a"},"values":[[1772000000,"1"]]}]}}"#;
        assert_ne!(
            digest_response(one, &[]).expect("valid").digest,
            digest_response(plus_empty, &[]).expect("valid").digest
        );
        assert_ne!(
            digest_response(one, &[]).expect("valid").digest,
            digest_response(regrouped, &[]).expect("valid").digest,
            "a sample under a different series identity is a different answer"
        );
    }

    #[test]
    fn a_streams_answer_and_a_matrix_answer_are_never_the_same_answer() {
        let streams = br#"{"status":"success","data":{"resultType":"streams","result":[]}}"#;
        let matrix = br#"{"status":"success","data":{"resultType":"matrix","result":[]}}"#;
        assert_ne!(
            digest_response(streams, &[]).expect("valid").digest,
            digest_response(matrix, &[]).expect("valid").digest
        );
    }

    #[test]
    fn the_direction_the_query_asked_for_is_checked_separately_from_the_digest() {
        let backward = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a"},"values":[["2","beta"],["1","alpha"]]}]}}"#;
        let forward = br#"{"data":{"resultType":"streams","result":[
            {"stream":{"app":"a"},"values":[["1","alpha"],["2","beta"]]}]}}"#;
        let backward = digest_response(backward, &[]).expect("valid");
        let forward = digest_response(forward, &[]).expect("valid");
        assert!(backward.ordered);
        assert!(
            !forward.ordered,
            "the matrix asks for direction=backward, so ascending entries are a finding"
        );
        assert_eq!(
            backward.digest, forward.digest,
            "the digest itself stays order-independent, or a reordered answer would read as a \
different one"
        );
    }

    #[test]
    fn a_body_that_is_not_a_loki_result_is_an_error_not_an_empty_answer() {
        assert!(digest_response(b"not json", &[]).is_err());
        assert!(digest_response(br#"{"status":"error"}"#, &[]).is_err());
    }

    /// A malformed response used to digest as an empty agreement: the old
    /// digest skipped whatever it could not read.
    #[test]
    fn a_response_whose_shape_is_wrong_is_an_error_rather_than_a_silent_skip() {
        for body in [
            br#"{"data":{"resultType":"streams","result":[{"stream":{"app":"a"}}]}}"#.as_slice(),
            br#"{"data":{"resultType":"streams","result":[{"values":[["1","a"]]}]}}"#.as_slice(),
            br#"{"data":{"resultType":"streams","result":[
                {"stream":{"app":"a"},"values":[["1"]]}]}}"#
                .as_slice(),
            br#"{"data":{"resultType":"streams","result":[
                {"stream":{"app":"a"},"values":[[1,"a"]]}]}}"#
                .as_slice(),
            br#"{"data":{"resultType":"streams","result":[
                {"stream":{"app":null},"values":[["1","a"]]}]}}"#
                .as_slice(),
            br#"{"data":{"resultType":"matrix","result":[
                {"metric":{},"values":[[1772000000,"nope"]]}]}}"#
                .as_slice(),
        ] {
            assert!(
                digest_response(body, &[]).is_err(),
                "{} digested instead of erroring",
                String::from_utf8_lossy(body)
            );
        }
    }

    /// Nothing is exempt from placement any more, and only the two derived
    /// names are exempt at all.
    #[test]
    fn every_label_is_digested_where_the_response_put_it() {
        assert_eq!(tag("trace_id", "stream"), Some("stream"));
        assert_eq!(tag("trace_id", "entry"), Some("entry"));
        assert_eq!(tag("app", "stream"), Some("stream"));
        assert_eq!(tag("level", "entry"), Some("entry"));
        assert_eq!(tag("detected_level", "stream"), None);
        assert_eq!(
            tag("__stream_shard__", "stream"),
            None,
            "Loki's shard label is its own sharding decision, not an answer"
        );
        assert_eq!(
            tag("service_name", "metric"),
            Some("metric"),
            "service_name is pushed data under OTLP, not a derived label"
        );
        // The guard on the exemption: it is by name, and the neighbouring
        // reserved names are not swept in with it. A prefix rule would have
        // exempted whatever Loki adds next without anyone deciding to.
        assert_eq!(tag("__error_details__", "entry"), Some("entry"));
        assert_eq!(tag("__stream_shard", "stream"), Some("stream"));
    }

    #[test]
    fn window_boundaries_snap_down_to_a_whole_step() {
        let step = 10_000_000_000i64;
        assert_eq!(
            align_to_step(1_785_307_465_000_000_000, step),
            1_785_307_460_000_000_000
        );
        assert_eq!(
            align_to_step(1_785_307_460_000_000_000, step),
            1_785_307_460_000_000_000
        );
        assert_eq!(
            align_to_step(12_345, 0),
            12_345,
            "a zero step aligns nothing"
        );
    }

    #[test]
    fn an_unset_anchor_is_refused_because_two_runs_would_seed_different_data() {
        let mut verify = verify_for_test();
        verify.anchor_ns = 0;
        assert!(require_anchor(&verify).is_err());
        verify.anchor_ns = 1_772_000_000_000_000_000;
        assert!(require_anchor(&verify).is_ok());
    }

    fn verify_for_test() -> Verify {
        Verify {
            tenant_prefix: "verify-tenant".to_string(),
            rows: 1_000,
            streams: 8,
            labels_per_stream: 6,
            step_ns: 1_000_000,
            anchor_ns: 1_772_000_000_000_000_000,
            entries_per_push: 100,
            push_connections: 2,
            windows: 3,
            repeats: 2,
            limit: 100,
            step_seconds: 10,
        }
    }

    /// The check that keeps the port honest: the same log rows, answered in
    /// Loki's response schema and in the first-party NDJSON, reduce to the
    /// same digest — the cross-schema basis the timing tables cite.
    #[test]
    fn the_first_party_log_shape_reduces_to_the_same_digest_as_lokis() {
        let loki = br#"{"status":"success","data":{"resultType":"streams","result":[{"stream":{"service_name":"api","level":"error","trace_id":"abc"},"values":[["1700000000000000002","boom"],["1700000000000000001","earlier"]]}]}}"#;
        let ndjson = br#"{"timestamp":"1700000000000000002","line":"boom","attributes":{"service_name":"api","level":"error","trace_id":"abc"}}
{"timestamp":"1700000000000000001","line":"earlier","attributes":{"service_name":"api","level":"error","trace_id":"abc"}}"#;
        let basis = vec!["service_name".to_string(), "level".to_string()];
        let from_loki = digest_response(loki, &basis).unwrap();
        let first_party = digest_first_party_response(ndjson, &basis).unwrap();
        assert_eq!(from_loki.reduced_digest, first_party.reduced_digest);
        assert_eq!(first_party.rows, 2);
        assert!(first_party.ordered, "backward order holds");
    }

    /// A histogram bucket is the matrix sample the other systems label the
    /// same instant with: count over width, timestamped at the bucket's end.
    #[test]
    fn a_histogram_bucket_reduces_to_the_matrix_sample_it_answers() {
        let loki = br#"{"status":"success","data":{"resultType":"matrix","result":[{"metric":{},"values":[[1700000010,"0.5"],[1700000020,"0"]]}]}}"#;
        let ndjson =
            br#"{"bucket_start":"1700000000000000000","bucket_end":"1700000010000000000","count":5}
{"bucket_start":"1700000010000000000","bucket_end":"1700000020000000000","count":0}"#;
        let basis: Vec<String> = Vec::new();
        let from_loki = digest_response(loki, &basis).unwrap();
        let first_party = digest_first_party_response(ndjson, &basis).unwrap();
        assert_eq!(from_loki.reduced_digest, first_party.reduced_digest);
        assert_eq!(first_party.kind, "matrix");
        assert_eq!(first_party.series, 1);
    }

    /// Every first-party matrix path is a flat query the shared parameter
    /// grammar accepts — no LogQL text survives in a signy URL.
    #[test]
    fn first_party_matrix_paths_are_flat_queries() {
        let mut cfg = crate::config::Config::from_env().expect("the env defaults build a config");
        cfg.target = Target::Signy;
        cfg.verify = verify_for_test();
        let corpus = verify_corpus(&cfg);
        let queries = build_queries(&cfg, &corpus);
        assert!(!queries.is_empty());
        for query in &queries {
            if query.shape == Shape::Rate {
                assert!(
                    query.path.starts_with("/signy/api/v1/logs/histogram?"),
                    "{}",
                    query.path
                );
                assert!(query.path.contains("bucket=10s"), "{}", query.path);
            } else {
                assert!(
                    query.path.starts_with("/signy/api/v1/logs?"),
                    "{}",
                    query.path
                );
                assert!(query.path.contains("direction=backward"), "{}", query.path);
            }
            assert!(!query.path.contains("query="), "{}", query.path);
        }
    }
}
