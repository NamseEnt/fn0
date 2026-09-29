# Phase 3 Results: Logical Batch Planner, Routing, and Same-Leaf Coalescing

Status: implementation and correctness complete; development smoke/profiling
complete; Phase 4 parallel page execution not started.

Branch: `experiment/b-link-batched-engine`

Starting HEAD: `863017b08ba0d344d18c7c2b8b4866cbe1070c5f`

Implementation commit used for the smoke runs: `4ad2ee895c43d930a234955fee17b86e6750e32a`

The Mac results in this document are development smoke/profiling only. They
are not a production benchmark, adoption evidence, an adoption-threshold
decision, or a final main-versus-experimental performance conclusion. The
production target is OCI A1 with 2 OCPU, 12 GiB RAM, and a 200 GB block
volume. The same implementation commit must be measured there before an
adoption decision.

## 1. Scope and non-goals

This phase adds a logical batch planner and a serial physical executor to the
versioned Blink engine. The planner provides FIFO admission, sparse staged
state, route hints, leaf grouping, dependency metadata, and same-leaf setup
reuse. The physical Blink writer remains exactly one serial executor.

This phase does not add page workers, multi-writer execution, parallel
mutation, concurrent SMO, parallel splits, writer latches, adaptive batching,
or WAL format changes. The Phase 2 versioned read path remains in use.

The new benchmark selector is `planned-blink`. Existing `main-btree`,
`serial-blink`, and `versioned-blink` selectors remain available.

## 2. Before structure from the repository

Before Phase 3, `BlinkStore::apply_transaction_group` used one mutable
`BlinkState` clone for the group and then cloned the complete working state for
each candidate transaction. An accepted candidate was cloned again when it
was saved for later WAL/publication processing. In simplified form, the path
was:

```text
apply_transaction_group
    working = self.state.clone()
    for request:
        candidate_state = working.clone()
        validate request and conditions
        mutate candidate_state
        compute commit LSN
        save candidate_state.clone()
    append one WAL group
    publish candidates
```

The serial control retains this path and now counts the full-state clone call
sites as `full_state_clones`: one group working clone, one candidate clone per
request, and one saved-candidate clone per accepted request. This is an
instrumentation control, not a claim that every clone has identical byte
cost.

Coordinator segmentation was preserved. Mutation requests can be collected
into one logical group, while `TransactGet`, `Observe`, and checkpoint-like
operations remain barriers. A mutation before and after such an operation is
not merged into one Phase 3 plan.

The Phase 2 read path was not replaced. `BlinkReadHandle` still pins one
immutable generation, traverses the `PageCatalog` and `PageCell` values
without the coordinator or writer store mutex, and observes one-generation
consistent state.

## 3. Logical admission and overlay

`LogicalOverlay` is a sparse map keyed by encoded document key. It references
the committed `BlinkState` and stores only keys touched by accepted mutations.
Each staged entry contains:

| Field | Meaning |
| --- | --- |
| `present` | Whether the staged logical value is present rather than a tombstone/missing state. |
| `value` | The staged value when value materialization is required. |
| `revision` | An internal `ProvisionalRevisionToken`. |
| `originating_transaction_position` | FIFO position that accepted the delta. |

The admission pass processes requests in FIFO order. It validates request
structure, keys, values, duplicate conditions/mutations, and conditions
against committed state plus the current sparse overlay. An accepted request
gets a FIFO position and provisional token, then contributes its mutation
delta to the overlay. An invalid or conflicting request contributes no delta,
no physical plan, and no WAL image; later requests continue against the
previously accepted overlay.

The implementation preserves the current public `TransactionCondition` shape.
`Exists` and `NotExists` conditions observe staged present/missing state. For
revision checks, the overlay carries a provisional token until page-image
counts determine the final commit LSN. The public condition API currently
accepts only a numeric `Revision`, so a caller cannot name a not-yet-issued
same-batch symbolic token directly. Existing numeric revision behavior remains
unchanged; this API boundary is an unresolved question for a future contract
extension rather than an implicit change to revision semantics.

## 4. Plan representation

The planner and executor are separate in the code structure.

```text
BatchPlan
    transactions: Vec<PhysicalTransactionPlan>
    leaf_groups: Vec<LeafGroupPlan>
    dependencies: Vec<DependencyEdge>
```

