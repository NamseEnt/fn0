# GPT-6 Luna execution runbook for B1/B2

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: future implementation instructions. Writing this runbook does not start implementation or experiments. The complete target, architecture, dependency order, task ownership, and acceptance matrix are in [B1-B2-IMPLEMENTATION-PLAN.md](B1-B2-IMPLEMENTATION-PLAN.md).

## Copyable implementation assignment

Use the following assignment when implementation is requested:

> Implement issue #126 B1/B2 using B1-B2-IMPLEMENTATION-PLAN.md and this runbook, with GPT-6 Luna at high reasoning effort. Work only in /Users/namse/fn0/.worktrees/dodb-tenant-density-20261027 on experiment/blink-tenant-density-20261027. Read the current files and receipts; start at the first incomplete task whose prerequisites have passed. Execute tasks sequentially through T16. Complete one reviewable task at a time, validate it, record exact evidence, commit only task-owned files, and push without merge or force-push. Preserve all unrelated edits and failed artifacts. Do not stop after foundations and call B1/B2 complete. Do not launch T17 performance experiments without an explicitly authorized frozen matrix. Keep defaults unchanged. Do not add comments, single-letter abstract names, lint suppressions, or Co-Authored-By trailers. Repository artifacts and GitHub content must be English; user-facing updates must be Korean. If architecture fails a stated invariant, preserve the counterexample and amend the contract before implementing a different protocol. No parallel agents are necessary. Report concise outcomes and remaining gates, with links to detailed receipts.

## Start or resume

Run from the dedicated worktree:

```sh
pwd
git rev-parse --show-toplevel
git branch --show-current
git status --short --branch
git rev-parse HEAD
git remote get-url origin
git ls-remote origin refs/heads/experiment/blink-tenant-density-20261027
```

Expected root/branch are in the assignment. The planning baseline was `7e37551314b3b2ae14c019e9141c546fa729b914`; later plan/implementation commits are expected. Never reset to that SHA or discard other work to recreate the baseline. If unrelated changes exist, list them and stage only owned paths. If a required path conflicts with live unrelated edits, preserve them and resolve scope before overwriting.

Read `AGENTS.md`, the master plan, the current `B1-B2-CONTRACTS.md` once it exists, the previous task receipt, and the source symbols named by the next card. Historical handoff paragraphs that describe older worker assignments are evidence history, not instructions to repeat completed work.

Determine completion from code, exact tests, receipts, and remote commits together. A checked box or `accepted: true` alone is insufficient. Do not rerun already accepted remote cells for a reporting-only correction. Never claim new implementation performance using old source/binary evidence.

## Sequential order and scope

```text
T00 contracts
T01 shared resource ledger
T02 WAL locations and reconstruction
T03 disk-backed version index
T04 paged manifest and recovery index
T05 root/source retention and GC
T06 fallible page interfaces
T07 cache and coalesced loads
T08 read route
T09 atomic writer
T10 streaming checkpoints
T11 full validation and storage crash matrix
T12 service integration
T13 foreground byte admission
T14 fair maintenance scheduling
T15 real-service crash/write/crash and resource acceptance
T16 harness and experiment-ready handoff
T17 separately authorized comparison and decision
```

The master plan gives each task its dependencies, owned modules, and acceptance tests. Do not skip later tasks because a tiny correctness smoke passes. In particular, leaving `BlinkState.pages`, a full `base_owners` map, an all-page WAL recovery map, a high-water-sized catalog, or whole-store checkpoint image materialization on the paged route fails the target.

T00 is a separate specification commit. For subsequent tasks, add only interfaces needed by the next cards; avoid broad stylistic refactoring. A task can be split into smaller commits with a shared receipt. Do not expose the new runtime route before its source-lifetime/recovery foundations pass.

## Task receipt and preserved evidence

