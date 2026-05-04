# fn0 Platform Architecture

## Overview

fn0 is a distributed FaaS (Functions-as-a-Service) platform. It uses wasmtime to execute user WASM components. The design mirrors Cloudflare Workers: a single WASM instance is reused across many requests on the same thread.

## Components

| Component | Binary / Crate | Role |
|-----------|---------------|------|
| `fn0` | `fn0/fn0` | Core execution library (wasmtime integration, bundle caching, instance warm-up) |
| `fn0-worker` | `fn0/worker` | Worker node: HTTP server, queue consumer, deployment watcher |
| `fn0-hq` | `fn0/hq` | Headquarters: control plane, deployment orchestration, custom domains, DNS |
| `fn0-cli` | `fn0/cli` | Developer CLI: `init`, `build`, `local` |
| `fn0-deploy` | `fn0/deploy` | Deployment client library |
| `fn0-ski` | `fn0/ski` | Minimal WinterCG JS runtime (deno_core, no Node.js) |
| `fn0-wasmtime` | `fn0/wasmtime` | Shared wasmtime engine factory with tuned settings |
| `fn0-compiler` | `fn0/compiler` | WASM compilation tooling |

## Request Flow

```
User Request
     │
     ▼
Worker HTTP Server (hyper/tokio)
     │
     ├─[warm instance available]─────────────────────────────────────────────┐
     │                                                                        │
     └─[no warm instance]────────────────────────────────────────────────────┤
                                                                              │
                                                            Execute on Instance
                                                                     │
                                                            1. Download module if missing/stale
                                                            2. Instantiate WASM component
                                                            3. Run handler
                                                            4. Keep instance in memory (warm cache)
```

### Load Balancing

When a request arrives at a worker:

1. The worker finds **two** warm instances using the [power of two choices](https://www.eecs.harvard.edu/~michaelm/postscripts/handbook2001.pdf) algorithm and forwards to the less-loaded one.
2. If the forwarding is rejected, it tries once more.
3. If rejected again, or no warm instances exist, it runs the request itself (cold start or local capacity).
4. If all instances are saturated → returns **503 Service Unavailable**. This should be monitored and alerted.

Cluster membership uses [memberlist](https://github.com/al8n/memberlist.git) (SWIM protocol), bootstrapped via the cloud provider's instance discovery API.

## Wasmtime Configuration

Source: `fn0/wasmtime/src/lib.rs`

| Setting | Value | Reason |
|---------|-------|--------|
| Allocation strategy | Pooling | Reuse memory for low-latency warm starts |
| Max memory per instance | 128 MB | Matches fn0 Cloud limit |
| Epoch interruption | Enabled | CPU time enforcement |
| Cranelift optimization | None (fast compile) | Minimize cold-start latency |
| Parallel compilation | Enabled | Faster initial build |
| Component model | Enabled | WASI 0.2 support |
| Async support | Enabled | Async handlers |
| Memory protection keys | Enabled (if available) | Security isolation |

## Handler Contract

**Critical** — violations produce data leakage or inconsistent behavior.

- A single WASM instance serves **many concurrent requests** on one thread.
- Requests interleave at `await` points.
- **Do not use module-level mutable state to carry per-request data.**
- If shared state is necessary, guard it with `RefCell` and never hold it across an `await`.
- Module-level initialization (`lazy_static!`, `OnceCell`, etc.) runs once, not per request.

fn0 does not enforce this at runtime.

## Bundle Caching

Source: `fn0/fn0/src/cache.rs`

Modules are cached in memory and on S3-compatible storage. The worker checks the module's last-modified time and re-downloads if updated.

## JS Runtime (ski)

Source: `fn0/ski`

- Built on `deno_core`
- WinterCG-compatible (Fetch API, etc.)
- **No Node.js** features: no `require`, no built-in module resolution
- Expects **fully bundled** input (via rolldown/bun)
- Used when WASM returns `x-fn0-next: js` — see [response spec](./response-spec.md)

## Deployment Spec (`x-fn0-next`)

Source: `fn0/SPEC.md`

- `x-fn0-next: <runtime>` on HTTP 200 from the WASM component delegates the response to the specified runtime.
- Currently the only runtime is `js` (ski).
- All `x-fn0-*` headers are stripped before the response is forwarded to the client.
- Any other response is forwarded as-is.

## Worker Node

Source: `fn0/worker/src/`

Key modules:
- `main.rs` — HTTP server setup (hyper/tokio)
- `cache.rs` — S3-backed bundle cache
- `queue_consumer.rs` — Message queue processing
- `deployments_watcher.rs` — File-based deployment tracking
- `worker_pool.rs` — Request distribution
- `env_yaml.rs`, `env_crypto.rs` — Environment and secrets management

## HQ (Headquarters)

Source: `fn0/hq/src/`

Key modules:
- `deploy.rs` — Deployment orchestration
- `custom_domain.rs`, `custom_domain_job_worker.rs` — Custom domain management
- `dns/` — DNS record synchronization
- `site/` — Site management
- `cloudflare_saas.rs` — Cloudflare SaaS integration
- `lambda.rs`, `ssh.rs`, `ssh_pool.rs` — Cloud provider integrations
- `wasmtime_rollout.rs` — Wasmtime version rollout coordination

## Infrastructure

Source: `infra/pulumi/`

- **OCI** (Oracle Cloud Infrastructure): Compute instances for workers, VCN networking, Vault for secrets
- **AWS**: Lambda for preprocessing
- **Cloudflare**: CDN, DNS, static asset Workers

Pulumi stacks are TypeScript. Build and deploy via:

```sh
cd infra/pulumi
bun install
pulumi up
```

## Observability

- **OpenTelemetry** (`opentelemetry`, `opentelemetry-otlp`): distributed tracing exported via OTLP
- **tracing** / **tracing-subscriber**: structured logging
- CPU time measurement: `fn0/fn0/src/measure_cpu_time.rs`
- Warm-up tracking: `fn0/fn0/src/warm_up_map.rs`

## CI/CD

Source: `.github/workflows/`

- `release.yml` — cargo-dist release pipeline, triggered on version tags (`v0.x.y`)
  - Multi-platform artifact builds
  - Uses `sccache` for build caching
  - Publishes to GitHub Releases
- `publish.yml` — crates.io publication
- `release-forte-rs-to-ts.yml` — Dedicated release for `forte-rs-to-ts`
