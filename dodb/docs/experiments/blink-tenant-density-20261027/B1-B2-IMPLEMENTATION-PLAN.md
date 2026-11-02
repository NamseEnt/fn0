# B1/B2 implementation plan for sequential GPT-6 Luna execution

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: planning only. No implementation, experiment, migration, default promotion, merge, or deployment is performed or authorized by creating this document. This document supersedes stale next-action paragraphs in the historical handoff for B1/B2 planning; it does not supersede recorded evidence.

Read [B1-B2-LUNA-RUNBOOK.md](B1-B2-LUNA-RUNBOOK.md) for the execution prompt, per-task receipt, commands, and restart procedure. The intended implementer is GPT-6 Luna, with high reasoning effort. Execute one task at a time. No parallel implementation agents or autonomous architecture substitutions are needed.

## 1. Objective and completion levels

Implement an experimental B-link service whose data and version metadata can exceed its configured in-process working memory while preserving DODB semantics. All continuously active tenants share bounded application memory and maintenance capacity. A checkpoint or long reader must not silently remove the bound. The final comparison remains against the shared-resource DODB-contract TransactionDB reference, not raw RocksDB WriteBatch.

Use these separate acceptance levels:

| Level | Required evidence | What it permits claiming |
| --- | --- | --- |
| P: plan | This document and the runbook | An executable plan exists |
| F: foundations | T00-T05 receipts | Locators, bounded metadata, and source lifetime foundations exist |
| S: storage | T06-T11 receipts | Demand-loaded storage, bounded reconstruction/checkpoint/opening, corruption and crash correctness |
| V: service | T12-T15 receipts | Real service uses the route with global/per-tenant budgets, bounded admission, fair maintenance, recovery, and shutdown |
| E: experiment ready | T16 receipt and frozen matrix | Candidate can be evaluated under matched conditions |
| C: comparative decision | T17 evidence and decision | Observed resource/SLO trade-offs under the named tested policy |

B1/B2 engine/service correctness requires S and V, including every mandatory implementation test below. The complete implementation assignment through T16 also requires instrumentation and the experiment-ready handoff. Foundation completion is not implementation completion. Comparative execution and adoption are separate decisions. A candidate can be correctly implemented and fail the product objective; report that result. Keep the feature disabled by default and issue #126 open until its remaining issue-wide gates are actually resolved.

B3 leaf merging/relocation, free-tail truncation, general epoch-safe reuse, B4 value placement, and compressed-RocksDB deployment evaluation remain separate work. B1/B2 must implement safe source GC and measure retention/release. They must not claim complete physical disk efficiency or implement those separate projects to hide a failed B1/B2 result.

## 2. Verified starting point

On 2026-11-02, local and remote continuation HEAD were `7e37551314b3b2ae14c019e9141c546fa729b914`. The worktree was clean. Implementation source for the completed checkpoint-locator primitive is `aca32bbab2dd3136eebd9cacf20c42cac65194ce`.

Workspace: `/Users/namse/fn0/.worktrees/dodb-tenant-density-20261027`, branch `experiment/blink-tenant-density-20261027`. Leave `/Users/namse/fn0` main, other worktrees, and `/Users/namse/dodb` unchanged. Preserve linear history and all previous failed databases and reports.

Read these existing artifacts, without repeating their completed experiments:

- [PLAN.md](PLAN.md), [HANDOFF.md](HANDOFF.md), [DENSITY-DECISION.md](DENSITY-DECISION.md).
- [B1-CHECKPOINT-LOCATOR-PLAN.md](B1-CHECKPOINT-LOCATOR-PLAN.md) and [results](B1-CHECKPOINT-LOCATOR-RESULTS.md).
- [A1-RESULTS.md](A1-RESULTS.md), especially the real-service crash/write/crash gate.
- [MEMORY-PRESSURE-RESULTS.md](MEMORY-PRESSURE-RESULTS.md), [E2-RESIDENT-FRONTIER-RESULTS.md](E2-RESIDENT-FRONTIER-RESULTS.md), and [E4-COMMON-POPULATION-RESULTS.md](E4-COMMON-POPULATION-RESULTS.md).

Source anchors are symbols, not permanently stable line numbers:

| Path under repository root | Current structure and required work |
| --- | --- |
| `dodb/crates/dodb-storage/src/blink/page_locator.rs` | `ValidatedCheckpointPack`, `CheckpointPageLocator`; reuse the completed validator and exact source binding |
| `dodb/crates/dodb-storage/src/blink/durable_bases.rs` | `scan_pack`, `read_pack`, `BasePackJob`; preserve full validation and replace whole-image checkpoint materialization on the new route |
| `dodb/crates/dodb-storage/src/wal.rs` | `scan_wal`, `ScanMaterialization`, `PendingBatch`, `plan_redo`, `finish_group_append`; share parsing, capture committed offsets, bound reconstruction and recovery memory |
| `dodb/crates/dodb-storage/src/blink/segmented_wal.rs` | `SegmentedWal::open`, `rotate`, `advance_checkpoint_with_base`, `cleanup_obsolete_segments`, `base_owners`, manifest encoding; replace latest-only reclamation and resident owner maps on the new route |
| `dodb/crates/dodb-storage/src/blink/mod.rs` | `BlinkState`, `WorkingBlinkState`, `PageCell`, `PageCatalog`, `PublishedGeneration`, `GenerationPin`, `ReadPageSource`, `BlinkMutationState`, `load_base_images`, `load_file_with_validation`, `check_state`; remove hidden all-page residency and non-fallible miss assumptions |
| `dodb/crates/dodb-storage/src/blink/physical_checkpoint.rs` | `prepare_segmented_checkpoint`, `BlinkCheckpointSnapshot::run`, completion/capacity logic; reserve before snapshot construction, stream output |
| `dodb/crates/dodb-server/src/lib.rs` | `BlinkTenantService`, `AsyncBlinkShard`, `QueuedBlinkTransaction`, open/read/request paths; share one service resource manager |
| `dodb/crates/dodb-server/src/physical_checkpoint.rs` | `PhysicalCoordinator`, `CHECKPOINT_CAPACITY`; retain one running checkpoint, add byte reservations and a fair ready queue |
| `dodb/crates/dodb-rocksdb/src/bin/tenant_density_workload.rs` and its existing supporting modules | Extend explicit experimental configuration and evidence; locate supporting modules before editing |

