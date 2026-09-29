## F1 step (Phase E, F1)

| Run | phase-e tx/s (runs) | f1 tx/s (runs) | f1 / phase-e |
|---|---|---|---|
| 64w w16 uniform sync disabled | 3,755 (3,683, 3,810, 3,772) | 4,253 (4,449, 3,952, 4,359) | 1.133 |
| 64w w16 uniform sync real | 3,523 (3,513, 3,669, 3,386) | 3,989 (3,988, 3,950, 4,028) | 1.132 |
| 64w w1 uniform sync real | 28,397 (28,548, 28,771, 27,873) | 30,246 (30,533, 30,229, 29,978) | 1.065 |

GM f1 / phase-e over 3 rows: 1.110

Coordinator µs per transaction, 64w w16 uniform, sync disabled (stepf1):

| µs / tx | phase-e | f1 |
|---|---|---|
| coordinator | 227.9 | 204.9 |
| planner | 48.9 | 47.2 |
| leaf on coordinator | 73.0 | 64.0 |
| lanes busy | 148.2 | 127.8 |
| dispatch wait collect | 27.9 | 26.3 |
| catalog publication install dirty | 48.5 | 37.8 |
| wal cpu | 18.0 | 18.5 |
| wal sync | 0.0 | 0.0 |
| unattributed | 11.6 | 11.3 |
| lane: base image + chain check | 35.7 | 31.7 |
| lane: mutation | 24.4 | 15.3 |
| lane: page encode | 42.0 | 31.3 |
| lane: delta encode + verify + CRC | 33.3 | 34.3 |

Coordinator µs per transaction, 64w w16 uniform, sync real (stepf1):

| µs / tx | phase-e | f1 |
|---|---|---|
| coordinator | 252.4 | 223.6 |
| planner | 45.5 | 43.1 |
| leaf on coordinator | 65.7 | 54.2 |
| lanes busy | 130.3 | 106.2 |
| dispatch wait collect | 17.8 | 15.2 |
| catalog publication install dirty | 46.1 | 33.3 |
| wal cpu | 16.4 | 16.2 |
| wal sync | 48.5 | 50.3 |
| unattributed | 12.4 | 11.2 |
| lane: base image + chain check | 31.1 | 27.4 |
| lane: mutation | 21.6 | 13.1 |
| lane: page encode | 37.5 | 26.2 |
| lane: delta encode + verify + CRC | 29.8 | 29.6 |

## Adaptive parallel control (F code: serial, 2 lanes, 2 lanes with --parallel-min-mutations 32)

| Run | f-serial tx/s (runs) | f-2lane tx/s (runs) | f-adaptive tx/s (runs) | f-serial / f-2lane | f-adaptive / f-2lane |
|---|---|---|---|---|---|
| 16w w1 uniform sync real | 8,337 (8,163, 8,406, 8,441) | 8,241 (8,189, 8,233, 8,301) | 8,280 (8,165, 8,327, 8,349) | 1.012 | 1.005 |
| 64w w1 uniform sync real | 26,590 (26,600, 26,481, 26,690) | 29,164 (25,966, 30,010, 31,515) | 30,113 (30,162, 30,504, 29,674) | 0.912 | 1.033 |
| 16w w16 uniform sync real | 2,509 (2,469, 2,532, 2,526) | 3,026 (3,003, 3,071, 3,003) | 2,980 (2,961, 2,984, 2,996) | 0.829 | 0.985 |
| 64w w16 uniform sync real | 3,075 (3,113, 3,108, 3,003) | 3,872 (3,817, 3,796, 4,004) | 3,930 (3,896, 4,002, 3,891) | 0.794 | 1.015 |
| 64w w16 compact sync real | 11,811 (11,908, 11,707, 11,817) | 12,944 (12,224, 13,359, 13,249) | 13,174 (13,019, 13,253, 13,250) | 0.912 | 1.018 |

GM f-serial / f-2lane over 5 rows: 0.889
GM f-adaptive / f-2lane over 5 rows: 1.011

Coordinator µs per transaction, 16w w1 uniform, sync real (adaptive):

