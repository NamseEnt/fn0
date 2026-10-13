# R1b-3b WAL-sync-boundary all-or-none recovery report

## Result

The one planned crash case for each backend passed locally and on OCI. In all
four accepted runs, the parent received the handoff from the actual backend
WAL sync hook, killed that child with SIGKILL, confirmed signal 9, reopened the
same path, and recovered the complete target state. Each reopened service then
accepted a new transaction and returned the expected full Scan. Recovering the
complete target is valid because process SIGKILL leaves kernel page cache in
place. The cases were not repeated to seek a different recovery outcome.

This verifies one point after target WAL append and before its durability sync
call, per backend. It does not test interruption during the sync syscall,
power loss, page-cache loss, storage loss, or the rest of the process-crash
inventory. R1 remains partial.

## Storage paths and hook placement

Native `LocalTenantService::execute(Request::Transact)` opens the production
`BTreeStore<ProductionFile, ProductionFile>` for the tenant and executes via
`AsyncShard`. `Wal::finish_group_append` finishes writing the commit group,
calls the existing `before_wal_sync` fault point, and then calls
`self.file.sync_data()`. The target response is returned only after this
sequence. The test reuses `FaultInjectorFactory` on this service path and
blocks inside the named fault point before `sync_data()` begins. The target
file is the tenant shard `.wal` file. Observed local and OCI files were
`tenant-913-shard-913.wal` under each run's temporary service directory.

RocksDB `RocksDbService::execute(Request::Transact)` calls
`dodb_rocksdb_transaction_commit`, which invokes `rocksdb::Transaction::Commit()`.
The transaction starts with `WriteOptions.sync = true` and WAL enabled. In the
pinned RocksDB 11.8.1 source, the synchronous DB write calls `DBImpl::SyncWAL()`;
`SyncWalImpl` then calls `WritableFileWriter::SyncWithoutFlush` on the active
WAL before any WAL-directory fsync. A test-only `EnvWrapper` wraps writable
files only when `DODB_R1B3B_ENABLE_WAL_SYNC_HOOK=1` is set in the child. Its
`WritableFileWrapper::Sync()` distinguishes a numeric `<digits>.log` WAL
filename from all other filenames. Once armed, it reports and blocks before
delegating to the existing `Sync()`. A non-WAL file sync reports
`R1B3B_WRONG_SYNC_TARGET` and fails the parent case. MANIFEST, SST, directory,
and other sync operations cannot be reported as the accepted WAL hook.

The native fault injector is installed before the child opens the tenant
shard, but its shared atomic arm flag remains false for DB open and the seed
transaction. RocksDB opens with the wrapper active only in its child; the
wrapper's arm flag is false through DB open and seed. Both children require a
successful seed response before arming the hook. The target request has a
parent-visible `TARGET_RETURNED` failure marker if it returns before the hook.

## Shared crash scenario and oracle

The parent owns the unique temporary data path, seed request, target request,
and two complete expected maps. The seed transaction creates four documents.
The width-four target transaction has successful seed-state conditions and
updates two existing values, creates one key, and deletes one existing key.
One seeded document is untouched. The parent derives both maps directly from
those requests; no child DB dump is used.

The child flushes a backend-specific hook message from inside the storage
callback. For native, the message names `before_wal_sync` and the `.wal` path.
For RocksDB, it names `wal_sync_before` and the concrete numeric `.log` path.
The parent checks the message and that the child is still alive, sends SIGKILL
to that exact PID, reaps it, and requires `signal() == Some(9)` before reopen.
The seed success response and hook arrival prove the target response had not
returned. Hook timeout, hook absence, wrong RocksDB sync filename, unexpected
termination signal, reopen failure, and partial or unexpected state are
separate test failures. No case timed out, missed its hook, used the wrong
sync target, exited with another signal, failed reopen, or exposed a partial
state.

The first full Scan after reopen must equal either the complete seed map or
the complete target map. Equality checks reject missing and extra documents.
The seed-state outcome preserves every seed value, including the key that the
target would delete, and requires the never-created key to have the existing
zero missing revision. The complete-target outcome checks both updated values
and the created value, the deleted key's missing tombstone, and that all four
target mutations share one logical revision greater than the untouched seed
revision. The untouched document is preserved in both states. Revision
numbers are checked only within each backend; no numeric value or increment
size is compared across backends. A new conditional transaction after reopen
must succeed, and a second complete Scan must contain exactly the recovered
state plus that new document.

## Execution counts

| Environment | Planned accepted cases | Accepted cases run | Passed | Failed | Not reached |
| --- | ---: | ---: | ---: | ---: | ---: |
| Local macOS ARM64 | 2 | 2 | 2 | 0 | 0 |
| OCI A1 ARM64 Release | 2 | 2 | 2 | 0 | 0 |
| Total | 4 | 4 | 4 | 0 | 0 |

