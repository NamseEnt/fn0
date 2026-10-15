# Validation Attempts

All workload data and failed orchestration/checker attempts are retained.

| Attempt | Outcome | Follow-up |
| --- | --- | --- |
| First smoke runner command capture | Failed before launching any workload because Bash `printf` treated `--data-dir` as an option. | Corrected command recording and ran the three planned smoke configurations once under `oci/dodb-service-shared-slots-00fc0e382/smoke/attempt-02/`. The first attempt is recorded in the OCI evidence tree. |
| First summary pass | Local report summarizer expected the run-order CSV header to count as a data row and stopped with 15 data rows found instead of 16. | Corrected the row-count assertion to 15, then validated all original reports without rerunning any workload. `paired-input-validation.json` records all five matching schedule and first-100 input fingerprint checks. |

Neither failure represents a failed smoke or performance workload. No performance condition was rerun.
