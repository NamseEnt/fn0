# External shutdown wait observation

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: external observer implementation and eleven local contract checks passed; frozen runtime observation is pending. No kernel-wait attribution or completed sequence is claimed yet.

The primary owns design and evidence judgment. GPT-6 Luna at high reasoning effort implemented the five new helper/test files and supplied concise summaries. The existing b226 Rust source, seventy-nine accepted checks, default/A1 compatibility links, engine settings, budgets, and both shutdown deadlines are reused unchanged.

[SHUTDOWN-OFFCPU-PLAN.md](SHUTDOWN-OFFCPU-PLAN.md) was preserved before implementation. [SHUTDOWN-OFFCPU-MATRIX.md](SHUTDOWN-OFFCPU-MATRIX.md) fixes the external observer's conditions, limits, and stop rule. The local checks cover partial trace lines, exact identity and owned leaf groups, fast normal shutdown, status context-switch fields beyond 1 KiB, capture/output limits, and read errors. Additional implementation review exercised controller success/stop and collection/reduction with the actual earlier failed-report shape, without inventing runtime results.

The initial OCI owned-child probe confirmed readable wchan, syscall, and stack fields and verified child termination. Its raw output and executed prototype are preserved in `evidence/shutdown-offcpu/probe-initial/`. The final observer reads selected status fields up to 4 KiB instead of the initial prototype's truncated 1 KiB. A fresh final-implementation probe is required before the new sequence.

The first controller launch stopped in preflight because it incorrectly treated the harness directory as a Git checkout for the plan commit. No run namespace, cgroup, database, worker, or performance cell was created. Its actual controller, launcher log, and PID receipt are preserved under `evidence/shutdown-offcpu/preflight-r1/`. The corrected controller verifies the clean b226 source checkout and records the separately frozen controller/plan commit; a non-Git harness plus Git source fixture passed. The next launch has a separate receipt and does not rerun a previously executed benchmark cell.

Keep the earlier source-023 measurement failure and b226 setup failure distinct. A kernel wait snapshot can locate a sampled wait; it does not by itself identify the application's allocation site or explain a different unprofiled failure. Issue #126 remains open.
