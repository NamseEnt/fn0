# Production readiness review (2026-07-25)

Reviewed revision: `56afbbe` (M7 completion, working tree clean)
Review scope: entire `src/` (22,436 LOC), `docs/`, and deployment assets
Test status: all 211 `cargo test` tests passed

## Verdict

**Not production-ready in its current state.** Feature completeness (LogQL subset, Loki/Tempo API, S3
tiering, graceful shutdown, retention, and merge) is high, and crash-recovery invariants are carefully
designed at the code and comment level. However, all three of the following hold simultaneously.

1. On object-store backends, **the flush loop permanently stops on the second compaction** and cannot recover by restarting.
2. **There is no ingest backpressure.** Even when flush stops, the server keeps returning `204` while RAM and disk grow without bound.
3. **Multi-tenancy is not implemented.** `X-Scope-OrgID` is not parsed, so tenant data is mixed and there is no target for per-tenant throttles or quotas.

(1) and (2) together guarantee "failure → automatic OOM/full disk." These three items are the production gate.

TLS was later **confirmed out of scope** (`docs/ARCHITECTURE.md`, "Transport security — TLS unsupported").
The reverse proxy handles end-to-end encryption and authentication/authorization; in return, the listening
address must remain inside the trust boundary as a deployment requirement.

The complete list by severity follows. The `Verification` field distinguishes items reproduced or executed
in this review from those inferred by reading the code.

## Change history

After the review, a batch of **localized input-validation, configuration, and observability fixes** was
applied. Each item is marked `fixed` / `partially fixed` in its body, with remaining work recorded.

| Item | Status |
|---|---|
| P1-6 timestamp acceptance window | Fixed |
| P1-7 label/line size limits | Partially fixed (stream-count limit awaits multi-tenancy) |
| P2-3 snappy reported-length validation + body limit knob | Fixed |
| P2-4 retention-specific timeout knob | Partially fixed |
| P2-6 honor `RUST_LOG` | Fixed |
| P2-9 `file://` warning | Partially fixed (opt-in enforcement not applied) |
| Document TLS unsupported / multi-tenancy requirements | Complete (`ARCHITECTURE.md`) |

**Intentionally excluded from that batch:** P0-1 (WAL compaction) and P0-2 (backpressure) change durability
and the hot path, so dedicated crash-injection tests and O(1) size tracking (P1-5) must come first.
They remain separate work because mixing them with localized fixes would make regression causes harder to trace.

---

## P0 — production gates (do not deploy without these)

### P0-1. WAL compaction permanently wedges on the second call (not recoverable by restart)

- Location: `src/journal/compaction.rs:1-30`, `src/journal/replay.rs:27`
- Verification: **Reproduced** (temporary test was written and reverted)

`docs/M7_LOAD_RESULTS.md` already recorded this blocker, but the review additionally confirmed **two
subcases and that a restart cannot recover it.**

After successful compaction, `journal.wal.compact.state` (phase=2) remains instead of being deleted.
Compaction truncates the WAL and resets checkpoint to 0, so later offsets use a **new coordinate system**.
The next compaction compares against an offset from the old system and takes one of three paths.

| Next offset | Code path | Result |
|---|---|---|
| `< state.offset` | `compaction.rs:22-27` | `Err("WAL compaction checkpoint moved backwards")` → **permanent wedge** |
| `== state.offset` | `compaction.rs:12-20` | Quietly returns no-op. **WAL is not truncated** (case missing from the M7 document) |
| `> state.offset` | fall-through | Happens to work |

Reproduction result (first record 32 B, second 31 B):

```
first_offset=32 wal_after_first=0 second_offset=31
result=Err(Custom { kind: InvalidInput, error: "WAL compaction checkpoint moved backwards" })
wal_after_second=31
```

If the second batch is even slightly smaller than the first, it wedges immediately. This is nearly certain in real traffic.

**Additional fact missing from the M7 document — restart does not recover it.** `recover_unfinished_compaction`
returns immediately at `replay.rs:27` when `state.phase != 1`, so the phase-2 state file remains after restart
and the same wedge recurs. The only recovery is for an operator to manually delete `journal.wal.compact.state`.
This was undocumented.

After the wedge, `flush.rs:52-91` retries the same doomed offset every tick → always fails → `continue`
blocks entry into a new flush. `part_count` freezes, WAL and MemTable grow without bound, `/ready` is 503,
but **ingest continues returning 204**.

**Remediation**
- Remove the state file immediately after successful compaction and make that removal durable (fsync the parent directory).
- On restart, treat surviving phase-2 state as "already complete" and remove it (fix the early return in `replay.rs`).
- Replace absolute-offset comparisons with a coordinate-system generation. All three current branches assume invalidly that offsets share a coordinate system.
- Test gate: there is currently no **two or more consecutive successful compactions** case
  (all four compaction tests in `src/journal/tests.rs` cover one compaction). Add shrinking, equal, and growing size cases plus crash injection at each crash point.

### P0-2. No ingest backpressure — unbounded growth and OOM during failure

