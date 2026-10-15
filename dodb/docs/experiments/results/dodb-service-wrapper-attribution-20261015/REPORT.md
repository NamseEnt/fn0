# RocksDB wrapper cost investigation

The largest observed adapter costs are serialized revision allocation and
read admission/scheduling. Full-value reads and copies are unnecessary in
some paths, but their measured cost is much smaller in these cache-friendly
512-byte workloads. The existing DocDb transaction consumer uses revision
checks, including missing-state revisions; those requirements are not merely
unused features invented by this benchmark.

This investigation added opt-in attribution and a separate overwrite probe.
It did not change service semantics, user locks, allocator algorithms,
sync/WAL policy, execution-slot defaults, or the product backend.

## Evidence and scope

- OCI `instance-20260923-1013`, ARM64 Neoverse-N1, two CPUs, Oracle Linux,
  Linux 6.12, ZFS `/bench/zfs/db`.
- Rust/Cargo 1.97.1, Release, repository ARM64 `+crc`, existing shared Cargo
  target/sccache, and the existing RocksDB 11.8.1 archive. RocksDB was not rebuilt.
- Main source: `d56382ef236542846b3bb836570566f835b2a0e3`.
  Workload binary SHA-256:
  `5b16734e865a525a42fd339f8f89f4f10f71977aa046a6d905492ee6fa905c00`.
- Overwrite source: `4418f506b0e93ed5d4dde5590b291d38031b8bda`.
  Probe binary SHA-256:
  `cee52eb77e6fa6e0d9752363ba4b4a4a7dab939927be6a8250e7ded232fc4614`.
  This SHA only adds the probe binary; the service library, shim, and original
  workload driver are identical to the main source.
