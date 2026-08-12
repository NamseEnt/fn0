# HTTP Body Streaming

Status: approved design, not yet implemented. Tracked by
[GitHub issue #108](https://github.com/NamseEnt/fn0/issues/108).

## Product contract

fn0 Cloud accepts HTTP request bodies up to 100 MB. This is a transport-size
limit, not a promise that the complete body fits in application memory.

Incoming request bodies must be exposed to Forte applications as a
single-consumer, backpressured stream. An application that processes and
discards bounded chunks must be able to handle a request near the 100 MB limit
without retaining the complete body in WASM memory.

The following limits apply independently:

| Limit | Value |
|---|---:|
| Request body | 100 MB |
| WASM memory | 128 MB |
| CPU time | 50 ms |
| Wall time | 15 seconds |

A request within the body-size limit can still fail because it exceeds memory,
CPU time, or wall time. In particular, buffering a large body or materializing a
large JSON value can exhaust the 128 MB WASM memory limit. Slow uploads can
exceed the 15-second wall-time limit.

Presigned object-storage URLs are the recommended path for durable file uploads.
They are not required: applications must remain able to stream large HTTP bodies
through compute for use cases such as hashing, incremental parsing,
transformation, and proxying.

## Enforcement

fn0-worker must enforce the 100 MB limit while reading the request body.

- A valid `Content-Length` above 100 MB is rejected with HTTP 413 before the
  application is invoked.
- `Content-Length` is not trusted as the only enforcement mechanism.
- A request without a length, or one using chunked transfer encoding, is stopped
  with HTTP 413 as soon as the received byte count crosses 100 MB.
- A client disconnect, size violation, or invocation timeout cancels body
  delivery and associated application work.

## Streaming requirements

Backpressure must remain intact across the entire path:

```text
Cloudflare
  -> OCI Network Load Balancer
  -> fn0-worker-proxy
  -> Hyper
  -> project worker
  -> WASI HTTP
  -> Forte handler
```

No layer may eagerly collect the complete request. Per-stream queues and
aggregate buffering must be bounded so concurrent large requests cannot turn
streaming into unbounded host memory use.

Forte may provide convenience operations for reading bytes, text, JSON, or form
data. Those operations buffer data and must make their memory cost and any
smaller buffering limit explicit. The 100 MB transport limit does not imply that
these convenience operations are safe for a 100 MB body.

Response bodies follow the same streaming principle. The documented unlimited
response-body size requires fn0-worker to forward response chunks with
backpressure instead of collecting the complete response before sending it.

## Current implementation gap

The host-side ingress path currently preserves the Hyper request-body stream
through the WASI HTTP boundary. The Forte SDK then collects the complete stream
into a `Vec<u8>` before route dispatch and codegen exposes it as `raw_body`.
Typed action deserialization can allocate an additional representation while
the original bytes remain alive.

The reused WASM instance has a 128 MB linear-memory ceiling and can serve
concurrent requests. The current buffering behavior therefore cannot safely
realize the published 100 MB request-body contract.

The response path also collects the complete guest response in fn0-worker before
returning it to Hyper, so the published unlimited response-body contract is not
currently end-to-end streaming.

## WebSocket relationship

This contract does not set the WebSocket message-size limit. An HTTP body is a
byte stream, while a WebSocket `on_message` callback receives one complete
message. WebSocket messages therefore require an independent atomic-message
limit. Applications should split large real-time data at the application
protocol level or use HTTP and presigned object-storage URLs where appropriate.

## Completion criteria

The implementation is complete when tests demonstrate all of the following:

- A streaming handler consumes a request near 100 MB with bounded guest memory.
- Concurrent large streams apply bounded buffering and backpressure.
- Oversized fixed-length and chunked requests receive HTTP 413.
- Disconnect and timeout cancellation propagate through every layer.
- A large response reaches the client without being fully collected by
  fn0-worker.
- Existing small pages, APIs, actions, hooks, queue tasks, and SSR behavior remain
  compatible through the request API migration.
