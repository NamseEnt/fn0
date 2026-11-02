# T04 implementation status

Status: in progress; acceptance is not claimed.

The opt-in paged manifest format, immutable-generation publication, durable version-index root validation, and resident-route rejection are implemented. Recovery-index construction accepts committed WAL locations and applies them incrementally to copy-on-write index roots. A focused test also writes and syncs a real WAL commit, scans it through the shared WAL parser, and indexes its committed location.

Focused evidence currently passes:

- `paged_manifest::tests`: 8 passed.
- `paged_recovery::tests`: 3 passed, including the real committed WAL path and WAL epoch rejection.
- `version_index::tests`: 9 passed, including durable-root promotion.
- `resident_open_rejects_paged_marker_without_creating_the_database`: 1 passed.
- Targeted `rustfmt --edition 2024 --check` and `git diff --check` passed.
- The opt-in and default-feature library checks passed. Strict Clippy still exits 101 with 12 pre-existing findings in `blink/mod.rs` and `wal.rs`; it reported no finding in the new manifest or recovery modules.

Raw focused test output is retained in `logs/`.

Blocking acceptance gap: `WalLog::scan_committed_page_locations` returns a `Vec<WalPageLocation>` and its shared scanner retains complete page-recovery state and latest-location metadata while replaying. The recovery builder itself can consume an iterator, but this scanner path is not bounded by the index/cache budgets. T04 must replace or extend this path with bounded streaming/spill recovery, including delta-base validation, before the card can be accepted. No complete-recovery, tenant-capacity, service-route, or performance claim follows from the current evidence.

The paged route remains opt-in and is not connected to the service. No performance matrix has been started.
