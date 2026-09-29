# What collecty's memory is supposed to be, and what it is

collecty's resident memory is supposed to be a function of the ceilings its
operator declares and of nothing else. [`CONFIGURATION.md`](CONFIGURATION.md)
says so in one sentence — "resident memory is roughly this ceiling plus one
batch buffer plus the runtime. It is not affected by how far behind signy is —
that backlog lives on disk" — and until 2026-09-04 nothing had measured it.

**It is not true.** Over thirteen hours at 20 k eps a single collecty process
went from 40 MiB to 192 MiB and was still climbing, its disk queue empty the
whole time. This document declares what the footprint should be, records what
it is, and is the place every experiment that follows writes its result.

The order of the two halves is deliberate. The budget below was written before
the instrument that could attribute it existed, so it is a prediction the
measurements can falsify rather than a description fitted to them.

---

## 1. The declared budget

At the shipped defaults — 64 MiB in flight, 8 MiB segments, a 1 GiB queue — and
under a workload whose exports are far smaller than the 16 MiB request ceiling:

| term | where it comes from | bytes |
|---|---|---|
| process floor | measured: a release build, idle, having taken nothing | **4.6 MiB** |
| three open segments' compressors | measured: `ZSTD_CCtx` at level 3, in use, 3,663,385 B, plus the writer's `ZSTD_CStreamOutSize` output buffer, 131,591 B, per signal | **10.9 MiB** |
| exports admitted and not yet appended | `COLLECTY_MAX_INFLIGHT_BYTES`, the declared ceiling | ≤ 64 MiB |
| the copy of each of those the framing makes | `wire::frame_record` allocates a second buffer per export to prepend four bytes | ≤ 64 MiB |
| the segment being shipped | `Queue::read_segment`'s buffer, ≤ `COLLECTY_QUEUE_SEGMENT_BYTES`, and the `Bytes` it becomes | ≤ 16 MiB |
| per-connection read buffers | hyper's, ≤ ~400 KiB each | connections × 0.4 MiB |
| the queue's own bookkeeping | one 24-byte `Held` per segment, at most `queue_max / segment` of them | ~3 KiB |

Two numbers come out of that, and they are what this document gates on.

* **Steady state ≤ 64 MiB.** A workload that keeps only a small fraction of the
  in-flight ceiling occupied should sit near floor + compressors + one segment
  in flight, which is 35–45 MiB. 64 MiB is that with headroom.
* **Peak ≤ 128 MiB**, which is what the declared ceilings permit once the
  in-flight gate is actually holding the line it names.

And three invariants, which matter more than either number, because each one is
a thing the trace below shows collecty currently is a function of:

1. **Not a function of elapsed time.** A process at hour ten holds what a
   process at hour one holds.
2. **Not a function of connection count.** `COLLECTY_MAX_INFLIGHT_BYTES` is the
   ceiling on bytes being admitted, whatever number of clients offer them.
3. **Not a function of how far behind signy has been.** A backlog is disk. A
   backlog that has drained leaves nothing behind in memory.

---

## 2. What this does not establish

Stated first, in the discipline signy's
[`MEMORY_ATTRIBUTION.md`](../../signy/docs/MEMORY_ATTRIBUTION.md) sets.

* **Nothing here attributes anything.** collecty has no arena instrument yet, so
  every number below is a total — the cgroup's, or the kernel's view of the
  process. Which of the six suspects the growth belongs to is not decided by
  this document; the instrument is being built to decide it.
* **It is one workload on one machine.** 20 k eps of the load harness's log
  corpus over 8 ingest connections, on Linux 6.12.57, 12 logical CPUs, 31 GiB
  RAM, glibc 2.41, release build `2aa5ed6`, no allocator chosen and no
  `MALLOC_*` tuning. Another export size distribution is another experiment.
* **The run was stopped at 13 h of 24.** It was a signy soak, and collecty was
  a passenger in it; it was stopped on purpose to free the machine for the
  experiments this document sets up. The trend at the moment it stopped had not
  flattened, and a run that flattens at hour eighteen would not contradict
  anything below — it would answer a question this run leaves open.
* **The two outage steps are one observation each.** They are consistent with
  each other and with a mechanism, which is why they are recorded, but two
  events are not a curve.
* **Sampling is one reading per five seconds**, from the cgroup's `memory.stat`.
  A spike shorter than that is missed.

---

## 3. What is actually observed

`signy/target/soak/day-1`, 2026-09-04, 20 k eps through collecty into signy,
collecty in a 256 MiB cgroup at the shipped defaults. Its artifacts are kept in
`signy/target/soak/day-1-13h-partial/`.

### It does not converge

The collector was restarted by the rig's own fault injection at t = 8643 s, so
the ten hours after that are one process:

| hour | mean anon (MiB) | peak (MiB) |
|---|---|---|
| 0 | 102.5 | 124.4 |
| 1 | 134.2 | 143.5 |
| 2 | 115.0 | 146.2 |
| 3 | 136.2 | 145.7 |
| 4 | 150.0 | 156.3 |
| 5 | 158.3 | 161.3 |
| 6 | 165.1 | 167.4 |
| 7 | 169.6 | 172.1 |
| 8 | 173.0 | 175.1 |
| 9 | 179.2 | 188.1 |
| 10 | 188.7 | 190.4 |
| 11 | 191.0 | 191.9 |
| 12 | 192.0 | 192.1 |

