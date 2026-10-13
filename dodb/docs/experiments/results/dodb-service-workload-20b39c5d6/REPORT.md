# Dodb service-level workload comparison

Status: complete. This is a one-repetition, short synthetic comparison of the current B-link and RocksDB services through the shared `DodbService::execute` boundary. It is not production trace replay, a long-term stability result, or a claim about either storage engine in isolation.

## Provenance

- Implementation and measurement source SHA: `20b39c5d61d496d03c95225508d07c3b98222856` on `experiment/b-link-batched-engine-monorepo`; the OCI checkout was detached and clean at that exact SHA.
- Final report and artifact commit: recorded after this report is committed; it is distinct from the implementation SHA.
- OCI: `instance-20260923-1013`, ARM64 Linux 6.12, ZFS dataset mounted at `/bench/zfs/db`; 2 online processors. Toolchain: Rust/Cargo 1.97.1; target `aarch64-unknown-linux-gnu`, `+crc`, Release profile with debug symbols.
- Workload executable SHA-256: `f583ea753ca6559fd57f0a63daed284d57c83b5280075911cc117ab9b0754ea9`.
- RocksDB source: `abeebd9630f11bd08c28b7bd43c7bdfc62050654`, version 11.8.1. Existing archive SHA-256: `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`. The archive was reused and not rebuilt.
- Release build command: `DODB_BUILD_RUN_ID=service-workload-20b39c5d6-build-1 bash dodb/scripts/run-oci-rocksdb-build.sh build --locked --release -p dodb-rocksdb --bin service_workload`.
- Shared Cargo target: `/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target`. The final build recompiled `dodb-rocksdb` and linked the changed binary; workspace dependencies reused the existing target. sccache reported one non-cacheable call, no cache errors, and no RocksDB build. Build log and before/after cache reports are in `build-run/`.
- OCI run evidence remains at `/bench/zfs/db/dodb-service-workload-runs/20b39c5d61d496d03c95225508d07c3b98222856/`. Each record includes its exact command, exit code, binary hash, JSON report, stdout, and summary. The isolated DB directories remain on OCI and are not copied into this source report.

## Driver and workload

The driver selects and constructs only the backend-specific service. Workload plans, deterministic keys and values, client loops, admission, service requests, response classification, latency sampling, throughput, and validation are shared. Every measured request calls `DodbService::execute` with the same `Request`, `Response`, and `ExecutionBudget`; no raw storage API or transport is used. A single Tokio multi-thread runtime has 16 worker threads and the same closed-loop client structure for both backends.

Each profile uses a deterministic 1,000-call logical schedule. Preparatory Gets occupy the read quota and are counted as real service calls; a workflow reserves both its Get and following transaction before admission. Reads are 80% Get and 20% Query16, counting preparation Gets. Writes use 20% Put, 10% Delete, 30% conditional width 1, 30% conditional width 4, and 10% conditional width 8. Width 4 and 8 are each one `Transact` request with that many mutations. The conditional mix uses RevisionEquals, Exists, and NotExists in a 1:2:1 ratio; 10% of condition-bearing requests target an intentional rejection. Successful conditional writes use backend-returned revisions. Values change on every Put.

Query16 selects changing partition and cursor positions and requests up to 16 rows; returned rows are counted separately. Primary and sort keys are each 8 raw bytes (16 combined). Canonical encoding escapes zero bytes, so the 10,000-row initial dataset encodes to 33–36 bytes per key; the recorded sample key is 33 bytes. Each active tenant begins with 10,000 rows of 512-byte values, seeded through service API transactions of 64 mutations. Seed setup is excluded from throughput and reported separately.

Single-tenant runs use 16 clients. MT-MIX uses four open, active tenant databases in one process and four clients per tenant at RW95, RW80, RW50, and RW20. All runs use seed `20261013`, 1 s warmup, 5 s admission, one repetition, and a 1,000,000 API-call cap. Each run starts with an empty, unique database path. Backend order alternated across profiles; runs were serial.

B-link used the existing 128-page per-tenant cache, 256-entry coordinator queue, maximum group size 64 requests/4 MiB, and zero collection delay. RocksDB used the existing TransactionDB defaults (`create_if_missing`, WAL enabled, no compression); no cache, worker, or transaction options were tuned. Its service retained the 8-operation blocking limit and tenant-scoped revision-counter serialization. RocksDB write options remain synchronous with WAL enabled. B-link retains its coordinator/group-commit durability path. These are service configurations, not matched internal scheduling mechanisms.

## Single-tenant throughput and request counts

