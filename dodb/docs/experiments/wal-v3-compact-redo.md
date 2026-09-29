# WAL v3 Compact Redo Experiment

Branch `experiment/wal-v3-compact-redo`. The planned-blink durable baseline this experiment starts from is in `planned-blink-durable-baseline-results.md` (planned-blink / ExactMain multiwriter GM 1.861×, / experiment main-btree 1.645×, / RocksDB 0.363×, 4,224 WAL bytes per width-1 transaction).

WAL v3 / PageDelta is not implemented yet. Phase A below is a prerequisite that fixes a memory problem in the current WAL v2 without changing its format.

## Phase A — runtime WAL replay-payload lifetime

Code commit: `16a5b207fc9d4076fd0a43d4adc539a5d6825904` (`wal: drop committed replay payloads after recovery`). Raw results: `results/wal-v3-phase-a/`.

### Root cause

`WalLog` had a field `committed: Vec<CommittedWalBatch>`. It was filled in two places:

1. `WalLog::open` stored every complete commit found by the WAL scan, with all its page images.
2. `append_group_inner` cloned every page image of every successful commit (`commit.pages.clone()`) and appended it after the WAL sync.

Nothing removed entries except `WalLog::reset` (checkpoint). The baseline sustained runs never checkpoint, so every committed page image stayed in memory for the life of the process. Each width-1 transaction added one 4,096-byte leaf image plus its page ID and batch record, close to the 4,224 bytes it added to the WAL file. That is the 1.04 RSS bytes per WAL byte measured in the baseline, and the reason the 64-writer run hit the memory guard at about 87 s.

### Ownership before the change

Every read of `WalLog::committed` / `committed_batches()`, classified:

| Place | What it read | Class |
|---|---|---|
| `BlinkStore::open_internal` (`recover_data_file(&mut file, wal.committed_batches(), checkpoint_hint)`) | page images of commits newer than the checkpoint | A. recovery only, at open |
| `BTreeStore::open_internal` (`recover_data_file(..., wal.committed_batches(), ...)`) | same | A. recovery only, at open |
| Blink and BTree open, `file.is_empty()? && wal.committed_batches().is_empty()` | whether any commit exists | A. recovery only (emptiness) |
| `BTreeStore::open_internal`, `max_commit_lsn` for `next_revision` | LSN of the last commit | B. metadata |
| `BlinkStore::checkpoint`, `BTreeStore::checkpoint`: `committed_batches().last().commit_lsn` | LSN of the last commit, the new checkpoint LSN | B. metadata |
| `BTreeStore::checkpoint`, `should_reset_wal`: `!committed_batches().is_empty()` | whether any commit exists since the last reset | B. metadata |
| `BTreeStore::check_invariants`: `latest_known_lsn` | LSN of the last commit | B. metadata |
| `WalLog::metrics().committed_batches` = `committed.len()` | commit count since open / reset | C. metrics (count only) |
| Page images kept after `append_group_inner` | nothing in production read them | D. historical payload, not needed |
| Unit tests in `wal.rs` and `blink/mod.rs` | page images of recent commits | test inspection only |

So production needs the page images only between the WAL scan and `recover_data_file` at open. After that it needs one LSN and some counters.

The Blink open flow is: validate the superblock identity and read the checkpoint hint → open the WAL (scan, truncate a torn tail) → if the data file is empty and there are no commits, initialize → otherwise `recover_data_file` writes the newer page images into the data file and syncs it → `load_file` → `resume_after(checkpoint_lsn)`. BTree is the same shape.

Checkpoint needs: the LSN of the last durable commit (or the current superblock checkpoint LSN when there is none), whether there is anything to reset, and the WAL history start.

### Ownership after the change

`WalLog` fields:

- `recovery_batches: Vec<CommittedWalBatch>` — the open-time scan result. Only `WalLog::open` fills it. `take_recovery_batches()` hands it to the caller with `std::mem::take`, which also frees the vector here. `reset` clears it.
- `last_commit_lsn: Option<Lsn>` — set from the last scanned commit at open, set after each successful group sync, cleared by `reset`.
- The commit count comes from the existing `scan_report.committed_batches` counter, which was already equal to `committed.len()` at every point.
- `append_group_inner` no longer clones page images.

Open flow: `let recovery_batches = wal.take_recovery_batches();` → emptiness check → `recover_data_file(&mut file, &recovery_batches, checkpoint_hint)` → `drop(recovery_batches)` → load. If recovery fails, the vector is dropped with the error.

Checkpoint and invariant checks read `wal.last_commit_lsn()`. `should_reset_wal` uses `last_commit_lsn().is_some()`. The checkpoint order is unchanged: flush dirty data pages, sync the data file, write and sync the new superblock, then truncate the WAL, sync, write INIT, sync.

New `WalMetrics` fields `retained_recovery_batches` and `retained_recovery_page_images` show what is still held. Tests read recent commits through a test-only `committed_batches_on_disk()`, which rescans the WAL file.

