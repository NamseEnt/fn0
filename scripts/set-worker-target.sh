#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PULUMI_DIR="${PULUMI_DIR:-${REPO_ROOT}/infra/cloud}"

WAIT=1
TIMEOUT=1800
POLL_INTERVAL=5
POSITIONAL=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-wait) WAIT=0; shift ;;
    --wait) WAIT=1; shift ;;
    --timeout=*) TIMEOUT="${1#--timeout=}"; shift ;;
    --timeout) TIMEOUT="${2:-}"; shift 2 ;;
    --poll-interval=*) POLL_INTERVAL="${1#--poll-interval=}"; shift ;;
    -h|--help)
      cat <<EOF
usage: $0 <site-name> [tag] [--no-wait] [--timeout=<seconds>] [--poll-interval=<seconds>]

Writes worker-target:<site-name> and (by default) waits for the HQ rolling
update to converge (worker-last-stable == worker-target).

Options:
  --no-wait              fire-and-forget; return immediately after the write
  --timeout=<seconds>    max wait time (default: ${TIMEOUT})
  --poll-interval=<secs> doc-db poll interval (default: ${POLL_INTERVAL})
EOF
      exit 0 ;;
    --) shift; break ;;
    -*) echo "unknown flag: $1" >&2; exit 2 ;;
    *) POSITIONAL+=("$1"); shift ;;
  esac
done
set -- "${POSITIONAL[@]}"

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <site-name> [tag] [--no-wait] [--timeout=<seconds>]" >&2
  exit 2
fi
SITE_NAME="$1"
TAG_OVERRIDE="${2:-}"

if ! [[ "$TIMEOUT" =~ ^[0-9]+$ ]] || (( TIMEOUT <= 0 )); then
  echo "invalid --timeout: $TIMEOUT" >&2
  exit 2
fi
if ! [[ "$POLL_INTERVAL" =~ ^[0-9]+$ ]] || (( POLL_INTERVAL <= 0 )); then
  echo "invalid --poll-interval: $POLL_INTERVAL" >&2
  exit 2
fi

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
  TAG="$(cd "${REPO_ROOT}" && cargo pkgid -p fn0-worker | sed -E 's/.*[#@]([^:]+)$/\1/')"
fi

if [[ -z "$TAG" ]]; then
  echo "failed to determine image tag" >&2
  exit 1
fi

WASMTIME_VERSION="$(cd "${REPO_ROOT}" && cargo pkgid -p fn0-wasmtime | sed -E 's/.*[#@]([^:]+)$/\1/')"
if [[ -z "$WASMTIME_VERSION" ]]; then
  echo "failed to determine fn0-wasmtime version" >&2
  exit 1
fi

HTTPS_URL="${DOC_DB_URL/libsql:\/\//https://}"
HTTPS_URL="${HTTPS_URL%/}"

TARGET_JSON="$(jq -nc \
  --arg reg "$REGISTRY" \
  --arg repo "$REPOSITORY" \
  --arg tag "$TAG" \
  --arg wasmtime "$WASMTIME_VERSION" \
  '{image_registry: $reg, image_repository: $repo, image_tag: $tag, fn0_wasmtime_version: $wasmtime}')"

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
  echo "worker-target:${SITE_NAME} = ${REGISTRY}/${REPOSITORY}:${TAG} (fn0-wasmtime ${WASMTIME_VERSION})"
else
  echo "doc-db write unexpected response:" >&2
  cat "$RESP_BODY" >&2
  exit 1
fi

if (( WAIT == 0 )); then
  exit 0
fi

TARGET_NORM="$(jq -Sc . <<<"$TARGET_JSON")"
LAST_STABLE_PK="worker-last-stable:${SITE_NAME}"
LAST_STABLE_REQ="$(jq -nc \
  --arg sql "SELECT value FROM docs WHERE pk = ? AND sk = 0" \
  --arg pk "$LAST_STABLE_PK" \
  '{
    requests: [
      {type: "execute", stmt: {sql: $sql, args: [{type: "text", value: $pk}]}},
      {type: "close"}
    ]
  }')"

echo "waiting for rollout to converge (timeout ${TIMEOUT}s, poll ${POLL_INTERVAL}s)..."
START_TS="$(date +%s)"
DEADLINE=$(( START_TS + TIMEOUT ))
LAST_REPORTED=""

while :; do
  HTTP_CODE="$(curl -sS -o "$RESP_BODY" -w '%{http_code}' \
    -X POST "${HTTPS_URL}/v2/pipeline" \
    -H "Authorization: Bearer ${DOC_DB_TOKEN}" \
    -H "Content-Type: application/json" \
    --data-raw "$LAST_STABLE_REQ" || echo 000)"

  if [[ "$HTTP_CODE" == "200" ]] && jq -e '.results[0].type == "ok"' <"$RESP_BODY" >/dev/null 2>&1; then
    LS_RAW="$(jq -r '.results[0].response.result.rows[0][0].value // empty' <"$RESP_BODY")"
    if [[ -n "$LS_RAW" ]]; then
      LS_NORM="$(jq -Sc . <<<"$LS_RAW")"
      if [[ "$LS_NORM" == "$TARGET_NORM" ]]; then
        ELAPSED=$(( $(date +%s) - START_TS ))
        echo "rollout converged in ${ELAPSED}s"
        exit 0
      fi
      CURRENT="$(jq -r '.image_tag // "?"' <<<"$LS_RAW")"
      if [[ "$CURRENT" != "$LAST_REPORTED" ]]; then
        echo "  last_stable: ${CURRENT} (target ${TAG})"
        LAST_REPORTED="$CURRENT"
      fi
    elif [[ -z "$LAST_REPORTED" ]]; then
      echo "  last_stable: <none> (target ${TAG})"
      LAST_REPORTED="<none>"
    fi
  else
    echo "  doc-db poll failed (HTTP ${HTTP_CODE}), retrying" >&2
  fi

  NOW="$(date +%s)"
  if (( NOW >= DEADLINE )); then
    ELAPSED=$(( NOW - START_TS ))
    echo "timeout after ${ELAPSED}s waiting for worker-last-stable == worker-target" >&2
    exit 1
  fi
  sleep "$POLL_INTERVAL"
done