`PhysicalTransactionPlan` contains:

- FIFO position;
- provisional revision token;
- ordered `PlannedMutation` values;
- encoded keys and the mutated-key set;
- route hints for each mutation; and
- dependency metadata for same-key, condition-key, same-target-page,
  same-transaction, and structural-route relationships.

`LeafGroupPlan` identifies a route-hint leaf and the ordered
`(fifo_position, mutation_index)` references assigned to it. Grouping records
locality without changing logical FIFO order. A multi-leaf transaction remains
one `PhysicalTransactionPlan` and can participate in several leaf groups.

`DependencyEdge` records predecessor, successor, and one of:
`SameKey`, `ConditionKey`, `SameTargetPage`, or `StructuralRoute`.
Same-transaction multi-leaf membership is represented in per-transaction
metadata and the leaf-group membership map. This is enough to identify
independent candidate work without pretending that this phase can execute it
in parallel.

The `independent_leaf_groups` metric counts groups not connected to another
leaf group by a dependency edge or by a multi-leaf transaction. Sequential
mutations within one leaf group are not incorrectly counted as cross-group
dependencies.

## 5. Routing and same-leaf coalescing

Planning routes each accepted mutation against the committed state and stores
the resulting leaf ID as a hint. A hint is not immutable truth. The serial
executor validates the cached page and uses the leaf high key and right
right sibling to correct a stale route; if that cannot prove the target, it falls
back to a fresh root traversal.

For ordered operations targeting the same current leaf, the executor keeps a
single mutable cached leaf candidate, loads and decodes it once, and applies
the mutations in FIFO order. It reuses the candidate for later mutations in
the same leaf range and records route reuse, leaf loads, leaf encodes, and
coalesced mutations. Logical transaction boundaries remain intact.

If the candidate no longer fits, the executor interrupts coalescing, invokes
the existing serial split path, invalidates the route, and lets subsequent
operations reroute through the B-link high-key/right-link rules. This is the
structural fallback. No concurrent SMO or parallel split was added.

## 6. Revision, WAL, and publication integration

The planner uses a provisional ordinal while logical admission and physical
mutation are being prepared. After each transaction's dirty-page set is known,
the FIFO WAL layout determines its commit LSN. Dirty page images are then
restamped from the transaction's provisional revision to that commit LSN.
The final `TransactionResult` and stored leaf revisions use the commit LSN,
not the provisional ordinal.

Each accepted transaction still has its own page-image sequence and its own
WAL `COMMIT` record. Same-leaf coalescing reuses computation and page setup;
it does not merge logical commits. A multi-leaf transaction can contribute
images from several leaf groups but receives one commit LSN, one result, and
one logical WAL commit boundary.

The publication sequence remains:

```text
logical admission
    -> serial physical execution
    -> per-transaction page-image assembly
    -> one WAL append group
    -> one WAL sync
    -> one prepared generation/catalog installation
    -> one generation publication
    -> successful responses
```

The general reader cannot observe intermediate transactions in the batch.
The WAL still retains every logical transaction boundary, so a tail ending
before a later commit recovers only the complete earlier logical commits.

## 7. Metrics and clone results

The planner exports counters/timers for logical groups and transaction
admission, conflicts and rejections, planning, physical execution, full-state
clones, route calculations/reuse/reroutes, leaf grouping/coalescing,
dependencies, leaf loads/encodes, structural fallback, page images, WAL
bytes, catalog construction, and generation publication.

Representative cumulative development-smoke observations from the final
release binary are:

| Engine/workload | Logical transactions | Full-state clones | Clones/group | Clones/transaction |
| --- | ---: | ---: | ---: | ---: |
| `serial-blink`, 1 writer, width 1, uniform | 35 | 105 | 3.00 | 3.00 |
| `planned-blink`, 1 writer, width 1, uniform | 36 | 36 | 1.00 | 1.00 |
| `serial-blink`, 16 writers, width 1, uniform | 224 | 476 | 17.00 | 2.13 |
| `planned-blink`, 16 writers, width 1, uniform | 245 | 30 | 1.00 | 0.12 |