| µs / tx | f-serial | f-2lane | f-adaptive |
|---|---|---|---|
| coordinator | 116.9 | 118.4 | 117.7 |
| planner | 4.2 | 4.3 | 4.2 |
| leaf on coordinator | 5.0 | 5.2 | 4.8 |
| lanes busy | 0.0 | 8.2 | 0.0 |
| dispatch wait collect | 0.0 | 3.5 | 0.0 |
| catalog publication install dirty | 6.2 | 5.0 | 6.2 |
| wal cpu | 8.7 | 4.8 | 8.6 |
| wal sync | 90.5 | 93.5 | 91.5 |
| unattributed | 2.3 | 2.2 | 2.4 |
| lane: base image + chain check | 0.0 | 2.6 | 0.0 |
| lane: mutation | 0.0 | 0.9 | 0.0 |
| lane: page encode | 0.0 | 2.0 | 0.0 |
| lane: delta encode + verify + CRC | 0.0 | 2.0 | 0.0 |

## Durable gate (Phase C, Phase E, Phase F)

| Run | phase-c tx/s (runs) | phase-e tx/s (runs) | f-adaptive tx/s (runs) | phase-c / phase-e | f-adaptive / phase-e |
|---|---|---|---|---|---|
| 16w w1 uniform sync real | 8,240 (8,246, 8,252, 8,222) | 8,031 (7,868, 8,128, 8,097) | 8,454 (8,454, 8,331, 8,576) | 1.026 | 1.053 |
| 16w w16 uniform sync real | 1,928 (1,915, 1,926, 1,944) | 2,694 (2,732, 2,729, 2,619) | 3,016 (2,990, 3,003, 3,057) | 0.716 | 1.120 |
| 64w w1 uniform sync real | 24,441 (24,930, 24,696, 23,697) | 27,480 (26,014, 28,039, 28,386) | 30,501 (30,224, 31,126, 30,153) | 0.889 | 1.110 |
| 64w w16 uniform sync real | 2,067 (2,052, 2,076, 2,072) | 3,591 (3,531, 3,688, 3,554) | 4,031 (4,040, 3,949, 4,103) | 0.575 | 1.122 |
| 64w w16 compact sync real | 10,812 (10,866, 10,773, 10,797) | 13,158 (13,046, 13,225, 13,203) | 13,298 (13,391, 13,475, 13,028) | 0.822 | 1.011 |
| 64w w16 spread sync real | 6,084 (6,138, 5,983, 6,131) | 7,587 (7,585, 7,676, 7,499) | 8,268 (7,938, 8,434, 8,434) | 0.802 | 1.090 |

GM phase-c / phase-e over 6 rows: 0.793
GM f-adaptive / phase-e over 6 rows: 1.083

Coordinator µs per transaction, 16w w1 uniform, sync real (gate):

| µs / tx | phase-c | phase-e | f-adaptive |
|---|---|---|---|
| coordinator | 118.3 | 121.4 | 115.2 |
| planner | 4.7 | 4.3 | 4.3 |
| leaf on coordinator | 6.1 | 5.8 | 4.8 |
| lanes busy | 0.0 | 9.7 | 0.0 |
| dispatch wait collect | 0.0 | 3.6 | 0.0 |
| catalog publication install dirty | 6.1 | 5.6 | 6.5 |
| wal cpu | 9.9 | 4.8 | 8.5 |
| wal sync | 88.9 | 94.9 | 88.7 |
| unattributed | 2.7 | 2.3 | 2.5 |
| lane: base image + chain check | 0.0 | 2.5 | 0.0 |
| lane: mutation | 0.0 | 1.5 | 0.0 |
| lane: page encode | 0.0 | 2.9 | 0.0 |
| lane: delta encode + verify + CRC | 0.0 | 2.0 | 0.0 |

Coordinator µs per transaction, 64w w16 uniform, sync real (gate):

| µs / tx | phase-c | phase-e | f-adaptive |
|---|---|---|---|
| coordinator | 443.5 | 245.8 | 221.1 |
| planner | 93.3 | 45.0 | 43.2 |
| leaf on coordinator | 99.2 | 65.4 | 54.9 |
| lanes busy | 0.0 | 129.1 | 106.7 |
| dispatch wait collect | 0.0 | 16.9 | 14.9 |
| catalog publication install dirty | 75.1 | 45.2 | 33.4 |
| wal cpu | 97.3 | 16.4 | 16.1 |
| wal sync | 51.4 | 45.2 | 47.4 |
| unattributed | 27.2 | 11.9 | 11.2 |
| lane: base image + chain check | 0.0 | 30.7 | 27.3 |
| lane: mutation | 0.0 | 21.3 | 13.4 |
| lane: page encode | 0.0 | 37.3 | 26.3 |
| lane: delta encode + verify + CRC | 0.0 | 29.6 | 29.6 |

