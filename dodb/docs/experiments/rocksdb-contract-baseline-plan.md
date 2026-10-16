# RocksDB Contract Baseline and Optimization Plan

> Historical checkpoint (2026-10-16 UTC): Execution stopped after read-only preflight because OCI ZFS capacity was insufficient. At that checkpoint, no implementation or benchmarks had started.

> Current status (2026-10-16 UTC): The user chose to compare both engines first with a dataset that fits in memory; memory-exceed experiments are Deferred by user, not complete. User-authorized OCI cleanup continues independently. Phase 1 contract-audit documentation is complete; correctness tests have not been rerun, so Phase 1 remains In progress. Phases 2–6 remain Pending. Final disk capacity is unverified until cleanup completes. No remote builds or benchmarks until cleanup completes and a fresh capacity preflight passes; then run them serially.

> The earlier read-only API audit was preliminary. The completed documentation audit is recorded in [CONTRACT-AUDIT.md](results/rocksdb-contract-baseline-20261016/CONTRACT-AUDIT.md); it does not establish that correctness validation has passed.

## Goal and claim boundary

Establish a fast RocksDB implementation that satisfies the DODB API and correctness contract, then compare it fairly with the B-link service. The raw unconditional subset is a reference point for measuring required API and wrapper costs. The winner is unknown in advance. Results must not claim an absolute RocksDB maximum: the full-semantics synchronous core may still contain DODB-required work, and the service paths retain their own integration costs.

The comparison must separate raw subset throughput, successful transactions, successful mutations, preparatory reads, and end-to-end service calls. A result is workload-specific and configuration-specific. No semantics or durability guarantees may be weakened to improve a score.

## Execution rules

- Preserve source hashes, raw logs, reports, reproduction instructions, input seeds, validation output, host and filesystem details, and rejected-candidate evidence. Reuse existing verified evidence only when its source, workload, correctness, and provenance match this plan; record its artifact path and why it qualifies.
- The user authorized deletion of completed experiment databases, old builds and binaries, and unnecessary archives. Preserve the current environment, source, and current required data. Record every removed historical database or binary in a cleanup manifest. Retain raw logs, reports, reproduction instructions, source hashes, and validation evidence; retaining every validated database or archive is not required.
- Change one measured cause at a time. Keep rejected candidates' raw evidence and manifests.
- Keep the default `Serialized` revision allocator mode and shared concurrency setting of 8 unless a separately justified change is explicitly approved. Benchmark optimized settings explicitly without changing defaults by implication.
- Root owns design and candidate-selection decisions. Execution follows the selected scope and reports blockers immediately.
- Do not deploy to production.

## Phase 1: Contract and architecture audit

**Status: In progress**

Contract-audit documentation is complete in [CONTRACT-AUDIT.md](results/rocksdb-contract-baseline-20261016/CONTRACT-AUDIT.md), based on architecture, backend-contract, and revision-range documentation at commit `543b55c9`. Phase 1 remains In progress because correctness tests have not been rerun; audit documentation completion is not correctness validation.

Map the DODB API, consumers, tests, RocksDB wrapper, service path, B-link path, and relevant concurrency and durability boundaries. Resolve the contract of revision values versus physical commit LSNs before changing identity or revision behavior. Audit every `commit_lsn` consumer before considering an opaque logical write identity. If current consumers rely on physical ordering or meaning, document that promise and obtain an explicit contract decision before weakening it. Independent concurrent response order is not a linearization order.

The audit must determine whether the implementation preserves all of the following:

- `RevisionEquals`, `Exists`, and `NotExists` conditions.
- Atomic writes across multiple keys, including condition keys that are not modified.
- Duplicate conditions and duplicate mutations are rejected according to the existing contract.
- Successful condition-only transactions do not allocate a revision.
- Version changes and increasing revisions for successive committed transactions that modify the same key.
- Missing-key revisions and delete/recreate ABA behavior.
- No revision reuse after restart.
- One transaction revision shared by every key modified in that transaction.
- Visibility of committed writes and read-after-success behavior.
- A consistent snapshot for each `Query` and `Scan`, including ordering, cursor behavior, and limits.
- Tenant isolation.
- Response-budget enforcement before large result materialization.
- Synchronized WAL for successful writes.
- Cancellation and shutdown lifetime safety.

