//! Deterministic corpus for the benchmarks and the load harness.
//!
//! Every number this repository has published came from a harness whose lines
//! were `"x".repeat(n)` at cardinality 1 (`docs/VISION.md`, "The ruler comes
//! before the work"). Near-zero entropy made a 5.9x compression ratio read as
//! 31.5x, and one hardcoded label set meant the stream index, label matching
//! and row-group selection were never under load. So the generator below is
//! the part that has to be right: seeded, wall-clock-free, and shaped like the
//! data the engine actually sees.
//!
//! It lives in the library rather than under `benches/` because the load
//! harness needs the same bytes the benches measure. Two generators would
//! drift, and a load result would then be uncomparable with a bench result for
//! a reason nobody could see. `benches/corpus/mod.rs` re-exports this module
//! and adds the bench-only counting allocator and scratch directory, which
//! have no business in a library that a server binary links.

use crate::memtable::{Labels, LogEntry, MemTableSnapshot, SharedLabels};
use crate::part::Row;
use crate::tenant::TenantId;

/// splitmix64. Seeded from the spec, never from the clock, so two runs of the
/// same bench compare the same bytes.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x2545_f491_4f6c_dd1d)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % n as u64) as usize
    }

    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn hex(&mut self, nibbles: usize) -> String {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(nibbles);
        for _ in 0..nibbles {
            out.push(DIGITS[self.below(16)] as char);
        }
        out
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Plain,
    Json,
    Logfmt,
}

#[derive(Clone, Debug)]
pub struct CorpusSpec {
    pub seed: u64,
    pub tenants: usize,
    /// Distinct streams, i.e. distinct label sets. The knob the old harness
    /// did not have.
    pub streams: usize,
    /// Labels carried by every stream. Invariant II's cost is
    /// labels-per-stream x rows, so this is the other half of every sweep.
    pub labels_per_stream: usize,
    pub rows: usize,
    /// Prefix of the generated tenant ids. A load run writes into a server a
    /// human later inspects, so its tenants have to be nameable as its own.
    pub tenant_prefix: String,
    /// Relative weights, not percentages; they are normalized by their sum.
    pub plain_weight: u32,
    pub json_weight: u32,
    pub logfmt_weight: u32,
    /// Structured-metadata pairs per entry.
    pub metadata_pairs: usize,
    pub start_ts_ns: i64,
    pub step_ns: i64,
    /// Jitter timestamps backwards so a stream is not already sorted. The
    /// memtable query path sorts the whole stream on every query and the flush
    /// path sorts globally; pre-sorted input hides both.
    pub out_of_order: bool,
}

impl Default for CorpusSpec {
    fn default() -> Self {
        Self {
            seed: 0x10_9700_2026,
            tenants: 1,
            streams: 64,
            labels_per_stream: 5,
            rows: 10_000,
            tenant_prefix: "bench-tenant".to_string(),
            plain_weight: 3,
            json_weight: 5,
            logfmt_weight: 2,
            metadata_pairs: 2,
            // A fixed instant, not `SystemTime::now()`: a corpus whose
            // partition depends on when the bench ran is a corpus whose
            // row-group layout depends on when the bench ran.
            start_ts_ns: 1_772_000_000_000_000_000,
            step_ns: 1_000_000,
            out_of_order: false,
        }
    }
}

impl CorpusSpec {
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    pub fn streams(mut self, streams: usize) -> Self {
        self.streams = streams;
        self
    }

    pub fn labels_per_stream(mut self, labels: usize) -> Self {
        self.labels_per_stream = labels;
        self
    }

    pub fn tenants(mut self, tenants: usize) -> Self {
        self.tenants = tenants;
        self
    }

    pub fn tenant_prefix(mut self, prefix: &str) -> Self {
        self.tenant_prefix = prefix.to_string();
        self
    }

    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn step_ns(mut self, step_ns: i64) -> Self {
        self.step_ns = step_ns;
        self
    }

    /// Where this corpus's window starts. What lets a bench build several parts
    /// whose ranges do not overlap, the way a flush interval produces them.
    pub fn start_ts_ns(mut self, start_ts_ns: i64) -> Self {
        self.start_ts_ns = start_ts_ns;
        self
    }

