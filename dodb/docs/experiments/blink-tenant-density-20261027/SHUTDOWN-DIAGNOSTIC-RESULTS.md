# Shutdown attribution and failure-counter preservation

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: implementation and clean-source checks passed; the new frozen runtime diagnostic is pending. The earlier measurement-worker failure remains unresolved until phase evidence is reviewed.

## Implementation and validation

Runtime source is `b22638bdcc4a0b22dad0606c3003d193cf41bbed`. The primary owns design and acceptance; GPT-6 Luna at high reasoning effort implemented the changes and returned summaries. [SHUTDOWN-DIAGNOSTIC-PLAN.md](SHUTDOWN-DIAGNOSTIC-PLAN.md) was committed before implementation, and [SHUTDOWN-DIAGNOSTIC-MATRIX.md](SHUTDOWN-DIAGNOSTIC-MATRIX.md) fixes the next runtime boundary.

The driver retains a successfully returned workload summary before sending stop/shutdown commands. If a later operation fails, `partial_workload_counters.status` is `available_unvalidated_after_later_failure`, with the original summary under `workload`. Earlier failures retain `unavailable_after_early_abort`. These counters do not validate the cell or its oracles. The additional summary lives in the driver outside the backend budget; completed reports retain their existing shape. This cannot reconstruct the missing source-023 workload counters retroactively.

Opt-in pressure diagnostics now emit bounded structured stderr markers around command handling, final native sampling/properties, asynchronous server shutdown, final checkpoint metrics, server/backend destruction, health-report writing, async-run return, and runtime destruction. Markers carry timestamp and PID/start-time/source identity. They do not alter readiness stdout. Disabled tracing avoids process lookup, source copying, stderr locking, and timestamp collection. Enabled tracing is initialized after cgroup attachment and before runtime creation. Logging errors are best-effort and do not replace the primary result.

No engine/storage/service code, request recipe, lock/admission rule, native option, checkpoint policy, durability, default, or timeout changed. The worker's fifteen-second asynchronous shutdown timeout and parent's twenty-second stop watchdog remain distinct. Phase markers locate elapsed time; they do not establish the internal cause of a stall.

All seventy-nine clean-source checks passed: forty-two library, nine worker, nineteen density-driver, five read-contract, and four transaction-contract tests. The five additions cover available/unavailable failure metadata and trace identity/encoding, disabled output, and write errors. Targeted formatting passed. Advisory Clippy matched the same seventeen message/path findings without a new finding or suppression. A1/default compatibility links passed with default last, and the pinned native archive and relocated cache symlink were preserved.

One validation-wrapper preflight attempt failed on a stale variable reference before any Cargo check started. Its launcher log remains preserved. The corrected wrapper completed the real checks and can reuse only provenance-matching successful checks; failed/incomplete checks are not automatically retried.

## Runtime acceptance boundary

The new namespace is `shutdown-b22638bdc-r1`: a tiny paired correctness/trace check, then up to three fresh paired five-tenant E3 observations. Stop at the first operational or diagnostic failure and preserve omitted cells. This is a changed instrumentation variant for attribution, not a repeat to repair the old result or obtain a passing median. No six/eight-tenant or maintenance cells are authorized by this matrix.

Do not present the phase trace as an engine fix, a production maximum, or new crash-recovery evidence. If all pairs complete, report only that the bounded variant did not reproduce the earlier failure. Issue #126 remains open.
