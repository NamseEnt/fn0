# Code Organization and Deduplication Plan

This document defines a structure-only refactoring plan for signy. The
goal is to make the codebase easier to navigate and reduce repeated setup,
storage, lifecycle, and error-handling code without changing runtime behavior.

The refactoring should be kept separate from feature work whenever possible.
Each phase must preserve the existing public module API through re-exports and
must be validated before the next phase starts.

## Goals and guardrails

### Goals

- Split large files by responsibility rather than by arbitrary line counts.
- Keep individual implementation files roughly between 300 and 800 lines.
- Remove repeated construction and lifecycle code.
- Keep log and trace domain types distinct while sharing infrastructure where
  their behavior is genuinely identical.
- Make the code easier to inspect selectively, reducing unnecessary context
  and token usage during future maintenance.

### Guardrails

- Do not change persistence formats, manifest semantics, query results, or
  crash-recovery behavior as part of this refactor.
- Do not introduce a single large generic abstraction for log and trace rows.
- Keep public functions and types stable through `mod.rs` re-exports where
  practical.
- Preserve tests near the implementation they verify, except for shared test
  fixtures and end-to-end tests.
- Keep the current uncommitted M5 work intact and avoid destructive rewrites.

## Current hotspots

| File | Approx. size | Main responsibilities |
| --- | ---: | --- |
| `src/part.rs` | 3,054 lines | Part format, Parquet I/O, indexes, metadata, reader, tombstones, tests |
| `src/object_storage.rs` | 2,786 lines | Manifests, CAS, upload, restore, eviction, recovery, path safety, tests |
| `src/query.rs` | 2,677 lines | HTTP handlers, limits, log execution, metric evaluation, restore, responses, tests |
| `src/logql.rs` | 1,915 lines | AST, parser, pipeline processing, field evaluation, tests |
| `src/journal.rs` | 1,260 lines | Writer, WAL replay, checkpoints, compaction, tests |
| `src/tempo.rs` | 721 lines | Tempo handlers, scans, tags, response formatting, tests |
| `src/main.rs` | 879 lines | Application wiring, startup recovery, routes, background workers, tests |

## Target module layout

### Part storage

Convert `src/part.rs` into a directory module:

```text
src/part/
├── mod.rs          # Public types, constants, and re-exports
├── format.rs       # Row conversion, flush, Parquet writing
├── reader.rs       # PartReader and row-group query execution
├── metadata.rs     # Metadata files, CRC validation, load/discovery
├── indexes.rs      # Bloom, stream, and exact-field indexes
├── tombstone.rs    # Merge tombstones, fsync, and part cleanup
└── tests.rs
```

`mod.rs` should retain the current public surface for `Part`, `Row`,
`PartReader`, `flush_rows`, `load_part`, and related helpers.

### Object storage

Convert `src/object_storage.rs` into:

```text
src/object_storage/
├── mod.rs          # ObjectStorage, RemoteCache, public API
├── catalog.rs      # Manifest types, validation, and CAS updates
├── object_io.rs    # Upload, delete, and immutable object operations
├── cache.rs        # Local catalog restore, body restore, and eviction
├── recovery.rs     # Startup reconciliation and interrupted merge recovery
├── paths.rs        # Safe path and symlink validation
└── tests.rs
```

Log and trace manifests should keep their typed public representations. Shared
CAS and object-I/O mechanics may use small internal traits or adapters.

### Query execution

Convert `src/query.rs` into:

```text
src/query/
├── mod.rs          # Public module facade and route re-exports
├── handlers.rs     # Loki HTTP handlers
├── limits.rs       # Range, scan, memory, timeout, and limit validation
├── execution.rs    # MemTable/part unified log execution
├── metrics.rs      # Metric evaluation and aggregation
├── restore.rs      # Remote part pinning and restore lifecycle
├── response.rs     # Loki JSON responses and query statistics
└── tests.rs
```

The HTTP layer should parse parameters and map errors, while execution modules
should not depend on Axum-specific response types.

### LogQL

Convert `src/logql.rs` into:

```text
src/logql/
├── mod.rs
├── ast.rs          # Query and metric expression types
├── parser.rs       # LogQL parsing
├── pipeline.rs     # JSON/logfmt and pipeline execution
├── field_filters.rs
└── tests.rs
```

### Journal and application wiring

```text
src/journal/
├── mod.rs
├── writer.rs       # Journal worker and append path
├── replay.rs       # WAL replay and corruption handling
├── checkpoint.rs
├── compaction.rs
└── tests.rs

src/app_state.rs   # AppState and shared runtime resources
src/router.rs      # Axum router construction
src/startup.rs     # Recovery, storage initialization, worker startup
src/tests/e2e.rs   # Process-level and startup recovery tests
```

