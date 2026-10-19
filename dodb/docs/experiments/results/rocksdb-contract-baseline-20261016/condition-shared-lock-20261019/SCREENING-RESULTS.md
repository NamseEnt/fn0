# Condition-Key Shared-Lock Screening Results

## Decision status

The opt-in shared-lock candidate passed its required deterministic and contract gates and completed the approved 44-case short screen. The screen favors Shared in the tested write-heavy workloads, especially `rw80`, `rw50`, and `rw20`. This is preliminary, workload-specific evidence. The 30-second confirmation and five-minute comparison remain pending, and the production default remains Exclusive.

The workload used immutable condition-only keys, so it cannot establish writer fairness. The separate diagnostic run measured lock wait while holders competed on condition-only keys; it does not establish fairness under a changing condition-key workload. The single five-second diagnostic repetition per mode is an attribution observation, not a stable distribution.

## Source, binaries, and host

- Source SHA: `88b43098dab8736363d8c36446bbede27e1b77e1`
- Source tree: `0e8d1da6c60fd83fb11fd3d6f1a643665e3079a2`
- Worktree: detached and clean at `/bench/zfs/db/condition-shared-lock-20261019/source-88b43098d`
- `service_workload` SHA-256 after the release correctness gate: `ff659fb3f0ea5ce8998bec1fa4297b9ec16b706f8553ba3555394a21a8db6977`
- `unconditional_write_workload` SHA-256 after the release correctness gate: `6d1913ba3da2ec87b16c8c9c62955afe76f586ecec22812858c81ba2a64d3890`
- Host: `instance-20260923-1013`, ARM64, 2 online CPUs, ZFS `/bench/zfs/db`; preflight available memory was 5,047,584 KiB and available ZFS space was 76,266,508,288 bytes.
- Toolchain and native dependency: Rust/Cargo 1.97.1, `aarch64-unknown-linux-gnu`, pinned RocksDB 11.8.1. The existing OCI helper and shared release cache were used; no new native archive was built.
- Runner SHA-256: `d708bbd162d0cc87cbfdff910e8c569c6d4e20c707e2637f498361e8d8a0353b`.

The final release source build completed with helper exit 0. The 35-test release gate passed at source `15e0077b4e648930c7e01def84317b566f4e18e3`: lib 27, history 1, process-crash 4, read 2, and transaction 1. Between that source and the screening source, production code did not change: the later changes scope a test helper to test builds and add present/missing condition-only keys to the crash scenario. The final-source release crash gate then passed 4/4 tests with exit 0. This final crash scenario exercises Shared condition locks on an existing untouched key and a missing key while preserving the existing atomic-recovery oracle.

Build and gate logs, environment captures, per-case commands, reports, manifests, and copied RocksDB logs are in [the preserved OCI evidence](oci-evidence-88b43098d/). The screen preflight records the final source and binary hashes, host, available memory, ZFS mount and capacity, and zero active build/workload processes.

The final-source release build and crash-gate commands, from the `dodb` directory, were:

```sh
DODB_BUILD_RUN_ID=condition-shared-lock-20261019-release-final RUST_TEST_THREADS=1 bash scripts/run-oci-rocksdb-build.sh build --release --locked -p dodb-rocksdb --bin service_workload --bin unconditional_write_workload
DODB_BUILD_RUN_ID=condition-shared-lock-20261019-crash-final RUST_TEST_THREADS=1 bash scripts/run-oci-rocksdb-build.sh test --release --locked -p dodb-rocksdb --test process_crash_contract
python3 /bench/zfs/db/condition-shared-lock-20261019/run-condition-lock-ab-88b43098d.py ff659fb3f0ea5ce8998bec1fa4297b9ec16b706f8553ba3555394a21a8db6977
```

## Workload and validation

The serial matrix contained 42 performance cases and two diagnostic cases. At 16 clients, it covered `rw95`, `rw80`, `rw50`, `rw20`, and `mt-mix`; at 32 clients, it covered `rw20` and `mt-mix`. Every profile compared Exclusive and Shared with three repetitions, rotating mode/profile order. The two diagnostic cases were one `rw20`, 16-client run per mode.

