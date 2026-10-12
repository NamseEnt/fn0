# Backend contract for the product comparison

Status: R0 specification, 2026-10-12. This describes the common contract and the
next RocksDB implementation. It does not claim a RocksDB-backed dodb exists.
See [the roadmap](single-instance-product-roadmap.md) for execution order.

## Existing API boundary

Implement `dodb_service::DodbService` for `dodb-rocksdb`. Reuse `TenantId`,
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

Ordinary separate reads do not promise a common snapshot across calls. Query
and Scan must not expose a mixture of one transaction's partial publications.
The first comparison does not add cross-tenant atomicity or range predicates.
Reading an uncreated tenant returns the native empty/missing results without
creating storage. A condition-only request on an uncreated tenant also follows
the native missing-state semantics without creating a database just to check it.

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

Revisions are database-scoped identities used for equality. They are not
portable RocksDB sequence numbers or required to equal native physical WAL
LSNs. Match observable behavior, not the numeric spacing of native LSNs.
Persist enough information that acknowledged identities survive restart and
old tokens never become valid for a different logical state after deletion,
recreation, checkpointing, or compaction. Record commit-token allocation and
ordering guarantees separately from per-key revision equality.

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
Their presence does not supply dodb key encoding, revisions, API errors, tenant
lifecycle, or DynamoDB request translation automatically.

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
| Process crash | Acknowledged writes survive kill/reopen; interrupted transactions are absent or complete; independent expected history. |

The deterministic reference can replay a backend's recorded serialized logical
commits; cross-backend tests should normalize opaque identities rather than
require matching physical LSN numbers. Concurrent checks must account for
allowed histories, not assume the value is unchanged while another writer runs.
Independent parent/oracle state is required for recovery checks. Process kill
does not remove kernel page cache or establish power-loss safety.

## DynamoDB compatibility inventory

Before exposing a DynamoDB-compatible endpoint, record exactly which actions,
conditions, errors, limits, key/value types, and retry semantics are supported.
Existing dodb Transact is a semantic primitive; it is not an implemented AWS
TransactWriteItems endpoint. Existing conditions are revision/presence checks,
not an attribute-expression parser. Existing opaque values are not an AWS
AttributeValue implementation.

Translate only the declared subset into the shared service contract, identically
for both backends. Full Update expressions, ClientRequestToken idempotency,
cross-table transactions, and other AWS behavior require explicit future scope.
Unsupported operations must be reported as unsupported rather than partially
executed. Consult the
[AWS operation reference](https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_TransactWriteItems.html)
when writing that mapping. The first backend comparison exercises the current
dodb contract and does not require building a complete DynamoDB emulator.
