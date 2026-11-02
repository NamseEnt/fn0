# T11 Partial Results: Bounded Paged Storage Validation

## Scope

`PagedReadHandle::open_with_indexed_page_validation` and `validate_all_indexed_pages` now perform a complete storage-graph validation over a paged generation. The index descriptor cursor visits every allocated page in order. Page images are decoded through the bounded loader and checked against descriptor LSNs and checksums.

The validator reserves three pages from `CheckpointValidation` and reserves one scratch byte per page ID from the tenant `Scratch` disk budget. It uses the scratch file to mark the root, internal-child, overflow, and free-list roles. This detects duplicate ownership, cycles, unreachable pages, unlisted free pages, and references outside the high-water range without an all-page in-memory set. Scratch files and reservations are released on success and error.

The validator also checks internal child types and levels, separator/child fences, leaf and internal sibling continuity, overflow ownership and lengths, and free-list page types. A corrupt leaf not touched by the requested read fails validation. Test fixtures cover a missing allocated page, an omitted free list, a free-list cycle, and a multiply owned tree child.

## Validation

- `rustfmt --edition 2024 --check crates/dodb-storage/src/blink/mod.rs crates/dodb-storage/src/blink/source_manager.rs`: passed.
- Focused paged validation regression: passed, 1 test.
- `cargo test -p dodb-storage --lib --features 'blink-demand-paging blink-writer-pin-reuse' -- --test-threads=1`: passed, 310 tests; 4 ignored.
- `git diff --check`: passed.
- Strict Clippy remains blocked by 12 existing diagnostics in `blink/mod.rs` and `wal.rs`; none point to the new paged validator or `source_manager.rs`. No lint suppression was added.

## Remaining T11 Work

The bounded structural checker is implemented for explicit paged validation opens. Recovery/service open-path integration, above-cache reopen with a large WAL, and the required storage process-kill/crash-boundary matrix remain. T11 is partial until those gates have evidence.
