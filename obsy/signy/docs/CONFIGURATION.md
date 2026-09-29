# Configuration reference

All settings are environment variables. `Config::from_env` reads them and `Config::validate` checks
them; if validation fails, startup is refused because not starting is safer than running with a bad configuration.

For a working set of them rather than the full reference, see [`DEPLOYMENT.md`](DEPLOYMENT.md).

Tests enforce that this document does not omit any knob from `src/config.rs`
(`every_configuration_knob_is_documented`). Adding a knob to the code without documenting it here
breaks the tests rather than the build.

Duration values use formats such as `500ms`, `30s`, `5m`, `2h`, and `7d`. `off`/`none`/an empty
string mean "disabled" and are valid only for knobs that can be disabled.

---

## Required settings

| Variable | Default | Description |
|---|---|---|
| `SIGNY_DATA_DIR` | `./data` | WAL, checkpoint, and local part cache. **This directory is the only copy of unflushed data.** Do not discard this disk during hardware replacement |
| `SIGNY_OBJECT_STORE_URL` | unset (local-only) | `s3://bucket/prefix` or `file:///path`. **When unset, only the local disk is used without S3 tiering** — unsuitable for production because the disk becomes the source of truth |
| `SIGNY_LISTEN_ADDR` | `127.0.0.1:3100` | The one listener: the collect route, the query API, the admin API and `/metrics`. **Loopback by default**: there is no TLS and no authentication here, so reaching this listener from off the machine has to be a decision rather than the result of not making one. A container that receives traffic sets this to `0.0.0.0:3100`. There is no second address any more — `SIGNY_OTLP_GRPC_ADDR` went with the OTLP gRPC services |

`file://` is for **single-process development and does not provide conditional catalog creation**. Using it
on shared or network storage can lose catalog generations, which means data loss. `from_url` logs a warning
at startup.

Credentials and endpoints passed to `object_store` are supplied through `AWS_*` or `OBJECT_STORE_*`
environment variables (`OBJECT_STORE_*` takes precedence). For S3-compatible stores,
**`OBJECT_STORE_CONDITIONAL_PUT=etag` and `SIGNY_OBJECT_STORE_CATALOG_LOCKED=true` are effectively required**;
the first enables conditional catalog creation and the second asserts that an R2 Bucket Lock rule covers the
`catalog/` prefix. Without either, startup refuses to run.

## Multi-tenancy

| Variable | Default | Description |
|---|---|---|
| `SIGNY_MISSING_TENANT` | unset (reject) | Tenant a **read** without an `X-Tenant-Id` header is filed under. **Unset — the default — rejects such reads with 400**: behind a gateway a missing header is the gateway failing, which should fail loudly rather than quietly pool everyone's data. Set it (any valid tenant id) for single-tenant deployments where nothing mints the header. Writes are unaffected — their tenant is the `tenant.id` resource attribute, and an export naming none is dropped, since a default there would pool every misconfigured exporter into one tenant |
| `SIGNY_MAX_CONCURRENT_QUERIES_PER_TENANT` | 4 | Queries one tenant may run at once, so one tenant cannot take every permit of the shared query semaphore |

**The pushed policies are the tenant registry**: only tenants the control plane
has pushed a policy for are served. A **read** naming any other gets 403. A
**write** naming any other is dropped, silently, and counted in
`signy_ingest_dropped_resources_total{reason="tenant_not_served"}` — the status
an ingest answers says whether the body arrived and nothing about whose it was,
because one export may name several tenants and one application's mistake must
not refuse another's data. The admin API that pushes policies carries no
authentication of its own — signy is not built to be reachable from the outside
network, and assumes every request arrives through a secured channel.

**A write names its tenant in the payload**, at the `tenant.id` resource
attribute of the OTLP export, the same key for logs, traces and metrics on both
transports. It is not configurable: a key both ends must agree on is a key that
can be disagreed about, and the failure is silent. signy strips it before
storage, so it never becomes a queryable label.

**Note:** with `MISSING_TENANT` naming a tenant, headerless **reads** are served
only once a policy has been pushed for that tenant — it is onboarded like any
other.

### Per-tenant limits are not configured here

Because limits vary by plan and can change after launch, **the control plane
pushes them per tenant**, alongside `retention` in the policy body. There are no
per-tenant *rates*: how fast the instance accepts work is the global
backpressure gate's question, answered from the server's own state.

```
PUT /signy/api/v1/admin/tenants/{tenant}/retention
{"retention": "7d", "max_stored_bytes": "10GiB"}
```

`max_stored_bytes` is the other half of a plan that sells a period and a size:
retention decides when bytes leave, and this decides how many may pile up before
they do. It is a size, not a rate, so it is spelled `10GiB` rather than
`10GiB/s`, and pushing a rate spelling is refused rather than guessed at.

It is enforced by **refusing writes, not by deleting**. A tenant over its limit
keeps every byte it has and gets `429` with `Retry-After: 60`; the space comes
back on its own when retention retires the oldest parts, and writes resume
without anyone doing anything. Deleting to make room would mean this engine
choosing which of a customer's logs to destroy, which is not a decision it has
the standing to make.

