# Backend contract for the product comparison

Status: the common read/transaction service contract and RocksDB wrapper
concurrency/lifetime checks were recorded as passing for B-tree, B-link, and
RocksDB at implementation verification SHA `139d9ae40120e35aad226bb13830c9347e297595`,
2026-10-13. This historical result does not establish a performance baseline
or report checks rerun for this contract audit. The existing
R1b-3b WAL-sync-boundary all-or-none process-kill result is preserved; neither
statement claims that every lifecycle or crash case is complete. See
[the roadmap](single-instance-product-roadmap.md) for execution order and the
[B-link comparison readiness report](blink-service-comparison-readiness.md)
for service wiring and OCI provenance. The earlier
[R1b-1 report](../experiments/results/rocksdb-service-r1b1-2774bd1c/REPORT.md)
records the separate read implementation slice. R1b-2 transaction conformance
is in the [R1b-2 report](../experiments/results/rocksdb-service-r1b2/REPORT.md),
and R1b-3a is in the
[R1b-3a report](../experiments/results/rocksdb-service-r1b3a-20261013-local/REPORT.md).
R1b-3b's native and RocksDB WAL-sync-boundary checks are recorded in the
[R1b-3b report](../experiments/results/rocksdb-service-r1b3b-20261013-local/REPORT.md).

## Existing API boundary

Implement `dodb_service::DodbService` for each compared backend. The existing
services are `LocalTenantService` backed by B-tree,
`BlinkTenantService` backed by B-link, and `RocksDbService`. Reuse `TenantId`,
`DocumentKey`, `Request`, `Response`, `TransactionRequest`, `ExecutionBudget`,
and application errors. The same transport must be able to call either service.
An engine-only adapter returning a bool for writes is insufficient.

| Operation | Required observable behavior |
| --- | --- |
| Get | Present(value, revision) or Missing(revision); never pair a value with another version's revision. |
| Put/Delete | Durable successful response with a revision; deletion retains a logical missing-state revision. |
| Query | Same PK, canonical key order, exclusive SK cursor, limit, skip logical tombstones, consistent view within this operation. |
| Scan | Canonical document order, exclusive document cursor, limit, consistent view within this operation. |
| Transact | Single-shot evaluation of all point conditions and atomic application of all mutations in one tenant DB. |
| Shutdown | Stop new work and complete service-owned work/handle cleanup under documented drain semantics. |

The operation boundary is the existing `dodb_service::Request` and `Response`
API; this comparison adds no response field or wire conversion. `Get` returns
the present value with its revision, or the missing state with its revision.
`Put` and `Delete` return the revision assigned by their successful write.
`Query` selects one partition key and returns canonical sort-key order after
the exclusive sort-key cursor. `Scan` returns canonical document-key order
after the exclusive document-key cursor. Both honor `limit`, omit logical
tombstones, and give each individual call one consistent snapshot. Different
calls or pages do not share a snapshot. Execution budgets must be checked before
copying large result values into a response.

`Transact` supports `RevisionEquals`, `Exists`, and `NotExists`, with `Put` and
`Delete` mutations. It rejects empty requests, duplicate conditions, and
duplicate mutations; a condition and mutation may name the same key. It
protects the canonical union of condition and mutation keys through decision
and commit, evaluates every condition before applying any mutation, and
applies all mutations atomically. A successful write assigns one revision to
every changed key and returns a nonzero logical `commit_lsn`. A successful
condition-only request returns `commit_lsn: None` and allocates no revision.
Initial missing state has revision zero; deleted keys retain a nonzero missing
revision. For a given key, issued revisions identify committed states and are
not reused after delete/recreate or restart. Gaps are allowed.

Ordinary separate reads do not promise a common snapshot across calls. Query
and Scan must not expose a mixture of one transaction's partial publications.
The first comparison does not add cross-tenant atomicity or range predicates.
Reading an uncreated tenant returns the native empty/missing results without
creating storage. A condition-only request on an uncreated tenant also follows
the native missing-state semantics without creating a database just to check it.

R1b-1 implements Query and Scan with one RocksDB snapshot per request. The
snapshot is attached to the iterator's `ReadOptions`; a request does not retain
that snapshot across calls or pages. Read response budgets reject oversized
results without truncating them and are reserved per live document before its
value bytes are copied into the response. The RocksDB cursor seeks use the
canonical escaped `DocumentKey` encoding and RocksDB's bytewise comparator.

## RocksDB handle ownership and concurrency

The tenant-wide mutex around the raw RocksDB DB pointer has been removed.
`RocksDbHandle` is shared through `Arc`; the pinned RocksDB 11.8.1 `DB` API
documents concurrent access from multiple threads without external
synchronization, and `TransactionDB` derives from that DB hierarchy. A separate
RocksDB `Transaction` is created for each request and is not shared. Its guard
retains the DB owner through commit or rollback and transaction destruction.
Query/Scan iterators retain the DB owner until the iterator and its snapshot
have both been released. This means clearing the tenant registry cannot close a
DB while a request, transaction, or iterator still owns it.

