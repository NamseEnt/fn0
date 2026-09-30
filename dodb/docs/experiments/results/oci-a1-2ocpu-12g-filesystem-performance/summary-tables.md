# Filesystem performance summary

Each cell reports the median and sample coefficient of variation (CV = sample standard deviation / mean × 100%). fio and dodb use five repetitions per filesystem; aggregate Btrfs cells pool Btrfs-pre and Btrfs-post (ten repetitions).

## Requested comparison table

| Metric | ext4 | XFS | Btrfs | ZFS |
|---|---:|---:|---:|---:|
| fio fdatasync IOPS (IOPS) | 972.7 (33.7%) | 1,345.2 (24.2%) | 364.8 (3.0%) | 1,275.6 (0.5%) |
| fio fdatasync p50 (ms) | 0.676 (61.2%) | 0.651 (37.0%) | 2.703 (4.2%) | 0.725 (0.8%) |
| fio fdatasync p99 (ms) | 3.228 (40.6%) | 1.565 (12.1%) | 3.604 (3.6%) | 1.401 (2.1%) |
| 4 KiB overwrite IOPS (IOPS) | 1,344.9 (26.7%) | 1,438.2 (14.9%) | 353.1 (2.1%) | 1,287.4 (1.0%) |
| dodb width1 throughput (tx/s) | 11,555.6 (2.9%) | 16,481.5 (1.6%) | 9,617.3 (0.8%) | 23,344.1 (1.3%) |
| dodb width1 p99 (us) | 8,239.0 (8.8%) | 5,770.5 (2.3%) | 9,987.4 (1.7%) | 4,915.1 (5.0%) |
| dodb width16 throughput (tx/s) | 4,396.4 (1.9%) | 4,949.4 (1.5%) | 4,177.1 (1.2%) | 4,517.3 (3.6%) |
| dodb width16 p99 (us) | 22,387.9 (7.0%) | 20,046.1 (7.0%) | 23,569.0 (6.1%) | 24,063.1 (7.2%) |

## fio sequential durable write

| Metric | ext4 | XFS | Btrfs | ZFS |
|---|---:|---:|---:|---:|
| IOPS (IOPS) | 972.7 (33.7%) | 1,345.2 (24.2%) | 364.8 (3.0%) | 1,275.6 (0.5%) |
| Throughput (MB/s) | 3.98 (33.7%) | 5.51 (24.2%) | 1.49 (3.0%) | 5.23 (0.5%) |
| fdatasync mean (ms) | 1.015 (45.5%) | 0.732 (31.6%) | 2.721 (3.0%) | 0.765 (0.6%) |
| fdatasync p50 (ms) | 0.676 (61.2%) | 0.651 (37.0%) | 2.703 (4.2%) | 0.725 (0.8%) |
| fdatasync p95 (ms) | 2.179 (13.6%) | 1.335 (21.0%) | 3.244 (2.1%) | 1.044 (1.5%) |
| fdatasync p99 (ms) | 3.228 (40.6%) | 1.565 (12.1%) | 3.604 (3.6%) | 1.401 (2.1%) |
| fio process CPU (CPU-s) | 0.647 (16.5%) | 0.795 (2.1%) | 1.812 (2.0%) | 0.981 (1.9%) |
| device bytes written (bytes) | 605,925,376 (7.6%) | 126,750,208 (1.2%) | 1,220,800,512 (2.5%) | 318,355,968 (1.6%) |
| device busy time (ms) | 18,144 (0.6%) | 18,812 (0.4%) | 17,970 (0.6%) | 17,236 (0.7%) |

## fio 4 KiB random overwrite

| Metric | ext4 | XFS | Btrfs | ZFS |
|---|---:|---:|---:|---:|
| IOPS (IOPS) | 1,344.9 (26.7%) | 1,438.2 (14.9%) | 353.1 (2.1%) | 1,287.4 (1.0%) |
| Throughput (MB/s) | 5.51 (26.7%) | 5.89 (14.9%) | 1.45 (2.1%) | 5.27 (1.0%) |
| write p50 (ms) | 0.009 (10.5%) | 0.009 (2.2%) | 0.022 (13.4%) | 0.016 (2.4%) |
| write p95 (ms) | 0.015 (21.4%) | 0.015 (5.0%) | 0.033 (5.2%) | 0.030 (2.6%) |
| write p99 (ms) | 0.022 (12.5%) | 0.019 (3.7%) | 0.041 (9.9%) | 0.039 (3.4%) |
| fdatasync mean (ms) | 0.732 (36.5%) | 0.683 (18.4%) | 2.802 (2.1%) | 0.756 (0.9%) |
| fdatasync p50 (ms) | 0.643 (59.8%) | 0.651 (11.1%) | 2.802 (3.2%) | 0.717 (0.8%) |
| fdatasync p95 (ms) | 1.892 (42.6%) | 0.881 (32.3%) | 3.342 (1.8%) | 1.028 (1.8%) |
| fdatasync p99 (ms) | 2.179 (19.6%) | 1.384 (16.1%) | 3.686 (2.5%) | 1.417 (2.2%) |
| fio process CPU (CPU-s) | 0.720 (10.3%) | 0.816 (3.4%) | 1.877 (3.0%) | 1.040 (2.7%) |
| device bytes written (bytes) | 142,679,040 (16.5%) | 125,167,104 (13.7%) | 1,236,598,272 (2.3%) | 320,640,512 (1.8%) |
| device busy time (ms) | 18,848 (0.6%) | 18,824 (0.5%) | 17,940 (0.6%) | 17,280 (0.6%) |

