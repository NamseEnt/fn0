# Bring your own Cloudflare account

Your project's object storage, public objects, deployed frontend assets,
CDN-cached pages and custom domain all run on **your** Cloudflare account, not
fn0's. You keep R2's free tier, your own purge budget and your own bill; fn0
runs the compute and holds no storage on your behalf.

This setup is required before a project can serve a frontend: fn0 Cloud has no
way to meter object storage usage on an account it does not own, so
bring-your-own-Cloudflare is the default premise for every project, not an
opt-in. There is no `fn0.dev` fallback to fall back to — every project gets a
hostname inside one of the owner's zones.

## What you need

A Cloudflare account with **a zone in it** — a domain whose nameservers point
at Cloudflare. Not just an account: an R2 custom domain has to live in a zone
in the same account as the bucket, and that hostname is where your frontend
assets get served from.

## One command

```sh
forte cloud init \
  --project . \
  --project-name my-app \
  --zone example.com
```

The command is non-interactive. Set `CLOUDFLARE_API_TOKEN` in the execution
environment before running it. `forte login` must also be run first — the
command loads fn0 credentials to register the project and fails immediately if
they are absent. The token is not a command-line argument, and
the CLI never prompts or reads standard input. If the variable or any required
argument is missing, the command exits with an error.

As part of initialization, the CLI enables WebSockets for the selected zone.
This is required for Forte WebSocket routes to complete their upgrade through
Cloudflare's proxy. It is safe to run repeatedly because the setting is
written to `on` each time.

`--zone` is the zone name, such as `example.com`. It is not the hexadecimal
zone ID shown in Cloudflare's API details. The CLI resolves the exact zone
named by the argument and never picks the first zone returned by the API.

`--project-name` is also a DNS label. It must contain only lowercase ASCII
letters, digits, and hyphens, be 1–63 characters long, and start and end with a
letter or digit. The public hostname is derived as:

```text
<project-name>.<zone>
```

For example, this project answers on `my-app.example.com`. A separate domain
argument is not needed.

## Setup credential

Setup has to create buckets, point a hostname at one, write one zone rule, add
one DNS record and sign a certificate. Create one reusable bootstrap token and provide it through
`CLOUDFLARE_API_TOKEN`. Cloudflare dashboard → **My Profile → API Tokens →
Create Token → Create Custom Token**. Give it exactly one permission:

| Scope | Permission |
| --- | --- |
| User | API Tokens → Edit |

The CLI uses this token locally for every project. It creates a short-lived
provisioning token when needed, provisions the account, creates the narrow
credentials fn0 keeps, and revokes the short-lived token. The bootstrap token
itself is not sent to fn0.

This token is powerful: a token that can create tokens can create any token
allowed by the account. Store it in a secret manager or protected process
environment and rotate it deliberately. Do not put its value in the command
line, project files, or logs.

## What the stored credentials can do

| Credential | What it can do |
| --- | --- |
| Worker R2 | read and write objects in this project's two object-storage buckets. Cannot reach the frontend-asset bucket, cannot reach another project's buckets, cannot delete a bucket, cannot call the Cloudflare API at all |
| Frontend-asset R2 | read and write objects in this project's frontend-asset bucket, and nothing else. Never sent to a worker |
| Purge | purge this one zone's cache. Nothing else — not DNS, not cache rules, not R2, not certificates |

Those limits are measured against the live API, not inferred from the
permission names.

So the worst a total compromise of fn0 can do to your Cloudflare account is
rewrite objects in the three buckets it created for this project, and clear your
cache.

## What gets created

Three buckets, all this project's alone. Nothing is shared with your other fn0
projects, so no key prefix is carrying the separation.

| Bucket | Holds | Reachable at |
| --- | --- | --- |
| `fn0-<project-id>-private-object-storage` | what `object_storage::private` writes | nowhere — signed requests only |
| `fn0-<project-id>-public-object-storage` | what `object_storage::public` writes | `fn0-<project-id>-public-object-storage.<zone>` |
| `fn0-<project-id>-frontend-asset` | your deployed frontend build | `fn0-<project-id>-frontend-asset.<zone>` |

The two public buckets answer on a hostname that is the bucket's own name in
your zone, so a bucket and the address it serves from cannot drift apart. Each
costs one DNS record.

fn0 adds one rule to your zone and leaves your own rules in place. It matches
`fn0-*-frontend-asset.<zone>` and
`fn0-*-public-object-storage.<zone>`; the cache rule also matches the
custom domains registered for fn0 projects. This covers every fn0 project you
add without a rule per project — a free zone allows ten of each, and a rule per
project would run out at ten projects. Both halves of each pattern are required
so a rule cannot swallow a hostname of your own.

The **cache rule** makes static HTML eligible for the CDN and respects the
origin's cache headers. Your other hostnames keep the zone setting.

Smart Tiered Cache is enabled for the zone so a cache miss in an edge location
can be filled by an upper tier instead of reaching the worker fleet directly.

The two buckets a browser can reach are also given a **CORS allowlist holding
one origin: your project's own domain**. Cloudflare keys a separate cache entry
per `Origin` value and `Origin` is not verified, so an allowlist of `*` would
let any site on the web read every one of those entries and bill the misses to
you. The allowlist moves with the domain, so changing the domain rewrites it.

Workers pick a connection up within about a second; no redeploy is needed.

**Set the project up before it stores anything.** A project has nowhere to
store and cannot serve a frontend until it is connected.

Connecting is first-time only, and there is no way back. Reconnecting, rotating
a credential and moving to a different Cloudflare account are all unsupported —
not merely undocumented, but refused. If a stored credential is lost or
revoked, the project cannot be repaired through the CLI. Treat the three
credentials as things you do not lose.

## The domain

Not optional: a project answers on the hostname derived from its project name
and zone, and on nothing else. There is no `fn0.dev` fallback.

Signing an origin certificate needs a permission fn0 deliberately does not
hold, so this runs locally too. The CLI generates a key pair, has Cloudflare
sign the certificate through your own Origin CA, uploads the certificate and
key, and then writes the **proxied** `CNAME` for that hostname into your zone,
pointing at the fn0 origin hostname. Nothing is left for you to add by hand.

The record is written last, after fn0 holds the certificate and the zone
carries the cache rule, so the hostname resolves nowhere until everything
behind it is ready.

A hostname that is already taken is not silently overwritten. An existing
`CNAME` on that name is repointed — you named the hostname on the command line,
so where it resolves is what you are asking to change — but an `A` or `AAAA`
record there stops the command with an error instead. Changing a project's
domain removes the old record, and only that record: if the `CNAME` fn0 wrote
has since been edited, it is left in place and reported. A record still
pointing at fn0 is worth removing — the next project to register that hostname
inherits whatever reaches it.

The derived hostname is written to `Forte.toml`, and `forte deploy` refuses if
the stored project name, zone, and hostname disagree with the live project.
Changing the zone or project hostname requires a new project connection.

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
it.

There is no recovery path. A project that is already connected is refused a
second connection, and nothing else can replace a stored credential, so a
revoked token means the project has to be recreated. Rotation is on the list;
it is not built.
