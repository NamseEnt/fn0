# Multi-tenancy design

Design record for making signy multi-tenant. Written to be self-contained:
a fresh context should be able to start implementing from this document alone.

Status: **accepted; steps 1, 2 (partly), and 4 are implemented, plus per-tenant
retention end to end ([`RETENTION_DESIGN.md`](RETENTION_DESIGN.md)).** See
[Implementation status](#implementation-status) for what actually landed.
Supersedes the "Multi-tenancy" section of
`ARCHITECTURE.md` (lines 48-65), which specified tenant-as-path-axis. That
approach is rejected here on cost grounds — see [Why not tenant-per-path](#why-not-tenant-per-path).

## Why this exists

signy is the log and trace backend for **fn0** (`~/fn0`), which sells a
**$1/month plan**. The engine must serve every project on that plan without
losing money, so the cost model is a hard design constraint rather than an
afterthought.

Requirements:

1. Per-tenant monthly quotas and usage limits.
2. A tenant must never see another tenant's data.
3. Total cost per project must fit the budget derived below.

## Budget

From `~/fn0/docs/cloud/dollar-plan.md`:

| Item | Value |
|---|---|
| Revenue after Paddle + domain | **$0.35–0.80 / subscriber / month** (infra budget) |
| Current expected total, all resources | $0.06–0.16 / user |
| Metrics line (500 active series, self-hosted VictoriaMetrics, 2-replica HA) | **$0.008 / project / month** — stated as "1.0–2.4% of the infra budget" |
| Grafana Cloud at $3.25/project | rejected, "~390x more" |

The metrics line is the governing precedent: **one observability axis is
expected to cost about $0.01/project/month.** Logs and traces are held to the
same bar.

**$0.01/project/month = 2,222 R2 Class A operations/month.**

### R2 unit costs

| Resource | Price |
|---|---|
| Storage | $0.015 / GB-month |
| Class A (PUT, LIST) | **$4.50 / million** |
| Class B (GET, HEAD) | $0.36 / million |
| Egress | free |
| DELETE | free |
| Free tier | 10 GB storage, 1M Class A, 10M Class B per month |

Verify against current Cloudflare pricing before committing numbers to
public copy; these match `dollar-plan.md` as of 2026-07.

## Cost model of the current engine

The dominant cost is **Class A operations**, and their count is set by the
flush cadence, not by data volume.

Per flush with an object store configured:

| Operation | Code | Class |
|---|---|---|
| Upload 3 part files (`data.parquet`, `index.bin`, `meta.json`) | `object_storage/cache.rs:2` `upload_part` over `PART_FILES` (`object_storage/mod.rs:19`) | **A** × 3 |
| Manifest read for CAS | `object_storage/object_io.rs:304` `load_manifest_versioned` | B × 1 |
| Manifest write | `object_storage/object_io.rs:298` `publish` | **A** × 1 |
| Trace manifest read (**happens even with zero traces**) | `flush.rs:353` → `object_storage/catalog.rs:265` | B × 1 |
| Delete superseded objects (merge, retention) | `delete_part_objects` | free on R2 |

The flush trigger (`flush.rs:87-96`):

```rust
if memtable.is_empty() && trace_memtable.is_empty() { continue; }
if (size as u64) < flush_max_bytes && elapsed < flush_max_interval { continue; }
```

Defaults are `flush_max_bytes = 1 MiB`, `flush_max_interval = 5s`
(`config.rs:85-86`). Any tenant with continuous trickle traffic therefore
flushes every 5 seconds — **518,400 flush cycles per month**, independent of
how little data it sends.

### Why not tenant-per-path

If the tenant becomes a storage path axis, every tenant writes its own objects.
Even after consolidating the four sidecars into one object, and counting zero
manifest and zero merge overhead:

| | Value |
|---|---|
| Class A budget | 2,222 / month |
| Objects per flush per tenant | 1 (best case) |
| **Maximum flush frequency** | **once per 19.4 minutes** |

With realistic manifest and merge overhead this lands at an RPO of 45-60
minutes. Worse, a small tenant buffering 20 minutes of logs produces
multi-kilobyte parts, which compress poorly and pile up merge debt.

At the current 5s default the same layout costs **$9.33/project/month** —
about 9x the plan's entire revenue.

**Conclusion: per-tenant objects do not fit this price point at any usable
RPO. Objects must be shared across tenants.**

Note that flush cadence does **not** affect query freshness: the memtable is
part of the unified query path, so ingested data is visible immediately. A
longer flush interval only widens the window for *simultaneous loss of server
and disk*, a risk `ARCHITECTURE.md:13` already accepts deliberately. The WAL
covers process crashes with zero loss, and M6 graceful shutdown covers planned
machine replacement.

## Accepted design: shared parts with tenant-aligned segments

One part object holds **all tenants**. The tenant is a sort key and index axis
*inside* the object, not a path component.

### Layout

- Rows sorted by **`(tenant_id, timestamp_ns)`**.
  Currently `part/format.rs:20` and `:68` sort by `timestamp_ns` alone.
- **Row groups aligned to tenant boundaries** — a row group never spans two
  tenants (split at the boundary).
- `meta.json` gains a per-tenant index:
  `tenant -> { row_group_range, min_ts_ns, max_ts_ns, row_count, byte_range }`.
- Bloom and stream index stay per-row-group. **No change** to
  `part/indexes.rs` or the bloom sidecar format.

### Why the sort key change is what makes this work

An earlier analysis rejected tenant-as-label because pruning fails. The reason
was specific and fixable: the stream index (`part/indexes.rs:20-41`) is a
row-group-granular posting list, and with rows sorted by timestamp alone, every
active tenant appears in every row group. The tenant → row-group bitmap becomes
all-ones and prunes nothing.

Sorting by `(tenant, ts)` removes that. Within a tenant's segment rows remain
timestamp-ordered, so the early-termination logic in `part_registry.rs:307-310`
and `memtable.rs:47-51` keeps working unchanged. Only the part-level
`min_ts_ns`/`max_ts_ns` pruning has to become per-tenant, which the new meta
index provides.

### What this buys

| Requirement | How it is met |
|---|---|
| Isolation | A query cannot **address** row groups outside its tenant's range. Same fail-closed property as separate files. |
| Time pruning | Timestamp order preserved inside each tenant segment. |
| Bloom / stream index | Row-group granular; unchanged. |
| Transfer on cache miss | Range GET of the tenant's `byte_range` only. |
| **Class A** | **One object per flush for all tenants combined.** |

Cost per project becomes **inversely proportional to project count** — the
right shape for a platform.

### Retention: partition by tier, not by tenant

> **Superseded.** This section records the reasoning as it stood when the
> document was written. Partitioning on `(tier, day)` was rejected because it
> fixes retention at write time, so a plan upgrade or downgrade never reaches
> data that is already on disk. Per-tenant retention is instead decided from
> the `meta.json` tenant index at **deletion** time; see
> [`RETENTION_DESIGN.md`](RETENTION_DESIGN.md). Partitions stay on `day`.

Shared objects cannot express per-tenant retention. fn0 does not need it:
retention is a **plan attribute** ($1 plan, free plan), not a per-project
setting.

Partition on `(tier, day)`. Whole-object deletion works as it does today
(`retention.rs:88` filters on `max_ts_ns < cutoff` and removes directories),
and R2 charges nothing for DELETE.

Individual account deletion, in order of preference:

1. **Wait for retention expiry.** Document as "logs purged within N days of
   account deletion." Zero engineering. Standard industry practice.
2. **Lazy purge at merge.** Merge already rewrites parts; drop the deleted
   tenant's rows at that point. No extra I/O, but latency until the next merge.

*Resolved:* deleting a tenant is pushing `retention: "0"`, which does both — the
data is invisible from the next query, parts holding only that tenant are
deleted whole, and shared parts drop its rows at the next merge, ignoring the
rewrite threshold so reclamation is bounded. There is no completion report; see
[`RETENTION_DESIGN.md`](RETENTION_DESIGN.md).

## Read path

Planning is unaffected and remains **zero R2 requests**.

`remote_lifecycle.rs:35-42` returns early before touching the network:

```rust
let required = required_parts();
if missing_parts(&required).is_empty() {
    return Ok(read_guard);   // cache hit: zero R2 requests
}
```

`candidate_part_ids_with_exact_fields` (`part_registry.rs:107`) reads only
in-memory `PartReader`s backed by local catalog files, which are excluded from
LRU eviction (`ARCHITECTURE.md:123`). Total catalog size is roughly unchanged
by sharing, because bloom and stream index are row-group granular and the total
row count is the same either way.

Two conditions must hold or reads get much worse:

### Condition 1: range reads become mandatory

`download_part` (`object_storage/cache.rs:246`) currently does
`store.get(&path)` → `result.bytes()`, fetching the whole object.
`ARCHITECTURE.md:123` defers range reads to future work.

Against a shared part this would download **every tenant's data on every
miss** — a 200 MB part to serve a tenant's 200 KB is 1000x amplification.
Range GET using the per-tenant `byte_range` is a **prerequisite**, not an
optimization. R2 charges a range GET as one Class B, same as a full GET, and
egress is free, so this reduces bytes at no request cost.

> **Refuted by measurement, 2026-08-18** (`todo.md`, the struck "Add Parquet
> range reads" entry). The argument turns on "a range GET is one Class B, same
> as a full GET" — one fetch replacing one fetch. That is the step that is
> wrong. A whole restored body is read by **5.66 distinct tenants** before
> eviction takes it, and a selective fetch serves one tenant's slice, so the
> same work is **5.66 range GETs against one full GET**: requests ×6.7 to move
> ×0.37 the bytes. The amplification is real but it is not 1000x and it is not
> waste — the bytes another tenant's rows cost are the bytes that tenant's own
> query then does not pay for. Range reads are not a prerequisite here; they
> are a **loss** at the sharing this design is built on, and they get worse the
> better the sharing works.

### Condition 2: local cache layout is decoupled from remote layout

| | Purpose | Unit |
|---|---|---|
| Remote (R2) | minimize Class A | **shared** object |
| Local (disk cache) | query locality, isolation | **per-tenant** slice |

Store each range-fetched slice under a `(part_id, tenant)` key. The local cache
then behaves exactly as it does today, and `evict_cache` LRU works per slice.

Caching whole shared objects locally is the failure mode to avoid — it wastes
cache space and couples tenants.

### Read-side improvements

- Consolidating sidecars. **Done, partly.** The trigram blooms and the stream
  index are one `index.bin`, so a part is three files rather than four: a
  catalog restore is 2 GETs instead of 3 and a full restore is 3 instead of 4.
  Folding `meta.json` in as well would make it 2, but `meta.json` is also the
  self-describing version header that the registry and the recovery paths read
  standalone, so that is a separate change with its own migration question.
- Fetching only what a miss is actually missing. **Done.** Eviction removes the
  Parquet body and leaves `index.bin` and `meta.json` behind, so a body-only
  restore has the catalog on disk already, checksummed against the manifest
  descriptor it is restoring for. `download_part` used to refetch all three
  files anyway and `remove_dir_all` the healthy catalog to make room for the
  copy. It now GETs the body alone and renames that one file into place, so a
  cache miss costs **one Class B instead of three** and the merge tombstone
  beside the catalog is never rewritten. A part whose sidecar no longer matches
  its descriptor still falls back to fetching everything, which is the only way
  to obtain a catalog that does match. Pinned by
  `restoring_an_evicted_body_costs_one_get` and
  `restoring_a_part_with_a_damaged_catalog_fetches_every_file`.
- Merge concatenates each tenant's segments across parts, making a tenant's
  data **more contiguous** in merged parts. One range GET then retrieves more
  useful data. Merge helps more here than under per-tenant parts.

### Known read-side overhead

Tenant-aligned row groups mean a tenant with 50 rows still occupies its own row
group. With 1,000 active tenants a part may contain 1,000+ row groups, and
Parquet footer metadata (a few hundred bytes per column chunk) can reach 1-2 MB.

Assessment, now measured:

- **Compression loss is negligible in absolute terms.** It lands only where
  byte counts are tiny; large tenants still get full-size row groups.
- **Footer size: measured, and pooling is not needed.** 5,000 rows across 500
  tenants come to 691 KB against 28 KB for one tenant — 24.7x, about 1.3 KB per
  row group ([`LOAD_RESULTS.md`](LOAD_RESULTS.md) §2). But the ratio is a
  function of rows-per-tenant-per-part, not of tenant count, and merge is what
  bounds that: it cuts (tenant, part) pairs 3.6x and parts-per-tenant from 6.6
  to 1.85, taking the ratio to about **1.07x** (§6). At 10,099 parts the
  resident sidecar cost is 18.7 MB, not the 407 MB first extrapolated, because
  the parts that fragment that badly are also tiny (§8).
- **The premise this was watching for turned out to be inverted.** At this scale
  the sidecars are single-digit megabytes while one merge transiently holds
  hundreds, so the binding memory constraint is the merge budget, not
  fragmentation.

So the fallback — pooling tenants below ~512 rows into a shared row group,
weakening isolation from "cannot address" to "row-range filter" — is **not
being built.** It was conditional on a measurement demanding it, and the
measurement does not.

### What is left of the read path

Two of the four read-path items are done — tenant-aligned row groups with the
segment index, and the sidecar consolidation above. The other two are one piece
of work, not two:

1. **Range GET on restore**, using a tenant's byte range instead of the whole
   object.
2. **`(part, tenant)` local cache keys**, so what lands on disk is the slice
   rather than the shared object.

They are one piece because a range of a Parquet file is not a Parquet file. The
choice is between reconstructing a valid single-tenant file locally from the
fetched byte range plus a rewritten footer, and reading ranges straight from the
object store through an async Parquet reader with no local file at all. The
second is what arrow-rs is built for and it is the smaller amount of code — but
it adds a second way to reach a tenant's rows, and this engine has one scan
precisely so that the retention clamp, the tenant scope and the deletion mask
cannot be applied in one place and forgotten in the other.

~~**Not decided here.** It needs the byte ranges recorded in `TenantSegment` at
write time either way, and it should be decided against a measurement of how
often a shared-part miss actually happens under a real workload — which is a
number no run has produced yet, because every load run so far has been
ingest-dominated.~~

**Decided 2026-08-18: neither, and the deciding measurement was not the one
asked for above.** How often a shared-part miss happens is a deployment
property and still unmeasured; it turned out not to be needed, because the
*sign* of the work is negative whatever the rate. See the refutation under
Condition 1 and the struck entry in `todo.md`. Per-tenant cache keys go with
it: they need the range read to have a slice to key, and the sharing they would
split is what pays for the download.

### What is recorded at write time anyway

The write half landed from the other line of work before that decision was
taken, so each `TenantSegment` now carries:

- `bytes: ByteRange` — the half-open extent of that tenant's row groups in
  `data.parquet`, taken from the metadata `ArrowWriter::close` returns, because
  the writer is the only thing that knows the layout. Both writers record it,
  and `part::tests::the_streaming_writer_produces_the_same_part_as_the_batch_one`
  is what keeps them from disagreeing about where a row group is.
- `crc32` — of exactly that range, computed in the same single pass over the
  body that already produced `data_crc32`. Without it a fetched slice would be
  the one thing in this design that could not be checked.

Pinned by `part::tests::each_tenant_segment_records_the_byte_range_of_its_own_row_groups`,
which checks the recorded range against the Parquet footer that actually decides
the layout, against its own checksum, and against its neighbours.

**It is recorded and unused, and that is a cost, not a hedge.** Two fields per
(tenant, part) pair, paid on every part written, for a read path this document
decided against on measurement. Keeping it is a bet that the decision gets
revisited under a workload with a different miss rate; if that bet is not one
worth holding, the fields come out and the writers get simpler. Nothing reads
them today.

### The decode mechanism is proven; the cache lifecycle is not built

`part::tests::a_tenant_decodes_from_its_own_byte_range_and_asks_for_nothing_else`
decodes one tenant's rows from the footer plus that tenant's byte range alone,
and asserts that **every range the decoder asks for falls inside the segment**.
That second assertion is the load-bearing one: it is what makes fetching a
tenant's slice *sufficient*, and it is the same fail-closed property a
per-tenant file would have had. The rows match an ordinary whole-file read of
the same tenant.

What remains is not the decoder, it is the cache lifecycle around it, and it is
larger than it looks because **merge and retention still need whole parts**
(`merge/selection.rs` reads every tenant segment). So the cache has to hold two
kinds of entry — whole bodies for the writers, per-tenant slices for the readers
— and every part of the lifecycle has to know which it is looking at:

- `PartRegistry::missing_data_ids` becomes a question about `(part, tenant)`
  rather than about a part.
- `PartReader` needs a second backing: push buffers over a cached slice, beside
  today's `PreadReader` over a whole file.
- `restore_parts` becomes a range GET of the segment plus the footer, and
  `evict_cache` has to reclaim slices as well as bodies.
- `pin_remote_parts` has to carry the tenant, which it does not today.

Until all of that lands together, whole-part restore stays the only path.
Half-migrating it would leave the engine worse than it is now.


The reading half, if it is ever built, is decided and is pure code:
`parquet::arrow::push_decoder::ParquetPushDecoder` answers
`DecodeResult::NeedsData(Vec<Range<u64>>)` and the caller supplies the bytes, so
the scan stays synchronous inside `spawn_blocking` and the bytes can come from a
cached slice, a range GET, or a mix. Reconstructing a local single-tenant
Parquet file was rejected: a rewritten footer can be compared against nothing,
and an offset that is subtly wrong yields *wrong rows* rather than an error, in
an engine that checksums every file at every open.

**No format version came with any of this.** signy versions nothing on disk
— see the decided-choices table in [`ARCHITECTURE.md`](ARCHITECTURE.md). The
format changed; a stale data directory is deleted.

## Supporting changes

### Consolidate the sidecars into one object

**Half done.** The trigram blooms and the stream index are one `index.bin`, so
`PART_FILES` (`object_storage/mod.rs:19`) is three files rather than four: three
PUTs per flush instead of four, one fewer round trip per catalog restore, and
one fewer checksum pass per part at startup — which §8 of
[`LOAD_RESULTS.md`](LOAD_RESULTS.md) measured as the actual startup cost.

Folding `meta.json` in as well would make it two, but `meta.json` is the
self-describing header the registry and the recovery paths read standalone. It
is not versioned and nothing on disk is, so a format change is not an event to
be batched around: the `TenantSegment` byte ranges landed on their own, and
folding `meta.json` in can land on its own too.

Affected: `upload_part`, `download_part`, `restore_catalog`,
`delete_part_objects`, `garbage_collect_orphans`. All iterate the `PART_FILES` /
`CATALOG_FILES` constants, so the change is contained.

### Raise `flush_max_bytes` substantially

With all tenants in one memtable, the 1 MiB byte trigger (`config.rs:85`) fires
long before the interval, making the interval setting meaningless. Raise to
**64–256 MiB** so the interval is the only trigger.

The WAL protects against process crash, so a large memtable is an RPO concern
only, not a durability one. 256 MiB is comfortable on an OCI A1 (12 GB+).

### Tenant ID validation

The tenant ID arrives in the `tenant.id` resource attribute of an export, or in
the `X-Tenant-Id` header of a read, and flows into R2 object
keys and local filesystem paths. `is_safe_path_component`
(`object_storage/recovery.rs:439`) only rejects traversal, not hostile names.

Validate against a strict allowlist (`[a-zA-Z0-9_-]{1,64}`) **before journal
append**, consistent with the existing rule that all input limits apply before
the WAL (`ARCHITECTURE.md:135`).

## Cost result

Shared parts, 60s flush, consolidated sidecar:

| Line | Class A / month, platform-wide |
|---|---|
| Flush part objects | 43,200 |
| Manifest CAS PUT | 43,200 |
| Merge (write amplification ~4x, on large parts) | ~29,000 |
| **Total** | **~115,000 = $0.52 / month** |

| Projects | Class A cost / project |
|---|---|
| 100 | $0.005 |
| 1,000 | **$0.0005** |
| 10,000 | $0.00005 |

### Flush cadence is a platform-wide fixed cost

| Flush interval (RPO for total disk loss) | Platform-wide / month | At 100 projects |
|---|---|---|
| 5s | $4.67 | $0.047 |
| 15s | $1.56 | $0.016 |
| 30s | $0.78 | $0.008 |
| **60s** | **$0.52** | **$0.005** |
| 5 min | $0.10 | $0.001 |

Start at 60s and tighten as project count grows. Cadence can also be set per
tier (free 5 min, $1 plan 60s) since objects are already partitioned by tier.

## Proposed quotas

Sized bottom-up from what a real app emits, then priced — the same method
`dollar-plan.md` used for metrics.

### What a request generates

Anchor: the CPU pool is 500 CPU-minutes ≈ **2M SSR requests**; p50 usage is
1-5% of caps.

| Source | Per request | At cap (2M req) | p50 (50k req) |
|---|---|---|---|
| Platform access log (method, path, status, duration, trace_id) | ~300 B | 0.6 GB | 15 MB |
| User logs (~1 line per handler) | ~200 B | 0.4 GB | 10 MB |
| Traces (root + ~5 spans × 400 B) | ~2 KB | 4 GB | 100 MB |
| **Total** | **~2.5 KB** | **~5 GB** | **~125 MB** |

Maxing the CPU pool produces about **5 GB/month** of logs and traces. The
ingest quota is sized so it does not bind before the compute quota does.

### What actually costs money

Under shared parts, **ingest volume barely affects Class A**: flush object
count is set by cadence, and merge output count scales with part count, not
bytes. A heavy tenant makes parts *larger*, not *more numerous*.

| Axis | Cost source | Dominant variable |
|---|---|---|
| Ingest volume | R2 storage $0.015/GB-mo | **retention period** |
| Query volume | scan CPU + cache-miss Class B | **scanned bytes** |
| Stream cardinality | `stream.idx` is an eviction-exempt persistent catalog | active stream count |

### Recommended values ($1 plan, 7-day retention)

| Quota | Value | Worst | Expected |
|---|---|---|---|
| Log + trace ingest | **10 GB / month** | $0.0035 | ~$0.0001 |
| Retention | **7 days** | — | — |
| Query scan | **50 GB / month** (+2 GB / hour burst) | $0.019 | ~$0.0004 |
| Active log streams | **100 / project** | ~$0 (disk only) | ~$0 |
| Flush + merge Class A (shared across tenants) | — | $0.0005 | $0.0005 |
| **Total** | | **~$0.023** | **~$0.0015** |

Expected cost is **below the $0.008 metrics line**. Worst case is bounded and
barely moves the plan's existing $2.2/user worst-case total.

Free tier follows the documented 1/10 rule: **1 GB ingest, 5 GB scan, 3-day
retention, 20 streams** (~$0.002 worst case).

### Rationale

**Ingest 10 GB** — 2x headroom over the 5 GB a maxed CPU pool produces; covers
400k requests even for an app logging 5 lines each. Matches the existing
"Object storage 10 GB" line for table consistency.
Storage math: 10 GB × (7/30) = 2.33 GB retained, ~10x compression from Parquet
dictionary + zstd → 233 MB → **$0.0035**.

**Retention 7 days** — the single largest storage lever. At 30 days storage
becomes $0.015, twice the metrics line. Seven days is sufficient for
side-project debugging; **30-day retention is the natural upsell for the ~$5
tier** `dollar-plan.md` already anticipates.

**Query scan 50 GB** — a cap-using project retains 233 MB compressed, so this
is **214 full rescans per month (~7/day)**.
CPU: 50 GB at ~150 MB/s/core effective = 341 s = 0.095 OCPU-hr × $0.0138 =
$0.0013. Class B worst case (all misses, ~1 MB slices) = 50,000 GETs = $0.018,
the same order as the already-accepted "Object GET 100k → $0.036" line.

Scanned bytes is the right unit: the engine already measures it
(`metrics.rs:19` `query_scanned_bytes`, `config.rs:48` `max_query_scan_bytes`),
it matches Loki / CloudWatch / BigQuery convention, and unlike a query *count*
it actually bounds cost — a single query can scan everything.

**Active streams 100** — structurally identical to the metrics active-series
cap. `stream.idx` is excluded from the cache ceiling (`ARCHITECTURE.md:147`),
so **cardinality explosion becomes un-evictable disk**. A well-structured app's
`{service, env, level}` combinations number 10-30; 3x gives 100. Enforce with
the same keep-existing / drop-new semantics as `fn0/src/metric_gate.rs`, and
report drops back as `fn0.logs.dropped`.

This is the gap `PRODUCTION_READINESS_REVIEW.md:276` records as
`max_streams_per_user` — "missing — requires multi-tenancy".

## Implementation status

| Layer | State |
|---|---|
| Tenant identity | **done** — `tenant.rs` `TenantId` allowlist. A **write** names its tenant in the `tenant.id` OTLP resource attribute (`otlp_tenant.rs`), on both transports and all three signals; a **read** names it in the `X-Tenant-Id` header, with `SIGNY_MISSING_TENANT` as the single-tenant opt-in for a headerless one. Was a header on both until 2026-08-28: a collector that never decodes a payload cannot route on one, so the tenant moved inside the payload (issue #9) |
| One export, several tenants | **done** — resources are grouped by tenant and one journal record is written per group. The received bytes still ride through to the WAL untouched when there is one group and nothing was dropped, which is every export collecty forwards. The record/span/datapoint caps are counted over the whole request and before the split, so N groups cannot multiply them |
| Tenant refusal on a write | **a drop, not a refusal** — the status an ingest answers says whether the body arrived and nothing about whose it was. No tenant, an unparseable one, one not served, one at its storage limit: dropped, and counted in `signy_ingest_dropped_resources_total{reason=...}`. A request may carry several tenants, so one tenant's mistake must not refuse another's data in the same request. Reads still answer 400/403 — one read is one tenant |
| The tenant attribute in storage | **stripped** — removed during normalization rather than off the wire, so the WAL keeps the bytes that arrived and replay strips it the same way. Left in, the routing key would become a queryable label in every log entry's structured metadata and on every span's resource, a second and unenforced copy of the isolation the tenant column already provides |
| Journal | **done** — `LGY3` framed record carries the tenant; replay restores each record under its own tenant, and a pre-tenancy record falls back to the default tenant |
| MemTable | **done** — `HashMap<TenantId, HashMap<Labels, …>>`; every read method requires a tenant |
| Log part format | **done** — `_tenant` leading column, `(tenant, timestamp_ns)` sort, tenant-aligned row groups, per-tenant index in `meta.json` |
| Trace part format | **done** — same shape (`tenant` column, tenant-aligned row groups, tenant index) |
| Read path isolation | **done** — `PartReader`/`PartRegistry`/`TraceRegistry`/MemTable queries, `label_names`, `label_values`, `series`, `stats` all take a required tenant; part-level time pruning is per tenant |
| Merge | **done** — reads rows through the tenant index and re-sorts by `(tenant, ts)`; no cross-tenant mixing beyond the shared object |
| Per-tenant retention | **done** — the control plane pushes one tenant at a time to `PUT /signy/api/v1/admin/tenants/{tenant}/retention`, signy persists it before acknowledging, and it is applied at deletion time; partitions stay on `day`. See [`RETENTION_DESIGN.md`](RETENTION_DESIGN.md) |
| Partition on `(tier, day)` | **rejected** — retention baked in at write time cannot honour a plan change; superseded by [`RETENTION_DESIGN.md`](RETENTION_DESIGN.md) |
| Sidecar consolidation | **done for the sidecars** — the trigram blooms and the stream index are one `index.bin`, so `PART_FILES` is three. Folding `meta.json` in is deliberately left for the format change that adds `TenantSegment` byte ranges, so the on-disk format moves once rather than twice |
| Cache-miss fetch cost | **done** — a body-only restore GETs the body alone and leaves the local catalog in place: one Class B per miss instead of three |
| Range GET: recorded byte ranges | **done, and currently unused** — every `TenantSegment` carries its `ByteRange` in `data.parquet` and a CRC32 of exactly that range. The read half is decided against on measurement, so these two fields are a cost with no reader |
| Range GET: reading through them / per-`(part, tenant)` cache | **decided against, on measurement** — a restored body is read by several tenants before eviction, so serving them selectively trades Class A requests to save bytes R2 gives away. Design if ever revisited: the push decoder |
| Per-tenant rates (ingest, query scan) | **removed** — built, then taken out: how fast the instance accepts work is the global backpressure gate's question, answered from the server's own state, and adaptive resource-pressure throttling is the planned replacement. The stock quotas below stay |
| Stream-count quota | **removed with the stream concept** — storage keeps attributes per row and has no per-stream catalog, so the cost the quota guarded no longer exists |
| Per-tenant usage | **done** — `GET /signy/api/v1/admin/tenants/{tenant}/usage`. Deliberately *not* labels on `/metrics`: that scrape is unauthenticated and process-wide by design, and a label per tenant is the cardinality problem this engine bounds everywhere else |
| Durable monthly usage accounting | **out of scope, decided** — a month is spent across instances and outlives any of them, so the control plane holds it. An instance answers only for its own share, which is the endpoint above |

`/metrics` deliberately keeps process-wide gauges (`global_stats`): it is the
operator scrape, not a tenant-facing endpoint.

### Open questions

**Where does a tenant's tier come from?** *Answered* — the control plane pushes
one tenant's retention at a time, signy persists it before acknowledging,
and it is applied at **deletion** time rather than at write time. `Partition on (tier, day)` was rejected along with every
other scheme that fixes retention when the bytes are written, because none of
them honours a plan upgrade or downgrade. Partitions stay on `day`. Full record:
[`RETENTION_DESIGN.md`](RETENTION_DESIGN.md).

**Parquet footer size at high tenant counts** remains unmeasured — the open item
recorded under [Known read-side overhead](#known-read-side-overhead).

## Implementation checklist

Ordered by dependency.

### 1. Tenant identity

- [x] ~~Extract `X-Scope-OrgID` in the Loki push path and the OTLP gRPC metadata.~~ —
      superseded 2026-08-28: a write's tenant is the `tenant.id` resource
      attribute on both transports, and reads use `X-Tenant-Id`. See issue #9.
- [x] Strict allowlist validation before journal append.
- [x] Config for the missing-header policy (accept as default tenant vs reject).
- [x] Thread tenant through `Journal::append` / `append_trace` records.

### 2. Storage format

- [x] Add `tenant` to the row model; sort by `(tenant_id, timestamp_ns)` in
      `part/format.rs`.
- [x] Align row-group boundaries to tenant boundaries in `row_group_bounds`.
- [x] Add the per-tenant index to `meta.json` (`part/metadata.rs`).
- [x] ~~Partition on `(tier, day)` instead of `day`~~ — rejected; per-tenant
      retention is applied at deletion time instead ([`RETENTION_DESIGN.md`](RETENTION_DESIGN.md)).
- [x] Consolidate the sidecars — the trigram blooms and the stream index are one
      `index.bin`, so `PART_FILES` is three. `meta.json` stays separate on
      purpose: it is the self-describing version header the registry and the
      recovery paths read standalone, and folding it in belongs with the format
      change that adds `TenantSegment` byte ranges, so the on-disk format moves
      once rather than twice.

### 3. Read path

- [x] Per-tenant `min_ts`/`max_ts` pruning from the meta index.
- [x] Fetch only what a miss is missing — a body-only restore GETs the body and
      leaves the already-valid local catalog alone, one Class B instead of three.
- [x] Record each tenant's byte range and a CRC of it at write time. No format
      version came with it; signy versions nothing on disk.
- [x] ~~Range GET using those ranges in `download_part`, decoded through
      `ParquetPushDecoder`~~ — decided against on measurement.
- [x] ~~Key the local cache by `(part_id, tenant)`~~ — goes with the range read:
      it needs a slice to key, and the sharing it would split is what pays for
      the download.
- [x] Parallelize `restore_parts` and `restore_catalog` — both fan out at
      `RESTORE_CONCURRENCY` (16). The bound exists because unbounded fan-out
      opens a connection per part and turns a restore into a self-inflicted
      outage of the store it is reading.

### 4. Isolation surface

Every method below currently returns data across all tenants. **Make the
tenant a required argument enforced by the type system** — this is the whole
isolation boundary once files are no longer separate.

- [x] `PartRegistry`: `label_names`, `label_values`, `series`, `stats`
- [x] `MemTable`: `label_names`, `label_values`, `series`, `stats`
- [x] Tempo: `trace_by_id`, `search`, `search_tags`, `search_tag_values`

### 5. Quotas and limits

- [x] Active stream count cap — built as `max_streams`, later removed with the
      stream concept itself.
- [x] Per-tenant query concurrency — `max_concurrent_queries_per_tenant`
      (`tenant_quota.rs`). The global semaphores in `app_state.rs` stay: they
      are the process-wide memory bound, which is a different question from a
      tenant's share of it.
- [ ] Per-tenant cache budget (see risks below).
- [x] ~~Tenant labels on all quota and rejection counters in `metrics.rs`~~ —
      rejected. `/metrics` is an unauthenticated, process-wide scrape by design,
      and a label per tenant is the cardinality problem this engine bounds
      everywhere else. Per-tenant numbers are served from the admin usage
      endpoint instead.

### 6. Durable usage accounting

Monthly quotas need a counter that survives restart, and **it cannot be derived
from live parts** — retention deletes the evidence before the month ends.

- [ ] Store per-tenant monthly usage in the object store, CAS-updated like the
      manifest.
- [ ] Tie increments to `FlushTransaction` (`flush.rs:312-322`), which already
      provides a crash-consistent commit boundary keyed on the journal
      checkpoint. No new durability mechanism required.
- [ ] Restore in-memory counters at startup from the durable value plus
      unflushed memtable bytes.

## Risks and watch items

**Cache hit rate degrades as project count grows.** `cache_max_bytes` is a
single global setting (`config.rs:39`, default 10 GiB) and `evict_cache` is one
global LRU pass. At 1,000 projects × 233 MB retained = 233 GB against a 10 GiB
cache, the Class B "worst case" above becomes the expected case. Compare
$0.36/M Class B against OCI block volume at $0.025/GB-month: **growing the
cache is almost always cheaper than eating the misses.** Track this as an
operational scaling item.

**Cross-tenant cache interference.** One tenant's wide historical query evicts
everyone else's hot slices. Same class of problem as the global semaphores.
Needs a per-tenant cache budget.

**Cardinality explosion now pollutes shared parts.** Per-tenant ingest limits
move from "nice to have" to prerequisite.

**`is_empty()` skip stops helping.** With many tenants someone is always
sending, so flush runs every cycle. This is fine — cost is now proportional to
cadence, not tenant count, which makes it predictable.

**Trace sampling is the reserve lever.** The sizing above assumes 100% trace
collection, and traces are ~4x the volume of logs. If costs run over, head
sampling (always keep error traces, sample successful ones) adjusts spend
without touching user-facing quotas.

**Single writer is assumed throughout.** Confirmed out of scope — shared parts
would need rework for multiple writers. If bring-your-own-R2 ships later, route
those tenants through a separate per-tenant path instead.

## Related documents

- `ARCHITECTURE.md` — engine architecture; its "Multi-tenancy" section (48-65) is
  superseded by this document
- `PRODUCTION_READINESS_REVIEW.md` — P0-3 (119-145) is the production gate this
  design closes; line 276 records the missing `max_streams_per_user`
- `~/fn0/docs/cloud/dollar-plan.md` — plan economics, unit costs, quota method
- `~/fn0/docs/fn0/limits.md` — published quota table this must extend
