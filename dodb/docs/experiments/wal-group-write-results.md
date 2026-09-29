# WAL Group Write Results

## Motivation

After chunked catalog COW, the OCI planned width-1 control measured 12,643.78 mut/s and 840.65 ms of cumulative WAL append time per run. Sync was disabled, so the target here was WAL encoding, copies, file-length lookup, and write calls rather than durable sync latency.

## Existing Write Shape

`append_group()` received logically batched commits, but wrote each frame independently. A frame queried `file.len()` and wrote its header, payload, and trailer through separate `write_all_at()` calls. `ProductionFile::write_at()` seeks and writes. A common width-1 mutation has one data page image, one superblock image, and one COMMIT frame: up to nine `write_at()` calls per logical transaction, or about 90 calls for a ten-transaction group.

## Design

### Byte-identical group encoding

When no `FaultInjector` is configured, `encode_group()` validates all commits in order, reuses the existing page-image and frame encoders, preserves each commit's page sequence and digest, and concatenates the existing frame bytes into one buffer. LSN, batch ID, record index, page image, and size checks remain enabled.

### Single physical append

The fast path queries the end offset once, then writes the complete encoded group using one counted `write_all_at` operation. The existing helper loops for short writes. The group performs one `sync_data()` after all records, as before. In-memory commit history, scan indexes, LSNs, and batch IDs advance only after sync and the existing post-sync hook.

### Fault-injection slow path

Any configured `FaultInjector`, including a no-op injector, selects the old frame-granular path. It keeps the existing hook order and header/payload/trailer writes. The file-operation fault matrix installs a no-op injector so its intended frame-level partial-write cases continue to exercise that path.

### Recovery compatibility

No WAL format or scanner changes were made. Each input produces the same complete WAL bytes on the fast and slow paths. A torn final frame retains the existing behavior: complete earlier commits remain replayable and an incomplete final commit is not replayed.

## Correctness

Passed `cargo fmt --all -- --check`, `cargo test -p dodb-storage` (89 tests), `cargo test -p dodb-storage --bin phase0-bench` (10 tests), `cargo test --workspace`, and `git diff --check`.

Focused tests verify byte-for-byte fast/slow equivalence, one physical write and one sync for a ten-commit group, short writes capped at 137 bytes followed by reopen and scan, and a torn final commit retaining only the earlier complete commit. Existing WAL round-trip, checkpoint/reset/replay, sync-failure, planned FIFO boundary, and parallel WAL atomicity tests passed. The file-operation WAL failure matrix explicitly selects the slow path.

The local one-repetition smoke reported zero errors and overloads. It counted 1,046 WAL physical writes for 1,046 logical groups, for exactly 1.0 write/group.

## OCI Method

The binary was built in release mode at implementation commit `70105cd6d05c306c96f75587914faccef11c4099` on the two-OCPU OCI A1. Both workloads used sync-disabled mode and three repetitions. `/home/opc/dodb` is on the 30 GB XFS root filesystem; the 200 GB block device is not mounted as a filesystem. These are engine/CPU diagnostics, not durability or 200 GB volume results.

The width-1 primary used 16 writers, width 1, different-leaf-heavy, working set 100,000, cache 4096, 16-byte keys, 64-byte values, group limit 64, group byte limit 4 MiB, queue 256, delay 0, two Tokio workers, warmup 1 s, duration 2 s, and seed `0x3a042026`.

The width-16 regression check used 16 writers, width 16, uniform distribution, working set 100,000, cache 4096, sync disabled, delay 0, two Tokio workers, warmup 1 s, duration 2 s, three repetitions, and seed `0x3a032026`.

## Width-1 Primary Result

Before values are medians from the catalog-COW primary artifact; after values are medians from the three group-write records. Timings are accumulated per run, not normalized per mutation. The new run completed more mutations.

| metric | before | after | reduction |
|---|---:|---:|---:|
| mutation ops/s | 12643.78 | 15328.19 | — |
| processing | 1926.34 ms | 1907.30 ms | 1.0% |
| WAL append | 840.65 ms | 580.14 ms | 31.0% |
| group encode | n/a | 467.79 ms | n/a |
| group physical write | n/a | 97.16 ms | n/a |
| WAL sync | 0.352 ms | 0.387 ms | -9.9% |
| catalog construction | 71.59 ms | 88.72 ms | -23.9% |
| generation publication | 57.62 ms | 69.03 ms | -19.8% |
| physical execution | 494.86 ms | 599.26 ms | -21.1% |
| planning | 292.55 ms | 349.19 ms | -19.4% |

## WAL Breakdown

All width-1 repetitions recorded one physical WAL `write_at` call per logical group:

| repetition | physical writes | logical groups | writes/group |
|---:|---:|---:|---:|
| 0 | 3599 | 3599 | 1.000 |
| 1 | 3415 | 3415 | 1.000 |
| 2 | 3525 | 3525 | 1.000 |

Median group encoding was 467.79 ms; the `file.len()` plus physical group write took 97.16 ms. The append residual after subtracting those two medians was 15.20 ms. Thus the physical group write is now one call per group, while page-image/frame encoding remains most of the WAL append time.

WAL append fell by `1 - 580.14 / 840.65 = 31.0%`. Throughput speedup was `15328.19 / 12643.78 = 1.212x`.

## Width-16 Regression Check

| metric | before | after | change |
|---|---:|---:|---:|
| mutation ops/s median | 13457.64 | 14859.46 | +10.4% |

The result clears the 10% regression threshold of 12,111.88 mut/s.

## Interpretation

The fast path achieves the intended physical aggregation and is byte-identical to the fault-aware frame path. WAL append improved, and the primary throughput increased 21.2%. The WAL append ratio is 0.690 of baseline: above the 0.50 structural-success limit and below the 0.75 partial limit. This is a partial result under the predefined criteria.

## Remaining Bottleneck

Physical execution is the largest measured non-overlapping top-level serial component at 599.26 ms, slightly above WAL append at 580.14 ms. Planning is 349.19 ms. Within WAL append, group encoding remains 467.79 ms; the append residual is only 15.20 ms. No further WAL optimization or history ownership change was performed.

## Experiment Status

The B-link tree, logical batching, same-leaf coalescing, versioned reads, and WAL batching architecture are unchanged. This change only aggregates the physical writes for an already logically batched WAL group.

Phase 4 remains ineffective on the current 2-OCPU target. Phase 5 has not started, and this result does not change Phase 5 readiness.

## Artifacts

- Width-1 primary: [`planned-group-write-width1-nosync.jsonl`](results/oci-a1-2ocpu-12g-200g/wal-group-write/planned-group-write-width1-nosync.jsonl), SHA256 `e9b5968c0dfd628a2095dac394513c0275d768e086f3e4d8a5b1c5404daccf03`
- Width-16 check: [`planned-group-write-width16-nosync.jsonl`](results/oci-a1-2ocpu-12g-200g/wal-group-write/planned-group-write-width16-nosync.jsonl), SHA256 `1432d7b2c8bf756409880f9420c895a3118e683f2f40853dc901983131411533`
- Both artifacts contain three records at implementation commit `70105cd6d05c306c96f75587914faccef11c4099`; record validation passed and OCI/local SHA256 values match.
