# fn0 Infrastructure

## Setup

### Cloudflare User API Token

A pre-provisioned Cloudflare **user-owned** API token is required to mint other user-owned tokens at provision time (e.g. the R2 presign token used by the static asset storage component).

Cloudflare's API rejects the nested "apply to all R2 buckets in this account" wildcard scope on account-owned tokens, so user-owned tokens are required for that scope. Minting a user-owned token via the Cloudflare API requires pre-existing user-owned authentication — hence this manual bootstrap step.

#### Required permission

| Scope | Permission |
| --- | --- |
| User | `API Tokens Edit` (dashboard) / `API Tokens Write` (API) |

No account or zone resources need to be selected — this token's only job is to issue other tokens.

#### Create the token

1. Open <https://dash.cloudflare.com/profile/api-tokens> → **Create Token** → **Custom token**.
2. Name: anything descriptive (e.g. `fn0-user-admin-token`).
3. Permissions: add `User` / `API Tokens Edit`.
4. Account Resources / Zone Resources / Client IP filter: leave default / empty.
5. TTL: leave Start Date and End Date empty (no expiry).
6. Continue and copy the token value — it is shown only once.

#### Store the token in Pulumi config

From this directory:

```sh
pulumi config set --secret fn0Cloud:cloudflareUserApiToken <token-value>
```

Once set, `pulumi up` provisions the R2 presign token (and any other user-owned tokens) automatically via this bootstrap token.
