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

The branch starts at reviewed source `a7271e9e35c5e0fa9d2a1c0e5eedc030f12de497`, followed by the documentation-only preparation commit. It intentionally retains the backend-hotpaths experiment implementation and evidence rather than substituting the production main branch.

Do not use or modify `/Users/namse/dodb`. Keep the existing main checkout, source experiment worktree, other worktrees, databases, and archived/rejected evidence intact. The local common Git exclude file ignores `/.worktrees/` so this nested worktree does not create untracked noise in main. Do not remove other exclude entries.

## Preparation completed

- Rechecked the original worktree, branch, reviewed commit, clean state, and remote source SHA.
- Created this new branch and linked worktree under fn0.
- Published the complete analysis and plan in English as issue #126.
- Added the English plan and this handoff.
- No engine/harness implementation or new benchmark was started.
- No defaults, production deployment, main history, or previous evidence changed.

The preparation commit contains documentation only. Read the current Git log and tracking branch for its exact hash; do not confuse it with the frozen source SHA.

## First work after a future implementation request

Start with A1 in PLAN.md: measure and verify writer-owned generation pin lifetime versus page reuse in the planned service write path.

At the reviewed source, `apply_planned_transaction_group` pins `committed_view` for admission and retains it when `can_reuse_pages` checks for zero pins. The service enables planned execution. Values above 512 bytes use overflow allocation, so fixed-key overwrite is the narrow first diagnostic. This is a static source finding; no new growth measurement or performance attribution has been made.

1. Recheck the source around admission, reuse decisions, and allocation/free before editing.
2. Define task-owned accounting for allocations, free pages, reuse attempts/results, pin count/lifetime, high-water, and bytes by representation.
3. Preserve the implementation plan and acceptance criteria before implementation.
4. Validate no-external-reader and pinned-reader cases with 512/513-byte and 4 KiB values.
5. Preserve the page-reuse safety proof and real-service recovery sequence: incomplete transaction recovery -> write succeeds -> another recovery.
6. Keep any experimental behavior opt-in/default-disabled. Do not promote settings or run the broader density matrix based on one small diagnostic.

Then prepare separated backend-memory accounting, the configurable density harness, and the shared-cache/WBM minimal-native reference. The current memory diagnostic worker cannot measure segmented checkpoints unchanged. Current service_workload settings are not already a general many-tenant offered-load harness.

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

The issue remains open because implementation and density/recovery evidence are future work. Publishing the plan and preparing this worktree does not complete the engineering objective. Use the checklist in PLAN.md to record subsequent work without rewriting failed or historical evidence.
