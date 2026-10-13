# Dodb service-level workload comparison

Status: complete. This is a single-repetition, short synthetic comparison of the current B-link and RocksDB services through the shared service API. It is not production trace replay, a long-term stability result, or a claim about either storage engine in isolation.

## Provenance

- Implementation and measurement source SHA: `cbda6ed41099c569b060884a71bd2f1d98c1df7b` on `experiment/b-link-batched-engine-monorepo`; OCI checkout was detached and clean at this SHA.
- Report and artifact commit SHA is recorded after this report is committed; it is distinct from the implementation SHA.
- OCI `instance-20260923-1013`, ARM64 Linux 6.12, 2 online processors, ZFS `/bench/zfs/db`; Rust/Cargo 1.97.1, `aarch64-unknown-linux-gnu`, `+crc`, Release. See `host-and-source.txt`.
- Workload binary SHA-256: `69aabc42d08dfdf4185d40270e233a834c78edd67a5eb63a977dc82d7fbe0f02`. Existing RocksDB 11.8.1 source SHA `abeebd9630f11bd08c28b7bd43c7bdfc62050654`, archive SHA-256 `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`; archive reused without rebuild.
- Build command: `DODB_BUILD_RUN_ID=service-workload-cbda6ed41-build-1 bash dodb/scripts/run-oci-rocksdb-build.sh build --locked --release -p dodb-rocksdb --bin service_workload`. Shared target path is recorded in `build-run/environment.txt`; build exit 0, 22.23 seconds. Only `dodb-rocksdb` compiled/linked. sccache had two non-cacheable calls after build and zero cache errors; existing workspace target and RocksDB archive were reused.
- The ten selected run records and two OCI smoke records are in `oci-runs/`; each includes command, JSON, stdout, exit status, and before/after binary hashes. Isolated DB directories remain on OCI at `/bench/zfs/db/dodb-service-workload-runs/cbda6ed41099c569b060884a71bd2f1d98c1df7b/` and are not copied here.

## Driver and workload

Only service construction differs by backend. Both use one 16-worker Tokio runtime and the same client structure, schedule generation, admission/deadline/drain logic, request classification, latency/throughput collection, and verifier. Every request is sent through `DodbService::execute` with shared Request/Response and ExecutionBudget; the driver does not call raw storage APIs or add a transport.

Each profile has a deterministic logical 1,000-API-call schedule. Preparatory Gets consume read quota, count as real service calls, and are reserved atomically with their following conditional write. Reads are 80% Get and 20% Query16. Query partition and cursor positions advance; returned row counts are recorded. Writes are 20% unconditional Put, 10% unconditional Delete, 30% conditional width 1, 30% conditional width 4, and 10% conditional width 8. Width 4/8 each use one Transact request. Conditional types are RevisionEquals, Exists, NotExists in a 1:2:1 ratio; 10% of condition-bearing plans intentionally reject. Natural conflicts are counted separately. Conditional revisions are read from the active backend, and Put values change.

Per-client fingerprints cover the first 100 admitted logical workflows. The paired B-link/RocksDB fingerprints matched for all 16 clients in every single-tenant profile and for all clients in MT-MIX; the hash input includes profile, seed, tenant/client, workflow identifier, request kind/flags and key ordinal, and excludes engine-assigned revision numbers. Full schedules and actual client prefixes are in each JSON report.

Each key has an 8-byte raw partition key plus an 8-byte raw sort key (16 combined raw bytes). Canonical encoded keys in the seeded dataset range 33–36 bytes; the recorded sample key is 33 bytes. Each tenant starts with 10,000 rows, each value 512 bytes. Seed data uses service API transactions of 64 mutations; seed time is excluded. Each condition gets a unique database path.

Single-tenant conditions use 16 clients. MT-MIX runs four tenants in one process with four clients each: tenant_1 RW95, tenant_2 RW80, tenant_3 RW50, tenant_4 RW20. Every run uses seed `20261013`, 1 s warmup, 5 s workflow-admission interval, and one repetition. Backend order alternates by profile; conditions run serially. Admission stops at the deadline and admitted workflows drain. Throughput denominator is measured admission plus drain; drained completions are included with that same denominator.

