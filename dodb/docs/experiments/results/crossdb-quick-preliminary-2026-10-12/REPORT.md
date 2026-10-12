# Preliminary cross-engine comparison — 2026-10-12

This is a four-workload, one-repetition preliminary comparison. It does not establish long-term stability or general engine superiority. No engine optimization was implemented.

## Execution accounting

- OCI: 12 completed results reused from the clean `d3f4958cdcbba3508f79e6d5680b8d1dacbd3345` checkout; 0 new benchmark runs. Reused Point Get, Query16, and pure-write records were 2 s warmup / 10 s measurement; reused mixed records were 0.5 s / 1.501–1.502 s. They retain their actual durations and are not relabeled as 1 s / 5 s runs.
- Debian: 12 new results completed and validated (3 engines × 4 workloads), with one failed benchmark invocation (main-btree Get exited 101 at the data-root guard before creating DB files) preserved and retried successfully. One preflight stopped before any benchmark, and the first native build stopped because Cargo was absent from the noninteractive PATH; all records are preserved. The successful build used Rust 1.97.1.
- The superseded wider experiment's 40 completed records and one interrupted attempt remain preserved and are excluded from this comparison.
- Revised-scope totals: 12 reused, 12 successful new benchmark runs, 1 failed benchmark invocation, 0 interrupted quick-comparison runs. The failed invocation is an additional attempt, not one of the 12 valid results.

## Environment and configuration

| Environment | Host and storage | Clients and dodb workers | Actual timing |
|---|---|---|---|
| OCI | Oracle Linux / UEK ARM64, 2 logical CPUs, 10 GiB RAM, 4 GiB swap; `/bench/zfs/db` is `dodbbench/db` ZFS, `sync=standard`, 4 KiB record size, compression off, atime off | 16 external OS-thread clients per engine; dodb Tokio runtime 2 workers | Get, Query16, pure write: 2 s / 10 s; mixed: 0.5 s / 1.501–1.502 s |
| Debian 13 | AMD Ryzen 5 5600, 6 physical / 12 logical CPUs, 1 NUMA node; 31 GiB RAM, 12 GiB swap; `/home/namse/crossdb-quick-preliminary-2026-10-12` on `/dev/sda2` ext4, 32 GiB free before runs; affinity 0–11 and cgroup CPU/memory limits unlimited | 12 external OS-thread clients per engine; dodb Tokio runtime 12 workers | 1 s warmup / 5 s requested measurement; recorded measurement duration 5.000–5.011 s |

Both environments used 10,000 uniform rows, 16-byte keys, 512-byte values, 64 MiB dodb page cache and RocksDB block cache, width 1, unique per-run DB paths, and the same per-condition seed across engines. Write values changed with the logical request identity. Query16 used distributed partitions and cursor positions. Each client was supplied by one OS thread; there was no harness-wide DB mutex. Input hashes matched at 31,020 Get, 3,340 Query16, 159 pure-write, and 42 mixed OCI common checkpoints; Debian shared 97,081, 11,945, 12, and 29 checkpoints respectively. Different final trace lengths are expected.

On both hosts, dodb used real `ProductionFile` synchronization. Planned-blink parallel execution and borrowed-page views were off; one coordinator worker and read-observation metrics were enabled. Collection limit was 64 and queue capacity 256; main-btree had a 4 MiB group-byte limit. OCI used two Tokio workers; Debian used 12. Process thread samples showed 20 threads for OCI dodb, 24–26 for Debian dodb, and engine-default RocksDB threads (15–21 OCI; 6–17 Debian).

