# T15 Crash/Write/Crash Progress

Status: partial. This receipt records the first real-service crash/write/crash path and the recovery defect it exposed. T15 is not complete.

## Implementation

The service subprocess test now seeds acknowledged paged data, checkpoints it, writes a WAL-backed value, and pauses a conditional two-key transaction before its commit record. The parent verifies the barrier marker and child PID, sends SIGKILL, reopens through `BlinkTenantService`, and checks acknowledged values and revisions. It then acknowledges a conditional multi-key write, starts a second incomplete transaction, kills that child, and verifies the acknowledged values and revisions again after a second recovery. The test runs both `coalesce=false` and `coalesce=true`.

The first service recovery attempt found a stale-root failure: a committed WAL superblock advanced the high-water page ID beyond the manifest checkpoint, but recovery validated pages using only the manifest root metadata. `PagedRecoveryIndexBuilder` now replays committed superblock metadata, validates its database/shard identity, tracks root/free-list/high-water/generation/active slot through transaction rollback, and passes recovered metadata into the read and writer handles. A service-open writer resumes from that recovered superblock state.

The T14 old-reader service test was also made non-flaky by separating source-retention acceptance from capacity-wait acceptance. It retains a reader across overwrite and a second checkpoint. Capacity-wait resumption remains covered by the separate `paged_checkpoint_resumes_after_disk_capacity_release` test. Two rejected same-fixture headroom probes remain preserved in T14 logs.

A barrier-defined service Query test captures one paged snapshot, commits a conditional two-key update while the Query is paused, then verifies that the paused Query returns both old values and the next Query returns both new values at the same revision. The crash test also rereads all 16 checkpointed dataset keys after recovery and asserts the durable-source read count increased; this is source-reload evidence, not an eviction counter.

## Validation

| Check | Result | Evidence |
| --- | --- | --- |
| Paging server library suite, serial, final source | 52 passed, 0 failed | `T15/logs/server-paging-library-source-reads-final.log` |
| Default server library suite, serial | 26 passed, 0 failed | `T15/logs/server-default-library-final.log` |
| Storage library suite with paging, serial | 324 passed, 0 failed, 4 ignored | `T15/logs/storage-paging-library-final.log` |
| Crash/write/crash subprocess matrix, A1 disabled | 1 passed; both durable-base output modes and source-reload assertion exercised in final focused run | `T15/logs/paged-crash-a1-off-source-reads-final.log` |
| Crash/write/crash subprocess matrix, A1 enabled | 1 passed; both durable-base output modes and source-reload assertion exercised | `T15/logs/paged-crash-a1-on-source-reads-final.log` |
| Concurrent Query snapshot consistency | 1 passed; old snapshot returned the complete prior pair while the next Query returned the complete committed pair | `T15/logs/server-query-consistency-final.log` |
| Paging server strict Clippy, all targets | Passed after the Query and source-reload tests | `T15/logs/server-paging-clippy-source-reads-final.log` |
| Formatting and source whitespace | Passed after the final code edit with `cargo fmt -p dodb-server -p dodb-storage -- --check` and `git diff --check` | Raw test logs retain their original captured whitespace |

The local macOS runs establish correctness only. They do not establish performance, an SLO, maximum tenant count, physical-read behavior, or backend superiority.

Earlier accepted subprocess attempts with an absolute evidence path preserve their configuration, child markers/PIDs, manifests, and paged data under `T15/runs/`. Failed and incomplete probes remain under `T15/logs/`; no preserved run directory is being cleaned as part of this follow-up.

## Remaining T15 Gates

- Add old-operation-pin/checkpoint/reload coverage across service restart.
- Prove page-ID reuse across recovery and second crash at the service boundary.
- Exercise missing/corrupt reachable sources, overload and resource recovery, cancellation, and complete shutdown with bounded category peaks and no leaked tasks or charges.
- Record actual nonzero eviction, cache-miss, and WAL-reconstruction counters and full live category peaks for the crash scenario.
- Complete the A1-off/on and durable-base mode matrix for the full required acceptance set, not only this crash sequence.
- Preserve and receipt the failed databases and child markers with source/configuration hashes.

Do not proceed to T16 readiness or T17 performance execution until these gates and the remaining T15 acceptance mapping are closed.
