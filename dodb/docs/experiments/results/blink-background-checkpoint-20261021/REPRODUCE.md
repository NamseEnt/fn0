# Reproducing the concurrent checkpoint comparison

## Source and environment

- Implementation checkout: `d240416f0e0efafa7c08bb1e868387ebd0d894bc`.
- Branch: `experiment/blink-background-checkpoint-20261021` in the fn0 monorepo.
- Host: `opc@217.142.246.204`, using `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes`, and normal host-key validation.
- Final run root: `/bench/zfs/db/blink-background-checkpoint-20261022-threshold`.
- Preserved initial run root: `/bench/zfs/db/blink-background-checkpoint-20261022`, source `58a197a1711a51f76188db4e9bc314fc89bbf5c1`.
- Filesystem: `/bench/zfs/db`, ZFS dataset `dodbbench/db`; inspection reports `sync=standard`, `compression=off`, `recordsize=4096`, and `logbias=latency`.
- Rust/Cargo: 1.97.1; AArch64 GNU, release profile with existing debug information and `-C target-feature=+crc`.
- RocksDB: 11.8.1, source `abeebd9630f11bd08c28b7bd43c7bdfc62050654`, archive SHA256 `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.

The OCI source is a real detached Git worktree populated through verified Git bundles. The final root's `source` symlink points to the initial root's `source` worktree, which was advanced from the initial commit to the final commit between matrices. The initial frozen binaries and their compile-time identity remain unchanged; the initial matrix does not depend on the current HEAD of this reused source directory. The source path is recorded by every build and workload controller. Raw workload JSON contains compile-time `git_commit`; the controller rejects a mismatch with the checkout. Raw and enriched reports, executable hashes, commands, final/reopen validation, and resource samples are retained separately.

The approved shared target path is a symlink. Its canonical target is `/bench/zfs/db/dodb-rocksdb-r1b3a-551471d5/target/release`. The directory name is a cache identity rather than the implementation source. Validation resolves the alias and verifies executable hashes. Frozen workload executables under the run root's `bin/` are used throughout the matrix, so subsequent cache activity cannot replace measured executables.

## Build and functional gate

Run commands from the pinned monorepo source root. These scripts preserve the exact original run paths and IDs; for a new execution, allocate fresh run/build/test directories, update the helper constants and run ID, and freeze the newly verified executables under that new root. Existing completed/failed roots must remain intact. The matrix resolves `RUN_ROOT/source` and `RUN_ROOT/bin`; supply the pinned clean source checkout and its frozen executables there.

The preserved monitored command uses `run_oci_build_monitored.py` with the existing `dodb/scripts/run-oci-rocksdb-build.sh`. The final Cargo arguments are:

```sh
test --locked --release -j 1 -p dodb-storage -p dodb-server -p dodb-rocksdb \
  --lib --bin service_workload --bin unconditional_write_workload \
  --test background_checkpoint --test blink_wal_checkpoint \
  --test background_checkpoint_sigkill --test segmented_checkpoint_sigkill \
  --test segmented_wal_sigkill --test phase2_recovery \
  --test process_crash_recovery --test phase6_quic \
  --test transaction_contract --test read_contract --test contract_history --no-run
