# Concurrent checkpoint validation

## Final implementation identity

Implementation source: `d240416f0e0efafa7c08bb1e868387ebd0d894bc` on `experiment/blink-background-checkpoint-20261021`, in `/Users/namse/fn0-dodb-background-checkpoint`. The approved OCI checkout uses that exact detached commit. Validation checks that the checkout is clean before and after execution. The standalone `/Users/namse/dodb` worktree and its pre-existing `phase0-bench.rs` modification are preserved.

The default remains legacy WAL, synchronous execution, and disabled automatic checkpointing. Segmented background execution is explicit. Preparation retains the existing invariant check and durable rotation; worker page writes/data sync/metadata sync overlap later write groups. Manifest publication and reclamation run under coordinator ownership using the latest manifest.

## Functional and recovery evidence

The final OCI release validation completed **314 passing test functions, zero failures, and four ignored diagnostic probes**, across 16 test executables. The new shared transaction, read, conditional-history, and bounded linearizable-history checks run both segmented synchronous and background execution. Existing B-tree, legacy B-link, and native/wrapped RocksDB checks are retained.

| Evidence | Coverage |
| --- | --- |
| `background_checkpoint`, 7 passing functions | Barrier-controlled acknowledged writes while the worker is active; overwrite/delete/recreate and actual tree growth; post-target rotation; exact state/revision preservation; synchronous waiting; worker failure/panic; capacity waiting and oversized rejection; one process-wide worker across shards; shutdown joining active work; rechecking a coalesced automatic threshold after reclamation |
| `background_checkpoint_sigkill`, 3 passing functions | One child helper, 19 injected checkpoint failure boundaries, and ten distinct real SIGKILL boundaries with acknowledged post-target writes; recovery, later successful write/checkpoint, and another reopen |
| `blink_wal_checkpoint`, 8 passing functions | Default-disabled legacy behavior, explicit segmented selection, repeated reclamation, accounting, failure behavior, reopen, and existing process-termination coverage |
| `dodb_storage`, 202 passing functions, 4 ignored | Existing codecs, transaction execution, segment recovery/rotation, checkpoint fault boundaries, page deltas, pinned readers, and storage invariants |
| Shared contracts and remaining executables | Native/wrapped RocksDB semantics, sync/background transaction/read/history behavior, driver histograms/timing, transport, and legacy process/crash recovery |

The four ignored functions are existing `measure_bounded_overlay_read_path`, `measure_logical_overlay_write_cpu`, `measure_logical_wal_v4_write_bytes`, and `phase_b0_encoded_size_probe`. They are diagnostic/performance probes and were not enabled for this functional gate.

Local evidence includes a 309-pass/4-ignored full storage/server run at the initial implementation milestone, subsequent six-function overlap/shutdown checks, and a 17-function checkpoint/overlap/recovery selection after preserving invariant checks, and a final 18-function selection after the coalesced-threshold fix. These overlapping selections are recorded separately and are not added together as distinct tests. Final source hashes and original output bytes are under `evidence/concurrent-local/`.

## Finite model

`dodb/formal/quint/background_checkpoint.qnt` models fixed-target data sync, concurrent acknowledged writes/rotation, publication into the latest manifest, reclamation, and crash recovery. Quint 0.32.0 typechecking and its explicit concurrent-rotation example pass. The final 10,000-trace simulation found no invariant violation and witnessed overlap. TLC exhausted 150 distinct states, generated 261 states, and reached depth 13 without a violation.

The model abstracts transaction/WAL coverage. It does not prove the Rust implementation, page codecs, hardware persistence, or physical power-loss behavior. SIGKILL evidence is process-termination evidence.

## Initial implementation and rejected attempts

`VALIDATION-INITIAL.md` preserves the initial `58a197a17` validation: 313 passes, four ignored probes, build failures, a cache-alias launcher rejection, and missing auxiliary child executable paths. These overlapping initial/final suites are not added together. Both implementations retain the same invariant checks, transaction contracts, durability, workload, and retention ceilings.

The complete initial performance matrix revealed twice as many measured checkpoints in background 128 MiB as in segmented sync 128 MiB. A new 64 KiB-threshold regression confirmed that a coalesced request could start another checkpoint with only 4,328 required WAL bytes after reclamation. It failed before the fix and passes afterward. Completion now rechecks the remaining WAL threshold; capacity-forced checkpoints remain independent. The regression also recovers the acknowledged post-target value and exact revision after shutdown/reopen. Original before/after output and checksums are in `evidence/coalesced-threshold/`.

The final monitored OCI release build completed in 339.10 seconds, with observed process-group peak RSS 1,802,424,320 bytes, minimum host available memory 3,259,785,216 bytes, and minimum filesystem free space 42,473,693,184 bytes. Its final validation completed in approximately 98.29 seconds without a resource stop or rejected launcher attempt. The authoritative completion record reports 314 passes, zero failures, and four ignored functions across 16 executables. Non-strict local Clippy reports 15 diagnostics whose primary lines pre-exist the task baseline; strict Clippy is not claimed to pass. No suppressions were added. Final formatting passes for `dodb-storage`, `dodb-server`, and `dodb-rocksdb`. A repository-wide format check reports differences in 11 existing `vendor/deno/ext/crypto` files; those unrelated files remain untouched. Both outputs are preserved under `evidence/final-format/`.

## Performance gate

All ten eight-second preflight cells passed source/executable identity, full latency counts, fixed input-prefix comparison, final-state validation, and reopen validation. These are preflight results rather than the requested 30-second performance comparison. Both the preserved initial and corrected 30-cell A/E comparisons are recorded separately in `RESULTS.md`; no uncompleted or error-containing cell is accepted as a clean performance result.
