# RocksDB service revision-counter diagnostic

Status: complete. This is a short, single-run-per-condition attribution study of the RocksDB service implementation. It measures the current revision-counter path; it does not characterize RocksDB's maximum performance or establish a stable performance regression.

## Provenance and scope

- Implementation verification SHA: `113b712de62e013bd299ba1769ed7ac953dc75bc` on `experiment/b-link-batched-engine-monorepo`, detached and clean on OCI.
- The instrumentation fix is a separate follow-up to the initial implementation commit `a05fa627fec3fbb2f3bb557f632b3c229bd37ec1`. The earlier ten runs are rejected because their blocking-dispatch timer began after task dispatch; their data is preserved and described in [rejected-attempts.md](rejected-attempts.md).
- The final report and artifacts are committed separately from the implementation. The report commit SHA is provided in the task completion summary.
- Host, toolchain, source, command, and binary details are in [host-and-source.txt](host-and-source.txt). The host was OCI `instance-20260923-1013`, ARM64 Linux 6.12, two Neoverse-N1 CPUs, ZFS `/bench/zfs/db`. Rust/Cargo 1.97.1, `aarch64-unknown-linux-gnu` with `+crc`, Release. Workload binary SHA-256: `ad49619b1b7d859d51c54193388fbc896e27239e27c6f9a72a793c45fcfb91e8`. Pinned RocksDB source was `abeebd9630f11bd08c28b7bd43c7bdfc62050654`, version 11.8.1; existing archive SHA-256: `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`. No RocksDB source or archive rebuild was performed.
- Both build and test commands used `dodb/scripts/run-oci-rocksdb-build.sh`, its pinned toolchain, shared Cargo target, sccache, and existing RocksDB archive. Helper records are in `oci/evidence/build-focused-tests/` and `oci/evidence/build-workload/`.
- The five profiles were run on RocksDB only: `rw95`, `rw80`, `rw50`, `rw20`, and `mt-mix`. Each profile has one OFF and one ON run at the same implementation SHA and with an independent database path. The OFF/ON order alternated by profile. The driver and workload recipe were not expanded or changed.
- Every run used seed `20261013`, 16 total clients, four clients per tenant in `mt-mix`, 10,000 seed rows per tenant, 512-byte values, 1 s warmup, and 5 s admission. Existing drain, error accounting, state validation, shutdown/reopen, and revalidation stayed enabled. Each full report preserves the actual API read/write mix and driver latency histograms.

## Instrumentation

Enable the study with `--research-metrics 1`; `--research-metrics 0` leaves collection disabled and reports `{"enabled": false, "reason": "disabled"}`. The normal workload and `DodbService::execute` request and response types stay the same. The metric stage is disabled during seed setup, switched to warmup after seeding, and switched to measurement only after warmup clients drain.

The RocksDB 11.8.1 pinned `include/rocksdb/perf_context.h` defines `PerfContext::key_lock_wait_time` as time waiting on key locks in the transaction lock manager and `key_lock_wait_count` as the number of acquisitions blocked by another transaction. Its header says these fields are collected beginning at `PerfLevel::kEnableTimeExceptForMutex`. The shim enables that level only during research warmup or measurement blocking work, resets the thread-local PerfContext immediately before each `GetForUpdate`, and reads the two fields immediately after that call on the same blocking thread. It stays disabled during seed setup and in OFF mode. This provides a direct RocksDB key-lock wait metric. The `GetForUpdate` duration remains a separate wall-clock measurement and is not labeled pure lock wait.

Metrics are recorded in per-tenant atomic counters and fixed 64-bucket base-2 nanosecond histograms. There is no per-request output or unbounded sample vector, and no instrumentation mutex serializes requests. Tenant collection is capped at 64; the reports show zero omitted tenant events. Counts, total nanoseconds, and means are accumulated directly. Percentiles are approximate bucket upper bounds; all ON/OFF driver request latency percentiles in `report.json` retain the driver's original bounded sample calculation. `total_ms_across_requests` is a sum of overlapping request intervals, not elapsed wall time.

The metric meanings are:

