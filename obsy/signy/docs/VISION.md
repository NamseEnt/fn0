# signy vision

**signy answers, inside a memory budget you declared, on one machine, the
queries Loki has to brute-force.**

[`ARCHITECTURE.md`](ARCHITECTURE.md) records what this engine *is*. This document
records what it is *for*, which of its properties are load-bearing, and what
would prove the claim wrong. Where the two disagree, this one is the intent and
the other is the implementation.

The feature surface is done. Logs, traces and metrics ingest over OTLP, the first-party
query API answers the fn0 console and any agent driving `curl`
([`QUERY_API.md`](QUERY_API.md)), tenants are isolated and their retention is
enforced, and the durability protocol survives crashes, restarts and a split
writer. What remains is not features. It is that this engine does not yet keep
the promise its shape implies.

---

## Three invariants

Everything below is a target. The current state violates all three, and each
section says where.

**They are not independent, and the order was wrong when this document was
written.** It listed them as three parallel goals. [`MEMORY_ATTRIBUTION.md`](MEMORY_ATTRIBUTION.md)
established that **II is a precondition for I**: at the moment the kernel killed
the process at a 2 GiB limit, 44% of its anonymous footprint was memory it had
already freed, and its live heap was 669 MiB. An arena budget denominated in live
bytes would have reported a third full while the process was dying. You cannot
budget memory the allocator will not give back, and the way to make it give it
back is to stop asking for it — the engine requested 52 GB across 444 million
allocations in 33 seconds, 217× the rate data arrived at.

So invariant I's arena machinery is the *last* step, not the first. Ahead of it,
in order: ~~the verification test~~ — **built** ([`MEMORY_BUDGET_GATE.md`](MEMORY_BUDGET_GATE.md));
then II, because that is where the churn is; then whatever the allocator still
retains, measured and published as a multiplier; and only then the arenas, sized
from what remains.

**The order was right, and the first step of II is what proved it.** The gate was
red at 2 GiB and green at 5, with a 2.24× overshoot when given room. Sharing label
sets rather than copying them per row — nothing else on either list — made it
green at 2 GiB and took the overshoot to 0.93×, at a higher delivered rate than
before. That is one item of II's list moving invariant I's number by 2.4×, which
is the argument for doing II before I rather than a claim about it.

**The second step bought headroom rather than another whole step down.** The
streaming top-K executor took the 2 GiB pass from 90–96% of the budget to
**78–83%** and the workload's own anonymous high-water from 1913 to 1659 MiB, at a
slightly higher delivered rate again — and 1792 MiB is still `OOM_KILLED`. Two
items of II's list, and the second one moved the margin and not the floor;
[`MEMORY_BUDGET_GATE.md`](MEMORY_BUDGET_GATE.md) says why the two need not move
together.

### I. Memory is a budget you declare, not a number that emerges

An operator gives one number:

```
SIGNY_MEMORY_BUDGET=1GiB
```

That number is the same number they put in the container's memory limit, and the
engine's job is to stay under it. The budget is divided into **arenas**, each
with its own accounted allocation and its own refusal when full. The shares
below are the ones [`MEMORY_ATTRIBUTION.md`](MEMORY_ATTRIBUTION.md) measured;
the earlier table here was a guess and every one of its numbers moved:

| Arena | Share | Measured high-water at 2 GiB (2026-08-07, sweep build) | Holds | On overflow |
|---|---|---|---|---|
| ingest | 20% | ~0 (memtable gauge peak 5.2 MiB — the chunked flush drains it at cadence; 378 MiB on `50190cf`) | memtable, trace memtable, in-flight collected records (bounded since 2026-08-12: `max_inflight_push_bytes`, charged per record as the batch is decoded, so what is held is one record and the handful behind it awaiting an fsync — never the batch) | `429` + `Retry-After` (already the mechanism) |
| flush | 25% | 30.9 MiB (was 96.1 on `f7d9a36`, 721 on `50190cf`) | one chunk of materialized rows (`SIGNY_FLUSH_CHUNK_BYTES`), Parquet writer buffers | defer the flush; ingest backs up into its own arena and refuses there |
| merge | 25% | **442.4 MiB** — the dominant arena, 86% of a 25% share of 2 GiB (771 on `50190cf`; 326.5 on `761999a`) | one merge group's paging (`merge_max_memory_bytes / 2`, per-part pages clamped 2–8 MiB) | split the group; skip the tick |
| query | 25% | 298.4 MiB tagged, **of which ~284 MiB is the row-group cache's retained batches** — decoded under the query tag, held by the cache, separated by the `signy_row_group_cache_bytes` gauge; the scan transient itself is tens of MiB | every concurrent scan, pipeline stage and metric evaluation | queue, then `429` |
| row-group cache | (256 MiB knob) | 284.7 MiB gauge peak before the retraction fence; the fence now keeps the shared counter at its budget | decoded row groups and narrow-pass outcomes, evicted LRU per reader | retract the insert |
| sidecar | 5% | 34.9 MiB (17 before the 0.1% window blooms) | blooms, stream index, part metadata | evict least-recently-used sidecars; reload from `index.bin` |

Shares are defaults, individually overridable. What is not overridable is that
they sum to the budget.