The comparison is against the tenant's own byte extents in the shared objects —
logs, traces and metric parts — read from `meta.json` rather than from the local
files,
so it does not change as the cache evicts and restores bodies. The Parquet
footer and the sidecars belong to no single tenant and are not charged to one.
`GET …/tenants/{tenant}/usage` reports the same number as `stored_bytes`
alongside `max_stored_bytes`, which is what a control plane shows a customer.

The body is the complete policy, not a patch. If it is pushed without `max_stored_bytes`, the existing
value is **cleared** and reverts to the default above.

### Per-signal retention

`retention` applies to logs, traces and metrics alike unless the body overrides it for a signal:

```
PUT /signy/api/v1/admin/tenants/{tenant}/retention
{"retention": "30d", "log_retention": "14d", "trace_retention": "3d"}
```

`log_retention`, `trace_retention` and `metric_retention` take the same spellings as `retention`.
A signal with no override uses `retention`, so a signal is never kept longer by leaving it out, and
an override left out of a later push is cleared like `max_stored_bytes`. Each signal's queries are
clamped to that signal's floor. `retention: "0"` still deletes the tenant, so a push that pairs it
with an override that keeps data is refused with `400`.

These limits apply to one instance; they are not the monthly usage sold by a plan. Monthly usage spans
multiple instances and outlives any instance, so only the control plane can own that state.

## Ingest input limits

All limits are checked **before** writing to the journal, so rejected requests leave no WAL record.

| Variable | Default | Description |
|---|---|---|
| `SIGNY_MAX_LINE_BYTES` | 256 KiB | Maximum size of one log line |
| `SIGNY_MAX_TIMESTAMP_AGE` | `7d` (`off` allowed) | Reject timestamps older than this |
| `SIGNY_MAX_TIMESTAMP_SKEW` | `1h` (`off` allowed) | Reject timestamps farther in the future than this. **A future part never reaches the retention cutoff** — confusing seconds or milliseconds with nanoseconds is common |

Set both timestamp knobs to `off` only when bulk-loading historical data.

## Memory budget

| Variable | Default | Description |
|---|---|---|
| `SIGNY_MEMORY_BUDGET` | 60% of the detected limit (`off` disables) | Bytes the engine budgets for itself. Unset, the process reads cgroup v2 `memory.max` (falling back to `/proc/meminfo` `MemTotal`) and takes 60% — VictoriaLogs' own contract, and the one measured surviving the sustained 2 GiB workload that OOM-killed both this engine and Loki (`todo.md`, soak section, 2026-08-08). The budget re-seeds the **defaults** of the ceilings below; every explicit knob still overrides its derived value |
| `SIGNY_CAPACITY_PROBE` | `false` | **Dangerous test-only switch.** Set to `1` only in a disposable cgroup capacity experiment. It bypasses metric cardinality admission and shared metric/WAL/in-flight ingest backpressure so the kernel cgroup, rather than an application guard, exposes the raw OOM boundary. Request-size/sample, timestamp, tenant/storage, and disk-safety checks remain active. Startup emits a warning and `/metrics` exposes `signy_capacity_probe 1`; it is empty/disabled by default and is never enabled by the Compose defaults |

When the budget is active the following defaults are computed from it, with the
shares taken from the re-measured attribution (`docs/MEMORY_ATTRIBUTION.md`,
build `b9165b0`) rather than guessed. The startup log prints every derived
value beside the budget and its source.

| Derived knob | Share of budget | Floor |
|---|---|---|
| `MERGE_MAX_MEMORY_BYTES` | 25% | 64 MiB |
| `MERGE_MAX_INPUT_BYTES` | half the merge budget | 32 MiB |
| `MEMORY_ACCOUNT_BYTES` | 50% | 128 MiB |
| `MAX_QUERY_MEMORY_BYTES` | half the account, so 25% | 64 MiB |
| `ROW_GROUP_CACHE_MAX_BYTES` | 12.5% | 16 MiB |
| `MAX_MEMTABLE_BYTES` | 25% of the declared budget (256 MiB when the budget is off) | 32 MiB |
| `SIDECAR_CACHE_MAX_BYTES` | 10% | 32 MiB |


The listed nominal shares sum to 87.5%. The remaining 12.5% of the declared
budget, plus the 40% of the cgroup deliberately left outside
`SIGNY_MEMORY_BUDGET`, covers flush chunks, WAL and request queues, allocator
retention, and process/runtime state. `MAX_MEMTABLE_BYTES` is the shared
log/trace/metric ceiling; metric admission reserves projected sample bytes
against it before the WAL append.

