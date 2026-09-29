# Phase K Design

## Immutable Input

The worker receives a cloned `BlinkState`, the physical generation and page catalog identity, the immutable overlay slice, and a covered segment count capped at the four-segment soft limit. The input overlays and all page values are Arc-backed and immutable. Foreground commits append new segments to the currently published generation without changing the worker input.

## Thread and State

At most one `BackgroundMaterializer` exists per store. It owns one standard thread, one request channel, and one result channel. The thread blocks on the channel and exits after its single request. Its explicit lifecycle is `Requested`, `Building`, `ReadyToPublish`, `Failed`, and `Idle` during teardown.

The worker performs mutation replay, leaf/internal splits, overflow allocation, page encoding and CRC, page writes, data sync, checkpoint image write, checkpoint sync, and retired-page cleanup. Readers keep pinning immutable published generations. Logical writes continue to use durable WAL v4 followed by immutable overlay publication.

## Durable Order and Publication

The worker builds into inactive page IDs. It writes and syncs all new pages before writing and syncing the alternate superblock with the covered physical LSN and logical sequence. It writes retired pages as free only after the checkpoint watermark is durable. The store then compares the current base epoch and catalog Arc with the materializer input and verifies that the exact materialized overlay prefix remains at the front of the current overlay slice.

If compatible, the new `PublishedGeneration` uses the replacement base and the untouched suffix. A stale result is rejected and counted; because an unexpected base change after durable checkpoint creation would make continuing unsafe, the store is degraded until reopen. WAL reset runs only if the latest logical commit still equals the materialized LSN and sequence. When a suffix exists, the WAL remains intact so that recovery can filter the durable prefix and replay the suffix.

## Bounds and Backpressure

Four segments trigger a materialization request. Up to eight segments may coexist during the build. At eight, the writer waits for the current result and publication; if no background materializer can be created, the synchronous fallback runs. In-memory test files do not expose a cloneable worker handle and therefore use the synchronous path.

## Measurements

Metrics distinguish total wall time, Linux worker thread CPU time, data writes, data sync, checkpoint write and sync, publication pause, writer waiting, backpressure, overlay count/bytes, WAL retention/reclaim, failures, and stale results. On non-Linux hosts worker CPU time is reported as zero; OCI A1 is Linux and uses `CLOCK_THREAD_CPUTIME_ID`.
