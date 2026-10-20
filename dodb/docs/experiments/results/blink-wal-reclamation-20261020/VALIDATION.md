# Validation Record

## Source

- Starting source SHA: `a5b37c23ab72d95c8d5d7bed263459fd7af141e2`
- Work branch: `experiment/blink-wal-reclamation-20261020`
- Worktree: `/Users/namse/fn0-dodb-wal-reclamation`
- Existing checkouts and result trees were inspected and left unchanged.

## Local checks

- `cargo fmt --all -- --check`: passed after formatting the task-owned Rust files.
- `git diff --check`: passed.
- `cargo test -p dodb-server`: passed, including five new B-link WAL checkpoint tests and the existing server integration suite.
- `cargo test -p dodb-storage --lib page_delta_checkpoint_fault_matrix_preserves_acknowledged_writes`: passed. This validates the existing function-level physical checkpoint fault matrix.

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

No performance result is recorded yet. The four-path, two-workload, three-repetition matrix remains gated on OCI compilation and passing B-link-enabled contract tests.
