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

## Continued writer preparation and failure boundaries (2026-11-02)

- Before WAL append, the writer now reserves WAL buffer bytes, the bounded version-index insertion budget, and a source-root registration slot. The source-root reservation also reserves the root registry's hash capacity and root ID, and source collection defers while a generation reservation is active. Ordinary root pinning still validates the referenced index node.
- Added paged-writer append and sync fault injection. The focused file-backed fixture covers append failure, sync failure, and post-sync publication failure: each keeps the old reader generation visible and puts the failed writer into fail-stop. The successful transaction still publishes an exact full-image locator after sync and preserves the pinned old reader's value.
- Source commit: `2aaf794b4e9372ecdc81e93da0a2d5fd460f51cf`.
- Current feature library suite: 293 passed, 0 failed, 4 ignored. Default and feature library checks and targeted rustfmt passed. Strict feature Clippy still reports the same 12 existing findings, with no new findings from this change.

T09 remains partial. At this stage the writer still emitted full images, committed one transaction per append, and lacked the complete-or-absent reopen oracle, grouped transaction durability, ninth-delta image selection, and durable tombstone/ABA and page-reuse acceptance. The paged route remains disconnected from `dodb-server`; defaults remain unchanged. No OCI or performance experiment was run.

## Bounded delta ancestry in paged writes (2026-11-02)

- Added a per-dirty-page base-version input to WAL redo planning. The paged writer loads and reserves current dirty-page base images before append, without collecting a history-wide page map.
- The writer now emits page deltas when the encoded delta is smaller and the indexed predecessor has fewer than eight deltas. After WAL sync it writes and syncs the immutable delta-chain sidecar using a pre-reserved metadata charge, then inserts the descriptor that names the exact sidecar. Sidecar or index publication failures keep the writer fail-stopped; unreferenced sidecars remain preserved.
- The focused writer test verifies a full-image base, delta depths 1 through 8, and a full image on the ninth update. The feature library suite completed with 293 passed, 0 failed, and 4 ignored. Default and feature library checks and targeted rustfmt passed. Strict feature Clippy still reports the same 12 existing findings.
- Source commit: `78f7ad58c737ee3be118b132c17c75c72f47fe72`.
- The package-wide feature test command remains blocked by the pre-existing `phase0-bench.rs` initializer missing `WalRedoStats::image_delta_chain_limit`; the raw output is preserved in the T09 logs.

T09 remains partial. Grouped transaction boundaries and recovery, page reuse and durable ABA acceptance, split/overflow differential coverage, and delta-chain reconstruction after close/reopen and WAL rotation remain outstanding. The paged route remains disconnected from `dodb-server`; defaults remain unchanged. No OCI or performance experiment was run.


## Reopen oracle for paged-writer fault boundaries (2026-11-02)

- Extended the focused file-backed fixture to close and reopen each faulted WAL segment with the complete checkpoint page set as recovery bases. This covers delta records on pages beyond the first changed page.
- The pre-commit append failure recovers the unchanged checkpoint page set; the injected sync failure recovers at least one changed committed page. Every recovered image decodes and its embedded page LSN matches the recovered commit LSN. This is a WAL-level complete-or-absent oracle for these two boundaries, not a full store/service reopen or grouped transaction proof.
- Focused test: 1 passed. Full feature library suite: 293 passed, 0 failed, 4 ignored in 122.14 seconds. Targeted rustfmt and `git diff --check` passed.
- The first two reopen attempts seeded only page 2, while the faulted transaction also emitted a delta for page 3. Both failed logs and their temporary test directories are preserved; the corrected test seeds all existing checkpoint pages and passes.
- Raw logs: `logs/paged-writer-reopen-fault-oracle-final.log` and `logs/paged-writer-reopen-feature-lib.log`. The intermediate failed logs and preserved WAL directories are retained alongside them.

T09 remains partial. Grouped transaction recovery, durable tombstone/ABA and page reuse, split/overflow differential coverage, and delta-chain reconstruction after close/reopen and WAL rotation remain outstanding. The paged route remains disconnected from `dodb-server`; defaults remain unchanged. No OCI or performance experiment was run.
