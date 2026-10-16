# RocksDB Contract Baseline and Optimization Plan

## Goal and claim boundary

Establish a fast RocksDB implementation that satisfies the DODB API and correctness contract, then compare it fairly with the B-link service. The raw unconditional subset is a reference point for measuring required API and wrapper costs. The winner is unknown in advance. Results must not claim an absolute RocksDB maximum: the full-semantics synchronous core may still contain DODB-required work, and the service paths retain their own integration costs.

The comparison must separate raw subset throughput, successful transactions, successful mutations, preparatory reads, and end-to-end service calls. A result is workload-specific and configuration-specific. No semantics or durability guarantees may be weakened to improve a score.

## Execution rules

- Preserve source provenance, exact commands, binaries, input seeds, raw output, validation output, host and filesystem details, and rejected candidates.
- Reuse existing verified evidence when its source, workload, correctness, and provenance match this plan. Record the artifact paths and the reason it qualifies; do not silently treat older evidence as current.
- Change one measured cause at a time. Keep rejected candidates and their evidence.
- Keep the default `Serialized` revision allocator mode and shared concurrency setting of 8 unless a separately justified change is explicitly approved. Benchmark optimized settings explicitly without changing defaults by implication.
- Root owns design and candidate-selection decisions. Execution follows the selected scope and reports blockers immediately.
- Do not deploy to production.

## Phase 1: Contract and architecture audit

**Status: Pending**

Map the DODB API, consumers, tests, RocksDB wrapper, service path, B-link path, and relevant concurrency and durability boundaries. Resolve the contract of revision values versus physical commit LSNs before changing identity or revision behavior. Audit every `commit_lsn` consumer before considering an opaque logical write identity. If current consumers rely on physical ordering or meaning, document that promise and obtain an explicit contract decision before weakening it. Independent concurrent response order is not a linearization order.

The audit must determine whether the implementation preserves all of the following:

- `RevisionEquals`, `Exists`, and `NotExists` conditions.
- Atomic writes across multiple keys, including condition keys that are not modified.
- Version change and ordering for repeated writes to the same key in one transaction.
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

**Required evidence:** API and consumer map; contract table with implementation and test references; all `commit_lsn` call sites and their assumptions; open questions and explicit decisions; baseline correctness gaps; existing evidence accepted for reuse.

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

Each candidate must preserve transaction semantics, tenant isolation, response-budget behavior, cancellation and shutdown safety, and synchronized WAL guarantees. Evaluate CPU, resident memory, tail latency, fairness, and sustained engine health alongside throughput. Preserve failed and rejected candidates with their evidence. Compare configurations as workload-specific tradeoffs and Pareto choices; do not claim one universal best configuration.

**Required evidence:** measured bottleneck and attribution; one-change candidate definition; correctness and durability results; throughput and tail latency; CPU and RSS; fairness; RocksDB flush, compaction, backlog, and disk behavior when relevant; accepted or rejected decision with rationale; source diff and provenance.

## Phase 5: Authoritative OCI measurements

**Status: Pending**

Use only the authorized OCI A1 ARM64 host and ZFS path `/bench/zfs/db`, with the approved stored SSH identity and normal host-key verification. Use pinned RocksDB 11.8.1 and Rust 1.97.1 with `+crc`. Do not substitute local or Debian measurements for authoritative results.

Before testing, inspect live processes, host resources, and available storage. Do not run concurrent builds or benchmarks and do not perform destructive cleanup. Build from the exact source SHA and retain immutable binaries. Record commands and hashes, host and filesystem, seeds, workload, and validators.

Use existing five-second runs only for screening, together with missing candidate screening. Confirm candidates with a 10-second warmup and 30-second measurement for at least three repetitions in rotated serial order. Final representative measurements must run for at least five minutes with at least three repetitions. Record resident versus dataset size against an explicitly measured effective-cache budget, plus sustained flush, compaction, backlog, and disk behavior. Stage concurrency and avoid an exhaustive Cartesian product. Estimate capacity before expanding the matrix.

Stop and report before further work if live work conflicts, storage is insufficient, resource limits make the planned measurement invalid, or safe artifact retention cannot be maintained. Do not clean up unknown or unrelated data to make room.

**Required evidence:** preflight snapshot; source and binary hashes; complete build and run commands; toolchain and RocksDB versions; host and ZFS details; input and seed; validator output; RSS and completion evidence; raw data; repetition order; capacity estimate; dataset and cache-budget measurements; engine health over sustained runs; preserved rejected attempts.

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

All items begin **Pending**. Update them only with a link to the evidence that supports completion.

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
| Reused evidence independently checked for contract and provenance fit | Pending | |
| Normal performance and diagnostic instrumentation runs kept separate | Pending | |
| Candidate changes isolated, attributed, and rejected attempts retained | Pending | |
| OCI preflight, capacity, source/binary provenance, and host verification recorded | Pending | |
| Screening, confirmation, and final repetition requirements met | Pending | |
| Sustained cache, flush, compaction, backlog, and disk behavior recorded | Pending | |
| Independent replay, reopened state, and read-history validation completed | Pending | |
| Final paired comparison and workload-specific claims supported by artifacts | Pending | |
| Scoped commits/pushes completed; no production deployment performed | Pending | |

## Stop conditions requiring user intervention

Stop the affected phase and report the evidence and decision needed before proceeding when:

- The audit finds that optimization would weaken a real API or consumer promise, including commit-LSN meaning, atomicity, snapshot behavior, revision behavior, or durability.
- An implementation or benchmark reveals a correctness failure, unexplained error, inconsistent history, or failure of independent replay.
- A candidate requires changing public contract, default behavior, durability, or a previously agreed design choice.
- OCI work has conflicting live processes, insufficient disk or host capacity, uncertain authorization or host identity, or would require destructive cleanup.
- Required provenance or raw evidence cannot be retained, or a result cannot be validated against the same workload and contract.
- The available evidence does not support a conclusion and expanding the workload or measurement scope would exceed the approved constraints.

On stop, preserve the current state and artifacts, record the exact blocker and safe next decision, and do not silently narrow the plan into short smoke-only results.

## Execution log

Append one entry for each phase attempt, including blocked and rejected work. Do not overwrite earlier entries.

| Date (UTC) | Phase | Status | Source SHA / candidate | Commands and artifact paths | Findings and decision | Blocker or next action |
| --- | --- | --- | --- | --- | --- | --- |
| | 1 | Pending | | | | |
| | 2 | Pending | | | | |
| | 3 | Pending | | | | |
| | 4 | Pending | | | | |
| | 5 | Pending | | | | |
| | 6 | Pending | | | | |