A fresh process is at **40.2 MiB** twenty seconds after it starts and at
**98 MiB** fifteen minutes later. Ten hours later it is at **192.1 MiB**, and
the last four hours still add about 1 MiB each. The disk queue held 2.2 MiB and
3.7 segments throughout: **the growth is not the backlog**, and on that narrow
point `CONFIGURATION.md` is right.

**This supersedes the conclusion recorded on 2026-09-04 in `signy/todo.md`** —
"collecty converges, it does not leak", from a half-hour rehearsal that saw
111 MiB and a growth rate falling to 1.4 MiB/min. The rate does fall. It does
not reach zero, and half an hour was not long enough to tell the two apart. The
earlier observation is not retracted: at t = 30 min this run was also near
111 MiB.

### A drained backlog leaves memory behind

The rig took signy away twice. Both times collecty's anon was **flat for the
whole outage** and rose **while the queue drained**, and both times it stayed
up:

| outage | backlog built | anon before | anon after | permanent step |
|---|---|---|---|---|
| 50 s (t = 21600) | 92 MiB on disk | 161.3 MiB | 163.4 MiB | **+2.1 MiB** |
| 180 s (t = 34560) | 377 MiB on disk | 175.3 MiB | 187.6 MiB | **+12.3 MiB** |

Roughly 2–3 % of what passed through the drain stayed resident after the queue
was empty again. This is invariant 3 failing, and it is the sharpest causal
evidence in the trace: nothing about the *ingest* side changed across those
windows — the offered rate was constant — so what the drain did differently is
the send path shipping segments back to back as fast as signy would take them.

That is a hypothesis, not a finding. `Queue::read_segment` allocates a fresh
multi-megabyte buffer per segment, and a drain runs that loop at whatever rate
the network allows instead of the ~1/s a steady state runs it at; whether the
bytes that stayed are that buffer, its `Bytes`, the compressor churn beside it,
or the allocator declining to return any of them is exactly what the arena
instrument is for.

**It was the buffer.** Mapping the segment instead of reading it takes the
step to zero, twice — the drain below.

---

## 4. The experiments

Each is a 300 s run at 204 exports/s of a 41,656 byte mean over 8 connections,
plus 5 trace exports a second and a scrape every ten seconds, in a 256 MiB
cage, followed by a 60 s settle. `scripts/run_mem_local.sh` is the one command;
every row is that command with one thing changed. Peak anon is the cgroup's,
which is what an OOM kill is decided on; the arena columns are the collector's
own view and are never the verdict.

The rig is calibrated against what the 24-hour soak's collector actually
received, read out of its own queue reports: 205 exports/s of a 41,785 byte
mean. The one thing that does not match is the compression ratio — 3.63x here
against the soak's 4.87x — so a segment here is larger than production's, which
errs towards more memory pressure rather than less.

### Two baselines, and which is which

Every number below belongs to one of two builds of the same code, and they are
named here because otherwise the series has two baselines and no way to tell
them apart:

| | build | what it is for | at the start | at the end |
|---|---|---|---|---|
| `baseline` | `--features memprof` | attribution: the arena, `mallinfo2` and thread columns only exist here | 69.5 MiB | — |
| `baseline-shipped` | default features | the number that ships; carries no instrument | 60.7 MiB | **18.5 MiB** (`final-shipped`) |

Same load, same cage, same 300 s. **The instrument costs about 9 MiB — 14 %** —
so an instrumented row is never comparable with a shipped one, and the series
below is instrumented throughout so that its rows are comparable with each
other.

Those are also the directory names each run leaves under `target/mem/`.

### The instrumented series

Each row is the previous row's code plus one change, all on glibc unless the
row says otherwise, and all on the `baseline` (instrumented) build.

| # | change | peak anon | peak tagged live | libc retained | untagged in use | threads | allocated |
|---|---|---|---|---|---|---|---|
| 2 | baseline | **69.5 MiB** | 3.7 | 53.7 | 4.0 | 26 | 7.5 GB |
| 3 | `MALLOC_ARENA_MAX=1` *(diagnosis only)* | **16.2** | 3.9 | 6.1 | 7.5 | 24 | 7.5 GB |
| 4 | one spool thread, not the blocking pool | **26.1** | 3.8 | 16.6 | 7.4 | 16 | 7.4 GB |
| 5 | mimalloc *(rejected)* | **54.1** | 3.8 | 1.9 | 7.9 | 16 | 7.4 GB |
| 5b | mimalloc, `MIMALLOC_PURGE_DELAY=0` *(rejected)* | **35.7** | — | 5.4 | — | 16 | — |
| 6 | the framing copy removed | **21.8** | 1.4 | 18.0 | 7.4 | 16 | 4.9 GB |
| 7 | the compressor reused across segments | **18.8** | 3.8 | 9.4 | **0.4** | 16 | 4.9 GB |
| 8 | the segment read sized from the file | **18.8** | 5.3 | 10.8 | 0.5 | 16 | 4.9 GB |

Every run accepted 33,424 eps with no failures, so none of these numbers was
bought by refusing load.

