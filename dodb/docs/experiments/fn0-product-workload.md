# fn0 Product Workload and Benchmark Policy for dodb

Status: This document is the source of truth for evaluating dodb B-link engine
changes against fn0's general-purpose transactional Document/KV workload. It
was established from experiment branch base
`73171b64091b4f40a68fb986b49a9c115570b282` and `main` base
`d5e41bb823b55d68bad6ab5ba73b0b5113223d8b`.

## 1. Product design center

dodb is fn0's general-purpose transactional operational database for web,
mobile, and lightweight game backends. Its design center is not analytics or
large sequential ingestion. The target is low-latency point operations and
small transactions, useful short-range queries, and predictable behavior under
concurrent and bursty traffic.

The expected workload envelope includes:

- Frequent point `Get`, `Put`, and `Delete` operations.
- Short partition, prefix, and range queries. `Scan` is supported, but is not
  the primary hot path.
- Many width-1 transactions, common widths 2-4, normal support through width 8,
  and width 16 or greater as stress/large-transaction cases.
- Values from hundreds of bytes through a few KiB, as well as focused overflow
  page coverage for larger documents.
- Read-heavy, balanced, and write-heavy tenants; uniform access, hotspots, and
  locality.
- Sudden FaaS concurrency bursts in addition to steady traffic.
- Durable commits whose successful response means the configured durability
  boundary has been reached.

Throughput, p95/p99 latency, overload behavior, and backpressure are all
product concerns. A write-throughput gain alone does not establish a product
improvement.

## 2. Benchmark layers

### Layer A: single-tenant engine benchmark

Run each workload family in its own database. Examples include read-heavy,
ordinary CRUD, write-heavy, transaction-heavy, query-heavy, and hotspot
tenants. Use Layer A as the primary evidence for B-link data-structure,
scheduling, and write-path decisions.

### Layer B: multi-tenant system benchmark

Run this separately after the single-tenant engine passes its product
workloads. Multiple tenant files share process and host CPU, memory, page
cache, storage, and I/O queues. Evaluate at least:

| Case | Workload | Main observations |
| --- | --- | --- |
| MT1 | 100 and 1,000 mostly idle tenant databases, with a small active subset | Memory, open files, per-tenant metadata, startup and reopen cost |
| MT2 | Concurrent read-heavy, balanced, write-heavy, and query-heavy tenants | Aggregate throughput, per-tenant p99, fairness |
| MT3 | N normal tenants plus one saturated write-heavy tenant | Normal-tenant latency and throughput degradation |
| MT4 | One bursty tenant plus quiet tenants | Quiet-tenant p99 impact during and after the burst |
| MT5 | Multiple durable-write-heavy tenants | Shared sync and I/O queue behavior, tail latency |

Layer B also records noisy-neighbor effects and resource use. It assesses
dodb/fn0 resource isolation and scheduling, and must not be folded into the
Layer A engine score.

## 3. Single-tenant product workload suite

Use each workload family in an independent database. The values below define
the core product suite; additional combinations may be added as focused
diagnostics.

### W1: point CRUD and read-heavy

- 95% reads / 5% writes, primarily point `Get`.
- Transaction width 1.
- Run uniform and hotspot key distributions separately.
- Evaluate reader scaling and read p99 while writes are active.

This protects the common point-read path from regressions introduced by write
optimization.

### W2: general OLTP

- 70/30 and/or 50/50 read/write.
- Point `Get`, `Put`, and `Delete` operations.
- Width 1 and widths 2-4.
- Include unconditional writes and conditional/revision-based writes.

This represents profiles, application state, game state, small counters, and
ordinary mutable CRUD.

### W3: write-heavy transactional

- 20/80 read/write and a separate 100% write diagnostic.
- Widths 1, 4, and 8; width 16 is secondary stress.
- Uniform, different-leaf, and hotspot distributions.
- Attribute batching, group commit, WAL cost, parallel leaf execution, and
  coordinator/admission limits.

The combination of 100% writes, 64 writers, and width 16 is an architecture
stress case, not a representative product workload.

### W4: small multi-key transaction

Primary widths are 1, 2, 4, and 8. Width 16 is stress. Cover unconditional
mutations, insert-if-absent, `RevisionEquals`, read/condition-dependent writes,
same-key contention, disjoint keys, same-leaf keys, and multi-leaf keys. Report
conflicts and retries alongside successful transaction throughput.

