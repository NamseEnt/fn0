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

## `generate_env` (build.rs)

`forte-codegen` also exports a second build-script function:

```rust
fn main() {
    forte_codegen::generate_routes();
    forte_codegen::generate_env(); // optional, call after generate_routes
}
```

When called, `generate_env` reads the project's `.env` file (located one directory above `rs/`, i.e. `<project>/.env`) and writes `rs/src/env_generated.rs`. For each `KEY=value` line it generates a zero-cost accessor:

```rust
pub fn cookie_secret() -> &'static str { ... }  // for COOKIE_SECRET=...
pub fn turso_url() -> &'static str { ... }       // for TURSO_URL=...
```

Values are loaded from the real environment at first use via `std::sync::LazyLock`. This means the function always returns the runtime value of the variable, not the value in `.env`; `.env` is only used to determine which accessor functions to generate.

`cargo:rerun-if-changed=../.env` is emitted automatically so the file is regenerated when `.env` changes.

To use the generated module, add `mod env_generated;` to your `lib.rs` and call `env_generated::cookie_secret()` etc.

> **Note:** `generate_env` is not called by default. You must add it to `build.rs` explicitly.

## `forte-rs-to-ts`

A standalone binary (`forte-rs-to-ts`) that reads the Rust source tree and generates TypeScript files. Run automatically during `forte build` and `forte dev`.

### Pages → `.props.ts`

For each page handler (`rs/src/pages/<path>/mod.rs`), it generates:

```
fe/src/pages/<path>/.props.ts
```

The file exports a Zod schema and a `Props` TypeScript type:

```ts
import { z } from "zod";

export const PropsSchema = z.discriminatedUnion("t", [
    z.object({ t: z.literal("Ok"), message: z.string() }),
]);

export type Props = z.infer<typeof PropsSchema>;
```

Variant shape mapping (same as pages.md):

| Rust variant kind | JSON / TypeScript shape |
|---|---|
| Unit: `Ok` | `{ t: "Ok" }` |
| Newtype/tuple: `Ok(String)` | `{ t: "Ok", v: string }` |
| Struct: `Ok { message: String }` | `{ t: "Ok", message: string }` (fields spread flat, no `v` wrapper) |

### Actions → callable functions

For each action handler (`rs/src/actions/<name>.rs`), it generates:

```
fe/src/actions/.generated/<name>.ts
```

The file exports a camelCase function that wraps `callAction` from `@forte/react`:

```ts
import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({ ... });
const OutputSchema = z.discriminatedUnion("t", [ ... ]);

export function userLogin(input: z.infer<typeof InputSchema>) {
    return callAction("user_login", input, OutputSchema);
}
```

An `index.ts` re-exporting all action functions is also generated at `fe/src/actions/.generated/index.ts`.

### Hooks → React hooks

For each hook handler (`rs/src/hooks/<name>.rs`), it generates:

```
fe/src/hooks/.generated/use<Name>.ts
```

The file exports a React hook using `useForteHook` from `@forte/react`:

```ts
import { z } from "zod";
import { useForteHook } from "@forte/react";

const InputSchema = z.object({ ... });
const OutputSchema = z.discriminatedUnion("t", [ ... ]);

export function useMyHook(input: z.infer<typeof InputSchema>) {
    return useForteHook("my_hook", input, OutputSchema);
}
```

`useForteHook` uses React Suspense (throws a promise) to fetch the hook result server-side during SSR and cache-hit on the client.

### `@forte/react`

`@forte/react` is a Vite module alias resolved to a file bundled by `forte-cli`. It is **not** a published npm package. Both `callAction` and `useForteHook` are defined there.

## The `FORTE-MANAGED` Block

`rs/src/lib.rs` must contain this block for codegen to work:

```rust
// === FORTE-MANAGED START ===
// Auto-managed by `forte build`. Do not edit between the START/END markers.
mod route_generated;
// === FORTE-MANAGED END ===
```

`forte-codegen` replaces the content between the markers with the appropriate module declarations. If the markers are missing, the build panics with instructions to add them.