- `read_semaphore_wait` and `write_semaphore_wait`: time awaiting the shared blocking-operation semaphore, separated by operation class.
- `blocking_dispatch_wait`: time from immediately before `spawn_blocking` until its closure starts. The permit has already been acquired and remains held during this interval and the operation.
- `user_key_get_for_update_duration` and `revision_counter_get_for_update_duration`: full RocksDB calls for user keys and the tenant metadata counter, respectively.
- `user_key_lock_wait` and `revision_counter_lock_wait`: the corresponding direct `PerfContext::key_lock_wait_time` deltas; their count fields are in `outcomes`.
- `condition_check_duration`: condition evaluation only. `mutation_construction_duration`: revision-based mutation encoding and transaction `Put` calls after the counter read.
- `commit_duration`: successful `Commit` calls only. A rejected condition records `rollback_duration`, not commit time. Explicit read-only rollbacks are also recorded; implicit rollback during other error unwinding is not represented by this metric.
- `counter_held_duration_approx`: from return of the counter `GetForUpdate` to return of `Commit`. It approximates the counter lock hold interval and can differ slightly from the internal unlock instant.
- `service_request_duration`: the whole service request, including semaphore waits and response processing.

Every metric and phase distribution, including count, total, mean, p50, p95, and p99, is in [stage-metrics.csv](stage-metrics.csv) and the raw reports. Workload counts and throughput are in [condition-summary.csv](condition-summary.csv). Individual commands, stdout, JSON reports, exit codes, and before/after binary hashes are in `oci/runs/<scenario>-<mode>/`. Run-specific RocksDB directories remain on OCI at `/bench/zfs/db/dodb-rocksdb-revision-counter-diagnostic-113b712d/runs/`; the copied evidence archive omits only those database directories.

## OFF and ON results

Rates use the existing measured admission-plus-drain denominator. Read/write percentages are actual API-call ratios, including preparatory Gets. `write tx/s` counts successful Put/Delete/Transact requests; rejected conditions remain separate. The paired request-prefix fingerprints matched for all runs, and each report passed both the independent state validation and shutdown/reopen validation.

| Profile | Actual read/write OFF → ON | API/s OFF → ON | Successful write tx/s OFF → ON | Successful write tx count OFF → ON | Rejected conditions OFF → ON | Conflicts OFF → ON | API/s delta ON vs OFF |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RW95 | 95.04/4.96% → 95.02/4.98% | 19,922 → 18,929 | 910 → 865 | 4,554 → 4,327 | 395 → 384 | 0 → 0 | -4.99% |
| RW80 | 80.55/19.45% → 80.50/19.50% | 5,185 → 5,256 | 939 → 955 | 4,703 → 4,781 | 346 → 350 | 0 → 0 | +1.37% |
| RW50 | 49.31/50.69% → 49.48/50.52% | 1,910 → 1,824 | 926 → 882 | 4,641 → 4,417 | 213 → 199 | 0 → 0 | -4.52% |
| RW20 | 19.54/80.46% → 19.58/80.42% | 1,347 → 1,296 | 987 → 949 | 4,951 → 4,760 | 483 → 468 | 0 → 0 | -3.75% |
| MT-MIX | 71.25/28.75% → 71.24/28.76% | 7,649 → 7,599 | 2,042 → 2,030 | 10,221 → 10,161 | 780 → 773 | 5 → 5 | -0.65% |

Overloads and other errors were zero in all ten runs. The MT-MIX ratio is aggregated across four distinct profiles and is descriptive only.

The request-latency p95/p99 comparison below is from the workload driver's original bounded request samples, not the approximate research histograms.

| Profile | Get p95/p99 OFF → ON (ms) | Successful write transaction p95/p99 OFF → ON (ms) | Whole workflow p95/p99 OFF → ON (ms) |
| --- | ---: | ---: | ---: |
| RW95 | 1.320/2.164 → 1.343/2.113 | 13.987/20.539 → 14.298/22.685 | 2.134/7.173 → 2.049/6.687 |
| RW80 | 4.023/5.414 → 3.895/4.998 | 21.976/35.386 → 20.533/33.483 | 11.077/21.442 → 10.883/20.645 |
| RW50 | 9.358/10.971 → 9.829/11.637 | 27.820/44.473 → 30.051/48.419 | 22.890/40.623 → 24.578/40.659 |
| RW20 | 14.028/16.316 → 16.137/23.307 | 28.508/44.934 → 29.560/46.740 | 29.219/44.681 → 30.581/45.326 |
| MT-MIX | 2.609/3.245 → 2.618/3.273 | 7.381/10.069 → 7.406/10.064 | 3.029/4.511 → 3.033/4.379 |

