# Setup & Getting Started

## Prerequisites

| Tool | Purpose | Notes |
|------|---------|-------|
| Rust (stable) | Compile server code | `rustup update stable` |
| `wasm32-wasip2` target | Compile to WebAssembly | `rustup target add wasm32-wasip2` |
| `forte` CLI | Build and run Forte apps | Install from this repo or crates.io |
| `fn0` CLI | Local dev server | Install from this repo |
| Bun | TypeScript/JS bundling | Required for JS/TS deployments |

### Installing the CLIs

```sh
# From the repository root (requires a local build)
cargo install --path forte/cli
cargo install --path fn0/cli
```

### Cargo Target Configuration

Forte backend crates set the WASM target globally via `.cargo/config.toml`:

```toml
[build]
target = "wasm32-wasip2"

[target.wasm32-wasip2]
runner = "forte-test-runner"
```

This is already present in `doc-db/.cargo/config.toml` and is expected in Forte application crates. Do **not** override the target unless you have a specific reason.

## fn0.toml — Project Configuration

Running `fn0 init` creates `fn0.toml` in the project directory. The structure is:

```rust
struct Config {
    language_env: LanguageEnvironment,
}
enum LanguageEnvironment {
    TypescriptBunHono,
}
```

`init` asks:
1. **Language** — currently only `typescript`
2. **Package manager** — `bun`
3. **Framework** — `hono`

## CLI Commands

### `fn0 init`

Interactive wizard that scaffolds a project and writes `fn0.toml`.

### `fn0 build`

Reads `language_env` from `fn0.toml` and runs the appropriate build:

- `TypescriptBunHono` → `bun build`

Produces `dist/component.wasm`.

### `fn0 local`

Starts the fn0 server locally, loading `dist/component.wasm` by default.

```sh
fn0 local [--port|-p <port>] [--wasm-path <path>]
```

Both flags are optional. Default port is unspecified — check source at `fn0/cli/src/` for current default.

## Forte CLI Commands

The `forte` CLI is the primary build tool for Forte (Rust-backend) projects.

```sh
forte build   # run code generation + cargo build
forte dev     # watch mode (Unknown from repository — check forte/cli/src/)
```

### Build Cycle

The recommended development loop (from `forte/cli/CLAUDE.md`):

1. **Implement** your changes
2. **Test** — `cargo test` (tests run via `forte-test-runner` inside wasmtime)
3. **Lint** — `cargo clippy`
4. **Format** — `cargo fmt`

### Code Generation

`forte build` calls `forte_codegen::generate_routes()` from `build.rs`. It scans:

- `src/pages/` — page routes
- `src/apis/` — API routes
- `src/hooks/` — server hooks (self-invocable)
- `src/actions/` — server actions
- `src/queue_task/` — background queue tasks
- `src/admin/` — admin tasks

And writes generated code to `src/route_generated.rs` and a TypeScript path map to `../fe/src/paths.generated.ts`.

Your `src/lib.rs` **must** contain the forte-managed marker block:

```rust
// === FORTE-MANAGED START ===
// Auto-managed by `forte build`. Do not edit between the START/END markers.
mod route_generated;
// === FORTE-MANAGED END ===
```

The block is updated automatically by code generation.

## Environment Variables

| Variable | Used By | Purpose |
|----------|---------|---------|
| `TURSO_URL` | `doc-db` | Turso/libSQL HTTP endpoint (default: `http://127.0.0.1:8080`) |
| `TURSO_AUTH_TOKEN` | `doc-db` | Turso authentication token |
| `FN0_QUEUE_URL` | `enqueue` module | URL for queue task submission |

## Running Tests

Tests are compiled to WASM and executed by `forte-test-runner`:

```sh
cargo test
```

The `forte_sdk::test` proc-macro wraps `async fn` tests so they run via `forte_sdk::runtime::block_on`. Arguments to test functions are not supported.

```rust
#[forte_sdk::test]
async fn my_test() {
    // async test body
}
```
