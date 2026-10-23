# Checkpoint pauses and recovery: implementation and comparison

## Outcome

The implemented candidate improves the targeted long write pauses and exceeds the DODB-contract RocksDB reference's throughput in the declared sustained workloads. Across three 300-second repetitions, median throughput is **+25.72% for RW80 and +10.29% for E** versus RocksDB. Candidate E completes 35–36 in-window checkpoints per run; its worst recorded write across all three runs is **379.45 ms**.

**Default promotion is withheld.** The throughput criteria pass, but candidate reopen plus full-state verification is **25.45% slower in RW80 and 35.45% slower in E** than RocksDB. Sampled process-group RSS is **2.01× / 1.54×** respectively. p95 is also worse in both workloads, and ordinary Get p99 is worse. The separate same-window old/new comparison shows a RW80 p99.9 regression despite substantially lower maxima. This is a useful experimental throughput/maximum-latency improvement with remaining operating costs, not a universal or resource-normalized engine advantage.

## Implemented behavior

1. Immutable-base recovery validates the authoritative packs and committed WAL and constructs the existing in-memory tree directly, preserving metadata/identity/coverage checks, normal page decoding, invariants, revisions and replayed-page dirty tracking. It no longer rewrites and rereads the disposable `.db` cache.
2. Full checkpoints capture immutable references to clean decoded pages; the checkpoint worker performs their additional encoding. Already encoded dirty images retain the fixed target. Later writes cannot change the captured generation, and admission reserves the complete snapshot/output.
3. Immutable pack creation synchronizes each 8 MiB of page output before writing more. Intermediate syncs do not publish a pack or reclaim history. Final file/checksum/directory synchronization, atomic manifest publication and covered-file reclamation preserve their original order.

The existing version-3 format, WAL synchronization, transaction contract, grouping/admission and storage limits are unchanged. Legacy WAL, synchronous execution and disabled automatic checkpoints remain defaults. No existing database converts implicitly. See [protocol](RECOVERY-PROTOCOL.md).

## Sustained throughput and latency

Three rotated 300-second admission windows per implementation/workload follow ten-second warmup. Values are medians of per-run values; the last column gives the median maximum followed by the worst individual maximum. RW80 is an 80:20 read/write API mixture with sixteen dedicated clients and workflow latency, not an ordinary Get benchmark. E uses sixty-four dedicated clients and successful unconditional width-16 write batches. All rows use real synchronous WAL, 512-byte values, seed 20261013 and complete latency observations. Throughput includes final client completion and observed checkpoint drain.

| Workload / implementation | Throughput/s | Three-run range/s | p50 ms | p95 ms | p99 ms | p99.9 ms | Maximum ms: median / worst |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| RW80 candidate B-link | 16,374.20 | 16,337.95–16,518.18 | 0.013487 | 5.030 | 7.401 | 52.953 | 311.427 / 320.602 |
| RW80 RocksDB contract baseline | 13,024.20 | 13,022.56–13,101.88 | 0.013367 | 3.090 | 30.278 | 86.704 | 650.641 / 775.422 |
| E candidate B-link | 1,832.11 | 1,804.07–1,841.87 | 23.282 | 78.774 | 135.922 | 305.660 | 377.379 / 379.449 |
| E RocksDB contract baseline | 1,661.24 | 1,625.55–1,674.62 | 31.375 | 58.196 | 347.603 | 442.237 | 559.271 / 593.926 |

Candidate E p99/p99.9 are **60.90% / 30.88% lower** than RocksDB, with a lower maximum, but p95 is **35.36% higher**. RW80 p99/p99.9 are **75.55% / 38.93% lower**, while p95 is **62.76% higher**. Percentiles are independently reduced and are not added. RW80 maxima use occupied histogram bucket upper endpoints; E maxima are exact recorded values. The E throughput median clears the predeclared +10% threshold narrowly; this does not mean every paired repetition clears it.

RW80 API-specific median p99, separating individual reads from complete workflows:

| API | Candidate ms | RocksDB ms |
| --- | ---: | ---: |
| Ordinary Get | 0.239743 | 0.180351 |
| Query16 | 0.402175 | 0.432127 |
| Successful write transaction | 23.953407 | 69.861375 |

The higher mixed throughput therefore does not establish that every read API is faster. Ordinary Get p99 is **32.93% higher** in the candidate.

## Reopen and resource costs

| Workload | Candidate reopen + oracle ms | RocksDB reopen + oracle ms | Candidate RSS MiB | RocksDB RSS MiB | Candidate directory MiB | RocksDB directory MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| RW80 | 6,872.67 | 5,478.51 | 1,253.27 | 622.90 | 773.52 | 404.14 |
| E | 3,109.96 | 2,296.11 | 1,568.64 | 1,016.96 | 864.40 | 553.07 |

Reopen includes lazy backend opening and a complete final-state oracle. It is not pure redo replay or a measured production restart duration. Faster RW80 runs execute more mutation/oracle work; the combined metric does not isolate recovery CPU. E overwrites the same prepopulated pool. RSS is the externally sampled peak of the entire process group, including driver, runtime, seeding and validation, not isolated engine memory. Directory figures include all required/temporary storage, not only `.db` files.

