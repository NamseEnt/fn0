# RocksDB Condition-Key Shared-Lock Proposal

## Status

An opt-in candidate implementation is explicitly authorized for correctness review. The candidate keeps `Exclusive` as the default and does not change the production lock policy or default configuration. Implementation and required gates are tracked in [CONDITION-SHARED-LOCK-DESIGN.md](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/CONDITION-SHARED-LOCK-DESIGN.md), with pinned native behavior in [ROCKSDB-LOCK-REVIEW.md](results/rocksdb-contract-baseline-20261016/condition-shared-lock-20261019/ROCKSDB-LOCK-REVIEW.md). At the current checkpoint, correctness gates and performance comparisons remain pending.

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

## Required correctness gates before performance testing

1. Verify against the pinned RocksDB 11.8.1 implementation that a shared `GetForUpdate` lock is retained through transaction commit/rollback and prevents conflicting writes while the condition is evaluated and mutations commit.
2. Prove that any key appearing in both conditions and mutations is exclusively locked before its state is read; no shared-to-exclusive promotion is permitted.
3. Add deterministic concurrency tests showing multiple condition-only transactions can concurrently hold shared locks on the same present and missing keys, while a writer to either key waits or conflicts and cannot commit through the held condition lock.
4. Test the inverse order: a writer holding an exclusive lock prevents a later condition check from observing stale state or committing based on it.
5. Test atomic multi-key transactions where a condition-only key is shared-locked while other keys are mutated; verify rollback and conflict paths leave no partial writes.
6. Cover duplicate conditions/mutations, condition-only transactions allocating no revision, increasing revisions, missing-key behavior, delete/recreate ABA, same-transaction revision sharing, restart/no-reuse, snapshot reads, and tenant isolation.
7. Exercise lock timeout, rollback, cancellation, shutdown, and concurrent key acquisition in deterministic schedules; verify the tested schedules make progress and leave no lock leak. Do not claim universal deadlock freedom, writer priority, or starvation freedom.
8. Run the shared contract, restart, process-crash, and independent replay gates. State explicitly that process-crash tests do not prove power-loss behavior.
9. Only after every gate passes, use identical inputs and the existing full-window harness to compare the current policy and candidate across the approved read/write mixes and client counts. Record lock waits, conflicts, throughput, p50/p95/p99, CPU, RSS, fairness, and engine health. Diagnostic instrumentation must remain off in the performance runs.

## Decision boundary

The measured condition-key wait is a reason to investigate, not proof that changing lock mode improves end-to-end behavior or preserves DODB semantics. Keep production locking unchanged until the pinned RocksDB behavior is verified and every correctness gate passes. Adoption remains a separate root decision after the approved A/B comparison. Any change to lock ordering, transaction isolation, public semantics, default settings, or durability is outside this scope and requires a separate decision.
