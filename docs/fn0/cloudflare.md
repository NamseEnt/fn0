# Bring your own Cloudflare account

Your project's object storage, public objects, deployed frontend assets,
cached pages and custom domain all run on **your** Cloudflare account, not
fn0's. You keep R2's free tier, your own purge budget and your own bill; fn0
runs the compute and holds no storage on your behalf.

This is a one-time setup per project, and it is required before a project can
serve a frontend: fn0 Cloud has no way to meter object storage usage on an
account it does not own, so bring-your-own-Cloudflare is the default premise
for every project, not an opt-in. There is no `fn0.dev` fallback to fall back
to — a project without a custom domain has no public URL.

## What you need

A Cloudflare account with **a zone in it** — a domain whose nameservers point
at Cloudflare. Not just an account: an R2 custom domain has to live in a zone
in the same account as the bucket, and that hostname is where your frontend
assets get served from.

## Two ways to set this up

Setup has to create buckets, point a hostname at one, write two zone rules and
sign a certificate. Someone has to hold a Cloudflare token that can do those
things. Pick who.

| | Convenient | Careful |
| --- | --- | --- |
| Tokens you create by hand | 1 | 3 |
| Most powerful token you create | account-wide | can provision, cannot create tokens |
| fn0 ever sees it | no | no |
| Commands | 1 | 2 |

Both end in the same place: fn0 holds a bucket-scoped R2 credential and a
purge-only token, and nothing else. The difference is only what you have to
create along the way, and whether any of it could escalate if it leaked from
your own machine.

## Convenient: one token, the CLI does the rest

Cloudflare dashboard → **My Profile → API Tokens → Create Token → Create
Custom Token**. Give it exactly one permission:

| Scope | Permission |
| --- | --- |
| User | API Tokens → Edit |

```sh
forte cloudflare connect \
  --account-id <cloudflare account id> \
  --zone-id <zone id> \
  --api-token <the token you just made>
```

The CLI mints a short-lived provisioning token from it, provisions your
account, mints the two credentials fn0 keeps, and revokes the provisioning
token before it exits. If it dies mid-run, that token expires by itself within
ten minutes.

**Delete the setup token afterwards.** One checkbox does not make it a small
permission: a token that can create tokens can create *any* token in your
account, so until you delete it, it is a full-account credential. That is the
whole reason to consider the other path.

## Careful: you create every token yourself

Nothing you make here can create tokens, so nothing you make here can widen
itself.

**Step 1** — a provisioning token. My Profile → API Tokens → Create Custom
Token:

| Scope | Permission |
| --- | --- |
| Account | Workers R2 Storage → Edit |
| Zone | Zone → Read |
| Zone | Cache Rules → Edit |
| Zone | Transform Rules → Edit |
| Zone | SSL and Certificates → Edit |

Restrict the zone scopes to the one zone. Then:

```sh
forte cloudflare provision \
  --account-id <account id> --zone-id <zone id> --api-token <token>
```

This creates the buckets, the CDN hostname and the two zone rules, then stops
and prints the exact names for step 2.

**Step 2** — the three credentials fn0 keeps.

The two R2 tokens are separate on purpose. Only the worker one is handed to the
fleet, and only the frontend-asset one is used by the GC that deletes; scoping
them apart is what keeps either from reaching the other's bucket. Both are made
at R2 → **Manage API Tokens → Create API token**, permission *Object Read &
Write*, applied to the buckets the previous command named — the first to the
two object-storage buckets and the rendered-HTML cache, the second to the
frontend-asset bucket alone. Each screen shows an Access Key ID and a Secret
Access Key; keep all four values.

My Profile → **API Tokens → Create Custom Token**, permission *Zone → Cache
Purge → Purge*, restricted to your zone.

```sh
forte cloudflare connect \
  --account-id <account id> --zone-id <zone id> --zone-name <your-domain> \
  --worker-access-key-id <Access Key ID> \
  --worker-secret <Secret Access Key> \
  --frontend-asset-access-key-id <Access Key ID> \
  --frontend-asset-secret <Secret Access Key> \
  --purge-token <purge token>
```

Delete the provisioning token from step 1 once this succeeds.

## What the two stored credentials can do

| Credential | What it can do |
| --- | --- |
| Worker R2 | read and write objects in this project's two object-storage buckets and its rendered-HTML cache. Cannot reach the frontend-asset bucket, cannot reach another project's buckets, cannot delete a bucket, cannot call the Cloudflare API at all |
| Frontend-asset R2 | read and write objects in this project's frontend-asset bucket, and nothing else. Never sent to a worker |
| Purge | purge this one zone's cache. Nothing else — not DNS, not cache rules, not R2, not certificates |

