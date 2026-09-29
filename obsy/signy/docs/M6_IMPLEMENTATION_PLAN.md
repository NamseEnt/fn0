# M6 Implementation Plan

M6 makes planned machine replacement lossless. A running instance must react to
SIGTERM by refusing new ingest, draining in-flight acknowledged requests,
force-flushing the MemTable to the object store, and only then exiting, so the
disk can be discarded and a fresh machine can resume the same endpoint without
losing any acknowledged data. M5 remains partially open (the sustained S3 load
run), but its acceptance gate is orthogonal to M6 and does not block this work.

The behavior M6 implements is already specified in `docs/ARCHITECTURE.md`
(the graceful-shutdown procedure and the machine-replacement milestone row).
This plan records the execution design and the acceptance checklist.

## Current implementation constraints

- `startup::run` ends with `axum::serve(...).await` and a spawned tonic
  `Server::builder()...serve(addr).await`. Neither installs a signal handler, so
  a SIGTERM tears the tokio runtime down immediately and drops any MemTable and
  WAL suffix that was not yet flushed. This is exactly the loss window M6 removes
  for planned replacement.
- `Journal::append` / `append_trace` return only after the record is fsynced, so
  an HTTP 204 or an OTLP success is already a durability acknowledgement.
  "Draining in-flight requests" is therefore equivalent to letting in-flight
  HTTP and gRPC handlers run to completion.
- `flush::flush_once` already performs part write, object-store upload, manifest
  CAS, and checkpoint advance, but it is private to `flush_loop`, and the
  size/interval thresholds that decide whether to flush live in the loop. A
  forced-flush path can reuse `flush_once` directly, bypassing the thresholds.
- `flush_loop`, `merge_loop`, and the retention/eviction workers run
  indefinitely and coordinate on the registry `operation_lock`. Shutdown must
  stop them cleanly so the final force-flush is the only writer.
- `/ready` already aggregates `AtomicBool` health flags from `AppState`, which is
  the natural place to surface drain status.

## Design

The procedure follows the four steps in `docs/ARCHITECTURE.md`: block ingest,
drain in-flight, force-flush to S3 + manifest, then exit.

### 1. Shutdown state and signal source (`src/shutdown.rs`)

- Introduce a shared `ShutdownState` carrying `draining: AtomicBool`,
  `force_flush_complete: AtomicBool`, and `pending_flush_bytes: AtomicU64`, added
  to `AppState`.
- Build a shutdown future from `tokio::signal::unix::signal` for SIGTERM and
  SIGINT. The first signal starts the drain sequence; the sequence itself owns
  process exit.

### 2. Block ingest (reject new requests immediately)

- `ingest::push` checks `draining` at entry and returns `503 Service
  Unavailable` before touching the journal.
- `trace_ingest::TraceIngestService` returns gRPC `UNAVAILABLE` under the same
  condition.
- Because the record is never acked, Alloy (and any OTLP client with a WAL/queue)
  retries against the replacement machine. The narrow duplicate window described
  in `docs/ARCHITECTURE.md` (a connection dropped just before ack) remains the
  accepted at-least-once behavior; no data is lost.

### 3. Drain in-flight requests

- Replace `axum::serve(listener, app).await` with
  `axum::serve(...).with_graceful_shutdown(signal)` so the HTTP server stops
  accepting and waits for in-flight handlers to finish their append/ack.
- Replace the tonic `serve(addr)` with `serve_with_shutdown(addr, signal)` for
  the same drain guarantee on OTLP.
- The shutdown orchestrator waits for both servers to finish draining before
  moving to force-flush.

### 4. Stop background workers, then force-flush

- Signal `flush_loop`, `merge_loop`, retention, and cache-eviction loops to exit
  their loops (a shared shutdown signal / `Notify`), so no background task races
  the final flush on the registry or the object store.
- Add a `force_flush` entry point in `flush.rs` that calls the existing
  `flush_once` repeatedly, ignoring the size/interval thresholds, until both the
  log and trace MemTables are empty and any `pending_checkpoint` has been
  drained. Completion requires the part upload, manifest CAS, and checkpoint
  advance to all succeed.
- On each pass, publish progress into `ShutdownState.pending_flush_bytes`; set
  `force_flush_complete = true` once the MemTables are empty and durable.

### 5. Persistent retry with operator-gated exit (no hard timeout)

Machine replacement discards the disk, so a force-flush that gives up would lose
data. M6 therefore does **not** impose a hard timeout that auto-exits.

- `force_flush` retries indefinitely with bounded backoff. A transient
  object-store or checkpoint failure is retried, not surfaced as terminal.
- If retries keep failing past a warning threshold, print a prominent, repeating
  message to stdout stating that force-flush is still failing, data is not yet
  durable, and the process will not exit on its own. The operator must send an
  explicit stdin confirmation to force termination; absent that input, the
  process keeps retrying.
- If the object store recovers while retrying, the flush succeeds and the process
  exits cleanly on its own — no operator input needed.
- The WAL and checkpoint are left intact until force-flush confirms durability.
  A forced/manual exit before durability therefore recovers automatically on the
  **next start** through the normal journal replay and
  `reconcile_flush_transaction` path (restart, not machine replacement). Restart
  auto-recovery is an explicit acceptance item.

### 6. Drain-status readiness

