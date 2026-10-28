# Common memory enforcement and pressure accounting

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: opt-in common backend memory enforcement and periodic pressure accounting accepted from clean runtime source `d506d00dd1f6324bf0074ba2ec0fd0f84792a1f8`. All 68 tests, four OCI correctness cells, separate high/max probes, and real-worker OOM diagnostic preservation passed. This is not a density or performance result.

## Implemented behavior

The density driver accepts an optional complete set of `--backend-cgroup-root`, `--backend-memory-max-bytes`, and `--backend-memory-high-bytes`. `--pressure-sample-interval-ms` defaults to 100 and requires that budget. Paths must be canonical, dedicated cgroup v2 domains with the memory controller enabled, no processes, and no existing child groups. Invalid/incomplete/unsupported configuration is rejected before database creation.

The driver configures a common parent limit across the whole cell, including charges retained after earlier workers exit. Setup, measurement, full-population oracle, and reopened oracle have separate child groups with the same maximum/high settings, zero swap allowance, and group OOM handling. Workers verify membership before constructing the Tokio runtime and opening their backend. The driver remains outside the backend groups. Existing routes without budget arguments retain their defaults, and WBM writer stalls remain disabled.

Periodic and boundary observations retain cgroup current/available kernel peak usage, memory.stat, memory.events/local events, swap, and memory/CPU/I/O PSI with timestamps and availability states. The offered-load callback records an observation before drain; native worker measurement samples include drain. Kernel peaks are for the entire cell or epoch lifetime, not isolated offered-load peaks. Failed workers retain final group counters and partial process diagnostics in the driver's failure report. Cgroup directories and databases remain preserved.

Opt-in periodic native observations include shared cache/WBM usage once per service, dynamic per-tenant RocksDB properties, and existing B-link checkpoint metrics. Repeated shared-cache properties on individual DBs must not be summed. Observation retention is bounded by 4,096 samples and 8 MiB of serialized native observations; the serialized bound is not an exact heap bound. Truncation is explicit. These observations consume CPU and worker memory; their performance perturbation is not quantified by this increment.

## Budget boundaries

This enforces a Linux memory-controller budget, not an exact RSS or machine-wide memory ceiling. Kernel enforcement can temporarily overshoot. Loader/argument-parsing allocations and previously charged shared file pages do not migrate retroactively. The driver, uncharged ZFS ARC, and host caches remain outside the backend budget. Existing host ARC/swap policies are unchanged. OOM and allocation failure are failed capacity outcomes; no graceful application byte-admission controller or B-link eviction is added.

Allocator/representation retained bytes, generation-pin ages, and request/checkpoint-queue representation bytes remain unsupported. The designated OCI host exposes neither `/proc/pressure` nor cgroup `*.pressure` files; PSI observations are explicitly unavailable rather than zero. No reboot or host-wide policy change was made to enable them. Memory events, stat counters, and usage/peak observations remain available. Normalized checkpoint image bytes retain their existing meaning. This increment does not establish larger-than-memory B-link operation, sustained maintenance stability, crash recovery, density boundaries, or a performance winner.

## Disk preparation

The explicit cleanup retained source checkouts, historical result logs/reports, databases including failed attempts, top-level runtime binaries, the active tenant-density build cache, and the pinned RocksDB source/archive. It removed 13 explicit paths: standard intermediate directories under an obsolete Cargo target on ZFS, an obsolete sccache, and standard debug intermediates under the historical boot-filesystem checkout. Runtime/native file identities were checked before/after. No dataset/filesystem was wiped.

Benchmark free space before cleanup was 11,071,692,800 bytes. After asynchronous ZFS reclamation settled, it was 42,959,200,256 bytes, approximately 40.0 GiB free and 29.7 GiB reclaimed. Boot free space increased by approximately 0.76 GiB. Immediate and settled filesystem observations differ because ZFS reclamation continues after unlink. Keep the raw cleanup timeline and recheck capacity before future runs.

## Acceptance evidence

All 68 clean-source tests passed: 42 library checks, 17 density binary checks, five read-contract checks, and four transaction-contract checks. Four tests are new in this increment. The worker binary has no unit tests. Changed-target Clippy retained the same 17 message/path diagnostics as the shared-reference audit: 16 library findings and one unchanged common-test-helper finding. No new implementation finding or lint suppression was introduced. Both A1-feature and default-feature binary links passed. Runtime acceptance uses the final default-feature binaries.

Development typecheck/unit logs and source overlays are retained separately; the pre-acceptance overlay was stashed before checking out clean `d506d00dd`. Runtime namespace `memory-pressure-d506d00dd-r1` failed before database execution because the controller enabled child controllers on the allocation probe's leaf cgroup; attaching a process there returned EBUSY. Namespace r2 corrected the leaf setup but reached 597 high events and a 60,030,976-byte peak before the 30-second controller timeout, without reaching the 64 MiB maximum. This is preserved as a rejected combined probe, not a completed OOM test. The corrected controller separates a bounded high-event probe from a hard-limit probe with memory.high disabled. Neither attempt ran a database workload. Runtime source/binaries are unchanged by these controller amendments.

