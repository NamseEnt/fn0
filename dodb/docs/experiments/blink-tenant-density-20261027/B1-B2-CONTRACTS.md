# B1/B2 Paged Store Contracts

Status: frozen implementation contract. The T03 node-format detail below was frozen on 2026-11-02 before T03 code changes. This document does not describe a measured result.

## Compatibility boundary

The `blink-demand-paging` storage feature depends on `blink-checkpoint-locators` and is disabled by default. The service route is selected only by an explicit configuration. It accepts fresh experimental stores using segmented WAL with durable-base packs. It rejects legacy WAL, segmented WAL without durable bases, unsupported borrowed-page views, and incompatible store formats before opening writable handles or changing any file. Existing resident routes and database formats remain unchanged. A format marker is recognized by the existing open dispatch before legacy cleanup can run.

## Identity and version keys

`StoreIdentity` is the existing store UUID plus WAL epoch. `SourceIdentity` is `(StoreIdentity, FileKind, file_number, header_identity)`. `file_number` is monotonically allocated within an epoch and is never reused. `header_identity` is the validated immutable file-header digest. A `SourceBoundLocator` carries that identity, byte range, frame and image checksums, `PageId`, `PageVersionKey`, and committed-prefix fence; resolution opens through the source manager and verifies identity before reading.

`PageVersionKey` is `(StoreIdentity, PageId, VersionToken)`. `VersionToken` is a tagged union: `Bootstrap(PageId)` for a page created by initial tree construction, or `Wal(SourceIdentity.file_number, record_lsn)` for a committed page record. Checkpoint copies retain their original token. A transaction containing repeated records for one page assigns the token of its final committed page record; its earlier records remain valid reconstruction predecessors only within that transaction's staged chain. Recovery derives the same final token after validating the transaction digest and commit. Page image LSN and checksum are bound separately and are not interchangeable with commit LSN, WAL record LSN, or publication epoch.

The API boundary consists of `VersionIndex`, `PageSource`, `PageLoader`, `PageGuard`, and `Reservation`. `VersionIndex::lookup(root, page_id)` returns `Result<Option<VersionDescriptor>, StorageError>`; `insert(root, page_id, descriptor, reservation)` returns a new immutable root. `PageSource::acquire(identity)` returns an identity-checked `SourceLease`; `read(lease, locator, output)` performs a bounded positioned read. `PageLoader::load(version_key)` returns an owned `PageGuard` and coalesces concurrent loads. The guard exposes immutable page access and retains the decoded-payload charge until its last owner drops. Implementations may refine Rust visibility and concrete error variants but may not weaken these ownership and failure contracts.

## Persistent formats

All integers are little-endian. Every file starts with a fixed magic, format version, store UUID, WAL epoch, file number, file-kind tag, payload length, and BLAKE3 checksum over the header fields preceding the checksum. Unknown versions, overflowing lengths, identity mismatch, truncation, and checksum mismatch are errors.

The version index is an immutable copy-on-write 16-way radix tree over all sixteen nibbles of the big-endian `PageId` bit pattern. A node is fixed at 4 KiB including header, level, occupied-child bitmap, generation, store identity, payload, and checksum. Internal children are either fixed-width child references or empty. A leaf descriptor is a fixed-width tagged record containing bootstrap/WAL token, image LSN, image checksum, and either a source-bound image locator or a source-bound delta locator. Reserved bytes must be zero. Nodes are written to uniquely numbered immutable sidecar files; a node reference is `(file_number, offset, checksum)`. The maximum traversal is sixteen nodes. Overflow, duplicate child slots, wrong level, wrong generation ancestry, and a mismatched key are corruption.

### T03 node format freeze, 2026-11-02

Each node is one immutable 4,096-byte file. The 112-byte header is: bytes 0-7 magic `DDBVIDX1`; 8-9 format version 1; 10-11 file kind (`1` durable index, `2` rebuildable spill); 12 level (`0` leaf, `1..15` internal); 13-15 zero; 16-23 generation; 24-39 store UUID; 40-47 WAL epoch; 48-55 file number; 56-57 occupied-child bitmap; 58-63 zero; 64-67 payload length 3,952; 68-71 zero; and 72-103 BLAKE3 of bytes 0-71. Bytes 104-111 are zero and bytes 112-4,063 are the payload. Bytes 4,064-4,095 are BLAKE3 of bytes 0-4,063 and are the node-reference checksum. The exact node length is mandatory. Header identity is `(store UUID, WAL epoch, file kind, file number)`; a different header digest or file identity is corruption.

