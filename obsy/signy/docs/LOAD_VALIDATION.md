# Load-validation policy — local MinIO is not the target

## Decision

**This repository does not test against S3. That means neither real cloud storage nor local MinIO.**

Two different reasons lead to this decision.

- **Real cloud** — This is an indie project, and there are no plans to pay for meaningful duration and
  scale in a real account. Earlier reviews listed "at least 24 hours of sustained load on real S3" as a
  gate; that gate is retired by this decision.
- **Local MinIO** — Cost is not the issue. **We trust the `object_store` crate.** Correct S3 protocol
  implementation is that crate's responsibility, and spinning up Docker to revalidate it is not our job.
  `docker-compose.yml` and `scripts/run_load_s3.sh` were deleted accordingly.

Instead, **test our code in detail up to the point immediately before the crate is called.** The following
principle explains where that boundary is and why it is risky.

---

## Principle — test only our code

The first edition of this document described "MinIO validating the S3 wire protocol, conditional PUT
semantics, and multipart" as the core value of local validation. **That was wrong.** Everything in that
list is the `object_store` crate's responsibility; if it is wrong, it is a crate bug, and the crate has
its own test suite. Running Docker to revalidate another library is not our job.

The actual risk in our code is one layer below that.

> Does `object_store` implement conditional PUT correctly? (X — crate responsibility)
> **Does the store created by this binary with these environment variables actually perform CAS?** (O — our responsibility)

`from_url` lowercases the environment variables and passes them directly to `object_store`. There was
no code checking whether the returned store performed conditional writes. If `OBJECT_STORE_CONDITIONAL_PUT`
is wrong, every manifest guarantee in this engine — lost-update prevention, merge-input revalidation, and
writer fencing — rests on nothing.

**So this is solved by a startup preflight, not a load test**
(`ObjectStorage::verify_conditional_put`). At startup, it directly checks with a probe object that *a
write that should be rejected is rejected*, and refuses startup otherwise. The positive path proves nothing:
the first manifest write in an empty prefix succeeds whether conditions are honored or not.

### What to test closely on our side of the boundary

What we pass to the crate and how we interpret what it returns — this is our responsibility, and bugs here
are **all silent** (every write succeeds). Therefore, test this seam more tightly than any surrounding code.

| Our code | What happens if it is wrong | Test |
|---|---|---|
| `put_mode` — version → `PutMode` | If `Overwrite` replaces `Update`, **CAS disappears completely** without anyone noticing | `put_mode_conditions_on_every_backend_except_the_local_one` |
| `from_url` scheme classification | Classifying a non-`file` scheme as local has the same result | `only_the_file_scheme_opts_out_of_conditional_writes` |
| URL → prefix → key composition | Read somewhere other than where we wrote, or write over another deployment sharing the bucket | `keys_are_built_under_the_url_prefix` |
| `NotFound` interpretation | Treating a missing manifest as an error makes first boot impossible; treating **another** error as an empty manifest makes every registered part disappear | `a_missing_manifest_is_the_first_boot_and_nothing_else_is` |
| `format_version` interpretation | Treating an unknown version as "absent" discards parts registered by a newer writer | `an_unknown_manifest_format_version_is_refused` |
| Environment variables → `object_store` configuration keys | CAS configuration is not passed through | `object_store_environment_keys_are_normalized_and_explicit_values_win` |
| Whether conditional writes are actually enforced | Final defense for all of the above | `the_preflight_refuses_a_store_that_ignores_conditions` |

The first two were checked with **mutation tests**. Changing `put_mode` to always use `Overwrite` makes
only that test fail while **all existing CAS-contention tests pass** — this is exactly what silent failure
means. The preflight does not catch this mutation either (it specifies the mode directly without going
through `put_mode`). Both layers are therefore necessary.

The preflight is better than a load test for three reasons.

