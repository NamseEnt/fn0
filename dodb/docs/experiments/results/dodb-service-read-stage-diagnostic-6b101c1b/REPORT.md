# RocksDB Read-Path Stage Diagnostic

## Scope and result

This report decomposes reserved-range RocksDB Get and Query16 requests at the
shared `DodbService::execute` boundary. It uses implementation SHA
`6b101c1b44ec21704b060a99b61c7cac792a9e19`. The final documentation commit
is recorded separately in the delivery summary.

The accepted OCI plan was six independent RocksDB runs: read-stage metrics ON
for RW95, RW80, RW50, RW20, and MT-MIX, plus one RW95 run with metrics OFF.
All used reserved revision ranges of 4,096. No B-link or serialized-counter
run was added.

The main observation is that the direct RocksDB Get call averaged 0.0036 to
0.0071 ms in the single-tenant runs. Query16's snapshot/iterator setup,
iterator-call sum, and remaining query processing together averaged about
0.024 to 0.036 ms. By comparison, measured semaphore wait averaged 0.30 ms
at RW95 and 3.70 to 3.72 ms at RW20. Blocking dispatch and completion handoff
also contributed measurable time. The large read tails therefore occur
outside the direct RocksDB Get/iterator calls in these runs.

This is a one-run, five-second synthetic diagnosis per condition. It does not
re-measure B-link or establish a stable ranking, prove that sync commits cause
each wait, or identify the RocksDB cache, file-system, or device contribution.

## Measurement boundaries

Each read metric uses a fixed-size, per-tenant, per-kind histogram with atomic
counters. It records every request during warmup and measurement; there is no
probabilistic sampling. The histogram has 64 base-2 nanosecond buckets. Means
are exact sum/count values; histogram p95/p99 values are approximate bucket
upper bounds. The workload driver's separate request-latency reservoir is
capped at 20,000 observations per category and reports its sample count.
Thus the new stage histograms have one sample per completed request, while
driver latency categories above 20,000 requests have a bounded reservoir.

| Metric | Start and end | Includes | Does not include |
| --- | --- | --- | --- |
| Request to first permit attempt | `execute_request` entry to immediately before the first semaphore acquire | Lifecycle admission and service work before the first permit wait | Semaphore wait itself |
| Semaphore wait | Sum of permit-acquire waits for the request's blocking submissions | Both the tenant-path existence task and the actual read task for an existing tenant | Blocking dispatch and closure work |
| Blocking dispatch | Sum from each `spawn_blocking` submission to closure start | Tokio blocking-pool queue delay | Semaphore wait and closure work |
| Blocking closure | Sum of synchronous closure durations for the request | Tenant path existence closure and actual Get or Query closure | Async handoff after closure completion |
| Completion handoff | Sum from closure completion timestamp until the async caller resumes | Join completion delivery and async scheduling delay | Closure work |
| Tenant database lookup | Full `database(tenant, false)` call | Blocking path existence check, semaphore/dispatch for that check, map/OnceCell lookup | The later actual Get or Query closure |
| RocksDB Get call | Around the `dodb_rocksdb_get` FFI call | C++ shim and synchronous RocksDB `Get` call | Value copy and logical decode |
| Get copy/decode | After the FFI call through value copy/free and logical state decode | Rust copy, free, and decode/result mapping | RocksDB `Get` |
| Query snapshot/iterator setup | Around `dodb_rocksdb_iterator_new` | Snapshot creation, iterator creation, and initial `Seek` | Iteration and Rust row handling |
| Query iterator-call sum | Sum of timed `current` and `next` FFI wrapper calls, recorded once per Query | Validity/status/key/value access and advances | Snapshot/iterator setup and Rust row handling |
| Query copy/decode/budget | Query closure duration less setup and iterator-call sum, clamped at zero | Key decode/filtering, value copy, budget accounting, document allocation, remaining Rust loop work, and iterator destruction/snapshot release | Setup and timed iterator calls |
| Response processing | Response construction and Get response-budget enforcement after the blocking result | The measured response construction/budget step | Query budget work, which runs inside Query closure |
| Service request total | `execute_request` entry to just before it returns its result | All service stages above and other service request work | A small amount of final metric/outcome bookkeeping and the driver's outer call wrapper |

