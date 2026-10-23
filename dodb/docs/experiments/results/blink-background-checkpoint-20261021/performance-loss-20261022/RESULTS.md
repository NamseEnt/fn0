# Why B-link lost throughput from its fastest measured path

## Finding

The large loss is reproduced with the same unchanged executable: adding finite-WAL checkpointing reduces E from **2,706 to 1,387 batches/s (-48.8%)**. This is not explained by the small segmented synchronous/background difference. Checkpointing adds serialized database flush/sync work and repeatedly clears the WAL page-delta history, forcing full-page bootstrap images again. Observed E WAL append volume increases from **8,827 to 22,833 bytes per successful batch (2.59x)** while grouping stays near 54 transactions.

A measured policy improvement exists: 512 MiB instead of 128 MiB recovers **16.45% in RW80** and **30.62% in E**, with actual checkpoints and successful final/reopen validation. It permits four times the WAL trigger size and worsens rare E pauses to approximately four seconds. No engine code or default was changed. This is a useful diagnosis and a bounded-WAL trade-off, not a completed structural optimization.

## Uninstrumented matrix

One frozen implementation (`d240416f0e0efafa7c08bb1e868387ebd0d894bc`), four paths, two workloads, three rotating repetitions, **60 seconds after ten-second warmup**: 24/24 accepted. Four separate preflight cells also passed. Complete source/binary/controller identities, exact argv, and validators are preserved. The implementation was previously validated by 314 passing checks; this investigation does not claim to rerun those unchanged-engine checks.

RW80 is 80% read / 20% write, 16 dedicated callers. E is unconditional width-16 write batches, 64 dedicated callers. Values are 512 bytes, fixed seed 20261013, synchronous WAL/durability retained. The RocksDB reference uses the DODB contract with logical revisions; it is not raw WriteBatch or maximum-engine throughput. RW80 reports API calls/s; E reports write batches/s. Throughput includes observed final completion and checkpoint drain.

### RW80 medians

| Path | Throughput/s | Three-run range | p50 ms | p95 ms | p99 ms | p99.9 ms | Maximum ms |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| RocksDB minimal-native | 13,282.53 | 13,059.06–13,286.64 | 0.012 | 2.996 | 29.671 | 85.721 | 636.486 |
| B-link, checkpoint off | 17,274.81 | 17,040.72–17,323.07 | 0.013 | 5.542 | 7.774 | 11.141 | 55.542 |
| B-link, sync 128 MiB | 13,826.80 | 13,439.36–14,062.70 | 0.014 | 6.509 | 9.396 | 13.500 | 795.869 |
| B-link, sync 512 MiB | 16,101.82 | 16,002.32–16,518.24 | 0.014 | 5.796 | 8.221 | 12.345 | 1094.713 |

### E medians

| Path | Throughput/s | Three-run range | p50 ms | p95 ms | p99 ms | p99.9 ms | Maximum ms |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| RocksDB minimal-native | 1,712.40 | 1,688.39–1,752.54 | 30.671 | 57.082 | 335.282 | 419.168 | 520.837 |
| B-link, checkpoint off | 2,706.20 | 2,698.07–2,712.57 | 21.217 | 38.601 | 44.958 | 59.245 | 92.679 |
| B-link, sync 128 MiB | 1,386.69 | 1,379.03–1,387.25 | 28.574 | 55.083 | 1253.048 | 1492.124 | 1513.868 |
| B-link, sync 512 MiB | 1,811.36 | 1,810.60–1,816.14 | 25.166 | 50.364 | 67.764 | 3850.371 | 3888.333 |

Percentiles are medians of three per-run percentiles, not pooled or summed percentiles. E maxima are exact per-run maxima; RW80 maxima are histogram upper bounds. The maximum column is the median of per-run maxima. The worst E 512 MiB sample across all three runs is **4,033.815 ms**. p99 alone would hide this trade-off: E p99 improves from 1,253 to 68 ms, but p99.9 worsens from 1,492 to 3,850 ms.

Relative to the current RocksDB reference, the 512 MiB path is **+21.23% in RW80** and **+5.78% in E**. Its RW80 gain is workload-specific; E is not a broad or large throughput win and has much worse rare latency. Checkpoint-off is faster but allows WAL to grow throughout the run and defers periodic cleanup.

## Work added by checkpointing

