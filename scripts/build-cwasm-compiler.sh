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
need aws

echo ">> Reading Pulumi stack outputs from ${PULUMI_DIR}"
OUT="$(cd "$PULUMI_DIR" && pulumi stack output --show-secrets --json)"

pick() { jq -r ".${1} // empty" <<<"$OUT"; }

CWASM_REGION="$(pick cwasmCompilerBucketRegion)"
CWASM_BUCKET="$(pick cwasmCompilerBucket)"
CWASM_ECR="$(pick cwasmCompilerEcrRepository)"
CWASM_ROLE_ARN="$(pick cwasmCompilerRoleArn)"
BUILDER_AWS_ACCESS_KEY_ID="$(pick cwasmCompilerBuilderAccessKeyId)"
BUILDER_AWS_SECRET_ACCESS_KEY="$(pick cwasmCompilerBuilderSecretAccessKey)"

SCCACHE_BUCKET="$(pick sccacheBucketName)"
SCCACHE_REGION="$(pick sccacheBucketRegion)"
SCCACHE_ENDPOINT="$(pick sccacheBucketEndpoint)"
SCCACHE_ACCESS_KEY_ID="$(pick sccacheAccessKeyId)"
SCCACHE_SECRET_ACCESS_KEY="$(pick sccacheSecretAccessKey)"

R2_ENDPOINT="$(pick controlR2Endpoint)"
R2_ACCESS_KEY_ID="$(pick controlR2AccessKeyId)"
R2_SECRET_ACCESS_KEY="$(pick controlR2SecretAccessKey)"

for v in CWASM_REGION CWASM_BUCKET CWASM_ECR CWASM_ROLE_ARN \
         BUILDER_AWS_ACCESS_KEY_ID BUILDER_AWS_SECRET_ACCESS_KEY \
         SCCACHE_BUCKET SCCACHE_REGION SCCACHE_ENDPOINT \
         SCCACHE_ACCESS_KEY_ID SCCACHE_SECRET_ACCESS_KEY \
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
trap 'rm -f "$PUBLISH_LOG"' EXIT

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
  | docker login --username AWS --password-stdin "$ECR_REGISTRY"

echo ">> Building & pushing image"
docker buildx build \
  --platform linux/arm64 \
  --provenance=false \
  --sbom=false \
  --file "${REPO_ROOT}/cwasm-compiler/Dockerfile" \
  --build-arg SCCACHE_BUCKET="$SCCACHE_BUCKET" \
  --build-arg SCCACHE_REGION="$SCCACHE_REGION" \
  --build-arg SCCACHE_ENDPOINT="$SCCACHE_ENDPOINT" \
  --build-arg AWS_ACCESS_KEY_ID="$SCCACHE_ACCESS_KEY_ID" \
  --build-arg AWS_SECRET_ACCESS_KEY="$SCCACHE_SECRET_ACCESS_KEY" \
  --tag "$IMAGE_URI" \
  --push \
  "$REPO_ROOT"

ENV_VARS="Variables={BUCKET=${CWASM_BUCKET},XDG_CACHE_HOME=/tmp,HOME=/tmp,R2_ENDPOINT=${R2_ENDPOINT},R2_ACCESS_KEY_ID=${R2_ACCESS_KEY_ID},R2_SECRET_ACCESS_KEY=${R2_SECRET_ACCESS_KEY}}"

CREATE_LOG="$(mktemp)"
trap 'rm -f "$PUBLISH_LOG" "$CREATE_LOG"' EXIT

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
