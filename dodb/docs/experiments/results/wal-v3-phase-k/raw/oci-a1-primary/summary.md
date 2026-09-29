# OCI A1 Primary J/K Diagnostic

Three interleaved real-sync repetitions measured 64 writers, width 16, uniform distribution. Each run used a 2-second warmup, 10-second measured interval, working set 4096, 64-transaction group limit, 2 Tokio workers, and 2 B-link workers. Source commit was `08e82ec484bd62aeb659471a39f317b86cfb488f`; J and K selectors ran from the same release binary (`1b65b5f66a64e12251f4a39cef570047cad125371fc2cc29134b24264593f0b6`). Raw rows and logs are in this directory.

| Metric (median) | Phase J synchronous | Phase K background |
| --- | ---: | ---: |
| Logical transactions/s | 4,592.5 | 5,169.0 |
| Writer blocked ns | 6,000,772,980 | 2,280,917,070 |
| Materialization total ns | 5,958,038,769 | 8,329,465,182 |
| Materialization CPU ns | not instrumented | 4,054,408,880 |
| Materialization data-write ns | not instrumented | 858,595,083 |
| Materialization data-sync ns | not instrumented | 2,145,466,944 |
| Checkpoint construction ns | not instrumented | 12,553,937 |
| Checkpoint sync ns | not instrumented | 605,788,520 |
| Publish pause ns | not instrumented | 68,184,162 |
| Backpressure events | 0 | 320 |
| Backpressure ns | 0 | 2,581,875,293 |
| Materializations | 260 | 350 |
| Materialized segments | 1,040 | 1,400 |
| Materialized B-link data bytes | 602,832,896 | 843,669,504 |
| WAL bytes written | 124,814,727 | 143,546,523 |
| Peak overlay segments | 4 | 8 |

The writer-blocked reduction was 61.99%, below the required 90%. K's overlay count reached the hard limit of eight segments and caused repeated backpressure. Its measured materializer completed 350 jobs (35/s) over about 10 seconds while committing 5,169 transactions/s; hard-limit backpressure consumed 2.58 seconds. K's materialization work also consumed about 4.05 CPU-seconds, 2.15 seconds in data sync, and 0.61 seconds in checkpoint sync. Total maintenance work exceeded the measured interval because it overlapped foreground writes.

This diagnostic fails the primary pause gate. The subsequent six-workload comparison was run only after identifying the hard-limit backpressure as the cause of the remaining writer stalls.
