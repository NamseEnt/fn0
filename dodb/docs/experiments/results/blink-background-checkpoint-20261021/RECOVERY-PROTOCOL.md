# Segmented WAL Checkpoint Recovery Protocol

## Implementation status

Segmented append, rotation, recovery, synchronous checkpoint advancement, and closed-segment reclamation are implemented. The background worker, concurrent checkpoint execution, retention backpressure, workload CLI selection, and performance matrices remain future work. The sections explicitly labeled as background design below describe that future work rather than current behavior.

The default service layout remains `BlinkWalLayout::Legacy` and its checkpoint policy remains `Disabled`. `BlinkWalLayout::Segmented { max_segment_bytes }` explicitly opts a new database into the segmented format through service configuration. `BlinkStore::open_segmented_path` provides the storage entry point. Existing legacy databases are rejected by that entry point; default legacy opening rejects segmented artifacts. No automatic conversion occurs. The segment byte threshold triggers rotation between write groups and is not a memory limit or total retained-WAL cap.

## Durable metadata

`SegmentedWalManifest` is a versioned, checksummed authority record stored beside a segmented-mode database. It binds database UUID, tenant, shard, and epoch to a metadata generation, committed checkpoint LSN and tree metadata (root, free-list head, high-water page, format/page size), an ordered list of required WAL segment identities and LSN ranges, the active segment number, and the next segment number. Open validates identity, generation/checksum, unique ordered segment numbers, non-overlapping contiguous ranges, active-file identity, and the relation between checkpoint and retained ranges before replay. A legacy database or incompatible/mismatched manifest is rejected; no automatic conversion is attempted.

The manifest commit uses a unique temporary file, complete write, file sync, atomic replacement, and parent-directory sync. Its generation increases monotonically. The manifest is the durable checkpoint boundary; replayed superblock images are tree data and cannot lower that boundary.

## WAL rotation and acknowledged writes

For rotation from active segment N to N+1:

1. Commit and synchronously flush the current transaction group to N using the existing physical transaction, commit, page-image, and page-delta encoding. Successful writes are not acknowledged before the sync returns.
2. Seal N at its last complete record/commit LSN and sync it. N remains required.
3. Create N+1 with a header bound to the database identity, epoch, segment number, and first expected global LSN. Sync the file and its parent directory so the name is durable.
4. Atomically persist and directory-sync a new manifest generation listing N and N+1, their ranges, and N+1 as active.
5. Only then append later commits to N+1 and acknowledge them after their normal WAL sync.

The global LSN and transaction/batch identifiers continue across file boundaries. A newly created segment emits an independent page image before any delta for that page. A delta is valid only when its base image and required preceding deltas remain in the retained segment chain.

Before step 4, an unlisted N+1 is an incomplete, uncommitted rotation artifact. Recovery may remove it only after the authoritative manifest and listed files have been validated and the file is proven not to contain an acknowledged write. Once step 4 is durable, both listed files are required even if no later commit reached N+1. A listed-file absence, duplicate range, identity mismatch, or corruption in a closed segment is a recovery error, never an ignored orphan.

## Synchronous checkpoint advancement and reclamation

`BlinkStore::checkpoint` runs while the caller owns the mutable store. The existing service coordinator awaits the whole operation between transaction groups, so writes on that shard wait for it. Immutable read handles keep their published generations. This implementation does not run a checkpoint concurrently with writes.

1. Fix the latest committed global LSN C. A checkpoint with no progress beyond the current manifest boundary is a no-op.
2. Force rotation at C, even below the normal segment byte threshold. The rotation manifest durably lists the old closed files and a new empty active file starting at C+1. Batch identifiers also continue without reset.
3. Write dirty data pages and the dirty superblock, then synchronize the database. Write an alternate superblock containing the current tree metadata and C, and synchronize it. A failure anywhere here leaves all old required WAL files available.
4. Atomically write, file-sync, replace, and directory-sync a new manifest generation with checkpoint C, the current root/free-list/high-water metadata, and only segments above C. This manifest is the checkpoint commit point.
5. Unlink only the now-unlisted closed files wholly covered by C, then synchronize the segment directory. The active WAL is never truncated or reset. An interruption here can leave obsolete files, but cannot remove a file required by the committed manifest.
6. Update the store's checkpoint metadata without rewinding its pages, revisions, next LSN, batch identifiers, or published read generations. Any checkpoint error fails the store closed for subsequent writes; reopen resolves the durable manifest and retained WAL.

Reopen validates every required file before cleaning obsolete files. Canonically named files below the first required segment number are cleanup candidates only after validating their identity, format, and fully covered LSN history. Unknown or corrupt unlisted files are left untouched and are never used as recovery substitutes. A missing required file remains an error even if obsolete files exist.

## Background checkpoint design, not implemented