Facts to preserve: current `BlinkState.pages` and catalogs retain the entire tree; `page_chain` has LSN/CRC but no reload offsets; `base_owners` is a resident latest-owner map; a complete checkpoint snapshot retains clean decoded pages and later materializes images. Merely putting an LRU beside those objects will not achieve B1.

The existing checkpoint locator keeps a file open after unlink. That is useful process-local ownership, not restart-safe retention. Current manifest advancement and startup cleanup can unlink sources needed by an evicted old version. Do not wire eviction into those paths before T05.

## 3. Architecture decisions fixed for the implementer

These are target contracts, not descriptions of existing code. Record exact Rust APIs and byte layouts in T00 before implementation. If a contract is impossible, preserve the failing case and propose a narrowly stated amendment; never quietly select a resident or weaker alternative.

### 3.1 Opt-in boundary and compatibility

- Add default-disabled storage feature `blink-demand-paging`, depending on `blink-checkpoint-locators`. B2 service configuration is also explicit and absent by default. Feature compilation alone must not enable paging for existing stores.
- The first enabled route supports segmented WAL with durable-base packs. Reject paging with legacy or non-base segmented WAL before opening or modifying files. Continue testing their existing resident routes for regressions.
- Use a versioned experimental paged-store manifest and immutable sidecar index files. Do not reinterpret old manifest bytes or silently enable a new format. Existing databases and recorded benchmark paths are never migrated in place.
- Fresh experimental directories only for the first implementation. Existing resident databases can remain readable by their existing route; migration is not required for B1/B2. Opening an experimental directory through an incompatible route must fail before any truncation/cleanup/write. Include an explicit format marker recognizable by the existing open entry point; do not assume unknown sidecars alone prevent an older reader from writing.
- Preserve all existing service semantics and default settings: synchronous WAL, conditional/multi-key atomicity, revision/tombstone identity, consistent ordered Query/Scan, cursor semantics, response limits, and existing page-reuse conservatism. A1 remains a separate opt-in feature.

### 3.2 Exact version and source identity

Define `PageVersionKey` as store identity including WAL epoch, PageId, and an unambiguous version token. Bind it to the exact image LSN and checksum. Distinguish commit LSN, WAL record LSN, page image LSN, and publication epoch in types or field names; they are not interchangeable.

The version token must distinguish page-ID reuse even if page contents coincide. Use the originating committed page record's source number and record LSN as its token; checkpoint copies preserve that token rather than assigning a new logical version. Initial pages created before the first page record use a separately tagged bootstrap token bound to store identity and PageId. Once a page changes it receives a WAL-origin token. T00 specifies the encoded tag/layout and recovery rule, including repeated page records within a transaction, so recovery derives the same token without adding a new counter to the WAL format. Never reuse a committed source number/record LSN within a store epoch. Do not add an incarnation field to existing page bytes incidentally. Existing page checksums/LSNs plus the versioned index bind the content. Retain zero-reader reuse until a separate B3 design is proved.

Define a source identity with store identity, file kind, monotonically allocated file number, and validated file header identity. A source-bound locator contains offsets, frame/image checksums, expected page/version, and committed-prefix bound. Never reopen a bare pathname and infer it still names the same source.

Use a full checkpoint image or a full WAL image plus at most eight deltas. Eight is an initial correctness cap, not a tuned production recommendation. On the next update, emit a full image before extending beyond the cap. A delta locator names its exact predecessor, base LSN/CRC, and result LSN/CRC. Detect loops, missing predecessors, cross-store references, out-of-range offsets, arithmetic overflow, wrong page identity, and result mismatches. Reload is iterative and bounded; no recursive unbounded stack or whole-history scan.

Only verified committed records may be indexed on recovery. During writing, offsets become visible to published readers only after required WAL sync succeeds. A complete commit marker observed in an arbitrary live file is not proof that another process has synced it. Live indexing takes its durable fence from the writer; offline indexing operates after exclusive recovery has validated the file. No read-only locator helper truncates or syncs a source.

### 3.3 Disk-backed version metadata

Use a persistent copy-on-write radix index keyed by the full 64-bit PageId, with immutable checksummed fixed-size nodes and a fixed maximum depth. A 16-way radix with at most sixteen key levels is the starting design. Root handles are small. Path copying must not clone a directory proportional to high-water. Leaves contain version descriptors, not decoded payloads. A generation pins an index root and tree root; unchanged subtrees are shared.