### W5: short Query

Measure Query independently from Get and Scan. Use limits 1, 8, 16, and 64;
limit 16 is the primary short-query case. Record requests/s, returned rows/s,
p50/p95/p99, and traversal/materialization cost.

### W6: Scan and long range

Retain Scan as a supported-feature regression and stress suite, with limits 128
and 1,024. Keep its results separate from Query. Do not trade point Get or
small-transaction performance for higher Scan throughput.

### W7: hotspot and contention

Concentrate traffic on selected keys or ranges. Record throughput, conflicts,
queue wait, p99, retries/restarts, page-latch contention, and fairness. A
different-leaf-only result is insufficient to characterize product behavior.

### W8: sequential growth and split

Measure sequential inserts and right-edge growth, including leaf/internal/root
splits, structural retries, write amplification, and p99 stalls. This is a
structural robustness suite rather than the primary product mix.

### W9: burst

Measure separately from steady-state saturation. Offer a normal load, step up
to a 5x-10x load, sustain the burst, then return to normal. Record peak queue
depth, overload/rejection, p95/p99, throughput during the burst, time to
recover, post-burst latency, memory growth, and backpressure activation.

Closed-loop worker saturation and a high writer count are not substitutes for
an open-loop offered-load burst test.

## 4. Data shape and cache regimes

Keep the existing 16-byte key / 64-byte value case as a diagnostic
microbenchmark for storage-engine CPU and scheduling overhead. Product
conclusions must also cover these value classes, adjusted where necessary to
match actual page and overflow representation:

| Class | Value size | Purpose |
| --- | ---: | --- |
| Tiny diagnostic | 64 B | Isolate engine overhead |
| Small primary | 512 B | Ordinary inline documents |
| Medium | 2 KiB | Meaningful page pressure |
| Overflow | 8 KiB | Exercise overflow pages |
| Large focused case | 32 KiB | Focused large-document behavior |

The matrix must include inline small documents, medium values that create page
pressure, and values that use overflow pages.

Run every primary workload in two explicitly labeled regimes:

- **Resident:** working set fits in cache; isolates tree, synchronization,
  scheduling, and WAL/commit CPU costs.
- **Non-resident / larger than cache:** working set substantially exceeds the
  configured cache; includes page fetch, cache, traversal, and I/O interaction.

Do not infer production performance from resident results alone. Report cache
capacity, working-set size, and database/page counts.

## 5. Concurrency and offered load

The product core concurrency matrix is 1, 4, and 16 concurrent clients.
Scaling and stress levels are 32, 64, and 128. Width-1 and low/moderate
concurrency performance are protected product paths; behavior at 64/128
writers does not excuse their regression.

For W9, use an open-loop or otherwise explicitly rate-controlled offered-load
driver so queue growth and overload can be observed. Record the offered rate,
achieved rate, queue depth, and rejection/error behavior. Keep this distinct
from closed-loop saturation measurements.

## 6. Benchmark result classes

Label every result as exactly one of these evidence classes:

1. **Product workload benchmark:** primary adoption evidence, following W1-W9
   and the product core matrix.
2. **Architecture stress benchmark:** tests specific mechanisms and limits,
   such as 64 writers at width 16 on different leaves, same-leaf contention,
   injected sync delay, no-sync, maximum batches, or forced splits.
3. **Attribution microbenchmark:** identifies component costs such as CRC,
   page/WAL encoding, planner clone, materialization, routing, or fsync control.

Architecture stress and attribution results are valuable engineering evidence,
but do not alone justify product adoption. Disabled-sync results are diagnostic
and are not durability evidence.

## 7. Comparison baselines

For each B-link change retain and report both:

1. Production/current `main` B+Tree.
2. The accepted B-link baseline immediately before the change.

Report `candidate / main` and `candidate / previous-blink` where comparable.
This distinguishes whether B-link is valuable against production from whether
the latest change improves the existing B-link line. RocksDB may be an
external reference, but is not dodb's direct correctness or adoption baseline.

## 8. Product adoption policy

Freeze thresholds before inspecting final results. Do not adjust them after
seeing the outcome.

### Core regression budget

