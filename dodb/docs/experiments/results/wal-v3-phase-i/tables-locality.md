## disabled sync

| Scenario | Groups | Tx/group mean / p50 / p95 / max | Mutations/group mean / p50 / p95 / max | Boundaries/group mean | Leaves/group mean | Model A removable | Mutations/leaf mean / p50 / p95 / max | Tx/leaf mean / p50 / p95 / max | Leaves touched by 1 / 2 / 3+ tx | Page encodes/group | PageDelta/group |
|---|---:|---|---|---:|---:|---:|---|---|---:|---:|---:|
| 64w width16 uniform | 2263 | 42.70 / 45 / 63 / 64 | 683.26 / 720 / 1008 / 1024 | 682.54 | 646.43 | 5.29% | 1.06 / 1 / 2 / 5 | 1.06 / 1 / 2 / 4 | 1384194 (94.6%) / 75766 (5.2%) / 2922 (0.2%) | 682.54 | 682.45 |

64w width16 uniform disabled: requests/group (42.70349094122846, 45, 63, 64); failed transactions/group 0.000; unique keys/group 680.76; successful tx/s median 6369.

| 64w width1 uniform | 43748 | 23.47 / 33 / 52 / 64 | 23.47 / 33 / 52 / 64 | 23.47 | 23.41 | 0.30% | 1.00 / 1 / 1 / 3 | 1.00 / 1 / 1 / 3 | 1020895 (99.7%) / 3028 (0.3%) / 6 (0.0%) | 23.47 | 23.47 |

64w width1 uniform disabled: requests/group (23.474650269726617, 33, 52, 64); failed transactions/group 0.000; unique keys/group 23.47; successful tx/s median 66284.

| 64w width16 same-leaf-heavy | 9106 | 38.21 / 44 / 49 / 64 | 611.36 / 704 / 784 / 1024 | 76.42 | 4.59 | 93.99% | 133.17 / 136 / 238 / 700 | 16.65 / 16 / 31 / 60 | 2399 (5.7%) / 425 (1.0%) / 38979 (93.2%) | 76.42 | 76.42 |

64w width16 same-leaf-heavy disabled: requests/group (38.20986162969471, 44, 49, 64); failed transactions/group 0.000; unique keys/group 57.00; successful tx/s median 23141.

| 64w width16 different-leaf-heavy | 6736 | 40.77 / 43 / 50 / 64 | 652.30 / 688 / 800 / 1024 | 81.54 | 81.54 | 0.00% | 8.00 / 8 / 15 / 15 | 1.00 / 1 / 1 / 1 | 549236 (100.0%) / 0 (0.0%) / 0 (0.0%) | 81.54 | 81.54 |

64w width16 different-leaf-heavy disabled: requests/group (40.7687054631829, 43, 50, 64); failed transactions/group 0.000; unique keys/group 652.30; successful tx/s median 18372.

| 16w width16 uniform | 10312 | 8.08 / 8 / 16 / 16 | 129.21 / 128 / 256 / 256 | 129.07 | 127.67 | 1.09% | 1.01 / 1 / 1 / 4 | 1.01 / 1 / 1 / 4 | 1302178 (98.9%) / 14228 (1.1%) / 121 (0.0%) | 129.07 | 129.05 |

16w width16 uniform disabled: requests/group (8.075640031031808, 8, 16, 16); failed transactions/group 0.000; unique keys/group 129.12; successful tx/s median 5573.

## real sync

| Scenario | Groups | Tx/group mean / p50 / p95 / max | Mutations/group mean / p50 / p95 / max | Boundaries/group mean | Leaves/group mean | Model A removable | Mutations/leaf mean / p50 / p95 / max | Tx/leaf mean / p50 / p95 / max | Leaves touched by 1 / 2 / 3+ tx | Page encodes/group | PageDelta/group |
|---|---:|---|---|---:|---:|---:|---|---|---:|---:|---:|
| 64w width16 uniform | 1651 | 43.93 / 44 / 47 / 64 | 702.89 / 704 / 752 / 1024 | 702.16 | 666.42 | 5.09% | 1.05 / 1 / 2 / 5 | 1.05 / 1 / 2 / 5 | 1043295 (94.8%) / 54984 (5.0%) / 1981 (0.2%) | 702.16 | 702.05 |

64w width16 uniform real: requests/group (43.93034524530587, 44, 47, 64); failed transactions/group 0.000; unique keys/group 700.47; successful tx/s median 4775.

| 64w width1 uniform | 10885 | 34.73 / 35 / 44 / 64 | 34.73 / 35 / 44 / 64 | 34.73 | 34.63 | 0.28% | 1.00 / 1 / 1 / 3 | 1.00 / 1 / 1 / 3 | 375900 (99.7%) / 1072 (0.3%) / 2 (0.0%) | 34.73 | 34.73 |

64w width1 uniform real: requests/group (34.73128158015618, 35, 44, 64); failed transactions/group 0.000; unique keys/group 34.73; successful tx/s median 25311.

| 64w width16 same-leaf-heavy | 5342 | 44.30 / 44 / 48 / 64 | 708.74 / 704 / 768 / 1024 | 88.59 | 4.86 | 94.51% | 145.82 / 140 / 284 / 882 | 18.23 / 18 / 37 / 63 | 508 (2.0%) / 450 (1.7%) / 25007 (96.3%) | 88.59 | 88.59 |

64w width16 same-leaf-heavy real: requests/group (44.29651815799326, 44, 48, 64); failed transactions/group 0.000; unique keys/group 60.78; successful tx/s median 15758.

| 64w width16 different-leaf-heavy | 3420 | 43.99 / 44 / 48 / 64 | 703.84 / 704 / 768 / 1024 | 87.98 | 87.98 | 0.00% | 8.00 / 8 / 15 / 15 | 1.00 / 1 / 1 / 1 | 300892 (100.0%) / 0 (0.0%) / 0 (0.0%) | 87.98 | 87.98 |

64w width16 different-leaf-heavy real: requests/group (43.990058479532166, 44, 48, 64); failed transactions/group 0.000; unique keys/group 703.84; successful tx/s median 10025.

| 16w width16 uniform | 6166 | 8.22 / 8 / 11 / 16 | 131.57 / 128 / 176 / 256 | 131.43 | 130.20 | 0.93% | 1.01 / 1 / 1 / 3 | 1.01 / 1 / 1 / 3 | 795344 (99.1%) / 7459 (0.9%) / 41 (0.0%) | 131.43 | 131.41 |

16w width16 uniform real: requests/group (8.222997080765488, 8, 11, 16); failed transactions/group 0.000; unique keys/group 131.49; successful tx/s median 3379.

