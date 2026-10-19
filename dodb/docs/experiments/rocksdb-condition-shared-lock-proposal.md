# RocksDB Condition-Key Shared-Lock Proposal

## Status

The opt-in candidate is implemented and its required correctness gates passed. The 44-case five-second comparison completed successfully; it favors Shared in the tested write-heavy mixes. This remains short-screen evidence. The engine-health observation harness gate precedes the approved 30-second confirmation and representative five-minute comparison; both measurements and the adoption decision remain pending. `Exclusive` remains the default and the production lock policy is unchanged. See the [candidate design and gate status](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/CONDITION-SHARED-LOCK-DESIGN.md), [screening results](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/SCREENING-RESULTS.md), and [pinned native behavior review](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/ROCKSDB-LOCK-REVIEW.md).

## Evidence and motivation

The diagnostic screen at source `15762ba40725a16f09de20d15ffcb97ebb42cef4` recorded the `rw20`, 16-client, `ReservedRanges`, shared-16 case in [report r56](results/rocksdb-contract-baseline-20261016/gate/rescreen-r3-results/reports/r56-rw20-c16-rocks-reserved-shared16-diagnostic-diagnostic.json). It attributes lock-wait values to RocksDB 11.8.1 `PerfContext` deltas around `GetForUpdate` calls. Condition-only keys averaged 3.9765 ms of native lock wait over 12,403 observations; mutation-only keys averaged 0 ms over 45,757 observations. The condition-only p95 and p99 histogram estimates were 33.554 and 67.109 ms. These are diagnostic observations from one short run, not a general performance claim.

The diagnostic was a mixed `rw20` workload, while the raw screen below is unconditional and has no condition-key comparison; do not interpret the diagnostic gap as a measured raw-to-wrapper cost. The harness reports roles by whether a key appears among conditions, so a key that is both conditioned and mutated can be counted in the condition-key role in a general workload. In this particular diagnostic workload, mutation ordinals start at `1 << 60` while condition/missing-key ordinals are below `100,000`, so the measured condition-key set is disjoint from the mutation set and supports the narrower condition-only interpretation for this run.

The current synchronous transaction path forms the sorted union of condition and mutation keys and calls `GetForUpdate` on each key in `dodb/crates/dodb-rocksdb/src/lib.rs`. The C++ shim currently uses the default `exclusive=true` overload in `dodb/crates/dodb-rocksdb/shim/transaction_shim.cc`. RocksDB 11.8.1 exposes an `exclusive` argument on `Transaction::GetForUpdate`; `false` requests a shared lock. A candidate could request shared locks for keys that are condition-only in a transaction, while retaining exclusive locks for keys that are mutated. Keys that are both conditioned and mutated must be exclusive from their first lock acquisition.

## Authorized candidate scope

Change only the lock mode selected for a transaction's user keys:

- Condition-only keys: acquire a shared `GetForUpdate` lock.
- Mutation keys, including keys also present in conditions: acquire an exclusive `GetForUpdate` lock.
- Keep the existing sorted key acquisition order, transaction boundaries, commit behavior, rollback behavior, revision allocation, synchronized WAL, semaphore/admission settings, and defaults.
- Do not change missing-key encoding, logical revisions, atomic multi-key commits, or public API behavior.

This proposal does not authorize adding a separate lock manager, lock upgrade path, retry policy, optimistic read path, or new database algorithm. If RocksDB cannot provide the required behavior without one of those changes, stop and return a design question.

## Correctness gates completed before the short screen

1. Verify against the pinned RocksDB 11.8.1 implementation that a shared `GetForUpdate` lock is retained through transaction commit/rollback and prevents conflicting writes while the condition is evaluated and mutations commit.
2. Prove that any key appearing in both conditions and mutations is exclusively locked before its state is read; no shared-to-exclusive promotion is permitted.
3. Add deterministic concurrency tests showing multiple condition-only transactions can concurrently hold shared locks on the same present and missing keys, while a writer to either key waits or conflicts and cannot commit through the held condition lock.
4. Test the inverse order: a writer holding an exclusive lock prevents a later condition check from observing stale state or committing based on it.
5. Test atomic multi-key transactions where a condition-only key is shared-locked while other keys are mutated; verify rollback and conflict paths leave no partial writes.
6. Cover duplicate conditions/mutations, condition-only transactions allocating no revision, increasing revisions, missing-key behavior, delete/recreate ABA, same-transaction revision sharing, restart/no-reuse, snapshot reads, and tenant isolation.
7. Exercise lock timeout, rollback, cancellation, shutdown, and concurrent key acquisition in deterministic schedules; verify the tested schedules make progress and leave no lock leak. Do not claim universal deadlock freedom, writer priority, or starvation freedom.
8. Run the shared contract, restart, process-crash, and independent replay gates. State explicitly that process-crash tests do not prove power-loss behavior.
9. All listed correctness requirements passed on the pinned release gates. The exact-source follow-up crash gate covers Shared locks for an existing untouched condition key and a missing condition key. The 44-case screen used identical inputs and full-window validation across the approved read/write mixes and client counts; diagnostic instrumentation stayed off for all 42 performance cases and ran separately for the two diagnostic cases. Full details and evidence links are in [SCREENING-RESULTS.md](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/SCREENING-RESULTS.md).

## Decision boundary

The measured condition-key wait and short-screen throughput are reasons to continue confirmation, not proof of universal benefit or fairness. Keep production locking unchanged through the approved 30-second and five-minute comparisons and the separate adoption decision. Any change to lock ordering, transaction isolation, public semantics, default settings, or durability remains outside this scope and requires a separate decision.
