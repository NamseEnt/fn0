# Phase 2 Results — Optimistic/Versioned Multicore Reads

> Historical measurement notice: the performance numbers in this document
> were measured on the previous machine and remain historical only. The
> current Phase 2 baseline is in
> [`rebaseline-results.md`](rebaseline-results.md), with separate raw
> artifacts under
> `docs/experiments/results/rebaseline-macbookpro17-1-macos26.4-8c8t/phase2/`.
> The Phase 2 correctness, publication, and recovery results remain valid and
> are not invalidated by the machine change.

This is the Phase 2 companion to
[`b-link-batched-engine.md`](b-link-batched-engine.md). The goal was to keep
serial Blink writes unchanged while removing the Phase 1 benchmark adapter's
read mutex from `Get`, `Query`, and `Scan`.

Phase 3 batch planning, same-leaf coalescing, mutation workers, parallel
preparation, concurrent SMO optimization, adaptive batching, and WAL redesign
were not started.

## Scope and identity

| Item | Value |
| --- | --- |
| Starting branch HEAD | `23dc4b38b9249e2f7814c099866be100ef0a54a0` |
| Phase 1 historical implementation SHA | `0d023088f73fb0040e01a97c3897cf57dbd69773` |
| Phase 2 implementation/test source SHA used for benchmark | `0e8ac5caece809fc2f502443f4404357f83943f4` |
| Branch | `experiment/b-link-batched-engine` |
| Baseline selector | `main-btree` |
| Phase 1 control selector | `serial-blink` |
| Phase 2 selector | `versioned-blink` |
| Physical writer | one serial Blink coordinator and one serial mutation path |

The Phase 1 SHA above is historical metadata. All Phase 2 implementation work
was based on the requested branch HEAD, not on that earlier SHA. The raw JSONL
records preserve the Phase 2 source identity shown above.

## Chosen publication architecture

The implementation uses a safe immutable snapshot design:

```text
GenerationPublisher
  current: RwLock<Arc<PublishedGeneration>>
  PublishedGeneration
    epoch, root_page_id, high_water_page_id
    Arc<PageCatalog>
      PageId -> Arc<PageCell>
        PageVersion { epoch, Arc<BlinkPage> }
```

`BlinkStore::versioned_read_handle()` returns a cloneable `BlinkReadHandle`.
At the beginning of each operation it clones the current generation `Arc` and
releases the short publication `RwLock` immediately. The full traversal then
uses only immutable catalog/page objects. There is no traversal-wide global
lock and no benchmark adapter mutex on this path.

The selected synchronization primitive is a short standard-library
`RwLock<Arc<PublishedGeneration>>` snapshot combined with immutable `Arc`
page cells. No unsafe raw pointer, custom seqlock, or dependency was added.
The page cell does not need a local lock: a changed logical PageId is replaced
in the next immutable catalog, while old catalogs retain the old cell.

This is intentionally a conservative safe implementation rather than a
lock-free claim. The only global synchronization is the short generation
pointer snapshot/publication section; root-to-leaf traversal, sibling walking,
overflow materialization, and value copying happen after it is released.

## Atomic visibility and ordering proof

For a physical group containing several logical transactions, publication uses
the dirty-page union of every accepted candidate in the group. This matters
because the final candidate state contains all earlier accepted candidates,
not only the last candidate's dirty set.

The sequence is:

```text
serial candidate preparation and immutable catalog construction
        ↓
WAL page-image append
        ↓
WAL sync / durability point
        ↓
serial store state and metadata update
        ↓
single RwLock write section swaps Arc<PublishedGeneration>
        ↓
success response
```

Catalog/page objects prepared before the WAL sync are unreachable from the
published pointer and therefore invisible to readers. The write-lock release
of the pointer swap synchronizes with a later read-lock acquire for a reader's
`Arc` clone. This gives the required happens-before edge: every page cell and
root/catalog field reachable from generation `E+1` is fully initialized before
`E+1` becomes visible. A reader that already cloned `E` retains exactly `E`,
even if `E+1` or later generations publish while it traverses.

WAL sync failure marks the existing degraded store state and returns before the
pointer swap. The published generation is unchanged.

High-key/right-sibling correction is part of the versioned traversal. If a
page snapshot has `key >= high_key`, the reader follows that same pinned
generation's right link and continues checking the fence. There is no fake
optimistic retry counter: this implementation has no seqlock retry; it records
actual right-link corrections only.

