# Durable dodb Benchmark on OCI A1 with OpenZFS

## Provenance correction (added 2026-09-25)

Every column labelled "Planned" in this document is **actually experiment main-btree due missing --engine argument**. It is not the planned Blink engine.

- `scripts/run_core_matrix.py` never passed `--engine`. The `phase0-bench` default is `EngineKind::MainBtree`, so both binaries ran the baseline `BTreeStore`.
- All 84 raw rows in `raw/*.jsonl` record `"engine":"main-btree"`, including the 42 rows named `*-planned-*`. Every Blink counter in those rows is zero (`superblock_images_emitted`, `superblock_images_elided`, `leaf_splits_total`, `logical_groups`).
- The measured comparison is therefore main-btree built from the experiment source `0d310da` against main-btree built from main `ac45cf5`. The 1.131× geometric mean is a main-btree to main-btree ratio. The ExactMain column is correct as labelled.
- The group-size table below prints `planned-blink` as a row label. Those rows are also main-btree.
- The 8,380 WAL bytes per width-1 transaction come from main-btree: one leaf image and one superblock image per commit (2 × 4,156-byte frames + a 68-byte commit frame).
- The numbers below are kept unchanged. The true planned-blink durable baseline, re-measured with `--engine planned-blink` on the same host, dataset and binary, is in `planned-blink-durable-baseline-results.md`.

## Environment

- OCI A1, 2 OCPU, AArch64 Neoverse-N1, Oracle Linux Server 9.8, 12 GiB configured RAM.
- Running kernel: `6.12.0-206.104.4.4.el9uek.aarch64`; no reboot was performed.
- Device `/dev/sda` is the 200 GiB OCI boot BlockVolume. The ZFS pool uses only the existing `/dev/sda4` partition, range 97,726,464–419,430,366, 153.40 GiB. `/dev/sda1` EFI, `/dev/sda2` boot, and `/dev/sda3` LVM boundaries were unchanged.
- The OCI block volume is the underlying storage and shares the boot volume I/O path. This is one ZFS vdev, not a separate attached volume.
- `environment-final.txt`, `final-storage-invariants.txt`, `mounts-lvm-final.txt`, and `partition-final.txt` retain system and device evidence.

## ZFS configuration

- OpenZFS tag `zfs-2.2.11`, resolved source commit `f2d87f5c724ccdbb27a11fb84c4a185e0a917598`.
- The module loaded successfully for the running UEK kernel. `modinfo` vermagic matches the running kernel and `lsmod` showed `zfs` and `spl`.
- Pool `dodbbench` is ONLINE on the single vdev `/dev/sda4`, with `ashift=12`.
- Pool and dataset properties: `sync=standard`, `compression=off`, `atime=off`, `recordsize=4K`; benchmark dataset is `/bench/zfs/db`.
- Pool status was ONLINE with no known data errors after the run. Final dataset use was about 1.91 MiB; the temporary benchmark files were removed between runs.
- Detailed build, module, pool and dataset state is preserved in `zfs-build.log`, `zfs-provenance-final.txt`, `zpool-status.txt`, `zpool-properties-final.txt`, `zfs-properties-root.txt`, and `zfs-properties-db.txt`.

## fio sync context

- On the same dataset, fio 3.35 ran 4 KiB random writes with `ioengine=sync`, `iodepth=1`, `fsync=1`, and a 15 s time-based interval.
- It measured 1,457 IOPS and 5,831 KiB/s. Fsync/fdatasync latency: mean 667 µs, p50 635 µs, p95 857 µs, p99 1,352 µs.
- These values characterize the storage path; they do not rank the database engines. Full output is in `fio-sync.txt`.

## Build provenance

