# Short comparison

This is the three-repetition, 60-second comparison. It is separate from the 300-second sustained comparison and is not a default-promotion result. All 24 cells pass functional, completion, identity, input-prefix, latency-count, resource and checkpoint evidence guards.

The control source is `cc80d1fdc7a98792be1a739056a3013debf39268`; the candidate is `3b94ee88eb4e7c02416cde14d3ac48c55e45b991`. Workloads, synchronous WAL, conditions/revisions, admission/grouping and checkpoint settings are unchanged.

Values below are medians of per-run values. RW80 throughput is API calls/s; E throughput is successful width-16 batches/s. RSS is the sampled whole-process-group peak; reopen includes the full state oracle.

| Workload / implementation | Throughput/s | p95 ms | p99.9 ms | Peak RSS MiB | Reopen + oracle ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| RW80 blink-control | 16,072.81 | 5.521 | 44.106 | 741.02 | 2,627.14 |
| RW80 blink-candidate | 16,726.29 | 5.325 | 41.189 | 800.70 | 2,915.95 |
| RW80 rocksdb-control | 13,455.20 | 2.959 | 83.558 | 423.22 | 1,518.40 |
| RW80 rocksdb-candidate | 14,084.72 | 2.820 | 80.937 | 387.78 | 1,727.52 |
| E blink-control | 1,903.65 | 76.481 | 299.106 | 1,486.03 | 3,509.28 |
| E blink-candidate | 1,875.53 | 76.743 | 304.087 | 1,688.02 | 3,332.89 |
| E rocksdb-control | 1,710.98 | 55.411 | 425.984 | 963.32 | 1,684.69 |
| E rocksdb-candidate | 2,301.08 | 44.171 | 449.577 | 949.60 | 2,647.00 |

## Old/new changes

B-link RW80 throughput rises **4.07%**. Ordinary Get p95 falls **44.64%**, but ordinary Get p99 rises **11.57%** and Query16 p99 rises **41.26%**. The predeclared target of +5% RW80 throughput or -5% Get p99 therefore fails. RW80 sampled peak RSS rises 8.05%, directory peak rises 6.64%, and combined reopen/oracle rises 10.99%. E throughput changes -1.48%, while sampled RSS rises 13.59%. These observations do not identify a retained-heap cause.

Minimal-native RocksDB E throughput rises **34.49%** and E p95 falls **20.28%**. Its E median maximum rises 22.11%, histogram p99.9 rises 5.54%, and combined reopen/oracle rises 57.12%. RW80 throughput rises 4.68%, ordinary Get p99 falls 19.97%, and sampled RSS falls 8.38%, while combined reopen/oracle rises 13.77%. The throughput target passes, but the joint screening gates fail.

The RocksDB change applies to `MinimalNativeTenant`, the direct DODB-contract adapter used in this comparison. The other `RocksDbService` adapter is unchanged. Mutation-only keys retain sorted exclusive locking and native conflict validation but no longer fetch or validate an old value payload. Condition keys and revision metadata retain their existing value-read paths.

## Improved backends compared

| Workload | B-link throughput change vs improved RocksDB | B-link p95 ratio | B-link p99.9 ratio | B-link RSS ratio | B-link reopen/oracle ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| RW80 | +18.75% | 1.89x | 0.51x | 2.06x | 1.69x |
| E | -18.49% | 1.74x | 0.68x | 1.78x | 1.26x |

In these short cells, B-link retains a mixed-throughput and upper-tail advantage, while improved RocksDB has higher unconditional-write throughput and lower p95 and resource costs. This is a comparison of these contract-preserving implementations, not an isolated tree-versus-LSM or maximum-RocksDB experiment.

Both sets of joint screening gates fail. The predeclared sustained comparison proceeds because all correctness/resource hard guards pass and the longer run can characterize the observed trade-offs; it does not authorize default promotion. All per-run ranges, API percentiles, paired throughput ratios and gate booleans are preserved in [the original summary](evidence/short-summary.json).