Two additional local pre-evidence runs, one per backend, also reached the
actual hook, received SIGKILL, and recovered the complete target state. The
accepted local run logs, both preliminary logs, and the failed local link log
are retained in `local-execution-logs.tar.gz`. The initial OCI direct helper
invocation failed with `Permission denied` before the helper or Cargo started.
Retrying the same helper through `bash` reached both accepted OCI cases. These
setup or link failures are not crash-case failures.

## Verification provenance

Implementation SHA: `e742ebb30cbe0452df73c1390b95280ad038b07c`.

| Environment | Source | Toolchain and build | RocksDB | Test executable |
| --- | --- | --- | --- | --- |
| Local macOS 26.4, ARM64 | Worktree contents committed unchanged as implementation SHA above | `rustc 1.97.1 (8bab26f4f 2026-07-14)`; `cargo 1.97.1 (c980f4866 2026-06-30)`; `DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9`; focused debug tests | Source `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; archive SHA-256 `c425d7aee1ffb38f3093e43014d56504e398cd68bfd2262f4b6a0881601efc0f` | `/Users/namse/fn0-dodb-blink-experiment/target/debug/deps/process_crash_contract-f8961cdc8dfe59b3`; SHA-256 `155f2651c691b8cc87cb56e50f3e01fd68becf63e736461ee9e3da7ffb9b19b8` |
| OCI A1, Oracle Linux 9.8, ARM64 Neoverse-N1, 2 OCPU, ZFS | Clean detached checkout at `e742ebb30cbe0452df73c1390b95280ad038b07c` | Rust and Cargo 1.97.1; Release; shared helper; shared Cargo target and sccache | Source `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; archive SHA-256 `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5` | `/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target/release/deps/process_crash_contract-5fccc606caf78a3e`; SHA-256 `c6d7b917fdd50fef0a12d9005e0c70d363906ecd7a34bbf67acf4213f3df2390` |

Local focused commands were run separately for `native_wal_sync_interruption_recovers_all_or_none`
and `rocksdb_wal_sync_interruption_recovers_all_or_none`:

```sh
DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo test --locked --offline \
  -p dodb-rocksdb --test process_crash_contract \
  native_wal_sync_interruption_recovers_all_or_none -- --nocapture

DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo test --locked --offline \
  -p dodb-rocksdb --test process_crash_contract \
  rocksdb_wal_sync_interruption_recovers_all_or_none -- --nocapture
```

Before those runs, `cargo fmt -p dodb-rocksdb -- --check` and the following
locked compile check passed:

```sh
DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo check --locked --offline \
  -p dodb-rocksdb --test process_crash_contract
```

The first locked check preceded the offline lockfile update and stopped before
compilation; details are in `setup-attempts.md`.

OCI used the same filters with
`DODB_BUILD_RUN_ID=r1b3b-e742ebb30-native-01` and
`DODB_BUILD_RUN_ID=r1b3b-e742ebb30-rocksdb-01`:

```sh
bash dodb/scripts/run-oci-rocksdb-build.sh test --locked --release \
  -p dodb-rocksdb --test process_crash_contract \
  native_wal_sync_interruption_recovers_all_or_none -- --nocapture

bash dodb/scripts/run-oci-rocksdb-build.sh test --locked --release \
  -p dodb-rocksdb --test process_crash_contract \
  rocksdb_wal_sync_interruption_recovers_all_or_none -- --nocapture
```

The first OCI helper run compiled six workspace crates because the clean
checkout root was new; sccache recorded six Rust misses and one C/C++ miss,
with zero hits. It reused the existing shared target contents and the existing
RocksDB archive without rebuilding RocksDB. The second run finished from the
shared Cargo target without compiling crates or adding sccache requests. Full
helper evidence, including environment, command, timing, sccache counters,
test logs, and helper binary manifests, is retained in `oci-build-runs.tar.gz`.
The helper's manifest did not extract the executable from Cargo's `Running`
line, so its path was independently resolved from that line and hashed as
listed above.

## Historical Rust version record

The earlier R1b-3a report's OCI `Rust 1.98.1` and local `Rust 1.97.1` entries
are corrected to **unverified**. Their retained direct test logs have no
`rustc -vV` or `cargo -V` output. The repository pin and a reused Cargo
fingerprint cannot prove the historical compiler versions. The current local
run recorded both versions directly above, and the current OCI runs recorded
both versions in each helper `environment.txt`.

## Remaining scope

Only this WAL-append-to-sync-call boundary was verified. Interruption inside
the sync syscall, power failure, cache or device loss, other commit boundaries,
tenant lifecycle and capacity management, and the remaining R1 gates are
unverified. No benchmark, Debian task, RocksDB archive build, or large
repetition campaign was run.

This report is part of the final documentation commit. Its final SHA is
recorded in the delivery summary and the branch history.
