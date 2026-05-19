# Forte Actions

Actions are server-side functions callable from the browser via HTTP POST. They accept a typed JSON input and return a typed JSON output.

## File Location

Place action handlers under `rs/src/actions/`. Two layouts are supported:

```
rs/src/actions/user_login.rs        →  POST /__forte_action/user_login
rs/src/actions/products_list.rs     →  POST /__forte_action/products_list
rs/src/actions/user_login/mod.rs    →  POST /__forte_action/user_login  (directory module)
```

The codegen discovers actions by scanning the top-level `src/actions/` directory (non-recursive) for:
- `.rs` files with `struct Input`, an `Output` type, and `pub async fn handler`
- subdirectories containing a `mod.rs` with the same structure

## Rust Handler

```rust
// rs/src/actions/user_login.rs
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok { token: String },
    InvalidCredentials,
    Error { message: String },
}

pub async fn handler(req: forte_sdk::ForteRequest<'_, Input>) -> Output {
    let input = &req.body;
    // validate credentials ...
    Output::Ok {
        token: "...".to_string(),
    }
}
```

Key conventions:
- `Input` — the deserialized request body (JSON); **must be a struct named exactly `Input`** (not an enum)
- `Output` — the serialized response body (JSON); **must be named exactly `Output`** (struct or enum)
- `handler` — must be `pub async fn`, takes `ForteRequest<'_, Input>`; **must be named exactly `handler`**
- Return type: `Output` directly (not `Result<Output>`). The codegen calls `forte_json::to_vec(&output)` on the return value, so it must implement `Serialize`. `anyhow::Error` does not implement `Serialize`, so `anyhow::Result<Output>` will not compile.

`forte add action <name>` generates a backend file with the correct names and signatures ready to compile.

## TypeScript Client

`forte add action` creates only the Rust backend file. On the next `forte build` or `forte dev`, `forte-rs-to-ts` generates a fully-typed Zod client at `fe/src/actions/.generated/<name>.ts`. Import it directly:

```ts
import { submit } from "../../actions/.generated/submit";

const result = await submit({ message: "hello" });
```

The generated file exports:
- `InputSchema` / `Input` — Zod schema and TypeScript type for the action input
- `OutputSchema` / `Output` — Zod schema and TypeScript type for the action output
- A `callAction` helper (or equivalent typed fetch function)

Do not hand-write a TypeScript wrapper for actions — use the generated file.

## Accessing Cookies and Headers

Actions receive the full `ForteRequest` context, so you can read and write cookies:

```rust
use forte_sdk::cookie_sign::{sign_cookie, unsign_cookie};

pub async fn handler(req: forte_sdk::ForteRequest<'_, Input>) -> Output {
    // read
    let session: Option<Session> = unsign_cookie(req.jar, "session");

    // write
    sign_cookie(req.jar, "session", &session, None);

    // read headers
    let auth = req.headers.get("authorization");

    Output::Ok { ... }
}
```

Cookie changes made in `req.jar` are written back to `Set-Cookie` response headers automatically.

## Hooks (Self-Invoke)

Hooks are similar to actions but are invoked server-to-server (via `/__self_invoke/<name>`). They live under `rs/src/hooks/` and follow the same `Input` / `Output` / `handler` convention. They are called internally via the fn0 queue or control plane, not from the browser directly.

## Deserialization: `forte_json` vs `serde_json`

Action inputs are deserialized with `forte_json`, which converts camelCase keys from the browser to snake_case Rust field names. A TypeScript caller sending `{"userName": "alice"}` maps to a Rust field `pub user_name: String`.

Queue task and admin task inputs are deserialized with standard `serde_json` (no key conversion). Their struct field names must match the JSON keys exactly as sent by the caller — typically the generated `enqueue::*` functions (for queue tasks) or the `--input` JSON provided to `forte admin run` (for admin tasks). Use snake_case field names to match the default serde naming.

## Queue Tasks

Background tasks live under `rs/src/queue_task/`. They have an `Input` struct and a `pub async fn handle` (not `handler`) function:

```rust
// rs/src/queue_task/send_email.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]  // both required: Deserialize for execution, Serialize for enqueue
pub struct Input {
    pub to: String,
    pub subject: String,
    pub body: String,
}

pub async fn handle(input: Input) -> Result<()> {
    // send email ...
    Ok(())
}
```

To enqueue a task from anywhere in the backend:

```rust
use crate::enqueue;
use crate::queue_task::send_email;

enqueue::send_email(send_email::Input {
    to: "user@example.com".to_string(),
    subject: "Welcome".to_string(),
    body: "...".to_string(),
}).await?;
```

The `enqueue` module is generated automatically when queue tasks exist. It requires the `FN0_QUEUE_URL` environment variable to be set.

## Admin Tasks

Admin tasks live under `rs/src/admin/`. Same `Input` / `handle` convention as queue tasks. They are called via `forte admin run <name>` and require the `x-fn0-admin: true` header (added automatically by the CLI). Unauthenticated requests get HTTP 401.

```rust
// rs/src/admin/seed_database.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input {
    pub count: u32,
}

#[derive(Serialize)]
pub struct Output {
    pub created: u32,
}

pub async fn handle(input: Input) -> Result<Output> {
    // seed logic ...
    Ok(Output { created: input.count })
}
```

Run it:

```sh
forte admin run seed_database --input '{"count": 10}'
```
