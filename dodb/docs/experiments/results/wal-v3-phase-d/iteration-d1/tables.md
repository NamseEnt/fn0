> **Iteration D1 (code `a917c84`, bench flags `9932f93`, binary SHA256 `cb1d9a41…`).** In D1, `--parallel-workers n` started n worker threads, the coordinator only waited during the parallel section, jobs were split statically between the threads, and the coordinator cloned every target leaf while building the jobs. So for D1 read `par1` as "one worker thread, coordinator waiting" and the row "coordinator waits for worker threads" as the whole parallel wall time. The header line below was written for D2 and does not apply to this folder.

# WAL v3 Phase D tables

planned Blink, 100,000 rows, 16-byte keys, 64-byte values, 2 s warmup, 5 s measure, 3 repetitions unless noted. `phase-c` = binary `cb7fb54` (serial planned executor). `serial-d`, `par1`, `par2` = Phase D binary with `--parallel-workers 0 / 1 / 2` (lanes: 1 = the coordinator alone runs the leaf jobs, 2 = coordinator + one worker thread). `disabled` rows skip fsync (CPU control, not durable throughput).

## Sync-disabled CPU gate

| Scenario | phase-c tx/s (runs) | serial-d tx/s (runs) | par1 tx/s (runs) | par2 tx/s (runs) | serial-d / phase-c | par1 / phase-c | par2 / phase-c | p99 µs phase-c → serial-d → par1 → par2 |
|---|---|---|---|---|---|---|---|---|
| 64w w16 uniform | 2,254 (2,257, 2,249, 2,255) | 2,236 (2,230, 2,244, 2,234) | 2,343 (2,395, 2,326, 2,307) | 2,489 (2,414, 2,453, 2,599) | 0.992 | 1.040 | 1.104 | 56,370 → 59,934 → 50,442 → 60,715 |
| 64w w16 compact | 14,204 (14,480, 14,223, 13,909) | 14,023 (14,318, 13,833, 13,917) | 14,146 (14,210, 14,277, 13,952) | 15,641 (15,609, 15,544, 15,769) | 0.987 | 0.996 | 1.101 | 7,157 → 8,379 → 7,034 → 7,015 |
| 64w w16 spread | 7,795 (7,975, 7,833, 7,577) | 7,809 (7,754, 8,017, 7,655) | 8,224 (8,206, 8,290, 8,177) | 8,238 (8,294, 8,083, 8,336) | 1.002 | 1.055 | 1.057 | 16,438 → 17,919 → 14,749 → 15,013 |

Same-binary ratios:

| Scenario | par2 / serial-d | par1 / serial-d | par2 / par1 |
|---|---|---|---|
| 64w w16 uniform | 1.113 | 1.048 | 1.062 |
| 64w w16 compact | 1.115 | 1.009 | 1.106 |
| 64w w16 spread | 1.055 | 1.053 | 1.002 |

### Coordinator time per transaction, ns (cpu, sync disabled)

| Component | 64w w16 uniform phase-c | 64w w16 uniform par1 | 64w w16 uniform par2 | 64w w16 compact phase-c | 64w w16 compact par1 | 64w w16 compact par2 | 64w w16 spread phase-c | 64w w16 spread par1 | 64w w16 spread par2 |
|---|---|---|---|---|---|---|---|---|---|
| admission | 12,095 | 11,896 | 12,431 | 7,073 | 7,101 | 7,187 | 10,633 | 10,090 | 10,087 |
| planning | 83,282 | 80,629 | 84,644 | 20,202 | 20,364 | 20,427 | 31,500 | 29,844 | 30,310 |
| serial physical mutation | 59,671 | 399 | 581 | 12,201 | 0 | 0 | 17,314 | 0 | 0 |
| serial physical restamp | 3,041 | 26 | 29 | 913 | 0 | 0 | 1,263 | 0 | 0 |
| serial page encode | 38,689 | 309 | 414 | 2,912 | 0 | 0 | 3,526 | 0 | 0 |
| parallel dispatch (job build) | 0 | 47,951 | 47,997 | 0 | 387 | 390 | 0 | 5,312 | 5,529 |
| parallel lanes: coordinator runs leaf jobs | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| parallel lanes: coordinator waits for worker threads | 0 | 126,776 | 96,856 | 0 | 21,717 | 14,954 | 0 | 29,087 | 26,710 |
| parallel result collection | 0 | 4,556 | 4,577 | 0 | 296 | 277 | 0 | 710 | 742 |
| physical other | 1,365 | 281 | 318 | 333 | 75 | 72 | 1,252 | 82 | 91 |
| dirty union | 901 | 706 | 714 | 99 | 84 | 88 | 149 | 111 | 122 |
| catalog construction | 23,322 | 25,146 | 24,469 | 165 | 181 | 180 | 2,933 | 3,123 | 3,266 |
| WAL assembly | 28,581 | 1,192 | 1,233 | 1,150 | 120 | 125 | 2,745 | 197 | 233 |
| WAL redo plan (serial: delta encode; parallel: chain check) | 57,670 | 4,020 | 4,110 | 6,035 | 188 | 174 | 8,977 | 455 | 468 |
| WAL frame encode | 10,144 | 8,967 | 9,118 | 1,412 | 1,233 | 1,253 | 1,780 | 1,577 | 1,650 |
| WAL write | 3,693 | 3,924 | 3,820 | 1,419 | 1,230 | 1,313 | 2,148 | 2,042 | 2,151 |
| WAL sync | 5 | 4 | 4 | 3 | 3 | 3 | 3 | 3 | 4 |
| state install | 18,267 | 16,389 | 15,607 | 67 | 65 | 66 | 1,542 | 1,535 | 1,568 |
| generation publication | 18,863 | 18,353 | 18,251 | 229 | 217 | 226 | 3,938 | 3,750 | 4,012 |
| dirty tracking | 16,032 | 19,840 | 20,085 | 64 | 81 | 84 | 1,455 | 1,589 | 1,840 |
| unattributed | 26,253 | 22,385 | 23,299 | 3,978 | 4,710 | 4,333 | 10,898 | 9,208 | 8,998 |
| **total** | 401,874 | 393,746 | 368,558 | 58,256 | 58,053 | 51,153 | 102,057 | 98,714 | 97,779 |

