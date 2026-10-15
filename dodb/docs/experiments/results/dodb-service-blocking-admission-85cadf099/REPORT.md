# RocksDB Read/Write Blocking Admission Split

## Scope and decision

This experiment compares the existing shared eight-slot blocking semaphore with an opt-in split configuration of two read slots and six write slots. Both configurations cap admitted blocking work at eight. Each of the five scenarios was measured once per mode on the same implementation SHA. Results are a short synthetic workload comparison, not a repeatability or production claim.

Recommendation: keep shared eight-slot admission as the default. Preserve split 2/6 as a research option and consider more paired repetitions before adoption. Split substantially lowered the observed Get and Query16 tails in most cases, while successful write throughput fell in RW80, RW50, RW20, and three of four MT-MIX tenants. RW95 showed a smaller read-tail gain and nearly unchanged throughput. The single short runs do not establish stable effect sizes or an optimal split.

## Implementation

`RocksDbServiceConfig::blocking_admission` selects `Shared` or `SplitReadWrite`; the default remains `Shared`. The workload driver accepts `--blocking-admission shared|split-read-write` and records the selected mode and slot counts in its JSON report. Split mode requires the unchanged total limit of eight and fixes its lanes at read 2/write 6.

Get, Query, and Scan use the read lane. Put, Delete, and every Transact use the write lane, including condition-only and mutation-free transactions. Driver preparation Gets call the same Get API and therefore use the read lane.

A missing-tenant path check and cold tenant open inherit the request's lane. The path check and open are sequential submissions; the open does not nest another permit inside an already-held request permit. User transaction execution, revision-range reservation metadata, and sync commit remain in one write blocking closure and use one write permit. Shutdown closes lifecycle admission, waits for request leases, drains all permits from the active configuration, then performs its database-handle cleanup while those permits remain held. The selected permit is moved into each normal blocking closure and stays there until the synchronous operation returns. Cancellation of the async caller does not release a running closure's permit. No Tokio runtime or blocking-pool limits changed.

## Focused checks

The OCI Release run passed 19 library tests, two shared read-contract tests, and one shared transaction-contract test. The deterministic lane test held six write closures at a handoff, then started two reads before releasing any writer. Its observed maxima were read 2, write 6, total 8. The default-mode test confirmed the existing shared semaphore starts with eight permits. Request mapping checks Get/Query/Scan as read and Put/Delete/Transact as write. Split-mode cancellation and shutdown/reopen checks passed; the existing shared-mode in-flight shutdown/reopen check also passed. Existing read and transaction contracts passed. Crash tests were not run.

Local `cargo check --tests` and formatting checks passed. Local test execution was unavailable because the local RocksDB 11.8.1 archive was an empty placeholder; the executable focused checks ran on OCI against its pinned existing archive.

## Smoke and matrix completion

Both shared and split smoke runs used 10,000 rows per tenant and passed state replay plus shutdown/reopen validation for all four tenants. Reports emitted the expected shared 8/8 and split 2/6 settings. Smoke is separate from performance run counts.

The first shared smoke process exited successfully and produced a valid report, but a post-run checker had a quoting error while formatting the mode. That checker-only failure is retained in `oci/smoke/shared/checker-failure.txt`; the report was rechecked without rerunning the workload. The split smoke then completed normally. No performance run failed or required a rerun.

The ten performance runs were serial and used independent initial DB directories. Mode order alternated within each paired condition: shared/split RW95, split/shared RW80, shared/split RW50, split/shared RW20, and shared/split MT-MIX. First-100 per-client workflow fingerprints matched in all five pairs. All ten state validations and all ten reopen validations passed. `oci/matrix-summary.txt` records planned, passed, failed, rerun, and not-reached counts.

## Performance results

All rates use the driver's existing measurement-plus-drain denominator. The actual read/write ratio includes preparatory Gets. Successful write transaction rate excludes rejected conditions. Error columns are condition rejection / natural conflict / overload / other error counts. MT-MIX values are per tenant; tenants use profiles RW95, RW80, RW50, and RW20 with four clients each.

### Single-tenant throughput and outcomes


| Mode | Profile | Actual read/write | API/s | Successful write tx/s | Preparatory Gets | Reject / conflict / overload / other |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| shared | rw95 | 95.01/4.99% | 38641 | 1774 | 1739 | 774 / 0 / 0 / 0 |
| split | rw95 | 95.01/4.99% | 38925 | 1786 | 1754 | 780 / 0 / 0 / 0 |

| Mode | Profile | Get p50/p95/p99 ms | Query16 p50/p95/p99 ms | Successful write p50/p95/p99 ms | Workflow p95/p99 ms |
| --- | --- | ---: | ---: | ---: | ---: |
| shared | rw95 | 0.231/0.686/1.241 | 0.244/0.628/1.141 | 2.356/6.239/11.242 | 1.296/4.073 |
| split | rw95 | 0.181/0.479/0.736 | 0.230/0.550/0.805 | 2.888/7.254/10.301 | 1.024/6.336 |

