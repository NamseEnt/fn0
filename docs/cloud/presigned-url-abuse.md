# Presigned URL Abuse Defense

Decision record, 2026-07-20. fn0 cloud currently runs **GET Stage 1** below,
with the #11 quota enforcement implemented on top of it (see "Interaction
with quota enforcement" at the bottom).

Business decision (2026-07-20): default presign quotas are deliberately
small (100k mints and 100k Class B reads per rolling 30 days, per project —
see `dollar-plan.md`). Apps that legitimately need more are asked to contact
us; that demand funds the Stage 2/3 upgrade as a **paid add-on**, and the
zone-level fixed cost ($5 Workers Paid, later $20–25 Pro) is shared across
every add-on customer.

Tenant apps mint presigned URLs through the object-storage hijack and hand
them to untrusted end users, so the URLs are an open attack surface for the
free / $1 tiers: denial-of-wallet (driving R2 or Workers billing) and DoS.

## Why the data plane is left undefended

Presigned URLs work only on the S3 API endpoint
(`<account_id>.r2.cloudflarestorage.com`) and cannot be attached to custom
domains (custom domains are read-only public access). That endpoint is
outside the fn0.dev zone, so zone WAF, rate-limiting rules, Workers routes,
and cache can never see this traffic — and that is acceptable, because:

- Requests without a valid signature are rejected by R2 and **unbilled**
  (HTTP 401), and volumetric floods land on Cloudflare-operated
  infrastructure, not ours.
- Every *valid* URL was minted by our runtime. The mint gate in
  `ObjectStorageHijack` is therefore the single control point, and worst-case
  damage is a design parameter, not an unknown:

  ```text
  max damage ≈ mint-rate cap × expiry cap × op unit cost
               (+ size cap × storage price for PUT)
  ```

## Verified facts (Cloudflare docs, checked 2026-07-20)

