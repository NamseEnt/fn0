# M14 implementation plan

This plan turns issue #8 into an executable implementation and verification
sequence: signy gets metrics as its third signal — stored as
Gorilla-compressed time series behind the same journal/part/offload lifecycle
logs and traces use, read through the first-party flat-parameter API, and
measured against VictoriaMetrics by a comparison bed whose series-churn phase
is the point of the whole milestone.

The decisions, recorded here because they dictate everything else:

- **Parity is the bar; the budget is the axis.** signy does not try to
  beat VictoriaMetrics on compression, ingest, or query throughput. On the fn0
  query shapes at an equal container memory limit it must be *not materially
  worse* (the `verdict()` thresholds already in `src/bin/compare_report.rs`).
  The owned axis is the declared memory budget: under series churn and
  cardinality explosion inside a 2 GiB container, the engine **degrades by
  refusing new series with a named, teaching refusal — it does not die.** The
  degradation ladder (§4) is designed, configured, observable, and gated, not
  emergent.
- **One sample kind everywhere: `(series labels, timestamp_ns, f64)`.** All
  five OTLP metric types are decomposed at ingest into ordinary float series
  (§3). Histograms become `_bucket{le=}`/`_sum`/`_count` series; exponential
  histograms are downscaled into the same shape; summaries become
  `{quantile=}` series. Below the ingest normalizer there is exactly one
  encoder (Gorilla), one part format, one index, one executor. The cardinality
  multiplication this causes is not dodged — it is precisely the churn axis
  the design must survive, and the bed measures it.
- **The third signal is a fork, not a trait.** Traces set the precedent: a
  parallel copy of the log lifecycle (memtable, part, registry, flush
  threading, remote domain), not an abstraction. Metrics follow it. The one
  place traces set *no* precedent — merge — gets a metric-specific compactor
  (§5), not a generalization of the log-only `src/merge/`.
- **Journal the OTLP bytes, decode once.** Like OTLP logs (kind 2), the
  metrics journal record is the raw `ExportMetricsServiceRequest` protobuf
  (`TENANT_RECORD_KIND_OTLP_METRICS = 3`). Replay runs the same normalizer as
  live ingest. The decomposition is therefore a pure function of the request
  bytes — no clock, no config-dependent output — and a replay-equivalence test
  asserts it.
- **No PromQL, and no expression language of any kind.** One operation per
  request: an optional per-series function (`rate`/`increase`), then an
  optional one-level aggregation (`sum|avg|min|max|count` grouped `by` label
  keys), or a histogram quantile as its own route. Anything else is a 400 that
  teaches what the surface does support — ratios are two requests composed
  client-side.
- **Rate answers what arrived, and scales nothing.** (Amended 2026-08-27,
  after the first published run measured what the neighbours actually do.)
  `increase` = the sum of positive deltas over the window, counter resets
  folded in; `rate` = `increase / window_seconds`. Prometheus extrapolates to
  the window boundaries. VictoriaMetrics does not extrapolate past the data
  but *does* scale a partially-covered window up to the full range — so the
  original decision record below, which said VictoriaMetrics "does not
  extrapolate", was half right and is corrected here. The three agree wherever
  a window is fully covered; they diverge only where a window reaches past the
  last sample, which is exactly where a target has stopped reporting, and
  inventing traffic there is the one place a scaled answer does real harm — an
  alert evaluated on `/metrics/instant` would keep firing on a rate the dead
  target is no longer producing. So this engine keeps answering what arrived,
  and `QUERY_API.md` says when that differs from the neighbours.

  *Superseded, kept because a moved target does not retract what was believed
  when it was set:* the original record read "**Rate uses the VictoriaMetrics
  definition, not Prometheus extrapolation** — the only competitor in the bed
  is VictoriaMetrics, which deliberately dropped Prometheus's extrapolation;
  matching its definition keeps the rate shapes inside the exact agreement
  digest instead of a tolerance comparison." Two halves of that were wrong and
  the run found both: VictoriaMetrics scales a partially-covered window, and
  the "exact" digest was unsatisfiable against it for an unrelated reason
  (decimal storage). The arithmetic the engine implements did not change; what
  changed is the claim about whose behaviour it matches.
- **The ruler comes first.** Phase 1 is documents (claim, shapes, this plan);
  Phase 2 builds the workload generator, the VictoriaMetrics bed target, and
  the shape matrix *before* the engine can answer a single query — so the
  shapes are frozen before the engine grows to fit them, per the issue's
  explicit instruction.
