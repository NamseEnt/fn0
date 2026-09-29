## E1 step (Phase D, E1)

| Run | phase-d tx/s (runs) | e1 tx/s (runs) | e1 / phase-d |
|---|---|---|---|
| 64w w16 uniform sync disabled | 2,723 (2,746, 2,684, 2,739) | 2,919 (2,916, 2,779, 3,062) | 1.072 |
| 64w w16 uniform sync real | 2,412 (2,253, 2,480, 2,505) | 2,604 (2,685, 2,517, 2,609) | 1.079 |
| 64w w1 uniform sync real | 26,018 (25,308, 26,148, 26,598) | 26,555 (26,132, 27,002, 26,531) | 1.021 |

GM e1 / phase-d over 3 rows: 1.057

Coordinator µs per transaction, 64w w16 uniform, sync disabled (stepe1):

| µs / tx | phase-d | e1 |
|---|---|---|
| coordinator | 328.3 | 302.1 |
| planner | 97.5 | 99.0 |
| leaf on coordinator | 68.7 | 70.5 |
| lanes busy | 138.5 | 140.5 |
| dispatch wait collect | 44.1 | 26.2 |
| catalog publication install dirty | 75.2 | 72.2 |
| wal cpu | 17.7 | 18.0 |
| wal sync | 0.0 | 0.0 |
| unattributed | 25.2 | 16.1 |
| lane: base image + chain check | 28.6 | 34.5 |
| lane: mutation | 24.8 | 25.5 |
| lane: page encode | 36.6 | 39.6 |
| lane: delta encode + verify + CRC | 39.4 | 31.3 |

## E2a step (Phase D, E1, E2a)

| Run | phase-d tx/s (runs) | e1 tx/s (runs) | e2a tx/s (runs) | e1 / phase-d | e2a / phase-d |
|---|---|---|---|---|---|
| 64w w16 uniform sync disabled | 2,687 (2,696, 2,647, 2,716) | 2,939 (2,891, 2,848, 3,079) | 3,133 (3,119, 2,927, 3,353) | 1.094 | 1.166 |
| 64w w16 uniform sync real | 2,468 (2,409, 2,476, 2,520) | 2,664 (2,634, 2,652, 2,706) | 2,821 (2,759, 2,900, 2,803) | 1.079 | 1.143 |
| 64w w1 uniform sync real | 25,565 (25,417, 25,243, 26,036) | 25,553 (26,614, 24,323, 25,722) | 27,532 (27,099, 28,022, 27,474) | 1.000 | 1.077 |

GM e1 / phase-d over 3 rows: 1.057
GM e2a / phase-d over 3 rows: 1.128

Coordinator µs per transaction, 64w w16 uniform, sync disabled (stepe2a):

| µs / tx | phase-d | e1 | e2a |
|---|---|---|---|
| coordinator | 332.0 | 299.3 | 279.2 |
| planner | 100.5 | 99.2 | 100.4 |
| leaf on coordinator | 71.4 | 69.2 | 72.4 |
| lanes busy | 141.6 | 138.4 | 144.7 |
| dispatch wait collect | 41.1 | 24.8 | 24.1 |
| catalog publication install dirty | 75.3 | 72.1 | 48.6 |
| wal cpu | 17.7 | 17.5 | 17.9 |
| wal sync | 0.0 | 0.0 | 0.0 |
| unattributed | 26.0 | 16.5 | 15.8 |
| lane: base image + chain check | 29.2 | 32.5 | 37.2 |
| lane: mutation | 26.0 | 25.1 | 25.2 |
| lane: page encode | 36.0 | 39.4 | 39.8 |
| lane: delta encode + verify + CRC | 39.8 | 31.8 | 31.5 |

## E2b step (Phase D, E2a, E2b)

| Run | phase-d tx/s (runs) | e2a tx/s (runs) | e2b tx/s (runs) | e2a / phase-d | e2b / phase-d |
|---|---|---|---|---|---|
| 64w w16 uniform sync disabled | 2,692 (2,726, 2,606, 2,743) | 3,229 (3,242, 3,129, 3,314) | 3,151 (3,250, 3,059, 3,144) | 1.200 | 1.171 |
| 64w w16 uniform sync real | 2,412 (2,423, 2,412, 2,401) | 2,825 (2,880, 2,815, 2,781) | 2,907 (2,897, 2,917, 2,908) | 1.171 | 1.205 |
| 64w w1 uniform sync real | 25,144 (26,092, 25,039, 24,300) | 27,317 (27,731, 26,834, 27,385) | 27,559 (27,890, 27,645, 27,143) | 1.086 | 1.096 |

