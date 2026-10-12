# dodb RocksDB service R1b-1 Query/Scan verification

Status: passed for the R1b-1 Query/Scan and read-contract scope only. No
performance measurement, long-duration run, process-kill test, or Debian work
was performed.

## Source and artifacts

- Repository: `https://github.com/NamseEnt/fn0.git`
- Branch: `experiment/b-link-batched-engine-monorepo`
- Final implementation SHA tested locally and on OCI: `e59ae32117df13e23cf298d2f8ec6a8fc4c53a93`
- Earlier implementation SHA `2774bd1c3491134bbc944c91a8bf34d3bfa22a6e` also
  passed local and OCI checks before commit `e59ae32117df13e23cf298d2f8ec6a8fc4c53a93`
  added an explicit success assertion at the exact response-budget boundary.
- RocksDB source SHA: `abeebd9630f11bd08c28b7bd43c7bdfc62050654`
- RocksDB release: 11.8.1
- Final OCI run directory: `/bench/zfs/db/dodb-rocksdb-r1b1-e59ae32117df13e2`
- OCI test log: `oci-release-test.log`
- OCI test log SHA-256: `78ffc676ff05984cdf983beebd641ce40e8d113d1697ec84a03b3ede5c8b0ca5`
- Prior successful OCI log at SHA `2774bd1c3491134bbc944c91a8bf34d3bfa22a6e` is
  preserved as `oci-release-test-2774bd1c.log` with SHA-256
  `5d1706bfe133fb515b66ab34ccb8e331503136a322c9297f40d704970d204aff`.
- Existing local untracked experiment result directories were preserved and not staged.
- Repository content added for this task is English-only; no source comments were added.

## Implementation behavior

Query seeks to a RocksDB key prefix for the requested partition and stops at the
first different partition. With an exclusive sort-key cursor it seeks directly
to the encoded cursor key and skips equality. Scan seeks to the first user-key
prefix or the encoded exclusive document cursor. `DocumentKey::encode()` uses
escaped components with terminators, which preserve the logical unsigned-byte
ordering under RocksDB's bytewise comparator, including `0x00` and `0xff`.
Internal metadata has prefix `0x00`; user records have prefix `0x01`. Iteration
therefore excludes metadata. Logical tombstones share user-key order but are
skipped by their stored state tag. Neither operation reads and sorts the full
database.

Each request creates a RocksDB snapshot and an iterator whose `ReadOptions`
references that exact snapshot. The C++ iterator wrapper owns and releases both
objects. The tenant mutex is held only while creating the snapshot and iterator;
the iterator continues after the mutex is released, allowing writes to proceed
against the stable read view. The service-held database reference keeps the DB
alive until iterator destruction. Snapshots are not kept across separate
requests or pages.

Native Query/Scan use a 17-byte response-envelope wire reservation; each
document reserves `PK + SK + value + 20` wire bytes and
`max(PK + SK + value + 64, wire bytes)` materialization bytes. The memory
ceiling is wire maximum plus 64 KiB. RocksDB uses those same values and rejects
an oversized response with an error rather than truncating it. It checks the
encoded value length and reserves the document before copying its value into
the response. The `limit` remains an item count. An absent tenant's empty
response uses the existing protocol encoder budget check.

The R1a write-budget issue was checked against the native service. RocksDB
mutations commit before their response-budget check. Native single-key Put/Delete
also commit before the protocol layer encodes the response; its batch mutation
fast path does not reserve a read response budget. An over-budget write response
can thus fail after the mutation commits on both paths. This task did not change
write behavior; a write-budget contract change remains separate work.

## Local verification

- Host: macOS 26.4, ARM64; Rust 1.97.1, Cargo 1.97.1.
- RocksDB source: `/tmp/dodb-r1a-rocksdb-abeebd9`, clean at the pinned SHA.
- Commands:

```sh
cargo fmt -p dodb-rocksdb -- --check
DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo test --locked --offline -p dodb-rocksdb --all-targets
DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo check --locked --offline -p dodb-rocksdb --all-targets
```