The 25% memtable share is intentional. In the first 2026-08-31 post-admission
run, the old 10% derived ceiling (122.9 MiB at this budget) produced 58
whole-export `429`s and accepted only 270,744 of 560,032 offered datapoints;
that run peaked at 374.23 MiB anonymous memory. The new 25% ceiling is 307.2
MiB (322.1 million bytes). On the same 2 GiB/480 s workload it was validated
as `UNDER_BUDGET`: 560,032/560,032 explosion datapoints, 102,912/102,912
churn datapoints, 2,880/2,880 steady datapoints, no refusals (`200=114`), and
655.527 MiB peak anon (957.1445 MiB cgroup `memory.peak`, including page
cache), with 454.715 MiB at settle and zero OOM kills. Explicit
`SIGNY_MAX_MEMTABLE_BYTES` still overrides this derived value.

The earlier 748.1 MiB / 20-refusal result was measured with a different
allocator and configuration; it is historical context, not a pure before/after
measurement of this share.

**Measured capacity at this budget** (24-hour soak, 2 GiB container, retention
30 m, 2026-08-12): the engine sustains **the full offered 20 k eps — 19,999.8,
nothing throttled** — for 24 hours, 1.73 billion events, with anon flat between
1.47 and 1.57 GiB (peak 1.84), query response p95 428 ms / p99 640 ms, and zero
5xx in 432,001 queries. The residents that a day is run to watch are flat:
sidecar 120 MiB, row-group cache 153 MiB, WAL file 34 MiB, WAL backlog 3 MiB
against its 1 GiB bound.

**The ceiling** (rate ladder at the same configuration, 45 minutes a rung, with
enough ingest connections that the client is never the constraint — 32 at 30 k,
96 at 45 k, sized from the measured service p95; 2026-08-14):

| offered | achieved | refused `429` | `memtable_buffered` peak |
|---|---|---|---|
| 30 k | **29,996.5 eps** | **0%** | 78.0 MiB |
| 45 k | **34,666 eps** | 22.9% | 126.3 MiB |

Two numbers, and an operator wants the first: **30 k eps sustained with nothing
refused**, and **34,666 eps absorbed** while refusing the excess. Everything
refused is a `429`; no rung produced a 5xx or an OOM. What sets the ceiling is
**flush, not the WAL** — the saturated rung pins `memtable_buffered` at
126.3 MiB against the 122.9 MiB `max_memtable_bytes` ("flush is not keeping up"),
while the WAL backlog peaks at 27.7 MiB against its 1 GiB bound and the journal
writer is *less* busy at the higher rate because group commit amortizes
(`records/batch` 1.59 → 4.07). Raising `SIGNY_MAX_MEMTABLE_BYTES` moves the
refusal, not the flush rate.

No rung between 30 k and 45 k has been run, so the knee is bracketed to 15 k and
not narrower. The pair this replaces — **22 k sustained / 24,274 absorbed**, the
same rig two commits earlier — moved because the flush pass spent 63% of itself
building blooms through a `BTreeSet` over a domain a bitmap covers in 2 MiB
(`816b260`): `write_index` is now 10.9 µs an event instead of 25.2. A change to
the flush path invalidates this table, which is why it carries its date.

The predecessor of this measurement read ~18.6 k eps with 6.9% answered `429`
(2026-08-10) — that run carried a retention/merge lock-order stall that froze the
flush thread for up to 52 s at a time, which is where the throttling came from;
fixed in `ca32ee5`. `todo.md`'s soak section holds both verdicts and the
run-by-run history.

## Backpressure

| Variable | Default | Description |
|---|---|---|
| `SIGNY_MAX_MEMTABLE_BYTES` | 256 MiB (`off` allowed; 25% when budget-derived) | Return 429 when the shared log/trace/metric memtables and projected metric append bytes exceed this combined size |
| `SIGNY_MAX_WAL_BACKLOG_BYTES` | 1 GiB (`off` allowed) | Return 429 when unflushed WAL exceeds this size |
| `SIGNY_MAX_INFLIGHT_PUSH_BYTES` | 128 MiB (`off` allowed; budget-derived at 5%) | Return 429 when request bodies admitted and not yet answered exceed this size. The other two bound buffers this server owns; this one bounds what its callers hand it, which was otherwise `concurrency × 16 MiB` with nothing limiting concurrency. Safe at any value: an idle server always admits one body, so a ceiling below one legal push cannot refuse it forever. Measured in flight on the comparison bed: 0.3 MiB — this closes a hole rather than recovering memory. Read it live as `signy_inflight_push_bytes`. Charged per **record** as a collected batch is decoded, not per request: the route holds one record and the handful behind it awaiting an fsync, and a batch that reaches the ceiling waits on its own records rather than refusing itself |
| `SIGNY_MIN_FREE_DISK_BYTES` | 2 GiB | Free space on the data directory's filesystem below which ingest is refused with 429, or `off`. **The last guard, not the first** — eviction bounds the cache and the backlog limit bounds the WAL; this covers what neither does. Past it a flush cannot write, which is the worst state this engine has. Must not be below `FLUSH_MAX_BYTES` |
| `SIGNY_DISK_SAMPLE_INTERVAL` | `10s` | How often free space is re-read. Bounds how stale the number the ingest gate reads can be |
| `SIGNY_BACKPRESSURE_RETRY_AFTER` | `1s` | How long a throttled collecty is told to wait, rendered as `Retry-After`. It is what makes a refusal survivable rather than lossy: the collecty stops the segment there and offers it again. Whole-second granularity, rounded up and never zero — `Retry-After: 0` reads as "retry immediately", the opposite of what an overloaded server is asking for |

