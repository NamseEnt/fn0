# Performance loss from the fastest contract-equivalent path

## Scope

Explain the loss from the earlier fastest B-link path, not merely the small synchronous/background difference. The historical same-source checkpoint-off comparison measured 5,203 API calls/s in write-heavy A and 2,853 width-16 batches/s in E. Enabling 128 MiB reclamation reduced these to 3,968 and 1,362. Read-heavy RW80 is also relevant after the latest user-requested measurement.

Use the accepted frozen implementation `d240416f0e0efafa7c08bb1e868387ebd0d894bc` and its existing two executables. Do not change engine code, defaults, durability, invariant validation, transaction semantics, group limits, admission, or revision allocation. Keep the existing OCI host, ZFS configuration, and resource guards. Write a separate report and preserve every earlier successful/failed result.

## Evidence and hypotheses before new measurements

Historical WAL byte conservation gives appended bytes = ending retained bytes - starting retained bytes + reclaimed bytes. In E, this was approximately 8,828 bytes/transaction with checkpointing off and 22,836 at 128 MiB, a 2.59x increase; group sizes stayed near 53 transactions. A increased from approximately 3,347 to 5,249 bytes/transaction. These are derived from existing original reports, not new performance runs.

`WalLog::reset` clears `page_chain`. `plan_redo` then emits a full page image on the first touch, rather than a delta. Segment rotation also starts with an empty chain. This is required by the current recovery protocol; deleting the reset is not an authorized or safe optimization. Checkpointing therefore adds database writes and pauses while repeatedly reintroducing full-page WAL records. Quantify this amplification and its relation to the observed loss.

## New compact matrix

Measure RW80 (16 dedicated callers) and E (64 dedicated callers, width 16). Compare minimal-native RocksDB, B-link legacy checkpoint-off, legacy synchronous 128 MiB, and legacy synchronous 512 MiB. Each retains the preceding workload, seed, 512-byte values, synchronous WAL, conditions/revisions, and validators. Use three 60-second measurements after ten-second warmup, rotating path order: 24 cases, approximately 30 minutes. The longer window is selected before execution so read-heavy 512 MiB cells can actually pay reclamation costs instead of winning by postponing the first checkpoint. A separate preflight covers off/512 MiB in both workloads for eight seconds after one-second warmup. Reuse the existing frozen binaries and verify their source/hash before starting.

The 512 MiB option is an explicit finite-WAL policy candidate, not a default change or an equal-resource optimization. It allows four times as much WAL before reclamation. Report RSS/WAL/database use alongside any throughput gain. Require at least one completed checkpoint in each non-preflight threshold case, and report actual counts. A short cell with one checkpoint cannot establish steady-state performance. If the candidate needs longer confirmation to distinguish repeated cleanup from deferred cost, run paired 128/512 MiB confirmation at 120-second windows, with the decision and workload chosen from this compact matrix and recorded before execution.

Require exact latency sample counts/histogram sums, matching fixed input prefixes within each workload, full windows, zero unintended errors/overloads, final/reopen validation, and preserved limits: 3 GiB process-group RSS, 4 GiB database, 1500 MiB available memory, 10 GiB free space, 150-second short-cell timeout. Preserve failed attempts and their databases. Accepted cells are never rerun merely to correct reporting.

## Diagnosis and outcome

Separate bytes written to WAL from retained WAL size. Report observed group size, checkpoint time/count, appended WAL bytes per successful transaction, and throughput/latency/resources. Distinguish direct accounting from causal claims. If needed, collect separately labeled syscall/CPU profiles of checkpoint-off versus 128 MiB; profiler runs do not enter performance medians.

Explain the code path and measured costs in plain language. Identify improvements supported by results and distinguish an already available policy change from any unimplemented recovery-format or incremental-checkpoint design. Do not promise that B-link must beat RocksDB, or conclude that it cannot, from these measurements.

## Diagnostic follow-up record

The 24 uninstrumented cells completed. Both first attached-profile workloads passed their outcome and final/reopen gates, but the 128 MiB syscall trace captured fewer WAL bytes than the independently reconstructed writes. Absence of lost-event warnings did not establish complete process coverage. Preserve that failed attribution and all accepted workloads. One additional separately rooted E/128 MiB diagnostic launches the benchmark under inherited syscall tracing, executing the workload as opc, rather than attaching after process start. Accept its timing attribution only if syscall WAL bytes exactly match independently reconstructed physical WAL writes. This is diagnostic follow-up, not a rerun or replacement of an accepted headline cell.

Legacy reclamation records net bytes removed (before minus after). Exact physical append accounting adds the replacement INIT frame bytes remaining after each reset, normally 104 bytes per checkpoint. This reporting correction is negligible for the original rounded ratios and requires no workload rerun.
