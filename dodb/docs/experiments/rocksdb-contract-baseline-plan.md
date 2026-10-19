# RocksDB Contract Baseline and Optimization Plan

> Historical checkpoint (2026-10-16 UTC): Execution stopped after read-only preflight because OCI ZFS capacity was insufficient. At that checkpoint, no implementation or benchmarks had started. The subsequent direct helper launch exited 126 before Cargo; that failed attempt remains in the append-only execution log below.

> Current status (2026-10-19 UTC): The user chose resident-data comparisons first; memory-exceed experiments remain Deferred by user. After the historical exit-126 attempt, the existing OCI helper was invoked through `bash`. Shared contract/restart/crash gates passed; the B-link growth regression fix passed focused and shared checks, including the 182-pass/4-ignored storage suite at OCI SHA `01492ef403d02ed8ff41237b584d5e46dbe8a3dc` (same tracked tree as `15762ba40725a16f09de20d15ffcb97ebb42cef4`). The prior 57-case screen at source `15762ba40725a16f09de20d15ffcb97ebb42cef4` remains preliminary. Raw and full-API harnesses compile and pass focused unit tests at source `5d772a7448b1b6c29a0f00f501bd51fd83be03b9`; four-path smoke checks passed. Representative raw and RocksDB-service log1024 statistics matched exact-sample counts and percentile bounds. All 60 approved raw/full-API short-screen cases completed and passed validation at fixed source/binary; this remains screening evidence only. The opt-in condition-key Shared candidate passed its required release correctness gates and the approved 44-case five-second A/B screen at source `88b43098dab8736363d8c36446bbede27e1b77e1`. Write-heavy cases favored Shared, but the result remains preliminary; an engine-health observation harness gate precedes the 30-second confirmation and representative five-minute comparison. The final independent audit also remains pending. Phases 1–4 are In progress; Phases 5–6 remain Pending. See the append-only execution log, [raw harness evidence](results/rocksdb-contract-baseline-20261016/gate/raw-probe-787e48120/), and [shared-lock candidate evidence](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/). Production locking remains Exclusive by default and no adoption decision has been made.

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

**Status: In progress; shared contract, restart, and process-crash gates passed**

Contract-audit documentation is in [CONTRACT-AUDIT.md](results/rocksdb-contract-baseline-20261016/CONTRACT-AUDIT.md). Shared correctness, restart, and process-crash gates passed; the storage focused suite passed 182 tests with 4 ignored. See the gate artifacts under [gate](results/rocksdb-contract-baseline-20261016/gate/). This status covers those listed gates; it does not claim power-loss validation or complete the independent final audit in Phase 6.

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

**Required evidence:** API and consumer map; contract table with implementation and test references; all `commit_lsn` call sites and their assumptions; open questions and explicit decisions; baseline correctness gaps; existing evidence accepted for reuse. The audit and shared gates are linked above; independent replay and final paired validation remain in Phase 6.

## Phase 2: Freeze the four comparison paths

**Status: In progress; four-path smoke validation passed**

Define and freeze four paths with a shared external workload, resource limits, input data, and measurement method while retaining each path's necessary internal engine differences:

1. Raw RocksDB unconditional subset reference.
2. Full-semantics synchronous RocksDB core. Reuse the current research baseline where appropriate, but optimize the core too. Never label it fully unwrapped or maximal RocksDB.
3. Actual RocksDB service.
4. Final B-link service.

Record which contract operations each path includes, the exact RocksDB and B-link settings, synchronization and WAL behavior, concurrency controls, and any work excluded by the raw subset. Keep calls/s, successful transactions/s, successful mutations/s, preparatory reads, natural conflicts, intended conflicts, and unexpected errors distinct. Report service end-to-end latency separately from engine and stage timing. Do not add stage percentiles as if they were additive.

**Required evidence:** path definitions, input contract, and raw/full-API subset boundary are encoded in the preserved runner and execution artifacts under [raw probe gate](results/rocksdb-contract-baseline-20261016/gate/raw-probe-787e48120/). Four backends passed width 1/16 × clients 16/64 smoke validation, and 60 short-screen cases passed. Remaining: consolidate exact per-path settings and memory/resource-budget evidence into a reviewable path specification. The screen does not constitute authoritative performance evidence.

