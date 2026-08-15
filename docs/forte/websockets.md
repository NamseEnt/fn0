# WebSockets

Forte WebSockets use event callbacks while fn0 owns the network connection. Create inbound route
modules under `rs/src/ws_in`; they are published below `/ws`. Create outbound route modules under
`rs/src/ws_out`; they receive messages from connections opened by the application.

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
| `ws_in/index.rs` | `/ws` |
| `ws_in/chat.rs` | `/ws/chat` |
| `ws_in/rooms/[room_id].rs` | `/ws/rooms/:room_id` |

A dynamic module declares `PathParams` and receives it after the event argument, matching page and
API routes.

## Outbound routes

Outbound routes do not accept inbound client connections. They define `on_message` and may define
`on_disconnect`; `on_connect` is not allowed. Forte generates a route-bound `connect` function for
each outbound route.

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

## Disconnect causes

`DisconnectEvent` includes a `cause: DisconnectCause` field that explains why fn0 closed the
connection:

| Variant | Meaning |
| --- | --- |
| `Peer` | The client closed the connection |
| `Application` | Your code called `forte_sdk::websocket::disconnect` |
| `Deployment` | A deploy closed the connection (clients receive close code `1012`) |
| `HeartbeatTimeout` | No ping/pong exchange within the keepalive window |
| `ProtocolError` | A WebSocket protocol violation |
| `TransportError` | A network-level error |
| `InternalError` | An fn0-internal error |

```rust
use forte_sdk::websocket::{DisconnectCause, DisconnectEvent};

pub async fn on_disconnect(event: DisconnectEvent) -> anyhow::Result<()> {
    match event.cause {
        DisconnectCause::Deployment => {
            // client will reconnect automatically (1012)
        }
        DisconnectCause::Peer | DisconnectCause::Application => {
            // intentional close — clean up session state
        }
        _ => {
            tracing::warn!(
                connection_id = %event.connection_id,
                cause = ?event.cause,
                close_code = ?event.close_code,
                "unexpected disconnect"
            );
        }
    }
    Ok(())
}
```

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
