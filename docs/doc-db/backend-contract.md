# Document backend contract

This document fixes the backend-neutral contract implemented by `fn0-doc-db`. It describes application data access, optimistic transactions, and the administrative primitives needed for inspection and schema migration. It does not require a particular storage engine or wire protocol.

## Backends

`fn0-doc-db` provides three backends:

- `Memory` for local tests and in-process use;
- `Turso` for libSQL over Hrana HTTP;
- `Dibi` for the Dibi protocol through the fn0 host transport.

The Dibi path is:

```text
fn0-doc-db -> WIT host transport -> fn0 host -> authenticated QUIC -> Dibi -> RocksDB
```

`fn0-doc-db` does not implement QUIC, TLS, authentication, worker credentials, or tenant selection. A Dibi guest request always carries an empty tenant. The trusted fn0 host injects `project_id` as the Dibi tenant after validating the logical endpoint. Applications select the Dibi endpoint through `DIBI_URL`; the host normally supplies `dibi://fn0-db.fn0.dev`.

Dibi protocol pages are limited to 10,000 records, but the public `query`, `scan`, and `admin_scan` limits are not. The Dibi client follows cursors across protocol pages until the caller's limit is satisfied. Atomic `batch`, administrative conditional writes, and optimistic conditional commits are never split when their operation count or encoded frame exceeds a protocol limit; they return an error instead.

## Application API contract

The application API stores opaque byte values under a `(pk, sk)` string key. Backends must provide:

- `get(pk, sk) -> Option<Bytes>`
- `put(pk, sk, data)`
- `delete(pk, sk)` (deleting a missing key succeeds)
- `query(pk, after_sk, limit)`
- `scan(after, limit)`
- atomic `batch` containing puts and deletes
- explicit transactions with `get`, `put`, `delete`, `commit`, and `rollback`
- optimistic transactions with version checking and retry-compatible conflict reporting
- `execute_ops` and `DbRequest`

Values are opaque bytes. No operation may assume that a value is JSON.

`execute_ops` and `DbRequest` are request composition facilities; they do not add an implicit transaction around the operations.

## Key ordering

`query` returns keys for one `pk` in lexicographic ascending `sk` order. `after_sk` is an exclusive cursor, and `limit` is exact.

`scan` returns the complete key space in lexicographic `(pk, sk)` order. Its `(pk, sk)` cursor is also exclusive, so the cursor row is never repeated on the next page.

Ordering applies to arbitrary UTF-8 strings and does not depend on a delimiter being absent. The contract includes empty strings, embedded NUL bytes, slashes, ampersands, Korean and Japanese text, and emoji.

## Version semantics

Every stored document has opaque `data` and a document version. Versions are part of the document incarnation:

- a newly created document starts at version `0`;
- a normal `put` on an existing document increments the version by one;
- a successful optimistic update increments the version by one;
- deleting a document and creating the same key later starts a new incarnation at version `0`.

The ordinary application `get`, `query`, and `scan` APIs do not expose versions. Internal optimistic-transaction reads and the administrative scan do.

## Batch atomicity

`batch` is all-or-nothing. A successful batch applies every operation. A failed batch may not leave a state in which only a prefix of the operations is visible.

## Explicit transactions

An explicit transaction provides read-your-own-writes. A write is visible to subsequent reads in the same transaction, a deleted key reads as missing, and uncommitted data is not visible outside the transaction.

`commit` makes all writes visible together. `rollback` makes none of the transaction's writes visible. A transaction containing multiple writes therefore has the same all-or-nothing boundary as `batch`. The backend-neutral guarantee is read-your-own-writes, atomic commit, and rollback. Snapshot isolation, repeatable reads, conflict detection, and serializable explicit transactions are not guaranteed.

## Optimistic transactions

An optimistic transaction records the version (or missing state) observed by each read. Every observed key is validated exactly once at commit, including documents that were read but not mutated. Its writes are conditional on those observations:

- updating version `N` succeeds only while the document is still version `N`;
- deleting version `N` succeeds only while the document is still version `N`;
- creating a document read as missing succeeds only while the key is still missing.

The client expresses observations with conditional transaction items:

- reading an existing document and not mutating it uses `ConditionCheck { expected_version: N }`;
- reading a missing document and not mutating it uses `ConditionCheck { expected_version: None }`;
- reading an existing document and updating or deleting it is validated by the mutation's `expected_version` condition;
- reading a missing document and creating it is validated by `Create`'s missing-key precondition.

If a key was read as missing, then created and deleted before commit, the final item is `ConditionCheck { expected_version: None }`. A direct create that is deleted before commit is an unobserved net no-op and does not add a condition.

If a concurrent writer changes the observed state, the commit conflicts. A conflict reports the affected `ConflictKey` entries with `expected_version` and the best available `actual_version` (`None` means the key is currently missing). The high-level `trx` operation may retry the closure; after retry exhaustion it returns `TrxResult::Conflict`. Only a conditional conflict is retryable. Transport failures, timeouts, authentication failures, malformed responses, protocol errors, and backend errors return `TrxResult::Err`. The user closure runs without a backend write transaction. Memory validates and applies in one mutex critical section, Turso uses `BEGIN IMMEDIATE` only during its short commit phase, and Dibi sends one `TRANSACT_WRITE_ITEMS` request.

