# Validation Record

## Source and preserved state

- Starting source SHA: `a5b37c23ab72d95c8d5d7bed263459fd7af141e2`.
- Branch: `experiment/blink-wal-reclamation-20261020`.
- Worktree: `/Users/namse/fn0-dodb-wal-reclamation`.
- Implementation and process-termination test SHA: `876d5a1b4b7c0b2792efc27237fa3367cd8d1d9d`.
- The existing `/Users/namse/dodb`, `/Users/namse/fn0-dodb-rocksdb-baseline`, and the prior baseline result directory were left unchanged.

## Local validation

- `cargo fmt -p dodb-server -p dodb-storage -p dodb-rocksdb -- --check`: passed.
- `git diff --check`: passed.
- `cargo test -p dodb-server`: passed, including six B-link checkpoint integration tests and the existing server suite.
- `cargo test -p dodb-storage --lib page_delta_checkpoint_fault_matrix_preserves_acknowledged_writes`: passed. This reuses the physical checkpoint fault matrix covering page write/sync, checkpoint metadata write/sync, and WAL reset boundaries.
- `cargo test -p dodb-server --test blink_wal_checkpoint`: passed 6/6, including default-disabled behavior, threshold behavior and repeated reset, policy rejection, fail-closed handling, reopen/revision preservation, and process termination during checkpoint.

The SIGKILL test commits and acknowledges a transaction, blocks at `before_checkpoint_data_flush`, kills the child process, then reopens the same database and verifies WAL replay. This is real process termination evidence, separate from function-level fault injection. It is not a host-reset or power-loss test.

The local RocksDB build dependency had source files but incomplete Git metadata and no `librocksdb.a`, so local `dodb-rocksdb` tests could not compile. A separate local fetch attempt was interrupted by a remote connection reset; the existing dependency path was not changed.

## OCI build and contract tests

