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
let items: Vec<(String, Bytes)> = db.query("User/id=42", None::<&str>, 100).await?;
// items: Vec of (sk, data)
```

### `scan` — full table scan

```rust
let items: Vec<(String, String, Bytes)> = db.scan(None, 100).await?;
// items: Vec of (pk, sk, data)
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
use doc_db::{Trx, TrxControl};

let result = db.trx(|trx: Trx| async move {
    let handle = trx.read(UserGet { id: "42".to_string() }).await?;

    let new_user = /* compute update */;
    trx.write(handle, new_user)?;

    Ok(TrxControl::Commit(result_value))
}).await;
```

On conflict, the closure is retried automatically. Return `TrxControl::Cancel(value)` to abort without retry.

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

This generates:
- `UserPut(User)` — implements `DbRequest<Output = ()>` to put the document
- `UserGet` — struct with PK/SK fields; implements `DbRequest<Output = Option<User>>`
- `UserQuery` — struct with PK fields + optional SK fields + `limit`; implements `DbRequest<Output = Vec<User>>`
- `UserDelete` — struct with PK/SK fields; implements `DbRequest<Output = ()>`
- `impl Document for User` — provides the `key()` method for use in `trx`

### Key Formatting

Keys are formatted as `TypeName/pk_field=value&…`. Integer fields are zero-padded to preserve lexicographic sort order (e.g. `u32` → 10 digits). Signed integers are offset-encoded (added `|T::MIN|`) before padding.

Example PK for `User { id: "alice" }` → `"User/id=alice"`.

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

```rust
let db = doc_db::memory();

// Set up expectations
db.mock_get("User/id=42", "profile")
    .returns(Some(my_data));

db.mock_put("User/id=42", "profile")
    .returns_ok();

// ... run code under test ...

db.clear_mocks();
```

The mock API is in `doc_db::mock`. Unknown from repository: the exact builder API surface — check `doc-db/src/mock.rs` for `MockGetBuilder`, `MockPutBuilder`, `MockDeleteBuilder`.
