# Bounded staggered E2 resident-population frontier

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

This is the next design after [E2-ARRIVAL-CONTROLS-MATRIX.md](E2-ARRIVAL-CONTROLS-MATRIX.md). It does not assert that the running controls have completed. Execution requires their verified terminal evidence, a separate frozen controller commit, and a fresh space check. The primary owns design and acceptance; GPT-6 Luna at high reasoning effort owns implementation and returns concise summaries.

## Question and invariants

Find bounded tested active-population brackets for small stores with deterministic staggered arrivals. Do not assume that synchronized bursts identify a resident-memory maximum, that either backend wins, or that a thirty-second screen is an accepted capacity result.

Reuse clean runtime source `cb0e0448270c1da3d9a214fb008ec01406d8e40d` and the exact preserved default binaries from `arrival-cb0e04482-r1`. The passed eighty-five native checks, unchanged seventeen advisory Clippy diagnostics, and A1/default links may be reused only after their provenance and binary hashes are verified. Do not rebuild or repeat passed native checks for a reporting change.

Use new namespace `resident-frontier-cb0e04482-r1`. Retain E2 rows 2,048, values 512 bytes, seed 20261027, mixed recipe and widths, synchronous durability, revision and condition semantics, global and queue caps eight, fifty calls/s per active tenant, client count one, both full-population oracles, all native shared/cache/WBM and B-link checkpoint options, 256/192 MiB max/high, zero backend swap, and existing request/drain/shutdown/watchdog limits. All registered tenants are active; there are no opened-idle tenants. Every new cell uses staggered phase and 5,000 ms pressure sampling. Preserve the 8 MiB / 4,096 observation bounds and invalidate truncation.

Use the same process-local soft open-file limit 65,536, preserve the hard limit, verify a separate owned child before creating workloads, and record actual driver/descendant inheritance. Keep the controller outside the backend cgroup. Do not change host ARC/swap policy, host-wide limits, native per-database settings, defaults, source, or the earlier namespaces.

## Entry gate and deterministic screens

The entire sixteen-cell arrival-control cohort must be complete and operationally valid, with both oracles and tiny gates passing, verified source/binaries/limits, all children exited, and all groups empty. Ordinary control SLO misses are allowed at entry; an operationally failed or incomplete control cohort blocks this phase until a separate design addresses that failure.

A route has an accepted twenty-eight-tenant baseline only if all three of its staggered sixty-second controls passed every per-tenant SLO. Those three distinct cells may be reused as the accepted baseline; do not rerun them for aggregation.

For each route with this accepted baseline, screen 32, 48, 64, 80, and 96 active tenants in ascending order. For a route without it, screen 16 and 24 in ascending order. Execute the union of required counts in ascending order, one new thirty-second cell per eligible route/count. Alternate the first backend for each count group, including groups with only one eligible route; record every omitted case and its rule.

After the first ordinary SLO miss on a route, omit its higher screen counts. The other route may continue. This conservative stopping rule limits resource use; it does not prove that all untested larger counts would fail. Preserve any observed non-monotonicity between prior controls and new screens rather than smoothing it away.

## Confirmations and one fallback

For each route with a passing screen, select its highest passing screen count as the candidate. Run up to three new sixty-second repetitions at that exact count. Alternate backend order by confirmation round when both routes remain eligible; different tenant counts are capacity observations, not matched throughput pairs. An ordinary SLO miss makes that candidate ineligible immediately, and its remaining repetitions are omitted. Preserve the failed repetition; do not repeat the same condition to obtain a passing median.

If the candidate fails this confirmation, permit exactly one fallback: the next lower passing screen count, if one exists. Run up to three fresh sixty-second repetitions there with the same immediate SLO-stop rule. If no lower passing screen exists, retain an already accepted twenty-eight-tenant control baseline when available; otherwise report no accepted count. After a fallback miss, do not search additional counts. A candidate or fallback is accepted only after all three fresh repetitions pass; partial repetitions and thirty-second screens never count toward acceptance. If every larger candidate fails, the preserved accepted control baseline remains valid for its original cohort.

At most ten screen cells and twelve confirmation/fallback cells can run. No new cell repeats an earlier operationally failed condition. Compare exact recipe/fingerprint pairs only when backend, population, duration, mode, and other workload axes permit it. For differing accepted populations, report their tested brackets and the fixed per-tenant demand instead of a raw throughput ranking.

## Global stops, sizing, and limits

Any operational failure, watchdog, OOM, oracle error, provenance/identity mismatch, observation truncation, or host guard failure stops the entire new cohort immediately. Do not proceed to confirmations or fallback after such a failure. Preserve available unvalidated counters and the complete failed database/logs; never invent zero throughput or turn a diagnostic receipt into an accepted cell. Collect and review the stopped cohort without rerunning its cells.

Before execution, require at least 12 GiB benchmark space and 1.5 GiB host MemAvailable, and retain the existing 10 GiB / 1.5 GiB runtime guards and 1,200-second per-cell controller cap. Before the arrival controls, the host had approximately 26.7 GiB benchmark space free. Their first completed twenty-eight-tenant measurement observations were about 70 MiB B-link and 86 MiB shared RocksDB cgroup peak, with raw native histories about 16 KiB and 610 KiB at thirteen samples. These are sizing observations only. The new count cap and cadence must fit retained observation limits; fresh measured space after collecting the controls remains authoritative. Reserve up to 8 GiB for the bounded new databases, binaries, and evidence, with all prior data preserved.

Longer maintenance is deliberately a separate matrix after this frontier is reviewed. It requires an exact common accepted population, provenance, fresh sizing, and a sampling cadence that fits the fixed observation bound. This document does not authorize larger-than-budget/cold-storage/growing-churn cells or architecture changes.

The result is a bounded staggered small-store service comparison under backend cgroup accounting. Driver and uncharged host ARC remain outside that budget. Any advantage is conditional on these stores, arrival pattern, workload, wrapper/admission policy, and resource scope. It is not an engine maximum, physical host-memory advantage, disk-return result, or general B-link adoption decision. Issue #126 remains open.
