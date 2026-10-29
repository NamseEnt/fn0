# Locate shutdown stalls without changing the resource policy

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

The primary owns this design and evidence judgment. All implementation is delegated to GPT-6 Luna with high reasoning effort; only concise implementation/validation summaries are returned.

## Trigger and measured boundary

The frozen raw-observation sequence stopped at `confirm-e3-n5-r3-blink` in `observations-023b18bd8-r1`. The first two B-link five-tenant confirmations passed, and all three shared-reference confirmations passed. This third B-link cell reached `offer-start` and `after-drain`, then its parent reported `diagnostic worker shutdown timed out and was killed`. No post-measurement or reopened oracle ran. Full workload counters were lost because the existing driver propagates shutdown errors before moving its locally completed workload result into the report.

This is the parent twenty-second stop watchdog in `memory_diagnostic.rs`, distinct from the worker's fifteen-second asynchronous server-shutdown timeout. Neither timeout changed in the raw-observation implementation. The failed worker did not emit the earlier fifteen-second error. The after-drain to after-worker-exit boundary interval was 20.039 seconds. Kernel high events were present; max/OOM/kill events were zero. These facts do not locate the stalled operation or prove that reclaim caused it.

Do not resume the old namespace, synthesize completion, retry the unchanged cell, relax either deadline, or run later capacity/maintenance cells before reviewing this failure. Preserve its report, raw status/review error, log, database, binaries, and cgroups.

## Approved implementation

Change diagnostic observability only. Engine/storage/service behavior, durability, locks, admission, caches/WBM, checkpoints, request recipe, schedules, budgets, deadlines, and host policy remain unchanged. Keep A1 disabled in accepted binaries.

1. Preserve a successfully returned `run_workload` JSON result in driver-owned failure state before issuing stop/shutdown commands. A later failure must report those counters as available but unvalidated, with no successful-cell or oracle claim. Keep the existing explicit unavailable state if workload execution never returned a result. Do not invent missing counters, acknowledged mutation state, or oracle outcomes. Success-report behavior remains the same. Record the extra driver retention outside the backend budget.
2. Emit bounded, low-frequency structured shutdown phase events to stderr only when existing opt-in pressure diagnostics are enabled. Include timestamp, PID/start-time identity, source revision, and stage. Preserve readiness stdout. Mark control-command receipt and distinguish final sampler/property capture, asynchronous server shutdown, checkpoint-metric capture, server/backend drop, health-report write, async-run return, and runtime destruction. A missing final marker is diagnostic evidence, not a reason to bypass the watchdog. Do not stream full periodic samples, introduce per-request logging, add a new background thread, or alter engine locks.
3. Keep diagnostic output small and explicitly fallible/best-effort without introducing a new hang or replacing the primary shutdown error. Follow existing repository constraints: English, no comments, no single-letter abstract names, no lint suppressions, no Co-Authored-By trailer.

The implementation worker owns the density driver, diagnostic worker, and narrowly scoped new support/tests. The existing `memory_diagnostic.rs` timeout and lifecycle protocol must remain unchanged unless the primary reviews a concrete necessity. Legacy capacity controllers/reducers and original runtime records are immutable.

## Checks and new experiment boundary

Test preservation of available versus unavailable failure counters, explicit unvalidated status, unchanged success shape, enabled/disabled trace behavior, identity/timestamp encoding, and trace write-error behavior where practical. Run affected tests, targeted formatting, advisory Clippy comparison without new findings/suppressions, and default/A1 compatibility links. Save a clean source SHA and build/native identities before OCI execution.

Freeze the exact runtime controller after reviewing the implementation and live host capacity. Start with a tiny paired correctness cell and inspect normal trace ordering, counters, both oracles, and empty groups. Then use up to three fresh paired E3 five-tenant sixty-second diagnostic repetitions with the unchanged 1,000 ms cadence and 256/192 MiB limits, alternating backend order with shared first in repetition one. This is a new instrumentation variant for shutdown attribution, not a retry to repair the prior report or obtain a passing median. Stop the sequence at the first unexpected operational failure and retain all completed and failed attempts. No unchanged failure is repeatedly provoked.

Report the last completed stage, the outstanding stage, and the parent watchdog separately. A stage interval identifies where time was spent, not its internal cause. If three pairs complete, report that the earlier failure was not reproduced; do not erase it or claim that logging fixed it. Do not claim a performance ranking from the traced cohorts. Resume broader capacity work only after this review.
