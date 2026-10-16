# Transaction semantic contract

The Phase 3 executor is optimistic and single-shot. Reads may be performed
outside the commit request, but every point read that influenced a write must
be represented by a condition in the request:

```text
TransactionRequest {
    conditions: Vec<TransactionCondition>,
    mutations: Vec<TransactionMutation>,
}
```

The supported conditions are `RevisionEquals`, `Exists`, and `NotExists`.
`RevisionEquals(key, revision)` compares the complete logical state identity,
including the revision of a missing key. `NotExists(key)` tests only logical
absence, so a deleted key can satisfy it even when its missing revision is not
zero. This distinction detects missing -> insert -> delete ABA.

The supported mutations are `Put` and `Delete`. A request rejects duplicate
conditions and duplicate mutations for one key. A condition and mutation may
name the same key. The backend protects every condition and mutation key from
conflicting changes through validation and commit, including condition-only
keys that are not mutated. The protection mechanism and allocation order are
backend-specific. A successful write transaction is all-or-nothing; every
changed key receives the same nonzero logical revision. A condition-only
success allocates no revision and returns no commit identity.

`RevisionEquals` compares the complete logical state identity for a key,
whether present or missing. An initial missing key has revision zero. Every
successful write to a key advances its revision, and a previously issued
revision is never reused for a different committed state of that key, including
across delete/recreate, restart, checkpoint, or compaction. Allocators may
leave gaps.

The in-process reference executor records the equivalent form:

```text
read_set:  DocumentKey -> observed Revision
write_set: DocumentKey -> Put(bytes) | Delete
```

Writes also establish an observation for their key in the reference helper, so
write/write conflicts are validated even when a caller did not explicitly read
the key. At commit, every condition is evaluated against the latest staged
state before any mutation is applied. A mismatch returns a structured conflict
containing the key, expected condition, and actual logical state.

For the native B-tree/B-link WAL implementation, the shard commit coordinator
is the physical serialization point. It may stage several independent logical
transactions in one coordinator group. Each accepted transaction receives
its own increasing physical commit LSN and WAL `COMMIT` record. The WAL records
can share one sync; publication waits for that sync and follows coordinator
order. A later candidate validates against earlier accepted candidates in the
same group. These physical ordering details describe that implementation;
another backend can use a different durable allocator while preserving the
logical contract above.

The RocksDB TransactionDB implementation acquires locks for the canonical
union of condition and mutation keys before revision allocation and holds them
through the transaction commit. This is its mechanism for meeting the shared
protection guarantee; it is not a required locking strategy for other
backends.

The shared service exposes `TransactionOutcome.commit_lsn` as a nonzero logical
write identity for the tenant/backend identity domain. The field name and wire
encoding are retained for compatibility, but callers must not interpret this
value as a RocksDB physical WAL sequence, timestamp, or snapshot handle. The
contract does not define a numeric order across independent keys or tenants,
nor does it require response completion order to match token order when
requests overlap. Writes to the same key remain ordered by its revisions. A
read issued after a successful write response must observe that write or a
later committed write.

Ordinary `get`, Query, and Scan calls are independent operations. Separate
calls, including pages of one logical scan, do not promise a common snapshot.
Each Query or Scan call has one consistent committed snapshot and does not
traverse a mixture of transaction publications. The native storage API's
`transact_get` remains coordinator-serialized and returns requested point
states in input order from one committed coordinator state; it is not an
additional shared service request. A client can use ordinary reads plus
`RevisionEquals` conditions to form an optimistic write request. A read issued
after a successful write response observes that write or a later committed
write.

The `ReferenceDb` is the deterministic oracle for these semantics. It uses a
`BTreeMap<DocumentKey, RevisionState>`, keeps deleted keys as
`Missing(revision)`, gives each successful request one synthetic logical
identity, and applies all writes with that one revision. It supports explicit
conditions, multi-key mutation, point get/put/delete, same-pk ordered query
with an exclusive sort-key cursor, ordered scan with an exclusive document-key
cursor, and limit. `ReferenceDb::transact_at` can replay native physical commit
LSNs when a differential test has that implementation's serialized commit
order. It does not establish a physical-order requirement for other backends.

This is semantic reference code, not a production concurrency-control engine:
there is no locking, MVCC, WAL, recovery, or range-read validation.
