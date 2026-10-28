# Bounded active-tenant capacity under a common backend budget

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: the frozen main sequence is still collecting confirmation and maintenance evidence. This document is an in-progress reduction, not a completed adoption decision.

## Fixed conditions

Runtime source is `aaa40565529ca92150df5d94e19b02ca93859f71`. Relative to the accepted memory-pressure source, it only summarizes periodic checkpoint detail; engine behavior and defaults remain unchanged. Default-feature binaries are used; A1 stays disabled. All 68 clean-source checks and both default/A1 compatibility links passed. Advisory Clippy retained exactly the same seventeen existing message/path findings, with no new finding or suppression.

The designated two-core OCI/ZFS host uses a common 256 MiB maximum and 192 MiB high boundary for the whole cell and every backend epoch, with zero backend swap. The driver, previously charged shared pages, and uncharged host ARC remain outside this budget. Host policy is unchanged. This is an instrumented backend-cgroup comparison, not a machine-wide memory ceiling or isolated engine-heap comparison.

Every active tenant receives fifty scheduled API calls/s through one sequential lane, with synchronized tenant deadlines, a queue bound of eight, and a shared in-flight maximum of eight. The mixed profile is Get/Query/write=40/20/40; writes are half conditional with widths 1/4/16 at 80/15/5, and ten percent of mutations are Delete. Values are 512 bytes. E2 seeds 2,048 rows/1 MiB of value payload per tenant; E3 seeds 32,768 rows/16 MiB. Query always returns the first sixteen seeded rows. Get walks the deterministic seeded-row sequence. Writes use a separate fixed sixteen-key overwrite/delete/reinsert lane.

B-link explicitly uses segmented coalesced bases, background checkpoints, 256 KiB WAL segments, and a 1 MiB checkpoint threshold. The TransactionDB reference uses a shared 64 MiB cache and 32 MiB WBM, 1 MiB tenant write buffers, one flush/compaction thread each, 64 open files per tenant, and disabled WBM writer stalls. Actual native sharing/options are verified; this is not maximum-tuned RocksDB. The native service uses blocking dispatch while B-link uses its direct async service path. Both retain the same outer admission and durability/transaction contract.

The experimental objective requires every tenant to finish every scheduled request successfully, with no drops/conflicts/errors/timeouts, at least 99% completed by offer end, and per-tenant scheduled-to-completion p99 below 8.388608 ms Get, 16.777216 ms Query, and 67.108864 ms Write. At least one hundred observations per operation per tenant are required. These thresholds align with the existing histogram buckets and are not product requirements or interpolated exact percentiles. Full-population value/state validation, clean reopened validation, accounting, identity, sharing, and shutdown gates are separate requirements.

## Current observations

For E2, both routes passed all three sixty-second repetitions at 24 and 28 active tenants. Both failed the Get objective in their single thirty-second 32-tenant screens. Counts 29-31 and sixty-second 32-tenant repetitions were not tested. Shared RocksDB's thirty-second 28-tenant screen failed Get p99, while its three sixty-second repetitions passed; these are different duration cohorts and must not be pooled as four identical trials. There is no demonstrated E2 active-count advantage at the confirmed counts.

For E3, both routes passed the one-, two-, four-, and five-tenant thirty-second conditions. Shared RocksDB also passed the six- and eight-tenant screens. The five-tenant confirmation cohort is still running.

B-link at eight E3 tenants failed with kernel OOM during setup. Its cell lasted 1,023.29 seconds including failure detection; measurement never began. B-link at six E3 tenants completed seeding but its setup worker exceeded the existing fifteen-second shutdown deadline. That cell lasted 346.43 seconds, peaked at 245,739,520 bytes, recorded 18,569 parent high events, and recorded no maximum/OOM events. Measurement never began. Keep these operational failures separate from a zero-throughput result, a workload timeout, or a corruption claim.

## Preservation and continuation

The original namespace is `capacity-aaa405655-r1`. It stopped after the six-tenant deadline because the initial reducer handled OOM but did not classify that operational timeout. The raw reports, original statuses, logs, and databases remain immutable. The source of the rejection was reviewed before continuation.

`capacity-aaa405655-r2` is an explicit composite evidence view. It reuses the original 36 raw reports byte-for-byte and runs only previously unexecuted cells. The known deadline receives a derived classification with its original review error retained. A preflight reviewed all 36 reports and verified rejection of unrecognized stderr, mismatched source, and missing high-event evidence. `reused_existing_cases` and `executed_new_cases` distinguish evidence reuse from actual execution; do not add the two namespaces' case counts together.

See [the main plan](DENSITY-CAPACITY-PLAN.md), [the continuation protocol](CAPACITY-CONTINUATION.md), [the frozen conditional supplement](CAPACITY-SUPPLEMENT-PLAN.md), and [the source audit](CAPACITY-SOURCE-AUDIT.md). The supplement independently confirms a surviving eight-tenant E3 route and observes maintenance at the actual confirmed E2 boundary when the main sixteen-tenant retention cap prevents doing so.

## Interpretation limits

Kernel peaks cover cell/epoch lifetimes, including opening, preflight, and drain. Sampled RSS and periodic pressure have different scopes and can miss short peaks. Native per-database JSON observations and B-link global counter observations have different retained representations. Their 8 MiB serialized bound is not exact heap. The one-tenant cadence sensitivity does not quantify instrumentation effects at the boundary. No corrected or extrapolated engine-only memory number is reported.

The short Get recipe accesses only part of each seed dataset; Query and mutations have small fixed working sets. Source-confirmed whole-tree residency makes B-link's opening footprint data-dependent, but the observations do not isolate decoded pages, allocator retention, recovery buffers, catalogs, old versions, or checkpoint debt. A high/OOM event is not a retained-heap profile. Data larger than the backend budget, cold-storage misses, and host ARC enforcement are not established.

Filesystem phase snapshots are reachable-file boundary observations, not continuous temporary-space peaks or physical returned capacity. Whole-device writes include unrelated host work and are not backend write amplification. Reopened full-population validation is not isolated recovery latency or new crash-recovery evidence. The hot overwrite workload and short maintenance windows do not establish indefinite stability, growing-key churn, long-reader behavior, or staggered-demand fairness.

Issue #126 remains open. No default, merge, deployment, or production adoption is promoted.
