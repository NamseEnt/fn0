# OCI A1 dual rebaseline: exact main vs planned Blink

## Scope and source provenance

- Experiment source: `93b1e7546710ef4863398c36464595c912f85dc4` (`experiment/b-link-batched-engine`).
- Exact main source: `ac45cf519d6cd008dfbc5362f72561f9092a7b11` (`main`).
- Merge base: `1ff96e1b3d205074d4c1b820f5f2680bd3226a8b`; the branches are diverged.
- Both comparisons are unconditional mutation transactions only. Main supports condition-only transactions and returns `Option<Lsn>`; experiment rejects mutation-free transactions and returns `Lsn`. This benchmark makes no condition-only performance or API-compatibility claim.
- Workload: working set 100,000; cache 4,096; 16-byte key; 64-byte value; group limit 64; group bytes 4,194,304; queue 256; collection delay 0 µs; sync disabled; Tokio workers 2; warmup 1 s; measured duration 2 s; 3 repetitions.
- OCI A1 host: AArch64 Neoverse-N1, 2 vCPU; visible root filesystem is 29.4 GiB XFS with 16.8 GiB available at capture. No 200 GiB filesystem or ZFS was used.
- All measurements are sync-disabled CPU/engine diagnostics. They are not a real-durability benchmark.

## Comparison A: same-source BTree vs planned

Twelve-scenario mutation-throughput geometric mean: **3.04x (Very strong)**. Ratio is `Planned / E-BTree` from the exact same experiment executable.

## Comparison B: exact current-main BTree vs planned

Twelve-scenario mutation-throughput geometric mean: **3.59x (Very strong)**. Ratio is `Planned / ExactMain`; ExactMain is a benchmark-only port built on the exact main revision.

The two ratios answer different questions. The exact-main comparison includes branch-local BTree/coordinator/WAL differences and the distinct transaction contract; the common matrix uses only transactions both accept. Confirmed source differences are described in `branch-divergence.txt`; no single code change is asserted as the cause of any measured gap.

The two requested questions have separate answers:

1. On the experiment HEAD shared infrastructure, planned B-link reached **3.04x** the branch-local `main-btree` mutation throughput geometric mean.
2. Against the exact current-main source, planned B-link reached **3.59x** the ExactMain harness geometric mean.

ExactMain's comparator itself measured lower throughput than experiment `main-btree`: the geometric mean of `E-BTree / ExactMain` across the same 12 scenarios was **1.18x**. This observed baseline difference accounts arithmetically for why the exact-v1 ratio is higher. The revisions have confirmed BTree, coordinator and WAL implementation differences, plus compiled Tokio feature differences: verbose logs show exact main includes `fs`, `io-util`, `net`, and `process`, while experiment does not. The experiment WAL contains grouped frame encoding/write changes. These are observed differences that may contribute, but this run does not isolate their effects. The workload excludes condition-only transactions, so that contract difference is documented for scope and is not a measured performance cause here.

Both comparisons meet the **Very strong** development-checkpoint threshold (GM >= 1.50x). This classification is not a merge/adoption recommendation.

## Per-scenario engine metrics

Throughputs and latencies are medians across three repetitions; throughput brackets show min–max. Errors, overloads, and conflicts are totals across repetitions. WAL bytes/mutation and page images/mutation are total deltas divided by successful mutation operations. `batches/max(syncs,1)` is the existing harness counter; actual WAL sync count is zero in this sync-disabled workload, so a physical transactions-per-sync ratio is undefined.

