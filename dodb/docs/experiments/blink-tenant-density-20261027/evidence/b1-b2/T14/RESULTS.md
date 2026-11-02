# T14 Results: Fair Checkpoint Admission

Status: complete

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

T14 acceptance is complete for its correctness and progress gates. Four active tenants are exercised through actual service writes in synchronized and staggered pressure scenarios, each receives checkpoint service, and observed maximum request wait stays below the test's 10-second ceiling. A deterministic scheduler test separately bounds synchronized and staggered queue wait to three service turns. The old-reader integration holds a pinned root through writes and a capacity-waiting checkpoint, then verifies progress after root release. Existing tests cover a saturated queued-foreground category, capacity release, shutdown cancellation, worker failure, and idle/failed/cancelled ready tickets. These results are local correctness observations, not a performance, SLO, or tenant-capacity result. No defaults, sync behavior, or worker concurrency were changed.

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

The disk resource manager now supports bounded asynchronous capacity waiters. Its pending waiter count is capped by the configured tenant bound and included in disk snapshots. A waiter is notified when a reservation drops or persistent source usage is reconciled downward. Checkpoint output sizing runs on a blocking worker against the pinned target index; the resulting disk reservation is acquired before the pack is created and remains attached to the checkpoint job. The service uses a validated per-route capacity-wait timeout. This keeps a checkpoint from failing immediately when reclaimable retained-source accounting temporarily occupies the configured disk budget. Service shutdown signals paged checkpoint continuations; a continuation waiting for disk capacity drops its waiter registration and exits without publishing a partial checkpoint.

The paging service also maintains global and per-tenant checkpoint debt ceilings. Admission reserves a conservative worst-case WAL append before a mutating transaction can write. The bound includes maximum working-page images, superblock, page/commit frames, and initial WAL segment header. Failed or cancelled transactions release pending reservations. After a committed write, actual threshold-excess debt is recorded and the checkpoint is scheduled before snapshot publication. When a write is rejected by the debt ceiling, it prompts checkpoint scheduling; synchronous mode waits for that checkpoint and retries admission once, while asynchronous mode returns overload. Open rejects a configuration whose caps cannot admit one maximum transaction. The limits must also fit within configured global and per-tenant disk budgets.

Named tests and validation:

- `blink::resource::tests::disk_capacity_waiter_resumes_after_retained_source_release`: passed; persistent retained-source usage falls, the waiter wakes, and the requested reservation is granted.
- `blink::resource::tests::disk_capacity_waiters_are_bounded_and_cancellation_releases_a_slot`: passed; waiters are capped and task cancellation releases its slot.
- `tests::paged_checkpoint_resumes_after_disk_capacity_release`: passed through the writable service checkpoint path; it waits at the disk limit, resumes after capacity release, publishes, and clears the pending waiter.
- `tests::paged_service_shutdown_cancels_checkpoint_waiting_for_disk_capacity`: passed through the writable service route; shutdown cancels the capacity wait, drains the checkpoint task, and clears the waiter registration.
- `blink::resource::tests::disk_capacity_waiter_wakes_when_a_root_releases`: passed; a root-release signal wakes a bounded output-capacity waiter and releases its waiter slot.
- `blink::source_manager::tests::source_collection_resumes_after_the_last_old_root_releases`: passed; a pinned checkpoint source remains live until its root releases, then collection unlinks it.
- `tests::paged_checkpoint_progresses_while_foreground_queue_budget_is_full`: passed; checkpoint publication completes while the queued-foreground category is at its per-tenant byte ceiling. This does not fill every foreground category or prove a multi-tenant pressure bound.
- `tests::checkpoint_debt_admission_enforces_global_and_tenant_limits`: passed.
- `tests::paged_open_rejects_checkpoint_debt_cap_below_one_worst_case_transaction`: passed.
- `tests::paged_service_backpressures_writes_at_checkpoint_debt_limit`: passed; the service rejects at the configured ceiling and actual tenant debt stays at or below the cap.
- `blink::tests::maximum_paged_transaction_wal_reservation_includes_images_and_segment_header`: passed.
- `physical_checkpoint::tests::fair_ready_queue_bounds_synchronized_and_staggered_pressure_wait`: passed; under synchronized and staggered arrivals, queue-level maximum wait was 3 service turns. This is a deterministic scheduler test, not a service SLO result.
- `tests::foreground_queue_count_limit_rejects_excess_without_leaking_charges`: passed in isolation and in the serial full-suite run. A parallel full-suite run stalled at this test and was interrupted; the serial rerun passed all 45 paging server library tests at that point.
- `tests::cancelled_paged_write_keeps_charges_until_shutdown_drain`: passed with a bounded wait for asynchronous metadata-charge release after shutdown. An earlier full-suite run observed a transient 512-byte metadata charge immediately after shutdown; the failed logs are retained.
- `cargo test -p dodb-server --lib -- --test-threads=1`: 26 passed.
- `cargo test -p dodb-server --features blink-demand-paging --lib -- --test-threads=1`: 48 passed.
- `cargo test -p dodb-storage --features blink-demand-paging --lib -- --test-threads=1`: 322 passed, 4 ignored.
- Revalidation after root-release wakeup and output-to-retained reservation transfer: storage paging library 324 passed, 4 ignored; server paging library 48 passed.
- `cargo check -p dodb-server --all-targets`: passed.
- `cargo check -p dodb-server --all-targets --features blink-demand-paging`: passed.
- `cargo clippy -p dodb-server --all-targets --no-deps -- -D warnings`: passed.
- `cargo clippy -p dodb-server --all-targets --features blink-demand-paging --no-deps -- -D warnings`: passed.
- Storage-wide Clippy did not pass: pre-existing storage-source findings remain, with preserved output in `storage-paging-clippy-capacity-wake.log` and `storage-paging-clippy-lib-capacity-wake.log`; no lint suppressions were added.

