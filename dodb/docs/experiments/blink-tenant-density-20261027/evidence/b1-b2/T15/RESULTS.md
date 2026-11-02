# T15 Crash/Write/Crash Progress

Status: complete. The real-service crash/write/crash oracle, A1 and durable-base mode matrix, resource accounting, and shutdown/recovery acceptance are recorded below.

## Implementation

The service subprocess test now seeds acknowledged paged data, checkpoints it, writes a WAL-backed value, and pauses a conditional two-key transaction before its commit record. The parent verifies the barrier marker and child PID, sends SIGKILL, reopens through `BlinkTenantService`, and checks acknowledged values and revisions. It then acknowledges a conditional multi-key write, starts a second incomplete transaction, kills that child, and verifies the acknowledged values and revisions again after a second recovery. The test runs both `coalesce=false` and `coalesce=true`.

The first service recovery attempt found a stale-root failure: a committed WAL superblock advanced the high-water page ID beyond the manifest checkpoint, but recovery validated pages using only the manifest root metadata. `PagedRecoveryIndexBuilder` now replays committed superblock metadata, validates its database/shard identity, tracks root/free-list/high-water/generation/active slot through transaction rollback, and passes recovered metadata into the read and writer handles. A service-open writer resumes from that recovered superblock state.

The T14 old-reader service test was also made non-flaky by separating source-retention acceptance from capacity-wait acceptance. It retains a reader across overwrite and a second checkpoint. Capacity-wait resumption remains covered by the separate `paged_checkpoint_resumes_after_disk_capacity_release` test. Two rejected same-fixture headroom probes remain preserved in T14 logs.

A barrier-defined service Query test captures one paged snapshot, commits a conditional two-key update while the Query is paused, then verifies that the paused Query returns both old values and the next Query returns both new values at the same revision. The crash test also rereads all 16 checkpointed dataset keys after recovery and asserts the durable-source read count increased; this is source-reload evidence, not an eviction counter.

The service crash sequence now deletes the recovered dataset, checkpoints a persisted free-list root, reinserts the same keys, and checkpoints again before acknowledging the next multi-key transaction and taking the second crash. In the A1-on run, both `coalesce=false` and `coalesce=true` retained high-water page ID 40 across delete/reinsert (`40 -> 40`). The A1-off run observed the same result for this separate-transaction path; this does not establish an A1-on/off performance or reuse difference.

The service no longer keeps a permanent writable snapshot pin. A paged Get, Query, or Scan now takes a short writer snapshot under the writer mutex inside its blocking worker. A request already in progress retains its own snapshot across a concurrent commit. The barrier-defined Query test verifies this consistency. This adds a snapshot-lock cost that must be measured in a later matched experiment before any performance conclusion.

The service now rejects both a missing checkpoint pack and a checkpoint pack with a corrupted reachable page frame after shutdown and reopen. The test exercises `coalesce=false` and `coalesce=true`; each A1 configuration was run separately. The first corruption attempt changed the final pack byte, which was outside a reachable page frame and was accepted. That failed database is preserved under `T15/runs/reachable-corruption-accepted-first-attempt/` with its raw test log. The corrected test flips a byte inside the first page frame, verifies the reopened service returns an error rather than a value or absence, and separately verifies removal of the reachable pack is rejected.

The crash child now records loader and resource counters immediately before each SIGKILL, then the parent records recovery counters after reopening. In both A1 settings and both durable-base modes, the first child recorded 97 misses, 89 evictions, 57 WAL reconstructions, 32,768 cache-peak bytes, and zero active loads/followers; the second child recorded 140 misses, 132 evictions, 21 WAL reconstructions, the same cache bound, and zero active loads/followers. The successful recoveries recorded 149 misses, 141 evictions, 15 WAL reconstructions, and 149 source reads. These counters prove the requested sequence exercised eviction and WAL reconstruction at both crash boundaries; they are correctness/accounting evidence, not a performance result.

The paged service crash fixture now captures loader counters and peak resource accounting after reopening the first killed service. In both A1 settings and both coalesce modes, the run recorded 149 cache misses, 141 evictions, 15 WAL-sourced reconstructions, 149 source reads, a 32,768-byte decoded cache peak against its 32,768-byte limit, and zero active loads or followers at the observation point. The service resource manager reported a 923,614-byte peak total and these independent per-category peaks: decoded payload 32,768; metadata 808,704; load reconstruction 69,692; queued foreground 167; write preparation 131,072; dirty images 0; checkpoint validation 12,288. Disk peaks were 3,746,353 bytes total, with independent category peaks of WAL 358,916; checkpoint output 0; version index 3,227,648; scratch 1,920; retained source 159,748. Category peaks occur at different times, so their sum can exceed the total peak.

