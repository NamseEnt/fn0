# Backend hot-path improvements: implementation and comparison

## Outcome

Both targeted paths are implemented and measured. Across three 300-second repetitions, B-link RW80 throughput improves **4.77%**, while minimal-native RocksDB unconditional width-16 throughput improves **33.40%** versus their own frozen controls.

Against the improved DODB-contract RocksDB reference, improved B-link is **24.60% faster in RW80** and **19.47% slower in E**. It retains substantially lower upper-tail latency, but p95 is 1.63x / 1.72x higher, sampled process-group RSS is 1.88x / 1.64x, and combined reopen/oracle time is 1.16x / 1.20x. These costs prevent a general B-link adoption conclusion.

**Default promotion is withheld.** Both implementations fail their joint predeclared performance gates, despite passing every functional, completion and resource hard guard. The measured changes and evidence remain on the dedicated experiment branch. Storage defaults remain legacy WAL, synchronous checkpoint execution and disabled automatic checkpoints.

## Implemented behavior

1. `BlinkTenantService::read_shard` consults the open-shard registry before building a path or checking filesystem metadata. Cache misses retain existing filesystem/open/capacity behavior. Absence is not cached. This covers ordinary Get, Query, Scan and condition-only transactions on an already open shard.
2. `MinimalNativeTenant::transact` retains the sorted union of condition and mutation keys. Condition keys still use exclusive GetForUpdate with full value decoding. Mutation-only keys call GetForUpdate with a null value pointer, preserving exclusive locking and native conflict validation while avoiding the old-value fetch, return copies and decoding. Revision metadata keeps its existing read/reservation path.

The RocksDB change is confined to the `minimal-native` contract adapter used in this comparison. The other `RocksDbService` adapter is unchanged. Blind mutation keys no longer inspect an old value payload; condition keys and ordinary reads retain their existing validation. WAL synchronization, transaction/revision encoding, tombstones/ABA, admission, grouping, checkpoint parameters and workload drivers are unchanged. No database converts implicitly.

Sources: [B-link service](../../../../crates/dodb-server/src/lib.rs), [minimal-native transactions](../../../../crates/dodb-rocksdb/src/minimal_native.rs), [native lock shim](../../../../crates/dodb-rocksdb/shim/transaction_shim.cc).

## Sustained throughput and latency

Each row summarizes three fresh 300-second admission windows after ten-second warmup. Values are medians of per-run values; throughput ranges cover all three runs. RW80 is the 80:20 read/write API mixture with sixteen dedicated callers, conditions/revisions and widths 1/4/8/16. E uses sixty-four callers and unconditional width-16 overwrite batches. Both use 512-byte values, seed 20261013 and real synchronous WAL.

| Workload / implementation | Throughput/s | Three-run range/s | p95 ms | p99 ms | p99.9 ms | Maximum ms: median / worst |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| RW80 blink-control | 16,504.44 | 16,448.71–16,647.87 | 4.989 | 7.393 | 52.494 | 308.019 / 308.019 |
| RW80 blink-candidate | 17,291.53 | 17,234.89–17,340.57 | 4.706 | 7.107 | 52.855 | 303.563 / 306.446 |
| RW80 rocksdb-control | 13,089.71 | 13,039.85–13,154.74 | 3.076 | 30.147 | 85.787 | 644.350 / 858.259 |
| RW80 rocksdb-candidate | 13,877.48 | 13,532.80–13,960.27 | 2.879 | 28.295 | 80.937 | 911.737 / 988.807 |
| E blink-control | 1,804.75 | 1,724.02–1,856.07 | 79.167 | 139.330 | 305.660 | 405.434 / 431.835 |
| E blink-candidate | 1,822.56 | 1,791.17–1,840.13 | 78.774 | 136.053 | 310.116 | 393.038 / 452.155 |
| E rocksdb-control | 1,696.57 | 1,627.20–1,700.86 | 57.606 | 348.127 | 450.888 | 607.300 / 637.153 |
| E rocksdb-candidate | 2,263.24 | 2,241.26–2,286.86 | 45.842 | 342.098 | 463.471 | 633.461 / 660.584 |