**The shipped build goes 60.7 → 18.5 MiB, a 70 % reduction**, at 33,424 eps
accepted with no failures either side. That the instrumented build could be run
at all for every row of this series — signy's could not, and the day it most
wanted attribution it had to give it up — is what making the tagging
independent of the allocator bought.

### What each row established

* **The footprint was never the collector's data.** At the baseline, peak live
  bytes are 3.7 MiB inside a 69.5 MiB anonymous footprint — **5 %**. signy's
  equivalent measurement found 39.7 %. Whatever collecty is holding, it is not
  telemetry in flight.
* **It was threads times arenas.** `MALLOC_ARENA_MAX=1` takes the same run to
  16.2 MiB with identical allocation volume, identical live bytes and identical
  throughput.

  **That row is the smallest number in this document and it is deliberately not
  what collecty ships. Three reasons, because someone will otherwise read the
  table and ask why 16.2 lost to 18.5.**

  1. **They are not the same experiment.** 16.2 MiB is the *old* dispatch with
     one arena; 18.5 MiB is the *new* dispatch with the allocator left alone.
     "Capping arenas beats fixing the dispatch" is not what those two rows say,
     and the run that would say it — `MALLOC_ARENA_MAX=1` on top of the spool
     thread — has not been made. It is one six-minute run and it is worth
     making.
  2. **It is an environment variable, so it is a property of a deployment
     rather than of the collector.** A footprint that depends on an operator
     knowing to set `MALLOC_ARENA_MAX` is one that is wrong everywhere it is
     forgotten, and it is wrong silently. The structural fix is true in every
     deployment including the ones that never read this document.
  3. **It has a measured cost elsewhere.** signy tried the same setting and it
     re-inflicted the backlog that had made it non-default there: the WAL
     backlog climbed 2.9 → 50.7 MiB while the default's stayed near 1.4. One
     arena serializes allocation across threads, which is the whole point of it.
     collecty now does nearly all of its allocating on one thread, so that cost
     may be small here — but "may be" is not a measurement, which is reason 1
     again.
* **The structural fix recovers most of it.** `Queue::append` takes the queue's
  lock as its first act, so dispatching each append to the blocking pool bought
  no parallelism and cost a thread — and therefore an arena — per concurrent
  export. One spool thread: 69.5 → 26.1 MiB, threads 26 → 16.
* **mimalloc is the wrong allocator here, which is the opposite of signy's
  answer and the result most likely to be "corrected" by someone harmonizing
  the two.** It doubled the footprint at its defaults and was still worse than
  glibc with an eager purge. signy chose it for a workload holding hundreds of
  live megabytes across hundreds of millions of allocations a run; collecty
  holds one or two megabytes across a million, and an allocator that keeps
  arenas warm for that has nothing to keep them warm for.
* **A third of everything allocated was one needless copy.** `frame_record`
  built a second buffer per export whose only difference was four bytes in
  front, and both were alive at once: 7.4 → 4.9 GB allocated, peak live
  3.8 → 1.4 MiB.
* **The compressor was the C-side term, and moving it moved the C-side
  column.** `libc_in_use` minus the tagged total is where an allocation that
  does not go through the Rust allocator shows up. Reusing one context per
  signal instead of building one per segment took that column from 7.4 MiB to
  0.4 MiB. This is the row that most needed a column of its own to be
  believable, and it is why the instrument reports one.
* **Sizing the segment read measured nothing at steady state**, and is kept for
  a reason the steady state cannot show. See the drain below.

### The drain, which is its own experiment

The sink is taken away for 60 s at t=120 and comes back; the backlog goes to
disk and then drains. Every run is the shipped build and every one built a
backlog of about 200 MB, so this compares like with like. The last two builds
were run twice, because half a megabyte is not a difference one run can tell
from noise:

| | before every fix | reading the segment | mapping it |
|---|---|---|---|
| runs | 1 | 3 | 2 |
| anon before the outage | 59.4 | 15.9, 17.5, 18.2 | **9.7, 7.6** |
| anon after the drain finished | 105.8 | 23.7, 23.6, 23.2 | **9.6, 7.5** |
| **permanent step** | **+46.4** | **+7.8, +6.1, +5.0** | **−0.1, −0.1** |
| `memory.current` after settling | — | 25.3, 26.0 | **10.7, 9.9** |
| page cache at its peak | — | 202.5, 141.6 | 208.9, 200.4 |
| CPU over the 360 s | — | 17.3, 17.6 s | 18.1, 18.2 s |

All MiB except the CPU row. The first column predates the sampler that reports
the last three.

**The step is gone.** A sealed segment is the wire body byte for byte, so
`Bytes::from_owner` over a mapping hands hyper the file's own pages and nothing
on the send path allocates per segment any more. Twice the drain ended
*below* where it started, which is what a queue that leaves nothing behind
looks like once ordinary jitter is allowed for.

**The steady state fell with it, which was not the prediction.** 17.9 MiB
before the outage on average against 8.7 after — the per-segment buffer was
sizing the arena in the steady state too, at the ~1/s a quiet host rolls
segments at, and not only in the burst a drain makes of it.

**It is a smaller heap and not a relabel.** Mapping a file moves bytes from
anon into page cache, and a cgroup counts both, so the win had to be checked
against `memory.current` rather than declared from anon. The page cache peak is
the same either way — that is the backlog on disk, which both builds wrote —
and `memory.current` after settling went 25.6 → 10.3 MiB. The bytes were
removed, not moved.

