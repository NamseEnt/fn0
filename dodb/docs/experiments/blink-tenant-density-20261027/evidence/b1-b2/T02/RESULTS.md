# T02 Committed WAL Locations

Status: partial. The shared scanner now records page image/delta frame offsets, payload offsets and lengths, payload CRC32C, header CRC32C, record LSN, page ID, batch and record identity, and commit frame/LSN. Locations remain staged with the pending batch and become visible only after commit sequence and rolling digest validation. `WalLog::scan_committed_page_locations` is read-only and returns only committed locations plus the committed-prefix length and incomplete tail size. The returned scan does not retain reconstructed page images.

Checks: `cargo test -p dodb-storage --features blink-demand-paging wal::tests:: --lib` passed 29 tests. `rustfmt --check --edition 2024 --config skip_children=true dodb/crates/dodb-storage/src/wal.rs dodb/crates/dodb-storage/src/lib.rs` passed.

This scan temporarily uses the existing page recovery map to validate deltas, then drops it before returning. T02 allows a temporary O(page count) foundation collector, but a bounded paged recovery consumer is not implemented. Live append-side location publication after successful WAL sync, exact image/delta reconstruction from retained locators, source identity and changed/truncated-source validation, and the remaining T02 acceptance cases are not implemented. No runtime route consumes these locations yet.
