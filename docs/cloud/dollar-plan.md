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
| Compute (OCI A1) | $0.0138/OCPU-hr + $0.0027/GB-hr | OCI list |
| Compute (OCI A1 free tier) | 2 OCPU + 12GB always free | bootstrap capacity |
| Egress (OCI) | 10TB/month free, then $0.0085/GB | OCI list |
| DB (Turso Scaler, $24.92/mo) | 2,500 active DBs, 24GB, 100B row reads, 100M row writes included | turso.tech/pricing |
| DB overage | $0.80/B row reads, $0.80/M row writes, $0.50/GB-mo storage | turso.tech/pricing |
| Object storage (R2) | $0.015/GB-mo; Class A $4.50/M; Class B $0.36/M; egress free | CF R2 pricing |
| Custom hostname (CF for SaaS) | $0.10/mo, first 100 free | CF for SaaS plans |
| Block volume (OCI, Lower Cost 0 VPU) | $0.025/GB-mo, 2 IOPS/GB | OCI list |
| Metrics (self-hosted VictoriaMetrics on A1) | ~$0.0000073/series-mo — see below | derived |
| Metrics (Grafana Cloud, rejected) | 10k active series included, then $6.50/1k series-mo **plus $19/mo Pro platform fee** | grafana.com/pricing |

R2 chosen over OCI Object Storage ($0.0255/GB-mo) for price and free egress.

VM.Standard.A4 is not offered in `ap-osaka-1` or `ap-tokyo-1` (checked against the
live tenancy, 2026-07-21); A1.Flex is, up to 80 OCPU / 512 GB. Vertical headroom
on A1 alone covers every scale this plan models.

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
| Static asset downloads | unlimited (served via CDN cache; R2 egress is free, origin ops only on cache miss) | cache-miss bounded | ~$0.01 |
| Object GET (Class B) | 100k (+5k/hour burst cap) | $0.036 | ~$0 |
| Presigned URLs minted | 100k (+1k/hour burst cap), 5 min max expiry | $0 (local signing) | $0 |
| Active DBs | 1 | $0.01 | $0.01 |
| DB storage | 500 MB | $0.25 | ~$0.005 |
| DB row reads | 150M | $0.12 | ~$0 |
| DB row writes | 1M | $0.80 | ~$0.01 |
| Object storage | 10 GB | $0.15 | ~$0.01 |
| Object PUT (Class A) | 100k (+2k/hour burst cap) | $0.45 | ~$0 |
| Metric active series — platform-emitted | ~200 (RED per route, 40-route cap) | $0.0034 | $0.0034 |
| Metric active series — user custom | 300 (100 names, 100 label values/key) | $0.0050 | ~$0.0002 |

Totals: worst ≈ $2.2/user, expected ≈ $0.06–0.16/user against a $0.35–0.80
budget. Holds as long as cap-maxing users stay a small minority.

The two metric lines are costed at the HA (2-replica) rate derived in "Metrics
sizing" below. Platform-emitted series are the one line where worst = expected:
every deployed project with traffic emits them continuously, so the overbooking
argument above does not apply to it.

Rationale for the generous caps: they meet or beat the Cloudflare free tier
on every line users actually compare (see Positioning), and the lines where
they don't (PUT count, unmetered egress) are not comparison points for the
target audience.

### Egress semantics

"Egress" in the plan means **compute egress**: bytes leaving the runtime
(SSR responses, API responses, subrequest downloads billed to the requester).
**Static asset** downloads are not metered — they are served through the CDN
cache and R2 charges no egress. Public copy must scope the unlimited promise
to static assets, or the 20 GB number reads as a gallery-killer.

**Object storage** downloads are metered: presigned URLs work only on the S3
API endpoint, which bypasses the CDN cache entirely, so every GET is a
billed Class B op ($0.36/M). Hence the Class B / mint quotas above. The
hourly caps are early abuse cut-off, not cost control; only the monthly
(rolling 30-day) caps bound cost. Enforcement is presign refusal (worker
mint gate driven by hourly metering), auto-released when usage drops back
under the caps — see `presigned-url-abuse.md` for the full defense ladder.

Sizing rationale: reaching 100k presigned downloads/month legitimately means
a private-content UGC app with thousands of DAU — beyond this plan's
side-project target. Such apps are the paid add-on's market ("contact us",
served cached via the Stage 2/3 path in `presigned-url-abuse.md`), not a
quota-sizing problem.

### Metrics sizing

Sized bottom-up from what a service actually needs to be operable, then priced.
A 20-route SSR app has to answer four questions — where traffic lands, what is
failing, what is slow, and how the app's own business metrics are moving:

| Need | Instrument | Label combos | Series each | Series |
|---|---|---|---|---|
| Per-route latency p50/p95/p99 | OTel exponential histogram, `max_size=8` | 20 routes | ~8 | 160 |
| Per-route request volume | `histogram_count()` of the above | — | — | 0 |
| Per-route errors | counter `{route, status_class}` | ~30 | 1 | 30 |
| CPU saturation | exponential histogram `{project}` | 1 | 8 | 8 |
| Platform throttling | counter `{project}` | 1 | 1 | 1 |
| **Platform-emitted subtotal** | | | | **~200** |
| Business metrics | user's own instruments | ~100 needed | | **300 allowed** |
| **Total per project** | | | | **500** |

Series-per-histogram is measured, not assumed: at `max_size=8` the OTel SDK
raises its scale to fit the data and lands on 5–7 populated buckets across
latency distributions from tight (σ=0.15) to wide (σ=0.9), plus `_sum` and
`_count`. VictoriaMetrics stores each populated bucket as one `vmrange` series,
so there is no bucket discount to claim — unlike Grafana Cloud, which bills
native-histogram buckets at 0.25x.