The approved OCI host was connected using `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes`, and normal host-key validation. It reported Linux AArch64, Rust/Cargo 1.97.1, ZFS at `/bench/zfs/db`, pinned RocksDB source `abeebd9630f11bd08c28b7bd43c7bdfc62050654`, and archive SHA-256 `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.

The final source SHA build used `CARGO_BUILD_JOBS=1` and the repository OCI helper. Both release workload binaries built successfully. The monitored build reached 126,709,760 bytes process-group RSS, minimum 5,113,733,120 bytes `MemAvailable`, and minimum 49,102,462,976 bytes filesystem free. The earlier unconstrained build attempt exceeded the 3 GiB process-group RSS stop line and is preserved; it was not used for measurement.

At the final source SHA, the following OCI tests passed serially:

- `cargo test --locked --release -p dodb-rocksdb --test transaction_contract --test read_contract --test contract_history -- --test-threads=1`: 9 tests passed, including B-link with automatic threshold checkpoints, minimal-native, and RocksDB common transaction/read/history contracts.
- `cargo test --locked --release -p dodb-server --test blink_wal_checkpoint -- --test-threads=1`: 6 tests passed, including SIGKILL recovery.

Each OCI build/test run was monitored for process-group RSS, `MemAvailable`, and filesystem free space. Build and workload execution did not overlap. The first two failed build-controller invocations and the prior SHA's successful build/test records remain under `oci/builds/` and `oci/helper-runs/` as historical evidence.

## Performance validation

The predeclared matrix ran 24/24 cells: minimal-native, B-link off, B-link 32 MiB, and B-link 128 MiB; A/rw20 and E/width-16 unconditional; three independent repetitions. Each report records a successful full measurement window and passing final/reopen validation. The exact report set and per-run metric checks are recorded in `oci/matrix-verification-final-v3.json`.

The first controller attempt stopped after the first completed cell because its A fingerprint parser read `client_input_prefixes` at the wrong JSON nesting level. A resumed attempt then rejected serialized matrix-plan keys before launching a workload. Both attempts and the first successful cell were preserved; that cell was not rerun. The final full run completed all cells, after which its equality check mistakenly included `admitted_workflows`, a throughput-dependent count, alongside the actual input fingerprint. The failed comparison is retained in `oci/input-prefix-fingerprints.json` and controller-failure records. The separate final verifier compares only per-client input-prefix fingerprints and fixed prefix lengths; all fingerprints matched, all 24 cells passed their controller and validation checks, and no cell was silently replaced.

For A, checkpoint event wall timestamps were compared against the 30-second measurement interval using the workload's timestamped process samples. For E, whose process sampler has no wall-clock timestamps, the conservative in-measurement count subtracts one possible final queued coordinator group from post-reset checkpoint completions. E's threshold runs still have at least six conservatively counted measurement checkpoints. All 12 threshold workload/repetition cells therefore satisfy the three-checkpoint observation requirement.

Full stdout, stderr, raw/enriched JSON, per-run commands and settings, process/resource time series, checkpoint events, database-file manifests and hashes, RocksDB LOG files, completion records, and controller failure artifacts are under `oci/`. The source SHA and executable hashes are in the matrix plan, commands, and reports.

The 30-second results answer the short-run comparison only. They do not establish long-term steady state. The memory experiment remains a follow-up; these data do not show equal-memory performance or page-cache eviction/reload behavior.

## Measurement correction validation (2026-10-21)

Correction implementation source: `817f4173e90b9a6c0dbae0fde263ea3876915719`. Performance measurement provenance remains the original `876d5a1b4` source and original binaries.

Local checks:

- `cargo test --locked -p dodb-storage --lib wal::tests:: -- --test-threads=1`: 24 passed, including physical and logical WAL reset counters checked against actual file sync calls, repeated resets, and failure after the first sync.
- `cargo test --locked -p dodb-server --test blink_wal_checkpoint -- --test-threads=1`: 6 passed, including SIGKILL recovery and the strengthened exact commit/checkpoint sync assertions.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s dodb/docs/experiments/results/blink-wal-reclamation-20261020 -p test_measurement_corrections.py -v`: 5 passed. These cover numeric RocksDB WAL classification, diagnostic-log exclusion, manifest accounting, nonzero/submillisecond checkpoint drain, validated historical sync correction, conservative measurement checkpoint counts, and unavailable peak WAL values.
- `cargo test --locked -p dodb-storage --lib page_delta_checkpoint_fault_matrix_preserves_acknowledged_writes -- --test-threads=1`: 1 passed after the counter correction.
- `cargo fmt -p dodb-storage -p dodb-server -p dodb-rocksdb -- --check`: passed.
- The corrected verifier rechecked all 24 existing cells, their report hashes, fixed input fingerprints, and repeated checkpoint evidence. All 367 original OCI evidence hashes are unchanged. The new output is `oci/matrix-verification-corrected-v4.json`; prior summaries and original reports remain intact.

OCI validation used the same approved AArch64 ZFS host, pinned Rust 1.97.1, RocksDB source/archive, and shared build cache as the original matrix. A clean detached worktree at correction source `817f4173e90b9a6c0dbae0fde263ea3876915719` was created separately from the original measurement checkout.

- The monitored helper build used `CARGO_BUILD_JOBS=1` and `cargo test --locked --release -p dodb-rocksdb --bin service_workload --bin unconditional_write_workload --test measurement_corrections --test transaction_contract --test read_contract --test contract_history --no-run`. It passed in 299.8 seconds, with 1,897,218,048 bytes maximum process-group RSS, minimum 3,181,490,176 bytes host MemAvailable, and minimum 46,560,485,376 bytes filesystem free.
- The same test targets then ran with `-- --test-threads=1`: 24 passed (9 service driver unit tests, 5 unconditional driver unit tests including final checkpoint drain, 4 history tests, 1 WAL classification regression, 3 read contracts, and 2 transaction contracts). The test controller passed with no stop condition; its process-group peak RSS was 126,529,536 bytes.
- Build and test execution did not overlap. The helper's copied SHA256SUMS were independently checked after transfer.

Complete build/controller logs, helper provenance and hashes, resource samples, test commands, and raw test output are recorded under `corrections/oci-build/`, `corrections/oci-helper-run/`, and `corrections/oci-tests/`. Local command results are summarized in `corrections/local-validation.json`. These records are separate from the original measurement evidence. No new performance matrix or memory-limit experiment was run.