The tenant-database-lookup interval contains the first blocking submission's
semaphore wait, dispatch, and closure. It overlaps those stage metrics and
must not be added to them. Get's native call/decode and Query's setup,
iterator-call sum, and remaining Query work are inside the blocking closure;
they also must not be added to the closure duration. Stage percentiles are
independent distributions and are never added to construct a request p95 or
p99.

For an already seeded tenant, `database(tenant, false)` submits a blocking
`path.exists()` check, then the service submits the actual Get or Query
closure. The aggregate semaphore, dispatch, closure, and handoff metrics are
one value per API read: they sum the two submissions where both occur.

The driver's `get`, `preparatory_get`, and `query16` latencies start just
before the service call and stop when its future returns, before driver-side
result verification. Preparation Gets have their own category. The service
uses a task-scoped research label for those Gets while preserving the same
`DodbService::execute` implementation path.

## OCI environment and commands

- Host `instance-20260923-1013`, ARM64, Linux 6.12, two vCPUs; ZFS at
  `/bench/zfs/db`.
- Rust/Cargo 1.97.1, `aarch64-unknown-linux-gnu` with the repository `+crc`
  flags and existing Release profile.
- Existing RocksDB 11.8.1 source SHA
  `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; existing archive SHA-256
  `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
- The shared Cargo target and sccache were used through the existing helper.
  No new target or RocksDB archive was created. Transaction PerfContext and
  detailed write-stage metrics were disabled in ON runs.
- Clean detached source checkout:
  `/bench/zfs/db/dodb-read-stage-diagnostic-6b101c1b/checkout`; status was
  clean before and after all runs.
- Workload binary SHA-256, unchanged before and after every run:
  `4e6a143207e9d66343cc7f38a095a4a65ceac8b202dea2694f9b1c22384a931f`.

Focused checks and build:

```sh
DODB_BUILD_RUN_ID=read-stage-6b101c1b-focused-2 \
  bash dodb/scripts/run-oci-rocksdb-build.sh \
  test --locked --release -p dodb-rocksdb --lib --test research_metrics

DODB_BUILD_RUN_ID=read-stage-6b101c1b-workload-build-1 \
  bash dodb/scripts/run-oci-rocksdb-build.sh \
  build --locked --release -p dodb-rocksdb --bin service_workload
```

Each accepted workload command used the following fixed settings, with only
scenario, metrics flags, and run paths varying:

```sh
service_workload --backend rocksdb --scenario <rw95|rw80|rw50|rw20|mt-mix> \
  --seed 20261013 --warmup-ms 1000 --measurement-ms 5000 --clients 16 \
  --rows 10000 --value-size 512 --call-cap 1000000 \
  --revision-allocator reserved-ranges --research-metrics <0|1> \
  --read-stage-metrics <0|1> --data-dir <fresh-run-db> --report <run-report.json>
```

All accepted conditions retained the existing deterministic request mix,
including preparation Gets, Query16, conditional/write widths 1/4/8, WAL,
`sync=true`, deadlines, drain, independent replay validation, and shutdown /
reopen validation. MT-MIX used four clients per tenant. Each run had a unique
empty DB directory. The database directories remain on OCI under
`/bench/zfs/db/dodb-read-stage-diagnostic-6b101c1b/runs/`.

## Single-tenant read stages

Driver p95/p99 below are in milliseconds from the capped request-latency
reservoir. Every other triplet is `mean / p95 / p99` in milliseconds; stage
p95/p99 are approximate histogram bucket bounds. “Closure” is the sum of the
two synchronous closures; it is not engine-only time.

### Get