## Page lifetime, reclamation, and reuse

`GenerationPin` increments active-pin counters and decrements them on `Drop`.
An old `PublishedGeneration` owns its catalog, its catalog owns old `Arc` page
cells, and a `PageCell` owns its immutable `Arc<BlinkPage>`. When the last old
generation reference disappears, the old cells are reclaimed by ordinary Arc
lifetime; no version is permanently retained to hide a leak.

The serial allocator now reuses a free-list PageId only when there are no
active generation pins. While any reader is pinned, freed overflow/page IDs
remain retired and allocation grows the high-water mark instead. After the
last pin releases, the existing free-list path can reuse those IDs. This is a
safe conservative rule: an old generation can never observe a reused PageId's
unrelated page object.

Metrics exposed by `BlinkVersionedReadMetrics` include generation pins,
maximum concurrent pins, page-version installs, retained/reclaimed versions,
retired/reusable page IDs, read operations, and right-link corrections. A
long Query/Scan is not cancelled or artificially limited; it simply retains
its generation until the operation returns. The tests deliberately keep a
generation pinned while writes and checkpoint proceed, then verify reclamation
after release.

## Checkpoint, reopen, and durability

Runtime generation/page-version information is not written to WAL or the
superblock. Checkpoint keeps the existing ordering: data flush and sync,
checkpoint metadata sync, then WAL reset. Reopen reconstructs fresh page cells
and a fresh runtime generation from the durable Blink pages plus committed WAL
replay. An active old reader uses immutable in-memory cells and is not
invalidated by checkpoint.

The existing serial transaction contract is unchanged: FIFO conditions and
revisions, transaction atomicity, logical ordering, full-page redo images,
one commit marker per logical transaction, group WAL sync, degraded behavior,
and recovery ordering remain in force.

## Correctness tests

The Blink unit suite contains 18 tests after Phase 2. The new coverage
includes:

- `versioned_generation_is_atomic_for_multi_page_transaction`: old readers see
  both old values and new readers see both new values, including a group with
  multiple logical transactions;
- `reader_during_unpublished_install_sees_only_old_generation`: prepared page
  cells remain invisible until the generation pointer is swapped;
- `root_split_and_leaf_links_are_safe_for_pinned_reader`: an old root remains
  usable while a new root and many leaf splits publish;
- `high_key_right_link_correction_finds_stale_route`: stale B-link route
  correction remains active on the read path;
- `page_reuse_is_delayed_until_pinned_generation_releases`: retired overflow
  PageIds are not reused while an old reader can reach them;
- `query_and_scan_keep_one_generation_during_publication`: range results stay
  entirely old or new across a concurrent publication;
- `wal_sync_failure_does_not_publish_a_generation`: a sync fault exposes no
  new state;
- `query_scan_tombstone_overflow_checkpoint_and_reopen`: checkpoint and
  reopen preserve versioned reads;
- `concurrent_versioned_readers_survive_serial_writer_publications`: four
  concurrent readers and one serial writer complete without panic/deadlock;
- `randomized_concurrent_reads_and_serial_writes_preserve_ordering`: seed
  `0x2a022026`, six readers, 1,000 randomized operations each, and 300
  serial Put/Delete publications verify ordered Query/Scan results;
- the existing Phase 1 randomized differential test remains in place with
  seed `0x51a12026` and 500 operations.

Root split, leaf split/right-link, multi-page visibility, pre-publication
installation, page reuse, WAL failure, checkpoint/reopen, and recovery all
passed. No read coordinator groups were recorded in read-only benchmark runs.

## Benchmark methodology

The same Phase 0 harness and workload generator were used for all three
engines:

| Parameter | Value |
| --- | --- |
| Build | `cargo build --release -p dodb-storage --bin phase0-bench` |
| Runtime | Tokio multi-thread, 12 workers |
| Machine | AMD Ryzen 5 5600, 6 physical / 12 logical CPUs |
| Warmup / measurement | 1 s / 2 s |
| Repetitions | 3; tables report median repetition |
| Key/value | 16-byte key, 64-byte inline value |
| Cache / working set | 256 pages / 4,096 rows |
| Read limit | 16 |
| Seed base | `0xd0db2026` |
| Read/write durability | real `ProductionFile` sync |

