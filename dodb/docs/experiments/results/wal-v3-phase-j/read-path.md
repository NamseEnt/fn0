# J0 Read-Path Results

## Method

The ignored Rust microbenchmark creates an 8,192-key in-memory B-link base and up to four immutable overlays with 512 overlapping sorted entries each. It measures point GET newest hit, oldest hit, tombstone, base hit, and miss at overlay counts 1/2/4; base hit and miss at count 0; Query limits 8/128; and Scan limits 8/128/4096. Each reported latency is the median of three timed repetitions. There was no separate warmup pass.

H1 values use the direct existing B-link `read_state`, `query_state`, and `scan_state` functions over the same pinned base and the same binary. J0 values use the published-view read functions over that pinned base and the overlay slice. This isolates the read-representation cost; it is not a durable write benchmark and does not include generation pin acquisition.

The run was on the supplied OCI A1 host: AArch64 Neoverse-N1, 2 CPUs, 12 GiB RAM, Linux 6.12.0-206.104.4.4.el9uek. The fixture uses `MemoryFile`; `/bench/zfs/db` was mounted and accessible but is not part of this CPU-only read microbenchmark.

## Gate Result

At four segments the worst measured GET ratio was 1.459x, satisfying the 1.50x target. The worst Query/Scan ratio was 1.745x at limit 8. A Scan with one overlay was 1.490x at limit 8, 1.363x at limit 128, and 1.301x at limit 4096. Four overlays were 1.576x at Scan limit 128 and 1.393x at limit 4096. These results are far below the Phase I naive Scan regression.

Maximum allocations per operation were 3 for GET at zero overlays and 3 at four overlays. Query allocations were 38/556 for H1 and 44/556 for J0 at limits 8/128. Scan allocations were 35/520/16402 for H1 and 41/553/16627 for J0 at limits 8/128/4096. The range-read increase comes from bounded cursor/key scratch storage plus decoding keys whose visible state may be suppressed by a tombstone.

The eight-operation small-limit sweep reached 1.75x at four overlays. The requested preferred goal was below 2x, so this passed without reducing the segment cap. Large-limit Scan remained near 1.4x.

## J3 Materialized-State Read Path

J3 materialization publishes a base-only immutable generation after checkpoint durability. Its read dispatch is the same base-only fast path measured at zero overlays above. The J3 correctness and crash tests verify Get, Query, Scan, ordering, revisions, and pinned readers across materialization. A separate read-latency microbenchmark after a durable checkpoint was not run; the zero-overlay timing is the measured reference for that identical read representation, not a post-checkpoint observation.

At the maximum permitted four-segment state, the measured worst GET ratio was 1.459x, Query limit 8 was 1.745x, and Scan limit 8 was 1.710x. Scan ratios at limits 128/4096 were 1.569x/1.442x. This passes the J0 read gate but does not offset J4's durable write throughput regression.

## Full Measurements

`tables.md` is generated from `raw/j0/j0-read-oci-a1.jsonl` by `scripts/summarize_j0.py`. `perf/j0-perf-stat.txt` records whole-process cycles and instructions for the benchmark test binary.
