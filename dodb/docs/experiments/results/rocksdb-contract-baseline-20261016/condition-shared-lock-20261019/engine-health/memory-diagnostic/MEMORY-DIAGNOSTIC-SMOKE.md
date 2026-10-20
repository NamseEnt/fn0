# Memory Diagnostic Smoke and 7-Case Budget

## Provenance and gates

The memory-only smoke used source `843f0ea1598a4fa87284d1e3ee421612f288b4c2`, a release build from the pinned OCI toolchain, and three frozen executables. The frozen binary hashes are in [`evidence-843f0ea15/binary-sha256.txt`](evidence-843f0ea15/binary-sha256.txt); identical executable copies remain on the OCI host under `/bench/zfs/db/condition-shared-lock-20261019/engine-health/memory-diagnostic-843f0ea15/frozen-binaries/`.

The smoke completed all seven paths with 1 second warmup and 5 seconds of measurement. Each report passed its final and reopen oracle, process identity and shutdown checks, and bounded residency scan checks. All reports requested and accepted the full window. The parent-death control killed the workload process while its worker was opening the backend; both processes shared one process group and `worker_exited_after_parent_kill=1` was recorded in [`evidence-843f0ea15/parent-death/processes.txt`](evidence-843f0ea15/parent-death/processes.txt). Every case saved file sizes and hashes and copied each native `LOG` and `LOG.old.*` before removing its validated database. The incomplete parent-death database was retained.

The release gate ran at `c39db5ddd857ec807580aad143c81e3ee38e53a9`. Relative to smoke source `843f0ea15`, it adds only nine test-initializer fields in `service_workload.rs` and `unconditional_write_workload.rs`; production code and frozen executables are unchanged. The gate passed 63 tests: 34 unit, 9 service, 4 unconditional workload, 4 history, 4 process-crash, 2 read, 5 research metrics, and 1 transaction. The residency negative control `residency_requires_valid_unchanged_device_and_backend_identity` passed as part of the unit suite. Failed gate logs are retained under `evidence-ee6/gate` and the OCI build-run directory; they are not counted as passes.

## Smoke observations

RSS and PSS values below are sampled maxima for the workload process group across setup, measurement, shutdown, reopen, and residency scans. Parent and backend columns show the highest separately sampled process values. They are not allocator-only measurements. Swap stayed at zero in every process and group sample.

| Path | Rows in bounded full-scan digest | Key bytes | Value bytes | Parent RSS / PSS | Backend RSS / PSS | Group RSS / PSS | DB bytes after reopen validation | Second scan from RAM |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| A RocksDB service, c16 rw20 | 1,187 | 34,757 | 607,744 | 18,868 / 15,977 KiB | 23,996 / 20,823 KiB | 37,444 / 31,095 KiB | 11,018,063 | 4/4 tenants |
| A synchronous core, c16 rw20 | 1,193 | 34,909 | 610,816 | 18,740 / 15,863 KiB | 24,136 / 20,961 KiB | 37,520 / 31,178 KiB | 11,180,798 | 4/4 tenants |
| A B-link, c16 rw20 | 1,198 | 35,055 | 613,376 | 18,816 / 15,912 KiB | 16,304 / 13,326 KiB | 29,820 / 23,669 KiB | 16,084,306 | 4/4 tenants |
| E raw, c16 width1 smoke | 4,096 | 254,224 | 2,097,152 | 28,960 / 26,099 KiB | 31,512 / 28,322 KiB | 53,244 / 46,856 KiB | 2,423,773 | 1/1 database |
| E synchronous core, c16 width1 smoke | 4,096 | 254,224 | 2,097,152 | 29,604 / 26,503 KiB | 29,504 / 26,340 KiB | 59,108 / 52,776 KiB | 8,434,787 | 1/1 database |
| E RocksDB service, c16 width1 smoke | 4,096 | 254,224 | 2,097,152 | 29,900 / 26,796 KiB | 27,432 / 24,222 KiB | 57,332 / 50,916 KiB | 8,227,077 | 1/1 database |
| E B-link, c16 width1 smoke | 4,096 | 254,224 | 2,097,152 | 30,524 / 27,412 KiB | 40,368 / 37,338 KiB | 70,892 / 64,649 KiB | 30,278,481 | 1/1 database |

The E smoke used width1 and 16 callers only to exercise all four adapters. Its result is not the c64/width16 case budget. All 16 second scans used `/dev/sda4`, observed the same backend PID/starttime before and after, matched the reopened scan digest, saw zero 512-byte-sector read delta and zero backend Swap, and were therefore classified as RAM-served for that second bounded scan only. This does not prove that the earlier five-minute measurements were resident.

Every RocksDB database reported a 33,554,432-byte block-cache capacity. On reopened A databases, per-database block-cache usage ranged from 139,440 to 184,208 bytes and estimated live data ranged from 134,042 to 177,173 bytes. Reopened E raw/core/service estimates were 2,332,908–2,332,927 bytes, with 2,423,008–2,423,216 bytes of block-cache usage. The reported live data and cache values are engine properties; they are not a sum with the shared ZFS ARC or a dedicated host-memory guarantee. B-link reports health properties as unsupported.

## Approved 7-case diagnostic budget

The selected run is 3 conditional A paths and 4 unconditional E paths, one serial repetition each, at 10 seconds warmup and 30 seconds measurement. It reuses the 843 frozen binaries and the five-minute input settings: seed `20261013`, 512-byte values, full 100M call cap, width16 enabled, log1024 reporting, 1-second engine-health sampling, and stage diagnostics off. A uses c16 `rw20`, 10,000 initial rows, 1,024 overwrite slots per client, reserved-range revisions, shared condition locks, and 16 shared driver permits on RocksDB service/core paths. E uses c64, width16, and the 256 preseed slots per client; raw/core have 64 direct callers, RocksDB service uses 32 requested shared slots, and B-link uses its existing unconditional configuration. The raw path has no logical revision, OCC, or ABA semantics.

The observed five-minute maximum whole-process RSS was 1.51 GiB. Adding a 512 MiB planning margin gives an estimated peak of about 2.01 GiB for this diagnostic run. This is a budget estimate, not a guarantee or a measured seven-case peak. The tiny smoke's highest sampled group RSS was 70,892 KiB and its highest backend post-shutdown `VmHWM` was 40,100 KiB. The run controller enforces a 3 GiB process-group RSS cap, a 1.5 GiB runtime `MemAvailable` reserve, a 12 GiB per-case database cap, a 900 second case timeout, and a 300 second worker-startup timeout. The preflight also requires at least 26 GiB free ZFS space and 4.5 GiB available RAM before starting.

The plan-only preflight measured 48.0 GiB free in `/bench/zfs/db` and 5.11 GiB `MemAvailable`. Its seven-job plan fingerprint is `ddc2652882b69277584a08c6c8ecb09f5914ae7b83b2aafc8a11e7f6de5ad1a1`; the exact plan and host snapshot are in [`plan-only-843f0ea15-7x30.json`](plan-only-843f0ea15-7x30.json). The controller is [`run-memory-diagnostic-7x30.py`](run-memory-diagnostic-7x30.py). It uses a separate process group per case, records 1-second parent/child RSS, PSS, anonymous, and Swap samples plus host memory, samples database size every 5 seconds, and terminates only that case's group on a cap or timeout. Failed cases keep their database and raw outputs. Successful cases are deleted only after report validation, full file size/hash collection, and native log copies are verified.

This diagnostic is for process memory and the second bounded-scan residency check. Loopback transport, service, and worker costs remain in the backend-process measurements; none of these numbers replace the frozen direct-path performance results.
