# B-link and DODB-contract RocksDB: CPU and wait profiles

## Outcome

Both implementations have concrete costs worth investigating. B-link's mixed-read service path performs filesystem existence checks on every read; that path accounts for **10.89%** of sampled RW80 CPU, including **7.89%** with filesystem metadata lookup on the stack. Its write workload spends substantial CPU in WAL planning/appending, checkpoint output and allocation/free. RocksDB's current adapter spends **31.23%** of write-workload CPU under `GetForUpdate`, including existing-value reads as well as locking, and **18.68%** under flush/compaction jobs. These are observations about the measured implementations and their service paths, not a tree-versus-LSM causal experiment.

No engine code or defaults changed. Four profiled preflights and eight main diagnostic cells passed workload validation. Each main cell used ten seconds of warmup and sixty seconds of admission. There is one control and one profiled cell per workload/backend; this is not a replacement for the earlier three-by-300-second performance matrix.

## Conditions and recording

The source remains the clean frozen `cc80d1fdc7a98792be1a739056a3013debf39268` checkout from the preceding experiment. The original executables were reused without rebuilding:

| Executable | SHA256 |
| --- | --- |
| `service_workload` | `25128f9be2f9c1ddeeb3b663be299c3798d892ac82aa4ba9a2fcd7377b5a4187` |
| `unconditional_write_workload` | `dd37ec860d3cd6c2bb24abde48f903cd7e87c06842e3a7651589f370cae15828` |

The designated OCI AArch64 two-core host and ZFS `/bench/zfs/db` were verified live (`sync=standard`, `recordsize=4K`, `compression=off`, `atime=off`). The original Rust 1.97.1/ARM64 `+crc` and pinned RocksDB 11.8.1 build provenance remains applicable because source and executable identities are unchanged. RW80 retains sixteen dedicated clients, the 80:20 read/write API mixture, conditions/revisions and widths 1/4/8/16. E retains sixty-four dedicated clients and unconditional width-16 overwrite batches. Both retain 512-byte values, seed 20261013 and real synchronous WAL. B-link retains background coalesced immutable bases, a 128 MiB trigger, 32 MiB segments and existing admission/grouping. RocksDB is the direct DODB-contract `minimal-native` TransactionDB path.

CPU recording uses `cpu-clock` at 99 Hz, existing DWARF, 16 KiB stack snapshots and realtime timestamps. Collectors attach to a single-thread gate before it executes the workload; workload execution starts only after CPU and BPF readiness. Created threads inherit CPU recording. CPU samples are partitioned by the exact report window and checkpoint-event intervals. CPU percentages below count samples with a function on the stack: **parent/child and category percentages overlap and must not be added**. They describe the workload process, including driver/runtime and synchronous kernel work; separate ZFS kernel workers are not included.

## Main CPU findings

| Workload | B-link | RocksDB contract baseline |
| --- | --- | --- |
| E writes | WAL append **21.22%**, including redo planning **10.85%**; checkpoint worker **15.34%**; invariant checks **3.70%** | `GetForUpdate` **31.23%**; commit/write path **26.95%**; flush/compaction jobs **18.68%** |
| RW80 mixed | Transaction group **17.66%**; WAL append **17.55%**; `read_shard` **10.89%**; checkpoint worker **3.14%** | `GetForUpdate` **26.48%**; commit/write path **32.57%**; MemTable/SkipList paths **31.26%** |

### B-link: a specific mixed-read overhead

`BlinkTenantService::read_shard` constructs a database path and calls `Path::exists` before consulting `open_shard`'s cache. This runs for ordinary Get and Query requests even when the shard is already open. Of 3,664 RW80 measurement samples, 399 contain `read_shard`; 289 also contain metadata/stat/path-lookup frames. Thus **10.89%** is the whole lookup path and **7.89%** is its metadata-query subset. Database-path construction appears in 77 samples (**2.10%**), overlapping these costs.

This is a service lookup cost rather than B-link page search. It is a concrete candidate when investigating the earlier ordinary-Get p99 regression. This profile does not establish that it alone caused that percentile change; request-level latency timestamps were not collected.

Source: [read path and existence check](../../../../crates/dodb-server/src/lib.rs), functions `execute_request`, `read_shard`, `open_shard` and `database_path`.

### B-link: WAL and checkpoint work remain substantial

