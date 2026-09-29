# Physical Execution Attribution

## Motivation

After WAL group-write, OCI's width-1 primary measured 15,328.19 mut/s, with
about 599.26 ms of physical execution, 580.14 ms of WAL append, and 349 ms of
planning per run. This experiment separates physical execution into mutation,
clone, restamp, cached refresh, and page-image encoding costs so the next
engineering priority can be selected from measured costs.

The primary workload recorded 30,685 planned mutations, 30,685 leaf loads,
30,685 leaf encodes, no coalesced mutations, and no leaf splits or reroutes.
It therefore measures the ordinary leaf mutation path without structural
fallback work.

## Instrumentation

The added counters time the transaction mutation loop; the leaf page clone;
the leaf entries vector clone; the overlay installation clone and insert;
transaction restamp; cached leaf refresh; dirty page encoding; and superblock
encoding. Clone counts are recorded separately. Leaf load time excludes the
page lookup: the lookup returns a reference before the timer, and the timer
covers the `BlinkPage` clone.

Clone timers are nested within `physical_mutation`. They are shown separately
for attribution and are not subtracted from `physical_execution` when computing
its residual. The physical residual is the physical execution median minus the
medians of mutation, restamp, cached refresh, page encoding, and superblock
encoding, floored at zero. Mutation residual is mutation minus its three clone
medians, also floored at zero. The tables use medians of the three repetition
values; consequently, displayed component medians need not sum exactly to the
physical median before residual is applied.

## OCI Method

The instrumented binary was built in release mode at
`5f57cfe6fffd037b30282ebaefbd41d9ff6fae59` on the 2-OCPU OCI A1 host. Both
measurements used 16 writers, working set 100,000, cache capacity 4,096,
16-byte keys, 64-byte values, group limit 64, group byte limit 4 MiB, queue
capacity 256, zero collection delay, disabled sync, two Tokio workers, 1 s
warmup, 2 s duration, and three repetitions. The width-1 primary used
transaction width 1, different-leaf-heavy distribution, and base seed
`0x3a042026`. The width-16 check used uniform distribution and base seed
`0x3a032026`.

The benchmark files report `git_commit` equal to the instrumentation commit,
`engine=planned-blink`, `sync_mode=disabled`, zero errors, zero overloads, and
zero full-state clones. The width-1 median throughput was 15,222.51 mut/s,
0.69% below the preceding 15,328.19 mut/s result. The width-16 median was
14,575.44 mut/s, 1.91% below the preceding 14,859.46 mut/s result. Neither
change reaches the 15% instrumentation-overhead threshold.

These are sync-disabled CPU and engine diagnostics, not durability results.
`/home/opc/dodb` resides on the 30 GB XFS root filesystem; the 200 GB block
device is not mounted as a filesystem.

## Width-1 Breakdown

The median processing time was 1,905.748 ms. Percentages below use the median
physical execution total of 619.932 ms.

| component | ms | % physical |
|---|---:|---:|
| physical execution | 619.932 | 100.00% |
| physical mutation | 319.628 | 51.56% |
| └ mutation residual | 209.696 | 33.83% |
| physical restamp | 3.266 | 0.53% |
| physical cached refresh | 40.518 | 6.54% |
| physical page encode | 185.561 | 29.93% |
| physical superblock encode | 31.052 | 5.01% |
| physical residual | 39.907 | 6.44% |

Mutation residual is nested inside physical mutation. The clone times below
are nested within physical mutation, and are not additive rows in this
top-level breakdown.

## Clone Breakdown

| clone | count | total ms | ns/op |
|---|---:|---:|---:|
| leaf load | 30,470 | 24.133 | 792.0 |
| leaf entries | 30,470 | 36.696 | 1,195.6 |
| leaf install | 30,470 | 49.104 | 1,611.6 |
| cached refresh | 30,470 | 40.518 | 1,330.2 |

The three mutation clone costs total 109.933 ms. Including cached refresh,
the four clone costs total 150.451 ms, or 24.27% of physical execution.

## Width-16 Breakdown

