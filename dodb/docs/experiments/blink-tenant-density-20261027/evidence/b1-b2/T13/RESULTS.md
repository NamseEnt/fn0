# T13 Foreground Admission Progress

Status: complete.

With `blink-demand-paging` enabled and a shared resource manager configured, the service now reserves owned request capacity before dispatch and retains that guard in the queued transaction through coordinator completion. Key and value vector capacities, request and transaction containers, queue entries, and channel clone capacity are included with checked arithmetic. The transaction preparation category is reserved before the coordinator clones requests into a group. The response budget is reserved while the service owns the response. Capacity failures return overload errors without waiting; shared-resource configurations reject zero coordinator queue or group limits.

The opt-in writable paged route now opens the recovered manifest and active WAL through `PagedTransactionWriter`. Writes reserve the server request and preparation charges before dispatch, then use the writer's storage reservations before WAL append and publish a new read snapshot only after commit. The active WAL is resumed after the manifest committed-prefix LSN. This service-open writer emits full page images because paged recovery currently needs each active-WAL delta base to be present in that same WAL history; the existing writer constructors retain their delta behavior.

Tests verify request backing-capacity accounting by rejecting a small-length request with deliberately oversized key/value allocations, release of request charges when preparation admission fails, response-budget rejection cleanup, global queued-byte backpressure across tenants, bounded queue-count rejection, and cancellation both before queue insertion and after the commit worker starts. The paged writer route rejects a full worker before dispatch and releases both foreground charges. A barrier test cancels the caller during a paged write, starts service shutdown, and confirms that charges remain live through worker completion and drain to zero afterward. A closed coordinator rejects a request without retaining either charge.

Service fault tests inject failures before WAL append, during WAL sync, and after sync before generation publication. After each failure, queued and preparation charges are zero. Reopen proves the pre-append write is absent, the sync-failed write is absent or fully present, and the post-sync publication-failed write is present. The paged service test also commits a Put, reads it through the published snapshot, shuts down, reopens, and reads the recovered value. Storage writer tests cover full and incremental checkpoint publication, delta behavior, sync failures, and crash recovery.

## Verification

- `cargo test -p dodb-server --features blink-demand-paging --lib`: 31 passed, 0 failed.
- `cargo test -p dodb-server --lib`: 17 passed, 0 failed.
- `cargo test -p dodb-storage --features blink-demand-paging --lib`: 319 passed, 0 failed, 4 ignored.
- `cargo test -p dodb-core --lib`: 6 passed, 0 failed.
- Default and feature `cargo check -p dodb-server --all-targets`: passed.
- `cargo fmt -p dodb-server -p dodb-storage --check`: passed.
- Strict server Clippy stops in `dodb-storage` with 12 existing findings in Blink/WAL functions before reaching the server target; its full output is preserved in `logs/server-clippy-feature-final.log`. No lint suppressions were added.

## Next Tasks

T13 acceptance is complete. Returned response memory leaves the service accounting boundary at API return. Continue with T14 fair maintenance scheduling and T15 service crash/write/crash acceptance before the T16 readiness gate. No tenant-density performance matrix was run.
