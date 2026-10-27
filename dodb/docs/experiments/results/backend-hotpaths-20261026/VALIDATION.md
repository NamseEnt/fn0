# Validation and limits

## Functional validation

The frozen candidate source is `3b94ee88eb4e7c02416cde14d3ac48c55e45b991`. On the designated OCI AArch64 host and ZFS dataset, twenty optimized executables pass **343 tests, zero failures and four existing ignored probes**. The suite includes both workload drivers, shared read/transaction/history contracts, transport, incomplete-transaction recovery, modeled I/O failures, ordinary segmented recovery/checkpoint SIGKILL, and immutable-base synchronous/background failure/SIGKILL boundaries.

The new B-link regression moves the open database directory temporarily and verifies Get, Query, Scan and condition-only transactions from the cached shard. It restores the path and verifies shutdown/reopen across legacy, segmented and immutable-base layouts. Additional tests cover absent-tenant reads without file creation or cached absence, capacity enforcement and concurrent initialization sharing one shard.

The new native regression verifies that lock-only acquisition excludes writers for present and missing keys, and that both rollback and commit release those locks. It performs a subsequent successful write and checks the resulting value/revision. Existing shared contracts cover conditional keys, overlapping condition/mutation keys, failed predicates without mutations, tombstones/ABA, transaction widths and reopen behavior.

Release type checking passes before the optimized build. Source audit identifies only the four intended production source files plus one inherited test-only strengthening relative to the frozen control. No storage format, WAL synchronization, checkpoint policy, admission, grouping or workload-driver source changes are present.

## Static checks

Formatting checks for the changed server/RocksDB packages and task diff-whitespace checks pass. A workspace-wide formatting probe encounters existing vendored Deno formatting differences; no vendor formatting changes are made.

Strict Clippy is not clean. Dependency-inclusive checking stops at the twelve existing storage-library findings. Package-scoped strict checking then stops at three existing server-library findings. To inspect both selected packages completely, ordinary package-scoped Clippy is recorded as structured diagnostics on the candidate and the exact frozen control, without suppressing any lint. Both emit 62 diagnostic occurrences, including repeated lib/test diagnostics. Matching diagnostic code/message, file and primary source text while ignoring source positions yields **zero new diagnostics**. Original strict failures, full JSON diagnostic logs and the comparison are preserved. This is not reported as a clean strict-Clippy pass.

## Performance evidence

Preflight is correctness/resource evidence and does not contribute to performance medians. Non-preflight comparisons require three complete repetitions for each backend/version/workload, matching fixed per-client input prefixes, complete latency observations, zero unintended errors and final/reopen values and revisions. B-link additionally requires completed checkpoint drain, checkpoint coverage and acknowledged writes during checkpoints. Resource ceilings and reserved capacity remain as declared in the plan.

All performance measurements are unprofiled. Throughput units differ by workload: RW80 API calls and E successful width-16 batches. Per-run percentile medians are reduced independently. RW80 histogram maxima are occupied-bucket upper endpoints; E maxima are exact recorded values. Process-group RSS includes driver/runtime/seeding/validation. Combined reopen/oracle time includes a complete state check and is not pure recovery. Faster fixed-duration runs can execute more driver/oracle work. Shared resource ceilings do not establish equal actual engine-memory use.

Database compression, hash verification, archive relocation, result reduction and evidence compression run outside workload execution. Every new completed database is retained as a verified lossless archive on the host. Existing experiment databases, source checkouts and original executables are preserved.

## Rejected setup/build attempts

An initial source fetch encountered `early EOF` before its bundle transfer finished. An early optimized build was interrupted after source review found a missing test import. A subsequent build exposed test helper access from the wrong module; the corrected frozen candidate passes release type checking and all functional tests. These attempts started no performance workloads and their records remain separate from accepted evidence.

The first Clippy invocation lacked the pinned toolchain's Clippy component. The component was installed, and subsequent strict findings were retained and compared with the control as described above. No lint suppressions, unrelated lint repairs or storage/default changes are introduced.

## Independent offline result

All 956 inventoried original files verify byte for byte. The four preflight, twenty-four short and twenty-four sustained cells reproduce their original summary objects exactly. Cross-stage full-window input prefixes, every measured source/binary identity, latency counts, final/reopen state, resource ceilings and checkpoint coverage/overlap pass. Recounting the twenty functional logs reproduces 343 passed, zero failed and four ignored. Both builds use the same RocksDB static archive; structured package diagnostics reproduce zero new occurrences. All 52 new database archives remain preserved on the host. See [the verification record](evidence/offline-verification.json).
