# OCI RocksDB Research Build Cache

This runbook shares Cargo outputs and sccache entries across clean research
checkouts on the OCI ARM64 host. It keeps source checkouts and run evidence
separate from the compiler cache. The helper runs Cargo in the current clean
checkout, pins the repository toolchain, and records the source SHA, command,
environment, elapsed time, compiler-cache statistics, and test executable
hashes for every invocation.

## Fixed environment

The root `rust-toolchain.toml` pins Rust and Cargo to `1.97.1`. OCI is Linux
`aarch64-unknown-linux-gnu`. The repository's `.cargo/config.toml` sets
`-C target-feature=+crc` for that target. The existing release profile has
`debug = true`; builds retain that profile and do not add target CPU or
optimization overrides. `Cargo.lock` is retained and the command uses
`--locked`. The helper sets `CARGO_INCREMENTAL=0`, which is the release
profile's default and is required for sccache's Rust caching support.

The fixed Cargo target path is
`/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target`.
It uses the existing R1b-3a target contents through a stable symlink, after
checking the Cargo rustc fingerprint reports Rust `1.97.1` and the `+crc`
target feature. The previous per-run path remains intact. Host, architecture,
toolchain, release profile, and repository Rust flags are encoded in the path;
the path is shared across source SHAs and run IDs. It is not cleared between
clean checkouts.

The fixed sccache path is
`/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/sccache`.
The helper sets `RUSTC_WRAPPER`, `SCCACHE_DIR`, and a research-specific server
port (`4227`). sccache `0.18.0` is installed from Mozilla's official
`aarch64-unknown-linux-musl` release archive. The pinned archive SHA-256 is
`2b3284d5da3b46a47dc4229e75bb7b88ac4aa99c8d754fb7d2f84997e5a4354a`; verify
it with `sha256sum` before unpacking. The binary is kept at
`/bench/zfs/db/tooling/sccache-0.18.0-aarch64-unknown-linux-musl/sccache`.
No ccache is configured. Cargo can skip a rustc invocation using target
fingerprints before sccache is called, so an unchanged second build can have
zero sccache hits without indicating a cache failure.

The Rust source checkout must be a clean checkout at the exact SHA under
validation. Each invocation has a unique `DODB_BUILD_RUN_ID`; its log, timing,
temporary files, command record, binary path/hash, and SHA256SUMS are stored
under `/bench/zfs/db/dodb-rocksdb-build-runs/<run-id>/`. The database or crash
artifacts are not part of this build helper. A build lock prevents concurrent
use by invocations of this helper; it does not stop or alter unrelated jobs.

## RocksDB reuse and provenance

Use the existing checkout at
`/bench/zfs/db/product-build/rocksdb-source`, pinned to source commit
`abeebd9630f11bd08c28b7bd43c7bdfc62050654`. Its actual
`include/rocksdb/version.h` defines `ROCKSDB_MAJOR 11`, `ROCKSDB_MINOR 8`,
and `ROCKSDB_PATCH 1`; `HISTORY.md` also contains the 11.8.1 release entry.
The existing `/bench/zfs/db/product-build/rocksdb-source/librocksdb.a` is
reused. The helper records the source SHA plus archive and `make_config.mk`
SHA-256 on every invocation. Do not rebuild RocksDB for this cache workflow.

## Running a build

From the clean checkout at the exact source SHA, pass Cargo arguments to the
helper. For example, the build-only `process_crash_contract` command is:

```sh
DODB_BUILD_RUN_ID=process-crash-contract-no-run-01 \
  dodb/scripts/run-oci-rocksdb-build.sh \
  test --locked --release -p dodb-rocksdb --test process_crash_contract --no-run
```

Use a new run ID for every invocation. The helper does not execute test
binaries. Check `environment.txt` for the exact checkout SHA and command,
`cargo.log` for Cargo's `Executable` path, `test-binaries.txt` for the output
binary SHA-256, `time.txt` for elapsed/user/system time, and the before/after
sccache reports for cache counters. Cargo always runs from the checkout being
validated. Never run an executable from a different SHA's checkout as
verification of the current SHA.

## Reuse check

Two consecutive `--no-run` builds were requested for one clean SHA and this
fixed Cargo target. Results, including times and whether Cargo rebuilt any
dependencies, are recorded below after the OCI check. The first build measures
the cost of populating the shared target; the second measures reuse. No test
was executed as part of this check.

## sccache references and limits

The official [sccache Rust documentation](https://github.com/mozilla/sccache/blob/main/docs/Rust.md)
requires incremental compilation to be disabled and lists compiler cases that
cannot be cached, including procedural macros that read files and crates that
invoke the system linker. The [configuration documentation](https://github.com/mozilla/sccache/blob/main/docs/Configuration.md)
describes the local `SCCACHE_DIR` and `SCCACHE_BASEDIRS` path normalization.
This helper does not normalize source paths: Cargo target reuse is the primary
mechanism, and sccache keys retain their normal source, compiler, argument,
and path distinctions. The small C++ transaction shim is compiled by
`cc-rs`; this helper does not change its compiler configuration. The existing
RocksDB archive is reused without a rebuild, so this task makes no claim about
C++ cache benefit.
