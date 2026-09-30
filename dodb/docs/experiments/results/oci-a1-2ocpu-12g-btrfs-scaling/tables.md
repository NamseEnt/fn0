# A1 Btrfs durable-write scaling tables

All throughput, latency, CPU, group, and timing columns are medians across three repetitions. Latencies are end-to-end transaction latencies. `worker parallelism` is dodb's measured effective worker parallelism or RocksDB process CPU-seconds divided by wall-seconds.

## Throughput, speedup, and dodb/RocksDB ratio

| OCPU | Workload | dodb tx/s | dodb speedup | RocksDB tx/s | RocksDB speedup | dodb / RocksDB |
|---:|---|---:|---:|---:|---:|---:|
| 2 | width1-uniform | 10675.7 | 1.000x | 10102.7 | 1.000x | 1.057x |
| 2 | width16-uniform | 4641.7 | 1.000x | 8046.1 | 1.000x | 0.577x |
| 2 | width16-compact | 9607.3 | 1.000x | 8376.3 | 1.000x | 1.147x |
| 2 | width16-spread | 8290.5 | 1.000x | 8471.9 | 1.000x | 0.979x |
| 4 | width1-uniform | 11830.9 | 1.108x | 11996.4 | 1.187x | 0.986x |
| 4 | width16-uniform | 6006.4 | 1.294x | 9227.9 | 1.147x | 0.651x |
| 4 | width16-compact | 12927.7 | 1.346x | 9784.8 | 1.168x | 1.321x |
| 4 | width16-spread | 10911.1 | 1.316x | 10027.2 | 1.184x | 1.088x |
| 6 | width1-uniform | 10525.7 | 0.986x | 11280.9 | 1.117x | 0.933x |
| 6 | width16-uniform | 6017.2 | 1.296x | 9051.0 | 1.125x | 0.665x |
| 6 | width16-compact | 12263.1 | 1.276x | 9227.9 | 1.102x | 1.329x |
| 6 | width16-spread | 10783.4 | 1.301x | 9325.7 | 1.101x | 1.156x |
| 2-post | width1-uniform | 9635.3 | 0.903x | 9230.9 | 0.914x | 1.044x |
| 2-post | width16-uniform | 4273.6 | 0.921x | 7267.8 | 0.903x | 0.588x |

## CPU and durability pipeline metrics

