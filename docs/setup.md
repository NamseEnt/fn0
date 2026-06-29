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

To deploy, authenticate first:

```sh
forte login          # PKCE flow: opens browser, exchanges code for token, saves credentials
forte deploy
```

See [forte/overview.md](forte/overview.md) for project layout and [forte/cli.md](forte/cli.md) for all CLI commands.
