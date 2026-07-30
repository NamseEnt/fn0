# Bring your own Cloudflare account

Your project's object storage, public objects, deployed frontend assets,
cached pages and custom domain all run on **your** Cloudflare account, not
fn0's. You keep R2's free tier, your own purge budget and your own bill; fn0
runs the compute and holds no storage on your behalf.

This is a one-time setup per project, and it is required before a project can
serve a frontend.

## What fn0 is trusted with

Setup needs an account-wide token. fn0 never receives it: `forte cloudflare
connect` runs that part on your machine, and what it sends to fn0 is two much
smaller credentials it mints on the way out.

| Credential | Where it lives | What it can do |
| --- | --- | --- |
| Your setup token | your machine, for the length of one command | everything below, plus delete any bucket in the account |
| R2 data-plane token | fn0 | read and write objects in fn0's three buckets. Cannot reach another bucket, cannot delete a bucket, cannot call the Cloudflare API at all |
| Purge token | fn0 | purge this one zone's cache. Nothing else |

Those limits are measured against the live API, not inferred from the
permission names.

So the worst a total compromise of fn0 can do to your Cloudflare account is
rewrite objects in the three buckets it created, and clear your cache. It
cannot delete a bucket, reach R2 data you keep for anything else, issue
certificates, or change your account.

## What you need

A Cloudflare account with **a zone in it** — a domain whose nameservers point
at Cloudflare. Not just an account: an R2 custom domain has to live in a zone
in the same account as the bucket, and that hostname is where your frontend
assets get served from.

## 1. Create the setup token

Cloudflare dashboard → **My Profile → API Tokens → Create Token → Create
Custom Token**. Give it:

| Scope | Permission | Why |
| --- | --- | --- |
| Account | Workers R2 Storage → Edit | Create the buckets and attach the CDN hostname |
| Zone | Zone → Read | Resolve your zone's name |
| Zone | Cache Purge → Purge | Prove the purge path before fn0 depends on it |
| Zone | Cache Rules → Edit | Create the cache rule that makes the assets hostname cacheable and purgeable |
| Zone | SSL and Certificates → Edit | Sign the origin certificate for a custom domain |
| User | API Tokens → Edit | Mint the two narrow tokens fn0 actually gets |

Restrict the zone scopes to the one zone you want to use.

This token is powerful, which is exactly why it stays on your machine. Delete
it once setup is done; nothing fn0 runs will ever need it again.

The two cache permissions are not optional. Public objects are stored with a
one-year edge TTL and a purge is the only thing that replaces them — without
Cache Purge, overwriting an object silently keeps serving the old bytes, and
without Cache Rules the rule that makes `PURGE` reach the edge at all cannot
be created.

## 2. Connect

```sh
forte cloudflare connect \
  --account-id <cloudflare account id> \
  --zone-id <zone id> \
  --api-token <the token you just made>
```

Everything up to the last step happens locally. The command creates three
buckets in your account, points a CDN hostname at the first, writes one cache
rule, mints the two narrow tokens, and only then talks to fn0.

- `fn0-object-storage-<project-id>` — your app's private object storage
- `fn0-static-asset` — your deployed frontend and your public objects, served
  at `static.<your-domain>`
- `fn0-static-page` — cached SSR HTML, private

The last two are shared by every fn0 project in that account, separated by a
`<project-id>/` key prefix, so however many projects you run your zone needs
exactly one DNS record for them.

fn0 adds exactly one cache rule and leaves your own rules in place; running
`connect` again replaces that one rule rather than stacking copies. The rule
also pins browser caching on the assets hostname to whatever fn0 stored on the
object, because a zone's default Browser Cache TTL would otherwise leave
browser copies of a replaced object that no purge can reach. Your other
hostnames keep the zone setting.

Objects your project already had on the fn0 platform account are copied across
in the background, and the project keeps serving from the platform account
until that finishes. Check with:

```sh
forte cloudflare status
```

## 3. Custom domain (optional)

Signing an origin certificate needs a permission fn0 deliberately does not
hold, so this command also runs the signing locally and needs the setup token
again:

```sh
forte domain add app.example.com \
  --account-id <account id> --zone-id <zone id> --api-token <token>
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
