# dodb architecture overview

Phase 0 established contracts for a transactional document/KV database whose
logical key is `(tenant, (pk, sk))` and whose value is opaque bytes. Phase 1
adds a checked single-file B+Tree storage engine for one shard. Phase 2 adds
the redo-only WAL and crash recovery. Phase 3 adds optimistic same-shard
point-key transactions and WAL group commit.
Phase 5 adds formal checkpoints, WAL reclamation, and the storage-level
crash-consistent backup contract. Phase 6 adds the bounded raw QUIC application protocol,
the transport-independent service boundary, the local tenant/shard service,
and the first-party Rust client.

The planned v1 deployment is one compute process with one file per shard. A
tenant may later span multiple shards, so `TenantId` and `ShardId` remain
separate types and routing stays outside the physical document key.

## Workspace

- `dodb-core`: strong identifiers, revision state, key codec, semantic error,
  transaction request, condition, mutation, result, and transaction-set
  types.
- `dodb-storage`: fixed-page/header and double-superblock codecs, the
  `DurableFile` contract and production adapter, and the Phase 1 B+Tree,
  allocator, overflow storage, cache, invariant checker, and async shard
  facade with the Phase 4 committed-read lane and single-writer coordinator.
- `dodb-testkit`: deterministic volatile/durable file simulation, named crash
  points, and the simple reference database/transaction oracle.
- `dodb-service`: storage-independent request/response values and the
  asynchronous `DodbService` boundary used by transports and test doubles.
- `dodb-protocol`: the versioned big-endian binary frame codec and stable
  application error mapping. It depends on `dodb-core` and `dodb-service`,
  but not on storage or QUIC.
- `dodb-client`: the async Rust client. It depends on `dodb-core`,
  `dodb-service`, `dodb-protocol`, Quinn, and Tokio, and owns one reusable
  QUIC connection per client.
- `dodb-server`: the Quinn transport, TLS configuration, bounded stream
  execution, metrics, and local `TenantId -> ShardId -> AsyncShard` service.
  It depends on `dodb-core`, `dodb-service`, `dodb-protocol`, and
  `dodb-storage`.

Phase 5 remains local to one physical shard. Phase 6 keeps the same local
durability boundary: network requests never expose pages, WAL records, LSN
frames, superblocks, or checkpoints. Application-level backup files are not
part of the protocol. There is no replication,
remote object storage, public PITR, distributed checkpoint, authentication,
tenant authorization, or distributed routing.

The Phase 1 engine is a mutable fixed-4KiB slotted-page B+Tree with inline
small values and overflow pages. Phase 2 adds a separate redo-only full-page
after-image WAL, NO-STEAL, NO-FORCE publication, committed dirty-page flushing,
and startup recovery. Phase 4 adds an atomically published committed read view
for ordinary reads while retaining one physical mutation writer per shard. The
single-file constructor remains for storage-format tests; production path
opening uses the paired data/WAL files.
