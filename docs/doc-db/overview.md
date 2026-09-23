# doc-db

`doc-db` is a document-oriented database library backed by Turso/libSQL (or an in-memory store for tests). It works in both WASI components (Forte backends) and native Rust binaries.

All documents are stored in a single table with a composite key: `pk` (partition key) and `sk` (sort key), both strings. The value is an opaque byte blob (usually JSON).

## Creating a Database Connection

```rust
use doc_db::{Database, database, memory};

// Normal fn0 application code: uses the backend-neutral semantic RPC.
let db: Database = database();

// Legacy/direct Turso path for raw SQL, migrations, or explicit transactions.
let db: Database = doc_db::turso_with_config(
    "https://my-db.turso.io".to_string(),
    "my-token".to_string(),
);

// In-memory (tests)
let db: Database = memory();
```

`Database` is `Clone`. Share it across your handler by cloning.

## Semantic RPC boundary

Normal fn0 application code should use `doc_db::database()`. It is the
backend-neutral client for the semantic document API. It reads
`FN0_DOC_DB_URL`, and fn0 routes that endpoint through `DocDbHijack` to the
host document service in both deployed workers and `forte dev`. The current
host backend is an implementation detail and is not exposed to the guest.

`doc_db::semantic()` and `doc_db::semantic_with_config()` remain available as
compatibility APIs. `doc_db::turso()` and `doc_db::turso_with_config()` remain
the direct/legacy path for raw SQL, migrations, or explicit session
transactions. Do not switch those users to `database()`.

Standalone `fn0 local` uses the same semantic boundary with a process-local
in-memory host backend. Its document state is lost when the local server
restarts; this does not describe the persistent backends used by Forte or
production workers.

The semantic request payload contains document keys and operations, but no
project or tenant identity. fn0 obtains the authoritative project identity
from the invocation context before forwarding the request to the host service.

The semantic observed state uses an opaque `DocDbRevision`. A present document carries its revision. A missing document carries `Some(revision)` only when the backend has an exact revision for that missing state; `None` means that the backend knows only that the key is currently absent. Turso physically removes rows and therefore returns `None` for missing documents. A backend with persistent missing-state revisions must return the exact missing revision, including revision zero for a key that has never existed. The `trx` layer turns an exact missing revision into `RevisionEquals`, preserving insert-delete ABA detection, and uses `NotExists` when no exact revision is available.

`RevisionEquals(key, revision)` compares the current logical state revision of the key. It is not limited to an existing row: a backend that returns `Missing { revision: Some(revision) }` must allow the same equality condition to succeed while the key is missing. The current Turso and Memory backends expose `Missing { revision: None }` only and do not provide persistent missing-state revisions. The numeric representation of `DocDbRevision` exists for serialization and backend adaptation; ordering, arithmetic, and global monotonicity are not semantic contracts.

The semantic transaction request is a pair of condition and mutation lists. Updates and deletes use `RevisionEquals` with the observed revision, inserts use `NotExists`, and read-only observations contribute a condition without a mutation. Conditions are checked in order; a conflict reports the first failing condition index and publishes zero mutations.

Each transaction request allows at most one condition and at most one mutation for a key. A condition and a mutation for the same key are valid together; duplicate conditions or duplicate mutations are invalid requests and are rejected before backend execution. Condition-only transactions are valid and must still validate their conditions. The dodb adapter sends conditions and mutations through dodb's atomic `Transact` operation, including condition-only requests.

The semantic protocol contains single-operation requests for `Get`, `Put`,
`Delete`, `Query`, and `Scan`, plus the internal single-document
`GetObserved` request used by optimistic transactions. `Transact` is the only
multi-item request: it carries conditions and mutations atomically.

The semantic RPC applies a 16 MiB fn0 platform frame limit to requests and responses. This limit is independent of any future dodb storage value limit. JSON and base64 encoding add overhead, so the largest usable binary document is smaller than 16 MiB.

## Basic Operations

### `get`

```rust
let bytes: Option<Bytes> = db.get("User/id=42", "profile").await?;
```

### `put`

```rust
db.put("User/id=42", "profile", &serde_json::to_vec(&user)?).await?;
```

### `delete`

```rust
db.delete("User/id=42", "profile").await?;
```

### `query` — scan by partition key

Returns all documents with the given `pk`, optionally starting after `after_sk`, up to `limit` items. Results are sorted by `sk` ascending.

```rust
let items: Vec<(String, Bytes)> = db.query("User/id=42", None::<&str>, 100_usize).await?;
// items: Vec of (sk, data)
```

### `scan` — full table scan

Scans all documents, optionally resuming after a `(pk, sk)` cursor.

```rust
// From the beginning
let items: Vec<(String, String, Bytes)> = db.scan(None, 100).await?;
// items: Vec of (pk, sk, data)

// Resume after a cursor
let items = db.scan(Some(("User/id=42", "profile")), 100).await?;
```

### `transaction` — explicit ACID transaction (legacy/direct Turso only)