B-link retained 128 cached pages, queue capacity 256, maximum coordinator group 64 requests/4 MiB, zero collection delay. RocksDB retained WAL and synchronous writes, TransactionDB defaults, and the existing 8-operation blocking limit. RocksDB tenant revision-counter serialization and B-link group commit remain in the observed service implementations; no internals were retuned.

## Single-tenant throughput and request counts

The actual ratio is based on API calls and includes preparatory Gets. `Write tx/s` includes successful Put/Delete/Transact requests; condition rejections do not count. Query values show call count/returned rows. All raw counts and rates use the same measured admission-plus-drain denominator.

| Profile | Backend | Actual read/write | API/read/write calls | API/s | Read/s | Successful write tx/s | Successful write tx count | Mutations/s | Successful mutations | Prep Gets | Ordinary Gets | Query16 calls/rows | Intentional rejects | Natural conflicts | Other/overload |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RW95 | blink | 94.99%/5.01% | 220,765/209,713/11,052 | 44,126 | 41,917 | 2,030 | 10,156 | 5,276 | 26,398 | 2,002 | 165,743 | 41,968/206,140 | 896 | 0 | 0/0 |
| RW95 | rocksdb | 95.02%/4.98% | 81,110/77,070/4,040 | 16,205 | 15,398 | 743 | 3,720 | 1,929 | 9,656 | 724 | 60,923 | 15,423/76,174 | 320 | 0 | 0/0 |
| RW80 | blink | 80.14%/19.86% | 76,132/61,010/15,122 | 15,222 | 12,198 | 2,813 | 14,070 | 6,940 | 34,708 | 2,676 | 46,082 | 12,252/60,039 | 1,052 | 0 | 0/0 |
| RW80 | rocksdb | 80.56%/19.44% | 25,902/20,867/5,035 | 5,168 | 4,163 | 936 | 4,689 | 2,285 | 11,451 | 899 | 15,761 | 4,207/20,940 | 346 | 0 | 0/0 |
| RW50 | blink | 50.01%/49.99% | 32,454/16,231/16,223 | 6,485 | 3,243 | 3,017 | 15,098 | 7,586 | 37,960 | 2,850 | 10,151 | 3,230/16,311 | 1,125 | 0 | 0/0 |
| RW50 | rocksdb | 49.38%/50.62% | 8,785/4,338/4,447 | 1,753 | 866 | 850 | 4,260 | 2,216 | 11,103 | 737 | 2,656 | 945/4,703 | 187 | 0 | 0/0 |
| RW20 | blink | 20.04%/79.96% | 20,908/4,191/16,717 | 4,178 | 838 | 3,097 | 15,496 | 8,065 | 40,355 | 2,970 | 400 | 821/4,051 | 1,221 | 0 | 0/0 |
| RW20 | rocksdb | 19.30%/80.70% | 7,050/1,361/5,689 | 1,407 | 272 | 1,035 | 5,187 | 2,854 | 14,306 | 1,016 | 97 | 248/1,217 | 502 | 0 | 0/0 |
## Single-tenant latency percentiles

Every cell gives p50/p95/p99 milliseconds. Samples are bounded at 20,000 per metric; exact sample counts and counts are retained in JSON.

