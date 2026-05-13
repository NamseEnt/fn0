# shellcheck shell=bash
# Build fn0-control raw bundle (bundle.raw.tar). Source-only.
#
# build_control_raw_bundle <env_yaml_path> <output_path>
#   Runs `forte build` in fn0/control and assembles the resulting dist/ plus
#   the encrypted env.yaml into a raw bundle tar (manifest.json + backend.wasm
#   + entry.js + env.yaml). Mirrors the layout of
#   fn0_deploy::create_raw_bundle_forte.

if [[ -n "${__FN0_CONTROL_BUNDLE_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_BUNDLE_LOADED=1

__forte_cmd() {
  if command -v forte >/dev/null 2>&1; then
    forte "$@"
  else
    (cd "$REPO_ROOT" && cargo run --release -p forte-cli --quiet -- "$@")
  fi
}

build_control_raw_bundle() {
  local env_yaml_path="$1"
  local out_path="$2"
  if [[ -z "$env_yaml_path" || -z "$out_path" ]]; then
    echo "build_control_raw_bundle: missing args" >&2
    return 2
  fi

  local control_dir="${REPO_ROOT}/fn0/control"
  if [[ ! -d "$control_dir" ]]; then
    echo "build_control_raw_bundle: ${control_dir} not found" >&2
    return 1
  fi

  if [[ -f "${control_dir}/fe/package.json" ]]; then
    echo ">> npm ci (fn0/control/fe)"
    (cd "${control_dir}/fe" && npm ci --silent)
  fi

  echo ">> forte build (fn0/control)"
  (cd "$control_dir" && __forte_cmd build)

  local backend_wasm="${control_dir}/dist/backend.wasm"
  local server_js="${control_dir}/dist/server.js"
  if [[ ! -f "$backend_wasm" || ! -f "$server_js" ]]; then
    echo "forte build produced unexpected artifacts in ${control_dir}/dist" >&2
    return 1
  fi

  local stage
  stage="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$stage'" RETURN

  printf '{"kind":"wasmjs"}' > "${stage}/manifest.json"
  cp "$backend_wasm" "${stage}/backend.wasm"
  cp "$server_js"    "${stage}/entry.js"

  local entries=(manifest.json backend.wasm entry.js)
  if [[ -n "$env_yaml_path" && -f "$env_yaml_path" ]]; then
    cp "$env_yaml_path" "${stage}/env.yaml"
    entries+=(env.yaml)
  fi

  echo ">> assembling raw bundle ${out_path}"
  (cd "$stage" && tar -cf "$out_path" --format=ustar "${entries[@]}")
}
