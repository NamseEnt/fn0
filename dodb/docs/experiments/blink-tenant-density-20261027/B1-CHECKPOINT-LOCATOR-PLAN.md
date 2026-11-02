# B1.0 validated checkpoint-image locators

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

## Decision and first boundary

The completed resident-frontier and N24 maintenance evidence does not support adopting the current B-link implementation for greater active-tenant density or practical disk efficiency. Further small-store repetitions are not the next investment. Continue with the first correctness prerequisite of B1: identify and reload an exact immutable checkpoint image without retaining every encoded/decoded payload. B2 admission/cache budgets must follow a reconstructable-version contract, not precede it with unsafe eviction.

This increment implements an opt-in, read-only checkpoint-pack locator prototype. It is not a page cache, service backend, WAL reconstruction path, durable generation root, page-reuse incarnation scheme, or complete B1/B2 implementation. The existing resident reader/writer/checkpoint/recovery routes remain the only service routes. Do not run a new density matrix or claim memory savings from this primitive.

## Source-confirmed constraints

At `5603a58f7110b7979b4d16e332291d1593f211e0`, `BlinkState.pages` retains decoded pages and `GenerationPin` reaches generation-owned `PageCell` payloads. `durable_bases::read_pack` validates every pack header/image and the aggregate CRC before successful recovery, but returns all images in an in-memory map. Fixed-size records already permit exact offsets after a validated scan. `base_owners` identifies only the latest owning pack; checkpoint reclamation unlinks packs/WAL according to the latest manifest. `WalLog.page_chain` has LSN/CRC state but no reconstruction offsets. None of these structures supplies a pinned-old-version miss path.

Use the existing on-disk format and full corruption validation. Share one streaming scanner with existing recovery rather than creating a second, weaker parser. Its default recovery consumer must preserve existing page ordering, high-water, identity/header padding/CRC, page decode/CRC, checkpoint LSN, file-length, aggregate checksum, and trailer checks. Do not change manifest publication, sync ordering, pack creation, page reuse, or deletion policy.

## Implementation contract

- Add default-disabled Cargo feature `blink-checkpoint-locators` and a public experimental module available only under that feature.
- Open a specific file once, validate the entire pack, and build only page-ID/offset/LSN/image-CRC metadata. Publish no locator after partial validation. Keep one encoded-page scanning buffer; the map is still O(pack page count), not a bounded metadata cache or exact heap accounting.
- A locator holds the validated source handle, tenant/database/shard identity, immutable pack/checkpoint identity, and exact image metadata. It must not reopen a pathname and accidentally read a newer file after replacement or reuse.
- Reload exactly one fixed-size record through the held handle, checking length, its recorded image checksum, page-ID decoding, and exact image LSN. Return an owned encoded page to the caller; do not retain loaded payloads in the locator/index.
- Serialize access to the shared seek cursor so cloned locators may load concurrently without cross-record reads. This is neither global I/O scheduling nor concurrent miss coalescing.
- Demonstrate that an old locator remains tied to its original open file after the index is dropped and the pathname is unlinked/replaced. Unix open-file lifetime retention is process-local only: it is not manifest reachability, crash-safe retention, physical disk return, or permission to integrate current reclamation with eviction.

## Acceptance and exclusions

Compare every indexed image byte-for-byte with existing recovery, including sparse IDs and free/leaf images. Test wrong tenant/database/shard/epoch/pack/checkpoint identity, invalid lengths/header padding/CRC/trailer, unsorted/duplicate/out-of-range pages, invalid page CRC and page LSN beyond checkpoint. Corruption in an unrequested record must reject the entire initial scan. After validation, detect a changed/truncated requested image and reject its reload. Test repeated and concurrent loads, source ownership across index drop/path replacement, missing page lookup, and absence of payload retention in the index design.

Run feature/default storage checks appropriate to the shared scanner, including existing durable-base segmented checkpoint/recovery/fault cases and compatibility with `blink-writer-pin-reuse`. Keep tests as correctness evidence, not performance or tenant-density evidence. Do not add comments, abstract one-letter variable names, lint suppressions, or commit coauthor trailers. Preserve prior evidence and use the dedicated worktree only.

The real-service crash/write/crash gate remains mandatory before any future writer/recovery/publication/eviction integration. This read-only prototype cannot satisfy that gate or the larger-than-memory objective. Next gates are committed-WAL image/delta offsets and bounded replay, pinned version/recovery-root retention across manifest publication/reclamation, generation-aware reader/writer loading, bounded catalog/cache/loading/dirty/queue budgets, and then one-tenant forced-eviction/recovery correctness before many-tenant comparisons. Keep #126 open and defaults unchanged.
