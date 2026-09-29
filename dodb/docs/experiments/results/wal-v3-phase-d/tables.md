# WAL v3 Phase D tables

planned Blink, 100,000 rows, 16-byte keys, 64-byte values, 2 s warmup, 5 s measure, 3 repetitions unless noted. `phase-c` = binary `cb7fb54` (serial planned executor). `serial-d`, `par1`, `par2` = Phase D binary with `--parallel-workers 0 / 1 / 2` (lanes: 1 = the coordinator alone runs the leaf jobs, 2 = coordinator + one worker thread). `disabled` rows skip fsync (CPU control, not durable throughput).

## Sync-disabled CPU gate

| Scenario | phase-c tx/s (runs) | serial-d tx/s (runs) | par1 tx/s (runs) | par2 tx/s (runs) | serial-d / phase-c | par1 / phase-c | par2 / phase-c | p99 µs phase-c → serial-d → par1 → par2 |
|---|---|---|---|---|---|---|---|---|
| 64w w16 uniform | 2,249 (2,259, 2,231, 2,256) | 2,236 (2,270, 2,242, 2,195) | 2,313 (2,317, 2,307, 2,315) | 2,765 (2,759, 2,822, 2,716) | 0.994 | 1.029 | 1.230 | 53,255 → 52,072 → 53,335 → 47,237 |
| 64w w16 compact | 14,145 (14,335, 14,298, 13,802) | 14,350 (14,504, 14,148, 14,397) | 15,019 (14,916, 15,168, 14,973) | 15,765 (15,731, 15,925, 15,637) | 1.014 | 1.062 | 1.115 | 7,052 → 7,127 → 6,472 → 7,779 |
| 64w w16 spread | 7,915 (8,051, 7,749, 7,945) | 7,895 (7,891, 7,894, 7,901) | 8,087 (8,020, 8,209, 8,032) | 8,664 (8,655, 8,663, 8,673) | 0.998 | 1.022 | 1.095 | 14,861 → 15,086 → 15,187 → 14,763 |

Same-binary ratios:

| Scenario | par2 / serial-d | par1 / serial-d | par2 / par1 |
|---|---|---|---|
| 64w w16 uniform | 1.237 | 1.035 | 1.196 |
| 64w w16 compact | 1.099 | 1.047 | 1.050 |
| 64w w16 spread | 1.097 | 1.024 | 1.071 |

### Coordinator time per transaction, ns (cpu, sync disabled)

