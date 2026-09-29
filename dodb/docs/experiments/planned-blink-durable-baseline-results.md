# True planned-blink Durable Baseline on OCI A1 with OpenZFS

## Why this run exists

The earlier durable ZFS results labelled "Planned" did not run the planned Blink engine. `run_core_matrix.py` (real-sync matrix) and `run_crossdb_matrix.py` (sustained runs) never passed `--engine`, and the `phase0-bench` default is `main-btree`. All 84 real-sync rows and both sustained dodb rows record `"engine":"main-btree"`. The 8,380 WAL bytes per width-1 transaction, the 0.221× ratio to RocksDB and the 2,883 / 3,944 sustained tx/s all describe main-btree (the baseline `BTreeStore`) built from the experiment source.

This run measures the planned Blink engine itself, with `--engine planned-blink --sync-mode real` passed explicitly, on the same host, dataset, binary, seeds and workload. The earlier documents keep their numbers and now carry a provenance correction at the top (`zfs-real-sync-results.md`, `zfs-durable-crossdb-results.md`).

No production source was changed. No WAL format, memory-retention or engine optimization was implemented.

## Comparison targets

| Label | What it is | Source of the numbers |
|---|---|---|
| A. planned-blink | `BlinkStore` with planned execution, experiment source `0d310da` (crate tree identical to `1123b21`) | this run |
| B. experiment main-btree | `BTreeStore` from the same experiment binary; previously reported as "Planned" | reused `../oci-a1-2ocpu-12g-zfs-real-sync/raw/*-planned-*.jsonl` and `../oci-a1-2ocpu-12g-zfs-crossdb/raw/sustained-dodb-planned-*.jsonl` |
| C. ExactMain | `BTreeStore` from main `ac45cf519d6cd008dfbc5362f72561f9092a7b11` | reused `../oci-a1-2ocpu-12g-zfs-real-sync/raw/*-exactmain-*.jsonl` |
| D. RocksDB v11.8.1 | default write path, `sync = true` | reused `../oci-a1-2ocpu-12g-zfs-crossdb/raw/rocksdb-*.jsonl` |

B, C and D were not re-run. They come from the same host and dataset on the same day (B and C 11:00–12:10 UTC, D 12:55–13:51 UTC). This run was 14:30–14:49 UTC. fio sync latency before and after this run matched the earlier fio result (see Environment), so the storage path did not visibly drift, but A and B/C/D are still separate sessions and not an interleaved A/B.

## Environment and binaries

- OCI A1, 2 OCPU (AArch64 Neoverse-N1), 12 GiB configured RAM (10,898 MiB visible), Oracle Linux Server 9.8, kernel `6.12.0-206.104.4.4.el9uek.aarch64`, `rustc 1.98.1`.
- OpenZFS 2.2.11, pool `dodbbench`, dataset `/bench/zfs/db`: `sync=standard`, `recordsize=4K`, `compression=off`, `atime=off`, `ashift=12`. The runner aborts unless `findmnt` reports `zfs dodbbench/db`. `environment.txt` has the values read back at the start.
- fio 4 KiB random write, `ioengine=sync`, `fsync=1`, 15 s on the dataset: 1,452 IOPS and 669 µs mean sync before the runs, 1,489 IOPS and 651 µs after (`fio-sync-before.txt`, `fio-sync-after.txt`). The earlier real-sync run measured 1,457 IOPS and 667 µs.
- Core matrix binary: `/tmp/dodb-zfs-exp-target/release/phase0-bench`, SHA256 `79122436e330be122059f8a4bf5908406e55b74596bfab2ee3cf745dd17fb6cb`. This is the same file that produced the B rows. Only the `--engine planned-blink` argument differs.
- Sustained binary: `/tmp/dodb-crossdb-window-target/release/phase0-bench`, SHA256 `fba986d4dea7d306b4fb814e4cd82c468f427572362e95a9c1e06e5525304999`. This is the same window build that produced the B sustained rows (`dodb-sustained-window.patch` changes `phase0-bench.rs` only).
- Both binaries were built from `0d310da32af5dfaa9df6469788998dacccebb841` with `-C target-feature=+crc` (`.cargo/config.toml`). `git rev-parse 0d310da:crates` and `1123b21:crates` are both `9304c61d7fb0ae6d4854b417756051ae6a73ae80`, so the engine source equals the `1123b21` production state. `binary-proof.txt` has the hashes, source commits and worktree status.

