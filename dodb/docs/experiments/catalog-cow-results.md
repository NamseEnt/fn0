# Chunked COW Generation Catalog Results

## Motivation

Phase 4's OCI delay-zero planned control showed a 7818.20 mut/s median. Catalog construction took about 348.46 ms per measured run; generation publication took about 413.72 ms, including about 413.09 ms dropping the retired generation. The dominant costs came from cloning the full `PageCatalog.pages` BTreeMap per generation and releasing all of its page references when the generation retired.

This catalog lifecycle experiment evaluates that serial cost independently. Phase 4's worker result remains ineffective on the current 2-OCPU target. Phase 5 implementation has not started.

## Design

### 64-page chunks

The catalog is a flat `Vec<Arc<PageCatalogChunk>>`. Each chunk contains a fixed array of 64 optional page cells. Physical page IDs map directly by integer division and remainder; page IDs 0 and 1 are excluded. Lookup uses one directory index and one array index.

### Shared immutable chunks

Generation preparation clones the outer vector of chunk Arcs. Chunks without dirty pages remain shared by old and new generations.

### Dirty-chunk copy-on-write

Dirty IDs are grouped by chunk. Each affected chunk is cloned once, then only its dirty slots receive new `PageCell` values. Page version install accounting increments only for those new cells. The historical `catalog_map_clone_nanos_total` now measures outer-directory clone time plus dirty-chunk clone time; its name is retained for comparison with the old BTreeMap implementation.

### Pinned-generation retention

A pinned reader owns its generation, which owns the corresponding chunk Arcs and PageCells. Updating a dirty slot installs a new chunk for the next generation; the old pinned generation continues to read its old PageCell and BlinkPage. Publication still drops the retired generation while holding the write lock.

### High-water extension

When the high-water mark grows, the flat directory extends with empty chunks through the required index. Every newly allocated physical page from the previous high-water mark through the new one must be dirty and present in the state before the planned generation can be prepared.

## Correctness

The following passed: `cargo fmt --all -- --check`, `cargo test -p dodb-storage` (85 tests), `cargo test -p dodb-storage --bin phase0-bench` (10 tests), `cargo test --workspace`, and `git diff --check`.

Focused tests cover lookup across physical IDs 63/64/65/127/128, unchanged chunk `Arc::ptr_eq` sharing, one clone for several dirty pages in one chunk, three clones for three dirty chunks, directory extension and rejection of missing new high-water pages. Existing pinned-generation, page-reuse, root-split, unpublished-install, WAL-failure atomicity, randomized differential, query, and scan tests passed.

Local write smoke used planned Blink, 16 writers, width 1, different-leaf-heavy, working set 4096, disabled sync, zero delay, and one 1 s measurement. It reported zero errors, overloads, full-state clones, and state-clone nanoseconds; chunk size was 64 and it cloned 2735 dirty chunks. Local read smoke used two Get readers over working set 4096 and completed with zero errors. These local runs are smoke evidence only, not performance results.

## OCI Method

The binary was built in release mode at catalog implementation commit `5258862824c02269c885e4753f38ec9d9f7ab8e7` on the 2-OCPU OCI A1 host. Both runs used sync-disabled mode and three repetitions. The mounted path `/home/opc/dodb` resides on the 30 GB XFS root filesystem; the 200 GB block device is not mounted as a filesystem. These are CPU and engine diagnostics, not durability or 200 GB volume measurements.

The width-1 primary run used 16 writers, width 1, different-leaf-heavy, working set 100,000, cache 4096, 16-byte keys, 64-byte values, group limit 64, group byte limit 4 MiB, queue 256, delay 0, two Tokio workers, warmup 1 s, duration 2 s, and seed `0x3a042026`.

The width-16 check used planned Blink, 16 writers, width 16, uniform distribution, working set 100,000, cache 4096, sync disabled, delay 0, two Tokio workers, warmup 1 s, duration 2 s, three repetitions, and seed `0x3a032026`.