| Component | 64w w16 uniform phase-c | 64w w16 uniform par1 | 64w w16 uniform par2 | 64w w16 compact phase-c | 64w w16 compact par1 | 64w w16 compact par2 | 64w w16 spread phase-c | 64w w16 spread par1 | 64w w16 spread par2 |
|---|---|---|---|---|---|---|---|---|---|
| admission | 12,035 | 12,141 | 11,780 | 7,092 | 7,084 | 7,286 | 10,380 | 10,456 | 10,532 |
| planning | 83,784 | 87,878 | 87,147 | 20,383 | 20,128 | 20,833 | 30,688 | 30,917 | 30,959 |
| serial physical mutation | 59,333 | 427 | 488 | 12,215 | 0 | 0 | 17,028 | 0 | 0 |
| serial physical restamp | 3,003 | 27 | 31 | 921 | 0 | 0 | 1,206 | 0 | 0 |
| serial page encode | 38,722 | 306 | 336 | 2,927 | 0 | 0 | 3,512 | 0 | 0 |
| parallel dispatch (job build) | 0 | 28,851 | 27,440 | 0 | 296 | 323 | 0 | 3,393 | 3,546 |
| parallel lanes: coordinator runs leaf jobs | 0 | 129,101 | 65,647 | 0 | 19,644 | 10,575 | 0 | 26,305 | 15,651 |
| parallel lanes: coordinator waits for worker threads | 0 | 27 | 7,146 | 0 | 15 | 3,346 | 0 | 26 | 3,443 |
| parallel result collection | 0 | 3,823 | 5,678 | 0 | 244 | 298 | 0 | 573 | 634 |
| physical other | 1,380 | 1,360 | 1,408 | 374 | 75 | 137 | 817 | 204 | 240 |
| dirty union | 860 | 655 | 647 | 106 | 69 | 74 | 152 | 98 | 101 |
| catalog construction | 23,614 | 24,206 | 22,983 | 166 | 148 | 174 | 2,887 | 3,007 | 3,074 |
| WAL assembly | 28,917 | 1,086 | 1,165 | 1,179 | 102 | 111 | 2,778 | 185 | 198 |
| WAL redo plan (serial: delta encode; parallel: chain check) | 58,484 | 3,861 | 3,853 | 6,011 | 149 | 172 | 8,822 | 441 | 466 |
| WAL frame encode | 10,257 | 8,694 | 8,536 | 1,402 | 1,206 | 1,251 | 1,798 | 1,644 | 1,625 |
| WAL write | 3,755 | 3,782 | 3,691 | 1,365 | 1,232 | 1,392 | 2,220 | 2,134 | 2,083 |
| WAL sync | 5 | 3 | 4 | 3 | 2 | 2 | 3 | 3 | 3 |
| state install | 18,314 | 15,705 | 15,141 | 71 | 59 | 76 | 1,537 | 1,604 | 1,747 |
| generation publication | 19,424 | 17,008 | 16,519 | 226 | 183 | 233 | 3,926 | 3,603 | 3,890 |
| dirty tracking | 16,032 | 19,508 | 18,060 | 63 | 73 | 87 | 1,482 | 1,676 | 1,704 |
| unattributed | 25,492 | 28,222 | 24,842 | 4,071 | 3,751 | 4,431 | 10,946 | 10,014 | 10,051 |
| **total** | 403,410 | 386,671 | 322,543 | 58,576 | 54,460 | 50,801 | 100,181 | 96,282 | 89,947 |

### Grouped as Phase C categories, share of coordinator time (cpu, sync disabled)

| Category | 64w w16 uniform phase-c | 64w w16 uniform par1 | 64w w16 uniform par2 | 64w w16 compact phase-c | 64w w16 compact par1 | 64w w16 compact par2 | 64w w16 spread phase-c | 64w w16 spread par1 | 64w w16 spread par2 |
|---|---|---|---|---|---|---|---|---|---|
| planner (admission + planning) | 23.8% | 25.9% | 30.7% | 46.9% | 50.0% | 55.4% | 41.0% | 43.0% | 46.1% |
| leaf physical on coordinator (serial mutation, restamp, encode, other) | 25.4% | 0.5% | 0.7% | 28.1% | 0.1% | 0.3% | 22.5% | 0.2% | 0.3% |
| leaf jobs run by the coordinator lane | 0.0% | 33.4% | 20.4% | 0.0% | 36.1% | 20.8% | 0.0% | 27.3% | 17.4% |
| parallel dispatch + wait for worker threads + collect | 0.0% | 8.5% | 12.5% | 0.0% | 1.0% | 7.8% | 0.0% | 4.1% | 8.5% |
| WAL CPU (assembly, redo plan, frame encode, write) | 25.1% | 4.5% | 5.3% | 17.0% | 4.9% | 5.8% | 15.6% | 4.6% | 4.9% |
| catalog / publication / install / dirty | 19.4% | 19.9% | 22.7% | 1.1% | 1.0% | 1.3% | 10.0% | 10.4% | 11.7% |
| WAL sync | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| unattributed | 6.3% | 7.3% | 7.7% | 7.0% | 6.9% | 8.7% | 10.9% | 10.4% | 11.2% |

### Worker side per transaction, ns (cpu, sync disabled)

