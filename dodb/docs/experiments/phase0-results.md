# Phase 0 Baseline Benchmark Results

> Historical measurement notice: the numbers in this document were measured
> on the previous machine and are preserved for historical context only. They
> are not the current performance baseline. The current baseline is the new
> machine rebaseline in [`rebaseline-results.md`](rebaseline-results.md),
> with raw artifacts under
> `docs/experiments/results/rebaseline-macbookpro17-1-macos26.4-8c8t/`.
> The correctness results and storage semantics described here are not
> invalidated by the machine change.

This is the executed Phase 0 record for
[`b-link-batched-engine.md`](b-link-batched-engine.md). It measures the
current `main` B+Tree through the new sustained benchmark harness. It does not
measure or implement B-link, page versions, parallel page workers, or any
other experimental-engine feature.

The historical short-run `phase4-bench` figures remain compatibility
reference material only; none of them is combined with the sustained numbers
below.

## Scope and identity

| Item | Value |
| --- | --- |
| Baseline storage commit | `1ff96e1b3d205074d4c1b820f5f2680bd3226a8b` |
| Experiment branch | `experiment/b-link-batched-engine` |
| Harness/source commit used for these runs | `97d8b060732dec6231c7ba0c7bc928a8bc33dd27` |
| Engine identifier | `main-btree` |
| Build | `cargo build --release`, release profile |
| Runtime | Tokio multi-thread, 12 workers |
| Seed base | `0xd0db2026` |
| Warmup | 1 s per repetition |
| Timed interval | 2 s per repetition |
| Repetitions | 3; tables report median and raw JSONL retains every repetition |
| Key/value shape | 16-byte key, 64-byte inline value |
| Cache/working set | 256-page cache, 4,096 pre-seeded rows |
| Group limits | 64 requests, 4 MiB, queue capacity 256 |

The release binary was run with `TMPDIR=target/phase0/tmp` because the shared
machine `/tmp` tmpfs was full from unrelated files. This changes only the
benchmark file location, not the storage implementation or durability mode.

## Machine and durability conditions

- AMD Ryzen 5 5600 6-Core Processor; 6 physical cores and 12 logical CPUs.
- Debian GNU/Linux 13 (trixie), Linux `6.12.57-deb13-amd64`.
- Rust `rustc 1.98.1 (48a229cea 2026-09-01)`, release build.
- No process affinity or governor change was applied; this is therefore a
  machine-local baseline, not a portable absolute performance claim.
- Core write/read/mixed runs use real `ProductionFile` files and real
  `sync_data`.
- Collection-delay sweep uses injected sync delay `0` to isolate coordinator
  collection from filesystem durability.
- Sync sweep uses `BenchFile` with injected delays. Its results are not real
  filesystem-sync results. A separate real-sync width-1, 64-writer control is
  included below.

## Harness and raw artifacts

Build and help:

```text
cargo build --release -p dodb-storage --bin phase0-bench
target/release/phase0-bench --help
```

The sustained lifecycle is setup/preseed, warmup, timed measurement, adapter
shutdown/cleanup, and repetition. Every run is a fresh database with a
deterministic scenario/repetition seed. The same benchmark-local
`EngineAdapter` boundary will allow a future experimental engine to reuse the
generator, worker pools, latency collector, and JSONL schema.

The checked-in raw JSONL artifacts are:

```text
docs/experiments/results/phase0-baseline/write-core.jsonl
docs/experiments/results/phase0-baseline/read-core.jsonl
docs/experiments/results/phase0-baseline/mixed-core.jsonl
docs/experiments/results/phase0-baseline/delay-sweep.jsonl
docs/experiments/results/phase0-baseline/sync-sweep.jsonl
docs/experiments/results/phase0-baseline/sync-real-width1-w64.jsonl
```

Each line includes commit, machine, runtime, seed, workload parameters,
attempted/successful counts, conflict/overload/error counts, operation rates,
latency percentiles, CPU utilization, group metrics, coordinator/storage
timing totals, WAL bytes/page images, append/sync totals, and transactions per
sync. All official samples have `errors=0`, `overloads=0`, and
`attempted_operations == successful_operations`.

The exact commands used were:

