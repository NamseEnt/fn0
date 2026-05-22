# Setup

## Prerequisites

- **Rust ≥ 1.84** (2024 edition) with the `wasm32-wasip2` target. Stable 1.84+ ships a `wasm-component-ld` that links `wit-bindgen 0.50` output; older versions do not.
- **Node.js** and **npm** (for frontend builds)
- **Docker** (optional, for local Turso/libSQL database)

### Install Rust target

```sh
rustup target add wasm32-wasip2
```

### Install forte CLI

Build from source (the CLI is in the `forte/cli` crate):

```sh
cargo install --path forte/cli
```

Or download a pre-built binary from the GitHub releases if available.

### Install fn0 CLI (optional, for running fn0 locally)

```sh
cargo install --path fn0/cli
```

## Local Database (Turso/libSQL)

The `doc-db` crate connects to Turso/libSQL. A local development server is provided via Docker Compose:

```sh
docker-compose up -d   # starts libsql on port 8080
```

Environment variables used by `doc-db`:

| Variable | Default | Description |
|---|---|---|
| `TURSO_URL` | `http://127.0.0.1:8080` | Database URL |
| `TURSO_AUTH_TOKEN` | *(empty)* | Auth token (empty for local) |

For production, set these to your Turso cloud credentials.

## Environment Variables

| Variable | Required | Description |
|---|---|---|
| `COOKIE_SECRET` | Yes (if using `cookie_sign`) | HMAC secret for signed cookies |
| `TURSO_URL` | No | Database URL (defaults to localhost) |
| `TURSO_AUTH_TOKEN` | No | Database auth token |
| `FN0_QUEUE_URL` | Only for queue tasks | Endpoint for enqueuing tasks |

## Creating a Forte Project

```sh
forte init my-app
cd my-app
forte dev
```

`forte init` scaffolds the project, installs npm packages, and prints next steps.

To deploy, authenticate first:

```sh
forte login          # opens browser, saves token to ~/.config/fn0/credentials
forte deploy
```

See [forte/overview.md](forte/overview.md) for project layout and [forte/cli.md](forte/cli.md) for all CLI commands.
