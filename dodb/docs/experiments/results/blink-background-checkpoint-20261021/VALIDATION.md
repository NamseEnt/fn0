# Final concurrent checkpoint validation

The final implementation and 314-pass OCI validation are documented in [VALIDATION-CONCURRENT.md](VALIDATION-CONCURRENT.md). Complete initial/corrected performance results are in [RESULTS.md](RESULTS.md). The records below preserve earlier codec and synchronous segmented milestones; their implementation-status statements describe those historical sources.

# Background Checkpoint Validation Record

The early sections below preserve the codec-only milestone. The final section, "Segmented recovery corrections and synchronous checkpoint milestone", records the current source and validation.

## Worktree and starting state

- Baseline: `39cdfe48a20fb6f381dbb461d721fd51fa8f9f0c`.
- Branch: `experiment/blink-background-checkpoint-20261021`.
- Worktree: `/Users/namse/fn0-dodb-background-checkpoint`.
- The existing `/Users/namse/dodb` and `/Users/namse/fn0-dodb-wal-reclamation` worktrees were not modified.
- Existing WAL reclamation evidence was read from `RESULTS.md`, `VALIDATION.md`, `corrections/README.md`, and `oci/matrix-verification-corrected-v4.json`; no original file was changed.

## Initial implementation check

The first implementation artifact is the versioned, CRC32C-protected segmented-WAL manifest codec at `dodb/crates/dodb-storage/src/blink/segmented_wal.rs`. It encodes shard/database identity, checkpoint tree metadata, ordered segment LSN ranges, the active/next segment number, and metadata generation. Its atomic writer synchronizes the temporary file and parent directory. It is not yet connected to `BlinkStore`, WAL rotation, recovery, checkpoint execution, service configuration, or workload CLI.

The code tested below is commit `0a57da5d3133081145f681ed96984b2893449462`, built with Rust/Cargo 1.97.1. Local debug test executable SHA-256 values are:

- `dodb-storage` library tests: `edfcfd23bbdcf7b057acf80de36ddaccbf33dbe89daa987a9bb2a5555b13e25c`.
- `dodb-server` `blink_wal_checkpoint`: `525c744eb6ae245c816e100e582c8f1ca2c1a12353cebd64db504cb1c44e63f8`.

- `cargo +1.97.1 test --locked -p dodb-storage --lib blink::segmented_wal::tests -- --test-threads=1`: passed, 3 tests.
- `cargo +1.97.1 fmt -p dodb-storage -- --check`: passed.
- `git diff --check`: passed.
- An initial format check before applying rustfmt reported formatting differences; the final format check above is clean.
- `cargo +1.97.1 test --locked -p dodb-storage --lib -- --test-threads=1`: passed after correcting the legacy checkpoint barrier's expected physical WAL sync count from 4 to 5; 187 passed and 4 were ignored. The initial failing output and unchanged-baseline reproduction are preserved in `evidence/legacy-sync-count-baseline-failure.txt`.
- `cargo +1.97.1 test --locked -p dodb-server --test blink_wal_checkpoint -- --test-threads=1`: passed, 6 tests, covering the existing default-disabled legacy policy, threshold checkpointing, failure behavior, reopen, and process termination.
- The focused test build emitted existing dead-code warnings in experiment binaries included by the package test target. No warning suppression was added.

These tests validate manifest encoding and local atomic file replacement only. They do not validate segmented WAL storage, acknowledged-write recovery, background checkpoint overlap, required format contracts, fault injection, SIGKILL recovery, or any performance behavior.

## Approved OCI environment check

The authorized host was reached using the specified identity, `IdentitiesOnly=yes`, and normal host-key validation. It reported:

- Host: `instance-20260923-1013`, Linux AArch64.
- Filesystem: `/bench/zfs/db`, ZFS dataset `dodbbench/db`, 149 GiB total and 44 GiB free at inspection.
- `MemAvailable`: 4,678,552 KiB at inspection.
- Active benchmark/build process check: no matching workload, Cargo, or rustc process was running; only the inspection shell matched its own process list.
- Default `rustc`/Cargo: 1.98.1. The required Rust/Cargo 1.97.1 toolchain is installed and reports `rustc 1.97.1 (8bab26f4f 2026-07-14)` and `cargo 1.97.1 (c980f4866 2026-06-30)` when selected explicitly.
- Existing RocksDB source checkouts at `/bench/zfs/db/crossdb-b1648b03-oci/rocksdb-source` and `/bench/zfs/db/product-build/rocksdb-source` both report the required source commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654`; their working trees were clean at inspection. The prior experiment records the pinned archive SHA-256 as `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
- The existing OCI build helper is present at `dodb/scripts/run-oci-rocksdb-build.sh`. No build or measurement was started during this environment inspection.

The environment matches the named host, filesystem, Rust toolchain availability, and pinned RocksDB source checkout requirements. The older OCI RocksDB source/archive and build cache are candidates for reuse only after their exact helper configuration and archive/source hashes are checked for a new build. No alternate host, local filesystem, or toolchain substitution was made.

## Required validation status