```text
TMPDIR=/home/namse/dodb-link/target/phase0/tmp target/release/phase0-bench \
  --suite write --duration 2s --warmup 1s --repetitions 3 \
  --cache-capacity 256 --working-set 4096 --key-size 16 --value-size 64 \
  --group-limit 64 --group-bytes 4194304 --queue-capacity 256 \
  --sync-mode real --tokio-workers 12 --seed 0xd0db2026 \
  --output target/phase0/write-core.jsonl

TMPDIR=/home/namse/dodb-link/target/phase0/tmp target/release/phase0-bench \
  --suite read --duration 2s --warmup 1s --repetitions 3 \
  --cache-capacity 256 --working-set 4096 --key-size 16 --value-size 64 \
  --read-limit 16 --sync-mode real --tokio-workers 12 --seed 0xd0db2026 \
  --output target/phase0/read-core.jsonl

TMPDIR=/home/namse/dodb-link/target/phase0/tmp target/release/phase0-bench \
  --suite mixed --duration 2s --warmup 1s --repetitions 3 \
  --cache-capacity 256 --working-set 4096 --key-size 16 --value-size 64 \
  --read-limit 16 --group-limit 64 --group-bytes 4194304 \
  --queue-capacity 256 --sync-mode real --tokio-workers 12 \
  --seed 0xd0db2026 --output target/phase0/mixed-core.jsonl

TMPDIR=/home/namse/dodb-link/target/phase0/tmp target/release/phase0-bench \
  --suite delay-sweep --duration 2s --warmup 1s --repetitions 3 \
  --cache-capacity 256 --working-set 4096 --key-size 16 --value-size 64 \
  --read-limit 16 --group-limit 64 --group-bytes 4194304 \
  --queue-capacity 256 --sync-mode injected --sync-delay 0 \
  --tokio-workers 12 --seed 0xd0db2026 --output target/phase0/delay-sweep.jsonl

TMPDIR=/home/namse/dodb-link/target/phase0/tmp target/release/phase0-bench \
  --suite sync-sweep --duration 2s --warmup 1s --repetitions 3 \
  --cache-capacity 256 --working-set 4096 --key-size 16 --value-size 64 \
  --group-limit 64 --group-bytes 4194304 --queue-capacity 256 \
  --tokio-workers 12 --seed 0xd0db2026 --output target/phase0/sync-sweep.jsonl

TMPDIR=/home/namse/dodb-link/target/phase0/tmp target/release/phase0-bench \
  --suite write --writers 64 --widths 1 --distributions uniform \
  --duration 2s --warmup 1s --repetitions 3 --cache-capacity 256 \
  --working-set 4096 --key-size 16 --value-size 64 --group-limit 64 \
  --group-bytes 4194304 --queue-capacity 256 --sync-mode real \
  --tokio-workers 12 --seed 0xd0db2026 \
  --output target/phase0/sync-real-width1-w64.jsonl
```

The CLI also supports `sequential`, `hotspot`, `same-leaf-heavy`, and
`different-leaf-heavy`; `--widths`, `--writers`, `--readers`, `--read-kinds`,
`--mixes`, cache/working-set sizes, collection delay, sync mode/delay,
transaction mode, and Tokio worker count are independent controls.

## Workload definitions used

- Write scaling: writers `1/4/16/32/64/128`, widths `1/16`, and
  `uniform/same-leaf-heavy/different-leaf-heavy`.
- Read scaling: readers `1/4/16/32/64/128`, separately for `Get`, short
  `Query`, and short `Scan`; Query/Scan limit is 16.
- Mixed: `(readers,writers)` in `(16,16)`, `(64,16)`, `(16,64)`, `(64,64)`;
  both 95/5 read-heavy and 50/50 balanced mixes. The quota is enforced by
  independent reader and writer pools, not by a single random client pool.
- Same-leaf-heavy uses one fixed primary-key range and compact sort-key range;
  different-leaf-heavy varies primary keys so pre-seeded keys are distributed
  across the tree. This is reproducible key-range locality, not runtime leaf
  inspection; actual leaf concentration is a limitation noted below.
- The generator supports transaction widths `1/4/16/25` and conditional
  `insert-if-absent`, although the core matrix uses unconditional PUT updates
  at widths 1 and 16.
- Delay sweep compares `0/50/100/250/500/1000/2000/5000 us` for a 64-writer
  write control and a 64-writer/16-reader balanced mixed control.
- Sync sweep compares injected `0/100/1000/5000/10000 us`; the separate real
  control uses actual production file synchronization.

## Write scaling: real sync, uniform, width 1

Values are median across three repetitions; `p99` is end-to-end write request
latency and `group` is actual requests per coordinator group.

| Writers | logical tx/s | p50 (us) | p95 (us) | p99 (us) | group |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 159 | 5,912 | 8,745 | 15,679 | 1.00 |
| 4 | 552 | 6,263 | 12,664 | 15,101 | 3.52 |
| 16 | 1,670 | 8,983 | 13,262 | 57,254 | 15.35 |
| 32 | 2,944 | 10,775 | 13,546 | 16,828 | 31.60 |
| 64 | 4,567 | 13,868 | 16,888 | 21,823 | 63.31 |
| 128 | 4,637 | 26,403 | 31,431 | 38,561 | 64.00 |

