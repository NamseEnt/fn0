#!/usr/bin/env bash
set -euo pipefail

toolchain="1.97.1"
target_path="/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-${toolchain}/release-crc/target"
sccache_path="/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-${toolchain}/release-crc/sccache"
sccache_binary="/bench/zfs/db/tooling/sccache-0.18.0-aarch64-unknown-linux-musl/sccache"
rocksdb_path="/bench/zfs/db/product-build/rocksdb-source"
result_root="/bench/zfs/db/dodb-rocksdb-build-runs"
run_id="${DODB_BUILD_RUN_ID:-}"
run_suffix="$(printf '%s' "$run_id" | sha256sum | cut -c 1-16)"
tmp_path="/tmp/db-${run_suffix}"
repo_root="$(git rev-parse --show-toplevel)"
source_sha="$(git -C "$repo_root" rev-parse HEAD)"

if [[ -z "$run_id" || ! "$run_id" =~ ^[A-Za-z0-9._-]+$ ]]; then
  printf 'DODB_BUILD_RUN_ID must be a unique path-safe run identifier\n' >&2
  exit 2
fi

if [[ "$#" -eq 0 ]]; then
  printf 'pass Cargo arguments after the script name\n' >&2
  exit 2
fi

if [[ -n "$(git -C "$repo_root" status --porcelain)" ]]; then
  printf 'source checkout must be clean\n' >&2
  exit 2
fi

run_path="$result_root/$run_id"
lock_path="/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-${toolchain}/release-crc/build.lock"
mkdir -p "$result_root" "$(dirname "$target_path")" "$sccache_path"
if ! mkdir "$lock_path" 2>/dev/null; then
  printf 'shared build path is locked: %s\n' "$lock_path" >&2
  exit 2
fi
trap 'rmdir "$lock_path" 2>/dev/null || true' EXIT
if ! mkdir "$run_path"; then
  printf 'run evidence path already exists: %s\n' "$run_path" >&2
  exit 2
fi
if ! mkdir "$tmp_path"; then
  printf 'run temporary path already exists: %s\n' "$tmp_path" >&2
  exit 2
fi

if [[ ! -x "$sccache_binary" ]]; then
  printf 'pinned sccache binary is missing: %s\n' "$sccache_binary" >&2
  exit 2
fi
if [[ ! -f "$rocksdb_path/librocksdb.a" ]]; then
  printf 'pinned RocksDB archive is missing: %s\n' "$rocksdb_path/librocksdb.a" >&2
  exit 2
fi

export CARGO_TARGET_DIR="$target_path"
export DODB_ROCKSDB_DIR="$rocksdb_path"
export CARGO_INCREMENTAL=0
export SCCACHE_DIR="$sccache_path"
export RUSTC_WRAPPER="$sccache_binary"
export SCCACHE_SERVER_UDS="/tmp/dodb-sccache-rust1971-arm64.sock"
export TMPDIR="$tmp_path"

{
  date -u '+utc=%Y-%m-%dT%H:%M:%SZ'
  printf 'source_path=%s\nsource_sha=%s\n' "$repo_root" "$source_sha"
  printf 'toolchain=%s\n' "$toolchain"
  rustup run "$toolchain" rustc -vV
  rustup run "$toolchain" cargo -V
  printf 'host_architecture=%s\n' "$(uname -m)"
  printf 'cargo_target_dir=%s\nsccache_dir=%s\nrocksdb_dir=%s\n' "$CARGO_TARGET_DIR" "$SCCACHE_DIR" "$DODB_ROCKSDB_DIR"
  printf 'run_tmpdir=%s\n' "$TMPDIR"
  printf 'sccache_version=%s\n' "$("$sccache_binary" --version)"
  printf 'sccache_sha256=%s\n' "$(sha256sum "$sccache_binary" | cut -d ' ' -f 1)"
  printf 'cargo_incremental=%s\nrustc_wrapper=%s\nsccache_server_uds=%s\n' "$CARGO_INCREMENTAL" "$RUSTC_WRAPPER" "$SCCACHE_SERVER_UDS"
  printf 'rocksdb_source_sha=%s\n' "$(git -C "$rocksdb_path" rev-parse HEAD)"
  sha256sum "$rocksdb_path/librocksdb.a"
  sha256sum "$rocksdb_path/make_config.mk"
  sha256sum "$repo_root/rust-toolchain.toml" "$repo_root/.cargo/config.toml" "$repo_root/Cargo.lock" "$repo_root/Cargo.toml"
  git -C "$repo_root" status --short --branch
  printf 'command='
  printf '%q ' rustup run "$toolchain" cargo "$@"
  printf '\n'
} > "$run_path/environment.txt" 2>&1

if ss -xlpn | grep -F "$SCCACHE_SERVER_UDS" | grep -q 'LISTEN'; then
  printf 'reused sccache server at %s\n' "$SCCACHE_SERVER_UDS" > "$run_path/sccache-server-start.txt"
else
  "$sccache_binary" --start-server > "$run_path/sccache-server-start.txt" 2>&1
fi
printf '%s\n' 'sccache --show-stats' > "$run_path/sccache-before-command.txt"
"$sccache_binary" --show-stats >> "$run_path/sccache-before-command.txt" 2>&1
set +e
/usr/bin/time -f 'elapsed_seconds=%e\nuser_seconds=%U\nsystem_seconds=%S' -o "$run_path/time.txt" \
  rustup run "$toolchain" cargo "$@" > "$run_path/cargo.log" 2>&1
cargo_status=$?
set -e
printf '%s\n' "$cargo_status" > "$run_path/exit-status.txt"
printf '%s\n' 'sccache --show-stats' > "$run_path/sccache-after-command.txt"
"$sccache_binary" --show-stats >> "$run_path/sccache-after-command.txt" 2>&1

python3 - "$run_path/cargo.log" "$target_path" > "$run_path/test-binaries.txt" <<'PY'
import hashlib
import re
import sys
from pathlib import Path

log_path = Path(sys.argv[1])
target_path = Path(sys.argv[2]).resolve()
for line in log_path.read_text(errors="replace").splitlines():
    match = re.search(r"^\s*Executable .*\((.*process_crash_contract-[^)]*)\)$", line)
    if match is None:
        continue
    binary_path = Path(match.group(1)).resolve()
    if target_path not in binary_path.parents or not binary_path.is_file():
        raise SystemExit(f"unexpected test executable path: {binary_path}")
    digest = hashlib.sha256(binary_path.read_bytes()).hexdigest()
    print(f"path={binary_path}")
    print(f"sha256={digest}")
PY

find "$run_path" -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 -r sha256sum > "$run_path/SHA256SUMS"
exit "$cargo_status"
