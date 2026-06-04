# Testing Forte Backends

Backend tests in a Forte project are async Rust functions annotated with `#[forte_sdk::test]`. This macro runs the test inside `forte_sdk::runtime::block_on`, which is compatible with the WASI async runtime used by Forte components.

## `#[forte_sdk::test]`

Use `#[forte_sdk::test]` for all async tests in backend crates. Do not use `#[tokio::test]` — it is not available in `wasm32-wasip2`.

```rust
#[forte_sdk::test]
async fn my_test() {
    // async test code
}
```

The test function must be `async fn` with no parameters.

## Testing with an In-Memory Database

`doc_db::memory()` returns an in-memory backend with the same `Database` API as the production Turso/libSQL backend. Each call returns a fresh, isolated instance — no setup, no cleanup, no external server.

```rust
use doc_db::DbRequest;

#[forte_sdk::test]
async fn test_user_roundtrip() {
    let db = doc_db::memory();

    UserPut(User {
        id: "alice".into(),
        version: 1,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    })
    .send_with(&db)
    .await
    .unwrap();

    let user: Option<User> = UserGet { id: "alice".into(), version: 1 }
        .send_with(&db)
        .await
        .unwrap();

    assert_eq!(user.unwrap().name, "Alice");
}
```

Two `doc_db::memory()` calls return independent databases that never share state.

### Simulating database errors

The mock API on a `memory()` database lets you force specific return values or errors. Mocks are consumed one-at-a-time (FIFO) per `(op, pk, sk)` key; after the mock fires, subsequent calls hit the real in-memory backend.

```rust
#[forte_sdk::test]
async fn test_get_error_propagation() {
    let db = doc_db::memory();

    // Force the next get on this key to fail
    db.mock_get("User/id=alice", "version=0000000001")
        .returns_err("network timeout");

    let result = db.get("User/id=alice", "version=0000000001").await;
    assert!(result.is_err());

    // Next call hits the real backend (returns None — no data was written)
    let result = db.get("User/id=alice", "version=0000000001").await;
    assert!(result.unwrap().is_none());
}
```

See [doc-db/overview.md](../doc-db/overview.md#mocking-tests) for the full mock API (`mock_put`, `mock_delete`, `clear_mocks`, etc.).

> **Note on raw pk/sk keys:** The mock API takes raw string keys. When using the `#[forte_doc]` macro, keys follow the format `TypeName/pk_field=value` for the pk and `sk_field=value` for the sk. Integer fields are zero-padded (e.g. `version=0000000001` for a `u32` of 1). Prefer testing through the generated typed helpers (`UserGet`, `UserPut`, etc.) and only reach for the raw mock API when you need to simulate errors.

## Testing with In-Memory Object Storage

`object_storage::memory()` returns an in-process `Bucket` backed by a `BTreeMap`. The API is identical to production; each call returns a fresh, isolated instance.

```rust
#[forte_sdk::test]
async fn test_file_roundtrip() {
    let bucket = object_storage::memory();

    bucket.put("avatars/alice.png", b"fake png data" as &[u8]).await.unwrap();

    let data = bucket.get("avatars/alice.png").await.unwrap();
    assert_eq!(data.unwrap().as_ref(), b"fake png data");

    bucket.delete("avatars/alice.png").await.unwrap();
    assert!(bucket.get("avatars/alice.png").await.unwrap().is_none());
}
```

## Testing Action Handlers Directly

Action and hook handlers are plain `async fn`s. You can call them directly in tests by constructing a `ForteRequest` manually — there is no constructor, so initialize all fields directly.

```rust
use forte_sdk::{ForteRequest, CookieJar};
use forte_sdk::http::{Method, HeaderMap};

#[forte_sdk::test]
async fn test_login_ok() {
    let mut jar = CookieJar::new();
    let method = Method::POST;
    let headers = HeaderMap::new();

    let req = ForteRequest {
        uri_authority: "localhost",
        method: &method,
        headers: &headers,
        jar: &mut jar,
        raw_body: &[],
        body: crate::actions::user_login::Input {
            email: "alice@example.com".to_string(),
            password: "correct".to_string(),
        },
    };

    let output = crate::actions::user_login::handler(req).await;
    assert!(matches!(output, crate::actions::user_login::Output::Ok { .. }));
}
```

Combine with `doc_db::memory()` and `object_storage::memory()` to test handlers without any external services.

## Running Tests

From inside a Forte project's `rs/` directory:

```sh
cargo test
```

From the monorepo workspace root:

```sh
cargo test                  # all workspace crates
cargo test -p fn0-doc-db    # doc-db only
cargo test -p forte-sdk     # forte-sdk only
```

### doc-db integration tests

`doc-db/tests/integration_test.rs` connects to a live libSQL server. See [development.md](../development.md#doc-db-integration-tests) for how to start one locally.
