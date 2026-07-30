# Presigned URL Abuse Defense

Decision record, 2026-07-20. Superseded in part by BYO-Cloudflare (#79),
2026-07-30 — see "What BYO-Cloudflare changed" below before acting on
anything here.

Business decision (2026-07-20, now void): default presign quotas were
deliberately small (100k mints and 100k Class B reads per rolling 30 days,
per project) so that apps needing more would fund the Stage 2/3 upgrade as a
paid add-on.

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
  ```

  There is no size term. Storage is a stock, not a flow: however large and
  however many PUT URLs a project mints, its stored bytes are bounded by the
  R2 account they land in — since #79 that is the project owner's own
  account and their own bill.

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
2. **Sign `content-length`.** DONE (#53), but it is an *app-facing tool, not
   a platform control*. `presigned_put_url` takes `content_length:
   Option<u64>`; when set, the header joins `SignedHeaders` and R2 rejects a
   mismatched upload with 403 (verified: over, under, and chunked all 403;
   matching size 200). `None` stays unbounded.

   It is deliberately not enforced. The size is the app's own number spent
   against the app's own storage quota, so requiring it would defend nothing
   — a hostile project would just declare a large size. What it does buy is
   the app's ability to bound what *its* untrusted end users can upload, so
   a leaked URL cannot store more than the app intended.

   The bound is exact, not a maximum: SigV4 cannot express a size range, and
   R2 does not implement presigned POST (verified: 501 NotImplemented),
   which is the only S3 mechanism offering `content-length-range`.
3. **Per-project mint rate limit**: worker-local 1k/hour gate. Minting is a
   pure local signing operation (free), but the mint rate is the coefficient
   in the damage formula above. The control-side monthly cap that shipped
   with #11 was removed by #79 along with the metering that fed it.

Open verification (tracked in **#54**): are 429-rejected writes billed as
Class A? The pricing FAQ only exempts 401. If they bill at full rate, a
PUT-replay botnet costs $4.50/M despite the 1/s success cap, and PUT needs
the Stage 2 treatment below. Note that AWS classifies the same condition as
a 5xx (`503 SlowDown`) and does not bill it, while R2 returns a 4xx — so
the analogy points the wrong way. Measured: 429s are recorded as
`actionStatus: userError`, but analytics recording is not billing.

## GET: staged plan

Stage 1 is deliberate: attack cost is bounded and cheap at Class B prices,
and the availability of the S3 endpoint is Cloudflare's problem, not ours.

### Stage 1 — direct presigned GET (current)

- No availability cap; a valid-URL botnet costs us $0.36/M — about $130/hour
  at a sustained 100k req/s, so real damage ≈ detection latency.
- No sensor on our side since #79: S3-endpoint traffic is visible only in
  the account that owns the bucket, which is the user's. Their own R2
  analytics and billing alerts are the detection path.

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

## What BYO-Cloudflare changed (#79, 2026-07-30)

Every user project's objects now live in that user's own Cloudflare account.
The presigned URLs an app mints are signed with the user's own R2
credentials, against the user's own bucket, and every operation they drive
is billed to the user.

That removes the premise the whole quota loop rested on. Denial-of-wallet
against *us* is no longer possible through this surface, so the sensor and
the decision layer were deleted rather than retuned:

- Removed: `actions/usage_metering.rs`, `actions/presign_quota.rs`,
  `common/cloudflare_analytics.rs`, `PresignBlockedDoc`,
  `PresignMintCountDoc`, `ProjectQuotaOverridesDoc`, the worker's
  `presign_sync`, and the object-storage quota constants in `quota.rs`.
- Kept: the worker-local mint ceiling (`fn0::presign_gate`,
  `PRESIGN_MINT_PER_HOUR`) and the 5-minute expiry cap
  (`PRESIGN_MAX_EXPIRES_SECS`). Both are policy against a runaway app, not
  cost control, and both still hold with no control-plane round trip.

The Stage 2/3 plan below is unchanged in mechanism but no longer has a
funding argument attached to it: a user who wants cached, authenticated
downloads owns the zone it would run on. It stays here as a design record.

The damage formula still describes the exposure — it is just the user's
money on the right-hand side now, and the storage-quota term it referred to
no longer exists on our side at all.

## The other presigned-PUT surface: static asset deploys

`actions/deploy` mints presigned PUTs of its own, and there the size *is* a
platform control, because control picks the number rather than the app.

The deploy request declares `FileEntry { path, size }` per file; control
checks the declared total against `MAX_TOTAL_SIZE_PER_BUILD` (1 GB). Until
the fix that check was decorative — the URLs signed only `host`, so a client
could declare a kilobyte per file and upload a gigabyte to each. Control now
signs `content_length: Some(f.size)`, which makes the declaration binding
and the existing quota real.

The bundle tar gets the same treatment (#55): the deploy request carries a
required `bundle_size`, and control signs `content_length:
Some(bundle_size)` into the bundle's presigned PUT. Required, not defaulted
— an older CLI that omits it is rejected at deserialization (400) rather
than minting an unbounded URL.

The 1 GB total remains the only size ceiling (#56). A per-file ceiling was
considered and dropped: static storage costs $0.015/GB-mo against free
CDN-cached egress, so the cap is insurance, not a cost lever
(`dollar-plan.md`), and the now-binding 1 GB total already bounds the stock.
An asset that large belongs in object storage, which since #79 is billed to
the project owner's own Cloudflare account.
