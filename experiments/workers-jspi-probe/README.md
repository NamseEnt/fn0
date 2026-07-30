# workers-jspi-probe

P0 probe for [#80](https://github.com/NamseEnt/fn0/issues/80): can a `wasi:http`
p3 component run on Cloudflare Workers, lowered by `jco` and lifted through JSPI?

Verified 2026-07-30 against a **Workers free plan** account.

## Result: pass, with one blocker

The component runs under `wrangler dev` and deployed on the free plan, returning
200 with the fetched bytes streamed. The one blocker found is that a component
instance serves exactly one request (see "Instance reuse" below).

P1 took this to a real Forte app: `../workers-forte-app`. One conclusion recorded
here — that the free plan does not enforce its documented CPU limit — was
overturned there, and is withdrawn below.

## Layout

| Path | What it is |
|---|---|
| `src/lib.rs` | The component. Awaits `monotonic-clock.wait-for`, awaits an outbound `http/client.send`, returns the fetched bytes as a `stream<u8>` fed by a spawned task. No forte-sdk, so a failure points at the toolchain. |
| `wit/` | Copy of `forte/wit/wit` — the same WASI `0.3.0-rc-2026-03-15` definitions forte-sdk builds against. |
| `worker/shim.js` | Workers-side implementation of every host import the glue asks for. Shared with `../workers-forte-app`, so it covers that app's surface too. |
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

### `async: true` in wit-bindgen produces a component jco rejects

With `async: true`, wit-bindgen lowers *every* import through the async ABI,
including functions that are plain `func` in WIT. jco refuses that component:

```
the `async` canonical option requires an async function type (at offset 0x41d98)
```

Dropping the option — wit-bindgen's default, where only WIT `async func`s are
async — transpiles. This probe is built that way, which is why it exists at all.

forte-sdk has since dropped the option too, so path A no longer needs a special
artifact; see `../workers-forte-app/README.md`.

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

### The documented 10 ms free-plan CPU limit — withdrawn, it is enforced

This section previously recorded that the limit was not enforced: `?burn=1e8` and
`?burn=1e9` — ~100 ms and ~960 ms of CPU by node's calibration of the same loop —
both returned 200 on a free-plan account.

**That does not reproduce.** Re-run hours later against the same worker and
account, `?burn=1e7` (~10 ms) and everything above it returns `1102 Worker
exceeded CPU limit`, and the threshold sits between 1e6 and 1e7 — the documented
number. The original reading was taken inside an anomalous window. The measured
table and what it costs a real app are in `../workers-forte-app/README.md`.

Deploying `limits.cpu_ms` does fail with
`CPU limits are not supported for the Free plan [code: 100328]`; that part
stands, and is unrelated to what is enforced.

The probe itself now returns 200 for only about 3 requests in 10, for the same
reason.

### Numbers

| Measurement | Value |
|---|---|
| Bundle, gzipped | 144.46 KiB (free-plan limit 3 MB) |
| Startup | accepted at deploy, so within the 1 s budget; wrangler reports no startup time |
| Instantiate, warm isolate | 0–1 ms |
| Instantiate, first request in an isolate | 8 ms |
| End-to-end, `wrangler dev` | 65–170 ms |
| End-to-end, deployed | 0.49–0.68 s typical, occasional 6 s outlier — measured in the window where the CPU limit was not being enforced |

`wrangler tail` produced no events during this run and the observability
telemetry API rejects wrangler's OAuth token (403, `Authentication error`), so
per-request CPU was never read directly; getting it needs an API token with
observability read. `wrangler tail` did work later, during P1, and is what
identified the `1102`s as `Worker exceeded CPU time limit`.
