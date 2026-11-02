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

## Follow-up Implementation Milestone

The paged writer now routes checkpoint begin, bounded output slices, and publish through the dedicated progress thread. Each output slice reacquires the fair tenant scheduler ticket and releases it after at most 128 pages, so another ready tenant can run between slices. Resident checkpoint execution remains on its existing route. The service records per-tenant debt, threshold duration, queue wait, and completed slices. The progress worker catches job panics and remains available for subsequent tasks.

The service tests also cover a checkpoint continuation returning behind another ready tenant, a failed progress task releasing its worker, and a panicking progress task not terminating the worker. The paged writer service test verifies checkpoint completion and reopen state. The concurrent tenant test verifies both a hot and a smaller tenant receive checkpoint service while the hot tenant spans multiple slices.

Validation from the follow-up milestone:

- `cargo test --manifest-path crates/dodb-server/Cargo.toml --lib`: 25 passed.
- `cargo test --manifest-path crates/dodb-server/Cargo.toml --features blink-demand-paging --lib`: 41 passed.
- `cargo check --manifest-path crates/dodb-server/Cargo.toml --all-targets`: passed.
- `cargo check --manifest-path crates/dodb-server/Cargo.toml --all-targets --features blink-demand-paging`: passed.
- `cargo clippy --manifest-path crates/dodb-server/Cargo.toml --all-targets --no-deps -- -D warnings`: passed.
- `cargo clippy --manifest-path crates/dodb-server/Cargo.toml --all-targets --features blink-demand-paging --no-deps -- -D warnings`: passed.

Logs: `server-default-library-final-2.log`, `server-default-all-targets-final-2.log`, `server-default-clippy-final.log`, `server-paging-library-final-2.log`, `server-paging-all-targets-final-2.log`, and `server-paging-clippy-confirmation-2.log`.

Failed attempts remain in `server-paging-library-final.log` (40/41 before the snapshot-compatibility fix), `server-paging-clippy-final.log` and `server-paging-clippy-confirmation.log` (initial lint findings), and `server-paging-clippy-confirmation-2.log` is the successful rerun after those findings were fixed.

This remains a partial milestone. Disk-capacity release and reader-pin release do not yet wake a deferred checkpoint, there is no global/per-tenant debt ceiling that applies foreground backpressure, and the required synchronized/staggered all-active pressure suite and deterministic maximum service-wait bound are not yet demonstrated. Full-foreground-pool interaction, long pinned-reader pressure, shutdown under queued maintenance, and recovery after capacity release remain open. T14 acceptance is therefore still open, and T15/T17 have not started.
