# Forte CLI Reference

The `forte` CLI is the primary developer tool for creating, running, building, and deploying Forte projects.

## Commands

### `forte init <name>`

Scaffold a new project in a new directory named `<name>`.

Creates:
- `Forte.toml`, `.gitignore`
- `rs/` — Rust backend with an index page handler (`rs/Cargo.toml` is a standalone crate, not a workspace)
- `fe/` — React/TypeScript frontend with Vite
- Runs `npm install` for frontend dependencies

| Flag | Description |
|---|---|
| `--dev` | Use `path = "..."` deps pointing at the local fn0 monorepo instead of crates.io versions. For in-monorepo development only. |

```sh
forte init my-app
cd my-app
forte init --dev my-app   # monorepo development
```

---

### `forte dev [options]`

Start the development server with hot reload.

| Flag | Default | Description |
|---|---|---|
| `-P, --port <port>` | auto (from 3000) | Port to listen on |
| `-p, --project <dir>` | `.` | Project directory |

Behavior:
- Downloads and starts a local sqld (libSQL) server automatically; data persists in `.forte/data/`
- Starts a Vite dev server for the frontend (HMR)
- Rebuilds the Rust backend on `.rs` file changes
- Hot-reloads `env.yaml` / `env.local.yaml` on change — updated variable values take effect on the next request without a Rust rebuild (adding a new variable still requires `cargo build`, since `generate_env()` emits its accessor at build time)
- Handles SSR requests
- Routes queue task messages locally (loopback; no external queue needed)
- Serves object storage from `.forte/data/objects/` (no cloud credentials needed)
- Fires cron jobs from `cron.yaml` at the appropriate minute boundaries

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

Deploy steps (in addition to `forte build`):
1. **Registers project** — on first deploy, prompts for a display name and saves the assigned `project_id` to `Forte.toml`.
2. **Uploads static assets** — `fe/dist/` (client JS/CSS/assets) is uploaded to the project's own frontend-asset bucket, served from `https://fn0-<project_id>-frontend-asset.<your-domain>/<code_version>/`. The `VITE_PUBLIC_URL` env var is set to this URL during the Vite client build so asset references resolve correctly.
3. **Uploads backend bundle** — packages `dist/backend.wasm`, `dist/server.js`, and `env.yaml` into `dist/bundle.raw.tar` and uploads it to the fn0 Cloud control plane.
4. **Compiles to native** — the control plane invokes the `fn0-cwasm-compiler` Lambda to ahead-of-time compile `backend.wasm` to a Wasmtime-native `.cwasm` bundle. The CLI polls `deploy_status` until compilation finishes (one compilation per active Wasmtime version). Workers load the pre-compiled bundle on the next request, so there is no JIT cost at runtime.
5. **Registers cron jobs** — if `cron.yaml` exists, schedules are synced.

```sh
forte deploy
forte deploy --name "My App"
```

The app URL is printed when the deploy finishes.

---

### `forte destroy`

Delete the deployed project and all of its resources: routing, custom domain, deployed bundles, static assets, object storage, and its database.

| Flag | Default | Description |
|---|---|---|
| `--yes` | off | Skip the interactive confirmation |

Runs against the project in the current directory only — the `project_id` is read from `Forte.toml`, and there is no flag to target another directory or another project id. Without `--yes`, you must type the project id to confirm.

On success the `project_id` key is removed from `Forte.toml` (all other keys and formatting are preserved), so the next `forte deploy` registers a new project. Teardown is enqueued on the control plane and runs asynchronously.

```sh
forte destroy
forte destroy --yes
```

---

### `forte open [options]`

Print the deployed app's URL and open it in the default browser.

| Flag | Default | Description |
|---|---|---|
| `-p, --project <dir>` | `.` | Project directory |
| `--print` | off | Print the URL only, do not open a browser |

The project is always reachable at `https://<project_id>.fn0.dev`. If a custom domain is attached and active, that domain is used instead. When the custom domain is not serving yet, or the lookup cannot be made, the default subdomain is used and the reason is printed to stderr.

```sh
forte open
forte open --print
```

