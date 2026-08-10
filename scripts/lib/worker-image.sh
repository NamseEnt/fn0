# shellcheck shell=bash
# Worker image build & push. Source-only.
#
# build_and_push_fn0_worker
#   Idempotent. Publishes fn0-worker and builds/pushes the linux/arm64 image to
#   every registry in pulumi.workerImageRegistries, skipping the whole thing
#   when the remote tags already carry this source. Echoes the resulting
#   image_ref (first registry) via FN0_WORKER_PUSHED_IMAGE_REF.
#
# resolve_fn0_worker_image_ref <version>
#   Resolves an already-published image_ref without building anything, for
#   callers that only need to point at a worker image. Sets
#   FN0_WORKER_IMAGE_REF.

if [[ -n "${__FN0_WORKER_IMAGE_LOADED:-}" ]]; then
  return 0
fi
__FN0_WORKER_IMAGE_LOADED=1

# shellcheck source=container-runtime.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/container-runtime.sh"

# Stamped on every image this script pushes, and the only thing the overwrite
# guard compares. The built artifact cannot serve that role: fn0/ski/build.rs
# bakes a V8 startup snapshot whose module-map ordering differs between builds,
# so two builds of one commit yield different layer digests with a byte-identical
# .text. Comparing digests answers "was this the same build run", never "is the
# remote tag this source", and reports every rebuild as a mismatch.
FN0_WORKER_SOURCE_LABEL="fn0.worker.source"

# Hashes every file that can change the worker binary: the path-dependency
# closure cargo resolves for fn0-worker, plus the lockfile, the workspace
# manifest (profile settings) and the two build inputs outside cargo.
fn0_worker_source_hash() {
  local package_dirs file_list
  package_dirs="$(cd "$REPO_ROOT" && cargo metadata --format-version 1 | jq -r '
    (.packages | map({key: .id, value: .}) | from_entries) as $package_by_id
    | (.resolve.nodes | map({key: .id, value: [.deps[].pkg]}) | from_entries) as $dependencies_of
    | [(.packages[] | select(.name == "fn0-worker") | .id)]
    | until(
        . as $seen | (($seen + [$seen[] | $dependencies_of[.][]?]) | unique) == $seen;
        ((. + [.[] | $dependencies_of[.][]?]) | unique)
      )
    | map($package_by_id[.] | select(.source == null) | .manifest_path | sub("/Cargo.toml$"; ""))
    | sort | .[]')"
  if [[ -z "$package_dirs" ]]; then
    echo "failed to resolve fn0-worker path dependencies" >&2
    return 1
  fi

  local -a paths=()
  local dir
  while IFS= read -r dir; do
    paths+=("${dir#"${REPO_ROOT}/"}")
  done <<<"$package_dirs"
  paths+=("Cargo.lock" "Cargo.toml" "scripts/build-rust-linux-arm64-bin.sh")

  file_list="$(cd "$REPO_ROOT" && git ls-files --cached --others --exclude-standard -- "${paths[@]}" | LC_ALL=C sort)"
  if [[ -z "$file_list" ]]; then
    echo "fn0-worker source file list came back empty" >&2
    return 1
  fi

  (
    cd "$REPO_ROOT" || exit 1
    {
      printf '%s\n' "$file_list"
      printf '%s\n' "$file_list" | git hash-object --stdin-paths
    } | git hash-object --stdin
  )
}

fn0_worker_remote_source_label() {
  local reg="$1" tag="$2"
  registry_inspect_image_label \
    "$(jq -r .url <<<"$reg")" \
    "$(jq -r .repository <<<"$reg")" \
    "$tag" \
    "$(jq -r .username <<<"$reg")" \
    "$(jq -r .password <<<"$reg")" \
    "$FN0_WORKER_SOURCE_LABEL"
}

fn0_worker_remote_manifest_exists() {
  local reg="$1" tag="$2"
  registry_inspect_manifest_exists \
    "$(jq -r .url <<<"$reg")" \
    "$(jq -r .repository <<<"$reg")" \
    "$tag" \
    "$(jq -r .username <<<"$reg")" \
    "$(jq -r .password <<<"$reg")"
}

fn0_worker_registries_json() {
  local registries_json
  registries_json="$(pulumi_pick_json workerImageRegistries)"
  if [[ -z "$registries_json" || "$registries_json" == "null" ]]; then
    echo "missing pulumi output: workerImageRegistries" >&2
    return 1
  fi
  if [[ "$(jq length <<<"$registries_json")" -eq 0 ]]; then
    echo "no worker image registries configured" >&2
    return 1
  fi
  printf '%s' "$registries_json"
}

fn0_worker_registry_login() {
  local reg="$1"
  local url username password
  url="$(jq -r .url <<<"$reg")"
  username="$(jq -r .username <<<"$reg")"
  password="$(jq -r .password <<<"$reg")"
  echo ">> ${CONTAINER_RUNTIME_CLI} login ${url}"
  echo "$password" | container_runtime_registry_login "$url" "$username"
}

fn0_worker_version() {
  local version
  version="$(cd "$REPO_ROOT" && cargo pkgid -p fn0-worker | sed -E 's/.*[#@]([^:]+)$/\1/')"
  if [[ -z "$version" ]]; then
    echo "failed to determine fn0-worker version" >&2
    return 1
  fi
  printf '%s' "$version"
}

