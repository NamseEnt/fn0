# Phase K Performance Counters

No separate OCI `perf stat` or equivalent hardware-counter capture was performed. The OCI benchmark JSONL records engine-level materializer CPU time, write and sync durations, publication pause, writer blocking, backpressure, and throughput. Those counters and raw runs are preserved under `../raw/oci-a1-primary/` and `../raw/oci-a1-durable/`.
