//! What gets sent: push bodies built from `signy::corpus`, and the query
//! shapes that read them back.
//!
//! The corpus is the library's, not a second one written here. The harness it
//! replaces had one hardcoded label set and lines padded with `"x".repeat(n)`
//! (`bin/load.rs:680,693` before the rewrite), which compressed 31.5x where
//! realistic lines compress about 5.9x, and left the stream index, label
//! matching and row-group selection untouched by every run.

use signy::corpus::{Corpus, LEVELS, PHRASES, Rng, STATUSES};
use signy::memtable::LogEntry;

/// One push, ready for the wire.
pub struct PushBody {
    /// What goes on the wire. For signy that is the export inside a one-record
    /// collecty batch, compressed; for the others it is the export itself.
    /// `encoded_bytes` is the export either way, so the two stay comparable.
    pub bytes: Vec<u8>,
    /// The tenant header to send, when the target reads one. `None` for signy:
    /// the tenant is a resource attribute inside `bytes`.
    pub tenant_header: Option<(&'static str, String)>,
    pub entries: usize,
    pub line_bytes: usize,
    pub encoded_bytes: usize,
    pub streams: usize,
    pub out_of_order_entries: usize,
    pub max_lateness_ms: u64,
}

pub struct ArrivalOrder {
    /// Milliseconds of backward jitter applied to every entry, which is what
    /// makes a batch unsorted on arrival.
    pub spread_ms: u64,
    /// Fraction of entries that arrive materially late, on top of the spread.
    pub late_fraction: f64,
    pub late_max_ms: u64,
}

pub struct PushGenerator {
    corpus: std::sync::Arc<Corpus>,
    rng: Rng,
    streams_by_tenant: Vec<Vec<usize>>,
    cursors: Vec<usize>,
    entries_per_push: usize,
    streams_per_push: usize,
    wire: crate::config::PushWire,
    arrival: ArrivalOrder,
}

impl PushGenerator {
    pub fn new(
        corpus: std::sync::Arc<Corpus>,
        seed: u64,
        entries_per_push: usize,
        streams_per_push: usize,
        arrival: ArrivalOrder,
        wire: crate::config::PushWire,
    ) -> Self {
        let mut streams_by_tenant = vec![Vec::new(); corpus.tenant_ids.len().max(1)];
        for (index, stream) in corpus.streams.iter().enumerate() {
            let tenant = corpus
                .tenant_ids
                .iter()
                .position(|id| id == &stream.tenant)
                .unwrap_or(0);
            streams_by_tenant[tenant].push(index);
        }
        let cursors = vec![0usize; corpus.streams.len()];
        Self {
            corpus,
            rng: Rng::new(seed ^ 0x9e37_79b9),
            streams_by_tenant,
            cursors,
            entries_per_push,
            streams_per_push,
            wire,
            arrival,
        }
    }

