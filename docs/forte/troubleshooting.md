# Troubleshooting

Common issues when developing with Forte.

## Build / Codegen

### `route_generated.rs` is wrong after adding a handler

Codegen runs during `cargo build` (via `build.rs`). If the generated file looks stale:

```sh
# From rs/ inside a Forte project
cargo build
```

Or regenerate everything including TypeScript types:

```sh
forte build
```

### Handler not being discovered

Codegen uses static analysis (not Rust compilation) to discover handlers. Make sure the file matches the exact naming conventions:

| Type | Required names | Location |
|---|---|---|
| Page | `pub async fn handler` returning `Result<Props>` | `src/pages/` (recursive) |
| API | `pub async fn handler` returning `Result<Props>` | `src/apis/` (recursive) |
| Action | `struct Input`, `Output` type, `pub async fn handler` | `src/actions/` (flat or `<name>/mod.rs`) |
| Hook | `struct Input`, `Output` type, `pub async fn handler` | `src/hooks/` (flat `.rs` only) |
| Queue task | `struct Input` or `type Input`, `pub async fn handle` | `src/queue_task/` (flat `.rs` only) |
| Admin task | `struct Input` or `type Input`, `pub async fn handle` | `src/admin/` (flat `.rs` only) |
| Static file | any file | `public/` (recursive) |

For pages/APIs the return type string must contain both `"Result"` and `"Props"`. Naming the type `Props` is the simplest way to satisfy this.

For actions and hooks, `Input` must be a **struct** (not a type alias). For queue/admin tasks `pub type Input = ()` is also accepted.

Nested paths under `src/actions/` (e.g. `src/actions/user/login.rs`) are not discovered — only flat files and `<name>/mod.rs`.

### `lib.rs` is missing the forte-managed marker block

The build panics if `src/lib.rs` does not contain the managed block markers. Add them:

```rust
// === FORTE-MANAGED START ===
// Auto-managed by `forte build`. Do not edit between the START/END markers.
mod route_generated;
// === FORTE-MANAGED END ===
```

Then re-run `forte build`.

---

## `forte dev`

### `forte dev` fails to start sqld

**Symptom:** `sqld did not become ready within 5 seconds`

- Another process may be using the sqld port. Try stopping any other local DB services.
- The sqld binary may be corrupted. Clear the cache and retry:
  ```sh
  rm -rf ~/.forte/bin/sqld-*
  forte dev
  ```

### Environment variable changes don't take effect

`forte dev` hot-reloads `env.yaml` and `env.local.yaml` on file change — updated values for **existing** variables take effect on the next request without a Rust rebuild.

Adding a **new** variable still requires a `cargo build`, because `generate_env()` emits its accessor function at build time.

`env.local.yaml` is local-only: it is gitignored and never bundled into a deploy. `env.yaml` is the one that ships.

---

## `forte-rs-to-ts`

### TypeScript types are not generated

`forte-rs-to-ts` runs during `forte build` and `forte dev`. If the generated `.props.ts` or `.generated/*.ts` files are missing:

1. Check that `forte build` or `forte dev` completed without errors.
2. Verify the Rust handler compiles successfully (`cargo build` in `rs/`).
3. Check that the platform is supported: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`. Windows is not supported.

To force a fresh download of the tool:

```sh
rm -rf ~/.forte/bin/forte-rs-to-ts-*
forte build
```

---

## Testing

### `cargo test` builds but doesn't run (WASM crates)

A Forte project's `rs/` crate targets `wasm32-wasip2`. Without the `forte-test-runner` configured, `cargo test` compiles the test binary but cannot execute it.

Add the runner to `rs/.cargo/config.toml`:

```toml
[build]
target = "wasm32-wasip2"

[target.wasm32-wasip2]
runner = "forte-test-runner"
```

Install the runner from the monorepo:

```sh
cargo install --path <monorepo>/forte/test-runner
```

`forte init` does not add the runner configuration automatically.

### `doc-db` or `object-storage` tests fail

These crates compile to `wasm32-wasip2` for tests and require:

1. `forte-test-runner` in PATH.
2. A running libSQL server for `doc-db` tests:
   ```sh
   docker compose up -d libsql-test
   ```

See [development.md](../development.md) for full prerequisites.

---

## `forte dev` local data

### Reset local database

The local libSQL database is stored in `.forte/data/` inside your project directory. To wipe it and start fresh:

```sh
rm -rf .forte/data/
forte dev
```

`forte dev` recreates the directory on next start.

### Reset local object storage

Local object storage files are stored in `.forte/data/objects/`. To clear all objects without dropping the database:

```sh
rm -rf .forte/data/objects/
```

---

## Deployment

### `forte deploy` fails on first run

`forte deploy` needs you to be authenticated:

```sh
forte login      # opens browser for PKCE flow
forte deploy
```

### Environment variables not available in production

Secrets and env vars set via `forte env set` are stored in `env.yaml` and bundled during `forte deploy`. Plain entries are also read by `forte dev`; encrypted ones are not, since decrypting needs the vault — override those in `env.local.yaml` for local runs.

### Cron jobs not firing

- Verify `cron.yaml` exists in the project root.
- Each `function` must match a `rs/src/queue_task/<name>.rs` file.
- That task's `Input` must be empty: `pub struct Input;`, `pub struct Input {}`, or `pub type Input = ()`.
- Cron jobs are registered during `forte deploy`. Run `forte deploy` again after adding or changing `cron.yaml`.