A per-document revision conversion is not mandatory. Any alternative must be judged against the audited contract, not assumed to be required or free.

**Required evidence:** API and consumer map; contract table with implementation and test references; all `commit_lsn` call sites and their assumptions; open questions and explicit decisions; baseline correctness gaps; existing evidence accepted for reuse. The documentation audit is recorded in [CONTRACT-AUDIT.md](results/rocksdb-contract-baseline-20261016/CONTRACT-AUDIT.md); correctness tests and validation remain pending.

## Phase 2: Freeze the four comparison paths

**Status: Pending**

Define and freeze four paths with a shared external workload, resource limits, input data, and measurement method while retaining each path's necessary internal engine differences:

1. Raw RocksDB unconditional subset reference.
2. Full-semantics synchronous RocksDB core. Reuse the current research baseline where appropriate, but optimize the core too. Never label it fully unwrapped or maximal RocksDB.
3. Actual RocksDB service.
4. Final B-link service.

Record which contract operations each path includes, the exact RocksDB and B-link settings, synchronization and WAL behavior, concurrency controls, and any work excluded by the raw subset. Keep calls/s, successful transactions/s, successful mutations/s, preparatory reads, natural conflicts, intended conflicts, and unexpected errors distinct. Report service end-to-end latency separately from engine and stage timing. Do not add stage percentiles as if they were additive.

**Required evidence:** path definitions; identical external input and resource-budget specification; source revision and configuration per path; metric definitions; validation and error-count definitions; rationale for any path-specific internal behavior.

## Phase 3: Correctness and workload matrix

**Status: Pending**

Reuse existing verified evidence where it matches the frozen contract and provenance. Add deterministic, identical-input coverage for any missing cases. Keep new-key and overwrite workloads separate. Include:

- Write mixes: 95/5, 80/20, 50/50, and 20/80 reads/writes.
- Multiple tenants.
- Conditional transactions that protect read-only keys.
- Transaction widths 1, 4, and 8, plus width 16 where historical comparisons make it relevant.
- Delete/recreate behavior and missing revisions.
- Representative `Query` and `Scan` cases with distributed primary keys and cursor boundaries.

Run normal performance measurements with diagnostic instrumentation OFF. Use a separate diagnostic run with stage timing for admission, dispatch, locks, reads, conditions, allocation, mutation, synchronization, and response work. Preserve the distinction between natural workload conflicts and intentionally generated conflicts; unexpected errors invalidate a run.

**Required evidence:** deterministic input generator/version and seed; exact workload distributions; replayable correctness artifacts; test results for every contract item; uninstrumented performance results; separate diagnostic results where needed; explicit identification of reused evidence.

## Phase 4: Optimize measured costs

**Status: Pending**

Prioritize candidates only when attribution shows that they address a material cost. Candidate areas include:

- `ReservedRanges` to remove or reduce revision-counter cost.
- Admission and client concurrency tuning.
- Encoding, allocation, copying, and FFI overhead.
- Bounded RocksDB configuration candidates when measurements justify them.

Each candidate must preserve transaction semantics, tenant isolation, response-budget behavior, cancellation and shutdown safety, and synchronized WAL guarantees. Evaluate CPU, resident memory, tail latency, fairness, and sustained engine health alongside throughput. Preserve failed and rejected candidates with their evidence. Compare configurations using workload-specific throughput, latency, CPU, resident memory, fairness, and engine-health tradeoffs. Do not adopt a candidate based on one arbitrary throughput threshold or claim one universal best configuration.