**`SIGNY_MEMORY_BUDGET` exists now (2026-08-08)** — unset, the engine
reads the cgroup limit and declares 60% of it, deriving the merge, query,
row-group-cache, sidecar-cache and memtable ceilings from the measured shares
(`docs/CONFIGURATION.md`, "Memory budget"). These are derived *ceilings*, not
yet the per-arena accounted refusal this section sketches — the shares above
remain the target shape. The allocator-retention multiplier this section said
to measure and publish was measured, published, and then retired outright:
the 24-hour soak showed glibc's retained-free creep killing 2 GiB in hours
with every knob applied. The production allocator was **jemalloc** from
`8592094` and is **mimalloc** since 2026-08-31. And the number an operator actually asks this invariant for, from
the completed 24-hour soak at a 2 GiB container (2026-08-10): **sustained
capacity is the whole offered 20 k eps** — 19,999.8, nothing throttled — for
24 hours and 1.73 billion events, with anon flat between 1.47 and 1.57 GiB,
query response p95 428 ms / p99 640 ms and zero 5xx in 432,001 queries
(`soak-24h-lockorder`, 2026-08-12). The predecessor read ~18.6 k eps with 6.9%
throttled, and the difference was one lock-order defect — retention held the lock
every query needs while it spun for one a merge rewrite was holding, stopping the
server for up to 52 s at a time and starving the flush thread into backpressure
(`ca32ee5`).

**And the ceiling above 20 k is measured now** (a rate ladder at the same
configuration, 45 minutes a rung, 2026-08-13 and re-run 2026-08-14). Two
numbers, because a capacity has two honest forms:

| offered | achieved | refused `429` | memtable peak (limit 122.9 MiB) |
|---|---|---|---|
| 20 k (24 h) | 19,999.8 eps | 0% | — |
| **30 k** | **29,996.5 eps** | **0%** | 78.0 MiB |
| **45 k** | **34,666 eps** | 22.9% | **126.3** |

**Sustained without refusing anything: 30 k eps.** **Absorbed under overload:
34,666 eps**, the rate the engine settles at while refusing the rest — and every
refusal is a `429`, with no 5xx and no OOM at any rung.

*Both numbers replace lower ones, and the reason is one line of code.* The
ladder first read **22 k sustained / 24,274 absorbed**, and the flush loop's
phase table then put 63% of the pass inside `write_index` — building the trigram
and exact-field blooms that make the read claim below true. That set was a
`BTreeSet` over a 2²⁴ domain where a bitmap over the same domain produces the
identical filter (`816b260`); with it, `write_index` costs 10.9 µs an event
instead of 25.2 and is 37% of the pass instead of 65%. **This engine buys its
read speed with its write capacity, and the price was being paid twice.** The
22 k/24,274 pair is retired rather than deleted: it is what the same rig
measured two commits earlier, and it is why the ladder is re-run after any change
to the flush path.

**What set that historical rung was flush, not durability.** At every refusing
rung `memtable_buffered` pinned against the then-derived 122.9 MiB
`max_memtable_bytes` — the gate
whose message is "flush is not keeping up" — while the WAL backlog peaks two
orders of magnitude below its own limit. At 30 k, where nothing is refused, the
memtable sits at 78.0 MiB and the flush loop is 88% busy rather than 95%. The
write-ahead path has headroom the whole way: group commit amortizes as the rate
rises (`records/batch` 1.59 → 4.07), so the journal writer is *less* busy at a
higher accepted rate. The capacity of this engine at 2 GiB is how fast a memtable
becomes parts.

*Where the knee is, stated as what was measured rather than as a curve:* 30 k
refuses nothing and 45 k refuses 22.9%, and no rung between them has been run.
The earlier ladder bracketed its knee to 2 k and this one is bracketed to 15 k.

**Two of the five did not fit their share when this was measured**, and that is
the finding rather than a sizing problem: flush materialized a whole memtable
snapshot at 3.3× its accounted size, and one merge group reached 771 MiB against
a `merge_max_memory_bytes` default of 1 GiB — half the container. Their shares
are targets the code must be made to meet (invariant II's `Arc<Labels>` and a
chunked flush; a group split sized from the budget), not descriptions of it.
Both have since been made to meet them: the chunked flush (`f7d9a36`) bounds
the flush transient at the chunk (96.1 MiB measured against a 25% share of
512 MiB), and `merge_max_memory_bytes` actually bounds a rewrite's paging
again. `memory_gate --budget 2GiB` reads UNDER_BUDGET at 39.8% with the settle
included, and todo.md's stall section has the run-by-run record.