## Method

- 14 scenarios identical to the real-sync matrix: writers 16 and 64 × width 1 and 16 × uniform, same-leaf-heavy (compact locality), different-leaf-heavy (spread locality), plus single-writer width 1 and width 16 uniform controls. Working set 100,000, key 16 B, value 64 B, 2 s warmup, 5 s measurement, 3 repetitions, fresh database per run, group cap 64 transactions / 4 MiB, queue 256, collection delay 0, unconditional transactions, 2 Tokio workers.
- Seeds equal the earlier run: `979000000 + scenario_index × 1009 + repetition_index`.
- The runner (`scripts/run_planned_blink.py`) passes `--engine planned-blink` and `--sync-mode real`, and aborts on any row whose `engine` is not `planned-blink`, whose `sync_mode` is not `real`, whose WAL sync count or time is not positive, or whose Blink superblock counters are both zero (which would mean the planner path did not run).
- All 42 rows passed: engine `planned-blink`, sync `real`, 0 errors, 0 overloads, 0 conflicts. `run-order.txt` records every command line; `process-metrics.jsonl` has sampled RSS.
- Sustained: working set 1,000,000, width 1, uniform, 16 and 64 writers, 10 s warmup, 120 s measurement, seeds `979000000 + 14 × 1009` and `+ 15 × 1009` as in the cross-DB run. The runner sampled RSS, `MemAvailable`, WAL size and data size once per second, and killed the process if `MemAvailable` fell below 256 MiB to protect the host.
- `scripts/analyze_planned_blink.py` produces `tables.md` and `analysis.json`. Applied to the reused rows, it reproduces the previously published values exactly (B / C 1.131×, B / RocksDB 0.221×, B / Turso WAL 2.733×, B / Turso MVCC 4.532×), which checks the aggregation.

## Durability evidence

- Every core row reports `sync_mode=real`, positive `wal_syncs_delta` and positive `wal_sync_nanos_total`.
- A syscall probe with the same binary (`strace -f -tt -y -e trace=fdatasync,fsync,openat`, planned-blink, 16 writers, width 1, 1 s) saw 494 successful `fdatasync` calls on the `.wal` file under `/bench/zfs/db` for 492 counted WAL syncs; the other two are the WAL initialization sync and the seeding group. No sync call failed and none targeted another file system (`sync-strace.txt`, `sync-strace.raw`).
- Source order is unchanged from the earlier proof: `WalLog::append_group_inner` writes the group and calls `sync_data()` before returning; `apply_planned_transaction_group` installs state and publishes the generation only after that call returns; the benchmark coordinator sends responses after the group call returns.
- No crash or reopen test was run in this task.

## Core 14-scenario results

### Successful logical tx/s (mean of 3 repetitions)