    pub fn out_of_order(mut self, out_of_order: bool) -> Self {
        self.out_of_order = out_of_order;
        self
    }

    pub fn metadata_pairs(mut self, pairs: usize) -> Self {
        self.metadata_pairs = pairs;
        self
    }

    pub fn only(mut self, shape: Shape) -> Self {
        let (plain, json, logfmt) = match shape {
            Shape::Plain => (1, 0, 0),
            Shape::Json => (0, 1, 0),
            Shape::Logfmt => (0, 0, 1),
        };
        self.plain_weight = plain;
        self.json_weight = json;
        self.logfmt_weight = logfmt;
        self
    }
}

pub struct Stream {
    pub tenant: TenantId,
    pub labels: SharedLabels,
    pub entries: Vec<LogEntry>,
}

pub struct Corpus {
    pub spec: CorpusSpec,
    pub tenant_ids: Vec<TenantId>,
    pub streams: Vec<Stream>,
    pub label_names: Vec<String>,
}

pub const LABEL_NAMES: [&str; 10] = [
    "app",
    "env",
    "cluster",
    "namespace",
    "container",
    "region",
    "component",
    "level",
    "instance",
    "pod",
];

/// How many values each label takes before the next one starts varying. The
/// last label in a set absorbs whatever is left over, so a spec asking for
/// more streams than the product always gets exactly that many distinct sets.
const LABEL_CARDINALITY: [usize; 10] = [8, 3, 2, 6, 4, 3, 5, 4, 16, 64];

pub const APPS: [&str; 8] = [
    "api-gateway",
    "checkout",
    "search",
    "ingest",
    "billing",
    "notifier",
    "web",
    "scheduler",
];
const ENVS: [&str; 3] = ["prod", "staging", "dev"];
const CLUSTERS: [&str; 2] = ["eu-west-1a", "eu-west-1b"];
const NAMESPACES: [&str; 6] = [
    "default",
    "payments",
    "platform",
    "observability",
    "edge",
    "data",
];
const CONTAINERS: [&str; 4] = ["server", "sidecar", "migrator", "exporter"];
const REGIONS: [&str; 3] = ["eu-west-1", "us-east-1", "ap-northeast-1"];
const COMPONENTS: [&str; 5] = ["http", "worker", "scheduler", "storage", "cache"];
pub const LEVELS: [&str; 4] = ["info", "warn", "error", "debug"];

const METHODS: [&str; 5] = ["GET", "POST", "PUT", "DELETE", "PATCH"];
const PATHS: [&str; 8] = [
    "/api/v1/query_range",
    "/api/v1/push",
    "/healthz",
    "/api/v1/labels",
    "/v1/checkout/session",
    "/v1/accounts/me",
    "/internal/metrics",
    "/api/v1/series",
];
pub const STATUSES: [u32; 6] = [200, 200, 204, 400, 404, 500];

/// Filler composed from recurring phrases, not from a repeated byte and not
/// from uniformly random words.
///
/// Both extremes are wrong in the same way. `"x".repeat(n)` compresses 31.5x
/// and made the checked-in ratio meaningless; word soup compresses about 2x
/// and would understate the disk footprint just as badly in the other
/// direction. Real application logs are a small set of messages repeated with
/// varying identifiers, which is what this is — the `entropy check` table in
/// `part.rs` is what holds the result near the 5.9x
/// `docs/LOAD_RESULTS.md` §2 measured.
pub const PHRASES: [&str; 24] = [
    "request completed successfully",
    "upstream connection reset by peer, retrying",
    "cache miss, restoring object from remote store",
    "tenant quota exceeded, shedding load",
    "backpressure engaged: memtable above threshold",
    "flush segment committed, manifest generation advanced",
    "compaction scheduled for partition",
    "row group pruned by bloom filter",
    "candidate deadline exceeded before first byte",
    "shard rebalanced after lease renewal",
    "checkpoint durable, replay resumed from offset",
    "connection idle, closing after keepalive timeout",
    "retrying request after transient upstream failure",
    "handler returned early: client disconnected",
    "authorization token accepted for principal",
    "rate limiter admitted request within budget",
    "background worker woke on schedule",
    "object store returned a throttling response",
    "manifest CAS lost the race, reloading",
    "writer epoch verified against the manifest",
    "structured metadata parsed without error",
    "query admitted after waiting for a scan permit",
    "partial result returned: scan limit reached",
    "graceful shutdown drained in-flight requests",
];