- Planned source: `0d310da32af5dfaa9df6469788998dacccebb841`; binary SHA256 `79122436e330be122059f8a4bf5908406e55b74596bfab2ee3cf745dd17fb6cb`.
- Exact-main source: `ac45cf519d6cd008dfbc5362f72561f9092a7b11`; benchmark-only harness port; binary SHA256 `d32ced9744fd16f7949bca0d263ce8f6de3d0d9b38ac3ed655da81bd63e77417`.
- Both are fresh AArch64 release builds. The verbose logs show `-C target-feature=+crc` on `crc32c` and `dodb_storage` builds. `main-harness.patch` and `main-harness-equivalence.txt` record the temporary comparator adapter and equivalence of shared workload generation, seed generation, writer loop, sync behavior, scenario generation, and measured loop.
- The exact-main source worktree had only temporary, untracked benchmark files; the experiment production source remained unchanged. No production optimization was made.
- Build and binary records: `experiment-build.log`, `main-build.log`, `experiment-binary-proof.txt`, `main-binary-proof.txt`, `experiment-crc-build-proof.txt`, and `main-crc-build-proof.txt`.

## Durability proof

- Every core run records `sync_mode=real`, positive `wal_syncs_delta`, and positive `wal_sync_nanos_total`. Across 84 runs, successful transactions totaled 548,247 for ExactMain and 642,667 for Planned; all runs reported zero errors, conflicts, and overloads.
- A dedicated Linux syscall trace observed successful `fdatasync` on the WAL and `fsync` on the database file under `/bench/zfs/db`. See `sync-strace.txt` and its smoke JSON row. Across the 42 ExactMain invocations there were 38,120 WAL syncs and 81.304 s cumulative WAL sync time; across 42 Planned invocations there were 41,676 syncs and 104.999 s cumulative sync time.
- Source order proof: the benchmark real-mode `BenchFile::sync_data()` delegates to `ProductionFile::sync_data()` / `File::sync_data()`. WAL group append writes records and then calls `sync_data()` before returning success. Planned Blink installs state and publishes the generation after the WAL call; the coordinator sends transaction responses after segment processing returns. Line references are recorded in `durability-source-proof.txt`.
- Clean close/reopen checks each committed one width-16 transaction containing 16 mutations, then reopened the database and verified all 16 values for ExactMain and Planned. Both passed; helper sources and output are retained as `scripts/reopen_*.rs` and `*-reopen-check.txt`.
- The syscall trace is a dedicated real-sync smoke, while the core runs validate real-mode counters for every measured invocation. No crash/fault-injection suite was run in this task.

## Benchmark matrix and workload equivalence

- Two engines: ExactMain and Planned. Each had 14 scenarios × 3 repetitions, 84 invocations overall.
- Core matrix: working set 100,000; key 16 B; value 64 B; writers 16/64; width 1/16; uniform, same-leaf-heavy, and different-leaf-heavy. Single-writer controls were width 1 and width 16 uniform.
- Each invocation used fresh database files under `/bench/zfs/db`, 2 s warmup, 5 s measurement, 3 repetitions per scenario, group cap 64 transactions / 4 MiB, queue 256, zero collection delay, unconditional transactions, and 2 Tokio workers. Run order alternated engine first/second across repetitions. Paired engines used the same scenario seed.
- The locality names describe dodb’s compact and spread key-range distributions. For external engines these semantics would be named compact-locality and spread-locality; this task did not benchmark external engines.
- `run-order.txt` records every invocation and `raw/*.jsonl` retains all engine outputs. `raw/process-metrics.jsonl` retains sampled process RSS. `scripts/run_core_matrix.py` and `scripts/analyze_results.py` define execution and aggregation.
- Table throughput is the arithmetic mean of the three run-level throughputs. Latencies are the arithmetic mean of each repetition’s reported end-to-end p50/p95/p99. Width-16 logical tx/s and mutation ops/s differ by exactly 16. Resource and durability detail is in `resource-metrics.tsv`.

### Full scenario throughput