At width 1, throughput rises through 64 writers and is effectively at a
plateau at 128 (1.5% over the 64-writer median), while latency continues to
increase. This is the principal multi-writer comparison baseline.

For width 16, logical tx/s medians are `126 / 306 / 604 / 812 / 647 / 667`
for writers `1 / 4 / 16 / 32 / 64 / 128`; mutation ops/s are approximately
`2,019 / 4,902 / 9,664 / 12,989 / 10,357 / 10,677`. The transaction rate
peaks near 32 writers and then regresses; wider transactions expose the
single preparation/publication path more clearly.

## Read scaling: coordinator-free current read path

Values are median requests/s across three repetitions. Query/Scan also report
returned rows/s; both use limit 16.

| Readers | Get req/s | Query req/s | Query rows/s | Scan req/s | Scan rows/s |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 2.01 M | 0.479 M | 7.66 M | 0.469 M | 7.50 M |
| 4 | 5.23 M | 1.29 M | 20.68 M | 1.43 M | 22.86 M |
| 16 | 8.33 M | 2.06 M | 32.95 M | 2.21 M | 35.28 M |
| 32 | 8.60 M | 1.99 M | 31.88 M | 2.16 M | 34.55 M |
| 64 | 6.87 M | 1.92 M | 30.71 M | 2.09 M | 33.44 M |
| 128 | 6.86 M | 1.96 M | 31.39 M | 2.14 M | 34.23 M |

Read-only runs have zero coordinator groups and zero queued mutation requests.
Get p99 medians range from 0.50 to 1.40 us; Query from 1.79 to 3.64 us; and
Scan from 1.95 to 3.29 us. The current read path therefore does execute
outside the write coordinator and scales materially to 16/32 readers, but
plateaus or regresses at 64/128 on this 12-logical-CPU machine.

## Mixed workload behavior

| Readers | Writers | Mix | tx/s | read/s | aggregate ops/s | p95 (us) | p99 (us) | group |
| ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 | 16 | 95/5 | 1,408 | 26,791 | 28,200 | 5,300 | 15,182 | 11.40 |
| 64 | 16 | 95/5 | 1,269 | 24,143 | 25,412 | 3,847 | 14,474 | 12.53 |
| 16 | 64 | 95/5 | 2,923 | 55,550 | 58,473 | 6,204 | 25,833 | 36.01 |
| 64 | 64 | 95/5 | 2,798 | 53,187 | 55,986 | 6,700 | 25,623 | 40.19 |
| 16 | 16 | 50/50 | 1,554 | 1,570 | 3,124 | 14,860 | 18,082 | 14.11 |
| 64 | 16 | 50/50 | 3,789 | 3,811 | 7,600 | 19,981 | 26,518 | 59.79 |
| 16 | 64 | 50/50 | 1,501 | 1,515 | 3,015 | 13,165 | 56,211 | 15.53 |
| 64 | 64 | 50/50 | 3,835 | 3,854 | 7,689 | 19,784 | 26,872 | 62.68 |

The 95/5 cases retain high read request rate because reads bypass the
coordinator, but writer load raises read tail latency substantially. Balanced
cases expose the single write path most clearly: p95 is about 13--20 ms and
p99 reaches 56 ms in the 16-reader/64-writer case.

## Collection-delay sweep

The sweep uses injected sync delay `0` and 64 writers (write control) or
64 writers plus 16 readers (balanced mixed control). Values are medians across
three repetitions.

| Intentional delay | Write tx/s | Write group | Write p99 us | Mixed tx/s | Mixed group |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0 us | 4,406 | 63.23 | 21,444 | 3,625 | 62.03 |
| 50 us | 4,541 | 64.00 | 19,026 | 3,588 | 61.31 |
| 100 us | 4,191 | 64.00 | 18,797 | 3,583 | 60.47 |
| 250 us | 4,828 | 64.00 | 20,494 | 3,422 | 61.70 |
| 500 us | 5,927 | 64.00 | 13,652 | 3,957 | 61.64 |
| 1 ms | 5,456 | 64.00 | 16,317 | 3,837 | 61.02 |
| 2 ms | 5,671 | 64.00 | 16,200 | 3,936 | 62.27 |
| 5 ms | 5,544 | 64.00 | 14,637 | 3,740 | 63.36 |

With 64 writers the queue is already full enough to form near-maximum groups
at delay 0. The noisy result does not justify a default above the proposed
1 ms; 500 us--2 ms is a tuning region to revisit after an experimental
planner exists. The mixed control's p99 medians are 26.6--30.4 ms across the
sweep, so collection delay is not the only source of tail latency.

## Sync-delay sensitivity

These rows use injected `BenchFile` delay and are not real fsync results.

