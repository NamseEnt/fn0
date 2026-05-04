# doc-db: Document Database

`doc-db` is a document-oriented database library backed by Turso (libSQL) or an in-memory store (for testing). It uses a two-key model: **partition key (pk)** and **sort key (sk)**.

Crate path: `doc-db`

> **Compile target**: `doc-db` compiles to `wasm32-wasip2`. Its `.cargo/config.toml` sets this target and uses `forte-test-runner` as the test runner.

## Database Creation

```rust
// Turso (reads TURSO_URL and TURSO_AUTH_TOKEN env vars)
let db = doc_db::turso();

// Turso with explicit config
let db = doc_db::turso_with_config(url, auth_token);

// In-memory (for tests)
let db = doc_db::memory();
```

## Core Operations

All operations on `Database` are async:

```rust
// Get a document by pk + sk → Option<Bytes>
let data: Option<Bytes> = db.get(pk, sk).await?;

// Put a document
db.put(pk, sk, &data_bytes).await?;

// Delete a document
db.delete(pk, sk).await?;

// Query all documents under a pk, optionally after a sk, up to limit
let rows: Vec<(String, Bytes)> = db.query(pk, after_sk, limit).await?;

// Scan all documents in the table
let rows: Vec<(String, String, Bytes)> = db.scan(after, limit).await?;

// Batch write (put + delete operations atomically)
db.batch(&[
    BatchOp::Put { pk, sk, data },
    BatchOp::Delete { pk, sk },
]).await?;
```

## Transactions

```rust
let mut tx = db.transaction().await?;
let val = tx.get(pk, sk).await?;
tx.put(pk, sk, &new_data).await?;
tx.commit().await?;
// or tx.rollback().await?;
```

## Optimistic Concurrency (`trx`)

The `db.trx()` API provides optimistic locking with automatic retry on conflict:

```rust
let result = db.trx(|trx| async move {
    let handle = trx.get(MyDocGet { user_id: 1 }).await?;
    let doc = handle.require()?;  // returns ConflictKey or the doc
    // ... modify doc ...
    trx.put(MyDocPut(modified_doc));
    Ok(TrxControl::Commit(output))
}).await;
```

`TrxResult<Out, Cancel, E>` is the return type — check `doc-db/src/trx.rs` for full details.

## Low-Level Operations API (`DbOp` / `DbRequest`)

For batching multiple operations in a single round-trip:

```rust
pub enum DbOp {
    Get { pk: String, sk: String },
    Query { pk: String, after_sk: Option<String>, limit: Option<usize> },
    Put { pk: String, sk: String, data: Vec<u8> },
    Delete { pk: String, sk: String },
}
```

Implement `DbRequest` to bundle ops + a parser:

```rust
pub trait DbRequest: Sized {
    type Output;
    fn prepare(self) -> Prepared<Self::Output>;
    async fn send_with(self, db: &Database) -> Result<Self::Output>;
}
```

Tuples of `DbRequest` items are also `DbRequest` — operations are sent in one batch:

```rust
let (user, post) = (UserGet { id: 1 }, PostGet { id: 2 })
    .send_with(&db)
    .await?;
```

Supported tuple sizes: 1–12 elements. `Vec<T: DbRequest>` is also supported.

## `#[forte_doc]` Macro

The `forte_doc` attribute macro on a struct generates typed database access types. Requires `forte-macros`.

```rust
use forte_sdk::forte_doc;

#[forte_doc]
pub struct Post {
    #[pk] pub user_id: u64,
    #[sk] pub post_id: u64,
    pub title: String,
    pub body: String,
}
```

This generates:

| Generated type | Purpose |
|---------------|---------|
| `Post` | The document struct (with `serde::{Serialize, Deserialize}`, implements `doc_db::Document`) |
| `PostPut(Post)` | `DbRequest<Output = ()>` — insert/update |
| `PostGet { user_id, post_id }` | `DbRequest<Output = Option<Post>>` — fetch by key |
| `PostQuery { user_id, post_id: Option<u64>, limit: Option<usize> }` | `DbRequest<Output = Vec<Post>>` — range query |
| `PostDelete { user_id, post_id }` | `DbRequest<Output = ()>` — delete by key |

### Key Encoding

Keys are formatted as `"TypeName/pk_field=value&..."` for the pk and `"sk_field=value&..."` for the sk.

Integer types in pk/sk are zero-padded so that lexicographic order matches numeric order:

| Type | Width | Example |
|------|-------|---------|
| `u8` / `i8` | 3 digits | `042` |
| `u16` / `i16` | 5 digits | `00042` |
| `u32` / `i32` | 10 digits | `0000000042` |
| `u64` / `i64` / `usize` / `isize` | 20 digits | `00000000000000000042` |

Signed integers are offset by `|T::MIN|` before formatting to preserve sort order.

`String` fields in pk are accepted as any `AsRef<str>` type.

### Using Generated Types

```rust
// Put
PostPut(post).send_with(&db).await?;

// Get
let post: Option<Post> = PostGet { user_id: 1, post_id: 42 }.send_with(&db).await?;

// Query posts for user_id = 1, after post_id 10, up to 20
let posts: Vec<Post> = PostQuery {
    user_id: 1,
    post_id: Some(10),
    limit: Some(20),
}.send_with(&db).await?;

// Delete
PostDelete { user_id: 1, post_id: 42 }.send_with(&db).await?;
```

## Mock API (for Testing)

```rust
let db = doc_db::memory();  // or turso() in tests with env vars

// Mock a get to return specific data
db.mock_get("pk", "sk").returns(Some(data_bytes));

// Mock a get to return an error
db.mock_get("pk", "sk").returns_error("simulated failure");

// Clear all mocks
db.clear_mocks();
```

The mock layer checks `(op, pk, sk)` before hitting the real backend. See `doc-db/src/mock.rs` for the full builder API.

## Raw SQL

For advanced use cases:

```rust
let rows = db.execute_raw(
    "SELECT * FROM docs WHERE pk = ?",
    vec![doc_db::text_value("my_pk")],
    true,
).await?;
```