- **Internal naming avoids `crate::metrics`.** `src/metrics.rs`
  (`RuntimeMetrics`, `GET /metrics`) is self-telemetry and keeps its name. The
  signal's modules use the `series_*` family: `series.rs`, `series_ingest.rs`,
  `series_index.rs`, `series_part.rs`, `series_registry.rs`,
  `series_merge.rs`, `gorilla.rs`. Wire-facing names stay "metrics" because
  that is what they are: `/v1/metrics` (OTLP, spec-fixed),
  `/signy/api/v1/metrics/*` (first-party; no collision with the root
  `/metrics` scrape route), config knobs `SIGNY_MAX_ACTIVE_SERIES` /
  `SIGNY_MAX_METRIC_*`, `RemoteDomain::Metrics`, object-store prefix
  `metrics/`.

## Scope

M14 delivers:

- OTLP metrics ingest (HTTP `/v1/metrics` + gRPC `MetricsService`) for all
  five OTLP metric types, with the three-phase admit/accept contract, count
  caps, and OTLP `partial_success` refusals.
- Journal record kind 3, third memtable in `Journal`, checkpoint snapshot,
  replay.
- `src/gorilla.rs`: delta-of-delta timestamps + XOR values, encode-on-insert
  in the memtable, streaming decode.
- The series index with the degradation ladder: budget-metered, idle-series
  eviction, `max_active_series` refusal, self-telemetry counters.
- Metric parts (`data.bin` Gorilla chunks + `index.bin` series catalog/label
  index + `bloom` + `meta.json`), flush threading, registry, object-storage
  offload under `RemoteDomain::Metrics`, retention, and a metric-specific
  size-tiered compactor.
- The read surface: `/metrics/query`, `/metrics/instant`, `/metrics/quantile`,
  `/metrics/names`, `/metrics/labels`, `/metrics/labels/{key}/values`,
  `/metrics/series` — flat params, NDJSON, teaching errors, `QUERY_API.md`
  sections under the pinning test.
- The metrics comparison bed (`compare/run_metrics.sh`, VictoriaMetrics
  target, churn-explosion phase, `docs/COMPARISON_METRICS.md` regenerated from
  JSON) and a metrics memory-gate scenario.
