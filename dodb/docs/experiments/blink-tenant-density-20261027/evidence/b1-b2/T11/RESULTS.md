# T11 Partial Results: Bounded Paged Storage Validation

## Scope

`PagedReadHandle::open_with_indexed_page_validation` and `validate_all_indexed_pages` perform complete storage-graph validation over a paged generation. `PagedManifestStore::open_with_validated_pages` combines durable manifest/index opening with that validation before returning a reader. The index descriptor cursor visits every allocated page in order. Page images are decoded through the bounded loader and checked against descriptor LSNs and checksums.

The validator reserves three pages from `CheckpointValidation` and reserves one scratch byte per page ID from the tenant `Scratch` disk budget. It uses the scratch file to mark the root, internal-child, overflow, and free-list roles. This detects duplicate ownership, cycles, unreachable pages, unlisted free pages, and references outside the high-water range without an all-page in-memory set. Scratch files and reservations are released on success and error.

The validator also checks internal child types and levels, separator/child fences, leaf and internal sibling continuity, overflow ownership and lengths, and free-list page types. A corrupt leaf not touched by the requested read fails validation. Test fixtures cover missing allocation, an omitted free list, a free-list cycle, multiply owned tree child, and scratch release. The validated-open arguments are grouped in `PagedValidatedOpenConfig`.

Recovery now exposes `PagedRecoveryIndexBuilder::validate_recovered_pages`, which validates the builder's recovered index root before publication. The integration test replays a WAL larger than the two-page cache, validates all 24 allocated pages, and reopens the published generation.

Storage tests cover all four paged manifest pointer publication fault boundaries and real process `SIGKILL` at each pointer stage. Source GC is killed before and after unlink, then reopened and retried. Delta-chain sidecar tests reject wrong embedded identity, corrupted payload, truncation, and missing sidecar.

## Validation

- `rustfmt --edition 2024 --check crates/dodb-storage/src/blink/paged_recovery.rs crates/dodb-storage/src/blink/paged_manifest.rs crates/dodb-storage/src/blink/version_chain.rs crates/dodb-storage/src/blink/source_manager.rs crates/dodb-storage/src/blink/mod.rs`: passed.
- Focused paged validation, recovery streaming validation, manifest validated-open, pointer and source-GC `SIGKILL` boundaries, and delta-chain sidecar regressions: passed.
- `cargo test -p dodb-storage --lib --features 'blink-demand-paging blink-writer-pin-reuse' -- --test-threads=1`: passed, 313 tests; 4 ignored.
- `git diff --check`: passed.
- Strict Clippy remains blocked by 12 diagnostics in existing code in `blink/mod.rs` and `wal.rs`; none point to the new recovery validation API, bounded validator, validated-open API, or `source_manager.rs`. No lint suppression was added.

## Remaining T11 Work

The bounded checker is integrated with explicit paged manifest opening and WAL recovery validation. Service tenant opening will route through the validated path as part of T12. The remaining T11 gap is process-kill evidence for WAL-tail recovery, delta/base and pack construction, sidecar/index/manifest ordering, page reuse and pinned checkpoint races, and recovery followed by a write and a second process crash. Existing fault-injection tests are supporting coverage and do not replace those process-kill cases. T11 remains partial until those gates pass.
