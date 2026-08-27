# fn0 Platform Deployment

This document covers deploying the fn0 platform itself (control plane and worker nodes), not deploying Forte applications onto fn0. For Forte app deployment, see [setup.md](../setup.md#deploying-to-fn0-cloud).

## Prerequisites

The following tools must be present in PATH before running any deploy script:

- `pulumi` — reads stack outputs (Turso credentials, image registry URL, etc.)
- `jq` — JSON processing in shell scripts
- `cargo` — resolves crate versions from workspace
- `aws` — S3/ECR access (R2 is S3-compatible)
- `oci` — OCI Vault and registry operations
- `curl` — calls control plane actions
- A container runtime (`docker` or `apple/container` on macOS)

Scripts detect missing tools and exit early with a clear error.

## Infrastructure

Infrastructure is managed with Pulumi (`infra/`). Stack outputs are loaded by every deploy script via `scripts/lib/pulumi-outputs.sh`. The scripts read credentials, DB host suffixes, image registry URLs, and control URLs from Pulumi — do not hard-code them.

## Deploying the Control Plane

The control plane (`fn0/control`) is a Forte application. Use the bootstrap script — **never** `forte deploy`:

```sh
scripts/bootstrap-fn0-control.sh
```

> **Why not `forte deploy`?** `forte deploy` bundles the local `fn0/control/env.yaml`, which only holds developer-visible keys. The pulumi-managed variables (`FN0_TELEMETRY_*`, vault credentials, etc.) are not in that file, so a `forte deploy`-produced bundle ships an incomplete environment. Since 2026-08-26 this breaks every log/trace action with an env panic on startup.

The script is **idempotent** — it can be re-run from any failure point:

1. Builds and pushes the cwasm-compiler Lambda for the workspace `fn0-wasmtime` version.
2. Resolves the already-published `fn0-worker` image (see Worker Deployment below).
3. Runs `forte build` on `fn0/control`, assembles `bundle.raw.tar`.
4. Uploads `original/fn0-control.tar` to the R2 bundle-store bucket.
5. Invokes the cwasm-compiler Lambda to produce the pre-compiled bundle.
6. Seeds the control Turso database with:
   - `Fn0WasmtimeVersionDoc` (active version)
   - `CompiledBundleDoc` (project_id=fn0-control, code_version)
   - `WorkerManifestDoc` (routes fn0-control to its registered domain + storage)
   - `ProjectCloudflareConfigDoc` (R2 buckets and credentials)
7. Seeds the worker-agent Turso database with the target `fn0-worker` image ref.

The script derives its environment entirely from `pulumi stack output` — it does not read `env.yaml`.

### When to run

Run `bootstrap-fn0-control.sh` every time you change the control plane source. It is the only safe redeployment path for control.

## Deploying Worker Nodes

```sh
scripts/deploy-fn0-worker.sh
```

This script is also **idempotent** and handles both code and Wasmtime version upgrades:

1. Resolves target `fn0-wasmtime` and `fn0-worker` versions from the workspace `Cargo.toml`.
2. Reads the current `Fn0WasmtimeVersionDoc` from the control DB.
3. If the target Wasmtime version differs from active and pending, calls `ensure_cwasm_pending` to register it and pre-compile all existing bundles on R2.
4. Builds and pushes the `fn0-worker` container image (skips if the image already exists in the registry).
5. Writes `TargetFn0WorkerConfigDoc.image_ref` to the control DB.
6. Waits until every live host's `WorkerHostStatusDoc` reports the new image ref as active.
7. If a new Wasmtime version was staged, promotes `pending → active` and removes the old Lambda + image.

**Rollback:** check out the old commit and re-run the script. It follows the same path in reverse.

### Blue-Green Lifecycle

`fn0-worker-agent` runs on each host and supervises the active container:

- Polls the control DB for the target image ref.
- Starts the new container and waits for health checks to pass.
- Drains the old container; existing WebSockets receive close code `1012` and the old container stays alive through their graceful close timeout.
- `fn0-worker-proxy` (a tiny TCP forwarder on `:443`) switches new connections to the active container after the agent writes the target file.

## Building Rust Binaries for Linux ARM64

All native Linux binaries are compiled with:

```sh
scripts/build-rust-linux-arm64-bin.sh <package> <out_dir>
```

This runs `cargo build --release -p <package>` inside a `rust:bookworm` container with the repo bind-mounted. The `target/` directory and Cargo registry live on persistent named Docker volumes (`fn0-build-target`, `fn0-build-cargo-registry`), so incremental compilation is fast across runs.

**Do not replace this with a `COPY`-into-`docker build` flow.** `docker build`'s `COPY . .` re-stamps source mtimes, causing Cargo to recompile the entire dependency graph (~500 crates, ~10 min per change). The bind-mount approach rebuilds only the changed crate (~1 min).

Convenience wrappers for individual binaries:

| Script | Output |
|---|---|
| `scripts/build-fn0-worker-agent.sh` | `fn0-worker-agent` ARM64 binary |
| `scripts/build-fn0-worker-proxy.sh` | `fn0-worker-proxy` ARM64 binary |
| `scripts/build-cwasm-compiler.sh` | cwasm-compiler Node.js Lambda bundle |

The `deploy-fn0-worker.sh` script calls `build-rust-linux-arm64-bin.sh fn0-worker` internally.

## Publishing Crates

Publish in dependency order from a machine that holds publish credentials:

1. `fn0-ski`
2. `fn0` (depends on `fn0-ski`)
3. `fn0-worker` (depends on `fn0`)

Flow: bump versions → commit → `cargo publish` locally. Never wait for `publish.yml` CI; crates.io indexing is fast enough that the next publish in the chain resolves immediately. CI publish and local publish are both idempotent — "already uploaded" is a no-op.

When a deploy step requires a dependency crate to be published (e.g., `deploy-fn0-worker.sh` needs `fn0` before `fn0-worker`), publish it locally rather than blocking on CI.

## Telemetry

The platform's telemetry stack is self-hosted on one node behind a Cloudflare Tunnel:

| Signal | Backend | Auth |
|---|---|---|
| Metrics | VictoriaMetrics (`metricsHostname`) | Basic auth built into the worker binary |
| Logs and traces | loggytracy (`telemetryHostname`) | Cloudflare Access service token at the edge |

loggytracy has no TLS or authentication of its own. A Cloudflare Access service token authenticates callers at the edge, and a Transform Rule overwrites `X-Scope-OrgID` there. Any change that exposes loggytracy's listener another way, or lets a caller's own tenant header survive the Transform Rule, is a security change — not a routing one.

Platform health can be assessed through the fn0-control dashboard, which exposes per-project log and trace viewers that query the telemetry backends via their HTTP query APIs (`/loki/api/v1/query_range` on loggytracy and `/select/logsql/query` for log streams). The telemetry node itself has no directly attached viewer UI; all reads go through the control plane. For raw data plane probing, use the backend APIs directly.

For setting up a new telemetry node: `scripts/setup-telemetry-node.sh` (or `scripts/setup-telemetry-node-remote.sh` for a remote node).