| Profile | Requests / driver samples | Driver p95 / p99 | Service total | Semaphore wait | Blocking dispatch | Blocking closure | Completion handoff | Pre-permit / response means | RocksDB Get / copy-decode means |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RW95 | 87,453 / 20,000 | 1.110 / 1.846 | 0.576 / 2.097 / 2.097 | 0.297 / 1.049 / 1.049 | 0.114 / 0.524 / 1.049 | 0.014 / 0.033 / 0.131 | 0.146 / 0.524 / 1.049 | 0.00095 / 0.00161 | 0.0036 / 0.0005 |
| RW80 | 36,693 / 20,000 | 1.751 / 2.469 | 0.846 / 2.097 / 4.194 | 0.683 / 2.097 / 4.194 | 0.056 / 0.262 / 0.524 | 0.017 / 0.033 / 0.262 | 0.083 / 0.524 / 1.049 | 0.00143 / 0.00216 | 0.0049 / 0.0005 |
| RW50 | 8,193 / 8,193 | 4.268 / 5.656 | 2.146 / 8.389 / 8.389 | 1.954 / 4.194 / 8.389 | 0.076 / 0.524 / 0.524 | 0.022 / 0.033 / 0.524 | 0.085 / 0.524 / 0.524 | 0.00191 / 0.00279 | 0.0065 / 0.0005 |
| RW20 | 336 / 336 | 6.309 / 7.266 | 3.901 / 8.389 / 8.389 | 3.698 / 8.389 / 8.389 | 0.083 / 0.524 / 1.049 | 0.025 / 0.066 / 0.131 | 0.084 / 0.524 / 1.049 | 0.00174 / 0.00250 | 0.0071 / 0.0003 |

### Query16

| Profile | Requests / driver samples | Driver p95 / p99 | Service total | Semaphore wait | Blocking dispatch | Blocking closure | Completion handoff | Pre-permit / response means | Snapshot/iterator setup / iterator-call sum / copy-decode-budget means |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RW95 | 22,099 / 20,000 | 1.163 / 1.906 | 0.598 / 2.097 / 2.097 | 0.300 / 1.049 / 1.049 | 0.113 / 0.524 / 1.049 | 0.033 / 0.066 / 0.262 | 0.149 / 0.524 / 1.049 | 0.00152 / 0.00006 | 0.0122 / 0.0028 / 0.0085 |
| RW80 | 9,757 / 9,757 | 1.856 / 2.635 | 0.860 / 2.097 / 4.194 | 0.675 / 2.097 / 4.194 | 0.057 / 0.262 / 0.524 | 0.039 / 0.131 / 0.524 | 0.083 / 0.524 / 1.049 | 0.00130 / 0.00009 | 0.0159 / 0.0032 / 0.0098 |
| RW50 | 2,759 / 2,759 | 4.335 / 6.194 | 2.218 / 8.389 / 8.389 | 1.999 / 4.194 / 8.389 | 0.078 / 0.524 / 0.524 | 0.046 / 0.131 / 0.524 | 0.089 / 0.524 / 1.049 | 0.00189 / 0.00007 | 0.0195 / 0.0039 / 0.0094 |
| RW20 | 721 / 721 | 6.644 / 8.417 | 3.944 / 8.389 / 16.777 | 3.721 / 8.389 / 8.389 | 0.084 / 0.524 / 1.049 | 0.049 / 0.131 / 0.524 | 0.084 / 0.262 / 0.524 | 0.00210 / 0.00009 | 0.0217 / 0.0029 / 0.0097 |

Get's direct RocksDB call and copy/decode are a small fraction of the 0.576
to 3.901 ms mean service duration. For Query16, the three measured inner
pieces total about 0.024 to 0.036 ms on average. The closure and service
totals are higher because they also contain the separate path-existence task,
the two blocking submissions, and service work. Do not add the inner Query
pieces to the closure total. The full Query function elapsed time was used to
calculate the per-request copy/decode/budget remainder, so the three means
reconstruct its mean exactly. A separate full-Query percentile histogram was
not recorded; use the driver Query16 p95/p99 for the whole service request,
not sums of the three component percentiles.

