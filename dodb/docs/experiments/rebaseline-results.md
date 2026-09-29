# New-Machine Phase 0/1/2 Rebaseline Results

This document is the current performance baseline for the
`experiment/b-link-batched-engine` branch. It replaces the previous-machine
Phase 0/1/2 performance numbers as the direct comparison point for future
Phase 3/4/5/final work. The old documents and their raw artifacts remain
historical and were not overwritten.

No Phase 3 implementation was started. No LogicalOverlay, batch planner,
same-leaf coalescing, parallel mutation worker, multi-writer, concurrent SMO,
or WAL redesign work was included.

## Identity and environment

| Item | Value |
| --- | --- |
| Branch | `experiment/b-link-batched-engine` |
| Branch HEAD at benchmark invocation | `37ead78583d5b39bf6ad84db939688dfdedfb79f4f` |
| Required HEAD check | passed |
| Machine identity | MacBookPro17,1 / Apple M1 / 8 physical and 8 logical CPUs / 16 GiB RAM |
| OS | macOS 26.4, build `25E246` |
| Kernel | Darwin 25.4.0, ARM64 |
| Filesystem | APFS, case-insensitive, `/Users/namse/dodb` on `/dev/disk3s1` |
| Benchmark storage | Internal Apple Fabric solid-state device, 2.0 TB APFS physical store `disk0s2` |
| Mount observation | `local,journaled,nobrowse,protect,root data` |
| Rust | `rustc 1.97.1 (8bab26f4f 2026-07-14)` |
| Cargo | `cargo 1.97.1 (c980f4866 2026-06-30)` |
| Tokio workers | 8, matching the machine's logical CPU count |
| CPU affinity | No change applied |
| Governor/power mode | No governor change; benchmark observed on AC power with `lowpowermode 0` |
| Benchmark file location | `/Users/namse/dodb/target/rebaseline-macbookpro17-1-macos26.4-8c8t/tmp` |

The benchmark file location and all raw artifacts are on the same APFS
filesystem. No attempt was made to move files to another device or to alter
CPU affinity or power policy.

The machine was not completely idle. At an environment observation point,
the system reported load average `6.12, 5.23, 4.39` and `56.99%` idle, with
WindowServer, Chrome, and Warp active. This is recorded as a limitation of
the absolute numbers. The benchmark process itself reports process CPU usage
in every current raw record.

## Method and harness status

The official runs use release build, 1 s warmup, 2 s measurement, 3
repetitions, deterministic seeds, 16-byte keys, 64-byte inline values, 256
page cache, 4,096-row working set, 64-request groups, 4 MiB group byte limit,
256 queue capacity, and real `ProductionFile::sync_data` unless a synthetic
sync sweep is explicitly named. Query and Scan use limit 16. The structural
stress run is the predeclared exception with 500 ms warmup/measurement, 128
rows, 512-byte keys, one writer, width 1, sequential insert-if-absent, and
the same eight Tokio workers.

The current selectors are present and were exercised:

```text
main-btree
serial-blink
versioned-blink
```

The harness required one minimal portability correction before measurement:
its existing CPU metadata and CPU utilization collection used Linux-only
`/proc` files, which produced unknown CPU fields and null CPU utilization on
macOS. The benchmark binary now uses macOS `sysctl` for CPU metadata and
`getrusage` for process CPU time, while retaining the existing Linux path.
This changes only measurement metadata; it does not change workload
generation, engine selection, storage behavior, durability, or benchmark
semantics. The raw JSONL therefore contains `Apple M1`, `cpu_cores=8`, and
non-null CPU utilization.

The raw records were generated while the branch HEAD field still reported
`37ead78583d5b39bf6ad84db939688dfdedfb79f4f`; the portability correction is
included in the delivery commit. The correction is source-compatible with the
measured code and does not alter the engine result.

## Correctness gate

Both required commands passed after the portability correction:

```text
cargo fmt --all -- --check
cargo test --workspace
```

The workspace test run passed all suites, including 62 storage unit tests, 5
benchmark unit tests, 8 recovery tests, 14 Phase 1 integration tests, 11
QUIC integration tests, and 16 testkit tests. There were no correctness or
environment failures. Structural insert-if-absent runs intentionally reported
condition conflicts because the stress generator revisits keys across its
warmup/measurement lifecycle; they had zero errors and zero overloads.

