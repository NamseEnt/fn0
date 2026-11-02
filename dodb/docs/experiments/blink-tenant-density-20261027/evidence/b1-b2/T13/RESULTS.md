# T13 Foreground Admission Progress

Status: partial.

With `blink-demand-paging` enabled and a shared resource manager configured, the service now reserves owned request capacity before dispatch and retains that guard in the queued transaction through coordinator completion. Key and value vector capacities, request and transaction containers, queue entries, and channel clone capacity are included with checked arithmetic. The transaction preparation category is reserved before the coordinator clones requests into a group. The response budget is reserved while the service owns the response. Capacity failures return overload errors without waiting; shared-resource configurations reject zero coordinator queue or group limits.

Tests verify request backing-capacity accounting by rejecting a small-length request with deliberately oversized key/value allocations, release of request charges when preparation admission fails, response-budget rejection cleanup, global queued-byte backpressure across tenants, bounded queue-count rejection, and a cancelled caller whose committed request retains queued and preparation charges until the coordinator finishes. The latter test reads the committed value after cancellation.

## Verification

- `cargo test -p dodb-server --features blink-demand-paging --lib`: 25 passed, 0 failed.
- `cargo test -p dodb-server --lib`: 17 passed, 0 failed.
- `cargo test -p dodb-core --lib`: 6 passed, 0 failed.
- Default and feature `cargo check -p dodb-server --all-targets`: passed.
- `cargo fmt --check`: passed.
- Strict server Clippy stops in `dodb-storage` with 12 previously recorded findings in unrelated Blink/WAL functions; its full output is preserved in `logs/server-clippy-feature.log`. No lint suppressions were added.

## Remaining T13 Work

The service still opens the resident writer for writable tenant paths. The queued/preparation reservations currently cover the transferred request and group clone allocations, but the service has not yet reserved the full storage publication envelope and post-sync bookkeeping before WAL append. Complete the paged writer/coordinator integration and prove the reservation handoff with append, sync, and publication failures before marking T13 complete. Caller cancellation before enqueue, queue closure, and shutdown drain also need exact acceptance cases. Returned response memory leaves the service accounting boundary at API return. No tenant-density performance matrix was run.
