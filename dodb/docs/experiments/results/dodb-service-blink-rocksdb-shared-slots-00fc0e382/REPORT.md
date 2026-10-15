# B-link and RocksDB Shared-Slot Comparison

## Scope and decision boundary

This report compares three fixed service profiles at implementation SHA
`00fc0e38246069869f08db8cac7e430d31fb7212`:

1. B-link with its existing coordinator and page cache.
2. RocksDB TransactionDB with reserved revision ranges of 4,096, initialized
   tenant fast path, and shared admission capped at eight blocking operations.
3. The same RocksDB configuration with shared admission capped at sixteen
   blocking operations.

There are 15 OCI performance runs: one five-second run for every profile in
RW95, RW80, RW50, RW20, and MT-MIX. These are current-service observations,
not repeated estimates. They do not establish a stable rank, a maximum engine
throughput, or a product-wide backend choice. No default or product backend
was changed.

## Changes and focused checks

The only runtime setting added is the RocksDB shared blocking-operation
limit. `--shared-blocking-slots` defaults to eight and accepts sixteen for the
comparison. Zero, use with split 2/6 admission, and use for B-link are rejected.
The emitted report records the selected mode and effective limit. Split 2/6
and its previous result remain unchanged and were not measured again.

Focused checks passed on OCI Release at the implementation SHA:

- 21 `dodb-rocksdb` library tests, including default shared eight, shared
  sixteen, zero rejection, a held-closure concurrency bound, permit retention
  through closure completion, request cancellation, shutdown drain, and
  reopen.
- 2 shared read contract tests.
- 1 shared transaction contract test.
- Total: 24 passed, 0 failed, 0 not reached. Crash tests were not run.
- Local formatting and whitespace checks passed. Local Rust tests were not
  run because this checkout has no RocksDB 11.8.1 archive; no local RocksDB
  build was attempted.

Three 10,000-row smoke runs (B-link, shared 8, shared 16) each passed replay
and shutdown/reopen validation for all four tenants. The smoke used four
clients, 50 ms warmup, and 100 ms measurement and is separate from the 15
performance runs. Invalid option checks for zero slots, split plus shared
slots, and B-link plus 16 slots all exited with status 2. One smoke runner
attempt failed while formatting the command, before launching a workload; it
is retained under `oci/dodb-service-shared-slots-00fc0e382/smoke/attempt-01.txt`.
The corrected runner completed all three smoke cases without rerunning a
workload that had started.

The later local summary checker first counted the CSV header as a run and
expected 16 rows; the CSV contains 15 data rows. It was corrected to expect
the 15 planned runs and validated the preserved reports without rerunning
workloads. Details are in [`validation-attempts.md`](validation-attempts.md).

## Environment and workload

- OCI `instance-20260923-1013`, ARM64 Neoverse-N1, two online CPUs, Oracle
  Linux 9.8, Linux 6.12, 10.64 GiB physical memory.
- ZFS `/bench/zfs/db`; at host capture it was 149 GiB total, 120 GiB used,
  29 GiB available. Filesystem and host details are in
  [`host-and-source.txt`](oci/dodb-service-shared-slots-00fc0e382/host-and-source.txt).
