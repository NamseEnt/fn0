# Durable checkpoint bases: implementation and measurements

## Outcome

The cross-checkpoint page-base design is implemented and validated as an opt-in experiment. Over three 300-second repetitions, coalesced immutable bases with background execution improve RW80 throughput by **34.23%** and E write-batch throughput by **7.99%** against ordinary segmented synchronous checkpoints at the same 128 MiB trigger. E WAL bytes per mutation fall **61.47%**, and independent process-issued write-request bytes fall **24.23%**.

**The predeclared adoption gate fails.** E misses the +10% throughput requirement, p99.9 and maximum write-batch latency increase, both workloads' maxima increase, and reopening plus state validation becomes slower. Legacy WAL, synchronous execution, and disabled automatic checkpoints remain the defaults. This completes the implementation and experiment, not production promotion or a complete elimination of checkpoint stalls.

## Implemented behavior

Immutable checksummed page packs are authoritative checkpoint storage in `BlinkWalLayout::SegmentedBases`. Manifest v3 binds required packs and WAL segments. Recovery seeds delta decoding from these packs and carries committed page state across segment boundaries. The active WAL writer also preserves page-chain metadata across rotations. Checkpoints no longer reset every existing page's delta eligibility.

The first checkpoint and bounded consolidation write a complete allocated-page image; other checkpoints write dirty pages and retain unchanged bases. Fully superseded packs are reclaimed only after durable manifest publication. Main database pages are a disposable reopen cache and are not rewritten during immutable checkpoints. Recovery materializes that cache before the existing tree loader runs. Replayed post-checkpoint pages remain dirty until included in a later pack, preventing reclamation of their sole authoritative WAL history.

`--blink-wal-layout segmented-bases` selects uncoalesced writes; `segmented-bases-coalesced` groups at most 64 page images per request. Existing `--blink-checkpoint-execution sync|background` selects execution. Background execution retains the existing process-wide permit, one task per shard, latest-manifest publication, acknowledged post-target write preservation and terminal failure behavior. Existing databases never convert implicitly. See [protocol](RECOVERY-PROTOCOL.md) and [plan](PLAN.md).

## Evidence and comparison contract

Measured source: `400ad0ac6b7d8e6306bcb2ab7946747bddcb06c4`, a clean detached fn0 checkout on the designated OCI AArch64 host, Rust/Cargo 1.97.1 with `+crc`, ZFS `/bench/zfs/db`, normal WAL synchronization. Later documentation and reducer additions do not change the measured source. Frozen binaries were hashed again after all measurements; [final identity](evidence/final-oci-identity.json) matches their original identities.

All new rows run B-link through the existing service/workload paths. RW80 uses sixteen callers, 80% reads, 512-byte values and width-16 workflows; E uses sixty-four callers and unconditional width-16 overwrite batches. Seed 20261013, conditions, revisions, admission, group caps, durability and client scheduling are unchanged. This is not a new RocksDB comparison, and historical checkpoint-off or 512 MiB threshold results are not pooled with this matrix.

Accepted evidence comprises eight preflight cells, twenty-four 60-second cells and twelve 300-second cells, each with its own warmup. All 44 pass source/binary identity, fixed input-prefix equality, complete latency counts, final/reopen values and revisions, zero unintended errors/overloads and resource checks. Background performance rows demonstrate actual acknowledged-write overlap. Every sustained cell completes at least ten checkpoints; no 900-second fallback is needed. The separate rejected combined-cap preflight is retained and excluded.

Functional OCI validation passes **337 tests, zero failures, four existing ignored probes**, across twenty optimized executables, including public-path incomplete-transaction recovery, a new acknowledged write, checkpoint, and another recovery; injected failures; corruption; and synchronous/background SIGKILL. These prove process-crash behavior, not arbitrary physical power loss. [Validation details](VALIDATION.md) record the corrected recovery-growth and readiness-marker regressions.

## Sustained results

