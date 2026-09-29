# WAL v3 Phase C attribution tables

All rows: planned Blink with PageDelta (`cb7fb54`), 100,000 rows, 2 s warmup, 5 s measure, 3 repetitions. `disabled` rows skip fsync and are a CPU control, not durable throughput.

## Throughput

| Scenario | sync | tx/s (runs) | mean tx/s | mutation/s | p50 µs | p99 µs | CPU % (one core) | coordinator busy | tx/group | tx/sync | mean sync ms |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 16w w1 uniform | real | 8,179, 7,939, 8,350 | 8,156 | 8,156 | 1,920 | 3,249 | 32 | 97.6% | 8.1 | 8.1 | 0.73 |
| 16w w1 uniform | disabled | 32,881, 33,680, 33,677 | 33,413 | 33,413 | 413 | 1,338 | 99 | 89.6% | 9.3 | 9.3 | 0.00 |
| 16w w16 uniform | real | 1,891, 1,918, 1,931 | 1,913 | 30,612 | 7,784 | 16,251 | 78 | 92.9% | 14.5 | 14.5 | 1.94 |
| 16w w16 uniform | disabled | 2,586, 2,625, 2,605 | 2,606 | 41,690 | 5,606 | 12,536 | 100 | 90.8% | 12.1 | 12.1 | 0.00 |
| 64w w1 uniform | real | 24,480, 22,339, 23,817 | 23,545 | 23,545 | 2,548 | 5,564 | 69 | 93.5% | 60.6 | 60.6 | 0.92 |
| 64w w1 uniform | disabled | 36,534, 37,471, 38,044 | 37,350 | 37,350 | 1,578 | 3,883 | 102 | 89.4% | 30.9 | 30.9 | 0.00 |
| 64w w16 uniform | real | 2,055, 2,067, 2,068 | 2,063 | 33,014 | 30,153 | 58,977 | 92 | 92.3% | 61.5 | 61.5 | 3.15 |
| 64w w16 uniform | disabled | 2,251, 2,268, 2,291 | 2,270 | 36,323 | 27,113 | 57,212 | 101 | 90.8% | 52.6 | 52.6 | 0.00 |
| 64w w16 compact | real | 10,836, 10,792, 10,738 | 10,788 | 172,615 | 5,496 | 9,306 | 87 | 87.0% | 43.0 | 43.0 | 0.98 |
| 64w w16 compact | disabled | 14,093, 14,403, 14,238 | 14,245 | 227,916 | 3,917 | 6,858 | 114 | 82.3% | 42.3 | 42.3 | 0.00 |
| 64w w16 spread | real | 5,886, 6,096, 5,598 | 5,860 | 93,757 | 10,469 | 20,250 | 85 | 84.1% | 61.3 | 61.3 | 2.40 |
| 64w w16 spread | disabled | 7,831, 7,922, 7,959 | 7,904 | 126,461 | 7,797 | 14,852 | 106 | 79.6% | 46.2 | 46.2 | 0.00 |

## Coordinator time per tx, ns (real sync)

