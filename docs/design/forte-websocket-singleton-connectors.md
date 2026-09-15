# Forte Persistent Outbound WebSocket Design

Status: implemented for the initial declarative lifecycle

This document defines project-scoped outbound WebSockets that Forte keeps connected without an
application invocation calling `connect`. It extends the physical outbound transport in
[Forte WebSocket Design](./forte-websockets.md).

The implemented scope is deployment-time discovery, active-version registration, per-connector
ownership claims, lease renewal with local fencing, reconnect reconciliation, callback delivery,
a named send API, the outbound destination policy, and compute egress accounting. Runtime pause,
runtime resume, and public singleton status APIs are intentionally unsupported.

## Application configuration

Persistent WebSockets are declared as Rust modules below `rs/src/ws_singleton`. For example,
`rs/src/ws_singleton/market_feed.rs` declares `singleton_id = "market_feed"` and the internal route
`/ws_singleton/market_feed`. Nested files such as `feeds/us/market.rs` produce
`singleton_id = "feeds/us/market"`. Dynamic path segments are rejected.

```rust
use forte_sdk::anyhow::Result;
use forte_sdk::websocket::{
    DisconnectEvent, MessageEvent, SingletonConnectEvent, SingletonConnectionOptions,
};

pub async fn connect() -> Result<SingletonConnectionOptions> {
    Ok(SingletonConnectionOptions::new(
        "wss://stream.example.com/market",
    ))
}

pub async fn on_connect(_event: SingletonConnectEvent) -> Result<()> {
    Ok(())
}

pub async fn on_message(_event: MessageEvent) -> Result<()> {
    Ok(())
}

pub async fn on_disconnect(_event: DisconnectEvent) -> Result<()> {
    Ok(())
}
```

`connect` and `on_message` are required. `on_connect` and `on_disconnect` are optional. `connect`
returns the URL, handshake headers, and requested subprotocols. Code generation writes the derived
singleton IDs and callback routes to `.forte/ws_singletons.json`; this file is an internal deploy
artifact and is not application configuration. A project may declare at most 1,000 singletons.

## Identity

The logical identity is:

```text
(project_id, singleton_id)
```

The control plane has one current, unexpired reservation for that identity under normal operation.
One worker normally owns the current physical connection. A physical connection uses the existing
opaque `connection_id`. Control uses an internal, short-lived `claim_token` to fence assignment
attempts and late status updates. The token is not application-visible and does not change the
logical identity. The same claim is idempotent on a worker; a different singleton ID creates a
different connection even when URL and receive path are equal. A version change, disconnect, crash,
activation failure, or declaration removal preserves the reservation until its stored deadline;
another claim is allowed only at or after that deadline.

The database reservation is the authority, not the number of sockets that an upstream server may
temporarily observe. If an old worker is paused or partitioned, its socket can remain open until the
local safety deadline, or it can close earlier while the reservation still blocks replacement. The
platform prevents stale database writes and stale local sends, but it cannot forcibly close a socket
across a network partition.

## Deployment state

Control stores every deployment's declarations under its `code_version`. Only declarations for the
project's active code version are eligible for connection. This makes the receive path a deployment
artifact instead of permanent configuration.

After activation, `deploy_artifact_prune` also prunes these declaration documents, including empty
declarations. It queries one project's configurations in pages and retains the active code version,
every newer version that may still be uploading or awaiting activation, and the newest two versions
below active. Older versions are deleted; a failed deployment becomes eligible once later active
versions move it outside this retained history. If no later deployment activates, its configuration
remains retained.

Each deletion re-reads the manifest and the exact `(project_id, code_version)` document in one
transaction. Cleanup stops if the project disappears, the active version changes, or activation is
no longer `active`. Already-deleted documents are harmless on retry. Cleanup marks stale singleton
runtime records unavailable and preserves their reservation until expiry; it deletes them only at or
after the stored deadline.

When a worker adopts a new project code version, it closes every WebSocket for that project with
`1012 Service Restart`. Control does not reconnect a persistent WebSocket from the previous
deployment before its reservation expires. It uses the declaration shipped with the active
deployment, so a renamed handler either produces the new path or fails deployment validation rather
than reconnecting to a stale path. The old connection can close before the reservation expires, so
sends can fail during the remaining reservation period.

