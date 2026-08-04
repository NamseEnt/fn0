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
  local full_ref="$1"
  docker buildx imagetools inspect "$full_ref" --format '{{json .Image}}' 2>/dev/null \
    | jq -r --arg label "$FN0_WORKER_SOURCE_LABEL" '
        (if has("config") then . else (to_entries[] | select(.key | endswith("linux/arm64")) | .value) end)
        | .config.Labels[$label] // empty' 2>/dev/null
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
  echo ">> docker login ${url}"
  echo "$password" | docker login "$url" -u "$username" --password-stdin >/dev/null
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

  fn0_worker_registry_login "$reg"
  if ! docker manifest inspect "$full_ref" >/dev/null 2>&1; then
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

  local count first_image_ref="" registry_index reg url repo full_ref remote_source
  local -a refs_to_push=()
  count="$(jq length <<<"$registries_json")"
  for registry_index in $(seq 0 $((count - 1))); do
    reg="$(jq -c ".[$registry_index]" <<<"$registries_json")"
    url="$(jq -r .url <<<"$reg")"
    repo="$(jq -r .repository <<<"$reg")"
    full_ref="${url}/${repo}:${tag}"
    if [[ -z "$first_image_ref" ]]; then
      first_image_ref="$full_ref"
    fi

    fn0_worker_registry_login "$reg"

    if ! docker manifest inspect "$full_ref" >/dev/null 2>&1; then
      refs_to_push+=("$full_ref")
      continue
    fi

    remote_source="$(fn0_worker_remote_source_label "$full_ref")"
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

  local publish_log iid_file build_log bin_dir
  publish_log="$(mktemp)"
  iid_file="$(mktemp)"
  build_log="$(mktemp)"
  bin_dir="$(mktemp -d)"
  trap 'rm -f "$publish_log" "$iid_file" "$build_log"; rm -rf "$bin_dir"' RETURN

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

  local docker_status
  echo ">> docker build runtime image"
  set +e
  docker build \
    --file "${REPO_ROOT}/fn0/worker/Dockerfile" \
    --label "${FN0_WORKER_SOURCE_LABEL}=${source_hash}" \
    --iidfile "$iid_file" \
    --progress=plain \
    "$bin_dir" 2>&1 | tee "$build_log"
  docker_status=("${PIPESTATUS[@]}")
  set -e
  if [[ "${docker_status[0]}" -ne 0 ]]; then
    echo "docker build failed (exit ${docker_status[0]})" >&2
    return "${docker_status[0]}"
  fi

  local local_iid
  local_iid="$(cat "$iid_file")"

  for full_ref in "${refs_to_push[@]}"; do
    echo ">> push ${full_ref}"
    docker tag "$local_iid" "$full_ref"
    docker push "$full_ref"
  done

  FN0_WORKER_PUSHED_IMAGE_REF="$first_image_ref"
  echo ">> done. image_ref=${FN0_WORKER_PUSHED_IMAGE_REF}"
}
