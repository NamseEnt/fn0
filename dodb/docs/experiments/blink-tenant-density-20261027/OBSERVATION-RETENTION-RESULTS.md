# Raw observation retention and streamed health reports

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: implementation and clean-source checks passed. The frozen runtime comparison stopped for review on a measurement-worker shutdown failure. The completed current-host cadence control confirms lower instrumented worker memory; new capacity acceptance is incomplete.

## Implementation

Source `023b18bd8921b593f243f59b9eaa1576125c6572` changes the diagnostic worker's opt-in periodic pressure storage from retained `serde_json::Value` objects to validated raw JSON frames. Each accepted frame is serialized once and retained under the existing 4,096-frame / 8 MiB serialized-byte bounds. Frame/vector/allocator overhead and temporary sample construction remain outside that byte count; it is not an exact heap ceiling.

The final worker health report streams metadata and raw frames through a buffered writer. It does not reconstruct all frame DOM objects or allocate one complete serialized worker report buffer. The parsed JSON shape, timestamps, phases, sample order, truncation, unsupported-property states, shared-resource counting, and complete final checkpoint details are preserved. Source JSON parsing remains in the driver outside the backend cgroup; total-machine cost is not removed by moving that parsing.

Only the worker, its new helper/tests, and the pinned serde_json raw-value feature changed. Engine code, defaults, caches/WBM, checkpoint policy, locks, WAL/durability, transactions, revisions, admission, budgets, and deadlines remain unchanged. A1 stays disabled in accepted runtime binaries.

## Clean-source checks and disk preparation

All 74 checks passed: the previous 68 plus six worker tests covering empty/multiple-frame schema, nested/escaped/unavailable values, count and byte boundaries, metadata misuse, and output-error propagation. Both default/A1 feature links passed. Advisory Clippy retained the same seventeen message/path findings without a new finding or suppression.

The first new feature build left only 1,123,028,992 boot bytes free. The complete approximately 4.3 GiB target cache was copied to `/bench/zfs/db/tenant-density-build-cache-20261029/target`, checksum-compared with an empty dry-run result, and activated through the unchanged logical Cargo target path. Runtime/native hashes matched before and after. Only the verified duplicate on boot was removed. The active cache, every database, frozen binary, source checkout, native archive, log, and failed artifact remain preserved. See [OBSERVATION-CACHE-RELOCATION.md](OBSERVATION-CACHE-RELOCATION.md).

After relocation, boot/benchmark free space was 5,712,830,464 / 32,584,036,352 bytes before further feature links. Build-cache occupancy is an explicit host-provenance difference; preserve it in timing comparisons.

## Executed protocol and preserved stop

[OBSERVATION-RETENTION-MATRIX.md](OBSERVATION-RETENTION-MATRIX.md) freezes the comparison. Runtime source and binary identities are separate from its controller/plan commit. The current-host control uses the archived default-feature `aaa405655` binaries without rebuilding; its namespace is `observations-control-aaa405655-r1`. All four cadence cells completed and met the objective. Its source checkout was selected only for provenance and restored to `023b18bd8` afterward.

Candidate namespace `observations-023b18bd8-r1` executed sixteen cells: fifteen operational completions and one measurement-worker shutdown failure. Its `controller-failure.json` is preserved and no `completion.json` exists. The frozen stop rule worked. The subsequent six/eight-tenant screens, eligible eight-tenant confirmations, and maintenance pair were not executed. Do not resume or relabel this namespace as complete.

Together with the four archived-binary control cells, the twenty unique new executions contain nineteen completions, fourteen SLO passes, five completed SLO misses, and one operational failure. Complete reports account for 603,000 successful offered API calls and thirty-eight passing full-population oracles. The failed cell contributes no invented request count, throughput, or oracle result. Kernel OOM occurred in none of these twenty cells.

## Current-host cadence control

Every cell below is one tenant, E2, sixty seconds, and the same 256/192 MiB policy. Each source completed both oracles and met the objective at both cadences. Peaks include the diagnostic worker and cgroup-accounted resources. The offer-boundary peak includes pre-offer opening in the measurement epoch; the final epoch peak also includes drain and shutdown/reporting.

| Backend / cadence | Archived-source peak through offer / final epoch, MiB | Raw-frame source peak through offer / final epoch, MiB |
| --- | --- | --- |
| B-link / 1,000 ms | 5.293 / 5.773 | 5.176 / 5.188 |
| Shared RocksDB / 1,000 ms | 6.332 / 7.812 | 5.766 / 5.766 |
| B-link / 100 ms | 8.469 / 13.949 | 5.875 / 6.117 |
| Shared RocksDB / 100 ms | 16.273 / 34.000 | 7.016 / 8.246 |