Logs: `storage-paging-library-capacity-wake.log`, `server-default-all-targets-shutdown-cancel.log`, `server-paging-all-targets-shutdown-cancel.log`, `server-default-clippy-shutdown-cancel.log`, `server-paging-clippy-shutdown-cancel-final.log`, `server-paging-library-shutdown-cancel-final.log`, `server-paging-library-shutdown-drain-wakeup.log`, and `storage-paging-library-debt-cap-final.log`, alongside earlier pressure-bound and failed diagnostic logs in this directory.

The first foreground-category saturation probe could not reserve the full category because its aggregate per-tenant memory limit left no headroom for existing metadata charges. The corrected test retains the category ceiling and increases only the aggregate memory limits. The rejected probe is recorded in `server-paging-foreground-full-initial-capacity-reject.log`.

## Root-Release Wakeup Follow-Up

Paged checkpoint output capacity now listens for source-root lease releases. Before each reservation attempt, the checkpoint worker scans checkpoint packs and WAL segments through the source collector. If output capacity is still unavailable, the bounded waiter wakes on either capacity change or a root release, runs collection again, and retries against the same absolute deadline.

Checkpoint bytes are reserved under `CheckpointOutput` while the pack is being produced and transferred to `RetainedSource` at publication. If transfer or retention fails before manifest publication, the unpublished pack is removed when possible. The writable shard no longer keeps a redundant copy of its initial root handle; active operations continue to snapshot the writer's current root.

The source-manager test proves old-root pin release allows exact source unlinking. The resource test proves a root-release signal wakes and unregisters a bounded waiter. Existing writable-service tests prove output-capacity waiting resumes after capacity release and shutdown cancels a waiter.

The service integration exercises source retention and capacity wait through actual requests. `tests::paged_checkpoint_reclaims_old_reader_source_before_publication` writes 50 distinct 4 KiB values, pins the old service read root, publishes a checkpoint, confirms a retained WAL source remains readable through that root, and overwrites all 50 keys. It then holds a 700 KiB scratch reservation, starts another checkpoint, and waits until the bounded disk waiter is registered. Releasing the old reader lets the checkpoint finish while the scratch reservation is still held; the test then releases the blocker, shuts down, reopens, and verifies the latest value. The test does not claim that every pre-checkpoint WAL path is unlinked: a WAL segment can remain referenced by the current root. It also does not use total retained-source bytes as a collection oracle because the new checkpoint output increases that category.

During a diagnostic run of that same fixture, the configured disk cap was 64 MiB and observed peak accounted disk usage was 25,719,300 bytes. The peak is a local correctness-fixture observation, not an SLO or density result.

Validation after this integration addition:

- `cargo test -p dodb-server --features blink-demand-paging --lib -- --test-threads=1`: 49 passed; full output is retained in `server-paging-library-old-reader-lifecycle.log`.
- `cargo test -p dodb-storage --features blink-demand-paging blink::source_manager::tests --lib -- --test-threads=1`: 14 passed; full output is retained in `storage-paging-source-manager-final.log`.
- `cargo clippy -p dodb-server --features blink-demand-paging --all-targets --no-deps -- -D warnings`: passed.
- `rustfmt --check --edition 2024 crates/dodb-server/src/lib.rs` and `git diff --check`: passed.
- `cargo clippy -p dodb-storage --features blink-demand-paging --all-targets --no-deps -- -D warnings`: failed on 32 existing workspace findings; no finding referenced `source_manager.rs`. Full output is retained in `storage-paging-clippy-accounting-diagnostic.log`.

