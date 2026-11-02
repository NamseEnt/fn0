# T08 demand-loaded read route results

## Changes

- Added `PagedReadHandle` over an immutable version-index root, root lease, bounded page loader, source manager, and bounded immutable overlay segments.
- Routed Get, Query, Scan, overflow reads, cursor traversal, and response-size budgets through owned page guards.
- Changed the page cache to retain decoded B-link pages and added a successful source-read counter for correctness evidence.
- The route has no `BlinkState.pages` payload map or duplicate owner directory.

## Acceptance

The integrated fixture contains eight indexed checkpoint pages and configures a two-page decoded cache, so the indexed dataset is four times the cache capacity. The named test compares deterministic seeded values against a sorted reference, verifies a 16-row Query and multi-page Scan, checks tombstones, empty/missing values, exclusive cursor boundaries, response-budget rejection, and overflow reads. It observes at least three direct source reads and rejects changed source bytes on a cache miss.

The fixture also retains an old read handle while publishing a new root with a split leaf and an overwritten value. Barrier-started readers verify the old handle reads its original overflow value while the new split tree reads the replacement and returns the full ordered result. Bounded cache charge, coalescing, cancellation, and eviction ownership are covered by the page-cache tests listed in the receipt.

## Verification

The post-change serialized feature library suite completed with 293 passed, 0 failed, and 4 ignored. Default and feature library checks, package formatting, and `git diff --check` passed. Strict feature Clippy still reports the same 12 existing findings in `blink/mod.rs` and `wal.rs`; no lint suppressions were added.

Source commits `c84bca9f1a5c4bcd02142e18418e7f49878dcce0` and `81edcc6ca63d45d8edd47c8ce4a0ead60ed55ed3` are pushed to the continuation branch. Raw logs, source hashes, commands, and test mapping are recorded in `receipt.json`.

## Limits

This is storage-fixture correctness evidence. The production service route remains disabled, defaults are unchanged, and no OCI run or performance matrix was started. Source-read counts establish source access, not physical-device misses or performance. T09-T16 remain required before B1/B2 implementation handoff; T17 remains separately authorized.
