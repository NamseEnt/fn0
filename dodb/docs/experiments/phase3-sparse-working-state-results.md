# Phase 3.1 Sparse Working State Results

## Motivation

The Phase 3 OCI attribution measured 632.706 ms of whole-state clone time and
280.772 ms of state installation per ws100k run. At 6,074.8 mutation ops/s,
planned Blink reached 0.530x the main baseline. This phase removed the planned
path's whole-`BlinkState` clone and replacement while preserving the WAL and
per-transaction page-image boundaries.

## Implementation

### WorkingBlinkState

`WorkingBlinkState` keeps an immutable reference to the committed state, scalar
metadata copied at group start, and a `BTreeMap` containing only modified or
new pages. Page reads check the overlay first and the committed map second;
insertions affect only the overlay. Dirty-page restamping requires the page to
already exist in the overlay.

### BlinkMutationState

The internal `BlinkMutationState` abstraction is implemented by both
`BlinkState` and `WorkingBlinkState`. The serial and planned paths continue to
share route lookup, mutation, allocator, overflow, leaf split, separator
installation, internal split, and page-level helpers. The serial path retains
its clone-based transaction behavior.

### WAL-before-install atomicity

Each accepted logical transaction still produces its own intermediate page
images and `COMMIT` record in FIFO order. After generation preparation and WAL
assembly, the working state is consumed into a sparse delta before WAL append
and sync. Only after successful WAL group sync are changed pages inserted into
the committed page map and scalar metadata updated; generation publication
follows. A sync failure test verifies that logical contents, encoded committed
pages, root/free-list/high-water metadata, superblock, revision/LSN/batch IDs,
and the published generation remain unchanged.

### Dirty-only generation preparation

The planned path clones the existing catalog map, then visits only the union
of group dirty pages to replace or add `PageCell`s. It does not materialize a
complete `BlinkState` or scan every committed page. The existing serial
`prepare()` method and `PageCatalog` representation are unchanged.

## Correctness

- `cargo fmt --all -- --check`, `cargo test -p dodb-storage`,
  `cargo test -p dodb-storage --bin phase0-bench`, `cargo test --workspace`,
  and `git diff --check` passed at implementation commit
  `2621e8e052b02cf6c4706edbfe6b6f82e2773774`.
- Existing planned WAL-boundary, reroute, split, root/internal split,
  multi-page atomic publication, pinned-reader, page-reuse, randomized
  differential, and structural stress tests passed.
- `planned_group_uses_sparse_working_state` verifies a tree with more than 20
  committed pages has an overlay of at most 3 pages for a point update, checks
  that every overlay page is dirty, and verifies zero full-state clone count
  and clone time for a planned group.
- The old ws100k attribution recorded a median of 54 full-state clones across
  54 groups (1 clone/group); the sparse ws100k records show 0 clones across
  120, 120, and 113 groups.
- `planned_wal_failure_does_not_install_working_delta` verifies committed
  state and publication metadata remain unchanged after injected WAL sync
  failure.
- All three OCI no-sync records for each working set and all three real-sync
  records report zero errors, overloads, full-state clones, and state-clone
  nanoseconds.

## OCI Diagnostic Method

The sparse binary was built in release mode on the OCI A1 2 OCPU, 12 GiB
instance at commit `2621e8e052b02cf6c4706edbfe6b6f82e2773774`. Runs used 16
writers, width 16, uniform distribution, 2 Tokio workers, 1 s warmup, 2 s
measurement, and 3 repetitions. The no-sync runs used the harness's true
sync-disabled mode: WAL and buffered writes remain, while `sync_data()` and
`sync_all()` do not call the underlying filesystem sync. These are engine and
CPU diagnostics, not durability benchmarks. Files remained on the root
filesystem; the results do not represent the unmounted 200 GB target volume.

### No-Sync Performance

Throughput is the median of three repetitions; spread is min..max.

| working set | old planned | sparse planned | speedup | main baseline | sparse/main |
|---:|---:|---:|---:|---:|---:|
| 100,000 | 6,074.8 | 13,773.5 (12,927.8..13,821.0) | 2.267x | 11,467.8 | 1.201x |
| 4,096 | 15,730.2 | 16,552.5 (16,261.8..16,598.1) | 1.052x | 23,441.4 | 0.706x |