Those limits are measured against the live API, not inferred from the
permission names.

So the worst a total compromise of fn0 can do to your Cloudflare account is
rewrite objects in the four buckets it created for this project, and clear your
cache.

## What gets created

Four buckets, all this project's alone. Nothing is shared with your other fn0
projects, so no key prefix is carrying the separation.

| Bucket | Holds | Reachable at |
| --- | --- | --- |
| `fn0-<project-id>-private-object-storage` | what `object_storage::private` writes | nowhere — signed requests only |
| `fn0-<project-id>-public-object-storage` | what `object_storage::public` writes | `fn0-<project-id>-public-object-storage.<your-domain>` |
| `fn0-<project-id>-frontend-asset` | your deployed frontend build | `fn0-<project-id>-frontend-asset.<your-domain>` |
| `fn0-<project-id>-rendered-html-cache` | HTML rendered on the server and kept for the next request | nowhere — private |

The two public buckets answer on a hostname that is the bucket's own name in
your zone, so a bucket and the address it serves from cannot drift apart. Each
costs one DNS record.

fn0 adds two rules to your zone and leaves your own rules in place. Both match
`fn0-*-frontend-asset.<your-domain>` and
`fn0-*-public-object-storage.<your-domain>`, so they cover every fn0 project you
ever add without a rule per project — a free zone allows ten of each, and a rule
per project would run out at ten projects. Both halves of each pattern are
required so a rule cannot swallow a hostname of your own.

The **cache rule** pins browser caching on those hostnames to whatever fn0
stored on the object, because a zone's default Browser Cache TTL would otherwise
leave browser copies of a replaced object that no purge can reach. Your other
hostnames keep the zone setting.

The **response header rule** removes `Vary: Origin`, which R2 attaches to every
CORS response. The buckets allow `*`, so that header says nothing — but it makes
Cloudflare keep one cache entry per requesting origin, and a purge by URL clears
only the entry for a request that sent no `Origin`. Browsers send one, so
without this rule a replaced object keeps serving its old bytes to browsers for
the full year fn0 stores on public objects.

Workers pick the change up within about a second; no redeploy is needed. Check
with:

```sh
forte cloudflare status
```

**Connect before your project stores anything.** A project has nowhere to store
and cannot serve a frontend until it is connected.

Connecting is first-time only, and there is no way back. Reconnecting, rotating
a credential and moving to a different Cloudflare account are all unsupported —
not merely undocumented, but refused by `connect`. If a stored credential is
lost or revoked, the project cannot be repaired through the CLI. Treat the
three credentials as things you do not lose.

## Custom domain (optional)

Signing an origin certificate needs a permission fn0 deliberately does not
hold, so this runs locally too.

With a provisioning token (the careful path's step 1 token, which already has
SSL and Certificates → Edit):

```sh
forte domain add app.example.com \
  --account-id <account id> --zone-id <zone id> --api-token <token>
```

With the one-permission setup token, add `--mint-signing-token` so the CLI
mints a signing token, uses it and revokes it:

```sh
forte domain add app.example.com \
  --account-id <account id> --zone-id <zone id> --api-token <token> \
  --mint-signing-token
```

The CLI generates a key pair, has Cloudflare sign the certificate through your
own Origin CA, uploads the certificate and key, and prints the one thing left
to do: add a **proxied** `CNAME` record for that hostname pointing at the
printed origin hostname.

The record must stay orange-clouded. An Origin CA certificate is not valid for
a direct visitor connection, so switching the record to DNS-only breaks the
hostname immediately and visibly.

## What fn0 still holds

- The compute and the request routing
- Your document database (Turso), which is not part of this
- The bundle store your deployed code is distributed from. It holds compiled
  WebAssembly rather than anything your app stores, and it grows with deploys
  rather than with traffic, so it stays on fn0's account

## Removing a project

`forte destroy` empties fn0's buckets in your account but does not delete
them — fn0 holds an object-scoped credential there by design. The buckets are
yours to remove.

## Revoking

Deleting either R2 token in your Cloudflare dashboard breaks the project at
request time — there is no grace period, because every request signs against
it. `forte cloudflare status` probes both and reports which one failed.

There is no recovery path. `connect` refuses a project that is already
connected, and nothing else can replace a stored credential, so a revoked token
means the project has to be recreated. Rotation is on the list; it is not
built.