Both modes used the same 10,000-row seed, 512-byte values, 1,024 overwrite slots per client, widths 1/4/8 with width-16 workloads enabled, `log1024` latency storage, 100-million-call cap, five-million-entry write-ledger cap, full-window requirement, one-second warmup, and five-second measurement. RocksDB used the ReservedRanges allocator with a 4,096-revision range; blocking admission was Shared-16 for c16 and Shared-32 for c32. The runner checked that width-16 workloads were attempted. Normal performance cases kept detailed RocksDB research metrics disabled. Only the two final diagnostic cases enabled them.

The runner validated mode/configuration reporting, matching per-client input prefixes, acknowledgment and revision identity, success/reject/conflict/error counts, latency sample counts, attempted width-16 workloads, full measurement windows, process CPU/RSS samples, final-state oracle, reopen oracle, and isolated database size. All 42 performance and two diagnostic cases passed. All 44 successful databases were removed only after validation and after their file/LOG manifests were written. The evidence contains 44 reports, 44 database file manifests covering 883 database files, 160 copied and hash-verified RocksDB LOG files, and no remaining database directories. Recorded database files totaled 3,509,842,852 bytes; the largest case database was 122,975,430 bytes. The local post-copy audit is [post-run-verification.json](oci-evidence-88b43098d/post-run-verification.json).

## Results

Each cell reports the median of three five-second repetitions and the observed minimum–maximum across those repetitions. Workflow throughput is `(api_calls - preparatory_gets) / measurement_seconds`; each preparatory Get is a separate API call and is excluded from the workflow count. Successful-write throughput is reported separately as committed transactions per second. Workflow and successful-write p99 values are medians of the three reported histogram upper-bound estimates. CPU seconds and peak sampled RSS are medians over the measurement interval. Times are milliseconds; RSS is KiB.

