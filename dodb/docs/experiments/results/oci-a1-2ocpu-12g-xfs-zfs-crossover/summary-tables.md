# XFS and ZFS Crossover Results

Cells report the median and sample coefficient of variation (CV = sample standard deviation / mean × 100%). Each stage uses five repetitions. XFS-pre and XFS-post remain separate.

## fio payload-size sweep

| payload and metric | XFS-pre | ZFS | XFS-post |
|---:|---:|---:|---:|
| 4,096 B operations/s | 1,372.4 (23.6%) | 1,270.4 (0.7%) | 1,399.5 (23.6%) |
| 4,096 B MB/s | 5.62 (23.6%) | 5.20 (0.7%) | 5.73 (23.6%) |
| 4,096 B fdatasync mean ms | 0.717 (30.9%) | 0.768 (0.7%) | 0.704 (31.1%) |
| 4,096 B fdatasync p50 ms | 0.643 (35.8%) | 0.733 (0.8%) | 0.627 (36.0%) |
| 4,096 B fdatasync p95 ms | 1.319 (22.3%) | 1.020 (1.8%) | 1.368 (21.6%) |
| 4,096 B fdatasync p99 ms | 1.516 (15.7%) | 1.384 (4.1%) | 1.548 (10.6%) |
| 4,096 B CPU seconds | 0.780 (6.0%) | 1.040 (1.3%) | 0.791 (4.4%) |
| 4,096 B device bytes written | 128,705,536 (1.3%) | 310,422,528 (1.8%) | 130,555,904 (0.9%) |
| 4,096 B device busy ms | 18,868 (0.7%) | 17,100 (0.6%) | 18,848 (0.9%) |
| 16,384 B operations/s | 1,398.8 (23.0%) | 1,079.4 (1.3%) | 1,384.2 (23.1%) |
| 16,384 B MB/s | 22.92 (23.0%) | 17.68 (1.3%) | 22.68 (23.1%) |
| 16,384 B fdatasync mean ms | 0.702 (33.3%) | 0.894 (1.2%) | 0.710 (33.9%) |
| 16,384 B fdatasync p50 ms | 0.700 (33.0%) | 0.815 (0.9%) | 0.709 (33.4%) |
| 16,384 B fdatasync p95 ms | 0.881 (30.4%) | 1.434 (4.0%) | 0.905 (30.2%) |
| 16,384 B fdatasync p99 ms | 1.020 (29.5%) | 2.245 (5.9%) | 1.044 (29.4%) |
| 16,384 B CPU seconds | 0.809 (4.5%) | 1.199 (2.5%) | 0.780 (3.4%) |
| 16,384 B device bytes written | 458,428,416 (16.0%) | 764,473,344 (1.7%) | 453,832,704 (16.2%) |
| 16,384 B device busy ms | 18,936 (0.8%) | 16,844 (0.3%) | 19,000 (0.8%) |
| 65,536 B operations/s | 1,062.4 (5.8%) | 429.6 (0.6%) | 1,051.8 (5.7%) |
| 65,536 B MB/s | 69.63 (5.8%) | 28.16 (0.6%) | 68.93 (5.7%) |
| 65,536 B fdatasync mean ms | 0.924 (6.3%) | 2.225 (0.7%) | 0.933 (6.3%) |
| 65,536 B fdatasync p50 ms | 0.922 (1.9%) | 2.179 (0.8%) | 0.930 (2.5%) |
| 65,536 B fdatasync p95 ms | 1.122 (18.2%) | 2.507 (1.2%) | 1.139 (18.3%) |
| 65,536 B fdatasync p99 ms | 1.270 (19.8%) | 3.031 (6.9%) | 1.286 (17.2%) |
| 65,536 B CPU seconds | 0.733 (3.4%) | 1.757 (1.4%) | 0.731 (1.6%) |
| 65,536 B device bytes written | 1,392,981,504 (5.2%) | 613,146,624 (1.6%) | 1,378,957,824 (5.1%) |
| 65,536 B device busy ms | 18,860 (0.4%) | 16,536 (0.6%) | 18,904 (0.5%) |
| 262,144 B operations/s | 394.3 (0.9%) | 250.0 (0.8%) | 395.2 (0.8%) |
| 262,144 B MB/s | 103.37 (0.9%) | 65.52 (0.8%) | 103.60 (0.8%) |
| 262,144 B fdatasync mean ms | 2.485 (1.0%) | 3.723 (0.7%) | 2.480 (0.8%) |
| 262,144 B fdatasync p50 ms | 1.860 (5.8%) | 3.555 (0.7%) | 1.827 (2.8%) |
| 262,144 B fdatasync p95 ms | 4.014 (1.3%) | 4.014 (1.1%) | 4.047 (1.1%) |
| 262,144 B fdatasync p99 ms | 4.178 (0.8%) | 9.765 (6.6%) | 4.178 (0.8%) |
| 262,144 B CPU seconds | 0.599 (6.6%) | 2.845 (0.6%) | 0.563 (3.7%) |
| 262,144 B device bytes written | 2,067,664,384 (1.0%) | 1,423,362,048 (0.7%) | 2,072,529,408 (0.9%) |
| 262,144 B device busy ms | 19,324 (0.3%) | 16,188 (0.8%) | 19,396 (0.5%) |
| 1,048,576 B operations/s | 100.7 (1.0%) | 91.7 (1.1%) | 98.5 (0.1%) |
| 1,048,576 B MB/s | 105.58 (1.0%) | 96.18 (1.1%) | 103.26 (0.1%) |
| 1,048,576 B fdatasync mean ms | 9.727 (1.0%) | 9.888 (1.2%) | 9.945 (0.1%) |
| 1,048,576 B fdatasync p50 ms | 11.076 (0.0%) | 9.896 (0.7%) | 11.076 (0.0%) |
| 1,048,576 B fdatasync p95 ms | 11.207 (0.0%) | 15.532 (1.3%) | 11.207 (0.0%) |
| 1,048,576 B fdatasync p99 ms | 11.338 (0.0%) | 22.413 (10.4%) | 11.338 (0.5%) |
| 1,048,576 B CPU seconds | 0.507 (3.3%) | 3.737 (2.1%) | 0.475 (2.6%) |
| 1,048,576 B device bytes written | 2,112,217,600 (1.0%) | 2,068,688,896 (1.1%) | 2,066,962,944 (0.1%) |
| 1,048,576 B device busy ms | 19,448 (0.3%) | 14,776 (1.5%) | 19,476 (0.1%) |