The first service reuse probe still grew high-water from 40 to 72 because the writable service kept a permanent `latest` snapshot. The service now obtains read snapshots from the writer inside the blocking worker, so idle service state does not pin the current generation. With A1 enabled, both durable-base output modes reuse pages after recovery without growing high-water. A1 disabled also reused pages in this separate-transaction case; this observation does not predict same-transaction overwrite churn. A first full-suite attempt caught the test gate placed before snapshot capture; the corrected gate now pauses only after the Query pins its root. Both failed probes remain in `T15/logs/`.

## Validation

| Check | Result | Evidence |
| --- | --- | --- |
| Paging server library suite, A1 disabled | 53 passed, 0 failed | `T15/logs/server-paging-library-final-matrix-a1-off.log` |
| Paging server library suite, A1 enabled | 53 passed, 0 failed | `T15/logs/server-paging-library-final-matrix-a1-on.log` |
| Default server library suite | 26 passed, 0 failed | `T15/logs/server-default-library-final-matrix.log` |
| Storage library suite with paging | 324 passed, 0 failed, 4 ignored | `T15/logs/storage-paging-library-loader-metrics-final.log` |
| Crash/write/crash, A1 disabled and enabled | Both passed; each covers both durable-base modes, two crash boundaries, complete reopen oracle, cache metrics, and resource peaks | `T15/logs/paged-crash-precrash-metrics-a1-off.log`, `T15/logs/paged-crash-precrash-metrics-a1-on.log`, `T15/logs/paged-crash-metrics-a1-off.log`, `T15/logs/paged-crash-metrics-a1-on.log` |
| Query snapshot consistency, A1 disabled and enabled | Passed across both durable-base modes | `T15/logs/server-query-matrix-a1-off.log`, `T15/logs/server-query-matrix-a1-on.log` |
| Old reader across overwrite, checkpoint, and reopen, A1 disabled and enabled | Passed across both durable-base modes | `T15/logs/server-old-reader-matrix-a1-off.log`, `T15/logs/server-old-reader-matrix-a1-on.log` |
| Checkpoint-debt overload and recovery, A1 disabled and enabled | Passed across both durable-base modes | `T15/logs/server-checkpoint-debt-matrix-a1-off.log`, `T15/logs/server-checkpoint-debt-matrix-a1-on.log` |
| Disk-capacity release/resume, A1 disabled and enabled | Passed across both durable-base modes; memory charge returned to zero after owners were dropped | `T15/logs/server-capacity-recovery-matrix-a1-off.log`, `T15/logs/server-capacity-recovery-matrix-a1-on.log` |
| Cancelled write and shutdown drain, A1 disabled and enabled | Passed across both durable-base modes | `T15/logs/server-cancelled-write-matrix-a1-off.log`, `T15/logs/server-cancelled-write-matrix-a1-on.log` |
| Shutdown while checkpoint waits for disk capacity, A1 disabled and enabled | Passed; shutdown completed and retained charges drained after references were released | `T15/logs/server-shutdown-cancel-a1-off-final.log`, `T15/logs/server-shutdown-cancel-a1-on-final.log` |
| Missing and corrupted reachable checkpoint sources, A1 disabled and enabled | Both rejected across both durable-base modes | `T15/logs/server-reachable-source-corruption-a1-off-retry-frame.log`, `T15/logs/server-reachable-source-corruption-a1-on.log` |
| Paging server strict Clippy, A1 disabled and enabled | Passed with `--no-deps` and `-D warnings` | `T15/logs/server-paging-clippy-final-matrix-a1-off.log`, `T15/logs/server-paging-clippy-final-matrix-a1-on.log` |
| Storage strict Clippy | Failed with 14 findings in existing storage code regions; no finding is in `page_cache.rs` or the new metrics accessor | `T15/logs/storage-paging-clippy-loader-metrics.log` |
| Formatting and source whitespace | Passed with `cargo fmt -p dodb-server -p dodb-storage -- --check` and `git diff --check` | Final commands run after the last source edit |

The local macOS runs establish correctness only. They do not establish performance, an SLO, maximum tenant count, physical-read behavior, or backend superiority.

Failed and incomplete attempts remain preserved in `T15/logs/` and `T15/runs/`; run configurations, child markers, databases, and hashes are indexed by the receipt and `T15/runs/SHA256SUMS`. No preserved run directory was cleaned as part of this follow-up.

## T15 Acceptance

All T15 correctness gates are complete: the real service used paging in the crash/write/crash path; both A1 settings and both durable-base modes passed; tests observed nonzero misses, evictions, WAL reconstructions, and source reads; cache and resource peaks were recorded; the full acknowledged state/revision oracle passed after each recovery; missing and corrupted reachable sources were rejected; and shutdown/recovery tests verified charge/task cleanup. Strict Clippy passed for the server targets with dependencies excluded. Storage strict Clippy still reports 14 pre-existing findings, recorded separately without lint suppressions.

T16 experiment-readiness work is next. T17 performance execution has not started; it requires the frozen T16 matrix and a reviewable capacity/evidence gate.
