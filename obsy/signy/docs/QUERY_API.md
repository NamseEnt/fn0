# The query API

This is the authoritative reference for signy's first-party read
surface. It is written for both consumers at once: the fn0 console, and any
agent driving `curl`. The API is small on purpose — GET requests, URL query
parameters, NDJSON out — and every refusal names the offending input, what the
endpoint accepts, and a correct form, so a caller that only sees error text
can correct itself. A test (`every_query_api_route_and_param_is_documented`)
fails if a route or parameter exists in the code and not on this page.

The engine assumes a secured channel ([`DEPLOYMENT.md`](DEPLOYMENT.md) §5):
there is no TLS and no authentication in this process, and the tenant a read
sees is whatever the `X-Tenant-Id` header says. The gateway in front — fn0's
control plane in the intended deployment — authenticates the caller and
**overwrites** that header.

Reads name their tenant in a header because they carry no payload to put it
in. Writes do not: an OTLP export names its tenant in the `tenant.id` resource
attribute, and signy reads no header on an ingest route. See
[`MULTI_TENANCY_DESIGN.md`](MULTI_TENANCY_DESIGN.md).

Every example below works as written:

```sh
curl -sH 'X-Tenant-Id: acme' \
  'http://127.0.0.1:3100/signy/api/v1/logs?start=-1h&attr=level=error&contains=timeout&limit=50'
```

## Routes

| Method | Path | Answers |
|---|---|---|
| GET | `/signy/api/v1/logs` | log rows matching the filters, newest first by default |
| GET | `/signy/api/v1/logs/histogram` | per-bucket row counts under the same filters |
| GET | `/signy/api/v1/logs/attributes` | attribute key names in the window (autocomplete) |
| GET | `/signy/api/v1/logs/attributes/{key}/values` | a key's values in the window (autocomplete) |
| GET | `/signy/api/v1/logs/tail` | a live chunked-NDJSON stream of new rows |
| POST/GET/DELETE | `/signy/api/v1/logs/delete` | submit / list / cancel deletion requests |
| GET | `/signy/api/v1/traces` | trace summaries matching the filters, newest first |
| GET | `/signy/api/v1/traces/{trace_id}` | every span of one trace, flat rows for a timeline |
| GET | `/signy/api/v1/traces/attributes` | span/resource attribute keys in the window (autocomplete) |
| GET | `/signy/api/v1/traces/attributes/{key}/values` | a key's values in the window (autocomplete) |
| GET | `/signy/api/v1/metrics/query` | per-series samples on a step grid, optionally rated and aggregated |
| GET | `/signy/api/v1/metrics/instant` | one value per series at a single instant (the alert evaluation) |
| GET | `/signy/api/v1/metrics/quantile` | a quantile interpolated from a histogram's `_bucket` series |
| GET | `/signy/api/v1/metrics/names` | metric names in the window (autocomplete) |
| GET | `/signy/api/v1/metrics/labels` | metric label keys in the window (autocomplete) |
| GET | `/signy/api/v1/metrics/labels/{key}/values` | a metric label key's values (autocomplete) |
| GET | `/signy/api/v1/metrics/series` | matching series identities, `__name__` included |

Unchanged and outside this document's scope: `/metrics` (Prometheus text),
`/ready`, the admin routes under `/signy/api/v1/admin`, and OTLP ingest.
A request to any other path answers 404 with this route list.

## The parameter grammar

One grammar serves every endpoint; each endpoint accepts the subset listed in
its section, and an unknown parameter is a 400 that names the accepted set.

