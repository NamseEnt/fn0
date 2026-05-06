#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PULUMI_DIR="${PULUMI_DIR:-${REPO_ROOT}/infra/cloud}"
PARALLEL="${PARALLEL:-20}"

usage() {
  cat <<EOF
usage: $0 <new_fn0_wasmtime_version>

Idempotent. Builds fn0-cwasm-compiler-<ver-dashed>, registers <ver> as
control's pending fn0-wasmtime version (or as the very first active
version if no doc exists yet), and synchronously compiles every R2
/original/ object that is not yet under /compiled/<ver>/ on the new
lambda. Re-running picks up where it left off.

Once pending is set, control fans out all subsequent deploys to both
active and pending lambdas, so any deploy that lands during this script
is automatically caught up.

env:
  PARALLEL  parallel sync invokes (default: 20)
EOF
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

NEW_VERSION="$1"
NEW_VERSION_DASH="${NEW_VERSION//./-}"
FUNCTION_NAME="fn0-cwasm-compiler-${NEW_VERSION_DASH}"

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "required command not found: $1" >&2; exit 1; }
}
need pulumi
need jq
need aws
need docker
need curl

echo ">> Reading Pulumi stack outputs from ${PULUMI_DIR}"
OUT="$(cd "$PULUMI_DIR" && pulumi stack output --show-secrets --json)"
pick() { jq -r ".${1} // empty" <<<"$OUT"; }

CWASM_REGION="$(pick cwasmCompilerBucketRegion)"
CWASM_ECR="$(pick cwasmCompilerEcrRepository)"
CWASM_ROLE_ARN="$(pick cwasmCompilerRoleArn)"
BUILDER_AWS_ACCESS_KEY_ID="$(pick cwasmCompilerBuilderAccessKeyId)"
BUILDER_AWS_SECRET_ACCESS_KEY="$(pick cwasmCompilerBuilderSecretAccessKey)"
HQ_AWS_ACCESS_KEY_ID="$(pick hqAwsAccessKeyId)"
HQ_AWS_SECRET_ACCESS_KEY="$(pick hqAwsSecretAccessKey)"
R2_ENDPOINT="$(pick bundleStoreR2Endpoint)"
R2_BUCKET="$(pick bundleStoreR2BucketName)"
R2_ACCESS_KEY_ID="$(pick bundleStoreR2AccessKeyId)"
R2_SECRET_ACCESS_KEY="$(pick bundleStoreR2SecretAccessKey)"
CONTROL_URL="$(pick controlUrl)"
ADMIN_TOKEN="$(pick controlAdminTokenBase64)"

for v in CWASM_REGION CWASM_ECR CWASM_ROLE_ARN \
         BUILDER_AWS_ACCESS_KEY_ID BUILDER_AWS_SECRET_ACCESS_KEY \
         HQ_AWS_ACCESS_KEY_ID HQ_AWS_SECRET_ACCESS_KEY \
         R2_ENDPOINT R2_BUCKET R2_ACCESS_KEY_ID R2_SECRET_ACCESS_KEY \
         CONTROL_URL ADMIN_TOKEN; do
  if [[ -z "${!v}" ]]; then
    echo "missing Pulumi output: ${v}" >&2
    exit 1
  fi
done

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

echo ">> NEW_VERSION=${NEW_VERSION}"
echo ">> FUNCTION_NAME=${FUNCTION_NAME}"

IMAGE_TAG="${NEW_VERSION_DASH}"
IMAGE_URI="${CWASM_ECR}:${IMAGE_TAG}"

call_set_pending() {
  local resp http outcome
  resp="${WORK_DIR}/set_pending_resp.json"
  http="$(curl -sS -o "$resp" -w '%{http_code}' \
    -X POST "${CONTROL_URL%/}/__forte_action/set_pending_fn0_wasmtime" \
    -H "Authorization: Bearer ${ADMIN_TOKEN}" \
    -H "Content-Type: application/json" \
    --data "$(jq -nc --arg v "$NEW_VERSION" '{version: $v}')")"
  if [[ "$http" != "200" ]]; then
    echo "control set_pending HTTP ${http}" >&2
    cat "$resp" >&2
    return 1
  fi
  outcome="$(jq -r 'if type=="string" then . else (keys[0]) end' < "$resp")"
  case "$outcome" in
    Ok) ;;
    Unauthorized) echo "admin token rejected" >&2; return 1 ;;
    *) echo "unexpected set_pending response:" >&2; cat "$resp" >&2; return 1 ;;
  esac
}

