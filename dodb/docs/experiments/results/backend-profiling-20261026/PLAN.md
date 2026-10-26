# B-link and RocksDB CPU and wait profiling

## Scope

Diagnostic profiling only. Preserve engine code, transaction/revision semantics, synchronous WAL, admission, grouping, checkpoint policy, workload drivers and defaults. Reuse the frozen `cc80d1fdc7a98792be1a739056a3013debf39268` source and original hashed executables from `/bench/zfs/db/blink-checkpoint-costs-20261023` on `opc@217.142.246.204`, ZFS `/bench/zfs/db`. Existing evidence and databases remain untouched.

Compare candidate B-link against the direct DODB-contract RocksDB `minimal-native` backend. RW80 retains 16 dedicated callers, the 80:20 API mixture, conditions/revisions, widths 1/4/8/16 and 512-byte values. E retains 64 dedicated callers and unconditional width-16 overwrite batches. Seed is 20261013. B-link retains background coalesced immutable bases, the 128 MiB WAL trigger and 32 MiB segments.

## Collection

Run a short profiled preflight for each workload/backend, then eight diagnostic cells: two workloads, two backends, one unprofiled control and one profiled execution. Each main cell has ten seconds of warmup and sixty seconds of admission. Alternate backend order between controls and profiles. Expected workload time is roughly twelve minutes plus setup, validation and profile reduction. This is a diagnostic comparison, not a new sustained performance claim or a default-promotion experiment.

CPU collection uses the original release binaries with existing DWARF, `cpu-clock` at 99 Hz, 16 KiB user stack capture and realtime timestamps. Attach to a single-thread gate before it executes the workload; release the gate only after collectors are ready. All workload-created threads inherit recording. Preserve full-lifetime data and reduce the exact declared measurement window separately from setup and post-window validation/reopen. Partition B-link measurement samples by checkpoint-event overlap; these intervals include worker execution and concurrent foreground work and are not isolated checkpoint CPU.

Wait collection uses BPF kernel stacks at scheduler switch-out and syscall elapsed-time aggregation for fsync, fdatasync, pwrite64, pread64 and futex. Filter by workload TGID before exec. Aggregate per thread and monotonic second; use only full one-second start buckets inside the declared measurement window for phase summaries. Wait totals sum across threads and must not be interpreted as request latency. Syscall elapsed time includes execution and waiting. Exclude idle-thread parking from conclusions about storage bottlenecks. No allocation/retained-heap attribution is claimed.

Reduction audit: this kernel encodes preemption as tracepoint state `R+` (256), so the collector's raw `runnable` field (`prev_state == 0`) cannot distinguish all runnable intervals from sleeping. Pool all off-CPU intervals in derived flamegraphs and use kernel call paths plus syscall timings for interpretation; do not publish a runnable-versus-sleeping total. Full start-second selection does not guarantee that a long parked interval finishes before the measurement boundary. Preserve and flag this boundary limitation; do not interpret idle parking totals as request latency. The CPU and syscall data are unaffected by this aggregation correction, so no accepted workload is rerun for it.

Compare profiled throughput and latency against the same-session unprofiled control and report perturbation. Do not use profiled throughput as an engine ranking. If collection fails, preserve the failed attempt and retry only the missing diagnostic cell with an explicit reason. Expand only when an observed anomaly prevents interpretation.

## Acceptance and preservation

Record host/filesystem/tool versions, source status/commit, executable hashes, command lines, collector hashes and workload PID. Preserve raw reports/stdout/stderr, external resource samples, raw CPU data, BPF aggregates, folded stacks, SVG flamegraphs, reduction code and SHA256 inventory. Validate source/build identity, zero request errors, complete latency counts, fixed input-prefix equality, full admission, final and reopen state, checkpoint drain and acknowledged-write overlap. Main B-link cells need at least three completed in-window checkpoints. Retain the existing 3 GiB workload process-group RSS, 4 GiB database, 1.5 GiB host memory reserve and 10 GiB filesystem reserve guards. Profile files have a separate 2 GiB per-cell ceiling. Keep all task databases on the host, subject to the same host free-space guard.

Publish an English report and reproducible artifacts in the dedicated profiling branch, commit and push task-owned files only, and answer in Korean. Do not optimize engine code during this task.
