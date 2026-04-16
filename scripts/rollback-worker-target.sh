#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PULUMI_DIR="${PULUMI_DIR:-${REPO_ROOT}/infra/cloud}"

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <site-name>" >&2
  exit 2
fi
SITE_NAME="$1"

need() {
  command -v "$1" >/dev/null 2>&1 || { echo "required command not found: $1" >&2; exit 1; }
}
need pulumi
need jq
need curl

OUT="$(cd "$PULUMI_DIR" && pulumi stack output --show-secrets --json)"
DOC_DB_URL="$(jq -r '.docDbUrl // empty' <<<"$OUT")"
DOC_DB_TOKEN="$(jq -r '.docDbToken // empty' <<<"$OUT")"

for v in DOC_DB_URL DOC_DB_TOKEN; do
  if [[ -z "${!v}" ]]; then
    echo "missing Pulumi output: ${v}" >&2
    exit 1
  fi
done

HTTPS_URL="${DOC_DB_URL/libsql:\/\//https://}"
HTTPS_URL="${HTTPS_URL%/}"

PK="worker-last-stable:${SITE_NAME}"

READ_BODY="$(jq -nc \
  --arg sql "SELECT value FROM docs WHERE pk = ? AND sk = 0" \
  --arg pk "$PK" \
  '{
    requests: [
      {type: "execute", stmt: {sql: $sql, args: [{type: "text", value: $pk}]}},
      {type: "close"}
    ]
  }')"

RESP_BODY="$(mktemp)"
trap 'rm -f "$RESP_BODY"' EXIT

HTTP_CODE="$(curl -sS -o "$RESP_BODY" -w '%{http_code}' \
  -X POST "${HTTPS_URL}/v2/pipeline" \
  -H "Authorization: Bearer ${DOC_DB_TOKEN}" \
  -H "Content-Type: application/json" \
  --data-raw "$READ_BODY")"

if [[ "$HTTP_CODE" != "200" ]]; then
  echo "doc-db read failed (HTTP ${HTTP_CODE}):" >&2
  cat "$RESP_BODY" >&2
  exit 1
fi

LAST_STABLE_JSON="$(jq -r '.results[0].response.result.rows[0][0].value // empty' <"$RESP_BODY")"
if [[ -z "$LAST_STABLE_JSON" ]]; then
  echo "rollback impossible: worker-last-stable:${SITE_NAME} not recorded yet" >&2
  exit 3
fi

WRITE_PK="worker-target:${SITE_NAME}"
WRITE_BODY="$(jq -nc \
  --arg sql "REPLACE INTO docs (pk, sk, value) VALUES (?, 0, ?)" \
  --arg pk "$WRITE_PK" \
  --arg val "$LAST_STABLE_JSON" \
  '{
    requests: [
      {type: "execute", stmt: {sql: $sql, args: [
        {type: "text", value: $pk},
        {type: "text", value: $val}
      ]}},
      {type: "close"}
    ]
  }')"

HTTP_CODE="$(curl -sS -o "$RESP_BODY" -w '%{http_code}' \
  -X POST "${HTTPS_URL}/v2/pipeline" \
  -H "Authorization: Bearer ${DOC_DB_TOKEN}" \
  -H "Content-Type: application/json" \
  --data-raw "$WRITE_BODY")"

if [[ "$HTTP_CODE" != "200" ]]; then
  echo "doc-db write failed (HTTP ${HTTP_CODE}):" >&2
  cat "$RESP_BODY" >&2
  exit 1
fi

if jq -e '.results[0].type == "ok"' <"$RESP_BODY" >/dev/null 2>&1; then
  SUMMARY="$(jq -c '{image_tag, fn0_wasmtime_version}' <<<"$LAST_STABLE_JSON")"
  echo "worker-target:${SITE_NAME} rolled back to ${SUMMARY}"
else
  echo "doc-db write unexpected response:" >&2
  cat "$RESP_BODY" >&2
  exit 1
fi
