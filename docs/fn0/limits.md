# Limits & Quotas

Every limit that applies to fn0 Cloud, in one place. Self-hosted fn0 has none
of these except the per-project compute egress quota and the outbound
destination policy, which the worker enforces wherever it runs; run it yourself
and the rest are yours to change.

fn0 Cloud is not open yet; values below are the planned launch limits and may
be adjusted before general availability.

## Per-request runtime limits

These apply to every invocation — HTTP requests, queue tasks, and cron runs —
on every plan.

| Limit | Value |
| --- | --- |
| CPU time | 50 ms |
| Memory | 128 MB |
| Max duration | 15 seconds |
| Request headers | 128 KB |
| Request body | 100 MB |
| Response headers | 128 KB |
| Response body | Unlimited |
| Subrequests | 50 per request |

CPU time counts only time spent executing your code. Waiting on I/O — a
database query, an upstream API call, a slow LLM response — costs nothing.

The 100 MB request-body value is a transport-size limit, not an in-memory
buffering allowance. Request and response bodies must remain backpressured
streams end to end. Buffering a body is still subject to the 128 MB application
memory limit, and every request remains subject to the CPU and duration limits.
Presigned object-storage URLs are recommended for durable file uploads, but
applications may stream large bodies through compute when needed. Request and
response streaming is implemented end to end; see the
[HTTP body streaming design](../design/http-body-streaming.md).

A request larger than the limit is refused before your application runs. Through
fn0's edge the refusal comes from the edge itself, so the 413 your client sees
is an HTML page rather than the application's own response.

The unlimited response-body value is a size limit, not a time allowance. A
response streams under backpressure within the same 15-second duration, so a
client that cannot read the whole body before the deadline has its response cut
off. Serve downloads that need longer than that from object storage instead of
through compute.

Guest-to-host control requests have smaller buffering limits than the general
transport limit. These are worker safety bounds rather than billing quotas;
self-hosted operators can change them in the worker source.

| Operation | Request body |
| --- | ---: |
| Queue enqueue and cross-project enqueue | 128 KiB |
| Vault operations | 16 KiB |
| Static page cache purge | 256 KiB |
| WebSocket singleton connect | 256 KiB |
| WebSocket singleton activation and abort | 16 KiB |
| OTLP metrics, logs, and traces export | 8 MiB |

The worker enforces these values while reading each body, including bodies
without a declared length. An oversized control request receives HTTP 413 from
the guest's outbound call. Object storage and regular WebSocket message bodies
remain streamed and are not subject to these buffering limits.

The first request to reach a cold JavaScript instance runs on a much larger
allowance, because it also pays for module instantiation and the renderer's
first uncompiled pass — work later requests inherit for free. Requests that
arrive while an instance is still cold get the same allowance. The limit above
is what a warm instance is held to.

## WebSocket limits

| Limit | Value |
| --- | --- |
| Inbound message | No explicit fn0 byte cap |
| Outbound message | No explicit fn0 byte cap |
| Waiting inbound messages per connection | 4 |
| Active outbound send per connection | 1 |
| Waiting outbound sends per connection | 4 |
| Active invocations per project per worker | 32 |
| Waiting invocations per project per worker | 128 |
| Connections per project per worker | 1,000, provisional |
| Connections per worker process | 10,000, provisional |
| Singleton declarations per project | 1,000 |

Outbound bodies stream without a declared length. Inbound and outbound messages remain subject to
transport backpressure, available worker and application memory, the per-connection queue limits,
and the invocation's remaining 15-second duration. Inbound callback dispatch materializes a
complete message before the application sees it. The absence of an explicit byte cap is not a
promise of unlimited memory or bandwidth. WebSocket delivery is at-most-once and not durable.
Reconnecting clients synchronize authoritative state through HTTP.

## Outbound destinations

Application code may name any URL, but the worker opens the socket, so the worker refuses
destinations that are not on the public internet. This covers Rust HTTP requests, JavaScript
`fetch`, and outbound and singleton WebSockets. Refused ranges include loopback, private
networks (`10/8`, `172.16/12`, `192.168/16`, `fc00::/7`), link-local and cloud metadata
(`169.254/16`, `fe80::/10`), carrier-grade NAT (`100.64/10`), multicast, documentation,
benchmarking, and reserved ranges, and IPv6 forms that embed one of those IPv4 addresses.

The worker resolves the hostname, drops non-public answers, and connects to the exact address it
approved. TLS still verifies the original hostname. Every new connection and every singleton
reconnect resolves again, so DNS that later points at a private address is refused at that
point. A request refused this way fails with `destination-IP-prohibited` in Rust, a `403`
response from JavaScript `fetch`, and `WebSocketConnectError::DestinationForbidden` for
WebSockets.

Self-hosted operators who intentionally reach private services set
`FN0_ALLOW_PRIVATE_OUTBOUND_DESTINATIONS=true` on the worker. The variable accepts only `true`
or `false`; any other value stops the worker at startup. `forte dev` applies no destination
policy and no egress quota.

## Monthly quotas — one dollar plan

### Projects & domains

| Quota | Value | Notes |
| --- | --- | --- |
| Projects | 1 | |
| Custom domains | 1 | Automatic TLS — point a CNAME and you're done |

### Compute

