# planned-blink durable baseline tables

## Successful logical tx/s (mean of 3 repetitions)

| # | Writers | Width | Distribution | A planned-blink | B experiment main-btree | C ExactMain | D RocksDB | A/B | A/C | A/D |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,969 | 3,534 | 3,220 | 10,034 | 1.123 | 1.233 | 0.396 |
| 1 | 16 | 1 | compact locality | 4,757 | 3,851 | 4,097 | 10,913 | 1.235 | 1.161 | 0.436 |
| 2 | 16 | 1 | spread locality | 4,592 | 4,005 | 3,664 | 10,614 | 1.147 | 1.253 | 0.433 |
| 3 | 16 | 16 | uniform | 1,012 | 499 | 469 | 5,723 | 2.027 | 2.158 | 0.177 |
| 4 | 16 | 16 | compact locality | 3,358 | 2,149 | 2,016 | 6,827 | 1.563 | 1.665 | 0.492 |
| 5 | 16 | 16 | spread locality | 3,167 | 1,871 | 1,759 | 7,126 | 1.693 | 1.801 | 0.445 |
| 6 | 64 | 1 | uniform | 11,617 | 5,289 | 4,374 | 26,254 | 2.197 | 2.656 | 0.442 |
| 7 | 64 | 1 | compact locality | 11,780 | 8,116 | 5,846 | 27,378 | 1.451 | 2.015 | 0.430 |
| 8 | 64 | 1 | spread locality | 11,914 | 6,768 | 5,059 | 26,852 | 1.760 | 2.355 | 0.444 |
| 9 | 64 | 16 | uniform | 1,132 | 612 | 533 | 9,573 | 1.849 | 2.125 | 0.118 |
| 10 | 64 | 16 | compact locality | 5,552 | 2,747 | 2,385 | 11,601 | 2.021 | 2.328 | 0.479 |
| 11 | 64 | 16 | spread locality | 4,344 | 1,980 | 1,826 | 11,351 | 2.194 | 2.379 | 0.383 |
| 12 | 1 | 1 | uniform | 1,242 | 1,083 | 939 | 1,512 | 1.148 | 1.323 | 0.822 |
| 13 | 1 | 16 | uniform | 377 | 280 | 303 | 1,353 | 1.346 | 1.245 | 0.278 |

## Successful mutation ops/s (mean of 3 repetitions)

| # | Writers | Width | Distribution | A planned-blink | B experiment main-btree | C ExactMain | D RocksDB |
|---:|---:|---:|---|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,969 | 3,534 | 3,220 | 10,034 |
| 1 | 16 | 1 | compact locality | 4,757 | 3,851 | 4,097 | 10,913 |
| 2 | 16 | 1 | spread locality | 4,592 | 4,005 | 3,664 | 10,614 |
| 3 | 16 | 16 | uniform | 16,184 | 7,984 | 7,499 | 91,576 |
| 4 | 16 | 16 | compact locality | 53,730 | 34,377 | 32,263 | 109,224 |
| 5 | 16 | 16 | spread locality | 50,678 | 29,930 | 28,141 | 114,010 |
| 6 | 64 | 1 | uniform | 11,617 | 5,289 | 4,374 | 26,254 |
| 7 | 64 | 1 | compact locality | 11,780 | 8,116 | 5,846 | 27,378 |
| 8 | 64 | 1 | spread locality | 11,914 | 6,768 | 5,059 | 26,852 |
| 9 | 64 | 16 | uniform | 18,107 | 9,795 | 8,520 | 153,162 |
| 10 | 64 | 16 | compact locality | 88,831 | 43,955 | 38,154 | 185,614 |
| 11 | 64 | 16 | spread locality | 69,509 | 31,678 | 29,224 | 181,616 |
| 12 | 1 | 1 | uniform | 1,242 | 1,083 | 939 | 1,512 |
| 13 | 1 | 16 | uniform | 6,027 | 4,477 | 4,842 | 21,645 |

## Per-repetition planned-blink tx/s