Cells show median (sample CV%). Each stage has five repetitions; XFS-pre and XFS-post are not pooled.

## dodb transaction-width sweep

| width and metric | XFS-pre | ZFS | XFS-post |
|---:|---:|---:|---:|
| width 1 tx/s | 16,568.7 (0.8%) | 22,986.5 (1.4%) | 16,458.3 (0.8%) |
| width 1 p50 us | 3,814.0 (0.7%) | 2,852.2 (1.4%) | 3,844.5 (0.6%) |
| width 1 p95 us | 4,537.0 (1.8%) | 3,343.1 (1.7%) | 4,554.3 (1.4%) |
| width 1 p99 us | 5,725.6 (1.2%) | 4,792.1 (4.5%) | 5,729.1 (1.6%) |
| width 1 WAL bytes | 18,806,028 (0.8%) | 26,079,851 (1.4%) | 18,675,862 (0.8%) |
| width 1 WAL sync count | 2,593 (0.9%) | 3,353 (1.4%) | 2,553 (0.4%) |
| width 1 WAL bytes/sync | 7,262.1 (0.5%) | 7,789.9 (0.8%) | 7,319.0 (0.6%) |
| width 1 WAL sync mean ms | 1.459 (1.1%) | 0.980 (1.2%) | 1.476 (0.8%) |
| width 1 successful tx/sync | 32.03 (0.5%) | 34.36 (0.8%) | 32.27 (0.6%) |
| width 1 average group size | 32.03 (0.5%) | 34.36 (0.8%) | 32.27 (0.6%) |
| width 1 groups/s | 518.08 (0.9%) | 670.20 (1.4%) | 510.19 (0.4%) |
| width 1 physical execution ms | 435.05 (1.0%) | 591.67 (1.7%) | 429.76 (0.9%) |
| width 1 planning ms | 186.45 (0.9%) | 247.42 (1.5%) | 186.50 (1.3%) |
| width 1 validation ms | 42.71 (1.2%) | 57.01 (1.3%) | 43.06 (4.2%) |
| width 1 publication ms | 4,556.09 (0.1%) | 4,391.67 (0.2%) | 4,554.73 (0.4%) |
| width 1 CPU seconds | 1.760 (1.0%) | 2.390 (0.9%) | 1.780 (2.2%) |
| width 1 device bytes written | 458,416,128 (0.1%) | 508,944,384 (0.9%) | 458,058,752 (0.1%) |
| width 1 device busy ms | 6,592 (1.5%) | 5,944 (1.7%) | 6,576 (1.3%) |
| width 1 errors | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) |
| width 2 tx/s | 13,928.3 (0.8%) | 18,427.1 (1.0%) | 14,072.3 (1.3%) |
| width 2 p50 us | 4,574.9 (0.8%) | 3,582.0 (0.9%) | 4,532.4 (1.3%) |
| width 2 p95 us | 5,256.5 (3.7%) | 4,283.9 (2.0%) | 5,222.0 (4.2%) |
| width 2 p99 us | 6,641.9 (4.0%) | 6,229.9 (5.4%) | 6,721.2 (3.1%) |
| width 2 WAL bytes | 26,861,883 (0.8%) | 35,531,188 (1.0%) | 27,134,200 (1.3%) |
| width 2 WAL sync count | 2,153 (1.7%) | 2,649 (0.8%) | 2,177 (3.2%) |
| width 2 WAL bytes/sync | 12,413.1 (1.6%) | 13,458.8 (0.4%) | 12,482.0 (3.2%) |
| width 2 WAL sync mean ms | 1.525 (2.0%) | 1.031 (0.4%) | 1.515 (1.5%) |
| width 2 successful tx/sync | 32.21 (1.6%) | 34.93 (0.4%) | 32.39 (3.2%) |
| width 2 average group size | 32.21 (1.6%) | 34.93 (0.4%) | 32.39 (3.2%) |
| width 2 groups/s | 430.17 (1.7%) | 529.48 (0.8%) | 435.05 (3.2%) |
| width 2 physical execution ms | 628.95 (1.4%) | 836.47 (1.1%) | 639.97 (5.5%) |
| width 2 planning ms | 278.80 (2.7%) | 353.87 (1.7%) | 280.60 (1.1%) |
| width 2 validation ms | 61.32 (1.6%) | 79.08 (2.1%) | 60.79 (4.2%) |
| width 2 publication ms | 4,399.82 (0.2%) | 4,227.54 (0.3%) | 4,395.05 (0.5%) |
| width 2 CPU seconds | 2.370 (1.1%) | 3.079 (0.7%) | 2.400 (1.3%) |
| width 2 device bytes written | 464,716,800 (0.1%) | 522,857,472 (1.2%) | 465,375,232 (0.2%) |
| width 2 device busy ms | 6,016 (2.6%) | 5,264 (1.2%) | 6,072 (2.8%) |
| width 2 errors | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) |
| width 4 tx/s | 11,205.0 (0.8%) | 13,347.4 (0.7%) | 11,287.4 (1.0%) |
| width 4 p50 us | 6,010.9 (0.9%) | 5,132.0 (0.6%) | 5,926.6 (0.9%) |
| width 4 p95 us | 6,773.3 (2.6%) | 6,575.0 (3.6%) | 6,970.6 (2.0%) |
| width 4 p99 us | 7,327.1 (1.6%) | 8,598.6 (10.7%) | 7,490.1 (14.3%) |
| width 4 WAL bytes | 39,375,037 (0.8%) | 46,948,065 (0.8%) | 39,692,118 (1.0%) |
| width 4 WAL sync count | 1,622 (1.3%) | 1,818 (0.8%) | 1,627 (2.3%) |
| width 4 WAL bytes/sync | 24,425.6 (1.2%) | 25,893.2 (0.5%) | 24,540.4 (2.5%) |
| width 4 WAL sync mean ms | 1.611 (1.7%) | 1.162 (0.3%) | 1.624 (1.6%) |
| width 4 successful tx/sync | 34.78 (1.3%) | 36.84 (0.5%) | 34.92 (2.5%) |
| width 4 average group size | 34.78 (1.3%) | 36.84 (0.5%) | 34.92 (2.5%) |
| width 4 groups/s | 324.14 (1.3%) | 363.31 (0.8%) | 325.01 (2.3%) |
| width 4 physical execution ms | 938.38 (0.9%) | 1,139.93 (0.8%) | 934.31 (5.5%) |
| width 4 planning ms | 412.37 (1.4%) | 476.78 (0.5%) | 399.74 (2.0%) |
| width 4 validation ms | 92.15 (0.9%) | 108.96 (1.1%) | 89.94 (1.0%) |
| width 4 publication ms | 4,170.86 (0.2%) | 4,023.02 (0.2%) | 4,179.31 (0.4%) |
| width 4 CPU seconds | 3.300 (0.6%) | 3.860 (0.4%) | 3.270 (0.9%) |
| width 4 device bytes written | 476,463,104 (0.2%) | 546,471,936 (0.5%) | 476,471,296 (0.2%) |
| width 4 device busy ms | 4,992 (1.6%) | 4,548 (1.7%) | 4,988 (1.7%) |
| width 4 errors | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) |
| width 8 tx/s | 8,046.9 (2.2%) | 7,085.6 (1.5%) | 8,284.4 (2.3%) |
| width 8 p50 us | 8,975.9 (1.4%) | 9,760.1 (1.4%) | 8,845.0 (0.9%) |
| width 8 p95 us | 10,145.2 (2.1%) | 11,640.8 (2.9%) | 10,044.8 (2.1%) |
| width 8 p99 us | 10,952.5 (3.2%) | 13,221.9 (4.4%) | 10,800.3 (8.7%) |
| width 8 WAL bytes | 53,889,726 (2.2%) | 47,438,913 (1.5%) | 55,525,801 (2.3%) |
| width 8 WAL sync count | 1,067 (2.2%) | 974 (2.4%) | 1,077 (2.9%) |
| width 8 WAL bytes/sync | 50,471.8 (1.6%) | 49,571.9 (2.6%) | 52,323.9 (2.1%) |
| width 8 WAL sync mean ms | 1.753 (2.2%) | 2.183 (2.8%) | 1.765 (1.4%) |
| width 8 successful tx/sync | 37.73 (1.6%) | 37.06 (2.6%) | 39.12 (2.1%) |
| width 8 average group size | 37.73 (1.6%) | 37.06 (2.6%) | 39.12 (2.1%) |
| width 8 groups/s | 213.19 (2.2%) | 194.53 (2.4%) | 215.06 (2.9%) |
| width 8 physical execution ms | 1,291.25 (1.5%) | 1,165.46 (1.5%) | 1,309.88 (3.3%) |
| width 8 planning ms | 561.24 (1.5%) | 502.72 (1.7%) | 543.04 (1.3%) |
| width 8 validation ms | 126.88 (0.9%) | 117.13 (2.1%) | 131.14 (2.1%) |
| width 8 publication ms | 3,922.89 (0.4%) | 4,041.37 (0.3%) | 3,947.09 (0.2%) |
| width 8 CPU seconds | 4.319 (0.4%) | 3.939 (0.7%) | 4.260 (1.4%) |
| width 8 device bytes written | 490,455,040 (0.3%) | 515,088,384 (0.5%) | 492,118,016 (0.3%) |
| width 8 device busy ms | 4,076 (2.4%) | 3,916 (2.2%) | 4,088 (2.8%) |
| width 8 errors | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) |
| width 16 tx/s | 5,012.8 (2.7%) | 4,513.2 (2.8%) | 5,013.8 (1.4%) |
| width 16 p50 us | 14,660.7 (17.1%) | 15,521.3 (4.9%) | 14,762.1 (1.4%) |
| width 16 p95 us | 17,112.8 (5.5%) | 20,222.9 (3.9%) | 17,422.0 (2.7%) |
| width 16 p99 us | 21,002.6 (8.5%) | 23,142.1 (9.3%) | 19,009.2 (7.4%) |
| width 16 WAL bytes | 65,413,019 (2.6%) | 59,289,698 (2.6%) | 65,554,782 (1.1%) |
| width 16 WAL sync count | 636 (6.8%) | 576 (4.0%) | 626 (2.7%) |
| width 16 WAL bytes/sync | 105,326.2 (6.3%) | 102,123.7 (4.1%) | 104,720.1 (3.6%) |
| width 16 WAL sync mean ms | 2.199 (6.7%) | 2.698 (6.6%) | 2.243 (2.3%) |
| width 16 successful tx/sync | 40.37 (6.6%) | 38.94 (4.3%) | 40.14 (3.9%) |
| width 16 average group size | 40.37 (6.6%) | 38.94 (4.3%) | 40.14 (3.9%) |
| width 16 groups/s | 127.07 (6.8%) | 114.86 (4.0%) | 124.93 (2.7%) |
| width 16 physical execution ms | 1,553.74 (2.1%) | 1,433.98 (2.0%) | 1,534.00 (0.5%) |
| width 16 planning ms | 670.63 (0.9%) | 632.95 (1.9%) | 664.05 (0.6%) |
| width 16 validation ms | 171.79 (0.9%) | 158.27 (1.9%) | 170.86 (0.9%) |
| width 16 publication ms | 3,738.40 (0.6%) | 3,830.63 (0.6%) | 3,761.91 (0.4%) |
| width 16 CPU seconds | 4.950 (0.8%) | 4.719 (2.0%) | 4.949 (0.7%) |
| width 16 device bytes written | 501,495,808 (0.4%) | 532,986,368 (0.6%) | 501,739,520 (0.3%) |
| width 16 device busy ms | 3,456 (3.6%) | 3,616 (3.0%) | 3,432 (1.9%) |
| width 16 errors | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) |