**What it costs is CPU: about 4 %, and it reproduces.** 17.3 and 17.6 s against
18.1, 18.2, 17.8 and 18.2 s over four runs of the mapped build, ranges that do
not overlap. A read copies out of the page
cache in one bulk move; a mapping takes a minor fault per page instead, and a
drain touches every page of 200 MB. It is kept because the memory result is
large, reproducible and structural while the CPU cost is small — but it is a
cost, and `MAP_POPULATE` or a sequential `madvise` is the obvious thing to try
against it.

**What it does not fix is the cage.** `memory.current` still peaked at 215–229
MiB of the 256 MiB limit, and that is the queue's own page cache during the
outage, unchanged. A sidecar is still sized by the backlog it is allowed to
build and not by its heap, which is the deployment fact §3's tmpfs trap ends
on.

### Two worker threads instead of one per core

Nothing left on the tokio runtime is CPU-heavy. Compression and every write to
the queue belong to the spool thread, and what remains is accepting
connections, reading bodies and shipping segments — so a worker per core was
twelve threads doing almost nothing on this machine, and a thread that
allocates is an arena that keeps whatever it grew to.

Same drain, two runs each, on the mapped build:

| | one per core | two |
|---|---|---|
| anon before the outage | 12.7, 11.4 | **9.9, 9.5** |
| anon after the drain | 12.8, 11.2 | **9.5, 9.3** |
| `memory.current` after settling | 13.5, 12.0 | 12.0, 11.4 |
| CPU over the 360 s | 17.8, 18.2 s | **17.6, 17.0 s** |
| steady p50 | 1.26, 1.26 ms | 1.31, 1.28 ms |
| steady p95 | 2.61, 2.72 ms | 2.78, 2.67 ms |
| steady p99 | 24.1, 27.2 ms | 24.2, 40.7 ms |
| accepted | 62,286 / 62,286 | 62,286 / 62,286 |

**Two megabytes, and no cost that two runs can find.** Taking the means, and
keeping the three memory numbers apart because they are three different
things: anon before the outage falls 12.05 → 9.70 MiB (−19.5 %), anon after
the drain 12.0 → 9.4 MiB (−21.7 %), and `memory.current` after settling 12.75
→ 11.7 MiB (−8.2 %). None of these is RSS, which this rig does not sample on
the shipped build. The anon ranges do not meet. CPU came out lower rather than
higher, which two runs each is enough to read as *no penalty* and not enough
to read as an improvement. p50 and p95 move by less than a tenth of a
millisecond.

**The p99 row says 27 against 41 ms, and what the samples support is that
there is no evidence of a regression — not that there is none.** This is why
the samples are kept. In the steady window the share of exports over 10 ms is
2.26 / 2.61 % against 2.71 / 2.57 %, and over 50 ms 0.29 / 0.06 % against
0.06 / 0.52 %: the two configurations straddle each other on both cuts, so
the tail is a handful of outliers in either and two runs cannot separate them.
The single worst sample of a run — 384 ms against 302 ms — carries almost no
information about a tail and is in the table only because it is what `max`
means. What would be a regression, the body of the distribution moving, did
not happen.

**One worker is the next step and not this one.** The sender still calls
`Queue::seal_if_due` on this runtime, and that closes a segment and `fsync`s
it, so a single worker would stop accepting connections, polling them and
firing timers for as long as the disk took. `current_thread` is what this
should become once the seal, the `fsync` and the rest of the queue's
filesystem work belong to the spool thread — one runtime thread and one disk
thread, which is a shape that suits a sidecar.

### The queue's files, off the runtime and onto threads of their own

The sender called `seal_if_due`, `read_segment` and `commit` straight from its
task. None of those is asynchronous — an `fsync` returns when the device says
so — and a stall on a tokio worker is not one task waiting but every task on
that worker waiting. That is survivable with a worker per core and it is the
thing that decides whether one worker is possible at all.

Two shapes were measured, two runs each, on the same drain:

* **one thread for all of it.** Appends already ran on the spool thread;
  sealing, reading and committing joined them.
* **reads in a lane of their own.** `append`, `seal` and `commit` all take the
  queue's lock and so were serial with each other before any of this;
  `read_segment` takes no lock and ran beside an append. The first shape gave
  that up, the second gives it back, and both keep the runtime free of
  filesystem syscalls.

| | on the runtime | one thread | reads apart |
|---|---|---|---|
| anon before the outage | 9.9, 9.5 | 7.9, 8.5 | 8.4, 8.3 |
| anon after the drain | 9.5, 9.3 | 7.5, 8.4 | 10.3, 7.9 |
| CPU over the 360 s | 17.6, 17.0 s | 17.1, 16.9 s | 17.1, 17.0 s |
| accepted | 100 % | 100 % | 100 % |
| draining, over 100 ms | 0.00, 0.03 % | 0.26, 0.20 % | 0.06, 0.03 % |
| write lane p99 wait | — | 16–33 ms | 33–65 ms |
| read lane p99 wait | — | — | **256 µs**, never more than one waiting |