resolve_fn0_worker_image_ref() {
  local tag="${1:?usage: resolve_fn0_worker_image_ref <version>}"
  local registries_json reg url repo full_ref
  registries_json="$(fn0_worker_registries_json)" || return 1
  reg="$(jq -c '.[0]' <<<"$registries_json")"
  url="$(jq -r .url <<<"$reg")"
  repo="$(jq -r .repository <<<"$reg")"
  full_ref="${url}/${repo}:${tag}"

  local manifest_status
  if fn0_worker_remote_manifest_exists "$reg" "$tag"; then
    manifest_status=0
  else
    manifest_status=$?
  fi
  if [[ "$manifest_status" -eq 2 ]]; then
    echo "failed to query ${full_ref}" >&2
    return 1
  fi
  if [[ "$manifest_status" -eq 1 ]]; then
    echo "no published fn0-worker image for version ${tag} at ${full_ref}." >&2
    echo "run scripts/deploy-fn0-worker.sh to build and roll it out first." >&2
    return 1
  fi

  FN0_WORKER_IMAGE_REF="$full_ref"
  echo ">> resolved published worker image ${FN0_WORKER_IMAGE_REF}"
}

build_and_push_fn0_worker() {
  local registries_json source_hash version tag
  registries_json="$(fn0_worker_registries_json)" || return 1
  source_hash="$(fn0_worker_source_hash)" || return 1
  version="$(fn0_worker_version)" || return 1
  tag="$version"
  echo ">> fn0-worker version: ${version}"
  echo ">> fn0-worker source: ${source_hash}"

  local count first_image_ref="" registry_index reg url repo full_ref remote_source manifest_status
  local -a refs_to_push=() registries_to_push=()
  count="$(jq length <<<"$registries_json")"
  for registry_index in $(seq 0 $((count - 1))); do
    reg="$(jq -c ".[$registry_index]" <<<"$registries_json")"
    url="$(jq -r .url <<<"$reg")"
    repo="$(jq -r .repository <<<"$reg")"
    full_ref="${url}/${repo}:${tag}"
    if [[ -z "$first_image_ref" ]]; then
      first_image_ref="$full_ref"
    fi

    if fn0_worker_remote_manifest_exists "$reg" "$tag"; then
      manifest_status=0
    else
      manifest_status=$?
    fi
    if [[ "$manifest_status" -eq 2 ]]; then
      echo "failed to query ${full_ref}" >&2
      return 1
    fi
    if [[ "$manifest_status" -eq 1 ]]; then
      refs_to_push+=("$full_ref")
      registries_to_push+=("$reg")
      continue
    fi

    remote_source="$(fn0_worker_remote_source_label "$reg" "$tag")"
    if [[ "$remote_source" == "$source_hash" ]]; then
      echo "   ${full_ref}: same source (skip push)"
    elif [[ -z "$remote_source" ]]; then
      echo "   ${full_ref}: no ${FN0_WORKER_SOURCE_LABEL} label, pushed before source stamping."
      echo "   leaving it in place; the next fn0-worker version bump publishes a stamped image."
    else
      echo "ERROR: ${full_ref} was built from different source:" >&2
      echo "       local  = ${source_hash}" >&2
      echo "       remote = ${remote_source}" >&2
      echo "       bump the fn0-worker version rather than overwriting a published tag." >&2
      return 1
    fi
  done

  if [[ "${#refs_to_push[@]}" -eq 0 ]]; then
    FN0_WORKER_PUSHED_IMAGE_REF="$first_image_ref"
    echo ">> nothing to push. image_ref=${FN0_WORKER_PUSHED_IMAGE_REF}"
    return 0
  fi

  local publish_log build_log bin_dir
  publish_log="$(mktemp)"
  build_log="$(mktemp)"
  bin_dir="$(mktemp -d)"
  trap 'rm -f "$publish_log" "$build_log"; rm -rf "$bin_dir"' RETURN

  echo ">> cargo publish fn0-worker"
  if (cd "$REPO_ROOT" && cargo publish -p fn0-worker) 2>&1 | tee "$publish_log"; then
    echo "   published."
  else
    if grep -qE "already (uploaded|exists)" "$publish_log"; then
      echo "   already published, continuing."
    else
      echo "cargo publish failed" >&2
      return 1
    fi
  fi

  "${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh" fn0-worker "$bin_dir"

  echo ">> build runtime image (${CONTAINER_RUNTIME_CLI})"
  if ! container_runtime_build_image \
    "${REPO_ROOT}/fn0/worker/Dockerfile" \
    "$bin_dir" \
    "$build_log" \
    --label "${FN0_WORKER_SOURCE_LABEL}=${source_hash}"; then
    echo "runtime image build failed" >&2
    return 1
  fi

  local push_index
  for push_index in "${!refs_to_push[@]}"; do
    full_ref="${refs_to_push[$push_index]}"
    reg="${registries_to_push[$push_index]}"
    fn0_worker_registry_login "$reg"
    echo ">> push ${full_ref}"
    container_runtime_tag "$CONTAINER_RUNTIME_BUILT_IMAGE" "$full_ref"
    container_runtime_push "$full_ref"
  done

  FN0_WORKER_PUSHED_IMAGE_REF="$first_image_ref"
  echo ">> done. image_ref=${FN0_WORKER_PUSHED_IMAGE_REF}"
}
