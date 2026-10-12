#!/usr/bin/env bash
set -euo pipefail

build_root=${1:?usage: build-native.sh BUILD_ROOT ROCKSDB_SOURCE}
rocksdb_source=${2:?usage: build-native.sh BUILD_ROOT ROCKSDB_SOURCE}
repo_root=$(git rev-parse --show-toplevel)
harness_root="$repo_root/dodb/docs/experiments/results/oci-a1-2ocpu-12g-zfs-crossdb/harness"
source_sha=$(git -C "$repo_root" rev-parse HEAD)
[[ -z "$(git -C "$repo_root" status --porcelain)" ]] || {
    printf 'dodb checkout must be clean before building\n' >&2
    exit 2
}
[[ "$(git -C "$rocksdb_source" rev-parse HEAD)" == abeebd9630f11bd08c28b7bd43c7bdfc62050654 ]] || {
    printf 'RocksDB source commit does not match the pinned build\n' >&2
    exit 2
}
[[ "$(git -C "$rocksdb_source" rev-parse refs/tags/v11.8.1^{commit})" == abeebd9630f11bd08c28b7bd43c7bdfc62050654 ]] || {
    printf 'RocksDB v11.8.1 tag does not point to the pinned source commit\n' >&2
    exit 2
}
[[ -z "$(git -C "$rocksdb_source" status --porcelain)" ]] || {
    printf 'RocksDB source checkout must be clean before building\n' >&2
    exit 2
}
[[ ! -e "$rocksdb_source/librocksdb.a" && ! -e "$rocksdb_source/make_config.mk" ]] || {
    printf 'RocksDB build directory already contains compiled outputs; preserve it and provide a fresh source checkout\n' >&2
    exit 2
}
mkdir -p "$build_root"
export RUSTFLAGS="-C target-cpu=native"
export DODB_SOURCE_SHA="$source_sha"
export DODB_SOURCE_CLEAN=true
export CROSSDB_SOURCE_SHA="$source_sha"
export CROSSDB_SOURCE_CLEAN=true
export CROSSDB_ROCKSDB_TAG=v11.8.1
export CROSSDB_ROCKSDB_COMMIT=abeebd9630f11bd08c28b7bd43c7bdfc62050654
export CROSSDB_ROCKSDB_DIR="$rocksdb_source"
rocksdb_extra_cxxflags='-Wno-error=unused-parameter -fPIC'
make -C "$rocksdb_source" -j"$(getconf _NPROCESSORS_ONLN)" EXTRA_CXXFLAGS="$rocksdb_extra_cxxflags" static_lib
export CARGO_TARGET_DIR="$build_root/target-dodb"
cargo build --locked --release --manifest-path "$harness_root/dodb-bench/Cargo.toml"
export CARGO_TARGET_DIR="$build_root/target-rocksdb"
cargo build --locked --release --manifest-path "$harness_root/rocksdb-bench/Cargo.toml"
dodb_binary="$build_root/target-dodb/release/dodb-bench"
rocksdb_binary="$build_root/target-rocksdb/release/rocksdb-bench"
rustc -vV > "$build_root/rustc-version.txt"
cargo --version > "$build_root/cargo-version.txt"
cc --version > "$build_root/c-compiler-version.txt"
c++ --version > "$build_root/cxx-compiler-version.txt"
file "$dodb_binary" "$rocksdb_binary" "$rocksdb_source/librocksdb.a" > "$build_root/file-identification.txt"
git -C "$repo_root" status --porcelain > "$build_root/dodb-status-after-build.txt"
git -C "$rocksdb_source" status --porcelain > "$build_root/rocksdb-status-after-build.txt"
[[ ! -s "$build_root/dodb-status-after-build.txt" ]]
[[ ! -s "$build_root/rocksdb-status-after-build.txt" ]]
{
    printf 'source_sha=%s\n' "$source_sha"
    printf 'source_clean=true\n'
    printf 'architecture=%s\n' "$(uname -m)"
    printf 'cargo_build_command=dodb: cargo build --locked --release --manifest-path %s\n' "$harness_root/dodb-bench/Cargo.toml"
    printf 'cargo_build_command=rocksdb: cargo build --locked --release --manifest-path %s\n' "$harness_root/rocksdb-bench/Cargo.toml"
    printf 'rocksdb_build_command=make -C %s -j%s EXTRA_CXXFLAGS=%s static_lib\n' "$rocksdb_source" "$(getconf _NPROCESSORS_ONLN)" "$rocksdb_extra_cxxflags"
    printf 'rustflags=%s\n' "$RUSTFLAGS"
    printf 'rocksdb_source_sha=%s\n' "$(git -C "$rocksdb_source" rev-parse HEAD)"
    printf 'rocksdb_tag=v11.8.1\n'
    printf 'rocksdb_library_sha256=%s\n' "$(sha256sum "$rocksdb_source/librocksdb.a" | cut -d' ' -f1)"
    printf 'rocksdb_make_config_sha256=%s\n' "$(sha256sum "$rocksdb_source/make_config.mk" | cut -d' ' -f1)"
    printf 'dodb_cargo_lock_sha256=%s\n' "$(sha256sum "$harness_root/dodb-bench/Cargo.lock" | cut -d' ' -f1)"
    printf 'rocksdb_cargo_lock_sha256=%s\n' "$(sha256sum "$harness_root/rocksdb-bench/Cargo.lock" | cut -d' ' -f1)"
    printf 'cargo_config_sha256=%s\n' "$(sha256sum "$repo_root/.cargo/config.toml" | cut -d' ' -f1)"
    printf 'dodb_harness_sha256=%s\n' "$(find "$harness_root/dodb-bench/src" "$harness_root/common" -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum | cut -d' ' -f1)"
    printf 'rocksdb_harness_sha256=%s\n' "$(find "$harness_root/rocksdb-bench" -type f -not -path '*/target/*' -print0 | sort -z | xargs -0 sha256sum | sha256sum | cut -d' ' -f1)"
    printf 'dodb_binary_sha256=%s\n' "$(sha256sum "$dodb_binary" | cut -d' ' -f1)"
    printf 'rocksdb_binary_sha256=%s\n' "$(sha256sum "$rocksdb_binary" | cut -d' ' -f1)"
    printf 'host=%s\n' "$(hostname -f)"
    printf 'kernel=%s\n' "$(uname -a)"
} > "$build_root/build-provenance.txt"
printf 'DODB_MAIN_BINARY=%q\nDODB_PLANNED_BINARY=%q\nROCKSDB_BINARY=%q\n' "$dodb_binary" "$dodb_binary" "$rocksdb_binary" > "$build_root/runner.env"
find "$build_root" -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > "$build_root/SHA256SUMS"
sha256sum -c "$build_root/SHA256SUMS" > "$build_root/checksum-verification.txt"
