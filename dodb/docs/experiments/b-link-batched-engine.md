# B-link + Batched Storage Engine Experiment

Status: Phase 4 persistent worker pool implementation and correctness are
complete. The OCI delay-zero diagnostic found insufficient worker overlap and
a throughput regression; the worker mechanism needs further review. Phase 5
has not started. See [`phase4-results.md`](phase4-results.md).

The independent serial generation/catalog lifecycle optimization is recorded in
[`catalog-cow-results.md`](catalog-cow-results.md). It does not change the
Phase 4 conclusion or imply Phase 5 readiness.

The WAL physical group-write result is recorded in
[`wal-group-write-results.md`](wal-group-write-results.md). The B-link +
batching architecture is unchanged; this optimization only aggregates the
physical WAL writes of an already logically batched group.

WAL group encoding costs are attributed in
[`wal-group-encoding-attribution.md`](wal-group-encoding-attribution.md). WAL
format and semantics are unchanged.

The direct WAL group-buffer implementation and OCI result are recorded in
[`direct-wal-buffer-results.md`](direct-wal-buffer-results.md). The default
width-1 result is a structural success: temporary payload, digest, and frame
buffers and their copies are removed, group encode is 24.23% lower per
mutation, and throughput is 6.69% higher. The separate `+crc` control is 5.91%
faster than the default direct-buffer build. WAL byte format, commit digest
semantics, fault-injection path and hook order, strict/trusted validation
semantics, and recovery format are unchanged. Target-specific AArch64 CRC
codegen is the next engineering investigation; shared descent and arenas remain
later candidates.

The page-CRC reuse experiment is recorded in
[`wal-crc-combine-results.md`](wal-crc-combine-results.md). Combine preserved
WAL bytes, but local digest-combine cost increased sharply, so the candidate
was rejected and the OCI run was skipped.

The AArch64 CRC caller-specialization diagnostic is recorded in
[`aarch64-crc-specialization-results.md`](aarch64-crc-specialization-results.md).
The local Apple AArch64 prototype did not recover the global build gain; no
production WAL code changed, and the OCI A/B/C comparison remains unmeasured
because target SSH access was unavailable.

The current-HEAD OCI default versus global `+crc` reattribution is recorded in
[`current-crc-reattribution-results.md`](current-crc-reattribution-results.md).
At source SHA `9e82d9d`, global `+crc` reduced WAL group encode by 25.4%, WAL
append by 13.9%, and raised width-1 throughput by 3.9%. This passes the
diagnostic CRC high-priority gate; it is a known-hardware control, not a
portable production implementation. Stable physical page encoding remains the
follow-up candidate after the CRC backend investigation.

The production AArch64 Linux CRC32 requirement and repository-default build
policy are recorded in
[`aarch64-crc-baseline-results.md`](aarch64-crc-baseline-results.md). Its
same-source OCI comparison confirms that ordinary `aarch64-unknown-linux-gnu`
release builds produce the same binary and equivalent results as an explicit
global `+crc` build. This decision supersedes the earlier recommendation to
investigate portable CRC runtime specialization; no later optimization was
started in this task.

The trusted internal WAL page-image validation fast path and OCI result are
recorded in [`trusted-wal-image-results.md`](trusted-wal-image-results.md).
WAL format is unchanged. Public WAL append validation remains strict.
Fault-injection and recovery paths remain strict. Only the release fast path
for internally generated Blink WAL images skips redundant full page decoding.

Planner routing attribution and borrowed-page routing are recorded in
[`planner-borrowed-routing-results.md`](planner-borrowed-routing-results.md).
Only planner routing ownership changed: planner routing now borrows
`BlinkPage` values from `BlinkState`, removing route-time page clones and
`Arc` allocations. B-link structure, transaction ordering, dependency
semantics, WAL semantics, publication, and recovery are unchanged. The OCI
width-1 sync-disabled comparison meets the strong-success criteria; WAL
append is now the largest measured top-level component.

Physical execution costs are attributed in
[`physical-execution-attribution.md`](physical-execution-attribution.md). No
physical execution optimization was implemented in this task.

The exact-layout Blink page fit-check implementation and its measurements are
recorded in [`fit-check-results.md`](fit-check-results.md). Fit checks no
longer serialize a page to calculate fit, though canonical-key validation
still allocates decoded key components. The sync-disabled OCI width-1 result is
a structural success, and the width-16 control passed its regression gate.
These are CPU and engine diagnostics from the 30 GB root filesystem, not
durability results or measurements on the unmounted 200 GB block volume.
The allocation-free canonical key validation follow-up is recorded in
[`key-validation-results.md`](key-validation-results.md). No page format, split
policy, batch semantics, WAL semantics, or encoder format changed; only
canonical encoded-key validation changed from allocating decode to
allocation-free scanning.

The direct fixed-buffer Blink page encoding result is recorded in
[`direct-page-encoding-results.md`](direct-page-encoding-results.md). The page
format is unchanged. Direct encoding produces byte-identical 4096-byte page
images. B-link split policy, batching semantics, WAL semantics, and recovery
format are unchanged.

The redundant leaf encoded-key validation result is recorded in
[`page-encode-validation-results.md`](page-encode-validation-results.md). The
layout boundary retains ordering and canonical-key validation while the
record writer reuses that validated precondition. OCI width-1 physical page
encode cost is 0.784x the fresh base and throughput is 1.045x; the width-16
control is 0.977x base. Page bytes and format are unchanged.

Planned serial leaf ownership and clone elimination are recorded in
[`leaf-ownership-results.md`](leaf-ownership-results.md). The planned path now
keeps one batch-local owning page copy in `WorkingBlinkState`, while
`CachedLeaf` retains only a `PageId`. Committed Blink state remains immutable
until WAL success. Page format, split semantics, WAL semantics, transaction
ordering, and publication semantics are unchanged. The OCI sync-disabled
width-1 diagnostic is a structural success, and the width-16 control is 4.05%
above its prior median. Both runs used the mounted 30 GB root filesystem; they
are CPU and engine diagnostics, not durability measurements or results from
the unmounted 200 GB block volume.

Baseline branch: `main`

Baseline commit: `1ff96e1` (`storage: rely on crash-consistent storage snapshots`)

Experiment branch: `experiment/b-link-batched-engine`

Phase 2 result: [`phase2-results.md`](phase2-results.md)

Phase 3 result: [`phase3-results.md`](phase3-results.md)

Current OCI Phase 3 diagnostic result:
[`phase3-oci-results.md`](phase3-oci-results.md)

Phase 3.1 sparse working-state result:
[`phase3-sparse-working-state-results.md`](phase3-sparse-working-state-results.md)

The report separates partial real-sync observations from true sync-disabled
CPU/engine diagnostics; neither establishes performance on the unmounted
200 GB target filesystem.

Phase 3 Mac runs are development smoke/profiling only. They are not the
production benchmark, adoption evidence, or a final performance conclusion.
The production target is an OCI A1 instance with 2 OCPU, 12 GiB RAM, and a
200 GB block volume. The available OCI run is a deliberately partial
diagnostic; it is not an adoption decision.

Performance status:

- Old Phase 0/1/2 performance numbers are previous-machine historical
  measurements only.
- The current benchmark baseline is the new-machine rebaseline in
  [`rebaseline-results.md`](rebaseline-results.md).
- The new raw artifacts are separated under
  `docs/experiments/results/rebaseline-macbookpro17-1-macos26.4-8c8t/`.
- Correctness, format, publication, and recovery results are not invalidated
  by replacing the performance machine.

The Phase 1 implementation SHA recorded in the earlier result document is a
historical identity only. Phase 2 started from the requested branch HEAD,
`23dc4b38b9249e2f7814c099866be100ef0a54a0`, and its implementation and test
commits are recorded in `phase2-results.md`.

This document is the source of truth for the implementation history and for
the later benchmark that decides whether the experimental engine should be
adopted. Its baseline sections describe the repository at the frozen starting
commit; the linked phase result records describe completed changes.

