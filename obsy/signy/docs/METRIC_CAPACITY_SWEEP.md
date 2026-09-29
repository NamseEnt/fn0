# Metric active-series capacity sweep

compare/run_metric_capacity.sh measures the largest one-shot metric
population accepted by each target under the same Docker memory and memswap
limit. It runs Signy, VictoriaMetrics, and Grafana Mimir separately. Each
candidate destroys the selected target's named volume before starting it, so a
previous candidate cannot make the next candidate pass or fail.

The default experiment uses the old comparison budget and searches from 10,000
to 2,000,000 requested burst series with a 10,000-series final uncertainty:

    cd signy
    COMPARE_MEMORY=2g compare/run_metric_capacity.sh

This is intentionally a long experiment. A short smoke run that validates the
argument logic without Docker is:

    CAPACITY_SWEEP_LOWER=100 \
    CAPACITY_SWEEP_UPPER=400 \
    CAPACITY_SWEEP_TOLERANCE=100 \
    CAPACITY_SWEEP_HOLD_SECONDS=1 \
    compare/run_metric_capacity.sh --dry-run

The same options are available as command-line arguments. For example:

    compare/run_metric_capacity.sh \
      --targets 'signy victoriametrics mimir' \
      --memory 2g --lower 10000 --upper 2000000 --tolerance 10000

If another comparison stack is using the default ports, isolate the run with a
Compose project name and alternate host ports:

    COMPOSE_PROJECT_NAME=signy-metric-capacity \
    SIGNY_PORT=3111 VICTORIAMETRICS_PORT=3141 MIMIR_PORT=3151 \
    compare/run_metric_capacity.sh --targets 'signy victoriametrics mimir'

The optional probe mode is deliberately Signy-only:

    CAPACITY_SWEEP_PROBE=1 \
    compare/run_metric_capacity.sh --targets signy --lower 10000 --upper 10000

`--probe` is equivalent to `CAPACITY_SWEEP_PROBE=1`. It passes
`SIGNY_CAPACITY_PROBE=1` to the Signy container and load process and records
`probe: true` in the manifest and every trial. The script rejects probe mode
for VictoriaMetrics or Mimir. Probe mode may change Signy's admission and
memory behavior while that experimental path is being developed; treat it as
disposable-only, use a fresh output/project, and do not compare its results to
normal-mode artifacts.

When the requirement is a specific target rather than the exact OOM boundary,
set `--lower` and `--upper` to that same target. For example, the 10-million
acceptance gate is:

    compare/run_metric_capacity.sh --targets signy --probe \
      --lower 10000000 --upper 10000000 --hold-seconds 1

The search exponentially ramps candidates (lower, lower*ramp_factor, ...)
until the first failing point, then binary-searches the bracket. A candidate
passes only if the harness observed 100% series, request, and datapoint acceptance,
the target stayed alive, it was not OOM-killed, and sampled anonymous memory
was strictly below that target's cgroup memory.max. A live 429 is recorded
as safe_saturation=true, but it is not a capacity pass.

Artifacts are written incrementally and are safe to resume:

- trials.jsonl: one complete record per target/candidate, including probe mode,
  offered,
  accepted, and refused series/datapoints, HTTP status counts, anonymous
  peak, cgroup memory.peak, cgroup limit, alive/OOM state, elapsed time, and
  latency.
- trials.csv: the same flattened fields for spreadsheets.
- manifest.json: immutable experiment settings.
- state-<target>.json: the current exponential/binary bracket.
- <target>_<series>.json and logs: the raw load-harness result and stderr.

Existing target/candidate records are reused. If a run is interrupted, invoke
the same command and it will continue at the first missing candidate. Use a
new CAPACITY_SWEEP_OUT directory when changing memory, image versions,
workload knobs, or fairness assumptions.

The harness sends the exact same OTLP protobuf metric bodies to all targets.
Signy uses its collect framing; VictoriaMetrics uses
/opentelemetry/v1/metrics; Mimir uses the official /otlp/v1/metrics route
and a pinned grafana/mimir:3.1.4 monolith on local filesystem storage.
Signy's old SIGNY_MAX_ACTIVE_SERIES=500000 compose override is empty by
default, so this experiment exercises byte admission.