The counts come from separate short runs with different request totals and
are clone-attribution evidence, not a throughput comparison. The planned
path performs one `BlinkState` clone per admitted physical group and does not
clone a complete state per transaction.

Representative planner locality observations:

| Planned workload | Leaf groups | Same-leaf groups | Route reuse | Coalesced mutations | Independent leaf groups | Leaf loads |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 16 writers, width 16, same-leaf-heavy | 96 | 96 | 3,632 | 3,299 | 0 | 429 |
| 16 writers, width 1, different-leaf-heavy | 238 | 4 | 4 | 0 | 238 | 242 |

The first row demonstrates same-leaf setup reuse. The second demonstrates
that the planner can expose independent leaf groups when transactions are
single-leaf and the distribution is deliberately separated. Width-16
different-leaf transactions can still have zero independent groups when one
logical transaction spans several leaves; that is required atomicity, not a
planner collapse.

## 8. Correctness coverage

Deterministic Phase 3 tests cover:

- staged A/B/C visibility through `Exists` conditions;
- failed middle transaction isolation with no overlay delta;
- same-key Put/Delete/Put ordering, final revisions, coalescing, and separate
  WAL commits;
- same-leaf and different-leaf grouping/dependency metadata;
- stale planned route after a preceding split;
- leaf, internal, and root structural fallback;
- one multi-leaf transaction with one commit LSN and atomic generation
  visibility;
- WAL tail recovery that retains the first logical commit and discards an
  incomplete later commit; and
- old-generation reader visibility while a new planned generation is
  published.

The randomized differential test uses seed `0x3a032026` and 300 operations.
It includes Put, Delete, Exists, NotExists, RevisionEquals, staged same-key
dependencies, multi-key transactions, split-heavy keys, and checkpoint/reopen
recovery. Mismatches report the seed and operation index. The reference model
compares accepted/rejected results, values, missing/tombstone state, revision,
and reopened state.

The existing Phase 2 read and publication tests remain enabled, including
generation pinning, stale route correction, root split traversal, checkpoint,
and reopen behavior.

## 9. Development-machine smoke and profiling

Machine: Apple M1, 8 logical CPUs, macOS 26.4, Rust release build, Tokio 8
workers, injected sync mode with zero delay, working set 512, cache capacity
128, one repetition. Write runs used 200 ms duration and 100 ms warmup. Read
and mixed runs used 150 ms duration and 75 ms warmup. These are intentionally
short development runs.

Representative write throughput observations, shown only to expose obvious
regressions and planner behavior:

| 16 writers, width 16 | `serial-blink` tx/s | `versioned-blink` tx/s | `planned-blink` tx/s |
| --- | ---: | ---: | ---: |
| uniform | 650.4 | 651.4 | 625.5 |
| same-leaf-heavy | 1,150.2 | 1,092.8 | 1,091.2 |
| different-leaf-heavy | 1,477.8 | 1,114.8 | 1,072.7 |

The values vary with the short run and are not used as Phase 3 performance
evidence or adoption evidence. No official 1/4/16/32/64/128 sweep was run.

Read regression smoke produced zero errors and kept the direct versioned read
path active. At 16 readers, `planned-blink` measured approximately 1.89M
Get/s, 0.94M Query/s, and 1.01M Scan/s in this run; the corresponding
`versioned-blink` controls were approximately 1.87M, 0.92M, and 1.09M/s.
The result is only a regression signal for this Mac and workload.

Mixed smoke also produced zero errors:

| Mix, 16 readers / 16 writers | Planned write tx/s | Planned read ops/s | Planning time | Physical execution time |
| --- | ---: | ---: | ---: | ---: |
| 95/5 | 1,505.6 | 29,214.7 | 3.96 ms | 10.74 ms |
| 50/50 | 1,720.1 | 1,962.0 | 3.64 ms | 12.57 ms |

The times are cumulative counters for the short process run, not per-request
latencies. The same runs reported catalog construction of 0.64 ms and 0.51
ms, and generation publication of 0.61 ms and 0.78 ms. WAL sync time was much
larger than planner time under this injected-sync file path. Within the
planner/tree portion, serial physical execution is currently the largest
measured Phase 3 component; the WAL image and durability path remain
substantial. A sampled CPU profile was not used, so this is counter-based
attribution rather than a flamegraph claim.