Not run: the sync/background contract suites; concurrent-write barrier tests; checkpoint state-preservation tests; segment transition and fault-boundary tests; format/model checks; required-file and corruption recovery cases; parent-oracle SIGKILL recovery; shutdown and resource-cap tests; OCI build/test; short or long performance matrices.

The performance stage remains prohibited until implementation and every required functional/recovery validation pass.

## Segmented recovery corrections and synchronous checkpoint milestone

Source identity:

- Starting reviewed source: `d73ff0d01018800fb68c49453bc17fcc44fa316d`.
- Review corrections: `c13e86e6793b8cea69747338093338bde255bb1b`.
- Checkpoint/reclamation implementation: `3d883db0c75b51f31463f197eefe53157974bf0a`.
- Branch and worktree remain the ones named above. All Rust sources in the final test invocation match the implementation commit; `evidence/segmented-checkpoint/source-sha256.json` records seven relevant source files.
- Local Rust/Cargo 1.97.1 on Darwin ARM64. No OCI build, execution, benchmark, or Git history rewrite was performed for this milestone.

Validation and preserved evidence:

| Command or check | Result | Evidence |
| --- | --- | --- |
| `cargo test --locked -p dodb-storage -p dodb-server` after review corrections | 292 passed, 4 ignored | `corrections-full-tests.log.gz` |
| Initial full checkpoint test run | 300 passed, 4 ignored | `checkpoint-initial-full-tests.log.gz` |
| Final `cargo test --locked -p dodb-storage -p dodb-server` | 301 passed, 0 failed, 4 ignored; storage library 202 passed | `checkpoint-final-full-tests.log.gz` |
| Final `cargo check --locked -p dodb-storage -p dodb-server` | Passed; existing experiment-binary dead-code warnings remain | `cargo-final-check.log.gz` |
| Final `cargo fmt -p dodb-storage -p dodb-server -- --check` | Passed | `cargo-final-format.log.gz` |
| `git diff --check` | Passed | Rechecked before commits |
| `cargo clippy --locked -p dodb-storage -p dodb-server --lib -- -D warnings` | Failed on 12 diagnostics in pre-existing code | `clippy-strict.log.gz`, `clippy-baseline.json` |

Evidence files in this table are under `evidence/segmented-checkpoint/`. Gzip files preserve the original command output bytes. `SHA256SUMS`, `summary.json`, and test-executable hashes accompany them. No Clippy suppression was added. The remaining strict diagnostics include existing argument-count, nested-if, needless-deref, and redundant-slicing findings; their primary source lines are all present in the reviewed baseline. Strict Clippy is not reported as a passing check.

The first checkpoint test fixture used too few overflow-backed entries to force a leaf split. Its split assertion failed, and the fixture was increased to 192 entries while retaining the assertion. The final run verifies an actual split; this was a fixture correction, not a removed invariant. A later review found that inspecting an obsolete torn INIT could mutate that unlisted file. Closed/read-only opening now refuses incomplete INIT instead, and the added preservation regression is included in the final 301-test run. Earlier passing runs are retained separately.

Coverage specific to the implemented milestone:

- Eight active-WAL truncation cases: partial page header, complete uncommitted page sequence, partial commit header, and partial commit tail, each before and after a nonzero checkpoint. Every case verifies acknowledged state, new successful writes, another recovery, later rotation, and another recovery.
- Four successive writes across three rotations validate required WAL bytes against actual file lengths, cumulative batch/sync/page/write counters, and reopen counter semantics. Reclamation tests also verify cumulative counters do not reset when covered files are deleted.
- Default service layout creates the legacy `.wal`; explicit segmented selection writes and reopens through the public transaction service. Zero segment size and use of the B-link layout by the B-tree service are rejected.
- Repeated synchronous checkpoints force a new active file, durably advance C, remove only covered closed files, leave one required active file, preserve global next LSN/batch identifiers, and make an idle checkpoint a no-op.
- Actual root growth, immutable pinned reads, overwrite/delete/recreate ABA, multi-key conditional transactions, tombstone revisions, post-reopen checkpointing, and stale WAL-superblock checkpoint fields are covered.
- Twelve rotation fault-injection boundaries and 19 checkpoint fault-injection boundaries preserve acknowledged state and permit successful recovery, a later write, checkpoint, and another recovery. Failures make subsequent writes fail closed. Missing required files are rejected even when obsolete files are present. Unknown/torn unlisted files remain unchanged.
- `segmented_checkpoint_sigkill.rs` has one child helper and one parent test which executes ten distinct real SIGKILL cases: after a page write; before/after data sync; after metadata sync; before/after checkpoint manifest publication; before/after unlink; and before/after reclamation-directory sync. The parent compares values derived independently from deterministic inputs and revisions from successful-return records, then performs another write, checkpoint, and reopen. This is ten crash scenarios, not ten separate Cargo test functions. The older segmented rotation SIGKILL test and existing legacy crash suites also pass.

Implemented scope ends at synchronous checkpoint advancement and reclamation. Background worker snapshots, write/checkpoint overlap, concurrent manifest update coordination, retention-cap backpressure, workload CLI selection, the full background format/model gate, OCI validation, and performance matrices remain unimplemented or unrun. Existing legacy defaults and historical benchmark evidence are preserved. Process SIGKILL is not a power-loss test.