## Phase 3: Correctness and workload matrix

**Status: In progress**

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

**Status: In progress; shared-lock candidate correctness gates pending**

Prioritize candidates only when attribution shows that they address a material cost. Candidate areas include:

- `ReservedRanges` to remove or reduce revision-counter cost.
- Admission and client concurrency tuning.
- Encoding, allocation, copying, and FFI overhead.
- Bounded RocksDB configuration candidates when measurements justify them.
- The opt-in condition-key Shared candidate passed its required correctness gates and approved 44-case five-second screen. See [candidate design](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/CONDITION-SHARED-LOCK-DESIGN.md) and [screen results](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/SCREENING-RESULTS.md). The longer confirmation and representative five-minute comparison remain pending; Exclusive remains the default and no adoption decision has been made.

Each candidate must preserve transaction semantics, tenant isolation, response-budget behavior, cancellation and shutdown safety, and synchronized WAL guarantees. Evaluate CPU, resident memory, tail latency, fairness, and sustained engine health alongside throughput. Preserve failed and rejected candidates with their evidence. Compare configurations using workload-specific throughput, latency, CPU, resident memory, fairness, and engine-health tradeoffs. Do not adopt a candidate based on one arbitrary throughput threshold or claim one universal best configuration.

**Required evidence:** measured bottleneck and attribution; one-change candidate definition; correctness and durability results; throughput and tail latency; CPU and RSS; fairness; RocksDB flush, compaction, backlog, and disk behavior when relevant; accepted or rejected decision with rationale; source diff and provenance.

## Phase 5: Authoritative OCI measurements

**Status: Pending**

**OCI capacity/preflight gate: Passed for recorded runs.** User-authorized cleanup is complete. The initial fresh preflight at [gate/preflight.txt](results/rocksdb-contract-baseline-20261016/gate/preflight.txt) passed with 106,483,843,072 bytes available on ZFS. The historical direct helper invocation exited 126 before Cargo and is retained as a launch failure, not a test failure. Later builds and measurements used the helper through `bash`; current source and binary provenance is recorded with each run under [gate](results/rocksdb-contract-baseline-20261016/gate/).

The initial read-only preflight transcript is preserved at [PREFLIGHT.txt](results/rocksdb-contract-baseline-20261016/PREFLIGHT.txt). The test-gate preflight is [gate/preflight.txt](results/rocksdb-contract-baseline-20261016/gate/preflight.txt). The authorized host is ARM64 Neoverse-N1 with 2 CPUs and `MemTotal` 11,159,744 kB. The default Rust toolchain remains unchanged; the helper selected Rust 1.97.1 and `+crc`. Current raw-screen source is `5d772a7448b1b6c29a0f00f501bd51fd83be03b9`; the final research binaries and their hashes are recorded in the raw-probe evidence directory. The prior 57-case screen is source `15762ba40725a16f09de20d15ffcb97ebb42cef4`, and is reused only as preliminary screen evidence.

The cleanup report at [cleanup/README.md](results/rocksdb-contract-baseline-20261016/cleanup/README.md) records ZFS available space increasing from 1,301,846,016 to 107,144,802,304 bytes, a net increase of 105,842,956,288 bytes. The exact pre-cleanup available-byte count for `/` was not recorded; the report gives post-cleanup free space without claiming an exact delta. The fresh gate preflight followed cleanup and confirmed 106,483,843,072 bytes available on ZFS at its timestamp. This does not establish capacity for over-cache datasets or the full measurement matrix. Use only the authorized OCI A1 ARM64 host and ZFS path `/bench/zfs/db`, with the approved stored SSH identity and normal host-key verification. Use pinned RocksDB 11.8.1 and Rust 1.97.1 with `+crc`. Do not substitute local or Debian measurements for authoritative results.

### Resident-memory feasibility gate

The current B-link implementation keeps all pages in RAM. Its `cache_capacity` bounds only the B-tree cache and does not evict pages. The [RESIDENCY-AUDIT.md](results/rocksdb-contract-baseline-20261016/RESIDENCY-AUDIT.md), recorded at commit `0358eb4e`, finds that same-budget paired throughput with an over-memory dataset is unsupported under this behavior. No paging or eviction implementation is included in this plan.

