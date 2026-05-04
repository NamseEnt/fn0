#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PULUMI_DIR="${PULUMI_DIR:-${REPO_ROOT}/infra/cloud}"

usage() {
  cat <<EOF
usage: $0 <fn0_wasmtime_version>

Idempotent. Promotes <ver> to control's active fn0-wasmtime version
via activate_fn0_wasmtime. If a previous active version is reported
back, its lambda function (fn0-cwasm-compiler-<old-dashed>) and ECR
image tag are deleted.

Run this only after every worker is already on <ver>; otherwise live
deploys won't compile for whatever fn0-wasmtime the worker is still
on.
EOF
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

VERSION="$1"

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "required command not found: $1" >&2; exit 1; }
}
need pulumi
need jq
need aws
need curl

echo ">> Reading Pulumi stack outputs from ${PULUMI_DIR}"
OUT="$(cd "$PULUMI_DIR" && pulumi stack output --show-secrets --json)"
pick() { jq -r ".${1} // empty" <<<"$OUT"; }

CWASM_REGION="$(pick cwasmCompilerBucketRegion)"
CWASM_ECR="$(pick cwasmCompilerEcrRepository)"
BUILDER_AWS_ACCESS_KEY_ID="$(pick cwasmCompilerBuilderAccessKeyId)"
BUILDER_AWS_SECRET_ACCESS_KEY="$(pick cwasmCompilerBuilderSecretAccessKey)"
CONTROL_URL="$(pick controlUrl)"
ADMIN_TOKEN="$(pick controlAdminTokenBase64)"

for v in CWASM_REGION CWASM_ECR BUILDER_AWS_ACCESS_KEY_ID \
         BUILDER_AWS_SECRET_ACCESS_KEY CONTROL_URL ADMIN_TOKEN; do
  if [[ -z "${!v}" ]]; then
    echo "missing Pulumi output: ${v}" >&2
    exit 1
  fi
done

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

ECR_REPO_NAME="${CWASM_ECR##*/}"

echo ">> activate_fn0_wasmtime ${VERSION}"
RESP="${WORK_DIR}/activate_resp.json"
HTTP="$(curl -sS -o "$RESP" -w '%{http_code}' \
  -X POST "${CONTROL_URL%/}/__forte_action/activate_fn0_wasmtime" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  --data "$(jq -nc --arg v "$VERSION" '{version: $v}')")"

if [[ "$HTTP" != "200" ]]; then
  echo "control activate HTTP ${HTTP}" >&2
  cat "$RESP" >&2
  exit 1
fi

OUTCOME="$(jq -r '
  if type=="string" then .
  elif has("Ok") then "Ok"
  else (keys[0]) end' < "$RESP")"

case "$OUTCOME" in
  Ok)
    OLD_ACTIVE="$(jq -r '.Ok.old_active // empty' < "$RESP")"
    ;;
  Unauthorized)
    echo "admin token rejected" >&2
    exit 1
    ;;
  *)
    echo "unexpected activate response:" >&2
    cat "$RESP" >&2
    exit 1
    ;;
esac

echo ">> active=${VERSION}, old_active=${OLD_ACTIVE:-<none>}"

if [[ -z "$OLD_ACTIVE" || "$OLD_ACTIVE" == "$VERSION" ]]; then
  echo ">> nothing to clean up."
  exit 0
fi

OLD_DASH="${OLD_ACTIVE//./-}"
OLD_FN_NAME="fn0-cwasm-compiler-${OLD_DASH}"

(
  export AWS_ACCESS_KEY_ID="$BUILDER_AWS_ACCESS_KEY_ID"
  export AWS_SECRET_ACCESS_KEY="$BUILDER_AWS_SECRET_ACCESS_KEY"
  export AWS_DEFAULT_REGION="$CWASM_REGION"
  unset AWS_SESSION_TOKEN AWS_PROFILE

  ERR="${WORK_DIR}/lambda_delete.err"
  rc=0
  aws lambda delete-function --function-name "$OLD_FN_NAME" 2>"$ERR" || rc=$?
  if [[ $rc -ne 0 ]] && ! grep -q "ResourceNotFoundException" "$ERR"; then
    cat "$ERR" >&2
    exit 1
  fi
  echo ">> deleted lambda ${OLD_FN_NAME} (or already absent)"

  ERR="${WORK_DIR}/ecr_delete.err"
  rc=0
  aws ecr batch-delete-image \
    --repository-name "$ECR_REPO_NAME" \
    --image-ids "imageTag=${OLD_DASH}" \
    >/dev/null 2>"$ERR" || rc=$?
  if [[ $rc -ne 0 ]] && ! grep -q "RepositoryNotFoundException\|ImageNotFoundException" "$ERR"; then
    cat "$ERR" >&2
    exit 1
  fi
  echo ">> deleted ECR image tag ${OLD_DASH} (or already absent)"
)

echo ">> done."