`main.rs` should eventually contain only module declarations, tracing setup,
configuration loading, and the top-level server startup call.

### Tempo and merge

```text
src/tempo/
├── mod.rs
├── handlers.rs
├── scan.rs
├── tags.rs
├── response.rs
└── tests.rs

src/merge/
├── mod.rs
├── scheduler.rs
├── selection.rs
├── transaction.rs
└── tests.rs
```

## Deduplication opportunities

### 1. Shared `AppState` construction

`AppState` is manually initialized in production and many tests, including
config, three semaphores, registries, health flags, and optional remote state.

Add a constructor or test-only builder:

```rust
AppState::from_config(config, dependencies)
```

For tests, provide helpers such as `test_state`, `remote_test_state`, and
`trace_test_state`. Semaphore creation must happen in one place so test and
production limits cannot drift.

### 2. Log and trace remote lifecycle

The log query path and Tempo path both implement:

1. acquire a read lifecycle guard;
2. identify candidate parts;
3. detect missing bodies;
4. upgrade to a write guard;
5. restore missing bodies;
6. update remote health;
7. downgrade to a read guard.

Extract a shared lifecycle helper with typed log/trace adapters. The adapters
should provide candidate IDs, missing IDs, restore operations, and API-specific
error mapping. The lock and health-state logic should not be duplicated.

### 3. Log and trace object-store operations

Upload, delete, restore, manifest replacement, and CAS retry logic should share
internal helpers. Keep typed wrappers for log and trace descriptors so callers
cannot accidentally publish a trace part to the log manifest.

Recommended internal boundaries:

```rust
trait CatalogDescriptor {
    fn id(&self) -> &str;
    fn partition(&self) -> &str;
}

async fn update_manifest_with_cas(...)
async fn upload_immutable_files(...)
async fn delete_immutable_files(...)
```

Use this only inside the object-storage module; do not expose a broad generic
storage API to the rest of the application.

### 4. Flush publication and rollback

Log and trace publication in `flush.rs` currently have parallel error paths.
Extract one operation that publishes both outputs, marks remote health, and
cleans up all generated directories on failure. The helper must preserve the
current rollback ordering and error context.

### 5. Configuration parsing

Centralize environment parsing and validation in the config module:

- generic scalar parsing for integers;
- positive-integer validation for limits and semaphore sizes;
- optional duration parsing;
- common error formatting;
- default-value handling.

Keep field-specific validation explicit where different constraints apply.
Avoid a macro that hides the meaning of every configuration field.

### 6. Runtime error mapping

Query and Tempo currently map timeout, remote, limit, and internal errors in
parallel. Introduce an internal `RuntimeError` type with conversion methods to
HTTP status and public error text. The API modules should perform only the
final protocol conversion.

### 7. Test fixtures

Create a `#[cfg(test)]` test-support module for:

- temporary data directories;
- default and remote configurations;
- `AppState` construction;
- sample log rows and trace spans;
- in-memory object-store setup.

This removes repeated setup without adding test-only dependencies to runtime
modules.

## Recommended execution order

### Phase 0: Baseline

- Record current test count and verification commands.
- Add no-op module facades only where needed.
- Confirm that M5 changes are clean before moving code.

### Phase 1: Low-risk deduplication

- Extract `AppState` construction.
- Extract test fixtures.
- Centralize Config parsing.
- Extract common flush publication cleanup.

### Phase 2: Lifecycle deduplication

- Extract shared remote pin/restore lifecycle.
- Extract common runtime error mapping.
- Add focused tests for lock downgrade, restore timeout, and remote failure.

### Phase 3: File-level decomposition

- Split `part.rs`.
- Split `object_storage.rs`.
- Split `query.rs`.
- Preserve public APIs through re-exports.

### Phase 4: Remaining decomposition

- Split `logql.rs`, `journal.rs`, `tempo.rs`, and `merge.rs`.
- Move application wiring and end-to-end tests out of `main.rs`.

## Validation requirements

After every phase:

```text
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
git diff --check
```

Additional checks:

- test count must not decrease;
- manifest and WAL compatibility tests must remain green;
- object-store local and in-memory tests must both pass;
- query and Tempo responses must remain byte/schema compatible where tested;
- no new cross-domain dependency from trace code into log-specific types;
- no module should require reading unrelated storage, query, or HTTP code to
  understand its primary behavior.

## Completion criteria

- No implementation file exceeds approximately 1,000 lines without a written
  reason.
- Repeated `AppState`, remote restore, object-store deletion, and test fixture
  setup has one canonical implementation.
- Public APIs and persistence semantics are unchanged.
- All validation commands pass after the final move.
- The resulting module boundaries are documented in this file or in the
  corresponding module facade.