GM e2a / phase-d over 3 rows: 1.151
GM e2b / phase-d over 3 rows: 1.156

Coordinator µs per transaction, 64w w16 uniform, sync disabled (stepe2b):

| µs / tx | phase-d | e2a | e2b |
|---|---|---|---|
| coordinator | 331.2 | 271.0 | 278.9 |
| planner | 101.6 | 96.9 | 96.4 |
| leaf on coordinator | 69.6 | 68.8 | 78.8 |
| lanes busy | 136.1 | 137.4 | 156.5 |
| dispatch wait collect | 39.8 | 24.8 | 26.8 |
| catalog publication install dirty | 75.8 | 46.9 | 43.5 |
| wal cpu | 17.8 | 17.9 | 18.1 |
| wal sync | 0.0 | 0.0 | 0.0 |
| unattributed | 26.5 | 15.6 | 15.3 |
| lane: base image + chain check | 28.1 | 33.0 | 47.7 |
| lane: mutation | 25.1 | 24.6 | 16.3 |
| lane: page encode | 35.4 | 38.0 | 47.4 |
| lane: delta encode + verify + CRC | 38.3 | 31.5 | 31.9 |

## E3 step (Phase D, E2b, E3 = E1+E2a+E2b+E3)

| Run | phase-d tx/s (runs) | e2b tx/s (runs) | e3 tx/s (runs) | e2b / phase-d | e3 / phase-d |
|---|---|---|---|---|---|
| 64w w16 uniform sync disabled | 2,771 (2,744, 2,830, 2,739) | 3,294 (3,307, 3,243, 3,331) | 3,601 (3,650, 3,572, 3,580) | 1.189 | 1.299 |
| 64w w16 uniform sync real | 2,358 (2,408, 2,231, 2,433) | 2,885 (2,870, 2,896, 2,891) | 3,508 (3,427, 3,506, 3,590) | 1.224 | 1.488 |
| 64w w1 uniform sync real | 25,279 (25,038, 25,710, 25,088) | 27,645 (28,269, 27,454, 27,213) | 27,712 (27,498, 27,286, 28,353) | 1.094 | 1.096 |

GM e2b / phase-d over 3 rows: 1.167
GM e3 / phase-d over 3 rows: 1.284

Coordinator µs per transaction, 64w w16 uniform, sync disabled (stepe3):

| µs / tx | phase-d | e2b | e3 |
|---|---|---|---|
| coordinator | 321.6 | 267.0 | 238.7 |
| planner | 97.5 | 95.3 | 50.1 |
| leaf on coordinator | 68.4 | 73.6 | 81.0 |
| lanes busy | 136.6 | 148.8 | 162.4 |
| dispatch wait collect | 40.8 | 24.2 | 29.0 |
| catalog publication install dirty | 72.8 | 41.4 | 48.0 |
| wal cpu | 17.0 | 17.2 | 18.3 |
| wal sync | 0.0 | 0.0 | 0.0 |
| unattributed | 25.1 | 15.3 | 12.3 |
| lane: base image + chain check | 28.9 | 44.3 | 50.5 |
| lane: mutation | 24.1 | 15.4 | 14.4 |
| lane: page encode | 35.6 | 46.5 | 51.5 |
| lane: delta encode + verify + CRC | 38.1 | 30.6 | 34.0 |

## Final step (Phase D, E3, final = E3 with E2b reverted)

| Run | phase-d tx/s (runs) | e3 tx/s (runs) | final tx/s (runs) | e3 / phase-d | final / phase-d |
|---|---|---|---|---|---|
| 64w w16 uniform sync disabled | 2,755 (2,812, 2,648, 2,803) | 3,700 (3,733, 3,716, 3,649) | 3,755 (3,748, 3,837, 3,681) | 1.343 | 1.363 |
| 64w w16 uniform sync real | 2,435 (2,416, 2,490, 2,397) | 3,543 (3,605, 3,505, 3,520) | 3,464 (3,330, 3,532, 3,532) | 1.455 | 1.423 |
| 64w w1 uniform sync real | 25,985 (26,270, 26,364, 25,321) | 28,697 (28,831, 28,575, 28,684) | 27,609 (28,577, 27,234, 27,016) | 1.104 | 1.063 |

GM e3 / phase-d over 3 rows: 1.292
GM final / phase-d over 3 rows: 1.273

Coordinator µs per transaction, 64w w16 uniform, sync disabled (stepfinal):

