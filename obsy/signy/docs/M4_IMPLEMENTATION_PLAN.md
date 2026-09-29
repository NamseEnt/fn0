# M4 implementation plan

This plan turns the M4 outcome in `docs/ARCHITECTURE.md` into an executable
implementation and verification sequence.

## Scope

M4 delivers:

- OTLP gRPC `TraceService/Export` ingestion.
- Durable trace storage with restart recovery and object-store restoration.
- Trace-ID normalization and row-group Bloom pruning.
- Tempo-compatible trace-by-ID and bounded search APIs.
- Grafana Tempo datasource smoke coverage.

The following remain outside M4: OTLP HTTP ingestion, trace metrics, tail
sampling, global deduplication, and compaction tuning/load targets. Existing
at-least-once WAL semantics remain in effect, so crash replay may duplicate a
span until a later deduplication milestone.

## Design decisions

1. Traces use typed domain objects and trace-specific immutable parts instead
   of being encoded as Loki log rows. This keeps LogQL and log-part schemas
   unchanged while allowing the full OTLP span shape to be preserved.
2. The existing WAL is extended with a versioned log/trace record envelope.
   Legacy Loki records remain replayable.
3. Trace IDs are validated as 16-byte identifiers and stored as lowercase
   32-character hexadecimal strings. Span IDs are validated as 8-byte
   identifiers and stored as lowercase 16-character hexadecimal strings.
4. OTLP gRPC runs on a configurable listener (`SIGNY_OTLP_GRPC_ADDR`,
   default `0.0.0.0:4317`). The existing HTTP listener serves Loki and Tempo
   APIs.
5. Trace parts have their own catalog and cache namespace, while upload,
   conditional manifest replacement, cache restoration, and lifecycle locking
   reuse the existing object-store contracts.

## Implementation sequence

### 1. Protocol and domain layer

- Add compatible `tonic` and `opentelemetry-proto` dependencies.
- Add trace normalization types for resource spans, instrumentation scopes,
  spans, attributes, events, links, status, kind, flags, and trace state.
- Reject invalid IDs, all-zero IDs, invalid timestamps, and oversized requests
  before journaling.
- Add `TraceMemTable` with snapshot/flush semantics parallel to `MemTable`.

### 2. Durability and lifecycle

- Add a versioned WAL envelope that distinguishes Loki and OTLP records.
- Preserve replay of existing raw Loki `PushRequest` records.
- Include trace snapshots in checkpoint/flush handling.
- Install log and trace generations under the operation lock so queries cannot
  observe a partially committed visibility transition.
- Return OTLP success only after the WAL record has been durably written.

### 3. Trace parts and indexing

- Write trace spans to trace-specific Parquet parts with scalar columns for
  IDs, timestamps, service/name/kind/status and serialized columns for nested
  OTLP values.
- Write a row-group trace-ID Bloom sidecar and metadata with integrity checks.
- Add a trace registry and catalog-only reader mode for evicted Parquet bodies.
- Query memory and flushed parts together, scan exact IDs after Bloom pruning,
  and enforce span/scan/runtime bounds.

### 4. Object storage and recovery

- Generalize the object-store helpers to support the trace dataset without
  changing the existing log manifest format.
- Publish trace parts before exposing them in the trace catalog.
- Restore only trace parts whose catalog Bloom can match the requested ID.
- Include trace bodies in cache eviction while retaining metadata and Bloom
  sidecars locally.
- Test interrupted upload, restart recovery, cache eviction, and remote
  restoration for traces.

### 5. Tempo API

Add the minimum Grafana-compatible HTTP surface:

- `GET /api/traces/{trace_id}`
- `GET /api/search`
- `GET /api/search/tags`
- `GET /api/search/tag/{tag}/values`

Trace-by-ID returns a Tempo-compatible JSON shape and uses `400` for malformed
IDs and `404` for an absent trace. Search initially supports bounded time,
tag, duration, and limit filters; unsupported query syntax returns a clear
client error. Trace metrics and advanced Tempo APIs are deferred.

### 6. Verification and documentation

- Add unit tests for normalization, WAL compatibility, part round trips, Bloom
  pruning, and Tempo JSON fixtures.
- Add end-to-end tests for ingest, flush, restart, object-store restore, and
  Grafana-style trace lookup/search.
- Keep all existing Loki tests passing.
- Run formatting, Clippy with warnings denied, all targets, and `git diff
  --check` before marking M4 complete.

## Acceptance checklist

- [x] OTLP gRPC accepts valid ResourceSpans/ScopeSpans and preserves span data.
- [x] Invalid IDs/timestamps/requests are rejected without partial ingestion.
- [x] OTLP acknowledgement follows durable WAL append.
- [x] Legacy Loki WAL records replay after the WAL format extension.
- [x] Trace data survives flush and process restart.
- [x] Trace-ID Bloom pruning never produces false negatives.
- [x] Lookup combines memtable and immutable parts across partitions.
- [x] Evicted trace bodies restore from the object store on demand.
- [x] Tempo trace-by-ID and bounded search APIs match the implemented compatibility shape.
- [x] Existing Loki ingest, LogQL, object-store, and readiness tests remain green.
- [ ] A fresh-context review reports no blocking findings.
- [ ] The complete validation set passes after the final review fix.
