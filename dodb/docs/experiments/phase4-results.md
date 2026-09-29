# Phase 4 Results: Persistent Parallel Leaf Workers

## Scope

Phase 4 keeps logical admission and FIFO transaction ordering on one
coordinator, while eligible non-structural leaf work runs on persistent OS
workers. This diagnostic compares the `planned-blink` sparse serial executor
with `parallel-blink` using two workers on the 2-OCPU OCI target. The run uses
sync-disabled writes and is not a durability benchmark. No Phase 5 work was
started.

## Initial Scoped-Thread Attempt

The initial scoped-thread implementation measured effective parallelism
0.9824 and throughput ratio 0.954x. Per-group OS-thread creation and teardown
was identified as the execution problem. The worker implementation was changed
to a fixed pool whose threads live with `BlinkStore` and receive one command
per participating worker per group.

## Persistent Worker Pool

### Pool lifecycle

`BlinkStore` owns the configured worker count and pool. Enabling parallel
execution creates one persistent thread per worker, each with its own dedicated
`std::sync::mpsc` receiver. Group execution sends worker-bucket commands; pool
drop sends shutdown commands and joins all threads. The benchmark seeds the
store before enabling this pool.

### Deterministic job partition

Planner-ordered leaf jobs are assigned by `job_index % worker_count`. Each
participating worker receives one vector of jobs and processes that vector in
order. The coordinator collects every participating worker result before
continuing.

### Private leaf ownership

Each job owns a private clone of its existing target `BlinkPage`. Worker
threads receive only owned leaf jobs and do not access the store, sparse
working state, WAL, generation publisher, or batch metrics. Coordinator code
aggregates metrics and merges final leaf pages into the sparse working-state
overlay. No page latch or shared mutable page is used.

### FIFO commit and WAL assembly

The coordinator assigns final commit LSNs in transaction FIFO order before
dispatch. Workers apply `Revision::from(commit_lsn)` and encode a page image at
each transaction boundary. Worker completion order is discarded during
assembly: page results are indexed by FIFO position and the existing ordered
`ExecutedPlanTransaction` path builds WAL commits in logical FIFO order.

### Structural/overflow fallback

Single-leaf groups are skipped. Multi-leaf transactions, cross-leaf dependency
edges, overflow/allocator requirements, and leaf overflow requiring a split
discard speculative worker output and run the original plan through the sparse
serial executor. Structural split metrics are changed only by that serial
execution. This initial private-page design has no shared page latch and no
worker dependency waiting; cross-leaf dependencies use group fallback.

### WAL-before-install atomicity

Workers prepare private candidates only. The coordinator joins all workers,
validates and assembles the plan, appends and syncs WAL, then installs the
sparse `BlinkStateDelta` and publishes the generation. A worker never appends
WAL or changes committed state.

## Correctness

The implementation passed `cargo fmt --all -- --check`,
`cargo test -p dodb-storage`, `cargo test -p dodb-storage --bin phase0-bench`,
`cargo test --workspace`, and `git diff --check`. The Phase 4 tests remain in
place, including FIFO WAL assembly, fallback behavior, generation atomicity,
WAL failure atomicity, randomized differential behavior, and a focused test
that verifies persistent worker threads are reused across execute cycles.

## Local Smoke

The delay-zero one-repetition development smoke passed its gate:

| engine | mutation ops/s | relative |
| --- | ---: | ---: |
| planned | 17,769.77 | 1.000x |
| parallel | 17,093.47 | 0.962x |

Effective worker parallelism was 1.292. The parallel executor completed 1,733
groups and skipped 429 single-leaf groups, for coverage of approximately
80.2%. Multi-leaf, dependency, overflow, and structural fallback counts were
all zero. Errors and overloads were zero, and full-state clones and clone time
were zero. This is a development smoke, not an OCI performance conclusion.

## OCI Diagnostic

The OCI diagnostic used three repetitions per engine, 16 writers, width 1,
`different-leaf-heavy`, working set 100,000, cache 4,096, 16-byte keys,
64-byte values, group limit 64, 4 MiB group cap, queue 256, collection delay
0 us, sync disabled, two Tokio workers, 1 s warmup, 2 s duration, and seed
`0x3a042026`.

| engine | mut/s median | min..max | relative |
| --- | ---: | ---: | ---: |
| planned-blink | 7,818.20 | 7,678.95..7,870.21 | 1.000x |
| parallel-blink | 6,717.26 | 6,689.50..6,755.83 | 0.859x |