- The falsifiable claim in `VISION.md` (replacing "this engine does not do
  metrics at all"), the M14 row in `ARCHITECTURE.md`, the `PROJECT_PLAN.md`
  acceptance section, the `todo.md` M14 section, `CONFIGURATION.md` knobs.

Outside M14: Prometheus remote_write ingest and scrape compatibility (the
consumer sends OTLP); recording rules and any server-side alert scheduler (fn0
evaluates alerts by calling `/metrics/instant`); exemplars (OTLP exemplars are
dropped, documented); native exponential-histogram storage (§3 records the
accepted loss); multi-step expression composition (no
`sum(rate(...))/sum(rate(...))` — fn0 computes ratios client-side from two
requests); downsampling/rollups for long ranges; per-signal retention periods
(retention stays per-tenant); generalizing `src/merge/` over signals; any
fn0-side panel work.

## Current implementation constraints

- `crate::metrics` is taken by self-telemetry; `metric_error_status` in
  `src/query/mod.rs` is the shared string-prefix → status mapping despite its
  name. A missing prefix is a silent 500 — a documented past bug — so every
  new error-string family lands in that mapping in the same commit that mints
  it.
- The journal owns the memtables. Kinds 0/1/2 are logs/traces/otlp_logs
  (`src/journal/mod.rs`); `JournalCmd::Append`, `AppendBatchItem`,
  `CheckpointSnapshot`, `writer.rs`, and `replay.rs` each grow a third arm.
  The journal is unversioned by policy: nothing to migrate, but replaying raw
  OTLP bytes means the ingest decomposition must stay deterministic.
- `src/flush.rs` threads memtables/registries/parts through ~40 call sites in
  lockstep; this is the largest mechanical edit and is compile-driven. The
  existing log/trace flush tests must pass unmodified in the same commit.
- `src/merge/` is log-only; traces (few large parts) never needed it. Metric
  parts are small and frequent — a compactor is not optional, and it must land
  before the bed runs or object-store operation counts and query fan-in
  measure an unmerged artifact rather than the design.
- Query admission order is fixed by precedent (`src/query/trace_scan.rs`
  header): pin parts (restore is network wait and must not hold a slot) →
  per-surface scan semaphore → shared query-memory pool → blocking scan under
  the outer timeout. Each surface gets its own semaphore so cost profiles do
  not starve each other. `query_memory.rs` is signal-generic; reservations are
  held alongside the results they paid for.
- The response is decided before the first byte (the `QUERY_API.md` contract):
  series selection and point-count bounding happen before any chunk is
  decoded.
- `TenantQuota::admit_storage` sums stored bytes across signals; the series
  registry joins that census. The retention period is per-tenant, not
  per-signal (`tenant_policy.rs` gains a `metric_part_fully_expired` predicate
  on the same period).
- Tests are full-sentence snake_case; `src/tests/*.rs` are `include!`d; query
  tests live in `src/query/tests.rs`;
  `every_query_api_route_and_param_is_documented` pins `QUERY_API.md` against
  `ROUTES`/`*_PARAMS` in `src/query/params.rs`, so a route and its doc section
  land in the same commit.
- Reusable as-is: `query_memory.rs`, `backpressure.rs` (`IngestGate`),
  `ingest.rs::TimestampWindow`, `tenant.rs`, `clock.rs`, `bloom.rs`,
  `disk.rs`, `shutdown.rs`, `page_cache.rs`, `restore_meter.rs`,
  `OtlpEncoding` (`otlp_http.rs`), the `admit_inflight_body` middleware,
  `part::partition_of`, `part::ByteRange`.

## Design

### 1. The fn0 query shapes and the read surface

fn0 draws dashboard panels and evaluates alerts. Four panel archetypes cover
what it draws: a line chart of raw gauge values, a line chart of counter rates
(optionally grouped), a single-stat/threshold (alerts), and a
latency-percentile panel. Discovery (autocomplete) makes the panel editor
work. That is seven routes, all under `/signy/api/v1/`, tenant from
`X-Scope-OrgID`, NDJSON out, `X-Signy-Scanned-Rows` (samples decoded) /
`X-Signy-Scanned-Bytes` headers, refusals as `application/json`
`{"error": ...}` naming the offending input, the accepted set, the fix, and
`docs/QUERY_API.md`.

Series selection grammar, shared by all shapes: `metric=<name>` (exact
`__name__` equality) and repeatable `attr=key=value` / `attr=key!=value` /
`attr=key=~regex` / `attr=key!~regex` — the identical operator grammar
`src/query/params.rs` already parses for logs, reused verbatim so one grammar
serves all three signals. Duration comparisons stay trace-side; comparison
operators on metric label filters are refused with the same teaching error the
log endpoints use.

| route | params const | params |
|---|---|---|
| `GET /metrics/query` | `METRIC_QUERY_PARAMS` | `metric` (required), `attr`*, `start`, `end`, `step` (duration, required), `func` (`rate`\|`increase`), `range` (duration; required iff `func`), `agg` (`sum`\|`avg`\|`min`\|`max`\|`count`), `by`* (label key; only with `agg`), `lookback` (duration, default `5m`), `limit` (series cap) |
| `GET /metrics/instant` | `METRIC_INSTANT_PARAMS` | `metric` (required), `attr`*, `at` (default now), `func`, `range`, `agg`, `by`*, `lookback`, `limit` |
| `GET /metrics/quantile` | `METRIC_QUANTILE_PARAMS` | `metric` (required; the *base* histogram name — the engine selects `<metric>_bucket` and groups by labels-minus-`le`), `q` (required, 0–1), `attr`*, `start`, `end`, `step`, `range` (required; the increase window per bucket), `by`*, `limit` |
| `GET /metrics/names` | `METRIC_NAMES_PARAMS` | `start`, `end` |
| `GET /metrics/labels` | `METRIC_LABELS_PARAMS` | `start`, `end`, `metric`, `attr`* |
| `GET /metrics/labels/{key}/values` | `METRIC_LABEL_VALUES_PARAMS` | `start`, `end`, `metric`, `attr`* |
| `GET /metrics/series` | `METRIC_SERIES_PARAMS` | `metric`, `attr`*, `start`, `end`, `limit` |

Output shapes:

- `/metrics/query`, `/metrics/quantile`: one NDJSON line per output series:
  `{"labels":{...},"samples":[["<ts ns as string>",1.5],...]}` — samples
  aligned to `start + k*step`, steps with no sample within `lookback` omitted.
  One line per series streams naturally and bounds the per-line buffer to one
  series.
- `/metrics/instant`: one line per output series:
  `{"labels":{...},"timestamp":"<ns>","value":0.97}`. This is the
  alert-evaluation shape: fn0 compares `value` to its threshold; `agg=max`
  with an empty `by` gives "worst instance".
- Discovery routes: `{"name":"..."}` / `{"key":"..."}` / `{"value":"..."}` /
  `{"labels":{...}}` lines, answered from the in-memory index and per-part
  catalogs — exact, not sampled (unlike log attribute values, and the doc says
  so).

Refusal teaching, beyond the shared unknown-parameter/operator errors: `func`
without `range` names both params; `by` without `agg` explains grouping is an
aggregation property; a selector matching more than
`max_metric_series_per_query` series, or `series × steps` past
`max_metric_points_per_query`, is refused *before scanning* with the matched
count, the cap, the governing knob, and the two fixes (narrow the selector, or
add `agg`); `/metrics/quantile` on a summary-backed name explains that
summary quantiles cannot be re-aggregated. Composition requests get the
sentence: one function and one aggregation per request; compute ratios
client-side from two requests.

New `QueryEndpoint` variants in `src/metrics.rs` (self-telemetry):
`MetricQuery`, `MetricInstant`, `MetricQuantile`, `MetricNames`,
`MetricLabels`, `MetricLabelValues`, `MetricSeries`. New error-string prefixes
registered in `metric_error_status` in the same commits that mint them.

### 2. The falsifiable claim

`VISION.md`'s "this engine does not do metrics at all" sentence is rewritten,
and the metrics claim joins the claims in the established pattern — equalized
conditions, a named shape tied to a storage-design difference, asymmetric
halves with rationale, the "without giving up" axes, an explicit abandonment
condition. The claim's wording lives in `VISION.md`; its two conditions are:

1. **Steady:** at an equal container memory limit, on the same corpus and the
   same machine, the fn0 dashboard shapes — a windowed counter `rate` and its
   label-grouped `sum` over the flat-parameter API — answer not materially
   worse than VictoriaMetrics, without giving up ingest throughput or disk
   footprint.
2. **Churn:** when a series-churn workload fills the resident-byte budget,
   signy keeps ingesting within the process memory watermark and answers a
   complete export with `429` + `Retry-After`; the collector retries after
   flush makes room rather than the process slowing, swapping, or dying.

Abandonment: if VictoriaMetrics inside the same limit both survives the same
churn with at least the same sample acceptance *and* beats signy
materially on the steady shapes, the budget axis has bought nothing and the
metrics engine is abandoned as a differentiated claim — it remains a
convenience feature, and the document publishes the loss.

The per-competitor rationale is single-competitor but two-condition: the
steady half concedes VictoriaMetrics' decade of tuning (parity is the win
condition); the churn half names the design difference — VictoriaMetrics
sizes its index to the workload, signy sizes the workload to its budget.