The user chose to compare both engines first with a dataset that fits in memory. The memory-exceed experiment axis is Deferred by user and is not complete. Preserve the full six-phase scope, resident comparison on both engines, all planned workload axes, and the final 30-second three-repetition and five-minute three-repetition requirements. Any future memory-exceed comparison must document the B-link residency limitation. User-authorized storage cleanup is complete. Subsequent helper, test, build, smoke, and statistics-validation gates have passed as listed in the execution log. The approved 60-case screen is limited to 1-second warmup plus 5-second measurement, three rotated repetitions. No 30-second or five-minute measurement has started.

Before testing, inspect live processes, host resources, and available storage. Do not run concurrent builds or benchmarks. The user authorized cleanup of identified completed experiment databases, old builds and binaries, and unnecessary archives; apply that authorization only to listed targets and record removed databases and binaries in a cleanup manifest. Preserve unknown or unrelated data, the current environment and source, current required data, raw logs, reports, reproduction instructions, source hashes, and validation evidence. Build from the exact source SHA and retain binaries only as long as required for active validation; record their hashes and any later removal in the manifest. Record commands, host and filesystem, seeds, workload, and validators.

Use existing five-second runs only for screening, together with missing candidate screening. The approved 60-case screen completed with three five-second repetitions per profile; see [SCREENING-RESULTS.md](results/rocksdb-contract-baseline-20261016/gate/raw-probe-787e48120/SCREENING-RESULTS.md). The estimated maximum resident payload was 134,217,728 bytes; per-case estimates and ZFS free bytes are in the preserved `command.json` files. All 60 completed databases passed final and reopen checks; the file manifest covered 603 files and 8,409,781,017 bytes. Their LOG content was copied and verified before deleting only those validated databases; see [the cleanup manifest](results/rocksdb-contract-baseline-20261016/gate/raw-probe-787e48120/screen-r1-artifacts/database-cleanup-manifest.json). After cleanup, the authorized ZFS host reported 81 GiB free. Confirm candidates with a 10-second warmup and 30-second measurement for at least three repetitions in rotated serial order. Final representative measurements must run for at least five minutes with at least three repetitions. Before long runs, inspect every call-count and sample limit in the harness: the current one-million-call cap must not end a duration-governed run early. Remove the cap, or use a validated bounded-memory statistics method that continues measurement for the full requested duration and retains valid latency distributions. Require recorded elapsed time and completed operation counts to prove that the run did not stop at the old cap.

Before testing, budget the memory of input traces, write ledgers, latency samples or bounded statistics, validation state, and collectors. Prefer streaming or disk-backed traces and ledgers; place explicit bounds on resident buffers and account for their RSS in the host resource budget. Verify that harness metadata does not consume the RocksDB cache budget or materially change the measured process RSS; report harness RSS separately and apply the same collection method to every comparison path. Validate any bounded-memory statistics method against full-sample results on a representative run.

Estimate the full five-minute, three-repetition matrix before starting, including dataset growth, WAL growth, builds, active binaries, temporary databases, raw logs, reports, reproduction instructions, and evidence artifacts. Record per-run and total storage estimates, available capacity, and retention needs. Stage concurrency and avoid an exhaustive Cartesian product. Do not start a long run or matrix expansion without adequate capacity for the run and preservation of required evidence. Completed experiment databases and unnecessary archives need not be retained after validation; record their removal in the cleanup manifest. Record resident versus dataset size against an explicitly measured effective-cache budget, plus sustained flush, compaction, backlog, and disk behavior.

