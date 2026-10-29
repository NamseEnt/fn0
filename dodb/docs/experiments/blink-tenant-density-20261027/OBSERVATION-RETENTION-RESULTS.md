# Raw observation retention and streamed health reports

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Status: implementation and clean-source checks passed; the frozen runtime comparison is in progress. Do not claim a measured memory reduction before reducing those new cohorts.

## Implementation

Source `023b18bd8921b593f243f59b9eaa1576125c6572` changes the diagnostic worker's opt-in periodic pressure storage from retained `serde_json::Value` objects to validated raw JSON frames. Each accepted frame is serialized once and retained under the existing 4,096-frame / 8 MiB serialized-byte bounds. Frame/vector/allocator overhead and temporary sample construction remain outside that byte count; it is not an exact heap ceiling.

The final worker health report streams metadata and raw frames through a buffered writer. It does not reconstruct all frame DOM objects or allocate one complete serialized worker report buffer. The parsed JSON shape, timestamps, phases, sample order, truncation, unsupported-property states, shared-resource counting, and complete final checkpoint details are preserved. Source JSON parsing remains in the driver outside the backend cgroup; total-machine cost is not removed by moving that parsing.

Only the worker, its new helper/tests, and the pinned serde_json raw-value feature changed. Engine code, defaults, caches/WBM, checkpoint policy, locks, WAL/durability, transactions, revisions, admission, budgets, and deadlines remain unchanged. A1 stays disabled in accepted runtime binaries.

## Clean-source checks and disk preparation

All 74 checks passed: the previous 68 plus six worker tests covering empty/multiple-frame schema, nested/escaped/unavailable values, count and byte boundaries, metadata misuse, and output-error propagation. Both default/A1 feature links passed. Advisory Clippy retained the same seventeen message/path findings without a new finding or suppression.

The first new feature build left only 1,123,028,992 boot bytes free. The complete approximately 4.3 GiB target cache was copied to `/bench/zfs/db/tenant-density-build-cache-20261029/target`, checksum-compared with an empty dry-run result, and activated through the unchanged logical Cargo target path. Runtime/native hashes matched before and after. Only the verified duplicate on boot was removed. The active cache, every database, frozen binary, source checkout, native archive, log, and failed artifact remain preserved. See [OBSERVATION-CACHE-RELOCATION.md](OBSERVATION-CACHE-RELOCATION.md).

After relocation, boot/benchmark free space was 5,712,830,464 / 32,584,036,352 bytes before further feature links. Build-cache occupancy is an explicit host-provenance difference; preserve it in timing comparisons.

## Executed protocol and live continuation

[OBSERVATION-RETENTION-MATRIX.md](OBSERVATION-RETENTION-MATRIX.md) freezes the comparison. Runtime source and binary identities are separate from its controller/plan commit. The current-host control uses the archived default-feature `aaa405655` binaries without rebuilding; its namespace is `observations-control-aaa405655-r1`. All four cadence cells completed and met the objective. Its source checkout was selected only for provenance and restored to `023b18bd8` afterward.

The candidate namespace is `observations-023b18bd8-r1`, controlled by `/home/opc/tenant-density-harness-20261027/oci_observations_compare.py`. Inspect `observations-runtime-r1-launcher.log`, `observations-runtime-r1-pid.json`, and `capacity/observations-023b18bd8-r1/` on the designated OCI host. The controller stops for review on `controller-failure.json`; `completion.json` marks a completed sequence. Do not overlap builds/compression/workloads or restart completed cells.

Candidate cadence checks precede three paired repetitions at the old E2/E3 confirmed counts, one new-representation attempt at each earlier failed E3 count, eligible eight-tenant confirmation, and the twenty-eight-tenant maintenance pair. Known operational failures remain failures and are not repeatedly provoked for a median. The 256/192 MiB policy, all workload settings, low/high resource guards, oracles, and original fifteen-second shutdown gate remain unchanged.

Validation and relocation evidence is in `evidence/observation-retention/validation/` with `validation-review.json`. Runtime data remains in unique OCI namespaces until collection. Keep the earlier [capacity result](DENSITY-CAPACITY-RESULTS.md) and both original archives intact; these new cohorts supplement them rather than replacing them. Issue #126 remains open.
