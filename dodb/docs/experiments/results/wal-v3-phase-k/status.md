# Phase K Status

## Decision Status

Implementation and deterministic local tests are present. Phase K is not performance-gated. The required OCI A1 host was unreachable from this environment during this run, so the primary J/K writer-blocking diagnostic and all durable workload comparisons remain pending. The implementation must not be adopted or rejected from local laptop measurements.

## Repository Baseline

- Canonical branch: `experiment/b-link-batched-engine-monorepo`
- Canonical worktree: `/Users/namse/fn0-dodb-blink-experiment`
- Starting commit: `af95f67ccbd3e7ed6312344461b9582d77156dea`
- Source path: `dodb/`
- Main checkout was not modified.
- The requested OCI host check timed out before any remote benchmark command or file write.

## Implemented

- Four overlay segments request one background materialization. The hard in-memory limit is eight segments.
- Materialization pins the physical base and immutable overlay prefix, builds replacement B-link pages on one standard thread, writes and syncs replacement pages, then writes and syncs the checkpoint superblock before returning a publishable result.
- A writer may append newer segments while the worker builds. Publication validates the base catalog identity and exact overlay prefix and preserves the newer suffix.
- WAL v4 is reset only when the durable materialized watermark equals the current WAL tail. Prefix-only reclamation is not attempted.
- A pre-checkpoint write failure leaves the old base, overlays, and WAL authoritative and retryable. Failure after checkpoint writing starts degrades the in-memory store until reopen because the selected superblock may be uncertain.
- The phase0 benchmark accepts `phase-j` for synchronous materialization and `phase-k` for background materialization.

## Local Validation

The requested dodb release package gate passed. `dodb-storage` passed 174 library tests and 10 benchmark tests, with four existing microbenchmarks ignored; its eight recovery integration tests passed. The full command also passed the selected `dodb-core`, `dodb-testkit`, `dodb-service`, `dodb-protocol`, `dodb-client`, `dodb-server`, and `dodb-soak` unit, integration, recovery, and doc-test targets. `cargo fmt --check` passed. Deterministic tests cover commits during a paused materializer, same-key writes, suffix preservation, pinned readers, WAL retention, reopen recovery, and a pre-checkpoint materialization write failure.

This is local correctness evidence, not OCI performance or durability evidence. The full requested dodb package set and six-scenario gate have not yet run.

## Pending Gates

1. On the exact OCI A1/ZFS host, run the short 64-writer width-16 uniform J/K diagnostic. Require at least a 90% reduction in materialization-related writer-blocked time before proceeding.
2. If that passes, run three real-sync repetitions for all six required workloads against H1 and synchronous J.
3. Apply the H1 throughput and width-1 protection thresholds before sustained or RocksDB runs.
4. Run the complete dodb release correctness gate after the performance implementation settles.

See `concurrency.md`, `faults.md`, `tables.md`, and `write-amplification.md` for current evidence and limits.
