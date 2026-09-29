# signy architecture

A single-machine log, trace and metric engine written in Rust. It combines VictoriaLogs' logical design with
the Parquet physical format and S3 tiering.

This document records what the engine *is*. [`VISION.md`](VISION.md) records what it is *for* — the three
invariants that are load-bearing, what is deliberately not built, and what would falsify the claim. Where
the two disagree, `VISION.md` is the intent and this one is the implementation.

[`COMPARISON.md`](COMPARISON.md) records how it currently measures against Loki on the same machine, at the
same container memory limit, over the same corpus. **The claim in `VISION.md` does not survive that run**:
signy loses `| json | field="x"`, its own headline query, by 1.85x. It no longer loses on memory —
both systems survived a 2 GiB container, where the first run had to be published at 8 GiB because
signy was OOM-killed at 2.
The bed is [`compare/`](../compare/), the raw artifacts are in [`artifacts/m9/`](artifacts/m9/), and the
document is regenerated from them rather than written. Read it before trusting any performance statement
elsewhere in these docs.

## Decided choices

| Item | Decision |
|---|---|
| Deployment | Single machine, single writer |
| Source of truth | S3-compatible object storage |
| Local disk | Cache (LRU eviction) |
| Memory | **One declared budget** (`SIGNY_MEMORY_BUDGET`) divided into ingest/flush/merge/query/sidecar arenas, each with its own accounting and its own refusal. Not a set of independent limits whose product is discovered afterwards — see [`VISION.md`](VISION.md) invariant I |
| Format versioning | **None.** Nothing on disk or on the wire is versioned and no build reads another's data — see [`VISION.md`](VISION.md), "What is deliberately not built". Changing a format means changing it; a stale data directory is deleted |
| Execution engine | **Hand-written.** DataFusion is rejected: its memory is not accountable at arena granularity, and the flat filter surface is small enough that a planner for it is smaller than the integration |
| Durability | Journal (append-only) + group commit + ack after fsync. Alloy WAL is assumed as a safety net |
| Replication | Catalog generations have two fixed replicas in one bucket. Entire-bucket loss, or loss of both replicas, remains outside the guarantee; the accepted unflushed RPO is determined by the flush interval (`flush_max_bytes`/`flush_max_interval`, whichever comes first: 1 MiB/5 s by default) |

## Durability and recovery semantics

