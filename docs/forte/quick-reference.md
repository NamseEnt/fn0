# Forte Quick Reference

Concise patterns for the most common Forte tasks. Follow links to the full docs for details.

## Project Lifecycle

```sh
forte init my-app     # scaffold new project
cd my-app
forte dev             # start dev server (HMR + Rust rebuild on change)

forte add page about          # add page at /about
forte add page product/[id]   # add page at /product/:id
forte add action user_login   # add server action

forte build           # verify production build
forte login           # authenticate with fn0 Cloud
forte deploy          # build + upload to fn0 Cloud
```

## Handler Types at a Glance

| Type | Directory | Route | Trigger |
|---|---|---|---|
| Page | `rs/src/pages/` | `GET /<path>` | Browser navigation; runs SSR → React |
| API | `rs/src/apis/` | `/api/<path>` (any method) | Direct JSON; no React rendering |
| Action | `rs/src/actions/` | `POST /__forte_action/<name>` | Called from browser via typed TS client |
| Hook | `rs/src/hooks/` | `POST /__self_invoke/<name>` | Called during SSR; result cached in HTML |
| Queue task | `rs/src/queue_task/` | internal | Background job via `enqueue!` |
| Admin task | `rs/src/admin/` | internal | `forte admin run <name>` |

## Page Handler

```rust
// rs/src/pages/about/mod.rs  →  GET /about
use anyhow::Result;
use forte_sdk::ForteRequest;
use serde::Serialize;

#[derive(Serialize)]
pub enum Props {
    Ok { title: String },
}

pub async fn handler(_req: ForteRequest<'_>) -> Result<Props> {
    Ok(Props::Ok { title: "About".to_string() })
}
```

Dynamic path — add `PathParams`:

```rust
// rs/src/pages/product/[id]/mod.rs  →  GET /product/:id
pub struct PathParams { pub id: String }

pub async fn handler(_req: ForteRequest<'_>, params: PathParams) -> Result<Props> {
    Ok(Props::Ok { id: params.id })
}
```

React component (`fe/src/pages/about/page.tsx`):

```tsx
import type { Props } from "./.props";  // auto-generated

export default function AboutPage(props: Props) {
    if (props.t !== "Ok") return null;
    return <h1>{props.title}</h1>;
}
```

See [pages.md](pages.md) for search params, redirects, cookies, and error handling.

## Action (Server Mutation)

```rust
// rs/src/actions/user_login.rs  →  POST /__forte_action/user_login
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]  // receives camelCase JSON from browser
pub struct Input {
    pub email: String,      // ← TypeScript sends { email, password }
    pub password: String,
}

#[derive(Serialize)]
pub enum Output {
    Ok { token: String },
    InvalidCredentials,
}

pub async fn handler(req: forte_sdk::ForteRequest<'_, Input>) -> Output {
    let input = &req.body;
    // validate ...
    Output::Ok { token: "...".to_string() }
}
```

TypeScript call (auto-generated client):

```ts
import { userLogin } from "../../actions/.generated/user_login";

const result = await userLogin({ email, password });
if (result.t === "Ok") { /* result.token */ }
```

See [actions.md](actions.md) for hooks, queue tasks, and admin tasks.

## Hook (SSR Data Fetch)

```rust
// rs/src/hooks/user_session.rs   (manual — no forte add hook command)
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Input { pub session_token: String }

#[derive(Serialize)]
pub enum Output { Ok { name: String }, Unauthorized }

pub async fn handler(req: forte_sdk::ForteRequest<'_, Input>) -> Output { ... }
```

Auto-generated React hook (`fe/src/hooks/.generated/UserSession.ts`):

```tsx
import { useUserSession } from "../hooks/.generated/UserSession";

function Profile() {
    const session = useUserSession({ sessionToken: getCookie("session") });
    if (session.t !== "Ok") return null;
    return <div>{session.name}</div>;
}
```

Invalidate after mutation:

```ts
import { clearHookCache } from "@forte/react";
await submitAction({ ... });
clearHookCache("user_session");   // re-fetch on next render
```

## Queue Task (Background Job)

```rust
// rs/src/queue_task/send_email.rs
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Input { pub to: String, pub subject: String }

pub async fn handle(input: Input) -> anyhow::Result<()> {
    // send email ...
    Ok(())
}
```

Enqueue from a handler:

```rust
use crate::enqueue;
use crate::queue_task::send_email;

enqueue::send_email(send_email::Input {
    to: "user@example.com".to_string(),
    subject: "Welcome".to_string(),
}).await?;
```

