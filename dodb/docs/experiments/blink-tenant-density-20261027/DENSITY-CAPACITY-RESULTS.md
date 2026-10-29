# Bounded active-tenant capacity under a common backend budget

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: the frozen main and conditional supplemental sequences are complete. Fifty-seven unique OCI cells produced 2,304,000 successful offered requests in complete reports, 110 passing full-population oracles, three completed latency-objective misses, one setup OOM, and one setup shutdown-deadline failure. No successful cell was rerun to repair reporting. The result describes this instrumented service route; engine-only attribution and adoption remain incomplete.

## Fixed conditions

Runtime source is `aaa40565529ca92150df5d94e19b02ca93859f71`. Relative to the accepted memory-pressure source, it only summarizes periodic checkpoint detail; engine behavior and defaults remain unchanged. Default-feature binaries are used; A1 stays disabled. All 68 clean-source checks and both default/A1 compatibility links passed. Advisory Clippy retained exactly the same seventeen existing message/path findings, with no new finding or suppression.

The designated two-vCPU OCI/ZFS host uses a common 256 MiB maximum and 192 MiB high boundary for the whole cell and every backend epoch, with zero backend swap. The driver, previously charged shared pages, and uncharged host ARC remain outside this budget. Host policy is unchanged. This is an instrumented backend-cgroup comparison, not a machine-wide memory ceiling or isolated engine-heap comparison.

Every active tenant receives fifty scheduled API calls/s through one sequential lane, with synchronized tenant deadlines, a queue bound of eight, and a shared in-flight maximum of eight. The mixed profile is Get/Query/write=40/20/40; writes are half conditional with widths 1/4/16 at 80/15/5, and ten percent of mutations are Delete. Values are 512 bytes. E2 seeds 2,048 rows/1 MiB of value payload per tenant; E3 seeds 32,768 rows/16 MiB. Query always returns the first sixteen seeded rows. Get walks the deterministic seeded-row sequence. Writes use a separate fixed sixteen-key overwrite/delete/reinsert lane.

B-link explicitly uses segmented coalesced bases, background checkpoints, 256 KiB WAL segments, and a 1 MiB checkpoint threshold. The TransactionDB reference uses a shared 64 MiB cache and 32 MiB WBM, 1 MiB tenant write buffers, one flush/compaction thread each, 64 open files per tenant, and disabled WBM writer stalls. Actual native sharing/options are verified; this is not maximum-tuned RocksDB. The native service uses blocking dispatch while B-link uses its direct async service path. Both retain the same outer admission and durability/transaction contract.

The experimental objective requires every tenant to finish every scheduled request successfully, with no drops/conflicts/errors/timeouts, at least 99% completed by offer end, and per-tenant scheduled-to-completion p99 below 8.388608 ms Get, 16.777216 ms Query, and 67.108864 ms Write. At least one hundred observations per operation per tenant are required. These thresholds align with the existing histogram buckets and are not product requirements or interpolated exact percentiles. Full-population value/state validation, clean reopened validation, accounting, identity, sharing, and shutdown gates are separate requirements.

## Confirmed finite-window frontier

At 1 MiB of value payload per tenant, both routes demonstrated 28 active tenants in all three sixty-second repetitions. At 16 MiB, B-link demonstrated five and the shared reference demonstrated at least eight. These are highest tested three-of-three counts under the complete-cell acceptance gates, not universal production maxima or a claim that the reference is 60% faster per tenant.

The table uses medians of three repetitions. Both columns are kernel cgroup peaks, not RSS or retained heap. The first is the measurement epoch's lifetime peak as read at the offer boundary, so it also includes opening/preflight. The second includes drain, shutdown, and final diagnostic serialization. The eight-tenant reference repetitions form an independently frozen supplemental cohort; they are not pooled with the thirty-second screen.

| Value payload / tenant | Route | Active tenants | 60-second passes | Epoch peak through offer end, MiB | Final measurement-epoch peak, MiB |
| --- | --- | ---: | ---: | ---: | ---: |
| 1 MiB | B-link | 28 | 3/3 | 67.54 | 67.71 |
| 1 MiB | Shared TransactionDB | 28 | 3/3 | 103.74 | 135.38 |
| 16 MiB | B-link | 5 | 3/3 | 184.61 | 184.61 |
| 16 MiB | Shared TransactionDB | 5 | 3/3 | 40.24 | 46.91 |
| 16 MiB | Shared TransactionDB, supplement | 8 | 3/3 | 62.64 | 72.75 |