These are five pairs of one-shot comparisons. The throughput deltas range from -4.99% to +1.37%; request tails also move in both directions. This is too little evidence to call an instrumentation regression or improvement. The ON path enables thread-local PerfContext timing around each `GetForUpdate` and performs atomic histogram updates, so some measurement cost is expected; its stable cost is not estimated here.

## Measured phase distributions

Means below are exact sum/count values; p95 values are approximate bucket upper bounds. The complete p50/p95/p99 distributions and accumulated totals are in `stage-metrics.csv`.

| Profile | Counter calls | Counter `GetForUpdate` mean / p95 ms | Pure counter lock wait mean / p95 / p99 ms | User-key `GetForUpdate` mean ms | User-key lock wait mean ms | Commit mean ms | Counter held mean ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RW95 | 4,327 | 2.944 / 16.777 | 2.936 / 16.777 / 16.777 | 0.325 | 0.318 | 1.148 | 1.159 |
| RW80 | 4,781 | 4.294 / 16.777 | 4.284 / 16.777 / 33.554 | 0.620 | 0.611 | 1.075 | 1.087 |
| RW50 | 4,417 | 4.771 / 16.777 | 4.761 / 16.777 / 33.554 | 0.808 | 0.801 | 1.184 | 1.196 |
| RW20 | 4,760 | 4.583 / 16.777 | 4.573 / 16.777 / 33.554 | 0.648 | 0.641 | 1.108 | 1.119 |

For each successful write, the counter wait mean is close to the entire counter `GetForUpdate` call mean and is about 2.5–4.5 times the successful commit-call mean. User-key processing also contains lock wait, but its mean is lower here. Condition checks averaged below 0.001 ms per request; mutation construction averaged 0.010–0.011 ms. The current diagnostic therefore directly observes substantial tenant-counter key-lock wait alongside roughly 1.1–1.2 ms successful commit calls. It does not establish what portion of commit time is WAL sync versus other commit work.

The shared semaphore and blocking dispatch had these ON means (p95 in parentheses), in milliseconds:

| Profile | Read semaphore wait | Write semaphore wait | Blocking dispatch wait |
| --- | ---: | ---: | ---: |
| RW95 | 0.202 (0.524) | 0.221 (1.048) | 0.052 (0.262) |
| RW80 | 0.822 (4.194) | 0.823 (4.194) | 0.030 (0.262) |
| RW50 | 2.894 (8.389) | 2.936 (8.389) | 0.019 (0.066) |
| RW20 | 5.079 (8.389) | 5.135 (8.389) | 0.026 (0.066) |

Read semaphore waiting is observable in all four profiles. A Get or Query performs a blocking tenant-path existence check and then a blocking storage operation, so read semaphore metric counts are about twice the API read count. Dispatch wait is materially smaller than semaphore wait on average, though it has a measurable tail.

A mutation acquires the same eight-permit semaphore before `spawn_blocking`; the permit is moved into that closure and stays live through the transaction. Therefore every measured transaction counter-lock wait occurs while its blocking-operation permit is held. The separate semaphore histogram shows additional waiting before a permit is obtained. This is direct control-flow evidence; the run does not attribute every semaphore wait to a particular counter-lock waiter.

## MT-MIX tenant differences

Each tenant has a separate RocksDB database and its own revision-counter key. The ON mean counter lock wait, counter call, and commit call were:

| Tenant | Profile | Successful writes / condition rejections | Counter lock wait count | Counter call mean ms | Pure counter lock wait mean ms | Commit mean ms | Counter held mean ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| tenant_1 | RW95 | 588 / 48 | 67 | 0.089 | 0.083 | 1.351 | 1.370 |
| tenant_2 | RW80 | 2,289 / 180 | 847 | 0.361 | 0.355 | 1.339 | 1.354 |
| tenant_3 | RW50 | 3,606 / 269 | 3,244 | 1.803 | 1.794 | 1.365 | 1.380 |
| tenant_4 | RW20 | 3,678 / 281 | 3,530 | 2.626 | 2.617 | 1.386 | 1.399 |

The counter wait is low for the read-heavy tenant and exceeds commit time for the two write-heavy tenants. These are tenant-local counters, not a shared cross-DB lock. Across tenants, read semaphore wait means were 0.513–0.629 ms and write semaphore means 0.526–0.668 ms; blocking dispatch means were 0.044–0.047 ms.

