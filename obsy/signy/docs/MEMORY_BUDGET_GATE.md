# The budget gate

[`VISION.md`](VISION.md) invariant I says memory is a budget you declare. This is
the test that says whether it is: it runs the engine at a declared budget under
the comparison bed's mixed load and asserts that the **cgroup's anonymous
footprint** stays under the number that was declared.

**The current, honest answer to "at what declared budget does this engine survive
its own load, settle included?" is 1792 MiB.**

Every "2 GiB" figure below it was measured with a gate that stopped when the
workload stopped, so what it answered was **"survives ingest"** — a narrower
question than it reads as. Extended through a settle, build `df6d65b` was
`OOM_KILLED` at 2 GiB: it peaked at 1702.9 MiB during ingest, comfortably
inside, and then reached 1991.5 MiB **28.6 seconds after the last row was
accepted**, while merge consolidated what ingest had left behind. The comparison
bed found that first and the gate now reproduces it in ninety seconds.

The term was merge, holding **829 of 847 live megabytes** at the settle peak
because a rewrite materialized its whole group first
([`MEMORY_ATTRIBUTION.md`](MEMORY_ATTRIBUTION.md)). Streaming the rewrite is what
moved the number: on `3ca3bb8`, 2 GiB is green at 89 % and 1792 MiB — red even on
ingest alone before any of this — is green at 95 %, with the ingest and settle
phase peaks now equal in every passing run.

Shrinking the merge *budget* instead was tried first, measured **worse**, and
reverted; that is recorded below, because a direction that ought to have worked
and did not is worth more than the diff would have been.

The progression below is real and none of it is retracted; it is simply measuring
the ingest phase, and it says so. Against that phase: 5 GiB on `50190cf`, 2 GiB
at 90–96 % on `9199e07` after sharing label sets, 2 GiB at 78–83 % on `df6d65b`
after the streaming top-K executor.

This gate landed before the fixes on purpose, and that is why the move can be
stated as a number at all. [`MEMORY_ATTRIBUTION.md`](MEMORY_ATTRIBUTION.md)
established that the engine's own accounting reported 669 MiB live and 111 MB of
memtable at the instant the kernel killed it, so **every other M10 item is a fix
and this is the only thing that can say whether a fix worked.**

---

## Running it

```
cargo run --release --bin memory_gate -- --budget 2GiB
```

One command, one machine-readable verdict, and an exit code per outcome. It
builds the release binaries, puts the server in a `systemd-run --user --scope`
cgroup at `MemoryMax=<budget>` with `MemorySwapMax=0` — the same native cgroup v2
mechanism [`../scripts/run_memprof_local.sh`](../scripts/run_memprof_local.sh)
establishes, and the same zero swap the comparison bed's
`memswap_limit == mem_limit` means — drives it with the M8 harness, and samples
`memory.stat` at 4 Hz.

`--help` lists every knob; all of them default to the comparison bed's values
(1.2 M events, 20 k eps offered over 8 connections, 5 qps of queries over their
own 4 connections, seed 1592598566). `--limit` sets the cgroup's `memory.max`
separately from the declared budget, which is how an overshoot is measured
instead of merely being fatal. `--server-env K=V` reaches the server, which is
how `SIGNY_MEMORY_BUDGET` will be handed to this gate when it exists.

Output lands in `target/memory_gate/<name>/`: `gate.json` (the verdict and every
number behind it), `anon.csv` (the sampled series), `load.json` (the harness's own
report), `server.log`, `harness.log`.

**The workload is reads concurrent with writes, and `--query-eps 0` is refused.**
The attribution measured query at 57–77 % of all allocation traffic, and
allocation traffic is what the allocator retains, so an ingest-only run is a
different experiment. The shape is the one `compare/` ran, so the gate's number
and the published one are comparable.

---

## What it validates against, and what it refuses to

**Peak `anon` from the cgroup's `memory.stat`. Only that.**

The engine's own view is the thing being audited, so it cannot be the auditor: at
the 2 GiB kill it read a third full. `memory.peak` is not it either — it includes
the page cache this engine's own Parquet and WAL writes create, which is
reclaimable and is not what an OOM kill is decided on. Both are recorded beside
the verdict; neither is gated.