Exponential over explicit buckets because explicit boundaries have to be picked
in advance and degrade without warning when a project's latency sits outside
them. Measured against a known log-normal distribution, the current fixed
boundaries put p99 at 2.2x the true value (everything between 0.1s and 0.5s is
one bucket) while the exponential histogram held ~3%; the exponential form also
bounds relative error by construction at whatever scale it settles on.

The `route` label is safe to carry because `classify_route` emits build-time
route *patterns* (`/docs/[section]/[page]`), not raw paths, with unmatched paths
collapsing to `unknown`. The 40-route cap is what bounds the platform-emitted
line; it has to move together with the per-route panel count.

**Cost, self-hosted on A1.** VictoriaMetrics needs ~1 GB RAM per 1M active
series and should stay under 50% of system memory, so RAM is the binding
constraint. Capacity and cost scale linearly with shape, and the per-project
figure is therefore flat:

| A1 shape | VM budget | Active series | Projects @ 500 | Node/mo |
|---|---|---|---|---|
| 1 OCPU / 6 GB | 3 GB | 3M | 6,000 | $21.90 |
| 4 OCPU / 24 GB | 12 GB | 12M | 24,000 | $87.58 |
| 80 OCPU / 512 GB | 256 GB | 256M | 512,000 | $1,815 |

Per project: **$0.0036 compute + $0.0005 disk = ~$0.004/month**, doubled to
**~$0.008** for the two-replica HA pair. Disk is 30-day retention at under a
byte per sample on a Lower Cost (0 VPU) volume — VM is documented to run on 35
IOPS, so the 2 IOPS/GB tier is roughly an order of magnitude more than it needs.

That is **1.0–2.4% of the $0.35–0.80 infra budget**. The same 500 series on
Grafana Cloud list price would be $3.25/project/month — ~390x more, and over
three times the plan's entire revenue. Self-hosting is what makes a per-project metrics
dashboard fit inside a $1 plan at all; see `#57` for the viewing side.

What the per-project cap protects is no longer a purchased allowance but the
**shared node**: one project putting an unbounded value in a label (user ID,
request ID, raw path) can exhaust the RAM every other project depends on,
including the platform's own `fn0-control` telemetry. This is the aggregate-pool
case from "Cost model" above — oversubscribed on purpose, protected per-project
so one tenant cannot take the whole node, monitored in aggregate rather than
divided up front.

Enforcement is inline in the worker's OTLP hijack (`fn0/fn0/src/metric_gate.rs`),
the same trust boundary that stamps `fn0.project_id`. Semantics are keep
existing / drop new, so a project at its cap keeps every series it already had
and loses only newly appearing ones; the drop count rides back on the
project's own payload as `fn0.metrics.dropped` so the owner sees the
throttling. Series idle for 5 minutes free their slot, matching Alloy's
`deltatocumulative.max_stale`, so the count tracks active series the way the
backend holds them.

The gate governs the **user custom** line only. Platform-emitted series are
recorded on the worker's own meter, never pass through the OTLP hijack, and are
bounded by the 40-route cap instead.

Caps are per worker node (each node sees only the slice of a project it
serves), which is loose by design: the true ceiling is the metrics node's RAM,
and the per-project cap only has to stop one tenant from monopolising it.
Operators watch `fn0.metric.active_series` against the node budget; raising the
caps is a constant change in `metric_gate.rs`, deliberately cheap because the
cost of being wrong on the low side (a paying user's legitimate instrumentation
silently truncated) is worse than being wrong on the high side.

The custom cap is 300, down from an earlier 1,000 that was set against a
mis-stated price (see below). 300 is 3x the ~100 series a well-instrumented app
actually emits, and the 100-values-per-key cap still catches unbounded labels
early. Starting values, to be tuned against real usage once user-facing metrics
ship — the same "design targets, not commitments" stance as the rest of this
document.

**Correction, 2026-07-21.** An earlier revision of this section priced 1,000
series at $0.0065/month. At $6.50/1k series-mo the correct figure is $6.50 —
the per-series price was written as the total, a 1000x error. Under that
mistake the metrics line looked like the cheapest quota in the plan; corrected,
it was the single largest, at 6.5x the plan's own revenue, and it also omitted
Grafana Cloud's $19/month Pro platform fee that the first series over the free
10k triggers. Self-hosting was chosen off the corrected numbers.

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

- Monitoring and logs (the metrics pipeline and its cardinality caps are
  implemented; what remains is per-project dashboards and query scoping)
- Pay-as-you-go overage beyond the included quotas
- A higher tier (~$5) so cap-hitting users have somewhere to go
- High-volume presigned downloads add-on: served cached through our zone
  (Stage 2 Worker at $5/mo Workers Paid, then Stage 3 Pro WAF HMAC — see
  `presigned-url-abuse.md`). Priced per customer; the zone-level cost is
  shared across every add-on customer.
- Bring-your-own Turso database: quotas for that resource become the user's
  own. The R2 half of this line shipped as #79 and is no longer planned —
  see below.

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
  Decided for the object-storage/presign lines (2026-07-20): 1/10 of the $1
  plan caps, worst-case ~$0.12/free project. Applies once plan tiers exist;
  until then every project runs on the $1-plan defaults in `quota.rs`.
  The metrics caps follow the same 1/10 shape when tiers land, as product
  tiering rather than cost recovery — both tiers cost effectively nothing.
- Queue/cron abuse ceilings vs implementation reality.
- Whether enqueue operations map to DB writes in billing.