**Re-measured 2026-08-07** (`scripts/run_memprof_local.sh`, 2 GiB cgroup,
20k eps + 5 qps, the bed's seed): anon peak 951–1245 MiB across two legs
(the spread is the row-group cache filling under the query mix),
**anon/live 1.34–1.69** where `50190cf` measured 5–8 — the allocator-retention
multiplier the plan said to measure and publish is now this, mostly retired
by the arena cap (4), the trim threshold and invariant II's copy removals.
What the re-measurement surfaced beyond the table: the ingest arena has
effectively dissolved (the flush cadence keeps the memtable at single-digit
MiB), and the row-group cache's bytes ride the *query* tag because memprof
attributes at allocation — the gauge is the separator, and the cache's own
budget was measured 11% over under concurrent readers, which is what the
insert-retraction fence fixed.

**Two consequences that are architectural, not tuning.**

*The query arena replaces the concurrency knob.* Today the query bound is
`MAX_CONCURRENT_QUERY_SCANS × MAX_QUERY_MEMORY_BYTES` — 8 × 512 MiB = 4 GiB
(`config.rs:522` says so in a comment). A product of two knobs that never appear
next to each other is not a budget. Admission becomes "is there room in the query
arena", not "is there a free slot", so a burst of cheap queries runs wide and a
burst of expensive ones queues. Same ceiling, better degradation, one number.

*Sidecars move inside the budget.* They are currently outside it on purpose
(`part/reader.rs:77-81`), which means resident memory grows with part count and
nothing stops it. `PartMeta::streams` is worse than the blooms: it materializes
every distinct label set in every open part (`part/mod.rs:231`,
`part/metadata.rs:172-176`), so its cost is stream cardinality × part count in
live `String`s. Sidecars are already durable on local disk, so making them
evictable costs a re-read of `index.bin`, and stream identity does not need the
label text — a fingerprint is enough. Measured: **~240 kB of sidecar and ~140 kB
of `PartMeta` per part**, which is 0.2–1.2% of the anonymous peak at the part
counts the comparison bed reaches. The concern is right and the 10% this
document used to give it was wrong by an order of magnitude.

**Accounting must be honest before any of this means anything.** `entries_bytes`
(`memtable.rs:69-81`) counts `line.len()` plus label name and value lengths and
nothing else — not the 56-byte `LogEntry`, not the 48-byte slot per metadata
pair, not malloc headers, not `Vec` slack. Measured shapes come out **1.4× to
2.8× under** in isolation and **1.70–1.79× under** in situ on the comparison
corpus, so `MAX_MEMTABLE_BYTES=256 MiB` is really ~440 MiB. A budget computed
from a dishonest meter is a worse guarantee than no budget, because it will be
believed.

**And a live-byte meter is not enough on its own.** At the moment the kernel
killed the process at 2 GiB, the engine's live heap was **669 MiB** — every
arena inside its share, the budget reporting a third full — while 44% of the
anonymous footprint was memory the process had already freed and glibc had not
returned. The engine asked for 52 GB across 444 million allocations in 33
seconds; at that rate the allocator's steady state is a heap the size of the
transient peak. So the budget has a precondition: the process must make its own
anonymous footprint track its live bytes (measured: `MALLOC_ARENA_MAX=1` with
trim thresholds takes `anon / live` from 2.5–4.1 to **1.34** and more than
doubles time-to-OOM), and the budget must be verified against the cgroup's
`anon` rather than against the sum of its own arenas.
[`MEMORY_ATTRIBUTION.md`](MEMORY_ATTRIBUTION.md) is the measurement.

**How this is verified:** a test that runs the engine at a declared budget under
sustained mixed load and asserts peak RSS stays under it. Not a sizing paragraph
in the runbook. Today the runbook says "size on peak, not idle — roughly fifty
times" ([`RUNBOOK.md`](RUNBOOK.md):27), which is the honest description of an
engine that does not have this invariant.

**That test is built, it is the first of these steps rather than the last, and it
was red.** `src/bin/memory_gate.rs` runs the comparison bed's workload — ingest
with reads concurrent with writes — in a cgroup v2 scope at a declared budget and
compares the peak of the cgroup's `anon` against it, because the engine's own
accounting is the thing being audited and cannot be the auditor. It distinguishes
four outcomes by exit code, and *could not be measured* is one of them and is a
failure. Measured on build `50190cf`: at the 2 GiB the comparison bed used,
**OOM-killed at t≈49 s**; the smallest declared budget it survived its own load at
was **5 GiB**, and given 8 GiB of room while asked to stay inside 2 GiB its
anonymous peak was 4586 MiB — **2.24× the budget it was given.**

**Invariant II's first step closed most of that gap.** On build `9199e07`, with
label sets shared instead of copied per row and nothing else on either list built,
the gate is `UNDER_BUDGET` at **2 GiB** at 90–96% of it across three runs, red at
1792 MiB, and the same `--limit 8GiB` experiment reads **1913 MiB — 0.93×** rather
than 2.24×, at 19.7 k eps against 18.7 k. What is left is 6% of headroom at 2 GiB,
which is not a budget anyone should deploy on, and the rest of the invariant is
what buys it. [`MEMORY_BUDGET_GATE.md`](MEMORY_BUDGET_GATE.md) is both baselines
and their limitations.

### II. A line's bytes are copied a bounded number of times, and label sets are never de-shared

**This is the load-bearing one.** It was written as the middle of three and
[`MEMORY_ATTRIBUTION.md`](MEMORY_ATTRIBUTION.md) moved it to the front: the
churn it describes is what makes invariant I unachievable, because an allocator
asked for 52 GB in 33 seconds keeps most of it. 72% of that traffic is the read
path returning a hundred rows, and 20% is a flush path spending 26 kB and 356
allocations on a 368-byte line. The measurement also confirmed the label clone
in situ at 1,326 bytes per row against the microbenchmark's 1,345 — the two agree
to 1.4%, which is why this invariant can be worked on with a bench rather than a
container.

Target: **three copies** from socket to disk — the request body, the WAL write
buffer, the Arrow column buffer. Today it is six before the `204` and twelve
before the bytes land in Parquet.

Two of the six are free:

- `push_inner` iterates `&push_req.streams` and clones each line
  (`ingest.rs:247`) while `push_req` is separately owned and could be consumed.
- `frame_tenant_record` copies the entire payload to prepend a 7-byte prefix
  (`journal/mod.rs:90-101`), which `writer_loop` then copies again into the batch
  buffer (`writer.rs:304-316`). The prefix belongs in the batch buffer.

The JSON and OTLP paths are worse: they re-encode a Loki `PushRequest` for the
WAL so replay has one decoder (`proto.rs:99-127`). The single-decoder property is
right and worth keeping; materializing a whole second message with per-line and
per-label clones to get it is not.

**Label sets were the bigger term, and this part is done.** The memtable holds one
`Labels` per stream — correct, and its byte accounting reflects it
(`memtable.rs:57-67`). Then `Row::from_entry` cloned the whole `BTreeMap` **per
row**, `encode_stream_index` cloned every name and value again per row per label
because `BTreeMap::entry` wants an owned key, and `write_meta` cloned the set a
third time. The read path mirrored it: reader → registry → execution materializes,
cloned per row, and sorts, **three times** — and a **fourth** on the metric path,
which [`MEMORY_ATTRIBUTION.md`](MEMORY_ATTRIBUTION.md) found by measuring, on an
async worker thread outside the `spawn_blocking` and outside every budget, 203 MiB
at its high-water. A **fifth** turned up while removing them: `sample_value` built
the whole field set per row to read the one field an `unwrap` names.

`Labels` is now reached through `SharedLabels = Arc<Labels>` from the memtable to
the query result, so a stream's label set is allocated once and every row points
at it. The two writer sites that needed owned keys are keyed by borrows instead,
and the reader interns one label set per distinct stream per scan. **The claim
that this was the largest single payoff in the repository held, and it is the one
number that says so:** the gate went from `OOM_KILLED` at 2 GiB to `UNDER_BUDGET`
at 2 GiB, and the measured overshoot from 2.24× to 0.93×
([`MEMORY_BUDGET_GATE.md`](MEMORY_BUDGET_GATE.md)).

On the benches, where the number that had to move was bytes per row:
`rows_from_snapshot` allocated **1 457 / 1 505 / 1 569 bytes and 11 / 17 / 27
allocations per row** at 2 / 5 / 10 labels, identically at 1, 256 and 8192 streams
— the clone was per row, not per stream — and now allocates **823.4 bytes and 6.00
allocations per row at every point of that sweep**, with peak live flat at
13.85 MB instead of 26.0–28.3 MB. The part scan went from **3796 / 3955 / 4078
bytes and 11.2 / 19.3 / 27.4 allocations per row** to **3167 / 3263 / 3337 and
6.19 / 6.32 / 6.45**, with peak live flat at 31.0 MB instead of 54.0–58.4 MB.
Nothing in either bench got slower, including the two-label case where an atomic
increment replaces a small bulk allocation and a regression was the expectation:
`rows/from_entry` is 1.82× faster at two labels and 4.40× at ten.

**Three things that did not remove, and one of them is now gone too.** The three
materialize-and-sort hops on the read path **are one materialization of `limit`
rows and one sort over them**, which is invariant III's streaming executor below
and the step that closed it. What is still there:
`process_entry_with_labels_cancellable` clones the label set into a mutable field
map **per row**, for every query including one with no pipeline stages, and
undoing that is a copy-on-write field view across the whole pipeline rather than a
type change — the bound reduced how many times it is called, not what a call
costs. And `Row::materialized_bytes` and `estimated_log_entry_memory_bytes` still
charge every row for label bytes it now shares, so merge sizing and the query
memory ceiling are conservative by the rows-per-stream factor — a meter to fix
with the rest of the metering, not a limit to loosen on the way past.

Also in this invariant: **one decode, one sort, one parse.** `sort_rows` runs
globally and then again per partition (`part/format.rs:22`, `:91`), and
`encode_blooms` runs the JSON and logfmt parsers over every line twice — once to
size the filter, once to fill it (`format.rs:335-341`, `:360-366`).

### III. Query cost is bounded before the scan starts, and pruning goes as deep as the format allows

**The worst violation was the most common query, and it is fixed.** When a query
had any stage beyond a line filter — `| json`, `| logfmt`, a field filter — the
scan limit was set to `usize::MAX`, because the limit could not be applied before
the pipeline ran. So `{app="x"} | json | status="500"` with `limit=100`
materialized every matching row in the window, up to the 512 MiB ceiling, and then
threw almost all of it away. That was the shape Grafana sends most, and the shape
this engine claims to be good at.

Execution now **streams**. `PartReader`, `PartRegistry` and `MemTable` offer rows
one at a time to a `RowSink` (`part/sink.rs`), and the executor's sink
(`log_scan.rs`) is a bounded top-K collector that runs the pipeline as each row
arrives. So a pipeline stage no longer defeats the limit: the scan reads until it
holds `limit` rows that *survived* the pipeline. From the sink's frontier — the
timestamp a row must beat once the sink is full — the scan skips a whole part on
its tenant segment's span without opening it, a whole row group on
`row_group_min_ts`/`max_ts` without touching the Parquet body, and the rest of a
part on the first row past the frontier, which is sound because a tenant's rows
are ordered inside a part. Ties go to whichever row arrived first, so the answer
is exactly what a stable sort plus `truncate(limit)` gave. That also deleted the
triple-materialize of invariant II: there is nothing left to materialize but the
`limit` rows, and one sort over those.

**Measured on `benches/query.rs`, limit 100 over 202 000 rows in the window.**
`| json | field=` went from processing every row in the window — 200 250 lines,
703 MB of allocation traffic, 26.8 MB of peak live, 308.78 ms — to **3 130 lines,
43.9 MB, 5.81 MB and 13.067 ms**, and label-only from 69 178 lines and 178 MB to
**187 lines and 0.37 MB**. On the gate the margin at 2 GiB widened from 6% to
17–22% and the workload's own anonymous high-water fell from 1913 to 1659 MiB, but
the smallest surviving budget is still 2 GiB
([`MEMORY_BUDGET_GATE.md`](MEMORY_BUDGET_GATE.md)). Two figures the old text
quoted from [`MEMORY_ATTRIBUTION.md`](MEMORY_ATTRIBUTION.md) — 111 MiB of live
materialization against 17 MiB for the two shapes, and 270 MB per label-only query
— were build `50190cf`'s and are not comparable to these; the attribution has not
been re-run and its tables still describe an engine that no longer exists.

**And a limit only helps where a limit binds.** On the comparison bed's own
dataset the `json_field` figure at `limit=20 000` is unchanged at 16 384 lines,
because 20 000 over ~1 250 matching rows never fills the sink. At Grafana's
default 100 it is 5 528–11 072, against Loki's 1 251 for the same answer. The
residue is not the limit: Loki's index takes it to one stream's chunks, while a
row group here interleaves every stream in it and each of their rows is decoded
and filtered. That is the pruning below, and it is now the largest remaining term
on the shape the claim rests on.

**Pruning we index for and then do not use:**

- **No projection pushdown.** `ProjectionMask` appears nowhere in the tree. Every
  label column and the `structured_metadata` JSON blob is decoded even for
  `count_over_time({app="x"}[5m])`, which needs one column.
- **No predicate pushdown, no Parquet statistics, no Parquet blooms.** Filtering
  is a row loop (`reader.rs`, `scan_batch`) that allocates the line with
  `to_string()` before testing the filter that will reject it.
- **`|~` never prunes.** `bloom_prune` matches only `LineFilter::Contains`, so
  `|~ "error.*timeout"` gets no trigram pruning even though both literals are
  indexable. Extracting required literals from the regex closes this.
- **Sequential parts.** Within one query, parts are scanned one at a time, and an
  object-store restore holds a scan permit while doing pure network I/O
  (`execution.rs`).
- ~~**Footer re-parsed per row group.**~~ Closed by the streaming rewrite: the
  footer is parsed once per part scan, and the reader clones an `Arc` of it per
  row group and per backward window. Caching it across *scans* is still open.

What is already right and must not regress: row groups aligned to tenant
boundaries (`format.rs:221-238`), which is what makes every row-group-granular
structure selective in a shared part; the exact-field blooms with canonical
numeric forms, a `label_format` barrier, and absence used as a *positive*
prune — since BTF5 one sub-bloom per 1024-row window, so an admitted group
decodes only the windows whose filters admit the token (`ast.rs:154-189`,
`part/mod.rs`); and the single read funnel that `volume`, `patterns`,
`detected_fields` and `tail` are all expressed in terms of rather than given
private scans.

---

## Ingest is OTLP, and it arrives through collecty

**Logs, traces, and — with M14 — metrics arrive over OTLP and nothing
else, and they arrive through collecty and nothing else.** The Loki push endpoint
was removed first; the Loki *query* API followed with the read-path decision
(issue #3). An ingest protocol and a query protocol are separate decisions that
happened to share a name, and both went the same way once the viewer stopped
being Grafana: the engine answers the fn0 control plane and curl-driving agents
over its own flat-filter API ([`QUERY_API.md`](QUERY_API.md)).

**Why.** signy has one intended consumer, and everything it would store
already arrives as OTLP there: guest traces, guest logs (stdout, converted to
OTLP log records by the host), and the worker's own telemetry. Two things in that
deployment are not OTLP today, and only one of them stays out of signy's
business: systemd journal currently goes by Loki push, which the collector can
convert, because `otelcol.receiver.loki` exists precisely for that. Node
metrics went by Prometheus remote_write while this engine did not do metrics;
that sentence is retired by M14 (issue #8,
[`M14_IMPLEMENTATION_PLAN.md`](M14_IMPLEMENTATION_PLAN.md)) — metrics become
the third signal, and they arrive the way everything else does, as OTLP, with
the collector converting what the node exporters emit.

**What it buys is larger than one endpoint.** An OTLP record is re-encoded into
a Loki `PushRequest` before it reaches the WAL, so that replay has a single
decoder (`proto.rs`). That is a whole second message materialized with a clone
per line and per label, then serialized, then framed, then batched — five copies
for the WAL alone, on the exact path the consumer uses. The re-encode exists
only because two ingest protocols had to converge on one WAL record. With one
protocol it has nothing to do, and [invariant II](#ii-a-lines-bytes-are-copied-a-bounded-number-of-times-and-label-sets-are-never-de-shared)
gets the copies back.

**What stays.** The JSON parser stage stays and is not deprecated — spelled
`parse=json` on the first-party API now. A guest's `println!` becomes the
*body* of an OTLP log record as a plain string, and nothing in that chain
parses it — so a guest that logs JSON still needs a parser stage at query time.
What changes is that this stops being the headline; see the claim below.

**One producer, one route.** The OTLP push endpoints and the OTLP gRPC
services are gone too, leaving `POST /signy/api/v1/collect` as the whole write
surface. This is the same decision as the paragraph above, taken one step
further: an engine an application can push to directly has no queue in front of
it, so every refusal it gives — draining for a machine replacement, flush
behind, disk low — is telemetry lost unless that application happens to hold it.
collecty's append-only disk queue is exactly the thing that holds it, and a
queue only helps when nothing can go around it. Leaving the push routes in as a
convenience would have meant an ingest path whose durability story is "the SDK
retried, probably".

What it costs is named rather than hidden. **OTLP JSON is no longer accepted
anywhere in the product** — only the push routes decoded it, and collecty
refuses it with `415` because it never decodes a payload. **An application
exporting OTLP over gRPC has nowhere to send it**; collecty takes OTLP/HTTP
1.1, protobuf, uncompressed, and that is the supported wire for the whole
stack. Both are exporter configuration, and both are the price of one ingest
path rather than three.

**A consequence worth knowing before it surprises someone.** Which OTLP
attributes become stream labels is a schema decision, and signy currently
uses *Loki's own default promotion list* (`otlp_log.rs`,
`PROMOTED_RESOURCE_ATTRIBUTES`), so a collector configured for Loki produces the
same streams here. The corollary is that a collector which moves a source to
OTLP without mapping its labels to OTel semantic conventions loses index pruning
in **both** systems identically: `{unit="fn0-worker"}` stops being a label
lookup and becomes a structured-metadata filter. The fix is a `transform`
mapping on the collector — `unit` to `service.name` and so on — and it is one
piece of work that serves both.

---

## The claim, and what would falsify it

The differentiator is invariant III applied to one query shape — and **which
shape is the right one changed when ingest became OTLP.**

The claim used to be about `| json | field="value"`: a log line written as JSON
text, parsed at query time. That is a real shape and it is still supported, but
it is not the one the consumer sends. Over OTLP the fields that matter arrive as
**attributes** — `project_id`, `unit`, `trace_id`, `span_id`, severity — and
signy puts them in `structured_metadata`, which the exact-field bloom
already indexes. The query is `| trace_id="x"`, with **no parser stage at all**.

So the claim is now about that shape, and it is a better claim, because it is
where the three systems genuinely differ:

* **signy** indexes structured metadata into a per-row-group bloom.
* **Loki** stores structured metadata and **does not index it** — the filter is a
  scan.
* **VictoriaLogs** turns OTLP attributes into columns.

`| json | field="value"` remains measured, as `json_field` and
`json_field_rare` in [`COMPARISON.md`](COMPARISON.md), because a guest that
prints JSON produces exactly it. It is no longer what the engine is *for*.

The claim is therefore stated as a falsifiable one:

> At an equal container memory limit, on the same corpus and the same machine,
> signy answers an attribute-equality query over structured metadata — the
> shape an OTLP attribute produces, `attr=field=value` over its own query API —
> in materially less time than Loki, which does not index it, and not
> materially worse than VictoriaLogs, which columnizes it, without giving up
> ingest throughput or disk footprint.

**Both halves hold now, on the wire the claim was always about — and the
VictoriaLogs half holds past its own wording.** The bed ingests OTLP on all
three systems since 2026-08-02 — the same protobuf body at each engine's own
`/v1/logs` spelling — and every ratio below printed only after all three
pairs agreed on all 168 queries of every shape. The claim shape: Loki at
**0.00x** (0.22 ms against 78.8), VictoriaLogs at **0.23x cold / 0.16x warm**
as of 2026-08-06. "Not materially worse" undersells the evening's state
**at the bed's 150 k-row dataset**: every log shape — broad selector, line
filter, parsed field, both rare lookups, the trace window — answers under
VictoriaLogs' time on both the cold and the warm pass (`label_only`
0.90x/0.61x is the closest race; the rest sit 0.00x–0.88x), with
`sum(rate(...))` at 1.06x, a 0.02 ms gap at the HTTP jitter floor.

**That sentence is true of one dataset size, and a ten-times run says so**
([`COMPARISON_LARGE_CORPUS.md`](COMPARISON_LARGE_CORPUS.md), 1.5 M rows,
2026-08-12). The two halves of the read path scale in opposite directions and
the whole-shape sweep does not survive it:

* The **pruned** shapes barely move and the gap *widens*: `metadata_rare` — the
  claim's own shape — goes 0.23 → 0.39 ms for ten times the data, against
  VictoriaLogs 0.19x → **0.06x**, which is 15.4x cold and 17.8x warm. Ten times
  the rows do not make a bloom read ten times as much; they do make a column
  scan.
* The **scanned** shapes lose: `line_filter` 0.47x → **1.27x**, `json_field`
  0.91x → **1.33x**, `sum(rate(...))` 1.09x → **3.66x**. `label_only` stays a
  win at 0.53x, and its warm advantage is gone — 15.9 → 69.4 ms, level with its
  own cold pass, which is the working set outgrowing the caches in one number.

So the claim holds where it was made, and holds harder: the indexed-structured-
metadata shape is further ahead at 1.5 M rows than at 150 k. What does not hold
is the sweep — "every log shape" is a statement about the 150 k dataset, and it
is scoped here rather than quietly left standing.

That document is generated by the same one command with the dataset as its only
change, which is also why it is a companion rather than a replacement — the
150 k table stays as published:

```
COMPARE_VERIFY_ROWS=1500000 \
COMPARE_OUT=target/compare-10x \
COMPARE_DOC=docs/COMPARISON_LARGE_CORPUS.md \
COMPARE_ARTIFACTS=docs/artifacts/m9-10x \
  compare/run.sh
```

The artifacts directory is part of that command, not a detail: the first
ten-times run left it at the default and wrote its JSON over the 150 k run's, so
for a day the 150 k document cited a directory belonging to a different run. A
second corpus takes a second directory, and each document now prints the one it
read.

Two things it reports that the reader should not skip. The Loki columns were
**withheld** on the first pass at this size — 32 of 168 answers disagreed,
every one because Loki attached its own `__stream_shard__` to the stream labels
once the streams were ten times larger, with identical row counts on both sides
— and that is now a **declared exemption by name**, the same class as
`detected_level`, reported with its count in the document. The re-run agrees
168 of 168 and the ratios print: signy is faster than Loki on all seven
shapes at 1.5 M rows, 0.00x–0.26x, and 1470x on the claim's own shape. And
`line_filter` degraded **super**-linearly, 3.5 → 53 ms for ten times the data —
which was the one number in the run a scan cost alone did not explain, and is
now explained: ten of the fifteen was a ten-times-bigger *answer*, and the scan
path's own scaling is 1.95x for 10x with the answer held fixed (`todo.md`).
It began at 12.6x when `structured_metadata` was a JSON blob parsed per row;
columnizing it, then page-level time selection, then the `_stream` ordinal
table brought it to 1.46x; what closed it was keeping the decode — a
selection-keyed cache of decoded row groups and narrow-pass outcomes on
immutable parts, plus window blooms tight enough (0.1% FPP) that a rare token
admits the window it is in and almost never another. The bed's own caveat is
recorded in `todo.md` round four: its rare-shape sequence repeats each
distinct query eight times, and on a first-ever issue the two engines are at
parity (1.01x/0.91x/1.34x across the three windows) — the repeat is where
signy is ~6x ahead, and repeats are what trace-lookup traffic is made
of. An earlier short run had signy *losing* the Loki half at 3.10x; that
number was this engine's own bounded-scan defect wearing a performance
costume, and it reversed the day the defect did. The honest wire also
resolved the `json_field_rare` pair the design intended: the same rare value
reached *through a parser* holds **0.25 ms flat** — under OTLP it keeps the
line unparsed like everyone else, pays the unpack at query time, and the
per-row-group bloom over the parsed field is the difference.

It is abandoned if the comparison shows Loki within noise on that shape despite
not indexing it, or shows signy losing on ingest or bytes-per-GB by enough
that the query win does not pay for it. Publishing the comparison means
publishing it when it loses — which it has already done once, on the shape this
claim replaced.

**What the previous claim's measurements said, kept because moving a target does
not retract them.** Two comparison runs, both in
[`COMPARISON.md`](COMPARISON.md) and its artifacts. The first published at 8 GiB
because signy was OOM-killed at 2 and Loki was not: `| json | field=` 1.49x
slower cold, `|=` 1.69x, `sum(rate())` 7.1x, winning only label-only at 0.36x.
The second published at 2 GiB, which both systems survived — that being the
change — and read 1.85x on `| json | field=`, 2.81x on `|=`, 4.50x on
`sum(rate())`, 0.57x on label-only, with a per-limit table showing the bounded
scan working: 282 676 lines read at a limit of 100 against 1 331 072 at 20 000.

Since that second run the rows have been ordered by stream before time, which a
short local run measured at 0.08x on label-only, 0.15x on `|=` and **0.47x on
`json_field`** — a shape that was losing. **That has not been confirmed in the
bed**, and one shape got worse in the same change (`json_field` backward, see
[`todo.md`](../todo.md)), so it is a signal and not a result.

None of that is retracted by this section. What it establishes is that
invariant III was unbuilt and is now half built, and that the number to beat is
written down for the shape that is no longer the headline.

---

## The metrics claim (M14, issue #8) — stated before the engine exists

Metrics are the third signal an fn0 console needs, and the axis this engine
can own there is the same one it claims for logs: the declared memory budget.
The Prometheus family's chronic failure mode is cardinality explosion —
per-pod, per-request labels minting unbounded new series until the process
OOMs or refuses queries at a hard limit. Nobody in metrics owns the axis "a
series-churn explosion inside a 2 GiB container degrades instead of dying."
On the axes VictoriaMetrics has spent a decade tuning — compression, ingest,
query throughput — parity is success and winning is not the plan.

The claim, in the same falsifiable form as the log claim above:

> At an equal container memory limit, on the same corpus and the same
> machine, signy answers the fn0 dashboard shapes — a windowed counter
> `rate` and its label-grouped `sum`, over its own flat-parameter API — not
> materially worse than VictoriaMetrics, which has spent a decade on exactly
> this; and when a series-churn workload fills the resident-byte budget,
> signy keeps its process below the memory watermark and returns `429` for a
> complete export with `Retry-After`. The collector's durable queue retries it
> after flush makes room, rather than the process slowing into an OOM or
> silently dropping samples.

The two halves differ by design, not by tuning. The steady half concedes
VictoriaMetrics its maturity — Gorilla-family sample encoding runs near the
encoding family's entropy limit, and there is no headroom worth chasing. The
churn half names the structural difference: VictoriaMetrics sizes its index
to the workload, signy sizes resident state to its budget — idle series leave
the index at a declared horizon, and the shared byte governor rejects a
complete export only when the process cannot safely reserve its projected
state. `max_active_series` is an opt-in emergency guard, not the normal
workload ceiling.

It is abandoned if VictoriaMetrics inside the same limit both survives the
same churn with at least the same sample acceptance *and* beats signy
materially on the steady shapes — then the budget axis bought nothing, the
metrics engine remains a convenience of the one-binary packaging rather than
a differentiator, and the document publishes the loss.

**The first run is in, and it decides neither half in this engine's favour**
([`COMPARISON_METRICS.md`](COMPARISON_METRICS.md), 2026-08-27, both engines
at 2 GiB).

*The churn half happened, and it is not yet a differentiator.* Offered
520 288 series, the earlier build refused 24 288 datapoints by name at its
then-default `max_active_series` of 500 000. The current build uses the
resident-byte governor and keeps the count guard off by default. VictoriaMetrics
accepted all 520 288 without refusing anything, peaking at **627 MiB against
signy's 1 038**. So the competitor held more series in less memory and
never had to degrade at all: "refuses rather than dies" is not a
differentiator against an engine that neither refused nor died. And the limit
that bound was a *policy* number, not the budget — 500 000 is a default the
plan admitted was a guess, and the memory gate that was supposed to calibrate
it against the real per-series cost has still not been run.

*The steady half is measured but not settled.* The first run's shapes
disagreed on values; that was diagnosed to two causes — VictoriaMetrics
returns a decimal approximation of the double it was given, and it scales a
window the samples only partly cover — and with the comparison rules
corrected all six shapes now agree. The ratios print, and they are not
flattering: `raw_range` 1.76x, `rate_range` 1.62x, `instant_alert` 2.14x,
`quantile_p99` 1.68x, with `agg_sum_by` the one win at 0.78x. But the verdict
is **decided by noise**: two consecutive runs of identical query code put
`rate_range` at 1.03x and then 1.62x, either side of the threshold, because
every number is a sub-millisecond HTTP exchange. No latency conclusion should
be drawn from this bed until it can measure at a scale where the work
dominates the round trip.

The claim is not abandoned: abandonment needs VictoriaMetrics to beat it
materially on the steady shapes, and one coin-flip run does not establish
that. What the runs establish is that the comparison is finally *possible* —
all six shapes agree — and that two things stand between it and meaning: an
instrument that can resolve sub-millisecond differences, and the calibration
of the ladder's default limit that was never done. Both are open in
[`todo.md`](../todo.md).

The design behind it is recorded in
[`M14_IMPLEMENTATION_PLAN.md`](M14_IMPLEMENTATION_PLAN.md); the bed that
produced the numbers is `compare/run_metrics.sh`, which regenerates the
document from result JSON and publishes it win or lose — as it has here.

---

## What is deliberately not built

- **No Loki, no LogQL, no Grafana compatibility.** The read-path decision
  (issue #3): the viewer is the fn0 control plane speaking the first-party API,
  and a compatibility surface with no remaining consumer was an obligation with
  no customer. The Tempo surface went with it, and the first-party trace API
  (M13, issue #7) is its replacement. If external demand ever materializes, a
  Grafana datasource plugin over the first-party API is the cheap insurance;
  the compat endpoints do not come back. The same decision extends to the
  metrics signal before it is built: **no PromQL** — M14's read surface is
  first-party flat parameters, for the same reason the Loki surface went.
- **No cluster, no replication, no in-process query UI, no separate index
  store.**
  Single writer is the design, and writer fencing enforces it. Every guarantee
  here — the manifest CAS commit, merge input revalidation, the flush
  transaction — rests on there being exactly one.
- **No DataFusion.** Recorded as "under consideration" in
  [`ARCHITECTURE.md`](ARCHITECTURE.md); closed here as rejected. It conflicts with
  invariant I — its memory is not accountable at arena granularity — and with
  invariant III, where the bounded top-K stream is a specific execution shape
  rather than a general one. The flat filter surface is small enough that a
  planner for it is smaller than the integration.
- **No TLS, no authentication.** Already decided; unchanged.
- **No per-tenant object paths.** Already decided on cost grounds; unchanged.
- **No format versioning.** Nothing on disk or on the wire carries a version, and
  nothing reads data written by an older build — not parts, trace parts, the
  manifest, the WAL, the compaction state, or the bloom container. There is no
  deployment, so there is never old data, and every compatibility path was code
  that could not run. It is not free to keep: the bloom container carried four
  formats, which made the reader hold three flags describing what each writer had
  been capable of and made `exact_field_bloom` an `Option` that was always
  `Some`. Changing a format is therefore just changing it; a stale local data
  directory is deleted rather than migrated.

  Two things that resemble versioning and are not: `object_store`'s
  `UpdateVersion`, which is the compare-and-swap ETag every durability guarantee
  here rests on, and `CARGO_PKG_VERSION` in `/metrics`, which is API surface.

---

## The ruler comes before the work

Every performance number in this repository was produced by a harness that
cannot be trusted, and the documents holding those numbers say so themselves in
places. Optimizing against them would reproduce them.

What is wrong, concretely:

- The harness is **single-connection, one request in flight, `Connection: close`
  per request** (`bin/load.rs:715`, `:732`). Every latency includes a TCP
  handshake; keep-alive is never exercised.
- **Coordinated omission is uncorrected.** The pacer keeps a nominal schedule
  (`bin/load.rs:184-191`) but the stopwatch starts at the *actual* send (`:195`),
  so when the server slows the harness stops issuing and never records the delay
  it owes. `offered_eps: 3000` against `achieved_eps: 478` in the one checked-in
  artifact is the offered rate being unreachable by construction.
- **Cardinality 1** — one hardcoded label set (`bin/load.rs:693`) — and lines
  padded with `"x".repeat(...)` (`:680`), i.e. near-zero entropy. The documents
  record that realistic lines compress 5.9× where this data compressed 31.5×
  ([`LOAD_RESULTS.md`](LOAD_RESULTS.md) §2) and the harness was not changed.
- **Reads never contend with writes** — queries fire inline in the push loop on
  the same thread (`bin/load.rs:221`).
- Percentiles over samples of 167 and 8; a "p95" over eight samples is the
  maximum.
- Not reproducible: the only run script `cd`s to a path that does not exist on
  this machine, one cited artifact is missing, the surviving one disagrees with
  the document citing it on both build revision and verdict, and a cited test
  name does not exist in the tree.
- **No benchmarks at all.** No criterion, no `benches/`, no CI. There is no
  performance regression detection of any kind.

So the first work is the ruler, and it is a precondition for the rest:

1. **Retire the numbers, keep the reasoning.** `LOAD_RESULTS.md` and
   `M7_LOAD_RESULTS.md` contain genuinely valuable narrative — refuted
   hypotheses, the terminal-sample lesson, the discovery that "merge disabled"
   runs all contained one merge. That survives. The tables do not.
2. **Microbenchmarks for the hot paths**: WAL append, memtable insert, flush row
   materialization, bloom construction and lookup, Parquet scan, LogQL
   evaluation. These are the regression gate.
3. **Rewrite the harness**: N connections with keep-alive, latency measured from
   *intended* send time, a realistic corpus with a cardinality knob and
   out-of-order arrival, reads concurrent with writes, RSS from `VmHWM`.
4. **The comparison bed**: Loki single-binary beside signy, same corpus,
   same machine, same container memory limit. Four query shapes — label-only,
   `|=` substring, `| json | field=`, `rate()` aggregation — plus ingest
   throughput, bytes on disk per GB ingested, and object-store request counts.
5. **CI**, because none of this holds without it.

Only then: the budget architecture (I), the copy elimination (II), the streaming
executor and pushdown (III).

---

## How to read a disagreement

If a measurement contradicts this document, the measurement wins and this
document is edited. If an implementation contradicts it, the implementation is
wrong. If a feature request contradicts "what is deliberately not built", the
answer is no, and the reason is that these three invariants are the product.
