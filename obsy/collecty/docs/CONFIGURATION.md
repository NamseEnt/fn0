# collecty configuration

Every knob is an environment variable. There is no config file and no command
line. Defaults are in `src/config.rs`, and the process refuses to start on a
combination that cannot work rather than starting and failing later.

Sizes accept a plain byte count or a binary suffix (`512`, `64KiB`, `8MiB`,
`1GiB`). Durations require a unit (`500ms`, `30s`, `5m`, `2h`); a bare number is
rejected, because a number with no unit is a guess about which unit was meant.

## Where it listens and where it puts things

| Variable | Default | What it does |
|---|---|---|
| `COLLECTY_LISTEN_ADDR` | `127.0.0.1:4318` | Where applications export to, over OTLP/HTTP. The default answers only the machine it runs on; a sidecar or host daemon other containers export to needs `0.0.0.0:4318`, which the image sets |
| `COLLECTY_DATA_DIR` | `/var/lib/collecty` | Holds the queue, under `queue/`. **This directory is the only copy of an acknowledged export until signy takes it** — it must outlive the container |
| `COLLECTY_SIGNY_URL` | `http://127.0.0.1:3100` | Where batches go. `http://` and `https://` are supported; HTTPS verifies the normal public certificate chain and hostname |
| `COLLECTY_SIGNY_ACCESS_CLIENT_ID` | unset | Cloudflare Access service-token client ID, sent as `CF-Access-Client-Id` when the secret is also set |
| `COLLECTY_SIGNY_ACCESS_CLIENT_SECRET` | unset | Cloudflare Access service-token client secret, sent as `CF-Access-Client-Secret` when the ID is also set |

The collecty ingest listener has no authentication and no TLS, so **its bind
address is the whole of the access control**. Binding a routable address
publishes an endpoint that takes anything anyone sends it; keep it inside the
same trust boundary as the applications that export to it. The outbound hop
to signy may use HTTPS and Cloudflare Access service-token headers. Set both
Access variables together; the values are never included in log messages or
delivery error reasons.

Three paths are served, and nothing else: `POST /v1/logs`, `POST /v1/traces`
and `POST /v1/metrics`. The body must be an uncompressed OTLP protobuf export
request (`Content-Type: application/x-protobuf`). The JSON encoding and a
`Content-Encoding` are both refused with `415` — see
[`ARCHITECTURE.md`](ARCHITECTURE.md) for why neither can be stored.

**This is the stack's only ingest wire, not merely collecty's.** signy takes a
collecty batch and nothing else — its OTLP push routes and its OTLP gRPC
listener were removed — so OTLP/HTTP 1.1, protobuf, uncompressed is what an
exporter must be configured for. There is nowhere to send OTLP JSON, and
nowhere to send OTLP over gRPC.

**The export has to name its tenant**, in the `tenant.id` resource attribute
signy reads it from. collecty does not check this and cannot: checking would
mean decoding, and not decoding is the point. An export that names no tenant
signy serves is accepted here, queued, shipped, and dropped on arrival —
counted in signy's `signy_ingest_dropped_resources_total` and nowhere on this
side. Configure the exporting SDK, not collecty.

## Built-in host sources

Built-in sources are disabled by default. They are intended for a per-machine
collecty deployment; a sidecar should leave them unset. Enabling either source
requires `COLLECTY_GENERATED_TELEMETRY_TENANT_ID`, which is written into the
generated OTLP resource.

| Variable | Default | What it does |
|---|---|---|
| `COLLECTY_HOST_METRICS_INTERVAL` | unset | Enables Linux host metrics at this interval, e.g. `15s`; unset disables the source |
| `COLLECTY_HOST_METRICS_ROOT` | `/` | Prefix for procfs, rootfs and hostname reads; set to `/host` when the host root is mounted there |
| `COLLECTY_JOURNAL` | `false` | Enables the systemd journal source; accepts `true`, `false`, `1`, `0`, `yes`, `no`, `on` or `off` |
| `COLLECTY_JOURNAL_DIRECTORY` | journalctl default | Journal directory passed to journalctl, useful for a read-only host mount |
| `COLLECTY_JOURNALCTL_PATH` | `journalctl` | journalctl executable used by the source |
| `COLLECTY_JOURNAL_UNITS` | unset | Comma-separated systemd units to follow; unset follows all units |
| `COLLECTY_JOURNAL_MIN_PRIORITY` | `6` (`info`) | Highest numeric priority accepted; accepts `0`–`7` or `emerg`, `alert`, `crit`, `err`, `warning`, `notice`, `info`, `debug` |

