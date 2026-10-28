# Shared-resource RocksDB reference

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: opt-in shared-resource reference implemented; clean-source checks and four OCI correctness cells passed. This is not a density or performance result.

## Behavior and boundaries

The new `minimal-native-shared` route uses the existing independent TransactionDB adapter through the same density driver, diagnostic-worker transport, blocking adapter, in-flight limit, scheduler, preflight, and full-population/reopened oracles. It preserves reserved-range revisions, exclusive condition locks, synchronous WAL, disabled compression, and separate tenant databases. The historical `minimal-native` route and B-link behavior retain their defaults.

Each worker service owns one LRU cache and one WriteBufferManager shared by all its tenant databases. Memtable reservations are charged to that same cache; index/filter blocks use it as well. Native option snapshots verify the common pointers and actual per-tenant settings. Cache/WBM usage is counted once across tenants. WBM usage and its cache reservation are separate observations: do not add either to cache usage as if it were disjoint memory.

The driver requires all six resource options on this route and rejects them on other backends. It rejects missing, duplicate, invalid, or incompatible values before creating the data directory. Native resource configuration is opt-in through `MinimalNativeConfig.shared_resources`.

| Option | Runtime smoke setting | Scope |
| --- | --- | --- |
| `--rocksdb-shared-cache-bytes` | 8 MiB | One non-strict, single-shard LRU cache per service |
| `--rocksdb-shared-wbm-bytes` | 4 MiB | Shared soft flush threshold; writer stalls disabled |
| `--rocksdb-write-buffer-bytes` | 64 KiB | Per-tenant write buffer; two buffers configured |
| `--rocksdb-flush-threads` | 1 | Process-global Env::Default HIGH pool |
| `--rocksdb-compaction-threads` | 1 | Process-global Env::Default LOW pool |
| `--rocksdb-max-open-files` | 64 | Per-tenant file limit |

The shared route configures one subcompaction and a one-byte write-history target per tenant. It disables explicit L0/top-level index/filter pinning. Background-pool sizes are not a total thread limit: runtime, transport, application, and other native threads remain outside them. Multiple independently configured services in one process would share the Env::Default pools; the density worker uses one service per process epoch.

These cache/WBM settings are not a hard combined-memory or RSS budget. Pinned blocks, reservations, dirty/idle memtables, and retained history can exceed the soft settings. Other native allocations, request/runtime memory, OS cache, and ARC are outside them. A common enforced process/OS budget remains future work.

Native snapshots are taken at measurement start/end control commands and before shutdown. The end includes drain. They are non-atomic across native counters and are not periodic or peak instrumentation. Separate PID/start-time process samples retain the harness's sampling limits.

## Rejected stall-enabled pressure probes

`shared-probe01-check` and `shared-probe02-check` failed native compilation while adapting to the pinned cache API. `shared-probe03-check` failed two test assertions using the wrong RevisionState accessor. All logs and source archives are preserved; none is runtime performance evidence.

`shared-probe04-unit` stalled in a three-tenant pressure test with a 128 KiB WBM and TransactionDB's implicit 128 KiB write-history target. Its native log recorded 31 successful native writes and a WBM stall. `shared-probe05-unit`, using a one-byte history target, progressed through the first two tenants but stalled in the third. Its third-tenant log recorded a WBM-triggered flush and a WBM stall. The tests were terminated after preserving database copies and thread wait observations. These are rejected unit-test configurations on the boot filesystem, not accepted ZFS workload cells.

The pinned source confirms that TransactionDB replaces a zero history target with `max_write_buffer_number * write_buffer_size`; flushed-history retention and idle dirty memtables remain charged to WBM. The observed blocked writers had no subsequent native writes or flushes while sampled threads waited. This supports rejecting a stall-enabled tiny WBM for this adapter; it does not prove that every stall-enabled configuration or every RocksDB workload fails.

