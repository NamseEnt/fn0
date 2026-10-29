# Decision boundary for the active-tenant investigation

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: assessment after the bounded capacity experiment and the stopped raw-observation confirmation. Shutdown attribution is a separate next diagnostic; unfinished cells are not accepted.

The evidence does not support adopting the current B-link implementation for greater active-tenant density or practical disk efficiency. Keep defaults unchanged. This is a decision about the measured implementation and policies, not a general B-tree versus LSM conclusion.

## What supports that decision

The completed `aaa405655` experiment found the same highest three-of-three E2 count, twenty-eight active tenants, for both routes. B-link used less instrumented worker memory in that small-data condition, but did not demonstrate a higher accepted tenant count. At E3, B-link passed five active tenants, failed setup at six through the unchanged shutdown deadline, and failed setup at eight through kernel OOM. The shared-resource contract reference passed eight active tenants three times. The reference was not screened above eight, so eight is a lower bound on its accepted count, not its maximum.

The raw-observation follow-up confirms an actual diagnostic cost: at one E2 tenant and 100 ms cadence, the final worker epoch peak fell from 34.00 to 8.25 MiB for the reference and from 13.95 to 6.12 MiB for B-link. Its new larger-population cohorts did not reconfirm the old E2 acceptance: B-link passed one of three twenty-eight-tenant cells and the reference passed none, despite complete accounting and both oracles. At E3 five tenants, B-link passed twice then exceeded the parent shutdown watchdog; the reference passed three times. Later screens and maintenance were omitted by the frozen stop rule. This supports neither a new B-link capacity advantage nor a source-causal latency claim.

Report both sources and dates. The current-host archived-binary cadence control addresses that narrow low-load comparison; it does not recreate every large-population baseline. [OBSERVATION-RETENTION-RESULTS.md](OBSERVATION-RETENTION-RESULTS.md) preserves all twenty new cells and their acceptance boundaries.

The subsequent b226 shutdown diagnostic stopped at its first five-tenant B-link setup, before offered load. Its trace locates an outstanding asynchronous server-shutdown interval under the same high/max policy, without identifying the internal cause. Three other diagnostic cells completed, including the five-tenant reference. [SHUTDOWN-DIAGNOSTIC-RESULTS.md](SHUTDOWN-DIAGNOSTIC-RESULTS.md) preserves this separate cohort; the further external wait observation is not a completed capacity result.

File allocation snapshots also remain relevant to the goal. They measure reachable files at named phase boundaries. They do not measure continuous peak space, unlinked-open files, physical space returned after deletion, or steady-state write amplification. Do not replace the disk-efficiency question with a throughput comparison or infer disk reclamation from checkpoint completion.

## Structure confirmed in source

Current B-link opens and retains the complete decoded tree. Its reader obtains resident page objects; it has no page-miss reconstruction and eviction path. The cache-capacity field does not bound this residency. Generation catalogs, dirty encoded images, snapshot references, request/preparation buffers, and allocator costs can coexist with the current pages. Normalized checkpoint image bytes are not a heap measurement.

The experimental cgroup is an external backend limit. It can force reclaim, throttle, or kill a worker; it does not make B-link evict pages, admit work by bytes, or close idle tenants. A1 remains disabled in these runtime binaries and does not add eviction even when enabled. Shared RocksDB cache/WBM settings are soft native controls inside the same hard backend limit, not a complete machine-memory policy.

These facts explain why larger-than-resident activity would require the B1/B2 architecture described in [PLAN.md](PLAN.md). They do not identify how much memory each current representation retains or prove the cause of a particular tail-latency observation.

## What remains before a stronger claim or another implementation

- Locate the stopped measurement worker using [SHUTDOWN-DIAGNOSTIC-PLAN.md](SHUTDOWN-DIAGNOSTIC-PLAN.md). Preserve the completed raw-observation cells and original stop; do not silently resume its omitted matrix or relax deadlines.
- Treat a non-reconfirmed E2 boundary as unresolved for the new cohort. A current-host large-population source control or matched profiling would be needed to attribute a difference to source; do not tune load, cadence, or SLO thresholds to obtain a pass.
- A larger-than-backend-budget dataset would still leave ARC and previously charged shared file pages outside this limit. A physical disk-miss or total-machine-density claim needs an explicit host-cache policy and measured reads. None follows from the current small read coverage.
- Longer maintenance, growing distinct-key churn, staggered checkpoint demand, reader retention, and physical disk-return work remain separate gates. Three-minute fixed-key maintenance is not steady state.
- If the product still requires many continuously active tenants above aggregate resident memory, evaluate B1/B2 as a separate storage/recovery architecture project. Preserve durable page reconstruction, version/pin reachability, byte accounting, fairness, and the real-service crash/write/crash recovery gate. Avoid further small hot-path changes as a substitute for that requirement.

Issue #126 remains open. No default promotion, production capacity promise, engine-only ranking, or B1/B2 implementation acceptance is implied.
