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
The helper sets `RUSTC_WRAPPER`, `SCCACHE_DIR`, and a research-specific Unix
socket at `/tmp/dodb-sccache-rust1971-arm64.sock`. Its per-run `TMPDIR` is a
short unique path under `/tmp` because sccache creates its startup notification
socket under the temporary directory, which otherwise can exceed the Unix
socket path limit. The helper reuses an active listener on this dedicated
socket and starts one only when none is listening. sccache `0.18.0` is
installed from Mozilla's
[official release](https://github.com/mozilla/sccache/releases/tag/v0.18.0)
`aarch64-unknown-linux-musl` archive. The pinned archive SHA-256 is
`2b3284d5da3b46a47dc4229e75bb7b88ac4aa99c8d754fb7d2f84997e5a4354a`; verify
it with `sha256sum` before unpacking. The binary is kept at
`/bench/zfs/db/tooling/sccache-0.18.0-aarch64-unknown-linux-musl/sccache`.
Its SHA-256 is
`0fd82ece4469b791e30fffdd43d58d2e6cae303950c0d6c8faad932d93e35cf4`.
No ccache is configured. Cargo can skip a rustc invocation using target
fingerprints before sccache is called, so an unchanged second build can have
zero sccache hits without indicating a cache failure.

The Rust source checkout must be a clean checkout at the exact SHA under
validation. Each invocation has a unique `DODB_BUILD_RUN_ID`; its log, timing,
command record, binary path/hash, and SHA256SUMS are stored under
`/bench/zfs/db/dodb-rocksdb-build-runs/<run-id>/`; temporary files use a
unique short path under `/tmp`. The database or crash artifacts are not part
of this build helper. A build lock prevents concurrent use by invocations of
this helper; it does not stop or alter unrelated jobs.

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
  bash dodb/scripts/run-oci-rocksdb-build.sh \
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

The first four helper attempts stopped before Cargo compilation with a Unix
socket path error; their sccache compile request counts stayed at zero. Those
failed attempts are retained in their own run directories. A standalone check
showed that the pinned compiler responds when the sccache server is started
with the short temp path and dedicated socket. The helper now starts one
server only when its listener is absent and reuses the listener otherwise.
No unrelated process was stopped. No test was executed as part of this check.

The final helper was exercised twice consecutively from a clean detached OCI
checkout at source SHA `913c9728c21cb5b60f0fe2803d7e94b8957ea8ac`, using the
fixed shared target. The first invocation reused the prior target contents
but compiled six workspace crates for the new checkout root. This was not an
empty-target cold build. The second invocation reused the first invocation's
artifacts without any `Compiling` lines.

| Run evidence directory | Elapsed | Cargo output | sccache change |
| --- | ---: | --- | --- |
| `/bench/zfs/db/dodb-rocksdb-build-runs/process-crash-contract-913c9728c21cb5b60f0fe2803d7e94b8957ea8ac-build-1/` | 60.29 s | Compiled `dodb-core`, `dodb-rocksdb`, `dodb-service`, `dodb-protocol`, `dodb-storage`, and `dodb-server`; Release `--no-run` succeeded. | 9 requests; 7 executed, with 7 misses (6 Rust, 1 C/C++), 0 hits, and 2 non-cacheable calls. |
| `/bench/zfs/db/dodb-rocksdb-build-runs/process-crash-contract-913c9728c21cb5b60f0fe2803d7e94b8957ea8ac-build-2/` | 0.37 s | `Finished release` in 0.33 s; no crates recompiled; Release `--no-run` succeeded. | Before/after remained 9 requests, 7 executed, 7 misses, and 0 hits. Cargo did not call the compiler, so no new sccache hit was expected. |

Both runs used `bash dodb/scripts/run-oci-rocksdb-build.sh test --locked --release -p dodb-rocksdb --test process_crash_contract --no-run` and returned status 0. The Cargo-reported executable was under the shared target at `release/deps/process_crash_contract-8a59a0b53a7c7951`; its SHA-256 was `23ccd28c43120d4ab582b0e13bff6e7e45896c38af250c6100e19df2bf30c5b5`. The helper records the resolved executable path and hash in each run directory. The executable was not run.

The Cargo target existed before this check and was reused from the R1b-3a
target path after its `.rustc_info.json` confirmed Rust `1.97.1` and `+crc`.
The R1b-3a report previously listed OCI Rust `1.98.1`, while the repository's
explicit `rust-toolchain.toml` pins `1.97.1`. The retained R1b-3a direct run log
does not record `rustc -vV` or `cargo -V`, so the historical actual versions
are unverified; the pin and reused target fingerprint do not establish them.
This run follows the repository pin. The new clean checkout root differed from
the earlier checkout root, so Cargo
rebuilt workspace crates on the first invocation. The helper leaves
`SCCACHE_BASEDIRS` unset to retain absolute path distinctions in cache keys.
For later sequential SHAs, keep the clean checkout at the same path when
possible. Compiler version, source, and flags remain part of normal cache
decisions.

The existing RocksDB source SHA was
`abeebd9630f11bd08c28b7bd43c7bdfc62050654`; the archive SHA-256 was
`9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5` and
`make_config.mk` SHA-256 was
`bcd375a99550a9314a92ec3673cb7540ed4eea06909477bf3675d772baefaf71`.
`make_config.mk` defines Linux with `CC=cc`, `CXX=g++`, and `AR=gcc-ar`, and
the source `include/rocksdb/version.h` defines version 11.8.1. The archive was
not rebuilt. The single C/C++ sccache miss was from this build's C++ shim
compilation; no RocksDB archive cache effect was measured.

## sccache references and limits

The official [sccache Rust documentation](https://github.com/mozilla/sccache/blob/main/docs/Rust.md)
requires incremental compilation to be disabled and lists compiler cases that
cannot be cached, including procedural macros that read files and crates that
invoke the system linker. The [configuration documentation](https://github.com/mozilla/sccache/blob/main/docs/Configuration.md)
describes the local `SCCACHE_DIR` and `SCCACHE_BASEDIRS` path normalization.
This helper does not normalize source paths: Cargo target reuse is the primary
mechanism, and sccache keys retain their normal source, compiler, argument,
and path distinctions. The C++ transaction shim produced one sccache miss;
the existing RocksDB archive was reused without a rebuild.
