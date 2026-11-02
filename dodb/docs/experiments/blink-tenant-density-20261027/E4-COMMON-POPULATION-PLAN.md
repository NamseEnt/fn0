# E4 fixed-population maintenance comparison

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

## Question and scope

Continue the reviewed resident frontier with a matched twenty-four-active-tenant, five-minute fixed-key maintenance comparison. B-link has three accepted sixty-second N24 confirmations; the shared reference has three accepted staggered N28 controls. N24 is the common population selected before new results, not a newly measured reference capacity. This phase independently tests both routes at N24 and cannot change the earlier accepted frontier.

Run three pairs of 300,000 ms cells, ordered B-link/reference, reference/B-link, B-link/reference. Each fresh database has 24 registered and active tenants, zero idle tenants, 2,048 rows/tenant, 512-byte values, seed 20261027, staggered arrivals, and fifty calls/s/tenant. Preserve the mixed request/transaction-width recipe, synchronous durability, revision/condition semantics, caps eight, client count one, existing request/drain/shutdown/watchdog deadlines, 256/192 MiB cgroup max/high and zero backend swap. Reuse runtime cb0e0448270c1da3d9a214fb008ec01406d8e40d and its exact validated default binaries with A1 disabled. Keep the existing B-link checkpoint and shared RocksDB options unchanged.

Keep 5,000 ms backend/native pressure sampling. Prior N28 reference controls retained about 610 KiB in thirteen samples; approximately 61 samples at N24 should fit below 3 MiB, with margin under the unchanged 8 MiB/4,096-sample bounds. Truncation invalidates a cell. Record driver memory and host/ARC observations separately; this remains a backend-only enforced budget. Do not change host policy or attempt larger-than-memory, cold-cache, growing-key, reader-retention, physical disk-return, or engine architecture work.

## Entry, sizing, and preservation

Require the corrected frontier review and arrival controls to be evidence-valid, preserving ordinary SLO misses. Verify exact source/native/binary/helper hashes, clean frozen source, inherited soft/hard descriptor limits, exited prior identities, and empty prior cgroups. Freeze the new controller in a clean commit before launch; use namespace `maintenance-cb0e04482-r1`. Every evidence path must be new; preserve previous namespaces and all databases, including failures.

Live preflight on 2026-11-02 found 27,222,994,944 benchmark bytes free and 5,179,195,392 host MemAvailable bytes. These are sizing observations, not future guarantees. Require 12 GiB free at entry, reserve at most 8 GiB for six new databases/evidence, and retain the existing runtime 10 GiB free/1.5 GiB MemAvailable guards and 1,200-second controller cell cap. Record fresh capacity, ZFS properties, ARC, and competing processes in provenance. Do not delete artifacts to make this phase fit.

## Acceptance and stops

Operational completion, experimental SLO pass, and maintenance coverage are independent. Each complete cell must have exactly 360,000 scheduled successful requests, no drops/errors/conflicts/timeouts/end backlog, full-population and reopen oracles, verified identities, complete observations, and clean shutdown. Review exact commands including report/database/cgroup paths. Match request and schedule fingerprints within every pair. Acceptance requires all three cells on each route to pass the unchanged per-tenant histogram SLO reducer and both oracles. Do not select a passing median.

An ordinary SLO miss remains evidence-valid and does not stop the fixed six-cell matrix. Any operational failure, OOM/watchdog, oracle/provenance/identity mismatch, observation truncation, or resource guard failure stops the entire cohort immediately. Record omitted cases and collect the stopped cohort without replay. No fallback or tuning is permitted.

Report B-link measurement-worker checkpoint counters and periodic counter changes, including failures, overlap, and at least two completed cycles for maintenance coverage. Their scope includes that worker's preflight/drain/shutdown; they are not exact offer-boundary deltas. Report available reference flush/compaction/native snapshots without inventing missing cycle counters. Preserve first/last sampled backend memory, observed sample gaps, offer-end backlog, file allocation phase snapshots, and worker write-I/O scope. Sampled trends and phase allocation are not continuous peaks, physical disk-return, allocator attribution, or steady-state proof. Five-minute repetitions remain finite-window maintenance evidence.

Keep defaults unchanged and #126 open. After review, update the decision and handoff with the actual outcome and remaining structural gates; do not substitute further hot-path tuning for the B1/B2 architecture decision.
