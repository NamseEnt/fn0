# ADR 0005: Raw QUIC RPC for v1

Status: accepted

The native boundary is raw QUIC RPC. Each request uses one bidirectional QUIC
stream and each stream carries exactly one versioned binary request and one
versioned binary response. The application protocol has no HTTP, SQL, REST,
or storage-admin operations.

The Phase 6 dependency direction is:

```text
dodb-core
    -> dodb-service
    -> dodb-protocol
        -> dodb-client
        -> dodb-server -> dodb-storage
```

The protocol and service layers do not depend on storage implementation types.
The server resolves a trusted `TenantId` to a physical `ShardId` and keeps one
reusable `AsyncShard` per opened local tenant database. A missing database is
read as an empty database without creating files; the first mutation creates
the paired data and WAL files.

QUIC TLS uses explicit server certificate/private-key configuration and client
trust roots with server-name validation. Application authentication and tenant
authorization remain outside dodb. There is no v1 idempotency key, so a lost
response after a mutation leaves the client with an unknown outcome and the
client does not retry that mutation automatically.
