# T09 bounded paged mutation preparation

## Completed substep

- Added a demand-loaded mutation working state that reads required base pages through the exact-version cache and retains only dirty copies.
- Added `PagedReadHandle::prepare_transaction` for validated conditional and multi-key preparation. It rejects mutation while committed logical overlays remain unmaterialized.
- Reserves the configured maximum working-page footprint and request bytes in `WritePreparation`, and the dirty-image footprint in `DirtyImages`, before applying mutations. Reservations stay live through commit-image encoding and are released when preparation drops or fails.
- Enforces a per-transaction working-page cap and returns encoded after-images with the chosen commit revision.

## Verification

The focused paged read and mutation-preparation test passed. It covers request widths 1, 4, and 16; a successful NotExists condition; a stale revision conflict; overflow allocation and deletion; a working-set overload; reservation lifetime while prepared images are live; and reservation release after overload. The serialized feature library suite completed with 293 passed, 0 failed, and 4 ignored. Default and feature library checks, package formatting, and `git diff --check` passed.

Strict feature Clippy reports 12 existing findings in `blink/mod.rs` and `wal.rs`, matching the established baseline. No lint suppressions were added.

## Remaining T09 acceptance

This is a preparation-only substep. It does not append or sync WAL, publish an exact committed version-index root, or expose a new reader generation. WAL append/sync/publication fault injection, grouped transaction boundary recovery, ninth-delta image emission through this writer, and page reuse integration remain unimplemented. T09 is partial and remains the next task.

No production service route or default was changed. No OCI or performance experiment was run.

## Additional substep: atomic paged transaction publication

- Added `PagedTransactionWriter` for one logical transaction per WAL commit. It writes full after-images, waits for WAL sync, derives exact WAL frame locators, inserts them into a copy-on-write version-index root, and creates a new `PagedReadHandle` generation. Structural changes append the alternate superblock image in the same commit.
- Added a bounded recent-location result to `WalLog`; it retains only locations from the latest successful append unless the older explicit full tracking mode is enabled. The writer validates WAL format and store identity before accepting work.
- The focused storage fixture now exercises a real file-backed WAL commit, loads the committed value through the new generation, and confirms a pinned old reader still returns its previous value. A publication fault after WAL sync leaves the previous generation visible and rejects subsequent writes from that writer.
- Source commit: `7f480c3f649d63f87f3dca365bf4db0681a4e185`.
- Current feature library suite: 293 passed, 0 failed, 4 ignored. Default and feature library checks, focused test, and formatting passed. Strict feature Clippy remains blocked by the same 12 pre-existing findings. Package-wide feature check remains blocked by the existing `phase0-bench.rs` initializer missing `WalRedoStats::image_delta_chain_limit`.

## Remaining T09 acceptance

T09 remains partial. The writer currently emits full images, commits one logical transaction per append, and does not inject append or WAL sync faults through the paged writer. The ninth-delta decision is not exercised by this writer. Publication preparation is not reserved before sync, and durable ABA/page-reuse, grouped transaction recovery, and full crash/reopen acceptance remain outstanding. The route is not connected to `dodb-server`; defaults remain unchanged. No OCI or performance experiment was run.