Raw artifacts are in:

```text
docs/experiments/results/phase3-development-smoke/
    serial-write.jsonl
    versioned-write.jsonl
    planned-write.jsonl
    versioned-read.jsonl
    planned-read.jsonl
    versioned-mixed.jsonl
    planned-mixed.jsonl
```

## 10. Phase 4 readiness

The plan now exposes ordered jobs per leaf group and explicit cross-group
dependency metadata. In the representative smoke runs, different-leaf,
single-key work exposed 238 independent groups in one run, while same-leaf
work exposed high route reuse and coalescing. Multi-leaf transactions remain
linked as one atomic logical unit. This is the representation Phase 4 needs;
no worker pool or parallel execution was started in this phase.

The largest remaining write bottlenecks are the serial physical executor,
per-transaction full-page redo image assembly, and the WAL sync/durability
path. PageCatalog construction/publication is measurable but smaller in the
short smoke counters. PageCatalog copy-on-write or chunked storage was not
introduced because the measured Phase 3 priority was planner/coalescing and
the existing immutable-generation contract is already preserved.

## 11. OCI benchmark follow-up

The partial OCI diagnostic results for the final Phase 3 branch HEAD are in
[`phase3-oci-results.md`](phase3-oci-results.md). They are not a complete
production benchmark or an adoption decision. The commands below remain
historical examples and were not the commands used for that diagnostic. A
future complete benchmark should compare `main-btree`
with `planned-blink` under identical files, seeds, runtime workers, cache,
working set, durability mode, and repetitions:

```text
cargo build --release -p dodb-storage --bin phase0-bench

target/release/phase0-bench --engine main-btree --suite write \
  --writers 1,4,16,32,64,128 --widths 1,4,16,25 \
  --distributions uniform,sequential,hotspot,same-leaf-heavy,different-leaf-heavy \
  --duration 30s --warmup 10s --repetitions 5 \
  --working-set 100000 --cache-capacity 4096 --group-limit 64 \
  --tokio-workers 8 --sync-mode real --seed 0x3a032026 \
  --output docs/experiments/results/phase3-oci/main-write.jsonl

target/release/phase0-bench --engine planned-blink --suite write \
  --writers 1,4,16,32,64,128 --widths 1,4,16,25 \
  --distributions uniform,sequential,hotspot,same-leaf-heavy,different-leaf-heavy \
  --duration 30s --warmup 10s --repetitions 5 \
  --working-set 100000 --cache-capacity 4096 --group-limit 64 \
  --tokio-workers 8 --sync-mode real --seed 0x3a032026 \
  --output docs/experiments/results/phase3-oci/planned-write.jsonl
```

The first OCI pass should use `main-btree` versus `planned-blink`. Keep
`serial-blink` as a code-level ablation/control and use `versioned-blink` only
when a Phase 3 regression needs attribution. The OCI matrix must additionally
cover the requested 1/16 reader smoke, Query/Scan read limits, 95/5 and 50/50
mixed workloads, real sync, and the required sync-delay controls. It is not a
Phase 3 Mac result and must not be inferred from the artifacts above.

## 12. Unresolved questions

- Whether the public transaction contract should gain an explicit symbolic
  same-batch revision dependency, rather than only the internal provisional
  token representation.
- How the planner and full-page image cost behave on the OCI A1 CPU and block
  volume with real sync latency.
- Whether PageCatalog cloning becomes a dominant cost at the OCI working set
  and whether a chunked copy-on-write catalog is justified.
- Which dependency partitions are sufficiently independent after accounting
  for overflow pages, page allocation, and structural fallback for Phase 4.
- Whether per-transaction page-image assembly remains the dominant CPU cost
  after page-local workers are introduced.

## 13. Gate status

Phase 3 implementation, focused correctness coverage, randomized differential
coverage, and development smoke/profiling are complete. On the final tree,
`cargo fmt --all -- --check` passed and `cargo test --workspace` passed,
including 70 `dodb-storage` unit tests, 8 Phase 2 recovery tests, 16 testkit
tests, 14 Phase 1 B-tree tests, and all doc tests. Phase 4 parallel page-local
writer implementation and OCI official benchmark execution are intentionally
outside this result.
