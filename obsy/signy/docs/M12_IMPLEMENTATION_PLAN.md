# M12 implementation plan

This plan turns the read-path decision (issue #3) into an executable
implementation and verification sequence: signy gets a first-party log
query API and stops speaking Loki and Tempo.

The decision, recorded here because the API shape dictates everything else:

- **The viewer is fn0.** signy is the observability engine of the fn0
  control plane, not a standalone store courting Grafana users. fn0 serves the
  UI, terminates auth, and reverse-proxies query requests to signy,
  overwriting `X-Scope-OrgID` (the gateway contract of
  [`DEPLOYMENT.md`](DEPLOYMENT.md) §5, unchanged). signy serves no static
  files and needs no CORS.
- **The API is GET + URL query parameters in, NDJSON out.** A query is a flat
  AND of repeatable filters — no query language, no JSON request bodies. The
  same endpoints serve the fn0 UI and AI agents driving `curl`; the contract
  for the second audience is teaching error messages and one authoritative
  document ([`QUERY_API.md`](QUERY_API.md)) pinned by tests.
- **Both compatibility surfaces go.** The Loki endpoints are removed once the
  first-party log endpoints and the comparison-bed port have landed, so logs
  are never unreadable. The Tempo endpoints are removed immediately, before
  the replacement exists: trace ingest and storage continue, but traces are
  unreadable until M13 ships the first-party trace API. That gap is accepted
  knowingly — the alternative was maintaining a Grafana-shaped surface with no
  remaining consumer.

## Scope

M12 delivers:

- The shared flat-filter parameter grammar and its parser (`src/query/params.rs`).
- `GET /signy/api/v1/logs` (search), `/logs/histogram` (bucketed counts),
  `/logs/attributes` and `/logs/attributes/{key}/values` (autocomplete),
  `/logs/tail` (chunked NDJSON streaming), and the delete-request API moved to
  `/signy/api/v1/logs/delete` with a flat-filter persisted form.
- Removal of the Tempo surface (immediately) and the Loki surface (after
  replacement), with the dead code each unlocks: the LogQL text parser, the
  metric evaluator, and four configuration knobs.
- The comparison bed's signy driver ported to the first-party API.
- `docs/QUERY_API.md` with a documentation-pinning test, plus the VISION /
  ARCHITECTURE / DEPLOYMENT / CONFIGURATION / todo.md edits the removal forces.

Outside M12: the trace read path (M13 — first-party trace API), any new index
or storage work motivated by the new read shapes, per-part attribute-value
census for autocomplete, OR / exists / numeric comparison filters, and any
fn0-side work (tracked in `namseent/fn0`).

## Current implementation constraints

- All read routes are registered in `src/router.rs`; the Loki handlers live in
  `src/query/*.rs` files `include!`d by `src/query/mod.rs`. New files join that
  chain.
- The engine boundary is already text-free: `run_unified_query_with_stats`
  (`src/query/execution.rs`) takes a ready-made `logql::LogQuery`, and
  `run_metric_count_scan` drives the `CountingSink` difference-array grid.
  Nothing below the handlers needs the LogQL parser.
- axum's `Query<T>` cannot deserialize repeated keys; the repo's existing
  pattern is `RawQuery` + `url::form_urlencoded::parse`
  (`metadata_params_from_raw`). The flat-filter parser is hand-written on that
  pattern, which is also where unknown-parameter refusal lives.
- The LogQL parser has one consumer outside the Loki handlers:
  `src/delete_requests.rs` parses the persisted delete-request selector at
  startup. Moving delete to flat filters changes a persisted format; the
  format is unversioned by policy, so outstanding delete requests do not
  survive the upgrade and must be re-submitted.
- The comparison bed touches the Loki API in `src/bin/load/workload.rs`
  (query-under-load URL) and `src/bin/load/matrix.rs` (the six query shapes and
  the digest); `compare/run.sh` captures `/loki/api/v1/status/buildinfo`.
  `benches/query.rs` drives `LogScan` directly and is unaffected.
- Attribute keys come from memtable keys plus the part metadata census;
  attribute values come from a newest-1000-rows sample
  (`METADATA_SAMPLE_ROWS`). The new autocomplete endpoints inherit both
  mechanisms and both limitations.

## Design

### 1. Shared parameter grammar

One parser, used by every endpoint. Parameters:

| parameter | repeatable | meaning |
|---|---|---|
| `start`, `end` | no | absolute time (`parse_time_ns` formats: unix s/ms/µs/ns, decimal seconds, RFC3339) or relative `-1h`/`-30m`/`-90s` (unit suffix required, so a bare negative integer stays a negative epoch). Defaults: `end = now`, `start = end − max_query_range` |
| `attr` | yes | attribute filter with the operator embedded: `attr=level=error`, `attr=level!=debug`, `attr=path=~/api/.*`, `attr=host!~db-.*`. The key ends at the first `!=`, `!~`, `=~`, or `=` (longest match at that position) |
| `contains`, `not_contains`, `regex`, `not_regex` | yes | line filters, all ANDed |
| `parse` | ≤1 each of `json`, `logfmt` | run the parser stage before `attr` filters, exposing extracted fields to them |
| `limit` | no | default 100, capped by `max_log_limit` |
| `direction` | no | `forward` / `backward`, default `backward` |
| `bucket` | no | histogram bucket width, duration syntax (`30s`, `5m`) |

An unknown parameter is a 400 whose message names the parameter, lists the
accepted ones, and points at `docs/QUERY_API.md`. An `attr` value without an
operator gets a message showing the correct form. This hand parser is the GET
equivalent of `deny_unknown_fields`. The router's `.fallback()` returns a 404
listing the valid first-party routes, so one wrong `curl` teaches an agent the
whole surface.

### 2. Endpoints

All under `/signy/api/v1/`, tenant from `X-Scope-OrgID` as today. Data
responses are `application/x-ndjson`; refusals are `application/json`
`{"error": "..."}`.

- **`GET /logs`** — maps to `run_unified_query_with_stats`. One row per line,
  sorted in query direction:
  `{"timestamp":"<ns as string>","line":"...","attributes":{...}}`.
  Timestamps are strings because nanosecond epochs exceed 2^53. Scan stats ride
  in `X-Signy-Scanned-Rows` / `X-Signy-Scanned-Bytes` headers — the
  top-K is fully collected before the first byte, so no mid-stream errors
  exist on this endpoint.
- **`GET /logs/histogram`** — same filters plus `bucket`. Default bucket: the
  smallest of 1s/10s/1m/10m/1h/1d that keeps the bucket count ≤ 100; hard cap
  `max_histogram_buckets`. Implemented over `run_metric_count_scan` with
  `times[i] = bucket_end_i − 1` and `range_ns = bucket_ns`, which makes each
  bucket exactly the half-open `[bucket_start, bucket_end)`. Buckets are
  epoch-aligned to the width, clipped to the retention-clamped range, and empty
  buckets are emitted:
  `{"bucket_start":"...","bucket_end":"...","count":42}`.
- **`GET /logs/attributes`** — key names for autocomplete; `start`/`end` only.
  Reuses the current `labels()` internals. `{"key":"service_name"}` per line.
- **`GET /logs/attributes/{key}/values`** — values for autocomplete;
  `start`/`end` plus optional `attr` filters (the flat replacement for the old
  `query=` selector). Line filters are not accepted; the unknown-parameter
  refusal explains why. Bounded by `METADATA_SAMPLE_ROWS`. `{"value":"api"}`
  per line.
- **`GET /logs/tail`** — chunked NDJSON streaming. `TailCursor` and
  `tail_poll` are transport-independent and stay; the WebSocket shell is
  replaced by an `axum` `Body::from_stream` loop. `{"heartbeat":true}` every
  ~15 idle seconds keeps intermediaries from reaping the connection. Params:
  the filters, `limit` (per poll), `start` (the resume cursor), `delay`
  (seconds, ≤ 5). Fresh rows are ordinary row lines in ascending time; the
  `dropped_entries` envelope was constitutionally empty and dies.
- **`GET|POST|DELETE /logs/delete`** — the delete-request API, tenant-scoped
  as today. POST takes flat filters (at least one `attr`; `parse=` refused —
  deletion by parsed field would change meaning when the parser does). The
  persisted form becomes the canonical serialized flat query string, parsed at
  startup by the same shared parser: one parser total, and the stored form is
  the documented form.
- **Unchanged:** `/metrics`, `/ready`, `/signy/api/v1/admin/...`, OTLP
  ingest — including trace ingest and storage.

Errors: `ApiError(StatusCode, String)` rendering `{"error":"..."}` with the
existing status taxonomy — 400 malformed/over-broad (naming the limit and its
knob, showing a correct form), 429 tenant quota / memory pool (says it is
retryable), 504 timeout, 404 unknown route or delete request, 503 drain.

### 3. Internal mapping

- Without `parse=`, every `attr` filter compiles to a `LabelMatcher` in
  `LogQuery.matchers` — the exact current selector semantics, including
  `Eq` matchers feeding exact-field pruning.
- With `parse=json`/`parse=logfmt`, the parse stages are emitted first and
  every `attr` filter compiles to a `PipelineStage::Field` after them. The
  inherited rule, stated verbatim in QUERY_API.md: pushed attributes shadow
  extracted fields of the same name (`merge_extracted`'s behavior). The `_pf:`
  precomputed columns, `json_only_extraction` pruning, and `__error__`
  synthesis keep working untouched — `?parse=json&attr=f=v` is the corpus
  shape VISION defends.
- The four line parameters go into `LogQuery.line_filters`, the position that
  prunes (memtable scan and `ExactFieldPruning`).

### 4. What dies, what stays

Dies with the Loki surface: `src/logql/parser.rs`; the metric and template
halves of `src/logql/ast.rs` (`MetricExpr`, `RangeFunction`, `Unwrap`,
`Grouping`, `BinaryOp`, `AggregateOp`, `QueryExpr`, `Template`,
`LabelFormat`, `LineFormat`); `src/query/loki_extra.rs`, `patterns.rs`, and
`metrics.rs` (the metric evaluator — the histogram keeps only the count-scan
grid helpers); the Loki handlers in `handlers.rs`, which is dissolved (the
Prometheus `/metrics` scrape moves to `src/query/prometheus.rs`). Dies with
the Tempo surface: `src/tempo/` and its routes. Config knobs removed:
`max_metric_series`, `max_metric_samples`, `max_series_matchers`,
`max_concurrent_metric_evaluations`; `max_metric_evaluation_points` is renamed
`max_histogram_buckets`. No new knobs.

Stays: `field_filters.rs` in full; `pipeline.rs`'s extractors and duration
parsing (ingest-side `_pf:`/index writers depend on them); `LabelMatcher`; the
query limits (`max_log_limit`, `max_query_range/runtime/scan_rows/scan_bytes/
memory_bytes`, `max_concurrent_tails`, `tail_poll_interval`, retention
clamping, tenant quota, the query memory pool) — consumed unchanged.

### 5. Comparison bed

The digest layer already bridges response schemas (`reduced_digest` compares a
per-row basis each system computes from its own response). The signy
driver in `matrix.rs` gets a first-party URL builder — `label_only`/
`line_filter` → `/logs`, `json_field*` → `/logs?parse=json&...`,
`metadata_rare`/`trace_window` → `/logs?attr=...`, `rate` →
`/logs/histogram?bucket=<step>` with counts normalized to rates (`count/step`,
as the LogsQL branch already does) — plus an NDJSON reader beside the Loki and
LogsQL readers. The full-digest history breaks at the port (schema changed);
the cross-system reduced-digest agreement — the check that makes the timing
tables citable — survives. Performance stays frozen: this is plumbing, not a
rerun; the next real rerun annotates COMPARISON.md that signy is measured
over its first-party API.

## Implementation sequence

Each phase ends in a commit (and push). Verification per phase:
`cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`; from phase 8 on also `cargo build --bin load`.

1. **Decision and tracking** (this document; issues filed; #3 closed with the
   decision record). No code.
2. **Tempo removal.** Routes, `src/tempo/`, tests; ARCHITECTURE/DEPLOYMENT
   mentions. Trace ingest, storage, retention, and deletion untouched.
3. **Flat parser** — `src/query/params.rs`: `FilterParams::parse(raw_query,
   now_ns)` with the teaching errors; relative-time parsing beside
   `parse_time_ns`; the `ROUTES`/`PARAMS` consts. Unit tests: operator longest
   match, relative vs negative epoch, error wording, `parse=json`
   recompilation.
4. **`/logs`** — `src/query/logs.rs`, `ApiError`, router entries + fallback.
   Integration tests in the `oneshot` style: filters and operators,
   `parse=json` incl. `_pf` precompute, limit/direction, retention clamp,
   429/400/504, NDJSON shape, timestamp-as-string.
5. **`/logs/histogram`** — bucket ladder, epoch alignment, the
   `bucket_end − 1` evaluation trick. Boundary tests (`ts == bucket_start`,
   `bucket_end − 1`, `bucket_end`), clipping, cap refusal, `sum(count)` equals
   the `/logs` row count under the same filters.
6. **Attributes** — `src/query/attributes.rs`; tests ported from the current
   `labels`/`label_values` tests.
7. **Tail rewrite** — streaming body + heartbeat; the raw-socket WebSocket
   test becomes a streamed-body test; drain closes the stream cleanly.
8. **Delete move** — flat parameters, persisted-form change, restart
   round-trip test, `parse=` refusal.
9. **`docs/QUERY_API.md`** + the pinning test; DEPLOYMENT.md example URLs.
10. **Comparison-bed port** — `matrix.rs`/`workload.rs` URL builders, NDJSON
    reader, rate normalization, per-shape URL unit tests; `compare/run.sh`
    buildinfo capture swap.
11. **Loki surface removal** — routes; `loki_extra.rs`, `patterns.rs`,
    `query/metrics.rs`, `logql/parser.rs`, metric AST; `handlers.rs`
    dissolved; dead knobs and CONFIGURATION.md. Tests that guard engine
    behavior rather than the wire — retention clamping, quota 429, scan and
    memory budgets, delete-request visibility, the synthesized-extracted-field
    pruning pair, the e2e read-back sections — are ported to the first-party
    API **in the same commit**, so coverage never dips.
12. **Docs sweep** — VISION.md, ARCHITECTURE.md, todo.md; optional
    `logql` → `log_query` module rename as a final, skippable step.

## Acceptance checklist

- [x] Tempo routes and `src/tempo/` are gone; trace ingest and storage still
  work (covered by ingest/retention tests).
- [x] `GET /signy/api/v1/logs` answers flat-filter queries over memtable
  and parts with NDJSON rows, string timestamps, and scan-stat headers.
- [x] `attr` filters without `parse=` prune via exact-field blooms (the
  synthesized-extracted-field pruning tests pass re-expressed against the new
  API).
- [x] `/logs/histogram` buckets are half-open `[start, end)`, epoch-aligned,
  dense, and `sum(count)` matches `/logs` row counts under the same filters.
- [x] `/logs/attributes` and `/logs/attributes/{key}/values` serve
  autocomplete within the documented sampling bounds.
- [x] `/logs/tail` streams chunked NDJSON with heartbeats, resumes via
  `start`, and ends cleanly on drain.
- [x] Delete requests are submitted, listed, cancelled, and persisted in the
  flat form; a restart re-parses the persisted form.
- [x] Unknown parameters, missing operators, and over-broad queries are
  refused with messages that name the input, the accepted forms, and the
  governing knob.
- [x] The comparison bed drives signy over the first-party API and the
  cross-schema digest equivalence is pinned by unit tests; the live cross-system
  smoke run is deferred to the next real bed rerun.
- [x] Every `/loki/api/v1/*` route is gone, along with the LogQL text parser,
  the metric evaluator, and the four dead knobs; `CONFIGURATION.md` and its
  pinning test agree.
- [x] `docs/QUERY_API.md` exists and its pinning test fails if a route or
  parameter is undocumented.
- [x] Engine-behavior tests (retention clamp, quota, budgets, delete
  visibility, e2e read-back) are ported, not deleted.
- [x] `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test --all-targets`, and `git diff --check` pass.

## Risks and open questions

- **Tail transport.** Chunked NDJSON assumes fn0 can proxy a streaming
  response body without buffering. If it cannot, SSE is the drop-in fallback
  over the same poll core. Verified early in the fn0 issue.
- **Histogram boundaries.** The `(t−range, t]` → `[start, end)` conversion is
  the likeliest silent bug; the phase-5 boundary tests are the guard. The
  auto-bucket ladder is a taste call and cheap to change.
- **Autocomplete quality.** Values come from a newest-1000-rows sample; rare
  or old values will not appear over long ranges. Accepted and documented; a
  per-part value census is the recorded future fix.
- **Expressiveness.** Flat AND-only filters: no OR, no attribute-exists, no
  numeric/duration comparisons. Deliberate; `FieldOp::Lt..Gte` already exist
  engine-side, and the operator-scan design reserves `attr=latency>=1.5s` as a
  cheap later extension. Revisit only when the UI hits a wall.
- **Delete-request format.** Outstanding delete requests do not survive the
  upgrade (unversioned by policy); the operational note is cancel/re-submit.
- **Trace read gap.** Between the Tempo removal and M13, traces are write-only.
  Accepted explicitly in the decision record.
