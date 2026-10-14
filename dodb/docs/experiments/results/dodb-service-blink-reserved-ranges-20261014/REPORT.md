# B-link and Reserved-Range RocksDB Service Comparison

## Result scope

This report records two narrow revision-allocation failure checks and a new
same-source-SHA service workload comparison between B-link and RocksDB with
revision reservation explicitly selected at a range size of 4,096. The prior
serialized-counter RocksDB mode was not measured. The B-link measurements are
new runs at this SHA; no earlier B-link result is reused.

The implementation verification SHA for the final checks, build, smoke runs,
and all ten accepted measurements is
`3dd51326565fad6638bbcd2420208faa0b55a9f7`. The final report and its evidence
are committed separately. The workload executable SHA-256 before and after
every accepted OCI run was
`a80738c4dd6553e79dd6faa86e6313acce4056bbca8f1246f1042cc8a93a76f0`.

The results describe the current service implementations under the stated
settings and one five-second measurement per condition. They do not establish
stable rankings, long-run behavior, production workload performance, or a
cause for backend differences.

## Reservation failure checks

The checks use test-only callbacks at two narrow points. The production path
continues to pass no-op callbacks.

### User transaction rollback after token assignment

The deterministic range-size-four check first commits an initial value at
revision 1. The tested mutation has a passing `RevisionEquals` condition on
that same key. It acquires the user-key lock, confirms the condition, and
receives revision 2 from the durable range. A test callback then returns an
error before mutation writes and user commit. The key remains at its initial
value and revision 1. The next successful write receives revision 3, so the
failed request's assigned token is not reused. After normal shutdown and
reopen, the next write receives revision 5, above the persisted high-water
mark of 4.

This is a forced rollback after token assignment. It is not a user-transaction
commit I/O failure test.

### Reservation commit result not confirmed to the caller

The check commits the reservation metadata transaction successfully with the
existing synced path, then uses a test callback to return an API-level error
before publishing the candidate range in memory. The caller receives an error
and the user key remains unchanged. The next write reads the persisted
high-water mark, reserves the following range, and uses revision 5. After
normal reopen, the next successful write uses revision 9. The unconfirmed
range 1 through 4 is not used by a user mutation.

This injects uncertainty at an API boundary after a successful metadata
commit. It does not reproduce storage failure, an actual commit I/O error,
process termination, or power loss.

## Contract and local checks

At the implementation SHA, local focused checks passed:

- 11 `dodb-rocksdb` library tests, including both new failure checks, range
  uniqueness, abandoned tokens, reopen, mode switching, overflow, and corrupt
  metadata.
- 2 shared read-contract tests.
- 4 research-metrics tests.
- 1 shared transaction-contract test, which applies the common contract to
  native, B-link, serialized RocksDB, and reserved-range RocksDB services.
- `cargo fmt --package dodb-rocksdb -- --check`.

The final focused local invocation passed 18/18 tests. OCI Release focused
checks at the same SHA also passed 18/18 tests using `--locked --release`.
No existing crash-contract test or broader repository suite was repeated.

## OCI environment and build provenance

- Host: `instance-20260923-1013`, ARM64, Linux 6.12, two online CPUs.
- Filesystem: ZFS mounted at `/bench/zfs/db`.
- Toolchain: Rust/Cargo 1.97.1; target `aarch64-unknown-linux-gnu` with the
  repository's `+crc` target flag; release profile retained `debug=true`.
- RocksDB: existing 11.8.1 source SHA
  `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; existing archive SHA-256
  `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
- Cargo reused the shared release target and sccache through
  `dodb/scripts/run-oci-rocksdb-build.sh`. No new target was created and
  RocksDB was not rebuilt.
- Helper run IDs were
  `blink-ranges-3dd513265-focused` and
  `blink-ranges-3dd513265-workload`. Their exact cargo commands, environment,
  sccache counters, elapsed time, exit status, and helper checksums are in the
  evidence archive.

```sh
DODB_BUILD_RUN_ID=blink-ranges-3dd513265-focused bash dodb/scripts/run-oci-rocksdb-build.sh test --locked --release -p dodb-rocksdb --lib --test transaction_contract --test read_contract --test research_metrics
DODB_BUILD_RUN_ID=blink-ranges-3dd513265-workload bash dodb/scripts/run-oci-rocksdb-build.sh build --locked --release -p dodb-rocksdb --bin service_workload
```
- A dedicated clean detached checkout at
  `/bench/zfs/db/dodb-blink-reserved-ranges-20261014-3dd513265` used the exact
  implementation SHA. The existing checkout was left untouched.
