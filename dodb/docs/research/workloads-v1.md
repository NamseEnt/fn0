# Product workloads v1

Status: required coverage established 2026-10-12; generator and run manifests
are pending. These are synthetic application scenarios, not production replay.
Use with [the roadmap](single-instance-product-roadmap.md) and
[backend contract](backend-contract.md).

## Required mixes

Percentages are read/write in that order. A write means one public mutation
request, including a whole Transact request; it is not one changed key.

| Profile | Read/write | Role |
| --- | --- | --- |
| RW95 | 95/5 | Preserve the previous read-heavy product case. |
| RW80 | 80/20 | User-required ordinary read-heavy application traffic. |
| RW50 | 50/50 | Balanced application traffic. |
| RW20 | 20/80 | User-required write-heavy application traffic. |

All four are mandatory in the first product round. Other mixes such as 90/10
and 10/90 can extend coverage; they do not replace these profiles. Ratios are
frozen coverage, not production measurements. Reducing repetitions or duration
must not silently remove RW80 or RW20 again.

## Application actions

Freeze the following initial synthetic recipe before examining results.
Weights apply within the read or write category, independently of RW profile.

| Category | Action | Weight | Semantics |
| --- | --- | ---: | --- |
| Read | Get | 80% | Include present and missing states with revisions. |
| Read | Query16 | 20% | Distributed PK/cursor positions; check exact ordered rows. |
| Write | Put | 20% | Unconditional single-key replacement with changing value bytes. |
| Write | Delete | 10% | Include existing and already missing keys. |
| Write | Conditional single-key write | 30% | RevisionEquals/Exists/NotExists; generate valid success and deliberate stale/absence failures. |
| Write | Transact width 4 | 30% | Conditional multi-key Put/Delete, with at least one unmodified condition key in part of the stream. |
| Write | Transact width 8 | 10% | Atomic multi-key work with conditions; preserve coverage without a separate full matrix. |

Use deterministic, bounded blocks to realize weights; round only as documented
and report actual admitted/successful mixes. The exact recipe is a starting
assumption, not a claim about user traffic. Width two remains a conformance and
focused follow-up case. Scan is a conformance/regression case, not a replacement
for short Query.

Unconditional and conditional success/failure rates must be separately visible.
A declared fraction of condition-bearing requests should deliberately reject
without applying writes; initially target 10% deliberate rejection, with the
actual value recorded. Additional naturally occurring contention conflicts are
separate. Do not count valid rejection as successful mutation throughput or as
infrastructure error.

Initial keys are arbitrary binary PK/SK pairs with about 16 raw bytes combined;
record their canonical encoded size separately. Start with 10,000 live rows per
active tenant and 512-byte changing values, sufficient for the first resident
round. Record exact data/metadata/tombstone size and initialized rows. Follow-up
coverage adds 2 KiB and 8 KiB values, growth, and non-resident data without
expanding every dimension at once. Never call engine-cache pressure proof of a
larger-than-RAM workload.

## Stateful request generation

Revisions differ between engines. Generate logical dependencies such as
"expect the revision returned by read request 417", then resolve that reference
using each backend's own response. Input equivalence compares the logical
operation, keys, values, condition kind, and dependency identity, not numeric
physical revisions. Record actual input fingerprints on common request-index
prefixes; do not compare final hashes of different-length runs.

A Get performed to prepare RevisionEquals is a real application request. Count
it in the offered read/write ratio, end-to-end workflow latency, and operation
totals. Never hide preparatory reads, retries, revision reservations, or syncs
outside the application cost. Distinguish public API-call rates from complete
workflow rates and engine-internal reads. A workflow may be a read followed by
a conditional multi-key transaction; report both calls and the full workflow.

Initially one quarter of condition-bearing requests use a read-derived revision
predicate, one half use Exists, and one quarter use NotExists. A read-derived
request consumes at most one prerequisite Get; other dependencies use presence
conditions or already recorded observations. This makes the RW20 read budget
feasible: 80% writes times 70% condition-bearing writes times 25% read-derived
requests needs at most 14 prerequisite Gets per 100 calls, within the 16 Get
calls supplied by its 20% read category. Schedule those dependencies inside the
read quota; do not issue extra hidden reads or deadlock waiting for a quota
that has already been exhausted. Deliberately stale references reuse recorded
older observations.

Freeze deterministic no-contention and deliberate-stale correctness streams
first. Concurrent streams may legitimately finish with different conflict
outcomes and completion order. Verify a valid history rather than forcing the
same winner or treating every mismatch as corruption. Preserve per-backend
attempts and report their actual successful read/write shares.

Use the existing reference model and bounded histories; rejected or uncertain
validation cannot be marked passed. Correctness smoke must prove actual
conditional requests, width-four/eight commits, deliberate condition failures,
and multiple simultaneously opened tenant DBs before any product table runs.

## First single-tenant comparison

Use RW95, RW80, RW50, RW20 with each of two declared product backends through
the same `DodbService` boundary. One tenant DB, 16 external clients on OCI;
on Debian choose and record a common client count that exercises the machine.
Use the same logical input and declared worker/resource settings for paired
backends within each host. Internal engine scheduling can differ; document it.