- Result: format check passed; all-targets check passed; tests passed (1 unit,
  3 integration; 4 total, 0 failed).
- The shared read test uses temporary on-disk directories for the native and
  RocksDB services and applies the same requests and expected logical records
  to both.

## OCI verification

- Host: OCI A1, Linux `aarch64`; Rust 1.98.1, Cargo 1.98.1.
- Filesystem: ZFS mounted at `/bench/zfs/db`.
- Checkout: clean detached checkout at final implementation SHA in
  `/bench/zfs/db/dodb-rocksdb-r1b1-e59ae32117df13e2/source`.
- Cargo target and `TMPDIR`: dedicated subdirectories under that run directory.
- RocksDB source was clean at
  `abeebd9630f11bd08c28b7bd43c7bdfc62050654`. Its existing archive SHA-256 was
  `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`; its
  `make_config.mk` reports version 11.8.1 and `g++`. The archive and version
  provenance were already established by the R1a report; this run confirmed
  they were unchanged. No source or infrastructure settings were modified.
- Command:

```sh
cargo test --locked -p dodb-rocksdb --release --all-targets
```

- Result: native ARM64 Release build passed; 1 unit and 3 integration tests
  passed, 0 failed. Temporary tenant database paths were on the ZFS-backed
  dedicated `TMPDIR`.

## Common read-contract coverage

The same read-contract test runs against the native `LocalTenantService` and the
RocksDB service. It checks: empty database and missing partition; multiple
partitions and sort keys; total Query and Scan order; existing and nonexistent
exclusive cursors, first/last keys, and partition boundary; binary `0x00` and
`0xff` keys and binary values; tombstone omission and delete/reinsert; zero,
bounded, and page-continuation limits without gaps or duplicates; exact budget
boundary and a single over-budget document; identical keys in four tenants;
and a coordinated atomic two-key transaction racing a read, which may see the
complete old or complete new state only. A separate RocksDB test deterministically
opens a snapshot iterator, commits a two-key transaction between iterator reads,
and verifies that the remaining read comes from the original snapshot. A
shutdown/reopen test compares logical Query and Scan contents.

Revision number equality is not used across backends; comparisons use logical
keys, values, ordering, and whole-state outcomes.

## Failed attempts and corrections

- `cargo fmt --check --manifest-path dodb/Cargo.toml` failed because this
  repository's workspace manifest is at its root, not `dodb/Cargo.toml`. It was
  corrected to `cargo fmt -p dodb-rocksdb -- --check`.
- The first locked test invocation failed before running because adding the
  native-service test dependency required a lockfile update. The lockfile was
  updated with an offline package check and subsequent commands used
  `--locked --offline`.
- That first lockfile-update check omitted `DODB_ROCKSDB_DIR` and failed in
  `build.rs` with `DODB_ROCKSDB_DIR ... NotPresent`. The check was rerun with the
  pinned local RocksDB source and passed.
- The first focused test run found an R1a test still expecting empty-partition
  Query to be invalid because Query had previously been an unimplemented stub.
  The native service accepts this request and returns an empty result, so the
  test now checks that actual contract. The full focused suite then passed.
- An OCI process-inspection command used `rg`, which is not installed there and
  returned `rg: command not found`; the same inspection was repeated with
  `grep`. This did not affect the checkout or validation command.

## Remaining limitations and gates

- R1 remains incomplete. Common transaction conformance, tenant lifecycle and
  capacity management, and process-kill recovery remain pending.
- RocksDB still serializes a tenant's writes through its handle mutex and shared
  revision-counter key. Resolve this serialization design before performance
  comparison; this task ran no performance measurement.
- Tests establish process-level snapshot semantics and file-backed operation,
  not power-loss, disk-loss, or process-kill recovery guarantees.
- Write-response budget behavior remains post-commit as described above.
- No benchmark, long-duration loop, Debian access, or infrastructure change was
  performed.
