# fn0 Platform Overview

fn0 (pronounced "f-n-zero") is a FaaS (Function-as-a-Service) platform powered by [Wasmtime](https://wasmtime.dev/). It executes WebAssembly components compiled to WASI 0.2 (Component Model).

## Core Concepts

### Execution Model

fn0 uses the same model as Cloudflare Workers:

- A single WASM instance (and, for JS deployments, a single V8 isolate) is reused to serve many concurrent requests on the same worker thread.
- Handlers must be **effectively stateless across requests**. Module-level mutable state must not carry information between requests because another request may be interleaved at any `await` point.
- Module-level initialization runs once. Per-request setup belongs inside the handler.

fn0 does not enforce this contract. Violating it causes request-level data leakage or inconsistent behavior.

### Response Specification

The WASM component returns an HTTP response. fn0 inspects it:

- If the response has status 200 and the header `x-fn0-next: js`, fn0 delegates to the named runtime (currently only `js` — the Ski JavaScript runtime).
- Otherwise fn0 forwards the response as-is.
- All `x-fn0-*` headers are stripped from the response before it is sent to the client.

Forte uses `x-fn0-next: js` for page handlers (SSR) and skips it for API endpoints and actions.

### fn0 Cloud Limits

These limits apply to fn0 Cloud. Self-hosted deployments can remove them.

| Limit | Value |
|---|---|
| Request headers | 128 KB |
| Request body | 100 MB |
| Response headers | 128 KB |
| Response body | Unlimited |
| Memory | 128 MB |
| CPU time | 10 ms |
| Max duration | 15 seconds |
| Subrequests (external HTTP) | 50 per request |

### Cluster Architecture (Internal)

- **Monolith architecture** — no microservices.
- On startup, each instance calls the cloud provider's Instance Discovery API (AWS or OCI), then uses [memberlist](https://github.com/al8n/memberlist) for gossip-based cluster membership.
- Request routing uses the **Power of Two Choices** algorithm: pick two warmed instances, forward to the less loaded one.
- If the first forward is rejected, retry once. If all retries fail or no warm instances exist, attempt a cold-start (the instance may start on itself).
- If all instances are overloaded, return HTTP 503.
- WASM modules are cached in memory after the first download. On subsequent requests, the module's last-modified time is checked; re-download only if updated.

## Packages

| Package | Version | Description |
|---|---|---|
| `fn0` | 0.2.27 | Core FaaS runtime (`ExecutionContext`, `Bundle`, `build_engine`) |
| `fn0-cli` | 0.1.0 | Local development CLI |
| `fn0-worker` | 0.3.23 | Worker binary (distributed execution node) |
| `fn0-worker-agent` | 0.1.0 | Per-instance container supervisor (blue-green deploys, self DNS, in-host TCP proxy) |
| `fn0-deploy` | 0.1.6 | fn0 Cloud deployment client |
| `fn0-wasmtime` | 0.1.3 | Wasmtime wrapper with fn0-specific config |
| `fn0-ski` | 0.1.4 | WinterCG-compatible JS runtime (Deno-based, no Node.js) |
| `fn0-compiler` | 0.1.0 | Compiler utilities (internal) |
| `hq` | 0.1.0 | Headquarter management service |

## fn0-cli (Local Development)

The fn0 CLI (`fn0/cli`) provides local development tooling:

```sh
# Initialize a new fn0 project
fn0 init

# Build (compiles to WASM)
fn0 build

# Run locally
fn0 local
```

Unknown from repository: detailed `fn0-cli` commands — check `fn0/cli/README.md` and `fn0/cli/src/`.

## Supported Languages

- **Rust** — primary target; compiles to `wasm32-wasip2`
- **JavaScript / TypeScript** — via the Ski runtime (WinterCG subset, no Node.js APIs)

## Supported Cloud Providers

- Amazon Web Services (AWS) — EC2, ECS
- Oracle Cloud Infrastructure (OCI)

## Supported CDN Providers

- Cloudflare Workers (integration)

## Supported Code Storage Providers

- File system (including NFS, e.g. AWS EFS)
- S3 and compatible object storage (via `opendal`)

## Observability

fn0 has built-in OpenTelemetry support:

- OTLP exporter via `fn0/fn0/src/otlp_hijack.rs`
- Structured logging via `tracing`
- Metrics and distributed tracing

Unknown from repository: OTLP endpoint configuration for self-hosted deployments — check `fn0/fn0/src/telemetry.rs`.

## Hijack Architecture

fn0 uses "hijack" components to inject platform services into the WASM execution environment without modifying the user's code:

| Hijack | Purpose |
|---|---|
| `turso_hijack` | Injects Turso/libSQL database connection |
| `otlp_hijack` | Injects OpenTelemetry OTLP endpoint |
| `queue_hijack` | Intercepts outgoing queue requests |
| `vault_hijack` | Injects secrets (Vault integration) |
| `control_invoke_queue_hijack` | Routes control-plane queue invocations |

These are configured on `ExecutionContext` via builder methods:

```rust
let ctx = ExecutionContext::new(engine, linker, bundle_cache)
    .with_turso_hijack(turso_config)
    .with_otlp_hijack(otlp_config);
```
