# fn0 Cloud $1 Plan — Economics & Quota Design

Working document for the paid plan. Numbers here are design targets, not
published commitments; the public quota table lives on fn0.dev once finalized.

Status: draft. Blocked on Paddle bespoke pricing response (asked 2026-07-16).

## Revenue per subscriber

| Scenario | Fee | Net/month | Infra budget (after $0.10 domain) |
|---|---|---|---|
| Paddle standard (5% + $0.50) | $0.55 | $0.45 | $0.35 |
| Paddle bespoke for sub-$10 (reported ~10% flat) | $0.10 | $0.90 | $0.80 |
| Standard rate, $12/year billing | $1.10/yr | $0.908 | $0.81 |

- Custom domain: Cloudflare for SaaS custom hostname, $0.10/month each,
  first 100 hostnames free account-wide.
- Chargebacks: deliberately ignored in this model (accepted risk).
- Annual billing is only a fee optimization under the standard rate; if the
  bespoke flat rate lands, monthly billing stands on its own.

## Cost model: caps are insurance, not cost

Quotas must not be derived by dividing the infra budget at list price.
The model is overbooking, same as every PaaS free tier:

- **Expected cost (p50 user)** must stay under the infra budget. Side
  projects typically use 1–5% of their caps.
- **Worst-case cost (user maxing every cap)** may exceed the budget; it is
  bounded by the caps and such users are rare. The $1 card-on-file
  requirement is the abuse filter.

Aggregate pools (Turso plan allowances, OCI 10TB egress free tier) are
oversubscribed by design. Monitor aggregate usage and alert; do not size
per-user caps by dividing pools.

## Unit costs

| Resource | Price | Source |
|---|---|---|
| Compute (OCI A4) | $0.0138/OCPU-hr + $0.0027/GB-hr | OCI list |
| Compute (OCI A1 free tier) | 4 OCPU + 24GB always free | bootstrap capacity |
| Egress (OCI) | 10TB/month free, then $0.0085/GB | OCI list |
| DB (Turso Scaler, $24.92/mo) | 2,500 active DBs, 24GB, 100B row reads, 100M row writes included | turso.tech/pricing |
| DB overage | $0.80/B row reads, $0.80/M row writes, $0.50/GB-mo storage | turso.tech/pricing |
| Object storage (R2) | $0.015/GB-mo; Class A $4.50/M; Class B $0.36/M; egress free | CF R2 pricing |
| Custom hostname (CF for SaaS) | $0.10/mo, first 100 free | CF for SaaS plans |

R2 chosen over OCI Object Storage ($0.0255/GB-mo) for price and free egress.

## Proposed quotas

Per request (runtime limits, all plans):

| Limit | Value |
|---|---|
| CPU time | 50 ms |
| Memory | 128 MB |
| Max duration | 15 s |
| Request headers / body | 128 KB / 100 MB |
| Response headers / body | 128 KB / unlimited |
| Subrequests | 50 |

