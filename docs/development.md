# Development Workflow

## Code Style

- **No explanatory comments in code.** TODO comments are allowed.
- **Rust edition:** 2024
- Run `cargo clippy` and fix all warnings before committing.
- Run `cargo fmt` before committing.
- Tests must pass.

These rules apply across all crates in the workspace (see individual `CLAUDE.md` files in each crate).

## Dev Cycle (Forte Project)

1. `forte dev` — starts the dev server (Vite HMR + Rust rebuild on change)
2. Make changes to `rs/src/` or `fe/src/`
3. The dev server auto-rebuilds and reloads
4. Write tests and run them
5. `cargo clippy && cargo fmt`
6. `forte build` — verify the production build works

## Running Tests

### Rust tests

```sh
# From workspace root
cargo test

# Single crate
cargo test -p forte-sdk
cargo test -p doc-db
```

### Async tests in forte-sdk / backend crates

Use `#[forte_sdk::test]` (not `#[tokio::test]`) for async tests in WASM-compiled crates:

```rust
#[forte_sdk::test]
async fn my_async_test() {
    // ...
}
```

This macro uses `forte_sdk::runtime::block_on` which is compatible with the WASI async runtime.

### doc-db integration tests

Require a running libSQL server:

```sh
docker-compose up -d
cargo test -p doc-db
```

## Build Targets

| Binary | Cargo target | Notes |
|---|---|---|
| Forte backend | `wasm32-wasip2` | Set via `rs/.cargo/config.toml` |
| fn0-worker | Native (aarch64/x86_64) | Via `scripts/build-fn0-worker.sh` |
| fn0-worker-agent | Native | Via `scripts/build-fn0-worker-agent.sh` |
| cwasm-compiler | Node.js | Via `scripts/build-cwasm-compiler.sh` |

## Workspace Layout

The Cargo workspace root (`/Cargo.toml`) includes:

```toml
[workspace]
members = [
    "fn0/*",
    "forte/*",
    "doc-db",
]
# Excluded: vendor/*, forte/rs-to-ts, fn0/control
```

`forte/rs-to-ts` and `fn0/control` are excluded from the workspace and must be built separately.

## Code Generation (forte-codegen)

Code generation runs as part of `cargo build` via `build.rs`. If `route_generated.rs` looks wrong after adding/removing pages or actions, run:

```sh
cargo build  # inside rs/ or forte project root
```

Or `forte build` to regenerate everything including TypeScript types.

Never edit `route_generated.rs`, `actions/mod.rs`, `admin/mod.rs`, `queue_task/mod.rs`, or the `FORTE-MANAGED` block in `lib.rs` manually.

## Release Process

Releases are managed via `cargo-dist` and GitHub Actions:

- **`publish.yml`** — triggers on push to `main`, runs `cargo-release`, creates version tags for `forte-cli` and `forte-rs-to-ts`
- **`release.yml`** — triggers on version tags, builds release artifacts and creates GitHub releases
- **`release-forte-rs-to-ts.yml`** — specialized release pipeline for `forte-rs-to-ts`

Version tags follow SemVer: `forte-cli-v0.3.30`, `forte-rs-to-ts-v0.1.7`.

## Infrastructure

Infrastructure is managed in `infra/`:

- `infra/pulumi/` — Pulumi IaC for cloud resources
- `infra/cloud/` — Cloud provider configurations (AWS, OCI)
- `infra/r2-worker/` — Cloudflare R2 worker for blob storage

Scaling configuration: `scripts/scale-config.sh`

## Local Database

For local development with `doc-db`, start the libSQL server:

```sh
docker-compose up -d   # libsql on port 8080
```

Environment variables (set in shell or `.env`):
```
TURSO_URL=http://127.0.0.1:8080
TURSO_AUTH_TOKEN=
```