Cells show median (sample CV%). Each stage has five repetitions; XFS-pre and XFS-post are not pooled.

## WAL and grouping correlation table

Cells show median (sample CV%). Successful transactions per sync are computed from `successful_transactions / wal_syncs_delta`; WAL bytes per sync and WAL sync mean are computed per run before summarizing.

| Filesystem stage | Width | tx/s | WAL bytes/sync | WAL sync mean ms | Avg group | Groups/s |
|---|---:|---:|---:|---:|---:|---:|
| XFS-pre | 1 | 16,568.7 (0.8%) | 7,262.1 (0.5%) | 1.459 (1.1%) | 32.03 (0.5%) | 518.08 (0.9%) |
| XFS-pre | 2 | 13,928.3 (0.8%) | 12,413.1 (1.6%) | 1.525 (2.0%) | 32.21 (1.6%) | 430.17 (1.7%) |
| XFS-pre | 4 | 11,205.0 (0.8%) | 24,425.6 (1.2%) | 1.611 (1.7%) | 34.78 (1.3%) | 324.14 (1.3%) |
| XFS-pre | 8 | 8,046.9 (2.2%) | 50,471.8 (1.6%) | 1.753 (2.2%) | 37.73 (1.6%) | 213.19 (2.2%) |
| XFS-pre | 16 | 5,012.8 (2.7%) | 105,326.2 (6.3%) | 2.199 (6.7%) | 40.37 (6.6%) | 127.07 (6.8%) |
| ZFS | 1 | 22,986.5 (1.4%) | 7,789.9 (0.8%) | 0.980 (1.2%) | 34.36 (0.8%) | 670.20 (1.4%) |
| ZFS | 2 | 18,427.1 (1.0%) | 13,458.8 (0.4%) | 1.031 (0.4%) | 34.93 (0.4%) | 529.48 (0.8%) |
| ZFS | 4 | 13,347.4 (0.7%) | 25,893.2 (0.5%) | 1.162 (0.3%) | 36.84 (0.5%) | 363.31 (0.8%) |
| ZFS | 8 | 7,085.6 (1.5%) | 49,571.9 (2.6%) | 2.183 (2.8%) | 37.06 (2.6%) | 194.53 (2.4%) |
| ZFS | 16 | 4,513.2 (2.8%) | 102,123.7 (4.1%) | 2.698 (6.6%) | 38.94 (4.3%) | 114.86 (4.0%) |
| XFS-post | 1 | 16,458.3 (0.8%) | 7,319.0 (0.6%) | 1.476 (0.8%) | 32.27 (0.6%) | 510.19 (0.4%) |
| XFS-post | 2 | 14,072.3 (1.3%) | 12,482.0 (3.2%) | 1.515 (1.5%) | 32.39 (3.2%) | 435.05 (3.2%) |
| XFS-post | 4 | 11,287.4 (1.0%) | 24,540.4 (2.5%) | 1.624 (1.6%) | 34.92 (2.5%) | 325.01 (2.3%) |
| XFS-post | 8 | 8,284.4 (2.3%) | 52,323.9 (2.1%) | 1.765 (1.4%) | 39.12 (2.1%) | 215.06 (2.9%) |
| XFS-post | 16 | 5,013.8 (1.4%) | 104,720.1 (3.6%) | 2.243 (2.3%) | 40.14 (3.9%) | 124.93 (2.7%) |