## Width-1 Primary Result

The before values are medians from the exact Phase 4 planned control artifact. The after values are medians from the three chunked-catalog records. Component times are cumulative per measured run, not normalized per mutation; the new run completed more mutations.

| metric | before | after | reduction |
|---|---:|---:|---:|
| mutation ops/s | 7818.20 | 12643.78 | — |
| processing | 1929.65 ms | 1926.34 ms | 0.2% |
| catalog construction | 348.46 ms | 71.59 ms | 79.5% |
| catalog map/structural clone | 311.88 ms | 26.69 ms | 91.4% |
| catalog directory clone | n/a | 4.45 ms | n/a |
| dirty chunk clone | n/a | 22.31 ms | n/a |
| dirty chunk clones | n/a | 25,302/run | n/a |
| catalog state scan | 36.69 ms | 65.05 ms | -77.3% |
| generation publication | 413.72 ms | 57.62 ms | 86.1% |
| retired generation drop | 413.09 ms | 56.65 ms | 86.3% |
| WAL append | 528.79 ms | 840.65 ms | -59.0% |
| physical execution | 316.39 ms | 494.86 ms | -56.4% |
| planning | 177.72 ms | 292.55 ms | -64.6% |

## Lifecycle Breakdown

Using the prescribed sum of median catalog construction and median generation publication, lifecycle fell from 762.19 ms to 129.20 ms per run, an 83.0% reduction. The throughput ratio was `12643.78 / 7818.20 = 1.617x`.

The median dirty-chunk clone count was 25,302 per run, or 8.01 clones per logical group. The historical aggregate now equals directory clone plus dirty-chunk clone (4.45 ms + 22.31 ms = 26.69 ms, subject to timer granularity). Directory cloning only copies chunk Arcs; retired-generation release now touches the shared directory and changed chunks rather than traversing every catalog page.

Both strong-success conditions passed: lifecycle is below 40% of baseline and throughput speedup exceeds 1.15x. The result is strong success for removing the measured O(total-pages) lifecycle cost.

## Width-16 Regression Check

| metric | before | after | change |
|---|---:|---:|---:|
| mutation ops/s median | 13773.48 | 13457.64 | -2.3% |

The result remains above the 0.90 regression threshold of 12396.13 mut/s.

## Interpretation

The catalog map clone fell from 311.88 ms to 26.69 ms. Retired generation drop fell from 413.09 ms to 56.65 ms. Unchanged chunk sharing therefore substantially reduced both preparation and retirement costs while preserving old pinned generations.

## Remaining Bottleneck

WAL append is now the largest measured non-overlapping top-level serial component at 840.65 ms per run. Physical execution is 494.86 ms and planning is 292.55 ms. WAL append is the next engineering priority; it was not optimized in this task.

## Experiment Status

Phase 4 remains ineffective on the current 2-OCPU target. This serial lifecycle optimization was evaluated independently. Phase 5 implementation has not started, and this result does not mark Phase 5 ready.

## Artifacts

- Width-1 primary: [`planned-chunked-catalog-width1-nosync.jsonl`](results/oci-a1-2ocpu-12g-200g/catalog-cow/planned-chunked-catalog-width1-nosync.jsonl), SHA256 `f27588264950c30de29164c58bde3822bd4d2f0b58766fb33f8a534d81b0650f`
- Width-16 regression check: [`planned-chunked-catalog-width16-nosync.jsonl`](results/oci-a1-2ocpu-12g-200g/catalog-cow/planned-chunked-catalog-width16-nosync.jsonl), SHA256 `fe90047e94b8d91be8ac777407efdd168f89658d86a27b21c5c592f3fa449332`
- Both artifacts contain three validated records at implementation commit `5258862824c02269c885e4753f38ec9d9f7ab8e7`. Local SHA256 values match the OCI copies.