- **WAL + checkpoint invariant**: An acked record is always in the WAL and is also inserted into the memtable (the writer task inserts it after a successful write and immediately before ack). Therefore, the `(offset, memtable snapshot)` captured by `checkpoint()` is atomically consistent.
- **Recovery**: On startup, replay only `[checkpoint..replay_end]` from the WAL into the memtable and truncate a corrupt or partial tail at `replay_end`. Do not advance checkpoint during recovery — in-flight data exists only in the memtable, so checkpoint stays unchanged until the next flush's `checkpoint()` records the correct offset. Thus, in-flight data survives repeated "restart -> restart" cycles.
- **At-least-once (flush boundary)**: If the process crashes after flush completes part disk writes but before `set_checkpoint`, the part and the next replay can both contain the same data. This is an intentional durability-over-correctness trade-off; duplicates can appear in query results and deduplication is deferred to a later milestone.
- **Flush visibility boundary**: Queries hold the part/memtable operation read lock for the full scan, while flush performs part registration and flushing-buffer commit under the same operation write lock. A metric or log query overlapping a normal flush therefore does not count the same row in both memtable and part. At-least-once recovery duplicates remain possible when flush stops after part commit but before checkpoint, and durable deduplication is deferred to a later milestone.
- **Single writer, enforced rather than assumed**: the catalog carries a `writer_epoch`. A starting writer takes the next one, and every subsequent generation create re-checks the epoch it loaded (`object_storage/catalog.rs`, `check_epoch`), so a previous instance that is still running — the split-brain a "single writer" deployment note cannot prevent on its own — fails its next write instead of interleaving with the new one, and stops rather than retrying forever.
- **Two lifecycle locks, and their order is load-bearing**: the *operation* lock is visibility — a query holds its read half for a whole scan, and flush, merge commit and part retirement take its write half. The *deletion* lock guards part files against removal, and nothing else: a merge rewrite holds its read half for as long as its group takes, because the rewrite needs its inputs to exist but does not care what is visible. **Deleters take both, deletion first.** Retention and cache eviction took the operation lock first until 2026-08-11, which meant a deleter's wait for a merge rewrite was served holding the lock every query needs: all 39 freezes across four one-hour soaks began on a retention tick, the longest 52 s, one of them to delete a single part. Deletion-then-operation is also merge's own order, so no cycle remains (`retention.rs`, `part_registry.rs`'s `deletion_lock` doc, and `a_retention_pass_waiting_for_a_merge_does_not_stop_queries`).
- **Merge replacement invariant**: A merge tombstone is recorded before the new part directory is renamed from `.tmp` to its final location. Restart recovery deletes old parts only after successfully opening the new part; if validation fails, old parts are retained.
- **Merge tombstone chain recovery**: On restart, collect all tombstone relationships first, follow them transitively through earlier generations, and then clean up old parts. Thus, even when merges across generations overlap after a failed deletion, deleting an intermediate tombstone cannot resurrect an earlier-generation part.
- **Size tiers decide what compacts together** (`compaction_tier.rs`, shared by the log merge, the trace compactor and the metric compactor). A tier is a size band eight times as wide as the one below, so a full tier of parts merges into one part of the next tier and a byte is rewritten once per tier it climbs — about 3.7 times over a day of one-minute flushes, whatever the flush size. Grouping by a fixed ceiling instead, which is what these three did until 2026-09-16, put the part a partition had been accumulating in a group with each flush's new parts: simulated over a day, the log grouping wrote 241 times what it ingested and four days wrote 961, and production wrote 111 MB of trace parts in seven hours for about 1.5 MB of spans. A partition whose newest row is older than an hour compacts a tier of two parts as well, so a partition nobody writes to any more does not keep its stragglers forever.
- **Trace compaction**: A full size tier of trace parts in one partition is rewritten into one part (`trace_merge.rs`). The pass holds the deletion lock's read half while it restores missing bodies and rewrites, writes a commit record under `traces/.compact/` once the replacement is durable, swaps the inputs for the replacement in one catalog commit, and only then retires the inputs under the operation lock. Local mode resolves a leftover record before the registry loads; object-store mode replays it against the manifest during reconcile, and so does the metric compactor's record. Input objects are left to the orphan collector.
- **Object-store collection**: Orphaned part objects are collected on `SIGNY_ORPHAN_GC_INTERVAL`, independent of retention, and catalog commits and snapshots older than `SIGNY_CATALOG_PRUNE_MIN_AGE` are pruned when no startup reads them any more: two snapshots that verify on both replicas, and every commit from the older one on, always remain (`object_store_gc.rs`, `prune_catalog`).
- **WAL compaction**: With object storage enabled, the writer task removes the WAL range before checkpoint after immutable part upload and one global catalog commit succeed. Checkpoint is reset to 0 before replacement, so a crash during compaction replays either the entire old WAL or the new suffix; duplicates are possible but data is not lost. Local-only mode preserves the existing offset checkpoint.
- **What an acknowledgement costs, and why this engine's push latency looks the way it does.** A `204` here means the bytes are on the device, not in a buffer: the writer task writes the batch, `sync_all`s it, inserts it into the memtable, and only then answers. Measured over an hour at 20 k eps, that sync **is** the push service time — write, memtable insert and checkpoint together are under 0.5 ms of a 12 ms median, while `sync_all` averages 7.6 ms and is amortized over 1.6 pushes per batch (`signy_journal_*_ms` on `/metrics`; `todo.md`, 2026-08-12). The tail is the same fsync waiting behind merge's own writes on the shared device: 294 batches over 250 ms in one hour against 294 merges, 235 of them within 5 s of a merge completing, the worst a single 10 KB record taking 4.1 s to become durable.

  This is a **deliberate trade and it is why the comparison bed shows this engine's push p95 well above Loki's and VictoriaLogs'** (91.9 ms against 4.7 and 1.4 at 1.5 M rows). A 1.4 ms acknowledgement cannot contain a device sync on this hardware — the same hardware where this engine measures one at ~7 ms — so those systems are answering before the data is durable and flushing behind it. Neither behaviour is wrong; they are different promises, and this one is the promise the "the client's own WAL is the safety net" premise elsewhere in this document depends on. **The cost is not being reduced**: the alternatives all buy roughly 9% of one latency percentile with something real — a linger raises the floor for every idle-server push, pacing merge trades read latency for write tail, and splitting the writer gives up the single ordered WAL that makes a checkpoint one number. See "the ceiling is flush, not the WAL" in [`CONFIGURATION.md`](CONFIGURATION.md): the sync path is not what limits capacity either.
- **Unexpected disk loss**: If the local disk is lost entirely, WAL/MemTable data not flushed after the last successful flush (S3 upload + catalog commit) cannot be recovered from the server side. This loss window is determined by `flush_max_bytes`/`flush_max_interval` (1 MiB/5 s by default, whichever comes first), and this level of loss is intentionally accepted without WAL replicas. Planned hardware replacement follows the graceful-shutdown procedure below so this window does not apply.

| Physical format | Parquet (dictionary + zstd) + sidecar index files |
| Indexes | Stream index + per-block trigram bloom filter (no inverted index) |
| Query language | None — a query is a flat AND of URL filters; refusals teach the accepted set |
| API | First-party flat-filter query API for logs, traces and metrics ([`QUERY_API.md`](QUERY_API.md)); the Loki and Tempo compatibility surfaces were removed with the read-path decision (issue #3, M12), and the first-party trace endpoints (M13, issue #7) are their replacement. The metric surface arrived with M14 as seven routes under `/signy/api/v1/metrics/` |
| Ingest protocols | **collecty only** — one route, `POST /signy/api/v1/collect`, taking a collecty's zstd batch of OTLP exports for any of the three signals. The OTLP push routes (`/v1/logs`, `/v1/traces`, `/v1/metrics`) and the OTLP gRPC services on `:4317` are removed; applications export to collecty, which ships here. See [`VISION.md`](VISION.md), "Ingest is OTLP", and "Ingest is collecty's" below |
| Query protocol | **First-party HTTP API** (GET + URL filters, NDJSON out). The viewer is the fn0 control plane behind the gateway; agents drive the same endpoints with `curl` |
| Transport security | **TLS is unsupported.** Only plain HTTP is provided; a reverse proxy or service mesh handles end-to-end encryption |
| Multi-tenancy | Multi-tenant. A write names its tenant in the `tenant.id` resource attribute, a read in the `X-Tenant-Id` header, and tenants are the unit of quota and retention |
| Validation environment | **Do not test against S3** (neither real cloud nor local MinIO). Trust the `object_store` crate and test our code closely up to the crate boundary |

## Transport security — TLS unsupported

TLS termination is not this process's responsibility. Certificate issuance, renewal, SNI, and mTLS
policies belong to layers that already handle them well (reverse proxy, ingress, or service mesh),
while the engine provides only plain HTTP. S3 access in the storage layer uses HTTPS
through `object_store`, so it is unaffected by this decision.

Therefore, deployments must satisfy the following requirements.

- Keep the listening address inside the trust boundary. **signy is not built to be a server
  reachable from the outside network**: it is provided on the assumption that every request — the
  admin API included — arrives through a secured channel, and direct exposure to a public network is
  unsupported.
- Perform authentication and authorization outside this process (at a proxy or gateway). A read's
  tenant arrives as `X-Tenant-Id`, which the gateway should overwrite; a write's arrives inside the
  export, written by the exporting application's SDK, and a stage that wants it enforced has to
  overwrite it there. The engine proves neither — **tenant isolation fails if the engine is directly
  reachable from a network location where either can be forged.**
- The listeners bind loopback by default. A `0.0.0.0` bind is a deliberate configuration and belongs only behind something that terminates TLS and authenticates.

## Validation environment — do not test against S3

Do not connect to real cloud storage because of cost, or to local MinIO because **we trust the
`object_store` crate**. Correct S3 protocol implementation is the crate's responsibility and it has
its own test suite. Load validation uses only in-process latency and fault injection.

The validation principle is **test only our code**. Whether the S3 wire protocol, conditional PUT
semantics, and multipart behavior are correctly implemented belongs to the `object_store` crate, so we
do not revalidate them here. Our responsibility is one layer below — **whether the store created by this
binary with this configuration actually performs CAS** — and that is checked by a startup preflight,
not a load test (see below).

Therefore, load runs observe our loop behavior (flush progress, bounded backlog, backpressure, stable
RSS, and startup time), and none of these items depends on the object-store backend. The following risks
remain nonetheless.

- **Latency tail** — Loopback is below 1 ms while real S3 p99 is hundreds of milliseconds to seconds.
  The latency-injection wrapper is backend-agnostic, so running it over MinIO can provide both real wire
  behavior and a tail, and the load script does this by default. However, injected values are assumptions,
  not measurements, and because injection occurs above the `object_store` client, its retry layer is not
  validated. **This is the largest remaining risk.**
- **Throttling** — S3 limits request rates per prefix while MinIO does not. With the default configuration,
  catalog commits are about 0.4 PUT/s (two replicas) and the total is about 10 PUT/s, four orders of magnitude below the limit,
  so the actual risk is small.
- **Cost** — This design has been dominated by R2 Class A costs, but MinIO provides no cost signal.
  Amounts cannot be measured locally, but operation counts can.
- **Provider-specific conditional PUT semantics** — CAS working on MinIO does not imply that it works on R2.

**CAS preflight.** At startup, `ObjectStorage::verify_conditional_put` checks with a probe object that
*a write that should be rejected is rejected*, and refuses startup otherwise. The positive path proves
nothing — the first catalog write in an empty prefix succeeds whether conditions are honored or not.
Because this check runs against the deployment target itself, it answers the locally unanswerable question
of whether CAS works with that provider. `file://` is a development backend that intentionally gives up
CAS, so it is skipped.

Under an R2 Bucket Lock rule, a create on a catalog key that already exists is answered with
`409 ObjectLockedByBucketPolicy` rather than `412`. `object_store` maps both to `AlreadyExists`, so a
writer that lost the race to a generation rereads the catalog either way; this was checked against R2.

The procedure and acceptance criteria are in [`LOAD_VALIDATION.md`](LOAD_VALIDATION.md).

## Multi-tenancy

Multi-tenancy is not an optional feature but **the basic unit of resource management**. Because quotas
and retention are operated per tenant, the tenant must be a first-class identifier across ingest,
storage, and query paths.

- **Identification, writes**: the `tenant.id` **resource attribute** of the OTLP export, the same key
  for all three signals on both transports. `Resource` is the one field `ResourceLogs`, `ResourceSpans`
  and `ResourceMetrics` share and is already a batch boundary; an attribute on a record or a span would
  split a request per row. The key is fixed rather than configurable — a key both ends must agree on is
  a key that can be disagreed about, and the failure is silent. Validate against `[a-zA-Z0-9_-]{1,64}`
  **before** journal append because it is used directly in object-store keys and local file paths, and
  strip it before storage so the routing key does not become a queryable label.
  - **One export may name several tenants.** Its resources are grouped and one journal record is
    written per group. The bytes that arrived are passed through to the WAL untouched when there is one
    group and nothing was dropped, which is every export a collecty forwards.
  - **A tenant refusal is a drop, not a refusal.** The status an ingest answers says whether the body
    arrived — did it decode, does the stream decompress, can the instance take it — and nothing about
    whose it was. A resource naming no tenant, an unparseable one, one this instance does not serve, or
    one at its storage limit is dropped and counted in
    `signy_ingest_dropped_resources_total{reason=...}`. A request may carry several tenants, so one
    tenant's mistake or full plan must not refuse another's data in the same request.
- **Identification, reads**: the `X-Tenant-Id` header. A query carries no payload to put a tenant in.
  - `SIGNY_MISSING_TENANT` (unset by default): tenant applied to **reads** without a header. Unset,
    such requests get 400 — the single-tenant opt-in for deployments with no gateway minting the header.
    A **blank** header is rejected either way so client bugs are not silently routed to another tenant.
    Writes have no fallback: a default there would pool every misconfigured exporter into one tenant.
- **Isolation point**: The tenant is **not** a storage-path partitioning axis, but a sort and index key
  inside each part. One part object contains all tenants, rows are sorted by `(tenant, timestamp_ns)`,
  and row groups never cross tenant boundaries. The tenant index in `meta.json` contains each tenant's row
  group ranges and min/max timestamps, and every read path **cannot address** a row group outside that range.
  The previous design that used tenants as a path axis was discarded because of R2 Class A costs — see
  [`docs/MULTI_TENANCY_DESIGN.md`](MULTI_TENANCY_DESIGN.md) for the cost model and rationale.
- **Retention**: Retention is a plan property and varies by tenant. The control plane pushes one tenant at
  a time, and signy returns success only after storing it in object storage. It applies at **deletion**
  time, not write time, so plan upgrades and downgrades affect already-recorded data. A tenant never pushed
  is **policy-unknown, and unknown means retain**. Tenant deletion is retention `0`. See
  [`docs/RETENTION_DESIGN.md`](RETENTION_DESIGN.md) for details.
- **Quota targets**: storage capacity and concurrent query count.
  There are no per-tenant rates — how fast the instance accepts work is the global backpressure gate's
  question, answered from the server's own state, and that gate still answers `429`. Queries over
  their concurrency limit answer `429` or `422`. The **storage** quota answers neither: it is a
  property of the tenant rather than of the instance, so an export over it is dropped and counted like
  any other tenant refusal.
  A backpressure refusal carries `Retry-After`, which is what makes "the client holds its data
  because the server declined it" true: collecty stops the segment there and offers it again. A
  *limit* violation is the opposite instruction — permanent for that record — and is dropped and
  counted rather than answered, since resending the identical bytes produces the identical refusal.
- **Observability**: rejection counters are on `/metrics` without tenant labels (a label per tenant
  multiplies every series by the tenant count); per-tenant numbers are the admin usage endpoint's.
  `signy_ingest_dropped_resources_total{reason=...}` is the one counter that has to be watched rather
  than merely reported: it is the only signal a dropped export produces, since the sender was told
  its body arrived.
- **Storage limit**: `max_stored_bytes` is pushed alongside retention and bounds the bytes a tenant may
  keep. Charged on the tenant's own extents in the shared objects — logs, traces and metric parts — read from `meta.json`
  rather than from the local files, so it does not move as the cache evicts and restores bodies. Over the
  limit, that tenant's resources are dropped; nothing is deleted to make room, because the space comes back
  when retention retires the oldest parts and choosing which of a customer's logs to destroy is not this
  engine's call.
- **Current state**: Identification, validation, isolation, per-tenant retention, and the stock quotas
  (stored bytes, query concurrency) are implemented. `tenant.id` is read off the resource of every
  collected export and recorded in WAL records (the owner survives restart), while MemTable, part, trace
  part, query, and catalog reads all require a tenant argument. Only `/metrics` retains a process-wide
  operator aggregation. **Durable usage accounting and tier partitioning** remain, along with adaptive
  (resource-pressure-based) throttling in place of the removed per-tenant rates.

## Data model

**A log is `tenant + timestamp + line + attributes`. There is no stream.**
Logs and spans are unified as wide events consisting of "timestamp + field set"
(following the OTel data model). Every attribute — resource and record alike —
lives in the row's own structured metadata under a normalized key
(`service.name` → `service_name`); nothing is promoted to a label, and no
label-set grouping exists anywhere in storage. An `attr=k=v` filter is a
filter over those attributes, accelerated by per-row-group exact-field blooms
and the `_sm:` columns rather than by a stream index. High-cardinality
attributes (`user_id`, `trace_id`) are ordinary attributes — the cardinality
problem the stream model had does not exist here.

## Write path

```
app ──OTLP/HTTP──▶ collecty ──POST /signy/api/v1/collect──▶ signy
                   (disk queue)                              │
                                                             ▼
         Journal append (sequential write, group commit: N MB or T ms, whichever comes first)
            │ Batch ack after fsync
            ▼
         MemTable (Arrow RecordBatch, immediately queryable)
            │ Size/time-based flush (deferred independently of ack)
            ▼
         Part creation (immutable): Parquet + sidecars
            │
            ▼
         S3 upload → catalog commit → journal truncate
```

- Crash recovery = journal replay. Part size is independent of ingest speed.
- **Ingest is collecty's.** One route takes writes, and only a collecty is expected to call it.
  An engine an application can push to directly has no queue in front of it, so a refusal it gives
  is telemetry lost unless the application happens to hold it; collecty's append-only disk queue is
  what makes a refusal survivable, and it only helps when nothing can go around it. So the OTLP push
  routes and the gRPC services were removed rather than left as a second way in.
  A batch is one signal's exports back to back inside one zstd stream, each behind its length. The
  server reads it a record at a time and never holds the batch, which is why the route carries no
  body limit — how large a batch may be is collecty's decision, made on how long it is willing to
  wait for an answer. The answer is `{"stored":n,"rejected":m}`: `stored` is the last segment held
  whole, which is what a collecty may unlink, and `rejected` is datapoints a metrics record lost to
  the active-series cap.
- Background merge: small parts → large parts (LSM-style). Daily time partitions.
- Sort out-of-order timestamps during merge. The allowed window for late data outside partition boundaries is configurable.

## Part structure

A part is one immutable directory:

- `data.parquet` — Schema built from fields actually present in the part (dynamic per-part schema):
  `_tenant`, `timestamp_ns`, `_msg`, one `_sm:<key>` column per attribute key up to a cap of 128
  chosen by row count, and a residual JSON column for the rare keys past the cap — null for every row
  whose keys all made columns, which is every row the intended consumer sends. Rows are sorted
  `(tenant, timestamp)` and row groups never straddle a tenant.
- `index.bin` — Per-row-group trigram blooms over `_msg` (pruning `|=`/`|~`) and per-window
  exact-field blooms over attribute pairs (pruning `{k="v"}` selectors and `| field="v"` filters).
- Metadata file — Time range, row count, per-tenant segments, column census, and CRC32 for each part
  file. Metadata itself is also checked with CRC32; inconsistent parts are not loaded.

Bloom filters are only for pruning; the final decision always comes from scanning blocks, so the scan guarantees correctness.

The format is unversioned and carries no compatibility code: nothing detects or reads parts written
before the streamless model (the `_stream`-ordinal era). No deployment ever stored data in that
format, so a directory holding such parts is simply not this engine's data.

### What a part keeps in memory, and what it re-reads

Every structure above is durable in the part directory, which is what lets the
resident half of it be a *cache* rather than a commitment. Three mechanisms
younger than the rest of this document:

- **Sidecars are evictable.** The blooms — the megabytes — live under one
  process-wide LRU byte budget (`sidecar_cache_max_bytes`, 10% of the declared
  budget) and are re-read from `index.bin` on the next pruning query. Answer
  equality under eviction is pinned by a test that forces every open to evict
  every other part. Before this the sidecars grew with the part count and
  nothing evicted them, which is what ended the first day-long soak at
  630 MiB of sidecar in a 2 GiB container.
- **Decoded row groups are cached, and so are narrow-pass outcomes.**
  `part/group_cache.rs` keeps decoded groups for reuse across scans under
  `row_group_cache_max_bytes` (12.5% of the budget), keyed so that a selection is
  served by slicing what is already decoded. Beside it, the narrow pass — the
  per-row evaluation for rows a group-level filter could not exclude — memoizes
  its outcome per (immutable part, group, predicate), so a repeated rare query
  pays it mostly on groups holding nothing. Both are bounded and both are
  droppable: the cost of a miss is a re-read, never a wrong answer.
- **Row groups are decode units, and that sets their size.** A group must be
  decoded to read any of it, so larger groups mean more wasted decode per match:
  measured at 1.5 M rows and a 100-row answer, 8× larger groups cost **3.75×**
  the time (`benches/scan_scaling.rs`). The 8192-row default is on the right side
  of that curve, and the ceiling is 65 536 rows regardless — a group's bloom
  windows are 1024 rows each and the selection mask is a `u64`.

## Read path

Flat-filter parsing (`query/params.rs`) → plan → pruning in this order:

1. Time range → partition/part selection (catalog + part metadata)
2. `attr` equalities and structured-field filters → row-group and window pruning with exact-field blooms
3. Line filters (`contains`, `regex`) → row-group pruning with trigram blooms
4. Scan only remaining row groups (MemTable + local parts; a part evicted from the local cache is
   restored whole from object storage first — range reads were measured and decided against, `todo.md`)

- This engine's differentiator is accelerating Loki's slow `| json | field="x"` pattern by push-down
  bloom pruning when the field was columnized at ingest. Push-down is part of the planner design from the start.
  **Measured against Loki and VictoriaLogs since**, on an equal container limit and the same corpus:
  [`COMPARISON.md`](COMPARISON.md) at 150 k rows and
  [`COMPARISON_LARGE_CORPUS.md`](COMPARISON_LARGE_CORPUS.md) at 1.5 M, both generated by `compare/run.sh`
  with a row-equality gate that withholds any timing whose answers disagreed. The claim and what would
  falsify it are stated in [`VISION.md`](VISION.md), which also scopes what the ten-times run changed.
- DataFusion was considered as the execution engine and is **rejected** — see "Decided choices" above.
  Execution is a merge of per-part sorted iterators feeding a bounded top-K heap, so that a query's memory
  is `limit` rows plus the heap rather than everything the window matched. **It is that now**: the log path
  passes the request's own `limit` into the scan alongside a scan-row budget, a scanned-byte ceiling and a
  memory reservation (`query/execution.rs`, `LogScan::new(..., limit, ...)`) — the `usize::MAX` this
  document used to name as invariant III's worst violation is gone from it. One `usize::MAX` remains and is
  a different thing: a histogram has no `limit` to stop at, since every matching row in the window
  contributes to a bucket, so the counting scan is bounded by `max_query_scan_rows`,
  `max_query_scan_bytes` and `max_histogram_buckets` instead of by a row count the client chose.

## Object storage catalog

- Upload immutable part objects before publishing one global catalog commit. The commit is an append-only object with two fixed replicas and an embedded digest; periodic snapshots accelerate startup replay. Conditional object creation fences competing writers, and R2 Bucket Lock on `catalog/` prevents deletion or overwrite of history. The S3 bucket-versioning API is not required.
- The local disk is a part cache. Keep small metadata/bloom catalog files and LRU-evict only `data.parquet` bodies. Queries and merges download only bodies selected by time and label pruning into a verified temporary directory and pin them against eviction while reading. Parquet range-read optimization is deferred.
- Hardware replacement (graceful shutdown): 1) on SIGTERM, immediately block the ingest endpoint (reject new requests); 2) drain until accepted in-flight requests finish WAL append/ack; 3) force-flush the MemTable accumulated by then and verify S3 upload and catalog commit completion; 4) terminate the process, discard the disk, and switch hardware. Data Alloy tried to send after blocking is retried from Alloy's own buffer because it receives no ack, then reaches the replacement when it resumes the same endpoint. A narrow disconnect just before ack during drain can cause duplicates (same nature as at-least-once above), but not loss.