The parallel speedup was 0.859x.

## OCI Parallel Metrics

| metric | value |
| --- | ---: |
| effective parallelism, repetition 0 | 0.976 |
| effective parallelism, repetition 1 | 1.092 |
| effective parallelism, repetition 2 | 1.075 |
| effective parallelism, median | 1.075 |
| parallel group coverage | 3,531 / 4,361 = 81.0% |
| leaf jobs / parallel group | 11.20 |
| worker dispatches / parallel group | 2.00 |
| multi-leaf fallback | 0 |
| dependency fallback | 0 |
| overflow fallback | 0 |
| structural fallback | 0 |
| single-leaf skips | 830 |

## Interpretation

The OCI result is a **mechanism failure** under the overlap criterion because
median effective parallelism was below 1.20. It is also a **regression** in
throughput because parallel median throughput was 0.859x the serial planned
median, below 0.95. Coverage exceeded 80%, so low coverage does not explain
the result. No worker-pool redesign or additional benchmark matrix was started.

The local smoke and OCI result agree in throughput direction: parallel was
slower in both. Local smoke showed useful overlap (1.292), while OCI did not
meet the overlap threshold (1.075 median), so the overlap result does not
replicate across targets.

## Remaining Serial Costs

Median cumulative timings from the parallel OCI records:

| component | median |
| --- | ---: |
| planning | 178.65 ms |
| physical execution | 309.61 ms |
| WAL assembly | 26.37 ms |
| WAL append | 459.11 ms |
| dirty tracking | 14.46 ms |
| catalog construction | 412.78 ms |
| generation publication | 448.73 ms |
| parallel worker busy time, summed | 208.37 ms |
| dispatch-to-final-result wall time | 193.59 ms |

These are the existing cumulative benchmark attributions for each repetition;
they are not per-request timings or exclusive CPU profiles.

## Phase 5 Readiness

**NO.** Correctness, FIFO/WAL ordering, sparse-state behavior, and
WAL-before-install atomicity passed. However, OCI effective parallelism was
1.075, below 1.20, and throughput speedup was 0.859x, below 0.95. Review the
worker mechanism before Phase 5 design proceeds. Phase 5 implementation,
concurrent splits, SMO coordination, and page latches were not started.

## Artifacts

- Planned control:
  [`planned-pool-different-width1-nosync.jsonl`](results/oci-a1-2ocpu-12g-200g/phase4/planned-pool-different-width1-nosync.jsonl)
  SHA256 `6b6bdc8044ed4713e8c7b0cf9d4c8cd2b7510c4afc83dce0bc416c3580939919`
- Parallel run:
  [`parallel-pool-different-width1-nosync.jsonl`](results/oci-a1-2ocpu-12g-200g/phase4/parallel-pool-different-width1-nosync.jsonl)
  SHA256 `4bd3b0a9929de7c47df4d40491147f6430e010340300944b6ee4e630ca7bf50d`

## OCI Artifact Preservation

Before fast-forward, the three pre-existing Phase 3.1 artifacts matched their
expected SHA256 values. They were moved unchanged to
`/home/opc/dodb-oci-artifacts-pre-phase4-0dc5219/phase3/`. After fast-forward,
the tracked copies matched both the expected SHA256 values and the preserved
backup copies:

| artifact | SHA256 |
| --- | --- |
| `planned-sparse-nosync-w16-width16-ws100k.jsonl` | `24c0426b0768e74fb8e44791c82dc3414a95d52503720268dd1d494ba4a01d54` |
| `planned-sparse-nosync-w16-width16-ws4096.jsonl` | `fd0c57281c3090c1ba83887364238f6f8286d78b39dbddb36f84f76a7818a3b3` |
| `planned-sparse-real-w16-width16-ws100k.jsonl` | `ecd444dc25c5f409ba16f39b03c7efb2ce8bff3fc3f9570e0355cdece4a14317` |

## OCI CPU Availability and Granularity Control

The OCI host reports two online Neoverse-N1 CPUs. `nproc` returned 2,
`taskset -pc` reported affinity `0,1`, Python affinity was `[0, 1]`, and
`cpuset.cpus.effective` contained `0-1`. `/sys/fs/cgroup/cpu.max` produced no
content at the queried path. `/proc/self/cgroup` reported
`0::/user.slice/user-1000.slice/session-352.scope`. No configuration was
changed.