| Quota | Value | Notes |
| --- | --- | --- |
| CPU pool | 500 CPU-minutes / month | ≈ 2M server-rendered pages at ~15 ms each |

The pool is monthly, not daily. A launch-day traffic spike draws on the whole
month's budget instead of hitting a daily wall.

### Network

| Quota | Value | Notes |
| --- | --- | --- |
| Compute egress | 20 GB / month | Bytes your application sends out of the platform; see below |
| Static asset downloads | Unlimited | Served through the CDN cache — never metered, never counted as egress |

Compute egress counts, per project and per UTC calendar month:

- HTTP response bodies the worker sends to clients.
- WebSocket message payloads the application sends, on any connection kind.
- Request bodies of the application's own outbound HTTP calls, from Rust and from JavaScript
  `fetch`.

It does not count headers, WebSocket control frames, received bytes, queue tasks, cron runs, or
callbacks the platform invokes internally. The fn0 control project is not metered.

A byte is charged before it is handed to the network. When the monthly quota is exhausted, new
HTTP requests receive `429 Monthly egress quota exhausted`, a response or request body that is
streaming stops at the next chunk, WebSocket sends fail with `EgressQuotaExceeded`, and the
connection closes with code `1008`. Bytes already written stay written. Workers take credit
from the control plane in chunks of up to 64 MiB and credit is charged when it is granted, so a
worker restart can count up to one unused chunk. When the control plane cannot be reached,
egress is refused rather than allowed.

The quota is stored per project in the control database as `ProjectEgressQuotaDoc`, with
`monthly_egress_limit` set to `{"Bytes": <limit>}` or `"Unlimited"`. New projects are created
with 20 GB. Operators change it by editing that document; a project without the document can
send nothing. A raised limit takes effect within a minute on workers that already saw the old
limit exhausted. Monthly usage is kept in `ProjectEgressUsageDoc` under `YYYY-MM`.

Static assets (your deployed build's files) are served from your own
Cloudflare account through your own CDN hostname, so their bandwidth is
between you and Cloudflare — R2 egress is free. Object storage downloads go
directly to the storage endpoint and are not cached.

### Document database

| Quota | Value | Notes |
| --- | --- | --- |
| Active databases | 1 | |
| Storage | 500 MB | |
| Row reads | 150M / month | |
| Row writes | 1M / month | A busy community site writes ~300k a month |

### Object storage

Object storage, public objects, deployed static assets and cached pages all
live in **your own Cloudflare account**, connected with `forte cloud login` and `forte cloud init`. Their storage and operation limits are whatever your Cloudflare
plan gives you — R2's forever-free tier is 10 GB, 1M writes and 10M reads a
month — and fn0 does not meter or cap them.

What fn0 still limits is how fast your app can mint presigned URLs and ask
for purges, because those are actions the runtime takes on your behalf:

| Quota | Value | Notes |
| --- | --- | --- |
| Presigned URLs minted | 1k / hour | Per project; a runaway loop stops here, not on your bill |
| Presigned URL expiry | 5 minutes maximum | Longer requested expiries are clamped, not rejected |
| Public object purges | 1k / hour | Explicit `public::purge` calls; the purge a `put` triggers on its own is not counted |

Pages cached by [lazy static page caching](../forte/pages.md#lazy-static-page-caching) live in a private bucket in your account.

Treat presigned URLs as opaque, short-lived strings: mint one right before
use, and never store one or parse its structure — the URL format may change.

Exceeding the mint ceiling refuses new presigned URLs (`429`) for the rest of
the hour; your deployed app keeps serving and already-minted URLs stay valid
until they expire.

### Metrics

Metrics your app records through the SDK are exported as OpenTelemetry series.
A *series* is one metric name plus one distinct combination of label values —
`http_requests{route="/posts",status="200"}` and
`http_requests{route="/posts",status="404"}` are two series.

| Limit | Value | Notes |
| --- | --- | --- |
| Active series | 1,000 per project | A series counts as active while it keeps reporting |
| Metric names | 100 per project | |
| Label values | 100 per label key | The usual cause of an explosion — see below |

A series that stops reporting for 5 minutes goes inactive and frees its slot.

Over the limit, existing series keep reporting and only **new** series are
dropped, so a running app never loses the series it already had. Drops are
reported back to you as `fn0.metrics.dropped`.

Nearly every cardinality explosion comes from putting an unbounded value in a
label: a user ID, a request ID, a raw URL path. Label with values you could
list in advance — route templates, status codes, regions — and 1,000 series is
a comfortable ceiling. If your app legitimately needs more, contact us.

### Queues & cron

Queue task execution and cron runs consume the shared monthly CPU pool —
there is no separate billing for them.

| Limit | Value | Notes |
| --- | --- | --- |
| Cron jobs | 10 per project | |
| Cron interval | 1 minute minimum | |
| Queue message size | 128 KB | |
| Queue backlog | 100k messages per project | |

## Monthly quotas — free plan

To be announced. The free plan targets trying things out: one project on a
custom domain from your own Cloudflare account, with quotas sized for
development traffic.

## When you outgrow these

- **Pay-as-you-go overage** (planned) — keep growing past the included quotas
  without hitting a wall.
- **Bring your own resources** (planned) — connect your own R2 bucket or
  Turso database; quotas for that resource become whatever your own account
  allows.
- **Self-host** — fn0 is open source. Run it on your own infrastructure with
  no limits at all.
