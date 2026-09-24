# shellcheck shell=bash

if [[ -n "${__FN0_CONTROL_DB_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_DB_LOADED=1
unset __FN0_CONTROL_DB_INITIALIZED CONTROL_DB_BACKEND

source "${REPO_ROOT}/scripts/lib/dodb-control-db.sh"

control_db_init() {
  if [[ -n "${__FN0_CONTROL_DB_INITIALIZED:-}" ]]; then
    return 0
  fi
  CONTROL_DB_BACKEND="$(cd "${REPO_ROOT}/infra/cloud" && pulumi config get fn0Cloud:dbBackend)"
  case "$CONTROL_DB_BACKEND" in
    turso)
      CONTROL_DB_URL="$(pulumi_pick controlDbUrl)"
      CONTROL_DB_TOKEN="$(pulumi_pick forteDbGroupToken)"
      if [[ -z "$CONTROL_DB_URL" || -z "$CONTROL_DB_TOKEN" ]]; then
        echo "Turso backend requires controlDbUrl and forteDbGroupToken Pulumi outputs" >&2
        return 1
      fi
      CONTROL_DB_URL="${CONTROL_DB_URL/libsql:\/\//https://}"
      CONTROL_DB_URL="${CONTROL_DB_URL%/}"
      ;;
    dodb) ;;
    *)
      echo "invalid fn0Cloud:dbBackend value: ${CONTROL_DB_BACKEND:-<empty>} (expected turso or dodb)" >&2
      return 2
      ;;
  esac
  __FN0_CONTROL_DB_INITIALIZED=1
}

control_db_ensure_ready() {
  control_db_init
  if [[ "$CONTROL_DB_BACKEND" == turso ]]; then
    local sql request response_file http_code
    sql="CREATE TABLE IF NOT EXISTS docs (pk TEXT, sk TEXT, data BLOB, version INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (pk, sk))"
    request="$(jq -nc --arg sql "$sql" '{requests:[{type:"execute",stmt:{sql:$sql}},{type:"close"}]}')"
    response_file="$(mktemp)"
    http_code="$(curl -sS -o "$response_file" -w '%{http_code}' -X POST "${CONTROL_DB_URL}/v2/pipeline" \
      -H "Authorization: Bearer ${CONTROL_DB_TOKEN}" -H 'Content-Type: application/json' --data "$request")"
    if [[ "$http_code" != 200 ]] || jq -e '.results[0].type == "error"' <"$response_file" >/dev/null 2>&1; then
      cat "$response_file" >&2
      rm -f "$response_file"
      echo "control DB ensure failed (HTTP $http_code)" >&2
      return 1
    fi
    rm -f "$response_file"
  fi
}

control_db_open() {
  control_db_init
  if [[ "$CONTROL_DB_BACKEND" == dodb ]]; then
    dodb_control_db_open
  fi
}

control_db_close() {
  dodb_control_db_close
}

__control_db_turso_request() {
  local request="$1" context="$2" response_file http_code
  response_file="$(mktemp)"
  http_code="$(curl -sS -o "$response_file" -w '%{http_code}' -X POST "${CONTROL_DB_URL}/v2/pipeline" \
    -H "Authorization: Bearer ${CONTROL_DB_TOKEN}" -H 'Content-Type: application/json' --data-raw "$request")"
  if [[ "$http_code" != 200 ]] || ! jq -e '.results[0].type == "ok"' <"$response_file" >/dev/null 2>&1; then
    cat "$response_file" >&2
    rm -f "$response_file"
    echo "control DB ${context} failed (HTTP $http_code)" >&2
    return 1
  fi
  cat "$response_file"
  rm -f "$response_file"
}

