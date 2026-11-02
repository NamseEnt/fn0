# T02 Committed WAL Locations

Status: accepted at source commit `ec3284865d7b52d18f471c76a48238ccc0907702`. Source commits are `589af0cab` (synced append capture and source binding), `ababd98a2` (negative source cases), `4d9bb8f55` (commit-digest rejection), and `ec3284865` (prepared append coverage). The worktree and remote continuation branch are at the same commit.

The shared scanner stages page locations until the matching batch sequence and rolling commit digest validate. A location records the WAL identity, page and record identity, exact frame/payload offsets and checksums, commit fence, version LSN/CRC, predecessor, and bounded delta depth. Offline scans leave the source unchanged and report both the valid committed prefix and trailing bytes. Reads revalidate the source INIT identity, target frame, all sibling records in its transaction, commit marker, and digest before returning a payload.

When location tracking is explicitly enabled, both normal and prepared append encoders collect relative locations before writing. The writer resolves them against the append offset and current page chain before writing, then publishes them only after `sync_data` succeeds. A failed sync leaves the new locations unpublished. Location tracking remains opt-in and no service route consumes it.

Image and delta versions reconstruct iteratively from the exact predecessor or a supplied checkpoint base. The writer permits eight consecutive deltas and writes a full image on the next eligible update. The scanner and reconstructor verify depth, source identity, base LSN/CRC, result LSN/CRC, page identity, bounds, and transaction digest. Tests cover all versions of a two-delta history, independent concurrent reconstructions, wrong identity/base/result metadata, a truncated source, changed sibling bytes, incomplete commits, and a valid frame with a mismatched commit digest.

Checks at `ec3284865d7b52d18f471c76a48238ccc0907702`: `cargo test -p dodb-storage --features blink-demand-paging wal::tests:: --lib` passed 37 tests; `cargo check -p dodb-storage --features blink-demand-paging --lib` passed; rustfmt checks passed. Strict Clippy still exits 101 on 12 pre-existing findings in resident Blink/WAL code; the T02 additions introduce no finding. Exact command logs and hashes are in `logs/` and `receipt.json`.

The temporary scan recovery map and returned location vector are O(page count) and O(committed records), respectively. The plan permits these collectors only in T02; T03-T04 replace them with bounded persistent structures and streaming recovery. No process-memory, service, density, or performance result is claimed. The experimental route remains disabled.

One initial prepared-writer test used a generic B-tree image with the B-link-only prepared API and failed before writing. That fixture failure is preserved in `logs/failed-prepared-append-fixture.log`; the corrected B-link fixture test passed.

Next task: T03, the persistent bounded version index.
