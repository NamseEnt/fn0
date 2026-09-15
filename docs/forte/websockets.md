# WebSockets

Forte WebSockets use event callbacks while fn0 owns the network connection. Create inbound route
modules under `rs/src/ws_in`; they are published below `/ws`. Create outbound route modules under
`rs/src/ws_out`; they receive messages from connections opened by the application. Create singleton
route modules under `rs/src/ws_singleton`; fn0 maintains one current connection assignment per
singleton and routes callbacks through the worker that owns the current physical connection.
An old physical socket may close before its reservation expires; the reservation and fencing checks
remain authoritative while sends to that unavailable connection fail.

## Local development

`forte dev` accepts WebSocket upgrades for `/ws` and `/ws/...` and runs the same generated
`on_connect`, `on_message`, and `on_disconnect` callbacks as the deployed worker. Text and binary
messages are delivered through the internal callback request body protocol. Selected subprotocols
and application response headers are forwarded when they pass the same platform header checks;
transport and `x-fn0-*` headers remain owned by fn0.

The local server installs a WebSocket hijack, so `forte_sdk::websocket::send` and
`forte_sdk::websocket::disconnect` operate on active local connections. A Rust source rebuild or
server shutdown closes active connections with close code `1012` and invokes `on_disconnect` on a
best-effort basis. Local development has no distributed ownership or cross-worker routing.

Generated `ws_out` `connect(url)` functions support `ws://` and `wss://` targets and route inbound
messages to the corresponding callbacks. The target must be reachable from the development
machine and present a certificate trusted by the bundled WebPKI root certificates for `wss://`
connections.
Persistent `ws_singleton` connections are not opened by `forte dev`.

## Route example

```rust
use forte_sdk::anyhow::Result;
use forte_sdk::websocket::{
    ConnectDecision, ConnectEvent, DisconnectEvent, IncomingMessage, MessageEvent,
    WebSocketMessage,
};

pub async fn on_connect(event: ConnectEvent) -> Result<ConnectDecision> {
    if event.requested_protocols.iter().any(|protocol| protocol == "chat.v1") {
        Ok(ConnectDecision::accept_with_protocol("chat.v1"))
    } else {
        Ok(ConnectDecision::reject(forte_sdk::http::StatusCode::BAD_REQUEST))
    }
}

pub async fn on_message(event: MessageEvent) -> Result<()> {
    let response = match event.message {
        IncomingMessage::Text(text) => WebSocketMessage::text(text),
        IncomingMessage::Binary(bytes) => WebSocketMessage::binary(bytes),
    };
    forte_sdk::websocket::send(&event.connection_id, response).await?;
    Ok(())
}

pub async fn on_disconnect(_event: DisconnectEvent) -> Result<()> {
    Ok(())
}
```

`on_connect` and `on_message` are required. `on_disconnect` is optional and best-effort. An error
returned by `on_message` or `on_disconnect` is logged and metered but does not disconnect the
client. Use `forte_sdk::websocket::disconnect` for an application-requested graceful close.

`DisconnectEvent` carries `connection_id`, `close_code: Option<u16>`, `reason: Option<String>`, and `cause: DisconnectCause`:

| Cause | Meaning |
| --- | --- |
| `Peer` | The client closed the connection |
| `Application` | `websocket::disconnect` was called |
| `Deployment` | The deploy replaced the running instance |
| `HeartbeatTimeout` | The connection missed a heartbeat |
| `ProtocolError` | WebSocket protocol violation |
| `TransportError` | Network-level transport failure |
| `InternalError` | Internal fn0 error |

## Mapping

| Module | URL |
| --- | --- |
| `ws_in/index.rs` | `/ws` |
| `ws_in/chat.rs` | `/ws/chat` |
| `ws_in/rooms/[room_id].rs` | `/ws/rooms/:room_id` |

A dynamic module declares `PathParams` and receives it after the event argument, matching page and
API routes.

## Outbound routes

Outbound routes do not accept inbound client connections. They define `on_message` and may define
`on_disconnect`; `on_connect` is not allowed. Forte generates a route-bound `connect` function for
each outbound route. Dynamic path segments (`[param]`) are not allowed in outbound route modules —
the build panics if any are present.

| Module | Generated path |
| --- | --- |
| `ws_out/slack.rs` | `crate::ws_out::slack::connect(url)` |
| `ws_out/index.rs` | `crate::ws_out::connect(url)` |