## Phase 0: main-btree write scaling

Values below are median repetitions. Latencies are end-to-end microseconds,
CPU is percent of the eight-logical-CPU machine, and group is actual requests
per coordinator group. WAL/page columns are per repetition deltas.

### Uniform, width 1

| Writers | logical tx/s | mutation ops/s | p50/p95/p99 us | group | CPU % | WAL bytes | page images | WAL syncs |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 194 | 194 | 5,084 / 6,067 / 6,298 | 1.00 | 1.26 | 3,259,820 | 778 | 389 |
| 4 | 678 | 678 | 5,410 / 9,414 / 12,244 | 3.81 | 2.07 | 11,371,660 | 2,714 | 356 |
| 16 | 2,438 | 2,438 | 6,105 / 9,399 / 14,507 | 15.80 | 2.71 | 41,036,860 | 9,794 | 310 |
| 32 | 3,385 | 3,385 | 9,101 / 13,637 / 19,094 | 31.00 | 5.31 | 56,883,440 | 13,576 | 219 |
| 64 | 5,395 | 5,395 | 11,303 / 15,831 / 22,385 | 62.08 | 5.38 | 91,040,320 | 21,728 | 175 |
| 128 | 5,381 | 5,381 | 23,486 / 26,518 / 33,486 | 64.00 | 5.71 | 91,174,400 | 21,760 | 170 |

### Uniform, width 16

| Writers | logical tx/s | mutation ops/s | p50/p95/p99 us | group | CPU % | WAL bytes | page images | WAL syncs |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 148 | 2,366 | 6,992 / 7,285 / 8,125 | 1.00 | 3.66 | 20,342,968 | 4,890 | 296 |
| 4 | 480 | 7,680 | 7,281 / 12,158 / 17,530 | 3.83 | 4.36 | 66,108,616 | 15,891 | 252 |
| 16 | 849 | 13,582 | 17,300 / 31,828 / 38,883 | 15.52 | 6.84 | 117,265,404 | 28,188 | 110 |
| 32 | 1,233 | 19,735 | 25,278 / 30,432 / 35,591 | 31.69 | 8.69 | 169,803,548 | 40,817 | 78 |
| 64 | 1,469 | 23,506 | 39,370 / 70,848 / 90,777 | 63.09 | 9.66 | 203,941,208 | 49,023 | 47 |
| 128 | 1,533 | 24,530 | 82,863 / 93,254 / 97,288 | 64.00 | 9.82 | 215,793,280 | 51,872 | 49 |

The complete width/distribution matrix is in `phase0/write-core.jsonl`.
For width 1 at 1/16/64 writers, uniform/same-leaf-heavy/different-leaf-heavy
logical tx/s were respectively `194/2,438/5,395`,
`195/2,000/4,899`, and `192/2,032/4,980`. For width 16, the corresponding
mutation ops/s were `2,366/13,582/23,506`, `2,947/26,414/55,428`, and
`2,947/25,167/55,862`. Locality therefore materially changes the wide
transaction result on this machine.

## Phase 0: main-btree read scaling

The following table is the dedicated Phase 0 read run. Query and Scan rows/s
are shown in parentheses. All rows had zero coordinator groups, zero errors,
and zero overloads. Full p50/p95/p99 and CPU fields remain in the raw file.

| Readers | Get req/s | Query req/s / rows/s | Scan req/s / rows/s |
| ---: | ---: | ---: | ---: |
| 1 | 1.394M | 0.403M / 6.446M | 0.485M / 7.755M |
| 4 | 2.841M | 1.084M / 17.352M | 1.374M / 21.989M |
| 16 | 3.100M | 1.167M / 18.671M | 1.410M / 22.554M |
| 32 | 3.169M | 1.164M / 18.630M | 1.319M / 21.108M |
| 64 | 3.086M | 1.277M / 20.430M | 1.480M / 23.679M |
| 128 | 2.928M | 1.025M / 16.399M | 0.234M / 3.744M |

