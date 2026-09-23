# Phase 4: Concurrency, scheduling, and group commit

## Scope

Phase 4 improves request scheduling around the Phase 3 transaction contract.
The physical B+Tree mutation path remains one ordered writer per shard. There
are no page latches, page-level concurrent writers, MVCC versions, range locks,
or a lock manager.

## Phase 3 implementation investigation

Before the changes, `AsyncShard` sent every internal `BatchRequest` through one
bounded Tokio channel, including `Get`, Query, Scan, and its internal
point-read grouping helper. The coordinator
owned one mutable `BTreeStore`, collected at most 64 requests, and started a
1 ms timer immediately after receiving the first request. A single-client
benchmark therefore waited for that timer on every request. Queue admission
used `send().await`, so callers waited for capacity instead of receiving an
explicit overload result.

Mutation preparation already serialized through `apply_transaction_group`.
Accepted transactions staged against the committed base plus earlier accepted
transactions. WAL append already kept one page-image sequence and one COMMIT
record per logical transaction, followed by one shared WAL sync.

## Concurrent committed read model

`AsyncShard` now builds a committed read view from the validated tree when it
starts. The view contains immutable decoded page objects held behind `Arc`s.
After a successful publication, changed decoded pages and the new root metadata
are installed under one `RwLock` write guard. Each ordinary Get, Query, and Scan
holds one read guard for its complete operation, so a single operation cannot
traverse a half-published root or leaf chain. Reads never consult staged overlay
pages.

The internal point-read grouping helper remains on the coordinator and retains
its one-state, input-order semantics for service-side condition evaluation. It
is not a network API; independent wire Gets use independent QUIC streams. A
write response is sent only after WAL durability and committed-view publication,
so a read issued after that response observes the new value.

The view retains one immutable object for each page that has been reachable in
the committed tree. This is bounded by the allocated tree rather than by the
number of requests, and it is intentionally simpler than a persistent or MVCC
history. The existing page cache remains the synchronous engine cache; the
committed read view is the async read-safety mechanism.

## Queueing and backpressure

The write/request channel remains bounded. Mutation and internal grouped-read
admission
uses nonblocking `try_send`: a full channel returns structured `Overloaded`,
which is retryable after load decreases. A closed coordinator remains an
internal lifecycle error. Ordinary committed reads do not consume write queue
capacity.

The default internal coordinator limits are:

- 256 queued requests;
- 64 logical requests per group;
- approximately 4 MiB of estimated request payload per group;
- a configurable maximum collection delay, disabled by default.

An oversized individual request is allowed to occupy a group by itself. A
bounded request count and FIFO admission prevent one group from growing without
limit. Conflicts remain ordinary `Conflict` results and do not degrade the
shard.

## Group formation policy

The previous policy started a fixed 1 ms collection timer for every first
request. The Phase 4 policy is:

1. receive the first queued request;
2. yield once so already-runnable callers can enqueue;
3. drain immediately available requests up to request and byte limits;
4. if at least one additional request arrived, optionally collect until the
   configured deadline; and
5. process the group in FIFO order.

The default deadline is zero. This avoids an artificial low-load sleep while
still forming large groups when work is already queued. A positive internal
deadline remains available for controlled experiments. The collector is
deterministically bounded and never splits one logical transaction across WAL
durability boundaries.

## WAL group commit

The WAL format and durability point are unchanged:

```text
Tx1 PAGE_IMAGE... COMMIT
Tx2 PAGE_IMAGE... COMMIT
Tx3 PAGE_IMAGE... COMMIT
                 one WAL sync
```

Every successful transaction retains its own commit LSN, batch identity, page
images, and COMMIT record. A conflict produces no page images or COMMIT record.
The existing framed append and complete-write loop is retained. Measurement did
not justify changing its deterministic header/payload/trailer fault points in
this phase; WAL framing and recovery behavior therefore remain unchanged.

## Dirty-page flushing

No background flush worker was added. Measurement did not justify another
concurrent storage actor, and the current policy remains understandable:

- only WAL-durable committed images enter `dirty_pages`;
- successful writes do not require a data-file flush;
- `flush` writes committed dirty pages, writes the selected superblock copy,
  and syncs the data file;
- a data-file write or sync failure follows the existing degraded-shard policy;
- dirty pages and retained WAL remain bounded only by the existing explicit
  flush/reopen operational boundary.

Phase 5 adds formal checkpointing and WAL reclamation while keeping the
coordinator as the write serialization point.

