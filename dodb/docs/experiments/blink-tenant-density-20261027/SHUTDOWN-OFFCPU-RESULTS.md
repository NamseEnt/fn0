# External shutdown wait observation

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: the frozen three-attempt sequence stopped on its third setup-worker watchdog failure. Eighteen individually verified snapshots locate one Tokio-named thread in the memory.high kernel wait path. The sequence remains stopped and the failed cell invalid; its application's allocating call site is not established.

The primary owns design and evidence judgment. GPT-6 Luna at high reasoning effort implemented the five new helper/test files and supplied concise summaries. The existing b226 Rust source, seventy-nine accepted checks, default/A1 compatibility links, engine settings, budgets, and both shutdown deadlines are reused unchanged.

[SHUTDOWN-OFFCPU-PLAN.md](SHUTDOWN-OFFCPU-PLAN.md) was preserved before implementation. [SHUTDOWN-OFFCPU-MATRIX.md](SHUTDOWN-OFFCPU-MATRIX.md) fixes the external observer's conditions, limits, and stop rule. The local checks cover partial trace lines, exact identity and owned leaf groups, fast normal shutdown, status context-switch fields beyond 1 KiB, capture/output limits, and read errors. Additional implementation review exercised controller success/stop and collection/reduction with the actual earlier failed-report shape, without inventing runtime results.

The initial OCI owned-child probe confirmed readable wchan, syscall, and stack fields and verified child termination. Its raw output and executed prototype are preserved in `evidence/shutdown-offcpu/probe-initial/`. The final observer reads selected status fields up to 4 KiB instead of the initial prototype's truncated 1 KiB. Its fresh final-implementation probe also passed and is preserved in the runtime namespace.

The first controller launch stopped in preflight because it incorrectly treated the harness directory as a Git checkout for the plan commit. No run namespace, cgroup, database, worker, or performance cell was created. Its actual controller, launcher log, and PID receipt are preserved under `evidence/shutdown-offcpu/preflight-r1/`. The corrected controller verifies the clean b226 source checkout and records the separately frozen controller/plan commit; a non-Git harness plus Git source fixture passed. The next launch has a separate receipt and does not rerun a previously executed benchmark cell.

## Executed cells and accepted counters

Namespace `shutdown-offcpu-b22638bdc-r1` used source b226, unchanged frozen default-feature binaries, controller/plan commit `1f7a5a02a0709fb40f362955a304af26e615b723`, and the original memory/durability/timeout/workload settings. It ran B-link only. No reference result or performance ranking is invented.

| Repetition | Outcome | Successful offered calls | Measurement peak | Parent high events |
| --- | --- | --- | --- | --- |
| 1 | Completed; SLO and both oracles passed | 15,000 | 184.488 MiB | 1,242 |
| 2 | Completed; SLO and both oracles passed | 15,000 | 185.559 MiB | 1,081 |
| 3 | Setup parent watchdog; offered load never started | Unavailable | Unused epoch | 3,176 |

Complete reports account for 30,000 successful calls and four full-population oracles. The third cell took 70.092 seconds and ended with `diagnostic worker shutdown timed out and was killed`. Parent/setup peaks were 222,236,672 / 222,195,712 bytes; setup-local high events were 1,588. Max, OOM, and kill events were zero. Later epochs were unused. There is a preserved failure receipt and no completion receipt. Both earlier completions also had high events; those counters alone do not identify a watchdog failure.

## Directly observed wait

The failed process identity is `2461833:258566584`. After `final_property_capture_end`, its trace entered `async_server_shutdown_start` and emitted no end/error marker before the parent watchdog. The interval still contains connection/service shutdown and the server-task join; phase tracing does not distinguish those awaits.

Captures 1 through 18, from +2.039 to +19.109 seconds relative to that marker, verified PID/start-time/source and the owned setup leaf cgroup both before and after reading. Every capture recorded TID/start ticks `2461850:258566869`, named `tokio-rt-worker`, in state `D` with both wchan and stack identifying `mem_cgroup_handle_over_high`. Its recorded kernel stack includes `do_notify_resume` and `el0_da`; the syscall field reports `-1`. Other sampled threads included futex, epoll, and stdin-pipe waits. No userspace call chain was collected, so the thread name does not identify a particular coordinator task or allocation object.

