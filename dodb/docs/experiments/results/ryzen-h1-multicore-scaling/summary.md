# Ryzen 5 5600 storage scaling benchmark

Three repetitions per configuration; six for 6C/12T width16 uniform and twelve for 1C width16 compact after high initial variance. Raw per-run metrics are in per-run.csv.

## Throughput (transactions/s)

| Engine | Workload | 1C | 2C | 4C | 6C | 6C/12T |
|---|---|---:|---:|---:|---:|---:|
| dodb | w1_uniform | 6,748 | 5,401 | 5,151 | 5,275 | 5,136 |
| dodb | w16_uniform | 4,400 | 3,542 | 3,730 | 3,771 | 3,848 |
| dodb | w16_compact | 15,563 | 4,690 | 4,704 | 4,913 | 4,867 |
| dodb | w16_spread | 5,975 | 4,405 | 4,440 | 4,541 | 4,331 |
| rocksdb | w1_uniform | 5,841 | 5,509 | 5,551 | 5,533 | 5,804 |
| rocksdb | w16_uniform | 4,781 | 4,646 | 4,930 | 4,845 | 4,876 |
| rocksdb | w16_compact | 9,419 | 4,764 | 4,257 | 4,893 | 5,047 |
| rocksdb | w16_spread | 4,888 | 4,822 | 4,789 | 4,783 | 4,884 |

## dodb/RocksDB ratio

| Workload | 1C | 2C | 4C | 6C | 6C/12T |
|---|---:|---:|---:|---:|---:|
| w1_uniform | 1.155x | 0.980x | 0.928x | 0.953x | 0.885x |
| w16_uniform | 0.920x | 0.762x | 0.757x | 0.778x | 0.789x |
| w16_compact | 1.652x | 0.985x | 1.105x | 1.004x | 0.964x |
| w16_spread | 1.222x | 0.914x | 0.927x | 0.949x | 0.887x |

## Speedup from 1C

| Engine | Workload | 1C | 2C | 4C | 6C | 6C/12T |
|---|---|---:|---:|---:|---:|---:|
| dodb | w1_uniform | 1.00x | 0.80x | 0.76x | 0.78x | 0.76x |
| dodb | w16_uniform | 1.00x | 0.81x | 0.85x | 0.86x | 0.87x |
| dodb | w16_compact | 1.00x | 0.30x | 0.30x | 0.32x | 0.31x |
| dodb | w16_spread | 1.00x | 0.74x | 0.74x | 0.76x | 0.72x |
| rocksdb | w1_uniform | 1.00x | 0.94x | 0.95x | 0.95x | 0.99x |
| rocksdb | w16_uniform | 1.00x | 0.97x | 1.03x | 1.01x | 1.02x |
| rocksdb | w16_compact | 1.00x | 0.51x | 0.45x | 0.52x | 0.54x |
| rocksdb | w16_spread | 1.00x | 0.99x | 0.98x | 0.98x | 1.00x |

## Width16 uniform latency medians (microseconds)

| Engine | Cores | p50 | p95 | p99 | CPU % of one allocated core |
|---|---|---:|---:|---:|---:|
| dodb | 1c | 13,315 | 21,144 | 24,812 | 47.6% |
| dodb | 2c | 17,423 | 20,426 | 33,306 | 40.2% |
| dodb | 4c | 16,604 | 19,500 | 27,587 | 45.3% |
| dodb | 6c | 16,293 | 19,716 | 37,693 | 47.7% |
| dodb | 6c12t | 16,305 | 18,871 | 26,071 | 67.7% |
| rocksdb | 1c | 13,848 | 16,306 | 43,953 | 22.5% |
| rocksdb | 2c | 13,249 | 16,294 | 46,622 | 25.5% |
| rocksdb | 4c | 12,679 | 15,034 | 38,682 | 27.9% |
| rocksdb | 6c | 12,585 | 15,735 | 44,666 | 28.9% |
| rocksdb | 6c12t | 12,305 | 14,639 | 44,722 | 34.9% |
