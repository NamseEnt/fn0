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

## Acceptance Status

T14 remains partial. The implemented fairness and resource-control mechanisms below have focused correctness coverage, but deterministic synchronized/staggered all-active maximum service-wait bounds and the full pressure/shutdown interactions remain unproven. These tests are not a performance, SLO, or tenant-capacity result. Do not start T15 or the T17 comparison matrix until the remaining T14 gates and the plan's dependency checks are accepted. No defaults, sync behavior, or worker concurrency were changed.

## Follow-up Implementation Milestone

The paged writer now routes checkpoint begin, bounded output slices, and publish through the dedicated progress thread. Each output slice reacquires the fair tenant scheduler ticket and releases it after at most 128 pages, so another ready tenant can run between slices. Resident checkpoint execution remains on its existing route. The service records per-tenant debt, threshold duration, queue wait, and completed slices. The progress worker catches job panics and remains available for subsequent tasks.

The service tests also cover a checkpoint continuation returning behind another ready tenant, a failed progress task releasing its worker, and a panicking progress task not terminating the worker. The paged writer service test verifies checkpoint completion and reopen state. The concurrent tenant test verifies both a hot and a smaller tenant receive checkpoint service while the hot tenant spans multiple slices.

Validation from the follow-up milestone, before the capacity-wakeup and debt-cap additions:

- `cargo test --manifest-path crates/dodb-server/Cargo.toml --lib`: 25 passed.
- `cargo test --manifest-path crates/dodb-server/Cargo.toml --features blink-demand-paging --lib`: 41 passed.
- `cargo check --manifest-path crates/dodb-server/Cargo.toml --all-targets`: passed.
- `cargo check --manifest-path crates/dodb-server/Cargo.toml --all-targets --features blink-demand-paging`: passed.
- `cargo clippy --manifest-path crates/dodb-server/Cargo.toml --all-targets --no-deps -- -D warnings`: passed.
- `cargo clippy --manifest-path crates/dodb-server/Cargo.toml --all-targets --features blink-demand-paging --no-deps -- -D warnings`: passed.

Logs: `server-default-library-final-2.log`, `server-default-all-targets-final-2.log`, `server-default-clippy-final.log`, `server-paging-library-final-2.log`, `server-paging-all-targets-final-2.log`, and `server-paging-clippy-confirmation-2.log`.

Failed attempts remain in `server-paging-library-final.log` (40/41 before the snapshot-compatibility fix), `server-paging-clippy-final.log` and `server-paging-clippy-confirmation.log` (initial lint findings), and `server-paging-clippy-confirmation-2.log` is the successful rerun after those findings were fixed.

## Capacity-Wakeup and Debt-Cap Follow-up

The disk resource manager now supports bounded asynchronous capacity waiters. Its pending waiter count is capped by the configured tenant bound and included in disk snapshots. A waiter is notified when a reservation drops or persistent source usage is reconciled downward. Checkpoint output sizing runs on a blocking worker against the pinned target index; the resulting disk reservation is acquired before the pack is created and remains attached to the checkpoint job. The service uses a validated per-route capacity-wait timeout. This keeps a checkpoint from failing immediately when reclaimable retained-source accounting temporarily occupies the configured disk budget.

The paging service also maintains global and per-tenant checkpoint debt ceilings. Admission reserves a conservative worst-case WAL append before a mutating transaction can write. The bound includes maximum working-page images, superblock, page/commit frames, and initial WAL segment header. Failed or cancelled transactions release pending reservations. After a committed write, actual threshold-excess debt is recorded and the checkpoint is scheduled before snapshot publication. When a write is rejected by the debt ceiling, it prompts checkpoint scheduling; synchronous mode waits for that checkpoint and retries admission once, while asynchronous mode returns overload. Open rejects a configuration whose caps cannot admit one maximum transaction. The limits must also fit within configured global and per-tenant disk budgets.

Named tests and validation:

- `blink::resource::tests::disk_capacity_waiter_resumes_after_retained_source_release`: passed; persistent retained-source usage falls, the waiter wakes, and the requested reservation is granted.
- `blink::resource::tests::disk_capacity_waiters_are_bounded_and_cancellation_releases_a_slot`: passed; waiters are capped and task cancellation releases its slot.
- `tests::paged_checkpoint_resumes_after_disk_capacity_release`: passed through the writable service checkpoint path; it waits at the disk limit, resumes after capacity release, publishes, and clears the pending waiter.
- `tests::checkpoint_debt_admission_enforces_global_and_tenant_limits`: passed.
- `tests::paged_open_rejects_checkpoint_debt_cap_below_one_worst_case_transaction`: passed.
- `tests::paged_service_backpressures_writes_at_checkpoint_debt_limit`: passed; the service rejects at the configured ceiling and actual tenant debt stays at or below the cap.
- `blink::tests::maximum_paged_transaction_wal_reservation_includes_images_and_segment_header`: passed.
- `tests::foreground_queue_count_limit_rejects_excess_without_leaking_charges`: passed in isolation and in the serial full-suite run. A parallel full-suite run stalled at this test and was interrupted; the serial rerun passed all 45 paging server library tests.
- `cargo test -p dodb-server --lib -- --test-threads=1`: 25 passed.
- `cargo test -p dodb-server --features blink-demand-paging --lib -- --test-threads=1`: 45 passed.
- `cargo test -p dodb-storage --features blink-demand-paging --lib -- --test-threads=1`: 322 passed, 4 ignored.
- `cargo check -p dodb-server --all-targets`: passed.
- `cargo check -p dodb-server --all-targets --features blink-demand-paging`: passed.
- `cargo clippy -p dodb-server --all-targets --no-deps -- -D warnings`: passed.
- `cargo clippy -p dodb-server --all-targets --features blink-demand-paging --no-deps -- -D warnings`: passed.
- Storage-wide Clippy did not pass: pre-existing storage-source findings remain, with preserved output in `storage-paging-clippy-capacity-wake.log` and `storage-paging-clippy-lib-capacity-wake.log`; no lint suppressions were added.

Logs: `storage-paging-library-capacity-wake.log`, `server-default-library-capacity-wake.log`, `server-paging-library-capacity-wake.log`, `server-default-all-targets-debt-cap-final.log`, `server-paging-all-targets-debt-cap-final.log`, `server-default-clippy-debt-cap-final.log`, `server-paging-clippy-debt-cap-final.log`, `server-paging-library-debt-cap-serial.log`, `server-default-library-debt-cap-final.log`, and `storage-paging-library-debt-cap-final.log`, alongside earlier debt-cap-specific logs in this directory.

Remaining T14 gates: the capacity waiter is proven to resume after retained-source accounting is released, but the complete old-reader pin-release lifecycle has not been shown to release retained bytes and resume checkpoint publication. Synchronized and staggered all-active pressure runs with a deterministic maximum service-wait bound are not implemented. Full foreground-pool interaction, a long pinned reader under pressure, shutdown with queued capacity waiters, and recovery after capacity release remain unverified. Debt-cap tests show configured foreground backpressure, not a multi-tenant SLO bound. T14 acceptance remains open; T15, T16, and T17 have not started.
