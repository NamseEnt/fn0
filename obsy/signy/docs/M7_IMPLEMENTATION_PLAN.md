# M7 Implementation Plan

> **Superseded in part (2026-07-26).** Tier C (local MinIO over the real S3
> protocol) has been removed. Its stated purpose — validating conditional put,
> path-style addressing, and multipart — is `object_store`'s responsibility, not
> this repository's, and re-verifying a dependency was never our job. The
> replacement is finer-grained testing on our side of the crate boundary plus a
> boot-time preflight on the other. See [`LOAD_VALIDATION.md`](LOAD_VALIDATION.md).

M7 closes the load-validation gate that M5 left open, using only a local
developer machine. It does so without real cloud S3 by driving the engine under
two complementary object-store backends:

- **Tier B — deterministic in-process fault/latency injection.** A wrapper
  `ObjectStore` adds configurable latency, jitter, and error rate around an
  in-memory or local-file backend. It runs inside the real server binary (over
  the real HTTP/gRPC paths), is seeded and reproducible, needs no external
  process, and is the CI-friendly load gate.
- **Tier C — real S3 protocol against a local MinIO.** The unchanged
  `object_store` `AmazonS3` code path talks HTTP to a MinIO container
  (conditional put, path-style, multipart, network latency). This validates the
  actual production code path — manifest CAS, remote restore, retention GC — that
  the in-memory and file backends never exercise.

Docker is assumed to be available in the development environment, so Tier C runs
locally via `docker compose`.

M7 also fixes an observability gap discovered during survey: `RuntimeMetrics`
today exposes only monotonic counters. Load analysis needs point-in-time gauges
(active part count, WAL backlog bytes, MemTable bytes, merge debt), so the load
report can attribute latency/backpressure to storage state.

The behaviors under test are already specified in `docs/ARCHITECTURE.md` and the
M5 targets table (`docs/M5_IMPLEMENTATION_PLAN.md`). This plan records the
execution design and the acceptance checklist. It defines no new engine feature;
the deferred LogQL/storage work in `todo.md` (P1/P2) stays out of scope.

## Current implementation constraints

- `ObjectStorage::from_url` builds the backend through
  `object_store::parse_url_opts`, and `normalized_object_store_options` forwards
  every `AWS_*` and `OBJECT_STORE_*` environment variable to it. Pointing at a
  local MinIO therefore needs **no code change** — only URL + env
  (`SIGNY_OBJECT_STORE_URL=s3://bucket/prefix`, `OBJECT_STORE_ENDPOINT`,
  `OBJECT_STORE_ALLOW_HTTP=true`, credentials).
- Manifest CAS relies on `PutMode::Create` (If-None-Match). AWS S3 supports this
  natively, but a local MinIO may require `OBJECT_STORE_CONDITIONAL_PUT=etag`.
  Confirming this against MinIO is an explicit Tier C task, not an assumption.
- `ObjectStorage::from_store(Arc<dyn ObjectStore>, prefix)` already exists as a
  test injection hook (used by `shutdown_rehearsal.rs`), but it is
  `#[cfg(test)]` and only wraps an in-process store. The real server binary
  builds its store only from `from_url`, so Tier B needs a production-reachable
  way to wrap the constructed store.
- `RuntimeMetrics` (`src/metrics.rs`) holds only monotonic `AtomicU64` counters.
  There is no gauge for active parts, WAL backlog, MemTable size, or merge debt,
  which the M5 plan itself lists as required load signals.
- `src/bin/m5_load.rs` is an **open-loop** client: it pushes as fast as the
  server accepts, then reports p50/p95/p99 and peak RSS. It does not offer a
  fixed target rate, has no warmup/steady-state split, does not force
  eviction→restore, and does not evaluate results against the target table.
- No `docker-compose.yml`, load script, or results document exists yet.

## Design

### Phase 0 — Observability gauges (small engine change)

Add point-in-time gauges so the load report can explain latency by storage
state. Extend `RuntimeMetrics` with:

- `active_log_parts` / `active_trace_parts` (registry sizes),
- `wal_backlog_bytes` (journal end offset minus checkpoint),
- `memtable_bytes` (log + trace in-memory footprint),
- `merge_debt_parts` (parts eligible for but not yet merged).