Internal nodes have sixteen 48-byte child references in nibble-slot order. Each reference is file number u64, offset u64, and 32-byte node checksum. Empty slots are all zero and absent from the bitmap. Current references use offset zero. Leaf nodes have sixteen 240-byte descriptors followed by 112 zero payload bytes. Each descriptor has: page ID u64 at 0; token tag u8 at 8 (`1` bootstrap, `2` WAL); locator tag u8 at 9 (`1` checkpoint image, `2` WAL image, `3` WAL delta); delta depth u8 at 10; source file kind u8 at 11 (`1` checkpoint pack, `2` WAL segment); zero bytes 12-15; token source number u64 at 16; token record LSN u64 at 24; image LSN u64 at 32; image CRC32C u32 at 40; zero bytes 44-47; source file number u64 at 48; source header identity bytes 56-87; locator offset u64 at 88; frame length u32 at 96; frame header CRC32C u32 at 100; payload offset u64 at 104; payload length u32 at 112; payload CRC32C u32 at 116; committed-prefix length u64 at 120; commit-frame offset u64 at 128; commit LSN u64 at 136; base page LSN u64 at 144; base image CRC32C u32 at 152; base source file kind u8 at 156 (`1` checkpoint pack, `2` WAL segment); zero bytes 157-159; base source file number u64 at 160; base source header identity bytes 168-199; base locator offset u64 at 200; base frame length u32 at 208; base frame header CRC32C u32 at 212; base image LSN u64 at 216; base image CRC32C u32 at 224; and zero bytes 228-239. Bootstrap token source number and record LSN are zero. WAL tokens require both values to be nonzero. A checkpoint-image locator uses offset and payload fields for its exact 4 KiB image, requires commit and base fields to be zero, and carries its original bootstrap or WAL token. An image locator has zero delta depth and base fields. A delta locator has depth `1..8` and a complete base locator. Unused fields and reserved bytes must be zero.

Nibble zero is the high nibble of the PageId's big-endian byte sequence. Root level is 15 and leaf level is 0; the node at level `L` selects nibble `15-L`. The leaf's selected slot contains the descriptor whose PageId must equal the full lookup key. A child level must be exactly its parent level minus one and its generation must not exceed the parent generation. A COW insert creates only the changed path at one new generation and shares all untouched children. File names are `version-index-` followed by a zero-padded 20-digit decimal file number and `.node`; the number is allocated monotonically by scanning existing names and using create-new semantics. A failed create consumes/skips that number. Spill files use the same name and node bytes with kind 2; they are rebuildable and cannot be installed as a durable manifest root. Each cached node reserves 8,192 metadata bytes before its read buffer and decoded node are created; cache admission and root/operation references retain the reservation until the last owner drops. No file handle remains open after a node operation.

The experimental manifest is a fixed header and checksummed versioned payload in a uniquely named immutable file. It contains store identity, manifest generation, tree root and free-list/high-water metadata, durable version-index root, checkpoint source catalog root, WAL epoch and recovery start fence, and the previous recoverable manifest generation. Publication switches a small checksummed current-manifest pointer atomically. The pointer and containing directory are synced as required by the existing durability contract. A previous manifest remains protected until the new root and all recovery sources are durable and startup can unambiguously select the new generation. Unknown, partial, or invalid manifest state fails closed; it is never interpreted as a resident manifest.

The manifest payload has fixed-width little-endian fields in this order: generation (u64), tree root (u64), free-list root (u64), high-water PageId (u64), durable index root reference (file number u64, offset u64, checksum 32 bytes), source-catalog root reference (file number u64, offset u64, checksum 32 bytes), WAL recovery file number (u64), recovery offset (u64), committed-prefix LSN (u64), previous manifest generation (u64), and payload checksum (32 bytes). The current-manifest pointer contains magic, format version, store UUID, WAL epoch, selected manifest file number, generation, and checksum. Unknown trailing fields are rejected for this format version.

Temporary spill nodes use the same node encoding and identity checks but are marked rebuildable and are never referenced from a durable manifest. Recovery reconstructs post-checkpoint committed state from the committed WAL prefix. Checkpoint index nodes must be synced before a durable manifest may reference them. Source catalog and dependency traversal state are streamed through bounded spill runs and fixed-size handles; no permanent per-page or per-source in-memory map is allowed.

## WAL locators and reconstruction

Only the shared WAL scanner may parse WAL frames. It stages frame offsets while checking existing framing, page identity, checksums, transaction sequence, rolling digest, and commit marker. It exposes staged locations only after the matching commit and digest are verified. Live writers publish locator changes only after successful required WAL sync, with the writer's synced committed-prefix fence. Offline scans require exclusive recovery ownership and never mutate or sync their input.