| # | Writers | Width | Distribution | rep 1 | rep 2 | rep 3 | min/max |
|---:|---:|---:|---|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,084 | 4,421 | 4,401 | 0.698 |
| 1 | 16 | 1 | compact locality | 4,759 | 4,907 | 4,605 | 0.939 |
| 2 | 16 | 1 | spread locality | 4,633 | 4,568 | 4,576 | 0.986 |
| 3 | 16 | 16 | uniform | 1,022 | 991 | 1,022 | 0.970 |
| 4 | 16 | 16 | compact locality | 3,354 | 3,407 | 3,314 | 0.973 |
| 5 | 16 | 16 | spread locality | 3,157 | 3,200 | 3,145 | 0.983 |
| 6 | 64 | 1 | uniform | 11,671 | 11,527 | 11,654 | 0.988 |
| 7 | 64 | 1 | compact locality | 11,789 | 11,694 | 11,859 | 0.986 |
| 8 | 64 | 1 | spread locality | 11,826 | 12,323 | 11,593 | 0.941 |
| 9 | 64 | 16 | uniform | 1,124 | 1,138 | 1,133 | 0.988 |
| 10 | 64 | 16 | compact locality | 5,638 | 5,595 | 5,422 | 0.962 |
| 11 | 64 | 16 | spread locality | 4,338 | 4,358 | 4,337 | 0.995 |
| 12 | 1 | 1 | uniform | 1,284 | 1,222 | 1,221 | 0.951 |
| 13 | 1 | 16 | uniform | 377 | 364 | 389 | 0.936 |

## Geometric means over the 12 multiwriter scenarios

| Category | A / B experiment main-btree | A / C ExactMain | A / D RocksDB | B / D (previous "Planned" / RocksDB) | B / C |
|---|---:|---:|---:|---:|---:|
| overall (12) | 1.645 | 1.861 | 0.363 | 0.221 | 1.131 |
| writers 16 | 1.429 | 1.505 | 0.377 | 0.264 | 1.053 |
| writers 64 | 1.893 | 2.301 | 0.350 | 0.185 | 1.215 |
| width 1 | 1.440 | 1.682 | 0.430 | 0.298 | 1.168 |
| width 16 | 1.879 | 2.059 | 0.307 | 0.163 | 1.096 |
| uniform | 1.744 | 1.968 | 0.246 | 0.141 | 1.129 |
| compact locality | 1.543 | 1.736 | 0.458 | 0.297 | 1.125 |
| spread locality | 1.655 | 1.886 | 0.425 | 0.257 | 1.139 |

## Geometric means against the Turso rows of the cross-DB run

| Category | A / Turso WAL | A / Turso MVCC GC-on | B / Turso WAL (previously reported as Planned) | B / Turso MVCC GC-on (previously reported as Planned) |
|---|---:|---:|---:|---:|
| overall (12) | 4.495 | 7.454 | 2.733 | 4.532 |
| writers 16 | 3.216 | 5.157 | 2.250 | 3.608 |
| writers 64 | 6.283 | 10.776 | 3.319 | 5.693 |
| width 1 | 5.199 | 8.246 | 3.610 | 5.725 |
| width 16 | 3.886 | 6.739 | 2.069 | 3.587 |
| uniform | 4.863 | 6.742 | 2.789 | 3.867 |
| compact locality | 4.346 | 18.011 | 2.818 | 11.676 |
| spread locality | 4.296 | 3.411 | 2.596 | 2.061 |

## Scenario 0 control (run after the matrix; not part of the primary table)

Three extra 16-writer width-1 uniform runs with the matrix seeds gave 4,595, 4,477, 4,550 tx/s (mean 4,541). Replacing only scenario 0 with this mean changes the 12-scenario GM to 1.664 vs experiment main-btree, 1.882 vs ExactMain and 0.367 vs RocksDB.

## Latency p50 / p95 / p99 µs (mean of repetition percentiles)

