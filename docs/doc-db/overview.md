# doc-db

`doc-db` is a document-oriented database library backed by Turso/libSQL (or an in-memory store for tests). It works in both WASI components (Forte backends) and native Rust binaries.

All documents are stored in a single table with a composite key: `pk` (partition key) and `sk` (sort key), both strings. The value is an opaque byte blob (usually JSON).

## Creating a Database Connection

```rust
use doc_db::{Database, turso, memory};

// Production: reads TURSO_URL and TURSO_AUTH_TOKEN from environment
let db: Database = turso();

// Explicit config
let db: Database = doc_db::turso_with_config(
    "https://my-db.turso.io".to_string(),
    "my-token".to_string(),
);

// In-memory (tests)
let db: Database = memory();
```

`Database` is `Clone`. Share it across your handler by cloning.

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

### `batch` — atomic multi-operation write

```rust
use doc_db::BatchOp;

let ops = vec![
    BatchOp::Put { pk: "User/id=1", sk: "profile", data: &user1_bytes },
    BatchOp::Delete { pk: "User/id=2", sk: "profile" },
];
db.batch(&ops).await?;
```

### `transaction` — explicit ACID transaction

```rust
let mut tx = db.transaction().await?;
let data = tx.get("User/id=42", "profile").await?;
tx.put("User/id=42", "profile", &new_data).await?;
tx.commit().await?;
```

Call `tx.rollback()` to abort.

## `trx` — Optimistic Concurrency Transaction

Higher-level API with conflict detection. Reads are batched upfront; writes use optimistic locking with version checks.

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
- `trx.get(request)` — reads one or more documents; returns `Option<DocHandle<T>>`
- `trx.create(doc)` — inserts a new document; returns `Result<DocHandle<T>>`
- Modify loaded documents by dereferencing the handle (`*handle = new_value` or field assignment)
- `handle.delete()` — marks the document for deletion on commit
- `trx.commit(value)` — commit and return `value` as `TrxResult::Committed(value)`
- `trx.cancel(reason)` — abort without retry; returns `TrxResult::Cancelled(reason)`
- On conflict, the closure is retried automatically; `TrxResult::Conflict` is returned only when retries are exhausted

## Batching Requests with `DbRequest`

The `DbRequest` trait enables combining multiple reads into a single round-trip:

```rust
use doc_db::DbRequest;

let (user, settings) = (
    UserGet { id: "42" },
    SettingsGet { user_id: "42" },
).send_with(&db).await?;
```

Tuples up to 12 elements and `Vec<impl DbRequest>` are supported.

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

## Raw SQL

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
