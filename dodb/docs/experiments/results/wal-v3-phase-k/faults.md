# Phase K Fault and Recovery Status

## Verified on OCI A1

- The full dodb release package gate passed, including Phase J synchronous materialization crash/recovery tests and the two retained Phase K concurrency/failure tests.
- A deterministic background page-write failure before checkpoint write leaves the old base authoritative, keeps committed overlays and logical WAL, permits reads, and recovers acknowledged writes after reopen.
- The worker writes replacement pages and syncs them before it starts writing the alternate checkpoint superblock.
- Runtime WAL reset is restricted to a durable checkpoint watermark covering the current WAL tail. A newer suffix prevents reclaim.

## Not Covered

The Phase K release tests do not inject faults at every requested background crash boundary. Missing deterministic cases include failure after request, after snapshot, during B-link build, during physical writes, before/during/after data sync, before/after checkpoint write and sync, before/after publication, before overlay retirement, and before/during WAL reclaim. Stale materializer result injection and multiple-request collapse are also not covered.

The present evidence is not an exhaustive crash matrix. The implementation conservatively degrades the current store if checkpoint writing has started and its durability is uncertain; reopen remains the recovery path. A materializer failure does not undo acknowledged logical transactions because the overlay and retained logical WAL remain authoritative.