| | MinIO load run | Startup preflight |
|---|---|---|
| Validation target | Local MinIO | **Actual deployment target store** (R4 below is solved) |
| Cost | Docker + benchmark script | A few round trips per startup |
| Nature | Test | **Defense** — bad configuration cannot start |

`cargo test` verifies that the preflight actually blocks a fake store that ignores conditions
(`the_preflight_refuses_a_store_that_ignores_conditions`). It runs in parallel without Docker.

## What is the local load run for, then?

It tests **what our code does under load**. The backend does not matter.

| Item | Why it is ours |
|---|---|
| Whether flush/merge/retention loops keep progressing | They are all our loops. The M7 WAL-compaction wedge was found on exactly this axis |
| Whether WAL backlog is bounded and backpressure engages and clears | Our thresholds and gates |
| Whether RSS is stable | Our data structures |
| Eviction → restore round trip | Our cache policy and restore path |
| Startup time, flush latency, and query-planning time as part count grows (P1-11) | A function of **data volume and execution time**, not the backend |
| Row-group fragmentation proportional to tenant count (N3) | A property of our part format |
| Lossless graceful shutdown and writer fencing | Our sequence |

None of these items **depends on which object-store backend is used.** Therefore, a load run with
`file://` + latency injection (Tier B) is sufficient, and MinIO is optional.

### What the harness sends, since 2026-08-28

The bed used to POST `/v1/logs` and `/v1/metrics` and export traces over gRPC.
Those paths were removed — `POST /signy/api/v1/collect` is the whole write
surface now — so the harness sends what the one intended producer sends: the
same OTLP export, framed as a one-record collecty batch (the payload behind its
length, zstd over it) under `Content-Encoding: zstd` and `x-collecty-signal`.
It sends no sender or segment header: those number a collecty's queue and a
harness has none, which signy reads as "nothing to resume".

**Two numbers moved and one did not**, which matters for reading a result
against an older one:

- **`encoded_bytes` did not move.** It is the OTLP export, identical across
  every target, so the cross-system throughput and ratio axes are unchanged.
- **`wire_bytes` moved for signy and only for signy.** It is now the compressed
  frame rather than the export, so it is no longer comparable with Loki's or
  VictoriaLogs' on that axis — it is a smaller number measuring a different
  thing.
- **Push latency includes work neither the harness nor the engine did before**:
  the harness compresses each body, the engine decompresses it as it streams.
  A push p95 from before this date and one from after are not the same
  measurement. Nothing has been re-measured yet, so
  [`LOAD_RESULTS.md`](LOAD_RESULTS.md) and [`COMPARISON.md`](COMPARISON.md)
  still describe the older wire.

### Through a collecty, since 2026-09-04

The framing above is the harness standing in for a collector. `SIGNY_LOAD_PUSH_ADDR`
is the other way: writes go to a real collecty at that address as the bare
export on the signal's own path (`/v1/logs`, `/v1/traces`, `/v1/metrics`),
reads stay on `SIGNY_LOAD_ADDR`, and the batch signy reads is one collecty
built — numbered, resumable, and compressed by the collector rather than by the
harness. Unset, nothing changes.

What that adds to a run, and none of it was ever under load before:

- collecty's intake, its zstd, its segment rotation and its disk queue
- the sender's retry and backoff against an engine that is slow or gone
- the `x-collecty-sender` / `x-collecty-segment` resume, which is what stops a
  resend after a crash from being stored twice

What it costs the reading of a result:

- **A `200` no longer means the engine has the export.** It means the collector
  has it on disk. The run therefore waits for the queue to reach signy before
  it scrapes its end metrics (`SIGNY_LOAD_DRAIN_SECONDS`, 180 s) and reports
  whether that wait settled. Accounting taken before it did is not a loss
  report, and the gate says so.
- **Push latency is the collector's, not the engine's.** It is the time to get
  an export onto collecty's disk. An engine push p95 and a collector push p95
  are different measurements.