### Object-store settings

 - `SIGNY_OBJECT_STORE_URL`: Format `s3://bucket/prefix`. Development and tests may use single-process `file:///absolute/path`. `file://` catalog updates use in-process serialization and atomic rename and do not provide multi-writer conditional creation. Unset means local-only mode.
 - S3 credentials, region, endpoint, and path-style options use the AWS/OBJECT_STORE environment variables read by `object_store`.
 - `SIGNY_CACHE_MAX_BYTES`: Local Parquet-body cache limit (10 GiB by default; small catalog files excluded). Evict the least recently accessed bodies first and download them again from the catalog for later queries.
 - On startup, recover the append-only catalog first. Replay from the newest valid snapshot, require contiguous generations and matching parent digests, and refuse a legacy mutable manifest layout. A missing or invalid one of the two fixed replicas is recovered from the other; divergent valid replicas and gaps fail closed before workers start. Leave a durable marker before upload so work interrupted before catalog publication can be validated and resumed on the next startup. A part with only a complete remote object set and no marker is considered an inactive generation and is not resurrected. Local directories outside the registry are not deleted automatically and remain for later retention.

### Ingest input limits

All limits apply **before** journal append, so rejected requests leave no trace in the WAL.
Protecting the engine takes priority over preserving every log line — Alloy retries or drops rejected batches from its own WAL.