Both routes also passed all three E2 repetitions at 24 tenants. Their single thirty-second 32-tenant screens missed the Get objective. Counts 29-31 and sixty-second 32-tenant repetitions were not tested. Shared RocksDB's thirty-second 28-tenant screen missed Get p99, while its three sixty-second repetitions passed; these different duration cohorts must not be treated as four identical trials. E2 demonstrates no active-count advantage at the confirmed counts.

E3's next tested B-link count, six, failed before offered load because the setup worker missed its shutdown deadline. Eight failed during setup with kernel OOM. The five-tenant lower point passed three times; neither operational failure was repeatedly provoked to manufacture a median. The reference's eight-tenant confirmation passed three times; counts above eight remain untested.

## Operational failures

B-link at eight E3 tenants failed with kernel OOM during setup. Its cell lasted 1,023.29 seconds including failure detection; measurement never began. Parent counters recorded 39,293 high events, 73 maximum events, three OOM events, two OOM-kill counter increments, and one group kill. Those counters describe one failed cell, not two accepted workload trials or request counts.

B-link at six E3 tenants completed seeding but its setup worker exceeded the existing fifteen-second shutdown deadline. That cell lasted 346.43 seconds, peaked at 245,739,520 bytes, recorded 18,569 parent high events, and recorded no maximum/OOM events. Measurement never began. High events and the deadline failure are measured; the exact allocation/checkpoint cause has not been profiled. Neither failure establishes zero throughput, a workload timeout, or corruption.

## Cadence and report-retention cost

The one-tenant E2 sensitivity ran identical sixty-second recipes at 1,000 and 100 ms cadences. All four cells passed. Shared RocksDB's epoch peak through the offer boundary increased from 6.49 to 16.23 MiB, and its final epoch peak increased from 7.96 to 33.70 MiB. B-link's corresponding observations were 5.49 to 8.39 MiB through offer end and 5.72 to 14.12 MiB for the final epoch. These are single observations per route/cadence, not a statistical zero-overhead result.

The timing distinction matters at larger populations too. In the first 28-tenant shared-reference repetition, the epoch peak was 107,806,720 bytes at offer end and 142,434,304 bytes at worker completion, an increase of 34,627,584 bytes after offer end. The source retains sample DOM objects and embeds them in another `json!` value before serializing a full health-report buffer, after dropping the backend. This confirms a reporting path capable of additional allocation; it does not attribute every byte of the measured increase without profiling. Do not turn final epoch peaks into an engine-only memory ranking.

[OBSERVATION-RETENTION-PLAN.md](OBSERVATION-RETENTION-PLAN.md) freezes the next correction: retain validated serialized frames and stream the final health report. That implementation is not included in this result.

## Maintenance and sparse observations

Both routes passed one 180-second E2 maintenance cell at sixteen active tenants with 1,000 ms sampling and one at the actual confirmed twenty-eight-tenant frontier with 2,000 ms sampling. The different cadences are separate cohorts. B-link recorded 96 and 168 successful checkpoint events started during the offered intervals respectively, without failed or omitted final events. No event reported an explicit full-base snapshot; coalesced-pack I/O still occurs, so this is not proof that full-pack rewrite pressure or indefinite consolidation stability was exercised.

At twenty-eight tenants, sampled B-link WAL current bytes ranged from 27,062 to 1,025,791 and finished at 31,061. Native shared cache usage reached 67,108,592 bytes and ended at 52,742,536; WBM usage reached 28,501,704 and ended at 14,738,304. Sampled pending-compaction bytes and write-stopped flags were zero at the two-second cadence. Short flush/compaction activity can occur between samples. At sixteen tenants, sampling did capture pending compaction up to 5,834,887 bytes, which ended at zero. These are progress observations, not a proof of perpetual stability or absence of engine activity.

The sparse E3 pair used five registered, one active, one opened-idle, and three never-opened-in-measurement tenants. Both passed sixty seconds and both complete-population oracles. Their measurement-epoch peaks were 81,162,240 bytes for B-link and 15,790,080 for the reference. Setup/oracle still opened the complete population under the same budget. Sparse results are not included in the all-active frontier.

## Preservation and continuation

The original namespace is `capacity-aaa405655-r1`. It stopped after the six-tenant deadline because the initial reducer handled OOM but did not classify that operational timeout. The raw reports, original statuses, logs, and databases remain immutable. The source of the rejection was reviewed before continuation.

