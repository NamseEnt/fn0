# Where the memory actually goes

M9 left one number without an explanation: at a 2 GiB container limit, ingesting
1.2 M events at an offered 20 k eps with a ~5 qps read workload, **signy is
OOM-killed and Loki is not** ([`COMPARISON.md`](COMPARISON.md)), while
`signy_memtable_bytes` reported 111 MB. This document is the measurement
that says what the rest of it was.

It is a diagnosis, and every number in it is build `50190cf`'s. One of the things
it diagnosed has since been fixed — hypothesis 2, the per-row `Labels` clone —
and **nothing here was re-measured afterwards**, so this document describes an
engine that no longer exists and its tables should be read that way. The budget of
[`VISION.md`](VISION.md) invariant I is still not built — but the *gate* for it is,
because a fix needs something that can tell whether it worked, and it is what said
this one did:
[`MEMORY_BUDGET_GATE.md`](MEMORY_BUDGET_GATE.md) runs the engine at a declared
budget and compares peak cgroup `anon` against it. The reason for measuring
first is written into that invariant: its arena split — ingest 25 %, flush 15 %,
merge 20 %, query 30 %, sidecar 10 % — was a guess, and an accounting scheme
built on a guess produces a number that is believed and wrong, which is the
failure this repository has already had once with its RSS gauge.

**The headline is that the guess was wrong in a way no adjustment of the
percentages would fix.** At the moment the kernel killed the process, the
engine's live heap was 669 MiB out of a 2 GiB limit — a live-byte budget would
have read 33 % full. The largest single term in the anonymous footprint is
memory the process had already freed and the allocator had not returned.

---

## What this measurement does not establish

Stated first, in the discipline [`COMPARISON.md`](COMPARISON.md) sets.