| Writers | Width | Distribution | ExactMain tx/s | Planned tx/s | ExactMain ops/s | Planned ops/s | Planned / ExactMain |
|---:|---:|---|---:|---:|---:|---:|---:|
| 16 | 1 | uniform | 3220 | 3534 | 3220 | 3534 | 1.0977 |
| 16 | 1 | same-leaf-heavy | 4097 | 3851 | 4097 | 3851 | 0.9401 |
| 16 | 1 | different-leaf-heavy | 3664 | 4005 | 3664 | 4005 | 1.093 |
| 16 | 16 | uniform | 469 | 499 | 7499 | 7984 | 1.0647 |
| 16 | 16 | same-leaf-heavy | 2016 | 2149 | 32263 | 34377 | 1.0655 |
| 16 | 16 | different-leaf-heavy | 1759 | 1871 | 28141 | 29930 | 1.0636 |
| 64 | 1 | uniform | 4374 | 5289 | 4374 | 5289 | 1.2091 |
| 64 | 1 | same-leaf-heavy | 5846 | 8116 | 5846 | 8116 | 1.3884 |
| 64 | 1 | different-leaf-heavy | 5059 | 6768 | 5059 | 6768 | 1.3378 |
| 64 | 16 | uniform | 533 | 612 | 8520 | 9795 | 1.1496 |
| 64 | 16 | same-leaf-heavy | 2385 | 2747 | 38154 | 43955 | 1.1521 |
| 64 | 16 | different-leaf-heavy | 1826 | 1980 | 29224 | 31678 | 1.084 |
| 1 | 1 | uniform | 939 | 1083 | 939 | 1083 | control |
| 1 | 16 | uniform | 303 | 280 | 4842 | 4477 | control |

### Full scenario latency

| Writers | Width | Distribution | ExactMain p50 / p95 / p99 (µs) | Planned p50 / p95 / p99 (µs) |
|---:|---:|---|---:|---:|
| 16 | 1 | uniform | 4632 / 6685 / 9164 | 4314 / 6090 / 8065 |
| 16 | 1 | same-leaf-heavy | 3540 / 5584 / 7575 | 3564 / 9682 / 12236 |
| 16 | 1 | different-leaf-heavy | 4028 / 6005 / 8258 | 3908 / 4664 / 6760 |
| 16 | 16 | uniform | 31706 / 47646 / 69083 | 30684 / 40533 / 63042 |
| 16 | 16 | same-leaf-heavy | 7312 / 10220 / 13562 | 6830 / 11178 / 12596 |
| 16 | 16 | different-leaf-heavy | 8509 / 11139 / 14135 | 7990 / 11766 / 14604 |
| 64 | 1 | uniform | 14103 / 18357 / 26979 | 11377 / 17060 / 22323 |
| 64 | 1 | same-leaf-heavy | 10278 / 16226 / 23828 | 7348 / 11547 / 14263 |
| 64 | 1 | different-leaf-heavy | 11509 / 17693 / 27442 | 8897 / 13078 / 15495 |
| 64 | 16 | uniform | 116599 / 159714 / 205993 | 103147 / 119823 / 216045 |
| 64 | 16 | same-leaf-heavy | 25112 / 34400 / 52802 | 21931 / 31209 / 40224 |
| 64 | 16 | different-leaf-heavy | 33390 / 46360 / 70939 | 31262 / 40014 / 44266 |
| 1 | 1 | uniform | 982 / 1608 / 2135 | 867 / 1171 / 1740 |
| 1 | 16 | uniform | 2928 / 4366 / 5570 | 3272 / 4483 / 5598 |

### Single-writer controls

| Width | ExactMain tx/s | Planned tx/s | ExactMain ops/s | Planned ops/s | ExactMain p50/p95/p99 µs | Planned p50/p95/p99 µs |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 939 | 1083 | 939 | 1083 | 982/1608/2135 | 867/1171/1740 |
| 16 | 303 | 280 | 4842 | 4477 | 2928/4366/5570 | 3272/4483/5598 |

## Group commit and actual sync batching

| Writers | Width | Engine | Mean tx/sync | Mean realized group | Maximum realized group | Mean WAL sync latency | Mean syncs/s |
|---:|---:|---|---:|---:|---:|---:|---:|
| 16 | 1 | exact-main | 15.63 | 15.63 | 16 | 2366.1 µs | 206.09 |
| 16 | 1 | planned-blink | 15.65 | 15.65 | 16 | 2688.3 µs | 225.89 |
| 16 | 16 | exact-main | 15.6 | 15.6 | 16 | 6458.0 µs | 30.05 |
| 16 | 16 | planned-blink | 15.93 | 15.93 | 16 | 8867.1 µs | 31.34 |
| 64 | 1 | exact-main | 62.36 | 62.36 | 64 | 4424.5 µs | 70.15 |
| 64 | 1 | planned-blink | 63.48 | 63.48 | 64 | 5061.2 µs | 83.33 |
| 64 | 16 | exact-main | 61.59 | 61.59 | 64 | 24090.5 µs | 8.65 |
| 64 | 16 | planned-blink | 63.21 | 63.21 | 64 | 28863.5 µs | 9.69 |

