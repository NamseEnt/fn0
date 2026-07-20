# Limits & Quotas

Every limit that applies to fn0 Cloud, in one place. Self-hosted fn0 has none
of these — run it yourself and they are yours to change.

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

## Monthly quotas — one dollar plan

### Projects & domains

| Quota | Value | Notes |
| --- | --- | --- |
| Projects | 1 | |
| Custom domains | 1 | Automatic TLS — point a CNAME and you're done |
| fn0.dev subdomain | Included | |

### Compute

| Quota | Value | Notes |
| --- | --- | --- |
| CPU pool | 500 CPU-minutes / month | ≈ 2M server-rendered pages at ~15 ms each |

The pool is monthly, not daily. A launch-day traffic spike draws on the whole
month's budget instead of hitting a daily wall.

### Network

| Quota | Value | Notes |
| --- | --- | --- |
| Compute egress | 20 GB / month | Bytes leaving your handlers: SSR pages, API responses |
| Static asset downloads | Unlimited | Served through the CDN cache — never metered, never counted as egress |

Static assets (your deployed build's files) are the unlimited-download path.
Object storage downloads go directly to the storage endpoint, are not
cached, and carry the quotas below.

### Document database

| Quota | Value | Notes |
| --- | --- | --- |
| Active databases | 1 | |
| Storage | 500 MB | |
| Row reads | 150M / month | |
| Row writes | 1M / month | A busy community site writes ~300k a month |

### Object storage

| Quota | Value | Notes |
| --- | --- | --- |
| Storage | 10 GB | |
| Write operations | 100k / month, 2k / hour | Uploads, copies, lists — from your handlers or presigned PUT |
| Read operations | 100k / month, 5k / hour | Downloads and HEADs — from your handlers or presigned GET |
| Presigned URLs minted | 100k / month, 1k / hour | |
| Presigned URL expiry | 5 minutes maximum | Longer requested expiries are clamped, not rejected |

Treat presigned URLs as opaque, short-lived strings: mint one right before
use, and never store one or parse its structure — the URL format may change.

Going over quota blocks **presigned URL minting only** (requests get `429`);
your deployed app keeps serving and already-minted URLs stay valid until
they expire. The block lifts automatically once usage falls back under the
hourly and rolling 30-day limits — no action needed. If your app legitimately
needs high-volume presigned downloads, contact us: a paid add-on serving
them through the CDN cache is planned.

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
`*.fn0.dev` subdomain, with quotas sized for development traffic.

## When you outgrow these

- **Pay-as-you-go overage** (planned) — keep growing past the included quotas
  without hitting a wall.
- **Bring your own resources** (planned) — connect your own R2 bucket or
  Turso database; quotas for that resource become whatever your own account
  allows.
- **Self-host** — fn0 is open source. Run it on your own infrastructure with
  no limits at all.