| µs / tx | phase-d | e3 | final |
|---|---|---|---|
| coordinator | 323.6 | 233.4 | 228.7 |
| planner | 98.8 | 49.4 | 49.9 |
| leaf on coordinator | 68.0 | 79.5 | 71.5 |
| lanes busy | 133.1 | 160.5 | 147.2 |
| dispatch wait collect | 38.1 | 28.2 | 30.3 |
| catalog publication install dirty | 75.3 | 46.5 | 47.9 |
| wal cpu | 17.7 | 17.9 | 17.4 |
| wal sync | 0.0 | 0.0 | 0.0 |
| unattributed | 25.7 | 12.0 | 11.7 |
| lane: base image + chain check | 28.2 | 49.2 | 34.7 |
| lane: mutation | 23.9 | 14.8 | 24.7 |
| lane: page encode | 35.2 | 51.3 | 41.8 |
| lane: delta encode + verify + CRC | 36.7 | 33.7 | 33.0 |

Coordinator µs per transaction, 64w w16 uniform, sync real (stepfinal):

| µs / tx | phase-d | e3 | final |
|---|---|---|---|
| coordinator | 372.5 | 249.7 | 255.0 |
| planner | 97.3 | 45.6 | 46.5 |
| leaf on coordinator | 66.0 | 72.0 | 65.9 |
| lanes busy | 129.8 | 141.2 | 130.7 |
| dispatch wait collect | 35.6 | 16.2 | 18.2 |
| catalog publication install dirty | 75.4 | 42.9 | 47.4 |
| wal cpu | 16.9 | 16.4 | 16.7 |
| wal sync | 54.5 | 44.6 | 47.4 |
| unattributed | 26.8 | 11.9 | 12.9 |
| lane: base image + chain check | 27.6 | 43.6 | 31.3 |
| lane: mutation | 23.4 | 12.6 | 21.6 |
| lane: page encode | 33.9 | 44.9 | 37.5 |
| lane: delta encode + verify + CRC | 36.5 | 30.3 | 29.3 |

Coordinator µs per transaction, 64w w1 uniform, sync real (stepfinal):

| µs / tx | phase-d | e3 | final |
|---|---|---|---|
| coordinator | 35.9 | 32.5 | 33.6 |
| planner | 3.7 | 3.1 | 3.4 |
| leaf on coordinator | 4.4 | 4.7 | 4.4 |
| lanes busy | 8.4 | 8.9 | 8.3 |
| dispatch wait collect | 2.7 | 1.5 | 1.9 |
| catalog publication install dirty | 5.4 | 3.9 | 4.2 |
| wal cpu | 2.1 | 2.0 | 2.1 |
| wal sync | 15.8 | 16.0 | 16.4 |
| unattributed | 1.8 | 1.3 | 1.3 |
| lane: base image + chain check | 1.8 | 3.0 | 2.1 |
| lane: mutation | 1.4 | 0.8 | 1.4 |
| lane: page encode | 2.2 | 2.8 | 2.5 |
| lane: delta encode + verify + CRC | 2.3 | 1.8 | 1.9 |

## CPU gate (sync disabled)

| Run | phase-d tx/s (runs) | final tx/s (runs) | final / phase-d |
|---|---|---|---|
| 64w w16 uniform sync disabled | 2,696 (2,689, 2,645, 2,754) | 3,821 (3,726, 3,734, 4,003) | 1.417 |
| 64w w16 compact sync disabled | 15,836 (15,997, 15,546, 15,966) | 18,416 (18,163, 19,007, 18,079) | 1.163 |
| 64w w16 spread sync disabled | 8,474 (8,713, 8,817, 7,892) | 10,485 (10,621, 10,531, 10,302) | 1.237 |

GM final / phase-d over 3 rows: 1.268

Coordinator µs per transaction, 64w w16 uniform, sync disabled (cpu):

| µs / tx | phase-d | final |
|---|---|---|
| coordinator | 331.8 | 224.2 |
| planner | 99.6 | 48.4 |
| leaf on coordinator | 69.4 | 71.6 |
| lanes busy | 138.8 | 147.6 |
| dispatch wait collect | 42.4 | 28.8 |
| catalog publication install dirty | 76.3 | 46.3 |
| wal cpu | 18.0 | 17.7 |
| wal sync | 0.0 | 0.0 |
| unattributed | 26.1 | 11.4 |
| lane: base image + chain check | 29.5 | 35.4 |
| lane: mutation | 24.6 | 24.5 |
| lane: page encode | 36.2 | 41.8 |
| lane: delta encode + verify + CRC | 39.0 | 34.1 |

Coordinator µs per transaction, 64w w16 compact, sync disabled (cpu):

