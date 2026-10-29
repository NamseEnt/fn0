# Shutdown attribution and failure-counter preservation

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: implementation and clean-source checks passed. The new frozen diagnostic stopped on a five-tenant setup-worker watchdog failure. Its phase trace narrows the outstanding interval to asynchronous server shutdown; the internal cause remains unresolved.

## Implementation and validation

Runtime source is `b22638bdcc4a0b22dad0606c3003d193cf41bbed`. The primary owns design and acceptance; GPT-6 Luna at high reasoning effort implemented the changes and returned summaries. [SHUTDOWN-DIAGNOSTIC-PLAN.md](SHUTDOWN-DIAGNOSTIC-PLAN.md) was committed before implementation, and [SHUTDOWN-DIAGNOSTIC-MATRIX.md](SHUTDOWN-DIAGNOSTIC-MATRIX.md) fixes the next runtime boundary.

The driver retains a successfully returned workload summary before sending stop/shutdown commands. If a later operation fails, `partial_workload_counters.status` is `available_unvalidated_after_later_failure`, with the original summary under `workload`. Earlier failures retain `unavailable_after_early_abort`. These counters do not validate the cell or its oracles. The additional summary lives in the driver outside the backend budget; completed reports retain their existing shape. This cannot reconstruct the missing source-023 workload counters retroactively.

Opt-in pressure diagnostics now emit bounded structured stderr markers around command handling, final native sampling/properties, asynchronous server shutdown, final checkpoint metrics, server/backend destruction, health-report writing, async-run return, and runtime destruction. Markers carry timestamp and PID/start-time/source identity. They do not alter readiness stdout. Disabled tracing avoids process lookup, source copying, stderr locking, and timestamp collection. Enabled tracing is initialized after cgroup attachment and before runtime creation. Logging errors are best-effort and do not replace the primary result.

No engine/storage/service code, request recipe, lock/admission rule, native option, checkpoint policy, durability, default, or timeout changed. The worker's fifteen-second asynchronous shutdown timeout and parent's twenty-second stop watchdog remain distinct. Phase markers locate elapsed time; they do not establish the internal cause of a stall.

All seventy-nine clean-source checks passed: forty-two library, nine worker, nineteen density-driver, five read-contract, and four transaction-contract tests. The five additions cover available/unavailable failure metadata and trace identity/encoding, disabled output, and write errors. Targeted formatting passed. Advisory Clippy matched the same seventeen message/path findings without a new finding or suppression. A1/default compatibility links passed with default last, and the pinned native archive and relocated cache symlink were preserved.

One validation-wrapper preflight attempt failed on a stale variable reference before any Cargo check started. Its launcher log remains preserved. The corrected wrapper completed the real checks and can reuse only provenance-matching successful checks; failed/incomplete checks are not automatically retried.

## Runtime acceptance boundary

Namespace `shutdown-b22638bdc-r1` attempted four cells before the frozen stop rule fired. Both tiny correctness cells completed one hundred successful API calls, both oracles, verified worker identities, and normal shutdown-marker ordering. Their Query sample counts are below the SLO minimum, so their recorded SLO=false is not a capacity-tail miss. The first five-tenant shared-reference cell completed 15,000 calls and both oracles and passed the objective. The following B-link cell failed in setup; offered load never started. No remaining repetition ran, and no completion receipt was fabricated.

The three complete reports account for 15,200 successful offered calls and six full-population oracles. Collection verified four preserved databases, seventeen exited process identities, empty groups, clean source, unchanged frozen binaries/native archive, and the existing target symlink. The setup failure contributes no throughput, measurement-native summary, request success, or oracle result. The available-after-later-failure metadata branch was covered by unit checks and source review; this setup abort did not exercise it with an actual returned workload summary.

## Located interval and remaining uncertainty

The failed case is `confirm-e3-n5-r1-blink`, with backend PID/start ticks `2406935:250149303`. Its exact stderr sequence is:

1. `command_received_shutdown`
2. `final_property_capture_start`
3. `final_property_capture_end`
4. `async_server_shutdown_start`

Final property capture took 28,800 ns. No asynchronous server-shutdown end/error marker, worker-internal fifteen-second error, final checkpoint-metric, backend-drop, health-report-write, or runtime-destruction marker was observed afterward. The parent reported `diagnostic worker shutdown timed out and was killed` and the original completion reducer stopped for review. Its raw review error remains unchanged.

The marked interval awaits `server.shutdown()` and then the server task. Source shows that server shutdown closes the endpoint, awaits connection idle, and awaits service-owned shard/coordinator shutdown. The trace does not distinguish these awaits, executor scheduling, or kernel waits. This setup failure is not the same measured phase as the earlier source-023 measurement failure; no shared internal cause is established.

The failed cell took 106.129 seconds. Parent/setup peaks were 228,511,744 / 228,470,784 bytes. Parent/setup-local high counts were 6,218 / 3,109; max, OOM, and kill counts were zero. Measurement/oracle/reopen peaks were zero. Between the approximately +1.036 s and +19.047 s shutdown samples, setup cgroup CPU advanced only 22,348 microseconds while high events advanced from 2,802 to 3,109. Memory remained above the 192 MiB high limit. These observations motivate external thread-wait sampling; they do not identify a specific blocking allocation or syscall. [SHUTDOWN-OFFCPU-PLAN.md](SHUTDOWN-OFFCPU-PLAN.md) preserves that next diagnostic before implementation, with both timeouts and the resource policy fixed.

## Reproduction and preserved evidence

The raw `evidence/shutdown-diagnostic/runtime.tar.gz` archive is 217,505 bytes with SHA256 `faa8385a15da717d4c5b1f9337e244d6fe7e351a71df136443a975c26e3204a4`. It includes runtime records, postflight, validation logs/provenance, the actual executed validation wrapper, and both the failed preflight and successful validation launchers. The extracted source-023 evidence and older capacity archives remain intact.

From the worktree, run:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 dodb/docs/experiments/blink-tenant-density-20261027/evidence/shutdown-diagnostic/review_shutdown.py dodb/docs/experiments/blink-tenant-density-20261027/evidence/shutdown-diagnostic/runtime/shutdown-b22638bdc-r1 --postflight dodb/docs/experiments/blink-tenant-density-20261027/evidence/shutdown-diagnostic/collection/shutdown-runtime-b22638bdc-r1-postflight.json --validation-summary dodb/docs/experiments/blink-tenant-density-20261027/evidence/shutdown-diagnostic/validation/shutdown-validation-summary.json
```

The derived review distinguishes setup from measurement, preserves the original error, verifies the archive transfer, limits, identities, and normal traces, and classifies only the exact observed setup watchdog failure. Re-running identical reduction reuses the matching derived result. The earlier unclassified derived review is separately retained; raw reports were not rewritten.

Do not present this trace as an engine fix, a production maximum, or new crash-recovery evidence. Issue #126 remains open.
