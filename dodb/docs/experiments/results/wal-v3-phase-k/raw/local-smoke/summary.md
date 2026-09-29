# Local J/K Smoke Run

This one-repetition, one-second run validates the benchmark modes and instrumentation only. It ran on an Apple M1 Mac with eight logical CPUs, a 512-row working set, and the local filesystem. It is not the requested OCI A1/ZFS diagnostic and cannot satisfy any architecture gate.

| Mode | Successful tx/s | p50 us | p95 us | p99 us | Materializations | Materialization wall ms | Writer-blocked ms | Peak overlays | WAL retained bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Synchronous J | 2,625.1 | 13,098.6 | 49,051.7 | 54,136.0 | 18 | 577.8 | 578.1 | 4 | 171,690 |
| Background K | 3,936.6 | 15,499.0 | 25,663.9 | 26,871.6 | 25 | 771.6 | 0.94 | 8 | 11,002,991 |

The local writer-blocked counter fell by 99.84%. K had higher throughput but a worse p50, and its worker CPU counter is zero on macOS because thread CPU time is currently collected only on Linux. One materialization was still outstanding when the measured window ended. These results are smoke evidence only; use the retained raw JSONL and run order for exact fields.