Representative Phase 0 median latencies at 16 readers were Get
`1.791/3.500/4.542 us`, Query `2.583/7.875/12.167 us`, and Scan
`2.125/6.834/11.250 us` for p50/p95/p99. At 128 readers the corresponding
values were Get `1.458/3.333/4.750 us`, Query `3.500/8.833/13.375 us`, and
Scan `8.125/18.292/93.208 us`.

## Phase 0: mixed baseline

| Readers | Writers | Mix | tx/s | read/s | aggregate/s | p95/p99 us | group |
| ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 16 | 16 | 95/5 | 844 | 16,068 | 16,911 | 5,288 / 23,700 | 10.83 |
| 16 | 16 | 50/50 | 376 | 384 | 760 | 69,370 / 119,429 | 13.72 |
| 64 | 16 | 95/5 | 508 | 9,698 | 10,207 | 8,383 / 39,596 | 11.15 |
| 64 | 16 | 50/50 | 582 | 598 | 1,180 | 48,039 / 89,340 | 15.18 |
| 16 | 64 | 95/5 | 729 | 13,887 | 14,616 | 8,777 / 94,297 | 29.34 |
| 16 | 64 | 50/50 | 696 | 716 | 1,412 | 172,761 / 230,292 | 48.59 |
| 64 | 64 | 95/5 | 758 | 14,417 | 15,175 | 14,286 / 94,147 | 33.54 |
| 64 | 64 | 50/50 | 770 | 788 | 1,558 | 115,243 / 139,309 | 50.45 |

## Collection-delay sweep

The sweep uses injected sync delay 0 only to isolate collection behavior.
Queue wait is normalized per attempted operation and collection time per group.

| Workload | Delay us | tx/s | group | queue wait us/op | collection us/group | p95/p99 us |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| write | 0 | 1,337 | 55.22 | 3,552 | 405 | 75,642 / 89,304 |
| write | 50 | 1,402 | 63.33 | 704 | 415 | 64,215 / 88,402 |
| write | 100 | 3,417 | 64.00 | 115 | 122 | 25,234 / 26,447 |
| write | 250 | 5,625 | 64.00 | 38 | 51 | 12,585 / 13,472 |
| write | 500 | 5,568 | 64.00 | 41 | 55 | 14,373 / 16,284 |
| write | 1 ms | 5,709 | 64.00 | 40 | 51 | 12,435 / 13,527 |
| write | 2 ms | 5,653 | 64.00 | 38 | 51 | 12,166 / 12,577 |
| write | 5 ms | 4,171 | 64.00 | 62 | 74 | 24,394 / 39,259 |
| mixed | 0 | 868 | 50.51 | 3,463 | 545 | 120,797 / 208,327 |
| mixed | 50 | 1,714 | 60.98 | 988 | 1,244 | 54,533 / 104,113 |
| mixed | 100 | 2,708 | 63.79 | 243 | 476 | 27,540 / 40,588 |
| mixed | 250 | 3,339 | 63.37 | 183 | 198 | 22,156 / 27,782 |
| mixed | 500 | 3,570 | 63.60 | 110 | 162 | 23,599 / 27,838 |
| mixed | 1 ms | 3,948 | 63.99 | 66 | 117 | 18,985 / 21,588 |
| mixed | 2 ms | 3,646 | 64.00 | 59 | 90 | 20,463 / 32,085 |
| mixed | 5 ms | 3,822 | 64.00 | 87 | 167 | 18,986 / 21,328 |

The 1 ms candidate remains unchanged. The sweep suggests a write optimum
around 250 us–2 ms and a mixed optimum near 1 ms on this run, but the low
delay rows are noisy and the system was not fully idle.

## Sync sensitivity

Synthetic rows use injected delay and must not be interpreted as filesystem
durability.

| Injected sync delay | tx/s | p50/p95/p99 us | group |
| ---: | ---: | ---: | ---: |
| 0 us | 5,349 | 11,266 / 14,428 / 34,103 | 62.99 |
| 100 us | 5,457 | 11,279 / 15,146 / 18,278 | 62.38 |
| 1 ms | 4,959 | 12,421 / 15,255 / 23,531 | 61.80 |
| 5 ms | 3,502 | 18,170 / 19,122 / 34,539 | 62.29 |
| 10 ms | 2,542 | 25,126 / 26,411 / 49,463 | 63.20 |