The generated callback path for `ws_out/slack.rs` is `/ws_out/slack`. It is an internal callback
route, not a public WebSocket endpoint.

```rust
let connection_id = crate::ws_out::slack::connect("wss://example.com/socket").await?;
forte_sdk::websocket::send(
    &connection_id,
    forte_sdk::websocket::WebSocketMessage::text("hello"),
)
.await?;
```

## Connect decisions

`ConnectEvent` exposes the connection ID, URI, headers, client address, and requested WebSocket
subprotocols. Return `ConnectDecision::Accept` with an optional selected protocol and response
headers, or `ConnectDecision::Reject` with any non-101 status and response headers.

The selected protocol must be one the client requested. Forte controls the WebSocket handshake,
transport, and every `x-fn0-*` header, so those response headers cannot be overridden.

## Sending

`WebSocketMessage::Text(Body)` and `Binary(Body)` accept buffered or streaming HTTP bodies. A
streaming body is read only when its connection reaches the front of the send queue. Text is
validated incrementally as UTF-8.

```rust
let (mut writer, body) = forte_sdk::http::Body::channel();
forte_sdk::runtime::spawn(async move {
    let _ = writer.write_all(first_chunk).await;
    let _ = writer.write_all(second_chunk).await;
});
forte_sdk::websocket::send(
    &connection_id,
    forte_sdk::websocket::WebSocketMessage::Binary(body),
)
.await?;
```

A successful send means the owning worker wrote and flushed the message, not that the browser
processed it. Inspect `WebSocketSendError::delivery_state()` before deciding whether an
application-level retry is safe. Forte does not retry automatically.

## Recovery

WebSocket delivery is at-most-once and not durable. Deploys close affected project connections
with `1012`. Clients reconnect, fetch authoritative state over HTTP, and only then resume applying
live messages.

## Singleton connections

Singleton routes model **one named outbound connection assignment per project** — a market-data
feed, a third-party push channel, or a chat firehose. The assignment is project-scoped and the
singleton name is derived from the module path. fn0 opens the current physical socket and re-opens
it when reconciliation finds that the assignment is missing or expired. Version changes,
disconnects, crashes, activation failures, and declaration removals preserve the reservation until
its deadline, so a replacement claim waits even when the old socket has already closed.

Create modules under `rs/src/ws_singleton`. Dynamic path segments (`[param]`) are rejected at
build time. `codegen` scans this directory recursively for `.rs` files, and derives the singleton
id from the file path relative to `ws_singleton/`:

| Module | Singleton id | Route |
| --- | --- | --- |
| `ws_singleton/market_feed.rs` | `market_feed` | `/ws_singleton/market_feed` |
| `ws_singleton/feeds/us_market.rs` | `feeds/us_market` | `/ws_singleton/feeds/us_market` |

Every discovered singleton is written to `.forte/ws_singletons.json` by the build (each entry has
`singleton_id` and `route_path`). `forte deploy` reads the manifest and posts the declarations to
control, which validates them and rejects the deploy if any `singleton_id` is empty, duplicated,
or does not match `/ws_singleton/<singleton_id>`.

### Handler shape

Each singleton module defines `connect` (required), `on_message` (required), and optionally
`on_connect` and `on_disconnect`. All are `pub async` and return `Result<...>`; `codegen` panics
at build time otherwise.

```rust
use forte_sdk::anyhow::Result;
use forte_sdk::websocket::{
    DisconnectEvent, IncomingMessage, MessageEvent, SingletonConnectEvent,
    SingletonConnectionOptions, WebSocketMessage,
};

pub async fn connect() -> Result<SingletonConnectionOptions> {
    Ok(SingletonConnectionOptions::new("wss://feed.example.com/market"))
}

pub async fn on_connect(event: SingletonConnectEvent) -> Result<()> {
    tracing::info!(?event.connection_id, ?event.protocol, "singleton up");
    Ok(())
}

pub async fn on_message(event: MessageEvent) -> Result<()> {
    match event.message {
        IncomingMessage::Text(text) => tracing::info!(%text, "tick"),
        IncomingMessage::Binary(bytes) => tracing::info!(len = bytes.len(), "tick"),
    }
    Ok(())
}

pub async fn on_disconnect(_event: DisconnectEvent) -> Result<()> {
    Ok(())
}
```

`connect` is called by fn0 whenever the singleton needs to be (re-)established. It returns a
`SingletonConnectionOptions` describing the target URL and optional per-connection headers or
requested subprotocols. Add extras with the builder-style helpers:

```rust
let mut options = SingletonConnectionOptions::new("wss://feed.example.com/market");
options.headers.insert("authorization", "Bearer ...".parse()?);
options.protocols.push("market.v1".to_string());
Ok(options)
```

The URL must be `ws://` or `wss://` and must resolve to a public internet address; see
[Outbound destinations](../fn0/limits.md#outbound-destinations). Reserved headers (`host`, `content-length`, `upgrade`,
`connection`, `sec-websocket-*`, and anything starting with `x-fn0-`) are stripped; fn0 owns
those. Protocols must be non-empty, must not contain whitespace, and must not contain commas.

`on_connect` fires once each time the connection completes the WebSocket handshake and receives
the selected `Sec-WebSocket-Protocol`, if any. `on_message` fires for every inbound frame.
`on_disconnect` is best-effort, with the same `DisconnectCause` mapping as inbound routes; use
`Deployment` and `TransportError` to distinguish an intentional restart from a wire drop.

### Sending and disconnecting

`forte build` generates a `send` function for each singleton module, addressed by module path.
`rs/src/ws_singleton/market_feed.rs` produces `crate::ws_singleton::market_feed::send`, and
`rs/src/ws_singleton/feeds/us.rs` produces `crate::ws_singleton::feeds::us::send`. Any handler in
the same project can call it; another project cannot address the singleton.

```rust
crate::ws_singleton::market_feed::send(
    WebSocketMessage::text("{\"op\":\"subscribe\",\"symbol\":\"AAPL\"}"),
)
.await?;
```

The runtime resolves the current active connection once, caches that result only until a safety
deadline before the control reservation expires, and sends once. Concurrent cache misses for the
same `(project_id, singleton_id)` share one control lookup. It returns
`WebSocketSendError::ConnectionNotFound` when the singleton has no active connection: it is
connecting, its lease expired, it belongs to an older deployment, its socket closed before the
reservation expired, or its owner was replaced before the write. A send never retries on a
replacement connection, because the replacement may not have finished its own subscription or
authentication. Delivery keeps the at-most-once rules of `forte_sdk::websocket::send`;
`delivery_state()` tells whether any bytes were written.

Singleton callbacks also receive the `ConnectionId` of their current connection, and
`forte_sdk::websocket::send` and `forte_sdk::websocket::disconnect` accept it as with other
WebSockets.

There is no public `connect_singleton` in the SDK — fn0 owns opening and re-opening the socket
in response to the manifest declaration, so user code never calls it.

Runtime pause, resume, and public status operations are intentionally unsupported. The active
deployment is the desired state: remove a singleton module and deploy to stop reconciling it; add
the module and deploy to restore it. These are deployment procedures, not runtime WebSocket APIs.

The internal `websocket_singleton_status` action accepts worker heartbeat and disconnect reports;
it is not a public status query or a dashboard contract. The current runtime record contains the
assignment version, claim token, physical connection ID, preparing/active/terminating state, and
lease expiry. It does not store a handshake timestamp or the last error.

### Lifecycle

- Registered at deploy time from `.forte/ws_singletons.json`; a rename or removal takes effect on
  the next `forte deploy`.
- fn0 keeps one reservation per `(project, singleton_id)` even across many worker instances. A
  version change, disconnect, crash, activation failure, or declaration removal cannot create a new
  claim before the existing reservation expires. The socket may close earlier, and sends fail while
  the reservation remains.
- A new deploy closes project connections on the worker generation that adopts the new code. The
  old connection may close before its reservation expires; the replacement waits for that
  reservation before it can claim.
- Delivery is still at-most-once. A dropped message is not replayed.
- Per-connector owner reservations are renewed periodically; when a worker vanishes, control waits
  for the stored reservation to expire before handing the singleton to another worker and calling
  `connect` there.
- A worker that cannot renew for 30 seconds stops the connection. Every send admission, every
  outgoing frame, and every callback start checks the local lease deadline, so a worker that
  resumes after a long pause refuses traffic before its renewal task runs. Bytes already handed to
  the operating system before the deadline cannot be recalled.
- Message payloads count toward the project's monthly compute egress. When the quota is
  exhausted, sends fail with `EgressQuotaExceeded` and the connection closes with
  `DisconnectCause::EgressQuotaExceeded`.

See [Limits & Quotas](../fn0/limits.md) and the internal
[WebSocket design](../design/forte-websockets.md) for queue, size, and lifecycle details.
