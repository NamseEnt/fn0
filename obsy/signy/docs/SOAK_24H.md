# The 24-hour production soak

**2 GiB / 20 k eps / 24 h, faults on: PASS.** Engine `c0247b0`, production
build (mimalloc), `memory.max=2G`, `memory.high` unset, swap off, retention
30 m, collecty in front, logs and traces and metrics all on. Run `day-5-final`,
2026-09-08.

Everything below is from that run's own artefacts except where a paired
experiment is named. Nothing in the engine changed after it: every commit from
`c0247b0` to the one that records this is under `src/bin/load/` or `scripts/`.

## What it showed

**Memory does not ratchet.** Anon by six-hour quarter: 1001.7, 1005.2, 965.5,
993.8 MiB. Resident 1251.7, 1300.8, 1251.1, 1297.2. Allocator committed 2298.4,
2339.3, 2308.2, 2327.2. All flat. Anon peak 1708.3 MiB against a cgroup peak of
2050.7, **no OOM and no oom_kill**, and the memory account was never exhausted
or deferred.

**The cage was not the binding constraint.** `mem_full` 0.06 % of the day,
`io_full` 0.79 %. Seven stalls totalling 162.7 s, of which the 135 s one is the
shutdown; the rest are 4.2–6.5 s and sit in fault windows.

**Retention reaches a steady state and stays there.** 1,388 retention ticks,
1,417 metric compactions, 9,243 log merges. Parts 73/80/77/87, store 10,555 /
10,435 / 10,422 / 10,500 MiB, data directory flat beside it. The orphan grace
ledger held 1,818 entries at the end, inside the band every shorter run showed.
Zero incomplete local part directories for logs, metrics or traces. Zero error
signatures and zero `ERROR` lines in 24 hours of server log.

**Every fault came back.** Seven windows, 920 s of arranged unavailability:

| at | fault | back in |
|---|---|---|
| +8643 s | collecty SIGTERM | 1 s |
| +21600 s | signy SIGTERM | 73 s |
| +34563 s | signy down 180 s | 235 s |
| +47524 s | signy SIGKILL | 72 s |
| +60481 s | signy SIGTERM, parts on disk | 72 s |
| +68257 s | signy down 180 s | 231 s |
| +76032 s | signy down 180 s | 236 s |

The 21600 s restart is the one that ended the previous attempt (`day-4-prod`)
with `part is missing metadata`. The collector drained in 12.0 s at the end and
the cache was healthy.

**Ingest.** 1,727,987,700 events accepted. Drops over the whole run — summed
from the server log, which survives the restarts a counter does not — were one
resource at +1.6 s and nothing else; see *The startup drop* below.

## The rig, which is not the product

This soak runs on a 250 GB SATA SSD (Samsung 860 EVO) and writes about
31 MiB/s to it, which is 2.5 TB over a day on a drive that holds a tenth of
that. ext4 is mounted without `discard`, so the free extents a soak's
create/delete churn makes are not handed back to the device until `fstrim`
runs, and the weekly timer is far too slow for that rate.

Left alone, the drive degrades and takes the engine's measurements with it.
Five identically-shaped runs on 2026-09-07 had push p95 rise 215.8 → 243.8 →
289.0 → 312.5 → 358.5 ms **in wall-clock order and independent of the commit**
— the oldest binary, run last, was the worst of them. One `fstrim` discarded
63.91 GiB and put the same workload back at 8.5 ms with device write await
falling 11–12 ms → 5.3 ms; every host metric moved with it.

So the rig procedure is: **`fstrim` before a run and hourly during it**, no
containers running, and a host sampler recording `/proc/diskstats`,
dirty/writeback, CPU frequency and `k10temp` beside the cgroup's own counters.
A mid-run trim costs 45–65 s at ~93 % device utilisation and 23 % `io_full`
pressure, with no lasting latency regression. Over this soak, 24 trims
discarded 764.3 GiB and held device write await flat across the day: 5.445,
5.321, 5.268, 5.320, 5.230, 5.124 ms by four-hour block.

**This is test-rig maintenance, not a product requirement.** It exists because
an accelerated soak makes a create/delete rate no deployment does. Nothing in
signy asks for it, and no deployment guidance should carry it.

## The defect the soak found, and the fix

A metric read answered `500 {"error":"No such file or directory (os error 2)"}`
once in a three-hour production-shaped run, on `/metrics/series`, with no
matching server log line at all.

`read_chunk_bytes` opens a metric part's body by path on every chunk read
rather than holding it open, because the body is evictable and an open
descriptor would keep the bytes after the cache had given them up. The
discovery routes reach the body: a stored histogram is enumerated under the
names it answers as, which decodes its chunk — and they reach it for any query,
because the expansion runs for every histogram row whatever the caller asked
about. The scan path pins what it reads; discovery did not, so a body
`evict_metric_cache` had taken back came out as the reader's bare `ENOENT`.