| Item | 64w w16 uniform phase-c | 64w w16 uniform par1 | 64w w16 uniform par2 | 64w w16 compact phase-c | 64w w16 compact par1 | 64w w16 compact par2 | 64w w16 spread phase-c | 64w w16 spread par1 | 64w w16 spread par2 |
|---|---|---|---|---|---|---|---|---|---|
| base image + chain check | 0 | 28,791 | 28,014 | 0 | 100 | 159 | 0 | 2,796 | 3,086 |
| mutation | 0 | 23,183 | 24,261 | 0 | 11,233 | 11,840 | 0 | 12,824 | 13,673 |
| page encode | 0 | 33,402 | 35,148 | 0 | 2,854 | 2,964 | 0 | 3,315 | 4,043 |
| delta encode + verify + CRC | 0 | 36,086 | 37,799 | 0 | 5,135 | 5,370 | 0 | 6,505 | 7,903 |
| all lanes busy (coordinator lane + threads) | 0 | 129,101 | 134,271 | 0 | 19,644 | 20,795 | 0 | 26,305 | 30,730 |
| lane idle inside the parallel section | 0 | 27 | 11,315 | 0 | 15 | 7,047 | 0 | 26 | 7,458 |
| effective parallelism (lanes busy / parallel wall) | 0.00 | 1.00 | 1.84 | 0.00 | 1.00 | 1.49 | 0.00 | 1.00 | 1.61 |
| coordinator busy (incl. join wait) | 90.7% | 89.4% | 89.2% | 82.9% | 81.8% | 80.1% | 79.3% | 77.9% | 77.9% |
| coordinator busy (excl. join wait) | 90.7% | 89.4% | 87.2% | 82.9% | 81.8% | 74.8% | 79.3% | 77.9% | 75.0% |

### Parallel execution counters (sync-disabled gate)

| Run set | groups | parallel groups | parallel share | fallback groups | after dispatch | overflow / structural / route / no-delta WAL | single-leaf groups | leaf jobs / parallel group | operations / job | mutations / job | tx / group |
|---|---|---|---|---|---|---|---|---|---|---|---|
| cpu disabled 64w w16 uniform par1 | 614 | 610 | 99.3% | 4 | 4 | 0 / 4 / 0 / 0 | 0 | 841.0 | 1.076 | 1.08 | 56.7 |
| cpu disabled 64w w16 uniform par2 | 743 | 737 | 99.2% | 6 | 6 | 0 / 6 / 0 / 0 | 0 | 831.9 | 1.076 | 1.08 | 56.0 |
| cpu disabled 64w w16 compact par1 | 5,344 | 5,344 | 100.0% | 0 | 0 | 0 / 0 / 0 / 0 | 0 | 4.8 | 17.549 | 140.39 | 42.2 |
| cpu disabled 64w w16 compact par2 | 5,738 | 5,738 | 100.0% | 0 | 0 | 0 / 0 / 0 / 0 | 0 | 4.8 | 17.067 | 136.53 | 41.2 |
| cpu disabled 64w w16 spread par1 | 2,338 | 2,338 | 100.0% | 0 | 0 | 0 / 0 / 0 / 0 | 0 | 103.9 | 1.000 | 8.00 | 52.0 |
| cpu disabled 64w w16 spread par2 | 2,393 | 2,393 | 100.0% | 0 | 0 | 0 / 0 / 0 / 0 | 0 | 108.7 | 1.000 | 8.00 | 54.3 |

## Real-sync durable gate (ZFS)

| Scenario | phase-c tx/s (runs) | par2 tx/s (runs) | par2 / phase-c | p99 µs phase-c → par2 |
|---|---|---|---|---|
| 16w w1 uniform | 7,830 (7,746, 7,818, 7,927) | 7,593 (7,608, 7,644, 7,526) | 0.970 | 3,496 → 3,754 |
| 16w w16 uniform | 1,933 (1,981, 1,891, 1,926) | 2,177 (2,075, 2,279, 2,177) | 1.126 | 15,874 → 14,282 |
| 64w w1 uniform | 23,275 (23,394, 23,451, 22,980) | 25,650 (25,994, 25,444, 25,512) | 1.102 | 5,374 → 5,021 |
| 64w w16 uniform | 2,031 (2,038, 2,026, 2,030) | 2,417 (2,427, 2,404, 2,418) | 1.190 | 58,693 → 48,987 |
| 64w w16 compact | 10,673 (10,673, 10,750, 10,596) | 11,788 (11,742, 11,726, 11,897) | 1.105 | 9,412 → 8,774 |
| 64w w16 spread | 5,996 (6,035, 5,928, 6,025) | 6,376 (6,514, 6,566, 6,048) | 1.063 | 19,754 → 19,068 |