Cron job (scheduled task with empty input):

```yaml
# cron.yaml
- function: send_digest_email
  every_minutes: 60
```

## Key Naming Rules

| Handler | File name | `Input` | Output | Entry point |
|---|---|---|---|---|
| Action | `<snake>.rs` or `<snake>/mod.rs` | `struct Input` | `Output` (any name) | `pub async fn handler` |
| Hook | `<snake>.rs` (flat only) | `struct Input` | `Output` | `pub async fn handler` |
| Queue task | `<snake>.rs` (flat only) | `struct Input` or `type Input = ()` | — | `pub async fn handle` |
| Admin task | `<snake>.rs` (flat only) | `struct Input` or `type Input = ()` | — | `pub async fn handle` |

- Action/hook: function is named `handler`; queue/admin task: function is named `handle`.
- Codegen discovers handlers by static analysis — name and signature must match exactly.
- No `mod` declarations needed in `lib.rs`; codegen updates the FORTE-MANAGED block automatically.

## JSON Serialization Rules (`forte_json`)

Used for all handler I/O (except queue/admin task deserialization, which uses `serde_json`):

| Direction | Transformation |
|---|---|
| Rust → JSON (serialize) | `snake_case` field names → `camelCase` |
| JSON → Rust (deserialize) | `camelCase` keys → `snake_case` |
| `Option::None` field | omitted entirely (not `null`) |
| Enum variant | `{"t":"VariantName", ...fields...}` |

## Cookie Utilities

```rust
use forte_sdk::cookie_sign::{sign_cookie, unsign_cookie};
use forte_sdk::time;

// Write HMAC-signed cookie (requires COOKIE_SECRET env var)
sign_cookie(req.jar, "session", &my_value, Some(time::Duration::days(30)));

// Read and verify
let value: Option<MyType> = unsign_cookie(req.jar, "session");
```

## Environment Variables

| Variable | Used by | Notes |
|---|---|---|
| `COOKIE_SECRET` | `cookie_sign` | HMAC secret for signed cookies |
| `TURSO_URL` | `doc-db` | Injected by `forte dev`; set in `env.yaml` for production |
| `TURSO_AUTH_TOKEN` | `doc-db` | Injected by `forte dev`; set in `env.yaml` for production |
| `FN0_QUEUE_URL` | queue tasks | Injected by `forte dev`; required in production if using queue tasks |
| `FN0_OBJECT_STORAGE_URL` | object-storage | Injected by `forte dev`; required in production |
| `OTEL_SERVICE_NAME` | tracing/metrics | Service name in OTLP exports (default: `"forte-app"`) |

Plain env vars: `forte env set KEY value`  
Secrets: `forte env set KEY value --secret`  
List/unset: `fn0 env list` / `fn0 env unset KEY`

## Running Tests

```sh
# Inside rs/ of a Forte project (requires forte-test-runner)
cargo test

# From the monorepo
cargo test -p forte-sdk
cargo test -p fn0-doc-db   # requires docker-compose up -d + forte-test-runner
```

Async test macro:

```rust
#[forte_sdk::test]
async fn my_test() {
    let db = doc_db::memory();     // in-memory db, isolated per test
    let bucket = object_storage::memory();  // in-memory storage
    // ...
}
```

Configure the WASM test runner in `rs/.cargo/config.toml`:

```toml
[target.wasm32-wasip2]
runner = "forte-test-runner"
```

Install: `cargo install --path <monorepo>/forte/test-runner`

See [testing.md](testing.md) for full details.

## Auto-generated Files (do not edit)

| File | Generated by | When |
|---|---|---|
| `rs/src/route_generated.rs` | `forte-codegen` (build.rs) | every `cargo build` |
| `rs/src/actions/mod.rs`, `admin/mod.rs`, `queue_task/mod.rs` | same | same |
| `rs/src/env_generated.rs` | `generate_env()` in build.rs | when `.env` changes |
| `fe/src/pages/<path>/.props.ts` | `forte-rs-to-ts` | `forte build`/`forte dev` |
| `fe/src/actions/.generated/<name>.ts` | same | same |
| `fe/src/hooks/.generated/<Name>.ts` | same | same |
| `fe/src/paths.generated.ts` | `forte-codegen` | same as `route_generated.rs` |
| `fe/.forte/` (entire dir) | `forte dev`/`forte build` | each run |
