# T03 persistent bounded version index

Status: accepted for the T03 card. Source commits: `c8126d965` and `17b32a65f`; final source: `17b32a65f`. Both commits were pushed to `experiment/blink-tenant-density-20261027`.

The opt-in storage feature now exposes an immutable 16-level radix index with 4 KiB checksummed nodes, store identity and generation validation, copy-on-write updates, fixed-width source-bound version descriptors, and resource-accounted node caching and root pins. Nodes are written as uniquely numbered sidecar files. New and updated nodes are rebuildable spill nodes; this implementation does not publish a durable manifest root. The normal route remains unchanged.

The focused index suite passed all eight tests. Coverage includes all descriptor encodings, sparse and maximum `PageId` values, an index with at least 448 on-disk nodes and a two-node cache, copy-on-write root preservation, page reuse with distinct version tokens, metadata-limited root pins, wrong-store/corrupt/truncated nodes, wrong levels, generation ancestry, and generation overflow. The full storage library suite passed 247 tests with 4 ignored when run serially.

Strict Clippy still exits 101 with 12 findings in existing `blink/mod.rs` and `wal.rs` code; it reported no finding in `version_index.rs`. A parallel full-suite attempt was also preserved: four existing checkpoint-locator fixtures collided while creating temporary directories. The same full suite passed serially.

Captured logs are losslessly gzip-compressed so their original terminal output, including trailing blank lines, remains intact while the evidence diff passes whitespace checks.

The implementation is local correctness evidence only. It does not route the B-link service through this index or establish bounded recovery, source retention/GC, checkpoint publication, performance, tenant capacity, or production suitability. Continue with T04 durable manifest and recovery-index integration.
