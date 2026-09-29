# Planner Borrowed Routing Results

## Motivation

The current width-1 planned Blink workload calculates one route per mutation.
Before this experiment, `plan_batch()` used the generic `ReadPageSource`
implementation for `BlinkState`; each page lookup cloned the complete
`BlinkPage` and wrapped it in a new `Arc`. The experiment measured route cost
before replacing only planner routing with borrowed page access. Arena
allocation was not used.

## Source Finding

`BlinkState::page()` still returns `Arc<BlinkPage>` by cloning the page from
`state.pages`. Generic routing continues to serve query, scan, read, and
`GenerationPin` call sites. The planner now uses a specialized traversal that
borrows each `&BlinkPage` directly from `state.pages.get(&page_id)`.

The planner route timer encloses only the route helper call. Key encoding and
validation, route-hint and planned-mutation construction, dependency updates,
and leaf-group construction remain outside it. The route page-visit counter
counts each page successfully fetched during traversal. Right-link hops are
reported separately.

## Phase A Routing Attribution

OCI median, three repetitions:

| metric | Phase A median |
|---|---:|
| throughput | 22,317.09 mutations/s |
| planning | 504.823 ms/run; 11.132 µs/mutation |
| route | 425.891 ms/run; 9.482 µs/mutation |
| route share of planning | 84.36% |
| route calls | 44,664/run |
| page visits per route | 4.00 |
| right-link hops | 0/run |

Every run reported zero errors and zero overloads. The route share exceeded the
25% decision threshold, so Phase B proceeded.

## Borrowed Routing Design

`plan_batch()` calls `find_leaf_in_blink_state_borrowed()`. It follows the
existing route rules directly over borrowed pages and keeps the same
`HashSet` cycle guard. No page clone, `Arc::new`, `Arc` clone, or other page
ownership is created during planner traversal. The generic route helper and
all non-planner routing call sites remain in place.

## Correctness

Generic and borrowed routing returned the same page IDs, correction counts,
and visit counts for a single-leaf tree and a multi-leaf tree with two internal
levels and separator-boundary keys. The comparison also covered high-key/right-
link correction, cycle detection, missing root pages, and a non-tree root page.

The focused tests and all Phase B validation gates passed:

- `cargo fmt --all -- --check`
- `cargo test -p dodb-core`
- `cargo test -p dodb-storage` (103 unit tests; recovery child 8/8)
- `cargo test -p dodb-storage --bin phase0-bench` (10 tests)
- `cargo test --workspace`
- `git diff --check`

## OCI Method

Both runs used the same planned Blink width-1 workload: 16 writers,
`different-leaf-heavy`, working set 100,000, cache 4,096, key size 16,
value size 64, group limit 64, group byte limit 4,194,304, queue 256,
collection delay 0 µs, sync disabled, two Tokio workers, 1-second warmup,
2-second duration, three repetitions, and seed `0x3a042026`. Phase A ran at
`cde31db49916127ef3d9ce72bc6b719ee69fd34d`; Phase B ran at
`97d28221c5bfdd5c268ac0c67b0aecbf4e73d9ae`. Every record identifies the
expected source commit and has zero errors and overloads.

The benchmark host is OCI A1 with 2 OCPU and 12 GiB RAM. `/home/opc/dodb` is
on the mounted 30 GB XFS root filesystem; the 200 GB block device is not
mounted as the benchmark filesystem. These are sync-disabled CPU and engine
diagnostics, not durability results or measurements on the 200 GB volume.

## Results

The table compares per-run medians. Normalized cost and route-share values are
medians of each run's ratio to successful mutations or planning time.

| metric | before | after | change |
|---|---:|---:|---:|
| throughput (mut/s) | 22,317.09 | 27,529.42 | +23.36% |
| planning (ms/run) | 504.823 | 171.152 | -66.10% |
| planning (ns/mutation) | 11,132.05 | 3,105.60 | -72.10% |
| route (ms/run) | 425.891 | 69.166 | -83.76% |
| route (ns/mutation) | 9,481.85 | 1,241.66 | -86.90% |
| route share of planning | 84.36% | 40.84% | -43.52 pp |
| route calls/run | 44,664 | 55,111 | +23.39% |
| page visits/route | 4.00 | 4.00 | unchanged |
| right-link hops/run | 0 | 0 | unchanged |

