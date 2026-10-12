# RocksDB service backend R1b-1

Status: Query/Scan and shared read-contract verification passed locally and on
OCI at implementation SHA `e59ae32117df13e23cf298d2f8ec6a8fc4c53a93`. This is a
partial R1 result; it does not complete backend conformance.

## Read implementation

Query and Scan use the existing `DodbService` request and response types. RocksDB
user keys remain `0x01 || DocumentKey::encode()` and internal metadata remains
under `0x00`. The existing escaped-component key encoding preserves the
`DocumentKey` byte ordering: component terminators sort before extensions,
including extensions containing `0x00`, and all other bytes retain unsigned
lexicographic order. RocksDB's configured default bytewise comparator therefore
orders user keys like the native service.

Query seeks to the encoded partition prefix, or directly to the requested full
partition/cursor key. It stops when the decoded partition changes and applies
the sort-key cursor exclusively. Scan seeks to the user-key prefix or the full
exclusive document cursor. Both paths skip tombstone records, stop before
metadata keys, validate encoded cursor lengths, and apply `limit` to returned
live documents. A zero limit returns an empty result. No path scans the full
database to filter or sort results.

The C++ shim obtains one RocksDB snapshot, puts that snapshot in
`ReadOptions::snapshot`, and creates the iterator from those options. The
iterator wrapper owns both the iterator and snapshot and releases them together
on every return path, including budget errors. The tenant handle mutex protects
snapshot/iterator creation and is then released; iteration does not hold the
mutex, so a transaction can commit while an existing snapshot iterator remains
active. The database `Arc` held by the blocking request keeps the handle alive
through iterator destruction. Snapshot state is per request and is not carried
between calls or pages.

## Read response budget

The native storage path initializes its response budget with 17 wire bytes for
the response envelope, then reserves `PK length + SK length + value length + 20`
wire bytes and `max(PK length + SK length + value length + 64, wire bytes)`
materialization bytes per document. The memory ceiling is the configured wire
maximum plus 64 KiB. RocksDB Query/Scan use the same calculation and return a
`ResponseTooLarge` error without a partial result. The iterator's value pointer
and encoded length are inspected first; the document budget is reserved before
copying the value bytes. `limit` remains a document count and is independent of
the byte budget.

An absent tenant has no iterator; its synthesized empty response is checked
against the protocol encoder as in the native service. The existing protocol
frame encoder remains the final wire-format validator.

The R1a write-budget finding was compared with native behavior. RocksDB commits
Put/Delete/Transact before checking their response budget. Native single-key
Put/Delete also commit before response encoding; the B-tree batch mutation path
does not apply the read response budget. A too-small response frame can therefore
produce a response error after the mutation has committed on either service.
R1b-1 leaves this write behavior unchanged; any write-budget contract change
needs a separate design.

## Verification

The shared read contract is in
`dodb/crates/dodb-rocksdb/tests/read_contract.rs`; each backend receives the
same seeded records, requests, and independent expected values. Coverage and
host provenance are recorded in the
[R1b-1 verification report](../experiments/results/rocksdb-service-r1b1-2774bd1c/REPORT.md).

The verifier covers empty and missing partitions, binary keys and values,
partition and sort ordering, existing and absent exclusive cursors, first/last
and partition-boundary cursors, deleted and reinserted keys, zero and bounded
limits, page continuation, response-budget boundaries, four-tenant isolation,
reopen, and coordinated read/atomic-transaction races. A RocksDB-focused
iterator test opens a snapshot, reads one record, commits a two-key update, and
then reads the remaining record from the original snapshot; the result remains
entirely pre-transaction. This test uses explicit operation ordering rather
than a timing delay.

## Remaining work

The R1 exit condition remains open. Shared transaction conformance coverage,
tenant lifecycle and capacity management, process-kill recovery, and the
per-tenant write serialization caused by the handle mutex and shared revision
counter remain pending. The serialization issue must be addressed before a
performance comparison. This slice establishes no throughput or latency claim,
power-loss behavior, process-kill recovery, or bounded tenant-memory guarantee.