**Required evidence:** measured bottleneck and attribution; one-change candidate definition; correctness and durability results; throughput and tail latency; CPU and RSS; fairness; RocksDB flush, compaction, backlog, and disk behavior when relevant; accepted or rejected decision with rationale; source diff and provenance.

## Phase 5: Authoritative OCI measurements

**Status: Pending**

**OCI capacity/preflight gate: Not passed. User-authorized cleanup continues independently. No remote build or benchmark may start until cleanup is complete and a fresh capacity preflight passes; after that, builds and benchmarks must run serially.**

The read-only preflight transcript is preserved at [PREFLIGHT.txt](results/rocksdb-contract-baseline-20261016/PREFLIGHT.txt). It identifies the authorized instance `instance-20260923-1013` as ARM64 Neoverse-N1 with 2 CPUs and approximately 12 GiB nominal RAM. `/proc/meminfo` reports `MemTotal` 11,159,744 kB (about 10.64 GiB) and `MemAvailable` 4,786,040 kB (about 4.6 GiB); swap use is 360 MiB. The stored SSH identity and existing `known_hosts` were verified. The default `rustc` and `cargo` are 1.98.1; Rust 1.97.1 is installed. No matching active build or benchmark process was found. The exact RocksDB 11.8.1 archive was not confirmed: the follow-up lookup was incomplete and no conclusion about its availability is drawn. `/bench/zfs/db` reports 149 GB total, 148 GB used, 1.3 GB available (displayed as 100% used); ZFS available space is 1.21 GB, and the `dodbbench` pool is ONLINE at 96%. This transcript records read-only preflight evidence only, not a measurement result. Do not make final measurement claims from it.

The initial preflight's 1.21 GB available is from before the authorized cleanup; current free space is unverified until cleanup completes and the capacity preflight is repeated. That initial reading does not establish capacity for over-cache datasets, three 30-second confirmations, three five-minute repetitions, builds, temporary databases, WAL, and evidence. Do not start new benchmarks until cleanup is complete and a fresh capacity preflight passes. Use only the authorized OCI A1 ARM64 host and ZFS path `/bench/zfs/db`, with the approved stored SSH identity and normal host-key verification. Use pinned RocksDB 11.8.1 and Rust 1.97.1 with `+crc`. Do not substitute local or Debian measurements for authoritative results.

### Resident-memory feasibility gate

The current B-link implementation keeps all pages in RAM. Its `cache_capacity` bounds only the B-tree cache and does not evict pages. The [RESIDENCY-AUDIT.md](results/rocksdb-contract-baseline-20261016/RESIDENCY-AUDIT.md), recorded at commit `0358eb4e`, finds that same-budget paired throughput with an over-memory dataset is unsupported under this behavior. No paging or eviction implementation is included in this plan.

The user chose to compare both engines first with a dataset that fits in memory. The memory-exceed experiment axis is Deferred by user and is not complete. Preserve the full six-phase scope, resident comparison on both engines, all planned workload axes, and the final 30-second three-repetition and five-minute three-repetition requirements. Any future memory-exceed comparison must document the B-link residency limitation. User-authorized storage cleanup continues separately; final disk capacity remains unverified until cleanup completes and a fresh preflight passes. No remote builds or benchmarks may start before that gate, and then they must run serially.

Before testing, inspect live processes, host resources, and available storage. Do not run concurrent builds or benchmarks. The user authorized cleanup of identified completed experiment databases, old builds and binaries, and unnecessary archives; apply that authorization only to listed targets and record removed databases and binaries in a cleanup manifest. Preserve unknown or unrelated data, the current environment and source, current required data, raw logs, reports, reproduction instructions, source hashes, and validation evidence. Build from the exact source SHA and retain binaries only as long as required for active validation; record their hashes and any later removal in the manifest. Record commands, host and filesystem, seeds, workload, and validators.

