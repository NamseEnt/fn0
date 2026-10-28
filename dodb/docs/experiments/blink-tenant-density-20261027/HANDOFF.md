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

The isolated OCI build root is `/home/opc/tenant-density-harness-20261027`; build output is on the boot filesystem. Preserve source probe archives, failed logs, databases, and previously accepted cells. Real smoke databases are under `/bench/zfs/db/tenant-density-harness-20261027`. The benchmark filesystem had only about 0.39 GiB above the required 10 GiB free reserve during preparation, so this increment permits only tiny correctness/accounting smokes. Recheck live capacity before future work.

The next separate implementation scope is the shared-cache/WBM minimal-native reference. Production-wide pin ages, allocator/representation retained bytes, queue/snapshot byte budgets, and enforced common memory pressure remain absent. The density/maintenance matrix and adoption decision are not completed by making the harness executable.

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