Index nodes, source descriptors, dependency descriptors, free-list metadata, and GC traversal state must themselves be loaded through bounded caches or bounded external work files. Do not replace `PageCatalog` with another permanent per-page hash map or retain every historical locator in RAM. Full validation may stream the whole store; its memory cannot scale with the dataset. Source-file handles need a bounded pool, with identity verified on reopen and source lifetime protected independently of an open descriptor.

The index is a derived representation of committed page state, not a substitute WAL. Its durable checkpoint root is published with the paged manifest. Commits newer than that checkpoint remain recoverable from the synced WAL; their in-process index nodes may spill to checksummed immutable temporary files and be rebuilt after a crash. Do not fsync every derived index node as a new acknowledged-write requirement without explicitly documenting the cost. Never advance a durable checkpoint past index nodes or images that are not durable. A checkpoint root can share nodes already durable under earlier roots, but every node reachable from its new root must be durable. It cannot reference merely rebuildable spill files. Store active WAL state as a bounded committed prefix plus a small append tail; keep historical page-chain/base-owner metadata in the disk-backed index rather than retaining the current full maps indefinitely.

Unpublished/spilled nodes need disk capacity reservations and unique source identities. Readers of a live generation can rely on those files while the process is alive; recovery relies only on published durable roots plus committed WAL. The manifest identifies the recovery WAL start, so no unbounded list of segments needs to be retained in process memory.

### 3.4 Source retention and publication

Protect the union of durable recovery roots, current published generation, active operation pins, checkpoint snapshots, and in-flight reconstruction. Runtime pins do not survive process death and must not be resurrected as client snapshots. Persist the durable recovery roots and referenced source information; after restart re-establish the protected set before GC.

Use one source-retirement protocol for checkpoint completion, startup cleanup, consolidation, failed writes, and shutdown. Publication order is: write outputs -> sync required source/index files -> publish and sync manifest/directory according to the existing durability contract -> install the new recoverable root -> retire superseded roots -> reclaim only unreachable files -> sync reclamation directory. Older recoverable roots remain protected for as long as recovery can select them. Do not invent fallback to an older checkpoint if it would discard acknowledged WAL state.

A source can be deleted only when no protected root, delta dependency, or active load can reach it. Test the race between acquiring a generation/load lease and retiring a source. A missing source that was required is corruption, never a cache miss treated as absence. An orphan is deletable only after identity and non-reachability validation. GC marks/work queues must be spillable and budgeted; holding all roots' transitive file sets in a permanent map is not acceptable.

### 3.5 Page access and memory ownership

- Introduce owned page guards that hold the exact decoded version and its byte charge. Cache eviction drops cache ownership only. A reader/transaction may retain a guard, and the charge stays live until the last owner drops it.
- Page access on the paged route is fallible, including writer base-page loads, free-list traversal, overflow chains, split planning, and invariant validation. Do not keep `Option<&BlinkPage>` assumptions by eagerly reloading the whole tree. Propagate I/O/corruption/overload errors without converting them into missing documents.
- Keep the resident route while introducing interfaces, then route the explicit paged configuration through them. `blink-borrowed-page-views` cannot return references into an evictable cache. Either implement a guard-backed compatible view or reject that feature/runtime combination explicitly before data access, with a compile/configuration test.
- Coalesce loads by full version key. One leader reserves memory and performs I/O; bounded followers share the result. Cancellation, load failure, and shutdown must release reservations and wake all followers. A follower must not hold a scarce loader permit while waiting for its leader.
- Bound blocking I/O/decode workers explicitly. Run loads outside generation/registry/cache/scheduler locks; do not block a Tokio executor thread or hold the global tenant registry across open/recovery I/O. Synchronization order is specified in T00 and tested with barriers.

### 3.6 B2 budget contract

One resource manager is shared by all tenants in one `BlinkTenantService`. The server's usual single-service process topology is the initial boundary; multiple independent services in one process are not automatically covered. Storage tests explicitly pass the same manager where global behavior is claimed. Never create one full global budget per shard.

Use reservation guards with checked arithmetic and explicit transfer of ownership. Every category has global and per-tenant current/peak usage. Transfer a charge when buffers move or are shared; do not charge the same Arc payload twice, and do not release its charge while any owner remains. Separate:

| Memory category | Included ownership |
| --- | --- |
| Decoded payload | Cache ownership and guards, including pinned old pages; packed leaf/overflow capacities |
| Metadata | Index node/source/dependency caches, generation handles, bounded load/waiter tables, tenant state |
| Load/reconstruction | Encoded input, delta frame buffers, decoded output overlap, worker jobs |
| Queued foreground work | Owned transaction keys/values/conditions, encoded copies and channel entries |
| Write preparation | Working pages, logical overlays, planning structures, transaction-boundary images, WAL encoding |
| Dirty images | Latest committed encoded images until safely replaced by reconstructable descriptors |
| Checkpoint/validation | Snapshot handles, streaming buffers, output assembly, invariant-check scratch |

Use separate counters and capacity limits for on-disk WAL, pack/index output overlap, temporary spill/scratch, and reader/recovery-retained source debt. Disk bytes are not part of the memory sum. Unlinked-open bytes still count as retained disk, not reclaimed storage.

Application accounting is not an exact RSS cap: allocator retention, runtime stacks, response ownership outside the service, and host caches have separate boundaries and observations. Charge known capacities and conservative metadata bounds; use cgroups for external process enforcement and report unexplained residuals. A test of counters alone does not establish bounded process memory. Explicitly bound tenant count, worker count, request size, response size, handles, and pending waiters in addition to bytes.