- The OTLP body limit is a 16 MiB constant, matched across both transports.
- The collect route has no limit on the segment it is sent. It decompresses and
  ingests a record at a time and never holds more than one, so the 16 MiB
  constant applies to a record's payload and nothing bounds the segment around
  it. How large a segment is, is collecty's own question.

**What reading a record at a time cost.** The route used to merge every record
of a signal in a request into one export and write one WAL record for it. One
WAL record per collected record instead means one zstd frame each, and exports
from one service repeat their resource attributes, their service name and their
line shapes — which a merged payload compresses across and separate ones do not.
Measured on synthetic checkout-service exports at zstd level 1, WAL bytes for the
same data:

| records per export | exports per batch | merged | per record |
|---|---|---|---|
| 512 | 1024 | 2.77 MB | 3.82 MB (1.38×) |
| 100 | 1024 | 0.58 MB | 1.09 MB (1.87×) |
| 10 | 1024 | 57 KB | 479 KB (8.4×) |
| 1 | 1024 | 7.4 KB | 243 KB (33×) |

A collector SDK batches, so the realistic rows are the top two: about 1.4× the
WAL write amplification for a normal export size, and a cliff only for a client
that exports one record at a time. It is amplification on a buffer the flush
loop retires about once a second, not on stored bytes — parts are written from
the memtable and are unchanged.

