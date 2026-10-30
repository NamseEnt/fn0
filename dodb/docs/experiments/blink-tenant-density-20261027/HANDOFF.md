# Continue the B-link active-tenant density investigation

Tracking issue: [NamseEnt/fn0#126](https://github.com/NamseEnt/fn0/issues/126).

Read [PLAN.md](PLAN.md) for the complete cost audit, source map, evidence boundaries, prioritized candidates, experiment design, and correctness gates.

## Enter the correct workspace

```sh
cd /Users/namse/fn0/.worktrees/dodb-tenant-density-20261027
git status --short --branch
git rev-parse --show-toplevel
git rev-parse --git-common-dir
```

Expected branch: `experiment/blink-tenant-density-20261027`.

This is a linked worktree of `/Users/namse/fn0`, with origin `https://github.com/NamseEnt/fn0.git`. It is under the fn0 checkout, not the standalone dodb repository. Its code is under `dodb/`; the Cargo workspace manifest is at the worktree root.

The branch starts at reviewed source `a7271e9e35c5e0fa9d2a1c0e5eedc030f12de497`, followed by documentation-only preparation commit `f9a9c0889` and the subsequent A1 increment. It intentionally retains the backend-hotpaths experiment implementation and evidence rather than substituting the production main branch. Read the Git log for the current A1 commit.

Do not use or modify `/Users/namse/dodb`. Keep the existing main checkout, source experiment worktree, other worktrees, databases, and archived/rejected evidence intact. The local common Git exclude file ignores `/.worktrees/` so this nested worktree does not create untracked noise in main. Do not remove other exclude entries.

## Preparation completed before implementation

- Rechecked the original worktree, branch, reviewed commit, clean state, and remote source SHA.
- Created this new branch and linked worktree under fn0.
- Published the complete analysis and plan in English as issue #126.
- Added the English plan and this handoff.
- No engine/harness implementation or new benchmark was started during preparation.
- No defaults, production deployment, main history, or previous evidence changed.

The preparation commit contains documentation only. Do not confuse it with the frozen source SHA or the later A1 implementation.

## A1 implementation and evidence

The user subsequently authorized A1 implementation with one GPT-6 Luna worker at high reasoning effort, with planning and review owned by the primary agent. A1 is complete as an opt-in implementation and local allocation/recovery diagnostic. Read [A1-IMPLEMENTATION.md](A1-IMPLEMENTATION.md), [A1-RESULTS.md](A1-RESULTS.md), and [A1-WORKER-EVIDENCE.md](A1-WORKER-EVIDENCE.md) for the implementation, acceptance status, commands, and evidence limits.

The candidate is enabled by the default-disabled dodb-storage Cargo feature `blink-writer-pin-reuse`; a server test build enables it with `--features dodb-storage/blink-writer-pin-reuse`. It releases the writer-owned admission pin and evaluates parallel reuse before acquiring the preparation pin. Storage/WAL defaults and file formats remain unchanged.

The local fixed-key diagnostic compares 512, 513, and 4096-byte values with no external reader, an old pinned generation, and that generation released. Overflow high-water plateaus in safe candidate phases; the external-reader phase remains conservative. These observations do not establish retained-memory savings, physical disk return, throughput, or active-tenant density. The full tree remains resident.

The real-service recovery test uses subprocess SIGKILL before commit for both legacy WAL and actual segmented durable-base packs, then recovery, an acknowledged service write, a second pre-commit SIGKILL, and final value/revision verification. The separate appended-tail test uses clean shutdown and must not be presented as the subprocess crash evidence.

Whole-workspace formatting is blocked by existing vendor Cargo workspace metadata. Strict Clippy retains existing findings; targeted package formatting and advisory Clippy are recorded separately. Preserve the interrupted worker suite as an interrupted attempt, not a passing result.

## A1 scope preserved from preparation

The original first implementation scope was A1 in PLAN.md: measure and verify writer-owned generation pin lifetime versus page reuse in the planned service write path. The list below records that preparation scope; it is not a request to repeat completed A1 checks.

At the reviewed source, `apply_planned_transaction_group` pins `committed_view` for admission and retains it when `can_reuse_pages` checks for zero pins. The service enables planned execution. Values above 512 bytes use overflow allocation, so fixed-key overwrite was the narrow first diagnostic. This was a static source finding at preparation time. A1 now supplies the local growth/reuse observations above; performance attribution remains unmeasured.

1. Recheck the source around admission, reuse decisions, and allocation/free before editing.
2. Define task-owned accounting for allocations, free pages, reuse attempts/results, pin count/lifetime, high-water, and bytes by representation.
3. Preserve the implementation plan and acceptance criteria before implementation.
4. Validate no-external-reader and pinned-reader cases with 512/513-byte and 4 KiB values.
5. Preserve the page-reuse safety proof and real-service recovery sequence: incomplete transaction recovery -> write succeeds -> another recovery.
6. Keep any experimental behavior opt-in/default-disabled. Do not promote settings or run the broader density matrix based on one small diagnostic.

## Density harness increment

After A1, the user authorized the next preparation increment using the same single GPT-6 Luna worker at high reasoning effort, with planning/review and real smoke execution owned by the primary. Read [DENSITY-HARNESS-PLAN.md](DENSITY-HARNESS-PLAN.md) and [DENSITY-HARNESS-RESULTS.md](DENSITY-HARNESS-RESULTS.md) for its current acceptance status, commands, source provenance, and limits.

The increment adds a dedicated `tenant_density_workload` binary and extends the diagnostic worker for minimal-native and explicit segmented checkpoint settings. The historical `service_workload` is preserved. Separate setup, measurement, oracle, and reopened-oracle processes distinguish the measurement epoch's active, opened-idle, and never-opened population. Memory summaries filter the measurement backend by PID/start-time and exclude setup/oracle observations. Workload schedules and failure accounting must be accepted before any capacity claim.

This preparation increment is complete. Accepted runtime source is `99932ec33419214a7ee1cc542e937d31ef733a7c`; later documentation/evidence commits do not change that frozen binary provenance. New workload tests (15) and affected existing checks (9) passed, as did default/A1-feature link builds. Four real cells in `correctness-99932ec33-r1` passed acknowledged-key and clean-reopen validation, accounting, and matched-input checks. All-active cells used 17 active tenants and 1,700 requests; sparse cells used 17 registered, 3 active, 2 opened-idle, and 90 requests. No cell is a density/SLO/performance result. Keep the raw checkpoint counters' worker-lifetime scope and the sparse memory sampling explicit.

The isolated OCI build root is `/home/opc/tenant-density-harness-20261027`; build output is on the boot filesystem. Preserve source probe archives, failed logs, databases, and previously accepted cells. Real smoke databases are under `/bench/zfs/db/tenant-density-harness-20261027`. The benchmark filesystem had only about 0.39 GiB above the required 10 GiB free reserve during preparation, so this increment permits only tiny correctness/accounting smokes. Recheck live capacity before future work.

The shared-cache/WBM reference and common cgroup budget increments are implemented separately; see the following sections. Production-wide pin ages, allocator/representation retained bytes, and queue/snapshot byte budgets remain absent. The density/maintenance matrix and adoption decision are not completed by making the harness executable.

## Shared-resource RocksDB increment

Read [SHARED-ROCKSDB-PLAN.md](SHARED-ROCKSDB-PLAN.md) and [SHARED-ROCKSDB-RESULTS.md](SHARED-ROCKSDB-RESULTS.md). The primary completed this increment without delegation. Accepted implementation/runtime source is `b67484d02faf8fdac3007a123f863c130ef906e5`; documentation/evidence commits follow it.

`minimal-native-shared` requires explicit shared cache/WBM bytes, per-tenant write-buffer bytes/open-file limits, and process-global flush/compaction pool counts. The service uses one cache and WBM across tenant TransactionDBs, charges memtables to the cache, caches index/filter blocks, and verifies actual native options and aggregate usage. Existing routes retain their defaults. WBM writer stalls are disabled and the write-history target is one byte: two preserved tiny stall-enabled pressure probes stopped making progress. This reference provides soft flush/cache settings, not a hard process/OS memory cap.

All 37 clean-source tests passed, including seven new checks; default and A1 feature links passed. Four default-feature OCI correctness cells completed 3,580 successful requests, both full-population oracles, exact matched-input review, actual resource-sharing checks, and complete process shutdown. Evidence is in `evidence/shared-rocksdb/oci/`. The remote namespace is `shared-correctness-b67484d02-r1`; frozen binaries and all databases remain preserved. No performance/density or new crash-recovery claim is made.

At the shared-reference increment's completion, common enforcement and further pressure accounting were still missing. The following increment adds cgroup enforcement and periodic native/kernel observations. The historical shared-reference free-space observation remains 11,074,265,088 bytes; use the later cleanup/capacity evidence for continuation and keep the 10 GiB reserve. Issue #126 remains open.

## Common memory budget and disk preparation increment

Read [MEMORY-PRESSURE-PLAN.md](MEMORY-PRESSURE-PLAN.md) and [MEMORY-PRESSURE-RESULTS.md](MEMORY-PRESSURE-RESULTS.md). Runtime source is `d506d00dd1f6324bf0074ba2ec0fd0f84792a1f8`; controller corrections and evidence are recorded separately without changing those binaries. The primary executes this increment without delegation.

The density driver can enforce one common cgroup v2 parent budget across all backend epochs, with identical child limits, zero swap allowance, verified worker membership before runtime/backend construction, periodic native/kernel observations, and preserved partial reports after worker failure. The driver and uncharged ARC/host caches remain outside the budget. This is opt-in harness enforcement, not an engine eviction or application admission feature. The host exposes no PSI files; these observations remain explicitly unavailable.

All 68 clean-source tests and both default/A1 compatibility links passed. Four OCI correctness cells in `memory-pressure-d506d00dd-r3` completed 1,200 requests with 4 KiB values under identical 256/192 MiB maximum/high limits, both full-population oracles, and exact matched-input review. Separate high and anonymous-OOM probes passed. A real-driver 4 MiB expected-failure cell verified worker OOM, driver survival, and preserved partial diagnostics. The 22 recorded driver/backend identities had exited at final review. No successful cell was rerun, and these results do not establish a capacity boundary or performance ranking.

Cleanup removed only 13 explicit obsolete compiler/intermediate paths while preserving source, result logs/reports, databases, frozen runtime binaries, and the pinned native archive. Settled benchmark free space increased from about 10.3 GiB to 40.0 GiB before the new small cells. Recheck live free space, RAM, ARC, and competing processes before continuation; preserved databases and build outputs still occupy both filesystems.

The next decision work is to size and freeze a modest paired E2/E3 capacity bracket under the same explicit budget and per-tenant service objective, then examine E4 maintenance near that boundary. Keep sampling gaps and instrumentation perturbation explicit. Unsupported retained-heap/pin/queue representations prevent exact causal attribution; they must not be presented as zero or silently substituted with normalized checkpoint bytes. No density/performance winner or default promotion follows from the correctness cells.

## Bounded capacity increment accepted

Read [DENSITY-CAPACITY-PLAN.md](DENSITY-CAPACITY-PLAN.md), [DENSITY-CAPACITY-RESULTS.md](DENSITY-CAPACITY-RESULTS.md), [CAPACITY-CONTINUATION.md](CAPACITY-CONTINUATION.md), [CAPACITY-SUPPLEMENT-PLAN.md](CAPACITY-SUPPLEMENT-PLAN.md), and [CAPACITY-SOURCE-AUDIT.md](CAPACITY-SOURCE-AUDIT.md). Runtime source is `aaa40565529ca92150df5d94e19b02ca93859f71`. Only periodic diagnostic detail changed after the accepted memory-pressure source; engine semantics and A1/default settings are preserved. All 68 checks and both feature links passed; the same seventeen pre-existing Clippy diagnostics remain.

The original `capacity-aaa405655-r1` namespace stopped after 36 unique cells because B-link's six-tenant E3 setup worker exceeded its existing fifteen-second shutdown deadline. B-link's eight-tenant E3 setup had already failed with kernel OOM; shared RocksDB completed both counts. The stopped namespace, failed databases, logs, and original statuses remain immutable.

`capacity-aaa405655-r2` completed the frozen sequence using an explicit composite evidence view: 36 unchanged prior reports plus sixteen new executions. It classifies the exact recorded setup-shutdown deadline separately from OOM. Its provenance links the original namespace and controller/reducer hashes. Do not add the two namespaces' case counts together or restart already completed cells.

The main and supplemental controllers have finished. Runtime records and frozen binaries remain under `/home/opc/tenant-density-harness-20261027/capacity/` on OCI; the complete namespaces are `capacity-aaa405655-r2` and `capacity-supplement-aaa405655-r1`. Databases and cgroups remain in their recorded original locations, including the failed r1 cells. Both complete views have `completion.json`; every recorded process identity had exited at collection. The source checkout was still clean and detached at the runtime SHA during the 2026-10-29 collection.

The supplement added three sixty-second shared-RocksDB confirmations at eight E3 tenants and a paired 180-second maintenance observation at the actual twenty-eight-tenant E2 boundary with 2,000 ms sampling. All five completed and met the experimental objective. Across all cohorts there are 57 unique executions, 55 operational completions, 52 SLO passes, 2,304,000 successful offered requests in complete reports, and 110 passing full-population oracles. The highest three-of-three counts are E2 28 for both routes and E3 five for B-link versus at least eight for the reference. These are instrumented finite-window observations, not a production maximum.

Both raw archives and their extracted original/composite/supplemental records are preserved locally in `evidence/capacity/`. `review_all.py` reproduces the hash/accounting/preservation checks and writes `final-review.json`. Collection verified 57 distinct databases and 279 exited process identities, with at least 34.72 GiB benchmark free space and 5.02 GiB host MemAvailable in recorded runtime samples. Recheck live capacity before new work. Keep issue #126 open.

The completed one-tenant cadence sensitivity exposed material observation-retention cost: shared RocksDB's epoch peak was 8,351,744 bytes at 1,000 ms and 35,336,192 bytes at 100 ms. [OBSERVATION-RETENTION-PLAN.md](OBSERVATION-RETENTION-PLAN.md) preserved the follow-up before editing. Its implementation is now source `023b18bd8921b593f243f59b9eaa1576125c6572`; all 74 checks, unchanged seventeen advisory Clippy findings, and default/A1 links passed. Read [OBSERVATION-RETENTION-RESULTS.md](OBSERVATION-RETENTION-RESULTS.md) and [OBSERVATION-RETENTION-MATRIX.md](OBSERVATION-RETENTION-MATRIX.md) for the live runtime comparison.

The current-host archived-binary cadence control `observations-control-aaa405655-r1` completed four cells. Candidate `observations-023b18bd8-r1` stopped at its sixteenth cell, `confirm-e3-n5-r3-blink`, after the measurement worker exceeded the unchanged parent twenty-second stop watchdog. Its failure is distinct from the worker-internal fifteen-second server-shutdown gate. It has `controller-failure.json` and no `completion.json`. Later six/eight-tenant, survivor, and maintenance cells were not executed. Do not resume or replay this namespace.

All twenty new raw cells are collected in `evidence/observation-retention/runtime.tar.gz` and extracted beside it. `review_observations.py` reproduces nineteen operational completions, fourteen SLO passes, 603,000 successful offered requests in complete reports, thirty-eight full-population oracles, twenty preserved databases, and ninety-eight exited process identities. The single failure has no available workload counts or complete measurement health report; do not count it as zero throughput or a validated success. Source/native/binary hashes and old artifacts remain preserved. At 100 ms, the current-host low-load control confirms reference/B-link final epoch peak reductions from 34.00/13.95 to 8.25/6.12 MiB; driver DOM cost remains outside the backend budget. E2 twenty-eight-tenant acceptance was not reconfirmed in this new source/date cohort.

Read [DENSITY-DECISION.md](DENSITY-DECISION.md), [SHUTDOWN-DIAGNOSTIC-PLAN.md](SHUTDOWN-DIAGNOSTIC-PLAN.md), [SHUTDOWN-DIAGNOSTIC-MATRIX.md](SHUTDOWN-DIAGNOSTIC-MATRIX.md), and [SHUTDOWN-DIAGNOSTIC-RESULTS.md](SHUTDOWN-DIAGNOSTIC-RESULTS.md). Shutdown tracing and available-but-unvalidated failure counters are implemented separately in source `b22638bdcc4a0b22dad0606c3003d193cf41bbed`. All seventy-nine checks, unchanged seventeen advisory Clippy findings, and default/A1 links passed with default last. Its new bounded diagnostic namespace is `shutdown-b22638bdc-r1`. It leaves engine code and both deadlines unchanged. The primary owns design and evidence judgment; the user's latest instruction requires every implementation task to use GPT-6 Luna at high reasoning effort and return concise summaries only.

The shutdown namespace stopped after four attempts: two tiny correctness/normal-trace completions and one five-tenant reference completion, followed by a five-tenant B-link setup watchdog failure. Complete reports contain 15,200 successful calls and six oracles. Four databases and seventeen exited process identities are verified in the collected archive. The failed worker completed final properties and entered `async_server_shutdown`; no terminal marker was observed before the parent killed it. Offered load never started in that cell. Do not call it the same measured phase as the earlier source-023 measurement failure. The source-023 stop and all omitted cells remain preserved.

Both stopped namespaces and complete controls are locally collected. `review_shutdown.py` reproduces the exact setup-parent-watchdog classification, PID/source/phase evidence, normal traces, archive hash, and preservation checks. The prior unclassified derived review is retained separately. Do not restart either stopped namespace.

The external wait observation is also finished and preserved; read [SHUTDOWN-OFFCPU-RESULTS.md](SHUTDOWN-OFFCPU-RESULTS.md). `shutdown-offcpu-b22638bdc-r1` reused source b226 and its accepted default binaries, with no rebuild or timeout/policy change. First two cells completed 30,000 calls and four oracles together; the third failed in setup before offered load. Eighteen valid snapshots over +2.039 to +19.109 seconds showed the same Tokio-named thread in state D / `mem_cgroup_handle_over_high`, with source/PID/start-time/leaf ownership verified before and after reads. One post-exit FileNotFound remains a raw observer error; it does not invalidate those earlier individual snapshots or make the cell pass. The allocating user-space call site and earlier unprofiled failures' internal causes remain unproven.

`evidence/shutdown-offcpu/runtime.tar.gz` and `final-review.json` preserve the three attempts, three databases, twelve exited identities, sixteen empty groups, binaries, kernel observations, original review errors, and actual/planned collector/reviewer versions. The reducer verifies source/native/binary/file/archive/validation identities and distinguishes `all_cells_valid=false` from `usable_wait_evidence=true`. Extracted duplicate binaries are ignored by exact path but remain in the archive, local extraction, and OCI. The first preflight and premature transfer check are recorded separately; no workload was rerun for reporting corrections.

The next controlled question is E2 arrival phase under the same rate, memory policy, global in-flight limit, and SLO. Read [E2-ARRIVAL-PHASE-PLAN.md](E2-ARRIVAL-PHASE-PLAN.md). Only optional density-driver/config/scheduler changes are approved; synchronized remains default with legacy fingerprints. Fixed-total rejects staggered mode. No existing E3 failed setup is retried using an offered-load option. Implementation is assigned to GPT-6 Luna high; the primary must review source, preserve a clean runtime commit, complete checks/links, and freeze exact new controls before OCI execution.

The entire active build cache was checksum-relocated to ZFS and the original logical target path retained as a symlink. Source, active cache, runtime/native evidence, failed databases, and cgroups remain preserved. Keep offer-boundary and post-offer peaks separate and record driver memory; worker memory is not isolated engine heap. Issue #126 remains open.

## Read existing evidence in order

1. `dodb/docs/experiments/results/backend-hotpaths-20261026/RESULTS.md`
2. `dodb/docs/experiments/results/backend-profiling-20261026/RESULTS.md`
3. `dodb/docs/experiments/results/query-get-put-mix-20261027/RESULTS.md`
4. Their plans, summary JSON, and raw evidence when needed.

The profile predates the accepted hot-path changes. Open-shard filesystem-check removal and minimal-native mutation-only old-value-fetch removal already exist. Do not propose them again or use their old profile shares as current causal evidence.

The current tree is fully resident. The B-link `cache_capacity` field does not bound residency. There is already one process-wide physical-checkpoint worker permit, but no global byte budget for waiting tenants. Previous single/four-tenant results do not establish active-tenant density.

The full issue describes A1-A5, B1-B4, E0-E4, all-active versus sparse-active cases, below/above-memory cases, service objectives, fair resource sharing, and recovery gates. Keep measured facts, code-confirmed structure, and hypotheses distinct.

## Execution and preservation rules

- User-facing conversation is Korean. Repository artifacts, issues, and commit messages are English.
- Do not add source comments, single-letter abstract variable names, lint suppressions, or Co-Authored-By trailers.
- Preserve synchronous WAL, acknowledged durability, conditional/multi-key transactions, revisions/tombstones, ordered consistent Query/Scan, cursor and response limits, checksums/corruption detection, and safe reuse/recovery.
- Compare with contract-equivalent minimal-native TransactionDB; raw WriteBatch is not equivalent.
- Stage only task-owned files. Complete authorized repository changes through commit/push on this branch. Do not merge, force-push, or deploy as part of this foundation.
- Performance evidence must use the designated OCI host `opc@217.142.246.204` and ZFS `/bench/zfs/db`, with `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes`, and normal host-key validation. Recheck live capacity and existing artifacts before any future run. Do not substitute local or Debian performance evidence.
- Preserve failed logs/databases and prior evidence. Do not rerun passed cells merely to repair aggregate reporting.
- Host capacity, proposed SLOs, tenant counts, dataset sizes, and expected improvements in PLAN.md are planning inputs, not measurements or adoption decisions.

## Completion boundary for this issue

The issue remains open because the A1 and harness increments do not complete the density comparison or structural objective. Use the checklist in PLAN.md and the acceptance status in A1-RESULTS.md and DENSITY-HARNESS-RESULTS.md without rewriting failed or historical evidence. Do not promote defaults or close the issue based on local overwrite/recovery diagnostics or tiny harness smokes.