| Parameter | Repeatable | Meaning |
|---|---|---|
| `start`, `end` | no | Absolute time — unix seconds (`1756100000`), milliseconds (13 digits), microseconds (16), nanoseconds (19), decimal seconds (`1756100000.123`), or RFC3339 (`2026-08-25T14:00:00+09:00`) — or relative: `-1h`, `-30m`, `-90s` is that long before now. The unit suffix is what makes it relative; a bare negative integer stays a negative epoch. Defaults: `end` = now, `start` = `end` − `SIGNY_MAX_QUERY_RANGE`. |
| `attr` | yes | An attribute filter with the operator embedded in the value: `attr=level=error`, `attr=level!=debug`, `attr=path=~/api/.*`, `attr=host!~db-.*`. The key ends at the first `!=`, `!~`, `=~`, `=`, `>=`, `<=`, `>`, or `<`. Regexes are anchored: the value must match whole. Repeated filters AND. The comparison operators exist for the trace endpoints' `duration` intrinsic only (`attr=duration>=250ms`); a log endpoint refuses them with an error that says so. |
| `contains` | yes | The line must contain this substring. |
| `not_contains` | yes | The line must not contain this substring. |
| `regex` | yes | The line must match this regex (unanchored, like `grep`). |
| `not_regex` | yes | The line must not match this regex. |
| `parse` | `json` and/or `logfmt`, once each | Run that parser over the stored line first; `attr` filters then see pushed attributes *and* extracted fields. One non-obvious rule, inherited from the pipeline: a pushed attribute **shadows** an extracted field of the same name. |
| `limit` | no | Rows to return (`/logs`), rows per poll (`/logs/tail`), or traces to return (`/traces`). Default 100, capped by `SIGNY_MAX_LOG_LIMIT` (`SIGNY_MAX_TRACE_SEARCH_LIMIT` on `/traces`). |
| `direction` | no | `forward` or `backward` (default) — ascending or descending time. |
| `bucket` | no | Histogram bucket width, duration syntax: `30s`, `5m`, `1h`. |
| `delay` | no | Tail only: whole seconds (≤ 5) to hold rows back, for writers whose clocks trail the server's. |
| `request_id` | no | Cancelling a delete request: the id from the GET listing. |
| `metric` | no | Metric endpoints: the exact `__name__` to read, like `metric=http_requests_total`. On `/metrics/quantile` it is the histogram's *base* name — the engine selects `<metric>_bucket` itself. |
| `step` | no | Metric range endpoints: the evaluation grid's spacing, duration syntax. Samples land at `start + k*step`. Required. |
| `func` | no | `rate` or `increase`, applied per series before any aggregation. Needs `range`. |
| `range` | no | The window `func` (or a quantile) reads per evaluation point, duration syntax: `range=60s`. |
| `agg` | no | `sum`, `avg`, `min`, `max`, or `count`, folding the per-series values at each step. |
| `by` | yes | Label keys the aggregation groups by, like `agg=sum&by=service`. Grouping is a property of the aggregation, so `by` without `agg` is refused. |
| `lookback` | no | How far behind an evaluation point a raw sample may be and still answer for it (default `5m`). |
| `q` | no | `/metrics/quantile` only: the quantile in `[0, 1]`, like `q=0.99`. |

All filters AND. There is deliberately no OR and no attribute-exists, and the
only numeric comparison is the trace endpoints' `duration` — the flat model
stays flat until a real consumer hits the wall. Percent-encode values
(`curl --get --data-urlencode` does it for you); a practical URL stays far
under the ~8 KB proxies allow.

## `GET /logs` — search

Accepts `start`, `end`, `attr`, `contains`, `not_contains`, `regex`,
`not_regex`, `parse`, `limit`, `direction`.

One row per line, `application/x-ndjson`, sorted in query direction:

```json
{"timestamp":"1756100000123456789","line":"request timed out","attributes":{"level":"error","service_name":"api"}}
```

`timestamp` is nanoseconds **as a string**: the values exceed 2^53, past
which JSON numbers silently lose precision in every JavaScript consumer.
Scan cost rides in response headers: `X-Signy-Scanned-Rows`,
`X-Signy-Scanned-Bytes`. The response is fully decided before the first
byte, so an error is always an HTTP status with a JSON body — never a broken
stream.

```sh
# Today's errors mentioning a user, oldest first
curl -sH 'X-Tenant-Id: acme' --get \
  --data-urlencode 'start=2026-08-25T00:00:00+09:00' \
  --data-urlencode 'attr=level=error' \
  --data-urlencode 'contains=user_4711' \
  --data-urlencode 'direction=forward' \
  'http://127.0.0.1:3100/signy/api/v1/logs'

# Structured search over JSON lines: parsed fields become filterable
curl -sH 'X-Tenant-Id: acme' \
  'http://127.0.0.1:3100/signy/api/v1/logs?start=-15m&parse=json&attr=status=500'
```

## `GET /logs/histogram` — the chart

Accepts `start`, `end`, `attr`, `contains`, `not_contains`, `regex`,
`not_regex`, `parse`, `bucket`.

