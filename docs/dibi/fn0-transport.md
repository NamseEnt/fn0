# fn0 Dibi Transport

The fn0 host provides the Dibi transport to WASM guests through the custom `fn0:dibi-transport/client` WIT interface.

## Boundary

```text
WASM guest
    -> WIT request(endpoint, frame)
    -> trusted fn0 host validates and injects project_id
    -> authenticated reusable QUIC connection
    -> Dibi server
```

The Forte SDK exposes only a raw frame wrapper. It does not choose tenants, understand database semantics, or open QUIC connections.

## Guest request rules

The guest frame uses Dibi protocol version 2. The tenant field in every tenant-scoped guest request must be the empty string. The host decodes and validates the complete frame before any network operation. A non-empty guest tenant, malformed frame, unsupported version, invalid flags, unknown opcode, oversized value, or trailing bytes returns `invalid-request` and is never sent to Dibi.

The guest may provide only the logical endpoint placeholder. With the default configuration it is `dibi://fn0-db.fn0.dev`. Any other endpoint returns `endpoint-not-allowed` without DNS or QUIC activity.

## Trusted host injection

The host obtains `project_id` from the current invocation runtime context. It does not read tenant identity from guest environment variables or request bodies. After validation, the host re-encodes the request with `tenant = project_id` and sends that frame over the authenticated worker connection.

The guest receives only `DIBI_URL=<placeholder_url>`. Target host, target port, server name, worker token, custom CA contents, internal address, and tenant override are not injected into the guest environment. When the bridge is configured, its placeholder takes precedence over a user-provided `DIBI_URL`.

## Worker configuration

The native worker supports:

| Environment variable | Meaning |
| --- | --- |
| `FN0_DIBI_TARGET_HOST` | Actual Dibi target host; absent disables the bridge |
| `FN0_DIBI_TARGET_PORT` | Actual target port; default `4433` |
| `FN0_DIBI_SERVER_NAME` | TLS server name; defaults to target host |
| `FN0_DIBI_PLACEHOLDER_URL` | Guest logical endpoint; default `dibi://fn0-db.fn0.dev` |
| `FN0_DIBI_WORKER_TOKEN` | Worker authentication secret; required when enabled |
| `FN0_DIBI_CA_CERT` | Optional PEM CA certificate |
| `FN0_DIBI_CA_CERT_BASE64` | Optional base64-encoded PEM CA certificate |

The worker token is kept in native host configuration, is never logged, is never placed in `DIBI_URL`, and is never exposed to the guest. A configured target without a worker token is a worker configuration error.

## Connection and replay behavior

One `Arc<DibiBridge>` is shared by the worker's WASM instances. It owns one reusable QUIC endpoint and caches a successfully authenticated connection. A new connection performs QUIC/TLS, `AUTH`, and only then request streams. A lost cached connection is replaced by a new connection with a new `AUTH` before the next request.

The bridge validates response magic, version 2, flags, request ID, payload length, frame size, response payload, and trailing bytes before returning the complete response frame to the guest. It does not automatically replay a request after bytes may have reached Dibi, including writes whose commit status is uncertain.

## Security model

The security boundary is:

```text
untrusted guest
    -> trusted fn0 worker
    -> authenticated QUIC worker connection
    -> tenant-scoped Dibi operation
```

The Dibi server trusts tenant values only from an authenticated worker connection. This Phase uses one platform worker secret and does not implement per-project credentials, IAM, ACL management, or a global admin interface.
