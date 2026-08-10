#!/usr/bin/env bash
# Compiles a workspace crate to a linux/arm64 release binary, outside the
# container image build.
#
#   build-rust-linux-arm64-bin.sh <package> <out_dir>
#
# Runs `cargo build --release` in a `rust:bookworm` container with the repo
# bind-mounted and target/cargo-registry on persistent named volumes, then
# copies the binary to <out_dir>/<package>. Bind-mounting (vs `COPY` into
# an image build) keeps source mtimes stable so cargo stays incremental; the
# named volumes persist the compile cache across runs. Assumes an arm64 host.
set -euo pipefail

package="${1:?usage: build-rust-linux-arm64-bin.sh <package> <out_dir>}"
out_dir="${2:?usage: build-rust-linux-arm64-bin.sh <package> <out_dir>}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# shellcheck source=lib/container-runtime.sh
source "${repo_root}/scripts/lib/container-runtime.sh"
container_runtime_ensure_available

target_volume="fn0-build-target"
cargo_registry_volume="fn0-build-cargo-registry"

container_runtime_volume_create "$target_volume"
container_runtime_volume_create "$cargo_registry_volume"
mkdir -p "$out_dir"

container_runtime_set_full_host_resources

echo ">> cargo build --release -p ${package} (linux/arm64, ${CONTAINER_RUNTIME_CLI})"
container_runtime_run --rm \
  ${CONTAINER_RUNTIME_FULL_HOST_RESOURCE_ARGS[@]+"${CONTAINER_RUNTIME_FULL_HOST_RESOURCE_ARGS[@]}"} \
  -v "${repo_root}:/app" -w /app \
  -v "${target_volume}:/app/target" \
  -v "${cargo_registry_volume}:/usr/local/cargo/registry" \
  -v "${out_dir}:/out" \
  rust:bookworm \
  sh -c "cargo build --release --locked -p ${package} && cp target/release/${package} /out/${package}"

echo ">> binary: ${out_dir}/${package}"
