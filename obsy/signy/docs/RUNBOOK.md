# Operations runbook

Getting from nothing to a running process is [`DEPLOYMENT.md`](DEPLOYMENT.md).
Setting meanings are in [`CONFIGURATION.md`](CONFIGURATION.md), and design rationale is in
[`ARCHITECTURE.md`](ARCHITECTURE.md). This document covers only **what to look at and what to do when something goes wrong**.

---

## Deployment prerequisites — start here

This engine is **single-machine, single-writer**. Breaking that assumption corrupts data.
[`DEPLOYMENT.md`](DEPLOYMENT.md) has the `docker run` command, the gateway contract and the free-tier
defaults that satisfy the four points below; this is the short form of why they exist.

1. **The container is disposable and the data directory is not.** The WAL in `SIGNY_DATA_DIR` is the
   **only copy** of data acked since the last flush. Bind-mount it from the host, and never run without
   that mount — a container's own filesystem goes away with the next `docker rm`, which the upgrade in
   [`DEPLOYMENT.md`](DEPLOYMENT.md) §9 does every time. If this ever moves to an orchestrator, the same
   rule reads: StatefulSet + fixed PV with the volume following the pod, never Deployment + emptyDir.
2. **Give the stop no deadline.** Force-flush retries without a hard timeout during shutdown, and Docker's
   default stop timeout is **10 seconds** — a SIGKILL in the middle of it. `--stop-timeout=-1` at
   `docker run` makes the daemon wait; under an orchestrator, `terminationGracePeriodSeconds` is the same
   setting with a floor instead of an off switch, and 10 minutes is the minimum worth setting.
   The point either way is that the decision to give up should be an operator's
   (`docker kill --signal=USR1 signy`, which exits non-zero and says so) rather than a timer's
   (SIGKILL, which says nothing).
3. **Run one container against one prefix.** A second instance claims the writer epoch and the first one is
   fenced and killed. This is intentional, but such a configuration must not be created — which is why the
   upgrade stops the old container before starting the new one, rather than the other way around.
4. **Keep the listening address inside the trust boundary.** TLS and authentication are outside this process,
   and a client names its own tenant. Publishing a port with `-p` writes an iptables rule that
   the host firewall does not get to see first, so the `127.0.0.1:` prefix is what holds this line.

## Sizing

**Size on peak, not on idle.** RSS idles low and peaks far higher, reaches that
peak within about a minute of load starting, and returns to idle when load
stops. That is live memory held while ingest, flush and merge overlap — not a
leak, and not something a smaller `merge_max_memory_bytes` reduces, because the
groups being merged are usually far below that budget already. An instance sized
from a quiet screenshot is sized far too small.

**The multiple is unverified.** This paragraph used to quote an idle figure, a
peak figure and the ratio between them. They came from the retired harness
([`LOAD_RESULTS.md`](LOAD_RESULTS.md) §7) — cardinality 1, no reader contending
with the writer, and an achieved rate far below the offered one — so the peak is
the peak of a workload nobody runs. The direction of the advice survives; how
much headroom to leave is not a question this repository can answer today. Size
against the bound in [`CONFIGURATION.md`](CONFIGURATION.md) "Sizing an
instance", which is arithmetic on the configured limits rather than a
measurement, and watch the instance's own peak once it is carrying traffic.

## What to alert on

Loadable form: [`deploy/alerts.yml`](../deploy/alerts.yml), one Prometheus rule
per row below. This table is the decisions and that file is their form, tied by
`query::tests::every_alert_signal_in_the_runbook_has_a_rule` so neither can move
without the other.

