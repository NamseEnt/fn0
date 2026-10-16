# B-link Residency Audit

## Scope and result

This source audit records the B-link memory-residency constraint relevant to
the planned over-cache comparison. It makes no claim that an out-of-memory
condition was induced or that any memory-capacity experiment was run.

The current `BlinkTenantService` path loads the persisted B-link page set into
`BlinkState` when a shard opens, then publishes a page catalog over that set.
Reads resolve pages through the pinned in-memory generation. The current path
has no lazy disk page source, bounded page cache, or page eviction mechanism.
Consequently, a dataset larger than the host's effective memory cache cannot
be run as a comparable cache-resident versus over-cache throughput benchmark
with the current B-link implementation. An external process memory cap could
be placed on both backends, but it would not give B-link an evictable cache;
that setup would test whether its mandatory resident state fits. No OOM outcome
is asserted here.

## Service open and page residency

`BlinkTenantService` stores a map of open `AsyncBlinkShard` handles and accepts
a `LocalTenantServiceConfig`. Its constructor copies `database_config` and
`max_open_shards`; the service defaults `max_open_shards` to 1,024. The
benchmark driver sets it to 16 and supplies `DatabaseConfig::default()` with a
reported cache capacity of 128 pages:

- [`LocalTenantServiceConfig` defaults and `BlinkTenantService` fields](../../../../crates/dodb-server/src/lib.rs#L928-L1001)
- [Benchmark driver B-link configuration](../../../../crates/dodb-rocksdb/src/bin/service_workload.rs#L96-L108)
- [Reported B-link configuration fields](../../../../crates/dodb-rocksdb/src/bin/service_workload.rs#L1758-L1761)

When a tenant shard opens, the service calls
`BlinkStore::open_path` with a tenant-specific path and config. `open_path`
selects `open_with_wal`, which enters `open_internal`. The on-disk page loader
reads every page ID from the first data page through the superblock high-water
mark, decodes each page, and inserts it into `BlinkState.pages`:

- [Service shard open](../../../../crates/dodb-server/src/lib.rs#L1144-L1165)
- [`BlinkStore::open_path` selects `open_with_wal`](../../../../crates/dodb-storage/src/blink/mod.rs#L1913-L1923)
- [Open recovery and data-file loading](../../../../crates/dodb-storage/src/blink/mod.rs#L1925-L1971)
- [All-pages load into `BlinkState.pages`](../../../../crates/dodb-storage/src/blink/mod.rs#L2036-L2085)
- [`BlinkState` resident page map](../../../../crates/dodb-storage/src/blink/mod.rs#L699-L706)

This includes leaf, internal, overflow, and free pages. `BlinkPage` stores
leaf entries, internal entries, and overflow chunks as owned data:
[`BlinkPage` variants](../../../../crates/dodb-storage/src/blink/mod.rs#L580-L606).
The loader does not defer decoding a page until a Get, Query, or Scan touches
it.

## Published reads and eviction

`GenerationPublisher::new` walks `state.pages` and installs a `PageCell` for
each page, retaining an `Arc<BlinkPage>` in the published catalog. A
`BlinkReadHandle` holds the generation publisher, and each Get, Query, and Scan
pins a published generation before traversing it. The generation's
`ReadPageSource` retrieves the page from that catalog and returns its in-memory
page reference:

- [Page catalog population from every state page](../../../../crates/dodb-storage/src/blink/mod.rs#L1169-L1202)
- [`PageCell` retains an `Arc<BlinkPage>`](../../../../crates/dodb-storage/src/blink/mod.rs#L887-L905)
- [Generation pin and page lookup](../../../../crates/dodb-storage/src/blink/mod.rs#L1537-L1559)
- [`BlinkReadHandle` and request-scoped generation pins](../../../../crates/dodb-storage/src/blink/mod.rs#L1570-L1678)
- [Generation-pinned reads resolve through the catalog](../../../../crates/dodb-storage/src/blink/mod.rs#L4281-L4322)

Pinned old generations can retain superseded page objects until readers
release them. The generation publication code drops a retired generation when
it is no longer referenced, which reclaims superseded versions; it does not
evict current pages to a configured cache budget:
[`publish` and generation release](../../../../crates/dodb-storage/src/blink/mod.rs#L1510-L1525).

There is a separate `open_with_logical_wal` constructor that reconstructs a
`recovered_entries` map and publishes the recovered entries as an in-memory
overlay. The service's `open_path` call goes through `open_with_wal`, so that
`recovered_entries` branch is not the current `BlinkTenantService` recovery
path:
[`open_with_logical_wal` recovery entries](../../../../crates/dodb-storage/src/blink/mod.rs#L1846-L1898).
The current physical-WAL service path still loads the data file's full page set
into memory as described above.

## Memory and cache controls

`DatabaseConfig.cache_capacity` defaults to 128 and has a
`with_cache_capacity` setter. The native B-tree constructs and uses a
`PageCache` with that value. B-link stores the config, but its page state and
open path use the full resident page map and catalog described above; no
B-link page-cache lookup or eviction path consumes `cache_capacity`:

- [`DatabaseConfig.cache_capacity` and its default](../../../../crates/dodb-storage/src/btree/mod.rs#L96-L125)
- [Native B-tree `PageCache` field and construction](../../../../crates/dodb-storage/src/btree/mod.rs#L327-L348)
- [`BlinkStore` retains config alongside the full page state](../../../../crates/dodb-storage/src/blink/mod.rs#L1810-L1831)
- [B-link open loader inserts decoded pages directly into its full state](../../../../crates/dodb-storage/src/blink/mod.rs#L2071-L2085)

The service's `max_open_shards` is a count ceiling: once reached, opening
another shard returns `Overloaded`; it does not evict an old shard. It is not a
RAM or byte budget:
[`open_shard` capacity behavior](../../../../crates/dodb-server/src/lib.rs#L1144-L1165).
Storage value limits constrain an individual value size, and the execution
budget constrains a single response. Neither controls the total resident page
set. The coordinator's queue and transaction-group sizes bound queued work,
not database pages:
[`StorageLimits`](../../../../crates/dodb-storage/src/btree/mod.rs#L78-L105),
[response budgets in B-link service requests](../../../../crates/dodb-server/src/lib.rs#L1023-L1096),
[`CoordinatorConfig` limits](../../../../crates/dodb-storage/src/btree/coordinator.rs#L18-L38).

## Comparison constraint

The current implementation can be compared on datasets whose complete B-link
page state fits in available memory. It cannot provide a managed, bounded
B-link page cache for an over-effective-cache workload. Applying the same
external OS or container memory limit to both processes does not change this
storage behavior: B-link retains the loaded page state in process memory. If
that resident set exceeds the imposed limit, the run becomes a capacity/failure
case rather than a comparable over-cache throughput measurement.

This audit records a source-backed feasibility limit only. No B-link paging or
cache behavior was changed, no benchmark axis was narrowed, and no memory or
OOM experiment was performed. The phase decision remains pending the user's
response to the root's question about comparison scope.
