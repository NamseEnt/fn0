# shellcheck shell=bash
# Container runtime selection and primitives. Source-only.
#
# macOS runs apple/container so Docker Desktop is not required there; every
# other OS runs docker. Override with FN0_CONTAINER_RUNTIME=docker|container.
#
# Remote-registry questions (tag existence, labels, digests) go through
# lib/registry-inspect.sh over HTTP on both runtimes; only local operations
# (build, run, tag, push, login) go through the runtime CLI.

if [[ -n "${__FN0_CONTAINER_RUNTIME_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTAINER_RUNTIME_LOADED=1

# shellcheck source=registry-inspect.sh
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/registry-inspect.sh"

if [[ -n "${FN0_CONTAINER_RUNTIME:-}" ]]; then
  CONTAINER_RUNTIME_CLI="$FN0_CONTAINER_RUNTIME"
elif [[ "$(uname -s)" == "Darwin" ]]; then
  CONTAINER_RUNTIME_CLI="container"
else
  CONTAINER_RUNTIME_CLI="docker"
fi

container_runtime_ensure_available() {
  if ! command -v "$CONTAINER_RUNTIME_CLI" >/dev/null 2>&1; then
    echo "required command not found: ${CONTAINER_RUNTIME_CLI}" >&2
    return 1
  fi
  case "$CONTAINER_RUNTIME_CLI" in
    container)
      if ! container system status >/dev/null 2>&1; then
        echo ">> starting apple/container system service"
        container system start
      fi
      ;;
    docker)
      if ! docker info >/dev/null 2>&1; then
        echo "docker daemon is not running" >&2
        return 1
      fi
      ;;
    *)
      echo "unsupported FN0_CONTAINER_RUNTIME: ${CONTAINER_RUNTIME_CLI}" >&2
      return 1
      ;;
  esac
}

container_runtime_volume_create() {
  local volume_name="$1"
  case "$CONTAINER_RUNTIME_CLI" in
    container)
      # apple/container errors on an existing volume where docker no-ops.
      if ! container volume inspect "$volume_name" >/dev/null 2>&1; then
        container volume create "$volume_name" >/dev/null
      fi
      ;;
    docker)
      docker volume create "$volume_name" >/dev/null
      ;;
  esac
}

container_runtime_run() {
  "$CONTAINER_RUNTIME_CLI" run "$@"
}

# Fills CONTAINER_RUNTIME_FULL_HOST_RESOURCE_ARGS with the flags a
# whole-machine build container needs. apple/container boots one VM per
# container with small defaults (4 cpus / 1 GiB) that OOM a workspace cargo
# build; docker containers share the daemon's resources, so no flags there.
container_runtime_set_full_host_resources() {
  # shellcheck disable=SC2034  # consumed by scripts that source this lib
  CONTAINER_RUNTIME_FULL_HOST_RESOURCE_ARGS=()
  if [[ "$CONTAINER_RUNTIME_CLI" == "container" ]]; then
    local host_memory_bytes host_memory_gigabytes
    host_memory_bytes="$(sysctl -n hw.memsize)"
    host_memory_gigabytes=$((host_memory_bytes / 1073741824))
    CONTAINER_RUNTIME_FULL_HOST_RESOURCE_ARGS=(
      --cpus "$(sysctl -n hw.ncpu)"
      --memory "$((host_memory_gigabytes * 3 / 4))G"
    )
  fi
}

# container_runtime_registry_login <registry_url> <username>  (password on stdin)
container_runtime_registry_login() {
  local registry_url="$1" username="$2"
  case "$CONTAINER_RUNTIME_CLI" in
    container)
      container registry login --username "$username" --password-stdin "$registry_url" >/dev/null
      ;;
    docker)
      docker login "$registry_url" -u "$username" --password-stdin >/dev/null
      ;;
  esac
}

# container_runtime_build_image <dockerfile> <context_dir> <build_log> [--label key=value]... [--platform os/arch]
# Streams build output to stdout and <build_log>, and sets
# CONTAINER_RUNTIME_BUILT_IMAGE to a reference container_runtime_tag/push
# accept. Returns the build's exit status.
CONTAINER_RUNTIME_BUILT_IMAGE=""