These are updated where the underlying state already changes (flush, merge,
retention, checkpoint advance, ingest) and rendered by `query::metrics`. Gauges
are read at the end of a load run to snapshot terminal storage state; counters
remain monotonic. Keep cardinality flat — no per-part or per-partition labels.

### Phase 1 — Tier B: in-process latency/fault-injecting store

Add `src/object_storage/fault_store.rs` with a `LatencyFaultStore<S>` that wraps
any `Arc<dyn ObjectStore>` and, per operation:

- sleeps `base + U(0, jitter)` (deterministic from a seeded RNG keyed by
  operation counter, so a fixed seed replays identically),
- optionally returns a retriable error at a configured probability, exercising
  the engine's own retry/backpressure paths,
- distinguishes read vs. write latency so restore and flush can be shaped
  independently.

Make it reachable from the **real binary** (not just tests): in
`ObjectStorage::from_url`, after building the inner store, wrap it when the load
knobs are present:

- `SIGNY_OBJECT_STORE_LATENCY_MS`, `_LATENCY_JITTER_MS`,
  `_READ_LATENCY_MS`, `_ERROR_RATE`, `_FAULT_SEED`.

When none are set, `from_url` behaves exactly as today (zero overhead, no
wrapper). This keeps a single object-store construction path and lets the
existing `m5_load` client drive the actual HTTP/gRPC server over an S3-like
backend with reproducible latency, without Docker.

Tier B uses an in-memory or `file://` inner store, so it validates
latency-driven backpressure, flush/restore/merge/retention scheduling, RSS under
sustained ingest, and injected-error recovery — deterministically and in CI.

### Phase 2 — Tier C: local MinIO over the real S3 protocol

Add `docker-compose.yml` with a MinIO service (console + S3 port) and a
one-shot `mc`/init step that creates the bucket. Add `scripts/run_load_s3.sh`
that:

1. `docker compose up -d minio` and waits for readiness,
2. creates the bucket,
3. launches the release server with
   `SIGNY_OBJECT_STORE_URL=s3://<bucket>/signy` and the MinIO endpoint
   env (`OBJECT_STORE_ENDPOINT`, `OBJECT_STORE_ALLOW_HTTP=true`,
   `OBJECT_STORE_VIRTUAL_HOSTED_STYLE_REQUEST=false`, `AWS_ACCESS_KEY_ID`,
   `AWS_SECRET_ACCESS_KEY`, and `OBJECT_STORE_CONDITIONAL_PUT=etag` if manifest
   CAS needs it),
4. runs the load tool against it,
5. scrapes `/metrics`, tears down.

First, confirm manifest CAS works against MinIO (a competing conditional put is
rejected). If it does not without `OBJECT_STORE_CONDITIONAL_PUT`, document the
required setting; if `object_store` 0.12 cannot express it, that is an
environment finding to record, not a silent pass. Tier C is what actually
exercises `AmazonS3`: conditional put, multipart upload for large parts,
path-style addressing, and real socket latency for eviction→restore round trips.

### Phase 3 — Closed-loop workload harness

Evolve `src/bin/m5_load.rs` (or add `src/bin/load.rs` superseding it) so results
are comparable to the target table:

- **Target-rate pacing.** Offer a fixed `SIGNY_LOAD_TARGET_EPS`
  (events/s), sleeping to hold the rate, so ack p95/p99 is measured at a defined
  offered load rather than open-loop saturation. Report achieved vs. offered
  rate.
- **Warmup / steady-state split.** Exclude a warmup window from the reported
  percentiles so cold-cache and first-flush effects do not skew steady-state
  latency.
- **Forced eviction→restore.** Configure a small `SIGNY_CACHE_MAX_BYTES`
  and query time ranges old enough to have been evicted, so remote-restore
  latency is actually on the measured path (this is the S3 behavior in-memory
  runs miss).
- **Retention catch-up measurement.** Set a short retention period and measure
  time from eligibility to reclamation via the retention gauges/counters.
- **End-of-run `/metrics` capture.** Record the Phase 0 gauges plus
  flush/merge/retention/restore/eviction counters into the result JSON.
- **Explicit pass/fail.** Compare measured percentiles, RSS, error rate, and
  retention catch-up against the target table and emit an overall verdict.

