# RocksDB wrapper attribution

This investigation preserves request semantics, user-key locking, revision
allocation algorithms, WAL, sync writes, deadlines, and validation. No default
or optimized execution path changes. New opt-in measurements split existing
write-side full-value reads by condition-bearing and mutation-only keys.

Native GetForUpdate timing includes RocksDB lock wait, value lookup, and its
internal string construction, but excludes the shim copy. Shim copy measures
malloc/memcpy. Rust copy measures copying/freeing the returned value; decode
measures constructing the logical state, including the payload Vec copy. These
metrics count every attempted key read, including reads before a condition
rejection. Fetched bytes are logical returned bytes, not storage/device IO.
Mutation-only keys have no condition referencing them. Condition keys may
also be mutation keys. Both kinds retain their original full-value reads.

The workload adds `--detailed-write-metrics 0|1`. Its default retains the old
behavior: enabled unless `--read-stage-metrics 1` is selected. Explicitly
enabling both produces combined read-stage and write PerfContext diagnostics.
Metrics remain opt-in with `--research-metrics 1`.

On the pinned OCI A1 host and ZFS, run six configurations across RW95, RW80,
RW50, RW20, and MT-MIX, with three independent repetitions each:

- Serialized revisions, shared eight slots, combined metrics ON.
- Reserved ranges of 4,096, shared eight slots, combined metrics ON.
- Reserved ranges, shared sixteen slots, combined metrics ON.
- Reserved ranges, split two read/six write slots, combined metrics ON.
- Reserved ranges, shared eight slots, metrics OFF for an overhead control.
- Existing B-link service, metrics OFF for service context.

Total: six smoke runs and 90 five-second performance runs, in rotated serial
order, with independent databases. The unchanged seed is 20261013, with
16 clients, 10,000 initial rows per tenant, 512-byte values, one-second warmup,
and five-second admission. MT-MIX has four clients per tenant. Preparation
Gets remain real API calls, included in read quota and workflow latency.

Every run must pass replay and reopen validation, zero other/overload errors,
unchanged binary hashes, full-count stage accounting, and the new key/byte
identities. Across performance runs, compare each client's first 100 input
workflows and each tenant's schedule fingerprint. Inspect tenant results
separately for MT-MIX. Retain every failed attempt and do not silently retry.

Report median and min/max across three repetitions, not pooled percentiles.
ON/OFF overhead control applies only to the range/shared-eight configuration.
B-link versus instrumented RocksDB is context, not a clean engine ranking;
the uninstrumented range/shared-eight comparison is reported separately.
No fsync count, WAL-sync-only duration, long-run compaction, larger-than-memory
workload, or production superiority claim is made.

Build using the existing shared Cargo target, sccache helper, Rust 1.97.1,
ARM64 +crc, and pinned RocksDB 11.8.1 archive. Build-only helper commands use
`--no-run` for tests. Execute the produced focused tests separately on ZFS.
