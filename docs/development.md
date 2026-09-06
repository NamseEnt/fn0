# Development Workflow

## Code Style

- **Rust edition:** 2024
- **Rust toolchain:** pinned in `rust-toolchain.toml` (currently 1.97.1). `rustup` picks this up automatically — do not override it.
- **Rust file layout:** `mod` declarations go at the very top of the file, then `use` statements below them. No blank lines between consecutive `mod` declarations, nor between consecutive `use` statements.
- Run `cargo clippy` and fix all warnings before committing.
- Run `cargo fmt` before committing, and CI checks it. Run it per crate (`cargo fmt -p forte-sdk`) or plain from the workspace root, which covers every workspace member. Never run `cargo fmt --all`: it follows path dependencies into `vendor/` and reformats the vendored deno crates. `vendor/` is excluded from formatting and keeps upstream deno's style.
- In `fn0/control/rs`, `cargo fmt` needs a build to have run first: `build.rs` generates `src/route_generated.rs`, which rustfmt must resolve as a module. `forte/rs-to-ts` is its own workspace root and formats with its pinned nightly toolchain.
- Tests must pass.

These rules apply across all crates in the workspace (see individual `CLAUDE.md` files in each crate).

### Comments

Write only what code and history tools cannot carry. Run every comment through these three filters — drop it if it fails any:

1. Can it be expressed in code (renaming, types, structure)? → fix the code, not add a comment.
2. Does git history (commit/blame) or the issue tracker already record it? → leave it there. No `// fixed #1425`, no change logs, no commented-out code.
3. Would a competent teammate actually break something without this *why*? → if no, skip it even if the why is real.

Worth keeping: non-obvious rationale, unidiomatic code that is intentionally correct, workarounds / perf trade-offs / system limits, links to external specs or standards, `// TODO`, math derivations.

Never write: comments that restate the code (`i += 1; // add one`), syntax explanation, history/bugfix notes, stale comments, commented-out code.

`//!` module-level doc comments are a separate category: they are `cargo doc` API documentation, not inline commentary, and the above filters do not gate them. Add a `//!` at the top of `lib.rs`/`main.rs` (crate purpose, setup, gotchas) and at the top of a non-obvious module file. First line is a one-line summary.

### Naming

A name states what the thing holds or does, and — when code branches on it — which kind it is.

- **Avoid pure-category names.** `page`, `data`, `info`, `config`, `manager`, `handler` say what category the thing belongs to, not what it actually is. `private-object-storage` says what it holds and who may access it; `page` says neither. Where a distinction drives behavior, put the distinction in the name: a bucket that GC may empty and a bucket that holds irreplaceable user data must not read alike.
- **No abbreviations or acronyms** outside a single function body. Spell names out. `ssr`, `cfg`, `req_id` are not acceptable in type, field, function, or module names.
- If a comment is needed to explain what a name means, the name is wrong — rename and delete the comment.
- **Names that escape the codebase** — R2 bucket names, hostnames, env var keys, doc/table field names, published crate items — cannot be renamed cheaply once they exist. Settle them before the first write, and state the naming convention in the module that mints them.

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
cargo test -p fn0-doc-db
```

### Distributed WebSocket delivery

```sh
cargo test -p fn0-worker distributed_send_reaches_connection_on_second_worker
cargo test -p fn0-worker-agent worker_container_quic_env_uses_worker_port_and_private_ip
```

### Async tests in forte-sdk / backend crates

Use `#[forte_sdk::test]` (not `#[tokio::test]`) for async tests in WASM-compiled crates:

```rust
#[forte_sdk::test]
async fn my_async_test() {
    // ...
}
```

These tests do not run under libtest — `forte-test-runner` discovers them through the `fn0:test-harness/harness` export and runs each in its own instance. The target must set `harness = false` and call `forte_sdk::test_main!()`; see [forte/testing.md](forte/testing.md) for the wiring and the reason.

### Crates whose tests compile to wasm32-wasip2

`doc-db`, `object-storage` and `fn0-control` build their tests for `wasm32-wasip2` (configured in their `.cargo/config.toml`), so `cargo test` runs them through `forte-test-runner`.

1. **`forte-test-runner` in PATH** — the configured WASM test runner. Install from the monorepo:

```sh
cargo install --path forte/test-runner
```

