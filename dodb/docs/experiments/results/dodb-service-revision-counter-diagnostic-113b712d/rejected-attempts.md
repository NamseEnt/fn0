# Rejected diagnostic attempts

These artifacts are retained for audit and are not used as final measurements.

- The first workload orchestration attempt did not create its `runs/` parent and therefore launched zero workload processes. Its record is under `oci/attempts/preflight-1/` in the a05fa627 artifact directory.
- Ten workloads then completed at implementation SHA `a05fa627fec3fbb2f3bb557f632b3c229bd37ec1`. Inspection found that `blocking_dispatch_wait` started its timer after entering the `spawn_blocking` closure, so it measured only the interval after dispatch had finished. Those ten reports and logs are retained under `dodb-service-revision-counter-diagnostic-a05fa627/oci/runs/`; none are included in the final comparison.
- The timer now starts immediately before `spawn_blocking`. The corrected implementation SHA is `113b712de62e013bd299ba1769ed7ac953dc75bc`. Its five OFF/ON pairs are the only runs used in `REPORT.md`.
- No workload process failed. The corrected matrix ran once per condition with no workload retry.
