# Forte CLI Reference

The `forte` CLI is the primary developer tool for creating, running, building, and deploying Forte projects.

## Commands

### `forte init <name>`

Scaffold a new project in a new directory named `<name>`.

Creates:
- `Forte.toml`, `Cargo.toml` (workspace), `.gitignore`
- `rs/` — Rust backend with an index page handler
- `fe/` — React/TypeScript frontend with Vite
- Runs `npm install` for frontend dependencies

```sh
forte init my-app
cd my-app
```

---

### `forte dev [options]`

Start the development server with hot reload.

| Flag | Default | Description |
|---|---|---|
| `-P, --port <port>` | auto (from 3000) | Port to listen on |
| `-p, --project <dir>` | `.` | Project directory |

Behavior:
- Starts a Vite dev server for the frontend (HMR)
- Serves the Rust backend by rebuilding on change
- Handles SSR requests

```sh
forte dev
forte dev --port 8080
```

---

### `forte build [options]`

Build the project for production without deploying.

| Flag | Default | Description |
|---|---|---|
| `-p, --project <dir>` | `.` | Project directory |

Build steps:
1. **Codegen** — runs `forte-rs-to-ts` to generate `.props.ts` files, then generates frontend route file (`routes.generated.ts`)
2. **Backend** — `cargo build --release --target wasm32-wasip2` inside `rs/`
3. **Frontend** — `npx vite build --config <config>` (client) and `npx vite build --ssr <entry> --config <config>` (SSR)
4. **Dist** — copies `backend.wasm` and `server.js` to `dist/`

Output files:
- `dist/backend.wasm`
- `dist/server.js`

```sh
forte build
```

---

### `forte deploy [options]`

Build and upload the project to fn0 Cloud.

| Flag | Default | Description |
|---|---|---|
| `-p, --project <dir>` | `.` | Project directory |
| `--name <name>` | — | Display name for first-time registration |

If a `cron.yaml` file exists in the project root, its scheduled jobs are registered during deploy. See [Cron Jobs](#cron-jobs) below.

```sh
forte deploy
forte deploy --name "My App"
```

---

### `forte add page <path>`

Add a new page (Rust handler + React component).

The `path` argument supports dynamic segments using `[param]` syntax:

```sh
forte add page about
forte add page product/[id]
forte add page blog/[year]/[slug]
```

Creates:
- `rs/src/pages/<path>/mod.rs` — Rust handler
- `fe/src/pages/<path>/page.tsx` — React component

---

### `forte add action <path>`

Add a new server action (Rust handler + TypeScript client).

```sh
forte add action user_login
forte add action products_list
```

Creates:
- `rs/src/actions/<path>.rs` — Rust action handler
- `fe/src/actions/<path>.ts` — TypeScript fetch wrapper

> **Important:** Use underscores, not slashes, in action paths. `forte add action user/login` creates `rs/src/actions/user/login.rs` (a subdirectory), but codegen only scans the top-level `src/actions/` directory. That file will never be discovered. Use `forte add action user_login` instead.
>
> The generated code also has naming and return-type bugs that must be fixed before `forte build` will succeed. See the [actions guide](actions.md) for the correct pattern.

---

### `forte login`

Authenticate with fn0 Cloud. Opens a browser to the fn0 tokens page, prompts you to paste an API token, and saves credentials locally (shared with the `fn0` CLI).

| Flag | Default | Description |
|---|---|---|
| `--token <token>` | — | Provide token directly (skips interactive flow) |

Tokens must start with `fn0_`. Credentials are saved to a local file (path printed on success).

```sh
forte login
forte login --token fn0_xxxxx
```

---

### `forte domain <subcommand>`

Manage custom domains for the deployed project.

| Subcommand | Description |
|---|---|
| `add <domain>` | Attach a custom domain (CNAME-based) |
| `remove` | Detach the custom domain |
| `status` | Show custom domain status |

```sh
forte domain add www.example.com
forte domain status
forte domain remove
```

---

### `forte admin run <task> [options]`

Run an admin task against the deployed app.

| Flag | Default | Description |
|---|---|---|
| `task` | — | Task name (matches `rs/src/admin/<name>.rs`) |
| `-p, --project <dir>` | `.` | Project directory |
| `--input-file <file>` | — | Read input JSON from file |
| `--input <json>` | — | Input JSON as string |
| `--timeout-seconds <n>` | 300 | Timeout |

```sh
forte admin run seed-database --input '{"count": 100}'
```

### `forte admin run-local <task> [options]`

Same as `run` but targets a locally-running `forte dev` server.

| Flag | Default | Description |
|---|---|---|
| `-P, --port <port>` | 3000 | Local dev server port |

---

## Cron Jobs

Place a `cron.yaml` file in the project root to schedule queue tasks. The file is read during `forte deploy` and the jobs are registered with fn0 Cloud.

```yaml
# cron.yaml
- function: send_digest_email
  every_minutes: 60
- function: cleanup_old_sessions
  every_minutes: 1440
```

Each entry:
- `function` — must match a file in `rs/src/queue_task/<name>.rs`, and that task's `Input` must be a unit struct (no fields).
- `every_minutes` — run interval (must be ≥ 1).

Cron jobs are not supported in local development (`forte dev`).