- **`wire_bytes` is the export**, uncompressed: the compression happens inside
  collecty now, where the harness cannot see it. The ratio a run through a
  collecty achieves is `collecty_bytes_appended_total` against
  `collecty_bytes_sent_total`, read back out of signy.

The trace leg has a read-back to go with it. Every tenth exported trace is kept
and asked for by id a lag later (`SIGNY_LOAD_TRACE_VERIFY_LAG_SECONDS`, 60 s),
and the span count has to match; a first 404 is asked again rather than counted,
because a backlog in front of the engine is not the engine losing a trace.

---

## What is not validated (remaining risks)

### R1. Latency-distribution tail (largest unvalidated risk)

Loopback MinIO round trips are below 1 ms and nearly flat. Real S3/R2 has a p50 of tens of milliseconds,
p99 of hundreds of milliseconds to seconds, and a thick tail. The ack path does not use object storage,
but **flush, merge, retention, and restore are all affected.** More flush latency changes WAL-backlog and
backpressure-limit tuning completely, while restore latency directly affects cache-miss query p95.

**Mitigation — this can be largely covered without Docker.** The latency-injection wrapper is backend-
agnostic, so it can provide a tail close to real S3 over `file://`. The question is "does the flush loop
progress and does the backlog remain bounded at this latency?" The backend is irrelevant to that question.

What this injection cannot do: because it occurs **above** the `object_store` client, it does not exercise
the client's own retry/backoff layer. Real S3 would absorb many 5xx responses there before the engine saw
them. Therefore, this injection is **more pessimistic** for the engine — a safe direction. The retry layer
itself belongs to the crate and is not ours to validate.

The injector is `base + uniform(0, jitter)`, so it **cannot produce a thick tail.** Values are therefore
chosen so the *maximum*, rather than the median, is near real S3 p99. The median is pessimistic, but that
is the right direction for asking whether the backlog is bounded at this latency.

### R2. Throttling

S3 limits request rates per prefix (approximately PUT 3,500/s and GET 5,500/s). Exceeding them returns
`503 SlowDown`. R2 has its own limits. **MinIO never throttles locally.**

The risk is small given the numbers. With the default configuration, the manifest is written once every
five seconds by `flush_max_interval`, or **0.2 PUT/s**. Part objects add about 1 PUT/s, and even a 10x
merge/retention burst is about 10 PUT/s. That is **four orders of magnitude** below the limit. Part-object
keys are also all different and are not hot keys — the only hot key is the manifest (P1-11), at 0.2 PUT/s.

**There is no mitigation, but it is not worth building one.** Reproducing exact `503 SlowDown` requires
placing the fault *below* the client, which requires a proxy (MinIO request-limit settings, toxiproxy at
the TCP layer without status codes, or a small reverse proxy returning S3 error XML). The calculation
above does not justify that cost. Observe it during the first real deployment.

### R3. Cost

This project's design has been dominated by R2 Class A costs (the reason tenant-per-object partitioning
was discarded, [`MULTI_TENANCY_DESIGN.md`](MULTI_TENANCY_DESIGN.md)). MinIO provides no cost signal.

**Partial mitigation:** Even though amounts cannot be measured, **operation counts can be measured locally.**
The number of PUT/GET/LIST operations per flush, merge, and retention cycle is backend-independent. Count
them and only multiplying by the pricing table remains. There is no instrumentation yet; it is recorded in `todo.md`.

### R4. Provider-specific conditional PUT semantics — **resolved**

Conditional PUT behavior differs in details between S3, R2, and MinIO. CAS working on MinIO does not
mean it works on R2.

**Resolution:** The startup preflight checks this against **the deployment target store itself**. A question
that could never be answered by running locally is answered at the only place where it can be answered.
Failure blocks startup, so there is no window in which writes are accepted under a bad configuration.

### R5. Real network failure modes

DNS failures, TLS handshake failures, connection resets, and partial responses — Tier B's in-process
injection simulates these only as `object_store::Error`; it does not reproduce them on the wire.