Throughput is in mutation ops/s. At ws100k, sparse planned exceeded the strong
success thresholds: at least 1.50x the old planned result and at least 0.80x
the main baseline. At ws4096, throughput increased 5.23%; this is not a small
working-set regression.

## Cost Breakdown

Median accumulated timing per measured run. `Reduction` is `(before - after) /
before`; a negative value means the accumulated time increased. The new run
completed substantially more mutations in the same duration, so accumulated
component times are not per-operation costs. Catalog subcomponents are nested
within catalog construction, and retired-generation drop is nested within
generation publication.

| component | before ms | after ms | reduction |
|---|---:|---:|---:|
| processing total | 1,974.088 | 1,923.288 | 2.6% |
| state clone | 632.706 | 0.000 | 100.0% |
| state install | 280.772 | 67.801 | 75.9% |
| physical execution | 207.909 | 518.890 | -149.6% |
| planning | 147.881 | 451.970 | -205.6% |
| catalog construction | 87.597 | 123.590 | -41.1% |
| └ catalog map clone | 26.083 | 54.189 | -107.8% |
| └ catalog state scan | 61.434 | 68.637 | -11.7% |
| WAL assembly | 23.280 | 49.037 | -110.6% |
| WAL append | 188.448 | 439.451 | -133.2% |
| generation publication | 207.187 | 99.399 | 52.0% |
| └ retired generation drop | 207.124 | 99.301 | 52.1% |
| dirty tracking | 16.228 | 37.281 | -129.7% |

The sparse run's state-clone fields are exactly zero. The changed-page install
time fell by 212.971 ms per run. Catalog map cloning and retired-generation
lifecycle remain present by design.

## Small-State Regression Check

The ws4096 median was 16,552.5 mutation ops/s versus 15,730.2 before, a 5.23%
increase. It clears the 10% regression gate of 14,157.18 mutation ops/s.

## Real-Sync Sanity

The ws100k no-sync median was 13,773.5 mutation ops/s, above the 9,112
threshold, so the conditional real-sync run was performed. Sparse planned
measured 10,447.1 mutation ops/s median (10,199.4..10,537.7) across three
repetitions. This was a planned-only sanity run; main was not rerun.

## Remaining Bottleneck

Using ws100k processing total as the denominator, the largest top-level
contributions were WAL assembly + WAL append + dirty tracking at 27.34%,
physical execution at 26.98%, and planning at 23.50%. Catalog construction
plus generation publication accounted for 11.59%. The next engineering
priority is the WAL/page-image path, which is the largest remaining serial
component group under the predefined 25% rule. No further optimization or
benchmark was run in this phase.

## Phase 4 Readiness

**Ready to proceed to Phase 4 design; Phase 4 implementation has not started.**
Correctness gates passed, `full_state_clones` is zero, ws100k achieved 2.267x
old planned and 1.201x main, and ws4096 did not regress by 10%.

## Artifacts

- [ws100k true no-sync](results/oci-a1-2ocpu-12g-200g/phase3/planned-sparse-nosync-w16-width16-ws100k.jsonl)
- [ws4096 true no-sync](results/oci-a1-2ocpu-12g-200g/phase3/planned-sparse-nosync-w16-width16-ws4096.jsonl)
- [ws100k real-sync sanity](results/oci-a1-2ocpu-12g-200g/phase3/planned-sparse-real-w16-width16-ws100k.jsonl)

All three artifacts contain three valid records at the sparse implementation
commit. Their SHA256 values were identical between OCI and the local copies.

## OCI Artifact Preservation

Before synchronization, the two untracked attribution artifacts had SHA256:

- ws100k: `759dc1edeb671d3cb94a76b67d418d8072d656b33d860bb68a4a154d2a6ebe57`
- ws4096: `6e1e2eb96215820ae4043fee52feb699cd3cfea5b400d52e1e801b481acc947e`

Backup: `/home/opc/dodb-oci-artifacts-pre-sparse-2621e8e/phase3/`.

After fast-forward, the tracked attribution artifacts had identical SHA256
values to both the pre-sync values and the preserved backups. No prior backup
directory was changed.
