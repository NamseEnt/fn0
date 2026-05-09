# shellcheck shell=bash
# Worker image build & push. Source-only.
#
# build_and_push_fn0_worker
#   Idempotent. Publishes fn0-worker (skips if already), builds linux/arm64
#   image, pushes to every registry in pulumi.workerImageRegistries (skip if
#   remote config digest matches local), runs OCIR cleanup. Echoes the resulting
#   image_ref (first registry) on stdout via FN0_WORKER_PUSHED_IMAGE_REF.

if [[ -n "${__FN0_WORKER_IMAGE_LOADED:-}" ]]; then
  return 0
fi
__FN0_WORKER_IMAGE_LOADED=1

# shellcheck source=ocir-cleanup.sh
source "${REPO_ROOT}/scripts/lib/ocir-cleanup.sh"

build_and_push_fn0_worker() {
  local registries_json worker_compartment_id
  registries_json="$(pulumi_pick_json workerImageRegistries)"
  if [[ -z "$registries_json" || "$registries_json" == "null" ]]; then
    echo "missing pulumi output: workerImageRegistries" >&2
    return 1
  fi
  worker_compartment_id="$(pulumi_pick workerCompartmentId)"
  if [[ -z "$worker_compartment_id" ]]; then
    echo "missing pulumi output: workerCompartmentId" >&2
    return 1
  fi

  local publish_log iid_file inspect_log build_log
  publish_log="$(mktemp)"
  iid_file="$(mktemp)"
  inspect_log="$(mktemp)"
  build_log="$(mktemp)"
  trap 'rm -f "$publish_log" "$iid_file" "$inspect_log" "$build_log"' RETURN

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

  local version tag
  version="$(cd "$REPO_ROOT" && cargo pkgid -p fn0-worker | sed -E 's/.*[#@]([^:]+)$/\1/')"
  if [[ -z "$version" ]]; then
    echo "failed to determine fn0-worker version" >&2
    return 1
  fi
  tag="$version"
  echo ">> fn0-worker version: ${version}"

  local docker_status
  echo ">> docker build (host-native)"
  set +e
  docker build \
    --file "${REPO_ROOT}/fn0/worker/Dockerfile" \
    --iidfile "$iid_file" \
    --progress=plain \
    "$REPO_ROOT" 2>&1 | tee "$build_log"
  docker_status=("${PIPESTATUS[@]}")
  set -e
  if [[ "${docker_status[0]}" -ne 0 ]]; then
    echo "docker build failed (exit ${docker_status[0]})" >&2
    return "${docker_status[0]}"
  fi

  local local_iid local_config_digest
  local_iid="$(cat "$iid_file")"
  local_config_digest="$(grep -oE 'exporting config sha256:[a-f0-9]+' "$build_log" | head -1 | awk '{print $3}')"
  if [[ -z "$local_config_digest" ]]; then
    echo "failed to extract local image config digest" >&2
    return 1
  fi
  echo ">> local image config digest: ${local_config_digest}"

  local count first_image_ref=""
  count="$(jq length <<<"$registries_json")"
  if (( count == 0 )); then
    echo "no worker image registries configured" >&2
    return 1
  fi

  local i reg url username password repo full_ref arm64_digest remote_config_digest
  for i in $(seq 0 $((count - 1))); do
    reg="$(jq -c ".[$i]" <<<"$registries_json")"
    url="$(jq -r .url <<<"$reg")"
    username="$(jq -r .username <<<"$reg")"
    password="$(jq -r .password <<<"$reg")"
    repo="$(jq -r .repository <<<"$reg")"
    full_ref="${url}/${repo}:${tag}"

    echo ">> docker login ${url}"
    echo "$password" | docker login "$url" -u "$username" --password-stdin >/dev/null

    if docker manifest inspect "$full_ref" >"$inspect_log" 2>&1; then
      if jq -e '.manifests' <"$inspect_log" >/dev/null 2>&1; then
        arm64_digest="$(jq -r '.manifests[] | select(.platform.architecture == "arm64" and .platform.os == "linux") | .digest' <"$inspect_log" | head -1)"
        if [[ -z "$arm64_digest" ]]; then
          echo "ERROR: ${full_ref} is a manifest list with no linux/arm64 entry" >&2
          return 1
        fi
        if ! docker manifest inspect "${url}/${repo}@${arm64_digest}" >"$inspect_log" 2>&1; then
          cat "$inspect_log" >&2
          return 1
        fi
      fi
      remote_config_digest="$(jq -r .config.digest <"$inspect_log")"
      if [[ "$local_config_digest" == "$remote_config_digest" ]]; then
        echo "   ${full_ref}: match (skip push)"
      else
        echo "ERROR: ${full_ref} exists but image config digest differs:" >&2
        echo "       local  = ${local_config_digest}" >&2
        echo "       remote = ${remote_config_digest}" >&2
        return 1
      fi
    else
      if grep -qiE "no such manifest|not found|manifest unknown" "$inspect_log"; then
        echo ">> push ${full_ref}"
        docker tag "$local_iid" "$full_ref"
        docker push "$full_ref"
      else
        cat "$inspect_log" >&2
        return 1
      fi
    fi

    if [[ "$url" == *oraclecloud.com* ]]; then
      ocir_cleanup_keep_top_semver \
        --registry-url   "$url" \
        --repository     "$repo" \
        --compartment-id "$worker_compartment_id" \
        --keep           2 || echo "warn: cleanup failed for ${url}/${repo}" >&2
    fi

    if [[ -z "$first_image_ref" ]]; then
      first_image_ref="$full_ref"
    fi
  done

  FN0_WORKER_PUSHED_IMAGE_REF="$first_image_ref"
  echo ">> done. image_ref=${FN0_WORKER_PUSHED_IMAGE_REF}"
}