container_runtime_build_image() {
  local dockerfile="$1" context_dir="$2" build_log="$3"
  shift 3
  local -a label_args=() platform_args=()
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --label)
        label_args+=(--label "$2")
        shift 2
        ;;
      --platform)
        platform_args+=(--platform "$2")
        shift 2
        ;;
      *)
        echo "container_runtime_build_image: unknown option: $1" >&2
        return 1
        ;;
    esac
  done

  local build_status
  CONTAINER_RUNTIME_BUILT_IMAGE=""
  case "$CONTAINER_RUNTIME_CLI" in
    docker)
      local image_id_file
      image_id_file="$(mktemp)"
      set +e
      docker build \
        --file "$dockerfile" \
        --iidfile "$image_id_file" \
        --progress=plain \
        ${label_args[@]+"${label_args[@]}"} \
        ${platform_args[@]+"${platform_args[@]}"} \
        "$context_dir" 2>&1 | tee "$build_log"
      build_status="${PIPESTATUS[0]}"
      set -e
      if [[ "$build_status" -eq 0 ]]; then
        CONTAINER_RUNTIME_BUILT_IMAGE="$(cat "$image_id_file")"
      fi
      rm -f "$image_id_file"
      ;;
    container)
      local build_tag
      build_tag="fn0-local-build:$(uuidgen | tr '[:upper:]' '[:lower:]')"
      set +e
      container build \
        --file "$dockerfile" \
        --tag "$build_tag" \
        --progress plain \
        ${label_args[@]+"${label_args[@]}"} \
        ${platform_args[@]+"${platform_args[@]}"} \
        "$context_dir" 2>&1 | tee "$build_log"
      build_status="${PIPESTATUS[0]}"
      set -e
      if [[ "$build_status" -eq 0 ]]; then
        CONTAINER_RUNTIME_BUILT_IMAGE="$build_tag"
      fi
      ;;
  esac
  return "$build_status"
}

container_runtime_tag() {
  local source_reference="$1" target_reference="$2"
  case "$CONTAINER_RUNTIME_CLI" in
    container) container image tag "$source_reference" "$target_reference" ;;
    docker) docker tag "$source_reference" "$target_reference" ;;
  esac
}

# container_runtime_push <reference> [--platform os/arch]
# --platform narrows an apple/container push to one entry of its local OCI
# index (Lambda rejects multi-entry indexes); docker-built images are
# single-arch already, so it is a no-op there.
container_runtime_push() {
  local reference="$1"
  shift
  local platform=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --platform)
        platform="$2"
        shift 2
        ;;
      *)
        echo "container_runtime_push: unknown option: $1" >&2
        return 1
        ;;
    esac
  done
  case "$CONTAINER_RUNTIME_CLI" in
    container)
      if [[ -n "$platform" ]]; then
        container image push --platform "$platform" "$reference"
      else
        container image push "$reference"
      fi
      ;;
    docker)
      docker push "$reference"
      ;;
  esac
}

# container_runtime_built_image_identity <build_log>
# Prints an identity for the image container_runtime_build_image just built,
# comparable against container_runtime_remote_image_identity for the same
# runtime. Empty output means the identity could not be determined.
container_runtime_built_image_identity() {
  local build_log="$1"
  case "$CONTAINER_RUNTIME_CLI" in
    docker)
      # BuildKit's --iidfile holds the config digest under the classic store
      # but the manifest digest under containerd snapshotters; the build log
      # is the one place the config digest appears in both setups.
      grep -oE 'exporting config sha256:[a-f0-9]+' "$build_log" | head -1 | awk '{print $3}'
      ;;
    container)
      container image inspect "$CONTAINER_RUNTIME_BUILT_IMAGE" \
        | jq -r '[.[0].variants[] | select(.platform.architecture == "arm64" and .platform.os == "linux")][0].digest // empty'
      ;;
  esac
}

# container_runtime_remote_image_identity <registry_url> <repository> <reference> <username> <password>
# Prints the remote counterpart of container_runtime_built_image_identity:
# docker compares config digests (the only pre-push identity docker exposes),
# apple/container compares manifest digests (its local store keeps the OCI
# manifest bytes that push uploads verbatim).
container_runtime_remote_image_identity() {
  local registry_url="$1" repository="$2" reference="$3" username="$4" password="$5"
  local manifest_file identity_status remote_identity
  manifest_file="$(mktemp)"
  identity_status=0
  case "$CONTAINER_RUNTIME_CLI" in
    docker)
      if registry_inspect_arm64_manifest "$registry_url" "$repository" "$reference" "$username" "$password" "$manifest_file" >/dev/null; then
        jq -r '.config.digest // empty' "$manifest_file"
      else
        identity_status=$?
      fi
      ;;
    container)
      if remote_identity="$(registry_inspect_arm64_manifest "$registry_url" "$repository" "$reference" "$username" "$password" "$manifest_file")"; then
        printf '%s\n' "$remote_identity"
      else
        identity_status=$?
      fi
      ;;
  esac
  rm -f "$manifest_file"
  return "$identity_status"
}