## 1. Goal

Build a separate experimental storage engine that can increase sustained
mutation throughput under many writers while allowing ordinary reads to scale
across CPU cores. The final comparison is between the current `main` mutable
B+Tree and the experimental B-link + batching engine on the same machine, with
the same workload, cache/working set, and durability conditions.

The design target is:

```text
logical serialization without physical serialization
```

Transaction requests must retain their current FIFO logical serialization
order. Physical tree work may run in parallel only where the dependency graph
allows it. B-link pages, high keys, right links, page-local synchronization,
optimistic versioned reads, micro-batching, routing, same-leaf coalescing, and
parallel page workers are means to that end; none is a goal by itself.

The initial batching policy must not intentionally wait longer than 1 ms. The
benchmark must still measure the complete candidate set, including 2 ms and
5 ms, to show whether a larger experimental delay is useful or unacceptable.

The read target is multicore scalability, not a particular thread or executor
implementation. `Get`, `Query`, and `Scan` must not pass through the write
coordinator, must not require a global tree read lock for the traversal, and
must continue to observe only committed state.

## 2. Non-goals

- Do not replace or modify the current `main` B+Tree baseline as part of this
  experiment. The current engine remains the control implementation.
- Do not redesign the WAL frame format or change the full-page redo WAL
  contract. The current durability point and group-commit behavior remain in
  force.
- Do not add on-disk backward compatibility between the baseline format and
  the experimental format. A new engine/page format version is allowed and
  expected if B-link metadata requires it.
- Do not introduce user-visible historical reads, range-read validation,
  snapshot APIs, replication, distributed transactions, or cross-shard
  coordination. Internal page-version retention for safe concurrent reads is
  not a user-facing MVCC feature.
- Do not assume that B-link or batching is valuable without an ablation and a
  sustained-throughput result. A simpler design that wins the benchmark is
  preferred.
- Do not change request ordering, conflict behavior, revision assignment,
  atomicity, recovery, or the meaning of a successful durable write.
- Do not use a synthetic fsync result as the final durability result. Injected
  sync delays are useful attribution controls; real durable runs are required
  for the final comparison.

## 3. Current main baseline

### 3.1 Repository and storage boundaries

The workspace separates `dodb-core`, `dodb-storage`, `dodb-testkit`, the
service/protocol layers, and the QUIC server. The storage implementation is in
`crates/dodb-storage/src/btree/`. The public production path opens one paired
data file and WAL file per local shard through `BTreeStore::open_path`.

The relevant source map at the baseline commit is:

| Concern | Current source | Observed behavior |
| --- | --- | --- |
| Storage API and mutable engine | `crates/dodb-storage/src/btree/mod.rs` | `BTreeStore`, `Overlay`, prepare/publish, mutation, lookup, split, allocator |
| Async scheduling and committed reads | `crates/dodb-storage/src/btree/coordinator.rs` | FIFO coordinator, bounded queue, group collection, immutable read view |
| Page body format | `crates/dodb-storage/src/btree/format.rs` | version-1 slotted leaf/internal/overflow/free pages |
| Page header/checksum | `crates/dodb-storage/src/page.rs` | fixed 4096-byte page, page LSN, CRC32C |
| Structural checking | `crates/dodb-storage/src/btree/checker.rs` | tree ranges, leaf chain, overflow reachability, free-list checks |
| WAL | `crates/dodb-storage/src/wal.rs` | framed version-2 WAL, full page images, explicit `COMMIT`, one sync per group |
| Transaction contract | `crates/dodb-core/src/transaction.rs` and `revision.rs` | point conditions, mutations, revisions, structured conflicts |
| Reference oracle | `crates/dodb-testkit/src/reference.rs` | deterministic serial model and differential-test helper |
| Existing benchmark | `crates/dodb-storage/src/bin/phase4-bench.rs` | coordinator/read measurements, but current-thread runtime and a small run |

The architectural documents that define the current contract are
`docs/architecture/00-overview.md`, `01-invariants.md`,
`02-storage-format.md`, `03-transactions.md`, and `03-wal-recovery.md`.

### 3.2 Current page and tree format

The baseline uses 4096-byte fixed pages. The common page header contains the
page format version, page type, physical page ID, page LSN, flags, and a CRC32C
over the full page. Pages 0 and 1 are alternate superblocks. Data pages begin
at page 2; the superblock stores the root page, free-list head, high-water page,
and checkpoint LSN.

The B+Tree bodies are version-1 slotted pages:

- A leaf stores ordered `(encoded document key, revision, value/tombstone)`
  records and one `next_leaf` page ID. Empty leaves are allowed after delete;
  there is no delete merge/rebalance.
- An internal page stores a `leftmost_child` and ordered separator records,
  each pointing to a right child. The separator is the smallest key in the
  right subtree; it remains a valid partition boundary even when a child is
  empty.
- A large value uses an overflow chain. Replaced/deleted overflow pages are
  placed on the free stack.
- The current format has no high key and no right sibling link on internal
  pages. The leaf chain is the only sibling-style traversal path.

The exact format and invariant behavior are documented in
`docs/architecture/02-storage-format.md` and implemented in
`btree/format.rs` and `btree/checker.rs`.

### 3.3 Current write path

The synchronous path is an overlay-and-publish pipeline:

```text
BTreeStore API
  -> Overlay over the committed/cache state
  -> validate/read/modify pages in the overlay
  -> prepare full encoded page images and a new superblock
  -> append WAL page images and COMMIT records
  -> WAL sync (durability point)
  -> publish committed cache/dirty pages
  -> later data-file flush/checkpoint
```

`Overlay::page` checks overlay pages, the synchronous page cache, and then the
data file. A mutation finds a leaf from the root, clones the decoded page into
the overlay, changes the leaf, allocates overflow/sibling/parent pages when
needed, and marks pages dirty. `insert_leaf_entries` and
`insert_parent_separator` perform leaf, internal, cascading, and root splits
inside the same mutable overlay. There are no page latches or concurrent
physical writers in the baseline.

`finish`/`finish_transaction` encode the dirty pages as full after-images and
prepare a new alternate superblock. Publication updates the tree metadata,
cache, dirty-page set, and optional read-view update in coordinator order.

### 3.4 Current coordinator path and batching

`AsyncShard::execute_with_response_budget` first tries the committed read view
for ordinary `Get`, `Query`, and `Scan`. `Put`, `Delete`, transactions,
`TransactGet`, `Observe`, and checkpoint operations are sent to a bounded Tokio
channel using nonblocking `try_send`; a full queue returns `Overloaded`.

The current coordinator defaults are:

```text
queue capacity       256 requests
maximum group        64 logical requests
maximum group bytes   approximately 4 MiB
collection delay      0 by default
```

Group collection yields once, drains immediately available requests until a
request/byte limit, and only arms the optional delay when more than one request
has already been collected and no pending request hit a limit. A positive
delay is bounded by `CoordinatorConfig::max_collection_delay`.

The coordinator then processes contiguous mutation segments together. A
mutation segment becomes `apply_transaction_group`; non-mutation operations
are processed in FIFO barriers around it. This means a read queued between two
mutation runs observes the state published by the earlier run before the later
run executes.

The physical group is not one logical transaction. For each accepted logical
transaction, current code creates a separate page-image sequence and separate
WAL `COMMIT` record. `WalLog::append_group` appends all those records in order
and performs exactly one WAL `sync_data`. A condition conflict produces no
page images for that candidate; later candidates can still be evaluated against
the state of earlier accepted candidates.

### 3.5 Current read path and concurrency model

At startup, `AsyncShard` constructs `CommittedReadView` by decoding all pages
reachable from the validated root. It stores immutable `Arc<PageData>` objects.
After a successful publication, changed pages and root/high-water metadata are
installed under one `Arc<RwLock<ReadViewState>>` write guard.