The separate real `ProductionFile` control at 64 writers, width 1, had
`5,328 tx/s`, p50/p95/p99 `11,507/15,646/21,434 us`, group `62.87`, and
`62.87 transactions per sync`. It is not combined with the synthetic table.

## Phase 1: serial-blink control

### Focused write control

The following are median logical tx/s for the requested uniform/sequential
matrix. The full p50/p95/p99, mutation ops/s, WAL, page, and group metrics are
in the two raw files.

| Writers | Width | main uniform / sequential tx/s | serial uniform / sequential tx/s |
| ---: | ---: | ---: | ---: |
| 1 | 1 | 193 / 185 | 127 / 122 |
| 1 | 16 | 147 / 142 | 119 / 122 |
| 16 | 1 | 1,977 / 2,093 | 655 / 682 |
| 16 | 16 | 965 / 1,016 | 487 / 439 |
| 64 | 1 | 5,409 / 5,440 | 886 / 867 |
| 64 | 16 | 1,378 / 1,609 | 530 / 576 |

### Focused read control

| Readers | Engine | Get req/s | Query req/s / rows/s | Scan req/s / rows/s |
| ---: | --- | ---: | ---: | ---: |
| 1 | main-btree | 1.024M | 0.220M / 3.525M | 0.229M / 3.671M |
| 16 | main-btree | 1.150M | 0.431M / 6.898M | 0.457M / 7.320M |
| 1 | serial-blink | 0.114M | 0.092M / 1.471M | 0.042M / 0.672M |
| 16 | serial-blink | 0.080M | 0.069M / 1.099M | 0.071M / 1.138M |

Serial-blink read results are a control-mutex measurement and do not
represent multicore read scalability.

### Structural stress

The structural run used sequential insert-if-absent with 128 preseed rows and
512-byte keys. The main-btree adapter does not expose structural counters.
Serial-blink genuinely exercised leaf and internal splits; the root counter
was non-zero cumulatively.

| Engine | tx/s | p50/p95/p99 us | measured leaf/internal/root | cumulative leaf/internal/root | errors / overloads |
| --- | ---: | ---: | --- | --- | ---: |
| main-btree | 172 | 4,784 / 6,224 / 6,957 | not instrumented | not instrumented | 0 / 0 |
| serial-blink | 154 | 203 / 6,385 / 7,219 | 14 / 4 / 0 | 84 / 19 / 3 | 0 / 0 |

The structural run had expected insert-if-absent conflicts due to its warmup
and measurement key lifecycle; those conflicts are separate from errors and
overloads and are retained in raw records.

## Phase 2: versioned-blink read scaling

The read matrix compares all three selectors on the same new machine. Tables
show median requests/s; Query and Scan show rows/s in parentheses. Full
percentiles, CPU, generation pins, version fields, and right-link fields are
in the raw JSONL.

### Get

| Readers | main-btree | serial-blink | versioned-blink |
| ---: | ---: | ---: | ---: |
| 1 | 1.079M | 0.162M | 0.917M |
| 4 | 1.109M | 0.100M | 0.974M |
| 16 | 1.139M | 0.099M | 1.194M |
| 32 | 1.482M | 0.101M | 1.211M |
| 64 | 1.480M | 0.101M | 1.209M |
| 128 | 1.631M | 0.099M | 1.222M |

Versioned Get rises through 16 readers and then is effectively flat through
128. In this comparison run it reaches 1.222M req/s at 128 readers, versus
the main-btree 1.631M. The dedicated Phase 0 main read run was faster and
peaked around 32–64 readers, so exact absolute saturation is sensitive to the
observed host load.

### Query, limit 16

