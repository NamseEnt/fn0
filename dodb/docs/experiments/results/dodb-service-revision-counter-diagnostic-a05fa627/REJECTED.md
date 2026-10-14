# Rejected revision-counter diagnostic run set

These artifacts are retained but do not support the final blocking-dispatch result. The implementation timer was started after the `spawn_blocking` closure had already started, so `blocking_dispatch_wait` captured only the post-dispatch interval. All ten processes completed their workload and validation, but the set is rejected for the requested attribution analysis. The corrected timer starts immediately before `spawn_blocking`; final measurements are in `dodb-service-revision-counter-diagnostic-113b712d/`.

A separate initial orchestration preflight failed before launching a workload because the `runs/` parent was missing. Its record is preserved under `oci/attempts/preflight-1/`.