### 3. Data model and OTLP decomposition

**Series** = metric name (stored as label `__name__`) + sorted label pairs.
Labels are the OTLP datapoint attributes plus resource attributes from the
same promotion list logs use (`otlp_log.rs::PROMOTED_RESOURCE_ATTRIBUTES` —
one schema decision shared across signals); other resource/scope attributes
are dropped, documented. Canonical encoding: length-prefixed sorted
`key\0value` bytes — the identity across memtable, index, and parts.
`SeriesId` is a per-tenant, per-process `u64` assigned sequentially by the
index, never reused; parts carry their own catalog mapping local ordinals to
canonical labels, so ids need no cross-restart stability.

Decomposition to `(labels, ts_ns, f64)` samples
(`series_ingest.rs::normalize_request`, a pure function of the request bytes —
replay depends on it):

- **Gauge** → one sample per datapoint. Int datapoints become f64 (53-bit
  exactness documented).
- **Sum** → one series per label set; `is_monotonic` recorded in series
  state. Delta temporality is converted to cumulative at ingest by a
  per-series running total in the index state; if that state was evicted
  (idle/churn) the total restarts at the delta — exactly a counter reset,
  which `rate`/`increase` already absorb. Non-monotonic sums are gauges with
  sum semantics.
- **Histogram** → `<name>_bucket{le="<bound>"}` per explicit bound plus
  `le="+Inf"`, `<name>_sum`, `<name>_count` (cumulative bucket counts).
  Delta temporality: the same running-total conversion per bucket series.
- **Exponential histogram** → downscaled at ingest until the bucket count is
  ≤ 64 (halving resolution per scale step, exact by construction), then
  emitted as `_bucket{le}` series using the base-2 boundaries as `le` values,
  the zero bucket folded into the smallest bound, plus `_sum`/`_count`.
  **Accepted loss**: quantiles are boundary-limited like any bucketed
  histogram; storing the native exponential form would require a second
  sample kind, encoder, and quantile path for a fidelity gain the bar
  (parity) does not demand. Recorded in `QUERY_API.md` and the risks.
- **Summary** → `<name>{quantile="<q>"}` gauge series plus `_sum` and
  `_count`.

Timestamps are OTLP `time_unix_nano`, admitted through the existing
`TimestampWindow`. `MAX_OTLP_METRIC_SAMPLES` caps a request by samples counted
*after* decomposition, since one histogram datapoint fans out to ~66 samples —
the cap must bound what the engine actually stores.

### 4. The series index and the degradation ladder