Merging could come back in the journal writer, which already groups a batch into
one write, by concatenating same-tenant same-kind payloads before compressing.
That would mean moving compression from the ingest tasks into the single writer
task, where it would serialize instead of scaling with connections — a trade
worth making only if this amplification shows up as a real cost.
- `SIGNY_MAX_LINE_BYTES` (256 KiB by default)
- `SIGNY_MAX_TIMESTAMP_AGE` (7d by default) and `SIGNY_MAX_TIMESTAMP_SKEW` (1h by default): Acceptance window relative to the server clock. Disable with `off` when bulk-loading historical data. Because partitions are UTC-day based, clock errors or unit mistakes (sending seconds/milliseconds as nanoseconds) multiply partitions; in particular, **a future-date part never reaches the retention cutoff.**

- `SIGNY_MISSING_TENANT`: tenant identification for **headerless reads** (see "Multi-tenancy" above). Writes are unaffected — their tenant is the `tenant.id` resource attribute and has no fallback. Tenant-id validation applies before journal append either way, like the other limits.

### Retention settings

Retention is **per tenant only**. The
`PUT/GET/DELETE /signy/api/v1/admin/tenants/{tenant}/retention` routes (plus
`GET …/admin/tenants`) are always mounted — with no authentication of their own, per the trust
boundary above — and pushed policies are the sole authority, including over which tenants are served
at all: the pushed policies are the tenant registry, a push onboards a tenant and a delete offboards
it, with no restart. Use with
`SIGNY_RETENTION_REWRITE_THRESHOLD` (0.5 by default — expired-row fraction at which a part is rewritten).
There is **no** setting that caps pushed values. An instance-side cap would not reach unknown tenants,
causing a tenant explicitly marked "retain forever" to retain less data than a tenant with no policy
(`RETENTION_DESIGN.md`, "Rejected: an instance-side maximum").