Rates use the same denominator for each row: actual admission plus drain. Actual read/write percentages use API calls; preparatory Gets are in reads. Successful write transactions include successful Put, Delete, and Transact requests; rejected conditions are excluded.

| Profile | Backend | Actual read/write | API calls/s | Read calls/s | Successful write tx/s | Mutations/s | Prep Gets | Query rows | Intended rejects | Natural conflicts | Other errors / overload |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| RW95 | blink | 94.99/5.01% | 43271 | 41105 | 1990 | 5170 | 1,968 | 202,110 | 882 | 0 | 0/0 |
| RW95 | rocksdb | 95.03/4.97% | 19611 | 18636 | 897 | 2325 | 880 | 92,382 | 390 | 0 | 0/0 |
| RW80 | rocksdb | 80.58/19.42% | 5188 | 4180 | 939 | 2296 | 906 | 21,009 | 345 | 0 | 0/0 |
| RW80 | blink | 80.26/19.74% | 14452 | 11600 | 2652 | 6498 | 2,528 | 57,202 | 1,001 | 0 | 0/0 |
| RW50 | blink | 50.01/49.99% | 6413 | 3207 | 2983 | 7502 | 2,821 | 16,159 | 1,118 | 0 | 0/0 |
| RW50 | rocksdb | 49.42/50.58% | 1819 | 899 | 880 | 2288 | 771 | 4,916 | 199 | 0 | 0/0 |
| RW20 | rocksdb | 19.40/80.60% | 1389 | 269 | 1020 | 2817 | 1,004 | 1,234 | 501 | 0 | 0/0 |
| RW20 | blink | 20.13/79.87% | 4076 | 820 | 3021 | 7855 | 2,897 | 3,992 | 1,172 | 0 | 0/0 |

## Single-tenant latency percentiles

Values are p95/p99 milliseconds. The JSON preserves p50, p95, p99, counts, sample counts, and bounded sampling limits for every operation.

| Profile | Backend | Get | Preparatory Get | Query16 | Successful write tx | Condition rejection | Whole workflow |
|---|---|---:|---:|---:|---:|---:|---:|
| RW95 | blink | 0.11/0.25 | 0.10/0.21 | 0.12/0.31 | 11.91/15.26 | 11.22/16.36 | 0.65/8.29 |
| RW95 | rocksdb | 1.30/2.15 | 1.17/2.03 | 1.19/1.95 | 13.78/21.15 | 0.92/1.56 | 1.97/7.74 |
| RW80 | rocksdb | 4.05/5.56 | 4.06/5.48 | 4.13/5.87 | 22.18/34.03 | 2.82/4.89 | 11.24/22.06 |
| RW80 | blink | 0.09/0.16 | 0.08/0.17 | 0.11/0.21 | 9.46/12.24 | 8.78/10.98 | 6.23/9.79 |
| RW50 | blink | 0.05/0.11 | 0.05/0.12 | 0.08/0.14 | 8.26/12.84 | 7.74/10.83 | 7.48/11.83 |
| RW50 | rocksdb | 10.43/15.53 | 11.23/18.06 | 10.22/15.34 | 30.49/50.90 | 5.56/12.73 | 24.66/43.95 |
| RW20 | rocksdb | 13.20/14.44 | 13.69/16.26 | 13.31/15.24 | 27.20/47.94 | 7.22/8.56 | 27.89/45.06 |
| RW20 | blink | 0.06/0.10 | 0.06/0.10 | 0.08/0.13 | 7.81/10.16 | 7.64/9.98 | 7.69/10.02 |

## Write and condition coverage

Each category cell is `attempts/successes`. Successful counters exclude all condition rejections. Condition-kind counts include condition-bearing requests; the actual intended rejection fraction was close to the 10% target in each single-tenant run.

