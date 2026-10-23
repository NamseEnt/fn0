# Functional and durability validation

The primary-budget implementation at `400ad0ac6b7d8e6306bcb2ab7946747bddcb06c4` passes **337 tests, zero failures, four existing ignored probes**, across twenty selected optimized OCI test executables. The complete commands, executable identities, output and count reduction are preserved in the evidence archive's `validation/` directory. Both storage and server executables named `durable_base_checkpoint` are included.

## New coverage

The twelve storage integration tests cover:

- Page-delta reuse across repeated checkpoints, forced segment rotations, normal reopen and a second reopen, in both write-coalescing modes.
- Unchanged primary-cache bytes during checkpoint, cold-page base reuse, bounded pack count, consolidation and obsolete-file reclamation.
- Every new page-pack write/sync/directory boundary and checkpoint manifest/reclamation boundary, with acknowledged state preserved and future writes failed closed after an error.
- Required-file loss, truncation, changed identity, page corruption and reconstruction of a corrupted disposable database cache.
- A complete uncommitted transaction plus torn terminal frame, recovery, a new successful mutation and another recovery.
- A real public `put` failing during delta-payload writing or before its commit record, followed by recovery, a successful acknowledged mutation, checkpoint and second recovery.
- Multi-key conditions, shared logical revisions, tombstones, overflow growth and page reuse.
- The regression where pages replayed after a checkpoint were omitted from the next immutable checkpoint before WAL deletion.
- Rejection of implicit legacy/ordinary-segmented/bases layout conversion.
- Reservation of pending output and primary-page versus extra-recovery capacity, including progress when the live primary image exceeds half the extra budget.

The storage SIGKILL executables interrupt synchronous and background checkpoints at actual completed file-write, sync, manifest and reclamation boundaries. Background cases acknowledge post-target writes, force additional rotations and tree growth, then verify them after restart and after another successful write/checkpoint/restart. These are process-crash checks, not physical power-loss experiments.

The six new server tests exercise the public service path, acknowledged write overlap, the identical synchronous worker path, failure/panic propagation, multi-shard scheduling through the existing one-worker permit, coordinator shutdown and the coalesced-threshold regression. Existing storage/server/transaction/read/history/QUIC and recovery suites remain in the OCI gate.

## Retained corrections

The initial recovery-growth failure and its databases/logs were preserved. The fix marks replayed post-checkpoint data pages dirty before the next immutable checkpoint is allowed to reclaim their WAL.

A pre-existing SIGKILL readiness fixture could expose an empty marker between file creation and write completion. Both affected existing checkpoint fixtures and the new fixtures now publish the marker by atomic rename. The rejected empty-marker logs remain separate from successful crash validation.

The first combined 512 MiB primary-plus-redo budget passed small durability tests but rejected E's larger prepopulated dataset during seeding. The failed preflight and database remain under the original OCI root. The corrected capacity model counts the live primary checkpoint image separately, retains a fixed 512 MiB additional recovery budget and adds a regression covering that distinction. Its accepted performance evidence starts from the new clean source/root.

## Other checks

Formatting checks pass for the three changed packages. Non-strict Clippy completes with the existing twelve storage and three server library warnings; no lint suppression was introduced and strict Clippy is not claimed as passing. Local successful and failed compilation/test logs are inventoried in `evidence/local-validation-inventory.json`.

## Completed measurement and evidence gates

The corrected candidate completes eight preflight cells, twenty-four 60-second cells and twelve 300-second cells. All 44 pass final/reopen state, exact revisions, complete latency counts, input-prefix, source/binary identity, checkpoint and resource gates. Each sustained cell has at least ten completed checkpoints. The selected candidate fails the separate performance adoption gate; see RESULTS.md for the measured latency, reopening and resource trade-offs.

Offline verification checks every original file in both OCI archives (608 accepted-root files and 113 rejected-root files), recounts all twenty functional logs, reproduces every original short/sustained summary value, and checks selection hash, cross-stage prefixes, command-contract equality and final source/binary identities. The original summaries are preserved. Full-lifetime external resource samples are checked separately from the capped internal resource vectors. Local rejected fixtures retain six directories and 45 files with verified archive SHA256 `027569f2c581ad3e01e8795f6dea8038d42e1511bfc68cde17b4e2e8a7318cd2`.