- OCI run outputs and fresh per-run DB directories remain under
  `/bench/zfs/db/dodb-blink-reserved-ranges/3dd513265/`. The earlier
  `82160803` checkout and run artifacts are preserved as excluded attempts.
  The accepted evidence archive
  excludes the DB directories and contains commands, reports, logs, exit
  statuses, binary hashes, and build-helper records.

## Backend and measurement settings

Every measured request went through the shared `DodbService::execute` path,
using the same request/response API, response budget, deterministic schedule,
seed, key generation, values, deadlines, drain rule, latency collector, and
independent replay validator.

| Setting | B-link | RocksDB |
| --- | --- | --- |
| Data path | Fresh directory per run | Fresh directory per run |
| Database cache / executor | 128-page cache; 16 max open shards | 8 blocking operations |
| Write coordinator | Queue 256; max group 64 requests / 4 MiB; no collection delay | TransactionDB; revision allocator `reserved-ranges`; range 4,096 |
| Write durability | Existing persistent file and WAL sync path | Existing WAL; user transactions and reservation commits use `sync=true` |

The shared Tokio runtime used 16 worker threads. No slot limit, group size,
cache, delay, durability, or engine setting was tuned for this comparison.
The default serialized RocksDB mode remains unchanged and was not included as
a third measurement target.

Detailed RocksDB PerfContext timing and its lock-wait, allocator, and dispatch
histograms were disabled with `--research-metrics 0`. The driver retained
request counts, correctness validation, workflow and operation latency
sampling, and process-resource sampling. There was no per-request debug
output. The standard workload latency sampler is capped at 20,000 observations
per latency category; raw reports include count and sample count.

The ten measurement conditions were run once each, serially, in alternating
backend order. The sequence was RW95 B-link/RocksDB, RW80 RocksDB/B-link, RW50
B-link/RocksDB, RW20 RocksDB/B-link, then MT-MIX B-link/RocksDB. Every run
used:

- 1 second warmup and 5 seconds admission, followed by the existing drain.
- 16 total clients; MT-MIX assigned four clients to each of four tenants.
- 10,000 seed rows per tenant, 512-byte values, seed `20261013`, and the
  existing binary key format.
- Preparation Gets counted as ordinary measured service requests and included
  in the observed read fraction and workflow latency.
- Existing Get/Query16 mix, write categories, conditional types, intentional
  condition rejections, transaction widths 1/4/8, and WAL/sync settings.
  Throughput uses the driver's measured admission plus drain duration as its
  denominator.

Each backend used an independent fresh database directory per condition.
Static schedule fingerprints matched for all five conditions. The first 100
workflow-input fingerprints matched for all 16 single-tenant clients and all
four clients on each MT-MIX tenant. Revision values are not part of these
input fingerprints. Different admitted request counts, conflict winners, and
committed histories are reported as measured outcomes. Final state was
checked by replaying successful mutation history using commit identity against
the independent expected state, then by a second Scan after shutdown/reopen.

## Single-tenant results

Each row is one five-second measurement. `R/W` is the measured share of API
calls, including preparation Gets. Latencies are milliseconds. “Workflow”
includes any preparation Get and the associated request sequence.

