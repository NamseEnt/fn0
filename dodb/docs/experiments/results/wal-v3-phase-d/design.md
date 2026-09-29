# Phase D design — leaf-partitioned parallel physical execution

Code: `a917c84` (`blink: execute independent leaf chains in parallel`), `df8b8f4` (`blink: run leaf jobs on the coordinator lane and clone leaves in workers`), bench flags `9932f93`. This page describes the final version (D2). The first version (D1, `a917c84`) differed in two places, noted below.

## Flow of one WAL group

1. Coordinator: logical admission and `plan_batch` exactly as before (unchanged code). The plan is wrapped in `Arc<BatchPlan>` so leaf jobs can read it without copying mutations.
2. Coordinator: `prepare_leaf_parallel_execution` (only when leaf-parallel execution is enabled, `enable_parallel_execution(n)`, n ≥ 1 lanes = the coordinator plus n − 1 worker threads):
   - fewer than 2 leaf groups → serial path (`parallel_skipped_single_leaf`);
   - no page-delta WAL → serial path (`parallel_fallback_no_delta_wal`);
   - builds one step list per leaf group: `(transaction index, mutation index)` in FIFO order. The planner already appends mutations in transaction order, so a leaf's steps are A…, B…, C…; the coordinator checks this;
   - counts, per transaction, the number of distinct leaves it touches. For an eligible transaction this is its redo record count, so `commit_lsn = next_lsn + distinct_leaves` and `next_lsn = commit_lsn + 1`, the same formula the serial executor uses (`dirty.len() + metadata_changed`, with `metadata_changed = false`). All commit LSNs are fixed before any worker starts. There is no circular dependency: the record count comes from the plan, not from execution. The worker results are then checked: every transaction must return exactly that many boundaries, on distinct leaves, each with the precomputed commit LSN, or the group is an invariant error (before the WAL);
   - any put larger than the inline limit (512 B) → serial path (`overflow`), checked before dispatch;
   - one job per leaf: the committed leaf as an `Arc<BlinkPage>` taken from the published generation (the same page as the committed working state; debug builds check equality), the WAL page-chain entry of the leaf (LSN, CRC32C), and a copy of the leaf's current dirty image when the chain has one (the delta base). Jobs share the plan and the commit-LSN array through `Arc`;
   - all jobs go into one queue. The coordinator wakes the n − 1 persistent worker threads and then drains the same queue itself; every lane takes chunks of `max(1, jobs / (8 × lanes))` jobs until the queue is empty, then the coordinator collects the worker results.
   - D1 differed here: the coordinator cloned each leaf (`BlinkPage` clone) while building the jobs, split the jobs statically between n worker threads, and only waited for them.
3. Lane (`run_leaf_chain_job`, on a worker thread or on the coordinator): clones the leaf out of the shared `Arc`, then per transaction segment in FIFO order:
   - applies the mutations in place, with revision = commit LSN (what the serial restamp produces), page LSN = commit LSN;
   - returns a fallback reason instead of a result if a key is not below the leaf high key (`route`), an existing value is an overflow value (`overflow`), or the leaf no longer fits (`structural`: the serial executor would split, or reject the group for an oversize update);
   - encodes the canonical 4 KiB page (`encode_blink_page`);
   - computes the image CRC32C, then either a full image (leaf not yet in the WAL chain = first touch after reset, or a delta not smaller than an image) or a PageDelta against the previous committed image of this leaf: the pre-group image for the first segment, the previous segment's image afterwards. The delta is decoded and applied back to the base and must rebuild the image, the same check the WAL did in Phase B;
   - the base of the first segment is checked against the WAL chain entry (LSN and CRC32C). If the dirty map has no copy (after `flush()`), the worker encodes the committed page itself.
   - Result: per-transaction boundaries (transaction index, commit LSN, image CRC, image or delta payload), the final `BlinkPage`, and the final image (for the dirty-page map), plus worker-side timers.
4. Coordinator after the join: any fallback in any job → the whole group goes to the unchanged serial executor with the same plan. Nothing has been written. Otherwise it installs each final page in the working overlay (move, no clone), builds one `ExecutedPlanTransaction` per transaction (superblock generation +1 each, slot unchanged, no superblock image), and sorts each transaction's records by page ID — the order the serial executor writes its dirty pages.
5. WAL: `WalLog::append_group_prepared` takes borrowed records (`PreparedWalRecord`: page ID, page LSN, image CRC, image bytes or delta payload with base LSN/CRC). Before writing anything it re-checks: batch IDs and commit LSNs in sequence, `commit_lsn = first_record_lsn + records`, records in page order, no superblock page, page LSN = commit LSN, delta header page ID and base LSN, and — the Phase B chain rule — every delta's (base LSN, base CRC) equals the latest committed image of that page (earlier in this group, or the persistent chain). With a fault injector, and in debug builds, it also validates every image with the full Blink page validator, checks image CRCs and decodes every delta payload. Frames, digests and the commit record are built by the same helpers as the serial path. One physical write, one sync; the fault-injected path writes frame by frame with the same fault points.
6. Publication, state install, dirty tracking: unchanged code; the dirty-page map gets only the final image of each leaf.

Lanes never touch the WAL, the published generation or the store. A worker error or panic becomes a group error before the WAL append; the store is not marked broken because nothing was written.

## Eligibility (the whole group or nothing)

Parallel only if all hold; otherwise the unchanged serial executor runs the group:

- a worker pool exists and the group has at least 2 target leaves;
- the WAL is format 3 with page deltas (planned Blink);
- every put value is inline (≤ 512 B);
- every target page is a leaf in the committed state;
- every key is below its leaf's high key (the planner routed from the committed state; no split happens inside an eligible group, so the route cannot change);
- no mutated key currently holds an overflow value (no allocator / free-list change);
- after every mutation the leaf still fits (no split, no oversize update).

Consequences: no superblock image, no root/internal page, no allocator, overflow or page reuse change, and exactly one redo record per touched leaf per transaction. Inserts and deletes (tombstones) that fit in place are eligible; they change only one leaf.

The last three conditions are only known after executing, so they are checked by the workers; a fallback found there costs the dispatch (`parallel_fallback_after_dispatch`) but still happens before the WAL.

## Ownership and copies (per touched leaf)

| Item | Serial planned path (Phase C) | Leaf-parallel path |
|---|---|---|
| Leaf clone | `ensure_overlay_page` clone on the coordinator | one clone, done by the lane that runs the job (D1: on the coordinator) |
| Mutation | in place in the overlay, then restamp pass | in place in the job's page, revision set directly |
| Page encode | coordinator, into `WalPageImage` | worker, stack buffer |
| Image copies before WAL | `images` vec → `wal_commits` clone → `final_images` map | none for deltas; a boxed copy only for image records (first touch) |
| Delta base | WAL callback borrows the dirty image or re-encodes | a copy of the dirty image in the job (4 KiB), or a worker encode after `flush()` |
| Delta encode + verify + chain CRC | coordinator inside the WAL | worker |
| Final page into state | move | move |
| Dirty map | copy of the final image | copy of the final image |
| Published catalog | page clone into a new `Arc` (unchanged) | same |

The mutations themselves are not copied into jobs (the jobs read them through the shared `Arc<BatchPlan>`). Values and keys become new `Arc` payloads exactly as in the serial path.
