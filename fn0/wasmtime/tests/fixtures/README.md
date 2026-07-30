# Test fixtures

## `rc-named-service.wasm`

A component whose WASI imports and exports carry the pre-ratification
`0.3.0-rc-2026-03-15` version, i.e. what every bundle deployed before the
wasmtime 47 bump looks like. It exports `wasi:http/handler`, imports
`wasi:http/{types,client}` and `wasi:clocks/{types,monotonic-clock}`, and is
otherwise the smallest thing that exercises the p3 surface forte-sdk uses.

Built from `experiments/workers-jspi-probe` at commit 162e423f:

```sh
cd experiments/workers-jspi-probe
cargo build --target wasm32-wasip2 --release
wasm-tools strip target/wasm32-wasip2/release/workers_jspi_probe.wasm \
  -o fn0/wasmtime/tests/fixtures/rc-named-service.wasm
```

It is checked in rather than built on demand because the RC WIT it needs no
longer exists in `forte/wit`, and because the point of the fixture is to stay
frozen: it is the input the version rewrite has to keep handling.