Counts rows per bucket under the same filters `/logs` takes. Buckets are
half-open `[bucket_start, bucket_end)`, epoch-aligned to the width, and
clipped to the query range — the partial first and last buckets count only
in-range rows. Empty buckets are emitted, so the series is dense. Without
`bucket`, the smallest of 1s/10s/1m/10m/1h/1d keeping the count ≤ 100 is
chosen; the hard cap is `SIGNY_MAX_METRIC_EVALUATION_POINTS`.

```json
{"bucket_start":"1756100000000000000","bucket_end":"1756100030000000000","count":42}
```

`sum(count)` over the buckets equals the row count `/logs` would return for
the same filters and range (unlimited), and a test holds the two to it.

## `GET /logs/attributes` and `/logs/attributes/{key}/values` — autocomplete

Keys accept `start`, `end`. Values accept `start`, `end`, `attr`.

```json
{"key":"service_name"}
{"value":"api"}
```

Keys come from the memtable and the part metadata census. Values come from a
bounded sample — the newest 1000 rows in the window — not from a catalog:
rare or old values may be missing over a long range. The optional `attr`
filters on the values endpoint narrow the sample to matching rows, so a
filter chip dropdown offers values that actually co-occur with the chips
already placed. Line filters are not accepted here: values are sampled
without evaluating line content, and answering as though they were filtered
by it would be a silent approximation.

## `GET /logs/tail` — follow

Accepts `start`, `attr`, `contains`, `not_contains`, `regex`, `not_regex`,
`parse`, `limit`, `delay`.

A chunked NDJSON stream: rows in the same shape as `/logs`, ascending in
time, plus a keep-alive line every ~15 idle seconds:

```json
{"heartbeat":true}
```

A heartbeat is not data — skip any line carrying a `heartbeat` key. `start`
is the resume cursor: reconnect with the last timestamp you saw and nothing
is missed (a row you already have may repeat; rows are deduplicated per
connection, not across them). Nothing is ever dropped: a burst larger than
`limit` is delivered across later polls, so a slow reader falls behind
visibly rather than losing lines. On shutdown the stream ends cleanly —
reconnect. Concurrent tails are capped by
`SIGNY_MAX_CONCURRENT_TAILS`; past the cap the request is refused with
429 rather than queued.

```sh
curl -sNH 'X-Tenant-Id: acme' \
  'http://127.0.0.1:3100/signy/api/v1/logs/tail?attr=service_name=api&contains=ERROR'
```

The proxy in front must pass streaming responses through unbuffered and not
apply an idle timeout shorter than the heartbeat interval.

## `GET /traces` — search

Accepts `start`, `end`, `attr`, `limit`.

