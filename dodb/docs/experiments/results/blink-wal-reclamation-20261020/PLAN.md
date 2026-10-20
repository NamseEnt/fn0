# B-link WAL Reclamation Comparison Plan

## Question and scope

Determine whether B-link sustains higher throughput and lower p99 than the minimal-native RocksDB TransactionDB contract baseline while physical WAL checkpoints and reclamation repeat during measurement. This experiment isolates checkpoint policy cost. It does not test a memory-bounded cache, page eviction, or operation on data sets larger than available memory.

## Source and preserved state

The implementation starts at `a5b37c23ab72d95c8d5d7bed263459fd7af141e2` in dedicated branch `experiment/blink-wal-reclamation-20261020` and worktree `/Users/namse/fn0-dodb-wal-reclamation`. The pre-existing `/Users/namse/dodb` checkout and `/Users/namse/fn0-dodb-rocksdb-baseline` checkout contain unrelated edits and untracked evidence; neither is modified by this work. Existing baseline results under `minimal-native-baseline-20261020` remain read-only.

## Source facts verified before implementation

- The B-link service opens `BlinkStore::open_path`, which uses the physical WAL, and enables planned execution.
- `AsyncBlinkShard` collects requests and calls `BlinkStore::apply_transaction_group` from its coordinator.
- Physical `BlinkStore::checkpoint` flushes dirty pages and syncs data, publishes and syncs checkpoint metadata, then resets the physical WAL. The WAL must remain intact until checkpoint metadata is durable.
- The B-link service path has no WAL-size-triggered automatic checkpoint policy.
- `DatabaseConfig.cache_capacity` sizes the B-tree page cache and does not impose a memory bound on the B-link path. B-link may retain the full page state in memory after checkpoint.
- The pre-existing experiment's RSS and database-size ceilings are stop conditions, not cache settings.
- Existing checkpoint metrics include logical-WAL materialization data and do not provide the complete physical-WAL automatic-checkpoint lifecycle, so this experiment will use dedicated coordinator metrics and bounded event capture.

## Planned files and boundaries

- `dodb/crates/dodb-server/src/lib.rs`: add an opt-in B-link checkpoint policy and bounded cumulative metrics, pass policy through `LocalTenantServiceConfig` to each coordinator, run physical checkpoints in `spawn_blocking` between committed groups, preserve acknowledged results, and fail the shard closed after checkpoint failure while draining queued responses consistently.
- `dodb/crates/dodb-rocksdb/src/bin/service_workload.rs`: parse `--blink-checkpoint-wal-bytes off|<positive-bytes>`, reject invalid values and non-B-link use, pass the policy to the service, and include applied policy and measurement checkpoint/group/sync observations in JSON.
- `dodb/crates/dodb-rocksdb/src/bin/unconditional_write_workload.rs`: provide the same policy CLI and result fields for the unconditional B-link comparison.
- `dodb/crates/dodb-server/tests` and relevant workload tests: verify threshold behavior, repeated reset-safe observations, error fan-out, and shutdown completion. Extend existing storage checkpoint fault tests only where a missing boundary requires it.
- `dodb/docs/experiments/results/blink-wal-reclamation-20261020/`: retain this plan, implementation and metric definitions, validation record, commands, raw output, resource traces, manifests, and the final comparison or an explicit reason the measurement gate could not be completed.

The DodbService protocol, RocksDB behavior, B-tree behavior, WAL format, grouping configuration, workload mix, durability, and default policy remain unchanged. No source comments or lint suppressions will be added.

## Checkpoint execution order and failure behavior

1. The existing coordinator applies each request group and completes its normal physical-WAL synchronization.
2. It sends each transaction's original result to its caller.
3. If the policy is enabled, a successful mutation group is followed by a WAL-size check. At or above the threshold, the coordinator moves the owned store into `spawn_blocking`, checkpoints it, and regains ownership before receiving the next group.
4. A failed checkpoint marks the coordinator failed, records the failure, closes intake, and returns one consistent durability error to queued and future writes. Earlier acknowledged commits remain committed. Shutdown joins the coordinator and observes any final in-flight checkpoint.

Checkpoint pause and queue waiting remain inside caller latency. The threshold is a trigger, not a hard cap; a group can exceed it, and idle time alone does not cause a checkpoint.

## Measurement fields

Capture applied policy and threshold; lifecycle started/completed/failed counts; total and maximum checkpoint duration; bounded per-checkpoint phase, UTC start/end, WAL bytes before/after, and reclaimed bytes; current/maximum WAL bytes and peak threshold excess; committed mutation transactions and mutation-bearing commit groups; actual transactions-per-group histogram and sample count; cumulative physical WAL sync counts observed before and after reset; measurement boundary database/WAL sizes; and each bounded resource-sample timestamp, process RSS, database bytes, and WAL bytes. Include omitted-event/sample counts. Workload phases (seed, warmup, measurement, drain, final validation, reopen) have separate counters and checkpoints during setup are excluded from measurement checkpoint totals.

These file-size observations are not cumulative disk writes or write amplification. Existing `bytes_written` is not used as a physical I/O total without source proof.

## Validation and measurement gates

Run formatting, focused checkpoint/coordinator tests, service transaction and read contract tests, and backend-wide contract tests before any performance run. Reuse physical checkpoint fault injection at page write/sync, metadata write/sync, and WAL reset boundaries; report function-level fault injection separately from process termination and make no power-loss claim.

Only after all correctness gates pass, use the already authorized OCI host `opc@217.142.246.204`, `/bench/zfs/db`, Rust 1.97.1, ARM64 `+crc`, RocksDB 11.8.1, and the existing OCI build/cache helper. Check host identity, tools, active processes, memory, disk and filesystem reserve first. Preserve at least the 3 GiB process-group RSS, 1,500 MiB `MemAvailable`, 4 GiB per-run DB, and 10 GiB filesystem-free stop limits. Never overlap build and measurement.

Run the four requested backends across A/rw20 and E with seed `20261013`, 10-second warmup, 30-second measurement, three independent repetitions, and predeclared rotating order (24 cells total). Keep each condition on a fresh DB and compare all per-client input fingerprints. Include measurement-window checkpoint pause and waiting in latency; report drain separately. Threshold cells with fewer than three completed measurement checkpoints are retained and marked insufficient for observing repeated reclamation. Stop or failed cells remain preserved and are never silently replaced.

Report full completed-sample latency counts, throughput, p50/p95/p99, outcomes, CPU, RSS, DB/WAL sizes, actual grouping and syncs, checkpoint phases and durations, source and binary hashes, command/configuration, RocksDB LOG, database manifests/hashes, final validation, and reopen results. Separate B-link off versus threshold policy impact from threshold B-link versus minimal-native path comparison. Do not infer equal-memory performance or long-term steady state from the 30-second window.

## Follow-up memory experiment

First, separately test whether the current engine can operate under an externally enforced memory ceiling; this is a feasibility test and may stop before the dataset exceeds usable memory. Second, after a separate engine change that implements a bounded page cache with eviction and reload, evaluate operation on a dataset larger than available memory. Changing `cache_capacity` or adding checkpoints does not implement that cache. Neither experiment is part of this task.