fn label_value(name_index: usize, value_index: usize) -> String {
    let pool: &[&str] = match LABEL_NAMES[name_index] {
        "app" => &APPS,
        "env" => &ENVS,
        "cluster" => &CLUSTERS,
        "namespace" => &NAMESPACES,
        "container" => &CONTAINERS,
        "region" => &REGIONS,
        "component" => &COMPONENTS,
        "level" => &LEVELS,
        "instance" => return format!("10.4.{}.{}", value_index / 256, value_index % 256),
        _ => return format!("pod-{:04x}", value_index),
    };
    if value_index < pool.len() {
        pool[value_index].to_string()
    } else {
        // A spec that asks for more streams than the pool has values still
        // gets distinct sets, and they still read like the thing they name.
        format!(
            "{}-{}",
            pool[value_index % pool.len()],
            value_index / pool.len()
        )
    }
}

/// Mixed-radix decomposition of the stream ordinal, least significant label
/// first, with the last label absorbing the remainder.
fn labels_for_stream(stream_index: usize, labels_per_stream: usize) -> Labels {
    let count = labels_per_stream.clamp(1, LABEL_NAMES.len());
    let mut remaining = stream_index;
    let mut labels = Labels::new();
    for position in 0..count {
        let value_index = if position + 1 == count {
            remaining
        } else {
            let radix = LABEL_CARDINALITY[position];
            let value = remaining % radix;
            remaining /= radix;
            value
        };
        labels.insert(
            LABEL_NAMES[position].to_string(),
            label_value(position, value_index),
        );
    }
    labels
}

/// A length target, drawn from a three-mode mixture rather than held constant.
/// Real log lines are mostly short with a tail that is not, and the tail is
/// what drives the Parquet page and bloom sizes.
fn target_len(rng: &mut Rng) -> usize {
    let roll = rng.unit();
    if roll < 0.70 {
        70 + rng.below(100)
    } else if roll < 0.95 {
        170 + rng.below(340)
    } else {
        510 + rng.below(1500)
    }
}

fn filler(rng: &mut Rng, out: &mut String, target: usize) {
    while out.len() < target {
        if !out.is_empty() {
            out.push_str("; ");
        }
        out.push_str(PHRASES[rng.below(PHRASES.len())]);
    }
}

fn timestamp_text(ts_ns: i64) -> String {
    let secs = ts_ns.div_euclid(1_000_000_000);
    let nanos = ts_ns.rem_euclid(1_000_000_000);
    let dt = chrono::DateTime::from_timestamp(secs, nanos as u32).unwrap_or_default();
    dt.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string()
}

struct LineFacts {
    level: &'static str,
    status: u32,
    duration_ms: f64,
    trace_id: String,
    span_id: String,
    user_id: String,
    method: &'static str,
    path: &'static str,
    bytes: u32,
    ts_text: String,
}

/// Identifiers drawn from a finite population rather than minted per line.
///
/// A request emits several lines under one `trace_id`, and a day's traffic
/// comes from a bounded set of users. Minting both per line makes every line
/// unique noise, which understates compression as badly as `"x".repeat(n)`
/// overstates it — and it also destroys the exact-field bloom's selectivity,
/// which is the thing this engine claims to be good at.
struct Vocab {
    trace_ids: Vec<String>,
    user_ids: Vec<String>,
}

impl Vocab {
    fn new(rng: &mut Rng, rows: usize) -> Self {
        let traces = (rows / 4).clamp(64, 65_536);
        let users = (rows / 16).clamp(32, 16_384);
        Self {
            trace_ids: (0..traces).map(|_| rng.hex(32)).collect(),
            user_ids: (0..users)
                .map(|_| format!("u-{}", rng.below(500_000)))
                .collect(),
        }
    }
}