## `execute_ops` semantics

`execute_ops` preserves input order and returns one result per input operation in that same order:

- `Get` returns `DbResult::Single`;
- `Query` returns `DbResult::Multiple`;
- `Put` and `Delete` return `DbResult::Done`.

Combining requests through tuples or vectors via `DbRequest` preserves the same operation and result ordering. `execute_ops` is not implicitly transactional.

For a Dibi `Query`, `limit: None` means unlimited according to the public API. The client paginates instead of truncating it to the Dibi protocol page limit. If an operation sequence cannot be represented by one `EXECUTE_OPS` request without changing its semantics, the client executes it sequentially while preserving result order and visibility.

## Administrative API

The administrative API is backend-neutral and intentionally smaller than SQL. It provides an ordered scan and conditional writes so that tooling can inspect data and perform migrations without embedding a predicate language in the database server.

### Admin scan

`AdminScanRequest` contains:

```text
after: Option<(String, String)>
limit: usize
pk_prefix: Option<String>
```

`Database::admin_scan` returns a page of `AdminDocument` values and an optional exclusive `next` cursor. Every document contains:

```text
pk: String
sk: String
data: Bytes
version: i64
```

With no prefix it scans the whole database. With `pk_prefix`, it returns only keys whose partition key starts with that prefix while retaining global `(pk, sk)` ordering. The cursor is exclusive and can be passed back in the next request. No SQL predicate language is part of this API.

`AdminScan` is not a snapshot scan, and one page or a complete paginated pass does not represent a single point-in-time database state. Concurrent creates, updates, and deletes may occur during a scan. For example, a document created behind the current cursor may not be visible during that pass.

Each returned `AdminDocument` must still contain `data` and `version` from the same state of that document at the time it is read. This allows a migration client to transform `data` observed at version `N` and safely submit a `Put` with `expected_version: N`. A concurrent update after the read makes that conditional write stale, so it conflicts instead of overwriting the newer document.

Callers that require a completely static full-database migration must stop application writes or use a separate maintenance procedure. Phase 0 adds neither a snapshot API nor a migration lock.

### Conditional writes

`AdminWriteOp` has these forms:

- `Create { pk, sk, data }`: succeeds only when the key is missing and creates version `0`;
- `Put { pk, sk, expected_version, data }`: succeeds only when the current version exactly equals `expected_version`, then increments it;
- `Delete { pk, sk, expected_version }`: succeeds only when the current version exactly equals `expected_version`.

A conditional failure is reported as a conflict with the expected and actual versions. It is not a best-effort write.

### Atomic migration batches

Submitting multiple `AdminWriteOp` values to `Database::admin_write_batch` evaluates every precondition before making the batch visible. If all preconditions hold, all operations commit atomically and the outcome is `Applied`. If any operation conflicts, none of the operations is applied and the outcome is `Conflict`.

Every `(pk, sk)` in one administrative write batch must be unique, regardless of operation type. A duplicate key makes the batch invalid: `admin_write_batch` returns an error before starting a transaction, and no write is executed. This error is not an `AdminWriteOutcome::Conflict` because it does not represent a concurrent modification.

## Migration workflow

The recommended client-side migration workflow is:

1. Read one page with `AdminScan`.
2. Keep each document's `(pk, sk, data, version)`.
3. Deserialize the old schema and convert it to the new schema in the CLI or migration binary.
4. Submit a page-sized (or otherwise bounded) conditional atomic batch using each document's observed version.
5. If a document conflicts, read it again, convert the current value again, and retry that document.
6. Continue with the returned cursor.

The Dibi server executes no arbitrary migration code. It supplies only the scan and conditional atomic-write primitives. Schema conversion remains in Forte CLI or another client-side migration tool.

## Relationship to `forte db query` and `forte db exec`

Today, `forte db query <SQL>` and `forte db exec <SQL file>` are Turso SQL backend operations. A future Dibi-oriented administration interface will provide equivalent use cases without promising SQL syntax compatibility:

```text
forte db query 'SELECT pk, sk, data FROM docs ...'
    -> AdminScan

forte db exec migration.sql
    -> AdminScan
    -> client-side schema conversion
    -> conditional atomic AdminWriteOp batch
```

Inspection and migration remain supported as capabilities, but arbitrary SQL text and SQL parser compatibility are not part of the Dibi contract.

## Unsupported backend-specific features

### Raw SQL

`execute_raw` and `execute_raw_transactional` are Turso-only escape hatches. The Dibi backend returns an explicit `raw SQL is only supported by the Turso backend` error. Arbitrary SQL execution, SQL query syntax, and SQL migration files are explicitly outside the backend-neutral contract and are not required from Dibi.

### Turso commit locking

Turso does not begin `BEGIN IMMEDIATE` when an optimistic transaction starts or while the user closure runs. It reads versions without a write lock, then opens `BEGIN IMMEDIATE` only for commit-time validation and conditional application. A failed validation rolls back the whole commit phase.

### Turso-specific row counters

Hrana protocol details and Turso billing or diagnostics counters are also outside the contract. In particular, Dibi does not emulate `RawStatementResult`'s `rows_read`, `rows_written`, or SQL query-duration fields. SQL row counters are not application semantics and must not be used as backend portability requirements.