Values are medians of three 300-second repetitions after ten-second warmup. Throughput is RW80 API calls/s or E write batches/s; bytes are decimal bytes per successful mutation transaction, not per API call. Byte/CPU counters include checkpoint drain after admissions end. Each metric is reduced independently, so ratios are ratios of the reported medians.

| Metric | RW80 segmented sync | RW80 selected candidate | E segmented sync | E selected candidate |
| --- | ---: | ---: | ---: | ---: |
| Throughput/s | 11,851.07 | 15,908.02 (+34.23%) | 1,377.83 | 1,487.97 (+7.99%) |
| Throughput range/s | 11,697.63–11,922.37 | 15,886.76–15,978.88 | 1,370.88–1,379.49 | 1,464.33–1,505.23 |
| WAL bytes/transaction | 6,360.17 | 2,471.18 (−61.15%) | 22,906.61 | 8,826.02 (−61.47%) |
| Checkpoint bytes/transaction | 4,058.33 | 4,513.35 | 21,805.13 | 25,054.04 |
| Process write-request bytes/transaction | 10,421.19 | 6,985.77 (−32.97%) | 44,712.99 | 33,880.07 (−24.23%) |
| Write syscalls/run | 949,759 | 370,652 | 2,209,717 | 51,574 |
| p50, ms | 0.013687 | 0.013527 | 29.065 | 24.052 |
| p95, ms | 7.201 | 5.165 | 55.378 | 50.921 |
| p99, ms | 10.043 | 7.451 | 1,235.223 | 330.039 |
| p99.9, ms | 14.950 | 12.853 | 1,488.978 | 2,879.390 |
| Median per-run maximum, ms | 1,009.779 | 2,447.376 | 1,645.767 | 3,271.781 |
| Worst individual maximum, ms | 1,026.032 | 2,457.862 | 1,955.591 | 3,616.128 |
| Reopen + state oracle, ms | 5,024.182 | 9,780.672 | 2,930.591 | 7,093.500 |
| Process CPU seconds | 145.19 | 181.55 | 115.60 | 124.27 |

RW80 aggregate latencies describe workflows; E latencies describe write batches. RW80 histogram percentiles and maximum use the occupied bucket's upper endpoint, while E maximum is the recorded exact maximum. Every completed latency observation is retained. The highest E maximum is about 3.62 seconds despite much better p99. The aggregate maximum gate fails even though normal reads and writes can improve.

RW80 API-specific median p99/p99.9 in milliseconds:

| API | Segmented sync p99 / p99.9 | Candidate p99 / p99.9 |
| --- | ---: | ---: |
| Get | 0.190 / 0.972 | 0.180 / 0.910 |
| Query16 | 0.287 / 1.071 | 0.349 / 1.010 |
| Successful write transaction | 12.804 / 56.132 | 10.199 / 37.945 |

Linux `wchar` measures bytes requested by the process, including WAL, page packs and metadata. It is not physical-device write traffic. Page-pack/checkpoint bytes increase while WAL and aggregate process-requested bytes fall; WAL reduction alone would overstate the total-write benefit. Reopen timing includes backend opening and a complete final-state oracle, not pure redo replay. Candidate cache reconstruction is part of that operational cost; the experiment does not separately attribute the slowdown to replay CPU or validation.

## Checkpoint costs and remaining stalls

| Sustained median per run | RW80 baseline | RW80 candidate | E baseline | E candidate |
| --- | ---: | ---: | ---: | ---: |
| Checkpoints fully inside client window | 32 | 16 | 70 | 30 |
| Full base snapshots, including drain | 0 | 12 | 0 | 30 |
| Checkpoint elapsed seconds | 25.67 | 39.56 | 91.36 | 126.85 |
| Coordinator preparation + completion seconds | 7.96 | 6.72 | 24.13 | 17.52 |
| Preparation seconds | 6.19 | 4.82 | 19.08 | 9.13 |
| Page-write seconds | 5.14 | 6.84 | 18.85 | 20.81 |
| Data-sync seconds | 12.09 | 25.17 | 48.05 | 83.40 |
| Metadata-sync seconds | 0.040 | 0.129 | 0.105 | 0.163 |
| Completion seconds | 1.79 | 1.86 | 5.16 | 8.48 |
| Acknowledged writes during in-window checkpoints | 0 | 15,104 | 0 | 32,593 |
| Transactions per commit group | 8.77 | 8.76 | 52.53 | 53.42 |
| Capacity waits | 0 | 0 | 0 | 0 |

