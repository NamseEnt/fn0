# Single-instance transactional database research

Status: active research plan, established 2026-10-12. The foundation inventory
is complete; backend implementation and new product measurements are pending.
Inspected source: `7785241d11cf909111c1226afd3bed666f010cd4`.

## Product objective

Build a high-performance single-instance database that provides selected
DynamoDB-style capabilities through dodb's own endpoint and API: KV access,
ordered short queries, atomic multi-key writes, and conditional writes. One
native process must manage many independent tenant databases and their files.
Evaluate successful application work, latency, resource use, and interference
between tenants.

A DynamoDB-compatible endpoint is not currently planned. Selected capabilities
follow dodb's documented contract; AWS HTTP protocol compatibility is not a
product goal or a requirement for this research.

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
cross-file atomic transactions require a separate design. DynamoDB-style
capabilities do not imply compatibility with DynamoDB endpoints, expression
language, transaction limits, or idempotency behavior.

## Verified foundation

| Area | Existing implementation | Consequence |
| --- | --- | --- |
| Service boundary | `DodbService::execute(TenantId, Request, ExecutionBudget)` | Reuse the existing API boundary for both backends. |
| Public requests | Get, Put, Delete, Query, Scan, Transact | Preserve response and error semantics instead of defining a benchmark-only API. |
| Transaction input | `TransactionRequest` with conditions and Put/Delete mutations | KV-only WriteBatch measurements do not cover the product. |
| Conditions | RevisionEquals, Exists, NotExists | Both backends need atomic validation and application. |
| Missing documents | Missing carries a revision; untouched missing is revision zero | Persist deletion identity and detect missing/insert/delete ABA. |
| Native tenant service | `LocalTenantService`, lazy opening, one tenant/one shard by default | Multi-database routing exists, but high-performance tenant management is unproven. |
| Native service storage | Hardwired `BTreeStore` and `AsyncShard` | Planned Blink is not already a selectable backend of this service. |
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

Use two primary backends per measurement round. Initially `dodb-native` is the
existing B-tree service. Promote a declared Planned Blink configuration into the
same service boundary only after it passes the same contract. Keep exact engine
selection in every manifest; do not relabel B-tree as Blink. Previous native
versions are controls for specific changes, not an automatic third engine in
every product matrix. Turso is an optional later comparator with the same gate.

Service execution is the first product comparison boundary. Run through the same
QUIC server/client afterward to expose transport and request-budget costs. Keep
raw engine calls as diagnostics, with their boundary clearly named.

## Staged work and progress

| Stage | Status | Deliverable | Exit condition |
| --- | --- | --- | --- |
| R0: foundation | Complete for this inventory | This roadmap, backend contract, workload definitions, policy cross-reference | Existing behavior and proposed work are separated; required mixes and multi-tenancy are explicit. |
| R1: equivalent backend | Partial; R1a and R1b-1 slices implemented, remaining R1 work pending | Experimental `dodb-rocksdb` service, shared read contract coverage, format/revision design notes | KV, Query/Scan, conditional and multi-key writes, tenant isolation, reopen, and process-crash cases pass on both backends. |
| R2: product driver | Pending | Stateful application-workload driver through `DodbService`; one process with multiple DBs | Logical input equivalence, actual operation accounting, correctness, resource bounds, and all mandatory profiles pass short smoke. |
| R3: first product evidence | Pending | Small single-tenant and multi-tenant OCI results, followed by Debian | All mandatory mixes are present, per-tenant behavior is visible, source/binary/host provenance is valid. |
| R4: focused improvement | Pending | One measured bottleneck, one implementation change, before/after evidence | Same contract and frozen workloads pass; gains and regressions are recorded. |
| R5: confirmation | Pending | Targeted repetitions, sustained storage behavior, bursts, cold-tenant and capacity checks, common transport | Claims survive observed noise, DB lifecycle transitions, and meaningful storage activity. |

R1 remains incomplete until its remaining implementation and correctness gates
pass. Complete its code and correctness evidence before launching a broad
performance matrix. R2 must start with the
small profiles below and must not revive the previous 126-run plan by default.
Update this table after each deliverable with source SHA, artifact path, actual
checks performed, and remaining gates. Documentation alone does not complete R1.

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
R1 remains incomplete. Common transaction conformance, tenant lifecycle and
capacity management, process-kill recovery, and the serialization bottleneck
before performance comparison remain open. No performance campaign was run.

R1a source SHA `f7325bc24a46afe0912e200ae8a08b9275157c6e` passed the disk-backed
integration test locally on macOS ARM64 and in an OCI A1 native ARM64 Release
build. The parent SHA `e5a4692c229dcfe211db02b58552e3da63a8bff2` passed five
consecutive local repeats of the cold-open and concurrency suite. The OCI run used a clean detached
checkout and the pinned RocksDB 11.8.1 archive; the
[run report](../experiments/results/rocksdb-service-r1a-70105e0f/REPORT.md)
records commands, provenance, failed-attempt correction, and coverage.
Common transaction conformance, full lifecycle management, process-kill
recovery, and all other R1 exit conditions remain pending after R1b-1.

### R1 implementation order

1. Add an experimental backend module/crate implementing existing `DodbService`.
   Keep transport and protocol storage-independent. Pin RocksDB 11.8.1 initially
   to retain build provenance; use native builds on each measurement host.
2. Specify value/tombstone encoding, revision allocation and restart behavior,
   transaction locking, conflict mapping, and request/response limits.
3. Implement Get/Put/Delete/Query/Scan and single-shot Transact with conditions.
   Start from a supported RocksDB transaction implementation; document any
   performance-oriented alternative and prove the same semantics first.
4. Reuse/extract backend-neutral correctness cases. Include a condition on an
   unmodified key, absent-key ABA, races with unconditional writes, and atomic
   failure. Do not add a global cross-tenant write mutex for convenience.
5. Expose tenant DB open/close/shutdown through a small reusable factory/registry
   boundary. Keep the native and RocksDB handles internal to their backends.
6. Establish process-kill/reopen recovery evidence for acknowledged requests
   and all-or-none unacknowledged transactions, with independent expectations.

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

The first single-tenant evidence has four mixes (95/5, 80/20, 50/50, 20/80) and
two backends: eight selected records per host. The transaction/conditional mix
is defined in [workloads-v1.md](workloads-v1.md). Five-second, one-repetition
runs are preliminary only. Keep query correctness and width-eight transactions
in conformance coverage and include them in the tenant scenarios; do not expand
every dimension into a Cartesian product.

The first multi-tenant evidence has three scenarios and two backends: six
selected records per host. Run OCI first, then Debian. Together the first
product round is 14 records per host, at most 28 across both hosts, excluding
correctness/smoke checks and narrowly justified retries. Setup may dominate
wall time; report setup and measured time separately. Reuse only genuinely
equivalent results. Existing quick KV results cannot fill new conditional or
multi-tenant cells.

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