In E, `WalLog::append_group_inner` accounts for **21.22%** of measurement samples and its `plan_redo` subtree for **10.85%**. The checkpoint worker accounts for **15.34%**, versus page encoding **3.61%** and CRC-related stacks **3.10%**. Encoding or checksum acceleration alone would address only a limited part of the observed CPU work.

Within E intervals overlapping checkpoints, the worker represents **40.17%** of CPU samples and invariant checks **9.68%**. Outside those intervals, transaction-group processing represents **33.94%**. This confirms a change in the CPU work mix during checkpoints. Overlap intervals include concurrent foreground execution and are not isolated checkpoint CPU or request-stall duration.

### Allocation is material, but includes the workload driver

Allocator stacks are restricted to malloc/free/realloc and actual `alloc::alloc` functions, rather than the general Rust `alloc` crate. Allocation/free and memory-copy categories can overlap.

| CPU sample category | E B-link | E RocksDB | RW80 B-link | RW80 RocksDB |
| --- | ---: | ---: | ---: | ---: |
| Allocation/free, inclusive | 29.01% | 15.40% | 23.77% | 11.92% |
| Memory copy/set, inclusive | 8.45% | 4.03% | 6.17% | 3.57% |

Nearest-project-caller attribution partitions the allocator samples. In E B-link, **13.96 percentage points of total CPU** attribute to `dodb_storage`, **8.34** to the workload driver, **4.16** to `dodb_core`, and **2.53** to `dodb_server`. In RW80 B-link, the corresponding storage/server/core/driver shares are **9.93 / 8.27 / 3.55 / 1.88** points. These are CPU costs, not retained-heap sizes. The profile does not explain the measured RSS multiplier or prove a leak.

### RocksDB: GetForUpdate is not just lock waiting

The current `MinimalNativeTenant::transact` visits the union of condition and mutation keys, calls `GetForUpdate`, decodes the existing values, and stores them before evaluating conditions. It does this for unconditional E overwrites too. The measured **31.23%** therefore includes key locking and existing-value lookup; `PessimisticTransaction::TryLock` alone is **5.16%** in E and **12.78%** in RW80. Calling the entire GetForUpdate subtree lock overhead would be incorrect.

RocksDB also performs 482,328 `pread64` calls in E's included 59 start-second buckets. This supports the observed existing-value/SST lookup work. It does not establish physical disk misses: ZFS and operating-system caches can satisfy reads. The cost of this adapter's unconditional existing-value reads is a candidate for a separate contract-preserving investigation; it is not proven unavoidable RocksDB work. Locking, conditional semantics and revisions must remain equivalent in any follow-up.

Source: [native transaction implementation](../../../../crates/dodb-rocksdb/src/minimal_native.rs), function `MinimalNativeTenant::transact`.

## Synchronization and off-CPU evidence

BPF syscall aggregation covers full one-second **start buckets** inside each measurement window. There are 59 included seconds per profiled cell. The following are syscall elapsed times, including execution and waiting; totals sum across threads and do not represent elapsed application pause time.

| Workload / backend | `fdatasync` calls | Mean ms/call | Largest call ms | Sum of call durations s |
| --- | ---: | ---: | ---: | ---: |
| E B-link | 2,260 | 16.898 | 96.680 | 38.190 |
| E RocksDB | 6,427 | 8.108 | 451.394 | 52.111 |
| RW80 B-link | 18,454 | 2.096 | 77.585 | 38.671 |
| RW80 RocksDB | 43,678 | 1.164 | 224.662 | 50.825 |

B-link has fewer calls with a higher mean and lower observed maximum in these cells. The data is consistent with different grouping/checkpoint synchronization patterns, but no per-file syscall attribution was recorded, so these counts cannot be divided exactly into WAL, pack and metadata calls. The longer RocksDB synchronization maxima are a concrete tail-latency candidate, not a proof of the cause of a particular request's p95/p99.

Off-CPU flamegraphs contain all switch-out intervals, including idle worker parking and runnable descheduling. Futex totals are dominated by many waiting threads and must not be presented as a serialized engine mutex bottleneck. This kernel encodes preemption as `R+`; the collector's raw `runnable` boolean only checks state zero and is deliberately ignored in the final reduction. Kernel call paths and syscall timings remain usable. Start-bucket attribution can include a long interval ending after the window, and intervals never resumed before exit are not counted. These limitations particularly affect idle-thread totals.

## Reopen and validation