| Profile | Backend | Put | Delete | Conditional width 1 | Conditional width 4 | Conditional width 8 | RevisionEquals | Exists | NotExists | Rejection rate |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| RW95 | blink | 2,184/2,184 | 1,060/1,060 | 3,282/2,848 | 3,229/2,781 | 1,080/1,080 | 1,968 | 3,873 | 1,750 | 11.62% |
| RW95 | rocksdb | 987/987 | 480/480 | 1,475/1,283 | 1,454/1,256 | 483/483 | 880 | 1,745 | 787 | 11.43% |
| RW80 | rocksdb | 938/938 | 502/502 | 1,608/1,518 | 1,511/1,351 | 487/392 | 906 | 1,779 | 921 | 9.57% |
| RW80 | blink | 2,799/2,799 | 1,407/1,407 | 4,384/4,150 | 4,282/3,789 | 1,399/1,125 | 2,528 | 5,004 | 2,533 | 9.95% |
| RW50 | blink | 3,201/3,201 | 1,619/1,619 | 4,809/4,267 | 4,807/4,551 | 1,598/1,278 | 2,821 | 5,608 | 2,785 | 9.97% |
| RW50 | rocksdb | 861/861 | 550/550 | 1,280/1,193 | 1,448/1,400 | 471/407 | 771 | 1,623 | 805 | 6.22% |
| RW20 | rocksdb | 1,193/1,193 | 552/552 | 1,476/1,255 | 1,644/1,445 | 749/668 | 1,004 | 1,827 | 1,038 | 12.95% |
| RW20 | blink | 3,297/3,297 | 1,678/1,678 | 4,681/4,216 | 4,872/4,325 | 1,762/1,602 | 2,897 | 5,649 | 2,769 | 10.36% |

## Multi-tenant MT-MIX

The combined row is descriptive only: the four tenants use different mixes and different achieved rates, so it is not a fairness result. Compare each tenant row with the same profile in the other backend. Per-operation p50/p95/p99 remain available in each run JSON.

| Backend | Tenant profile | Actual read/write | API calls/s | Read calls/s | Successful write tx/s | Mutations/s | Prep Gets | Query rows | Intended rejects | Natural conflicts | Write tx p95/p99 ms |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| rocksdb | combined, descriptive | 71.22/28.78% | 7281 | 5185 | 1946 | 5009 | 1,826 | 25,997 | 745 | 5 | 8.08/11.54 |
| rocksdb | tenant_1 rw95 | 95.06/4.94% | 2507 | 2383 | 114 | 294 | 108 | 12,038 | 48 | 0 | 4.24/5.73 |
| rocksdb | tenant_2 rw80 | 79.96/20.04% | 2360 | 1887 | 439 | 1171 | 416 | 9,371 | 168 | 0 | 5.04/7.21 |
| rocksdb | tenant_3 rw50 | 49.45/50.55% | 1465 | 724 | 687 | 1744 | 656 | 3,554 | 260 | 5 | 7.64/10.51 |
| rocksdb | tenant_4 rw20 | 20.06/79.94% | 948 | 190 | 704 | 1800 | 646 | 1,034 | 269 | 0 | 9.40/13.82 |
| blink | combined, descriptive | 84.99/15.01% | 21399 | 18186 | 2978 | 7705 | 2,837 | 90,110 | 1,169 | 8 | 8.61/12.12 |
| blink | tenant_1 rw95 | 95.02/4.98% | 14618 | 13890 | 670 | 1746 | 656 | 68,822 | 288 | 0 | 9.11/12.05 |
| blink | tenant_2 rw80 | 80.01/19.99% | 4068 | 3254 | 755 | 2002 | 715 | 16,103 | 292 | 0 | 8.51/11.35 |
| blink | tenant_3 rw50 | 50.04/49.96% | 1660 | 831 | 769 | 1941 | 746 | 4,064 | 293 | 8 | 8.44/12.37 |
| blink | tenant_4 rw20 | 20.05/79.95% | 1054 | 211 | 783 | 2016 | 720 | 1,121 | 296 | 0 | 8.46/12.21 |

## Resources, timing, and validation

Process CPU is reported both as percent of one core and as percent of the two-processor host. RSS is the peak 200 ms `/proc` sample during admission and drain. Disk use is bytes occupied below that run’s isolated DB path after measurement and before shutdown. All ten RSS sample series were complete; numbers are point observations from this short run.

| Condition | Backend | CPU, one core % | CPU, host % | Peak RSS MiB | DB files MiB | Admission ms | Drain ms | Seed setup ms | Warmup API calls |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| RW95 | blink | 68.6 | 34.3 | 98.1 | 149.8 | 5000.79 | 0.01 | 558.3 | 45,326 |
| RW95 | rocksdb | 98.7 | 49.4 | 31.4 | 12.5 | 5001.65 | 3.16 | 419.2 | 16,557 |
| RW80 | rocksdb | 53.3 | 26.7 | 27.5 | 12.6 | 5000.76 | 7.79 | 380.7 | 5,873 |
| RW80 | blink | 46.4 | 23.2 | 113.7 | 190.1 | 5000.73 | 2.65 | 563.1 | 15,488 |
| RW50 | blink | 39.0 | 19.5 | 121.5 | 209.0 | 5000.93 | 0.01 | 546.6 | 6,588 |
| RW50 | rocksdb | 29.7 | 14.9 | 25.9 | 12.7 | 5000.37 | 9.39 | 389.7 | 2,105 |
| RW20 | rocksdb | 27.3 | 13.7 | 27.4 | 14.3 | 5002.20 | 12.40 | 379.6 | 1,412 |
| RW20 | blink | 41.0 | 20.5 | 130.4 | 220.4 | 5002.30 | 2.09 | 558.2 | 4,340 |
| MT-MIX | rocksdb | 73.3 | 36.7 | 54.7 | 37.1 | 5002.78 | 1.35 | 1545.0 | 6,692 |
| MT-MIX | blink | 71.0 | 35.5 | 191.9 | 262.8 | 5002.38 | 1.13 | 2403.1 | 21,236 |