```rust
let mut tx = doc_db::turso().transaction().await?;
let data = tx.get("User/id=42", "profile").await?;
tx.put("User/id=42", "profile", &new_data).await?;
tx.commit().await?;
```

Call `tx.rollback()` to abort.

## `trx` — Optimistic Concurrency Transaction

Higher-level API with conflict detection. Multiple requested documents are
read with concurrent single-document observed requests; writes use optimistic
locking with version checks.

```rust
let result = db.trx(|trx| async move {
    // Read — returns Option<DocHandle<User>>
    let mut user_handle = trx
        .get(UserGet { id: "42".to_string(), version: 1 })
        .await?
        .ok_or_else(|| anyhow::anyhow!("user not found"))?;

    // Modify in-place via DerefMut (marks the handle dirty automatically)
    user_handle.name = "New Name".to_string();

    // Create a new document in the same transaction
    trx.create(AuditLog { user_id: "42".to_string(), action: "rename".to_string() })?;

    // Commit — returns Result<TrxControl<Out, Cancel>>
    trx.commit(())
}).await;

match result {
    doc_db::TrxResult::Committed(()) => { /* success */ }
    doc_db::TrxResult::Cancelled(reason) => { /* trx.cancel(reason) was called */ }
    doc_db::TrxResult::Conflict(_) => { /* retries exhausted */ }
    doc_db::TrxResult::Err(e) => { /* handler returned Err */ }
}
```

Key points:
- `trx.get(request)` — reads one or more documents concurrently; returns `Option<DocHandle<T>>`
- `trx.create(doc)` — inserts a new document; returns `Result<DocHandle<T>>`
- Modify loaded documents by dereferencing the handle (`*handle = new_value` or field assignment)
- `handle.delete()` — marks the document for deletion on commit
- `trx.commit(value)` — commit and return `value` as `TrxResult::Committed(value)`
- `trx.cancel(reason)` — abort without retry; returns `TrxResult::Cancelled(reason)`
- On conflict, the closure is retried automatically; `TrxResult::Conflict` is returned only when retries are exhausted

`trx` is the atomic API. It supports unconditional multi-write transactions,
conditional optimistic transactions, and condition-only transactions. An
empty transaction is a local no-op and is not sent to the dodb backend.

The dodb transport is created lazily by production workers. Startup validates
the configured endpoint and TLS roots but does not dial the server. The first
request establishes the shared connection; after transport loss, a later
independent request reconnects through that same manager. Uncertain mutations
are returned as errors and are never replayed by fn0.

## Aggregating Requests with `DbRequest`

The `DbRequest` trait enables combining independent requests while preserving
the convenient tuple/vector result shape. A single request produces one
single-operation semantic RPC. Tuple and vector requests produce one
single-operation RPC per prepared operation and await them concurrently.
They are non-atomic, have no rollback, and operations may succeed or fail
independently.

One request:

```rust
let user = UserGet { id: "42" }.send_with(&db).await?;
```

Different request types:

```rust
use doc_db::DbRequest;

let (user, settings) = (
    UserGet { id: "42" },
    SettingsGet { user_id: "42" },
).send_with(&db).await?;
```

Same request type in bulk:

```rust
let users = vec![
    UserGet { id: "41" },
    UserGet { id: "42" },
].send_with(&db).await?;
```

Tuples up to 12 elements and `Vec<impl DbRequest>` are supported. Results stay
in input order, but execution order is not guaranteed. All started operations
settle before an error is returned; if multiple operations fail, the first
error in input order is returned. Tuple and vector `send_with` calls are not
transactions.

Do not use a tuple or vector to express ordering or atomicity. For example,
two writes to the same key have no guaranteed last-writer order. Await them
sequentially when order matters, or use `trx` when the writes must commit
atomically.

## `#[forte_doc]` Macro

The `forte_doc` procedural macro (from `forte-macros`) derives typed CRUD operations for a struct with `#[pk]` and `#[sk]` field attributes:

```rust
use forte_sdk::forte_doc;
use doc_db::DbRequest;

#[forte_doc]
pub struct User {
    #[pk]
    pub id: String,
    #[sk]
    pub version: u32,
    pub name: String,
    pub email: String,
}
```

`#[forte_doc]` automatically adds `#[derive(serde::Serialize, serde::Deserialize, Clone)]` to the struct. Do not add those derives manually — it will fail to compile.

This generates:
- `UserPut(User)` — implements `DbRequest<Output = ()>` to put the document
- `UserGet` — struct with PK/SK fields; implements `DbRequest<Output = Option<User>>`
- `UserQuery` — struct with PK fields + optional SK fields + `limit`; implements `DbRequest<Output = Vec<User>>`
- `UserDelete` — struct with PK/SK fields; implements `DbRequest<Output = ()>`
- `impl Document for User` — provides the `key()` method for use in `trx`

### Key Formatting

Keys are formatted as `TypeName/pk_field=value&…`. Integer fields are zero-padded to preserve lexicographic sort order so that string comparison matches numeric order. Signed integers are offset-encoded (shifted by `|T::MIN|`) before padding.

