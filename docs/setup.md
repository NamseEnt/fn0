# Setup

## Prerequisites

- **Rust ≥ 1.84** (2024 edition) with the `wasm32-wasip2` target. Stable 1.84+ ships a `wasm-component-ld` that links `wit-bindgen 0.50` output; older versions do not.
- **Node.js ≥ 20** and **npm** (for frontend builds; Vite 8 requires Node.js 20+)
- **Docker** (optional, for local Turso/libSQL database)

### Install Rust target

```sh
rustup target add wasm32-wasip2
```

### Install forte CLI

Option A — `cargo binstall` (downloads a pre-built binary):

```sh
cargo binstall forte-cli
```

Option B — build from source (requires the monorepo):

```sh
cargo install --path forte/cli
```

Pre-built binaries for `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, and `aarch64-unknown-linux-gnu` are published to GitHub Releases on every version tag and picked up automatically by `cargo binstall`.

### Install fn0 CLI (optional, for raw fn0 projects without Forte)

```sh
cargo binstall fn0-cli
# or from source:
cargo install --path fn0/cli
```

## Local Database (Turso/libSQL)

The `doc-db` crate connects to Turso/libSQL.

**Forte projects:** `forte dev` downloads and starts sqld automatically — no manual setup needed. The database file is stored in `.forte/data/` inside your project directory. `TURSO_URL` and `TURSO_AUTH_TOKEN` are injected automatically and do not need to be set for local development.

**Running `doc-db` tests directly** (outside of `forte dev`) requires a separately running libSQL server:

```sh
docker-compose up -d   # starts libsql on port 8080
cargo test -p doc-db
```

Environment variables for direct `doc-db` usage:

| Variable | Default | Description |
|---|---|---|
| `TURSO_URL` | `http://127.0.0.1:8080` | Database URL |
| `TURSO_AUTH_TOKEN` | *(empty)* | Auth token (empty for local) |

For production, set these to your Turso cloud credentials.

## Environment Variables

| Variable | Required | Description |
|---|---|---|
| `COOKIE_SECRET` | Yes (if using `cookie_sign`) | HMAC secret for signed cookies |
| `TURSO_URL` | No | Database URL; injected automatically by `forte dev` |
| `TURSO_AUTH_TOKEN` | No | Database auth token; injected automatically by `forte dev` |
| `FN0_QUEUE_URL` | No | Queue endpoint; injected automatically by `forte dev`; required in production if using queue tasks |
| `FN0_OBJECT_STORAGE_URL` | No | Object storage endpoint; injected automatically by `forte dev`; required in production if using object storage |
| `OTEL_SERVICE_NAME` | No | Service name in OpenTelemetry traces and metrics (defaults to `"forte-app"`) |

## Creating a Forte Project

```sh
forte init my-app
cd my-app
forte dev
```

`forte init` scaffolds the project, installs npm packages, and prints next steps.

On the first `forte dev` run, two tools are downloaded automatically and cached in `~/.forte/bin/`:

- **sqld** (libSQL server) — for the local database
- **forte-rs-to-ts** — Rust→TypeScript type generator

Subsequent runs use the cached binaries. See [forte/cli.md#local-tool-cache](forte/cli.md#local-tool-cache) for cache paths and how to clear them.

## Deploying to fn0 Cloud

Deploying requires three steps the first time:

**1. Authenticate with fn0 Cloud:**

```sh
forte login
```

Opens a browser for PKCE OAuth, exchanges the code for a token, and saves credentials locally.

**2. Connect your Cloudflare account:**

```sh
CLOUDFLARE_API_TOKEN=<your-token> forte cloud init \
  --project . \
  --project-name my-app \
  --zone example.com
```

Registers the project with fn0 Cloud, provisions three R2 buckets in your Cloudflare account, and writes the proxied `CNAME` your app answers on into your zone. Requires `forte login` to have run first.

The token must have **User → API Tokens → Edit** permission. See [Bring Your Own Cloudflare](fn0/cloudflare.md) for token setup and what gets created in your account.

This step is idempotent — re-running it on an existing project reuses the existing resources.

**3. Deploy:**

```sh
forte deploy
```

Builds, uploads, and activates the project. A project that has not completed `forte cloud init` is refused at this step.

On subsequent deploys, only step 3 is needed.

See [forte/overview.md](forte/overview.md) for project layout and [forte/cli.md](forte/cli.md) for all CLI commands.
