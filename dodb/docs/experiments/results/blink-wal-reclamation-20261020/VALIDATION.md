# Validation Record

## Source

- Starting source SHA: `a5b37c23ab72d95c8d5d7bed263459fd7af141e2`
- Work branch: `experiment/blink-wal-reclamation-20261020`
- Worktree: `/Users/namse/fn0-dodb-wal-reclamation`
- Existing checkouts and result trees were inspected and left unchanged.

## Local checks

- `cargo fmt --all -- --check`: passed after formatting the task-owned Rust files.
- `git diff --check`: passed.
- `cargo test -p dodb-server`: passed, including six new B-link WAL checkpoint tests and the existing server integration suite.
- `cargo test -p dodb-storage --lib page_delta_checkpoint_fault_matrix_preserves_acknowledged_writes`: passed. This validates the existing function-level physical checkpoint fault matrix.

The local B-link checkpoint tests include a subprocess that commits and acknowledges a transaction, pauses inside the physical checkpoint before page flush, receives SIGKILL, and then verifies recovery after reopening the same database. This is actual process termination coverage; it does not simulate host reset or power loss.

The local RocksDB build dependency at `/tmp/dodb-r1a-rocksdb-abeebd9` contains source files but has incomplete Git metadata and no `librocksdb.a`; Cargo stops in `dodb-rocksdb/build.rs` before compiling its binaries or tests. An attempt to fetch a fresh RocksDB source checkout was interrupted after the remote connection disconnected. Neither pre-existing path was changed. The required RocksDB-backed CLI checks and common backend contract tests remain gated on the approved OCI build helper.

## OCI preflight

Read-only preflight connected to the requested `opc@217.142.246.204` using `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes`, and normal host-key validation.

- Host: `instance-20260923-1013`, Linux AArch64.
- Rust: `1.97.1`; Cargo: `1.97.1`.
- Filesystem: `/bench/zfs/db`, ZFS, 47 GiB available at preflight.
- `MemAvailable`: 6,756 MiB at preflight.
- Pinned RocksDB archive present; source SHA: `abeebd9630f11bd08c28b7bd43c7bdfc62050654`.
- The process list showed no benchmark or build process.

Preflight values are snapshots, not a reservation. Recheck all limits immediately before every build and run. No OCI build, contract test, or performance cell has run yet.

## Performance status

Before adding the process-termination test, OCI build `blink-wal-reclamation-build-j1-faf1f2b0` and contract run `blink-wal-reclamation-contracts-faf1f2b0` passed for source `faf1f2b0df3f4af0b7bc108df675561dbf98b0bb`. The build used `CARGO_BUILD_JOBS=1`, peak process-group RSS 1,946,234,880 bytes, minimum `MemAvailable` 3,574,898,688 bytes, and minimum ZFS free space 48,212,434,944 bytes. The transaction, read, and conditional-history suites passed with B-link's one-byte WAL threshold. That evidence predates the added SIGKILL test and does not validate the current source SHA; repeat the build and contracts at the updated SHA before performance measurement.

No performance result is recorded yet. The four-path, two-workload, three-repetition matrix remains gated on building and passing the B-link-enabled contract tests at the current source SHA.