    /// A push carries several streams of one tenant, because that is the shape
    /// an agent sends and because the tenant is a request header: one push
    /// cannot span two of them.
    pub fn next_body(&mut self, now_ns: i64) -> PushBody {
        let corpus = self.corpus.clone();
        let tenant_index = self.rng.below(self.streams_by_tenant.len());
        let candidate_count = self.streams_by_tenant[tenant_index].len();
        let stream_count = self.streams_per_push.min(candidate_count).max(1);
        let first = self.rng.below(candidate_count);
        let chosen: Vec<usize> = (0..stream_count)
            .map(|position| {
                self.streams_by_tenant[tenant_index][(first + position) % candidate_count]
            })
            .collect();

        let mut out_of_order_entries = 0usize;
        let mut max_lateness_ms = 0u64;
        let mut line_bytes = 0usize;
        let mut batch: Vec<(signy::memtable::Labels, Vec<LogEntry>)> =
            Vec::with_capacity(stream_count);

        for (position, stream_index) in chosen.into_iter().enumerate() {
            let share = self.entries_per_push / stream_count
                + usize::from(position < self.entries_per_push % stream_count);
            if share == 0 {
                continue;
            }
            let stream = &corpus.streams[stream_index];
            if stream.entries.is_empty() {
                continue;
            }
            let mut entries = Vec::with_capacity(share);
            let mut previous_ts = i64::MIN;
            for _ in 0..share {
                let cursor = self.cursors[stream_index] % stream.entries.len();
                self.cursors[stream_index] = cursor + 1;
                let source = &stream.entries[cursor];
                let lateness_ms = self.lateness_ms();
                max_lateness_ms = max_lateness_ms.max(lateness_ms);
                let timestamp_ns = now_ns - (lateness_ms as i64) * 1_000_000;
                if timestamp_ns < previous_ts {
                    out_of_order_entries += 1;
                }
                previous_ts = timestamp_ns;
                line_bytes += source.line.len();
                entries.push(LogEntry {
                    timestamp_ns,
                    line: source.line.clone(),
                    structured_metadata: source.structured_metadata.clone(),
                });
            }
            batch.push(((*stream.labels).clone(), entries));
        }

        // The OTLP body every target ingests — see `otlp::encode_export_logs`
        // for the label mapping it applies. signy reads the tenant out of it,
        // the others out of the header, so exactly one of the two carries it.
        let tenant = corpus.tenant_ids[tenant_index].as_str();
        let header = self.wire.tenant_header(tenant);
        let payload = crate::otlp::encode_export_logs(&batch, header.is_none().then_some(tenant));
        let encoded_bytes = payload.len();
        let bytes = self.wire.wrap(payload);
        PushBody {
            tenant_header: header,
            entries: batch.iter().map(|(_, entries)| entries.len()).sum(),
            streams: batch.len(),
            line_bytes,
            encoded_bytes,
            out_of_order_entries,
            max_lateness_ms,
            bytes,
        }
    }

    fn lateness_ms(&mut self) -> u64 {
        let mut lateness = self.rng.below(self.arrival.spread_ms as usize + 1) as u64;
        if self.arrival.late_fraction > 0.0 && self.rng.unit() < self.arrival.late_fraction {
            lateness += self.rng.below(self.arrival.late_max_ms as usize + 1) as u64;
        }
        lateness
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum QueryShape {
    /// Label matchers only — what a stream browser sends.
    LabelOnly,
    /// `|=` substring, which trigram blooms can prune.
    LineFilter,
    /// `| json | field="value"`, the shape `docs/VISION.md` stakes the claim
    /// on and the shape whose scan limit is `usize::MAX` today.
    JsonField,
    /// A range aggregation, which reads one column and throws the rest away.
    Rate,
    /// An old window, so a part evicted from the local cache has to be
    /// restored from the object store on the measured path.
    RestoreProbe,
    /// A deliberately expensive scan: every stream, a wide window, a large
    /// limit. Weighted **zero by default** — it exists to measure what one
    /// slow query does to every other query's latency (the fair
    /// operation-lock queue), not to be part of the standard mix.
    Heavy,
}

pub const QUERY_SHAPES: [QueryShape; 6] = [
    QueryShape::LabelOnly,
    QueryShape::LineFilter,
    QueryShape::JsonField,
    QueryShape::Rate,
    QueryShape::RestoreProbe,
    QueryShape::Heavy,
];

impl QueryShape {
    pub fn name(self) -> &'static str {
        match self {
            QueryShape::LabelOnly => "label_only",
            QueryShape::LineFilter => "line_filter",
            QueryShape::JsonField => "json_field",
            QueryShape::Rate => "rate",
            QueryShape::RestoreProbe => "restore_probe",
            QueryShape::Heavy => "heavy",
        }
    }
}

pub struct QueryPlan {
    pub shape: QueryShape,
    pub path: String,
    /// The tenant header a read names its tenant with. Every target has one:
    /// a query carries no payload to put it in.
    pub tenant: (&'static str, String),
    pub expression: String,
}

pub struct QueryGenerator {
    corpus: std::sync::Arc<Corpus>,
    rng: Rng,
    target: crate::config::Target,
    weights: [u32; 6],
    window_seconds: i64,
    restore_lookback_seconds: i64,
    limit: usize,
    heavy_window_seconds: i64,
    heavy_limit: usize,
}

impl QueryGenerator {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        corpus: std::sync::Arc<Corpus>,
        seed: u64,
        target: crate::config::Target,
        weights: [u32; 6],
        window_seconds: i64,
        restore_lookback_seconds: i64,
        limit: usize,
        heavy_window_seconds: i64,
        heavy_limit: usize,
    ) -> Self {
        Self {
            corpus,
            rng: Rng::new(seed ^ 0x2545_f491),
            target,
            weights,
            window_seconds,
            restore_lookback_seconds,
            limit,
            heavy_window_seconds,
            heavy_limit,
        }
    }

