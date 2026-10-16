# RocksDB read/write admission comparison

Keep Shared 8 and Serialized as the service defaults. Neither candidate passes
the predeclared screen across all eight tenant/profile groups. Shared 10
passes all four MT-MIX tenant groups but fails the four single-tenant groups
on successful-write p99. Split read-two/write-eight passes single-tenant RW20
only. It sharply reduces single-tenant read tails, while increasing write
tails and reducing write-heavy tenant throughput in MT-MIX.

The implementation adds an opt-in `BlockingAdmissionMode::SplitRead2Write8`
and `--blocking-admission split-read-2-write-8`. It reserves two read slots and
eight write slots, requires ten total operations, and drains those configured
limits on shutdown. The existing split-two/six option and shared admission
retain their limits and defaults. This experiment uses the existing
ReservedRanges allocator in every condition; it does not promote that allocator.

## Evidence and method

- Source: `d0f93e1e8b6ce54221d413897724a36ddd6725da`.
- Immutable workload binary SHA-256:
  `135db6b423a98b4a542318fbb7be26a9e3f6745488649edf2cc4d52648fe3fd7`.
- Host: OCI `instance-20260923-1013`, ARM64 Neoverse-N1, two CPUs,
  Oracle Linux, ZFS `/bench/zfs/db`.
