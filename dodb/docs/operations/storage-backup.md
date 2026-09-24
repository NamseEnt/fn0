# Storage-level backup and restore

dodb supplies the WAL durability and crash-recovery protocol. It does not
create, copy, retain, or transfer application-level backup snapshots.

## Required storage contract

Production storage must provide an atomic, crash-consistent point-in-time
snapshot covering the database file and its WAL in the same recovery domain:

```text
one storage snapshot
├── tenant-X-shard-X.db
├── tenant-X-shard-X.wal
├── tenant-Y-shard-Y.db
└── tenant-Y-shard-Y.wal
```

Copying the database and WAL independently at unrelated times is not a valid
backup. Deployments spanning multiple physical or cloud volumes require a
storage feature that creates an atomic or group-consistent snapshot across all
participating volumes. Independently created volume snapshots do not provide
that guarantee.

An optional dodb checkpoint before the storage snapshot may reduce WAL size
and recovery time, but it is not required for correctness. A crash-consistent
database-plus-WAL snapshot can be restored and opened normally even when no
checkpoint immediately preceded it.

## Recommended self-hosted reference: ZFS

ZFS is the recommended reference filesystem for self-hosted deployments. A
dataset snapshot is a near-instant copy-on-write point-in-time operation, for
example:

```text
zfs snapshot pool/dodb@backup-N
```

dodb neither calls ZFS nor depends on libzfs. The database need not stop while
the snapshot is created, and there is no application-level backup barrier.
Snapshot creation and off-host transfer are separate operations; the latter
still takes time and can use a full or incremental transfer such as:

```text
ZFS snapshot
    ↓
zfs send / incremental send
    ↓
another disk or server
```

Do not disable the storage durability semantics required by dodb. A successful
durable write or fsync issued by dodb must retain the durability guarantees
expected by the storage engine; configurations that deliberately ignore
synchronous write durability are unsupported.

## Other storage systems

Suitable cloud block-volume systems and other filesystems may be used when
they provide the same atomic crash-consistent point-in-time block snapshot
contract. The dodb implementation has no provider-specific dependency.

## Restore

1. Stop dodb for the target recovery instance.
2. Restore or clone the storage-level snapshot.
3. Confirm that the database file and WAL came from the same consistency point.
4. Start dodb normally.
5. Allow normal WAL recovery to complete.
6. Verify storage invariants and service health.

This is deliberately the same recovery path as a crash-consistent disk image;
there is no separate snapshot-restore API.
