# Reproduction

Use the designated `opc@217.142.246.204` host with `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes` and ordinary host-key validation. The experiment root is `/bench/zfs/db/backend-hotpaths-20261026`. The control remains the original `/bench/zfs/db/blink-checkpoint-costs-20261023` checkout and frozen binaries at `cc80d1fdc7a98792be1a739056a3013debf39268`. The candidate source is frozen at `3b94ee88eb4e7c02416cde14d3ac48c55e45b991`.

The controller scripts live outside the frozen candidate checkout. Their recorded hashes identify the exact scripts used. Repository commits after the frozen candidate add experimental controls, reduction and reporting without changing the measured engine paths.

## Build and functional validation

Prepare a fresh unique root with `source` pointing to a clean detached candidate checkout, and place this experiment's scripts under `controller`. Keep the original control root unchanged. Copy `blink-background-checkpoint-20261021/performance-loss-20261022/analyze_loss.py` from the candidate's experiment results directory into `controller/accounting-helper.py` for independent offline reduction.

```sh
PYTHONDONTWRITEBYTECODE=1 \
DODB_HOTPATH_ROOT=/bench/zfs/db/backend-hotpaths-20261026 \
python3 /bench/zfs/db/backend-hotpaths-20261026/controller/run_pipeline.py build
```

The build uses the existing monitored OCI helper, Rust 1.97.1, ARM64 `+crc`, pinned RocksDB 11.8.1 and the shared build-cache lock. It records source/build identities, commands, executable hashes and full-window build resource samples. It then runs the existing twenty-executable functional suite on ZFS, including the added cache and lock regressions.

The recorded build stage uses protected run IDs derived from the frozen source revision. It refuses to overwrite those IDs on the original host. For another build of the same revision, choose a unique `DODB_BUILD_RUN_ID` and invoke `dodb/scripts/run-oci-rocksdb-build.sh` with the identical Cargo arguments recorded in the archived build command/environment files. Reuse the frozen source and executable payloads for a measurement-only repetition under a new evidence/database root. Offline verification needs neither a build nor workload execution.

## Performance comparison

After functional validation passes:

```sh
PYTHONDONTWRITEBYTECODE=1 \
DODB_HOTPATH_ROOT=/bench/zfs/db/backend-hotpaths-20261026 \
python3 /bench/zfs/db/backend-hotpaths-20261026/controller/run_pipeline.py measure
```

The controller runs four candidate preflights, twenty-four 60-second short cells and twenty-four 300-second sustained cells. Each non-preflight stage compares old/new B-link and old/new minimal-native RocksDB, with three rotated repetitions and both RW80 and E. Preflight uses one-second warmup and eight-second admission; the other stages use ten-second warmup. Workload drivers, client counts, synchronous WAL, transaction/revision semantics and checkpoint parameters remain fixed as declared in [the plan](PLAN.md).

The runner preserves raw and enriched reports, hashes, commands, external process-group resource samples, complete latency counts, state/reopen validation, input-prefix fingerprints and checkpoint evidence. Completed case databases are archived losslessly between cells. Each archive is decoded and checked against every original database file's size and SHA256 before only that task case's original directory is removed. Archive work never overlaps a measured workload. Archives and their sidecars remain under each stage's `database-archives` directory on the host. Existing experiment databases are untouched.

For this run, the operator sent SIGSTOP only to the parent pipeline while its short-matrix child continued normally. After that child exited, the operator ran `relocate_archives.py` and then sent SIGCONT to the parent. The runner does not create this pause automatically. This external stage barrier prevents archive relocation from overlapping workloads.

`relocate_archives.py` copies verified new preflight/short database archives to `/home/opc/backend-hotpaths-20261026-database-archives`, verifies SHA256 again, synchronizes the destination and redirects only the original task archive paths with symlinks. It preserves at least 3 GiB free on that filesystem. The case databases themselves always run on the original ZFS dataset. No existing experiment data moves. `archive-relocations.json` records every destination and hash.

Failed/incomplete attempts are preserved. The runner can finish archival and accounting for a successful existing case without rerunning its workload. A failed case requires a new explicit attempt root; existing evidence is not overwritten. Never rerun accepted workloads solely to fix a reducer or report.

## Offline evidence

After all workloads and controllers exit, run `archive_evidence.py` with `DODB_HOTPATH_ROOT` set. The archive includes original reports, all resource/validation/build logs, controller snapshots, database archive metadata and a SHA256 inventory. Database payloads, frozen source and executable payloads remain on the host and are excluded from the Git evidence archive.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 \
  dodb/docs/experiments/results/backend-hotpaths-20261026/verify_results.py \
  dodb/docs/experiments/results/backend-hotpaths-20261026/evidence/backend-hotpaths-20261026-evidence.tar.gz
```

The verifier checks every inventoried file and reproduces all three original summary objects without executing workloads. It also checks cross-stage full-window input prefixes, source/binary identities, latency counts, resource/state/checkpoint gates and the functional test recount. Throughput is API calls per second in RW80 and successful width-16 batches per second in E. RSS covers the entire process group; reopen includes the full state oracle. These are not isolated engine-memory or pure recovery measurements.

The final offline result verifies 956 original files, 52 accepted workload cells, 343 passed functional tests with four existing ignored probes, the identical RocksDB archive and zero new package lint diagnostic occurrences. The short measurement windows span `2026-10-26T02:34:09Z` to `2026-10-26T03:09:32Z`; sustained windows span `2026-10-27T00:11:43Z` to `2026-10-27T02:25:53Z`. The recorded parent-stage barrier is excluded from per-cell performance timings, and durations are never pooled. See [measurement windows](evidence/measurement-windows.json).