Ordinary `Get`, `Query`, and `Scan` hold the read guard for the complete
operation. Thus multiple readers can run at the same time and do not consume
the write queue, but the read view still has a global `RwLock`: publication
waits for all current read guards, and readers wait for publication's short
write section. A reader traverses one immutable view, so it cannot observe a
mixture of two publications. `TransactGet` remains coordinator-serialized and
reads one coordinator state in input order.

This is a useful baseline safety mechanism, but it is not the target read
architecture: a global view lock is still present and ordinary read traversal
is not a page-versioned, lock-free/optimistic path.

### 3.6 Phase 2 versioned read implementation

The Phase 1 `serial-blink` selector remains unchanged as the serial control.
Phase 2 adds `versioned-blink`. Its writer is still the same one-coordinator,
serial Blink mutation path, but `Get`, `Query`, and `Scan` use a separate
`BlinkReadHandle` and never lock the writer store or enter its queue.

The read handle pins an immutable `PublishedGeneration` through a short
`std::sync::RwLock` snapshot. Traversal then uses an immutable
`PageCatalog` (`PageId -> Arc<PageCell>`) and releases all publication
synchronization before it reads the root, internal pages, leaf chain, or
overflow values. A publication replaces cells only for pages changed by the
whole physical group; old generation Arcs retain old cells until their last
reader releases them. High-key/right-link correction is performed against the
pinned catalog.

The detailed architecture, ordering proof, reclamation rule, tests, and
results are in [`phase2-results.md`](phase2-results.md). The benchmark
selector accepts `main-btree`, `serial-blink`, `versioned-blink`, and
`planned-blink`. The Phase 3 selector keeps the versioned direct-read path and
adds logical planning/coalescing with one serial physical writer.

### 3.6 Current durability path

The WAL is redo-only and contains complete encoded page images plus a `COMMIT`
frame. The WAL sync is the durability point. No changed data page is required
to be on the data file before a successful write response. After WAL sync,
committed images enter the in-memory dirty set and the committed read view.
Data-file flushing is later; checkpoint writes and syncs the database image,
advances the checkpoint LSN, and resets the WAL using the existing crash-safe
reset protocol.

If WAL sync or the uncertain post-WAL persistence path fails, the shard is
marked degraded and rejects further writes until reopened. Recovery replays
only complete committed logical transactions in WAL order and then checks tree
invariants.

### 3.7 Existing benchmark evidence and validation

`docs/phases/phase-4-concurrency.md` records representative baseline numbers
from the existing phase benchmark. The earlier fixed-delay benchmark measured
approximately 457, 1,748, 5,247, and 14,651 PUT ops/s at 1, 4, 16, and 64
clients. The later zero-delay coordinator measurements reported roughly
23,008, 19,096, 17,742, and 15,832 transactions/s at 1, 4, 16, and 64
clients for a small width-1 run. These are historical local observations, not
portable performance guarantees.

The checked-in `phase4-bench` uses a current-thread Tokio runtime and only
`OPERATIONS_PER_CLIENT = 16` for its transaction run. It is therefore useful
for regression signals and coordinator instrumentation, but it is not yet a
valid multicore scalability or sustained-load comparison. Phase 0 below
replaces it with a controlled, multi-threaded, longer-running harness while
retaining the existing benchmark as a compatibility reference.

At this baseline commit, `cargo test --workspace` passed: storage unit tests,
recovery tests, B-tree integration/differential tests, testkit tests, and QUIC
service tests all completed without failure.

## 4. Semantics that must not change

The following are hard compatibility requirements. Any performance result from
an engine that violates one of them is invalid regardless of throughput.

### 4.1 Logical transaction ordering

Mutation requests admitted to one shard have a coordinator FIFO order. If the
order is:

```text
Tx A -> Tx B -> Tx C
```

then validation and logical publication order must be exactly A, then B, then
C. B's conditions must see A's accepted staged result, and C's conditions must
see the state after both A and B. A physical worker may execute independent
page work out of order, but it may not change this logical order.

An accepted transaction has one commit LSN. Every changed key in that
transaction receives that LSN as its revision. The group may share one WAL
sync, but it must not collapse A, B, and C into one logical commit or reorder
their commit LSNs.

### 4.2 Conditions, missing revisions, and conflicts

The supported point conditions are `RevisionEquals`, `Exists`, and `NotExists`.
`RevisionEquals` compares the complete state identity, including a missing
key's revision. A key that has never existed is `Missing(0)`; a deleted key is
`Missing(delete_commit_lsn)`. `NotExists` tests logical absence only, so a
deleted key can satisfy `NotExists` while failing `RevisionEquals(0)`.

All conditions are evaluated before that transaction's mutations are applied.
Duplicate condition keys and duplicate mutation keys are rejected. A
condition mismatch returns the structured expected-vs-actual conflict and
publishes no part of that transaction. A failed candidate does not add a delta
to the group overlay, so later candidates do not see failed work.

Storage-level transactions require at least one mutation. The server's
condition-only request path is a separate point-observation/validation path in
the current repository; the experimental engine must preserve that externally
observable behavior unless a later, separately approved semantic change says
otherwise.

### 4.3 Atomicity and visibility

Multi-key transactions are all-or-nothing. No ordinary read may observe
uncommitted page work or a mixture of pages from a partially published
transaction. A write response is successful only after the current WAL
durability point and committed-state publication have succeeded according to
the existing contract.

Ordinary `Get`, `Query`, and `Scan` are independent committed reads. They do
not promise one common instant across separate calls, but each individual
operation must traverse one valid committed state. `TransactGet` retains its
one coordinator-state and caller-input-order behavior.

Range reads are not transactional validation primitives in v1. Scan/query
correctness means ordered committed results with the existing exclusive cursor
semantics, not serializable range predicates.

### 4.4 Durability and recovery

The existing full-page redo WAL, explicit `COMMIT` frames, WAL sync point,
NO-STEAL/NO-FORCE behavior, degraded-shard behavior, checkpoint ordering, and
crash recovery rules remain unchanged. A crash after a successful WAL sync but
before in-memory publication must recover all complete committed transactions
in their logical order. A torn or uncommitted WAL tail must not become visible.

## 5. Current bottlenecks / hypotheses

### 5.1 Confirmed baseline serialization points

1. One coordinator task owns one mutable `BTreeStore`; all physical mutation
   preparation is serialized there.
2. One `Overlay` performs route lookup, page decode/clone, mutation, split,
   allocator changes, and encoding for its current logical sequence.
3. A mutation that touches one leaf can also touch its ancestors, sibling, free
   list, overflow pages, and superblock. There is no page-local parallel work
   scheduler.
4. `publish_prepared_group` publishes prepared transactions in order and
   updates the cache/dirty state serially. This is required for correctness in
   the baseline, but it is also a physical serialization point.
5. Full-page WAL images are written for every page image in every logical
   transaction. Group commit removes some sync cost but does not remove page
   encoding, append, or image bytes.
6. Ordinary reads already bypass the write queue, but a global read-view
   `RwLock` still surrounds each complete read and each publication update.
7. The checked-in async benchmark runs on a current-thread runtime, so its
   client concurrency does not demonstrate multi-core execution.

### 5.2 Hypotheses to test, not assumptions to preserve

- **H1: B-link metadata helps structural concurrency.** High keys and right
  links should let a reader or worker correct a stale parent path after a split
  without taking a global tree lock. This matters most during concurrent split
  workloads; it should not materially improve a no-split point workload by
  itself.
- **H2: Routing and coalescing reduce CPU work.** Sorting/grouping by leaf can
  avoid repeated root-to-leaf traversal and repeated decode/encode setup. The
  gain should be visible even when WAL sync cost is near zero.
- **H3: Independent leaves scale.** Different-leaf page work can run in
  parallel after logical admission. Same-leaf work cannot be made independent
  without changing transaction order, so a same-leaf-heavy workload is an
  important negative/control case.
- **H4: Group commit is already present.** Batching may add little when WAL sync
  is cheap because `main` already shares one sync across a group. Any gain must
  be attributed to planner/page execution work, not counted as a new group
  commit feature.