**A drift control, and what it took back.** Those columns were run in the order
they appear, over about two hours, and the last thing this experiment did was
re-run the *first* column again at the end. It came back **worse than either
of the others on every cut** — steady over 50 ms at 1.28 % against 0.06 and
0.52 % for the same binary two hours earlier, draining over 100 ms at 0.23 %
against 0.00 and 0.03 %. The machine drifted, and a comparison laid out in
time order cannot tell that from a change.

So the middle column is **not** the regression it first looked like: its
draining 0.26 / 0.20 % sits beside the contemporaneous baseline's 0.23 %, not
beside the two-hour-old 0.00 / 0.03 %. What survives is the comparison between
the two shapes, because the third column ran *after* the second and drift does
not run backwards: separating the reads is better than sharing one queue, and
better than a baseline re-run beside it. The rest of the first reading was the
machine.

**Every A/B from here interleaves its runs** — A, B, A, B — rather than
running one arm and then the other.

**Memory did not drift within this section** — the baseline's anon was 9.9 at
the start and 9.9 at the end — and both threaded shapes came in at 8.3–8.5,
so about a megabyte and a half lower. **That is an observation and not a
result.** The section below runs the same two-worker code four more times and
gets 7.4 as well as 9.9, a spread wider than the gap being read here, so what
this table supports is *around one to two megabytes lower, seen twice each*,
and nothing about why.

One of the seven runs in this section ended a drain 1.9 MiB above where it
started, where the other six ended within half a megabyte; it is recorded
rather than explained.

**The write lane waits, and it always did.** Its p99 of 33–65 ms is the `fsync`
that closes a segment, which under the old shape ran on a worker while holding
the queue's lock — so an append waited exactly as long and nothing counted it.
The instrument is new; the wait is not.

### One runtime thread, measured and not taken

With no filesystem syscall left on the runtime, one worker becomes possible.
Four runs, laid out **A B B A** so that each arm takes a turn at being first
and last and a drift in time cancels rather than accumulates:

| | two workers | `current_thread` |
|---|---|---|
| CPU over the 360 s | 17.2, 17.1 s | **15.9, 16.1 s** |
| steady p50 | 1.28, 1.27 ms | **1.41, 1.45 ms** |
| steady p95 | 2.71, 2.66 ms | **2.99, 2.86 ms** |
| anon before the outage | 9.9, 7.4 | 10.2, 10.0 |
| draining, over 100 ms | 0.07, 0.00 % | 0.07, 0.17 % |
| accepted | 100 % | 100 % |

**The CPU is real and the latency cost is real.** Both `current_thread` runs
came in under both two-worker runs, which their placement rules out as
position; and both are about 10 % slower at the median and at p95, which the
two-worker arm's own stability rules out as drift. One runtime thread queues
what two absorbed.

**It is not taken, because of what the absolute numbers are.** 17.15 against
16.00 CPU-seconds over 360 is 0.048 against 0.044 of a core: this collector is
nowhere near CPU-bound, and 6.7 % of very little is being bought with 10 % of
the body of every ordinary request. What that CPU is worth in money on a FaaS
bill has not been established either.

**And one thread has no headroom to give.** Two workers absorb a rise in load;
one saturates and then p50, p95 and p99 climb together. That p50 and p95 moved
at all at this rate — where the runtime is idle almost all of the time — is a
small signal in that direction.

**What would settle it is a saturation experiment**, which this is not: the
offered rate at 1x, 2x, 4x and 8x against both runtimes, looking for where
latency and accepted exports start to come apart. If the knee is in the same
place for both, 6.7 % of CPU is worth taking.

**The tail is not evidence either way.** Over 100 ms reads 0.07 / 0.00 %
against 0.07 / 0.17 %, which is too few samples to separate — no repeatable
regression was observed, which is not the same as none being there. The
page-fault risk this experiment was watching for — a mapped segment first
touched on the sole runtime thread, during a drain that reads 200 MB of them —
did not show up at this rate.

**Memory could not be judged at all.** The two-worker arm's own runs are 9.9
and 7.4 MiB of anon before the outage, 2.5 MiB apart on identical code, so a
1.5 MiB difference between arms is inside the noise of one of them. Nothing
about the allocator is claimed from this.

### The second copy of a body, which was only there for large ones

`route` used `collect().to_bytes()`: gather the body's frames, then join them.
The join is a fresh allocation the size of the whole export and a copy of every
byte into it, and nothing downstream wants them joined — the queue writes a
four byte length and then the payload into the encoder, which does not care
where one piece ends and the next begins. So the frames now go to the queue as
they arrive.

**At the corpus every other row of this document uses, it measures exactly
nothing.** Four shipped runs laid out A B B A, and an instrumented pair:

| | joined | in pieces |
|---|---|---|
| bytes allocated | 4.2 GB | 4.2 GB |
| allocations | 1.1 M | 1.1 M |
| CPU | 17.1, 17.1 s | 17.6, 17.2 s |
| anon before the outage | 7.9, 9.8 | 8.2, 8.6 |

Identical, because a 41 kB body arrives in **one** frame and joining one buffer
copies nothing. An earlier reading of this document's 1.9 bytes allocated per
byte appended blamed this join for half of it; that was wrong, and the join was
not costing anything at this size.

**At ten times the body it is the change it was supposed to be.** Same byte
rate, same everything, 1,610 records per export instead of 161, so a body is
415 kB and spans frames:

| | joined | in pieces |
|---|---|---|
| bytes allocated | 5.3 GB | **4.1 GB** |
| peak anon | 19.4 MiB | **16.9 MiB** |
| heap held free at the end | 5.6 MiB | **1.4 MiB** |
| peak `memory.current` | 230.6 MiB | 221.1 MiB |
| CPU | 12.8 s | 12.6 s |

The allocation figure is a count of what the workload asked for and is the
solid one; anon and CPU are one run each and anon has shown a 2.5 MiB spread
elsewhere in this document, so read the first row and treat the second as
agreeing with it rather than as its own result.

**Which makes this a change for the deployments that batch.** A collector's
request ceiling is 16 MiB and an exporter that fills even a fraction of it
sends a body that cannot arrive in one piece — and every byte of it was being
allocated and copied a second time, outside the in-flight gate, which is the
same accounting hole the connection sweep found. At 41 kB there was nothing to
fix and there is now nothing to lose: one small `Vec` of frame pointers stands
where a second copy of the body used to be.

### What the kernel does about the queue's page cache

Anon went from 60.7 MiB to about 8 and the cage's peak never moved: 200–230
MiB of a 256 MiB limit, nearly all of it the queue's own file pages. Before
changing any policy about that, the question is what the kernel can take back
and what taking it back costs — so `memory.high` is set below `memory.max` and
the same outage is run against it. Crossing `memory.high` makes the kernel
reclaim this cgroup and hold it up while it does; crossing `memory.max` is
where the OOM killer starts.

| `memory.high` | queue on disk | page cache resident | `memory.current` peak | `high` events | `max` | `oom` | PSI stalled | accepted |
|---|---|---|---|---|---|---|---|---|
| unset | 213.6 | 213.6 | 231.3 | 0 | 0 | 0 | 0 s | 100 % |
| 192 MiB | 138.7 | 138.7 | 152.2 | 0 | 0 | 0 | 0 s | 100 % |
| 96 MiB | 205.4 | **83.6** | **95.9** | 501 | 0 | 0 | 0.15 s | 100 % |
| 96 MiB | 194.1 | **83.5** | **95.3** | 463 | 0 | 0 | 0.05 s | 100 % |
| 48 MiB | 141.2 | **35.1** | **47.6** | 436 | 0 | 0 | 0.12 s | 100 % |

MiB throughout; PSI is total stall in a 360 s run.

**The disk queue and the resident page cache are already separate things.**
With the threshold at 96 MiB and a 205 MiB backlog being written, usage peaks
at 95.9: the kernel takes the pages back five hundred times and the workload is
stopped for 0.15 seconds out of 360. At a threshold of 48 — under a quarter of
the backlog — the same, at 47.6. `memory.high` does not cap anything, so those
peaks are how well reclaim kept up rather than a bound it enforced; what the
setting does is make the kernel start early enough that `memory.max` stays out
of the story. Every export is accepted in every
row, `memory.max` is never reached, nothing is killed, and no latency figure
orders itself by the setting: p50 runs 1.20–1.28 ms and draining p99 19–31 ms
across all five with the unthrottled run in the middle of both ranges.

**The 192 MiB row is not a result.** That run's backlog only reached 138.7 MiB,
so the throttle was never crossed and its `high` count is zero. The backlog
comes out either ~140 or ~205 MiB depending on whether the sender's backoff —
up to 30 s — happens to straddle the sink's return.

**Why reclaim is this cheap here is structural.** Every page of the queue is
clean almost as soon as it is written: a segment is `fsync`ed when it closes,
so at most one open segment per signal is dirty, and the dirty and writeback
columns never exceed 8 MiB in any run. Reclaim never has to write anything
back to take a page. The cache is also entirely on the inactive list — 213.6
MiB of 213.6 in the unthrottled run, with the active list at zero — which is
the list reclaim takes from first.

**So neither of the two things this was going to lead to is needed.** Admission
control tying the disk queue to a RAM budget would be answering a question the
kernel already answers, and it would answer it wrongly: a durable backlog of
5 GiB with 30 MiB resident is a correct state, not a violation. Dropping the
pages by hand after a seal — `POSIX_FADV_DONTNEED`, which the seal's `fsync`
would make legal — would buy the same separation while paying for it in reads
at drain time, and there is nothing left to buy.

**What comes out of this is a deployment fact rather than a change.** Left
alone, `memory.current` rises to whatever the backlog is, because nothing asked
the kernel to stop it — that is not danger, it is an unbounded cache doing what
a cache does, but it leaves no headroom to read and no signal to alert on.
`memory.high` below `memory.max` costs a fraction of a second per outage and
turns the container's memory into a number that means something.
[`CONFIGURATION.md`](CONFIGURATION.md) now says so.

**Ten times the backlog, the same resident set.** The run above builds about
200 MiB. A 900 s outage building **2,095 MiB** across 266 segments, threshold
still at 96 MiB, 23 minutes end to end:

| | 205 MiB backlog | **2,095 MiB backlog** |
|---|---|---|
| page cache resident, peak | 83.6 MiB | **82.0 MiB** |
| `memory.current`, peak | 95.9 MiB | 96.1 MiB |
| anon, peak | 8.5 MiB | 10.5 MiB |
| dirty / writeback, peak | 8.0 / 8.0 MiB | 8.0 / 8.0 MiB |
| `high` / `max` / `oom` events | 501 / 0 / 0 | 8,062 / 0 / 0 |
| PSI stalled | 0.15 s of 360 | 0.48 s of 1,380 |
| accepted | 62,286 / 62,286 | **249,098 / 249,098** |
| dropped | 0 | 0 |

For the 480 seconds the backlog was over a gibibyte, resident file pages stayed
between 71 and 80 MiB and `memory.current` between 87 and 96 — flat, while the
queue on disk went on growing to ten times that. The collector was stalled for
under half a second in twenty-three minutes and delivered 2.84 GB with nothing
dropped.

`memory.current` peaked at 96.1 MiB against a 96 MiB threshold, which is the
point about `memory.high` made concrete: it is where reclaim starts, not a wall.
Reclaim kept it within a rounding error of the line anyway, and `memory.max`
was never in the story.

The drain moved 2,095 MiB in about 6 seconds once the sink came back. That is a
number about this rig — the sink is in the same process as the load generator —
and not about a network, but it says the backlog is not something the send path
struggles to get rid of.

One kernel, one filesystem, one disk. What generalises is the mechanism — clean
pages, inactive list — rather than the numbers.

### The connection sweep, which is its own experiment

Fixed byte rate, only the connection count varies, 120 s runs on the shipped
build:

| connections | peak anon | in-flight bytes the gate saw | peak live (instrumented) |
|---|---|---|---|
| 8 | 17.9 MiB | 0.08 MiB | 1.4 MiB |
| 64 | 40.3 MiB | — | — |
| 256 | 72.4 MiB | 9.69 MiB | **34.2 MiB** |

**Invariant 2 fails, and the instrument says why.** At 256 connections the
collector holds 34.2 MiB of live bytes at peak while its in-flight gate reports
9.69 MiB. The gap is request bodies that have been read into memory but not yet
charged: `route` collects the whole body and only then does `accept` acquire
the semaphore, so `COLLECTY_MAX_INFLIGHT_BYTES` bounds what is past the gate
and not what is buffered before it. Peak memory is therefore a function of how
many clients there are, not of the ceiling the operator declared. With the
16 MiB request ceiling the defaults allow, 64 connections could buffer a
gigabyte against a 64 MiB declared ceiling.

One hypothesis about this was measured and **rejected**: hyper's 400 kB
per-connection read buffer is not the term. Capping it at 64 kB moved 256
connections from 72.4 to 75.1 MiB, which is nothing, and the change was
reverted rather than kept for looking plausible.

**The fix is not in this series, deliberately — but it is smaller than this
document first claimed.**

Charging the gate incrementally, a chunk at a time as bytes arrive, does
deadlock: a set of requests that each hold some permits and each need more can
all wait on each other. That much stands. What does *not* follow, and what an
earlier draft of this section wrongly concluded, is that avoiding the deadlock
requires a new `503`.

**Reserving the whole request up front is deadlock-free and refuses nothing.**
Acquire before reading a byte of the body: `Content-Length` when the request
declares a valid one, `COLLECTY_MAX_REQUEST_BYTES` when it does not; poll the
body only once the permit is held; release when the export is appended. Every
request acquires all-or-nothing, so no request can hold permits while waiting
for permits, and tokio's semaphore is FIFO-fair, so the waiter at the head is
always the next served. Under pressure a body simply waits to be read, which is
backpressure the way the gate already documents it — "a request waits for room
rather than being refused".

What it costs is utilization rather than correctness. A chunked sender that
declares no length reserves the 16 MiB ceiling whatever it actually sends, so
the 64 MiB default admits four of those at once; a sender that declares its
length — which every OTLP exporter this collector has met does — reserves
exactly what it sends, and 64 MiB holds about sixteen hundred exports of the
size measured here.

So the decision left is about latency: a client whose body sits unread until
room appears may hit its own timeout, and that is a product decision about what
collecty does to a slow-consumed client. It is not a decision about whether to
start refusing.

### Against the budget

| | declared | shipped build, measured |
|---|---|---|
| steady state | ≤ 64 MiB | **18.5 MiB** at 8 connections, 300 s |
| peak | ≤ 128 MiB | **18.5 MiB**, from 60.7 before the series |
| | | both predate the mapped segment, whose drain runs sat at 8.7 MiB before the outage; a steady run has not been repeated on it |
| not a function of time | — | the 13-hour trace predates every fix; unretested |
| not a function of connections | — | **fails**: 17.9 → 72.4 MiB from 8 to 256 |
| not a function of past backlog | — | **met in the rig**: within half a megabyte on eight of nine drains since the segment was mapped; one ended 1.9 MiB up |
| the cage, which is not the heap | — | `memory.current` follows the backlog unless `memory.high` is set; with it, 2,095 MiB of queue sits in 82 MiB of cache for 0.48 s of stall in 23 minutes |

**What this series has not established.**

1. **Whether the ratchet over hours is gone.** Every run above is minutes, and
   the longest is twenty-three. The only evidence that collecty grows with time
   is thirteen hours of a build that no longer exists, and reducing a cause is
   not the same as removing an effect. This is the one question the series was
   for and the one none of its rows answers. It is a soak's to answer, and the
   next section says what that soak has to show.
