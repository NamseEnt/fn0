# Phase K OCI Performance Tables

All durable rows below were measured on the designated OCI A1 host with ZFS and real sync. The source SHA for Phase K measurements is `08e82ec484bd62aeb659471a39f317b86cfb488f`; binary SHA256 is `1b65b5f66a64e12251f4a39cef570047cad125371fc2cc29134b24264593f0b6`. Full provenance and unmodified run rows are in `raw/oci-a1-primary/` and `raw/oci-a1-durable/`.

The six-workload matrix measured H1, J, and K from the same release binary and same source commit. H1 selects the retained `parallel-blink` physical path, J selects synchronous logical-overlay materialization, and K selects background materialization. The original Phase I H1 binary could not be built from the old pre-J monorepo snapshot because that source predates current monorepo storage API contracts. The current-source H1 path provides a same-binary comparison; historical Phase J H1 results remain in `dodb/docs/experiments/results/wal-v3-phase-j/tables.md`.

The short primary pause diagnostic is a separate three-repetition paired run. It measured a 61.99% reduction in writer-blocked time, below the required 90%. K reached the hard overlay limit of eight segments, with 320 median backpressure events and 2.58 seconds of backpressure. After this cause was identified, the six-workload matrix was run to complete the H1-based decision gate.

## Six Real-Sync Workloads

Throughput is the median of three 10-second runs after a 2-second warmup. Ratios are medians of paired per-repetition ratios using identical seeds. Latency is median p50/p95/p99 across runs. H1 is the decision baseline.

| Writers / width / distribution | H1 tx/s | J tx/s | K tx/s | J/H1 | K/J | K/H1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 / 1 / uniform | 9046.8 | 1538.2 | 1656.9 | 0.170x | 1.077x | 0.181x |
| 16 / 16 / uniform | 4107.7 | 1440.4 | 1556.0 | 0.360x | 1.080x | 0.383x |
| 64 / 1 / uniform | 27984.6 | 6475.3 | 6379.4 | 0.231x | 0.990x | 0.229x |
| 64 / 16 / uniform | 5815.3 | 4711.1 | 5568.9 | 0.805x | 1.181x | 0.961x |
| 64 / 16 / compact | 15923.5 | 5558.5 | 5608.9 | 0.349x | 0.994x | 0.346x |
| 64 / 16 / spread | 14208.1 | 4835.9 | 5667.4 | 0.340x | 1.152x | 0.390x |

| Writers / width / distribution | H1 p50/p95/p99 us | J p50/p95/p99 us | K p50/p95/p99 us |
| --- | ---: | ---: | ---: |
| 16 / 1 / uniform | 1652/2515/2858 | 16536/21348/23218 | 5900/20684/24415 |
| 16 / 16 / uniform | 3770/5732/7881 | 18060/20474/21877 | 8540/21176/25682 |
| 64 / 1 / uniform | 2323/2755/3897 | 2464/19059/21294 | 7124/20392/26218 |
| 64 / 16 / uniform | 10521/18934/22353 | 6716/31411/33421 | 10805/21247/26953 |
| 64 / 16 / compact | 4054/6257/7781 | 6427/25235/26615 | 10857/20468/24717 |
| 64 / 16 / spread | 5026/6893/10156 | 6632/28949/31087 | 11050/20166/25072 |

| Primary 64 / 16 / uniform metric (median) | H1 | J | K |
| --- | ---: | ---: | ---: |
| writer blocked ns | 0 | 6201071735 | 2450400586 |
| materialization total ns | 0 | 6157329849 | 8377531685 |
| materialization CPU ns | 0 | 0 | 4219084200 |
| data write ns | 0 | 0 | 895813554 |
| data sync ns | 0 | 0 | 2141408274 |
| checkpoint construction ns | 0 | 0 | 14685692 |
| checkpoint sync ns | 0 | 0 | 600992785 |
| publish pause cumulative ns | 0 | 0 | 69460794 |
| backpressure ns | 0 | 0 | 2736767889 |
| backpressure events | 0 | 0 | 327 |
| materializations | 0 | 270 | 360 |
| segments materialized | 0 | 1080 | 1440 |
| B-link data bytes materialized | 0 | 625991680 | 870006784 |
| WAL bytes reclaimed during materialization | 0 | 128308141 | 0 |
| WAL bytes retained at run end | 0 | 171290 | 186817567 |
| peak overlay segments | 0 | 4 | 8 |

