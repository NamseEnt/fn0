# B-link and RocksDB Workload Comparison Follow-up

Status: partial methodology update. No comparative baseline has been accepted;
the requested 24-run matrix, profiling, optimization gates, and 120-second
mixed run remain unrun.

The original report and result directory remain unchanged. Raw files and build
provenance collected so far are in
[`dodb-rocksdb-followup-b5aad16-20261007`](results/dodb-rocksdb-followup-b5aad16-20261007/).

## 1. Existing comparison limits and corrections

The earlier comparison used different pure-read request generators and
execution models. Its fixed Query16 repeatedly reads the reserved first
partition, so it does not represent ranges spread across the seeded keys. The
`logical_trace_prefix_hash` is reconstructed from the first 1,000 generated
mixed requests; it is neither a record of all executed requests nor their
completion order.

The B-link cache-capacity setting does not cap total process memory. The live
state retains page data through `BlinkState.pages` and the generation catalog.
The benchmark's previous one-million-entry latency reservoirs could themselves
retain hundreds of MiB at high client counts. Both benchmark paths now cap
each reservoir at 16,384 entries. At 16 clients, three 16-byte duration
reservoirs per client need at most 12 MiB for sample values, compared with
about 768 MiB at the previous one-million-entry cap; this is a capacity
estimate, not an observed RSS reduction.

For pure reads, the B-link timer now starts before request generation, matching
the RocksDB driver's timing boundary. Both engines use the same measured-phase
and worker seed formula. Get validation checks every returned byte against the
seeded value. Fixed Query16 validation checks row count, key order, value size,
and every returned value byte. Point-read setup now seeds the same 10,000 rows
for both engines.

The remaining execution mismatch is material: B-link readers are Tokio tasks
on two workers, while RocksDB readers are client OS threads. The harness also
does not yet report per-client completion counts, a full trace identity plus a
separately labeled prefix fingerprint, measurement-window RSS percentiles, or
ZFS ARC and OS cache observations. Therefore no throughput or latency
comparison is accepted from the smoke runs.

## 2. Corrected smoke

The latest smoke used source `a7c9d396d8b7a74a5959beee09b739696e391ad2`,
seed `981100100`, one client, 10,000 rows, 16-byte keys, 512-byte values, a
100 ms warmup, and a 200 ms measurement. Both raw records identify the same
source SHA and seed, both seeded 10,000 rows, and both reported zero errors.
B-link completed 147,824 reads; RocksDB completed 114,252 and its full-value
post-run verification passed. The raw data also records the B-link 16,384
latency sample cap. These short-run rates are smoke evidence only because the
two engines still use different client execution models and the smoke has no
trace fingerprint or client-distribution record.

The corrected smoke binary hashes, raw checksums, host, filesystem, and
commands are recorded in the result manifest and provenance files. The source
checkout was clean before and after the build and run.

Two earlier smoke pairs are preserved but excluded. The `b5aad16` pair lacked
the new per-read full-value check and seeded 256 extra B-link rows for Get.
The `406d93d` pair fixed value checking and row counts but still used a
differently masked B-link reader seed. The later seed correction is the first
smoke whose generator seed formula matches across both paths. An initial
RocksDB link attempt also failed because the available static archive could
not link as PIE; rebuilding with `-no-pie` succeeded without changing source.

## 3. Requested baseline matrix

The planned core remains 24 measured executions: 10,000-row Get at c1 and
c16, 10,000-row distributed Query16 at c16, and 250,000-row Get at c16, each
for two engines and three paired repetitions. At 2 seconds of warmup and 5
seconds of measurement this is 168 seconds of timed intervals, excluding
database setup, builds, validation, and host overhead. An initial estimate is
20–40 minutes total; it will be revised after a complete methodology smoke.

The matrix has not run. Distributed Query16 and the common execution mode are
not implemented yet, so existing fixed Query16 results cannot substitute.

## 4. Get and mixed-path evidence

The source exposes generation pinning and page lookup on the B-link read path.
The shared generation lock, `Arc` reference updates, atomics, and value-copy
cost remain candidate explanations only. No accepted profile from the corrected
comparison exists, and no engine optimization has been selected.

The old mixed data is useful for choosing follow-up cases, but it does not
isolate the requested group collection, planning, WAL, sync, publication, and
response costs. In the coordinator, `queue_wait_nanos` accumulates elapsed time
from enqueue until after group processing. It overlaps group collection and
processing and must not be added to those metrics as a separate component.
No width-4/8 optimization was attempted, and no 120-second representative
mixed run was performed.

## 5. Memory and measurement program cost

The corrected smoke's RocksDB record contains process RSS start, after-seed,
end, and high-water values. It is too short and uses one client; the B-link raw
record does not yet provide the same process RSS series. Neither result
separates measurement-window median, p95, and maximum RSS, engine-resident
data, dirty pages, RocksDB cache and memtables, or ARC and OS cache. No
memory-normalized throughput claim is made.

## 6. Product workload replay

The repository contains a product workload policy describing point reads,
widths 1–8, short queries, hotspots, and burst traffic. No authorized,
anonymized production request trace was found. Until one is supplied, any
replay will be labeled synthetic and its PK, operation, width, pagination,
arrival, and burst distributions will be explicit assumptions. The existing
`operation_index % 100` schedule is a deterministic closed-loop ratio
schedule, not an observed product arrival pattern.

## 7. Adoption and remaining limits

No engine change has been adopted and no performance result from this
follow-up establishes a winner. The measurement-only changes are committed at
`b5aad16cef949e9382232abcb6c6bec2b5661cd6`,
`406d93d304b9ab630a7db024a51ac6729fde4a01`,
`022ad1c5c382d3e40703e6c2df72c64c72947255`, and
`a7c9d396d8b7a74a5959beee09b739696e391ad2` on the experiment branch.

The next acceptance gate is to add distributed Query16, record trace identity
and per-client completions, and run both engines through a shared reader
execution model. Then run the 24-case matrix and retain paired raw values,
resource samples, and provenance. Get profiling, mixed-path changes, their
regression checks, and the long durable mixed run follow only after that
baseline is accepted.