- Rust/Cargo 1.97.1 release, repository ARM64 `+crc`, existing RocksDB 11.8.1
  source `abeebd9630f11bd08c28b7bd43c7bdfc62050654`, archive SHA-256
  `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
- Six accepted smoke runs, 45 primary runs with metrics OFF, and 15 diagnostic
  runs with metrics ON all passed. No accepted run failed or was retried.
  Every run passed expected-state replay and shutdown/reopen validation.
- Three independent primary repetitions per condition, rotated serial order,
  fresh DBs, seed 20261013, 16 clients, 10,000 seed rows per tenant, 512-byte
  values, one-second warmup, five-second admission plus drain. MT-MIX uses
  four clients per tenant. No build or other benchmark overlapped the matrix.
- All 60 measured reports matched schedules and per-client first-100 input
  fingerprints in eight tenant/profile groups. All write categories and
  Exists/NotExists/RevisionEquals conditions were reached. Metric counts,
  outcome counts, byte accounting, process sampling, completion, and executable
  hashes passed the recorded checks. All driver latency counts equal sample
  counts, including merged aggregate counters.

Request construction, conditional and multi-key transaction semantics,
protected user-key locks, revision algorithms, WAL, and sync are unchanged.
The API is still `DodbService::execute`; this is a wrapper configuration
comparison, not a direct or maximum-native-RocksDB baseline.

Primary tables show medians of three run-level measurements. Ranges are
available in [median-ranges.csv](median-ranges.csv) and [summary.json](summary.json).
Percentiles are medians of run percentiles, not pooled across repetitions.
The [predeclared screen](PLAN.md) requires Get and Query16 p95/p99 all to improve,
write throughput to remain at least 95%, and write p99 at most 110% of Shared 8
in each group. It is a descriptive research screen, not a statistical claim.

## 1. Single-tenant primary results, metrics OFF

All latencies are milliseconds; writes/s counts successful write transactions.

| Profile | Admission | Writes/s | Get p95 | Get p99 | Query16 p95 | Query16 p99 | Write p99 | Screen |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| RW95 | Shared 8 | 1,549 | 0.851 | 1.427 | 0.891 | 1.576 | 11.792 | reference |
| RW95 | Shared 10 | 1,848 | 0.634 | 1.104 | 0.663 | 1.151 | 15.106 | fail |
| RW95 | Split 2/8 | 1,700 | 0.616 | 0.980 | 0.639 | 1.020 | 14.106 | fail |
| RW80 | Shared 8 | 2,384 | 1.713 | 2.342 | 1.754 | 2.404 | 17.389 | reference |
| RW80 | Shared 10 | 2,503 | 1.363 | 1.998 | 1.373 | 2.040 | 27.833 | fail |
| RW80 | Split 2/8 | 2,366 | 0.385 | 0.558 | 0.405 | 0.564 | 21.994 | fail |
| RW50 | Shared 8 | 2,583 | 3.061 | 4.304 | 3.052 | 4.212 | 18.832 | reference |
| RW50 | Shared 10 | 2,660 | 2.390 | 3.357 | 2.433 | 3.356 | 27.453 | fail |
| RW50 | Split 2/8 | 2,581 | 0.348 | 0.507 | 0.388 | 0.554 | 20.914 | fail |
| RW20 | Shared 8 | 2,540 | 3.878 | 4.852 | 4.245 | 5.850 | 20.454 | reference |
| RW20 | Shared 10 | 2,837 | 3.048 | 3.873 | 3.032 | 4.081 | 26.620 | fail |
| RW20 | Split 2/8 | 2,699 | 0.366 | 0.519 | 0.410 | 0.534 | 20.356 | pass |

RW50 Split 2/8 reduces Get p95 from 3.061 to 0.348 ms (-88.6%) and Query16
p95 from 3.052 to 0.388 ms (-87.3%). Successful writes remain essentially
unchanged, 2,583 to 2,581/s. Write p99 increases from 18.832 to 20.914 ms
(+11.1%), narrowly failing the predeclared 10% screen. The write-p99 ranges
are [18.546, 18.970] and [20.912, 21.721] ms. The threshold is not evidence
of a statistically meaningful boundary between 10% and 11.1%.

RW20 Split 2/8 passes: writes improve 6.3%, Get p95 falls 90.6%, Query16 p95
falls 90.3%, and write p99 changes -0.5%. RW80 and RW95 Split 2/8 fail on
write p99 (+26.5% and +19.6%). Shared 10 increases successful-write p99 by
28.1%-60.1% across the four single-tenant profiles, despite improving reads.

## 2. MT-MIX primary results, metrics OFF

The read/write budgets are global across all four tenants. They are not
per-tenant reservations. T1/T2/T3/T4 use RW95/RW80/RW50/RW20 respectively.

| Tenant/profile | Admission | Writes/s | Get p95 | Get p99 | Query16 p95 | Query16 p99 | Write p99 | Screen |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| T1/RW95 | Shared 8 | 213 | 1.596 | 2.143 | 1.675 | 2.242 | 5.514 | reference |
| T1/RW95 | Shared 10 | 299 | 1.173 | 1.586 | 1.221 | 1.653 | 5.001 | pass |
| T1/RW95 | Split 2/8 | 413 | 1.123 | 2.032 | 1.166 | 2.041 | 9.325 | fail |
| T2/RW80 | Shared 8 | 665 | 1.522 | 2.090 | 1.571 | 2.215 | 5.854 | reference |
| T2/RW80 | Shared 10 | 800 | 1.125 | 1.557 | 1.139 | 1.578 | 6.121 | pass |
| T2/RW80 | Split 2/8 | 719 | 1.401 | 2.323 | 1.466 | 2.416 | 9.157 | fail |
| T3/RW50 | Shared 8 | 1,022 | 1.485 | 2.082 | 1.535 | 2.069 | 6.819 | reference |
| T3/RW50 | Shared 10 | 1,088 | 1.047 | 1.543 | 1.141 | 1.569 | 7.101 | pass |
| T3/RW50 | Split 2/8 | 887 | 1.728 | 2.693 | 1.755 | 2.534 | 9.951 | fail |
| T4/RW20 | Shared 8 | 1,153 | 1.602 | 2.500 | 1.526 | 2.128 | 7.568 | reference |
| T4/RW20 | Shared 10 | 1,162 | 1.131 | 1.616 | 1.140 | 1.520 | 7.875 | pass |
| T4/RW20 | Split 2/8 | 943 | 1.660 | 2.861 | 1.760 | 2.368 | 10.167 | fail |

Shared 10 passes all four MT groups on the median screen. Get p99 falls
25.5%-35.4%; write throughput changes +0.8% to +40.4%, and write p99 changes
-9.3% to +4.6%. Individual repetitions have wider ranges; the median screen
does not establish nonregression on every run or sustained behavior.

Split 2/8 fails all four MT groups. The RW50 and RW20 tenants lose 13.1%
and 18.2% of successful-write throughput, and their Get p99 rises 29.3%
and 14.5%. All four tenants' write p99 increases by 34.3%-69.1%. The RW95
tenant's write-p99 range is [8.511, 42.617] ms, compared with Shared 8's
[4.608, 5.532] ms. A higher read-heavy tenant rate does not establish overall
tenant fairness. These results do not support general fixed partitioning.

## 3. Diagnostic attribution, metrics ON

These are single-repetition run means, in milliseconds. They explain measured
queue locations and do not rank production performance or identify individual
permit holders. ON and OFF tail distributions differ materially in MT-MIX;
do not infer an OFF causal breakdown or instrumentation cost by subtracting
the two matrices. Mean service time and driver p95/p99 are different statistics.

### RW50

| Admission | Get service | Get permit wait | Native Get | Write permit wait | User-key lock wait | User commit |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Shared 8 | 1.512595 | 1.423988 | 0.005861 | 1.429487 | 0.248841 | 1.915967 |
| Shared 10 | 1.113955 | 1.015598 | 0.006023 | 1.011219 | 0.390367 | 2.006326 |
| Split 2/8 | 0.112141 | 0.006055 | 0.005309 | 2.790078 | 0.259134 | 1.989122 |

Native Get remains approximately 5-6 microseconds. Split 2/8 removes most
Get permit wait (1.424 to 0.006 ms), while write permit wait grows from 1.429
to 2.790 ms. Its user-key lock mean remains close to Shared 8 (0.249 versus
0.259 ms) and user commit changes modestly (1.916 versus 1.989 ms). This
supports read-admission separation as the source of this diagnostic read
improvement, with additional waiting concentrated in the write lane.

Shared 10 reduces permit wait, but user-key lock wait rises from 0.249 to
0.390 ms. This is consistent with extra protected-key contention at higher
concurrency. The timings do not isolate a WAL, filesystem, sync, or specific
lock-holder cause for each write-tail event. Stage percentiles are not additive.

## 4. Latency reporter correction and excluded attempt

The existing reporter used `count * 2654435761 % count` after 20,000 samples.
Before integer overflow this is zero, so it retained the first 19,999 samples
and repeatedly replaced sample zero. Aggregate merging also truncated samples.
Where count exceeded sample_count, its percentiles could favor the beginning
of the window. Prior capped percentiles must not be interpreted as full-window
percentiles. Throughput counters and separately recorded research stage
histograms do not depend on this sampling formula.

The corrected workload reporter stores every completion latency and merges
every tenant sample. The one-million call cap bounds this experiment's volume;
latency memory now scales with completed calls. All accepted reports require
count == sample_count in every category and aggregate. Primary peak sampled
RSS spans 30.0-68.8 MiB, with complete process sampling.

The initial capped attempt is preserved: six smoke reports, 26 main reports,
and 25 checked main runs. The runner was stopped with SIGTERM (observed exit
143); the in-progress child completed its report before the next build, but
has no completed parent provenance/check record. There is no complete marker.
41 tenant/category records exceed the old sample cap. None is used in the
accepted summaries. See [rejected-attempt-summary.json](rejected-attempt-summary.json),
the raw stop reason, and [REPRODUCTION.md](REPRODUCTION.md). Three build logs
are retained: initial inactive-lane correction, capped reporter, and accepted
full-latency reporter. No earlier attempt was overwritten.

## 5. Decision and limits

- Keep the service defaults and the new partition as an experiment option.
- Shared 10 is a follow-up candidate for this MT-MIX workload; Split 2/8 is a
  follow-up candidate for single-tenant RW20. Neither is a universal selection.
- The next admission investigation should measure write-permit occupancy,
  long holders, and tenant fairness under read reservation while preserving
  transaction conditions, user-key locking, WAL, and sync. Current evidence
  supports prioritizing queue/concurrency policy before full-value-copy work.

All results are preliminary five-second measurements on small, cache-friendly
fresh databases. RW95 throughput varies substantially: Shared 8 spans
910-1,750 successful writes/s, Shared 10 1,039-1,904, and Split 2/8 862-1,769.
The cause of that run variation was not isolated. MT RW20 ordinary Get has only
84-163 samples per run; its p99 represents very few observations. A median of
three repetitions is not a confidence interval or a sustained/production claim.
The ZFS dataset reaches 95% utilization, with 8.7 GiB available at completion;
the captured host snapshots retain capacity context. This experiment does not
establish behavior on a less-full dataset or isolate capacity effects.

Release compilation, formatting checks, six smoke workloads, 60 accepted
measured workloads, full-count consistency, expected-state replay, and reopen
validation completed. No new unit/integration suite was added or run in this
turn, and no process-crash, power-loss, or sustained endurance result is claimed.

See [comparison-ratios.csv](comparison-ratios.csv),
[condition-summary.csv](condition-summary.csv), [median-ranges.csv](median-ranges.csv),
[stage-metrics.csv](stage-metrics.csv), [outcomes.csv](outcomes.csv),
[paired-input-validation.json](paired-input-validation.json),
[summary.json](summary.json), and the raw `oci/` evidence. `SHA256SUMS` covers
all local artifacts, including the preserved excluded attempt.