| Fact | Source |
|---|---|
| Presigned URLs: S3 endpoint only, no custom domains | [R2 presigned URLs](https://developers.cloudflare.com/r2/api/s3/presigned-urls/) |
| Custom domains are read-only (no PUT) | [R2 public buckets](https://developers.cloudflare.com/r2/buckets/public-buckets/) |
| Unauthorized requests (401) are not billed | [R2 pricing FAQ](https://developers.cloudflare.com/r2/pricing/) |
| Writes to one object key: 1/s, excess → 429 | [R2 limits](https://developers.cloudflare.com/r2/platform/limits/) |
| Reads: no rate limit; Class B $0.36/M; egress free | [R2 pricing](https://developers.cloudflare.com/r2/pricing/) |
| Free-zone rate limiting: 1 rule, IP-only, 10 s window | [WAF rate limiting](https://developers.cloudflare.com/waf/rate-limiting-rules/) |
| `is_timed_hmac_valid_v0()` requires Pro+ | [Rules functions](https://developers.cloudflare.com/ruleset-engine/rules-language/functions/) |
| Workers Free: 100k req/day **per account**, then fail open (Worker bypassed) or fail closed (error 1027) | [Workers limits](https://developers.cloudflare.com/workers/platform/limits/) |
| Workers Paid: $5/mo, 10M requests included, +$0.30/M | [Workers pricing](https://developers.cloudflare.com/workers/platform/pricing/) |
| Custom cache keys (Pro+) can ignore query strings, so differently-tokenized URLs for one object share a cache entry — required for Stage 3 cache hits | [Cache keys](https://developers.cloudflare.com/cache/how-to/cache-keys/) |

## PUT: direct presigned, hardened mint gate

PUT stays on the S3 endpoint permanently. Replay of a single PUT URL is
already capped by R2 itself (1 write/s per key ⇒ a 10-minute URL yields at
most ~600 billable Class A ops ≈ $0.003). Required hardening in the hijack
(`fn0/fn0/src/object_storage_hijack.rs`):

1. **Cap expiry per plan.** DONE (#11): clamped to 5 minutes
   (`PRESIGN_MAX_EXPIRES_SECS`). Expiry is the only revocation mechanism —
   once minting stops, exposure ends when outstanding URLs expire.
2. **Sign `content-length`.** The SDK presign API takes the intended size and
   the header joins `SignedHeaders`, bounding the upload size per URL.
   Today only `host` is signed, so one leaked URL can upload objects of any
   size. Still open — blocked on the curl verifications below.
3. **Per-project mint rate limit**, with mint counts recorded in usage
   metering. DONE (#11): worker-local 1k/hour gate + `PresignMintCountDoc`
   reporting + 100k rolling-30-day cap enforced control-side. Minting is a
   pure local signing operation (free), but the mint rate is the coefficient
   in the damage formula above.

Open verifications before relying on this (test with curl against a real
bucket):

- Are 429-rejected writes billed as Class A? The pricing FAQ only exempts
  401. If they bill at full rate, a PUT-replay botnet costs $4.50/M despite
  the 1/s success cap, and PUT needs the Stage 2 treatment below.
- Does R2 actually enforce a signed `content-length` mismatch with a 403?

## GET: staged plan

Stage 1 is deliberate: attack cost is bounded and cheap at Class B prices,
and the availability of the S3 endpoint is Cloudflare's problem, not ours.

### Stage 1 — direct presigned GET (current)

- No availability cap; a valid-URL botnet costs us $0.36/M — about $130/hour
  at a sustained 100k req/s, so real damage ≈ detection latency.
- Sensor: hourly per-bucket operation counts in usage metering
  (`actions/usage_metering.rs`) — the only sensor that sees S3-endpoint
  traffic. Response: set the project's mint-gate flag (stop issuing URLs)
  and let outstanding URLs age out.

### Stage 2 — custom hostname + Worker (Workers Paid, $5/mo)

Trigger: observed GET abuse, or product need for cached/authenticated
downloads.

- DNS record + Worker route + R2 binding (no R2 custom domain needed).
  Worker validates an HMAC token, checks the per-project blocklist, serves
  via `caches.default`, falls back to the bucket on miss. Botnet replay of
  one URL becomes cache hits; R2 sees ~1 Class B op per URL.
- Inline per-project counting in the Worker replaces polling lag with
  immediate detection and blocking.
- **The token wire format must be `is_timed_hmac_valid_v0()`-compatible from
  day one** — `/<key>?verify=<10-digit unix seconds>-<base64 MAC>` — so that
  Stage 3 is a configuration swap (add WAF rule, remove Worker route) with
  all outstanding URLs still valid.
- Workers Free is rejected, not deferred: the 100k req/day account-wide cap
  makes the Worker either a $0-cost global kill switch for every tenant
  (fail closed, and it takes fn0's other Workers down with it) or an
  authentication bypass (fail open).

### Stage 3 — Pro plan + WAF HMAC rule

Trigger: Worker request volume where overage exceeds the Pro delta —
roughly **60–77M requests/month** (Pro $20–25 vs Workers $5, difference
÷ $0.30/M above the included 10M).

- One WAF custom rule with `is_timed_hmac_valid_v0()` on a wildcard host
  match blocks invalid/expired tokens at the edge, before Worker billing.
  Valid-URL replays are served by cache. The Worker route is removed (or
  kept solely as the metering path).

## Interaction with quota enforcement (#11) — implemented

The sensor→actuator loop shipped with #11:

- **Sensor**: #48's `usage_metering` (hourly per-bucket/per-prefix ops from
  GraphQL Analytics) plus worker-reported `PresignMintCountDoc` rows.
- **Decision**: `presign_quota::run_enforcement` in fn0-control, hourly after
  metering. Compares object-storage Class A/B and mint counts against
  `quota.rs` defaults (or `ProjectQuotaOverridesDoc`), both per latest hourly
  window and per rolling 30 days, and publishes breaches in
  `PresignBlockedDoc`.
- **Actuator**: workers poll `PresignBlockedDoc` (1 s) into the fn0
  `PresignGate`; the object-storage hijack refuses minting (`429`) for
  blocked projects. Already-minted URLs age out within 5 minutes. Release is
  automatic: the next hourly evaluation unlists a project once usage is back
  under the caps.
- There is still no per-project Cloudflare-side lever — #31 consolidated
  static assets onto one shared custom domain, so the per-project
  `R2CustomDomain.enabled` toggle from the original #11 text no longer
  exists. Deploys are not blocked by this quota; the mint gate is the single
  lever.
- The public "unlimited downloads" claim is now scoped to static assets;
  object-storage reads carry explicit quotas (`docs/fn0/limits.md`). The
  cached-download story returns as the paid Stage 2/3 add-on.
