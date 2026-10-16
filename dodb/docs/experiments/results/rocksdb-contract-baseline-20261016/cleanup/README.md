# OCI Experiment Cleanup Evidence

This cleanup removed completed experiment database state and old generated build output from `opc@217.142.246.204`. It retained source, run logs, measurement results, provenance, commands, the current RocksDB build environment, and the shared Cargo cache.

## Capacity

ZFS available space increased from 1,301,846,016 bytes to 107,144,802,304 bytes, a net increase of 105,842,956,288 bytes. The `/` filesystem has 5,794,115,584 bytes available after cleanup; its exact pre-cleanup available-byte count was not recorded, so no exact delta is reported for that filesystem.

The recorded allocated-byte categories are:

| Category | Removed | Allocated bytes |
| --- | ---: | ---: |
| Fixed-group completed DB and WAL directories | 54 directories | 15,877,656,576 |
| Completed experiment DB subtrees | 318 subtrees | 27,788,779,520 |
| Completed DB archives outside the current path | 270 archives | 5,393,957,376 |
| Completed DB archives under the current path | 26 archives | 518,661,120 |
| Old Cargo target directories | 40 directories | 34,955,830,272 |
| Marker-verified product build targets | 3 directories | 5,140,584,448 |
| Old process-crash release targets | 4 directories | 2,369,351,680 |
| Verified duplicate source checkouts | 6 source subtrees | 5,206,168,576 |
| Fixed-group smoke, product baseline, and WAL payload data | 51 files | 2,198,758,912 |
| Old RocksDB build archives | 3 archives | 3,671,946,752 |

Nineteen old executable or compressed binary files were also removed. Their combined logical size was 4,357,675,231 bytes. Their allocated blocks were not captured before deletion. The ZFS available-space delta is the direct filesystem measure; category totals use directory allocation reports or file block counts and can differ because of ZFS metadata and compression.

## Preserved environment and evidence

- RocksDB 11.8.1 source at `/bench/zfs/db/product-build/rocksdb-source`, commit `abeebd9630f11bd08c28b7bd43c7bdfc62050654`.
- Pinned archive SHA-256 `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
- Rust 1.97.1, `/bench/zfs/db/tooling`, `/bench/zfs/db/dodb-rocksdb-shared-cache`, and `/bench/zfs/db/dodb-final-path-20261016`.
- Two representative completed DB archives under the current path: `017-mt-mix-serialized-final-after-rep1` and `020-mt-mix-reserved-ranges-final-after-rep1`.
- Measurement logs, raw results, run commands, source bundles, manifests, and adjacent build/run logs were retained.
- The old dodb source checkouts were clean at their recorded HEADs, and all three HEAD commits exist in the local repository object database. The old RocksDB source copies matched the retained keeper commit. The retained keeper's Git metadata was independently verified after deletion and has no removed worktree dependency.

## Reproducibility impact

The exact historical DB contents in removed native DB directories and archives cannot be reopened from the OCI host. The deleted archive paths, sizes, allocated sizes, and SHA-256 values remain in the journals; old binary paths, logical sizes, and SHA-256 values remain in the binary journal. Two representative completed DB archives remain. Re-running the preserved commands with the preserved source and pinned build environment can recreate experiment state, but it does not restore the byte-identical deleted DB snapshots.

Some database subtree candidates lacked a conclusive DB marker and were skipped. Old revision-oracle directories and all uncertain paths remain untouched. No benchmark or build was started during cleanup.

## Remaining largest directories

Values are allocated KiB from the final inventory, with the three verified source checkout roots reduced by their deleted source subtrees and the four removed process-crash target directories excluded.

| Allocated KiB | Path |
| ---: | --- |
| 9645843 | `/bench/zfs/db/dodb-rocksdb-shared-cache` |
| 5554073 | `/bench/zfs/db/dodb-service-workload-runs` |
| 3735466 | `/bench/zfs/db/product-build` |
| 1630411 | `/bench/zfs/db/experiment-build` |
| 1391856 | `/bench/zfs/db/crossdb-b1648b03-oci` |
| 1012291 | `/bench/zfs/db/experiment-results` |
| 804898 | `/bench/zfs/db/process-crash-recovery-3b3c4998aa1cab1cc6243e1c3f90dfaa8eac3d93` |
| 740794 | `/bench/zfs/db/oci-a1-query-cursor-95954ecaa` |
| 614878 | `/bench/zfs/db/dodb-rocksdb-r1a-70105e0f871a25a9` |
| 556091 | `/bench/zfs/db/dodb-rocksdb-r1b2-558a9283c1a9` |
| 554490 | `/bench/zfs/db/dodb-revision-range-runs` |
| 541102 | `/bench/zfs/db/process-crash-recovery-a3b554191e6c003fd723d4f2b9ade886cbd1d5ee` |
| 524314 | `/bench/zfs/db/dodb-read-fastpath-b4325ee98` |
| 40073 | `/bench/zfs/db/crossdb-d3f4958-oci` |
| 277 | `/bench/zfs/db/crossdb-4e35e99d-oci` |
| 34 | `/bench/zfs/db/crossdb-be1c1786-oci` |