* **It is not the Docker bed.** These runs use a native cgroup v2 scope on the
  same machine, with the same `memory.max`, the same `MemorySwapMax=0` (the
  bed's `memswap_limit == mem_limit`), and the same `anon` an OOM kill is
  decided on. It is a container of the same kind, not the same container.
* **I could not reproduce "ingest-only survived".** `COMPARISON.md` records that
  an ingest-only run reached exactly the 2 GiB limit without being killed. On
  this bed an ingest-only run **is** killed, at t≈49–60 s, and the kill
  coincides with the first merge tick. Both observations are recorded; neither
  is retracted. The difference is not explained.
* **glibc reports retained-free memory as one number.** `mallinfo2.fordblks`
  does not say how much of it is fragmentation the allocator cannot reuse and
  how much is simply untrimmed and would have been reused. Distinguishing them
  needs a run that reaches a steady state, and every 2 GiB run here dies first.
* **C-side allocations are not separable from allocator metadata.** zstd's
  compression contexts go through `malloc` directly, not through the Rust global
  allocator, so they are invisible to the arena tags and land in the gap between
  tagged-live and `mallinfo2.uordblks`. That gap is 96–112 MiB across the runs;
  it is an upper bound on zstd, not a measurement of it.
* **The instrument costs 16 bytes per live allocation**, which is 66–268 MiB in
  these runs and is reported as its own row rather than hidden. The
  uninstrumented and instrumented runs die at almost the same anonymous
  footprint (1717 MiB vs 1687 MiB), so the retained-free term simply shrinks to
  make room; but every per-arena figure below is a figure from an instrumented
  build.
* **Sampling is 4 Hz.** A spike shorter than 250 ms between samples is missed by
  the time series. The allocator keeps its own per-arena high-water marks, which
  do catch them, and those are reported separately and are explicitly *not*
  simultaneous with each other.
* **The early attribution runs used glibc only.** They did not compare an
  alternative allocator; later work moved the production binary through
  jemalloc and, since 2026-08-31, mimalloc. The historical glibc figures below
  must not be read as a measurement of the current production allocator.

---

## Reproducing it

```
M10_FEATURES=memprof scripts/run_memprof_local.sh base
```

`scripts/run_memprof_local.sh` is the one-command reproduction: it puts the
server in a `systemd-run --user --scope` cgroup at `MemoryMax` with swap off,
drives it with the comparison bed's ingest parameters and seed, and samples the
cgroup's `memory.stat`, the engine's gauges and the arena attribution into one
CSV at 4 Hz. Every knob is a default. The variants below are that script with
exactly one thing changed.

The instrument is `src/memprof.rs`, behind the **default-off `memprof`
feature**. With the feature off, `memprof::enter` is an `#[inline(always)]`
function returning a zero-sized guard and the process keeps the system
allocator; with it on, the binary installs an arena-tagging global allocator and
`/metrics` grows the `signy_memprof_*` families. `cargo test`
(437 + 24), `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`
are clean in both configurations.

**How the tag is assigned.** The label is a thread-local. A guard held across an
`.await` would tag another task's work, so every guard wraps a *synchronous*
region: the `spawn_blocking` closures the flush, merge, query and trace-scan
paths already run inside; the straight-line decode in `push_inner`, dropped
before the journal append; the journal's framing and batch buffer; the sidecar
decode in `PartReader::open_internal`; and `load_part`. Nesting restores the
enclosing arena, so blooms faulted in by a query are charged to `sidecar` —
which is what residency attribution wants, the sidecar outliving the query.
Everything else lands in `other`, which is a reported bucket, not a silent one.

Because the tag travels in a per-allocation header, `dealloc` refunds the arena
that allocated, so these are **live** bytes and not merely allocated bytes. Peak
anonymous memory is what an OOM kill is decided on, and cumulative allocation
says nothing about it — except that here it turns out to say almost everything.

Machine: Linux 6.12.57, 12 logical CPUs, 31 GiB RAM, glibc 2.41.

---

## The runs

All at a 2 GiB cgroup limit with swap disabled, ingesting 1.2 M events at an
offered 20 k eps over 8 connections, seed 1592598566, unless the variation says
otherwise. "plain" is the default build; "memprof" is `--features memprof`.

| run | build | variation | outcome | t | anon peak (MiB) |
|---|---|---|---|---|---|
| `baseline` | plain | — | **killed** | 37.8 s | 1717 |
| `ingest-only` | plain | `query_eps=0` | **killed** | 48.6 s | 1617 |
| `ingest-only-arena1` | plain | `query_eps=0`, `MALLOC_ARENA_MAX=1` | survived ≥62.6 s | — | 1225 |
| `plain-malloc-tuned` | plain | glibc single arena + trim | **killed** | 81.6 s | 1626 |
| `mp-base` | memprof | — | **killed** | 32.6 s | 1687 |
| `mp-ingest-only` | memprof | `query_eps=0` | **killed** | 59.6 s | 1404 |
| `mp-nomerge` | memprof | `SIGNY_MERGE_INTERVAL=3600s` | **killed** | 41.5 s | 1656 |
| `mp-label-only` | memprof | only `{app="x"}` queries | **killed** | 49.9 s | 1463 |
| `mp-json-only` | memprof | only `\| json \| field=` queries | **killed** | 38.2 s | 1677 |
| `mp-malloc-tuned` | memprof | glibc single arena + trim | **killed** | 86.1 s | 1578 |
| `mp-8g` | memprof | 8 GiB limit | survived | 77.6 s | 4937 |

"glibc single arena + trim" is
`MALLOC_ARENA_MAX=1 MALLOC_TRIM_THRESHOLD_=131072 MALLOC_MMAP_THRESHOLD_=131072 MALLOC_TOP_PAD_=0`.

Two things are visible before any attribution. **The query workload is not
required**: ingest alone is killed. **Nor is merge**: with merge pushed past the
end of the run, the process is still killed, sooner than ingest-only.

---

## The attribution

`mp-base`, at the last sample before the kill (t = 32.27 s). This is one
simultaneous snapshot, not a set of independent high-water marks.

cgroup `anon` = 1687.1 MiB. glibc's own view of its heap is
`arena` 1677.3 + `hblkhd` 36.0 = **1713.3 MiB**, i.e. 101.6 % of `anon`:
**the anonymous footprint is the C heap**, and thread stacks, BSS and non-malloc
mappings are inside the rounding.

| term | MiB | % of anon | measured by |
|---|---|---|---|
| **allocator-retained free memory** | **742.0** | **44.0 %** | `mallinfo2.fordblks` |
| merge — one group's read/filter/rewrite | 302.4 | 17.9 % | `memprof live_bytes{merge}` |
| flush — `Vec<Row>` + Parquet writer | 119.7 | 7.1 % | `memprof live_bytes{flush}` |
| ingest — memtable + in-flight decode | 119.4 | 7.1 % | `memprof live_bytes{ingest}` |
| query — scan, pipeline, sort | 116.7 | 6.9 % | `memprof live_bytes{query}` |
| sidecar — blooms + stream index | 6.0 | 0.36 % | `memprof live_bytes{sidecar}` |
| part metadata — `PartMeta`, `streams` | 3.8 | 0.23 % | `memprof live_bytes{part_meta}` |
| unlabelled (HTTP, runtime, metric-path rows) | 1.1 | 0.07 % | `memprof live_bytes{other}` |
| glibc chunk overhead + C-side (zstd) `malloc` | 112.0 | 6.6 % | `uordblks` − tagged − header |
| the instrument's own tag headers | 154.3 | 9.1 % | 16 B × live allocations |
| mmapped chunks, outside the arena above | 36.0 | 2.1 % | `mallinfo2.hblkhd` |

The first eight rows are `uordblks` = 935.3 and the first row is `fordblks` =
742.0; together they are `arena` = 1677.3 exactly, and `hblkhd` adds 36.0 on top.
The rows therefore sum to 101.6 % of `anon` — the allocator has reserved a little
more than has been faulted in, which is the only slack in this accounting.

**The engine's live data is 669.0 MiB — 39.7 % of the anonymous footprint it was
killed for.** A budget enforced against a sum of accounted live structures would
have reported the process a third full at the instant the kernel killed it. That
is the same class of error as the RSS gauge this repository already retired,
arrived at from the other direction.

The retained-free term is not an artefact of one run:

| run | anon (MiB) | glibc in-use | glibc free | free / anon | tagged live |
|---|---|---|---|---|---|
| `mp-base` | 1687.1 | 935.3 | 742.0 | 0.44 | 669.0 |
| `mp-ingest-only` | 1404.1 | 673.0 | 712.6 | 0.51 | 566.4 |
| `mp-nomerge` | 1655.9 | 673.1 | **1014.1** | **0.61** | 487.9 |
| `mp-label-only` | 1462.7 | 490.2 | 1003.0 | **0.69** | 360.1 |
| `mp-json-only` | 1676.5 | 1205.1 | 447.6 | 0.27 | 897.3 |
| `mp-malloc-tuned` | 1578.3 | 1461.4 | **19.5** | **0.01** | 1176.1 |
| `mp-8g` (survived) | 4936.8 | 1670.4 | **3330.4** | **0.67** | 1248.2 |

### Why the allocator retains it

Not because anything leaks. Because of the rate.

| run | GB/s allocated | M allocations/s |
|---|---|---|
| `mp-base` | 1.61 | 13.6 |
| `mp-nomerge` | 1.66 | 14.2 |
| `mp-json-only` | 1.37 | 15.1 |
| `mp-8g` | 1.46 | 14.3 |

`mp-base` allocated **52.3 GB in 32.6 s across 444 million allocations**, while
the offered data rate was 7.4 MB/s of log lines — a **217× amplification** of
allocation traffic over the bytes being stored. glibc gives each contending
thread its own arena (up to 8 × cores), each arena grows to its own transient
high-water, and `free()` returns pages to the kernel only from the top of a
heap. At 13–15 million allocations per second across twenty threads, the
steady state of that policy is a heap the size of the transient peak.

Where the traffic comes from, as a share of bytes allocated:

| arena | `mp-base` | `mp-ingest-only` | `mp-nomerge` | `mp-label-only` | `mp-json-only` |
|---|---|---|---|---|---|
| query | **71.9 %** | — | **76.8 %** | 56.6 % | **70.3 %** |
| flush | 19.7 % | **38.2 %** | 17.7 % | 17.3 % | 17.7 % |
| merge | 2.8 % | **45.7 %** | — | 19.6 % | 6.6 % |
| ingest | 2.3 % | 6.8 % | 2.3 % | 2.7 % | 2.7 % |
| wal | 1.0 % | 2.9 % | 1.0 % | 1.2 % | 1.2 % |
| other | 2.2 % | 6.1 % | 2.3 % | 2.5 % | 1.4 % |
| sidecar + part metadata | 0.0 % | 0.2 % | 0.0 % | 0.1 % | 0.0 % |

Per unit of work, exactly:

* **Flush allocates 22.6–35.0 kB and 313–470 allocations for every row it
  writes**, against a 368-byte average line (441 571 512 line bytes over
  1 200 000 events, the corpus `COMPARISON.md` publishes). That is a **61–95×
  byte amplification per row.**
* **A query allocates 260–428 MB and 1.2–3.8 million allocations**, and returns
  at most `limit = 100` rows.

### What this does to the published comparison

`COMPARISON.md` reports, at the 8 GiB limit both systems survived, an anonymous
ingest peak of **3936.2 MiB for signy against 1294.3 MiB for Loki**, and
that row is the honest thing to compare because it excludes page cache.

The 8 GiB run here reaches an anonymous peak of 4936.8 MiB with a **live heap of
1248.2 MiB and 3330.4 MiB of retained-free** — 67 %. On that split, the 3.0×
gap the comparison publishes is mostly a statement about two allocators rather
than about two engines: Go's runtime returns freed spans to the operating system
on a timer, and glibc's malloc does not. signy's *live* working set at
8 GiB, 1248 MiB, is within noise of the 1294 MiB Loki peaks at.

That does not make the comparison wrong — the kernel kills on `anon`, not on
live bytes, and signy is the one that got killed at 2 GiB. It makes it a
different finding from the one it reads as, and the same is true of the disk and
latency rows only in reverse: none of them changes.

### The control: take the retention away

`MALLOC_ARENA_MAX=1` with forced trimming drives `fordblks` from 742–1014 MiB to
**19.5 MiB**, and `anon / live` from 2.5–4.1 down to **1.34**. The effect on
survival is large and is not a fix:

| | plain default | plain, glibc tuned |
|---|---|---|
| time to OOM (ingest + query) | 37.8 s | **81.6 s** (2.16×) |
| time to OOM (ingest only) | 48.6 s | **survived ≥ 62.6 s** |

Once the allocator stops hiding it, the engine's own live memory is what kills
the process: at `mp-malloc-tuned`'s last sample the live heap is **1176 MiB**, of
which **flush 721 MiB** and **ingest 375 MiB**. So the retention is the largest
term, and removing it exposes a second one that is also unbounded.

---

## The hypotheses, one at a time

### 1. `normal_scan_limit = usize::MAX` — **confirmed for live bytes, and it is not the largest term**

`query/execution.rs:102-106` sets the scan limit to `usize::MAX` whenever a query
has any stage beyond a line filter. Running the identical workload with only
`{app="x"}` queries and then with only `{app="x"} | json | field="v"` queries:

| | label-only | `\| json \| field=` |
|---|---|---|
| query arena, live at the anon peak | **17.2 MiB** | **111.2 MiB** (6.5×) |
| query arena, live high-water | **24.9 MiB** | **152.8 MiB** (6.1×) |
| allocations per query | 1.19 M | 3.56 M (3.0×) |
| bytes allocated per query | 270 MB | 320 MB (1.2×) |
| time to OOM | 49.9 s | 38.2 s |

So the limit does bound the materialization: a shape it applies to holds a
seventh of what a shape it does not applies holds. **But the label-only shape
still allocates 270 MB and 1.19 million allocations per query for 100 returned
rows**, and query is still 56.6 % of all allocation traffic in that run. Killing
`usize::MAX` removes roughly the query arena's live term and about half its
allocation traffic. It does not remove the query path as the dominant source of
churn, because most of that churn happens in the scan, before any limit applies
(hypothesis 3, and `reader.rs:727` allocating the line before the filter that
rejects it).

### 2. `Row::from_entry` cloning `Labels` per row — **confirmed, and it is the largest *live* term once the allocator is honest**

**Fixed, and the fix is the strongest evidence this hypothesis was right.** Build
`9199e07` shares one label set per stream through `Arc<Labels>` from the memtable
to the query result. On the controlled side `rows_from_snapshot` went from 1 345 to
**823 bytes per row live** and from 17 to **6 allocations per row**, flat across
2, 5 and 10 labels where it used to grow. On the gate — the only instrument that
answers invariant I — `--budget 2GiB` went from `OOM_KILLED` at t≈49 s to
`UNDER_BUDGET`, and the measured overshoot from **2.24× to 0.93×**
([`MEMORY_BUDGET_GATE.md`](MEMORY_BUDGET_GATE.md)). **Everything below in this
document is still build `50190cf`'s and has not been re-measured**, including the
721 MiB flush figure this hypothesis rests on, the 44% retained-free share, and
the 203 MiB of hypothesis 8. A second run of this document is what would say where
the memory goes now; scaling its tables by the bench's factor would be the error
it exists to prevent.

Two independent measurements agree.

*In situ*: at `mp-malloc-tuned`'s peak the flush arena holds **721.0 MiB across
10 394 910 live allocations** — 72.7 bytes per live allocation — while
`signy_memtable_bytes` reports 217.9 MiB for the buffer still filling. The
materialized `Vec<Row>` is **3.3×** the accounted memtable it was built from, and
**1.9×** what the ingest arena holds for the same rows.

*Controlled*: the M8 bench (`cargo bench --bench rows`) puts
`rows_from_snapshot` at **1 503 bytes allocated and 17 allocations per row** at
5 labels, with a peak live of **1 345 bytes per row**. The in-situ figure derived
from the same run — 721 MiB over a snapshot whose accounted size was ~218 MiB,
i.e. ~570 k rows — is **1 326 bytes per row**. The two agree to 1.4 %.

Label cardinality does not change it: the bench's amplification is identical at
1, 256 and 8192 streams, because the clone is per *row*, not per stream.

### 3. Triple materialize-and-sort on the read path — **confirmed as churn, not as residency**

`reader.rs:1041` → `part_registry.rs:628` → `execution.rs:202` each materialize
and clone per row. The residency this produces is bounded and modest — the query
arena holds 111–129 MiB in the simultaneous snapshots and has a high-water of
242 MiB, at 4 concurrent queries. What it produces
instead is **1.2–3.8 million allocations per query**, and the query arena is
57–77 % of all allocation traffic in every run that has queries. Given that
allocator retention is 44–69 % of the anonymous peak and is a function of
allocation traffic, this hypothesis is a first-order term in the OOM — via the
allocator, not via anything it holds.

The label-only run isolates it: with the pipeline stages removed and the scan
limit therefore in force, per-query allocation only falls from 327 MB to 270 MB.

### 4. `entries_bytes` under-reporting — **confirmed at 1.70–1.79×, and it is ~7 % of the problem**

`memprof live_bytes{ingest}` divided by `signy_memtable_bytes`, measured
every 250 ms across every run:

| run | ratio at peak | max observed |
|---|---|---|
| `mp-base` | 1.72 | 1.75 |
| `mp-ingest-only` | 1.72 | 1.79 |
| `mp-nomerge` | 1.73 | 1.74 |
| `mp-label-only` | 1.73 | 1.78 |
| `mp-json-only` | 1.70 | 1.75 |
| `mp-malloc-tuned` | 1.72 | 1.92 |

Consistent with the 1.4–2.8× band `VISION.md` states, at the low end of it for
this corpus. The ingest arena also holds in-flight decode buffers, so this is an
**upper** bound on the memtable's own under-report.

Quantitatively it is a small part of the gap, exactly as the milestone
suspected. `MAX_MEMTABLE_BYTES = 256 MiB` is really ~440 MiB on this corpus; the
ingest arena's live peak across every 2 GiB run is 119–375 MiB, i.e. **7.1–24 %
of the anonymous peak.** The meter must be fixed before a budget can be built on
it, and fixing it recovers none of the missing memory.

### 5. In-flight push bodies outside the accounting — **refuted at the measured concurrency**

The premise is wrong in one place and unreachable in another. `Journal::append`
awaits its completion oneshot (`journal/writer.rs:118-137`), so an in-flight push
holds exactly one slot of the 4096-deep writer channel per concurrent request;
the channel cannot fill ahead of the HTTP concurrency. At the bed's 8
connections and ~37 kB decompressed bodies that is **~0.3 MiB** — below the noise
floor of every measurement here. The whole ingest arena, memtable included, is
119.4 MiB in `mp-base`.

The unboundedness is real (`in-flight requests × 64 MiB` still has no limit) and
belongs in the budget. It is not where this memory went.

### 6. `rows_from_snapshot` and the global sort outside `spawn_blocking` — **refuted as a memory term**

`flush.rs:233` runs before the `spawn_blocking` at `:253`. This is a real defect
— an O(n log n) pass over a full snapshot blocking an async worker — but it is a
*latency* defect. The bytes are identical either side of the boundary, and the
arena tag confirms it: the guard covers both regions and the flush arena's live
curve has no step at the hand-off. Nothing about moving it changes the 721 MiB.

### 7. Sidecar residency and `PartMeta::streams` — **refuted at this part count, and the engine's own gauge is honest**

| run | parts | sidecar live | part metadata live | combined, % of anon |
|---|---|---|---|---|
| `mp-base` | 28 | 6.0 MiB | 3.8 MiB | **0.6 %** |
| `mp-ingest-only` | 55 | 8.9 MiB | 7.3 MiB | **1.2 %** |
| `mp-8g` | 24 | 5.7 MiB | 3.3 MiB | **0.2 %** |

`signy_part_sidecar_resident_bytes` reported 5.5 MiB where the allocator
measured 5.7 MiB — **the existing gauge is accurate to 4 %**, which is worth
recording because almost nothing else here is.

The sidecar arena's *high-water* is larger than its residency — 18.4 MiB in
`mp-8g` against 5.7 MiB resident — because a merge opens a reader per input part
and drops them again. That is transient, it is charged to the right arena, and it
is still under 0.4 % of the anonymous peak.

Per part: **~240 kB of sidecar and ~140 kB of `PartMeta`**, both linear in part
count as `VISION.md` says. The concern is correct and the magnitude is not:
`VISION.md` gives sidecars 10 % of the budget, which at 2 GiB is 205 MiB, or
about **540 parts** — a number this workload does reach in a longer run, but not
before the process has died of something else three times over. Sidecars belong
inside the budget with a share derived from the per-part cost, not with a share
of 10 %.

### An eighth term nobody proposed: a fourth per-row `Labels` clone, on the metric path, outside every arena

`other` — everything with no guard around it — is the bucket that says whether
the instrument missed something. It did.

| run | query shapes | `other`, live high-water |
|---|---|---|
| `mp-label-only` | `{app="x"}` only | **1.9 MiB** |
| `mp-json-only` | `\| json \| field=` only | **2.0 MiB** |
| `mp-nomerge` | the bed's mix, `rate()` weight 2 | **67.3 MiB** |
| `mp-base` | the bed's mix | **84.8 MiB** |
| `mp-8g` | the bed's mix | **203.2 MiB** |

It tracks one query shape exactly, and `query/metrics.rs:134-155` is why.
`evaluate_metric_query` runs the unified scan with a limit of
`max_metric_rows + 1` — **1 000 001 rows** at the default, not the API's limit —
and then, on the async worker thread *before* the `spawn_blocking` at `:158`,
walks every returned row and builds `entries: Vec<(Labels, LogEntry)>` with
`stream.labels.clone()` **per row**.

`VISION.md` invariant II names three materialize-and-clone hops on the read path
(`reader.rs:1041`, `part_registry.rs:628`, `execution.rs:202`). This is a
**fourth**, it is specific to metric queries, it is outside `spawn_blocking`, and
it is outside every arena a budget would define. It is also, at 203 MiB, larger
than the sidecar term the arena table gave 10 % of the budget to.

---

## What the arena boundaries should be

### First, what the measurement says about the model itself

**A budget denominated in live bytes cannot deliver invariant I on this
allocator.** At the instant of the kill the live heap was 669 MiB of a 2 GiB
limit; at `mp-8g`'s peak it was 1248 MiB against an anonymous footprint of
4937 MiB. Any arena scheme that sums accounted live structures and compares the
sum to the container limit would have reported headroom at every moment,
including the moment of death. That is precisely the failure mode this phase
exists to avoid.

Two things follow, in order:

1. **The process must make its own anonymous footprint track its live bytes
   before an arena means anything.** Measured: `MALLOC_ARENA_MAX=1` plus trim
   thresholds takes `anon / live` from 2.5–4.1 to 1.34 and more than doubles
   time-to-OOM. That is a `mallopt` call at startup, or an allocator whose heap
   decays (jemalloc's `dirty_decay_ms`, mimalloc), or the arena-tagging allocator
   promoted into production so the budget is enforced at the allocation site.
   Whichever is chosen, the ratio it achieves is the multiplier every arena
   share must be divided by, and it must be published beside the budget.
2. **The budget must be validated against `anon` from the cgroup**, not against
   the sum of its own arenas. `VISION.md`'s verification — "a test that runs the
   engine at a declared budget and asserts peak RSS stays under it" — is the
   right test, and it is the *only* thing in the invariant that would have
   caught this. **It exists now**, and it is the first thing built after this
   measurement rather than the last: `src/bin/memory_gate.rs`, with its baseline
   and its four outcomes in [`MEMORY_BUDGET_GATE.md`](MEMORY_BUDGET_GATE.md). At
   the 2 GiB this document is about it is red, and the smallest declared budget
   the engine currently survives its own load at is **5 GiB**.

### Second, the shares

Live high-water per arena, the maximum over every 2 GiB run, read from the
allocator's own per-arena peak counters. These are **not** simultaneous with each
other; the simultaneous composition is the attribution table above. Their sum,
2214 MiB, is what the engine would need if they ever did coincide — on a 2 GiB
container.

| arena | max live seen | share of the sum | `VISION.md` guess | verdict |
|---|---|---|---|---|
| merge (one group) | **771 MiB** | 34.8 % | 20 % | too small, and its own ceiling is 1 GiB |
| flush | **721 MiB** | 32.6 % | 15 % | **far too small** |
| ingest | **378 MiB** | 17.1 % | 25 % | about right |
| query | **242 MiB** | 10.9 % | 30 % | too generous |
| metric-path materialization | **203 MiB** | (at 8 GiB) | not an arena | **missing entirely** |
| sidecar + part metadata | **17 MiB** | 0.8 % | 10 % | **wrong by an order of magnitude** |

Recommended, with the evidence for each:

| arena | share of budget | at 2 GiB | why this number |
|---|---|---|---|
| ingest | **20 %** | 410 MiB | Live high-water **378 MiB**, and the meter must first be corrected by the measured **1.72×** so `max_memtable_bytes` means what it says (256 MiB is really ~440 MiB). This share fits the measurement as it stands. |
| flush | **25 %** | 512 MiB | Live high-water **721 MiB** — it does *not* fit, and that is the point: `rows_from_snapshot` materializes the whole snapshot at **3.3× the accounted memtable** and **1 326–1 345 bytes per row** by two independent methods. The share is a target the flush must be made to meet by streaming, not a description of it. |
| merge | **25 %** | 512 MiB | Live high-water **771 MiB** in one group rewrite of 55 parts. Today's `merge_max_memory_bytes` default is **1 GiB — half the whole container** — and is not derived from any number the operator gave. Groups already split; the split must be sized from the budget. |
| query | **25 %** | 512 MiB | Live high-water **242 MiB** at 4 concurrent queries, **plus** the 203 MiB of metric-path materialization that is outside the arena today. Today's effective ceiling is `8 × 512 MiB = 4 GiB`, twice the container, which is the knob-product `VISION.md` already calls out. |
| sidecar + part metadata | **5 %** | 102 MiB | Measured **~240 kB of sidecar and ~140 kB of `PartMeta` per part**, so 5 % is a real ceiling at about **270 parts** — a count this workload reaches. 10 % is unearned by an order of magnitude; the share should be derived from the per-part cost, which is now measured. |

**Ingest and flush cannot be sized independently, and this is the finding that
breaks the table's shape.** The flush arena holds a *copy* of the ingest arena's
contents — `rows_from_snapshot` materializes the whole snapshot — so the flush
share is not a free parameter: it is measured at 3.3× the accounted memtable and
1.9× the ingest arena's live bytes, and the two peak together (375 MiB ingest and
721 MiB flush in the same sample). A budget that gives ingest 20 % and flush 25 %
is therefore only satisfiable if the memtable cap is set so that
`3.3 × cap ≤ flush share`, which at a 2 GiB budget means a memtable cap of about
**155 MiB accounted** — well under today's 256 MiB default, which is really
440 MiB, so the ingest share is the one the flush share actually constrains.
Either the flush share is expressed as a multiple of the ingest share,
or `flush_rows` streams the snapshot in bounded chunks so the multiple stops
mattering. The second is the real answer and it is `VISION.md` invariant II's
`Arc<Labels>` plus a chunked flush, not an arena boundary.

### Third, what is already inconsistent in the configuration

`Config::peak_materialized_bytes` (`config.rs:530`) reports `queries + merge` =
`8 × 512 MiB + 1 GiB` = **5 GiB**, and its own doc comment says it excludes the
memtable, trace scans and allocator retention. On this measurement the omissions
are the larger half: flush (unbounded, measured 721 MiB), the memtable at 1.72×
its meter, the metric path's untagged materialization at up to 203 MiB, and a
retention factor of 1.3–4.1. A configuration
that already admits to a 5 GiB ceiling on a 2 GiB container does not need an
arena table to know it is wrong; it needs the arenas to be **derived from one
declared number** instead of eight defaults that never appear next to each other.

---

## The settle, measured (2026-07-30)

The first attribution sampled while load was running. The budget gate has since
been extended through a settle, and the engine turns out to die there rather
than under load — so the phase after the last row was accepted was measured the
same way. Run `settle_attr`, build `1edb750`, `--budget 2GiB --limit 8GiB` so
the kernel does not end the run before the peak is visible, server built with
`--features memprof`, `/metrics` scraped at 1 Hz.

Anon peaked at **2805 MiB in the settle** against **1766 MiB in ingest**, at
t=91.6 s with load stopping at t=61.8 s. Live bytes by arena, at the last sample
before load stopped and at the settle peak:

| arena | at ingest end | at settle peak | change | live allocations at peak |
|---|---|---|---|---|
| ingest | 338.6 MiB | 0.0 | −338.6 | 0 |
| flush | 217.2 MiB | 0.0 | −217.2 | 3 |
| **merge** | **0.0** | **829.4 MiB** | **+829.4** | **7,217,588** |
| sidecar | 7.7 | 16.2 | +8.5 | 652 |
| part_meta | 0.3 | 0.6 | +0.3 | 12,738 |
| query | 0.0 | 0.0 | — | 0 |
| **total live** | **564.3 MiB** | **846.7 MiB** | +282.4 | |

**The settle peak is merge, and nothing else is in the room.** Ingest and flush
have both released everything by then; merge holds 98% of what is live. It runs
from about t=85 s to t=125 s and then the process falls to 16.8 MiB, so the term
is transient and very large rather than retained.

**This corrects a statement made when the derived-budget experiment failed.**
That experiment concluded "the merge budget is not the binding term at 2 GiB",
and the second half of that is right while the first half is not: merge *is* the
term. What the experiment established is narrower and still holds — **its budget
is not the lever.** Shrinking it to 512 MiB produced more, smaller merges that
overlapped ingest and moved the kill earlier; merge at the 1 GiB default runs to
829 MiB, close enough to its ceiling that the ceiling is doing its job.

**What the lever is, from the allocation count.** 7.2 million live allocations
holding 829 MiB is about 120 bytes each: `read_all_rows_with_limit` materializes
the whole group as a `Vec<Row>` whose lines and structured metadata are still
owned per row. `Arc<Labels>` removed the label term from this and the rest of it
is untouched. The ratio is the same story the first attribution told — 2805 MiB
of anon over 846.7 MiB of live is **3.3×**, and the ingest phase reads 1766 over
564.3, or 3.1× — so the allocator is holding roughly two thirds of the footprint
in both phases, and it is holding it because of the count, not the size.

So the direction is invariant II applied to merge, the same shape as the
streaming top-K executor was for the read path: rewrite a group without
materializing it. Not a smaller budget, which was measured.

Caveat carried from the first attribution: the instrument costs 16 bytes per
live allocation, so at 7.2 million allocations about **115 MiB** of merge's
829 MiB is the tag. The uninstrumented arm of this comparison is the gate's own
`settle_red` and `old1g_2` runs, which peak at 1991 and 1954 MiB of anon.

## The one-line summary

The memtable was never where the memory went, and neither was any single arena.
**44 % of the anonymous peak was memory the process had already freed**, held by
an allocator that was asked for 52 GB in 33 seconds across 444 million
allocations — 72 % of it by a read path that materializes everything and returns
a hundred rows, and 20 % by a flush path that spends 26 kB and 356 allocations
on a 368-byte line. Of the memory that *was* live, the two largest terms are one
merge group's rewrite at 771 MiB and the flush's whole-snapshot `Vec<Row>` at
721 MiB — 3.3× the memtable it copies. The sidecars, the in-flight push bodies
and the placement of the global sort are all real defects and all noise against
this.

## Re-measured on build `b9165b0` (2026-08-08), before sizing the declared budget

The shares below supersede every figure above for sizing purposes: 721 and
771 MiB were measured on a build that no longer exists (pre-streaming-merge,
pre-`Arc<Labels>`, pre-row-group-cache). The instrument is
`scripts/run_soak_local.sh` — the same offered 20 k eps and 5 qps, retention on
at 5 m, memprof build, 8 GiB cgroup so the run completes, 25 minutes, sampled
once a second (`target/soak/probe-8g-memprof`). Two layers, separated by the
same run:

**Live, bounded, oscillating — no arena trends upward over 25 minutes:**

| term | high-water | at the coincident live peak (1524 MiB, t=509 s) |
|---|---|---|
| merge | 689 MiB | 607 MiB |
| ingest accumulation (mis-binned, see below) | 445 MiB | 441 MiB |
| query (pool + decodes) | 429 MiB | 236 MiB |
| sidecar | 241 MiB | 128 MiB |
| flush | 112 MiB | 111 MiB |
| row-group cache (gauge; arena inert, see below) | ~250 MiB | ~250 MiB |

**Retained by glibc on top of that:** free ratchets 915 → 2628 MiB across the
run and never returns; anon steps 1710 → 2525 → 3157 → 3295 with shrinking
increments — the high-water of *coincident* live spikes times fragmentation.
Final anon/live **5.30**. Fixing `MALLOC_MMAP_THRESHOLD_=131072` (arenas left
at 4) collapses that to **1.60** and moves the 2 GiB kill from t≈150 s to
t≈502 s — the allocator layer is real but second; the live coincidence is
first.

Two attribution gaps the run itself exposed, so the table is read correctly:

* **The `ingest` arena reads 0.1 MiB and the memtable lands in `other`.** The
  `other` series tracks `signy_memtable_buffered_bytes` at **~1.73×**
  point for point (444 MiB against a saturated 256 MiB cap) — the in-situ
  confirmation of the metering item's 1.70–1.79× undercount. The Ingest guard
  sits in the journal writer; the memtable's own allocations happen outside
  any guard.
* **The `row_group_cache` arena reads ~0 while the cache gauge holds
  ~250 MiB.** Decodes are allocated under the scanning thread's Query guard
  and only *retained* by the cache, so `query`'s high-water includes the
  cache's bytes.

Also observed, recorded rather than diagnosed: between t≈445 and t≈465 the
query success counter froze for ~20 s while merge ran and the WAL backlog
climbed — the shape of the run's 1 % query 504 rate (48/4,926).

---

## The metric path, measured on build `df601dd` (2026-08-31)

[`COMPARISON_METRICS.md`](COMPARISON_METRICS.md) left the same shape of number
unexplained that M9 left for logs: at a 2 GiB container, 520 288 series offered,
signy peaked at **1 037.8 MiB** and refused 24 288 datapoints at
`max_active_series`, while VictoriaMetrics took every one of them at **627.3
MiB**. The engine's own accounting charges a series its canonical label bytes
plus `SERIES_OVERHEAD_BYTES = 160`, which puts half a million series near
160 MiB — a sixfold gap against what the container held.

Two arenas were added to close it (`series_memtable`, `series_catalog`), and a
first run showed the profile could not answer at all: **`other` peaked at
361.7 MiB**, larger than either named arena, because the metric decode, flush
and compaction sat outside every guard — `ingest`, `flush` and `merge` all read
a flat zero for a metrics-only workload. Those three now enter the arena their
role already names, and `other` reads **0.4 MiB**.

### Reproducing it

```sh
# Absolute anon, on the production allocator (mimalloc), no instrument tax
SIGNY_LOAD_METRIC_CHURN_REPLACE=2000 SIGNY_LOAD_METRIC_EXPLOSION_SERIES=500000 \
  cargo run --release --bin memory_gate -- --scenario metrics --budget 2GiB --seconds 480

# The arena split
export SIGNY_LOAD_METRIC_ANCHOR_NS=1 SIGNY_LOAD_METRIC_CHURN_REPLACE=2000 \
       SIGNY_LOAD_METRIC_EXPLOSION_SERIES=500000 \
       SIGNY_LOAD_METRIC_STEADY_SECONDS=96 SIGNY_LOAD_METRIC_CHURN_SECONDS=240 \
       SIGNY_LOAD_METRIC_EXPLOSION_SECONDS=144
M10_PHASE=metric-load M10_LIMIT=2G M10_FEATURES=memprof \
  scripts/run_memprof_local.sh metrics-churn
```

The earlier pre-mimalloc baseline was **748.1 MiB** peak anon with 20 requests
refused in the explosion phase, accepting 460 744 of 560 032 datapoints. That
number is retained as historical context only: its allocator and lifecycle
configuration differ, so subtracting it from either result below would not
isolate the effect of one structural change.

With mimalloc and the old 10% budget-derived memtable ceiling, the first
post-admission run peaked at **374.23 MiB** anon but refused **58** whole
exports, accepting only **270 744 of 560 032** explosion datapoints. The 25%
ceiling was then validated on the same 2 GiB/480 s workload:

| result | value |
|---|---:|
| verdict | **UNDER_BUDGET** |
| peak anon | **655.527 MiB (32.008% of 2 GiB)** |
| cgroup `memory.peak` (including page cache) | **957.1445 MiB** |
| settle anon | **454.7148 MiB** |
| OOM kills | **0** |
| requests refused | **0** (`200=114`) |
| explosion accepted | **560 032 / 560 032** |
| churn accepted | **102 912 / 102 912** |
| steady accepted | **2 880 / 2 880** |

The validation run therefore both admits the full comparison-bed offer and
stays well below the cgroup limit. Its higher peak than the first 10% run is
expected because it processes the previously refused work; it is not evidence
that the allocator alone or the share alone caused a particular delta.

### The attribution, at the instrumented run's anon peak

482 320 active series, 10 open metric parts, peak `anon` 677.1 MiB.

| arena | MiB | per series | what it is |
|---|---|---|---|
| `series_memtable` | 201.1 | 437 B | live series entries: buffer struct, open Gorilla stream, spill |
| `flush` | 130.7 | — | **transient**: the partition map and re-encoded chunks of a flush in progress |
| `series_catalog` | 88.5 | 192 B | one `CatalogEntry` per series per open part, labels included |
| `merge` | 71.0 | — | **transient**: compaction's merged map |
| `ingest` | 55.2 | 120 B | the decode's `SeriesLabels`, which the memtable then keeps by `Arc` |
| `other` | 0.4 | — | untagged remainder |
| instrument header | 31.4 | — | memprof's 16 bytes per live allocation |
| **live** | **546.9** | | |
| allocator, not returned | ~130 | — | `anon − live`; ratio 1.24 |

### Four things this says

**1. The memtable's accounting undercounts by ~1.7×.** At the same instant,
`signy_series_memtable_bytes` read **117.1 MiB** against the arena's 201.1 MiB —
254 B charged per series against 437 B held. Counting the labels the decode
allocates and the memtable retains (`ingest`, 120 B/series), the real figure is
about **557 B per live series** against 254 B charged. `SERIES_OVERHEAD_BYTES`
is the constant to re-derive, and this is the number to derive it from.

**2. The catalog is residency the accounting has never seen.** A
`SeriesPartReader` decodes its part's whole catalog — a full `SeriesLabels`
allocation per series — and holds it for as long as the registry holds the
part, `open_cached` included, so an offloaded body does not release it. Traces
keep only blooms; logs keep a cached Parquet footer. Metrics is the only signal
that keeps a per-row structure resident, and at 192 B per active series it is
the same order as the memtable it is invisible beside. Measured at 186.9 MiB in
the partially-tagged run and 88.5 MiB here, so treat it as **190–400 B per
series** and part-count-dependent rather than as one number.

**3. The peak instant is a flush and a compaction, not steady state.** `flush`
and `merge` together held **201.7 MiB** at the peak and read 0.0 forty seconds
earlier. They are not residency, and they are what decides the high-water mark.

**4. Compaction materializes its whole group before writing.** `compact_once`
builds a `BTreeMap<TenantId, BTreeMap<SeriesLabels, Vec<(i64, f64)>>>` over
every input part's every series and every sample, and only then calls the flush
writer. That is the shape [`MEMORY_BUDGET_GATE.md`](MEMORY_BUDGET_GATE.md)
already fixed once on the log side, where a rewrite that materialized its group
held 829 of 847 live MiB at the settle peak and streaming it is what moved the
gate. The metric compactor repeats it.

### Reading the table correctly

* **The instrumented build is not on mimalloc.** memprof installs its own
  tagging global allocator over `System`, so the `anon`, the 1.24 ratio and the
  `glibc arena/free` lines are glibc's behaviour, not the production binary's.
  The *split between arenas* is what this run establishes; the absolute peak
  comes from the gate run above.
* **`ingest` is charged where the bytes are allocated, not where they live.**
  The 55.2 MiB plateau does not fall after the journal write because
  `normalize_request` allocates the `SeriesLabels` the memtable then holds by
  `Arc`. Nesting works as documented — `admit_datapoints` enters
  `series_memtable` inside the ingest guard — so the buffer is separated from
  the labels, and both belong to the memtable's true cost.
* **`series_catalog` is likewise charged at decode.** Compaction clones those
  `Arc`s into its merged map, so a catalog entry can outlive the reader that
  decoded it while still being charged here.

### The derived memtable ceiling after streaming (2026-08-31)

The first byte-admission run used the old budget-derived ceiling of 10%:
122.9 MiB from a 60% declared budget on a 2 GiB cgroup. That ceiling was below
the measured metric resident set and produced **58 whole-export `429`s**,
accepting only **270,744 of 560,032** offered datapoints. The same run's
streaming flush/compaction build peaked at **374.23 MiB anonymous memory**, so
the refusal was the derived ceiling rather than cgroup pressure. This was the
first post-admission run, not the final validation.

The default is now **25% of the declared budget** for the shared
log/trace/metric memtables. At that setup this is **307.2 MiB (322.1 million bytes)**. It is large
enough for the measured ~482k-series memtable (201.1 MiB in the instrumented
run) with churn and conservative sample reservations, while the engine still
declares only 60% of the cgroup and leaves the rest for allocator retention,
query/cache state, request queues, and runtime overhead. The gate remains a
hard byte boundary: metric admission rejects the complete export with
`429`/`Retry-After` when the shared ceiling cannot fit it; an explicit
`SIGNY_MAX_MEMTABLE_BYTES` continues to override the derived value.

The second run validated this derivation on the comparison bed: it was
`UNDER_BUDGET`, accepted **560,032/560,032** explosion datapoints with no
refusals, and peaked at **655.527 MiB anon** (32.008% of the 2 GiB cgroup),
with **454.715 MiB** at settle and no OOM kill. This is the measured result for
the current workload, not a claim that every workload has a 500k-series
capacity; explicit overrides remain the operator's responsibility.
