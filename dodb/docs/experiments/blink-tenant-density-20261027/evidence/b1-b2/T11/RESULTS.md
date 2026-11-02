# T11 Partial Results: Bounded Paged Storage Validation

## Scope

`PagedReadHandle::open_with_indexed_page_validation` and `validate_all_indexed_pages` perform complete storage-graph validation over a paged generation. `PagedManifestStore::open_with_validated_pages` combines durable manifest/index opening with that validation before returning a reader. The index descriptor cursor visits every allocated page in order. Page images are decoded through the bounded loader and checked against descriptor LSNs and checksums.

The validator reserves three pages from `CheckpointValidation` and reserves one scratch byte per page ID from the tenant `Scratch` disk budget. It uses the scratch file to mark the root, internal-child, overflow, and free-list roles. This detects duplicate ownership, cycles, unreachable pages, unlisted free pages, and references outside the high-water range without an all-page in-memory set. Scratch files and reservations are released on success and error.

The validator also checks internal child types and levels, separator/child fences, leaf and internal sibling continuity, overflow ownership and lengths, and free-list page types. A corrupt leaf not touched by the requested read fails validation. Test fixtures cover missing allocation, an omitted free list, a free-list cycle, multiply owned tree child, and scratch release. The validated-open arguments are grouped in `PagedValidatedOpenConfig`.

Additional storage tests cover all four paged manifest pointer publication fault boundaries and real process `SIGKILL` at all four pointer stages; source GC is also killed before and after unlink, then reopened and retried and fail-closed delta-chain sidecar checks for wrong embedded identity, corrupted payload, truncation, and missing sidecar.

## Validation

- `rustfmt --edition 2024 --check crates/dodb-storage/src/blink/paged_manifest.rs crates/dodb-storage/src/blink/version_chain.rs crates/dodb-storage/src/blink/mod.rs`: passed.
- Focused paged validation, manifest validated-open, pointer and source-GC `SIGKILL` boundaries, and delta-chain sidecar regressions: passed.
- `cargo test -p dodb-storage --lib --features 'blink-demand-paging blink-writer-pin-reuse' -- --test-threads=1`: passed, 313 tests; 4 ignored.
- `git diff --check`: passed.
- Strict Clippy remains blocked by 12 diagnostics in existing code in `blink/mod.rs` and `wal.rs`; none point to the new validated-open API, bounded validator, or `source_manager.rs`. No lint suppression was added.

## Remaining T11 Work

The bounded structural checker is implemented for explicit paged validation opens and the manifest-plus-reader open helper. Route recovery and service opens through this helper or validate the recovered root before publication. Above-cache reopen with a large WAL and the required storage process-kill/crash-boundary matrix remain. Existing fault-injection and WAL recovery unit tests do not replace the required process-kill evidence. T11 remains partial until those gates pass.
