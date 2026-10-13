# B-link service comparison readiness

Status: B-link service connection and common service-contract checks complete.
The workload driver and performance records are pending. No speed comparison
was run for this report.

## Backend paths

The product request boundary is `dodb_service::DodbService::execute(TenantId,
Request, ExecutionBudget)`. Three distinct constructors now lead to that same
boundary:

| Backend | Service | Storage path |
| --- | --- | --- |
| Existing native regression | `dodb_server::LocalTenantService` | `BTreeStore` plus the B-tree `AsyncShard` coordinator. |
| Native comparison target | `dodb_server::BlinkTenantService` | `BlinkStore<ProductionFile, ProductionFile>` plus the B-link tenant coordinator added for this service. |
| Comparison backend | `dodb_rocksdb::RocksDbService` | RocksDB `TransactionDB` tenant wrapper. |

The B-link service reuses the existing request, condition, response, error,
revision, and transaction types. Put and Delete become one-mutation
`TransactionRequest`s; Transact passes its original request unchanged to
`BlinkStore::apply_transaction_group`. The tenant coordinator collects queued
mutation requests and submits the group to that existing B-link commit path.
Each logical request keeps its own result and commit LSN while the group shares
the B-link WAL sync boundary. Mutations are not split into separately committed
condition checks and writes.

Get, Query, and Scan use one `BlinkReadHandle` per open tenant. Query and Scan
pin one published generation for the complete request. Their response budget,
including the Get value budget, is reserved before B-link copies a value into a
service response. Conditional failures, tombstones, revisions, exclusive
cursors, response-too-large errors, and protocol responses retain the existing
types and meanings.

Each tenant is stored in
`<data_dir>/blink/tenant-<tenant>-shard-<tenant>.db` with its adjacent `.wal`
file. The separate `blink/` directory keeps B-link page files from colliding
with the existing B-tree service's files. The service uses `BlinkStore::open_path`,
which opens `ProductionFile` for both data and WAL; no memory-backed file is
used in this service path. A normal service shutdown closes each coordinator,
and a new service instance reopens the same files.

The opened B-link store uses `DatabaseConfig::default()` with tenant and shard
identity set to the tenant ID and the existing deterministic database UUID.
`BlinkStore::open_path` selects the page-image WAL path. The service enables
planned execution; it does not enable the parallel worker pool. The per-tenant
coordinator uses `CoordinatorConfig::default()`: queue capacity 256, up to 64
logical requests and 4 MiB per group, and zero collection delay. It collects
already-runnable requests after one scheduler yield. `apply_transaction_group`
performs the existing condition evaluation, revision allocation, atomic
publication, and durability sync for those requests.

The tenant map mutex is held while looking up or opening a shard. Mutation work
is outside that map lock: each tenant has its own queue/coordinator, and each
group is dispatched through `spawn_blocking`. No service-wide write mutex
serializes different tenants. The open-shard limit is the existing
`LocalTenantServiceConfig` default of 1,024; eviction and a hard process memory
limit are not implemented by this adapter.

## Common checks

The existing read and transaction contract bodies now create the B-tree,
B-link, and RocksDB services with file-backed paths and apply the same requests
and independent expectations to each backend. No B-link-specific relaxed
expectations or numeric cross-backend revision comparisons were introduced.

The common read contract checks binary values and keys, key order, Query and
Scan exclusive cursors and pagination, tombstone omission and reinsertion,
zero and bounded limits, response budgets, four-tenant isolation, and a
Query-versus-multi-key-commit snapshot. The transaction contract checks
condition success and failure, unchanged state after a failed condition,
widths 1/4/8, multi-key atomicity and shared transaction revision, conditional
races, missing/delete/reinsert ABA, tenant isolation, and normal shutdown and
reopen.

The shared tests are `dodb/crates/dodb-rocksdb/tests/read_contract.rs` and
`dodb/crates/dodb-rocksdb/tests/transaction_contract.rs`. Local verification
used Rust/Cargo 1.97.1 on macOS ARM64 and the pinned local RocksDB checkout at
`abeebd9630f11bd08c28b7bd43c7bdfc62050654`:

```sh
cargo +1.97.1 fmt --package dodb-storage --package dodb-server --package dodb-rocksdb -- --check
DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 \
  cargo +1.97.1 test --locked -p dodb-rocksdb \
  --test read_contract --test transaction_contract
```

Result: the read integration target passed 2 tests and the transaction target
passed 1 test on macOS ARM64.

The OCI implementation verification used clean detached source SHA
`8dc4f3b7565f5bb0b963f782721378a55616f516` on host
`instance-20260923-1013`, native AArch64 Linux, Rust/Cargo 1.97.1, and the
repository Release profile with `+crc`. The exact helper command was:

```sh
DODB_TEST_DATA_DIR=/bench/zfs/db/dodb-blink-service-8dc4f3b7/test-databases/contract-run-01 \
DODB_BUILD_RUN_ID=blink-service-contract-8dc4f3b7-01 \
  bash dodb/scripts/run-oci-rocksdb-build.sh \
  test --locked --release -p dodb-rocksdb \
  --test read_contract --test transaction_contract
```

The DB fixture parent was on the `zfs` filesystem mounted at
`/bench/zfs/db`; each temporary backend directory was created under that path
and cleaned by its test fixture. Both targets passed: 2 read tests and 1
transaction test. Cargo reported 82.65 seconds. The resulting ELF hashes are:

| Test executable | SHA-256 |
| --- | --- |
| `read_contract-c907033e9384b264` | `0c8bf91d4ceb6e78f31a2e8a5ce29f6949921207de9ef77fa616b5e25ff09b80` |
| `transaction_contract-832acd252ecc23fe` | `af55ea35d7c5f00e88087a28700efa6a0167adc0cb3b36f03332f557e3d93bf7` |