Preparation Gets were counted independently from ordinary Gets:

| Profile | Preparation Gets | Driver p95 / p99 ms |
| --- | ---: | ---: |
| RW95 | 1,039 | 1.192 / 2.221 |
| RW80 | 2,134 | 1.911 / 2.570 |
| RW50 | 2,271 | 4.122 / 5.798 |
| RW20 | 2,469 | 6.658 / 8.411 |

## MT-MIX tenant results

Each tenant had four clients and its own 10,000-row database. Driver latency
p95/p99 uses the capped request reservoir. Stage values are means in
milliseconds. The request counts in the first columns match the per-tenant
read-stage histogram counts exactly.

| Tenant / profile | Get count; p95 / p99 | Prep Get count; p95 / p99 | Query16 count; p95 / p99 | Semaphore mean Get / Query16 | Dispatch mean Get / Query16 | Closure mean Get / Query16 | Handoff mean Get / Query16 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| tenant 1 / RW95 | 10,896; 2.714 / 3.932 | 136; 2.881 / 3.775 | 2,783; 2.806 / 3.894 | 0.971 / 0.988 | 0.110 / 0.109 | 0.029 / 0.056 | 0.180 / 0.202 |
| tenant 2 / RW80 | 8,004; 2.671 / 3.970 | 464; 2.387 / 3.465 | 2,108; 2.828 / 4.194 | 0.939 / 0.966 | 0.120 / 0.118 | 0.027 / 0.057 | 0.178 / 0.202 |
| tenant 3 / RW50 | 3,029; 2.677 / 3.715 | 869; 2.877 / 3.981 | 983; 2.799 / 3.818 | 0.923 / 0.914 | 0.123 / 0.115 | 0.032 / 0.062 | 0.185 / 0.215 |
| tenant 4 / RW20 | 152; 2.961 / 3.896 | 1,054; 2.640 / 3.860 | 300; 2.538 / 4.583 | 0.919 / 0.918 | 0.143 / 0.112 | 0.024 / 0.050 | 0.209 / 0.201 |

Semaphore means are similar across tenants in this single MT-MIX run. Get
direct-call means ranged from 0.0082 to 0.0125 ms. Query16 inner means were
0.0198 to 0.0243 ms for snapshot/iterator setup, 0.0039 to 0.0049 ms for the
iterator-call sum, and 0.0108 to 0.0180 ms for remaining query processing.
Tenant 4 has only 152 ordinary Gets and 300 Query16 requests; its tail
percentiles are correspondingly less stable.

## RW95 metrics ON/OFF

Both runs used binary SHA-256
`4e6a143207e9d66343cc7f38a095a4a65ceac8b202dea2694f9b1c22384a931f`.
Each is one independent five-second run, not a repeated paired estimate.

| Mode | API/s | Actual read/write | Get requests / latency samples | Get p95 / p99 ms | Query16 requests / samples | Query16 p95 / p99 ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Metrics ON | 23,269 | 95.048 / 4.952% | 87,453 / 20,000 | 1.110 / 1.846 | 22,099 / 20,000 | 1.163 / 1.906 |
| Metrics OFF | 23,486 | 95.030 / 4.970% | 88,227 / 20,000 | 1.428 / 2.112 | 22,322 / 20,000 | 1.203 / 1.890 |

The ON run completed about 0.9% fewer API calls/s. Get tails were lower in
that run, while Query16 p95 was slightly lower and p99 slightly higher. This
single pair does not establish a stable instrumentation overhead.

## Interpretation and remaining unknowns

1. The largest measured stage is semaphore waiting. Its mean rises from about
   0.30 ms at RW95 to about 3.7 ms at RW20. Blocking-pool dispatch and async
   completion handoff are also visible, typically about 0.08 to 0.15 ms each
   on average for the single-tenant rows. Request-to-first-permit work and
   response processing are much smaller.
