# T17 Readiness and Capacity Results

Status: the frozen T17 first screening cell completed with an operational setup failure; the runner stopped the matrix under the first-failure rule. The earlier readiness record omitted harness and recipe prerequisites and incorrectly described the unstarted adoption decision as evidence insufficient; that record is corrected below.

## T16 completion

T16 is complete at implementation source `0d7c59a3092c79632590e5faffaec9adcf629a10`, with completion/evidence commits `b17880d5b`, `0d7c59a30`, and `a0a05d53c`. The demand-paged route remains opt-in and default-disabled. Existing T16 correctness and instrumentation evidence does not establish T17 SLO or performance results.

## Frozen capacity gate

The corrected peak estimate is `32 GiB + 5 GiB + 10 GiB = 47 GiB`, where 32 GiB is the single active backend's disk cap including checkpoint/scratch overlap, 5 GiB covers reports/manifests and one preserved failed cell, and 10 GiB is the minimum ZFS reserve. The exact threshold is 50,465,865,728 bytes.

The initial free space was 11,463,950,336 bytes. After the recorded database and regenerable-data cleanup, final preflight availability was 51,568,455,680 bytes, 1,102,589,952 bytes above the exact 47 GiB launch threshold. The mandatory 10 GiB reserve remains included in that gate. At 2026-11-05 05:48:10 UTC, `MemAvailable` was 8,560,888 KiB; no workload process, T17 cgroup, run directory, or cgroup base was present. Cgroup v2 and the memory controller were available. ARC maximum was unset and ARC size was 2,087,831,488 bytes; the frozen per-repetition procedure sets the maximum to 2 GiB before each run. The raw preflight, including binary and native hashes, is preserved in `logs/final-preflight-host-state-f75679e88.log`. Matrix commit `b373db31d` is pushed; frozen matrix SHA-256 is `fc43fe095fa819653a817f3010cd0e8eeacff0f6b9a07aa5b628cc9fd9dcc649`. The immediate execution preflight at 2026-11-05 05:53:43 UTC recorded 51,568,435,200 bytes free, 1,102,569,472 bytes over the gate, and no workload processes or run cgroups; its log SHA-256 is `9a2f9068b4c7719d63632c4b8b4dd1de5c84e9fd5368828a7cd0ae8b3d0c1107`.

## Cleanup and preservation

The following pre-existing database data and regenerable build data were removed after recording path inventories. The local T16 receipts, logs, reports, source files, frozen T17 binaries, and pinned `librocksdb.a` were retained. Four clean source snapshots were consolidated into a verified Git bundle. One old Cargo target was preserved as a verified Zstandard tar archive before its expanded cache was removed.