An image locator identifies the exact committed image record. A delta locator identifies its exact predecessor and carries base LSN/CRC and result LSN/CRC. Reconstruction iteratively verifies each source identity, committed-prefix fence, range, frame checksum, page identity, version token, predecessor, and intermediate/result checksum. It accepts at most eight consecutive deltas after a full image. The ninth update writes a full image. Missing, cyclic, cross-store, wrong-epoch, replaced, truncated, or changed sources are corruption. A valid committed prefix remains readable if the active segment has a later incomplete tail; no location beyond the prefix is visible.

## Roots, leases, and reclamation

Reachability is the union of durable recovery roots, the currently published generation, operation and reader generation pins, checkpoint snapshots, and active reconstruction/load leases. Runtime-only pins are not restored after restart. Recovery first validates and protects the selected durable manifest and its source roots, then enables reclamation. Source retirement has one protocol across checkpoint completion, startup cleanup, consolidation, failed writes, and shutdown. A source is reclaimable only after bounded mark/sweep or equivalent external traversal proves that no protected root, delta predecessor, or in-flight load can reach it. Unknown files and uncertain orphans are retained. Open descriptors do not substitute for durable reachability.

The source manager uses a bounded handle pool and verifies `SourceIdentity` on every reopen. A load lease is acquired atomically with source reachability before I/O. Retirement cannot unlink a leased source. A missing reachable source is corruption, never a cache miss or not-found result.

## Memory and disk reservations

One `BlinkResourceManager` is shared by all tenants in one `BlinkTenantService`. It maintains checked global and per-tenant current and peak counters. Reservation guards own charges and transfer them explicitly between owners. Shared `Arc` payloads are charged once and released only when the final charged owner drops. Categories are decoded payload, metadata, load/reconstruction, queued foreground work, write preparation, dirty images, and checkpoint/validation. Disk counters separately bound WAL, pack/index output overlap, temporary spill/scratch, and reader/recovery-retained source debt. Allocations, copies, or queue insertion require a conservative reservation first.

The configuration also bounds tenant count, request and response bytes, blocking workers, source handles, loader jobs, and waiters. A non-borrowable progress pool reserves one checkpoint/recovery/GC unit and its required loader; foreground admission cannot use it. Invalid configurations that cannot fit this minimum are rejected. Accepted transactions reserve their complete publication envelope before WAL append. WAL grouping has byte and count caps. A post-sync bookkeeping shortage cannot roll back or forget a committed transaction. Recovery stages large committed batches through bounded spill before installation.

Application byte accounting is not an RSS limit. Caller-owned requests before ownership transfer, returned responses after transfer, allocator retention, runtime stacks, and host caches are separate boundaries. Reports state these boundaries and pair application counters with process/cgroup observations.

## Lock order, waits, cancellation, and errors

The acquisition order is service admission, tenant state, generation/root lease, resource reservation, cache metadata, then source handle. No service registry, generation, cache, or scheduler lock is held across blocking I/O or decode. The scheduler lock is acquired only to select/return a work slice and is released before work. A loader leader reserves bytes and a worker slot before I/O. Followers wait without holding loader permits. Cancellation of a leader removes its in-flight entry, releases all reservations and worker permits, and wakes followers with a retryable cancellation result; followers may elect one new leader. Shutdown rejects new work, cancels queued work, waits for active leases, then closes handles.

Errors distinguish invalid configuration/unsupported format, overload or bounded-capacity rejection, cancellation/shutdown, I/O failure, corruption/identity mismatch, transaction conflict, and logical absence. Only a validated tree lookup may return logical absence. I/O, corruption, and overload never become not-found or conflict. Once WAL sync succeeds, errors in publication bookkeeping are reported as committed-but-recovery-required while retaining the transaction's reserved envelope and recoverable WAL state.

## Fair maintenance slices

One running-checkpoint permit is retained. A fair tenant-ready queue has one pending ticket per tenant. Each slice emits at most 1 MiB of checkpoint image/index output or performs one bounded publication/GC unit; the continuation carries a charged snapshot/root lease and cursor. At most one slice runs at once. With N eligible tenants, deterministic round-robin selection gives each tenant a turn after at most N-1 other ready turns. This is not a wall-clock guarantee when filesystem operations stall. Eligibility, wait, debt, threshold excess, and completed service are tracked per tenant. Per-tenant floors are service opportunities, not preallocated memory.

## State transitions and acknowledged-state protection