`phase0-bench` gained `--window-seconds N`. Without the flag the benchmark behaves as before. With it, the JSON row gets per-window tx/s and latency (the same code as the baseline's `dodb-sustained-window.patch`) and a set of samples taken after seeding, at measurement start, at each window end and at measurement end: process RSS, WAL bytes, committed batches, retained recovery payload, dirty pages.

Invariants now:

- open: replay payload is available;
- after recovery: it is dropped (retained counts are 0);
- runtime commit: no page image is kept after the sync.

### On-disk format unchanged

No encoder, decoder or frame layout was touched (`Init`, `PageImage`, `Commit` and the format version are the same). To check the bytes, `scripts/wal_byte_probe.rs` runs a fixed workload through the public API on real files: 2,000 planned-blink puts, an overflow value, a delete, 40 groups of 16 transactions mixing width 1 and 16, a checkpoint, 500 more puts, 100 single-key updates, and a BTree run with a checkpoint. It was run on old (`3ac7515`) and new code, twice each (`wal-byte-probe.txt`):

| File | SHA256 (old = new) |
|---|---|
| `blink.wal` (2,534,504 B) | `02c0da2ab5eecd4711fdfb6a3b1f1327dc82803a1ae34bb0b5ecf748f871247d` |
| `blink.db` | `fc9a84f357a0314d78bffb68e010ecff88c12ca4879efdbf48198f229da86e8d` |
| `btree.wal` (2,514,104 B) | `8f76b74ed48ffdb34a32baa6bfddd68b245ea5f3062db5c396bdc98a9da2f0dc` |
| `btree.db` | `af5cc20d2977f14103c9ed414b6b70964e0531f3cdd4ab64f9ff981daa9483ef` |

All four files match byte for byte, and both runs of each version gave the same hashes. The width-1 existing-key update wrote exactly 4,224 bytes in all 100 samples on both versions. Every benchmark row below also measured 4,224.0 WAL bytes per transaction.

### Correctness

`cargo test --workspace --release --no-fail-fast`: 197 passed, 0 failed. New tests:

- `wal::appended_commits_keep_only_metadata_and_reopen_hands_payload_once`: 3,000 commits in groups of 7 keep 0 retained batches/images; reopen finds 3,000 batches equal to what was written; after `take_recovery_batches` the retained counts are 0, a second take is empty, and `last_commit_lsn` survives; `reset` clears it.
- `blink::runtime_commits_do_not_retain_wal_page_images` (A): 4,000 planned-blink commits, retained payload checked as 0 every 1,000 commits; `last_commit_lsn` equals the last revision.
- `blink::reopen_without_checkpoint_replays_every_commit_and_drops_payload` (B): 3,000 commits, drop without flush or checkpoint, the reopened WAL holds all commits, the reopened store has every value, retained payload 0, invariants pass, later commits still work.
- `blink::checkpoint_uses_last_commit_lsn_without_retained_payload` (C): 1,500 commits → checkpoint LSN equals the last commit, WAL history start moves to it, WAL bytes reclaimed → an idle checkpoint keeps the same LSN and reclaims nothing → 1,000 more commits → reopen: superblock checkpoint LSN kept, every value present.
- `blink::torn_final_commit_is_discarded_on_reopen_without_retained_payload` (D): cut the last 10 bytes of the WAL (inside the final commit frame) → reopen drops that commit, keeps the previous value, truncates only the torn frame (the same rule as the existing `fast_group_torn_tail_keeps_only_complete_commits`), accepts new commits, and reopens again.

Existing WAL, recovery, checkpoint and fault-injection tests pass unchanged except for the test-only accessor swap.

One existing test is flaky on this Mac with and without the change: `wal::group_fast_path_is_byte_identical_to_fault_injectable_path` asserts `group_commit_direct_encode_nanos > 0` for four tiny commits, which can measure 0 with the ~41 ns clock tick. Isolated runs failed 10/200 on old code and 13/200 on new code. It was left as is.

### Short A/B throughput

OCI A1 2 OCPU, ZFS `/bench/zfs/db` (same host and dataset as the baseline). `old-v2` is the baseline core binary (SHA256 `79122436…`, built from `0d310da`; crates are identical to `3ac7515`). `retention-fixed-v2` is built from `16a5b20` (SHA256 `f9d5199f…`). Width 1, uniform, working set 100,000, 2 s warmup, 5 s measure, the same seeds as the baseline scenarios 0 and 6, old/new order swapped every repetition. Every row: `engine=planned-blink`, `sync_mode=real`, 0 errors.

| Writers | Variant | rep1 | rep2 | rep3 | mean tx/s | mean p99 µs | WAL B/tx | mean peak RSS MiB |
|---|---|---|---|---|---|---|---|---|
| 16 | old-v2 | 4,323 | 4,499 | 4,488 | 4,437 | 6,387 | 4,224.0 | 712 |
| 16 | retention-fixed-v2 | 4,586 | 4,568 | 4,297 | 4,484 | 6,424 | 4,224.0 | 155 |
| 64 | old-v2 | 11,777 | 11,216 | 11,436 | 11,476 | 10,769 | 4,224.0 | 918 |
| 64 | retention-fixed-v2 | 12,240 | 11,892 | 11,644 | 11,925 | 10,570 | 4,224.0 | 169 |

New / old: 1.011 at 16 writers, 1.039 at 64 writers. Both are inside the ±5% band, and the repetition spread overlaps, so there is no regression. The 64-writer gain may be real (one less 4 KiB copy and allocation per commit on the single coordinator thread) but this run is too short to claim it.

### 120-second sustained runs

Working set 1,000,000, width 1, uniform, 10 s warmup, 120 s measure, real sync, planned-blink, the baseline seeds, no checkpoint. Same once-per-second process monitor and 256 MiB `MemAvailable` guard as the baseline. The windows use the 12 full 10 s windows; RSS and WAL columns come from the external monitor.

**16 writers, retention-fixed-v2** — completed. 4,308 tx/s, p50 3,597 µs, p99 7,080 µs, 517,036 transactions, 0 errors.

| Window | tx/s | p99 µs | WAL growth MiB | RSS MiB | RSS growth MiB | dirty pages | retained batches / images |
|---|---|---|---|---|---|---|---|
| 0-10 s | 4,205 | 6,893 | 170 | 1,203 | 30 | 68,448 | 0 / 0 |
| 20-30 s | 4,313 | 7,309 | 517 | 1,236 | 64 | 68,448 | 0 / 0 |
| 50-60 s | 4,338 | 6,582 | 1,039 | 1,265 | 92 | 68,448 | 0 / 0 |
| 80-90 s | 4,320 | 7,272 | 1,558 | 1,285 | 112 | 68,448 | 0 / 0 |
| 110-120 s | 4,307 | 6,832 | 2,083 | 1,298 | 126 | 68,448 | 0 / 0 |

**64 writers, retention-fixed-v2** — completed all 120 s (the baseline was killed at about 87 s). 10,166 tx/s, p50 5,598 µs, p99 13,633 µs, 1,220,140 transactions, 0 errors. Peak RSS 1,547 MiB, lowest `MemAvailable` 3,044 MiB.

| Window | tx/s | p99 µs | WAL growth MiB | RSS MiB | RSS growth MiB | dirty pages | retained batches / images |
|---|---|---|---|---|---|---|---|
| 0-10 s | 10,172 | 13,599 | 417 | 1,270 | 37 | 68,448 | 0 / 0 |
| 20-30 s | 10,210 | 13,465 | 1,238 | 1,313 | 80 | 68,448 | 0 / 0 |
| 50-60 s | 10,117 | 14,285 | 2,464 | 1,447 | 214 | 68,448 | 0 / 0 |
| 80-90 s | 10,192 | 13,766 | 3,684 | 1,487 | 254 | 68,448 | 0 / 0 |
| 110-120 s | 10,303 | 13,029 | 4,915 | 1,547 | 314 | 68,448 | 0 / 0 |

Every window is in `results/wal-v3-phase-a/tables.md`.

**16 writers, old-v2 control, same session** — the baseline window binary (SHA256 `fba986d4…`) re-run right after, to get a "before" from the same host state. 4,355 tx/s, p99 7,345 µs. WAL grew 2,105 MiB and RSS grew 2,187 MiB (1.039 RSS bytes per WAL byte), from 5,871 to 8,058 MiB. This repeats the baseline (2,108 / 2,192 MiB, 1.04).

Summary:

| Run | Completed | tx/s | p99 µs | WAL growth MiB | RSS at start MiB | RSS growth MiB | RSS / WAL (0-120 s) | RSS / WAL (60-120 s) |
|---|---|---|---|---|---|---|---|---|
| 16w old-v2 (baseline) | yes | 4,369 | — | 2,108 | — | 2,192 | 1.04 | — |
| 16w old-v2 (control) | yes | 4,355 | 7,345 | 2,105 | 5,871 | 2,187 | 1.039 | 1.009 |
| 16w retention-fixed-v2 | yes | 4,308 | 7,080 | 2,083 | 1,173 | 126 | 0.060 | 0.032 |
| 64w old-v2 (baseline) | no, guard at ~87 s, RSS 9,645 MiB | ~9,600 (derived) | unknown | 3,093 in 80 s | — | 3,238 in 80 s | 1.05 | — |
| 64w retention-fixed-v2 | yes | 10,166 | 13,633 | 4,915 | 1,233 | 314 | 0.064 | 0.040 |

The 1:1 link between WAL history and RSS is gone. The process also starts the measurement much smaller: seeding 1,000,000 rows writes a 4.6 GiB WAL, which used to stay in memory (5.9 GiB RSS at measurement start), and now leaves 1.13 GiB RSS.

### What the remaining RSS is

- Retained WAL payload: 0 batches and 0 images in every sample.
- Dirty pages: 68,448 in every sample, from seeding to the end, in both runs. Uniform width-1 updates only rewrite leaves that are already dirty, so this map does not grow. At 4 KiB per image it is about 267 MiB, already inside the 1.13 GiB after seeding.
- Benchmark timeline: `--window-seconds` stores 16 bytes per successful transaction (finish time and latency). That is 7.9 MiB at 16 writers and 18.6 MiB at 64 writers, up to twice that with `Vec` spare capacity. The baseline window binary had the same cost.
- The rest, about 110 MiB at 16 writers and about 280 MiB at 64 writers over 120 s, is not attributed by this run. It does not follow the WAL: at 16 writers the growth per window falls from 30 MiB to 3–7 MiB, and at 64 writers it comes in steps (for example +60 MiB at 30-40 s and +71 MiB at 50-60 s) with flat windows in between, which looks like allocator arenas or retained state growing in chunks rather than a per-commit cost. In the second half of the run it is 0.03–0.04 bytes per WAL byte. Finding it would need a heap profile; that is outside this phase.

### Notes

- `MemAvailable` on the host was about 5 GiB lower at the start of the later runs than at the start of the session. It excludes the ZFS ARC, which grows while the benchmark writes the WAL. The runs never came close to the guard.
- The primary sustained runs do not checkpoint, as in the baseline. The WAL file still grows without limit; a checkpoint policy is separate work.

### Files

`results/wal-v3-phase-a/`: `environment.txt`, `run-order.txt`, `process-metrics.jsonl`, `short-progress.log`, `sustained-progress.log`, `sustained-control-progress.log`, `wal-byte-probe.txt`, `tables.md`, `raw/` (12 short rows and logs, 3 sustained rows, logs and once-per-second monitors), `scripts/run_phase_a.py`, `scripts/analyze_phase_a.py`, `scripts/wal_byte_probe.rs`, `SHA256SUMS`.

## Phase B — compact PageDelta redo

Question: if ordinary existing-page updates log a compact byte delta instead of a 4 KiB page image, does durable throughput go up clearly? Answer: yes. Planned Blink multiwriter GM is 1.853× Phase A on the 14-scenario matrix. That is the ">1.50×" band of the plan, so full-page WAL was a major bottleneck.

Code (branch `experiment/wal-v3-compact-redo`, not merged):

| Commit | Content |
|---|---|
| `4ac3a2e` wal: add versioned page-delta redo records | WAL format 3, PageDelta record, v3 commit digest, full-image-first page chain, streaming scan, Blink recovery from rebuilt pages |
| `6bcf8bb` blink: emit compact redo for existing pages | planned Blink producer, checkpoint fault points, store-level tests, B0 probe, bench metrics |
| `11639db` wal: keep redo counters across WAL reset | counters only; no format or recovery change |
| `def6f7e` bench: add WAL-size checkpoint control | `--checkpoint-wal-bytes` in `phase0-bench` |

`BTreeStore` still writes format 2 page images; its WAL and data-file bytes are identical to Phase A (`results/wal-v3-phase-b/byte-probe.txt`).

### Format and recovery rule

Full specification: `results/wal-v3-phase-b/format-specification.md`. Summary:

- PageDelta (record type 4, format 3 only) = page ID u64, base page LSN u64, span count u16 (1..=820), then spans of offset u16, length u16, bytes. Header 18 B, 4 B per span. It is always one redo record per dirty page, so `commit_lsn = first_record_lsn + record_count` is unchanged.
- Spans are the maximal changed-byte runs between the previous committed image and the new canonical `encode_blink_page` image, merged left to right when the unchanged gap is at most 4 bytes. Decode and apply reject anything that is not exactly that encoding.
- The v3 commit digest is CRC32C over each record's type, index, length and payload.
- **Full image first:** after WAL initialization or reset, the first committed record for a data page is always a PageImage. Later changes of that page in the same WAL history may be deltas against the previous committed image. The WAL tracks page ID → (page LSN, CRC32C) of the latest committed image, O(unique pages since reset), rebuilt on open and cleared on reset. A delta base that does not match that entry is an invariant error; a page not in the chain is written as a full image.
- **Recovery never uses data-file bytes as a delta base.** The scan keeps page ID → latest rebuilt image, applies a transaction's records only after its commit frame and digest check, and validates every rebuilt page with the full Blink page validator (checksum included) plus the commit LSN. `recover_data_file` then overwrites each page newer than the checkpoint with its rebuilt image. So a checkpoint that crashed after writing some pages, or tore a page, before the checkpoint superblock was durable, is repaired from the WAL.
- The planned Blink producer marks a transaction eligible only if it emits no superblock image (no split, allocation, page reuse, free-list or overflow allocator change). The WAL still decides per page: superblock, first touch since reset, missing base, or a delta not smaller than an image all stay full images.

### B0 — encoded size (deterministic probe, local)

`results/wal-v3-phase-b/b0-encoded-size-probe.jsonl`, 20,000 rows (1,279 leaves), 16-byte keys, 64-byte values:

| Case | tx | WAL B/tx mean (p50 / p95 / max) | delta frame B mean (p50 / p95 / max) | spans | changed B | fallback |
|---|---|---|---|---|---|---|
| existing-key width-1 update | 4,000 | 226.9 (227 / 228 / 228) | 158.9 (159 / 160 / 160) | 4.0 | 72.9 | 0 |
| first touch after checkpoint (1 in 40 keys) | 500 | 1,584.7 (225 / 4,224 / 4,224) | 157.1 | 4.0 | 71.1 | 34% first touch |
| delete existing key | 500 | 1,190.8 (1,165 / 2,056 / 3,586) | 1,122.8 | 15.2 | 992 | 0 |
| insert new key (no split) | 500 | 1,189.4 (1,165 / 2,056 / 3,511) | 1,121.4 | 15.2 | 990 | 0 |
| width-16 existing update | 500 | 2,596.1 (2,605 / 2,610 / 2,615) | 159.1 (max 309) | 4.0 | 73.1 | 0 |
| 16 tx on one leaf per group | 3,200 | 224.8 | 156.8 | 4.0 | 70.8 | 0 |
| 16 tx on different leaves per group | 3,200 | 226.4 | 158.4 | 4.0 | 72.4 | 0 |

Width-1 update: **4,224 → 227 B/tx** (PageImage frame 4,156 B → delta frame ~159 B; commit frame 68 B). The four spans are the page LSN, the page checksum, the entry revision and the value. Delete and insert are about 1.2 KB because the canonical leaf encoding moves every record after the changed slot. The gate (<512 B/tx, stop above 1 KiB) passed.

### Correctness

`cargo test --workspace --release --no-fail-fast`: 212 passed, 0 failed, 1 ignored (the B0 probe). New tests:

- codec: 3,000 random Blink leaf mutations (value same size, value resized, delete, insert, tombstone, revision only) and 2,000 random raw edits round-trip byte for byte, re-encode to the same delta, and the rebuilt Blink page validates; gap merge boundary; 18 malformed cases (bad base LSN, wrong page ID, zero-length, out-of-range, overlapping, unsorted, unmerged gap, truncated, trailing bytes, span count 0 / too high / too large, not starting on a changed byte, long unchanged gap, invalid rebuilt checksum).
- WAL: first redo after reset is an image then deltas, and again an image after the next reset; base mismatch (wrong bytes or stale LSN) is an invariant error and writes nothing; four transactions on two pages in one group chain in FIFO order (base LSNs checked in the frames); superblocks, ineligible commits and a Blink WAL still at version 2 get images; 11 malformed delta records written into an otherwise valid WAL (with the digest recomputed so only the delta check can fail) are rejected as corruption; bad digest and a delta in a version-2 WAL are rejected; omission, duplication, reordering, type substitution, payload corruption and a spliced commit are rejected.
- store: torn checkpoint (page set to the old checkpoint image, an intermediate committed image, the latest image, half old/half new, random bytes; another dirty page partially flushed) always recovers the latest committed page; delta-append fault matrix over 14 points (including during delta header, delta payload, after a delta record, before/after the commit record, before/during/after the WAL sync) at every occurrence, with reopen, atomicity check (a width-16 transaction is all-or-nothing and visible exactly when its commit frame was written), further writes and a second reopen; every byte cut of the WAL tail inside a width-16 delta transaction; checkpoint fault matrix over 18 points (page writes, data sync, superblock write and sync, WAL truncate, INIT rewrite, reset sync) with reopen, more writes, reopen, checkpoint, reopen; deltas after `flush()` and from the parallel executor.

The known timing-only flake `group_fast_path_is_byte_identical_to_fault_injectable_path` did not fire in these runs.

### First OCI gate

OCI A1 2 OCPU, ZFS `/bench/zfs/db`, same host as the baseline. Baseline `phase-a` = `12ab044` (SHA256 `f9d5199f…`, byte-identical to the Phase A build), candidate `page-delta` = `6bcf8bb` (SHA256 `0dc74cc6…`). `--engine planned-blink --sync-mode real`, working set 100,000, 2 s warmup, 5 s measure, baseline seeds, 3 repetitions with the variant order swapped every repetition. Every row was checked for `engine`, `sync_mode`, WAL syncs and planner counters; every page-delta row wrote deltas.

| Scenario | phase-a tx/s | page-delta tx/s | ratio | p99 µs | WAL B/tx | mean sync ms | tx/sync |
|---|---|---|---|---|---|---|---|
| 16w width 1 uniform | 4,597 | 8,298 | 1.805 | 6,077 → 3,343 | 4,224 → 227 | 1.42 → 0.74 | 7.9 → 8.3 |
| 16w width 16 uniform | 1,088 | 1,829 | 1.681 | 33,601 → 27,017 | 66,506 → 2,622 | 7.17 → 2.34 | 14.7 → 14.7 |
| 64w width 1 uniform | 12,655 | 24,111 | 1.905 | 9,752 → 5,154 | 4,224 → 227 | 3.03 → 0.89 | 59.8 → 58.8 |
| 64w width 16 uniform | 1,169 | 2,033 | 1.739 | 143,131 → 54,228 | 66,490 → 2,612 | 24.69 → 3.23 | 61.5 → 62.3 |

GM 1.781, far above the 1.10 stop line. Transactions per sync do not change; each sync gets much cheaper because it writes about 1/18 of the bytes, so the coordinator turns over faster. CPU per run went up (for example 22% → 32% at 16w width 1) because more transactions are processed. Peak RSS is about the same.

### Full matrix

14 scenarios × 3 repetitions × 2 variants, interleaved (84 rows). Per-scenario tables with p50/p95/p99, CPU, RSS, WAL bytes, image and delta records per transaction, span counts, fallback counts, tx/sync, sync count, sync latency and WAL MiB/s: `results/wal-v3-phase-b/tables.md`.

| # | Scenario | page-delta tx/s | phase-a tx/s | PD / phase-a | p99 µs phase-a → PD |
|---|---|---|---|---|---|
| 0 | 16w w1 uniform | 8,211 | 4,477 | 1.834 | 6,308 → 3,240 |
| 1 | 16w w1 compact | 9,474 | 4,900 | 1.933 | 6,315 → 2,827 |
| 2 | 16w w1 spread | 8,524 | 4,712 | 1.809 | 6,855 → 3,052 |
| 3 | 16w w16 uniform | 1,923 | 1,121 | 1.716 | 29,966 → 16,020 |
| 4 | 16w w16 compact | 6,016 | 3,318 | 1.813 | 7,897 → 4,059 |
| 5 | 16w w16 spread | 5,532 | 3,344 | 1.654 | 9,731 → 5,898 |
| 6 | 64w w1 uniform | 24,234 | 12,066 | 2.009 | 14,461 → 5,058 |
| 7 | 64w w1 compact | 32,897 | 11,426 | 2.879 | 9,673 → 3,772 |
| 8 | 64w w1 spread | 26,522 | 12,481 | 2.125 | 9,238 → 5,312 |
| 9 | 64w w16 uniform | 2,026 | 1,172 | 1.729 | 95,087 → 59,505 |
| 10 | 64w w16 compact | 10,768 | 5,779 | 1.863 | 21,861 → 9,797 |
| 11 | 64w w16 spread | 5,941 | 4,727 | 1.257 | 25,979 → 19,584 |
| 12 | 1w w1 uniform | 1,389 | 1,318 | 1.053 | 1,300 → 1,073 |
| 13 | 1w w16 uniform | 812 | 388 | 2.091 | 3,265 → 1,730 |

Multiwriter geometric means (12 scenarios):

| Category | PD / phase-a | PD / ExactMain | PD / RocksDB | phase-a / ExactMain | phase-a / RocksDB |
|---|---|---|---|---|---|
| overall | **1.853** | **3.605** | **0.704** | 1.945 | 0.380 |
| writers 16 | 1.791 | 2.843 | 0.713 | 1.587 | 0.398 |
| writers 64 | 1.918 | 4.572 | 0.695 | 2.384 | 0.362 |
| width 1 | 2.071 | 3.618 | 0.925 | 1.747 | 0.447 |
| width 16 | 1.659 | 3.593 | 0.536 | 2.166 | 0.323 |
| uniform | 1.818 | 3.854 | 0.481 | 2.120 | 0.265 |
| compact locality | 2.082 | 3.639 | 0.961 | 1.747 | 0.462 |
| spread locality | 1.681 | 3.342 | 0.753 | 1.988 | 0.448 |

ExactMain and RocksDB are the reused rows of the earlier runs (ExactMain: real-sync matrix; RocksDB v11.8.1: cross-DB matrix, 2026-09-25). They are **not interleaved** with this session. The phase-a row of this session gives 1.945× ExactMain against 1.861× in the baseline document, so this session ran about 4% faster for planned Blink than the day of the reused rows. Single-writer width 1 barely moves (1.053): with one writer every transaction waits for its own sync, and the sync itself is only a little cheaper for a 4 KiB write.

### RocksDB confirmation (same session)

Because the matrix GM gain is above 1.25×, RocksDB was re-run in this session on the four uniform scenarios, alternating order with page-delta, same binary (SHA256 `78f222f2…`), `write_options_sync=true`, WAL on, pipelined write off, verification passed on every row.

| Scenario | page-delta tx/s | RocksDB tx/s | RocksDB reused tx/s | PD / RocksDB | p99 µs PD / RocksDB |
|---|---|---|---|---|---|
| 16w w1 | 8,256 | 10,103 | 10,034 | 0.817 | 3,272 / 2,358 |
| 16w w16 | 1,923 | 6,147 | 5,723 | 0.313 | 16,147 / 5,135 |
| 64w w1 | 24,071 | 26,802 | 26,254 | 0.898 | 5,125 / 4,154 |
| 64w w16 | 2,054 | 9,852 | 9,573 | 0.208 | 50,106 / 14,375 |

The same-session RocksDB numbers are within 1–7% of the reused ones, so the reused 0.704× matrix GM stands. On these four scenarios PD / RocksDB is 0.468 (reused: 0.467). Page-delta closes most of the width-1 gap (0.82–0.90×) but width 16 uniform stays at 0.2–0.3×: 16 random leaves per transaction still cost 16 records, 16 leaf re-encodes and 16 page validations in the single coordinator, where RocksDB appends one small WriteBatch.

### 120-second sustained (no checkpoint)

Candidate `6bcf8bb`, working set 1,000,000, width 1, uniform, 10 s warmup, 120 s measure, real sync, same seeds and monitor as Phase A (`results/wal-v3-phase-b/sustained-tables.md`).

| Run | tx/s | p99 µs | WAL growth 120 s | RSS growth 120 s | peak RSS | Phase A tx/s / p99 / WAL growth |
|---|---|---|---|---|---|---|
| 16 writers | 8,704 | 3,380 | 227 MiB | 169 MiB | 1,374 MiB | 4,308 / 7,080 / 2,083 MiB |
| 64 writers | 20,440 | 5,993 | 533 MiB | 371 MiB | 1,694 MiB | 10,166 / 13,633 / 4,915 MiB |

About 2.0× Phase A throughput at both writer counts, with half the p99 and about 1/9 of the WAL growth. Every window wrote only deltas (delta ratio 1.000), dirty pages stayed at 68,448, mean WAL sync stayed at 0.75 ms (16w) and 0.91 ms (64w), and no window dropped. Seeding 1,000,000 rows now leaves a 3.6 GiB WAL instead of 4.6 GiB (seed transactions still write first-touch images). RSS grows by about 170 / 370 MiB, which is not tied to the WAL and has the same shape as in Phase A (the benchmark's 16-byte-per-transaction timeline is 17 / 39 MiB of it).

### Checkpoint control

Separate from the headline. Both variants get the same bench-only `--checkpoint-wal-bytes` patch: phase-a = `12ab044` + `scripts/checkpoint-control-on-12ab044.patch` (SHA256 of binary `fa8c8334…`), page-delta = `def6f7e` (`11953816…`). 16 writers, same sustained workload; a checkpoint right after seeding, then a synchronous checkpoint in the writer thread whenever the WAL reaches the threshold.

| Threshold | Variant | tx/s | p99 µs | checkpoints in 120 s | checkpoint duration | WAL reclaimed | WAL B/tx written |
|---|---|---|---|---|---|---|---|
| 1 GiB | phase-a | 4,091 | 6,936 | 2 | 2.43 / 2.69 s | 2,048 MiB | 4,224 |
| 1 GiB | page-delta | **8,707** (2.13×) | 3,636 | 0 | — | 0 | 355 |
| 256 MiB | phase-a | 3,835 | 7,781 | 7 | 1.40–1.59 s | 1,792 MiB | 4,224 |
| 256 MiB | page-delta | **5,600** (1.46×) | 6,713 | 5 | 2.17–2.36 s | 1,280 MiB | 1,777 |

Post-checkpoint full-image tax (page-delta, 1 GiB, after the post-seed checkpoint): image share of redo records per 10 s window 30.8% → 10.1% → 3.0% → 0.9% → 0.2% → 0.1% → 0; estimated WAL bytes per transaction 1,457 → 629 → 348 → 261 → 237 → 229 → 227. Throughput rises from 7,098 to about 9,300 tx/s as the image share falls. After about 40 s every page is covered again and the run is back at the no-checkpoint steady state.

With a 256 MiB threshold the WAL never reaches that steady state: covering 68k leaves once costs about 280 MiB of images, more than the threshold, so each checkpoint cycle is spent mostly on first-touch images (image share 18–77% per window, 1,777 B/tx overall). Page-delta still wins 1.46× because the delta part is cheap and it needs fewer checkpoints (5 vs 7), but each of its checkpoints is longer (about 2.2 s vs 1.5 s) because more transactions, and so more dirty pages, accumulate between them. Checkpoint windows show the stall in both variants (p99 up to about 8.9 ms). The PageDelta advantage survives periodic checkpoints, but how much of it survives depends on the WAL budget compared with the size of the dirty working set. A checkpoint policy should be sized to at least one full-image pass over the hot pages.

The window after a checkpoint can show its WAL drop one window early, because the resource sampler waits for the store lock that the checkpoint holds.

### Files

`results/wal-v3-phase-b/`: `format-specification.md`, `b0-encoded-size-probe.jsonl`, `byte-probe.txt`, `environment.txt`, `run-order.txt`, `process-metrics.jsonl`, progress logs, `tables.md`, `analysis.json`, `sustained-tables.md`, `raw/` (gate, matrix, confirmation, sustained and checkpoint rows, logs and monitors), `scripts/run_phase_b.py`, `scripts/analyze_phase_b.py`, `scripts/analyze_sustained.py`, `scripts/checkpoint-control-on-12ab044.patch`, `SHA256SUMS`.

## Phase C — width-16 bottleneck attribution

Question: why is width 1 close to RocksDB (0.92×) while width 16 uniform is still about 4.8× slower? Measurement only; no algorithm was changed. Instrumentation commit `cb7fb54` adds per-transaction locality histograms (mutations, dirty pages, dirty leaves, structural transactions) and a separate timer for WAL redo planning (page-delta diff and checks). Raw results, tables and perf data: `results/wal-v3-phase-c/`.

### Method

- OCI A1 2 OCPU, ZFS, same host. Binary `cb7fb54` (SHA256 `b6c74b1c…`), `--engine planned-blink`; every row was checked for `engine`, `sync_mode` and page deltas.
- Six scenarios: 16w/64w × width 1/16 uniform, plus 64w width 16 compact and spread. Working set 100,000, 2 s warmup, 5 s measure, 3 repetitions, baseline seeds.
- Each scenario ran with `--sync-mode real` and `--sync-mode disabled`, with the order swapped every repetition. **The `disabled` rows skip fsync. They separate CPU from storage and are not durable throughput.**
- Times below are the coordinator's `apply_transaction_group` time (`processing_nanos`), split by the existing Blink batch timers and WAL timers. "physical other" is physical execution minus mutation, restamp and page encode; "WAL frame encode" is WAL group encode minus redo planning; "unattributed" is what the timers do not cover.
- perf: a separate build of the same commit with frame pointers and line tables (`0db4bd76…`), `perf record -F 499 -g` on the running process for 15 s of the measurement, 64w width 1 and width 16 uniform (real sync).
- RocksDB v11.8.1 (same binary as before) on 64w width 1 and width 16 uniform, 3 repetitions, alternating with dodb.

### Throughput

| Scenario | real tx/s | no-sync tx/s | no-sync / real | real p99 µs | coordinator busy (real) | tx per sync |
|---|---|---|---|---|---|---|
| 16w w1 uniform | 8,156 | 33,413 | 4.10 | 3,249 | 97.6% | 8.1 |
| 16w w16 uniform | 1,913 | 2,606 | 1.36 | 16,251 | 92.9% | 14.5 |
| 64w w1 uniform | 23,545 | 37,350 | 1.59 | 5,564 | 93.5% | 60.6 |
| 64w w16 uniform | 2,063 | 2,270 | 1.10 | 58,977 | 92.3% | 61.5 |
| 64w w16 compact | 10,788 | 14,245 | 1.32 | 9,306 | 87.0% | 43.0 |
| 64w w16 spread | 5,860 | 7,904 | 1.35 | 20,250 | 84.1% | 61.3 |

The single coordinator is busy about 90% of the wall time in every case. At 64w width 16 uniform, removing fsync entirely gives only 1.10×: that scenario is CPU-bound, not storage-bound.

### Page locality per transaction

| Scenario | mutations | dirty pages mean / p50 / p95 / max | dirty leaves mean / p50 / p95 | mutations sharing a leaf | structural tx |
|---|---|---|---|---|---|
| 64w w16 uniform | 16 | 15.98 / 16 / 16 / 18 | 15.98 / 16 / 16 | 0.02 | 4 of ~31k |
| 16w w16 uniform | 16 | 15.98 / 16 / 16 / 18 | 15.98 / 16 / 16 | 0.02 | 6 |
| 64w w16 compact | 16 | 2.00 / 2 / 2 / 2 | 2.00 / 2 / 2 | 14 | 0 |
| 64w w16 spread | 16 | 2.00 / 2 / 2 / 2 | 2.00 / 2 / 2 | 14 | 0 |

A uniform width-16 transaction touches 16 different leaves: every mutation is on its own page. Compact and spread touch 2 leaves (8 mutations each). There are no overflow values; splits are rare (4–6 transactions per scenario).

### Coordinator time per transaction (real sync, ns)

| Component | 64w w1 | 64w w16 uniform | per mutation (w16) | w16 / w1 | share (w16) | 64w w16 compact |
|---|---|---|---|---|---|---|
| admission | 750 | 11,788 | 737 | 15.7× | 2.6% | 7,032 |
| planning | 3,259 | 82,974 | 5,186 | 25.5× | 18.6% | 19,941 |
| physical mutation | 3,388 | 57,976 | 3,623 | 17.1× | 13.0% | 12,081 |
| physical restamp | 150 | 2,963 | 185 | 19.7× | 0.7% | 919 |
| page encode | 2,281 | 38,067 | 2,379 | 16.7× | 8.5% | 2,894 |
| physical other | 326 | 1,345 | 84 | 4.1× | 0.3% | 333 |
| dirty union | 89 | 837 | 52 | 9.4× | 0.2% | 98 |
| catalog construction | 1,967 | 21,951 | 1,372 | 11.2× | 4.9% | 162 |
| WAL assembly | 1,331 | 27,629 | 1,727 | 20.8× | 6.2% | 1,157 |
| WAL redo plan (delta encode) | 3,773 | 56,341 | 3,521 | 14.9× | 12.6% | 5,798 |
| WAL frame encode | 890 | 9,972 | 623 | 11.2× | 2.2% | 1,376 |
| WAL write | 837 | 3,538 | 221 | 4.2× | 0.8% | 1,321 |
| WAL sync | 15,136 | 51,150 | 3,197 | 3.4× | 11.4% | 22,810 |
| state install | 1,218 | 17,718 | 1,107 | 14.5× | 4.0% | 98 |
| generation publication | 1,536 | 18,569 | 1,161 | 12.1× | 4.2% | 264 |
| dirty tracking | 839 | 16,535 | 1,033 | 19.7× | 3.7% | 83 |
| unattributed | 1,955 | 27,878 | 1,742 | 14.3× | 6.2% | 4,294 |
| **total** | **39,727** | **447,232** | **27,952** | **11.3×** | 100% | **80,661** |

Per-mutation, per-scenario and no-sync versions, plus detail timers (planner route, leaf clone, catalog scan/clone, retired-generation drop), are in `results/wal-v3-phase-c/tables.md`. Work counts per transaction at 64w width 16 uniform: 16 mutations, 15.98 page encodes (0.999 per mutation), 15.98 page deltas, 0.002 page images, 1,703 delta payload bytes (106 per mutation), 63.9 spans (4.0 per mutation), 2,612 WAL bytes.

What scales with what:

- Every per-page stage grows 15–21× from width 1 to width 16 (physical mutation 17×, page encode 17×, delta encode 15×, WAL assembly 21×, state install 15×, dirty tracking 20×, catalog 11×, publication 12×) — in line with 16× dirty leaves.
- Planning grows 25.5× at 64 writers (13.7× at 16): per mutation it costs 5.2 µs in 64-transaction groups against 3.4 µs in 16-transaction groups, so it grows faster than linearly with group size.
- Only WAL sync (3.4×) and WAL write (4.2×) stay mostly per group.
- **Cost follows touched leaves, not mutations.** With sync disabled, coordinator time per touched leaf is 23.9 µs at 64w width 1, 25.0 µs at 64w width 16 uniform and 28.9 µs at 64w width 16 compact (8 mutations per leaf). Compact width 16 is 5.2× faster than uniform width 16 because it touches 8× fewer leaves.

### Real sync against sync disabled

| Scenario | real ns/tx | no-sync ns/tx | WAL sync ns/tx | sync share |
|---|---|---|---|---|
| 16w w1 | 119,675 | 26,800 | 89,986 | 75.2% |
| 64w w1 | 39,727 | 23,928 | 15,136 | 38.1% |
| 16w w16 | 485,509 | 348,307 | 133,980 | 27.6% |
| 64w w16 uniform | 447,232 | 399,921 | 51,150 | 11.4% |
| 64w w16 compact | 80,661 | 57,782 | 22,810 | 28.3% |

Width 1 is mostly waiting for the sync (38–75%); width 16 uniform at 64 writers is 89% CPU.

### perf (64w width 16 uniform, real sync, frame-pointer build, 1,910 tx/s)

Self time, top entries: `memcpy` 14.1%, `plan_batch` 11.4%, `_int_malloc` 7.1%, Arc refcount atomics (`ldadd8`) 10.0%, `memcmp` 5.5%, `free` 4.7%, `malloc_consolidate` 3.7%, CRC32C 4.3%, `DocumentKey::validate_encoded` 3.5%, `encode_page_delta` 3.4%, `prepare_planned_serial_execution` 3.2%, `_int_free` 2.8%, `memmove` 2.6%. Memory copy, allocation and refcounting together are about 45% of samples.

Inclusive: `apply_transaction_group` 88%, `prepare_planned_serial_execution` 22.4%, `plan_batch` 18.4%, WAL `append_group_inner` 16.5%, `BTreeMap<PageId, [u8; 4096]>::insert` (dirty-page copies) 6.9%, `encode_blink_page` 6.5%, `leaf_body_layout` 5.1%, `ensure_sorted_leaf` 4.8%, `apply_cached_leaf_mutation` 4.7%, `prepare_delta` (catalog) 4.6%, `BlinkPage` / `Vec<LeafEntry>` clones about 4% (their Arc refcount increments are most of the `ldadd8_relax` samples), dropping the retired generation (`PublishedGeneration`/`PageCatalog`/`PageCell` drop) about 4%.

The 64w width 1 profile has the same functions, with a larger share for publication drop (6.7%) and WAL append (24.7%), and a smaller share for `plan_batch` (10.3%). Of the areas the plan asked about: leaf lookup (`memcmp`, `ensure_sorted_leaf`), allocation/free, page encoding, PageDelta diff, CRC32C, catalog/generation publication and `Arc`/`BTreeMap` cloning are all visible; none is above 15% on its own. About 2.6–4.4% of samples are `Vec<TransactionMutation>` growth in the benchmark coordinator closure, outside the timed `apply_transaction_group`.

Reports: `results/wal-v3-phase-c/perf/*.report-{top,inclusive,dso,hot-callers,callers,children,flat}.txt`, `*.inclusive-summary.md` (roughly demangled), and the `perf.data` files.

### RocksDB same session

| Scenario | dodb tx/s | RocksDB tx/s | dodb / RocksDB | p99 µs dodb / RocksDB | CPU % (one core) dodb / RocksDB |
|---|---|---|---|---|---|
| 64w w1 uniform | 23,986 | 26,208 | 0.915 | 5,148 / 3,927 | 68 / 64 |
| 64w w16 uniform | 2,060 | 9,805 | 0.210 | 57,860 / 14,893 | 92 / 85 |

RocksDB matches the Phase B session (26,802 / 9,852 tx/s), so storage did not drift. At width 16 RocksDB spends about 0.85 core for 9,805 tx/s (roughly 87 µs of CPU per transaction). dodb spends about 400 µs of coordinator CPU per transaction, all in one thread, which caps it near 2,300 tx/s even without fsync.

### Attribution of the remaining RocksDB gap (64w width 16 uniform)

| Plan category | Components | Share of coordinator time |
|---|---|---|
| A. physical execution per leaf | mutation, restamp, page encode, other | 22.5% |
| B. planner | planning + admission | 21.2% |
| D. WAL CPU | redo plan (delta encode) 12.6%, assembly 6.2%, frame encode 2.2%, write 0.8% | 21.8% |
| C. catalog / publication / install | catalog 4.9%, publication 4.2%, state install 4.0%, dirty tracking 3.7%, dirty union 0.2% | 17.0% |
| E. durability sync | WAL sync | 11.4% |
| — | unattributed | 6.2% |

There is no single dominant stage. The gap comes from about 25 µs of serial CPU per touched leaf, spread over the whole pipeline, times 16 leaves per transaction. Sync is 11%. Even infinitely fast storage would give only 1.10×.

### Recommendation

Primary direction: **A — transaction-internal parallel per-leaf execution.** Extend the existing parallel executor, which today falls back on multi-leaf transactions (`parallel_fallback_multi_leaf`), so that the independent per-leaf work of one non-structural transaction runs on workers: leaf mutation, restamp, page encode, and the page-delta diff for that leaf. That covers about 22.5% + 12.6% + part of 8.4% ≈ 40% of coordinator time, all of it independent across the 16 leaves. The coordinator then keeps planning, WAL framing and sync, and publication.

Order of what follows, from the same data: B planner (21%, growing faster than linearly with group size) second, C catalog/publication (17%) third, E sync (11%) last. D (delta encode, 12.6%) should move into the per-leaf workers as part of A rather than be tuned on its own. Two caveats for A on this host:
- Two OCPUs cap it near 2× on the parallel part.
- The profile shows about 45% of cycles in memcpy, malloc/free and Arc refcounting spread across all stages, so per-leaf copy and allocation cost is the other lever if parallel speedup falls short.

### Files

`results/wal-v3-phase-c/`: `environment.txt`, `run-order.txt`, `process-metrics.jsonl`, progress logs, `raw/` (36 attribution rows, 6 dodb and 6 RocksDB confirmation rows, 2 perf rows, logs), `tables.md`, `analysis.json`, `perf/`, `scripts/run_phase_c.py`, `scripts/analyze_phase_c.py`, `scripts/summarize_perf.py`, `SHA256SUMS`.

## Phase D — leaf-partitioned parallel physical execution

Question: if the per-leaf work of a WAL group (leaf mutation, page encode, PageDelta diff) runs on 2 lanes instead of the single coordinator, how much does durable width-16 throughput go up on 2 OCPU? Answer: **64w width 16 uniform 1.190× Phase C with real sync** (1.230× with sync disabled). That is just under the 1.20× bar, and 16w width 1 lost 3%, so the full matrix and the RocksDB rerun were skipped as the plan says.

Code (branch `experiment/wal-v3-compact-redo`, not merged):

| Commit | Content |
|---|---|
| `a917c84` blink: execute independent leaf chains in parallel | leaf-chain jobs, fallback rules, `WalLog::append_group_prepared`, tests (iteration D1) |
| `9932f93` bench: measure parallel leaf execution | `--parallel-workers` for `planned-blink` (0 = serial executor, default), new counters |
| `df8b8f4` blink: run leaf jobs on the coordinator lane and clone leaves in workers | iteration D2, the measured version |

Raw results, tables and perf data: `results/wal-v3-phase-d/`. Full design: `results/wal-v3-phase-d/design.md`. Correctness details: `results/wal-v3-phase-d/correctness.md`.

### Design

- The coordinator keeps logical admission and `plan_batch` unchanged. It then partitions the **whole WAL group** by target leaf, not one transaction at a time: leaf L gets the mutations of every transaction that routes to L, in transaction FIFO order (A → B → C on the same leaf, independent leaves in parallel).
- **Commit LSNs are fixed before any lane starts.** For an eligible transaction the redo record count is the number of distinct leaves it touches, known from the planner's routes, so `commit_lsn = next_lsn + distinct_leaves` — the same formula as the serial executor with no superblock image. There is no circular dependency. After the lanes finish, the coordinator checks that every transaction got exactly one boundary per leaf, each with its precomputed commit LSN.
- Each job carries the committed leaf (`Arc<BlinkPage>` from the published generation), the leaf's WAL page-chain entry, the current dirty image as delta base, and shared `Arc`s to the plan and the commit-LSN array. The lane clones the leaf once, applies the mutations in place per transaction (revision = page LSN = commit LSN), encodes the page, computes the image CRC, and produces a full image (first touch after a WAL reset, or delta not smaller) or the canonical PageDelta against the previous committed image of that leaf (pre-group image first, then the previous transaction's image). The delta is applied back and must rebuild the image, the check the WAL did in Phase B.
- `parallel_workers = n` means n lanes: the coordinator plus n − 1 persistent threads draining one job queue in chunks. The primary configuration is 2 lanes.
- The coordinator puts the final pages into the working overlay (move), builds each transaction's record list in page-ID order, and calls `WalLog::append_group_prepared`, which re-checks batch IDs, commit LSNs, page LSNs, record order, and the page-chain rule (each delta's base LSN and base CRC must be the latest committed image of that page) before writing a byte. One write, one sync. Lanes never touch the WAL. Publication, state install and dirty tracking are unchanged; the dirty map gets only each leaf's final image.
- **Eligibility is all-or-nothing per group:** at least 2 target leaves, a page-delta WAL, all put values inline, every target a leaf, every key below its leaf's high key, no mutated key holding an overflow value, and every leaf still fits after every mutation. So no split, root/internal change, allocator, free-list, overflow or page-reuse change, and no superblock image. Anything else runs the unchanged serial executor with the same plan. Every fallback is decided before the WAL append; a lane error or panic becomes a group error with nothing written.