Post-measurement profiles include final-state verification, draining/shutdown, reopen, a second verification and report construction. They are not pure recovery profiles. In E B-link, immutable-base loading appears in **20.99%** of post-window samples, page decoding in **4.94%**, invariant checks in **6.58%**, allocator stacks in **29.42%**, and driver validation in **35.39%**; these overlap. Both base-pack validation and tree construction decode pages, as confirmed in source, but the measurements do not assign the earlier 25–35% sustained reopen disadvantage to that duplicate decoding.

In this separate sixty-second diagnostic, combined E reopen/oracle is 2.666 s for the B-link control and 3.900 s for the RocksDB control. That direction differs from the earlier five-minute matrix. Duration, retained WAL/LSM state, compaction and single-run variation prevent pooling these values or treating the new observation as a reversal of the prior result.

## Profiling perturbation and acceptance

| Workload / backend | Unprofiled throughput/s | Profiled throughput/s | Change |
| --- | ---: | ---: | ---: |
| RW80 B-link | 15,390.24 | 14,815.32 | -3.74% |
| RW80 RocksDB | 12,886.96 | 11,622.55 | -9.81% |
| E B-link | 1,736.01 | 1,718.77 | -0.99% |
| E RocksDB | 1,693.79 | 1,522.15 | -10.13% |

Each percentage compares one cell to one same-session control. These numbers demonstrate perturbation, not an isolated instrument-overhead estimate or a new engine ranking. In particular, CPU and syscall tracing affect the backends differently. BPF-related stacks themselves represent 0.81–3.54% of measured CPU, which does not capture every source of instrumentation cost.

All twelve workload cells pass source/binary identity, zero unintended request errors, full latency counts, fixed per-client input-prefix equality within each workload/stage, full admission, final/reopen state and resource guards. Each main B-link cell completes at least three in-window checkpoints with acknowledged-write overlap and drains all checkpoint work. BPF maps report zero insertion failures. Main CPU windows contain 3,664–5,578 samples; unknown leaf shares are 0.23–1.06%. Existing databases and evidence were preserved. Engine tests were not rerun because the measured source and executables were unchanged.

## Artifacts

| Workload / backend | Measurement CPU | Off-CPU kernel stacks | Post-measurement CPU |
| --- | --- | --- | --- |
| E B-link | [CPU](flamegraphs/E-blink-cpu.svg) | [Off-CPU](flamegraphs/E-blink-offcpu.svg) | [Post-window](flamegraphs/E-blink-post-window.svg) |
| E RocksDB | [CPU](flamegraphs/E-minimal-native-cpu.svg) | [Off-CPU](flamegraphs/E-minimal-native-offcpu.svg) | [Post-window](flamegraphs/E-minimal-native-post-window.svg) |
| RW80 B-link | [CPU](flamegraphs/RW80-blink-cpu.svg) | [Off-CPU](flamegraphs/RW80-blink-offcpu.svg) | [Post-window](flamegraphs/RW80-blink-post-window.svg) |
| RW80 RocksDB | [CPU](flamegraphs/RW80-minimal-native-cpu.svg) | [Off-CPU](flamegraphs/RW80-minimal-native-offcpu.svg) | [Post-window](flamegraphs/RW80-minimal-native-post-window.svg) |

Checkpoint-overlap and outside-checkpoint CPU SVGs/folded stacks are also in the archive. SVGs support click-to-zoom and search.

- [Compact comparison](evidence/comparison.json), [plan](PLAN.md), [reproduction](REPRODUCE.md).
- [Complete profiling evidence](evidence/backend-profiling-20261026-evidence.tar.gz), [identity/inventory sidecar](evidence/backend-profiling-20261026-evidence.tar.gz.json), [offline verification](evidence/offline-verification.json).
- Archive SHA256: `c83050a51e2fd8c045215c7cf5dd0518344fe981e597a32eed84fb89d71277da`; 83,131,493 compressed bytes, 654 inventoried files, 739,263,206 uncompressed inventoried bytes.
- All task databases remain on the host: 107 excluded files, 2,798,455,178 bytes. Frozen binaries remain at the preceding experiment root and are identified above.
- FlameGraph source is pinned to `41fee1f99f9276008b7cd112fca19dc3ea84ac32` from [Brendan Gregg's repository](https://github.com/brendangregg/FlameGraph). Its original source/license bundle is preserved in the archive.