| Profile | Backend | Actual R/W | API/s | Successful write tx/s | Successful write p50/p95/p99 | Get p95/p99 | Query16 p95/p99 | Workflow p50/p95/p99 | Preparation Gets |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RW95 | B-link | 94.9900/5.0100% | 43,539.30 | 2,003.82 | 6.36/11.94/14.78 | 0.080/0.167 | 0.106/0.202 | 0.008/0.669/8.127 | 1,983 |
| RW95 | RocksDB | 95.0262/4.9738% | 23,615.21 | 1,079.02 | 2.27/7.75/15.96 | 1.630/4.153 | 1.162/2.635 | 0.471/1.568/4.023 | 1,070 |
| RW80 | B-link | 80.1902/19.8098% | 14,035.85 | 2,586.62 | 5.04/9.63/12.94 | 0.077/0.172 | 0.099/0.204 | 0.010/6.884/10.124 | 2,429 |
| RW80 | RocksDB | 80.1886/19.8114% | 11,974.31 | 2,209.41 | 2.53/8.78/17.26 | 1.898/2.592 | 1.979/2.709 | 0.858/3.569/8.391 | 2,102 |
| RW50 | B-link | 49.8890/50.1110% | 6,028.91 | 2,808.34 | 4.80/8.63/11.89 | 0.067/0.133 | 0.094/0.181 | 3.657/7.734/11.337 | 2,647 |
| RW50 | RocksDB | 49.7462/50.2538% | 4,882.27 | 2,306.02 | 3.40/10.58/20.05 | 4.630/6.630 | 4.646/6.829 | 2.967/8.581/16.791 | 2,108 |
| RW20 | B-link | 20.0623/79.9377% | 4,168.69 | 3,089.01 | 4.43/7.69/10.42 | 0.074/0.137 | 0.096/0.153 | 4.352/7.637/10.033 | 2,965 |
| RW20 | RocksDB | 20.1070/79.8930% | 3,549.26 | 2,636.82 | 4.10/10.40/19.20 | 7.839/9.772 | 6.873/9.562 | 4.282/11.003/18.492 | 2,503 |

All four requested read/write ratios were reached closely. Actual category
attempts and successes for Put, Delete, conditional widths 1/4/8, condition
kinds, and preparation Gets are in [`condition-summary.csv`](condition-summary.csv).
Every single-tenant and MT-MIX run reached both width-4 and width-8 writes.

| Profile | Backend | Intentional condition rejections | Natural conflicts | Overloads | Other errors |
| --- | --- | ---: | ---: | ---: | ---: |
| RW95 | B-link | 889 | 0 | 0 | 0 |
| RW95 | RocksDB | 478 | 0 | 0 | 0 |
| RW80 | B-link | 970 | 0 | 0 | 0 |
| RW80 | RocksDB | 815 | 0 | 0 | 0 |
| RW50 | B-link | 1,065 | 0 | 0 | 0 |
| RW50 | RocksDB | 738 | 0 | 0 | 0 |
| RW20 | B-link | 1,218 | 0 | 0 | 0 |
| RW20 | RocksDB | 995 | 0 | 0 | 0 |

## MT-MIX per-tenant results

MT-MIX used all four target profiles concurrently, with four clients per
tenant. Latencies are milliseconds. Both backends passed replay and reopen
validation for every tenant. B-link observed four natural conflicts in the
RW50 tenant and RocksDB observed eight; both had zero overloads and other
errors.

| Backend | Tenant/profile | Actual R/W | API/s | Successful write tx/s | Write p50/p95/p99 | Get p95/p99 | Query16 p95/p99 | Workflow p95/p99 | Preparation Gets | Intended rejects |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| B-link | tenant 1 / RW95 | 94.9937/5.0063% | 14,169.28 | 651.99 | 4.76/9.56/12.74 | 0.019/0.080 | 0.043/0.431 | 0.007/2.006/6.065 | 642 | 287 |
| B-link | tenant 2 / RW80 | 79.9369/20.0631% | 3,866.37 | 720.15 | 4.56/8.89/14.34 | 0.040/0.202 | 0.067/0.485 | 0.011/5.834/8.901 | 680 | 278 |
| B-link | tenant 3 / RW50 | 49.9498/50.0502% | 1,593.00 | 741.13 | 4.52/8.70/13.00 | 0.051/0.267 | 0.121/0.666 | 2.938/7.622/11.181 | 704 | 277 |
| B-link | tenant 4 / RW20 | 20.1473/79.8527% | 1,003.97 | 744.93 | 4.55/8.38/11.68 | 0.070/0.268 | 0.087/0.681 | 4.409/8.244/11.690 | 687 | 284 |
| RocksDB | tenant 1 / RW95 | 94.9956/5.0044% | 3,862.37 | 177.30 | 1.80/3.06/3.94 | 1.897/2.567 | 1.946/2.629 | 0.913/2.082/2.951 | 176 | 80 |
| RocksDB | tenant 2 / RW80 | 79.9339/20.0661% | 3,386.84 | 631.43 | 2.04/3.63/4.60 | 1.825/2.427 | 1.804/2.501 | 0.975/2.866/3.967 | 589 | 241 |
| RocksDB | tenant 3 / RW50 | 49.6543/50.3457% | 2,313.06 | 1,082.17 | 2.47/4.33/6.01 | 1.809/2.539 | 1.743/2.499 | 1.703/4.091/5.594 | 1,018 | 404 |
| RocksDB | tenant 4 / RW20 | 20.0491/79.9509% | 1,709.81 | 1,270.47 | 2.65/4.67/6.69 | 1.781/2.777 | 1.763/2.529 | 2.596/4.826/6.642 | 1,177 | 483 |