    pub fn next_plan(&mut self, now_seconds: i64) -> QueryPlan {
        let shape = self.pick_shape();
        let stream = &self.corpus.streams[self.rng.below(self.corpus.streams.len())];
        let tenant = self.target.read_tenant_header(stream.tenant.as_str());
        // The selector comes off a stream that exists, so a query that returns
        // nothing means the engine did not find it rather than that the
        // harness asked for something never written.
        let app = stream
            .labels
            .get("app")
            .cloned()
            .unwrap_or_else(|| "api-gateway".to_string());
        // Both languages are drawn from the same rolls, so the paced sequence
        // of questions is identical whichever target answers them.
        let json_field = if self.rng.below(2) == 0 {
            (
                "status",
                STATUSES[self.rng.below(STATUSES.len())].to_string(),
            )
        } else {
            ("level", LEVELS[self.rng.below(LEVELS.len())].to_string())
        };
        let phrase = PHRASES[self.rng.below(PHRASES.len())];
        let (start, end, step, direction) = match shape {
            QueryShape::RestoreProbe => {
                let start = now_seconds - self.restore_lookback_seconds;
                (start, start + self.window_seconds, 60, "forward")
            }
            QueryShape::Heavy => (
                now_seconds - self.heavy_window_seconds,
                now_seconds,
                60,
                "backward",
            ),
            _ => (
                now_seconds - self.window_seconds,
                now_seconds,
                10,
                "backward",
            ),
        };
        let limit = match shape {
            QueryShape::Heavy => self.heavy_limit,
            _ => self.limit,
        };
        let (expression, path) = match self.target {
            // The first-party API's flat form of the same five questions.
            crate::config::Target::Signy => {
                let mut query = url::form_urlencoded::Serializer::new(String::new());
                match shape {
                    QueryShape::LabelOnly | QueryShape::RestoreProbe => {
                        query.append_pair("attr", &format!("service_name={app}"));
                    }
                    QueryShape::LineFilter => {
                        query.append_pair("attr", &format!("service_name={app}"));
                        query.append_pair("contains", phrase);
                    }
                    QueryShape::JsonField => {
                        let (field, value) = &json_field;
                        query.append_pair("parse", "json");
                        query.append_pair("attr", &format!("service_name={app}"));
                        query.append_pair("attr", &format!("{field}={value}"));
                    }
                    QueryShape::Rate => {
                        query.append_pair("attr", &format!("service_name={app}"));
                        query.append_pair("bucket", "1m");
                    }
                    QueryShape::Heavy => {
                        query.append_pair("attr", "service_name=~.+");
                        query.append_pair("contains", phrase);
                    }
                }
                query.append_pair("start", &start.to_string());
                query.append_pair("end", &end.to_string());
                let path = if shape == QueryShape::Rate {
                    format!("/signy/api/v1/logs/histogram?{}", query.finish())
                } else {
                    query.append_pair("limit", &limit.to_string());
                    query.append_pair("direction", direction);
                    format!("/signy/api/v1/logs?{}", query.finish())
                };
                let expression = path
                    .split_once('?')
                    .map(|(_, expression)| expression.to_string())
                    .unwrap_or_default();
                (expression, path)
            }
            crate::config::Target::Loki => {
                // The OTLP encoder sends `app` as `service.name`, so the
                // promoted stream label both systems answer under is
                // `service_name`.
                let selector = format!("{{service_name=\"{app}\"}}");
                let expression = match shape {
                    QueryShape::LabelOnly | QueryShape::RestoreProbe => selector,
                    QueryShape::LineFilter => format!("{selector} |= \"{phrase}\""),
                    QueryShape::JsonField => {
                        let (field, value) = &json_field;
                        format!("{selector} | json | {field}=\"{value}\"")
                    }
                    QueryShape::Rate => format!("rate({selector}[1m])"),
                    // Every stream, so no index prunes it; a line filter over
                    // a phrase the corpus actually contains, so the scan
                    // decodes lines instead of counting.
                    QueryShape::Heavy => {
                        format!("{{service_name=~\".+\"}} |= \"{phrase}\"")
                    }
                };
                let query = url::form_urlencoded::Serializer::new(String::new())
                    .append_pair("query", &expression)
                    .append_pair("start", &start.to_string())
                    .append_pair("end", &end.to_string())
                    .append_pair("step", &step.to_string())
                    .append_pair("limit", &limit.to_string())
                    .append_pair("direction", direction)
                    .finish();
                (expression, format!("/loki/api/v1/query_range?{query}"))
            }
            // The same questions in LogsQL, the translation `matrix::logsql`
            // uses. This phase measures latency under load rather than row
            // equality, so the languages' bucket-labeling difference does not
            // matter here; sending Loki paths would 404 and measure nothing.
            crate::config::Target::VictoriaLogs => {
                // VictoriaLogs keeps the resource attribute's dotted name.
                let selector = format!("service.name:\"{app}\"");
                // `sort by (_time)` before a `limit`, matching the direction
                // the Loki-path query asks for: a bare LogsQL `limit` has no
                // order contract, and a bound that binds must cut the same
                // end of the window on every target.
                let cut = match direction {
                    "backward" => format!("sort by (_time) desc | limit {limit}"),
                    _ => format!("sort by (_time) | limit {limit}"),
                };
                let expression = match shape {
                    QueryShape::LabelOnly | QueryShape::RestoreProbe => {
                        format!("{selector} | {cut}")
                    }
                    QueryShape::LineFilter => {
                        format!("{selector} AND ~\"{phrase}\" | {cut}")
                    }
                    // `*` is LogsQL's match-all, the translation the matrix
                    // uses for the selector-less rare shapes.
                    QueryShape::Heavy => {
                        format!("* AND ~\"{phrase}\" | {cut}")
                    }
                    // `unpack_json` before the field filter: an OTLP body is a
                    // string VictoriaLogs stores as `_msg` without parsing, so
                    // the JSON fields exist only after a query-time unpack —
                    // the same stage `| json` is on the LogQL side.
                    QueryShape::JsonField => {
                        let (field, value) = &json_field;
                        format!(
                            "{selector} | unpack_json fields ({field}) keep_original_fields | filter {field}:\"{value}\" | {cut}"
                        )
                    }
                    QueryShape::Rate => {
                        format!("{selector} | stats by (_time:1m) rate() as value")
                    }
                };
                let query = url::form_urlencoded::Serializer::new(String::new())
                    .append_pair("query", &expression)
                    // Milliseconds: `/select/logsql/query` reads `start`/`end`
                    // in a different unit than the Loki API's nanoseconds.
                    .append_pair("start", &(start * 1000).to_string())
                    .append_pair("end", &(end * 1000).to_string())
                    .finish();
                (expression, format!("/select/logsql/query?{query}"))
            }
            crate::config::Target::VictoriaMetrics => {
                unreachable!("main refuses the log load phase for victoriametrics")
            }
            crate::config::Target::Mimir => {
                unreachable!("main refuses the log load phase for mimir")
            }
        };
        QueryPlan {
            shape,
            path,
            tenant,
            expression,
        }
    }

