# Condition-Key Shared Lock Candidate

## Scope

The candidate adds an explicit `ConditionKeyLockMode` to the RocksDB service and synchronous research tenant. `Exclusive` remains the default. The workload driver selects `exclusive` or `shared` with `--condition-key-lock-mode` and records the selected mode in its report. The RocksDB-direct path receives the same setting; B-link reports the setting as not applicable.

## Lock classification

Each transaction uses one sorted `BTreeMap` keyed by canonical encoded user key to form the condition/mutation union. A condition-only entry requests a shared `GetForUpdate` lock in Shared mode. Adding a mutation for the same key marks that entry exclusive before acquisition. All mutation keys are exclusive, including condition-and-mutation overlap. Revision metadata reads remain exclusive in both allocators. The wrapper continues to validate conditions after locking the complete key union, preserve the current key order, and commit the transaction atomically with synchronous WAL.

The implementation does not upgrade locks. It does not alter retry behavior, isolation, revision allocation, tenant admission, lock manager, public transaction APIs, or service defaults. Unconditional writes have no condition-only key and therefore do not use shared locks for their mutation keys.

## Native lock behavior and limits

The pinned RocksDB 11.8.1 source review is recorded in [ROCKSDB-LOCK-REVIEW.md](ROCKSDB-LOCK-REVIEW.md). `GetForUpdate` acquires the point lock before looking up the value, so missing keys receive the same key-identity lock protection. Shared holders coexist, while an exclusive request conflicts with them. Native lock tracking retains the locks through commit, rollback, or transaction destruction.

The inspected point lock manager retries acquisition after a condition-variable wakeup but does not reserve a key for an already-waiting exclusive transaction. A new shared request may join an existing shared lock while an exclusive request waits. No writer-priority or starvation-freedom guarantee is established. The deterministic tests demonstrate writer blocking while shared holders remain and progress after release for bounded test schedules; they do not prove fairness or deadlock freedom for arbitrary schedules. The immutable condition keys in the performance workload cannot establish writer fairness.

## Correctness gates and screening checkpoint

Before any A/B measurement, run the native deterministic gates for present and missing condition keys, two simultaneous shared holders, exclusive-writer blocking and post-release progress, writer-first stale-condition rejection, condition/mutation overlap exclusion, condition-only no-revision behavior, atomic rollback, lock timeout, and lock release after timeout and rollback. Run the existing transaction, reopen, revision no-reuse, read-snapshot, tenant lifecycle, history, and process-crash contracts in both lock modes and both revision allocator modes where applicable.

The deterministic and shared-contract gates passed. The release suite passed 35 tests at source `15e0077b4e648930c7e01def84317b566f4e18e3`. At final source `88b43098dab8736363d8c36446bbede27e1b77e1`, a follow-up release crash gate passed all four tests after adding present and missing condition-only keys to the crash transaction. The source changes after the 35-test run are test-only: `open_with_allocator` is now test-only, and the crash scenario adds those two shared-lock conditions. See [the final screening report](SCREENING-RESULTS.md) and [preserved OCI evidence](oci-evidence-88b43098d/).

The approved 44-case screen completed successfully: 42 performance runs with diagnostics disabled and two separate diagnostic runs. Shared mode had higher observed workflow throughput in the tested write-heavy mixes, but the five-second screen is preliminary. The approved longer confirmation and representative five-minute comparison remain pending after the separately scoped engine-health observation harness gate; Exclusive remains the default.

These tests cover their controlled schedules only. They do not prove universal fairness, starvation freedom, or deadlock freedom. The immutable condition-key performance workload cannot establish writer progress under changing conditions. The diagnostic workload measures bounded contention from condition-only readers and must not be generalized to all key roles.