| # | Writers | Width | Distribution | A planned-blink | B experiment main-btree | D RocksDB p99 |
|---:|---:|---:|---|---:|---:|---:|
| 0 | 16 | 1 | uniform | 3,654 / 6,993 / 12,883 | 4,314 / 6,090 / 8,065 | 2,331 |
| 1 | 16 | 1 | compact locality | 3,221 / 4,681 / 6,012 | 3,564 / 9,682 / 12,236 | 2,313 |
| 2 | 16 | 1 | spread locality | 3,397 / 4,968 / 6,234 | 3,908 / 4,664 / 6,760 | 2,245 |
| 3 | 16 | 16 | uniform | 14,149 / 26,716 / 33,625 | 30,684 / 40,533 / 63,042 | 6,714 |
| 4 | 16 | 16 | compact locality | 5,299 / 6,662 / 7,795 | 6,830 / 11,178 / 12,596 | 4,129 |
| 5 | 16 | 16 | spread locality | 4,352 / 8,355 / 10,618 | 7,990 / 11,766 / 14,604 | 3,624 |
| 6 | 64 | 1 | uniform | 4,999 / 8,390 / 10,904 | 11,377 / 17,060 / 22,323 | 4,103 |
| 7 | 64 | 1 | compact locality | 4,259 / 7,925 / 9,183 | 7,348 / 11,547 / 14,263 | 3,886 |
| 8 | 64 | 1 | spread locality | 4,845 / 8,677 / 12,806 | 8,897 / 13,078 / 15,495 | 3,837 |
| 9 | 64 | 16 | uniform | 55,651 / 68,736 / 101,395 | 103,147 / 119,823 / 216,045 | 14,653 |
| 10 | 64 | 16 | compact locality | 10,057 / 19,703 / 26,873 | 21,931 / 31,209 / 40,224 | 10,869 |
| 11 | 64 | 16 | spread locality | 13,624 / 21,256 / 30,709 | 31,262 / 40,014 / 44,266 | 12,421 |
| 12 | 1 | 1 | uniform | 749 / 1,052 / 1,652 | 867 / 1,171 / 1,740 | 968 |
| 13 | 1 | 16 | uniform | 2,533 / 2,989 / 4,163 | 3,272 / 4,483 / 5,598 | 1,135 |

## WAL and resources

| # | Writers | Width | Distribution | A WAL B/tx | A images/tx | A tx/sync | A mean sync ms | A WAL MiB/s | B WAL B/tx | B images/tx | B tx/sync | B mean sync ms | B WAL MiB/s | A CPU % | B CPU % | A peak RSS MiB | B peak RSS MiB |
|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0 | 16 | 1 | uniform | 4,224 | 1.000 | 8.31 | 1.73 | 16.0 | 8,380 | 2.000 | 15.65 | 2.69 | 28.2 | 22 | 44 | 711 | 788 |
| 1 | 16 | 1 | compact locality | 4,224 | 1.000 | 7.79 | 1.43 | 19.2 | 8,380 | 2.000 | 15.74 | 3.29 | 30.8 | 18 | 24 | 320 | 404 |
| 2 | 16 | 1 | spread locality | 4,224 | 1.000 | 7.99 | 1.45 | 18.5 | 8,380 | 2.000 | 15.67 | 2.73 | 32.0 | 22 | 35 | 320 | 384 |
| 3 | 16 | 16 | uniform | 66,506 | 15.986 | 14.32 | 7.19 | 64.2 | 70,665 | 16.987 | 15.93 | 8.86 | 33.6 | 57 | 76 | 1,073 | 864 |
| 4 | 16 | 16 | compact locality | 8,380 | 2.000 | 9.96 | 2.08 | 26.8 | 12,536 | 3.000 | 15.95 | 3.24 | 25.7 | 38 | 61 | 374 | 344 |
| 5 | 16 | 16 | spread locality | 8,380 | 2.000 | 14.05 | 2.67 | 25.3 | 12,536 | 3.000 | 15.97 | 3.20 | 22.4 | 47 | 67 | 375 | 324 |
| 6 | 64 | 1 | uniform | 4,224 | 1.000 | 59.57 | 3.17 | 46.8 | 8,380 | 2.000 | 63.48 | 5.06 | 42.3 | 46 | 63 | 930 | 889 |
| 7 | 64 | 1 | compact locality | 4,224 | 1.000 | 43.89 | 2.98 | 47.5 | 8,380 | 2.000 | 62.93 | 4.85 | 64.9 | 28 | 45 | 514 | 616 |
| 8 | 64 | 1 | spread locality | 4,224 | 1.000 | 59.66 | 3.24 | 48.0 | 8,380 | 2.000 | 63.34 | 4.87 | 54.1 | 43 | 55 | 544 | 560 |
| 9 | 64 | 16 | uniform | 66,490 | 15.982 | 59.17 | 23.75 | 71.8 | 70,646 | 16.982 | 63.20 | 28.84 | 41.2 | 63 | 77 | 1,147 | 952 |
| 10 | 64 | 16 | compact locality | 8,380 | 2.000 | 46.09 | 4.58 | 44.4 | 12,536 | 3.000 | 62.93 | 6.80 | 32.8 | 55 | 76 | 506 | 388 |
| 11 | 64 | 16 | spread locality | 8,380 | 2.000 | 60.97 | 5.08 | 34.7 | 12,536 | 3.000 | 63.17 | 6.98 | 23.7 | 72 | 83 | 471 | 359 |
| 12 | 1 | 1 | uniform | 4,224 | 1.000 | 1.00 | 0.72 | 5.0 | 8,380 | 2.000 | 1.00 | 0.76 | 8.7 | 14 | 21 | 612 | 633 |
| 13 | 1 | 16 | uniform | 66,504 | 15.986 | 1.00 | 2.10 | 23.9 | 70,660 | 16.985 | 1.00 | 2.15 | 18.9 | 25 | 43 | 764 | 718 |

