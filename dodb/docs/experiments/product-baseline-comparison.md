# dodb Product Baseline Comparison

This document records product workload comparisons between the current dodb
B+Tree and active B-link candidate, plus external storage-engine references.
The product workload policy in `fn0-product-workload.md` is authoritative;
the synthetic matrices in `b-link-batched-engine.md` remain architecture tests.

## Comparison classes

| Comparison | Meaning |
| --- | --- |
| dodb `main-btree` vs `parallel-blink` | Semantically comparable, both use dodb's unconditional transactional mutation API and real sync. |
| SQLite WAL | Approximately comparable for point operations and simple atomic SQL transactions. SQLite allows one writer at a time. |
| Turso WAL | Approximately comparable for simple reads and unconditional transactions. WAL writers are serialized. |
| Turso MVCC | Approximately comparable for concurrent unconditional transactions. It uses snapshot isolation and its MVCC group commit implementation; conditional OCC semantics are not part of this harness. |
| RocksDB | Atomic WriteBatch storage-engine reference. It does not provide dodb condition/revision transaction semantics. |

The external adapters store the shared 16-byte logical key as a bytewise
sortable BLOB. Short Query uses an indexed key range corresponding to one
8-byte primary-key prefix and returns at most 16 rows. The dodb Query case uses
the same logical prefix and stable seeded rows. Query comparisons are therefore
approximately comparable at the logical result level; database API and row
materialization costs remain engine-specific.

## Candidate and source baselines

The active candidate is `parallel-blink`. The overlay implementation is
excluded because its recorded synchronous materialization result failed the
product guardrail. The dodb `main-btree` selector is run from the experiment
source tree to keep the two engine adapters in one harness. The experiment tree
has a small B+Tree/WAL recovery-interface delta from current `main`; this is
listed in provenance and must not be described as a bit-for-bit main build.

| Source | SHA |
| --- | --- |
| Product benchmark source | Filled from the exact benchmark checkout. |
| Current `main` reference | `d5e41bb823b55d68bad6ab5ba73b0b5113223d8b` |
| Previous accepted B-link comparison | `524d0d99375c649ea92ac273f160e724f011a4f7` |

## Comparator configuration

| Engine | Configuration |
| --- | --- |
| dodb | `main-btree` and `parallel-blink`; 16-byte keys; real sync; 16,384 4 KiB cache pages (64 MiB); primary values 512 B. |
| RocksDB | 11.8.1 (`abeebd9630f11bd08c28b7bd43c7bdfc62050654`); WAL enabled; `WriteOptions.sync=true`; one atomic WriteBatch per transaction; pipelined writes off; 64 MiB block cache. |
| SQLite | rusqlite 0.37.0 with bundled SQLite; WAL; `synchronous=FULL`; 64 MiB page cache; `WITHOUT ROWID` BLOB primary key. Exact bundled SQLite version is included in every record. |
| Turso WAL/MVCC | Turso pre-release `v0.8.0-pre.13` (`64b8ef5742fc18937f9c89806c81e3f6475dc7a3`); embedded API; `synchronous=FULL`; 64 MiB cache; WAL `BEGIN` or MVCC `BEGIN CONCURRENT`; MVCC group commit enabled. |

All benchmark database files are placed under `/bench/zfs/db`. The ZFS dataset
uses `sync=standard`, `recordsize=4K`, `compression=off`, and `atime=off`.
Durable calls must return only after each engine's configured durable commit.
Turso MVCC is an experimental reference and is not an adoption baseline.

## Workload matrix and run method

The checked-in runner is
`results/oci-a1-2ocpu-12g-zfs-crossdb/scripts/run_product_matrix.py`.
It uses a 10,000-row resident working set and a 250,000-row cache-pressure
working set. Primary values are 512 B, diagnostic values are 64 B, and focused
2 KiB writes exercise dodb's overflow-page path (the inline limit is 512 B).
The Query limit is 16. Mixed dodb clients are split between read and write
roles to approximate the requested aggregate client count, then governed by
the configured global operation ratio.

Each run uses a fixed scenario/repetition seed, at least 2 seconds of warmup,
at least 5 seconds of measurement, and three repetitions by default. Engine
order rotates per scenario and repetition. Raw JSONL records contain the
effective settings, machine fields, latency percentiles, conflicts/overloads,
and engine metrics where the adapter exposes them.

```sh
python3 scripts/run_product_matrix.py --results results/oci-a1-2ocpu-12g-zfs-product-baseline --phase all
```

The runner supports `smoke`, `core`, and `cache-pressure` phases and an
`--engine-filter` option. Executable paths can be overridden with
`PHASE0_BINARY`, `SQLITE_BINARY`, `TURSO_BINARY`, and `ROCKSDB_BINARY`.

## Results

The result table, exact machine/build provenance, repetition summaries,
artifact hashes, product guardrail assessment, external Pareto interpretation,
and architecture-stress conclusion are appended after the OCI run. A missing
or unsupported semantic comparison is shown as N/A with its reason.

## Limitations

- The dodb B+Tree selector is from the experiment checkout; its small WAL
  recovery-interface delta from current `main` is recorded above.
- External adapters cover unconditional mutations; insert-if-absent and
  revision-dependent transaction workloads are not represented by RocksDB's
  WriteBatch.
- CPU utilization is process-wide, not per-engine-thread attribution.
- The matrix is single-tenant. Multi-tenant fairness and noisy-neighbor tests
  remain a separate production-readiness gate.