The existing shared Cargo target was reused at
`/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target`;
the sccache server and shared cache directory were also reused. The shared
sccache counters increased from 14 to 21 misses and from 20 to 30 compile
requests during this command window, with no cache hits; because the counters
are shared with other builds, they are not attributed to this run alone. Cargo
rebuilt the changed dodb crates only, not the full dependency graph. RocksDB
11.8.1 at `abeebd9630f11bd08c28b7bd43c7bdfc62050654` was not rebuilt. The
reused `librocksdb.a` SHA-256 was
`9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
Complete helper logs and checksums remain on OCI at
`/bench/zfs/db/dodb-rocksdb-build-runs/blink-service-contract-8dc4f3b7-01/`.

An earlier OCI run at source SHA `3f8d5180ea61a77a59d7beb1ce946af67ebc97eb`
also passed the same contracts but created test DBs below the helper's `/tmp`
directory. It is preserved at
`/bench/zfs/db/dodb-rocksdb-build-runs/blink-service-contract-3f8d5180-01/`
and is not counted as the OCI ZFS verification. Commit `6bd01f0c` adds the
explicit test-data-root setting; `8dc4f3b7` preserves queued requests at the
coordinator group byte limit.

## RocksDB wrapper and scheduling difference

`TenantDatabase.handle` is a `std::sync::Mutex<NonNull<c_void>>`. Point Get
holds it during the RocksDB call. Transaction execution acquires it before
`dodb_rocksdb_begin` and holds it through sorted `GetForUpdate` calls,
condition evaluation, writes, and commit or rollback. Query/Scan hold it while
creating the iterator and snapshot, then release it while iterating. This is a
wrapper-level per-tenant serialization in addition to RocksDB's own
TransactionDB key locks.

Every mutating transaction also calls `GetForUpdate` on metadata key
`0x00 0x01`, increments the persisted tenant revision counter, and commits that
counter with the requested mutations. The counter serializes revision
allocation for successful mutations in a tenant inside TransactionDB. The
wrapper mutex is broader: it serializes otherwise independent writes and
overlaps reads with a tenant's transaction from the wrapper's point of view.
The service has a global semaphore of eight blocking operations and runs those
operations on Tokio's blocking pool. Separate tenants have separate database
handles and can use those slots concurrently.

B-link reads pin immutable published generations without taking the writer
queue. Its per-tenant coordinator serializes logical writes in request order,
but can place several logical transactions under one physical WAL sync. The
current RocksDB wrapper holds one tenant mutex through each synchronous
TransactionDB commit; it does not collect several service transactions into a
shared sync group. Both successful write responses wait for the backend's
durability path, while batching and concurrency differ inside the backends.

Do not use the current RocksDB wrapper as evidence that B-link is generally
faster. Before treating it as a competitive RocksDB baseline, the first narrow
wrapper change should remove the `handle` mutex from ordinary concurrent
operations while retaining safe handle lifetime and the existing
TransactionDB `GetForUpdate` locks. Keep the revision counter transaction and
measure its effect before proposing a revision redesign. The service's eight
blocking-operation limit remains an explicit resource setting. This isolates
the wrapper's broad serialization without changing public semantics or
rebuilding the backend architecture.

## Existing workload harness and remaining gap

The existing `phase0-bench` and the historical
`blink-rocksdb-workload-comparison` use storage-engine benchmark entrypoints.
They call B-link storage APIs such as `BlinkStore::apply_transaction_group`
and a RocksDB benchmark adapter; they do not instantiate B-link and RocksDB
through these three `DodbService` constructors. The earlier comparison uses
different reader execution models and a synced RocksDB WriteBatch path. It
does not exercise the `RocksDbService` TransactionDB conditions, persisted
revision counter, service response budgets, or the same API call accounting.
Those artifacts remain engine diagnostics and cannot fill product comparison
records.

The repository does not yet contain a service-level stateful workload driver
that can:

- run both backends through identical `DodbService` requests;
- include Get, Query16, unconditional and conditional writes, and atomic width
  four/eight transactions in the frozen workload recipe;
- count public API attempts, successful write transactions, mutations,
  expected condition rejections, conflicts, and preparation Gets separately;
- charge a revision-preparation Get to the same offered API-call stream;
- pair backend runs with the same seed and logical input dependencies; and
- run one service instance with several active tenant files and report each
  tenant's outcomes.

## Initial OCI comparison table

Do not run this table until the service-level driver exists and its input and
accounting smoke passes. These ten records are one short preliminary round,
not a stability campaign:

| Record group | Profiles per backend | Backends | Clients | Active tenant files | Rows per tenant | Seed / request ceiling | Warmup / sample / repetitions | Count |
| --- | --- | --- | ---: | ---: | ---: | --- | --- | ---: |
| Single tenant | RW95, RW80, RW50, RW20 | B-link, RocksDB | 16 | 1 | 10,000 | `20261013` / 1,000,000 public calls | 1 s / 5 s / 1 | 8 |
| MT-MIX | RW95, RW80, RW50, RW20 run concurrently on four tenants | B-link, RocksDB | 4 per tenant, 16 total | 4 | 10,000 each | `20261013` / 1,000,000 public calls | 1 s / 5 s / 1 | 2 |

Use the existing workload recipe: 16-byte combined raw keys, 512-byte changing
values, Get/Query16 read weighting, Put/Delete/conditional/width-four/width-
eight write weighting, and the 10% deliberate condition rejection target.
Use the same closed-loop request generator, fixed seed, and deterministic
per-tenant substreams for each backend pair. The 1,000,000-call limit counts
public calls, including preparatory Gets; charge those Gets to the read ratio
and workflow time. Record setup separately from measurement.
No B-tree third measurement target is included. Keep MT-IDLE and MT-NOISY as
later workloads, and do not claim LSM steady state from these short runs.

## Build provenance and next work

The repository pins Rust/Cargo 1.97.1. OCI uses the existing Release profile,
`+crc`, shared Cargo target, sccache, and pinned RocksDB 11.8.1 archive through
`dodb/scripts/run-oci-rocksdb-build.sh`. The helper must be invoked from a clean
checkout at the exact implementation SHA with a unique `DODB_BUILD_RUN_ID`;
each run has separate logs, database paths, and temporary paths. It must reuse
the existing Cargo cache and RocksDB archive rather than create a per-SHA
target or rebuild RocksDB.

The next smallest implementation task is the service-level stateful workload
driver described above. It should expose B-link and RocksDB service factories,
implement only the frozen ten-record workload, and emit the accounting and
manifest fields needed to validate paired OCI runs. This report establishes
service readiness; it does not establish throughput, latency, or a backend
winner.