**Mitigation:** The engine's defenses work independently of the backend (unbounded retries + health gating
+ backpressure). Response to failure *duration* matters more than failure *type*, and Tier B validates that.

---

## Local validation procedure

The names "Tier B/C" come from [`M7_IMPLEMENTATION_PLAN.md`](M7_IMPLEMENTATION_PLAN.md) (there is no A).
That plan described Tier C's purpose as validating conditional put, path-style, and multipart behavior
that in-memory and file backends do not exercise. Under the principle above, **those are not things to
validate here**, so Tier C was removed from the gate.

### Tier B — load run (`scripts/run_load_local.sh`) — **default**

Start our server binary and inject latency and errors with `LatencyFaultStore` over `file://`. The seed is
fixed for reproducibility and no external process is needed. **It covers every item in the "our code" table
above and R1/R5.** Because the latency wrapper is backend-agnostic, it can provide a tail close to real S3 here.

### ~~Tier C — MinIO~~ — **removed**

`docker-compose.yml` and `scripts/run_load_s3.sh` were deleted because everything the plan said this tier
would validate was the responsibility of `object_store`. Trusting the crate means we do not verify its work
again; this deletion records that decision in the codebase.

`cargo test` checks **this side** of the crate boundary in parallel without Docker, as in the table above;
the preflight checks **the other side** at startup against the actual deployment target. There is no place
for MinIO between them.

### Tier D — duration and scale (new, not run)

The existing two tiers run for tens of seconds. The remaining axes (P1-11's O(N) path, N3 fragmentation,
and long-running leaks) are problems of **time and volume**, so they can be measured locally. Initial targets:

| Item | Value | Reason |
|---|---|---|
| Backend | Tier B configuration (`file://` + latency injection) | None of the items in this table depends on the backend, so Docker is unnecessary |
| **Termination condition** | **Processed event count** (`SIGNY_LOAD_EVENTS`) | Memory stability, part accumulation, and backlog trends are functions of *work processed*. Time was a proxy and a poor one — pushing harder reaches the same state sooner. Time (`LOAD_SECONDS`) is only a safety cap |
| ~~Reach 10,000 parts~~ | **Separate configuration required** | Part count stays bounded while merge works normally. Measuring P1-11 (startup time linear in part count) requires effectively disabling merge, which asks a different question from steady-state stability and cannot be mixed into the same run |
| Tenant count | At least 500 | Scale at which row-group fragmentation (N3) appears |
| Restart | At least once during the run | Measure startup time at that part count |

### Acceptance criteria

Because the machine is not the target specification (4 vCPU / 16 GiB), **absolute values are records, not
gates**. Gates are behavioral invariants.

All of these were met on the Tier D run recorded in
[`LOAD_RESULTS.md`](LOAD_RESULTS.md) §10 except the eviction→restore round
trip, which needs the separate configuration described below and was measured
on its own.

**The gates held; the records beside them are retired.** Every number that run
produced was struck when the harness it came from was retired
([`VISION.md`](VISION.md) "The ruler comes before the work", M8 in
[`todo.md`](../todo.md)). The behavioural gates survive because they are
booleans that do not depend on the machine being idle or on the corpus being
realistic — which is why they are the gates. The last item below, "record the
percentiles", is the item that has to be redone, and it cannot be redone until
the harness can measure.

- [x] Zero loss of acked data (confirmed by replay after restart)
- [x] WAL backlog is bounded — 429 appears above the limit and clears when load subsides
- [x] RSS is stable below the configured limit (no upward trend)
- [x] Flush, merge, and retention all progress (`*_success_total` increases, `*_errors_total` stays flat)
- [x] Eviction → restore round trip succeeds, with zero restore errors
- [x] Graceful shutdown is lossless (M6 rehearsal) — and the run found that it took 118 seconds because
      merge kept starting groups after the signal, now fixed
