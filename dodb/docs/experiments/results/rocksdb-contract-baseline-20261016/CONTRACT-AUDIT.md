# RocksDB Contract Audit

## Scope and result

This audit records the API and consumer contract used to compare the RocksDB
wrapper with the native B-tree and B-link services. It is documentation work
only. No tests or benchmarks were rerun for this audit, and it does not claim
that the six-phase plan or a performance baseline is complete.

The review found no current service consumer that requires RocksDB's logical
write identities to reproduce a global physical WAL order. The changes in the
architecture and backend contract clarify the existing distinction between
per-key revision identity, the `commit_lsn` response token, and native physical
LSNs. They do not change the API, wire encoding, condition behavior, transaction
atomicity, or committed read visibility. The service workload's write IDs are
used for uniqueness checks and final-state construction, not as evidence of a
read-inclusive global serialization order.

## API and implementation map

The public `Request` and `Response` variants and `TransactionOutcome.commit_lsn`
are defined in [`dodb-service/src/lib.rs`](../../../../crates/dodb-service/src/lib.rs#L47-L117).
The RocksDB wrapper dispatches those same variants in
[`dodb-rocksdb/src/lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L1059-L1262).

| Operation | Observable contract and source evidence |
| --- | --- |
| Get | A present value and its revision or a missing state and its revision; an uncreated tenant returns missing revision zero. RocksDB dispatch: [`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L1059-L1098). |
| Put | One successful key mutation, returning its assigned revision. It uses the transaction path: [`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L1100-L1128). A later successful write to that key advances the revision; common service test evidence: [`transaction_contract.rs`](../../../../crates/dodb-rocksdb/tests/common/transaction_contract.rs#L111-L164). |
| Delete | A successful delete retains a logical missing-state revision, including deleting an already missing key: [`transaction_contract.rs`](../../../../crates/dodb-rocksdb/tests/common/transaction_contract.rs#L127-L164). The wrapper uses the transaction path: [`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L1130-L1157). |
| Query | One partition key, canonical sort-key ordering, exclusive sort-key cursor, limit, tombstone omission, a per-call snapshot, and budget reservation before value copy. Wrapper dispatch: [`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L1188-L1243); seek, cursor, limit, and budget handling: [`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L1476-L1569). |
| Scan | Canonical document-key order, exclusive document-key cursor, limit, tombstone omission, a per-call snapshot, and budget reservation before value copy. Wrapper dispatch: [`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L1244-L1262); cursor, limit, and budget handling: [`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L1591-L1633). |
| Transact | Conditions are evaluated under locks covering the canonical union of condition and mutation keys; all mutations commit atomically and share one nonzero revision/identity. Empty requests and duplicate condition/mutation keys are invalid; condition plus mutation on one key is supported. A condition-only success allocates nothing and returns no identity. Implementation: [`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L1792-L2084); common evidence for condition-only and shared multi-key revisions: [`transaction_contract.rs`](../../../../crates/dodb-rocksdb/tests/common/transaction_contract.rs#L263-L290) and [`transaction_contract.rs`](../../../../crates/dodb-rocksdb/tests/common/transaction_contract.rs#L352-L405). |

## Response-budget enforcement boundary

The current response limit preserves the API's rejection behavior, but it is
not a universal guard before result materialization. For `Get`, RocksDB returns
a native allocation and `decode_and_free_get_value` copies the encoded state
into a Rust `Vec`; only after `RevisionState` exists does the wrapper construct
`Response::Get` and call `enforce_response_budget`. Thus a single returned value
is copied before the frame-size check. This is per-key bounded by the stored
value, but it does not satisfy a plan requirement to reject every oversized
result before allocating its result value.

For `Query` and `Scan`, each key is decoded and the native iterator exposes the
current value before `ReadResponseBudget::reserve_document` checks the
cumulative encoded result budget. The wrapper performs that reservation before
copying that document's value into the returned vector. Earlier documents may
already be materialized, and the current native iterator value has already
been read; the guarantee is limited to checking before each per-document Rust
value copy, not before all query/scan work or allocation.

The response-budget validation in `read_contract.rs` proves exact-fit and
too-small responses and tenant-scoped behavior. It does not prove pre-allocation
rejection for every API path. A broader pre-materialization enforcement
requirement therefore remains incomplete; no implementation change is made by
this audit.

Implementation: [Get dispatch and post-response check](../../../../crates/dodb-rocksdb/src/lib.rs#L1100-L1139), [native Get decoder](../../../../crates/dodb-rocksdb/src/lib.rs#L2523-L2533), [present-value Vec copy](../../../../crates/dodb-rocksdb/src/lib.rs#L2386-L2400), [Query reserve and copy](../../../../crates/dodb-rocksdb/src/lib.rs#L1622-L1631), [Scan reserve and copy](../../../../crates/dodb-rocksdb/src/lib.rs#L1660-L1700), [budget tests](../../../../crates/dodb-rocksdb/tests/read_contract.rs#L342-L463).

For each key, `RevisionEquals` identifies the complete present or missing state.
Missing revision zero is the initial state. Deleted states retain revisions, and
issued revisions are not reused for another committed state after recreation
or reopen. The common reopen/ABA contract is covered in
[`transaction_contract.rs`](../../../../crates/dodb-rocksdb/tests/common/transaction_contract.rs#L952-L1027).
A Query or Scan call is snapshot-consistent, but separate calls and pages do
not share a snapshot; the atomic query-view case is in
[`transaction_contract.rs`](../../../../crates/dodb-rocksdb/tests/common/transaction_contract.rs#L860-L908).

## Identity consumers and order scope

`commit_lsn` remains the existing API and protocol field. The protocol encodes
its optional u64 without conversion or added fields in
[`dodb-protocol/src/lib.rs`](../../../../crates/dodb-protocol/src/lib.rs#L680-L696).
It identifies a successful logical write within the tenant/backend identity
domain. The source does not support claiming global uniqueness across tenants.
It is not a RocksDB WAL sequence, wall-clock timestamp, or snapshot handle.
Numeric order across independent keys and response completion order for
concurrent requests are not part of the shared service contract. Per-key
revision progression and read-after-success visibility remain requirements.

The in-repository `doc-db` consumer maps returned present and missing revisions
into its own revision type and sends them back through `RevisionEquals`:
[`doc-db/src/dodb.rs`](../../../../../doc-db/src/dodb.rs#L361-L389). Its transaction
tracker stores the observed revision or missing state and builds the matching
condition later: [`doc-db/src/trx.rs`](../../../../../doc-db/src/trx.rs#L389-L419),
[`trx.rs`](../../../../../doc-db/src/trx.rs#L490-L513). This consumer depends on
per-key equality identity, not a cross-key numeric ordering.

The soak consumer builds conditions from observed per-key revisions and applies
the returned `commit_lsn` to the successful transaction's modeled mutations:
[`dodb-soak/src/main.rs`](../../../../crates/dodb-soak/src/main.rs#L805-L850),
[`main.rs`](../../../../crates/dodb-soak/src/main.rs#L1172-L1190). The service
workload requires nonzero write identities, checks their uniqueness within
its tenant run, and separately uses workflow IDs to derive final expected
rows: [`service_workload.rs`](../../../../crates/dodb-rocksdb/src/bin/service_workload.rs#L1053-L1090),
[`service_workload.rs`](../../../../crates/dodb-rocksdb/src/bin/service_workload.rs#L1344-L1395).
Its final-state oracle is not a read-inclusive history proof and must not be
presented as one.

The formal correspondence test at
[`formal_correspondence.rs`](../../../../crates/dodb-testkit/tests/formal_correspondence.rs#L669-L700)
asserts monotonically increasing commit LSNs while replaying the native
coordinator's ordered transaction groups. That check remains valid for the
native physical LSN implementation. It is not a shared wrapper contract and
must not be used to require a RocksDB allocator to reproduce native physical
sequence numbers. No consumer requiring cross-tenant numeric uniqueness or
cross-key token ordering was found in the inspected service, protocol, soak,
workload, or `doc-db` paths.

## Existing evidence and reuse

The following are existing source/test artifacts, not checks rerun for this
audit:

- Common transaction contract runs B-tree, B-link, RocksDB serialized mode,
  and RocksDB reserved-range mode from one test entry point:
  [`transaction_contract.rs`](../../../../crates/dodb-rocksdb/tests/transaction_contract.rs#L15-L51).
- The shared transaction cases cover revision advancement, condition-only
  behavior, same-key condition/mutation, atomic multi-key mutations, common
  transaction revisions, and reopen/ABA as cited above.
- RocksDB allocator unit cases cover partial-range reopen and abandoned values,
  user-transaction rollback token burn, uncertain reservation commit, and
  overflow/corrupt counter behavior in
  [`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L3362-L3463) and
  [`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L3559-L3593).
- Shared read contract source is
  [`read_contract.rs`](../../../../crates/dodb-rocksdb/tests/read_contract.rs#L57-L90);
  it is reusable for B-tree, B-link, and RocksDB through the same test harness.
- Paired service workload artifacts and prior implementation verification are
  indexed by the backend contract and comparison reports; they require their
  recorded source SHA, binary provenance, host/filesystem, workload seed,
  worker/trace settings, validator, and completion/RSS evidence before reuse as
  performance evidence. Historical contract passes alone do not establish a
  performance baseline.

## Remaining gates

Existing common cases already exercise same-key conditional contention and a
writer racing a non-mutated condition key
([`transaction_contract.rs`](../../../../crates/dodb-rocksdb/tests/common/transaction_contract.rs#L510-L615));
allocator cases cover uncertain reservation commit and mode switching
([`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L3462-L3514),
[`lib.rs`](../../../../crates/dodb-rocksdb/src/lib.rs#L3516-L3557)). These are
reusable prerequisites, but this audit did not execute them. Remaining useful
gates include a read-inclusive schedule across overlapping independent-key
writes, verifying each returned key revision against its observed history;
condition failures and failed/uncertain allocations must still publish no
mutation or reissue a token; mode switching and reopen must not reuse reserved
values.

A meaningful concurrency gate must validate histories with invocation and
response intervals plus observed reads. Sorting all successful tokens and
replaying that order is insufficient: independent writes may commute for a
final-state oracle, while a lower token may become visible after a higher token
when those writes overlap. Preserve atomic multi-key visibility, per-call
Query/Scan snapshots, and committed read-after-success while evaluating such
histories.

Raw RocksDB `WriteBatch` alone does not provide the conditional locking,
logical revision retention, or result mapping above. Keep it outside any
claimed common-service comparison. The default allocator remains
`Serialized`; reserved ranges remain opt-in until the planned correctness and
paired performance evidence are complete.
