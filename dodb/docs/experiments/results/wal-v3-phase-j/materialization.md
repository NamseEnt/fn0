# WAL v4 Materialization and Reclaim

J3 implements synchronous bounded materialization for `BlinkStore::open_with_logical_wal`. Four segments are the runtime limit. When a group arrives while all four are present, the writer materializes the current view before admitting that group. `checkpoint()` can also request maintenance explicitly. The writer is paused during this first materializer implementation; pinned readers continue on their immutable generation.

The full current overlay set is applied in commit order. Existing overlay revisions are copied into B-link entries unchanged. B-link routing, splits, overflow allocation, and page encoding run only in this maintenance path. Before writing, the materializer copies the currently reachable tree and overflow pages into page IDs that are not referenced by the current base. Remaining inactive page IDs become a bounded scratch/free pool. It reserves enough spare page capacity for a second base-sized generation. The page file can expand when the live base outgrows that reserve; repeated compactions at a stable live size reuse the inactive pages.

The durable order is:

1. Build the replacement base in inactive page IDs and apply all current overlays.
2. Write replacement and scratch/free pages, then sync the data file.
3. Write and sync superblock version 3 with the replacement root, high-water mark, physical WAL checkpoint LSN, and logical checkpoint sequence.
4. Publish the immutable base-only `PublishedGeneration`.
5. Rewrite the previous active page set as free pages and sync it. These IDs were included in the new superblock's free list. If this step is interrupted, logical open reconstructs the free list from pages outside the durable root.
6. Truncate and sync the logical WAL, then write and sync a new logical INIT frame containing both the physical LSN and logical sequence watermarks.

The old active pages are never overwritten before the new base and its superblock are durable. WAL reclaim happens only after that checkpoint is durable. A crash after superblock sync but before view publication reopens the new base and filters WAL transactions at or below its physical checkpoint LSN. A crash before the superblock sync opens the old base and replays the still-retained WAL. A crash during WAL reset is safe because the base watermark is already durable; an empty WAL is initialized at that LSN and sequence on reopen.

The superblock remains able to decode version 2 files. Version 3 stores `checkpoint_sequence` separately from `checkpoint_lsn`. Logical WAL INIT frames use a 60-byte identity/watermark payload; earlier 52-byte version 4 INIT frames remain readable and use their LSN boundary as the sequence boundary. WAL v4 reset writes the expanded INIT with independent values.

When an interrupted reclaim leaves a durable checkpoint and an older untruncated WAL, open scans and validates the complete WAL, then replays only commits with physical Commit LSN greater than the superblock watermark. It retains the highest sequence and LSN for subsequent commits. This preserves correctness without reclaiming data before checkpoint durability.

The bound has two parts. The immutable view contains at most four segments, and the logical WAL rejects an append group that would exceed 256 MiB before writing it. A fifth segment triggers synchronous checkpoint maintenance before admission. If maintenance fails before the checkpoint watermark is durable, the current base, overlays, and WAL remain authoritative and the fifth group is not appended. Failures after an uncertain superblock write or after publication leave the store degraded until reopen.

`BlinkCheckpointReport` records checkpoint LSN and sequence, pages and bytes written, reclaimed WAL bytes, segment count and packed bytes materialized, transaction lag, and total synchronous maintenance duration. On a 2 OCPU host the writer pause is an explicit cost; J4 measured its durable throughput effect below.

J3 tests cover split and overflow values, repeated updates across materializations, reader pins, WAL reclaim, later writes, second reopen, page-capacity reuse, and faults around page write, data sync, superblock sync, WAL truncation, and WAL INIT sync. `dodb-testkit` crash tests separately discard unsynced file bytes at those boundaries and recover through the durable base plus unreclaimed WAL. A StorageFull write fault and a data sync I/O fault also preserve acknowledged state.

The current policy materializes all segments at the four-segment bound rather than just the oldest segment. That keeps writer pause work simple and deterministic. J4 measured maintenance duration and lag; sustained growth behavior remains unmeasured because the performance prerequisite failed.

## J4 Measured Maintenance Cost

The primary 64-writer width-16 uniform run recorded 270 materializations during its 10-second measured interval. Average maintenance duration was 22.90 ms, maximum 31.22 ms, and cumulative maintenance duration was 6.18 seconds. The 64-writer width-1 uniform run materialized 500 times with 16.47 ms average and 23.98 ms maximum duration. The writer pause is synchronous, so this cost appears directly in write latency and throughput.

The measured J/H1 ratio was 0.720x for the primary width-16 case and 0.376x for the width-1 guardrail. Since both durable performance gates failed, the 120-second sustained run and RocksDB comparison were skipped. These measurements do not claim bounded sustained RSS or WAL growth under a longer run. The next bottleneck is the synchronous full-overlay-set materialization pause.
