# Frozen raw-observation comparison matrix

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Run only after clean-source validation of `023b18bd8921b593f243f59b9eaa1576125c6572` and final default-feature linking. Preserve baseline runtime `aaa405655` and every earlier accepted/failed cell. The only source changes are retained observation representation, streamed worker-health serialization, its tests, and the pinned serde_json raw-value feature. Engine code/configuration, source-native archive, durability, transactions, revisions, key/value recipe, load, admission, budgets, deadlines, and host ARC/swap policy remain unchanged.

The complete target cache was checksum-verified and relocated to ZFS while retaining its original logical Cargo path. Record the resolved target location and relocation receipt in provenance. This changes host disk occupancy and build provenance; do not attribute every timing difference across dates to the representation change. No builds or evidence compression may overlap workload cells.

## Exact sequence

Use the original controller's base settings: maximum/high 256/192 MiB across the enclosing cell and every backend epoch, zero backend swap, seed 20261027, 512-byte values, mixed profile, width mix 80/15/5, fifty arrivals/s per tenant, synchronized absolute-deadline lanes, queue bound eight, global in-flight eight, five-second request timeout, fifteen-second drain and worker shutdown gates, and both full-population oracles. B-link and shared-reference options remain the same as [DENSITY-CAPACITY-PLAN.md](DENSITY-CAPACITY-PLAN.md). All populations below are registered=active, with no opened-idle tenants.

0. After cache relocation, run a new current-host control with the archived default-feature `aaa405655` binaries: E2 one tenant, sixty seconds, paired 1,000 ms cadence followed by paired 100 ms cadence. Verify the archived hashes before use and temporarily select the matching clean source checkout for provenance, restoring the validated candidate checkout afterward. Do not rebuild the baseline or overwrite its earlier evidence. This controls the cadence comparison's date/cache-location context; it does not recreate every large-population baseline condition.
1. With the validated candidate binaries, E2 one tenant, sixty seconds: paired 1,000 ms cadence, then paired 100 ms cadence. This repeats the scientifically changed representation against the old cadence cohort; it is not a reporting repair.
2. E2 twenty-eight tenants, sixty seconds, 1,000 ms cadence: three fresh paired repetitions.
3. E3 five tenants, sixty seconds, 1,000 ms cadence: three fresh paired repetitions.
4. E3 eight tenants, then six tenants, thirty seconds and 1,000 ms cadence: one fresh pair at each population. These are new-representation variants of the earlier failed conditions. Keep an operational failure even if no offered load starts; never relabel it as a zero-throughput interval. Do not rerun an unchanged failure to obtain a median.
5. E3 eight tenants, sixty seconds, 1,000 ms cadence: three repetitions only for routes that completed their eight-tenant screen and have no operational failure at six/eight tenants in this sequence. Run paired repetitions when both routes remain eligible; otherwise independently confirm the surviving route. If neither remains eligible, write an omission receipt. Stop for review if eligibility is non-monotonic or a survivor unexpectedly fails operationally; do not repeat until success.
6. E2 twenty-eight tenants, 180 seconds, 2,000 ms cadence: one maintenance pair. Keep this duration/cadence cohort separate from sixty-second confirmations.

Rotate backend order for every pair. This sequence's paired cadence and confirmation order has the same alternating parity as the corresponding baseline cohorts. Shared-only survivor confirmation has no invented paired offered-work comparison against an aborted backend.

## Acceptance and stop boundaries

The experimental objective is unchanged: every tenant schedules/admit/succeeds at the full offered count, no errors/conflicts/timeouts/drops, at least 99% completed by offer end, and per-operation scheduled-to-completion p99 below the aligned 8.388608/16.777216/67.108864 ms Get/Query/Write thresholds with at least one hundred observations per operation. Preserve raw histograms and report ranges rather than interpolated exact p99 values.

Require source/binary/controller identities, native sharing/options, verified pre-runtime cgroup membership, empty final groups, untruncated observations, accounting/fingerprints, both oracles, and clean shutdown for accepted completions. Known setup shutdown-deadline failures require the exact stderr, recorded high events, no OOM, and unused measurement/oracle epochs. Unexplained errors, uncertain mutations, a controller timeout, mismatched provenance/accounting, or reserve breach stop for review. Keep the original operational-deadline classification distinct from kernel OOM.

Require twelve GiB benchmark free before a cell and stop owned processes below ten GiB or 1.5 GiB host MemAvailable. The original twenty-minute controller deadline remains. Freeze new binaries in a unique namespace and keep the driver outside backend groups. Preserve all databases, cgroups, logs, and rejected attempts.

Compare matched recipes/source settings and report actual realized counts. Separately report memory peaks through the offer boundary, final epoch peaks, after-offer peak increases, sampled backend/driver RSS, serialized frame volume/count/gaps, CPU, latency, native usage, and operational failures. Verify both report JSON schema and original artifact preservation. These new cohorts do not retroactively replace the baseline or establish an engine-only/production density winner without the broader remaining gates.