- **H5: Versioned reads improve mixed-load scaling.** An epoch/page-version read
  path should remove the global view lock and allow readers to continue while a
  writer publishes one page. The cost of retries, page-version retention, and
  cache misses may outweigh that benefit under intense writes.
- **H6: Full-page WAL may dominate.** If page image generation or WAL append
  remains the dominant cost, parallel tree execution may not improve durable
  mutation throughput. The instrumentation must expose this rather than hiding
  it in end-to-end latency.

## 6. Proposed architecture

The experimental engine should be a separate implementation selected behind a
benchmark-only constructor or feature. The baseline `BTreeStore` remains
available and is not converted in place. The exact Rust type/module names are
an implementation decision, but the layers and ordering below are normative.

```text
request ingress
  -> FIFO logical serialization coordinator
       -> condition validation against committed base + ordered staged deltas
       -> commit sequence / revision reservation
  -> batch planner
       -> key encoding and route snapshot
       -> dependency graph and leaf/page grouping
       -> same-leaf ordered coalescing
  -> page-local workers
       -> independent leaf/overflow jobs in parallel
       -> ordered structural-modification jobs for splits
  -> WAL assembler
       -> per-logical-transaction full-page images in FIFO order
       -> existing WAL append_group + one sync
  -> committed publication
       -> install versioned page images and root/allocator manifest
       -> advance published epoch
       -> complete write responses

Get / Query / Scan
  -> versioned committed read path (no write coordinator)
```

The logical coordinator is still a single ordering authority. It should do
only the work needed to make admission deterministic: request validation,
condition checking, creation of ordered staged deltas, dependency discovery,
and commit sequencing. It must not be the executor for every independent page
mutation.

The batch planner receives accepted transactions in that order. It routes each
key using a committed route snapshot and B-link correction. It groups work by
leaf/page, retains per-transaction order for overlapping keys/pages, and emits
independent jobs only when no earlier transaction can affect the job's input.

The page workers operate on private page state or page-local version candidates.
They never publish an uncommitted page directly to readers. A worker failure
aborts the affected logical transaction/group before WAL append; it cannot
leave a partial logical transaction visible.

The WAL assembler is the final order boundary. It must receive all prepared
logical transactions in FIFO order, assign the actual WAL record LSNs in that
order, restamp page images/revisions as needed, and call the existing group WAL
path. No worker is allowed to choose a commit LSN based on completion race.

## 7. B-link page format and invariants

### 7.1 Format version

The experimental engine uses an explicit new engine/page body format version.
The 4096-byte page size, explicit little-endian fields, physical page ID,
page-LSN field, and whole-page CRC32C remain. The baseline version-1 B+Tree
decoder must continue to reject unsupported layouts explicitly; it must not
silently interpret an experimental page as a baseline page.

The superblock needs an explicit engine/layout identity so an experimental
database cannot be accidentally opened by the baseline constructor. WAL frame
format and commit framing remain the current version unless an implementation
constraint proves that the existing append/recovery code cannot validate the
new page body; in that case only the page-image validation seam may be
generalized, not the WAL protocol redesigned.

### 7.2 Required page metadata

Every B-link internal and leaf page has:

- an ordered fence `high_key`, interpreted as an exclusive upper bound;
- a `right_sibling` page ID at the same tree level, or null for the rightmost
  page at that level;
- the existing page LSN and a new page-body layout version;
- the existing slotted records/children, with the current canonical key and
  revision/value encodings unless a benchmark proves a compatible improvement;
- reserved bytes that decode as zero and are checked as corruption otherwise.

The right sibling link is not a child pointer. An internal page's sibling is
another internal page at the same level; a leaf's sibling is the next leaf in
key order and also supplies the query/scan chain. `high_key = +infinity` is
represented by an explicit null/flagged fence, not by an invented user key.

### 7.3 Search and split invariants

For a valid page at a fixed committed epoch:

1. Every key in the page is greater than or equal to its lower fence (implicit
   from the parent/previous sibling) and strictly less than `high_key` when a
   high key exists.
2. If `key >= high_key`, the search must follow `right_sibling` and retry the
   page read/version check. A missing right link in this case is corruption or
   a retryable publication race, never a reason to return a false miss.
3. The right sibling's lower range starts at the current page's high key after
   a split. Sibling links at each level form an acyclic, strictly ordered
   chain.
4. Parent separator insertion may lag the split publication. A stale parent
   route remains correct because the child page's high key/right link redirects
   the search. Parent separators must eventually be installed before the
   structural job is reported complete.
5. Internal separators remain strictly ordered and partition child ranges.
   Each internal page's child pointer must lead to the page range selected by
   that separator, subject to B-link correction.
6. Leaf entries remain strictly ordered, including tombstones. Tombstones keep
   their last committed revision. Overflow chains remain reachable exactly once
   from their owning committed leaf.
7. A page ID is not reused while an old page version or old right-link target
   can still be observed by a pinned reader. Free-page reuse is therefore
   epoch-delayed; the baseline immediate free-stack assumption is insufficient
   for the experimental read path.

### 7.4 Runtime page version

The durable page LSN identifies the committed WAL image. It is not sufficient
as a transient synchronization primitive because a writer may prepare a new
image before publication. Each in-memory `PageCell` therefore needs a runtime
sequence/version independent of the encoded page bytes. A writer changes that
sequence around a page-version install; a reader retries if it observes an
odd or changed sequence while taking a page snapshot.

The first safe implementation may use a short page-local read/write guard to
clone an immutable `Arc<PageVersion>` and release it before traversing the
page. A later optimization may use a seqlock plus safe epoch/hazard reclamation
for the pointer. No implementation may use an unprotected raw pointer or free
an old page object while a reader can still hold it.

## 8. Read concurrency design

### 8.1 Read state and publication

The proposed read path uses an atomic published commit epoch plus immutable
versioned page objects. A committed publication descriptor contains:

- the published commit LSN/epoch;
- root page ID;
- allocator/high-water metadata needed for bounds checks; and
- the page-version updates belonging to that logical commit or committed WAL
  group.

After WAL sync, page-version candidates are installed into their page cells and
the root/allocator manifest is made available. The published epoch advances
only after all page updates for the transaction/group are installed. Readers
pin an epoch before traversal and select, for each page, the newest page
version visible at or before that epoch. This gives a reader one committed
tree state without a global tree read lock.

The old versions are internal read-safety history, not a public historical-read
API. An epoch/reclaimer can drop versions once no reader is pinned below the
reclamation point. Memory usage, version-chain length, and reclamation lag are
explicit benchmark metrics.

### 8.2 Optimistic traversal algorithm

For each ordinary `Get`, `Query`, or `Scan`:

1. Pin the current published epoch and load the corresponding root metadata.
2. Read an internal page's runtime sequence, take a page-version snapshot, and
   validate the sequence. Retry the page snapshot if the sequence is odd or
   changed.
3. Route through the internal separators. If the selected page's high key says
   the key belongs to the right, follow the same-level right link; do not
   restart at the root merely because the parent was stale.
4. At the leaf, validate the fence/right-link relationship and perform the
   point lookup or ordered iteration. Query/Scan follows right links while
   retaining the pinned epoch and response budget.
5. Read overflow pages through the same epoch/version mechanism.
6. If a required page version is unavailable, a page sequence changes after
   the snapshot, a structural link is inconsistent, or a validation check sees
   a newer conflicting publication, restart the operation and increment an
   optimistic-read-retry counter.
7. Release the epoch only after materializing the response.

Readers run on caller/executor worker threads and never enqueue ordinary reads
to the write coordinator. A page-local writer can make readers of that page
retry, but it must not block unrelated pages or all tree readers. There is no
global traversal `RwLock`.

The experimental implementation must preserve current response budgets,
ordered query/scan results, exclusive cursors, tombstone revisions, and
read-after-success visibility. `TransactGet` remains coordinator-serialized
unless a later design proves an equivalent atomic point snapshot without
changing its current semantics.

## 9. Write batching design