Reserve before allocation/copy/enqueue, using a checked conservative upper bound from bounded request structure. At the service API, caller-owned request/response memory outside the service is not controllable; document when ownership is transferred and whether the returned response leaves accounting. Do not claim control of upstream networking allocations without integrating that layer.

Foreground requests that cannot fit a finite reservation fail with existing overload semantics before commit; a bounded wait is permitted only with cancellation/deadline behavior covered by tests. A single request exceeding a hard limit is rejected, never given a bypass. After WAL sync, lack of publication bookkeeping memory must not turn a committed transaction into an untracked rollback; reserve the entire accepted transaction envelope before starting commit. WAL group collection has both byte and count caps. Recovery stages a large pending committed sequence in bounded spill storage if necessary; it cannot assume all page records up to a commit fit in RAM. Validate rolling digest and sequence before installing any of those staged versions.

Reserve a non-borrowable progress pool for one checkpoint/recovery/GC unit and its required loader. The same bytes are subtracted from the global configured budget. Foreground work cannot consume them. Reject configurations where this minimum cannot fit. A checkpoint must not wait for bytes held by the queued work that only that checkpoint can drain. Bound checkpoint inputs by streaming rather than reserving an entire store.

### 3.7 Fairness and reader pressure

Keep the existing one-running-checkpoint cap. Use one fair tenant-ready queue, one pending ticket per tenant, and bounded work slices. Pressure/debt can make a tenant eligible; round-robin eligible tenants and prohibit an unlimited priority bypass. Record eligibility time, wait time, threshold excess, bytes of debt, and completed service per tenant.

A slice is a bounded batch of image/index output or one bounded publication/GC unit. A large checkpoint keeps a resumable cursor and a charged root/output lease between slices; it must not monopolize the worker until an entire store is encoded. T00 fixes the batch size and continuation/result types. Only one slice may perform checkpoint work at once; queued continuations are not extra workers. With N eligible tenants, a ready tenant receives a slice after at most N-1 other ready-tenant slices in the deterministic scheduler model. This is a bound in service turns, not a wall-clock guarantee against indefinitely stalled filesystem I/O. Measure actual wait and non-preemptible sync latency separately.

Per-tenant floors are service opportunities under contention, not eagerly allocated memory. All active tenants participate. Enforce hard per-tenant ceilings and admission limits so one tenant cannot retain the entire shared pool. No success claim can rely on closing idle tenants while the target workload keeps every tenant active.

An already active read keeps its consistent version. Under long-reader pressure, reject/defer new admissions or checkpoint output when memory/disk retention reaches its configured boundary; do not revoke a snapshot mid-operation or discard its sources. Bound new pin creation and pending requests. API cursor pagination does not create a persistent cross-request snapshot. Test a held internal operation pin explicitly.

## 4. Sequential task cards

All cards require the common receipt and checks in the runbook. New module/test filenames below are planned artifacts, not existing files. If a card needs more than one reviewable commit, split it into numbered substeps in its receipt without dropping acceptance criteria. Interfaces may be introduced before runtime activation; activate the paged service only after its prerequisites pass.

### T00 - Freeze contracts, format, and failure ordering

Dependencies: completed B1.0. Ownership: documentation only.

Create `B1-B2-CONTRACTS.md`: exact version/source key fields, radix node/manifest/spill layouts and limits, compatibility marker, durable versus runtime roots, version-token recovery, sync/publication order, active-WAL committed-prefix ownership, lock order, reservation categories, error taxonomy, cancellation, and GC algorithm. Include a state-transition table for append, sync, publish, checkpoint, recovery, and GC, identifying which sources protect acknowledged state at each transition. List existing source functions to replace and call sites to migrate.

Acceptance: every decision in section 3 maps to a concrete layout/API/invariant and planned test. No unresolved alternative such as "LRU or disk map". No claim of implementation. Freeze the specification in a separate commit before implementing disk structures. The 2026-11-02 amendment closes the missing persisted delta-ancestry path and adds explicit T05, T07, T11, and T16 acceptance.

### T01 - Shared byte ledger and reservation ownership

Dependencies: T00. Ownership: new storage resource module, storage Cargo/lib wiring, unit tests. Do not activate server behavior yet.

Add the `blink-demand-paging` feature dependency and implement validated global/per-tenant limits, reserve/transfer/release guards, progress-pool separation, checked counters, peak/wait/rejection metrics, and bounded request/worker counts. Use existing error variants where suitable.

Acceptance: exact-fit/one-byte-over; overflow/underflow; tenant cap; shared-manager contention; transfer without double-counting; Arc final-owner release; cancellation/panic/error cleanup; progress pool cannot be consumed by foreground; invalid minimum-capacity configuration. Test recorded peak against all simultaneously live owners, not only ledger calls in isolation.

### T02 - Committed WAL locations and exact reconstruction

Dependencies: T00-T01. Ownership: `wal.rs`, new locator module as appropriate, WAL tests. Leave service routes resident.

Extend the shared scanner with staged location metadata published only on matching commit/digest verification. Preserve all existing parsing/corruption/torn-tail checks. Add exact image/delta offsets and committed-prefix fence. Offline scans do not mutate files. Share decoding rather than writing a permissive second parser. Capture append locations in the writer only after successful sync. Split this into frame-location capture, commit-gated visibility, exact reconstruction, and append-fence tests. A temporary O(page count) location collector is permitted only in this isolated foundation; T03-T04 replace it with a streaming disk-index consumer. Existing resident recovery can retain its existing consumer. Do not claim bounded opening from this card or duplicate the current recovery page map in the eventual paged consumer.

