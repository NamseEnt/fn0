# RocksDB service backend R1a

Status: R1a implementation and local correctness check complete; OCI validation
is pending. This is not a claim of complete backend conformance.

## Storage format

Each tenant has an independent RocksDB directory named `tenant-<id>` below the
configured data directory. RocksDB WAL is enabled. Every transaction uses
`WriteOptions.sync = true`; a successful mutation response follows the durable
TransactionDB commit.

User records use the key `0x01 || DocumentKey::encode()`. Internal metadata
uses keys beginning with `0x00`; the revision counter is stored at `0x00 0x01`.
This disjoint prefix keeps internal state separate from arbitrary binary user
keys.

Each user record contains a format byte, a state tag, an eight-byte big-endian
revision, and the value bytes when present. A delete writes a tombstone record
instead of removing the key. An absent RocksDB key is logical Missing at
revision zero; a tombstone is Missing at its persisted nonzero revision.

The tenant revision counter is read and advanced in the same RocksDB
transaction as the user mutations. It starts at zero, increments once for each
successful mutating request, and is never derived from RocksDB's sequence
number. All mutations in one request receive the allocated counter value.
Condition-only transactions do not advance it and return no commit identity.
Overflow is rejected before mutation.

## Transaction and execution model

The service validates requests with `TransactionRequest::validate`. It locks
the sorted union of condition and mutation keys with `Transaction::GetForUpdate`
before checking any predicate. Missing keys are locked too, and predicates on
unmodified keys stay locked through commit. The mutation and metadata counter
are written through that same transaction. Predicate failures roll back before
any write is committed.

An additional mutex per tenant serializes calls through each opened handle,
including reads, condition checks, and synchronous commits. There is no
cross-tenant write mutex. A bounded semaphore limits blocking RocksDB work to
the configured operation count, and RocksDB work runs on Tokio's bounded
blocking pool.

Concurrent first access shares one async initialization cell per tenant. The
registry lock is not held during RocksDB open or ordinary I/O. `shutdown`
removes registry entries and drops service-owned handles after in-flight
references are released. Reopening is supported by constructing the service
again with the same data directory.

## Scope and limits

R1a implements Get, Put, Delete, and the existing conditional transaction API.
Query and Scan return an explicit invalid-request error. The crate has no
transport CLI integration, eviction, hard memory limit, configured open-handle
capacity, or cross-tenant transaction support. Shutdown assumes admission has
quiesced; a concurrent execute after shutdown can open the tenant again.

The tenant mutex and counter key serialize independent writes in one tenant
through sync completion. This is a correctness-first baseline for API
comparison, not a final high-performance RocksDB configuration. R1a does not
implement process-crash fault injection or claim power-loss testing. Response
budgets are checked against the encoded service response; reads currently
materialize a value before that check, and mutations commit before validating
the response frame budget.

The build uses a small C++ TransactionDB shim against RocksDB 11.8.1 commit
`abeebd9630f11bd08c28b7bd43c7bdfc62050654`. The existing benchmark shim exposes
only ordinary DB/WriteBatch and cannot provide `GetForUpdate` or transactional
commit semantics, so it is not reused as the service implementation.

## Build and use

Build or select a RocksDB 11.8.1 source checkout at the pinned commit and build
its static library. `DODB_ROCKSDB_DIR` must point to that git checkout, which
contains `include/`, `make_config.mk`, and `librocksdb.a`. The build script
checks the checkout SHA and the C++ shim checks the linked version.

```sh
DODB_ROCKSDB_DIR=/path/to/rocksdb-11.8.1 cargo test -p dodb-rocksdb
```

Create `RocksDbServiceConfig` with an application-owned data directory and
construct `RocksDbService`. Call its `DodbService::execute` method with the
existing `TenantId`, `Request`, and `ExecutionBudget` values. `shutdown` drops
opened handles; a new service can reopen the same directory. The crate is not
wired into the QUIC server's backend selection.

## R1a verification record

The correctness test uses temporary on-disk tenant directories and independent
expected values. It does not use a mock or derive expected values from the
service under test.

| Environment | RocksDB source | Command | Result |
| --- | --- | --- | --- |
| Local macOS 25.4 ARM64 | dodb source `f7325bc24a46afe0912e200ae8a08b9275157c6e`; RocksDB `abeebd9630f11bd08c28b7bd43c7bdfc62050654` | `DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo test --locked -p dodb-rocksdb --all-targets` | Passed: 1 on-disk integration test; 0 failed. Parent SHA `e5a4692c229dcfe211db02b58552e3da63a8bff2` also passed 5 consecutive repeats. |
| OCI A1 Linux aarch64 Release | dodb source `f7325bc24a46afe0912e200ae8a08b9275157c6e`; RocksDB `abeebd9630f11bd08c28b7bd43c7bdfc62050654` | `env DODB_ROCKSDB_DIR=/bench/zfs/db/product-build/rocksdb-source CARGO_TARGET_DIR=/bench/zfs/db/dodb-rocksdb-r1a-70105e0f871a25a9/target TMPDIR=/bench/zfs/db/dodb-rocksdb-r1a-70105e0f871a25a9/tmp cargo test --locked -p dodb-rocksdb --release --all-targets` | Passed: native ARM64 Release build and 1 on-disk integration test; 0 failed. |

The local Rust all-target check and formatting check passed. The local RocksDB
static library was built with `make static_lib -j6 DISABLE_WARNING_AS_ERROR=1`
because pinned RocksDB source emits an unrelated unused-parameter warning
under this macOS clang. OCI reused an existing clean ARM64 static archive at the
pinned source SHA; its 11.8.1 version macros, AArch64 archive object, defined
TransactionDB symbols, and SHA-256 were recorded in the
[correctness report](../experiments/results/rocksdb-service-r1a-70105e0f/REPORT.md).
The preserved pre-fix cold-open assertion failure and its corrected commit-order
invariant are recorded in the [R1a correctness report](../experiments/results/rocksdb-service-r1a-70105e0f/REPORT.md).
No performance test is part of R1a.
