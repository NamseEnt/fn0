# Concurrent checkpoint validation

## Final implementation identity

Implementation source: `58a197a1711a51f76188db4e9bc314fc89bbf5c1` on `experiment/blink-background-checkpoint-20261021`, in `/Users/namse/fn0-dodb-background-checkpoint`. The approved OCI checkout uses that exact detached commit. Validation checks that the checkout is clean before and after execution. The standalone `/Users/namse/dodb` worktree and its pre-existing `phase0-bench.rs` modification are preserved.

The default remains legacy WAL, synchronous execution, and disabled automatic checkpointing. Segmented background execution is explicit. Preparation retains the existing invariant check and durable rotation; worker page writes/data sync/metadata sync overlap later write groups. Manifest publication and reclamation run under coordinator ownership using the latest manifest.

## Functional and recovery evidence

The final OCI release validation completed **313 passing test functions, zero failures, and four ignored diagnostic probes**, across 16 test executables. The new shared transaction, read, conditional-history, and bounded linearizable-history checks run both segmented synchronous and background execution. Existing B-tree, legacy B-link, and native/wrapped RocksDB checks are retained.

| Evidence | Coverage |
| --- | --- |
| `background_checkpoint`, 6 passing functions | Barrier-controlled acknowledged writes while the worker is active; overwrite/delete/recreate and actual tree growth; post-target rotation; exact state/revision preservation; synchronous waiting; worker failure/panic; capacity waiting and oversized rejection; one process-wide worker across shards; shutdown joining active work |
| `background_checkpoint_sigkill`, 3 passing functions | One child helper, 19 injected checkpoint failure boundaries, and ten distinct real SIGKILL boundaries with acknowledged post-target writes; recovery, later successful write/checkpoint, and another reopen |
| `blink_wal_checkpoint`, 8 passing functions | Default-disabled legacy behavior, explicit segmented selection, repeated reclamation, accounting, failure behavior, reopen, and existing process-termination coverage |
| `dodb_storage`, 202 passing functions, 4 ignored | Existing codecs, transaction execution, segment recovery/rotation, checkpoint fault boundaries, page deltas, pinned readers, and storage invariants |
| Shared contracts and remaining executables | Native/wrapped RocksDB semantics, sync/background transaction/read/history behavior, driver histograms/timing, transport, and legacy process/crash recovery |

The four ignored functions are existing `measure_bounded_overlay_read_path`, `measure_logical_overlay_write_cpu`, `measure_logical_wal_v4_write_bytes`, and `phase_b0_encoded_size_probe`. They are diagnostic/performance probes and were not enabled for this functional gate.

Local evidence includes a 309-pass/4-ignored full storage/server run at the initial implementation milestone, subsequent six-function overlap/shutdown checks, and a final 17-function checkpoint/overlap/recovery selection after preserving invariant checks. These overlapping selections are recorded separately and are not added together as distinct tests. Final source hashes and original output bytes are under `evidence/concurrent-local/`.

## Finite model

`dodb/formal/quint/background_checkpoint.qnt` models fixed-target data sync, concurrent acknowledged writes/rotation, publication into the latest manifest, reclamation, and crash recovery. Quint 0.32.0 typechecking and its explicit concurrent-rotation example pass. The final 10,000-trace simulation found no invariant violation and witnessed overlap. TLC exhausted 150 distinct states, generated 261 states, and reached depth 13 without a violation.

The model abstracts transaction/WAL coverage. It does not prove the Rust implementation, page codecs, hardware persistence, or physical power-loss behavior. SIGKILL evidence is process-termination evidence.

## Preserved rejected attempts

- The initial local capacity fixture used 128 KiB and did not force a wait. The corrected fixture uses 96 KiB and retains the wait assertion; no production bound or assertion was relaxed.
- OCI report construction first exceeded the existing JSON macro expansion limit. Reports/events were split into separate values; no recursion-limit or lint-suppression attribute was added. The rejected `d8ef2cf53` and `4600981d5` builds are preserved.
- A subsequent all-target check found an unqualified workload test-module reference. Explicit crate qualification corrected it; its rejected `8610b7e14` check is retained.
- The first validation launcher rejected the shared-cache symlink alias. Resolving the approved cache path fixed the path guard; no host/cache substitution occurred.
- The second launcher passed all new contracts, overlap checks, storage tests, and new crash checks, then two existing legacy recovery functions stopped before invoking the engine because runtime `CARGO_BIN_EXE_*` paths were missing. The auxiliary executable paths and hashes were supplied. Successful executable runs were preserved, the failed log remains intact, and only that failed executable plus unrun executables continued. The accepted final selection has 313 passes; the earlier launcher failure remains explicitly recorded.

The final monitored release build completed in 550.45 seconds, with observed process-group peak RSS 2,056,318,976 bytes, minimum host available memory 3,116,957,696 bytes, and minimum filesystem free space 44,338,761,728 bytes. The resumed validation completed without a resource stop. Non-strict local Clippy reports 15 diagnostics whose primary lines pre-exist the task baseline; strict Clippy is not claimed to pass. No suppressions were added.

## Performance gate

All ten eight-second preflight cells passed source/executable identity, full latency counts, fixed input-prefix comparison, final-state validation, and reopen validation. These are preflight results rather than the requested 30-second performance comparison. The 30-cell A/E comparison is recorded separately in `RESULTS.md` when complete; no uncompleted or error-containing cell is accepted as a clean performance result.