| Component | 16w w1 uniform | 16w w16 uniform | 64w w1 uniform | 64w w16 uniform | 64w w16 compact | 64w w16 spread |
|---|---|---|---|---|---|---|
| admission | 788 | 10,099 | 750 | 11,788 | 7,032 | 10,509 |
| planning | 4,012 | 54,797 | 3,259 | 82,974 | 19,941 | 31,563 |
| physical mutation | 3,279 | 55,782 | 3,388 | 57,976 | 12,081 | 17,211 |
| physical restamp | 109 | 2,604 | 150 | 2,963 | 919 | 1,272 |
| page encode | 2,351 | 36,036 | 2,281 | 38,067 | 2,894 | 3,582 |
| physical other | 395 | 1,022 | 326 | 1,345 | 333 | 859 |
| dirty union | 138 | 924 | 89 | 837 | 98 | 152 |
| catalog construction | 2,137 | 23,672 | 1,967 | 21,951 | 162 | 3,481 |
| WAL assembly | 1,155 | 20,974 | 1,331 | 27,629 | 1,157 | 2,859 |
| WAL redo plan (delta encode) | 4,101 | 55,909 | 3,773 | 56,341 | 5,798 | 8,965 |
| WAL frame encode | 927 | 9,662 | 890 | 9,972 | 1,376 | 1,763 |
| WAL write | 3,675 | 6,642 | 837 | 3,538 | 1,321 | 2,281 |
| WAL sync | 89,986 | 133,980 | 15,136 | 51,150 | 22,810 | 39,166 |
| state install | 1,112 | 15,743 | 1,218 | 17,718 | 98 | 2,076 |
| generation publication | 1,964 | 21,654 | 1,536 | 18,569 | 264 | 4,142 |
| dirty tracking | 837 | 14,918 | 839 | 16,535 | 83 | 1,789 |
| unattributed | 2,708 | 21,091 | 1,955 | 27,878 | 4,294 | 11,915 |
| **total (coordinator processing)** | 119,675 | 485,509 | 39,727 | 447,232 | 80,661 | 143,585 |

## Coordinator time per mutation, ns (real sync)

| Component | 16w w1 uniform | 16w w16 uniform | 64w w1 uniform | 64w w16 uniform | 64w w16 compact | 64w w16 spread |
|---|---|---|---|---|---|---|
| admission | 788 | 631 | 750 | 737 | 440 | 657 |
| planning | 4,012 | 3,425 | 3,259 | 5,186 | 1,246 | 1,973 |
| physical mutation | 3,279 | 3,486 | 3,388 | 3,623 | 755 | 1,076 |
| physical restamp | 109 | 163 | 150 | 185 | 57 | 79 |
| page encode | 2,351 | 2,252 | 2,281 | 2,379 | 181 | 224 |
| physical other | 395 | 64 | 326 | 84 | 21 | 54 |
| dirty union | 138 | 58 | 89 | 52 | 6 | 10 |
| catalog construction | 2,137 | 1,479 | 1,967 | 1,372 | 10 | 218 |
| WAL assembly | 1,155 | 1,311 | 1,331 | 1,727 | 72 | 179 |
| WAL redo plan (delta encode) | 4,101 | 3,494 | 3,773 | 3,521 | 362 | 560 |
| WAL frame encode | 927 | 604 | 890 | 623 | 86 | 110 |
| WAL write | 3,675 | 415 | 837 | 221 | 83 | 143 |
| WAL sync | 89,986 | 8,374 | 15,136 | 3,197 | 1,426 | 2,448 |
| state install | 1,112 | 984 | 1,218 | 1,107 | 6 | 130 |
| generation publication | 1,964 | 1,353 | 1,536 | 1,161 | 16 | 259 |
| dirty tracking | 837 | 932 | 839 | 1,033 | 5 | 112 |
| unattributed | 2,708 | 1,318 | 1,955 | 1,742 | 268 | 745 |
| **total (coordinator processing)** | 119,675 | 30,344 | 39,727 | 27,952 | 5,041 | 8,974 |

## Share of coordinator processing time (real sync)

