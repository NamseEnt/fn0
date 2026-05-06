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

### `forte login [--token <token>]`

Log in to fn0 Cloud. Opens the token page in a browser, prompts to paste the token, and saves credentials shared with the `fn0` CLI.

```sh
forte login
forte login --token fn0_...   # non-interactive
```

Credentials are stored on disk (path printed on success). Required before `forte deploy`.

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
1. **Codegen** — runs `forte-rs-to-ts` to generate `.props.ts` files, then generates the frontend route table (`routes.generated.ts`)
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
| `--name <name>` | *(prompted)* | Display name — used only on the **first** deploy to register the project |

On the first deploy the command prompts for a display name (or use `--name`) and writes the assigned `project_id` back to `Forte.toml`. Subsequent deploys use the stored `project_id`.

If a `cron.yaml` file exists in the project root, its scheduled jobs are validated and uploaded with the deployment. See [Actions & Tasks — Cron Scheduling](actions.md#cron-scheduling).

If an `env.yaml` file exists in the project root, it is bundled with the deployment as encrypted environment variables.

Requires credentials saved by `forte login`.

```sh
forte deploy
forte deploy --name "My App"   # first deploy only
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
forte add action user/login
forte add action products/list
```

Creates:
- `rs/src/actions/<path>.rs` — Rust action handler
- `fe/src/actions/<path>.ts` — TypeScript fetch wrapper

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