list_r2() {
  local prefix="$1"
  local token=""
  local out=""
  while :; do
    if [[ -n "$token" ]]; then
      out="$(AWS_ACCESS_KEY_ID="$R2_ACCESS_KEY_ID" \
             AWS_SECRET_ACCESS_KEY="$R2_SECRET_ACCESS_KEY" \
             AWS_DEFAULT_REGION=auto \
             aws s3api list-objects-v2 \
               --endpoint-url "$R2_ENDPOINT" \
               --bucket "$R2_BUCKET" \
               --prefix "$prefix" \
               --starting-token "$token" \
               --output json)"
    else
      out="$(AWS_ACCESS_KEY_ID="$R2_ACCESS_KEY_ID" \
             AWS_SECRET_ACCESS_KEY="$R2_SECRET_ACCESS_KEY" \
             AWS_DEFAULT_REGION=auto \
             aws s3api list-objects-v2 \
               --endpoint-url "$R2_ENDPOINT" \
               --bucket "$R2_BUCKET" \
               --prefix "$prefix" \
               --output json)"
    fi
    jq -c '.Contents[]? | {Key, LastModified}' <<<"$out"
    token="$(jq -r '.NextContinuationToken // empty' <<<"$out")"
    [[ -z "$token" ]] && break
  done
}

normalize_lm() {
  # 2026-05-04T10:16:03.773000+00:00 -> 2026-05-04T10:16:03Z
  # 2026-05-04T10:16:03+00:00        -> 2026-05-04T10:16:03Z
  echo "$1" | sed -E 's/\.[0-9]+\+00:00$/Z/; s/\+00:00$/Z/'
}

# --- step 1: build image + create/update lambda + wait active ---
(
  export AWS_ACCESS_KEY_ID="$BUILDER_AWS_ACCESS_KEY_ID"
  export AWS_SECRET_ACCESS_KEY="$BUILDER_AWS_SECRET_ACCESS_KEY"
  export AWS_DEFAULT_REGION="$CWASM_REGION"
  unset AWS_SESSION_TOKEN AWS_PROFILE

  echo ">> Logging in to ECR"
  ECR_REGISTRY="${CWASM_ECR%%/*}"
  aws ecr get-login-password --region "$CWASM_REGION" \
    | docker login --username AWS --password-stdin "$ECR_REGISTRY"

  echo ">> Building & pushing image ${IMAGE_URI}"
  docker buildx build \
    --platform linux/arm64 \
    --provenance=false \
    --sbom=false \
    --file "${REPO_ROOT}/cwasm-compiler/Dockerfile" \
    --tag "$IMAGE_URI" \
    --push \
    "$REPO_ROOT"

  ENV_VARS="Variables={BUCKET=${R2_BUCKET},XDG_CACHE_HOME=/tmp,HOME=/tmp,R2_ENDPOINT=${R2_ENDPOINT},R2_ACCESS_KEY_ID=${R2_ACCESS_KEY_ID},R2_SECRET_ACCESS_KEY=${R2_SECRET_ACCESS_KEY}}"

  if aws lambda get-function --region "$CWASM_REGION" --function-name "$FUNCTION_NAME" >/dev/null 2>&1; then
    echo ">> Lambda exists; updating code + config"
    aws lambda update-function-code \
      --region "$CWASM_REGION" \
      --function-name "$FUNCTION_NAME" \
      --image-uri "$IMAGE_URI" \
      >/dev/null
    UPDATE_LOG="${WORK_DIR}/lambda_update.log"
    until aws lambda update-function-configuration \
      --region "$CWASM_REGION" \
      --function-name "$FUNCTION_NAME" \
      --environment "$ENV_VARS" \
      >/dev/null 2>"$UPDATE_LOG"; do
      if ! grep -q "ResourceConflictException" "$UPDATE_LOG"; then
        cat "$UPDATE_LOG" >&2
        exit 1
      fi
      echo "   update-function-configuration conflicting; retry..."
      sleep 5
    done
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
      >/dev/null
  fi

  echo ">> Waiting for Lambda to be active"
  aws lambda wait function-active --region "$CWASM_REGION" --function-name "$FUNCTION_NAME"
  aws lambda wait function-updated --region "$CWASM_REGION" --function-name "$FUNCTION_NAME"
)

