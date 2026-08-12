# Forte WebSocket Design

Status: implemented

## Architecture

`fn0-worker` owns each WebSocket from HTTP upgrade through final cleanup. Forte applications run
short-lived `on_connect`, `on_message`, and `on_disconnect` callbacks through the existing WASI
HTTP component path. A callback never owns a network socket.

The fn0-control Turso database is an ownership directory only. Each connection document maps a
connection ID to its project ID, owning worker process ID, and private endpoint. Message bodies
never pass through Turso. Commands targeting a remote owner use a direct encrypted QUIC stream.

Delivery is online-only and at-most-once. The platform does not retry an ambiguous send, replay
messages after reconnect, or provide durable delivery. Clients reconnect and recover authoritative
state through the application's normal HTTP API.

## Application surface

WebSocket modules are discovered recursively under `rs/src/websockets` and mounted below `/ws`.

| Source | Route |
| --- | --- |
| `src/websockets/index.rs` | `/ws` |
| `src/websockets/chat.rs` | `/ws/chat` |
| `src/websockets/rooms/[room_id].rs` | `/ws/rooms/:room_id` |

`on_connect` and `on_message` are required public async functions. `on_disconnect` is optional.
Dynamic routes use the existing `PathParams` convention.

```rust
use forte_sdk::anyhow::Result;
use forte_sdk::websocket::{
    ConnectDecision, ConnectEvent, DisconnectEvent, MessageEvent, WebSocketMessage,
};

pub async fn on_connect(event: ConnectEvent) -> Result<ConnectDecision> {
    let mut decision = ConnectDecision::accept();
    if let ConnectDecision::Accept { headers, .. } = &mut decision {
        headers.insert("x-app-connection", event.connection_id.as_str().parse()?);
    }
    Ok(decision)
}

pub async fn on_message(event: MessageEvent) -> Result<()> {
    let body = match event.message {
        forte_sdk::websocket::IncomingMessage::Text(text) => WebSocketMessage::text(text),
        forte_sdk::websocket::IncomingMessage::Binary(bytes) => WebSocketMessage::binary(bytes),
    };
    forte_sdk::websocket::send(&event.connection_id, body).await?;
    Ok(())
}

pub async fn on_disconnect(_event: DisconnectEvent) -> Result<()> {
    Ok(())
}
```

`ConnectEvent` contains the opaque connection ID, absolute request URI, public request headers,
client address when available, and requested subprotocols. `ConnectDecision` is either
`Accept { protocol, headers }` or `Reject { status, headers }`. Empty headers are available through
`HeaderMap::new()` and the convenience constructors.

The application may read every upgrade request header. A rejection may use any status other than
`101 Switching Protocols`. The application may add ordinary response headers, but
the generated router rejects `connection`, `upgrade`, `content-length`, `transfer-encoding`, every
`x-fn0-*` header, and WebSocket handshake headers controlled by the platform. A selected
subprotocol must have appeared in the client's request.

`on_connect` runs before the `101 Switching Protocols` response and fails closed. An
`on_message` or `on_disconnect` `Err` is recorded by the normal invocation telemetry and does not
close the connection. Applications call `disconnect` when they intend to close it.

## Commands

```rust
pub async fn send(
    connection_id: &ConnectionId,
    message: WebSocketMessage,
) -> Result<(), WebSocketSendError>;

pub async fn disconnect(
    connection_id: &ConnectionId,
) -> Result<(), WebSocketDisconnectError>;
```

`WebSocketMessage` is `Text(Body)` or `Binary(Body)`. The body is not required to declare a length.
The owner reads it only when that connection reaches its write turn and turns body chunks into one
fragmented WebSocket message. A future optimization may prefetch a queued body's chunks, but the
initial implementation intentionally does not.

Text is validated incrementally as UTF-8. Invalid text found before any frame is written returns
`InvalidText` and leaves the socket open. Invalid text found after partial output closes the socket
with `1007` because the fragmented message cannot be repaired.

`send` succeeds after the owning worker has written and flushed the complete message to its local
WebSocket transport. It does not mean that the peer application processed the message. Every write
error closes the target socket. Sends for one connection are serialized; there is no global order
across connections or independent calls.

