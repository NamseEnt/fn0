# E2 arrival-phase comparison results

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

The sixteen-cell current-source cohort completed correctly. B-link has no consistent active-tenant density advantage: at twenty-eight active tenants its synchronized controls passed three of three SLO repetitions, while staggered controls passed two of three. Shared RocksDB passed one of three synchronized repetitions and all three staggered repetitions. Keep defaults unchanged.

## Provenance and validation

Runtime source is `cb0e0448270c1da3d9a214fb008ec01406d8e40d`, with A1 disabled. Controller commit is `95c3b8c5323415dfc493bdfcb32c7dc5e057e8b8`; the frozen controls plan is `e02a1fbdde81fd6fc363a26e5341c8c61000023e`. Runtime namespaces are `arrival-cb0e04482-r1` (build validation) and `arrival-controls-cb0e04482-r1` (controls).

The preserved native validation passed 85 checks across five groups, A1 and default links with default last, and advisory Clippy with the same seventeen pre-existing diagnostics. Final collection verifies source, native archive, binary hashes, helper provenance, archive/extracted manifests, descriptor inheritance, and the original build logs. No accepted workload cell was rerun for reporting.

The final local review and its tests are reproducible from the worktree root:

```sh
python3 -m unittest discover -s dodb/docs/experiments/blink-tenant-density-20261027/evidence/arrival-phase -p 'test_*.py'
python3 dodb/docs/experiments/blink-tenant-density-20261027/evidence/arrival-phase/review_arrival.py
```

All 26 Python checks passed. The review reproduced sixteen operationally valid cells and thirteen per-cell SLO passes. Its `accepted_run: false` means that all sixteen cells did not pass SLO; `completed_run_evidence_valid: true` independently records the valid completed cohort. This ordinary SLO distinction does not block the next plan's operational entry gate.

## Observed controls

| Arrival phase | B-link SLO passes | Shared RocksDB SLO passes |
| --- | ---: | ---: |
| Synchronized | 3/3 | 1/3 |
| Staggered | 2/3 | 3/3 |

Four tiny gates and twelve sixty-second twenty-eight-tenant controls completed 1,014,000 successful scheduled requests: 6,000 tiny-gate requests and 1,008,000 control requests. All 32 full-population/reopen oracles passed. All sixteen databases remain preserved, all eighty recorded workload process identities exited, and all 81 collected cgroup process lists were empty.

Same-mode backend comparisons matched schedules and inputs. Cross-mode comparisons matched request recipes and counts while verifying the expected distinct schedule identities. The global in-flight and queue caps remained eight, per-tenant offered rate fifty calls/s, memory maximum/high 256/192 MiB with zero backend swap, and pressure sampling 5,000 ms. Both routes inherited soft/hard descriptor limits 65,536/524,288; a separate owned preflight child verified inheritance.

Maximum measurement cgroup peaks across repetitions were 69,779,456 bytes synchronized and 70,483,968 staggered for B-link; 85,962,752 synchronized and 86,474,752 staggered for shared RocksDB. These are instrumented backend measurements, not exact retained heap or total host-memory costs. Native histories retained 39 measurement samples per backend/mode condition, totaling approximately 47 KiB for B-link and 1.83 MB for shared RocksDB. The observation bounds remain material when sizing wider populations.

## Evidence and continuation

[Final review](evidence/arrival-phase/final-review.json), [collection receipt](evidence/arrival-phase/collection/collection.json), the raw `runtime.tar.gz`, extracted logs/reports, and validation records are preserved under `evidence/arrival-phase/`. Extracted runtime binaries are ignored in Git because identical binaries remain inside the committed raw archive and on OCI; they remain available locally for full review.

The results show arrival-pattern sensitivity under this fixed workload and service policy. A synchronized SLO miss alone cannot identify a resident-memory boundary. Driver memory and uncharged host ARC remain outside the backend cgroup. The separate eighteen shutdown snapshots identify a sampled memory.high wait contribution without an exact user-code attribution.

[E2-RESIDENT-FRONTIER-PLAN.md](E2-RESIDENT-FRONTIER-PLAN.md) defines the next bounded staggered search: shared RocksDB may reuse its accepted twenty-eight-tenant baseline and screen 32/48/64/80/96; B-link has no three-of-three staggered baseline and starts at 16/24. Thirty-second screens do not establish acceptance. Confirmation requires three fresh sixty-second passes, with the frozen fallback and global operational-stop rules. Longer maintenance and larger-than-budget architecture remain separate work. Issue #126 stays open.