### Phase 4 — Run, evaluate, document

1. Reconfirm or revise the M5 provisional targets before the first accepted run.
2. Record the **machine profile**. A developer laptop is not the 4-vCPU/16-GiB
   reference machine, so absolute throughput targets are validated on
   target-class hardware; the local runs assert error-free operation, bounded
   RSS relative to the configured cap, correct backpressure, and successful
   eviction→restore and retention, and record the achieved numbers.
3. Run Tier B (deterministic, seeded) and Tier C (MinIO) with the same workload.
4. Write `docs/M7_LOAD_RESULTS.md`: seed, build revision, machine profile, both
   tier configs, latency percentiles, throughput, peak RSS, terminal gauges,
   restore/merge/retention activity, error counts, the identified bottleneck,
   and per-target pass/fail. Include a baseline-vs-tuned comparison if a tuning
   change is made in response to a bottleneck.

## Acceptance checklist

- [ ] `RuntimeMetrics`/`/metrics` expose active part counts, WAL backlog,
  MemTable bytes, and merge debt as point-in-time gauges.
- [ ] `from_url` optionally wraps the constructed store in `LatencyFaultStore`
  when load knobs are set, and is a zero-cost no-op otherwise.
- [ ] Tier B: a seeded, reproducible run drives the real server over a
  latency/fault-injected backend and recovers from injected object-store errors
  with zero acknowledged-data loss.
- [ ] `docker-compose.yml` brings up MinIO and `scripts/run_load_s3.sh` runs the
  server against it end to end.
- [ ] Tier C: manifest CAS, remote restore (eviction→restore), and retention GC
  are confirmed against MinIO over the real S3 protocol; any required
  `OBJECT_STORE_*` setting is documented.
- [ ] The load harness paces to a target rate, splits warmup from steady state,
  forces eviction→restore, and emits an explicit per-target pass/fail verdict.
- [ ] A run meets the agreed throughput, latency, memory, retention, and
  error-rate targets on target-class hardware; the local runs are error-free
  with bounded RSS and correct backpressure, and the achieved numbers are
  recorded.
- [ ] `docs/M7_LOAD_RESULTS.md` records both tier results, the machine profile,
  the identified bottleneck, and per-target pass/fail.
- [ ] The M6 fresh-context review gate is cleared (carried over; blocks final
  acceptance).
- [ ] A fresh-context review reports no blocking findings.
- [ ] Focused tests, `cargo test --all-targets`, `cargo fmt --all -- --check`,
  Clippy with warnings denied, and `git diff --check` pass.

## Risks and open questions

- **MinIO conditional put.** The single biggest unknown is whether `PutMode::
  Create` manifest CAS works against MinIO out of the box or needs
  `OBJECT_STORE_CONDITIONAL_PUT=etag`. Verified first in Phase 2; if unsupported,
  it is a recorded environment finding.
- **Target realism on a laptop.** Absolute ingest/latency targets are
  hardware-bound. M7 separates "meets the numeric target" (target-class
  hardware) from "behaves correctly under sustained load" (local), and records
  the machine profile so results are interpretable rather than falsely
  pass/fail.
- **Determinism of Tier B.** Latency/error injection is seeded so a run is
  replayable, but wall-clock scheduling still varies; percentiles are reported
  as distributions, not asserted to exact values, in tests.
- **Scope discipline.** M7 is validation + observability only. If a run reveals a
  real bottleneck requiring an engine change (for example trace-part merge, or
  streaming metric evaluation), that change is scoped as its own follow-up, not
  folded into M7 silently.

## Implementation notes

- New/changed files (planned): `src/object_storage/fault_store.rs`
  (`LatencyFaultStore`), `src/object_storage/catalog.rs` (opt-in wrap in
  `from_url`), `src/config.rs` (fault + load knobs), `src/metrics.rs` +
  `src/query/handlers.rs` (gauges), `src/bin/load.rs` (closed-loop harness,
  superseding or extending `m5_load.rs`), `docker-compose.yml`,
  `scripts/run_load_s3.sh`, `docs/M7_LOAD_RESULTS.md`.
- Live cloud S3 (AWS) remains out of scope; MinIO is the local stand-in for the
  real S3 protocol, and a real-AWS run stays an environment-level deployment
  check as in M2–M6.