## Validity

```json
{
  "exact_main_engine_fields": [
    "main-btree"
  ],
  "experiment_main_btree_engine_fields": [
    "main-btree"
  ],
  "planned_blink_conflicts": 0,
  "planned_blink_engine_fields": [
    "planned-blink"
  ],
  "planned_blink_errors": 0,
  "planned_blink_measured_leaf_splits": 9,
  "planned_blink_overloads": 0,
  "planned_blink_rows": 42,
  "planned_blink_superblock_images_elided": 1033167,
  "planned_blink_superblock_images_emitted": 9,
  "planned_blink_sync_mode_fields": [
    "real"
  ]
}
```

## 120-second sustained, planned-blink

### 16 writers

Completed. engine `planned-blink`, sync `real`, 4,369 tx/s over 120.0 s, p50/p95/p99 3,615 / 5,411 / 7,269 µs, errors 0, WAL 4,224 B/tx, 1.000 images/tx, 8.71 tx/sync, mean sync 1.52 ms, CPU 29%.

Seeding 1,000,000 rows took about 96.7 s and left RSS 5,653 MiB. Measurement started 106.7 s after process start with RSS 5,877 MiB and WAL 4,769 MiB.

| Window | measured tx/s | derived tx/s (WAL growth / 4,224 B) | p50 µs | p99 µs | WAL at end (MiB) | WAL growth (MiB) | RSS at end (MiB) | RSS growth (MiB) | MemAvailable (MiB) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0-10 s | 4,316 | 4,314 | 3,629 | 7,015 | 4,943 | 174 | 6,076 | 199 | 1,883 |
| 10-20 s | 4,292 | 4,290 | 3,640 | 7,565 | 5,116 | 173 | 6,265 | 189 | 1,899 |
| 20-30 s | 4,416 | 4,416 | 3,570 | 7,301 | 5,294 | 178 | 6,456 | 191 | 1,905 |
| 30-40 s | 4,291 | 4,297 | 3,639 | 6,702 | 5,467 | 173 | 6,638 | 182 | 1,880 |
| 40-50 s | 4,411 | 4,416 | 3,593 | 6,878 | 5,645 | 178 | 6,819 | 181 | 1,866 |
| 50-60 s | 4,397 | 4,393 | 3,608 | 7,269 | 5,822 | 177 | 7,002 | 183 | 1,844 |
| 60-70 s | 4,481 | 4,477 | 3,565 | 7,203 | 6,002 | 180 | 7,182 | 181 | 1,865 |
| 70-80 s | 4,435 | 4,435 | 3,575 | 7,263 | 6,181 | 179 | 7,372 | 190 | 1,860 |
| 80-90 s | 4,393 | 4,394 | 3,657 | 7,276 | 6,358 | 177 | 7,549 | 176 | 1,894 |
| 90-100 s | 4,301 | 4,288 | 3,665 | 6,966 | 6,530 | 173 | 7,721 | 172 | 1,897 |
| 100-110 s | 4,318 | 4,327 | 3,617 | 8,381 | 6,705 | 174 | 7,895 | 174 | 1,890 |
| 110-120 s | 4,380 | 4,275 | 3,610 | 7,372 | 6,877 | 172 | 8,068 | 173 | 1,805 |