### 9.1 Logical admission pass

The collector produces a FIFO group. Before any parallel physical work, the
logical coordinator processes the requests in order:

1. Validate request structure, keys, values, and duplicate rules.
2. Evaluate every condition against a `LogicalOverlay` containing the last
   published committed state plus the accepted deltas from earlier requests in
   this exact group.
3. For an accepted request, assign a serial position and a provisional commit
   token, add its resulting key states to the overlay, and emit a physical
   transaction plan.
4. For a conflict or invalid request, emit only that error and add no delta.
5. Continue to the next request, preserving the current behavior that later
   requests can still be accepted after an earlier candidate conflict.

This pass is deliberately serial in logical order. It is the semantic fence;
it is not allowed to move into page workers.

### 9.2 Routing and dependency groups

For every accepted mutation, the planner records the encoded key, logical
transaction position, target level/leaf route, and read/write dependencies.
Dependencies include:

- any earlier transaction that writes a condition key;
- any earlier transaction that writes the same mutation key;
- any earlier transaction that changes a page needed by a structural route;
- all mutations within the same multi-key transaction.

Non-overlapping leaf/page jobs may run concurrently. Overlapping keys/pages
form an ordered chain. A same-leaf group is not declared independent merely
because its keys differ: the planner must preserve the transaction order at
each page-image boundary and the logical overlay result.

### 9.3 Same-leaf coalescing

The planner may decode a leaf once, apply a sequence of ordered operations in
one worker, and reuse the resulting page state for subsequent operations. This
removes repeated routing and page setup, but it does not merge logical commits.

Because this experiment keeps the existing full-page redo WAL contract, the
WAL assembler must still be able to emit the correct full after-image for every
logical transaction that changed the leaf. If A and B both change one leaf,
the WAL sequence must contain the A after-image and the B after-image (or an
equivalent representation that the unchanged WAL recovery code can replay as
two committed logical units). A final-page-only optimization is out of scope
unless the WAL contract is separately redesigned.

### 9.4 Parallel page workers

Workers receive immutable route inputs and private mutable page candidates.
The initial parallel worker implementation should support non-structural
leaf/overflow mutations and fall back to an ordered structural path when a
page would split. This isolates page-local throughput gains from split
correctness. Later phases add concurrent split jobs.

Workers join at a deterministic barrier before WAL append. Completion order is
not commit order. The planner/assembler sorts all logical results by the
original FIFO position, and the WAL append order is that same order.

### 9.5 Existing WAL group commit

After all accepted jobs in a group are prepared, append:

```text
Tx A: PAGE_IMAGE... COMMIT
Tx B: PAGE_IMAGE... COMMIT
Tx C: PAGE_IMAGE... COMMIT
                              one WAL sync
```

Only the existing WAL sync establishes durability. Publication and response
completion happen after it. A WAL append/sync failure follows the current
degraded-shard policy; parallel workers must not create a second uncertain
outcome protocol.

## 10. Transaction ordering and dependencies

The logical state machine is:

```text
published committed state
  + accepted A delta -> staged state A
  + accepted B delta -> staged state B
  + accepted C delta -> staged state C
```

Conditions for B are evaluated against staged state A; conditions for C are
evaluated against staged state B. This rule applies even if B and C use
different leaves and their physical workers execute concurrently.

The implementation should represent each accepted transaction with at least:

- FIFO serial position;
- conditions and mutations;
- logical read/write key sets;
- staged resulting states for keys changed in the group;
- provisional revision/commit token;
- routed page jobs and dependency predecessors;
- final WAL image list and final commit LSN.

The difficult part is revision allocation. Current revisions equal the final
WAL commit LSN, while the final LSN depends on the number of page-image
records. The experimental plan is:

1. perform logical admission and physical planning in FIFO order with
   provisional revisions/tokens;
2. determine each transaction's page-image count deterministically;
3. reserve actual WAL record LSNs in FIFO order in the WAL assembler;
4. restamp each changed page and response revision from its provisional token
   to its final commit LSN before encoding/appending; and
5. publish only the final ordered images.

No physical worker completion race may affect conditions, revisions, or WAL
ordering. Differential tests must compare accepted/rejected results,
structured conflicts, final values, missing revisions, and commit-LSN order
against `ReferenceDb`.

## 11. Structural modification / split strategy

### 11.1 Initial split publication

When a leaf overflows, the structural job creates a right sibling containing
the upper entries, gives it the old high key/right link, changes the left page
to have `high_key = separator` and `right_sibling = new_right`, and then
arranges the parent separator insertion. The left/right metadata must become
visible as one committed publication; a reader may not see a new link to an
unpublished page.

The parent separator can be installed after the child split because a stale
parent path that still reaches the left page will follow the left page's high
key/right link. The structural job is complete only after the parent is
correct, or after the job is recorded for a deterministic retry.

Internal page splits use the same high-key/right-link rule at the internal
level. The promoted separator is removed from the split child pair and added
to the parent according to the new format's checked partition invariant.

### 11.2 Root split

Root changes update root metadata in the committed publication descriptor. The
new root and its child pages must be atomically visible at one committed epoch.
The implementation must test readers that pin the old root while a root split
is published and readers that start after it. No reader may follow a freed or
partially initialized root.

### 11.3 Concurrency and latching

The first concurrent-SMO design should use an explicit structural-modification
order and page-local latches with a documented lock order (for example,
parent/child or left-to-right at one level). It must never hold a page latch
while waiting on the WAL sync. A version mismatch causes a route/SMO retry,
not an in-place overwrite of a page changed by another worker.

The optimization target is page-local coordination, not a promise that all
splits are lock-free. If concurrent SMO complexity does not improve sustained
throughput, the engine may retain serialized structural jobs while preserving
parallel non-structural leaf work.

### 11.4 Deletion and page reuse

The initial experimental engine should retain the baseline's no-merge delete
behavior. That limits structural races. Overflow and eventually free-page
reuse must be delayed until epoch reclamation proves that no reader can hold an
old page ID/link. Reusing an ID too early can make a stale right link reach an
unrelated valid page, which is a correctness failure rather than a retry.

## 12. WAL and durability interaction

The current WAL contract is fixed for this experiment:

- framed WAL version 2 with explicit identity, checksums, page-image records,
  commit records, and strict LSN/batch ordering;
- full encoded page after-images, including the appropriate superblock image;
- one `COMMIT` record per logical transaction;
- one `sync_data` for an accepted physical group;
- WAL sync before committed page publication and before successful mutation
  response;
- data-file flush/checkpoint later, with the existing crash-consistent reset;
- recovery replays complete committed transactions in FIFO/WAL order.

The experimental page body can have a new version and B-link fields. WAL
recovery must validate and replay those full images, then rebuild transient
page-cell sequence/version metadata from the committed page LSNs. Runtime
seqlock values, epoch pins, reader slots, and free-page retirement state are
not WAL data.

The page-image budget is a first-class concern. Same-page operations may share
decoding and mutation computation, but each logical commit boundary must remain
recoverable under the unchanged full-page redo semantics. Instrument page
images and WAL bytes both per logical transaction and per physical group.

Checkpoint is a coordinator barrier. It must not race page workers or retire a
page version that a pinned reader can still need. The implementation must
preserve the current ordering of data-file sync, checkpoint superblock sync,
WAL reset, and WAL reset sync.

## 13. Batch latency policy

The experimental collector has two distinct concepts:

- **queue wait:** time from request enqueue to the collector receiving it;
- **intentional collection delay:** time the collector deliberately waits for
  more requests after the first request/group evidence.

The policy is:

1. At low load, execute a lone request without an intentional sleep.
2. If enough requests are already queued to fill a useful group, drain and
   execute immediately.
3. At intermediate load, permit adaptive collection when the observed arrival
   rate and current group size indicate that another request is likely to
   arrive before the deadline.
4. Bound the default/initial maximum intentional delay at 1 ms.
5. Never allow collection to bypass FIFO order, split a logical transaction,
   or wait past request/byte/structural barriers.

