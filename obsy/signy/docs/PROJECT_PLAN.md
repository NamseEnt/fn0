# Project plan

The milestone definitions and acceptance outcomes originate in
`docs/ARCHITECTURE.md`. This file records execution state and verification so a
new Codex context can continue without relying on chat history.

| Milestone | Status | Outcome |
| --- | --- | --- |
| M0 | Complete (`8594b63`) | Loki push ingest, durable journal acknowledgement, MemTable, and the initial LogQL/query APIs |
| M1 | Complete locally (`2b51b29`) | Parquet parts, bloom/stream indexes, restart recovery, unified query, and merge |
| M2 | Complete | Object-store publication, conditional manifest updates, local cache eviction, and query restoration |
| M3 | Complete (review remediated) | JSON/logfmt parsing, metric queries, field-filter push-down, and bounded query execution sufficient for real dashboards |
| M4 | Complete | OTLP trace ingest, trace-ID lookup, and Tempo-compatible APIs |
| M5 | In progress | Compaction tuning, retention, resource limits, and load validation against explicit targets |
| M6 | Complete (review pending) | Graceful shutdown for machine replacement (SIGTERM handling, forced flush-to-S3, drain-status readiness) and a machine-replacement rehearsal |
| M7 | Planned | Local S3 load validation via in-process latency/fault injection (Tier B) and a local MinIO over the real S3 protocol (Tier C), plus point-in-time gauge observability for load analysis |

## Repository state note

Local `master` and `origin/master` contain different M1 commits based on the
same M0 commit. Continue from local `2b51b29`; do not reconcile or rewrite that
divergence without an explicit user decision.

## M2 acceptance checklist

- [x] A flushed part is uploaded before it becomes visible in the manifest.
- [x] Manifest replacement uses conditional object-store writes and rejects a
  competing replacement.
- [x] Startup restores the manifest catalog and safely reconciles interrupted
  uploads and local merge tombstones.
- [x] Independent local-only merge trees migrate into an initially empty
  manifest without losing or duplicating a tree.
- [x] Eviction retains metadata and indexes while removing least-recently-used
  Parquet bodies to the configured bound.
- [x] A query restores only matching evicted parts and succeeds afterward.
- [x] Object-store durability permits WAL prefix compaction without losing
  acknowledged suffix records.
- [x] A fresh-context review reports no blocking findings.
- [x] The complete required validation set passes after the final review fix.

Actual S3 credentials or an S3-compatible test endpoint are not stored in this
repository. In-memory and local-file backends exercise the object-store
contract in automated tests; live S3 validation remains an environment-level
deployment check.

M2 final verification: `cargo fmt --all -- --check`, Clippy with warnings
denied, all 134 tests, and `git diff --check` passed. A process-level local-file
object-store run also restored an evicted Parquet body and returned the expected
query results. The final fresh-context review reported no blocking findings.

## M3 acceptance checklist

- [x] LogQL has an ordered AST for line filters, `json`, `logfmt`, and field
  comparisons using `=`, `!=`, `=~`, `!~`, `<`, `<=`, `>`, and `>=`.
- [x] JSON and logfmt extract scalar fields deterministically; structured
  metadata participates in field filtering, and malformed parser input has a
  consistent error-field behavior instead of aborting an entire scan.
- [x] String, regex, exact-decimal numeric, and duration filters return
  identical results for MemTable and flushed-part data. Numeric field values
  are rejected when they are not finite decimal values; the 2^53 boundary is
  covered by regression tests.
- [x] `rate`, `count_over_time`, and `bytes_over_time` implement `(t-range, t]`
  windows without inheriting the log query limit.
- [x] `sum`, `avg`, `min`, and `max`, with optional `by (...)`, plus `topk`,
  operate independently at every evaluation timestamp.
- [x] Range metrics return Loki `matrix` results and instant metrics return
  `vector` results; log queries remain `streams`.
- [x] Metric lookback scans and restores data before the HTTP start time, and
  invalid steps/ranges or unsupported expressions return clear client errors.