Iteration D1 (`a917c84`) had the coordinator clone each leaf while building jobs, split jobs statically over 2 worker threads, and only wait. Its sync-disabled gate gave 1.104× at 64w width 16 uniform: job building cost 48 µs/tx on the coordinator, and the two threads plus the waiting coordinator on 2 OCPU reached only 1.49 effective parallelism. D2 moved the clone to the lanes and made the coordinator a lane. D1 raw data: `results/wal-v3-phase-d/iteration-d1/`.

### Correctness

`cargo test --workspace --release` and `cargo test -p dodb-storage` (debug, which adds full page validation on the prepared WAL path and checks each published leaf against the committed state): all pass (storage lib 142 passed, 1 ignored). New tests:

- one transaction on 16 distinct existing leaves: same results, pages, dirty images, counters and **byte-identical data and WAL files** as serial, for 1 and 2 lanes;
- same-leaf chain A → B → C: FIFO values and revisions, three PageDeltas on that leaf with increasing commit LSNs, WAL identical to serial;
- mixed A → L1 L2, B → L2 L3, C → L4: byte-identical to serial;
- lane error and lane panic inside a 16-leaf transaction: group error, nothing written, no visible change, store not broken, next group succeeds, reopen correct;
- 40 random groups: WAL identical, reopen without checkpoint gives the same documents as serial, further groups stay identical;
- fault matrix over 17 points (`before_parallel_leaf_dispatch`, `after_parallel_leaf_join`, every WAL write point, before/during/after sync, `before_generation_publication`) at every occurrence, 212 injected failures: nothing acknowledged or visible for the failed group, each transaction all-or-nothing after reopen, no later transaction without the earlier one, both present once both commit frames were written;
- WAL prepared path rejects a delta with the wrong base LSN or CRC and writes nothing.