`src/series_index.rs`: a per-tenant `SeriesIndex` inside the memtable's lock
domain — `HashMap<Box<[u8]>, SeriesState>` (canonical labels → state) plus an
inverted label index (`key → value → sorted series ids`) for memtable-side
selection and discovery. `SeriesState`: `id`, `last_ts_ns`,
`cumulative_total: Option<f64>` (delta conversion), `is_monotonic`, chunk
handle. Every insertion self-meters (canonical bytes + state + inverted-index
postings + chunk bytes) into the memtable's `approximate_size`, which already
feeds the flush trigger and the budget accounting — the index lives *inside*
the declared budget, which is the whole game.

The ladder, in order, each rung observable in `RuntimeMetrics`
(`active_series` gauge; `series_created_total`, `series_evicted_idle_total`,
`series_rejected_total`, `metric_cardinality_rejected_total`, and
`metric_memory_rejected_total` counters):

1. **Early flush.** Sample memory (Gorilla chunks) is reclaimed by the
   ordinary flush path when the memtable share of the budget fills —
   churn-neutral pressure relief, no new mechanism beyond correct metering.
2. **Idle-series eviction.** A series with no sample for
   `metric_series_idle_timeout` (default 10m) is evicted from the index after
   its chunk is flushed: its history is in parts; if it returns it is
   re-created (new id; the only artifact is a possible counter reset for
   delta sums, absorbed by rate). This makes *churn* — pod restarts replacing
   label sets — a bounded cost: dead series leave the budget at the timeout
   horizon. The sweep runs on the flush cadence, longest-idle first.
3. **Process-wide byte admission.** The normal guard is the shared
   `max_memtable_bytes` budget, charged with canonical label bytes, calibrated
   per-series state overhead, and a conservative sample-buffer reservation.
   All tenant groups in one export are admitted under one decision before the
   first WAL append. If the reservation does not fit, the complete export is
   answered with `429` and `Retry-After`; no datapoint is filtered into a
   partial success. `SIGNY_MAX_ACTIVE_SERIES` is disabled by default and is
   retained only as an optional process-wide emergency count guard.
4. **IngestGate.** The existing gate (WAL backlog, memtable bytes, disk
   floor) still fronts everything; nothing new.

Deliberately *not* done: spilling the series index to disk (turns an
explosion into unbounded write amplification and silent latency death — the
opposite of a declared budget), and evicting *active* series (silently
corrupts delta accumulation for live traffic; refusal at the boundary is
honest, eviction of the living is not).

### 5. Part format, lifecycle, and the compactor

`src/series_part.rs`, modeled file-for-file on `trace_part.rs` (data + bloom
+ meta.json, own magic) so `object_storage`'s three-artifact lifecycle
generalizes without redesign:

- **`data.bin`** — magic `LMS1` + per-series Gorilla chunks concatenated in
  series-ordinal order; chunks for a series are time-sorted at write
  (memtable out-of-order spill vectors merge-sorted in).
- **`index.bin`** — magic `LMI1`; series catalog (ordinal → canonical labels,
  chunk `ByteRange`, sample count, min/max ts) + inverted label index (key →
  value → sorted ordinals) + the name dictionary for `/metrics/names`.
  Loaded on demand, sized in `meta.json` for the registry census.
- **`bloom`** — `src/bloom.rs` over hashed `key=value` pair tokens (plus
  `__name__=<name>`), for part pruning *without* fetching `index.bin` — the
  same role the trace bloom plays.
- **`meta.json`** — tenant, partition (`part::partition_of`), min/max ts,
  series count, sample count, per-artifact sizes, journal watermark — the
  fields `trace_part.rs` meta carries, so `series_registry.rs` (modeled on
  `trace_registry.rs`, per-tenant stored-bytes census feeding `TenantQuota`)
  and `FlushTransaction` slot in unchanged in shape.

Object storage: `MetricManifest` + its manifest JSON + the `metrics/` prefix;
`RemoteDomain::Metrics` in `remote_lifecycle.rs` (already
domain-parameterized); publish/remove/restore/evict/reconcile mirroring
traces; `FlushTransaction` gains its third list. Retention: the third list in
`retention.rs`; `tenant_policy.rs::metric_part_fully_expired` on the tenant's
single retention period.

**Compactor** — `src/series_merge.rs`, metric-specific: size-tiered per
tenant+partition. Trigger: ≥ 8 parts in a tier (L0 = fresh flushes, promote
at ~16 MiB, again at ~256 MiB; constants, not knobs, until load says
otherwise — recorded in `todo.md`). A merge streams the input catalogs,
unions series, concatenates and re-sorts each series' chunks (decode +
re-encode only when chunks interleave in time), writes one part, publishes
through the same transactional path flush uses, then removes inputs locally
and remotely. It shares flush's writer code, not `src/merge/`'s. It lands
**before** the bed runs: without it, object-store operation counts and query
fan-in measure an unmerged artifact, not the design.

### 6. The range executor