| Profile | Backend | Get | Prep Get | Query16 | Successful write tx | Condition rejection | Whole workflow |
| --- | --- | --- | --- | --- | --- | --- | --- |
| RW95 | blink | 0.01/0.09/0.23 | 0.01/0.08/0.18 | 0.02/0.13/0.26 | 6.12/12.12/15.98 | 5.35/10.70/13.55 | 0.01/0.61/8.43 |
| RW95 | rocksdb | 0.52/1.66/2.72 | 0.69/1.66/2.44 | 0.70/1.73/2.56 | 3.45/13.94/30.00 | 0.40/1.37/2.48 | 0.52/2.74/11.14 |
| RW80 | blink | 0.01/0.06/0.14 | 0.01/0.05/0.14 | 0.02/0.08/0.18 | 4.64/8.83/11.24 | 4.65/8.85/11.96 | 0.01/5.87/8.84 |
| RW80 | rocksdb | 1.53/4.01/5.32 | 1.51/3.88/5.24 | 1.53/3.92/5.21 | 6.82/21.86/32.76 | 0.74/2.50/3.34 | 1.96/11.06/21.26 |
| RW50 | blink | 0.01/0.06/0.11 | 0.01/0.06/0.11 | 0.02/0.08/0.18 | 4.49/8.15/10.57 | 4.25/7.88/10.23 | 3.56/7.03/9.52 |
| RW50 | rocksdb | 5.87/10.72/15.29 | 5.84/10.17/14.01 | 5.62/9.75/12.31 | 9.16/30.24/53.89 | 2.95/5.43/7.32 | 7.51/24.81/44.73 |
| RW20 | blink | 0.01/0.05/0.08 | 0.01/0.06/0.10 | 0.03/0.08/0.12 | 4.37/8.01/11.01 | 4.13/8.02/11.08 | 4.30/7.94/10.79 |
| RW20 | rocksdb | 9.59/12.95/16.68 | 9.42/13.42/15.34 | 9.33/13.04/14.10 | 10.14/27.01/44.79 | 4.61/7.17/8.10 | 10.84/27.99/41.33 |

## Write and condition coverage

Category columns show attempts/successes. The condition columns count condition-bearing write requests. The actual intentional-rejection target was 10%; achieved percentages vary due admission ending on whole workflows and finite execution.

| Profile | Backend | Put | Delete | Cond width 1 | Cond width 4 | Cond width 8 | RevisionEquals | Exists | NotExists | Intentional rejects/conditional attempts |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RW95 | blink | 2,233/2,233 | 1,098/1,098 | 3,319/2,871 | 3,307/2,859 | 1,095/1,095 | 2,002 | 3,950 | 1,769 | 896/7,721 |
| RW95 | rocksdb | 821/821 | 400/400 | 1,215/1,055 | 1,203/1,043 | 401/401 | 724 | 1,442 | 653 | 320/2,819 |
| RW80 | blink | 2,961/2,961 | 1,488/1,488 | 4,625/4,391 | 4,494/3,993 | 1,554/1,237 | 2,676 | 5,278 | 2,719 | 1,052/10,673 |
| RW80 | rocksdb | 935/935 | 501/501 | 1,605/1,515 | 1,511/1,351 | 483/387 | 899 | 1,786 | 914 | 346/3,599 |
| RW50 | blink | 3,239/3,239 | 1,654/1,654 | 4,847/4,299 | 4,876/4,620 | 1,607/1,286 | 2,850 | 5,652 | 2,828 | 1,125/11,330 |
| RW50 | rocksdb | 806/806 | 545/545 | 1,241/1,156 | 1,405/1,357 | 450/396 | 737 | 1,568 | 791 | 187/3,096 |
| RW20 | blink | 3,366/3,366 | 1,696/1,696 | 4,835/4,345 | 5,011/4,441 | 1,809/1,648 | 2,970 | 5,770 | 2,915 | 1,221/11,655 |
| RW20 | rocksdb | 1,206/1,206 | 558/558 | 1,504/1,282 | 1,665/1,467 | 756/674 | 1,016 | 1,856 | 1,053 | 502/3,925 |

## Multi-tenant MT-MIX

Each row describes one distinct workload. The combined rate is descriptive only because tenants have different API mixes and different achieved rates; it is not a fairness claim. Query values show call count/returned rows.

