# Reproduction and evidence layout

The measured source is `d0f93e1e8b6ce54221d413897724a36ddd6725da`.
The initial `7b839c60d` build was superseded before workloads: shared mode
should retain its existing inactive read-two/write-six semaphores, restored
in `6331aae31`. The capped-latency matrix at that second source was stopped
when the driver's biased sampler was identified. Both builds and the partial
attempt are preserved and excluded from acceptance. `d0f93e1e8` records all
latencies and is used for every accepted smoke and measured run. The service
library is identical between the second and third source commits.

## Build

On `opc@217.142.246.204`, from the clean measured checkout:

```sh
DODB_BUILD_RUN_ID=read-write-admission-20261016-build3 \
  bash dodb/scripts/run-oci-rocksdb-build.sh \
  build --locked --release -p dodb-rocksdb --bin service_workload
```

Copy the executable to an immutable task path before running measurements.
The helper uses the pinned RocksDB archive and compatible shared Cargo target;
its captured environment records the source, dependency, toolchain, compiler
flags, cache, command, timing, and hashes. These builds do not run unit or
integration suites.

## Run

```sh
PYTHONDONTWRITEBYTECODE=1 python3 dodb/scripts/run-rocksdb-read-write-admission.py \
  --binary /bench/zfs/db/dodb-read-write-admission-20261016/binaries/service_workload-d0f93e1e8 \
  --result-root /bench/zfs/db/dodb-read-write-admission-20261016/measurements-full-latencies \
  --source-sha d0f93e1e8b6ce54221d413897724a36ddd6725da
```

The runner requires a new result directory, the authorized host, ARM64, ZFS,
and a clean exact source SHA. It records six smoke runs, 45 primary OFF runs,
and 15 diagnostic ON runs. Every run has a separate fresh database and saves
its exact command, report, output, exit status, binary hashes, elapsed time,
and validation status. Failures stop execution and remain on disk.

Accepted raw evidence is in `oci/measurements`, `oci/matrix-full-latencies.log`,
and `oci/builds`. The stopped attempt is in `oci/measurements-capped-latencies`,
`oci/matrix.log`, and `oci/capped-latency-attempt-rejection.txt`. OCI paths for
the accepted matrix contain `measurements-full-latencies`; files are copied
unchanged into the local `oci/measurements` directory.
Database directories, executables, checkout contents, and temporary build
files stay on OCI and are not copied into this repository. Raw files are
preserved byte for byte. External Cargo logs contain their original whitespace;
source and derived artifact whitespace checks exclude those raw logs.
`SHA256SUMS` covers the local evidence and derived artifacts with paths
relative to this directory, excluding only itself.

## Summarize

From the repository root:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 \
  dodb/docs/experiments/results/dodb-service-read-write-admission-20261016/summarize.py
```

The summarizer checks successful completion, configuration and workload
identity, before/after executable hashes, replay/reopen results, metrics
counts, write-category and condition reachability, complete process sampling,
and matching schedules/per-client input prefixes. It reports OFF medians and
ranges separately from single-repetition ON diagnostics and evaluates the
predeclared descriptive decision screen in `PLAN.md`.
