# Frozen E2 arrival-phase controls

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

The design authority is [E2-ARRIVAL-PHASE-PLAN.md](E2-ARRIVAL-PHASE-PLAN.md). This matrix freezes only correctness smokes and current-source matched controls. Resident-population screens and maintenance require a separate frozen matrix after these controls and fresh sizing; they are not implicitly authorized by a passing control.

## Provenance and common policy

Runtime source is `cb0e0448270c1da3d9a214fb008ec01406d8e40d`, with opt-in staggered arrivals and synchronized default. Use namespace `arrival-controls-cb0e04482-r1`. Record the full controller/plan commit separately from the frozen runtime source. Require clean-source native validation before execution: the affected library, worker, driver, read, and transaction checks; advisory Clippy with the same seventeen existing diagnostics; A1 compatibility link; and default link last. Preserve the exact default binary hashes and copy those validated binaries into this namespace.

All cells retain the original 256/192 MiB backend cgroup max/high, zero backend swap, fifty calls/s per active tenant, one client, global in-flight cap eight, per-tenant queue eight, five-second request timeout, fifteen-second drain, fifteen-second internal asynchronous shutdown limit, twenty-second parent stop watchdog, and 1,200-second per-cell controller limit. Retain seed `20261027`, 2,048 initial rows per registered tenant, 512-byte values, mixed 40/20/40 Get/Query/Write, mixed 80/15/5 transaction widths 1/4/16, conditional writes, logical revisions, synchronous WAL durability, and both full-population oracles. The native shared cache/WBM remains 64/32 MiB, per-database write buffer 1 MiB, flush/compaction threads 1/1, and max-open-files 64. B-link retains segmented coalesced bases, 256 KiB segments, 1 MiB WAL checkpoint threshold, background execution, and A1 disabled.

Use a common 5,000 ms pressure sampling interval for every cell in this cohort. Retain final properties and existing 8 MiB / 4,096 observation bounds; any truncation invalidates a completed cell. This cadence differs from older cohorts, so historical differences cannot be attributed to arrival phase.

Set only the controller process's soft `RLIMIT_NOFILE` to 65,536, preserving its hard limit, and verify the same limits in an owned child before creating workloads. Record the original, new, and inherited limits. Both backend processes inherit this common policy. Do not change host-wide limits, host swap/ARC settings, or native per-database settings. The previously observed soft/hard limits were 1,024 / 524,288; fresh observations take precedence.

Before each cell require at least 12 GiB free on the benchmark dataset and 1.5 GiB host MemAvailable. During execution retain the 10 GiB free-space and 1.5 GiB MemAvailable guards. Initial sizing before the native build found 28,697,985,024 benchmark bytes and 5,453,029,376 boot bytes free, with 4,901,860 KiB MemAvailable. Recheck after the build. Existing twenty-eight-tenant sixty-second E2 reports retained about 55 MiB native-reference and 98 MiB B-link reachable allocated file bytes after reopening; these are sizing observations, not peak physical dataset measurements. Sixteen new cells plus binaries, reports, and build outputs fit the current reserve conservatively, while live guards remain authoritative.

## Exact order

Each row is a backend pair. Odd-numbered pairs run B-link first; even-numbered pairs run shared RocksDB first. All databases are new and preserved.

| Pair | Case | Mode | Registered / active / opened idle | Offer duration |
| --- | --- | --- | --- | --- |
| 1 | tiny-synchronized | synchronized | 4 / 3 / 1 | 10 s |
| 2 | tiny-staggered | staggered | 4 / 3 / 1 | 10 s |
| 3 | control-n28-r1-synchronized | synchronized | 28 / 28 / 0 | 60 s |
| 4 | control-n28-r1-staggered | staggered | 28 / 28 / 0 | 60 s |
| 5 | control-n28-r2-staggered | staggered | 28 / 28 / 0 | 60 s |
| 6 | control-n28-r2-synchronized | synchronized | 28 / 28 / 0 | 60 s |
| 7 | control-n28-r3-synchronized | synchronized | 28 / 28 / 0 | 60 s |
| 8 | control-n28-r3-staggered | staggered | 28 / 28 / 0 | 60 s |

This is sixteen actual attempts at most: four tiny cells and twelve matched-control cells. Tiny cells schedule 500 requests per active tenant, 1,500 per cell. Controls schedule 3,000 per active tenant, 84,000 per cell. A complete sequence offers 1,014,000 requests and performs thirty-two full-population oracles.

## Acceptance and stops

Both tiny modes must pass source/identity, phase-offset, complete accounting, request recipe, full oracles, observation bounds, and unchanged per-tenant SLO checks before controls. For the controls, preserve ordinary SLO misses and continue the frozen sequence; acceptance requires all tenants in all three fresh sixty-second repetitions for the same backend and mode. Retain the existing completion-rate and Get/Query/Write p99 bucket thresholds, not new thresholds chosen from results.

Within a mode, compare deterministic and realized backend recipes with the existing immutable comparator. Across modes, require equal input/request recipes and intended per-tenant counts, validate the staggered offset formula, and expect distinct schedule identities. Preserve actual request/schedule fingerprint differences caused by admission drops; do not fabricate equality or use the same-mode comparator across modes.

Any operational error, watchdog, OOM, oracle failure, identity/provenance mismatch, observation truncation, or host guard failure stops the whole cohort immediately. Preserve partial workload counters as available but unvalidated after a later failure; never promote them to an accepted cell, invent zero throughput, retry the failed cell, or skip to later controls. Keep original reports, logs, databases, all child identities, cgroup limits/events, failed launchers, validated binaries, and native artifacts. A later correction to collection or reduction reuses these attempts instead of rerunning them.

This cohort tests a controlled arrival-pattern effect at twenty-eight active small-store tenants. It establishes neither an engine maximum nor B-link adoption superiority, and it does not resolve the E3 pre-offer shutdown failure, host ARC accounting, larger-than-budget behavior, cold reads, growing churn, long maintenance, physical disk return, or architecture gates. Issue #126 remains open.