## XFS-pre and XFS-post drift

Each value is the XFS-post median divided by the XFS-pre median. Positive percentages mean an increase in the metric; for latency metrics, that is slower.

| Workload | Point | Metric | XFS-pre median | XFS-post median | Post / pre |
|---|---:|---|---:|---:|---:|
| fio | 4,096 B | operations/s | 1,372.4128 | 1,399.4800 | 1.0197 (+2.0%) |
| fio | 4,096 B | fdatasync mean ms | 0.7173 | 0.7037 | 0.9811 (-1.9%) |
| fio | 4,096 B | fdatasync p50 ms | 0.6431 | 0.6267 | 0.9745 (-2.5%) |
| fio | 4,096 B | fdatasync p95 ms | 1.3189 | 1.3681 | 1.0373 (+3.7%) |
| fio | 4,096 B | fdatasync p99 ms | 1.5155 | 1.5483 | 1.0216 (+2.2%) |
| fio | 16,384 B | operations/s | 1,398.8301 | 1,384.2308 | 0.9896 (-1.0%) |
| fio | 16,384 B | fdatasync mean ms | 0.7019 | 0.7095 | 1.0109 (+1.1%) |
| fio | 16,384 B | fdatasync p50 ms | 0.7004 | 0.7086 | 1.0117 (+1.2%) |
| fio | 16,384 B | fdatasync p95 ms | 0.8806 | 0.9052 | 1.0279 (+2.8%) |
| fio | 16,384 B | fdatasync p99 ms | 1.0199 | 1.0445 | 1.0241 (+2.4%) |
| fio | 65,536 B | operations/s | 1,062.3969 | 1,051.7974 | 0.9900 (-1.0%) |
| fio | 65,536 B | fdatasync mean ms | 0.9238 | 0.9334 | 1.0104 (+1.0%) |
| fio | 65,536 B | fdatasync p50 ms | 0.9216 | 0.9298 | 1.0089 (+0.9%) |
| fio | 65,536 B | fdatasync p95 ms | 1.1223 | 1.1387 | 1.0146 (+1.5%) |
| fio | 65,536 B | fdatasync p99 ms | 1.2698 | 1.2861 | 1.0129 (+1.3%) |
| fio | 262,144 B | operations/s | 394.3303 | 395.2105 | 1.0022 (+0.2%) |
| fio | 262,144 B | fdatasync mean ms | 2.4852 | 2.4805 | 0.9981 (-0.2%) |
| fio | 262,144 B | fdatasync p50 ms | 1.8596 | 1.8268 | 0.9824 (-1.8%) |
| fio | 262,144 B | fdatasync p95 ms | 4.0141 | 4.0468 | 1.0082 (+0.8%) |
| fio | 262,144 B | fdatasync p99 ms | 4.1779 | 4.1779 | 1.0000 (+0.0%) |
| fio | 1,048,576 B | operations/s | 100.6849 | 98.4754 | 0.9781 (-2.2%) |
| fio | 1,048,576 B | fdatasync mean ms | 9.7272 | 9.9451 | 1.0224 (+2.2%) |
| fio | 1,048,576 B | fdatasync p50 ms | 11.0756 | 11.0756 | 1.0000 (+0.0%) |
| fio | 1,048,576 B | fdatasync p95 ms | 11.2067 | 11.2067 | 1.0000 (+0.0%) |
| fio | 1,048,576 B | fdatasync p99 ms | 11.3377 | 11.3377 | 1.0000 (+0.0%) |
| dodb | 1 | tx/s | 16,568.6773 | 16,458.2700 | 0.9933 (-0.7%) |
| dodb | 1 | p50 us | 3,813.9930 | 3,844.5130 | 1.0080 (+0.8%) |
| dodb | 1 | p95 us | 4,537.0390 | 4,554.3190 | 1.0038 (+0.4%) |
| dodb | 1 | p99 us | 5,725.5690 | 5,729.1290 | 1.0006 (+0.1%) |
| dodb | 1 | WAL sync mean ms | 1.4587 | 1.4757 | 1.0117 (+1.2%) |
| dodb | 2 | tx/s | 13,928.2539 | 14,072.2783 | 1.0103 (+1.0%) |
| dodb | 2 | p50 us | 4,574.8790 | 4,532.4390 | 0.9907 (-0.9%) |
| dodb | 2 | p95 us | 5,256.5250 | 5,222.0440 | 0.9934 (-0.7%) |
| dodb | 2 | p99 us | 6,641.8970 | 6,721.1780 | 1.0119 (+1.2%) |
| dodb | 2 | WAL sync mean ms | 1.5251 | 1.5147 | 0.9932 (-0.7%) |
| dodb | 4 | tx/s | 11,204.9765 | 11,287.3634 | 1.0074 (+0.7%) |
| dodb | 4 | p50 us | 6,010.8920 | 5,926.6500 | 0.9860 (-1.4%) |
| dodb | 4 | p95 us | 6,773.2980 | 6,970.6190 | 1.0291 (+2.9%) |
| dodb | 4 | p99 us | 7,327.1030 | 7,490.1440 | 1.0223 (+2.2%) |
| dodb | 4 | WAL sync mean ms | 1.6110 | 1.6243 | 1.0083 (+0.8%) |
| dodb | 8 | tx/s | 8,046.9114 | 8,284.4156 | 1.0295 (+3.0%) |
| dodb | 8 | p50 us | 8,975.8770 | 8,844.9560 | 0.9854 (-1.5%) |
| dodb | 8 | p95 us | 10,145.1660 | 10,044.8060 | 0.9901 (-1.0%) |
| dodb | 8 | p99 us | 10,952.4530 | 10,800.2920 | 0.9861 (-1.4%) |
| dodb | 8 | WAL sync mean ms | 1.7525 | 1.7648 | 1.0070 (+0.7%) |
| dodb | 16 | tx/s | 5,012.8377 | 5,013.8204 | 1.0002 (+0.0%) |
| dodb | 16 | p50 us | 14,660.6860 | 14,762.0860 | 1.0069 (+0.7%) |
| dodb | 16 | p95 us | 17,112.7870 | 17,422.0290 | 1.0181 (+1.8%) |
| dodb | 16 | p99 us | 21,002.5800 | 19,009.1630 | 0.9051 (-9.5%) |
| dodb | 16 | WAL sync mean ms | 2.1985 | 2.2427 | 1.0201 (+2.0%) |