`capacity-aaa405655-r2` is an explicit composite evidence view. It reuses the original 36 raw reports byte-for-byte and runs only previously unexecuted cells. The known deadline receives a derived classification with its original review error retained. A preflight reviewed all 36 reports and verified rejection of unrecognized stderr, mismatched source, and missing high-event evidence. `reused_existing_cases` and `executed_new_cases` distinguish evidence reuse from actual execution; do not add the two namespaces' case counts together.

The main view contains 52 unique executions: 36 reused reports plus sixteen new continuation cells. The supplement adds five new executions. Fifty-five cells completed all operational/oracle gates, fifty-two also met the latency objective, and three completed screens missed that objective. Both full-population oracles passed for every completed cell. The 2,304,000 successful-request total excludes unreported offered work in the two pre-load failures and excludes setup/oracle requests.

See [the main plan](DENSITY-CAPACITY-PLAN.md), [the continuation protocol](CAPACITY-CONTINUATION.md), [the frozen conditional supplement](CAPACITY-SUPPLEMENT-PLAN.md), and [the source audit](CAPACITY-SOURCE-AUDIT.md). The supplement independently confirms a surviving eight-tenant E3 route and observes maintenance at the actual confirmed E2 boundary when the main sixteen-tenant retention cap prevents doing so.

## Evidence and resource preservation

Raw evidence, report hashes, source/binary/controller identities, commands, matched recipes, population checks, host samples, and independent reductions are under [evidence/capacity/](evidence/capacity/). The reproducible aggregate receipt is [final-review.json](evidence/capacity/final-review.json). Run `PYTHONDONTWRITEBYTECODE=1 python3 evidence/capacity/review_all.py` from this document's directory to verify both archives, all reports, the unchanged original 36 reports/statuses, cohort reductions, and aggregate counts. Runtime frozen binaries and every database remain on OCI; local archives exclude binaries and databases.

The original/composite archive SHA-256 is `08bda865f29e4472535dbfb6974b3864d03862433c29d238c419cc0010f2db15`; the supplemental archive is `7cb98a232110de005259bf4c851e9d86d4951ae4245f2a21ef3224e3870aa58a`. Collection verified 57 distinct database paths and 279 exited process identities. Original/composite database records overlap; they are not 88 distinct executions.

The minimum sampled benchmark free space was 34.72 GiB and minimum sampled host MemAvailable was 5.02 GiB. The last collection on 2026-10-29 recorded 37,298,245,632 bytes free on the benchmark filesystem and 2,542,931,968 on boot, preserving the ten-GiB benchmark reserve. These are dated observations; recheck capacity before new runs. No host ARC/swap policy, production data, engine defaults, main branch, merge, or deployment changed.

## Interpretation limits

Capacity and latency refer to this instrumented loopback QUIC/TLS workload, including its transport/runtime and the fixed eight-request outer admission. The existing memory-diagnostic label remains applicable: these observations do not replace direct-path throughput/latency evidence or establish a production tenant maximum.

Kernel peaks cover cell/epoch lifetimes, including opening, preflight, and drain. Sampled RSS and periodic pressure have different scopes and can miss short peaks. Native per-database JSON observations and B-link global counter observations have different retained representations. Their 8 MiB serialized bound is not exact heap. The one-tenant cadence sensitivity does not quantify instrumentation effects at the boundary. No corrected or extrapolated engine-only memory number is reported.

The short Get recipe accesses only part of each seed dataset: sixty-second E2 cells offered Get to 1,048 distinct seed rows per tenant (51.17%), and E3 to 1,200 (3.66%). Query and mutations have small fixed working sets. The 180-second E2 recipe reaches every seed row with Get. Full-population oracle reads are outside the offered interval. The largest seeded value payload is 128 MiB, below the 256 MiB backend maximum; larger-than-budget logical datasets are not established by these cells.

Source-confirmed whole-tree residency makes B-link's opening footprint data-dependent, but the observations do not isolate decoded pages, allocator retention, recovery buffers, catalogs, old versions, or checkpoint debt. A high/OOM event is not a retained-heap profile. Cold-storage misses and host ARC enforcement are not established.

Filesystem phase snapshots are reachable-file boundary observations, not continuous temporary-space peaks or physical returned capacity. Whole-device writes include unrelated host work and are not backend write amplification. Reopened full-population validation is not isolated recovery latency or new crash-recovery evidence. The hot overwrite workload and short maintenance windows do not establish indefinite stability, growing-key churn, long-reader behavior, or staggered-demand fairness.

Issue #126 remains open. No default, merge, deployment, or production adoption is promoted.