RW80 throughput is API calls/s; E throughput is successful width-16 batches/s. Percentiles are reduced independently from complete observations and are not added. Histogram-derived values are quantized. RW80 maxima are occupied histogram bucket upper endpoints; E maxima are exact recorded values.

### B-link versus its control

Ordinary Get p95 improves **47.76%**, from 32.495 to 16.975 microseconds. Whole-workflow p95 improves 5.67%. However, ordinary Get p99 changes +2.27%, Query16 p99 changes +17.93%, and successful write-transaction p99 changes +6.99%. The throughput increase is below the predeclared +5% target and Get p99 does not improve by 5%, so the target fails. E throughput changes +0.99%; its three-run old/new ranges overlap and the measurement write path was not the cache optimization's target.

### RocksDB versus its control

E throughput improves **33.40%** and E p95 improves **20.42%**. Its E p99.9 and median maximum change +2.79% and +4.31%, respectively, within the declared 5% guards. RW80 throughput improves 6.02% and Get p99 improves 10.58%. The remaining failures are RW80 median maximum latency, which increases **41.50%**, and combined reopen/oracle, which increases **7.55%**. The worst individual RW80 maximum is 988.807 ms versus 858.259 ms for the control.

### Ordinary reads and complete workflows

RW80 API-specific p99 separates individual reads from complete workflows:

| API | Improved B-link ms | Improved RocksDB ms |
| --- | ---: | ---: |
| Ordinary Get | 0.184575 | 0.136319 |
| Query16 | 0.397311 | 0.417023 |
| Successful write transaction | 25.575423 | 65.044479 |

Higher mixed throughput therefore does not establish lower latency for every read API. The profile motivated the changes; this follow-up does not publish a new CPU profile or identify a particular request percentile with a particular syscall.

## Reopen and resource costs

| Workload / implementation | Reopen + oracle ms | Sampled RSS MiB | Maximum directory MiB |
| --- | ---: | ---: | ---: |
| RW80 blink-control | 7,096.03 | 1,298.55 | 757.82 |
| RW80 blink-candidate | 6,592.17 | 1,145.02 | 739.23 |
| RW80 rocksdb-control | 5,274.65 | 622.67 | 418.20 |
| RW80 rocksdb-candidate | 5,672.97 | 610.61 | 420.87 |
| E blink-control | 3,607.40 | 1,701.22 | 864.11 |
| E blink-candidate | 2,952.29 | 1,655.78 | 864.85 |
| E rocksdb-control | 3,925.15 | 1,013.41 | 564.89 |
| E rocksdb-candidate | 2,459.26 | 1,011.15 | 549.96 |

RSS covers the complete process group, including driver, seeding, runtime and validation. It is not retained engine heap. Directory peaks include all required and temporary files. Reopen/oracle includes lazy backend opening and a complete state check, rather than an isolated recovery phase. Faster fixed-duration runs can execute more driver/oracle work. Shared ceilings do not establish equal actual engine-memory use.

B-link's own-control RW80 sampled RSS and combined reopen/oracle improve 11.82% and 7.10%. RocksDB E combined reopen/oracle improves 37.35%. These sustained directions differ from several short-stage observations. Duration, retained WAL/LSM state, compaction and run variation prevent pooling short and sustained values or assigning those changes to one retained-heap or recovery mechanism.

The designated OCI host has two AArch64 Neoverse-N1 cores. ZFS `/bench/zfs/db` retains `sync=standard`, `recordsize=4K`, `compression=off` and `atime=off`. Guards remain 3 GiB process-group RSS, 4 GiB per database, 1500 MiB available host memory and 10 GiB free benchmark filesystem capacity. B-link retains the live primary image plus 512 MiB additional recovery envelope, the 128 MiB WAL trigger, 32 MiB segments and background coalesced immutable bases.

