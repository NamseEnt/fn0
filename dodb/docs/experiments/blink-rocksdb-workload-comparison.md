# Borrowed-Page B-link and RocksDB Workload Comparison

## Scope

This study compares the borrowed-page B-link candidate with the pinned RocksDB
adapter on the OCI product workload profiles. The B-link binary enables
`blink-borrowed-page-views`; mixed cases use the main-parity collection policy
with two B-link workers. Both engines use durable-return writes. The read
comparison is descriptive: the adapters use different reader execution models
and independent point/range key streams, so it does not claim per-request read
trace identity.

The measured source is `81f6ce268a26e6521d62de7cd50797052b408c9b`. The
mixed summary metric correction is in `4bce49836558bfc00bb14fe5717a713cbdb0de49`;
it changes only write-rate interpretation and regenerates summaries from the
preserved raw runs. The correction uses write transactions per second instead
of treating total logical transactions per second as writes. Twenty local
mixed-harness tests passed after the correction.

RocksDB is version 11.8.1 at commit
`abeebd9630f11bd08c28b7bd43c7bdfc62050654`, verified from its version header.
The comparison uses the existing 64 MiB RocksDB block cache and dodb 64 MiB
page-cache configuration. The workload has 16-byte keys, 512-byte values,
uniform distribution, and a 10,000-row resident set. Cache-pressure reads use
250,000 rows. Reads run at c1/c4/c16; mixed and transaction cases run at c4/c16
as specified below.

The OCI host is an AArch64 Neoverse-N1 VM.Standard.A1.Flex with 2 OCPU and 12
GiB RAM. Data files use the same ZFS dataset, `dodbbench/db`, with
`sync=standard`, 4 KiB record size, compression off, and atime off. Dodb uses
real WAL sync. RocksDB uses its synced WAL WriteBatch contract. Read cases are
seeded before timing; mixed/write cases retain the same per-repetition seed
and the paired 1,000-operation logical trace prefix hash across engines.

The measured source was fetched and checked out at the exact pushed SHA. It
remained clean before and after native ARM64 builds and measurements. Build
commands, compiler and host provenance, and binary SHA256 values are in
[`build-proof`](results/dodb-rocksdb-81f6ce268-20261007/build-proof/). Native
binaries remain on the OCI host; their hashes are retained locally.

## Execution and validation

The smoke stage completed 20 executions: eight Get/Query runs at c4/c16 and
twelve mixed runs across 95/5, 50/50, and 20/80 at c4/c16. All 20 passed
semantic and output validation, and every raw record has process RSS.

The full stage completed 104 executions:

- 44 resident Get/Query runs: c1/c4 used three repetitions with a 2 s warmup
  and 5 s measurement; c16 used five repetitions with a 2 s warmup and 10 s
  measurement.
- 12 cache-pressure Get/Query runs at c16, 250,000 rows, three repetitions,
  2 s warmup, and 5 s measurement.
- 36 width-1 mixed runs across three read/write ratios at c4/c16, three
  repetitions, 2 s warmup, and 5 s measurement.
- 12 c16 mixed runs at 95/5 with transaction widths 4 and 8, three
  repetitions, 2 s warmup, and 5 s measurement.

Every matrix validator accepted all planned rows. There were no errors,
conflicts, overloads, or failed verifications. Mixed successful read/write
shares matched the requested ratios. Same-seed mixed trace-prefix hashes
matched between B-link and RocksDB. Process RSS end and high-water marks are
sampled from `/proc` at 20 ms intervals; high-water RSS includes seeding,
warmup, and measurement.

## Results

Mixed throughput ratios are medians of the three same-seed paired
B-link/RocksDB ratios. Latencies and RSS are medians across the three runs.
Write p99 is shown in milliseconds. `HWM` is process high-water RSS in MiB.

