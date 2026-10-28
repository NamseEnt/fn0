# B-link active-tenant density: cost audit and improvement plan

Tracking issue: [NamseEnt/fn0#126](https://github.com/NamseEnt/fn0/issues/126). Start with [HANDOFF.md](HANDOFF.md) when continuing in a new session.

Status: A1 implementation and local correctness validation are tracked in [A1-IMPLEMENTATION.md](A1-IMPLEMENTATION.md) and [A1-RESULTS.md](A1-RESULTS.md). Executable harness/process-accounting preparation and four Linux correctness smokes are complete; see [DENSITY-HARNESS-RESULTS.md](DENSITY-HARNESS-RESULTS.md). The original analysis below is the frozen preparation baseline. Density measurements, shared-resource RocksDB reference, default promotion, merge, and deployment remain future work.

## Objective and decision criteria

Build a single-machine service with separate tenant storage files that has a meaningful memory and disk advantage over an equivalent DODB-contract RocksDB service while satisfying throughput and latency requirements for many concurrently active tenants.

Do not assume B-link is preferable. Include the case where every registered tenant receives sustained requests. Optimize the number of active tenants meeting the same service objective, not just maximum single-tenant throughput. Report the throughput, latency, recovery, and temporary-space price of lower resource use.

Separate measured facts, source-confirmed structure, and unverified hypotheses. Fix the workload and resource conditions before drawing each conclusion. Distinguish general B-tree/LSM properties from this implementation. Preserve failed attempts and existing evidence.

The current conclusion is that local optimizations can reduce waste and peaks, but the current fully resident tree cannot meet a larger-than-memory, many-active-tenant objective through local tuning alone. Disk efficiency is not established either. This is a source-based architectural judgment, not a new density measurement.

## Frozen baseline and working location

- Repository: NamseEnt/fn0.
- Reviewed source: a7271e9e35c5e0fa9d2a1c0e5eedc030f12de497.
- Source branch: experiment/backend-hotpaths-20261026.
- Source worktree: /Users/namse/fn0-dodb-backend-hotpaths.
- The source branch, HEAD, clean worktree, and remote source-branch SHA were rechecked during preparation.
- Continuation branch: experiment/blink-tenant-density-20261027.
- Continuation worktree: /Users/namse/fn0/.worktrees/dodb-tenant-density-20261027.
- Work from the fn0 monorepo. Do not use or modify /Users/namse/dodb.
- Preserve the previous worktrees, benchmark databases, rejected attempts, and result archives.
- The new branch starts from the reviewed experiment commit, not main. It therefore retains the measured implementation and evidence.
- Leave the existing main checkout and its branch unchanged. Keep history linear; do not merge or force-push.

All repository documents, issues, and commit messages are English. Conversational updates are Korean. Do not add source comments, single-letter abstract variable names, lint suppressions, or Co-Authored-By trailers.

## Existing evidence and limits

Read these reports first:

- [Backend hot paths](../results/backend-hotpaths-20261026/RESULTS.md)
- [Backend profiling](../results/backend-profiling-20261026/RESULTS.md)
- [Query/Get/Put mixture](../results/query-get-put-mix-20261027/RESULTS.md)

Follow their plans, summary JSON, and original evidence as needed. The profiling source was cc80d1fdc7a98792be1a739056a3013debf39268, before the accepted hot-path changes. The hot-path candidate production source was 3b94ee88eb4e7c02416cde14d3ac48c55e45b991. The Query mixture measured source 54812222e95e1335a5e9409d3fddbabf38c0f876. Later report commits do not turn older profiles into current profiles.

The following changes already exist and must not be proposed again:

1. Open B-link shard reads consult the registry before path construction and filesystem existence checks.
2. Minimal-native mutation-only keys retain exclusive GetForUpdate locking and conflict validation but pass a null value pointer to avoid fetching and decoding unused old values. Condition keys retain their existing read/validation path.

### Separate measured conditions

Each row is a separate workload, with three 300-second repetitions and real synchronous WAL. Compare backends within a row, not workloads across rows.

| Condition | B-link / minimal-native throughput | B-link / minimal-native sampled group RSS | B-link / minimal-native maximum directory size |
| --- | --- | --- | --- |
| RW80, 16 callers, 512-byte values, transaction widths 1/4/8/16 | 17,291.53 / 13,877.48 API calls/s | 1,145.02 / 610.61 MiB | 739.23 / 420.87 MiB |
| Unconditional writes, 64 callers, width 16, 512-byte values | 1,822.56 / 2,263.24 batches/s | 1,655.78 / 1,011.15 MiB | 864.85 / 549.96 MiB |
| Query/Get/Put/conditional Put = 40/30/15/15, one tenant, 10,000 seeded rows, 16 callers, 512-byte values | 19,068 / 20,739 API calls/s | 270.2 / 523.5 MiB | 143.9 / 69.3 MiB |

Under RW80, B-link has higher throughput and lower upper-tail transaction latency, but higher p95 and group RSS. Under unconditional width-16 writes, improved minimal-native has higher throughput and lower p95/RSS/directory size. Under the small Query mixture, B-link has lower RSS and Query/Get p50/p95, while RocksDB has higher aggregate throughput and lower Get p99.

The Query limit was 16, but the seeded layout contained at most ten sort keys per partition and actually returned about five rows per call. It is not evidence for a full 16-row page.

Group RSS includes the driver, runtime, seeding, and validation; it is not retained engine heap. Reopen/oracle combines lazy backend opening and complete state validation, not isolated recovery. Reported directory sizes do not establish physical allocation or device writes. Shared hard ceilings do not establish equal actual memory use.

These reports do not prove large-scale active-tenant density, a common actual-memory-budget comparison, or larger-than-memory behavior. Historical inclusive CPU samples overlap and must not be added or treated as current causal evidence. The old allocator profiles are CPU costs, not retained-heap attribution. Old syscall elapsed totals sum across threads and do not establish serialized pauses.

Defaults remain legacy WAL, synchronous checkpoint execution, and disabled automatic checkpoints. Measured background coalesced durable-base runs used different explicit experimental settings; keep those conditions separate.

## Current source map

Use the reviewed commit for stable line references; recheck line locations after future changes.

| Component | File and principal functions |
| --- | --- |
| Tenant lifecycle and request execution | dodb/crates/dodb-server/src/lib.rs: BlinkTenantService, AsyncBlinkShard, read_shard, open_shard |
| Machine-wide physical checkpoint scheduling | dodb/crates/dodb-server/src/physical_checkpoint.rs: PhysicalCoordinator, semaphore, collect, apply_group, start, finish |
| Resident state and publication | dodb/crates/dodb-storage/src/blink/mod.rs: BlinkStore, BlinkState, PageCell, PageCatalog, PublishedGeneration, GenerationPin, GenerationPublisher |
| Mutation and reads | same file: apply_planned_transaction_group, WorkingBlinkState, LogicalOverlay, ReadPageSource, allocate_value, free_value, allocate_page |
| Checkpoint snapshots and capacity | dodb/crates/dodb-storage/src/blink/physical_checkpoint.rs: prepare_segmented_checkpoint, BlinkCheckpointSnapshot::run, complete_segmented_checkpoint, check_segmented_group_capacity |
| Pack format | dodb/crates/dodb-storage/src/blink/durable_bases.rs: BasePackJob::run, read_pack, encoded_size |
| WAL segments and publication | dodb/crates/dodb-storage/src/blink/segmented_wal.rs: open, rotate, base_job, advance_checkpoint_with_base, cleanup_obsolete_segments, manifest write_atomic |
| WAL encoding and recovery | dodb/crates/dodb-storage/src/wal.rs: WalLog, page_chain, plan_redo, append_group_inner, finish_group_append, reset |
| Packed leaf representation | dodb/crates/dodb-storage/src/blink/leaf.rs: LeafEntries, replace, Clone, compact_if_sparse |
| Equivalent RocksDB contract | dodb/crates/dodb-rocksdb/src/minimal_native.rs and shim/transaction_shim.cc |
| Existing harness | dodb/crates/dodb-rocksdb/src/bin/service_workload.rs |

## Memory cost model

### Fully resident current tree

This implementation requires the whole tree in memory; B-trees in general do not.

BlinkState.pages is a BTreeMap of PageId to Arc<BlinkPage>. load_file_with_validation (mod.rs:2235) reads and decodes every page from the first data page through high-water, including free pages. The durable-base load_base_images path (mod.rs:2053) also constructs the complete allocated range.

PublishedGeneration owns a catalog of page cells and overlay references. ReadPageSource for GenerationPin obtains an existing memory page; it has no disk miss/load path. PageCell contains an Arc to an immutable decoded page, not an address from which an evicted version can be reconstructed.

State and the current catalog share payload through Arc. They are not two complete payload copies. They do maintain separate indexing/publication metadata. The catalog uses 64-entry chunks and a directory indexed by page ID. Every publication clones the directory and replaces affected chunks (mod.rs:1421), so directory work and metadata grow with high-water.

### Retained and temporary objects

Opening an existing tenant retains the store, complete decoded pages, map/catalog metadata, publisher/read handle, database/WAL handles, WAL chain metadata, coordinator task, request channel, and failure/activity state. A coordinator task is not a dedicated OS thread per tenant.

Each leaf retains a slot vector and packed key/inline-value bytes. Existing same-length inline replacement and leaf garbage compaction already exist; do not propose them as new optimizations. Internal keys and overflow value chunks are resident too. Free pages remain represented in maps/catalogs.

dirty_pages stores a separate encoded 4 KiB image for each latest dirty page. The decoded current page and encoded dirty image coexist. WAL page_chain stores page LSN/CRC metadata, not a retained full image. Durable-base mode additionally tracks page-to-pack owners.

During a write, queued request payloads, cloned execution requests, encoded keys, logical admission metadata, planned writes, working decoded page copies, transaction-boundary images, delta payloads, and the final encoded WAL group can overlap. Transaction boundaries must survive optimization.

Generation pins retain older catalogs and changed page objects. Unchanged payload is shared. The retained version count is not capped by a global memory policy; it depends on reader lifetimes and mutation overlap.

Physical checkpoint preparation moves dirty images into a snapshot rather than immediately copying all of them. New writes can then create newer dirty images for those same pages. A full base snapshot also retains Arc references to clean decoded pages; the worker encodes them into an image map before pack output. References initially share payload, but later mutations retain old decoded versions and newer pages concurrently.

BlinkCheckpointSnapshot::retained_bytes is normalized page-image accounting, not an exact measurement of heap capacity, BTreeMap nodes, decoded buffers, or allocator retention. Do not use it as an exact resident-memory cap.

check_state (mod.rs:8964) constructs reachable, free, expected, known, and other sets. These produce page-count/high-water-dependent temporary allocations during opening and checkpoint validation.

The approximate model is:

current decoded payload + current indexing metadata + dirty encoded images + old versions pinned by readers/snapshots + queued payload + transaction/WAL preparation + checkpoint output + shared runtime/allocator overhead.

Do not assign a numeric multiplier without measurement.

### Cache and lifecycle limits

DatabaseConfig.cache_capacity=128 does not bound B-link residency. PageCache uses this setting in BTreeStore; B-link has no corresponding eviction mechanism.

BlinkTenantService has no global memory budget, page eviction, or automatic idle-tenant close. max_open_shards limits the number of open shards and rejects opening when full. It is not an LRU. shutdown clears all open shards.

Global shard registry locking currently covers synchronous opening/recovery. Slow opening can delay access to other tenants. Minimal-native also takes its tenant_open mutex before its existing-tenant fast path.

### Source-confirmed reuse concern

In the service's planned write path (mod.rs:3131), committed_view is pinned for admission at line 3159. Dropping the logical overlay does not drop committed_view. That pin is still alive when can_reuse_pages is evaluated around lines 3280/3295. can_reuse_pages (line 1538) requires zero active generation pins.

Thus this path supplies false for page reuse even without an external reader because the writer itself owns a pin. This is a static control-flow finding, not a measured leak or throughput cause.

Values up to 512 bytes are inline. Larger values allocate overflow pages. Replacement allocates the new chain before freeing the old chain. With reuse denied, repeated updates of a fixed key set can increase high-water. The resulting free objects, catalog range, and checkpoint images can accumulate. Existing 512-byte experiments do not characterize this condition.

Even after shortening the writer pin, a global zero-reader reuse rule can remain restrictive under continuously overlapping reads. A long-term solution needs a safe epoch/version-based retirement rule.

## Disk cost model

Keep storage modes separate.

| Artifact | Role and overlap | Reclamation |
| --- | --- | --- |
| Legacy .db | Stored page images and double superblocks; may lag acknowledged writes | Free slots can be reused, but normal code does not shrink the file tail |
| Legacy .wal | Synchronously committed image/delta recovery history | Flush/sync data, sync metadata, then reset/sync WAL |
| Segmented WAL without durable bases | .db plus closed and active segment history | Publish the checkpoint durably before deleting covered closed segments |
| Durable-base packs | Checkpoint page images that form replay bases; newer WAL remains above them | Drop packs with no remaining page owners or replace them with a complete base |
| .db in durable-base mode | Initially created bootstrap pages; packs are the maintained primary checkpoint representation | Do not count this as another full current database image |
| Segment manifest | Identity, root/high-water, checkpoint boundary, pack/segment inventory | Temporary file sync, rename, parent-directory sync |
| Unpublished/retired files | New pack can coexist with old packs/WAL; crashes can leave unreferenced artifacts | Validated identity/coverage/checksum-aware cleanup, not indiscriminate deletion |

Legacy default automatic checkpoints are disabled. Do not assume bounded WAL growth for a perpetually writing tenant under those defaults.

The .db-mode peak consists of allocated .db space, retained WAL, writes arriving during checkpoint, and temporary metadata. The pack-mode peak consists of all old packs, the new pack being written, retained/new WAL, bootstrap .db, and temporary metadata. There is no universal fixed multiplier.

base_job chooses full output for initial coverage, pack-count limits, projected duplication, or budget pressure. MAX_PACKS is eight. One still-owned page can keep an old pack file alive. Full rewrite temporarily duplicates old and new representations.

prepare_segmented_checkpoint retains every state page for a full base, including free pages. Full pack rewrite does not by itself compact page IDs or lower high-water.

The current 512 MiB default retention setting also applies the condition (page_count + 2) * 4096 <= limit to a durable-base snapshot. A sufficiently large full snapshot can therefore hit a checkpoint-capacity error even if WAL headroom seems available. The setting is not a complete process heap cap or a general large-tenant solution.

Delete retains the key and logical tombstone revision. Overflow pages become free, but the physical mutation path does not merge sparse leaves or truncate the database tail. Distinguish leaf-buffer garbage compaction, free-page reuse, logical file shrink, and physical space returned to the filesystem.

On recovery, durable-base/WAL raw images, decoded pages, newly recovered dirty images, and replay preparation can overlap in memory. Current recovery can rewrite/extend .db in place in the non-base modes; do not invent a mandatory full temporary recovery file.

Always record separately:

- Logical file length.
- Actual allocated file and dataset space, including temporary/unlinked-open files where applicable.
- Cumulative application writes.
- Actual block-device writes, including filesystem amplification.
- Time from deletion/checkpoint completion to reusable or physically returned space.

Do not infer physical writes from pread calls or directory logical sizes.

## Execution and shared-resource costs

The write path is request queue -> tenant-local group collection -> conditions/revision/planning -> page mutation/encoding -> WAL append and sync -> generation publication -> response.

Defaults are 256 queued requests, up to 64 requests and 4 MiB per group, and zero collection delay. A request-count queue limit is not a byte limit for the queued payload. A per-group byte limit does not bound all tenant queues.

The coordinator clones requests into an execution vector and dispatches blocking storage work. Planned execution performs key encoding, dependency/routing work, copy-on-write page mutation, catalog preparation, transaction images, and image/delta WAL preparation. WalLog::finish_group_append synchronizes the WAL before successful results/publication are exposed.

With many active tenants, each tenant may form smaller groups. Single-tenant grouping benefits cannot be assumed to survive. Sharing workers or sync scheduling does not make separately stored WAL files durable through a single file sync.

PhysicalCoordinator already uses a process-global semaphore with capacity one for segmented physical checkpoints. Do not propose a global checkpoint concurrency cap as if none exists. The limit prevents all physical workers from running at once, but waiting tenants can continue accumulating WAL/dirty state and eventually stall or hit capacity. The sum is not governed by a machine-wide byte budget.

Sustained stability requires aggregate checkpoint/reclamation work to remain below the machine's service capacity. Otherwise backlog, resource use, or latency grows. A larger concurrency setting does not establish stability and can increase interference.

Share cache budgets, bounded buffers, execution pools, checkpoint bandwidth, and admission policy. Preserve separate tenant files/WAL, revision domains, transaction order, identity, and recovery/publication state.

The optional logical-WAL/immutable-overlay path is separate from the measured physical-WAL service configurations. It has packed immutable overlay segments, publication limits, background materialization, and whole-tree materialization copies. Do not add that path's steady-state costs to physical-WAL measurements or assume it is an already accepted remedy. Analyze its base-plus-overlay merge/read costs and pinned materializer state separately if revisiting it.

## A: Optimize while retaining the resident page model

Effects below are directional hypotheses. None has a newly measured gain.

| Priority/candidate | Source and present cost | Scope/difficulty | Trade-offs, minimum verification, implementation judgment |
| --- | --- | --- | --- |
| A1: Writer pin lifetime and reuse decision | apply_planned_transaction_group and can_reuse_pages retain the writer pin across the zero-pin test | Medium; identify the exact end of admission references and prove safety against new readers | Expect less high-water/free-metadata accumulation under overflow churn. Check no-reader and long-reader cases, alloc/free/reuse counters, file/pack size, and crash/reopen. First candidate to validate. This alone does not fix continuous external reader retention. |
| A2: Stream full-base encoding/output | BlinkCheckpointSnapshot::run collects all encoded pages before BasePackJob::run | Medium/high; bounded batches with unchanged format, snapshot identity, checksums, sync and manifest ordering | Lower peak memory expected; output call count or snapshot duration may affect tails. Measure current/peak bytes and checkpoint throughput; fault every publication/reclamation boundary. Worth pursuing; whole decoded tree remains resident. |
| A3: Compact invariant-check working state | check_state builds several page-ID BTreeSets and complete expected-ID sets | Medium; use compact visit state/bitmaps or equivalent representations | Lower validation peak/search cost expected. Preserve every cycle, ownership, leak, range, sibling and checksum check. Corruption cases must remain rejected. Worth doing after attribution; do not remove validation work merely to make a result look better. |
| A4: Move request ownership and bound scratch reuse | Coordinator request clones; planned keys/values, transaction images, delta payload and encoded WAL group overlap | Medium; map stage lifetimes before changing ownership | Fewer allocations/copies expected. A large request must not leave an oversized scratch buffer retained per tenant indefinitely. Retain transaction boundaries, commit ordering and WAL validation. Follow the structural work. |
| A5: Separate tenant opening from registry locking | B-link opens/replays while holding the shared shard registry lock; minimal-native locks tenant_open even for open-tenant lookup | Medium; per-tenant initialization, duplicate-open and shutdown races | Lower cross-tenant cold-open interference expected; no change to full-residency complexity. Measure open latency and existing tenants' p99 together. Isolation improvement, not a density solution by itself. |

Catalog directory cloning also grows with high-water. If current stage metrics show it is material, a hierarchical persistent directory could reduce publication work. This changes publication indexing rather than physical page residency. Do not claim each publication currently copies all page payloads.

Do not propose existing packed leaves, same-length replacement, leaf compaction, open-shard metadata-check removal, or mutation-only RocksDB old-value removal as new work.

## B: Structural changes for active-tenant efficiency

### B1: Durable page-version locators, demand loading, and a global cache

This is required for the larger-than-memory objective and has high scope/difficulty.

A page that can be evicted must have an exact reload contract. Identify it by tenant, logical page ID, reuse incarnation, and required page LSN/version. A generation pins a version map/root, not just whichever page currently occupies an offset.

- Checkpoint-covered pages load from a validated .db location or an immutable pack plus exact record offset.
- Post-checkpoint pages reconstruct from a synchronously committed WAL image and its bounded delta chain.
- Maintain a locator/index so a cache miss does not scan the entire WAL or every pack.
- Check page identity, incarnation, required LSN, checksums, base LSN/CRC, and the reconstructed result.
- Bound replay chain length through durable image materialization.
- Coalesce concurrent loads and account for loading buffers.
- Evict dirty pages only when their committed version remains reconstructable from durable data.

Current page_chain contains LSN/CRC, not replay offsets. base_owners contains pack ownership, not a complete version/offset index. Current .db offsets are overwritten and old packs/WAL are reclaimed relative to the latest checkpoint. Old readers are safe today because old payload remains in RAM. Eviction changes that premise.

Reclamation must retain every source still reachable from pinned generations or recovery roots. Old readers can then increase disk retention; quantify that trade-off and define reader-lifetime/backpressure behavior without breaking read consistency.

Change both reader and writer page access, mutation lifetime management, checkpoint encoding, and recovery. A payload cache alone still leaves O(total allocated pages) catalogs, owner maps, WAL indices, and opening scans. Make those structures hierarchical or disk-backed as needed. Preserve current corruption validation through streaming validation or authenticated validated indices; do not silently replace it with weaker lazy checks.

Expected effect: resident payload follows the working set and budget instead of the complete dataset. Costs: cache-miss I/O, decoding/redo CPU, possible read-tail regression, more complex recovery and GC.

Minimum experiment: one tenant with data larger than cache, forced eviction of current and pinned-old versions, a dirty page reloaded from WAL, a checkpoint race, and crash/reopen/write/crash/reopen. Advance to many tenants only after correctness. High priority if pursuing the stated goal.

### B2: Global byte budgets, checkpoint debt, and fair execution

High scope/difficulty, paired with B1.

Account globally and per tenant for queued payload, current dirty images, checkpoint snapshots, old pinned versions, WAL/pack reclamation debt, and preparation buffers. Provide bounded shared capacity with per-tenant service floors and caps rather than a large fixed reserve for every tenant.

Schedule checkpoint work by pressure/debt and fairness as well as individual WAL thresholds. Keep the existing one-worker cap as the starting control; change concurrency only with evidence. Record time waiting for capacity, per-tenant threshold excess, and aggregate debt. Add bounded foreground admission rather than allowing arbitrary aggregate backlog.

Share workers/budgets but preserve per-tenant serialization, files, WAL sync requirements, revisions, and identities. Idle-tenant closing can be added with safe drain/reopen semantics, but cannot be credited as the solution when all tenants are continuously active.

Expected effect: bounded machine peaks and controlled interference. Costs: queuing, rejection, and possible throughput/latency loss when budgets bind. Measure synchronized versus staggered checkpoint demand and verify all tenants meet SLOs. High priority.

### B3: Epoch-safe reuse and actual space reclamation

High scope/difficulty.

After A1, replace the all-or-nothing zero-reader condition with retirement epochs/version ownership so pages safe for the oldest active generation can be reused. Prove safety for page-ID reuse, stale links, old WAL records, checkpoint races, and restart.

Actual disk return additionally needs sparse leaf merge/repacking, verified free-tail truncation or compact relocation, and pack GC based on live occupancy. Preserve old reader/recovery sources until safe publication. Include old-plus-new temporary space and relocation writes.

Expected effect: lower retained metadata and disk use under churn. Costs: extra writes, temporary space, maintenance interference, and more complex recovery. Test fixed-key overwrites, fixed-key delete/reinsert, and ever-growing distinct deleted keys separately. Required for a disk-efficiency claim.

Tombstone revisions cannot be discarded arbitrarily. Both implementations retain logical missing-state revisions for DODB semantics. Continually introducing distinct deleted keys has a contract cost; do not compare against an LSM that deletes that information.

### B4: Value placement and the inline boundary

Medium/high scope/difficulty.

The 512-byte inline boundary sends a 513-byte value into a separate 4 KiB overflow page. Reuse fixes do not remove that internal fragmentation. Evaluate a different inline policy or packing several values into extents.

Expected effect: lower allocation and cache footprint for affected sizes. Costs: reduced leaf fanout, value-GC complexity, additional I/O, and format/recovery work. Test 511/512/513 bytes, 1 KiB, and 4 KiB before implementation. Prioritize only if the actual product value distribution makes the boundary important.

## Fair RocksDB comparison

The comparator is minimal-native DODB-contract TransactionDB, not raw WriteBatch. It preserves conditional/multi-key transactions, sorted key locks, logical revisions/reservation, tombstones, snapshot Query/Scan, response budgets, and synchronous WAL commit. Mutation-only lock-without-fetch already exists.

The existing minimal-native route creates default Options per DB, sets create_if_missing and no compression, and does not pass shared block-cache or WriteBufferManager objects. Its default Env is already shared, so background pools are not necessarily duplicated per DB. The new opt-in minimal-native-shared route now implements explicit cache/WBM sharing and background-pool configuration; see [SHARED-ROCKSDB-RESULTS.md](SHARED-ROCKSDB-RESULTS.md) for its soft-limit policy and acceptance evidence.

| Resource | Available shared configuration | Existing minimal-native route |
| --- | --- | --- |
| Block cache | Same cache object across independent tenant DBs | No explicit shared cache |
| Memtable budget | Same WriteBufferManager across DBs, optionally charged to cache | No explicit shared WBM |
| Flush/compaction workers | Shared Env pools | Default Env::Default pools shared |
| Tenant storage and transactions | Independent databases | Separate DBs retained |

Pinned RocksDB 11.8.1 describes a default 32 MiB block-cache capacity. Capacity is not eager RSS allocation.

Primary sources:

- https://raw.githubusercontent.com/facebook/rocksdb/v11.8.1/include/rocksdb/table.h
- https://raw.githubusercontent.com/facebook/rocksdb/v11.8.1/include/rocksdb/write_buffer_manager.h
- https://github.com/facebook/rocksdb/wiki/RocksDB-Tuning-Guide

Use three configurations in the initial density comparison: current B-link, current minimal-native, and contract-equivalent minimal-native with shared cache/WBM and explicitly recorded global worker limits. A later B-link candidate must be compared against the shared-resource reference.

WBM/cache limits do not cap every native allocation, pinned iterator, table metadata object, or filesystem cache. Record actual use. Small buffers may increase flushes, small SSTs, compaction, stalls, and disk overhead. Include those costs.

Keep compression disabled for the first layout-controlled comparison. Before claiming practical deployment disk superiority, separately evaluate a reasonable compressed RocksDB configuration. Do not call the current adapter maximum-tuned RocksDB.

## Minimal experiment design: future work only

### Host and budgets

The historical host has two AArch64 Neoverse-N1 cores and used ZFS /bench/zfs/db with sync=standard, recordsize=4K, compression=off, and atime=off. Historical available-memory samples around 5 GiB are not a current capacity guarantee.

Before running, inspect live RAM, MemAvailable, ARC, CPU, disk allocation/free space, and current artifacts. Propose an initial 1 GiB backend-process resident-memory budget; isolate the driver and record its memory. Record OS/page-cache/ARC usage separately. If a process/cgroup limit does not cover ZFS memory, explicitly state that boundary.

A dataset exceeding the engine/process budget is not proof of physical disk misses. Include a genuinely storage-read-sensitive condition if making disk-miss claims and the host can support it.

Size each cell from measured setup expansion and peak overlap, not payload alone. Reserve host memory and filesystem space, including the established 10 GiB free-filesystem reserve, validation, failed-database retention, and archives. Reduce the cell if it does not fit. Preserve existing artifacts. The sizes below are proposals, not capacity results.

Performance evidence remains on the designated OCI host opc@217.142.246.204, ZFS /bench/zfs/db. Use /Users/namse/.ssh/id_late_sh_ed25519 with IdentitiesOnly=yes and normal host-key verification. Do not substitute a local Mac, Debian, or another host.

### Stages

| Stage | Proposed scope | Question |
| --- | --- | --- |
| E0: Open costs | Registered 1/32/128/512; vary opened count independently; empty and populated DBs | Fixed per-open-tenant cost versus data-dependent residency |
| E1: Reuse diagnosis | One tenant, fixed keys, 512/513-byte and 4 KiB repeated overwrite; no external readers versus a long reader | Writer pin, external pin, free-list reuse, and high-water growth |
| E2: Resident-density sweep | About 1 MiB value payload per tenant; active counts 1/8/32/128/256, optionally 512 if safe | Per-active-tenant growth under a resident aggregate dataset |
| E3: Memory-pressure sweep | About 16 MiB value payload per tenant; active 8/32/64/128, aggregate about 128 MiB to 2 GiB | Below-budget versus above-budget behavior |
| E4: Maintenance pressure | Near the density boundary, synchronized versus staggered checkpoint demand and sustained churn | Backlog stability, reclamation, peak space/memory, and fairness |

Core E2/E3 cells use registered=active, with sustained requests to every tenant. Separately test registered=512/active=8 and distinguish registered-but-never-opened from opened-but-idle. Do not mix sparse-active conclusions into all-active results.

### Workload definition

- Keys: 16-byte PK plus 16-byte SK; record actual canonical encoded lengths.
- Main values: 512 bytes; repeat boundary-sensitive cells at 4 KiB after E1.
- Query: at least 256 rows per partition and actual 16-row returns; record rows/call. Keep Scan checks separate and add a long-read stress cell.
- Read-heavy: Get 80%, Query 15%, writes 5%.
- Write-heavy: Get 10%, Query 10%, writes 80%.
- Mixed: Get 40%, Query 20%, writes 40%.
- Within writes: half unconditional and half conditional requests.
- Proposed transaction-width mix: 80% width 1, 15% width 4, 5% width 16; freeze before running.
- Main conditions: Exists and revision comparisons. Correctness cells also include NotExists, multi-key overlap, intended rejection, and tombstone/ABA behavior.
- Ten percent of mutations are Delete. Keep fixed-key delete/reinsert separate from growing distinct-key tombstone history. State overwrite/new-key ratios and retain equal initial/final logical state.
- Main access is uniform. Only boundary sensitivity adds 90% of requests to the hottest 10% of keys.
- Fix seed, initial state, key/value bytes, schedule fingerprints, durability, and response limits.
- Record successful commits and rejected/conflicting requests separately. Do not let one backend silently do a different amount of logical work.

### Two load topologies

1. Fixed total clients: 16 logical streams multiplex all tenants. Total concurrency stays fixed and per-tenant demand falls as active count rises.
2. Fixed per-tenant offered load: initially propose 50 API calls/s per tenant, then 25/100 sensitivity near the boundary. Aggregate load rises with active count. The rate is a proposal, not a measured attainable SLO.

For offered-load tests, measure latency from scheduled arrival through queueing to completion. Count backlog, dropped, rejected, and timed-out requests; avoid coordinated omission. Logical clients need not imply an OS thread per tenant. Record and hold executor/admission settings fairly across backends.

### Harness preparation required before running

service_workload currently hard-codes one/four-tenant patterns, limits B-link open shards to 16 in the normal setup, has a client key-space bound, and uses execution modes with different synchronous/asynchronous behavior. It is not already a general active-tenant density harness.

The existing backend-process memory diagnostic worker does not support segmented checkpoint configurations. Extend the harness/worker to configurable tenant counts, separated backend accounting, explicit offered load, and the measured segmented-base path before claiming this design is executable. Do not mistake a cache-capacity field in report JSON for an implemented B-link cache.

### Service objective and metrics

Provisional thresholds, pending a product requirement: per-tenant Get p99 <= 5 ms, Query p99 <= 10 ms, write p99 <= 50 ms. These are proposed experiment thresholds, not accepted production requirements or results.

Define capacity as the greatest active tenant count where every tenant sustains its offered rate and meets the same latency/error/resource objectives. Report the full resource/latency frontier and percentage regressions, not only a winner.

Record:

- Backend RSS/PSS/anonymous, allocated versus retained heap, and driver memory.
- Current decoded bytes, catalog/map bytes, dirty images, old versions, pin count and age, queue bytes, preparation bytes, and checkpoint output bytes.
- Normal and checkpoint peak memory, sampling gaps, and memory increase per tenant.
- File logical lengths, actual allocated blocks/dataset space, normal and instantaneous peak disk use, and temporary/unlinked-open files where relevant.
- Application WAL/pack/DB write bytes, actual device writes, sync counts/durations, and filesystem amplification.
- Free/reuse/allocation rates, high-water, pack live occupancy, reclamation debt, and time to reusable/physically returned space.
- Total and per-tenant offered/completed throughput, operation latency, worst-tenant behavior, errors, queueing, and fairness.
- Newly created open, clean reopen, crash recovery, first usable response, and full oracle time separately.
- Global configured budgets and actual use for both engines, including native caches/memtables, OS/ARC, worker counts, and client topology.
- Source/checkout/binary/library hashes, toolchain, host/filesystem configuration, commands, seed, and per-client schedule/input fingerprints.

Screen adaptively with one run per step, then repeat only boundary-adjacent accepted cells three times. Sustained write cells must span multiple checkpoint/consolidation cycles. A run with ever-growing backlog is not steady-state evidence. Rotate backend order and preserve failed attempts; do not rerun accepted cells just to repair reporting.

## Correctness and recovery gates

Do not weaken:

- Synchronous WAL and acknowledged-write durability.
- Conditional and multi-key transaction semantics.
- Revision identity, tombstone revisions, and ABA behavior.
- Per-operation Query/Scan consistency, sort order, cursor semantics, and response-size limits.
- Checksums, corruption detection, page ownership, and page-reuse safety.
- Crash recovery and reopen correctness.

Do not imply that cursor pagination already provides a longer snapshot lifetime than the existing API implements.

Every candidate changing recovery, publication, value placement, reuse, or file layout must include:

incomplete-transaction recovery -> successful write through the real service path -> another crash/recovery -> acknowledged-state validation.

Include torn active WAL tails, invalid deltas/bases, missing/corrupt pages, crashes before/after manifest publication, partial pack construction, interrupted reclamation, page-ID reuse, pinned-old-generation reads, and subsequent write/recovery. A manifest-only or unit-only test is insufficient.

## Recommended order and decision gates

1. Prepare the issue, plan, and continuation worktree. This preparation was completed before A1 implementation was authorized.
2. When implementation is requested, first add narrowly scoped A1 allocation/reuse/pin accounting and a fixed-key overflow diagnostic. Validate the source-confirmed self-pin condition before broad optimization.
3. Add resource accounting and a fair shared-cache/WBM minimal-native reference.
4. Establish E2/E3 active-density boundaries and E4 maintenance stability.
5. Decide whether to invest in B1/B2 and disk reclamation. Keep new behavior opt-in/default-disabled until correctness and comparative evidence justify promotion.
6. Attribute a candidate's effects with matching source, workload, durability, budgets, and topology. Do not use the old profile as its causal proof.

Local optimizations can address waste, peaks, cold-open interference, and unnecessary growth when the aggregate data is resident. Larger-than-memory continuous activity requires B1/B2. Disk-return requirements need B3 and potentially B4.

If shared-resource minimal-native meets the service objective with no meaningful disadvantage in active tenants, memory, or disk, the evidence does not support adopting B-link for this goal. Do not continue collecting small throughput improvements as a substitute for this decision.

## Continuation checklist

- [x] Recheck the frozen source branch/commit and preserve previous worktrees.
- [x] Create an isolated branch and worktree under the fn0 checkout.
- [x] Preserve this analysis and plan in English.
- [x] A1 opt-in implementation, test-scoped allocation/reuse/pin diagnostics, and real-service recovery checks; see [A1-RESULTS.md](A1-RESULTS.md). No density or production resource-accounting completion is claimed.
- [x] Process/disk accounting and executable density harness preparation, with four Linux correctness smokes; see [DENSITY-HARNESS-RESULTS.md](DENSITY-HARNESS-RESULTS.md). Retained-heap/pin-age instrumentation remains incomplete.
- [x] Opt-in shared-cache/WBM minimal-native reference with explicit soft limits and native accounting; see [SHARED-ROCKSDB-RESULTS.md](SHARED-ROCKSDB-RESULTS.md).
- [x] Common opt-in Linux cgroup v2 backend budgets, periodic pressure observations, explicit unsupported metrics, and disk preparation; see [MEMORY-PRESSURE-RESULTS.md](MEMORY-PRESSURE-RESULTS.md). This excludes uncharged ARC/host memory and does not supply application byte admission or eviction.
- [ ] Live host sizing and an explicit frozen experiment matrix.
- [ ] Density/maintenance experiments and evidence reduction.
- [ ] Decision on B1/B2/B3/B4 implementation and adoption.

Unchecked items are future work. No implementation or experiment completion is claimed by creating the issue and documents.