One summary per matching trace, `application/x-ndjson`, newest first (by the
trace's first windowed span, descending):

```json
{"trace_id":"0af7651916cd43dd8448eb211c80319c","root_service":"api","root_name":"GET /items","start":"1756100000123456789","end":"1756100000273456789","duration":"150000000","span_count":3}
```

Three rules decide what matches and what the summary says, and each exists
for the same reason — the scan must be prunable to the window:

- **A trace matches when any of its spans overlaps the window.** A request
  that began before the window and was still running inside it is exactly
  what an operator searches for, and overlap is the predicate the storage's
  row-group bounds already answer, so the scan reads only the row groups that
  can contribute.
- **Every `attr` filter must be matched by at least one span of the trace**
  (not necessarily the same span for all filters). Filters read the span's
  flattened view: the intrinsics `name`, `duration`, `status`
  (`unset`/`ok`/`error`), then `service.name`, span attributes, resource
  attributes. An absent key compares as the empty string, exactly as on logs.
- **The summary is built from the windowed spans only** — `root_*` from the
  first windowed span without a parent (or the earliest), `start`/`end`/
  `duration`/`span_count` from what the window holds. Reading the rest of
  the trace would mean restoring the parts the window was pruned to avoid;
  the full extent is what the by-id fetch shows.

`duration` is where the comparison operators apply, and they are **per
span**: `attr=duration>=1.5s` means "some span ran at least 1.5s", not "the
trace's extent is at least 1.5s" — a deliberate difference from Tempo's
`minDuration`. Values need a unit (`250ms`, `1.5s`); equality parses the
unit too, so `attr=duration=150ms` compares nanoseconds rather than strings.
A comparison on any other key is refused: every other value is stored
stringified, and a lexicographic `>=` would answer wrongly without saying so.

```sh
# Slow error traces from the api service, last hour
curl -sH 'X-Tenant-Id: acme' --get \
  --data-urlencode 'start=-1h' \
  --data-urlencode 'attr=service.name=api' \
  --data-urlencode 'attr=status=error' \
  --data-urlencode 'attr=duration>=1.5s' \
  'http://127.0.0.1:3100/signy/api/v1/traces'
```

## `GET /traces/{trace_id}` — the trace timeline

Takes **no query parameters** — the trace id in the path is the whole
request, and a query string is a 400 that says so. The id is 32 hexadecimal
characters, exactly as ingest stored it and as log rows carry it in their
`trace_id` attribute.

One span per line, `application/x-ndjson`, sorted by start time then span id:

```json
{"trace_id":"0af7651916cd43dd8448eb211c80319c","span_id":"b7ad6b7169203331","parent_span_id":"","name":"GET /items","kind":"server","service":"api","status":"ok","start":"1756100000123456789","end":"1756100000273456789","duration":"150000000","attributes":{"http.method":"GET","service.name":"api"},"events":[{"timestamp":"1756100000200000000","name":"exception","attributes":{"exception.type":"IOError"}}]}
```

`start`, `end`, and `duration` are nanoseconds as strings, like every
timestamp this API emits. `kind` is one of `unspecified`, `internal`,
`server`, `client`, `producer`, `consumer`; `status` is `unset`, `ok`, or
`error`. `attributes` is one merged map — resource attributes first, span
attributes overwriting same-named keys — with every value stringified;
non-scalar values (arrays, kvlists) appear as compact JSON rather than being
dropped. An empty `parent_span_id` marks a root span.

Retention holds span by span: spans below the tenant's retention floor are
omitted, and a trace with none left answers 404 — the same answer as an id
that never existed here, because this engine cannot tell the two apart.

```sh
curl -sH 'X-Tenant-Id: acme' \
  'http://127.0.0.1:3100/signy/api/v1/traces/0af7651916cd43dd8448eb211c80319c'
```

## `GET /traces/attributes` and `/traces/attributes/{key}/values` — autocomplete

Keys accept `start`, `end`. Values accept `start`, `end`, `attr`.

Same record shapes as the log autocomplete — `{"key":…}` / `{"value":…}`
lines, sorted. Keys are the union over the window's spans of the intrinsics
`duration`, `name`, `status`, plus `service.name` where a span carries it,
span attribute keys, and resource attribute keys. Values are read through the
same flattened view the filters use, and the optional `attr` filters narrow
them to traces the already-placed filters match — search semantics, per
trace — so a filter chip dropdown offers only values whose click still
returns something.

Unlike the log autocomplete there is no catalog to read: a span's attributes
live inside its stored payload, so these endpoints answer from a bounded
window scan and pay the same admission as any trace scan (its scan slots, the
shared memory account, the span budget). Keep the window as narrow as the
dropdown allows.

## `GET /metrics/query` — the dashboard panel

Accepts `metric`, `attr`, `start`, `end`, `step`, `func`, `range`, `agg`,
`by`, `lookback`, `limit`.

One line per output series, `application/x-ndjson`: the series' labels
(`__name__` omitted — the query names the metric) and its samples on the
grid, ascending:

```json
{"labels":{"instance":"instance-0","service_name":"api"},"samples":[["1756100000000000000",12.5],["1756100030000000000",13.0]]}
```

Selection is `metric=` (exact `__name__` equality, required) plus repeated
`attr` filters with the four matchers — the same grammar as everywhere else,
and a comparison operator is refused the same way the log surface refuses it.
`start` is required (the step grid is aligned to it); `end` defaults to now.
At each `t = start + k*step`:

- Without `func`, a series answers its newest sample in `(t − lookback, t]`;
  a step with no sample within the lookback is **omitted**, not zeroed.
- `func=increase` answers the sum of the **positive deltas** over
  `(t − range, t]`, walking from the last sample at or before the window's
  start; a counter reset contributes the post-reset value. `func=rate` is
  that divided by the window in seconds. **Nothing is scaled or
  extrapolated**: a window the samples only half cover answers half the
  increase, because that is what arrived.

  This differs from both neighbours, and the difference was measured rather
  than assumed (2026-08-27, `COMPARISON_METRICS.md`). Prometheus extrapolates
  to the window boundaries. VictoriaMetrics does not extrapolate *past* the
  data, but it does scale a partially-covered window up to the full range —
  at a dataset's trailing edge, where a 60 s window held 50 s of samples, it
  answered `60/50` of what this engine answered. The three agree wherever a
  window is fully covered, which is every window a live dashboard asks
  about; they diverge at the edges of a finite dataset.
- `agg` then folds the per-series values at each step across the series,
  grouped by the `by` projection of their labels (no `by`: one group, empty
  labels; a key a series lacks is omitted from its group's labels rather
  than materialized empty). `count` counts contributing series.

One function and one aggregation per request — there is no expression
language. A ratio is two requests composed client-side, and the refusal for
anything more says exactly that.

A selector matching more series than
`SIGNY_MAX_METRIC_SERIES_PER_QUERY`, or `series × steps` past
`SIGNY_MAX_METRIC_POINTS_PER_QUERY`, is refused **before any chunk is
decoded**, with the matched count and the knob. The fixes are narrowing the
selector, shortening the window, or coarsening `step` — aggregation is not
one of them, because the scan decodes every matched series whether or not the
fold aggregates.

```sh
# Request rate per service, last hour, 30s grid
curl -sH 'X-Tenant-Id: acme' --get \
  --data-urlencode 'metric=http_requests_total' \
  --data-urlencode 'start=-1h' \
  --data-urlencode 'step=30s' \
  --data-urlencode 'func=rate' \
  --data-urlencode 'range=60s' \
  --data-urlencode 'agg=sum' \
  --data-urlencode 'by=service' \
  'http://127.0.0.1:3100/signy/api/v1/metrics/query'
```

What ingest already decided, visible on this surface: exponential OTLP
histograms are downscaled to at most 64 finite bucket boundaries at ingest
(quantile precision is boundary-limited, like any bucketed histogram). A
histogram is **stored as one series** carrying its whole bucket vector, and
`<name>_bucket{le=...}`, `<name>_sum` and `<name>_count` are synthesized when a
selector asks for them — an instrument costs one identity in the index and the
catalogs rather than `bounds + 3`, and every name it used to answer as still
answers. OTLP summaries
become `{quantile="…"}` gauge series plus `_sum`/`_count`; delta-temporality
sums are accumulated into running totals at ingest, and a series that churns
away and returns restarts its total — a counter reset, which `rate` absorbs;
OTLP exemplars are dropped.

## `GET /metrics/instant` — the alert evaluation

Accepts `metric`, `attr`, `at`, `func`, `range`, `agg`, `by`, `lookback`,
`limit`.

The `/metrics/query` grammar at a single instant `at` (default now). One
line per output series:

```json
{"labels":{"service_name":"api"},"timestamp":"1756100000000000000","value":0.97}
```

This is the shape fn0's alert rules evaluate: compare `value` against the
threshold; `agg=max` gives the worst instance in one line.

```sh
curl -sH 'X-Tenant-Id: acme' --get \
  --data-urlencode 'metric=http_errors_total' \
  --data-urlencode 'func=rate' --data-urlencode 'range=60s' \
  --data-urlencode 'agg=max' \
  'http://127.0.0.1:3100/signy/api/v1/metrics/instant'
```

## `GET /metrics/quantile` — the latency panel

Accepts `metric`, `q`, `attr`, `start`, `end`, `step`, `range`, `by`,
`limit`.

`metric` names the histogram's **base** name — the engine selects
`<metric>_bucket` and groups by the series' labels minus `le` (or by the
`by` projection when given). Per group and step, each bucket's `increase`
over `(t − range, t]` is taken (`range` is required: a bucket count without
a window is a lifetime total), the cumulative counts are monotone-fixed, and
the quantile is interpolated linearly within the bracketing bucket — the
`histogram_quantile` convention, the `+Inf` bracket answering the highest
finite bound. The response is the `/metrics/query` samples shape.

