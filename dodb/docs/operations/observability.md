# DODB observability

## Collection path

`dodb-server` exports OTLP metrics over HTTP to `127.0.0.1:4318`. A local
collecty container receives the metrics and host metrics, persists its queue in
`/var/lib/collecty`, and forwards them to Signy. Both sources use the platform
tenant `fn0`. The server resource uses `service.name=dodb-server`,
`service.instance.id=<node hostname>`, and `host.name=<node hostname>`.

The `scripts/deploy-dodb-observability.sh` operator installs or updates the
collecty environment file, unit, container, and persistent queue on the current
DODB node through the worker bastion. It reuses the pinned image and the
collecty settings used by worker hosts. It does not change Pulumi resources or
restart `dodb-server`. Run this operator only after reviewing its output and
this Phase 1 implementation report.

## Host metrics

Collecty exports host metrics every 60 seconds:

| Metric | Attributes used | Console value |
| --- | --- | --- |
| `system.cpu.time` | `host_name`, `cpu_mode` | CPU busy percent from total and idle counter increases |
| `system.memory.usage` | `host_name`, `state` | Used and available bytes |
| `system.filesystem.usage` | `host_name`, `mountpoint`, `state` | Used and available bytes; total is their sum |
| `system.disk.io` | `host_name`, `direction`, `device` | Read and write bytes per minute |
| `system.network.io` | `host_name`, `direction`, `interface` | Received and sent bytes per minute |

Collecty's resource `host.name` is not promoted to a Signy metric label, so the
host metrics exporter now also puts `host.name` on each metric datapoint. Signy
normalizes that key to `host_name`. The console matches `host_name` with
`service.instance.id`, then selects the longest reported mountpoint containing
`/var/lib/dodb`. This identifies a dedicated data mount when present and
otherwise selects `/`. The filesystem values come only from collecty;
`dodb-server` does not inspect filesystem capacity. This identity and path
selection are covered by collecty and console tests. The actual production
mountpoint and its first reported values remain to be confirmed after collecty
is installed on the node.

## DODB metrics

Counters are exported cumulatively; the console requests their five-minute
increase and presents request and byte rates per minute. Operation names are
the bounded protocol set `get`, `put`, `delete`, `query`, `scan`, and `transact`.
No tenant, project, key, or request identifiers are metric attributes.

| Metric | Type | Attributes |
| --- | --- | --- |
| `dodb.server.requests` | Counter | `operation` |
| `dodb.server.request.bytes` | Counter | none |
| `dodb.server.response.bytes` | Counter | none |
| `dodb.server.connections` | Counter | none |
| `dodb.server.connections.active` | Gauge | none |
| `dodb.server.streams.active` | Gauge | none |
| `dodb.server.protocol.errors` | Counter | none |
| `dodb.server.transport.errors` | Counter | none |
| `dodb.server.application.errors` | Counter | none |
| `dodb.server.overloaded.responses` | Counter | none |
| `dodb.storage.database.file.bytes` | Gauge | none |
| `dodb.storage.wal.file.bytes` | Gauge | none |
| `dodb.storage.shards.persisted` | Gauge | none |
| `dodb.storage.shards.open` | Gauge | none |
| `dodb.server.request.duration_bucket` | Cumulative bucket Counter | `operation`, `le` |

Persisted shards are distinct shard files found in the data directory, counting
a shard with either a database file or WAL file. Open shards are the current
in-memory service count and can be lower than persisted shards. Database and
WAL file sizes are summed separately. These four storage gauges are refreshed
once per minute using asynchronous directory inspection.

## Latency buckets

Request latency is recorded at completion as cumulative per-operation buckets;
the previous `request_latency_nanos` cumulative sum remains unchanged and is
not used to estimate quantiles. Bounds in seconds are:

`0.000025, 0.00005, 0.0001, 0.00025, 0.0005, 0.001, 0.0025, 0.005, 0.01,
0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10, +Inf`

The finite bounds cover 25 microseconds through 10 seconds, with 1-2.5-5
scaling in the latency range. This is based on checked-in Phase 3/4 benchmark
data: local PUT p50 ranged from 0.068 ms to 2.19 ms across client counts,
zero-delay transaction p50 ranged from 40 microseconds to 4.09 ms, and injected
sync transaction p50 ranged from 196.8 microseconds to 13.99 ms. The upper
finite buckets include headroom for the injected-sync tail. These benchmark
results are local measurements, not production latency evidence. The console
calculates p50/p95/p99 from bucket increases using the same aggregation method
as its existing worker latency view.

## Canary behavior

The existing read-only canary stays at
`fn0-ops-canary/known-value` / `health`. A separate write/read canary writes a
fresh random nonce and reads the same value at the fixed key
`fn0-ops-canary/write-probe` / `current`. Its outcomes are `write_failed`,
`read_failed`, `mismatch`, or success. The console exposes separate DODB read
and write health. Success means the current DODB mutation and subsequent read
path are functioning; it does not claim that an individual write's fsync or
durability has been verified.

The write/read probe currently runs on each `/api/live` request. It overwrites
one key, so key count stays fixed. Probe load and WAL churn must be observed
after rollout before deciding whether to throttle it.
