# Tenant-density harness preparation

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: executable harness preparation, focused checks, clean-source link builds, and four Linux correctness/accounting smokes complete. This is not a performance or density experiment result.

## Decision and scope

This increment prepares a separate `tenant_density_workload` driver and backend-process accounting. It does not establish an active-tenant capacity, a service-objective boundary, a common enforced memory budget, or a B-link advantage over RocksDB. The shared-cache/WriteBufferManager minimal-native reference and the E2/E3/E4 performance matrix remain future work.

The historical `service_workload` source is preserved. Both new comparator routes use the same loopback QUIC/TLS transport and frontend in-flight limit. Minimal-native executes its synchronous TransactionDB service through the Tokio blocking pool. Its existing reserved-range revisions, exclusive condition locking, synchronous WAL, disabled compression, and independent tenant databases remain in effect. This is the current contract baseline, not the future shared-resource reference or maximum-RocksDB result.

The diagnostic worker now accepts explicit B-link WAL layout, segment size, checkpoint threshold, and checkpoint execution settings. Omitted settings retain legacy WAL, disabled automatic checkpoints, and synchronous execution. The A1 writer-pin reuse Cargo feature remains disabled by default and is not promoted by this increment.

## Workload and correctness boundaries

Registered tenants are the seeded on-disk population, not a registration API. Setup, measurement, full-population oracle, and reopened oracle run in separate backend processes. Only active and explicitly opened-idle tenants are opened during the measurement epoch. Full-population validation occurs after that epoch and must not be included in its resource accounting.

The oracle covers every seeded row, every acknowledged mutation key, and the preflight keys across all registered tenants. It does not enumerate arbitrary keys outside those sets. Reopen checks use clean worker shutdown; this increment adds no new crash-recovery claim.

Get/Query/write profile weights are 80/15/5, 40/20/40, and 10/10/80. Keys use 16-byte PK and SK. Each tenant's initial partition has at least 256 rows; Query reads its first 16 rows and validates ordered keys and full deterministic values. Get selects deterministic seeded rows. This is a synthetic workload, and Query does not rotate its range across the partition. Seeded data stays separate from bounded mutation lanes. The separate correctness preflight checks intended conditional rejection, revision progression, multi-key rejection atomicity, and delete/reinsert behavior; it is outside the offered-load window.

Fixed-total mode uses concurrent sequential client streams that multiplex active tenants. Fixed-per-tenant mode schedules absolute arrival deadlines independently of completion and uses bounded nonblocking admission queues. Scheduled-to-completion latency includes queueing. Drops, errors, timeout uncertainty, final backlog, and drain work must remain visible; these must not be interpreted as successful capacity.

Latency summaries use 64 logarithmic power-of-two nanosecond buckets and retain every completed outcome, with exact counts and maxima. Their coarse bucket bounds are not exact percentiles. A later SLO study must account for that resolution before interpreting thresholds inside a bucket. Actual admitted-request fingerprints and counts are separate from the input/schedule recipe fingerprints; equal recipes alone do not establish equal executed workloads.

## Accounting and unsupported metrics

Backend and driver memory are separated by PID/start-time identity. Summaries use offered-load samples and boundaries, excluding setup and oracle processes. RSS/PSS/anonymous/swap observations are samples; peaks between samples can be missed. Process lifetime VmHWM also includes worker startup, tenant opening, and correctness preflight. Missing PSS or other fields remain unavailable rather than becoming zero. Short windows with no periodic sample are labeled boundary-only.

Filesystem snapshots distinguish logical file lengths, allocated regular-file blocks, and allocated directory blocks. Hardlinks are deduplicated by device/inode; symlinks are refused. These are reachable-file boundary observations, not continuous peaks, ZFS dataset usage, unlinked-open-file accounting, or physical write amplification. Raw Linux process I/O includes runtime/transport costs and delayed buffered writeback. Host device counters include unrelated I/O.

Allocator retained heap, representation-specific retained bytes, production pin ages, snapshot/queue byte budgets, and an enforced common memory budget are not implemented. Normalized checkpoint image bytes must retain their normalized-image label. B-link cache capacity is not a residency bound. No memory-budget argument claims enforcement.

## Host and build provenance

