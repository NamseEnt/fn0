# TypeScript / Hono on fn0

fn0 supports TypeScript apps built with [Hono](https://hono.dev/) using the `fn0` CLI. The build pipeline compiles TypeScript to a WASM component via [jco](https://github.com/bytecodealliance/jco) and runs it in the fn0 Wasmtime runtime.

> **Most developers should use `forte init` instead.** Forte is a full-stack framework (Rust + React) with a dev server, hot reload, SSR, typed clients, and database/storage integrations built in. The TypeScript/Hono path targets apps that specifically need JavaScript on fn0 without Forte's conventions.

## Prerequisites

- **Bun ≥ 1.0** — used as the package manager and local development runtime
- **fn0 CLI** — install via `cargo binstall fn0-cli` or `cargo install --path fn0/cli`

## Creating a Project

```sh
fn0 init my-app
```

The CLI prompts for:
- Language — choose **TypeScript**
- Package manager — choose **Bun**
- Framework — choose **Hono**

This creates a project directory with the following structure:

```
my-app/
├── fn0.toml              # project config (name, language_env, project_id)
├── package.json          # dev/build scripts
├── tsconfig.json
├── rolldown.config.mjs   # bundler config
├── .gitignore
├── wit/
│   ├── component.wit     # WIT interface (exports wasi:http/incoming-handler)
│   └── deps/             # WIT dependencies
└── src/
    ├── index.ts          # Hono app
    └── component.ts      # WASM component entry (calls fire(app))
```

## `fn0.toml`

The project config is written by `fn0 init` and updated by `fn0 deploy`:

```toml
name = "my-app"
language_env = "TypescriptBunHono"
# project_id is added after the first deploy
```

`language_env = "TypescriptBunHono"` tells the CLI which toolchain to use for `fn0 build` and `fn0 local`.

## Local Development

For local development, run the Hono app directly with Bun — no fn0 runtime is needed:

```sh
cd my-app
bun run dev   # runs bun run src/index.ts
```

This uses Bun as the HTTP server, so Node.js and Bun APIs are available in this mode. They are **not** available when the app runs on fn0 (see [Limitations](#limitations) below).

## Building

```sh
fn0 build
# or equivalently:
bun run build
```

The build runs two steps:
1. **Rolldown** — bundles `src/component.ts` (and its imports) to `dist/component.js`
2. **jco componentize** — compiles `dist/component.js` into `dist/component.wasm`, a standard WASI HTTP component

```
rolldown -c && jco componentize -w wit -o dist/component.wasm dist/component.js
```

The resulting `dist/component.wasm` is the artifact that fn0 runs in production.

## Local fn0 Testing

To test the WASM component in the fn0 Wasmtime runtime locally:

```sh
fn0 local
fn0 local --port 8080   # custom port (default: 3000)
```

This runs `fn0 build` first, then starts a local HTTP server backed by Wasmtime. Use this to catch issues that only appear in the WASM environment before deploying.

## Deploying

```sh
fn0 login         # authenticate with fn0 Cloud (one time)
fn0 deploy        # build and upload to fn0 Cloud
```

`fn0 deploy` builds the project (equivalent to `fn0 build`), packages `dist/component.wasm` into a bundle, and uploads it to fn0 Cloud. The first deploy registers the project and writes the `project_id` back to `fn0.toml`.

### First deploy prerequisites

The same Cloudflare setup as Forte is required before the project can serve traffic. See [setup.md](../setup.md#deploying-to-fn0-cloud) for `forte cloud init` steps (fn0 CLI shares the same cloud setup flow).

## Hono Application

The generated `src/index.ts` is a standard Hono app:

```ts
import { Hono } from "hono";

const app = new Hono();

app.get("/", (c) => c.text("Hello Hono!"));

export default app;
```

Add routes, middleware, and handlers as you would with any Hono app. Hono's core API is fully available; adapters for Node.js, Deno, Bun-specific features, and `node:*` imports are not.

The generated `src/component.ts` wires the Hono app to the WASI HTTP interface:

```ts
import app from "./index";
import { fire } from "@bytecodealliance/jco-std/wasi/0.2.6/http/adapters/hono/server";

fire(app);

export { incomingHandler } from "@bytecodealliance/jco-std/wasi/0.2.6/http/adapters/hono/server";
```

Do not change `component.ts` unless you need to export additional WASM interfaces.

## WIT Interface

The WIT component interface is in `wit/component.wit`:

```wit
package example:hono;

world component {
    export wasi:http/incoming-handler@0.2.6;
}
```

This is the standard WASI HTTP handler contract. fn0 calls the `incoming-handler` export for every HTTP request.

## Limitations

Code compiled via `jco componentize` runs in a sandboxed WASM environment:

- **No Node.js APIs.** `node:fs`, `node:crypto`, `node:path`, etc. are unavailable. Use Web-standard alternatives (e.g. `crypto.subtle` for cryptography).
- **No Bun APIs.** `Bun.file`, `Bun.serve`, etc. are unavailable. Local dev with `bun run dev` hides this — always test with `fn0 local` before deploying.
- **No dynamic imports.** The bundle must be fully static (Rolldown enforces this).
- **Subject to the same fn0 Cloud limits as Rust components.** See [limits.md](limits.md) for CPU time, memory, and request body limits.

## fn0 CLI Commands

| Command | Description |
|---|---|
| `fn0 init [name]` | Scaffold a new project (prompts for language, package manager, framework) |
| `fn0 build` | Bundle TypeScript and compile to `dist/component.wasm` |
| `fn0 local [--port <port>]` | Run the WASM component in fn0 locally |
| `fn0 deploy` | Build and deploy to fn0 Cloud |
| `fn0 destroy` | Delete the deployed project |
| `fn0 login [token]` | Authenticate with fn0 Cloud |
| `fn0 env set <key> <value> [--secret]` | Set an environment variable |
| `fn0 env list` | List all environment variables |
| `fn0 env unset <key>` | Remove an environment variable |

See [overview.md](overview.md#fn0-cli-commands) for the full command reference.