A summary-backed name is refused with the reason: summary quantiles were
computed by the client and cannot be re-aggregated — query the
`<metric>{quantile="0.99"}` series with `/metrics/query` instead.

```sh
curl -sH 'X-Tenant-Id: acme' --get \
  --data-urlencode 'metric=http_request_duration_seconds' \
  --data-urlencode 'q=0.99' \
  --data-urlencode 'start=-1h' --data-urlencode 'step=30s' \
  --data-urlencode 'range=60s' \
  --data-urlencode 'attr=service_name=api' \
  'http://127.0.0.1:3100/signy/api/v1/metrics/quantile'
```

## `GET /metrics/names`, `/metrics/labels`, `/metrics/labels/{key}/values`, `/metrics/series` — autocomplete

Names accept `start`, `end`. Labels and values accept `start`, `end`,
`metric`, `attr`. Series accepts `metric`, `attr`, `start`, `end`, `limit`.

```json
{"name":"http_requests_total"}
{"key":"service_name"}
{"value":"api"}
{"labels":{"__name__":"http_requests_total","service_name":"api"}}
```

Unlike the log attribute endpoints these are **exact**, not sampled: a
series' identity lives in the memtable index and the part catalogs, so the
answers come from catalogs alone and no sample body is read. Both halves obey
the window — the memtable from the timestamp span it records per series, the
parts from their catalogs — so a key whose samples all sit outside the window
is not offered. `/metrics/series` is
the one metric surface whose labels objects keep `__name__`: it enumerates
identities, and without the name two metrics' series would be
indistinguishable. The optional `metric`/`attr` narrowing on labels, values
and series means a filter chip dropdown offers only values that co-occur
with the chips already placed.