### Grouped as Phase C categories, share of coordinator time (cpu, sync disabled)

| Category | 64w w16 uniform phase-c | 64w w16 uniform par1 | 64w w16 uniform par2 | 64w w16 compact phase-c | 64w w16 compact par1 | 64w w16 compact par2 | 64w w16 spread phase-c | 64w w16 spread par1 | 64w w16 spread par2 |
|---|---|---|---|---|---|---|---|---|---|
| planner (admission + planning) | 23.7% | 23.5% | 26.3% | 46.8% | 47.3% | 54.0% | 41.3% | 40.5% | 41.3% |
| leaf physical on coordinator (serial mutation, restamp, encode, other) | 25.6% | 0.3% | 0.4% | 28.1% | 0.1% | 0.1% | 22.9% | 0.1% | 0.1% |
| leaf jobs run by the coordinator lane | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| parallel dispatch + wait for worker threads + collect | 0.0% | 45.5% | 40.5% | 0.0% | 38.6% | 30.5% | 0.0% | 35.6% | 33.7% |
| WAL CPU (assembly, redo plan, frame encode, write) | 24.9% | 4.6% | 5.0% | 17.2% | 4.8% | 5.6% | 15.3% | 4.3% | 4.6% |
| catalog / publication / install / dirty | 19.3% | 20.4% | 21.5% | 1.1% | 1.1% | 1.3% | 9.8% | 10.2% | 11.1% |
| WAL sync | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% | 0.0% |
| unattributed | 6.5% | 5.7% | 6.3% | 6.8% | 8.1% | 8.5% | 10.7% | 9.3% | 9.2% |

### Worker side per transaction, ns (cpu, sync disabled)

| Item | 64w w16 uniform phase-c | 64w w16 uniform par1 | 64w w16 uniform par2 | 64w w16 compact phase-c | 64w w16 compact par1 | 64w w16 compact par2 | 64w w16 spread phase-c | 64w w16 spread par1 | 64w w16 spread par2 |
|---|---|---|---|---|---|---|---|---|---|
| base image + chain check | 0 | 9,167 | 11,500 | 0 | 62 | 80 | 0 | 775 | 885 |
| mutation | 0 | 32,090 | 38,810 | 0 | 11,735 | 11,747 | 0 | 14,053 | 14,484 |
| page encode | 0 | 36,239 | 42,786 | 0 | 2,872 | 2,874 | 0 | 3,338 | 3,530 |
| delta encode + verify + CRC | 0 | 37,275 | 42,303 | 0 | 5,259 | 5,452 | 0 | 6,982 | 7,418 |
| all lanes busy (coordinator lane + threads) | 0 | 121,979 | 144,565 | 0 | 20,286 | 20,603 | 0 | 26,703 | 28,228 |
| lane idle inside the parallel section | 0 | 4,797 | 49,148 | 0 | 1,432 | 9,304 | 0 | 2,383 | 25,193 |
| effective parallelism (lanes busy / parallel wall) | 0.00 | 0.96 | 1.49 | 0.00 | 0.93 | 1.38 | 0.00 | 0.92 | 1.06 |
| coordinator busy (incl. join wait) | 90.6% | 92.2% | 91.7% | 82.8% | 82.1% | 80.0% | 79.6% | 81.2% | 80.5% |
| coordinator busy (excl. join wait) | 90.6% | 62.5% | 67.6% | 82.8% | 51.4% | 56.6% | 79.6% | 57.3% | 58.5% |

### Parallel execution counters (sync-disabled gate)

| Run set | groups | parallel groups | parallel share | fallback groups | after dispatch | overflow / structural / route / no-delta WAL | single-leaf groups | leaf jobs / parallel group | operations / job | mutations / job | tx / group |
|---|---|---|---|---|---|---|---|---|---|---|---|
| cpu disabled 64w w16 uniform par1 | 654 | 650 | 99.4% | 4 | 4 | 0 / 4 / 0 / 0 | 0 | 800.5 | 1.075 | 1.08 | 53.9 |
| cpu disabled 64w w16 uniform par2 | 690 | 685 | 99.3% | 5 | 5 | 0 / 5 / 0 / 0 | 0 | 804.3 | 1.076 | 1.08 | 54.2 |
| cpu disabled 64w w16 compact par1 | 5,199 | 5,199 | 100.0% | 0 | 0 | 0 / 0 / 0 / 0 | 0 | 4.8 | 16.955 | 135.64 | 40.8 |
| cpu disabled 64w w16 compact par2 | 5,885 | 5,885 | 100.0% | 0 | 0 | 0 / 0 / 0 / 0 | 0 | 4.8 | 16.482 | 131.85 | 39.9 |
| cpu disabled 64w w16 spread par1 | 2,240 | 2,240 | 100.0% | 0 | 0 | 0 / 0 / 0 / 0 | 0 | 110.3 | 1.000 | 8.00 | 55.2 |
| cpu disabled 64w w16 spread par2 | 2,289 | 2,289 | 100.0% | 0 | 0 | 0 / 0 / 0 / 0 | 0 | 108.1 | 1.000 | 8.00 | 54.0 |

