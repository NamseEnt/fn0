# Shared-resource RocksDB reference

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

This plan is recorded before implementation. Work stays in the existing tenant-density worktree and branch. The primary executes this increment without delegation.

## Scope

Add an opt-in `minimal-native-shared` density/diagnostic-worker route using the existing independent TransactionDB adapter. Preserve reserved-range revisions, exclusive condition locks, synchronous WAL, disabled compression, read/transaction contracts, and historical routes/defaults. No B-link changes or E2/E3/E4 performance matrix are included.

One service owns one shared LRU cache and one WriteBufferManager across all its tenant databases. Charge memtable reservations to that cache, cache index/filter blocks, report non-strict cache capacity and soft WBM flush thresholds accurately. The initial stall-enabled proposal was rejected after the pressure probes documented below; accepted configuration disables WBM stalls. Neither is a hard process/RSS/OS-cache budget. Native handles must retain the resource owner until database destruction, including iterator lifetime and service shutdown.

Require explicit cache bytes, WBM bytes, per-tenant write-buffer bytes, flush/compaction thread counts, and per-tenant open-file limits on the new route. Reject incomplete, invalid, duplicate, or incompatible resource options before database creation. Configure and report process-global Env::Default high/low pools, per-DB background limits, and one subcompaction. The worker isolates each backend epoch; other diagnostic routes retain their existing behavior. These background pools do not cap application/Tokio threads or all native threads.

Add native resource snapshots with global usage counted once, configured limits, actual pool sizes, opened-tenant counts, and verification that tenant options reference the same cache/WBM. Expose measurement start/end and pre-shutdown snapshots; do not sum per-tenant block-cache properties into a false global usage figure. Keep sampled process accounting separate.

## Reuse and validation

Reuse the native transaction shim, MinimalNativeService, existing worker transport, density scheduler/oracle, and OCI build controller. Add a small shared-resource module and focused contract/lifetime/accounting checks; no new dependencies or lint suppressions. Repository artifacts remain English; add no source comments or Co-Authored-By trailers.

Run formatting, targeted unit and read/transaction/session checks, changed-target Clippy review, and default/A1 compatibility builds. Use the pinned RocksDB 11.8.1 archive read-only on `opc@217.142.246.204`. Preserve prior logs/databases and failed attempts. Real runtime acceptance uses clean committed source and frozen binary hashes.

The initial live check found 11,115,610,112 benchmark-filesystem bytes free, 2,443,997,184 boot-filesystem bytes free, and about 6.7 GB available RAM. Preserve at least 10 GiB benchmark free and the existing build RAM/boot reserves. Run only tiny correctness cells: 17 all-active fixed-total and 17 registered / 3 active / 2 opened-idle fixed-per-tenant for the shared route, with matching old-reference controls if space permits. Retain full-population and reopened oracles, fingerprints, configuration, native snapshots, and raw outcomes. Pressure behavior may be checked in a bounded unit/integration test; these cells do not establish density, SLO, or a common enforced process budget.

## Completion

The first bounded pressure test exposed a non-progressing WBM stall with TransactionDB's implicit `max_write_buffer_size_to_maintain = max_write_buffer_number * write_buffer_size`. Preserve that attempt and its database. A second probe with a one-byte history target progressed through two tenants but stalled in the third. Preserve both failed databases. The accepted shared route explicitly uses a one-byte history target and disables WBM writer stalls, so the WBM requests flushes without blocking progress on retained or idle memtables. This adapter never calls Transaction::SetSnapshot and retains exclusive GetForUpdate locking; Query/Scan DB snapshots remain unchanged. Revalidate all shared read/transaction contracts and an interleaved three-tenant pressure test proving actual WBM-triggered flushes and clean reopen before acceptance. Report threshold overshoot rather than claiming enforcement. A later density matrix still needs independent common process/OS memory enforcement. This is an explicit shared-reference configuration, not a change to the old comparator or a guarantee that arbitrary tiny global WBM settings progress with many idle dirty databases.

Record exact outcomes and limitations in SHARED-ROCKSDB-RESULTS.md, update continuation documents, commit/push task-owned files, and keep issue #126 open. Retained-heap instrumentation, host sizing, a frozen resource-pressure matrix, and density/maintenance measurements remain subsequent work.