The tenant rows show why the aggregate write rate alone would be misleading:
B-link had more successful writes per second for the read-heavy tenants 1 and
2, while RocksDB had more for tenants 3 and 4 in this run. The full per-tenant
counts, latency samples, condition kinds, category widths, and independent
validation fields are retained in the CSV and raw reports.

## Process and disk observations

CPU is reported as percent of one core. RSS is the sampled peak in MiB. Disk
values are regular-file byte totals under that run's isolated data directory,
shown as MiB before shutdown and after reopen validation.

| Condition | Backend | CPU % of one core | Peak sampled RSS MiB | Disk before shutdown MiB | Disk after reopen validation MiB |
| --- | --- | ---: | ---: | ---: | ---: |
| RW95 | B-link | 66.9 | 98.1 | 149.5 | 202.0 |
| RW95 | RocksDB | 105.6 | 35.5 | 13.9 | 27.5 |
| RW80 | B-link | 45.8 | 109.3 | 181.8 | 240.9 |
| RW80 | RocksDB | 84.5 | 39.4 | 22.0 | 43.6 |
| RW50 | B-link | 39.8 | 118.1 | 201.4 | 266.9 |
| RW50 | RocksDB | 53.8 | 38.7 | 23.5 | 46.6 |
| RW20 | B-link | 39.2 | 130.3 | 221.9 | 296.5 |
| RW20 | RocksDB | 45.0 | 41.2 | 26.7 | 52.9 |
| MT-MIX | B-link | 69.2 | 189.0 | 256.8 | 365.9 |
| MT-MIX | RocksDB | 87.1 | 66.1 | 47.0 | 92.7 |

CPU tick deltas cover admission and drain. RSS was sampled every 200 ms; the
peak can miss shorter spikes. Disk totals count regular-file lengths, not
allocated ZFS blocks or device-wide activity. The driver does not expose sync
counts, group-commit details, or compaction observations.

## Reading the comparison

1. **Current service implementation:** B-link led API/s and successful write
   transactions/s in all four single-tenant conditions. Its API/s lead was
   about 1.84x at RW95, 1.17x at RW80, 1.23x at RW50, and 1.17x at RW20. B-link
   Get and Query16 tails were substantially lower in every single-tenant
   condition. RocksDB had lower successful-write p50 in each single-tenant
   row. Its p95 was lower at RW95 and RW80; B-link's p95 was lower at RW50 and
   RW20. B-link's successful-write p99 was lower in all four profiles.
   Workflow tails varied by profile: RocksDB was lower at RW95/RW80, while
   B-link was lower at RW50/RW20.
   MT-MIX results depend on tenant mix: RocksDB had a higher successful-write
   rate for RW50 and RW20 tenants, while B-link led for RW95 and RW80 tenants.
2. **Prior serialized-counter bottleneck:** the separate same-RocksDB
   allocator experiment at SHA `d9c5e01e` measured serialized → reserved
   successful write transactions/s of 885 → 787 at RW95 (-11%), 925 → 2,231
   at RW80 (+141%), 979 → 2,523 at RW50 (+158%), 980 → 2,601 at RW20
   (+166%), and 2,042 → 2,842 in MT-MIX (+39%). It also attributed substantial
   per-write time to counter lock waits in serialized mode. Those historical
   observations used research metrics enabled and are not included in this
   report's measurements; see the [revision-range report](../dodb-service-revision-range-d9c5e01e/REPORT.md).
   The current matrix answers the previously missing B-link versus
   reserved-mode question at one SHA.
