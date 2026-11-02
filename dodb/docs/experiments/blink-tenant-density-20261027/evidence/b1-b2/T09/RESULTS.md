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
