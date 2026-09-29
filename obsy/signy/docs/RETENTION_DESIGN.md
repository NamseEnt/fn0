# Per-tenant retention design

Design record for per-tenant retention in signy. Written to be
self-contained: a fresh context should be able to start implementing from this
document alone.

Status: **implemented.** Push intake and the two-layer deletion are both live
code; the polling loop, its five configuration variables and the direct
`reqwest` dependency are gone. signy no longer calls out to anything but
its own object store.

Closes the open question left by `MULTI_TENANCY_DESIGN.md` ("Where does a
tenant's tier come from?", lines 416-420) and supersedes the
`Partition on (tier, day)` row of its status table. Partitions stay on `day`.

## Why this exists

Retention was one global `retention_period` applied to every tenant. A platform
needs per-tenant retention because retention is a plan attribute.

The requirement that drove the design: **a tenant that upgrades or downgrades
should get the retention they pay for now, not the one that was in force when
the bytes were written.**

Any scheme that bakes retention into the write path fails that requirement,
because the write path only knows the tier at write time. That rules out the
obvious designs, so they are recorded here to keep a fresh context from
re-deriving them.

### Rejected: partition by tier or by expiry, tier supplied at ingest

Partition on `(tier, day)` — or, better, on the computed expiry day — and let
the client pass its retention in a request header. Deletion becomes a prefix
listing plus a free DELETE, and no rewrite is ever needed.

Rejected because retention is then **fixed at write time**. Upgrading from a
3-day plan to a 30-day plan leaves the last three days of data dying on the old
schedule. Making it retroactive requires either an event-driven backfill or a
rewrite at merge, which is exactly the cost this design was supposed to avoid.
It also requires the writer to name the retention alongside the tenant, and a
new `LGY4` journal framing to carry it through crash replay — the record's
tenant is framed for the same reason: replay reads the stored bytes back, and
what decided the routing has to survive in the frame rather than in whatever
carried it in,
and a rewrite of the `part/metadata.rs:205` invariant that ties a partition
name to its data's day.

### Rejected: a separate GC process

Move retention out of signy into a janitor process that reads the control
plane and rewrites objects.

Rejected on two grounds. signy holds **one machine, one process** as a
design constraint. And the system rests on a single-writer assumption:
`operation_lock` (`part_registry.rs:25`) is an in-process `RwLock` and means
nothing across a process boundary, so a second writer would force a re-review
of every crash-safety invariant built in M5-M7 — temp-dir rename, merge
tombstones, startup cache reconciliation, manifest CAS contention, and orphan
attribution. Too much risk for one feature.

### Rejected: polling the control plane for the whole map

The first implementation polled `GET <url>` every five minutes and kept the
last successful response in memory. It has been removed; this section is why.

Rejected because **the map holds relative durations, and a relative duration
means something different every time it is re-applied.** A map that says
`acme: 3d` deletes more data every hour it stays in force. A stale map is not a
frozen decision, it is an actively destructive one.

Concretely: the control plane goes down, a customer upgrades from `3d` to
`30d`, signy cannot see the upgrade, and every retention tick keeps
deleting another slice of data the new plan should have kept. The exposure is
the length of the outage — hours or days — and signy cannot even detect
the situation, because "I could not fetch" does not distinguish *the policy is
unchanged* from *the policy changed and I cannot see it*.

Polling puts the knowledge on the wrong side. Only the control plane knows
whether a policy changed, and only it can know whether signy has been
told. Push moves the retry duty to the side holding that knowledge and bounds
the exposure to a retry rather than to an outage. A second, smaller reason: the
polled map lived in memory only, so a restart dropped it entirely.

### Rejected: absolute-cutoff delete commands with job tracking

Considered: drop the retention concept from signy altogether and expose
`delete tenant X's data older than <absolute timestamp>`, returning a job id,
with a second endpoint reporting whether that job has finished.

The core of the idea is right. An absolute cutoff is idempotent, order-
independent, and safe to store indefinitely — precisely what a relative
duration is not. Per-tenant watermarks under a `max` merge would make retries
and reordering free.

It was rejected on the completion side. Physical deletion here is deliberately
lazy: a part is rewritten only when enough of it has expired, and a part that
has grown past `merge_max_part_rows` is never merged again. So a large part
holding a handful of expired rows is reclaimed by no existing path, and a job
defined as *the bytes are gone* would never complete. Making it complete means
rewriting large parts to reclaim a few rows on demand — the exact cost the lazy
design exists to avoid — plus durable job records reconciled across restarts.

Best-effort retention needs none of that. Revisit only if a completion
guarantee becomes a product requirement.

### Rejected: an instance-side maximum

`SIGNY_MAX_TENANT_RETENTION` capped every pushed value at a duration the
operator set. It has been removed; this section is why.

The cap was applied where the value was read, which structurally could only
reach tenants that *had* a policy. A tenant the control plane never mentioned
has no entry to clamp, so it kept everything. With the cap at `30d` that
produced:

| What the control plane said | Result |
|---|---|
| nothing at all | kept forever |
| `"infinite"` — keep this forever | **deleted after 30 days** |

**Asking for preservation cost data that staying silent would have kept.** That
inverts the central safety rule: signy invented a deletion nobody
requested, from an instruction that said the opposite. It was also invisible —
`GET` returned `"infinite"`, and `tenant_policy_infinite_tenants` counted the
tenant as infinite, because both reported what was pushed rather than what was
applied.

Fixing the inversion the other way — clamping unknown tenants too — was
rejected because it means the instance deleting data for tenants the control
plane has never mentioned, which is the one thing this design refuses to do.
Rejecting an over-long push with a `400` was rejected as a second authority on
plans: the control plane already decides retention, and a value it cannot push
is a plan it cannot sell.

So the cap is gone and the control plane is the sole authority. Note that no
guarantee was lost: a real "nothing on this box outlives *n*" bound never
existed, because unknown tenants always bypassed it. If such a bound is ever
needed it belongs in the control plane, which knows every tenant, rather than in
an instance that only knows the ones it was told about.

## Decision

**The control plane pushes one tenant's retention at a time. signy
persists it before acknowledging, and applies it at deletion time.**

Two independent halves:

- **Intake is push, per tenant, durable.** signy never calls out. The
  control plane learns immediately whether its change took effect, and owns the
  retry when it did not.
- **Application is at deletion time, not write time.** Upgrades and downgrades
  are honoured automatically, with no header, no journal change, and no
  partition-scheme change.

Three properties make the second half cheap:

1. **The tenant index already exists.** `meta.json` carries a
   `Vec<TenantSegment>` with each tenant's row-group range, `min`/`max`
   timestamps and row count (`part/metadata.rs:66`,
   `validate_tenant_segments` at `:247`). Deciding *who has expired in this
   part* reads local metadata only — **no object is downloaded to make the
   decision.**
2. **Logical and physical deletion are separated.** Users see retention
   enforced immediately; bytes are reclaimed lazily, when it is cheapest.
3. **Rewrites reuse merge.** Retention never writes a part itself.

## Policy intake

### Contract

```
PUT /signy/api/v1/admin/tenants/{tenant}/retention
Content-Type: application/json

{"revision": 1, "retention": "30d"}

200 OK    the policy is durable and in force; the response names applied, duplicate, or stale
400       malformed tenant id or retention value; nothing stored
409       the revision already names a different policy body
503       could not be persisted; the control plane must retry
```

```
GET    /signy/api/v1/admin/tenants/{tenant}/retention
       200 {"tenant":"acme","retention":"30d","updated_at":"<rfc3339>"}
       404 no policy for this tenant

DELETE /signy/api/v1/admin/tenants/{tenant}/retention
       200 the tenant returns to *unknown*, which keeps its data forever

GET    /signy/api/v1/admin/tenants
       200 {"tenants":[{"tenant":"acme","retention":"30d",...,"updated_at":"<rfc3339>"}]}
           every pushed tenant with its policy, in name order — the control
           plane's reconciliation read
```

**The pushed policies are also the tenant registry.** Only tenants that have
a pushed policy are served at all: the `PUT` above is how a tenant is
onboarded (its requests are accepted the moment the 200 arrives), and the
`DELETE` offboards it (its reads get 403 from then on and its exports are
dropped and counted, while its data is kept).

The admin routes carry no authentication of their own. signy is not
built to be reachable from the outside network; it assumes every request —
these included — arrives through a secured channel.

Values are Prometheus-style durations (`7d`, `24h`, `90m`), the literal
`"infinite"`, or `"0"` (see [Tenant deletion](#tenant-deletion)).

**One tenant per request. There is no bulk endpoint.** A bulk write would be a
read-modify-write over shared state, and a partially applied or reordered bulk
push is exactly the failure the per-tenant shape makes impossible.

**A tenant that was never pushed has an unknown policy, and unknown means
exception.** Signy does not invent a retention value or delete physical data
for a tenant the control plane has not mentioned. When the policy registry is
enabled, the tenant is not served: its writes are dropped and its reads are
refused. The physical bytes remain for operator handling rather than being
silently interpreted as an explicit infinite policy.

An explicit `"infinite"` is an operator-selected policy and is different from
*never pushed*. They are tracked separately in metrics, so a control plane
that silently stops managing a tenant is visible as a rising
`tenant_policy_unknown_tenants` gauge rather than as invisible unbounded
storage.

### Durability

A push is acknowledged only once the policy is durable. This is the whole point
of the push shape: a `200` is a promise that the policy survives a restart, so
the control plane's retry loop terminates on a real guarantee.

- **One object per tenant**, at `<prefix>/tenant_policies/<tenant>.json`,
  holding the retention values, storage limit, monotonic revision, and access
  fence metadata.
- Project-owned writes require the next revision and use the object version for
  CAS. An exact retry of the same revision is idempotent; the ordinary operator
  endpoint cannot mutate a project-managed tenant. Access revoke advances the
  fence without changing retention values or deleting telemetry.
- With no object store configured (`SIGNY_OBJECT_STORE_URL` unset) the
  same files live under `<data_dir>/tenant_policies/`, written temp-file-then-
  rename like the rest of the local state.
- **Startup loads every object under the prefix** into the in-memory map, and a
  failed load is **fatal at boot** — the same class of failure as a manifest
  that cannot be read. Booting with a silently empty map would unclamp every
  query and hand back data a downgrade had already hidden.
- The in-memory map serves every read; the objects are the truth restored at
  boot. A push updates the map only after the write succeeds.

### Validation

The request body is untrusted input.

- The tenant id is re-validated through the existing `TenantId` allowlist
  (`tenant.rs`). A malformed id is a `400`, never propagated into a path.
- A malformed retention value is a `400`. Nothing partial can be stored, because
  one request carries one tenant.
- A well-formed value is applied exactly as sent. **Nothing caps, rounds or
  otherwise reinterprets it** —
  see [Rejected: an instance-side maximum](#rejected-an-instance-side-maximum).

The global `retention_period` this design once coexisted with is gone: the
pushed policies are the sole authority, and there is no other mode.

## Two-layer deletion

### Logical: enforced immediately at query time

Every read path clamps the requested range to the tenant's retention:

```
effective_start = max(requested_start, now - retention(tenant))
```

- Applied in `query/handlers.rs` at the entry point, so logs, metric queries,
  `label_names`, `label_values`, `series` and `stats` all inherit it.
- Trace lookup by `trace_id` has no range, so spans older than the cutoff are
  filtered by timestamp instead.
- **Unknown tenant → no clamp** (fail-open). Combined with the ingest path
  never consulting a policy, retention is entirely off the hot path.

This is what makes lazy physical deletion acceptable — the data is already
invisible to the tenant before the bytes are gone.

### Physical: lazy, and always through merge

Each retention tick walks the `TenantSegment`s of every part and compares each
segment's `max_ts_ns` against that tenant's cutoff. Timestamps stay event-time,
matching the original `max_ts_ns < cutoff` semantics (`retention.rs:88`).

| Part state | Action | Cost |
|---|---|---|
| every segment expired | delete the part whole — the existing path, unchanged | free |
| some segments expired, part is a merge candidate anyway | merge drops those rows while rewriting | no extra I/O |
| some segments expired, merge would not pick it up | it becomes a group of one, so merge rewrites it | one rewrite |

**Retention never writes a part.** For the third case `merge_once`
(`merge/scheduler.rs:57-62`) admits the part as a valid group of one. Merge
already reads rows and re-sorts them, so dropping expired rows is a filter
applied to rows it has already loaded.

This keeps **one commit path** for part replacement. Merge's transaction
(`merge/transaction.rs`), its tombstone, and its manifest CAS are reused
unchanged, and no second crash-safety story has to be written or reviewed.

Cache invalidation also disappears as a problem: a rewritten part gets a new id
and the old one is unregistered, which is exactly the lifecycle merge already
drives.

To avoid rewriting a large part to reclaim a handful of rows, the third case
triggers only once the expired fraction *reaches*
`SIGNY_RETENTION_REWRITE_THRESHOLD` (expired rows ÷ part rows, from the
tenant index — again, no download; the comparison is `>=`). Below the threshold
the rows stay on disk, invisible to queries, until the part is merged or
expires whole. A part that has grown past `merge_max_part_rows` is never merged
for size reasons, so rows below the threshold in a large part can survive
indefinitely. That is the accepted price of laziness, and the reason
job-tracked deletion was rejected.

A group of one can also turn out to be unreadable — a part above
`merge_max_memory_bytes` fails `read_all_rows_with_limit` on every tick, and no
retry changes that because the inputs never change. Such a group is counted in
`retention_rewrite_skipped` and skipped, **not** reported as a merge error:
holding `merge_healthy` low would take `/ready` to 503 over reclamation that
correctness never depended on, while the rows in question are already invisible
to queries. A group that reaches `merge_min_part_count` is work that has to
happen either way, so a read failure there still fails the tick.

### Cost

With three distinct retention values in play, a day partition is rewritten
about twice before being dropped whole, so total bytes written roughly triples.
Against the `MULTI_TENANCY_DESIGN.md` budget — storage around $0.0035 per
project per month, Class A well under that — this is immaterial at the scale
the plan targets. It should be re-checked if retention tiers proliferate.

## Tenant deletion

**Deleting a tenant is `retention: "0"`.** There is no separate purge endpoint,
and no job to track.

Zero retention puts the cutoff at *now*, so:

- every query for that tenant returns empty from the next request onward, logs
  and traces alike;
- parts holding only that tenant are deleted whole on the next retention tick;
- shared **log** parts drop the tenant's rows the next time merge rewrites them.

Because reclamation is best-effort, the last point would otherwise carry no
bound at all — rows can survive in a large part indefinitely. So that the word
*deletion* means something, **a tenant at zero retention ignores
`retention_rewrite_threshold`**: any log part still holding rows for it is
eligible for rewrite regardless of the expired fraction. That turns "never" into
"the next few merge ticks" without introducing job tracking.

### What zero retention does not do

Two limits, stated here because *deletion* is the operation whose expectations
run furthest ahead of what this design delivers.

**Shared trace parts are not rewritten, so the spans stay.** There is no trace
merge (`Cutoffs::holds_zero_retention_rows` has exactly one caller,
`merge/selection.rs`), so the bound above has no vehicle on the trace side. A
trace part is deleted only once *every* tenant in it has expired — so if it also
holds a tenant that is unknown or infinite, the deleted tenant's spans stay
indefinitely. They are invisible to every Tempo handler from the next request
onward, but the bytes are there. Closing this needs either a trace rewrite path
or trace parts that are not shared across tenants; neither is built.

**Ingest is not blocked.** The write path deliberately never consults a policy —
that is what keeps retention off the hot path — so a deleted tenant that keeps
sending is still accepted and still journalled. Its data is invisible
immediately and reclaimed by the next tick or two, but "deleted" describes the
data on disk, not the tenant's ability to write. Refusing the writes is a
control-plane or gateway concern.

Neither limit is a compliance-grade guarantee, and signy never reports that
the last byte is gone; see the rejected design above for what that would cost.

`DELETE` on the policy endpoint is **not** tenant deletion. It returns the
tenant to *unknown*, which keeps the data forever.

## Traces

Trace parts carry the same tenant segments (`MULTI_TENANCY_DESIGN.md` status
table: "Trace part format — done"). Everything above applies symmetrically via
`TraceRegistry`; the retention loop already handles both registries
(`retention.rs:99-112`).

## Configuration

| Variable | Default | Meaning |
|---|---|---|
| `SIGNY_RETENTION_REWRITE_THRESHOLD` | 0.5 | Expired-row fraction that forces a rewrite. Ignored for tenants at zero retention. |

Checked in `config.rs` `validate` alongside the other retention settings.

There is deliberately **no setting that bounds a pushed value.** The control
plane is the sole authority on retention; see
[Rejected: an instance-side maximum](#rejected-an-instance-side-maximum).

Removed with the polling loop: `SIGNY_TENANT_POLICY_URL`,
`..._INTERVAL`, `..._TIMEOUT`, `..._AUTH_HEADER`, `..._MAX_BYTES`. The
direct `reqwest` dependency went with them: the only HTTP client left in the
process is the one `object_store` uses to reach the bucket.
`SIGNY_MAX_TENANT_RETENTION` is gone too, for a different reason.

## Metrics

Following `metrics.rs` conventions (monotonic counters plus the gauges added in
M7):

- `tenant_policy_push_accepted`, `tenant_policy_push_rejected`,
  `tenant_policy_push_persist_errors`. `push_rejected` counts only requests
  that meant to change a policy and were refused for being malformed, so a bad
  `GET` never makes the control plane look like it is pushing garbage.
- `tenant_policy_known_tenants` (gauge), `tenant_policy_infinite_tenants`
  (gauge), `tenant_policy_unknown_tenants` (gauge — tenants with data but no
  policy). "With data" includes both memtables, not only the parts: a tenant
  that has just started pushing owns no `meta.json` segment until its first
  flush, which is exactly when a control-plane omission is newest.
- `tenant_policy_last_push_age_seconds` (gauge — age of the newest policy on
  this pod). It gates nothing; it tells an operator whether the control plane
  is still talking to this instance. It is only meaningful once a policy
  exists: with an empty map there is no newest push and it reads zero, so an
  alert on it must be paired with `known_tenants > 0`. The "never pushed at
  all" state belongs to `unknown_tenants`, not to this gauge.
- `retention_expired_rows_dropped` (every expired row a merge dropped, whatever
  selected the group), `retention_parts_rewritten` (inputs of groups **only**
  retention would have selected — the extra I/O retention is paying for, which
  is why a size-driven merge that happens to drop expired rows is not counted
  here), `retention_rewrite_skipped` (retention-only merge groups that could not
  be read; a number that keeps rising means a part is permanently too large for
  `merge_max_memory_bytes`)
- `merge_inputs_changed` (merge groups abandoned because another writer replaced
  their inputs first — normally retention retiring an expired part). Benign on
  its own: nothing was written and the next tick sees the new state. It is
  counted rather than only logged because a number that keeps rising while
  `merge_success` makes no progress is the one way to see the registry failing to
  converge on the manifest.
- `signy_merge_debt_parts` counts both kinds of pending merge work, ordinary
  groups and retention-forced ones, so retention backlog is visible before it
  turns into `retention_rewrite_skipped`.

`/metrics` stays operator-facing and process-wide; no tenant labels here (see
`MULTI_TENANCY_DESIGN.md`, "Implementation status").

## Failure modes

| Situation | Result |
|---|---|
| Per-tenant retention disabled | The old global behaviour, unchanged. |
| No tenant has ever been pushed | Nothing is deleted; queries unclamped. |
| Store write fails during a push | `503`, nothing applied, nothing changed. The control plane retries. |
| Policy load fails at boot | Fatal; the process does not start. Same as an unreadable manifest. |
| Tenant never pushed | Not served when the policy registry is enabled; physical data is not automatically deleted or treated as explicit infinite retention. Visible via `tenant_policy_unknown_tenants`. |
| Control plane stops pushing | The last pushed policies stay in force forever. See [Accepted risks](#accepted-risks). |
| Retention shortened (downgrade) | Query clamp effective on the next request; bytes reclaimed at the next merge. |
| Tenant deleted (`retention: "0"`) | Invisible immediately. Log bytes reclaimed within a few merge ticks; spans in a shared trace part may stay indefinitely, and ingest keeps being accepted. See [What zero retention does not do](#what-zero-retention-does-not-do). |
| Retention retires a part while merge is publishing it | The manifest CAS refuses the replacement, merge discards its own output and retries next tick. Counted in `merge_inputs_changed`; neither the store nor merge is marked unhealthy. |
| Retention lengthened (upgrade) | Effective immediately for everything still on disk. |
| Data already deleted before an upgrade | Gone. Retention lengthening cannot resurrect bytes — documented product behaviour. |
| Upgrade push delayed or lost | Data keeps being deleted at the old, shorter retention meanwhile. See [Accepted risks](#accepted-risks). |

## Accepted risks

**A delayed upgrade can cost data.** signy keeps applying the last policy
it was told. If a tenant is upgraded from `3d` to `30d` and the push does not
land — signy unreachable, the store failing the write, the control plane
backing off — then for as long as that lasts, retention and merge keep deleting
at `3d`. Data crossing the old cutoff during the window is gone, and the
upgrade cannot bring it back.

**This is accepted, not mitigated.** The exposure is bounded by the control
plane's retry rather than by an outage, because the control plane receives an
explicit failure and owns the retry: seconds to minutes in practice, and the
loss is the slice of data that crosses the old cutoff during that window. This
is the specific improvement over polling, where the same window was the entire
length of the outage and signy could not even detect it.

The opposite direction is safe. A delayed *downgrade* costs storage only, and a
policy that never arrives is never invented — an unknown tenant keeps
everything.

If this ever needs closing: have the control plane re-push unchanged policies on
a heartbeat, then stop physical deletion when the newest policy is older than
some multiple of that heartbeat. It is deliberately not built now, because
without a heartbeat "no push" cannot be distinguished from "nothing changed",
so such a gate would have nothing to measure.

## Non-goals

- **Compliance-grade proof of deletion.** `retention: "0"` deletes a tenant and
  bounds reclamation, but signy never reports that the last byte is gone.
  A guarantee would need job tracking — rejected above, with the reasons.
- **Per-tenant quotas, throttling, tenant-labelled metrics** — step 5 of
  `MULTI_TENANCY_DESIGN.md`.
- **Durable usage accounting** — step 6.
- **Tier-based partitioning** — rejected above; the status table row should be
  updated to point here.

## Migration checklist

### Already implemented, keep unchanged

- [x] `src/tenant_policy.rs` — `TenantRetention` (`Finite(Duration)` /
      `Infinite`), `Cutoffs`, duration parsing, `TenantId` re-validation.
- [x] `retention.rs` — per-tenant cutoff from segments; whole-part deletion
      keeps the existing path.
- [x] `merge/scheduler.rs`, `merge/selection.rs` — `select_groups` admits parts
      with expired rows as groups of one; expired rows are dropped after
      `read_all_rows_with_limit` and before the flush.
- [x] `query/handlers.rs` — range clamp; metric scan-start floor;
      `tempo/handlers.rs` trace lookup timestamp filter.
- [x] `AppState` — holds `Arc<TenantPolicy>` for the query handlers.
- [x] `/metrics` rendering of the retention counters and tenant gauges.
- [x] The per-tenant deletion tests in `retention.rs`, `merge/tests.rs` and
      `query/tests.rs`, and the trace-side clamp tests in `tempo/tests.rs`
      (lookup, search, both tag endpoints, and an unknown tenant on all four).

### Changed for push intake

- [x] The poller is gone from `retention::retention_loop`, along with
      `TenantPolicy::refresh`, `fetch_snapshot`, `parse_auth_header` and the
      `reqwest` dependency.
- [x] `PolicySnapshot` (one whole-map snapshot with a single `fetched_at`) is
      now `PolicyMap`, a per-tenant map where each entry carries its own
      `updated_at`. The `Arc<…>` read path is unchanged — a push copies the map,
      inserts, and swaps, which is cheap because pushes are rare and keeps query
      reads lock-light.
- [x] The three admin routes live in `src/admin.rs`, always mounted by
      `router.rs`; authentication is the secured channel's job, not theirs.
- [x] One object per tenant at `tenant_policies/<tenant>.json`, through
      `ObjectStorage` when configured and under `<data_dir>` otherwise. The
      in-memory map is updated only after the write succeeds; a failure is a
      `503`.
- [x] `TenantPolicy::load` runs in `startup.rs` before the workers spawn.
      Failure is fatal.
- [x] `config.rs` — the five polling variables are gone, and later the token
      and the global `retention_period` went too: per-tenant policy is the
      only mode.
- [x] `merge/selection.rs` — a tenant at zero retention ignores
      `retention_rewrite_threshold` (`Cutoffs::holds_zero_retention_rows`).

### Tests

- [x] a push is acknowledged only after the write lands; a failing store
      returns `503` and changes neither the map nor query behaviour
- [x] policies survive a restart, and a tenant downgraded before the restart is
      still clamped after it
- [x] a boot with an unreadable policy object fails instead of starting with an
      empty map
- [x] `DELETE` returns a tenant to unknown: data kept, queries unclamped
- [x] `retention: "0"` empties queries immediately and makes every part holding
      that tenant merge-eligible regardless of the threshold
- [x] a malformed tenant id or retention value is a `400` and stores nothing
- [x] a pushed value is reported and enforced exactly as sent, and an explicit
      `infinite` keeps exactly as much as never being pushed
- [x] a merge whose inputs another writer replaced first is skipped, not
      reported as a store failure, and `PartRegistry::replace` tolerates an
      input already retired

### Three calls the intake design left open

**Two pushes for the same tenant are serialized.** One object per tenant makes
a push a blind write with no CAS, which is what the per-tenant shape is for —
but it also means two concurrent pushes for the *same* tenant could land in the
store in one order and in the in-memory map in the other. A `tokio::sync::Mutex`
covers the write and the swap together. Pushes are rare, so the cost is nil.

**The pushed string is stored verbatim and applied verbatim.** A `GET` returns
what was sent because that is also what is in force. The instance-side maximum
that used to sit between the two is [gone](#rejected-an-instance-side-maximum).

**A push is accepted while draining.** Ingest is rejected during shutdown, but a
policy is not data this instance owns — it goes to the object store, which the
replacement instance reads at boot. Refusing it would make a machine replacement
look like a control-plane outage.

## What the implementation settled that the design left open

Six points where the deletion side had to make a call the design did not spell
out. All of them survive the move to push intake.

**A retention batch is not a replacement, and the manifest must know it.** Both
manifest writers rejected a removal set that was not intact, so that a merge
replacement could never be reapplied on top of another writer's output and
duplicate every row. Retention reuses those writers with an empty `added` set,
where that rule is not protective but actively harmful: there is no output to
duplicate, and a tick that removed some of its ids and then failed before
retiring them from the registry leaves the *next* tick holding a batch that
mixes already-removed ids with newly expired ones. The intact-set check turned
that into a permanent failure — the batch could only grow, so no later tick
could recover, and `retention_healthy` stayed low until a restart. A pure
removal is now idempotent per id and deletes whatever is still present
(`object_io.rs` `publish`, `catalog.rs` `remove_trace_parts`), while the
non-empty-`added` case keeps the strict rule that merge depends on.

Retention also retires each side's descriptors as soon as *that* side's
manifest write lands, rather than after both. A part that is absent from the
manifest but still advertised by the registry is only serviceable until the
orphan collector's grace period passes and its objects are deleted, so the
window where a failure can produce one is kept to a single manifest operation.

**Merge's commit point is the publish, and the two writers now overlap on
purpose.** `select_groups` deliberately picks the parts retention cares about,
so retention retiring a part while merge is mid-replacement stops being an
accident and becomes routine. Two consequences had to be handled.

A merge that finds its inputs already replaced is not a failure. The CAS refuses
the replacement precisely so two outputs cannot both survive, and nothing was
written — the same benign situation merge already detects *before* publishing
and skips with a debug line. Treating the later detection as a store error
marked the object store unhealthy and dropped `merge_healthy`, taking `/ready`
to 503 for a tick over a race that cost nothing. It is now skipped and counted
in `merge_inputs_changed`.

And past a successful publish the replacement is committed: the manifest lists
the output and no longer lists the inputs. Re-checking the registry there and
discarding the output on a miss — which is what the code did, duplicating the
pre-publish check — left the merged part in the manifest but in no registry
while a stale input stayed registered and *absent* from the manifest, which is
exactly the unserviceable state the paragraph above works to avoid. The check now
runs only while the transaction is still abandonable. `PartRegistry::replace`
already ignores ids that are gone, so reconciling to the manifest is the
one-line path.

**Retention does not mark; merge decides.** The design has retention flag a
part as merge-eligible. The code has `merge_once` derive that itself from the
same policy state (`merge/selection.rs` `select_groups`). Same outcome —
retention still never writes a part, and the rewrite still goes through merge's
one commit path — with no shared mutable set for two workers to race over.
`select_groups` also admits parts above `merge_max_part_rows`, which
`group_for_merge` never considers at all and which would otherwise never
reclaim anything. That opens one path the old candidate filter made
impossible — such a part can be too large to materialize — so a group that
exists only for retention carries a flag and a read failure on it is counted
and skipped rather than failing the tick. See
[Physical: lazy, and always through merge](#physical-lazy-and-always-through-merge).

**The rewrite trigger is coarser than the drop.** Eligibility is computed from
whole tenant segments (`expired_log_row_fraction`), so it stays a pure
`meta.json` read with no download. The drop applied during the rewrite is
per row against that row's tenant cutoff, which reclaims strictly more. The
threshold is a trigger heuristic; the filter is exact.

**Trace parts are never rewritten.** There is no trace merge, so the third row
of the physical-deletion table has no vehicle on the trace side: a trace part
is deleted whole when every tenant in it has expired, and otherwise waits.
Partially expired trace spans are still invisible — every Tempo handler applies
the floor, in one of two shapes. `trace_by_id` and the two tag endpoints have
no range, so they drop spans one by one; `search` clamps its window, and since
a trace is placed by its own earliest span, a trace that *began* below the
floor leaves the results whole rather than appearing shortened. That is the
range semantics Tempo already has, and it errs toward hiding. All four are
covered in `tempo/tests.rs`. `TraceTenantSegment` also carries no timestamps,
so a segment's bound is taken from the `row_group_max_ts` entries it owns.

This is also the reason `retention: "0"` cannot bound reclamation on the trace
side, which the deletion section states outright rather than leaving to be
inferred from here — see
[What zero retention does not do](#what-zero-retention-does-not-do).

**Metadata endpoints clamp at part granularity.** `labels`, `label_values`,
`series` and `index_stats` have no range to clamp, so they take the floor
directly. MemTable entries are filtered per entry; parts are pruned per part
through their tenant segments. A part that straddles the floor therefore
still contributes all of its sampled attribute values.

**A fully clamped-away range returns empty, not an error.** When the whole
requested window is older than the tenant's retention, the clamped start passes
the end. The range is validated against the request the client actually made,
then clamped, and an empty `streams`/`matrix`/`vector` result is returned — a
downgrade must not turn previously valid queries into `400`s.

## Deletion requests (`/loki/api/v1/delete`)

Retention answers "how long may this tenant's data live." A deletion request
answers a different question — "remove these lines, now" — and the two share
one mechanism at the bottom and nothing above it.

**Two obligations at two speeds.** The lines must stop being readable when the
request is accepted, because a deletion request is usually somebody exercising
a right and an hour's delay is not an answer. The bytes must also actually go,
and that cannot happen at the same moment: parts are immutable, and rewriting
one to drop a few rows is the most expensive operation this engine has.

So a request does both, each at the speed it can go:

| | when | how |
|---|---|---|
| Stops being readable | on acceptance | a per-row mask applied at the single scan every read path funnels through |
| Bytes removed | at the next rewrite touching the part | the same per-row predicate, inside `rewrite_group` next to the retention cutoff |

**The mask lives at one place on purpose.** `run_unified_query_with_stats` is
where logs, metric queries, tail, volume, detected fields and the restore probe
all meet their rows. A second place to apply the mask would be a second place to
forget it, and forgetting it means serving a line somebody was told was gone.
It is applied *before* the pipeline runs, because a delete selector matches the
line as it was written and `line_format` would have rewritten it.

**Durable before visible.** The request is written to the object store — one
object per request, under `delete_requests/`, outside the prefixes
`garbage_collect_orphans` sweeps — before it hides anything, and it is loaded at
startup on the same fatal-on-failure terms as the tenant policies. A request
that is masking rows but would not survive a restart is the one failure this
must not have.

**A covered part is worth rewriting on its own.** Merge admits it the same way
it admits a part holding rows for a tenant at zero retention, and for the same
reason stated there: otherwise a part too large for any ordinary group keeps the
rows until retention deletes the whole part, and "deleted" would be describing
the mask rather than a removal. The test is made from `meta.json` — tenant
segments overlapping the window — so it costs a map lookup rather than a read.

**`status` is conservative.** A request is promoted from `received` to
`processed` only when no part's metadata could still hold a row it covers, and
only the tenant segment's time span can rule a part out. The status lags; it
never claims removal that has not happened.

**Refused: pipeline stages in the selector.** Matchers and line filters name
rows that exist. A stage names a value derived from them, and "the lines whose
parsed `status` was 500" would mean something different whenever the parser
changed. A deletion that cannot be honoured consistently is refused rather than
accepted and half-applied.

**Not offered: Loki's cancellation grace period.** There, a request sits
unapplied for 24 hours so it can be withdrawn — during which the data is still
being served. Here it is hidden immediately and can be withdrawn until the
rewrite lands, which is the same affordance without that window.

**The limit is a scan cost, not a storage cost.** Each outstanding request is a
predicate every scan for that tenant evaluates per row, which is why a tenant
holds at most `MAX_DELETE_REQUESTS_PER_TENANT` of them. When no request exists
anywhere the scan pays a single atomic load, which is the normal case.