The short lifecycle state lock admits requests and tracks active execute
futures; it is not held during database work. Shutdown closes admission, waits
for active request leases, then drains the unchanged eight-slot blocking
semaphore before clearing the registry on Tokio's blocking pool. This also
waits for already-started blocking work whose request future was cancelled. The
prior reusable-service behavior is retained: a later execute can reopen the
tenant. A spawn-blocking closure also owns the DB `Arc`, so cancellation of its
awaiting future does not free the handle before the blocking call finishes.

The focused concurrency check pauses a transaction immediately after the
actual `BeginTransaction` call and confirms a same-tenant Get returns a
committed value before that transaction is released. Another check overlaps a
service Get with shutdown, confirms the request completes, and reopens the same
tenant afterward. A third cancels the request future after its RocksDB Get has
started and confirms shutdown waits for that blocking operation before the
tenant can be reopened. The existing shared read and transaction contracts
still pass, including Query/Scan snapshot consistency. These are correctness
checks; they do not measure performance.

TransactionDB key locks acquired by `GetForUpdate` remain responsible for
condition and mutation key protection. Every mutating transaction also locks
and updates the persisted tenant revision counter in the same transaction, so
successful write commits can remain serialized per tenant by that counter.
This change does not redesign revision allocation or establish parallel
independent write commits. It also does not change the blocking-operation
limit.

## Transaction validation and state identity

Use existing input validation: empty requests are invalid; duplicate condition
keys and duplicate mutation keys are each invalid. A condition and a mutation
may name the same key. Conditions can also name keys not mutated by the request.
Evaluate conditions against one serialization point before applying mutations.

| Condition | Meaning |
| --- | --- |
| RevisionEquals | Exact revision equality for present or missing logical state. |
| Exists | Logical document exists, independently of revision. |
| NotExists | Logical absence, including deleted documents with nonzero revision. |

A condition mismatch returns the existing structured key/expected/actual
conflict and applies zero mutations. Condition-only success returns no logical
commit identity. All changed keys of a successful transaction share its logical
revision. Independently committed writes remain independent transactions even
when the storage engine shares a sync.

Per-key revisions are identity tokens used for equality. `commit_lsn` is the
existing API and wire field for a successful write transaction's logical
identity within its tenant/backend identity domain. Neither value is a
portable RocksDB WAL sequence, timestamp, or snapshot handle. Do not claim
global uniqueness across tenants, numeric ordering across independent keys, or
that response completion order for overlapping requests matches token order.
The native B-tree/B-link coordinator happens to use increasing physical LSNs;
that implementation detail does not require a RocksDB wrapper to reconstruct
native physical order. Keep same-key revision progression and committed
read-after-success visibility intact. Separate Get calls or pages remain
independent reads, while each Query and Scan call retains its own consistent
snapshot. These clarifications retain the existing consumer-visible API and
atomicity; they do not permit weaker visibility.

Persist enough information that acknowledged identities survive restart and
old tokens never become valid for a different logical state after deletion,
recreation, checkpointing, or compaction. Record token allocation guarantees
separately from per-key revision equality.

## Proposed RocksDB implementation

Start with RocksDB 11.8.1 at
`abeebd9630f11bd08c28b7bd43c7bdfc62050654` and a supported transaction API.
TransactionDB plus GetForUpdate is the initial candidate for atomic conditions;
OptimisticTransactionDB is an alternative to evaluate, not a silently mixed
configuration. Read the pinned headers before selecting binding capabilities.

Required design decisions before claiming conformance:

1. Store revision and value/tombstone together so they commit atomically. A
   physical Delete alone loses missing-state identity. Keep internal metadata
   outside user key space and exclude it from Query/Scan.
2. Protect every condition key, including missing and unmodified keys, until
   the decision/commit. Acquire the required key locks in a deterministic order
   while preserving request-order conflict reporting where the contract demands
   it. Unconditional Put/Delete must participate in the same conflict discipline.
3. Validate conditions and apply all writes in one transaction. A Get followed
   by an unrelated WriteBatch permits a race and is not a conditional write.
4. Commit successful mutations with WAL enabled and sync requested. Account for
   revision metadata and all required initialization/reservation writes. Do not
   hide extra syncs outside the measured application workflow.
5. Design recoverable revision allocation. Document gaps, overflow, concurrency,
   committed ordering, and restart. Do not use a racy GetLatestSequenceNumber
   after commit as the transaction's own revision. A shared counter key held
   through every sync can serialize unrelated writes: do not accept that as the
   final performance baseline without a recorded design evaluation.