| µs / tx | phase-d | final |
|---|---|---|
| coordinator | 50.7 | 42.1 |
| planner | 28.1 | 22.0 |
| leaf on coordinator | 10.7 | 9.3 |
| lanes busy | 20.7 | 18.6 |
| dispatch wait collect | 4.0 | 3.9 |
| catalog publication install dirty | 0.6 | 0.7 |
| wal cpu | 2.8 | 2.8 |
| wal sync | 0.0 | 0.0 |
| unattributed | 4.5 | 3.3 |
| lane: base image + chain check | 0.2 | 0.2 |
| lane: mutation | 11.8 | 10.9 |
| lane: page encode | 3.0 | 2.4 |
| lane: delta encode + verify + CRC | 5.3 | 4.7 |

Coordinator µs per transaction, 64w w16 spread, sync disabled (cpu):

| µs / tx | phase-d | final |
|---|---|---|
| coordinator | 91.7 | 70.9 |
| planner | 41.6 | 29.7 |
| leaf on coordinator | 13.4 | 14.3 |
| lanes busy | 30.2 | 29.7 |
| dispatch wait collect | 11.9 | 6.7 |
| catalog publication install dirty | 10.7 | 9.2 |
| wal cpu | 4.6 | 4.3 |
| wal sync | 0.0 | 0.0 |
| unattributed | 9.6 | 6.8 |
| lane: base image + chain check | 3.1 | 4.0 |
| lane: mutation | 13.9 | 12.6 |
| lane: page encode | 3.5 | 4.2 |
| lane: delta encode + verify + CRC | 7.7 | 7.1 |

## Durable gate (real sync, ZFS)

| Run | phase-c tx/s (runs) | phase-d tx/s (runs) | final tx/s (runs) | phase-c / phase-d | final / phase-d |
|---|---|---|---|---|---|
| 16w w1 uniform sync real | 8,342 (8,300, 8,310, 8,416) | 7,976 (7,854, 7,981, 8,092) | 8,112 (8,027, 8,178, 8,131) | 1.046 | 1.017 |
| 16w w16 uniform sync real | 1,947 (1,891, 1,995, 1,955) | 2,293 (2,285, 2,347, 2,248) | 2,666 (2,713, 2,680, 2,604) | 0.849 | 1.163 |
| 64w w1 uniform sync real | 23,906 (24,321, 24,194, 23,204) | 26,272 (26,353, 25,980, 26,483) | 28,901 (28,981, 28,702, 29,019) | 0.910 | 1.100 |
| 64w w16 uniform sync real | 2,058 (2,042, 2,088, 2,045) | 2,423 (2,450, 2,431, 2,388) | 3,449 (3,456, 3,374, 3,518) | 0.849 | 1.423 |
| 64w w16 compact sync real | 10,762 (10,715, 10,801, 10,770) | 11,935 (12,003, 12,024, 11,778) | 13,232 (13,384, 13,324, 12,987) | 0.902 | 1.109 |
| 64w w16 spread sync real | 5,842 (6,161, 6,050, 5,315) | 6,427 (6,544, 6,621, 6,117) | 7,606 (7,481, 7,823, 7,514) | 0.909 | 1.183 |

GM phase-c / phase-d over 6 rows: 0.909
GM final / phase-d over 6 rows: 1.159

Coordinator µs per transaction, 16w w1 uniform, sync real (gate):

| µs / tx | phase-c | phase-d | final |
|---|---|---|---|
| coordinator | 116.9 | 122.4 | 120.0 |
| planner | 4.7 | 4.9 | 4.3 |
| leaf on coordinator | 6.1 | 5.7 | 6.1 |
| lanes busy | 0.0 | 9.3 | 10.0 |
| dispatch wait collect | 0.0 | 5.0 | 3.4 |
| catalog publication install dirty | 6.3 | 6.7 | 5.7 |
| wal cpu | 9.7 | 5.0 | 4.9 |
| wal sync | 87.4 | 92.4 | 93.3 |
| unattributed | 2.8 | 2.7 | 2.2 |
| lane: base image + chain check | 0.0 | 2.2 | 2.6 |
| lane: mutation | 0.0 | 1.4 | 1.5 |
| lane: page encode | 0.0 | 2.4 | 3.0 |
| lane: delta encode + verify + CRC | 0.0 | 2.5 | 2.0 |

Coordinator µs per transaction, 64w w16 uniform, sync real (gate):

