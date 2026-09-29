# Phase K Concurrency Evidence

## Deterministic Test

`background_materialization_keeps_committing_and_preserves_newer_suffix` uses a test file with a channel barrier on the materializer's first physical write. After the materializer reaches that barrier, the writer commits a new value to a key in the covered prefix and another key in the newer suffix. The materializer resumes and publishes the four-segment prefix.

Assertions verify that the old pinned reader still sees its prior value and does not see the later key; the new reader sees the latest same-key value and the suffix key; two newer segments remain published; the durable checkpoint sequence stops at the covered prefix; and WAL remains larger than its INIT frame. A later checkpoint and reopen recover the latest values.

## Failure Test

`background_materializer_write_failure_keeps_overlay_and_wal_authoritative` injects a page-write failure before checkpoint writing begins. The failure is counted, the store remains writable, acknowledged overlay values stay readable, and reopen reconstructs them from the retained logical WAL.

## Limits

The test barriers cover concurrent commit during physical writes, same-key replacement, suffix preservation, pinned readers, retryable pre-checkpoint failure, and recovery after a later checkpoint. The full per-stage background crash matrix, stale-result injection, explicit multiple-request collapse test, and reader latency regression measurements remain pending. Existing synchronous Phase J crash/fault tests continue to pass.