The designated host is `opc@217.142.246.204`, Linux aarch64, Rust 1.97.1. Native RocksDB is pinned at `abeebd9630f11bd08c28b7bd43c7bdfc62050654` in `/bench/zfs/db/product-build/rocksdb-source`. Its archive is used read-only. Local macOS package execution was blocked by the missing pinned native build; Linux compile/unit evidence substitutes for that package execution. Four standalone local resource-accounting tests passed and remain separate evidence.

The task-owned build checkout, target, temporary files, and logs live under `/home/opc/tenant-density-harness-20261027` on the boot filesystem. The existing source/build artifacts on ZFS are preserved. The checkout excludes historical documentation during OCI builds to avoid copying more than 1 GiB of old evidence. Real smoke databases use unique directories under `/bench/zfs/db/tenant-density-harness-20261027` and are preserved.

The initial live capacity check found about 10.39 GiB free on the benchmark filesystem, only about 0.39 GiB above the preserved 10 GiB reserve. Build and smoke controllers enforce that reserve and check available RAM. This permits only tiny correctness/accounting cells in this increment; it does not size a density matrix.

`oci_harness_build.py` records source state, native archive hash, toolchain, command, environment, capacity, raw output, and exit status. Dirty compile probes are explicitly probes. Accepted real smoke binaries must come from a clean committed source and are frozen with hashes beside each run. `oci_harness_smoke.py` checks report validity and both oracles in addition to the process exit code. It stops on a failed cell and retains that cell's database and evidence.

## Accepted source and real smoke

Accepted source is clean commit `99932ec33419214a7ee1cc542e937d31ef733a7c`, following implementation commit `f0e602ce5e0fb53a8b601aaa4e4338c58f92811b`. Final review found that the preflight response helper incorrectly rejected a successful Delete response. The small follow-up commit fixes that response classification before any real smoke was launched. The earlier clean link builds at `f0e602ce5` are preserved as superseded builds; no database cell used them.

The previous dirty probe checkout, including incidental archive metadata files, is preserved in remote stash `a9aa4eec7d1701bf8817a8db11f2640e3542aaa2` and the source snapshot archives. The clean checkout transitions are recorded in `evidence/harness/oci/clean-acceptance-checkout.json` and `preflight-fix-checkout.json`. Neither the historical source checkout nor the shared native archive was changed.

Run namespace: `correctness-99932ec33-r1`. Raw commands, status, reports, binary hashes, and the post-run review are in `evidence/harness/oci/smokes/correctness-99932ec33-r1/`. Frozen binaries remain beside the remote reports under the same run namespace; all four databases remain on ZFS. No cell was rerun.

All cells used seed 20261027, 17 registered tenants, 256 initial rows per tenant, 512-byte values, mixed write widths, queue bound 8, common in-flight cap 8, a 5-second request timeout, and a 15-second drain timeout. Fixed-total used four client streams and 1,700 operations with the mixed profile. Fixed-per-tenant used the write-heavy profile and 10 arrivals/s per tenant for 3 seconds. B-link explicitly used segmented coalesced bases, 8,192-byte WAL segments, a 16,384-byte checkpoint trigger, and background execution. These smokes used default Cargo features; A1 pin reuse remained disabled.

| Cell | Active / opened-idle / never-opened in measurement epoch | Completed requests | Seed rows + extra expected keys verified in each of two oracle epochs | Completed checkpoints in measurement-worker lifetime |
| --- | --- | --- | --- | --- |
| B-link, fixed-total | 17 / 0 / 0 | 1,700; 100 per tenant | 4,352 + 596 | 58 |
| Minimal-native, fixed-total | 17 / 0 / 0 | 1,700; 100 per tenant | 4,352 + 596 | Not applicable |
| Minimal-native, fixed-per-tenant | 3 / 2 / 12 | 90; 30 per active tenant | 4,352 + 51 | Not applicable |
| B-link, fixed-per-tenant | 3 / 2 / 12 | 90; 30 per active tenant | 4,352 + 51 | 7 |