| Backend | Tenant | Profile | Actual read/write | API/read/write calls | API/s | Read/s | Successful write tx/s | Successful tx count | Mutations/s | Successful mutations | Prep Gets | Ordinary Gets | Query16 calls/rows | Rejects | Conflicts | Write tx p95/p99 ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| rocksdb | tenant_1 | RW95 | 95.12%/4.88% | 12,973/12,340/633 | 2,592 | 2,466 | 117 | 585 | 305 | 1,525 | 113 | 9,768 | 2,459/12,387 | 48 | 0 | 3.57/4.72 |
| rocksdb | tenant_2 | RW80 | 80.00%/20.00% | 12,232/9,786/2,446 | 2,444 | 1,955 | 453 | 2,266 | 1,199 | 6,001 | 432 | 7,409 | 1,945/9,658 | 180 | 0 | 4.36/5.65 |
| rocksdb | tenant_3 | RW50 | 49.47%/50.53% | 7,598/3,759/3,839 | 1,518 | 751 | 713 | 3,569 | 1,818 | 9,097 | 669 | 2,346 | 744/3,633 | 264 | 6 | 6.99/9.46 |
| rocksdb | tenant_4 | RW20 | 20.06%/79.94% | 4,886/980/3,906 | 976 | 196 | 725 | 3,630 | 1,860 | 9,308 | 665 | 102 | 213/1,058 | 276 | 0 | 8.85/12.32 |
| blink | tenant_1 | RW95 | 95.00%/5.00% | 72,084/68,483/3,601 | 14,411 | 13,691 | 662 | 3,313 | 1,724 | 8,624 | 649 | 54,133 | 13,701/67,869 | 288 | 0 | 9.25/13.95 |
| blink | tenant_2 | RW80 | 79.99%/20.01% | 19,869/15,894/3,975 | 3,972 | 3,178 | 738 | 3,692 | 1,965 | 9,828 | 698 | 12,019 | 3,177/15,759 | 283 | 0 | 8.67/11.66 |
| blink | tenant_3 | RW50 | 49.91%/50.09% | 7,988/3,987/4,001 | 1,597 | 797 | 744 | 3,719 | 1,882 | 9,415 | 705 | 2,489 | 793/3,884 | 278 | 4 | 8.67/12.64 |
| blink | tenant_4 | RW20 | 20.11%/79.89% | 5,220/1,050/4,170 | 1,044 | 210 | 775 | 3,876 | 1,997 | 9,991 | 713 | 112 | 225/1,120 | 294 | 0 | 7.93/12.06 |
MT-MIX per-tenant operation latency p50/p95/p99, categories, condition-kind counts, and full counters are preserved in each backend `report.json`. The aggregate descriptive values are:

| Backend | Actual read/write | API/s | Read/s | Successful write tx/s | Mutations/s | Prep Gets | Query rows | Rejects | Conflicts |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| rocksdb | 71.28%/28.72% | 7,531 | 5,368 | 2,008 | 5,182 | 1,879 | 26,736 | 768 | 6 |
| blink | 85.03%/14.97% | 21,024 | 17,876 | 2,919 | 7,569 | 2,765 | 88,632 | 1,143 | 4 |

## Resources, timing, and validation

CPU percentages are normalized to one core and to the two-processor host. RSS is peak sampled `/proc` RSS in MiB at 200 ms intervals during admission and drain; all selected runs reported complete samples. DB disk use is the run directory’s regular-file bytes after measurement and before shutdown.

| Condition | Backend | CPU one core % | CPU host % | Peak RSS MiB | DB files MiB | Admission ms | Drain ms | Seed setup ms, excluded | Warmup API calls |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| RW95 | blink | 66.8 | 33.4 | 99.7 | 152.1 | 5001.17 | 1.90 | 582.0 | 45,281 |
| RW95 | rocksdb | 92.3 | 46.2 | 31.2 | 11.5 | 5002.08 | 3.06 | 401.5 | 18,553 |
| RW80 | blink | 47.0 | 23.5 | 112.7 | 189.8 | 5001.48 | 0.01 | 621.1 | 15,578 |
| RW80 | rocksdb | 55.5 | 27.7 | 27.3 | 12.6 | 5002.15 | 9.91 | 386.5 | 5,820 |
| RW50 | blink | 40.6 | 20.3 | 121.3 | 206.7 | 5000.26 | 3.94 | 579.2 | 5,317 |
| RW50 | rocksdb | 28.9 | 14.5 | 25.8 | 12.4 | 5000.04 | 11.04 | 376.3 | 1,967 |
| RW20 | blink | 38.4 | 19.2 | 132.2 | 225.0 | 5002.39 | 1.52 | 613.4 | 4,372 |
| RW20 | rocksdb | 26.5 | 13.3 | 27.3 | 14.3 | 5003.17 | 8.91 | 364.6 | 1,296 |
| MT-MIX | rocksdb | 75.7 | 37.9 | 55.6 | 38.0 | 5000.57 | 3.86 | 2045.1 | 7,843 |
| MT-MIX | blink | 70.4 | 35.2 | 189.6 | 258.0 | 5000.50 | 1.38 | 2455.3 | 22,258 |