Reinstall it after pulling changes to the WASI wit or to `forte/test-runner` itself. An installed runner older than the checkout that built the component fails to link it, and reports that as an unimplemented WASI import rather than as a version skew.

2. **Running libSQL server** — for `doc-db` only: the `libsql-test` service, on `127.0.0.1:18123`. Override with `DOC_DB_TEST_URL`.

```sh
docker compose up -d libsql-test
```

Then run the tests:

```sh
cargo test -p fn0-doc-db
cargo test -p fn0-object-storage
cd fn0/control/rs && cargo test   # its own workspace, so -p from the root does not reach it
```

`forte-sdk` and other non-WASM crates do not require the test runner and run with the default host target.

## Build Targets

| Binary | Cargo target | Notes |
|---|---|---|
| Forte backend | `wasm32-wasip2` | Set via `rs/.cargo/config.toml` |
| fn0-worker | Native linux/arm64 | Built inside `deploy-fn0-worker.sh` via `build-rust-linux-arm64-bin.sh fn0-worker` |
| fn0-worker-agent | Native linux/arm64 | Via `scripts/build-fn0-worker-agent.sh` |
| fn0-worker-proxy | Native linux/arm64 | Via `scripts/build-fn0-worker-proxy.sh` |
| cwasm-compiler | Node.js | Via `scripts/build-cwasm-compiler.sh` |

All native Linux binaries are compiled using `scripts/build-rust-linux-arm64-bin.sh <package> <out_dir>`, which runs `cargo build --release` inside a `rust:bookworm` container. The repo is bind-mounted and `target/` + cargo-registry live on persistent named Docker volumes (`fn0-build-target`, `fn0-build-cargo-registry`) to keep incremental compilation fast across runs. Do not replace this with a `COPY`-into-`docker build` flow — that would rebuild the full dependency graph on every source change.

## Workspace Layout

The Cargo workspace root (`/Cargo.toml`) includes:

```toml
[workspace]
members = ["fn0/*", "forte/*", "doc-db", "object-storage"]
exclude = ["vendor/*", "forte/rs-to-ts", "fn0/control"]
```

`forte/rs-to-ts` and `fn0/control` are excluded from the workspace and must be built separately.

### Building excluded members

**`forte/rs-to-ts`** uses private rustc APIs and requires the pinned nightly compiler bundled inside the crate. Build and install it from inside its own directory:

```sh
cd forte/rs-to-ts
cargo build --release
# the binary is at target/release/forte-rs-to-ts
```

The CLI downloads the correct pre-built version automatically on first use (`~/.forte/bin/forte-rs-to-ts-<version>/forte-rs-to-ts`). You only need to build from source if you are developing `forte-rs-to-ts` itself.

**`fn0/control`** is a full Forte project (Rust backend + React frontend). Build it with `forte build` from inside `fn0/control/`, using a `forte` CLI built from the monorepo. Use `scripts/bootstrap-fn0-control.sh` for the initial platform deploy.

### vendor/deno

`vendor/deno/core/` is a patched copy of `deno_core` used by `fn0-ski`. The patch makes the module map serialization order deterministic, which is required for reproducible builds. The upstream crate iterates a `HashMap` in non-deterministic order.

- **Never** replace `vendor/deno/core/` with the upstream crate — that breaks `fn0-ski` build reproducibility.
- **Never** run `cargo fmt --all` — it reaches into `vendor/` and reformats the vendored code, breaking its diff from upstream. Use `cargo fmt` or `cargo fmt -p <crate>` instead.
- To update `deno_core` to a newer upstream version: copy the new upstream source into `vendor/deno/core/`, re-apply the determinism patch (see the git diff for `vendor/deno/core/` on the commit that introduced it), and update the `[patch.crates-io]` entry in the workspace `Cargo.toml`.

## Code Generation (forte-codegen)

Code generation runs as part of `cargo build` via `build.rs`. If `route_generated.rs` looks wrong after adding/removing pages or actions, run:

```sh
cargo build  # inside rs/ or forte project root
```

Or `forte build` to regenerate everything including TypeScript types.

Never edit `route_generated.rs`, `actions/mod.rs`, `admin/mod.rs`, `queue_task/mod.rs`, or the `FORTE-MANAGED` block in `lib.rs` manually.

## Release Process

Releases are managed via `cargo-dist` and GitHub Actions:

- **`publish.yml`** — triggers on push to `main`, runs `cargo-release`, creates version tags for `forte-cli`, `fn0-cli`, and `forte-rs-to-ts`
- **`release-release.yml`** — cargo-dist autogenerated; triggers on `release/`-namespaced version tags, builds release artifacts and creates GitHub releases for `forte-cli` and `fn0-cli`
- **`release-forte-rs-to-ts.yml`** — specialized release pipeline for `forte-rs-to-ts`

Version tags: `forte-cli` and `fn0-cli` use the cargo-dist `release/` tag namespace (`release/forte-cli-v0.3.37`, `release/fn0-cli-v0.1.5`); `forte-rs-to-ts` keeps its own `forte-rs-to-ts-v0.1.9` form. The namespace keeps `forte-rs-to-ts` tags out of cargo-dist's `release-release.yml` trigger.

## Infrastructure

Infrastructure is managed in `infra/`:

- `infra/pulumi/` — Pulumi IaC for cloud resources
- `infra/cloud/` — Cloud provider configurations (AWS, OCI)
- `infra/r2-worker/` — Cloudflare R2 worker for blob storage

Scaling configuration: `scripts/scale-config.sh`

### Static page caching prerequisites

[Lazy static page caching](forte/pages.md#lazy-static-page-caching) needs environment on both control and worker. Pulumi provisions both, but each has a silent-until-used failure mode worth knowing:

| Component | Variables | Missing behaviour |
|---|---|---|
| control | `FN0_CLOUDFLARE_ZONE_ID`, `FN0_CLOUDFLARE_API_TOKEN`, `FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID` | Deploy-time cache purge fails, the project stays in `pre_purge`, and `forte deploy` never reaches `Done` |
| worker | `FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID`, `_BUCKET`, `_ACCESS_KEY_ID`, `_SECRET_ACCESS_KEY` | Worker exits at startup |

`FN0_STATIC_ASSET_STORAGE_ENDPOINT` is optional and defaults to `https://<account_id>.r2.cloudflarestorage.com`.

Control reads its environment from `fn0/control/env.yaml`, which `forte deploy` ships. `infra/cloud/index.ts` builds the same set for the one-time bootstrap only, so a variable added there must be added to `env.yaml` as well or redeploys will not pick it up. Worker environment is written by cloud-init from `infra/pulumi/OciFn0WorkerSite.ts` and is only re-read when an instance is recreated — see the instance roll procedure before assuming a worker deploy applied it.

## Database Migrations

Run one-off SQL statements or migration files against the deployed database through the control plane — no credentials needed:

```sh
# Inspect data
forte db query 'SELECT pk, sk FROM docs LIMIT 20'

# Run a migration file as one atomic transaction
forte db exec migrations/2026-08-18-backfill.sql
```

`forte db exec` wraps the whole file in one transaction: either every statement commits or none does. A failing statement rolls everything back and reports which statement failed.

When writing raw SQL against `docs`, always increment `version` on every `UPDATE` of `data` (`SET data = ..., version = version + 1`); the `trx` optimistic-locking layer uses `version` for conflict detection.

See [`forte db` in the CLI reference](forte/cli.md#forte-db-query-sql-options) for all flags and bind-parameter syntax.

## Local Database

`forte dev` downloads and starts sqld automatically — no manual setup needed for Forte projects. For running `doc-db` tests directly, see [setup.md](setup.md#local-database-tursolibsql).

## Developing Framework Changes

When working on `forte-sdk`, `forte-codegen`, `forte-cli`, `doc-db`, or `object-storage`, use `forte init --dev` to create a test project that depends on the local monorepo instead of crates.io:

```sh
# From the monorepo root
forte init --dev my-test-app
cd my-test-app
forte dev
```

`--dev` writes `path = "..."` dependencies pointing at the local crates in the monorepo. Changes to `forte-sdk` or `forte-codegen` are picked up on the next `cargo build` inside the test project. Changes to `forte-cli` require rebuilding the CLI itself first:

```sh
# From the monorepo root
cargo build -p forte-cli
# Then use the built CLI binary (target/debug/forte) directly, or reinstall:
cargo install --path forte/cli
```

The `forte-rs-to-ts` binary is always downloaded from GitHub Releases — it cannot be pointed at the local source via `--dev` because it requires a nightly toolchain that `forte init --dev` does not manage. Build it from `forte/rs-to-ts` manually if you are developing that tool specifically.
