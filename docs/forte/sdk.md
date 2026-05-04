# Forte SDK Reference

`forte-sdk` is the runtime library for Forte applications. It re-exports commonly needed crates and provides HTTP types, an outbound HTTP client, cookie handling, and utilities.

Crate path: `forte/sdk`

## Re-exports

`forte-sdk` re-exports the following for convenience:

| Item | Source |
|------|--------|
| `anyhow`, `anyhow::Result` | `anyhow` |
| `Cookie`, `CookieBuilder`, `CookieJar` | `cookie` |
| `forte_json` | `forte-json` |
| `forte_macros::{forte_doc, test}` | `forte-macros` |
| `futures` | `futures` |
| `hex` | `hex` |
| `serde`, `serde_json` | `serde` / `serde_json` |
| `sha2` | `sha2` |
| `time` | `time` |
| `tracing` | `tracing` |
| `uuid::Uuid` | `uuid` |
| `wit_bindgen` | `wit-bindgen` |
| `http_header::*` | `http::header` |

`DateTime` is `chrono::DateTime<chrono::Utc>`. Use `forte_sdk::now()` for the current UTC time.

## ForteRequest

```rust
pub struct ForteRequest<'a, Body = ()> {
    pub uri_authority: &'a str,
    pub method: &'a http::Method,
    pub headers: &'a http::HeaderMap,
    pub jar: &'a mut CookieJar,
    pub raw_body: &'a [u8],
    pub body: Body,
}
```

- `uri_authority` — the `host:port` authority from the request URI.
- `method` — HTTP method.
- `headers` — full header map (including `Cookie`).
- `jar` — mutable cookie jar; cookies added here are emitted as `Set-Cookie` response headers.
- `raw_body` — raw request body bytes.
- `body` — typed body, parameterized per handler type (`()` for pages, `Input` for actions/hooks).

## HTTP Module (`forte_sdk::http`)

### Types (re-exported from `http` crate)

```rust
pub use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode};
pub use http::uri::{Authority, PathAndQuery, Scheme, Uri};
pub use http::request::Builder as RequestBuilder;
pub use bytes::Bytes;
```

### Body

```rust
pub enum Body {
    Empty,
    Bytes(Vec<u8>),
    Stream(StreamReader<u8>),
}
```

Conversions:
- `Body::from(Vec<u8>)`, `Body::from(&[u8])`, `Body::from(String)`, `Body::from(&str)`, `Body::from(Bytes)`
- `Body::from(())` → `Body::Empty`

Methods:
- `async fn bytes(self) -> Bytes` — consumes and collects the body.
- `async fn json<T: DeserializeOwned>(self) -> Result<T>` — deserializes as JSON.

### Outbound HTTP Client

```rust
use forte_sdk::http::{Client, Request, Body};

let client = Client::new();
let req = Request::builder()
    .method("GET")
    .uri("https://example.com/api/data")
    .body(Body::empty())?;
let resp = client.send(req).await?;
let body = resp.into_body().bytes().await;
```

`Client::send` maps `http::Request<B: Into<Body>>` to a WASI:HTTP p3 outbound request and returns `http::Response<Body>`. Streaming request bodies are not yet supported — the client returns `Error::StreamBodyNotSupported` if the body is a `Stream`.

### Errors

```rust
pub enum Error {
    Headers(p3::HeaderError),
    InvalidScheme,
    InvalidAuthority,
    InvalidPathWithQuery,
    InvalidMethod,
    StreamBodyNotSupported,
    Wasi(p3::ErrorCode),
    BuildResponse(http::Error),
    Json(serde_json::Error),
}
```

## Serve (`forte_sdk::serve`)

`serve::serve(req, dispatch)` is called by the generated `route_generated.rs`. It:
1. Converts the WASI p3 request to `http::Request<Vec<u8>>`.
2. Creates an OpenTelemetry span `"http.request"`.
3. Calls the dispatch closure.
4. Converts the response back to WASI p3.

You do not call this directly in application code.

## Runtime (`forte_sdk::runtime`)

- `forte_sdk::runtime::block_on(future)` — drives an async future to completion. Used by the `#[forte_sdk::test]` macro.
- `forte_sdk::runtime::spawn(future)` — spawns a task (used internally for body streaming).

## Utilities

| Function / Type | Description |
|----------------|-------------|
| `forte_sdk::now()` | Current UTC time as `chrono::DateTime<Utc>` |
| `forte_sdk::Uuid` | UUID type (re-exported) |
| `forte_sdk::rand` | Random number utilities |
| `forte_sdk::time_wasi` | WASI-compatible time helpers |
| `forte_sdk::cookie_sign` | Signed cookie helpers |
| `forte_sdk::otel` | OpenTelemetry initialization (called once per instance) |

## Macros (`forte_sdk` re-exports from `forte-macros`)

### `#[forte_sdk::test]`

Wraps an `async fn` so it can run as a standard Rust test inside wasmtime:

```rust
#[forte_sdk::test]
async fn my_test() {
    assert_eq!(1 + 1, 2);
}
```

Requirements: must be `async fn`, must take no arguments.

### `#[forte_sdk::forte_doc]`

See [../doc-db.md](../doc-db.md) for the `forte_doc` macro that generates database access types.
