# Background B-link Checkpoint Results

## Status

Implementation is incomplete. The current change contains the pre-implementation protocol documents and an isolated manifest codec with three passing unit tests. The codec is not yet used by the B-link storage path. The complete `dodb-storage` library suite passed 187 tests with 4 ignored after updating a stale exact-sync expectation; its original failure and same-baseline reproduction are preserved. There is no segmented WAL rotation or recovery, no physical background checkpoint, no write/checkpoint overlap evidence, and no end-to-end execution mode or CLI support.

No performance measurement has run. The mandatory functional and recovery gates are not complete, so the short and long matrices were not started.

## Environment gate

The approved OCI host was reachable and had the required ZFS mount. Its default Rust/Cargo is 1.98.1, and the required 1.97.1 toolchain is installed for explicit selection. Two clean existing RocksDB source checkouts match the pinned source commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; the previous experiment records archive SHA-256 `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`. No performance execution has used the host. The source helper/cache provenance must still be verified before any build.

## Current evidence

See `VALIDATION.md` for the exact worktree, test command, OCI inspection, and unrun requirements. `PLAN.md` and `RECOVERY-PROTOCOL.md` describe the predeclared implementation and recovery contract. No claim is made that those protocols are implemented or that this change is safe to use for durable storage.