## Normalized Planning Cost

The borrowed route met the strong-success thresholds: route cost is below 50%
of its prior normalized cost, planning is below 75%, and throughput is at
least 8% higher. Throughput increased 23.36%. The route's normalized cost
fell by 86.90%, and total planning cost per mutation fell by 72.10%.

The fixed-duration run processed more mutations after the change. Therefore,
the normalized per-mutation figures are the primary comparison; absolute run
times alone do not describe the amount of work completed.

## Remaining Planner Cost

For each Phase B run, non-route planning cost is calculated as
`planning_nanos - planner_route_nanos`, before taking the median:

| metric | Phase B median |
|---|---:|
| non-route planning | 100.835 ms/run |
| non-route planning per mutation | 1.830 µs/mutation |
| non-route share of planning | 59.16% |

## Interpretation

This is a strong success. Borrowed routing preserved route results while
removing page ownership work from planner traversal. Throughput improved and
normalized planning cost dropped substantially. B-link structure, transaction
ordering, dependency semantics, WAL semantics, publication, and recovery did
not change.

## Next Bottleneck

The largest measured top-level component after borrowed routing is WAL append
at 566.517 ms/run. Physical execution is 400.362 ms/run and planning is
171.152 ms/run. The next engineering priority is WAL append attribution; this
experiment did not change WAL code.

The remaining planner work has not been attributed further. Candidate costs
visible in `plan_batch()` include mutation key encoding, encoded-key
validation, mutation clones, the encoded-key clones for `RouteHint` and
`PlannedMutation`, `last_key_writer` as `BTreeMap<Vec<u8>, ...>`, cloned
`encoded_keys`, cloned `mutated_key_set`, and the leaf-group and transaction-
group `BTreeMap`/`BTreeSet` structures. These remain candidates for a later
attribution only; no planner representation or allocation strategy changed
here.

## Arena Assessment

Arena allocation is not recommended from this result. The measured routing
ownership cost was addressed by borrowing existing pages, which removes the
page clone and `Arc` allocation directly. The remaining non-route planning
cost has not been shown to be dominated by temporary allocations. Reconsider
an arena only after separate attribution demonstrates that those allocations
are a major remaining cost.

## Experiment Status

The borrowed-routing optimization is complete and meets the strong-success
criteria. `PLANNER_ROUTE_ATTRIBUTION_SHA` is
`cde31db49916127ef3d9ce72bc6b719ee69fd34d`;
`BORROWED_ROUTING_SHA` is
`97d28221c5bfdd5c268ac0c67b0aecbf4e73d9ae`. Phase 4 remains an experimental
worker-pool implementation whose OCI delay-zero diagnostic found insufficient
worker overlap and a throughput regression. Phase 5 has not started. The B-link
+ batching engine remains experimental and has not been adopted as the main
engine. The trusted internal WAL page-image fast path and its recorded result
remain unchanged.

## Artifacts

Raw OCI JSONL artifacts, copied locally with matching SHA256 checksums:

- `docs/experiments/results/oci-a1-2ocpu-12g-200g/planner-routing/planned-routing-before-width1.jsonl` — `4752e45661c6b3600602e1cb0b7d70170c3a2f9eb8e0b6518effd64d41cbd17b`
- `docs/experiments/results/oci-a1-2ocpu-12g-200g/planner-routing/planned-routing-after-width1.jsonl` — `1e5ed9492b071a3c38b16f69fa256cca77f65364e1e2eaedfca92778caec11dd`

OCI copies outside the repository:

- `/home/opc/dodb-oci-artifacts-planner-routing-cde31db/planned-routing-before-width1.jsonl`
- `/home/opc/dodb-oci-artifacts-planner-routing-97d2822/planned-routing-after-width1.jsonl`