**Constraint:** `MAX_MEMTABLE_BYTES` cannot be smaller than `FLUSH_MAX_BYTES`, or writes would be
rejected for data that has not even reached the threshold at which flushing is requested.

Disabling these limits restores the old behavior of growing without bound until OOM. The architecture
assumes clients back off on 429 and rely on their own WAL, so disabling them breaks that assumption.

## Journal and flush

| Variable | Default | Description |
|---|---|---|
| `SIGNY_MAX_BATCH_BYTES` | 1 MiB | Maximum bytes grouped into one write+fsync |
| `SIGNY_MAX_BATCH_MS` | `0` (no wait) | **0 is the default and recommended.** Group commit forms behind writes: data arriving during write/fsync goes into the next batch. A nonzero value caps per-connection throughput at `1000/this value` pushes/s. Increase it only on disks where fsync costs more than waiting |
| `SIGNY_FLUSH_MAX_BYTES` | 1 MiB | Flush when the memtable reaches this size |
| `SIGNY_FLUSH_MAX_INTERVAL` | `5s` | Flush at this interval even when the size threshold is not reached. **This value is the RPO for unexpected disk loss, and it is also the object-store bill** — see below |
| `SIGNY_FLUSH_CHECK_INTERVAL` | `500ms` | Interval at which the flush loop checks conditions |
| `SIGNY_FLUSH_CHUNK_BYTES` | 32 MiB (minimum 1 MiB) | Most a flush materializes at once. The snapshot is written in chunks of this many bytes, so the flush transient is bounded by the chunk rather than by how large the memtable grew while the previous flush ran |
| `SIGNY_WAL_COMPACT_MIN_BYTES` | 64 MiB, `off` disables | Local-only mode truncates the WAL's dead prefix (the bytes before the checkpoint, which no recovery path reads) once it exceeds both this floor and the live suffix. `off` restores the old behaviour where `journal.wal` keeps everything ever ingested. With an object store configured the WAL always compacts and this knob is ignored |
| `SIGNY_ROW_GROUP_SIZE` | 8192 (maximum 65536) | Parquet row group row count. Groups also stop at tenant boundaries, so **the number of tenants in a part is a lower bound for the actual row group count** |

## Merge

| Variable | Default | Description |
|---|---|---|
| `SIGNY_MERGE_INTERVAL` | `30s` | |
| `SIGNY_MERGE_MIN_PART_COUNT` | 4 (minimum 2) | Do not perform a normal merge below this count |
| `SIGNY_MERGE_TARGET_PART_ROWS` | 1,000,000 | Target output row count (soft) |
| `SIGNY_MERGE_MAX_PART_ROWS` | 4,000,000 | Maximum output row count (hard) |
| `SIGNY_MERGE_MAX_INPUT_BYTES` | 512 MiB (budget-derived) | Input limit for one group. **Uncompressed (materialized) bytes** |
| `SIGNY_MERGE_MAX_MEMORY_BYTES` | 1 GiB (budget-derived) | Hard limit that one read can materialize |
| `SIGNY_MERGE_MAX_GROUPS_PER_TICK` | 16 | |

**Constraint:** `MERGE_MAX_INPUT_BYTES <= MERGE_MAX_MEMORY_BYTES`. Both values are compared with
`materialized_bytes` recorded in part metadata (the memory actually occupied when read), so their units
match. If a limit is exceeded, groups are split in half and a single part is rewritten in row-group
windows, so the operation cannot fail permanently.

## Retention

Retention is **per tenant only**: each tenant keeps data for the period its
pushed policy names, a tenant pushed `infinite` or nothing keeps data forever,
and there is no global period.