- [x] When two instances use the same prefix, the old instance is fenced and exits abnormally
- [x] **Record** p50/p95/p99, RSS, part count, and startup time (do not gate on them)

#### Observing eviction → restore

This is not observed with the default configuration. Merge consolidates recent parts every eight seconds,
and its result was just written locally so it always exists; retention deletes old data after 20 seconds,
leaving the probe to query an empty range. Both behaviors are normal on their own.

Change all three settings together to observe it.

```
SIGNY_MERGE_INTERVAL=3600s            # let parts accumulate
SIGNY_LOAD_TENANT_RETENTION=infinite  # retain old data for the probe
SIGNY_CACHE_MAX_BYTES=524288          # make the working set exceed the cache
SIGNY_LOAD_RESTORE_LOOKBACK_SECONDS=40
```

With this configuration, `restore_observed: true` and zero restore errors were observed. The counts and
restore-latency percentiles that used to be quoted here are struck with the rest of the harness's output
([`LOAD_RESULTS.md`](LOAD_RESULTS.md) §5); the percentiles in particular were over a handful of samples.

The probe now counts the lines that came back (`restore_probe_rows`,
`restore_probes_with_rows`), so "restored and read" is distinguishable from "nothing matched" without
inferring it from a server-side counter. It is reported rather than gated: with retention on, the probe's
window is legitimately empty, and a zero there describes the configuration rather than a failure.

Record measurements in [`LOAD_RESULTS.md`](LOAD_RESULTS.md). Numbers kept only in chat or a terminal
disappear and must be measured again. Always record the machine profile, build revision, and seed with the
result, and never call values from a non-target machine a pass against the target.

**That document is retired and holds no numbers today.** Recording the machine profile, the build and the
seed was never enough: the run that produced them has to be capable of measuring what it reports. New
percentiles go into a new section of a document that begins after the ruler exists.

The rewritten harness (`src/bin/load/`) is that ruler, and it records what a rerun needs without being
asked: the build revision, the seed, every workload knob, every `SIGNY_*` the server was started with,
the machine profile, and the corpus's measured Parquet compression ratio. Three of its outputs change how a
result is read.

- **Two latencies per request.** `service` starts at the actual send; `response` starts at the *intended*
  send. Quote `response`; `service` is only interpretable next to it. Their gap, in
  `coordinated_omission`, is how much of the offered rate the client could not issue.
- **A percentile that the sample count cannot support is `null`,** with the count and the requirement
  beside it. A gate on a `null` fails. Do not quote a percentile without its `count`.
- **429 is not an error and not a pass.** It is `throttled_rate`, gated separately from `error_rate`. A run
  that was refused most of what it offered did not measure the rate it names.

**Keep reproducible facts in tests, not in prose.** Keep runs only for things that require load — whether
backpressure engages and clears, timestamp boundaries, and fragmentation ratios are deterministic, so
`cargo test` checks them and this document quotes only numbers.

---

## First real-deployment procedure

Completing local validation does not remove R1–R5. When connecting for the first time, do the following in order.

1. **CAS check — automatic (R4).** The preflight checks itself at startup and refuses to run if it fails.
   The startup-rejection message says what to configure. Nothing is required from a person.
2. **Small canary (R1).** Send only a fraction of real traffic and watch `remote_restore_latency_ns_total`
   and `flush_errors_total`. If their scale differs from local measurements, retune `flush_max_interval`
   and the cache limit to that scale.
3. **Observe throttling (R2).** The calculation suggests it is unlikely, but it will not be silent if it
   occurs — growth in `flush_errors_total` comes with `503`/`SlowDown`. The first response is to increase
   the flush interval; the fundamental response is to change the manifest to generational deltas plus
   periodic snapshots (P1-11).
4. **Check cost (R3).** Check the first billing cycle. If local operation-count instrumentation exists,
   compare it with the estimate; if not, this is a reason to add it.

Until these four items are confirmed, **do not make this engine the only copy of operational observability data.**
