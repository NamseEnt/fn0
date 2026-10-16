# RocksDB read/write admission experiment

## Change and hypothesis

Add an opt-in `SplitRead2Write8` admission mode to the RocksDB service and
`--blocking-admission split-read-2-write-8` to the existing workload driver.
The existing shared-eight and split-two/six modes retain their defaults and
limits. The new mode requires ten total operations, reserves two read slots,
and retains eight write slots. Shutdown drains the configured lane limits.
The new option does not modify request semantics, key locks, revision
algorithms, WAL, sync, workload construction, or the product backend.

Compare shared eight, shared ten, and split read-two/write-eight with the
existing ReservedRanges allocator. Shared ten controls for increased total
concurrency. Split two/eight may reduce read admission delay without the
write-slot reduction of the previous split two/six experiment.

## Primary measurements

Run all three configurations on RW95, RW80, RW50, RW20, and MT-MIX with
research/read-stage/detailed-write metrics disabled, three repetitions each:
45 performance runs. Rotate configuration order across scenarios and
repetitions. Run one additional diagnostic repetition per condition with
metrics enabled: 15 diagnostic runs. Diagnostics do not rank performance.
Six short smoke runs cover every configuration/metrics combination.

Use the authorized OCI A1 two-CPU ARM64 host, pinned RocksDB 11.8.1 archive,
Rust 1.97.1 release with ARM64 +crc, ZFS /bench/zfs/db, one immutable binary,
and a clean exact source commit. Preserve the existing workload: seed
20261013, 16 clients, 10,000 rows/tenant, 512-byte values, a one-second warmup,
five-second admission window plus drain, and fresh databases per run.
Run serially without concurrent builds or benchmarks. Preserve command,
hashes, host/source provenance, exit status, RSS/CPU/completion evidence,
independent expected-state replay and shutdown/reopen validation, and any
failed attempts. Match schedules and per-client input prefixes across runs.

## Decision screen declared before execution

For each tenant/profile group, compare medians of the three OFF runs with
shared eight. A candidate passes the descriptive screen only if ordinary Get
and Query16 p95 and p99 all improve, successful write throughput is at least
95% of the reference, and successful-write p99 is at most 110% of the reference.
Report each metric and its observed range; a group-level failure prevents a
general recommendation. This is a research screen, not a statistical
noninferiority claim. MT-MIX tenant groups must be reported separately.

Keep the serialized allocator and shared-eight service defaults. No default
promotion is authorized by this experiment. Five-second runs cannot establish
sustained, large-dataset, crash-recovery, or production performance.
