# Separate arrival phase from resident-tenant capacity

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

The primary owns design and acceptance. All implementation is delegated to GPT-6 Luna at high reasoning effort, with concise summaries only. Preserve this plan before editing source; freeze the exact runtime matrix after clean-source checks and live sizing.

## Decision question

The source-023 E2 cohort at twenty-eight active tenants completed correctly but did not reconfirm the earlier SLO acceptance. Source inspection shows every tenant producer uses the same start instant and `floor(operation_number * 1e9 / rate)` deadline offset. At fifty calls/s, all tenants nominally arrive in the same twenty-millisecond waves under the unchanged global in-flight cap of eight.

Test whether deterministic within-period phase spreading changes the accepted active count for small E2 stores under the same resource/SLO policy. Do not assume either backend benefits. This is an arrival-pattern comparison, not a fix for the E3 setup watchdog: that failure precedes offered load. Do not rerun those unchanged E3 failed conditions using this option.

## Approved source implementation

Change only the experimental density driver, its config/scheduler support, and meaningful tests. No engine/server/storage, worker, allocator, cache/WBM, checkpoint, revision/condition, durability, admission/queue, request timeout, drain, shutdown, or dependency change is approved.

Add `--tenant-arrival-phase synchronized|staggered`, default synchronized. Staggered applies only to fixed-per-tenant topology; reject it for fixed-total. Default deadline calculation, request recipe, deterministic fingerprints, and realized scheduled fingerprints must remain exactly compatible with b226. Effective configuration must report the selected mode; fixed-total must clearly report that tenant phase does not apply.

For active tenant number `tenant_number` in one through `active_tenants`, use:

`phase_offset_ns = floor((tenant_number - 1) * 1_000_000_000 / (rate * active_tenants))`

Staggered deadline is the original absolute per-operation deadline plus this constant nominal offset. Compute in checked/wide integer arithmetic and retain the original overflow handling. The global start/end window, bounded nonblocking admission, queue count, global semaphore count, request preparation, sequential consumer, drain, and scheduled-to-completion latency origin remain unchanged. This is neither random jitter nor a guarantee of physical dispatch spacing below timer resolution.

Expose each tenant's nominal phase offset with its existing report row. Preserve input/request identity across modes. Encode the staggered mode/formula/active count into the deterministic schedule identity and encode nominal deadline offsets into its realized scheduled fingerprint. Leave the synchronized identity/fingerprint bytes unchanged. Cross-backend comparisons within a mode require matching schedules; cross-mode comparisons require matching request/count recipes while expecting different phase/schedule identities.

The empirical b226 synchronized recipe for mixed profile, Mixed width, one client, rate fifty, duration 60,000 ms, and no operation count has schedule fingerprint `fnv1a64:e913b4f65d1ccd15`. Its five-tenant / 32,768-row / 512-byte input fingerprint is `fnv1a64:5b2fc19f8839634b`. Use this preserved external regression fixture rather than a self-derived expected value.

## Checks

Test default/explicit mode parsing, invalid fixed-total use, exact four-tenant fifty-Hz offsets 0/5/10/15 ms, single-tenant behavior, rational-rate rounding/overflow bounds, common offer-window inclusion, and unchanged request recipe. Verify synchronized legacy fingerprints and the staggered schedule distinction. Use whole-period duration examples to prove equal per-tenant offered counts across modes; arbitrary nonintegral windows must follow actual deadline inclusion rather than invent equal counts. Retain existing queue/drop/accounting/latency coverage and global in-flight bounds. Do not add tests that only restate JSON construction.

Run the accepted affected library/worker/driver/read/transaction checks, targeted formatting, advisory Clippy with no new findings/suppressions, and A1/default compatibility links with default last. Keep A1 disabled in runtime. Preserve a clean source SHA, native hashes, actual commands, failed attempts, and binaries before OCI execution.

## Runtime design boundary

Freeze tiny paired correctness cells for both modes first. Then compare current-host/current-source synchronized versus staggered E2 twenty-eight-tenant sixty-second controls with three repetitions per mode and rotating backend order. New controls use the same sampling policy and descriptor limit as the variants; do not compare across historical source/date/cadence as if only phase changed.

Use the original 256/192 MiB backend max/high, zero backend swap, fifty calls/s per active tenant, global/queue cap eight, mixed recipe/widths, key/value bytes, deadlines, native/shared settings, checkpoint configuration, complete oracles, and unchanged per-tenant SLO thresholds. For the new wide-population cohort, use a common low-overhead main cadence fixed before running, sized for the existing 8 MiB raw-observation bound. A separate longer maintenance cadence must be frozen separately.

Current root/opc soft/hard open-file limits were observed at 1,024 / 524,288. Set the new controller and inherited child soft limit to a common 65,536, preserving the hard limit, and verify an owned child sees it before workloads. Record the old/new limits. This common new-cohort policy avoids an incidental descriptor ceiling when comparing many independent native databases; it does not change the native per-database open-file option or a production default. Hold it equal for both modes and backends.

After controls, size a modest staggered E2 screening/refinement grid, initially considering 32/48/64/80/96 active tenants. Stop a route above a recorded operational failure; preserve SLO misses and choose only bounded, predeclared boundary-adjacent confirmations. Do not retry an unchanged operational failure for a median. One accepted count requires all tenants and both oracles across three fresh sixty-second cells. Report tested brackets, not an engine maximum. Test longer maintenance only at an accepted common count, with both backends and a cadence that fits retained observation bounds.

Recheck source/build/dataset space before freezing exact cells. Preserve the twelve-/ten-GiB free-space and 1.5 GiB MemAvailable guards, host ARC/swap policy, prior databases, failed attempts, binaries, and archives. The backend cgroup still excludes driver and uncharged host memory. This comparison does not complete larger-than-budget, cold-storage, growing-churn, representation-byte, or B1/B2 architecture gates. Issue #126 remains open.