Policies are stored as one object per tenant (`tenant_policies/<tenant>.json`) and return `200` only
after storage completes. Failure returns `503` and the control plane retries. All policies are loaded at
startup, and failure is fatal — the same severity as being unable to read the manifest.

Policies are **not** on the ingest or query hot path. Writes do not look them up at all, and reads leave
the range of a tenant without a policy unchanged (fail open).


## Query API

The read surface is first-party ([`QUERY_API.md`](QUERY_API.md) is the
authoritative reference, pinned by a test): GET + flat URL filters in
(`attr=level=error`, `contains=timeout`, `parse=json`, `start=-1h`), NDJSON
out, under `/signy/api/v1/`. Endpoints: `/logs` (search),
`/logs/histogram` (bucketed counts over the counting sink),
`/logs/attributes[/{key}/values]` (bounded autocomplete), `/logs/tail`
(chunked NDJSON streaming), `/logs/delete` (deletion requests, persisting the
same flat form). The trace routes are `/traces` (search), `/traces/{trace_id}`
and its autocomplete pair; the metric routes (M14) are `/metrics/query`,
`/metrics/instant`, `/metrics/quantile` and four discovery routes, with one
function and one aggregation per request and no PromQL. LogQL and the
Loki/Tempo compatibility surfaces were removed with the read-path decision
(issue #3); an unknown parameter or route is refused with a message that
teaches the accepted set.

## Milestones

| Stage | Scope | Completion criteria |
|---|---|---|
| M0 | axum + Loki push ingest + journal (group commit, ack) + MemTable + LogQL matchers/line filters | Query logs through the Grafana Loki data source |
| M1 | Part flush (Parquet + trigram bloom + stream index) + journal recovery + unified MemTable/part queries + merge | Data survives restart and pruning is verified |
| M2 | object_store S3 upload + append-only catalog (conditional create) + disk-cache eviction | Queries succeed from S3 after cache deletion |
| M3 | LogQL expansion: json/logfmt parsers, metric queries, field-filter push-down | Real dashboards run |
| M4 | Traces: OTLP ingest + trace_id lookup (bloom) + Tempo API | Query traces through the Grafana Tempo data source |
| M5 | Merge/compaction tuning, retention, resource limits (query memory/range limits), load tests | Target throughput is achieved |
| M6 | Graceful-shutdown hardware replacement (SIGTERM handler + force-flush + drain-status readiness) | Hardware replacement rehearsal succeeds with traffic moved to new hardware without loss |
| M7 | Local S3 load validation (Tier B: in-process latency/fault-injection store / Tier C: local MinIO real S3 protocol) + stronger load-analysis gauge observability | Throughput, latency, memory, retention, and error rate are validated against targets; bottlenecks are documented; manifest CAS, remote restore, and retention GC are verified on MinIO |
| M8 | **The ruler.** Retire the untrustworthy numbers, add criterion microbenchmarks for the hot paths, rewrite the load harness (N connections, intended-send-time latency, realistic corpus, concurrent reads), add CI | A performance regression is detected by a test rather than by reading prose |
| M9 | **The comparison bed.** Loki beside signy at an equal container memory limit, same corpus, four query shapes plus ingest, disk-per-GB and object-store operation counts | The claim in [`VISION.md`](VISION.md) is either supported by a published table or abandoned |
| M10 | **Declared memory budget** ([`VISION.md`](VISION.md) I). One budget knob, arena accounting, honest memtable metering, sidecars inside the budget, admission by budget rather than by slot | The engine runs a sustained mixed load under a declared budget and a test asserts peak RSS stays under it |
| M11 | **Bounded copies and deep pruning** ([`VISION.md`](VISION.md) II, III). `Arc<Labels>` end to end, the two free memcpys removed, single sort and single parse; streaming top-K execution, projection pushdown, cached Parquet footers, regex literal extraction | The M8 benchmarks move, and M9's table is regenerated against the same Loki build |
| M12 | **The first-party read path** (issue #3, [`M12_IMPLEMENTATION_PLAN.md`](M12_IMPLEMENTATION_PLAN.md)). Flat-filter GET + NDJSON log query API (`/logs`, histogram, attributes, tail, delete); Loki and Tempo surfaces removed; comparison bed ported; `QUERY_API.md` pinned by tests | Every read goes through the first-party API, no `/loki` or Tempo route remains, and the reduced digests still agree on a bed smoke run |
| M13 | **Trace read path** (issue #7). First-party trace query API (search, trace-by-id, autocomplete — same parameter grammar, plus the reserved duration comparison); trace scans joined the shared memory account; ended the trace read gap opened by the M12 Tempo removal | Traces are queryable again through the first-party API |
| M14 | **The metrics engine** (issue #8, [`M14_IMPLEMENTATION_PLAN.md`](M14_IMPLEMENTATION_PLAN.md)). OTLP metrics as the third signal: all five types decomposed to float series, Gorilla encoding, the series index with byte-budget admission and idle eviction (the optional process-wide `max_active_series` is an emergency guard), metric parts + compactor on the shared lifecycle, first-party range/instant/quantile/discovery API (no PromQL), a comparison bed against VictoriaMetrics with a series-churn phase, and a metrics memory-gate scenario | The claim in [`VISION.md`](VISION.md) is either supported by a published `COMPARISON_METRICS.md` or abandoned — published either way |

## References

- VictoriaLogs `lib/logstorage` (Go, Apache 2.0 — design reference): part/block structure, type-detecting encoding, bloom tokenizer, indexdb
- VictoriaTraces: precedent for traces on the same storage
- Quickwit: search architecture on object storage
- ClickHouse `ngrambf_v1`, Google Code Search: trigram indexing techniques

## Main crates

`tokio`, `axum`, `tonic`/`prost` (OTLP), `arrow`/`parquet`, `object_store`, `roaring`, `opentelemetry-proto`