## Validation, attempts, and limits

Focused verification at final implementation SHA `113b712de62e013bd299ba1769ed7ac953dc75bc`:

- Planned 4 tests, executed 4, passed 4, failed 0, not reached 0. Three `research_metrics` tests cover metric OFF/ON logical results, exact fixed-request expected counts and condition rejection split, warmup/measurement boundary, concurrent request accounting, per-tenant attribution, disabled metrics representation, and snapshot access after shutdown. The existing common B-link/native/RocksDB transaction contract passed 1/1.
- Local `cargo check --locked -p dodb-rocksdb --bin service_workload --tests` and crate formatting passed. The local machine does not have the 1.2 GB pinned archive; runtime tests used the existing OCI archive and build helper.
- Planned 10 OCI workload runs, executed 10, passed 10, failed 0, not reached 0. Every OFF/ON pair had matching first-100 workflow fingerprints. Every run passed independent expected-state validation and shutdown/reopen validation. All before/after binary hashes matched.
- One orchestration preflight failed before launching any workload because `runs/` had not been created; it is recorded at `dodb-service-revision-counter-diagnostic-a05fa627/oci/attempts/preflight-1/`. The first implementation's 10 workload reports were then rejected for the dispatch timer defect and preserved under the same directory. The corrected SHA's 10 runs were executed once each without retry.

This study does not identify what fraction of counter `GetForUpdate` call time would be seen under other workloads, distinguish commit WAL sync from other commit internals, establish long-run LSM/compaction behavior, compare B-link again, or prove the OFF/ON performance difference statistically. The approximate histogram percentiles have bucket-width error, and interval totals overlap across concurrent requests.

## Questions and next change candidate

1. **Counter versus user-key and commit:** in the single-tenant ON runs, direct counter lock wait averaged 2.936–4.761 ms per successful write, versus 0.318–0.801 ms for user-key lock wait and 1.075–1.184 ms for commit. MT-MIX varied by tenant from 0.083 to 2.617 ms counter wait.
2. **Pure lock wait:** yes. `PerfContext::key_lock_wait_time` was enabled and differenced around each `GetForUpdate` on the same blocking thread. The full call duration is reported separately.
3. **Blocking permit ownership:** yes. The write path holds a blocking permit while awaiting the counter lock inside its `spawn_blocking` transaction closure.
4. **Read semaphore wait:** yes, including all four single-tenant profiles and all MT-MIX tenants.
5. **Single versus MT-MIX:** single-tenant counter waits were 2.936–4.761 ms; MT-MIX ranged 0.083–2.617 ms by tenant, with the RW95/RW80 tenants lower than commit and RW50/RW20 tenants higher.
6. **Instrumentation cost:** the one-run API throughput shifts ranged from -4.99% to +1.37%, with mixed latency shifts. These do not support a stable overhead estimate.
7. **Not established:** maximum RocksDB service capability, long-run stability, WAL sync's share of commit time, causal performance significance, or behavior outside the frozen synthetic workloads.

Next candidate: test a narrow per-tenant revision-range reservation mechanism in `dodb-rocksdb`, so transactions consume revisions from a previously persisted range instead of contending on the same counter key for each write. Keep user-key locking, transaction mutations, the API, and other backends unchanged in that experiment. The small follow-up should compare one fixed reservation size against the current allocator, verify unique monotone committed revisions and reopen behavior, and rerun this attribution workload before considering broader changes. Unused reserved IDs may create gaps and need an explicit contract decision.

## Artifacts

- `condition-summary.csv`: OFF/ON counts, actual mix, rates, validation, and pair deltas.
- `stage-metrics.csv`: warmup and measurement distributions for every metric and tenant, including count, total, mean, p50, p95, and p99.
- `oci/runs/<scenario>-<mode>/`: command, stdout, report JSON, exit status, and executable SHA-256 before and after. Per-run DBs are retained at the OCI path listed above.
- `oci/evidence/build-focused-tests/` and `oci/evidence/build-workload/`: exact helper environment, build/test logs, status, duration, shared-cache counters, archive provenance, and checksums.
- `oci/evidence.tar.gz` SHA-256: `e3a10f4f3949802a0ef0ac0e3996942bbee3b1424f45e6e66d0c17d91e68f016`.