- [x] Positive exact field equality can conservatively prune new parts or row
  groups, including canonical numeric/duration values before remote-body
  restoration, while old M1/M2 parts load and fall back to scanning without
  false negatives.
- [x] A fresh-context review reports no blocking findings.
- [x] The complete required validation set passes after the final review fix.

M3 explicitly defers `line_format`, `label_format`, `unwrap`,
`quantile_over_time`, binary/vector operators, `without`, offsets, subqueries,
deduplication for crash-replay duplicates, and Parquet range reads. Metric
queries have bounded evaluation points, scanned rows, output samples, and
series; concurrent scans/evaluations are scheduled and timed out, but the
bounded row set is still evaluated in memory rather than using streaming or
pre-aggregation. Structured metadata is promoted to metric labels only when a
`by (...)` clause names the field, with a series cardinality limit. JSON
extraction currently supports scalar/object fields; top-level arrays and null
values are ignored or produce parser-error behavior rather than Loki's full
JSON semantics. Empty-string equality, stream-label fields, and synthesized
`_extracted` collision names remain conservative and do not drive exact-field
pruning.

M3 final verification: `cargo fmt --all -- --check`, Clippy with warnings
denied, `cargo test --all-targets` (171 tests), and `git diff --check` passed.
Review remediation now scans every overlapping part before applying the global
limit, includes the first metric lookback window when `start` is omitted,
counts and bounds physical row scans, streams metric evaluation output under
the sample cap, cooperatively cancels timed-out blocking work before releasing
permits and part guards, and allows parser fields named after storage columns
in metric grouping. Query limits include a 5,000,000-row scan budget, bounded
metric output, eight concurrent log scans, four concurrent metric evaluations,
and a 30-second execution/restore timeout. Object-store and journal failures
are reflected in readiness and retried without requiring new ingest.

## M4 acceptance checklist

The detailed English implementation plan and design decisions are recorded in
[`docs/M4_IMPLEMENTATION_PLAN.md`](M4_IMPLEMENTATION_PLAN.md).

- [x] OTLP gRPC accepts valid ResourceSpans/ScopeSpans and preserves span data.
- [x] Invalid IDs, timestamps, and oversized requests are rejected without partial ingestion.
- [x] OTLP acknowledgement follows durable WAL append.
- [x] Legacy Loki WAL records replay after the WAL format extension.
- [x] Trace data survives flush and process restart.
- [x] Trace-ID Bloom pruning never produces false negatives.
- [x] Lookup combines memtable and immutable parts across partitions.
- [x] Evicted trace bodies restore from the object store on demand.
- [x] Tempo trace-by-ID and bounded search APIs match the implemented compatibility shape.
- [x] Existing Loki ingest, LogQL, object-store, and readiness tests remain green.
- [x] A fresh-context review reports no blocking findings.
- [x] The complete validation set passes after the final review fix.

## M5 acceptance checklist

The implementation plan and provisional validation targets are recorded in
[`docs/M5_IMPLEMENTATION_PLAN.md`](M5_IMPLEMENTATION_PLAN.md).

- [x] M4 review and complete validation are green.
- [x] Merge parameters and query/retention limits are configurable and validated.
- [x] Compaction preserves restart/object-store correctness and stays within the agreed memory and query-disruption targets.
- [x] Expired log and trace parts are removed without deleting active or boundary data.
- [x] Loki and Tempo resource limits cover range, scan, memory, output, concurrency, and timeout.
- [ ] Reproducible load validation meets the agreed throughput, latency, memory, retention, and error-rate targets.

The sustained S3 load validation was not executed in this workspace because
no S3 credentials or S3-compatible deployment/test endpoint is available.
The mixed-workload tool is implemented, but remote restore, retention GC, and
long-running object-store behavior require an environment-level run.
- [ ] Load results and bottlenecks are documented.

## M6 acceptance checklist

The implementation plan and design decisions are recorded in
[`docs/M6_IMPLEMENTATION_PLAN.md`](M6_IMPLEMENTATION_PLAN.md).