Covered 120 s of measurement: WAL grew 2,108 MiB and RSS grew 2,192 MiB (1.040 RSS bytes per WAL byte). Peak sampled RSS 8,072 MiB, max WAL 6,880 MiB, minimum MemAvailable 1,717 MiB.

### 64 writers

Killed by the runner's low-memory guard (MemAvailable < 256 MiB) 194.0 s after process start, 86.6 s into the estimated measurement interval. phase0-bench writes its JSON row only at the end, so tx/s below is derived from WAL growth at 4224 B/tx; latency is unavailable.

Seeding 1,000,000 rows took about 97.5 s and left RSS 5,656 MiB. Measurement started 107.5 s after process start with RSS 6,189 MiB and WAL 5,002 MiB.

| Window | measured tx/s | derived tx/s (WAL growth / 4,224 B) | p50 µs | p99 µs | WAL at end (MiB) | WAL growth (MiB) | RSS at end (MiB) | RSS growth (MiB) | MemAvailable (MiB) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 0-10 s | — | 9,918 | — | — | 5,401 | 400 | 6,629 | 440 | 593 |
| 10-20 s | — | 10,154 | — | — | 5,810 | 409 | 7,059 | 430 | 551 |
| 20-30 s | — | 9,561 | — | — | 6,196 | 385 | 7,456 | 397 | 479 |
| 30-40 s | — | 9,665 | — | — | 6,585 | 389 | 7,849 | 392 | 471 |
| 40-50 s | — | 9,469 | — | — | 6,966 | 381 | 8,223 | 374 | 503 |
| 50-60 s | — | 9,656 | — | — | 7,355 | 389 | 8,665 | 442 | 526 |
| 60-70 s | — | 9,155 | — | — | 7,724 | 369 | 9,056 | 391 | 521 |
| 70-80 s | — | 9,209 | — | — | 8,095 | 371 | 9,427 | 371 | 437 |

Covered 80 s of measurement: WAL grew 3,093 MiB and RSS grew 3,238 MiB (1.047 RSS bytes per WAL byte). Peak sampled RSS 9,645 MiB, max WAL 8,316 MiB, minimum MemAvailable 274 MiB.

### Sustained comparison (B and D reused from the cross-DB run)

| Writers | A planned-blink tx/s | B experiment main-btree tx/s | D RocksDB tx/s | A/B | A/D | A p99 µs | B p99 µs | D p99 µs |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 16 | 4,369 | 2,883 | 10,367 | 1.515 | 0.421 | 7,269 | 10,259 | 2,331 |
| 64 | 9,598 (derived, 80 s before kill) | 3,944 | 25,768 | 2.434 | 0.372 | — | 30,477 | 4,915 |

### Window alignment check

```json
{
  "seed_wal_bytes_estimate": 4809739366.719212,
  "w16_derived_vs_measured_max_window_deviation": 0.023848605015009086,
  "w16_derived_vs_measured_total_deviation": 0.002022537921787156
}
```
