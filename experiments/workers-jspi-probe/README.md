# workers-jspi-probe

P0 probe for [#80](https://github.com/NamseEnt/fn0/issues/80): can a `wasi:http`
p3 component run on Cloudflare Workers, lowered by `jco` and lifted through JSPI?

Verified 2026-07-30 against a **Workers free plan** account.

## Result: pass, with one blocker

The component runs under `wrangler dev` and deployed on the free plan, returning
200 with the fetched bytes streamed. The one blocker found is that a component
instance serves exactly one request (see "Instance reuse" below).

## Layout

| Path | What it is |
|---|---|
| `src/lib.rs` | The component. Awaits `monotonic-clock.wait-for`, awaits an outbound `http/client.send`, returns the fetched bytes as a `stream<u8>` fed by a spawned task. No forte-sdk, so a failure points at the toolchain. |
| `wit/` | Copy of `forte/wit/wit` — the same WASI `0.3.0-rc-2026-03-15` definitions forte-sdk builds against. |
| `worker/shim.js` | Workers-side implementation of every host import the glue asks for. |
| `worker/index.js` | The `fetch` handler: instantiates the component, converts request/response. |
| `worker/gen/` | `jco transpile` output. Regenerate with the command below; not hand-edited. |
| `jspi-check/` | Two-line Worker that reports whether `WebAssembly.promising` exists. The cheapest reproduction of the gate. |

## Reproducing

```sh
cargo build --target wasm32-wasip2 --release
npx -y @bytecodealliance/jco@latest transpile \
  target/wasm32-wasip2/release/workers_jspi_probe.wasm -o worker/gen \
  --async-mode jspi --async-wasi-imports --async-wasi-exports \
  --instantiation async --no-wasi-shim --name probe
npx -y wrangler@latest dev            # then curl localhost:8787
npx -y wrangler@latest deploy
```

`wrangler` must be `@latest`; the locally installed 4.90.1 caps out at
compatibility date 2026-05-15.

Query parameters on the deployed Worker:

- (none) — the probe; a fresh component instance per request
- `?reuse=1` — reuse one instance across requests, which hangs after the first
- `?burn=<iterations>` — spin the CPU for N iterations, used to find where the
  plan's CPU limit actually bites

## Findings

### JSPI is available, with no compatibility flag

`WebAssembly.promising` and `WebAssembly.Suspending` are both `function` in
workerd at compatibility date 2026-07-29, in an ordinary JS Worker — not only in
Python Workers. The transpiled glue uses both (`WebAssembly.promising` wraps the
`[async-lift]` export, `new WebAssembly.Suspending` wraps async imports).

### `async: true` in wit-bindgen produces a spec-invalid component

With `async: true`, wit-bindgen lowers *every* import through the async ABI,
including functions that are plain `func` in WIT. jco refuses that component:

```
the `async` canonical option requires an async function type (at offset 0x41d98)
```

Dropping the option — wit-bindgen's default, where only WIT `async func`s are
async — transpiles.

This is not jco being behind, and not a wit-bindgen version to bump:

| Check | Result |
|---|---|
| `wasm-tools 1.254.0 validate --features all`, wit-bindgen 0.50 + `async: true` | same error, offset 0x41d98 |
| same, rebuilt on wit-bindgen **0.60.0** (newest) | same error, offset 0x42f13 |
| same, wit-bindgen default async | validates clean |

The Component Model Explainer states the rule directly: "the `async` option may
only be used with async function types". `async: true` async-lowers WIT `func`s,
which violates it. wasmtime 43 — what the fleet runs, on wasmparser 0.243 —
accepts it anyway, which is why every component we deploy today is built this way
and nothing has complained.

Two consequences. Path A cannot use the artifact forte-sdk produces today, at the
artifact level, before JSPI or Workers enter the picture. And independently of
#80, the components we ship are spec-invalid and only run because of the pinned
wasmtime; a wasmtime bump is where that surfaces.

Also of note for a future bump: wit-bindgen 0.60 renamed `spawn` to
`spawn_local`.

### p3 streams and futures land on web standards

jco maps `stream<u8>` to `ReadableStream` and `future<T>` to `Promise`, so the
Workers side is a natural fit: an upstream `fetch` response body is handed
straight to `Response.consumeBody`, and the guest's response stream becomes a
`ReadableStream` for the Workers `Response`.

Two mismatches the shim absorbs: guest-to-host chunks arrive as plain number
arrays, and the guest stream's `read()` defaults to a count of **1**, so it must
be driven with an explicit `read({count})` rather than `for await`.

### The host surface is small and fully enumerable

`grep -o "fnName = '[^']*'" worker/gen/probe.js` lists 28 functions. `wasi:cli/*`
and `wasi:io/*` @0.2.9 appear because Rust's wasm32-wasip2 std links them for
stdio and panics; routing `stderr` to `console.log` is what makes a guest panic
visible.

`@bytecodealliance/preview3-shim` (0.1.2, bundled with jco) implements this
surface for **Node.js only** — `lib/nodejs`, node `worker_threads` — so it is not
reusable here. `worker/shim.js` is the fork the issue predicted, at ~260 lines
for this narrow surface.

### Instance reuse hangs — the one blocker

Reusing an instantiated component across requests: the first request returns 200,
every later one hangs forever with no error and no log line. A fresh instance per
request works every time. Reproduce with `?reuse=1`; in production the hang shows
up a few requests in, because isolate spread hides it at first.

The cause was not isolated further. The cost of the workaround is small measured
locally — `new WebAssembly.Instance` for an already-compiled module is 0–1 ms
warm, 8 ms on the first request of an isolate.

### The documented 10 ms free-plan CPU limit is not what is enforced

The account is on the free plan — deploying `limits.cpu_ms` fails with
`CPU limits are not supported for the Free plan [code: 100328]`. Yet on that same
account:

| `?burn=` | wall time | same loop in node (V8) |
|---|---|---|
| 1e8 | 1.06 s | 100 ms CPU |
| 1e9 | 6.49 s | 960 ms CPU |

Both returned 200. A request burning ~1 s of CPU is ~100x the documented 10 ms
free-plan limit and was not killed, so "10 ms free-tier CPU rules out the free
plan for SSR" cannot be taken at face value. This does not prove what the ceiling
is; it proves the documented number is not the enforced one here.

### Numbers

| Measurement | Value |
|---|---|
| Bundle, gzipped | 144.46 KiB (free-plan limit 3 MB) |
| Startup | accepted at deploy, so within the 1 s budget; wrangler reports no startup time |
| Instantiate, warm isolate | 0–1 ms |
| Instantiate, first request in an isolate | 8 ms |
| End-to-end, `wrangler dev` | 65–170 ms |
| End-to-end, deployed | 0.49–0.68 s typical, occasional 6 s outlier |

The probe's own CPU time is not recorded: `wrangler tail` produced no events, and
the observability telemetry API rejects wrangler's OAuth token (403,
`Authentication error`). Getting it needs an API token with observability read.
