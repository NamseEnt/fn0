# Forte API Endpoints

API endpoints live under `rs/src/apis/` and return JSON directly to the client. Unlike page handlers, they do not trigger the SSR step — there is no React rendering or `x-fn0-next: js` header.

## File Location and Route Mapping

Files in `rs/src/apis/` map to routes prefixed with `/api/`. Discovery is recursive, so subdirectories work:

| File path | Route |
|---|---|
| `rs/src/apis/users.rs` | `GET /api/users` |
| `rs/src/apis/products.rs` | `GET /api/products` |
| `rs/src/apis/orders/[id]/mod.rs` | `GET /api/orders/:id` |

There is no `forte add api` command. Create files manually.

## Handler Signature

An API handler looks identical to a page handler, but the return type **must be named `Props`** (or contain the string `"Props"`) for codegen to discover it:

```rust
// rs/src/apis/users.rs
use anyhow::Result;
use forte_sdk::ForteRequest;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok { users: Vec<User> },
    Empty,
}

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
}

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    // ... fetch users ...
    Ok(Props::Ok { users: vec![] })
}
```

The response is serialized with `forte_json::to_vec`, so:
- Struct field names become camelCase (`user_id` → `"userId"`)
- Enum variants use the `t` discriminant (same as page Props)

The response is sent with `Content-Type: application/json` and HTTP 200.

## Path and Search Parameters

API endpoints support the same `PathParams` and `SearchParams` conventions as pages:

```rust
// rs/src/apis/orders/[id]/mod.rs
use anyhow::Result;
use forte_sdk::ForteRequest;
use serde::Serialize;

pub struct PathParams {
    pub id: String,
}

pub struct SearchParams {
    pub include_items: Option<bool>,
}

#[derive(Serialize)]
pub enum Props {
    Ok { id: String, total: f64 },
    NotFound,
}

pub async fn handler(
    _req: ForteRequest<'_>,
    path: PathParams,
    search: SearchParams,
) -> Result<Props> {
    // ...
    Ok(Props::Ok { id: path.id, total: 99.99 })
}
```

## Accessing Request Data

API handlers receive the full `ForteRequest` context, so you can read headers, cookies, and the raw body:

```rust
pub async fn handler(req: ForteRequest<'_>) -> Result<Props> {
    let auth = req.headers.get("authorization");
    let raw = req.raw_body;
    // ...
}
```

For POST endpoints that accept a JSON body, use `forte_sdk::forte_json::from_slice`:

```rust
use forte_sdk::forte_json;
use serde::Deserialize;

#[derive(Deserialize)]
struct CreateUserRequest {
    pub name: String,
}

pub async fn handler(req: ForteRequest<'_>) -> Result<Props> {
    let body: CreateUserRequest = forte_json::from_slice(req.raw_body)?;
    // body.name is parsed with camelCase→snake_case key conversion
    // ...
}
```

## Discovery Rules

Codegen discovers API endpoints by scanning `rs/src/apis/` recursively (including subdirectories). A file is included if it contains `pub async fn handler` with a return type containing both `"Result"` and `"Props"`.

`mod.rs` at any subdirectory level maps to the handler for that directory (e.g., `apis/orders/mod.rs` → `/api/orders`). There is no generated `mod.rs` in the `apis/` directory.

## Differences from Page Handlers

| Feature | Page (`src/pages/`) | API (`src/apis/`) |
|---|---|---|
| Route prefix | none | `/api/` |
| Response | `x-fn0-next: js` → SSR | `Content-Type: application/json` |
| `Redirect` | supported (via `Err(Redirect::...)`) | supported (same mechanism) |
| React component | required (`fe/src/pages/`) | not applicable |
| `.props.ts` generated | yes | no |

## Error Responses

When the handler returns `Err(e)`:
- If `e` downcasts to `Redirect`, the response is HTTP 302 with a `Location` header.
- Otherwise, the response is HTTP 500 with the error debug string in the body. The error is also printed to stderr.

## `paths.generated.ts`

API routes are included in the `paths.generated.ts` object alongside page routes:

```ts
import { paths } from "./paths.generated";
paths["/api/users"]();                      // "/api/users"
paths["/api/orders/:id"]({ id: "42" });     // "/api/orders/42"
```