Rejected capacity-pressure probes for this integration are preserved in `server-paging-old-reader-capacity-probe-failures.log`. They reached either working-page/index limits or failed WAL disk reconciliation before the intended wait; none is counted as acceptance evidence.

The diagnostic probe found that service `transact_group` writes appended WAL bytes without reserving the growth or reconciling the reservation to the actual WAL directory size. The observed directory measured 2,085,008 bytes while the ledger accounted for 834,704 bytes, a 1,250,304-byte mismatch. `PagedTransactionWriter::transact_group` now reserves the checked aggregate append bound before writing and reconciles the reservation to actual directory usage after success or append failure. The old-reader capacity integration now registers its waiter and resumes after reader release with the scratch blocker still held. Exact WAL-directory and `DiskCategory::Wal` equality is asserted after initial service writes and after post-checkpoint transaction-group writes.

Two rejected assertion probes are retained in `server-paging-wal-accounting-fix-followup.log`. Requiring all old WAL paths to disappear was too strong because a current-root segment remained present. Requiring retained-source bytes to fall was also invalid because the second checkpoint adds a new retained pack; the combined waiter-resumption test is the direct capacity-release evidence.

The first service probe applied the artificial blocker to total disk capacity. That exhausted WAL growth before the checkpoint reached output reservation. A second probe reduced the retained-source category and prevented source-catalog reservation. Both failed attempts are retained as `server-paging-root-release-capacity-probe-failures.log`. The corrected service test limits only `CheckpointOutput`, leaving WAL and source-catalog budgets available.

New logs: `server-paging-root-release-capacity-retry.log`, `server-paging-capacity-shutdown-cancel.log`, `server-paging-root-release-capacity-probe-failures.log`, `storage-paging-root-release-collection.log`, `storage-paging-root-release-wakeup.log`, and `storage-paging-root-release-collection-compile-failure.log`.

T14 has no remaining acceptance gates. The service-pressure test fills the queued-foreground category for its checkpoint test, but does not claim that every resource category is simultaneously full. Debt-cap tests show configured foreground backpressure, not a multi-tenant SLO bound. T15 may proceed under the implementation plan; T16 and the comparison milestone remain gated by their dependencies and separate execution authorization.


## WAL Group Accounting and Combined Reader-Release Capacity Wakeup

`PagedTransactionWriter::transact_group` now accounts for the full checked WAL append bound before issuing a group append. It reconciles persistent WAL usage to the measured directory after both successful and failed append attempts, and reports durability errors if post-write reconciliation fails. The service integration asserts exact directory-to-ledger equality after grouped writes before and after checkpoint publication.

The combined old-reader and capacity-wait service test now passes. It observes one pending disk waiter while the scratch blocker remains held, drops the pinned reader, and observes checkpoint completion before releasing the blocker. Shutdown and reopen confirm the newest value. This establishes waiter progress after old-reader release under this fixture; it does not establish a tenant-capacity or SLO bound.

Validation after the WAL accounting fix:

- `cargo test -p dodb-server --features blink-demand-paging --lib -- --test-threads=1`: 49 passed; full output is in `server-paging-library-wal-accounting-fix.log`.
- `cargo test -p dodb-server --features blink-demand-paging paged_checkpoint_reclaims_old_reader_source_before_publication --lib -- --nocapture`: 1 passed.
- `cargo test -p dodb-storage --features blink-demand-paging blink::source_manager::tests --lib -- --test-threads=1`: 14 passed.
- `cargo clippy -p dodb-server --features blink-demand-paging --all-targets --no-deps -- -D warnings`: passed.
- `cargo fmt -p dodb-server -p dodb-storage -- --check`: passed. The workspace-wide `cargo fmt --all -- --check` could not run because Cargo resolves the separate vendored Deno workspace under the parent repository; this scoped package check passed.


## Final T14 Service-Pressure Acceptance

The four-tenant service test passes both synchronized and staggered start scenarios. All four tenants produce queued checkpoint requests, finish with no pending requests, and receive one or more service slices. The hot tenant receives 48 slices and each smaller tenant receives 15 slices in both scenarios. The final retained log measured maximum per-tenant request waits of 658,710,875 ns for synchronized starts and 696,811,958 ns for staggered starts. These values are fixture observations; the test's local bound is 10 seconds and does not define an operational SLO.

The complete paging server library suite passes 50 tests. Server paging Clippy with warnings denied and scoped formatting for the server and storage packages pass. Full output is retained in `server-paging-library-four-tenant-fairness.log`, and the synchronized/staggered metrics are in `server-four-tenant-fairness-final.log`. The rejected initial waiter-limit configuration is preserved in `server-four-tenant-fairness-probe-failures.log`.
