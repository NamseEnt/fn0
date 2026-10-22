# Concurrent checkpoint implementation update

Background checkpoint execution is now implemented in the task-owned working tree. See [CONCURRENT-CHECKPOINT.md](CONCURRENT-CHECKPOINT.md) for the actual runtime and recovery path and [EXECUTION-PLAN.md](EXECUTION-PLAN.md) for the staged validation and measurement plan. The worker writes fixed images; the coordinator publishes into the latest manifest. Local overlap, failure, bounded-retention, recovery, and finite-model evidence is being archived. OCI validation and performance results are reported separately when complete.

## Preserved earlier milestone

The following text records the earlier synchronous-only milestone and its original plan. Statements that background execution is unimplemented describe that earlier source, not the current implementation.

# Background B-link Checkpoint Plan

## Current implementation boundary

The larger background plan below remains incomplete. The current implemented milestone covers explicit segmented-mode selection, append/rotation/recovery, the three review corrections, and synchronous checkpoint boundary advancement with WAL reclamation. It does not implement the worker, concurrent write/checkpoint execution, bounded background retention, CLI selection, or performance matrices. See `RESULTS.md` and the milestone section in `VALIDATION.md` for current evidence. Future background execution must coordinate manifest publication with concurrent rotation; calling the synchronous checkpoint from a worker that owns the whole store does not establish write/checkpoint overlap.

## Scope and baseline

The implementation starts at `39cdfe48a20fb6f381dbb461d721fd51fa8f9f0c` on branch `experiment/blink-background-checkpoint-20261021`. The current physical path is `BlinkStore::open_internal`, `BlinkStore::apply_serial_transaction_group`, and `BlinkStore::checkpoint` in `dodb/crates/dodb-storage/src/blink/mod.rs`, backed by one `WalLog` in `dodb/crates/dodb-storage/src/wal.rs`. `AsyncBlinkShard::start` in `dodb/crates/dodb-server/src/lib.rs` currently runs a whole checkpoint as coordinator work. The logical-WAL materializer is a separate path and is outside this change.

The existing `BlinkState`, `dirty_pages`, `dirty_superblock`, `current_superblock`, `next_revision`, and `next_lsn` remain the live tree state. Legacy mode continues using the existing single-WAL checkpoint path. Segmented mode is a new-database-only format and rejects legacy or mismatched metadata without converting it.

## Intended runtime and durable state

| Concern | Runtime representation and code location |
| --- | --- |
| Live tree and post-target dirty set | `BlinkStore.state`, `BlinkStore.dirty_pages`, `BlinkStore.dirty_superblock`, `current_superblock`, `next_revision`, and `next_lsn` in `dodb/crates/dodb-storage/src/blink/mod.rs` |
| Fixed checkpoint target | `BlinkCheckpointSnapshot`: task id, target LSN, immutable `Arc<[u8; PAGE_SIZE]>` page images moved from the dirty map, encoded superblock/root/free-list/high-water metadata, and format/identity fields |
| Durable checkpoint authority | Versioned, checksummed `SegmentedWalManifest` in `dodb/crates/dodb-storage/src/blink/segmented_wal.rs`; its generation, DB/tenant/shard/epoch identity, checkpoint LSN and tree metadata, ordered required segment identities/ranges, active segment number, and next segment number are authoritative |
| Active and closed WALs | `SegmentedWal` owns the active `WalLog` plus manifest-listed closed segments; a rotation seals the old active file and starts a new active file without resetting global LSN or transaction identifiers |
| Checkpoint result | `BlinkCheckpointResult` carries task id, target LSN, written/synced bytes, phase durations, reclaimed segment identities, and failure boundary; coordinator completion applies it only if task id and generation are current |
| Coordinator scheduling | `AsyncBlinkShard` tracks one running task and one merged pending request, polls completions while idle, and applies the configured `sync` or `background` wait policy |
| CLI and effective configuration | `BlinkWalLayout` and `BlinkCheckpointExecution` flow from workload argument parsing through `LocalTenantServiceConfig`; result JSON reports selected values |