## dodb width 1

| Metric | ext4 | XFS | Btrfs | ZFS |
|---|---:|---:|---:|---:|
| Throughput (tx/s) | 11,555.6 (2.9%) | 16,481.5 (1.6%) | 9,617.3 (0.8%) | 23,344.1 (1.3%) |
| p50 (us) | 5,669.6 (1.8%) | 3,819.6 (1.5%) | 6,563.8 (0.8%) | 2,819.9 (1.7%) |
| p95 (us) | 6,429.6 (1.8%) | 4,731.5 (0.8%) | 7,502.5 (1.9%) | 3,375.7 (2.2%) |
| p99 (us) | 8,239.0 (8.8%) | 5,770.5 (2.3%) | 9,987.4 (1.7%) | 4,915.1 (5.0%) |
| process CPU (CPU-s) | 1.240 (2.5%) | 1.740 (0.6%) | 1.360 (1.4%) | 2.390 (0.3%) |
| WAL sync count (count) | 1,736 (1.5%) | 2,524 (1.2%) | 1,511 (1.2%) | 3,361 (1.9%) |
| WAL sync time (ms) | 4,132.54 (0.6%) | 3,797.14 (0.3%) | 4,233.18 (0.2%) | 3,255.51 (0.6%) |
| average group size (tx/group) | 33.07 (1.5%) | 32.21 (1.2%) | 31.82 (0.5%) | 34.91 (0.7%) |
| groups per second (groups/s) | 346.99 (1.5%) | 504.50 (1.2%) | 301.87 (1.2%) | 671.66 (1.9%) |
| physical execution time (ms) | 301.54 (2.1%) | 434.64 (1.9%) | 263.44 (1.2%) | 597.83 (1.0%) |
| device bytes written (bytes) | 511,455,232 (1.2%) | 458,089,984 (0.1%) | 791,662,592 (1.0%) | 511,864,832 (2.2%) |
| device busy time (ms) | 7,092 (1.0%) | 6,604 (0.8%) | 7,418 (1.1%) | 6,064 (0.8%) |
| errors (count) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) |

## dodb width 16

| Metric | ext4 | XFS | Btrfs | ZFS |
|---|---:|---:|---:|---:|
| Throughput (tx/s) | 4,396.4 (1.9%) | 4,949.4 (1.5%) | 4,177.1 (1.2%) | 4,517.3 (3.6%) |
| p50 (us) | 17,014.2 (1.8%) | 14,938.5 (2.0%) | 17,563.4 (1.0%) | 15,547.1 (2.3%) |
| p95 (us) | 18,716.8 (4.7%) | 17,573.8 (3.4%) | 19,479.3 (2.8%) | 20,502.4 (5.7%) |
| p99 (us) | 22,387.9 (7.0%) | 20,046.1 (7.0%) | 23,569.0 (6.1%) | 24,063.1 (7.2%) |
| process CPU (CPU-s) | 4.459 (0.9%) | 4.919 (0.6%) | 4.290 (0.9%) | 4.689 (2.6%) |
| WAL sync count (count) | 561 (4.1%) | 617 (2.4%) | 544 (2.0%) | 579 (4.3%) |
| WAL sync time (ms) | 1,739.65 (3.0%) | 1,402.72 (1.1%) | 1,933.08 (1.9%) | 1,578.67 (3.2%) |
| average group size (tx/group) | 39.23 (3.2%) | 40.24 (2.7%) | 38.14 (2.0%) | 39.19 (2.3%) |
| groups per second (groups/s) | 111.98 (4.1%) | 123.10 (2.4%) | 108.55 (2.0%) | 115.59 (4.3%) |
| physical execution time (ms) | 1,381.73 (1.3%) | 1,525.44 (0.9%) | 1,295.93 (1.1%) | 1,440.92 (1.0%) |
| device bytes written (bytes) | 510,001,152 (0.4%) | 501,219,840 (0.3%) | 619,820,544 (0.7%) | 528,039,936 (0.9%) |
| device busy time (ms) | 3,976 (2.3%) | 3,644 (4.2%) | 4,388 (1.8%) | 3,532 (2.7%) |
| errors (count) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) | 0 (0.0%) |

## Fastest filesystem and repeat variation

Fastest means highest IOPS/throughput or lowest latency. For the variation check, the absolute median gap between the top two filesystems is compared with the root mean square of their sample standard deviations. This is a descriptive comparison with five repetitions per stage, not a significance test.

