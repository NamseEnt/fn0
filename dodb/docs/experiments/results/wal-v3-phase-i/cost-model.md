# Phase I Cost Model

## Measurement basis

Primary profile: 64 writers, width 16, uniform keys, unconditional writes, 100,000-row working set, 64-request group limit, two Tokio and two B-link workers, OCI A1 2 OCPU. The H1 profile recorded approximately 38.04 billion user cycles across 61,804 successful transactions, or about 616,000 sampled user cycles per transaction. The separate `perf stat cycles:u` throughput runs are the basis for total CPU-demand estimates and must not be numerically mixed with sampled profile event counts.

H1 disabled-sync throughput medians were 6,122 tx/s (6,102, 6,122, 6,190); real-sync medians were 4,598 tx/s (4,598, 4,492, 4,614). CPU utilization was approximately 1.18 one-core equivalents in disabled-sync runs, giving `1.18e9 / 6122 = 193,000 ns/tx` estimated CPU demand. Real-sync WAL sync time was about 72 microseconds per transaction. These are host-level estimates, not a direct phase timer.

## H1 foreground components

Profile attribution is sampled and flat percentages may omit inlined work. The `prepare_leaf_parallel_execution` callgraph subtree was approximately 42.78% inclusive. Component rows below use the explicit instrumented cumulative timing counters from a representative H1 run, normalized by 61,804 successful transactions; cumulative lane timings can overlap and are not additive. The perf flat report is an independent attribution, not a partition.

| Component | Profile / measured evidence | Under Model C response path? | Reason |
|---|---:|---|---|
| Logical admission and condition validation | `logical_admission_nanos_total` 424.3 ms, about 6.9 us/tx | Yes | Conditions, revision checks, and failure isolation remain required. |
| Tree routing | planner route 1.195 s / 988,864 route calls, about 1.21 us/mutation | Partial | Conditional reads and base fallback still route; unconditional put/delete can avoid B-link routing. |
| Leaf load/clone and mutation | physical worker subtree; mutation timer 18.8 ms plus leaf-load clone 6.9 ms | No for pure overlay writes | Logical staging replaces in-place physical leaf mutation; transient admission overlay remains. |
| Page encoding and canonical checksum | physical page encode 14.0 ms; superblock encode negligible | No | Canonical physical pages move to materialization. |
| Image fingerprint and PageDelta | parallel worker delta 2.241 s; flat `encode_page_delta` symbols sum to about 10.64% | No | Physical redo generation is absent from logical commit foreground. |
| PageDelta validation/rebuild | outside fast H1 append path for trusted planned images; residual validation remains in recovery/fallback paths | No on the normal logical path | A logical WAL records committed mutations and must not generate physical deltas. |
| WAL payload/frame/digest CRC and framing | WAL append about 985 ms / 61,804 = 15.9 us/tx in the profile run; includes per-frame CRC/digest and framing | Yes, changed | Logical records still need framing, digest, commit marker, and validation. Size and CPU must be measured in a real WAL prototype. |
| WAL write | 241.8 ms cumulative write counter, about 3.9 us/tx; can overlap with group batching | Yes | Durable bytes must still be written. |
| fsync | benchmark H1 real-sync about 72 us/tx; shared per group | Yes | Durable transaction response still waits for sync completion. |
| Catalog construction | 478.8 ms / 61,804 = 7.7 us/tx | No for response, after-image catalog only | Deferred logical commit need not build a physical page catalog. WAL metadata/catalog for replay may remain. |
| Generation publication | 405.5 ms / 61,804 = 6.6 us/tx | Partial | Overlay view publication remains; physical generation publication moves later. |
| State install | 142.2 ms / 61,804 = 2.3 us/tx | No for B-link state; yes for overlay view | Publish immutable view pointer/snapshot metadata after fsync. |
| Dirty tracking | 346.7 ms / 61,804 = 5.6 us/tx | No for physical pages | Logical WAL retention and overlay segment accounting remain. |
| Allocator | `mi_page_malloc_zero` 3.41% plus other allocator symbols | Partial | Removing page/catalog allocations helps; overlay entries and immutable segments allocate. |
| Scheduler/dispatch | parallel dispatch 518 ms and join 383 ms cumulative; perf includes worker/coordination costs | Partial | B-link work dispatch is removed; logical admission, WAL dispatch, and publication coordination remain. |