| Injected sync delay | tx/s | p50 (us) | p95 (us) | p99 (us) | group |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 0 us | 5,398 | 11,929 | 14,035 | 17,871 | 63.95 |
| 100 us | 5,219 | 12,217 | 15,615 | 18,544 | 63.38 |
| 1 ms | 4,463 | 13,933 | 16,425 | 23,564 | 63.51 |
| 5 ms | 3,445 | 18,340 | 20,581 | 27,645 | 63.18 |
| 10 ms | 2,506 | 25,397 | 27,522 | 58,409 | 63.78 |

The separate real-filesystem, 64-writer, width-1 control has median 4,271
tx/s, p50 11,541 us, p95 16,675 us, p99 94,092 us, and 63.15 transactions per
sync. Real durability has a much heavier and more variable tail than the
synthetic sleep control. It must remain a separate comparison dimension.

## Component metrics and latency attribution

For the real-sync, 64-writer, uniform, width-1 write row, the median
end-to-end values are p50 13.868 ms, p95 16.888 ms, and p99 21.823 ms. The
corresponding cumulative metrics, normalized by successful transaction or
group where appropriate, are approximately:

| Existing metric | Normalized value |
| --- | ---: |
| queue wait | 174 us/request |
| collection | 98 us/group |
| processing | 216 us/transaction |
| validation | 0.23 us/transaction |
| B-tree preparation | 13.1 us/transaction |
| publication | 3.3 us/transaction |
| WAL append | 22.6 us/transaction |
| WAL sync | 169 us/transaction |
| WAL bytes | 8,380 bytes/transaction |
| page images | 2/transaction |
| transactions/sync | 63.31 |

These are averages derived from existing cumulative counters, not per-request
percentiles. The harness intentionally does not add timestamp calls to the
production hot path. Consequently, it reports exact end-to-end request
percentiles and cumulative queue/collection/validation/preparation/publication
and WAL components, but cannot claim exact per-request pre-durable, durable,
or publication percentile boundaries. This is a known Phase 0 limitation and
must be addressed with low-overhead common instrumentation before the final
experimental-vs-main attribution claim.

## Phase 0 correctness gate

Passed:

```text
cargo test --workspace
```

This includes the existing storage, WAL/recovery, ReferenceDb differential,
and QUIC tests plus five benchmark-local tests:

- same seed produces the same transaction sequence;
- transaction width and no-duplicate fallback behavior are preserved;
- same-leaf and different-leaf generators have distinct primary-key shapes;
- duration parsing accepts the documented units;
- JSONL escaping remains valid for control characters.

All official samples completed without errors, overloads, or failed requests.
No storage API, transaction semantics, WAL format, durability point, or
production timing instrumentation was changed. The workspace Tokio dependency
only gained the `rt-multi-thread` feature needed by the benchmark binary.

## Observed baseline bottlenecks and Phase 1 implications

1. The write coordinator and synchronous B-tree preparation/publication path
   remain the primary physical serialization point. Grouping amortizes WAL
   sync, but it does not make independent page mutations execute concurrently.
2. At high writer counts, groups reach the 64-request cap while transaction
   latency continues upward. A future engine must compare page-local execution
   against this already-effective group-commit baseline, not against one WAL
   sync per client.
3. Read operations genuinely bypass the coordinator in the current API. The
   baseline read scaling is already multicore up to the machine's useful CPU
   range, so Phase 1/2 must preserve this property and prove whether page
   versioning improves the 64/128-reader plateau without harming committed-view
   correctness.
4. The 95/5 mixed rows show that coordinator-free reads can maintain high
   request rate, but writer activity still increases read tail latency. This
   makes reader interference and page/cache contention important experimental
   metrics.
5. Real sync p99 is far beyond the 9 ms design target while injected 0--1 ms
   controls are much lower. Final adoption must report engine/tree/scheduler
   latency separately from actual durable latency and must not treat synthetic
   sync rows as real durability.
6. The initial 1 ms intentional collection delay remains a reasonable default
   candidate, but the sweep is too noisy to select a final value. Adaptive
   collection and batch-aware execution should be benchmarked only after
   correctness-preserving serial B-link behavior exists.

The largest unresolved measurement gap is per-request attribution. The next
design phase should keep the benchmark schema stable while adding only
experimental-engine-local counters or carefully scoped low-overhead common
instrumentation needed to separate traversal, page mutation, WAL append, sync,
and publication percentiles.

## Adoption decision at Phase 0

There is no experimental engine yet, so Phase 0 makes no adoption decision.
It freezes the comparison baseline. The future engine must beat these numbers
under identical machine, build, seed, working-set, WAL, sync, and repetition
conditions, pass every semantic/durability gate, and satisfy the adoption
criteria in Section 19 of the source-of-truth design document. A small local
win or a synthetic-sync-only win is insufficient.
