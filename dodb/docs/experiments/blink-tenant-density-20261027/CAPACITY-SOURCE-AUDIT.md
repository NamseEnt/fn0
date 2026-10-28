# Source boundaries for the bounded capacity comparison

Runtime source: `aaa40565529ca92150df5d94e19b02ca93859f71`. These are source-confirmed facts, not retained-heap measurements or proof of a measured bottleneck.

## Residency and opening

`dodb/crates/dodb-storage/src/blink/mod.rs:2060` (`load_base_images`) decodes the complete allocated page range into a `BTreeMap<PageId, Arc<BlinkPage>>`. The file-backed loader at line 2242 does the same. `load_decoded_pages` constructs the state and published generation from that map. The generation read path at line 4704 obtains an already resident page; it does not implement an eviction miss followed by storage loading.

`BlinkTenantService` keeps opened shards in its registry until service shutdown. The open-shard maximum rejects additional opens; it is not a memory budget or automatic idle close. Thus setup and both oracle epochs open and retain the registered population. Sparse measurement has a separate worker and opens only the configured active and opened-idle population. Its lighter measurement cannot replace the all-population acceptance requirement.

Whole-tree residency can explain why data-dependent memory costs matter. A measured high event or OOM does not by itself identify the exact contribution of decoded pages, allocator retention, catalog maps, checkpoint overlap, or recovery validation. Those require additional representation/heap attribution.

## Workload and timing

`tenant_density_workload.rs:310` seeds rows in synchronous sixteen-mutation transactions, one tenant after another. Each tenant's seed keys share a sixteen-byte zero PK and have a sixteen-byte big-endian row SK. Seeding is outside the offered-load measurement and may itself fail a resource limit.

The scheduler's `seeded_read_row` uses `(operation_number * 0x9e37 + tenant_number) % initial_rows`. With the mixed selector, Get occurs on forty of each hundred offered operations. Query repeatedly reads the first sixteen seed rows. Mutation lanes are separate from seeded rows and overwrite/delete/reinsert a fixed sixteen-key set. A large seeded population is not a claim that the complete seed dataset is accessed during a short measurement. `evidence/capacity/frontier.py` reports the distinct offered Get rows and their seed-row fraction. Only no-drop cells realize that entire offered recipe.

Request latency starts at its absolute scheduled arrival and includes the bounded tenant queue, shared in-flight admission, transport, and backend completion. The per-tenant lane is sequential; the aggregate in-flight limit is eight. All tenant lanes have synchronized arrival deadlines. This can expose burst queueing even when mean CPU or memory is below a ceiling. It is not a maximum-engine-throughput test.

The diagnostic worker directly awaits the B-link service. Minimal-native requests run through `tokio::task::spawn_blocking` around the synchronous TransactionDB service. These are the accepted service routes with the same outer in-flight cap, not identical internal scheduling or a raw engine comparison. A latency difference alone does not isolate storage CPU, blocking dispatch, transport, or checkpoint interference.

Offer-end completed counts are captured before the diagnostic callback. The reported offer elapsed time is captured after that callback, so derived completed-per-second can be slightly below the nominal rate even when every offered request completed on time. The service objective compares per-tenant completed counts with the frozen nominal offered count, rather than treating callback time as dropped service.

## Observation boundaries

The preparation amendment in `memory_diagnostic_worker.rs` removes checkpoint-event details and the group histogram from periodic JSON samples, retaining cumulative counters and the unchanged final detailed worker report. It still constructs a cumulative checkpoint snapshot before removing those fields, so transient allocation/copy and CPU costs remain possible. The 100/1,000 ms sensitivity cannot prove zero instrumentation overhead.

The final checkpoint-event vector has an existing 1,024-event limit and an explicit omitted-event counter. Final cumulative counters can remain usable when detailed events are omitted; any event-derived maintenance analysis must retain that limitation. The bounded comparison requires no omitted events in accepted cells and stops for review if that gate is crossed.

Kernel memory peaks cover the complete cell or worker epoch, including opening/preflight/drain. Periodic memory and native samples are non-atomic. Process RSS sampling is separate and slower. Neither is retained engine heap. Linux charging leaves the driver and uncharged host ARC outside the common backend cgroup limit; previously charged shared pages are not retroactively migrated.

Filesystem phase snapshots and whole-host device counters have different scopes. Reachable-file logical/allocated bytes at shutdown do not establish continuous temporary-space peaks, physical returned capacity, or backend write amplification. Clean full-population reopen/oracle is not isolated recovery latency or a new crash-recovery test.