| Workload / policy | WAL bytes/write transaction | Transactions/group | Checkpoint wall s | Checkpoint share | Completed checkpoints r1/r2/r3 |
| --- | ---: | ---: | ---: | ---: | --- |
| RW80, off | 3,389 | 8.83 | 0.000 | 0.00% | 0/0/0 |
| RW80, sync-128m | 4,940 | 8.86 | 3.930 | 6.55% | 6/6/6 |
| RW80, sync-512m | 3,784 | 8.87 | 1.083 | 1.80% | 1/1/1 |
| E, off | 8,827 | 53.79 | 0.000 | 0.00% | 0/0/0 |
| E, sync-128m | 22,833 | 53.55 | 18.597 | 30.98% | 14/14/14 |
| E, sync-512m | 15,660 | 53.27 | 11.443 | 19.07% | 3/3/3 |

WAL bytes above are **appended bytes**, reconstructed as ending retained bytes minus starting retained bytes plus reclaimed bytes and reset reinitialization bytes, divided by successful mutation transactions. They are not retained file size. Checkpoint wall time is serialized with mutations in this legacy synchronous comparison. Similar group sizes rule out a collapse in transaction batching as the explanation for this large loss.

Subtracting checkpoint wall from elapsed time only provides descriptive accounting: the E 128 MiB rate still remains below checkpoint-off after this adjustment. It does not measure a counterfactual engine or assign every remaining microsecond to WAL size. The independent evidence for extra WAL work is the byte conservation result and the actual redo/recovery code path.

### Verified code mechanism

1. `WalLog::reset` (`crates/dodb-storage/src/wal.rs`, lines 857–923) durably truncates and reinitializes WAL, then starts with an empty `page_chain`.
2. `plan_redo` (lines 1666–1755) checks for a known page base. A first touch without one selects a complete page image; later eligible updates can use smaller page deltas. Repeated checkpoints therefore repeat the bootstrap cost across touched pages.
3. `BlinkStore::checkpoint_inner` (`crates/dodb-storage/src/blink/mod.rs`, lines 3753–3807) flushes dirty pages, syncs data, publishes and syncs the new superblock, resets/syncs WAL, and checks invariants. `flush_dirty` writes dirty pages individually in 4 KiB writes.
4. Recovery (`wal.rs`, around lines 3385–3392) explicitly rejects a delta with no full-image base in retained WAL history. Merely preserving the in-memory map across reset is unsafe. A safe design must also cover crash-interrupted database writes/checkpoint publication, torn pages, page reuse, and independently available durable redo bases.

## Diagnostic profile

The accepted follow-up starts E/128 MiB under inherited syscall tracing, with the workload still executing as `opc`. Its **1,825,891,189 traced WAL bytes exactly equal** independently reconstructed physical WAL append bytes. The trace spans the entire measured window, has zero unpaired measured entry/exit events, and retains successful final/reopen validation. These timings are diagnostic and excluded from the 24-cell medians.

| Accepted E/128 MiB diagnostic, 60.049 s | Observed work / time |
| --- | ---: |
| Completed checkpoints | 13 |
| Checkpoint wall time | 16.501 s |
| Database page writes | 409,835 calls; 1,678,684,160 bytes |
| Database writes inside checkpoints | 3.863 s |
| Database syncs inside checkpoints | 7.919 s |
| WAL write/sync inside checkpoints | 0.044 s |
| Union of traced syscalls inside checkpoints | 11.826 s |
| Remaining checkpoint wall, not attributed by these syscalls | 4.676 s |

Database writes/sync therefore account for **71.7% of observed checkpoint wall** in this diagnostic. The remainder includes userspace work and any operations outside the selected syscalls; it is not automatically all invariant-check CPU. WAL append work also occurs between checkpoints: this same run records 22,832 WAL bytes per batch, consistent with the uninstrumented amplification. Merely moving checkpoint work to another thread does not remove these bytes.

The first attach-after-start diagnostic is preserved separately. Its off trace exactly matches the WAL accounting, but its 128 MiB trace captures only 1,188,746,425 of 1,829,235,609 expected WAL bytes (65.0%), despite no lost-event warning. **Reject its complete time attribution**. Its workload outcome/reopen checks still passed. Do not infer an exact reason for the missing trace events solely from this result; the subsequent launch-under-tracing run eliminates the observed byte mismatch. Both attempts and the failed strict-check log remain archived. CPU samples are preserved without a full-process CPU attribution claim.

