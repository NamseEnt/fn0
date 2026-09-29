# M5 Implementation Plan

M5 hardens the storage and query paths for sustained operation. Its scope is
compaction tuning, retention, configurable resource limits, observability, and
reproducible load validation. M4 must be fully verified before M5 is accepted:
the remaining fresh-context review and complete validation checklist are gates,
not M5 work.

## Current implementation constraints

- Log compaction is partition-local and selects the smallest parts. Remote
  restore/publication and merge preparation use shared lifecycle protection;
  exclusive registry replacement is limited to final revalidation.
- Merge, retention, query, metric, trace, restore, scan, memory, and scheduler
  limits are validated environment-backed `Config` fields and semaphores live
  in each `AppState`.
- Retention removes expired descriptors by manifest CAS, retires local bodies
  with active-part revalidation, and garbage-collects remote orphans only
  after `SIGNY_RETENTION_GRACE_PERIOD`.
- Trace parts support flush, lookup, restore, and cache eviction, but do not
  yet have a merge replacement path. M5 therefore applies retention and
  resource limits to traces; trace compaction is an explicit follow-up if the
  load results show that trace part growth is the limiting factor.

## Initial validation profile

The initial target profile is a release build on a 4-vCPU, 16-GiB RAM machine
with local NVMe and an S3-compatible object store. These are provisional targets
and must be confirmed or revised before the first acceptance run.

| Measure | Initial target |
| --- | --- |
| Sustained ingest | 10,000 1-KiB events/s for 30 minutes |
| Ingest acknowledgement | p95 <= 250 ms, p99 <= 1 s |
| Common one-hour log query | p95 <= 2 s |
| Selective seven-day query | p95 <= 5 s |
| Process RSS during load | <= 4 GiB |
| Non-limit query error rate | 0% |
| Retention catch-up | <= 10 minutes after data becomes eligible |

## Implementation phases

### 1. Configuration and observability

Add validated environment-backed configuration for merge, retention, query,
metric, trace, and remote-restore limits. Pass the runtime limits and
per-instance semaphores through `AppState` instead of using process-global
constants.

Expose counters and latency measurements for ingest, flush, merge, retention,
query scans, remote restores, cache eviction, WAL backlog, and active part
counts. The Prometheus-compatible `/metrics` endpoint and `/ready` health
state expose these signals.

### 2. Compaction tuning

1. Add size-aware merge selection and limits based on both row count and
   estimated bytes.
2. Add a per-tick merge budget and avoid repeatedly rewriting already-large
   parts.
3. Separate merge preparation from the short registry replacement section:
   read and write output while inputs remain valid, acquire the lifecycle lock
   for final revalidation and commit, then clean up old parts.
4. Preserve tombstone, manifest CAS, and restart recovery guarantees.
5. Bound merge memory and record lock hold time, input/output sizes, and merge
   debt for tuning.

### 3. Retention

Add a periodic retention worker for logs and traces. A part is eligible only
when its metadata proves that all contained data is older than the cutoff; the
boundary part is retained rather than partially deleted.

For object storage, remove expired descriptors using a manifest CAS, update the
local registry under the lifecycle lock, and delete local and remote data only
after the new manifest is durable. Orphaned remote objects remain recoverable
until a grace-period garbage-collection pass. Local-only mode must use the same
active-part safety checks.

Use an injectable clock in tests and cover crashes before and after manifest
replacement, merge/retention races, restart recovery, and protection of active
or young parts.

### 4. Resource limits

Make the following limits configurable and enforce them before and during
execution:

- maximum log and metric time range/lookback;
- physical scan rows and bytes;
- materialized query memory;
- log output rows, metric points/series/samples, and trace spans;
- per-endpoint concurrency;
- query and object-store restore runtime.

Count approximate bytes for log entries, structured metadata, metric series,
and trace spans. Reject resource violations with clear client errors, return
timeouts consistently, and always cancel blocking work before releasing query
permits and lifecycle guards.

### 5. Load validation

Add a reproducible load tool that mixes Loki push, OTLP ingest, log queries,
metric queries, Tempo lookups, cache eviction, remote restores, merges, and
retention. Each run records the seed, workload, build revision, machine
profile, latency percentiles, throughput, RSS, WAL backlog, part counts, merge
debt, restore activity, and errors.

The final report must compare the baseline and tuned implementation, identify
the bottleneck, and record whether every acceptance target was met.

## Acceptance checklist

- [x] M4 review and complete validation are green.
- [x] Merge parameters and query/retention limits are configurable and
  validated.
- [x] Compaction keeps query disruption and merge memory within the agreed
  target while preserving restart and object-store correctness.
- [x] Expired log and trace parts are removed without deleting active or
  boundary data; local and remote manifests remain recoverable across crashes.
- [x] Query range, scan, memory, output, concurrency, and timeout limits are
  enforced consistently for Loki and Tempo APIs.
- [ ] The reproducible load run meets the agreed throughput, latency, memory,
  retention, and error-rate targets.
- [ ] Focused tests, `cargo test --all-targets`, formatting, Clippy, and
  `git diff --check` pass.
- [ ] Load results and identified bottlenecks are documented.

## Verification so far

- `cargo fmt --all -- --check` passed.
- `cargo test --all-targets` passed with 200 tests.
- `cargo clippy --all-targets -- -D warnings` passed.
- `git diff --check` passed.
- A temporary local server smoke run completed 15 push batches (150 events)
  and 7 query requests with zero push or query errors. The measured mean push
  latency was approximately 199 ms and mean query latency approximately 6 ms.
- `src/bin/m5_load.rs` now produces seeded mixed Loki/OTLP/metric/Tempo
  workload results with p50/p95/p99 latency, RSS peak, build revision, and
  `/metrics` status. The sustained S3 load run was not executed: this
  workspace has neither S3 credentials nor an S3-compatible deployment/test
  endpoint, so remote restore, retention GC, and long-running object-store
  behavior could not be measured meaningfully here. It remains an
  environment-level validation item.
