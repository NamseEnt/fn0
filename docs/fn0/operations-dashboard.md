# Operations Console

The operations console answers one question for the fn0 Cloud operator: if a
user sent a request now, would fn0 serve it — and if not, is the problem the
runtime, dodb, object storage, telemetry, or worker capacity?

It is read-only and operator-only. It is not a project telemetry viewer
(that is #111) and not a query console.

## Architecture

```text
Browser ── Cloudflare Access ──> ops Cloudflare Worker (ops/console)
                                   ├── canary.fn0.dev (Access service token)
                                   │     └── normal fn0 ingress → ops/canary
                                   └── signy.fn0.dev (Access service token)
                                         └── platform tenant `fn0`
```

The console runs on Cloudflare, not on fn0, so it keeps answering when fn0 is
down. The browser only ever talks to the ops Worker; the Worker holds the
Signy and canary credentials and sends only the fixed queries in
`ops/console/src`.

## Canary

`ops/canary` is an ordinary Forte app deployed as project `1lmlydc3` at
`canary.fn0.dev`. Cloudflare Access refuses any request without the canary's
service token, so outside traffic cannot use up the canary project's admission
limit and make the console report a false outage.

| Probe | Path through fn0 | Success |
|---|---|---|
| `/api/runtime` | Cloudflare → NLB → worker-proxy → worker → routing → bundle → wasm | always `{"ok":true}` |
| `/api/dodb` | the runtime path + `DocDbHijack` → dodb | reads pk `fn0-ops-canary/known-value`, sk `health`, expects `fn0-canary-v1` |
| `/api/storage` | the runtime path + `ObjectStorageHijack` → project private R2 bucket | reads `canary/known-object-v1.txt`, expects `fn0-canary-v1\n` |

A probe that finds a problem answers 503 `{"ok":false,"failure":"missing" |
"mismatch" | "unavailable"}` and changes nothing. Only the admin task
`seed_known_values` writes the known values, and only
`scripts/bootstrap-fn0-ops-canary.sh` runs it. The script exits 1 for a
bootstrap failure and 2 when the bootstrap succeeded but a probe then failed.

## Platform metric contract

All from the platform tenant `fn0`. Signy stores attribute keys with `_` for
`.` (`service.instance.id` → `service_instance_id`); metric names keep their
dots. Every metric below except the collecty and Signy ones carries
`service_instance_id`, one value per worker process.

| Metric | Kind | Labels | Meaning |
|---|---|---|---|
| `fn0.platform.request.duration` | histogram (s), 13 bounds 0.005–30 | `outcome` | one guest invocation, to its response head |
| `fn0.platform.cpu_timeouts` | counter | — | a wasm instance stopped for exceeding its cumulative CPU budget |
| `fn0.platform.guest.cpu.duration` | histogram (s) | — | instance-lifetime CPU; not used by the console |
| `fn0.worker.dispatch.rejections` | counter | `reason` | a user request answered without running it |
| `fn0.failures{component=executor,error_type=deadline_exceeded}` | counter | — | a request answered 504 at its deadline |
| `fn0.worker.manifest_loaded`, `fn0.worker.draining` | gauge 0/1 | — | the loopback `/ready` and drain state |
| `fn0.worker.requests.in_flight`, `fn0.worker.websocket.connections` | gauge | — | the loopback `/status` counts |
| `collecty_queue_bytes` | gauge | — | collecty's unsent backlog |
| `collecty_queue_dropped_segments_total`, `collecty_segments_refused_total` | counter | — | telemetry lost on the way to Signy |

Signy's own health, `signy_remote_healthy` and `signy_ingest_errors_total`,
is not in any tenant; the console reads it from Signy's `/metrics`.

No platform metric carries a project, route, hostname, raw path or error
message.

## What the numbers mean

**Invocations, not HTTP requests.** `fn0.platform.request.duration` is
recorded where the guest runs, so it also counts queue tasks, WebSocket
events and cross-project invocations. It does not count:

- a 504: the deadline drops the invocation before it records anything, so the
  console reads 504s from `fn0.failures` instead;
- a 503 dispatch rejection: the request never reached a guest.

The overview therefore reports two separate ratios:

- `guest_server_error = server_error / invocations` — the application's own
  5xx;
- `unanswered = (failed + 504 + all 503 rejections) / (invocations + 504 + all
  503 rejections)` — requests fn0 got no guest answer for.

**503 rejections are split by reason.** `queue_full` is the worker's own
queue and `closed` a broken worker thread; both are platform capacity
problems. `project_admission_full` is one project reaching its own limit of 32
running and 128 waiting requests, and says nothing about the platform.

**CPU.** There is no request-level guest CPU figure: the CPU tracker belongs
to a wasm instance, not a request, and its histogram is recorded only when an
instance ends normally, which production has not recorded in a week. The
console shows `fn0.platform.cpu_timeouts` as "instance CPU budget exceeded":
an instance's cumulative CPU passed 1 s, not a single request's.

**First sample of a series.** Signy's `increase` starts from the last sample
at or before the window, so the first export of a new series — a worker
restart, or the first-ever rejection of a reason — is not counted. Counts are
lower bounds for the minute after a series starts.

**Latency quantiles.** Signy's `/metrics/quantile` cannot merge series, so the
console asks for `…_bucket` with `func=increase&agg=sum&by=le` and
interpolates p50/p95/p99 itself (`ops/console/src/quantile.ts`), with the
`histogram_quantile` convention.

## Health

`/api/live` classifies fn0 into one of four states:

| State | When |
|---|---|
| `down` | the runtime canary timed out or got an answer other than `{"ok":true}` through fn0 |
| `unknown` | the runtime canary could not be measured: Access refused the ops credential, or the request never left the console |
| `degraded` | the runtime canary succeeded, and anything else is not healthy |
| `healthy` | the runtime canary succeeded and every component is healthy |

A dodb or storage probe only counts once the runtime probe on its own
succeeded; while the runtime is down they are `unknown`.

Telemetry is `down` when Signy queries fail and `degraded` when Signy is not
ready, its R2 remote is unhealthy, collecty lost segments in the last 15
minutes, or the newest worker gauge is older than 180 s (three missed 60 s
exports). While telemetry is missing or stale, the worker and capacity
components are `unknown` rather than trusted. None of this can make fn0
`down`: Signy being unavailable is `degraded`.

Capacity is `degraded` when the last five minutes saw a `queue_full` or
`closed` rejection. A `project_admission_full` rejection does not change it.

## API

All routes are `GET`, answer JSON with `Cache-Control: no-store`, and refuse
any parameter other than `window` with 400. `window` is one of `15m`, `1h`
(default), `6h`, `24h`, `7d`; the step is 60 s, 60 s, 5 m, 15 m and 1 h, so a
series has at most 168 points. When Signy cannot be read, a route still
answers 200 with `"telemetry": "unavailable"` and `null` numbers, never zeros.

| Route | Answers |
|---|---|
| `/api/live` | the health state, each component, the three canary probes, Signy readiness, telemetry freshness, worker instances, rejections in the last 5 minutes |
| `/api/overview?window=` | invocations by outcome, 504s, rejections by reason, the two ratios, p50/p95/p99, instance CPU budget exceeded |
| `/api/series?window=` | the same, per step |
| `/api/errors?window=` | up to 50 newest `ERROR` log records from the platform tenant |

The response types are `ops/console/src/schema.ts`.
