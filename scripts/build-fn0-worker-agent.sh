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
need docker

# shellcheck source=lib/pulumi-outputs.sh
source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"

load_pulumi_outputs

REGISTRIES_JSON="$(pulumi_pick_json workerImageRegistries)"
if [[ -z "$REGISTRIES_JSON" || "$REGISTRIES_JSON" == "null" ]]; then
  echo "missing Pulumi output: workerImageRegistries" >&2
  exit 1
fi

IID_FILE="$(mktemp)"
INSPECT_LOG="$(mktemp)"
BUILD_LOG="$(mktemp)"
cleanup() { rm -f "$IID_FILE" "$INSPECT_LOG" "$BUILD_LOG"; }
trap cleanup EXIT

echo ">> Building local image (host-native)"

DOCKER_BUILD_PIPESTATUS=0
set +e
docker build \
  --file "${REPO_ROOT}/fn0/worker-agent/Dockerfile" \
  --iidfile "$IID_FILE" \
  --progress=plain \
  "$REPO_ROOT" 2>&1 | tee "$BUILD_LOG"
DOCKER_BUILD_PIPESTATUS=("${PIPESTATUS[@]}")
set -e
if [[ "${DOCKER_BUILD_PIPESTATUS[0]}" -ne 0 ]]; then
  echo "docker build failed (exit ${DOCKER_BUILD_PIPESTATUS[0]})" >&2
  exit "${DOCKER_BUILD_PIPESTATUS[0]}"
fi

LOCAL_IID="$(cat "$IID_FILE")"
LOCAL_CONFIG_DIGEST="$(grep -oE 'exporting config sha256:[a-f0-9]+' "$BUILD_LOG" | head -1 | awk '{print $3}')"
if [[ -z "$LOCAL_CONFIG_DIGEST" ]]; then
  echo "failed to extract local image config digest from build log" >&2
  exit 1
fi
echo ">> Local image config digest: ${LOCAL_CONFIG_DIGEST}"

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

  echo ">> Login ${URL}"
  echo "$PASSWORD" | docker login "$URL" -u "$USERNAME" --password-stdin >/dev/null

  PUSH=1
  if docker manifest inspect "$LATEST_REF" >"$INSPECT_LOG" 2>&1; then
    if jq -e '.manifests' <"$INSPECT_LOG" >/dev/null 2>&1; then
      ARM64_DIGEST="$(jq -r '.manifests[] | select(.platform.architecture == "arm64" and .platform.os == "linux") | .digest' <"$INSPECT_LOG" | head -1)"
      if [[ -z "$ARM64_DIGEST" ]]; then
        echo "ERROR: ${LATEST_REF} is a manifest list with no linux/arm64 entry" >&2
        exit 1
      fi
      if ! docker manifest inspect "${URL}/${REPO}@${ARM64_DIGEST}" >"$INSPECT_LOG" 2>&1; then
        cat "$INSPECT_LOG" >&2
        echo "failed to inspect arm64 manifest of ${LATEST_REF}" >&2
        exit 1
      fi
    fi
    REMOTE_CONFIG_DIGEST="$(jq -r .config.digest <"$INSPECT_LOG")"
    if [[ "$LOCAL_CONFIG_DIGEST" == "$REMOTE_CONFIG_DIGEST" ]]; then
      echo "   ${LATEST_REF}: match (skip push)"
      PUSH=0
    fi
  elif ! grep -qiE "no such manifest|not found|manifest unknown" "$INSPECT_LOG"; then
    cat "$INSPECT_LOG" >&2
    echo "docker manifest inspect failed for ${LATEST_REF}" >&2
    exit 1
  fi

  if [[ "$PUSH" -eq 1 ]]; then
    echo ">> Pushing ${LATEST_REF} (mutable; running agents poll this)"
    docker tag "$LOCAL_IID" "$LATEST_REF"
    docker push "$LATEST_REF"
  fi
done

echo ">> Done. :latest updated."