Read scaling used 1/4/16/32/64/128 readers for each Get, Query, and Scan.
Mixed runs used `(16,16)`, `(64,16)`, `(16,64)`, `(64,64)` reader/writer
pairs for 95/5 and 50/50. Write control used writers 1/16/64, widths 1/16,
uniform distribution. All official samples reported zero errors and zero
overloads.

Raw artifacts:

```text
docs/experiments/results/phase2-versioned-reads/main-read.jsonl
docs/experiments/results/phase2-versioned-reads/serial-blink-read.jsonl
docs/experiments/results/phase2-versioned-reads/versioned-blink-read.jsonl
docs/experiments/results/phase2-versioned-reads/main-mixed.jsonl
docs/experiments/results/phase2-versioned-reads/serial-blink-mixed.jsonl
docs/experiments/results/phase2-versioned-reads/versioned-blink-mixed.jsonl
docs/experiments/results/phase2-versioned-reads/serial-blink-write.jsonl
docs/experiments/results/phase2-versioned-reads/versioned-blink-write.jsonl
```

## Read scaling

Throughput is requests/s, median of three repetitions. `M` means million
requests/s.

### Get

| Readers | main-btree | serial-blink | versioned-blink |
| ---: | ---: | ---: | ---: |
| 1 | 1.840M | 0.209M | 1.532M |
| 4 | 4.762M | 0.125M | 3.534M |
| 16 | 6.245M | 0.124M | 4.774M |
| 32 | 8.517M | 0.124M | 4.654M |
| 64 | 8.519M | 0.123M | 4.709M |
| 128 | 8.525M | 0.124M | 4.465M |

Versioned Get scales 3.12x from one to 16 readers and 2.91x from one to 128
readers. The serial control is effectively flat after one reader because the
Phase 1 adapter mutex is held for the entire read operation.

### Query, limit 16

| Readers | main req/s | serial req/s | versioned req/s | versioned rows/s |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 0.473M | 0.173M | 0.467M | 7.477M |
| 4 | 1.149M | 0.117M | 1.222M | 19.547M |
| 16 | 1.903M | 0.103M | 1.991M | 31.859M |
| 32 | 2.014M | 0.107M | 1.817M | 29.065M |
| 64 | 2.008M | 0.106M | 1.781M | 28.503M |
| 128 | 1.977M | 0.103M | 1.866M | 29.858M |

### Scan, limit 16

| Readers | main req/s | serial req/s | versioned req/s | versioned rows/s |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 0.475M | 0.181M | 0.552M | 8.833M |
| 4 | 1.248M | 0.119M | 1.365M | 21.837M |
| 16 | 2.094M | 0.107M | 2.104M | 33.660M |
| 32 | 2.212M | 0.106M | 2.015M | 32.241M |
| 64 | 1.637M | 0.107M | 2.027M | 32.432M |
| 128 | 2.089M | 0.108M | 2.067M | 33.069M |

Versioned read p95/p99 medians were `0.73/0.85 us` for Get at one reader and
`1.93/2.13 us` at 128 readers; Query ranged from `2.60/2.79 us` to
`3.88/4.15 us`; Scan ranged from `1.59/1.79 us` to `3.33/3.59 us`. The
complete p50/p95/p99 values for every reader count are in the raw artifacts.
CPU utilization reached approximately 62--65% of the 12-logical-CPU machine
in the 16--128 reader versioned runs. Read-only coordinator groups remained
zero.

Against `main-btree`, versioned Blink is not yet the fastest read view: at 16
readers it reached 76% of main Get, 105% of main Query, and 100% of main Scan.
The Phase 2 attribution target was the Phase 1 serial Blink mutex, and that
bottleneck was removed: at 16 readers versioned Get/Query/Scan were about
38.6x/19.4x/19.6x the serial control.

## Mixed workload

Values are median requests/s across three repetitions. `tx/s` is successful
logical write transactions and `read/s` is successful Get throughput.