The median processing time was 1,927.927 ms. Percentages use the median
physical execution total of 591.779 ms.

| component | ms | % physical |
|---|---:|---:|
| physical execution | 591.779 | 100.00% |
| physical mutation | 392.672 | 66.35% |
| └ mutation residual | 228.007 | 38.53% |
| physical restamp | 5.651 | 0.95% |
| physical cached refresh | 2.724 | 0.46% |
| physical page encode | 183.652 | 31.03% |
| physical superblock encode | 2.019 | 0.34% |
| physical residual | 5.061 | 0.86% |

| clone | count | total ms | ns/op |
|---|---:|---:|---:|
| leaf load | 29,222 | 65.708 | 2,248.6 |
| leaf entries | 29,232 | 24.550 | 832.1 |
| leaf install | 29,232 | 74.407 | 2,545.4 |
| cached refresh | 1,827 | 2.724 | 1,491.0 |

## Workload Comparison

| normalized cost | width 1 | width 16 |
|---|---:|---:|
| physical mutation per mutation | 10,489.9 ns | 13,433.0 ns |
| all four clone costs per mutation | 4,936.0 ns | 5,728.4 ns |
| page encoding per mutation | 6,090.0 ns | 6,280.8 ns |
| restamp per mutation | 107.2 ns | 193.4 ns |
| restamp per transaction | 107.2 ns | 3,093.0 ns |
| page encoding per encoded leaf page | 6,090.0 ns | 6,280.8 ns |

The total leaf entries and install clone cost per clone fell at width 16, while
leaf load and install clone costs rose. Aggregate clone cost per mutation rose
about 16.1%. Page encoding per mutation was similar, rising about 3.1%.
Restamp cost per mutation was also higher, while restamp remained below 1% of
physical execution for both widths. The larger difference in physical
execution composition is the mutation residual share: 33.83% at width 1 and
38.53% at width 16.

## Interpretation

Width-1 decision thresholds:

- Leaf clone costs plus cached refresh: 24.27%, below the 40% clone threshold.
- Page and superblock encoding: 34.94%, below the 40% encoding threshold.
- Restamp: 0.53%, below the 25% restamp threshold.
- Mutation residual: 33.83%, above the 30% mutation residual threshold.

Source inspection identifies `leaf_fits()` as the leading candidate inside
mutation residual. Each ordinary mutation calls it after updating the leaf;
`leaf_fits()` calls `encode_leaf_body()`, which allocates a BODY_SIZE temporary
vector, allocates a record vector for each entry, and copies the completed body
to another vector. Routing, key/value copies, and overlay map operations are
also included in the residual, but were not separately timed in this task.

## Next Engineering Priority

First priority: further attribute the mutation residual, starting with the
repeated `leaf_fits()` / `encode_leaf_body()` cost identified by source
inspection. Page image encoding is second by absolute measured time: page and
superblock encoding account for 216.613 ms at width 1, although their combined
share is below the 40% threshold. No candidate was implemented here.

## Experiment Status

This task added instrumentation only. No physical execution optimization was
implemented in this task. The B-link + batching architecture and WAL group
writer are unchanged. Phase 4 remains ineffective on the current 2-OCPU target
per its existing result; Phase 5 has not started.

## Artifacts

- Width 1: [`planned-physical-attribution-width1.jsonl`](results/oci-a1-2ocpu-12g-200g/physical-attribution/planned-physical-attribution-width1.jsonl), SHA256 `d54939ca872565d3914caaf79e14335b53628fb6c8475cdb40325a9347c522a1`
- Width 16: [`planned-physical-attribution-width16.jsonl`](results/oci-a1-2ocpu-12g-200g/physical-attribution/planned-physical-attribution-width16.jsonl), SHA256 `1bb8a8d6c6820cf02d2372e1212900bb70af1c69fcbc723e4eefe7f8e02744d6`
- Both files contain three validated records at instrumentation commit `5f57cfe6fffd037b30282ebaefbd41d9ff6fae59`; local SHA256 values match the OCI copies.
