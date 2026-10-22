# Concurrent physical checkpoint results

## Conclusion

Concurrent segmented checkpoints are implemented and verified: acknowledged same-shard writes complete while the fixed checkpoint worker is active. They did **not improve aggregate throughput** in this short comparison. At 128 MiB, background versus the identical segmented synchronous path changes median throughput by -0.94% for A and -1.56% for E. E p99 falls from 1,210.06 ms to 559.42 ms (-53.77%); A p99 rises from 11.89 ms to 12.51 ms (+5.17%).

Implementation source: `d240416f0e0efafa7c08bb1e868387ebd0d894bc`. OCI functional gate: **314 passed, zero failed, four existing ignored probes**. All 30 corrected-source cells passed outcome, latency-count, fixed-input-prefix, source/binary, final/reopen, checkpoint/overlap, and sampled resource gates. Defaults remain legacy layout, synchronous execution, and automatic checkpointing disabled.

The prior assumption that checkpoint blocking explains a general RocksDB throughput disadvantage is not established by these measurements. Relative to the DODB-contract native RocksDB reference, background 128 MiB is +4.41% in A and -19.19% in E. The A reference comparison includes all engine/layout/service-path differences; it does not isolate checkpoint scheduling.

## Median results

Three 30-second admission windows per cell, after ten-second warmup. Throughput includes observed final client completion and checkpoint drain. A reports API calls/s; E reports width-16 write batches/s. Each percentile includes every completed latency sample in log1024 histograms. Ranges show the three throughput repetitions.

### Workload A

| Path | Throughput/s | Throughput range | p50 ms | p95 ms | p99 ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| RocksDB minimal-native | 3453.56 | 3452.69-3465.65 | 2.12 | 27.05 | 63.73 |
| Legacy sync 128 MiB | 3831.15 | 3788.46-3906.58 | 4.31 | 8.24 | 11.21 |
| Segmented sync 128 MiB | 3640.08 | 3572.13-3657.13 | 4.61 | 8.50 | 11.89 |
| Segmented background 128 MiB | 3605.85 | 3578.64-3643.10 | 4.66 | 8.66 | 12.51 |
| Segmented background 32 MiB | 3418.31 | 3378.29-3426.35 | 4.72 | 9.03 | 17.45 |

### Workload E

| Path | Throughput/s | Throughput range | p50 ms | p95 ms | p99 ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| RocksDB minimal-native | 1698.39 | 1679.16-1715.57 | 30.72 | 56.92 | 331.35 |
| Legacy sync 128 MiB | 1379.48 | 1376.81-1385.91 | 28.95 | 57.15 | 1247.81 |
| Segmented sync 128 MiB | 1394.12 | 1353.77-1402.63 | 29.51 | 57.25 | 1210.06 |
| Segmented background 128 MiB | 1372.39 | 1370.47-1373.29 | 31.11 | 83.43 | 559.42 |
| Segmented background 32 MiB | 1230.90 | 1226.49-1233.89 | 30.74 | 207.49 | 292.81 |

## Background 128 MiB comparisons

| Workload | Reference | Throughput change | p99 change |
| --- | --- | ---: | ---: |
| A | Segmented sync 128 MiB | -0.94% | +5.17% |
| A | Legacy sync 128 MiB | -5.88% | +11.54% |
| A | RocksDB minimal-native | +4.41% | -80.37% |
| E | Segmented sync 128 MiB | -1.56% | -53.77% |
| E | Legacy sync 128 MiB | -0.51% | -55.17% |
| E | RocksDB minimal-native | -19.19% | +68.83% |

The same-layout sync/background comparison holds snapshot preparation, invariants, segmented format, thresholds, and reclamation protocol constant. The legacy comparison also changes the layout, rotation, and manifest path. The 32 MiB background option increases checkpoint frequency and lowers throughput further; its E p99 is 292.81 ms. This is a measured throughput/latency trade-off, not a reason to change defaults.

## Actual overlap and remaining serialized work

