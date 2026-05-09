# Forte Actions

Actions are server-side functions callable from the browser via HTTP POST. They accept a typed JSON input and return a typed JSON output.

## File Location

Place action handlers under `rs/src/actions/`. Each file becomes one action:

```
rs/src/actions/user_login.rs   →  POST /__forte_action/user_login
rs/src/actions/products_list.rs →  POST /__forte_action/products_list
```

The codegen discovers actions by looking for files with an `Input` struct and a `pub async fn handler`.

## Rust Handler

```rust
// rs/src/actions/user_login.rs
use anyhow::Result;
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

pub async fn handler(req: forte_sdk::ForteRequest<'_, Input>) -> Result<Output> {
    let input = &req.body;
    // validate credentials ...
    Ok(Output::Ok {
        token: "...".to_string(),
    })
}
```

Key conventions:
- `Input` — the deserialized request body (JSON)
- `Output` — the serialized response body (JSON)
- `handler` — must be `pub async fn`, takes `ForteRequest<'_, Input>`
- Return type: `Result<Output>` (using `anyhow::Result`)

## TypeScript Client

`forte-rs-to-ts` (run automatically by `forte dev` and `forte build`) generates a typed caller in `fe/src/actions/.generated/<name>.ts`:

```ts
// fe/src/actions/.generated/user_login.ts  (auto-generated — do not edit)
import { z } from "zod";
import { callAction } from "@forte/react";

const InputSchema = z.object({
    email: z.string(),
    password: z.string(),
});

const OutputSchema = z.discriminatedUnion("t", [
    z.object({ t: z.literal("Ok"), v: z.object({ token: z.string() }) }),
    z.object({ t: z.literal("InvalidCredentials") }),
    z.object({ t: z.literal("Error"), v: z.object({ message: z.string() }) }),
]);

export function userLogin(input: z.infer<typeof InputSchema>) {
    return callAction("user_login", input, OutputSchema);
}
```

An index re-export is also generated at `fe/src/actions/.generated/index.ts`:

```ts
// fe/src/actions/.generated/index.ts  (auto-generated — do not edit)
export { userLogin } from "./user_login";
```

Import actions from the generated index:

```ts
import { userLogin } from "./actions/.generated";

const result = await userLogin({ email: "user@example.com", password: "secret" });
// result.t === "Ok" | "InvalidCredentials" | "Error"
```

`callAction` posts to `/__forte_action/<name>`, validates the response against `OutputSchema` (Zod), and returns the typed result. Requires `@forte/react`.

## Accessing Cookies and Headers

Actions receive the full `ForteRequest` context, so you can read and write cookies:

```rust
use forte_sdk::cookie_sign::{sign_cookie, unsign_cookie};

pub async fn handler(req: forte_sdk::ForteRequest<'_, Input>) -> Result<Output> {
    // read
    let session: Option<Session> = unsign_cookie(req.jar, "session");

    // write
    sign_cookie(req.jar, "session", &session, None);

    // read headers
    let auth = req.headers.get("authorization");

    Ok(Output::Ok { ... })
}
```

Cookie changes made in `req.jar` are written back to `Set-Cookie` response headers automatically.

## Hooks (Self-Invoke)

Hooks live under `rs/src/hooks/` and follow the same `Input` / `Output` / `handler` convention as actions. They are served at `/__self_invoke/<name>`.

`forte-rs-to-ts` generates a React hook in `fe/src/hooks/.generated/<name>.ts`:

```ts
// fe/src/hooks/.generated/notify_user.ts  (auto-generated)
import { z } from "zod";
import { useForteHook } from "@forte/react";

const InputSchema = z.object({ ... });
const OutputSchema = z.object({ ... });

export function useNotifyUser(input: z.infer<typeof InputSchema>) {
    return useForteHook("notify_user", input, OutputSchema);
}
```

Use the generated hook in a React component to call the hook endpoint reactively.

## Queue Tasks

Background tasks live under `rs/src/queue_task/`. They have an `Input` struct and a `pub async fn handle` (not `handler`) function:

```rust
// rs/src/queue_task/send_email.rs
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
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

enqueue::send_email(enqueue::send_email::Input {
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