- [x] SIGTERM/SIGINT starts the drain sequence and the sequence owns process exit.
- [x] While draining, Loki push returns 503 and OTLP returns UNAVAILABLE before any journal append.
- [x] In-flight acknowledged HTTP and gRPC requests complete before force-flush begins.
- [x] Background flush/merge/retention/eviction workers stop before the final force-flush.
- [x] Force-flush drains both MemTables and any pending checkpoint to the object store and manifest, ignoring size/interval thresholds.
- [x] Persistent object-store failure keeps retrying, warns on stdout, and exits only on explicit operator input; recovery mid-retry finishes and exits cleanly.
- [x] A manual/forced exit before durability recovers with zero loss on the next start via journal replay.
- [x] `/ready` returns 503 while draining and `/metrics` exposes drain progress, pending bytes, and force-flush completion.
- [x] The machine-replacement rehearsal shows a fresh instance resuming with zero acknowledged-data loss.
- [ ] A fresh-context review reports no blocking findings.
- [ ] The complete required validation set passes after the final review fix.

Live S3 is not exercised in this workspace; the in-memory and local-file
object-store backends exercise the same durability contract, and live S3
remains an environment-level deployment check.

## M7 acceptance checklist

The implementation plan and design decisions are recorded in
[`docs/M7_IMPLEMENTATION_PLAN.md`](M7_IMPLEMENTATION_PLAN.md).

- [x] `RuntimeMetrics`/`/metrics` expose active part counts, WAL backlog, MemTable bytes, and merge debt as point-in-time gauges.
- [x] `from_url` optionally wraps the constructed store in a latency/fault-injecting store when load knobs are set, and is a no-op otherwise.
- [~] Tier B: a seeded, reproducible run drives the real server over a latency/fault-injected backend and recovers from injected object-store errors. Storage-layer lossless recovery is covered by a test; the full end-to-end run is blocked by the WAL-compaction wedge bug (see M7_LOAD_RESULTS.md).
- [x] ~~`docker-compose.yml` brings up MinIO and a load script runs the server against it end to end.~~ Done in M7, then **removed** on 2026-07-26: validating the S3 protocol is `object_store`'s job, not this repository's. See `LOAD_VALIDATION.md`.
- [~] Tier C: manifest CAS is confirmed against MinIO over the real S3 protocol (needs `OBJECT_STORE_CONDITIONAL_PUT=etag`, documented). Remote restore/retention GC confirmation is blocked by the same wedge bug.
- [x] The load harness paces to a target rate, splits warmup from steady state, forces eviction→restore, and emits an explicit per-target pass/fail verdict.
- [ ] A run meets the agreed throughput, latency, memory, retention, and error-rate targets on target-class hardware; local runs are error-free with bounded RSS and correct backpressure, and the achieved numbers are recorded. (Blocked: WAL-compaction wedge bug.)
- [x] `docs/M7_LOAD_RESULTS.md` records both tier results, the machine profile, the identified bottleneck, and per-target pass/fail.
- [ ] The M6 fresh-context review gate is cleared (carried over; blocks final acceptance).
- [ ] A fresh-context review reports no blocking findings.
- [x] Focused tests, `cargo test --all-targets`, `cargo fmt --all -- --check`, Clippy with warnings denied, and `git diff --check` pass.

## M12 acceptance checklist