| writers | width | distribution | engine | mutation ops/s median [min–max] | logical tx/s median | p50 / p95 / p99 µs | CPU one-core % | avg group requests | errors / overloads / conflicts | WAL bytes/mutation | page images/mutation | batches/max(syncs,1) |
|---:|---:|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 16 | 1 | uniform | E-BTree | 10519 [10174–10723] | 10519 | 1471.4 / 1751.9 / 2432.1 | 101.4 | 15.94 | 0 / 0 / 0 | 8380.00 | 2.0000 | 15.94 |
| 16 | 1 | uniform | Planned | 37784 [37021–37974] | 37784 | 391.8 / 703.2 / 896.9 | 104.5 | 11.41 | 0 / 0 / 0 | 4224.00 | 1.0000 | 11.41 |
| 16 | 1 | uniform | ExactMain | 9098 [9006–9208] | 9098 | 1714.8 / 2042.9 / 3052.6 | 101.1 | 15.92 | 0 / 0 / 0 | 8380.00 | 2.0000 | 15.92 |
| 16 | 1 | same-leaf-heavy | E-BTree | 26144 [25130–26432] | 26144 | 605.0 / 690.3 / 935.1 | 102.4 | 15.32 | 0 / 0 / 0 | 8380.00 | 2.0000 | 15.32 |
| 16 | 1 | same-leaf-heavy | Planned | 72963 [72948–73657] | 72963 | 221.3 / 286.2 / 623.3 | 108.6 | 7.75 | 0 / 0 / 0 | 4224.00 | 1.0000 | 7.75 |
| 16 | 1 | same-leaf-heavy | ExactMain | 18967 [18371–19391] | 18967 | 838.5 / 925.6 / 1180.4 | 101.9 | 15.47 | 0 / 0 / 0 | 8380.00 | 2.0000 | 15.47 |
| 16 | 1 | different-leaf-heavy | E-BTree | 18945 [18915–19267] | 18945 | 827.6 / 980.0 / 1109.8 | 101.9 | 15.93 | 0 / 0 / 0 | 8380.00 | 2.0000 | 15.93 |
| 16 | 1 | different-leaf-heavy | Planned | 47413 [47007–47685] | 47413 | 317.4 / 564.8 / 647.8 | 106.9 | 14.11 | 0 / 0 / 0 | 4224.00 | 1.0000 | 14.11 |
| 16 | 1 | different-leaf-heavy | ExactMain | 14624 [14501–14810] | 14624 | 1083.1 / 1260.6 / 1374.8 | 101.4 | 15.90 | 0 / 0 / 0 | 8380.00 | 2.0000 | 15.90 |
| 16 | 16 | uniform | E-BTree | 12938 [12690–13149] | 809 | 19777.6 / 22068.4 / 23350.5 | 101.1 | 15.85 | 0 / 0 / 0 | 4416.38 | 1.0616 | 15.85 |
| 16 | 16 | uniform | Planned | 43817 [43605–43821] | 2739 | 5516.0 / 9984.8 / 11184.1 | 103.4 | 14.00 | 0 / 0 / 0 | 4156.58 | 0.9991 | 14.00 |
| 16 | 16 | uniform | ExactMain | 11822 [10790–11897] | 739 | 21548.8 / 24775.8 / 26403.6 | 101.1 | 15.82 | 0 / 0 / 0 | 4416.23 | 1.0616 | 15.82 |
| 16 | 16 | same-leaf-heavy | E-BTree | 66523 [66460–66944] | 4158 | 3808.6 / 3886.5 / 4075.6 | 103.4 | 15.98 | 0 / 0 / 0 | 783.50 | 0.1875 | 15.98 |
| 16 | 16 | same-leaf-heavy | Planned | 215452 [214655–216099] | 13466 | 1346.6 / 1550.1 / 1770.9 | 112.9 | 10.20 | 0 / 0 / 0 | 523.75 | 0.1250 | 10.20 |
| 16 | 16 | same-leaf-heavy | ExactMain | 61278 [59610–61309] | 3830 | 4168.4 / 4334.1 / 4536.0 | 102.9 | 15.95 | 0 / 0 / 0 | 783.50 | 0.1875 | 15.95 |
| 16 | 16 | different-leaf-heavy | E-BTree | 54934 [54826–55868] | 3433 | 4625.2 / 4845.5 / 4953.2 | 103.4 | 15.96 | 0 / 0 / 0 | 783.50 | 0.1875 | 15.96 |
| 16 | 16 | different-leaf-heavy | Planned | 151132 [135996–155499] | 9446 | 1579.2 / 2940.8 / 3049.7 | 109.9 | 14.08 | 0 / 0 / 0 | 523.75 | 0.1250 | 14.08 |
| 16 | 16 | different-leaf-heavy | ExactMain | 51091 [50705–51504] | 3193 | 5001.8 / 5214.4 / 5382.1 | 102.9 | 15.96 | 0 / 0 / 0 | 783.50 | 0.1875 | 15.96 |
| 64 | 1 | uniform | E-BTree | 10535 [10348–10827] | 10535 | 6055.5 / 6572.2 / 7643.8 | 101.3 | 63.47 | 0 / 0 / 0 | 8380.00 | 2.0000 | 63.47 |
| 64 | 1 | uniform | Planned | 39490 [39090–41121] | 39490 | 1582.6 / 1789.2 / 3067.7 | 104.9 | 61.30 | 0 / 0 / 0 | 4224.00 | 1.0000 | 61.30 |
| 64 | 1 | uniform | ExactMain | 8993 [8883–9212] | 8993 | 7093.8 / 7616.4 / 8086.9 | 101.3 | 63.46 | 0 / 0 / 0 | 8380.00 | 2.0000 | 63.46 |
| 64 | 1 | same-leaf-heavy | E-BTree | 28604 [28410–28991] | 28604 | 2240.6 / 2306.0 / 2539.8 | 102.9 | 63.65 | 0 / 0 / 0 | 8380.00 | 2.0000 | 63.65 |
| 64 | 1 | same-leaf-heavy | Planned | 83801 [83428–90466] | 83801 | 619.9 / 1140.4 / 1206.6 | 110.9 | 45.26 | 0 / 0 / 0 | 4224.00 | 1.0000 | 45.26 |
| 64 | 1 | same-leaf-heavy | ExactMain | 19349 [19300–20476] | 19349 | 3296.3 / 3396.3 / 3548.5 | 101.4 | 63.57 | 0 / 0 / 0 | 8380.00 | 2.0000 | 63.57 |
| 64 | 1 | different-leaf-heavy | E-BTree | 18073 [17377–18227] | 18073 | 3495.3 / 3855.1 / 4132.1 | 102.1 | 63.59 | 0 / 0 / 0 | 8380.00 | 2.0000 | 63.59 |
| 64 | 1 | different-leaf-heavy | Planned | 46292 [45958–46555] | 46292 | 1347.4 / 1452.3 / 2585.0 | 105.4 | 62.37 | 0 / 0 / 0 | 4224.00 | 1.0000 | 62.37 |
| 64 | 1 | different-leaf-heavy | ExactMain | 13503 [13461–14601] | 13503 | 4575.5 / 5076.7 / 5575.2 | 101.2 | 63.39 | 0 / 0 / 0 | 8380.00 | 2.0000 | 63.39 |
| 64 | 16 | uniform | E-BTree | 13379 [13059–13566] | 836 | 77329.1 / 87799.3 / 94861.7 | 100.6 | 62.32 | 0 / 0 / 0 | 4415.84 | 1.0615 | 62.32 |
| 64 | 16 | uniform | Planned | 37702 [37355–38423] | 2356 | 26481.6 / 29858.0 / 51536.7 | 102.3 | 59.10 | 0 / 0 / 0 | 4155.69 | 0.9989 | 59.10 |
| 64 | 16 | uniform | ExactMain | 12446 [12236–12516] | 778 | 82924.8 / 92916.7 / 96080.3 | 101.3 | 61.81 | 0 / 0 / 0 | 4415.77 | 1.0615 | 61.81 |
| 64 | 16 | same-leaf-heavy | E-BTree | 66546 [65845–66862] | 4159 | 15327.1 / 15616.5 / 26649.4 | 102.9 | 63.04 | 0 / 0 / 0 | 783.50 | 0.1875 | 63.04 |
| 64 | 16 | same-leaf-heavy | Planned | 233679 [232358–234549] | 14605 | 3908.7 / 6248.0 / 6416.3 | 115.3 | 45.89 | 0 / 0 / 0 | 523.75 | 0.1250 | 45.89 |
| 64 | 16 | same-leaf-heavy | ExactMain | 60892 [60098–61339] | 3806 | 16780.6 / 17077.5 / 24513.5 | 102.8 | 63.36 | 0 / 0 / 0 | 783.50 | 0.1875 | 63.36 |
| 64 | 16 | different-leaf-heavy | E-BTree | 45187 [44653–45891] | 2824 | 22775.6 / 26758.4 / 28490.3 | 103.1 | 63.27 | 0 / 0 / 0 | 783.50 | 0.1875 | 63.27 |
| 64 | 16 | different-leaf-heavy | Planned | 132070 [129987–132502] | 8254 | 7653.0 / 9165.1 / 13525.7 | 108.8 | 59.24 | 0 / 0 / 0 | 523.75 | 0.1250 | 59.24 |
| 64 | 16 | different-leaf-heavy | ExactMain | 43083 [41515–43088] | 2693 | 24048.4 / 27904.0 / 30656.7 | 103.5 | 63.22 | 0 / 0 / 0 | 783.50 | 0.1875 | 63.22 |
| 1 | 1 | uniform | E-BTree | 7945 [7675–7990] | 7945 | 106.7 / 183.9 / 200.9 | 100.8 | 1.00 | 0 / 0 / 0 | 8380.00 | 2.0000 | 1.00 |
| 1 | 1 | uniform | Planned | 27530 [26810–29859] | 27530 | 34.8 / 41.4 / 59.9 | 101.9 | 1.00 | 0 / 0 / 0 | 4224.00 | 1.0000 | 1.00 |
| 1 | 1 | uniform | ExactMain | 7562 [7511–7929] | 7562 | 109.4 / 187.7 / 204.7 | 101.5 | 1.00 | 0 / 0 / 0 | 8380.00 | 2.0000 | 1.00 |
| 1 | 16 | uniform | E-BTree | 12276 [12086–12277] | 767 | 1204.0 / 1378.3 / 1450.6 | 100.0 | 1.00 | 0 / 0 / 0 | 4416.09 | 1.0616 | 1.00 |
| 1 | 16 | uniform | Planned | 41564 [39800–42170] | 2598 | 345.3 / 404.2 / 467.2 | 98.5 | 1.00 | 0 / 0 / 0 | 4156.25 | 0.9990 | 1.00 |
| 1 | 16 | uniform | ExactMain | 11419 [11297–11422] | 714 | 1304.0 / 1489.4 / 1572.0 | 99.9 | 1.00 | 0 / 0 / 0 | 4416.29 | 1.0616 | 1.00 |

