# Phase 1 Results — Serial B-link Format and Correctness Control

> Historical measurement notice: the performance numbers in this document
> were measured on the previous machine and remain historical only. The
> current Phase 1 control baseline is in
> [`rebaseline-results.md`](rebaseline-results.md), with separate raw
> artifacts under
> `docs/experiments/results/rebaseline-macbookpro17-1-macos26.4-8c8t/phase1/`.
> The Phase 1 correctness and format results remain valid and are not
> invalidated by the machine change.

This record is the Phase 1 companion to
[`b-link-batched-engine.md`](b-link-batched-engine.md). It describes the
experimental engine at the historical implementation commit
`0d023088f73fb0040e01a97c3897cf57dbd69773` and the focused serial control
runs. Phase 2 intentionally started from the later requested branch HEAD
`23dc4b38b9249e2f7814c099866be100ef0a54a0`; this document is not the identity
of the current branch. It does not implement or claim Phase 2 optimistic
reads, epoch reclamation, multi-writer page workers, batch planning, same-leaf
coalescing, concurrent SMO, or WAL redesign.

## Scope and identity

| Item | Value |
| --- | --- |
| Baseline main commit | `1ff96e1b3d205074d4c1b820f5f2680bd3226a8b` |
| Phase 0 document commit | `49f262d0e97a8681cd48b8b1ad22b1ada5b86f4f` |
| Phase 1 implementation/control commit | `0d023088f73fb0040e01a97c3897cf57dbd69773` |
| Baseline engine selector | `main-btree` |
| Experimental engine selector | `serial-blink` |
| Physical writer | one serial Blink mutation worker |
| Read path in this phase | serial control adapter; no multicore-read claim |

The existing `btree` module and its on-disk format remain the baseline. The
experimental implementation is a separate `crates/dodb-storage/src/blink`
module. The benchmark-local adapter selects either engine without changing the
baseline engine API or storage behavior.

## Experimental format decision

The experimental database has an explicit `DBLK` superblock identity and
format version `2`. The baseline `DSBK` decoder and the Blink decoder reject
each other before a non-empty database can be opened. The superblock is still
two 4096-byte copies, but its fields are independent:

| Offset | Size | Field |
| ---: | ---: | --- |
| 0 | 4 | magic `DBLK` |
| 4 | 2 | superblock version `2` |
| 8 | 8 | generation |
| 16 | 16 | database UUID |
| 32 | 8 | tenant ID |
| 40 | 8 | shard ID |
| 48 | 8 | shard epoch |
| 56 | 4 | page size, must be 4096 |
| 60 | 8 | root page ID |
| 68 | 8 | free-list head or explicit NULL `u64::MAX` |
| 76 | 8 | high-water page ID |
| 84 | 8 | checkpoint LSN |
| 92 | 4 | reserved, must be zero |
| 96 | 4 | engine tag `BLNK` |
| 100 | 4 | CRC32C, calculated with this field zeroed |
| 104..4096 | — | reserved, must be zero |

Data pages reuse the existing fixed 4096-byte physical page header, including
page ID, page LSN, page type, and CRC32C. Their body has Blink layout version
`2` and distinct `BLKL`, `BLKI`, `BLKO`, and `BLKF` magic values, so a baseline
body decoder cannot silently interpret it.

Leaf and internal bodies are strict slotted pages. A page stores a nullable
high-key offset/length and a nullable right-sibling page ID. A missing high key
is the explicit `+infinity` representation; it is never a user key. Decoder
checks include body version, flags, reserved bytes, slot bounds, canonical
document keys, record overlap, and page ID validity.

## High-key and right-link invariants

For every leaf and internal page, normal routing is valid only for
`key < high_key` when the high key is finite. If the test is false, traversal
follows `right_sibling` and tests the destination again. The sibling is always
the same tree level. Leaf links are the ordered Query/Scan chain.

The checker verifies that finite fences have a right sibling, rightmost pages
have `+infinity` and no sibling, sibling targets are not self-links or cycles,
sibling levels match, ranges increase, and the leaf chain is exactly the leaf
order implied by the parent tree. A deterministic stale-parent-like test
starts at the left leaf, crosses its finite fence, and finds the key in the
right leaf without consulting a newer parent separator.

## Split implementation

Leaf insertion first creates a sorted candidate page. If it does not fit, the
candidate is split into non-empty lower and upper halves:

```text
left.high_key       = first key in right
left.right_sibling  = new_right
right.high_key      = old left.high_key
right.right_sibling = old left.right_sibling
```

The separator is then installed synchronously in the parent. Internal pages
use the same level/fence/link rule; the middle separator is promoted, the
right page starts at the promoted separator's right child, and cascading
splits are handled up the path. A root split allocates a new higher-level root
and increments the root split counter. Deletes retain tombstones and do not
merge or rebalance pages. Overflow values use their own chained pages; old
chains are moved to the free list and allocation can reuse free pages.

The control exposes `leaf_splits`, `internal_splits`, `root_splits`,
`right_link_corrections`, `pages_touched`, and `page_images`. These are
Phase 1 counters only; no page-worker/latch/version retry counters were added.

## Checker coverage

`BlinkStore::check_invariants` checks:

- page ID/range ownership, root and high-water metadata, and page types;
- strict leaf key and internal separator ordering;
- canonical keys and `key < high_key` fences;
- parent-child ranges and tree reachability/cycle rejection;
- same-level right links, range continuity, rightmost termination, and leaf
  chain ordering;
- overflow chain length, ownership, cycles, and multiply-owned pages;
- free-list type, cycles, overlap, leaked pages, and allocator coverage.

Physical page decode checks the page header checksum, page ID, body version,
reserved bytes, slot ranges, and record checksums through the shared low-level
codec. Negative tests cover bad body version, checksum, sibling cycle, wrong
sibling level/type, unordered separators, and invalid format identity.

## Transaction and WAL integration

`apply_transaction_group` clones the committed state as a serial working
overlay. Requests are validated and condition-checked in FIFO order. An
accepted request mutates a private candidate; the next request validates
against that candidate. A failed request contributes neither mutations nor
state visible to later requests. The final state is published only after the
WAL group is durable.

The implementation preserves `RevisionEquals`, `Exists`, `NotExists`, missing
revision `0`, delete tombstone revisions, duplicate mutation/condition
rejection, multi-key atomicity, one commit LSN per logical transaction, one
explicit `COMMIT` record per transaction, FIFO page-image ordering, and one
shared WAL sync for the physical group.

Provisional revisions are restamped to the final commit LSN after the dirty
page count is known. Restamping is restricted to the transaction's mutated
canonical keys; this is important because a provisional numeric value can
otherwise equal an older committed revision after a split. Every changed data
page receives the transaction commit LSN in its page header. The WAL frame
protocol is unchanged. `WalPageImageFormat::ExperimentalBlink` is an explicit
validation seam for Blink superblocks and page bodies; it never guesses the
format from the image.

WAL-backed open replays only complete committed groups, handles a torn WAL
tail through the existing scanner, writes full-page redo images, and loads the
highest valid Blink superblock. Flush/checkpoint ordering remains WAL first,
then data pages, then checkpoint metadata, then WAL reset. No runtime epoch,
reader pin, or page sequence metadata exists yet.

## Code split and benchmark adapter

Shared with `main`:

- `DurableFile`, `ProductionFile`, fixed page header/CRC32C primitives;
- `WalLog` frame protocol, commit records, group sync, identity, and fault
  injection seam;
- core keys, revisions, transaction conditions, and public batch request types.

Separate in `blink`:

- superblock identity and codec;
- leaf/internal/overflow/free body codec;
- tree state, B-link route correction, serial split path, allocator, checker,
  query/scan traversal, and recovery loader.

The `phase0-bench` `EngineAdapter` now supports `--engine main-btree` and
`--engine serial-blink`. The Blink benchmark adapter has a benchmark-local
serial coordinator with the same queue/group-limit/collection-delay controls
and uses `apply_transaction_group`, so it does not accidentally pay one WAL
sync per request. Reads in the Blink adapter take the serial control mutex;
this is intentional for Phase 1 and is not a Phase 2 scalability result.

## Tests and randomized differential coverage

The Phase 1 additions include:

- baseline/experimental open rejection in both directions;
- high-key/right-link correction;
- page format version/checksum rejection;
- leaf, internal, and root split stress;
- Query/Scan ordering, exclusive cursor behavior, tombstones, and overflow;
- FIFO A/B/C transaction dependency and failed-transaction atomicity;
- checkpoint, close, reopen, WAL replay, and free/overflow ownership;
- checker negative tests for cycle, wrong level/type, and unordered separator;
- a deterministic randomized differential sequence with seed
  `0x51a12026`, 500 operations, 100-key domain, Put/Delete/Get, and a
  reference `BTreeMap` model. The failure message includes seed and operation.