| Metric | Fastest median | Next median | Gap | Combined within-filesystem SD | Gap exceeds SD |
|---|---:|---:|---:|---:|:---:|
| fio fdatasync IOPS | XFS 1,345.2 IOPS | ZFS 1,275.6 IOPS | 69.5 IOPS | 213.1 IOPS | No |
| fio fdatasync throughput | XFS 5.51 MB/s | ZFS 5.23 MB/s | 0.28 MB/s | 0.87 MB/s | No |
| fio fdatasync mean | XFS 0.732 ms | ZFS 0.765 ms | 0.033 ms | 0.189 ms | No |
| fio fdatasync p50 | XFS 0.651 ms | ext4 0.676 ms | 0.025 ms | 0.459 ms | No |
| fio fdatasync p95 | ZFS 1.044 ms | XFS 1.335 ms | 0.291 ms | 0.194 ms | Yes |
| fio fdatasync p99 | ZFS 1.401 ms | XFS 1.565 ms | 0.164 ms | 0.139 ms | Yes |
| 4 KiB overwrite IOPS | XFS 1,438.2 IOPS | ext4 1,344.9 IOPS | 93.3 IOPS | 270.6 IOPS | No |
| 4 KiB overwrite write p50 | XFS 0.009 ms | ext4 0.009 ms | 0.000 ms | 0.001 ms | No |
| 4 KiB overwrite write p95 | ext4 0.015 ms | XFS 0.015 ms | 0.001 ms | 0.003 ms | No |
| 4 KiB overwrite write p99 | XFS 0.019 ms | ext4 0.022 ms | 0.004 ms | 0.002 ms | Yes |
| dodb width1 throughput | ZFS 23,344.1 tx/s | XFS 16,481.5 tx/s | 6,862.6 tx/s | 285.4 tx/s | Yes |
| dodb width1 p50 | ZFS 2,819.9 us | XFS 3,819.6 us | 999.6 us | 52.3 us | Yes |
| dodb width1 p95 | ZFS 3,375.7 us | XFS 4,731.5 us | 1,355.8 us | 60.2 us | Yes |
| dodb width1 p99 | ZFS 4,915.1 us | XFS 5,770.5 us | 855.4 us | 199.7 us | Yes |
| dodb width16 throughput | XFS 4,949.4 tx/s | ZFS 4,517.3 tx/s | 432.1 tx/s | 125.7 tx/s | Yes |
| dodb width16 p50 | XFS 14,938.5 us | ZFS 15,547.1 us | 608.6 us | 332.6 us | Yes |
| dodb width16 p95 | XFS 17,573.8 us | ext4 18,716.8 us | 1,143.0 us | 754.3 us | Yes |
| dodb width16 p99 | XFS 20,046.1 us | ext4 22,387.9 us | 2,341.8 us | 1,500.0 us | Yes |

By median, the slowest filesystem across these direct throughput and latency metrics is Btrfs 16/18, ZFS 2/18. The metric breakdown is Btrfs: fio fdatasync IOPS, fio fdatasync throughput, fio fdatasync mean, fio fdatasync p50, fio fdatasync p95, fio fdatasync p99, 4 KiB overwrite IOPS, 4 KiB overwrite write p50, 4 KiB overwrite write p95, 4 KiB overwrite write p99, dodb width1 throughput, dodb width1 p50, dodb width1 p95, dodb width1 p99, dodb width16 throughput, dodb width16 p50; ZFS: dodb width16 p95, dodb width16 p99.

The `Btrfs` column in the aggregate tables pools five Btrfs-pre and five Btrfs-post repetitions. The separate drift table shows the change between those stages.

## Btrfs pre/post drift

Percent change is `(Btrfs-post median / Btrfs-pre median - 1) × 100`. Positive changes mean a higher metric; for latency, a positive change is slower.

| Metric | Btrfs-pre median (CV) | Btrfs-post median (CV) | Post vs pre | Gap exceeds pooled SD |
|---|---:|---:|---:|:---:|
| fio sequential IOPS (IOPS) | 355.5 (3.0%) | 367.6 (2.3%) | +3.4% | Yes |
| fio sequential fdatasync mean (ms) | 2.794 (3.0%) | 2.695 (2.3%) | -3.5% | Yes |
| fio sequential fdatasync p50 (ms) | 2.769 (4.5%) | 2.638 (3.5%) | -4.7% | Yes |
| fio sequential fdatasync p99 (ms) | 3.621 (3.6%) | 3.555 (2.1%) | -1.8% | No |
| 4 KiB overwrite IOPS (IOPS) | 350.4 (1.1%) | 361.5 (2.2%) | +3.2% | Yes |
| dodb width1 throughput (tx/s) | 9,602.5 (0.9%) | 9,653.1 (0.7%) | +0.5% | No |
| dodb width1 p99 (us) | 10,010.9 (2.0%) | 9,985.4 (1.1%) | -0.3% | No |
| dodb width16 throughput (tx/s) | 4,097.3 (1.5%) | 4,179.8 (0.5%) | +2.0% | Yes |
| dodb width16 p99 (us) | 24,069.7 (6.5%) | 23,312.7 (5.2%) | -3.1% | No |
