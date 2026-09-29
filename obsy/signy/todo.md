# TODO

This tracks work deferred beyond M3's current scope and work for later milestones.

The complete production-readiness gate list is in [`docs/PRODUCTION_READINESS_REVIEW_2026-07-26.md`](docs/PRODUCTION_READINESS_REVIEW_2026-07-26.md)
(previous review: [`docs/PRODUCTION_READINESS_REVIEW.md`](docs/PRODUCTION_READINESS_REVIEW.md)).
The three invariants the work below serves are in [`docs/VISION.md`](docs/VISION.md).

## Open correctness defects

**Read this section before any milestone below.** These are wrong answers, not slow ones, and a milestone is
never a reason to leave one open. Each was found by a measurement rather than by review, and each is recorded
here with the measurement that found it so that fixing it can be verified the same way.

Neither is fixed at the point of discovery, on purpose: both were found by [`compare/`](compare/) while it was
being built, and a ruler that edits the thing it is measuring in the same change measures nothing. That
reason expires now — the bed is built and its baseline is published.

- [x] **`query_range` treats `end` as inclusive; Loki treats it as exclusive.**
      *Found by:* M9's row-equality check — 2 of 96 otherwise identical answers differed, always by exactly
      the row whose timestamp equals the window's `end`.
      *Confirmed:* directly against both endpoints over the same window. Both include `start`.
      *Severity:* a Loki-compatibility defect rather than a preference, because the endpoint claims Loki's
      contract. Invisible unless a boundary lands exactly on a row, which is why nothing before the
      comparison's step-aligned windows surfaced it.
      *Fixed by:* `part::QueryTimeRange`, which now owns the question "does this timestamp fall in the
      window" for the whole read path. The owner was not `parse_time_ns` — the timestamps parsed correctly;
      the boundary was spelled out four separate times below the query layer (the memtable scan, the
      part-level prune, the row-group prune and the row-level reject) and each one closed `end`. Log queries
      construct `half_open`, so `[start, end)` is decided once, by the handler. The scans whose `end` is not
      a client-supplied exclusive bound say `closed` explicitly: a metric scan's `end` is its last evaluation
      point, tail's is "now minus delay", and a merge's is "every row".
      *Verify with:* `compare/run.sh`, matrix phase — the check that found it is the end-to-end regression
      test. At unit level, three tests put a row exactly on the boundary and each fails if a single site
      drifts back: `memtable::tests::the_range_decides_whether_a_row_on_end_is_returned`,
      `part::tests::a_row_on_end_belongs_to_a_closed_window_and_not_to_a_half_open_one` (which also asserts
      row-group pruning agrees with the row-level test, since a tighter prune drops rows silently) and
      `query::tests::query_range_includes_start_and_excludes_end_in_the_memtable_and_in_parts`.
      `memtable::tests::query_includes_the_end_timestamp` had encoded the defect as the contract and is the
      first of those three now.
      *Not changed:* the metric step grid, which
      [`docs/COMPARISON.md`](docs/COMPARISON.md) records as a separate still-open difference — Loki aligns
      samples to absolute multiples of `step` and signy steps from `start`.

- [x] **`| json` does not promote extracted fields into a log response's stream labels; Loki's does.**
      *Found by:* the same run, but **not** by the equality check — its digest is over `(timestamp, line)`
      pairs, so a label-set difference is structurally invisible to it. `json_field` was reported as 24/24
      agreed. The two label sets appear in [`docs/COMPARISON.md`](docs/COMPARISON.md) only because they were
      captured alongside.
      *Measured:* signy returned 6 labels where Loki returned 22, the difference being every field the
      parser extracted.
      *Severity:* the log-query response shape, which is what Grafana's Logs panel renders as a line's
      detected fields. Metric grouping is **not** affected — `sum(count_over_time({app="api"} | json [5s])) by
      (level)` is covered by `query::tests::metric_grouping_uses_extracted_fields_and_parser_errors` and works.
      *Blocked on:* nothing. The checker is fixed first (next item) and is now **red on this defect**:
      `json_field` reports 0 of 24 agreed, and the fix is what has to turn it green.
      *Also measured while extending the checker:* signy does not drop the extracted fields, it returns
      them in the **third element of each `values` tuple** — the structured-metadata slot — where Loki returns
      them as stream labels. So the response carries the same names in the wrong place, plus one name Loki does
      not have at all (`trace_id_extracted`, signy's collision rename for a field that is also pushed
      metadata). The fix is about placement, not about extraction.
      *This and the "accepted" pushed-metadata difference were one defect, and that was settled against Loki
      rather than by argument.* `grafana/loki:3.3.2`, one stream with `level` as a stream label, `trace_id` and
      `pod_ip` pushed as structured metadata, and a line whose JSON carries `level` and `trace_id`:
      `{app="probe"}` answers with `trace_id` and `pod_ip` **among the `stream` labels** and a **two-element**
      `values` tuple, and `{app="probe"} | json` adds the extracted fields to that same set. Loki's default JSON
      encoding never uses the third element; the three-element tuple is its *opt-in*
      `X-Loki-Response-Encoding-Flags: categorize-labels` shape, and there it is an object of categories
      (`{"structuredMetadata": {…}, "parsed": {…}}`) rather than the flat map signy was returning. So the
      digest's declared exemption described the same slot as this defect, and both are fixed.
      *Loki's collision rule, measured the same way:* a `| json` field colliding with a **stream label** becomes
      `<name>_extracted` and both survive and both filter. A field colliding with a **pushed metadata key** is
      **discarded** — `| json | trace_id="<the JSON value>"` matches nothing, `| json | trace_id="<the metadata
      value>"` matches, `trace_id_extracted` does not exist, and `line_format "{{.trace_id}}"` renders the
      metadata value. That is why Loki's response had no `trace_id_extracted` and signy's did.
      *Fixed by:* `query::build_stream_data`, which now merges each row's stream labels with its post-pipeline
      field set, emits one stream per distinct merged label set and a `[timestamp, line]` tuple. It takes the
      requested `direction`, because merging two input streams into one group interleaves their rows.
      `logql::merge_extracted` implements the collision rule above; `LogQuery::process_entry_with_labels_*` now
      also passes a stream label that a `label_format` **rewrote** through to the response (Loki answers
      `| label_format level="rewritten"` with the new value), while an unchanged one is still not duplicated
      into the query-local metadata. The internal representation is unchanged — extracted fields still live on
      the query-local entry's `structured_metadata`, which is what filtering and `sum(...) by (extracted_field)`
      read (`query/tests.rs`, `sum(count_over_time(… | json [5s])) by (level)`).
      *Deliberately different:* a second collision on the same name. signy answers `foo_extracted_2` and
      keeps the `foo_extracted` stream label; Loki appends `_extracted` once and **overwrites** the
      `foo_extracted` stream label with the extracted value, losing it. signy does not lose a value it was
      given. This is the `_extracted` pruning item under "P1 — LogQL improvements", now with the measurement.
      Also unchanged: an *unaggregated* metric query's series identity. Loki promotes a row's metadata and
      extracted fields into it and signy groups by stream labels plus whatever `by`/`without` names, so
      `rate({app="api"})` answers one series per `trace_id` on Loki and one per stream here. That is an
      identity/cardinality difference rather than a misplaced field, it is why the matrix asks for
      `sum(rate(...))`, and it stays reported as a difference.
      *Verified with:* the digest, not by assertion — the same short comparison the item below documents.
      `json_field` **0 of 24 before, 24 of 24 after**, with `label_only`, `line_filter` and `rate` unchanged at
      24/24. Then the digest's placement exemption was removed, because after the fix it could only hide a
      regression back into the `values` triple, and the same comparison re-run against a fresh Loki volume:
      **96 of 96**, with `label_only` now reading `stream:trace_id`/`stream:pod_ip` on both sides where it used
      to read the placement-blind `metadata:` tag. At unit level, five tests at the response-builder level pin
      this without Loki running: `query::tests::a_log_response_promotes_pushed_metadata_into_the_stream_labels`,
      `a_log_response_promotes_json_extracted_fields_into_the_stream_labels`,
      `an_extraction_shadowed_by_pushed_metadata_never_reaches_the_response`,
      `promotion_regroups_rows_by_their_whole_label_set_and_keeps_the_direction` and
      `label_format_over_a_stream_label_reaches_the_response`, plus
      `logql::tests::json_scalar_extraction_is_ordered_and_metadata_wins_collisions` for the collision rule and
      `matrix::tests::structured_metadata_in_the_values_triple_is_a_disagreement` for the ruler.
      *Not regenerated:* [`docs/COMPARISON.md`](docs/COMPARISON.md), which is the M9 run's document. Its
      generator's prose is updated with the exemption it no longer has; the next `compare/run.sh` publishes it.

- [x] **Extend the row-equality digest to cover labels.** The finding above matters less than the blind spot
      that hid it: a checker that proves two engines agree while silently not looking at half the response is
      the kind of green light this repository has already been burned by once
      ([`docs/LOAD_RESULTS.md`](docs/LOAD_RESULTS.md), retired).
      *Fixed by:* `matrix::digest_response`, which now digests one record per returned entry holding the
      entry's timestamp, its line **and every label it carried together with where the response put it**
      (`stream:` / `entry:` / `metric:`), so the same lines under different labels are no longer the same
      answer. A metric result additionally digests one record per series identity, so an extra empty series
      cannot hide, and the result type and `status` are hashed in as an envelope. A malformed `values` tuple is
      now an error rather than a silently skipped row — the old digest read what it could and ignored the rest,
      so a response it could not parse compared equal to an empty one.
      *One accepted difference turned out to be the same defect and was withdrawn:* pushed structured-metadata
      keys were digested **without** their placement, on the grounds that Loki promotes them into result
      identity while signy returned them in the `values` triple. That is the same slot and the same
      sentence as the `| json` defect above, so it was fixed instead, and the exemption is gone — nothing is
      exempt from placement now, because after the fix the exemption could only hide a regression. Their
      *values* were and are still compared per entry, so `label_only` checks per-row `trace_id`/`pod_ip` that
      nothing checked before. What remains exempt is by name, not by placement: Loki's derived `detected_level`
      and `service_name` are dropped, and every answer records which dropped names it carried so the document
      states the exemption with its counts instead of the digest being quietly narrower than it claims. The
      metric step grid stays handled by `align_to_step`, i.e. by the query rather than by the digest.
      *Reported as:* the disagreement table gains a labels-differ column, and the label difference itself is
      printed grouped by difference — "24 queries: only signy `entry:level`…, only Loki `stream:level`…"
      — instead of reaching the document by accident.
      *Also now checked:* response order against the requested `direction` (kept out of the digest, which must
      stay order-independent, and reported as its own fact plus a distrust item). *Deliberately not digested:*
      `data.stats`, which reports how much each engine had to read (16,384 lines against Loki's 1,251 for the
      same answer) and is the thing they are supposed to differ on; a log response's grouping into streams,
      because both now return one stream per metadata combination but neither promises the same partition of
      the same rows; and `limit` truncation, which no window in the matrix reaches (`limit` 20,000 against
      ~6,250 rows).
      *Verified by:* seed + matrix phases only, against Loki 3.3.2 in `compare/docker-compose.yml` and a local
      signy, 30,000 rows over 8 streams and 3 windows — the shortest configuration that surfaces
      `json_field`, about two seconds per phase instead of `compare/run.sh`'s 25 minutes.
      `label_only` 24/24, `line_filter` 24/24, `rate` 24/24, **`json_field` 0/24** at the point the checker was
      fixed; 96/96 once the defect above was. The unit tests that pin each rule are
      `matrix::tests::the_same_lines_under_different_labels_are_not_the_same_answer`,
      `structured_metadata_in_the_values_triple_is_a_disagreement`,
      `json_extracted_fields_in_the_wrong_place_are_a_disagreement` and
      `every_label_is_digested_where_the_response_put_it`.
      *Re-seeding Loki is not idempotent, and a comparison rerun has to start from a fresh volume.* Loki drops
      an exactly duplicate entry from a **log** answer but still counts it in `rate`, so a second `seed` against
      the same volume leaves `label_only`/`line_filter`/`json_field` agreeing and doubles every `rate` sample.
      That was hit while verifying the fix above and is a property of the bed, not of either engine.
      *Not regenerated:* [`docs/COMPARISON.md`](docs/COMPARISON.md), which is the M9 run's document and
      describes the digest that run used. The next `compare/run.sh` publishes the extended one; regenerating it
      from the old artifacts would print the new prose over old digests.

- [x] **A bounded scan over a multi-stream row group dropped the newest rows.**
      *Found by:* the three-way bed's strict row-equality check, live — `{app="checkout"}` with
      `direction=backward`, `limit=100`: Loki returned the window's newest hundred rows and signy
      returned a hundred rows from the *middle* of the window. Two of 24 `label_only` queries and the same
      two `(app, window)` pairs under `json_field`, wherever the part layout happened to align.
      *Cause:* `scan_batch` answered a row beyond the sink's frontier with `ScanStep::StopGroup`, on the
      stated premise that "a row group is one stream's run, ordered by time". It is not: `row_group_bounds`
      cuts per tenant × size only, so a group holds several whole streams, each ordered inside itself — the
      backward walk fed the sink one stream's tail, the frontier tightened on it, and the rest of the group,
      including other streams' newer rows, was skipped. Forward had the same defect: it returned the first
      stream's oldest rows instead of the oldest rows.
      *Fixed by:* deleting `StopGroup` — a frontier crossing now rejects the row, never the group; the only
      group-level skip is `span_beyond_frontier` over `meta.json`'s recorded span, which is sound regardless
      of interleaving. The backward doubling-window walk from the group's end is gone with it, because
      windowing from the end is only exact when the end is the newest row; a backward group is decoded once
      and offered newest-batch-first.
      *Verify with:* `part::tests::a_limited_scan_over_interleaved_streams_returns_what_truncation_would` —
      two time-interleaved streams in one group, both directions, asserted equal to unlimited-scan-then-
      truncate; red before the fix ([0,2,4,6,8] where truncation gives [0,1,2,3,4]). End to end: the bed's
      strict agreement, 24/24 on every shape after the fix.
      *Cost, measured and accepted:* `benches/query.rs` backward improved everywhere (`line_filter` −56% to
      4.81 ms, `json_field` −15% to 12.43 ms — better than the pre-layout 13.07 ms the backward-regression
      item wanted back) and **forward regressed +108–139%** (`label_only` 1.65 → 3.93 ms), because forward's
      early group exit was the same unsound assumption. Reading less than a whole group needs the format to
      record that the group is time-ordered; that flag rides Phase 2's meta change, and projection pushdown
      shrinks what "whole group" costs. A slower right answer over a faster wrong one is not a trade this
      repository debates.

## M14 — the metrics engine (issue #8)

**Built, measured, and the first comparison lost.** The plan is
[`docs/M14_IMPLEMENTATION_PLAN.md`](docs/M14_IMPLEMENTATION_PLAN.md); execution state is the phase list
in [`docs/PROJECT_PLAN.md`](docs/PROJECT_PLAN.md); the numbers are
[`docs/COMPARISON_METRICS.md`](docs/COMPARISON_METRICS.md) and the claim they answer is in
[`docs/VISION.md`](docs/VISION.md). The engine ingests all five OTLP metric types, stores them as
Gorilla-encoded float series behind the shared journal/part/offload lifecycle, contains a cardinality
burst by refusing new series with a named `partial_success`, and answers seven first-party routes. What
it has not done is win: see the two open items below.

Deferred items this plan minted, so they are not re-litigated mid-build:

- [ ] **Compactor tier constants are constants, not knobs** (8 parts per tier, ~16 MiB / ~256 MiB
      promotion), until a load run says otherwise. If the bed's object-store counts or query fan-in blame
      the tiers, that measurement is the reason to promote them to configuration — not before.
- [x] **Exponential histograms are downscaled to ≤ 64 finite boundaries at ingest** (decided with the
      user, 2026-08-26). Lossy and irreversible for stored data, and it stays that way: what changed on
      2026-09-01 is that the cap no longer bounds *cardinality*. A histogram is stored as one series
      carrying its bucket vector, with `_bucket{le=}`, `_sum` and `_count` synthesized on read, so
      sixty-four boundaries are a schema inside one identity rather than sixty-seven of them. Issue #12.
      Native *resolution* — keeping the exponential representation rather than boundary-limited
      buckets — remains the recorded future work if fn0 ever needs tighter tail quantiles.
- [ ] **OTLP exemplars are dropped** at the decomposition, documented in `QUERY_API.md` when Phase 7
      writes it.
- [ ] **Retention stays per-tenant, not per-signal.** Metrics reuse the tenant's single period via a
      `metric_part_fully_expired` predicate; a per-signal period is future work with no current consumer.
- [ ] **`rate`/`increase` are the VictoriaMetrics definition** (positive-delta sum, no extrapolation —
      decided with the user, 2026-08-26). Revisiting this means revisiting the bed's exact-agreement
      digests, so it does not happen casually.

- [x] **The metrics memtable could count its way to an outage, and the first published bed run is how it
      was found.** During the churn phase a series whose samples were mid-flush was evicted by the idle
      sweep; the flush's abort then re-created the entry *without* its state bytes, so the next eviction
      released what had never been reserved and the `u64` wrapped. The ingest gate read
      `memtable holds 18446744073708471276 bytes, over the limit of 128849018` and refused every push
      from then on — which is how the run's seed phase landed 44 992 of 106 560 datapoints and made that
      run's query columns a comparison of nothing. Fixed twice over: the abort brings the state bytes
      back with the entry, and every release clamps at zero, because a gauge that is a little wrong is a
      bug to find while a gauge at `u64::MAX` is an outage. Pinned by
      `an_abort_that_recreates_an_evicted_series_keeps_the_accounting_sound` and
      `releasing_more_bytes_than_were_taken_clamps_at_zero`.

- [x] **The churn axis is won, and what bound it was never memory.** Measured 2026-08-27, both engines
      at a 2 GiB container, 520 288 series offered: signy refused 24 288 datapoints (five whole-request
      429s) at its `max_active_series` default of 500 000 while peaking at 1 037.8 MiB, and
      VictoriaMetrics accepted every one of them at 627.3 MiB. The competitor held *more* series in
      *less* memory without refusing anything, and the two things that followed were both structural.

      Closed 2026-09-01, recorded in [`docs/METRIC_CAPACITY_SWEEP.md`](docs/METRIC_CAPACITY_SWEEP.md).
      Normal mode now takes **eight million series with zero refusals at 215 MiB** — a fifth of the
      memory for fifteen times the series — and refuses with named 429s and stays alive past that. The
      probe path reaches 28 million at 1 443 MiB where VictoriaMetrics OOM-killed at 16.8 million. So
      the claim's churn half no longer rests on "refuses rather than dies" against an engine that did
      neither: this one holds more *and* refuses rather than dying.

      Three changes did it, each gated by its own probe and each with a loss published beside it. A
      flushed gauge or cumulative series is **retired from the active index** the moment its samples are
      durable, because the only reason it stayed was `last_ts` and that reason ends at a part boundary.
      The **part catalog is mapped** rather than read: `index.bin` became a fixed-stride row array with
      the labels in a `labels.bin` beside it, a selector tests a row's bytes in place, and what was
      anonymous memory the kernel could only OOM on became page cache it can reclaim — the 20-million
      normal-mode run touched the cgroup limit exactly and survived, 890 MiB anonymous against 1 098 MiB
      of file. And **`SERIES_OVERHEAD_BYTES` is gone**: the calibration this item asked for turned out
      not to be a better constant but no constant at all, since the maps can be asked their own
      `capacity()` and a label allocation is arithmetic.

      Two losses are in the document rather than only here: bounding the weak label pool cost 2.3 million
      series and 1 063 timed-out requests before the sweep was paced by attempts instead of insertions,
      and the mapping trades memory for time — 28 million series ingest in five times the wall clock of
      ten million, with push p99 at 1.4 s. That trade is the one VictoriaMetrics makes too.

- [ ] **What binds the metric path now is the memtable's *share*, not any per-series cost.** Refusal
      begins around 10.7 million offered where the probe path reaches 28 million with the guards off, so
      the remaining factor is 2.6x rather than the 35x a stale constant was worth. That gap is
      `max_memtable_bytes` = a quarter of the declared budget, shared by the log, trace and metric
      memtables with no per-signal floor — which is also how one tenant's cardinality burst can push
      another tenant's log ingest into a 429. Raising the share, giving each signal a floor, or leaving
      it alone are all defensible; none of them is a measurement, so this is a decision to make rather
      than a number to find. Note what the ceiling now bounds: a flushed series is retired, so it is the
      flush window's working set and not the tenant's cardinality, and refusal is therefore a
      throughput boundary — the same 20 million offered more slowly is accepted whole.

- [ ] **Two query-side reductions are known and unmeasured.** The selection walk does not consult the
      bloom — only `pin_metric_parts` does — so every part whose time range overlaps has its whole row
      array read whether or not it can hold the metric; and the rows are label-sorted, so an exact
      `__name__` could binary search rather than scan. Both matter more now that the mapping made time
      the limiter rather than memory.

- [x] **Native histogram storage, issue #12, done 2026-09-01.** One exponential histogram cost up to 67
      series (`bounds + 3`, downscaled to 64 finite boundaries) — 13% of a `$1` project's whole
      500-series allowance for one instrument. It is one series now, and the names it used to be are
      synthesized on read so the API contract and the VictoriaMetrics comparison are untouched.

      Two things fell out that were not the point. The rescale churn is gone: a widening range used to
      change every boundary and therefore mint a fresh set of 67 identities, abandoning the old ones to
      the catalogs for the whole retention window, and is now a new run inside the same series' chunk.
      And the sample cap `MAX_OTLP_METRIC_SAMPLES` now bounds datapoints rather than resolution.

      The encoding is the part that could have gone wrong. Counts written plainly are eight bytes a
      bucket a point — at 64 buckets, five times what the Gorilla fan-out paid — so a chunk delta-codes
      each bucket against the point before it and writes the boundary schema once per run. Measured at
      **79 bytes a point** for a 64-bucket histogram over a hundred scrapes.

- [x] **The value disagreement is diagnosed, and it is two unrelated things.** The published run's
      per-answer records settled it (`docs/artifacts/m14/metric-matrix_*.json`, 2026-08-27); every
      disagreement falls in one of two buckets and neither is a wrong answer.

      *One ULP, on the shapes held to the `exact` rule.* `raw_range` (19/24 answers) and `agg_sum_by`
      (3/3) differ by at most `2.99e-16` — one or two representable doubles, on a shape that performs no
      arithmetic at all. Measured directly against a VictoriaMetrics container: pushed
      `250.07999999999998` (`c2f5285c8f426f40`), read back `250.08` (`c3f5285c8f426f40`), one ULP apart.
      VictoriaMetrics does not promise to return the bits it was given — it stores a decimal
      approximation — while this engine's Gorilla XOR is bit-exact by construction
      (`value_bit_patterns_round_trip_exactly`). So the **`exact` digest class is unsatisfiable for any
      pair containing VictoriaMetrics**, which is precisely the lesson the log bed already learned about
      VictoriaLogs and the strict digest: the basis has to be something both systems can produce. The
      fix belongs to the bed, not the engine — move `raw_range` and `agg_sum_by` to `tolerance`, and say
      in the document why the strongest available rule is not bit-equality.

      *One sixth, at exactly one instant.* `rate_range`, `instant_alert` and `churned_selector` differ
      only in the **last window's last step** — 8 of 24 answers, every one of them `w2`, one timestamp
      each, `L/V = 0.8333` flat. The dataset's last sample sits at `anchor + 3590 s` while the window's
      end is `anchor + 3600 s`, so a 60 s range holds 50 s of samples: this engine answers the five
      deltas that arrived, VictoriaMetrics scales them to the full window and answers `60/50` as much.
      `churned_selector`'s `2/3` is the same mechanism over a sparser series. **The M14 decision record
      and `QUERY_API.md` were wrong to say VictoriaMetrics "does not extrapolate"** — it does not
      extrapolate past the data, but it does scale a partially-covered window, and the document has been
      corrected to what was measured.