| # | Writers | Width | Distribution | A planned-blink | B experiment main-btree | C ExactMain | D RocksDB | A/B | A/C | A/D |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,969 | 3,534 | 3,220 | 10,034 | 1.123 | 1.233 | 0.396 |
| 1 | 16 | 1 | compact locality | 4,757 | 3,851 | 4,097 | 10,913 | 1.235 | 1.161 | 0.436 |
| 2 | 16 | 1 | spread locality | 4,592 | 4,005 | 3,664 | 10,614 | 1.147 | 1.253 | 0.433 |
| 3 | 16 | 16 | uniform | 1,012 | 499 | 469 | 5,723 | 2.027 | 2.158 | 0.177 |
| 4 | 16 | 16 | compact locality | 3,358 | 2,149 | 2,016 | 6,827 | 1.563 | 1.665 | 0.492 |
| 5 | 16 | 16 | spread locality | 3,167 | 1,871 | 1,759 | 7,126 | 1.693 | 1.801 | 0.445 |
| 6 | 64 | 1 | uniform | 11,617 | 5,289 | 4,374 | 26,254 | 2.197 | 2.656 | 0.442 |
| 7 | 64 | 1 | compact locality | 11,780 | 8,116 | 5,846 | 27,378 | 1.451 | 2.015 | 0.430 |
| 8 | 64 | 1 | spread locality | 11,914 | 6,768 | 5,059 | 26,852 | 1.760 | 2.355 | 0.444 |
| 9 | 64 | 16 | uniform | 1,132 | 612 | 533 | 9,573 | 1.849 | 2.125 | 0.118 |
| 10 | 64 | 16 | compact locality | 5,552 | 2,747 | 2,385 | 11,601 | 2.021 | 2.328 | 0.479 |
| 11 | 64 | 16 | spread locality | 4,344 | 1,980 | 1,826 | 11,351 | 2.194 | 2.379 | 0.383 |
| 12 | 1 | 1 | uniform | 1,242 | 1,083 | 939 | 1,512 | 1.148 | 1.323 | 0.822 |
| 13 | 1 | 16 | uniform | 377 | 280 | 303 | 1,353 | 1.346 | 1.245 | 0.278 |

Width-16 mutation ops/s are exactly 16 × these values (for example 88,831 ops/s for A in scenario 10). Scenarios 12 and 13 are controls and stay out of every geometric mean. `tables.md` also has mutation ops/s, per-repetition values and latency for every scenario.

### Geometric means over the 12 multiwriter scenarios

| Category | A / B experiment main-btree | A / C ExactMain | A / D RocksDB | B / D (reported earlier as "Planned / RocksDB") |
|---|---:|---:|---:|---:|
| overall (12) | **1.645** | **1.861** | **0.363** | 0.221 |
| writers 16 | 1.429 | 1.505 | 0.377 | 0.264 |
| writers 64 | 1.893 | 2.301 | 0.350 | 0.185 |
| width 1 | 1.440 | 1.682 | 0.430 | 0.298 |
| width 16 | 1.879 | 2.059 | 0.307 | 0.163 |
| uniform | 1.744 | 1.968 | 0.246 | 0.141 |
| compact locality | 1.543 | 1.736 | 0.458 | 0.297 |
| spread locality | 1.655 | 1.886 | 0.425 | 0.257 |

- planned-blink delivers 1.645× experiment main-btree and 1.861× ExactMain durable throughput. RocksDB is still 2.75× faster (A / D 0.363×), not 4.5×.
- The gap to RocksDB is smallest at width 1 (0.430×) and largest for uniform width 16 (0.177× at 16 writers, 0.118× at 64 writers), where every width-16 transaction logs about 16 leaf images.
- Against the Turso rows of the cross-DB run, planned-blink is 4.495× Turso WAL and 7.454× Turso MVCC with group commit (previously reported with main-btree as 2.733× and 4.532×). Category breakdowns are in `tables.md`.

### Repetition spread and scenario 0 control

Repetition min/max was 0.936–0.995 in 13 scenarios. Scenario 0 repetition 1 was the first run of the session and ran at 3,084 tx/s against 4,421 and 4,401; its mean WAL sync took 2.22 ms against 1.46 ms in repetition 2, so the drop came from the storage side. Three extra scenario-0 runs after the sustained runs (`control-scenario0/`) gave 4,595, 4,477 and 4,550 tx/s. The primary tables keep the original three repetitions. Replacing only scenario 0 with the control mean would change the 12-scenario GMs to 1.664× (vs B), 1.882× (vs C) and 0.367× (vs D).

### Latency p50 / p95 / p99 µs (mean of repetition percentiles)