The implementation must expose both configured and actual delay. The mandatory
comparison set is:

```text
0
50 us
100 us
250 us
500 us
1 ms
2 ms
5 ms
```

The 2 ms and 5 ms values are sensitivity experiments beyond the initial 1 ms
design maximum; they must not silently become the default. Batch-size limits,
byte limits, worker count, and delay are separate knobs so their effects can
be measured independently.

For latency interpretation, report engine/tree/scheduler time separately from
actual durable time:

```text
pre-durable = enqueue -> WAL sync start
durable     = enqueue -> WAL sync completion
publication = WAL sync completion -> committed publication/response
end-to-end  = enqueue -> caller response
```

For reads, report queue-free read traversal/materialization latency. The design
targets are `p50 < 1 ms` desirable, `p95 < 5 ms`, and `p99 < 9 ms` for normal
point operations, but an environment whose real sync latency exceeds those
values is expected to exceed them for durable writes. In that case the report
must show the engine component and actual durable component separately rather
than treating the durable p99 as a tree failure.

## 14. Implementation phases

The ordering below changes the initially suggested sequence in one important
way: transaction ordering and atomic publication are specified and tested
before parallel workers are trusted. They cannot be postponed until after
physical concurrency because every later optimization depends on the exact
logical overlay and commit boundary. Phase 6 remains a dedicated adversarial
semantic/durability hardening phase.

### Phase 0 — establish the baseline benchmark first

- Freeze the baseline SHA and build a multi-threaded sustained-load harness.
- Run current `main` B+Tree and current `AsyncShard` with the required writer,
  reader, width, key-distribution, workload, delay, and working-set matrix.
- Add only benchmark/instrumentation code needed to observe existing behavior;
  do not alter storage semantics.
- Store machine metadata, configuration, raw samples, summaries, and seeds.
- Validate the harness against the existing phase benchmark and all current
  tests.

The executed baseline record, raw-artifact paths, and the Phase 0 decisions are
kept in [the Phase 0 result record](phase0-results.md). That record is part of
this experiment's source of truth; its measurements must not be confused with
the historical short-run `phase4-bench` numbers described in Section 3.7.

### Phase 1 — B-link format, invariants, and serial correctness

- Add the experimental page/superblock format and decoder/checker in the
  experimental engine only.
- Implement high key/right sibling routing and serial split behavior.
- Keep one writer and no parallel physical workers.
- Differential-test all point operations, query/scan order, tombstones,
  revisions, transaction groups, reopen, checkpoint, and WAL recovery.
- Establish a serial B-link performance control before adding batching.

The completed Phase 1 implementation and control measurements are recorded in
[`phase1-results.md`](phase1-results.md). Phase 2 work must not start from
memory or from the code alone; that result record is part of the Phase 1
source of truth.

### Phase 2 — optimistic/versioned multicore reads

- Add versioned page cells, epoch pinning/reclamation, and atomic committed
  publication.
- Route `Get`, `Query`, and `Scan` directly to the read path.
- Prove stale-path correction during split and one-epoch read consistency.
- Run read-only and mixed read/write workloads on a genuinely multi-threaded
  runtime. Keep the coordinator read path as an explicit control variant.

### Phase 3 — logical batch planner, routing, and coalescing

- Completed: add the FIFO logical admission pass and staged dependency model.
- Completed: add route snapshots, key/leaf grouping, and same-leaf ordered
  coalescing.
- Completed: keep physical page execution serial so planner effects remain
  separate from worker parallelism.
- Completed: preserve per-transaction WAL image/`COMMIT` boundaries, revision
  restamping, and one-generation publication.
- Completed: record Phase 3 correctness and development smoke results in
  [`phase3-results.md`](phase3-results.md).

### Phase 4 — parallel page-local mutation workers

- Implemented persistent page-local leaf workers with private page candidates,
  deterministic job partition, FIFO LSN/WAL assembly, and sparse-state install.
- Structural, allocator/overflow, multi-leaf transaction, and cross-leaf
  dependency cases fall back as a whole group to the sparse serial executor.
- Correctness passed; OCI delay-zero results show median effective
  parallelism 1.075 and throughput speedup 0.859x. The worker mechanism needs
  further review before Phase 5 design. See
  [`phase4-results.md`](phase4-results.md).
- Phase 5 implementation has not started.

### Phase 5 — concurrent structural modification and splits

- Add page-local SMO coordination for leaf, internal, cascading, and root
  splits.
- Test parent-lag correction via high key/right link under concurrent readers
  and writers.
- Add safe epoch-delayed free-page reuse and structural retry handling.
- Keep a serialized-SMO variant for ablation; do not assume concurrent splits
  are a net win.

### Phase 6 — transaction ordering, atomicity, and durability hardening

- Run exhaustive A/B/C staged-dependency tests, multi-key atomicity tests,
  conflict/error tests, and randomized `ReferenceDb` differential tests.
- Run crash/fault matrices around WAL append, WAL sync, page-version install,
  published-epoch advance, response completion, data flush, checkpoint, and
  WAL reset.
- Verify recovery with separate logical commit records and shared sync.
- Verify no reader sees a mixed epoch or uncommitted page after any injected
  interruption.

### Phase 7 — adaptive batching and tuning

- Compare every required collection delay and batch-size candidate.
- Tune adaptive collection, worker count, byte/request caps, route cache, page
  version retention, and structural retry policy.
- Re-run the full benchmark matrix and all correctness gates after each change.

### Final — identical-condition adoption comparison

- Run the preregistered final matrix against current `main` and the final
  experimental configuration on the same machine and dataset seeds.
- Produce feature ablations, raw artifacts, summary tables, and an adoption or
  rejection decision using Section 19.
- Stop after the decision. Do not merge or replace `main` based only on a
  microbenchmark.

## 15. Correctness gates per phase

Every phase starts with all previous gates green. A performance win cannot
waive a failed gate.

| Phase | Required gate before proceeding |
| --- | --- |
| 0 | Harness repeatability; current `main` tests and crash/recovery tests pass; serial results match `ReferenceDb`; raw measurements include enough warmup/repetitions. |
| 1 | New format rejects bad magic/version/checksum/offsets; checker proves key ranges, fences, right-link chains, allocator, overflow, and root invariants; serial B-link differential tests match baseline values/revisions/conflicts. |
| 2 | `Get/Query/Scan` never enter the write queue; readers observe one committed epoch; no global traversal lock; page-version retries are memory-safe; stale paths during leaf/internal/root split return correct results; no UAF or premature page reuse. |
| 3 | A/B/C conditions see A then A+B staged state; conflicts do not mutate later overlays; same-leaf coalescing preserves per-transaction responses, revisions, page images, and WAL order; no duplicate/missing mutation. |
| 4 | Disjoint worker jobs are race-free under stress; overlapping jobs preserve serial order; all failures abort before publication; final state and WAL recovery match the serial oracle. |
| 5 | Concurrent leaf/internal/root splits preserve high-key/right-link routing, ordered scans, parent separators, no duplicate pages, no page leaks, no cycles, and correct recovery under split crashes. |
| 6 | Fault injection at every durability/publication boundary proves all-or-nothing visibility and current degraded/reopen semantics; committed WAL units recover in order; no read observes an uncommitted or mixed multi-page transaction. |
| 7 | Adaptive scheduler never exceeds configured policy unexpectedly, preserves FIFO/barriers, reports queue wait vs intentional delay separately, and does not trade correctness for throughput. |

The test suite should include deterministic tests plus randomized differential
tests. `ReferenceDb::transact_at` should be used when comparing real commit
LSNs. Concurrency tests must use barriers/latches and deterministic schedules
where possible; wall-clock timing alone is not a correctness proof.

## 16. Benchmark methodology

### 16.1 Controlled environment

For the final comparison, record and hold constant:

- exact baseline and experimental commit SHAs;
- CPU model, physical/logical core count, RAM, kernel, filesystem, mount
  options, storage device, and power/performance governor;
