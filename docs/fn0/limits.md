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
| Object storage downloads | Unlimited | Served through the CDN cache — never metered, never counted as egress |

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
| Uploads | 100k / month | |
| Downloads | Unlimited | Served through the CDN cache |

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