GM par2 / phase-c over the 6 scenarios: 1.091

### Coordinator time per transaction, ns (gate, sync real)

| Component | 16w w1 uniform phase-c | 16w w1 uniform par2 | 16w w16 uniform phase-c | 16w w16 uniform par2 | 64w w1 uniform phase-c | 64w w1 uniform par2 | 64w w16 uniform phase-c | 64w w16 uniform par2 | 64w w16 compact phase-c | 64w w16 compact par2 | 64w w16 spread phase-c | 64w w16 spread par2 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| admission | 780 | 785 | 10,626 | 9,653 | 676 | 598 | 11,729 | 11,474 | 7,017 | 7,067 | 10,340 | 10,512 |
| planning | 3,985 | 4,009 | 55,055 | 54,650 | 3,140 | 3,051 | 82,893 | 86,505 | 20,001 | 20,222 | 30,526 | 30,685 |
| serial physical mutation | 3,303 | 144 | 55,532 | 156 | 3,333 | 6 | 57,930 | 381 | 12,085 | 0 | 16,822 | 0 |
| serial physical restamp | 109 | 2 | 2,596 | 9 | 151 | 0 | 2,981 | 25 | 908 | 0 | 1,207 | 0 |
| serial page encode | 2,346 | 28 | 35,953 | 130 | 2,235 | 1 | 38,219 | 257 | 2,903 | 0 | 3,430 | 0 |
| parallel dispatch (job build) | 0 | 2,287 | 0 | 25,195 | 0 | 1,653 | 0 | 27,788 | 0 | 304 | 0 | 3,418 |
| parallel lanes: coordinator runs leaf jobs | 0 | 4,871 | 0 | 66,477 | 0 | 4,348 | 0 | 64,087 | 0 | 10,384 | 0 | 15,175 |
| parallel lanes: coordinator waits for worker threads | 0 | 2,213 | 0 | 5,192 | 0 | 531 | 0 | 3,183 | 0 | 1,916 | 0 | 1,822 |
| parallel result collection | 0 | 534 | 0 | 3,775 | 0 | 394 | 0 | 4,586 | 0 | 313 | 0 | 632 |
| physical other | 390 | 658 | 1,055 | 1,516 | 329 | 163 | 1,358 | 1,221 | 340 | 106 | 822 | 243 |
| dirty union | 136 | 102 | 914 | 704 | 90 | 57 | 835 | 658 | 97 | 70 | 147 | 105 |
| catalog construction | 2,148 | 2,343 | 23,752 | 23,203 | 1,651 | 1,658 | 22,320 | 22,838 | 160 | 169 | 2,841 | 3,545 |
| WAL assembly | 1,224 | 182 | 20,679 | 819 | 1,397 | 159 | 28,094 | 1,030 | 1,165 | 107 | 2,817 | 203 |
| WAL redo plan (serial: delta encode; parallel: chain check) | 4,110 | 305 | 56,471 | 3,381 | 3,757 | 229 | 56,468 | 3,721 | 5,790 | 166 | 8,850 | 445 |
| WAL frame encode | 910 | 859 | 9,645 | 7,903 | 889 | 806 | 10,213 | 8,383 | 1,363 | 1,205 | 1,764 | 1,650 |
| WAL write | 3,601 | 3,768 | 6,575 | 6,734 | 866 | 896 | 3,533 | 3,642 | 1,299 | 1,287 | 2,232 | 2,303 |
| WAL sync | 94,851 | 98,405 | 130,685 | 137,640 | 16,442 | 16,340 | 55,195 | 55,915 | 23,468 | 23,850 | 39,685 | 40,955 |
| state install | 1,154 | 1,380 | 15,004 | 17,887 | 1,083 | 1,236 | 17,981 | 16,832 | 100 | 115 | 1,776 | 2,524 |
| generation publication | 1,998 | 1,853 | 21,000 | 19,102 | 1,546 | 1,404 | 18,442 | 17,808 | 262 | 300 | 3,943 | 4,589 |
| dirty tracking | 863 | 1,027 | 14,688 | 17,017 | 852 | 956 | 16,805 | 18,073 | 76 | 98 | 1,688 | 1,825 |
| unattributed | 2,776 | 2,748 | 21,319 | 21,351 | 1,895 | 1,727 | 27,216 | 28,259 | 4,269 | 4,751 | 11,047 | 10,142 |
| **total** | 124,683 | 128,504 | 481,550 | 422,496 | 40,330 | 36,213 | 452,210 | 376,666 | 81,302 | 72,431 | 139,936 | 130,776 |