Sustained B-link cases complete 17–36 in-window checkpoints with acknowledged-write overlap. No checkpoint work remains after drain. External full-window samples, rather than capped internal resource vectors, provide the resource evidence. The largest sustained sampling gap is 1.129 seconds.

## Decision and scope

| Declared criterion | B-link | Minimal-native RocksDB |
| --- | --- | --- |
| Primary target | Fail: RW80 +4.77%; Get p99 +2.27% | Pass: E +33.40% |
| All relative latency/resource guards | Fail | Fail |
| Functional, completion and hard resource guards | Pass | Pass |
| Default promotion | Withheld | Withheld |

The write conclusion changes when the reference adapter stops fetching unused values. Improved B-link retains a meaningful mixed-throughput and upper-tail advantage under the declared settings, while improved RocksDB has higher unconditional-write throughput and lower p95, RSS, directory and combined reopen/oracle costs. This evidence supports workload-specific trade-offs rather than a general B-link replacement claim. Native scheduling/admission and B-link coordinator/grouping retain their existing distinct paths. No maximum-RocksDB tuning, common actual-memory-budget comparison, larger-than-memory workload or production deployment is claimed.

This completes the first two implementation targets. WAL planning, checkpoint architecture, retained-heap attribution and request-to-sync tracing remain follow-up work; their cost mechanisms and adoption value require separate evidence. Storage/recovery checksum, invariant and durability validation paths are unchanged.

## Provenance, verification and preservation

The control source remains `cc80d1fdc7a98792be1a739056a3013debf39268` with its original executable hashes. The candidate source is frozen at `3b94ee88eb4e7c02416cde14d3ac48c55e45b991`. Later commits contain experiment tooling and reports without changing measured production code. Rust 1.97.1 with ARM64 `+crc` and pinned RocksDB 11.8.1 are retained; the control and candidate link the identical hashed RocksDB archive. Both frozen checkouts remain clean and executable hashes match after all runs.

All **52 workload cells** pass evidence acceptance: four eight-second preflights, twenty-four 60-second cells and twenty-four 300-second cells. Independent offline verification checks all **956 inventoried files**, reproduces all three original summary objects exactly, matches full-window input prefixes across stages, recounts **343 passed tests, zero failures and four existing ignored probes**, and rechecks the identical control/candidate RocksDB archive and zero new lint diagnostics. Package formatting passes; strict Clippy retains existing findings. See [verification](evidence/offline-verification.json) and [validation](VALIDATION.md).

The short matrix ran on October 26; the sustained matrix ran on October 27 after a recorded external stage barrier. Only the parent pipeline was stopped while its short-matrix child finished. Before sustained work, the operator moved 28 new verified database archives, totaling 4,447,320,976 bytes, to the same host boot filesystem and resumed the parent. All measured database paths stayed on ZFS. Archive creation, relocation, reduction and compression did not overlap workloads. Stages are reported separately.

All 52 new database archives remain on the host, totaling 9,734,587,949 compressed bytes. Each original database file was checked against its lossless archive before only that task database directory was reclaimed. Original control sources, binaries, prior evidence and existing experiment databases remain untouched. Rejected setup/build and checker-environment attempts are retained separately; accepted workloads were not rerun for reporting corrections.

- [Original sustained summary](evidence/sustained-summary.json), [short report](SHORT-RESULTS.md), [original short summary](evidence/short-summary.json).
- [Evidence archive](evidence/backend-hotpaths-20261026-evidence.tar.gz), [identity/inventory sidecar](evidence/backend-hotpaths-20261026-evidence.tar.gz.json), [offline verification](evidence/offline-verification.json).
- [Plan](PLAN.md), [reproduction](REPRODUCE.md), [correctness and limits](VALIDATION.md).

The evidence archive contains 956 inventoried original files and 464,191,250 inventoried uncompressed bytes; compressed size is 32,889,958 bytes. SHA256: `48725c654511878504655086f8347051c41ea9f763d1b2e03b8b23b92a4e9871`.
