#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PULUMI_DIR="${PULUMI_DIR:-${REPO_ROOT}/infra/cloud}"

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "required command not found: $1" >&2; exit 1; }
}
need pulumi
need jq
need cargo
need docker

echo ">> Reading Pulumi stack outputs from ${PULUMI_DIR}"
OUT="$(cd "$PULUMI_DIR" && pulumi stack output --show-secrets --json)"
pick() { jq -r ".${1} // empty" <<<"$OUT"; }

REGISTRIES_JSON="$(jq -c '.workerImageRegistries' <<<"$OUT")"
if [[ -z "$REGISTRIES_JSON" || "$REGISTRIES_JSON" == "null" ]]; then
  echo "missing Pulumi output: workerImageRegistries" >&2
  exit 1
fi

SCCACHE_BUCKET="$(pick sccacheBucketName)"
SCCACHE_REGION="$(pick sccacheBucketRegion)"
SCCACHE_ENDPOINT="$(pick sccacheBucketEndpoint)"
SCCACHE_ACCESS_KEY_ID="$(pick sccacheAccessKeyId)"
SCCACHE_SECRET_ACCESS_KEY="$(pick sccacheSecretAccessKey)"

for v in SCCACHE_BUCKET SCCACHE_REGION SCCACHE_ENDPOINT SCCACHE_ACCESS_KEY_ID SCCACHE_SECRET_ACCESS_KEY; do
  if [[ -z "${!v}" ]]; then
    echo "missing Pulumi output: ${v}" >&2
    exit 1
  fi
done

echo ">> Attempting cargo publish for fn0-worker"
PUBLISH_LOG="$(mktemp)"
IID_FILE="$(mktemp)"
INSPECT_LOG="$(mktemp)"
cleanup() { rm -f "$PUBLISH_LOG" "$IID_FILE" "$INSPECT_LOG"; }
trap cleanup EXIT

if (cd "${REPO_ROOT}/fn0-worker" && cargo publish) 2>&1 | tee "$PUBLISH_LOG"; then
  echo "   published."
else
  if grep -qE "already (uploaded|exists)" "$PUBLISH_LOG"; then
    echo "   already published, continuing."
  else
    echo "cargo publish failed (see log above)" >&2
    exit 1
  fi
fi

VERSION="$(cd "${REPO_ROOT}/fn0-worker" && cargo pkgid | sed -E 's/.*[#@]([^:]+)$/\1/')"
if [[ -z "$VERSION" ]]; then
  echo "failed to determine fn0-worker version" >&2
  exit 1
fi
TAG="$VERSION"

echo ">> fn0-worker version: ${VERSION}"
echo ">> Building local image (host-native)"

docker build \
  --file "${REPO_ROOT}/fn0-worker/Dockerfile" \
  --iidfile "$IID_FILE" \
  --build-arg SCCACHE_BUCKET="$SCCACHE_BUCKET" \
  --build-arg SCCACHE_REGION="$SCCACHE_REGION" \
  --build-arg SCCACHE_ENDPOINT="$SCCACHE_ENDPOINT" \
  --build-arg AWS_ACCESS_KEY_ID="$SCCACHE_ACCESS_KEY_ID" \
  --build-arg AWS_SECRET_ACCESS_KEY="$SCCACHE_SECRET_ACCESS_KEY" \
  "$REPO_ROOT"

LOCAL_DIGEST="$(cat "$IID_FILE")"
echo ">> Local image config digest: ${LOCAL_DIGEST}"

COUNT="$(jq length <<<"$REGISTRIES_JSON")"
TO_PUSH=()
MISMATCHED=0

for i in $(seq 0 $((COUNT - 1))); do
  REG="$(jq -c ".[$i]" <<<"$REGISTRIES_JSON")"
  URL="$(jq -r .url <<<"$REG")"
  USERNAME="$(jq -r .username <<<"$REG")"
  PASSWORD="$(jq -r .password <<<"$REG")"
  REPO="$(jq -r .repository <<<"$REG")"
  FULL_REF="${URL}/${REPO}:${TAG}"

  echo ">> Login ${URL}"
  echo "$PASSWORD" | docker login "$URL" -u "$USERNAME" --password-stdin >/dev/null

  if docker manifest inspect "$FULL_REF" >"$INSPECT_LOG" 2>&1; then
    if jq -e '.manifests' <"$INSPECT_LOG" >/dev/null 2>&1; then
      ARM64_DIGEST="$(jq -r '.manifests[] | select(.platform.architecture == "arm64" and .platform.os == "linux") | .digest' <"$INSPECT_LOG" | head -1)"
      if [[ -z "$ARM64_DIGEST" ]]; then
        echo "ERROR: ${FULL_REF} is a manifest list with no linux/arm64 entry" >&2
        MISMATCHED=1
        continue
      fi
      if ! docker manifest inspect "${URL}/${REPO}@${ARM64_DIGEST}" >"$INSPECT_LOG" 2>&1; then
        cat "$INSPECT_LOG" >&2
        echo "failed to inspect arm64 manifest of ${FULL_REF}" >&2
        exit 1
      fi
    fi
    REMOTE_DIGEST="$(jq -r .config.digest <"$INSPECT_LOG")"
    if [[ "$LOCAL_DIGEST" == "$REMOTE_DIGEST" ]]; then
      echo "   ${FULL_REF}: match (will skip push)"
    else
      echo "ERROR: ${FULL_REF} exists but config digest differs:" >&2
      echo "       local  = ${LOCAL_DIGEST}" >&2
      echo "       remote = ${REMOTE_DIGEST}" >&2
      MISMATCHED=1
    fi
  else
    if grep -qiE "no such manifest|not found|manifest unknown" "$INSPECT_LOG"; then
      echo "   ${FULL_REF}: missing, queued for push"
      TO_PUSH+=("$FULL_REF")
    else
      cat "$INSPECT_LOG" >&2
      echo "docker manifest inspect failed for ${FULL_REF}" >&2
      exit 1
    fi
  fi
done

if (( MISMATCHED )); then
  echo "ABORTING: at least one registry already has a different image under tag '${TAG}'." >&2
  exit 1
fi

if (( ${#TO_PUSH[@]} == 0 )); then
  echo ">> All registries already have the matching image; nothing to push."
  exit 0
fi

for REF in "${TO_PUSH[@]}"; do
  echo ">> Pushing ${REF}"
  docker tag "$LOCAL_DIGEST" "$REF"
  docker push "$REF"
done

echo ">> Done."