| Workload | B-link/RocksDB throughput | Read p99, µs B-link/RocksDB | Write p99, ms B-link/RocksDB | HWM, MiB B-link/RocksDB |
| --- | ---: | ---: | ---: | ---: |
| c4, 95/5, width 1 | 1.362x | 5.68 / 9.44 | 2.248 / 2.600 | 85.1 / 62.0 |
| c4, 50/50, width 1 | 1.480x | 8.72 / 13.72 | 1.976 / 2.445 | 85.1 / 37.4 |
| c4, 20/80, width 1 | 1.455x | 9.20 / 15.76 | 2.025 / 2.807 | 85.1 / 36.3 |
| c16, 95/5, width 1 | 1.252x | 3.44 / 7.00 | 3.307 / 4.349 | 98.7 / 133.3 |
| c16, 50/50, width 1 | 1.427x | 7.00 / 12.00 | 2.773 / 3.205 | 85.2 / 69.5 |
| c16, 20/80, width 1 | 1.350x | 8.00 / 14.92 | 2.803 / 3.067 | 85.2 / 63.7 |
| c16, 95/5, width 4 | 0.932x | 3.56 / 12.84 | 5.688 / 6.212 | 85.4 / 131.5 |
| c16, 95/5, width 8 | 1.004x | 3.84 / 29.64 | 8.873 / 8.588 | 85.5 / 148.6 |

For width 1, B-link throughput leads RocksDB by 25–48% in these tested
resident mixed profiles, with lower read p99 and write p99. Width 4 is 6.8%
behind RocksDB; width 8 is effectively tied on throughput, with a slightly
higher write p99. These are profile-specific results, not a universal ranking.

Pure-read throughput ratios use each engine's median rate and are descriptive,
not paired same-key ratios. P99 values are median latency across repetitions.

| Read workload | Get throughput B-link/RocksDB | Get p99, µs B-link/RocksDB | Query16 throughput B-link/RocksDB | Query16 p99, µs B-link/RocksDB |
| --- | ---: | ---: | ---: | ---: |
| c1, 10,000 rows | 1.408x | 1.36 / 1.92 | 2.145x | 2.76 / 6.12 |
| c4, 10,000 rows | 0.950x | 2.04 / 2.16 | 1.954x | 3.40 / 7.08 |
| c16, 10,000 rows | 0.935x | 2.04 / 2.16 | 2.002x | 3.40 / 7.36 |
| c16, 250,000 rows | 4.085x | 3.80 / 32.60 | 3.319x | 3.56 / 16.96 |

At the 250,000-row cache-pressure point, B-link process HWM is 824.7 MiB for
Get and 825.2 MiB for Query16; RocksDB is 371.5 MiB and 279.5 MiB,
respectively. This peak includes database seeding and is not a steady-state
memory measurement. At 10,000 rows, c16 read HWM is 149.3/763.9 MiB for Get
and 145.6/195.5 MiB for Query16. The varied HWM shows that memory should be
judged per workload phase and needs a separate steady-state sample before a
product memory claim.

## Interpretation

The combined borrowed-page B-link candidate now beats RocksDB on all six
width-1 mixed profiles tested and on Query16 throughput. It remains slightly
behind RocksDB on resident c4/c16 Get and c16 width-4 mixed throughput. Width
8 is near parity. The cache-pressure read runs show a substantial B-link
throughput lead together with a higher measured process peak, so the memory
cost needs investigation before broad adoption.

RocksDB's mixed runner and B-link share the tested operation schedule and
paired logical trace prefix. Get/Query use separate engine key generators and
different reader execution models (B-link reader tasks on two Tokio workers;
RocksDB OS threads). Treat those pure-read cross-engine ratios as workload
references rather than exact-trace comparisons. The benchmark also exercises
unconditional writes; it does not compare dodb conditional/revision
transaction semantics with RocksDB's WriteBatch semantics.

The results strengthen B-link for this workload mix, but do not establish that
it is better for every dodb workload. The next useful gates are a workload
replay with production key skew and transaction widths, Get optimization at
c4/c16, and steady-state plus peak memory profiling for the 250,000-row
working set.

## Artifacts

The complete raw data, logs, per-matrix summaries and manifests are in
[`dodb-rocksdb-81f6ce268-20261007`](results/dodb-rocksdb-81f6ce268-20261007/).
The top-level `study-artifact-sha256.txt` covers the local artifact set;
matrix-level `artifact-sha256.txt` files cover each accepted run set. The
binary SHA256 values and build provenance are recorded without copying the
large native executables into the repository.