Create `evidence/b1-b2/Txx/receipt.json` and `RESULTS.md` for each task. Use unique subdirectories for repeated development attempts rather than overwriting failures. Keep large regenerable builds outside Git; retain necessary frozen sources/binaries/artifacts in the established evidence location with explicit hashes and retrieval paths. Never commit credentials or raw environment dumps.

Required receipt fields:

| Field | Required contents |
| --- | --- |
| `task_id`, `scope`, `status` | Exact task and accepted/partial/failed outcome |
| `parent_source_commit`, `implementation_commit` | Actual runtime source boundary; a later evidence commit is not a new binary |
| `prerequisite_receipts` | Paths/hashes and inspected acceptance of dependencies |
| `owned_paths`, `source_hashes` | Actual final changed source and test files |
| `host`, `toolchain`, `features`, `runtime_configuration` | Explicit route/layout/limits; distinguish local correctness and OCI |
| `checks` | Argument arrays, cwd, timestamps, exit status, raw log path/hash, exact test names/counts and ignored tests |
| `acceptance_mapping` | Every task criterion -> named test/artifact -> observed result |
| `ownership_peaks` | Limits, live categories/peak overlap, uncharged boundaries; unavailable values explicitly unavailable |
| `failed_attempts` | Seed/configuration, database/artifact paths/hashes, failure, correction, preserved raw records |
| `remaining_work`, `next_task` | Unmet criteria and first safe next step |

Run commands with output captured to a task-specific log and preserve the real exit code. Write structured JSON using a script rather than hand-editing supposed results. Test counts do not substitute for checking which runtime path the tests exercised. Tests using internal helpers must not be described as actual-service coverage.

After a failing crash/corruption test, preserve its database before temporary-directory teardown. Capture the actual original failure output; do not replace it with a transcribed success or a later corrected receipt. A corrected reducer may derive new evidence from unchanged raw artifacts without a new workload execution.

After validation, stage explicit owned files, inspect the staged diff for comments/non-English content/lint suppression/coauthor trailers and unrelated artifacts, commit with an English message, and push the current branch normally. Do not use `git add .`, amend other people's commits, merge, force-push, or deploy. Keep an implementation-source commit distinct from evidence written about that commit; record final source hashes when an earlier working-tree test preceded the commit.

## Commands and test coverage

Existing focused baseline checks are runnable now:

```sh
cargo test -p dodb-storage --lib
cargo test -p dodb-storage --features blink-checkpoint-locators --lib
cargo test -p dodb-storage --features blink-checkpoint-locators --test durable_base_checkpoint
cargo test -p dodb-storage --features blink-checkpoint-locators --test durable_base_sigkill --test durable_base_background_sigkill --test segmented_checkpoint_sigkill --test segmented_wal_sigkill
cargo test -p dodb-server --test blink_writer_pin_reuse
```

Confirm applicable baseline before shared scanner/publication/recovery changes. Preserve baseline failures rather than silently changing unrelated code. These commands verify existing behavior and the B1.0 primitive; they cannot prove the future paging route.

When T00 defines and later tasks add `blink-demand-paging`, use these feature combinations. They are planned commands and must not be reported as executed before the feature exists:

```sh
cargo test -p dodb-storage --features blink-demand-paging --lib
cargo test -p dodb-storage --features blink-demand-paging
cargo test -p dodb-storage --features blink-demand-paging,blink-writer-pin-reuse
cargo test -p dodb-server --features dodb-storage/blink-demand-paging
cargo test -p dodb-server --features dodb-storage/blink-demand-paging,dodb-storage/blink-writer-pin-reuse
```

Add named integration tests for version index, paging reads/writes, source retention, bounded resources, and service crash recovery. Select focused tests during development, then run the affected full suites at each activation/recovery milestone. Keep four existing ignored manual library probes distinct from newly required tests; no new required case may be ignored. Large-resource tests use deterministic small bounds and isolated scratch files, not an accidental local benchmark.

Run targeted formatting on actual touched Rust paths, for example:

```sh
rustfmt --check --edition 2024 --config skip_children=true dodb/crates/dodb-storage/src/blink/page_locator.rs
cargo clippy -p dodb-storage --lib -- -D warnings
cargo clippy -p dodb-storage --features blink-demand-paging --lib -- -D warnings
cargo clippy -p dodb-server --features dodb-storage/blink-demand-paging --lib -- -D warnings
```

The feature commands become valid only after the feature is added. Replace the formatting example with every touched Rust file. Whole-workspace formatting has existing vendor workspace metadata limitations; do not misreport a skipped workspace check as passed. B1.0 observed the same twelve strict storage-library Clippy findings in default/locator builds. Reproduce the actual current baseline and distinguish new findings; do not assume the count remains twelve. Fix new findings without suppressions. Include server/harness strict baseline comparisons when those packages change.

At T16, verify the actual `dodb-rocksdb` harness entry point and supporting modules from Cargo metadata and source before selecting build/test commands. Preserve default and candidate binary hashes separately. Existing RocksDB native build provenance and linkage must remain pinned; do not rebuild or replace reference native artifacts casually.

## Test design rules

- Use a small reference map/oracle with deterministic seeds for logical equivalence. Include conflicting and rejected transactions separately from successful commits.
- Force cache eviction through a deterministic capacity/test control, assert nonzero misses and exact source reconstruction, and verify the test did not fall back to resident reads.
- Use barriers/markers for publication, loader/GC, checkpoint, and cancellation races. Timing sleeps alone cannot prove that a fault reached its intended boundary.
- Test byte bounds at live-owner overlap, not just after everything is dropped. Include guard, queue, preparation, checkpoint, loader, metadata, and scratch ownership.
- Test non-progress using owner/waiter state and bounded completion after capacity release. Do not fix deadlocks by enlarging timeouts.
- Corrupt unrelated pages to prove full validation. A test that touches only the requested page cannot prove unchanged corruption coverage.
- For crash children, validate both process status and barrier markers. SIGKILL without the expected marker is an invalid attempt, not a successful durability test.
- After an interrupted command, inspect its real session/process identity before retry. Retain partial logs, child processes, and databases until their state is known.

## Future OCI execution boundary

Performance execution is T17, after an explicitly authorized frozen matrix. Use only `opc@217.142.246.204`, key `/Users/namse/.ssh/id_late_sh_ed25519`, `IdentitiesOnly=yes`, normal host-key verification, and ZFS `/bench/zfs/db`. Do not substitute a local Mac, Debian host, or another machine for capacity/performance evidence.

Before any new cells, inspect live RAM, MemAvailable, ARC, CPU load, filesystem allocation/free space, existing sessions/processes/cgroups, and preserved artifacts. Reserve at least 10 GiB free on ZFS after worst-case overlap and failed-cell preservation. Preserve source/reports/logs/databases/frozen binaries/pinned native artifacts; remove only explicitly identified regenerable intermediates when separately appropriate.

Charge backend setup/measurement/oracle/reopen consistently using the existing common cgroup policy. Keep driver/runtime/OS/ARC boundaries explicit. Record PID and start time, actual limits/membership, exact commands/configuration, source/native/binary hashes, input/schedule fingerprints, full and reopen oracle, failures, and terminal process/cgroup state.

A reducer checks operational completeness, SLO, and acceptance independently. Reporting failure is repaired over raw evidence; an accepted cell is not rerun for aggregation. A failed cell stops the cohort according to its frozen rule. Never manufacture a maximum density claim from a bounded population or a plateau from a finite rising trace.

## Final response and handoff

Use concise Korean progress/final messages, with English artifacts linked. State completed task IDs, implementation/evidence commits, checks and material failures, actual remaining requirements, and whether the route is enabled anywhere. Do not claim B1/B2 complete until the master acceptance matrix and T00-T16 are satisfied. Do not claim adoption or physical disk superiority without the separate comparative gates.

If work stops between tasks, update the task receipt and handoff with the exact next command or missing criterion. Do not shrink the objective or promote defaults to obtain a superficially finished result.