| Readers | main req/s / rows/s | serial req/s / rows/s | versioned req/s / rows/s |
| ---: | ---: | ---: | ---: |
| 1 | 0.290M / 4.646M | 0.113M / 1.813M | 0.264M / 4.229M |
| 4 | 0.379M / 6.070M | 0.078M / 1.243M | 0.435M / 6.962M |
| 16 | 0.483M / 7.778M | 0.081M / 1.293M | 0.557M / 8.913M |
| 32 | 0.530M / 8.475M | 0.076M / 1.216M | 0.553M / 8.846M |
| 64 | 0.549M / 8.790M | 0.081M / 1.298M | 0.525M / 8.407M |
| 128 | 0.592M / 9.471M | 0.073M / 1.163M | 0.534M / 8.550M |

### Scan, limit 16

| Readers | main req/s / rows/s | serial req/s / rows/s | versioned req/s / rows/s |
| ---: | ---: | ---: | ---: |
| 1 | 0.220M / 3.520M | 0.119M / 1.912M | 0.314M / 5.020M |
| 4 | 0.403M / 6.448M | 0.089M / 1.421M | 0.512M / 8.200M |
| 16 | 0.575M / 9.200M | 0.087M / 1.386M | 0.579M / 9.263M |
| 32 | 0.560M / 8.968M | 0.086M / 1.369M | 0.616M / 9.858M |
| 64 | 0.660M / 10.554M | 0.085M / 1.365M | 0.539M / 8.632M |
| 128 | 0.669M / 10.700M | 0.082M / 1.319M | 0.624M / 9.976M |

At 16 readers, versioned/main ratios are approximately `1.05` for Get,
`1.15` for Query, and `1.01` for Scan. At 128 readers they are approximately
`0.75`, `0.90`, and `0.93`, respectively. Versioned read-only samples had
generation pins equal to read operations, zero page-version installs, zero
retained/reclaimed runtime versions during the read-only interval, and zero
right-link corrections; the raw records retain the exact counters.

## Phase 2: mixed comparison

| Engine | Readers | Writers | Mix | tx/s | read/s | aggregate/s | p95/p99 us | group |
| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| main-btree | 16 | 16 | 95/5 | 1,471 | 27,990 | 29,461 | 1,315 / 14,483 | 11.62 |
| serial-blink | 16 | 16 | 95/5 | 318 | 6,079 | 6,397 | 23,832 / 47,748 | 4.67 |
| versioned-blink | 16 | 16 | 95/5 | 301 | 5,772 | 6,073 | 710 / 55,955 | 7.69 |
| main-btree | 16 | 16 | 50/50 | 1,649 | 1,650 | 3,299 | 13,731 / 16,792 | 14.86 |
| serial-blink | 16 | 16 | 50/50 | 289 | 296 | 585 | 79,493 / 92,331 | 10.12 |
| versioned-blink | 16 | 16 | 50/50 | 276 | 293 | 568 | 84,540 / 104,181 | 11.53 |
| main-btree | 64 | 16 | 95/5 | 2,430 | 46,178 | 48,608 | 3,244 / 30,785 | 34.99 |
| serial-blink | 64 | 16 | 95/5 | 449 | 8,567 | 9,015 | 29,150 / 72,364 | 7.23 |
| versioned-blink | 64 | 16 | 95/5 | 334 | 6,368 | 6,702 | 1,535 / 202,711 | 31.22 |
| main-btree | 64 | 16 | 50/50 | 3,096 | 3,107 | 6,203 | 27,089 / 35,027 | 57.14 |
| serial-blink | 64 | 16 | 50/50 | 345 | 364 | 709 | 271,343 / 300,600 | 37.42 |
| versioned-blink | 64 | 16 | 50/50 | 338 | 347 | 685 | 229,984 / 265,860 | 34.10 |
| main-btree | 16 | 64 | 95/5 | 1,398 | 26,616 | 28,014 | 788 / 15,243 | 11.81 |
| serial-blink | 16 | 64 | 95/5 | 321 | 6,143 | 6,464 | 27,439 / 46,828 | 4.85 |
| versioned-blink | 16 | 64 | 95/5 | 268 | 5,139 | 5,407 | 550 / 71,697 | 9.02 |
| main-btree | 16 | 64 | 50/50 | 1,680 | 1,699 | 3,378 | 12,282 / 16,296 | 15.49 |
| serial-blink | 16 | 64 | 50/50 | 278 | 295 | 573 | 79,812 / 93,733 | 9.26 |
| versioned-blink | 16 | 64 | 50/50 | 255 | 273 | 528 | 93,437 / 138,066 | 11.70 |
| main-btree | 64 | 64 | 95/5 | 1,839 | 34,973 | 36,813 | 6,592 / 32,998 | 35.06 |
| serial-blink | 64 | 64 | 95/5 | 450 | 8,591 | 9,041 | 32,688 / 79,585 | 7.61 |
| versioned-blink | 64 | 64 | 95/5 | 305 | 5,811 | 6,116 | 1,607 / 228,188 | 31.10 |
| main-btree | 64 | 64 | 50/50 | 2,691 | 2,708 | 5,399 | 34,454 / 43,179 | 55.24 |
| serial-blink | 64 | 64 | 50/50 | 410 | 434 | 845 | 224,513 / 293,733 | 34.00 |
| versioned-blink | 64 | 64 | 50/50 | 313 | 331 | 644 | 297,782 / 327,472 | 41.38 |