The repository gate passed:

```text
cargo test --workspace
```

At the Phase 1 implementation commit this ran 54 storage tests, including the
Blink tests, all Phase 1 recovery tests, and the existing workspace/server/
testkit suites. Phase 2-specific tools were not introduced.

## Serial control benchmark methodology

The focused runs use the same release binary, machine, key/value shape,
preseed strategy, deterministic seed, runtime, and raw JSONL collector for
both engines. The measured policy is:

| Parameter | Value |
| --- | --- |
| Build | release |
| Runtime | Tokio multi-thread, 12 workers for write/read; 4 for structural stress |
| Warmup | 500 ms per repetition |
| Measurement | 1 s per repetition for write/read; 500 ms structural stress |
| Repetitions | 3; tables use the median repetition |
| Cache | 256 |
| Working set | 4,096 for write/read; 128 for structural stress |
| Key/value | 16-byte key/64-byte inline value for write/read |
| Collection delay | 0 |
| Artificial sync delay | 0 |
| Files | fresh `ProductionFile` data/WAL files per repetition |
| Seed bases | `0xd0db2026b1000001`, `0xd0db2026b1001001`, `0xd0db2026b1002001` |

The Phase 1 write/read runs use `sync_mode=injected --sync-delay 0`. In this
harness setting the injected delay is zero but `BenchFile::sync_data` still
calls the real production file sync, so both engines have the same actual
sync path. The run uses the current machine's default temporary-file location;
its absolute sync timing must not be compared directly with the Phase 0 runs
that used `target/phase0/tmp`. The raw record reports both engine/tree metrics
and WAL append/sync metrics; durable latency must be interpreted separately.

Raw artifacts, retaining every repetition and machine metadata, are checked in
under:

```text
docs/experiments/results/phase1-serial-blink/main-write.jsonl
docs/experiments/results/phase1-serial-blink/serial-write.jsonl
docs/experiments/results/phase1-serial-blink/main-read.jsonl
docs/experiments/results/phase1-serial-blink/serial-read.jsonl
docs/experiments/results/phase1-serial-blink/main-structural.jsonl
docs/experiments/results/phase1-serial-blink/serial-structural.jsonl
```

Representative commands are:

```text
cargo build --release -p dodb-storage --bin phase0-bench

target/release/phase0-bench --engine main-btree \
  --suite write --writers 1,16,64 --widths 1,16 \
  --distributions uniform,sequential --duration 1s --warmup 500ms \
  --repetitions 3 --working-set 4096 --key-size 16 --value-size 64 \
  --sync-mode injected --sync-delay 0 --tokio-workers 12 \
  --seed 0xd0db2026b1000001 --output main-write.jsonl

target/release/phase0-bench --engine serial-blink \
  --suite read --readers 1,16 --read-kinds get,query,scan \
  --duration 1s --warmup 500ms --repetitions 3 \
  --working-set 4096 --key-size 16 --value-size 64 \
  --sync-mode injected --sync-delay 0 --tokio-workers 12 \
  --seed 0xd0db2026b1001001 --output serial-read.jsonl
```

The write matrix is writers `1/16/64`, widths `1/16`, distributions
`uniform/sequential`. The read matrix is readers `1/16`, separately for Get,
short Query, and short Scan. Structural stress uses one sequential writer,
width 1, `insert-if-absent`, 128 preseed rows, 512-byte key shape, and records
actual leaf/internal/root split counters. It is not called a structural test
unless those counters are non-zero.

## Main versus serial Blink benchmark

The following are median values across three repetitions. Throughput is
logical transactions/s except `mutation/s`; latency is end-to-end request
latency in microseconds. The complete 12-scenario write matrix remains in raw
JSONL.

### Width 1, uniform write scaling

| Writers | Main tx/s | Serial Blink tx/s | Main p50/p95/p99 | Blink p50/p95/p99 |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 10,192 | 853 | 18 / 50 / 54 | 1,169 / 1,210 / 1,269 |
| 16 | 8,568 | 1,427 | 256 / 685 / 753 | 11,288 / 12,153 / 21,486 |
| 64 | 7,733 | 1,288 | 1,013 / 2,547 / 2,702 | 48,439 / 59,451 / 95,758 |

### Width 16, uniform write control