RocksDB was v11.8.1, commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654`, built natively on each host. WAL was enabled, `WriteOptions.sync=true`, each logical transaction used one atomic `WriteBatch`, and pipelined writes were off. RocksDB and dodb write operations have different transaction semantics; their rates do not imply equivalent transaction behavior.

## OCI results

| Condition | Engine | Read/s | Write tx/s | Aggregate ops/s | Read p95/p99 µs | Write p95/p99 µs | CPU % machine | RSS p50/p95/max MiB (samples) | Verified |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| get-c16 | main-btree | 702,717 | 0 | 702,717 | 2.7/3.1 | — | 98.7 | 82.6/82.6/82.6 (10/10) | pass |
| query-c16 | main-btree | 79,505 | 0 | 79,505 | 29.0/62.4 | — | 96.7 | 81.2/81.2/81.2 (10/10) | pass |
| write-c16-w1 | main-btree | 0 | 3,728 | 3,728 | — | 6,539.3/7,738.8 | 17.2 | 73.8/83.7/83.7 (10/10) | pass |
| mixed-50-c16-w1 | main-btree | 3,900 | 3,888 | 7,788 | 20.9/68.7 | 6,339.0/7,094.0 | 18.6 | 61.0/61.0/61.0 (1/1) | pass |
| get-c16 | planned-blink | 667,483 | 0 | 667,483 | 2.9/3.2 | — | 98.2 | 63.3/63.3/63.3 (10/10) | pass |
| query-c16 | planned-blink | 71,735 | 0 | 71,735 | 33.2/67.1 | — | 99.0 | 60.0/60.0/60.0 (10/10) | pass |
| write-c16-w1 | planned-blink | 0 | 8,802 | 8,802 | — | 2,779.7/3,512.6 | 17.9 | 82.7/96.6/96.6 (10/10) | pass |
| mixed-50-c16-w1 | planned-blink | 8,255 | 8,325 | 16,580 | 19.9/92.4 | 2,685.0/2,923.7 | 20.3 | 49.3/49.3/49.3 (1/1) | pass |
| get-c16 | rocksdb | 863,170 | 0 | 863,170 | 2.2/2.6 | — | 98.2 | 49.6/49.6/49.6 (10/10) | pass |
| query-c16 | rocksdb | 111,204 | 0 | 111,204 | 17.2/23.1 | — | 99.0 | 49.8/49.8/49.8 (10/10) | pass |
| write-c16-w1 | rocksdb | 0 | 7,804 | 7,804 | — | 2,598.1/3,760.4 | 14.8 | 93.6/128.2/128.2 (10/10) | pass |
| mixed-50-c16-w1 | rocksdb | 7,555 | 7,592 | 15,147 | 16.1/34.6 | 2,614.5/3,251.9 | 17.0 | 41.2/41.2/41.2 (1/1) | pass |

## Debian results

| Condition | Engine | Read/s | Write tx/s | Aggregate ops/s | Read p95/p99 µs | Write p95/p99 µs | CPU % machine | RSS p50/p95/max MiB (samples) | Verified |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| get-c12 | main-btree | 5,420,743 | 0 | 5,420,743 | 1.9/2.2 | — | 99.6 | 78.6/78.6/78.6 (4/5 incomplete) | pass |
| get-c12 | planned-blink | 4,970,793 | 0 | 4,970,793 | 2.5/2.7 | — | 100.0 | 58.1/58.1/58.1 (4/5 incomplete) | pass |
| get-c12 | rocksdb | 6,078,412 | 0 | 6,078,412 | 1.7/1.8 | — | 100.0 | 48.8/49.7/49.7 (5/5) | pass |
| mixed-50-c12-w1 | main-btree | 908 | 899 | 1,808 | 12.9/16.0 | 15,539.4/64,576.6 | 0.5 | 59.6/62.5/62.5 (5/5) | pass |
| mixed-50-c12-w1 | planned-blink | 1,034 | 1,034 | 2,067 | 13.3/16.1 | 12,472.3/14,940.3 | 0.3 | 43.3/44.2/44.2 (5/5) | pass |
| mixed-50-c12-w1 | rocksdb | 1,223 | 1,218 | 2,440 | 14.0/17.1 | 12,180.4/13,816.3 | 0.6 | 34.6/39.6/39.6 (5/5) | pass |
| query-c12 | main-btree | 613,409 | 0 | 613,409 | 22.5/23.1 | — | 99.7 | 79.4/79.6/79.6 (5/5) | pass |
| query-c12 | planned-blink | 615,787 | 0 | 615,787 | 22.3/22.8 | — | 99.6 | 58.8/58.9/58.9 (5/5) | pass |
| query-c12 | rocksdb | 795,294 | 0 | 795,294 | 14.1/14.5 | — | 99.8 | 50.1/50.1/50.1 (4/5 incomplete) | pass |
| write-c12-w1 | main-btree | 0 | 814 | 814 | — | 42,243.4/71,901.5 | 0.4 | 57.7/58.7/58.7 (5/5) | pass |
| write-c12-w1 | planned-blink | 0 | 961 | 961 | — | 25,341.0/28,142.7 | 0.3 | 38.5/39.5/39.5 (5/5) | pass |
| write-c12-w1 | rocksdb | 0 | 1,280 | 1,280 | — | 12,005.5/14,287.7 | 1.0 | 32.1/35.8/35.8 (5/5) | pass |

## Findings and limits

- OCI's largest throughput gap was pure write: planned-blink completed 8,802 durable transactions/s versus main-btree 3,728/s and RocksDB 7,804/s. OCI Point Get and Query16 were faster on RocksDB; Query16 reached 111,204/s versus 71,735/s on planned-blink. The OCI mixed comparison is only 1.5 s long; planned-blink had 16,580 aggregate ops/s with a 92.4 µs read p99 and 2.9 ms write p99, compared with RocksDB's 15,147 ops/s, 34.6 µs read p99, and 3.3 ms write p99.
- Debian's largest tail-latency separation was pure write: main-btree p99 71.9 ms, planned-blink 28.1 ms, RocksDB 14.3 ms. Durable write rates were 814, 961, and 1,280 transactions/s. RocksDB also led Get and Query16 throughput; planned-blink and main-btree Query16 were nearly tied.
- Read workloads used about 97–100% of reported machine CPU on both hosts. Durable write and mixed workloads used about 15–20% on OCI and below 1% on Debian. This is consistent with sync or serialization time being material, but does not isolate ZFS, ext4, device latency, or any single cause. Host-to-host deltas also include CPU count, architecture, filesystem, client count, and dodb worker-count differences.
- RSS is measurement-window sampled RSS, not VmHWM. Three Debian runs recorded 4 of 5 expected samples and are marked incomplete; OCI mixed runs have one sample each. Their RSS maxima are observed maxima, not guaranteed process peaks. Other reused OCI core runs had 10 samples each.
- RocksDB snapshots for these short writes showed no SST files, flushes, compactions, or stalls. This does not characterize longer runs. No 120-second stability test was performed.
- Validation passed for all 24 selected records. Get checked complete values and lengths; dodb also checked independently tracked revisions, while RocksDB revisions were unavailable. Query16 checked row count, keys, order, exclusive cursor boundary, and full values; no Query revision check is claimed. Mixed read-history validation passed, and each DB was reopened and checked. All selected records had zero measured read/write failures, errors, conflicts, overloads, or retries.

## Next candidate

Investigate WAL bytes emitted per committed mutation in the main-btree write path at [`dodb/crates/dodb-storage/src/btree/mod.rs`](../../../../crates/dodb-storage/src/btree/mod.rs), around WAL group append and page-image construction. In the width-1 pure-write run, the measured WAL delta was about 10.1 KiB/transaction for OCI main-btree versus 0.79 KiB for planned-blink, and 10.7 KiB versus 0.83 KiB on Debian. This repeated 12–13× gap is evidence to inspect WAL/page-image amplification, not proof of the cause. A follow-up should retain real sync and independent reopen validation, target at least a 50% reduction in main-btree WAL bytes per mutation, and require zero correctness failures with no p99 regression. No change is made here.

## Artifacts and provenance

- OCI selected raw records, logs, build details, hashes, seed/input selection, and validation: `oci/`.
- Debian raw JSONL, logs, attempt ledger, environment snapshots, summary, and trace checks: `debian/runs/` and adjacent files. Full DB files remain on Debian at `/home/namse/crossdb-quick-preliminary-2026-10-12/runs/db`; the full remote SHA-256 manifest and verification are preserved locally. Raw and log files copied into this report were checked against that manifest. A separate local full-run copy check is preserved as incomplete because not every DB directory was copied; the complete remote DB set and its full-manifest verification remain on Debian. One partial local DB directory from the stopped transfer is preserved and excluded from analysis.
- Both builds use clean source SHA `d3f4958cdcbba3508f79e6d5680b8d1dacbd3345`. Binary hashes and native build commands are in each environment's `*-build-provenance.txt`.
