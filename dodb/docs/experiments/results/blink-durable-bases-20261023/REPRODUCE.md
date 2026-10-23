# Reproduction and provenance

## Source and environment

The primary-budget candidate is `400ad0ac6` on `experiment/blink-durable-bases-20261023`, in the fn0 monorepo. The prior strict combined-cap attempt is `7c048e2595b21b0597ca0477760fa56649f2521e` and remains a separate rejected preflight. The initial implementation build at `479d6fe92` is not performance evidence.

The designated host is `opc@217.142.246.204`, accessed using `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes`, and normal host-key validation. Root paths are:

- Rejected combined-cap attempt: `/bench/zfs/db/blink-durable-bases-20261023`.
- Primary-budget candidate: `/bench/zfs/db/blink-durable-bases-20261023-primary-budget`.
- Each root has its own detached clean `source` worktree and frozen `bin` directory.

The dataset is ZFS `dodbbench/db`, mounted at `/bench/zfs/db`, with `sync=standard`, `compression=off`, `recordsize=4096`, and `logbias=latency`. `source-environment.txt` captures the current CPU, kernel, filesystem, memory, toolchain and checkout. Build provenance captures Rust/Cargo 1.97.1, AArch64 `+crc`, the pinned RocksDB 11.8.1 source/archive, sccache, Cargo.lock, command, and source identity. RocksDB is linked for the shared workload driver; all new matrix paths execute B-link.

## Build and functional validation

From the clean source root, the recorded pipeline is:

```sh
PYTHONDONTWRITEBYTECODE=1 \
DODB_BASE_RUN_ROOT=/bench/zfs/db/blink-durable-bases-20261023-primary-budget \
python3 dodb/docs/experiments/results/blink-durable-bases-20261023/run_pipeline.py
```

The pipeline first runs the existing monitored OCI build with `test --locked --release -j 1`, selecting storage/server/RocksDB library tests, both workload binaries, existing transaction/read/history/transport/recovery tests, and all new durable-base tests. It separately builds the main workload executables, freezes them, then executes twenty selected test binaries with `--test-threads=1`. Both storage and server tests named `durable_base_checkpoint` are required; identities include the executable hash suffix to avoid dropping either one. Exact paths, hashes, commands and individual counts are in `validation/commands.json`.

Do not rerun the original pipeline in an existing root or reuse a build-run ID. It intentionally rejects overwrite. To rebuild on the same host, allocate new unique `DODB_BUILD_RUN_ID` values with `run_build.py` and the recorded Cargo arguments, or reuse the already frozen accepted executables for reporting. Never replace accepted cells because a reducer or document changed.

## Matrix

`run_matrix.py` uses the unchanged workload commands from the prior checkpoint controller, replacing only the named layout/execution path. RW80 uses sixteen dedicated callers, 80% reads, 512-byte values and width-16 workflows. E uses sixty-four callers and unconditional width-16 overwrite batches over the same prepopulated pool. Seed is 20261013. WAL synchronization, transaction conditions, revisions, group limits and client scheduling remain enabled and unchanged.

The four paths are ordinary segmented/sync, immutable bases/sync, coalesced immutable bases/sync, and coalesced immutable bases/background. Every path has a 128 MiB soft WAL checkpoint trigger and 32 MiB segment trigger. Candidate live bases count as primary database storage; additional WAL/duplicate/in-flight storage is bounded by 512 MiB beyond the live image. Snapshot memory remains bounded by 512 MiB. This is not a common complete-engine memory-budget experiment.

The preflight uses eight seconds after one-second warmup, one repetition per path/workload. It is excluded from throughput medians. The short matrix uses sixty seconds after ten-second warmup, three rotated repetitions, twenty-four cells. The selection and sustained gates were declared in ACCEPTANCE.md before accepted performance cells.

To resume a matrix, retain its existing root, binaries, plan, completed-case ledger and reports, and rerun its exact command. The runner skips accepted cases and rejects changed plans or identities. The source checkout must stay at the compile-time binary commit even when later result documents are committed elsewhere.

## Accounting

The independent reducer checks raw/enriched report hashes, source and executable identity, exact latency histogram counts, fixed input-prefix equality, full windows, zero unintended errors, final/reopen values and revisions, checkpoint counts, background overlap, and resource samples. WAL appended bytes equal ending retained WAL minus starting retained WAL plus reclaimed WAL; segmented rotation has no legacy reset-INIT correction.

`checkpoint_bytes_written` includes all page-pack/consolidation bytes or ordinary checkpoint data/metadata bytes. Linux process `wchar` differences independently include process-issued WAL, checkpoint and metadata write requests. These are not physical device write counts. `syscw` records write syscalls separately. Timing labeled reopen-and-validation includes backend opening and the final-state oracle, not only redo replay.

The recovery envelope is independently recorded per case in `recovery-envelope.json`, using the validated final allocated page image and the fixed extra budget. External samples include obsolete/orphan page packs, the materialized database cache, and total directory size. In-process admission enforces the corresponding current live-image bound before accepting mutations. The existing 3 GiB RSS, 4 GiB directory, 1500 MiB available-memory and 10 GiB free-space guards remain active.

## Retained failures

Local compilation/test failures and corrected regression runs are preserved under `evidence/`. The recovery-growth regression exposed missing dirty tracking for replayed pages before the next WAL reclamation; the fixed public path now preserves them. A readiness-file race in SIGKILL fixtures was fixed with atomic publication, and its failed logs remain available. The strict combined-cap preflight preserves its failed E database and reports on OCI; no row from that attempt enters the corrected candidate medians.

Archive verification checks the archive SHA256 and every inventoried original file byte for byte. Source trees, executable payloads and live/failed database directories are excluded from the OCI report archives; their source identities, hashes, commands and file inventories are retained, and failed databases remain on the host. The separate local rejected-fixture archive includes the selected small local database fixtures.

For 300-second runs, both workload drivers' internal 200 ms resource vectors reach their pre-existing 1024-sample caps. Their omitted-sample counters are preserved, and legacy completeness labels are not treated as proof of full-window resource coverage. Headline RSS and resource limits use the separate unbounded one-second external controller samples spanning the whole child process; report their count and maximum interval. Latency histograms are checked independently and must retain every completed observation.

## Independent archive reduction

The accepted archive is `evidence/blink-durable-bases-20261023-primary-budget-evidence.tar.gz`: 608 inventoried files, 321,275,178 uncompressed bytes, SHA256 `2960f164a80361c5bb6b295434b6d76b817085d0548ac5751bf5c9549742cfae`. The rejected-preflight archive is `evidence/blink-durable-bases-20261023-evidence.tar.gz`: 113 files, 14,212,675 uncompressed bytes, SHA256 `808de61f7318d8493db8d6bfdf7627c3c7829a188242c570ed723dce6d70ced6`. Its provenance includes the initial `479d6fe92` build as well as `7c048e259` builds. Sidecars record compressed sizes and exclusions. `preserved-oci-database-inventory.json` records the six unchanged failed-database files left on the host.

From `dodb/`, reproduce the offline verification and augmented summaries with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 \
  docs/experiments/results/blink-durable-bases-20261023/verify_results.py
```

This reads only archived evidence and local accounting code; it does not rerun workloads. It verifies all 721 archived file hashes, reduces the twenty functional-test logs, checks all 44 accepted cells, compares fixed input prefixes across stages, and requires every original short/sustained summary value and the frozen selection hash to match. Original archived summaries remain unchanged; separately published summaries add CPU, API-specific latency and sampling-coverage fields. There was no original preflight summary, so that stage's original-summary comparison is inapplicable. `independent-verification.json` records the result and every external sampling window.