- Rust toolchain, build profile, compiler flags, Tokio runtime worker count,
  and process CPU affinity if used;
- database page size, cache capacity, value/key sizes, WAL/checkpoint policy,
  database UUID setup, and pre-seed procedure;
- no competing workloads, identical OS cache state policy, and identical
  database lifecycle (fresh, warm, or reopened).

The benchmark must use a multi-threaded runtime and enough operations for a
sustained interval, not only a fixed handful per client. Use a warmup period,
then several timed repetitions with deterministic seeds. Report each
repetition and an aggregate (median plus spread); do not report only the best
run.

### 16.2 Durability modes

Run at least:

1. real production WAL/data files with actual `sync_data`;
2. a test-file mode with measured sync latency reported explicitly; and
3. injected sync-delay controls at 0, 100 us, 1 ms, 5 ms, and 10 ms to show
   whether a result is CPU/tree work or group-commit amortization.

The same mode, file location, cache policy, and checkpoint schedule must be
used for both engines in any direct comparison. A result from an in-memory
file or a different sync policy is an auxiliary result, not the final answer.

### 16.3 Measurements and attribution

Instrument monotonic timestamps at request creation, enqueue, dequeue, group
formation, logical validation, route/traversal, page planning, page mutation,
WAL append start/end, WAL sync start/end, page-version publication, response,
and read completion. Use thread-safe low-overhead counters/histograms and
export raw event/counter data so percentile summaries can be recomputed.

Every write report must separate:

```text
engine/tree/scheduler component: validation + planning + mutation + WAL append
actual durable component:         WAL sync and all time through durability
publication/response component:   committed publication and response delivery
```

Use the same workload with these feature controls:

| Variant | Purpose |
| --- | --- |
| Current `main` B+Tree | End-to-end baseline |
| Experimental serial B-link, no batch optimization | B-link/format overhead and benefit |
| B-link + planner, no coalescing | Planner/routing effect |
| B-link + planner + same-leaf coalescing | Coalescing effect |
| Above + page workers, serialized SMO | Multi-writer page execution effect |
| Above + versioned direct reads | Reader parallelism effect |
| Final adaptive configuration | Combined candidate |
| Each experimental variant with batching disabled/forced delay | Collection-policy effect |

These are attribution controls, not separate products. If a control is too
expensive to implement, record why and do not claim isolated causality for the
missing feature.

## 17. Benchmark workloads

The minimum matrix is the Cartesian product of the following dimensions where
the run budget permits; otherwise use the staged core/focused matrix described
below and document omissions.

### 17.1 Concurrency and transaction shape

```text
writers: 1 / 4 / 16 / 32 / 64 / 128
readers: 1 / 4 / 16 / 32 / 64 / 128
transaction width: 1 / 4 / 16 / 25 mutations
```

Writers and readers are independent worker tasks/threads. A write-only run
uses the writer dimension; a read-only run uses the reader dimension; mixed
runs use the selected pair.

### 17.2 Key distributions

- **Uniform random:** keys sampled uniformly over the working set.
- **Sequential:** monotonically increasing inserts/updates, exercising right
  edge growth and split behavior.
- **Hotspot:** a small hot key/range fraction receives most operations,
  exercising same-page contention and conflicts.
- **Same-leaf-heavy:** keys selected from a compact range known to fit in a
  small number of leaves; report the actual leaf concentration.
- **Different-leaf-heavy:** keys pre-seeded across many leaves and assigned so
  concurrent writers mostly touch distinct leaves.

For conditional workloads, include explicit revision dependencies,
insert-if-absent, and read-dependent writes so logical ordering is exercised,
not just unconditional PUT throughput.

### 17.3 Operation mixes

Run:

```text
100% write
100% read
95% read / 5% write
50% read / 50% write
write-heavy mixed
```

Reads must include separate `Get`, `Query`, and `Scan` workloads. For Query and
Scan, fix and report limits/cursor distributions (for example point-like
limit-1, short limit-16, and longer limit-128). Report returned rows as well
as request ops so a larger scan is not mistaken for a faster point operation.

### 17.4 Batch and collection policy

Compare the required collection delays:

```text
0 / 50 us / 100 us / 250 us / 500 us / 1 ms / 2 ms / 5 ms
```

Also compare small, medium, and maximum batch sizes and byte caps. Record the
actual mean/p50/p95 batch size, queue depth, and intentional delay; configured
values alone are insufficient.

### 17.5 Working set and data shape

- cache-resident working set;
- larger-than-cache working set;
- same key/value size and seed for both engines;
- at minimum, small inline values and values that use overflow pages;
- report cache capacity and database/page counts.

The primary decision matrix should use a stable small value to isolate tree
and scheduling behavior, then a focused overflow matrix to expose full-page
WAL and value-chain effects.

### 17.6 Staged run matrix

To keep the full factorial manageable:

1. **Core:** writers/readers 1/16/64, width 1/16/25, all five key patterns,
   write/read/mixed ratios, resident and larger-than-cache, delay 0/500 us/1 ms.
2. **Scaling:** all writer and reader counts with width 1 and 16 on uniform,
   same-leaf-heavy, and different-leaf-heavy workloads.
3. **Delay sweep:** all required delay candidates at representative 1/16/64
   writer/readers and at least one mixed workload.
4. **Stress:** 128 writers/readers, hot keys, sequential split growth,
   concurrent splits, and real/injected durable latency.
5. **Final confirmation:** repeat winning configurations and the main baseline
   with fresh seeds and independent repetitions.

## 18. Metrics

### 18.1 Throughput and latency

- logical transactions/s (successful, attempted, conflicts separately);
- mutation ops/s, counting successful mutations inside transactions;
- read ops/s, with Get/Query/Scan separated;
- aggregate mixed ops/s and returned rows/s for range operations;
- end-to-end `p50`, `p95`, and `p99`;
- pre-durable, actual durable, publication, and end-to-end write latency;
- read traversal/materialization latency and response-budget failures.

### 18.2 Scaling and resource use

- process CPU utilization and per-core utilization;
- physical/logical core count and runtime worker count;
- writer scaling efficiency, defined as
  `throughput(N) / (N * throughput(1))` for the same workload;
- reader scaling efficiency using the same definition;
- memory usage, page-version retention, reclamation lag, and queue depth;
- coordinator utilization, worker utilization, and page-latch contention.

### 18.3 Scheduler, tree, and dependency timings

- batch size and bytes;
- configured and actual batch collection delay;
- queue wait;
- logical validation time;
- tree traversal time (including route correction and page-version retries);
- batch-planner/grouping time;
- tree mutation time;
- worker dependency wait and page-latch wait;
- publication time;
- number and duration of structural jobs/retries;
- overloads, conflicts, and failed requests.

### 18.4 Structural and optimistic counters

- pages touched per logical transaction and per mutation op;
- changed pages per group;
- leaf, internal, cascading, and root page splits;
- structural modification retries and stale-route corrections;
- optimistic read retries, retry depth, and read restarts;
- page-version chain length, epoch pin duration, and deferred free pages;
- same-leaf coalescing count and operations/coalesced page;
- independent page jobs and parallel worker utilization.

### 18.5 WAL and durability

- WAL bytes per logical transaction and per mutation op;
- full page images per transaction/group;
- WAL append/serialization time;
- WAL sync time and actual sync latency distribution;
- transactions per sync;
- page images/operation and superblock images/group;
- data-file flush/checkpoint bytes and time in separate maintenance runs;
- recovery time, replayed commits/pages, and post-recovery invariant-check time.

Metrics must be reported for both successful and failed/conflicting requests
where meaningful. A throughput increase caused by silently dropping or
overloading requests is not a win.

## 19. Adoption criteria

These criteria should be recorded before the final result is inspected. The
thresholds below are the proposed decision rule; if the project changes them,
the change must be documented before rerunning the final matrix.

### 19.1 Hard gates

Adoptability is false if any of these fail:

- all phase correctness gates pass;
- no mismatch with `ReferenceDb` for values, tombstones, revisions, conflicts,
  transaction order, query/scan order, or atomicity;
- no crash/recovery or WAL durability regression across the fault matrix;
- no reader observes an uncommitted or mixed committed state;
- no unexplained memory-safety, page-reuse, deadlock, livelock, or starvation
  failure under stress;
- existing public semantics and degraded-shard behavior remain unchanged.

### 19.2 Performance gates

The experimental engine should be considered a meaningful candidate only when
all of the following are true on repeated final runs:

- sustained multi-writer mutation throughput is at least 1.5x `main` at 16,
  32, and 64 writers on different-leaf-heavy workloads, with no material
  increase in conflict/overload rate;
- the geometric-mean mutation throughput across the core write/write-heavy
  matrix is at least 1.25x `main`, or the report explicitly calls the result
  inconclusive rather than claiming adoption;
- writer scaling efficiency at 32/64 writers is at least 20 percentage points
  better than `main` on the disjoint/different-leaf workloads, while the
  same-leaf result is reported as a known contention limit;
- read throughput scales with reader count without entering the write queue;
  at 16/32/64 readers, the experimental engine should reach at least 1.25x
  `main` on read-heavy workloads or show a clearly better scaling slope;
- in 95/5 and 50/50 mixed workloads, read throughput does not fall more than
  10% versus `main` at the same writer load, and writer throughput is not
  materially lower;
- engine/tree/scheduler p99 remains within the point-operation target where
  the machine can support it (`p50 < 1 ms` desirable, `p95 < 5 ms`, `p99 <
  9 ms`), while actual durable p99 is interpreted against measured sync
  latency rather than rejected solely because fsync is slow;
- WAL bytes/operation, page images/transaction, optimistic retries, and
  structural retries are reported and remain within an explainable range. A
  throughput result that depends on unbounded version retention or retry churn
  is not acceptable.

If the engine misses the thresholds but has a localized win, the decision may
adopt only that simpler feature (for example, direct versioned reads or
same-leaf planning) while retaining `main` for writes. B-link and batching are
not bundled into the product by default. If the gain is below the thresholds
or appears only in a short microbenchmark, keep the current main engine and
record the experimental result as rejected/inconclusive.

## 20. Risks / unresolved questions

### 20.1 Main risks

- **Atomic multi-page publication:** page-local parallelism is easy to make
  fast and hard to make atomically visible. A reader must not see half of a
  multi-key transaction.
- **Version retention and page reuse:** optimistic readers require safe object
  lifetime and deferred free-page reuse. Memory growth may erase throughput
  gains.
- **Full-page WAL amplification:** same-leaf coalescing may reduce CPU work.
  Metadata-stable transactions can omit the superblock image while retaining
  their data-page after-image and transaction boundary.
- **Revision/LSN allocation:** final WAL LSNs depend on image counts, while
  logical conditions need earlier committed revisions. Provisional tokens and
  ordered restamping must be exact.
- **Stale structural paths:** parent-lag, internal splits, root changes, and
  right-link chains can produce rare lost-key or duplicate-key errors if any
  fence invariant is wrong.
- **Hot-page contention:** same-leaf-heavy traffic may gain nothing or regress
  because page-local locking and coalescing still serialize the hot page.
- **Retry storms:** a high write rate can cause optimistic read or structural
  retries to dominate CPU and tail latency.
- **Scheduler distortion:** a collection delay that improves transactions/s can
  violate the latency target or hide queueing; both must be measured.
- **Benchmark/runtime bias:** the existing current-thread benchmark can make a
  multicore design look ineffective or effective for the wrong reason.
- **Implementation complexity:** a small measured win may not justify new
  invariants, recovery code, page-version management, and debugging cost.

### 20.2 Unresolved questions to decide before implementation

1. Should the first safe page-cell implementation use `std::sync::RwLock` for
   short pointer snapshots, a custom seqlock, or a dependency such as a safe
   atomic-Arc/epoch primitive? The choice must include a memory-reclamation
   proof and a benchmark, not only a microbenchmark of the lock.
2. Is an epoch-version chain per page sufficient for atomic group publication,
   or is a separate immutable publication manifest/root indirection needed for
   the final design? The chosen mechanism must handle multi-page transactions,
   root changes, and readers pinned across publication.
3. How should the existing WAL validation/recovery seam accept experimental
   page-body version 2 while leaving baseline WAL/page decoding strict? The
   answer must not change WAL frame ordering or durability semantics.
4. What is the exact provisional-revision to final-commit-LSN algorithm when
   workers discover different page-image counts or a transaction causes a
   split? This needs a deterministic test before parallel workers.
5. Should same-leaf coalescing always materialize an intermediate full-page
   image for each logical transaction, or can an unchanged-WAL-compatible
   representation prove the same recovery result without a WAL redesign?
   The default answer for implementation is “materialize each boundary”; any
   deviation requires a separate design review.
6. What page-level dependency granularity gives useful parallelism without
   incorrectly treating a root/internal route as independent of a leaf job?
   The planner should start conservatively and expose false-dependency cost.
7. What worker count, queue bound, and admission policy prevent page workers
   from starving the logical coordinator or readers at 128-client loads?
8. Should structural modification jobs be one ordered per-level queue, one
   global SMO queue, or page-local lock-coupled retries? The first choice should
   optimize for proof/debuggability; the benchmark decides whether to relax it.
9. How long may Query/Scan retain an epoch pin, and what is the bounded-memory
   policy if a large scan pauses reclamation under write load?
10. What exact value/key sizes and cache capacities represent the target
    deployment for the adoption decision? The final benchmark must predeclare
    them rather than selecting a favorable subset after measurement.
11. Are the proposed 1.5x/1.25x adoption thresholds appropriate after Phase 0
    establishes real-device sync cost, or should the project revise them before
    the final run? Any revision must be made before looking at final engine
    results.

## 21. Superblock WAL Elision

The metadata-stable superblock image elision passed its correctness gates and
met the strong-success thresholds on the OCI A1 width-1 workload. Page images
fell from 2.00 to 1.00 per mutation, WAL bytes fell 49.59%, and throughput
improved 16.98% for default release and 16.68% for the separate `+crc` build.
The crash-recovery audit, normalized costs, and raw artifacts are recorded in
[`superblock-wal-elision-results.md`](superblock-wal-elision-results.md).

## 22. Leaf Payload Sharing

The `Arc<[u8]>` leaf-payload COW experiment met the strong-success thresholds
on the same-host OCI width-1 A/B and improved the width-16 control. Full
normalized costs, raw artifact hashes, and the regression review are in
[`leaf-payload-sharing-results.md`](leaf-payload-sharing-results.md).

## 23. Planner Key Copy Cleanup

The planner key-copy cleanup removed two unused encoded-key duplicates and
showed lower planning cost on OCI, with inconclusive overall classification.
The usage audit, correctness evidence, A/B metrics, and raw artifact hashes
are in [`planner-key-copy-results.md`](planner-key-copy-results.md).

## 24. Admitted Mutation Key Reuse

The first OCI rejection used a binary built from the wrong checkout. A clean
forensic rebuild and rerun corrected the classification to structural/useful
success, and the validated candidate source has been restored to this branch.
Provenance evidence, corrected metrics, restore commits, and artifact hashes are in
[`admitted-key-reuse-results.md`](admitted-key-reuse-results.md).

## 25. Dual Main vs Planned Rebaseline

The OCI A1 sync-disabled dual rebaseline measured 3.04x same-source `main-btree`
vs planned and 3.59x exact-current-main vs planned across the 12-scenario
multiwriter geometric means. Both exceed the Very strong development-checkpoint
threshold; writer scaling from 16 to 64 remained effectively flat. These are
CPU/engine diagnostics on the root XFS filesystem, not a durability or adoption
result. The full scenario table, single-writer controls, source/harness proofs,
build provenance, and raw artifacts are in
[`dual-main-vs-planned-results.md`](dual-main-vs-planned-results.md).
