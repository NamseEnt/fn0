# New-Key D30 Results

The separate D capacity sample completed three 10-second warmup plus 10-second measurement cases. This report contains the standard D cohort's nine cases, each with a 10-second warmup and 30-second measurement across three rotated repetitions. The capacity sample is not counted in the standard matrix. Together, the earlier 102 A/B/C/E cases and these nine D cases complete the planned 111-case 30-second matrix; no five-minute run is included.

The workload source is `ee6cd94f1a4e7b2bab7af603bc8983bbf8184542`. The frozen `service_workload` SHA-256 is `7c8b35679df63402b3454067d720c53e1a194f5be6fe459eb7bec35862bdd4b5`; `unconditional_write_workload` is `97b2b38717679bba4000a9eed2bf8c2435114e3d7fa5717e9727381e3586e3f4` and is not used by cohort D. The D controller SHA-256 is `bd3b789734b1843c98c322098dc283ca69ddfbbc982f369d7d28f85d87b46f1f`; plan fingerprint is `c77853b17cd09c62778608b54df71ae4c37c70ee44c19ef2329b6d9b67d1ceea`. The plan-only output is `plan-only.json`. The source-frozen release binaries were executed directly without a build.

All cases used seed `20261013`, 10,000 initial rows, 512-byte values, 16 clients, `rw50`, new-key mutation workload, 1,024 slots per client, width 16 enabled, ReservedRanges, Shared condition locks, 16 service/core admission slots, dedicated client executors, `log1024`, a 100,000,000 call cap, a 5,000,000-entry ledger cap, full-window enforcement, 1-second engine-health sampling, and detailed/stage diagnostics disabled. RocksDB cases used sync WAL; Blink does not use a RocksDB WAL.

| Backend | Successful write transactions/s, median (min–max) | Workflow p99 ms, median | Write p99 ms, median | Whole-process max RSS, median | Reopened rows, median (min–max) |
|---|---:|---:|---:|---:|---:|
| Blink | 2,453 (2,396–2,485) | 11.16 | 12.29 | 1,187 MiB | 284,478 (280,855–289,675) |
| RocksDB service Shared-16 | 4,535 (4,461–4,572) | 6.85 | 8.48 | 739 MiB | 527,632 (525,140–532,052) |
| Synchronous RocksDB core Shared-16 | 5,258 (5,188–5,299) | 5.96 | 7.92 | 853 MiB | 607,580 (603,815–615,314) |

All nine reports passed the exact-source and binary checks, complete measurement-window checks, sparse histogram count and percentile checks, ACK and input-prefix checks, final-state oracle, reopen oracle, and database log/hash manifest checks. Each RocksDB case recorded 29 one-second engine-health samples over the 30-second window; Blink reports engine properties as unsupported. The `/usr/bin/time -v` whole-process maximum includes validation and reopen, unlike the in-measurement RSS samples. Every successful database was deleted only after its manifest was written. No database remains in the remote result root.

The first runner attempt stopped at preflight before starting a workload because the controller checked the shared build-cache path instead of the approved frozen binary path. Its stdout, stderr, and status are preserved in `failed-attempt-1/`; no database was created. The controller was corrected to use the frozen `ee6` path, then the D-only run completed successfully. This harness correction did not change workload source or binaries.

The remote result root is `/bench/zfs/db/condition-shared-lock-20261019/new-key-d30-ee6/results`; its compressed archive is retained remotely at `/bench/zfs/db/condition-shared-lock-20261019/new-key-d30-ee6/d30-results-ee6.tar.gz` and locally at `d30-results-ee6.tar.gz`, with SHA-256 `716a56331debb3cbf3b7a7df1adcc5e1a97b946860c50eb48d3548120cae1e9a`. The primary review evidence is the committed extracted case reports and manifests; the archive is not a repository link. The capacity-only three-case sample remains separate at `/bench/zfs/db/condition-shared-lock-20261019/new-key-capacity-ee6/results` and is excluded from the 111-case count.
