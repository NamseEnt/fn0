# Hooks

Hooks are server-side functions that the backend can call on **itself** via HTTP (self-invocation). They are used to run logic that is logically separate from the main request path, for example background work that is still part of the same deployment.

Hooks live in `src/hooks/`.

## Defining a Hook

A hook file must define:

```rust
// src/hooks/send_welcome_email.rs

pub struct Input {
    pub user_id: u64,
    pub email: String,
}

pub struct Output {
    pub sent: bool,
}
// Output can also be an enum.

pub async fn handler(req: forte_sdk::ForteRequest<'_, Input>) -> Output {
    // req.body is the deserialized Input
    Output { sent: true }
}
```

Requirements detected by code generation (`forte_codegen::has_hook_handler`):
- A struct named `Input`
- A struct **or enum** named `Output`
- A `pub async fn handler`

## HTTP Contract

Hooks are routed to `POST /__self_invoke/{name}` where `{name}` is the file stem.

- **Request body**: JSON-encoded `Input` (deserialized via `forte_json::from_slice`)
- **Response body**: JSON-encoded `Output` (serialized via `forte_json::to_vec`)
- **Response headers**: `Content-Type: application/json`
- Cookies written to `req.jar` are sent back as `Set-Cookie` headers.

## Difference from Actions

| | Hooks | Actions |
|--|-------|---------|
| Caller | Backend code (self-invoke) | Frontend JavaScript |
| Path prefix | `/__self_invoke/` | `/__forte_action/` |
| Typical use | Internal background work | Frontend RPC |

## Self-Invocation

To call a hook from the backend, make an outbound HTTP request to `/__self_invoke/{name}` on the same worker. Use `forte_sdk::http::Client` for the request. The `FN0_SELF_URL` or similar environment variable may be needed — Unknown from repository, check `fn0/fn0/src/self_invoke.rs`.

## JSON Encoding

Same conventions as actions — see [json.md](./json.md).
