# Production readiness review (2026-07-26, fresh context)

Reviewed revision: `32dd7b6` (multi-tenancy step 1 + retention push complete, working tree clean)
Review scope: entire `src/` (28,300 LOC), `docs/`, and deployment assets
Validation: all 274 `cargo test` tests passed, with zero `cargo clippy --all-targets` warnings

The previous review is [`PRODUCTION_READINESS_REVIEW.md`](PRODUCTION_READINESS_REVIEW.md) (revision `56afbbe`).
This document rereads the repository from scratch, including the multi-tenancy and retention code added since
then. It continues the previous review's item numbers (P0-1 and so on) and prefixes new findings with `N`.

> **Update (after fixes):** The diagnoses below reflect the review point (`32dd7b6`). P0-1, P0-2, and N1
> in Gate 1 were fixed immediately after this review. See each item heading and the "Change history" section.

## Verdict

**Not production-ready at the time of review.** P0-1 and P0-2, which the previous review designated as
Gate 1 (data safety), remained open, and the newly added multi-tenancy/retention code combined with P1-8
(merge-limit unit mismatch) to **break even the physical guarantee of tenant deletion.**

That verdict is this date's and stays as written. For where the gates actually
stand, read ["Production-readiness gates"](#production-readiness-gates-updated)
below, audited against the code on 2026-08-12: Gates 1 and 4 are closed, Gate 2
is closed but for one open *decision*, Gate 3 holds two partials and one
deliberate withdrawal, and Gate 5 is feature backlog tracked in `todo.md`.

Changes since the previous review:

| Item | Previous | Current |
|---|---|---|
| P0-1 WAL compaction wedge | Open | **Open** (reproduced in this review) |
| P0-2 ingest backpressure | Open | **Open** (no 429 path in the code) |
| P0-3 multi-tenancy not implemented | Open | **Identification, isolation, and retention closed**, throttles/quotas open |
| P1-1 Tempo search restores everything | Open | **Fixed since** — the window reaches the pin set and the row-group selection (see N6) |
| P1-4 writer fencing | Open | **Fixed since** — manifest `writer_epoch` |
| P1-8 merge-limit unit mismatch | Open | **Fixed since** — `materialized_bytes`, plus the split fallback (see N1) |

## Reproduced in this review

P0-1 was confirmed by execution rather than code inspection. A consecutive-compaction test was temporarily
added to `src/journal/tests.rs`, observed, and reverted.

```
first_offset=57 wal_after_first=0 second_offset=49 wal_after_second=49
result=Err(InvalidInput, "WAL compaction checkpoint moved backwards")
```

---

## Gate 1 — deployment-blocking conditions (open)

### P0-1. WAL compaction wedge — later **fixed**

- Location: `src/journal/compaction.rs`, `src/journal/replay.rs:37`; wedge propagation in `src/flush.rs:51-84`

`compact_wal` ends by leaving a state file with `write_compaction_state(&state_path, &state, 2)` and
**does not delete it.** Compaction truncates the WAL and resets checkpoint to 0, so the next offset is in
a new coordinate system, but the next call compares it with a stale offset from the old system.

| Next offset | Result |
|---|---|
| `< state.offset` | `Err("WAL compaction checkpoint moved backwards")` → permanent wedge |
| `== state.offset` | Silent no-op; WAL is not truncated |
| `> state.offset` | Happens to work |

It cannot recover by restart because of `if state.phase != 1 { return Ok(()) }` at `replay.rs:37`.
The wedge propagation path was also confirmed: `pending_checkpoint` in `flush.rs` remains `Some` forever and
`continue` runs every tick, **blocking entry into a new flush** while ingest continues returning `204`.

### P0-2. No ingest backpressure — later **fixed**

- Location: `src/ingest.rs:114` (`push_inner`), `src/trace_ingest.rs`, `src/config.rs`

`rg "429|TOO_MANY_REQUESTS"` found no 429 path anywhere in the code. `push_inner` checks only draining,
body size, labels, lines, and timestamps; it does not check MemTable/WAL backlog and has no corresponding knobs.
Combined with P0-1, "failure → guaranteed OOM" follows directly.

### P1-4. No single-writer enforcement (open)

`src/object_storage/catalog.rs` has no lease/fencing token. Two processes with the same prefix both believe
they can operate normally. M6 hardware replacement reaches exactly this state if the new instance starts
before the old one is fully dead.

---

## N — new findings from this review

### N1. Tenant deletion (`retention: "0"`) does not guarantee physical deletion

Two paths cause this simultaneously.

**(a) Large parts are never rewritten.**
`src/merge/scheduler.rs:150-160` counts a read failure for a retention-only group as
`retention_rewrite_skipped` and continues. Comments justify this as "the input is fixed, so the next tick
will not fit either; it is only a missed optimization," but **in the zero-retention path it is not an
optimization; it is the deletion itself.**

Using the defaults:

| Value | Default |
|---|---|
| `merge_max_part_rows` | 4,000,000 |
| `merge_max_memory_bytes` | 1 GiB |
| Budget per row | About 268 B (including `size_of::<Row>()`) |

Unless lines are short, rewriting a full-size part is structurally impossible, and deleted-tenant rows
remain forever on disk and S3. The previous review classified P1-8 as a merge-performance problem; with
tenant deletion now present, it also means **deletion requests cannot be handled.**

**(b) Deletion depends on one runtime flag.**
If `SIGNY_TENANT_POLICY_TOKEN` is missing, `TenantPolicy::load` (`tenant_policy.rs:419`) returns
`disabled()`. Then every `query_floor_ns` becomes `None`, and **all deleted data that was hidden without
being rewritten becomes queryable again.** The admin route also disappears (`router.rs:35`), leaving no
surface for an operator to notice. A restart with one missing environment variable resurrects data.

**Remediation**
- Split and retry a group when rewriting fails; split a single part by row-group range and rewrite it into
  multiple output parts. Promote zero-retention failures from `retention_rewrite_skipped` to errors.
- Refuse startup when any policy is stored but the token is missing.

### N2. No tenant allowlist

`TenantId::parse` (`src/tenant.rs:18`) checks only `[a-zA-Z0-9_-]{1,64}`. Without allowlist validation,
anyone who can send the header can create unlimited tenants. `todo.md` marks this complete, but the code
does not implement it. N3 and N4 amplify the result.

### N3. Row groups are forcibly split at tenant boundaries

`row_group_bounds` in `src/part/format.rs:210` ends a row group whenever the tenant changes. This is
correct, but **tenant count becomes a lower bound for row-group count.** In the target workload with many
small tenants, one flush containing 500 tenants produces 500 five-row groups; Parquet column-chunk metadata
and per-group bloom filters scale with tenant count, and compression collapses. This must be considered
alongside the 1 MiB default `flush_max_bytes`.

### N4. `/metrics` traverses every part on every scrape

`tenant_policy_gauges` (`src/query/handlers.rs:587`) scans tenant segments in every part, while
`merge_debt_part_count` in the same handler calls `select_groups` → `estimated_part_bytes` → `fs::metadata`
for every part. This is O(parts × tenants) work on an unauthenticated endpoint and is even heavier than
the previous review's P2-7 finding.

### N5. The part on-disk format has no version field and its upgrade path is asymmetric

`MetaFile` (`src/part/metadata.rs:99`) gained `tenants: Vec<TenantSegment>` without `#[serde(default)]`
and has no `version` field. Parts from an existing deployment fail `meta.json` deserialization and panic at
`startup.rs:132`.

In contrast, the WAL explicitly handles pre-tenancy records as the default tenant at `replay.rs:145-153`.
**The journal is designed for lossless upgrades, but parts cannot be read at all.** This is not currently
visible because there is no data, but every future schema change hits the same wall without a format version field.

### N6. Tempo metadata paths improved, but have no time pruning

`pin_all_trace_parts` changed to `tenant_part_ids(tenant)`, narrowing it to the tenant range (partial P1-1
improvement). But `search` computes start/end and passes `scan_trace_spans(..., None, ...)`, then **filters
after scanning** (`tempo/handlers.rs:113-130`), while `search_tags`/`search_tag_values` have no time
parameters. One Grafana tag-dropdown opening still restores all traces for that tenant.

### N7. A single failed request marked the object store down — **fixed**

- Location: `RemoteCache` health state (`src/object_storage/catalog.rs`)
- Verification: **Reproduced and fixed**, `docs/LOAD_RESULTS.md` section 6

`/ready` reads `remote_healthy`, and one failed object-store request set it.
At a 3% injected write-error rate — which the engine survives with no ingest
errors and no lost data — the flag flipped 14-17 times a minute and read false
34-59% of the time. That is an instance an orchestrator pulls in and out of
service over an error rate that cost nothing.

The first explanation recorded here was wrong and is left in
[`LOAD_RESULTS.md`](LOAD_RESULTS.md) rather than quietly replaced: it blamed the
epoch guard in `mark_remote_healthy_since`, predicting that recovery degrades as
reporters multiply. Sampling the flag through a run refutes that — the
configuration with *fewer* reporters has the *worse* duty cycle. Both flapped,
and the PASS/FAIL difference between them was which side of a signal changing
every few seconds the terminal sample happened to land on.

**Fixed**: health is hysteretic. Three consecutive failures with no success
between them mark the store down; one success clears it. Callers report the
outcome of the operation they finished instead of capturing an epoch first,
which removed the guard along with its call-site ceremony. What is given up is
that a slow success predating an outage now resets the count, delaying detection
by one more round of failures — bounded, where the guard's failure mode was not.
`signy_remote_consecutive_failures` exposes the pressure the flag now hides
below its threshold. Measured after: 99.3-100% healthy, 0-2 transitions.

### N8. Merge and flush contend — **measured, and it is bounded**

- Verification: **Measured**, `docs/LOAD_RESULTS.md` section 6

With merge enabled the WAL backlog runs several times higher than with it
disabled. Sampled every 500 ms over ten minutes it oscillates between 6 and
140 MB with a linear trend of -0.04 MB/s and a second-half mean below the first
half's. Merge and flush contend and flush catches up, so this is a tuning note
rather than a defect: the contention costs backlog depth, not divergence.

It also produced the third instance in this investigation of a terminal sample
meaning nothing — the peak was 140 MB while the run ended at 47.6 MB. The
harness now samples the backlog and gates on whether flush ever drains it,
against the engine's own `max_wal_backlog_bytes` rather than a number the
harness invented.

What is not settled is the depth itself. 140 MB of unflushed WAL is 140 MB that
a simultaneous loss of machine and disk would take, and that window widens with
merge running. Whether that is acceptable is a retention-of-risk question for
the deployment, not a defect in the engine.

### N9. Peak RSS is concurrent live memory — **explained, not a defect**

- Verification: **Measured**, `docs/LOAD_RESULTS.md` section 7

Peak RSS is 173-187 MB with merge off and 697-758 MB with merge on, and cutting
`merge_max_memory_bytes` eightfold barely moved it. Two hypotheses were recorded
and both are wrong: it is not the merge budget, and it is not allocator
high-water retention either.

Watching RSS through a run and for 90 seconds after settles it. The peak is
853.6 MB; the final reading is 14.8 MB, back to the starting value, while merge
kept running through the idle tail. The memory is live, held while ingest, flush
and merge overlap, and the allocator returns it. The budget knob did not move it
because the groups being merged were far smaller than the budget — with parts of
a few megabytes, `merge_max_input_bytes` was never binding.

**Consequence is a sizing rule.** Peak is roughly 50x idle and is reached within
a minute of load starting, so an instance sized from its idle footprint is sized
about fifty times too small. [`RUNBOOK.md`](RUNBOOK.md) says so.

---

## Still open from the previous review

Only status is confirmed here; see the previous document for details.

| Item | Status |
|---|---|
| P1-2 OTLP log ingest unimplemented | **Fixed** — `LogsService` on the gRPC listener, plus `POST /v1/logs` and `/v1/traces` on the HTTP one. Both transports share one admission and normalization path |
| P1-3 group commit consumes batch timer | **Fixed** — the batch loop no longer waits out `max_batch_ms` on an empty channel, and the default is now zero linger |
| P1-5 MemTable O(rows) size calculation + flush deep clone | **Fixed** — size in O(1) (with P0-2); the snapshot is now shared with the flush through an `Arc` instead of copied |
| P1-9 eviction holds write lock during synchronous directory traversal | **Fixed** — the work runs in `spawn_blocking` with the guard moved into it, and it is now driven by the registry's part directories rather than by two levels of `read_dir` over the whole tree |
| P1-10 object-store startup errors panic | **Fixed** — retried within `SIGNY_STARTUP_RETRY_BUDGET`. The remaining `panic!` sites are the deliberate give-up past that budget |
| P1-11 startup/flush cost linear in part count | **Partly fixed** — restore overlaps its downloads and reconcile no longer re-reads the manifest per merge group. The manifest is still rewritten in full per flush |
| P2-1 Loki/Tempo API gaps | **Mostly fixed** — `tail`, `index/volume(_range)`, `detected_labels`, `detected_fields`, `format_query`, JSON push, Tempo v2 tags and `/api/echo`. `patterns` is a read-time miner over a bounded sample, and the `delete` API hides on acceptance and removes at the next rewrite |
| P2-2 resource guards on metadata endpoints | **Fixed** — semaphore, timeout, `start`/`end`, and `match[]` count limits |
| P2-5 duplicates after a crash unobservable | **Fixed** — startup reports replayed records and entries, as a WARN and as `signy_wal_replayed_entries`. Removing them is fixed too: every part is written through one sort, which drops entries identical in tenant, stream, timestamp, line and metadata. The replay counters remain the upper bound on what a restart introduced before the next merge collapses it |
| P2-7 `/metrics` has no histograms or labels | **Partly fixed** — latency histograms are there, so p95/p99 is derivable. Endpoint labels are still absent |
| P2-8 stdin abort ineffective in containers | **Fixed** — `SIGUSR1` abandons a stuck force-flush and exits non-zero. stdin is kept for interactive use |
| P3 deployment assets | **Fixed** — Dockerfile, [`CONFIGURATION.md`](CONFIGURATION.md), [`RUNBOOK.md`](RUNBOOK.md) |
| P2 real S3 validation | **Confirmed out of scope** — local MinIO is the limit ([`LOAD_VALIDATION.md`](LOAD_VALIDATION.md)) |

---

## Strong areas (in the newly added code)

- **Retention push design.** The code records the rationale for "ack after storage"
  (`tenant_policy.rs:497-513`), fatal startup-load failures, the distinction *unknown tenant* ≠ *infinite*,
  and preventing push-age reversal by recording `updated_at` under `write_lock`. Removing polling and the
  `reqwest` dependency also leaves object storage as the only outbound call.
- **Admin authentication.** Not mounting routes when the token is unset, constant-time `secret_matches`,
  a 4 KiB body limit, and separating `push_rejected` from `admin_unauthorized` correctly distinguish
  "is the control plane broken?" from "who is knocking?"
- **Tenant isolation is applied across read paths.** `query_floor_ns` covers every log/metric/labels/
  label_values/series/index_stats/Tempo handler, and splitting shared-part row groups at tenant boundaries
  guarantees isolation at the index level.
- **Retention reuses merge transactions, tombstones, and manifest CAS** instead of writing its own parts,
  a good choice that does not expand the crash-safety surface.
- Changing log deletion to **unregister immediately without waiting for trace deletion failure**
  (`retention.rs:255-262`) is correct.

---

## Change history

Immediately after this review, a batch of Gate 1 items was fixed. 285 tests passed, with zero clippy warnings.

| Item | Status | Summary |
|---|---|---|
| P0-1 WAL compaction wedge | Fixed | Remove the intent record durably immediately after success. Phase 2 is represented by record absence; surviving phase-2 records are treated as complete and removed, so already-wedged instances recover automatically after upgrade |
| P0-2 ingest backpressure | Fixed | Track MemTable/WAL backlog in O(1); above the limit, return `429` + `Retry-After` before journal append (`RESOURCE_EXHAUSTED` for OTLP) |
| N1(a) merge split fallback | Fixed | Split groups in half and rewrite single parts in row-group windows. Tenant deletion no longer permanently skips large parts |
| N1(b) startup without policy token | Fixed | Refuse startup when any policy is stored but the token is missing |
| N5 format version field | Fixed | Add `version` to part/trace-part `meta.json`. Check before checksum validation so format differences are not treated as corruption |
| P1-8 merge-limit units | Fixed | Add `materialized_bytes` to part metadata. Group selection and read budgets use the same unit, and `validate` enforces limit ordering |
| N4 `/metrics` O(parts) | Fixed | The merge worker publishes merge debt and the retention worker publishes unknown tenants. Scrapes only read |
| N2 tenant allowlist | Fixed | `SIGNY_ALLOWED_TENANTS`; tenants outside the list receive 403 |
| N6 Tempo time pruning | **fixed** | `search`, `search_tags` and `search_tag_values` all take `start`/`end`, and the window reaches the pin set and the row-group selection. `search` now matches a trace on span overlap rather than on its earliest span, which is both Tempo's rule and the one the row-group bounds can answer |
| P2-2 metadata guards | Fixed | Semaphore, timeout, `start`/`end`, and `match[]` count limits |
| P1-4 writer fencing | Fixed | `writer_epoch` in the manifest; claim at startup, verify on every CAS, self-fence and terminate on fencing |

### P0-1 — fix details

`compact_wal` removes the intent record at the end of the success path and fsyncs the parent directory.
The record no longer exists in the normal state, so the next compaction cannot compare offsets from different
coordinate systems. Only two kinds of records can remain, each with one meaning.

| Record | Meaning | Handling |
|---|---|---|
| Phase 1 + tmp exists | Crash before rename | Restore old checkpoint and retry (unchanged) |
| Phase 1 + tmp missing | Crash after rename | Treat as complete and remove the record |
| Phase 2 | Leftover from an old build | Treat as complete and remove the record |

`replay.rs` follows the same rules, so restart recovery also works. Tests cover consecutive compactions
(shrinking, equal, and growing cases), compaction and replay recovery for an old-build phase-2 leftover,
and caller retry after intent-removal failure. Crash injection is armed per WAL path — a process-global flag
could be consumed first by an unrelated parallel test.

### P0-2 — fix details

`MemTable`/`TraceMemTable` maintain total bytes with atomic counters (half of P1-5). Stream identifiers are
double-counted when buffers merge, so `merge_snapshot` returns the duplicate amount and subtracts it from
the counter; the counter then matches a full traversal **exactly** (fixed by tests). `Journal` holds
`wal_bytes`/`checkpoint_bytes`, making backlog O(1) — `/metrics` no longer runs `stat` + checkpoint-file
reads on every scrape.

`IngestGate` is the single decision point for both protocols. Blocking only one would move excess traffic
to the other. New knobs are `SIGNY_MAX_MEMTABLE_BYTES` (256 MiB by default),
`SIGNY_MAX_WAL_BACKLOG_BYTES` (1 GiB by default), and `SIGNY_BACKPRESSURE_RETRY_AFTER` (1 s by
default); the first two can be disabled with `off`. `config.validate` rejects a MemTable limit below
`flush_max_bytes`, which would reject data before asking flush to move it.

New metrics: `signy_ingest_throttled_total`, `signy_memtable_buffered_bytes`.

### N1(a) — fix details

`rewrite_group` alternates reading, filtering, and writing within one blocking task. Batching would consume
the memory the split was intended to save. Above the limit, a multi-part group is recursively split in half,
while a single part is traversed in row-group windows with `PartReader::read_rows_in_row_groups`. Window size
is calculated from the part's actual average row width. Regardless of output-part count, all outputs share
the same merge tombstone (`old_dirs`), so the commit remains one transaction. Outputs already written are
removed in place if an intermediate step fails.

`retention_rewrite_skipped` now counts only cases that cannot work even after splitting — effectively a
configuration where a single row group exceeds the budget. Retention-only groups also use `meta.json` to
filter already-reclaimed groups before reading them.

### N5/P1-8 — fix details

Check `version` in `meta.json` **before** checksum validation. Checksums are calculated over the structure,
so they matter only after both sides agree on the structure; reversing the order makes a format change look
like a checksum mismatch — a disk failure rather than a version difference. The manifest already had
`format_version`.

P1-8 is solved by recording `materialized_bytes` (memory actually occupied when read) in part metadata.
Centralizing calculation in `Row::materialized_bytes` prevents group selection and read budgets from diverging,
and removing `fs::metadata` from `estimated_part_bytes` also lightens N4.

### N2/P2-2 — fix details

Tenants outside the allowlist receive **403**. It is not 400 because the request itself is valid and the
client has nothing to fix. If the allowlist is enabled without the default tenant, every headerless request
would create an out-of-list tenant, so `validate` rejects the configuration.

The four metadata endpoints acquire `MetadataGuard`. The retention floor is folded into `start_ns`, expressing
"the range requested by the client" and "the range the tenant is still entitled to see" as one boundary
(`MetadataWindow`). Without `start`, the query goes back only `max_query_range`, not through all history —
the infinite default was what caused every part to be read in the first place. An empty range is an empty response,
not an error.

### P1-4 — fix details

`writer_epoch` is in the manifest rather than a separate object because it costs nothing to check. Every
write already reads the manifest it will replace. At startup, claim the same number for both log and trace
(claiming only one would leave trace writes unfenced), then verify the epoch read by every CAS.

`ObjectStorage` directly reports fencing to `ShutdownState`. Flush, merge, retention, and force-flush respond
identically without each worker needing to understand fencing.

**Self-fencing is a decision made in this work.** Force-flush stops retrying when fenced. Every other
force-flush failure is transient and merits infinite retry, but this one does not — continuing leaves the
process alive until the orchestrator kills it, after which the pod may be scheduled on another node and the
disk containing the only copy of unflushed data is discarded. The loss M6 intended to prevent would occur
through exactly that path. Exit code 1, data remains in the WAL, and the log explicitly states disk preservation.

**Operational implication:** A new instance claims the epoch immediately at startup. Thus M6's procedure of
"fully drain the old instance before starting the new one" is now **enforced**. Violating the order makes the
old instance stop and exit abnormally instead of failing silently.

The rest remains in `todo.md`.

---

## Production-readiness gates (updated)

**Audited against the code on 2026-08-12**, item by item, because a gate list is
only worth what its last check is worth and this one had drifted in *both*
directions: five items were done and still marked open, and one marked done had
since been deliberately undone. Each line below carries where to look, so the
next audit starts from evidence rather than from this sentence. Two items are
reframed rather than answered — an open question is not the same as unfinished
work, and recording it as the latter is how a list stops being read.

**Re-audited 2026-08-18, and it had drifted again — the same way and for the
same reason.** Gate 5 was the part the 2026-08-12 pass did not reach: its three
lines carry no evidence, and two of them were already answered elsewhere *in
this document*, in the findings table above. The trigger was reaching for the
list to decide what to do next and finding it named work that was finished, so
the scope of this pass is what could have moved since `8b767a5`: Gate 5, and
every line the work since then touched.

**Gates 1 and 4 were then re-read too, the same day**, so this pass is a full
one and every line below now carries where to look. They hold — but one of them
holds by a different mechanism than its own words describe, and that mismatch
had already leaked into the runbook and out of it into the alert rules. Written
down where it happened, under N1(a).

### Gate 1 — data safety

- [x] Fix P0-1 WAL-compaction wedge + consecutive-compaction/crash-injection tests
      — `journal/tests.rs`: `compaction_failure_does_not_fence_journal_writer`,
      `compaction_retry_after_rename_failure_keeps_acknowledged_suffix`,
      `replay_rolls_back_uncommitted_compaction_before_rename`,
      `consecutive_compactions_truncate_whatever_the_batch_sizes_are`
- [x] P0-2 ingest backpressure (MemTable/WAL-backlog limit → 429) —
      `backpressure.rs`, refused before decompression so a rejected request
      costs neither CPU nor WAL bytes
- [x] N1(a) split fallback on merge memory overflow (guaranteed tenant deletion)
      — **holds, by a mechanism this line does not describe.** The fallback was
      to halve an oversized group and retry; what shipped instead is a rewrite
      that interleaves reading and writing (`merge/selection.rs`,
      `rewrite_group`), so there is no memory overflow left to fall back from at
      any part size. The guarantee is stronger and the words are stale, which
      matters because they had been copied onward: `retention_rewrite_skipped`
      still meant "a part too large to rewrite" in `RUNBOOK.md` on 2026-08-18,
      and from there into `deploy/alerts.yml` two commits earlier. Both now say
      what the counter means — a rewrite that failed on I/O or a corrupt input
- [x] N1(b) fail startup when policies are stored but the token is missing —
      `tenant_policy.rs:565`, pinned by
      `dropping_the_token_with_stored_policies_fails_the_boot`
- [x] P1-4 writer fencing (manifest epoch + self-fence) — `writer_epoch` on the
      manifest, `check_epoch` on every commit path, `FENCED_ERROR` as the one
      string a fenced writer reports
- [x] Unify units for P1-8 `merge_max_input_bytes` vs `merge_max_memory_bytes` —
      `Config::validate` (`config.rs:880`) refuses an input bound above the
      memory bound, both in bytes

### Gate 2 — multi-tenancy completion

- [x] N2 tenant allowlist
- [x] Per-tenant throttles and quotas — `tenant_quota.rs` (a token bucket per
      tenant, burst floored at `max_push_bytes` so a legal body is never refused
      forever), `default_tenant_max_streams` for `max_streams_per_user`, and the
      rate applies to both transports (`the_tenant_rate_applies_to_the_http_transport_too`)
- [x] **Tenant-labeled metrics — the decision exists and is not this list's to
      make** (closed 2026-08-18). The line asked for a decision about the trust
      boundary of `/metrics`, and that decision was taken where the alternative
      shipped: `admin.rs`'s `get_usage` says in its own doc comment that
      per-tenant numbers are on the authenticated admin API *deliberately
      rather than* as labels on `/metrics`, because that endpoint is
      unauthenticated and process-wide by design and a label per tenant
      multiplies every series by the customer count — the cardinality problem
      this engine bounds everywhere else, on a workload whose whole point is
      many small tenants. The reader that needs per-tenant numbers is the
      control plane, which is already authenticated there and already asks one
      tenant at a time. A deployment that wants them in Prometheus scrapes that
      endpoint, not this one. What follows is the reframing this replaces. The
      original line bundled this with the quotas above, which are done. What is
      not done is emitting one series per tenant, and it is not obvious it should
      be: `/metrics` is unauthenticated (`metrics.rs`, the gauge doc), so
      per-tenant series hand a scraper the tenant list and make cardinality a
      function of the customer count. The aggregate shape shipped instead —
      known / infinite / unknown tenant counts and the newest push age
      (`tenant_policy_gauges`). Closing this needs a decision about the trust
      boundary of that endpoint, not code — **still true on 2026-08-18**: no
      metric in `query/handlers.rs` carries a `tenant` label
- [x] P2-2 metadata endpoint resource guards + apply `start`/`end`
- [x] N4 remove O(parts) work from `/metrics`
- [x] Adjust the default bind to the trust boundary — the defaults are
      `127.0.0.1:3100` and `127.0.0.1:4317`, and startup says so in one line
      naming the variable that widens it (`startup.rs`, "bound to loopback only")

### Gate 3 — operability

- [ ] ~~N5 part format version field~~ **Withdrawn, and the withdrawal is the
      decision.** The field was built for this gate and then removed on
      2026-07-31 (`f0da5bd`): signy is not deployed, so there is never older
      data to read, and every compatibility path was code that could not run —
      the bloom container had carried four formats, which forced the reader to
      hold three capability flags. Recorded in `VISION.md` under "What is
      deliberately not built" and in `ARCHITECTURE.md`'s decided-choices table.
      Left visible rather than deleted, because a gate that was met and then
      unmet on purpose is exactly what a later reader will otherwise re-open
- [x] P1-10 retry transient failures at startup — `with_object_store_retry` in
      `startup.rs`, 250 ms backoff against a budget, after which the process
      exits and the orchestrator's own restart backoff takes over
- [x] **P2-7 — done** (2026-08-18). `signy_query_latency_ms` carries an
      `endpoint` label over a fixed set of six, the cardinality being a property
      of the binary rather than of what a client sends. Closing it found a
      second hole the line did not name: the metric path recorded **no latency
      at all**, so `rate`, `count_over_time` and everything `index/volume`
      reduces to were absent from the read path's published latency — which was
      therefore describing log queries while being read as describing queries.
      Both funnels observe now. What follows is the state before it.
      `LatencyHistogram` emits
      cumulative `le` buckets, which is the only shape `histogram_quantile` can
      read, and it replaced `*_latency_ns_total` counters that could only ever
      yield a mean while every target in these documents is written as p95/p99.
      What is still missing is a per-endpoint dimension: one histogram covers the
      query path as a whole
- [x] P2-8 non-stdin abort path — `SIGUSR1` forces termination during a drain
      (`shutdown.rs`), alongside the `exit`-on-stdin path a terminal has
- [x] **P3 — four of four** (2026-08-18). `deploy/alerts.yml` ships 18 Prometheus
      rules, one per row of the runbook's table, tied to it by
      `query::tests::every_alert_signal_in_the_runbook_has_a_rule`: the test
      fails on a row with no rule, on a rule naming a metric this engine does
      not export, and on a rule alerting on a signal the runbook never
      explained. Writing it caught two of the third kind. What follows is the
      state before it. `Dockerfile`, `docs/CONFIGURATION.md` (with a test,
      `every_configuration_knob_is_documented`, that fails the build when a knob
      is added without a row) and `docs/RUNBOOK.md` all exist; the runbook's
      "What to alert on" is prose, and machine-readable alert rules are not
      shipped. **Confirmed open 2026-08-18** — the repository contains no rules
      file of any format. The runbook's table is nine rows of signal, condition
      and meaning, which is the ruleset's own content: what is missing is the
      form, not the decisions

### Gate 4 — scale validation

- [x] Measure row-group fragmentation with many tenants for N3 —
      `part::tests::tenant_breadth_sets_the_row_group_floor_and_what_that_costs`
      builds parts across a tenant sweep and asserts the cost rather than
      describing it
- [x] N6 Tempo time pruning (search and both tag endpoints) — `tempo/scan.rs`
      prunes by `tenant_part_ids_in_range`, and `tempo/handlers.rs` runs every
      window through `validate_query_range` first

### Gate 5 — feature completeness

- [x] P1-2 OTLP logs — implement/register `LogsService`
- [x] P1-2 OTLP/HTTP (`/v1/logs`, `/v1/traces`), protobuf and JSON
- [x] **P2-5 duplicate observability → deduplication** — done, and done before
      this line was last read. Observability landed first (replay WARN plus
      `signy_wal_replayed_entries`); removal followed, in the one sort every
      part is written through (`part/format.rs:195`, `dedup_by` on
      `Row::sort_key`), so a chunk cut between two twins does not save one. The
      findings table above already said so
- [x] **P2-1 Loki API gaps — closed 2026-08-18, and the last one was not what
      this line implied.** Audited against the finding's own table on 2026-08-18: `tail`,
      `index/volume(_range)`, `detected_fields`, `detected_labels`,
      `format_query`, JSON push, the delete API, Tempo v2 and `/api/echo` are
      implemented; `patterns` is implemented on stated terms; `start`/`end` on
      the metadata endpoints landed with P2-2; `buildinfo` reports a real
      revision (`handlers.rs`, `build_revision()`). The last gap —
      `label_values` ignoring Loki's `query` matcher parameter — **closed the
      same day**: the selector filters the values through the `series` lookup
      both the memtable and the registry already had. One deliberate
      divergence, in the same shape as the `_extracted` one: a `query` carrying
      a **line filter** is refused rather than answered as though it were
      absent, because label values come from stream labels and no filter over
      line content narrows them without a scan this endpoint does not run
- [ ] **LogQL improvements in P1 of `todo.md`** — one line open there, and it is
      a stated divergence rather than a gap: exact-field pruning stays
      conservative for empty-string equality and `_extracted` collisions, because
      an empty equality also matches an absent field and absence is indexed
      nowhere. Feature completeness, not a deployment gate

### Where the five gates stand after 2026-08-18

Two lines are open and neither is work: N5 is a gate that was met and then unmet
on purpose, left visible so a later reader does not re-open it, and the LogQL
line is a stated divergence in feature completeness. **Nothing on this list
blocks a deployment.**

That is a statement about this list, and the list has been wrong in both
directions twice. What makes it worth more this time is that every line now
names where to look, so the next reader checks rather than believes — and two
of the four things this pass fixed were found by a line's own evidence
disagreeing with the code it pointed at.

What is *not* on this list and still true: the query-side risks in
[`LOAD_VALIDATION.md`](LOAD_VALIDATION.md) — latency tails against a real
backend, throttling, and the cost model — remain unvalidated because they need
a deployment, and no audit of a document can close them.