2. **Whether it holds on a corpus that compresses like production's.** This
   rig achieves 3.63x where the soak's collector achieved 4.87x, so its
   segments are larger than production's. That errs towards more memory
   pressure rather than less, but it has not been shown to err only that way.
3. **Whether the reclaim result travels.** One kernel, one filesystem, one
   disk. What should generalise is the mechanism — a segment is `fsync`ed when
   it closes, so its pages are clean and on the inactive list, and reclaim can
   drop them without writing anything — not the 82 MiB.
4. **What to do about connection scaling**, which is a policy decision and is
   set out above.

And two runs that would close questions this document opens rather than
answers: `MALLOC_ARENA_MAX=1` **on top of** the spool thread, which is the
comparison the arena row cannot make; and the saturation sweep that would
decide `current_thread`.

### What the soak has to show

The optimisation is finished. What is left is a full-system soak of a day,
with **every setting fixed for its whole length** — changing the allocator,
the worker count or `memory.high` partway through spends the run.

The vehicle is `signy/scripts/run_soak_local.sh`, which already runs collecty
in a cage of its own beside signy, on disk, with a fault schedule. Run it with
collecty's cage at the recommended pair — `memory.max` 256 MiB, `memory.high`
96 MiB.

**It is not looking for another optimisation.** It is looking for a slope. The
shape to want is anon that rises and falls with the workload around a level
baseline; the shape that fails is a staircase, where each outage leaves the
floor a little higher than it found it. That is the failure the 13-hour trace
showed on the old build and the one every fix above was aimed at.

| | passes | fails |
|---|---|---|
| anon | flat baseline across quarters | rises quarter on quarter |
| `memory.current` | swings with load, baseline level | baseline climbs |
| `memory.events` `oom` / `oom_kill` | 0 | anything |
| `memory.events` `max` | 0 | anything, and `high` is fine |
| PSI | steady per outage, not accumulating | growing stall per fault |
| accepted | 100 % | any refusal that is not a declared limit |
| `dropped_bytes` | 0 | anything, since the queue is 1 GiB |
| queue after a drain | back to its pre-outage level | a floor that creeps up |
| p50 / p95 | flat across quarters | worse late than early |
| threads, file descriptors | flat | either one growing |

**Several outages, not one.** A day of clean traffic exercises none of the
paths this document spent itself on. The value is in repeating
outage → backlog → recovery → drain enough times to see whether the cycle
leaves anything behind, which is exactly what a single drain cannot say.

**The soak script now carries all of it.** `SOAK_COLLECTY_HIGH` puts a
`memory.high` on collecty's cage (96 MiB by default, and the run stops if it
did not take); `collecty.csv` gained `file`, `file_dirty`, `file_writeback`,
`threads`, `fds`, `ev_high`, `ev_max`, `ev_oom`, `oom_kills`, `psi_some_us` and
`psi_full_us`, all appended after the existing columns so the report's
positions still hold; the trend table gained `file`, `threads` and `fds` rows
and two summary lines; and `SOAK_OUTAGES` (3 by default) repeats the outage
through the last third of the run.

A fifteen-minute smoke ran the whole thing — `memory.high` applied and crossed,
3,131 `high` events with `max`, `oom` and `oom_kill` at zero, 0.94 s of PSI,
threads flat at 5, descriptors flat at 24, three outages each followed by a
recovery, nothing dropped or refused, and the report's existing sections
unchanged.

**Two things about that smoke that a day-long run has to be read differently
from.** Its queue never returned to baseline: it drains whenever signy is up —
105 → 63 MiB in one fault-free window while still ingesting — but its faults
are 45 to 90 seconds apart, and clearing an outage takes longer than that. The
numbers to size a real schedule with: ingest is about 1.6 MB/s compressed, the
queue drains net at 0.3–0.7 MB/s while ingesting, so a 180 s outage builds
roughly 280 MB and needs seven to sixteen minutes of quiet to clear. In a
day-long run the faults are hours apart and there is room; in a fifteen-minute
one there is not. And every memory row came out `GROWING`, which at that length
is the ramp from an empty process being read as a slope: the quarter-on-quarter
trend needs a run long enough that its first quarter is a steady state.

### A trap this rig fell into first

The first drain run put its queue under `/tmp`, which is tmpfs. Those pages are
shmem, they are charged to the cage, and with swap off they cannot be
reclaimed, so a backlog the collector is designed to put on disk filled the
cage and the kernel killed the process at 190 MB of queue with anon flat at
65 MiB. Worse, the rig reported that run `MEASURED`: it read `memory.events`
once, at the end, by which time the scope was gone and the OOM counter with it.

Both are fixed — the runner refuses a queue directory on tmpfs, samples the OOM
counter throughout, and fails a run whose process stopped early — and both are
recorded because a gate that can report a kill as a pass is the same class of
failure signy's `NOT_MEASURED` rule exists for.

**On a real filesystem the page cache is reclaimable and the process survives**,
but `memory.current` still reached 255.9 MiB of the 256 MiB cage while anon was
105.8. A sidecar sized to its collector's heap will meet its limit through the
queue's page cache long before the queue's own 1 GiB budget is reached. That is
a deployment fact `CONFIGURATION.md` does not currently state.
