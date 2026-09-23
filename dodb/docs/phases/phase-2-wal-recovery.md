# Phase 2: Write-ahead log and crash recovery

## Delivered

- Explicit versioned `DWAL` framing with little-endian fields, header and
  payload CRC32C checksums, trailing frame lengths, and database identity.
- `INIT`, full-page `PAGE_IMAGE`, and explicit `COMMIT` records.
- Strict record-LSN and batch sequencing with deterministic short-write loops.
- WAL-first commit ordering for `BTreeStore::open_with_wal` and production
  `open_path`, with the WAL sync as the durability point.
- One commit LSN for every mutation batch; document revisions and tombstones
  use that committed LSN.
- NO-STEAL committed dirty-page publication and an explicit data-file flush
  path. WAL remains retained from database creation.
- Startup recovery, torn-final-frame truncation, committed-unit filtering,
  full-page redo, superblock redo, idempotence, and post-recovery invariant
  checking.
- Distinct durability and recovery error categories in addition to I/O,
  corruption, unsupported-format, and invariant errors.
- Operation-indexed and named deterministic fault hooks, simulated durable-disk
  crash tests, structural recovery tests, corruption tests, and a real abrupt
  subprocess restart test.

## Phase 2 baseline semantics

`PreparedBatch` remained a private overlay and one storage batch was
all-or-nothing. Individual request errors aborted the entire preparation and
did not publish earlier requests. Phase 3 retains this low-level API for
compatibility, while its async coordinator maps independent mutation requests
to independent logical transactions that can share one WAL sync.

The single-file `BTreeStore::open` constructor remains as a format/testing
compatibility path without a separate WAL. Production path opening and the new
paired `open_with_wal` constructor use the Phase 2 protocol.

## Non-goals

Phase 2 does not add a user-facing multi-key transaction API, OCC, MVCC,
application-level backup formats, checkpoints, WAL compaction, replication, networking, or a
background flush service. The WAL API already treats a full set of page images
as one commit unit so Phase 3 can use the same recovery format for a validated
multi-key transaction.

## Validation

The test suite covers successful writes after data-file rollback, faults in WAL
headers/payloads/trailers and sync, missing commit records, named commit-boundary
injection, structural B+Tree changes, overflow/free-page reuse, torn tails,
middle corruption, wrong identity, recoverable page corruption, unrecoverable
page corruption, randomized deterministic crash/reopen differential sequences,
and a SIGKILL child-process restart loop.

The checked-in benchmark was run with 2,000 rows. On the recorded run, direct
small-value PUTs measured approximately 4,902, 6,462, and 7,044 ops/s for
cache capacities 0, 16, and 256, respectively, with one WAL sync per
operation. An overflow-value PUT run measured approximately 2,721 ops/s at
cache capacity 256. The async coordinator measured approximately 448, 1,678,
5,355, and 25,715 PUT ops/s for 1, 4, 16, and 64 clients. It used 65 WAL
syncs for 64 batches, amortizing about 63 operations per sync at 64 clients.
The 64-client run measured about 107.6 us average WAL append time and 0.4 us
average WAL sync time in this local environment. Small-value single-request
PUT used approximately 17.2 MiB of WAL for 2,000 operations; the 64-client
run used approximately 1.7 MiB because requests shared prepared batches and
page images. These are local benchmark observations, not compatibility
thresholds.
