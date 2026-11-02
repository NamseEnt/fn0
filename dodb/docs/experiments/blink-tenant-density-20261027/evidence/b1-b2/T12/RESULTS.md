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

A shutdown-timeout retry regression test verifies that previously open shards stay registered when an in-flight open exceeds the drain deadline. After the blocked open exits, a later shutdown attempt drains the open attempt and retained shard.

After this fix, the full server library suite again passed with 15 tests, and the server all-target check and formatting check passed. Raw outputs are recorded as `logs/server-lib-tests-after-shutdown-fix.log`, `logs/server-all-targets-check-after-shutdown-fix.log`, and `logs/server-format-after-shutdown-fix.log`.

## Remaining T12 work

T12 remains partial. Service opening does not yet route paged manifests through complete validated page opening. A shared `BlinkResourceManager` and validated paging configuration are not yet passed into every shard. Open-tenant metadata accounting, task-owned resource charges, and early rejection of all unsupported feature/layout combinations remain open. These gaps prevent T12 acceptance and block the dependent service tasks. No tenant-density performance matrix was run.
