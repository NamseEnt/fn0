# Phase 0: Architecture contracts and correctness harness

## Delivered

- Rust workspace with core, storage-foundation, and testkit crates.
- Strong newtypes for tenant, shard, epoch, page, LSN, transaction, and
  revision identities.
- Canonical arbitrary-binary `(pk, sk)` mem-comparable encoding with strict
  decode errors and property tests.
- Explicit 4096-byte page header codec with CRC32C and version/type checks.
- Double-superblock codec and highest-generation valid-copy selection.
- `DurableFile` plus thin `std::fs::File` adapter.
- Deterministic crashable in-memory file with separate volatile/durable bytes,
  operation-indexed short-write and I/O faults, and crash recovery.
- Named `CrashInjector` with deterministic Nth-hit behavior.
- Reference get/put/delete/query/scan and optimistic transaction oracle.
- Conflict, ABA, atomicity, and deterministic randomized tests.

## Explicit non-goals

No B+Tree, page allocator, free list, overflow implementation, WAL, recovery,
fsync commit protocol, production OCC, MVCC, checkpoint, replication,
shard movement, 2PC, query parser, network/server/client, or performance
tuning is present.

## Exit criteria

`cargo fmt --check`, workspace clippy with `-D warnings`, and
`cargo test --workspace` must pass. Phase 1 may use the existing key, page,
superblock, durable-file, and testkit contracts as its starting surface.