Differential: three seeds × 80 random groups (width 1–16, inserts, deletes, overflow values in every 8th group, splits, conflicts, rejected requests, a `flush()` and a checkpoint), run on the serial executor, 1 lane and 2 lanes. After every group: identical results and commit LSNs (or the same group error), pages, dirty images, superblock and counters. At the end: identical scans, invariants, data file, **byte-identical WAL**, identical reopen. 47–57 of 80 groups ran in parallel per seed; 21–33 fell back (overflow or structural). The WAL is byte-identical because the lane makes the same image-or-delta choice the WAL made in Phase B and records are in page order.

### Sync-disabled CPU gate (OCI A1 2 OCPU, ZFS, D2)

`phase-c` = binary `cb7fb54` (SHA256 `b6c74b1c…`), Phase D binary `df8b8f4` (SHA256 `fbb70d7c…`) with `--parallel-workers 0 / 1 / 2`. 3 repetitions, order rotated. Not durable throughput.

| 64w width 16 | Phase C | Phase D serial | 1 lane | 2 lanes | 2 lanes / Phase C | 2 lanes / serial (same binary) | D1 2 threads / Phase C |
|---|---|---|---|---|---|---|---|
| uniform | 2,249 | 2,236 | 2,313 | 2,765 | **1.230** | 1.237 | 1.104 |
| compact | 14,145 | 14,350 | 15,019 | 15,765 | 1.115 | 1.099 | 1.101 |
| spread | 7,915 | 7,895 | 8,087 | 8,664 | 1.095 | 1.097 | 1.057 |