3. **Remaining implementation differences:** B-link retains its group
   coordinator and page-cache behavior. RocksDB retains TransactionDB,
   eight blocking-operation slots, and tenant-local range reservations.
   Current results include these service and engine differences and do not
   isolate a single cause.
4. **Long-run behavior:** one five-second repetition does not measure
   variance, steady-state compaction, longer WAL growth, sustained memory
   pressure, or performance under production traffic.

The next investigation should explain the large RocksDB Get/Query16 latency
gap and the read-heavy MT-MIX tenant spread, with path-level read and blocking
dispatch attribution while preserving the same service boundary.

## Execution accounting and artifacts

| Phase | Planned | Executed | Passed | Failed | Not reached |
| --- | ---: | ---: | ---: | ---: | ---: |
| Local focused tests | 18 | 18 | 18 | 0 | 0 |
| Local workload smoke at final SHA | 2 backend scenarios | 2 | 2 | 0 | 0 |
| OCI Release focused tests | 18 | 18 | 18 | 0 | 0 |
| OCI smoke | 2 | 2 | 2 | 0 | 0 |
| OCI performance conditions | 10 | 10 | 10 | 0 | 0 |

The table counts accepted work at implementation SHA `3dd51326`. Earlier
attempts are accounted for separately:

| Excluded attempt | Planned | Executed | Passed | Failed | Reason |
| --- | ---: | ---: | ---: | ---: | --- |
| Local smoke at `82160803`, 64 seed rows | 2 | 2 | 0 | 2 | Existing smoke oracle assumes at least 500 seed rows |
| Local smoke at `82160803`, 10,000 seed rows | 2 | 2 | 2 | 0 | Superseded by final-SHA rerun |
| OCI smoke at `82160803` | 2 | 2 | 2 | 0 | Superseded after failure-test scope review |
| OCI measurement at `82160803` | 10 | 10 | 10 | 0 | Superseded because the rollback test lacked a passing condition |

The first local wrong-path invocation exited before a workload started, so it
is recorded as a command error rather than a smoke execution. There were no
not-reached conditions in the accepted final-SHA plan.

At the prior implementation SHA `82160803`, two local smoke runs with 64 seed
rows failed replay because that reduced size falls outside the existing
smoke validator's seed-delete oracle assumptions; a 10,000-row pair passed.
After strengthening the failure test, both final-SHA local smoke runs passed.
A local shell invocation first used the wrong binary path and exited before
starting; all of this is recorded in [`local-smoke/attempts.md`](local-smoke/attempts.md).

The prior OCI attempt at SHA `82160803` completed and passed focused tests,
smoke, and all ten measurements, but it is excluded from this report because
the rollback test had no non-empty passing condition and therefore did not
fully establish the requested failure ordering. Its build/run evidence is
preserved in [`oci/rejected-82160803/`](oci/rejected-82160803/); its DBs remain
under the separate OCI `82160803` run root. At final SHA `3dd51326`, OCI smoke
and all ten measurements passed with no workload failure or retry.

The earlier read-only diagnostic commands against the excluded reports had a
stray heredoc terminator and queried `latencies` instead of `latency`. They did
not alter artifacts. The full final-SHA configuration, fingerprint, state,
reopen, and hash checks passed.

The implementation SHA was pushed to
`experiment/b-link-batched-engine-monorepo` before OCI work. The report and
evidence are added in a later commit; the implementation SHA above remains
the exact source used by every OCI build and run.

Artifacts:

- [`condition-summary.csv`](condition-summary.csv): every tenant's actual
  ratios, throughput, latency counts and percentiles, preparation Gets,
  conditions, write categories, resources, and validation.
- [`paired-input-validation.json`](paired-input-validation.json): exact
  schedule and per-client input-prefix fingerprint comparisons.
- [`oci/evidence-export.tar.gz`](oci/evidence-export.tar.gz): helper build
  provenance and logs, focused test log, smoke reports, all ten measurement
  reports, commands, run order, exit statuses, and before/after binary hashes.
  Archive SHA-256:
  `43053cc5d0edce9346e8b82ccc5c321fafddf0ecc43e95f107f16f2738b2da82`.
- [`local-smoke/`](local-smoke/): smoke output and excluded-attempt record.
- OCI databases remain on the OCI ZFS path and were not copied into the
  repository.
