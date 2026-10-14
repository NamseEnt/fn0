# RocksDB Read Tenant Lookup Fast Path

Status: complete. This report covers one five-second OCI measurement for each of five reserved-range RocksDB scenarios before and after the read lookup change. It is a preliminary synthetic service workload comparison, not a repeatability or production claim.

## Provenance and environment

- Baseline source SHA: `de790c685dee9551170d6094c945ff03ca762ac4`.
- Fast-path implementation SHA: `b4325ee98be07ed7efe10119bf4e7b7c997a2b25`, pushed to `experiment/b-link-batched-engine-monorepo`. The report and implementation SHAs are distinct commits.
- OCI host: `instance-20260923-1013`, ARM64, Oracle Linux 9.8, Linux 6.12, two online CPUs, ZFS `/bench/zfs/db`.
- Rust/Cargo 1.97.1, `aarch64-unknown-linux-gnu`, repository `+crc` target flags, Release profile.
- Both binaries were built from clean detached checkouts with the existing helper and shared Cargo target. Existing RocksDB 11.8.1 source SHA: `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; archive SHA-256: `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`. No RocksDB rebuild or new Cargo target was used.
- Baseline binary SHA-256: `10c12d6b540d0b6175c9dcf0e288c7ad716e20db5f43236bd00b2c9a82fd02d0`.
- Fast-path binary SHA-256: `725e5cb017ec3f038a70eee43772b787713cae9e3adba9a94a18b81e86393f72`.
- The only source difference between the two SHAs is `dodb/crates/dodb-rocksdb/src/lib.rs`. The two binaries are retained separately on OCI under `/bench/zfs/db/dodb-read-fastpath-b4325ee98/binaries/`; their hashes and build evidence are recorded in `oci/host-and-source.txt` and `oci/builds/`.
- `oci/run-matrix.sh` is the exact serial workload runner. Every run keeps its command, stdout, exit status, binary hashes, and JSON report under `oci/runs/<run>/`. Run databases stay on OCI at `/bench/zfs/db/dodb-read-fastpath-b4325ee98/oci/runs/`.

## Lookup change

Before this change, Get, Query, and Scan each called `database(tenant, false)`. Even when the registry `OnceCell` already held an open `TenantDatabase`, that method submitted `path.exists()` through `run_blocking`, acquired a shared blocking permit, and only then cloned the initialized database `Arc`.

Now these read requests first look up the tenant's cell and call `OnceCell::get()`. If initialization has completed, they clone the contained `Arc<TenantDatabase>` and proceed to the existing read submission. The DashMap guard is released immediately after the clone; the registry lock is not held during disk operations or the read. The request already holds its lifecycle lease at this point.

A missing registry entry or a cell whose initialization is still in progress does not count as an open database. Those requests use the unchanged cold path: check the directory through the existing blocking path, return an empty/missing read without creating a database when it is absent, and use the shared `OnceCell::get_or_try_init` open when it exists. This retains one open under concurrent cold access and retains retry after an open error.

The read owns an `Arc`; the actual blocking closure also owns that `Arc` and its semaphore permit. Request cancellation therefore cannot destroy the database under an in-flight blocking operation. Shutdown still closes admission, waits for requests and all blocking permits, clears the registry, and permits a later request to reopen the tenant.

## Focused checks

Local RocksDB archive was present at `/tmp/dodb-r1a-rocksdb-abeebd9`; all local checks below used it with `--locked --offline` and Release mode. Crash tests were not run.

- `cargo test --locked --offline --release -p dodb-rocksdb --lib --test read_contract --test transaction_contract`: 16 library tests, 2 read contract tests, and 1 transaction contract test passed.
- A test-only counter records actual calls into the path-existence branch. Repeated warm Get, Query, and Scan requests each completed with zero path checks. An unregistered on-disk tenant was found and opened once under eight simultaneous Gets; later reads made no additional path checks. A cell held in its initialization future was not treated as open, and the Get waited for initialization before returning the persisted value.
- Missing-tenant Get, Query, and Scan each used the cold path, returned empty/missing results, and left the tenant directory absent; the test observed three path checks and zero opens.
- An intentionally failed open left the `OnceCell` retryable; after replacing the invalid path with a valid database, the next Get opened it and returned the persisted value.
- Existing snapshot/read contract, transaction conditions and revisions, shutdown with an in-flight read, cancellation while a blocking operation drains, and same-service shutdown/reopen tests passed.
- The first local test attempt exposed an existing test hook that waited for the second blocking submission. After the fast path removed that submission, the hook was moved to the first submission (the actual Get). The complete focused command was rerun and passed.

## OCI smoke and measurement plan

One fast-path smoke was accepted before the measured matrix. It used the shared workload driver and full state plus shutdown/reopen replay verification. All four tenants passed both validations.

The first smoke attempt used 128 initial rows. The existing workload's delete replay expects the first 500 seeded keys, so the reduced seed produced row-count mismatches despite no key/value mismatch samples. That attempt is preserved under `oci/smoke/` and excluded from the measurements. The smoke was rerun with the required 10,000 initial rows per tenant and passed. The first checker also asked for a nonexistent top-level `verification` key; the driver exposes `validation` and `reopen_validation`. The corrected checker reads both fields. These smoke issues did not change product code or measurement inputs.

The ten planned measurements were all executed serially. Version order alternated within each scenario pair: baseline/fast RW95, fast/baseline RW80, baseline/fast RW50, fast/baseline RW20, baseline/fast MT-MIX. No measurement failed or needed a rerun; none was left unexecuted. All ten state replays and shutdown/reopen validations passed, and the paired per-client workflow input fingerprints matched.

Settings for each measured run: `DodbService::execute`; reserved revision ranges of 4,096; detailed research and read-stage metrics off; 16 clients total; MT-MIX four clients per tenant; 10,000 initial rows per tenant; 512-byte values; seed `20261013`; warmup 1,000 ms; measurement admission 5,000 ms; `call_cap=1,000,000`; fresh DB directory per run. The driver retained its existing workload mix, deadline, drain, independent replay validation, and reopen validation. RocksDB WAL and `sync=true` remained active; the existing eight-operation semaphore and all write/read execution structures were unchanged.

## Single-tenant results

Throughput is measured over admission plus drain. The actual read/write ratio includes preparatory Gets. Successful write transaction rate excludes condition rejections. Latencies are milliseconds in `p50/p95/p99` order; successful write and workflow show `p95/p99`.

| Profile | Version | API/s | Successful write tx/s | Read/write | Get p50/p95/p99 | Query16 p50/p95/p99 | Write p95/p99 | Workflow p95/p99 | Rejects / conflicts / overload / other |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| RW95 | baseline | 19,194 | 879 | 95.02/4.98% | 0.67/1.73/2.59 | 0.63/1.70/2.66 | 6.29/9.70 | 2.14/4.43 | 384 / 0 / 0 / 0 |
| RW95 | fast | 39,928 | 1,832 | 95.01/4.99% | 0.22/0.61/1.10 | 0.23/0.59/1.02 | 5.96/11.00 | 1.27/4.00 | 810 / 0 / 0 / 0 |
| RW80 | baseline | 12,689 | 2,352 | 80.08/19.92% | 0.70/1.88/2.59 | 0.70/1.81/2.53 | 7.79/15.83 | 3.47/8.00 | 880 / 0 / 0 / 0 |
| RW80 | fast | 12,455 | 2,306 | 80.11/19.89% | 0.57/1.75/2.33 | 0.60/1.82/2.43 | 9.41/18.25 | 4.01/10.13 | 857 / 0 / 0 / 0 |
| RW50 | baseline | 5,351 | 2,528 | 49.69/50.31% | 2.01/4.16/5.67 | 2.04/4.18/5.64 | 9.25/17.48 | 6.96/14.35 | 823 / 0 / 0 / 0 |
| RW50 | fast | 5,389 | 2,547 | 49.61/50.39% | 1.40/3.02/4.17 | 1.42/2.98/3.98 | 10.14/18.48 | 7.53/15.88 | 841 / 0 / 0 / 0 |
| RW20 | baseline | 3,424 | 2,547 | 20.11/79.89% | 3.57/6.47/7.13 | 3.62/7.46/9.66 | 10.97/20.30 | 11.76/20.29 | 943 / 0 / 0 / 0 |
| RW20 | fast | 3,514 | 2,612 | 20.12/79.88% | 2.20/4.00/4.77 | 2.27/4.27/5.28 | 11.06/19.30 | 10.75/18.60 | 975 / 0 / 0 / 0 |

## MT-MIX tenant results

Each tenant uses four clients. Latencies are milliseconds in `p50/p95/p99` order for Get and Query16; write and workflow show `p95/p99`.

| Version | Tenant / profile | API/s | Write tx/s | Read/write | Get p50/p95/p99 | Query16 p50/p95/p99 | Write p95/p99 | Workflow p95/p99 | Rejects / conflicts / overload / other |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | tenant 1 / RW95 | 3,066 | 141 | 94.99/5.01% | 0.97/2.87/4.57 | 1.04/3.18/5.11 | 4.42/6.27 | 3.13/4.98 | 64 / 0 / 0 / 0 |
| fast | tenant 1 / RW95 | 4,539 | 210 | 94.95/5.05% | 0.71/1.58/2.13 | 0.75/1.64/2.17 | 4.03/5.23 | 1.90/3.14 | 96 / 0 / 0 / 0 |
| baseline | tenant 2 / RW80 | 2,718 | 508 | 79.86/20.14% | 0.94/2.88/4.73 | 0.98/2.89/4.93 | 4.94/7.27 | 3.74/6.16 | 196 / 0 / 0 / 0 |
| fast | tenant 2 / RW80 | 3,493 | 653 | 79.87/20.13% | 0.68/1.55/2.00 | 0.71/1.62/2.03 | 4.31/5.69 | 3.45/4.66 | 251 / 0 / 0 / 0 |
| baseline | tenant 3 / RW50 | 1,954 | 912 | 49.81/50.19% | 0.91/2.68/4.39 | 0.99/2.95/4.93 | 5.78/8.38 | 5.27/8.00 | 337 / 9 / 0 / 0 |
| fast | tenant 3 / RW50 | 2,147 | 1,002 | 49.75/50.25% | 0.67/1.48/2.04 | 0.68/1.57/1.88 | 4.77/6.30 | 4.51/5.80 | 377 / 9 / 0 / 0 |
| baseline | tenant 4 / RW20 | 1,461 | 1,087 | 20.07/79.93% | 0.97/3.12/6.21 | 0.99/3.49/5.29 | 6.21/8.74 | 6.45/9.28 | 407 / 0 / 0 / 0 |
| fast | tenant 4 / RW20 | 1,547 | 1,151 | 20.03/79.97% | 0.60/1.52/1.80 | 0.64/1.39/1.78 | 4.93/6.65 | 4.99/6.63 | 430 / 0 / 0 / 0 |

The sum of per-tenant API/s was 9,200 baseline and 11,725 fast in this one run. This aggregate hides the distinct tenant mixes; per-tenant rates and tails are the comparison of record.

## CPU, RSS, and limits

CPU came from the workload process's `/proc/self/stat` tick delta and RSS from `/proc/self/status`. RSS was sampled every 200 ms during measurement and drain. All ten runs report complete sampling with 24 samples. CPU is normalized to one core and to the two-CPU host. Values are per process, including all four MT-MIX tenants.

| Run | CPU one core / host | Peak sampled RSS |
| --- | ---: | ---: |
| baseline RW95 | 100.2% / 50.1% | 32.4 MiB |
| fast RW95 | 106.3% / 53.2% | 38.8 MiB |
| baseline RW80 | 89.0% / 44.5% | 40.1 MiB |
| fast RW80 | 72.6% / 36.3% | 39.4 MiB |
| baseline RW50 | 58.0% / 29.0% | 40.3 MiB |
| fast RW50 | 51.5% / 25.8% | 40.4 MiB |
| baseline RW20 | 44.6% / 22.3% | 40.6 MiB |
| fast RW20 | 43.4% / 21.7% | 40.9 MiB |
| baseline MT-MIX | 77.2% / 38.6% | 59.7 MiB |
| fast MT-MIX | 78.0% / 39.0% | 64.2 MiB |

The driver's latency reservoir retains at most 20,000 samples per category. Counts, sample counts, per-run CPU/RSS details, timing, query rows, write categories, and independent validation are in each raw JSON report and the consolidated `results.csv`. Detailed stage metrics were disabled, so this comparison does not measure how many milliseconds semaphore wait changed. It also does not measure the individual path-check submission or distinguish permit holders.

## Interpretation and remaining limit

The test counter proves that warm Get, Query, and Scan no longer enter the path-existence branch; they avoid one extra blocking submission and its shared-permit acquisition. The single RW95 run showed API/s rising from 19,194 to 39,928 and Get/Query16 p95 falling from about 1.7 ms to about 0.6 ms. RW80 throughput was about 1.8% lower, RW50 about 0.7% higher, RW20 about 2.6% higher, and MT-MIX aggregate API/s about 27.4% higher. The direction and size vary by scenario, and there is one five-second sample per case; small differences do not establish stable improvement.

The change removes one unnecessary submission for each read to an already initialized tenant. It does not remove contention among the existing eight shared blocking slots. Writes still hold those slots while synchronous durable transactions run. These results do not establish that all read-tail causes are solved, rank RocksDB against B-link, or attribute a specific semaphore wait reduction.

One next investigation: measure per-submission wait contributions under the same workload mix to determine how much of the remaining read tail comes from waiting behind shared-slot write work.
