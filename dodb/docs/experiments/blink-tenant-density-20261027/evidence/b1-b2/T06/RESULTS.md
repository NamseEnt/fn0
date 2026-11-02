# T06 fallible page access and writer interfaces results

## Changes

- Replaced the mutation page lookup's infallible borrowed result with `Result<Option<OwnedBlinkPageGuard>>`.
- Added an Arc-backed owned guard and implemented resident and working-state adapters.
- Stored working overlay pages in `Arc<BlinkPage>` and mutated them through copy-on-write, preserving independent mutation state and immutable shared versions.
- Propagated access errors through mutation routing, cache hints, catalog preparation, free-page allocation, and the split/reuse/overflow call paths. Required missing pages continue to return corruption.
- Added tests proving a page guard remains readable after its resident map entry is removed and that I/O, corruption, and overload errors survive mutation traversal.

## Verification

The serialized feature library run completed with 273 passed, 0 failed, and 4 ignored. The error-propagation and guard-lifetime tests passed in this run. Default and `blink-demand-paging` library checks passed. The combined `blink-demand-paging,blink-borrowed-page-views` library test target compiled, and rustfmt passed.

The first parallel full-suite attempt had one unrelated temporary-file collision in `full_scan_rejects_resealed_pack_and_checkpoint_header_mismatch` (`AlreadyExists`, OS error 17). That test passed in isolation. The serialized full-suite result is the acceptance result.

Strict feature Clippy remains blocked by 12 existing findings in `blink/mod.rs` and `wal.rs`; the T06 change adds no lint suppression. Full crate `cargo check` remains blocked by an unrelated missing `image_delta_chain_limit` field in the `WalRedoStats` initializer at `dodb/crates/dodb-storage/src/bin/phase0-bench.rs:4786`. Both default and feature library-only checks pass.

## Limits

These results cover the resident storage adapter. They do not establish service integration, paged runtime activation, blocking I/O behavior, eviction, capacity, performance, or OCI evidence. The borrowed-page-views combination was compile-checked, not run as a second full suite.