- Once `draining` is set, `/ready` returns `503` so the load balancer / orchestrator
  stops routing to this instance; the body names the drain state and remaining
  `pending_flush_bytes`.
- Expose the same signals on `/metrics`: `drain_in_progress`,
  `pending_flush_bytes`, and `force_flush_complete`, so an external controller can
  confirm the flush finished before decommissioning the disk.

### 7. Machine-replacement rehearsal

- Add an integration rehearsal (in `src/tests/` or `e2e.rs`) using the in-memory
  or local-file object store: ingest logs and traces, send SIGTERM, then assert
  (a) new pushes return 503 and OTLP returns UNAVAILABLE, (b) already-acked
  requests are durable, (c) force-flush published the parts and updated the
  manifest, (d) the process exits cleanly, and (e) a fresh instance started
  against the same object store (simulating the replacement machine) returns all
  acked data with zero loss and zero duplication beyond the accepted window.
- Cover the retry path: a failing object store keeps the process alive and
  recovers when the store returns; a restart after a manual exit replays the WAL
  and loses nothing.
- As with M2-M5, live S3 is not exercised here (no credentials / endpoint in this
  workspace); the in-memory and local-file backends exercise the same
  object-store contract, and live S3 remains an environment-level check.

## Acceptance checklist

- [x] SIGTERM/SIGINT starts the drain sequence and the sequence owns process exit.
- [x] While draining, Loki push returns 503 and OTLP returns UNAVAILABLE before any journal append.
- [x] In-flight acknowledged HTTP and gRPC requests complete before force-flush begins.
- [x] Background flush/merge/retention/eviction workers stop before the final force-flush.
- [x] Force-flush drains both MemTables and any pending checkpoint to the object store and manifest, ignoring size/interval thresholds.
- [x] Persistent object-store failure keeps retrying, warns on stdout, and exits only on explicit operator input; recovery mid-retry finishes and exits cleanly.
- [x] A manual/forced exit before durability recovers with zero loss on the next start via journal replay.
- [x] `/ready` returns 503 while draining and `/metrics` exposes drain progress, pending bytes, and force-flush completion.
- [x] The machine-replacement rehearsal shows a fresh instance resuming with zero acknowledged-data loss.
- [x] A fresh-context review reports no blocking findings; the observability/robustness findings it raised (distinct operator-abort exit code and log, retry/abort test coverage, `Ok(false)` yield, stalled-store/non-TTY documentation) are remediated.
- [x] Focused tests, `cargo test --all-targets`, `cargo fmt --all -- --check`, Clippy with warnings denied, and `git diff --check` pass.

## Implementation notes

- `src/shutdown.rs` holds `ShutdownState` (draining flag, force-flush progress,
  and a `watch` drain signal), the SIGTERM/SIGINT future, and `finalize_flush`
  (the persistent-retry, operator-gated force-flush driver).
- `flush::force_flush_pass` reuses the existing `flush_once`/checkpoint logic for
  one threshold-free pass; `finalize_flush` loops it with bounded backoff.
- `startup::run` installs the signal handler, drives the HTTP server with
  `with_graceful_shutdown` and the OTLP server with `serve_with_shutdown`, joins
  every background worker, then calls `finalize_flush` before returning.
- `finalize_flush` returns a `ShutdownOutcome`. A `Durable` result logs a clean
  completion; an `AbortedByOperator` result logs that data is only on the WAL and
  `startup::run` exits with a non-zero code, so an automated controller never
  mistakes an operator-forced exit for a durable shutdown and discards the disk.
  The retry loop lives in `finalize_flush_with_abort`, which takes an injectable
  abort source so tests can drive the operator-abort path without real stdin.
- The drain signal reaches workers through `watch::Receiver<bool>`;
  `shutdown::wait_for_drain` treats a dropped sender as "never drains" so the
  existing single-worker test fixtures keep running.
- `SIGNY_SHUTDOWN_FLUSH_WARN_AFTER` (default 30s) controls when the operator
  prompt appears.

## Verification

- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --all-targets` (205 tests), and `git diff --check` all pass.
- New tests: `m6_machine_replacement_force_flush_is_lossless` (drain →
  force-flush → fresh instance restores all acknowledged logs and traces from
  the object store with zero loss), `m6_draining_rejects_new_ingest_and_readiness`
  (push 503 + `/ready` 503 while draining),
  `export_rejects_while_draining_for_shutdown` (OTLP UNAVAILABLE),
  `m6_force_flush_retries_until_object_store_recovers` (a fault-injecting object
  store fails the first writes, then the retried force-flush publishes durably and
  reports `Durable`), and `m6_operator_abort_preserves_wal_for_restart_recovery`
  (an always-failing store plus an injected operator abort returns
  `AbortedByOperator`, leaves `force_flush_complete=false` with non-zero pending
  bytes, and a WAL replay recovers the unflushed data on the next start).
- Live smoke run against a file-backed object store: `/ready` returned 200, a
  SIGTERM drained the servers, stopped the workers, force-flushed to the object
  store (`force-flush complete ... attempts=1`), and the process exited 0 in
  about one second.
- Live S3 is not exercised here (no credentials / endpoint in this workspace);
  the in-memory and file backends exercise the same durability contract, and
  live S3 remains an environment-level check.
