# Tenant-density harness preparation

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: implementation and focused checks complete; clean-source link builds and real Linux smoke acceptance pending. This is not a passing performance experiment record.

## Decision and scope

This increment prepares a separate `tenant_density_workload` driver and backend-process accounting. It does not establish an active-tenant capacity, a service-objective boundary, a common enforced memory budget, or a B-link advantage over RocksDB. The shared-cache/WriteBufferManager minimal-native reference and the E2/E3/E4 performance matrix remain future work.

The historical `service_workload` source is preserved. Both new comparator routes use the same loopback QUIC/TLS transport and frontend in-flight limit. Minimal-native executes its synchronous TransactionDB service through the Tokio blocking pool. Its existing reserved-range revisions, exclusive condition locking, synchronous WAL, disabled compression, and independent tenant databases remain in effect. This is the current contract baseline, not the future shared-resource reference or maximum-RocksDB result.

The diagnostic worker now accepts explicit B-link WAL layout, segment size, checkpoint threshold, and checkpoint execution settings. Omitted settings retain legacy WAL, disabled automatic checkpoints, and synchronous execution. The A1 writer-pin reuse Cargo feature remains disabled by default and is not promoted by this increment.

## Workload and correctness boundaries

Registered tenants are the seeded on-disk population, not a registration API. Setup, measurement, full-population oracle, and reopened oracle run in separate backend processes. Only active and explicitly opened-idle tenants are opened during the measurement epoch. Full-population validation occurs after that epoch and must not be included in its resource accounting.

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

## Validation

| Check | Status | Evidence |
| --- | --- | --- |
| Local filesystem/process accounting tests | Passed, 4 tests | `evidence/harness/local-resource-tests.log` |
| Default-feature new binary and worker tests | Passed, 15 workload tests; worker has no unit tests | `evidence/harness/oci/builds/worker19-final-bin-tests/` |
| A1 feature compatibility | Typecheck passed; clean-source link build pending | `evidence/harness/oci/builds/worker15-a1-feature-check/` |
| Existing affected contract/session checks | Passed, 1 memory diagnostic + 1 native locking + 3 transaction + 4 read tests | `evidence/harness/oci/builds/worker11-memory-diagnostic/` through `worker14-read-contract/` |
| Formatting and changed-target Clippy audit | Package formatting passed; advisory Clippy passed with existing dependency/library findings and no new binary findings | `evidence/harness/local-package-format.log`, `evidence/harness/oci/builds/worker18-advisory-clippy/` |
| B-link and minimal-native, 17 all-active tenants | Pending | Tiny fixed-total correctness smoke only |
| B-link and minimal-native, 17 registered / 3 active / 2 opened-idle | Pending | Tiny fixed-per-tenant correctness smoke only |

Preserved rejected attempts include `probe01-typecheck`, `probe02-typecheck`, and `worker04/05/06/08/09-typecheck`. They failed compilation or harness unit tests and are not database-performance observations. `worker03-typecheck` has no completion status and is not counted as passing. `worker07` and `worker10` are earlier passing checkpoints; final source is frozen in `sources/worker17-source.tar.gz` with SHA-256 `dfa3bf29d71e2134fa9db2b59a6483c11e9f30b2918e407beb1897caea4cd5f3` and passes `worker19-final-bin-tests`. The archive and member hashes are preserved in `evidence/harness/oci/source-snapshot-manifest-final-worker.json`. The initial clean A1 worker prewarm build passed; it does not validate this new harness.

## Continuation

Next prepare the explicit shared-cache/WBM contract-equivalent RocksDB reference. Then complete the missing retained-memory and pressure instrumentation, recheck host capacity, freeze a safe matrix, and run the density/maintenance experiments. Keep the current harness's loopback transport/runtime costs, feature selection, admitted/completed input differences, sampling gaps, and unsupported budgets explicit. Issue #126 remains open.