Acceptance: compare each reconstructed version byte-for-byte with existing recovery for image/delta histories; incomplete commit and digest mismatch; active torn tail versus closed-segment tail; wrong identity/epoch; wrong base LSN/CRC/result; corrupted unrequested committed frame; changed/truncated source; pathname replacement; concurrent loads; same PageId with different versions; cap eight and forced image at the next step. A valid committed prefix may remain usable before an incomplete active tail, but no uncommitted locator is exposed. Prove scan output retains metadata, not all after-images; any temporary resident compatibility consumer is excluded from the new route.

### T03 - Persistent bounded version index

Dependencies: T00-T02. Ownership: new `blink/version_index.rs`, format tests, storage wiring.

Implement immutable radix nodes, checksums/identity, COW update, generation root lookup, budgeted node caching, spills, and bounded source/dependency access. Keep the old `PageCatalog` for the resident route. Index persistence follows the frozen checkpoint protocol.

Acceptance: sparse and very large PageIds without allocating a high-water-sized vector; old/new roots read distinct versions; reuse token separation; torn/corrupt/wrong-store nodes; max-depth and overflow checks; randomized update/lookup equivalence to a small reference map; bounded-memory streaming build with a much larger on-disk index. Track every cache/handle/work table. Test that pinning many roots cannot bypass metadata/admission limits.

### T04 - Durable paged manifest and bounded recovery index build

Dependencies: T02-T03. Ownership: experimental manifest/recovery modules, `segmented_wal.rs`, format/open tests.

Persist the checkpoint index root, source identity/catalog roots, WAL recovery boundary, and tree/free-list/high-water metadata. Publish atomically with checksummed versioned data and required syncs. Rebuild post-checkpoint committed state using bounded streaming/spill, not `recovery_pages`/`recovery_batches` for the complete history. Validate existing compatible pack/WAL bytes using shared parsers.

Acceptance: fresh-store creation and normal reopen; unknown/truncated/bad-checksum metadata; wrong identity; valid checkpoint plus newer acknowledged WAL; failed manifest construction/publication; incompatible/default route refuses without file mutations. Snapshot file lengths/hashes before and after a rejected open. Record durable index versus rebuildable spill distinctions explicitly.

### T05 - Root reachability, source leases, and safe GC

Dependencies: T03-T04. Ownership: source manager/GC modules, segmented reclamation integration, fault tests.

Implement runtime root/load/snapshot leases, durable-root discovery, bounded mark/sweep or equivalent external traversal from the frozen contract, handle pooling, and safe file retirement. Replace every deletion path on the paged route, including startup cleanup. Keep old resident reclamation unchanged.

Acceptance: pin an old generation, checkpoint/consolidate, evict/drop original payloads, close/reopen source handles, and read exact old bytes; release the pin and prove sources become reclaimable only when all other roots/dependencies drop. Race load acquisition with retirement using barriers. Crash before/after manifest publish and before/during/after unlink; recover acknowledged state. Missing reachable source rejects open/read. Foreign files, unreferenced delta-chain sidecars, and uncertain orphans are preserved. Mark all source dependencies in each referenced delta-chain sidecar. Show allocated/unlinked-open bytes separately; an open descriptor after unlink alone cannot pass this card.

### T06 - Fallible page access and writer interfaces

Dependencies: T01, T03, T05. Ownership: `blink/mod.rs`, access/guard helpers, resident compatibility tests.

Introduce owned page guards and fallible source APIs. Migrate read traversal and mutation base access, including free/overflow/internal pages and split/reuse paths. Keep immutable mutation working copies distinct from shared decoded versions. List and remove paged-route dependencies on borrowed map references. Preserve resident behavior through its adapter.

Acceptance: Get/Query/Scan and mutation regressions pass on resident adapter; injected load failures propagate as I/O/corruption/overload, not NotExists/conflict; guard lifetime survives cache removal; no lock is held across blocking load. Resolve borrowed-view feature compatibility explicitly. Do not activate eviction yet.

### T07 - Bounded decoded cache and coalesced loads

Dependencies: T01-T06 and the T00 bounded-delta-ancestry amendment. Ownership: cache/load modules, bounded delta-chain sidecars, and guard wiring.

Implement one shared budgeted cache keyed by exact version, bounded loader jobs/followers, direct source reads and bounded WAL-delta reconstruction from the immutable ancestry sidecar, byte charges for encoded/decoded overlap, and eviction of cache ownership. Sync the ancestry sidecar before publishing an index root that references it. An old generation retains a small index root rather than all decoded payloads.

Acceptance: repeated misses trigger one physical load; distinct-version misses never coalesce; all followers see failure/cancellation/shutdown terminal state; no permit leak; cache removal does not release live guard bytes; all-pinned capacity produces bounded overload/backpressure, not allocation bypass. Force both current and old versions out of cache and load exact bytes from still-live sources. Reconstruct one-, two-, and eight-delta chains across WAL rotation after source handles close/reopen without scanning WAL; validate every intermediate and final LSN/CRC. Reject missing/corrupt chain sidecars as corruption. Test capacity below one legal load is rejected at configuration.

### T08 - Demand-loaded read route

Dependencies: T06-T07. Ownership: storage read handle/generation path and paging read integration tests.

Replace paged-route `PageCell` payload/catalog ownership with the version-index root and loader. Keep logical overlays bounded and attached to the correct committed generation. Route Get, Query, Scan, overflow reads, cursor traversal, and response budgets through guards. No complete `BlinkState.pages` or duplicate owner directory may remain on this route.