| Component | 16w w1 uniform | 16w w16 uniform | 64w w1 uniform | 64w w16 uniform | 64w w16 compact | 64w w16 spread |
|---|---|---|---|---|---|---|
| admission | 0.7% | 2.1% | 1.9% | 2.6% | 8.7% | 7.3% |
| planning | 3.4% | 11.3% | 8.2% | 18.6% | 24.7% | 22.0% |
| physical mutation | 2.7% | 11.5% | 8.5% | 13.0% | 15.0% | 12.0% |
| physical restamp | 0.1% | 0.5% | 0.4% | 0.7% | 1.1% | 0.9% |
| page encode | 2.0% | 7.4% | 5.7% | 8.5% | 3.6% | 2.5% |
| physical other | 0.3% | 0.2% | 0.8% | 0.3% | 0.4% | 0.6% |
| dirty union | 0.1% | 0.2% | 0.2% | 0.2% | 0.1% | 0.1% |
| catalog construction | 1.8% | 4.9% | 5.0% | 4.9% | 0.2% | 2.4% |
| WAL assembly | 1.0% | 4.3% | 3.4% | 6.2% | 1.4% | 2.0% |
| WAL redo plan (delta encode) | 3.4% | 11.5% | 9.5% | 12.6% | 7.2% | 6.2% |
| WAL frame encode | 0.8% | 2.0% | 2.2% | 2.2% | 1.7% | 1.2% |
| WAL write | 3.1% | 1.4% | 2.1% | 0.8% | 1.6% | 1.6% |
| WAL sync | 75.2% | 27.6% | 38.1% | 11.4% | 28.3% | 27.3% |
| state install | 0.9% | 3.2% | 3.1% | 4.0% | 0.1% | 1.4% |
| generation publication | 1.6% | 4.5% | 3.9% | 4.2% | 0.3% | 2.9% |
| dirty tracking | 0.7% | 3.1% | 2.1% | 3.7% | 0.1% | 1.2% |
| unattributed | 2.3% | 4.3% | 4.9% | 6.2% | 5.3% | 8.3% |

## Detail timers per tx, ns (real sync)

| Timer | 16w w1 uniform | 16w w16 uniform | 64w w1 uniform | 64w w16 uniform | 64w w16 compact | 64w w16 spread |
|---|---|---|---|---|---|---|
| planner route | 2,324 | 25,333 | 1,779 | 22,731 | 7,345 | 10,661 |
| leaf load clone (inside physical mutation) | 1,424 | 20,940 | 1,331 | 20,540 | 64 | 2,191 |
| catalog state scan (inside catalog) | 1,931 | 22,794 | 1,900 | 21,142 | 90 | 3,335 |
| catalog chunk clone (inside catalog) | 1,040 | 5,905 | 799 | 2,551 | 20 | 1,344 |
| catalog map clone (inside catalog) | 1,164 | 6,028 | 825 | 2,595 | 73 | 1,395 |
| catalog directory clone (inside catalog) | 125 | 123 | 26 | 44 | 54 | 51 |
| retired generation drop (inside publication) | 1,907 | 21,605 | 1,526 | 18,557 | 252 | 4,129 |
| publication swap (inside publication) | 4 | 3 | 1 | 1 | 1 | 0 |
| WAL append total (plan + encode + write) | 8,801 | 72,306 | 5,517 | 69,883 | 8,525 | 13,033 |

## Coordinator time per tx, ns (disabled sync)

| Component | 16w w1 uniform | 16w w16 uniform | 64w w1 uniform | 64w w16 uniform | 64w w16 compact | 64w w16 spread |
|---|---|---|---|---|---|---|
| admission | 726 | 10,065 | 678 | 11,903 | 7,049 | 10,518 |
| planning | 3,478 | 54,468 | 3,152 | 84,567 | 20,149 | 31,299 |
| physical mutation | 3,420 | 57,310 | 3,446 | 58,613 | 12,090 | 16,962 |
| physical restamp | 157 | 2,556 | 155 | 3,000 | 900 | 1,224 |
| page encode | 2,322 | 36,737 | 2,319 | 38,791 | 2,912 | 3,412 |
| physical other | 359 | 1,036 | 339 | 1,340 | 326 | 853 |
| dirty union | 140 | 956 | 92 | 835 | 96 | 150 |
| catalog construction | 1,963 | 24,382 | 1,698 | 22,483 | 164 | 2,880 |
| WAL assembly | 1,136 | 20,321 | 1,376 | 28,202 | 1,159 | 2,707 |
| WAL redo plan (delta encode) | 4,138 | 57,148 | 3,817 | 57,641 | 5,952 | 8,835 |
| WAL frame encode | 953 | 9,855 | 888 | 10,178 | 1,395 | 1,756 |
| WAL write | 2,891 | 6,397 | 1,025 | 3,723 | 1,282 | 2,176 |
| WAL sync | 8 | 13 | 2 | 5 | 3 | 3 |
| state install | 864 | 13,671 | 906 | 17,592 | 64 | 1,558 |
| generation publication | 1,723 | 21,145 | 1,536 | 18,777 | 218 | 3,973 |
| dirty tracking | 641 | 13,254 | 762 | 15,880 | 62 | 1,433 |
| unattributed | 1,882 | 18,992 | 1,737 | 26,390 | 3,963 | 10,981 |
| **total (coordinator processing)** | 26,800 | 348,307 | 23,928 | 399,921 | 57,782 | 100,718 |