# --- step 2: set pending (or seed active) before fanout so live deploys catch up automatically ---
echo ">> set_pending_fn0_wasmtime ${NEW_VERSION}"
call_set_pending

# --- step 3: list originals/compiled diff and sync-invoke ---
ORIGINALS_FILE="${WORK_DIR}/originals.jsonl"
COMPILED_FILE="${WORK_DIR}/compiled.txt"
TODO_FILE="${WORK_DIR}/todo.jsonl"
SUCCESS_FILE="${WORK_DIR}/success.txt"
FAIL_FILE="${WORK_DIR}/fail.txt"
: > "$ORIGINALS_FILE"
: > "$COMPILED_FILE"
: > "$TODO_FILE"
: > "$SUCCESS_FILE"
: > "$FAIL_FILE"

list_r2 "original/" \
  | jq -c 'select(.Key | test("^original/[^/]+\\.tar$")) | {project_id: (.Key | capture("^original/(?<p>[^/]+)\\.tar$").p), last_modified_raw: .LastModified}' \
  > "$ORIGINALS_FILE" || true

list_r2 "compiled/${NEW_VERSION}/" \
  | jq -r 'select(.Key | test("^compiled/[^/]+/[^/]+/.+\\.tar\\.zst$")) | (.Key | capture("^compiled/[^/]+/(?<p>[^/]+)/(?<lm>.+)\\.tar\\.zst$") | "\(.p)|\(.lm)")' \
  > "$COMPILED_FILE" || true

while IFS= read -r line; do
  pid="$(jq -r '.project_id' <<<"$line")"
  lm_raw="$(jq -r '.last_modified_raw' <<<"$line")"
  lm="$(normalize_lm "$lm_raw")"
  key="${pid}|${lm}"
  if grep -Fxq "$key" "$COMPILED_FILE"; then
    continue
  fi
  jq -nc --arg p "$pid" --arg lm "$lm" '{project_id: $p, last_modified: $lm}' >> "$TODO_FILE"
done < "$ORIGINALS_FILE"

TOTAL="$(wc -l < "$ORIGINALS_FILE" | tr -d ' ')"
TODO_COUNT="$(wc -l < "$TODO_FILE" | tr -d ' ')"
ALREADY=$((TOTAL - TODO_COUNT))

echo ">> originals=${TOTAL} already_compiled=${ALREADY} todo=${TODO_COUNT}"