| Writers | Main tx/s | Main mutation/s | Serial Blink tx/s | Blink mutation/s |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1,295 | 20,715 | 673 | 10,764 |
| 16 | 1,130 | 18,073 | 1,020 | 16,319 |
| 64 | 1,021 | 16,329 | 898 | 14,368 |

The serial control is slower at width 1 and has materially higher tail
latency. This is expected for an unoptimized full-state clone/serial
implementation and is a measured cost, not something deferred to a presumed
future parallel speedup.

### Read control

| Operation/readers | Main ops/s | Serial Blink ops/s | Main p50/p95/p99 us | Blink p50/p95/p99 us |
| --- | ---: | ---: | ---: | ---: |
| Get / 1 | 1,970,750 | 1,629,295 | 0.4 / 0.5 / 0.5 | 0.5 / 0.6 / 0.6 |
| Get / 16 | 7,656,576 | 1,048,712 | 0.7 / 0.9 / 1.0 | 0.8 / 52.5 / 55.0 |
| Query / 1 | 473,935 | 478,448 | 1.7 / 1.8 / 1.9 | 1.7 / 1.8 / 1.8 |
| Query / 16 | 2,165,433 | 325,842 | 2.7 / 3.2 / 3.2 | 2.8 / 80.0 / 115.8 |
| Scan / 1 | 448,547 | 549,374 | 1.8 / 1.9 / 2.0 | 1.4 / 1.6 / 1.7 |
| Scan / 16 | 2,277,257 | 360,359 | 2.6 / 2.9 / 3.0 | 2.8 / 75.3 / 79.1 |

The 16-reader Blink result is the expected serial-control limitation: the
benchmark reader tasks contend on one adapter mutex. It confirms that the
current Phase 1 engine must not be used to infer Phase 2 multicore read
scalability. It also gives Phase 2 a direct control target: preserve the
single-reader traversal cost while removing the global serial adapter path.

### Structural stress

| Engine | tx/s | p50/p95/p99 us | measured leaf/internal/root splits | total leaf/internal/root splits |
| --- | ---: | ---: | ---: | ---: |
| Main B+Tree | 10,041 | 63 / 133 / 175 | not instrumented | not instrumented |
| Serial Blink | 913 | 241 / 864 / 1,004 | 152 / 40 / 1 | 338 / 81 / 4 |

The Blink structural run genuinely exercised leaf and internal splits. Root
splits occurred during preseed and measurement; both measured and cumulative
counters are retained so preseed work is not silently mistaken for measured
work.

## Current bottlenecks and Phase 2 implications

1. The dominant Phase 1 write cost is not B-link correction. It is serial
   candidate-state cloning, repeated full-page encoding, and the absence of a
   read view/parallel page execution. Width-1 Blink is about an order of
   magnitude below the main control under this file/sync environment.
2. The benchmark-local Blink coordinator restores group commit fairness, but
   it remains a single physical writer. Its collection/group metrics are
   comparable to the baseline; Blink's page counters are additional.
3. The Phase 1 read adapter serializes all Blink reads. Query and Scan are
   close to main at one reader, while 16-reader p95/p99 grows to tens or over
   100 microseconds. This is an adapter/control limitation that Phase 2 must
   remove, not a B-link traversal conclusion.
4. High-key correction was not a frequent event in the normal benchmark
   (`right_link_corrections` remained zero), which is expected because Phase 1
   updates parents synchronously. The deterministic correction test is still
   required for the future stale-path proof.
5. Full-page WAL remains the visible durability boundary. No WAL amplification
   conclusion can be drawn from this phase beyond the recorded page-image and
   WAL-byte counters.

The recommended Phase 2 design consequence is to retain the durable page
codec, fences, sibling links, and checker, then add an explicit page-local
version/read-view mechanism around the same state transitions. Do not start by
parallelizing the current state clone: it would preserve the measured copy and
publication bottleneck while making correctness harder to prove.

## Unresolved questions for Phase 2

- `page-local RwLock` versus atomic immutable page cells/epoch primitive;
- atomic publication mechanism for a multi-page committed transaction;
- version lifetime and safe free-page reuse;
- long Query/Scan epoch-pin policy and bounded memory behavior;
- whether the serial coordinator should remain the structural-modification
  fallback after read concurrency is added;
- how much of the measured Blink overhead is removed by page-local reads alone
  before any multi-writer mutation work is attempted.

No Phase 2 implementation was started in this change.