Host metrics are emitted as `system.*` OTLP metrics with `host.name` and
`tenant.id`. The current set covers CPU, memory, swap, load, filesystems, disk
I/O and network I/O. Journal records use `MESSAGE` as the body, map `PRIORITY`
to OTLP severity, and retain bounded systemd identity attributes.

The journal source stores its cursor below `COLLECTY_DATA_DIR/sources/` and
binds the checkpoint to the queue's sender identity. It appends a batch, closes
and syncs the queue segment, then atomically replaces and syncs the cursor file.
A crash may replay a batch, but cannot make a cursor point beyond durable queue
data. Queue overflow still drops whole segments and is reported by the existing
queue-drop counters.

### Host deployment access

On a native host, run collecty as an unprivileged account that can read the
journal, normally through the distribution's `systemd-journal` group. The
process does not enable a capability or change permissions itself.

In a container, mount the host procfs and root filesystem read-only at the path
selected by `COLLECTY_HOST_METRICS_ROOT`. For journald, mount the host journal
directory read-only and set `COLLECTY_JOURNAL_DIRECTORY` to that in-container
path. The container image includes `journalctl`, but the host's journal access
permissions still apply; grant only the read access needed by the enabled
source. Leave both mounts and both source settings out of a sidecar deployment.

## What bounds memory

| Variable | Default | What it does |
|---|---|---|
| `COLLECTY_MAX_REQUEST_BYTES` | `16MiB` | Largest single export accepted. Matches signy's own ceiling so an export that collecty takes is one signy can take. Refused with `413` before the body is buffered when the request declares its length, and while reading when it does not |
| `COLLECTY_MAX_INFLIGHT_BYTES` | `64MiB` | Total bytes of exports being compressed and written at once. A request waits for room rather than being refused. Must be at least `COLLECTY_MAX_REQUEST_BYTES`, or a large export could never be admitted |

Resident memory is **not** roughly this ceiling. Measured, at 204 exports/s of
41 kB over 8 connections, the collector's anonymous footprint is 8–10 MiB while
the declared ceiling is 64 MiB and the bytes actually in flight are under a
megabyte. [`MEMORY.md`](MEMORY.md) is the measurement. What it found:

- **A drained backlog no longer leaves itself behind.** It used to leave about
  4 % of what passed through, because the send path allocated a buffer the size
  of every segment; segments are mapped now and a drain ends where it started.
- **It is still a function of how many clients there are.** 18 MiB over 8
  connections, 72 MiB over 256, at the same byte rate, on a build predating
  most of the reductions above. A body is read into memory before the in-flight
  gate charges it, so `COLLECTY_MAX_INFLIGHT_BYTES` bounds what is past the
  gate and not what is buffered before it.
- **The disk backlog is charged to a container's memory limit as page cache**,
  and unless the container says otherwise it stays there: a 200 MB backlog is
  200 MiB of `memory.current`, which in a 256 MiB container leaves nothing to
  read. That is a cache and not a leak — see the next section — but it is worth
  bounding.
- **A queue directory on tmpfs is not a queue.** Those pages are shmem, they
  are charged to the container, and with swap off they cannot be reclaimed at
  all, so a backlog the collector is designed to put on disk fills the limit
  and the process is killed. Put the data directory on a real filesystem.

### Set `memory.high` below the container's limit

The one piece of container configuration worth stating outright, because
without it a healthy collector looks like a container about to die:

```
MemoryMax=256M
MemoryHigh=96M       # systemd; cgroup v2 memory.high
```

These are two different kinds of number. `memory.high` is where the kernel
starts reclaiming this cgroup and holding it up while it does — a pressure
threshold, **not a ceiling**: usage can go above it, and what happens then is
more reclaim and more throttling rather than a refusal. `memory.max` is the
ceiling, and the OOM killer is what enforces it. So `memory.high` is the
setting that keeps a container off its limit, and `memory.max` is what is left
if that does not work.

It works cheaply here. Measured over a 60 s outage: writing a **205 MiB**
backlog with the threshold at 96 MiB, `memory.current` peaked at **95.9 MiB**
with **83.6 MiB** of it page cache, the kernel reclaiming five hundred times,
the workload stalled for **0.15 seconds out of 360**, and every export
accepted. With the threshold at 48 MiB — less than a quarter of the backlog —
it peaked at 47.6 MiB on the same terms. That usage stayed under the threshold
in these runs is how well reclaim kept up, not a guarantee the setting offers.

**The resident set does not follow the backlog.** A 900 s outage building
**2 GiB** of queue across 266 segments, same 96 MiB threshold: resident page
cache peaked at **82 MiB** — the same as it was for a 205 MiB backlog — and
stayed between 71 and 80 MiB for the eight minutes the queue was over a
gibibyte, with 249,098 of 249,098 exports accepted, nothing dropped, and 0.48
seconds of stall in 23 minutes. Ten times the durable backlog is not ten times
the memory.

It is this cheap because the queue's pages are clean: a segment is `fsync`ed
when it closes, so no more than one open segment per signal is ever dirty and
reclaim never has to write anything back to take a page.

So **size the container against the collector's own footprint plus the
headroom you want to see, and set `memory.high` to it** — not against
`COLLECTY_QUEUE_MAX_BYTES`, which is a disk budget. A 5 GiB durable backlog
with 30 MiB resident is a correct state.

## What bounds disk

| Variable | Default | What it does |
|---|---|---|
| `COLLECTY_QUEUE_MAX_BYTES` | `1GiB` | The whole budget, shared by the three signals' queues rather than split between them. **This number is how long signy may be down before data is lost.** At 1 MB/s of compressed logs it is about 17 minutes |
| `COLLECTY_QUEUE_SEGMENT_BYTES` | `8MiB` | Segment close size, in compressed bytes, and therefore the unit of everything else: one request carries one segment, one `fsync` covers one, dropping under a full queue takes one at a time, and a cut delivery re-sends one. Smaller segments lose less per drop and cost more requests |

When `COLLECTY_QUEUE_MAX_BYTES` is reached the oldest segment is unlinked —
whichever signal it belongs to — and the application keeps running. Watch
`collecty_queue_dropped_bytes_total`: any movement means data was thrown away.

## What controls sending

| Variable | Default | What it does |
|---|---|---|
| `COLLECTY_SEGMENT_MAX_AGE` | `1s` | How long an open segment may keep collecting before it closes and becomes sendable. Nothing leaves the machine and nothing is on the device until a segment closes, so on a quiet host this is both the delivery latency and **the loss window for a power cut**. Each signal keeps its own age, so a quiet host closes up to three segments per interval rather than one |
| `COLLECTY_RETRY_INITIAL` | `100ms` | First backoff after signy declines |
| `COLLECTY_RETRY_MAX` | `30s` | Backoff ceiling. Doubling, with up to 25% jitter |
| `COLLECTY_SEND_TIMEOUT` | `30s` | How long one batch may wait for an answer before it counts as a retryable failure |

There is no linger and no minimum batch size: if the queue holds anything it is
sent immediately. Batches grow on their own when signy is slow, which is the only
time a larger batch helps.

## What it says about itself