## Real OCI acceptance

Run namespace: `memory-pressure-d506d00dd-r3`, on the designated `opc@217.142.246.204` host with preserved ZFS databases under `/bench/zfs/db/tenant-density-harness-20261027/`. The bounded controller ran both backends as UID 0 so it could move workers from the SSH session into its dedicated cgroups without changing existing user delegation. Driver processes stayed outside their backend budgets. Source, binary hashes, controller identity, commands, raw outcomes, and resource observations are preserved in `evidence/memory-pressure/oci/`. Remote frozen binaries remain in each run's `bin/`; they are excluded from the local evidence archive.

All four positive cells used 17 registered tenants, 256 seeded rows each, 4 KiB values, mixed transaction widths, the write-heavy profile, seed 20261027, one arrival lane per active tenant, 10 arrivals/s per active tenant for three seconds, queue bound eight, and common in-flight limit eight. The maximum/high settings were 256/192 MiB across every backend epoch and the enclosing cell. Native shared settings were the same explicit 8 MiB cache, 4 MiB WBM, 64 KiB per-tenant write buffer, one flush/compaction thread each, and 64 open files per tenant used by the reference. B-link used segmented coalesced bases, background checkpoints, 8 KiB segments, and a 16 KiB checkpoint threshold, with A1 disabled.

| Route | Active / opened-idle / never opened in measurement | Successful requests | Measurement cgroup lifetime peak bytes | Entire-cell cgroup lifetime peak bytes |
| --- | --- | --- | --- | --- |
| B-link, all active | 17 / 0 / 0 | 510 | 60,559,360 | 71,962,624 |
| Shared RocksDB, all active | 17 / 0 / 0 | 510 | 41,271,296 | 42,266,624 |
| B-link, sparse | 3 / 2 / 12 | 90 | 22,691,840 | 72,220,672 |
| Shared RocksDB, sparse | 3 / 2 / 12 | 90 | 12,619,776 | 26,353,664 |

These are accounting observations from tiny correctness cells, including runtime/transport/instrumentation and epoch opening/preflight. They are not RSS, retained heap, offered-load-only peaks, or a density advantage. Entire-cell observations also include setup and oracle epochs; sparse measurement must not be conflated with those all-population phases. Positive cells did not hit high/max or OOM. They do not establish behavior near a memory-capacity boundary.

All 1,200 requests succeeded without conflicts, errors, timeouts, drops, or final backlog. Both full-population oracles verified 4,352 seeded rows per cell plus 275 additional expected keys in each all-active cell and 51 in each sparse cell. The raw field is named `verified_mutated_keys`; its expected states include preflight and intended missing states, so this is not a distinct-successful-mutation count. Recipe/admitted-request/schedule/per-tenant fingerprints and operation/transaction/width counts matched within each B-link/shared pair. Membership, worker PID/start-time identity, actual native sharing/settings, histogram/accounting checks, and shutdown all passed. No successful cell was rerun.

Measurement-epoch periodic cgroup sample counts were 36/38/34/34 in table order, with maximum gaps of 104/101/101/101 ms. Worker measurement samples were 33/35/32/32. Samples are non-atomic and native measurement phases include drain. Missing PSI stays unavailable; sampling can still miss short native peaks. Kernel lifetime peak counters are recorded separately.

The bounded high probe observed 20 high events and a 51,175,424-byte kernel peak with no OOM, then the controller stopped it. The separate anonymous-allocation probe used a 64 MiB maximum and disabled high throttling; it reached a 67,108,864-byte kernel peak and recorded max/OOM/group-kill events while its controller survived. The real-driver expected-failure cell used a common 4 MiB maximum/high budget. Its worker was killed during setup; the driver exited nonzero and preserved the connection-loss error, partial diagnostics, final counters, and database. The parent recorded 58 max events, three OOM events, two `oom_kill` counter increments, and one group kill. Those counters do not represent successful requests or two separate accepted workload cells.

The final host review confirmed all 22 recorded driver/backend process identities had exited and the remote runtime checkout remained clean. Benchmark free space after the cells was 42,801,176,576 bytes, about 39.9 GiB, and boot free space was 3,062,747,136 bytes. The 10 GiB reserve was preserved. The first ad-hoc evidence reduction used a nonexistent oracle field; the corrected local review uses the actual schema without rerunning workloads. That reporting-only failure is recorded separately.

## Continuation

The reference, executable harness, common backend charging limit, disk reserve, and initial pressure observations are ready for a modest frozen E2/E3 capacity bracket. Recheck live capacity and define the common per-tenant service objective, load, dataset, duration, repetition policy, and sampling perturbation before execution. Separate all-active and sparse populations and inspect E4 maintenance near the observed boundary. Retained-heap/pin/queue attribution, PSI availability, machine-wide ARC accounting/enforcement, graceful application memory admission, and B-link demand loading remain incomplete. Issue #126 stays open; no default or adoption decision is promoted.