The fixed image set is captured at a successful write-group boundary after responses for that group have been committed. Ownership of changed images moves out of the live dirty map; later writes install new images in that map. The worker receives only the snapshot and a cloned database file handle. It never receives `BlinkStore`, `BlinkState`, a mutable catalog, or live revision/LSN state. Snapshot setup records elapsed time, moved/shared image count, and retained encoded bytes; it does not clone the full state or page catalog.

## Checkpoint and WAL transitions

1. At a write-group boundary, freeze the latest committed LSN and bundle dirty page images with the root, free-list, high-water, identity, format, and task metadata.
2. Rotate the active WAL using the durable ordering in `RECOVERY-PROTOCOL.md`; only after the manifest naming both the closed and new active files is durable can following writes be acknowledged.
3. For `sync`, run the same snapshot worker and wait for its result. For `background`, submit the same worker and keep serving writes from the live tree while it writes only the snapshot.
4. Write every fixed page image, synchronize the database file, and durably advance the manifest checkpoint LSN/tree metadata.
5. Remove only closed WAL files fully covered by that durable checkpoint, then synchronize the containing directory.
6. Send a completion result. Coordinator completion advances only the durable checkpoint boundary and reclaimable-WAL range; it never installs old root/page/revision/LSN state into the live store.

A new page's first record in a segment is a standalone image. Later deltas in that segment depend only on its image or prior records still in the same retained segment chain. Segment rotation therefore starts a new image base for every page whose first record would otherwise depend on a segment eligible for deletion.

## Concurrency, limits, and failure behavior

At most one checkpoint runs per shard and at most one runs process-wide. Each shard has at most one merged pending request. There is no unbounded worker queue. The checkpoint-retained encoded page images and versions held alive only by them are accounted against 512 MiB per shard. All manifest-listed active and closed WAL bytes share a 512 MiB per-shard limit. These are checkpoint/WAL retention limits, not an engine-wide memory cache limit. The existing process RSS stop line remains 3 GiB.

Before committing a mutation group, the coordinator waits when projected WAL or checkpoint-retained memory would exceed the fixed limits. It resumes after checkpoint completion releases ownership. A single request/snapshot that cannot fit is rejected with an explicit overload/resource error. Wait duration is included in request latency and recorded separately. If checkpoint work fails, acknowledged writes remain covered by retained WAL, the shard enters a durability-failed state, later writes fail consistently, and waiters and shutdown receive a terminal result.

Shutdown stops admission and scheduling, drains accepted requests and the active task, collects its result, and records remaining work and drain time. A worker/channel failure is terminal; no silent retry or limit relaxation occurs.

## Measurements and gates

Record bounded event samples for request/start/finish/failure/merge, task id and fixed LSN, and durations for snapshot preparation, WAL rotation, page writes, data sync, metadata commit, and reclamation. Record coordinator pause, capacity wait, writes completed during a checkpoint, mutation transaction/group counts, actual successful sync calls by purpose, current/peak active/closed/retained WAL bytes, current/peak retained image bytes, page bytes written and reclaimed, and final drain/remaining task state. Sample timestamps are partitioned into seed, warmup, measurement, drain, final validation, and reopen. Percentiles use all completed samples; histogram count must equal sample count. Storage is capped and omissions are explicit.

Before performance work, run the format model, legacy contract suite, sync/background transaction/read/history contracts, barrier-controlled concurrent-write tests, state-preservation cases, segment and checkpoint fault boundaries, and SIGKILL recovery against an independent parent-owned expected state. SIGKILL is process-termination evidence only, not power-loss evidence. Any failed required validation stops the performance stage.

The short OCI matrix is the exact five-path, A/E, three-repetition schedule in the task specification, on the approved host/filesystem/toolchain/source only. Preserve each attempt and apply the predeclared numeric gates without changing defaults. The 300-second matrix runs only if the 128 MiB background comparison meets every required condition and resource estimates remain under the fixed stop limits.