## Paired scenario ratios

| writers | width | distribution | Planned / E-BTree | Planned / ExactMain |
|---:|---:|---|---:|---:|
| 16 | 1 | uniform | 3.59x | 4.15x |
| 16 | 1 | same-leaf-heavy | 2.79x | 3.85x |
| 16 | 1 | different-leaf-heavy | 2.50x | 3.24x |
| 16 | 16 | uniform | 3.39x | 3.71x |
| 16 | 16 | same-leaf-heavy | 3.24x | 3.52x |
| 16 | 16 | different-leaf-heavy | 2.75x | 2.96x |
| 64 | 1 | uniform | 3.75x | 4.39x |
| 64 | 1 | same-leaf-heavy | 2.93x | 4.33x |
| 64 | 1 | different-leaf-heavy | 2.56x | 3.43x |
| 64 | 16 | uniform | 2.82x | 3.03x |
| 64 | 16 | same-leaf-heavy | 3.51x | 3.84x |
| 64 | 16 | different-leaf-heavy | 2.92x | 3.07x |
| 1 | 1 | uniform | 3.46x | 3.64x |
| 1 | 16 | uniform | 3.39x | 3.64x |

## Geometric means

| subset | GM_engine: Planned / E-BTree | GM_exact_v1: Planned / ExactMain |
|---|---:|---:|
| all 12 multiwriter | 3.04x | 3.59x |
| width-1 | 2.98x | 3.87x |
| width-16 | 3.09x | 3.33x |
| uniform | 3.37x | 3.78x |
| same-leaf-heavy | 3.11x | 3.87x |
| different-leaf-heavy | 2.68x | 3.17x |
| writers-16 | 3.02x | 3.55x |
| writers-64 | 3.06x | 3.64x |

