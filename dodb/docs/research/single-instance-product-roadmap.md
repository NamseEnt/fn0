# Single-instance transactional database research

Status: active research plan. The common service contract now has B-tree,
B-link, and RocksDB implementations; a service-level product workload driver
and product measurements remain pending. The latest historical inventory is
`7785241d11cf909111c1226afd3bed666f010cd4`; the B-link service connection is
recorded in [the readiness report](blink-service-comparison-readiness.md).

## Product objective

Build a high-performance single-instance database that provides selected
DynamoDB-style capabilities through dodb's own endpoint and API: KV access,
ordered short queries, atomic multi-key writes, and conditional writes. One
native process must manage many independent tenant databases and their files.
Evaluate successful application work, latency, resource use, and interference
between tenants.

Selected capabilities follow dodb's documented contract. AWS HTTP API
compatibility is outside product scope and is not a future milestone.

The primary alternative is **the same dodb API backed by RocksDB**. A successful
research outcome can select the native engine, the RocksDB backend, or different
documented operating profiles. Native-engine victory is a hypothesis to test.

## Authority and scope

This plan governs the product comparison and execution order. It extends
[the product workload policy](../experiments/fn0-product-workload.md), whose
historical engine studies and internal regression rules remain evidence.
[The backend contract](backend-contract.md) and
[workload definitions](workloads-v1.md) are required inputs to implementation.
Changes to their mandatory operations, ratios, or tenant scenarios require an
explicit recorded user decision. Faster turnaround reduces repetitions and
duration before it reduces product coverage.

The initial transaction boundary is one tenant database. Cross-tenant or
cross-file atomic transactions require a separate design. DynamoDB-style data
operation semantics do not imply AWS HTTP API compatibility.

## Verified foundation

| Area | Existing implementation | Consequence |
| --- | --- | --- |
| Service boundary | `DodbService::execute(TenantId, Request, ExecutionBudget)` | Reuse the existing API boundary for both backends. |
| Public requests | Get, Put, Delete, Query, Scan, Transact | Preserve response and error semantics instead of defining a benchmark-only API. |
| Transaction input | `TransactionRequest` with conditions and Put/Delete mutations | KV-only WriteBatch measurements do not cover the product. |
| Conditions | RevisionEquals, Exists, NotExists | Both backends need atomic validation and application. |
| Missing documents | Missing carries a revision; untouched missing is revision zero | Persist deletion identity and detect missing/insert/delete ABA. |
| Existing native service | `LocalTenantService`, lazy opening, one tenant/one shard by default, backed by `BTreeStore` and `AsyncShard` | Keep as the regression and diagnosis path. It is not the B-link measurement backend. |
| B-link service | `BlinkTenantService` implements `DodbService`, opens one file/WAL pair per tenant, and batches that tenant's mutations through `BlinkStore::apply_transaction_group` | It is now eligible for the common service comparison after the same contract checks. |
| Open database limit | Default 1,024; full map returns Overloaded | Existing code does not establish idle eviction or bounded memory per instance. |
| Tenant registry | Mutex held while opening a new shard | Cold-open interference is a concrete investigation target, not yet a measured bottleneck. |
| Reference semantics | `ReferenceDb` and transaction/server tests | Reuse the semantic oracle and move reusable cases to a backend contract suite. |

Source pointers:

- [Service types](../../crates/dodb-service/src/lib.rs).
- [Transaction input validation](../../crates/dodb-core/src/transaction.rs).
- [Revision and missing state](../../crates/dodb-core/src/revision.rs).
- [Local tenant implementation](../../crates/dodb-server/src/lib.rs).
- [Transaction semantics](../architecture/03-transactions.md).
- [Reference model](../../crates/dodb-testkit/src/reference.rs).
- [Existing service integration cases](../../crates/dodb-server/tests/phase6_quic.rs).

This inventory describes inspected behavior. The contracts below describe work
still needed to make the product alternatives comparable.

## What existing measurements establish

The [quick comparison](../experiments/results/crossdb-quick-preliminary-2026-10-12/REPORT.md)
measured one database, 10,000 uniformly accessed rows, width-one unconditional
writes, Get, Query16, and a 50/50 mix. The RocksDB path used ordinary DB writes
with synced WriteBatch, not its transaction API. Those runs remain useful engine
diagnostics. They do not answer conditional multi-key performance or native
multi-tenant behavior. Their single repetition and short intervals do not
establish steady-state LSM performance.

Earlier 95/5, 50/50, and 20/80 studies remain historical evidence. Their ratios
must not disappear from the product plan because the latest quick study omitted
them. No production request trace was established by this audit; representative
workloads must be labeled synthetic until observed service distributions or an
authorized trace support a production-replay claim.

## Architecture to compare

