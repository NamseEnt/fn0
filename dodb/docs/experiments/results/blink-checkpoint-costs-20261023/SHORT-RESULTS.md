# Sixty-second paired comparison

All eighteen measured cells and six separate preflight cells pass source/binary identity, full-window, zero-error, state/reopen, fixed-input-prefix, latency-count, checkpoint and resource gates. These are three rotated 60-second repetitions after ten-second warmup. Preflight is excluded. They authorize the predeclared sustained follow-up, not default promotion.

## Throughput and latency

Values are medians of three per-run values, except the explicitly labeled worst individual maximum. RW80 reports API calls/s and complete workflow latency; E reports successful width-16 write batches/s and batch latency. Maximum RW80 latency is a histogram upper endpoint, while E records an exact maximum.

| Workload / implementation | Throughput/s | Three-run throughput range/s | p99 ms | p99.9 ms | Median maximum ms | Worst maximum ms | Reopen + state oracle ms |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| RW80 previous B-link | 15,854.65 | 15,715.45–16,144.70 | 7.938 | 12.280 | 929.038 | 939.524 | 3,977.57 |
| RW80 candidate B-link | 15,642.64 | 15,505.45–15,845.25 | 8.237 | 46.268 | 161.350 | 168.821 | 2,293.01 |
| RW80 RocksDB contract baseline | 13,327.35 | 13,162.37–13,560.27 | 29.590 | 83.624 | 785.383 | 814.219 | 1,574.47 |
| E previous B-link | 1,472.21 | 1,455.61–1,491.13 | 327.156 | 2,818.572 | 3,000.146 | 3,079.573 | 7,119.57 |
| E candidate B-link | 1,871.43 | 1,800.84–1,884.72 | 134.611 | 303.038 | 333.519 | 463.574 | 3,295.02 |
| E RocksDB contract baseline | 1,703.28 | 1,691.95–1,703.91 | 333.971 | 457.703 | 593.523 | 601.401 | 3,967.12 |

Versus the previous B-link, candidate E throughput improves **27.12%**, E p99.9 falls **89.25%**, and combined reopen/oracle falls **53.72%**. Candidate RW80 throughput changes **−1.34%** and combined reopen/oracle falls **42.35%**. The previous implementation's second-scale E stalls are substantially reduced in these runs.

There is a separate regression: **RW80 workflow p99.9 rises 3.77×**, from 12.28 to 46.27 ms, while its worst maximum falls from 939.52 to 168.82 ms. Reducing rare large pauses does not imply improvement at every percentile. This would fail the earlier experiment's both-workload p99.9 adoption requirement. The current six predeclared follow-up gates pass; that result must not be described as passing the older adoption gate.

E p95 also worsens: previous B-link 48.366 ms, candidate 77.529 ms, and RocksDB 56.001 ms. RW80 p95 is 5.612 / 5.693 / 3.017 ms respectively. Smaller maxima coexist with higher intermediate percentiles; the measurements establish the distribution change, not an isolated per-component causal attribution.

Versus RocksDB, candidate throughput is **+17.37% RW80 / +9.87% E**, and both aggregate p99.9 and maximum are lower. E combined reopen/oracle is **16.94% lower**, but RW80 combined reopen/oracle is **45.64% higher**. Faster RW80 runs also execute more mutations and oracle work; the combined timing cannot isolate pure backend recovery. The E throughput median is just below the separately declared +10% meaningful-advantage threshold; the sustained comparison remains necessary.

## Costs and interpretation

External sampled peak process-group RSS is **1.74× RocksDB for RW80 and 1.49× for E**. Both have the same 3 GiB process-group guard, not equal actual engine memory use. Candidate median sampled peak total directories are 519,143,174 bytes for RW80 and 906,044,855 bytes for E, versus RocksDB's 200,424,373 and 582,548,937 bytes. The candidate E recovery working set stays within the live-image-plus-512-MiB envelope.

E WAL bytes per successful transaction remain essentially unchanged, 8,828.10 → 8,827.25. Amortized checkpoint bytes change 28,102.99 → 23,238.08 and process-requested write bytes 36,931.12 → 32,065.43 per transaction. Allocation/consolidation rules and full-pack size remain unchanged; these counters include final drain and can change with amortization over different completed transaction counts. They do not prove a smaller full checkpoint or physical-device write amplification.

E coordinator preparation plus completion falls from 4.116 to 2.024 seconds per run. Preparation alone changes 2.127 → 1.782 seconds and completion 1.989 → 0.242 seconds; accumulated pack synchronization changes 18.950 → 17.582 seconds. These phases overlap background execution and are not sums of request latency. The three changes are measured together, so no individual throughput attribution is claimed.

Engine source and workload definitions are frozen as described in [reproduction](REPRODUCE.md). The RocksDB reference executes the DODB contract directly through TransactionDB and keeps its existing native scheduling/admission. This is a bounded synthetic service-path comparison, not maximum-RocksDB tuning, a common complete-engine memory budget, or a general B-link/LSM ranking. Legacy/disabled defaults remain unchanged.