| # | Writers | Width | Distribution | A planned-blink | B experiment main-btree | D RocksDB p99 |
|---:|---:|---:|---|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,654 / 6,993 / 12,883 | 4,314 / 6,090 / 8,065 | 2,331 |
| 1 | 16 | 1 | compact locality | 3,221 / 4,681 / 6,012 | 3,564 / 9,682 / 12,236 | 2,313 |
| 2 | 16 | 1 | spread locality | 3,397 / 4,968 / 6,234 | 3,908 / 4,664 / 6,760 | 2,245 |
| 3 | 16 | 16 | uniform | 14,149 / 26,716 / 33,625 | 30,684 / 40,533 / 63,042 | 6,714 |
| 4 | 16 | 16 | compact locality | 5,299 / 6,662 / 7,795 | 6,830 / 11,178 / 12,596 | 4,129 |
| 5 | 16 | 16 | spread locality | 4,352 / 8,355 / 10,618 | 7,990 / 11,766 / 14,604 | 3,624 |
| 6 | 64 | 1 | uniform | 4,999 / 8,390 / 10,904 | 11,377 / 17,060 / 22,323 | 4,103 |
| 7 | 64 | 1 | compact locality | 4,259 / 7,925 / 9,183 | 7,348 / 11,547 / 14,263 | 3,886 |
| 8 | 64 | 1 | spread locality | 4,845 / 8,677 / 12,806 | 8,897 / 13,078 / 15,495 | 3,837 |
| 9 | 64 | 16 | uniform | 55,651 / 68,736 / 101,395 | 103,147 / 119,823 / 216,045 | 14,653 |
| 10 | 64 | 16 | compact locality | 10,057 / 19,703 / 26,873 | 21,931 / 31,209 / 40,224 | 10,869 |
| 11 | 64 | 16 | spread locality | 13,624 / 21,256 / 30,709 | 31,262 / 40,014 / 44,266 | 12,421 |
| 12 | 1 | 1 | uniform | 749 / 1,052 / 1,652 | 867 / 1,171 / 1,740 | 968 |
| 13 | 1 | 16 | uniform | 2,533 / 2,989 / 4,163 | 3,272 / 4,483 / 5,598 | 1,135 |

The scenario 0 p99 for A is raised by the slow first repetition (22.5 ms); the other two repetitions were 6.4 ms and 9.7 ms.

## WAL, sync batching and resources

| # | Writers | Width | Distribution | A WAL B/tx | A images/tx | A tx/sync | A mean sync ms | B WAL B/tx | B images/tx | B tx/sync | B mean sync ms | A CPU % | B CPU % | A peak RSS MiB | B peak RSS MiB |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 4,224 | 1.000 | 8.31 | 1.73 | 8,380 | 2.000 | 15.65 | 2.69 | 22 | 44 | 711 | 788 |
| 1 | 16 | 1 | compact locality | 4,224 | 1.000 | 7.79 | 1.43 | 8,380 | 2.000 | 15.74 | 3.29 | 18 | 24 | 320 | 404 |
| 2 | 16 | 1 | spread locality | 4,224 | 1.000 | 7.99 | 1.45 | 8,380 | 2.000 | 15.67 | 2.73 | 22 | 35 | 320 | 384 |
| 3 | 16 | 16 | uniform | 66,506 | 15.986 | 14.32 | 7.19 | 70,665 | 16.987 | 15.93 | 8.86 | 57 | 76 | 1,073 | 864 |
| 4 | 16 | 16 | compact locality | 8,380 | 2.000 | 9.96 | 2.08 | 12,536 | 3.000 | 15.95 | 3.24 | 38 | 61 | 374 | 344 |
| 5 | 16 | 16 | spread locality | 8,380 | 2.000 | 14.05 | 2.67 | 12,536 | 3.000 | 15.97 | 3.20 | 47 | 67 | 375 | 324 |
| 6 | 64 | 1 | uniform | 4,224 | 1.000 | 59.57 | 3.17 | 8,380 | 2.000 | 63.48 | 5.06 | 46 | 63 | 930 | 889 |
| 7 | 64 | 1 | compact locality | 4,224 | 1.000 | 43.89 | 2.98 | 8,380 | 2.000 | 62.93 | 4.85 | 28 | 45 | 514 | 616 |
| 8 | 64 | 1 | spread locality | 4,224 | 1.000 | 59.66 | 3.24 | 8,380 | 2.000 | 63.34 | 4.87 | 43 | 55 | 544 | 560 |
| 9 | 64 | 16 | uniform | 66,490 | 15.982 | 59.17 | 23.75 | 70,646 | 16.982 | 63.20 | 28.84 | 63 | 77 | 1,147 | 952 |
| 10 | 64 | 16 | compact locality | 8,380 | 2.000 | 46.09 | 4.58 | 12,536 | 3.000 | 62.93 | 6.80 | 55 | 76 | 506 | 388 |
| 11 | 64 | 16 | spread locality | 8,380 | 2.000 | 60.97 | 5.08 | 12,536 | 3.000 | 63.17 | 6.98 | 72 | 83 | 471 | 359 |
| 12 | 1 | 1 | uniform | 4,224 | 1.000 | 1.00 | 0.72 | 8,380 | 2.000 | 1.00 | 0.76 | 14 | 21 | 612 | 633 |
| 13 | 1 | 16 | uniform | 66,504 | 15.986 | 1.00 | 2.10 | 70,660 | 16.985 | 1.00 | 2.15 | 25 | 43 | 764 | 718 |