`WebSocketSendError::delivery_state()` is `NotSent` when the owner definitely emitted no frame and
`Unknown` when delivery may have started. Callers receive the invocation's remaining wall-clock
deadline rather than a fresh 15 seconds. The SDK never retries automatically.

`disconnect` is idempotent for missing connections. It rejects later sends, lets already admitted
sends finish, writes close code `1000`, and waits until the peer close or a 10-second close timeout.
The timeout forces local cleanup and still completes the disconnect successfully. An invocation
deadline or worker-to-worker transport failure is returned to the caller.

There is no fire-and-forget send API and no immediate, non-graceful disconnect API.

## Worker-to-worker protocol

Every connection ID contains 32 random bytes encoded as an opaque versioned token. It does not
contain a worker address and is not signed.

Each worker process generates a globally unique random `worker_id` when it starts. A connection
document contains exactly the routing and authorization data needed by a sender:

```text
connection_id
project_id
worker_id
endpoint
```

There are no worker leases, connection TTLs, or directory heartbeats. fn0-control scans one bounded
page of connection documents on each control interval and groups them by `worker_id` and endpoint.
It asks each reachable worker which connection IDs it still owns. A confirmed missing connection or
a responding replacement process is deleted with a conditional `worker_id` check. A timeout,
network error, or unavailable worker is inconclusive and never causes deletion. The scan cursor is
advanced before probing so one dead endpoint cannot permanently stall reconciliation.

Direct commands use QUIC with TLS server authentication and a shared bearer credential inside the
private worker subnet. Each command uses one bidirectional stream:

1. The caller sends a bounded JSON command header.
2. The owner validates the bearer, worker ID, project, connection, deadline, and socket queue.
3. For a send, the owner waits until that socket's writer selects the command and then replies
   `READY`.
4. Only after `READY` does the caller stream the body.
5. The owner writes and flushes the WebSocket message, then replies `COMPLETE`.

The caller checks the directory's project ID before opening the QUIC stream, and the owner repeats
the project check against its local connection. Only a newly created QUIC connection has a short
two-second dial timeout; an already established connection uses the invocation's remaining
deadline.

The protocol performs no automatic retry. A failure before owner admission is `NotSent`; failures
after admission are conservatively `Unknown` unless the owner returns a definite result.

## Scheduling and limits

Callbacks for messages on the same connection may execute concurrently. Applications that need
ordering implement it in their own state model. The shared per-project admission controller covers
HTTP, WebSocket callbacks, queue tasks, cron tasks, and cross-project invocation.

| Limit | Value |
| --- | --- |
| Active invocations per project per worker | 32 |
| Waiting invocations per project per worker | 128 |
| Invocation admission wait | 15 seconds |
| Invocation execution time | 15 seconds |
| Waiting inbound messages per connection | 4 |
| Active outbound send per connection | 1 |
| Waiting outbound sends per connection | 4 |
| Inbound message | No fn0 size limit |
| Outbound message | No fn0 size limit |
| Connections per project per worker | 1,000, provisional |
| Connections per worker process | 10,000, provisional |

Inbound text is validated before callback dispatch and invalid text closes with `1007`. Message
size is governed by transport backpressure, available worker and application memory, and the
remaining 15-second invocation deadline. The current inbound callback surface materializes a
complete `String` or `Vec<u8>` before application dispatch, while outbound bodies remain streamed.

A full or expired inbound queue closes that connection with `1013`. A full outbound queue rejects
the send with `Backpressure` and closes the target connection with `1013`. Project connection
overflow rejects the upgrade with `429`; worker capacity overflow uses `503`. Both include
`Retry-After: 1`.

The owner sends a ping every 30 seconds and requires a pong within 15 seconds. Heartbeat failure
closes and cleans up the connection.

## Lifecycle

Deploying new code closes only that project's established connections with `1012 Service Restart`.
A worker image drain closes every connection owned by that process with `1012` and the worker agent
keeps the old container alive until all WebSocket close handshakes or their timeouts finish.

`on_disconnect` is best-effort and at-most-once for a connection observed by a live owner. Process
death may prevent it. Directory cleanup is attempted immediately and otherwise converges through
fn0-control's bounded reconciliation scan.

Clients must treat every disconnect as a reconnect signal. After reconnecting, they fetch current
state over HTTP before applying new transient WebSocket messages.
