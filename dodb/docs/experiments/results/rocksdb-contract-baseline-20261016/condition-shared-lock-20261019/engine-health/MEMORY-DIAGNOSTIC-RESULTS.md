# Memory-Only Diagnostic Results

## Scope and provenance

This opt-in diagnostic ran the seven approved A/E paths once each with a 10-second warmup and 30-second measurement. Its loopback parent/worker transport changes execution overhead, so these results describe memory and residency observations only; they are not a substitute for the frozen performance results in [FIVE-MINUTE-RESULTS.md](FIVE-MINUTE-RESULTS.md).

The workload binaries and worker were built from clean source `843f0ea1598a4fa87284d1e3ee421612f288b4c2`. The `service_workload`, `unconditional_write_workload`, and `memory_diagnostic_worker` SHA-256 values are respectively `8814fd1b3ff71b80df8fc4f02f11df68c9b280321f58114cbedcfb99e96e4022`, `e0957ff46415ea3549ac50e9fc16ac1a2a4812ceede3cd0367715dc53ac7414d`, and `01e26347790d75992b47634df6c83eaa5cc5482c442dbbfb7795d17eca30717d`. The correctness release gate ran from source `c39db5ddd857ec807580aad143c81e3ee38e53a9`; it differs from `843f0ea15` only in nine test initializers and passed 63 tests. The two workload binaries were then rebuilt from the clean `843f0ea15` checkout and matched the frozen hashes above. The final measurement controller SHA-256 is `08b97c447b269d6560da332d9c22a85407074d00107b7e68f8f60782f6233dfa`.

The selected plan fingerprint is `cfae221e3d559ff09a038d548fe9a440b183cafb5409ce859ceac8498e14949a`. All RocksDB API paths used ReservedRanges and Shared condition-key locks. E's raw and Blink reports correctly mark DODB revisions/condition locks as not applicable; their command-line allocator/lock flags do not add those semantics. A core used 16 direct callers and a 16-permit driver semaphore. E core used 64 direct callers and no semaphore; the requested slot value of 32 is not applied there. E service used 32 shared admission slots. Raw RocksDB is the unconditional subset without DODB revisions, OCC, ABA, or condition checks.

The clean release gate evidence is in [gate-c39db5ddd-release-r2](memory-diagnostic/gate-c39db5ddd-release-r2/). The complete 7-case reports, commands, one-second process-group samples, health reports, file manifests, and preserved native `LOG` files are in [results-7x30-843f0ea15](memory-diagnostic/results-7x30-843f0ea15/); the machine-readable table is [resource-summary.json](memory-diagnostic/results-7x30-843f0ea15/resource-summary.json). The corrected plan-only snapshot is [plan-only.json](memory-diagnostic/results-7x30-843f0ea15/plan-only.json). All seven children exited successfully, accepted the full 30-second window, passed final/reopen validation and histogram checks, and had empty stderr. Each successful database was removed only after its size/hash manifest and native log content were preserved and checked.

## Process and host observations

Parent and backend figures are the maximum sampled RSS/PSS for their respective processes. Backend includes the adapter, service/runtime, and loopback transport; it is not a pure RocksDB allocator measurement. Group RSS/PSS uses the greater per-metric maximum from the controller live-budget samples and the independent backend process-memory samples. These are two independent one-second sample streams; the reported group value is the highest observed concurrent sample, not a sum of per-process peaks. The worker VmHWM is recorded after backend shutdown and provides a per-process lifetime high-water mark.

| Path | Parent RSS/PSS (MiB) | Backend RSS/PSS (MiB) | Backend VmHWM after shutdown (MiB) | Group RSS/PSS peak (MiB) | Minimum host MemAvailable (GiB) | Live database peak (GiB) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| A Blink | 133.2 / 130.1 | 246.0 / 243.1 | 245.8 | 372.0 / 365.9 | 4.26 | 0.61 |
| A RocksDB service Shared-16 | 143.8 / 140.7 | 241.6 / 238.5 | 245.4 | 330.4 / 324.2 | 5.06 | 0.13 |
| A synchronous RocksDB core Shared-16 | 145.5 / 142.4 | 240.0 / 236.9 | 242.2 | 334.2 / 328.0 | 5.03 | 0.14 |
| E raw RocksDB | 618.3 / 615.2 | 215.7 / 212.5 | 215.4 | 817.8 / 802.4 | 4.44 | 0.43 |
| E synchronous RocksDB core, 64 callers | 621.2 / 618.1 | 400.8 / 397.7 | 400.7 | 1,019.1 / 1,012.8 | 4.23 | 0.51 |
| E RocksDB service Shared-32 | 620.3 / 617.2 | 393.5 / 390.3 | 395.2 | 1,008.8 / 1,002.6 | 4.30 | 0.45 |
| E Blink | 621.5 / 618.4 | 916.3 / 913.4 | 916.1 | 1,536.2 / 1,530.1 | 3.25 | 1.42 |