Acceptance: deterministic and randomized reference equivalence including sorted 16-row Query, multi-page Scan, tombstones, empty/missing keys, cursor boundaries, and response-size rejection; split/overwrite races with pinned reads; source corruption on a miss; dataset at least four times the configured decoded cache. Verify nonzero storage-source reads and bounded retained decoded/metadata charges. This is cache correctness, not physical-device-miss or performance evidence.

### T09 - Demand-loaded atomic writer and bounded WAL chains

Dependencies: T02, T06-T08. Ownership: planned mutation/commit/publication paths, writer integration tests.

Load only required base pages into a bounded transaction working set. Reserve planning, copies, WAL buffers, dirty descriptors, and publication work before commit. Publish the exact committed index root after WAL sync; do not expose staged locators earlier. Drop committed encoded payloads when a durable locator can replace them. Preserve transaction boundaries under grouping and existing page-reuse rules.

Acceptance: conditional/multi-key overlap, expected conflict, tombstone/ABA, planned widths 1/4/16, split/overflow/free reuse; writer loads a previously evicted page and preserves an old reader's version. Test the ninth would-be delta emits an image. Inject append/sync/publication failures; acknowledged transactions survive, unacknowledged transactions may be complete or absent but never partial. Reject oversized working sets before commit without leaked reservations or changed logical state.

### T10 - Streaming checkpoints and bounded debt

Dependencies: T05, T09. Ownership: both physical-checkpoint modules, durable-base writer, checkpoint tests.

Stream exact target-generation images using index iteration and a bounded buffer; do not construct whole-store image maps or capture every clean decoded page. Reserve snapshot/output overlap and source/index disk capacity before work. Allow newer writes while the target root is pinned and publish only that target. Reuse one checkpoint worker. Update dirty/source ownership with reservation transfers.

Acceptance: incremental and complete/consolidating checkpoints, coalesced and uncoalesced output, forced cache misses during output, writes after target capture, checkpoint failure, old-reader source retention, and complete reopen oracle. Memory peaks stay within configured categories when checkpointing a store much larger than cache. Tiny disk/memory capacity yields bounded defer/overload with eventual progress after release, not a deadlock or partial manifest. Record queued debt and time above threshold.

### T11 - Streaming full validation and storage crash matrix

Dependencies: T04-T10. Ownership: bounded invariant checker/recovery integration and storage SIGKILL tests.

Replace paged-route whole-page/high-water sets in `check_state` and opening validation with bounded traversal and external sort/visited scratch. Preserve detection of duplicate ownership, cycles, missing/unreachable allocated pages, invalid free lists, cross-page links/order, overflow lengths, checksums, and malformed page formats. Scratch consumes explicit memory/disk capacity and is distinguishable from durable sources.

Acceptance: corruption in an unrequested page still fails complete validation; do not substitute lazy validation. Reopen a store above cache and replay a large WAL without an all-page temporary map. Test torn active tails, corrupt closed tails, invalid delta/base, missing/truncated/wrong-identity delta-chain sidecars, sidecar-before-index and index-before-manifest crash boundaries, pack construction failure, every publication/GC crash boundary, page-ID reuse, pinned-version/checkpoint race, and write after recovery followed by another crash/recovery. Storage tests complement but cannot replace T15's service test.

### T12 - Explicit service configuration and shared resources

Dependencies: T01-T11. Ownership: `dodb-server` configuration, tenant open/read lifecycle, paging service tests.

Pass one manager and validated paging configuration to all shards. Use bounded blocking workers for open/recovery/read loads. Do not hold tenant registry locks across disk work; concurrent same-tenant opens must coalesce with bounded waiters and one authoritative shard. Register tenant state and charges before allocation, release on failure/shutdown. Keep default routes unchanged.

Acceptance: invalid feature/layout/limit combinations fail early; same-tenant open race creates one shard; slow/corrupt tenant open does not monopolize unrelated open tenants; failed initialization releases every charge/job; shutdown drains guards/tasks without indefinite wait. Compare fixed registered/active counts and explicitly account for per-open-tenant metadata.

### T13 - Byte-bounded foreground admission

Dependencies: T09, T12. Ownership: `QueuedBlinkTransaction`, enqueue/group/apply paths, shared budget integration.

Charge owned queued request bytes before channel insertion and encoded/preparation copies before allocation. Attach guards to the queued request and transfer them into execution; cancellation of the response receiver does not prematurely free a still-running commit's charge. Enforce global/per-tenant byte/count limits and bounded waiting policy from T00. Include reads, outputs while service-owned, and pin/load waiters in the documented boundary.

Acceptance: few large requests hit bytes before count; many tiny requests hit count; oversized single request; queue-full/closed; caller cancellation before/after enqueue and during commit; mixed tenants; failed sync; shutdown drain. Live ownership and ledger agree at each barrier. No accepted transaction needs unreserved publication allocation after sync. Show bounded residual response/transport ownership rather than claiming all caller memory is capped.

### T14 - Fair debt-driven checkpoint admission

Dependencies: T10, T12-T13. Ownership: server physical coordinator and shared maintenance scheduler.

Add one ready ticket per tenant, global/per-tenant debt and disk-retention ceilings, fair service opportunities, bounded resumable work slices, and the dedicated progress pool. Preserve one running worker. Transfer charged cursor/output state back to the scheduler when a slice completes, release the worker permit, and resume the correct target snapshot on its next turn. Apply foreground backpressure before hard memory/disk exhaustion, and wake eligible work when capacity/pins release. Keep tenant serialization and WAL sync.

