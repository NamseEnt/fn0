# Forte Code Generation

Forte uses two code generation tools:

1. **`forte-codegen`** — a build-script library that generates `route_generated.rs` and TypeScript helpers from Rust source files
2. **`forte-rs-to-ts`** — a standalone binary that extracts Rust types and converts them to TypeScript

Both run automatically during `forte build` and `forte dev`.

## `forte-codegen` (build.rs)

Every Forte project's `build.rs` calls one or both codegen functions:

```rust
fn main() {
    forte_codegen::generate_routes();
    forte_codegen::generate_env(); // optional, see below
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

### `generate_env` (optional)

`forte_codegen::generate_env()` reads the project's `.env` file and generates `rs/src/env_generated.rs` with typed accessor functions for each variable:

```rust
// rs/src/env_generated.rs  (auto-generated)
pub fn database_url() -> &'static str { ... }
pub fn api_key() -> &'static str { ... }
```

Each function returns a `&'static str` cached in a `LazyLock`. Missing variables panic at first call (not at startup).

Cargo reruns the build script whenever `../.env` changes.

## `forte-rs-to-ts`

A standalone binary (`forte-rs-to-ts`) that reads the Rust source tree and generates TypeScript type files. Run automatically during `forte build`.

For each page handler (`rs/src/pages/<path>/mod.rs`), it generates:
- `fe/src/pages/<path>/.props.ts` — a Zod schema (`PropsSchema`) and an inferred `Props` type

For each action handler (`rs/src/actions/<name>.rs`), it generates:
- `fe/src/actions/.generated/<name>.ts` — Zod schemas for `Input` and `Output`, plus a typed `callAction` wrapper function
- `fe/src/actions/.generated/index.ts` — re-exports all generated action callers

For each hook handler (`rs/src/hooks/<name>.rs`), it generates:
- `fe/src/hooks/.generated/<name>.ts` — Zod schemas and a React hook using `useForteHook` from `@forte/react`

The generated `.props.ts` file structure:

```ts
// Auto-generated from rs/src/pages/product/[id]/mod.rs

import { z } from "zod";

export const PropsSchema = z.discriminatedUnion("t", [
  z.object({ t: z.literal("Ok"), v: z.object({ id: z.string() }) }),
  z.object({ t: z.literal("NotFound") }),
]);

export type Props = z.infer<typeof PropsSchema>;
```

Enum variants use `{ t: "VariantName", v: { ... } }` shape. Unit variants (no fields) omit `v`. Supporting types referenced by `Props` are generated as named Zod schemas in the same file.

## The `FORTE-MANAGED` Block

`rs/src/lib.rs` must contain this block for codegen to work:

```rust
// === FORTE-MANAGED START ===
// Auto-managed by `forte build`. Do not edit between the START/END markers.
mod route_generated;
// === FORTE-MANAGED END ===
```

`forte-codegen` replaces the content between the markers with the appropriate module declarations. If the markers are missing, the build panics with instructions to add them.
