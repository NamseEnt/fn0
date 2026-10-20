# Minimal Native Adapter Reuse and Bypass Map

## Reused

- Public DODB request, response, key, revision, condition, mutation, error, and execution-budget types.
- Deterministic workload input generation and the existing final-state, replay, reopen, and report validators where their backend interfaces permit.
- RocksDB 11.8.1 build integration and the existing native TransactionDB C ABI for database open/close, transaction begin, `GetForUpdate`, put, commit, rollback, snapshots, and iterators.
- Existing physical user-key encoding, value/tombstone encoding, condition matching, key validation, and response-budget rules as pure helpers.
- The already-reviewed 4,096-revision reserved-range policy: reserve a durable high-water range in its own synchronous TransactionDB transaction, publish the range only after commit, allow gaps on failure, and abandon unused tails on reopen.

## Implemented independently

- Tenant database ownership and open/reopen lifecycle for the benchmark-only service.
- `Get`, `Put`, `Delete`, and `Transact` request execution. The adapter creates native transactions and directly invokes TransactionDB `GetForUpdate`, writes, rollback, and commit operations.
- Revision range state and durable reservation flow, separate from `TenantDatabase` state and methods.
- Query and Scan execution over one RocksDB snapshot iterator per call, with DODB ordering, exclusive cursor, limit, tenant prefix, and response-budget behavior.

## Bypassed

- `TenantDatabase` and every method on it, including its transaction executor, range allocator state, and read/query/scan methods.
- Existing DODB transaction execution and service admission paths; the benchmark driver concurrency and any admission limit are recorded as benchmark settings rather than attributed to the adapter.
- Any new lock manager, transaction engine, or product-facing configuration/default change.

## Contract boundary

Conditions and mutations lock the union of their keys through native `GetForUpdate`, including condition-only dependencies. All mutations in one request are written in one native transaction and carry one allocated revision. Tombstones retain the last revision for ABA detection. Reserved revision ranges preserve uniqueness across restart without holding a global counter lock during each user commit. Successful user writes use synchronous WAL. Query and Scan take independent per-call snapshots. Tenant key prefixes isolate data, and responses are checked against the configured execution budget before returning.

The shared native C ABI is lower-level FFI, not transaction orchestration: the adapter itself constructs each transaction, selects and reads its lock set, evaluates conditions, allocates revisions, encodes values, issues writes, and commits or rolls back.
