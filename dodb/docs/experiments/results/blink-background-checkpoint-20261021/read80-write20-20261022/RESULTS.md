# Read-80/write-20 results

This follow-up measures the read-heavy workload omitted from the preceding checkpoint experiment. It uses the same frozen engine executable and real synchronous-WAL durability. The only workload argument changed from the preceding mixed comparison is `--scenario rw20` to `--scenario rw80`.

Legacy synchronous B-link has +5.43% median API throughput versus the DODB-contract RocksDB reference. Segmented background 128 MiB has -8.04%. B-link reduces successful-write and overall-workflow p99, while ordinary Get p99 is worse. This is a workload- and configuration-specific trade-off, not a general B-link/LSM ranking.

All **15 measured cases** and the separate **three preflight cases** passed. No engine code, defaults, locking, admission, revision policy, durability, checkpoint threshold, or retention budget was changed. The report does not constitute a new execution of the earlier 314-test implementation gate.

## Main results

Three independent 30-second admission windows per path, each after ten-second warmup. Values are medians of per-run measurements; the range contains the three API throughput observations. API throughput includes observed final client completion and checkpoint drain. Workflow percentiles include conditional preparation where the existing workload requires it; they are not ordinary-Get latency.

| Path | API calls/s | Throughput range | Versus RocksDB | Workflow p50 ms | Workflow p95 ms | Workflow p99 ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| RocksDB minimal-native | 13,511.06 | 13,465.22–13,544.85 | +0.00% | 0.010127 | 2.974 | 29.327 |
| B-link legacy sync 128 MiB | 14,244.73 | 14,050.06–14,466.89 | +5.43% | 0.012647 | 6.459 | 9.183 |
| B-link segmented sync 128 MiB | 12,628.32 | 12,458.15–12,731.85 | -6.53% | 0.013367 | 6.951 | 9.781 |
| B-link segmented background 128 MiB | 12,424.71 | 12,190.79–12,479.36 | -8.04% | 0.013567 | 7.131 | 10.011 |
| B-link segmented background 32 MiB | 11,788.93 | 11,713.40–11,804.82 | -12.75% | 0.013487 | 7.238 | 10.625 |

## Read and write latency separately

Get means an ordinary point read, excluding the separately recorded preparatory Gets. Query16 is one range-query call returning up to 16 rows. Write p99 includes only successful write transactions. Every latency category retains all completed samples in log1024 histograms; count, sample count, and histogram sums agree.

| Path | Get p50 ms | Get p95 ms | Get p99 ms | Query16 p99 ms | Successful-write p99 ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| RocksDB minimal-native | 0.004563 | 0.020847 | 0.055391 | 0.405247 | 67.437 |
| B-link legacy sync 128 MiB | 0.006443 | 0.034623 | 0.222719 | 0.312831 | 12.042 |
| B-link segmented sync 128 MiB | 0.007243 | 0.034847 | 0.167295 | 0.289791 | 12.452 |
| B-link segmented background 128 MiB | 0.006843 | 0.036031 | 0.224511 | 0.348159 | 13.648 |
| B-link segmented background 32 MiB | 0.006803 | 0.036991 | 0.214399 | 0.303103 | 26.755 |

Background 128 MiB versus the identical segmented synchronous path changes API throughput by -1.61% and workflow p99 by +2.35%. The legacy comparison also changes physical layout, rotation, and manifest/reclamation work; it does not isolate background scheduling.

## Workload, resources, and checkpoints

All paths use 16 dedicated clients, 512-byte values, 10,000 starting rows, seed 20261013, 1,024 overwrite slots/client, reserved-range revisions, the existing Get/Query16 mixture, and width-1/4/8/16 writes. The target 80:20 ratio counts API calls, including preparatory Gets; it is not a ratio of records read and mutated. Conditions, intentional rejections, logical revisions, tombstones/ABA, final-state checks, and reopen validation remain enabled.

| Path | Read fraction | Reads/s | Successful write tx/s | Median peak process-group RSS MiB | Median maximum WAL MiB | Measured checkpoints r1/r2/r3 | Overlap writes r1/r2/r3 |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| RocksDB minimal-native | 80.021% | 10,811.69 | 2,510.48 | 277.90 | 61.22 | 0/0/0 | 0/0/0 |
| B-link legacy sync 128 MiB | 80.024% | 11,398.93 | 2,646.89 | 415.38 | 128.02 | 3/3/3 | 0/0/0 |
| B-link segmented sync 128 MiB | 80.015% | 10,104.57 | 2,347.38 | 396.89 | 128.09 | 4/4/4 | 0/0/0 |
| B-link segmented background 128 MiB | 80.031% | 9,943.58 | 2,307.48 | 416.43 | 130.80 | 4/4/3 | 1348/1319/865 |
| B-link segmented background 32 MiB | 80.025% | 9,434.05 | 2,190.11 | 281.30 | 32.75 | 13/12/13 | 2012/1564/1722 |

The measured read fraction was 80.013%–80.035%. All measured cases had zero request errors and overloads. All final-state/reopen checks and fixed 16-client input-prefix comparisons passed. Capacity waits were 0. Every B-link measured case observed at least 3 completed checkpoints without forcing a lower threshold. Checkpoint counts/overlap exclude explicit final drain; throughput includes that drain.

The resource columns describe the benchmark process group, including driver, runtime, collectors, and validation; they are not isolated engine memory. All cases stayed within the preserved 3 GiB RSS, 4 GiB database, 1500 MiB available-memory, 10 GiB free-space, and 150-second timeout guards. No request/checkpoint failures, omitted events, or retained tasks/images remained after drain.

## Provenance and limits

- Engine source: `d240416f0e0efafa7c08bb1e868387ebd0d894bc`; clean before and after measurement.
- Frozen service executable SHA256: `09bd505d77ce0bcbc61ca46f7389373e60bc1ab1bf6b80a90077bed328418e40`.
- Initial plan/controller commit: `061ccc9c9cc2bdbb4e5e7b7bfa2d29c639ecf240`; controller script hashes are recorded separately from engine provenance.
- Authorized OCI A1 host: `opc@217.142.246.204`, two Neoverse-N1 cores; ZFS `/bench/zfs/db`, `sync=standard`, compression off, 4 KiB records, latency log bias.
- Rust 1.97.1, AArch64 `+crc`, pinned RocksDB 11.8.1, unchanged from the preceding frozen build.
- All 15 argv comparisons against their preceding mixed-workload counterparts passed after normalizing the scenario and fresh database/report/binary aliases. Source and binary digests are identical.

These are three short synthetic repetitions with a memory-resident working set and real durable writes. They do not establish steady-state, larger-than-memory, production throughput, or a common complete-engine memory budget. The reference directly executes RocksDB TransactionDB operations with the DODB contract; it is not a raw WriteBatch or maximum-RocksDB reference.

## Evidence

- [Plan and acceptance criteria](PLAN.md)
- [Reproduction and verification](REPRODUCE.md)
- [Complete per-run metrics and medians](evidence/matrix-summary.json)
- [Separate preflight summary](evidence/preflight-summary.json)
- [Original reports, argv, resource samples, environment, database hashes, logs, and controller snapshots](evidence/blink-read80-write20-20261022-evidence.tar.gz)
- [Archive size, file count, and SHA256](evidence/blink-read80-write20-20261022-evidence.tar.gz.json)
- [Offline verification](evidence/archive-verification.json)
