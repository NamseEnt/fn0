# Phase J Measurement Tables

Source commit: `ffe774c58da1157f6e735ee75262efd978022757`. Latencies are nanoseconds per operation.

## GET

| Segments | Case | H1 ns | J0 ns | Ratio | H1 alloc/op | J0 alloc/op |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 0 | base_hit | 397 | 401 | 1.010x | 3 | 3 |
| 0 | miss | 389 | 394 | 1.013x | 2 | 2 |
| 1 | newest_hit | 392 | 237 | 0.605x | 2 | 2 |
| 1 | oldest_hit | 392 | 238 | 0.607x | 2 | 2 |
| 1 | tombstone | 465 | 217 | 0.467x | 3 | 1 |
| 1 | base_hit | 398 | 422 | 1.060x | 3 | 3 |
| 1 | miss | 392 | 418 | 1.066x | 2 | 2 |
| 2 | newest_hit | 388 | 235 | 0.606x | 2 | 2 |
| 2 | oldest_hit | 388 | 361 | 0.930x | 2 | 2 |
| 2 | tombstone | 458 | 210 | 0.459x | 3 | 1 |
| 2 | base_hit | 396 | 432 | 1.091x | 3 | 3 |
| 2 | miss | 388 | 430 | 1.108x | 2 | 2 |
| 4 | newest_hit | 389 | 235 | 0.604x | 2 | 2 |
| 4 | oldest_hit | 388 | 566 | 1.459x | 2 | 2 |
| 4 | tombstone | 459 | 211 | 0.460x | 3 | 1 |
| 4 | base_hit | 394 | 464 | 1.178x | 3 | 3 |
| 4 | miss | 388 | 461 | 1.188x | 2 | 2 |

## Query and Scan

| Segments | Operation | Limit | H1 ns | J0 ns | Ratio | H1 alloc/op | J0 alloc/op |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | query | 8 | 2192 | 2194 | 1.001x | 38 | 38 |
| 0 | query | 128 | 35519 | 35643 | 1.004x | 523 | 523 |
| 0 | scan | 8 | 1850 | 1857 | 1.004x | 35 | 35 |
| 0 | scan | 128 | 34253 | 34225 | 0.999x | 520 | 520 |
| 0 | scan | 4096 | 1112628 | 1110078 | 0.998x | 16402 | 16402 |
| 1 | query | 8 | 2202 | 3196 | 1.451x | 38 | 44 |
| 1 | query | 128 | 35859 | 47879 | 1.335x | 523 | 556 |
| 1 | scan | 8 | 1855 | 2764 | 1.490x | 35 | 41 |
| 1 | scan | 128 | 34446 | 46749 | 1.357x | 520 | 553 |
| 1 | scan | 4096 | 1111396 | 1443608 | 1.299x | 16402 | 16627 |
| 2 | query | 8 | 2188 | 3385 | 1.547x | 38 | 44 |
| 2 | query | 128 | 35933 | 50631 | 1.409x | 523 | 556 |
| 2 | scan | 8 | 1853 | 2918 | 1.575x | 35 | 41 |
| 2 | scan | 128 | 34572 | 49266 | 1.425x | 520 | 553 |
| 2 | scan | 4096 | 1111399 | 1493881 | 1.344x | 16402 | 16627 |
| 4 | query | 8 | 2176 | 3798 | 1.745x | 38 | 44 |
| 4 | query | 128 | 35937 | 55423 | 1.542x | 523 | 556 |
| 4 | scan | 8 | 1861 | 3182 | 1.710x | 35 | 41 |
| 4 | scan | 128 | 34605 | 54306 | 1.569x | 520 | 553 |
| 4 | scan | 4096 | 1116343 | 1609537 | 1.442x | 16402 | 16627 |

## J1 CPU Gate

OCI A1 Neoverse-N1, 64 transactions per group, width 16, 8,192 transactions per measured run. User cycles are from `perf stat` around the timed CPU loop. H1 is the existing physical path using in-memory file implementations; logical overlay is volatile and skips WAL and sync. This isolates foreground CPU and must not be interpreted as durability throughput.

| Existing segments | Engine | CPU ns/tx | User cycles/tx | Instructions/tx | Allocations/tx | Admission ns/tx | Condition lookup ns/tx | Group mutation ns/tx | Segment build ns/tx | Publication ns/tx |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | H1 physical | 162035.19 | 481927 | 950031 | 297.219 | 9652.09 | included in H1 admission | included in H1 physical path | included in H1 physical path | 10.39 |
| 0 | logical overlay | 15996.20 | 47956 | 101256 | 87.734 | 12395.59 | 38.00 | 6751.61 | 2307.47 | 10.02 |
| 1 | logical overlay | 15329.81 | 45931 | 101267 | 87.734 | 11661.72 | 35.49 | 6214.93 | 2331.94 | 9.58 |
| 2 | logical overlay | 15272.96 | 45757 | 101270 | 87.734 | 11656.21 | 35.17 | 6223.10 | 2322.62 | 10.23 |
| 3 | logical overlay | 15218.23 | 45617 | 101270 | 87.734 | 11624.05 | 35.28 | 6226.36 | 2315.60 | 10.05 |

