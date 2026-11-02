# T14 Results: Fair Checkpoint Admission

Status: partial

## Implemented

- Replaced the process-wide checkpoint semaphore with a one-worker tenant-ready scheduler in the resident physical checkpoint path.
- The ready queue permits one pending ticket per tenant, preserves FIFO round-robin opportunities, and lets a cancelled waiter remove its ticket.
- Checkpoint coordinator admission now uses the tenant identity in both default and `blink-demand-paging` builds.
- The existing checkpoint request wait and threshold-excess measurements remain active.

## Validation

- `cargo test -p dodb-server physical_checkpoint::tests --lib -- --nocapture`: 4 passed.
- `cargo check -p dodb-server --all-targets`: passed.
- `cargo check -p dodb-server --all-targets --features blink-demand-paging`: passed.
- `cargo test -p dodb-server paged_writer_service_commits_and_recovers_service_requests --features blink-demand-paging -- --nocapture`: 1 passed.
- `cargo test -p dodb-server checkpoint_debt_metrics_track_excess_time_and_request_wait --lib -- --nocapture`: 1 passed.

Logs are retained in this directory's `logs/` subdirectory.

## Acceptance Still Open

This change is an admission-queue milestone, not T14 acceptance. Checkpoint work still holds the permit for an entire resident checkpoint. The paged writer has no service-integrated resumable checkpoint continuation. Global and per-tenant memory/disk debt ceilings, debt-driven eligibility, charged cursor/output transfer between slices, capacity and pin-release wakeups, and measured all-active pressure service bounds are not implemented or demonstrated. The tests above establish queue order and waiter cancellation only; they are not a performance or capacity result.

Do not start T15 or the T17 comparison matrix until the remaining T14 gates and the plan's dependency checks are accepted. No defaults, sync behavior, or worker concurrency were changed.