`src/query/metric_scan.rs` plus handlers `src/query/metrics_query.rs`,
`metrics_instant.rs`, `metrics_quantile.rs`, `metrics_metadata.rs` (joining
the `include!` chain in `src/query/mod.rs`; routes in `src/router.rs`; consts
in `src/query/params.rs`).

Bounding before scanning, in order:

1. **Select** matching series: memtable inverted index ∪ per-part selection
   (prune parts by time range + bloom `key=value` probes for equality
   selectors; open `index.bin` only for survivors; regex selectors iterate
   the surviving catalogs' value lists). Result: per-part ordinal lists and a
   merged output-series set keyed by canonical labels.
2. **Refuse** if the matched series count exceeds
   `max_metric_series_per_query` or `series × steps` exceeds
   `max_metric_points_per_query` — before any chunk is read.
3. **Admit**, in the fixed order: pin parts → `metric_scan_semaphore`
   (`max_concurrent_metric_scans`) → `query_memory.rs` reservation sized from
   the bound (`series × steps × 16 B` plus per-series overhead; held with
   the results, per the `QueryExecution` precedent) → blocking scan under
   `max_metric_query_runtime`.
4. **Execute** per series: k-way merge of that series' chunks across memtable
   and parts in time order, streaming Gorilla decode, folded into the
   operation — raw (last sample within `lookback` at each step),
   `rate`/`increase` (positive-delta sum over `(t-range, t]`, reset-aware),
   aggregation folding series into group accumulators keyed by
   `by`-projected labels (memory then bounded by groups × steps, already
   paid for by the reservation), quantile (per `(labels minus le, step)`:
   increase per bucket, monotone-fix, linear interpolation within the
   bracketing bucket). Emit each finished output series as one NDJSON line.

Instant queries are the same executor with one step. Discovery routes read
only catalogs and indexes, never `data.bin`.

### 7. The comparison bed and the memory gate

- **A separate script, `compare/run_metrics.sh`**, on the `run.sh` skeleton
  (fresh volumes, sequential ingest, settle, restart, query matrix cold/warm,
  peaks and disk, `COMPARE_MEMORY_LIMITS` default `2g 8g`, an OOM recorded as
  a result, per-phase JSON to `target/compare-metrics/`). Separate because
  the competitor set (VictoriaMetrics alone), the phases, and the rerun
  cadence all differ from the logs bed. `compare/docker-compose.yml` gains a
  `victoriametrics` service. **Ingest wire: identical OTLP protobuf bodies**
  to signy `/v1/metrics` and VictoriaMetrics' OTLP endpoint
  (`/opentelemetry/v1/metrics`) — the fairness rule the logs bed established.
  The VictoriaMetrics flags used (resource-attribute promotion, delta
  handling) are pinned by inspection of the actual container in Phase 2 and
  recorded as a fairness footnote in the bed document.
- **Workload** (`src/bin/load/metric_workload.rs` + OTLP metric bodies in
  `src/bin/load/otlp.rs`): seeded, deterministic; knobs: active series count,
  scrape interval, label vocabulary, type mix (gauges, counters, one
  histogram family — the decomposition multiplier is part of the workload's
  honesty). Phases: (1) **steady** — a fixed series set to an event target;
  (2) **churn** — rolling replacement (k% of series get new instance labels
  per minute, the pod-restart shape) at a rate sized so active + idle series
  exceed the 2 GiB capacity; recorded: sample acceptance %, refusal counts,
  memory peak, query latency under churn; (3) **explosion** — a burst
  creating 10× capacity distinct series; recorded: did the engine keep
  answering, what fraction was refused, recovery after the idle horizon;
  (4) **query matrix**.
- **Shapes** — a `MetricShape` enum beside `Shape` in the load harness, doc
  comments arguing each shape's storage rationale, per house style:
  `raw_range` (one gauge series — pure decode and seek), `agg_sum_by`
  (grouped sum over ~1k series — catalog selection and fan-in), `rate_range`
  (counter rate over the window — the chunk-layout shape), `instant_alert`
  (latest value across a selector — the lookback/index shape), `quantile_p99`
  (bucket merge — the decomposition's bill comes due here),
  `churned_selector` (a selector spanning replaced series — the claim's
  shape: reads must cross the churn boundary). The VictoriaMetrics side is
  queried via `/api/v1/query_range` / `/api/v1/query` with MetricsQL
  equivalents.
- **Agreement digests** in two declared classes: *exact* (raw and sum/count
  shapes — values rounded to 9 significant digits, sorted
  `(labels, ts, value)` sha256) and *tolerance* (rate and quantile —
  pointwise `|a−b| ≤ max(1e-9, 0.005·|b|)`, digest over the pass verdicts;
  the rate-definition note in the doc explains why bit-exactness is not
  claimed across engines). Disagreement withholds that shape's ratios, per
  the standing methodology.
- **Report**: `src/bin/compare_report.rs` gains a second document,
  `docs/COMPARISON_METRICS.md`, regenerated wholly from
  `target/compare-metrics/*.json` — fixed prose, generated numbers and
  verdicts, its own run sentinel pinned by a sibling of
  `a_published_document_cites_the_run_that_wrote_its_artifacts`, the
  mandatory "What I do not trust about these numbers" and deferred-axis
  sections, and a first-class **churn table** (acceptance %, refusals, peak
  anon, query latency during churn, per limit). Published win or lose.
- **Memory gate**: `src/bin/memory_gate.rs` gains a metrics scenario
  (env-selected) driving the steady + churn phases under `MemoryMax=2G`,
  gating peak cgroup `anon`; `NOT_MEASURED` if steady-phase sample acceptance
  is under 90% (churn/explosion-phase *new-series* refusals are excluded
  from the denominator — they are the designed behavior, and the gate
  asserts the refusal counters moved instead).

### 8. Configuration

Knobs, each at the four `config.rs` sites (declaration, default, env parsing,
validation) plus `CONFIGURATION.md`:

| knob | default |
|---|---|
| `SIGNY_MAX_METRIC_SAMPLES` (per request, post-decomposition) | 100 000 |
| `SIGNY_MAX_ACTIVE_SERIES` (process-wide emergency guard, optional) | off |
| `SIGNY_METRIC_SERIES_IDLE_TIMEOUT` | 10m |
| `SIGNY_MAX_CONCURRENT_METRIC_SCANS` | 8 |
| `SIGNY_MAX_METRIC_QUERY_RUNTIME` | 30s |
| `SIGNY_MAX_METRIC_RESTORE_RUNTIME` | 25s |
| `SIGNY_MAX_METRIC_SERIES_PER_QUERY` | 10 000 |
| `SIGNY_MAX_METRIC_POINTS_PER_QUERY` | 2 000 000 |

## Implementation sequence

Each phase ends in a commit and a push, with the tree green:
`cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`; from Phase 2 on also `cargo build --bin load`; the binary stays
runnable throughout.

1. **The ruler's paper half** (no code). This document; the `VISION.md` claim
   and the rewrite of the "does not do metrics" sentence; the
   `ARCHITECTURE.md` M14 row; the `PROJECT_PLAN.md` acceptance section; the
   `todo.md` M14 section.
2. **The ruler's iron half.** The metric workload generator, the
   `MetricShape` matrix with rationale doc comments, the digest classes, the
   VictoriaMetrics compose target, `compare/run_metrics.sh` runnable
   one-sided against VictoriaMetrics alone (the signy column
   absent-by-declaration). The shapes and workload are frozen before the
   engine exists to fit them. Unit tests: generator determinism, digest
   stability, URL builders.
3. **Model, Gorilla, ingest, journal.** `series.rs` (`Sample`, canonical
   labels, `SeriesMemTable` with per-series Gorilla chunks and out-of-order
   spill), `gorilla.rs` (encoder/decoder, exhaustive round-trip and
   pathological delta-of-delta tests), `series_ingest.rs`
   (`normalize_request` for all five types + `OtlpMetricIngest` with the
   three-phase contract), HTTP + gRPC ingest, journal kind 3 + replay.
   Tests: decomposition per type (including delta→cumulative and the
   exponential downscale), replay-equals-live-ingest, restart round-trip.
4. **The series index and the ladder.** `series_index.rs`, budget metering,
   idle eviction, `max_active_series` refusal with OTLP `partial_success`,
   `RuntimeMetrics` counters. Tests: the eviction horizon, refusal targets
   only unknown series, the counter-reset artifact after eviction is
   absorbed by a rate fold, metering matches a `memprof` measurement within
   tolerance.
5. **Parts and flush.** `series_part.rs` (writer/reader, magics, bloom),
   `series_registry.rs`, the `flush.rs` threading, local recovery in
   `startup.rs`, `app_state.rs` / `TenantQuota` wiring. Tests: part
   round-trip, pruning by bloom and time, flush abort/commit, recovery
   census; the existing log/trace flush tests pass unmodified.
6. **Object storage, retention, compactor.** `RemoteDomain::Metrics`,
   `MetricManifest`, the `FlushTransaction` third list,
   publish/restore/evict/reconcile; `retention.rs` +
   `tenant_policy.rs::metric_part_fully_expired`; the `series_merge.rs`
   size-tiered compactor with transactional publish. Tests: offload/restore
   round-trip, retention expiry, merge equivalence (query-before equals
   query-after), remote removal of merged inputs.
7. **The read path.** `params.rs` consts + `ROUTES`, the seven handlers,
   `metric_scan.rs` with the admission order, `query_memory` reservations,
   error prefixes in `metric_error_status`, `QueryEndpoint` variants,
   `docs/QUERY_API.md` sections (the pinning test forces this into the same
   commit). Tests in `src/query/tests.rs`: selection and bounding refusals
   with the teaching wording, rate reset handling, quantile bucket merge,
   step alignment and lookback boundaries, NDJSON shapes, headers,
   429/400/413/504 mapping.
8. **Bed completion and the gate.** The signy drivers for
   `MetricShape`, churn and explosion phases wired both-sided, both digest
   classes live, the `compare_report.rs` metrics document + its sentinel
   test, the `memory_gate.rs` metrics scenario. Verification adds a local
   one-sided smoke of `run_metrics.sh`.
9. **Run, regenerate, publish.** The full bed at `2g 8g`;
   `docs/COMPARISON_METRICS.md` regenerated from result JSON and published
   win or lose; the `VISION.md` claim annotated with the verdict; the gate
   run recorded in `MEMORY_BUDGET_GATE.md`; the acceptance checklist ticked;
   `todo.md` updated with every deferred item this plan minted.

## Acceptance checklist

- [ ] All five OTLP metric types ingest over HTTP and gRPC, decompose per
  §3, and survive restart via journal replay (replay-equals-live pinned by
  test).
- [ ] Under the optional process-wide `max_active_series` pressure, a complete
  metric export is refused with `429` + `Retry-After`, and no new series or
  WAL record is left behind. Under byte-budget pressure the same whole-export
  contract applies; the memory/cardinality counters move.
- [ ] Idle series leave the index at the timeout; a returning series works;
  the reset artifact is absorbed by `rate`.
- [ ] Metric parts flush, offload, restore, expire, and compact;
  merged-vs-unmerged query results are equal; `TenantQuota` sees metric
  bytes.
- [ ] The seven routes answer per `QUERY_API.md`; the pinning test covers
  every route and param; over-broad queries are refused before scanning with
  teaching errors; the scan headers are correct.
- [ ] The executor obeys pin → semaphore → memory pool → timeout, and
  reservations are held with the results.
- [ ] `compare/run_metrics.sh` runs both systems on identical OTLP bodies;
  churn and explosion are first-class phases; digests (both classes) gate
  every published ratio; `COMPARISON_METRICS.md` is regenerated from JSON
  with its sentinel test, "What I do not trust", and the churn table —
  published even if lost.
- [ ] The memory gate's metrics scenario passes at 2 GiB with churn, or the
  failure is the published result.
- [ ] `VISION.md` / `ARCHITECTURE.md` / `PROJECT_PLAN.md` /
  `CONFIGURATION.md` / `todo.md` updated; `cargo fmt --all -- --check`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`,
  `git diff --check` pass.

## Risks and open questions

- **Exponential-histogram downscaling is lossy.** Accepted (user decision):
  one sample kind buys the whole engine's simplicity, and the loss is
  boundary-limited quantile precision at 64 buckets. If fn0 ever needs
  tighter tails, native storage is the recorded future work; already-stored
  data does not regain resolution.
- **Rate semantics are the VictoriaMetrics definition** (user decision).
  fn0's alert math inherits it; the `QUERY_API.md` section states the
  deviation from Prometheus explicitly.
- **`max_active_series` is process-wide and opt-in.** It is an emergency
  operational guard, not the normal cardinality policy. The shared byte
  budget applies across tenants and signals, with no fixed default series
  ceiling.
- **The per-series memory estimate** is charged conservatively from measured
  allocator/container overhead rather than using a fixed count proxy; the
  live metric sample reservation is released by the journal writer after
  insertion and rolled back on an append failure.
- **Float digests may still disagree cross-engine** on rate/quantile
  shapes; the methodology's answer (withhold the ratios, say so in "What I
  do not trust") applies — the risk is a thinner published table, not a
  false one.
- **The `flush.rs` threading** is the likeliest silent-regression site;
  compile-driven, and the log/trace flush tests must pass unmodified in
  Phase 5's commit.
- **The journal replays raw OTLP through the decomposition**, so changing
  the decomposition changes what old journals replay into. The journal is
  unversioned by policy (short-lived by design), but the decomposition
  function carries a comment naming this coupling and the replay-equivalence
  test pins it.
- **VictoriaMetrics' OTLP ingest behavior** (resource-attribute promotion,
  delta handling) must be pinned in Phase 2 by inspection of the actual
  container, before the workload is frozen.
- **The compactor must precede the bed** (Phase 6 before Phase 8); if it
  slips, the bed publishes with a named deferred axis rather than silently
  measuring unmerged parts.