### Grouped as Phase C categories, share of coordinator time (gate, sync real)

| Category | 16w w1 uniform phase-c | 16w w1 uniform par2 | 16w w16 uniform phase-c | 16w w16 uniform par2 | 64w w1 uniform phase-c | 64w w1 uniform par2 | 64w w16 uniform phase-c | 64w w16 uniform par2 | 64w w16 compact phase-c | 64w w16 compact par2 | 64w w16 spread phase-c | 64w w16 spread par2 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| planner (admission + planning) | 3.8% | 3.7% | 13.6% | 15.2% | 9.5% | 10.1% | 20.9% | 26.0% | 33.2% | 37.7% | 29.2% | 31.5% |
| leaf physical on coordinator (serial mutation, restamp, encode, other) | 4.9% | 0.6% | 19.8% | 0.4% | 15.0% | 0.5% | 22.2% | 0.5% | 20.0% | 0.1% | 15.9% | 0.2% |
| leaf jobs run by the coordinator lane | 0.0% | 3.8% | 0.0% | 15.7% | 0.0% | 12.0% | 0.0% | 17.0% | 0.0% | 14.3% | 0.0% | 11.6% |
| parallel dispatch + wait for worker threads + collect | 0.0% | 3.9% | 0.0% | 8.1% | 0.0% | 7.1% | 0.0% | 9.4% | 0.0% | 3.5% | 0.0% | 4.5% |
| WAL CPU (assembly, redo plan, frame encode, write) | 7.9% | 4.0% | 19.4% | 4.5% | 17.1% | 5.8% | 21.7% | 4.5% | 11.8% | 3.8% | 11.2% | 3.5% |
| catalog / publication / install / dirty | 5.1% | 5.2% | 15.6% | 18.4% | 12.9% | 14.7% | 16.9% | 20.2% | 0.9% | 1.0% | 7.4% | 9.6% |
| WAL sync | 76.1% | 76.6% | 27.1% | 32.6% | 40.8% | 45.1% | 12.2% | 14.8% | 28.9% | 32.9% | 28.4% | 31.3% |
| unattributed | 2.2% | 2.1% | 4.4% | 5.1% | 4.7% | 4.8% | 6.0% | 7.5% | 5.3% | 6.6% | 7.9% | 7.8% |

### Worker side per transaction, ns (gate, sync real)

| Item | 16w w1 uniform phase-c | 16w w1 uniform par2 | 16w w16 uniform phase-c | 16w w16 uniform par2 | 64w w1 uniform phase-c | 64w w1 uniform par2 | 64w w16 uniform phase-c | 64w w16 uniform par2 | 64w w16 compact phase-c | 64w w16 compact par2 | 64w w16 spread phase-c | 64w w16 spread par2 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| base image + chain check | 0 | 2,237 | 0 | 27,805 | 0 | 1,832 | 0 | 27,039 | 0 | 168 | 0 | 3,331 |
| mutation | 0 | 1,367 | 0 | 22,300 | 0 | 1,423 | 0 | 23,638 | 0 | 11,556 | 0 | 13,722 |
| page encode | 0 | 2,406 | 0 | 34,954 | 0 | 2,183 | 0 | 34,109 | 0 | 2,895 | 0 | 3,388 |
| delta encode + verify + CRC | 0 | 2,517 | 0 | 35,829 | 0 | 2,305 | 0 | 36,305 | 0 | 5,109 | 0 | 7,932 |
| all lanes busy (coordinator lane + threads) | 0 | 9,304 | 0 | 131,092 | 0 | 8,444 | 0 | 129,432 | 0 | 20,203 | 0 | 30,348 |
| lane idle inside the parallel section | 0 | 4,864 | 0 | 12,248 | 0 | 1,314 | 0 | 5,108 | 0 | 4,397 | 0 | 3,646 |
| effective parallelism (lanes busy / parallel wall) | 0.00 | 1.31 | 0.00 | 1.83 | 0.00 | 1.73 | 0.00 | 1.92 | 0.00 | 1.64 | 0.00 | 1.79 |
| coordinator busy (incl. join wait) | 97.6% | 97.6% | 93.1% | 92.0% | 93.9% | 92.9% | 91.9% | 91.0% | 86.8% | 85.4% | 83.9% | 83.4% |
| coordinator busy (excl. join wait) | 97.6% | 95.9% | 93.1% | 90.9% | 93.9% | 91.5% | 91.9% | 90.3% | 86.8% | 83.1% | 83.9% | 82.2% |

