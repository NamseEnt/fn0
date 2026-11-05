# T17 Readiness and Capacity Results

Status: preflight incomplete; workload execution not started. No T17 workload cell was launched. The earlier readiness record omitted a harness and recipe prerequisite and incorrectly labeled the unmeasured adoption decision as evidence insufficient. T17 adoption has not been evaluated.

## T16 completion

T16 is complete at implementation source `0d7c59a3092c79632590e5faffaec9adcf629a10`, with completion/evidence commits `b17880d5b`, `0d7c59a30`, and `a0a05d53c`. The demand-paged route remains opt-in and default-disabled. Existing T16 correctness and instrumentation evidence does not establish T17 SLO or performance results.

## Frozen capacity gate

The corrected peak estimate is `32 GiB + 5 GiB + 10 GiB = 47 GiB`, where 32 GiB is the single active backend's disk cap including checkpoint/scratch overlap, 5 GiB covers reports/manifests and one preserved failed cell, and 10 GiB is the minimum ZFS reserve. The exact threshold is 50,465,865,728 bytes.

The initial free space was 11,463,950,336 bytes. After the recorded database and regenerable-data cleanup, the settled `/bench/zfs/db` availability was 51,941,564,416 bytes, 1,475,698,688 bytes above the 47 GiB launch threshold. The mandatory reserve remains included in that gate. Host `MemAvailable` was 6,566,248 KiB. No workload processes or task cgroups were active. ARC maximum was unset and ARC size was 4.5 GiB; the frozen per-repetition ARC/page-cache procedure had not been applied because execution did not start.

## Cleanup and preservation

The following pre-existing database data and regenerable build data were removed after recording path inventories. The local T16 receipts, logs, reports, source files, frozen T17 binaries, and pinned `librocksdb.a` were retained. Four clean source snapshots were consolidated into a verified Git bundle. One old Cargo target was preserved as a verified Zstandard tar archive before its expanded cache was removed.

- Removed 1,824 tenant-density database roots from the existing OCI experiment namespace; pre-clean allocated size was 10,079,969,792 bytes. The exact path inventory and remote deletion output are in `logs/preclean-database-inventory.tsv` and `logs/capacity-reclamation-remote.log`.
- Removed 2,278 existing `.db` files, 15 segmented-WAL database directories, and 117 RocksDB database directories across prior experiment namespaces. The corresponding inventories and exact deleted paths are in `logs/preclean-global-*-inventory.tsv` and `logs/global-database-reclamation.log`.
- Removed the regenerated Cargo cache at `/bench/zfs/db/tenant-density-build-cache-20261029/target` and duplicate build cache `/bench/zfs/db/product-build/t16-b17880d5/target-release`. The frozen binaries remain under `/bench/zfs/db/product-build/t16-b17880d5/artifacts/release/` with hashes `8222cd31df4bdab41cc0cd81221c1c50d83359dfeef464bba74f5c5847b6ee1c` and `e7f2bd38baf417fe567030c1deeb1ed024029414b9a387fdfd235b13f621c5cc`.
- Replaced four clean duplicate `source/dodb` checkouts at commits `0d7c59a3092c79632590e5faffaec9adcf629a10`, `85c0ffac08dce0328be8193d6a8a4a657034dec4`, `18679ccff1dfd62844c6681dcbd97d8a3a8e8338`, and `f2b5ecceee537d84b0c2b0f799288f09a98bf0e8` with `/bench/zfs/db/product-build/t16-b17880d5/artifacts/dodb-source-snapshots-20261105.bundle`. Bundle SHA-256: `5ecf0e6787cdc53de41751e56bface57e88c7b077e9544279b73ecda7814ad7e`.
- Preserved `/bench/zfs/db/dodb-rocksdb-r1b3a-551471d5/target` as `target-preserved-20261105.tar.zst` before deleting the regenerable expanded target. Archive SHA-256: `b1654a6693392509ecb9ea7e802fe234f26865e2a0119a51fd4a8532772b1b00`.
- The pinned native archive remains at `/bench/zfs/db/product-build/rocksdb-source/librocksdb.a`, SHA-256 `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.

All cleanup attempts and settled capacity readings are preserved in `logs/`. Historical T16 reports remain unchanged; this receipt records the later OCI storage reclamation.

## Frozen execution readiness

The T17 code and artifact identities match the frozen matrix: generator, scheduler, and argument-parser hashes are respectively `7a04a2e32212ce312293dcadd4428148c484ab5da933b295bd05c34271ede837`, `b108d33e05a4c444530f14076a3533bd57906bad6e282756a7fefeddd2b6d032`, and `dc3200451d8fb446b97ba56e115d45ac7d7f603bbf442c582c04a7806a3b8c51`. The preserved worker and harness hashes match the frozen values, and the pinned native archive hash matches.

The frozen worker does not expose a recovery-phase workload duration, rolling five-minute latency/error windows, dynamic duration extension based on checkpoint-cycle count, or a long-reader retention control. It performs one measurement epoch followed by full-population and reopen oracles; the reopen phase is an oracle, not a 30-minute workload. Its reports therefore cannot verify all frozen SLO and stop conditions. The frozen matrix also names five accepted-population stress cases without freezing their complete per-case input recipes and acceptance fingerprints. No existing T17 controller or reducer resolves these gaps. Running the available single-epoch worker would produce a partial, non-comparable result and would violate the frozen acceptance rules. These are preparation defects, not T17 adoption evidence.

## Adoption status

Adoption decision for the T17 target condition: **not evaluated**. T16 correctness and instrumentation checks remain valid, but no T17 correctness, operational, SLO, resource-acceptance, or performance comparison was measured. Defaults were not changed, deployment was not performed, and issue #126 remains open.

## Readiness correction

This record's prior evidence_insufficient value described the missing readiness evidence as an adoption result. That was incorrect: an unstarted experiment cannot support an adoption outcome. The matrix is reopened for an authorized revision that implements its stated phases and gates without lowering them. T16 remains complete; its earlier capacity snapshot remains historical. The prior cleanup receipt and all existing logs and binaries remain preserved.

The T17 matrix and runner must be revised, validated, and re-frozen before execution. The 47 GiB capacity gate, 10 GiB reserve, and existing stop criteria remain in force and must be rechecked after the build and other prerequisites.