| Signal | Condition | Meaning |
|---|---|---|
| `signy_ingest_throttled_total` | Increasing | Returning 429; flush cannot keep up with ingest |
| `signy_ingest_dropped_resources_total` | Increasing | **Data loss, and the only signal it produces.** An export named a tenant this instance will not file it under, so those resources were dropped and the sender was told its body arrived. The `reason` label says where to go. `no_tenant`: an exporter with no `tenant.id` resource attribute — nothing was ever configured. `invalid_tenant`: it is set to something outside `[a-zA-Z0-9_-]{1,64}`, or is not a string. `tenant_not_served`: a well-formed tenant the control plane never pushed a policy for — **push it**. Dropped rather than refused because one export may carry several tenants, and one application's mistake must not refuse another's data |
| `signy_collect_dropped_records_total` | Increasing | **Data loss.** A collected batch carried whole records this instance will never accept — a body it cannot decode, one past a limit — and the collect route dropped them. Deliberate: the collector has one queue per machine, so holding them would stop every other application on that host. A tenant problem is *not* here; it is the counter above. `signy_collect_dropped_bytes_total` is how much |
| `signy_wal_backlog_bytes` | Upward trend | Same cause, earlier signal |
| `signy_data_dir_free_bytes` | Below `SIGNY_MIN_FREE_DISK_BYTES` × 2 | **Alert here, not at the floor.** At the floor ingest is already being refused; this is the warning before it. Divide by `signy_data_dir_total_bytes` for a percentage |
| `signy_flush_errors_total` | Increasing while `flush_success_total` is flat | **Flush stopped.** Most dangerous state |
| `signy_remote_healthy` | Stays 0 | Object store unreachable. Set by three consecutive failures with no success between them, so an isolated failed request does not trip it |
| `signy_remote_consecutive_failures` | Rising but below 3 | The store is degrading without being declared down — the early signal the health flag deliberately hides |
| `signy_merge_debt_parts` | Upward trend | Merge cannot keep up; query-planning cost rises |
| `signy_retention_rewrite_skipped_total` | Increasing | A retention rewrite failed and its group was skipped. **Tenant deletion is not complete.** Not a size problem: the rewrite interleaves reading and writing, so it cannot fail for want of memory at any part size (`merge/selection.rs`, `rewrite_group`) — reaching this counter means I/O or a corrupt input |
| `signy_tenant_policy_unknown_tenants` | Greater than 0 | Policy-less tenant data is present as an operator migration exception; it is not an explicit infinite policy and is not automatically deleted |
| `signy_wal_replayed_entries` | Non-zero after a restart | The previous run did not shut down cleanly. **This is the upper bound on log lines this restart may have duplicated** — delivery is at-least-once, so records the WAL still held may already have been durable |
| `signy_query_memory_exhausted_total` | Increasing | **This instance could not serve a query it was willing to serve** — the shared memory account had no room when the request arrived. The read side's `ingest_throttled_total`: a scaling or budget question, not a plan one. The client is correctly told `429` and will retry, so nothing looks broken from outside; this counter is the only place the event is visible. It does not say *which* of three causes — too many concurrent queries, one query too greedy, or `SIGNY_MEMORY_ACCOUNT_BYTES` too small for this deployment — so it is a signal to go and look. Start by plotting `signy_memory_account_in_use_bytes` against `signy_memory_account_budget_bytes`: riding the ceiling is a budget too small, spiking is one greedy query, and sawtoothing at the ceiling is concurrency. A `400` naming the account is a different event with no counter — it means a single request was larger than the whole budget, which is a client to narrow rather than an instance to scale |
| `signy_memory_account_in_use_bytes` | At `signy_memory_account_budget_bytes` | The account is full *now*. Queries are being refused with `429` and metric compaction is postponing every tick — check `signy_memory_account_deferred_total` beside it, because compaction that never runs is disk growth an hour later. Note this is an accounted total, not the heap: it sits below `signy_process_rss_bytes`, which also carries the caches, the memtables and whatever the allocator has not given back |
| `signy_memory_account_deferred_total` | Increasing without settling | Metric flush and compaction keep finding the account full and leaving their work for the next tick. Unlike a query they have no client to refuse, so the symptom is not an error anybody sees — it is compaction debt that never drains. Either queries are holding the account continuously, or `MERGE_MAX_MEMORY_BYTES` is too large a share of `SIGNY_MEMORY_ACCOUNT_BYTES` for the two to overlap |
| `signy_query_quota_rejected_total` | Increasing | A tenant is over its concurrent-query limit. A plan question, not a scaling one |
| `signy_delete_hidden_rows_total` | Rising steadily long after a request | The rows are hidden but not gone — no rewrite has reached the parts holding them. See below |
| `signy_delete_requests_rejected_total` | Increasing | A tenant is at the per-request limit. Each outstanding request is a predicate every one of that tenant's scans evaluates per row |
| `signy_pending_flush_bytes` | Does not reach 0 while draining | Shutdown has not reached durability |
| `signy_part_sidecar_resident_bytes` | Rising relative to RSS budget | What the bloom cache holds. Since the cache keeps offsets into a mapped `index.bin` rather than decoded filters, this is ~1.3 KiB per part and reaching a budget derived from a real memory limit would take tens of thousands of parts. If it does rise, read it with the pair below rather than alone |
| `signy_part_sidecar_misses_total` vs `_hits_total` | Misses non-zero at all | Every miss re-maps and re-indexes a part's `index.bin`. A steady miss rate means the cache is smaller than the working set, and the shape to check is not the rate but the lifetime: divide `signy_part_sidecar_resident_nanos_total` by `signy_part_sidecar_evictions_total` for how long an entry survives, and `signy_part_sidecar_redecode_gap_nanos_total` by `signy_part_sidecar_redecodes_total` for how soon the part that lost it wants it back. A short life beside a short gap is thrashing whatever the hit rate says — 88% hit rate hid exactly that before the cache stopped holding copies of the file |
| `signy_part_tenant_segments` | Near `part_count × tenant count` | Each tenant is scattered across nearly every part, maximizing shared-part fixed cost |