## Coordinator time per mutation, ns (disabled sync)

| Component | 16w w1 uniform | 16w w16 uniform | 64w w1 uniform | 64w w16 uniform | 64w w16 compact | 64w w16 spread |
|---|---|---|---|---|---|---|
| admission | 726 | 629 | 678 | 744 | 441 | 657 |
| planning | 3,478 | 3,404 | 3,152 | 5,285 | 1,259 | 1,956 |
| physical mutation | 3,420 | 3,582 | 3,446 | 3,663 | 756 | 1,060 |
| physical restamp | 157 | 160 | 155 | 187 | 56 | 76 |
| page encode | 2,322 | 2,296 | 2,319 | 2,424 | 182 | 213 |
| physical other | 359 | 65 | 339 | 84 | 20 | 53 |
| dirty union | 140 | 60 | 92 | 52 | 6 | 9 |
| catalog construction | 1,963 | 1,524 | 1,698 | 1,405 | 10 | 180 |
| WAL assembly | 1,136 | 1,270 | 1,376 | 1,763 | 72 | 169 |
| WAL redo plan (delta encode) | 4,138 | 3,572 | 3,817 | 3,603 | 372 | 552 |
| WAL frame encode | 953 | 616 | 888 | 636 | 87 | 110 |
| WAL write | 2,891 | 400 | 1,025 | 233 | 80 | 136 |
| WAL sync | 8 | 1 | 2 | 0 | 0 | 0 |
| state install | 864 | 854 | 906 | 1,100 | 4 | 97 |
| generation publication | 1,723 | 1,322 | 1,536 | 1,174 | 14 | 248 |
| dirty tracking | 641 | 828 | 762 | 993 | 4 | 90 |
| unattributed | 1,882 | 1,187 | 1,737 | 1,649 | 248 | 686 |
| **total (coordinator processing)** | 26,800 | 21,769 | 23,928 | 24,995 | 3,611 | 6,295 |

## Share of coordinator processing time (disabled sync)

| Component | 16w w1 uniform | 16w w16 uniform | 64w w1 uniform | 64w w16 uniform | 64w w16 compact | 64w w16 spread |
|---|---|---|---|---|---|---|
| admission | 2.7% | 2.9% | 2.8% | 3.0% | 12.2% | 10.4% |
| planning | 13.0% | 15.6% | 13.2% | 21.1% | 34.9% | 31.1% |
| physical mutation | 12.8% | 16.5% | 14.4% | 14.7% | 20.9% | 16.8% |
| physical restamp | 0.6% | 0.7% | 0.6% | 0.8% | 1.6% | 1.2% |
| page encode | 8.7% | 10.5% | 9.7% | 9.7% | 5.0% | 3.4% |
| physical other | 1.3% | 0.3% | 1.4% | 0.3% | 0.6% | 0.8% |
| dirty union | 0.5% | 0.3% | 0.4% | 0.2% | 0.2% | 0.1% |
| catalog construction | 7.3% | 7.0% | 7.1% | 5.6% | 0.3% | 2.9% |
| WAL assembly | 4.2% | 5.8% | 5.8% | 7.1% | 2.0% | 2.7% |
| WAL redo plan (delta encode) | 15.4% | 16.4% | 16.0% | 14.4% | 10.3% | 8.8% |
| WAL frame encode | 3.6% | 2.8% | 3.7% | 2.5% | 2.4% | 1.7% |
| WAL write | 10.8% | 1.8% | 4.3% | 0.9% | 2.2% | 2.2% |
| WAL sync | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| state install | 3.2% | 3.9% | 3.8% | 4.4% | 0.1% | 1.5% |
| generation publication | 6.4% | 6.1% | 6.4% | 4.7% | 0.4% | 3.9% |
| dirty tracking | 2.4% | 3.8% | 3.2% | 4.0% | 0.1% | 1.4% |
| unattributed | 7.0% | 5.5% | 7.3% | 6.6% | 6.9% | 10.9% |