One row of one passing run, at a 5 GiB budget on build `50190cf`, is the whole
argument:

| in the same run | MiB |
|---|---|
| peak cgroup `anon` — what the kernel decides on | **4675** |
| peak cgroup `memory.peak` — includes page cache | 5120 (the limit) |
| peak `signy_memtable_bytes` — what M9 had to reason with | **193** |

The same three, from the `shared-labels-2g` run on build `9199e07`, say it again
at a quarter of the budget and with the spread between the three unchanged in
shape:

| in the same run | MiB |
|---|---|
| peak cgroup `anon` | **1916** |
| peak cgroup `memory.peak` | 2048 (the limit) |
| peak `signy_memtable_bytes` | **182** |

And once more from `streaming-topk-2g` on build `df6d65b`, where the anonymous
peak has come off the ceiling and `memory.peak` therefore no longer reads as the
limit:

| in the same run | MiB |
|---|---|
| peak cgroup `anon` | **1631** |
| peak cgroup `memory.peak` | 1991 |
| peak `signy_memtable_bytes` | **168** |

---

## The four outcomes

Distinguishable by exit code, and each one below was produced by a real run
rather than asserted:

| exit | verdict | means | seen in |
|---|---|---|---|
| 0 | `UNDER_BUDGET` | survived, delivered the offered workload, peak `anon` ≤ budget | the 5 and 6 GiB runs on `50190cf`; the 2 GiB runs on `9199e07` and `df6d65b` |
| 2 | `OVER_BUDGET` | survived, peak `anon` exceeded the declared budget | `--budget 2GiB --limit 8GiB` on `50190cf`; the same command is `UNDER_BUDGET` on `9199e07` and `df6d65b` |
| 3 | `OOM_KILLED` | the kernel killed it inside its own declared budget | the 2 and 4 GiB runs on `50190cf`; 1 GiB, 1536 MiB and 1792 MiB on `9199e07`; 1792 MiB on `df6d65b` |
| 4 | `NOT_MEASURED` | the measurement did not happen | `--seconds 8`, which delivers 13 % of the offered events |

**`NOT_MEASURED` is a failure, not a skip.** [`LOAD_RESULTS.md`](LOAD_RESULTS.md)
§3 is a peak RSS that had never been measured, written down as an engine result,
and the rule it arrived at is that a gate that cannot measure must not pass. So
each of these fails with a stated reason rather than reporting a budget: no
cgroup v2, no systemd user manager, a `memory.max` that is not the number the run
asked for, swap not disabled, the server never becoming ready, the server exiting
without a cgroup OOM event (a crash is not a budget result), zero samples of
`memory.stat`, no harness result, and — the one that matters for the fixes
ahead — **the engine accepting less than 90 % of the offered events**, because a
budget that is met by refusing the load was never exercised.

`OVER_BUDGET` is only reachable when `--limit` exceeds `--budget`. With the two
equal, which is the default and what an operator does, the kernel enforces the
ceiling before `anon` can cross it, so the honest verdict at a budget this engine
cannot hold is `OOM_KILLED`.

An OOM kill is established by two independent sources, recorded separately:
`oom_kill` in the cgroup's `memory.events`, and systemd's own `Result=oom-kill`
on the scope. In build `50190cf`'s 2 GiB run both fired, and the process also
exited on `SIGKILL`.

**The two sources do not always agree, and both are kept for that reason.** In
build `9199e07`'s 1 GiB run the cgroup's `oom_kill` counter read **0** while
systemd reported `Result=oom-kill` and the process exited on `SIGKILL`; the
1536 MiB and 1792 MiB runs had all three. A verdict resting on the cgroup counter
alone would have called that run a crash rather than a budget failure.

---

## The baseline, before invariant II

