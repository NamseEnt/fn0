# Phase F correctness

## Test runs

| Step | Commit | `cargo test --workspace --release --no-fail-fast` |
|---|---|---|
| F0 leaf probe (counters only) | `9d9927a` | not a store change; storage lib tests pass |
| F1 packed leaf | `fe9dfad` | 222 passed, 0 failed, 1 ignored |
| adaptive parallel threshold | `74d65b6` | 223 passed, 0 failed, 1 ignored |

On `74d65b6` also: `cargo test -p dodb-storage --lib` in debug (full page validation on the prepared WAL path, published-leaf check) 146 passed, 1 ignored; storage lib tests with `--features churn-counters` pass. Logs in `correctness/`.

Kept unchanged and passing: Phase B WAL / recovery / fault tests (append and checkpoint fault matrices, torn pages, malformed deltas), Phase D parallel differential tests (serial vs 1 and 2 lanes, same-leaf chains, mixed chains, worker failure and panic, 17-point fault matrix, three-seed randomized differential with byte-identical WAL), and Phase E's pinned-generation test (page objects, payload pointers, bytes, images, scans across later parallel and serial writes, splits, a delete, an overflow value and a checkpoint).

## New tests

- `packed_leaf_matches_reference_model_randomized` (F1): 400 seeds × 120 operations. Leaves start empty, small, or nearly full; values are inline (0–400 bytes), overflow references, or tombstones. After every insert, same-length replace, different-length replace, revision change and remove, the packed leaf is compared with a Phase-E-style owned entry list: logical entries, `search` against `binary_search` for present and absent keys, fit check, encoded page bytes against the independent reference encoder (`encode_leaf_body` + `encode_page`), decode round trip, split index against the same rule on the reference list, and both split halves. Clones must be equal and use their own buffer. The test also checks that compaction runs.
- `leaf_page_clone_copies_packed_entries_into_its_own_buffer` and `working_overlay_isolates_packed_entries_and_restamps` (F1, replacing Phase E's two Arc-sharing tests): a clone has different key/value addresses and equal contents; changing and restamping a working-overlay copy leaves the committed page object, its payload addresses and its contents unchanged.
- `parallel_min_group_mutations_sends_small_groups_to_the_serial_executor` (adaptive): a 3-mutation group goes to the serial executor and a 12-mutation group runs in parallel; commit LSNs, dirty images, scan and WAL bytes match a serial-only store.

## Phase E vs Phase F, byte for byte

`crossver/main.rs` (public `BlinkStore` API on real files): 120 groups of 1–48 random transactions of width 1–16 over 4,000 keys, with deletes, overflow values, `Exists` / `NotExists` conditions, a checkpoint and a flush; then 572 point gets, a 50-row query on each of 97 partitions, a full scan, the data file and the WAL file are hashed, and the store is reopened and scanned. Seeds 11–13, value limits 120 and 40 bytes, 0 and 2 lanes.

Built against Phase E (`4ead3de`), F1 (`fe9dfad`) and final Phase F (`74d65b6`): all 12 lines are identical across the three builds (transaction results and commit LSNs, gets and queries, scan, reopen scan, data-file bytes, WAL bytes).

## Reopen / decode

`reopen/main.rs`: 100,000 rows (benchmark-like keys, 64-byte values), checkpoint, then 5 × `open_path` (reads and decodes all 6,759 pages), counting allocations with a global allocator. Local MacBook (not OCI):

| | Phase E `4ead3de` | Phase F `fe9dfad` |
|---|---|---|
| allocations per open | 274,632 | 81,258 |
| allocated bytes per open | 79.5 MB | 78.3 MB |
| open time, 5 runs | 33.4–35.5 ms | 32.2–36.3 ms |

Decode now makes one slot vector and one byte buffer per leaf instead of a key and a value `Arc` per entry; open time is dominated by reading the file and did not change.
