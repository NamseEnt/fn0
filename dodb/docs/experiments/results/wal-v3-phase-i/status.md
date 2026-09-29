# Phase I Status

## Scope and provenance

Phase I evaluated whether durable writes can publish committed logical mutations before producing canonical B-link leaf pages. Production WAL v3 and production write behavior were not changed. The locality and H1 measurements were built from the real Git checkout at `94cfbbe9f8403c0ff2ee0ef063c5d9daa347de90`. The benchmark-only overlay prototype was built from the real Git checkout at `42e53838e7ebabcd5f48ef1e015e0f71e2758668`. Raw benchmark records include these SHAs. OCI was a 2 OCPU AArch64 Neoverse N1 host with OpenZFS 2.2.11 and `/bench/zfs/db`.

## Decision

**2. Worth pursuing, but redesign needed.**

Model C clears the write CPU gate: the isolated logical overlay costs a median 6.0–6.5 microseconds per transaction across 0–32 existing segments, versus an H1 estimated 193 microseconds of CPU demand per transaction. That is about 3.1–3.4% of H1 CPU demand. The current implementation is not a viable read design as-is: base-fallback GET is about 1.27x with two segments and 1.51–1.59x with four; the simple Query merge is about 3.2–3.4x and Scan is about 49.7x at one segment, increasing rapidly as segments accumulate.

The next phase recommendation is exactly one: **Phase J — committed logical overlay + WAL v4**, starting with a bounded-overlay read and materialization design. Phase I does not authorize production implementation or WAL migration by itself.

## Gates

- Model A is rejected as the uniform-width16 direction: measured removable materializations are 5.09% real-sync and 5.29% sync-disabled. It helps same-leaf-heavy traffic only.
- Model B's ideal overlap ceiling is 1.37x against a CPU-plus-sync serialization model; measured H1 real-sync/disabled throughput ratio is 0.751 (1.33x). Little uncontested pipeline headroom remains, and two CPUs share worker and storage sync CPU.
- Model C theoretical removal of the inclusive physical preparation subtree is 42.78% of sampled cycles, a 1.75x ideal CPU ceiling. This is an upper bound, not durable throughput.
- Prototype write CPU / H1 estimated CPU is 0.031–0.034, passing the requested 0.70 threshold. Prototype measurements use no-op sync and are CPU feasibility only.
- Prototype semantics smoke covered ordered conditions, failed-transaction isolation, tombstones, pinned views, Query, and Scan. It is not a durability or recovery implementation.

## Artifacts

- `tables.md`: measured locality, H1, and overlay results.
- `cost-model.md`: component classification and upper-bound calculations.
- `design.md`: WAL, recovery, revision, checkpoint, and failure design analysis.
- `raw/`: raw H1, locality, and profile records.
- `perf/`: H1 perf data and reports.
- `prototype/`: isolated CPU prototype, source, raw measurements, and allocation data.
- `SHA256SUMS`: checksums for retained files.

Instrumentation is feature-gated and does not affect normal builds. Benchmark-only artifacts remain outside the production path.
