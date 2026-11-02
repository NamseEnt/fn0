# T11 Partial Results: Bounded Paged Storage Validation

## Scope

`PagedReadHandle::open_with_indexed_page_validation` and `validate_all_indexed_pages` perform complete storage-graph validation over a paged generation. `PagedManifestStore::open_with_validated_pages` combines durable manifest/index opening with that validation before returning a reader. The index descriptor cursor visits every allocated page in order. Page images are decoded through the bounded loader and checked against descriptor LSNs and checksums.

The validator reserves three pages from `CheckpointValidation` and reserves one scratch byte per page ID from the tenant `Scratch` disk budget. It uses the scratch file to mark the root, internal-child, overflow, and free-list roles. This detects duplicate ownership, cycles, unreachable pages, unlisted free pages, and references outside the high-water range without an all-page in-memory set. Scratch files and reservations are released on success and error.

The validator also checks internal child types and levels, separator/child fences, leaf and internal sibling continuity, overflow ownership and lengths, and free-list page types. A corrupt leaf not touched by the requested read fails validation. Test fixtures cover missing allocation, an omitted free list, a free-list cycle, multiply owned tree child, and scratch release. The validated-open arguments are grouped in `PagedValidatedOpenConfig`.

Recovery now exposes `PagedRecoveryIndexBuilder::validate_recovered_pages`, which validates the builder's recovered index root before publication. The integration test replays a WAL larger than the two-page cache, validates all 24 allocated pages, and reopens the published generation.

Storage tests cover all four paged manifest pointer publication fault boundaries and real process `SIGKILL` at each pointer stage. A separate indexed-publication test kills the process after durable index nodes and the temporary pointer are synced but before rename; reopen selects the old manifest and old root while the new index nodes remain orphaned. Another test kills after delta-chain sidecar directory sync but before index insertion; reopen keeps the old checkpoint-image descriptor and the complete sidecar remains unreferenced. WAL recovery is also followed by a successful append, a second process kill during the next append, and a second reopen that retains the recovered write without exposing the incomplete transaction. Source GC is killed before and after unlink, then reopened and retried. Delta-chain sidecars reject wrong embedded identity, corrupted payload, truncation, and missing files; process-kill tests cover sidecar creation, file sync, and directory sync, then reopen each state.

Additional process-kill coverage now starts from a free page with one acknowledged WAL version, then kills a child after it writes an overflow image for that same page ID but before the commit marker. Reopen recovers the acknowledged free-page version and excludes the attempted reuse. The indexed-publication crash test now also pins the previous version-index root while writing the next COW root and manifest pointer; the child is killed before pointer rename, and reopen selects the pinned generation's durable root.

## Validation

- `rustfmt --edition 2024 --check crates/dodb-storage/src/blink/paged_recovery.rs crates/dodb-storage/src/blink/paged_manifest.rs crates/dodb-storage/src/blink/version_chain.rs crates/dodb-storage/src/blink/source_manager.rs crates/dodb-storage/src/blink/mod.rs`: passed.
- Focused paged validation, recovery streaming validation, manifest validated-open, pointer and source-GC `SIGKILL` boundaries, sidecar-before-index and index-before-manifest `SIGKILL`, reused free-page SIGKILL recovery, pinned-root manifest SIGKILL recovery, and delta-chain sidecar regressions: passed.
- `cargo test -p dodb-storage --lib --features 'blink-demand-paging blink-writer-pin-reuse' -- --test-threads=1`: passed, 319 tests; 4 ignored. A preceding full-suite attempt hit the existing background-materialization test's five-second start timeout; its focused retry and this full-suite rerun passed.
- `git diff --check`: passed.
- Strict Clippy remains blocked by 12 diagnostics in existing code in `blink/mod.rs` and `wal.rs`; none point to the new recovery validation API, bounded validator, validated-open API, or `source_manager.rs`. No lint suppression was added.

## Remaining T11 Work

The bounded checker is integrated with explicit paged manifest opening and WAL recovery validation. Service tenant opening will route through the validated path as part of T12. Additional process-kill cases cover an active WAL interrupted before its commit frame, paged base-pack construction, a free-page reuse attempt interrupted before commit, and a pinned old index root during pre-rename manifest publication. The remaining T11 gap is the delta-chain sidecar-to-index boundary through the actual transaction writer. The free-page test exercises WAL recovery at the page-image/commit boundary; it does not replace process-kill coverage through `PagedTransactionWriter` allocation and publication. T11 remains partial until the actual-writer boundary passes.
