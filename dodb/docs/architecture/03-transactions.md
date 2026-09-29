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
conditions or duplicate mutations for one key. It is all-or-nothing and has
one commit LSN; every changed key receives that LSN as its revision.

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

The shard commit coordinator is the physical serialization point. It may stage
several independent logical transactions in one coordinator group. Each
accepted transaction receives its own monotonically increasing physical commit
LSN and WAL `COMMIT` record. The WAL records can share one sync; publication
waits for that sync and follows coordinator order. A later candidate validates
against earlier accepted candidates in the same group.

Ordinary `get`, Query, and Scan calls are independent operations over the
currently published committed read view and do not promise one common read
instant with other operations. Each individual operation holds one read guard,
so it does not traverse a mixture of page publications. `transact_get` remains
coordinator-serialized and returns requested point states in input order from
one committed coordinator state. A client can use ordinary reads plus
`RevisionEquals` conditions to form an optimistic write request. A read issued
after a successful write response uses the newly published view.

The `ReferenceDb` is the deterministic oracle for these semantics. It uses a
`BTreeMap<DocumentKey, RevisionState>`, keeps deleted keys as
`Missing(revision)`, gives each successful request one synthetic monotonically
increasing logical LSN, and applies all writes with that one revision. It
supports explicit conditions, multi-key mutation, point get/put/delete,
same-pk ordered query with an exclusive sort-key cursor, ordered scan with an
exclusive document-key cursor, and limit. `ReferenceDb::transact_at` can
replay a real engine's physical commit LSN, so differential tests can compare
revisions directly when they have the serialized commit order.

This is semantic reference code, not a production concurrency-control engine:
there is no locking, MVCC, WAL, recovery, or range-read validation.
