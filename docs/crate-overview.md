# Crate Overview

All Rust crates in this monorepo in one place: what each does, where it lives, and when you would change it.

## Workspace layout

The root `Cargo.toml` defines a workspace with `resolver = "3"` and Rust edition 2024. `vendor/` and `forte/rs-to-ts/` are excluded from the workspace. Control lives in `fn0/control/` and is also excluded.

```
fn0/
├── forte/
│   ├── sdk/           forte-sdk            Runtime library for WASM handlers
│   ├── codegen/       forte-codegen        build.rs helpers; route + env generation
│   ├── cli/           forte-cli            Developer CLI (forte dev/build/deploy…)
│   ├── macros/        forte-macros         Proc-macros: #[forte_sdk::test], #[forte_doc], #[cache_static]
│   ├── json/          forte-json           Streaming JSON codec; snake/camelCase, t discriminant
│   ├── wit/           forte-wit            Embeds WASI WIT definitions (wasi:http p3 world)
│   ├── test-runner/   forte-test-runner    Binary test runner for wasm32-wasip2 targets
│   └── rs-to-ts/      forte-rs-to-ts       Standalone binary: Rust → TypeScript type generation
├── fn0/
│   ├── ski/           fn0-ski              WinterCG JS runtime (V8/deno_core) for SSR
│   ├── doc-db/        fn0-doc-db           Document-oriented Turso/libSQL wrapper (WASM + native)
│   ├── object-storage/ fn0-object-storage  S3-compatible object storage client
│   ├── worker/        fn0-worker           Worker process binary
│   ├── worker-agent/  fn0-worker-agent     Per-instance supervisor for blue-green deploys
│   └── worker-proxy/  fn0-worker-proxy     TCP forwarder fronting fn0-worker containers
└── vendor/            deno_core            Vendored and patched for deterministic module map serialization
```

## forte-sdk

**Crate:** `forte-sdk` · **Target:** `wasm32-wasip2`

The runtime library every Forte application handler imports. Exposes:

- `ForteRequest` — incoming request (method, URI, headers, body, path/search params, cookies)
- `Body` — `Empty | Bytes(Bytes) | Stream(…)` plus `Body::channel()` for streaming responses
- HTTP client (`forte_sdk::http::Client`)
- WebSocket operations (`forte_sdk::websocket::send`, `disconnect`)
- Document database (`forte_sdk::doc_db`)
- Object storage (`forte_sdk::object_storage::private` and `::public`)
- Static page cache (`forte_sdk::static_page_cache`)
- Metrics (OTLP, delta temporality) and tracing
- Cookie signing (`forte_sdk::cookie`)
- UUID v7 (`forte_sdk::uuid`)
- Async runtime (`forte_sdk::runtime::spawn_local`, `block_on`, `yield_async`)
- Re-exports `forte_json`, `anyhow`, `serde`, `bytes`, `http`

**Change when:** adding or changing any SDK API that handler code calls; changing serialization behavior; adding new platform capabilities.

## forte-codegen

**Crate:** `forte-codegen` · **Target:** native (runs during build)

A build-script library. Forte projects call it from `build.rs`:

```rust
forte_codegen::generate_routes();
forte_codegen::generate_env();
```

`generate_routes()` scans `rs/src/{pages,apis,actions,hooks,queue_tasks,admin,ws_in,ws_out,ws_singleton}` by static source analysis, emits `rs/src/route_generated.rs` and `.forte/ws_singletons.json`, updates the `FORTE-MANAGED` block in `lib.rs`, and drives `forte-rs-to-ts` to produce TypeScript type stubs.

`generate_env()` reads `env.yaml` and emits type-safe accessor functions.

**Change when:** adding a new handler type; changing the `FORTE-MANAGED` block format; changing the discovery rules or generated route table shape; changing how env accessors are generated.

## forte-cli

**Crate:** `forte-cli` · **Target:** native (installed on developer machines)

The `forte` binary. Key subcommands:

| Command | What it does |
|---|---|
| `forte init` | Scaffold a new project (or `--dev` to link local SDK) |
| `forte dev` | Local dev server with live rebuild |
| `forte build` | Compile WASM + frontend bundle |
| `forte deploy` | Build and push to fn0 Cloud |
| `forte cloud login` | Install per-account broker Worker in a Cloudflare account |
| `forte cloud init` | Provision Cloudflare resources for a project |
| `forte env` | Manage environment variable values |
| `forte db` | Run document-database operations against the live project |
| `forte admin` | Invoke admin task handlers |
| `forte open` | Open the project URL in a browser |
| `forte purge` | Purge CDN cache entries |
| `forte destroy` | Tear down a deployed project |
| `forte add page/action` | Scaffold a new handler module |

**Change when:** adding CLI commands or options; changing deploy protocol; changing `forte dev` behavior; changing scaffold templates.

## forte-macros

**Crate:** `forte-macros` · **Target:** proc-macro crate (native)

Three proc-macros re-exported through `forte-sdk`:

- `#[forte_sdk::test]` — marks a wasm32-wasip2 test; pairs with `forte-test-runner`
- `#[forte_doc]` — annotates public types for TypeScript type generation
- `#[cache_static]` — enables lazy static page caching for a page handler

**Change when:** changing test harness behavior; changing how types are marked for ts generation; changing static page cache annotation semantics.

## forte-json

**Crate:** `forte-json` · **Target:** `wasm32-wasip2` (also usable native)

Streaming JSON codec with Forte's wire conventions:

- Serialization: struct fields → camelCase; enum variants → `{"t": "VariantName", …}`; `Option::None` fields are omitted
- Deserialization: keys accepted in both camelCase and snake_case; `t` discriminant for enums
- Used only for actions, hooks, pages, and APIs — queue tasks and admin tasks use `serde_json`

**Change when:** changing the camelCase/t-discriminant conventions; fixing serialization edge cases.

## forte-wit

**Crate:** `forte-wit` · **Target:** build-time

Embeds the WIT definitions for the `wasi:http` p3 world the Forte WASM component implements. Not imported directly by application code — `forte-sdk` and `forte-codegen` depend on it.

**Change when:** updating WIT interface versions; adding new WIT worlds.

## forte-test-runner

**Crate:** `forte-test-runner` · **Target:** native binary

Executes tests compiled to `wasm32-wasip2` via the `fn0:test-harness/harness` WIT interface. Must be installed separately from the workspace:

```sh
cargo install --path forte/test-runner
```

**Change when:** changing test harness protocol; changing how test results are reported.

## forte-rs-to-ts

**Crate:** `forte-rs-to-ts` · **Target:** native binary, nightly Rust, private rustc APIs

Analyzes Rust source files and emits TypeScript type definitions for types annotated with `#[forte_doc]`. Uses private rustc compiler APIs so it must be compiled with nightly Rust; the CLI downloads a pre-built binary automatically.

Lives in `forte/rs-to-ts/` which is excluded from the workspace because it requires nightly.

**Change when:** fixing TypeScript type mapping; supporting new Rust type patterns; changing the `#[forte_doc]` annotation semantics.

## fn0-ski

**Crate:** `fn0-ski` · **Target:** native binary embedded in `fn0-worker`

A WinterCG-compatible JavaScript runtime built on V8 and `deno_core`. Runs server-side React rendering (`renderToString`). The worker invokes it for each SSR request, passing the props JSON from the Rust handler and receiving HTML.

**Change when:** upgrading deno_core/V8; changing the SSR protocol between worker and ski; fixing SSR-specific JavaScript runtime behavior.

## fn0-doc-db

**Crate:** `fn0-doc-db` · **Target:** `wasm32-wasip2` + native

Document-oriented database wrapper over Turso/libSQL. Supports `get`, `put`, `delete`, `query`, `scan`, and batch operations. Works in WASM (via the WIT HTTP interface to Turso) and natively (for tests with `doc_db::memory()`).

Re-exported through `forte-sdk` as `forte_sdk::doc_db`.

**Change when:** adding document database operations; changing query semantics; fixing Turso protocol issues.

## fn0-object-storage

**Crate:** `fn0-object-storage` · **Target:** `wasm32-wasip2` + native

S3-compatible object storage client. Two namespaces:

- `object_storage::private` — objects readable only via signed requests
- `object_storage::public` — objects served publicly from a CDN hostname

Supports `put`, `get`, `delete`, presigned URL generation, and cache purge. In-memory implementations available for tests via `object_storage::private::memory()`.

Re-exported through `forte-sdk`.

**Change when:** adding storage operations; changing presigned URL behavior; fixing S3 protocol issues.

## fn0-worker

**Crate:** `fn0-worker` · **Target:** native Linux arm64 binary

The worker process: receives WASM bundles from fn0 Cloud, executes them in Wasmtime, routes HTTP and WebSocket requests to the appropriate handler, manages the guest-host boundary, and enforces per-request and per-project limits.

Deployed with `scripts/deploy-fn0-worker.sh`.

**Change when:** changing execution limits; changing WIT host implementations; changing deploy or bundle-loading protocol; adding new platform capabilities to expose to handlers.

## fn0-worker-agent

**Crate:** `fn0-worker-agent` · **Target:** native Linux arm64 binary

Per-container supervisor that manages blue-green deploys for `fn0-worker`. Handles health checks and draining during version transitions.

**Change when:** changing deploy lifecycle; changing health-check protocol.

## fn0-worker-proxy

**Crate:** `fn0-worker-proxy` · **Target:** native Linux arm64 binary

TCP forwarder that sits in front of `fn0-worker` containers. Routes connections during blue-green transitions.

**Change when:** changing the TCP forwarding or connection-draining behavior.

## vendor/deno_core

Vendored fork of `deno_core` with a patch for deterministic module map serialization. Lives outside the workspace. Do not update without checking the patch applies cleanly.

## Dependency graph (simplified)

```
Application code
    ↓ imports
forte-sdk ──────────────────── fn0-doc-db
    │                          fn0-object-storage
    │ re-exports                   ↑
forte-json                        │
forte-macros                      │
                              fn0-worker
build.rs calls                    ├── fn0-ski (SSR)
forte-codegen                     ├── fn0-doc-db
    ├── calls forte-rs-to-ts       ├── fn0-object-storage
    └── reads forte-wit            └── forte-wit (WIT defs)

forte-cli (developer machine)
    ├── drives forte-codegen
    └── deploys to fn0-worker via fn0 Cloud API

forte-test-runner (test machine)
    └── executes wasm32-wasip2 test binaries
```

## Targets at a glance

| Crate | Primary target |
|---|---|
| forte-sdk, forte-json, fn0-doc-db, fn0-object-storage | wasm32-wasip2 |
| forte-codegen, forte-macros, forte-wit | native (build-time) |
| forte-cli, forte-test-runner, forte-rs-to-ts | native (developer machine) |
| fn0-ski, fn0-worker, fn0-worker-agent, fn0-worker-proxy | native Linux arm64 |