Initial timing is 1 s warmup and 5 s measurement, one repetition: four profiles
times two backends = eight records per host. Run each backend process alone;
it may serve many client requests concurrently. Alternate backend order across
profiles. Admission stops at the deadline; include outstanding completions in
the recorded drain and elapsed-time denominator.

These records are preliminary. Repeat decision-critical profiles using matched
seed pairs. Repetitions may use different seeds; never discard valid pairs
solely because the repetition seed changes. Do not add a third engine or extra
repetitions to every cell without a question that requires them.

## First multi-tenant comparison

The initial comparison uses one MT-MIX run per backend. Each run is one service
process managing four independent tenant DBs simultaneously, with one tenant
running each RW profile (RW95, RW80, RW50, RW20). Use 16 external clients total,
four clients assigned to each tenant, 10,000 live rows per tenant, 16-byte
combined raw keys, and 512-byte changing values. Use seed `20261013` for each
paired backend input and deterministic tenant substreams. Cap each run at
1,000,000 public API calls, including prerequisite Gets; the five-second
measurement deadline normally stops admission first. Use the same closed-loop
request generator and fixed client allocation for both backends. This is four
active/open tenants; it does not claim larger tenant capacity or lifecycle
readiness. Seed the same logical tenant data before timing.

The initial single-tenant and MT-MIX comparison therefore has ten records:
four profiles times two backends, plus one four-profile MT-MIX record per
backend. Every run uses one-second warmup, five-second measurement, and one
repetition, seed `20261013`, and a 1,000,000 public API-call ceiling per run.
It is preliminary evidence only. Record API-call attempts and
successes, successful write transactions, mutations, expected condition
rejections, conflicts, preparatory Gets, and per-tenant outputs separately.
Condition-generating Gets are public service requests and are included in the
same 16-client workload accounting. Both backends receive the same fixed seed
and logical input stream within each paired record.

The profiles below remain follow-up multi-tenant cases after the initial round.
Each run is one service process managing independent tenant DBs simultaneously.
Measure per tenant and per workload, not just total ops/s. Seed only active
tenant datasets to avoid turning a mostly idle scenario into a bulk load.

| Scenario | Persistent/open/active DBs | Traffic | Required observations |
| --- | --- | --- | --- |
| MT-IDLE | 100 persistent; warm 16 handles, 4 active | One active tenant per RW profile; open another idle tenant during the run | Idle/open RSS and FD cost, cold-open latency, active-tenant p99 during open, no duplicate handles. |
| MT-MIX | 16 persistent/open/active | Four tenants per RW profile with uniform keys | Per-tenant throughput/tails, shared CPU/memory/IO, imbalance and starvation. |
| MT-NOISY | 8 persistent/open/active | Seven normal tenants, rotating RW profiles; one write-heavy hot-key tenant saturates | Normal-tenant latency/throughput relative to the same seven-tenant control and the noisy tenant's conflicts/admission. |

Each scenario runs on both backends. The first two use 1 s warmup and 5 s
measurement. MT-NOISY is one run with a 5 s control phase and a 5 s interference
phase after warmup. Keep phase rates and histories separate; do not average away
the interference. Its offered-load driver must keep normal tenants at the same
declared input rate across phases and backends. Choose the rate below the lower
measured capacity before starting; record it, queues, rejections, and deadline
misses. This phase transition is not a production burst claim.

Use an instance-wide client/resource budget and record its per-tenant allocation.
Do not give each of 16 databases an independent full-machine CPU/cache budget.
Total memory accounting includes resources shared and retained by all DBs.
For hot keys specify the key-set size and access probability in the manifest.

The three scenarios above are follow-up coverage, not part of the initial ten
records. When run, retain all scenarios on both backends and report them
separately from the preliminary round. Correctness checks and failed attempts
are listed separately and are not successful benchmark records.

Later lifecycle work expands to 1,000 persistent tenants with a small active
set, exercises configured handle capacity and eviction, and measures cold-open
storms. Do not claim 1,000 open-database readiness from 100 directories or the
current maximum-open setting. Add open-loop burst/recovery, sustained mixed
writes, and cache/RAM pressure only after the small product round is valid.

## Required manifest and output

Every run records its backend source/build/options, API boundary, tenant topology,
actual open/active counts, instance limits, caller/internal workers, condition
recipe and rejection targets, seeds, actual input checkpoints, dataset shape,
filesystem/device, requested/actual timing, drain and client completion skew.

Report offered/attempted/successful API operations, successful application
workflows, write transactions, mutations, expected condition rejections,
concurrency conflicts, retries, overload, errors, validation completeness, and
per-operation/per-tenant latency. Report per-operation percentiles rather than
merging fast reads and durable writes into one latency distribution. Query
requests and returned rows are separate units.

Record measurement-window RSS completeness, CPU, file descriptors, thread count,
open latency, per-tenant queues, WAL/data growth, sync, flush, compaction, and
stall observations where available. Inspect quiet-tenant tails and total
resident cost before declaring native multi-tenant readiness. Any unsupported
metric is unavailable; a missing RSS sample or absent LSM observation is not a
zero or evidence of steady-state stability.