The serial executor in the Phase D binary matches Phase C (0.99–1.01). One lane (all leaf work on the coordinator, new path) is 1.02–1.06× from doing less copying (in-place mutation, no image copies before the WAL); the second lane adds the rest.

### Real-sync durable gate (ZFS)

`--engine planned-blink --sync-mode real`, 100,000 rows, 2 s warmup, 5 s measure, 3 repetitions, order rotated. Every row checked for engine, sync mode, WAL syncs, page deltas, `parallel_workers` and parallel groups.

| Scenario | Phase C tx/s | Phase D 2 lanes tx/s | ratio | p99 µs Phase C → D | parallel groups | leaf jobs / group | operations / job |
|---|---|---|---|---|---|---|---|
| 16w w1 uniform | 7,830 | 7,593 | **0.970** | 3,496 → 3,754 | 94.7% | 8.5 | 1.00 |
| 16w w16 uniform | 1,933 | 2,177 | 1.126 | 15,874 → 14,282 | 99.7% | 223 | 1.02 |
| 64w w1 uniform | 23,275 | 25,650 | 1.102 | 5,374 → 5,021 | 99.2% | 61 | 1.01 |
| 64w w16 uniform | 2,031 | 2,417 | **1.190** | 58,693 → 48,987 | 99.3% | 905 | 1.08 |
| 64w w16 compact | 10,673 | 11,788 | 1.105 | 9,412 → 8,774 | 100% | 4.8 | 17.5 |
| 64w w16 spread | 5,996 | 6,376 | 1.063 | 19,754 → 19,068 | 100% | 118 | 1.00 |

GM over the six: 1.091. Fallbacks: 7 groups at 16w width 16 and 4 at 64w width 16 (all structural: a leaf split, found by a lane after dispatch); single-leaf groups (serial) are 5.3% of groups at 16w width 1, 0.8% at 64w width 1. WAL bytes per transaction and transactions per sync are unchanged (for example 2,612 → 2,611 B/tx and 61.2 → 61.0 tx/sync at 64w width 16 uniform).

**Width 1 control.** 64w width 1 gains 10%. 16w width 1 loses 3% and the repetitions do not overlap (7,746–7,927 against 7,526–7,644). Coordinator time outside the sync is the same (29.8 vs 30.1 µs/tx); the difference is WAL sync time per transaction (+3.7%, mean sync 0.78 → 0.80 ms) at the same 8 transactions per sync. With 8-transaction groups the parallel section is short (effective parallelism 1.31) and the extra thread takes CPU on a host whose ZFS sync also needs CPU; that last point is a likely explanation, not measured.

### Where the time went (64w width 16 uniform, real sync, coordinator ns per transaction)

| Component | Phase C | Phase D | Note |
|---|---|---|---|
| admission + planning | 94,622 | 97,979 | unchanged work |
| leaf physical on the coordinator (serial mutation, restamp, encode, other) | 100,488 | 1,884 | moved to lanes |
| WAL assembly + redo plan (Phase C: delta encode) | 84,562 | 4,751 | delta encode moved to lanes; WAL now only checks the chain |
| job building (dispatch) | — | 27,788 | leaf lookups, base-image copy, chain lookups |
| leaf jobs run by the coordinator lane | — | 64,087 | half the leaf work |
| waiting for the worker thread + result collection | — | 7,769 | |
| WAL frame encode + write | 13,746 | 12,025 | |
| catalog / publication / state install / dirty tracking / union | 76,383 | 76,209 | unchanged |
| WAL sync | 55,195 | 55,915 | |
| unattributed | 27,216 | 28,259 | |
| **total** | **452,210** | **376,666** | 1.20× |

Lane side per transaction: base image + chain check 27.0 µs, mutation 23.6, page encode 34.1, delta encode + verify + CRC 36.3; all lanes busy 129.4 µs (coordinator lane 64.1, worker thread ~65.3), idle inside the parallel section 5.1 µs; effective parallelism 1.92. In the perf profile the worker thread has 14.6% of all samples.

**Amdahl.** The leaf-local stage (Phase C: physical + WAL assembly + delta encode = 185 µs/tx) now costs the coordinator 106 µs/tx (dispatch 28 + its own lane 64 + wait/collect 8 + leftovers 6): **1.74× faster**, close to the 2-lane limit once job building is counted. It was 41% of Phase C coordinator time, so the whole coordinator gets 1.20× and throughput 1.19×. The other 59% (planning 21%, catalog/publication/install 17%, sync 12%, WAL framing 3%, unattributed 6%) did not change and now caps the gain: even a free leaf stage would give at most 452 / 267 ≈ 1.69×. As shares of the Phase D coordinator time: planner 26.0%, catalog/publication 20.2%, sync 14.8%, leaf work on the coordinator lane 17.0%, dispatch/wait/collect 9.4%, WAL CPU 4.5%.

### Copy / allocation / refcount (perf, 64w width 16 uniform, real sync)

Frame-pointer builds of both binaries (Phase C `0db4bd76…`, Phase D `990ba79a…`), `perf record -F 499 -g` for 15 s of a 20 s measurement, same session. Self-time symbols grouped the same way for both (`results/wal-v3-phase-d/perf/copy-alloc-refcount.txt`); this grouping also counts `memmove`, `realloc` and the allocator's own lock atomics, so it is wider than the "≈45%" in Phase C.

| Category | Phase C share | Phase C cycles/tx | Phase D share | Phase D cycles/tx |
|---|---|---|---|---|
| memcpy / memmove | 15.8% | 215,949 | 14.4% | 186,938 |
| malloc / free (incl. allocator atomics) | 25.2% | 344,536 | 26.2% | 339,841 |
| Arc refcount atomics (`ldadd8`) | 10.1% | 137,596 | 10.7% | 138,742 |
| memcmp | 5.3% | 73,030 | 5.0% | 65,214 |
| **total** | **56.5%** | **771,111** | **56.2%** | **730,735** |
| all cycles per transaction | | 1,365,040 | | 1,299,085 |

The parallel path did not add copy or allocation work: per transaction it is 5% lower, and the share is unchanged. It still is more than half of all cycles. Where it comes from in Phase D (callers): `LeafEntry` vector clone and drop (the lane's leaf clone, the catalog's page clone, dropping retired generations' pages) for most of the `Arc` atomics; the 4 KiB dirty base image copied into each job (3.1% of samples) and `BTreeMap<PageId, [u8; 4096]>::insert` in dirty tracking (2.3%); `PhysicalTransactionPlan` / `TransactionRequest` drops and `TransactionMutation` vector growth (the latter in the benchmark's group collection, 2–4%); `plan_batch` itself is 11.7% self time.

### Decision

Phase D gives 1.19× at 64w width 16 uniform — below 1.20× — and a 3% width-1 loss at 16 writers. **Parallel execution is not the main answer on 2 OCPU.** The leaf stage itself runs 1.74× faster, but it was only 41% of the coordinator, and more than half of all cycles are still copying, allocating and refcounting.

Recommended next direction: **copy / allocation / refcount elimination**, measured per source before changing anything:

- `Arc` churn from cloning and dropping `Vec<LeafEntry>` (lane leaf clone, catalog `prepare_delta` clone, retired-generation drop);
- 4 KiB image copies: the dirty base image copied into each job and the `BTreeMap<PageId, [u8; 4096]>` dirty map insertion;
- temporary allocations in the planner (`PlannedMutation` copies of mutations and keys, per-group `BTreeMap`/`BTreeSet`s) and the benchmark's mutation vectors;
- WAL assembly copies are already gone on the parallel path (28 → 1 µs/tx) and need no further work.

The leaf-parallel code stays behind `--parallel-workers` (off by default). Whether to keep it on for width 16 and off for small width-1 groups is a later tuning question, not part of this phase.

### Files

`results/wal-v3-phase-d/`: `design.md`, `correctness.md`, `environment.txt`, `run-order.txt`, `process-metrics.jsonl`, progress logs, `raw/` (36 sync-disabled gate rows, 36 real-sync gate rows, 2 perf rows, logs), `tables.md`, `analysis.json`, `perf/` (`perf.data`, flat / children / dso / comm reports, `copy-alloc-refcount.txt`), `iteration-d1/` (the D1 sync-disabled gate), `scripts/run_phase_d.py`, `scripts/analyze_phase_d.py`, `scripts/perf_categories.py`, `SHA256SUMS`.

## Phase E — copy/allocation/refcount elimination

