# Phase 3: Optimistic multi-key transactions

## Scope

Phase 3 adds same-shard optimistic transactions whose dependencies are point
keys. A transaction is submitted as one request containing conditions and
mutations:

```text
TransactionRequest {
    conditions: [RevisionEquals | Exists | NotExists],
    mutations: [Put | Delete],
}
```

The request is validated completely before its B+Tree overlay is accepted.
`RevisionEquals` compares the exact current revision for either a present or a
missing key. `NotExists` checks logical absence only, so `Missing(120)` can
satisfy `NotExists` while failing `RevisionEquals(0)`. A successful request
with mutations receives one commit LSN and every changed key receives that LSN
as its new revision. A successful request with conditions but no mutations
returns no commit LSN and changes no state or revision. A request with neither
conditions nor mutations is invalid. A failed condition produces a structured
conflict and applies no mutation.

All point reads that influence a transaction must be carried into the request
as conditions, including reads of keys that are not written and reads that
observed missing state. This provides serializable optimistic validation for
point-key dependencies. The storage engine does not infer dependencies from
arbitrary application code.

## Coordinator and group commit

The single-shard commit coordinator serializes physical B+Tree preparation.
Transaction execution and ordinary reads can happen concurrently before that
point. A coordinator collection window is a physical batch, not a logical
transaction boundary. Phase 4 makes ordinary reads use the published committed
view and changes collection to an evidence-based bounded policy; it does not
change the transaction staging or validation rules below.

Accepted candidates are staged in coordinator order:

```text
durable committed base
    + accepted transaction 1
    + accepted transaction 2
    + ...
```

Each candidate validates against this staged state. Its page images therefore
include the effects of earlier accepted candidates, even when two candidates
modify the same physical leaf page. Each candidate receives its own batch ID,
commit LSN, WAL page-image sequence, and WAL `COMMIT` record. The records may
be appended as one group and followed by one WAL sync. Only after that sync
does the coordinator publish all accepted states in serialization order.

If the group sync fails or its outcome is uncertain, no candidate returns
success and the shard follows the Phase 2 degraded/reopen policy. If the sync
succeeds but publication or responses are interrupted, recovery replays every
complete committed logical transaction in WAL order.

## Read primitives and limitations

The storage coordinator has an internal point-read helper that preserves input
order and reads from one coordinator state. It is not exposed as a public wire
operation. Independent network `Get` calls use separate request streams and do
not promise one common read instant, but their observed revisions can be
supplied later as `RevisionEquals` conditions.

Transactional range queries and scans are not supported. Normal Query and Scan
remain available outside transactions. Phase 3 does not implement MVCC,
historical versions, predicate validation, range locks, a general lock
manager, parallel physical B+Tree writers, distributed transactions,
network idempotency, or a network wire protocol.

## Batch distinction

A physical coordinator batch reduces request overhead and can share one WAL
sync. A logical transaction is the condition-plus-mutation all-or-nothing
unit. The public condition-free multi-write path uses `Transact`; legacy
storage batch APIs remain useful for synchronous storage tests. The async
coordinator treats independent mutation requests as independent logical
transactions.

## Validation coverage

The Phase 3 tests cover write/write and read/write conflicts, missing-key ABA,
deleted-key revisions, existence predicates, insert-if-absent races, atomic
multi-key writes, rollback on a failed condition, same-group staged conflicts,
same-page successive page images, ordered point reads, WAL group durability,
and recovery after publication interruption. Existing B+Tree invariant,
Query, Scan, and Phase 2 crash/recovery tests remain in the workspace.

## Benchmark coverage

The checked-in benchmark includes one-, four-, and sixteen-key unconditional
and conditional transactions, plus asynchronous disjoint, hot-key, mixed, and
insert-if-absent workloads at 1, 4, 16, and 64 clients. It reports throughput,
conflict rate, WAL sync amortization, average WAL append/sync timing, and WAL
size. Any measured WAL sync latency is environment-specific; the Phase 2
recorded value of approximately 0.4 microseconds was a local observation and
is not representative of durable production block storage.
