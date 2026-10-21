# Background Checkpoint Validation Record

## Worktree and starting state

- Baseline: `39cdfe48a20fb6f381dbb461d721fd51fa8f9f0c`.
- Branch: `experiment/blink-background-checkpoint-20261021`.
- Worktree: `/Users/namse/fn0-dodb-background-checkpoint`.
- The existing `/Users/namse/dodb` and `/Users/namse/fn0-dodb-wal-reclamation` worktrees were not modified.
- Existing WAL reclamation evidence was read from `RESULTS.md`, `VALIDATION.md`, `corrections/README.md`, and `oci/matrix-verification-corrected-v4.json`; no original file was changed.

## Initial implementation check

The first implementation artifact is the versioned, CRC32C-protected segmented-WAL manifest codec at `dodb/crates/dodb-storage/src/blink/segmented_wal.rs`. It encodes shard/database identity, checkpoint tree metadata, ordered segment LSN ranges, the active/next segment number, and metadata generation. Its atomic writer synchronizes the temporary file and parent directory. It is not yet connected to `BlinkStore`, WAL rotation, recovery, checkpoint execution, service configuration, or workload CLI.

- `cargo +1.97.1 test --locked -p dodb-storage --lib blink::segmented_wal::tests -- --test-threads=1`: passed, 3 tests.
- `cargo +1.97.1 fmt -p dodb-storage -- --check`: passed.
- `git diff --check`: passed.
- An initial format check before applying rustfmt reported formatting differences; the final format check above is clean.
- `cargo +1.97.1 test --locked -p dodb-storage --lib -- --test-threads=1`: passed after correcting the legacy checkpoint barrier's expected physical WAL sync count from 4 to 5; 187 passed and 4 were ignored. The initial failing output and unchanged-baseline reproduction are preserved in `evidence/legacy-sync-count-baseline-failure.txt`.
- `cargo +1.97.1 test --locked -p dodb-server --test blink_wal_checkpoint -- --test-threads=1`: passed, 6 tests, covering the existing default-disabled legacy policy, threshold checkpointing, failure behavior, reopen, and process termination.
- The focused test build emitted existing dead-code warnings in experiment binaries included by the package test target. No warning suppression was added.

These tests validate manifest encoding and local atomic file replacement only. They do not validate segmented WAL storage, acknowledged-write recovery, background checkpoint overlap, required format contracts, fault injection, SIGKILL recovery, or any performance behavior.

## Approved OCI environment check

The authorized host was reached using the specified identity, `IdentitiesOnly=yes`, and normal host-key validation. It reported:

- Host: `instance-20260923-1013`, Linux AArch64.
- Filesystem: `/bench/zfs/db`, ZFS dataset `dodbbench/db`, 149 GiB total and 44 GiB free at inspection.
- `MemAvailable`: 4,678,552 KiB at inspection.
- Active benchmark/build process check: no matching workload, Cargo, or rustc process was running; only the inspection shell matched its own process list.
- Default `rustc`/Cargo: 1.98.1. The required Rust/Cargo 1.97.1 toolchain is installed and reports `rustc 1.97.1 (8bab26f4f 2026-07-14)` and `cargo 1.97.1 (c980f4866 2026-06-30)` when selected explicitly.
- Existing RocksDB source checkouts at `/bench/zfs/db/crossdb-b1648b03-oci/rocksdb-source` and `/bench/zfs/db/product-build/rocksdb-source` both report the required source commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; their working trees were clean at inspection. The prior experiment records the pinned archive SHA-256 as `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
- The existing OCI build helper is present at `dodb/scripts/run-oci-rocksdb-build.sh`. No build or measurement was started during this environment inspection.

The environment matches the named host, filesystem, Rust toolchain availability, and pinned RocksDB source checkout requirements. The older OCI RocksDB source/archive and build cache are candidates for reuse only after their exact helper configuration and archive/source hashes are checked for a new build. No alternate host, local filesystem, or toolchain substitution was made.

## Required validation status

Not run: the sync/background contract suites; concurrent-write barrier tests; checkpoint state-preservation tests; segment transition and fault-boundary tests; format/model checks; required-file and corruption recovery cases; parent-oracle SIGKILL recovery; shutdown and resource-cap tests; OCI build/test; short or long performance matrices.

The performance stage remains prohibited until implementation and every required functional/recovery validation pass.