Question: with the algorithm, the durability rules, the page format and the WAL format unchanged, and the Phase D parallel executor kept (2 lanes), how much of the width-16 CPU cost goes away if unnecessary copies, temporary allocations and reference-count churn are removed? Answer: **64w width 16 uniform 1.423× Phase D with real sync** (strong success by the plan's bar), 1.417× with sync disabled, and 22% fewer cycles per transaction. But 16w width 1 is still 0.972× Phase C, below the 0.98 bar, so the full matrix and the RocksDB rerun were skipped as the plan says.

Code (branch `experiment/wal-v3-compact-redo`, not merged). Each step has its own commit and its own OCI run:

| Commit | Step |
|---|---|
| `b783686` blink: count ownership churn behind the churn-counters feature | E0 counters (off in timed builds) |
| `ef99cdb` blink: move leaf page images instead of copying them | E1 |
| `df2ce35` blink: share committed pages between the state and the catalog | E2a (+ pinned-generation test) |
| `db6499f` blink: keep short leaf keys inside the entry | E2b, measured, then reverted |
| `b445043` blink: cut planner and admission temporaries | E3 |
| `4ead3de` Revert "blink: keep short leaf keys inside the entry" | final Phase E = E1 + E2a + E3 |

Raw results, counters, perf and scripts: `results/wal-v3-phase-e/`. Design details: `results/wal-v3-phase-e/design.md`. Correctness: `results/wal-v3-phase-e/correctness.md`.

### E0 — where the churn came from (Phase D, 64w width 16 uniform)

Counters (feature `churn-counters`: per-thread stage tags, a counting global allocator in the benchmark, and counters at every clone / drop / 4 KiB copy site; sync disabled, 2 runs, per committed transaction) and cycles (perf of the Phase D build, real sync):

| Source | Counters per transaction | Share of all cycles |
|---|---|---|
| **1. Whole-leaf clone and drop, with per-entry Arc counts** | 29.8 leaf clones (2 per touched leaf: lane and catalog), 452 entry clones, 468 entry drops, ≈ 1,840 atomic count changes on key/value payloads | Arc atomics 10%, 70% of them in `Vec<LeafEntry>` clone/drop and `BlinkPage` drop; plus the clone/drop allocations |
| **2. Planner and admission temporaries** | 162 planner and 86 admission allocations, 1,283 planner `BTreeSet`/`BTreeMap` inserts (1,219 of them building the "dependent leaf groups" metric), 3 key copies and 1 full `TransactionMutation` clone per mutation | `plan_batch` self 10–12%, planner malloc ≈ 2–3%, dropping the plan ≈ 2% |
| **3. 4 KiB page-image copies** | 62 copies = 255 KB per transaction: dispatch base copy, delta-check rebuild, chain-base copy, dirty-map insert | dispatch copy 3.1%, lane copies 2.3%, `BTreeMap<PageId, [u8; 4096]>::insert` 2.3%, delta rebuild 0.5% |

The benchmark harness itself (request generation and its per-group `request.clone()`) makes 166 of the 643 allocations per transaction and about 8–10% of cycles in malloc/free; it is outside the store and was not changed. WAL assembly copies were already gone in Phase D. Per-site tables: `results/wal-v3-phase-e/counters/`.

### Steps

Each step ran against Phase D (and the previous step) in the same session, interleaved, 3 runs each: 64w width 16 uniform with sync disabled and with real sync, and 64w width 1 uniform with real sync. Counters: 64w width 16 uniform, sync disabled, per transaction.

| Step | What changed | Counter change | disabled w16 / D | real w16 / D | real w1 / D |
|---|---|---|---|---|---|
| E1 | dirty map holds `Arc<[u8; 4096]>`; job borrows its base; lane encodes into a new heap image and moves it along the chain and into the dirty map; delta check in place | 4 KiB copies 62 → 0.3, bytes 255 KB → 1.3 KB | 1.072 | 1.079 | 1.021 |
| E2a | one `Arc<BlinkPage>` shared by committed state and catalog | leaf clones 29.8 → 15.0; entry clones 452 → 227; entry drops 468 → 243 | 1.166 | 1.143 | 1.077 |
| E2b (reverted) | keys ≤ 62 bytes inline in the entry | key Arc clones/drops 227/243 → 0 | 1.171 | 1.205 | 1.096 |
| E3 (on E2b) | planner/admission temporaries | planner allocs 162 → 103, admission 86 → 70, map inserts 1,283 → 16, mutation clones 16 → 0 | 1.299 | 1.488 | 1.096 |
| final (E3 without E2b) | | allocations 477 → 360 per transaction (harness excluded) | **1.363** | **1.423** | 1.063 |

Ratios are from each step's own session (Phase D varied 2,358–2,468 tx/s with real sync between sessions). Step effects in the same session: E2a over E1 1.066 / 1.059 / 1.077; E3 over E2b 1.093 / 1.216 / 1.002.

**E1.** Same WAL records, same dirty images; the delta check now compares base, payload spans and after-image in place (same validation function as `apply_page_delta`, randomized equivalence test). Dispatch fell from 44 to 26 µs/tx; lane time did not change, because the base image and the new image are now read from and written to heap memory that is cold in the lane instead of a hot stack copy.

**E2.** Phase D cloned each touched leaf twice (lane; catalog `prepare_delta`) and dropped it twice (state install; retired generation), each time changing the count of every key and inline value. With one shared page object the lane's copy-on-write clone is the only clone and the retiring generation's drop the only drop. Committed and published pages are never written in place (`Arc::make_mut` copies a shared page); the new pinned-generation test checks page objects, entry payload pointers, bytes, images and scans across later parallel and serial writes, splits and a checkpoint.

The remaining per-entry counts (one clone and one drop per unchanged entry of each touched leaf) are what E2b tried to remove, by storing short keys inside the entry. Key counts went to zero, but throughput did not change (GM 1.156 vs 1.151 for E2a) and lane time rose 137 → 156 µs/tx: 96-byte entries made each lane clone and page encode read and copy twice the memory. That is the "move the cost elsewhere" case the plan warns against, so E2b was reverted. Removing the value counts too would need entries without a reference each (a per-leaf byte arena), which changes every entry accessor; not attempted.

**E3.** The biggest single item turned out to be the planner's "independent leaf groups" metric: for each dependency edge it inserted every (predecessor group, successor group) pair into a `BTreeSet`, about 1,200 inserts per width-16 transaction. It is now a `bool` vector (same count). The last-writer maps borrow the admitted keys, the planned mutation carries its value once as `Arc<[u8]>` that the lane shares as the entry value, the admission overlay stops copying values it never reads, and `DocumentKey::encode` reserves its exact escaped length (16 reallocations per transaction were zero-byte escapes). Planner + admission time fell from about 97 to 49 µs/tx at width 16.

### Phase D vs Phase E

CPU gate (sync disabled, 3 runs, same session):

| 64w width 16 | Phase D tx/s | Phase E tx/s | E / D |
|---|---|---|---|
| uniform | 2,696 | 3,821 | **1.417** |
| compact | 15,836 | 18,416 | 1.163 |
| spread | 8,474 | 10,485 | 1.237 |

Coordinator time per transaction, 64w width 16 uniform, sync disabled:

| µs / tx | Phase D | Phase E |
|---|---|---|
| **coordinator total** | **331.8** | **224.2** |
| planner (admission + planning) | 99.6 | 48.4 |
| leaf jobs run on the coordinator lane | 69.4 | 71.6 |
| dispatch + wait + collect | 42.4 | 28.8 |
| catalog / publication / install / dirty tracking | 76.3 | 46.3 |
| WAL CPU | 18.0 | 17.7 |
| unattributed | 26.1 | 11.4 |
| all lanes busy (coordinator lane + worker thread) | 138.8 | 147.6 |

Perf, same classification as Phase D (`scripts/perf_categories.py`, self time), real sync, 64w width 16 uniform, frame-pointer builds with the repository's `+crc` target feature, same session (Phase D 2,279 tx/s, Phase E 3,032 tx/s under perf):

| Category | Phase D share | Phase D cycles/tx | Phase E share | Phase E cycles/tx |
|---|---|---|---|---|
| memcpy / memmove | 14.5% | 188,999 | 5.3% | 54,418 |
| malloc / free (incl. allocator atomics) | 26.8% | 350,570 | 27.2% | 278,227 |
| Arc refcount atomics (`ldadd8`) | 10.0% | 130,615 | 17.1% | 174,608 |
| memcmp | 5.1% | 67,136 | 6.6% | 67,920 |
| **copy / allocation / refcount** | **56.5%** | **737,320** | **56.2%** | **575,172** |
| CRC32C | 4.4% | 57,470 | 6.6% | 67,715 |
| planner (`plan_batch` self) | 11.5% | 150,599 | 3.7% | 37,438 |
| PageDelta | 4.0% | 52,376 | 4.7% | 48,383 |
| **all cycles per transaction** | | **1,305,974** | | **1,022,892** |

Copy/allocation/refcount cycles per transaction fell 22% (737k → 575k) and all cycles 22% (1.31M → 1.02M); the share stayed at 56% because the rest shrank too (mostly the planner). About 100k cycles/tx of malloc/free in both columns are the benchmark harness.

The Arc atomics went **up** per transaction although the number of count changes halved. By caller: `Vec<LeafEntry>` clone 63k cycles/tx in both (Phase D: two clones, Phase E: one), and the page drop 39k → 57k. Phase D's second clone (catalog) and first drop (install) ran on the coordinator right after the lane had touched the same payloads, so their cache lines were already there; the ones left in Phase E are the lane clone and the retired-generation drop, each of which pulls the payload's cache line from another core or from memory. So these atomics are cache-miss bound: about 120k cycles/tx (≈ 12% of all cycles) are spent changing the counts of entries that did not change.

A first perf pair was built with `RUSTFLAGS=-C force-frame-pointers=yes`, which silently replaces the repository's `+crc` flag; CRC32C then took 12–15% of cycles in both builds. Those files are kept (`*-fp-*`, `copy-alloc-refcount.txt`) but the table above uses the `+crc` pair (`*-fpcrc-*`, `copy-alloc-refcount-crc-enabled.txt`).

### Durable gate (OCI A1 2 OCPU, ZFS, real sync)

`--engine planned-blink --sync-mode real`, 100,000 rows, 2 s warmup, 5 s measure, 3 runs, Phase C (`b6c74b1c…`), Phase D (`fbb70d7c…`) and Phase E (`6670b610…`) interleaved in one session; every row checked for engine, sync mode, WAL syncs, page deltas, `parallel_workers` and parallel groups.

| Scenario | Phase C tx/s | Phase D tx/s | Phase E tx/s | **E / D** | E / C | p99 µs C → D → E |
|---|---|---|---|---|---|---|
| 16w w1 uniform | 8,342 | 7,976 | 8,112 | 1.017 | **0.972** | 3,410 → 3,278 → 3,387 |
| 16w w16 uniform | 1,947 | 2,293 | 2,666 | 1.163 | 1.369 | 15,920 → 13,350 → 11,917 |
| 64w w1 uniform | 23,906 | 26,272 | 28,901 | 1.100 | 1.209 | 5,576 → 4,914 → 4,728 |
| 64w w16 uniform | 2,058 | 2,423 | 3,449 | **1.423** | 1.676 | 57,027 → 48,028 → 38,098 |
| 64w w16 compact | 10,762 | 11,935 | 13,232 | 1.109 | 1.230 | 9,811 → 8,847 → 8,174 |
| 64w w16 spread | 5,842 | 6,427 | 7,606 | 1.183 | 1.302 | 24,374 → 18,845 → 15,542 |

GM over the six: E / D **1.159**, E / C 1.276 (D / C 1.101 in this session). WAL bytes per transaction and transactions per sync are unchanged (for example 2,611 → 2,616 B/tx and 61.5 → 60.2 tx/sync at 64w width 16 uniform).

**16w width 1.** Phase E is 1.7% above Phase D and 2.8% below Phase C (the three runs: C 8,300–8,416, E 8,027–8,178). Coordinator CPU outside the sync is lower than Phase C's (26.7 vs 29.5 µs/tx), but WAL sync time per transaction is higher (93.3 vs 87.4 µs) at slightly fewer transactions per sync (8.0 vs 8.4). This is the Phase D effect: with 8-transaction groups the second lane competes with ZFS's sync work for the two cores. Phase E did not remove it.

**Gates.** 64w width 16 uniform E / D = 1.423 ≥ 1.10, but 16w width 1 E / C = 0.972 < 0.98, so the full 14-scenario matrix and the RocksDB rerun were not run.

### Should the Phase D executor stay?

Same session, Phase C against Phase E with the serial executor (`--parallel-workers 0`, same binary) and with 2 lanes:

| Scenario | Phase C | E serial | E 2 lanes | E serial / C | E 2 lanes / C | 2 lanes / serial |
|---|---|---|---|---|---|---|
| 16w w1 uniform | 8,289 | 7,879 | 7,692 | 0.950 | 0.928 | 0.976 |
| 16w w16 uniform | 1,861 | 2,111 | 2,601 | 1.134 | 1.397 | 1.232 |
| 64w w1 uniform | 23,209 | 23,625 | 27,417 | 1.018 | 1.181 | 1.161 |
| 64w w16 uniform | 1,999 | 2,587 | 3,421 | 1.294 | 1.711 | 1.322 |
| 64w w16 compact | 10,608 | 11,587 | 12,955 | 1.092 | 1.221 | 1.118 |
| 64w w16 spread | 5,922 | 6,742 | 7,509 | 1.139 | 1.268 | 1.114 |

(Phase C's 16w width 1 runs were 9,384, 7,769 and 7,714 here; without the first run the three variants are within 1.3% of each other.)

On top of Phase E the parallel executor is worth 1.11–1.32× everywhere except 16w width 1, where it costs 2.4%. That is well above the "~10%" at which the plan says to revert, and the width-1 loss is small but real and limited to small groups. **Recommendation: keep the Phase D executor.** Making it fall back to the serial executor for groups with few leaf jobs is a tuning question for later, not part of this phase.

### Next bottleneck

**Per-entry reference counts on unchanged leaf entries.** After Phase E the lane's copy-on-write clone and the retired generation's drop still change the count of every key and inline value in each touched leaf: about 120k cycles per width-16 transaction, 12% of all cycles, the largest item the store itself owns. They are not many (about 950 count changes per transaction: a clone and a drop of each key and value) but each one misses cache, because the counts live in separate payload allocations last touched on another core. Inlining keys (E2b) showed that simply making entries bigger moves the cost into copying. The direction is a leaf representation whose entries hold no reference each (for example one shared byte arena per leaf with offsets in the entries, so a lane clone copies a plain vector and bumps one count), keeping the same page encoding and the same copy-on-write rule for published pages.

### Files

`results/wal-v3-phase-e/`: `design.md`, `correctness.md` (+ `correctness/` logs, `crossver/` Phase D vs Phase E byte-for-byte check), `environment.txt`, `run-order.txt`, `process-metrics.jsonl`, progress logs, `raw/` (counter rows, step rows, CPU gate, durable gate, retention, perf rows), `counters/` (per-site tables and `steps.md`), `tables.md`, `analysis.json`, `perf/` (`perf.data`, reports, `copy-alloc-refcount*.txt`, `sources-*.json`), `scripts/` (`run_phase_e.py`, `analyze_phase_e.py`, `churn_summary.py`, `perf_categories.py`, `perf_sources.py`), `SHA256SUMS`.

## Phase F — leaf-local packed storage

Question: can replacing per-entry `Arc` ownership with leaf-local packed storage trade remote atomic reference-count traffic for cheaper sequential copies? The page and WAL encodings, revision rules, routing, overflow pages, checkpoints, planner and two-lane executor remain unchanged. F1 adopts the packed leaf representation; F2 (combining the slot and byte buffers into one allocation) was not run because the measured clone/drop allocation cost was only about 27k cycles/transaction, too small to justify another representation change. The adaptive parallel threshold is a separate control measured on the same final representation.

### F0 population and representation

The OCI probe sampled 64 writers, width 16, before the representation change. Uniform touched leaves held 15.1 entries on average (p50 15, p95 15, max 29), with 495 key bytes and 970 inline-value bytes on average. The combined key/value payload averaged 1,464 bytes (p95 1,455, max 2,789); including slots, a packed clone copies about 2.1 KB per ordinary touched leaf, not a 4 KiB page.

Phase E's `LeafEntry` was 48 bytes and stored each key and inline value in a separate `Arc<[u8]>`; cloning a leaf copied about 720 bytes of entry records and changed roughly 948 payload reference counts per width-16 transaction. Phase F uses a 40-byte `LeafSlot` with offsets into one leaf-local byte buffer. A copied leaf has a slot vector and a byte vector; neither keys nor inline values have per-entry owners. The existing `Arc<BlinkPage>` remains the page-version ownership boundary, with no additional `Arc<LeafData>` layer. At 64w width 16 the measured clone volume is 9,090 slot bytes plus 21,967 payload bytes per transaction, or about 610 + 1,478 bytes per touched leaf. Cloning now needs two leaf-buffer allocations rather than one entry-vector allocation; total instrumented allocator calls were essentially flat (526 to 524 per transaction, harness included), and the lane allocation calls were 125 to 123.

### F1 CPU profile and step result

Real-sync frame-pointer profiling on the same OCI host compared Phase E (`4ead3de`) with packed F1 (`fe9dfad`), both built with CRC enabled. Arc reference-count samples fell from 110,451 to 35,871 cycles per transaction (11.8% to 4.5%); per-entry key/value clone and drop operations are zero in the packed leaf. The remaining Arc work belongs to page-version ownership. Sequential copies also fell from 52,930 to 38,088 cycles per transaction, while allocator time fell from 269,429 to 242,547 cycles. Total sampled cycles fell from 936,819 to 791,860 per transaction (15.5%). The copy/allocation/refcount group fell from 493,048 to 356,654 cycles. These results show the saved atomics were not paid back as memcpy work.

| Real-sync run | Phase E tx/s | F1 tx/s | F1 / E |
|---|---:|---:|---:|
| 64w w16 uniform | 3,523 | 3,989 | **1.132** |
| 64w w1 uniform | 28,397 | 30,246 | 1.065 |

The copy counters show 14.9 leaf clones and about 225 copied entries per transaction in width-16 uniform. They also show the additional payload copy F1 introduces: about 22.0 KB of key/value bytes per transaction, alongside 9.1 KB of slots. Entry clone/drop allocations are replaced by packed buffers without increasing total allocation calls. A separate reopen probe on a local Mac decoded 6,759 pages: allocations per open fell from 274,632 to 81,258, with open time remaining within the same 32–36 ms range. This is local decode evidence, not an OCI throughput result.

### Adaptive parallel control

On the same F code, `--parallel-min-mutations 32` selected the serial executor for small groups and retained two lanes for larger groups. Against always-two-lane F, the geometric mean across five measured cases was 1.011. The 16w width-1 case was effectively tied (1.005); 64w width-1 improved to 1.033, while 16w width-16 was 0.985. The control restores small-group performance without materially changing the larger-group wins. The final durable and matrix tables use the adaptive setting.

### Durable gate and full matrix

The six-scenario real-sync gate passed both thresholds: 64w width 16 uniform F/E was 1.122×, and 16w width 1 F/Phase C was 1.026×. The gate's six-row geometric mean was F/E 1.083. The full 14-scenario matrix then measured a 1.029 F/E geometric mean. Width-16 uniform was 1.098×; single-writer width-1 uniform was 0.980×, where throughput is dominated by sync and within run variation.

| Scenario | Phase C tx/s | Phase E tx/s | Phase F adaptive tx/s | F / E |
|---|---:|---:|---:|---:|
| 16w w1 uniform | 8,089 | 8,025 | 8,503 | 1.060 |
| 16w w1 compact | 9,395 | 9,500 | 9,325 | 0.982 |
| 16w w1 spread | 8,541 | 8,440 | 8,624 | 1.022 |
| 16w w16 uniform | 1,902 | 2,674 | 2,867 | 1.072 |
| 16w w16 compact | 5,975 | 6,588 | 6,592 | 1.001 |
| 16w w16 spread | 5,551 | 6,183 | 6,337 | 1.025 |
| 64w w1 uniform | 23,953 | 28,087 | 29,750 | 1.059 |
| 64w w1 compact | 32,241 | 33,521 | 33,176 | 0.990 |
| 64w w1 spread | 26,521 | 30,882 | 30,809 | 0.998 |
| 64w w16 uniform | 2,045 | 3,512 | 3,855 | 1.098 |
| 64w w16 compact | 10,800 | 13,132 | 13,351 | 1.017 |
| 64w w16 spread | 6,030 | 7,675 | 8,220 | 1.071 |
| 1w w1 uniform | 1,345 | 1,349 | 1,322 | 0.980 |
| 1w w16 uniform | 806 | 836 | 871 | 1.042 |

The width-1 durable result does not reverse the adoption decision: Phase F passed the predeclared gate, and F1's targeted CPU and throughput results both exceed the strong-candidate bar. The 14-case mean is smaller because it includes sync-bound and compact workloads with little leaf-copy work.

### Same-session RocksDB comparison

After the full-matrix gate, the two requested RocksDB cases were run on the same OCI ZFS session. Ratios are geometric means of paired repetitions.

| Scenario | dodb Phase F tx/s | RocksDB tx/s | dodb / RocksDB |
|---|---:|---:|---:|
| 64w w1 uniform | 29,772 | 26,320 | **1.130** |
| 64w w16 uniform | 3,820 | 9,814 | **0.394** |

### Correctness and decision

The randomized packed-leaf differential covers 400 seeds × 120 operations and compares lookup, fit, split, decode, and encoded page bytes against an independent Phase-E-style reference encoder. The existing published-generation isolation, WAL fault, parallel differential and recovery tests passed. Public API cross-version runs across Phase E, F1 and final F produced identical transaction results, commit LSNs, gets, queries, scans, reopen scans, data-file bytes and WAL bytes across 12 configurations. The on-disk 4 KiB page format is byte-identical.

**Keep the packed representation.** It removes all per-entry Arc operations and, on the targeted profile, reduces total cycles by 15.5% while improving durable throughput by 13.2%. The final full-matrix improvement is more modest at 2.9% geometric mean, so this adoption is based on the stated targeted gate and direct CPU evidence, not the atom count alone. The next bottleneck is **malloc/free cost**: 242,547 sampled cycles per transaction (30.6% of F1 samples), still the largest measured category. The counter breakdown attributes 166 allocator calls and about 8.0 KB per transaction to the benchmark harness.

`results/wal-v3-phase-f/` contains `design.md`, `correctness.md` and correctness logs, `environment.txt`, `run-order.txt`, progress logs, `raw/` benchmark rows, `counters/`, `tables.md`, `analysis.json`, `perf/` profiles and reports, `crossver/`, `reopen/`, `scripts/`, `process-metrics.jsonl`, and `SHA256SUMS`.

## Phase G — mimalloc and arena-based allocation elimination

### Environment and allocator

The OCI benchmark host was reached at `217.142.246.204` with strict host-key verification and the supplied identity. Work ran in detached temporary worktrees; `/home/opc/dodb` stayed untouched on `experiment/b-link-batched-engine`. The host is Oracle Linux 9.8, AArch64 Neoverse-N1, 2 OCPU, Rust 1.98.1; `/bench/zfs/db` is the mounted ZFS dataset.

The workspace pins mimalloc 0.1.52. All executable targets in this checkout select `mimalloc::MiMalloc` at their binary boundary, and the phase0 counting allocator delegates to mimalloc. The storage library has no global allocator. The `dodb-server` package in this checkout is library-only, so no production executable exists here to configure; the production binary must select mimalloc at its own binary boundary.

### G0 mimalloc baseline

The plain phase0 executable without churn counters is the performance baseline. Runs used 2 seconds warmup, 5 seconds measurement and 3 repetitions. G0 throughput medians are in `results/wal-v3-phase-g/g0-summary.csv`; it includes the requested 4 sync-disabled controls and 6 durable scenarios.

For 64w width16 uniform, the sync-disabled G0 median is 6,204 tx/s and real-sync is 4,994 tx/s. Counter-enabled storage allocation attribution for the same primary workload measured 361.69 allocation calls and 141,995 bytes per successful transaction, excluding 166.06 benchmark-harness calls and 7,980 bytes. The largest storage sources were planner (103.46 calls / 9,766 bytes), leaf lane execution (92.49 / 103,490 bytes), and admission (70.17 / 4,866 bytes). Full stages, frees and important size buckets are in `results/wal-v3-phase-g/counters/oci-g0-stage-attribution.csv`.

Admission and planner descriptors have group lifetime. Lane results mix scratch with owned redo and page images, so only local metadata/scratch could enter an arena. Catalog/publication allocations may be reachable from pinned generations or dirty state and must remain normally owned. WAL payloads and final page images must remain owned until append/publication completes.

A separate perf sample recorded 32.07B user cycles and 52.78B user instructions for 36,533 transactions (about 878k cycles and 1.445M instructions per transaction in that sample). `perf report` sampled mimalloc internals at 11.34% of total samples across allocator symbols (`_mi_page_malloc_zero` 3.12%, `mi_free` 2.26%, plus other allocation/free routines). At this sample rate that is roughly 99.6k sampled-equivalent cycles/tx; this is an estimate, not a precise allocator-cycle counter. Reports and raw profile are in `results/wal-v3-phase-g/perf/`.

### Arena and allocation candidates

No arena or reusable-buffer candidate passed the CPU gate, so none is retained. Candidate code snapshots, gate logs and raw rows are preserved in `results/wal-v3-phase-g/`.

| Step | Change measured | Primary result vs G0 | Allocation evidence | Decision |
|---|---|---:|---|---|
| G1a | Group `Bump` for dependency flags | 0.921x sync-disabled width16; 0.947x real-sync width16; 0.985x width1 | Reduced scratch heap work without total CPU gain | Reject |
| G1b | Also grouped transaction-to-leaf metadata in `BumpVec` | 0.949x; 0.958x; 0.965x | Allocation reduction did not offset regression | Reject |
| G1c | Arena-encoded mutation keys and logical overlay keys, borrowed requests | 1.007x; 1.003x; 1.008x | Storage calls 361.69 → 325.08/tx; bytes 141,995 → 140,888/tx; admission calls 70.17 → 35.22/tx | Reject; throughput and bytes were effectively unchanged |
| G2 | One shared job-operation slice, ranges per job (`Arc<[T]>`) | 0.921x; 0.977x; 0.976x | Removed per-job vectors but added shared result ownership cost | Reject |
| G2b | Shared operation `Arc<Vec<T>>` | 0.994x; 0.949x; 0.979x | Avoided slice conversion copy; no CPU gain | Reject |
| G3 | Two-pass canonical PageDelta span scan, no span `Vec` | 0.959x sync-disabled width16; 0.984x real-sync width16 diagnostic | PageDelta calls 32.17 → 16.05/tx and bytes 2,745 → 1,711/tx | Reject; the second scan cost more CPU than the removed allocation |
| G4 | Contiguous transaction-result metadata and index ranges | 0.974x sync-disabled width16 | Job-result collection calls 7.04 → 6.09/tx; storage total 361.69 → 359.60 calls/tx | Reject; small allocation reduction did not offset CPU cost |

G1a/G1b/G1c and G2/G2b passed `cargo test --workspace --release --no-fail-fast` on OCI. G3 and G4 also passed the full workspace release suite. G3 real-sync width1 diagnostics stopped at two repetitions after the primary gate failed. G4 was stopped after the primary gate failed; its real-sync controls were not run. The exact comparisons and runs are listed in `results/wal-v3-phase-g/gates/decision.csv`.

The arena-reuse test in the G1c candidate exercised group sizes `1, 64, 3, 128, 2, 32, 1, 256`, reopened the database and verified values remained isolated. It passed. Since all arena candidates were rejected, no arena memory is retained in the final candidate and no 120-second RSS test was triggered.

### Final gates and historical comparison

Final G is G0 mimalloc-only because every allocation optimization was rejected. Thus final/G0 is 1.000x for each scenario by identity. The six-scenario durable comparison is `results/wal-v3-phase-g/durable-g0-vs-phase-f.csv`:

| Scenario | G0 mimalloc tx/s | Phase F historical tx/s | G0 / F | Final / G0 |
|---|---:|---:|---:|---:|
| 16w width1 uniform | 8,692 | 8,454 | 1.028x | 1.000x |
| 16w width16 uniform | 3,361 | 3,016 | 1.114x | 1.000x |
| 64w width1 uniform | 25,628 | 30,501 | 0.840x | 1.000x |
| 64w width16 uniform | 4,994 | 4,031 | 1.239x | 1.000x |
| 64w width16 compact | 16,373 | 13,298 | 1.231x | 1.000x |
| 64w width16 spread | 10,370 | 8,268 | 1.254x | 1.000x |

The required arena success threshold of 1.05x over G0 is not met. Therefore the 14-scenario matrix, current RocksDB rerun and 120-second sustained-memory run are not triggered. This does not change the mandated mimalloc adoption. The Phase F RocksDB reference remains width1 1.130x and width16 0.394x; no Phase G RocksDB result is claimed.

A deterministic public-API comparison between Phase F and final G ran 12 seed/value-limit/worker configurations. Transaction results, commit LSNs, reads, queries/documents, scans, reopen scans, data-file SHA256/size and WAL SHA256/size are byte-identical. The outputs are `results/wal-v3-phase-g/crossver/output-phase-f.txt` and `output-g0.txt`.

### Decision and next bottleneck

Keep mimalloc. Reject G1a through G4 because none reduced total CPU work while preserving width1 throughput. The final measured profile's largest single storage-owned category is **CRC32C** at about 11.2% of sampled cycles across storage worker/coordinator threads. The raw perf report is authoritative for that estimate; the next experiment should target CRC work without changing PageDelta or WAL bytes.

Implementation, candidate source snapshots, benchmark rows, counters, perf samples, correctness logs, byte-identity outputs and run-order files are in `results/wal-v3-phase-g/`.

## Phase H — CRC32C scan elimination

### Hardware and provenance

H0/G0 was rebuilt from `a78472377fa16ed491c18bd2d6770f7dba7e9311`; H1 was built from `5998f7c47a15850c40bb40c8c378ac5c0deca061`. The accepted benchmark rows report those exact commits, and the binary SHA256 values are recorded with the raw run order. The OCI canonical checkout was `/home/opc/dodb-wal-v3`; G0 used a temporary detached worktree. Both builds used rustc 1.98.1, target `aarch64-unknown-linux-gnu`, repository `-C target-feature=+crc`, and `crc32c` 0.6.8 with `cfg armsimd`. `objdump` confirmed AArch64 `crc32cb` and `crc32cx` in both binaries. A first set of source-only-copy results reported `git_commit=unknown`; those files are retained under `results/wal-v3-phase-h/*/rejected-provenance/` and excluded from this section.

### H0 attribution and H1 result

For 64w width16 uniform, G0 performed 47.191 full-page CRC32C scans per transaction: 15.982 canonical page checksums, 15.983 image fingerprints, and 15.226 prior-page chain validations. This hashed 193,295 full-page bytes per transaction. Existing worker-stage timers put the base, encode, and fingerprint paths at about 27.93, 29.03, and 32.00 µs/tx respectively; these include adjacent work. The profile estimates CRC32C at 11.17% of sampled cycles, about 71,496 cycles/tx.

H1 keeps the existing PageDelta fingerprint and persisted checksum semantics. It carries trusted immutable page ID, LSN, chain fingerprint, and embedded checksum metadata with the image, then validates that association in constant time. Untrusted file/WAL bytes still take the original full validation path. The prior-page full CRC scan is eliminated: total full-page scans fall to 31.965 and bytes to 130,929 per transaction. Sampled CRC cost falls to 6.38%, about 39,970 cycles/tx; total profile cycles fall 2.1%.

| 64w width16 uniform | G0 | H1 | Throughput H1/G0 or cycles G0/H1 |
|---|---:|---:|---:|
| Sync-disabled tx/s | 5,707 | 6,150 | 1.078x |
| Sync-disabled cycles/tx | 984,778 | 895,007 | 1.100x |
| Real-sync tx/s | 4,461 | 4,473 | 1.003x |
| Real-sync cycles/tx | 1,028,777 | 1,019,775 | 1.009x |

The six-scenario durable gate completed three interleaved repetitions per variant. The 64w width1 real-sync protection is 0.997x, above its 0.98 floor. The 64w width16 uniform primary is 0.987x, below the 1.04 full-matrix gate, so the 14-scenario matrix and RocksDB rerun were not triggered.

### H2, H3, correctness, and next bottleneck

The H2 randomized digest test passed bit-for-bit across 4,096 cases on the OCI AArch64 host at H1 commit `5998f7c47a15850c40bb40c8c378ac5c0deca061`, but `crc32c_combine` was much slower than the second payload scan: 159-byte records took 73 ns direct versus 8,510 ns combined; 4,079-byte records took 545 ns versus 17,737 ns; 4,104-byte records took 543 ns versus 18,725 ns. H2 was rejected without production digest changes. H3 was skipped because the remaining canonical page checksum path is about 1.1% of sampled cycles, below its 5% prerequisite.

The full release workspace suite passed. The deterministic public API comparison is byte-identical across 12 configurations for transaction results, commit LSNs, reads, queries, scans, reopen scans, data-file SHA256 and WAL SHA256. The production executable allocator item remains open because `dodb-server` is library-only in this checkout; the eventual production binary must select mimalloc at its binary boundary.

The largest remaining storage-owned category in the H1 profile is **`encode_page_delta()` at 9.85%** of sampled cycles. It is larger than the remaining CRC32C category at 6.38% and is the one next bottleneck selected from the measurement. Phase H artifacts, provenance, raw data, scripts, profiles, correctness logs, tables, and checksums are in `results/wal-v3-phase-h/`.

## Phase I — deferred commit overlay feasibility

Phase I measured whether commit response can follow logical WAL durability and immutable committed-overlay publication, with B-link page materialization deferred. Production WAL v3 and the production write path were not changed. Actual group routing showed that 64-writer, width16 uniform groups touch 646–666 distinct leaves out of 683–702 boundaries; only 5.1–5.3% of current materializations could be removed by final-leaf-only coalescing. Model A is therefore not a solution for uniform traffic, although same-leaf-heavy controls had 94% coalescing opportunity.

For Model B, H1 CPU demand was estimated at 193 us/transaction and real-sync wait at about 72 us/transaction. Perfect CPU/fsync overlap has a 1.37x ceiling; measured real-sync throughput was 0.751 of sync-disabled throughput (1.33x), leaving little pipeline headroom on a 2 OCPU host. For Model C, the inclusive physical preparation subtree accounts for about 42.78% of sampled user cycles, a theoretical 1.75x CPU ceiling. An isolated volatile-sync prototype measured 6.0–6.5 us/transaction CPU, 0.031–0.034 of the H1 estimate, and passed the requested write CPU gate. These prototype numbers are not durable throughput.

The read model still needs redesign: base-fallback point GET reaches 1.51–1.59x at four segments; the prototype's Query merge is about 3.2–3.4x at one to sixteen segments, and its intentionally simple Scan merge is 49.7x at one segment and grows with segment count. The phase decision is **Worth pursuing, but redesign needed**. The next phase recommendation is **Phase J — committed logical overlay + WAL v4**, beginning with bounded read amplification and checkpoint/materialization semantics. WAL v4 and production overlay code remain unimplemented.

Phase I measurements, raw artifacts, perf reports, prototype source, and the detailed design are in `results/wal-v3-phase-i/`.

## Phase J — bounded committed overlay read model

J0 is implemented and its read gate passed on the OCI A1 Neoverse-N1 host. The production-style published generation now pins the immutable B-link base and a bounded set of packed sorted overlay segments as one snapshot. Point lookup walks newest to oldest with a segment-local Bloom filter. Query and Scan use a streaming ordered merge with newest-version-wins and tombstone suppression; the base-only case retains the H1 read path.

The measured maximum is 1.459x for GET at four overlays; four-overlay Query/Scan at limit 8 is 1.745x/1.710x. One-overlay Scan is 1.490x at limit 8, 1.363x at limit 128, and 1.301x at limit 4096. The initial maximum is four segments. J1's volatile 64-transaction, width-16 CPU prototype measured 9.39–9.87% of H1 foreground CPU across zero to three pre-existing segments and reduced allocations from 297.219 to 87.734 per transaction. J2 has an explicit v4 logical WAL open path that admits mutations without B-link page work, syncs before publishing an immutable segment, and recovers committed mutations. It reports physical Commit frame LSN and logical revision separately. The v3 physical writer remains available for H1 comparison and old-format recovery. The OCI A1 WAL byte-accounting control measured 2,704.062 bytes and 17 records per transaction at width 16, with 64 transactions sharing one sync; it used `MemoryFile`, so it is not a durable throughput result.

J3 implements synchronous checkpoint materialization at the four-segment bound. It copies the reachable base into inactive page IDs, applies the current overlays with their original revisions, syncs new data pages, durably publishes a superblock carrying separate physical LSN and logical sequence watermarks, publishes the new view, retires old pages, and only then resets WAL. The page bank is reusable after capacity expansion. A 256 MiB WAL cap rejects oversized groups before append. Full workspace release tests and crashable-file tests cover old/new superblock state, partial page writes, sync failures, WAL reset windows, overflow, pinned readers, subsequent writes, and repeated reopen. J0–J3 artifacts are in `results/wal-v3-phase-j/`.

J4 completed six real-sync scenarios with three same-session interleaved H1/J repetitions on the OCI A1 ZFS dataset. Primary 64-writer width-16 uniform reached 0.720x H1; the 64-writer width-1 guardrail reached 0.376x and the 16-writer width-1 guardrail 0.312x. The 16-writer width-16 uniform, compact, and spread ratios were 0.473x, 0.413x, and 0.395x. These miss both the 1.20x width-16 gate and the 0.95x width-1 protection. J's primary run materialized 270 times in 10 seconds with 22.90 ms average and 31.22 ms maximum pauses. The final decision is **Keep experimental only; the synchronous materialization cost is too high**. The sustained run and RocksDB comparison were not run because their prerequisite gates failed. The next bottleneck is the synchronous full-overlay-set materialization pause.

The final `cargo test --workspace --release --no-fail-fast` passed at source commit `0ec2528a7513e507b2033aba095a937373e754b7`. Artifacts and raw data are in `results/wal-v3-phase-j/`. The canonical push target `namseent/dodb` was unavailable; the existing `origin` remains `namse/dodb` and was not used for publication.
