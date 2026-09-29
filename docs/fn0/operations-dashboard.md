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

## DODB transport telemetry

`dodb.server.transport.errors` counts actionable QUIC transport failures only.
Expected peer closes, resets, stopped streams, incomplete client headers, and
idle connection timeouts are excluded. `dodb.server.transport.events` records
the bounded `stage`, `reason`, and `outcome` diagnostic combinations, including
benign disconnects. Diagnostic series are emitted only after that event occurs;
the aggregate error counter still exports a zero baseline at startup.

The earlier `transport.errors` total of 10 was collected before this meaning
was introduced and combined normal disconnects with actionable failures. It
must not be compared with post-restart totals. Reclassify only events observed
by the new process and distinguish restart warm-up from steady-state traffic.

## Canary

`ops/canary` is an ordinary Forte app deployed as project `1lmlydc3` at
`canary.fn0.dev`. Cloudflare Access refuses any request without the canary's
service token, so outside traffic cannot use up the canary project's admission
limit and make the console report a false outage.

| Probe | Path through fn0 | Success |
|---|---|---|
| `/api/runtime` | Cloudflare → NLB → worker-proxy → worker → routing → bundle → wasm | `{"ok":true}` |
| `/api/dodb` | the runtime path + `DocDbHijack` → dodb | reads pk `fn0-ops-canary/known-value`, sk `health`, expects `fn0-canary-v1` |
| `/api/dodb-write` | the runtime path + `DocDbHijack` → dodb | writes a random nonce to a unique key in pk `fn0-ops-canary/write-probe`, reads the same key and compares the exact value, then attempts a best-effort delete |
| `/api/storage` | the runtime path + `ObjectStorageHijack` → project private R2 bucket | reads `canary/known-object-v1.txt`, expects `fn0-canary-v1\n` |

The runtime probe has no dependency on the known dodb or storage values. A
non-success runtime response means the fn0 serving path is down; dodb and
storage are then unknown because their probes use that same path.

The runtime, dodb read, and storage probes do not change state. The dodb write
probe temporarily changes the canary project's DODB state: it writes a random
nonce under a unique sort key, reads the same key and compares the exact bytes,
then attempts to delete that key. This checks that a current DODB mutation and
subsequent read path are functioning; it is not an independent durability or
fsync verification. Cleanup is best effort, and stale probe keys are bounded
by age-based cleanup and a per-partition capacity guard.

A probe that finds a problem answers 503 with a bounded failure such as
`missing`, `mismatch`, `unavailable`, `write_failed`, or `read_failed`. Only
the admin task `seed_known_values` writes the known read-probe values, and only
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

## Deployment

The console is the `fn0-ops-console-u35twkcf` Cloudflare Worker. Build and
check it locally, then preview and deploy only its script from `infra/cloud`:

```sh
cd ops/console
npm ci
npm test
npx tsc --noEmit
npm run build
cd ../../infra/cloud
pulumi preview --diff --target 'urn:pulumi:prod::fn0Cloud::pkg:index:fn0-ops-console$cloudflare:index/workersScript:WorkersScript::script'
pulumi up --yes --target 'urn:pulumi:prod::fn0Cloud::pkg:index:fn0-ops-console$cloudflare:index/workersScript:WorkersScript::script'
```

Review the targeted preview and confirm that it contains only the Worker
script before applying it. Production has unrelated Pulumi drift; do not run
an untargeted `pulumi up` or use `--target-dependents`. The Worker serves the
static UI and API behind the same Cloudflare Access and JWT checks. Its
Signy and canary service credentials stay in Worker bindings and never go to
the browser.

The canary source and its known dodb/storage values are deployed or restored
with `scripts/bootstrap-fn0-ops-canary.sh`. The script is idempotent and
checks all three probes through the protected hostname.

## Production verification record

The Phase 5 console was deployed to `ops.fn0.dev` on 2026-09-28. An operator
session showed `HEALTHY` with runtime, dodb, storage, telemetry and worker
healthy. Live data refreshed about every 12 seconds and history refreshed
about every 60 seconds. A 390 px viewport had no horizontal overflow. The
browser console had no errors during the check. Hidden-tab polling was not
confirmed in the available browser session.

Phase 6 failure checks on 2026-09-28:

1. The known canary dodb value was temporarily changed. The console reported
   `DEGRADED` with the canary mismatch; runtime stayed healthy. The original
   value was restored through the bootstrap script, and the console returned
   to `HEALTHY`.
2. Starting from canary source commit `97eb08c94`, only
   `ops/canary/rs/src/apis/runtime.rs` was temporarily changed to return a
   deterministic HTTP 500. The direct protected `/api/runtime` request
   returned HTTP 500. The console then reported overall `DOWN`, runtime
   `DOWN` with `unexpected answer (HTTP 500)`, dodb and storage `UNKNOWN`, and
   telemetry and worker `HEALTHY`. No browser console warnings or errors were
   observed during this check.
3. The original runtime source was restored immediately. The bootstrap
   script redeployed the canary and seeded the known values; it reported
   matching dodb and storage values, refused an unauthenticated probe with
   HTTP 401, and verified runtime, dodb and storage each returned HTTP 200.
   The console returned to `HEALTHY` without a console error.

Signy was not stopped for this console failure check. Its unavailable state
is covered by `ops/console/tests/health.test.ts`; the production Signy outage
and queue recovery procedure is documented in
[signy-production.md](signy-production.md). Production 503 capacity
rejections were not induced.

## Known limitations

- Signy `increase` can undercount the first minute of a newly created series.
- Request-level guest CPU is unavailable; the displayed CPU timeout count is
  an instance cumulative budget signal.
- A Signy outage degrades telemetry, but the console can remain healthy or
  degraded based on runtime and other components; Signy being unavailable
  does not by itself mean fn0 is down.
- Production Pulumi drift remains outside this console deployment. Apply
  future console updates only through the targeted Worker script procedure
  above.

## Known Pulumi drift

The production stack has one accepted preview update for the Bundle Store R2
event notification (`cloudflare:index/r2BucketEventNotification:R2BucketEventNotification::events`).
The live Cloudflare rule exists with a valid rule ID, and its bucket, queue,
actions, description, prefix, and suffix match the program. Pulumi state has
the ID `missing ID`. The result is treated as a provider/state identity drift;
the rule is not applied, deleted, recreated, or removed from state. The drift
also remained with `@pulumi/cloudflare` 6.21.

The worker-site and operations-console component state was synchronized with
targeted updates after confirming that each targeted preview contained only
its component update and no child CustomResource operations. The current
`pulumi preview --refresh --diff` also plans an intentional account token
create for the Operations Console's account-scoped `Account Analytics Read`
permission and a Worker script update to bind that token and the current
console bundle. The R2 notification update remains the accepted identity drift
above. The expected plan is those two intentional changes plus the one known
R2 notification update, with zero creates, deletes, replacements, or updates
to other CustomResources. Review any additional planned CustomResource
operation before applying a stack update.

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
