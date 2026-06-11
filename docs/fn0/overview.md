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
- **Public ingress (OCI)**: Cloudflare (orange proxy) → OCI L4 Network Load Balancer (always-free) → worker pool. Workers run in an OCI InstancePool with AutoScaling; the NLB is the single entry. This replaces an older DNS round-robin scheme where each worker self-registered its public IP in Cloudflare DNS.
- On startup, each instance calls the cloud provider's Instance Discovery API (AWS or OCI).
- Request routing uses the **Power of Two Choices** algorithm: pick two warmed instances, forward to the less loaded one.
- If the first forward is rejected, retry once. If all retries fail or no warm instances exist, attempt a cold-start (the instance may start on itself).
- If all instances are overloaded, return HTTP 503.
- WASM modules are cached in memory after the first download. On subsequent requests, the module's last-modified time is checked; re-download only if updated.

## Packages

| Package | Version | Description |
|---|---|---|
| `fn0` | 0.2.42 | Core FaaS runtime (`ExecutionContext`, `Bundle`, `build_engine`) |
| `fn0-cli` | 0.1.5 | Local development CLI |
| `fn0-worker` | 0.3.46 | Worker binary (distributed execution node) |
| `fn0-worker-agent` | 0.1.5 | Per-instance container supervisor (blue-green deploys, in-host TCP proxy) |
| `fn0-worker-proxy` | 0.1.0 | Tiny TCP forwarder fronting fn0-worker containers; polls a target file written by worker-agent |
| `fn0-deploy` | 0.1.12 | fn0 Cloud deployment client |
| `fn0-wasmtime` | 0.1.3 | Wasmtime wrapper with fn0-specific config |
| `fn0-ski` | 0.1.6 | WinterCG-compatible JS runtime (Deno-based, no Node.js) |
| `fn0-compiler` | 0.1.0 | Compiler utilities (internal) |

## fn0-cli Commands

The fn0 CLI (`fn0/cli`) provides tooling for projects deployed directly to fn0 without Forte:

| Command | Description |
|---|---|
| `fn0 init [--name <name>]` | Scaffold a new fn0 project (prompts for framework and language) |
| `fn0 build` | Compile the project to WASM |
| `fn0 local [--port <port>]` | Run locally on the given port (default: auto) |
| `fn0 deploy` | Build and deploy to fn0 Cloud |
| `fn0 destroy` | Delete the deployed project |
| `fn0 rename <new-name>` | Rename the deployed project |
| `fn0 login [token]` | Authenticate with fn0 Cloud |
| `fn0 admin run <task>` | Run an admin task against the deployed app |
| `fn0 domain add <domain>` | Attach a custom domain (CNAME) |
| `fn0 domain remove` | Detach the custom domain |
| `fn0 domain status` | Show custom domain status |
| `fn0 env set <key> <value> [--secret]` | Set an env entry (plain by default; `--secret` encrypts via vault and injects through vault_hijack at runtime) |
| `fn0 env list` | List env entries with their kind (plain / secret) |
| `fn0 env unset <key>` | Remove an env entry |

> **Note:** Most Forte developers use `forte` CLI instead of `fn0` CLI. `fn0` CLI is for projects that use fn0 as a raw FaaS platform (e.g., Hono-based TypeScript apps).

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

- OTLP span exporter via `fn0/fn0/src/otlp_hijack.rs` (worker-side) and `forte/sdk/src/otel.rs` (WASM-side)
- Structured logging via `tracing`
- Distributed tracing with per-request spans

For Forte apps running on fn0 Cloud, traces are exported to `http://fn0-otel.fn0.dev/v1/traces`. The service name is controlled via the `OTEL_SERVICE_NAME` environment variable (defaults to `"forte-app"`).

For self-hosted fn0 worker deployments, configure the OTLP endpoint via environment variables on the worker binary:

| Variable | Required | Default | Description |
|---|---|---|---|
| `FN0_OTLP_TARGET_HOST` | Yes | — | OTLP collector hostname (e.g. your Alloy or Grafana Cloud OTLP host) |
| `FN0_OTLP_TARGET_SCHEME` | Yes | — | URL scheme for the OTLP collector: `http` or `https` |
| `FN0_OTLP_AUTH` | Yes | — | Base64-encoded Basic auth credentials (`user:token`); use empty string for unauthenticated |
| `FN0_OTLP_TARGET_PATH_PREFIX` | No | `""` | Path prefix prepended to every OTLP request path |
| `FN0_OTLP_PLACEHOLDER_HOST` | No | `fn0-otel.fn0.dev` | Placeholder hostname used inside WASM apps |

The worker intercepts outgoing OTLP requests that target `FN0_OTLP_PLACEHOLDER_HOST` and rewrites them to `FN0_OTLP_TARGET_HOST` with the configured auth header.

## Hijack Architecture

fn0 uses "hijack" components to inject platform services into the WASM execution environment without modifying the user's code:

| Hijack | Purpose |
|---|---|
| `turso_hijack` | Injects Turso/libSQL database connection |
| `object_storage_hijack` | Routes & SigV4-signs per-project object storage requests |
| `otlp_hijack` | Injects OpenTelemetry OTLP endpoint |
| `queue_hijack` | Intercepts outgoing queue requests |
| `vault_hijack` | Injects secrets (Vault integration) |
| `cross_project_enqueue_hijack` | Routes cross-project queue enqueue calls |
| `cross_project_invoke_hijack` | Routes cross-project direct invocations |

These are configured on `ExecutionContext` via builder methods:

```rust
let ctx = ExecutionContext::new(engine, linker, bundle_cache)
    .with_turso_hijack(turso_config)
    .with_otlp_hijack(otlp_config)
    .with_cross_project_enqueue_hijack(enqueue_config)
    .with_cross_project_invoke_hijack(invoke_config);
```