Deployments use the platform's rolling consistency model. HTTP requests, WebSocket callbacks, and
queue work may briefly run on old and new code during rollout. Applications that cannot tolerate
that overlap must quiesce traffic and drain asynchronous work. Persistent WebSockets add no stronger
cross-version guarantee; closing them prevents an old connection from surviving indefinitely.

## Control and worker responsibilities

Control is the only caller of the worker's internal operation:

```text
connect_singleton(project_id, singleton_id, claim_token, url, receive_path) -> connection_id
```

Control reads the active deployment declaration from its database. The worker never reads that
record and reuses the existing URL-based outbound WebSocket transport.

Control serializes assignment for one `(project_id, singleton_id)`. A committed, unexpired
reservation prevents another worker from receiving the same assignment, including after a version
change or declaration removal. Network connection establishment occurs after the database
transaction. The reservation is stored and renewed per connector; it is not a shared worker-session
lease.

## Connection lease

The worker reports the following internal state to control:

```text
project_id
singleton_id
claim_token
connection_id
status = heartbeat | disconnected
```

The existing `connection_id` lets control ignore a late heartbeat or disconnect from a replaced
physical connection.

The initial reservation is 60 seconds. A worker renews substantially earlier and uses a local safety
deadline shorter than the control reservation. If it cannot renew by that deadline, it stops
admitting sends and callback dispatch, then closes the socket. Control assigns a replacement only at
or after the stored reservation expires. A crashed worker cannot send `disconnected`; reservation
expiry is its recovery path. The worker uses a 10-second heartbeat interval and a 30-second local
safety deadline.

The worker keeps the deadline in a lease guard that every connection path reads directly:

- send admission, before a command is queued;
- the writer, before it starts a queued send and again before every outgoing data frame;
- inbound message dispatch, and the worker thread immediately before a message or `on_connect`
  callback starts;
- activation, before `on_connect` is invoked.

The renewal task extends the guard and closes the socket, but none of these checks wait for it.
A process that was paused past the deadline refuses traffic as soon as it resumes, even if the
renewal task has not been scheduled yet. An accepted renewal moves the deadline to 30 seconds after
the renewal request was sent, not after its answer arrived, because control stamps the lease no
earlier than that. A renewal whose answer is lost counts as a failure even though control may have
applied it. A rejected renewal revokes the guard at once. A frame already written to the operating
system before the deadline cannot be recalled; such a send reports delivery `Unknown`.

## Reconciliation

The previous implementation walked every active project on every control tick, loaded each
singleton runtime with a separate database request until it found one reconnect candidate, and
then enqueued deployment activation for the whole project. Its work grew with the total number of
projects and declarations, produced N+1 runtime reads, and retried healthy declarations together
with the failed declaration. One project-level error also stopped the remaining scan.

The control tick scans at most 64 projects and 256 declarations per invocation. It stores a
`(project_id, singleton_id)` cursor, reads each project's runtime records in one query, and enqueues
only missing or expired singletons. It marks old-version and undeclared records unavailable while
preserving their reservations. It enqueues one targeted reconcile task per
candidate; there is no batch claim transaction or deployment-configurable claim count in the current
implementation. For each targeted task it:

1. skips an unexpired current connection, unavailable reservation, or claim;
2. claims the singleton in a short database transaction;
3. calls `connect_singleton` on the worker executing the targeted queue task;
4. records the returned `connection_id` only when the claim token still matches;
5. retires its own failed claim without shortening or deleting the reservation.

The tick retires runtime state that no longer exists in the active deployment. A worker that still
owns the removed state can no longer renew it and self-fences at the local safety deadline; the
reservation remains until its stored deadline.

## Delivery behavior

Message callbacks use the declared outbound handler and retain the existing online-only,
at-most-once behavior. The platform does not replay messages received while disconnected and does
not retry ambiguous sends. `send` and `disconnect` continue to address the physical
`connection_id`.

## Named send

