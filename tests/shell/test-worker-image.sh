#!/usr/bin/env bash

set -euo pipefail

source_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
temporary_dir="$(mktemp -d)"
trap 'rm -rf "$temporary_dir"' EXIT
REPO_ROOT="${temporary_dir}/repo"
mkdir -p "${REPO_ROOT}/scripts/lib" "${REPO_ROOT}/scripts"
cp "${source_root}/scripts/lib/worker-image.sh" "${REPO_ROOT}/scripts/lib/worker-image.sh"
: >"${REPO_ROOT}/scripts/lib/container-runtime.sh"
cat >"${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh" <<'BUILD_BINARY'
#!/usr/bin/env bash
printf '%s\n' build-binary >>"$TEST_LOG"
BUILD_BINARY
chmod +x "${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh"
mock_bin="${temporary_dir}/bin"
mkdir -p "$mock_bin"
cat >"${mock_bin}/cargo" <<'CARGO'
#!/usr/bin/env bash
printf '%s\n' "$*" >>"$TEST_CARGO_LOG"
CARGO
chmod +x "${mock_bin}/cargo"
export PATH="${mock_bin}:$PATH"
export REPO_ROOT TEST_LOG="${temporary_dir}/calls" TEST_CARGO_LOG="${temporary_dir}/cargo-calls"
export TEST_SOURCE="source-419" TEST_REMOTE_STATE="absent" TEST_REMOTE_SOURCE=""
export CONTAINER_RUNTIME_CLI=mock-runtime
source "${REPO_ROOT}/scripts/lib/worker-image.sh"

pulumi_pick_json() {
  printf '%s\n' '[{"url":"registry.example","repository":"fn0-worker","username":"user","password":"password"}]'
}

fn0_worker_source_hash() { printf '%s' "$TEST_SOURCE"; }
fn0_worker_version() { printf '%s' '0.4.19'; }

fn0_worker_remote_manifest_exists() {
  [[ "$TEST_REMOTE_STATE" == exists ]]
}

fn0_worker_remote_source_label() { printf '%s' "$TEST_REMOTE_SOURCE"; }
container_runtime_build_image() { printf '%s\n' build-image >>"$TEST_LOG"; CONTAINER_RUNTIME_BUILT_IMAGE=image; }
fn0_worker_registry_login() { printf '%s\n' login >>"$TEST_LOG"; }
container_runtime_tag() { printf '%s\n' tag >>"$TEST_LOG"; }
container_runtime_push() { printf '%s\n' push >>"$TEST_LOG"; }

: >"$TEST_LOG"
: >"$TEST_CARGO_LOG"
build_and_push_fn0_worker
[[ "$(cat "$TEST_LOG")" == $'build-binary\nbuild-image\nlogin\ntag\npush' ]]
[[ ! -s "$TEST_CARGO_LOG" ]]
[[ "$FN0_WORKER_PUSHED_IMAGE_REF" == "registry.example/fn0-worker:0.4.19" ]]

TEST_REMOTE_STATE=exists
TEST_REMOTE_SOURCE="$TEST_SOURCE"
: >"$TEST_LOG"
build_and_push_fn0_worker
[[ ! -s "$TEST_LOG" ]]
[[ ! -s "$TEST_CARGO_LOG" ]]

TEST_REMOTE_SOURCE=different-source
: >"$TEST_LOG"
if build_and_push_fn0_worker >/dev/null 2>&1; then exit 1; fi
[[ ! -s "$TEST_LOG" ]]
[[ ! -s "$TEST_CARGO_LOG" ]]

printf '%s\n' 'worker image shell tests passed'