All 10 OCI measurements passed: exit status 0, final Scan against the independent expected state, unique commit identities, shutdown/reopen, and second Scan. The Scan validator compares 64-row pages with a streaming expected sequence; it keeps a bounded write ledger and at most 16 mismatch samples rather than retaining all returned documents. API-call and latency samples are bounded. Single-tenant runs recorded zero natural conflicts, overloads, and other errors. MT-MIX recorded five natural conflicts on RocksDB and eight on B-link, reported separately from intentional rejection; neither run had overloads or other errors.

Warmup writes are excluded from measured counters and rates but retained in the expected-state replay. Each run records measured admission, drain, and the exact denominator `admission + drain`; completion during drain is included in counts and the denominator. The observed drain ranged from sub-millisecond to about 12 ms.

WAL sync counts, transaction/group counts, and RocksDB compaction events are unavailable through the existing service interfaces. This short run does not establish whether compaction occurred. The storage footprint and RSS above are observed; they do not establish long-term steady state.

## Interpretation and limits

On these single runs, B-link delivered roughly 2.2–3.5 times the successful write-transaction rate across the four profiles. Its successful write p95/p99 was lower in the write-heavier profiles; RocksDB showed higher Get/Query and write tails as the write share rose. The workloads include actual preparatory Gets, atomic width-4/8 transactions, real WAL-backed synchronous writes, and condition rejection cost.

This compares the current service implementations. B-link can group durable work through its coordinator; RocksDB uses TransactionDB with a tenant-scoped revision counter that serializes successful writes, plus an 8-operation blocking limit. Those service mechanisms remain part of the observed difference. The result is not a statement about RocksDB maximum capacity or general engine superiority. Each condition ran once for five seconds, and the paired runs are preliminary.

MT-MIX aggregate throughput is not used to claim tenant fairness. Its tenant-level rows show different achieved rates and tails under different profiles. The four-tenant case demonstrates this one topology only.

## Attempts and artifact history

- Local schedule checks: 3/3 passed, including all four ratios, preparation-read quota, 80/20 Get/Query composition, condition rejection target, write categories, and width-4/8 Transact schedule.
- Local smoke: B-link and RocksDB each processed 4,000 calls over four tenants; all write categories and condition kinds were reached; final and reopen scans passed for all four tenants.
- OCI smoke at this SHA: B-link and RocksDB each processed 4,000 calls over four tenants; all checks above passed. Smoke is not included in the ten measured records.
- OCI selected measurements: 10/10 passed, one per declared condition. No measured run failed or required a retry.
- An earlier 10-run set at SHA `37f3ca73861fb6db3d1d5ec5becac5bb60c290ed` is preserved in [`dodb-service-workload-37f3ca738`](../dodb-service-workload-37f3ca738/). It is superseded because its final validator materialized the full Scan in memory. The bounded streaming validator was committed at `20b39c5d6` and all 10 selected measurements were rerun from that exact clean SHA.

## Next investigation

Attribute RocksDB write latency to wait on the tenant revision-counter lock. Keep the allocator unchanged during attribution; this targets the observed per-profile transaction-rate and tail gap without conflating it with a future allocator redesign.

## Artifacts

- `measure/<profile>/<backend>/report.json`: complete counters, operation and workflow percentiles, per-tenant results, resource samples, and final/reopen validation.
- `measure/<profile>/<backend>/command.txt`, `stdout.log`, `summary.txt`, and `exit-status.txt`: exact invocation and execution outcome.
- `smoke/<backend>/`: OCI smoke reports and commands.
- `local-smoke/`: local smoke reports and focused schedule test log.
- `build-run/`: OCI helper environment, compiler log, hashes, and sccache snapshots.
- `host-and-source.txt`, `binary.sha256`, and `rocksdb-archive.sha256`: host, clean checkout, toolchain, and binary/archive identity.