Against `main` under identical conditions, a sustained regression greater than
10% in any ordinary core workload is a default product-adoption blocker. Core
workloads include point Get, short Query, width-1 writes, width 2-4
transactions, W1 95/5, and W2 50/50. Quantify measurement noise first; smaller
differences require repeated runs and variance-aware interpretation.

A regression greater than 10% may be accepted only when the affected workload
was declared a non-goal in advance, or when an intentional trade-off against a
substantial product benefit is separately approved. Do not apply this numeric
rule mechanically to architecture stress workloads.

### Improvement and Pareto assessment

Do not summarize a result by its single highest throughput. Report gains and
regressions for every workload family, including read and small-transaction
performance, write gains, range-query behavior, tail latency, and resource or
write-amplification cost.

An improvement must be repeatable and clearly larger than measured noise. The
added B-link complexity should be justified by at least one of:

- A clear sustained throughput gain in write-heavy or small-transaction
  product workloads.
- A clear scaling improvement as concurrency rises.
- Higher mixed-workload throughput while protecting read latency.
- Better burst and backpressure behavior.

The gain must not push a core workload beyond its regression budget. Existing
`1.5x different-leaf-heavy` and `1.25x geometric mean` thresholds in
[`b-link-batched-engine.md`](b-link-batched-engine.md#19-architecture-hypothesis-criteria)
remain secondary architecture-hypothesis criteria only; they are not the sole
product adoption rule.

## 9. Tail latency and durable-write attribution

Record throughput and p50/p95/p99 for each primary workload. For durable
writes, split queue wait, validation/planning, physical execution, WAL
encoding/append, sync, publication, and end-to-end latency where available.
Separate storage-engine scheduling/backpressure delay from physical sync
latency. A mean throughput gain accompanied by a severe p99 regression is not
a product improvement.

## 10. Required result record

Every optimization or architecture result must record:

1. Exact candidate source SHA and baseline SHA(s).
2. Engine selector.
3. Machine, CPU, and RAM.
4. Filesystem and storage device.
5. Durability mode.
6. Cache capacity and working-set size/regime.
7. Key and value sizes.
8. Transaction width.
9. Read/write/Query mix.
10. Concurrency.
11. Key distribution.
12. Repetitions and run duration.
13. Throughput and p50/p95/p99.
14. Conflict, overload, and error counts.
15. Relevant internal attribution metrics.
16. Raw artifact path and hash.
17. `candidate / main` and `candidate / previous-blink` where available.

State conclusions separately as:

- Architecture diagnostic success/failure.
- Product workload improvement/regression.
- Production adoption readiness.

These conclusions are not interchangeable.

## 11. Relationship to `phase0-bench`

Extend the existing `phase0-bench` harness where practical; do not create a
replacement harness before assessing its capabilities. The current binary
already provides engine selectors, write/read/mixed suites, Get/Query/Scan,
uniform/sequential/hotspot/same-leaf/different-leaf distributions, configurable
writers/readers/transaction widths, 95/5 and 50/50 mixes, working-set/cache
settings, configurable key/value sizes, real/injected/disabled sync, collection
delay, repetitions, and JSONL output.

The current harness is not yet the complete product suite:

- Widths and sizes are configurable, but the product matrix is not encoded as
  named defaults; make widths 1/2/4/8 and the value classes above easy to run.
- Mixed workloads use width-1 transactions, uniform keys, and Get reads. Mixed
  Query and mixed key distributions are not currently represented.
- Write transactions support unconditional mutation and insert-if-absent;
  revision-equality and read-dependent transaction mixes need coverage.
- Cache and working set can be configured, but resident/non-resident regimes
  are not explicitly named or enforced by the suite.
- The workload loops are closed-loop worker saturation. Windowed metrics do
  not provide open-loop burst offered load, queue/rejection behavior, or
  recovery measurements.
- The binary is single-database/single-tenant and does not implement the
  Layer B multi-tenant system benchmark.

Treat the existing synthetic matrix in Section 17 of
[`b-link-batched-engine.md`](b-link-batched-engine.md#17-architecture-and-stress-benchmark-workloads)
as architecture/stress coverage. Preserve it for structural diagnosis, and
use this document as the product workload policy for future B-link
implementation and optimization decisions.