```text
same application requests / same dodb client and transport when enabled
                              |
                    DodbService + TenantId
                    /                    \
          dodb-native                 dodb-rocksdb
          native tenant DB files      tenant RocksDB directories
          B-tree or declared Blink    transaction + condition wrapper
                    \                    /
             common backend conformance requirements
```

Use two primary backends per initial measurement round: the B-link service and
the RocksDB service. Keep the existing B-tree service for regression and
diagnosis; do not add it as an automatic third product measurement. Record the
exact service and engine configuration in every manifest. Turso is an optional
later comparator with the same service-contract gate.

Service execution is the first product comparison boundary. Run through the same
QUIC server/client afterward to expose transport and request-budget costs. Keep
raw engine calls as diagnostics, with their boundary clearly named.

## Staged work and progress

| Stage | Status | Deliverable | Exit condition |
| --- | --- | --- | --- |
| R0: foundation | Complete for this inventory | This roadmap, backend contract, workload definitions, policy cross-reference | Existing behavior and proposed work are separated; required mixes and multi-tenancy are explicit. |
| R1: equivalent service contract | Complete at implementation verification SHA `8dc4f3b7565f5bb0b963f782721378a55616f516`; [report](blink-service-comparison-readiness.md) | B-tree, B-link, and RocksDB use the same API request and expected contract body | Reads, writes, conditions, transaction atomicity, budgets, tenant isolation, snapshot reads, and normal reopen pass locally and on OCI ZFS. Additional crash campaigns are not a prerequisite for the first performance comparison. |
| R2: product driver | Pending | Stateful synthetic workload driver through `DodbService`, with B-link and RocksDB factories | Same seeded logical inputs, call/write/rejection accounting, conditional preparation costs, and single-/multi-tenant execution pass a short correctness smoke. |
| R3: first product evidence | Pending; OCI only for the first round | Eight single-tenant records and two mixed-tenant records, one run per condition | Mandatory ratios and transaction actions are present; service, file, WAL, input, binary, host, and timing provenance is valid. |
| R4: focused improvement | Pending | One measured bottleneck, one implementation change, before/after evidence | Same contract and frozen workloads pass; gains and regressions are recorded. |
| R5: confirmation | Pending | Targeted repetitions, sustained storage behavior, bursts, cold-tenant and capacity checks, common transport | Claims survive observed noise, DB lifecycle transitions, and meaningful storage activity. |

Use this stage table to keep service correctness, workload preparation, and
measurement evidence distinct. The first comparison is deliberately small and
does not claim complete tenant lifecycle management or coverage of every crash
mode. Update this table after each deliverable with source SHA, artifact path,
actual checks performed, and remaining gates.

R1a was the first small vertical slice: existing service types,
recoverable revision/tombstone representation, Get/Put/Delete, width-four
conditional Transact, four independent tenant DBs, conflict/ABA checks, and
close/reopen. Its implementation and correctness evidence are recorded in
[the R1a backend note](rocksdb-backend-r1a.md).

R1b-1 adds snapshot-based Query/Scan, shared native/RocksDB read contract tests,
response-budget checks before result value copies, and local/OCI file-backed
verification. The final implementation SHA tested locally and on OCI is
`e59ae32117df13e23cf298d2f8ec6a8fc4c53a93`; the earlier implementation commit
`2774bd1c3491134bbc944c91a8bf34d3bfa22a6e` was also tested before commit
`e59ae32117df13e23cf298d2f8ec6a8fc4c53a93` added the exact-budget success
assertion. Details are in the
[R1b-1 report](../experiments/results/rocksdb-service-r1b1-2774bd1c/REPORT.md).
R1b-2 adds the shared native/RocksDB transaction contract and normal
shutdown/reopen verification. Its tested SHA and result are recorded in the
[R1b-2 report](../experiments/results/rocksdb-service-r1b2/REPORT.md).
R1b-3a and R1b-3b preserve the acknowledged-transaction and WAL-sync-boundary
SIGKILL/reopen results in their existing reports. Do not schedule another crash
campaign as the next required stage. Full lifecycle/capacity policy, additional
crash cases, power-loss evidence, and tenant eviction remain separate future
work; they are not prerequisites for the initial bounded service comparison.
Tenant write scheduling differences are documented in the
[readiness report](blink-service-comparison-readiness.md).

R1a source SHA `f7325bc24a46afe0912e200ae8a08b9275157c6e` passed the disk-backed
integration test locally on macOS ARM64 and in an OCI A1 native ARM64 Release
build. The parent SHA `e5a4692c229dcfe211db02b58552e3da63a8bff2` passed five
consecutive local repeats of the cold-open and concurrency suite. The OCI run used a clean detached
checkout and the pinned RocksDB 11.8.1 archive; the
[run report](../experiments/results/rocksdb-service-r1a-70105e0f/REPORT.md)
records commands, provenance, failed-attempt correction, and coverage.
The earlier R1 notes described the state at each RocksDB implementation slice;
they are historical and do not make additional crash work a prerequisite.
Common transaction conformance and normal reopen are now covered by R1b-2, and
the existing R1b-3a/R1b-3b process-kill results remain preserved. AWS HTTP API
support is not a current or future product requirement.

