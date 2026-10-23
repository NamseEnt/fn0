# Correctness and measurement validation

## Functional results

The frozen implementation is `cc80d1fdc7a98792be1a739056a3013debf39268`. On the designated OCI AArch64 host, twenty optimized executables pass **339 tests, zero failures, four existing ignored probes**. The selected suites include storage, server and RocksDB library tests; both workload binaries; read/transaction/history contracts; transport; incomplete-transaction recovery; ordinary segmented rotation/checkpoint faults and SIGKILL; and immutable-base synchronous/background failure and SIGKILL boundaries.

New coverage verifies recovery from an empty database with a damaged disposable cache, reopening without rewriting the cache, and intermediate pack-sync failure before and after sync while preserving post-target acknowledged writes. Intermediate-sync SIGKILL cases write a pack exceeding 8 MiB, kill at the actual sync boundary, verify acknowledged values/revisions, issue another acknowledged write, checkpoint, and recover again. Existing missing/corrupt/truncated/identity-mismatched required-pack rejection, incomplete transactions, growth/reuse and concurrent rotation coverage remain enabled.

Local optimized storage/server library and focused integration validation passes **248 tests, zero failures, four existing ignored probes**, across ten executables. A separate stronger consolidation regression also passes: capture a full snapshot containing clean decoded pages, modify a previously cold page after the target, run the worker, publish, and reopen to verify the latest state. This test-only strengthening follows the frozen implementation and changes no measured engine code.

Local formatting and diff-whitespace checks pass. Strict storage-library Clippy reports twelve existing diagnostics; the unchanged `e13910390` baseline reports the same diagnostic categories and counts. No lint suppressions are added, and this is not described as a clean strict-Clippy run. Both logs are preserved.

## Performance evidence gates

Each case must pass compile-time source identity, actual binary hash, fixed client input prefixes, complete latency counts, zero unintended request errors/overloads, a full admission window and complete final/reopen value/revision oracles. Background B-link cases must demonstrate acknowledged writes during completed in-window checkpoints. The 60-second stage requires at least three completed checkpoints and the 300-second stage at least ten. Resource guards are 3 GiB process-group RSS, 4 GiB total database directory, 1500 MiB available host memory and 10 GiB free filesystem space.

The immutable primary-page image is derived from the manifest high-water mark, not the disposable `.db` file length. Required packs, WAL and in-flight files must fit that primary image plus 512 MiB additional recovery storage. External samples include every page pack and cover the entire child process. Internal `.db`-only counters do not describe the full immutable-base footprint. All quoted resource peaks are sampled peaks.

The reducer independently rechecks raw and enriched report hashes, recorded source/binary identities, command equality after removing backend-specific controls and paths, input prefixes, latency counts, state oracles, resource limits, checkpoint overlap/coverage and WAL/checkpoint/process-write accounting. Reopen time includes backend opening and the complete state oracle; it is not pure redo replay. Process `wchar` records requested bytes, not physical device traffic. Checkpoint phase times overlap active execution and are not additive latency percentiles.

## Preserved failures and exclusions

An initial setup attempt fetched the source bundle before its upload completed and failed with `early EOF`. No build or measurement started from that attempt. The completed bundle was subsequently fetched successfully into a clean detached checkout; the rejected setup record is retained. Successful evidence is never rerun merely to repair a reducer or document.

The preflight is correctness/resource evidence and is excluded from performance medians. Strict-Clippy baseline findings and local validation logs are retained byte for byte in `evidence/local/local-validation-logs.tar.gz`, with their individual hashes in `evidence/local/summary.json`. Functional crash tests establish process-crash and modeled I/O-failure behavior, not arbitrary physical power-loss behavior.

## Final offline result

All 539 OCI inventoried files and five local raw logs verify byte for byte. The six preflight, eighteen short and twelve sustained cells reproduce their original summary objects exactly. Cross-stage input prefixes match, all measured source/binary identities remain unchanged, and recounting the twenty functional logs gives 339 passed, zero failed and four ignored. The sustained B-link cells finish seventeen RW80 or thirty-five to thirty-six E checkpoints fully inside their client windows. The largest external sampling gap across all sustained cells is 1.137 seconds. See `evidence/offline-verification.json`.