| Type | Width | Example (`42`) |
|---|---|---|
| `u8` | 3 | `042` |
| `u16` | 5 | `00042` |
| `u32` | 10 | `0000000042` |
| `u64` / `usize` | 20 | `00000000000000000042` |
| `i8` | 3 (offset +128) | `170` |
| `i16` | 5 (offset +32768) | `32810` |
| `i32` | 10 (offset +2147483648) | `2147483690` |
| `i64` / `isize` | 20 (offset +2^63) | `09223372036854775850` |

`String` and `&str` fields are used as-is (no padding).

Example PK for `User { id: "alice" }` → `"User/id=alice"`.

Multiple `#[pk]` or `#[sk]` fields are joined with `&`:

```rust
#[forte_doc]
pub struct Post {
    #[pk]
    pub user_id: String,
    #[pk]
    pub category: String,
    #[sk]
    pub created_at: u64,
    #[sk]
    pub post_id: String,
    pub title: String,
}
// pk → "Post/user_id=alice&category=news"
// sk → "created_at=00000000001234567890&post_id=abc"
```

#### No `#[pk]` or `#[sk]` fields

- **No `#[pk]` fields** → pk = `TypeName` (a singleton-per-type stored at one row)
- **No `#[sk]` fields** → sk = `""` (empty string)
- **Only `#[sk]` fields** → pk = `TypeName`, sk = `field=value&…`

These patterns are used for singleton config documents (e.g., a single document per deployment storing global state). A struct with no pk or sk fields is stored at `pk=TypeName, sk=""`.

### Usage

```rust
// Put
UserPut(user).send_with(&db).await?;

// Get
let user: Option<User> = UserGet { id: "alice".to_string(), version: 1 }
    .send_with(&db)
    .await?;

// Query (all versions of user "alice")
let users: Vec<User> = UserQuery {
    id: "alice".to_string(),
    version: None,
    limit: Some(10),
}.send_with(&db).await?;

// Delete
UserDelete { id: "alice".to_string(), version: 1 }
    .send_with(&db)
    .await?;
```

## Raw SQL (legacy/direct Turso only)

```rust
use doc_db::Value;

let rows = db.execute_raw(
    "SELECT pk, sk FROM docs WHERE pk = ?",
    vec![doc_db::text_value("User/id=42")],
    true,
).await?;
```

## Mocking (Tests)

Use `doc_db::memory()` to get an in-memory database. Register mock rules before running code under test; rules are consumed in FIFO order per `(op, pk, sk)` key.

```rust
let db = doc_db::memory();

// Get: return data
db.mock_get("User/id=42", "profile")
    .returns(my_data_bytes);       // returns Ok(Some(bytes))

// Get: return not found
db.mock_get("User/id=99", "profile")
    .returns_none();               // returns Ok(None)

// Get: return error
db.mock_get("User/id=00", "profile")
    .returns_err("db error");      // returns Err(...)

// Put: succeed
db.mock_put("User/id=42", "profile")
    .returns_ok();

// Put: fail
db.mock_put("User/id=42", "profile")
    .returns_err("write failed");

// Delete: succeed
db.mock_delete("User/id=42", "profile")
    .returns_ok();

// Delete: fail
db.mock_delete("User/id=42", "profile")
    .returns_err("delete failed");

// ... run code under test ...

db.clear_mocks();  // remove all remaining rules
```

Builder types: `MockGetBuilder`, `MockPutBuilder`, `MockDeleteBuilder` (in `doc_db::mock`, used via the `Database` methods above).

## CLI Access

The `forte` CLI provides direct SQL access to the deployed project's database through the control plane, without needing database credentials.

```sh
# One-off query (table output)
forte db query 'SELECT pk, sk FROM docs LIMIT 10'

# Query with bind parameters
forte db query 'SELECT data FROM docs WHERE pk = ?' \
  --arg 'User/id=alice'

# Run a SQL migration file as one atomic transaction
forte db exec migrations/2026-08-18-backfill.sql
```

### `forte db query`

Runs a single SQL statement and prints results as a table. Use `--json` for machine-readable output, `--arg` (repeatable) to bind `?` placeholders.

**When writing directly to `docs` via raw SQL:**
- Always increment `version` on every `UPDATE` of `data` — `SET data = ..., version = version + 1`. The optimistic-locking layer in `trx` uses `version` for conflict detection; an unchanged version can be silently overwritten.
- A `SELECT` without `LIMIT` can scan every row; the row-read count in the output is what a quota would charge.

### `forte db exec`

Runs every statement in a `.sql` file as one transaction — all statements commit or none do. A failing statement rolls the whole file back and reports which statement failed. Use this for schema migrations.

```sql
-- migrations/2026-08-18-add-tags.sql
INSERT INTO docs (pk, sk, data, version) VALUES ('Tag/id=foo', '', '{}', 1);
INSERT INTO docs (pk, sk, data, version) VALUES ('Tag/id=bar', '', '{}', 1);
```

```sh
forte db exec migrations/2026-08-18-add-tags.sql
```

See [`forte db` in the CLI reference](../forte/cli.md#forte-db-query-sql-options) for all flags.