Use existing five-second runs only for screening, together with missing candidate screening. Confirm candidates with a 10-second warmup and 30-second measurement for at least three repetitions in rotated serial order. Final representative measurements must run for at least five minutes with at least three repetitions. Before long runs, inspect every call-count and sample limit in the harness: the current one-million-call cap must not end a duration-governed run early. Remove the cap, or use a validated bounded-memory statistics method that continues measurement for the full requested duration and retains valid latency distributions. Require recorded elapsed time and completed operation counts to prove that the run did not stop at the old cap.

Before testing, budget the memory of input traces, write ledgers, latency samples or bounded statistics, validation state, and collectors. Prefer streaming or disk-backed traces and ledgers; place explicit bounds on resident buffers and account for their RSS in the host resource budget. Verify that harness metadata does not consume the RocksDB cache budget or materially change the measured process RSS; report harness RSS separately and apply the same collection method to every comparison path. Validate any bounded-memory statistics method against full-sample results on a representative run.

Estimate the full five-minute, three-repetition matrix before starting, including dataset growth, WAL growth, builds, active binaries, temporary databases, raw logs, reports, reproduction instructions, and evidence artifacts. Record per-run and total storage estimates, available capacity, and retention needs. Stage concurrency and avoid an exhaustive Cartesian product. Do not start a long run or matrix expansion without adequate capacity for the run and preservation of required evidence. Completed experiment databases and unnecessary archives need not be retained after validation; record their removal in the cleanup manifest. Record resident versus dataset size against an explicitly measured effective-cache budget, plus sustained flush, compaction, backlog, and disk behavior.

Stop and report before further work if live work conflicts, storage remains insufficient after user-authorized cleanup of identified targets, resource limits make the planned measurement invalid, or required raw evidence cannot be retained. The user-authorized cleanup may remove only identified completed experiment databases, old builds and binaries, and unnecessary archives with a cleanup manifest. Do not remove unknown or unrelated data, the current environment or source, or current required data.

**Required evidence:** preflight snapshot; source hashes; hashes for active binaries; complete build and run commands; toolchain and RocksDB versions; host and ZFS details; input and seed; validator output; RSS and completion evidence; proof that duration-governed runs passed the one-million-call boundary and reached their requested duration; latency-sample or validated bounded-statistics method and validation; memory and RSS budgets for traces, write ledgers, collectors, and validation state; raw logs, reports, reproduction instructions, and validation data; repetition order; five-minute matrix storage estimate covering dataset, WAL, active builds and binaries, temporary databases, and evidence; capacity estimate; cleanup manifest for removed databases and binaries; dataset and cache-budget measurements; engine health over sustained runs; workload-specific throughput, latency, and resource tradeoffs; preserved rejected-candidate evidence. Retaining every validated database or archive is not required.

## Phase 6: Independent validation and final comparison

**Status: Pending**

Run shared contract checks and independent replay against both implementations, then reopen persisted state and validate it. Validate request/response behavior and read histories; sorting returned revision tokens alone is insufficient. Allocator changes require rollback, failure, and restart/no-reuse coverage. State separately what process-crash tests establish and what they do not establish about power loss.

Compare the optimized RocksDB and B-link services using the same audited contract, external load, resource budgets, and authoritative host conditions. Report a workload-specific performance frontier and answer:

- What does the required DODB API cost relative to the raw unconditional subset reference?
- Which wrapper costs can be removed, and which constraints remain required by the contract?
- What are the B-link/RocksDB ratios for each representative workload and configuration?

Do not claim an absolute maximum or a universal winner. Preserve all artifacts and rejected attempts. Commit and push only scoped experiment work to the experiment branch. Never deploy to production.

**Required evidence:** shared contract results; independent replay and reopened-state validation; request/response and read-history checks; crash-scope statement; final paired OCI runs; per-workload ratios and uncertainty; resource and engine-health evidence; complete artifact index; final claims checked against the evidence.

## Cross-phase requirements and evidence checklist

Items begin **Pending** unless a different status is shown. Update them only with a link to the evidence that supports the status.

