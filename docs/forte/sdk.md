# forte-sdk Reference

`forte-sdk` is the runtime library that backend WASM components use. It provides HTTP types, request/response handling, cookie utilities, an outbound HTTP client, and re-exports commonly used crates.

## `ForteRequest<'a, Body>`

The request context passed to page and action handlers.

```rust
pub struct ForteRequest<'a, Body = ()> {
    pub uri_authority: &'a str,       // host:port, e.g. "example.com"
    pub method: &'a http::Method,
    pub headers: &'a http::HeaderMap,
    pub jar: &'a mut CookieJar,
    pub raw_body: &'a [u8],
    pub body: Body,                    // typed body; () for pages, Input for actions
}
```

## HTTP Types

Re-exported from the `http` crate:

```rust
pub use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode};
pub use http::uri::{Authority, PathAndQuery, Scheme, Uri};
pub use http::request::Builder as RequestBuilder;
```

Also available via `forte_sdk::http_header::*` (all `http::header::*` constants).

## `Body`

An enum representing an HTTP body:

```rust
pub enum Body {
    Empty,
    Bytes(Vec<u8>),
    Stream(StreamReader<u8>),
}

impl Body {
    pub fn empty() -> Self;
    pub async fn bytes(self) -> Bytes;
    pub async fn json<T: DeserializeOwned>(self) -> Result<T>;
}
```

Converts from `Vec<u8>`, `&[u8]`, `String`, `&str`, `Bytes`, and `()`.

## Outbound HTTP Client

Make outbound requests using `http::Client`:

```rust
use forte_sdk::http::{Client, Request};

let client = Client::new();

let req = Request::builder()
    .method("POST")
    .uri("https://api.example.com/data")
    .header("content-type", "application/json")
    .body(Body::from(r#"{"key":"value"}"#))?;

let resp = client.send(req).await?;
let status = resp.status();
let body = resp.into_body().bytes().await;
```

Limitations:
- Streaming request bodies are not supported (use `Body::Bytes`)
- Subject to fn0 Cloud subrequest limit (50 per request)

## Cookie Signing

Signed, HMAC-SHA256 cookies. Requires `COOKIE_SECRET` env var.

```rust
use forte_sdk::cookie_sign::{sign_cookie, unsign_cookie};
use forte_sdk::time;

// Write a signed cookie
sign_cookie(
    req.jar,
    "session",
    &my_value,               // any T: Serialize
    Some(time::Duration::days(30)),
);

// Read and verify a signed cookie
let value: Option<MyType> = unsign_cookie(req.jar, "session");
```

Cookies are set `HttpOnly`, `Secure`, `Path=/`. The value is serialized with `serde_json` and then HMAC-signed; the signature is appended as a hex suffix separated by `.`.

## `serve` function

Used internally by the generated `route_generated.rs`. Bridges between the WASI HTTP types and the `http::Request` / `http::Response` types:

```rust
pub async fn serve<F, Fut, E>(
    req: wasi::http::types::Request,
    dispatch: F,
) -> Result<wasi::http::types::Response, ErrorCode>
where
    F: FnOnce(http::Request<Vec<u8>>) -> Fut,
    Fut: Future<Output = Result<http::Response<Body>, E>>,
    E: fmt::Debug,
```

Also initializes OpenTelemetry and creates a tracing span for each request.

## Time

```rust
use forte_sdk::{DateTime, now};

pub type DateTime = chrono::DateTime<chrono::Utc>;

let t: DateTime = now(); // current UTC time
```

Also re-exports the `time` crate for use with cookie max-age.

## UUID

```rust
use forte_sdk::Uuid;
let id = Uuid::new_v4();
```

## Randomness

```rust
use forte_sdk::rand;
// See forte/sdk/src/rand.rs — Unknown from repository which exact API is exposed.
```

## Tracing / Logging

```rust
use forte_sdk::tracing;

tracing::info!("processing request");
tracing::error!("something failed: {}", err);
```

OpenTelemetry is initialized once per instance on the first request (via `otel::init_once()`). OTLP exporter configuration is Unknown from repository — check `forte/sdk/src/otel.rs`.

## Re-exported Crates

All re-exported at the crate root and usable via `forte_sdk::`:

| Symbol | Source crate |
|---|---|
| `anyhow` | `anyhow` |
| `chrono` | `chrono` |
| `cookie`, `Cookie`, `CookieBuilder`, `CookieJar` | `cookie` |
| `form_urlencoded` | `form_urlencoded` |
| `forte_json` | `forte-json` |
| `forte_macros::{forte_doc, test}` | `forte-macros` |
| `futures` | `futures` |
| `hex` | `hex` |
| `serde` | `serde` |
| `serde_json` | `serde_json` |
| `sha2` | `sha2` |
| `time` | `time` |
| `tracing` | `tracing` |
| `Uuid` | `uuid` |
| `wit_bindgen` | `wit-bindgen` |

## Runtime Utilities

Spawning async tasks (via `wit_bindgen` runtime):

```rust
use forte_sdk::runtime::{spawn, block_on};
```

`block_on` is used by the `#[forte_sdk::test]` macro.

## Macros

### `#[forte_sdk::test]`

Wraps an `async fn` as a synchronous test using `runtime::block_on`:

```rust
#[forte_sdk::test]
async fn my_test() {
    // async test code
}
```

Must be `async fn` with no arguments.

### `#[forte_doc]`

Derives database CRUD operations for a struct. See [doc-db/overview.md](../doc-db/overview.md) for full documentation.