## Detail timers per tx, ns (disabled sync)

| Timer | 16w w1 uniform | 16w w16 uniform | 64w w1 uniform | 64w w16 uniform | 64w w16 compact | 64w w16 spread |
|---|---|---|---|---|---|---|
| planner route | 1,976 | 25,466 | 1,719 | 23,186 | 7,389 | 10,330 |
| leaf load clone (inside physical mutation) | 1,403 | 21,822 | 1,342 | 20,894 | 64 | 2,077 |
| catalog state scan (inside catalog) | 1,780 | 23,495 | 1,620 | 21,673 | 93 | 2,743 |
| catalog chunk clone (inside catalog) | 831 | 6,464 | 614 | 2,710 | 20 | 940 |
| catalog map clone (inside catalog) | 946 | 6,591 | 646 | 2,759 | 72 | 980 |
| catalog directory clone (inside catalog) | 116 | 127 | 32 | 49 | 52 | 40 |
| retired generation drop (inside publication) | 1,692 | 21,107 | 1,527 | 18,764 | 211 | 3,964 |
| publication swap (inside publication) | 3 | 3 | 1 | 1 | 1 | 1 |
| WAL append total (plan + encode + write) | 8,053 | 73,511 | 5,752 | 71,579 | 8,655 | 12,793 |

## Work counts per transaction (real sync)

| Scenario | mutations/tx | page encodes/tx | encodes/mutation | deltas/tx | images/tx | delta payload B/tx | delta B/mutation | spans/tx | spans/mutation | WAL B/tx |
|---|---|---|---|---|---|---|---|---|---|---|
| 16w w1 uniform | 1.00 | 1.00 | 1.000 | 1.000 | 0.0000 | 106.8 | 106.8 | 4.00 | 4.00 | 226.8 |
| 16w w16 uniform | 16.00 | 15.98 | 0.999 | 15.980 | 0.0040 | 1,705.9 | 106.6 | 63.93 | 4.00 | 2,621.4 |
| 64w w1 uniform | 1.00 | 1.00 | 1.000 | 1.000 | 0.0000 | 106.7 | 106.7 | 4.00 | 4.00 | 226.7 |
| 64w w16 uniform | 16.00 | 15.98 | 0.999 | 15.980 | 0.0024 | 1,702.8 | 106.4 | 63.90 | 3.99 | 2,612.0 |
| 64w w16 compact | 16.00 | 2.00 | 0.125 | 2.000 | 0.0000 | 229.4 | 14.3 | 20.17 | 1.26 | 401.4 |
| 64w w16 spread | 16.00 | 2.00 | 0.125 | 2.000 | 0.0000 | 1,192.1 | 74.5 | 21.99 | 1.37 | 1,364.1 |

## Page locality per transaction (real sync, all measured transactions)