    fn pick_shape(&mut self) -> QueryShape {
        let total: u32 = self.weights.iter().sum();
        if total == 0 {
            return QueryShape::LabelOnly;
        }
        let mut roll = (self.rng.next_u64() % total as u64) as u32;
        for (index, weight) in self.weights.iter().enumerate() {
            if roll < *weight {
                return QUERY_SHAPES[index];
            }
            roll -= weight;
        }
        QueryShape::LabelOnly
    }
}

/// Log lines a Loki `query_range` response returned, across every stream.
///
/// Deliberately tolerant: a body this cannot parse counts as zero rows rather
/// than failing the query, because the gate is the status code and this number
/// describes what came back. A restore probe that answers `200` over an empty
/// window is not the same event as one that answered from a restored part, and
/// only the row count tells them apart.
pub fn result_rows(target: crate::config::Target, body: &[u8]) -> u64 {
    match target {
        crate::config::Target::Signy | crate::config::Target::Loki => {
            let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(body) else {
                return 0;
            };
            let result = &parsed["data"]["result"];
            let Some(entries) = result.as_array() else {
                return 0;
            };
            entries
                .iter()
                .map(|entry| entry["values"].as_array().map_or(0, Vec::len) as u64)
                .sum()
        }
        // Newline-delimited JSON, one object per row.
        crate::config::Target::VictoriaLogs => String::from_utf8_lossy(body)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count() as u64,
        crate::config::Target::VictoriaMetrics => {
            unreachable!("main refuses the log load phase for victoriametrics")
        }
        crate::config::Target::Mimir => {
            unreachable!("main refuses the log load phase for mimir")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use signy::corpus::{self, CorpusSpec};
    use std::sync::Arc;

    fn corpus() -> Arc<Corpus> {
        Arc::new(corpus::generate(
            &CorpusSpec::default()
                .rows(2_000)
                .streams(16)
                .tenants(4)
                .tenant_prefix("load-tenant"),
        ))
    }

    #[test]
    fn counts_the_lines_a_loki_response_returned() {
        let body = br#"{"status":"success","data":{"resultType":"streams","result":[
            {"stream":{"app":"a"},"values":[["1","x"],["2","y"]]},
            {"stream":{"app":"b"},"values":[["3","z"]]}]}}"#;
        assert_eq!(result_rows(crate::config::Target::Loki, body), 3);
    }