At the maximum measured prior-segment count, overlay CPU was 9.39% of H1. User-cycle figures are rounded from the perf-counter totals retained in `perf/j1-control-*-perf.txt`; raw test output is under `raw/j1/`.

## J2 Logical WAL Bytes

OCI A1, source `81a64e01ead3bb58dde74f17e8d11adb737e5915`, 64 transactions per group, width 16, 64-byte values. The file was `MemoryFile`; no durable throughput is implied.

| Transactions | Mutations | Records | Records/tx | WAL bytes | WAL bytes/tx | Syncs | Transactions/sync |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 1,024 | 1,088 | 17.000 | 173,060 | 2,704.062 | 1 | 64 |

## J4 Durable WAL v4 Matrix

OCI A1, real `sync_data`, 10 seconds measured after a 2 second warmup, three paired repetitions. Each H1/J pair used the same workload seed and ran consecutively in one session. Data and WAL files were created under `/bench/zfs/db/phase-j-data`, on ZFS dataset `dodbbench/db`. H1 used `parallel-blink` from Phase I final source `29f4f4bbc3172e7ef8c09e7bfcfa00b3bb54e814`; J used `logical-overlay-blink` from `0ec2528a7513e507b2033aba095a937373e754b7`. Binary hashes and all raw run rows are retained in `raw/j4/durable/`.

Throughput and latency are medians across repetitions. J/H1 is the median of the three paired throughput ratios. CPU is percent of one logical CPU. WAL bytes per transaction use the cumulative bytes appended during measurement, including frames later reclaimed. Frame counts include data frames and commit frames; J v4 emits one logical mutation frame per mutation and one commit frame per transaction. Transactions/sync is successful logical transactions divided by measured sync calls.

| Writers | Width | Distribution | H1 tx/s | J tx/s | J/H1 | H1 p50/p95/p99 us | J p50/p95/p99 us | H1/J CPU % | H1/J WAL B/tx | H1/J frames/tx | H1/J syncs/run | H1/J tx/sync | J materializations | J materialization avg/max ms |
| ---: | ---: | --- | ---: | ---: | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 1 | uniform | 4,962.3 | 1,547.8 | 0.312x | 3,067/4,661/5,287 | 16,639/21,240/22,868 | 13.7/37.2 | 225.5/241.1 | 2.00/2.00 | 6,528/2,535 | 7.60/6.10 | 507 | 16.67/27.46 |
| 16 | 16 | uniform | 3,104.1 | 1,464.8 | 0.473x | 4,894/7,271/8,042 | 17,929/20,027/21,740 | 50.7/43.5 | 2,676.2/2,717.2 | 16.57/17.00 | 4,032/2,380 | 7.71/6.16 | 476 | 16.71/22.51 |
| 64 | 1 | uniform | 17,319.7 | 6,486.4 | 0.376x | 3,812/4,552/5,234 | 2,459/20,877/21,630 | 27.0/39.0 | 225.3/241.1 | 2.00/2.00 | 5,156/2,501 | 33.61/25.94 | 500 | 16.47/23.98 |
| 64 | 16 | uniform | 6,496.1 | 4,694.5 | 0.720x | 10,578/16,698/18,705 | 6,646/31,147/34,162 | 92.8/50.9 | 2,672.0/2,717.1 | 16.60/17.00 | 1,498/1,350 | 43.39/35.47 | 270 | 22.90/31.22 |
| 64 | 16 | compact | 13,272.0 | 5,500.0 | 0.413x | 5,595/6,549/7,008 | 6,369/25,365/26,730 | 65.6/42.2 | 536.0/2,748.3 | 3.00/17.00 | 3,296/1,591 | 40.36/34.87 | 318 | 18.07/25.63 |
| 64 | 16 | spread | 12,585.1 | 4,972.9 | 0.395x | 5,894/7,146/7,678 | 6,561/29,337/30,788 | 68.7/49.5 | 663.3/2,702.1 | 3.00/17.00 | 3,023/1,406 | 41.77/35.39 | 281 | 21.60/28.99 |

The primary 64-writer width-16 uniform gate reached 0.720x H1, below the 1.20x comparison gate. Width-1 protection also failed: 0.376x at 64 writers and 0.312x at 16 writers. The 16-writer width-16 uniform case reached 0.473x. Width-16 compact/spread increased v4 bytes substantially against H1's 3-frame PageDelta representation.

The v4 path materialized 270 times in the primary 10-second run, averaging 22.90 ms with a 31.22 ms maximum. It spent 6.18 seconds total in measured materialization work. The synchronous all-segment policy caused pronounced tail latency and lower throughput. The 120-second sustained run and RocksDB comparison were not run because their stated prerequisite gates failed.