All selected ten runs passed their independent expected-state scan, unique commit-identity check, shutdown/reopen, and second scan. The verifier replays successful mutations by commit identity, excludes rejected writes, streams Scan pages of 64 rows, and holds only bounded write history and mismatch samples. Expected initial state plus the successful mutation stream is independent of backend dumps. Read validation uses request snapshots and bounded samples so later mutation does not invalidate an earlier consistent read.

Process CPU/RSS were available and complete. WAL sync counts and transaction/group counts are unavailable through the existing service interfaces. Compaction events are not exposed by this driver, so whether compaction occurred in these short runs is unconfirmed. No long-term steady-state result is claimed.

## Interpretation and limits

In these single repetitions, B-link achieved higher service-level throughput in all four single-tenant profiles. Successful write transaction rate was approximately 2.0–3.0k/s for B-link versus 0.7–1.0k/s for RocksDB; the paired request-prefix hashes matched. B-link successful write p95/p99 was lower across the profiles, while RocksDB Get/Query latency and write tails increased as write share rose. These are observations from this synthetic five-second admission interval, not a general engine benchmark.

The comparison includes current service mechanisms: B-link durable work can share group commit; RocksDB has a tenant revision-counter lock that serializes writes and an 8-operation blocking limit. The driver did not redesign that counter or tune either service. Results characterize these two implementations, not RocksDB maximum performance or all workloads. The initial per-condition 10% deliberate-rejection goal is approximate in finite, workflow-admitted counts; rejected conditions are never counted as successful writes.

## Attempts and next investigation

- Focused schedule checks: 3/3 passed. They cover all four ratios, preparatory Get quota/calls, Get/Query split, category coverage, condition targeting, and width-4/8 single Transact workflows.
- Local smoke: B-link and RocksDB each completed 4,000 API calls across four tenants; final and reopen scans passed.
- OCI smoke: 2/2 passed (4,000 calls/backend across four tenants), excluded from measurements.
- OCI measurement: 10/10 passed, exactly one run per declared condition. No run failed or needed retry. Order was RW95 B-link/RocksDB, RW80 RocksDB/B-link, RW50 B-link/RocksDB, RW20 RocksDB/B-link, MT-MIX RocksDB/B-link.
- Earlier complete measurement set at SHA `37f3ca73861fb6db3d1d5ec5becac5bb60c290ed` is retained under `../dodb-service-workload-37f3ca738/`; its Scan verifier retained all rows in memory, so it was superseded by bounded streaming validation. The `20b39c5d6` set passed with bounded scans but was superseded to add and verify per-client paired input-prefix fingerprints. Both earlier sets are preserved.

Next investigation: attribute RocksDB write latency to wait on the tenant revision-counter lock, using the existing implementation and service workload while collecting narrowly scoped lock-wait evidence. Do not optimize the allocator in that attribution step.

## Artifacts

- `oci-runs/<condition>/<backend>/report.json`: complete counters, p50/p95/p99, per-tenant records, resource/timing data, validation.
- `oci-runs/<condition>/<backend>/command.txt`, `stdout.log`, `exit-status.txt`, and binary hashes: exact invocation and run evidence.
- `oci-runs/smoke/<backend>/`: OCI smoke records.
- `local-smoke-*.json` and `local-schedule-tests.log`: local checks.
- `build-run/`: helper environment, compiler output, archive/build provenance and cache snapshots.
- `host-and-source.txt`, `binary.sha256`, and `rocksdb-archive.sha256`: environment and executable/archive identity.