Stop and report before further work if live work conflicts, storage remains insufficient after user-authorized cleanup of identified targets, resource limits make the planned measurement invalid, or required raw evidence cannot be retained. Ordinary shell, build, and harness errors may be corrected within the approved scope while preserving original failures. Stop for hardware/data-loss risk, additional spend, a new database algorithm, or a material contract, locking-policy, public API, default, or durability change. User-authorized cleanup may remove only identified completed experiment databases after validation and with a content/size/hash manifest. Do not remove unknown or unrelated data, the current environment or source, or current required data. Production locking mode remains unchanged; any shared-read-only condition-lock idea is proposal-only.

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
| Condition-key shared-lock candidate scoped and correctness gates recorded; production policy unchanged | In progress | The pinned native review and required release correctness gates passed. The 44-case five-second screen completed at source `88b43098dab8736363d8c36446bbede27e1b77e1`; see [design](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/CONDITION-SHARED-LOCK-DESIGN.md) and [results](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/SCREENING-RESULTS.md). Longer confirmation and adoption remain pending; Exclusive stays the default. |
| OCI preflight capacity gate | Complete | Fresh test-gate preflight passed after cleanup; [gate/preflight.txt](results/rocksdb-contract-baseline-20261016/gate/preflight.txt) records 106,483,843,072 bytes available on ZFS. This is a capacity snapshot, not proof the full measurement matrix fits. |
| Contract-audit documentation | Complete | [CONTRACT-AUDIT.md](results/rocksdb-contract-baseline-20261016/CONTRACT-AUDIT.md), based on architecture, backend-contract, and revision-range docs at commit `543b55c9`. |
| Contract correctness validation and tests | In progress | Shared contract/restart/crash gates passed. The storage suite passed 182 tests with 4 ignored at OCI source `01492ef403d02ed8ff41237b584d5e46dbe8a3dc`, whose tracked tree matches local source `15762ba40725a16f09de20d15ffcb97ebb42cef4`; see [storage suite log](results/rocksdb-contract-baseline-20261016/gate/blink-growth-followup/helper-runs/storage-lib/cargo.log). Independent replay and remaining contract checks are pending. |
| Resident-memory comparison strategy | Decision recorded | User chose resident-data comparison on both engines first; memory-exceed experiments are Deferred by user, not complete. [RESIDENCY-AUDIT.md](results/rocksdb-contract-baseline-20261016/RESIDENCY-AUDIT.md), commit `0358eb4e`, remains the audit evidence. |
| Remote build/benchmark execution gate | In progress | Helper-driven build, raw/service focused tests, four-path smoke, exact-stat validation, prior 57-case screen, and current approved short screen are recorded under [gate](results/rocksdb-contract-baseline-20261016/gate/). 30-second and five-minute runs remain pending. |
| User-authorized cleanup targets and removed database/binary manifest recorded | Complete | [cleanup/README.md](results/rocksdb-contract-baseline-20261016/cleanup/README.md) summarizes the deletion journals and retained evidence; exact `/` filesystem delta was not measured. |
| OCI source/binary provenance and final host verification for measurements | In progress | Current fixed source SHA `5d772a7448b1b6c29a0f00f501bd51fd83be03b9`, research binary hashes, commands, and host evidence are retained under [raw probe gate](results/rocksdb-contract-baseline-20261016/gate/raw-probe-787e48120/). |
| Screening, confirmation, and final repetition requirements met, including duration beyond the prior one-million-call cap | Pending | |
| Full latency samples or validated bounded-memory statistics continue for the requested duration | In progress | Representative 5-second validation passed: raw exact-vs-log1024 comparison, 40,636 observations; service comparison, 58,371 aggregate observations across six latency groups. Counts matched and p50/p95/p99 errors stayed within bucket bounds. See `stat-validation-ssh-output.txt` under [raw probe gate](results/rocksdb-contract-baseline-20261016/gate/raw-probe-787e48120/). Long requested-duration runs remain pending. |
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

Append one entry for each phase attempt, including blocked and rejected work. Do not overwrite earlier entries. The plan-preparation entry records documentation only. Current statuses are Phases 1–3 In progress and Phases 4–6 Pending. The resident-memory decision remains: compare both engines first with in-memory data; defer the memory-exceed axis. The historical exit-126 launch remains recorded below; later runs invoked the helper through `bash`.