The 64-writer width-16 uniform primary case reached K/J 1.181x, below the 1.30x background-materialization target, and K/H1 0.961x, below the 1.20x adoption gate. Width-1 protection failed sharply: K/H1 was 0.229x at 64 writers and 0.181x at 16 writers, versus the required 0.95x. Compact and spread also remained below H1. These results reject adoption.

## Primary Pause Decomposition

The table reports medians from the six-workload primary case. H1 has no overlay materialization; its zero-valued materializer counters mean not applicable. Phase J does not separately report worker CPU or I/O subphase timers. K `publish_pause_nanos_delta` is cumulative over all publications, not a p50 or p99 sample; its mean was about 0.193 ms per publication (69.5 ms / 360). Publication percentile targets cannot be evaluated from the current counters.

| Metric | H1 | J synchronous | K background |
| --- | ---: | ---: | ---: |
| Materialization total | n/a | 6.16 s | 8.38 s |
| Writer blocked | n/a | 6.20 s | 2.45 s |
| Materializer CPU | n/a | not measured | 4.22 s |
| Data write | n/a | not measured | 0.896 s |
| Data sync | n/a | not measured | 2.14 s |
| Checkpoint construction | n/a | not measured | 14.7 ms |
| Checkpoint sync | n/a | not measured | 0.601 s |
| Cumulative publish pause | n/a | not measured | 69.5 ms |
| Backpressure | n/a | none | 2.74 s / 327 events |
| Materializations | n/a | 270 | 360 |
| Segments consumed | n/a | 1,080 | 1,440 |
| Materialized B-link bytes | n/a | 626.0 MB | 870.0 MB |
| WAL bytes appended | 155.4 MB | 128.3 MB | 152.8 MB |
| WAL bytes reclaimed | n/a | 128.3 MB | 0 |
| WAL bytes retained at run end | n/a | 171 KB | 186.8 MB |
| Peak overlay segments | n/a | 4 | 8 |

K averaged 36 materialization jobs/s, consuming 144 overlay segments/s and writing about 87 MB/s of materialized B-link data during this primary interval. It committed about 5,569 transactions/s and 89,349 logical mutations/s. It still hit the hard overlay bound, and its WAL prefix reclaim counter remained zero while the newer suffix kept advancing. The 186.8 MB retained WAL after 10 seconds shows that this implementation did not demonstrate bounded WAL retention under ongoing writes.

## Read Regression

The existing bounded-overlay read microbenchmark ran on OCI from the same source commit with a `MemoryFile` fixture. At four overlays, the maximum J0/H1 ratios were GET 1.407x (oldest hit), Query limit 8 1.750x, and Scan limit 8 1.705x. Phase J recorded 1.459x, 1.745x, and 1.710x. The Phase K read path is effectively unchanged; the small Query increase is 0.005x and Scan improved by 0.005x. Raw rows and binary provenance are in `raw/oci-a1-read/`.

## Sustained and RocksDB Gates

The 120-second sustained run was not started because the required 64-writer width-16 K/H1 >= 1.20x and both width-1 K/H1 >= 0.95x gates failed. The same-session RocksDB rerun was also skipped because those prerequisites did not pass. Phase K therefore has no sustained RSS/WAL stability claim and no new RocksDB comparison.

The old one-second laptop instrumentation smoke remains in `raw/local-smoke/` as a historical artifact only. It is excluded from all Phase K OCI metrics and decisions.
