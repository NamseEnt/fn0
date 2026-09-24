# Phase 6: Native service, QUIC protocol, and Rust client

## Scope

Phase 6 exposes the existing local dodb semantics through raw QUIC and a
first-party async Rust client. The implementation does not add HTTP/1.1,
HTTP/2, HTTP/3, REST, SQL, fn0 integration, authentication, tenant
authorization, replication, remote WAL, distributed transactions, or admin
checkpoint or backup RPCs.

## Dependency and service structure

The workspace dependency graph is:

```text
dodb-core
    -> dodb-service
    -> dodb-protocol
        -> dodb-client
        -> dodb-server -> dodb-storage
```

`dodb-core` remains the semantic source for opaque document keys, revisions,
revision states, transaction conditions, transaction mutations, and
transaction requests. `dodb-service` defines transport-independent request and
response values plus:

```rust
trait DodbService {
    fn execute(
        &self,
        tenant: TenantId,
        request: Request,
        budget: ExecutionBudget,
    ) -> ServiceFuture;
}
```

The trait returns a boxed future so a QUIC server, an in-memory test double,
and a future host adapter can share the same boundary. It carries a tenant for
routing only; authorization is not part of dodb.

The local server service uses a process-lifetime cache of opened shards with a
configurable maximum. The v1 resolver maps one tenant to one distinct
`ShardId` value, while retaining separate tenant and shard types so a later
routing implementation can change the mapping without changing the wire
protocol.

## Binary protocol

Every message uses this big-endian frame:

```text
magic          4 bytes: DODB
version        u16: 1
message type   u8: request or response
payload length u32
payload        payload length bytes
```

The codec checks the magic, version, message type, checked length arithmetic,
payload length, operation discriminants, count limits, boolean flags, response
types, and trailing bytes. Requests use explicit operation codes for Get, Put,
Delete, Query, Scan, and Transact. Transact retains opcode 8; opcodes 6 and 7
are reserved and rejected. Responses use the same operation code for
successful results and a separate status for structured application errors.

The public protocol has no generic Batch operation: condition-free multi-key
writes use Transact with an empty condition list. The protocol also has no
multi-read operation; independent Gets use independent QUIC streams. The
protocol does not expose B+Tree pages, WAL records, page LSNs,
superblock slots, checkpoint state, or storage-provider backup details.

The default network limits are:

- 68 MiB request frame and 68 MiB response frame, including the header;
- 64 MiB maximum value, matching the storage maximum;
- 3,990 bytes per defensive key component;
- 3,992 bytes maximum canonical encoded document key;
- 4,096 point keys;
- 256 conditions and 256 mutations;
- 4,096 Query or Scan rows per response;
- 4,096 bytes of human-readable error detail.

The decoder validates declared frame length before allocating the payload, and
all nested counts and byte lengths are checked against these limits. A caller
may lower the limits but cannot raise key or value limits above the storage
contract through `ProtocolLimits`. Canonical key length includes component
terminators and zero-byte escapes, so component limits alone are not the key
contract. Query primary-key cursors, Scan cursors, conditions, mutations,
structured conflict keys use the same canonical check.

The server supplies an `ExecutionBudget` derived from the response frame limit.
The local service passes it into bounded committed reads. Query and Scan
reserve aggregate response space before each value is materialized;
an incomplete result is rejected as structured `ResponseTooLarge` rather than
silently truncated. This keeps a legal single maximum-size value usable while
bounding one response before encoding.

## QUIC and TLS

Quinn 0.11 provides one reusable connection per connection owner. The client
exposes a `DodbConnection` owner and cheap tenant-scoped `DodbClient` handles;
one connection can serve many tenants concurrently. Each application
request opens one bidirectional stream, writes one request frame, finishes the
send side, and reads one response frame. Independent streams are concurrent;
the client does not serialize them behind a connection mutex and does not
create a connection per operation.

The server has separate configurable QUIC per-connection bidirectional-stream
limits and a global active-request budget. The QUIC limit is transport-level;
the global budget is enforced by waiting before accepting another application
stream, allowing QUIC flow control to provide bounded backpressure. Filling the
global budget never closes a connection or interrupts another stream. A full
`AsyncShard` coordinator is returned as the structured `Overloaded` application
category. QUIC transport failure remains distinct from an application error
response.

The server accepts a certificate chain and private key in DER or PEM form.
The client requires one or more trusted root certificates and passes the
configured server name to Quinn for certificate-name validation. Client
certificate authentication is not enabled in this phase.

## Tenant routing and lazy creation

The v1 physical policy is one tenant database file pair:

```text
TenantId -> resolver -> ShardId -> tenant-<tenant>-shard-<shard>.db/.wal
```

The resolver is explicit and does not make `TenantId` and `ShardId` the same
type or identity. The local service holds an `AsyncShard` for each opened
tenant and keeps the cache entry while the process lives. The cache lock covers
the existence check and open operation, so concurrent first mutations cannot
create duplicate shard coordinators.

Read-only Get, Query, and Scan requests against a tenant with no
database return an empty state and do not create data or WAL files. A
condition-only Transact reads the same missing revision-zero state and also
does not create files. The first mutation opens the paired files and creates
the local database. A service restart reopens the same files through the normal
WAL recovery path. The 128-bit database UUID is a deterministic,
domain-separated digest of the complete logical `(TenantId, ShardId)` pair;
it is not a truncated shard field and is intentionally a logical-pair identity
for this phase rather than a persisted per-incarnation identity.

## Transaction semantics

Put and Delete are single-key atomic mutations. Transact submits multiple
condition-free or conditional mutations as one atomic transaction and preserves
`RevisionEquals`, `Exists`, and `NotExists`, including the revision of a
missing key and the missing-revision ABA check.

The transaction primitive accepts a request with conditions and no mutations.
It rejects duplicate condition keys, gathers the unique keys with one
coordinator-serialized internal point-read operation, evaluates conditions in
input order, and returns the first deterministic structured Conflict. On
success it returns an outcome with no commit LSN and performs no mutation,
revision, WAL, or superblock-generation change. A request with neither
conditions nor mutations remains invalid. For a tenant with no database file,
the service supplies the logical `Missing(0)` state for this read-only path
without creating files. It never implements this operation as independently
observed ordinary Gets.

Ordinary Query and Scan retain ordered results and exclusive cursors. The
storage coordinator may use an internal point-read grouping while evaluating a
condition-only transaction, but that helper is not a public wire operation.
Independent Gets do not promise one cross-key read snapshot.

## Errors and unknown outcomes

The wire error categories are InvalidRequest, Overloaded, Conflict,
ResponseTooLarge, StorageFailure, Corruption, DurabilityFailure, Internal, and
UnsupportedProtocol. Conflict includes the key, expected condition, and
actual `Present(revision)` or `Missing(revision)` observed state without a
document value. The category is
machine-readable; human-readable detail is supplementary.

An application error response is distinct from a QUIC stream or connection
failure. Each application error also carries a stable mutation outcome field:
`NotApplicable`, `NotApplied`, or `Unknown`. InvalidRequest, Conflict, and
Overloaded are definite `NotApplied` outcomes. WAL durability failures and
physical WAL-group I/O failures are `Unknown`, because records may have become
durable even when the server reports an error. The Rust client converts an
uncertain server outcome into `UnknownMutationOutcome` while retaining the
structured application cause, and does not retry automatically. A read-only
request may be retried by a caller after a transport failure, but the client
does not add an automatic retry loop in this phase.

Normal server shutdown closes the endpoint, waits for established connections
to drain, and then awaits each local shard coordinator. `AsyncShard::close` and
`LocalTenantService::shutdown` therefore do not race a subsequent reopen of the
same database files.

## Observability

`dodb-server` exposes in-process counters for total and active connections,
active streams, requests by operation, request and response bytes, request
latency, protocol errors, transport errors, application errors, and overloaded
responses. No metrics HTTP endpoint is added.

## Validation coverage

The protocol tests cover all request and response variants, empty and non-UTF8
binary data, missing revision zero, near-limit values, malformed headers and
payloads, invalid magic/version/opcodes, absurd lengths, response type
mismatch, canonical key boundaries with zero-byte expansion, structured
Conflict, mutation outcome certainty, and structured server errors.

The loopback Quinn tests cover TLS connection setup, mutations, Get, Query,
Scan, condition-free and conditional Transact, condition-only transactions,
concurrent independent streams, same-tenant write ordering, tenant isolation,
missing-revision ABA, lazy file creation, disconnect/reconnect through server
restart, persisted WAL-backed state after reopen, global request backpressure,
and an injected WAL durability failure reported as an unknown mutation
outcome.

## Deferred features

The protocol intentionally leaves application authentication, tenant
authorization, idempotency, distributed routing, ownership transfer,
replication, secondary replicas, shard migration, distributed transactions,
service discovery, load balancing, and operational checkpoint/backup control
surfaces for later phases.
Dropping or releasing a tenant handle does not close the shared transport.
The connection owner explicitly calls `DodbConnection::close()` during
shutdown. The original tenant-bound `DodbClient::connect` constructor remains
available as a convenience API.