| Readers | Writers | Mix | main tx/read | serial tx/read | versioned tx/read | versioned p95 / p99 (us) |
| ---: | ---: | --- | ---: | ---: | ---: | ---: |
| 16 | 16 | 95/5 | 1,235 / 23,494 | 330 / 6,312 | 342 / 6,549 | 1,930 / 59,532 |
| 16 | 16 | 50/50 | 1,453 / 1,473 | 372 / 373 | 370 / 373 | 59,032 / 81,494 |
| 64 | 16 | 95/5 | 1,427 / 27,158 | 345 / 6,598 | 337 / 6,451 | 5,978 / 62,497 |
| 64 | 16 | 50/50 | 1,580 / 1,597 | 364 / 374 | 354 / 373 | 61,387 / 80,368 |
| 16 | 64 | 95/5 | 2,715 / 51,594 | 552 / 10,499 | 408 / 7,763 | 4,841 / 183,223 |
| 16 | 64 | 50/50 | 3,198 / 3,208 | 596 / 615 | 437 / 444 | 178,591 / 209,013 |
| 64 | 64 | 95/5 | 2,732 / 51,914 | 575 / 10,972 | 390 / 7,434 | 6,216 / 180,415 |
| 64 | 64 | 50/50 | 3,651 / 3,662 | 643 / 649 | 408 / 423 | 168,620 / 200,416 |

Versioned reads no longer wait on the serial Blink store mutex, but the
single serial writer is now more visible: immutable catalog construction and
publication reduce write throughput in these durable mixed runs. Versioned
read p50 is generally low, while p95/p99 is dominated by serial-writer
publication/queue pressure in balanced cases. No read coordinator group or
read overload was recorded.

## Serial Blink vs versioned Blink write control

| Writers | Width | serial tx/s | versioned tx/s | versioned installs / reclaimed | versioned p95 (us) |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 1 | 142 | 140 | 281 / 281 | 7,858 |
| 1 | 16 | 117 | 121 | 3,765 / 3,763 | 10,011 |
| 16 | 1 | 681 | 672 | 1,317 / 1,317 | 30,100 |
| 16 | 16 | 472 | 460 | 9,960 / 9,959 | 50,198 |
| 64 | 1 | 980 | 1,006 | 1,811 / 1,811 | 71,853 |
| 64 | 16 | 646 | 655 | 4,985 / 4,983 | 128,866 |

The write control does not claim a speedup. Medians are close to the serial
control, with publication/version tracking measurable in raw counters. With
no long-lived readers in these write-only runs, nearly all installed old
cells were reclaimed by the end of the interval and retained versions were
the current catalog (about 243 pages in the 4,096-row setup).

## Contention and retention observations

- `groups=0` for every read-only main/serial/versioned scenario. Versioned
  reads bypass both the writer coordinator and the store mutex.
- Versioned read `max_concurrent_pins` tracked reader concurrency; the
  read-only runs reclaimed temporary generation pins immediately and retained
  only the current catalog.
- The deterministic pinned-reader tests show that old versions are retained
  while a reader is held, and page reuse resumes only after release.
- No optimistic retries are reported because there is no seqlock in this safe
  implementation. `right_link_corrections` is the only route-correction
  counter and remained zero in the uniform read matrix; the stale-route unit
  test exercises it directly.
- The main remaining read-side costs are immutable page/catalog traversal,
  Arc/page lookup and value materialization. The main remaining write-side
  cost is the existing serial state clone plus generation catalog cloning and
  page-cell installation.

## Gate results

Passed:

```text
cargo fmt --all -- --check
cargo test --workspace
```

The final workspace gate was run after the Phase 2 tests and documentation
were added. It includes the existing recovery, service, testkit, and Phase 1
differential suites in addition to the 18 Blink unit tests listed above.

The read path satisfies the Phase 2 structural gates: no traversal-wide
global lock, one generation per operation, atomic multi-page publication,
pre-publication invisibility, B-link correction, root/leaf split safety,
delayed PageId reuse, eventual Arc reclamation, WAL-failure no-publication,
checkpoint/reopen, and randomized concurrent read/write stress.

## Phase 3 design input and unresolved questions

Phase 2 confirms that generation publication is a usable atomic boundary for
future multi-page commits, but it also exposes the costs that Phase 3 must
address. The current serial state clone and immutable catalog cloning remain a
large part of write cost. The catalog shape is compatible with a future
planner receiving a serial candidate plus changed-page set, but no planner or
same-leaf representation should be inferred from this phase.

Before Phase 3, profile and decide:

- whether catalog construction can use a chunked/dense PageId structure
  without weakening old-generation lifetime;
- whether publication can keep the same manifest semantics while reducing
  write-side cloning;
- how a future batch planner will represent logical transaction boundaries and
  changed-page dependencies;
- how long-running scans should be observed or backpressured if a deployment
  needs a hard memory bound;
- whether versioned Blink should be compared against an optimized immutable
  main read view before any adoption decision.

No Phase 3 implementation was started in this work.
