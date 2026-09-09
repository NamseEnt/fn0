# HTTP Body Streaming

Status: implemented. Tracked by
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
exceed the 15-second wall-time limit, and so can slow downloads: the response
stream shares the same deadline, so an unlimited response size does not mean
unlimited delivery time.

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

Cloudflare enforces its own request-body ceiling ahead of all of this, and on
the fn0 zone that ceiling is 104,857,600 bytes inclusive — the same number as
`MAX_REQUEST_BODY_SIZE`. Measured 2026-09-08 against `https://fn0.dev/`: a body
of exactly 104,857,600 bytes reaches the origin, and 104,857,601 is answered by
the edge with its own HTML 413 (`server: cloudflare`) after roughly 3 MB has
been sent.

So the published limit is reachable, and the worker's `Content-Length`
preflight will not normally be what a client sees: through the edge, an
oversized request is refused before the origin learns of it. The worker's own
enforcement still has to exist and stay correct — it is what covers requests
that do not arrive through that zone, and it is the only thing that counts
received bytes when no length is declared.

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

## Implementation notes

The worker enforces the transport limit while preserving the Hyper request-body
stream through the WASI HTTP boundary. The Forte SDK exposes the request as a
single-consumer `Body` stream. Generated page and API handlers receive it in
`ForteRequest::body`; the legacy `raw_body` slice is empty on those routes.
Typed action and hook deserialization uses an explicit 1 MiB convenience buffer,
so it cannot materialize an arbitrary 100 MB transport body.

The reused WASM instance has a 128 MB linear-memory ceiling and can serve
concurrent requests. The worker limits active body chunks to 64 KiB per stream
and an 8 MiB aggregate request buffer; application-level buffering remains
explicitly separate from this transport budget.

The aggregate budget accounts for chunks a request body is holding, so a stream
waiting on the network releases its share instead of occupying it. Holding it
across the wait would instead cap the whole worker process at
`AGGREGATE_REQUEST_BUFFER_SIZE / MAX_CONNECTION_BUFFER_SIZE` concurrent readers,
and slow uploads would then stall every other project's body on that process.
The budget does not cover Hyper's own per-connection read buffer, which is
capped per connection at `MAX_CONNECTION_BUFFER_SIZE` and scales with the
connection count rather than with this budget.

A request that crosses the transport limit is refused by received-byte count,
recorded on the request, and answered 413 regardless of what the guest does
next. The guest's own size refusals travel back as
`ErrorCode::HttpRequestBodySize` and reach the worker as a typed
`RequestBodyTooLarge`, which is what selects 413 over 502 — not the text of an
error message.

The worker forwards guest response chunks to Hyper without collecting the complete
response body.

### Where buffering stays deliberate

Some hops read a whole body on purpose. Each one is bounded by what produces it
rather than by the transport limit, and none of them sits on the path a large
upload takes:

- **SSR props.** The JS render path buffers the wasm response before calling
  into the JS runtime, because a retry rebuilds the JS request from those same
  bytes. The size is a page's props, not a request body.
- **Static page capture.** Storing a rendered page requires the rendered bytes.
- **Generated action, hook, queue-task, admin and WebSocket-event dispatch.**
  These deserialize a typed input, so they read through the SDK's 1 MiB
  convenience buffer and fail explicitly above it rather than materializing a
  transport-sized body.

Guest-to-host hijack requests are a separate direction and are not covered by
this design; their unbounded reads are tracked in
[GitHub issue #116](https://github.com/NamseEnt/fn0/issues/116).

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
