# Diagnostic Attempt Log

All rejected attempts and post-run checks are preserved in the run folders
and `dodb-rocksdb-build-runs/`.

| Attempt | Planned / executed | Outcome | Handling |
| --- | ---: | --- | --- |
| Local Cargo check | 1 / 1 | Build script stopped because `DODB_ROCKSDB_DIR` was unset; pinned RocksDB 11.8.1 was unavailable locally. No tests ran. | Preserved as the local preflight limitation. |
| OCI focused build at `07eaf785` | 1 / 1 | Compile failed because the workload reopen-validation constructor lacked the new read-stage flag. | Corrected and pushed as `6b101c1b`; failed helper log retained under `dodb-rocksdb-build-runs/read-stage-07eaf785-focused-1/`. |
| OCI focused tests at `6b101c1b` | 15 / 15 | 11 library tests and 4 `research_metrics` tests passed. | Accepted. |
| OCI workload build at `6b101c1b` | 1 / 1 | Release `service_workload` build passed using the shared target and existing RocksDB archive. | Accepted. |
| Diagnostic smoke | 1 / 1 | Workload exited 0; replay and reopen validation passed; stage counts matched Get, preparation Get, and Query16 request counts. | Accepted after correcting an initial jq path in the post-filter. Original filter output remains in `oci/runs/smoke-on/`. |
| Six OCI conditions | 6 / 6 | All six binaries exited 0; all passed replay/reopen validation. Five ON runs had nonzero read-stage samples and exact service-histogram/request-count matches. The OFF run reported metrics disabled. | Accepted; no workload condition was retried. |

The matrix validator needed two corrections after preserving its initial
outputs. First it used incorrect tenant locations for the read-stage JSON.
The next filter compared bounded driver latency `sample_count` with the full
request count, which incorrectly rejected RW95 and RW80 because those
categories exceeded the 20,000-observation reservoir limit. The final filter
requires nonzero driver samples and matches the full request counts against
the new bounded histogram counts. This final check passed all five ON runs;
the OFF check passed on its first valid filter. These were post-processing
failures, not failed workload executions, and no workload was repeated.

A local final-audit command was also first run from the nested `dodb/`
directory while using repository-root-relative report paths, so it stopped
before reading any report. The same count/hash/replay/sample-count audit was
rerun from the repository root and passed all six reports. No artifact was
changed by the failed invocation.

| Phase accounting | Planned | Executed | Passed | Failed | Not reached |
| --- | ---: | ---: | ---: | ---: | ---: |
| Local focused tests | 15 | 0 | 0 | 0 | 15 |
| OCI focused tests | 15 | 15 | 15 | 0 | 0 |
| OCI smoke | 1 | 1 | 1 | 0 | 0 |
| OCI measurement | 6 | 6 | 6 | 0 | 0 |
