#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PULUMI_DIR="${PULUMI_DIR:-${REPO_ROOT}/infra/cloud}"

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <site-name> [tag]" >&2
  exit 2
fi
SITE_NAME="$1"
TAG_OVERRIDE="${2:-}"

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "required command not found: $1" >&2; exit 1; }
}
need pulumi
need jq
need cargo
need curl

OUT="$(cd "$PULUMI_DIR" && pulumi stack output --show-secrets --json)"
pick() { jq -r ".${1} // empty" <<<"$OUT"; }

REGISTRIES_JSON="$(jq -c '.workerImageRegistries' <<<"$OUT")"
DOC_DB_URL="$(pick docDbUrl)"
DOC_DB_TOKEN="$(pick docDbToken)"

for v in DOC_DB_URL DOC_DB_TOKEN; do
  if [[ -z "${!v}" ]]; then
    echo "missing Pulumi output: ${v}" >&2
    exit 1
  fi
done
if [[ -z "$REGISTRIES_JSON" || "$REGISTRIES_JSON" == "null" ]]; then
  echo "missing Pulumi output: workerImageRegistries" >&2
  exit 1
fi

FIRST_REG="$(jq -c '.[0]' <<<"$REGISTRIES_JSON")"
REGISTRY="$(jq -r .url <<<"$FIRST_REG")"
REPOSITORY="$(jq -r .repository <<<"$FIRST_REG")"

if [[ -n "$TAG_OVERRIDE" ]]; then
  TAG="$TAG_OVERRIDE"
else
  TAG="$(cd "${REPO_ROOT}/fn0-worker" && cargo pkgid | sed -E 's/.*[#@]([^:]+)$/\1/')"
fi

if [[ -z "$TAG" ]]; then
  echo "failed to determine image tag" >&2
  exit 1
fi

HTTPS_URL="${DOC_DB_URL/libsql:\/\//https://}"
HTTPS_URL="${HTTPS_URL%/}"

TARGET_JSON="$(jq -nc \
  --arg reg "$REGISTRY" \
  --arg repo "$REPOSITORY" \
  --arg tag "$TAG" \
  '{image_registry: $reg, image_repository: $repo, image_tag: $tag}')"

PK="worker-target:${SITE_NAME}"

REQ_BODY="$(jq -nc \
  --arg sql "REPLACE INTO docs (pk, sk, value) VALUES (?, 0, ?)" \
  --arg pk "$PK" \
  --arg val "$TARGET_JSON" \
  '{
    requests: [
      {type: "execute", stmt: {sql: $sql, args: [
        {type: "text", value: $pk},
        {type: "text", value: $val}
      ]}},
      {type: "close"}
    ]
  }')"

RESP_BODY="$(mktemp)"
trap 'rm -f "$RESP_BODY"' EXIT

HTTP_CODE="$(curl -sS -o "$RESP_BODY" -w '%{http_code}' \
  -X POST "${HTTPS_URL}/v2/pipeline" \
  -H "Authorization: Bearer ${DOC_DB_TOKEN}" \
  -H "Content-Type: application/json" \
  --data-raw "$REQ_BODY")"

if [[ "$HTTP_CODE" != "200" ]]; then
  echo "doc-db write failed (HTTP ${HTTP_CODE}):" >&2
  cat "$RESP_BODY" >&2
  exit 1
fi

if jq -e '.results[0].type == "ok"' <"$RESP_BODY" >/dev/null 2>&1; then
  echo "worker-target:${SITE_NAME} = ${REGISTRY}/${REPOSITORY}:${TAG}"
else
  echo "doc-db write unexpected response:" >&2
  cat "$RESP_BODY" >&2
  exit 1
fi