- RocksDB source: `abeebd9630f11bd08c28b7bd43c7bdfc62050654`.
  Archive SHA-256:
  `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
- 32 final focused cases, six smoke runs, 90 main performance runs, and nine
  overwrite performance runs passed. Every performance run passed independent
  expected-state and shutdown/reopen checks. No performance run failed or was
  retried. Two earlier focused attempts are preserved and explained in
  [validation-attempts.md](validation-attempts.md).

The main matrix used six configurations, five scenarios, and three independent
repetitions per condition, in rotated serial order. All used the existing
DodbService::execute schedule, 16 clients, 10,000 seed rows per tenant, 512-byte
values, seed 20261013, one-second warmup, five-second admission, and drain.
Preparation Gets remained real API calls. MT-MIX had four clients on each of
four tenants. Every run used a fresh DB directory. Schedule and per-client
first-100 input fingerprints matched across all 90 runs in eight tenant/profile
groups. Read/write and key/byte stage counts were checked against full request
counts, independently of the capped driver latency reservoir.

All tables below show medians of three run-level measurements. Bracketed
values are the minimum and maximum of those three runs. Stage values are
medians of run means; request percentiles are medians of run percentiles,
never pooled percentiles or sums of component percentiles.

## 1. Shared revision metadata serializes independent writes

In serialized mode every successful write locks the same tenant metadata key
and holds it through commit. The current default is still Serialized. This
forces independent user-key writes to wait for each other. Range reservation
is already implemented as an option and avoids per-user-transaction metadata
access while preserving protected-key ordering and restart-safe identities.

| Profile | Successful write tx/s, serialized 8 ON | Successful write tx/s, ranges 8 ON |
| --- | ---: | ---: |
| RW95 | 910 [801, 927] | 1,703 [1,634, 1,717] |
| RW80 | 950 [907, 965] | 2,377 [2,374, 2,460] |
| RW50 | 977 [950, 998] | 2,566 [2,536, 2,626] |
| RW20 | 972 [972, 990] | 2,686 [2,383, 2,696] |

For RW50, serialized counter lock wait averaged 4.557 ms per successful write
(run means 4.432–4.653 ms), versus a 1.073 ms user commit call. Range mode
recorded zero user-transaction counter accesses in every run; reservation
transactions remained separate and included their synchronous refill cost.
The observed RW50 write-rate ratio was approximately 2.63. This is a large
service-path difference with a directly confirmed removed path, not a claim
about maximum RocksDB engine throughput or a production effect size.

## 2. Most read time is spent before or after the native call

With reserved ranges and shared eight-slot admission:

| Profile | Get service mean, ms | Get semaphore mean, ms | Native Get call, microseconds |
| --- | ---: | ---: | ---: |
| RW95 | 0.297 | 0.195 | 2.82 |
| RW80 | 0.678 | 0.589 | 4.33 |
| RW50 | 1.541 | 1.441 | 5.40 |
| RW20 | 2.241 | 2.131 | 7.02 |

For RW50 the read semaphore accounts for approximately 94% of the observed
Get service mean. The read submits one blocking closure for an initialized
tenant. The previously removed path-existence submission is not present on
this warm path. Write closures retain the same shared permits while waiting
for user-key locks and commit, so this policy couples short reads to longer
write execution even across tenants.

With sixteen shared slots, RW50 Get semaphore mean fell to 0.000762 ms and
service mean to 0.0893 ms. Native Get remained 5.81 microseconds. Blocking
dispatch and completion handoff remained approximately 38.5 and 35.5
microseconds respectively. The adapter's dispatch/wakeup path is therefore
still material after admission wait becomes small. B-link's service calls
its versioned read handle directly; the two services have different read
execution costs despite sharing the request/response API.

## 3. Increasing slots shifts contention and worsens write tails

RW50, with the same reserved-range algorithm and combined metrics ON:

| Admission | Successful write tx/s | Get p95, ms | Successful write p99, ms |
| --- | ---: | ---: | ---: |
| Shared 8 | 2,566 [2,536, 2,626] | 3.021 [2.917, 3.087] | 18.246 [17.777, 18.923] |
| Shared 16 | 2,761 [2,754, 2,764] | 0.354 [0.354, 0.376] | 58.367 [58.139, 58.941] |
| Split read 2/write 6 | 2,304 [2,270, 2,335] | 0.390 [0.375, 0.413] | 15.067 [14.807, 16.926] |

Moving from eight to sixteen slots raised write rate about 7.6% while write
p99 rose about 3.2 times. Write semaphore mean fell from 1.451 ms to 0.000529
ms. User-key lock wait, measured per GetForUpdate, rose from 0.249 ms to
0.954 ms. User commit means stayed close, 1.933 versus 1.966 ms. Thus the
observed improvement in admission does not imply that write-side waiting has
disappeared. The mean user-lock increase and tail increase are observations;
they do not identify every long-tail request or attribute its delay to WAL sync.

The sixteen-slot run also has only sixteen client workflows in flight. Its
admission ceiling is at that workload's client count. The result does not
establish sixteen as an optimum at higher concurrency. Split admission
protects reads but reduces write capacity, explaining the need to report both
latency and throughput instead of declaring the split uniformly better.

## 4. Full-value copies are redundant, but small in the measured overwrite case

The main workload generates mostly fresh mutation keys, with some deletes of
seeded keys. Many mutation-only GetForUpdate calls consequently return no
value or a small tombstone. Its tiny average copy time does not characterize
overwriting an existing 512-byte value. The historical raw workload chose
uniform writes from a pre-seeded working set, another important difference
between those historical throughput figures.

The separate probe closes this coverage gap. All sixteen clients repeatedly
overwrite their own existing key; every measured read must return 522 encoded
bytes containing 512 payload bytes. It uses shared eight slots and reserved
ranges, the real service API, sync/WAL, and no contested user keys. The probe
has three Put ON runs, three Exists-plus-Put Transact ON runs, and three Put
OFF runs. Complete values, per-key revisions, the full sixteen-row state,
prefix fingerprints, and reopen state all passed.

| Probe | Native GetForUpdate mean, microseconds | Shim copy + Rust copy + decode, microseconds | Service request mean, microseconds | User commit mean, microseconds |
| --- | ---: | ---: | ---: | ---: |
| Existing-value Put ON | 9.36 [7.74, 9.36] | 0.567 [0.511, 0.593] | 4,326 [4,309, 4,670] | 2,061 [2,030, 2,185] |
| Existing-value Exists + Put ON | 9.38 [8.67, 10.46] | 0.595 [0.538, 0.789] | 4,254 [4,236, 4,260] | 2,002 [1,994, 2,007] |

User-key lock wait was zero in these disjoint-key probes. For Put ON, copying
and decoding alone occupied about 0.013% of the request mean. Even the whole
native read plus these copies was about 0.23% of the observed request mean.
These are wall-time shares in this probe, not predictions of throughput gains
from an unimplemented optimization.

Mutation-only keys do not need their old logical value for the current
operation. RocksDB 11.8.1 supports a lock-only GetForUpdate with null value
output. Condition keys need existence/revision, not payload materialization.
Both are valid optimization candidates, but the measured magnitudes do not
explain the multi-millisecond delays. No lock-only execution variant was
implemented or measured. Larger values, cold reads, and production contention
could change the priority; corruption/error behavior also needs deliberate
treatment before removing old-state reads.

## 5. Required behavior and service comparison

DocDb get_observed preserves missing-state revisions. Its transaction state
stores those revisions and later emits RevisionEquals. Dropping tombstones,
condition protection, or atomic multi-key behavior would affect an existing
consumer. By contrast, a shared counter lock on every commit, one specific
admission topology, and unnecessary payload materialization are implementation
choices. See [SOURCE-AUDIT.md](SOURCE-AUDIT.md) for the source paths and the
contract/implementation distinction.

Both raw RocksDB and the service shim already use WAL and sync=true. The raw
harness writes atomic WriteBatch operations, while the service workload adds
conditional transactions, preparation Gets, widths 4/8, and different key
patterns. Their historical rate ratio is not a measured wrapper tax.
Transaction commit call count must not be described as fsync count: neither
this driver nor this probe isolates RocksDB's internal write grouping or
WAL-sync-only duration.

For context, the two uninstrumented services in the main matrix were:

| Profile | Ranges/shared 8 API/s | B-link API/s | Ranges/shared 8 write tx/s | B-link write tx/s |
| --- | ---: | ---: | ---: | ---: |
| RW95 | 38,618 | 45,075 | 1,769 | 2,069 |
| RW80 | 12,842 | 15,095 | 2,383 | 2,789 |
| RW50 | 5,462 | 6,643 | 2,580 | 3,096 |
| RW20 | 3,642 | 4,194 | 2,705 | 3,108 |

MT-MIX must be read per tenant because the achieved mixes differ:

| Tenant/profile | Ranges/shared 8 OFF API/s | B-link OFF API/s | Ranges/shared 8 OFF write tx/s | B-link OFF write tx/s |
| --- | ---: | ---: | ---: | ---: |
| 1/RW95 | 4,787 | 14,806 | 220 | 680 |
| 2/RW80 | 3,671 | 4,103 | 679 | 762 |
| 3/RW50 | 2,234 | 1,685 | 1,047 | 779 |
| 4/RW20 | 1,586 | 1,051 | 1,179 | 781 |

These are current service implementations with their existing, different
cache/grouping behavior. B-link had lower read tails; write-heavy MT-MIX
tenants had higher write rates on RocksDB shared eight. The evidence supports
investigating the adapter's allocation/admission/execution choices, not a
universal storage-engine ranking.

## Instrumentation control and limitations

Combined metrics are not free. The paired API-rate changes for ranges/shared
eight ON versus OFF were:

| Profile | Median paired change | Minimum to maximum across three pairs |
| --- | ---: | ---: |
| RW95 | -3.12% | -7.96% to -2.16% |
| RW80 | -0.13% | -1.24% to +9.51% |
| RW50 | +1.96% | -1.78% to +5.16% |
| RW20 | -0.63% | -14.19% to +1.39% |

Positive changes do not mean instrumentation improves performance. There is
substantial short-run variation, and three repetitions do not estimate a
stable overhead distribution. Put-overwrite throughput medians were 3,695/s
ON and 3,881/s OFF. The ON/OFF controls do not calibrate every other allocator
or admission configuration. B-link versus instrumented configurations is
context only; the table above explicitly uses both services OFF.

The runs are short, synthetic, and cache-friendly. They do not establish
long-run compaction behavior, larger-than-memory performance, a production
default, power-loss coverage, or maximum engine throughput. Read driver
reservoirs are capped at 20,000; stage counts cover every request. RW20 read
tails have fewer samples. CPU and sampled RSS for the main matrix are retained
per run; overwrite GNU time files cover the whole process, including setup
and validation, and are not measurement-only CPU/RSS estimates.

## Recommended next change boundary

Keep the current correctness contract. Treat revision reservation and read
admission/dispatch as the primary mechanisms to evaluate for adoption or
further improvement. Measure user-key contention and write tails whenever
admission changes. The lock-only/more selective materialization candidate has
clear redundant work but lower measured priority for these hot 512-byte
values. This investigation changes no default.

## Reproduction and retained artifacts

- [PLAN.md](PLAN.md), [run-focused.py](run-focused.py), and
  [run-overwrite.py](run-overwrite.py) describe the executed phases.
- Main runner: `dodb/scripts/run-rocksdb-wrapper-attribution.py` at the main
  measurement source. The six profiles and rotated 90-run order are explicit.
- [condition-summary.csv](condition-summary.csv), [stage-metrics.csv](stage-metrics.csv),
  [write-read-metrics.csv](write-read-metrics.csv), [outcomes.csv](outcomes.csv),
  [summary.json](summary.json), and [paired-input-validation.json](paired-input-validation.json).
- [overwrite-summary.csv](overwrite-summary.csv) and
  [overwrite-summary.json](overwrite-summary.json).
- Recompute summaries with `PYTHONDONTWRITEBYTECODE=1 python3 summarize.py`
  and `python3 summarize-overwrite.py` from this artifact directory.
- `oci/measurements/` retains all main commands, logs, reports, statuses,
  before/after binary hashes, plan/completion records, and host/source evidence.
- `oci/overwrite/` retains the nine probe commands, reports, statuses,
  GNU time files, binary/source manifest, and completion record.
- `oci/builds/` retains all four build-only helper runs, including initial
  focused compilation. `oci/focused*/` retains accepted and rejected checks.
- Original binaries and all independent DBs remain on OCI under
  `/bench/zfs/db/dodb-wrapper-attribution-20261015/`. DBs, temporary directories,
  source checkouts, and binaries are excluded from this repository artifact.
- `SHA256SUMS` covers the final retained files using relative paths.