| Requirement or evidence | Status | Evidence |
| --- | --- | --- |
| API, consumer, test, and architecture audit completed | Pending | |
| Revision and physical commit LSN semantics resolved; all consumers checked | Pending | |
| All listed conditions, atomicity, revision, ABA, restart, and visibility behaviors validated | Pending | |
| Query and Scan snapshot, ordering, cursor, and limit behavior validated | Pending | |
| Tenant isolation and response budget before materialization validated | Pending | |
| Synchronized WAL and cancellation/shutdown lifetime safety validated | Pending | |
| Four paths and metric definitions frozen | Pending | |
| Shared deterministic workloads and workload coverage recorded | Pending | |
| Duplicate condition/mutation rejection and condition-only revision behavior validated | Pending | |
| Reused evidence independently checked for contract and provenance fit | Pending | |
| Normal performance and diagnostic instrumentation runs kept separate | Pending | |
| Candidate changes isolated, attributed, and rejected attempts retained | Pending | |
| OCI preflight capacity gate | Not passed | [PREFLIGHT.txt](results/rocksdb-contract-baseline-20261016/PREFLIGHT.txt) records the pre-cleanup 1.21 GB ZFS availability; current free space is unverified. Repeat preflight after cleanup before benchmarking. |
| Contract-audit documentation | Complete | [CONTRACT-AUDIT.md](results/rocksdb-contract-baseline-20261016/CONTRACT-AUDIT.md), based on architecture, backend-contract, and revision-range docs at commit `543b55c9`. |
| Contract correctness validation and tests | Pending | Tests have not been rerun; documentation completion is not a passing correctness result. |
| Resident-memory comparison strategy | Decision recorded | User chose resident-data comparison on both engines first; memory-exceed experiments are Deferred by user, not complete. [RESIDENCY-AUDIT.md](results/rocksdb-contract-baseline-20261016/RESIDENCY-AUDIT.md), commit `0358eb4e`, remains the audit evidence. |
| Remote build/benchmark execution gate | Pending | Wait for cleanup completion and a passing fresh capacity preflight; run builds and benchmarks serially. |
| User-authorized cleanup targets and removed database/binary manifest recorded | Pending | Cleanup is in progress; preserve current environment, source, and current required data. |
| OCI source/binary provenance and final host verification for measurements | Pending | No new benchmark may start before the capacity gate passes. |
| Screening, confirmation, and final repetition requirements met, including duration beyond the prior one-million-call cap | Pending | |
| Full latency samples or validated bounded-memory statistics continue for the requested duration | Pending | |
| Input traces, write ledgers, and harness memory budgeted and verified against cache and RSS limits | Pending | |
| Five-minute matrix growth and storage for WAL, builds, databases, and evidence estimated before execution | Pending | |
| Sustained cache, flush, compaction, backlog, and disk behavior recorded | Pending | |
| Workload-specific throughput, latency, and resource tradeoffs recorded without a single adoption threshold | Pending | |
| Independent replay, reopened state, and read-history validation completed | Pending | |
| Final paired comparison and workload-specific claims supported by artifacts | Pending | |
| Scoped commits/pushes completed; no production deployment performed | Pending | |

## Stop conditions requiring user intervention

Stop experiment execution and report the evidence and decision needed before proceeding when:

- The audit identifies an unresolved contract question. Resolve known document inconsistencies against the existing API, consumers, and tests; stop only if evidence shows that optimization would weaken a real API or consumer promise, including commit-LSN meaning, atomicity, snapshot behavior, revision behavior, or durability.
- An implementation or benchmark reveals a correctness failure, unexplained error, inconsistent history, or failure of independent replay.
- A candidate requires changing public contract, default behavior, durability, or a previously agreed design choice.
- OCI work has conflicting live processes, insufficient disk or host capacity after the user-authorized cleanup scope, uncertain authorization or host identity, unbudgeted harness memory, or would require deleting unknown or unrelated data, the current environment or source, or current required data.
- Required provenance or raw evidence cannot be retained, or a result cannot be validated against the same workload and contract.
- The available evidence does not support a conclusion and expanding the workload or measurement scope would exceed the approved constraints.