## Median gaps and repeat variation

This descriptive check compares the absolute gap between stage medians with the root mean square of the two stages' sample standard deviations. It is not a significance test.

| Metric | Point | XFS stage median | ZFS median | Gap | Combined within-stage SD | Gap exceeds SD |
|---|---:|---:|---:|---:|---:|:---:|
| fio operations/s | 4,096 B | XFS-pre 1,372.4 | 1,270.4 | 102.0 | 211.3 | No |
| fio operations/s | 4,096 B | XFS-post 1,399.5 | 1,270.4 | 129.1 | 214.9 | No |
| fio operations/s | 16,384 B | XFS-pre 1,398.8 | 1,079.4 | 319.4 | 205.0 | Yes |
| fio operations/s | 16,384 B | XFS-post 1,384.2 | 1,079.4 | 304.8 | 207.0 | Yes |
| fio operations/s | 65,536 B | XFS-pre 1,062.4 | 429.6 | 632.8 | 42.0 | Yes |
| fio operations/s | 65,536 B | XFS-post 1,051.8 | 429.6 | 622.2 | 41.4 | Yes |
| fio operations/s | 262,144 B | XFS-pre 394.3 | 250.0 | 144.4 | 2.9 | Yes |
| fio operations/s | 262,144 B | XFS-post 395.2 | 250.0 | 145.3 | 2.7 | Yes |
| fio operations/s | 1,048,576 B | XFS-pre 100.7 | 91.7 | 9.0 | 1.0 | Yes |
| fio operations/s | 1,048,576 B | XFS-post 98.5 | 91.7 | 6.8 | 0.7 | Yes |
| dodb tx/s | 1 | XFS-pre 16,568.7 | 22,986.5 | 6,417.9 | 240.7 | Yes |
| dodb WAL sync mean ms | 1 | XFS-pre 1.459 | 0.980 | 0.479 | 0.014 | Yes |
| dodb tx/s | 1 | XFS-post 16,458.3 | 22,986.5 | 6,528.3 | 244.0 | Yes |
| dodb WAL sync mean ms | 1 | XFS-post 1.476 | 0.980 | 0.496 | 0.012 | Yes |
| dodb tx/s | 2 | XFS-pre 13,928.3 | 18,427.1 | 4,498.8 | 155.7 | Yes |
| dodb WAL sync mean ms | 2 | XFS-pre 1.525 | 1.031 | 0.494 | 0.022 | Yes |
| dodb tx/s | 2 | XFS-post 14,072.3 | 18,427.1 | 4,354.8 | 183.1 | Yes |
| dodb WAL sync mean ms | 2 | XFS-post 1.515 | 1.031 | 0.484 | 0.016 | Yes |
| dodb tx/s | 4 | XFS-pre 11,205.0 | 13,347.4 | 2,142.5 | 95.0 | Yes |
| dodb WAL sync mean ms | 4 | XFS-pre 1.611 | 1.162 | 0.449 | 0.019 | Yes |
| dodb tx/s | 4 | XFS-post 11,287.4 | 13,347.4 | 2,060.1 | 107.0 | Yes |
| dodb WAL sync mean ms | 4 | XFS-post 1.624 | 1.162 | 0.463 | 0.018 | Yes |
| dodb tx/s | 8 | XFS-pre 8,046.9 | 7,085.6 | 961.3 | 147.0 | Yes |
| dodb WAL sync mean ms | 8 | XFS-pre 1.753 | 2.183 | 0.430 | 0.052 | Yes |
| dodb tx/s | 8 | XFS-post 8,284.4 | 7,085.6 | 1,198.8 | 152.6 | Yes |
| dodb WAL sync mean ms | 8 | XFS-post 1.765 | 2.183 | 0.418 | 0.047 | Yes |
| dodb tx/s | 16 | XFS-pre 5,012.8 | 4,513.2 | 499.6 | 130.0 | Yes |
| dodb WAL sync mean ms | 16 | XFS-pre 2.199 | 2.698 | 0.500 | 0.168 | Yes |
| dodb tx/s | 16 | XFS-post 5,013.8 | 4,513.2 | 500.6 | 101.3 | Yes |
| dodb WAL sync mean ms | 16 | XFS-post 2.243 | 2.698 | 0.456 | 0.134 | Yes |