- The local observation reproduces on OCI exactly: planned-blink writes 4,224 WAL bytes per width-1 transaction (one 4,156-byte leaf-image frame plus a 68-byte commit frame), main-btree writes 8,380 (leaf image plus superblock image plus commit). planned-blink elided the superblock image in 1,033,167 of 1,033,176 measured transactions; the 9 emitted images came with 9 leaf splits in uniform width-16 runs, where the benchmark's duplicate-key fallback inserts a new key.
- At width 16, planned-blink logs one image per touched leaf: about 16 in uniform (66.5 KB/tx) and 2 in the locality distributions (8,380 B/tx). main-btree logs one more image (the superblock) per transaction.
- planned-blink syncs smaller groups at 16 writers (7.8–14.3 transactions per sync vs 15.6–16.0 for main-btree) with shorter mean syncs (1.43–2.67 ms vs 2.69–3.29 ms outside uniform width 16, and 7.19 vs 8.86 ms there). At 64 writers planned-blink reaches 44–61 transactions per sync and main-btree about 63.
- A WAL MiB/s per scenario is in `tables.md`. At 64 writers width 1, planned-blink writes about 47–48 MiB/s of WAL.
- planned-blink used less CPU than main-btree in every scenario while delivering more throughput. Peak RSS includes the WAL payloads that `WalLog` keeps in memory (see below).

## 120-second sustained results

Working set 1,000,000, width 1, uniform, 10 s warmup, 120 s measurement, real sync, fresh database.

### 16 writers (completed)

4,369 tx/s over 120 s, p50 / p95 / p99 3,615 / 5,411 / 7,269 µs, 0 errors, 4,224 WAL B/tx, 1.000 images/tx, 8.71 transactions per sync, 1.52 ms mean sync, 29% of one core.

| Window | tx/s | p50 µs | p99 µs | WAL at end (MiB) | WAL growth (MiB) | RSS at end (MiB) | RSS growth (MiB) |
|---|---:|---:|---:|---:|---:|---:|---:|
| 0-10 s | 4,316 | 3,629 | 7,015 | 4,943 | 174 | 6,076 | 199 |
| 10-20 s | 4,292 | 3,640 | 7,565 | 5,116 | 173 | 6,265 | 189 |
| 20-30 s | 4,416 | 3,570 | 7,301 | 5,294 | 178 | 6,456 | 191 |
| 30-40 s | 4,291 | 3,639 | 6,702 | 5,467 | 173 | 6,638 | 182 |
| 40-50 s | 4,411 | 3,593 | 6,878 | 5,645 | 178 | 6,819 | 181 |
| 50-60 s | 4,397 | 3,608 | 7,269 | 5,822 | 177 | 7,002 | 183 |
| 60-70 s | 4,481 | 3,565 | 7,203 | 6,002 | 180 | 7,182 | 181 |
| 70-80 s | 4,435 | 3,575 | 7,263 | 6,181 | 179 | 7,372 | 190 |
| 80-90 s | 4,393 | 3,657 | 7,276 | 6,358 | 177 | 7,549 | 176 |
| 90-100 s | 4,301 | 3,665 | 6,966 | 6,530 | 173 | 7,721 | 172 |
| 100-110 s | 4,318 | 3,617 | 8,381 | 6,705 | 174 | 7,895 | 174 |
| 110-120 s | 4,380 | 3,610 | 7,372 | 6,877 | 172 | 8,068 | 173 |

