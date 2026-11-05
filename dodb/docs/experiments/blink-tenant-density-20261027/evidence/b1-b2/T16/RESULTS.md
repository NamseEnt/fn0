# T16 Results

Status: complete. Runtime paging remains opt-in and default-disabled. Implementation source: `0d7c59a3092c79632590e5faffaec9adcf629a10`.

## Instrumentation and validation

The measurement worker now records periodic reachable logical source file bytes and glibc allocator retained free arena bytes alongside the existing bounded process/cgroup/resource observations. Reachability uses the existing source manager walker and reports unique reachable checkpoint-pack and WAL-segment logical lengths. Allocator retained bytes are read directly from `mallinfo2.fordblks`; allocated arena bytes and mmap-allocated bytes are reported separately. These fields do not represent allocated filesystem blocks, device writes, total RSS, or allocated heap.

The release worker build used RocksDB commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654` and the pinned native library. The new 4-tenant, 400-operation paging instrumentation smoke completed with matching requested/compiled source identity and verified the paged route. It emitted 27 measurement-periodic samples at a 100 ms cgroup sampling interval. All 27 contained both measurements: reachable source logical bytes ranged from 0 to 3,186,812 across four files at the measurement boundary; glibc retained free arena bytes ranged from 304,160 to 729,008. The zero-byte reachable sample is retained as an observed setup/measurement boundary value, not replaced with an estimate. Post-measurement and reopen full-population oracles passed. This is instrumentation/correctness evidence, not SLO or performance evidence.

The optimized build and runtime smoke passed. Strict package Clippy commands remain blocked by existing findings in the storage and rocksdb library modules; the captured diagnostics contain no finding in either changed workload binary file, and no lint suppressions were added.

Previously accepted resident/paging correctness evidence remains unchanged: matched request/schedule fingerprints, 400/400 operations, full-population and reopen oracles. It was not rerun except for the candidate instrumentation smoke required to validate the new periodic fields.

## Frozen T17 handoff and capacity

The separate T17 matrix is frozen at `T17-FROZEN-MATRIX.md`. The prior 266 GiB estimate double-counted setup and checkpoint overlap already contained in the 32 GiB per-backend global disk cap and assumed parallel backend setup. The corrected sequential worst case is 32 GiB for the single active backend including overlap, 5 GiB for reports/manifests and one preserved failed cell, plus the 10 GiB ZFS reserve: 47 GiB required free space.

The latest `/bench/zfs/db` snapshot after T16 build, smoke, and Clippy is 11,463,950,336 bytes free (10.68 GiB), 726,532,096 bytes above the reserve and 36.32 GiB below the corrected T17 gate. T17 remains unavailable and was not launched. Recheck capacity before any separately authorized T17 execution.

## Authorized ZFS reclamation

Before cleanup, the three failed build roots contained regenerable Cargo `target` trees sized 1,793,584,640, 1,793,528,320, and 3,369,096,704 bytes. Only those three `target` directories were removed. Source snapshots, T16 reports/logs, failed and successful databases, frozen binaries, and pinned RocksDB native artifacts were preserved. The dataset free-space snapshot increased from 10,797,957,120 to 17,304,760,320 bytes after cleanup settled. Subsequent T16 build and smoke used ZFS space; latest settled availability is recorded above.

## Preserved follow-up evidence

- `remote/instrumentation-0d7c59a/paging-report.json`: successful periodic instrumentation smoke, full and reopen oracles, cgroup samples, and resource summaries.
- `remote/instrumentation-0d7c59a/paging-run.log`: successful smoke run trace.
- `remote/instrumentation-0d7c59a/build-release-final.log` and `SHA256SUMS`: final release build and binary identities.
- The debug readiness timeout, cgroup permission failures, and source-revision mismatch reports/logs remain in the same evidence directory; their failed databases and worker roots remain on OCI.
- `logs/oci-capacity-reclamation.log`: pre-cleanup inventory, exact removed paths, and post-cleanup capacity.

T17 was not executed. Defaults were not changed and issue #126 remains open.

## T17 readiness record correction

The later T17 review found that the frozen plan's 30-minute recovery workload, five-minute SLO windows, checkpoint-cycle extension, long-reader retention, and stress-case recipes were not supported by the then-frozen harness and matrix. The T17 receipt incorrectly represented that preflight gap as an adoption decision of evidence insufficient. No T17 cells had run, so adoption was not evaluated. This correction does not change T16 completion, its historical capacity readings, or its evidence. T17 has been reopened for the separately authorized implementation, matrix revision, validation, and re-freeze.