Checkpoint count ranges are RW80 baseline 31–32, candidate 16; E baseline 70, candidate 29–30. The E candidate writes thirty full packs, 11,185,152,120 bytes per run. At the declared envelope, this write-heavy workload repeatedly reaches the consolidation rule: its next incremental pack would exceed the retained-base allowance. Coalescing removes many write requests, but full image creation, synchronization and serialized preparation/publication still cost time. Reducing WAL bootstrap traffic does not remove those costs.

Checkpoint elapsed time includes background overlap and must not be read as total write-stop time. The coordinator-pause counter covers preparation/completion; synchronous worker waiting additionally blocks its coordinator. These metrics overlap and their independent medians are not additive. Zero capacity waits does not mean zero checkpoint pauses. E candidate checkpoints overlap 28,100–33,761 acknowledged writes per run.

## Resource costs and capacity correction

The candidate uses live checkpoint pages as primary database storage and permits **512 MiB additional recovery storage**, including duplicate bases, WAL and in-flight output. Admission reserves the larger of current pending output and a full consolidation. Snapshot memory remains limited to 512 MiB. The WAL trigger is 128 MiB, segment trigger 32 MiB, process-group RSS guard 3 GiB and total database-directory guard 4 GiB. These guardrails do not establish equal complete-engine memory consumption.

| Sustained median sampled peak, MiB | RW80 baseline | RW80 candidate | E baseline | E candidate |
| --- | ---: | ---: | ---: | ---: |
| Process-group RSS | 863.22 | 1,421.21 | 1,125.94 | 1,745.21 |
| Total database directory | 441.55 | 773.25 | 484.91 | 853.69 |
| Required recovery files + in-flight files | 128.07 | 773.23 | 129.34 | 853.68 |
| Live allocated checkpoint image | 313.56 | 321.27 | 355.57 | 355.57 |
| Live-image + 512 MiB reference envelope | 825.56 | 833.27 | 867.57 | 867.57 |

Baseline recovery-file samples count WAL; its mutable primary database is included in the total directory. Candidate recovery files include immutable primary bases as well as WAL and in-flight packs. Compare total directory sizes for the complete on-disk footprint. Sampled candidate E recovery files peak at 897,947,591 bytes across repetitions, below its 909,709,316-byte envelope. Ordinary segmented WAL keeps its existing retention admission; the live-image-plus-extra number is reported as a comparison reference, not a new baseline admission rule.

The earlier strict 512 MiB combined primary-plus-recovery attempt rejected E while seeding. E's complete primary page image alone is about 356 MiB; a replacement plus retained history cannot fit that combined cap. That attempt is preserved under source `7c048e259`, excluded from all accepted medians, and is not presented as a working equal-512-MiB configuration. The corrected primary-versus-extra interpretation and regression test were fixed before accepted measurements.

Timed RW80 runs complete different numbers of mutations and grow to different live images; identical input prefixes do not imply identical final footprints. E overwrites the same prepopulated pool. Headline RSS uses external controller samples across seeding, warmup, measurement, validation and reopen, whereas CPU and write counters cover measurement plus drain. Both 300-second drivers reach their internal resource-vector cap and omit 465–487 samples; those vectors are not full-window resource evidence. The independent external controller retains 317–328 samples per sustained child, with maximum interval 1.147 seconds. All sustained cells retain at least 3,055 MiB available host memory and 26.81 GiB free filesystem space. Peaks are sampled, not exact instantaneous maxima; runtime admission separately bounds required recovery storage.