## Single-writer controls

| width | distribution | E-BTree mutation ops/s | Planned mutation ops/s | ExactMain mutation ops/s | Planned / E-BTree | Planned / ExactMain |
|---:|---|---:|---:|---:|---:|---:|
| 1 | uniform | 7945 | 27530 | 7562 | 3.46x | 3.64x |
| 16 | uniform | 12276 | 41564 | 11419 | 3.39x | 3.64x |

## Best, worst, and regressions

- same-source: best `(64, 1, 'uniform')` = 3.75x; worst `(16, 1, 'different-leaf-heavy')` = 2.50x.
- same-source: multiwriter regressions below 0.90x: none.
- exact-main: best `(64, 1, 'uniform')` = 4.39x; worst `(16, 16, 'different-leaf-heavy')` = 2.96x.
- exact-main: multiwriter regressions below 0.90x: none.

## Writer scaling

Each cell is median throughput at 64 writers divided by the corresponding 16-writer scenario. Values above one mean throughput increased as writers rose.

| width | distribution | E-BTree scaling | Planned scaling | ExactMain scaling | Planned / E-BTree scaling | Planned / ExactMain scaling |
|---:|---|---:|---:|---:|---:|---:|
| 1 | uniform | 1.00x | 1.05x | 0.99x | 1.04x | 1.06x |
| 1 | same-leaf-heavy | 1.09x | 1.15x | 1.02x | 1.05x | 1.13x |
| 1 | different-leaf-heavy | 0.95x | 0.98x | 0.92x | 1.02x | 1.06x |
| 16 | uniform | 1.03x | 0.86x | 1.05x | 0.83x | 0.82x |
| 16 | same-leaf-heavy | 1.00x | 1.08x | 0.99x | 1.08x | 1.09x |
| 16 | different-leaf-heavy | 0.82x | 0.87x | 0.84x | 1.06x | 1.04x |

