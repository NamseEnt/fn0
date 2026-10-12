# dodb RocksDB service R1a correctness

Status: passed for the R1a scope only at final tested source SHA below. No
performance measurement was run.

## Tested source

- Repository: `https://github.com/NamseEnt/fn0.git`
- Branch: `experiment/b-link-batched-engine-monorepo`
- Final tested source SHA: `f7325bc24a46afe0912e200ae8a08b9275157c6e`
- RocksDB source commit: `abeebd9630f11bd08c28b7bd43c7bdfc62050654`
- RocksDB release: 11.8.1

## RocksDB build provenance

The OCI run reused the existing static archive at
`/bench/zfs/db/product-build/rocksdb-source/librocksdb.a`. Its source checkout
was clean at the pinned commit. `make_config.mk` reports version macros 11.8.1,
`CXX=g++`, C++20, Linux POSIX, ZLIB, and ZSTD. The archive SHA-256 was
`9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`. An
archive object was verified as ELF64 AArch64, and `nm -C` found the defined
`rocksdb::TransactionDB::Open` symbols.

The local archive was built from the same clean source commit with
`make static_lib -j6 DISABLE_WARNING_AS_ERROR=1`. The local static archive
SHA-256 was
`c425d7aee1ffb38f3093e43014d56504e398cd68bfd2262f4b6a0881601efc0f`. The
warning flag was disabled only because pinned RocksDB source emitted an
unrelated unused-parameter warning under local macOS clang. The RocksDB version
macros and C++ TransactionDB shim compiled successfully.

## Correctness runs

| Host | Checkout | Command | Result |
| --- | --- | --- | --- |
| macOS 25.4 ARM64, Apple M1 | Clean local checkout at final tested SHA | `DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo test --locked -p dodb-rocksdb --all-targets` | Passed: 1 integration test, 0 failed. `cargo check --locked -p dodb-rocksdb --all-targets` and `cargo fmt -p dodb-rocksdb -- --check` passed. At parent SHA `e5a4692c229dcfe211db02b58552e3da63a8bff2`, this test also passed 5 consecutive additional runs. |
| OCI A1, Linux aarch64, `/bench/zfs/db` on ZFS | Clean detached checkout at final tested SHA | `env DODB_ROCKSDB_DIR=/bench/zfs/db/product-build/rocksdb-source CARGO_TARGET_DIR=/bench/zfs/db/dodb-rocksdb-r1a-70105e0f871a25a9/target TMPDIR=/bench/zfs/db/dodb-rocksdb-r1a-70105e0f871a25a9/tmp cargo test --locked -p dodb-rocksdb --release --all-targets` | Passed: native ARM64 Release build and 1 integration test, 0 failed. |

An earlier OCI Release run at source SHA `70105e0f871a25a9fc75d74588da0ee9c19bba94`
also passed the then-current integration cases; its log remains at
`/bench/zfs/db/dodb-rocksdb-r1a-70105e0f871a25a9/oci-release-test.log`. The
final OCI command output is retained at
`/bench/zfs/db/dodb-rocksdb-r1a-70105e0f871a25a9/f7325bc2-oci-release-test.log`.
The
detached checkout and all OCI Cargo target and temporary test paths are under
`/bench/zfs/db/dodb-rocksdb-r1a-70105e0f871a25a9`. The integration test opened
real on-disk RocksDB databases for four tenants and exercised binary key/value
round trips, updates, repeated deletes and missing revisions, Exists, NotExists
and revision conditions,
condition-only requests, mixed Put/Delete width-four atomicity, failed
all-or-none mutations, unmodified-key predicates, expected conditional races,
conditional/unconditional Put and Delete competition, missing ABA, invalid
requests, concurrent first open of one tenant, four-tenant concurrency and
isolation, shutdown, and reopen. It checked that the revision counter advanced
past pre-shutdown state after reopen.

## Preserved failed validation attempt

At source SHA `de753fa06b290fe36cc6bed197397eac94650558`, the first cold-open
test run failed because the test expected each concurrent request's revision
to follow its task index. One observed document had revision 3 where the test
expected revision 2. This was a test-order assumption: commits may complete in
any serialization order. Commit `e5a4692c229dcfe211db02b58552e3da63a8bff2`
changed the test to assert each value plus the independent revision set
`{1, 2, 3, 4}`. The test passed five additional consecutive local runs at that
parent SHA. Source SHA `f7325bc24a46afe0912e200ae8a08b9275157c6e` added explicit
successful `Exists` and failed `NotExists` checks on present state. Its final
local and OCI Release runs passed those cases as well.

## Remaining R1 work

R1 is incomplete. Query and Scan, common conformance execution on the native
service, the remaining contract cases, explicit per-tenant close/drain and
capacity management, response pre-materialization limits, and process-kill
recovery evidence remain pending. This R1a implementation does not establish
power-loss behavior, high-performance RocksDB settings, or product performance.
