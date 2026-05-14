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
need cargo
need docker
need oci

# shellcheck source=lib/ocir-cleanup.sh
source "${REPO_ROOT}/scripts/lib/ocir-cleanup.sh"
# shellcheck source=lib/pulumi-outputs.sh
source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"

load_pulumi_outputs

REGISTRIES_JSON="$(pulumi_pick_json workerImageRegistries)"
if [[ -z "$REGISTRIES_JSON" || "$REGISTRIES_JSON" == "null" ]]; then
  echo "missing Pulumi output: workerImageRegistries" >&2
  exit 1
fi

WORKER_COMPARTMENT_ID="$(pulumi_pick workerCompartmentId)"

echo ">> Attempting cargo publish for fn0-worker-proxy"
PUBLISH_LOG="$(mktemp)"
IID_FILE="$(mktemp)"
INSPECT_LOG="$(mktemp)"
BUILD_LOG="$(mktemp)"
cleanup() { rm -f "$PUBLISH_LOG" "$IID_FILE" "$INSPECT_LOG" "$BUILD_LOG"; }
trap cleanup EXIT

if (cd "${REPO_ROOT}" && cargo publish -p fn0-worker-proxy) 2>&1 | tee "$PUBLISH_LOG"; then
  echo "   published."
else
  if grep -qE "already (uploaded|exists)" "$PUBLISH_LOG"; then
    echo "   already published, continuing."
  else
    echo "cargo publish failed (see log above)" >&2
    exit 1
  fi
fi

VERSION="$(cd "${REPO_ROOT}" && cargo pkgid -p fn0-worker-proxy | sed -E 's/.*[#@]([^:]+)$/\1/')"
if [[ -z "$VERSION" ]]; then
  echo "failed to determine fn0-worker-proxy version" >&2
  exit 1
fi
TAG="$VERSION"

echo ">> fn0-worker-proxy version: ${VERSION}"
echo ">> Building local image (host-native)"

DOCKER_BUILD_PIPESTATUS=0
set +e
docker build \
  --file "${REPO_ROOT}/fn0/worker-proxy/Dockerfile" \
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
  REPO="${REPO_BASE}-proxy"
  FULL_REF="${URL}/${REPO}:${TAG}"
  LATEST_REF="${URL}/${REPO}:latest"

  echo ">> Login ${URL}"
  echo "$PASSWORD" | docker login "$URL" -u "$USERNAME" --password-stdin >/dev/null

  if docker manifest inspect "$FULL_REF" >"$INSPECT_LOG" 2>&1; then
    if jq -e '.manifests' <"$INSPECT_LOG" >/dev/null 2>&1; then
      ARM64_DIGEST="$(jq -r '.manifests[] | select(.platform.architecture == "arm64" and .platform.os == "linux") | .digest' <"$INSPECT_LOG" | head -1)"
      if [[ -z "$ARM64_DIGEST" ]]; then
        echo "ERROR: ${FULL_REF} is a manifest list with no linux/arm64 entry" >&2
        exit 1
      fi
      if ! docker manifest inspect "${URL}/${REPO}@${ARM64_DIGEST}" >"$INSPECT_LOG" 2>&1; then
        cat "$INSPECT_LOG" >&2
        echo "failed to inspect arm64 manifest of ${FULL_REF}" >&2
        exit 1
      fi
    fi
    REMOTE_CONFIG_DIGEST="$(jq -r .config.digest <"$INSPECT_LOG")"
    if [[ "$LOCAL_CONFIG_DIGEST" == "$REMOTE_CONFIG_DIGEST" ]]; then
      echo "   ${FULL_REF}: match (skip push)"
    else
      echo "ERROR: ${FULL_REF} exists but image config digest differs:" >&2
      echo "       local  = ${LOCAL_CONFIG_DIGEST}" >&2
      echo "       remote = ${REMOTE_CONFIG_DIGEST}" >&2
      exit 1
    fi
  else
    if grep -qiE "no such manifest|not found|manifest unknown" "$INSPECT_LOG"; then
      echo ">> Pushing ${FULL_REF}"
      docker tag "$LOCAL_IID" "$FULL_REF"
      docker push "$FULL_REF"
    else
      cat "$INSPECT_LOG" >&2
      echo "docker manifest inspect failed for ${FULL_REF}" >&2
      exit 1
    fi
  fi

  echo ">> Pushing ${LATEST_REF} (mutable; pulled on proxy restart)"
  docker tag "$LOCAL_IID" "$LATEST_REF"
  docker push "$LATEST_REF"

  if [[ "$URL" == *oraclecloud.com* ]]; then
    if [[ -z "$WORKER_COMPARTMENT_ID" ]]; then
      echo "warn: workerCompartmentId not in Pulumi outputs; skipping OCIR cleanup for ${URL}/${REPO}" >&2
    else
      ocir_cleanup_keep_top_semver \
        --registry-url   "$URL" \
        --repository     "$REPO" \
        --compartment-id "$WORKER_COMPARTMENT_ID" \
        --keep           2 || echo "warn: cleanup failed for ${URL}/${REPO}" >&2
    fi
  fi
done

echo ">> Done. TAG=${TAG} (mutable :latest also updated)"
