# B-link Segmented WAL Checkpoint Results

## Current milestone

Implementation source: `3d883db0c75b51f31463f197eefe53157974bf0a`. Final local storage/server tests: **301 passed, 4 ignored**. Compile and formatting checks passed; strict Clippy remains blocked by 12 diagnostics on pre-existing code.

The three review findings are corrected, and segmented WAL now supports synchronous checkpoint advancement and closed-file reclamation. The actual path is `BlinkTenantService` with explicit `BlinkWalLayout::Segmented` configuration, through planned `BlinkStore` transaction execution, to segmented WAL append/sync and `BlinkStore::checkpoint`. The existing coordinator awaits checkpoint completion between write groups.

- Recovery removes complete uncommitted page frames and partial transaction tails through the last complete commit, then synchronizes truncation before writes resume. The regression covers recovery, a successful new write, another recovery, and later rotation, both before and after a nonzero checkpoint.
- The default layout is restored to `Legacy`; checkpoint policy remains `Disabled`. Segmented mode is explicit, positive segment-size configuration is validated, and neither layout automatically converts the other.
- WAL byte accounting includes all manifest-required files. Counters accumulate across rotation and reclamation; recovered history counters cover retained files and runtime sync/timing counters restart on open.
- Checkpoint forces an active-file rotation at C, synchronizes data and alternate superblock metadata, durably commits the manifest checkpoint at C, and only then unlinks covered closed files and synchronizes their directory. The new active file remains intact.
- Recovery repairs pages before loading the tree and restores the manifest checkpoint authority over older checkpoint fields in replayed superblocks. Missing required files are fatal; obsolete files cannot substitute for them.

The implemented checkpoint is synchronous. Background write/checkpoint overlap, immutable worker snapshots, concurrent manifest update coordination, retention backpressure, workload CLI selection, format/model validation for the full background design, OCI validation of this new milestone, and performance matrices remain unimplemented or unrun. No performance, equal-memory, steady-state, or power-loss claim is made. SIGKILL evidence covers process termination only.

See the appended milestone record in `VALIDATION.md` for source identity, commands, counts, failure boundaries, and preserved logs. `RECOVERY-PROTOCOL.md` distinguishes implemented behavior from the future background design. `PLAN.md` remains the larger background project plan.

## Preserved initial report

The following text records the earlier codec-only milestone at `c11f5a277`; its implementation status does not describe the current source.

# Background B-link Checkpoint Results

## Status

Implementation is incomplete. The current change contains the pre-implementation protocol documents and an isolated manifest codec with three passing unit tests. The codec is not yet used by the B-link storage path. The complete `dodb-storage` library suite passed 187 tests with 4 ignored after updating a stale exact-sync expectation; its original failure and same-baseline reproduction are preserved. There is no segmented WAL rotation or recovery, no physical background checkpoint, no write/checkpoint overlap evidence, and no end-to-end execution mode or CLI support.

No performance measurement has run. The mandatory functional and recovery gates are not complete, so the short and long matrices were not started.

## Environment gate

The approved OCI host was reachable and had the required ZFS mount. Its default Rust/Cargo is 1.98.1, and the required 1.97.1 toolchain is installed for explicit selection. Two clean existing RocksDB source checkouts match the pinned source commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; the previous experiment records archive SHA-256 `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`. No performance execution has used the host. The source helper/cache provenance must still be verified before any build.

## Current evidence

See `VALIDATION.md` for the exact worktree, test command, OCI inspection, and unrun requirements. `PLAN.md` and `RECOVERY-PROTOCOL.md` describe the predeclared implementation and recovery contract. No claim is made that those protocols are implemented or that this change is safe to use for durable storage.