Versioned publication timing in the median mixed rows was approximately
`2.35–3.27 ms per successful transaction` across the eight workloads, based
on the existing cumulative publication timer. This is attribution data, not a
new instrumentation claim. The versioned rows had 504–653 page-version
installs and zero right-link corrections in the measured interval.

## Phase 2: serial versus versioned write control

| Writers | Width | serial tx/s | versioned tx/s | versioned mutation ops/s |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1 | 148 | 139 | 139 |
| 1 | 16 | 142 | 138 | 2,205 |
| 16 | 1 | 667 | 662 | 662 |
| 16 | 16 | 450 | 443 | 7,085 |
| 64 | 1 | 798 | 762 | 762 |
| 64 | 16 | 503 | 503 | 8,053 |

Versioned-blink write throughput is within a few percent of serial-blink in
this control. The larger Phase 2 cost appears in mixed publication and tail
latency, not as a large isolated write-control throughput regression.

## Interpretation and readiness

The old-machine numbers differ qualitatively from this machine and must not
be used for cross-machine ratios. The main differences observed here are:

- Main width-1 uniform writes plateau at 64 writers: 5,395 tx/s at 64 and
  5,381 tx/s at 128. Width-16 uniform mutation throughput continues to its
  measured peak at 128 writers: 24,530 mutation ops/s.
- Main dedicated read scaling peaks around 32–64 readers for Get before the
  128-reader regression in the Phase 0 run. The separate Phase 2 comparison
  run was slower and did not reproduce that exact absolute peak, which shows
  the sensitivity to host load.
- Versioned-blink Get scales to roughly 16 readers and then saturates around
  1.2M req/s through 128 readers. Query and Scan are broadly flat after the
  16-reader region with run-to-run variation.
- Real sync latency is about 11.5 ms p50 and 21.4 ms p99 in the dedicated
  width-1, 64-writer control; synthetic sync rows are not a substitute.
- The best collection-delay region in this run is roughly 250 us–2 ms for
  write-only and close to 1 ms for mixed. The design candidate remains 1 ms.
- The largest current bottlenecks are real filesystem sync and the serial
  candidate/preparation/publication path. Versioned publication adds a clear
  mixed-workload tail cost, while serial-blink's read mutex dominates its
  read scaling.

Technical readiness for Phase 3 is conditional but sufficient to begin a
separate, explicitly scoped implementation phase: the correctness gate is
green, the three selectors are measurable, structural counters are present,
and the current-machine baseline is separated and reproducible. Phase 3 must
compare against this document and these raw artifacts only. It must not
compare old-machine Phase 2 directly with new-machine Phase 3, and this
rebaseline task is complete without starting Phase 3.

## Raw artifacts

The current raw artifacts are under:

```text
docs/experiments/results/rebaseline-macbookpro17-1-macos26.4-8c8t/
  phase0/
  phase1/
  phase2/
```

The interrupted first read attempt is preserved separately as
`phase0/read-core-interrupted-20260923.jsonl`; it is not part of the current
baseline. The completed files retain every repetition and all machine,
latency, CPU, group, WAL/page, Blink, and version metrics emitted by the
harness.
