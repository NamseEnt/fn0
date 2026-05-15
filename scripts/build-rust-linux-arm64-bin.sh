#!/usr/bin/env bash
# Compiles a workspace crate to a linux/arm64 release binary, outside `docker build`.
#
#   build-rust-linux-arm64-bin.sh <package> <out_dir>
#
# Runs `cargo build --release` in a `rust:bookworm` container with the repo
# bind-mounted and target/cargo-registry on persistent named volumes, then
# copies the binary to <out_dir>/<package>. Bind-mounting (vs `COPY` into
# `docker build`) keeps source mtimes stable so cargo stays incremental; the
# named volumes persist the compile cache across runs. Assumes an arm64 host.
set -euo pipefail

package="${1:?usage: build-rust-linux-arm64-bin.sh <package> <out_dir>}"
out_dir="${2:?usage: build-rust-linux-arm64-bin.sh <package> <out_dir>}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

target_volume="fn0-build-target"
cargo_registry_volume="fn0-build-cargo-registry"

docker volume create "$target_volume" >/dev/null
docker volume create "$cargo_registry_volume" >/dev/null
mkdir -p "$out_dir"

echo ">> cargo build --release -p ${package} (linux/arm64, containerized)"
docker run --rm \
  -v "${repo_root}:/app" -w /app \
  -v "${target_volume}:/app/target" \
  -v "${cargo_registry_volume}:/usr/local/cargo/registry" \
  -v "${out_dir}:/out" \
  rust:bookworm \
  sh -c "cargo build --release --locked -p ${package} && cp target/release/${package} /out/${package}"

echo ">> binary: ${out_dir}/${package}"