| µs / tx | phase-c | phase-d | final |
|---|---|---|---|
| coordinator | 447.6 | 374.0 | 255.5 |
| planner | 94.4 | 97.3 | 45.8 |
| leaf on coordinator | 100.4 | 66.6 | 66.4 |
| lanes busy | 0.0 | 130.0 | 132.2 |
| dispatch wait collect | 0.0 | 35.4 | 19.3 |
| catalog publication install dirty | 75.1 | 77.5 | 45.9 |
| wal cpu | 97.7 | 16.9 | 16.5 |
| wal sync | 52.3 | 52.5 | 49.5 |
| unattributed | 27.7 | 27.7 | 12.1 |
| lane: base image + chain check | 0.0 | 27.7 | 31.4 |
| lane: mutation | 0.0 | 23.2 | 21.8 |
| lane: page encode | 0.0 | 33.8 | 38.2 |
| lane: delta encode + verify + CRC | 0.0 | 36.2 | 30.1 |

## Retention (Phase C, Phase E serial executor, Phase E 2 lanes)

| Run | phase-c tx/s (runs) | final-serial tx/s (runs) | final tx/s (runs) | final-serial / phase-c | final / phase-c |
|---|---|---|---|---|---|
| 16w w1 uniform sync real | 8,289 (9,384, 7,769, 7,714) | 7,879 (7,971, 7,794, 7,871) | 7,692 (7,792, 7,745, 7,538) | 0.950 | 0.928 |
| 16w w16 uniform sync real | 1,861 (1,879, 1,881, 1,823) | 2,111 (2,144, 2,070, 2,119) | 2,601 (2,576, 2,592, 2,634) | 1.134 | 1.397 |
| 64w w1 uniform sync real | 23,209 (22,858, 23,367, 23,401) | 23,625 (24,538, 23,213, 23,122) | 27,417 (27,552, 27,037, 27,662) | 1.018 | 1.181 |
| 64w w16 uniform sync real | 1,999 (1,954, 2,001, 2,042) | 2,587 (2,547, 2,555, 2,657) | 3,421 (3,377, 3,406, 3,479) | 1.294 | 1.711 |
| 64w w16 compact sync real | 10,608 (10,647, 10,555, 10,623) | 11,587 (11,628, 11,589, 11,546) | 12,955 (12,975, 12,832, 13,059) | 1.092 | 1.221 |
| 64w w16 spread sync real | 5,922 (5,973, 5,914, 5,878) | 6,742 (6,650, 6,831, 6,746) | 7,509 (7,668, 7,563, 7,295) | 1.139 | 1.268 |

GM final-serial / phase-c over 6 rows: 1.099
GM final / phase-c over 6 rows: 1.263

Coordinator µs per transaction, 16w w1 uniform, sync real (retention):

| µs / tx | phase-c | final-serial | final |
|---|---|---|---|
| coordinator | 115.9 | 123.4 | 126.7 |
| planner | 4.8 | 4.4 | 4.5 |
| leaf on coordinator | 6.2 | 6.5 | 6.1 |
| lanes busy | 0.0 | 0.0 | 10.1 |
| dispatch wait collect | 0.0 | 0.0 | 3.6 |
| catalog publication install dirty | 6.4 | 6.7 | 5.9 |
| wal cpu | 9.8 | 8.5 | 4.9 |
| wal sync | 85.9 | 94.8 | 99.4 |
| unattributed | 2.8 | 2.4 | 2.3 |
| lane: base image + chain check | 0.0 | 0.0 | 2.6 |
| lane: mutation | 0.0 | 0.0 | 1.5 |
| lane: page encode | 0.0 | 0.0 | 3.1 |
| lane: delta encode + verify + CRC | 0.0 | 0.0 | 2.0 |

Coordinator µs per transaction, 64w w16 uniform, sync real (retention):

| µs / tx | phase-c | final-serial | final |
|---|---|---|---|
| coordinator | 459.5 | 358.1 | 258.4 |
| planner | 95.0 | 50.3 | 46.0 |
| leaf on coordinator | 101.7 | 101.5 | 66.3 |
| lanes busy | 0.0 | 0.0 | 131.2 |
| dispatch wait collect | 0.0 | 0.0 | 17.8 |
| catalog publication install dirty | 79.8 | 63.1 | 46.4 |
| wal cpu | 99.1 | 72.1 | 16.4 |
| wal sync | 56.3 | 55.4 | 53.0 |
| unattributed | 27.6 | 15.7 | 12.5 |
| lane: base image + chain check | 0.0 | 0.0 | 30.9 |
| lane: mutation | 0.0 | 0.0 | 21.8 |
| lane: page encode | 0.0 | 0.0 | 38.1 |
| lane: delta encode + verify + CRC | 0.0 | 0.0 | 29.9 |

