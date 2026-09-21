# Dibi Custom Protocol

Dibi Phase 2 exposes the synchronous DibiEngine through a custom binary application protocol carried directly over QUIC. The protocol is transport-independent at the codec layer, while the server uses one bidirectional QUIC stream for exactly one request and exactly one response.

Phase 2 provides encrypted QUIC transport and server identity verification, but no client authentication/authorization.

## Transport

The server uses QUIC with TLS 1.3. HTTP, HTTP/1, HTTP/2, HTTP/3, h3, gRPC, protobuf, JSON RPC, and WebSocket are not used.

A client opens one bidirectional stream for each operation, writes one request frame, finishes its send side, and reads one response frame. A connection may have many request streams active at the same time. The server applies a 30 second request read timeout and a 60 second request processing timeout. The stream must finish after the request frame; bytes after the frame are invalid.

## Frame format

All integers use big-endian byte order.

Every frame has this fixed 20 byte header:

| Field | Size | Meaning |
| --- | ---: | --- |
| magic | 4 | ASCII DIBI, bytes 44 49 42 49 |
| version | 1 | Protocol version 1 |
| opcode or status | 1 | Request opcode or response status |
| flags | 2 | Must be 0 in Phase 2 |
| request_id | 8 | Client-selected arbitrary u64; responses copy it |
| payload_len | 4 | Payload length in bytes |

The total frame is the header followed by exactly payload_len bytes. The maximum total frame size is 16 MiB. A frame whose declared length is too large, overflows, is truncated, or has trailing stream bytes is invalid.

Primitive values are encoded as follows:

- u8, u16, u32, u64, and i64 use their big-endian representation.
- bool is 0 for false and 1 for true. Other values are invalid.
- bytes is a u32 byte length followed by that many bytes.
- string is a u32 byte length followed by UTF-8 bytes. Invalid UTF-8 is invalid.
- Optional values are a one-byte presence marker: 0 means absent and 1 means the following value is present. Other markers are invalid.

The maximum string size is 1 MiB, maximum document size is 16 MiB, maximum batch operation count is 10,000, and maximum query, scan, and admin page limit is 10,000. Lengths are checked before allocation.

## Opcodes

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

Unknown opcodes are invalid requests.

## Statuses

| Value | Status |
| ---: | --- |
| 0x00 | OK |
| 0x01 | INVALID_REQUEST |
| 0x02 | NOT_FOUND |
| 0x03 | CONFLICT |
| 0x06 | UNAUTHORIZED |
| 0x07 | INTERNAL_ERROR |

Missing documents use an OK response with found = false. NOT_FOUND is not used for normal document absence. CONFLICT is used for conditional write conflicts. Server failures use a short generic error message and do not expose filesystem paths, backtraces, or RocksDB error details.

## Key and document operations

GET request payload is string pk, string sk. Its response is bool found, followed by bytes data when found.

PUT request payload is string pk, string sk, bytes data. Its response is u64 commit_id.

DELETE request payload is string pk, string sk. Its response is u64 commit_id. Deleting a missing key follows engine semantics and succeeds.

QUERY request payload is string pk, optional string after_sk, and u32 limit. Its response is u32 item_count, followed by item_count repetitions of string sk and bytes data. Sort order is ascending sk; after_sk is exclusive.

SCAN request payload is an optional cursor consisting of string pk and string sk, followed by u32 limit. Its response is u32 item_count, followed by item_count repetitions of string pk, string sk, and bytes data. Sort order is ascending (pk, sk); the cursor is exclusive.

BATCH request payload is u32 operation_count, followed by operations. A Put operation has type 0x01, string pk, string sk, and bytes data. A Delete operation has type 0x02, string pk, and string sk. The server performs one application_write_batch call. The response is an optional u64 commit_id; an empty batch has no commit ID.

EXECUTE_OPS request payload is u32 operation_count, followed by operations in input order:

- 0x01 Get: string pk, string sk
- 0x02 Query: string pk, optional string after_sk, u32 limit
- 0x03 Put: string pk, string sk, bytes data
- 0x04 Delete: string pk, string sk

The response contains u32 result_count, equal to the operation count. Results are 0x01 Done, 0x02 Single followed by bool found and optional data when found, or 0x03 Multiple followed by a query item list. Operations execute sequentially without an implicit transaction.

GET_WITH_VERSION has the same request as GET. Its response is bool found, followed when found by i64 version and bytes data.

BATCH_GET_WITH_VERSION has a u32 key_count followed by key_count repetitions of string pk and string sk. Its response contains the same number of items in input order. Each item contains bool found, followed when found by i64 version and bytes data. The operation reduces network round trips and does not promise a cross-key snapshot.

## Optimistic transaction items

TRANSACT_WRITE_ITEMS and ADMIN_TRANSACT_WRITE_ITEMS use the same request operation format. The request has u32 operation_count, followed by:

- 0x01 Create: string pk, string sk, bytes data
- 0x02 Put: string pk, string sk, i64 expected_version, bytes data
- 0x03 Delete: string pk, string sk, i64 expected_version

The engine conditional write API is used directly. An applied response is OK with an optional u64 commit_id. A conflict response is CONFLICT with u32 conflict_count, followed by string pk, string sk, optional i64 expected_version, and optional i64 actual_version for each conflict. Duplicate keys are invalid input and the request does not reach the engine.

ADMIN_SCAN request payload is an optional (string pk, string sk) cursor, u32 limit, and optional string pk_prefix. Its response is u32 count, followed by string pk, string sk, i64 version, and bytes data for each item, followed by an optional (string pk, string sk) next cursor. The server calls DibiEngine::admin_scan and does not reimplement scan semantics.

Dibi uses optimistic transaction items based on document versions. There is no server-side transaction session. GET_WITH_VERSION reads one document's current version. The client then sends all intended changes in one TRANSACT_WRITE_ITEMS request, and the server calls DibiEngine::conditional_write_batch once. The engine validates every condition before one atomic RocksDB commit; if any condition fails, zero writes are applied and all conflict details are returned.

An example fn0-doc-db::trx flow is:

```text
GET_WITH_VERSION A -> version 3
GET_WITH_VERSION B -> version 7

client code computes changes

TRANSACT_WRITE_ITEMS:
    PUT A expected_version=3
    DELETE B expected_version=7
    CREATE C

server:
    conditional_write_batch()

success:
    one atomic RocksDB commit

conflict:
    zero writes
    expected/actual versions returned
```

The server does not retry. Retry and closure re-execution are responsibilities of the future fn0-doc-db::trx client layer. This Phase 2 does not implement that client or explicit fn0-doc-db transaction compatibility. Client-side pending writes and read-your-own-writes overlay remain future client-layer behavior.

## TLS and authentication

The server requires PEM certificate and private key files supplied with --cert and --key. Production startup does not generate a certificate automatically. The client must validate the server certificate during the TLS handshake. Tests use a generated self-signed certificate and explicitly trust that certificate.

Phase 2 provides encrypted QUIC transport and server identity verification, but no client authentication/authorization.

## Server and malformed input behavior

The server accepts --data-dir, --listen, --cert, and --key. It supports multiple QUIC connections, multiple concurrent bidirectional streams per connection, and blocking-pool execution for all DibiEngine operations.

Malformed frames are rejected without panicking or allocating from unchecked remote lengths. Invalid streams are isolated from other streams and connections. A response is emitted with INVALID_REQUEST when a request ID can be recovered; otherwise the stream is closed without a response.