## `/logs/delete` — deletion requests

Deletion semantics — hide immediately, remove at the next rewrite — are in
[`RETENTION_DESIGN.md`](RETENTION_DESIGN.md). The surface:

- `POST /logs/delete?attr=app=api&contains=secret&start=-24h` — at least one
  `attr` filter and an explicit `start` are required; `end` defaults to the
  moment of submission. `parse` is not in this endpoint's grammar: deleting
  by a parsed field would change meaning whenever the parser does. Answers
  204.
- `GET /logs/delete` — the tenant's requests, one NDJSON line each. The
  `query` field is the persisted canonical form (percent-encoded flat
  filters) — resubmittable as-is.
- `DELETE /logs/delete?request_id=<id>` — withdraw a request whose rows are
  still only hidden. Answers 204, or 404 for an unknown id.

A tenant may hold a bounded number of requests at once; past it the POST
answers 429 with the bound. Requests persist across restarts in the
canonical form above. A stored request from before this format fails startup
loudly; the fix is to delete the stored object and re-submit.

## Errors

Refusals are `application/json`:

```json
{"error":"unknown parameter 'atr': this endpoint accepts start, end, attr, … — see docs/QUERY_API.md"}
```

| Status | Meaning | Retry? |
|---|---|---|
| 400 | The request is malformed or over-broad; the message names the input and the governing limit. Includes a request whose materialization would be larger than this instance's whole memory budget (`SIGNY_MEMORY_ACCOUNT_BYTES`) — no wait makes that fit, so it is not a `429` | No — fix the request |
| 401/403 | Tenant refusals from `X-Tenant-Id` handling ([`MULTI_TENANCY_DESIGN.md`](MULTI_TENANCY_DESIGN.md)). Reads only: an ingest never answers one, because it never reads a tenant off a header | No |
| 404 | Unknown route (the body lists the real ones) or unknown delete request | No |
| 413 | The trace holds more spans or bytes than one response may carry (`SIGNY_MAX_TRACE_SPANS`, `SIGNY_MAX_QUERY_MEMORY_BYTES`), or a metric selector matched more than `SIGNY_MAX_METRIC_SERIES_PER_QUERY` series / `SIGNY_MAX_METRIC_POINTS_PER_QUERY` points | No — narrow the request or raise the knob |
| 429 | Tenant query quota, tail cap, delete cap, or this instance's shared memory account is momentarily full. The account is priced before the scan starts, so this arrives on the request rather than partway through it | Yes — these clear on their own |
| 503 | Draining for shutdown, or a deletion could not be made durable | Yes, against the replacement instance |
| 504 | The query ran past `SIGNY_MAX_QUERY_RUNTIME` (`SIGNY_MAX_TRACE_QUERY_RUNTIME` on the trace routes, `SIGNY_MAX_METRIC_QUERY_RUNTIME` on the metric ones) | Narrow the range or filters |

## Limits

Every knob that bounds a query — range, runtime, scanned rows and bytes,
memory, limits, tail count — is documented in
[`CONFIGURATION.md`](CONFIGURATION.md) under its `SIGNY_*` name; the
messages above cite the same names.
