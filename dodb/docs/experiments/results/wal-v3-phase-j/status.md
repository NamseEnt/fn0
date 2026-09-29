# Phase J Status

## Current Gate

J0 through J3 passed on 2026-09-29. J4 completed its durable six-scenario matrix on OCI A1. Its performance gate failed, so the sustained run and conditional RocksDB comparison were correctly skipped. Read representation measurements are retained from J0 and remain within the requested target.

The retained limit is four immutable overlay segments. The OCI A1 microbenchmark measured a worst point GET ratio of 1.459x, a worst Query/Scan ratio of 1.745x at limit 8, and a one-overlay Scan ratio of 1.490x at limit 8. The four-overlay Scan ratio was 1.576x at limit 128 and 1.393x at limit 4096. This replaces the Phase I whole-result merge behavior that reached 49.7x for Scan.

J0 source and measured binary provenance are in `raw/j0/run-order.jsonl`. The 37 result rows and complete test output are retained under `raw/j0/`; perf counters are in `perf/`. J1 run output and perf counters are under `raw/j1/` and `perf/`.

The source commit for these measurements is `ffe774c58da1157f6e735ee75262efd978022757`. The Phase J branch was created from Phase I final commit `29f4f4bbc3172e7ef8c09e7bfcfa00b3bb54e814` in `/Users/namse/dodb-phase-j`.

J1's `cargo test --workspace --release --no-fail-fast` passed at source `824188d906b367e57d5b94df11096015a0cdf26f`: 150 storage tests passed, with 3 ignored microbenchmarks, and all workspace integration and recovery tests passed. J2's same command passed at the current J2 source with 160 tests passed and 4 ignored. Raw J1 and J2 output is retained under `raw/j1/` and `raw/j2/`.

The supplied canonical owner is `namseent/dodb`. GitHub returned “Repository not found” for that repository during this run, and the existing local `origin` still points to `namse/dodb`. No remote URL was changed and no push was attempted. The benchmark source was transferred to OCI as Git bundles; measured checkouts retained `.git` metadata and clean source commits.

## Implemented

- `PublishedGeneration` now holds an immutable base catalog and an immutable overlay slice under one generation pin.
- Overlay entries use sorted packed slots and segment-local key/value bytes. Point lookup uses a segment-local Bloom filter before binary search.
- Query and Scan use a bounded streaming merge over the base leaf chain and sorted overlay cursors. Newest overlay values win, tombstones suppress older values, and visible-row limits stop the merge early.
- Base-only reads retain the original H1 path.
- Four segments are the explicit initial bound. Preparing a view with more segments is rejected.

## J1 CPU Gate

The CPU-only group prototype passed the requested 64-transaction, width-16 gate. H1 physical execution used 162,035 ns and about 481,927 user cycles per transaction. Logical overlay groups used 15,218–15,996 ns and about 45,617–47,956 user cycles per transaction across 0–3 existing segments: 9.39–9.87% of H1 CPU time. Allocations fell from 297.219 to 87.734 per transaction. Segment build was about 2.31–2.33 us/transaction and publication about 10 ns/transaction. J1 met the <=70% CPU gate.

This is an isolated volatile CPU prototype over `MemoryFile`, not a durable write result and not production API routing. Revision values in its `TransactionResult` temporarily map to the existing `commit_lsn` field. WAL encoding, sync, crash recovery, and materialization are J2/J3 work.

## J2 Durable Logical Write Path

`BlinkStore::open_with_logical_wal` explicitly opens WAL v4. Logical transaction admission checks the current `PublishedGeneration` plus ordered group-local mutations, writes only committed logical Put/Delete records, syncs once for the transaction group, then publishes one packed immutable segment. The durable write path does no B-link routing, leaf COW, split-fit calculation, page encoding, or PageDelta generation. The retained `open_with_wal` path reads/writes WAL v3 and remains the H1 reference.

WAL v4 validates CRC32C frame checksums, record order/index, batch identity, transaction digest, and commit sequence. Recovery ignores request conditions, replays only complete committed mutations, coalesces them into one packed segment, supports later writes and a second reopen, and distinguishes physical Commit frame LSN from logical `TransactionResult.revision`.