```

`run_oci_validation.py` reads the resulting executable list, verifies canonical paths and hashes, and executes 16 selected binaries with `--test-threads=1`. Temporary/test databases live on ZFS. Direct test execution supplies `CARGO_BIN_EXE_phase2-crash-child` and `CARGO_BIN_EXE_process-crash-recovery-child`, which Cargo would normally supply. Its resumable execution preserves successful binary runs and retains earlier failed launcher attempts. See `VALIDATION-CONCURRENT.md` for counts and failure boundaries.

## Workload matrix

`run_matrix.py` contains the exact commands and rotated path orders. Its five paths are native RocksDB with the DODB transaction contract, legacy synchronous checkpoint at 128 MiB, segmented synchronous checkpoint at 128 MiB, and segmented background checkpoint at 32 MiB and 128 MiB. Every segmented path uses 32 MiB WAL segments and 512 MiB encoded-image/retained-WAL admission ceilings. A uses 16 dedicated callers, RW20, and width-16 workflows enabled; E uses 64 dedicated callers and unconditional width-16 transactions. Both use 512-byte values, seed 20261013, reserved revision ranges, exclusive condition locks, and real synchronous WAL durability.

The short comparison has ten-second warmup, 30-second admission windows, and three repetitions: 30 fresh database cells. Final request completion and checkpoint drain remain in the throughput denominator. Percentiles count every completed sample in the existing log1024 histograms. Fixed input prefixes must agree across paths; throughput-dependent admitted totals are not compared for equality.

```sh
DODB_CHECKPOINT_RUN_ROOT=/bench/zfs/db/blink-background-checkpoint-20261022-threshold \
DODB_CHECKPOINT_SMOKE=0 \
DODB_CHECKPOINT_MEASUREMENT_MS=30000 \
DODB_CHECKPOINT_WARMUP_MS=10000 \
PYTHONDONTWRITEBYTECODE=1 \
python3 dodb/docs/experiments/results/blink-background-checkpoint-20261021/run_matrix.py
```

The initial source completed an eight-second/one-second-warmup preflight using the separate `smoke/` root and `DODB_CHECKPOINT_SMOKE=1`; it is not a 30-second performance repetition. Its `bin` symlink still identifies the initial frozen executables. Its source alias now resolves to the reused final checkout; preserved raw compile-time identity records the source actually executed.

The controller samples process-group RSS, CPU, available host memory, database/WAL file bytes, and free filesystem space every second. Its sampled stop lines are 3 GiB RSS, 4 GiB database directory, 1500 MiB available host memory, and 10 GiB free filesystem space. These are observed stop guards rather than cgroup memory caps. Successful databases are removed after final/reopen checks and a filename/size/SHA256 manifest; failed databases remain intact.

`summarize_matrix.py RUN_ROOT` independently checks report/binary/source hashes, full latency histogram counts, expected cells, fixed input prefixes, outcomes, final/reopen state, resource bounds, zero remaining work, repeated measured checkpoints, and actual acknowledged write overlap. Its event window is the recorded dispatch-to-observed-last-client-completion interval; explicit final checkpoint drain is excluded from repeated-checkpoint counting. Mutation-path WAL sync counts include automatic segment-initializer syncs; checkpoint-path counts include forced-rotation initializer syncs. Manifest, directory, and database-file syncs are separate operations and are not included in these WAL-file counters.

## Interpretation and retained attempts

The segmented sync/background comparison holds the physical format and snapshot operation constant. Comparing background with legacy synchronous execution also includes the segmented layout, rotation, and manifest/reclamation differences. Group sizes, sync counts, outcomes, and phase timing remain observable; aggregate throughput alone does not isolate an individual cost.

The comparison imposes no common complete-engine memory budget and does not establish a maximum-RocksDB or production steady-state result. The 300-second follow-up is gated by at least 10% median throughput improvement and no worse median p99 in both workloads for background 128 MiB versus segmented synchronous 128 MiB, in addition to correctness/resource gates. Whether that follow-up runs is recorded with the final results.

The initial 30-cell matrix is retained in full. It exposed automatic coalesced requests that survived reclamation below the configured threshold. A deterministic regression failed on that source and passed after correction. The final source reruns all 30 cells with unchanged workload, thresholds, retention budgets, and invariants. Rejected build checks, launcher attempts, and corrected derived reporting remain separate evidence. A reporting correction does not replace a successful cell. Gzip/tar archives preserve original output bytes; verify their SHA256 manifests before interpreting results.

## Archived evidence

[EVIDENCE.md](EVIDENCE.md) lists complete initial/final report archives, build provenance, original-output checksums, local/model evidence, and offline rechecking. `capture_oci_environment.py` and `archive_oci_evidence.py` execute after measurement. The corrected matrix passed all 30 cells; its predeclared long-follow-up gate failed, so the 300-second follow-up did not run.