| Scenario | mutations mean / p50 / p95 | dirty pages mean / p50 / p95 / max | leaves mean / p50 / p95 | same-leaf mutations/tx | structural tx | leaf splits |
|---|---|---|---|---|---|---|
| 16w w1 uniform | 1.00 / 1 / 1 | 1.00 / 1 / 1 / 1 | 1.00 / 1 / 1 | 0.00 | 0 | 0 |
| 16w w16 uniform | 16.00 / 16 / 16 | 15.98 / 16 / 16 / 18 | 15.98 / 16 / 16 | 0.02 | 6 | 6 |
| 64w w1 uniform | 1.00 / 1 / 1 | 1.00 / 1 / 1 / 1 | 1.00 / 1 / 1 | 0.00 | 0 | 0 |
| 64w w16 uniform | 16.00 / 16 / 16 | 15.98 / 16 / 16 / 18 | 15.98 / 16 / 16 | 0.02 | 4 | 4 |
| 64w w16 compact | 16.00 / 16 / 16 | 2.00 / 2 / 2 / 2 | 2.00 / 2 / 2 | 14.00 | 0 | 0 |
| 64w w16 spread | 16.00 / 16 / 16 | 2.00 / 2 / 2 / 2 | 2.00 / 2 / 2 | 14.00 | 0 | 0 |

## Width 16 / width 1 scaling of per-transaction time (uniform)

| Component | 16w real | 64w real | 16w no-sync | 64w no-sync |
|---|---|---|---|---|
| admission | 12.8× | 15.7× | 13.9× | 17.6× |
| planning | 13.7× | 25.5× | 15.7× | 26.8× |
| physical mutation | 17.0× | 17.1× | 16.8× | 17.0× |
| physical restamp | 24.0× | 19.7× | 16.3× | 19.4× |
| page encode | 15.3× | 16.7× | 15.8× | 16.7× |
| physical other | 2.6× | 4.1× | 2.9× | 3.9× |
| dirty union | 6.7× | 9.4× | 6.8× | 9.0× |
| catalog construction | 11.1× | 11.2× | 12.4× | 13.2× |
| WAL assembly | 18.2× | 20.8× | 17.9× | 20.5× |
| WAL redo plan (delta encode) | 13.6× | 14.9× | 13.8× | 15.1× |
| WAL frame encode | 10.4× | 11.2× | 10.3× | 11.5× |
| WAL write | 1.8× | 4.2× | 2.2× | 3.6× |
| WAL sync | 1.5× | 3.4× | 1.6× | 2.0× |
| state install | 14.2× | 14.5× | 15.8× | 19.4× |
| generation publication | 11.0× | 12.1× | 12.3× | 12.2× |
| dirty tracking | 17.8× | 19.7× | 20.7× | 20.8× |
| unattributed | 7.8× | 14.3× | 10.1× | 15.2× |
| **total** | 4.1× | 11.3× | 13.0× | 16.7× |
| mutations | 16.0× | 16.0× | 16.0× | 16.0× |

## Real sync vs sync disabled

| Scenario | real tx/s | no-sync tx/s | no-sync / real | real ns/tx | no-sync ns/tx | WAL sync ns/tx (real) | sync share of real coordinator time |
|---|---|---|---|---|---|---|---|
| 16w w1 uniform | 8,156 | 33,413 | 4.10 | 119,675 | 26,800 | 89,986 | 75.2% |
| 16w w16 uniform | 1,913 | 2,606 | 1.36 | 485,509 | 348,307 | 133,980 | 27.6% |
| 64w w1 uniform | 23,545 | 37,350 | 1.59 | 39,727 | 23,928 | 15,136 | 38.1% |
| 64w w16 uniform | 2,063 | 2,270 | 1.10 | 447,232 | 399,921 | 51,150 | 11.4% |
| 64w w16 compact | 10,788 | 14,245 | 1.32 | 80,661 | 57,782 | 22,810 | 28.3% |
| 64w w16 spread | 5,860 | 7,904 | 1.35 | 143,585 | 100,718 | 39,166 | 27.3% |

## RocksDB same-session check

| Scenario | dodb tx/s | RocksDB tx/s | dodb / RocksDB | dodb p99 µs | RocksDB p99 µs | dodb CPU % | RocksDB CPU % |
|---|---|---|---|---|---|---|---|
| 64w w1 uniform | 23,986 | 26,208 | 0.915 | 5,148 | 3,927 | 68 | 64 |
| 64w w16 uniform | 2,060 | 9,805 | 0.210 | 57,860 | 14,893 | 92 | 85 |