| Date (UTC) | Phase | Status | Source SHA / candidate | Commands and artifact paths | Findings and decision | Blocker or next action |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-10-16 | Plan | Complete | Base SHA `404026706a9180078c65ce1d29ff499df6ecb5b6`; initial document commit `3b6d13f002693d6fa5254b94cfc1f5d6219b1f25` | Plan document only: `dodb/docs/experiments/rocksdb-contract-baseline-plan.md` | Six-phase scope recorded; no implementation or measurements performed | Phases 1–6 pending |
| 2026-10-16 | Preflight | Stopped | Plan branch HEAD `1fd4fe6ae39b7172142654c5ce3ed043d462029b` | [PREFLIGHT.txt](results/rocksdb-contract-baseline-20261016/PREFLIGHT.txt), read-only transcript for `instance-20260923-1013` | Transcript records 1.21 GB ZFS available and insufficient capacity for over-cache data, required repetitions, builds, databases, WAL, and evidence retention. `MemTotal` is 11,159,744 kB; default Rust tools are 1.98.1; 1.97.1 is installed; exact RocksDB 11.8.1 archive availability remains unconfirmed because the lookup was incomplete. API audit remains preliminary; contract resolution incomplete. This is preflight evidence, not measurement evidence. No cleanup, expansion, implementation, or benchmark was performed. | User decision required: storage expansion or approved artifact preservation/migration policy. |
| 2026-10-16 | Resume | In progress | User-authorized cleanup; plan baseline SHA `5275689f15b37710fc3c6960daef75d01082c851` | Plan update only; OCI capacity gate still Not passed | User authorized deletion of identified completed experiment databases, old builds/binaries, and unnecessary archives with a cleanup manifest. Historical stop remains recorded. User chose resident-data comparison on both engines first; memory-exceed experiments are deferred, not complete. Final disk capacity awaits cleanup and fresh preflight. | Complete authorized cleanup and record its manifest; repeat capacity preflight; then start remote builds and benchmarks serially. |
| 2026-10-16 | 1 | In progress | Architecture, backend-contract, and revision-range docs at commit `543b55c9` | [CONTRACT-AUDIT.md](results/rocksdb-contract-baseline-20261016/CONTRACT-AUDIT.md) | Contract-audit documentation complete. Correctness tests have not been rerun; audit completion is separate from correctness validation. | Rerun required correctness validation; documentation completion does not imply correctness validation passed. |
| 2026-10-16 | Residency gate | Decision recorded | Resource audit at commit `0358eb4e` | [RESIDENCY-AUDIT.md](results/rocksdb-contract-baseline-20261016/RESIDENCY-AUDIT.md) | User chose comparison on both engines with data that fits in memory first. The memory-exceed axis is Deferred by user, not complete; the audit result is unchanged. | Run resident comparison after cleanup and a passing capacity preflight. Keep memory-exceed work deferred unless the user revises the decision. |
| 2026-10-16 | OCI cleanup and gate preflight | Complete | Source worktree SHA `c4630fd6cf45dfd74236a6ac0d9320715d260df4` | [cleanup/README.md](results/rocksdb-contract-baseline-20261016/cleanup/README.md), [gate/preflight.txt](results/rocksdb-contract-baseline-20261016/gate/preflight.txt) | Cleanup recovered 105,842,956,288 bytes of ZFS available space from the recorded baseline; fresh preflight passed with 106,483,843,072 bytes available. Exact `/` filesystem delta was not measured. Memory-exceed experiments remain Deferred by user. | Preflight was followed by a helper launch attempt; see next entry. |
| 2026-10-16 | Contract test launch attempt | Stopped before tests | Source SHA `c4630fd6cf45dfd74236a6ac0d9320715d260df4`; DODB core matches `404026706a9180078c65ce1d29ff499df6ecb5b6` | [gate/commands.txt](results/rocksdb-contract-baseline-20261016/gate/commands.txt), [gate/core-contract-launch.txt](results/rocksdb-contract-baseline-20261016/gate/core-contract-launch.txt), [gate/source-transfer.txt](results/rocksdb-contract-baseline-20261016/gate/source-transfer.txt) | The direct `./dodb/scripts/run-oci-rocksdb-build.sh` invocation exited 126 with `Permission denied` before Cargo started. Tests were not started; this is a launch error, not a test failure. No retry or second command was run. | If execution resumes, invoke the existing helper through `bash dodb/scripts/run-oci-rocksdb-build.sh`; then follow the recorded serial test commands. |
| 2026-10-19 | Gates, raw harness, and four-path smoke | Complete for named gates | Current research source `5d772a7448b1b6c29a0f00f501bd51fd83be03b9`; prior correctness fix `15762ba40725a16f09de20d15ffcb97ebb42cef4` | [gate](results/rocksdb-contract-baseline-20261016/gate/), [raw probe evidence](results/rocksdb-contract-baseline-20261016/gate/raw-probe-787e48120/) | Shared contract/restart/crash gates passed. The 182-pass/4-ignored storage suite ran at OCI SHA `01492ef403d02ed8ff41237b584d5e46dbe8a3dc`, with the same tracked tree as `15762ba40725a16f09de20d15ffcb97ebb42cef4`. Raw and service harnesses compiled; focused test suites passed at current research source. All four backends passed smoke at clients 16/64 and widths 1/16. The prior 57-case screen remains preliminary. Exact-vs-log1024 representative checks passed for raw 40,636 observations and service 58,371 aggregate observations, with matching counts and p50/p95/p99 errors within bucket bounds. | Complete the remaining path/resource specification and independent validation; do not start 30-second/5-minute measurements or implement lock-policy changes without a new decision. |
| 2026-10-19 | Raw/full-API short screen | Complete for 60 short cases | Fixed source `5d772a7448b1b6c29a0f00f501bd51fd83be03b9`; raw binary SHA-256 `8ee11c9afeff6c3687643ad14d6f1c35597d984f988d4f8290935547566d8d48` | [screen result](results/rocksdb-contract-baseline-20261016/gate/raw-probe-787e48120/SCREENING-RESULTS.md), [preserved per-run artifacts](results/rocksdb-contract-baseline-20261016/gate/raw-probe-787e48120/screen-r1-artifacts/), [cleanup manifest](results/rocksdb-contract-baseline-20261016/gate/raw-probe-787e48120/screen-r1-artifacts/database-cleanup-manifest.json) | All 60 serial cases passed final/reopen validation at seed `20261013`, values 512 bytes, 256 preseeded slots/client, widths 1/16, clients 16/64, 1-second warmup, 5-second measurement, three rotated repetitions. The final runner print raised a Python 3.9 `json.dumps(flush=...)` argument error after all per-case artifacts were written; a separate postprocessor validated the existing artifacts without rerunning measurements. It corrected the report metadata to state three 5-second repetitions per profile and preserved the original misleading metadata and failure output. All 60 DBs were removed after validation; 603 database files totaling 8,409,781,017 bytes were recorded, and 96 LOG files were retained and hash-verified. | Do not start 30-second/5-minute runs or lock-policy changes without a new decision. |
| 2026-10-19 | Condition-key shared-lock correctness and short A/B screen | Complete for required gates and 44 short cases | Final source `88b43098dab8736363d8c36446bbede27e1b77e1`, tree `0e8d1da6c60fd83fb11fd3d6f1a643665e3079a2`; `service_workload` SHA-256 `ff659fb3f0ea5ce8998bec1fa4297b9ec16b706f8553ba3555394a21a8db6977`; raw SHA-256 `6d1913ba3da2ec87b16c8c9c62955afe76f586ecec22812858c81ba2a64d3890` | [design and gate status](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/CONDITION-SHARED-LOCK-DESIGN.md), [screen results](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/SCREENING-RESULTS.md), [per-run evidence and logs](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/oci-evidence-88b43098d/) | Release correctness suite passed 35 tests at parent candidate source `15e0077b4e648930c7e01def84317b566f4e18e3` (lib 27, history 1, process-crash 4, read 2, transaction 1). At final SHA, test-only changes scoped a helper to test builds and added existing/missing condition-only keys to the crash scenario; final release crash gate passed 4/4. The screen ran 42 diagnostic-off cases plus two separate diagnostics, all passed full-window/final/reopen/prefix/revision checks. All 44 successful databases were removed after 44 manifests covering 883 DB files and 160 hash-verified LOG files were saved; recorded DB files totaled 3,509,842,852 bytes. Shared median workflow throughput was 1.07×/2.02×/1.93×/2.08×/1.35× Exclusive at c16 `rw95`/`rw80`/`rw50`/`rw20`/`mt-mix`, 2.27× at c32 `rw20`, and 1.03× at c32 `mt-mix`; preparatory Gets are counted as separate API calls. The diagnostic mean condition-only lock wait was 3.949 ms Exclusive and 0 ms Shared, from one repetition per mode. These five-second results are preliminary; Exclusive remains the default. | Complete the approved engine-health observation harness gate before starting the 30-second confirmation and representative five-minute comparison. |