`part_tenant_segments` is the number of (tenant, part) pairs. Divide by `part_count` for the average tenant
width of one part, or divide by tenant count for **the number of parts containing one tenant**. Each pair
adds one row group, two blooms, and one metadata segment, so the cost of a nearly idle tenant is determined
by this value rather than its own ingest volume.

`signy_collect_skipped_records_total` is deliberately **not** in that table. It
counts records a collecty sent again that this instance already had, skipped
rather than stored twice, and a step in it after a collector restarts or a cut
connection is the mechanism working. What is worth looking at is the opposite:
flat while a collecty restarts means its numbering is not reaching here, and the
duplicates are landing. That is not a threshold, so it is not a rule.

`/ready` is lowered **independently** by flush, merge, retention, OTLP, object storage, and the local cache.
The 503 body identifies the problem.

---

## Response by symptom

### `/ready` stays at 503

```
curl -s localhost:3100/ready          # the body identifies the component
curl -s localhost:3100/metrics | grep _errors_total
```

Continue below based on what is increasing.

### Flush stopped (`flush_errors` increases, `flush_success` is flat)

WAL backlog and MemTable continue growing, and 429 starts once the limit is exceeded. Data is still safe — it is in the WAL.

1. If `remote_healthy` is 0, it is an object-store problem. Continue to the item below.
2. If the log says `fenced by a newer writer`, **another instance claimed the prefix.** This process will
   exit soon. Find why two instances were started first. **Do not discard this disk** because it contains unflushed data.
3. Check whether the disk is full. The WAL is truncated only after a successful flush.

### Object store is unreachable

The engine retries forever and keeps `/ready` at 503. Automatic recovery is expected, so **waiting is the default response**.
Ingest continues until the backlog limit, then returns 429 to clients.

During startup, it retries for `SIGNY_STARTUP_RETRY_BUDGET` (5 minutes by default) and then exits.
If it looks like a crash loop, fix the store rather than increasing that budget.

### Startup is rejected with a catalog protection or "conditional writes" error

The preflight detected **a store that does not enforce conditional writes**. Running it as-is allows catalog
generation collisions, which means data loss.

For an S3-compatible store, set `OBJECT_STORE_CONDITIONAL_PUT=etag` and configure an R2 Bucket Lock rule for
the `catalog/` prefix, then set `SIGNY_OBJECT_STORE_CATALOG_LOCKED=true`. For a single-process development
store, use a `file://` URL — it intentionally gives up conditional writes and catalog protection.

### Disk is full

Ingest is already returning 429 by the time it is actually full: writes stop at
`SIGNY_MIN_FREE_DISK_BYTES` of free space so that flush keeps room to run.
That is a recoverable state — the alternative, past the floor, is a flush that
cannot write, and acknowledged data stuck in a WAL that cannot be drained.

```
du -sh /var/lib/signy/*         # which of wal / parts / traces is large
```

Run that on the host against the bind-mount source rather than inside the
container: the numbers are the same and the shell is one that exists. If the
root filesystem is the one that filled instead, it is the container logs —
`docker ps -s` and `/var/lib/docker/containers` — which means the rotation
options in [`DEPLOYMENT.md`](DEPLOYMENT.md) §4 were dropped somewhere.

- **parts/traces are large** → Reduce `CACHE_MAX_BYTES`. They are cache only and can be restored from S3.
  However, nothing is deleted from S3 for a tenant whose pushed retention keeps data forever — decide the retentions first.
- **WAL is large** → Flush is not progressing. Follow the item above.

### Tenant deletion does not finish

If `retention_rewrite_skipped_total` increases, a part cannot be rewritten within `MERGE_MAX_MEMORY_BYTES`.
It is already invisible to queries, but **the bytes remain.**

Increase `MERGE_MAX_MEMORY_BYTES` or reduce `ROW_GROUP_SIZE` (which makes windows smaller).

### A deletion request stays at `received`

`GET /loki/api/v1/delete` reports `received` until no part could still hold a
row the request covers. The rows are already unreadable — the scan masks them
from the moment the request was accepted — so this is not an availability
problem. It means the bytes are still there.

The removal happens inside the merge rewrite. A part that could hold a covered
row is selected for rewrite on that basis alone, whatever its size, so the
request advances on the next merge tick that reaches it. If it does not:

- Check `signy_merge_success_total` is increasing. Nothing is removed while
  merge is failing.
- Check `signy_retention_rewrite_skipped_total`. A part too large to rewrite
  within `MERGE_MAX_MEMORY_BYTES` blocks deletion for the same reason it blocks
  tenant deletion, and the same fix applies.