control_db_get_observed() {
  control_db_init
  local pk="$1" sk="$2" request response cell_type data_b64 revision
  if [[ "$CONTROL_DB_BACKEND" == dodb ]]; then
    dodb_control_db_call get-observed --pk "$pk" --sk "$sk"
    return
  fi
  request="$(jq -nc --arg pk "$pk" --arg sk "$sk" '{requests:[{type:"execute",stmt:{sql:"SELECT data, version FROM docs WHERE pk = ? AND sk = ?",args:[{type:"text",value:$pk},{type:"text",value:$sk}]}},{type:"close"}]}')"
  response="$(__control_db_turso_request "$request" "get-observed")"
  cell_type="$(jq -r '.results[0].response.result.rows[0][0].type // empty' <<<"$response")"
  if [[ -z "$cell_type" ]]; then
    printf '%s\n' '{"found":false,"revision":0,"data_base64":null}'
    return 0
  fi
  data_b64="$(jq -r '.results[0].response.result.rows[0][0].base64 // empty' <<<"$response")"
  if [[ "$cell_type" != blob ]]; then
    data_b64="$(jq -r '.results[0].response.result.rows[0][0].value // empty | @base64' <<<"$response")"
  fi
  revision="$(jq -r '.results[0].response.result.rows[0][1].value // 0' <<<"$response")"
  jq -nc --argjson revision "$revision" --arg data "$data_b64" '{found:true,revision:$revision,data_base64:$data}'
}

control_db_put() {
  control_db_init
  local pk="$1" sk="$2" data="$3" data_b64 request
  if [[ "$CONTROL_DB_BACKEND" == dodb ]]; then
    dodb_control_db_call put --pk "$pk" --sk "$sk" < <(printf '%s' "$data")
    return
  fi
  data_b64="$(printf '%s' "$data" | base64 | tr -d '\n')"
  request="$(jq -nc --arg pk "$pk" --arg sk "$sk" --arg blob "$data_b64" '{requests:[{type:"execute",stmt:{sql:"INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, 0) ON CONFLICT(pk, sk) DO UPDATE SET data = excluded.data, version = docs.version + 1 WHERE docs.data IS NOT excluded.data",args:[{type:"text",value:$pk},{type:"text",value:$sk},{type:"blob",base64:$blob}]}},{type:"close"}]}')"
  __control_db_turso_request "$request" "put" >/dev/null
  printf '%s\n' '{"written":true}'
}

control_db_put_if_missing() {
  control_db_init
  local pk="$1" sk="$2" data="$3" data_b64 request response affected
  if [[ "$CONTROL_DB_BACKEND" == dodb ]]; then
    dodb_control_db_call put-if-missing --pk "$pk" --sk "$sk" < <(printf '%s' "$data")
    return
  fi
  data_b64="$(printf '%s' "$data" | base64 | tr -d '\n')"
  request="$(jq -nc --arg pk "$pk" --arg sk "$sk" --arg blob "$data_b64" '{requests:[{type:"execute",stmt:{sql:"INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, 0) ON CONFLICT(pk, sk) DO NOTHING",args:[{type:"text",value:$pk},{type:"text",value:$sk},{type:"blob",base64:$blob}]}},{type:"close"}]}')"
  response="$(__control_db_turso_request "$request" "put-if-missing")"
  affected="$(jq -r '.results[0].response.result.affected_row_count // 0' <<<"$response")"
  if [[ "$affected" == 0 ]]; then
    echo "conditional write conflict: document already exists" >&2
    return 3
  fi
  printf '%s\n' '{"written":true}'
}

control_db_put_if_revision() {
  control_db_init
  local pk="$1" sk="$2" expected_revision="$3" data="$4" data_b64 request response affected
  if [[ "$CONTROL_DB_BACKEND" == dodb ]]; then
    dodb_control_db_call put-if-revision --pk "$pk" --sk "$sk" --expected-revision "$expected_revision" < <(printf '%s' "$data")
    return
  fi
  data_b64="$(printf '%s' "$data" | base64 | tr -d '\n')"
  request="$(jq -nc --arg pk "$pk" --arg sk "$sk" --arg revision "$expected_revision" --arg blob "$data_b64" '{requests:[{type:"execute",stmt:{sql:"UPDATE docs SET data = ?, version = version + 1 WHERE pk = ? AND sk = ? AND version = ?",args:[{type:"blob",base64:$blob},{type:"text",value:$pk},{type:"text",value:$sk},{type:"integer",value:$revision}]}},{type:"close"}]}')"
  response="$(__control_db_turso_request "$request" "put-if-revision")"
  affected="$(jq -r '.results[0].response.result.affected_row_count // 0' <<<"$response")"
  if [[ "$affected" == 0 ]]; then
    echo "conditional write conflict: document revision changed" >&2
    return 3
  fi
  printf '%s\n' '{"written":true}'
}