| Mode | Profile | Actual read/write | API/s | Successful write tx/s | Preparatory Gets | Reject / conflict / overload / other |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| shared | rw80 | 80.09/19.91% | 12689 | 2350 | 2232 | 879 / 0 / 0 / 0 |
| split | rw80 | 80.20/19.80% | 11844 | 2184 | 2071 | 804 / 0 / 0 / 0 |

| Mode | Profile | Get p50/p95/p99 ms | Query16 p50/p95/p99 ms | Successful write p50/p95/p99 ms | Workflow p95/p99 ms |
| --- | --- | ---: | ---: | ---: | ---: |
| shared | rw80 | 0.541/1.763/2.542 | 0.568/1.774/2.481 | 2.865/9.613/17.773 | 3.795/8.779 |
| split | rw80 | 0.038/0.362/0.499 | 0.055/0.398/0.563 | 6.018/10.880/15.835 | 7.501/11.529 |

| Mode | Profile | Actual read/write | API/s | Successful write tx/s | Preparatory Gets | Reject / conflict / overload / other |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| shared | rw50 | 49.63/50.37% | 5467 | 2581 | 2340 | 866 / 0 / 0 / 0 |
| split | rw50 | 49.73/50.27% | 4815 | 2275 | 2075 | 729 / 0 / 0 / 0 |

| Mode | Profile | Get p50/p95/p99 ms | Query16 p50/p95/p99 ms | Successful write p50/p95/p99 ms | Workflow p95/p99 ms |
| --- | --- | ---: | ---: | ---: | ---: |
| shared | rw50 | 1.369/3.085/4.355 | 1.361/3.083/4.529 | 3.665/10.043/18.122 | 7.666/15.214 |
| split | rw50 | 0.052/0.383/0.517 | 0.072/0.404/0.529 | 6.020/11.123/17.269 | 9.378/14.703 |

| Mode | Profile | Actual read/write | API/s | Successful write tx/s | Preparatory Gets | Reject / conflict / overload / other |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| shared | rw20 | 20.12/79.88% | 3544 | 2632 | 2504 | 994 / 0 / 0 / 0 |
| split | rw20 | 20.01/79.99% | 3178 | 2363 | 2227 | 895 / 0 / 0 / 0 |

| Mode | Profile | Get p50/p95/p99 ms | Query16 p50/p95/p99 ms | Successful write p50/p95/p99 ms | Workflow p95/p99 ms |
| --- | --- | ---: | ---: | ---: | ---: |
| shared | rw20 | 2.082/4.049/5.248 | 2.115/3.964/6.120 | 4.349/11.030/21.303 | 11.013/20.439 |
| split | rw20 | 0.064/0.387/0.492 | 0.087/0.447/0.573 | 5.901/10.406/15.985 | 10.026/15.475 |

### MT-MIX per-tenant throughput and outcomes

| Mode | Profile | Actual read/write | API/s | Successful write tx/s | Preparatory Gets | Reject / conflict / overload / other |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| shared | rw95 | 95.03/4.97% | 5040 | 231 | 226 | 100 / 0 / 0 / 0 |
| shared | rw80 | 79.97/20.03% | 3839 | 714 | 678 | 276 / 0 / 0 / 0 |
| shared | rw50 | 49.66/50.34% | 2292 | 1072 | 1015 | 403 / 8 / 0 / 0 |
| shared | rw20 | 20.06/79.94% | 1631 | 1212 | 1142 | 458 / 0 / 0 / 0 |
| split | rw95 | 95.00/5.00% | 10020 | 460 | 453 | 203 / 0 / 0 / 0 |
| split | rw80 | 80.10/19.90% | 3671 | 679 | 635 | 260 / 0 / 0 / 0 |
| split | rw50 | 50.07/49.93% | 1651 | 765 | 742 | 290 / 9 / 0 / 0 |
| split | rw20 | 20.15/79.85% | 1076 | 799 | 740 | 301 / 0 / 0 / 0 |

### MT-MIX per-tenant latency