invoke_one() {
  local entry="$1"
  local pid lm input_key output_key payload_file out_file meta_file rc fn_err log_b64
  pid="$(jq -r '.project_id' <<<"$entry")"
  lm="$(jq -r '.last_modified' <<<"$entry")"
  input_key="original/${pid}.tar"
  output_key="compiled/${SETPENDING_NEW_VERSION}/${pid}/${lm}.tar.zst"

  payload_file="${SETPENDING_WORK_DIR}/payload.${pid}.${lm//[:.]/_}.json"
  out_file="${SETPENDING_WORK_DIR}/out.${pid}.${lm//[:.]/_}.json"
  meta_file="${SETPENDING_WORK_DIR}/meta.${pid}.${lm//[:.]/_}.json"

  jq -nc --arg ib "$SETPENDING_R2_BUCKET" --arg ik "$input_key" \
         --arg ob "$SETPENDING_R2_BUCKET" --arg ok "$output_key" \
         '{input_bucket: $ib, input_key: $ik, output_bucket: $ob, output_key: $ok, env_bucket: null, env_key: null}' \
    > "$payload_file"

  rc=0
  AWS_ACCESS_KEY_ID="$SETPENDING_HQ_AWS_ACCESS_KEY_ID" \
  AWS_SECRET_ACCESS_KEY="$SETPENDING_HQ_AWS_SECRET_ACCESS_KEY" \
  AWS_DEFAULT_REGION="$SETPENDING_CWASM_REGION" \
  aws lambda invoke \
    --function-name "$SETPENDING_FUNCTION_NAME" \
    --invocation-type RequestResponse \
    --log-type Tail \
    --cli-binary-format raw-in-base64-out \
    --payload "file://${payload_file}" \
    "$out_file" \
    > "$meta_file" 2>&1 || rc=$?

  if [[ $rc -ne 0 ]]; then
    {
      echo "[FAIL] ${pid} ${lm}: aws cli rc=${rc}"
      sed 's/^/  /' < "$meta_file"
      echo "---"
    } >> "$SETPENDING_FAIL_FILE"
    return
  fi

  fn_err="$(jq -r '.FunctionError // empty' < "$meta_file" 2>/dev/null || true)"
  log_b64="$(jq -r '.LogResult // empty' < "$meta_file" 2>/dev/null || true)"

  if [[ -n "$fn_err" ]]; then
    {
      echo "[FAIL] ${pid} ${lm}: FunctionError=${fn_err}"
      echo "  payload:"
      sed 's/^/    /' < "$out_file"
      if [[ -n "$log_b64" ]]; then
        echo "  log tail:"
        echo "$log_b64" | base64 -d | sed 's/^/    /'
      fi
      echo "---"
    } >> "$SETPENDING_FAIL_FILE"
  else
    echo "${pid} ${lm}" >> "$SETPENDING_SUCCESS_FILE"
  fi
}

export -f invoke_one
export SETPENDING_NEW_VERSION="$NEW_VERSION"
export SETPENDING_WORK_DIR="$WORK_DIR"
export SETPENDING_R2_BUCKET="$R2_BUCKET"
export SETPENDING_FUNCTION_NAME="$FUNCTION_NAME"
export SETPENDING_HQ_AWS_ACCESS_KEY_ID="$HQ_AWS_ACCESS_KEY_ID"
export SETPENDING_HQ_AWS_SECRET_ACCESS_KEY="$HQ_AWS_SECRET_ACCESS_KEY"
export SETPENDING_CWASM_REGION="$CWASM_REGION"
export SETPENDING_SUCCESS_FILE="$SUCCESS_FILE"
export SETPENDING_FAIL_FILE="$FAIL_FILE"

if [[ "$TODO_COUNT" -gt 0 ]]; then
  echo ">> invoking lambda for ${TODO_COUNT} bundle(s) (parallel=${PARALLEL})"
  xargs -I {} -P "$PARALLEL" bash -c 'invoke_one "$@"' _ {} < "$TODO_FILE"
fi

SUCCESS_COUNT="$(wc -l < "$SUCCESS_FILE" | tr -d ' ')"
FAIL_COUNT="$(grep -c '^\[FAIL\]' "$FAIL_FILE" || true)"
TOTAL_SUCCESS=$((ALREADY + SUCCESS_COUNT))

echo ""
echo ">> success(this run)=${SUCCESS_COUNT} fail=${FAIL_COUNT} total_success=${TOTAL_SUCCESS}/${TOTAL}"

# --- step 4: full coverage gate ---
if [[ "$FAIL_COUNT" -gt 0 ]]; then
  echo ""
  echo "=== failures ===" >&2
  cat "$FAIL_FILE" >&2
  echo ">> aborting; pending stays as-is. rerun the script after addressing the failures." >&2
  exit 1
fi
if [[ "$TOTAL_SUCCESS" -lt "$TOTAL" ]]; then
  echo ">> incomplete coverage (${TOTAL_SUCCESS}/${TOTAL}); pending stays as-is. rerun the script." >&2
  exit 1
fi

echo ">> done. ${NEW_VERSION} pending registered with full coverage."
