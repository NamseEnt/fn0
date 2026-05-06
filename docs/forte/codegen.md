# Forte Code Generation

Forte uses two code generation tools:

1. **`forte-codegen`** — a build-script library that generates `route_generated.rs` and TypeScript helpers from Rust source files
2. **`forte-rs-to-ts`** — a standalone binary that extracts Rust types and converts them to TypeScript

Both run automatically during `forte build` and `forte dev`.

## `forte-codegen` (build.rs)

Every Forte project's `build.rs` calls:

```rust
fn main() {
    forte_codegen::generate_routes();
}
```

This scans the backend source tree and writes:

| Output | Description |
|---|---|
| `rs/src/route_generated.rs` | Complete HTTP dispatch, WASM export, and route matching |
| `fe/src/paths.generated.ts` | Type-safe path builder object |
| `rs/src/actions/mod.rs` | Module declarations for discovered actions |
| `rs/src/admin/mod.rs` | Module declarations for discovered admin tasks |
| `rs/src/queue_task/mod.rs` | Module declarations for discovered queue tasks |

It also updates the `FORTE-MANAGED` block in `rs/src/lib.rs`.

Cargo reruns the build script when any of these directories change:
- `src/pages`
- `src/apis`
- `src/hooks`
- `src/actions`
- `src/queue_task`
- `src/admin`
- `Cargo.lock`

### Discovery Rules

Each handler type is discovered by statically parsing the Rust source:

| Directory | Discovery rule |
|---|---|
| `src/pages/` | File contains `pub async fn handler` returning `Result<Props>` or `Result<Redirect>` |
| `src/apis/` | Same as pages; route is prefixed with `/api/` |
| `src/hooks/` | File has `struct Input`, `Output` type, and `pub async fn handler` |
| `src/actions/` | File has `struct Input`, `Output` type, and `pub async fn handler` |
| `src/queue_task/` | File has `struct Input` and `pub async fn handle` |
| `src/admin/` | Same as queue tasks |

Files named `mod.rs` inside these directories are skipped (they are generated).

### Route Mapping

| File path | Route |
|---|---|
| `pages/index/mod.rs` or `pages/index.rs` | `/` |
| `pages/about/mod.rs` | `/about` |
| `pages/product/[id]/mod.rs` | `/product/:id` |
| `apis/users.rs` | `/api/users` |

Dynamic segments are `[param]` in the directory name, mapped to `:param` in the route.

### Generated Router Internals (`route_generated.rs`)

The generated file:
- Declares all discovered modules with `#[path = "..."] mod ...`
- Defines the WASM component export via `wit_bindgen::generate!`
- Implements `wasi::http::handler::Guest` which calls `forte_sdk::serve::serve`
- Defines `dispatch_inner` which routes to pages, API endpoints, actions, hooks, queue tasks, and admin tasks
- Handles path parameter extraction (`PathParams`) and search parameter parsing (`SearchParams`)
- Generates the `Redirect` enum with variants for every discovered page

**Special routes handled by the generated dispatcher:**

| Path prefix | Handler |
|---|---|
| `/__self_invoke/<name>` | Hook handler |
| `/__forte_action/<name>` | Action handler |
| `/__fn0_queue_task/execute` | Queue task executor |
| `/__forte_admin/<name>` | Admin task handler |

### `paths.generated.ts`

Emitted alongside `route_generated.rs`. Exports a `paths` const with one entry per page:

```ts
export const paths = {
  "/": () => "/",
  "/product/:id": ({ id }: { id: string }) => `/product/${id}`,
} as const;
```

Rust types are mapped to TypeScript: `String`/`&str` → `string`, integer types → `number`, `bool` → `boolean`.

## `forte-rs-to-ts`

A standalone binary (`forte-rs-to-ts`) that reads the Rust source tree and generates TypeScript type files. Run automatically during `forte build`.

For each page handler (`rs/src/pages/<path>/mod.rs`), it generates:
- `fe/src/pages/<path>/.props.ts` — the `Props` type as a TypeScript discriminated union

For each action handler (`rs/src/actions/<name>.rs`), it generates:
- Type files for `Input` and `Output` (format: Unknown from repository — check `forte/rs-to-ts/`)

The generated Props type uses `{ t: "VariantName", v: { ... } }` shape for enum variants with fields.

## The `FORTE-MANAGED` Block

`rs/src/lib.rs` must contain this block for codegen to work:

```rust
// === FORTE-MANAGED START ===
// Auto-managed by `forte build`. Do not edit between the START/END markers.
mod route_generated;
// === FORTE-MANAGED END ===
```

`forte-codegen` replaces the content between the markers with the appropriate module declarations. If the markers are missing, the build panics with instructions to add them.
