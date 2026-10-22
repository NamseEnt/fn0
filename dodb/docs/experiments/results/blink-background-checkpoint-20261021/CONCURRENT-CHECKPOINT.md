# Concurrent physical checkpoints

The service now supports `BlinkCheckpointExecution::Background` with explicit segmented WAL. `Sync` remains the execution default, `Legacy` remains the layout default, and automatic checkpointing remains disabled by default. Existing databases are not converted.

## Runtime path

`BlinkStore::prepare_segmented_checkpoint` fixes the latest committed LSN, forces a durable WAL rotation, and moves dirty encoded page images into a `BlinkCheckpointSnapshot`. It does not clone the live tree or page catalog. The snapshot owns immutable `Arc` page images, fixed superblock metadata, and a positional database file handle. Subsequent transactions keep the live store, use later WAL segments, and populate a new dirty set.

The independent worker writes snapshot pages, syncs data, writes the fixed checkpoint superblock, and syncs metadata. It owns neither `BlinkStore` nor the WAL manifest. `BlinkStore::complete_segmented_checkpoint` runs under coordinator ownership and publishes the target checkpoint into the latest manifest, preserving every intervening rotation and required post-target segment. Only after durable manifest publication are covered closed files unlinked and their directory synchronized. Completion updates checkpoint bookkeeping without installing the snapshot's root, free list, page catalog, revision allocator, LSN, batch identifier, or published read generation.

Segmented synchronous execution uses the same preparation, worker, and completion operations and waits immediately. Background execution polls the worker both between groups and while idle. The legacy service path continues using its existing checkpoint operation.

## Bounds and failure

A process-wide semaphore permits one active segmented physical checkpoint. Each shard retains one task and one coalesced checkpoint request. Shards waiting for the permit can continue writing until their retention budget requires waiting. Service configuration permits a positive smaller test/application budget up to the fixed 512 MiB ceiling; the default is 512 MiB. WAL and encoded dirty/snapshot image ownership are bounded separately by that budget. This is not a bound on the complete engine cache or process RSS.

Before WAL append, a conservative full-image bound checks the prospective group against retained WAL bytes and dirty/snapshot image bytes. It includes frame/commit and rotation reserves. Capacity failure happens before commit/state installation. The coordinator waits for checkpoint progress and retries the unchanged group at most twice; a group that cannot fit receives overload without a commit. These waits are inside request latency. A conservative bound can reject a group even when delta encoding would have fit.

Snapshot preparation, worker failure or panic, and manifest/reclamation failure terminate the coordinator consistently. Buffered clients receive durability errors; acknowledged transactions remain recoverable from retained WAL. The worker is joined before coordinator teardown. `shutdown_checked` reports terminal failure after joining all shard tasks; the pre-existing void shutdown interface still drains them and exposes failure through checkpoint metrics.

## Observability and use

Both workload binaries accept `--blink-wal-layout legacy|segmented`, `--blink-checkpoint-execution sync|background`, and positive `--blink-wal-segment-bytes`. Background execution requires segmented layout. `--blink-checkpoint-wal-bytes` selects the existing automatic threshold. Diagnostic-worker execution rejects segmented mode rather than reporting an unapplied configuration.

Reports contain effective layout/execution, compile-time Git identity, executable hash, measurement dispatch/last-client-completion timestamps, acknowledged writes completed while a worker was still active, coalesced requests, capacity waits, coordinator pause, retained-image gauges, and running-task gauges. Bounded per-checkpoint events contain task identity, fixed target, overlap, preparation/page-write/data-sync/metadata-sync/completion timing, and physically reclaimed bytes. Manifest/directory syncs are not classified as WAL-file syncs.

The implementation and measurement plan is `EXECUTION-PLAN.md`. Local evidence is archived separately from OCI evidence. Process SIGKILL scenarios demonstrate process-termination recovery, not physical power-loss safety. The finite checkpoint model abstracts transaction identities and retained WAL coverage; it does not prove the Rust implementation or page codecs.
