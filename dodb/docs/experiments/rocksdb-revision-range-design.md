# RocksDB Revision Range Reservation Design

## Scope

This experiment adds an opt-in tenant-local revision allocator to the RocksDB
service. The default remains the existing serialized counter transaction. The
range size is fixed at 4,096 revisions in the service; checks may use a smaller
internal range to exercise exhaustion and reopen behavior. The public service
requests and responses do not change.

## Allocation and persistence

After a user transaction acquires all condition and mutation key locks in the
existing canonical order and validates its conditions, range mode takes a
short tenant-local allocator mutex. An empty in-memory range is replenished by
an internal TransactionDB transaction that locks only the existing tenant
counter metadata key, reserves the next 4,096 values, and commits with WAL
enabled and sync requested. The mutex is released as soon as that reservation
commit returns. A token is taken from memory only after reservation success.
The user transaction writes the token into every mutation and commits using
the existing synced transaction path.

The persisted counter is the reservation high-water mark in range mode and the
last committed revision in serialized mode. Both meanings are safe when
switching modes: the next allocation starts strictly above the persisted value.
The existing revision counter has no readers outside revision allocation.
Unused range members are abandoned on process shutdown or reopen and are never
reissued. Gaps therefore remain valid opaque revision identities. Reservation
overflow or an uncertain reservation commit returns an error and does not
publish candidate tokens; no high-water estimate is used.

## Ordering and lock safety

User key TransactionDB locks, including condition-only keys, are unchanged and
remain held through the user commit. Allocation happens only after condition
validation. Transactions that share any protected key therefore allocate in
the same order in which their key lock dependency is serialized. Independent
transactions may commit in a different physical completion order; their
revision/commit identities define a valid logical serialization order because
their mutations commute. `commit_lsn` remains the unique logical write identity
used by the existing driver replay, not a RocksDB physical sequence number or
response completion timestamp. `Revision` and `Lsn` are distinct newtypes over
`u64`; the RocksDB service continues to return the same allocated integer in
both fields. The existing service contract documents per-key identity, not
contiguous values or response completion order. Condition-only requests still
allocate nothing.

Reservation transactions touch only counter metadata and never acquire user
keys, so a transaction holding user locks while requesting a range cannot form
a cycle with a reserver. The allocator mutex may cover the internal durable
reservation, but never the user transaction's commit. A failed or rolled-back
user transaction consumes its token permanently.

## Format and recovery boundary

The revision counter's eight-byte encoding is retained. No new metadata key or
record encoding is introduced, so existing databases can enter either mode.
When first opened in range mode, an existing counter is treated as a high-water
mark and the next reservation begins above it. Returning to serialized mode
also begins above the reserved high-water mark. This permits mode switching in
this experiment build without token reuse. It does not promise that an older
binary can infer how many unused revisions were reserved; the high-water value
still prevents reuse.

An uncertain internal reservation commit fails the request. The process does
not attempt to infer whether the reservation reached durable storage. After a
restart, the stored counter is authoritative and all values at or below it are
considered consumed. This change adds no process-kill or power-loss claim; the
existing WAL-sync contract and normal reopen checks are the only recovery
evidence.

## Validation and measurement

Run the common transaction contract and focused allocator checks for uniqueness,
re-reservation, rollback gaps, partial-range reopen, existing-format databases,
condition ABA, tenant isolation, overflow, and reservation failure. Compare the
default and experimental allocator with the existing service workload driver,
its independent state replay, and the same measured instrumentation settings.
Each allocator uses separate fresh database paths. The OCI matrix is one run
per allocator and each of RW95, RW80, RW50, RW20, and MT-MIX.