- [x] **Decided: a partially-covered window does not scale, and the bed stops asking about one.**
      Two changes, 2026-08-27. The digest classes are now named for the two *sources* of difference —
      `stored` (1e-12 relative, licensing VictoriaMetrics' decimal storage against this engine's
      bit-exact Gorilla) and `computed` (0.5%, licensing each engine's own window arithmetic) — and an
      unrecognized class falls to the tight rule, because a checker that reaches for its widest
      tolerance on a label it does not know can only fail by silently agreeing. And the rate semantics
      stay as they are: the two engines agreed at 40 of the 41 steps in the disagreeing answer, and the
      one that differed was the only step whose window reached past the last sample. That is where a
      target has stopped reporting, and scaling there would keep an alert firing on traffic the dead
      target is no longer producing. The bed now clamps every evaluation point to the last sample
      (`no_evaluation_point_sits_past_the_last_sample`), which is the move `matrix.rs` already makes for
      the log `rate` shape: a difference in edge conventions is not a difference in engines, and it
      belongs in `QUERY_API.md` rather than in a table.

- [x] **`churned_selector`'s remaining disagreement is the semantic difference itself, and it is now a
      declared exemption.** After the matrix stopped asking past the last sample, five of six shapes
      agreed and this one did not: 36 records, at 6 of its 118 steps, every one within `range` of a
      churn generation's end — the instants where a series stops reporting, which is precisely what the
      shape exists to cross. The series set it was built to check agreed on all 16. So the boundary
      instants' *values* go uncompared by name, derived from the workload's generation boundaries rather
      than from which records happened to differ, with the count printed beside the result — the same
      treatment `DERIVED_LABELS` gets on the log side. Pinned by
      `only_the_churn_shape_exempts_instants_and_only_at_generation_ends`; a comparison that decided what
      to skip by looking at what disagreed would license every difference it found.

- [ ] **The latency verdict is decided by noise, and the bed cannot settle it as built.** With the value
      disagreements resolved the ratios finally print — and two consecutive runs of *identical* query
      code read `rate_range` at 1.03x and then 1.62x, either side of the 1.1x threshold the verdict
      turns on. The per-query spread says why: signy's cold `rate_range` ranges 0.31–0.55 ms
      against VictoriaMetrics' 0.25–1.20 ms, so a single query's ratio spans 0.26x–2.15x. Every number
      is a sub-millisecond HTTP exchange where the request floor is a large share of the measurement.
      The document now says so in both the verdict and the distrust section rather than presenting a
      coin flip as a result. Settling it needs one of: a matrix large enough to average the floor out, a
      corpus where the work dominates the round trip, or a measurement taken inside the engine rather
      than across a socket — and until then no latency conclusion should be drawn in either direction.
      What the run *does* establish is that all six shapes now agree, which is what makes any of this
      measurable at all.

  *Superseded, kept for the reasoning it was decided against:* Two changes
      fall out of the diagnosis above and neither should be made blind. The digest class for
      `raw_range`/`agg_sum_by` has to move to `tolerance` or the bed will withhold those ratios forever
      over a difference no engine can avoid. And the rate semantics is a product decision the way the
      original VM-versus-Prometheus choice was: answering what arrived is honest and answers a partial
      window low, while scaling matches what a VictoriaMetrics user's dashboard shows at the trailing
      edge. Whichever is chosen, `QUERY_API.md`, the decision record and the bed's tolerance all move
      together — and the shapes agree everywhere a window is fully covered, so no live dashboard is
      affected either way.

Found while building the metrics bed, and not a metrics defect: **the log bed's next rerun would have
403'd at the seed phase.** The env tenant allowlist was removed after the last published run — the
pushed policy is now the registry and an instance with no pushed tenants serves nobody — and nothing
onboarded `verify-tenant-000` through the admin API. **Fixed in `6c58ab0`**: the harness onboards its
own tenants in all three phases (`run_load`, `run_verify`, `run_metric_verify`) and fails the run if a
push does not take, so every script that drives it inherits the call.

Worth keeping in view now that a tenant refusal on a write is a **drop rather than a 403**: had this
been found after that change instead of before it, the bed would not have failed at all. It would have
seeded into a void — every push answered `200`, nothing stored, and every query returning empty
against a corpus the report says was written. The onboarding failure is now the only thing standing
between that and a published comparison, which is why it exits the run rather than warning.

## M8 — the ruler (precondition for everything after it)

No optimization starts before this. Every performance number currently in the repository was produced by a
harness that cannot reach its own offered rate, on data with cardinality 1 and near-zero entropy, with reads
that never contend with writes. Optimizing against those numbers reproduces them.

- [x] **Retire the numbers, keep the reasoning.** Both documents carry a retirement header and no number in
      them may be cited until the rewritten harness regenerates it. What survives is what never depended on the
      magnitudes: three refuted hypotheses kept rather than replaced, the terminal-sample lesson, §8's finding
      that `tokio::time::interval` fires immediately — so every "merge disabled" run had one merge in it — now
      written as explicit Invalidated markers on the sections it killed, and the structural object-store counts,
      which are properties of the code rather than of the harness. Three citations were wrong and are now
      recorded as wrong: a named test that never existed, an artifact that was never checked in, and a surviving
      artifact that disagrees with the document citing it on both build and verdict. That artifact is kept, with
      a header, because it is the only checked-in evidence *for* the retirement
- [x] **`benches/` with criterion** — six targets over WAL append, memtable insert and query,
      `rows_from_snapshot`, bloom build and lookup, part write and scan, and LogQL parse and evaluation. The
      corpus is seeded and clock-free with a cardinality knob, and prints its measured compression ratio every
      run so a drift back toward the retired harness's 31.5x is visible immediately. The row and part benches
      carry a counting global allocator, because time is the wrong instrument for invariants I and II: it
      already reads two allocations per label per row at `part/mod.rs:302`, which is the number `Arc<Labels>`
      has to move — and did: 11/17/27 allocations per row at 2/5/10 labels became 6.00 at all three, so the
      table's job is now to keep it flat rather than to watch it grow
- [x] **Rewrite `src/bin/load.rs`** (now `src/bin/load/`): N keep-alive connections per workload over a
      hand-rolled HTTP/1.1 client on tokio (no new dependency — the server binary keeps object storage as its
      only outbound call); latency taken from the *intended* send, with service time and response time both
      reported and their gap called out; the corpus is `signy::corpus`, promoted out of `benches/` so
      there is one generator; out-of-order and late arrival are workload knobs; queries are an independent
      workload with their own rate and connections; percentiles carry their sample count and refuse to be
      computed below it; RSS is `VmHWM` from `/proc/<server pid>/status` with a sampled `VmRSS` series, and a
      run that cannot read it fails; the WAL gate is a trend, not `trough <= peak * 0.5`
- [x] **Delete `src/bin/m5_load.rs`** — deleted. It measured `std::process::id()`, the harness's own RSS
- [x] **The Dockerfile builds on a different compiler than CI gates** — fixed in M9, because the comparison
      bed needs an image. `rust-toolchain.toml` is now in the dependency-cache `COPY`, so the toolchain
      download happens once in the cached layer and the shipped binary is built by the compiler CI gates.
      Building it for the first time found two further defects that meant **the image had not been buildable
      at all** since the benches landed: `Cargo.toml` declares six `[[bench]]` targets and cargo refuses to
      *parse* a manifest whose declared targets are missing, so `benches/` is now copied too (nothing builds
      them; they only have to exist); and the dependency-cache stub wrote only `src/main.rs` while
      `Cargo.toml` declares a `[lib]`, so the layer failed outright — and once it is stubbed, `touch
      src/main.rs` alone leaves the empty library's fingerprint valid, so both targets are touched
- [x] **Fix `scripts/run_load_local.sh`** — the repository root is derived from the script's own location, the
      addresses and the result path are defaults rather than assignments, and the harness's non-zero verdict
      exit no longer truncates the server log
- [x] **CI.** [`../.github/workflows/signy.yml`](../.github/workflows/signy.yml), on every push and every pull request:
      `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (the tree is at zero warnings and that is
      a maintained property), `cargo test`, `cargo bench --no-run` so `benches/` — a separate crate nothing else
      compiles — cannot rot, a criterion `--test` pass that executes every bench body once, and a twenty-second
      debug run of [`scripts/run_load_local.sh`](scripts/run_load_local.sh). Neither the bench pass nor the load
      run is a measurement: a shared runner's timings are noise, and this repository has already published
      numbers from a machine that was busy doing something else. The load run is gated on the *presence* of a
      verdict, not its value. The toolchain is pinned in [`rust-toolchain.toml`](rust-toolchain.toml) rather than
      in the workflow, so the version that gates a pull request is the one `cargo` picks locally
- [ ] **Bench regression *detection*, not just bench execution.** CI proves the bench bodies run; it compares
      nothing. Criterion baselines live in `target/criterion/` and do not survive a fresh runner, so an honest
      gate needs a stored baseline — publish the estimates as an artifact, restore the previous one, and compare
      against a threshold wide enough to survive shared hardware, or run the comparison on a machine that is not
      shared. Half of this is worse than none: "regression check: passed" computed over noise is a claim nobody
      will re-examine

## M9 — the comparison bed

**Done, and the claim lost.** The bed is [`compare/`](compare/); the published table is
[`docs/COMPARISON.md`](docs/COMPARISON.md), regenerated from `target/compare/*.json` by
`compare/run.sh` rather than written. A full run is about twenty-five minutes.

- [x] **Loki single-binary beside signy under docker-compose** — `compare/docker-compose.yml`, identical
      `mem_limit` and `memswap_limit`, both on local filesystem storage, one isolated volume each. signy
      runs **local-only** so that, like Loki's filesystem backend, it keeps one durable copy of a part rather
      than a local one and a remote one. `compare/loki-config.yaml` documents every deviation from Loki's
      defaults beside the reason, and the run captures Loki's own `GET /config` diffed against
      `GET /config?mode=defaults` so the published deviations are Loki's report and not this repository's claim
- [x] **Harness target support** — `SIGNY_LOAD_TARGET=signy|loki` and no new dependency. The same
      corpus, seed, offered rate, connections and wire bytes go to both, because the push endpoint is the same
      endpoint. What differs is the metric vocabulary, the behavioural gates (Loki is gated on
      `loki_discarded_samples_total` being zero — a discard is this bed misconfiguring Loki, not a Loki result)
      and the memory source, which is now cgroup v2 `memory.peak` when `SIGNY_LOAD_CGROUP` is set
- [x] **A dataset both systems provably hold.** A paced load run sends what it managed to send at wall-clock
      timestamps, so two runs of it are two different datasets and nothing row-level can be compared between
      them. `SIGNY_LOAD_PHASE=seed` pushes a fixed corpus at fixed log timestamps from an anchor both runs
      are given; `matrix` then times the four shapes over it cold and warm and digests every answer
- [x] **Four query shapes plus ingest throughput and bytes on disk per GB ingested** — the table is in
      [`docs/COMPARISON.md`](docs/COMPARISON.md). At 8 GiB per container, 1.2 M events at an offered 20 k eps
      and zero errors on both sides: signy wins `label_only` (0.36x), loses `|=` (1.69x), loses
      `| json | field=` (1.49x cold, 1.44x warm) and loses `sum(rate())` (7.1x); ingest 16.8 k eps against
      19.9 k; settled data 323 MiB/GB against 267 MiB/GB
- [x] **Row equality: 94 of 96 queries returned identical rows.** The two that did not are the finding below
- [ ] **Object-store request counts — deferred, with the reason published.** On a filesystem backend the two
      counters do not count the same thing: signy's `signy_object_store_operations_total` counts
      calls into the `object_store` crate and reads zero with no store configured, and Loki's filesystem chunk
      client emits no request counter at all. Making the axis real means putting **both** on MinIO, which
      changes both storage paths end to end and needs its own settling and validation — a different experiment,
      not an extra column
- [x] **Publish the table, including when it loses** — it loses

### What the bed found

- **Two correctness defects**, both promoted to "Open correctness defects" at the top of this file so a
      completed milestone's log is not where they live: `query_range` treats `end` as inclusive where Loki
      treats it as exclusive, and `| json` does not promote extracted fields into a log response's stream
      labels. The second is the more useful finding, because the row-equality check **did not catch it** — the
      digest is over `(timestamp, line)` pairs, so `json_field` was reported as 24/24 agreed while the two
      responses carried 6 labels and 22
- [x] **signy was OOM-killed at a 2 GiB container limit where Loki was not**, ingesting 1.2 M events at
      20 k eps with the harness's query workload on. `memory.peak` climbed monotonically from 2 MB to the limit
      in forty seconds while `signy_memtable_bytes` reported 111 MB, so the accounted memtable is not
      where it went. **Answered by [`docs/MEMORY_ATTRIBUTION.md`](docs/MEMORY_ATTRIBUTION.md)**: 44% of the
      anonymous peak is allocator-retained free memory, and the largest live terms are one merge group
      (771 MiB) and the flush's whole-snapshot `Vec<Row>` (721 MiB). The query path is implicated through the
      allocation traffic it generates rather than through what it holds. The bed sweeps limits and reports the
      sweep, so the published run is at 8 GiB and says so
- [x] **In local-only mode the WAL is never compacted.** `flush.rs:219` passes `remote_cache.is_some()` as the
      `compact` flag, so without an object store the checkpoint offset advances and `journal.wal` keeps every
      byte ever ingested, uncompressed. It was 541 MiB against 143 MiB of parts — 79% of the disk footprint.
      Either local-only should compact too, or the mode should be documented as not for retention

      Local-only compacts too now (`0f24a97`, 2026-08-03): the prefix is cut when it outgrows both the live
      suffix and a 64 MiB floor (`SIGNY_WAL_COMPACT_MIN_BYTES`, `off` restores this item's behaviour).
      Measured: `data_dir` 1137 → ~240 MB steady on the 2 GiB rig, push latency unchanged. The W-series
      section below has the run-by-run record.
- [x] **cgroup `memory.peak` includes the cgroup's own page cache**, so it is not a footprint on its own; both
      systems write a WAL and then large data files. The harness now samples `anon` out of `memory.stat` and
      reports the anonymous high-water mark beside the cgroup peak, because the anonymous figure is what an OOM
      kill is decided on
- [x] **Loki promotes structured metadata into metric identity.** A bare `rate({app="x"}[1m])` returns one
      series per `trace_id` on Loki and one per stream on signy, so the matrix uses `sum(rate(...))`:
      otherwise the two are neither doing the same work nor producing comparable answers
- [x] **Loki puts metric samples on a grid aligned to absolute multiples of `step`** and will emit a point past
      the requested `end` to stay on it; signy steps from `start`. The matrix aligns its window boundaries
      so this is a no-op, because otherwise a row-equality failure would be about the bed's choice of window

## Next — the gate stops one phase too early

Found 2026-07-30 by trying to regenerate [`docs/COMPARISON.md`](docs/COMPARISON.md) at `5f1e9a2`. **The
comparison did not complete and no new verdict exists**; the published document is still the M9 run's.
Details and the timeline are in [`docs/MEMORY_BUDGET_GATE.md`](docs/MEMORY_BUDGET_GATE.md).

The engine was OOM-killed in its 2 GiB container **fifteen seconds after the last row was accepted**, in the
idle settle, while merge consolidated what ingest and the seed had left behind. The memory gate passes at
2 GiB because its workload *ends when ingest ends* — so the merge backlog load leaves behind is outside every
number that document reports. This blocks the arena work the same way building the gate blocked the fixes:
there is no point sizing arenas against a measurement that stops before the largest term runs.

- [x] **Extend the memory gate through a settle with merge active**, and gate on the peak across it, not on
      the peak of accepting load. The current pass means "survives ingest", which is not "fits a container"
      — **done, and it moved the answer.** `--settle` defaults to 150 s and the verdict reports which phase
      the peak fell in (`src/bin/memory_gate.rs:483`, `:550`). Extended, `df6d65b` was `OOM_KILLED` at 2 GiB
      having peaked 28.6 s *after* the last row was accepted; the honest floor is now
      "1792 MiB dies, 2 GiB lives at 89%" ([`docs/MEMORY_BUDGET_GATE.md`](docs/MEMORY_BUDGET_GATE.md))
- [x] **`merge_max_memory_bytes` must derive from the declared budget** — **done**:
      `derive_defaults_from_budget` (`src/config.rs:465`) takes 25% of the declared budget with a 64 MiB
      floor, and the budget itself defaults to 60% of the detected cgroup limit. An explicit env knob still
      overrides its derived value, because the derivation runs before the environment is read. The text
      below is what it was written against. It defaults to 1 GiB — *half* of a
      2 GiB container — from no number the operator gave, and
      [`docs/MEMORY_ATTRIBUTION.md`](docs/MEMORY_ATTRIBUTION.md) already measured one merge group's rewrite as
      the **largest single live term at 771 MiB**. That measurement predicted this kill and nothing acted on it
- [x] **Query latency under ingest has no gate.** In the load phase — queries concurrent with 20 k eps — p95
      was 22.9 s today at 2 GiB, and 5.7 s at 2 GiB / 27.2 s at 8 GiB in the M9 artifacts. The published
      comparison never showed it: its query columns come from the matrix phase, which is one connection over a
      small dataset with no ingest running. Three axes are now measured and two of them are ungated

      Gated now, twice over: the stall fix took load-phase response p95 to 98.5 ms in the bed (the 2 s
      numeric target passes inside the load verdict), and `compare/run.sh` fails unless signy's load
      verdict is PASS (`eeae4a2`, `COMPARE_REQUIRE_PASS`).
- [x] Re-run `compare/run.sh` — **done 2026-08-18** at `e79e78c`, and the document is republished from it.
      All three survived 2 GiB, agreement held at 168 of 168, and the claim's shape got faster: `metadata_rare`
      cold 0.23 → 0.21 ms, `label_only` cold 26.3 → 17.7, `json_field` cold 11.5 → 10.0, ingest 19767 → 19933
      eps — with Loki unchanged at 19869 and VictoriaLogs at 19930 → 19932, which is what makes the move the
      engine's rather than the machine's. Two rows went the other way and are published as they came out:
      `memory.peak` during queries 159.4 → 278.6 MiB, and 10.5 MiB of write-ahead log still on disk at the
      settle point where the last run had drained to 0. Neither is attributable from this run — the query row
      is `memory.peak`, which includes page cache, and no anonymous number is taken for that phase

## Done — the merge streams

Landed in `6ee46c4` (k-way merge read), `869bac5` and `d362eb8` (streaming part writer, proven byte-identical
against the batch one), and `3ca3bb8` (the switchover, retiring the split fallback).

- [x] A paged row iterator per `PartReader`, and a k-way merge across the group with the dedup `sort_rows` did
- [x] A streaming `write_part_files`. The schema, the one thing that must be known before the first row group,
      comes from the inputs' `meta.json` — checked, not assumed
- [x] **The split fallback is retired by argument and by test.** It existed so a part larger than the budget
      could still be rewritten, which is what makes zero-retention deletion actually delete (N1a). Liveness is
      now one page per input stream plus one output row group regardless of part size, so the case cannot
      arise; the test that pinned the old skip now asserts that a group is rewritten with the budget set below
      a single row
- [x] Re-run the gate. **2 GiB was `OOM_KILLED` through the settle and is now green at 89 %; 1792 MiB, red even
      on ingest alone before, is green at 95 %; 1536 MiB is still red.** The ingest and settle phase peaks are
      now equal in every passing run — the settle stopped adding anything. Delivered load unchanged at
      19.9 k eps. Numbers and caveats in [`docs/MEMORY_BUDGET_GATE.md`](docs/MEMORY_BUDGET_GATE.md)
- [x] **`peak_materialized_bytes` now overstates.** It still adds `merge_max_memory_bytes`, which no longer
      bounds a rewrite. Correcting it is a claim about memory and wants its own measurement rather than an edit
      *Closed structurally rather than by re-measuring, which is why it needed no measurement in the end:* the
      two terms it added were two pools that could both be full at once, and there is one account now. The
      number is `SIGNY_MEMORY_ACCOUNT_BYTES` itself — nothing can be held that was not charged to it — so the
      sum has no second term to overstate. See the pool removal below

## The 2 GiB soak died for four months on a copy of a file (2026-09-03)

A 15-minute soak at 2 GiB / 20 k eps was asked for and did not survive: OOM-killed at
t=211–576 s on **every** arm — mimalloc v3, jemalloc, and the commit before the memory account.
Three 900 s runs on mimalloc v3 defaults now finish it, anon peak 1450–1635 MiB against the 2 GiB
wall, 19.8–20.0 k eps delivered, zero query errors.

**The cause was the bloom cache holding a second copy of `index.bin`.** A stored bloom is a length,
three header words and a bitset laid out contiguously; decoding it copied the bitset into an owned
`Vec`, and the copy was almost exactly the file — 4418 KiB of sidecar became 4120 KiB of cache, a
factor of **1.01**. So a 122 MiB budget held about thirty parts against a working set of sixty to
two hundred, and the cache thrashed: mean residency **2.03 s**, mean time to the same part being
decoded again **0.144 s**, two thirds of installs a part coming back, each allocating four megabytes
and freeing it seconds later at two hundred a second. That is what filled the allocator's
unreturned dirty pages — 297 → 953 MiB while live memory stayed flat — and the dirty is what the
kernel could not reclaim with swap off.

`BloomIndex` holds the mapping and the offsets into it; `BloomRef` borrows the bits for the moment
of a lookup. Installs went 4120 KiB → **1.3 KiB**, re-decodes 6521 → **0**, hit rate 88.3% → **100%**,
cache resident 122 MiB → **0.6 MiB** while holding 233 parts.

### What it took to find, and every step of it was wrong first

Five hypotheses died to measurement, in order. Each is recorded because the *reason* each was wrong
is the method:

* **"The allocator."** mimalloc v3 and jemalloc OOM identically. Not an allocator choice.
* **"Millions of small allocations."** 12.76 M/s at 92 B mean — but slab occupancy went *up* through
  the explosion (84.7% → 95.2%, nonfull 18% → 6%) and slab growth matched live growth to within
  4 MiB. Small allocations were being reused perfectly. Killed by measuring
  `bins.<j>.curslabs/curregs` rather than reasoning about the rate.
* **"`parse_duration_ns` allocating on its failure path."** Real, 6.45 GiB per 80 s, 4.8% of
  turnover, and fixed — and peak anon moved 1873 → 1863 MiB, inside run-to-run spread. A genuine
  hot-path defect that was not the cause, which is a distinction worth keeping separate.
* **"`canonical_index_values` and the `Vec` growth under it."** 26–47 B allocations, the small side
  again. Nearly refactored before the slab measurement said the small side was innocent.
* **"`fs::read` of the sidecar."** The timing was perfect — misses started at t≈190 and dirty went
  271 → 1122 MiB with them. Mapping it instead moved dirty per GiB of sidecar traffic 111 → 105.
  A correlation that was real and an effect size that was 5%.

Three methodological errors of mine, each caught by pushing back rather than by more runs:
`jeprof`'s cumulative column read as if it were flat (`encode_group_blooms` is 24.8% inclusive and
**1.5%** flat); `--inuse_space` read as turnover, which it cannot be, since the objects a churn
creates are gone by the sample (`prof_accum:true` and `--alloc_space` are what answer that); and
time-to-OOM compared across arms in a workload whose part count grows, so the arm that lives longer
is answering a harder question — matched part count and miss rate is the only honest window.

- [ ] **mimalloc returns almost nothing on settle where jemalloc returns 396 MiB.** Measured on the
      passing runs: anon 717.6 → 701.9 MiB across the 120 s settle under mimalloc, against
      1038.6 → 642.8 under jemalloc. Not a problem at these peaks, and not worth a knob, but it is
      the shape that would decide a tighter budget and no run has asked it directly

## The pools are gone; a request is priced before it runs

Two byte pools stood between work and memory, and neither could answer *how much is in use*. The query pool
was a semaphore handed out 8 MiB at a time **during** a scan, so a query that had already read for seconds
could die halfway through; the background pool was a separate counter that flush and metric compaction shared
with nobody. Two ceilings, no total, and a refusal that arrived after the work rather than before it.

The write path already had the right shape. `IngestGate::admit_body` counted request bodies at admission,
refused with `429` when the sum would pass a ceiling, and released on `Drop`. The read path is that now:

* **One account** (`memory_budget.rs`). `in_use_bytes` is what admitted work declared and has not released;
  `admit` is one `fetch_update`. Queries, metric flush and metric compaction all charge it, so the process
  has a single number instead of two, published as `signy_memory_account_in_use_bytes` against
  `signy_memory_account_budget_bytes`.
* **A price per request, computed before the scan.** Logs multiply `limit` by the average row their own parts
  recorded (`PartMeta::materialized_bytes ÷ row_count`, written at flush time — measured, not assumed), and
  read it off the catalogs of exactly the parts the pin just fixed. Metrics use `series × steps × 16 + label
  bytes`, exact and available the moment index-only selection ends. Traces are the one estimate with an
  unmeasured term: a trace part records stored bytes per tenant and nothing about decode cost, so the stored
  average is scaled by a fixed expansion factor, the scan is admitted against its full `max_trace_spans`
  ceiling, and `reconcile` settles it back to the truth when it returns.
* **Two refusals with opposite instructions.** No room *right now* is `429` (`EXHAUSTED_PREFIX`), counted in
  `signy_query_memory_exhausted_total`, and it lifts when a peer finishes — a test asserts exactly that
  sequence, since it is the whole claim the status code makes. No room *at all* is `400`
  (`OVER_BUDGET_PREFIX`), because telling a client to retry something that can never succeed turns one bad
  query into an infinite loop of them. Reachable only on the metric surface in a valid configuration: log and
  trace prices are clamped to `max_query_memory_bytes`, which startup now refuses to let exceed the account.
* **Admission moved in front of the scan permit** on the log and trace paths. A request that cannot be served
  no longer queues behind requests that can. Metrics keep the permit first because their price is a function
  of what the selector matched, and selection is the cheap part.
* **Nothing asks the account twice.** Every incremental `ensure` call is gone — the sink's, the trace
  registry's `ByteCharge`, the metric scan's three. `max_query_memory_bytes` stays as the per-query backstop
  under an estimate that undershoots, and `MemoryCharge::reconcile` corrects the account to what the work
  really held: upward so an overrun stops hiding from peers, downward so a pessimistic trace admission is not
  kept from them.

Startup now refuses a configuration whose per-operation ceilings do not fit the account, because the two
symptoms are silent in different ways — a query permanently `400`s while flush and compaction, having no
client to refuse, simply postpone on every tick and show up as disk growth an hour later
(`signy_memory_account_deferred_total`, alerted).

### The soak the change was measured against, and what it found instead (2026-09-03)

Four 2 GiB / 20 k eps / 15 min soak arms, plus the published gate. **The change is not in any of
it**, and the arm that says so is the one built from its own parent:

| arm | binary | died at | anon peak | committed peak | delivered |
|---|---|---:|---:|---:|---:|
| `memory-account` | change, mimalloc | OOM t=576 s | 2019 MiB | 2347 MiB | 19.9 k eps |
| `baseline-5b5b8cd` | **parent**, mimalloc | OOM t=469 s | 1871 MiB | 2635 MiB | 19.9 k eps |
| `account-clean` | change, mimalloc | (harness stopped it at t=542 s, still alive) | — | 2671 MiB | — |
| `jemalloc-arm` | change, jemalloc | OOM t=211 s | 1873 MiB | 897 MiB | — |

anon tracks within noise between the change and its parent at every matched second (t=460:
1519 vs 1511 MiB), and the parent dies *sooner*. Every arm delivered the offered rate until the
kill and answered zero query errors.

**The memory account is not where it goes, and the jemalloc arm is what could say so**: a release
mimalloc compiles its live-bytes counter out, jemalloc does not. `account_in_use` quartile means
were **1.3 → 9.8 → 5.0 → 3.8 MiB** against a 614 MiB budget, with `exhausted` and `deferred` flat
at zero for the whole run. No request was refused, no background pass postponed, and the read
path's whole footprint is a rounding error beside a 1.9 GiB anonymous peak. That is the answer to
"did admission-by-estimate cost anything" — it did not, and it also did not help, because query
memory was never the term.

**The published gate still passes, with room.** `memory_gate --budget 2GiB` on the change reads
`UNDER_BUDGET` at **785.2 MiB, 38.3 % of budget**, 19,951 of 20,000 eps, zero throttling, zero
query errors, and a settle peak (734.5 MiB) *below* the ingest peak. So the engine is at its
published state and the soak is asking a harder question, which is what it was built to do.

**What the soak is actually finding, and it is not the allocator.** The arms differ by allocator
and die the same way, so the shared term is elsewhere. The strongest evidence is a gauge that has
no business being large: `signy_memtable_bytes` reached **190 MiB with 307 MiB more buffered**
against the gate's 5.1 MiB peak on the same offered rate. The memtable is a backlog, and it is
full because flush could not drain it — `visibility_ms` up to 3.6 s, `advance_ms` 1.4 s, journal
`fsync_ms` 500–2400 ms, `io_full` 8.7–10.5 % of every run. Named gauges sum to ~837 MiB against
1866 MiB of anon; the rest is what a stalled writer leaves behind.

- [x] **Re-run the soak on a quiet machine before drawing a conclusion from it.** This rig was
      sharing a disk with a rust-analyzer at 4.0 GB RSS, a stale `loggytracy` squatting port 3100 at
      1.38 GB, and a victoria-metrics, on a filesystem 85 % full. The I/O stall numbers above are not
      the engine's to answer for until that is ruled out, and the 24-hour pass this is being compared
      against (`soak-24h-lockorder`, 2026-08-12) predates both mimalloc and this machine's current
      state. Until then the honest statement is: **the gate passes and the soak does not, on a
      contended host, identically before and after the memory account**

      *The host never had to be quieted: the cause was in the engine.* The bloom cache was holding a
      copy of `index.bin`, thrashing, and re-decoding four megabytes two hundred times a second — see
      the section below. On the same contended host, three 900 s runs at 2 GiB now finish with anon
      peaking 1450–1635 MiB. Whether a quiet host would do better is now a question about margins
      rather than about whether the soak can pass at all

- [ ] **The trace expansion factor is a guess, and the only one left.** Logs and metrics price themselves from
      recorded numbers; traces scale stored bytes by a constant because `TracePartMeta` has no materialized
      figure. Measuring it means either recording one at write time — a format change — or sampling decoded
      spans against their stored extent under load. Until then `max_query_memory_bytes` is what catches an
      optimistic factor, and `reconcile` keeps the account honest after the fact

## The languages can ask the same question; two translations were wrong

Established by measurement, not by reading:

* **VictoriaLogs does not serve the Loki query API at all.** `/loki/api/v1/query_range` answers
  `unsupported path requested`. Only ingest is Loki-compatible. Translating to LogsQL is forced.
* **But LogsQL can express what LogQL asks**, and both places the comparison did not match were mistranslations
  rather than limits of the language:
  * `|=` is a raw substring; a bare quoted string in LogsQL is a *tokenized phrase*. `~"..."` is the
    equivalent. On five lines built to straddle token boundaries, the phrase filter returned two and `~"..."`
    returned the same three `|=` does, case-sensitivity included. Fixed.
  * `rate()` exists in LogsQL and divides by the bucket width exactly as LogQL's does — measured at 0.0833 for
    five rows in a minute. `count()`, which the translation used, returned the bucket total and made the units
    incomparable. Fixed.

### The empty VictoriaLogs answers were a missing settle

Three separate three-way runs returned zero rows for VictoriaLogs and the cause was neither the query nor the
client: **VictoriaLogs makes ingested rows searchable after an in-memory flush**, and the ad-hoc shell loop
queried immediately after seeding. `compare/run.sh` settles for 150 s and would not have hit it. signy
and Loki answer from their memtable and ingester, so the bed's settle was never load-bearing before and is now.

### What did not agree, and every cause — all four resolved

Verified by a full `compare/run.sh` smoke (30,000 rows, 8 streams, limit 100, all three containerized):
**every pair, every shape, 24/24** — strict between signy and Loki, reduced against VictoriaLogs.

- [x] **`line_filter` 2,008 against 1,147 — the corpus, not either engine.** `corpus::json_line` named its
      free-text key `msg`; VictoriaLogs parses JSON at ingest and keeps the message only under `_msg`, so
      every JSON row's text was discarded and `~"phrase"` (a filter on `_msg`) could not see it —
      plain+logfmt are 5 weight in 10, which is exactly 1,147/2,008. The key is `_msg` now, so all three
      engines see the message where they expect it; a unit test pins that `| json` extracting a field named
      `_msg` stays an ordinary field on this side
- [x] **`rate` 41 series against 24 — the window was a question only one language could ask.** The LogQL
      side sampled a 1m sliding window on a 10s step; LogsQL has only tumbling `_time` buckets. The rate
      window now *equals* the step and the first evaluation point moves one step in, which is the one
      configuration where consecutive lookbacks tile the range exactly as buckets do. Two residues, both
      converted rather than exempted: LogsQL labels a bucket by its epoch-aligned *start* where LogQL labels
      the evaluation point — the digest adds the query's own step; and a bucket closing on the dataset's
      trailing edge is asymmetric between `(start, end]` and `[start, end)` — measured as 124.9 against
      125.0, one boundary row — so rate windows are slid one step off the data's edge
- [x] **The reduced digest could never agree, and the fix was choosing the right basis.** "Timestamp plus
      the whole field set" compares the *storage models*: schema-on-write returns every field it parsed,
      schema-on-read returns what the pipeline produced, so 0/24 was the checker disagreeing with itself.
      The basis is now the row's nanosecond timestamp plus **the fields the query itself named**
      (`Query::basis_fields`) — the one set of fields every system returns for the same row — with
      VictoriaLogs' RFC 3339 `_time` converted to the nanosecond encoding the Loki side already used.
      Unit tests digest a hand-built Loki-shape and LogsQL-shape pair equal, for a log and a metric answer
- [x] **`json_field_rare` 8/24 against Loki — a label signy never attached.** Loki pairs `__error__`
      with `__error_details__` on a parser failure and Grafana renders both; signy set only the first.
      It sets both now. The details *text* is each engine's parser internals — Loki's comes from its JSON
      library — so the digest compares the label's presence and normalizes its value, an exemption by name,
      stated in the report with counts
- [x] **And one more the agreement check caught that no translation explains — see "Open correctness
      defects": a backward `limit=100` returned rows from the middle of the window.** The strict digest
      against Loki is what surfaced it; it was never a three-way issue at all

**The timing table from this configuration is published by `compare/run.sh` itself now**, and only because
every shape agrees — the report withholds a ratio for any shape that does not.

## The three-way table was published without an agreement check, and it should not have been

`docs/COMPARISON.md`'s own rule is that row equality "matters more than any timing, because a fast wrong
answer is not a win". The two-system bed enforces it — `compare/run.sh` runs the check and
`compare_report` prints it. The three-way run bypassed the bed and used a shell loop, so nothing compared the
answers, and a timing table went out anyway.

Comparing the answers afterwards, on the same run:

| shape | signy = Loki (strict) | signy = VictoriaLogs (reduced) | row-count mismatch vs VL |
|---|---|---|---|
| `label_only` | 24/24 | **0/24** | 0/24 |
| `line_filter` | 24/24 | **0/24** | **24/24** |
| `json_field` | 24/24 | **0/24** | 0/24 |
| **`json_field_rare`** | **8/24** | **0/24** | 0/24 |
| `metadata_rare` | 24/24 | **0/24** | 0/24 |
| `rate` | 24/24 | **0/24** | **17/24** |

### The reduced digest is not a common basis, by construction

It was supposed to be "timestamp plus field set, no message, no placement". The VictoriaLogs parser puts
`_msg` into the field set and the LogQL parser does not put the line into it at all, so the two can never
match — 0/24 everywhere is not a finding about the engines, it is a defect in the checker. This is the
question that was flagged as hard and then answered with a shortcut.

### And signy disagrees with Loki on `json_field_rare`, 16 answers out of 24

Between two systems that *do* share a basis, with the same 72 rows returned on both sides, so the difference
is content rather than count. Unnoticed because this run looked at no agreement at all.

- [x] **Fix the reduced basis so it is actually common** — done; the basis is the query's own fields, the
      full argument and the cross-shape digest tests are recorded in the section above
- [x] **Investigate the 16 `json_field_rare` disagreements between signy and Loki** — `__error_details__`,
      a label Loki attaches on parser failure and signy did not; attached now, wording exempted by name
- [x] **Fold VictoriaLogs into `compare/run.sh`** — `compare/docker-compose.yml` grew a pinned
      `victoria-logs:v1.52.0` service and `run.sh` is rewritten around a `TARGETS` list with per-target
      readiness (`/health`), volume-mounted disk measurement (the image is built from scratch and has no
      shell to `exec du` in), a per-target settle flush (`/internal/force_flush` — the settle is
      load-bearing for VictoriaLogs, not a courtesy), and a three-way `bed.json`. There is no per-target
      code path left to fork, so a run that includes a system includes its checks. The load phase asks
      VictoriaLogs LogsQL at its own endpoint with the same seeded rolls, so its ingest runs under the same
      concurrent read load as the other two
- [x] **Gate the timing table on agreement** — `compare_report` now prints agreement per pair per shape
      *before* any timing, and every ratio cell is `gated_ratio`: a shape whose answers disagree prints
      `withheld (N/M disagree)` where the ratio would be. The verdict computes only over agreeing shapes.
      One subtlety a binding limit forced: a bare LogsQL `limit` has no order contract and returned the
      window's *oldest* rows where `direction=backward` returns the newest — disjoint 100-row sets that are
      both "100 rows from the window" — so every translated `limit` is now `sort by (_time) desc | limit`

VictoriaLogs does **not** serve the Loki query API — `/loki/api/v1/query_range` answers `unsupported path
requested`. Only ingest is Loki-compatible. So translating to LogsQL is forced rather than chosen, and the
`|=` substring-versus-phrase and `rate` bucket differences that follow from it are real inequalities in the
comparison, not stylistic ones.

## VictoriaLogs is in the measurement now, and it changes the reading

The query adapter is built: LogsQL translation for all six shapes, `/select/logsql/query`'s
newline-delimited JSON parsed, and a **reduced digest** computed by every system so the three can be compared
at all — the strict digest covers a timestamp, a line and every label with its placement, and VictoriaLogs has
no line for a JSON row, so it is computed on the basis all three share (timestamp plus field set, no message,
no placement) and the strict one still holds between the two that keep lines.

Same corpus, same seed, same anchor, 30,000 rows over 8 streams, limit 100, all three local, build `0f7ca1c`:

| shape | signy | Loki | VictoriaLogs | lt/Loki | lt/VL | lt rows | VL rows |
|---|---|---|---|---|---|---|---|
| `label_only` | 0.77 ms | 8.76 ms | 3.10 ms | **0.09x** | **0.25x** | 2,400 | 2,400 |
| `line_filter` | 1.00 ms | 6.17 ms | 1.63 ms | **0.16x** | **0.61x** | 2,008 | 1,147 |
| `json_field` | 4.55 ms | 10.03 ms | 1.44 ms | **0.45x** | 3.15x | 2,393 | 2,393 |
| `json_field_rare` | 38.11 ms | 31.38 ms | 3.60 ms | 1.21x | **10.57x** | 72 | 72 |
| `metadata_rare` | 43.09 ms | 18.02 ms | 3.59 ms | 2.39x | **11.99x** | 72 | 72 |
| `rate` | 8.57 ms | 4.95 ms | 0.27 ms | 1.73x | **31.56x** | 41 | 24 |

**Against Loki the picture was mixed. Against VictoriaLogs it is not.** signy wins the two label-and-
substring shapes and loses everything else, by an order of magnitude on the two rare-value lookups and by
thirty on the metric.

**And VictoriaLogs answers `json_field_rare` and `metadata_rare` identically** — 3.60 ms and 3.59 ms — because
it does not distinguish a field that arrived as an attribute from one it extracted from the message. Both are
columns. That is the clearest statement of the gap: the 1.4x that columnizing bought *inside* VictoriaLogs
(measured earlier, JSON versus logfmt) is not the same thing as the 12x between the two engines. Columnizing
is necessary and it is not sufficient.

Two caveats on the numbers. `line_filter` returns 2,008 rows here and 1,147 there — LogQL's `|=` is a raw
substring and LogsQL's is a tokenized phrase, so those two are not answering quite the same question, and the
`rate` row is 41 series against 24 for the same reason of bucket semantics. Both are stated in `logsql()` and
neither is a defect in either engine.

- [x] The three-way run is ad hoc — **it is the bed now.** `compare/run.sh` drives all three, containerized
      at the same memory limit, and `docs/COMPARISON.md` is regenerated by it. The table above is the ad-hoc
      run's record and its shape held: the reproducible run at 2 GiB, 150,000 rows, over answers that all
      agree, reads `label_only` 0.33x / `line_filter` 0.89x / `json_field` 0.94x / `json_field_rare` 0.33x /
      `metadata_rare` **0.24x** / `rate` 2.33x against Loki, and 1.85x / 2.7x / 7.1x / **30.0x** / **12.6x** /
      **60.9x** against VictoriaLogs. Two of the numbers above were this side's own defects wearing a
      performance costume: `json_field_rare` 1.21x and `metadata_rare` 2.39x against Loki became 0.33x and
      0.24x once the bounded scan stopped dropping the newest rows — the claim's shape now *wins* its Loki
      half. The VictoriaLogs half is the columnization gap, unchanged in kind
- [x] Report the reduced digest as its own agreement column — the report prints agreement per pair per
      shape, names the basis of each pair, and withholds any ratio over a disagreement
- [ ] The read path after the pushdown work, all of it measured in the bed over full three-way agreement:
      the timestamp page index now prunes sub-group pages (a window sweep pins that it never drops a row),
      `|~` prunes by the literals every match must contain, and a restore no longer holds a scan slot.
      Final: `metadata_rare` **3.05 ms** — 0.04x against Loki, **2.04x** against VictoriaLogs, from 43 ms
      and 12.6x at the start. Deferred with its reason: parallel part scans, because backward queries lose
      cross-part frontier tightening and no measurement yet says the trade wins.
- [ ] The metric gap: projection pushdown took `rate` from 25 ms to **9.5 ms** — a 2.33x loss to Loki is
      now a 0.87x win, and the VictoriaLogs gap fell from 61x to 23x. What remains for it is sidecar-only
      evaluation for filterless windows (the per-row-group row counts are in `meta.json` already), which is
      the step VictoriaLogs itself does not have. And the two-pass scan over the `_sm:` columns took
      `metadata_rare` from 18.5 ms to **3.7 ms** — 0.05x against Loki, and the VictoriaLogs side of the
      claim is now 2.42x where it was 12.6x. **Every shape now beats Loki**; `json_field_rare` (25x against
      VictoriaLogs) waits on parsed-field columnization, the one shape whose cost is the line parse itself
- [x] **Parsed-field columnization landed** — `_pf:` columns holding exactly what `| json` would extract
      (the same `extract_json`, so the column is right by construction), the line kept, metadata shadowing
      preserved in the two-pass scan and pinned by a hit/miss/shadowed equality test. Measured in the bed
      over full agreement: `json_field_rare` 44 ms → **4.5 ms** — 0.03x against Loki, **3.3x** against
      VictoriaLogs from 28.5x — and `json_field` 48 → 21 ms (0.40x / 3.2x). The price, also measured:
      ingest 19.8k → 18.5k eps against an offered 20k, and settled disk 0.47x → 0.64x of Loki's. Half the
      added write cost was a duplicate extraction and is recovered (`part/write/json` −7%, one parse shared
      by the key counts and the column fill); the ingest number after that recovery is the re-run below
- [ ] **Sidecar-only metric evaluation has a precondition this bed does not meet, recorded before the
      wrong version of it gets built.** Counting matched rows from `row_group_rows` is sound only when a
      group holds matched streams alone; groups are cut per tenant at 8192 rows with streams contiguous
      inside, so at the bed's rows-per-stream a group nearly always mixes streams and the fast path
      degenerates into the scan it replaces. It pays exactly where groups are stream-pure — one hot
      stream, or matcherless windows — and paying anywhere else needs per-(group, stream) row counts in
      the stream index, a format change. The bed's remaining `rate` gap (0.42 ms against 9.6) is *not*
      this item: it is the metric path materializing sixty thousand `LogEntry`s to add one per row, which
      is a count-in-the-sink change, not a sidecar
- [x] **The count went into the sink.** `sum()` of `rate`/`count_over_time`/`bytes_over_time` (no unwrap,
      no grouping, no offset, no subquery) now accumulates a difference array over the evaluation grid —
      two array updates per row, nothing materialized — behind the same pipeline, deletion-mask and budget
      rules, with a fast-against-general equality test and the bed's digest holding at 144/144. With the
      stage-less pipeline skip that preceded it, `rate` went 11.4 → 8.35 → **6.56 ms**: 0.61x against Loki
      and 14.5x against VictoriaLogs, from 1.73x and 31.6x at the start. What remains of the 14.5x is scan
      itself — decode of the timestamp and label columns of 60 k rows against VictoriaLogs reading one
      column — which is where the (preconditioned) sidecar counts or a dictionary-run count would go
- [x] **The "unshareable parse" judgment was wrong, and reading the code corrected it.**
      `indexed_parser_fields`' json half *was* `extract_json` behind a first-byte gate — the same call the
      `_pf:` columns pay for — so the bloom now reads that parse and only the logfmt half still parses.
      The gate went with it, which closes a quiet pruning hole: a top-level-array line's extractions never
      entered the bloom, so an exact-field prune could false-negative a group that held the answer.
      Measured: `part/write/json` 331 → **281 ms** (+5% over the pre-`_pf:` 267, the columns' true cost)
      and ingest **19.1k → 19.6k eps** — within 1.3% of Loki's 19.86k at the same offered 20k. What still
      separates the last 2% is the wider Parquet encode, and per-column encoding choices measured against
      `part/write` remain the honest next lever.
      *First choice tried and discarded at the bench:* dictionary off for every `_sm:`/`_pf:` column read
      281 → 279 ms (p = 0.84) — the dictionary build is not a measurable term of the write, so the change
      never reached the bed and never earned its complexity. The 2% stays attributed to the encode as a
      whole, not to any one knob so far tried
- [x] **The trace-to-logs measurement ran, and it answers the join question by pointing somewhere else.**
      `trace_window` — the rare trace's own occurrence ±1 s, the window a click on a span sends — is in the
      matrix at full three-way agreement. Loki drops 79 → 32 ms and VictoriaLogs 1.3 → 1.15 ms on the
      narrow window; **signy does not move at all** (5.2 → 5.1 ms, lines-read 589,824 unchanged),
      because a stream-first group spans nearly the whole time range — group-level time pruning is inert —
      and the two-pass scan's first pass does not apply the page-index time selection, so it examines every
      row of every admitted group whatever the window says. Two conclusions, in order: the next read-path
      target is intersecting `time_page_selection` into pass one (a few lines, then the narrow window
      should land near VictoriaLogs); and the **server-side trace↔log join is rejected** — the client
      already sends the window, so once the window actually cuts work the join has nothing left to buy,
      and its soundness caveats (clock skew, late logs, absent traces) buy nothing back.
      *Both halves then ran.* Pass one obeys the page selection now, and the pages had to be given a row
      bound first — an 8192-row group's timestamp chunk fit one default-sized page, so the page index was
      exactly as coarse as the group bounds it exists to refine. At 1024 rows a page: `trace_window`
      lines-read 589,824 → **221,184** and `metadata_rare` → 360,448, disk *improved* to 0.60x Loki, and
      the latency barely moved (5.1 → 5.0 ms) — so the rare shapes' remaining ~5 ms floor is no longer row
      volume but per-scan constants, the footer and page-index load per part times the parts a
      matcherless `{app=~".+"}` query admits. That is where the next read-path work goes, and it is a
      caching question (footers across scans) rather than a pruning one.
      *Answered:* the reader now caches the parsed footer and page index for its lifetime — the parse, not
      the descriptor, so eviction still reclaims the bytes — and the floor fell exactly as named:
      `metadata_rare` 5.1 → **2.5 ms** (1.87x against VictoriaLogs, from 12.6x at the start),
      `trace_window` → 2.4 ms, `json_field_rare` → 3.0 ms, `rate` → **3.25 ms** (0.31x against Loki, 8.0x
      against VictoriaLogs from 31.6x), `label_only/backward` in the bench −68%. Memory gate UNDER_BUDGET
      at 2 GiB with the cache resident; the footers join the sidecars in the outside-the-budget residency
      question M10 already owns.
      *And the per-row label cost after it:* rows inside a group are stream runs, so the label hash, cache
      lookup and matcher evaluation now run at run boundaries only — a handful of memcmps everywhere else.
      `rate` 3.25 → **1.69 ms** (0.16x Loki, **4.0x** VictoriaLogs from 31.6x at the start),
      `metadata_rare` at **1.58x** against VictoriaLogs, `trace_window` 1.49x, `line_filter` 1.46x,
      `part/scan_tenants/128` 1.9 ms → 216 µs. What still stands against VictoriaLogs: `json_field` at
      2.9x, whose remaining cost is the response's own semantics — every returned row must carry the
      extracted fields, so the pipeline parses the line per survivor; feeding that extraction from the
      `_pf:` columns is the one idea left on this axis.
      *After it, the counting scan stopped reading labels where the index proves it may:* a stage-less
      count with no deletion mask needs a label only to check the matchers, and for a group that one
      value of every matched label touches — checkable from the stream index per group, given every
      stream carries the label — the check is already done. Such groups project no label columns at all.
      The bench's 128-stream corpus has no uniform groups (streams shorter than a group) and did not
      move; the bed's does, and `rate` went 1.68 → **1.47 ms cold, 1.30 warm** — 0.14x against Loki,
      **3.2–3.4x** against VictoriaLogs, from 31.6x at the session's start.
      *Tried after it and reverted as noise:* handing a blind batch's timestamps to the sink whole — no
      `LogEntry`, no per-row dispatch. The bed read 1.54/1.55 against 1.47/1.30, inside the run-to-run
      spread, so the per-row structure was not the remaining cost and the complexity went back out
      (revert of `68f3b98`; the evidence run is committed right before it). What remains of rate's ~3.5x
      is the timestamp decode and the grid arithmetic itself, which is the shape of a sidecar count — and
      that still waits on its recorded precondition.
      *Tried, measured, and turned off with its reason in `ColumnSet::for_log_query`:* the machinery is
      wired and correctness-pinned (sinks accept a precomputed extraction, `line_format` revokes it, the
      memtable-against-parts tests hold), but decoding ~15 parsed columns wide cost more than the
      per-survivor parse it saved — `json_field` 21.5 → 25.7 ms with it on, back to 20.6 off. It flips
      sign only when the survivor share of the decode is high; the projection waits for that measurement

## The claim moved onto its worst shape, and the measurement said so within the hour

`metadata_rare` — `| trace_id="x"` with no parser stage, which is what an OTLP attribute produces and what
[`docs/VISION.md`](docs/VISION.md)'s claim was rewritten to rest on — was added and run. Short run, 30,000
rows over 8 streams, limit 100, both systems local, build `b4a8589`:

| shape | signy | Loki | ratio | lt lines | Loki lines | rows |
|---|---|---|---|---|---|---|
| `label_only` | 0.77 ms | 8.22 ms | **0.09x** | 32,400 | 35,643 | 2,400 |
| `line_filter` | 1.00 ms | 6.08 ms | **0.16x** | 90,000 | 35,849 | 2,008 |
| `json_field` | 4.80 ms | 9.45 ms | **0.51x** | 56,556 | 35,791 | 2,393 |
| `json_field_rare` | 38.70 ms | 23.46 ms | 1.65x | 703,728 | 334,192 | 72 |
| **`metadata_rare`** | **42.98 ms** | **13.86 ms** | **3.10x** | 589,824 | 334,192 | 72 |
| `rate` | 8.55 ms | 4.63 ms | 1.85x | 253,728 | 60,034 | 41 |

**The shape the claim now rests on is the one signy loses worst** — worse than the parser shape it
replaced, and against a system that does not index structured metadata at all.

### Why: structured metadata is not a column, it is a JSON blob

`format.rs` writes `structured_metadata` as a single `Utf8` Parquet column holding
`serde_json::to_string` of the pairs, and `reader.rs` runs **`serde_json::from_str` per row** to read it back.
So a metadata filter pays a JSON parse per row — the same cost `| json` pays on the message, with none of a
column's benefits.

The arithmetic separates the two effects. signy reads 589,824 lines in 42.98 ms, or 13.7 M lines/s; Loki
reads 334,192 in 13.86 ms, or 24.1 M lines/s. So it is **1.76x more lines read and 1.8x slower per line**, and
the two multiply. And `metadata_rare` reads *fewer* lines than `json_field_rare` while taking *longer*, which
is the per-row parse showing up on its own.

**This contradicts the data model as documented.** [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) says general
fields are "stored in columns and pruned with bloom filters". They are pruned with bloom filters. They are not
in columns.

- [x] **Columnize structured metadata** — done, and the architecture sentence is true now. One canonical
      form at the memtable door (sorted keys, first value wins — the visibility the pipeline already gave a
      duplicate); one nullable `_sm:<key>` Utf8 column per key up to `MAX_METADATA_COLUMNS = 128`, chosen by
      row count so key churn cannot evict `trace_id`, with the leftover pairs in the old blob column and the
      invariant that a columnized key never also appears in a row's residual; the key list and per-key row
      counts in `meta.json`, so a merge picks its output's columns by summing its inputs' counts the way it
      already unions their `stream_labels` — no row read before the schema exists. `meta.json` also gained
      `row_group_rows` and `row_group_ts_monotonic` in the same format change, for the read-path work that
      needs them. The reader rebuilds a row's pairs by merging two key-sorted lists; the residual is null
      for every row of the intended consumer, so the common path runs no serde at all. Measured:
      `part/scan_label_columns` **−76–77%** (a full-part scan's per-row cost, which was the 1.8x/line term),
      `part/write/json` **−17%** (the bloom encoder now parses each line once instead of twice — sizing
      reused the tokens), `part/write/plain` flat, and the 2 GiB memory gate `UNDER_BUDGET` at **74.8%** of
      budget against 78–83% before. The bed rerun is the number that decides `metadata_rare`
- [ ] The scatter problem is still underneath it. 72 rows across 8 streams cannot be pruned to fewer row groups
      by any bloom, so even a free filter leaves the lines-read gap. Columnizing addresses the per-line cost,
      not the count — sub-row-group selection over the `_sm:` columns (late materialization) is the next step
- [x] **`docs/VISION.md`'s claim** — rewritten from the three-way agreement run: the Loki half holds at
      0.24x and the VictoriaLogs half fails at 12.6x, with the earlier 3.10x loss identified as this side's
      own bounded-scan defect. See the claim section in `docs/VISION.md`

## Next — OTLP only, and the claim moves with it

Decided 2026-07-31 and written down in [`docs/VISION.md`](docs/VISION.md), "Ingest is OTLP", and
[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)'s decided choices. The one intended consumer sends OTLP for
everything this engine would store; node metrics are not this engine's business, and journald can be converted
by the collector.

**Order matters here and is the opposite of the tempting one.** Measure that the shape being kept actually
wins *before* deleting the one being dropped — removing code is the irreversible half.

- [x] **Add the structured-metadata shape to the comparison.** `| trace_id="x"` with no parser stage, which is
      what an OTLP attribute produces. This is where the three genuinely differ: signy indexes structured
      metadata into a per-row-group bloom, Loki stores it and does **not** index it, VictoriaLogs turns it into
      a column. Done as `metadata_rare` (and its narrow-window sibling `trace_window`); it is the shape the
      claim now rests on.
- [x] **Move the bed's ingest to OTLP** so all three receive the same records the consumer sends. Loki accepts
      OTLP at `/otlp/v1/logs`, VictoriaLogs at `/insert/opentelemetry/v1/logs`; both need checking rather than
      assuming. This also removes the bed's dependence on the endpoint being deleted

      **Done 2026-08-02, gated on full agreement: 168/168 on all three pairs, every shape.** The re-baseline
      (2 GiB, cold p50 ratios, `docs/COMPARISON.md` of that date): vs Loki everything holds — label_only
      0.29x, line_filter 0.82x, json_field 0.57x, json_field_rare 0.02x, metadata_rare 0.03x, trace_window
      0.07x, rate 0.27x. vs VictoriaLogs: label_only 1.77x, line_filter 2.19x, json_field 2.10x (VL now
      pays `unpack_json` at query time, was 3.13x), **json_field_rare 0.04x** (was 2.55x — the parser-stage
      rare pair finally measures the bloom against a full-scan unpack), metadata_rare **1.49x** (was 2.18x —
      the claim shape, still shy of the 1.1x bar), trace_window 1.61x, rate 7.13x (was 4.55x — worse, VL's
      tumbling-bucket rate over OTLP data got faster). Ingest 19,619 eps of 20,000 offered (Loki 19,867,
      VictoriaLogs 19,941); disk 0.60x Loki / 1.29x VictoriaLogs WAL-excluded; `memory_gate --budget 2GiB`
      UNDER_BUDGET on the same revision.

      **Probed 2026-08-02** (Loki 3.3.2, VictoriaLogs v1.52.0, identical hand-built
      `ExportLogsServiceRequest` to both plus signy's own `/v1/logs`):
      * Loki `/otlp/v1/logs` answers 204 with `X-Scope-OrgID`; the six semconv resource attributes
        (`service.name`, `deployment.environment`, `k8s.{cluster,namespace,container}.name`, `cloud.region`)
        all promote to stream labels with dots sanitized to underscores — byte-identical to what signy's
        own promotion produces. Record attributes and the record `trace_id` (32-hex) land in structured
        metadata; nanosecond timestamps survive exactly; `detected_level` still appears (already exempt).
      * VictoriaLogs `/insert/opentelemetry/v1/logs` answers 200, takes `X-Scope-OrgID: 0`, keeps the
        **dotted** attribute names as fields (`service.name`), puts all resource attributes into `_stream`,
        and always adds `severity_text:"Unspecified"`/`severity_number:"0"` (outside every basis, harmless).
        LogsQL accepts the dotted name unquoted: `service.name:"x"`, `"service.name":"x"` and
        `{service.name="x"}` all answer the same rows.
      * **VictoriaLogs does not parse a JSON body arriving via OTLP.** `_msg` holds the raw line and no
        fields are extracted — its famous ingest-time parse was a property of its Loki push endpoint, not of
        the engine. `| unpack_json | filter field:"v"` supplies the parser stage at query time, so under OTLP
        all three pay the parse at read time and the `json_field`/`metadata_rare` pair separates parsed-line
        from attribute storage on every system, VictoriaLogs included.
      * Consequences carried into the harness: corpus labels ride as the semconv names above (`app` →
        `service.name`, queried as `service_name`), the reduced digest canonicalizes keys through the same
        sanitizer so `service.name` = `service_name`, and `service_name` left `DERIVED_LABELS` — it is pushed
        data now.

      **First OTLP bed run (2026-08-02), what actually broke and what did not:**
      * Five of seven shapes agreed across all three pairs on the first try; `metadata_rare`, `trace_window`
        and `rate` at 24/24 everywhere. The two that did not were both the bed's own defects, not engines':
        the harness counted OTLP's 200 as an error (the Loki-push arm accepted only 204), a bare
        `unpack_json` let the corpus's inner `_msg` JSON field clobber VictoriaLogs' message column
        (fixed with `unpack_json fields (...)`), and Loki's `label_only` at limit 20000 hit its own
        internal 4 MiB querier→frontend gRPC frame default — the answer was 4,233,058 bytes, 1% over,
        because the semconv label names repeat per structured-metadata-combination stream. Raised in
        `loki-config.yaml` with the measurement in the comment.
      * The signy load verdict failed on flush backlog and query queueing — and the same failure
        reproduces **with the pre-migration engine and pre-migration harness on the same host the same
        day** (worktree A/B at 84ea2e2, 20k eps: memtable waves to ~700k entries either way). The morning
        bed's signy ingest phase also shared its 12 cores with this session's own `cargo build/test`
        of P2 — the bed's fairness note exists precisely because the harness assumes the machine is its
        own. Not attributable to the OTLP migration; rerun on a quiet machine before reading those rows.
      * Wire bytes per entry rose 157.7 → 426.6 (snappy left with the Loki push client; OTLP protobuf is
        sent uncompressed). The WAL is unchanged: the old path stored the *decompressed* PushRequest at
        ~415 B/entry and the OTLP export measures ~418 B/entry on the same corpus.
      * VictoriaLogs' behavioral gate read zero rows from an engine that ingested millions: the probe
        client's only two uses are separated by the whole run, VictoriaLogs closes idle keep-alives, and
        the first request on the dead socket was the scrape. `scrape()` now retries once on a fresh
        connection.

      **Second run (quiet machine): 168/168 strict with Loki; VictoriaLogs pairs 160/168, and the last
      eight were the shadowing rule.** Every `json_field_rare` disagreement was `1 against 0 rows` in a
      window where the rare trace's row is not a JSON line: signy answers it because the structured-
      metadata label shadows the failed extraction, VictoriaLogs' `unpack_json` *erased* the attribute
      when the unpack found nothing. `keep_original_fields` is that shadowing rule spelled in LogsQL —
      verified live on the failing window before the fix went in.

## Queries during heavy ingest stall for tens of seconds, and it predates OTLP

Found by the OTLP bed runs and then measured against the pre-migration engine, so it is not the
migration's: under a 2 GiB cgroup on real disk at 20k eps offered with 5 qps of live-window queries,
**the 84ea2e2 engine driven by the 84ea2e2 harness over Loki push fails the same way the OTLP pair
does** — query service max 16.3s (old) vs 14.7s (new), response p95 ~12–13s both, anon peak riding the
container limit at ~2.1 GiB both, and a repeat load round OOM-killed the bed container (137). The
morning-bed FAIL rows that looked like a migration regression were this, plus this session's own
`cargo build` sharing the twelve cores during the first run's ingest phase.

What the local memprof legs showed while the waves ran: `flush` arena spikes to ~530 MB and `merge` to
~320 MB against a ~130 MB memtable — the flush pipeline materializes several times its input — and the
allocator keeps the high-water mark resident afterwards. In 2 GiB that reads as reclaim pressure,
20-second queries, and a WAL backlog that nets upward. Open questions, in measurement order: whether
the flush transient can stop being ~4x its input (rows_from_snapshot copies plus `serde_json::Value`
per JSON row plus arrow plus parquet buffers, all live at once); whether query-under-ingest deserves
its own budget the way scans have permits; and whether the load verdict's 2s query-p95 target was ever
passed by any revision — no retained artifact says it was.

**Resolved 2026-08-03** (`c0fd93c`..`f7d9a36`), in three measured moves; every leg is the memprof rig
(2 GiB `systemd-run`, real disk, 20 k eps + 5 qps, seed 1592598566), baselined the same day because the
stale-figure warning at "flush cannot be sized independently of ingest" was right — the fresh baseline
at `761999a` read flush **513.7 MiB** against a 157 MiB memtable, merge 328.2 MiB, and the server dead
at t≈56 s with anon 2002.5 MiB.

1. **The flush is chunked** (`part::flush_snapshot_chunked`, `SIGNY_FLUSH_CHUNK_BYTES`, default
   32 MiB): streams walked in `(tenant, labels)` order, entries ordered through a side index because
   the snapshot is shared with queries, per-stream adjacent dedup so a cut between twins is safe, all
   inside `spawn_blocking`, all-or-nothing rollback kept. The whole-snapshot copy, the global sort on
   an async worker, the partition-wide `parse_rows`, and the part-wide index buffers are all bounded by
   the chunk now. Flush arena peak: **513.7 → 96.1 MiB**; the run stopped dying. Companion:
   `merge_max_memory_bytes` is no longer dead — half of it is the paging budget, pages shrink from
   8 MiB toward a 2 MiB floor as the group widens, and `group_for_merge` caps a group's part count at
   what that budget can page, because a chunked flush feeds merge more, smaller parts.
2. **Memory alone did not move the stall.** With flush at 96 MiB the queries still served p99 at
   13.9 s, all five shapes alike, and the sampler said why: at every merge tick (t=30.6 exactly) parts,
   flushes and `query_success` froze together for the length of the rewrite. The 56-part group's
   rewrite held the fair `operation_lock`'s read half for ~13 s, the next flush queued its write, and
   every query arriving after queued behind the flush. Shrinking the flush writer's own section
   (readers opened on the blocking thread, checkpoint moved after the lock — `719e60b`) was correct
   but not sufficient: the holder was merge.
3. **Merge now reads under a `deletion_lock`** (`f7d9a36`): the rewrite only ever needed "nobody
   deletes these files", never visibility, so that guarantee is its own lock; deleters (retention
   retirement, cache eviction) take operation-then-deletion, and merge commit installs readers it
   opened before the operation write lock (`replace_opened`). Same rig after: **query response p95
   12,530 → 229 ms, service p99 13,880 → 327 ms, max 345 ms**, achieved 19,949 of 20,000 eps, WAL
   backlog trend negative over the run, `q_ok` incrementing straight through both merge ticks —
   verdict **PASS**, the first load-phase PASS any retained artifact shows, 2 s target included.

Found in the equality gate on the way and worth more than the feature: **every streamed merge since
"the merge streams" has been reading its inputs in query order, not layout order.**
`read_rows_in_row_groups` inherited `scan_into`'s visit-groups-by-`min_ts` sort, and once a row group
straddles two streams its `min_ts` reaches back to the younger stream's start — so a windowed rewrite
page left `Row::sort_key` order, `MergedRows` k-way-merged a broken promise, and the parts merge wrote
carried the interleaving (weaker stream contiguity, dedup adjacency not guaranteed, the reader's
within-tenant time-order assumption violated). A windowed read now keeps ordinal order
(`a_rewrite_read_returns_layout_order_not_query_order` holds it). The `metadata_rare` matrix rows
moving from 4.78x/4.98x slower than VictoriaLogs to **1.61x/1.83x** in the same bed is consistent with
merged parts pruning properly again, though the two changes landed together so the split is not
isolated.

Gates, all on `f7d9a36`: cargo test 452+39 green, clippy 0; `memory_gate --budget 2GiB`
**UNDER_BUDGET at 39.8%** (anon peak 814.8 MiB, settle included, memtable peak 5.4 MiB — the waves are
gone, ingest and settle peaks 792/815); full bed run with **agreement 168/168 on all three pairs,
every shape**, ingest 19,810 eps against Loki's 19,870 and VictoriaLogs' 19,932, all three systems
surviving the first 2 g attempt, signy's bed-phase query response p95 **98.5 ms** against Loki's
429 ms. The bed's anon-during-ingest peak fell 1278.4 → **832.0 MiB**, now below Loki's 1123.

What the run gave back to the open list: disk settled at **0.60x Loki / 1.28x VictoriaLogs** against
0.56x/1.18x before — the chunked flush leaves more, smaller parts at the settle and the per-part
overhead is visible; if it matters, the lever is `flush_chunk_bytes` up or merge appetite, measured,
not assumed. Query-under-ingest still has no *budget* (open question 2 stands, now with a passing
baseline to regress against). And `anon/live` in the memprof legs still reads ~5–8 — the allocator
retention item at M10's "honest metering" is untouched by any of this.

## Next: production polish, not more speed (decided 2026-08-07)

The performance arc is frozen where it stands: every log shape under VictoriaLogs on both passes,
`rate` at the 0.03 ms jitter floor, everything gated and recorded below — all of it **at the bed's
150 k-row dataset**, which the ten-times run of 2026-08-12 scoped rather than extended (item 3
below: the claim's own shape widens to 15.4x/17.8x, three scanned shapes lose). The user's direction
after the sweep: wrap toward production readiness rather than extend the bench. The reasoning,
recorded so the next session does not relitigate it: the claim is proven and sealed; further hot-path
caching widens the answer-risk surface for diminishing returns (the advisory serve lived one day);
and the next real bottleneck will come from running long, not from benching bigger — which the
retention lock-order stall then proved on 2026-08-11, found by running long and not by benching big. The one measured
rejection standing guard: **do not build the memory-budget arena derivation** — deriving knob
defaults from a memory limit was tried and made the engine worse (config.rs `HostMemory` doc,
docs/MEMORY_BUDGET_GATE.md), and it stays out until a budget-point measurement campaign says
otherwise.

The polish order, each item a measurement or a bound, none a feature:

1. **Soak leg** — hours-to-a-day of ingest + query + merge + retention on the 2 GiB rig. Every gate
   so far is minutes long. What only a soak can show: part-count growth against the unevictable
   sidecars, WAL behavior under the local-compaction policy (compacts only with an object store
   configured), merge cascade pacing, row-group-cache counter drift over thousands of
   evictions/retractions, allocator retention over hours (the 1.34-1.69 anon/live was measured at
   60 s). Deliverable: a `run_memprof_local.sh`-style script with an hours cap, and its verdict in
   this file.

   **Done, on the fifth attempt, and the four failures were the deliverable** (2026-08-10, the
   run-by-run record is in "The soak rig is built" below): the 24-hour run at 2 GiB / offered
   20 k eps / retention 30 m completed with anon quarter-means 1330 → 1367 → 1270 → **1112 MiB**
   (peak 1967), sidecars flat at 118–122 MiB, parts flat at ~320, disk oscillating 2.2–3.5 GB,
   WAL bounded, row-group-cache gauge pinned at 153 MiB through a day of evictions, and 568,721
   queries at 12 errors (0.002%) with response p95 633 ms. What it took: the sidecar bloom
   eviction, the declared budget, and jemalloc — each forced by the previous attempt's corpse.
   Two honest numbers out of the verdict (`PASS_BEHAVIORAL_ONLY`): the engine's sustained
   capacity at 2 GiB is **~18.6 k eps** — 6.9% of the offered 20 k was throttled with 429s, which
   is backpressure working, and the push p95 target is missed in the same breath — and
   `part_meta` is the one remaining GROWING row, 8.1 → 13.0 MiB over the day, which is
   `PartMeta::streams` (polish item 2's residue) presenting its bill in slow motion.

   **And again on the production build, with faults, on 2026-09-08: PASS**
   (docs/SOAK_24H.md). Engine `c0247b0`, mimalloc, 2 GiB / 20 k eps / 24 h,
   retention 30 m, collecty and traces and metrics all on, seven fault windows
   all recovered. Anon quarter-means 1001.7 / 1005.2 / 965.5 / 993.8 MiB, peak
   1708.3, no OOM. That document also carries what the run cost to read
   correctly: the rig's own disk had to be trimmed hourly before its latency
   meant anything, one metric read defect it found and the fix, and two
   readings that looked like defects and were the harness counting an arranged
   outage as one.
2. **The unbounded residents** — sidecar + `PartMeta::streams` grow with part count and nothing
   evicts them (~380 KB/part measured; VISION already scopes the fix: sidecars are durable in
   `index.bin`, so eviction is a re-read; stream identity can be a fingerprint). The in-flight push
   body bound is in the same class (recorded, unimplemented). These become the first production
   incident the day retention is long.

   **The `streams` half of this may already be done, and its evidence is stale by nine hours.** The
   finding that promoted it — `part_meta` the one GROWING row of the 24-hour soak, 8.1 → 13.0 MiB
   with the part count flat — was measured on the run that ended 08-10 13:13. `00f9799` landed at
   08-10 21:41 and interns one `SharedLabels` per distinct label set across every part
   (`intern_stream_labels`), so a part's per-stream cost is now one pointer into a table shared
   process-wide, and the reader's second copy is gone. The hours since read consistently with that
   and not with a leak: in `lockorder-1h`, `part_meta` Q2 → Q4 is 12.4 → 14.5 MiB, ×1.17, while the
   part count goes 270 → 310, ×1.15 — the gauge is tracking parts, which is what it should do.

   So this item is gated on the relaunched 24-hour soak, where the part count is flat and a real
   growth term would have nowhere to hide. Building the fingerprint before that reads is building on
   a measurement the code has already moved past. The sidecar half was separately fixed and verified
   (`6615a04`, flat at 118–122 MiB through a day), and the push-body bound is untouched by any of
   this and still owed.

   **Both halves are closed now, and so is the item** (audited 2026-08-13). The relaunched soak
   answered `streams`: `parts` and `part_meta` carry the GROWING flag *together* off the same Q4
   bump, ×1.15 each, so the gauge tracks the part count the way a per-part structure should and the
   fingerprint the item scoped is not needed — `00f9799`'s interning did that work. The push-body
   bound landed in `3bde4d7` as `SIGNY_MAX_INFLIGHT_PUSH_BYTES`, charged in the middleware
   where the bytes are actually spent, and `conn8-1h` measured what it costs the push path: nothing
   (response p95 266.4 ms against the pre-bound day's 273.3, service p50 unchanged).
3. **One large-corpus bed run** — 10-100x the 150k-row dataset, as claim-scope validation rather
   than tuning: does the 1.4 ms-constant race hold when parts multiply and the cache's working set
   overflows 256 MiB? Published win or lose, per house rule.

   **Done at 10x, and it answers the question with a yes and a no** (`COMPARE_VERIFY_ROWS=1500000`,
   published as [`docs/COMPARISON_LARGE_CORPUS.md`](docs/COMPARISON_LARGE_CORPUS.md), 2026-08-12,
   load verdict gate PASS, nothing OOM-killed). The read path's two halves scale in opposite
   directions, and the split is exactly along the mechanism:

   | shape | lt p50, 150k → 1.5M | lt/VL, 150k → 1.5M |
   |---|---|---|
   | `metadata_rare` (the claim) | 0.23 → 0.39 ms | 0.19x → **0.06x** |
   | `json_field_rare` | 0.25 → 0.44 ms | 0.00x → 0.00x |
   | `trace_window` | 0.24 → 0.66 ms | 0.22x → 0.35x |
   | `label_only` | 26.3 → 67.3 ms (warm 15.9 → 69.4) | 0.88x → 0.53x |
   | `line_filter` | 3.52 → **53.4 ms** | 0.47x → **1.27x** |
   | `json_field` | 11.5 → **103 ms** | 0.91x → **1.33x** |
   | `sum(rate(...))` | 0.46 → 4.57 ms | 1.09x → **3.66x** |

   **The claim's own shape got stronger**: 15.4x cold and 17.8x faster warm than VictoriaLogs at
   1.5 M rows, against ~5x at 150 k. Ten times the rows do not make a bloom read ten times as much;
   they do make a column scan. The constant-time race the item asked about holds, and widens.

   **The sweep does not.** Three shapes now lose to VictoriaLogs. So "every log shape under
   VictoriaLogs on both passes" — the sentence the 2026-08-07 freeze was declared on — is true of the
   150 k dataset and not of this one; VISION now scopes it rather than leaving it standing, and the
   150 k table stays published beside the new one rather than being replaced. `label_only` shows the
   cause of the scanned half in one number: its warm pass went 15.9 → 69.4 ms, level with its own
   cold pass, which is the working set outgrowing the caches.

   Memory and disk at this size, for the record: ingest peak 1234 MiB against Loki's 2048 (its limit)
   and VictoriaLogs' 952; settled data 307 MiB/GB against 292 and 187; ingest 19,916 eps against
   19,853 and 19,935.
- [x] **`line_filter` degraded super-linearly and that is a scaling question, not a tuning one.**
  Ten times the data, **fifteen** times the time: 3.52 → 53.4 ms. A scan cost alone predicts ten.
  The distinction that decides whether the 2026-08-07 freeze covers this: making a fast shape faster
  is the optimization that is frozen, while an engine that grows worse than its input is a
  production-readiness property — it is the shape of a curve a customer's growth walks along, and
  the same reasoning that put the soak first puts this here. So: **diagnose, do not tune.**

  **Diagnosed, and most of the fifteen was never a degradation** (`benches/scan_scaling.rs`,
  2026-08-12). Three findings, in the order they killed each other's hypotheses.

  1. **Ten of the fifteen is a ten-times-bigger answer.** At `limit 20000` the bound never reaches
     the scan, so the query returns everything the window matches — and the window holds ten times
     more: **10,208 → 102,351 rows returned**, ×10.0. Per returned row the cost went 8.28 → 12.52 µs,
     ×1.5. So 15.2x = ×10.0 more answer × ×1.5 per row. Comparing those two numbers as if they were
     the same query was the reading error, and the per-limit table had said so all along.
  2. **The scan path's own scaling is sub-linear, and it is not the part count.** At `limit 100` the
     answer is pinned at 100 rows and the bed still read 8.8x (1.55 → 13.6 ms) while scanning *less*
     (426,064 → 288,295 lines). The obvious suspect was per-part or per-row-group overhead — and it
     is neither: the bed held 1.5 M rows in **two** parts (`part_count` read live off the container),
     each with the default ~8,050-row groups, so no layout inflation to blame. The new bench sweeps
     the same change with everything else held: 150 k → 500 k → 1.5 M rows at a fixed row group size,
     same 100-row answer, **4.82 → 9.36 → 9.40 ms** — ×1.95 for ten times the data, and *flat* from
     500 k on. The engine does not grow worse than its input.
  3. **Row group size is the real cost driver, and it runs the other way.** Same 1.5 M rows, same
     100-row answer, groups made larger instead of more numerous: 184 groups **9.46 ms**, 46 groups
     **26.5 ms**, 24 groups **35.5 ms** — 8× bigger groups is **3.75× slower**, because a group is
     the unit that must be decoded to read any of it. Two things follow: the 8192 default sits on the
     right side of that curve, and "use bigger row groups" was never much of a lever anyway — the
     format caps a group at 65,536 rows, since its bloom windows are 1024 rows each and the selection
     mask is a `u64` (`part/reader.rs`, "exceeds the 64-window limit").

  So the item closes as a **documented design cost, not a defect**: what grows with the corpus is the
  answer, and what the scan pays per unit of window flattens out. `json_field` (9x for 10x) and `rate`
  (10x for 10x) were within a scan cost before this and are unaffected by it.
- [ ] **The residue, named so it is not mistaken for the above being incomplete.** The bed's
  `limit 100` ratio is 8.8x where the controlled sweep of the same change is 1.95x, and that gap is
  not the scan algorithm — it is everything the bench holds still and the bed does not. The bed issues
  24 *different* windows per shape against one process, so each query lands on different row groups
  with no locality, while the bench repeats one query and criterion warms it; and the bed's numbers
  include the HTTP round trip, the LogQL parse and `data.stats`. Both are plausible and neither is
  measured. Worth an hour only if a real deployment reports it — the engine's scaling curve is the
  question this item existed to answer, and that one is answered.
- [x] **Loki's `__stream_shard__` needs an exemption decision, or there are no Loki ratios above
  150k rows.** 32 of 168 answers disagreed at 1.5 M rows, every one of them for the same reason: Loki
  attaches `stream:__stream_shard__` once a stream is large enough to shard, with identical row counts
  on both sides and no label missing on the signy side. It is engine-internal, the same class as
  the already-declared `detected_level`, and the agreement gate withheld every Loki timing ratio in
  the run — which is the gate working. Declaring it by name, with counts, in
  `src/bin/load/matrix.rs`'s digest is the small change; deciding to is the judgement, and it should
  be made deliberately rather than to make a table look complete.

  **Declared by name, and the run that followed reads 168 of 168** (2026-08-12, the bed re-run at
  `COMPARE_VERIFY_ROWS=1500000`). The reasoning, recorded where the constant is: Loki derives the
  label from its own sharding decision, this bed never pushes one, the row counts were already
  identical on both sides and no other label differed — which is `detected_level`'s class exactly.
  The exemption is reported with its count like the other two: `stream:__stream_shard__` on **58**
  answers, `stream:detected_level` on 144.

  Three guards against this being a table-completing exemption rather than a decision. It is by
  **name**, not by prefix — pinned by a test that `__error_details__` and a lookalike
  `__stream_shard` are still digested, so whatever Loki adds next is not swept in without anyone
  deciding. The document states **what the drop could hide**: the label is part of a stream's
  identity, so it would also hide a difference in an *unaggregated* metric answer's series set —
  which is why the matrix asks for `sum(rate(...))`, and why row counts, the labels only one side
  had, and answer order all stay in force beside the drop. And the alternative was rejected on the
  bed's own rule rather than on taste: turning Loki's stream sharding off in `compare/loki-config.yaml`
  would have made the difference not happen, but that file's rule is that tuning choices stay at
  Loki's default, and sharding is Loki's tuning.

  **What the withheld column turned out to be hiding: nothing bad, and that is worth saying.** With
  the agreement gate satisfied, the Loki ratios print for all seven shapes and signy wins every
  one of them at 1.5 M rows — 0.00x–0.26x, `label_only` 0.12x, `line_filter` 0.26x, `json_field`
  0.21x — and the claim's own shape reads **1470x faster than Loki** cold. The two shapes that lose
  to VictoriaLogs still lose; withholding was never protecting a bad number, it was the gate refusing
  to price answers it had not checked, which is what it is for.

  The run also **reproduces the previous one** on the axis that matters, independently seeded from
  the same seed but built five commits later: `metadata_rare` against VictoriaLogs 15.71x cold /
  15.61x warm against the earlier 15.4x / 17.8x, `line_filter` 1.25x against 1.27x, `json_field`
  1.20x against 1.33x, `rate` 3.59x against 3.66x. The scanned shapes' losses are not run-to-run
  noise.
- [x] **A second corpus needed a second artifact directory, and for a day it did not have one.**
  Found while setting the re-run up, not by looking for it. `compare/run.sh` copies its JSON to
  `docs/artifacts/m9` by default and the generated document hardcoded that same path in its prose —
  so the ten-times run of 2026-08-12 wrote its artifacts over the 150 k run's, and
  `docs/COMPARISON.md`, which says on its own first line that it was generated on 2026-08-06 from
  revision `9a5ad8e`, was citing a directory whose `bed.json` said `238e34e` and 2026-08-12. That is
  the exact failure the header warns about — "one cited artifact did not exist and another disagreed
  with the document citing it on both build revision and verdict" — reintroduced by the document
  naming its own artifacts as a constant. The 150 k artifacts are restored from `3363d61^`, the link
  is now passed in by the script that does the copying (`COMPARE_ARTIFACTS_REL`), and the ten-times
  run has `docs/artifacts/m9-10x`. Left open only as a reminder that the copy-and-cite path has no
  test: nothing fails if a document cites a directory that was written by a different run.

  **Tested now** (2026-08-14). `compare_report::tests::a_published_document_cites_the_run_that_wrote_its_artifacts`
  reads every `docs/*.md` carrying the generation header, parses back the run it names and the
  directory it cites, and holds three things: the cited `bed.json` must agree with the document's
  own header on `generated_at`, `revision` and `branch`; no two documents may cite one directory;
  and no directory under `docs/artifacts/` may go uncited. The last is not decoration — it is what
  keeps the first two honest, because a reworded sentence that stopped parsing would otherwise leave
  the test passing over an empty set, and instead it leaves both directories orphaned and red. The
  two sentences it parses are constants shared with the generator that writes them
  (`HEADER_SENTINEL`, `ARTIFACTS_SENTINEL`), for the reason the query-memory refusal's
  `EXHAUSTED_PREFIX` is a constant: a literal spelled twice is a check that stops checking the day
  someone edits the prose, which is how these two came apart in the first place. Each of the three
  rules was verified by breaking the tree — a drifted revision, two documents on one directory (the
  2026-08-12 failure, replayed), and an uncited directory — and the duplicate rule was made to fire
  on its own with two documents describing the *same* run, since the first mutation tripped the
  identity check before it ever reached the second rule.

  **And the test found a bigger one on its way in: the published document could not be regenerated
  from its published artifacts.** `compare/run.sh` copied an enumerated 18 of the 29 files a run
  writes, and the missing eleven were not spares — the per-limit matrix JSON behind the document's
  own limit-sweep table, both buildinfo files behind its build table, and the startup log it prints
  verbatim. Pointed at `docs/artifacts/m9`, the directory the document tells a reader to point it
  at, `compare_report` aborted on the first missing file. So "every number below comes from the JSON
  in `artifacts/m9/`" was false for three of its sections, for the same reason as the original
  defect: a hand-maintained list that drifts from what the report reads. The list is gone — the
  script copies the run's whole output directory, which cannot drift — and `docs/artifacts/m9` is
  completed from `target/compare`, which still holds that run and is byte-identical to the published
  directory on all 18 files it already had. The document now regenerates from its own artifacts, and
  **not one number moves**: the only difference against the checked-in file is two prose paragraphs
  the generator gained in later commits.

  *Two things this deliberately does not do.* `docs/COMPARISON.md` is **not** regenerated with
  today's binary, even though it would now succeed: the generator's prose has since grown a
  `__stream_shard__` exemption that the 150 k run's digest never applied, so regenerating would
  print a description of three exemptions over answers computed with two. That also settles why the
  test is identity-based rather than a byte-for-byte regeneration check — the prose moves with the
  generator while a published document is fixed to its run, so the two are only equal on the day of
  the run. And `docs/artifacts/m9-10x` stays incomplete: its eleven files were overwritten and
  cannot be recovered, and reconstructing them from the numbers in the document would invert the
  direction the whole scheme rests on. It completes itself at the next `COMPARE_VERIFY_ROWS=1500000`
  run, and asserting completeness for every cited directory is the check that lands with it.
4. **The review gate list** — worked through on 2026-08-12, in three passes with a finding in each.

   **The gates, audited item by item against the code.** The list had drifted *both* ways. Done and
   still marked open: per-tenant throttles and quotas (`tenant_quota.rs`, `default_tenant_max_streams`,
   applied on both transports), the default bind to the trust boundary (loopback, with a startup line
   naming the widening variable), startup retry for transient object-store failures
   (`with_object_store_retry`), the non-stdin abort path (`SIGUSR1`), and N3's fragmentation
   measurement, which exists as an asserting test. Ticked and since deliberately undone: **N5's part
   format version field**, removed in `f0da5bd` — left in the list as a visible withdrawal, because a
   gate met and then unmet on purpose is precisely what a later reader re-opens. Two reframed from
   tasks into open *decisions*: tenant-labeled metrics would hand an unauthenticated endpoint the
   tenant list and make cardinality a function of the customer count, and P2-7's histograms shipped
   while its per-endpoint dimension did not. Every line now cites where to look.

   **ARCHITECTURE.md against behavior.** Two statements were false rather than merely dated. It said
   the differentiator "has never been measured against Loki" — there are two generated comparison
   documents with a row-equality gate. And it said "with any pipeline stage the scan limit becomes
   `usize::MAX`, which is invariant III's worst violation": the log path passes the request's own
   `limit` now, beside a scan-row budget, a byte ceiling and a memory reservation. The one remaining
   `usize::MAX` is the metric path, which has no `limit` to stop at because every matching row
   contributes to a sample, and is bounded by `max_query_scan_rows`/`max_metric_rows` instead — a
   different thing wearing the same constant. Added what was younger than the document: sidecar
   eviction in halves, the row-group and narrow-pass caches with their budgets, the row group as the
   decode unit with the 3.75x measurement and its 65 536-row ceiling, the `writer_epoch` fence, and
   the two lifecycle locks with the order this week's stall taught them.

   **The refusal paths, and one of them was losing data by design.** Consistent where it counts —
   every 429 carries `Retry-After`, and limit messages all name what was exceeded and its number. But
   `ingest_error_to_status` mapped both `429` and `413` to gRPC `RESOURCE_EXHAUSTED`, and the OTLP
   specification's retry table makes those opposite instructions. A limit violation is permanent for
   that batch, so it is `INVALID_ARGUMENT` now — the code the specification recommends for
   non-retryable, which tells a collector to split or drop rather than loop on identical bytes. Two
   tests moved with it.
- [x] **The other half of that finding: gRPC backpressure does not say "come back", it says "give
      up".** The specification is explicit — a client "SHOULD interpret `RESOURCE_EXHAUSTED` as
      retryable only if the server signals that recovery is possible", signalled by attaching
      `RetryInfo`; without it, non-retryable, and a client "SHOULD drop the telemetry data". This
      server attaches nothing. So on HTTP a throttled push is told `Retry-After: 1` and holds its
      data, while the identical refusal over gRPC may be dropped — against the architecture's own
      premise that a client can only hold data back if the server declines it. Fixing it means
      attaching `RetryInfo` with `backpressure_retry_after`, which needs either the `tonic-types`
      crate or ~20 lines defining `google.rpc.Status`/`Any`/`RetryInfo` against the `prost` already
      in the tree — **a dependency question to put to the user, not a call to make quietly.** Until
      then the two transports do not carry the same instruction, and that is written into
      `ingest_error_to_status`'s doc where the next reader of that mapping will find it.

      **Fixed, on `tonic-types` — the user's call, and the reason for it is the failure mode, not the
      line count** (2026-08-12). Hand-defining `google.rpc.Status`/`Any`/`RetryInfo` against the
      `prost` already here is the same ~20 lines it was, but a wrong `type_url` or a mis-encoded
      details field round-trips perfectly through a test that decodes with those same definitions —
      it would pass green and only a real collector would notice. tonic's own sibling crate at
      tonic's own version (`tonic-types = 0.14.6`, two new packages with `prost-types`) encodes it
      the way the ecosystem decodes it. The no-dependency-shim precedents — `malloc_tuning`,
      `posix_fadvise` — were syscalls, where the contract is checked by the kernel; a wire format
      has no such witness in-process.

      Three things it turned out to be, beyond the one line the item described:

      1. **The delay was never missing, only dropped.** `IngestError::retry_after` already carried
         it — `backpressure::overloaded` from config, `tenant_quota` computed from how far over the
         rate the tenant is — and `ingest_error_to_status` matched on `error.status` alone. So the
         fix is that the gRPC rendering stops discarding the field the HTTP rendering has always
         used, not that a number was invented for it.
      2. **Three more sites were building the same bare status by hand.** `IngestGate::check_grpc`,
         `IngestGate::admit_body_grpc` and `TenantQuota::check_grpc` each called
         `Status::resource_exhausted(error.message)` directly, so fixing only the mapping would have
         left the in-flight-body ceiling and the tenant rate still telling collectors to drop. All
         three now go through the one mapping.
      3. **The two transports had to be made to say the same number, not merely both say one.**
         `Retry-After` has whole seconds and nothing finer, so a `RetryInfo` of 1.7 s beside a header
         of `1` is two answers. `backpressure::retry_after_seconds` is now the single conversion both
         use, and it rounds **up** where the header used to truncate — 1.7 s truncated to 1 s sends
         the client back before the server's own arithmetic says it may, which just spends another
         refusal.

      Pinned by four tests: the memtable and tenant-rate refusals each assert their `RetryInfo`
      (`log_ingest`), the in-flight ceiling asserts its own (`backpressure`), and
      `a_throttled_push_names_the_same_delay_on_both_transports` drives one error down both
      renderings at 1 s / 0.3 s / 1.7 s and compares the number rather than asserting each side
      alone. `a_permanent_refusal_carries_no_invitation_to_retry` pins the split from the other
      side: `413`/`400` stay `INVALID_ARGUMENT` and acquire no `RetryInfo` even when a producer
      attaches one.

### The soak measured the engine and called it the stack (2026-09-04)

The 24-hour run above is the engine's day, not the product's. Three gaps, each of which would have
let a broken shipped path finish a soak with a clean verdict:

1. **collecty was never in it.** The harness framed its own one-record collecty batch and posted it
   to the collect route, so the collector's intake, its zstd, its segment rotation, its disk queue,
   its retry and — the one that decides whether a crash duplicates data — the
   `x-collecty-sender`/`x-collecty-segment` resume had never carried load for a minute, let alone a
   day.
2. **The trace leg was off** (`SIGNY_LOAD_OTLP_EPS=0`), and even on it only wrote. The four trace
   read routes had never been asked anything by any load run, so an engine that stored no spans at
   all would have passed.
3. **Eleven read routes were never exercised.** The harness drives six log query shapes and five
   metric ones. The autocompletes, the tail, the deletion surface, the trace timeline and the admin
   lifecycle were outside every run, and the soak's verdict is memory trends, which say nothing
   about whether a route still answers.

What was built for it, all of it in place and smoke-tested on 2026-09-04:

- `SIGNY_LOAD_PUSH_ADDR` splits the harness's two addresses — writes to a collecty as the bare
  export on the signal's own path, reads to signy, which has the only read surface. A `200` then
  means the collector has the export, not the engine, so the run waits for the queue to drain
  before it scrapes its end metrics and **fails if that wait did not settle**: accounting taken
  before the backlog arrives reads as loss.
- The trace leg keeps every tenth trace it exports and asks for it by id a lag later; the span
  count has to match. A first 404 is asked again rather than counted, because a backlog in front of
  the engine is not the engine losing a trace.
- `scripts/soak_probe.sh` walks the whole surface in `docs/QUERY_API.md` plus the admin routes every
  five minutes and writes a row per probe per round. The artifact is a per-route success table over
  the run — which is what "everything still works" has to mean to mean anything.
- The rig restarts the collector, restarts the engine, takes it down for minutes, kills it outright,
  and restarts it again with the run's parts on disk, at fractions of the run. The collector is
  never killed outright: an open segment has not been fsynced, so losing it would be collecty
  behaving as documented and this run could not then tell that from a defect.

Two findings out of the half-hour rehearsal at the real rate (`premeasure`, 20 k eps, 2 GiB,
retention 30 m, no faults), both of which decide how the day is run:

- **collecty converges, it does not leak.** Quarter means 69 → 97 → 106 → **111 MiB**, peak 122,
  and the growth rate fell 28.6 → 10.9 → 6.6 → 1.4 MiB/min across the run. It is above what
  `CONFIGURATION.md` implies — the configured ceilings are 64 MiB in flight plus an 8 MiB segment —
  and the gap has the shape of allocator retention rather than a buffer: collecty builds and drops
  a zstd context per closed segment, on a glibc heap with one arena per worker thread. So the first
  lever is `MALLOC_ARENA_MAX`, then `COLLECTY_MAX_INFLIGHT_BYTES`, and neither is code. Measured,
  not tuned: 122 MiB is the number to beat.
- **The memprof build cannot hold 2 GiB at this rate for half an hour.** OOM-killed at t=1797 s,
  anon peak 1957 MiB, cgroup peak 2050 against a 2048 limit, final anon/live **3.15**. That is the
  instrumented glibc build, not the shipped one — `src/main.rs` ships mimalloc — so a day has to be
  run with `SOAK_FEATURES=` (production) and the memory attribution columns given up. Worth
  recording as its own result: the instrumentation's own footprint is the difference between
  finishing and being killed at this rate.

And one that is what the whole exercise was for: **every route answered every round.** Six probe
rounds over the half hour, twenty-eight routes each, zero failures — the first time the tail, the
autocompletes, the deletion surface, the trace timeline and the admin lifecycle have been asked
anything by a run longer than a test.

### The soak rig is built, and its first four minutes contradicted the gates (2026-08-08)

`scripts/run_soak_local.sh` is the deliverable: the memprof rig pointed at hours — a native
cgroup v2 scope at 2 GiB, the bed's corpus at 20 k eps with queries at 5 eps, retention **on**
(30 m period / 60 s interval / 5 m grace by default; the probes below shortened it to 5 m so it
would fire inside a smoke), the data directory on disk rather than tmpfs (tmpfs pages are charged
to the writing cgroup as shmem and, with swap off, hours of parts would manufacture an OOM the
disk engine does not have), a disk-space guard, and a verdict of quarter-by-quarter trends per
resident rather than one peak. The 24-hour run it was built for has not run yet: the smoke runs
found four things first, three of them the soak's own questions answering early.

- **Sustained load does not fit 2 GiB: OOM-killed at t≈150 s (memprof build) and t≈255 s
  (production build), retention never having fired.** Every green 2 GiB gate measured a ~60 s
  ingest burst plus a settle; the soak's anon passes those gates' 1817–1832 MiB passing peaks at
  about the moment their workload would have ended, and keeps climbing (quarter means 496 → 1147 →
  1308 → 1526 MiB, peak 1921) until the kill. "Survives ingest" was never "fits a container" —
  [`docs/MEMORY_BUDGET_GATE.md`](docs/MEMORY_BUDGET_GATE.md) said exactly that — and the soak
  measured the difference in its first four minutes.
- **It is a ratchet, not a leak.** The 25-minute memprof run at 8 GiB: the live sum oscillates
  450–1500 MiB with no trend — merge saws 0↔600 MiB, query stays ≤300 MiB, the row-group cache
  sits flat at ~250 MiB, sidecars at ~100–200 MiB with retention deleting parts — while
  glibc-retained free ratchets 915 → 1078 → 1627 → 2628 MiB and never comes back down. anon steps
  1710 → 2525 → 3157 → 3295 MiB with shrinking increments: it converges on the historical maximum
  of *coincident* live spikes times fragmentation, ~3.4 GiB on this workload, which is why 8 GiB
  survives and 2 GiB cannot. Final anon/live **5.30**, against the 1.34–1.69 the attribution doc
  measured over 60 s — the tuning's trim threshold returns nothing here because the retained bytes
  are not at the top of any arena.
- **`MALLOC_ARENA_MAX=1` survives ten minutes at 2 GiB** (anon peak 1322 MiB, queries 2507/0
  errors) and re-inflicts exactly the cost that made it non-default: the WAL backlog climbs
  2.9 → 50.7 MiB and is still rising where the default's stayed near 1.4 MiB.
- **The same sustained workload was then pointed at the other two engines, same 2 GiB, same
  corpus, seed, rates and duration, containerized from `compare/docker-compose.yml`.** Loki was
  OOM-killed at **t≈112 s** — before signy's production build's 255 s — having delivered
  3.7 k of the offered 20 k eps overall. VictoriaLogs delivered **19,973 eps for the full 600 s,
  12 M events, zero errors, verdict PASS, anon flat between 420 and 554 MiB** — no ratchet at
  all. So the condition is not inherently beyond a 2 GiB container: an engine of this class holds
  it in half a gigabyte. signy outlives Loki and the allocator ratchet is the distance to
  VictoriaLogs. Why each, measured rather than guessed: VictoriaLogs reads the cgroup limit at
  startup and declares 60 % of it as its own budget (`vm_available_memory_bytes` 2147483648,
  `vm_allowed_memory_bytes` 1288490188 under a 2 g container) — M10's "declared memory budget",
  built in and automatic — while Loki, also Go and therefore without the glibc layer, died of a
  live working set whose defaults derive from no limit at all (anon 579 → 1632 MiB in 80 s).
  signy shares Loki's problem and adds glibc's on top.
- [x] **Retention and merge race on part files.** Merge selects a group, retention whole-part
  deletes an input, and `rewrite_group` fails with ENOENT — surfaced as an ERROR-level "merge
  iteration failed", four times in twenty minutes at the aggressive 5 m retention. The outcome is
  benign (the group is skipped and the next tick re-lists) but it is wasted work wearing an
  incident's log level, and it fires at any retention setting given enough hours.

  Fixed: the error path now looks at the inputs before deciding the log level — a missing
  `index.bin` is the file only retention removes (cache eviction legitimately reclaims
  `data.parquet` alone, so a missing body stays loud), and that case is a DEBUG skip with no error
  recorded. Pinned red-before/green-after by
  `merge::tests::an_input_deleted_mid_merge_is_a_skip_and_not_an_error`.
- [x] **~1 % of queries 504 under sustained 20 k eps with merge and retention active** (48/4,926
  at 8 GiB; first: `| json | level="debug"` timed out). The load gate's p95 passes in minutes-long
  runs; the sustained tail is a different number.

  **Gone with the retention lock order, and four runs since say so** (audited 2026-08-13). The
  timeouts were the world-stop: a query that arrives during a 20–50 s freeze is answered after the
  harness's 60 s request timeout has already given up on it. Since `ca32ee5` every run has answered
  every query it did not refuse on purpose — `soak-24h-lockorder` (a day), `conn8-1h`, `conn32-1h`
  and `phase-1h` (an hour each) return **zero `500`s and zero `504`s** between them, against the
  day-long pre-fix run's 856 and 17. Query response p99 is 519–640 ms across the four and the worst
  single answer in any of them is 3.8 s. The `400`s that remain are one refusal working
  (`query exceeds the maximum of 1000000 scanned rows`), which is not this item.
- **Fixing the mmap threshold (`MALLOC_MMAP_THRESHOLD_=131072`, arenas left at 4) collapses the
  ratchet and still loses**: anon/live 5.30 → **1.60**, survival 150 → 502 s, then killed anyway.
  With retention mostly gone the remaining arithmetic is plain: the live sum's spikes reach
  1.1–1.5 GiB (merge saws to ~600 MiB, the cache holds 256 MiB, query ~300 MiB, sidecars
  100–165 MiB before retention's steady state) and glibc still retains 0.65–1.2 GiB of
  small-chunk heap, and the two together brush 2 GiB. The old rejection of this knob dated from
  the pre-streaming-merge regime and did not survive remeasurement — but no allocator knob makes
  2 GiB hold this workload while the live spikes are allowed to coincide. That is
  [`merge_max_memory_bytes` must derive from the declared budget] and its M10 siblings, now with
  the measurement that makes them urgent rather than pending.
- [x] **The 24-hour run needs a configuration that can survive it** — it has one now, and it is the
  declared rig itself: the user chose to build the VictoriaLogs answer rather than pick a bigger
  container, and `SIGNY_MEMORY_BUDGET` landed the same day (`dcdd418`, M10 Phase B has the
  record). The 2 GiB / 20 k eps / retention-on configuration that died at 150–255 s **survives its
  600 s probe at anon peak 1480 MiB**, `memory_gate --budget 2GiB` is UNDER_BUDGET, and 8 GiB
  throughput is unchanged. The 24-hour run itself is still owed.
- [x] **The sidecars are evictable now, and the hour that used to be impossible passes whole.**
  `part/bloom_cache.rs` (`6615a04`): the bloom half of every sidecar — the megabytes — lives under
  one process-wide LRU byte budget (`SIGNY_SIDECAR_CACHE_MAX_BYTES`, derived at 10% of the
  declared budget, unbounded when budgeting is off), evicted across parts and re-read from
  `index.bin` on the next pruning query; the stream index — the kilobytes — stays resident so the
  infallible metadata paths stay infallible, and a matchers-only query still never touches blooms
  at all. Answer equality under eviction is pinned by test (a one-byte budget forces every open to
  evict every other part; the filtered answer must not change), and byte accounting survives
  reinstall races and reader drops by construction. Verified on the killing configuration —
  2 GiB, 20 k eps, retention 30 m, one hour: **survived with verdict PASS, zero query errors in
  14,998**, sidecar 630-and-climbing → **120 MiB flat**, parts steady at ~330 after the retention
  peak, anon peak 1730 MiB (84%). And the read tail moved with it: overall query response p95
  **33.3 s → 651 ms** over the hour — most of the "collapse under churn" below was the unbounded
  part/sidecar backlog. p99 16.9 s remains, which is the stall item's shape, not this one's.
- **The 24-hour run was launched on that configuration and died at t≈1834 s, and the verdict table
  names the killer in one line: the sidecars.** At the real 30 m retention (the probes above ran 5 m,
  which is why they passed), no part is deleted for the first ~35 minutes, the part count reaches 320
  and `signy_part_sidecar_resident_bytes` climbs monotonically 87.6 → 266.5 → 447.4 →
  **630.2 MiB** — ~2 MiB per part now, not the ~240 KB the old attribution measured, the 0.1% blooms
  having widened them — while every budgeted term holds (cache flat at 153 MiB, wal_backlog 2 MiB,
  anon peak 2009). Steady state at 30 m retention would be ~600+ live parts, which is ~1.2 GiB of
  sidecar in a 2 GiB container: **the soak cannot run a day at any real retention until the sidecars
  are evictable** — M10's "Sidecars inside the budget", VISION's scoping (durable in `index.bin`,
  eviction is a re-read), promoted from deferred to blocking by this run.
- **The second 24-hour attempt died at t≈8653 s with every gauged resident flat and anon creeping
  ~130 MiB/hour, and the 90-minute memprof diagnosis named the creep**: live oscillates 514–740 MiB
  with no trend while glibc-retained free climbs 846 → 1042 MiB (anon/live 2.94 at the end) — the
  middle-of-heap chunks the fixed trim threshold cannot release, accumulating across hours. Answered
  with `malloc_trim(0)` on a timer (`48fc971`, `SIGNY_MALLOC_TRIM_INTERVAL`, 60 s default, `off`
  disables): the one glibc call that `MADV_DONTNEED`s free pages wherever they sit in an arena. The
  relaunched 24-hour run is its verification.
- **Third attempt: t≈14,568 s (4.05 h), the trim halved the creep and the residue still kills.**
  Every gauge flat again (sidecar 120, parts ~315, cache 153, disk 2.40 GB, wal_backlog 4.7 MB,
  queries 0 errors in 60,654), anon slope ~130 → ~68 MiB/hour between the two attempts — the timer
  works — and the peak still reached 2015 MiB in four hours. What trim cannot return is a free chunk
  on a page it shares with a live one, and with four arenas the free space stays scattered across
  four heaps. The next and last glibc lever is `MALLOC_ARENA_MAX=1` — consolidation is what makes
  whole pages free — whose measured cost (flush cadence, WAL backlog 8 → 50 MiB) predates the budget
  and the streaming merge, so the fourth attempt runs with it and the backlog column is the thing to
  read. If the residue survives one arena too, the remaining option is the one the M10 item named:
  an allocator whose heap decays (jemalloc/mimalloc), which is a dependency decision to bring to the
  user, not a knob.
- [x] **The read tail under sustained churn — mostly explained, one residue left.** The collapse
  (p95 33.3 s at 2 GiB budgeted / 78.0 s at 8 GiB default, in every long pre-eviction run) was
  substantially the unbounded part/sidecar backlog: with the blooms evictable and parts steady
  under retention, the same hour reads p95 **651 ms** and the load verdict is a full PASS. What
  remains is the tail's tail: **p99 16.9 s, max 26.6 s** — the ~20 s query-counter stalls observed
  beside merge in the memprof run. That is a scheduling/stall question now, not a growth one.

  **It was not scheduling. It was the page cache against the cgroup limit, and the first guess cost
  a fix that stays anyway** (2026-08-10, `ad40a5d` and `7b7f6be`). The diagnosis in the order it
  happened, because the order is the lesson:

  1. The lock-convoy hypothesis was half right. Lifecycle writers did park behind the query tail on
     the fair lock, and `write_without_convoy` fixes a real hazard — kept, pinned by test — but the
     freeze did not move.
  2. The server log said it was not a query problem at all: through the freeze the **whole server is
     silent for 52 s**, flush lines included, and what ends the silence is retention deleting files.
  3. `mem.csv` named the mechanism. The freeze begins in the second that `memory.current` reaches
     2048 MiB (the limit) and ends in the second that the deletion drops `file` from 1099 to
     677 MiB. It is the kernel's direct-reclaim stall at a full cgroup, not a lock.

  The fix: `posix_fadvise(DONTNEED)` on `data.parquet` and `index.bin` right after the fsync that
  makes them durable — pages already clean, dropped deliberately so the write stream cannot ride
  `current` into the wall. The WAL carries durability, the first query on that part pays one extra
  read, and repeat access is the row-group cache's job, which has a budget. Same
  no-dependency-shim shape as `malloc_tuning`.

  **Verified over the hour, and it is half a fix.** `convoy-1h` → `fadvise-1h`, judged on the
  server's own query counter rather than client latency: freeze total **111.1 s → 50.1 s**, longest
  **51.6 s → 20.9 s**, query errors **4 → 0**, anon peak **1465 → 1170 MiB**. But five freezes
  remain and `memory.current` still reaches the limit, and the second-by-second window says why:
  the write side is dropped, and **queries scanning parts refill `file` to ~1.0 GiB** anyway. With
  anon's steady state ~750 MiB that is 1.85 of 2 GiB, so any burst hits the wall. The anon climb
  *inside* a freeze (718 → 924 MiB) is the memtable backlogging while flush is stalled — a
  consequence, and the `rows=325500 parts=6` flush right after the silence is that backlog draining.
  The structural cause is that the declared budget takes 60% of the limit while treating the page
  cache as free.

  **Closed on the item's own terms** (audited 2026-08-13). The tail's tail was **p99 16.9 s, max
  26.6 s** when this was written; the day on the fixed lock order reads **p99 639 ms, max 3.1 s**,
  and the three hours since read 520–561 ms and 1.4–3.8 s. The residue this item held open was the
  ~20 s stalls beside merge, and those are the retention lock order two items below — the item that
  found them is closed by the item that named them, not by them going unexplained. The last
  sentence above is worth keeping as a standing caution rather than as an open task: the declared
  budget still treats the page cache as free, which is why `posix_fadvise` on a written part is load
  bearing rather than an optimization.
- [x] **The stall's remaining half, and the sampler now measures it instead of inferring it.**
  `run_soak_local.sh` carries five more columns, all cumulative: PSI `some`/`full` stall
  microseconds, `pgscan_direct`/`pgsteal_direct` (direct reclaim, as against kswapd's, which costs
  the workload nothing) and `workingset_refault_file`. The verdict grew a stall table — every
  freeze the query counter shows, longest first, each with the reclaim deltas for its own window —
  so a run judges itself on the quantity this item is about. Episode detection reproduces the
  hand-computed numbers on both existing runs exactly. `SOAK_MEMORY_HIGH` is new beside it: set it
  and reclaim is throttled and gradual below the hard limit instead of a cliff at it.

  The next two runs are the free ones, no code, one variable each: ① the declared budget at 50%
  rather than 60% (`SIGNY_MEMORY_BUDGET=1G`) — is headroom the mechanism? ② `memory.high` at
  1800M — is the freeze the cliff, or the reclaim itself? A run is an hour because the freezes only
  appear after t≈1980 s, once the disk is past ~2 GB and retention is deleting.

  **① answered no twice over, and the second no is the bigger one** (`budget-50pct`, 2026-08-11).
  Headroom is not the mechanism: at 50% the freezes get *worse*, 5 episodes / 50.1 s → **15
  episodes / 134.0 s**, and anon rises rather than falls (quarter means 923 → 1307 MiB, peak 1548
  against 1170). The reason is visible in the same run: the smaller row-group cache makes queries
  re-read, 1.93 M file refaults over the hour ≈ 7.5 GB, and the allocator churns for it. A budget
  fraction is a trade, not a free win, and this direction stops here per the measurement gate.

  And the new columns overturned the diagnosis the fadvise fix was built on: **inside a 23-second
  freeze there are zero direct-reclaim pages and zero memory-PSI microseconds.** Over the whole run
  direct reclaim steals 4.2 M pages — that is just life at a full `memory.current` — but during the
  freezes, none. Nothing in the cgroup is asking for memory while it is frozen. Yesterday's "the
  freeze begins in the second `current` reaches the limit" was a correlation with a shared cause,
  not the cause. (The fadvise fix still earns its place — it halved the freezes and took anon peak
  down 295 MiB — but it was not treating what it was thought to treat.)

  What the freeze actually looks like, counted per second in the server log: from t=2701 to t=2722
  **not one line of any kind** — flush, merge, query, all of it — and the line that breaks the
  silence is retention deleting files. Logging stops too, and `server.log` is on the same
  filesystem as the data directory, which is the shape of a stall below the process: everything
  that touches the filesystem waits, including the log writer. The rig's disk is an SSD
  (`mq-deadline`) with 12.6 GB free, so it is not space — but the root filesystem it shares is 93%
  full, and ext4 at that fill level is a candidate on its own.

  **The item's deliverable is the instrument, and it did its job** (audited 2026-08-13). The five
  reclaim columns and the stall table are what killed the memory diagnosis — zero direct reclaim
  inside a 23-second freeze — and the `io.pressure`/`cpu.pressure` columns that followed eliminated
  the other two resources, which is what left a lock and led to the item below. Both candidate
  explanations this item was opened to test were answered **no** and the runs are recorded above.
  The ext4-fill suspicion in the last sentence was never needed: the freezes are gone at the same
  fill level, so the filesystem was not it.
- [x] **So the question is now which resource the threads wait on, and the sampler asks it
  directly.** Three more columns: `io.pressure` some/full and `cpu.pressure` some, and the stall
  table prints `mem_full` / `io_full` / `cpu_some` side by side for each freeze's own window.

  **② answered by eliminating all three, and then the clock gave it away** (`memhigh-1800m`,
  2026-08-11). `MemoryHigh=1800M` did what it says — `cgroup_peak` 1801 MiB, the hard limit never
  touched — and the freezes got *worse*: 13 episodes / **138.2 s**, longest 27.9 s. So the limit
  cliff was never the mechanism either. Inside the 27.9 s freeze: `mem_full=+0.0s`, `io_full=+0.7s`,
  `cpu_some=+0.0s`. Not memory, not disk, not CPU. The kernel says those threads were not waiting on
  any resource it accounts for, which leaves a lock.

  Then the arithmetic that should have been done on day one. **Every freeze in all four runs — 39
  of 39 — starts within ±0.5 s of a 60-second boundary**, and `SOAK_RETENTION_INTERVAL` is 60 s.
  The first freeze of every run lands at t≈1980–2100 s, which is when a 30 m period plus a 5 m grace
  first has anything to delete; before that `retention_once_at` returns at its empty-candidates
  check without ever taking an exclusive lock. And per-second in the log, **retention's own
  completion line is the last event of the freeze**, with the blocked merge's commit landing
  0.1–0.3 s after it. The freeze is not something retention interrupts. The freeze *is* retention's
  pass.
- [x] **The stall, named: retention holds the lock every query needs while it spins for a lock a
  merge rewrite is holding.** `retention.rs:243–285` takes `operation_lock` exclusively **first**,
  then spins for `deletion_lock` exclusively. `merge/scheduler.rs:154` holds `deletion_lock`'s read
  half for the whole rewrite of a group — deliberately, so a group's inputs cannot be deleted under
  it, and by its own comment "for as long as the group takes". So the sequence every 60 s once
  retention has work:

  1. retention takes `operation_lock` (write) — the lock queries take to read and flush takes to
     commit;
  2. it then spins in `write_without_convoy` for `deletion_lock` (write), which the merge rewrite
     holds;
  3. every query and every flush is stopped for the rest of that rewrite;
  4. and merge's own commit needs `operation_lock` (write), which retention is holding — so the two
     spin against each other until retention's `try_write` happens to win the gap between merge
     dropping its rewrite guard (`:154`) and taking its commit guard (`:310`). That gap is why the
     duration is 3–52 s and unpredictable rather than merely long.

  The measured cost of the world-stop: **28 seconds to delete one part.** The deletes themselves are
  nothing (1, 7, 11, 12 parts across the four longest freezes); the entire time is waiting, while
  holding the one lock that did not need to be held for it.

  Note also that the acquisition order is not the invariant the comment claims. Retention takes
  operation-then-deletion "the one order every double acquisition uses" — but merge takes
  deletion-then-operation (`:310` reads `deletion_lock`, then the replacement takes
  `operation_lock`). Retention is the odd one out, and the cycle is only survivable because
  `write_without_convoy` spins instead of parking.

  This also explains the two fixes that half-worked. `write_without_convoy` (`ad40a5d`) was the
  right instinct in the wrong place — the convoy is real, but it is retention's, not the query
  tail's. And `posix_fadvise` (`7b7f6be`) halved the freezes because less cache pressure makes merge
  rewrites finish sooner, which shortens the wait it never addressed. Both stay; neither was
  treating this.

  **Fixed and verified: the deleters wait on the deletion lock now** (`ca32ee5`). Deletion lock
  first, so the wait happens against the lock this work actually contends for; then the operation
  lock for the deletes and the retirement, which stay atomic together so no query can see a part
  registered with its files already gone. That is merge's order too, so the cycle is gone rather
  than survived by spinning. Cache eviction had the identical order with a *parked* acquisition,
  convoying new readers on top of it, and is reversed the same way. Pinned by
  `retention::tests::a_retention_pass_waiting_for_a_merge_does_not_stop_queries`, red before and
  green after.

  The hour on the killing configuration, against `fadvise-1h` as the baseline:

  | | `fadvise-1h` | `lockorder-1h` |
  |---|---|---|
  | freezes / total | 5 / 50.1 s | **0 / 0.0 s** |
  | query response p99 | 2804 ms | **525.6 ms** |
  | query response max | 20,803 ms | **1268.5 ms** |
  | 504s / 500s | 0 (`convoy-1h` had 4 × 504) | **0** |
  | throttled pushes | 3,322 | **0** |
  | achieved ingest | 19,903 eps | **19,994.6 of 20,000** |
  | push response max | 2435 ms | **988.3 ms** |

  Service and response percentiles now converge (524.9 against 525.6 ms), which is the arithmetic
  way of saying there is no queueing delay left to find. The 191 remaining query errors are all
  `400`s of one kind — `query exceeds the maximum of 1000000 scanned rows` — which is a refusal
  working, not a failure, and every run had them.

  **The throttling went with it, and that was not predicted.** 3,322 → 0 pushes answered `429`,
  offered rate finally achieved. The mechanism is the same one: through a 20–50 s world-stop the
  flush thread is frozen with everything else, the memtable fills, and backpressure refuses the
  clients — so "sustained capacity" was partly a measurement of the freeze. The one numeric target
  still missing is `push_response_p95_ms` at 255.7 ms against a 250 ms target, over by 5.7 ms.
- [x] **The published capacity number is now stale, and only the 24-hour run may replace it.**
  `docs/CONFIGURATION.md` and `docs/VISION.md` both carried "~18.6 k eps of an offered 20 k, 6.9%
  throttled with 429s", measured on the 2026-08-10 day-long soak — with this stall in it. One hour
  at 19,994.6 eps and zero throttling said the number understates the engine, but an hour is not a
  day and replacing a day's measurement with an hour's would repeat exactly the mistake that
  published it. So the day was re-run, and it settled it.

### The day again, on the fixed lock order, and it is the run this project has been trying to get (2026-08-12)

`soak-24h-lockorder`: 2 GiB, offered 20 k eps, retention 30 m, 24 h 00 m 05 s, `behavioral_pass`
true, 1,728,000,100 events accepted, `disk_guard=ok`.

| | `soak-24h` (2026-08-10) | `soak-24h-lockorder` |
|---|---|---|
| freezes | 5–8, longest 25–51 s | **1 × 3.2 s, and it is the run ending** |
| achieved ingest | 18,616.2 eps | **19,999.8 of 20,000** |
| throttled pushes | 1,195,480 | **0** |
| query response p95 | 633.4 ms | **427.6 ms** |
| query response p99 | 9425.5 ms | **639.5 ms** |
| query response max | 59,035.7 ms | **3123.6 ms** |
| push response max | 7966.5 ms | 5346.3 ms |
| statuses | 400 × 46,041, 429 × 700, **500 × 856, 504 × 17** | 400 × 4,273 and nothing else |
| answered `200` | 384,387 | 427,728 |

The single stall the table reports starts at t=86,402.1 of an 86,400 s run: it is the harness
stopping, not a freeze, and saying otherwise would be reading one's own instrument backwards. Every
5xx is gone — 856 `500`s and 17 `504`s to **zero** — and so are the 429s. The residents are flat
across the day the way a passing soak's should be: anon 1468 → 1574 MiB (peak 1841), sidecar
119.8 → 120.4, row-group cache pinned at 152.8 through a day of evictions, WAL file 34.6 → 34.3,
`wal_backlog` 2.9 → 3.2 MiB against a 1 GiB target.

The `400`s fell tenfold, 46,041 → 4,273, and the likely reason is worth recording as a hypothesis
rather than a finding: every one of them is `query exceeds the maximum of 1000000 scanned rows`, and
through a world-stop merge stops too, so parts pile up unmerged and a window's scan crosses more of
them. Fewer freezes, fewer oversized scans. Not proven here.

**`part_meta` is answered, and the fingerprint is not needed.** Both `parts` and `part_meta` carry
the GROWING flag, and they carry it *together* off the same Q4 bump: parts 294 → 337 is ×1.15,
`part_meta` 14.0 → 16.1 MiB is ×1.15. The gauge tracks the part count, which is what a per-part
structure should do — the interning of `00f9799` did the work the fingerprint was scoped for. Polish
item 2 reduced to the in-flight push-body bound, which `3bde4d7` then bounded — so polish item 2 is
closed, and with items 1, 3 and 4 already done the whole polish list is.

**What still fails, and it is one number.** `push_response_p95_ms` 273.3 ms against a 250 ms target,
over by 9.3%, which is why the verdict reads `PASS_BEHAVIORAL_ONLY` rather than a full pass. Every
other target passes: error rate 0.0, push p99 538.3 against 1000, query p95 427.6 against 2000,
throttled rate 0.0, RSS peak 2.0 GiB against 4, WAL backlog peak 21.2 MiB against 1 GiB.

**And what this run does not measure: the ceiling.** It sustained everything offered, so capacity at
2 GiB is now known to be *at least* 20 k eps and the upper bound is unmeasured. The honest way to
publish a capacity is to offer more until something refuses; that run has not been done.

### The ceiling, and the ladder that looks for it (registered before the runs, 2026-08-13)

A rate ladder at the published configuration — 2 GiB, retention 30 m / 60 s / 5 m, everything else
default — with the offered rate as the only variable: **30 k, then 45 k, then 60 k eps**, stopping at
the first rung that refuses. A rung is 45 minutes because retention first has something to delete at
t≈2100 s, and a capacity number measured before the deletes start is a number for a workload that
does not exist.

What counts as the ceiling, decided now rather than after seeing the numbers: **any of** achieved
ingest below 99% of offered, a non-zero throttled count, a 5xx, or an OOM kill. Each of those is the
server declining, which is the definition being measured; the p95 latency target is *not* one of them
— a server can be at capacity and slow, and conflating the two is how "~18.6 k eps" got published as
a capacity when it was partly a measurement of the retention freeze.

**The one deliberate departure from the published soak: 32 connections, not 8.** At 100 entries per
push, 60 k eps is 600 pushes/s, which over 8 connections is a 13 ms budget per push against a service
p50 of 12 ms — the client would saturate before the server did and the ladder would measure the
harness. `conn32-1h` measured the headroom directly: at 32 connections the queueing delay p95 is
6.8 ms, so the client is not the constraint anywhere on this ladder. It also means the ceiling found
here is **not** comparable to the 8-connection latency numbers, and the result has to say so.

The prediction, so the reading cannot follow the result: the writer task is the suspect — one
`sync_all` per batch, 96% busy at 20 k eps — but group commit is self-amortizing, so a batch at 3x
the rate carries roughly 3x the records for about the same fsync. If that holds, the fsync path is
*not* the ceiling and the ladder should climb until the memtable or WAL backlog gate refuses. If
instead throughput flattens near 20–25 k eps with `records/batch` failing to rise, the fsync rate is
the wall and the phase histograms will show it.

**The first rung refused, so the ladder stopped there** (`cap-30k`, 45 minutes at 30 k eps offered,
32 connections). 810,001 pushes offered, **154,610 refused with `429`** (19.1%), 65,539,100 of
81,000,100 events accepted — **80.9% of the offer**. No 5xx from ingest, no OOM, `disk_guard=ok`.
Every refusal is a `429`, which is backpressure working rather than the server failing, and the rate
it settles at is the answer: **24,274 eps sustained at 2 GiB**, measured over 45 minutes with merge
and retention active.

**And the prediction's first branch held: the WAL is not the ceiling.** Group commit amortized
exactly as it was supposed to — `records/batch` **1.59 → 2.39**, fsync mean **7.62 → 6.91 ms**, its
p95 bucket 25 → 10 ms — so at 1.2x the accepted rate the writer task got *less* busy, not more:
101 batches/s × 6.91 ms is a **70% duty cycle against 96%** at 20 k eps. The durability path had
headroom the whole time.

**Two more rungs bracket it, and the knee is sharp** (`cap-24k`, `cap-22k`). At 24 k offered the
engine refuses 2.31% and achieves 23,445 eps; at **22 k it refuses nothing at all** and achieves the
whole 22,000, with `memtable_buffered` peaking at **43.0 MiB — 35% of its limit**. So 9% more offered
is the difference between a memtable that drains comfortably and one pinned against its ceiling;
this is a flush rate being crossed, not a resource gradually filling. The two saturated rungs pin at
*exactly* the same place — 124.0 MiB memtable, 34.8 MiB WAL backlog, both to the decimal at 24 k and
at 30 k — which is a wall that does not move when the offer rises 25%, and is the strongest evidence
that the gate is what it says it is.

The publishable pair, both measured rather than inferred: **22 k eps sustained refusing nothing**
(45 min), **24,274 eps absorbed** under a 30 k offer. `docs/CONFIGURATION.md` and `docs/VISION.md`
carry both, and the sentence "the ceiling above 20 k is unmeasured" is retired from both.

**What refused is flush, and the gauges name it to the megabyte.** `memtable_buffered` peaked at
**124.0 MiB against the 122.9 MiB `max_memtable_bytes`** — the gate whose message is "flush is not
keeping up" — while the WAL backlog peaked at **34.8 MiB against a 1 GiB limit**, two orders of
magnitude of room. The flush lines say the same thing from the other side: `rows=153800 parts=3`,
where the 20 k runs flushed 11–31 k rows into one part. So the capacity of this engine at 2 GiB is
set by how fast a memtable becomes parts, not by how fast a WAL becomes durable — and the earlier
"~18.6 k eps" was below even this, because it was measured through the retention freeze.

### The ceiling, attributed to one line of code (2026-08-13)

The same treatment the push path got, on the loop the ladder named. Three 45-minute runs at 30 k eps
offered — the rung that pins the memtable — each adding one level of split, all agreeing with each
other on the levels they share. Phases are per part where noted; a flush cuts its snapshot into
**3.32** parts on average.

| | mean | share of the pass |
|---|---|---|
| **flush pass** (`build` + open + visibility + advance + checkpoint wait) | 6,184 ms | |
| `build` | **6,042 ms** | **98%** |
| ├ `write_part_files` (×3.32) | 1,385 ms → 4,598 ms | 76% |
| │ ├ **`write_index` — the blooms** (×3.32) | **1,181 ms → 3,922 ms** | **63%** |
| │ ├ `write_parquet` — Arrow, dictionary, zstd, fsync (×3.32) | 159 ms → 528 ms | 9% |
| │ └ `write_meta` (×3.32) | 44 ms → 147 ms | 2% |
| ├ `parse_rows` + the column census (×3.32) | 272 ms → 903 ms | 15% |
| ├ materialize and dedup (residual) | ~470 ms | 8% |
| └ sort + commit (×3.32) | ~15 ms → 49 ms | 1% |
| `flush_advance` | 90 ms | 1.5% |
| `flush_visibility` — the write-locked transition every query waits out | 23 ms | 0.4% |
| `flush_open` — re-reading and checksumming everything just written | 13 ms | 0.2% |

**So the engine's capacity is spent building the index that makes its read claim true.** Roughly
**63% of the flush pass is `write_index`**, and the flush loop is 95% busy at the ceiling, so about
six of every ten seconds this server can spend accepting logs are spent constructing trigram and
exact-field blooms. Those blooms are why `metadata_rare` answers in 0.38 ms against VictoriaLogs'
5.99 and Loki's 561 at 1.5 M rows. Both halves of that trade are now measured numbers rather than
one measured number and one intuition: **this engine buys its read speed with its write capacity.**

Two guesses this killed, both mine. `flush_open` — re-opening every part to validate checksums,
which the code's own comment flags as deliberate I/O — is **13 ms**, not a cost worth the sentence
defending it. And `write_parquet`, the obvious suspect because it holds the compression, is **9%**;
zstd is not where the time goes, so compression level is not a lever on capacity.

What is inside the 63%, from reading rather than measuring: a `BTreeSet<[u8; 3]>` insert for every
trigram of every line (a 100-byte line is ~98 trigrams, a part is ~47,000 rows, so ~4.6 M tree
operations per part over a domain of 2²⁴), an exact-field token encoded per metadata field, per
`| json` field and per logfmt field per canonical variant, the logfmt parse over every line, and then
sizing and filling one filter per group plus one per 1,024-row window.

- [x] **One lever here is not a trade, and it is the same shape as the one already taken.** Every
  other candidate costs read performance — a looser FPP, fewer windows, indexing fewer fields — and
  those are the claim's own foundation. The trigram set is not: `BTreeSet<[u8; 3]>` over a 2²⁴ domain
  produces exactly the same filter a bitmap or a hash set would, for O(log n) per insert against
  O(1), and there are millions of inserts per part. That is the identical shape as the two-pass parse
  this file recorded and the code has since removed — a structure doing work the output does not
  need. **Unmeasured**: reading says it is a large fraction of the 63% and reading has been wrong
  twice on this page already, so it needs the same treatment as everything else — a change, a run at
  30 k, and the ceiling compared. Whether to spend that is the open question; the diagnosis is done
  either way.

  **Spent, and the rung that defined the ceiling stopped refusing** (`bitmap-30k`, 2026-08-14,
  45 minutes at 30 k offered / 32 connections / 2 GiB, the same configuration as `writephase-30k`
  beside it). **29,996.5 eps achieved of 30,000 offered, nothing throttled, no 5xx, no OOM** —
  against 24,512 eps and 18.3% refused before. Every one of the ladder's registered stopping
  conditions is now unmet at this rung, so **the published pair (22 k refusing nothing, 24,274
  absorbed) is superseded and the ceiling is once again unmeasured, somewhere above 30 k.**
  `docs/VISION.md` and `docs/CONFIGURATION.md` carry the retired numbers and cannot be corrected
  until the ladder climbs again; that run is the item below.

  *The gate that refused names itself as the one that stopped:* `memtable_buffered` peaked at
  **78.0 MiB against its 122.9 MiB limit**, where both saturated rungs used to pin at 124.0 to the
  decimal. WAL backlog 27.6 → 6.8 MiB. The flush loop went from **95% busy to 88%** while accepting
  22% more events.

  *What actually moved, per event — the only fair unit, because the shape of an observation
  changed.* Flushes went 424 → 2,493 passes and parts 46,973 → 30,417 rows, since a flush that keeps
  up cuts on its interval instead of draining a memtable pinned at the limit. So per-part means
  flatter the result and per-event numbers are the honest ones:

  | phase | µs/event before | after | |
  |---|---|---|---|
  | `write_index` | 25.15 | **10.87** | **−57%** |
  | `write_part_files` | 29.48 | 19.67 | −33% |
  | `build` | 38.71 | **29.43** | **−24%** |
  | `write_parquet` | 3.39 | 6.32 | +87% |
  | `write_meta` | 0.94 | 2.45 | +160% |
  | `flush_visibility` | 0.15 | 1.10 | +642% |

  **The blooms are no longer the flush's largest term: `write_index` is 37% of `build`, from 65%.**
  The four rows that rose are per-part and per-pass fixed costs amortized over smaller parts and six
  times as many passes — the signature of an engine that is no longer saturated rather than a cost
  the bitmap introduced — but they are real at the new operating point, and `write_parquet` at 512 s
  of the 2,700 is now the second-largest term where it used to be 9%. `flush_visibility`, the
  write-locked transition every query waits out, went from 10 s to **89 s across the run**; it did
  not become a stall (11,237 queries, 0 errors, the stall detector at n=0), and it is the number to
  watch if the flush interval is ever shortened further.

  *Latency is worse and the two runs cannot be compared on it:* response p95 162 → 298 ms, but 18.3%
  of the earlier run's pushes were `429` refusals, which are cheap to serve and were flattering the
  distribution. At full service the harness itself is now inside the measurement — 600 pushes/s over
  32 connections is a 53 ms budget against a service p95 of 114 ms, and the queueing delay p95 is
  191 ms. The next rung needs more connections before it can claim to be measuring the server.
- [x] **The ladder has to climb again, and this is not optional bookkeeping.** Two published
  documents state a capacity this engine no longer has, and the honest correction is a measured
  number rather than a deletion. The next rung is 45 k at more than 32 connections — the client's own
  budget is the constraint at 30 k already — with the same registered stopping rule: achieved below
  99% of offered, any throttle, any 5xx, or an OOM kill.

  **Climbed, and 45 k is the rung that refuses** (`bitmap-45k`, 96 connections, 45 minutes).
  22.9% refused, **34,666 eps achieved** of 45,000 offered, no 5xx, no OOM. `memtable_buffered`
  peaked at **126.3 MiB against the 122.9 limit** and `rows/part` came back to 47,004 — the exact
  signature the old saturated rungs had, so the same gate is refusing for the same reason at a
  higher rate. The publishable pair is now **30 k sustained refusing nothing / 34,666 absorbed**,
  against 22 k / 24,274 two commits earlier: **+36% and +43%.** `docs/VISION.md` and
  `docs/CONFIGURATION.md` carry it with its date and with what invalidates it.

  *The per-event cost held across rungs, which is the check that the bench number was real:*
  `write_index` reads 10.87 µs/event at 30 k and **10.92 at 45 k** — the same work per event at
  1.5x the rate, where the pre-change run read 25.15. `write_parquet` fell back to 4.05 from the
  30 k run's 6.32, confirming that rise as the smaller-parts artifact it was read as rather than
  anything the change introduced: at 45 k the parts are 47,004 rows again and the fixed costs
  amortize as they used to.
- [ ] **The knee is bracketed to 15 k where the last ladder bracketed its own to 2 k.** 30 k refuses
  nothing, 45 k refuses 22.9%, and nothing between them has been run. One rung at ~37 k halves it and
  is the only thing standing between the published pair and the precision the previous pair had. Not
  urgent — both published numbers are measured and neither is an interpolation — but the asymmetry
  should be a decision rather than an omission.

  *The connection count, chosen before the run and by arithmetic rather than by taste:* 45 k eps at
  100 entries a push is 450 pushes/s, and a connection can issue one push per service time, so
  covering the 30 k run's service p95 of 114 ms needs 51 connections before the client is even at
  parity. **96**, for roughly twice that. The 30 k rung is not re-run at 96 because it does not need
  to be: it achieved 99.99% of its offer at 32, which is the client proving it was not the constraint
  there.
- [x] **The ladder found a defect on its way past: an exhausted query memory pool answers `500`.**
  One query in the run failed, and its message is
  `rate({service_name="api-gateway"}[1m]): query memory pool of 322122547 bytes is exhausted`.
  That is a **bounded resource refusing**, the same class as every other limit here, and
  `metric_error_status` (`query/mod.rs`) has no arm for it, so it falls through to
  `INTERNAL_SERVER_ERROR`. A client cannot tell it from a server fault, an operator gets paged for a
  working limit, and it is the same confusion the gRPC refusal audit found on the ingest side — a
  refusal wearing the wrong code. The fix is one arm in that function; what makes it a decision
  rather than an edit is that it is API-visible and it moves the number between two gate buckets
  (the harness excludes `429` from its error rate and counts `500`), so the run that reports it
  should be the run that declares it.

  **Fixed, and the user's reading of it corrected the fix** (2026-08-13). The first framing here —
  "a limit did its job, so `429` and done" — was half right and the missing half is the important
  one: the client should indeed be told `429`, but *this instance failed to serve a query it was
  willing to serve*, and that is not a thing to make disappear. As a `500` it hid among faults; as a
  bare `429` it would have hidden among healthy throttling, which is worse, because the first at
  least made someone look.

  So it is two pieces, not one. Outward, `metric_error_status` gains an arm and the refusal is
  `TOO_MANY_REQUESTS` — matched on `query_memory::EXHAUSTED_PREFIX`, a constant shared by the code
  that writes the message and the code that classifies it, because the scan path reports `String`
  and a literal typed twice is this same `500` returning the day someone rewords it. Inward,
  `QueryMemoryPool::exhausted` counts it at the point of refusal and `/metrics` publishes it.
  That counter is the read side's `ingest_throttled` — the distinction this file's own
  `RuntimeMetrics` doc comments already draw between "this instance is behind, scale or tune it" and
  "this tenant asked for more than it was sold", which the read path had only the second half of.

  Its limits are written where it is defined rather than left for a reader to discover: it cannot
  say whether the cause is concurrency, one greedy query, or a budget too small, so it says to go
  and look. `docs/RUNBOOK.md` carries it with that caveat and with where to look first. Pinned by
  `a_refusal_is_never_reported_as_a_server_fault`, which asserts every arm of the classifier rather
  than the new one alone, and by the pool's own test asserting the counter moves on refusal and not
  on success.

### The one failing gate, decomposed before it is moved (2026-08-12)

The temptation with a single 9.3%-over number is to re-aim the gate at it. What the run's own
percentiles say first:

| | p50 | p95 | p99 | max |
|---|---|---|---|---|
| push **response** (from the intended send) | 12.74 | **273.33** | 538.28 | 5346.3 |
| push **service** (from the actual send) | 11.96 | **47.60** | 128.27 | 4690.6 |
| the **queueing delay** between them | 0.90 | **233.32** | 498.27 | 5306.3 |

So 233 of the 273 ms is a push waiting for its own connection, not for the server. And the harness
arithmetic says that is not a rare-tail effect but the steady state: 20 k eps ÷ 100 entries =
**200 pushes/s over 8 connections = 40 ms of budget per push**, against a service p50 of 12 ms and a
service **p95 of 47.6 ms**. At p95 the server is already outside the per-connection budget, so a
backlog forms in the ordinary case and every 4.7-second service event puts ~100 pushes behind it.

Two questions, and the second is not answered by the first: **is the 233 ms a property of the rig's
8 connections?** and **what is the server's own tail — p99 128 ms, max 4.7 s — made of?**

**Pre-registered, before the runs, so the reading cannot follow the result:** the discriminator is
the connection count at a fixed offered rate. If head-of-line waiting is the mechanism, then at 32
connections the response p95 collapses toward the service p95 (~40–50 ms) while the service
percentiles stay put. If instead the server is saturated at 20 k eps, more concurrency arrives at a
server that cannot take it: service p95 *rises* and the response p95 does not come down much. The
first outcome says the 273.3 ms measures the client's fan-out and the gate has to name it; the
second says the number is real backpressure and re-gating on service would be hiding it.

Both arms run at the current revision rather than against `lockorder-1h`: that hour predates the
in-flight body bound (`3bde4d7`), which put a new counter on the HTTP push path, and comparing across
it would confound the connection count with a code change. The 8-connection arm is therefore also
the check that the middleware cost nothing.

**Neither prediction happened, and the third outcome is the more useful one** (`conn8-1h`,
`conn32-1h`, one hour each, 2026-08-12). The control arm reproduces the day in an hour — response
p95 266.4 against the day's 273.3 — so the middleware costs nothing and an hour is enough to ask the
question. Then, at four times the connections and the identical offered rate:

| | 8 connections | 32 connections | |
|---|---|---|---|
| push response p95 | 266.42 ms | **166.83 ms** | −37% |
| push **service** p95 | 40.29 ms | **106.47 ms** | **×2.64** |
| queueing delay p95 | 226.41 ms | **6.80 ms** | **÷33** |
| push response p50 | 12.47 ms | 12.64 ms | unchanged |
| pushes accepted / throttled | 720,001 / 0 | 720,001 / 0 | identical |

The head-of-line waiting was real and it is gone: 226 ms of queueing becomes 6.8 ms. But **the wait
did not disappear, it moved inside** — service p95 rose by 66 ms of the 220 that left the queue — and
the response p95 landed at 167 ms rather than at the ~40 ms service floor the first prediction named.
The unloaded cost is untouched (p50 12.5 ms both) and so is the throughput (720,001 pushes accepted
in both, nothing throttled). That is the signature of a **serialization point inside the server**: at
200 pushes/s the queue exists either way, and the connection count only decides which side of the
socket it forms on.

**So the gate stays on response, and the reasoning is the opposite of what it started as.** The
temptation was to re-aim it at service on the grounds that service is the server's own number. It is
not one: service p95 is 40 ms at 8 connections and 106 ms at 32, on an unchanged server at an
unchanged rate — it varies 2.6x with the client's fan-out, and it varies in the flattering direction,
reading *better* the thinner the client. A gate on it would have been more rig-dependent than the one
it replaced, not less. Response is what a client experiences and it moves the honest way; what it
needs is its **connection count named in its definition**, which is now recorded in the load config's
target rather than left as an implicit property of the rig.

What this promotes is the second question, which is now the only one left and has a prime suspect:
every push funnels through **one journal writer task** (`journal/writer.rs`, an mpsc to a single loop
that frames a batch, writes it, and `sync_all`s once for the batch), with `max_batch_ms=0` so a batch
is only what already arrived. A p50 of 12 ms against a p95 of 40–106 ms at 200 pushes/s is what an
M/G/1 queue at high utilization looks like, and nothing in the process measures it: the flush log
line carries no duration, and there is no timing on the append path at all. That is the next
measurement — phase attribution on the push path — and not a tuning change.

### The server's own tail is one `sync_all`, and its tail is the merge writing beside it (2026-08-12)

The writer task now times each of the four phases a push passes through and publishes them as
histograms (`signy_journal_{append_queue_wait,write,fsync,insert,checkpoint}_ms`, plus the batch
and record counters that give the batch size); a batch over 250 ms also logs a line naming which
phase it was, because a histogram cannot say which phase any *one* slow event was in and the tail is
what the whole argument is about. The soak scrapes `/metrics` once while the server is still up and
its verdict prints the table. `phase-1h`, an hour at 8 connections and 20 k eps:

| phase | n | mean | p50 ≤ | p95 ≤ |
|---|---|---|---|---|
| append queue wait | 720,001 | 7.00 ms | 5 | 25 |
| write (`write_all`+`flush`) | 453,779 | **0.00 ms** | 1 | 1 |
| **fsync (`sync_all`)** | 453,779 | **7.62 ms** | 10 | 25 |
| memtable insert | 453,779 | 0.16 ms | 1 | 1 |
| checkpoint | 4,450 | 0.30 ms | 1 | 1 |

**The service time is one fsync and nothing else.** Write, insert and checkpoint together are under
0.5 ms of a 12 ms median. 453,779 batches in 3,600 s is 126 fsyncs a second at 7.62 ms each — the
writer task is busy **96% of the time**, which is what the queue in front of it is made of, and
`records/batch = 1.59` says each of those fsyncs is amortized over barely more than one push.

**And the tail's tail is the merge.** Of the hour's slow batches, every one is at least 57% fsync by
duration, and the largest is unambiguous: `records=1 bytes=10772 total_ms=4094.8` of which
`fsync_ms=4094.6` — a single 10 KB record whose durability took four seconds. That is the same
number the day-long run reported as its push service max (4690 ms) and its response max (5346 ms),
so the 24-hour tail is now attributed rather than guessed. What it is waiting behind: **294 slow
batches in the hour against exactly 294 merges, and 235 of the 294 fall within 5 s of a merge
completing.** The 4.09-second one sits in a 4.2-second gap in the flush log with a merge completion
0.98 s after it. The WAL's fsync is queued on the same device the merge is rewriting a part onto —
`io_full` for the run is 135.5 s, 3.76% of it.

So the failing gate's causal chain, end to end and each link measured: `push_response_p95` 273 ms =
the client's 8 connections queueing (233 ms, which moves to 6.8 ms at 32 connections and reappears
inside the server) in front of a writer task at 96% utilization, whose service time is one WAL
`sync_all` averaging 7.6 ms, whose own excursions to seconds are merge I/O on the shared device.

- [x] **What to do about it is a decision, not a finding, and it is the user's.** Every candidate is a
  trade the 2026-08-07 freeze exists to stop being made casually, and none of them is a defect being
  fixed: `max_batch_ms` is 0, so a linger of a few ms would amortize each fsync over more pushes —
  it raises the floor for a lightly loaded server to buy tail at a busy one, and the comment on that
  knob records that a linger once capped a single connection's throughput outright. Giving merge its
  own I/O priority, or pacing its writes, would address the excursions rather than the median. And
  a second writer task is not available without giving up the single-file WAL's ordering. The
  diagnosis is what this item owed; the measurements above are what any of those choices would have
  to be judged against.

  **Decided: nothing changes, and the trade is written into the architecture instead**
  (2026-08-13, the user's call after the alternatives were laid out). What settled it was not the
  9% — it was that the question turned out to be about what an acknowledgement *means*. This engine
  answers after `sync_all`; the bed's other two answer in 1.4–4.7 ms, which cannot contain a device
  sync on hardware where this engine measures one at ~7 ms, so they are acking before durability and
  flushing behind it. That is a different promise, not a better implementation, and it is the
  promise the "a client can only hold data back if the server declines it" premise rests on.
  Splitting the writer is the one that would have removed the cost, and it removes the single
  ordered WAL with it — which is what makes a checkpoint one number, and what makes a hole in the
  middle of the WAL a refusal to start rather than a silent gap. `docs/ARCHITECTURE.md`'s durability
  section now carries the measurement, the comparison, and the reason each alternative was declined,
  so the next reader meets the trade instead of the symptom.
- [ ] **A 3.8x on the bed's own push p95 that nothing explains, recorded rather than chased.** Same
  bed, same corpus, same offered rate, same 8 connections: `3363d61`'s ten-times run read push
  response p95 **24.3 ms** and today's read **91.9 ms**, while Loki (4.92 → 4.70) and VictoriaLogs
  (2.16 → 1.44) did not move. Both still pass the harness's 250 ms target, which is why nothing
  failed and why this would have gone unnoticed. What it is not: the two changes that touched the
  push path in between — the in-flight body bound (`3bde4d7`) and the writer instrumentation
  (`8d6fea8`) — were both measured at zero cost on the soak rig at the same offered rate (266.4 ms
  without, 275.7 ms with, against the day's 273.3). What is left is host state — three 45-minute
  soaks wrote tens of GB to the same filesystem today — or a four-minute phase being sensitive to
  it. **Unknown, and left that way deliberately**: the discriminator is one more bed run, which is
  cheap, but the number is not a gate and chasing it now is not what this phase is for. If it
  reproduces on the next bed run for any reason, it is a real regression and this note is where it
  starts.

## The claim arc, round four: the decode is kept, and the claim holds (2026-08-06)

The user's bar for this round: "VL보다 빠르지 않으면 의미가 없습니다" — `metadata_rare` must beat
VictoriaLogs, not sit within 1.1x of it. Three changes, each forced by a measurement the previous one
produced:

**1. The decoded row-group cache, selection-keyed.** The per-admitted-group constant is the wide
reader's build; a part is immutable, so a decode can serve every later scan. The first wiring cached
only whole-group decodes and measured **zero effect in the bed** — live replay against the running bed
showed the gauge frozen through three identical `label_only` queries, because every matrix query is a
sub-window, streams span the whole window and groups hold whole streams, so the time page selection
keeps part of every group and the whole-group fill condition was unsatisfiable. (The unit tests had
passed vacuously around the same defect — the fill-eligibility check sat *after* the selection fold, so
two tests now anchor on the cache actually holding bytes.) The fix keys entries by
`(row group, normalized selection)`: a repeated window resolves to the same page selection and replays
the decode; a different predicate resolving to the same rows replays it too. Replay feeds the very
batches a miss would produce through the same `scan_batch` — the answer cannot change, which agreement
then confirmed. Local: repeat serves at **115 µs** against the 2.84 ms miss, miss path unchanged
(p=0.62). 256 MiB budget (`SIGNY_ROW_GROUP_CACHE_MAX_BYTES`), one global byte counter, per-reader
LRU, bytes returned on reader drop, gauge `signy_row_group_cache_bytes`, memprof arena
`row_group_cache`.

**2. Window bloom FPP 1% → 0.1%.** With the cache in, the bed read cold 0.67x — but warm 1.10x in the
same run after 0.57x the run before, and the miss tier showed why the margin was thin: a genuinely
cold `metadata_rare` read 3072 rows for a one-row answer, ~150 windows at 1% admitting 2-3 windows the
token is not in, each a ~0.5 ms narrow-pass examination. At 0.1% the expected false admission is ~0.15
window for ~1.5x the filter bytes (14.4 vs 9.6 bits/token, self-describing encoding, no compat break).

**3. The narrow pass is remembered.** The warm wobble (run 3 read every sub-2 ms shape's warm — `rate`
included, which the cache never touches — ~+0.6 ms over its own cold, monotonically rising through the
five repeats; a standalone matrix rerun minutes later showed none of it) was machine state, but it
exposed that warm still paid a narrow-pass builder per group per repeat. The pass is a pure function
of (group, window, base selection, definitive predicates) on an immutable part, so its outcome — the
selection, or the rejection, which is most of what repeats pay — is cached beside the batches.

**The bed with all three** (`compare/run.sh`, agreement **168/168 on all three pairs**, load PASS at
2 g, memory_gate 2 GiB UNDER_BUDGET at anon peak 1048.5 MiB / 19,765 eps — the wider blooms and both
caches cost +32 MiB over the first cache build's 1016.2): `metadata_rare` cold
**0.17x**, warm **0.17x** vs VictoriaLogs (0.23 ms against 1.36/1.30) — the claim holds on both
passes, and not narrowly. Collateral: `line_filter` 1.87x → **0.54x/0.64x**, `trace_window`
**0.17x/0.18x**, `json_field` warm 1.98x → **0.97x**, `json_field_rare` **0.25 ms** flat.

Read honestly, the bed's rare-shape "cold" p50 contains duplicate issues: the matrix builds the rare
shapes' windows from the window index alone, so the 8 apps × 3 windows are **3 distinct queries
issued 8 times each**, and 21 of 24 "cold" issues replay a decode some earlier duplicate filled. All
three systems face the identical sequence — Loki's result cache gets the same gift and still answers
at 79 ms — but the per-tier truth is recorded here: first issues 1.97/1.50/0.99 ms (w0/w1/w2) against
VictoriaLogs' 1.95/1.64/0.74 — parity, 1.01x/0.91x/1.34x — and every repeat ~0.23 ms against its flat
~1.4. The engine wins the bed's definition of cold/warm outright; on a query nobody has asked before
it is at VictoriaLogs parity, and the remaining constant there is the narrow+wide builder pair, still
the round-one item (dictionary/page reuse inside the parquet crate) if it is ever worth parquet
internals.

What this round rejected: filling the cache from whole-group decodes only (measured no-op in the bed,
above), and treating run-3's warm 1.10x as an engine regression (falsified by the standalone rerun;
recorded instead as the thin-margin signal that motivated change 3).

**Round-four follower, same day: the metric path joins the cache.** `rate` was the one shape the cache
could not touch — its named projection failed the exact-layout serve gate — and the round-four bed
priced that at 2.92x/3.11x vs VictoriaLogs, the worst ratio left. `ScanProjection::view_in`
re-addresses a scan's fields into the cache's batch layout, so any scan with a complete view is served
from cached batches while materializing only the columns it names (per-row work identical to its own
narrow decode; fills stay exact-layout, pinned by test). The bed with it: `rate` **1.31 -> 0.46 ms,
2.92x -> 1.06x cold / 1.09x warm** — parity on a sub-millisecond shape — with agreement 168/168 x 3 and
load PASS unchanged, and `metadata_rare`/`trace_window` drifting further down (0.12-0.15x). Remaining
vs VictoriaLogs: `label_only` 1.66x (42-48 ms, decode already cache-served, the residue is sink+
serialize per returned row) and `json_field` cold 3.25x (the per-row `| json` parse; warm is 0.96x via
replay).

**The sweep, 2026-08-06 evening: every log shape faster than VictoriaLogs on both passes.** Four
measured steps after the `rate` view (beds 6-9, agreement 168/168 x 3 and load PASS on every one):

* *The single-pass serializer* — the log result went struct -> `serde_json::Value` tree -> bytes, a
  second full pass over a 4 MB response. A typed payload enum serializes `Vec<StreamData>` straight
  to bytes. `label_only` 42-49 -> 29-30 ms same-data A/B; bed six: 1.66x -> 1.33x/1.19x vs VL.
* *The advisory serve, tried and reverted* — letting a common-value predicate skip the narrow pass
  and ride the base entry (the pipeline as its only filter) bought `json_field` cold 41.3 -> 29.6 but
  sold warm 10.3 -> 29.7: the parse ran on 6250 rows where the narrow selection kept it to ~1000.
  The narrow pass's value on a common predicate is the parses it prevents, every pass. Reverted on
  bed seven's numbers.
* *The subset serve* — the keeper: the narrow pass stays, and its wide decode is sliced out of the
  base entry a broad query cached (a narrow selection is a subset of the base it examined, translated
  through the entry's own selection key, zero-copy). Bed eight: `json_field` **0.98x cold / 0.91x
  warm** (13.3/11.8 ms from 41.3/29.7), everything else unchanged.
* *`StreamKey`* — the response's label union was built per returned row (deep-cloned label map, a
  `BTreeMap` keyed by full label-set comparison — `trace_id` unique per row and last alphabetically
  made every probe walk every label, ~4.4 us per returned row, the whole of `label_only`'s residue).
  The union is now never materialized: equality, hashing and the wire all read one sorted merge with
  metadata shadowing. Streams emit in first-occurrence order (the bed's digest is order-independent;
  its ordering check is per stream). Same-data A/B 28-36 -> 14-15 ms, response bytes identical.

Bed nine, the sweep: `label_only` **0.90x/0.61x**, `line_filter` 0.49x/0.50x, `json_field`
0.88x/0.81x, rare shapes 0.00-0.26x, `trace_window` 0.26x/0.17x — every log shape under 1.0x vs
VictoriaLogs cold and warm; vs Loki 0.00x-0.22x everywhere. The one ratio above water is `rate`
1.07x/1.06x — 0.44 vs 0.42 ms, a 0.02 ms gap at the HTTP jitter floor, recorded rather than chased.
memory_gate 2 GiB after the sweep: UNDER_BUDGET, anon peak 1187.1 MiB at 19,769 eps — up 139 MiB from
the first cache build's gate (wider blooms, narrow entries, and both caches warmer under query load),
at 58% of the declared budget.


## The claim arc, round three: labels leave the schema (`4bcd01c`, 2026-08-06)

The structural change the fold-rejection named, user-approved: the L per-row label columns became one
`_stream` UInt32 ordinal indexing `meta.streams` — now the load-bearing ordinal table, assigned in
**first-occurrence order over the sorted row stream** by a fold both writers share, so the two cannot
disagree (the byte-identity test still pins them; a new cross-tenant shared-set test pins the dedup).
The scan resolves labels by `Arc` clone, evaluates matchers once per stream, and the per-row run test
is a u32 compare; LabelSetCache, the per-row label memcmps, the blind second projection with its
uniform-match proof, and `ColumnSet.labels` are deleted. The merge writer derives `stream_labels` from
what survived instead of taking a superset — retiring the latent hazard where a label whose last rows
retention dropped failed the merged part's own `validate_meta_file` exactness check. Old parts fail at
open naming delete-and-re-ingest before any downcast can panic; a truncated ordinal table is refused
by the stream-index cross-check, with a scan-time bound check as the second fence. 467+40 tests,
clippy 0.

A measurement lesson paid for on the way: the local matrix probe read **2x worse across every shape**
after the change, which same-machine A/B (HEAD vs parent, same minute) refuted — ordinal build
10.77 ms vs parent 12.43 ms on the pure-scan bench, a **13% improvement**; the "regression" was the
machine's clock state between measurement days. Absolute local numbers across days are not
comparable; the bed's ratios are, because all three systems share the machine's state.

The bed with it in: agreement **168/168 on all three pairs** (the proof no answer changed), load PASS
at 19,771 eps with q p95 96.4 ms, and the claim shape moved: `metadata_rare` **2.3 → 2.02/2.07 ms**,
ratio vs VictoriaLogs **1.93x → 1.46x/1.47x** cold/warm (Loki side widened to ~39x faster).
`exact_field_hit` bench 3.01 → 2.77 ms. The claim (<1.1x) still does not hold: VictoriaLogs answers
at ~1.4 ms and ours needs ≤ ~1.55 — about half a millisecond of per-group constant remains, and the
named follower is round four: dictionary/decompressed-page reuse between the narrow and wide passes
(parquet-crate internals), for which this round's narrower projection was the prerequisite.

## The day after the PASS: the roadmap items, each measured (2026-08-03, `eeae4a2`..)

Six workstreams off the back of the stall fix, every one gated on the 2 GiB rig (release,
`systemd-run`, real disk, 20 k eps + 5 qps, 120 s legs) unless said otherwise.

**Bed gate (W1, `eeae4a2`).** `compare/run.sh` used to swallow the load verdict with `|| echo`; it
now reads the surviving limit's verdict out of the result file after the document and artifacts are
written, and fails the bed unless it is PASS. `COMPARE_REQUIRE_PASS` names the targets held to it
(default `signy`, `off` records a known-failing run). Gating last on purpose: a FAIL still
leaves everything needed to diagnose it.

**Allocator (W2, `b4a28de` then re-judged in `4a630d8`).** mallopt at startup, before the runtime
spawns a thread. The measured `MALLOC_ARENA_MAX=1` + 128 KiB trim was tried first and **rejected by
its own A/B**: anon peak 1726/1746 → 478/479 MiB (3.6x) with eps unchanged, but the allocation-heavy
flush path halved its cadence against the single arena — 241 → 113/117 flushes per leg, steady WAL
backlog 8 → 50 MiB — and that, not the WAL compaction it was blamed on, is what pushed the first W4
leg over the backlog-drain heuristic. **Arena cap 4** is the adopted compromise
(`SIGNY_MALLOC_ARENA_MAX`, 0 = glibc's own scaling): 241 flushes (full cadence), anon peak
1726 → **1017.6 MiB**, backlog peak 8.0 MiB, q p95 246.8 ms, PASS. Trim-only (arenas uncapped)
measured 1348.5 MiB — worse than the cap, kept as the record of why 4.

**The slow-query hole (W3, `81e5322`).** The load harness grew a `heavy` shape — every stream, an
hour's window, limit 20 000, weight **0 by default** — to measure what one slow query does to
everyone else through the fair operation lock. Answer, on this corpus: nothing. Heavy served at
280–400 ms (the scan budgets — 5 M rows / 2 GiB — cap a scan's duration long before
`max_query_runtime`), and every other shape's p99 stayed ≤ 85 ms with the run PASS. The hole is
real in the design (a slow query queues the flush writer and the fair queue behind it) but **this
engine's own scan budgets seal it at this scale**, so the superversion/immutable-snapshot rework
stays un-opened, with these numbers as the reason. Re-open it if a workload ever holds a scan for
tens of seconds — the harness knob to reproduce is `SIGNY_LOAD_QUERY_WEIGHT_HEAVY`.

**Local WAL compaction (W4, `0f24a97`).** The dead prefix — bytes before the checkpoint, which
replay seeks straight past — was 89% of the bed's disk total and never truncated in local mode.
`compact_wal` was never remote-specific; the change is the policy: remote always compacts, local
cuts when the prefix outgrows both the live suffix (O(1) amortized rewrite per logged byte) and a
64 MiB floor (`SIGNY_WAL_COMPACT_MIN_BYTES`, `off` = old behaviour). Measured on the rig:
`data_dir` 1137 → **~240–245 MB** steady, push p95 unchanged, backlog drains normal once W2's
arena=4 restored the flush cadence. New e2e crash arm: flush → compact → ingest → crash; parts
serve the flushed rows, the retained suffix serves the rest.

**Query byte pool (W5, `4a630d8`).** The 8 × 512 MiB = 4 GiB product `peak_materialized_bytes()`
documented as unenforced is now a pool: every log scan reserves from
`SIGNY_QUERY_MEMORY_BUDGET_BYTES` (512 MiB) in 8 MiB chunks as rows survive the pipeline,
the reservation rides inside `QueryExecution` until the results drop, exhaustion is a refusal
naming the pool, and the metric path builds its entries **inside** the blocking task and the query
arena (the loop used to run on an async worker, untagged — the "eighth term"). Per-query
`max_query_memory_bytes` unchanged. `peak_materialized_bytes` now reports pool + merge = 1.5 GiB
where the same defaults used to imply five. Final attribution run: query arena peak 63.7 MiB
under load, pool untouched at the margins — the budget is headroom, not a squeeze.

**metadata_rare decomposition (W6, measurement only).** Subtraction probes on the seeded matrix
corpus (verify tenant, 60 reps/variant): an absent token — bloom prunes every group — answers in
**0.28 ms**; the real rare token answers in **5.49 ms** with 3 row groups admitted and 24 576 rows
decoded. The whole VictoriaLogs gap is therefore the **predicate-column decode of admitted row
groups** (~1.7 ms per group, ~0.21 µs/row), not part opens, sidecars or planning. The lever for the
<1.1x claim is row-level postings (or page-level pruning) for exact fields inside an admitted
group; shrinking `row_group_size` is the anti-lever — cutting groups finer was already measured
worse for the broad shapes. Implementation is the next arc.

Final gates on `4a630d8`+: cargo test 456+39, clippy 0; memprof attribution — flush arena peak
**34.7 MiB** (was 513.7 at the baseline, 96.1 after chunking), merge 368.2, query 63.7, memtable
peak 6.6 MiB, backlog peak 8.8 MiB; `memory_gate --budget 2GiB` **UNDER_BUDGET at 42.7%** (874.2
MiB, ingest and settle phases equal).

The bed run with all of it in, W1's gate live and printing `load verdict gate: signy PASS at
2g`: agreement **168/168 on all three pairs, every shape**; signy PASS at 19,771 eps with
query response p95 101.0 ms (Loki 452.6, VictoriaLogs 19.0). The WAL compaction rewrote the disk
story the bed's own caveat used to apologize for: **total on disk 619.0 → 96.5 MiB** (WAL 548.7 →
26.2), which puts signy's *total* below Loki's 117.7 for the first time — 218 vs 266 MiB/GB
ingested — while the settled-data ratio stays 0.60x/1.28x. Anon-during-ingest fell again, 832.0 →
**674.6 MiB** against Loki's 1138.8. `metadata_rare` reads 1.77x/1.84x slower than VictoriaLogs —
the claim still does not hold, and W6 above says exactly which decode to shrink next.

## The claim arc, round one: the blooms window, and the constant is named (`f849e7c`)

W6's lever landed: **BTF5** stores one exact-field sub-bloom per 1024-row window (the same row
count the writer already cuts data pages at), admission ANDs per-predicate window masks — strictly
stronger than per-group admission, same no-false-negative obligation, since a matching row carries
every predicate's token in its own window — and the mask becomes a `RowSelection` that bounds the
narrow pass and, when pass one is skipped, the wide one. Bits are linear in token count, so eight
filters cost one filter plus headers. 461+39 tests green (five new window tests: never-drops-a-row
across boundaries with the time∩window path, boundary straddle, one-window narrow pass
`scanned_rows ≤ 1024`, cross-predicate mask intersection, multi-window byte-identity across both
writers), clippy 0.

What it measurably did: decode volume for the rare shapes fell **24,576 → 4,096 lines/query**
(machine-independent, from the matrix artifacts' own counters; `trace_window` 9,344 → 2,176), and
the decode-bound bench geometry (`part/scan_filters/exact_field_hit`) fell **7.38 → 3.01 ms**.
The fully-pruned probe rose 733 ns → 3.1 µs (≤64 window probes instead of one) — absolute noise
against the 280 µs fixed floor.

What it did **not** do: move the bed-shape latency. The subtraction probes on the seeded corpus,
quiet machine, before/after: absent token 0.28 → 0.29 ms, one-occurrence (one-group) query 2.62 →
**2.28 ms**, three-group query 5.49 → 5.48 ms. Cutting decoded rows 6x left the time flat, which
falsifies W6's "the gap is the decode volume" at this corpus scale and names the real term:
**~1.7–2.0 ms of per-admitted-group constant cost** — two `ParquetRecordBatchReader` builds (the
narrow pass and the wide one) and a whole-page zstd decompression for every projected column,
paid even when the selection keeps four rows, because a page decompresses wholly to serve any row
and the wide projection spans the full schema. The claim needs per-group ≤ ~1.2 ms; the next
levers, in order: fold the two passes into one read (one builder, one set of pages), then a
decompressed-page reuse between the passes if folding alone is short. Windowing stays: its decode
bound is what keeps those levers meaningful at real part sizes, and it is what scales when groups
hold more than this corpus's rows.

Gates on `f849e7c`: isolation PASS (19,870 eps, q response p95 249.3 ms), `memory_gate` UNDER_BUDGET
at 44.4% (910.2 MiB), data_dir bounded at 244 MB. The bed run: verdict gate PASS at 19,779 eps with
query p95 105.1 ms, agreement **168/168 on all three pairs, every shape** — the strongest possible
statement that windowed pruning changed no answer — and the claim verdict, honestly: `metadata_rare`
moved 2.6 → **2.3 ms** while VictoriaLogs' own number moved 1.4 → 1.2 ms, so the ratio reads
**1.93x/1.92x, does not hold**. Loki's side widened to 34.2x faster. The per-group constant above
is now the whole remaining gap: at one admitted group per bed query, 2.3 ms ≈ 0.3 fixed +
~2.0 constant, and ≤1.3 ms (1.1 × 1.2 ms) needs that constant at ≤ ~1.0 ms — the one-read fold is
the next arc, with the decompressed-page reuse behind it.

**The one-read fold was built, measured, and rejected (2026-08-06, not committed).** Skipping the
narrow pass when the window selection was already ≤2 windows — the definitive check moved into the
wide batch scan, answers proven identical by the test suite — made the one-group bed shape *worse*:
2.28 → 3.27 ms. The subtraction that explains it: the bare all-streams probe prices the wide decode
at ~1.7 ms per 1024 rows (arrow string materialization across the ~25-column projection), so the
fold traded a ~0.15 ms two-column narrow scan for a ~1.7 ms twenty-five-column wide scan of the
same rows. Which sharpens the constant's name again: the per-admitted-group ~1.7–2.0 ms is
dominated by the **wide reader's per-build cost across the full projection** — dictionary pages
and column-chunk setup for every projected column, paid once per group regardless of how few rows
the selection keeps — plus the second build the narrow pass adds. The levers that would actually
move it are a leaner wide projection (labels reconstructed from stream identity instead of label
columns — a real arc, the reader currently rebuilds label sets from columns) or reusing dictionary
state across the two builds inside one group (parquet-crate internals). Both are open; neither is
a quick fix, and the two-pass scan stays as the measured optimum of the shapes tried.
- [x] **Then remove Loki push ingest** — the protobuf and JSON variants, the snappy path, the Loki label-text
      parser, and `proto.rs`'s encode side

      Done 2026-08-02, after the OTLP bed run above proved the kept shape at full agreement — the order the
      section header demanded. Gone: the `/loki/api/v1/push` route and both body variants, the snappy path,
      `proto.rs` whole (decode side included — replay of a kind-0 or unframed record now fails with
      "delete the data directory and re-ingest", which is the no-versioning policy said out loud),
      `push_test.rs`, `max_decompressed_push_bytes`, and the `snap`/`prost 0.13`/`prost-types`
      dependencies. Kept and moved: `validate_label_name`/`validate_field_name` to `label_name.rs` — they
      were never about the wire; LogQL's grammar and the reserved Parquet column names are storage and
      query properties, and the OTLP path needs no call site because its labels come only from the fixed
      promotion list (a test on `otlp_log.rs` holds that every promoted name sanitizes valid and
      unreserved). `max_push_bytes` survives as the tenant quota's burst floor, which was its load-bearing
      role all along. The uniquely-covering push tests were ported, not dropped: both backpressure gates
      with their release halves, the allowlist 403, the promoted-label count bound, the u64→i64 timestamp
      overflow, the disabled window, and the full flush-loop-through-restart pipeline, all speaking OTLP.
      `bench_encode_push_request` retired with the thing it measured; `wal/append` now appends the bytes
      the WAL actually stores.

      **Gated on its own full bed run (2026-08-02): 168/168 on all three pairs, every shape, with the push
      surface gone.** Ingest 19,639 eps of 20,000 offered (Loki 19,872, VictoriaLogs 19,931); disk 0.56x
      Loki / 1.18x VictoriaLogs WAL-excluded; `memory_gate --budget 2GiB` UNDER_BUDGET on the same
      revision. One number moved between back-to-back runs without a cause in either engine:
      VictoriaLogs' `metadata_rare`/`trace_window` p50 swung 0.5–1.6 ms, taking the lt/VL ratio from
      1.49x to 4.78x — at millisecond scale that column is variance, and `docs/VISION.md` now quotes the
      range instead of whichever run flattered.
- [x] **Then stop re-encoding into a Loki `PushRequest` for the WAL.** It exists so replay has one decoder
      while two protocols converge, and it costs a whole second message materialized with a clone per line and
      per label, then serialized, framed and batched — five copies for the WAL alone, on the consumer's own
      path. With one protocol it has nothing to do

      Done 2026-08-02 as WAL record kind 2: the payload is the `ExportLogsServiceRequest` as it arrived —
      the HTTP protobuf transport passes its body through verbatim, gRPC and JSON re-encode the decoded
      message — and replay normalizes by kind, the pattern traces (kind 1) proved. The WAL's bytes did not
      move: the old path stored the decompressed `PushRequest` at ~415 B/entry and the export measures
      ~418 B/entry on the same corpus. What moved is the copies — the second message with its clone per
      line and per label is gone from the consumer's path. Ingest at the bed's offered rate is unchanged
      (19.6k eps before and after); the gain was allocation, not the wire.
- [ ] Keep the Loki **query** API and the `| json` parser. Grafana reads through the first; a guest's
      `println!` becomes an OTLP body string that nothing in the chain parses, so the second still has work

## VictoriaLogs: ingest works, and the query half is a design question

`Target::VictoriaLogs` is in the harness and **seeds successfully**: 30,000 rows, 304 pushes, every response
`204`, zero errors, verified present with `* | count()`. Three things the documentation did not answer, now
measured against `victoria-logs` v1.52.0:

- **It accepts Loki push in protobuf+snappy**, not only JSON. The harness sends the same bytes it sends
  signy and Loki, through `signy::proto`, so ingest is genuinely the same wire format on all three.
- **`X-Scope-OrgID` must be a `uint32`.** VictoriaLogs reads it as its numeric `AccountID` and refuses a name:
  `cannot parse "verify-tenant-000" as uint32`. Its tenancy is `AccountID:ProjectID`, so `Target::tenant_header`
  maps the comparison's single tenant to `0`. A multi-tenant comparison would need a name-to-number mapping.
- **Readiness is `/health`.** There is no `/ready`.

### The finding that matters, and it is not a defect

**VictoriaLogs parses JSON at ingest and does not keep the line.** A seeded JSON row comes back with `status`,
`level`, `trace_id`, `duration_ms` as top-level fields and `_msg` reading `missing _msg field`. signy and
Loki both store the raw line and parse it at query time.

That is schema-on-write against schema-on-read, and it is exactly the axis this comparison exists to test —
`docs/ARCHITECTURE.md` names VictoriaLogs' `lib/logstorage` as this engine's design reference, so where the two
diverge is the question. signy indexes JSON fields at ingest **into a bloom** (`indexed_parser_fields`)
but stores the line; VictoriaLogs turns them into **columns**. A bloom prunes row groups; a column is read
directly. That is very likely why `json_field_rare` reads 30,000 rows to return four — and it means
VictoriaLogs is the system that already solved the problem the selectivity axis just found.

- [ ] **Decide what row equality means across schema-on-write and schema-on-read.** The digest compares a
      timestamp, a line and every label with its placement. VictoriaLogs has no line to compare, so the check
      as written cannot run against it — and dropping it for one system would leave the comparison unable to
      say the answers agree, which is the thing it is for. A reduced basis (timestamp plus a canonical field
      set) is the obvious candidate and needs to be argued rather than assumed
- [ ] **Translate the five shapes into LogsQL** and parse `/select/logsql/query`'s newline-delimited JSON. This
      is the expensive half and it was expected to be: the push API is shared, the query language is not
- [ ] Decide whether to run VictoriaLogs with `disable_message_parsing`. It would make the responses
      comparable, and it would also handicap it in a way no real deployment would — so probably not, and the
      difference gets stated instead of removed
- [ ] VictoriaTraces is pulled and unexamined. The trace path has never been compared against anything

## The selectivity axis, and the tension it found

`json_field` filters on `status` or `level`, which match about a fifth of the rows, so the comparison has only
ever measured how fast each engine *scans*. `json_field_rare` filters on a `trace_id` drawn from a population
of `rows / 4` — four rows in thirty thousand — which is what a per-row-group bloom over a columnized field
exists to answer. It carries no `app` selector, because a trace is drawn independently of the app and "find
this trace across everything" is both the real query and the one where the field predicate is the only
selective thing.

Short run, 30,000 rows over 8 streams, limit 100, both systems local, build `f18b8ea`:

| shape | signy | Loki | ratio | lt lines | Loki lines | rows |
|---|---|---|---|---|---|---|
| `label_only` | 0.71 ms | 8.37 ms | **0.08x** | 32,400 | 35,633 | 2,400 |
| `line_filter` | 0.95 ms | 6.16 ms | **0.15x** | 90,000 | 35,839 | 2,008 |
| `json_field` | 4.44 ms | 9.53 ms | **0.47x** | 56,556 | 35,781 | 2,393 |
| **`json_field_rare`** | **39.18 ms** | **23.14 ms** | **1.69x** | **671,184** | 334,032 | **72** |
| `rate` | 8.95 ms | 4.75 ms | 1.89x | 253,728 | 60,034 | 41 |

**It loses on the one shape the design exists to win**, reading 671,184 lines to return 72 rows — every query
reading exactly 30,000, which is the whole dataset, so nothing was pruned at all.

**It is not a bug, and the bloom is not broken.** An absent `trace_id` prunes to **0 lines**; the present one
reads 24,576, which is three full row groups. The filter answers correctly both times. The rows are simply
*there*: a trace's four rows belong to four different streams, rows are now ordered by stream, so those four
rows land in three or four different row groups and not one of them can be skipped.

**So the layout has a tension in it, and both halves are measured.** Ordering by stream is what took
`label_only` to 0.08x and `json_field` to 0.47x, because a label predicate selects streams. The same ordering
scatters a high-cardinality field across every group, because such a field is orthogonal to the stream. Row
groups can be organized for one or the other, not both.

- [ ] **Decide what answers a rare field, given that row-group granularity cannot.** The options are not
      equivalent and one is already excluded: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) says "no inverted
      index" as a decided choice, so a secondary index on high-cardinality fields would reverse a decision
      rather than add a feature. What remains is sub-row-group skipping — Parquet's page index and
      `RowSelection` let a bloom hit narrow to pages instead of whole groups — or accepting that a rare-field
      lookup costs a scan and saying so
- [ ] Confirm the other four rows in the full bed. This run is 30,000 rows with both systems local, not the
      containerized comparison, so the ratios are indicative and the *shape* of the finding is what matters
- [ ] `rate` is still 1.89x and still reads what it reads at any limit, unchanged by the layout

## Next — the backward scan does not know the layout changed

Rows are now ordered by stream before time (`part/mod.rs`, `Row::sort_key`), which is what localizes a stream
to one or two row groups instead of spreading it through all of them. `docs/COMPARISON.md` measured the
reason: `{app="api-gateway"}` returned 6,250 rows and decoded 57,344 against Loki's 6,254.

Measured on `benches/query.rs`, limit 100 over 202,000 rows, against the pre-layout build:

| shape | direction | lines read | time |
|---|---|---|---|
| `label_only` | backward | 187 → 187 | 243 → 250 µs |
| `label_only` | forward | 1,003 → **288** | 2.04 → **1.65 ms** |
| `line_filter` | backward | 9,517 → 6,867 | 17.19 → **10.98 ms** |
| `line_filter` | forward | 15,183 → **2,187** | 10.32 → **2.15 ms** |
| **`json_field`** | **backward** | 3,130 → **5,701** | 13.07 → **14.60 ms** |
| `json_field` | forward | 4,783 → **820** | 9.26 → **6.81 ms** |

**Four improved, one flat, and one regressed — the claim's own shape, in Grafana's default direction.** That
is not a caveat to a win; it is the thing to fix next.

- [x] **Backward scans read a row group from its end, and the end is no longer the newest row.** The
      diagnosis above was half of it — "the window walk is still correct" was the other half, and it was
      **wrong**: the walk plus `StopGroup` returned rows from the middle of the window, which the three-way
      bed caught as a strict disagreement with Loki. See "Open correctness defects" at the top of this file
      for the full account. Of the two fixes this item proposed, "read it whole" won, unconditionally for
      now: choosing per group needs the format to say which groups are time-ordered, and that flag rides the
      next format change. Backward is *better* than the pre-layout numbers (`json_field` 12.43 ms against
      the 13.07 this item wanted back); forward paid +108–139% for losing an early exit that was the same
      unsound assumption, and that cost is recorded in the defect entry rather than smoothed over
- [ ] **Cutting a row group on every stream change is not the answer, and was measured.** With 128 streams
      over 8 parts it turned ~3 row groups per part into 128, all far under `row_group_size`, and per-group
      cost swamped the pruning: `label_only` forward went 2.04 → 5.19 ms while reading *half* the rows. The
      selectivity comes from the sort order, not from the cut. Recorded so it is not retried
- [ ] **Record per-group time-monotonicity in `meta.json`** (`row_group_ts_monotonic`, with the next format
      change), and gate the windowed backward walk and forward's early group exit on it. A single-stream
      group is exactly the case both were correct on, and it is the common case at low cardinality
- [x] Re-run `compare/run.sh` once the backward path is fixed — the three-way run at the top of "The
      languages can ask the same question" is that run, and the fix is what took `label_only` and
      `json_field` from 22/24 to 24/24 strict

## M10 — declared memory budget ([`docs/VISION.md`](docs/VISION.md) I)

M9 supplied the number this milestone was missing: **at a 2 GiB container limit, ingesting 1.2 M events at an
offered 20 k eps with a 5 qps read workload, signy is OOM-killed and Loki is not**
([`docs/COMPARISON.md`](docs/COMPARISON.md)). The accounted memtable was 111 MB at the time. That is invariant
I failing against a competitor on the same machine, and it is the test this section's last item asks for,
already written and already red.

### Phase A — diagnosis (done)

[`docs/MEMORY_ATTRIBUTION.md`](docs/MEMORY_ATTRIBUTION.md) is the measurement, `src/memprof.rs` is the
instrument (arena-tagging global allocator behind the default-off `memprof` feature), and
`scripts/run_memprof_local.sh` is the one-command reproduction. What it changes about the plan below:

- [x] **The dominant term is not any arena.** At the moment of the kill the engine's live heap was 669 MiB of
      2 GiB and **44% of the anonymous footprint was memory the process had already freed** and glibc had not
      returned — 61–69% in some variants, 67% in the surviving 8 GiB run. A budget denominated in live bytes
      would have read a third full at the instant the kernel killed the process
- [x] **It is a function of allocation rate, not of anything held.** 52 GB allocated in 33 s across 444 M
      allocations, 217x the offered data rate; query is 57–77% of that traffic and flush 17–38%.
      `MALLOC_ARENA_MAX=1` with trim thresholds takes `anon / live` from 2.5–4.1 to **1.34** and more than
      doubles time-to-OOM without fixing anything
- [x] **The query workload is not required and neither is merge.** Ingest alone is killed at 2 GiB on this bed
      (which contradicts `docs/COMPARISON.md`'s ingest-only observation; both are recorded, the difference is
      not explained). Merge disabled is killed sooner than ingest-only
- [x] **Refuted: sidecars and `PartMeta`** — 0.2–1.2% of the anonymous peak, ~240 kB + ~140 kB per part, and
      `signy_part_sidecar_resident_bytes` is accurate to 4%. **Refuted: in-flight push bodies** — the
      journal append awaits its own completion, so in-flight is bounded by HTTP concurrency and measures
      ~0.3 MiB. **Refuted as a memory term: `rows_from_snapshot` outside `spawn_blocking`** — same bytes either
      side of the hand-off; it remains a latency defect
- [x] **Found, and not on anyone's list: a fourth per-row `Labels` clone.** `query/metrics.rs:134-155` walks
      every scanned row on an async worker thread, before the `spawn_blocking` at `:158`, cloning
      `stream.labels` per row — outside every arena, scanning to `max_metric_rows + 1` = 1 000 001 rows rather
      than the API limit, and 203 MiB at its high-water. Folded into M11's read-path list

### Phase B — the budget

**The gate comes first and it is built.** [`docs/MEMORY_BUDGET_GATE.md`](docs/MEMORY_BUDGET_GATE.md)
is the baseline; `src/bin/memory_gate.rs` is the gate. Every item below is a fix, and this is the
only thing that can say whether a fix worked, so it landed before them and it landed red.

- [x] **A test that runs at a declared budget and asserts peak memory stays under it** — and it
      asserts against the cgroup's `anon`, not against the sum of the arenas, because the sum of the
      arenas was a third of `anon` at the moment of the kill. One command,
      `cargo run --release --bin memory_gate -- --budget 2GiB`, in a `systemd-run --user --scope`
      cgroup with swap off, driven by the M8 harness with reads concurrent with writes at the
      comparison bed's parameters and seed. Four outcomes by exit code — 0 under budget, 2 over
      budget, 3 OOM-killed, **4 could not be measured, which is a failure and not a skip**
      (`docs/LOAD_RESULTS.md` §3: a gate that cannot measure must not pass; a budget met by
      refusing 90% of the offered load counts as unmeasured). **Measured: OOM-killed at t≈49 s at
      2 GiB; survives at 5 GiB at 86–91% of it; 4586 MiB of `anon` — 2.24× — when given 8 GiB of
      room and asked to stay inside 2 GiB — all on build `50190cf`. On `9199e07`, with M11's shared
      label sets, the same commands read: **`UNDER_BUDGET` at 2 GiB at 90–96% across three runs, and
      0.93x rather than 2.24x when given 8 GiB of room.** Not in CI: it needs a cgroup scope and minutes per run,
      and a peak-memory number off a shared runner is the kind this repository has already retired.
      CI compiles it, so it cannot rot the way a script and a document did
- [x] **The gate should read the budget from the server once the knob exists**, rather than being
      told the same number twice. Resolved by construction rather than by plumbing: the gate creates
      the cgroup scope, and the server inside now detects that same scope's `memory.max` and declares
      60% of it — one number, read where it lives. `--server-env SIGNY_MEMORY_BUDGET=off`
      measures the pre-budget behaviour for an A/B
- [ ] **A killed run loses the harness's own numbers.** The harness is killed three seconds after
      the server dies and writes its report only at the end, so the ingest and query columns of
      every OOM row in `docs/MEMORY_BUDGET_GATE.md` are empty. A periodic partial write, or a
      result written on signal, would make a failing run as informative as a passing one
- [x] **Make the anonymous footprint track live bytes first.** Measured precondition, not a tuning note: with
      the default glibc configuration no live-byte budget can be honest. `mallopt` at startup, or an allocator
      whose heap decays, or the arena-tagging allocator promoted into production. Whichever is chosen, the
      `anon / live` ratio it achieves must be published beside the budget

      Done by fixing the mmap threshold (`malloc_tuning.rs`, `M_MMAP_THRESHOLD` 128 KiB beside the trim
      threshold, `SIGNY_MALLOC_MMAP_THRESHOLD` overrides, 0 restores the dynamic ratchet). The ratio it
      achieves: **anon/live 5.30 → 1.60** on the soak rig, time-to-OOM at 2 GiB 150 → 502 s. The cost,
      measured before defaulting it on — two 240 s runs per arm at 8 GiB, sustained 20 k eps: eps
      19,956–19,985 against 19,972–19,981, push service p99 57.6/82.9 ms against 81.7/87.8, all four
      verdicts PASS — nothing. The glibc parameter is the same shape as the trim threshold's story: a
      threshold that only ratchets upward, fixed once at startup. The earlier rejection of this knob was
      measured on the pre-streaming-merge build, whose kill was live spikes rather than retention
- [ ] **Honest metering.** `entries_bytes` (`memtable.rs:69-81`) counts line and label lengths only — not the
      56-byte `LogEntry`, the 48-byte slot per metadata pair, malloc headers, or `Vec` slack. Measured
      **1.70–1.79x under** in situ on the comparison corpus, so `MAX_MEMTABLE_BYTES=256 MiB` is really ~440 MiB
- [x] `SIGNY_MEMORY_BUDGET` divided into ingest 20% / flush 25% / merge 25% / query 25% / sidecar 5% —
      the measured shares, not the guessed ones. Existing knobs become overrides; what is not overridable is
      that they sum. **Flush and merge did not fit their shares** (721 MiB and 771 MiB measured against
      512 MiB each at a 2 GiB budget), which is the work, not a reason to raise the shares. Flush's share is
      no longer the same problem — M11's shared label sets took `rows_from_snapshot` from 1 345 to 823 bytes
      per row and its peak live from 26–28 MB to 13.85 MB on the bench — but the arena was never
      re-measured in situ, so **721 MiB is a figure for a build that no longer exists and the number for
      this one is not known.** Re-run the attribution before sizing anything from it

      Landed (`dcdd418`, 2026-08-08), in the order this item demanded: the attribution was re-run first
      (`docs/MEMORY_ATTRIBUTION.md`, build `b9165b0`) and the shares that shipped are that measurement's,
      not this item's guesses — merge 25%, query pool (and the per-query cap) 25%, row-group cache 12.5%,
      memtable 10% accounted (~17% resident at the measured ×1.73), floors under each, nominal sum 72.5%
      with the rest for flush-rides-ingest, the still-unbounded sidecars and the metering gap. Unset, the
      budget is 60% of the detected cgroup limit — VictoriaLogs' contract, adopted after it was measured
      holding this exact workload in half a gigabyte ([`docs/CONFIGURATION.md`](docs/CONFIGURATION.md),
      "Memory budget"). Every knob still overrides its derived default. Verified in the order the plan
      named: the 2 GiB / 20 k eps / retention-on soak that killed the engine at 255 s now **runs its full
      600 s at anon peak 1480 MiB**; `memory_gate --budget 2GiB` reads **UNDER_BUDGET**; and 8 GiB delivers
      19,961 eps at push service p99 57 ms against 19,956–19,985 and 57.6–82.9 before — no cost
- [ ] **Flush cannot be sized independently of ingest.** `rows_from_snapshot` held a copy of the memtable at
      **3.3x its accounted size** and 1 326–1 345 bytes per row, and the two peaked together. The label sets
      are now shared with the memtable rather than copied out of it, so the copy is the lines and the
      metadata; the multiple is no longer 3.3 and has not been measured in situ. Either the flush share is
      expressed as a multiple of the ingest share, or the flush streams the snapshot in bounded chunks
- [x] **Query admission by budget, not by slot.** Replace `MAX_CONCURRENT_QUERY_SCANS × MAX_QUERY_MEMORY_BYTES`
      (8 × 512 MiB = 4 GiB, admitted in a comment at `config.rs:522`) with a shared arena. Same ceiling, and a
      burst of cheap queries no longer queues behind a slot count. The arena must include the metric path's
      materialization, which is outside it today

      Done as the shared pool (`4a630d8`, 2026-08-03): `SIGNY_QUERY_MEMORY_BUDGET_BYTES` (512 MiB),
      reserved in 8 MiB chunks as rows survive the pipeline, held with the results, refusal on exhaustion;
      the metric path's entries loop moved inside the blocking task and the query arena with it. The slot
      semaphores still exist as *concurrency* bounds — what changed is that bytes are no longer implied by
      slots. `peak_materialized_bytes` reports pool + merge.
- [x] **`merge_max_memory_bytes` must come from the budget.** Its 1 GiB default is half a 2 GiB container and
      is derived from nothing the operator set; one group reached 771 MiB live

      Derived now (25% of the budget, `dcdd418`) — and the reason this is not the change the
      `HostMemory` doc recorded as measured-worse is that the streaming merge changed what the knob
      bounds: pages and writer state rather than group materialization, so a smaller budget pages
      smaller instead of merging oftener. At 2 GiB the derived cap is 322 MiB and the engine survived
      the workload that killed it, anon peak 1480 MiB; the merge arena under the cap was not itself
      re-measured (the surviving run is a production build), which a memprof soak can still do
- [x] **Sidecars inside the budget.** They are outside it on purpose today (`part/reader.rs:77-81`), so resident
      memory grows with part count unbounded. Make them LRU-evictable — they are already durable in `index.bin`.
      Sized from the measured ~240 kB per part, not from a share

      Done twice over, and the second time is the one that mattered. LRU eviction under
      `SIDECAR_CACHE_MAX_BYTES` bounded the total — and then the eviction itself became the problem,
      because what the cache held was a near-exact copy of the file (x1.01) at four megabytes a part.
      A 122 MiB budget held about thirty parts against a working set of sixty to two hundred, and the
      re-decode loop that produced filled the allocator faster than it could return pages. The cache
      holds offsets into a mapped `index.bin` now (`58a58eb`): 1.3 kB per part, 0.6 MiB resident
      across 233 parts, zero misses. The budget still exists and no longer binds — which is what
      "inside the budget" was asking for, reached by making the thing small rather than by evicting
      it harder
- [x] **Stop materializing `PartMeta::streams`** (`part/mod.rs:231`, `part/metadata.rs:172-176`) — every distinct
      label set in every open part, held as live `String`s. Measured ~140 kB per part

      Done by interning rather than by not materializing: `PartMeta::streams` is `Vec<SharedLabels>`
      now, each set resolved through a `Weak`-entried table at part open (`intern_stream_labels`),
      so a label set costs one allocation across every part that holds it and dies with the last
      one — and the reader's separate `stream_table` copy is deleted outright, because the interned
      `meta.streams` *is* the ordinal table. This is not the intern table the `SharedLabels` doc
      rejects: that was per-row on the ingest path; this is per-stream at part open, on the cold
      path, with no eviction policy to invent. Cross-part sharing is pinned by `Arc::ptr_eq` in
      `identical_streams_across_parts_share_one_interned_label_set` — equality would pass on
      duplicates, and duplicates are the bug. The 24-hour soak's GROWING `part_meta` gauge is a
      different quantity — `meta.json` bytes on disk, which grow as merge matures parts and cap by
      construction at (streams per part × labels) + (row groups per part × the min/max/rows
      arrays), both bounded by the corpus and `merge_max_part_rows` — so the gauge plateaus near
      ~25 MB at the soak's part count and the next long run carries the arena number for the
      resident side
- [x] **Bound in-flight push bodies.** The ingest gate is checked once at request entry and nothing limits
      concurrency, so (in-flight requests x 64 MiB) sits outside the accounting. Measured at 0.3 MiB on the bed,
      so this is closing a hole rather than recovering memory.

      Done: `SIGNY_MAX_INFLIGHT_PUSH_BYTES`, 128 MiB static / 5% of a declared budget, `off` allowed,
      counted at admission and released by an `InflightBody` guard's `Drop`, published as
      `signy_inflight_push_bytes`, refused with the 429 + `Retry-After` the other two thresholds
      already use. Three things worth keeping in mind about the shape it took:

      * **The check is a middleware, not a handler.** A handler takes `body: Bytes`, so by the time
        handler code could look at anything axum has already put the whole body in the heap — an
        admission check there would be counting memory it had already spent. The layer sits outside
        `DefaultBodyLimit` so it runs before the body is collected, and reads only `Content-Length`.
        A chunked body has no length until it has been read, which is the case a bound exists for, so
        it is charged the ceiling one request may reach.
      * **An idle server always admits one body, whatever the ceiling says.** Otherwise a ceiling set
        below one legal push refuses it forever with nothing in flight to wait for — the trap
        `max_push_bytes` flooring the token bucket's burst already avoids. That makes the knob safe at
        any value, and `the_tightest_inflight_ceiling_still_serves_a_lone_push` pins it through the
        router at a ceiling of one byte.
      * **HTTP only, and that is a decision rather than an omission.** gRPC has no `Content-Length` —
        the framing is streamed — and tonic hands the service an already-decoded message, so the wire
        size is gone before any code here could read it. Charging a flat ceiling per gRPC push instead
        would refuse four concurrent 100 KB batches on a 2 GiB container: a throughput regression
        wearing a memory bound's clothes. That transport stays bounded by tonic's
        `max_decoding_message_size` × its concurrency, recorded in `CONFIGURATION.md` beside the knob.
        Closing it properly needs a decode-layer change and has no measurement asking for one.
- [x] ~~**A test that runs at a declared budget and asserts peak RSS stays under it.**~~ Moved to the top of
      this phase and built there, because it is the gate the rest of these items are measured by rather than
      one of them: [`docs/MEMORY_BUDGET_GATE.md`](docs/MEMORY_BUDGET_GATE.md)

## M11 — bounded copies and deep pruning ([`docs/VISION.md`](docs/VISION.md) II, III)

Write path:

- [x] **`Arc<Labels>` end to end** — memtable, `Row`, part write, reader, registry, executor, query
      result and the metric path, as `SharedLabels = Arc<Labels>`. **The "largest single payoff" claim
      held.** `cargo bench --bench rows`: `rows_from_snapshot` goes from **1 457/1 505/1 569 bytes per
      row** at 2/5/10 labels to **823.4 at all three**, from **11/17/27 allocations per row** to
      **6.00**, and peak live from 26.0/27.0/28.3 MB to **13.85 MB flat** — the label term is not
      smaller, it is gone, and the table is now flat in the label sweep. `--bench part`: the scan goes
      from 3796/3955/4078 bytes per row to **3167/3263/3337** and from 11.2/19.3/27.4 allocations per
      row to **6.19/6.32/6.45**, with peak live 54.0/56.5/58.4 MB to **31.0 MB flat**. Timings improved
      everywhere and nothing regressed: `rows/from_snapshot` 1.80–4.31x, `rows/from_entry` 1.82x at two
      labels and **4.40x at ten**, `part/scan_tenants/1` 1.36x, `part/scan_label_columns/10` 1.27x,
      `memtable/query_cardinality/8192` 1.83x. **The gate moved from 5 GiB to 2 GiB**
      ([`docs/MEMORY_BUDGET_GATE.md`](docs/MEMORY_BUDGET_GATE.md)): `--budget 2GiB` was `OOM_KILLED` at
      t≈49 s and is now `UNDER_BUDGET` at 90–96% across three runs, and the 2.24x overshoot measured by
      `--budget 2GiB --limit 8GiB` is **0.93x** — the workload's own anonymous high-water fell from
      ~4.5 GiB to ~1.9 GiB while achieved eps rose from 18.7 k to 19.7 k. `encode_stream_index` is now
      keyed by borrows of the rows and `write_meta` collects distinct borrows, so both clone per stream
      rather than per row; the reader interns one label set per distinct stream per scan, keyed on a
      hash of the row's label columns with every candidate verified against them
  - [x] **A fifth site, on the metric path and not on this list:** `sample_value`
        (`query/metrics.rs`) built a `BTreeMap` of every label and every metadata pair per row to read
        the one field an `unwrap` names. It resolves that field directly now
  - [ ] **A sixth, still there:** `process_entry_with_labels_cancellable` (`logql/ast.rs`) clones
        `labels` into a mutable `fields` map **per row**, for every query including one with no pipeline
        stages at all. Removing it needs a copy-on-write field view across the whole pipeline, not a
        type change, and it sits directly on the just-fixed extracted-field placement.
        **It did not fall out of the streaming rewrite and was not taken there.** The sink calls it on
        every row the storage scan produces, so what the bound removed was the number of calls and not
        their cost — `benches/query.rs`'s `json_field` case now makes 3 130 of them instead of 200 250,
        which is the same 4.4x-per-row it always was. The one thing that changed is the shape of the
        fix: the pipeline now runs inside the sink with the row's own labels in hand, so a
        copy-on-write field view has exactly one caller to satisfy
  - [ ] **Two meters now over-count, both conservatively.** `Row::materialized_bytes`
        (`part/mod.rs`) and `estimated_log_entry_memory_bytes` (`query/mod.rs`) charge every row for the
        label bytes it shares, so merge group sizing and `max_query_memory_bytes` are stricter than the
        memory that is really held. Neither was loosened here: that changes a limit and belongs with
        M10's honest metering
- [x] Remove the two free memcpys: the line clone at `ingest.rs:247` (the source is separately owned and could
      be consumed), and the whole-payload copy in `frame_tenant_record` (`journal/mod.rs:90-101`) whose 7-byte
      prefix belongs in `writer_loop`'s batch buffer

      Done 2026-08-06 (`9ae6893`), in its modern form — the cited line clone died with the push surface, and
      its OTLP equivalent was the per-record clones in `normalize_request`, which now **consumes** the decoded
      message (body string, severity text, attribute keys and scalar values all move; callers settle the WAL
      bytes first). The frame copy is gone the way the item said: the command carries kind+payload unframed
      and `writer_loop` lays the frame straight into the batch buffer, CRC computed over the pieces.
      Measured on the memprof rig: **WAL-arena allocation traffic 2,046 → 1,023 MB per run (halved, exactly
      the frame), allocations 44.8k → 15.7k**, eps unchanged at 19.9k. Gated: memory_gate UNDER_BUDGET at
      44.0% (900.3 MiB), bed PASS at 19,769 eps with agreement 168/168 × 3 held, push response p95 5.21 ms —
      the lowest any bed run has recorded, though push p95 has been noisy across runs and the halved
      allocator traffic is the claim, not the tail.
- [x] One sort — the chunked flush (2026-08-03, below) emits streams in `(tenant, labels)` order with
      per-stream entry ordering, so the global sort is gone; the per-partition `sort_rows` stays as the
      dedup and as a near-O(n) safety net over already-sorted input
- [x] One parse — `encode_blooms` runs the JSON and logfmt parsers over every line twice, to size the filter
      and then to fill it (`part/format.rs:335-341`, `:360-366`). **Done, and found stale while measuring the
      flush ceiling** (audited 2026-08-13): `encode_group_blooms` collects tokens once into their windows and
      sizes each filter from what it collected, and its comment says so — "this used to be two passes that each
      ran the JSON and logfmt parsers over every line ... the tokens are collected once instead". The `| json`
      half now rides the parse the `_pf:` columns already paid for.
- [x] Move `rows_from_snapshot` and the global sort inside `spawn_blocking` — done by the chunked flush
      (2026-08-03, below): materialization happens per chunk inside the blocking task, and
      `rows_from_snapshot` is no longer on the production path at all
- [x] Cap the exact-field bloom. `exact_capacity` is the raw token count for the row group
      (`part/format.rs:329-347`), so a wide-JSON tenant can make `index.bin` larger than `data.parquet`

      Done 2026-08-06: a window holding more than 65,536 tokens is stored **saturated** — a sentinel
      length that decodes as admit-everything, distinct from the zero length that means prune — so the
      filter caps near 79 KB per window and an attack degrades pruning for its own window instead of
      growing the sidecar. A 130-field-per-row flood test pins `index.bin < data.parquet`, that saturation
      admits both present and absent values, and that the query still answers exactly.
- [x] Consider compressing the WAL payload. It stores the decompressed protobuf, discarding the client's
      snappy, which makes the WAL the dominant term in write amplification

      Done 2026-08-06 (`927e222`): kind 1/2 payloads are zstd-1 frames, compressed on the ingest task
      (parallel with connections, not behind the writer), decompressed at replay under the record-size cap.
      Measured: the bed's WAL-on-disk 26.2 → **9.2 MiB** and total disk 95.9 → **79.5 MiB** (Loki 117.8);
      eps 19.8k held, agreement 168/168 × 3 held, gate UNDER_BUDGET. A pre-compression WAL refuses replay
      with the delete-and-re-ingest message, pinned by a test. Fallout fixed in the same arc: the harness's
      backlog-drain heuristic was scale-free and read the now-2 MB-peak sawtooth as "never fell below half
      its peak" — a 16 MiB trivially-healthy floor makes a backlog a falling-behind flush would dwarf pass
      on its size, with tests for both sides.

Read path:

- [x] **`normal_scan_limit = usize::MAX` is gone**, and the read path is a bounded top-K sink the
      reader, the registry and the memtable stream into (`src/part/sink.rs`, `src/log_scan.rs`). The
      scan reads until it holds `limit` rows that **survived** the pipeline, and skips a whole part
      from its tenant segment's span, a whole row group from `row_group_min_ts`/`max_ts`, and the
      rest of a part on the first row past the sink's frontier — ties go to whichever row arrived
      first, so the answer is what a stable sort plus `truncate(limit)` gave. **`benches/query.rs`
      is the new bench and the evidence**, limit 100 over 202 000 rows in the window:

      | shape | dir | lines | bytes allocated | peak live | time |
      |---|---|---|---|---|---|
      | `label_only` | backward | 69 178 → **187** | 178 416 315 → **367 655** | 10.5 → **0.08** MB | 49.000 ms → **243.43 µs** |
      | `label_only` | forward | 6 750 → **1 003** | 165 841 034 → **8 031 426** | 10.2 → **2.89** MB | 31.377 ms → **2.0404 ms** |
      | `line_filter` | backward | 142 906 → **9 517** | 355 132 227 → **76 060 366** | 10.7 → **6.42** MB | 92.237 ms → **17.189 ms** |
      | `line_filter` | forward | 112 478 → **15 183** | 355 744 979 → **39 117 415** | 11.0 → **5.03** MB | 89.450 ms → **10.316 ms** |
      | `json_field` | backward | 200 250 → **3 130** | 703 107 130 → **43 949 524** | 26.8 → **5.81** MB | 308.78 ms → **13.067 ms** |
      | `json_field` | forward | 200 250 → **4 783** | 703 106 682 → **21 343 859** | 26.9 → **5.05** MB | 308.00 ms → **9.2568 ms** |

      `--bench part`: `scan_tenants/1` **−66.5 %**, `/16` −57.5 %, `/128` −12.5 %;
      `scan_filters/no_filter` **−75.3 %**, `line_filter_hit` −75.8 %, `exact_field_hit` −75.4 %. The
      unbounded full-part scan is flat — `scan_label_columns` −1.5 % / +0.4 % / +0.7 % and
      3163.5/3258.2/3330.4 bytes and 6.17/6.29/6.41 allocations per row against
      3166.7/3263.4/3337.4 and 6.19/6.32/6.45, peak live 30.96 MB against 30.97 — because a sink
      that has not filled holds a plain `Vec` and ranks nothing. `--bench memtable`:
      `query_cardinality/256` **−92.2 %**, `/8192` −82.2 %, `query_line_filter/contains` −93.3 %,
      `query_stream_depth/100` −61.0 %, `/2000` −41.0 %, `/50000` −2.6 % (the last is the
      whole-stream sort two items below, which this does not touch).
      **The gate margin widened and the surviving budget did not fall**
      ([`docs/MEMORY_BUDGET_GATE.md`](docs/MEMORY_BUDGET_GATE.md)): 2 GiB is `UNDER_BUDGET` at
      **78–83 %** across three runs against 90–96 %, the `--limit 8GiB` high-water is
      **1659 MiB / 0.81x** against 1913 / 0.93x, and 19 871–19 889 eps against 19 720–19 729 — but
      1792 MiB is still `OOM_KILLED`, at 94.4 %.
      **Regressed:** `part/write` +2.6 % with a byte-identical allocation table and +0.4 % on the
      same bench without this change, i.e. ~2 % of code layout in a crate that grew a module;
      `scan_filters/line_filter_pruned` +1.9 % at 583 ns, where every row group is pruned and
      constructing the sink is the only new work
  - [x] **The triple materialize-and-sort is gone with it.** `PartReader` and `PartRegistry` now push
        rows into the caller's sink instead of returning `Vec<StreamResult>`, so the only rows held
        anywhere are the `limit` in the executor's sink and the only sort is over those. The old
        `query_*` entry points remain as wrappers over a `TopKRows` sink, so tests, merge and the
        object-store restore probe kept their signatures. Merge reads `Row`s straight out of a
        `RowCollector` rather than through `StreamResult`s it immediately flattened
  - [ ] **The bed's own `data.stats` number moved less than the bench, and the reason is not the
        limit.** Reproducing the 30 000-row, 8-stream, 3-window seed dataset the 16 384-against-Loki's-1 251
        figure came from: at the matrix's own `limit` of **20 000** the figure is **unchanged** at
        16 384/16 384/13 616, because 20 000 over ~1 250 matching rows is a limit that never binds
        and therefore never bounds anything. At Grafana's default `limit=100` it is
        **16 384 → 11 072, 16 384 → 9 272, 13 616 → 5 528** for `json_field` and
        8 192 → 7 184, 8 192 → 5 376, 5 424 → 800 for `label_only`. Still 4.4–8.8x Loki's 1 251, and
        the residue is not the limit: Loki's index takes it to one stream's chunks, while a row group
        here interleaves all eight streams and every row of them is decoded and filtered. That is
        projection and predicate pushdown, the next two items, not this one
- [ ] **Projection pushdown.** `ProjectionMask` appears nowhere; `count_over_time({app="x"}[5m])` decodes every
      label column and the `structured_metadata` JSON blob. **This is now the largest remaining term on the
      shape the claim rests on**, measured: see the `data.stats` item above
- [x] **The Parquet footer is parsed once per part scan** instead of once per selected row group, which
      fell out of the streaming rewrite: the scan needs one handle it can clone per row group and per
      backward window, so hoisting `open_part_data` out of the loop was the way to get one.
      `PreadReader` is a `File` behind an `Arc` and `ArrowReaderMetadata` is an `Arc<ParquetMetaData>`,
      so the clones are refcounts. Caching it across *scans* on the reader is still open and is a
      different lifetime question
  - [x] **A backward scan no longer decodes a whole row group to reverse it.** Parquet reads forwards
        only, so a backward scan used to `collect()` every batch of the group and reverse the list —
        8192 rows of Arrow string arrays built to answer a `limit=100`, in the direction Grafana
        defaults to. It now reads the group in windows from its end with `with_offset`, doubling the
        window each time so a scan that does read the whole group pays O(group rows) in skipped
        records rather than one skip over the whole prefix per window
- [ ] Do not allocate the line before the filter that rejects it (`reader.rs:727` precedes `:728`)
- [ ] **Extract required literals from `|~` regexes** so trigram blooms apply. `bloom_prune` matches only
      `LineFilter::Contains` (`reader.rs:778-787`)
- [ ] Parallelize part scans within a query (`part_registry.rs:579` is sequential), and stop holding a scan
      permit across an object-store restore (`execution.rs:367` vs `:374`)
- [ ] Memtable query: binary-search the sorted stream instead of counting every entry against the scan budget,
      and stop sorting the whole stream on every query (`memtable.rs`, `scan_memtable_stream`). **The
      scan half is done** — the sink's frontier ends a stream at the first entry past it, which is
      `query_stream_depth/100` −61 % and `/2000` −41 % — and the sort is the whole of what is left,
      which is why `/50000` only moved 2.6 %
- [ ] Verify the trigram bloom's `to_lowercase()` on both sides (`bloom.rs:139-148`, `:57`) cannot produce a
      false negative for non-ASCII substring filters — a dropped result is a correctness bug, not a pruning miss
- [ ] Do not write an `.access` marker file per candidate part per query on the scan thread
      (`part_registry.rs`). Narrower than it was: a part the sink's frontier rejects is skipped before
      the marker is written, so a limited query now writes one per part it actually opens

## Test-suite repairs (fold into M8)

- [ ] **`src/tests/e2e.rs` is not e2e.** `ingest_once` (`:69`) hand-decodes the push and calls `journal.append`
      directly instead of going through `ingest::push`, and `:102-110` hand-executes what `flush.rs` does, in
      the test's own order. A bug in `flush_once`'s ordering is invisible to it
- [ ] Replace wall-clock assertions with virtual time. `tokio/test-util` was added for exactly this
      (`Cargo.toml:33-37`) and `start_paused` appears three times, all in `startup.rs`. The flush loop, merge
      scheduler, retention loop and eviction cadence are still untested under it
- [ ] Use the injectable `Clock` (`src/clock.rs`) in fixtures instead of `SystemTime::now()`, so window-boundary
      behaviour stops depending on when the suite runs
- [ ] Make the measurement tests pin their numbers. `part/tests.rs:907` computes the fragmentation cost,
      prints it, and asserts only that many > one — a 100x regression passes, and `LOAD_RESULTS.md` cites it as
      the fixed location for five specific figures
- [ ] Assert something in the parser list tests (`logql/tests.rs:240,285,537,727` check only `is_ok()`) and the
      readiness tests (`query/tests.rs:397,406,419`)
- [ ] Add a concurrent read/write test. There is none
- [ ] `#[cfg(test)]` the `pub fn for_test` constructors shipping in production code (`backpressure.rs:117`,
      `tenant_quota.rs:119`, `tenant_policy.rs:806,811`, `object_storage/fault_store.rs:90`), and test the
      wiring `startup.rs` actually uses rather than a parallel assembly

## P0 — production gates

- [x] **Fix WAL compaction wedge** (same item as the P5 BLOCKER below). Remove the intent record durably
      immediately after success, and treat remaining phase-2 records as complete and remove them — already
      wedged instances recover through an upgrade, so no manual deletion procedure is needed.
- [x] **Ingest backpressure**: return `429` before journal append when MemTable/WAL backlog limits are exceeded
      (+`Retry-After`, OTLP uses `RESOURCE_EXHAUSTED`). Track MemTable/WAL backlog size in O(1).
      Knobs: `SIGNY_MAX_MEMTABLE_BYTES`, `SIGNY_MAX_WAL_BACKLOG_BYTES`,
      `SIGNY_BACKPRESSURE_RETRY_AFTER`.
- [x] **Guarantee tenant deletion**: split groups in half when merge exceeds the memory limit, and rewrite a
      single part in row-group windows. Large parts cannot retain zero-retention rows forever.
- [x] **Block startup without the policy token**: fail startup when a tenant policy is stored but the token is
      missing, so hidden deleted data does not reappear.
- [x] **Writer fencing**: claim the manifest's `writer_epoch` at startup and verify it on every CAS.
      On fencing, return ingest 503, `/ready` 503, stop force-flush retries, and exit with code 1.
      **M6's "fully drain the old instance before starting the new one" procedure is now enforced.**
- [x] **Tenant allowlist**: `SIGNY_ALLOWED_TENANTS`. Tenants outside the list receive 403.
      Startup fails if the default tenant is not in the list.
- [x] ~~**On-disk format version**: Check `version` in part/trace-part `meta.json` before checksum validation.~~
      **Removed.** signy does not version its formats: it is not deployed anywhere, so there is no old
      data to read and every compatibility path was code that could never run. Gone with it: the manifest's
      `format_version`, the WAL's per-record version bytes, the pre-tenancy `LGY2` trace record that only
      replay could produce, the compaction state's version byte and its phase-2 acceptance, and the `BTF1`,
      `BTF2` and `BTF3` bloom readers. `PartReader` is simpler for it — three format flags that were constant
      became nothing, and `exact_field_bloom` stopped being an `Option`
  - [ ] **Reintroducing format versioning is an open decision, not pending work.** The other line of
        development raised `PART_META_VERSION` to 3 and wrote a migration design for crossing a format
        on a running deployment. Neither was taken: this trunk's recorded choice is that nothing on
        disk or on the wire is versioned (`docs/ARCHITECTURE.md`, decided choices). The design exists
        and can be recovered from `backup/pre-rebase-origin` if the choice is ever reversed — reversing
        it is a decision to make deliberately, not a thing to let arrive with a merge
- [x] **Metadata endpoint guards**: Add semaphore, timeout, `start`/`end`, and `match[]` count limits to
      `labels`/`label_values`/`series`/`index_stats`.
- [x] **Remove O(parts) from `/metrics`**: Workers publish merge-debt and unknown-tenant gauges.
- [x] **Multi-tenancy** — **the instance's share is complete** (2026-08-18). Every sub-item below is done:
      durable monthly usage accounting is the control plane's by the checklist's own reasoning, and Parquet
      range reads are struck in P2 on measurement rather than deferred. The design, cost
      model, and implementation checklist are in
      [`docs/MULTI_TENANCY_DESIGN.md`](docs/MULTI_TENANCY_DESIGN.md). The recorded byte ranges outlived
      the read path they were for: `a_tenant_decodes_from_its_own_byte_range_and_asks_for_nothing_else`
      proves a tenant's rows decode from the footer plus its own range and that the decoder reaches
      outside it for nothing — so if the range-read decision is ever revisited, the mechanism is already
      proven and what is left is the cache lifecycle, sized in the design doc.
      **The previous design using tenants as a storage-path axis (`docs/ARCHITECTURE.md`, "Multi-tenancy")
      was discarded because of R2 Class A costs** — writing objects per tenant does not fit the $1 plan
      budget at any RPO.
  - [x] Extract and validate `X-Scope-OrgID` (Loki push + OTLP gRPC), and configure the missing-header policy
  - [x] Record the tenant in WAL records (owner survives restart; existing WAL recovers under the default tenant)
  - [x] Shared tenant parts: `(tenant, ts)` sort + row groups aligned to tenant boundaries + tenant index in
        `meta.json` (for both logs and traces)
  - [x] Isolation surface: require tenant arguments for MemTable, PartRegistry, TraceRegistry, queries, and catalog reads
  - [x] Per-tenant retention deletion path — determine expiration from the tenant index, whole-delete + merge rewrite,
        and clamp every read path. Design and rationale: [`docs/RETENTION_DESIGN.md`](docs/RETENTION_DESIGN.md)
  - [x] Replace policy polling with **push**. `PUT` one tenant at a time, store it in object storage, then ack;
        load all at startup (failure is fatal). Tenant deletion = retention `0` (ignores rewrite threshold).
        Removed polling and the direct `reqwest` dependency — object storage is the only outbound call.
        Details: [`docs/RETENTION_DESIGN.md`](docs/RETENTION_DESIGN.md)
  - [x] ~~`(tier, day)` partitioning~~ — rejected. Fixing retention at write time does not apply plan changes
        to existing data. Partitions remain by `day`.
  - [x] ~~Add Parquet range reads (P2) and use `(part, tenant)` local cache keys~~ — **decided against**;
        the measurement and the reasons are on the P2 entry. Per-tenant cache keys go with it: they need
        the range read to have anything to key, and the sharing they would split is what pays for the
        download. **Sidecar consolidation is
        done**: the trigram blooms and the stream index are one `index.bin`, so a part is three files rather
        than four — one fewer billed PUT per flush, one fewer round trip per catalog restore, and one fewer
        checksum pass per part at startup, which §8 measured as the actual startup cost, and it stands
        on its own
  - [x] **Per-tenant ingest rate** — `ingest_rate` rides the same push as retention. The control plane sets the
        number; this side only owns the field and enforcement points. Check before decompression so an over-limit
        tenant cannot consume CPU.
  - [x] Per-tenant query-scan quota and concurrency — `query_rate` rides the same pushed
        policy as `ingest_rate`, charged after a scan with what it actually read
  - [x] Per-tenant stream cardinality limit — `max_streams` on the pushed policy, enforced
        against the union of what the tenant holds in parts and in the buffers
  - [x] Per-tenant usage — `GET .../tenants/{tenant}/usage` on the admin API. **Not** labels on
        `/metrics`: that scrape is unauthenticated and process-wide by design, and a label per
        tenant is the cardinality problem this engine bounds everywhere else
  - [x] ~~Durable monthly usage accounting~~ — **the control plane's, not the instance's.** A month spans
        instances and outlives them; this side only exports per-tenant usage for the control plane to
        account for. Recorded under "Decided against — not pending work".
- [x] Document TLS unsupported as an architecture decision
- [x] Ingest input limits (body/decompressed length/line/label count and length/timestamp acceptance window)

## P0 — closed after the 2026-08-21 review

- [x] **Per-tenant storage limit** (`max_stored_bytes`). A plan that sells a period and a size had only the
      period: retention decided when bytes left, nothing bounded how many piled up first. Pushed explicitly
      per tenant beside the rates, with no global fallback for tenants nothing has been pushed for. Enforced
      by refusing writes, never by deleting: the space returns when retention retires parts.
- [x] **Storage accounting no longer depends on the cache.** The usage endpoint prorated `fs::metadata` of
      the local Parquet body by row share, so an evicted part contributed nothing and the billed number fell
      as parts went cold. It reads the per-tenant extents in `meta.json` now. Trace parts gained the same
      extent (`TRACE_META_VERSION` 3), so a tenant sending traces is counted for them; version 2 parts
      report zero until they age out.
- [x] **Free disk space is measured, and bounds ingest.** `statvfs` on the data directory, sampled by a task,
      published as `signy_data_dir_free_bytes`/`_total_bytes`, and below
      `SIGNY_MIN_FREE_DISK_BYTES` (2 GiB) ingest returns 429. The last guard rather than the first:
      eviction bounds the cache and the backlog limit bounds the WAL, and past this one flush cannot write.
- [x] **Locks no longer poison.** Every shared structure was behind a `std::sync` lock opened with
      `.unwrap()`. A panic under a read guard — a Parquet decode is one — poisoned it, and every later
      reader panicked, leaving an instance that is up, passing liveness, and answering nothing. parking_lot
      throughout; the write guards were already treated as fatal and the read guards cannot leave a
      half-written structure.
- [x] **Structured logging.** `SIGNY_LOG_FORMAT=json`, set by the container image, text by default.
- [x] **CI.** Format, clippy at `-D warnings`, the suite, and a GHCR image build on master.
- [x] **Deployment guide** — [`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md). One `docker run` on a host that
      has nothing but Docker: `--stop-timeout=-1` because the default is a SIGKILL ten seconds into the
      force-flush, log rotation because the default json-file driver has none, `127.0.0.1:` because a
      published port outranks the host firewall. Plus the gateway's obligation to overwrite `X-Scope-OrgID`
      rather than append, R2 bucket versioning, free-tier defaults, and the alerts that cannot be sent from
      the machine they describe.

## P1 — LogQL improvements

**Historical record.** LogQL was removed with the read-path decision (issue
#3): the text parser, the metric evaluator and the format stages are gone, and
the first-party flat filters (`docs/QUERY_API.md`) are the query surface. The
engine capabilities this list built — parser stages, field filters, the
counting fast path, exact-field pruning — survive under the new grammar.

- [x] Support `line_format`, `label_format` — a deliberate subset of Go templates
      (literal text and `{{.field}}`), refusing what it cannot render rather than approximating
- [x] Support `unwrap` (bare field and `duration(field)`) plus `sum_over_time`,
      `avg_over_time`, `min_over_time`, `max_over_time` and `quantile_over_time`
- [x] Support binary operators with a scalar operand (`+ - * / %`, `== != > >= < <=`).
      Vector-to-vector is refused: both sides would need their own scan, which is a
      planner change rather than a parser one
- [x] Support `without`
- [x] Support `offset`
- [x] Support subqueries
- [x] Loki-compatible JSON semantics for arrays, top-level arrays and `null`
- [x] Exact-field pruning on stream-label fields, through the stream index

## P2 — correctness and storage performance

- [x] Deduplicate duplicate logs that can result from crash replay — every part is written through one
      sort, which now drops entries identical in tenant, stream, timestamp, line and metadata. A flush
      cannot see a twin that is already in an older part, so the removal lands the first time the two
      are merged; `signy_wal_replayed_entries` still reports the upper bound a restart introduced
- [x] ~~Add Parquet range reads (**multi-tenancy prerequisite** — shared parts must read only a tenant's byte
      range, so this is no longer an optional optimization)~~ — **decided against, on measurement**
      (2026-08-18). Both halves of the framing failed: the tenant byte range is the *only* over-fetch axis
      that exists, and it is the axis that pays for itself. A restored body is read by 5.66 distinct
      tenants before eviction, so serving them selectively is 5.66 fetches where the whole download is
      one — **requests ×6.7 to move ×0.37 the bytes**, spent on the axis R2 bills to save the axis it
      gives away. The two runs below are the record; the second is the one that decides it.

      **Measured before being built, and the measurement does not support the framing** (2026-08-14,
      Tier B rig — `scripts/run_load_local.sh`, which already runs the engine on `file://` with an
      8 MiB part cache to force eviction→restore — at 16 tenants for 240 s, with the byte meter added
      in `bd72732`). Four facts, in the order they killed each other's readings.

      1. **Part data is 99.9% of what this engine reads from object storage** — 917.3 MB against the
         manifest's 0.85 MB. The first arithmetic on the aggregate said the opposite, which is why
         the counter is split by object class: a part restore and a manifest rewrite are both bytes,
         and the total cannot tell them apart.
      2. **A restore already fetches only the parts a query admitted.** Eviction drops `data.parquet`
         and keeps `index.bin` and `meta.json` (`cache.rs`, `evict_bodies`), so bloom and index
         pruning happen locally and cost no bytes at all. There is no "download it to find out it
         does not match" to remove.
      3. **What it over-fetches is inside the admitted part, and it is not only the tenant axis.**
         Every part held **15.2 of 16 tenants**, so ~93% of a restored part is rows the querying
         tenant cannot see — but the download also ignores the row-group and page selection the scan
         then applies for time and labels. So the honest description is not "shared parts must read
         only a tenant's byte range": it is that **the download applies none of the selection the
         read path already computes**, which costs a single-tenant deployment the same way.
         — **Retracted 2026-08-18 by the block below.** The row-group half of this is wrong: the
         selection equals the tenant segment exactly, so there is no time or label narrowing to
         apply, and a single-tenant deployment has nothing to save at all.
      4. **And the whole of it is a function of cache pressure, which is a deployment property
         nothing states.** The same run with a cache that holds the working set: **64 restores and
         749 KiB per query becomes 2 restores and 17.4 KiB — 43x.** The production default is
         `SIGNY_CACHE_MAX_BYTES` = **10 GiB of local disk**, so the documented deployment is the
         second regime and not the first.

      So the item's value is `restore rate × over-fetch per restore`. The second factor is measured
      now; the first is not a property of this code and cannot be derived from it — it is how often
      queries reach outside the local disk cache, which depends on retention against cache size and
      on how far back users query, and **no document here states either.** Building against an
      unstated rate is the "arithmetic on an estimate" that `counting_store.rs`'s own header was
      written to warn about.

      *What this does not say:* that the work is worthless. At 30-day retention a 10 GiB cache holds a
      fraction of the data, and any query outside it pays the full over-fetch. It says the item is
      **unsizeable as written**, and that what it needs next is a stated deployment assumption rather
      than code — plus a re-scope, because "tenant byte range" names one axis of a download that
      currently uses none.

      *One caveat on the rig, stated rather than buried:* the large-cache run's verdict is `FAIL`, on
      `remote_healthy_fraction` 0.908 against a 0.95 gate. That is Tier B's injected 3% object-store
      error rate landing on a much smaller number of operations, not a behaviour change — the byte
      counters it produced are unaffected, and the small-cache run at the same injection passes.

      **The two numbers that decide the sign, measured** (2026-08-18, same rig and shape — 16 tenants,
      240 s, 8 MiB cache — with the meter in `src/restore_meter.rs`; verdict `PASS`, which also settles
      the caveat above: the earlier `FAIL` was the injected 3% landing on a different sample, and a
      re-run at the same seed passes at 0.958).

      The item trades a resource this backend does not bill for one it does. R2 bills per request and
      egress is free, so the 917 MB the byte meter found costs nothing there; what a selective download
      changes is the request count. Two numbers fix it, and both are properties of this code.

      1. **What a selective download would cost in requests.** Over the first scan of each restored body
         — the query the download is issued for — the selection is **1.00 contiguous run** of row groups,
         **6.5%** of the part. A row group's column chunks are contiguous and the log path projects every
         column, so that run is one byte range: **one range plus one footer, against the single GET a
         whole restore issues today.**
      2. **What the whole copy earns.** 152 bodies restored, **1,373 query scans served** before eviction
         took them — **9.03 scans per body** — across **5.66 distinct tenants per body**. A selective
         fetch serves one tenant's slice, so the same work is 5.66 separate fetches: **6.66 requests per
         body with the footer cached, 11.3 without, against 1.**

      So the trade, measured: **requests ×6.7, bytes ×0.37.** Not ×0.065 — the 93% over-fetch is not
      waste but sharing, and 5.66 of a part's 15.2 tenants collect on it before eviction. The axis it
      saves on is the one with no price and the axis it spends on is the one the whole layout was
      designed around (`docs/ARCHITECTURE.md`, Class A costs).

      **And the row-group axis turns out not to exist.** Selected row groups equal the tenant's segment
      exactly — 5,344 of 5,344 — so time and label selection pruned nothing beyond the tenant, because a
      tenant holds **1.2 row groups of a 17.8-group part**. That retracts the claim above that this
      "costs a single-tenant deployment the same way": with one tenant `present == tenant` and there is
      nothing to narrow. **The only over-fetch axis is the tenant axis, and it is the same axis that
      pays for itself through sharing.**

      *What the run does show as waste, on the other side of the ledger:* **23 of 152 restored bodies
      (15%) were never read by any scan** — admitted by `candidate_part_ids`, downloaded whole, and then
      either skipped by the scan frontier or found to select no row group. That is a restore-admission
      question, not a range-read one, and it is the one number here that says something is being paid
      for nothing.

      *Scope of these numbers:* they are the pressured regime's. Reuse is a function of the query mix and
      of 16 tenants, and at a 256 MiB cache the same run restores twice — so the regime where this item
      exists at all is the one measured, and the numbers do not transfer to a deployment that states a
      different query distribution.
- [ ] **Restores that no scan reads.** The run that closed the item above found the one thing in it that is
      paid for and returns nothing: **23 of 152 restored bodies (15%) were downloaded whole and then read by
      no scan at all** — not a partial read, none. A restore is admitted by `candidate_part_ids_with_exact_fields`
      and `may_match_exact_fields` under `pin_query_parts` (`src/query/restore.rs`), and the scan afterwards
      applies two things the admission did not: the frontier, which stops once the sink holds `limit` rows
      whose worst is ahead of the part's whole segment, and the row-group selection, which can come back
      empty on a part a bloom said "may match".

      **Which of the two it is, is not measured, and it decides where the fix goes.** A frontier miss is an
      ordering problem — the restore is issued before the scan knows it will not need the part, and the
      answer is on the admission side. An empty selection after a may-match is a bloom false positive, and
      the answer is in pruning. They are different work in different files, and 15% is not enough to pick
      one. **Splitting it is one counter** on the same meter (`src/restore_meter.rs` already knows which
      bodies went unread; it does not know why), so measure before building — the same order that turned
      the item above around.
- [x] Improve metric evaluation from bounded in-memory computation to streaming — the scan takes a row
      sink, and the metric path folds each row into per-series samples as it arrives instead of collecting
      `Vec<(SharedLabels, LogEntry)>` for the evaluator to discard one step later. The log paths keep the
      collecting sink and go through the same scan, because that is where the deletion mask lives. The
      `sum(rate(...))`-shaped fast path that materializes nothing at all stays where it was, in front of
      it. `SIGNY_MAX_METRIC_ROWS` is removed: it capped an intermediate, so a `rate()` over a busy
      stream was refused for materializing something the client would never receive. Cost is still bounded
      by `max_query_scan_bytes` (per part, not after the fact), `max_query_runtime`, `max_metric_series`
      and `max_metric_samples`. Pinned by
      `a_metric_query_folds_rows_a_log_query_could_not_materialize`; the run that motivated it is
      [`docs/LOAD_RESULTS.md`](docs/LOAD_RESULTS.md) §12, whose magnitudes are struck with the rest of the
      retired harness's
- [x] ~~Validate a deployment environment using real S3~~ — **confirmed out of scope.** This is an indie project,
      so local MinIO is the upper bound for load validation. What is validated, what risks remain, and what to
      check on the first real deployment are in [`docs/LOAD_VALIDATION.md`](docs/LOAD_VALIDATION.md)

## P3 — M5 operational validation

- [ ] Tune compaction
- [x] Fix the unit mismatch between `merge_max_input_bytes` and `merge_max_memory_bytes`. Record
      `materialized_bytes` in part metadata so group selection and read budgets use the same unit, and have
      `validate` enforce `merge_max_input_bytes <= merge_max_memory_bytes`.
- [x] Implement retention policies and expired-data deletion (including a separate retention timeout knob)
- [ ] Tune resource limits such as query memory, range, and concurrency to operational targets. **The
      measurement exists now** — [`docs/LOAD_RESULTS.md`](docs/LOAD_RESULTS.md) §11, a query-heavy profile
      (`scripts/run_query_load_local.sh`) that fills all 8 scan slots with 2,403 scans queued behind them.
      It says the knob to change is **not** a memory cap: peak RSS at full saturation is 496 MB, 9.2% of the
      configured 5 GiB, so `MAX_QUERY_MEMORY_BYTES` is nowhere near binding. What it found instead is
      head-of-line blocking: the scheduler admits by arrival order, so a 60-second dashboard query waited
      behind eight 120-second scans at p95 6.46 s against a 2 s target. **Then the metric fold (§12) took
      the same probe to 546 ms without touching the scheduler**, by making the queries ahead of it six
      times cheaper. p99 barely moved (8.59 → 8.12 s), so the blocking is still the shape of the tail —
      it is no longer the binding problem. **Deferred to the range-GET work on purpose**: that changes
      what a scan costs again, and picking a separation policy against a cost about to move is tuning
      with more steps
- [x] **Tier D duration/scale run** — 2.01 hours, 500 tenants, a graceful shutdown, a restart and a fence,
      every behavioural gate met. It found one defect nothing shorter could: shutdown waited out merge
      groups it started *after* the signal, 117 of the 118 seconds it took. The 10,000-part axis is
      measured separately in §8, because reaching it means effectively disabling merge, which is a
      different question from steady-state stability. Results: [`docs/LOAD_RESULTS.md`](docs/LOAD_RESULTS.md) §10
- [x] **CAS preflight** — verify at startup that conditional writes are enforced and refuse startup otherwise.
      Running against the deployment target itself resolves what local validation could not answer.
- [x] **Measure object-store operation counts** — `signy_object_store_operations_total` counts every
      request by kind, and a test pins publication at four PUTs per part plus one GET and one PUT for the
      manifest, independent of manifest size. Measured numbers and what they say about the bill (the flush
      interval sets the Class A cost, and the current default does not fit the $1 budget) are in
      [`docs/LOAD_RESULTS.md`](docs/LOAD_RESULTS.md) §9. Retirement costs four DELETEs per part (one per
      immutable file, unbatched) plus two LISTs per orphan sweep
- [x] Document load-test results and bottlenecks — [`docs/LOAD_RESULTS.md`](docs/LOAD_RESULTS.md).
      Keep reproducible facts in tests and quote only numbers in the document.
- [x] **Mitigate N3** — measured, and the mitigation already existed. The 24.7x is a function of
      rows-per-tenant-per-part, not of tenant count: merge cuts (tenant, part) pairs 3.6x and
      parts-per-tenant from 6.6 to 1.85, amortising the ratio to ~1.07x. At 10,099 parts the resident
      sidecar cost is 18.7 MB, not the 407 MB first extrapolated, because parts that fragmented are also
      small. The binding memory constraint at this scale is the merge budget, which is hundreds of
      megabytes against the sidecars' single digits — the opposite of the working assumption. Numbers in
      [`docs/LOAD_RESULTS.md`](docs/LOAD_RESULTS.md) §2, §6 and §8, with the two refuted hypotheses kept.
- [x] Improve the load probe to verify rows read — the probe counts the lines returned, so "restored and
      read" is distinguishable from "nothing matched"

## P2 — Loki API surface

**The surface this section tracked was removed** with the read-path decision
(issue #3, `docs/M12_IMPLEMENTATION_PLAN.md`): the first-party API replaced
every `/loki/api/v1/*` route, and the Tempo routes went with it. The whole
class of Loki-compatibility work — matching another system's parameter
semantics, response quirks and boundary conventions — died with the surface.
The items below stay as the record of what was done while the surface lived.

- [x] ~~**`query_range`'s `end` is inclusive and Loki's is exclusive**, and **`| json` does not promote extracted
      fields into a log response's stream labels**.~~ Both were fixed while the surface lived (see "Open
      correctness defects"); the boundary contract (`[start, end)`) and the field-merge semantics survive in
      the first-party `/logs` endpoint and its tests

- [x] `patterns` — a read-time miner over a bounded sample of the window, reporting the lines it
      looked at. No index is added to the write path
- [x] `delete` API — hides on acceptance at the single scan every read path funnels through, removes
      the bytes at the next rewrite. Design and the reasons for each refusal:
      [`docs/RETENTION_DESIGN.md`](docs/RETENTION_DESIGN.md)

## Decided against — not pending work

These are settled decisions, not unfinished items. They are listed so the reason survives, and kept out
of the checklists above so the open list means "not done yet" and nothing else.

- **Durable monthly usage accounting** — belongs to the control plane, not the instance. A month spans
  instances and outlives them. This side exports per-tenant usage (`GET .../tenants/{tenant}/usage`) and
  the control plane accounts for it. Nothing further is owed here.
- **Exact-field pruning for empty-string equality and `_extracted` collisions** — deliberately
  conservative. An empty equality also matches an absent field, and absence is not indexed anywhere, so
  pruning on it would drop matching rows. The conservatism is the correct behaviour, not a gap. The
  *naming* half is settled too: measured against `grafana/loki:3.3.2`, a `| json` field colliding with a
  stream label becomes `<name>_extracted` (both survive, both filter) and one colliding with a pushed
  structured-metadata key is discarded outright, which signy now matches — see the `| json` entry
  under "Open correctness defects". One deliberate divergence remains: on a *second* collision Loki
  appends `_extracted` once more and overwrites the existing `foo_extracted` stream label, while
  signy answers `foo_extracted_2` and keeps it, because it will not drop a value it was given. A
  name that could have been synthesized that way must therefore never drive a row-group prune, which is
  what `query::tests::synthesized_extracted_field_never_false_negative_prunes_parts` and
  `..._restores_an_evicted_part_conservatively` hold.

Decisions already recorded as closed items above, for cross-reference: `(tier, day)` partitioning
(rejected, see Multi-tenancy), validation against real S3 (out of scope, P2), and TLS (unsupported by
architecture decision, P0).

## Deferred with a reason

- [ ] **P1-11: manifest as generational deltas plus periodic snapshots.** The manifest is still rewritten in
      full on every publish, so one object takes every commit. What that costs is now measured
      ([`docs/LOAD_RESULTS.md`](docs/LOAD_RESULTS.md) §9): one PUT and one GET per flush, on one key,
      whose rate is the flush interval. **Not the startup bottleneck** — §8 measured startup at 10,099
      parts and found the cost is local checksum validation, not manifest I/O, so the O(N) half of P1-11
      is already answered.
      What remains is the hot key, and the fix that actually removes it is not "deltas" but making each
      commit a `PutMode::Create` at its own generation key, so no key takes two writes. That replaces the
      commit protocol every durability guarantee here rests on — lost-update protection, merge input
      revalidation, writer fencing, the cross-domain flush transaction — and it cannot be validated by
      anything short of another soak. It is listed in
      [`docs/LOAD_VALIDATION.md`](docs/LOAD_VALIDATION.md) as the *response to* observed throttling on the
      first real deployment, not as a precondition for it, and that is the right order: the mitigation
      should be built when there is a measurement saying it is needed, against the backend that produced
      the measurement.

## P4 — M6 hardware replacement

Detailed plan: [`docs/M6_IMPLEMENTATION_PLAN.md`](docs/M6_IMPLEMENTATION_PLAN.md)

- [x] Implement graceful-shutdown handler (SIGTERM/SIGINT starts the drain sequence, which owns process termination)
- [x] Block ingest: Loki push 503 and OTLP UNAVAILABLE while draining (before journal append)
- [x] in-flight drain: axum `with_graceful_shutdown` + tonic `serve_with_shutdown`
- [x] Final force-flush after background workers (flush/merge/retention/eviction) shut down normally
- [x] Implement force-flush: ignore thresholds, drain MemTable/pending checkpoint, and wait for S3 upload/manifest update completion
- [x] Infinite retry + stdout warning on persistent object-store failure; exit only from operator stdin input (no hard timeout)
- [x] Automatic lossless recovery through journal replay after forced termination and restart
- [x] Drain-status readiness: `/ready` 503 while draining + pending bytes/flush completion exposed in `/metrics`
- [x] Hardware replacement rehearsal (new instance resumes traffic without loss)
- [x] Fresh-context review (remaining gates) — `docs/PRODUCTION_READINESS_REVIEW_2026-07-26.md`

## P5 — M7 local S3 load validation

Detailed plan: [`docs/M7_IMPLEMENTATION_PLAN.md`](docs/M7_IMPLEMENTATION_PLAN.md)

- [x] Strengthen observability gauges (add merge-debt gauge; active part count, WAL backlog, and MemTable bytes already exist in `/metrics`)
- [x] Tier B: wrap `LatencyFaultStore` + `from_url` opt-in (in-process latency/fault injection, reproducible seed)
- [x] ~~Tier C: MinIO~~ — **removed.** Do not test against S3 because the `object_store` crate is trusted
- [x] ~~Verify MinIO manifest CAS~~ — replaced by startup preflight, which checks the deployment target store itself
- [x] Improve load harness: target-rate pacing, separate warmup/steady state, forced eviction→restore, pass/fail against targets (`src/bin/load.rs`)
- [x] Document results, machine profile, and bottlenecks in `docs/M7_LOAD_RESULTS.md`
- [x] **BLOCKER — fix the infinite WAL compaction wedge (complete):** After the first compaction, the phase-2
  compaction-state file was not removed. After the coordinate system reset, the compaction offset was compared
  with a stale offset and the flush loop permanently wedged with `"WAL compaction checkpoint moved backwards"`.
  It reproduced without fault injection in both Tier B (`file://`) and Tier C (MinIO). It occurred only on the
  object-store backend path (local-only mode uses `set_checkpoint`). Before this fix, the M7 acceptance run
  "lossless recovery + bounded backlog" could not pass.