## Full matrix (Phase C, Phase E, Phase F)

| Run | phase-c tx/s (runs) | phase-e tx/s (runs) | f-adaptive tx/s (runs) | phase-c / phase-e | f-adaptive / phase-e |
|---|---|---|---|---|---|
| 16w w1 uniform sync real | 8,089 (8,110, 8,099, 8,057) | 8,025 (7,904, 8,183, 7,988) | 8,503 (8,461, 8,340, 8,710) | 1.008 | 1.060 |
| 16w w1 compact sync real | 9,395 (9,258, 9,334, 9,592) | 9,500 (9,811, 9,095, 9,596) | 9,325 (9,170, 9,342, 9,465) | 0.989 | 0.982 |
| 16w w1 spread sync real | 8,541 (8,528, 8,795, 8,301) | 8,440 (8,536, 8,415, 8,367) | 8,624 (8,653, 8,521, 8,698) | 1.012 | 1.022 |
| 16w w16 uniform sync real | 1,902 (1,956, 1,898, 1,853) | 2,674 (2,683, 2,706, 2,634) | 2,867 (2,968, 2,708, 2,926) | 0.711 | 1.072 |
| 16w w16 compact sync real | 5,975 (6,057, 5,963, 5,907) | 6,588 (6,604, 6,612, 6,549) | 6,592 (6,653, 6,557, 6,566) | 0.907 | 1.001 |
| 16w w16 spread sync real | 5,551 (5,718, 5,499, 5,437) | 6,183 (6,261, 6,094, 6,195) | 6,337 (6,257, 6,501, 6,252) | 0.898 | 1.025 |
| 64w w1 uniform sync real | 23,953 (23,977, 24,079, 23,802) | 28,087 (27,986, 27,530, 28,745) | 29,750 (29,995, 30,350, 28,906) | 0.853 | 1.059 |
| 64w w1 compact sync real | 32,241 (31,728, 31,913, 33,081) | 33,521 (33,034, 33,866, 33,662) | 33,176 (33,644, 34,099, 31,786) | 0.962 | 0.990 |
| 64w w1 spread sync real | 26,521 (26,178, 26,193, 27,191) | 30,882 (31,364, 30,505, 30,775) | 30,809 (30,485, 31,315, 30,627) | 0.859 | 0.998 |
| 64w w16 uniform sync real | 2,045 (2,055, 2,028, 2,053) | 3,512 (3,621, 3,425, 3,489) | 3,855 (4,036, 3,585, 3,944) | 0.582 | 1.098 |
| 64w w16 compact sync real | 10,800 (10,762, 10,893, 10,746) | 13,132 (13,419, 12,958, 13,019) | 13,351 (13,406, 13,265, 13,383) | 0.822 | 1.017 |
| 64w w16 spread sync real | 6,030 (6,165, 6,001, 5,923) | 7,675 (7,992, 7,715, 7,318) | 8,220 (8,291, 8,210, 8,159) | 0.786 | 1.071 |
| 1w w1 uniform sync real | 1,345 (1,310, 1,427, 1,298) | 1,349 (1,334, 1,373, 1,341) | 1,322 (1,378, 1,339, 1,250) | 0.997 | 0.980 |
| 1w w16 uniform sync real | 806 (791, 821, 806) | 836 (840, 855, 814) | 871 (861, 865, 888) | 0.964 | 1.042 |

GM phase-c / phase-e over 14 rows: 0.873
GM f-adaptive / phase-e over 14 rows: 1.029

## Same-session RocksDB confirmation

Three interleaved real-sync repetitions on the same OCI ZFS host. Ratios are geometric means of paired repetitions; the displayed throughput values are medians of each set of three runs.

| Scenario | dodb F adaptive tx/s (runs) | RocksDB tx/s (runs) | dodb / RocksDB |
|---|---|---|---|
| 64w w1 uniform | 29,772 (29,771, 29,827, 29,967) | 26,320 (26,143, 26,320, 26,800) | **1.130** |
| 64w w16 uniform | 3,820 (3,820, 3,970, 3,781) | 9,814 (9,618, 9,814, 9,931) | **0.394** |
