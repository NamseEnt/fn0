# Forte Pages

Pages are Rust handlers that return server-side data (Props) which the React frontend renders.

## File Location

Place page handlers under `rs/src/pages/`. The directory structure maps directly to URL routes:

| File path | URL |
|---|---|
| `rs/src/pages/index/mod.rs` or `rs/src/pages/index.rs` | `/` |
| `rs/src/pages/about/mod.rs` or `rs/src/pages/about.rs` | `/about` |
| `rs/src/pages/product/[id]/mod.rs` | `/product/:id` |
| `rs/src/pages/blog/[year]/[slug]/mod.rs` | `/blog/:year/:slug` |

Both directory modules (`about/mod.rs`) and flat files (`about.rs`) are supported. The directory layout is preferred for pages that need companion files.

A page is discovered when its file contains `pub async fn handler`.

## Basic Page

A minimal page handler returns a `Props` type via `Result<Props>`:

```rust
// rs/src/pages/index/mod.rs
use anyhow::Result;
use forte_sdk::ForteRequest;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok { message: String },
}

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    Ok(Props::Ok {
        message: "Hello from Forte!".to_string(),
    })
}
```

The `Props` type is serialized to JSON and passed to the React component via the SSR pipeline.

`forte add page <path>` scaffolds a backend handler and a React component with the correct signatures and patterns shown above.

## ForteRequest

`ForteRequest<'a, Body>` is the request context injected into handlers:

```rust
pub struct ForteRequest<'a, Body = ()> {
    pub uri_authority: &'a str,       // e.g. "example.com"
    pub method: &'a http::Method,
    pub headers: &'a http::HeaderMap,
    pub jar: &'a mut CookieJar,       // mutable: add/remove cookies here
    pub raw_body: &'a [u8],
    pub body: Body,                    // typed body (default: ())
}
```

## Path Parameters

For dynamic routes like `/product/[id]`, declare a `PathParams` struct:

```rust
// rs/src/pages/product/[id]/mod.rs
use anyhow::Result;
use forte_sdk::ForteRequest;
use serde::{Deserialize, Serialize};

pub struct PathParams {
    pub id: String,
}

#[derive(Serialize)]
pub enum Props {
    Ok { id: String },
}

pub async fn handler(_req: ForteRequest<'_>, params: PathParams) -> Result<Props> {
    Ok(Props::Ok { id: params.id })
}
```

The codegen reads the `PathParams` struct and generates the extraction code. The parameter names must match the directory segment names (e.g. `[id]` → field named `id`).

Supported param types: `String`, and any type that implements `FromStr` (e.g. `u32`, `i64`). Non-`String` types that fail to parse return HTTP 400.

## Search Parameters

For query-string parameters, declare a `SearchParams` struct:

```rust
pub struct SearchParams {
    pub q: String,           // required
    pub page: Option<u32>,   // optional
}

pub async fn handler(_req: ForteRequest<'_>, search: SearchParams) -> Result<Props> {
    // ...
}
```

Required `SearchParams` fields that are missing return HTTP 400. Optional fields use `Option<T>`.

## Both Path and Search Parameters

```rust
pub async fn handler(
    _req: ForteRequest<'_>,
    path: PathParams,
    search: SearchParams,
) -> Result<Props>
```

## Redirects

A handler can redirect instead of rendering. Use `anyhow::Error` with a `Redirect` value:

```rust
use anyhow::Result;
use forte_sdk::ForteRequest;

// Type alias triggers redirect-only mode
pub type Props = Redirect;

pub async fn handler(_req: ForteRequest<'_>) -> Result<Redirect> {
    // Redirect to another page
    Err(Redirect::About.into())
}
```

Or return a redirect from an error path:

```rust
pub async fn handler(req: ForteRequest<'_>) -> Result<Props> {
    if !is_authenticated(&req) {
        return Err(Redirect::Login.into());
    }
    Ok(Props::Ok { ... })
}
```

`Redirect` is a generated enum with one variant per page. `Redirect::External { url }` redirects to an arbitrary URL.

## Error Handling

If a handler returns `Err` and the error is not a `Redirect`, the error is logged to stderr and the client receives HTTP 500 "Internal Server Error". The error message does not reach the client. Use structured logging (`tracing::error!`) to capture errors with span context, since `eprintln!` output only goes to the worker's stderr.

```rust
pub async fn handler(req: ForteRequest<'_>) -> Result<Props> {
    let data = fetch_data().await.map_err(|e| {
        tracing::error!(error = ?e, "fetch_data failed");
        e   // Err propagates to HTTP 500
    })?;
    Ok(Props::Ok { data })
}
```

## Cookies

Read and write cookies via `req.jar`:

```rust
use forte_sdk::cookie_sign::{sign_cookie, unsign_cookie};
use forte_sdk::time;

pub async fn handler(req: ForteRequest<'_>) -> Result<Props> {
    // Read a signed cookie
    let user: Option<User> = unsign_cookie(req.jar, "session");

    // Write a signed cookie
    sign_cookie(
        req.jar,
        "session",
        &user,
        Some(time::Duration::days(30)),
    );

    Ok(Props::Ok { ... })
}
```

`sign_cookie` / `unsign_cookie` use HMAC-SHA256 with the `COOKIE_SECRET` environment variable. Cookies are set `HttpOnly`, `Secure`, `Path=/`.

## React Component

The frontend counterpart lives at `fe/src/pages/<path>/page.tsx`:

```tsx
// fe/src/pages/index/page.tsx
import type { Props } from "./.props";

export default function IndexPage(props: Props) {
    if (props.t !== "Ok") {
        return <div>Error</div>;
    }
    return <div>{props.message}</div>;
}
```

The `.props.ts` file is auto-generated from the Rust `Props` type by `forte-rs-to-ts`. It exports a Zod schema and a `Props` type as a TypeScript discriminated union on the `t` field.

The JSON shape varies by variant kind:

| Rust variant kind | JSON shape | TypeScript type |
|---|---|---|
| Unit: `Ok` | `{"t":"Ok"}` | `{ t: "Ok" }` |
| Tuple/newtype: `Ok(String)` | `{"t":"Ok","v":"..."}` | `{ t: "Ok", v: string }` |
| Struct: `Ok { message: String }` | `{"t":"Ok","message":"..."}` | `{ t: "Ok", message: string }` |

Struct variant fields are spread alongside `t` (flat); there is no `v` wrapper. Rust field names are converted to camelCase (`user_name` → `userName`).

**Example** for `Props::Ok { message: String }`:
- JSON from Rust: `{"t":"Ok","message":"Hello"}`
- TypeScript access: `props.message` (after narrowing with `props.t === "Ok"`)

## API Endpoints

Files under `rs/src/apis/` work the same as pages but return JSON directly (no SSR). The route is prefixed with `/api/`:

```
rs/src/apis/users.rs  →  GET /api/users
```

The handler signature is identical, but the response is serialized as JSON and returned with `Content-Type: application/json` — the `x-fn0-next: js` header is not set, so the JS runtime is not invoked.