## Baseline measurements before changes

The existing Phase 3 release benchmark used 2,000 operations and a 1 ms
collection window. On this local filesystem it measured approximately:

| Async workload | Throughput | p50 where reported | WAL transactions/sync |
| --- | ---: | ---: | ---: |
| PUT, 1 client | 457 ops/s | 2.19 ms/op | 0.98 |
| PUT, 4 clients | 1,748 ops/s | 0.57 ms/op | 3.94 |
| PUT, 16 clients | 5,247 ops/s | 0.19 ms/op | 15.75 |
| PUT, 64 clients | 14,651 ops/s | 0.068 ms/op | 63.02 |

The local WAL sync was approximately 0.3 microseconds. This is not a durable
device model, so the Phase 4 benchmark injects deterministic sync delay in a
`DurableFile` wrapper.

## Phase 4 benchmark measurements

The checked-in `phase4-bench` reports operations/sec, p50/p95/p99 latency,
queue wait, collection delay, coordinator processing time, group size, WAL
bytes, page images, validation time, B+Tree preparation time, publication time,
WAL serialization/append time, sync time, conflicts, and overloads. Its
ordinary-read matrix covers cache capacities 0, 16, and 256 with 1, 4, 16,
and 64 clients. Reads report zero queued requests because they use the
committed view.

Representative zero-delay width-1 transaction results from 16 operations per
client were:

| Clients | Transactions/s | p50 | Transactions/sync |
| ---: | ---: | ---: | ---: |
| 1 | 23,008 | 40.0 us | 1.00 |
| 4 | 19,096 | 218.7 us | 4.00 |
| 16 | 17,742 | 909.8 us | 16.00 |
| 64 | 15,832 | 4.09 ms | 64.00 |

The higher-client latency is coordinator preparation and page-image work, not
an unconditional collection sleep. The 64-client group reached the configured
64-request limit.

## Injected sync-latency measurements

The benchmark wrapper sleeps only in its test `sync_data` implementation. It
does not change production durability behavior. Representative width-1 runs
used 1, 16, and 64 clients:

| Injected sync | 1-client p50 / tx/s | 16-client p50 / tx/s | 64-client p50 / tx/s |
| ---: | ---: | ---: | ---: |
| 0 ms | 40.1 us / 3,731 | 0.94 ms / 16,666 | 3.84 ms / 16,864 |
| 0.1 ms | 196.8 us / 4,808 | 1.09 ms / 14,613 | 3.99 ms / 16,171 |
| 1 ms | 1.10 ms / 712 | 1.99 ms / 8,130 | 4.89 ms / 12,531 |
| 5 ms | 5.10 ms / 196 | 5.99 ms / 2,678 | 8.96 ms / 6,971 |
| 10 ms | 10.10 ms / 99 | 10.99 ms / 1,428 | 13.99 ms / 4,459 |

The injected delay is implemented with an operating-system sleep and includes
normal scheduling variance. The result still demonstrates the required
relationship: a single client pays one sync per transaction, while concurrent
clients amortize one sync across the group without merging logical commits.

## Correctness and deterministic scheduling tests

The Phase 4 tests cover:

- read-after-success visibility;
- ordinary committed reads over root/leaf changes, Query, and Scan;
- group collection request limits with a controllable zero-delay collector;
- deterministic full-channel `Overloaded` admission;
- existing staged-state conflict ordering and separate commit LSNs;
- separate WAL COMMIT records with shared sync;
- WAL sync failure and degraded-shard behavior;
- randomized serializability differential testing and invariant checks;
- crash/recovery tests from Phases 1–3.

The scheduler tests use channels and a zero-delay configuration rather than
wall-clock race assertions. The injected-sync benchmark uses a durable-I/O
wrapper rather than a production sleep.

## Phase 5 readiness

The current durability relationship is:

```text
WAL retained history
    -> authoritative redo for every durable commit
committed dirty pages
    -> in-memory NO-FORCE publication, only after WAL sync
data-file flush
    -> writes committed dirty pages and syncs the data file
superblock state
    -> alternates durable root/allocator metadata images
checkpoint_lsn
    -> remains at the last formal checkpoint boundary; ordinary commits do not advance it
```

Phase 5 uses the coordinator FIFO to place checkpoint work between accepted
mutation groups. The write gate is held through data sync, checkpoint
superblock sync, WAL reset, and WAL reset sync; ordinary immutable reads keep
using the committed read view.