| Workload/path | Measured checkpoints r1/r2/r3 | Acknowledged overlap writes r1/r2/r3 | Median checkpoint wall time, s | Median preparation/completion pause, s |
| --- | --- | --- | ---: | ---: |
| A Segmented sync 128 MiB | 4/4/4 | 0/0/0 | 2.429 | 0.542 |
| A Segmented background 128 MiB | 4/4/4 | 1784/1841/1665 | 2.819 | 0.712 |
| A Segmented background 32 MiB | 14/14/14 | 2333/2462/2172 | 4.727 | 1.710 |
| E Segmented sync 128 MiB | 7/7/7 | 0/0/0 | 8.503 | 2.273 |
| E Segmented background 128 MiB | 7/7/7 | 3762/3857/3536 | 10.480 | 2.784 |
| E Segmented background 32 MiB | 24/24/24 | 5000/4705/5088 | 14.938 | 7.049 |

Measured checkpoint counts and overlap use only fully completed events inside dispatch through observed last-client completion, excluding explicit final drain. The two timing columns are cumulative report metrics, including all reported checkpoint activity; they are not percentile sums or a claim of isolated engine CPU time. A write counts as overlap only while the worker is actually active, not merely queued in the blocking pool.

Page writes/data sync/metadata sync overlap later writes. Invariant checking and durable rotation during preparation, and latest-manifest publication/reclamation during completion, remain serialized. Background execution therefore does not remove all checkpoint pauses or storage contention. The evidence establishes actual overlap and the measured trade-off; it does not establish a single cause for the remaining throughput cost.

## Preserved initial matrix and scheduling correction

The initial source `58a197a1711a51f76188db4e9bc314fc89bbf5c1` completed its entire 30-cell matrix. It is retained in full, including its negative throughput result. Initial background 128 MiB medians were A 3,511.14 API calls/s and E 1,301.63 batches/s. Its background checkpoint counts were A 8/8/8 and E 14/14/14, twice the same-layout synchronous counts.

A deterministic regression then failed because a coalesced automatic request launched another checkpoint at 4,328 retained WAL bytes despite a 65,536-byte threshold. The corrected source rechecks remaining WAL after reclamation. The new regression passes, and corrected counts match the synchronous path: A 4/4/4 and E 7/7/7. Capacity-forced checkpoints remain independent; the acknowledged below-threshold post-target write survives shutdown/reopen.

Corrected background 128 MiB throughput is 2.70% above the initial background result in A and 5.44% above it in E. These are observations across two sequential matrices, not a paired estimate that isolates every source of run-to-run change. No threshold, retention budget, invariant check, durability policy, admission/lock setting, or workload was relaxed. A reporting correction never reruns accepted cells.

## Resources and scope

Across corrected cells, sampled peak process-group RSS was 1,438,998,528 bytes; maximum database directory size was 559,081,658 bytes; maximum sampled WAL bytes were 154,500,350; maximum reported retained encoded snapshot bytes were 129,544,192. All cells stayed within the predeclared RSS/database/available-memory/free-space guards, with no capacity waits, overloads, request errors, failed checkpoints, omitted events, or remaining tasks/images after drain.

The host has two Neoverse-N1 cores and ZFS with `sync=standard`, compression disabled, 4 KiB records, and latency log bias. Rust 1.97.1, AArch64 `+crc`, and RocksDB 11.8.1 are pinned. The reference uses TransactionDB and the DODB conditional/revision contract; it is not raw WriteBatch or a maximum-RocksDB baseline. A and E retain their distinct caller counts and workloads. This is three short synthetic repetitions, with no common complete-engine memory budget and no steady-state or production-throughput claim. SIGKILL evidence covers process termination, not physical power loss.

The predeclared 300-second gate failed: neither workload improves throughput by at least 10% versus segmented sync 128 MiB, and A p99 is worse. The long follow-up was not run. Background mode remains opt-in; these results do not support promoting it as a throughput optimization.

## Reproduction and evidence

- [Read-80/write-20 follow-up with the same frozen engine executable](read80-write20-20261022/RESULTS.md)
- [Implementation and protocol](CONCURRENT-CHECKPOINT.md)
- [Execution plan and acceptance criteria](EXECUTION-PLAN.md)
- [Final functional/recovery validation](VALIDATION-CONCURRENT.md) and [initial validation](VALIDATION-INITIAL.md)
- [Reproduction commands and provenance](REPRODUCE.md)
- [Final matrix summary](evidence/oci-concurrent/final-matrix-summary.json) and [preserved initial summary](evidence/oci-concurrent/initial-matrix-summary.json)
- [Artifact inventory](EVIDENCE.md)
- [Earlier synchronous/codec milestone reports](RESULTS-HISTORICAL.md)