Acceptance: synchronized versus staggered pressure with all tenants active; hot tenant cannot starve a small active tenant; measured maximum service wait is bounded by the configured queue and work-slice contract in a deterministic test; an idle/failed/cancelled tenant cannot leak a ready ticket. Test full foreground pool plus needed checkpoint, long pinned reader, failed worker, shutdown, and capacity release. Do not resolve pressure by increasing concurrency or silently disabling sync/checkpoints.

### T15 - Real-service end-to-end and crash/write/crash acceptance

Dependencies: T11-T14. Ownership: new server paging/resource integration tests, existing compatibility tests.

Exercise actual `BlinkTenantService` requests with paging enabled, not only storage methods or manufactured manifests. Adapt the subprocess/barrier pattern in `dodb-server/tests/blink_writer_pin_reuse.rs`, preserving its existing legacy/bases coverage.

Mandatory new sequence: seed acknowledged state -> force eviction and a WAL-backed version -> pause a transaction before commit and SIGKILL -> reopen and validate complete acknowledged state -> successfully acknowledge a conditional/multi-key service write -> force checkpoint/eviction -> start another incomplete transaction and SIGKILL -> reopen and validate the new acknowledged values/revisions and absence of partial transactions. Save child markers, PIDs, source/configuration, logs, and failed databases. Assert the intended barriers and actual enabled route were reached.

Also test old operation pins across overwrite/checkpoint/reload, concurrent Query consistency, page-ID reuse, missing/corrupt sources, memory/disk overload and recovery, cancellation, and complete shutdown. Repeat with both durable-base output modes and with A1 off/on. Legacy/non-base layouts remain resident compatibility tests and reject paging requests.

Acceptance: all required correctness cases pass with nonzero eviction/miss/WAL reconstruction counters, bounded live category peaks, no leaked tasks/charges, and full final/reopen oracle. A test that quietly uses resident reads cannot pass.

### T16 - Instrumentation and experiment readiness

Dependencies: T15. Ownership: density harness supporting modules, reviewer, new result schema/tests and frozen plan.

Add explicit feature/runtime options to the existing isolated backend worker. Record actual applied limits, every category/current/peak, pin count/age, cache/load/coalescing counts, source opens, reconstructed chain depths, version-index v2 and delta-chain sidecar format identities, queue/admission waits/rejections, maintenance wait/debt, reachable/retained/temporary disk bytes, phase process/cgroup observations, and sampling gaps. Final report reads must not allocate an entire historical trace in the backend.

Acceptance: matched fingerprints and full/reopen oracle on tiny paired correctness cells; actual paged route verified; bounded diagnostics; failed worker yields preserved partial evidence; default reports retain compatible semantics. Use the existing reference options and shared budget boundaries. Implementation tests can be local; OCI timing/density conclusions cannot be derived from local tests.

Freeze a separate experimental matrix with numeric values, recipe hashes, setup expansion/overlap sizing, exact source/native/binary identity, host-cache controls, SLO, offered rates, run lengths, repetition/stop rules, evidence reviewer, and available capacity before launching performance work. If experimental execution has not been requested, stop at a reviewable ready state rather than launching a costly matrix from this planning request.

### T17 - Comparative validation and adoption decision

Dependencies: T16 and separately authorized frozen experiment execution. Ownership: controller/reviewer/results documents only unless a defect is found.

Compare the candidate, the recorded resident control where relevant, and shared-resource contract RocksDB under the same durability/API/input/schedule/client/admission/cgroup policy. Registered=active for the principal track. Include one tenant above cache, many active tenants above the backend budget, synchronized/staggered maintenance, fixed-key churn, growing distinct-key/tombstone history, and long-reader retention. Keep budget-fitting setup expansion and preserved failed cells in sizing.

Dataset greater than decoded cache is insufficient to prove physical reads. A storage-read-sensitive condition must document host RAM/ARC/page-cache coverage and actual device/read evidence. If the designated host cannot support that condition while preserving reserve/evidence, mark that gate unverified rather than relabeling a warm-cache run.

Report operational validity, SLO pass, resource acceptance, and comparison separately. Sustain enough checkpoint/consolidation cycles to test backlog/debt slopes; three five-minute passes alone do not prove a plateau. A finite window with an increasing queue/debt is not steady state. Report memory, disk, misses, latency, throughput, rejection, recovery, and temporary-space costs together. Do not tune only one backend's workload or call the comparator maximum-tuned RocksDB.

Conclude adopt, do not adopt, or insufficient evidence for the named condition. A correctness pass does not justify promotion. Actual source-file GC is distinct from B3's complete disk-return objective; measure reachable-file allocation and filesystem free-space changes separately. Do not close issue #126 while its remaining disk/host-cache/adoption gates are unresolved.

## 5. Mandatory acceptance matrix

Each cell needs a named test or collected artifact in the final receipt. "Covered by unit tests" without the exact test and exercised runtime path is insufficient.