## Short matrix and candidate selection

Three rotated 60-second repetitions per path/workload compare each implementation stage. Preflight is excluded. This matrix selects the candidate; it is not the sustained adoption result.

| Path | RW80 calls/s | Change | E batches/s | Change | E WAL bytes/transaction | E p99.9, ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Ordinary segmented sync | 12,399.33 | — | 1,389.84 | — | 22,905.49 | 1,513.10 |
| Immutable bases sync | 15,468.33 | +24.75% | 1,371.52 | −1.32% | 8,830.84 | 4,110.42 |
| Coalesced bases sync | 15,801.30 | +27.44% | 1,494.64 | +7.54% | 8,828.82 | 3,732.93 |
| Coalesced bases background | 15,805.92 | +27.47% | 1,514.46 | +8.97% | 8,829.09 | 2,864.71 |

Page-base reuse alone removes repeated WAL bootstrap images but does not improve E throughput here. Coalescing and background overlap together produce the best measured throughput score, with coalesced sync close behind. The predeclared geometric-mean rule, with E p99.9 as the one-percentage-point tie-break, selects coalesced background. [Selection](evidence/selection.json) binds the original short summary by SHA256. No run is replaced or pooled across stages.

## Adoption decision

| Sustained gate | Observed | Result |
| --- | --- | --- |
| E throughput ≥ +10% | +7.99% | Fail |
| RW80 throughput ≥ −3% | +34.23% | Pass |
| E WAL/transaction ≤ 70% of baseline | 38.53% | Pass |
| E process write-request bytes/transaction ≤ 80% | 75.77% | Pass |
| Both p99.9 ≤ 1.10× baseline | RW80 0.860×; E 1.934× | Fail |
| Both median maxima ≤ 1.10× baseline | RW80 2.424×; E 1.988× | Fail |
| Both reopen + oracle ≤ 1.25× baseline | RW80 1.947×; E 2.421× | Fail |

The structural WAL improvement is real under this workload and durability contract. The current bounded consolidation and reopen materialization costs prevent promotion. Partial base reclamation, a loader that reads immutable bases directly, and further reduction of serialized checkpoint preparation/publication are possible follow-up directions; they are **not implemented or validated by this change**.

## Artifacts

- [Sustained augmented summary](evidence/sustained-summary.json), [original summary](evidence/sustained-summary.original.json).
- [Short augmented summary](evidence/short-summary.json), [original summary](evidence/short-summary.original.json), [preflight summary](evidence/preflight-summary.json).
- [Independent verification](evidence/independent-verification.json): both OCI archives verified byte for byte, all original summary values reproduced, all 44 accepted cells rechecked, selection hash and cross-stage input prefixes matched, and all twenty functional logs independently recounted.
- [Accepted raw OCI archive](evidence/blink-durable-bases-20261023-primary-budget-evidence.tar.gz): 608 inventoried files; [sidecar](evidence/blink-durable-bases-20261023-primary-budget-evidence.tar.gz.json).
- [Rejected preflight/build archive](evidence/blink-durable-bases-20261023-evidence.tar.gz): 113 files; [sidecar](evidence/blink-durable-bases-20261023-evidence.tar.gz.json). The six failed-database files remain unchanged on OCI, with [inventory](evidence/preserved-oci-database-inventory.json).
- [Local rejected fixtures](evidence/local-rejected-fixtures.tar.gz), [verification](evidence/local-fixture-verification.json), and [local validation inventory](evidence/local-validation-inventory.json).
- [Reproduction and provenance](REPRODUCE.md), including exact archive hashes and offline reaggregation. Archives preserve all original raw/enriched JSON, command/binary/source identities, controller/resource reports, validation logs, build evidence and rejected attempts. Source/binary/database payload exclusions are explicit; failed OCI databases remain on the designated host.