| Variable | Default | Description |
|---|---|---|
| `SIGNY_RETENTION_INTERVAL` | `5m` | |
| `SIGNY_RETENTION_BATCH_SIZE` | 100 | Number of parts processed per tick |
| `SIGNY_RETENTION_GRACE_PERIOD` | `1h` | Grace period before deleting orphan objects |
| `SIGNY_MAX_RETENTION_RUNTIME` | `2m` | Object-store operation timeout for retention and catalog pruning |
| `SIGNY_ORPHAN_GC_INTERVAL` | `1h` | How often part objects that no catalog generation names are collected, whether or not retention retired anything. Merges and compactions leave their inputs behind as orphans too |
| `SIGNY_ORPHAN_GC_MAX_RUNTIME` | `2m` | What one collection pass may spend. The pass saves its scan cursor and its sightings, and the next one carries on from there, so this bounds a pass rather than the collection |
| `SIGNY_ORPHAN_GC_MAX_SCANNED_OBJECTS` | 200000 | Objects one pass lists before it stops and saves its place |
| `SIGNY_ORPHAN_GC_MAX_DELETED_OBJECTS` | 20000 | Objects one pass deletes. `signy_orphan_candidate_objects` reports what is still waiting |
| `SIGNY_ORPHAN_GC_MAX_DELETED_BYTES` | `4GiB` | Bytes one pass deletes |
| `SIGNY_ORPHAN_GC_DRY_RUN` | `false` | Report what collection would delete, in `signy_orphan_candidate_objects` and `signy_orphan_candidate_bytes`, and delete nothing. First sightings are still recorded, so a real pass afterwards deletes what the dry run reported |
| `SIGNY_CATALOG_PRUNE_MIN_AGE` | `off` | Delete catalog commits and snapshots older than this that no startup reads any more. Two snapshots that verify on both replicas, and every commit from the older of them on, are always kept. **Set it above the Bucket Lock age on `catalog/`**, which refuses younger deletes; with a seven-day lock, `8d` |
| `SIGNY_RETENTION_REWRITE_THRESHOLD` | 0.5 | Rewrite when the expired-row fraction of a part exceeds this value. Tenant deletion (`retention: "0"`) ignores this value |

## Cache

| Variable | Default | Description |
|---|---|---|
| `SIGNY_CACHE_MAX_BYTES` | 10 GiB | Local part cache limit. Exceeding it triggers LRU eviction |
| `SIGNY_CACHE_EVICTION_INTERVAL` | `30s` | |

Small catalog files such as `meta.json` are not evicted; the data body and the blooms are.

## Query resource limits

| Variable | Default | Description |
|---|---|---|
| `SIGNY_MAX_QUERY_RANGE` | unset | Maximum requested time range |
| `SIGNY_MAX_QUERY_SCAN_ROWS` | 5,000,000 | |
| `SIGNY_MAX_QUERY_SCAN_BYTES` | 2 GiB | |
| `SIGNY_MAX_QUERY_MEMORY_BYTES` | 512 MiB (budget-derived) | One query's own materialization cap, enforced inside the scan. It is the backstop under the estimate below, not the admission decision |
| `SIGNY_MEMORY_ACCOUNT_BYTES` | 1.5 GiB (budget-derived) | What every query, metric flush and metric compaction may hold **at once**. A request prices what it will materialize before it scans anything — a log query from `limit × the average row its parts recorded`, a metric query from `series × steps` after selection, a trace query from its span ceiling — and is admitted or refused on arrival. Two refusals: it does not fit *right now* is a `429` naming the account and counted in `signy_query_memory_exhausted_total`; it does not fit *at all* is a `400` telling the client to narrow, because a retry can never help. Flush and compaction charge the same account and, having no client to refuse, postpone to the next tick (`signy_memory_account_deferred_total`). `signy_memory_account_in_use_bytes` is the live figure. This replaced two ceilings that never met — a query pool and a background pool — so the process had two limits and no total |
| `SIGNY_ROW_GROUP_CACHE_MAX_BYTES` | 256 MiB, `off` disables (budget-derived) | Decoded row groups kept across scans. A part is immutable, so a group decoded whole by one scan serves every later scan without paying the reader build again; the budget bounds what stays resident (`signy_row_group_cache_bytes` reports it). Entries die with their part on merge or retirement |
| `SIGNY_SIDECAR_CACHE_MAX_BYTES` | unbounded (`off`; budget-derived) | Byte cap on what a part's bloom sidecar holds resident, evicted LRU across parts. **It now bounds an index rather than the blooms themselves**: `index.bin` is mapped and the cache keeps the offsets of each stored filter, so an entry is about 1.3 KiB per part instead of the four megabytes the decoded copy was. A 122 MiB budget held roughly thirty parts before and holds tens of thousands now — three 900 s soaks at 222–264 parts each read a 100% hit rate against 0.6 MiB resident. The knob is kept because the index is still linear in parts × row groups × windows, but it is no longer the term that decides whether a soak survives; it was, and `todo.md`'s bloom-cache section has the measurement |
| `SIGNY_MAX_LOG_LIMIT` | 100,000 | Maximum `limit` parameter |
| `SIGNY_MAX_QUERY_RUNTIME` | `30s` | Also the timeout for metadata endpoints |
| `SIGNY_MAX_CONCURRENT_QUERY_SCANS` | 8 | Shared with the attribute (autocomplete) endpoints |
| `SIGNY_MAX_HISTOGRAM_BUCKETS` | 10,000 | Most buckets one `/logs/histogram` answer may hold; over it the request is refused with the count and the fix |
| `SIGNY_MAX_CONCURRENT_TAILS` | 8 | Live tail (`/signy/api/v1/logs/tail`) streams held at once. Over the limit the request is refused with 429 rather than accepted and dropped |
| `SIGNY_TAIL_POLL_INTERVAL` | `1s` | How often a live tail asks for new lines. This is both its latency floor and its cost per connection |
| `SIGNY_MAX_RESTORE_RUNTIME` | `25s` | Cache-miss restore timeout |
| `SIGNY_MAX_TRACE_SPANS` | 100,000 | Most spans one trace scan may materialize; over it the request is refused with 413 |
| `SIGNY_MAX_TRACE_SEARCH_LIMIT` | 1,000 | Maximum `limit` on trace search |
| `SIGNY_MAX_CONCURRENT_TRACE_SCANS` | 8 | The trace surface's own scan slots — a trace scan decodes whole-span payloads, so it does not compete for the log scanner's |
| `SIGNY_MAX_TRACE_QUERY_RUNTIME` | `30s` | |
| `SIGNY_MAX_TRACE_RESTORE_RUNTIME` | `25s` | Cache-miss restore timeout for trace parts |
| `SIGNY_MAX_ACTIVE_SERIES` | `off` | Optional process-wide emergency limit on live metric series. The normal guard is the shared, byte-based `SIGNY_MAX_MEMTABLE_BYTES`; when this explicit count guard fires, the complete metric export is refused with `429` and `Retry-After` (there is no partial-success filtering) |
| `SIGNY_METRIC_SERIES_IDLE_TIMEOUT` | `600s` | How long a metric series may go without a sample before its index state is evicted (once flushed) and its capacity returns. History stays in parts; a returning series is re-created, and the one artifact — a delta counter restarting — is a counter reset `rate` absorbs |
| `SIGNY_MAX_METRIC_SERIES_PER_QUERY` | 10,000 | Most series one metric query may select; over it the request is refused with 413 before any chunk is decoded |
| `SIGNY_MAX_METRIC_POINTS_PER_QUERY` | 2,000,000 | Most `series × steps` output points one metric query may ask for — the bound the memory reservation is sized from |
| `SIGNY_MAX_CONCURRENT_METRIC_SCANS` | 8 | The metric surface's own scan slots — Gorilla decode plus per-step folds is a third cost profile, so it does not compete for the log or trace scanners' |
| `SIGNY_MAX_METRIC_QUERY_RUNTIME` | `30s` | |
| `SIGNY_MAX_METRIC_RESTORE_RUNTIME` | `25s` | Cache-miss restore timeout for metric parts |