## Observed crossover relationships

Median fio operations/s favors XFS-pre and XFS-post at every measured payload from 4,096 through 1,048,576 bytes. No payload-size refinement was triggered because the median ordering did not reverse between adjacent points.
XFS fio repetition spread is high at these points: XFS-pre 4,096 B CV 23.6% (rep1 784.4 ops/s, reps2–5 median 1,427.4); XFS-pre 16,384 B CV 23.0% (rep1 746.7 ops/s, reps2–5 median 1,406.7); XFS-post 4,096 B CV 23.6% (rep1 793.2 ops/s, reps2–5 median 1,447.2); XFS-post 16,384 B CV 23.1% (rep1 742.2 ops/s, reps2–5 median 1,385.5). The median rank is therefore less stable there than at points with lower CV.
For XFS-pre, median throughput changes from ZFS ahead at width 4 to XFS ahead at width 8. At the lower width, WAL bytes/sync are 24,426 for XFS and 25,893 for ZFS; WAL sync mean is 1.611 ms for XFS and 1.162 ms for ZFS. At the upper width, WAL bytes/sync are 50,472 for XFS and 49,572 for ZFS; WAL sync mean is 1.753 ms for XFS and 2.183 ms for ZFS.
At these widths, the WAL bytes/sync medians are similar across filesystems, while WAL sync mean and groups/s change ordering with throughput. This is consistent with dodb's observed sync/group dynamics being more informative than the fio payload-size ranking in this range; it does not establish a causal mechanism.
For XFS-post, median throughput changes from ZFS ahead at width 4 to XFS ahead at width 8. At the lower width, WAL bytes/sync are 24,540 for XFS and 25,893 for ZFS; WAL sync mean is 1.624 ms for XFS and 1.162 ms for ZFS. At the upper width, WAL bytes/sync are 52,324 for XFS and 49,572 for ZFS; WAL sync mean is 1.765 ms for XFS and 2.183 ms for ZFS.
At these widths, the WAL bytes/sync medians are similar across filesystems, while WAL sync mean and groups/s change ordering with throughput. This is consistent with dodb's observed sync/group dynamics being more informative than the fio payload-size ranking in this range; it does not establish a causal mechanism.

## Crossover locations

The initial fio sweep crosses only when the XFS-pre or XFS-post median operations-per-second ordering changes relative to ZFS between adjacent payload sizes. The dodb crossover is determined the same way from adjacent transaction widths. See `crossover-analysis.json` for the medians and intervals.

- fio XFS-pre: no observed operations-per-second crossover in the measured payload range.
- fio XFS-post: no observed operations-per-second crossover in the measured payload range.
- dodb XFS-pre: [{'lower_point': 4, 'upper_point': 8}].
- dodb XFS-post: [{'lower_point': 4, 'upper_point': 8}].

## Interpretation boundary

The results describe these two filesystems, this OCI storage device, this binary, and these diagnostic workloads. Fio payload points and dodb widths are not equivalent operations. No fio median throughput ranking reversal was observed from 4 KiB through 1 MiB, while dodb throughput reversed between widths 4 and 8 at roughly 25–52 KiB per WAL sync. Therefore, this sweep does not support a simple standalone sync-payload crossover as the explanation for dodb's ranking change. At the dodb crossover, WAL sync mean and groups/s reverse in the same direction as throughput while WAL bytes/sync remain similar between filesystems; that association is descriptive and does not establish the mechanism.