2. The direct Get FFI call is about 0.004 to 0.007 ms on average in the
   single-tenant runs. Query16's setup, timed iterator calls, and remaining
   query work together are about 0.024 to 0.036 ms. These values do not
   support attributing the observed millisecond tails to the direct read
   calls alone.
3. The write path acquires the same eight-operation semaphore before
   `spawn_blocking`; the permit is moved into the closure and held until the
   transaction call returns. The RocksDB transaction path uses
   `WriteOptions.sync=true`. This control flow permits a write waiting on
   durable commit work to occupy a shared slot while reads wait for a slot.
   The runs show increasing semaphore wait under write-heavy mixes, but do
   not identify which task held each permit or measure WAL-sync time itself.
4. MT-MIX tenant differences are modest in the measured semaphore means, but
   the one run does not establish that tenant workload composition never
   affects queueing. The rare RW20 tenant reads also have smaller samples.
5. `tenant_database_lookup` includes the path-existence blocking task and
   overlaps stage intervals. The separate `database(tenant, false)` path
   check is measurable as an extra semaphore/dispatch/closure submission,
   but this report does not isolate its stage distributions from the actual
   read request at the per-submission level.
6. The study does not separate permit waiting by read versus write holder,
   quantify per-commit sync duration, expose RocksDB block-cache/SST/device
   activity, or measure CPU scheduling beyond dispatch and completion
   handoff. The existing B-link comparison was not repeated here.

The next single mechanism to investigate is avoiding the repeated blocking
tenant-path existence check for a tenant whose database handle is already
open in the service's existing `OnceCell`. That path adds another blocking
submission and semaphore wait before each actual read. Keep the eight-slot
limit, transaction semantics, and other backends fixed while evaluating only
that mechanism. This proposal is not implemented in this diagnostic.

## Validation and artifacts

| Phase | Planned | Executed | Passed | Failed | Not reached |
| --- | ---: | ---: | ---: | ---: | ---: |
| Local fixed-request tests | 15 | 0 | 0 | 0 | 15 |
| OCI focused tests: `dodb-rocksdb --lib` and `research_metrics` | 15 | 15 | 15 | 0 | 0 |
| OCI diagnostic smoke | 1 | 1 | 1 | 0 | 0 |
| OCI performance conditions | 6 | 6 | 6 | 0 | 0 |

The local Cargo check stopped in `build.rs` because this Mac has no configured
pinned RocksDB 11.8.1 directory. The first OCI focused build at SHA
`07eaf785c449eec9470f20af7c17e9a77781010d` caught a missing boolean argument
on the driver's reopen-validation constructor. That compile attempt executed
no tests. The fix is in implementation SHA `6b101c1b`.

The first smoke and first matrix post-filters each had incorrect JSON paths;
the smoke itself exited successfully and passed replay/reopen checks. A later
matrix filter also incorrectly required the driver's bounded latency sample
count to equal the full request count. RW95 and RW80 exceed the driver's
20,000-observation cap. Corrected final validation confirmed every ON stage
histogram count equals its corresponding full request count and all driver
categories have nonzero samples. No workload condition was retried.
[`attempts.md`](attempts.md) retains this accounting.

The OCI reports, full stdout logs, commands, validation results, per-run
binary hashes, host/source record, and build-helper evidence are in
[`oci/runs/`](oci/runs/), [`oci/host-and-source.txt`](oci/host-and-source.txt),
and [`evidence.tar.gz`](evidence.tar.gz). The run-specific DBs remain on OCI;
they were excluded from the committed evidence archive. Summary tables are
also available in [`condition-summary.csv`](condition-summary.csv) and
[`stage-metrics.csv`](stage-metrics.csv). The evidence archive SHA-256 is
`764d6ade1b95cc172cd4f249d20a15c3aa03d81c6ec101d2f99e9344954fc3ef`.
