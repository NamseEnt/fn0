# Segmented WAL and Background Checkpoint Recovery Protocol

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

## Checkpoint target and completion

At the acknowledged write-group boundary, snapshot task T fixes `checkpoint_lsn = N`, root/free-list/high-water metadata, identity/format, and immutable encoded dirty-page images. Image ownership moves to T while later mutations populate a distinct live dirty set. The worker has no access to mutable tree state.

The worker performs these durable steps:

1. Write all target page images to the database file.
2. Synchronize the database file.
3. Persist and directory-sync a manifest generation whose checkpoint LSN and tree metadata equal T's fixed target. The target must not be older than the durable checkpoint.
4. Delete only closed WAL segments whose complete commit/range is at or before T's checkpoint LSN and which are not named as required by the new manifest. Synchronize the containing directory after deletion.
5. Deliver T's completion result.

The active WAL is never truncated or reset. New commits during page writes are in the active segment and remain above T's boundary. Coordinator completion validates task identity/generation and advances only durable checkpoint and reclaimable-range bookkeeping. Stale or duplicate results are rejected. It cannot rewind the live root, pages, revisions, or next LSN.

## Recovery decision and replay

Recovery first reads and validates the authoritative manifest, then loads the checkpointed database tree. It verifies every required segment's identity, number, declared range, header checksum, and cross-file LSN continuity before replay. It replays only complete committed transactions with commit LSN greater than the durable manifest checkpoint. Page-LSN checks remain idempotence guards; a WAL superblock image that contains an older checkpoint LSN cannot change the manifest boundary. Torn tail data is recoverable only in the final active segment, where the existing physical WAL framing rules identify an incomplete final record/group. Torn or corrupt bytes in a closed segment are fatal. Complete uncommitted transactions are not exposed.

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

## Resource and telemetry accounting

Retained WAL bytes are the sum of every manifest-listed active and closed `.wal` file, not the largest file and not cumulative disk writes. Checkpoint-retained memory counts unique encoded page-image allocations whose ownership/reference lifetime is extended by a snapshot, plus snapshot-only metadata and additional prior versions retained solely by those references. It excludes the ordinary live tree and general page catalog. Counters rise when snapshot ownership is acquired and fall when it is dropped, so reported lifetime follows actual ownership. Both WAL and retained-image totals have a fixed 512 MiB per-shard ceiling. Snapshot preparation time and copied/moved/shared bytes, all checkpoint phase times, successful actual sync calls, bytes written/reclaimed, capacity wait, coordinator pause, and drain are separately recorded.
