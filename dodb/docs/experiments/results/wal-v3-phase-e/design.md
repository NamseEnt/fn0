# Phase E design notes: copy, allocation and reference-count removal

Branch `experiment/wal-v3-compact-redo`, on top of Phase D (`bff651e`). The algorithm, the durability rules, the page format and the WAL format are the same as Phase D. The Phase D parallel executor (2 lanes) is kept as it is.

## E0 — how the churn was counted (`b783686`)

`crates/dodb-storage/src/churn.rs`, only compiled in with the `churn-counters` cargo feature. Without the feature every call is an empty inline function and the timed binaries are built without it (every timed row is checked for `churn_counters == "disabled"`).

- Each thread carries a site tag (admission, planner, serial execution, dispatch, lane, worker thread, collect, catalog, WAL assembly, WAL append, state install, publication, dirty tracking, group other, harness). `apply_planned_transaction_group` sets the tag around each stage; the lane sets `lane` inside `run_leaf_chain_job`; worker threads default to `worker_thread`; anything outside the store (the benchmark's own request generation and its `request.clone()`) is `harness`.
- `phase0-bench` built with the feature installs a counting global allocator: every `alloc`, `alloc_zeroed`, `dealloc` and `realloc` is counted with its size under the calling thread's site.
- Source counters: `LeafEntry` clone and drop (and the Arc key / Arc value inside each), leaf page clones and the bytes of the cloned entry vector, 4 KiB page-image copies and their bytes (every place that copies a `[u8; 4096]`), heap page-image buffers, page encodes, dirty-map inserts / replaces / bytes copied, PageDelta payload buffers, delta-verify images, planner map inserts, key copies and mutation clones, and leaf jobs built.
- The benchmark writes the counter deltas over the measured interval into its JSON row (`churn_<site>_<counter>`). `scripts/churn_summary.py` divides by committed transactions.

Cycles come from `perf record -F 499 -g` on frame-pointer builds (real sync, 64w width 16 uniform, 15 s inside a 20 s measurement), grouped with the Phase D grouping (`scripts/perf_categories.py`, copied from Phase D) and, for sources, by the first dodb frame above each copy / allocator / atomic sample (`scripts/perf_sources.py`).

## E1 — 4 KiB image ownership (`ef99cdb`)

Phase D copied the same committed leaf image 4 times per touched leaf (tx-level counters, 64w width 16: 62 copies, 255 KB per transaction):

1. dispatch copied the dirty image into a `Box` for the job's delta base;
2. the lane rebuilt the after-image from the delta (`apply_page_delta`) to check it, a full copy of the base plus a 4 KiB compare;
3. the lane copied the new image into the base buffer for the next boundary of the chain;
4. dirty tracking copied the final image into `BTreeMap<PageId, [u8; 4096]>` (and the map shifted whole 4 KiB values inside its nodes).

Now:

- `dirty_pages: BTreeMap<PageId, Arc<[u8; 4096]>>`. The job takes an `Arc` clone of its base (one reference count per job, no copy). This is the only new `Arc` and it is per touched page, not per entry.
- The lane encodes straight into a new heap image (`encode_blink_page_arc`: `Arc::new([0; 4096])` compiles to allocate + zero, no stack copy), and moves it into the chain base for the next boundary.
- The final image moves from the lane result into the dirty map (the result is consumed, not borrowed).
- `page_delta_rebuilds(base, delta, image)` checks the delta in place: same base-LSN and span checks as `apply_page_delta` (shared function), then compares the unchanged gaps of base and image and the span bytes of payload and image. For unsorted spans (impossible after `decode_page_delta`) it falls back to apply-and-compare. A randomized test checks that it gives the same answer or the same error as `apply_page_delta(...) == image` for matching images, one-bit-off images and changed bases.
- The serial planned path moves its images into the WAL commits (`std::mem::take`) instead of cloning them, and the dirty map takes one `Arc::new` copy per page.

The WAL record still holds the exact image or delta of each transaction boundary; the dirty map holds the group's final image of each page; the final image lives in the dirty map after the lane result is dropped.

## E2 — whole-leaf clone, drop and Arc churn

### Why Phase D cloned and dropped every entry twice

`BlinkState.pages` owned each committed page by value, and the published catalog held its own `Arc<BlinkPage>` copy. For each touched leaf in a group:

1. the lane cloned the leaf from the published `Arc<BlinkPage>` (N entries, 2 N reference-count increments: key and inline value);
2. `prepare_delta` cloned the new leaf again to put it in the new catalog (2 N increments);
3. installing the new state replaced the old owned page in `BlinkState.pages` (2 N decrements);
4. dropping the retired generation dropped the old catalog copy (2 N decrements).

With about 15 entries per leaf and 15 leaves per transaction this was 452 entry clones and 468 entry drops per transaction, about 1,840 atomic read-modify-writes.

### E2a — one page object for the state and the catalog (`df2ce35`)

`BlinkState.pages: BTreeMap<PageId, Arc<BlinkPage>>`. The working overlay still owns mutable pages; `into_delta` wraps each one in an `Arc` once (a move, no entry copy), the catalog takes an `Arc` clone of that same object (`prepare_shared_delta`), and the install moves the same `Arc` into the committed state. So a touched leaf is cloned once (in its lane, copy-on-write from the published page) and its old version is dropped once (when the last generation that holds it retires).

Committed and published pages are never written in place: the only mutable access to a committed page goes through `Arc::make_mut` (the non-planned serial path's restamp and a checker test), which copies a shared page. Side effect: `ReadPageSource for BlinkState` now returns an `Arc` clone instead of a deep clone of the page.

Reader-generation test (`pinned_generation_pages_entries_and_values_survive_later_writes`): pin a generation, record for every page its `Arc` pointer, a deep copy, its encoded image, and each entry's key / value payload pointers and bytes; then run parallel and serial groups (16-leaf transactions, same-leaf chains), 300 inserts with leaf splits, a delete, an overflow value and a checkpoint. Every pinned page must be the same object with the same contents, image, payload pointers and bytes, and a scan of the pinned generation must return the same documents. It also checks that every committed page is the same object as the published one. Run with 0 and 2 lanes.

### E2b — short keys inside the entry (`db6499f`, reverted by `4ead3de`)

To stop touching reference counts for unchanged entries, E2b stored encoded keys of up to 62 bytes inline in the entry (the whole key one 64-byte line; the benchmark's escaped keys are about 32 bytes), keeping inline values as shared `Arc`s. Counters: key Arc clones and drops went to 0 and lane allocations fell by one per mutation. Throughput did not move (GM over Phase D 1.156 vs 1.151 for E2a, three runs each) and the lane got slower (137 → 156 µs/tx): entries grew from 48 to 96 bytes, so every lane clone and every page encode copies and reads twice as much memory. The reference-count cost came back as copy and cache-miss cost, which the plan says not to do, so it was reverted. Removing the remaining per-entry value count needs entries that do not own a reference each (for example a per-leaf byte arena), which changes every entry accessor and was not attempted.

## E3 — planner and admission temporaries (`b445043`)

Counters at 64w width 16: 162 planner allocations, 86 admission allocations and 1,283 planner map inserts per transaction; perf put `plan_batch` self time at 12% of cycles. The inserts came from the independent-leaf-group metric: for every dependency edge it inserted every (predecessor group, successor group) pair into a `BTreeSet`, about 1,200 inserts per transaction at width 16 and 5 at width 1.

- Dependent groups are marked in a `Vec<bool>` indexed by group; per-transaction groups are a `Vec` indexed by FIFO position. Same count, no tree.
- `last_key_writer` is a `HashMap<&[u8], usize>` over the admitted encoded keys (no key copy); `last_leaf_writer` and `leaf_group_indices` are `HashMap`s with capacity for the group's mutations.
- `PlannedMutation.mutation: TransactionMutation` (a clone of the document key's two vectors and the value) became `PlannedMutation.write: PlannedWrite::{Put(Arc<[u8]>), Delete}`: the value is copied once into an `Arc`, and the lane (and the serial executor for inline values) shares that `Arc` as the entry's value instead of allocating another copy.
- The admission overlay no longer stores a copy of each value; it only reads presence and revision.
- `DocumentKey::encode` reserves its exact escaped length, so keys with zero bytes (every benchmark key) no longer reallocate.

The plan's routes, groups, dependencies, provisional revisions and mutated-key sets are the same; the existing planner tests (key-copy cleanup, dependency staging, borrowed routing) pass unchanged.