### Parallel execution counters (real-sync gate)

| Run set | groups | parallel groups | parallel share | fallback groups | after dispatch | overflow / structural / route / no-delta WAL | single-leaf groups | leaf jobs / parallel group | operations / job | mutations / job | tx / group |
|---|---|---|---|---|---|---|---|---|---|---|---|
| gate real 16w w1 uniform par2 | 13,987 | 13,242 | 94.7% | 0 | 0 | 0 / 0 / 0 / 0 | 745 | 8.5 | 1.001 | 1.00 | 8.1 |
| gate real 16w w16 uniform par2 | 2,309 | 2,302 | 99.7% | 7 | 7 | 0 / 7 / 0 / 0 | 0 | 222.7 | 1.017 | 1.02 | 14.2 |
| gate real 64w w1 uniform par2 | 6,335 | 6,287 | 99.2% | 0 | 0 | 0 / 0 / 0 / 0 | 48 | 61.0 | 1.005 | 1.00 | 60.8 |
| gate real 64w w16 uniform par2 | 596 | 592 | 99.3% | 4 | 4 | 0 / 4 / 0 / 0 | 0 | 905.4 | 1.077 | 1.08 | 61.0 |
| gate real 64w w16 compact par2 | 4,165 | 4,165 | 100.0% | 0 | 0 | 0 / 0 / 0 / 0 | 0 | 4.8 | 17.547 | 140.38 | 42.5 |
| gate real 64w w16 spread par2 | 1,628 | 1,628 | 100.0% | 0 | 0 | 0 / 0 / 0 / 0 | 0 | 117.6 | 1.000 | 8.00 | 58.8 |

### WAL and sync (real-sync gate)

| Scenario | variant | WAL B/tx | deltas/tx | images/tx | tx/sync | mean sync ms | CPU % one core |
|---|---|---|---|---|---|---|---|
| 16w w1 uniform | phase-c | 226.8 | 1.000 | 0.0000 | 8.2 | 0.78 | 30 |
| 16w w1 uniform | par2 | 226.8 | 1.000 | 0.0000 | 8.1 | 0.80 | 33 |
| 16w w16 uniform | phase-c | 2,623.9 | 15.980 | 0.0046 | 14.7 | 1.92 | 78 |
| 16w w16 uniform | par2 | 2,621.6 | 15.980 | 0.0041 | 14.2 | 1.95 | 87 |
| 64w w1 uniform | phase-c | 226.8 | 1.000 | 0.0000 | 57.3 | 0.94 | 66 |
| 64w w1 uniform | par2 | 226.8 | 1.000 | 0.0000 | 60.8 | 0.99 | 74 |
| 64w w16 uniform | phase-c | 2,612.1 | 15.980 | 0.0025 | 61.2 | 3.38 | 91 |
| 64w w16 uniform | par2 | 2,610.6 | 15.980 | 0.0021 | 61.0 | 3.41 | 104 |
| 64w w16 compact | phase-c | 386.8 | 2.000 | 0.0000 | 43.1 | 1.01 | 87 |
| 64w w16 compact | par2 | 397.3 | 2.000 | 0.0000 | 42.5 | 1.01 | 95 |
| 64w w16 spread | phase-c | 1,364.1 | 2.000 | 0.0000 | 60.5 | 2.40 | 84 |
| 64w w16 spread | par2 | 1,364.1 | 2.000 | 0.0000 | 58.8 | 2.41 | 90 |

