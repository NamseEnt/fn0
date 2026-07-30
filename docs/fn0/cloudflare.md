# Bring your own Cloudflare account

Your project's object storage, public objects, deployed frontend assets,
cached pages and custom domain all run on **your** Cloudflare account, not
fn0's. You keep R2's free tier, your own purge budget and your own bill; fn0
runs the compute and holds no storage on your behalf.

This is a one-time setup per project, and it is required before a project can
serve a frontend.

## What you need

A Cloudflare account with **a zone in it** — a domain whose nameservers point
at Cloudflare. Not just an account: an R2 custom domain has to live in a zone
in the same account as the bucket, and that hostname is where your frontend
assets get served from.

## Two ways to set this up

Setup has to create buckets, point a hostname at one, write a cache rule and
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
| Zone | SSL and Certificates → Edit |

Restrict the zone scopes to the one zone. Then:

```sh
forte cloudflare provision \
  --account-id <account id> --zone-id <zone id> --api-token <token>
```

This creates the buckets, the CDN hostname and the cache rule, then stops and
prints the exact names for step 2.

**Step 2** — the two credentials fn0 keeps.

R2 → **Manage API Tokens → Create API token**, permission *Object Read &
Write*, applied to the three buckets the previous command named. The screen
shows an Access Key ID and a Secret Access Key; keep both.

My Profile → **API Tokens → Create Custom Token**, permission *Zone → Cache
Purge → Purge*, restricted to your zone.

```sh
forte cloudflare connect \
  --account-id <account id> --zone-id <zone id> --zone-name <your-domain> \
  --dataplane-access-key-id <Access Key ID> \
  --dataplane-secret <Secret Access Key> \
  --purge-token <purge token>
```

Delete the provisioning token from step 1 once this succeeds.

## What the two stored credentials can do

| Credential | What it can do |
| --- | --- |
| R2 data-plane | read and write objects in fn0's three buckets. Cannot reach another bucket, cannot delete a bucket, cannot call the Cloudflare API at all |
| Purge | purge this one zone's cache. Nothing else — not DNS, not cache rules, not R2, not certificates |

Those limits are measured against the live API, not inferred from the
permission names.

So the worst a total compromise of fn0 can do to your Cloudflare account is
rewrite objects in the three buckets it created, and clear your cache.

## What gets created

- `fn0-object-storage-<project-id>` — your app's private object storage
- `fn0-static-asset` — your deployed frontend and your public objects, served
  at `static.<your-domain>`
- `fn0-static-page` — cached SSR HTML, private

The last two are shared by every fn0 project in that account, separated by a
`<project-id>/` key prefix, so however many projects you run your zone needs
exactly one DNS record for them.

fn0 adds exactly one cache rule and leaves your own rules in place; running
setup again replaces that one rule rather than stacking copies. The rule also
pins browser caching on the assets hostname to whatever fn0 stored on the
object, because a zone's default Browser Cache TTL would otherwise leave
browser copies of a replaced object that no purge can reach. Your other
hostnames keep the zone setting.

Objects your project already had on the fn0 platform account are copied across
in the background, and the project keeps serving from the platform account
until that finishes. Check with:

```sh
forte cloudflare status
```

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
to do: add a **proxied** `A` record for that hostname pointing at the printed
IP.

The record must stay orange-clouded. An Origin CA certificate is not valid for
a direct visitor connection, so switching the record to DNS-only breaks the
hostname immediately and visibly.

## What fn0 still holds

- The compute, the request routing and the `<project>.fn0.dev` subdomain
- Your document database (Turso), which is not part of this
- The bundle store your deployed code is distributed from

## Removing a project

`forte destroy` empties fn0's buckets in your account but does not delete
them — fn0 holds an object-scoped credential there by design. The buckets are
yours to remove.

## Revoking

Deleting the data-plane token in your Cloudflare dashboard breaks the
project's storage at request time — there is no grace period, because every
request signs against it. `forte cloudflare status` reports the failure;
re-run `forte cloudflare connect` to issue fresh credentials.
