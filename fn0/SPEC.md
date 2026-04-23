# fn0 Response Spec

wasm returns an HTTP response.

- `x-fn0-next: <runtime>` on status 200 delegates to the runtime.
- Otherwise fn0 forwards the response as-is.
- `x-fn0-*` headers are stripped.

Runtimes: `js`.