For fairness, the Mimir config explicitly disables non-memory cardinality and
rate guards: `max_global_series_per_user` and `max_global_series_per_metric`
are 0, while `ingestion_rate` and `ingestion_burst_size` are set to 1e9.
Mimir's documented defaults are conceptually 150000, 0, 10000/s, and 200000,
respectively. The single-process ingester also leaves `max_series` and
`max_ingestion_rate` at 0. These are benchmark settings, not recommended
production limits; see the [Mimir configuration parameters](https://grafana.com/docs/mimir/latest/configure/configuration-parameters/).
Block files and ingester TSDB files use separate directories in the fresh
`mimir-data` volume to avoid storage-role overlap.

This is an ingest capacity comparison, not a total product benchmark.
VictoriaMetrics and Mimir have different WAL/TSDB and background-flush
policies, Mimir's monolith is not a production cluster, and the local
filesystem, Docker image versions, CPU scheduling, compiler revision, request
batch size, scrape interval, and single-tenant setup all affect the number.
A one-shot burst also measures resident ingest capacity, not the number of
historical series retained on disk. Mimir is included for ingest only; its
query API is deliberately outside run_metrics.sh and this sweep. A result
within the binary-search tolerance is an interval, not an exact threshold.

## Signy raw-capacity probe (2026-08-31)

Before reintroducing a production backpressure threshold, the disposable
Signy-only probe was run with mimalloc, a 2 GiB Docker memory and memswap limit,
and the same one-shot OTLP workload. `SIGNY_CAPACITY_PROBE=1` bypasses the
application's metric cardinality, memtable, WAL-backlog, and in-flight-body
guards; the cgroup remained the hard boundary.

| requested series | accepted series | anon peak | result |
|---:|---:|---:|---|
| 2,000,000 | 2,000,013 | 1,793 MiB | pass |
| 3,000,000 | 3,000,013 | 1,800 MiB | pass |
| 3,500,000 | 3,500,013 | 1,987 MiB | pass |
| 3,750,000 | 3,670,010 | 1,972 MiB | cgroup OOM |
| 4,000,000 | 3,670,010 | 1,966 MiB | cgroup OOM |
| 10,000,000 | 3,670,010 | 2,030 MiB | cgroup OOM |

The measured raw boundary is therefore **3.5 million series, with an
uncertainty of at most 250,000** for this workload and image. The accepted
count in the failing trials is the last batch reached before the kernel killed
the process, not a valid capacity. These numbers are an OOM boundary, not a
safe production limit: flush/compaction overlap, allocator retention, query
traffic, and a different label shape can move it substantially. The artifacts
are under `compare/target/metric-capacity-probe-initial/`; use a fresh output
directory for any changed image, memory limit, or workload.

### Shape probe at the failure point

The fixed 10,000,000-series probe was repeated after the one-sample inline
buffer and probe-only structural gauges landed. It again stopped at 3,670,010
accepted series (`anon_peak_bytes=2,030 MiB`). The last successful scrape,
about one second before the kill, reported:

| gauge | value |
|---|---:|
| `signy_series_states_len` | 3,605,010 |
| `signy_series_states_capacity` | 3,670,016 |
| `signy_series_buffers_len` | 15,000 |
| `signy_series_buffers_inline` | 15,000 |
| `signy_series_flushing_series` | 1,030,000 |
| `signy_series_label_interner_len` | 3,605,010 |
| `signy_series_label_interner_capacity` | 3,670,016 |

The state-map capacity and accepted-series plateau coincide at 3,670,016
(within the sampler's one-second lag), while only 15,000 series still have
sample buffers. This identifies persistent index/interner growth and its
resize peak—not the one-sample buffer—as the next capacity limiter. The
instrumented artifacts are under
`compare/target/metric-capacity-probe-shape-10m/`.

### State-24-byte patch

The next fixed 10,000,000-series probe used the transient reservation/bounds
split (`SeriesState <= 24` bytes). It reached 6,555,010 accepted series before
cgroup OOM (`anon_peak_bytes=2,039 MiB`). The last scrape reported
`states_len=6,555,010`, `states_capacity=7,340,032`, and the same
`interner_len/capacity`; 460,000 sample buffers remained, of which 35,000 were
inline, and no flush snapshot was live. This is a substantial improvement over
the previous 3,670,010 plateau, but it is still below the 10-million target.
The artifact is under `compare/target/metric-capacity-probe-state24-10m/`.

### Catalog-source and singleton-buffer reductions

Two follow-up probes separated the remaining resident state from the
one-sample buffer representation. Sharing active labels with the part-catalog
source removed the process-wide weak interner from the cardinality path, and
demoting an aborted singleton stream back to the 16-byte inline form removed
the boxed Gorilla stream that otherwise remained after a flush abort. The
fixed 10,000,000-series probes reached, respectively:

| change | accepted series | anon peak | last state shape |
|---|---:|---:|---|
| catalog-source resolver | 6,610,010 | 2,027 MiB | `states_capacity=7,340,032`, `buffers_len=900,000`, `buffers_stream=895,000` |
| singleton demotion | 7,340,010 | 2,034 MiB | `states_capacity=7,340,032`, `buffers_len=25,000`, `buffers_inline=25,000` |

The second result moved the failure to the persistent active-series index:
nearly all accepted states were already flushed and only 25,000 inline sample
buffers remained. The artifacts are under
`compare/target/metric-capacity-probe-catalog-source-10m/` and
`compare/target/metric-capacity-probe-singleton-10m/`.

### Sharded active-series index (2026-08-31)

The `SeriesStates` index was then split into 64 ordinary `HashMap` shards
behind the existing tenant-wide lock. This limits a single resize allocation
to one shard and keeps full-label equality; it does not change the steady
state payload of a map entry. The one fixed 10,000,000-series probe still hit
the 2 GiB cgroup boundary after **6,255,010 accepted series**
(`anon_peak_bytes=2,037 MiB`, `oom_killed=true`). The last scrape reported
`states_capacity=7,340,032`, `buffers_capacity=1,835,008`,
`buffers_inline=1,090,000`, and no interner entries. Thus sharding removes
the monolithic rehash cliff but does not provide the roughly 40% payload
reduction needed for 10 million series; the next optimization target is the
per-series canonical label allocation/representation, not more HashMap
growth tuning. The artifact is under
`compare/target/metric-capacity-probe-sharded-10m/`.

### Retiring flushed identities from the active index (2026-09-01)

The sharded probe left the active-series index as the limiter, so the index
stopped keeping series it has no answer for. A gauge or cumulative series that
has been written to a part holds an entry for one thing, `last_ts`, and that
reason ends at a part boundary: the read path sorts and de-duplicates across
parts and the compactor merges their samples by timestamp. Retirement runs
after the flush's visibility transition — the parts carrying those series are
registered before their identities leave — and skips a series with samples
buffered mid-flush, a delta running total, or an admission in flight.

Three fixed 10,000,000-series probes, same 2 GiB Docker memory and memswap
limit as every row above:

| change | accepted series | anon peak | last state shape |
|---|---:|---:|---|
| retirement | 6,985,010 | 2,030 MiB | `states_len=1,050,000`, `interner_len=5,935,010` |
| bounded pool, as first written | 4,690,010 | 2,017 MiB | `states_len=1,840,000`, `interner_len=524,288` |
| bounded pool, sweep paced by attempts | **8,340,010** | 2,034 MiB | `states_len=630,000`, `interner_len=524,288` |

The first row is the change working and a second cost appearing in its place.
The index fell from 6,255,010 states to 1,050,000 — from every series ever
admitted to only those holding a sample buffer — but retirement offers every
flushed identity to the weak label pool, so a path that used to run at the
600-second idle horizon now runs at every flush. The pool took 5,935,010
entries at a capacity of 7,340,032 and gave back most of the buckets the
retirement had returned; the run gained 730,000 series where the index had
released five million entries.

The second row is a loss and is kept for what it cost to find. Bounding the
pool at 8,192 entries per shard is right, but the first version swept the
shard's dead entries on every offer once it was full, and a retirement offers
one per flushed series. The probe measured it end to end: 279 seconds instead
of 109, 1,063 requests timed out against 938 that answered, and 2.3 million
fewer series than the unbounded pool it was meant to improve on. Pacing the
sweep by attempts rather than by insertions makes a full shard decline in
constant time.

The third row is the state as it stands: **8,340,010 series, 33.3% above the
6,255,010 the sharded index reached**, with the pool holding exactly its bound
of 524,288 entries and push latency better than the baseline's (p50 25.1 ms
against 25.6, p99 406 ms against 702). The measured cost is about 256 bytes of
anonymous memory per series, against 343 before.

This does not reach the 10-million target and the next limiter is no longer in
the memtable. With the index holding only buffered series, what remains at the
failure point is the part catalogs: one `CatalogEntry` per series per part at
56 bytes, and the canonical labels those entries own — the memtable used to
share that allocation, and retirement leaves the catalog as its only owner.
Estimated from the structure rather than measured, that is roughly 1.28 GiB of
the 2 GiB at this population, so reading `index.bin` as a borrowed mmap is the
next step and the one that would also bound a residency that currently grows
with stored parts rather than with active series. The artifacts are under
`compare/target/metric-capacity-probe-retire-10m/`,
`compare/target/metric-capacity-probe-pool-10m/` and
`compare/target/metric-capacity-probe-pool2-10m/`.

### Mapping the part catalog (2026-09-01)

Retirement left the part catalogs as the whole of what a stored series costs:
one row per series per part, each owning an `Arc` to canonical labels, held on
the heap for as long as the registry held the reader. Three changes moved that
off the heap without changing what a query can answer.

`index.bin` became a 16-byte header and a fixed-stride 28-byte row array, with
the label payloads moved into a `labels.bin` beside it. The row also dropped
what it did not need: `sample_count` duplicated the Gorilla chunk's own header
under a file that already carries a checksum, the chunk length fits `u32`, and
absolute nanosecond bounds became milliseconds from the partition's midnight,
rounded outward so a stored range is never narrower than its samples. Both
files are then mapped rather than read, and a selector tests a row's label
bytes in place — a row becomes an owned identity only after it has matched,
where before every row on the walk allocated two `String`s per label. With no
catalog-owned allocation left there was nothing for the weak label pool to
hand back, so the pool and the memtable label source it was built around were
removed.

Fixed probes at the same 2 GiB Docker memory and memswap limit:

| requested series | accepted | anon peak | result | elapsed | push p50 / p99 |
|---:|---:|---:|---|---:|---:|
| 10,000,000 | 10,000,013 | 783 MiB | **pass** | 94 s | 24.7 / 246 ms |
| 20,000,000 | 20,000,013 | 1,420 MiB | **pass** | 330 s | 30.6 / 1,336 ms |
| 28,000,000 | 28,000,013 | 1,443 MiB | **pass** | 469 s | 33.4 / 1,400 ms |

**The 10-million gate is met**, at 38% of the budget and with 100% acceptance,
where the sharded index reached 6,255,010 and retirement reached 8,340,010.

Two things in that table matter more than the gate. Anonymous memory stopped
tracking series count: 8 million more series between the second and third rows
cost 23 MiB, about 3 bytes each, because the rows and their labels are no
longer anonymous at all. And **the boundary stopped being a memory boundary** —
none of these runs was OOM-killed, and the third still had 600 MiB of headroom.
What binds now is time. Ingesting 28 million series took five times as long as
ten million did, and push p99 rose from 246 ms to 1.4 seconds: page cache under
pressure is reclaimed and re-read, and the selection walk is still linear in
the rows a query's window covers. That is the trade the mapping makes, and it
is the same one VictoriaMetrics makes — an engine that gets slower under
cardinality rather than dying of it.

For scale, the comparison target OOM-killed at 16.8 million series in the same
container. This engine now accepts 28 million in 1,443 MiB without refusing a
datapoint. That is a probe-mode number and not a product capacity: the probe
bypasses the cardinality, memtable-byte, WAL-backlog and in-flight-body guards,
and normal-mode admission still charges `SERIES_OVERHEAD_BYTES = 320` against a
memtable ceiling that is a quarter of the declared budget — about 800,000
series at this container size, which is the calibration this campaign has still
not done. The artifacts are under
`compare/target/metric-capacity-probe-mmap-10m/`, `-20m/` and `-28m/`.

The next reductions are known and unmeasured: the selection walk does not
consult the bloom (only `pin_metric_parts` does), so every part whose time
range overlaps has its whole row array read whether or not it can hold the
metric; and the rows are label-sorted, so an exact `__name__` could binary
search instead of scan.

### What the product refuses at, after the charge stopped being a guess
(2026-09-01)

Every row above is probe mode, which bypasses the guards. The number that
ships is what byte admission does with them on, and until this point the two
had drifted about thirty-five fold apart: `SERIES_OVERHEAD_BYTES = 320` was
derived in M10 from a memtable that inlined four dynamic containers in every
index value and shared its label allocations with part catalogs, and none of
that had been true for several changes.

The charge is now arithmetic rather than calibration. The payload is the
allocation a canonical label actually needs — its two reference counts and
eight-byte rounding included, which `byte_len()` alone had never covered — and
the containers are read from the maps' own `capacity()` when the gate asks,
so a rehash that doubles a shard and a retirement that hands one back are both
visible at the instant they happen. There is no constant left to go stale.

Normal mode, same 2 GiB limit and workload, `SIGNY_MAX_ACTIVE_SERIES` unset so
byte admission is what decides:

| offered series | accepted | refused | 429s | anon peak | result |
|---:|---:|---:|---:|---:|---|
| 2,000,000 | 2,000,013 | 0 | 0 | 179 MiB | pass |
| 8,000,000 | 8,000,013 | 0 | 0 | 215 MiB | pass |
| 20,000,000 | 10,690,010 | 9,310,003 | 1,862 | 890 MiB | safe saturation, alive |

For the comparison this campaign started from: on 2026-08-27 the bed offered
520,288 series, and this engine refused 24,288 datapoints at its 500,000
count-guard default while peaking at 1,037.8 MiB, against VictoriaMetrics
accepting all of them at 627.3 MiB. It now takes **eight million without
refusing anything, at 215 MiB** — a fifth of the memory for fifteen times the
series — and when it is finally pushed past what it can hold it refuses with
named 429s and stays alive, which is the behaviour the claim was written
around.

The last row is the important one and it is not a pass: 20 million offered is
past the boundary, and 1,862 whole exports were refused. That is the engine
working as designed rather than a limit to remove. What the three rows
together say is that the gate and the process now agree — refusal begins at
about 10.7 million where probe mode reaches 28 million with the guards off,
so the remaining factor is roughly 2.6x rather than 35x, and it is the shared
25%-of-budget memtable ceiling rather than a stale per-series constant.
