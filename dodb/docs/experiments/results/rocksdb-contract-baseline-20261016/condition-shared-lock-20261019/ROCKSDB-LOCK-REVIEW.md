# RocksDB 11.8.1 Point-Lock Review

## Pinned source

- Upstream checkout: `/bench/zfs/db/product-build/rocksdb-source`
- Commit: `abeebd9630f11bd08c28b7bd43c7bdfc62050654`
- Tree: `061f8e3ce4ac04b1e9420ff92f6ed3c9b841b806`
- Local Cargo source: `librocksdb-sys-0.19.0+11.8.1/rocksdb`

The inspected local source files matched the pinned OCI checkout by SHA-256:

| File | SHA-256 |
| --- | --- |
| `include/rocksdb/utilities/transaction.h` | `cfb02ba4730ce59dee305bedd520024918600f61106ade119a358a5ba3d31d94` |
| `utilities/transactions/transaction_base.cc` | `d48578742f171536232e78169b9afe862080fe073870ce9d36f220e76cc08d7e` |
| `utilities/transactions/pessimistic_transaction.cc` | `6be3378342f7752a197bab4d29384fc8284938ff71176b2309334a918c1e46b6` |
| `utilities/transactions/lock/point/point_lock_manager.cc` | `27c1c27541190d6bc39dd7b564a3d308b38d4edfa90037b4eee95810d708a74f` |

## Observed behavior

`Transaction::GetForUpdate` exposes an `exclusive` argument that defaults to `true`. `TransactionBaseImpl::GetForUpdate` acquires the point lock through `TryLock` before reading the value. The point lock is therefore associated with the encoded key even when the database record is absent.

`PointLockManager::AcquireLocked` permits another shared request when the existing key lock is shared. An exclusive request conflicts with holders of a shared lock, and a shared request conflicts with an exclusive holder. The waiting path retries `AcquireLocked` after condition-variable wakeups; it does not reserve the key for an already-waiting exclusive transaction. A newly arriving shared request can therefore join an existing shared lock while an exclusive waiter is pending. This implementation does not establish writer priority or starvation freedom.

The pessimistic transaction retains tracked locks until commit, rollback, or destruction. Commit and rollback clear the tracked locks. This establishes the native lock lifetime used by the candidate, not the correctness of DODB transaction classification or validation.

## DODB integration constraints

The candidate mode uses shared locks only for condition-only user keys. Every mutation key, including keys also used by conditions, must request an exclusive lock on the first acquisition. Metadata keys remain exclusive. The implementation does not upgrade locks or change transaction ordering, validation, revision allocation, admission, or WAL settings. The existing Exclusive mode remains the default.

The bounded correctness tests can establish simultaneous shared holders and writer blocking/progress after release for the tested schedules. They do not prove deadlock freedom under arbitrary schedules, writer priority, or starvation freedom. Immutable condition-key workloads also cannot establish writer fairness under a benchmark.