| Variable | Default | What it does |
|---|---|---|
| `COLLECTY_GENERATED_TELEMETRY_TENANT_ID` | unset | Tenant for all telemetry collecty generates: `collecty_*` metrics, host metrics and journal logs. Everything it forwards carries its own tenant inside the payload, which collecty never decodes. **Unset, generated telemetry is not exported**. Validated at startup against `[a-zA-Z0-9_-]{1,64}` |
| `COLLECTY_REPORT_INTERVAL` | `60s` | How often `collecty_*` metrics are produced and the stderr summary is written |
| `COLLECTY_ZSTD_LEVEL` | `3` | 1 to 22. See the measurement below before raising it |
| `COLLECTY_LOG_FORMAT` | `text` | `json` for a log a collector will read |
| `COLLECTY_LOG` | `info` | `tracing` filter, e.g. `collecty::send=debug` |

Metrics go through collecty's own queue and land in signy as ordinary OTLP. There
is no `/metrics` port. While signy is unreachable the metrics describing that are
queued behind it, which is why the stderr summary exists.

### The metrics

One sender and one set of counters across the three queues, so every family is a
single series with no attributes.

| Family | Kind | What it answers |
|---|---|---|
| `collecty_queue_bytes` | gauge | What the segments occupy on disk, which is also how far behind signy is — an answered segment is unlinked. **The one to alert on** |
| `collecty_queue_segments` | gauge | Segment count across the three signals, never below three: each holds an open segment of its own |
| `collecty_records_appended_total` | counter | Exports accepted from applications |
| `collecty_bytes_appended_total` | counter | Plain bytes accepted, before the segment compresses them. Against `collecty_bytes_sent_total` this is the ratio this host is achieving |
| `collecty_segments_sent_total` | counter | Segments signy accepted |
| `collecty_bytes_sent_total` | counter | Compressed bytes shipped |
| `collecty_segments_refused_total` | counter | Segments **dropped** because signy would not take them. Any movement is data loss. Records signy itself drops are counted on its side, in `signy_collect_dropped_records_total`, and resources it drops for their tenant in `signy_ingest_dropped_resources_total` — neither of which moves anything here, because signy answers `200` for the segment either way |
| `collecty_bytes_refused_total` | counter | Compressed bytes dropped the same way |
| `collecty_send_retries_total` | counter | Deliveries signy declined and that were retried |
| `collecty_queue_dropped_bytes_total` | counter | Bytes **dropped** because the queue was full. Any movement is data loss |
| `collecty_queue_dropped_segments_total` | counter | Segments unlinked while full |
| `collecty_host_metrics_exports_total` | counter | Host metric exports admitted to the queue |
| `collecty_host_metrics_errors_total` | counter | Host metric collections or queue admissions that failed |
| `collecty_journal_exports_total` | counter | Journal records admitted to the queue and checkpointed |
| `collecty_journal_errors_total` | counter | Journal parse, queue, durability or reader failures |

These metrics are themselves an OTLP export, so they carry a tenant like any
other — `COLLECTY_GENERATED_TELEMETRY_TENANT_ID`, above. Without it they are not produced at all,
because signy would drop them without saying so.

Records are counted where they arrive and not where they leave: what a segment
holds is inside its compression, and counting it on the way out would mean
decompressing every segment to learn a number nothing acts on.

## Refusals at startup

The process exits with status 2 and one line on stderr when:

- `COLLECTY_MAX_INFLIGHT_BYTES` is below `COLLECTY_MAX_REQUEST_BYTES`
- `COLLECTY_QUEUE_MAX_BYTES` is below `COLLECTY_QUEUE_SEGMENT_BYTES`
- `COLLECTY_QUEUE_MAX_BYTES` cannot hold a single `COLLECTY_MAX_REQUEST_BYTES` export
- `COLLECTY_ZSTD_LEVEL` is outside 1 to 22
- `COLLECTY_GENERATED_TELEMETRY_TENANT_ID` is set to something signy would not parse as a tenant id
- a built-in source is enabled without `COLLECTY_GENERATED_TELEMETRY_TENANT_ID`
- `COLLECTY_HOST_METRICS_INTERVAL` is zero
- a size or duration cannot be parsed

Each of these would otherwise start a process that refuses or destroys every
export it is given, which is worse than not starting.