The highest sampled group RSS was 1,536.2 MiB, below the 3 GiB cap. The lowest sampled host `MemAvailable` was 3.25 GiB, above the 1.5 GiB runtime reserve. The largest sampled database was 1.42 GiB, below the 12 GiB per-case cap. Group swap remained zero. The one-second group samples and per-process VmHWM are distinct observations; the group peak can miss a sub-second transient, while each process VmHWM captures its own lifetime high water.

## Dataset, cache, and residency observations

| Path | Final rows | Key bytes (MiB) | Value bytes (MiB) | Reopened native block-cache capacity / usage (MiB) | Reopened estimated live data / live SST size (MiB) |
| --- | ---: | ---: | ---: | ---: | ---: |
| A Blink | 118,742 | 3.17 | 57.98 | Unsupported | Unsupported |
| A RocksDB service Shared-16 | 126,458 | 3.37 | 61.75 | 32 / 32.00 | 66.40 / 66.40 |
| A synchronous RocksDB core Shared-16 | 127,485 | 3.40 | 62.25 | 32 / 31.99 | 66.98 / 66.98 |
| E raw RocksDB | 262,144 | 15.27 | 128.00 | 32 / 31.99 | 139.56 / 275.94 |
| E synchronous RocksDB core, 64 callers | 262,144 | 15.27 | 128.00 | 32 / 31.99 | 139.56 / 303.50 |
| E RocksDB service Shared-32 | 262,144 | 15.27 | 128.00 | 32 / 31.99 | 139.56 / 276.81 |
| E Blink | 262,144 | 15.27 | 128.00 | Unsupported | Unsupported |

The RocksDB property reports show a configured native block-cache capacity of 32 MiB per database and usage near that capacity. `estimate-live-data-size` and `live-sst-files-size` are separate native estimates; they are not interchangeable with the exact key/value totals. ZFS ARC `c_max` was about 5.32 GiB and the shared ARC `size` varied from 4.50 to 5.34 GiB over these runs. ARC is host-wide, not dedicated to a workload process, and must not be added to the RocksDB block-cache capacity as a guaranteed cache budget. These observations do not establish a complete effective-cache capacity or a general cache-to-dataset residency ratio.

Each case then made a first bounded full Scan to warm the cache and a second bounded full Scan after reopen. All seven second scans matched the reopened data digest and row oracle, kept the same backend PID/starttime and `/dev/sda4` mapping, and recorded zero physical vdev read delta and zero backend-process Swap at scan boundaries. This supports a RAM-served conclusion for these seven second scans only. It does not prove residency during the earlier performance measurements, nor does it establish that all pages of a larger dataset remain resident.

The A final payload domains were 57.98–62.25 MiB, compared with about 132 MiB in the five-minute A performance reports. The A comparison used the same seeded input region and workload settings, but these memory-only runs had different duration, completion count, and loopback transport. E retained exactly 262,144 rows and 128 MiB of values in every path. Do not treat these memory diagnostic row/value totals as replacements for the performance reports' measurements.

## Controller recovery record

The first controller stopped after a successful A Blink workload because its validator looked for source and binary hashes at the report root instead of `memory_only_diagnostic`. The next attempt similarly treated the reopened worker's intentional zero-interval health state as a missing one-second measurement sample; its shutdown property snapshot was present. These were report-validator assumptions, not workload failures. The original reports and failed controller source remain in the result directory. The controller was corrected to validate the actual report schema and lifecycle scopes, and the two already completed runs were revalidated without rerunning. See [recovery record](memory-diagnostic/VALIDATOR-RECOVERY.md). The approved seven-case measurement itself has one final completed run, plan fingerprint above, and no failure databases.

## Evidence limits

The parent/backend process split is now observed, but backend RSS still includes the server adapter, runtime, and transport; no pure native-engine RSS decomposition is available. The resident-scan result is limited to the tested second scan and does not retroactively classify the 21 five-minute cases. Memory-exceed measurements remain Deferred by user. The performance result sources remain the earlier frozen ee6 binaries, not this diagnostic source or its loopback execution.
