# Pages & API Routes

Pages and API routes are file-based. `forte build` (via `forte_codegen::generate_routes()`) scans `src/pages/` and `src/apis/` at build time and generates routing code.

## File → Route Mapping

| File path | Route |
|-----------|-------|
| `src/pages/index.rs` | `/` |
| `src/pages/about.rs` | `/about` |
| `src/pages/blog/index.rs` | `/blog` |
| `src/pages/blog/[id].rs` | `/blog/:id` (dynamic) |
| `src/apis/users.rs` | `/api/users` |

- `index.rs` or `mod.rs` maps to the parent directory's route.
- Segments wrapped in `[brackets]` are dynamic.
- API routes are prefixed with `/api/`.

## Handler Signature

Every page or API file must expose a `pub async fn handler`. The exact signature depends on what parameter structs are defined in the same file:

```rust
// No path or search params
pub async fn handler(req: ForteRequest) -> anyhow::Result<Props> { ... }

// With path params
pub async fn handler(req: ForteRequest, path_params: PathParams) -> anyhow::Result<Props> { ... }

// With search params
pub async fn handler(req: ForteRequest, search_params: SearchParams) -> anyhow::Result<Props> { ... }

// With both
pub async fn handler(req: ForteRequest, path_params: PathParams, search_params: SearchParams) -> anyhow::Result<Props> { ... }
```

Code generation inspects the file with `syn` to determine which structs exist.

## Props (Page Data)

For pages (SSR), `handler` returns `anyhow::Result<Props>`. On success the props are serialized to JSON by `forte_json::to_vec` and returned with header `x-fn0-next: js` — signaling fn0 to delegate rendering to the JS runtime (React SSR).

```rust
pub type Props = MyProps;  // or any serializable type

#[derive(serde::Serialize)]
pub struct MyProps {
    pub title: String,
    pub items: Vec<String>,
}
```

For API routes, the return value is serialized as JSON and returned with `Content-Type: application/json`.

## Redirect

Handlers can redirect by returning `Err(Redirect::SomePage.into())` where `Redirect` is the generated enum of all routes:

```rust
pub async fn handler(req: ForteRequest) -> anyhow::Result<Props> {
    if !authenticated {
        return Err(Redirect::Login.into());
    }
    // ...
}
```

For pages whose `Props` type alias is itself `Redirect`, the handler returns `anyhow::Result<Redirect>` and code generation emits a pure-redirect handler (no props serialization).

## Path Parameters

Define a `PathParams` struct in the same file:

```rust
pub struct PathParams {
    pub id: u64,
}
```

The segment name in brackets must match the field name. Parsing failures return 400.

## Search Parameters

Define a `SearchParams` struct:

```rust
pub struct SearchParams {
    pub page: u32,           // required — missing → 400
    pub q: Option<String>,   // optional
}
```

All parsing is done by `.parse::<T>()`. Parse failures return 400.

## ForteRequest

`ForteRequest` is defined in `forte-sdk`:

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

For pages/APIs, `body` is `()`. For actions and hooks it is the deserialized `Input` type.

## Generated TypeScript Paths

`forte build` also writes `../fe/src/paths.generated.ts`:

```typescript
export const paths = {
  "/": () => "/",
  "/blog/:id": ({id}: {id: string}) => `/blog/${id}`,
} as const;
```

Use this on the frontend to construct type-safe URLs.