- With width 1, the mean group rose from 15.62 to 61.47 transactions for ExactMain and 15.69 to 63.25 for Planned as writers increased 16→64. The observed W64 groups reached the 64-transaction cap in some intervals; across all six multiwriter scenarios the mean group was 61.88 (ExactMain) and 63.17 (Planned), and the maximum was 64 for both.
- This is consistent with requests accumulating while prior groups wait on WAL sync: aggregate group size approaches the cap at 64 writers. These counters do not expose the arrival time of each request during an individual sync, so the precise fraction queued during fsync cannot be isolated.
- Mean queue wait per queued request across the six multiwriter scenarios was about 434 µs at 16 writers and 1,866 µs at 64 for ExactMain; Planned was about 424 µs and 1,722 µs. Mean collection time per group was about 336/1,338 µs (ExactMain) and 396/1,472 µs (Planned). Definitions and per-scenario values are in `resource-metrics.tsv`.
- Group p50/p95 cannot be derived from the current exported data: it reports mean and maximum realized group size, not a per-group histogram or samples. WAL sync p50/p95/p99 are likewise unavailable; `sync_nanos / wal_syncs` yields the mean only. No production hot-path instrumentation was added to obtain those percentiles.

## Planned vs ExactMain ratios

| Scenario group | Geometric mean of Planned / ExactMain mutation throughput |
|---|---:|
| all | 1.131× |
| different-leaf-heavy | 1.139× |
| same-leaf-heavy | 1.125× |
| uniform | 1.129× |
| width-1 | 1.168× |
| width-16 | 1.096× |
| writers-16 | 1.053× |
| writers-64 | 1.215× |

The overall geometric mean covers only the 12 multiwriter scenarios. Single-writer controls are excluded. A ratio above 1 means Planned was faster on the measured mutation throughput.

## Comparison with the earlier sync-disabled result

- The earlier 12-scenario sync-disabled dual rebaseline had a Planned / ExactMain geometric mean of 3.540×. The present real-sync matrix is 1.131×.
- This is a diagnostic comparison, not a controlled A/B ratio: the previous run used sync disabled, 2 s measurements, and different cache settings. It is not a durable-throughput result and is not combined with the present headline.
- For uniform width-1 groups, ExactMain mean group size changed 15.92→15.63 at 16 writers and 63.42→62.36 at 64 writers. Planned changed 13.97→15.65 at 16 and 58.19→63.48 at 64. The prior values are one short sync-disabled baseline; see `analysis.json` for all matching workload rows.

## Correctness and limitations

- All 84 measured runs completed with zero errors, conflicts, or overloads, and every row reported positive WAL sync count/time in real mode. Attempted and successful logical transaction counts matched in every run; mutation counts matched `successful_transactions × transaction_width` for all runs.
- The clean close/reopen test passed for both engines, verifying the 16 values in one width-16 logical transaction after reopening.
- Each engine was built from its pinned source, but exact-main used a benchmark-only harness port because the old main commit does not contain the experiment’s Blink engine. Shared workload generation and measured loop sections were source-hash compared; adapter changes are documented.
- Results represent this single OCI boot-volume-backed block device, one ZFS vdev, and a 2 OCPU host. fio’s small synchronous random-write latency and database WAL sync means are different workloads and need not match.
- Group-size and WAL sync latency percentiles are unavailable from the production metrics exported by the harness. RSS is a sampled process peak; CPU is the run-level utilization metric.
- No Turso or RocksDB benchmark was run. No tuning or production-source optimization was performed.

## Conclusion

On this real-sync ZFS setup, Planned delivered a 1.131× 12-scenario multiwriter geometric-mean mutation throughput relative to ExactMain. The advantage was modest at 16 writers (1.053× GM) and larger at 64 writers (1.215× GM), while all observed successful writes used the real durable WAL sync path. At 64 writers, the group size approached the configured 64-transaction cap. These findings establish a durable dodb baseline for this OCI storage path; they do not generalize beyond this single-vdev boot-volume environment.