6. Use one stable read snapshot for each Query/Scan, retain it until iteration
   ends, and release it. Preserve response-budget checks before materializing
   unbounded values. Do not use only a nontransactional iterator over live data
   and claim atomic read visibility.
7. Map predicate failures, concurrency/lock outcomes, overload, invalid input,
   I/O failures, and uncertain commit outcomes explicitly. Do not convert every
   RocksDB failure to condition failure or hide internal retries. A precise
   actual state in a conflict response must come from the protected observation,
   not an unrelated later Get.
8. Keep blocking RocksDB calls off asynchronous runtime worker threads using
   a bounded execution facility. Record wrapper scheduling overhead. No
   cross-tenant global write lock or unlimited thread creation.

RocksDB's transaction primitives are documented in its
[official transaction guide](https://github.com/facebook/rocksdb/wiki/Transactions).
Their presence does not supply dodb key encoding, revisions, API errors, or
tenant lifecycle automatically.

## Tenant databases and shared resources

One tenant maps to an independent persistent DB unit. Multiple tenants must
share one process and have disjoint key/value/revision state. A RocksDB directory
per tenant satisfies this topology; a column family or key prefix inside one DB
is a different topology and needs a separate decision.

Define a reusable backend factory and registry for lazy open, handle pinning,
close/reopen, maximum open databases, shutdown, and instance-level CPU/memory/
I/O budgets. Existing native lazy-open code is a starting implementation, not
evidence of fair eviction or isolated cold opens. Avoid holding a process-wide
registry lock through disk I/O. Requests to the same tenant must not race into
duplicate engine handles during open/close.

Reuse RocksDB shared block cache, WriteBufferManager, and bounded background
resources where supported. Record their configuration, memory outside those
budgets, and effects on other tenants. An isolated cache per tenant with no
instance bound is not the target multi-tenant baseline.

## Reusable conformance inventory

Implement one backend-neutral suite over service factories. Run it on the
existing native service and RocksDB service; add the Planned Blink service
adapter to the same suite when implemented. Reuse current reference and QUIC
cases instead of inventing a second semantic model.

| Case | Required evidence |
| --- | --- |
| Binary key/value and ordering | Empty/zero-containing keys, full values, exact canonical Query/Scan order and cursor boundaries. |
| Read/write state | Put, repeated update, Delete, repeated Delete, untouched missing and deleted missing; reopen preserves state. |
| Conditions | Exists, NotExists, current/stale RevisionEquals on both present and missing states. |
| Atomic multi-key mutation | Widths 1/2/4/8; independent oracle confirms all writes or no writes, including mixed Put/Delete. |
| Unmodified read dependency | A condition on key A protects writes to B/C when A changes concurrently. |
| Conflict atomicity | Stale condition on any key leaves every requested mutation unapplied. |
| Conditional races | Two requests using one expected revision cannot both succeed; unconditional writes participate correctly. |
| Missing ABA | Observe missing, insert, delete; stale missing revision fails while NotExists succeeds. |
| Condition-only request | Pass/fail behavior; no logical commit and no user data mutation. |
| Invalid request | Empty request and duplicate keys rejected before partial mutation. |
| Per-call snapshot | Query/Scan concurrent with multi-key writes never exposes a torn transaction. |
| Limits and errors | Same request/response budget and conflict encoding behavior; large values not materialized just to report revision conflict. |
| Tenant isolation | Same document key in different tenant DBs has independent state; one process serves them concurrently. |
| Lifecycle | Cold open, duplicate concurrent open, close with in-flight work, reopen, configured capacity, shutdown. |
| Process crash | Acknowledged transaction survives kill/reopen; interrupted transactions are absent or complete; independent expected history. |

The deterministic reference can replay a backend's recorded serialized logical
commits; cross-backend tests should normalize opaque identities rather than
require matching physical LSN numbers. Concurrent checks must account for
allowed histories, not assume the value is unchanged while another writer runs.
Independent parent/oracle state is required for recovery checks. R1b-3a
verifies one acknowledged width-four transaction after SIGKILL and reopen on
each backend; interrupted-transaction recovery and the rest of the
process-crash inventory remain open. Process kill does not remove kernel page
cache or establish power-loss safety.

## Independent dodb API scope

The product goal is to provide selected DynamoDB-style data-operation
semantics through dodb's own API. AWS HTTP API compatibility is outside current
scope and is not a future product milestone.

Both backends must implement the existing shared dodb service contract with the
same actions, conditions, errors, limits, key/value types, and retry semantics.
Existing Transact provides atomic mutations with revision/presence conditions;
values remain opaque bytes. These capabilities do not imply support for AWS
request formats, attribute expressions, AttributeValue encoding, transaction
limits, or ClientRequestToken behavior.

Unsupported dodb operations must be reported as unsupported rather than
partially executed. Product comparisons exercise dodb's contract, including
conditional and multi-key writes, without requiring DynamoDB protocol adapters
or AWS-specific features.