| Clients | Profile | Mode | Workflows/s, median [min, max] | Successful writes/s, median [min, max] | Workflow p99, median [min, max] | Write p99, median [min, max] | CPU seconds, median [min, max] | Peak RSS KiB, median [min, max] |
| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | rw95 | Exclusive | 45,432 [43,664, 45,765] | 2,104 [2,026, 2,124] | 5.21 [5.17, 5.70] | 55.90 [52.89, 64.91] | 5.69 [5.48, 5.69] | 44,676 [44,400, 45,044] |
| 16 | rw95 | Shared | 48,415 [47,824, 50,513] | 2,240 [2,218, 2,342] | 4.53 [3.76, 5.17] | 10.49 [9.59, 12.17] | 5.99 [5.98, 6.02] | 46,768 [45,596, 47,228] |
| 16 | rw80 | Exclusive | 13,943 [13,903, 14,470] | 2,665 [2,658, 2,771] | 27.13 [25.56, 27.18] | 57.34 [56.56, 60.33] | 3.54 [3.44, 3.55] | 46,720 [46,376, 48,004] |
| 16 | rw80 | Shared | 28,210 [27,891, 28,564] | 5,431 [5,371, 5,491] | 4.03 [4.02, 4.31] | 5.42 [5.32, 5.94] | 5.07 [4.94, 5.16] | 73,004 [72,660, 73,332] |
| 16 | rw50 | Exclusive | 5,628 [5,434, 5,648] | 2,877 [2,784, 2,884] | 44.47 [43.19, 46.86] | 58.13 [55.18, 60.16] | 2.68 [2.56, 2.76] | 49,604 [49,488, 49,640] |
| 16 | rw50 | Shared | 10,848 [10,716, 10,881] | 5,566 [5,496, 5,582] | 5.05 [4.80, 5.23] | 5.60 [5.29, 5.75] | 4.24 [4.07, 4.24] | 76,568 [75,868, 76,752] |
| 16 | rw20 | Exclusive | 3,320 [3,311, 3,374] | 2,872 [2,863, 2,917] | 53.08 [52.72, 55.28] | 56.79 [56.72, 57.05] | 2.41 [2.36, 2.42] | 50,704 [50,492, 52,512] |
| 16 | rw20 | Shared | 6,894 [6,807, 6,991] | 5,940 [5,870, 6,031] | 5.25 [4.92, 5.32] | 5.33 [4.98, 5.40] | 3.67 [3.66, 3.74] | 81,024 [80,580, 81,908] |
| 16 | mt-mix | Exclusive | 19,896 [18,506, 26,348] | 3,160 [2,708, 3,904] | 8.21 [7.76, 11.57] | 13.84 [12.32, 19.20] | 5.02 [4.89, 5.08] | 77,672 [74,388, 84,364] |
| 16 | mt-mix | Shared | 26,949 [18,694, 27,668] | 3,958 [3,074, 4,027] | 7.82 [7.71, 8.28] | 11.85 [11.51, 14.37] | 5.11 [5.06, 5.16] | 85,108 [75,800, 86,232] |
| 32 | rw20 | Exclusive | 3,095 [3,082, 3,130] | 2,637 [2,624, 2,670] | 150.99 [149.95, 156.11] | 159.65 [159.65, 166.59] | 2.54 [2.54, 2.56] | 51,276 [51,224, 52,092] |
| 32 | rw20 | Shared | 7,016 [6,969, 7,057] | 6,059 [6,014, 6,093] | 11.94 [11.92, 15.05] | 12.53 [12.26, 15.79] | 4.56 [4.41, 4.58] | 97,160 [96,796, 97,992] |
| 32 | mt-mix | Exclusive | 28,059 [27,712, 28,195] | 3,996 [3,915, 4,179] | 15.86 [14.71, 16.15] | 40.76 [32.60, 42.60] | 5.50 [5.49, 5.51] | 88,744 [88,200, 90,928] |
| 32 | mt-mix | Shared | 28,943 [27,971, 29,707] | 4,495 [4,441, 4,511] | 13.07 [12.73, 13.50] | 18.30 [17.96, 18.35] | 5.75 [5.60, 5.77] | 95,132 [93,692, 95,568] |

The two diagnostic runs found mean condition-only native lock wait of 3.949 ms over 12,442 observations in Exclusive and 0 ms over 25,971 observations in Shared. Their approximate p99 upper bounds were 67.109 ms and 0.000001 ms. User commit duration was 1.999 ms mean / 4.194 ms p99 in Exclusive and 2.354 ms mean / 8.389 ms p99 in Shared. Each mode has one short diagnostic repetition, so the commit-tail difference requires confirmation. The condition-only interpretation is specific to this workload: its condition/missing ordinals are below 100,000 while mutations begin at `1 << 60`, so the sets do not overlap. The instrumentation's generic condition-key role can include condition-plus-mutation keys in other workloads.

By median workflow throughput, Shared was about 1.07×, 2.02×, 1.93×, 2.08×, and 1.35× Exclusive for c16 `rw95`, `rw80`, `rw50`, `rw20`, and `mt-mix`, respectively; it was 2.27× for c32 `rw20` and 1.03× for c32 `mt-mix`. Successful-write throughput is a separate metric; its Shared/Exclusive ratios are 1.06×, 2.04×, 1.93×, 2.07×, 1.25×, 2.30×, and 1.12× in the same order. Shared runs also visited more work and recorded higher peak RSS in write-heavy profiles. These screen results do not isolate memory cost per lock or prove that lock mode alone caused every observed difference.

## Next step

Complete the separately scoped engine-health observation harness gate before using the approved 30-second repetitions and representative five-minute comparison to test the screen's write-heavy result and mixed-load behavior. Keep Exclusive as the default until those measurements and the separate adoption decision are complete. Do not infer writer priority, starvation freedom, or fairness from these immutable-condition workloads.
