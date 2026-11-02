# T11 Partial Results: Indexed Page Image Validation

## Scope

This increment adds an explicit `PagedReadHandle::open_with_indexed_page_validation` entry point. It walks the version index with its fixed-depth descriptor cursor and loads each indexed page through the configured bounded page loader. It rejects descriptors outside the allocated range, missing page images, checksum or decode failures, and image LSN mismatches. It does not claim complete B-link tree, free-list, reachability, overflow-chain, or crash-recovery validation.

The focused regression corrupts an indexed leaf page that is outside the already-cached requested route. The cached GET still succeeds, demonstrating why lazy reads alone do not satisfy opening validation; the validation-open path rejects the corrupted image. After restoring the image, validation reads all eight indexed pages with a two-page cache.

## Validation

- `rustfmt --edition 2024 --check crates/dodb-storage/src/blink/mod.rs`: passed.
- Focused `dodb-storage` test `paged_read_handle_routes_get_query_scan_and_response_budgets_through_disk_pages` with `blink-demand-paging blink-writer-pin-reuse`: passed, 1 test.
- `git diff --check`: passed.

The initial compile attempt failed because a local `Seek`/`SeekFrom` import duplicated one in the same test function. That attempt is preserved. The corrected focused run passed.

## Remaining T11 Work

The bounded structural invariant checker, external scratch accounting, high-water and free-list reconciliation, full cross-page structural validation, large-WAL reopen validation, and required crash/recovery matrix remain unimplemented. T11 remains partial and is the next task.