All implementations share the 3 GiB process-group RSS and 4 GiB directory guards. B-link retains the 128 MiB WAL trigger, 32 MiB segments, 512 MiB snapshot bound and live primary image plus 512 MiB extra recovery envelope. These are unchanged declared limits, not equal actual complete-engine memory consumption. The candidate still writes full allocated-page images during bounded consolidation; partial pack reclamation is not implemented.

Every sustained B-link cell finishes at least seventeen RW80 or thirty-five E checkpoints fully inside its client window, with acknowledged-write overlap, zero capacity waits and no retained task/image after drain. External samples span every child; the largest observed sampling gap is 1.137 seconds. Internal capped resource vectors are not treated as full-window evidence. E median WAL/checkpoint/process-requested bytes per successful transaction are 8,825.68 / 24,289.19 / 33,114.97. Requested bytes are not physical-device writes. Checkpoint phase totals overlap active execution and cannot be interpreted as additive request latency.

## Same-window old/new evidence

The separate three-repetition 60-second matrix compares the previous selected B-link path against the candidate and RocksDB under unchanged commands. Candidate E throughput improves **27.12%**, p99.9 falls **2,818.57 → 303.04 ms**, worst maximum falls **3,079.57 → 463.57 ms**, and combined reopen/oracle falls **7,119.57 → 3,295.02 ms**. WAL bytes per transaction are essentially unchanged, 8,828.10 → 8,827.25.

The trade-offs remain explicit: RW80 throughput changes −1.34%, RW80 p99.9 worsens **12.28 → 46.27 ms**, and E p95 worsens **48.37 → 77.53 ms**. This would fail the earlier experiment's both-workload p99.9 adoption gate. All six current predeclared follow-up gates pass and authorize the sustained comparison; they are not a substitute for the older adoption gate. See [complete short results](SHORT-RESULTS.md). Short and sustained rows are never pooled, and the historical previous 300-second experiment is not used as a fresh paired control.

## Decision

| Declared sustained decision | Observed | Result |
| --- | --- | --- |
| E throughput at least +10% versus RocksDB | +10.29% | Pass, narrow margin |
| RW80 throughput no worse than −3% versus RocksDB | +25.72% | Pass |
| No material tail/reopen regression | Lower upper tails/maxima, higher p95; reopen/oracle +25.45% / +35.45% | Fail |
| Functional, completion and declared resource gates | All accepted cells pass | Pass |

The necessary pause/recovery changes are implemented and measured. The tested configuration now has sustained throughput above the contract RocksDB reference and avoids the prior multi-second E stalls in these observations. Memory/storage usage and remaining percentile/reopen costs prevent a broad adoption claim. Default settings stay unchanged. No maximum-RocksDB tuning, common actual-memory-budget comparison, larger-than-memory workload or production deployment is claimed.

## Provenance and verification

Measured candidate/native source is `cc80d1fdc7a98792be1a739056a3013debf39268`; the old B-link control remains at `400ad0ac6b7d8e6306bcb2ab7946747bddcb06c4`. Both clean detached checkouts and frozen binary hashes match after all measurements. The shared workload drivers, native RocksDB adapter, server, core, service and protocol sources are unchanged between the measured old/new commits. The source audit and original previous-build metadata are archived. Later test/document additions change no measured engine code.

Host: designated OCI AArch64 two-core machine, ZFS `/bench/zfs/db`, Rust/Cargo 1.97.1 with `+crc`, pinned RocksDB 11.8.1. RocksDB directly executes the DODB transaction/revision contract through TransactionDB, including conditional/multi-key semantics; it is not raw WriteBatch. Existing native scheduling/admission and B-link grouping/coordinator paths remain distinct. Source/binary identity, fixed input prefixes, every latency count, zero unintended errors, final/reopen values and revisions, checkpoint coverage/overlap and resources pass for **all 36 accepted cells**: six preflight, eighteen short and twelve sustained.

OCI functional validation passes **339 tests, zero failures, four existing ignored probes** across twenty optimized executables, including incomplete-transaction recovery, another acknowledged write, checkpoint and another recovery; modeled failures; corruption; and synchronous/background SIGKILL at intermediate-sync/publication/reclamation boundaries. Local validation passes 248 tests plus the strengthened clean-page/post-target consolidation regression. Strict Clippy retains twelve pre-existing baseline diagnostics without suppressions. See [validation](VALIDATION.md).

- [Sustained metrics and per-run ranges](evidence/sustained-summary.json), [short metrics](evidence/short-summary.json), [preflight](evidence/preflight-summary.json).
- [Complete OCI evidence archive](evidence/blink-checkpoint-costs-20261023-evidence.tar.gz): 539 inventoried files, 292,945,693 uncompressed bytes, 18,647,192 archive bytes; SHA256 `1769daa047eee863a7656365e870fb88ec324c59d594642e7e43a38ae4945795`.
- [Archive identity and exclusions](evidence/blink-checkpoint-costs-20261023-evidence.tar.gz.json), [offline reproduction and hash verification](evidence/offline-verification.json), [local raw logs](evidence/local/local-validation-logs.tar.gz).
- [Plan](PLAN.md), [recovery protocol](RECOVERY-PROTOCOL.md), [reproduction](REPRODUCE.md). Successful measurements are not rerun for reporting corrections. The rejected premature source-bundle fetch is preserved separately from accepted builds/runs.