Requires `project_id` in `Forte.toml`, which `forte deploy` writes on first deploy.

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

Add a new server action (Rust handler only).

```sh
forte add action user_login
forte add action products_list
```

Creates:
- `rs/src/actions/<path>.rs` — Rust action handler with correct `Input` / `Output` / `handler` names

The TypeScript client is generated automatically by `forte-rs-to-ts` on the next `forte build` or `forte dev`. Import from `fe/src/actions/.generated/<name>.ts`.

> **Important:** Use underscores, not slashes, in action paths. `forte add action user/login` creates `rs/src/actions/user/login.rs`, but codegen only discovers handlers at `actions/<name>.rs` (flat file) or `actions/<name>/mod.rs` (directory module) — nested files like `actions/user/login.rs` are never discovered. Use `forte add action user_login` instead (or create `actions/user_login/mod.rs` manually if you need to split the file across multiple modules).

Dashes in the path are automatically converted to underscores: `forte add action user-login` creates `rs/src/actions/user_login.rs`.

---

### Adding hooks, queue tasks, and admin tasks

There are no `forte add hook`, `forte add queue-task`, or `forte add admin` commands. Create these files manually:

- Hooks: `rs/src/hooks/<name>.rs` — follow the `Input` / `Output` / `pub async fn handler` pattern (see [actions.md](actions.md#hooks))
- Queue tasks: `rs/src/queue_task/<name>.rs` — follow the `Input` / `pub async fn handle` pattern
- Admin tasks: `rs/src/admin/<name>.rs` — follow the `Input` / `pub async fn handle` pattern

Codegen picks them up automatically on the next build; no `mod` declaration needed in `lib.rs`.

---

### `forte login`

Authenticate with fn0 Cloud using a PKCE OAuth flow and saves credentials locally (shared with the `fn0` CLI).

| Flag | Default | Description |
|---|---|---|
| `--token <token>` | — | Provide token directly (skips interactive flow) |

Default interactive flow:
1. A loopback TCP listener is started on a random port.
2. A PKCE authorization URL is printed and the browser is opened automatically (falls back to manual URL if auto-open fails).
3. After you approve in the browser, the callback redirects to `http://127.0.0.1:<port>/callback`.
4. The CLI exchanges the authorization code for a token and saves it locally.

With `--token`, the interactive flow is skipped — the token is validated (must start with `fn0_`) and saved directly. Credentials are saved to a local file (path printed on success).

```sh
forte login
forte login --token fn0_xxxxx
```

---

### `forte cloudflare <subcommand>`

Run the project's object storage, CDN and custom domain on your own Cloudflare account. See [Bring Your Own Cloudflare](../fn0/cloudflare.md) for the token permissions this needs.

| Subcommand | Description |
|---|---|
| `connect` | Hand fn0 the two narrow credentials it keeps. With `--api-token` the CLI provisions and mints them for you; otherwise pass credentials you made yourself |
| `provision` | Create the buckets, CDN hostname and cache rule and stop, for people who would rather not hand any tool a token that can create tokens |
| `status` | Show which account the project uses and whether its credentials still work |

Whichever path you take, no token you create is ever sent to fn0.

```sh
# convenient: one token with User -> API Tokens -> Edit
forte cloudflare connect --account-id <a> --zone-id <z> --api-token <token>

# careful: a token that can provision but cannot create tokens
forte cloudflare provision --account-id <a> --zone-id <z> --api-token <token>
forte cloudflare connect --account-id <a> --zone-id <z> --zone-name <domain> \
  --worker-access-key-id <k> --worker-secret <s> \
  --frontend-asset-access-key-id <k> --frontend-asset-secret <s> \
  --purge-token <p>

forte cloudflare status
```

---

### `forte domain <subcommand>`

Manage custom domains for the deployed project.

| Subcommand | Description |
|---|---|
| `add <domain>` | Attach a custom domain. On your own Cloudflare account, pass `--account-id --zone-id --api-token` so the CLI can sign the origin certificate locally |
| `remove` | Detach the custom domain |
| `status` | Show custom domain status |

All subcommands accept `-p, --project <dir>` (default: `.`) to specify the project directory.

```sh
forte domain add www.example.com
forte domain status
forte domain remove
```

For a project on your own Cloudflare account, `add` signs an origin certificate through your Origin CA **on your machine** — fn0 holds no token that can sign one — uploads it, and prints the IP to point a **proxied** `A` record at. For a project still on the fn0 platform account, it registers a Cloudflare for SaaS hostname and you point a `CNAME` at fn0 instead.

---

### `forte env <subcommand>`

Manage `env.yaml` entries for the project. Entries are bundled into the deploy and exposed as environment variables at runtime. Secret entries are encrypted via fn0 Cloud's vault and decrypted in-worker.

| Subcommand | Description |
|---|---|
| `set <key> <value> [--secret]` | Set an env entry. Plain by default; `--secret` encrypts the value via the control vault (requires `forte login`). Silently overwrites an existing key. |
| `migrate` | Convert a legacy `.env` file into `env.local.yaml`. Refuses to run if `env.local.yaml` already exists, and leaves `.env` on disk for you to delete. |

| Flag | Default | Description |
|---|---|---|
| `--secret` | `false` | Encrypt the value as a secret entry |
| `-p, --project <dir>` | `.` | Project directory |

```sh
forte env set PUBLIC_API_URL https://api.example.com
forte env set DATABASE_PASSWORD hunter2 --secret
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
| `task` | — | Task name (matches `rs/src/admin/<name>.rs`) |
| `-P, --port <port>` | 3000 | Local dev server port |
| `--input-file <file>` | — | Read input JSON from file |
| `--input <json>` | — | Input JSON as string |
| `--timeout-seconds <n>` | 300 | Timeout |

---

## Secrets

Use `forte env set <key> <value> --secret` to encrypt a value and write it to `env.yaml`. To list or remove entries, use the `fn0` CLI from the project directory:

```sh
fn0 env list
fn0 env unset COOKIE_SECRET
```

Secrets are encrypted via fn0 Cloud's vault and decrypted in-worker at runtime. Decryption needs the vault, so `forte dev` cannot open them; give any secret you need locally a plaintext value in `env.local.yaml`.


### env.yaml format

`forte env set` manages `env.yaml` for you. The file is a YAML mapping of key → value, where plain values are strings and secret values are a nested mapping with a `secret` key:

```yaml
# env.yaml — managed by `forte env set`
PUBLIC_API_URL: https://api.example.com
STRIPE_SECRET_KEY:
  secret: <ciphertext written by forte env set --secret>
```

Do not edit the `__dek` key (auto-managed data-encryption-key entry). Plain values are available as environment variables in the deployed app; secret values are decrypted at runtime by the worker.


See [forte env](#forte-env-subcommand) above and [fn0 CLI Reference](../fn0/overview.md#fn0-cli-commands) for full `fn0 env` documentation.

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
- `function` — must match a file in `rs/src/queue_task/<name>.rs`, and that task's `Input` must be empty — either a unit struct (`pub struct Input;` or `pub struct Input {}`) or the type alias `pub type Input = ();`.
- `every_minutes` — run interval (must be ≥ 1).

Cron jobs run locally during `forte dev`: the CLI ticks at each minute boundary, reads `cron.yaml`, and enqueues matching tasks through the loopback queue.

---

## Local Tool Cache

`forte dev` and `forte build` download external tools on first use and cache them in `~/.forte/bin/`. You do not need to manage these manually; downloads happen automatically.

| Tool | Cache path | Source |
|---|---|---|
| `sqld` (libSQL server) | `~/.forte/bin/sqld-<version>` | tursodatabase/libsql GitHub Releases |
| `forte-rs-to-ts` | `~/.forte/bin/forte-rs-to-ts-<version>/forte-rs-to-ts` | NamseEnt/fn0 GitHub Releases |

To force a fresh download (e.g., after a corrupted download), delete the relevant entry:

```sh
rm -rf ~/.forte/bin/sqld-*
rm -rf ~/.forte/bin/forte-rs-to-ts-*
```

The next `forte dev` or `forte build` will re-download the tools.