The nearest driver pressure samples across that window show memory.current 213,061,632 to 221,671,424 bytes, anonymous memory 211,914,752 to 220,770,304 bytes, and file memory one 4 KiB page to zero. CPU advanced 50,189 microseconds over the aligned 17.010-second sample interval. Local high events rose 907 to 1,554; direct scanned pages rose 146 to 215 while reclaimed pages rose zero to one. These non-atomic samples show pressure and little reclaim progress alongside the observed kernel wait, without exact representation-level heap attribution.

The [Linux cgroup v2 memory interface](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files) defines high as a reclaim/throttling boundary and max as the hard limit. High pressure can delay a process without an OOM kill. [Upstream Linux v6.12 memcontrol](https://github.com/torvalds/linux/blob/v6.12/mm/memcontrol.c#L1939) implements reclaim and delay in the observed function. The running host is the recorded UEK build, so its actual snapshots, not an assumption of byte-identical upstream source, establish the sampled wait.

The evidence establishes a sustained sampled memory.high wait contribution in this failure. It does not prove its unique allocating call site, explain the earlier unprofiled source-023 measurement failure, or establish that every shutdown delay has this cause. Keep both deadlines unchanged. The external cgroup limit does not implement eviction or application byte admission for the fully resident tree.

## Partial evidence and observation cost

Capture 19 occurred at +20.111 seconds after the process disappeared and recorded FileNotFound. Its identity is invalid and it contains no accepted thread sample. The raw observer retains error count one, and complete observation/cell validity remains false. The reducer separately records eighteen valid captures, one unavailable-after-exit capture, and `usable_wait_evidence=true`; it does not turn the failure into a passing cell.

Valid captures observed all available threads below the thirty-two-thread cap. Their proc fields were readable without truncation or unsupported states. Capture-read durations ranged 1.703 to 2.591 ms and totaled 36.129 ms. This covers the capture/read section only; polling, JSON serialization/writes, and host contention are additional costs. Sampling is intermittent and does not prove continuous exclusive blocking or an unperturbed throughput ranking.

## Preservation and reproducible reduction

The complete raw archive is 13,824,403 bytes with SHA256 `9ae14b70c4d61612b6ef3af6f96ee1103346cf4a5a0d7aaf39b42f2c222e8ece`. It includes the actual frozen binaries. Collection verified three retained databases, twelve exited process identities, sixteen empty groups, native/binary hashes, the clean source checkout, and reused seventy-nine-check/default-link provenance. Only exact extracted duplicate binary paths are Git-ignored; binaries remain in the archive, local extraction, and OCI.

The collection implementation was corrected after workload execution to distinguish the non-budget grouping root from budgeted cell/epoch groups. The grouping root's actual max/high/swap values remain recorded, while cell/epoch limits verify 256/192 MiB and zero swap. The original planned collector and reviewer are preserved under `planned-helpers/`; actual collector/reviewer hashes are recorded separately. No completed workload was rerun for these reduction corrections.

A local verification initially inspected the still-running transfer and rejected its incomplete archive. After SCP completed, the full archive matched collection SHA256 and the partial extraction was replaced from that verified archive. `collection/transfer-status.json` preserves this distinction.

Run from the worktree:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 dodb/docs/experiments/blink-tenant-density-20261027/evidence/shutdown-offcpu/review_shutdown_offcpu.py --output dodb/docs/experiments/blink-tenant-density-20261027/evidence/shutdown-offcpu/final-review.json
```

Reduction verifies the archive and transfer, case-file hashes, source/helper/binary/native identities, accepted validation metadata, completed accounting/oracles through the immutable capacity reducer, the exact failed report, and per-capture ownership. Identical reduction reuses the matching derived output; differing output is rejected. All earlier failure records and omitted cells remain preserved. Issue #126 remains open.
