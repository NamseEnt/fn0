# Phase K Concurrency Evidence

## Deterministic Tests

The OCI A1 release package gate passed `background_materialization_keeps_committing_and_preserves_newer_suffix`. Its channel barrier pauses the materializer on the first physical write, commits a same-key replacement and a newer suffix key, then resumes publication. Assertions cover the old pinned reader, the new reader, suffix retention, the durable covered watermark, retained WAL, and reopen recovery.

The OCI release gate also passed `background_materializer_write_failure_keeps_overlay_and_wal_authoritative`. An injected page-write error before checkpoint writing leaves committed overlay values readable, retains logical WAL, permits later writes, and recovers the acknowledged values after reopen.

## Measured Concurrency

On the OCI A1 primary 64-writer width-16 uniform workload, background K reduced median writer-blocked time from J's 6.00 s to 2.28 s in the short diagnostic. The reduction was 61.99%, below the 90% target. K hit the hard overlay bound of eight segments, producing 320 median backpressure events and 2.58 s backpressure. The six-workload K matrix repeated the pattern: 327 median events, 2.74 s backpressure, and an eight-segment peak in its primary case.

K's materializer completed 360 jobs and consumed 1,440 segments in the six-matrix primary interval. WAL reclaim remained zero and 186.8 MB was retained at run end. The materializer therefore did not demonstrate a sustainable steady state under the tested arrival rate.

## Limits

The current tests do not cover every requested crash boundary, stale-result injection, or explicit request-collapse race. Publication pause is cumulative only; p50/p99 cannot be derived. Full details and raw source/binary/host provenance are in `raw/oci-a1-primary/` and `raw/oci-a1-durable/`.