Throughput and p99 stayed flat. WAL grew 2,108 MiB and RSS grew 2,192 MiB in the measured 120 s, 1.04 RSS bytes per WAL byte.

### 64 writers (killed by the memory guard)

The process was killed 194.0 s after start, about 87 s into the measurement interval, when `MemAvailable` fell below 256 MiB. RSS was 9,645 MiB and the WAL 8,316 MiB. `phase0-bench` writes its JSON row only at the end, so there is no measured throughput or latency for this run. The table below derives throughput from WAL growth: every measured transaction in this workload logs exactly 4,224 bytes, and on the 16-writer run the same derivation matched the measured value to 0.2% overall and 2.4% in the worst window. The measurement start is located from the seeding WAL size, which is identical in both runs (the seed rows do not depend on the writer count), plus the 10 s warmup.

| Window | derived tx/s | WAL at end (MiB) | WAL growth (MiB) | RSS at end (MiB) | RSS growth (MiB) | MemAvailable (MiB) |
|---|---:|---:|---:|---:|---:|---:|
| 0-10 s | 9,918 | 5,401 | 400 | 6,629 | 440 | 593 |
| 10-20 s | 10,154 | 5,810 | 409 | 7,059 | 430 | 551 |
| 20-30 s | 9,561 | 6,196 | 385 | 7,456 | 397 | 479 |
| 30-40 s | 9,665 | 6,585 | 389 | 7,849 | 392 | 471 |
| 40-50 s | 9,469 | 6,966 | 381 | 8,223 | 374 | 503 |
| 50-60 s | 9,656 | 7,355 | 389 | 8,665 | 442 | 526 |
| 60-70 s | 9,155 | 7,724 | 369 | 9,056 | 391 | 521 |
| 70-80 s | 9,209 | 8,095 | 371 | 9,427 | 371 | 437 |

About 9,600 tx/s over the 8 complete windows (80 s). WAL grew 3,093 MiB and RSS grew 3,238 MiB, 1.05 RSS bytes per WAL byte. The slight downward drift (9,918 → 9,209 tx/s) happened while free memory was already below 600 MiB. `MemAvailable` excludes the ZFS ARC, which also held memory during the run.

### Seeding

Seeding 1,000,000 rows took about 97 s in both runs and left a 4.81 GB (4,587 MiB) WAL and about 5,653 MiB RSS before warmup began. The uniform key layout sends each 25-row seed transaction to about 25 different leaves, so seeding logs full images of most leaves many times. That memory is retained for the rest of the process.

### Sustained comparison

| Writers | A planned-blink tx/s | B experiment main-btree tx/s | D RocksDB tx/s | A/B | A/D | A p99 µs | B p99 µs | D p99 µs |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 16 | 4,369 | 2,883 | 10,367 | 1.515 | 0.421 | 7,269 | 10,259 | 2,331 |
| 64 | about 9,600 (derived, 80 s before the kill) | 3,944 | 25,768 | 2.43 | 0.37 | — | 30,477 | 4,915 |

The B sustained runs were not stopped by a guard (their runner had none) and peaked at 8,626 / 9,694 MiB sampled RSS with 2.90 / 3.98 GB of measured WAL growth. planned-blink writes WAL faster in absolute terms (more transactions per second at half the bytes per transaction), so it reaches the memory limit sooner at 64 writers.

### Why RSS follows the WAL

`WalLog` keeps every committed batch in `committed: Vec<CommittedWalBatch>`, and `append_group_inner` copies each commit's page images into it after the sync. Each width-1 transaction therefore keeps at least 4,104 bytes (page ID and image) plus its batch record for the lifetime of the process, close to the 4,224 bytes it adds to the WAL file. The measured 1.04–1.05 RSS bytes per WAL byte fits that. This run only measured the slope; a per-structure breakdown (WAL payloads, `dirty_pages`, published generations) was not instrumented.