At 100 ms the final epoch peak fell 56.1% for B-link and 75.7% for the reference in these single current-host observations. Both retained 601 measurement-worker native frames, with matching parsed frame/report keys and exact serialized-byte recounts. This supports removing a material observation-retention/reporting cost. It is not isolated allocator attribution or a repeated estimate of a general engine-memory gain.

Driver memory did not disappear: sampled measurement-driver RSS at 100 ms was 36.49 MiB for B-link and 36.38 MiB for the reference with raw frames, versus 36.29/36.15 MiB in the control. Backend offered-window CPU was approximately 1.060/1.223 seconds with raw frames versus 1.043/1.202 seconds in the control. These one-off CPU observations do not show a CPU improvement. Driver parsing and its DOM remain outside the backend budget.

## New large-population confirmations

| Condition | B-link | Shared RocksDB |
| --- | --- | --- |
| E2, 28 active, three sixty-second cells | Three operational completions; one SLO pass, two Get-tail misses | Three operational completions; all three Get-tail misses |
| E3, 5 active, three sixty-second cells | First two completed and passed; third failed during measurement-worker shutdown | All three completed and passed |

E2 median peak through the offer / final measurement epoch was 68.797/68.797 MiB for B-link and 83.953/85.918 MiB for the reference. All six E2 cells completed the same successful request counts and both oracles. Missed Get histograms had a worst-tenant p99 bucket with upper bound 16.777216 ms, above the unchanged 8.388608 ms objective. No threshold, arrival pattern, or admission change was made to obtain a pass.

The earlier source's twenty-eight-tenant three-of-three result remains accepted for its recorded cohort. This new cohort did not reconfirm that acceptance. Engine settings and realized input match, but source/date/cache occupancy are distinct; there is no current-host archived-source large-population control. Do not attribute this difference to JSON storage or claim a latency regression mechanism without that control or matching profiles.

At E3 five tenants, the two completed B-link epoch peaks were 184.770 and 186.375 MiB. The reference's three-cell median was 37.199 MiB. Do not turn the two B-link successes into a three-of-three confirmation by omitting the failed repetition. After-measurement reachable file allocation was about 231.07 MiB for the median of those two B-link completions and 93.78 MiB for the three reference completions. These phase snapshots are not continuous disk peaks, write amplification, or physical space-return evidence.

## Measurement-worker watchdog failure

`confirm-e3-n5-r3-blink` reached offer start and after-drain, then the parent killed the measurement worker after its unchanged twenty-second stop watchdog. The worker's separate fifteen-second asynchronous server-shutdown timeout is also unchanged; its specific error was not emitted. This is distinct from the old source's six-tenant setup-worker fifteen-second failure and eight-tenant setup OOM.

The recorded after-drain to after-worker-exit interval was 20,039 ms. Parent/measurement peaks were 231,751,680 / 231,706,624 bytes, with 8,264 parent high events and 2,797 measurement-local high events. Max, OOM, and kill events were zero; all final groups were empty. High events establish pressure, not the cause or exact stalled shutdown operation.

Only the setup health report survived. The driver had not yet transferred its local workload result into the outer report when shutdown failed, so `partial_workload_counters` explicitly says unavailable. Neither a complete measurement-native trace nor final latency/success/oracle results are available. The source path can stall in command processing, server shutdown, final metric capture, backend destruction, report writing, or runtime destruction; current evidence does not distinguish these stages. [SHUTDOWN-DIAGNOSTIC-PLAN.md](SHUTDOWN-DIAGNOSTIC-PLAN.md) defines the next instrumentation variant and preserves both deadlines.

## Reproduction and preservation

Run `PYTHONDONTWRITEBYTECODE=1 python3 dodb/docs/experiments/blink-tenant-density-20261027/evidence/observation-retention/review_observations.py` from the worktree. The local reviewer preserves the runtime's original review error and classifies only this exact stopped measurement as an operational failure; legacy reducers and the old archives are unchanged. Completed source comparisons verify effective configuration, deterministic/realized requests, worker report keys, source/script/binary identities, native sharing, and observation bounds. Failed results remain separate.

Raw archive `evidence/observation-retention/runtime.tar.gz` is 2,110,210 bytes with SHA256 `724500af7b15fa2f464ea08580fee7be684b870ce959fad6470d672c18dd99ea`. Extraction, transfer receipt, postflight, and reduced data are preserved beside it. Collection verified twenty distinct databases retained on OCI, ninety-eight exited process identities, empty groups, clean source `023b18bd8`, and the unchanged pinned native archive. Minimum recorded benchmark free space was 27.73 GiB and host MemAvailable was 4.68 GiB. The active relocated build cache remains preserved.

Validation and relocation evidence is in `evidence/observation-retention/validation/` with `validation-review.json`. Keep the earlier [capacity result](DENSITY-CAPACITY-RESULTS.md) and both original archives intact. [DENSITY-DECISION.md](DENSITY-DECISION.md) records the current adoption boundary. Issue #126 remains open.
