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

T12 remains partial. Service opening does not yet dispatch experimental paged manifests through complete validated page opening, and the shared manager is not yet consumed by the paged index, source manager, loader, and read path. General operation-owned charges and per-shard validated paging configuration remain open. No tenant-density performance matrix was run.