Accepted configuration keeps the one-byte history target and disables WBM writer stalls. The adapter never calls Transaction::SetSnapshot; condition checks retain exclusive GetForUpdate locking, and Query/Scan retain their DB snapshots. No RocksDB internals, transaction implementation, WAL policy, or storage format are changed.

Preserved remote probe databases are under `/home/opc/tenant-density-harness-20261027/shared-pressure-hang-probe04/database/` and `shared-pressure-hang-probe05/database/`. Original temporary directories, source archives, and logs are also retained. The probe overlay was stashed before switching to clean acceptance source; see the checkout provenance.

Relevant upstream source: [WBM behavior](https://github.com/facebook/rocksdb/wiki/Write-Buffer-Manager), [memory accounting](https://github.com/facebook/rocksdb/wiki/Memory-usage-in-RocksDB), and the pinned [TransactionDB preparation](https://github.com/facebook/rocksdb/blob/abeebd9630f11bd08c28b7bd43c7bdfc62050654/utilities/transactions/pessimistic_transaction_db.cc) and [write-history option](https://github.com/facebook/rocksdb/blob/abeebd9630f11bd08c28b7bd43c7bdfc62050654/include/rocksdb/advanced_options.h).

## Acceptance evidence

Accepted implementation source is clean commit `b67484d02faf8fdac3007a123f863c130ef906e5`. Later evidence/documentation commits do not change the frozen runtime provenance. Builds used Rust 1.97.1 and the existing read-only RocksDB 11.8.1 archive at commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654`. The native archive SHA-256 remained `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.

All evidence below is under `evidence/shared-rocksdb/oci/`. `shared-clean-checkout.json` records the probe stash `7fbff3ba7f6486335bd91d0568f01aa71a42af45` and clean checkout. `shared-artifacts-manifest.json` records preserved source-archive hashes and rejected database-copy file hashes. The remote build root, previous source/builds/databases, and pinned native archive were preserved.

| Clean-source check | Outcome | Evidence directory under `builds/` |
| --- | --- | --- |
| Shared-resource, pressure, lifetime, configuration and matching existing unit checks | 10 passed, including 4 new tests | `shared-accepted-unit/` |
| Existing process identity/residency accounting | 1 passed | `shared-accepted-session/` |
| Existing native exclusive-lock behavior | 1 passed | `shared-accepted-locking/` |
| Density and diagnostic worker binaries | 16 passed, including 1 new parser test; worker has no unit tests | `shared-accepted-bins/` |
| Read and transaction contracts | 5 read + 4 transaction checks passed, including 2 new shared-route checks | `shared-accepted-contracts/` |
| Advisory changed-target Clippy | Passed with unchanged findings; no new implementation diagnostics | `shared-accepted-clippy/`, `clippy-review.json` |
| A1 feature compatibility link | Passed; no A1 runtime claim | `shared-accepted-a1-link/` |
| Default-feature runtime link | Passed; these binaries were frozen for the real cells | `shared-accepted-default-link/` |

This is 37 passing tests, including 7 new tests. Package formatting and source/document whitespace checks passed locally. Raw `.log` evidence retains its original whitespace. The Clippy audit retains the same 16 package-library diagnostics as the prior baseline and one additional inspected diagnostic in an unchanged common test helper; it is not a warning-free workspace claim. No lint suppression was added.

The accepted clean-source pressure test interleaves 384 Put requests across three tenants with 4 KiB values, a 512 KiB cache, a 128 KiB WBM, and 64 KiB write buffers. It recorded 40 WBM-triggered flush events and 45 SST files before shutdown, then reopened and verified every value. The earlier passing dirty `shared-probe06-unit` recorded 36 such flush events; these are separate runs, and neither is performance evidence. The lifetime test reads from a held iterator after dropping its service and tenant, then releases it and reopens the database. This checks that native shared-pointer ownership survives those Rust owner drops.

## Real OCI correctness cells

Run namespace: `shared-correctness-b67484d02-r1`. All four cells ran once on `opc@217.142.246.204`, using unique preserved ZFS database directories under `/bench/zfs/db/tenant-density-harness-20261027/`. Frozen default-feature binaries and their hashes remain under the remote run's `bin/`; commands, raw reports, status, binary identities, and `acceptance-review.json` are copied to `smokes/shared-correctness-b67484d02-r1/` in this evidence tree.

All cells used seed 20261027, 17 registered tenants, 256 seeded rows per tenant, 512-byte values, mixed write widths, queue bound 8, common in-flight limit 8, a 5-second request timeout, and a 15-second drain timeout. Fixed-total used four streams, the mixed profile, and 1,700 requests. Fixed-per-tenant used one lane per tenant, the write-heavy profile, and 10 arrivals/s per active tenant for three seconds. Setup, measurement, full-population oracle, and reopened oracle each use a separate backend process.

| Route and topology | Active / opened-idle / never opened in measurement epoch | Completed requests | Seed rows + extra expected keys verified in each oracle |
| --- | --- | --- | --- |
| Existing minimal-native, fixed-total | 17 / 0 / 0 | 1,700 | 4,352 + 596 |
| Existing minimal-native, fixed-per-tenant | 3 / 2 / 12 | 90 | 4,352 + 51 |
| Shared minimal-native, fixed-total | 17 / 0 / 0 | 1,700 | 4,352 + 596 |
| Shared minimal-native, fixed-per-tenant | 3 / 2 / 12 | 90 | 4,352 + 51 |

All 3,580 requests succeeded with no conflicts, errors, timeouts, drops, drain completions, or final backlog. Each pair matched recipe fingerprints, actual admitted-request/schedule fingerprints, request counts, per-tenant fingerprints, and per-tenant operation/transaction/width counts. Histogram/accounting identities and both full-population oracles passed. Extra expected keys include preflight and intended missing states; they are not a distinct-successful-mutation count.

Native snapshots verified all 17 opened measurement tenants in the all-active shared cell and all five in the sparse cell, plus all 17 tenants in each setup/oracle epoch. Actual cache capacity, WBM threshold, and HIGH/LOW pool sizes matched the requested configuration. Example measurement-end observations, counted once per shared service:

| Shared cell | Cache usage including WBM reservation | WBM reservation already charged to cache | Actual WBM memtable usage |
| --- | --- | --- | --- |
| All-active | 2,832,144 bytes | 1,048,576 bytes | 813,816 bytes |
| Sparse-active | 429,568 bytes | 262,144 bytes | 108,640 bytes |

These are boundary observations, not peaks or comparative memory advantages. End snapshots include drain and may race with background maintenance. Process sampling retained matching backend identity, no truncation, one periodic offered-load sample per all-active cell and three per sparse cell. Sampling gaps remain explicit in the reports. All recorded driver/backend processes had exited at review.

The minimum observed benchmark-filesystem free space was 11,074,265,088 bytes, above the 10 GiB reserve. This still leaves too little room to size a broad density/pressure matrix. Build outputs stayed on the boot filesystem. No successful cell was rerun. The first reduction script incorrectly assumed fixed-per-tenant reports had top-level operation totals; the preserved rejected script records that reporting-only error. The corrected review compares the actual per-tenant operation/transaction/width fields without rerunning workloads.

This acceptance covers clean shutdown/reopen correctness. It adds no crash-recovery claim, real 512-tenant run, 4 KiB runtime density cell, sustained pressure matrix, capacity boundary, SLO pass, or maximum-RocksDB result.

## Continuation

The reference prepares a configurable shared-resource comparator. Retained-heap/pin-age/queue-snapshot byte accounting, common enforced memory pressure, live host sizing, a frozen resource-pressure matrix, E2/E3 density boundaries, and E4 maintenance stability remain incomplete. Issue #126 stays open; no default is promoted and no adoption decision follows from tiny correctness cells.