fn line_facts(rng: &mut Rng, vocab: &Vocab, ts_ns: i64) -> LineFacts {
    LineFacts {
        level: LEVELS[rng.below(LEVELS.len())],
        status: STATUSES[rng.below(STATUSES.len())],
        duration_ms: (rng.below(200_000) as f64) / 1000.0,
        trace_id: vocab.trace_ids[rng.below(vocab.trace_ids.len())].clone(),
        span_id: rng.hex(16),
        user_id: vocab.user_ids[rng.below(vocab.user_ids.len())].clone(),
        method: METHODS[rng.below(METHODS.len())],
        path: PATHS[rng.below(PATHS.len())],
        bytes: rng.below(64 * 1024) as u32,
        ts_text: timestamp_text(ts_ns),
    }
}

fn message(rng: &mut Rng, target: usize, overhead: usize) -> String {
    let mut message = String::new();
    filler(rng, &mut message, target.saturating_sub(overhead));
    message
}

fn plain_line(rng: &mut Rng, facts: &LineFacts, target: usize) -> String {
    let message = message(rng, target, 150);
    format!(
        "[{}] {:<5} {} {} {} -> {} in {:.3}ms trace={} user={} - {}",
        facts.ts_text,
        facts.level.to_uppercase(),
        facts.path,
        facts.method,
        facts.span_id,
        facts.status,
        facts.duration_ms,
        facts.trace_id,
        facts.user_id,
        message,
    )
}

/// The free-text key is `_msg` because that is the field VictoriaLogs treats
/// as the message when it parses JSON at ingest. Under any other name VL
/// discards the text and a substring filter over the line has nothing to
/// match, so the three engines would be answering different questions — the
/// 2,008-versus-1,147 `line_filter` disagreement was exactly this. To Loki
/// and signy the key is opaque line text; `| json` extracts it as a
/// field named `_msg`, which collides with nothing (reserved names are
/// enforced for stream labels only).
fn json_line(rng: &mut Rng, facts: &LineFacts, target: usize) -> String {
    let message = message(rng, target, 240);
    format!(
        concat!(
            r#"{{"ts":"{}","level":"{}","trace_id":"{}","span_id":"{}","#,
            r#""status":{},"duration_ms":{:.3},"user_id":"{}","#,
            r#""http":{{"method":"{}","path":"{}","bytes":{},"remote":"10.{}.{}.{}"}},"#,
            r#""retry":{},"_msg":"{}"}}"#
        ),
        facts.ts_text,
        facts.level,
        facts.trace_id,
        facts.span_id,
        facts.status,
        facts.duration_ms,
        facts.user_id,
        facts.method,
        facts.path,
        facts.bytes,
        rng.below(32),
        rng.below(256),
        rng.below(256),
        rng.below(3),
        message,
    )
}

fn logfmt_line(rng: &mut Rng, facts: &LineFacts, target: usize) -> String {
    let message = message(rng, target, 180);
    format!(
        "ts={} level={} trace_id={} span_id={} status={} duration_ms={:.3} user_id={} \
         method={} path={} bytes={} msg=\"{}\"",
        facts.ts_text,
        facts.level,
        facts.trace_id,
        facts.span_id,
        facts.status,
        facts.duration_ms,
        facts.user_id,
        facts.method,
        facts.path,
        facts.bytes,
        message,
    )
}

fn pick_shape(rng: &mut Rng, spec: &CorpusSpec) -> Shape {
    let total = spec.plain_weight + spec.json_weight + spec.logfmt_weight;
    if total == 0 {
        return Shape::Plain;
    }
    let roll = (rng.next_u64() % total as u64) as u32;
    if roll < spec.plain_weight {
        Shape::Plain
    } else if roll < spec.plain_weight + spec.json_weight {
        Shape::Json
    } else {
        Shape::Logfmt
    }
}

fn metadata(rng: &mut Rng, facts: &LineFacts, pairs: usize) -> Vec<(String, String)> {
    let mut out = Vec::with_capacity(pairs);
    for index in 0..pairs {
        match index {
            0 => out.push(("trace_id".to_string(), facts.trace_id.clone())),
            1 => out.push((
                "pod_ip".to_string(),
                format!("10.9.{}.{}", rng.below(256), rng.below(256)),
            )),
            2 => out.push(("container_id".to_string(), rng.hex(24))),
            _ => out.push((format!("attr_{index}"), rng.hex(8))),
        }
    }
    out
}