| Transition | Ordered operation | State visible to new readers | Sources protecting acknowledged state |
| --- | --- | --- | --- |
| Append | Reserve complete transaction envelope; append frames and commit marker | No new generation | Prior published root; transaction is not acknowledged |
| Sync | Sync WAL through commit marker and validate writer fence | Prior generation until publication | Prior root plus synced committed WAL prefix |
| Publish | Install exact staged index/tree roots without further fallible allocation | New generation atomically | Prior durable root, synced WAL prefix, new runtime root |
| Checkpoint output | Pin target generation; stream bounded images/index; sync outputs | Current generation remains writable | Prior durable root, WAL, target snapshot lease |
| Checkpoint publish | Sync manifest/pointer/directory, install durable root | Current generation; restart selects new durable root | Old durable root and all WAL until new publication is durable, then new root and newer WAL |
| Recovery | Validate marker/manifest, source identities, checkpoint root, then committed WAL prefix | No service until validation completes | Selected durable root and committed WAL prefix; no GC before protection is rebuilt |
| GC | Mark protected roots/dependencies with bounded spill; unlink only proven unreachable files; sync directory | Unchanged | Every durable/runtime root and active lease; uncertain files remain |

## Existing integration points and required migrations

The current tree has these concrete integration points:

| Existing symbol | Planned replacement or adapter |
| --- | --- |
| `dodb-storage/src/blink/mod.rs::BlinkState`, `PageCatalog`, `PageCatalogChunk` | Preserve for the resident route; paged state carries tree root, version-index root, owned guards, and bounded cache handle |
| `dodb-storage/src/wal.rs::WalFile`, `recovery_batches`, `recovery_pages`, `page_chain`, `take_recovery_pages`, `take_recovery_batches`, `page_chain_entry`, `inherit_page_chain` | Shared scanner emits commit-gated source locations; resident consumers remain, paged consumer streams locations to spill/index |
| `dodb-storage/src/blink/segmented_wal.rs::SegmentedWal`, `base_owners`, `open_with_base_images`, `append_batch`, `checkpoint`, `publish_checkpoint_manifest` | Paged source catalog and bounded committed prefix replace the paged route's owner map; resident APIs remain available |
| `dodb-storage/src/blink/page_locator.rs::CheckpointPageLocator` | Reuse source identity and lifetime primitives where compatible; open descriptors alone do not satisfy paged reachability |
| `dodb-storage/src/blink/mod.rs::load_file_with_validation`, `load_base_images`, `recover_data_file`, `BlinkState.pages` construction | Add explicit format dispatch before writable open/cleanup; paged recovery validates marker/manifest and constructs bounded index roots |
| `dodb-storage/src/blink/mod.rs` checkpoint snapshot and image-map construction; `dodb-server/src/physical_checkpoint.rs` coordinator | Paged route uses a pinned root and resumable bounded output slices; resident path remains |
| `dodb-storage/src/blink/mod.rs` startup cleanup and segmented source reclamation in `segmented_wal.rs` | Paged route uses the common root-aware source retirement protocol |
| `dodb-storage/src/blink/mod.rs` page traversal, mutation, free-list and overflow access | Route through fallible owned page guards; resident adapter preserves existing semantics |
| `dodb-server/src/lib.rs::BlinkTenantService` | Explicit opt-in configuration, one shared resource manager, bounded foreground admission, fair maintenance queue |

The resident adapter retains current behavior. The experimental route replaces resident maps and whole-store materialization with the version index, owned guards, bounded cache, source leases, streaming checkpoints/validation, and budgeted scheduler. No route activation occurs until T05 retention and all prior dependencies pass.

## Acceptance mapping

| Contract | Planned proof |
| --- | --- |
| Compatibility and explicit opt-in | Fresh-format dispatch, incompatible-route no-mutation tests, feature/configuration matrix |
| Stable page/source identity | Reused PageId, repeated page records, checkpoint-copy token and pathname replacement tests |
| Index and manifest integrity | Sparse/high PageId, randomized reference map, torn/corrupt/wrong-store and crash-publication tests |
| Exact committed reconstruction | Image/delta oracle, cap-eight/ninth-image, commit/digest/torn-tail/corrupt-unrequested-frame tests |
| Safe retention and GC | Old pinned generation across checkpoint/reopen, load-retirement barrier race, crash-at-unlink tests |
| Resource bounds and progress | Exact-fit/over-limit, simultaneous-owner peak, cancellation/panic, progress-pool isolation and bounded completion tests |
| Read/write/service semantics | Query/Scan/cursor/response limits, conditional/multi-key transactions, conflicts, revisions/tombstones, service crash/write/crash tests |
| Streaming maintenance and fairness | Larger-than-cache checkpoint/validation, output-overlap budget, round-robin turn bound and measured I/O wait |
| End-to-end experiment readiness | Full receipts, source/binary hashes, harness config and separately frozen matched T17 matrix |
