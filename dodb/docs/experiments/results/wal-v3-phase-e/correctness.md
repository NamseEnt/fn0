# Phase E correctness

## Test runs

`cargo test --workspace --release --no-fail-fast` passed after every step; logs in `correctness/`.

| Step | Commit | Workspace tests passed / failed / ignored |
|---|---|---|
| E0 counters | `b783686` | 219 / 0 / 1 |
| E1 image ownership | `ef99cdb` | 220 / 0 / 1 (adds the delta-check test) |
| E2a shared pages | `df2ce35` | 220 / 0 / 1 before the pinned-generation test was added, 221 with it (the commit includes it) |
| E2b inline keys (reverted) | `db6499f` | 222 / 0 / 1 |
| E3 planner temporaries | `b445043` | 222 / 0 / 1 |
| final (E2b reverted) | `4ead3de` | 221 / 0 / 1 |

Also on the final commit: `cargo test -p dodb-storage --lib` in debug (full page validation on the prepared WAL path and the published-leaf check) 144 passed, 1 ignored; and the storage lib tests with `--features churn-counters` (where `LeafEntry` has a counting `Drop`) 144 passed, 1 ignored.

The Phase D suites run unchanged in every step: serial vs 1 and 2 lanes on the 16-leaf transaction, same-leaf chains, mixed chains, lane error and panic, the 17-point fault matrix, the wrong-chain-base check, and the three-seed randomized differential (results, commit LSNs, pages, dirty images, superblock, counters, scans, invariants, data file, byte-identical WAL, reopen).

## New tests

- `page_delta_rebuild_check_matches_apply_and_compare` (E1): 3,000 random page pairs; for the real after-image, a one-bit-off image and a changed base, `page_delta_rebuilds` returns the same answer, or the same error text, as `apply_page_delta(...) == image`.
- `pinned_generation_pages_entries_and_values_survive_later_writes` (E2a), with 0 and 2 lanes: a pinned generation keeps the same page objects, the same contents and encoded images, the same key and value payload pointers and bytes, and the same scan, across four rounds of 16-leaf transactions plus same-leaf chains, 300 inserts with leaf splits, a delete, an overflow value and a checkpoint. After the writes every committed page is the same object as the one in the published catalog.
- `leaf_keys_inline_short_keys_and_share_long_keys` (E2b only, removed with the revert).

## Phase D against Phase E, same inputs, byte for byte

`crossver/main.rs` is a small program on the public `BlinkStore` API (real files through `open_path`). It runs 120 groups of 1–48 random transactions of width 1–16 over 4,000 keys, with deletes (10%), overflow values (1 in 40), `Exists` / `NotExists` conditions, a checkpoint after group 70 and a flush after group 95; then it hashes every transaction result (commit LSN or error text), the final scan, the data file and the WAL file, and reopens the store and scans again. It was built twice, against the Phase D source (`bff651e`) and against the final Phase E source (`4ead3de`), and run for seeds 11, 12, 13, value limits 120 and 40 bytes, and 0 and 2 lanes.

All 12 output lines are identical between Phase D and Phase E (`crossver/output-*.txt`): same accepted transactions and group errors, same commit LSNs, same documents, same reopen scan, same data-file bytes and same WAL bytes. Serial and 2-lane runs are also identical to each other in both versions.

Observation, not new in Phase E: many groups in this workload end with the group error "document key and value cannot fit in a Blink leaf" (22–66 of 120 groups), from an update that makes a full leaf larger. Phase D and Phase E return the same error for the same groups.