## Resource cost

| Workload / path | Peak process-group RSS MiB | Maximum sampled WAL MiB | CPU seconds |
| --- | ---: | ---: | ---: |
| RW80, RocksDB minimal-native | 394.7 | 61.4 | 33.83 |
| RW80, B-link, checkpoint off | 767.4 | 824.2 | 34.29 |
| RW80, B-link, sync 128 MiB | 530.5 | 128.1 | 31.35 |
| RW80, B-link, sync 512 MiB | 701.1 | 512.0 | 33.70 |
| E, RocksDB minimal-native | 974.6 | 64.4 | 50.59 |
| E, B-link, checkpoint off | 1423.9 | 1971.0 | 46.76 |
| E, B-link, sync 128 MiB | 1112.2 | 129.3 | 34.75 |
| E, B-link, sync 512 MiB | 1444.0 | 512.4 | 36.26 |

Resource columns are medians of per-run sampled peaks; CPU is the workload report metric and includes phases beyond the admission window. In particular RSS includes seeding, not an established steady-state footprint. Thresholds are soft triggers, not hard byte caps, and concurrent group completion may overshoot them. The 512 MiB policy is not an equal-resource optimization. No common complete-engine memory budget was imposed.

## Improvement opportunities and limits

- **Measured, available policy:** increase the trigger to 512 MiB. Both workloads recover useful throughput while actually paying checkpoint cost. The cost is more WAL/RSS and longer rare pauses. RW80 only completes one checkpoint in each 60-second cell, so it does not establish steady-state behavior or justify changing defaults. E completes three per cell.
- **Structural WAL/recovery target:** avoid repeatedly paying full-image bootstrap for pages whose durable bases can be preserved safely. The 2.59x E append amplification makes this a substantive target. This requires a bounded retained-history/base protocol and crash-safe reclamation, or a revised durable page-base protocol. It is not implemented or performance-validated by this experiment; success cannot be promised.
- **Checkpoint target:** reduce/overlap the dirty-page write and sync cost while preserving metadata publication, recovery, and invariants. The previous background implementation proved overlap but did not increase aggregate throughput, so merely moving the same work to a worker is insufficient evidence of a throughput fix. The accepted profile attributes 11.8 of 16.5 checkpoint seconds to writes/sync. Coalescing small writes could target part of the 3.9 write seconds, but cannot by itself remove the 7.9 sync seconds or repeated full-image WAL amplification; it is not the central remedy for a roughly halved E throughput.

The evidence identifies costly mechanisms in the current implementation and a measured partial recovery. It does not establish that the B-link data structure must lose to LSM, nor that fixing these mechanisms will make it universally faster than RocksDB. No invariant check, sync, condition, revision, client/admission setting, or successful-case evidence was removed.

## Historical consistency

The preserved same-source 2026-10-20 comparison independently shows the same mechanism: A falls from 5,203 to 3,968 API calls/s with WAL bytes/write transaction rising 3,347 → 5,249; E falls 2,853 → 1,362 batches/s with 8,828 → 22,836 bytes/batch. E groups stay 53.69 → 52.74. These are earlier 30-second measurements and are not pooled with the new 60-second matrix. Historical raw reports lack a compile-time git_commit; their source identity is verified through controller/build records, a narrower claim than the new raw-source-verified results. [Derived historical accounting](evidence/historical-loss.json).

## Evidence and reproduction

- [Plan](PLAN.md) and [reproduction/provenance](REPRODUCE.md).
- [Uninstrumented 24-cell summary](evidence/matrix-summary.json), [preflight summary](evidence/preflight-summary.json), and separately labeled [accepted diagnostic summary](evidence/profile-summary.json) and [verified syscall accounting](evidence/profile-analysis.json).
- [Archive verification](evidence/archive-verification.json) and [artifact SHA256 inventory](evidence/SHA256SUMS).
- [Complete raw evidence archive](evidence/blink-performance-loss-20261022-evidence.tar.gz), including the rejected attached trace and accepted launch trace.

The downloaded evidence archive contains **477 files** (59,592,686 bytes). SHA256: `3e832508223b956ccb62142ac58d00a9951b6c8936f64e9fd54ced80334803b2`. Archive/file inventories, all four matrix summaries, both diagnostic reductions, and separately published summaries passed offline byte-for-byte verification. The incomplete attached attribution remains rejected.