| Mode | Profile | Get p50/p95/p99 ms | Query16 p50/p95/p99 ms | Successful write p50/p95/p99 ms | Workflow p95/p99 ms |
| --- | --- | ---: | ---: | ---: | ---: |
| shared | rw95 | 0.614/1.456/2.170 | 0.645/1.516/2.334 | 2.081/3.841/5.476 | 1.784/2.941 |
| shared | rw80 | 0.578/1.399/2.119 | 0.600/1.457/2.165 | 2.423/4.267/6.033 | 3.173/4.504 |
| shared | rw50 | 0.591/1.426/2.034 | 0.621/1.546/2.285 | 2.748/4.755/6.575 | 4.373/6.112 |
| shared | rw20 | 0.586/1.422/2.348 | 0.612/1.362/1.735 | 2.826/5.061/7.297 | 5.116/7.283 |
| split | rw95 | 0.085/0.750/1.348 | 0.099/0.719/1.254 | 4.344/6.297/9.316 | 2.436/5.161 |
| split | rw80 | 0.116/0.804/1.400 | 0.145/0.861/1.385 | 4.459/6.335/9.086 | 5.134/6.375 |
| split | rw50 | 0.178/0.889/1.561 | 0.216/1.106/1.765 | 4.524/6.656/8.893 | 6.017/7.738 |
| split | rw20 | 0.205/0.697/1.285 | 0.241/1.057/1.543 | 4.486/6.626/9.496 | 6.557/9.168 |

### CPU, RSS, and measurement limits

CPU is process CPU divided by elapsed measurement-plus-drain time, shown as percent of one core and of the two-core host. RSS is the maximum 200 ms `/proc/self/status` sample. All runs had complete 24-sample RSS collection.

| Scenario | Mode | CPU % one core / host | Peak sampled RSS MiB | RSS samples |
| --- | --- | ---: | ---: | ---: |
| rw95 | shared | 106.4/53.2% | 38.2 | 24 |
| rw95 | split | 106.1/53.1% | 37.3 | 24 |
| rw80 | shared | 70.2/35.1% | 39.4 | 24 |
| rw80 | split | 64.5/32.3% | 37.2 | 24 |
| rw50 | shared | 49.2/24.6% | 40.6 | 24 |
| rw50 | split | 43.9/22.0% | 38.0 | 24 |
| rw20 | shared | 43.0/21.5% | 41.0 | 24 |
| rw20 | split | 39.6/19.8% | 38.4 | 24 |
| mt-mix | shared | 80.9/40.5% | 66.0 | 24 |
| mt-mix | split | 86.5/43.3% | 62.1 | 24 |

The latency reservoir retains at most 20,000 observations per category. The CSV records each category's sample count alongside its full request count and percentiles. At RW20 and some MT-MIX tenant/categories, Get or Query16 samples are small, so their tail estimates are especially noisy. Detailed PerfContext and stage metrics were off. Semaphore wait time was not directly measured; the observed latency changes cannot quantify time saved waiting for read permits. The driver does not expose sync-call or group metrics.

## Interpretation

Split mode lowered Get and Query16 p95/p99 in RW95, RW80, and RW50, with a small API/write-rate increase only in RW95. For RW80 and RW50, successful write transaction throughput was about 7% and 12% lower while Get/Query16 p95 fell about 78% and 87%. RW20 API and successful-write rates were about 10% lower; sampled Get tails fell sharply, while Query16 p95 rose from 3.964 ms to 4.447 ms and the read sample counts were low.

In MT-MIX, read p95 fell for all four tenants. Successful write transaction rate rose for tenant 1 (RW95) and fell for tenants 2–4; successful-write p95 and workflow p95 increased for all four tenants. Summing tenant rates hides this workload redistribution, so the per-tenant table is the comparison of record. One 5-second run per mode and condition cannot settle repeatability, tail stability, or a general trade-off curve.

## Provenance and retained evidence

- Implementation and measurement source SHA: `85cadf0991de2b2052f5b6f7647dc46647b4ede6`.
- Workload binary SHA-256: `e54ebd3c6f842697fea1b8ce614bbb4ea87dd02c757dc96e8a8273b9a9bed8f1`.
- OCI: instance `instance-20260923-1013`, ARM64, Oracle Linux 9.8, Linux 6.12, two CPUs, ZFS `/bench/zfs/db`.
- Rust/Cargo 1.97.1, `aarch64-unknown-linux-gnu`, repository `+crc` flags, Release profile. Existing shared Cargo target and sccache were used.
- Existing RocksDB 11.8.1 source SHA `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; archive SHA-256 `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`. No RocksDB rebuild or new Cargo target.
- Each run used reserved revision ranges of 4,096, 16 clients, 10,000 rows per tenant, 512-byte values, seed `20261013`, 1-second warmup, 5-second admission, call cap 1,000,000, detailed metrics off, accuracy/count/latency reporting on, and the existing conditions, transaction widths 1/4/8, preparation Gets, deadline, drain, replay, and reopen checks. WAL, file-backed storage, and synchronous writes remained enabled.
- Exact commands, stdout, exit statuses, binary hashes, and JSON reports are under `oci/`. The unmodified build helper records are archived at `oci/builds.tar.gz`. Run databases remain on OCI at `/bench/zfs/db/dodb-service-blocking-admission-85cadf099/runs/`.

## Remaining limits

This experiment does not measure semaphore queue time, repeatability, long-run stability, power-loss behavior, or a production trace. It does not compare B-link, change sync/WAL/cache settings, or test another lane split. The split is opt-in; the default remains shared eight-slot admission.