Note: the fn0.dev homepage currently says 10 ms CPU; the plan design assumes
50 ms (real SSR frequently exceeds 10 ms — this is a structural advantage
over Cloudflare's free tier). The homepage table must be updated when the
plan page ships.

Per month, $1 plan (worst-case cost = cap × list price; expected = p50
side-project usage):

| Quota | Value | Worst | Expected |
|---|---|---|---|
| Projects | 1 | — | — |
| Custom domains | 1 | $0.10 | $0.10 |
| CPU pool | 500 CPU-minutes (≈ 2M typical SSR requests @ ~15 ms) | $0.115 | ~$0.005 |
| Compute egress | 20 GB | $0.17 | ~$0 |
| Object storage downloads | unlimited (served via CDN cache; R2 egress is free, origin ops only on cache miss) | cache-miss bounded | ~$0.01 |
| Active DBs | 1 | $0.01 | $0.01 |
| DB storage | 500 MB | $0.25 | ~$0.005 |
| DB row reads | 150M | $0.12 | ~$0 |
| DB row writes | 1M | $0.80 | ~$0.01 |
| Object storage | 10 GB | $0.15 | ~$0.01 |
| Object PUT (Class A) | 100k | $0.45 | ~$0 |

Totals: worst ≈ $2.2/user, expected ≈ $0.05–0.15/user against a $0.35–0.80
budget. Holds as long as cap-maxing users stay a small minority.

Rationale for the generous caps: they meet or beat the Cloudflare free tier
on every line users actually compare (see Positioning), and the lines where
they don't (PUT count, unmetered egress) are not comparison points for the
target audience.

### Egress semantics

"Egress" in the plan means **compute egress**: bytes leaving the runtime
(SSR responses, API responses, subrequest downloads billed to the requester).
Downloads from object storage are **not metered** — they are served through
the CDN cache and R2 charges no egress. Public copy must say this explicitly
("object storage downloads are unlimited") or the 20 GB number reads as a
gallery-killer.

Abuse guard for unmetered downloads: cache aggressively, normalize cache
keys (ignore query strings on asset routes), rate-limit origin fetches.

### Queue and cron

Queues and cron jobs are already part of the runtime (queue hijack,
cross-project enqueue, Forte queue tasks / cron jobs). They do not need
money-backed quotas of their own: **queue task execution and cron
invocations consume the same monthly CPU pool as HTTP requests.**

They still need abuse ceilings (strawman, to be checked against
implementation reality):

| Limit | Strawman value |
|---|---|
| Cron jobs per project | 10 |
| Min cron interval | 1 minute |
| Queue message size | 128 KB |
| Queue backlog per project | 100k messages |
| Enqueues | count as DB row writes (implementation-dependent — verify) |

## Included vs planned

Included in the $1 plan at launch: SSR compute, document DB, object storage,
queues, cron, custom domain with automatic TLS.

Planned (public copy uses the existing "planned" badge tone):

- Monitoring and logs
- Pay-as-you-go overage beyond the included quotas
- A higher tier (~$5) so cap-hitting users have somewhere to go
- Bring-your-own R2 bucket / Turso database: quotas for that resource become
  the user's own. Doubles as the open-source/no-lock-in story and moves the
  heaviest users out of our cost structure.

## Positioning vs Cloudflare

Do not fight CF free on raw totals (it wins on paper). The winning axes:

1. **Monthly pools vs daily resets.** A launch-day spike (Show HN) hits CF
   free's 100k req/day and D1's 100k writes/day hard caps; fn0's monthly
   pools absorb it.
2. **50 ms vs 10 ms CPU per request.** Real SSR on CF free is marginal;
   the CF answer is the $5 Paid plan, at which point fn0 is 1/5 the price
   with 2x the compute per dollar (500 CPU-min/$1 vs 500 CPU-min/$5).
3. **Custom domain via CNAME.** CF Workers custom domains require moving the
   whole zone to Cloudflare nameservers; fn0 needs one CNAME record.
4. **Zero assembly, local = prod.** No wrangler bindings/migrations; one
   deploy command. For the AI-coding audience the real CF price is the
   learning curve, not $5.
5. **Escape hatches.** PAYG overage, BYO resources, and self-hosting the
   open-source runtime. Disarms the lock-in objection entirely.

## Later: DB backend

If Turso billing becomes the dominant cost at scale, compare against
self-hosting: ScyllaDB is expected to be far more efficient for our purely
document-shaped workload; self-hosted libSQL keeps wire compatibility with
zero semantic change. Decision deferred until real usage data exists.

## Open decisions

- Paddle bespoke rate (email sent 2026-07-16) → sets the budget at $0.35 or ~$0.80.
- Annual-only vs monthly billing (only matters under the standard rate).
- Free plan quotas (must be small enough that $1 is an upgrade; free tier
  without a card needs a much tighter CPU cap against wasm cryptomining).
- Queue/cron abuse ceilings vs implementation reality.
- Whether enqueue operations map to DB writes in billing.
