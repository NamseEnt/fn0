# Dibi Protocol

Dibi protocol version 2 is a binary protocol carried directly over authenticated QUIC. The protocol is transport-independent at the codec layer. Each request uses one bidirectional QUIC stream and each connection may carry multiple request streams.

## Tenant model

`fn0 project_id == Dibi tenant`.

Every document operation is scoped to one tenant. A tenant is a non-empty UTF-8 string of at most 256 bytes. Tenant-scoped request payloads begin with exactly one tenant string. Batch items and transaction items do not contain tenant fields.

The tenant is authoritative only after the authenticated fn0 host injects the current project ID. A WASM guest cannot select or override it. The Dibi server accepts tenant values only from an authenticated worker connection.

## Transport and frame format

The server uses QUIC with TLS 1.3 and ALPN `dibi/2`. HTTP, HTTP/1, HTTP/2, HTTP/3, h3, gRPC, protobuf, JSON RPC, and WebSocket are not used.

Every frame has this fixed 20-byte header:

| Field | Size | Meaning |
| --- | ---: | --- |
| magic | 4 | ASCII `DIBI` |
| version | 1 | Protocol version `2` |
| opcode or status | 1 | Request opcode or response status |
| flags | 2 | Must be zero |
| request_id | 8 | Client-selected arbitrary `u64`; responses copy it |
| payload_len | 4 | Payload length in bytes |

The total frame is the header followed by exactly `payload_len` bytes. The maximum total frame size is 16 MiB. Invalid magic, version, flags, lengths, UTF-8, payloads, or trailing bytes are rejected.

Primitive values use big-endian encoding. `bytes` and `string` use a `u32` length followed by the value. Strings are UTF-8. The maximum tenant size is 256 bytes, maximum string size is 1 MiB, maximum document size is 16 MiB, maximum batch operation count is 10,000, and maximum query, scan, and admin page limit is 10,000.

## Opcodes and authentication

| Value | Opcode |
| ---: | --- |
| 0x01 | GET |
| 0x02 | PUT |
| 0x03 | DELETE |
| 0x04 | QUERY |
| 0x05 | SCAN |
| 0x06 | BATCH |
| 0x07 | EXECUTE_OPS |
| 0x10 | GET_WITH_VERSION |
| 0x11 | BATCH_GET_WITH_VERSION |
| 0x12 | TRANSACT_WRITE_ITEMS |
| 0x20 | ADMIN_SCAN |
| 0x21 | ADMIN_TRANSACT_WRITE_ITEMS |
| 0x40 | PING |
| 0x41 | STATUS |
| 0x42 | AUTH |

`AUTH` is a control operation with payload `bytes worker_token` and no tenant. The server requires `DIBI_WORKER_TOKEN` to be configured and non-empty. A new connection starts unauthenticated. `AUTH` compares the worker token in constant time and returns `OK` or `UNAUTHORIZED`. `PING` is allowed before authentication. `STATUS` and every tenant-scoped request return `UNAUTHORIZED` before successful `AUTH`.

The server returns `INVALID_REQUEST`, `NOT_FOUND`, `CONFLICT`, `UNAUTHORIZED`, or `INTERNAL_ERROR` as defined by the status codec. Normal document absence is an `OK` response with `found = false`.

## Tenant-scoped payloads

The first payload field for every tenant-scoped operation is `string tenant`.

```text
GET:
    string tenant
    string pk
    string sk
```

```text
TRANSACT_WRITE_ITEMS:
    string tenant
    u32 item_count
    item...
```

`BATCH`, `EXECUTE_OPS`, `BATCH_GET_WITH_VERSION`, `TRANSACT_WRITE_ITEMS`, and `ADMIN_TRANSACT_WRITE_ITEMS` contain one request-level tenant and never repeat it in items. A cross-tenant transaction cannot be represented by this wire format.

GET, PUT, DELETE, QUERY, SCAN, BATCH, EXECUTE_OPS, GET_WITH_VERSION, BATCH_GET_WITH_VERSION, TRANSACT_WRITE_ITEMS, ADMIN_SCAN, and ADMIN_TRANSACT_WRITE_ITEMS are tenant-scoped. PING, STATUS, and AUTH are tenant-independent control operations.

Query ordering is ascending `sk` within `(tenant, pk)` and its cursor is exclusive. Scan ordering is ascending `(pk, sk)` within one tenant and its cursor exposes only `(pk, sk)`. ADMIN_SCAN is also restricted to one tenant; there is no global admin scan.

## Conditional transactions

`TRANSACT_WRITE_ITEMS` and `ADMIN_TRANSACT_WRITE_ITEMS` use one request-level tenant. Each item is Create, Put with an expected version, or Delete with an expected version. The engine validates all conditions before one atomic RocksDB commit. If any condition fails, no writes are applied and conflict details are returned.

## TLS and connection behavior

The client verifies the Dibi server certificate and hostname during the TLS handshake. Public roots are supported, and an optional custom CA may be added. Certificate and hostname verification are never disabled.

After QUIC/TLS connection establishment, the client sends `AUTH` on a stream and publishes the connection for reuse only after receiving `OK`. A connection is authenticated as a worker, not bound to a tenant. Multiple projects can use the same authenticated connection; each request carries its host-injected tenant.

If a connection fails before any request bytes are sent, reconnecting is allowed. If request bytes have been sent and the outcome is uncertain, the client returns an error and does not replay the request automatically.

## Storage and format

The physical document key is `(tenant, pk, sk)` encoded as three components using UTF-8 bytes, `00 -> 00 FF`, and a `00 00` component terminator. Query, scan, and admin iterators stop at the requested tenant prefix. Commit outbox mutations retain the complete encoded three-component key; mutation ordering remains encoded-key ascending.

Dibi database format 2 introduces tenant-prefixed document keys. Format 1 databases require an explicit migration and are not opened automatically. No automatic format migration is performed.
