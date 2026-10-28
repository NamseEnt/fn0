# Common backend memory budget and pressure accounting

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

## Scope

Continue from `14d86ec1b` in the existing worktree without delegation. Add opt-in Linux cgroup v2 enforcement to the density driver for B-link and both contract-equivalent RocksDB routes. Preserve all engine defaults, synchronous WAL, transactions, revisions, read contracts, and stored evidence. This increment prepares pressure comparisons; it does not establish a density winner or implement B-link eviction.

Require an explicitly delegated, empty cgroup directory plus explicit `memory.max` and `memory.high` byte settings. Enforce the common parent budget across the whole cell, including charges retained from earlier epochs. Create separate setup, measurement, oracle, and reopened-oracle groups with identical limits, zero swap allowance, and group OOM handling. Attach each worker before constructing its runtime or opening a database. Keep the driver outside the backend budget. Fail before database creation if enforcement is unavailable or configuration is incomplete. Verify actual kernel settings and membership; do not relabel cache/WBM settings as process limits.

Record periodic and boundary `memory.current`, available kernel `memory.peak`, `memory.stat`, local memory events, swap usage, and PSI observations. Preserve final group observations and partial diagnostics after worker failure or OOM. Bound sampling storage and expose truncation and gaps. Kernel peak is per epoch; sampled measurements are not instantaneous peaks. Cgroup charging is not exact RSS or retained heap and excludes uncharged ZFS ARC, host caches, and driver memory. Do not change host-wide ARC/swap policy.

Add opt-in periodic worker observations of shared cache/WBM usage, per-tenant RocksDB pressure properties, and existing B-link checkpoint metrics. Count shared cache/WBM only once. Keep unsupported retained heap, pin ages, and queue representations explicit rather than inventing byte estimates. Preserve existing request, input-fingerprint, process-identity, full-population, and reopen checks.

## Disk preparation

Inventory the designated OCI host before cleanup and record explicit deletion targets and before/after capacity. Prefer regenerable Cargo outputs and temporary build products. Keep the pinned RocksDB source/archive, current build cache needed for this increment, source checkouts/Git objects, frozen accepted binaries, raw logs/reports, and failed databases. The user authorizes reclaiming unnecessary experiment disk usage; archive or retain non-regenerable evidence before removing any corresponding original. Do not wipe the filesystem or touch application/production data. Maintain the 10 GiB benchmark reserve.

## Validation and stop conditions

Use Linux on `opc@217.142.246.204` with existing SSH identity and pinned RocksDB archive. Record source/binary hashes and use clean committed source for final runtime acceptance. Run targeted parser/accounting tests, existing session/read/transaction checks, changed-target Clippy review, and default/A1-feature compatibility links. Preserve failures and avoid rerunning successful cells for reporting changes.

Prove actual kernel enforcement with an isolated anonymous-allocation probe whose memory exceeds its cgroup budget; require local OOM evidence while the controller survives. Then run small matched B-link/shared-RocksDB correctness cells with identical explicit budgets, all-active and sparse-active populations, periodic pressure observations, and clean reopen validation. Freeze commands and live capacity before execution. Keep pressure-probe failures distinct from accepted correctness cells and preserve their evidence. Stop workload execution on reserve breach, identity/provenance mismatch, or unexpected host pressure. Do not run a broad E2/E3/E4 matrix in this increment.

The acceptance controller uses UID 0 for both backend paths because the SSH session is outside the user's delegated subtree. It creates only dedicated `/sys/fs/cgroup/dodb-density-*` groups and keeps the driver outside each cell's budget. It does not change existing user.slice delegation or host-wide policy. Frozen positive cells use 17 registered tenants, 17 active or 3 active plus 2 opened-idle, 256 seeded rows each, 4 KiB values, write-heavy mixed-width transactions, 10 arrivals/s per active tenant for three seconds, and a 256 MiB maximum / 192 MiB high budget. A separate expected-failure cell uses 4 MiB for both limits to verify that the real driver preserves diagnostics after worker OOM. These are correctness and enforcement checks, not sustained pressure or SLO measurements.

Record outcomes and remaining gaps in `MEMORY-PRESSURE-RESULTS.md`, update continuation documents, and commit/push only task-owned changes. Keep issue #126 open and do not merge or deploy.

## Reference

[Linux cgroup v2 documentation](https://docs.kernel.org/admin-guide/cgroup-v2.html) defines memory.high reclaim/throttling, memory.max OOM enforcement with possible temporary overshoot, separate swap policy, and charging/migration boundaries. Validate available files against the running kernel rather than assuming every documented counter exists.