control_db_query() {
  control_db_init
  local pk="$1" after_sk="$2" limit="$3" request response rows
  if [[ ! "$limit" =~ ^[0-9]+$ ]] || (( limit < 1 || limit > 1000 )); then
    echo "control_db_query limit must be between 1 and 1000" >&2
    return 2
  fi
  if [[ "$CONTROL_DB_BACKEND" == dodb ]]; then
    local arguments=(query --pk "$pk" --limit "$limit")
    if [[ -n "$after_sk" ]]; then arguments+=(--after-sk "$after_sk"); fi
    dodb_control_db_call "${arguments[@]}"
    return
  fi
  if [[ -n "$after_sk" ]]; then
    request="$(jq -nc --arg pk "$pk" --arg sk "$after_sk" --arg limit "$limit" '{requests:[{type:"execute",stmt:{sql:"SELECT pk, sk, data FROM docs WHERE pk = ? AND sk > ? ORDER BY sk LIMIT ?",args:[{type:"text",value:$pk},{type:"text",value:$sk},{type:"integer",value:$limit}]}},{type:"close"}]}')"
  else
    request="$(jq -nc --arg pk "$pk" --arg limit "$limit" '{requests:[{type:"execute",stmt:{sql:"SELECT pk, sk, data FROM docs WHERE pk = ? ORDER BY sk LIMIT ?",args:[{type:"text",value:$pk},{type:"integer",value:$limit}]}},{type:"close"}]}')"
  fi
  response="$(__control_db_turso_request "$request" "query")"
  rows="$(jq -c '[.results[0].response.result.rows[]? | {pk:.[0].value,sk:.[1].value,data_base64:.[2].base64}]' <<<"$response")"
  jq -nc --argjson documents "$rows" '{documents:$documents}'
}

control_db_scan() {
  control_db_init
  local after_pk="$1" after_sk="$2" limit="$3" has_cursor="${4:-false}" request response rows
  if [[ ! "$limit" =~ ^[0-9]+$ ]] || (( limit < 1 || limit > 1000 )); then
    echo "control_db_scan limit must be between 1 and 1000" >&2
    return 2
  fi
  if [[ "$CONTROL_DB_BACKEND" == dodb ]]; then
    local arguments=(scan --limit "$limit")
    if [[ "$has_cursor" == true ]]; then
      [[ -n "$after_pk" ]] || { echo "control_db_scan requires a primary-key cursor" >&2; return 2; }
      arguments+=(--after-pk "$after_pk" --after-sk "$after_sk")
    fi
    dodb_control_db_call "${arguments[@]}"
    return
  fi
  if [[ "$has_cursor" == true ]]; then
    [[ -n "$after_pk" ]] || { echo "control_db_scan requires a primary-key cursor" >&2; return 2; }
    request="$(jq -nc --arg pk "$after_pk" --arg sk "$after_sk" --arg limit "$limit" '{requests:[{type:"execute",stmt:{sql:"SELECT pk, sk, data FROM docs WHERE pk > ? OR (pk = ? AND sk > ?) ORDER BY pk, sk LIMIT ?",args:[{type:"text",value:$pk},{type:"text",value:$pk},{type:"text",value:$sk},{type:"integer",value:$limit}]}},{type:"close"}]}')"
  else
    request="$(jq -nc --arg limit "$limit" '{requests:[{type:"execute",stmt:{sql:"SELECT pk, sk, data FROM docs ORDER BY pk, sk LIMIT ?",args:[{type:"integer",value:$limit}]}},{type:"close"}]}')"
  fi
  response="$( __control_db_turso_request "$request" "scan")"
  rows="$(jq -c '[.results[0].response.result.rows[]? | {pk:.[0].value,sk:.[1].value,data_base64:.[2].base64}]' <<<"$response")"
  jq -nc --argjson documents "$rows" '{documents:$documents}'
}
