# fn0 & Forte — Project Overview

## What is fn0?

**fn0** (pronounced "f-n-zero") is a FaaS (Functions-as-a-Service) platform built on [wasmtime](https://github.com/bytecodealliance/wasmtime). It executes user code compiled to WebAssembly (WASI 0.2 — Component Model). Inspired by Cloudflare Workers.

## What is Forte?

**Forte** is the fullstack web framework that targets fn0. It provides:

- A **Rust SDK** (`forte-sdk`) for writing server logic that compiles to `wasm32-wasip2`
- **Code generation** (`forte-codegen`) that scans your source tree and generates routing, action, hook, and queue-task dispatch boilerplate
- A **CLI** (`forte`) for building and running the application locally
- **Type generation** (`forte-rs-to-ts`) for sharing Rust types with a TypeScript/React frontend
- A custom **JSON library** (`forte-json`) with camelCase ↔ snake_case field mapping between Rust and JS

## Repository Layout

```
fn0/
├── fn0/            # FaaS platform (execution engine, CLI, worker, HQ server)
│   ├── fn0/        # Core execution engine (wasmtime integration)
│   ├── cli/        # fn0 CLI (init / build / local)
│   ├── worker/     # Worker node binary
│   ├── hq/         # Headquarters control server
│   ├── deploy/     # Deployment client library
│   ├── ski/        # Minimal WinterCG-compatible JS runtime (deno_core)
│   ├── wasmtime/   # Wasmtime engine configuration layer
│   └── compiler/   # WASM compiler tooling
├── forte/          # Forte framework libraries
│   ├── sdk/        # Runtime SDK for WASI:HTTP p3 components
│   ├── cli/        # forte CLI (build / dev)
│   ├── macros/     # Proc macros: #[test], #[forte_doc]
│   ├── json/       # Custom streaming JSON serializer
│   ├── codegen/    # Build-time code generator (routes, actions, hooks…)
│   └── rs-to-ts/   # Rust → TypeScript type generator
├── doc-db/         # Document-oriented database library (Turso/libSQL + in-memory)
├── infra/          # Infrastructure as Code (Pulumi / TypeScript)
│   ├── pulumi/     # Pulumi stacks for AWS, OCI, Cloudflare
│   └── cloud/      # TypeScript infra helpers
├── vendor/         # Vendored third-party crates (wasmtime, deno, oci-rust-sdk)
└── scripts/        # Build and deployment helper scripts
```

## Supported Languages

- **Rust** — primary language for server handlers
- **JavaScript / TypeScript** — via the ski runtime (WinterCG subset, no Node.js compatibility; bundled via rolldown/bun)

## fn0 Cloud Limits

| Resource | Limit |
|----------|-------|
| Request header | 128 KB |
| Request body | 100 MB |
| Response header | 128 KB |
| Response body | Unlimited |
| Memory | 128 MB |
| CPU time | 10 ms |
| Total duration | 15 s |
| Outbound subrequests | 50 |

These limits apply to the managed fn0 Cloud. Self-hosted deployments can remove them.

## Supported Cloud / CDN Providers

- **Compute**: Amazon Web Services (AWS), Oracle Cloud Infrastructure (OCI)
- **CDN**: Cloudflare (static assets, DNS, custom domains)
- **Code storage**: File system (including NFS/EFS), S3-compatible object storage

## License

AGPL v3. A commercial license is available for proprietary use — contact [projectluda@gmail.com](mailto:projectluda@gmail.com).

## See Also

- [setup.md](./setup.md) — prerequisites and getting started
- [forte/overview.md](./forte/overview.md) — Forte framework details
- [fn0/architecture.md](./fn0/architecture.md) — platform internals