The LogQL metric evaluator left with the read-path decision (issue #3), and
its knobs left with it: `SIGNY_MAX_METRIC_EVALUATION_POINTS` (renamed
`SIGNY_MAX_HISTOGRAM_BUCKETS` — the histogram grid is its one remaining
meaning), `MAX_METRIC_SERIES`, `MAX_METRIC_SAMPLES`, `MAX_SERIES_MATCHERS`,
and `MAX_CONCURRENT_METRIC_EVALUATIONS`. An instance that still sets one
starts normally and ignores it. The trace knobs above left with the Tempo
surface at the same decision and returned with the first-party trace API
(issue #7).

## Startup and shutdown

| Variable | Default | Description |
|---|---|---|
| `SIGNY_STARTUP_RETRY_BUDGET` | `5m` | Retry object-store startup steps for this duration. Absorb transient failures, then exit and let the orchestrator apply restart backoff |
| `SIGNY_SHUTDOWN_FLUSH_WARN_AFTER` | `30s` | Warn the operator on stdout when force-flush has failed for this long |

## Load harness only (do not use in production)

These settings inject in-process latency and errors for `scripts/run_load_local.sh`. Setting any of them
activates the wrapper, so **never set them in production.**

| Variable | Description |
|---|---|
| `SIGNY_OBJECT_STORE_LATENCY_MS` | Base write latency |
| `SIGNY_OBJECT_STORE_READ_LATENCY_MS` | Base read latency (write value when unset) |
| `SIGNY_OBJECT_STORE_LATENCY_JITTER_MS` | Added `uniform(0, jitter)` |
| `SIGNY_OBJECT_STORE_ERROR_RATE` | 0.0–1.0. Injected **only into writes** |
| `SIGNY_OBJECT_STORE_FAULT_SEED` | Reproduction seed |

## Clocks

There is nothing to configure in production. It is still useful to understand how time-dependent behavior is tested.

- **Monotonic clock** (flush interval, force-flush backoff, startup retry budget) uses `tokio::time::Instant`.
  `tokio::time::pause()` virtualizes it, so a five-minute budget can be tested in ten milliseconds.
- **Wall clock** (timestamp acceptance window, default query range, retention cutoff) is read through `Clock`.
  Tests can freeze and advance the clock to target boundaries precisely instead of changing data into the past.

## Logging

The process follows `RUST_LOG` directly. When unset, it uses `signy=info,warn`.

## Allocator

| Variable | Default | Description |
|---|---|---|
The production binary's global allocator is **mimalloc** (`src/main.rs`), which took the role from
jemalloc on 2026-08-31. glibc was measured out first: the soak read its retained-free creep killing
2 GiB in hours with every gauged resident flat and every glibc knob below already applied. The three
`MALLOC_*` knobs and the trim loop therefore act only in `--features memprof` builds, whose
instrumented allocator still goes through glibc.

mimalloc runs at its defaults, and unlike jemalloc's that is not yet a measured claim — jemalloc's
defaults survived a five-way A/B on the soak rig, mimalloc's have had no equivalent. An operator A/B
goes through the `MIMALLOC_*` environment variables; `MIMALLOC_PURGE_DELAY` is the one that decides
how quickly freed pages are handed back to the kernel.

| `SIGNY_MALLOC_TUNING` | on | On glibc the process caps malloc arenas and fixes a 128 KiB trim threshold before any thread exists — `docs/MEMORY_ATTRIBUTION.md` measured 44–69% of the cgroup's anonymous memory as freed-but-retained heap without it. `off` restores glibc's defaults for an A/B or if a throughput regression is suspected |
| `SIGNY_MALLOC_ARENA_MAX` | 4 | The arena cap the tuning applies. 1 was measured first and rejected: anon fell 3.6x but the allocation-heavy flush path halved its cadence contending for the single arena. 0 leaves glibc's own arena scaling in place (trim threshold still applied) |
| `SIGNY_MALLOC_TRIM_INTERVAL` | `60s` (`off` disables) | How often a background loop calls glibc's `malloc_trim(0)`, returning free pages from the middle of every arena to the kernel. The fixed trim threshold only releases heap tops; without this the second 24-hour soak measured an ~130 MiB/hour anonymous creep with every gauged resident flat, reaching a 2 GiB kill at t≈8653 s. No-op on non-glibc builds |

## Logging

| Variable | Default | Description |
|---|---|---|
| `SIGNY_LOG_FORMAT` | `text` | `text` or `json`. **The container image sets `json`** — a deployment ships these lines to a collector, and the default is the form worth having in front of a terminal |

Read before anything else, because it decides how everything after it is
written. A rejected configuration therefore reaches stderr through the panic
rather than through a subscriber, which is the same place it went before.

---

## Tuning starting points

- **Want a smaller RPO** → Lower `FLUSH_MAX_INTERVAL`. Object-store writes increase accordingly, and on a
  per-request backend that is money — see the table below before choosing
- **High ack latency** → First check whether `MAX_BATCH_MS` is 0. If not, that value is the latency floor
- **WAL backlog is growing** → Flush cannot keep up with ingest. Check for 429 responses
  (`signy_ingest_throttled_total`); if none appear, the limit is too high
- **`/ready` stays at 503** → Check which `/metrics` `*_errors_total` is increasing.
  Flush, merge, retention, object store, and local cache each lower readiness independently
- **The disk is filling** → Reduce `CACHE_MAX_BYTES` or shorten the pushed tenant retentions. Nothing is deleted for a tenant whose policy retains forever
- **Want p95/p99** → Apply `histogram_quantile` to `signy_query_latency_ms_bucket`,
  which carries an `endpoint` label (`query_range`, `query`, `tail`, `patterns`,
  `detected_fields`, `volume`). Per endpoint is the useful cut — a dashboard slow
  because `volume` is slow does not look like a slow `query_range` — and the whole
  read path is `sum by (le)` across them. There is no unlabeled series to read
  instead: one would double-count under `sum`.
  `*_latency_ns_total` provides only an average
- **Queries are slow but nothing is failing** → Check `signy_query_scans_queued_total`. Nonzero
  means scans waited for a slot, and `signy_query_scan_queue_wait_ns_total` divided by it is how
  long they waited. The scheduler admits by arrival order and knows nothing about cost, so a cheap
  dashboard query queues behind expensive ones: measured at 24 concurrent readers, a 60-second query
  reached p95 6.46 s while the wide scans ahead of it measured 392 ms
  ([`LOAD_RESULTS.md`](LOAD_RESULTS.md) §11). `signy_query_scans_in_flight_peak` is the high-water
  mark since start — a sampled gauge cannot see a burst that fills and drains between two scrapes


## The flush interval is the object-store bill

A flush costs **five PUTs**: three for the part's immutable files
(`data.parquet`, `index.bin`, `meta.json`) and two for the catalog generation's
`a` and `b` replicas. Once the catalog is loaded a flush reads nothing back.
**Pinned by a test rather than by a load run**
— `object_storage::tests::publishing_a_part_costs_a_fixed_number_of_requests` —
which also holds the two properties that matter: publishing the tenth part into
a nine-part catalog costs what publishing the first into an empty one cost, and
the file count per part is asserted rather than incidental. The load run that
first counted these requests is retired ([`LOAD_RESULTS.md`](LOAD_RESULTS.md) §9)
and the count outlived it, because a request count is a property of this code
rather than of a machine or a corpus.

A flush skips an empty memtable, so an idle instance costs nothing. An instance
with continuous traffic flushes on every tick, which makes PUT volume a function
of this one setting. The table below is arithmetic on the pinned count and
nothing else:

| `FLUSH_MAX_INTERVAL` | Class A / day | / month | R2 cost |
|---|---|---|---|
| 2 s | 216,000 | 6.48 M | $24.66 |
| 5 s (**default**) | 86,400 | 2.59 M | $7.16 |
| 15 s | 28,800 | 0.86 M | free tier |
| 30 s | 14,400 | 0.43 M | free tier |
| 60 s | 7,200 | 0.22 M | free tier |

**The default does not fit the budget this engine was designed around.** The
shared-part layout exists because per-tenant objects broke a $1/month plan, and
one busy instance at the default spends about seven times that before any
tenant multiplier. Consolidating the two index sidecars into one file took a
fifth off this table; the remaining term is the flush rate itself.

The default is 5 s anyway, because this setting is also the RPO — how much
acknowledged data a disk loss can cost — and that is a deployment decision
between money and durability, not one this repository should make on someone's
behalf. What it can do is refuse to let the choice be made without the price.

**What is not in the table, and is unverified:** merge and retention PUTs ride
on top of it, and their rate is a function of the workload. Every figure this
repository has for that rate came from the retired harness, so treat the table
as the floor for a busy instance rather than as its bill.

Watch `signy_object_store_operations_total{kind="put"}` to see the real
number for a real workload; the rates above will change and the counts will not.


## Sizing an instance

The query term used to be `MAX_CONCURRENT_QUERY_SCANS × MAX_QUERY_MEMORY_BYTES`
— 8 × 512 MiB, four gigabytes no single knob mentioned and nothing enforced.
Then it was a query pool plus a merge ceiling: two numbers added together
because both could be full at once. It is one account now, so the table is one
row and the peak is the knob:

| term | default |
|---|---|
| `MEMORY_ACCOUNT_BYTES` (every query, flush and compaction together) | 1.5 GiB |
| **Peak materialized** | **1.5 GiB** |

`MAX_QUERY_MEMORY_BYTES` (512 MiB) and `MERGE_MAX_MEMORY_BYTES` (1 GiB) still
cap one operation each, and startup refuses a configuration where either — or
twice `FLUSH_CHUNK_BYTES` — exceeds the account: an operation that cannot fit
the shared account is refused or postponed on every attempt it ever makes.

The process logs this number once at startup (`peak_materialized_bytes`), because
there is nowhere else to learn it. The memtable and the flush chunk sit outside
it — the startup log names what is excluded.

An instance sized from its idle footprint is still sized **far** too small: peak
RSS is reached within about a minute of load starting and returns to idle when
load stops, so a quiet screenshot describes nothing.

**This bound is the only sizing figure here that is not retired**, because it is
arithmetic on the configured limits rather than a measurement. The multiple
between idle and peak used to be quoted in this paragraph; it came from the
retired harness ([`LOAD_RESULTS.md`](LOAD_RESULTS.md) §7) and is struck. Size
against the bound, and measure the peak on a real workload.

**How far a real workload gets into the query term is now measured: 9.2%.** With
all eight scan slots occupied and 2,403 scans queued behind them, peak RSS was
496 MB ([§11](LOAD_RESULTS.md)). The per-scan cap is nowhere near binding at that
shape of query, so the bound above stays an upper bound rather than becoming a
sizing target — 850 MB with merge running is still the larger figure, and the
read path is not where this engine's memory goes.

Trace reads are **in** the number: a trace scan charges
`SIGNY_MEMORY_ACCOUNT_BYTES` like every other query. Its price is the one
estimate here with an unmeasured term — a trace part records stored bytes per
tenant but nothing that says what decoding them costs, so the stored average is
scaled by a fixed expansion factor and the scan is admitted against its full
`SIGNY_MAX_TRACE_SPANS` ceiling, then settled back to what it actually held the
moment it returns. It stays per-query byte-capped by
`SIGNY_MAX_QUERY_MEMORY_BYTES`, which is what catches an expansion factor that
turns out to be optimistic. The open note that once stood here (trace
scans carried only a span count and no byte budget) closed with the
first-party trace API (issue #7).

Not in the number, and why:

- **The memtable.** Bounded by backpressure rather than by a constant: ingest is
  refused before it grows without limit. Note that `MAX_MEMTABLE_BYTES` is
  enforced against an accounting that undercounts what a line actually occupies,
  so the real ceiling is above the setting — see [`VISION.md`](VISION.md) I.
- **Resident part sidecars.** Linear in (tenant, part) pairs, and small relative
  to the query term at every part count this engine has been run at. The
  absolute figures are retired; the shape is not, and
  `signy_part_sidecar_resident_bytes` reports it live.
- **Peak RSS above the sum of these.** It is live memory held while ingest,
  flush and merge overlap. Two explanations were proposed for it and both were
  refuted: it is not the merge memory budget, which barely moved it, and it is
  not allocator high-water retention, because RSS returns to its starting value
  after load stops ([`LOAD_RESULTS.md`](LOAD_RESULTS.md) §6 and §7 keep both
  refutations).
- **`CACHE_MAX_BYTES`.** Disk, not memory.

**To lower the peak, lower the concurrency first.** `MAX_CONCURRENT_QUERY_SCANS`
is the multiplier; halving it takes a gigabyte and a half off the bound, and it
degrades a burst of queries into a queue rather than degrading every query.
