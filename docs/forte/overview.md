# Forte Framework Overview

Forte is a full-stack web framework built on top of [fn0](../fn0/overview.md). It compiles a Rust backend to WebAssembly (WASI Component Model) and pairs it with a React/TypeScript frontend built by Vite.

## Architecture

```
┌────────────────────────────────────────────┐
│  Request                                   │
└────────────┬───────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────┐
│  fn0 runtime (Wasmtime)                    │
│  Executes backend.wasm                     │
└────────────┬───────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────┐
│  route_generated.rs  (auto-generated)      │
│  Routes request to page / action handler   │
└────────────┬───────────────────────────────┘
             │ page request
             ▼
┌────────────────────────────────────────────┐
│  Rust page handler                         │
│  Returns Props (serialized to JSON)        │
│  Sets x-fn0-next: js header                │
└────────────┬───────────────────────────────┘
             │
             ▼
┌────────────────────────────────────────────┐
│  JS runtime (server.js SSR bundle)         │
│  React SSR renders Props → HTML            │
└────────────────────────────────────────────┘
```

For API endpoints (`src/apis/`), the Rust handler returns JSON directly — no SSR step.

## Key Packages

| Package | Crate | Purpose |
|---|---|---|
| `forte-sdk` | `forte/sdk` | Runtime library for wasm components (HTTP types, `ForteRequest`, cookie utilities, etc.) |
| `forte-cli` | `forte/cli` | Developer CLI (`forte dev`, `forte build`, `forte deploy`, etc.) |
| `forte-macros` | `forte/macros` | Procedural macros: `#[forte_sdk::test]`, `#[forte_doc]` |
| `forte-json` | `forte/json` | Streaming JSON serializer used for Props serialization |
| `forte-codegen` | `forte/codegen` | Build-script library that generates `route_generated.rs` |

## Project Structure

See [project-structure.md](project-structure.md) for the full layout.

## Workflow Overview

1. `forte init <name>` — scaffold a new project
2. `forte dev` — run dev server with hot reload
3. `forte add page <path>` / `forte add action <path>` — add handlers
4. `forte build` — compile backend (WASM) + frontend (Vite)
5. `forte deploy` — upload to fn0 Cloud

See [cli.md](cli.md) for all commands.
