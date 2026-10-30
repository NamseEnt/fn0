# Bounded staggered resident-population results

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

The completed bounded search accepts twenty-four active tenants for B-link from three fresh sixty-second confirmations and retains shared RocksDB's previously accepted twenty-eight-tenant staggered control baseline. It establishes no B-link density advantage. These are tested accepted populations under this service policy, not engine maxima.

## Results and boundaries

| Route | New thirty-second screens | Fresh sixty-second confirmation | Accepted population |
| --- | --- | --- | ---: |
| B-link | 16 pass; 24 pass | 24 tenants, 3/3 pass | 24 |
| Shared RocksDB | 32 SLO miss | None; no passing new screen | 28, preserved controls |

B-link's prior staggered twenty-eight-tenant controls passed two of three; it therefore followed the predeclared 16/24 route rather than the wider grid. Shared RocksDB's prior controls passed three of three, so it screened 32. That cell completed correctly but missed per-tenant latency objectives; the route's higher 48/64/80/96 screens were omitted. No fallback condition was eligible or executed. Different accepted populations are not matched throughput pairs.

The six new cells completed 324,000 scheduled successful requests and twelve passing full-population/reopen oracles. Five cells met every SLO; the 32-tenant reference screen was the single ordinary SLO miss. Every cell had zero cgroup high/max/OOM events. That screen's cgroup peak was 83,136,512 bytes; this is not evidence of a memory.high capacity limit. The worst-tenant Get and Query p99 histogram upper-exclusive edges were 16.777216 ms there. Histogram buckets are not exact percentile timings.

B-link's three confirmations had whole-cell cgroup peaks of 62,922,752 / 60,502,016 / 60,817,408 bytes. The worst-tenant p99 upper-exclusive edges were Get 8.388608 / 4.194304 / 8.388608 ms, Query 4.194304 ms in each repetition, and Write 16.777216 ms in each. Smaller observed backend memory did not establish a higher accepted active population. Exact representation/allocator attribution and total host-memory density remain unmeasured.

## Provenance, corrections, and preservation

Runtime source and default binaries remain `cb0e0448270c1da3d9a214fb008ec01406d8e40d`, with A1 disabled. Native/build validation is reused by exact hashes from the arrival controls; no rebuild or accepted-cell replay occurred. Workload, 256/192 MiB backend cgroup policy, zero backend swap, fifty calls/s per tenant, caps eight, 5,000 ms sampling, descriptor limits, and timeouts follow the frozen plan.

The original `5cc6ac930` launch failed its helper lookup before creating a runtime namespace or workload. Its launcher, source snapshot, and traceback remain preserved. A corrected controller was frozen at `714320187d78aaf47d94a7579b77b3320e3b842f` and executed new namespace `resident-frontier-cb0e04482-r2`. The original r1 launch is not one of the six workload executions.

The first post-run reviewer rejected commands because its expected option map omitted the recorded `--report` path. That original `final-review.json` is retained inside the raw runtime tree. Reviewer commit `2c6c79ecc` adds exact report-path validation and produced the valid `review-r2.json` on OCI. Only reporting changed; no workload was rerun. The top-level [final-review.json](evidence/resident-frontier/final-review.json) is an exact copy of that corrected review. Both reviewer versions and their provenance remain archived.

[Collection receipt](evidence/resident-frontier/collection.json) verifies all six preserved databases, thirty unique exited workload identities, and 31 empty cgroups. Collection retained 27,222,994,944 benchmark bytes available and 4,974,395,392 bytes host MemAvailable. The 14,194,977-byte `runtime.tar.gz` has SHA256 `d8a7a932af7903e0c922a9c263bca5a9a0d14bfcb5aece93f7d2b2273c3a633c`. Its 59 regular files include source snapshots, frozen provenance, binaries, failed initialization/review records, completed reports, commands, logs, and final decisions. All archive and extracted-file hashes matched. Databases remain on OCI in the original namespace.

[Local verification](evidence/resident-frontier/collection-verification.json) additionally recomputes all six recipes, reductions, oracle passes, and final route decisions from the collected reports. All fifteen controller/reviewer tests pass against the committed evidence:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s dodb/docs/experiments/blink-tenant-density-20261027/evidence/resident-frontier
```

For an onsite repeat of the final review, use the preserved reviewer under `/home/opc/tenant-density-harness-20261027/resident-frontier/2c6c79ecc/` with the frozen capacity reviewer and the r2 run directory, supplying a new output path. It requires the original preserved database and cgroup paths; local extraction alone does not reproduce those host-state checks. The controller does not perform collection itself. Extracted binaries are ignored in Git but remain inside the committed archive and locally preserved extraction.

## Decision

Keep defaults unchanged and issue #126 open. The bounded small-store staggered comparison currently accepts B-link 24 versus reference 28 under backend-only cgroup accounting. B-link was not screened above 24 in this new cohort; the existing 28 result is 2/3 rather than an accepted boundary. Shared RocksDB's 32 result is a thirty-second SLO miss, not a proven memory or engine maximum.

Longer maintenance requires a separately frozen common-population design and fresh resource sizing. Larger-than-budget/cold-store activity, growing churn, retained representation bytes, physical disk return, and B1/B2 eviction/recovery architecture remain separate gates. Driver memory and uncharged host ARC remain outside the enforced backend budget.
