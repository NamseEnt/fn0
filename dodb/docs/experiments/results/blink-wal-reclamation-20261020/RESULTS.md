# B-link WAL Reclamation Results

## Result

The requested four-path, two-workload, three-repetition matrix completed all 24 cells. Every cell reported a completed 30-second measurement, passed its final and reopen validation, stayed below the run stop limits, and matched the other backends' per-client input-prefix fingerprints. The original controller's final equality check incorrectly included each client's total admitted-workflow count, which naturally varies with throughput. That failed comparison and the earlier controller attempts are preserved. `verify_matrix.py` rechecked the existing reports using the input-prefix fingerprints themselves; no performance cell was repeated or replaced.

The result is workload-dependent. With A/rw20, both checkpoint thresholds retained a small throughput lead over minimal-native and substantially lower workflow p99, while both gave up throughput and p99 versus B-link checkpoint-off. With E, both threshold policies were slower than minimal-native and had higher p99. The B-link off path was fastest in both workloads. These 30-second runs are short comparisons, not long-term steady-state evidence, and the experiment imposed no common memory budget.

## Median results

Each entry is the median of three independent runs. Latencies are the median of each run's all-completions histogram percentile. A reports the `workflow` latency category and aggregate API calls/s; E reports unconditional width-16 write-batch latency and write batches/s. Counts and every repetition remain in `oci/matrix-verification-final-v3.json` and the original JSON files under `oci/reports/`.

| Workload | Path | Throughput/s | p50 ms | p95 ms | p99 ms | CPU s | Completed checkpoints/run | Checkpoint wall time/run |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| A | minimal-native | 3,575.6 API calls | 2.06 | 25.82 | 62.62 | 13.55 | n/a | n/a |
| A | B-link off | 5,203.4 API calls | 3.45 | 6.68 | 8.78 | 13.57 | 0 | 0 ms |
| A | B-link 32 MiB | 3,596.6 API calls | 4.43 | 8.22 | 11.02 | 12.44 | 15 / 15 / 15 | 3,937.9 ms |
| A | B-link 128 MiB | 3,968.1 API calls | 4.21 | 8.22 | 10.87 | 12.38 | 3 / 3 / 3 | 1,995.2 ms |
| E | minimal-native | 1,712.5 batches | 31.26 | 55.35 | 325.06 | 26.14 | n/a | n/a |
| E | B-link off | 2,852.6 batches | 19.97 | 36.73 | 43.78 | 24.63 | 0 | 0 ms |
| E | B-link 32 MiB | 1,317.5 batches | 24.67 | 61.96 | 546.31 | 20.71 | 27 / 27 / 26 | 12,726.4 ms |
| E | B-link 128 MiB | 1,361.6 batches | 25.35 | 51.48 | 1,193.28 | 16.41 | 7 / 7 / 7 | 11,549.3 ms |

For A, threshold B-link was 0.6% faster at 32 MiB and 11.0% faster at 128 MiB than minimal-native; p99 was 82.4% and 82.6% lower. Compared with B-link off, throughput was 30.9% and 23.7% lower and p99 was 25.5% and 23.8% higher. For E, threshold B-link was 23.1% and 20.5% slower than minimal-native, with p99 68.1% and 267.1% higher. Compared with B-link off, throughput was 53.8% and 52.3% lower and p99 was 11.5× and 27.3× higher.

The checkpoint duration is the measured wall duration of the full synchronous checkpoint call. It is not a causal attribution of the whole throughput difference: checkpoint policy also changed transaction group counts and queue waiting. B-link mutation-group means and median measured sync counts were:

| Workload | Path | Mutation transactions/group | Commit WAL syncs | Checkpoint WAL syncs |
| --- | --- | ---: | ---: | ---: |
| A | B-link off | 9.74 | 11,819 | 0 |
| A | B-link 32 MiB | 9.67 | 8,295 | 15 |
| A | B-link 128 MiB | 9.67 | 9,164 | 3 |
| E | B-link off | 53.69 | 1,597 | 0 |
| E | B-link 32 MiB | 53.22 | 737 | 27 |
| E | B-link 128 MiB | 52.74 | 775 | 7 |

