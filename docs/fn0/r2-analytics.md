# R2 analytics in the operations console

The operations console reads Cloudflare's R2 Analytics datasets directly from
the GraphQL Analytics API. It queries the fn0 account once per minute from
`/api/r2`; `/api/live` does not query Cloudflare Analytics and R2 results do not
affect HEALTHY, DEGRADED, DOWN, or UNKNOWN.

The rolling 24-hour view combines Cloudflare's [`r2StorageAdaptiveGroups`,
`r2OperationsAdaptiveGroups`, and `r2BandwidthUsageAdaptiveGroups` datasets](https://developers.cloudflare.com/r2/platform/metrics-analytics/):

- `r2StorageAdaptiveGroups`: latest returned payload bytes, metadata bytes,
  and object count for each bucket.
- `r2OperationsAdaptiveGroups`: operation request counts grouped by bucket and
  status (`success`, `userError`, `internalError`).
- `r2BandwidthUsageAdaptiveGroups`: uploaded and downloaded bytes grouped by
  bucket over the same period.

Only bucket names beginning with `fn0-` are returned. This includes Signy,
Bundle Store, Forte static, and per-project R2 buckets without publishing
object keys. Analytics freshness is the age of the newest dataset sample
returned for each bucket. The page refreshes this usage view once per minute.

Cloudflare excludes bandwidth transfers smaller than 100 KiB. Therefore
upload/download bytes describe the reported transfer totals, not all small
object transfers. R2 Analytics is a usage and history source, not a health
probe.

The Worker requires an account-scoped Cloudflare API token with Account
Analytics Read permission ([Cloudflare token setup](https://developers.cloudflare.com/analytics/graphql-api/getting-started/authentication/api-token-auth/)). Configure it as the secret Pulumi value
`cloudflareAnalyticsApiToken`. Until configured, the R2 endpoint returns an
unavailable state without making an upstream request. Account Analytics Read
is read-only; the Worker does not receive the R2 object storage credentials.

Cloudflare documents a [31-day maximum R2 query range](https://developers.cloudflare.com/r2/platform/metrics-analytics/)
and a [default 300 GraphQL requests per five minutes per user or token](https://developers.cloudflare.com/analytics/graphql-api/limits/).
The console uses one account query per minute, one account per query, and a
fixed 24-hour window.