| Requirement | Required trigger | Evidence owner |
| --- | --- | --- |
| Exact version | Same PageId in two pinned generations, eviction and reload | T02, T03, T07-T09 |
| Commit/durability boundary | Image/delta frames without valid commit; sync failure | T02, T09, T15 |
| Replay bound | Eight deltas, then one more eligible update | T02, T09 |
| Full corruption validation | Corrupt a page/node/frame not requested by workload | T02-T04, T11 |
| GC safety | Old reader plus checkpoint publication, pooled handle closed | T05, T10, T15 |
| Runtime/durable root split | Crash removes runtime pins, recovery keeps acknowledged state | T04-T05, T11, T15 |
| Metadata bound | Sparse high-water, index much larger than node cache, many roots | T03, T07, T11 |
| Payload bound | Data >= 4x decoded cache; misses and eviction proven | T08-T11, T15 |
| Global ownership | Multiple active tenants and shared Arc/guards | T01, T12-T15 |
| Preparation/checkpoint peak | Full queue plus full/consolidating checkpoint and misses | T09-T10, T13-T15 |
| Bounded recovery/validation | Long WAL and complete tree validation above cache, including delta ancestry sidecars | T04, T07, T11, T15 |
| Fairness/progress | All tenants pressured, one hot writer and one old reader | T14-T15 |
| Request semantics | Conditional/multi-key, ABA/tombstone, cursor/response limits | T08-T09, T15 |
| Real-service durability | Incomplete recovery -> acknowledged write -> second crash/recovery | T15 |
| Opt-in compatibility | Defaults, A1 off/on, borrowed-view policy, rejected unsupported layouts | T06, T12, T15 |
| Physical-read evidence | Dataset exceeds relevant host caches, observed storage reads | T17, separate from implementation |
| Adoption | Shared-reference matched acceptance and explicit decision | T17, separate from implementation |

### Deterministic correctness fixture specification

T00 must freeze exact numeric fixtures in `B1-B2-CONTRACTS.md`, using these relationships rather than inventing production defaults:

- Use two fixed seeds, `12601` and `12602`, and a saved failing seed for every randomized failure. Cover 512, 513, and 4096-byte values, widths 1/4/16, conditions Exists/NotExists/revision equality, and deletes followed by reinsertion. Every successful/rejected result is compared with a reference state including logical missing revisions.
- Use one-tenant fixtures for reconstruction/eviction, four continuously active tenants for global ownership, and eight tenants for fair scheduling. Separate tenant-count bounds from per-tenant data size.
- Build above-cache fixtures with at least four times the configured decoded-payload cache in encoded page images and an index at least four times its configured node-cache allowance. Confirm actual encoded size and nonzero misses rather than assuming user value bytes imply the relationship. Seed incrementally through bounded accepted requests.
- Include limit-minus-one, exact-limit, and limit-plus-one requests for every reservation category, plus zero/overflow configurations. Derive legal minimum progress capacity from the largest admitted fixed-size I/O/decode/GC unit and required simultaneous guards. Reject a fixture whose configured minimum cannot fit; do not silently enlarge it at runtime.
- Pin one old operation while overwriting its pages through at least two complete/consolidating checkpoints. Evict current and old payloads, close pooled source handles, then verify both versions and release-driven reclamation.
- Trigger two incomplete-transaction crashes around an intervening acknowledged conditional/multi-key write. Cover both pack modes and A1 off/on; save a table of exercised child markers and final revisions.
- Repeat deterministic pressure/cancellation/GC races at barrier-defined ownership points. Fake capacity/worker completions are allowed for scheduler unit tests, but service acceptance must also run actual I/O and checkpoints.

Exact byte limits are correctness-fixture inputs, not performance recommendations. Record measured owner peaks and observed over-limit handling alongside the configured numbers. Increasing a fixture's limit after an unaccounted allocation/deadlock is a failed attempt requiring explanation and repair, not evidence that the original invariant passed.

## 6. Stop and repair rules

Stop the current dependent work when any of these occurs; independent documentation/evidence repair may continue:

- Acknowledged state is lost, a multi-key transaction becomes partial, or a generation returns a newer/foreign/reused page. Preserve the database and failing seed; disable runtime activation until corrected and the whole affected gate is rerun.
- A required source is retired early, corruption becomes lazy-only, unknown metadata is treated as empty, or a rejected configuration modifies files.
- Any request/load/checkpoint/validation path allocates outside its planned reservation or retains an O(dataset/high-water/history) structure in the supposedly bounded route. Profile/trace live owners and repair the path; do not increase the limit to hide it.
- No progress is possible because foreground bytes occupy required checkpoint/loader capacity. Preserve waiter/owner state, fix reservation ordering, and test the exact deadlock; increasing timeout is not a fix.
- Source/layout/sync ordering needs an architecture change. Amend T00 with the concrete counterexample and changed acceptance mapping before coding the changed protocol.
- A new Clippy diagnostic appears. Fix it without suppression. Existing strict-Clippy failures must be reproduced and identified separately, not reported as a pass.
- OCI capacity would violate the 10 GiB ZFS free reserve or memory/host safety sizing. Do not delete databases, sources, binaries, reports, or native artifacts to make the cell fit.

For interrupted work, inspect the actual PID/start-time, process/session handle, and completion artifact. A status file or observation timeout alone is not proof of a stopped process. Never duplicate a live run. Preserve failed attempts; repair reporting over preserved evidence rather than replaying accepted cells.

## 7. Final implementation handoff

Deliver task-owned commits pushed to the continuation branch, contract/format documentation, exact commands/log hashes, receipts for T00-T16, a named-test acceptance matrix, failed-attempt inventory, and a clear list of remaining experiment/issue-wide gates. Update the handoff and issue status in English with exact scope and links; do not rewrite historical evidence or imply implementation results were measured by old binaries.

If only some cards are complete, report those cards and resume at the first unmet dependency. Do not redefine the full B1/B2 objective as the subset already implemented.
