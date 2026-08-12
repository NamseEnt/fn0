# WebSockets

Forte WebSockets use event callbacks while fn0 owns the network connection. Create route modules
under `rs/src/websockets`; they are published below `/ws`.

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

## Mapping

| Module | URL |
| --- | --- |
| `websockets/index.rs` | `/ws` |
| `websockets/chat.rs` | `/ws/chat` |
| `websockets/rooms/[room_id].rs` | `/ws/rooms/:room_id` |

A dynamic module declares `PathParams` and receives it after the event argument, matching page and
API routes.

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

See [Limits & Quotas](../fn0/limits.md) and the internal
[WebSocket design](../design/forte-websockets.md) for queue, size, and lifecycle details.