/// A value the corpus actually contains, for a query whose predicate is rare.
///
/// The comparison's `json_field` shape filters on `status` or `level`, which
/// match roughly a fifth of the rows — so it measures scanning, not pruning,
/// and pruning is what a columnar layout with a per-row-group bloom is *for*.
/// A `trace_id` is drawn from a population of `rows / 4`, so one of them selects
/// about four rows out of a hundred and fifty thousand.
///
/// Reproducible because [`Vocab::new`] is the first thing to touch the seeded
/// generator, so rebuilding it here rebuilds the same vocabulary without
/// regenerating a single row.
///
/// The value is *probably* present rather than certainly: with four expected
/// occurrences, about 2% of the population never gets drawn. An empty answer is
/// still a fair comparison — both engines are asked the same thing — and the
/// matrix reports the row count, so it is visible rather than silent.
pub struct RareValues {
    pub trace_id: String,
    pub user_id: String,
}

pub fn rare_field_values(spec: &CorpusSpec) -> RareValues {
    let mut rng = Rng::new(spec.seed);
    let vocab = Vocab::new(&mut rng, spec.rows);
    RareValues {
        trace_id: vocab.trace_ids[vocab.trace_ids.len() / 2].clone(),
        user_id: vocab.user_ids[vocab.user_ids.len() / 2].clone(),
    }
}

pub fn generate(spec: &CorpusSpec) -> Corpus {
    let mut rng = Rng::new(spec.seed);
    let vocab = Vocab::new(&mut rng, spec.rows);
    let tenant_count = spec.tenants.max(1);
    let stream_count = spec.streams.max(1);
    let tenant_ids: Vec<TenantId> = (0..tenant_count)
        .map(|index| {
            TenantId::parse(&format!("{}-{index:03}", spec.tenant_prefix))
                .expect("corpus tenant id is valid")
        })
        .collect();

    let mut streams: Vec<Stream> = (0..stream_count)
        .map(|index| Stream {
            tenant: tenant_ids[index % tenant_count].clone(),
            labels: SharedLabels::new(labels_for_stream(index, spec.labels_per_stream)),
            entries: Vec::new(),
        })
        .collect();

    for row in 0..spec.rows {
        let stream = &mut streams[row % stream_count];
        let mut ts_ns = spec.start_ts_ns + (row as i64) * spec.step_ns;
        if spec.out_of_order {
            // Late arrivals, not random noise: a stream's entries land behind
            // the clock, they do not jump ahead of it.
            ts_ns -= (rng.below(64) as i64) * spec.step_ns;
        }
        let facts = line_facts(&mut rng, &vocab, ts_ns);
        let target = target_len(&mut rng);
        let line = match pick_shape(&mut rng, spec) {
            Shape::Plain => plain_line(&mut rng, &facts, target),
            Shape::Json => json_line(&mut rng, &facts, target),
            Shape::Logfmt => logfmt_line(&mut rng, &facts, target),
        };
        let structured_metadata = metadata(&mut rng, &facts, spec.metadata_pairs);
        stream.entries.push(LogEntry {
            timestamp_ns: ts_ns,
            line,
            structured_metadata,
        });
    }

    let mut label_names: Vec<String> = LABEL_NAMES
        .iter()
        .take(spec.labels_per_stream.clamp(1, LABEL_NAMES.len()))
        .map(|name| name.to_string())
        .collect();
    label_names.sort();

    Corpus {
        spec: spec.clone(),
        tenant_ids,
        streams,
        label_names,
    }
}

impl Stream {
    /// The entry with this stream's label pairs folded into its attributes —
    /// what reaches storage, where a "stream" is only a generator grouping.
    pub(crate) fn entry_with_labels(&self, entry: &LogEntry) -> LogEntry {
        let mut merged = entry.clone();
        merged
            .structured_metadata
            .extend(self.labels.iter().map(|(k, v)| (k.clone(), v.clone())));
        merged
    }
}