Code generation emits `crate::ws_singleton::<module path>::send(message)` for every singleton
module, with the singleton ID fixed in the generated function. The guest calls the WebSocket hijack
at `/send-singleton`; the worker takes the project from the calling invocation, not from the
request, so a project can only address its own singletons.

The worker asks the internal `websocket_singleton_resolve` control action for the current
connection. Control answers with a connection only when the project's active deployment is
active, the runtime record belongs to that code version, activation has completed, and the
reservation has not expired. The response includes the connection ID, reservation deadline, and
control lookup time. The worker derives a relative lifetime from those timestamps, subtracts its
safety margin, and caches the connection until that local deadline. Concurrent misses for the same
logical key share one lookup, and each send retains its own deadline. The worker then performs one
ordinary `send` to that connection, locally or through QUIC. If the connection disappears before
the write, the result is `ConnectionNotFound` and only the matching cache entry is removed. The
worker does not resolve again or send to a replacement for that message: the replacement's
protocol setup in `on_connect` may not have completed, and the application cannot tell which
physical socket received the message.

## Outbound destination policy

The singleton URL is application-controlled and the worker opens the socket, so the dial goes
through the same `OutboundDialer` as guest HTTP requests and JavaScript `fetch`. It resolves the
host, removes every address outside the public internet, and connects to an approved
`SocketAddr`. TLS SNI and the HTTP `Host` header keep the original hostname. Each reconnect
resolves again. A refused destination fails the claim like any other dial failure, so control
retries it on later ticks. Self-hosted workers can allow private destinations with
`FN0_ALLOW_PRIVATE_OUTBOUND_DESTINATIONS=true`; see
[Limits & Quotas](../fn0/limits.md#outbound-destinations).

## Egress accounting

Message payloads a singleton sends are charged to the project's monthly compute egress before each
frame is written, on the worker that owns the socket. When control refuses credit because the quota
is exhausted or not configured, the send fails with `EgressQuotaExceeded` and the connection closes
with code `1008`; when control cannot be reached, the send fails as a transport error and the
connection closes. Received messages are not charged. The quota model is described in
[Limits & Quotas](../fn0/limits.md#network).

## Failure behavior

| Event | Result |
| --- | --- |
| Duplicate control tick | One database claim wins; the other skips the singleton |
| Duplicate worker request | The worker keeps the current singleton connection |
| Upstream dial failure | The reservation remains until expiry; the failed attempt is unavailable and a later tick retries after the boundary |
| Worker crash | The socket disappears and control reassigns at or after reservation expiry |
| Worker cannot renew | The worker self-closes before control can reassign |
| Worker resumes after a pause past the deadline | Sends, frames, and callbacks are refused before the renewal task runs |
| Renewal answer lost | Treated as a failure; the worker fences 30 seconds after its last confirmed request |
| Named send races an owner change | `ConnectionNotFound`; no retry on the replacement |
| DNS changes to a private address | The next dial is refused |
| Egress quota exhausted | Sends fail and the connection closes with `1008` |
| Late disconnect | Control ignores it when `connection_id` is no longer current |
| Project deployment | Workers close project sockets; control uses only the new deployment declaration |
| Declaration removed | Control stops reconciling it and preserves the reservation until expiry; the previous owner may close earlier and sends fail during the remaining reservation |

## Unsupported lifecycle controls

Forte does not provide runtime pause, resume, or public status operations for persistent outbound
WebSocket singletons. The deployed module declaration is the desired state: while the active
deployment contains the declaration, fn0 reconciles the connection automatically. Stopping or
restoring a singleton requires changing the deployed declarations and deploying that change.

The internal `websocket_singleton_status` action is only the worker heartbeat and disconnect-report
endpoint. It is not an application read API or a dashboard contract and must not be exposed as one.

## Open follow-ups

The message contract remains online-only and at-most-once. It has no explicit fn0 per-message byte
cap; actual processing is bounded by transport backpressure, available memory, queue limits, the
invocation deadline described in [Forte WebSocket Design](./forte-websockets.md), and the project's
monthly egress quota.

Verification is by unit and component tests with virtual time, in-memory control databases, and
loopback sockets. These tests do not observe whether a kernel keeps a TCP connection open after a
worker fences it, and they do not exercise real DNS resolvers or remote TLS servers.