- `MERGE_MAX_GROUPS_PER_TICK` bounds how many parts one tick rewrites, so a
  tenant spread across many parts advances a few per tick rather than all at
  once.

The status is deliberately conservative: only a tenant segment's time span can
rule a part out, so any part overlapping the request's window keeps it at
`received`. It never claims a removal that has not happened.

### Two instances started with the same prefix

The old instance logs `fenced by a newer writer` and exits with code 1. This means the defense worked, not
that an incident occurred. However:

- **Do not discard the old instance's disk.** Its WAL contains unflushed data.
- The new instance operates normally.
- To preserve old data, stop the new instance, restart the old instance on its disk, let flushing complete,
  and then replace it through the normal procedure. If the old instance's container is still there,
  `docker start` on it is enough; if it was removed, run the same image against the same bind mount.

---

## Planned hardware replacement

Following the order is lossless. **If the order is violated, fencing kills the old instance**, so the violation
will not go unnoticed.

1. `docker stop signy` sends `SIGTERM`. Draining starts and ingest returns 503. With
   `--stop-timeout=-1` the command does not return until the process exits, which is the point of it.
2. Watch from another shell until `curl /metrics | grep pending_flush_bytes` is **0** and
   `force_flush_complete` is 1. The log warns the operator if this takes a long time. **Waiting is correct.**
3. The process exits on its own. `docker inspect -f '{{.State.ExitCode}}' signy` says 0 when all acked
   data is durable. **Do not discard the disk if it is not 0**, and read it before `docker rm` removes the
   container, because the exit code and the log go together.
4. Start the new instance afterward.

The exit code in step 3 is the only basis for judgment. SIGKILL has no exit code, so if step 2 was skipped
and the process was forced down, restart it on that disk to recover. `docker stop` on a container started
with `--restart unless-stopped` keeps it stopped, including across a host reboot, so nothing comes back up
behind you while the disk is being moved.

### Shutdown takes longer than expected

The drain waits for the merge group that was running when the signal arrived.
Merge stops taking *new* groups the moment it is told to drain, so the bound is
one group, not a whole tick — but one group can be a minute at scale, and
`MERGE_MAX_INPUT_BYTES` is what sets it.

If SIGTERM to exit is much longer than that, read the log timeline rather than
guessing: each stage announces itself (`flush task stopped`, `merge task
stopped`, `servers drained; force-flushing before exit`). Measured on a
two-hour run at 500 tenants, the force-flush itself was 47 ms.

### When the force-flush cannot finish

If the object store is down, step 2 never completes. That is the design: giving up would lose data, so the
retry has no timeout. Two ways out, and they are not equivalent.

- **`docker kill --signal=USR1 signy`** — abandon the force-flush deliberately. The process exits **non-zero**, logs that
  it did so, and leaves the data in the WAL. Use this when the store will not recover soon and the pod has
  to go. Then **keep the disk** and restart on it.
- **Doing nothing until the grace period expires** — the orchestrator sends SIGKILL. The data is in the
  same place, but there is no exit code and no log line saying why, so nothing distinguishes it from a
  clean shutdown afterwards. This is the case `terminationGracePeriodSeconds` is set high to avoid.

Inside a container stdin is not a TTY, so the `exit`/`quit` command on stdin is only useful when a person
is attached to a terminal. `SIGUSR1` is the one that works in a deployment, and `docker kill --signal` is
how it is sent — `docker stop` sends SIGTERM, which the process has already received by then.

## Recovery after forced termination

The WAL remains. Restarting on the **same disk** lets replay recover unflushed data.

Because delivery is at-least-once, **some logs may be duplicated** if termination happens at a flush boundary.
This is an intentional trade-off (`ARCHITECTURE.md`).

How much was duplicated is now observable. Startup logs a WARN naming the record and entry counts it
replayed, and `signy_wal_replayed_entries` holds the entry count for the life of the process. It is an
upper bound, not a measurement: those entries may have been durable already, or may not have been. Nothing
removes the duplicates yet — that is deduplication, still open in `todo.md`.

## Backups

S3 is the source of truth. The local disk is the cache plus the unflushed WAL.

- Configure R2 Bucket Lock for the `catalog/` prefix and keep the two catalog replicas in the same bucket.
  Catalog history is deleted only when `SIGNY_CATALOG_PRUNE_MIN_AGE` is set, and only past that age.
- Catalog generations are append-only and each is stored twice with a digest. Loss of both replicas or the
  entire bucket is outside the recovery guarantee. A missing replica is recreated from the other; an invalid
  one is read past with a warning and stays in place, since the lock forbids rewriting it.
- Backing up the local disk is not meaningful — it contains either cache data or data that is not yet durable.