impl Corpus {
    pub fn entry_count(&self) -> usize {
        self.streams.iter().map(|stream| stream.entries.len()).sum()
    }

    pub fn line_bytes(&self) -> u64 {
        self.streams
            .iter()
            .flat_map(|stream| stream.entries.iter())
            .map(|entry| entry.line.len() as u64)
            .sum()
    }

    /// The buffered form the flush path reads. Storage has no stream concept,
    /// so a stream's labels travel as each entry's attributes — the same fold
    /// the OTLP intake performs on resource attributes.
    pub fn snapshot(&self) -> MemTableSnapshot {
        let mut snapshot: MemTableSnapshot = MemTableSnapshot::new();
        for stream in &self.streams {
            let entries = snapshot.entry(stream.tenant.clone()).or_default();
            for entry in &stream.entries {
                entries.push(stream.entry_with_labels(entry));
            }
        }
        snapshot
    }

    /// Rows in the shape a part writer expects. Callers hand these straight to
    /// `flush_rows`, skipping the memtable — which is where structured metadata
    /// takes its canonical form on the real path — so it is canonicalized here.
    /// The entries themselves stay as generated: a push body carrying unsorted
    /// metadata is exactly what exercises the ingest path's own sort.
    pub fn rows(&self) -> Vec<Row> {
        let mut rows = Vec::with_capacity(self.entry_count());
        for stream in &self.streams {
            for entry in &stream.entries {
                let mut row = Row::from_entry(&stream.tenant, &stream.entry_with_labels(entry));
                crate::memtable::canonicalize_structured_metadata(&mut row.structured_metadata);
                rows.push(row);
            }
        }
        rows
    }

    pub fn lines(&self) -> Vec<&str> {
        self.streams
            .iter()
            .flat_map(|stream| stream.entries.iter())
            .map(|entry| entry.line.as_str())
            .collect()
    }

    pub fn entries(&self) -> Vec<&LogEntry> {
        self.streams
            .iter()
            .flat_map(|stream| stream.entries.iter())
            .collect()
    }

    /// A `(labels, entry)` pairing for the pipeline benches, which take the
    /// stream labels as the pipeline's initial field set.
    pub fn labelled_entries(&self) -> Vec<(&SharedLabels, &LogEntry)> {
        self.streams
            .iter()
            .flat_map(|stream| {
                stream
                    .entries
                    .iter()
                    .map(move |entry| (&stream.labels, entry))
            })
            .collect()
    }

    pub fn tenant(&self) -> &TenantId {
        &self.tenant_ids[0]
    }

    /// The value of `name` on the first stream, for a matcher that selects a
    /// real slice of the corpus rather than nothing.
    pub fn label_value(&self, name: &str) -> String {
        self.streams[0]
            .labels
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn min_ts_ns(&self) -> i64 {
        self.streams
            .iter()
            .flat_map(|stream| stream.entries.iter())
            .map(|entry| entry.timestamp_ns)
            .min()
            .unwrap_or(i64::MIN)
    }

    pub fn max_ts_ns(&self) -> i64 {
        self.streams
            .iter()
            .flat_map(|stream| stream.entries.iter())
            .map(|entry| entry.timestamp_ns)
            .max()
            .unwrap_or(i64::MAX)
    }

    /// Total bytes the label sets occupy once, i.e. before `Row::from_entry`
    /// clones them per row. The denominator invariant II is measured against.
    pub fn distinct_label_bytes(&self) -> u64 {
        self.streams
            .iter()
            .map(|stream| {
                stream
                    .labels
                    .iter()
                    .map(|(name, value)| (name.len() + value.len()) as u64)
                    .sum::<u64>()
            })
            .sum()
    }
}

/// Push-shaped entry batches, which is what the journal takes: one batch per
/// generator stream, each entry carrying the stream's attributes itself.
pub fn push_batches(corpus: &Corpus) -> Vec<Vec<LogEntry>> {
    corpus
        .streams
        .iter()
        .map(|stream| {
            stream
                .entries
                .iter()
                .map(|entry| stream.entry_with_labels(entry))
                .collect()
        })
        .collect()
}