## Multi-tenant product requirements

A tenant database is a separate persistent storage unit. Native storage uses
its DB/WAL files; RocksDB uses a directory containing its engine files. Do not
require RocksDB to have exactly one physical file, and do not substitute key
prefixes in one shared DB for the independent-DB topology without recording a
separate architecture decision.

The target manager owns a shared CPU/runtime budget, instance memory budget,
bounded open handles, admission queues, and background I/O concurrency. Measure
idle/open/active database counts separately. Tenant close/eviction must pin
in-flight handles, prevent duplicate opens, drain or reject work predictably,
and preserve durability. Reopening an idle tenant must not globally block active
tenants behind slow storage work. These are target requirements, not implemented
guarantees of the current native service.

RocksDB supports shared block-cache and WriteBufferManager objects across DB
instances. Use them to build a credible multi-DB baseline and record their soft
limits and stalls; they are not a hard bound on total process RSS. Native cache
settings also do not by themselves bound total memory. Charge tombstones,
revision metadata, memtables, indexes, queues, runtimes, and background work in
the comparison. Shared resources can cause interference; measure it.

## Fast execution without losing coverage

The initial comparison is limited to B-link and RocksDB on OCI. Its single-
tenant matrix is RW95, RW80, RW50, and RW20 on both backends. One MT-MIX record
per backend runs four active tenants together, one per profile. This makes ten
records total. Each run uses 16 external clients, 10,000 live rows per active
tenant, 16-byte combined raw keys, and 512-byte changing values; the stateful
write recipe and counters are defined in [workloads-v1.md](workloads-v1.md).
Use one-second warmup, five-second measurement, and one repetition. This is a
short preliminary comparison, not an LSM steady-state or long-term stability
claim. The first round is OCI only; Debian is not part of this task.

The existing workload definitions retain MT-IDLE and MT-NOISY as follow-up
profiles. Do not expand the initial ten records into a Cartesian product, but do
not remove conditional writes, widths four/eight, preparatory Get costs, or the
four required read/write ratios. Report setup separately from the five-second
measurement. Existing raw-engine studies cannot fill service-level records.

After the first round, repeat only inconclusive or decision-critical cases.
Sustained LSM evidence must observe flush/compaction and actual data growth;
120 seconds without those events is still insufficient for a steady-state
claim. Larger-than-engine-cache and larger-than-RAM regimes are distinct.
Preserve the required mixes when shortening or staging a round. Do not remove
the multi-tenant scenarios because single-tenant numbers look favorable.

## Evidence and decisions

Report successful application transactions, mutations, reads, expected condition
failures, concurrency conflicts, retries, overloads, and infrastructure errors
separately. Expected conditional rejection is a valid semantic outcome but is
not successful write throughput. Report per-operation and per-tenant p95/p99,
offered load, achieved load, queueing, instance RSS, CPU, file descriptors,
thread counts, bytes written, syncs, flushes, compactions, and stalls where
available. Missing observations are unavailable, not zero.

Use the same deterministic logical workload per backend. A matching seed across
backends within a repetition is required; changing seeds between repetitions
is valid when recorded and paired. Opaque revision numbers may differ across
backends; compare logical references and valid state histories. Do not derive
expected state from the result under test.

Use real files and synced successful writes. Preserve raw logs, invalid attempts,
checksums, clean source SHA, binary hashes, versions, host/filesystem, options,
and commands. Use existing OCI ZFS and Debian storage without changing formats
or deleting existing evidence. Debian may use all available cores, but low CPU
under synchronous I/O is an observation, not an invalid run.

Product superiority requires repeatable benefits on the required profiles with
acceptable per-tenant tails and total resources. Freeze intended latency/load
targets before confirmation using user requirements and measured capacity;
do not invent a production SLO from a microbenchmark. Publish each profile's
trade-offs; a geometric mean or high unconditional write rate cannot hide a
conditional, query, or tenant-fairness regression.

After R3 select one explanation to test: service/tenant management, conditional
validation, commit/sync, native traversal/planning, or RocksDB-wrapper overhead.
Collect attribution evidence and change one mechanism. Do not assume either
that no optimization exists or that native storage will eventually win.

## Primary external references

- [RocksDB transactions](https://github.com/facebook/rocksdb/wiki/Transactions):
  TransactionDB and OptimisticTransactionDB provide transaction primitives; the
  wrapper must implement dodb's API and state identity.
- [RocksDB memory usage](https://github.com/facebook/rocksdb/wiki/Memory-usage-in-RocksDB)
  and [WriteBufferManager](https://github.com/facebook/rocksdb/wiki/Write-Buffer-Manager):
  shared memory resources for multiple engine instances and accounting limits.
