#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT
PULUMI_DIR="${PULUMI_DIR:-${REPO_ROOT}/infra/cloud}"

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "required command not found: $1" >&2; exit 1; }
}
need pulumi
need jq

# shellcheck source=lib/pulumi-outputs.sh
source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"
# shellcheck source=lib/container-runtime.sh
source "${REPO_ROOT}/scripts/lib/container-runtime.sh"

container_runtime_ensure_available
load_pulumi_outputs

REGISTRIES_JSON="$(pulumi_pick_json workerImageRegistries)"
if [[ -z "$REGISTRIES_JSON" || "$REGISTRIES_JSON" == "null" ]]; then
  echo "missing Pulumi output: workerImageRegistries" >&2
  exit 1
fi

BUILD_LOG="$(mktemp)"
BIN_DIR="$(mktemp -d)"
cleanup() { rm -f "$BUILD_LOG"; rm -rf "$BIN_DIR"; }
trap cleanup EXIT

"${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh" fn0-worker-agent "$BIN_DIR"

echo ">> build runtime image (${CONTAINER_RUNTIME_CLI})"
container_runtime_build_image \
  "${REPO_ROOT}/fn0/worker-agent/Dockerfile" \
  "$BIN_DIR" \
  "$BUILD_LOG"

LOCAL_IDENTITY="$(container_runtime_built_image_identity "$BUILD_LOG")"
if [[ -z "$LOCAL_IDENTITY" ]]; then
  echo "failed to determine local image identity" >&2
  exit 1
fi
echo ">> Local image identity: ${LOCAL_IDENTITY}"

COUNT="$(jq length <<<"$REGISTRIES_JSON")"
if (( COUNT == 0 )); then
  echo "no worker image registries configured" >&2
  exit 1
fi

for i in $(seq 0 $((COUNT - 1))); do
  REG="$(jq -c ".[$i]" <<<"$REGISTRIES_JSON")"
  URL="$(jq -r .url <<<"$REG")"
  USERNAME="$(jq -r .username <<<"$REG")"
  PASSWORD="$(jq -r .password <<<"$REG")"
  REPO_BASE="$(jq -r .repository <<<"$REG")"
  REPO="${REPO_BASE}-agent"
  LATEST_REF="${URL}/${REPO}:latest"

  PUSH=1
  if registry_inspect_manifest_exists "$URL" "$REPO" "latest" "$USERNAME" "$PASSWORD"; then
    MANIFEST_STATUS=0
  else
    MANIFEST_STATUS=$?
  fi
  if [[ "$MANIFEST_STATUS" -eq 2 ]]; then
    echo "failed to query ${LATEST_REF}" >&2
    exit 1
  fi
  if [[ "$MANIFEST_STATUS" -eq 0 ]]; then
    REMOTE_IDENTITY="$(container_runtime_remote_image_identity "$URL" "$REPO" "latest" "$USERNAME" "$PASSWORD")"
    if [[ "$LOCAL_IDENTITY" == "$REMOTE_IDENTITY" ]]; then
      echo "   ${LATEST_REF}: match (skip push)"
      PUSH=0
    fi
  fi

  if [[ "$PUSH" -eq 1 ]]; then
    echo ">> Login ${URL}"
    echo "$PASSWORD" | container_runtime_registry_login "$URL" "$USERNAME"
    echo ">> Pushing ${LATEST_REF} (mutable; running agents poll this)"
    container_runtime_tag "$CONTAINER_RUNTIME_BUILT_IMAGE" "$LATEST_REF"
    container_runtime_push "$LATEST_REF"
  fi
done

echo ">> Done. :latest updated."