Instrumented worker, join, routing and timing counters have overlap and are not added to produce the 42.78% inclusive subtree. The profile is descriptive; the gate uses that subtree only as an optimistic removable ceiling.

The following table supplies the available normalized component timings. `Share of CPU-demand estimate` is each counter's ns/tx divided by the independent 193 us/tx CPU-demand estimate; it is an attribution aid, not an additive CPU partition. Worker lanes overlap coordinator and wall-time counters.

| Component counter | ns/tx | Approx share of H1 CPU-demand estimate | Model C response-path classification |
|---|---:|---:|---|
| Logical admission | 6,865 | 3.6% | Retained |
| Planner routing | 19,333 | 10.0% | Partial; unconditional pure writes can avoid B-link routes |
| Total planning | 28,417 | 14.7% | Partial; logical validation and staging remain |
| Physical execution | 75,908 | 39.3% | Mostly removable from response path |
| Parallel worker lane | 103,918 | 53.8% | Overlapping worker service time; physical base/mutation/encode/delta work moves out |
| Parallel base lane | 16,964 | 8.8% | Removable for unconditional writes; condition reads still need lookup |
| Parallel mutation lane | 13,857 | 7.2% | Removable for physical leaf mutations |
| Parallel encode lane | 29,705 | 15.4% | Removable for physical page encoding/checksum |
| Parallel delta lane | 36,256 | 18.8% | Removable for PageDelta production |
| Parallel dispatch | 8,388 | 4.3% | B-link dispatch removable; logical admission/WAL coordination remains |
| Parallel collect | 3,207 | 1.7% | Mostly removable with deferred physical work |
| WAL assembly | 445 | 0.2% | Retained but record shape changes |
| WAL append | 15,937 | 8.3% | Retained; encoding/checksum cost must be remeasured for logical records |
| Catalog construction | 7,747 | 4.0% | Physical catalog removed from response; logical recovery metadata remains |
| Catalog state scan | 5,467 | 2.8% | Physical state catalog scan removed |
| State install | 2,301 | 1.2% | B-link install removed; immutable overlay publication remains |
| Generation publication | 6,562 | 3.4% | B-link generation publication deferred; overlay pointer publication remains |
| Dirty tracking | 5,610 | 2.9% | Physical dirty tracking removed; WAL retention accounting remains |
| WAL sync counter | 4 | <0.1% | Retained; counter covers a short subset in this H1 instrumented row and is not the wait model |
| Real-sync wait estimate | ~72,000 | ~37% | Retained, group-shared wait divided by transactions/group |

The top-level categories overlap (for example, `physical execution` contains worker work, and generation publication contains retired-generation cleanup). Treating the table as a sum would double count. The separate sampled profile reports about 616k cycles/tx and 42.78% inclusive physical preparation.

## Three model ceilings

### Model A: final-leaf-only materialization

Real-sync width16 uniform groups have 702.16 boundaries and 666.42 distinct leaves on average: 5.09% materializations are theoretically removable. Sync-disabled is 682.54 versus 646.43, or 5.29%. If physical worker preparation scales linearly with boundaries, this removes about 2.2% of total CPU and yields roughly 1.02x. Same-leaf-heavy removes 94% and can matter for that workload; this does not transfer to uniform traffic.

### Model B: CPU/fsync pipeline

Use the current CPU demand estimate (193 us/tx) and real-sync time (72 us/tx). A serialized CPU+sync idealization costs 265 us/tx. Perfect overlap costs `max(193,72)=193 us/tx`, for 1.37x. This is a generous ceiling because the current group already shares sync and two OCPUs let storage sync compete with worker CPU. Measured throughput ratio is 6,122/4,598=1.33x. The remaining gain above measured H1 is therefore small.

### Model C: durable logical overlay

Removing the entire 42.78% inclusive physical preparation subtree gives a sampled-cycle floor of `616,000 * (1 - 0.4278) = 352,300 cycles/tx`, or an ideal 1.75x CPU ceiling. This floor still includes profile costs from WAL work, admission, overlays, coordination, reads required by conditions, and unrelated CPU.

The isolated prototype measured 6.0–6.5 us/tx across 0–32 prior segments, versus about 193 us/tx estimated H1 CPU demand. This is a 0.031–0.034 ratio and clears the CPU gate, but the comparison does not include real WAL encoding, CRC, fsync, production group dispatch, or mixed read/write impact. It is not a projected durable throughput result.