On stop, preserve the current state and artifacts, record the exact blocker and safe next decision, and do not continue other experiment phases until the user has resolved the blocker. Do not silently narrow the plan into short smoke-only results.

## Execution log

Append one entry for each phase attempt, including blocked and rejected work. Do not overwrite earlier entries. The plan-preparation entry records documentation only. Current phase statuses are Phase 1 In progress and Phases 2–6 Pending. The resident-memory decision is recorded: compare both engines first with in-memory data; defer the memory-exceed axis. User-authorized cleanup continues separately. Remote builds and benchmarks await cleanup completion and a passing fresh capacity preflight, after which they must run serially.

| Date (UTC) | Phase | Status | Source SHA / candidate | Commands and artifact paths | Findings and decision | Blocker or next action |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-10-16 | Plan | Complete | Base SHA `404026706a9180078c65ce1d29ff499df6ecb5b6`; initial document commit `3b6d13f002693d6fa5254b94cfc1f5d6219b1f25` | Plan document only: `dodb/docs/experiments/rocksdb-contract-baseline-plan.md` | Six-phase scope recorded; no implementation or measurements performed | Phases 1–6 pending |
| 2026-10-16 | Preflight | Stopped | Plan branch HEAD `1fd4fe6ae39b7172142654c5ce3ed043d462029b` | [PREFLIGHT.txt](results/rocksdb-contract-baseline-20261016/PREFLIGHT.txt), read-only transcript for `instance-20260923-1013` | Transcript records 1.21 GB ZFS available and insufficient capacity for over-cache data, required repetitions, builds, databases, WAL, and evidence retention. `MemTotal` is 11,159,744 kB; default Rust tools are 1.98.1; 1.97.1 is installed; exact RocksDB 11.8.1 archive availability remains unconfirmed because the lookup was incomplete. API audit remains preliminary; contract resolution incomplete. This is preflight evidence, not measurement evidence. No cleanup, expansion, implementation, or benchmark was performed. | User decision required: storage expansion or approved artifact preservation/migration policy. |
| 2026-10-16 | Resume | In progress | User-authorized cleanup; plan baseline SHA `5275689f15b37710fc3c6960daef75d01082c851` | Plan update only; OCI capacity gate still Not passed | User authorized deletion of identified completed experiment databases, old builds/binaries, and unnecessary archives with a cleanup manifest. Historical stop remains recorded. User chose resident-data comparison on both engines first; memory-exceed experiments are deferred, not complete. Final disk capacity awaits cleanup and fresh preflight. | Complete authorized cleanup and record its manifest; repeat capacity preflight; then start remote builds and benchmarks serially. |
| 2026-10-16 | 1 | In progress | Architecture, backend-contract, and revision-range docs at commit `543b55c9` | [CONTRACT-AUDIT.md](results/rocksdb-contract-baseline-20261016/CONTRACT-AUDIT.md) | Contract-audit documentation complete. Correctness tests have not been rerun; audit completion is separate from correctness validation. | Rerun required correctness validation; documentation completion does not imply correctness validation passed. |
| 2026-10-16 | Residency gate | Decision recorded | Resource audit at commit `0358eb4e` | [RESIDENCY-AUDIT.md](results/rocksdb-contract-baseline-20261016/RESIDENCY-AUDIT.md) | User chose comparison on both engines with data that fits in memory first. The memory-exceed axis is Deferred by user, not complete; the audit result is unchanged. | Run resident comparison after cleanup and a passing capacity preflight. Keep memory-exceed work deferred unless the user revises the decision. |
| | 2 | Pending | | | | |
| | 3 | Pending | | | | |
| | 4 | Pending | | | | |
| | 5 | Pending | | | | |
| | 6 | Pending | | | | |
