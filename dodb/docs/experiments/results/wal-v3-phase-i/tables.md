# Phase I Measurement Tables

All locality metrics below are measured from actual benchmark group plans and routed leaves. They are not inferred from the working-set size. The complete per-scenario details, including request counts and distributions, are in `tables-locality.md`; raw per-group rows are compressed JSONL under `raw/locality/`. The profile and throughput runs are separate from the counter-instrumented locality runs.

## Primary locality and Model A

| Mode / scenario | Groups | Transactions/group mean | Mutations/group mean | Boundaries/group | Distinct leaves/group | Model A removable | Leaves touched by 1 / 2 / 3+ transactions |
|---|---:|---:|---:|---:|---:|---:|---:|
| Disabled, 64w width16 uniform | 2,263 | 42.70 | 683.26 | 682.54 | 646.43 | 5.29% | 94.6% / 5.2% / 0.2% |
| Real, 64w width16 uniform | 1,651 | 43.93 | 702.89 | 702.16 | 666.42 | 5.09% | 94.8% / 5.0% / 0.2% |
| Disabled, 64w width1 uniform | 43,748 | 23.47 | 23.47 | 23.47 | 23.41 | 0.30% | 99.7% / 0.3% / 0.0% |
| Real, 64w width1 uniform | 10,885 | 34.73 | 34.73 | 34.73 | 34.63 | 0.28% | 99.7% / 0.3% / 0.0% |
| Disabled, 64w width16 same-leaf-heavy | 9,106 | 38.21 | 611.36 | 76.42 | 4.59 | 93.99% | 5.7% / 1.0% / 93.2% |
| Real, 64w width16 same-leaf-heavy | 5,342 | 44.30 | 708.74 | 88.59 | 4.86 | 94.51% | 2.0% / 1.7% / 96.3% |
| Disabled, 64w width16 different-leaf-heavy | 6,736 | 40.77 | 652.30 | 81.54 | 81.54 | 0.00% | 100% / 0% / 0% |
| Real, 64w width16 different-leaf-heavy | 3,420 | 43.99 | 703.84 | 87.98 | 87.98 | 0.00% | 100% / 0% / 0% |
| Disabled, 16w width16 uniform | 10,312 | 8.08 | 129.21 | 129.07 | 127.67 | 1.09% | 98.9% / 1.1% / 0.0% |
| Real, 16w width16 uniform | 6,166 | 8.22 | 131.57 | 131.43 | 130.20 | 0.93% | 99.1% / 0.9% / 0.0% |

For the primary uniform case, real-sync group distributions were tx/group mean/p50/p95/max `43.93/44/47/64`, mutations/group `702.89/704/752/1024`, mutations per touched leaf `1.05/1/2/5`, and transactions per touched leaf `1.05/1/2/5`. Failed transactions were zero in these unconditional runs. Average unique keys/group were 700.47. Model A is weak for uniform width16; it only removes about 5.1% of physical boundaries. Same-leaf-heavy controls show that coalescing is workload-specific.

## H1 throughput and current profile

| Mode | Three tx/s repetitions | Median tx/s | Transactions per sync | Approx CPU demand |
|---|---|---:|---:|---:|
| Sync disabled | 6,102 / 6,122 / 6,190 | 6,122 | 32.8–36.7 | 193 us/tx estimated from ~118% one-core utilization |
| Real sync | 4,598 / 4,492 / 4,614 | 4,598 | 35.1–36.8 | WAL sync about 72 us/tx |

The throughput profile was from source SHA `94cfbbe9f8403c0ff2ee0ef063c5d9daa347de90`. The 10-second H1 profile recorded approximately 38.04B `cycles:Pu` over 61,804 successful transactions, about 616k sampled user cycles/tx. Flat `encode_page_delta` symbols sum to about 10.64%; the H1 callgraph's inclusive physical preparation subtree is about 42.78%. See `perf/verified/h1-current-flat.txt` and `h1-current-callgraph.txt`.

## Model ceilings

| Model | Basis | Ideal ceiling | Interpretation |
|---|---|---:|---|
| A: final leaf once per group | 5.09% of real-sync uniform boundaries removable; linear scaling against 42.78% physical subtree | ~1.02x | About 2.2% total CPU reduction; not a uniform workload solution. |
| B: overlap CPU with fsync | 193 us CPU, 72 us sync; serialized 265 us versus `max(193,72)` | 1.37x | Generous ceiling. Measured real/disabled ratio is 0.751 throughput (1.33x), leaving limited headroom. |
| C: durable logical overlay | Remove 42.78% inclusive physical preparation subtree | 1.75x | Ideal CPU ceiling only; keeps logical admission, WAL, sync, overlay publication, condition reads and unrelated work. |

## Benchmark-only overlay prototype

Prototype source SHA is `42e53838e7ebabcd5f48ef1e015e0f71e2758668`; 3 repetitions were run on the same OCI host. It uses a volatile file with no-op sync and does not measure durability or durable throughput. Each write row processes 44 transactions/group × 16 mutations, includes condition/admission, transient overlay mutation, segment freeze/sort and publication; reported median is across three repetitions.

| Prior immutable segments | Write CPU ns/tx | Allocations/tx | Allocated bytes/tx | H1 CPU estimate / prototype |
|---:|---:|---:|---:|---:|
| 0 | 6,025 | 101.27 | 7,445 | 0.031 |
| 1 | 6,461 | 101.30 | 7,445 | 0.034 |
| 2 | 6,545 | 101.30 | 7,446 | 0.034 |
| 4 | 6,479 | 101.30 | 7,447 | 0.034 |
| 8 | 6,389 | 101.30 | 7,450 | 0.033 |
| 16 | 6,459 | 101.30 | 7,456 | 0.034 |
| 32 | 6,420 | 101.30 | 7,467 | 0.033 |

H1 CPU estimate is 193,000 ns/tx. The write CPU ratio is well below 0.70, passing the prototype CPU gate. Correctness smoke covered ordered condition visibility, failed transaction isolation, tombstones, pinned-view immutability, Query, and Scan.

### Point GET latency ratio to H1 base

Median prototype latency divided by base B-link GET latency. Each row includes hit in newest/oldest overlay, base hit, and total miss.

| Segments | Newest hit | Oldest hit | Base hit | Miss |
|---:|---:|---:|---:|---:|
| 1 | 0.195x | 0.201x | 1.251x | 1.109x |
| 2 | 0.180x | 0.343x | 1.330x | 1.273x |
| 4 | 0.192x | 0.639x | 1.591x | 1.512x |
| 8 | 0.200x | 1.398x | 2.184x | 2.166x |
| 16 | 0.184x | 3.207x | 3.899x | 4.081x |
| 32 | 0.192x | 6.615x | 6.673x | 7.119x |

At two segments, old overlay, base and miss remain at or below 1.33x. Four segments slightly exceed the requested 1.50x target for base fallback. A small hard segment limit or prompt compaction is needed.

### Query and Scan latency ratio to H1 base

20 operations per repetition, 100 rows returned per operation. Ratios are medians. The prototype uses a simple full-map merge and over-fetches base rows to account for overlay keys, so these are prototype-model costs rather than a lower bound on a sorted streaming merge.

| Segments | Query | Scan |
|---:|---:|---:|
| 1 | 3.30x | 49.74x |
| 2 | 3.33x | 98.41x |
| 4 | 3.19x | 159.29x |
| 8 | 3.27x | 320.97x |
| 16 | 3.40x | 670.05x |
| 32 | 4.62x | 1,457.12x |

The write gate passes, but read amplification requires a redesigned ordered merge and bounded overlay policy before this can be a strong implementation candidate.