- Location: `src/ingest.rs:17` (only the draining check), `src/memtable.rs:92`, `src/config.rs` (no related knob)
- Verification: Code inspection + behavior observed while reproducing P0-1

`push` checks only `is_draining()`. None of the following blocks ingest.

- No MemTable byte limit
- No WAL backlog limit (`signy_wal_backlog_bytes` is exposed but unused for gating)
- No block based on consecutive flush failures
- No `429 Too Many Requests` path anywhere in the code

Therefore, when flush stops due to an S3 failure or P0-1, the server keeps acking with `204` until it
exhausts RAM and disk. `/ready` changes to 503, but Alloy does not check `/ready`, so this has no effect.
The M7 run exceeded the limit through this path with `wal_backlog_bytes=19.9 MB`.

**Remediation**
- Add `SIGNY_MAX_MEMTABLE_BYTES` and `SIGNY_MAX_WAL_BACKLOG_BYTES`. Above the limit, return `429`
  **before** journal append to induce retry. Alloy backs off on 429, so the client WAL acts as a safety net.
- Use two soft/hard thresholds: 429 with `Retry-After` at soft and 503 at hard.
- The `ARCHITECTURE.md` assumption of "Alloy WAL as a safety net" holds only when the server sends a rejection signal.
  It is currently broken because the server acknowledges instead.

### P0-3. Multi-tenancy not implemented — no basis for per-tenant throttles and quotas

- Location: entire `src/router.rs` (no middleware), `src/ingest.rs`, `src/trace_ingest.rs`
- Verification: `rg` confirmed no tenant-related code

**TLS was confirmed out of scope** (`docs/ARCHITECTURE.md`, "Transport security — TLS unsupported").
The reverse proxy handles end-to-end encryption and authentication/authorization. Therefore, this item's gate is **multi-tenancy**.

`X-Scope-OrgID` is **not parsed**. Even when Loki/Tempo data sources and Alloy send tenant headers, everything is mixed in one namespace. Results:

- Tenant A's logs appear directly in tenant B's queries (silent data leakage).
- **There is no target for per-tenant throttles and quotas.** One tenant's surge can kill the whole service.
  Combined with P0-2 (no backpressure), this means "one tenant can OOM the server."
- Per-tenant retention, capacity accounting, and deletion requests are all impossible.