- Removed 1,824 tenant-density database roots from the existing OCI experiment namespace; pre-clean allocated size was 10,079,969,792 bytes. The exact path inventory and remote deletion output are in `logs/preclean-database-inventory.tsv` and `logs/capacity-reclamation-remote.log`.
- Removed 2,278 existing `.db` files, 15 segmented-WAL database directories, and 117 RocksDB database directories across prior experiment namespaces. The corresponding inventories and exact deleted paths are in `logs/preclean-global-*-inventory.tsv` and `logs/global-database-reclamation.log`.
- Removed the regenerated Cargo cache at `/bench/zfs/db/tenant-density-build-cache-20261029/target` and duplicate build cache `/bench/zfs/db/product-build/t16-b17880d5/target-release`. The frozen binaries remain under `/bench/zfs/db/product-build/t16-b17880d5/artifacts/release/` with hashes `8222cd31df4bdab41cc0cd81221c1c50d83359dfeef464bba74f5c5847b6ee1c` and `e7f2bd38baf417fe567030c1deeb1ed024029414b9a387fdfd235b13f621c5cc`.
- Replaced four clean duplicate `source/dodb` checkouts at commits `0d7c59a3092c79632590e5faffaec9adcf629a10`, `85c0ffac08dce0328be8193d6a8a4a657034dec4`, `18679ccff1dfd62844c6681dcbd97d8a3a8e8338`, and `f2b5ecceee537d84b0c2b0f799288f09a98bf0e8` with `/bench/zfs/db/product-build/t16-b17880d5/artifacts/dodb-source-snapshots-20261105.bundle`. Bundle SHA-256: `5ecf0e6787cdc53de41751e56bface57e88c7b077e9544279b73ecda7814ad7e`.
- Preserved `/bench/zfs/db/dodb-rocksdb-r1b3a-551471d5/target` as `target-preserved-20261105.tar.zst` before deleting the regenerable expanded target. Archive SHA-256: `b1654a6693392509ecb9ea7e802fe234f26865e2a0119a51fd4a8532772b1b00`.
- The pinned native archive remains at `/bench/zfs/db/product-build/rocksdb-source/librocksdb.a`, SHA-256 `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
- Removed two duplicate Cargo target caches at `/bench/zfs/db/product-build/t17-eb433fa35/source/target` and `/bench/zfs/db/product-build/t17-eb433fa35/target`; pre-clean allocations were 3,810,751,488 and 3,810,543,104 bytes. New and previous T17 binaries were copied to separate immutable release directories and hash-verified first.
- Removed the duplicate clean source checkouts at `/bench/zfs/db/product-build/t17-f75679e88/source/dodb` and `/bench/zfs/db/product-build/t17-eb433fa35/source`; the complete Git bundle at `/bench/zfs/db/product-build/t17-f75679e88/artifacts/dodb-source-f75679e88.bundle` preserves both source revisions and verifies cleanly. Bundle SHA-256: `2a75bab6d64d0b5b3005c2d70d07dea722182e70b57952ad78ec9c5394b4269e`.
- Removed 976 generated RocksDB `.o` and `.d` files (1,229,537,280 allocated bytes; 1,199,111,681 logical bytes) from the pinned source checkout after recording every path in `logs/rocksdb-generated-cache-removal.tsv`. The source files and pinned `librocksdb.a` were retained; its SHA-256 matched before and after cleanup.
- Removed the existing T16 smoke database at `/bench/zfs/db/product-build/t16-b17880d5/smoke/paging-retry/data` (200,330,240 allocated bytes). The initial unprivileged removal was denied; retry with noninteractive `sudo` removed the exact path. T16 logs, reports, source snapshots, binaries, and native artifacts remain.

All cleanup attempts and settled capacity readings are preserved in `logs/`. Historical T16 reports remain unchanged; this receipt records the later OCI storage reclamation.

## Frozen execution readiness

The frozen matrix and runner implement and validate five-minute SLO windows, two-window stopping, 30-minute recovery/reopen load, checkpoint-cycle extensions through 120 measured minutes, long-reader retention and B-link checkpoint overlap, and six complete stress recipes. Each stress recipe has a canonical per-scenario SHA-256 recorded in the matrix, controller command, binary manifest, and paired-input identity. The runner pairs seed, ordered schedule, and scenario hashes across backends and retains failed cells. Validation passed with 35 OCI Rust tests, 7 Python controller tests, and scoped Rust formatting.

The new source commit is `f75679e88618ebc018a61158e6b02acb5dd574e3`; the clean source is preserved in the verified bundle. Rust/Cargo are 1.97.1, target is `aarch64-unknown-linux-gnu`, linker is `/usr/bin/gcc` 11.5.0, and the only enabled feature is `blink-demand-paging`. New workload and memory-worker SHA-256 values are `7356e1b06e1f25354a3d194d04beb52f77f819328c9ee1753982cf663785969a` and `fb3fe085aa88f34238c8a7f0f1040fbe208ebc2a57b99f09a602a6f30fd7ab72`. The previous T16 fixed binaries and pinned RocksDB archive retain their original hashes. Full build provenance and log hashes are in `logs/build-f75679e88/artifact-manifest.json`.

A full 1,048,576-row smoke at the frozen offered rate and resource limits failed during seed setup with `overloaded: paged source root admission is paused during source collection`. Its report and failed database are preserved at `smoke-f75679e88-full-seed/` and on the OCI host. This did not reach warm-up, measurement, recovery, SLO, or performance comparison. The formal runner executed `n=1`, `synchronized-arrival`, repetition 0, `blink-paged` and stopped the entire matrix on this first failed cell without launching later backends or cells. The process failed during seed source collection with `Overloaded("paged source reads are paused during source collection")` before warm-up or workload measurement. The outcome recorded exit code 1, `data_directory_preserved=true`, 19,574,784 allocated database bytes, and no remaining workload process. The full database remains on OCI and is preserved locally as a verified 3,872-entry archive. Cell command, binary manifest, report, process log, outcome, runner log, and post-run state are in `formal-screen-f75679e88/` and `logs/post-run-state-f75679e88.log`.

## T17 first-screen result

The sole formal cell was `n=1`, `synchronized-arrival`, repetition 0, `blink-paged`, at the frozen 1,048,576 initial rows per tenant and 250 requests/s/tenant. Its exact candidate process log says `Error: Overloaded("paged source reads are paused during source collection")`; it exited 1 during seed setup before warm-up, workload measurement, oracles, recovery, and SLO windows. The runner wrote a complete failure report and stopped the matrix immediately under the frozen first-failure rule. No resident B-link or TransactionDB comparator ran.

| Evidence dimension | Result |
| --- | --- |
| Correctness and reopen oracles | Not assessed; setup aborted before the oracle phases. |
| Operational result and evidence validity | The candidate failed setup under matching source, binary, native archive, recipe, and scenario identities; the exact failed-cell report and database are preserved. The report status is `failed_or_inconclusive`, and the paired comparison is incomplete because no comparator ran. |
| SLO | Not measured; no warm-up or five-minute window ran. |
| Resource acceptance | The host reserve passed after the run with 51,531,583,488 bytes free, 1,065,717,760 bytes above the 47 GiB gate. The setup cgroup recorded no OOM or high/max events; full workload resource acceptance was not assessed. |
| Comparative performance | Not compared; no comparator or measured phase ran. |

The first-cell database uses 19,574,784 allocated bytes and remains preserved on the OCI ZFS dataset. Its compressed local archive SHA-256 is `8d1288cddddd973a8415cd382681ba14a5f56b2caa4cb5d13b30bcb40f50a748` and contains 3,872 entries. The exact cell report SHA-256 is `20f2dad5df99845de9985fb9a01c15f278262a7ba344cf2aa9b9c2ed80cbede2`; process log SHA-256 is `de38ef2447e79faa9c1c15958d5b8d0ba4bcfdfb520494a5ec3f3e5a6077d60d`.

## Adoption decision

For the frozen tested condition, the decision is **not adopted**. The candidate could not complete seed collection in the lowest-tenant screening cell at the prescribed offer, so it failed the operational gate for that condition. This is not a comparative performance result or a general ranking against RocksDB. Correctness, SLO, full resource acceptance, and comparative performance remain unmeasured because the frozen stop rule correctly prevented later cells.

Defaults remain unchanged, no production deployment occurred, and issue #126 remains open. The matrix remains frozen; this execution does not authorize continuation past the first failed cell.

## Readiness correction

Before execution, the readiness record incorrectly mapped missing harness phases and stress recipes to `evidence_insufficient`. Those were preparation gaps while zero workload cells had started. The corrected pre-execution adoption status was `not_evaluated`. After validation, re-freezing, and passing fresh capacity and identity gates, one formal cell ran and produced the operational failure above. T16 remains complete; T17 now has a condition-specific non-adoption decision based on that measured failure, with all unmeasured dimensions explicitly separated.
