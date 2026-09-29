# J0 Read-View Design

The published unit is one immutable `PublishedGeneration`. It owns the base B-link roots and catalog plus an immutable slice of overlay segments. A reader pins one `Arc<PublishedGeneration>` for its full operation, so base pages and the segment set cannot change during a Query or Scan.

Each `ImmutableOverlaySegment` owns a packed byte buffer and sorted slots. A slot stores key/value offsets and lengths, revision, and a tombstone flag. The segment owns no per-entry `Arc<Key>` or `Arc<Value>` objects. The segment-local Bloom filter rejects most point misses before binary search.

The overlay order in a view is oldest to newest. Point lookup traverses the slice newest to oldest. Range reads keep one cursor for the base and one for each overlay, repeatedly select the smallest encoded key, resolve equal keys from the newest overlay, then advance only matching sources. Tombstones produce no output. Memory usage during a range read is bounded by the cursors, one reusable key buffer, and returned documents; the keyspace is never collected into a map or sorted again.

The segment cap is four. At the cap, the logical writer synchronously materializes the current view before it admits a fifth segment. A 256 MiB WAL limit also rejects an oversized append group before writing. J4 measured that this synchronous policy fails the durable write throughput gates; the full-set materialization pause is the selected next bottleneck.

## J1 Group-Local Write Prototype

The CPU prototype pins one committed generation for the entire group. Its admission order is the mutable group overlay, committed overlays newest to oldest, then the B-link base. Accepted transactions are assigned monotonically increasing logical revisions. A failed validation or condition check does not mutate the group overlay or consume a revision.

The prototype collects each accepted group's final value or tombstone per key, sorts by encoded key through an ordered map, packs one immutable segment, then publishes a new generation sharing the same base catalog and existing segment allocations. Existing readers keep the generation they pinned before publication. The benchmark helper remains test-only until WAL v4 can make this path durable; the production transaction API still uses the H1 physical write path.

At the J1 CPU-only prototype, `TransactionResult.commit_lsn` temporarily carried the logical revision because the prototype did not encode WAL frames. J2 separates these values in the public result type; that historical bridge is no longer used by the durable logical writer.

## J2 Logical WAL Publication

The J2 constructor is `BlinkStore::open_with_logical_wal`; the retained H1 constructor remains `open_with_wal`. Logical mode admits conditions against the pinned view and mutable group overlay, prepares a packed final-state segment, appends logical mutation transactions, syncs once, and only then publishes. It performs no B-link route, leaf COW, split-fit calculation, page encoding, or PageDelta generation during the foreground logical write.

`TransactionResult` now exposes the physical `commit_lsn` and logical `revision` separately. Physical v3 engines set `revision` from their commit LSN to preserve existing values. WAL v4 stores the transaction-order revision in mutation records and the physical frame LSN in frame headers; `RevisionEquals` reads use the former.

Recovery filters WAL commits covered by the durable physical checkpoint LSN and coalesces the uncovered tail into one packed segment. Superblock version 3 stores both checkpoint LSN and logical sequence. This keeps logical revision order independent from frame count after WAL reclaim.

## J3 Materialization

The synchronous materializer clones only the currently reachable B-link tree and owned overflow pages into inactive page IDs, then applies all current overlay entries with their original revisions. The current root's pages are never overwritten before the replacement data pages and new superblock watermark are synced. Old pinned readers retain their immutable page cells while the new generation publishes.

Inactive page IDs form a reusable scratch/free pool. The materializer keeps enough capacity for an alternate base of the same live page count and extends the file as the live base grows. After the durable root switch, retired source pages are rewritten as free pages; logical open repairs the free list if a crash interrupts this cleanup. WAL reset occurs after the replacement base and checkpoint watermark are durable. Reclaim stores both the physical LSN and logical sequence in INIT.

The policy merges all four segments as one synchronous maintenance operation. The fifth group is admitted only after checkpoint returns successfully. Before-watermark failures leave the existing view and logical WAL as the source of truth. A post-watermark reclaim failure may degrade the store until reopen, while the durable base preserves all acknowledged mutations.
