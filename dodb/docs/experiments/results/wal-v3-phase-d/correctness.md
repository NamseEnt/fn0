# Phase D correctness results

Commands (macOS arm64, code `df8b8f4` plus the test-only coverage printout in the next commit):

- `cargo test --workspace --release --no-fail-fast`: all pass (`dodb-storage` lib 142 passed, 1 ignored = the B0 probe; every other test binary passes).
- `cargo test -p dodb-storage --no-fail-fast` (debug build): 142 passed, 1 ignored. In debug builds the prepared WAL path also runs the strict checks (full Blink page validation of image records, image CRC recheck, delta payload decode), and the coordinator checks that every published leaf handed to a job equals the committed working-state leaf.

## New tests (`crates/dodb-storage/src/blink/mod.rs`)

| Test | What it checks |
|---|---|
| `phase_d_independent_sixteen_leaf_transaction_matches_serial` | One transaction on 16 distinct existing leaves, for 1 and 2 lanes: same result and commit LSN as the serial executor; identical in-memory pages, dirty images, superblock, LSN/revision/batch counters; the commit has 16 PageDelta records; the coordinator's serial mutation and encode timers did not move; 16 leaf jobs, 16 operations, 0 fallbacks; data file and WAL file byte-identical to serial. |
| `phase_d_same_leaf_chain_keeps_fifo_and_delta_bases` | A → leaf 1, B → leaf 1 (other key), C → leaf 1 (A's key again), D → leaf 2. One parallel group, 2 jobs, 4 operations. On disk the three leaf-1 commits are PageDeltas with increasing commit LSNs; A's key ends with C's value and C's commit LSN as revision, B's key has B's commit LSN. WAL byte-identical to serial (so the delta chain A target → B base → C base is the same). |
| `phase_d_mixed_leaf_chains_match_serial` | A → L1 L2, B → L2 L3, C → L4, run twice: 2 parallel groups, 8 jobs, 10 operations, byte-identical files and identical state after each group. |
| `phase_d_worker_failure_is_atomic_and_leaves_store_usable` | A 16-leaf transaction where one leaf job returns an error, and again where it panics: the group returns an error, the store is not marked broken, WAL bytes / sync count / next LSN unchanged, pages and dirty images unchanged, versioned readers and `get` see only the old values. The same transaction then succeeds; reopen shows it. |
| `phase_d_parallel_wal_recovers_like_serial` | 40 random groups (1–20 transactions, width 1–16) on serial and 2-lane stores: same results, state and documents after each group; WAL bytes identical; both reopened without checkpoint give the same documents and pages, pass invariants, and keep producing identical results and files for 5 more groups (≥ 4 of them parallel after reopen). |
| `phase_d_fault_matrix_keeps_acknowledged_durability` | Two width-16 transactions (32 distinct leaves) per group, fault at every occurrence of 17 points: `before_parallel_leaf_dispatch`, `after_parallel_leaf_join`, `before_wal_append`, header / delta-header / payload / delta-payload / trailer writes, `after_page_delta_record`, `after_page_images_written`, `before_commit_record`, `after_commit_record_write`, `after_group_records_written`, `before_wal_sync`, `during_wal_sync`, `after_wal_sync`, `before_generation_publication`. 212 injected failures. Each time: the group returns an error and no reader sees it; after reopen each transaction is all-or-nothing, the second is never present without the first, nothing is present for faults before the WAL append, both are present for faults after both commit records were written (including after the sync and before publication, where the store refuses to publish and is marked broken); more parallel writes and a second reopen keep every value. |
| `phase_d_prepared_redo_with_wrong_chain_base_writes_nothing` | The WAL's prepared-redo path rejects a delta whose base CRC or base LSN does not match the page chain and writes no byte; the correct base is accepted. |
| `phase_d_randomized_differential_serial_one_and_two_workers` | See below. |

Updated existing tests: `parallel_multi_leaf_transaction_runs_on_leaf_workers` and `parallel_cross_leaf_condition_dependency_keeps_fifo_results` (were "falls back" tests; multi-leaf and cross-leaf condition groups now run in parallel and must match serial including the on-disk WAL); `persistent_parallel_pool_reuses_worker_threads_across_execute_cycles` (queue API, 64 jobs, 3 lanes, every job returned once). `parallel_worker_completion_order_does_not_change_wal_order` was removed with the function it tested; the order property is now covered by the byte-identical WAL checks above, where the jobs finish in any order.

## Differential test

`phase_d_randomized_differential_serial_one_and_two_workers`: three seeds, 500 seeded keys, 80 groups each of 1–24 transactions. Each transaction: width 1–16 over 625 keys (so some are inserts, some cause splits), 8% deletes, 2% duplicate-key requests (rejected), 6% `Exists` and 3% `RevisionEquals` conditions (conflicts), and in every 8th group 2% overflow-size values. A `flush()` after group 30 (the next delta bases come from re-encoding, not the dirty map) and a checkpoint after group 55 (WAL reset; the next touch of each page is a full image).

Stores: serial planned executor, 1 lane, 2 lanes. After every group: identical per-transaction results (commit LSN or error), or the same group-level error (the serial executor rejects a whole group when an update no longer fits in its leaf; the parallel path falls back and reproduces it), identical pages, dirty images, superblock, active slot, next LSN / revision / batch id. At the end: identical documents from a full scan, invariants pass, data file and WAL file byte-identical to serial, and each reopened store gives the same documents.

Coverage (`--nocapture`):

| Seed | lanes | parallel groups | fallback groups | of which overflow / structural | found after dispatch | single-leaf groups | leaf jobs | operations |
|---|---|---|---|---|---|---|---|---|
| 11 | 1 | 57 | 21 | 14 / 7 | 14 | 1 | 1,488 | 4,320 |
| 11 | 2 | 57 | 21 | 14 / 7 | 14 | 1 | 1,488 | 4,320 |
| 29 | 1 | 47 | 33 | 21 / 12 | 26 | 0 | 1,246 | 3,451 |
| 29 | 2 | 47 | 33 | 22 / 11 | 26 | 0 | 1,246 | 3,451 |
| 47 | 1 | 47 | 32 | 22 / 10 | 24 | 0 | 1,188 | 3,222 |
| 47 | 2 | 47 | 32 | 21 / 11 | 24 | 0 | 1,188 | 3,222 |

The fallback decision is the same for 1 and 2 lanes. The recorded reason can differ when several jobs of one group fall back for different reasons: the coordinator records the first one it collects, and collection order depends on which lane finished first.

WAL bytes: byte-identical to the serial executor in every test. The worker makes the same image-or-delta choice the WAL made in Phase B (image on first touch after a reset, image when the delta is not smaller, otherwise the canonical delta against the previous committed image), and the coordinator orders each commit's records by page ID, the serial order. No difference was found, so none needs explaining.
