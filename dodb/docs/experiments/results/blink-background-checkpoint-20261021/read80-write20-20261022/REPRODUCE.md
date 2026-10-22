# Reproducing the read-heavy follow-up

## Frozen implementation

The measured engine is unchanged from the accepted concurrent-checkpoint matrix:

- Source: `d240416f0e0efafa7c08bb1e868387ebd0d894bc`.
- Frozen `service_workload` SHA256: `09bd505d77ce0bcbc61ca46f7389373e60bc1ab1bf6b80a90077bed328418e40`.
- Initial controller/plan commit: `061ccc9c9cc2bdbb4e5e7b7bfa2d29c639ecf240`.
- OCI host: `opc@217.142.246.204`; authenticate with `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes`, and normal host-key validation.
- Main root: `/bench/zfs/db/blink-read80-write20-20261022`.
- Separate preflight root: `/bench/zfs/db/blink-read80-write20-20261022-smoke`.
- Shared frozen binary and source aliases resolve through `/bench/zfs/db/blink-background-checkpoint-20261022-threshold/{bin,source}`. Verify the actual clean source revision and executable digest; directory names are not source identity.

The engine's existing 314-passing OCI validation belongs to the preceding implementation gate, not a new test execution. This follow-up validates the unchanged executable through three preflight cases and every measured case's full-window, outcome, final-state, and reopen checks.

## Controller and commands

Allocate fresh run roots for a new experiment. The saved roots are immutable evidence and must not be overwritten. Supply `source` and `bin` aliases to the verified implementation checkout and frozen executable. Copy the preceding `run_matrix.py` into `controller/run_matrix_base.py`, and this directory's runner and summarizer into `controller/`. The wrapper reuses the original resource-sampling, raw-output, validation, preservation, and database-manifest controller. Its workload-specific validation replaces only the previous stress requirement for three measured checkpoints: it records actual checkpoint frequency without lowering thresholds in a read-heavy workload.

The run plan records both controller script SHA256 values independently from the engine commit and executable SHA256. The wrapper rejects a dirty implementation checkout, a different source commit, or a different executable before starting any case.

```sh
PYTHONDONTWRITEBYTECODE=1 \
DODB_READ80_RUN_ROOT=/bench/zfs/db/blink-read80-write20-20261022 \
DODB_READ80_SMOKE=0 \
DODB_READ80_CONTROLLER_SHA=061ccc9c9cc2bdbb4e5e7b7bfa2d29c639ecf240 \
DODB_READ80_BASE_RUNNER=/bench/zfs/db/blink-read80-write20-20261022/controller/run_matrix_base.py \
python3 -u /bench/zfs/db/blink-read80-write20-20261022/controller/run_read80.py
```

Preflight sets `DODB_READ80_SMOKE=1` and its separate root. It runs three paths for eight seconds after a one-second warmup. Main measurement runs all five paths, three repetitions, 30 seconds after a ten-second warmup. The scenario is `rw80`; all other workload and engine settings match the preceding mixed-workload commands. Complete argv for every case is saved in `reports/<case>/command.json`.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 controller/summarize_read80.py \
  /bench/zfs/db/blink-read80-write20-20261022
```

The summarizer independently checks the expected cases, source/binary/controller identity, raw and enriched report hashes, exact histogram sample counts, observed read ratio, fixed 16-client input prefixes, resource guards, outcomes, final/reopen state, effective checkpoint layout/execution, and drained checkpoint ownership. It reports per-run metrics and medians, including separate Get, Query16, successful-write, and workflow percentiles. Intentional condition rejections are retained separately from request errors and natural conflicts.

## Archive and offline verification

Capture environment and clean source state before and after measurement. Once both summaries pass, archive the two roots while excluding binaries, source trees, and removed successful databases. Preserve database filename/size/SHA256 manifests and RocksDB logs. The archive includes controller files, summaries, raw reports, exact argv, resource samples, preflight, and SHA256 inventories.

```sh
python3 archive_read80.py \
  /bench/zfs/db/blink-read80-write20-20261022-evidence.tar.gz \
  /bench/zfs/db/blink-read80-write20-20261022 \
  /bench/zfs/db/blink-read80-write20-20261022-smoke

python3 verify_read80.py evidence/blink-read80-write20-20261022-evidence.tar.gz
```

Offline verification checks archive/file inventories, safely materializes regular files, and reruns each summarizer without SSH, source checkout, or benchmark binaries. Recomputed summaries must match byte for byte. Accepted cases are never rerun solely to repair aggregate reporting.
