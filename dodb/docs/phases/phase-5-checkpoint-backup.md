# Phase 5: Checkpoint, WAL reclamation, and storage-level backup

## Durability-state investigation

Phase 4 publishes only WAL-durable page images. `BTreeStore::dirty_pages` is a
`BTreeMap<PageId, [u8; PAGE_SIZE]>`, so the latest committed image for each
page is retained once, including allocator, free-list, root, overflow, and
tree pages. `dirty_superblock` retains the latest committed alternate
superblock image. The existing `flush` writes those images, extends the file
to the current high-water page, and syncs the data file. It does not advance
`checkpoint_lsn` and it does not change the WAL.

The selected double-superblock copy is the valid copy with the greatest
generation. Ordinary commits preserve `checkpoint_lsn` and advance generation
in the alternate slot. The data file is therefore independently reopenable at
the last formal checkpoint, while retained WAL supplies later commits.

The WAL has one identity `INIT` record followed by strictly increasing record
LSNs. Page-image records consume LSNs and each logical `COMMIT` record supplies
the commit LSN used by revisions and page headers. Phase 5 WAL format version 2
extends `INIT` with `start_after_lsn`; version 1 WALs remain readable and use
zero as their history start. An empty WAL is initialized with the database
checkpoint as its history start.

## Checkpoint model

`BTreeStore::checkpoint` is a synchronous local primitive. `AsyncShard` exposes
the same operation through its bounded FIFO coordinator. A checkpoint
operation is ordered after already admitted mutations; mutations admitted
after it stay behind it. Collected mutations before and after a checkpoint
barrier are committed as separate physical groups, and responses for a
completed group are released before a later barrier runs. Ordinary immutable
reads continue through the committed read view.

The checkpoint chooses the last durable WAL commit LSN, or the current
checkpoint LSN when there is no later commit. It rejects a decreasing boundary,
writes all committed dirty page images and the current committed superblock,
and syncs the database file before writing checkpoint metadata. It then writes
the alternate superblock with a higher generation and the selected checkpoint
LSN, syncs that metadata, runs the invariant checker, and only then resets the
WAL.

The checkpoint report contains the LSN, page count, bytes written, reclaimed
WAL bytes, and elapsed time. Repeated checkpoints with no writes do not move
the boundary or rewrite the WAL unnecessarily.

## WAL reclamation and recovery

The local policy checkpoints the latest durable commit before reclamation. WAL
reset is performed while the write coordinator is paused:

1. truncate the WAL and sync the truncation;
2. write a new identity `INIT` with `start_after_lsn = checkpoint_lsn`;
3. sync the new initialization;
4. resume writes with the next record LSN equal to `checkpoint_lsn + 1`.

LSNs are never restarted at zero. Batch IDs are segment-local after reset; the
LSN sequence is the durable ordering authority.

Recovery reads the newest valid superblock, uses its checkpoint as the data-file
authority through N, and replays only complete committed WAL units with commit
LSN greater than N. Older WAL history is ignored. Page-LSN comparison remains
the idempotence guard for data pages. A crash before checkpoint metadata sync
leaves the previous checkpoint plus WAL authoritative; a crash after metadata
sync leaves the checkpointed database authoritative through N even if WAL reset
is incomplete.

## Backup policy

dodb does not provide an application-level native backup or snapshot format.
It provides WAL durability, crash recovery, checkpointing, and WAL reclamation.
Production deployments must place dodb storage on infrastructure that can
create an atomic crash-consistent point-in-time storage snapshot.

The database file and its WAL must be captured in one storage consistency
domain. For example:

```text
storage snapshot domain
├── tenant-X-shard-X.db
├── tenant-X-shard-X.wal
├── tenant-Y-shard-Y.db
└── tenant-Y-shard-Y.wal
```

Independently copying the database and WAL at unrelated times is not a valid
backup. If a deployment uses multiple physical or cloud volumes, the operator
must use a storage feature that provides an atomic or group-consistent
point-in-time snapshot across every volume participating in one recovery
domain. Arbitrary collections of independently created volume snapshots are
not equivalent.

An explicit checkpoint before a storage snapshot is allowed as an operational
optimization: it can reduce WAL size, recovery work, and restore startup time.
It is not required for correctness. A crash-consistent snapshot of the database
file and WAL is sufficient for normal dodb opening and WAL recovery even when
no checkpoint ran immediately before the snapshot.

The backup contract deliberately reuses ordinary crash recovery. It depends on
WAL framing, commit durability, redo recovery, checkpoint correctness,
database/WAL identity validation, and torn-write handling. There is no second
backup consistency protocol and no dodb-specific restore API.

## Recommended reference deployment: ZFS

ZFS is the recommended self-hosted reference deployment because a ZFS dataset
can provide near-instant point-in-time copy-on-write snapshot creation without
a database-level copy pause. Conceptually, an operator may use:

```text
zfs snapshot pool/dodb@backup-N
```

dodb does not call ZFS, use libzfs, or depend on ZFS APIs. Snapshot creation
and off-host transfer are separate operator actions. A snapshot can be created
quickly while dodb continues serving traffic; transferring it elsewhere still
takes time and may use a full or incremental send, for example:

```text
ZFS snapshot
    ↓
zfs send / incremental send
    ↓
another disk or server
```

Operators must preserve the storage durability semantics required by dodb. In
particular, configurations that deliberately ignore synchronous write
durability are unsupported. A successful durable write or fsync issued by dodb
must retain the durability guarantees expected by the storage engine.

Other systems may be valid when they provide the same atomic,
crash-consistent point-in-time block snapshot contract, including suitable
cloud block-volume snapshot services. The dodb architecture remains
provider-neutral.

## Restore procedure

Restoring a backup is a storage and process operation, not a separate dodb
code path:

1. stop dodb for the target recovery instance;
2. restore or clone the storage-level snapshot;
3. ensure the database file and WAL came from the same snapshot consistency
   point;
4. start dodb normally;
5. let normal WAL recovery run;
6. verify database invariants and service health.

The restored files are intentionally treated like a crash-consistent disk
image. Normal database opening and recovery are the source of truth.

## Fault and test coverage

Named fault points cover checkpoint gate acquisition, data page writes and
sync, alternate superblock writes and sync, WAL truncation, WAL initialization,
WAL reset, and reset completion. Deterministic tests reopen after every
checkpoint reset boundary and run the invariant checker.

The portable test suite includes checkpoint fault matrices, torn-`INIT`
recovery, abrupt subprocess restart tests, WAL framing and identity checks,
transaction-group recovery, and recovery after a durable WAL commit whose data
file publication was interrupted. It does not require ZFS or a cloud provider
and does not include storage-provider integration in ordinary `cargo test`.

`phase5-bench` reports checkpoint LSN, dirty pages, bytes written, reclaimed
WAL bytes, and checkpoint timing for small and medium local databases. These
are local observations, not compatibility thresholds.

## Limitations

Phase 5 is local and single-shard. There is no backup retention policy, remote
backup transfer, object-store integration, replication, public PITR API,
distributed checkpoint, background checkpoint policy, online shard migration,
or network endpoint for backup administration.