| OCPU | Workload | Engine | CPU s | CPU % of VM | Worker parallelism | p50/p95/p99 us | Avg tx/sync group | Groups/syncs per s | WAL sync mean/p50/p95/p99 us |
|---:|---|---|---:|---:|---:|---|---:|---:|---|
| 2 | width1-uniform | dodb | 1.42 | 14.2% | 1.62 | 5972/7032/9102 | 32.13 | 325.6 | 2549/—/—/— |
| 2 | width1-uniform | rocksdb | 1.75 | 17.5% | 0.35 | 6569/7405/7907 | 33.70 | 299.0 | 2975/3118/4278/4381 |
| 2 | width16-uniform | dodb | 4.50 | 44.9% | 1.92 | 16403/18711/21693 | 41.32 | 113.3 | 3188/—/—/— |
| 2 | width16-uniform | rocksdb | 3.72 | 37.1% | 0.74 | 8154/10135/12511 | 35.32 | 227.8 | 3228/3511/4336/5073 |
| 2 | width16-compact | dodb | 2.78 | 27.8% | 1.66 | 7877/8659/9180 | 39.57 | 242.6 | 2515/—/—/— |
| 2 | width16-compact | rocksdb | 2.89 | 28.9% | 0.58 | 7887/9232/11062 | 35.07 | 240.0 | 3403/3616/4336/4399 |
| 2 | width16-spread | dodb | 2.99 | 29.9% | 1.83 | 9032/10007/10612 | 39.45 | 210.3 | 2761/—/—/— |
| 2 | width16-spread | rocksdb | 2.95 | 29.5% | 0.59 | 7896/9433/11438 | 35.80 | 236.6 | 3378/3621/4344/5366 |
| 4 | width1-uniform | dodb | 1.77 | 8.8% | 2.62 | 5360/6224/8076 | 32.29 | 366.4 | 2274/—/—/— |
| 4 | width1-uniform | rocksdb | 2.29 | 11.4% | 0.46 | 5343/6043/6410 | 32.53 | 367.8 | 2474/2419/2886/4078 |
| 4 | width16-uniform | dodb | 5.99 | 29.9% | 3.77 | 10293/11205/20529 | 62.70 | 95.9 | 3183/—/—/— |
| 4 | width16-uniform | rocksdb | 4.51 | 22.5% | 0.90 | 7014/7946/10900 | 33.30 | 277.1 | 2971/2966/4275/5326 |
| 4 | width16-compact | dodb | 4.08 | 20.4% | 2.36 | 4830/5258/9372 | 62.91 | 205.9 | 2399/—/—/— |
| 4 | width16-compact | rocksdb | 3.79 | 18.9% | 0.76 | 6567/7352/7926 | 33.04 | 296.2 | 2886/2684/4203/4370 |
| 4 | width16-spread | dodb | 4.20 | 21.0% | 3.52 | 5679/6177/11103 | 62.84 | 176.1 | 2813/—/—/— |
| 4 | width16-spread | rocksdb | 3.67 | 18.3% | 0.73 | 6402/7192/7871 | 32.79 | 305.8 | 2842/2615/4167/4369 |
| 6 | width1-uniform | dodb | 1.76 | 5.9% | 2.86 | 5962/7189/9279 | 32.25 | 326.1 | 2579/—/—/— |
| 6 | width1-uniform | rocksdb | 2.44 | 8.1% | 0.49 | 5605/6537/7029 | 32.21 | 350.7 | 2620/2479/3870/4320 |
| 6 | width16-uniform | dodb | 6.23 | 20.7% | 5.47 | 10243/11528/20508 | 62.71 | 96.0 | 3434/—/—/— |
| 6 | width16-uniform | rocksdb | 4.74 | 15.8% | 0.95 | 7065/8089/8877 | 32.71 | 276.7 | 3085/3333/4302/4387 |
| 6 | width16-compact | dodb | 3.92 | 13.1% | 2.42 | 5110/5627/9864 | 62.92 | 194.9 | 2663/—/—/— |
| 6 | width16-compact | rocksdb | 3.89 | 13.0% | 0.78 | 6940/7871/8552 | 32.67 | 282.0 | 3116/3371/4308/4392 |
| 6 | width16-spread | dodb | 4.17 | 13.9% | 4.85 | 5793/6482/11304 | 62.85 | 172.2 | 3064/—/—/— |
| 6 | width16-spread | rocksdb | 3.80 | 12.7% | 0.76 | 6766/7687/8693 | 32.33 | 288.5 | 3081/3299/4300/4389 |
| 2-post | width1-uniform | dodb | 1.33 | 13.3% | 1.56 | 6520/7671/9995 | 31.79 | 303.1 | 2787/—/—/— |
| 2-post | width1-uniform | rocksdb | 1.70 | 17.0% | 0.34 | 7159/8077/8733 | 33.70 | 274.1 | 3269/3538/4319/4390 |
| 2-post | width16-uniform | dodb | 4.36 | 43.5% | 1.89 | 17365/19950/22081 | 39.17 | 107.2 | 3513/—/—/— |
| 2-post | width16-uniform | rocksdb | 3.62 | 36.2% | 0.72 | 8994/11077/13224 | 35.31 | 207.7 | 3612/3667/4360/5901 |

## dodb physical work, commit, and block I/O

All timing columns are total milliseconds per five-second measurement window, except WAL sync count, which is the total count in that window. Block bytes and busy time are deltas from the guest block-device counters during each run.