- Rust/Cargo 1.97.1, `aarch64-unknown-linux-gnu`, repository `+crc` target
  flags, Release profile. The existing shared Cargo target and sccache were
  reused. Existing RocksDB 11.8.1 archive SHA-256:
  `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
  No new Cargo target or RocksDB rebuild was used.
- Workload boundary: the same `DodbService::execute` request/response API,
  schedule, 16 total clients, binary 8-byte PK plus 8-byte SK, 512-byte
  values, seed `20261013`, 10,000 initial rows per tenant, 1 second warmup,
  5 seconds admission, and `call_cap=1,000,000`.
- MT-MIX used four tenants with four clients each and profiles RW95, RW80,
  RW50, and RW20. Each run began from its own new DB directory.
- Preparation Gets remain included in read counts, actual read/write shares,
  and workflow latency. Get/Query16 distribution, Put/Delete, condition
  kinds, intentional rejections, transaction widths 1/4/8, and the existing
  deadline, drain, replay, and reopen checks were retained. All five write
  categories and all three condition kinds were reached in every tenant.
- Detailed PerfContext, read-stage, and detailed write metrics were off.
  Request counts, correctness checks, latency sampling, CPU, and RSS
  collection were on. WAL and file-backed storage remained active; RocksDB
  transaction and revision reservation commits retained `sync=true`.
- Runtime and engine settings were unchanged: 16 Tokio worker threads;
  B-link max open shards 16, cache capacity 128 pages, queue 256, groups up
  to 64 requests or 4 MiB, zero collection delay; RocksDB uses its existing
  TransactionDB defaults, no compression, and no newly set cache or memory
  budget. Its shared semaphore limit is the selected 8 or 16. The limit is
  an admission ceiling; actual blocking concurrency was not assumed to equal
  it.
- Detailed command, stdout, JSON report, binary hash, and exit status for each
  performance run are retained under `oci/dodb-service-shared-slots-00fc0e382/runs/`.
  The 15-run order rotated by scenario and every run was serial.

The seeded workload is small enough for data and operating-system caches to
hold much of its working set. These file-backed, synced runs do not measure a
dataset larger than memory or prove that data stayed off-cache or off-disk.

## Single-tenant throughput and latency

Rates use the existing measurement-plus-drain denominator. Read/write shares
include preparation Gets. Latency values are milliseconds. `Get`, `Query16`,
and successful `Write` show `p50/p95/p99`; workflow shows `p95/p99`. Error
counts are intentional rejection / natural conflict / overload / other.

| Scenario | Profile | Configuration | Actual R/W % | API/s | Successful writes/s | Get p50/p95/p99 ms | Query16 p50/p95/p99 ms | Write p50/p95/p99 ms | Workflow p95/p99 ms | Errors R/C/O/E |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RW95 | RW95 | B-link | 95.00/5.00 | 44,562 | 2,048 | 0.005/0.090/0.196 | 0.015/0.092/0.187 | 6.093/11.852/15.451 | 0.705/7.911 | 896/0/0/0 |
| RW95 | RW95 | RocksDB shared 8 | 95.01/4.99 | 38,938 | 1,786 | 0.222/0.694/1.417 | 0.233/0.615/1.113 | 2.369/6.021/10.943 | 1.510/4.283 | 776/0/0/0 |
| RW95 | RW95 | RocksDB shared 16 | 95.01/4.99 | 41,029 | 1,883 | 0.107/1.161/2.091 | 0.129/0.901/1.835 | 2.842/11.527/24.764 | 1.868/3.752 | 828/0/0/0 |
| RW80 | RW80 | RocksDB shared 8 | 80.03/19.97 | 13,112 | 2,435 | 0.499/1.670/2.240 | 0.535/1.692/2.314 | 2.786/8.738/17.893 | 3.627/8.649 | 919/0/0/0 |
| RW80 | RW80 | RocksDB shared 16 | 80.12/19.88 | 14,373 | 2,658 | 0.038/0.295/0.500 | 0.057/0.336/0.498 | 2.143/26.845/57.709 | 2.608/23.553 | 997/0/0/0 |
| RW80 | RW80 | B-link | 80.18/19.82 | 14,862 | 2,742 | 0.006/0.041/0.123 | 0.018/0.057/0.150 | 4.709/9.200/12.100 | 6.211/8.878 | 1,017/0/0/0 |
| RW50 | RW50 | RocksDB shared 16 | 49.71/50.29 | 5,501 | 2,589 | 0.042/0.279/0.456 | 0.063/0.333/0.485 | 2.257/27.783/62.642 | 15.892/50.695 | 886/0/0/0 |
| RW50 | RW50 | B-link | 50.04/49.96 | 6,502 | 3,024 | 0.008/0.049/0.108 | 0.021/0.075/0.145 | 4.459/8.223/10.368 | 7.170/9.879 | 1,124/0/0/0 |
| RW50 | RW50 | RocksDB shared 8 | 49.74/50.26 | 5,128 | 2,422 | 1.458/3.377/5.024 | 1.534/3.443/5.019 | 3.897/10.638/19.125 | 8.167/16.257 | 777/0/0/0 |
| RW20 | RW20 | B-link | 20.04/79.96 | 4,178 | 3,097 | 0.017/0.073/0.134 | 0.032/0.094/0.175 | 4.383/7.780/10.803 | 7.633/10.769 | 1,220/0/0/0 |
| RW20 | RW20 | RocksDB shared 8 | 20.11/79.89 | 3,702 | 2,749 | 2.121/3.843/4.774 | 2.082/3.921/5.468 | 4.229/10.137/19.477 | 10.030/18.950 | 1,041/0/0/0 |
| RW20 | RW20 | RocksDB shared 16 | 20.14/79.86 | 3,797 | 2,818 | 0.049/0.367/0.529 | 0.075/0.381/0.531 | 2.241/26.635/55.959 | 23.881/53.529 | 1,076/0/0/0 |

The source CSV retains full request counts, latency sample counts, condition
kinds, and all five write-category attempt counts for each row. The reservoir
cap is 20,000 observations per latency category. RW20 read tails have only
hundreds of samples, so those percentiles are less stable than the high-count
RW95 estimates.

## MT-MIX per-tenant results

Each tenant used four clients. All tenants are shown separately; tenant totals
are not used to claim fairness. Columns and units match the single-tenant
table.

| Configuration | Tenant / profile | Actual R/W % | API/s | Successful writes/s | Get p50/p95/p99 ms | Query16 p50/p95/p99 ms | Write p50/p95/p99 ms | Workflow p95/p99 ms | Errors R/C/O/E |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RocksDB shared 8 | 1 / RW95 | 95.02/4.98 | 4,693 | 215 | 0.666/1.565/2.272 | 0.690/1.599/2.581 | 2.205/3.899/4.890 | 2.024/3.455 | 96/0/0/0 |
| RocksDB shared 16 | 1 / RW95 | 95.01/4.99 | 14,639 | 672 | 0.069/0.541/1.036 | 0.098/0.569/0.965 | 2.282/4.844/7.620 | 1.349/3.324 | 293/0/0/0 |
| B-link | 1 / RW95 | 95.00/5.00 | 14,235 | 654 | 0.005/0.018/0.071 | 0.017/0.051/0.601 | 4.765/9.470/12.268 | 2.042/6.148 | 288/0/0/0 |
| RocksDB shared 8 | 2 / RW80 | 80.01/19.99 | 3,616 | 671 | 0.632/1.495/2.214 | 0.655/1.553/2.359 | 2.553/4.418/6.426 | 3.335/4.725 | 259/0/0/0 |
| RocksDB shared 16 | 2 / RW80 | 80.01/19.99 | 5,427 | 1,008 | 0.109/0.601/1.126 | 0.135/0.634/1.171 | 2.715/5.761/8.943 | 3.674/5.772 | 386/0/0/0 |
| B-link | 2 / RW80 | 79.98/20.02 | 3,977 | 740 | 0.006/0.040/0.290 | 0.020/0.086/0.665 | 4.534/8.574/11.285 | 5.700/8.662 | 283/0/0/0 |
| RocksDB shared 8 | 3 / RW50 | 49.61/50.39 | 2,214 | 1,037 | 0.633/1.501/2.103 | 0.644/1.567/2.234 | 2.848/4.725/7.031 | 4.418/6.230 | 387/10/0/0 |
| RocksDB shared 16 | 3 / RW50 | 49.90/50.10 | 2,386 | 1,111 | 0.143/0.647/1.220 | 0.174/0.743/1.498 | 2.938/6.298/9.548 | 5.327/8.100 | 415/8/0/0 |
| B-link | 3 / RW50 | 50.05/49.95 | 1,642 | 761 | 0.009/0.057/0.331 | 0.025/0.121/0.463 | 4.499/8.436/11.531 | 7.185/9.893 | 289/8/0/0 |
| RocksDB shared 8 | 4 / RW20 | 19.97/80.03 | 1,586 | 1,180 | 0.682/1.501/3.744 | 0.651/1.576/2.528 | 2.922/5.075/7.345 | 5.172/7.387 | 446/0/0/0 |
| RocksDB shared 16 | 4 / RW20 | 19.99/80.01 | 1,546 | 1,151 | 0.150/0.888/1.535 | 0.197/0.866/1.883 | 2.960/6.485/9.649 | 6.274/9.484 | 429/0/0/0 |
| B-link | 4 / RW20 | 20.12/79.88 | 1,037 | 770 | 0.023/0.073/0.430 | 0.037/0.123/0.445 | 4.561/7.928/10.921 | 7.852/10.573 | 292/0/0/0 |

## Shared eight to sixteen: effect and resource cost

On these one-off runs, moving from eight to sixteen shared slots materially
lowered Get and Query16 p95 for RW80, RW50, and RW20, while RW95 read p95/p99
rose. Successful write throughput increased by about 5%, 9%, 7%, and 2% in
RW95, RW80, RW50, and RW20. Successful-write p95 rose from 6.02 to 11.53 ms
in RW95, 8.74 to 26.85 ms in RW80, 10.64 to 27.78 ms in RW50, and 10.14 to
26.64 ms in RW20. RW20 workflow p95 rose from 10.03 to 23.88 ms.

MT-MIX read p95 improved for every tenant with sixteen slots. Throughput
changed unevenly: API/s increased for tenants 1–3 and fell about 3% for tenant
4. Successful-write p95 rose for all four tenants. Thus the eight-slot cap
constrained some read-heavy and mixed tenant outcomes, but sixteen did not
improve every tenant or tail.

| Scenario | Profile | CPU % of one core, 8 → 16 | Peak RSS MiB, 8 → 16 | RSS samples |
| --- | --- | ---: | ---: | ---: |
| RW95 | single | 105.3 → 111.2 | 39.0 → 43.2 | 24 / 24 |
| RW80 | single | 72.3 → 74.8 | 40.2 → 42.8 | 24 / 24 |
| RW50 | single | 47.6 → 49.1 | 38.3 → 41.7 | 24 / 24 |
| RW20 | single | 43.2 → 46.8 | 42.1 → 43.9 | 24 / 24 |
| MT-MIX | four tenants | 79.2 → 102.2 | 65.4 → 75.5 | 24 / 24 |

The process sampler reads `/proc/self/stat` and `/proc/self/status`; CPU is
shown as percent of one core and the host has two cores. RSS is a sampled peak
at 200 ms intervals and can miss shorter peaks. B-link used 39–68% of one
core and 100.9–128.7 MiB RSS in single-tenant runs, and 72.5% of one core and
190.6 MiB in MT-MIX. These are process-level samples, not an engine memory
budget or device-I/O accounting.

## What remains after wrapper improvements

The earliest serialized revision-counter wrapper added a shared metadata
transaction to every write and serialized revision allocation. Its same-
RocksDB comparison showed reserved-range write rates lower in RW95 but higher
in RW80/RW50/RW20 and MT-MIX; that baseline is not a fair B-link comparison.
The initialized-tenant fast path removed an unnecessary path-existence
blocking submission for warm Get/Query/Scan. Its one-run evidence showed
large RW95 changes but small or mixed changes elsewhere, and it did not remove
contention among shared slots. See the
[revision reservation report](../dodb-service-revision-range-d9c5e01e/REPORT.md)
and [tenant fast-path report](../dodb-service-read-tenant-fastpath-b4325ee98/REPORT.md).

The earlier split 2/6 result remains a separate, unchanged comparison: it
lowered read tails in most cases but reduced successful-write throughput in
RW80/RW50/RW20 and several MT-MIX tenants. It was not repeated here. See the
[blocking-admission report](../dodb-service-blocking-admission-85cadf099/REPORT.md).

With reservation and tenant fast path enabled, the same-SHA comparison still
shows B-link with markedly lower Get/Query16 tails in the four single-tenant
profiles and all MT-MIX tenants. B-link also had higher API and successful
write rates than either RocksDB profile in each single-tenant run. RocksDB
shared 16 had lower successful-write p50 than B-link in those single-tenant
runs, while B-link generally had lower successful-write p95/p99; neither
pattern applies uniformly to MT-MIX. In MT-MIX, RocksDB shared 16 had higher
per-tenant API and successful-write rates than B-link in this run, while
B-link retained lower read tails and RocksDB shared 16 had lower write tails.

These are differences among current service implementations, including
B-link grouping/page-cache behavior and RocksDB TransactionDB/semaphore
behavior. They are not engine-only limits. The one 5-second sample per
condition cannot establish stable rankings or small effects. The initial data
set is cache-sized, and no long-run, larger-than-memory, or production trace
test was run. Split 2/6 was not remeasured, and this evidence does not justify
changing a default or product backend.

## Validation and retained evidence

| Check or run | Planned | Executed | Passed | Failed | Not reached |
| --- | ---: | ---: | ---: | ---: | ---: |
| Local formatting / whitespace | 2 | 2 | 2 | 0 | 0 |
| OCI focused library and contracts | 24 tests | 24 | 24 | 0 | 0 |
| Smoke configurations | 3 | 3 | 3 | 0 | 0 |
| Invalid option combinations | 3 | 3 | 3 rejected as expected | 0 | 0 |
| OCI performance conditions | 15 | 15 | 15 | 0 | 0 |

All five scenarios matched the schedule fingerprint and each client's first
100 workflow-input fingerprint across B-link, shared 8, and shared 16. All
15 state replays and all 15 shutdown/reopen validations passed. No performance
run was rerun; split 2/6, crash coverage, and Debian checks were not reached by
design.

Implementation verification SHA: `00fc0e38246069869f08db8cac7e430d31fb7212`.
Workload binary SHA-256:
`9c91b54718d19f9fd6a78274db5fda05beafe9da2c39e9f70cb2992203323228`.
The final report commit has a separate SHA recorded in Git history. Raw
reports and compact evidence are under
[`oci/dodb-service-shared-slots-00fc0e382/`](oci/dodb-service-shared-slots-00fc0e382/);
fresh measurement DBs remain on OCI under
`/bench/zfs/db/dodb-service-shared-slots-00fc0e382/runs/`.