The read-path decision (issue #3) and the implementation plan are recorded in
[`docs/M12_IMPLEMENTATION_PLAN.md`](M12_IMPLEMENTATION_PLAN.md). The
acceptance checklist lives there; this file tracks execution state.

- [x] Phase 2 — Tempo surface removed.
- [x] Phases 3–8 — first-party log API (`params`, `/logs`, histogram,
  attributes, tail, delete move).
- [x] Phase 9 — `docs/QUERY_API.md` + pinning test.
- [x] Phase 10 — comparison bed ported (a live `compare/run.sh` smoke run is
  deferred to the next real bed rerun).
- [x] Phases 11–12 — Loki surface removed (LogQL parser, metric evaluator,
  format stages, dead knobs — the trace query knobs included), docs swept.

## M13 acceptance checklist

The scope is issue #7; the executed plan and its design decisions are in the
issue and the commit messages of the five phases.

- [x] Phase 1 — the shared `attr` grammar learns `>=`, `<=`, `>`, `<`; every
  log endpoint refuses comparisons with a teaching error.
- [x] Phase 2 — the trace engine plumbing returns (five knobs, scan
  semaphore, `RemoteDomain::Traces`, `canonical_trace_id`) and
  `GET /traces/{trace_id}` answers flat span NDJSON; trace scans reserve from
  the shared query memory pool.
- [x] Phase 3 — `GET /traces` search: one overlap rule for both scan halves,
  per-span duration comparisons (`duration`-only, unit required), newest-first
  summaries built from the windowed spans.
- [x] Phase 4 — autocomplete (`/traces/attributes`, values), answered from
  the same bounded window scan and priced honestly in `QUERY_API.md`.
- [x] Phase 5 — budget refusals proven end to end (pool exhaustion 429 with
  the counter, span budget 413, per-tenant policy refusal on all four
  routes); docs swept of the "unreadable until M13" gap.

## M14 acceptance checklist

The scope is issue #8; the plan and its acceptance checklist live in
[`docs/M14_IMPLEMENTATION_PLAN.md`](M14_IMPLEMENTATION_PLAN.md). This file
tracks execution state per phase.

- [x] Phase 1 — the ruler's paper half: the plan document, the `VISION.md`
  metrics claim (the "does not do metrics at all" sentence retired), the
  `ARCHITECTURE.md` M14 row, this section, the `todo.md` M14 section.
- [x] Phase 2 — the ruler's iron half: metric workload generator,
  `MetricShape` matrix, digest classes, VictoriaMetrics compose target
  (v1.150.0, behavior pinned by a live one-sided smoke run),
  `compare/run_metrics.sh` one-sided.
- [x] Phase 3 — model, Gorilla, ingest, journal kind 3, replay-equals-live
  (plus a live HTTP seed of the running binary: 2 640 datapoints, PASS).
- [x] Phase 4 — series index and the degradation ladder: per-tenant
  `max_active_series` decided per datapoint, refused datapoints filtered out
  of the WAL bytes so replay cannot resurrect them, idle eviction lazy under
  pressure and on demand, ladder counters on `/metrics`.
- [x] Phase 5 — parts (`LMS1`/`LMI1` + bloom + meta), registry with the
  quota census, the flush/checkpoint/force-flush threading, local recovery;
  the WAL retires only once metric samples are durable in parts.
- [x] Phase 6 — object storage (`metric-manifest.json` under the shared
  writer epoch, offload/restore/evict/reconcile, the flush transaction's
  third list), retention's third expired list with the
  `metric_part_fully_expired` predicate, and the size-tiered metric
  compactor with a commit-record replay for every crash window, local and
  remote.
- [x] Phase 7 — the read path: seven first-party routes, the executor that
  bounds a selection before decoding a chunk, `QUERY_API.md` under the
  pinning test, and the bed's frozen parser accepting every answer from a
  live server (six shapes, zero errors).
- [x] Phase 8 — bed completion: both engines driven through the paced churn
  phases, both digest classes compared, `COMPARISON_METRICS.md` generated
  from the result JSON under the same sentinel test as the log document, and
  the memory gate's `--scenario metrics`.
- [x] Phase 9 — run, regenerate, publish `COMPARISON_METRICS.md`: it lost,
  and it is published. The churn axis was reached but the competitor took
  more series in less memory without refusing anything; the steady shapes
  disagree on values with equal record sets; the one shape that agreed reads
  1.74x against. The open work is in `todo.md`, and the memory-gate
  calibration of `max_active_series` has not been run.

## Completion protocol

For each pending milestone, replace broad outcome text with a concrete
acceptance checklist before implementation. Record only durable decisions and
verification results here; use Git history for the detailed patch record.