Geometric mean scaling: E-BTree 0.98x, Planned 0.99x, ExactMain 0.97x. Relative scaling GMs: Planned/E-BTree 1.01x; Planned/ExactMain 1.03x.

## Latency interpretation

Across the 12 multiwriter scenarios, the geometric mean of planned-to-comparator median latency ratios was p50 **0.317x / 0.268x**, p95 **0.413x / 0.350x**, and p99 **0.476x / 0.414x** for same-source E-BTree / ExactMain respectively. Planned therefore had lower sampled latency in aggregate while sustaining higher throughput. These latency GMs summarize per-scenario medians; individual rows remain the evidence for outliers. Collection delay was zero, while queue scheduling still forms groups.

Writer scaling was almost flat overall: throughput scaling GMs from 16 to 64 writers were 0.98x for E-BTree, 0.99x for Planned, and 0.97x for ExactMain. Planned's relative scaling GMs were only 1.01x vs E-BTree and 1.03x vs ExactMain, so its throughput lead does not imply a broad scaling advantage. For width-16 uniform, planned scaled to 0.86x while E-BTree scaled to 1.03x. The underlying result JSONL preserves the full planned-engine attribution counters.

## Limitations

- Sync is disabled; this is not a real durability benchmark.
- The benchmark uses the OCI root XFS filesystem, not ZFS.
- Main and experiment have different branch-wide transaction contracts. Only unconditional mutation transactions are compared.
- ExactMain uses a benchmark-only port of the latest experiment harness with Blink adapter/import/metrics removal and the `Option<Lsn>` success assertion. It is not a production binary or a committed main-branch change.
- Page-local multiwriter execution is not implemented.
- These results are a development checkpoint, not a merge or adoption decision.

## Artifacts and provenance

- Raw JSONL records and per-invocation stdout logs are in `results/oci-a1-2ocpu-12g-200g/dual-main-vs-planned/raw/`. Their embedded `git_commit` is `unknown` because the launcher ran from `/home/opc`; absolute executable paths and exact build worktree/binary SHA proofs establish source provenance.
- Build logs, source/environment provenance, binary SHA256 proofs, run order, harness patch and equivalence proof are in the same directory.
- OCI backup: `/home/opc/dodb-oci-artifacts-dual-main-vs-planned-93b1e75/`; its manifest was checked against the repository artifact manifest.
- `transactions_per_sync` in raw output follows the harness denominator floor; all actual sync counts are zero here.
