# Backend hot-path improvements

## Scope and decision

Implement and evaluate two independent changes on top of profiling report commit `79e6b71d09b6c1c1e0e5179c9d41f980341cb6b7`: cache-first B-link shard reads and lock-only RocksDB mutation keys. The frozen measured control remains source `cc80d1fdc7a98792be1a739056a3013debf39268` and its original hashed executables at `/bench/zfs/db/blink-checkpoint-costs-20261023`. Each backend's changed path is independent; a combined candidate checkout changes only the corresponding path for each backend.

Preserve conditional/multi-key atomicity, sorted exclusive key locking, missing/tombstone revisions, reserved revision ranges, synchronous WAL, admission, grouping, workload drivers, checkpoint policy and storage formats. Storage defaults remain unchanged. WAL, checkpoint architecture, scheduling and memory-policy changes are follow-up work selected after these results.

## Implementation

1. B-link checks the existing shard registry before constructing a path or checking the filesystem. Cache misses retain the existing open and capacity behavior. Do not cache absence.
2. Minimal-native RocksDB annotates the sorted union of condition and mutation keys with whether a value is required. Condition keys retain existing GetForUpdate/decode behavior. Mutation-only keys use exclusive GetForUpdate with a null value pointer, retaining native conflict validation and locking. Revision metadata retains its existing path. Do not change the other RocksDB adapter.
3. Add focused coverage for cached reads, uncreated tenants, capacity, reopen, initialization races, and native lock exclusion/release on present and missing keys. Run the existing shared transaction/read/history and recovery contracts on OCI/ZFS.

## Measurement and gates

Use only `opc@217.142.246.204`, AArch64 Rust 1.97.1 with `+crc`, pinned RocksDB 11.8.1, and ZFS `/bench/zfs/db`. Use the existing SSH identity with ordinary host-key verification. Preserve old checkouts, binaries, evidence and databases. Record source/build identity, commands, executable hashes and external full-window resource samples.

RW80 retains sixteen dedicated clients, the 80:20 API mixture, conditions/revisions, widths 1/4/8/16 and 512-byte values. E retains sixty-four clients and unconditional width-16 overwrite batches. Retain seed 20261013, ten-second warmup, synchronous WAL, background coalesced immutable bases, 128 MiB WAL trigger and 32 MiB segments. Compare uninstrumented throughput and complete latency observations.

Run four short preflight cells, then three 60-second repetitions of both workloads across old/new B-link and old/new minimal-native RocksDB: 24 short cells. Rotate implementation order. Evaluate each backend against its own control. The screening target is at least 5% improvement in a target metric without more than 3% throughput regression elsewhere or more than 5% latency/resource regression. Report variation and all failed gates rather than silently selecting favorable metrics.

After functional and evidence acceptance, run three 300-second repetitions across the same four implementations and two workloads: 24 sustained cells. A failed screening gate is reported explicitly; sustained comparison may still resolve a noisy or conflicting short result, but cannot turn a failed correctness/resource gate into acceptance. Compare improved B-link against improved RocksDB separately from old/new changes. Default promotion is outside this experiment.

Require zero unintended errors, complete latency counts, matching fixed per-client input prefixes, full admission, final/reopen values and revisions, completed checkpoint drain and acknowledged-write overlap. Require at least three in-window checkpoints in short B-link cells and ten in sustained cells. Keep 3 GiB process-group RSS, 4 GiB per-database, 1.5 GiB host-memory reserve, 10 GiB filesystem reserve and existing recovery-envelope guards.

The host currently has approximately 21 GiB free on the benchmark dataset. Preserve each new completed database in a lossless compressed archive with a file inventory and verified hashes before reclaiming only that case's original directory. Archive work runs between cells, never during measurement. Existing experiment data is untouched. Stop before violating resource guards; retain failures and incomplete attempts separately.

## Deliverables

Task-owned implementation commits, functional/build logs, all per-run reports and resource records, old/new and optimized cross-backend comparisons, reproducible reduction/verification, and an English results report. Preserve database archives on the host. Commit and push only this worktree's changes without co-author trailers.