**Remediation** (the "Multi-tenancy" section of the architecture document is the target design)
- Extract `X-Scope-OrgID` on both ingest and query. OTLP uses the same gRPC metadata key. Expose the missing-header policy (accept as default tenant vs reject) in configuration.
- Make tenants a **storage-path partitioning axis** (manifest/part paths). Treating them as stream labels would make per-tenant accounting and deletion require full scans.
- Quota targets: ingest rate, active stream count, storage capacity, concurrent queries, and query scan budget. Return ingest `429` above the limit (Alloy backs off and relies on its own WAL).
- Expose every quota/rejection counter on `/metrics` with a tenant label (together with P2-7's missing labels).

**Note — proxy trust assumption**: The engine trusts `X-Scope-OrgID` without verification. Tenant isolation
fails if the engine is directly reachable from a network location where the header can be forged. Keeping
the listening address inside the trust boundary is a deployment requirement, and the default `0.0.0.0` bind
(`src/config.rs`) conflicts with that requirement.

---

## P1 — problems soon encountered in real use

### P1-1. Tempo search restores every trace part from S3 and scans all of them

- Location: `src/tempo/handlers.rs:96, 159, 185` → `src/tempo/scan.rs:114` `pin_all_trace_parts`
- Verification: Code inspection (`state.trace_parts.part_ids()` = complete ID set)

`trace_by_id` correctly uses bloom pruning (`candidate_part_ids`). However, `search`, `search_tags`, and
`search_tag_values` all call `pin_all_trace_parts`, **restoring every trace-part body locally regardless of
time range** and then scanning all of them. `search` applies its start/end filter at `handlers.rs:130` after
the scan. `search_tags` / `search_tag_values` have no time-range parameters at all.

Grafana's Tempo data source calls `search/tags` whenever a tag dropdown opens. Thus **opening the UI once
downloads the entire trace dataset from S3.** Once the cache limit (`cache_max_bytes`) is exceeded, download
→ eviction → redownload repeats, causing S3 request costs and bandwidth to explode.

**Remediation**: Apply time pruning during pin using `min_ts_ns`/`max_ts_ns` from trace-part metadata and
support time-range parameters for tags/tag_values (the Tempo API has `start`/`end`). The standard approach
is to pre-aggregate the tag catalog in a part sidecar and answer without restoring bodies.

### P1-2. Documents claim OTLP log ingest support, but it is not implemented

- Location: `src/startup.rs:342` — `add_service(otlp_service.into_server())` registers only one `TraceServiceServer`
- Verification: Code inspection (no `LogsService` implementation or registration)

`docs/ARCHITECTURE.md` says "Ingest protocols: Loki push (protobuf+snappy) + OTLP (gRPC)" and the data
model says logs and spans are unified as wide events. In reality, only the trace gRPC service is registered,
and Alloy sending **logs** through `otelcol.exporter.otlp` receives `UNIMPLEMENTED`. OTLP/HTTP (`/v1/traces`,
`/v1/logs`) is also absent — `otlphttp` is a common Alloy configuration.

**Remediation**: Implement it (recommended: `LogsService` + OTLP/HTTP routes), or correct the document to
say "logs are Loki-push only." A documentation/implementation mismatch itself damages adoption confidence.

### P1-3. Group commit always consumes the batch timer — ~5 push/s per connection

- Location: `src/journal/writer.rs:230-231`
- Verification: Code inspection, consistent with measurements in `docs/M7_LOAD_RESULTS.md`

While `batch_bytes < max_batch_bytes`, the batch loop waits with `timeout_at(deadline, rx.recv())`.
Even when no follow-up request exists, it waits the full **`max_batch_ms` (200 ms by default).** Thus every
push has effectively fixed 200 ms ack latency and one connection is capped at ~5 push/s.

The M7 document classified this as a "client artifact," but consuming the timer while the server knows the
channel is empty is a server-side design problem. Correct group commit **starts writing immediately, with
requests arriving during fsync forming the next batch.** The current implementation waits for arrivals before writing.

Impact: In environments with few Alloy instances (small deployments, single-node k8s), the throughput cap is
very low. Lowering `max_batch_ms` trades the problem for more fsyncs and a disk bottleneck.

**Remediation**: Start write+fsync as soon as the first record arrives and collect arrivals during fsync into
the next batch. `max_batch_ms` should act only as a ceiling when fsync completes immediately, not as a mandatory wait.

### P1-4. No mechanism enforces the single-writer assumption (split brain)

- Location: `src/object_storage/catalog.rs` (no lease/fencing token)
- Verification: Code inspection

The architecture assumes "single machine, single writer," but **nothing enforces it.** Two processes with
the same `SIGNY_OBJECT_STORE_URL` prefix both believe they can operate normally. Manifest CAS prevents
lost updates but not the following.

- The two processes' local WALs/caches have different histories.
- Each retention worker can treat a part just registered by the other as expired.
- M6 hardware replacement reaches exactly this state if the new instance starts before the old one is fully dead.

M6 defines the order well, but **enforcing** the order is different from requesting it in documentation.
Operational automation (k8s rolling updates and so on) commonly violates procedures.

**Remediation**: Add a writer epoch/lease to the manifest and verify the own epoch during CAS. On observing
another epoch, immediately self-fence (reject ingest and terminate). This can be implemented with object storage alone.

### P1-5. MemTable flush deep-clones the full snapshot and size calculation is O(rows)

- Location: `src/memtable.rs:98-113` (`snapshot.clone()` in `begin_flush`), `src/memtable.rs:135-170`
- Verification: Code inspection

Two problems overlap.

1. `begin_flush` duplicates every entry with `snapshot.clone()` → memory doubles at flush time.
2. `approximate_size()` walks every entry in every stream. `flush_loop` calls it every
   `flush_check_interval` (500 ms by default), and `finalize_flush` calls it every loop.

This is harmless in normal operation (1 MiB memtable). The problem is **failure**. As the memtable grows,
the O(rows) walk every 500 ms consumes CPU, and `insert` waits for its write lock while the walk holds the
`inner` RwLock read lock → ingest latency worsens with memtable size. In P0-2's unbounded-growth scenario,
latency degrades linearly and accelerates the situation.

**Remediation**: Track size in O(1) with an `AtomicU64` accumulator (update on insert/commit/abort).
Share the `begin_flush` clone with `Arc`, or move the flushing buffer and reinsert it when needed.

### P1-6. No timestamp acceptance window causes unbounded partition growth — **fixed**

- Location: `src/part/format.rs:1-5` (`partition_of`), `src/ingest.rs:78` (only validates i64 range)
- Verification: Code inspection

Ingest checks only that timestamps fit the i64 nanosecond range. Because partitions are UTC-day based,
a clock-wrong client or a **unit mistake** (the common error of sending seconds/milliseconds as nanoseconds)
can create thousands of partition directories. In addition:

- A future-date part never reaches the retention cutoff (`max_ts_ns < cutoff`) and **remains forever.**
- `DateTime::from_timestamp(...).unwrap_or_default()` silently maps conversion failure to `1970-01-01`.

Loki has `reject_old_samples` and `creation_grace_period`; there was no equivalent defense here.

**Fixed**: Introduced `SIGNY_MAX_TIMESTAMP_AGE` (7d by default) / `SIGNY_MAX_TIMESTAMP_SKEW`
(1h by default) and reject with 400 before journal append (`TimestampWindow` in `src/ingest.rs`).
Disabling with `off` allows bulk loading historical data.

**Remaining work**: Expose rejection counters in metrics together with P2-7 (labeled metrics).
`unwrap_or_default()` in `partition_of` remains because every i64 nanosecond value produces a valid date,
so there is no practical risk.

### P1-7. No label/line/stream-cardinality limits — **partially fixed**

- Location: `src/proto.rs:90-140` (`parse_labels`), `src/memtable.rs:92`
- Verification: Code inspection

The following Loki limits are entirely absent.

| Loki limit | signy |
|---|---|
| `max_label_names_per_series` (30) | `max_label_names_per_stream` ✓ |
| `max_label_name_length` (1024) | `max_label_name_bytes` ✓ |
| `max_label_value_length` (2048) | `max_label_value_bytes` ✓ |
| `max_line_size` | `max_line_bytes` ✓ |
| `max_streams_per_user` | **Missing** — requires multi-tenancy |
| `max_entries_limit_per_query` | Present as `max_log_limit` ✓ |

A client that accidentally puts a request ID in one label can explode the MemTable stream HashMap and
the part stream index. The stream index is designed as a "small persistent catalog" excluded from the
cache limit (the `CATALOG_FILES` comment in `cache.rs`), so cardinality explosion becomes **non-evictable disk usage**.

**Fixed**: Apply limits for label count, name length, value length, and line size through `validate_labels`
and line validation in `src/ingest.rs`. All run before journal append, so rejected requests do not reach the WAL.

**Remaining work**: An active-stream limit (`max_streams_per_user`) is **meaningless without multi-tenancy** —
a global limit would let one tenant block another tenant's ingest. Address it with P0-3.

### P1-8. Merge defaults conflict — large-group merges fail permanently

- Location: `src/merge/selection.rs:31-76` (`group_for_merge`), `src/merge/selection.rs:89-120`
- Verification: Code inspection + default-value calculation

Group selection compares `estimated_part_bytes` (= **compressed** Parquet file size) with
`merge_max_input_bytes` (512 MiB). Actual reads compare **decompressed** row bytes in
`read_all_rows_with_limit` with `merge_max_memory_bytes` (1 GiB). A 5–20x compression ratio is common for
log text with zstd + dictionary, so the limits diverge by the compression ratio.

Rows are also risky. With `merge_target_part_rows` = 1,000,000 and 1 KiB lines, decompressed size is about
1 GiB, exactly at the default limit. If a group grows to `merge_max_part_rows` = 4,000,000, it certainly exceeds it.

When exceeded, it returns `Err("merge exceeds the maximum of ... materialized bytes")` and `merge_once`
continues. But **the same group is selected the same way on the next tick and fails for the same reason.**
There is no fallback to a smaller group. Result: permanent merge failure → unbounded part growth → higher
query-planning cost → `/ready` stuck at 503 with `merge_healthy=false`.

**Remediation**
- Use the same unit for both limits, or use a compression-ratio estimate during group selection (record uncompressed size in part metadata).
- Add a fallback that halves the group and retries on memory-limit failure.
- Ultimately replace merge with a streaming k-way merge to avoid materializing everything
  (the same axis as "Parquet range read" in P2 of `todo.md` — that item is now struck, but for a reason
  that does not carry here: it lost on object-store request count, and a streaming merge issues none).

### P1-9. Cache eviction holds the registry write lock during synchronous directory traversal

- Location: `src/startup.rs:242-250`, `evict_cache`/`evict_trace_cache` in `src/object_storage/cache.rs`
- Verification: Code inspection

The eviction worker takes `registry.operation_lock().write_owned()` and then calls `evict_cache`.
`evict_cache` is a **synchronous function**, neither async nor wrapped in `spawn_blocking`, and traverses the
entire parts tree with `read_dir` + `symlink_metadata` for every entry.

Two things are bad simultaneously.

1. **All queries, flush, merge, and retention are blocked** throughout the traversal (write lock).
2. A Tokio worker thread is blocked (every 30 seconds by default).

With tens of thousands of parts, every traversal performs tens of thousands of stat syscalls. This appears
as periodic spikes in query p99.

**Remediation**: Move it to `spawn_blocking` and keep size/access-time data in the registry so disk traversal
disappears. Hold the write lock only during actual file deletion.

### P1-10. Object-store startup errors panic immediately → crash loop on transient S3 failures

- Location: `src/startup.rs:83, 87, 91, 95, 109`
- Verification: Code inspection

Startup handles object-store initialization, flush-transaction recovery, local-cache reconciliation, and
trace reconciliation all with `panic!`. It **does not distinguish transient network errors from actual data
corruption.** If the process restarts while S3 is unstable for a few seconds, it panics; the orchestrator
restarts it and it panics again → crash loop. Ingest stops completely during the loop while the Alloy WAL fills.

**Remediation**: Retry recoverable I/O errors with backoff while keeping `/ready` at 503 (the listener must
be up so `/ready` can respond). Leave only real integrity violations (manifest format errors, validation
failures) as panics.

### P1-11. Startup time and flush cost are linear in part count

- Location: `restore_catalog` in `src/object_storage/cache.rs`, manifest CAS in `src/object_storage/catalog.rs`
- Verification: Code inspection

There are two O(N) paths.

1. `restore_catalog` checks local existence for **every** part in the manifest and then downloads missing catalog
   files **sequentially**. There is no parallelism. With 10,000 parts, tens of thousands of round trips are serial.
   `reconcile_local_cache` also calls `restore_catalog` twice, at the beginning and end, and calls
   `load_manifest()` again for every merge group.
2. The manifest is **one JSON containing every part**, rewritten in full by CAS on every flush (5 seconds by default).
   With 10,000 parts, several megabytes are PUT every five seconds. S3 request cost and CAS collision probability both rise.

The run in `docs/M7_LOAD_RESULTS.md` had three parts, so this axis was not validated at all.

**Remediation**: Parallelize catalog downloads (bounded concurrency) and remove duplicate
`load_manifest`/`restore_catalog` calls from `reconcile_local_cache`. In the medium term, change the manifest
to generational deltas plus periodic snapshots so each flush writes O(changes).

---

## P2 — operational quality / compatibility

### P2-1. Loki API compatibility gaps

- Location: `src/router.rs`
- Verification: Code inspection + comparison with the Loki API

Endpoints called by the Grafana Loki data source but missing:

| Endpoint | Impact |
|---|---|
| ~~`/loki/api/v1/tail` (WebSocket)~~ | **implemented.** Polls the ordinary query path rather than pushing from ingest, so it inherits every limit, the retention clamp and tenant isolation instead of reimplementing them |
| ~~`/loki/api/v1/index/volume`, `volume_range`~~ | **implemented.** Expressed as `bytes_over_time` and answered by the metric evaluator, so it inherits the scan budgets, the retention clamp and the tenant scope |
| `/loki/api/v1/patterns` | **implemented as a read-time miner over a bounded sample.** The objection stands and shapes the answer: pattern mining is a heuristic with no compatibility contract, so this one is not claimed to match Loki's, and it does not earn a second index on the write path. The response reports the lines it sampled, so a reader knows the summary describes the sample rather than the window |
| ~~`/loki/api/v1/detected_fields`, `detected_labels`~~ | **implemented.** Labels with cardinality from the same sources `labels` reads; fields from structured metadata over a bounded sample |
| ~~`/loki/api/v1/format_query`~~ | **implemented as a validator.** It does not rewrite: a faithful renderer for the whole LogQL surface is a second grammar to keep in step with the parser, and a formatter that silently changes a query is worse than one that leaves it alone |
| `/loki/api/v1/delete` (delete API) | **implemented, on the two terms this review named.** Request state is durable — one object per request, loaded at startup, fatal on failure — and query-time masking sits at the single scan every read path funnels through. The bytes leave through the merge rewrite that already existed, using the same per-row predicate, and `status` reports which of the two has happened |

Implemented but inaccurate:

- `labels`, `label_values`, `series`, and `index_stats` **completely ignore `start`/`end`**
  (`handlers.rs:199, 213, 290, 426`). Grafana always sends a time range, so dropdowns include labels
  that do not exist in that range; every request also scans the full history.
- `label_values` does not support Loki's `query` parameter (filter values with a matcher).
- ~~JSON push rejected with 415~~ — **implemented.** It decodes into the same streams the protobuf form
  produces and follows the same path, so the input limits and the journal encoding are not repeated for it.
- `buildinfo` hardcodes `revision: "unknown"`, `branch: "main"` (`handlers.rs:281`). There is no way to identify the deployed revision.

~~Tempo also lacks the v2 APIs and `/api/echo`~~ — **implemented.** The datasource tries v2 first and falls
back to v1, so without them every tag lookup paid a failed request before the one that worked. v2 answers
from the same traversal as v1.
The latest Grafana Tempo data source tries v2 first.

### P2-2. No resource guards on metadata endpoints

- Location: `src/query/handlers.rs:199, 213, 290, 426`
- Verification: Code inspection

Log/metric query paths have semaphores, timeouts, scan budgets, and memory budgets. However,
`labels`, `label_values`, `series`, and `index_stats` have **no semaphore, timeout, or range validation.**
`series` accepts unlimited `match[]` entries and scans every part for each matcher. Combined with P0-3
(no authentication), this is the cheapest DoS path.

### P2-3. Snappy decompression allocates the reported length, and body limit is not configurable — **fixed**

- Location: `src/ingest.rs:54`
- Verification: Inspected `snap-1.1.2` source (`vec![0; decompress_len(input)?]` in `decompress.rs`)

`decompress_vec` **immediately allocates the length reported by the header** before validation. Snappy's
`MAX_INPUT_SIZE` is `u32::MAX`, so a varint header only a few bytes long can trigger an allocation up to 4 GiB.
`MAX_RECORD_BYTES` (256 MiB) validation happens **after** decompression.

With Linux's default overcommit, `vec![0; n]` uses lazy zero pages so RSS impact is initially limited.
It fails with overcommit disabled or under virtual-address pressure, and more importantly, "untrusted input
determines allocation size" is itself a pattern that must be fixed.

Related problem: **the body-size limit is not configurable.** The implicit 2 MiB axum default is used,
so an operator who increases Alloy's batch size encounters an unexplained `413` with no environment knob to adjust.

**Fixed**: Check the reported length first with `snap::raw::decompress_len` and reject with `413` when it
exceeds `SIGNY_MAX_DECOMPRESSED_PUSH_BYTES` (64 MiB by default). Expose `DefaultBodyLimit` as
`SIGNY_MAX_PUSH_BYTES` (16 MiB by default), and have the handler check the same limit to provide a
specific error instead of axum's uninformative 413.

### P2-4. Retention deletes local bodies before manifest CAS — **partially fixed**

- Location: `src/retention.rs:130-165`
- Verification: Code inspection

It deletes local part directories first (under the write lock) and then performs remote manifest CAS.
Crash safety is as described in the comments, but **when CAS fails**, the part remains registered in the
registry and manifest while only its local body is missing (`removed_log_ids` calls `unregister` only after
CAS succeeds). Queries that need to restore the part fail until the next retention pass. It converges, but
query results error in the meantime.

Related problems:
- ~~Retention/GC timeout reuses `config.max_restore_runtime` (25 seconds)~~ → **Fixed**:
  `SIGNY_MAX_RETENTION_RUNTIME` (120 seconds by default) is separate. The old 25 seconds was too short
  for GC listing an entire prefix and would repeatedly time out on large buckets.
- `garbage_collect_orphans` LISTs the entire `parts` and `trace_parts` prefixes every time. LIST cost linear
  in object count occurs whenever retention deletes anything.
- `retention_period` defaults to `None` (infinite). A default deployment grows S3 and disk forever.

### P2-5. Duplicated logs after a crash cannot be observed

- Location: at-least-once item in `docs/ARCHITECTURE.md`, P2 in `todo.md`
- Verification: Documentation + code inspection

The at-least-once trade-off is documented as a conscious decision. The problem is that **there is no way
to know whether duplicates occurred.** There is no metric counting records replayed during crash recovery
and no warning log. Operators cannot determine whether the `count_over_time` result they see is inflated by duplicates.

**Remediation**: Even before deduplication, expose gauges such as `signy_replay_records_total` and
`signy_replay_duplicate_window_bytes`, and emit a WARN during recovery saying "duplicates may exist
after this point."

### P2-6. Log level hardcoded — ignores `RUST_LOG` — **fixed**

- Location: `src/main.rs:40` — `.with_env_filter("signy=debug,info")`
- Verification: Code inspection

Production forces `signy=debug` and operators cannot change it. This is a log-volume/cost problem and
also prevents temporarily increasing the trace level during a failure.

**Fixed**: Honor `RUST_LOG` with `EnvFilter::try_from_default_env()` and lower the default to
`signy=info,warn` when unset.

### P2-7. `/metrics` cannot calculate SLOs

- Location: `src/metrics.rs`, `src/query/handlers.rs:299-420`
- Verification: Code inspection

- There are no histograms/summaries. Latency is only the sum `*_latency_ns_total`, so **p95/p99 cannot be calculated.**
  M5/M7 target tables use p95/p99, but those metrics are unavailable in operation.
- There are no labels, so error rates cannot be calculated per endpoint or status code.
- There are no `# HELP` lines.
- There is no version/revision metric.
- Every scrape makes `merge_debt_part_count` call `estimated_part_bytes` → `fs::metadata` for every part
  (`selection.rs:78-82`). A stat syscall per part occurs on every scrape interval.

### P2-8. Shutdown operator abort does not work in container environments

- Location: `spawn_abort_watcher` in `src/shutdown.rs`, shutdown sequence in `startup.rs`
- Verification: Code inspection (some code comments also acknowledge it)

Force-flush retries forever without a hard timeout, and the only escape is **typing `exit` on stdin**.
Under systemd or in a container, stdin is not a TTY and this path is unusable (the code acknowledges
"stdin is unavailable; operator-initiated shutdown abort is disabled").

The actual container behavior is: SIGTERM while S3 is down → infinite force-flush retries → orchestrator
`terminationGracePeriodSeconds` expires (30 seconds by default) → **SIGKILL**. Data is in the WAL and is
not lost, but the orchestrator can **schedule the next pod on another node**, discarding that disk. The
loss M6 intended to prevent occurs through exactly this path.

Returning exit code 1 on abort in `startup.rs` is good design, but **SIGKILL has no exit code.**

**Remediation**: Receive abort through an administrative endpoint or signal (SIGUSR1) instead of stdin.
More importantly, state in the operations documentation that this workload must use a StatefulSet + fixed
PV and `terminationGracePeriodSeconds` must be effectively infinite, and provide a controller procedure
that replaces hardware based on `/ready` + `pending_flush_bytes`.

### P2-9. `file://` backend overwrites without CAS — no production-misuse defense — **partially fixed**

- Location: `local_manifest_overwrite` in `src/object_storage/catalog.rs`
- Verification: Code inspection

The `file://` scheme falls back to `PutMode::Overwrite` (relying on an in-process mutex + rename).
Comments say "single-process development backend," but **there is no runtime warning.** If someone points
`file://` at an NFS mount, manifest updates silently lack CAS and combine with P1-4 split brain to cause lost updates.

**Fixed**: When `ObjectStorage::from_url` detects the `file://` scheme, it emits a WARN that writes
overwrite without CAS and must not use shared/network storage.

**Remaining work**: Requiring explicit opt-in (`SIGNY_ALLOW_UNSAFE_LOCAL_STORE`) must be coordinated
with the load-harness script, which depends on `file://`.

### P2-10. No overload signal (429), so Alloy cannot back off

- Location: `src/ingest.rs`, `src/trace_ingest.rs`
- Verification: Code inspection

Current response mapping is parse failure → 400 (Alloy drops it, correctly), journal failure → 500 (Alloy
retries, correctly). But **there is no 429 to represent overload.** When the server cannot cope, it cannot
tell the client to "slow down," so P0-2's unbounded growth is not mitigated. OTLP also uses `RESOURCE_EXHAUSTED`
only for size overflow, not load.

---

## P3 — missing deployment and documentation assets

Verification: `ls` (the files do not exist)

| Item | Status |
|---|---|
| Dockerfile | Missing |
| k8s manifest / Helm chart | Missing |
| systemd unit | Missing |
| Configuration reference | Missing — roughly 40 environment knobs exist only in `src/config.rs` |
| Operations runbook | Partial (only the shutdown procedure in `ARCHITECTURE.md`) |
| Backup/DR procedure | Missing (S3 is the source of truth, but versioning/replication policy is undocumented) |
| SLO/capacity guide | Missing (M5 target table exists only in the plan document) |
| Example alert rules | Missing |
| Real S3 validation | Incomplete (`todo.md` P2; only MinIO validated) |

`docker-compose.yml` is for MinIO load tests and does not deploy the service itself.
`scripts/` contains only two load harnesses.

At minimum, the following are needed.
- Configuration reference (`docs/CONFIGURATION.md`): meaning, defaults, tuning direction, and mutual constraints for each knob (such as P1-8's `merge_max_input_bytes` vs `merge_max_memory_bytes`)
- Runbook: wedge recovery (including manual deletion of `journal.wal.compact.state`), S3 failure response, full-disk response, and hardware-replacement checklist
- Alerts: increase rate of `flush_errors`, `wal_backlog_bytes`, `merge_debt_parts`, `remote_healthy`, and `pending_flush_bytes`

---

## Production-readiness gates (recommended order)

Status is kept current here; the numbered sections above are the original
findings and are not rewritten. Where a fix changed a decision rather than only
the code, the note says which.

### Gate 1 — data safety (do not deploy without this)

- [x] P0-1 WAL-compaction wedge — intent record removed durably on success; a surviving phase-2 record is treated as complete, so an already-wedged instance recovers on upgrade alone
- [x] P0-2 ingest backpressure (MemTable/WAL-backlog limit → 429, before journal append)
- [x] P1-4 writer fencing (manifest `writer_epoch`, claimed at startup, verified on every CAS, self-fence on loss)
- [x] P1-6 timestamp acceptance window
- [x] P1-8 merge-limit unit mismatch — `materialized_bytes` in part metadata, plus the group-splitting fallback

### Gate 2 — multi-tenancy and input defenses

- [x] Document TLS unsupported as an architecture decision (`ARCHITECTURE.md`, "Transport security")
- [x] P0-3 `X-Scope-OrgID` extraction and tenant isolation — as a *shared*-object axis, not a storage-path one. Per-tenant objects were rejected on Class A cost; see [`MULTI_TENANCY_DESIGN.md`](MULTI_TENANCY_DESIGN.md)
- [x] P0-3 per-tenant ingest rate — `ingest_rate` on the pushed policy, enforced before the body is decompressed
- [x] P0-3 per-tenant query-scan quota (`query_rate` on the pushed policy) and per-tenant query concurrency
- [x] P0-3 stream-cardinality limit (`max_streams` on the pushed policy)
- [x] P0-3 per-tenant usage — on the authenticated admin API rather than as `/metrics` labels, which would multiply every series by the tenant count
- [x] N2 tenant allowlist (`SIGNY_ALLOWED_TENANTS`)
- [x] Default bind moved inside the trust boundary — loopback unless configured, and startup says which side of it the listener landed on
- [x] P2-2 resource guards on metadata endpoints (semaphore, timeout, `start`/`end`, `match[]` count)
- [x] P2-3 validate snappy reported length + expose body limit
- [x] P1-7 label/line size limits

### Gate 3 — operability

- [x] P1-10 retry transient object-store failures at startup (`SIGNY_STARTUP_RETRY_BUDGET`). The remaining `panic!` sites are the deliberate give-up past that budget, where the orchestrator's restart backoff is the better place to escalate
- [x] P2-6 honor `RUST_LOG`, default `info`
- [x] P2-7 latency histograms (p95/p99 derivable). Endpoint labels are still absent
- [x] P2-8 non-stdin abort (`SIGUSR1`) + the orchestrator requirement and the two ways out documented in [`RUNBOOK.md`](RUNBOOK.md)
- [x] P2-9 warn on `file://` production misuse (opt-in enforcement remains)
- [x] P2-4 `retention_period` default decided — unbounded, because per-tenant retention is the mechanism and a global default would delete data the control plane believes it owns. Startup warns when neither is configured
- [x] P3 Dockerfile + configuration reference + runbook + alert rules
- [x] **N7 readiness flapped on isolated object-store failures** — health is hysteretic now; measured 41-66% healthy before, 99.3-100% after
- [x] **N8 merge and flush contend** — measured bounded: oscillates 6-140 MB with a negative trend. A tuning note, not a defect
- [x] **N9 peak RSS is concurrent live memory** — returns fully when load stops. A sizing rule, not a defect
- [x] P2-5 duplicates after a crash are observable — startup reports what replay put back, as a WARN and as a gauge. Removing them is deduplication, still open

### Gate 4 — scale validation

- [x] P1-11 catalog restore overlaps its downloads, and reconcile stops re-reading the manifest per merge group
- [x] P1-11 removed the redundant third catalog pass — about 18 s of the 63.7 s
- [ ] P1-11 remaining: two catalog passes are still inherent, and the manifest is still one document rewritten in full per flush. Generational deltas plus periodic snapshots are a format change
- [x] P1-1 / N6 Tempo time pruning — `search` and both tag endpoints. `search` also changed rule: a trace matches on span overlap rather than on its earliest span, which is Tempo's semantics and the one the row-group bounds can answer
- [x] P1-3 group-commit latency structure — the batch loop no longer waits out `max_batch_ms` on an empty channel; the default is now zero linger
- [x] P1-5 MemTable size tracked in O(1)
- [x] P1-5 flush deep clone removed — the snapshot is shared with the flush through an `Arc`, on both the log and trace memtables
- [x] P1-9 eviction moved to `spawn_blocking`, carrying the lock guard with it so the atomicity against a reader pinning a part is unchanged
- [x] P1-9 eviction is driven by the registry's part directories instead of walking the tree
- [x] Instrument the layout axis — `part_tenant_segments`, `part_sidecar_resident_bytes`, `part_meta_bytes`. These are what a Tier D run has to answer before the N3 mitigation can be chosen
- [ ] Sustained load for **at least 24 hours** at the target specification (4 vCPU / 16 GiB). Real S3 is out of scope; local MinIO is the limit ([`LOAD_VALIDATION.md`](LOAD_VALIDATION.md))
- [x] Startup at 10,099 parts measured: **63.7 s**, 58% of it the log catalog restore. Query planning stays fast (0.02 s warm)

### Gate 5 — feature completeness

- [x] P1-2 OTLP logs — `LogsService` on gRPC, `POST /v1/logs` and `/v1/traces` on HTTP, protobuf and JSON
- [x] P2-1 `tail` (WebSocket live tail) and time ranges for `labels`/`series`
- [x] P2-1 `index/volume`, `volume_range`, `detected_labels`, `detected_fields`, `format_query`
- [x] P2-1 remaining: `patterns` and the `delete` API — both now implemented, see the table above
- [ ] P2-5 deduplication itself (`todo.md` P2)
- [ ] LogQL improvements in P1 of `todo.md`

---

## Strong areas

Items judged especially robust during the review — assets that must not be broken by future changes.

- **Crash-recovery invariants are explicit in both code and comments.** Flush transactions, merge-tombstone
  chain recovery, upload markers, and phase-based compaction intent explain what each crash point guarantees
  and why. This level is rare.
- **Path-safety validation is consistent.** `is_safe_path_component`, symlink rejection, and canonical-root
  checks cover cache, manifest, and transaction paths.
- **Resource budgets on log/metric query paths are tight.** Scan rows/bytes, materialized memory, concurrency
  semaphores, timeouts, and cancellation flags all exist and are tested.
- **Epoch-based CAS for remote health** (`remote_state` in `catalog.rs`) correctly prevents an old success from
  overwriting a newer failure.
- **Flush visibility transition is inside one write lock,** making part registration and MemTable commit atomic.
- **M7 load validation found its own blocker and recorded it as FAIL instead of hiding it.** The root-cause analysis
  in `docs/M7_LOAD_RESULTS.md` exactly matches this review's reproduction.
- All 211 tests pass, and crash-injection tests on journal, part, and object-storage paths are substantive.

---

## Appendix — reproducing P0-1

Add the following to `src/journal/tests.rs` to reproduce it (the second record must be smaller than the first).

```rust
#[tokio::test]
async fn two_consecutive_compactions_wedge() {
    let harness = harness("two_compactions").await;
    push(&harness, make_push_req(&[("{app=\"a\"}", vec![("one", 100)])])).await;
    let first = harness.journal.checkpoint().await.unwrap();
    harness.memtable.commit_flush();
    harness.journal.compact_checkpoint(first.offset).await.unwrap();

    push(&harness, make_push_req(&[("{app=\"b\"}", vec![("2", 200)])])).await;
    let second = harness.journal.checkpoint().await.unwrap();
    harness.memtable.commit_flush();
    let result = harness.journal.compact_checkpoint(second.offset).await;
    assert!(result.is_ok(), "second compaction failed: {result:?}");
}
```

Observed result:

```
first_offset=32 wal_after_first=0 second_offset=31
result=Err(Custom { kind: InvalidInput, error: "WAL compaction checkpoint moved backwards" })
wal_after_second=31
```

If the second record is made **the same size** as the first, the `offset == state.offset` branch returns
`Ok`, but the WAL is not truncated (a silent no-op).
