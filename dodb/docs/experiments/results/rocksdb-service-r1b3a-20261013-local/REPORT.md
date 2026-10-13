# R1b-3a acknowledged-transaction SIGKILL recovery report

## Result

The shared crash scenario passed on native and RocksDB locally and on OCI.
After each target `DodbService::execute` returned a committed transaction
response, the child emitted an explicit success handoff and stayed alive with
its service and database handles open. The parent then sent SIGKILL to that
child PID, reaped it, verified signal 9, reopened the same database path, and
compared the complete logical state with an oracle built from the parent-owned
seed and target requests.

This verifies only that one acknowledged width-four transaction survives
process SIGKILL and reopen on each backend. It does not verify interruption
during commit, a WAL or sync syscall boundary, power loss, loss of page cache or
storage, long-term stability, or performance. This is not completion of the
process-crash inventory or R1.

## Scenario and independent oracle

The parent creates a fresh temporary path for each backend, constructs the
seed transaction and width-four target transaction, and derives the expected
document map by applying the request mutations to the seeded map. The seed
contains four existing documents. The target conditions are successful for
that seed: `Exists` for both documents being changed/deleted and `NotExists` for
the new key. Its mutations update two existing values, create one value, and
delete one existing key. The fourth seeded document is untouched.

After reopen, a complete paginated Scan must equal the parent's four expected
key/value pairs, which rejects extra documents and partial application. Get
must report the deleted key missing with a nonzero tombstone revision. Both
updated values and the created value, plus the deleted tombstone, must share
one transaction revision. The untouched document must retain its seed value
and an older revision. Revision numbers are compared only within each backend;
no cross-backend numeric identity is assumed.

The native and RocksDB branches differ only in service construction for the
child and reopened parent service. Both use the same scenario builder, service
requests, handoff format, timeout, child termination checks, and reopen
assertions.

## Handoff and termination

The child requires committed `Response::Transact` outcomes for both seed and
target requests. Only then it writes and flushes
`R1B3A_HANDOFF|success|<backend>` to stdout. The parent accepts only that full
line before sending SIGKILL. The child then awaits a pending future while
retaining the service; it does not return from the service scope or execute
normal shutdown. No checkpoint, extra flush, explicit sync, or data mutation
occurs after the handoff. The durability under test is the commit response's
ordinary backend behavior.

The parent uses `Child::kill` on the specific child handle and requires
`ExitStatusExt::signal() == Some(9)` before reopening. A 60-second handoff
timeout is a test failure, not a durability result. A child error, early exit,
or incomplete handoff is also a failure. The child guard kills and reaps only
the child it created if the parent unwinds. The code implementing these checks
is in `dodb/crates/dodb-rocksdb/tests/process_crash_contract.rs`; the child
handoff/pending wait is at lines 100-120, the PID-specific kill and signal
assertion at lines 133-147, and the reopen comparison at lines 285-360.

## Execution counts

| Environment | Planned crash cases | Final implementation run | Passed | Failed | Not reached |
| --- | ---: | ---: | ---: | ---: | ---: |
| Local macOS ARM64 | 2 | 2 | 2 | 0 | 0 |
| OCI A1 ARM64 Release | 2 | 2 | 2 | 0 | 0 |
| Planned total | 4 | 4 | 4 | 0 | 0 |

An earlier local pre-commit pass ran the same two backend crash cases before
the final oracle was changed to derive expected values directly from request
mutations and before explicit per-backend evidence lines were added. Both
preliminary cases passed; the final implementation was then rerun locally and
passed both cases. Thus six crash cases actually executed across all attempts:
four final accepted cases and two passing preliminary local cases. Two setup
commands did not start any crash child: an initial local evidence-directory
command used the wrong relative path, and an OCI version probe assumed a
`VERSION` file that the pinned RocksDB checkout does not contain. Neither was
a scenario failure or durability result.

The integration executable reports three Rust test-harness tests because it
also contains a child-only entry point; the scenario count is two per test
invocation, one for each backend. The child entry point is selected explicitly
with `--exact service_child`, so parent scenarios cannot recurse.

## Verification provenance

| Environment | Source SHA | Build and command | RocksDB | Result |
| --- | --- | --- | --- | --- |
| Local macOS 26.4, ARM64, Rust 1.97.1 | `551471d5ea48d9e0d1a185b6bdf274ae09cf3ae5` | `cargo fmt -p dodb-rocksdb -- --check`; `env DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo check --locked --offline -p dodb-rocksdb --all-targets`; `env DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo test --locked --offline -p dodb-rocksdb --test process_crash_contract -- --nocapture` | 11.8.1, source `abeebd9630f11bd08c28b7bd43c7bdfc62050654` | Formatting, all-target check, and both final scenarios passed. Log: `local-process-crash-final.log`. |
| OCI A1, Linux aarch64, Rust 1.98.1, ZFS | `551471d5ea48d9e0d1a185b6bdf274ae09cf3ae5` | Clean detached checkout; `env DODB_ROCKSDB_DIR=/bench/zfs/db/product-build/rocksdb-source CARGO_TARGET_DIR=/bench/zfs/db/dodb-rocksdb-r1b3a-551471d5/target TMPDIR=/bench/zfs/db/dodb-rocksdb-r1b3a-551471d5/tmp cargo test --locked --release -p dodb-rocksdb --test process_crash_contract -- --nocapture` | 11.8.1, source `abeebd9630f11bd08c28b7bd43c7bdfc62050654` | Native ARM64 Release build and both scenarios passed. ZFS mount: `/bench/zfs/db` (`dodbbench/db`). ELF test binary SHA-256: `ef6887d4b19ddd8f70d2902e1bb672fb8220a4d5c998e1570538f6fba96db62f`. Checkout status was clean at the exact source SHA. Log: `oci-process-crash.log`. |

The OCI checkout and target/temp directories were new and dedicated to this
run. No source was changed on OCI. No existing OCI experiment was stopped,
deleted, or overwritten. No Debian task or performance measurement was run.

## Failures and corrections

No scenario failed, timed out, or required a backend change. The two preliminary
setup errors described above were corrected by using the repository-relative
evidence path and identifying the pinned RocksDB version from the source
commit (`Additional HISTORY.md update for 11.8.1`). The final local and OCI
runs used the same implementation SHA. No durability contract was weakened
and no extra flush was introduced.

## Remaining R1 work

Tenant lifecycle and capacity management remain unimplemented. Commit-
interruption recovery, other process-crash cases, and review of the existing
write-serialization constraint before performance comparison remain open.
This result does not complete R1.
