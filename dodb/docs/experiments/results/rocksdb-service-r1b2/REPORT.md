# R1b-2 shared transaction contract report

## Result

Implementation SHA `558a9283c1a9f5989130b720e8b71047744738f0` passed the same
transaction contract body on the native `LocalTenantService` and experimental
`RocksDbService`, locally and on OCI. Only service construction and each
service's temporary data path differ. Both execute through `DodbService` with
the same requests, independent expected values, and assertions. The integration
test is `dodb/crates/dodb-rocksdb/tests/transaction_contract.rs`; its shared
body is `dodb/crates/dodb-rocksdb/tests/common/transaction_contract.rs`.

The test uses separate file-backed temporary directories for each service.
On OCI, `TMPDIR` points inside this run's ZFS directory, so both backends open
and persist real tenant databases on ZFS during the test. Normal service
shutdown and reopening the same paths are covered. This does not test process
termination or crash recovery.

## Contract coverage

- Unconditional Put and Delete, including deleting a never-created key.
- Transaction widths 1, 4, and 8; mixed Put/Delete; all changed keys receive
  one shared nonzero revision, and each revision is ordered after its earlier
  state where checked.
- Successful and failed Exists, NotExists, and RevisionEquals predicates;
  same-key condition plus mutation; multiple predicates with one stale
  predicate; unmodified condition keys; condition-only pass/fail; invalid
  empty requests; duplicate condition and mutation keys; binary keys and values.
- Missing, created, updated, deleted, recreated, and missing-again states.
  Tokens are obtained from and reused with their own backend. A missing token
  from before create/delete fails, while the current missing token succeeds.
  The same checks survive shutdown/reopen, and a post-reopen mutation advances
  beyond the old token.
- Two requests start at an explicit barrier with the same revision; exactly
  one conditional update succeeds and the final value matches that winner.
  A condition-only key races a separate Put while another key is conditionally
  written. Conditional writes also race unconditional Put and Delete. The
  assertions admit either valid serialized order and do not assume a winner.
- A barrier-coordinated Query concurrent with a two-key transaction observes
  either no rows or both complete rows from the request-scoped snapshot.
- Four tenants concurrently use the same document key with NotExists
  transactions. Their independent values and successful outcomes survive
  normal shutdown/reopen.

Expected document values and tenant-specific outcomes are held independently in
the shared test, including a `BTreeMap` of the four tenant/value expectations.
Service-returned revisions are used only as opaque tokens for that same
service's later conditions and for logical revision ordering; native and
RocksDB numeric revision values are never compared across backends. Concurrent
results are judged against allowed serial histories.

The current services serialize writes per tenant. The synchronized requests
exercise the contract on that implementation, but do not prove correctness if
that serialization is removed or if a condition key is no longer protected
between validation and commit. No sleep or large stress loop is used.

## Verification evidence

| Environment | Source SHA | RocksDB | Command | Result |
| --- | --- | --- | --- | --- |
| Local macOS 26.4 ARM64; Rust 1.97.1 | `558a9283c1a9f5989130b720e8b71047744738f0` | 11.8.1, source `abeebd9630f11bd08c28b7bd43c7bdfc62050654` | `cargo fmt -p dodb-rocksdb -- --check`; `env DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo check --locked --offline -p dodb-rocksdb --all-targets`; same environment with `cargo test --locked --offline -p dodb-rocksdb --all-targets` | Passed formatting and all-target check. Tests passed: 1 unit, 2 read-contract integrations, and 1 shared transaction-contract integration. |
| OCI A1, Linux aarch64, native Release; Rust 1.98.1 | `558a9283c1a9f5989130b720e8b71047744738f0` | 11.8.1, source `abeebd9630f11bd08c28b7bd43c7bdfc62050654` | `env DODB_ROCKSDB_DIR=/bench/zfs/db/product-build/rocksdb-source CARGO_TARGET_DIR=/bench/zfs/db/dodb-rocksdb-r1b2-558a9283c1a9/target TMPDIR=/bench/zfs/db/dodb-rocksdb-r1b2-558a9283c1a9/tmp cargo test --locked --release -p dodb-rocksdb --test transaction_contract -- --nocapture` | Passed: 1 shared integration test, 0 failed. Checkout clean and detached at the exact source SHA. Test executable is ELF AArch64; SHA-256 `ae445d85b24d47b6a1a228068b0faf597e706e823c5506d6850f26a54c0aba8f`. `TMPDIR` and DB paths resolve to ZFS `dodbbench/db`. |

OCI run directory: `/bench/zfs/db/dodb-rocksdb-r1b2-558a9283c1a9`. The test
output is preserved at `transaction-contract.log`. No OCI source files were
modified. Existing experiment directories and processes were left intact.

## Failed attempts and correction

An early local test run exposed a test-sequence error: after the ABA sequence
recreated a key, the test incorrectly expected that key to be missing at
reopen. The observed state was `Present(value="recreated", revision=90)`.
The sequence was corrected to delete the recreated key, retain that new
missing-state token, and test it across reopen. No backend implementation or
contract assertion was weakened; the final shared test passes on both
backends. An initial local compile without `DODB_ROCKSDB_DIR` stopped in the
build script before compilation; rerunning with the pinned local RocksDB
checkout completed successfully.

The first OCI checkout attempt inherited a local-path Git remote and therefore
could not fetch the pushed branch name. The remote configuration in the new
dedicated checkout was corrected to the repository HTTPS URL, the exact pushed
SHA was fetched and checked out cleanly, and the one requested OCI test then
passed. No pre-existing checkout or artifact was modified.

## Scope and remaining limits

R1b-2 adds transaction conformance and normal shutdown/reopen coverage; it does
not complete R1. Tenant lifecycle and capacity/eviction management and
process-kill recovery remain open. Per-tenant write serialization remains in
place. No Debian work, performance measurement, new workload matrix,
serialization change, or process-kill experiment was run.

The existing write response-budget limitation remains: either backend can
commit a successful mutation before response-budget validation reports an
error. The test uses a 1 MiB budget so normal requests do not exercise that
separate behavior or confuse it with transaction atomicity.

The product scope is dodb's existing API with selected DynamoDB-style data
operation semantics. AWS HTTP API support is outside current scope and is not
a future product milestone. This result is correctness evidence only and does
not establish performance-comparison readiness.