A single two-process CPU saturation test ran for about five seconds. Both
children had affinity `[0, 1]` and each accumulated about 4.995 CPU seconds.
Combined child CPU time was 9.989 seconds over 5.007 seconds of parent wall
time, for `cpu_parallel_factor = 1.995`. This confirms that the OCI environment
can run two CPU-bound processes concurrently.

### Delay-zero reference

| engine | mut/s median | min..max | median avg group requests | process CPU, one-core median | process CPU, machine median |
| --- | ---: | ---: | ---: | ---: | ---: |
| planned-blink | 7,818.20 | 7,678.95..7,870.21 | 11.54 | 100.87% | 50.44% |
| parallel-blink | 6,717.26 | 6,689.50..6,755.83 | 9.08 | 104.35% | 52.18% |

Delay-zero parallel effective-parallelism samples were 0.976, 1.092, and
1.075, with median 1.075. Coverage was 81.0%, with 11.20 leaf jobs and 2.00
worker dispatches per successful parallel group. The process used only about
one CPU on average despite the host's verified two-CPU availability.

### 100 us control

The same workload was run with collection delay 100 us. Both artifacts have
three valid records at commit `3465dbb996bb4a0b9419a8a4099548ad0d8af8a0`,
zero errors and overloads, disabled sync, and zero full-state clones.

| engine | mut/s median | min..max | median avg group requests | process CPU, one-core median | process CPU, machine median |
| --- | ---: | ---: | ---: | ---: | ---: |
| planned-blink | 5,293.32 | 5,269.65..5,319.27 | 16.00 | 63.37% | 31.69% |
| parallel-blink | 5,196.71 | 5,191.95..5,241.33 | 16.00 | 66.42% | 33.21% |

The 100 us speedup was 0.982x. Parallel effective-parallelism samples were
1.092, 1.122, and 1.124, with median 1.122. Coverage was 100%: 1,957 of
1,957 groups used parallel execution. Each parallel group had 16.00 leaf jobs
and 2.00 worker dispatches; there were no single-leaf skips or correctness
fallbacks. Thus the average parallel job count increased from 11.20 to 16.00
per group, but worker overlap remained below 1.20. Process CPU use was about
66% of one core, or 33% of the machine, in the parallel records.

### Interpretation and next priority

The two-process saturation result rules out an OCI one-core allocation as the
cause of the weak Phase 4 overlap. The 100 us control increased group and job
granularity and reached full parallel-group coverage, but did not reach the
required 1.20 effective parallelism. Its throughput ratio was 0.982x, which
meets the 0.95 throughput floor but does not offset the overlap failure. The
granularity hypothesis is therefore **rejected**: larger groups alone did not
recover worker overlap on this target. The current Phase 4 leaf-job dispatch
design is not worthwhile on this OCI target under these measurements; no
worker-pool optimization was started.

At 100 us, Phase 4 does not justify proceeding to Phase 5. The next engineering
priority is serial generation/catalog lifecycle cost. In the delay-zero
parallel median repetition, catalog construction was 412.79 ms and generation
publication was 448.73 ms, together 861.52 ms of 1,945.14 ms processing
(about 44%). The main cost to investigate is the full `PageCatalog.pages`
`BTreeMap` clone and dropping the retired generation/catalog. WAL append was
459.11 ms, but the combined catalog/publication lifecycle was larger. No such
optimization was implemented in this diagnostic.

### 100 us artifacts

- Planned:
  [`planned-pool-different-width1-nosync-delay100us.jsonl`](results/oci-a1-2ocpu-12g-200g/phase4/planned-pool-different-width1-nosync-delay100us.jsonl)
  SHA256 `9751e3e14a43e2b20fe0324d895a55b307029532085b6a5880f9d5a01facbae6`
- Parallel:
  [`parallel-pool-different-width1-nosync-delay100us.jsonl`](results/oci-a1-2ocpu-12g-200g/phase4/parallel-pool-different-width1-nosync-delay100us.jsonl)
  SHA256 `a0da0176b1457ac91d716e5e4fd472911c9d54ed8981884f69bdbe6b0cbdf8fc`
- OCI backup:
  `/home/opc/dodb-oci-artifacts-cpu-diag-3465dbb/phase4/`