J2 passed its randomized differential and fault tests plus the full workspace release suite. The 64-transaction, width-16 WAL byte-accounting control recorded 2,704.062 WAL bytes and 17 frames per transaction with one sync per group. This was a `MemoryFile` accounting run, not durable throughput.

## J3 Materialization and Reclaim

The logical `checkpoint()` path now materializes all current segments into a copy-on-write B-link base using inactive page IDs. Data pages sync before the version 3 superblock publishes physical LSN and logical sequence watermarks. The immutable view swaps after that sync; retired page IDs are then rewritten as free pages and synced; only then does WAL v4 reset and reclaim the covered prefix.

When the four-segment bound is reached, the writer runs this maintenance synchronously before admitting the next group. WAL v4 also enforces a 256 MiB pre-append cap. Checkpoint reports segment count and bytes, transaction lag, pages/bytes written, reclaim bytes, and total pause duration. Repeated test materializations reuse page capacity after a single expansion. Crashable-file tests cover crashes before and after the checkpoint watermark, interrupted retired-page cleanup, every WAL reset stage, ENOSPC write failure, and data-sync failure.

The final `cargo test --workspace --release --no-fail-fast` passed at source commit `0ec2528a7513e507b2033aba095a937373e754b7`: 162 storage unit tests passed, 4 microbenchmarks were ignored, and all workspace integration, recovery, and testkit tests passed. The complete log is `raw/j4/j4-workspace-release.log`. An earlier instrumentation assertion failed because the test contained only one explicit materialization; that expectation was corrected and the final full run passed. The failed run is retained as `raw/j4/j4-pre-fix-workspace-release-failed.log`.

## J4 Durable Gate

The OCI A1 run used the ZFS dataset `/bench/zfs/db/phase-j-data`, real sync, 2 Tokio workers, 2 Blink workers, 2-second warmup, 10-second measurement, and three same-session interleaved H1/J repetitions per workload. Six scenarios were completed. H1 source/binary SHA are recorded in `raw/j4/durable/run-order.jsonl`; the source commits were H1 `29f4f4bbc3172e7ef8c09e7bfcfa00b3bb54e814` and J `0ec2528a7513e507b2033aba095a937373e754b7`. Host and binary detail is in `raw/j4/host-info.txt`; complete JSONL and console logs are in `raw/j4/durable/`.

J/H1 was 0.720x for the primary 64-writer width-16 uniform workload. The width-1 guardrails were 0.376x at 64 writers and 0.312x at 16 writers. J also reached 0.473x for 16-writer width-16 uniform, 0.413x for compact, and 0.395x for spread. This fails both the 1.20x primary comparison gate and the 0.95x width-1 protection gate.

The v4 durable path wrote 2,717.1 WAL bytes per transaction for the primary workload versus 2,672.0 for H1. It emitted 17 logical frames per width-16 transaction versus 16.60 H1 frames in the uniform case. Compact/spread wrote 2,748.3/2,702.1 bytes per transaction, while H1's PageDelta path used 536.0/663.3 bytes and three frames per transaction. J materialized 270 times in the primary run; average/max maintenance pause was 22.90/31.22 ms and cumulative materialization time was 6.18 seconds. Per-scenario medians and tail latencies are in `tables.md`.

J0 measured 0/1/2/4 overlays on the same pinned 8,192-key base. At four overlays, maximum GET overhead was 1.459x; worst small-limit Query/Scan was 1.745x/1.710x, and large Scan was 1.442x. The zero-overlay row measures the base-only fast path that J3 republishes after materialization. J3 verifies values, ordering, revisions, and pinned-view correctness after materialization, but no separate post-checkpoint read latency run was made. This keeps the read claim scoped to the measured representation.

The 120-second sustained run and RocksDB comparison were not run because their required throughput gates failed. No throughput or durability result is inferred from the J1 CPU prototype or J2 MemoryFile byte-accounting run.

## Final Decision

2. Keep committed overlay plus WAL v4 experimental only. The primary durable throughput was 0.720x H1 and width-1 protection was 0.376x/0.312x. The chosen next bottleneck is the synchronous full-overlay-set materialization pause.

## Not Yet Implemented

The J4 matrix and gate decision are complete. Code, raw results, and report artifacts are committed on the local Phase J branch. GitHub could not resolve the supplied `namseent/dodb` repository, while the existing `origin` still points to `namse/dodb`; publication was therefore not attempted against that unrelated remote.