Build `50190cf`, default features (**not** `memprof` — its 16-byte tag per live
allocation was 66–268 MiB in the attribution's runs). Default glibc, no
`MALLOC_*` tuning. Machine: Linux 6.12.57, 12 logical CPUs, 31.3 GiB RAM. Every
run ingests 1.2 M events at an offered 20 k eps over 8 connections with 5 qps of
queries on 4 more.

| declared budget | cgroup limit | verdict | `anon` peak (MiB) | share of budget | at | achieved eps | queries |
|---|---|---|---|---|---|---|---|
| 2 GiB | 2 GiB | **`OOM_KILLED`** | 1845 | 90 % | 49.3 s | — | — |
| 4 GiB | 4 GiB | **`OOM_KILLED`** | 4082 | 99.7 % | 60.8 s | — | — |
| **5 GiB** | 5 GiB | `UNDER_BUDGET` | **4675** | **91 %** | 65.3 s | 18 875 | 273 |
| 5 GiB (repeat) | 5 GiB | `UNDER_BUDGET` | 4535 | 89 % | 65.8 s | 18 761 | — |
| 5 GiB (repeat) | 5 GiB | `UNDER_BUDGET` | 4392 | 86 % | 65.3 s | 18 820 | — |
| 6 GiB | 6 GiB | `UNDER_BUDGET` | 4545 | 74 % | 65.3 s | 18 830 | 271 |
| 2 GiB | 8 GiB | **`OVER_BUDGET`** | 4586 | **224 %** | 64.5 s | 18 711 | 271 |

Three things this says beyond the headline.

**The overshoot at the bed's limit is 2.24×.** Given 8 GiB of room and asked to
stay inside 2 GiB, the engine's anonymous peak is 4586 MiB. That is the size of
the gap the M10 and M11 fixes have to close, and it is a measurement rather than
an extrapolation from a run that died.

**Above 5 GiB the peak stops tracking the limit.** 4392–4675 MiB at a 5 GiB
limit, 4545 at 6 GiB, 4586 at 8 GiB: this workload's own anonymous high-water is
about **4.5 GiB**, and it is flat in the headroom it is given. Below that, the
peak *is* the limit — 99.7 % of it at 4 GiB, and 90 % at 2 GiB with 191 MiB of
page cache making up the rest — which is what dying at the ceiling looks like. The
attribution's 8 GiB run reached 4937 MiB on a `memprof` build carrying its own
instrument, and 4586 MiB here is the same number without it.

**5 GiB is not a recommendation.** It is the smallest whole GiB that survives,
and it survives at 86–91 % of itself with a 283 MiB spread across three
identical runs. Nobody should deploy on 9 % headroom. It is where invariant I
stood at build `50190cf`, and the number M10 and M11 had to move.

---

## After sharing label sets

Build `9199e07`, same features, same glibc, same machine, same workload, same
seed. The one change between the two tables is
[`VISION.md`](VISION.md) invariant II's first step: `Labels` is reached through
`Arc<Labels>` from the memtable to the query result, so a label set is shared by
a stream's rows instead of copied into each of them. Nothing else on the M10 or
M11 lists is built — `normal_scan_limit` is still `usize::MAX`, there is still no
projection pushdown, `entries_bytes` still under-reports, no arena exists, and no
`MALLOC_*` variable is set.

| declared budget | cgroup limit | verdict | `anon` peak (MiB) | share of budget | at | achieved eps | queries |
|---|---|---|---|---|---|---|---|
| 1 GiB | 1 GiB | **`OOM_KILLED`** | 941 | 92 % | 48.8 s | — | — |
| 1536 MiB | 1536 MiB | **`OOM_KILLED`** | 1530 | 99.6 % | 52.5 s | — | — |
| 1792 MiB | 1792 MiB | **`OOM_KILLED`** | 1734 | 96.7 % | 59.8 s | — | — |
| **2 GiB** | 2 GiB | `UNDER_BUDGET` | **1916** | **93.5 %** | 62.3 s | 19 724 | 302 |
| 2 GiB (repeat) | 2 GiB | `UNDER_BUDGET` | 1842 | 90.0 % | 62.3 s | 19 720 | 302 |
| 2 GiB (repeat) | 2 GiB | `UNDER_BUDGET` | 1965 | 95.9 % | 62.3 s | 19 723 | 302 |
| 2 GiB | 8 GiB | `UNDER_BUDGET` | 1913 | **93.4 %** | 62.3 s | 19 729 | 302 |

**The overshoot is gone rather than smaller.** The row that measures it is the
last one: given 8 GiB of room and asked to stay inside 2 GiB, the anonymous peak
was **4586 MiB — 2.24×** before, and is **1913 MiB — 0.93×** now. That is the
same experiment, not a different one, and it is the honest form of the claim
because the kernel is not enforcing the ceiling in it. The workload's own
anonymous high-water fell from about **4.5 GiB to about 1.9 GiB, 2.4×**.

**The delivered load went up, not down.** 19 720–19 729 eps against 18 711–18 875
before, and 302 queries answered against 271–273, so this is not a budget met by
refusing work — which is the failure mode the 90 % delivery floor exists to
catch. `delivered_fraction` is 1.001 in every passing run.

**The floor is between 1792 MiB and 2 GiB, and it is not sharp.** The three
killed runs die at 92–99.6 % of their limits, and 1792 MiB dies at t≈60 s having
reached only 96.7 % of itself — the last reading before death, which is a lower
bound. A budget this close to the workload's own high-water is decided by where
the allocator happens to be when the merge tick lands, not by a threshold.

**2 GiB is still not a recommendation.** It survives at 90–96 % of itself with a
123 MiB spread across three identical runs, which is less headroom than the 5 GiB
runs had and the same objection applies: nobody should deploy on 6 %. The rest of
invariant II — the streaming top-K executor, `normal_scan_limit`, the free
memcpys, projection pushdown — is what would buy the headroom, and the
attribution says where: at build `50190cf` **44 % of the anonymous peak was
memory the process had already freed**, and that fraction is a function of
allocation traffic, which those items are about.

---

## After the streaming top-K executor

Build `df6d65b`, same features, same glibc, same machine, same workload, same
seed, no `MALLOC_*` variable set. The one change between this table and the last
is [`VISION.md`](VISION.md) invariant II's second step: `normal_scan_limit =
usize::MAX` is gone, the reader, the registry and the memtable stream rows into a
bounded top-K sink instead of each materializing and sorting the whole match set,
and a scan stops once it holds `limit` rows that survived the pipeline. Nothing
else on the M10 or M11 lists is built — there is still no projection pushdown,
`entries_bytes` still under-reports, no arena exists, and the two free memcpys
are still there.

| declared budget | cgroup limit | verdict | `anon` peak (MiB) | share of budget | at | achieved eps | queries |
|---|---|---|---|---|---|---|---|
| 1792 MiB | 1792 MiB | **`OOM_KILLED`** | 1691 | 94.4 % | 57.5 s | — | — |
| **2 GiB** | 2 GiB | `UNDER_BUDGET` | **1631** | **79.6 %** | 61.8 s | 19 871 | 302 |
| 2 GiB (repeat) | 2 GiB | `UNDER_BUDGET` | 1596 | 77.9 % | 60.5 s | 19 878 | 302 |
| 2 GiB (repeat) | 2 GiB | `UNDER_BUDGET` | 1706 | 83.3 % | 61.8 s | 19 876 | 302 |
| 2 GiB | 8 GiB | `UNDER_BUDGET` | 1659 | **81.0 %** | 61.8 s | 19 889 | 302 |

**The margin widened; the surviving budget did not fall.** 78–83 % of 2 GiB
against 90–96 %, and the workload's own anonymous high-water — the `--limit 8GiB`
row, where the kernel is not enforcing anything — is **1659 MiB against
1913 MiB**, 0.81× the declared budget against 0.93×. But 1792 MiB is still
`OOM_KILLED`, at 94.4 % of itself and 62 s in, so the smallest whole step this
engine survives its own load at is unchanged. Both facts are the answer to "does
the gate margin widen or does the surviving budget fall": the first, not the
second.

**Why the two do not move together.** The three killed and passing runs put this
workload's high-water at about 1.6–1.7 GiB, and 1792 MiB is inside that band
rather than above it — the 1792 MiB run reached 1691 MiB before it died, which is
*more* than the 1631 MiB the 2 GiB run peaked at. A budget within one merge
tick's worth of the high-water is decided by where the allocator happens to be,
which is the same thing the previous table's floor paragraph says. The next whole
step down, 1536 MiB, was not run: it was already killed at 99.6 % of itself on
`9199e07` and nothing here predicts a 200 MiB drop.

**The delivered load went up again, slightly.** 19 871–19 889 eps against
19 720–19 729, and 302 queries answered in every run. `delivered_fraction` is
1.0007–1.0009, so this is again not a budget met by refusing work.

**What this table does not say.** It is a peak over four runs of one workload,
and the query mix is the bed's — three parts label-only, three `|=`, three
`| json | field=`, two `rate()`, one restore probe, at 5 qps. The streaming
executor's effect on allocation traffic is largest on the shapes with a small
limit over a large window, and this workload's queries use a 60 s window and a
limit of 100 against a dataset that grows past it during the run, so the gate is
not the place to read the size of that effect. `benches/query.rs` is
([`../todo.md`](../todo.md), M11 read path).

---

## The gate stops one phase too early — found by the comparison bed, 2026-07-30

**The gate passes at 2 GiB and the engine still dies at 2 GiB.** Both are true,
and the difference is the gate's blind spot rather than a disagreement between
two measurements.

An attempt to regenerate [`COMPARISON.md`](COMPARISON.md) at revision `5f1e9a2`
was OOM-killed in its 2 GiB container (`OOMKilled: true`, exit 137). The
sequence, from the container's own log and the harness's result files:

| time (KST) | what |
|---|---|
| 14:44–14:45 | ingest phase, 1.2 M events — completed |
| 14:45:19 | seed phase, 150,000 verification rows — **`PASS`**, 1,504 pushes, 0 errors, all `204` |
| 14:45:30 | **merge completed: 6 parts into 1** |
| 14:45:32 | flush, 78,808 rows |
| 14:45:47 | **OOM-killed** |

It died **fifteen seconds after the last row was accepted**, in the idle settle,
while merge consolidated the parts that ingest and the seed had left behind. Not
during ingest. Not during the push. During the catch-up.

The gate never sees this, because the gate's workload ends when ingest ends. It
measures the peak of *accepting* load and then stops, so the merge backlog that
load leaves behind is outside every number in this document. That is the same
shape of defect as every other one this project has found: an instrument that
reports a pass over the half of the problem it happens to look at. The
[`MEMORY_ATTRIBUTION.md`](MEMORY_ATTRIBUTION.md) figure that predicted it is
already recorded — one merge group's rewrite was the **largest single live term
at 771 MiB**, against a `merge_max_memory_bytes` default of **1 GiB, half of a
2 GiB container**, derived from no number the operator gave.

So the honest reading of this document's "smallest surviving budget" is **the
smallest budget that survives ingest**, and it is not the same question as
whether the engine fits a container. Fixing the gate is a precondition for the
arena work, exactly as building the gate was a precondition for the fixes:

- The gate must run through a settle with merge active, and gate on the peak
  across it.
- `merge_max_memory_bytes` must derive from the declared budget instead of
  defaulting to half a small container.

A second axis surfaced in the same run and is **not** what killed it, but was
never published either: in the load phase — queries concurrent with 20 k eps of
ingest — query response p95 was **22.9 s** today at 2 GiB, and was **5.7 s at
2 GiB and 27.2 s at 8 GiB** in the M9 artifacts (`docs/artifacts/m9/`). The
published comparison table never showed this, because its query columns come
from the matrix phase: one connection, a small dataset, no ingest running. Query
latency *under* ingest is a third measured axis with no gate on it.

## After streaming the merge

The settle attribution named the term: merge held **829 of 847 live megabytes**
at the settle peak, because `rewrite_group` materialized the whole group as a
`Vec<Row>` before writing anything. It now streams — a k-way merge over the
group's parts feeds a part writer that holds one row group — so nothing
accumulates. Build `3ca3bb8`, same command, seed, machine and workload.

| declared budget | verdict | `anon` peak | share | ingest phase | settle phase | achieved eps |
|---|---|---|---|---|---|---|
| 1536 MiB | **`OOM_KILLED`** | 1501 MiB | 98 % | 1501 | — | — |
| 1792 MiB (`sm2_1792MiB`) | `UNDER_BUDGET` | 1709 MiB | 95 % | 1709 | 1709 | 19 891 |
| 2 GiB (`stream_merge_2GiB`) | `UNDER_BUDGET` | 1817 MiB | 89 % | 1817 | 1817 | 19 876 |
| 2 GiB (`sm2_2GiB`) | `UNDER_BUDGET` | 1832 MiB | 89 % | 1832 | 1832 | 19 874 |

**2 GiB was `OOM_KILLED` through the settle before this and is green after**, and
1792 MiB — red even on ingest alone before — is green too. The smallest surviving
budget is **1792 MiB**, with 1536 MiB still red.

The clearest signal is not the totals but the two phase columns: they are now
**equal in every passing run**. The settle no longer adds anything, which is what
"merge stopped accumulating" looks like from outside. Before, the settle added
about 290 MiB on top of ingest and that addition is what killed the process.

Delivered load is unchanged at 19.87–19.89 k eps, so this is not a budget met by
refusing work.

Two things it does not establish. The gate's spread at these budgets has been
wide before, and two runs at 2 GiB agreeing to 15 MiB is better than the earlier
spread but is still two runs. And `merge_max_memory_bytes` no longer bounds
anything, so `peak_materialized_bytes` — which still adds it — now overstates;
correcting that is a claim about memory and wants its own measurement.

## Shrinking the merge budget was tried, and it made things worse

The obvious fix from the section above — derive `merge_max_memory_bytes` from
the container instead of leaving it a constant 1 GiB — was implemented and
measured with the extended gate at 2 GiB. **It failed, and it moved the failure
earlier.** Two runs of each, build `d59706f`:

| merge budget | verdict | peak phase | ingest peak | settle peak |
|---|---|---|---|---|
| derived, 512 MiB (`settle_merge_fix`, `derived_2`) | `OOM_KILLED` | **ingest** | 2008.8 / 1970 MiB | never reached |
| built-in, 1 GiB (`settle_red`, `old1g_2`) | `OOM_KILLED` | settle | 1702.9 / 1682 MiB | 1991.5 / 1954 MiB |

Derivation worked exactly as intended — the server logged
`detected_memory_limit_bytes=2147483648`, `detected_memory_source="cgroup v2
memory.max"`, `merge_max_memory_bytes=536870912`. The engine still died, sooner,
with an ingest-phase peak about **290 MiB higher**.

The reading: smaller groups mean more merges, and more merges overlap ingest.
`docs/PRODUCTION_READINESS_REVIEW_2026-07-26.md` recorded that contention as N8
and measured it as costing backlog depth; here it costs resident memory during
the phase that was previously the safe one. **The merge budget is not the
binding term at 2 GiB.** The attribution's 771 MiB merge high-water is real and
was measured in a different configuration; it is not what this workload dies of.

Two caveats on the experiment. It changed two values at once —
`merge_max_input_bytes` is derived from the memory budget, so the 512 MiB arm
also halved the selection limit to 256 MiB, and the two are not separated here.
And run-to-run spread at this budget is wide: earlier 2 GiB runs read
1842–1965 MiB. Two runs per arm is enough to see a 290 MiB shift in *which
phase* peaks, and not enough to put a number on it.

**The change was reverted.** Keeping a default that a measurement says is worse,
because it is the direction that ought to help, is the mistake this repository
keeps declining to make. What was kept is the detection and the startup log —
`detected_memory_limit_bytes` and `detected_memory_source` now appear beside
`peak_materialized_bytes`, because an operator otherwise has no way to learn
what the process is inside. That is observability with no behavioural change.

**So the smallest budget this engine survives its own load at, measured through
the settle, is not yet known and is above 2 GiB.** The figure this document
previously reported — 2 GiB at 81% — is the ingest phase alone.

## A bench run on a busy machine invented a 41 % regression

Worth recording because it nearly went into a document. `part/write/json` was
measured while the 40-minute comparison bed was running in Docker on the same
host, and criterion reported **+41 %, p < 0.05, "Performance has regressed"** —
with a confidence interval spanning +23 % to +60 %, which is the tell.

Re-measured on an idle machine against the same code: `part/write/plain`
134.51 ms and `part/write/json` 321.28 ms, against 134.34 and 328.20 before
streaming the merge. The batch write path is **unchanged within noise**, which is
what a pure extraction of `encode_group_blooms` should cost.

Criterion then said "Performance has improved by 30 %", which is equally wrong:
its saved baseline was the contaminated run. A stored baseline is only as good as
the machine that produced it.

This is the same failure as `LOAD_RESULTS.md` §10, where a two-hour soak ran on a
machine that was compiling for most of it. The rule that follows is narrow and
worth keeping: **a timing taken while something else is running is not a
measurement**, and a wide confidence interval is the signal to throw it away
rather than to report it with a caveat.

## What this does not establish

Stated in the discipline [`COMPARISON.md`](COMPARISON.md) sets.

* **It is not the Docker bed.** A native cgroup v2 scope with the same
  `memory.max`, the same zero swap and the same `anon` — a container of the same
  kind, not the same container. It also achieves 18.7–19.7 k eps where the
  published Docker run achieved 16.8 k, so the two beds are not
  interchangeable and this one is the easier of them.
* **Sampling is 4 Hz**, so a spike shorter than 250 ms between samples is not in
  the series. A pass by less than one sample's worth of slope is therefore not
  proven; a *failure* is not affected, because a kill is the kernel's verdict and
  not the sampler's. The killed runs' peaks are the last reading before death and
  are lower bounds.
* **On a killed run the harness's own numbers are lost.** It is killed three
  seconds after the server dies, before it writes its report, so the ingest and
  query columns are empty for those rows and `delivered_fraction` reads zero.
  The verdict does not depend on them — an OOM kill outranks everything
  downstream of a dead server — but the rows are less informative than the
  surviving ones.
* **One workload, one corpus, one machine, one duration.** The run ends at 1.2 M
  events, i.e. about 65 s and a few dozen parts. A longer run reaches more parts,
  more merges and more sidecars, and nothing here says the 4.5 GiB high-water of
  `50190cf` or the 1.9 GiB of `9199e07` is the ceiling of a run that lasts an
  hour. `LOAD_RESULTS.md` §10's soak is the shape that would answer that.
* **It measures a peak, not a distribution.** Three runs at 5 GiB is enough to
  see a 283 MiB spread and not enough to put a bound on it; three at 2 GiB see
  123 MiB and say no more.
* **Two builds, one variable, and no proof it was the only one.** Everything
  between `50190cf` and `9199e07` is in the second table, not just the label
  sharing: the `end`-exclusive fix, the extracted-field placement, the index
  sidecar merge. None of them is a memory change and the bench tables attribute
  the drop to the label sets, but this document measured the pair of builds and
  not the pair of diffs.
* **The 90 % delivery floor is a judgement.** A run that delivered 89 % of its
  offered events would be reported as unmeasured; the number is chosen to reject
  a budget met by refusal, not calibrated against anything.

---

## Where it does not run

**Not in CI, and not on every push.** It needs a cgroup v2 scope, a systemd user
manager and minutes per run, and a peak memory figure from a shared runner whose
neighbours are invisible is exactly the kind of number this repository has
already published once and retired. CI does compile it —
`clippy --all-targets -D warnings` and `cargo test` both build the target — so it
cannot rot the way a script and a document did; and a run in an environment that
cannot give it a cgroup exits 4 rather than passing.

It is a binary and not a `#[test]` for the same reason: `cargo test` stays a
second long, and the verdict stays typed, compiled and gated instead of living in
`awk` beside a table someone has to remember to update.

---

## When the budget knob lands

`SIGNY_MEMORY_BUDGET` does not exist yet — it is the last step of M10, not
this one — so the gate takes the budget as its own input. That is deliberately
the contract the knob will have to satisfy: *given one declared number, peak
`anon` stays under it while the offered workload is delivered.* Until then the
knob can be exercised through `--server-env SIGNY_MEMORY_BUDGET=...` without
changing this program; once it exists, the gate should read it from the server it
started rather than accept it twice.