    #[test]
    fn an_empty_result_counts_zero_rows() {
        let body = br#"{"status":"success","data":{"resultType":"streams","result":[]}}"#;
        assert_eq!(result_rows(crate::config::Target::Loki, body), 0);
        assert_eq!(result_rows(crate::config::Target::Loki, b"not json"), 0);
    }

    /// Every stream in one push has to belong to the tenant in the header,
    /// because the header is per request and the server routes on it.
    #[test]
    fn a_push_carries_one_tenants_streams_and_the_requested_entry_count() {
        let corpus = corpus();
        let mut generator = PushGenerator::new(
            corpus.clone(),
            7,
            100,
            4,
            ArrivalOrder {
                spread_ms: 0,
                late_fraction: 0.0,
                late_max_ms: 0,
            },
            crate::config::PushWire::direct(crate::config::Target::Signy),
        );
        for _ in 0..16 {
            let body = generator.next_body(1_772_000_000_000_000_000);
            assert_eq!(body.entries, 100);
            assert!(body.streams > 1, "a push should carry several streams");
            // signy takes the tenant out of the body, so the header is absent
            // and the export is what has to name one.
            assert!(body.tenant_header.is_none());
            assert!(!body.bytes.is_empty());
        }
    }

    #[test]
    fn arrival_order_knobs_produce_late_and_unsorted_entries() {
        let mut ordered = PushGenerator::new(
            corpus(),
            7,
            64,
            2,
            ArrivalOrder {
                spread_ms: 0,
                late_fraction: 0.0,
                late_max_ms: 0,
            },
            crate::config::PushWire::direct(crate::config::Target::Signy),
        );
        let body = ordered.next_body(1_772_000_000_000_000_000);
        assert_eq!(body.out_of_order_entries, 0);
        assert_eq!(body.max_lateness_ms, 0);

        let mut jittered = PushGenerator::new(
            corpus(),
            7,
            64,
            2,
            ArrivalOrder {
                spread_ms: 500,
                late_fraction: 0.25,
                late_max_ms: 30_000,
            },
            crate::config::PushWire::direct(crate::config::Target::Signy),
        );
        let mut out_of_order = 0;
        let mut lateness = 0;
        for _ in 0..8 {
            let body = jittered.next_body(1_772_000_000_000_000_000);
            out_of_order += body.out_of_order_entries;
            lateness = lateness.max(body.max_lateness_ms);
        }
        assert!(out_of_order > 0, "spread must break the arrival order");
        assert!(lateness > 500, "a late fraction must produce late entries");
    }