The requested group limit was not used as the observed group size.

## WAL and resource observations

The policy triggers after a successful mutation group reaches the threshold, so WAL size can exceed it between checks. Across the threshold cells, peak WAL was about 32.1–33.4 MiB at 32 MiB and 128.1–129.2 MiB at 128 MiB. These are observed file sizes, not cumulative disk writes or write amplification. The coordinator's threshold-excess field and per-checkpoint before/after bytes are preserved per run.

Median process-group peak RSS and maximum run-directory size were:

| Workload | Path | Peak process-group RSS MiB | Maximum database-directory size MiB | Peak WAL MiB |
| --- | --- | ---: | ---: | ---: |
| A | minimal-native | 370 | 160 | 0 |
| A | B-link off | 703 | 791 | 629 |
| A | B-link 32 MiB | 390 | 163 | 32.1 |
| A | B-link 128 MiB | 499 | 246 | 128.1 |
| E | minimal-native | 950 | 543 | 0 |
| E | B-link off | 1,491 | 1,692 | 1,337 |
| E | B-link 32 MiB | 1,116 | 389 | 33.4 |
| E | B-link 128 MiB | 1,194 | 485 | 129.2 |

All cells remained under 3 GiB process-group RSS, 4 GiB database-directory size, 1,500 MiB host `MemAvailable`, and 10 GiB filesystem-free stop thresholds. Per-run CPU, initial/final memory, database/WAL samples, latency counts, success/rejection/error counters, sync counts, group histograms, checkpoint events, and manifests are retained with each report. The B-link service sampler records measurement start and end state. E's driver records the total admission-plus-final-completion duration; its separate drain duration is derived by subtracting the requested 30 seconds, and its final coordinator-checkpoint drain is separately recorded.

The A checkpoint event count uses event wall timestamps within the 30-second window bounded by the workload's timestamped process samples. E's process sampler has no wall-clock timestamp; its conservative repeated-checkpoint count is total post-reset completions minus one possible final queued coordinator group during drain. The E workloads have 64 dedicated clients and a 64-request coordinator group, and all threshold cells still have at least six conservatively counted in-window checkpoints.

## Provenance and verification

- Source commit: `876d5a1b4b7c0b2792efc27237fa3367cd8d1d9d`, built from the dedicated experiment branch.
- OCI: `opc@217.142.246.204`, Linux AArch64, `/bench/zfs/db` on ZFS, Rust/Cargo 1.97.1, pinned RocksDB 11.8.1 archive SHA `9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5`.
- `service_workload` SHA-256: `bfcbfd277172bf06f8e5bdab58b01e8589b9d2580b9ee23e5606644d805f0199`.
- `unconditional_write_workload` SHA-256: `aca4e83310fb35d3a4b2f378458e84cbcb6d414bbb3b3dbf597fac4a91c45f55`.
- All per-client input-prefix fingerprints matched across the four paths within each workload.
- All 24 reports passed measurement-window, final-state, reopen-state, report-hash, and latency `count == sample_count` checks. No latency cap or replacement sample was used.
- A threshold cell qualifies for repeated reclamation only when it has at least three completed checkpoints within the measurement window; all 12 threshold workload/repetition cells qualified.
- No conclusion is made about equal-memory performance, external memory limits, a bounded page cache, or steady state beyond the measured 30 seconds.

See `oci/matrix-verification-final-v3.json` for full per-run metrics, `oci/reports/` for raw and enriched reports, commands, resource samples, controller records, database hashes, and preserved RocksDB LOG files, and `VALIDATION.md` for contract, fault-injection, process-termination, and OCI build/test evidence.

## Follow-up memory work

First, run a separate feasibility experiment with an external memory limit to see whether the current engine can operate under that limit. Separately, after an engine change implements a bounded page cache with eviction and reload, measure a dataset larger than available memory. Neither changing `cache_capacity` nor adding checkpoints supplies that cache; neither follow-up ran here.
