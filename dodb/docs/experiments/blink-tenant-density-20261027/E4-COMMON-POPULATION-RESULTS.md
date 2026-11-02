# E4 matched fixed-population maintenance results

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Both routes passed all three five-minute repetitions at twenty-four continuously active tenants. The six fresh cells completed 2,160,000 successful scheduled requests and twelve full-population/reopen oracles, with matched request/schedule fingerprints in all three pairs. This extends the finite-window maintenance evidence. It does not establish greater B-link density, steady state, a maximum tenant count, or physical disk efficiency.

## Measured results

| Route | Five-minute SLO passes | Whole-cell cgroup peaks, MiB (r1 / r2 / r3) | Allocated reachable files after reopen, MiB (r1 / r2 / r3) |
| --- | ---: | --- | --- |
| B-link | 3/3 | 62.32 / 61.79 / 60.38 | 92.76 / 92.76 / 92.75 |
| Shared RocksDB | 3/3 | 101.02 / 100.07 / 97.60 | 44.01 / 44.01 / 43.63 |

All six cells had zero high/max/OOM events, zero request errors/conflicts/drops/timeouts, both oracles passing, and zero queued/in-flight requests after drain. One or two requests remained outstanding at the offer boundary in each cell and completed during drain; every tenant met the unchanged offer-rate objective. Do not substitute final drained completion for offer-boundary rate accounting.

Every repetition's worst-tenant Get and Query p99 histogram upper-exclusive edge was 4.194304 ms. Write was 16.777216 ms for B-link and 8.388608 ms for the reference. These are histogram bucket edges, not exact percentile timings or an uninstrumented engine ranking. Both routes met the unchanged experimental histogram SLO reducer.

Each B-link measurement worker completed 216 checkpoints; periodic cumulative counts increased by 216, with no failed checkpoints or omitted checkpoint events. This supplies repeated maintenance-cycle coverage. The counters include that worker's preflight/drain/shutdown and are not exact offer-boundary deltas. The frozen reference report does not expose a cumulative flush/compaction cycle count; its native shared-resource/property snapshots are retained without inventing one.

The observed cgroup memory series did not establish a plateau. B-link's first-to-last periodic measurement samples were 53.11->62.00, 53.57->61.36, and 52.61->59.82 MiB. The reference was 23.92->100.73, 23.74->99.61, and 23.94->96.58 MiB. Warmup, retained data/diagnostics, native cache growth, and allocation-level attribution were not isolated. Passing five minutes does not prove bounded long-term memory, debt, or backlog.

B-link used less charged backend memory, while the reference had smaller post-reopen file allocation and a lower write-p99 bucket edge in these matched cells. File allocation is Unix st_blocks for reachable files at a named phase, not a continuous peak, total ZFS usage, physical writes, or space returned after deletion. The comparison keeps compression disabled. Driver memory and uncharged/shared host ARC remain outside the enforced backend budget. Raw offer-boundary worker/process/device observations remain available with their original scope.

## Protocol and provenance

The separately committed [plan](E4-COMMON-POPULATION-PLAN.md) is `a01f6e2d7ce3411366bf9b97a9a3addf4b12e614`; the executing controller is `476c692186e4736a5b314c10e5034cb0320d01db`. Runtime source and exact default binaries remain `cb0e0448270c1da3d9a214fb008ec01406d8e40d`, with A1 disabled. Frozen source/native/binary/helper hashes, clean source, prior exited identities/empty cgroups, and descriptor inheritance passed entry checks. The preserved 85-test native validation and both feature links were reused by exact provenance; no engine rebuild or native-test rerun was needed for this controller/reporting increment.

New namespace `maintenance-cb0e04482-r1` ran B-link/reference, reference/B-link, B-link/reference. Every cell used N24 active=registered, 2,048 initial rows/tenant, 512-byte values, seed 20261027, fifty calls/s/tenant, staggered arrivals, the mixed recipe/widths, caps eight, client count one, 256/192 MiB cgroup max/high, zero backend swap, 5,000 ms sampling, and unchanged native/checkpoint settings and timeouts. The largest native history was 2,619,117 bytes, below 8 MiB; maximum observed measurement sample gap was 5,003 ms. No observations were truncated and no operational stop or fallback occurred.

The original post-run derived review rejected the three reference reductions because it attempted to read an absent B-link checkpoint counter. Its original `final-review.json` and launcher output remain inside the archive. The reporting-only fix is `04a1ba5b0`; corrected reviewer/collector source is preserved at `67119c8c5`. [Corrected final review](evidence/maintenance/final-review.json) is an exact copy of onsite `review-r2.json`: `completed_run_evidence_valid: true`, `accepted_run: true`. No workload was rerun or omitted to fix reporting. The local verifier additionally recomputes aggregate accounting/acceptance and checks original/corrected controller, collector, and binary identities.

## Collection and reproduction

[Collection receipt](evidence/maintenance/collection.json) verifies six preserved databases, thirty exited workload identities, 31 empty cgroups, unchanged native/binary identities, and clean frozen source. Collection retained 26,778,771,456 benchmark bytes free and 4,988,465,152 host MemAvailable bytes, above the preserved resource guards. All prior data and this cohort's databases remain on OCI; no cleanup occurred.

The 14,588,171-byte [runtime archive](evidence/maintenance/runtime.tar.gz) has SHA256 `7bd588c9bee02f7b81d6662bd894d3367391f74ec3b302d404c2b13eb5bbc30b`. Its 57 regular files include both controller/reviewer source versions, raw commands/logs/reports, original and corrected reductions, and exact runtime binaries. Extracted binaries remain locally preserved and ignored by their exact namespace path; they remain inside the archive. [Local verification](evidence/maintenance/collection-verification.json) matches every archived/extracted file hash, all six recipes/reductions, three paired fingerprints, aggregate acceptance, and original/corrected source and binary identities.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 dodb/docs/experiments/blink-tenant-density-20261027/evidence/maintenance/collect_maintenance.py --verify-local dodb/docs/experiments/blink-tenant-density-20261027/evidence/maintenance
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s dodb/docs/experiments/blink-tenant-density-20261027/evidence/maintenance
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s dodb/docs/experiments/blink-tenant-density-20261027/evidence/resident-frontier
```

All eight maintenance checks and fifteen existing frontier checks pass. The local verifier reproduces collected evidence, while the onsite collector additionally verifies live database/cgroup preservation and process liveness. Do not relaunch the controller against the preserved namespace or repeat accepted cells to reproduce a report.

## Decision and remaining gates

Keep defaults unchanged and #126 open. The previously accepted bounded staggered populations remain B-link 24 and reference 28. This matched N24 cohort accepts three five-minute windows on each route and does not extend either population frontier. It supplies no B-link adoption advantage for the active-tenant/disk-efficiency objective.

Steady-state/debt bounds, growing distinct-key churn, reader retention, physical disk return, larger-than-budget/cold-store behavior, and complete host-memory accounting remain unaccepted. Decide explicitly whether the product requires B1/B2 eviction and byte-budget architecture; further small-store repeats or hot-path tuning are not a substitute for that structural decision.