    #[test]
    fn every_query_shape_names_a_stream_that_exists() {
        let corpus = corpus();
        let mut generator = QueryGenerator::new(
            corpus.clone(),
            11,
            crate::config::Target::Signy,
            [1, 1, 1, 1, 1, 1],
            300,
            600,
            100,
            3600,
            20000,
        );
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..400 {
            let plan = generator.next_plan(1_772_000_000);
            seen.insert(plan.shape.name());
            if plan.shape == QueryShape::Rate {
                assert!(plan.path.starts_with("/signy/api/v1/logs/histogram?"));
            } else {
                assert!(plan.path.starts_with("/signy/api/v1/logs?"));
            }
            assert!(
                corpus
                    .tenant_ids
                    .iter()
                    .any(|id| id.as_str() == plan.tenant.1)
            );
            if plan.shape == QueryShape::Heavy {
                // The heavy shape deliberately selects every stream and asks
                // for its own, larger limit.
                assert!(plan.expression.starts_with("attr=service_name%3D%7E.%2B"));
                assert!(plan.path.contains("limit=20000"));
                continue;
            }
            let app = plan
                .expression
                .split_once("attr=service_name%3D")
                .and_then(|(_, rest)| rest.split_once('&'))
                .map(|(value, _)| value.to_string())
                .expect("every shape selects on service_name");
            assert!(
                corpus
                    .streams
                    .iter()
                    .any(|stream| stream.labels.get("app") == Some(&app))
            );
        }
        assert_eq!(seen.len(), QUERY_SHAPES.len(), "every shape must be drawn");
    }

    /// The load phase against VictoriaLogs asks LogsQL at its own endpoint —
    /// a Loki path would 404 there and the phase would measure error handling.
    /// Same seed, same rolls: the sequence of questions is the sequence the
    /// other two targets get.
    #[test]
    fn victorialogs_plans_ask_logsql_at_its_own_endpoint() {
        let corpus = corpus();
        let mut generator = QueryGenerator::new(
            corpus.clone(),
            11,
            crate::config::Target::VictoriaLogs,
            [1, 1, 1, 1, 1, 1],
            300,
            600,
            100,
            3600,
            20000,
        );
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..400 {
            let plan = generator.next_plan(1_772_000_000);
            seen.insert(plan.shape.name());
            assert!(plan.path.starts_with("/select/logsql/query?query="));
            if plan.shape == QueryShape::Heavy {
                assert!(plan.expression.starts_with("* AND "));
                continue;
            }
            assert!(plan.expression.starts_with("service.name:\""));
        }
        assert_eq!(seen.len(), QUERY_SHAPES.len(), "every shape must be drawn");
    }
}
