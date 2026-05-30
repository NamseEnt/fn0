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

## Async Sleep and Monotonic Time (`time_wasi`)

`forte_sdk::time_wasi` provides WASI-backed async sleep and monotonic time measurement. Use these instead of `std::thread::sleep` (which blocks) or `tokio::time::sleep` (which is not available in WASM components).

```rust
use forte_sdk::time_wasi;

// Async sleep
time_wasi::sleep(time_wasi::Duration::from_secs(1)).await;

// Measure elapsed time
let start = time_wasi::Instant::now().await;
// ... do work ...
let elapsed: time_wasi::Duration = start.elapsed().await;
```

API:
- `time_wasi::sleep(duration: Duration)` — suspend the current task for `duration`
- `time_wasi::Instant::now() -> Instant` — current monotonic time (async)
- `time_wasi::Instant::duration_since(&self, earlier: Instant) -> Duration` — time between two instants
- `time_wasi::Instant::elapsed(&self) -> Duration` — time since this instant was recorded (async)
- `time_wasi::Duration` — re-exported `std::time::Duration`

## UUID

```rust
use forte_sdk::Uuid;
let id = Uuid::new_v4();
```

## Randomness

Backed by WASI random. All functions are `async`.

```rust
use forte_sdk::rand;

// Fill a buffer with cryptographically secure random bytes
rand::fill_bytes(&mut buf).await;

// Get a Vec<u8> of secure random bytes
let bytes: Vec<u8> = rand::get_random_bytes(32).await;

// Get a single u64
let n: u64 = rand::get_random_u64().await;

// Insecure (fast) variants — not suitable for secrets
rand::get_insecure_random_bytes(&mut buf).await;
let n: u64 = rand::get_insecure_random_u64().await;
```

## Tracing / Logging

```rust
use forte_sdk::tracing;

tracing::info!("processing request");
tracing::error!("something failed: {}", err);
```

OpenTelemetry is initialized once per instance on the first request via `otel::init_once()`, which is called automatically by the `serve` function. Spans are exported to `http://fn0-otel.fn0.dev/v1/traces` (the fn0 Cloud collector) using the OTLP protobuf format.

The service name defaults to `"forte-app"` and can be overridden with the `OTEL_SERVICE_NAME` environment variable.

## Metrics

`forte_sdk::metrics` provides an OTLP metrics pipeline. Instruments created from the shared `Meter` are aggregated with **delta temporality** and flushed at the end of each request — the same path as traces. No background exporter is needed because a forte component only holds CPU during a request.

```rust
use forte_sdk::metrics::{meter, Counter, KeyValue};

// Create a counter once (e.g. at module level with LazyLock)
let requests: Counter<u64> = meter().u64_counter("requests").build();

// Record a measurement inside a handler
requests.add(1, &[KeyValue::new("route", "/api/users")]);
```

Instrument types re-exported from `forte_sdk::metrics`:

| Type | Description |
|---|---|
| `Counter<T>` | Monotonically increasing sum |
| `UpDownCounter<T>` | Sum that can increase and decrease |
| `Gauge<T>` | Last-value point-in-time measurement |
| `Histogram<T>` | Bucketed distribution |
| `KeyValue` | Attribute key/value pair for labels |

Get the shared `Meter` with `forte_sdk::metrics::meter()`. Instruments survive across the process lifetime (module-level `LazyLock` is fine).

Metrics are exported to `http://fn0-otel.fn0.dev/v1/metrics` (the fn0 Cloud collector). The service name defaults to `"forte-app"` and can be overridden with `OTEL_SERVICE_NAME`.

If no instrument has been created during a request the flush is a no-op; there is no overhead for apps that don't use metrics.

## `forte_json` — Serialization Format

`forte-json` is a custom JSON serializer/deserializer used for all handler I/O. It differs from `serde_json` in two ways:

**Serialization (Rust → JSON):**
- Struct field names are converted to camelCase (`user_name` → `"userName"`)
- Enum variants use a `t` discriminant field:

| Variant kind | Rust | JSON |
|---|---|---|
| Unit | `Ok` | `{"t":"Ok"}` |
| Tuple/newtype (1 field) | `Ok(String)` | `{"t":"Ok","v":"..."}` |
| Struct | `Ok { message: String }` | `{"t":"Ok","message":"..."}` |

For struct variants the fields are spread flat alongside `t`; there is no `v` wrapper.

**Deserialization (JSON → Rust):**
- All object keys are converted to snake_case before deserializing (`"userName"` → `"user_name"`)
- This means TypeScript action input shapes use camelCase while Rust struct fields use snake_case

```rust
// Rust
#[derive(Deserialize)]
pub struct Input {
    pub user_name: String,   // receives "userName" from the browser
}
```

The API:
```rust
use forte_sdk::forte_json;

let bytes: Vec<u8> = forte_json::to_vec(&my_value);
let value: MyType = forte_json::from_slice(&bytes)?;
```

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
| `metrics::{meter, Counter, Gauge, Histogram, UpDownCounter, KeyValue}` | `forte-sdk` (OTLP metrics) |

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
