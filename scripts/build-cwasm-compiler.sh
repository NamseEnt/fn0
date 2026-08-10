#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PULUMI_DIR="${PULUMI_DIR:-${REPO_ROOT}/infra/cloud}"

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "required command not found: $1" >&2; exit 1; }
}
# shellcheck source=lib/container-runtime.sh
source "${REPO_ROOT}/scripts/lib/container-runtime.sh"

need pulumi
need jq
need cargo
need aws
container_runtime_ensure_available

echo ">> Reading Pulumi stack outputs from ${PULUMI_DIR}"
OUT="$(cd "$PULUMI_DIR" && pulumi stack output --show-secrets --json)"

pick() { jq -r ".${1} // empty" <<<"$OUT"; }

CWASM_REGION="$(pick cwasmCompilerBucketRegion)"
CWASM_BUCKET="$(pick cwasmCompilerBucket)"
CWASM_ECR="$(pick cwasmCompilerEcrRepository)"
CWASM_ROLE_ARN="$(pick cwasmCompilerRoleArn)"
BUILDER_AWS_ACCESS_KEY_ID="$(pick cwasmCompilerBuilderAccessKeyId)"
BUILDER_AWS_SECRET_ACCESS_KEY="$(pick cwasmCompilerBuilderSecretAccessKey)"

R2_ENDPOINT="$(pick bundleStoreR2Endpoint)"
R2_ACCESS_KEY_ID="$(pick bundleStoreR2AccessKeyId)"
R2_SECRET_ACCESS_KEY="$(pick bundleStoreR2SecretAccessKey)"

for v in CWASM_REGION CWASM_BUCKET CWASM_ECR CWASM_ROLE_ARN \
         BUILDER_AWS_ACCESS_KEY_ID BUILDER_AWS_SECRET_ACCESS_KEY \
         R2_ENDPOINT R2_ACCESS_KEY_ID R2_SECRET_ACCESS_KEY; do
  if [[ -z "${!v}" ]]; then
    echo "missing Pulumi output for: ${v}" >&2
    exit 1
  fi
done

export AWS_ACCESS_KEY_ID="$BUILDER_AWS_ACCESS_KEY_ID"
export AWS_SECRET_ACCESS_KEY="$BUILDER_AWS_SECRET_ACCESS_KEY"
export AWS_DEFAULT_REGION="$CWASM_REGION"
unset AWS_SESSION_TOKEN AWS_PROFILE

echo ">> Attempting cargo publish for fn0-wasmtime"
PUBLISH_LOG="$(mktemp)"
BUILD_CTX="$(mktemp -d)"
trap 'rm -f "$PUBLISH_LOG"; rm -rf "$BUILD_CTX"' EXIT

if (cd "${REPO_ROOT}" && cargo publish -p fn0-wasmtime) 2>&1 | tee "$PUBLISH_LOG"; then
  echo "   published."
else
  if grep -qE "already (uploaded|exists)" "$PUBLISH_LOG"; then
    echo "   already published, continuing."
  else
    echo "cargo publish failed (see log above)" >&2
    exit 1
  fi
fi

VERSION="$(cd "${REPO_ROOT}" && cargo pkgid -p fn0-wasmtime | sed -E 's/.*[#@]([^:]+)$/\1/')"
if [[ -z "$VERSION" ]]; then
  echo "failed to determine fn0-wasmtime version" >&2
  exit 1
fi
VERSION_DASH="${VERSION//./-}"
FUNCTION_NAME="fn0-cwasm-compiler-${VERSION_DASH}"
IMAGE_TAG="${VERSION_DASH}"
IMAGE_URI="${CWASM_ECR}:${IMAGE_TAG}"

echo ">> fn0-wasmtime version: ${VERSION}"
echo ">> Lambda function name: ${FUNCTION_NAME}"
echo ">> Image URI: ${IMAGE_URI}"

echo ">> Logging in to ECR"
ECR_REGISTRY="${CWASM_ECR%%/*}"
aws ecr get-login-password --region "$CWASM_REGION" \
  | container_runtime_registry_login "$ECR_REGISTRY" AWS

echo ">> Building fn0-wasmtime binary"
"${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh" fn0-wasmtime "$BUILD_CTX"
cp "${REPO_ROOT}/cwasm-compiler/package.json" "${REPO_ROOT}/cwasm-compiler/handler.mjs" "$BUILD_CTX/"

echo ">> Building & pushing image"
# Lambda rejects multi-entry indexes and attestation manifests, so the image
# is built single-platform and pushed platform-filtered.
container_runtime_build_image \
  "${REPO_ROOT}/cwasm-compiler/Dockerfile" \
  "$BUILD_CTX" \
  /dev/null \
  --platform linux/arm64
container_runtime_tag "$CONTAINER_RUNTIME_BUILT_IMAGE" "$IMAGE_URI"
container_runtime_push "$IMAGE_URI" --platform linux/arm64

ENV_VARS="Variables={BUCKET=${CWASM_BUCKET},XDG_CACHE_HOME=/tmp,HOME=/tmp,R2_ENDPOINT=${R2_ENDPOINT},R2_ACCESS_KEY_ID=${R2_ACCESS_KEY_ID},R2_SECRET_ACCESS_KEY=${R2_SECRET_ACCESS_KEY}}"

CREATE_LOG="$(mktemp)"
trap 'rm -f "$PUBLISH_LOG" "$CREATE_LOG"; rm -rf "$BUILD_CTX"' EXIT

if aws lambda get-function --region "$CWASM_REGION" --function-name "$FUNCTION_NAME" >/dev/null 2>&1; then
  echo ">> Lambda exists; updating code + config"
  aws lambda update-function-code \
    --region "$CWASM_REGION" \
    --function-name "$FUNCTION_NAME" \
    --image-uri "$IMAGE_URI" \
    >/dev/null
  until aws lambda update-function-configuration \
    --region "$CWASM_REGION" \
    --function-name "$FUNCTION_NAME" \
    --environment "$ENV_VARS" \
    >/dev/null 2>"$CREATE_LOG"; do
    if ! grep -q "ResourceConflictException" "$CREATE_LOG"; then
      cat "$CREATE_LOG" >&2
      exit 1
    fi
    echo "   update-function-configuration conflicting (code update still in progress); retry..."
    sleep 5
  done
  echo ">> Updated Lambda: ${FUNCTION_NAME}"
else
  echo ">> Creating Lambda function"
  aws lambda create-function \
    --region "$CWASM_REGION" \
    --function-name "$FUNCTION_NAME" \
    --package-type Image \
    --code "ImageUri=${IMAGE_URI}" \
    --role "$CWASM_ROLE_ARN" \
    --architectures arm64 \
    --timeout 60 \
    --memory-size 10240 \
    --environment "$ENV_VARS" \
    >"$CREATE_LOG" 2>&1 || { cat "$CREATE_LOG" >&2; exit 1; }
  echo ">> Created Lambda: ${FUNCTION_NAME}"
fi