`c0247b0` pins the parts discovery is about to read, restoring the bodies it
needs and holding the lock metric compaction takes to unlink a replaced tier.
The required set is narrower than the scan's on purpose: only where a histogram
row falls in the window, so a metadata question about a store of scalars
restores nothing. The chunk read's errors now name the operation, the part, the
path and the kind. A deterministic test (`query::tests::
metric_discovery_restores_a_histogram_body_the_cache_evicted`) reproduces the
exact 500 before the fix.

Over the 24 hours: 85,479 metric queries and 1,417 compactions, **no
recurrence**.

## Two things that read as defects and were not

### The trace readback's retry budget

The 24-hour run reported `missing=158` — the one number on the read path whose
whole meaning is a trace the engine acked and cannot produce. It was not that.

The readback asked once more 60 s after a 404, so a trace had 120 s of patience
against a 180 s outage plus the 157–184 s the collector then spent draining what
it had held. Traces that lost that race were counted as lost data.

Proved by putting the budget back. Same engine, same fault schedule, 90 minutes:

| retry budget | missing | gave up after budget | quota_rejected | unexpected_status |
|---|---|---|---|---|
| 5 (current) | **0** | 0 | 0 | 0 |
| 5 (current, repeat) | **0** | 0 | 3 | 0 |
| 1 (the old one) | **135** | 135 | 0 | 0 |

All 135 exhausted the budget, and their timestamps fall in exactly three
clusters — each beginning within 2 s of the engine returning from one of the
three 180-second outages and lasting 82–94 s. The four short faults (1, 18, 38,
38 s) produced none, and neither did any quiet stretch. That is the shape of a
pipeline catching up, not of data going missing.

### An instant metric query while the collector drains

`metrics_instant` answered 200 with no rows twice in the soak, and the metric
leg's own instant shape did the same. An instant query asks about the last
minute by definition and the collector delivers an outage's exports in order,
so the minute after the engine returns is a minute it legitimately has nothing
for. The collector's queue held 288 MB / 66 segments and 139 MB / 61 segments at
those two moments and was near zero within two minutes of each; the metric leg
scraped every single interval over the day (8,641 of them, none late) and lost
exactly one scrape's worth of datapoints to one failed request.

Across three faulted runs every empty answer fell in a burst starting within
2 s of recovery and lasting 50–111 s. They are classified as recovery now, with
a window that needs both halves to lapse — the shape returning rows **and** the
grace since the last unanswered read — because a single row is not proof the
pipeline has caught up. A read path that goes quiet with no outage in front of
it still fails.

### The startup drop

Every soak ever taken recorded exactly one `tenant_not_served` drop, at +1.56 to
+1.60 s, always before the first flush. collecty exports its own metrics as soon
as it is up and the harness that onboards tenants did not start until after it,
so signy refused an export naming a tenant it did not serve — exactly as
specified. The running order was the defect. `run_soak_local.sh` now onboards
the collector's tenant before starting the collector, and the compressed runs
since record zero drops.

## How a faulted run is judged now

A soak stops the engine on purpose, and the verdict used to count what that cost
as the engine being wrong. Four classes are kept apart, and two of them fail a
run:

- **errors** — the engine answered, and answered wrongly. Must be 0.
- **missing / short / unexpected_status** — a trace it acked and cannot
  produce, one produced short, a status nobody planned for. Must be 0.
- **unavailable** — nobody answered. Arranged downtime, reported beside the
  verdict rather than inside it.
- **quota_rejected** — a 429 from `max_concurrent_queries_per_tenant`. The
  limit doing its job.

The standard is **no unexpected query errors while the service was up**, not
"no query errors". On the 24-hour run the query error rate was 1.07 % on both
the log and metric legs against 1.06 % of arranged unavailability; with the
classes separated, a 90-minute run over the same seven fault types passes
twice with `errors=0`, `missing=0` and `unexpected_status=0`.

Drops are accumulated over the sampler's own scrapes, banking a generation when
a counter goes backwards, because reading them from one scrape at the end gives
the last restart's tally and calls it the run's.

## Deferred, and deliberately outside this verdict

1. **Metric compaction reads input bodies unguarded against eviction.** The log
   merge holds `deletion_lock().read()` across its rewrite;
   `SeriesRegistry` has no deletion lock and `series_merge` takes the operation
   lock only at its commit, so eviction can take a body out from under a
   rewrite in progress. Structurally possible, not observed — 1,417 compactions
   in this soak and 189 in each shorter one, without an occurrence. It would
   surface as a background-task error, not a query failure.
2. **Orphan collection only runs on ticks that retire something**
   (namse/obsy#13). `retention_once_at` returns early when nothing expired and
   the sweep is after that return, so a store whose tenants never expire never
   collects the orphans merge leaves. Not a blocker at 30-minute retention.
3. **Publish is bounded but unaccounted.** Streaming removed the whole-file
   read, so a publish no longer needs a buffer proportional to the part; the
   chunk buffers are still outside `memory_account` and its admission. A
   separate change from the streaming one, and its effect must not be mixed
   with it.