Every request completed successfully, with zero drops, conflicts, other errors, timeouts, or final backlog. All admitted work finished within the offer window in these low-load cells. The two B-link workers completed background checkpoints without failures or remaining tasks. Those checkpoint counts include preflight, drain, and shutdown work and are not offered-window-only counts. The raw oracle field `verified_mutated_keys` counts the expected extra-key map, including the three preflight keys and intended missing states; it is not a count of distinct keys changed by successful workload writes alone.

Both backend pairs have identical input/schedule recipe fingerprints, actual admitted-request fingerprints, scheduled/admission fingerprints, request counts, and per-tenant request fingerprints. All accounting identities and total/per-kind/per-tenant latency observation counts passed. `acceptance-review.json` records the comparison without using throughput or latency as a ranking.

Every offer boundary retained the same measurement backend PID/start-time identity. All-active cells each retained one periodic offer sample and sparse-active cells each retained three, in addition to boundary observations. None were truncated. This is sparse sampled evidence, not continuous peak memory. Every recorded backend and driver process had stopped at post-run review. The minimum observed benchmark-filesystem free space was 11,115,614,208 bytes, above the 10 GiB reserve.

Each cell ran once. There was no real 512-tenant run, no real 4 KiB-value cell, no memory-pressure matrix, and no SLO/capacity acceptance. The parser covers 512 tenants and 4 KiB values; the read-heavy schedule is unit-tested, while the real cells exercised mixed and write-heavy profiles. The A1 feature combination was link-built but was not exercised by these runtime cells.

## Validation

| Check | Status | Evidence |
| --- | --- | --- |
| Local filesystem/process accounting tests | Passed, 4 tests | `evidence/harness/local-resource-tests.log` |
| Default-feature new binary and worker tests | Passed, 15 workload tests on accepted clean source; worker has no unit tests | `evidence/harness/oci/builds/accepted-default-bin-tests/` |
| Default-feature runtime link build | Passed on accepted clean source | `evidence/harness/oci/builds/accepted-default-link/` |
| A1 feature compatibility | Typecheck and clean-source link build passed | `evidence/harness/oci/builds/worker15-a1-feature-check/`, `accepted-a1-feature-link/` |
| Existing affected contract/session checks | Passed, 1 memory diagnostic + 1 native locking + 3 transaction + 4 read tests | `evidence/harness/oci/builds/worker11-memory-diagnostic/` through `worker14-read-contract/` |
| Formatting and changed-target Clippy audit | Package formatting passed; advisory Clippy passed on accepted source with existing dependency/library findings and no new binary findings | `evidence/harness/local-final-package-format.log`, `evidence/harness/oci/builds/accepted-advisory-clippy/` |
| B-link and minimal-native, 17 all-active tenants | Both passed | Tiny fixed-total correctness smoke only |
| B-link and minimal-native, 17 registered / 3 active / 2 opened-idle | Both passed | Tiny fixed-per-tenant correctness smoke only |

Preserved rejected attempts include `probe01-typecheck`, `probe02-typecheck`, and `worker04/05/06/08/09-typecheck`. They failed compilation or harness unit tests and are not database-performance observations. `worker03-typecheck` has no completion status and is not counted as passing. `worker07` and `worker10` are earlier passing checkpoints; the final worker-owned snapshot is frozen in `sources/worker17-source.tar.gz` with SHA-256 `dfa3bf29d71e2134fa9db2b59a6483c11e9f30b2918e407beb1897caea4cd5f3` and passes `worker19-final-bin-tests`. The archive and member hashes are preserved in `evidence/harness/oci/source-snapshot-manifest-final-worker.json`. The initial clean A1 worker prewarm build passed; it does not validate this new harness.

The worker snapshot predates the small Delete-response follow-up; accepted runtime source and final unit checks use `99932ec33` as recorded above. Raw test logs retain their original trailing blank lines. Source/document whitespace checks exclude raw `.log` artifacts rather than rewriting the evidence.

## Continuation

Next prepare the explicit shared-cache/WBM contract-equivalent RocksDB reference. Then complete the missing retained-memory and pressure instrumentation, recheck host capacity, freeze a safe matrix, and run the density/maintenance experiments. Keep the current harness's loopback transport/runtime costs, feature selection, admitted/completed input differences, sampling gaps, and unsupported budgets explicit. Issue #126 remains open.