## Which earlier conclusions are no longer valid

These statements were made about "Planned" but measured main-btree:

- Planned / ExactMain durable GM 1.131× (`zfs-real-sync-results.md`). The true planned-blink value is 1.861×. The 1.131× number is a main-btree to main-btree ratio (experiment build vs main build).
- Planned / RocksDB 0.221×, "RocksDB about 4.5× faster" (`zfs-durable-crossdb-results.md`). The true planned-blink value is 0.363×; RocksDB is about 2.75× faster.
- Planned / Turso WAL 2.733× and Planned / Turso MVCC 4.532×. The true planned-blink values are 4.495× and 7.454×.
- "about 8.4 KB of WAL per transaction" for Planned. planned-blink writes 4,224 B per width-1 transaction; 8,380 B is main-btree.
- Sustained Planned 2,883 / 3,944 tx/s. planned-blink sustains 4,369 tx/s at 16 writers and about 9,600 tx/s at 64 writers until memory runs out.
- The group-size rows labelled `planned-blink` in `zfs-real-sync-results.md` (15.65 and 63.48 transactions per sync at width 1) are main-btree. planned-blink batches 7.8–8.3 and 43.9–59.7 transactions per sync at width 1.
- The explanation that the RocksDB gap comes mainly from 8.4 KB page images per transaction applies to main-btree. For planned-blink the WAL volume per width-1 transaction is half, and the gap is smaller, but WAL bytes and sync time are still the largest visible difference: 4,224 vs about 80 bytes per transaction, and 1.4–3.2 ms mean WAL sync at width 1.

## Which conclusions remain valid

- All Turso and RocksDB measurements, their syscall durability proofs, harness details and workload-equivalence checks.
- ExactMain numbers, and the experiment-main-btree vs ExactMain ratio when read as a main-btree comparison.
- The earlier sync-disabled dual rebaseline (Planned / ExactMain 3.540×): its "Planned" rows in `results/oci-a1-2ocpu-12g-200g/dual-main-vs-planned/raw/` record `"engine":"planned-blink"`. The raw rows in the other `oci-a1-2ocpu-12g-200g/` experiment directories record `planned-blink` (or both engines where an experiment compared them); their labels were not re-audited one by one here.
- The fio storage characterization of the dataset.
- The memory-retention finding: RSS grows with total WAL history and does not level off. It holds for planned-blink too (about 1.04 bytes of RSS per WAL byte), and at 64 writers it stops a 120 s run on this host.
- dodb durable throughput remains well below RocksDB on this storage path in every multiwriter scenario.

## Limitations

- A, B, C and D are separate sessions on the same host and dataset, not an interleaved run. fio before and after this run agrees with the earlier fio result within 3%.
- The 64-writer sustained throughput is derived from WAL growth; its latency is unknown. The run would need less retained memory or a larger host to complete.
- The 1.04–1.05 RSS/WAL slope is measured from process RSS; the split between WAL payload retention, `dirty_pages` and other state was not instrumented.
- Single host, one ZFS vdev on an OCI boot block volume, 2 OCPUs, memory-resident working sets.

## Artifacts

`results/oci-a1-2ocpu-12g-zfs-planned-blink/`:

- `environment.txt`, `binary-proof.txt`, `fio-sync-before.txt`, `fio-sync-after.txt`
- `sync-strace.txt`, `sync-strace.raw`, `sync-strace-smoke.jsonl`, `sync-strace-smoke.log`
- `run-order.txt`, `process-metrics.jsonl`, `core-progress.log`, `sustained-progress.log`
- `raw/`: 42 core rows and logs, 2 sustained rows (the 64-writer row is empty because the process was killed), logs and once-per-second process monitors
- `control-scenario0/`: the scenario 0 control rows, logs and script
- `scripts/run_planned_blink.py`, `scripts/analyze_planned_blink.py`, `tables.md`, `analysis.json`
- `SHA256SUMS`