A future worker must receive an immutable target image set and independent database file handle, while later writes populate a separate live dirty set and new WAL segments. Its durable boundary publication must be coordinated with concurrent rotation so neither manifest update can replace the other's required segment set. Completion must update only checkpoint bookkeeping, and cannot restore an old live root or revision state. The synchronous implementation does not yet provide this worker or coordination.

## Recovery decision and replay

Recovery first reads and validates the authoritative manifest and every required WAL file. It repairs committed pages before loading or validating the complete database tree. It verifies every required segment's identity, number, declared range, header checksum, and cross-file LSN continuity before replay. It replays only complete committed transactions with commit LSN greater than the durable manifest checkpoint. Recovery derives checkpoint tree metadata from the manifest and post-checkpoint tree changes from the latest replayable superblock image. It installs a higher-generation alternate superblock with the manifest checkpoint boundary before loading the tree; an older checkpoint LSN embedded in a replayed WAL image cannot lower that boundary. Torn tail data is recoverable only in the final active segment. For segmented files, the scanner retains bytes only through the last complete commit (or INIT for an empty active segment). It truncates and syncs complete uncommitted page frames together with any partial final frame before reusing their LSN and batch identifiers. This makes recovery followed by a new acknowledged write and another recovery valid. Closed files containing such an incomplete transaction are errors. Torn or corrupt bytes in a closed segment are fatal. Complete uncommitted transactions are not exposed.

If page writes stop partway, all not-yet-deleted segments remain available and replay repairs pages. If data sync completed but the new checkpoint manifest did not, recovery uses the previous manifest and retained WAL; replay is idempotent. If the new manifest is durable but deletion did not complete, extra old files are harmless and are removed only after validating the manifest. If deletion completed but directory sync did not, recovery accepts only the durable manifest's required set and validates any surviving unlisted files as cleanup artifacts. If the new manifest names a missing file, recovery fails rather than silently choosing an older boundary.

## Failure boundary table

| Boundary | Durable authority after restart | Required recovery action |
| --- | --- | --- |
| Before active group sync | Previous manifest plus prior WAL records | Discard incomplete uncommitted tail in active WAL; acknowledge no new write |
| After group sync, before response | Manifest-listed active WAL | Replay complete commit; caller outcome may be unknown but transaction remains atomic |
| After response, before rotation | Current manifest-listed active WAL | Replay acknowledged commit |
| Closed WAL sync, before new file durable | Previous manifest | Keep old active/closed WAL; no subsequent new-segment write can be acknowledged |
| New file durable, before rotation manifest | Previous manifest plus old listed WAL | Treat unlisted new file as incomplete rotation; validate no acknowledged write depends on it before cleanup |
| Rotation manifest durable | Both old and new segments | Validate and replay in global LSN order |
| Partial checkpoint page write or before data sync | Previous checkpoint manifest plus all required WAL | Replay from previous boundary to repair partial pages |
| Data sync complete, before checkpoint manifest | Previous manifest plus all required WAL | Replay; do not infer a newer checkpoint from page contents |
| Checkpoint manifest durable, before WAL deletion | New checkpoint manifest plus possible extra old files | Use new boundary; remove only validated obsolete closed files |
| WAL unlink, before directory sync | New checkpoint manifest | Required files remain selected by manifest; reconcile leftover unlink state safely |
| Directory sync after reclamation | New checkpoint manifest and its required files | Open normally; do not require reclaimed segments |
| Worker/channel failure | Last successfully synced manifest | Preserve all files required by it; fail shard writes consistently and finish waiters/shutdown |

## Implemented telemetry and future resource limits

Retained WAL bytes are the sum of every manifest-required active and closed `.wal` file. Closed-file lengths are cached at rotation/open and removed from that sum only after checkpoint manifest publication. Extra obsolete or unknown files are not counted as required WAL. Successful WAL sync calls, committed batches, page-image/delta counters, encoding/writing durations, and physical-write counts accumulate across rotations and reclamation during an open store's lifetime. Recovery reconstructs history-derived counters from the retained files; runtime sync/timing counters restart at zero and are not durable lifetime counters. Recovery-memory and chain-page gauges describe live retained objects rather than already-dropped closed-file readers.

`BlinkCheckpointReport.wal_bytes_reclaimed` counts bytes physically unlinked by a successful checkpoint. It can exceed the net before/after WAL size reduction by the newly created active INIT header size. Database page and metadata writes are included in `bytes_written`; WAL, manifest, and directory writes are separate work. Existing service checkpoint events still record net retained-WAL reduction.

The proposed 512 MiB WAL/snapshot limits, snapshot ownership accounting, background task capacity waits, and worker phase telemetry are not implemented by this milestone. They remain requirements for the future background implementation. No bounded engine cache or equal-memory benchmark is claimed.
