# T12 service open lifecycle progress

## Completed substep

The B-link tenant registry now tracks opening tenants and active shards under a short synchronous lock. It reserves shard capacity before opening, coalesces concurrent opens for one tenant, and bounds blocking open workers and waiting callers. Storage open and recovery run in `spawn_blocking` without holding the tenant registry lock. Reads joining an in-progress open wait for its authoritative shard. Failed opens release their tenant reservation, and shutdown waits for in-flight opens up to a configured deadline.

The new open worker, waiter, and shutdown limits reject zero values. Defaults are four open workers, 128 open waiters, and a 30 second shutdown deadline. Existing route and storage layout defaults remain unchanged.

## Verification

- `cargo test -p dodb-server --lib -- --test-threads=1`: 15 passed, 0 failed.
- `cargo check -p dodb-server --all-targets`: passed.
- `cargo fmt --check -p dodb-server`: passed.
- Logs are recorded under `logs/` and hashed in `receipt.json`.

The tests cover same-tenant open coalescing, a read joining a blocked open, unrelated tenant progress, failed-open reservation cleanup, invalid open limits, and bounded shutdown waiting.

A shutdown-timeout retry regression test verifies that previously open shards stay registered when an in-flight open exceeds the drain deadline. After the blocked open exits, a later shutdown attempt drains the open attempt and retained shard. A cancellation test aborts a blocked open, verifies its tenant reservation is removed, then successfully reopens with a single-worker limit, proving the worker permit was released.

A bounded-capacity test configures one shard slot, one open worker, and one waiter. While the first tenant open is stalled, it verifies that a second same-tenant waiter and a different-tenant open are rejected; the admitted waiter later receives the same shard as the leader.

After the shutdown retry, cancellation, and bounded-capacity changes, the full server library suite passed with 17 tests. The server all-target check and formatting check passed. The initial cancellation-test compile failure and corrected run are both preserved in `logs/`.

## Shared resource accounting substep

Added the default-disabled server feature `blink-demand-paging`, which enables the matching storage feature without activating a runtime route. Under that feature, `LocalTenantServiceConfig` accepts one shared memory/disk resource configuration and a per-open-tenant metadata charge. `BlinkTenantService` creates one manager, reserves each shard's metadata before opening files, and retains the reservation for the shard lifetime. Failed opens drop the reservation; shutdown releases reservations as shards drain. The default configuration leaves this accounting disabled.

The opt-in resource configuration rejects Legacy and non-base segmented WAL layouts before creating the data directory. The B-tree service rejects the option before filesystem access. A capacity test opens two tenants under one global metadata budget, rejects a third before its database path is created, shuts down to release both charges, and then successfully opens the third tenant.

- `cargo test -p dodb-server --features blink-demand-paging --lib -- --test-threads=1`: 19 passed, 0 failed.
- `cargo test -p dodb-server --lib -- --test-threads=1`: 17 passed, 0 failed; default feature behavior remains unchanged.
- `cargo check -p dodb-server --all-targets` and the same check with `--features blink-demand-paging`: passed.
- `cargo fmt --check -p dodb-server`: passed.
- The first shared-resource test run used the default Legacy layout and was rejected as expected; the preserved log records the test setup error, which was corrected by selecting segmented WAL with durable bases.

## Remaining T12 work

The service now selects an explicit opt-in `paged_read_only` route for the experimental marker, uses the shared manager for its index, source manager, and page loader, replays the recovery WAL, validates every indexed page, and serves Get/Query/Scan on bounded blocking workers. Writes fail as unsupported while the route is read-only. The route follows the sidecar directory convention now recorded in `B1-B2-CONTRACTS.md`.

T12 remains partial pending a successful service request against a complete paged fixture, resource and worker release checks under concurrent page loads, and shutdown behavior while page loads are active. Operation-owned request and write charges belong to T13. No tenant-density performance matrix was run.

## Recovery-aware paged storage open substep

Added `PagedManifestStore::open_with_recovered_pages` for the opt-in paged route. It validates the marker, manifest, durable index, source catalog, and recovery source identity; pins the durable checkpoint root; streams the recovery WAL into the version index; then opens a handle only after complete indexed-page validation. Missing recovery WAL files fail before an open can create or change them. The page resolver can load an exact indexed version while replay is building a newer root, including versions written earlier in the same WAL.

The checkpoint publication failure regression now reopens through this path and verifies that a synced commit newer than the selected checkpoint is recovered (`not-published`). This covers the acknowledged-WAL boundary that a manifest-only open would miss.

- `cargo test --manifest-path crates/dodb-storage/Cargo.toml --features blink-demand-paging --lib`: 319 passed, 0 failed, 4 ignored.
- `cargo check --manifest-path crates/dodb-storage/Cargo.toml --all-targets` and the same command with `--features blink-demand-paging`: passed.
- `cargo fmt --manifest-path crates/dodb-storage/Cargo.toml --check`: passed.
- `cargo clippy --manifest-path crates/dodb-storage/Cargo.toml --lib --features blink-demand-paging -- -D warnings`: blocked by 12 existing Clippy findings in unrelated Blink/WAL functions; no lint suppressions were added. The full output is preserved in `logs/storage-clippy-paged-recovery.log`.

The server now calls this path during tenant opening. A focused service test verifies that an incomplete manifest is rejected before the index, packs, WAL, or scratch directories are created and that its tenant metadata reservation is released.

- `cargo test -p dodb-server --features blink-demand-paging --lib -- --test-threads=1`: 20 passed, 0 failed.
- `cargo test -p dodb-server --lib -- --test-threads=1`: 17 passed, 0 failed; the default resident route remains unchanged.
- The storage recovery-open regression passed again after switching recovery WAL access to open-existing semantics.