| OCPU | Workload | Physical | Planning | Validation | WAL assembly | Publication total | State install | Generation publish | Publish swap | WAL sync count | WAL sync total ms | Disk bytes written | Block busy ms |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 2 | width1-uniform | 272.4 | 127.3 | 30.2 | 6.3 | 4698.8 | 14.5 | 0.8 | 0.1 | 1630 | 4164.8 | 815443968 | 7512 |
| 2 | width16-uniform | 1358.7 | 612.9 | 160.6 | 11.1 | 3898.3 | 53.4 | 0.4 | 0.0 | 568 | 1801.4 | 628445184 | 4140 |
| 2 | width16-compact | 545.8 | 650.1 | 255.4 | 4.8 | 3749.5 | 1.4 | 0.7 | 0.0 | 1214 | 3048.3 | 434073600 | 4900 |
| 2 | width16-spread | 584.9 | 621.7 | 245.3 | 5.7 | 3772.0 | 14.3 | 0.6 | 0.0 | 1053 | 2904.1 | 446746624 | 4052 |
| 4 | width1-uniform | 236.0 | 132.2 | 31.3 | 8.1 | 4668.9 | 15.6 | 0.8 | 0.1 | 1833 | 4148.9 | 874688512 | 7428 |
| 4 | width16-uniform | 1094.5 | 765.6 | 205.1 | 15.1 | 3551.6 | 63.7 | 0.3 | 0.0 | 480 | 1526.9 | 639922176 | 3412 |
| 4 | width16-compact | 542.4 | 876.7 | 329.4 | 5.9 | 3189.2 | 1.3 | 0.5 | 0.0 | 1030 | 2468.0 | 392024064 | 3672 |
| 4 | width16-spread | 456.5 | 820.5 | 330.4 | 8.2 | 3279.6 | 18.4 | 0.4 | 0.0 | 881 | 2478.4 | 433889280 | 3724 |
| 6 | width1-uniform | 209.2 | 120.2 | 27.8 | 7.1 | 4695.3 | 14.8 | 0.7 | 0.1 | 1632 | 4237.1 | 824303616 | 7364 |
| 6 | width16-uniform | 900.0 | 794.6 | 210.2 | 15.1 | 3524.5 | 67.9 | 0.3 | 0.0 | 480 | 1644.9 | 643588096 | 3364 |
| 6 | width16-compact | 505.7 | 849.2 | 319.6 | 5.7 | 3264.3 | 1.2 | 0.4 | 0.0 | 975 | 2592.6 | 365518848 | 3560 |
| 6 | width16-spread | 362.3 | 805.0 | 323.7 | 8.0 | 3318.7 | 18.7 | 0.4 | 0.0 | 862 | 2613.5 | 422305792 | 3828 |
| 2-post | width1-uniform | 264.4 | 117.3 | 26.9 | 6.2 | 4727.2 | 13.6 | 0.8 | 0.0 | 1517 | 4228.5 | 796299264 | 7560 |
| 2-post | width16-uniform | 1313.1 | 578.4 | 151.4 | 10.6 | 3920.0 | 54.3 | 0.4 | 0.0 | 537 | 1886.3 | 615395328 | 4400 |

## Fio WAL-like sequential write and fdatasync

| OCPU | IOPS | MB/s | fdatasync samples | mean ms | p50 ms | p95 ms | p99 ms |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 2 | 394.1 | 1.614 | 5912 | 2.513 | 2.540 | 2.966 | 3.129 |
| 4 | 433.1 | 1.774 | 6497 | 2.280 | 2.245 | 2.671 | 3.228 |
| 6 | 430.9 | 1.765 | 6465 | 2.296 | 2.245 | 2.769 | 3.097 |
| 2-post | 367.6 | 1.506 | 5514 | 2.695 | 2.638 | 3.228 | 3.686 |
