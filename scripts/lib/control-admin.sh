# shellcheck shell=bash
# Control admin HTTP helpers. Source-only.

if [[ -n "${__FN0_CONTROL_ADMIN_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_ADMIN_LOADED=1

# Calls a control __forte_action endpoint. Echoes the response body to stdout.
# Args: <action_name> <json_body>
# Returns: HTTP status via global CONTROL_ADMIN_HTTP_CODE, response body via stdout.
control_admin_invoke() {
  local action="$1"
  local body="$2"
  local control_url admin_token resp_file
  control_url="$(pulumi_pick controlUrl)"
  admin_token="$(pulumi_pick controlAdminTokenBase64)"
  if [[ -z "$control_url" || -z "$admin_token" ]]; then
    echo "controlUrl / controlAdminTokenBase64 missing" >&2
    return 1
  fi
  resp_file="$(mktemp)"
  CONTROL_ADMIN_HTTP_CODE="$(curl -sS -o "$resp_file" -w '%{http_code}' \
    -X POST "${control_url%/}/__forte_action/${action}" \
    -H "Authorization: Bearer ${admin_token}" \
    -H "Content-Type: application/json" \
    --data "$body")"
  cat "$resp_file"
  rm -f "$resp_file"
}

# Reads Fn0WasmtimeVersionDoc directly from doc-db (control-independent).
# Echoes JSON: {"active": "...", "pending": "..."|null} or empty if not present.
get_fn0_wasmtime_version_doc() {
  local doc_db_url doc_db_token https_url resp_file http_code req
  doc_db_url="$(pulumi_pick docDbUrl)"
  doc_db_token="$(pulumi_pick docDbToken)"
  if [[ -z "$doc_db_url" || -z "$doc_db_token" ]]; then
    echo "docDbUrl / docDbToken missing" >&2
    return 1
  fi
  https_url="${doc_db_url/libsql:\/\//https://}"
  https_url="${https_url%/}"
  req="$(jq -nc \
    --arg sql "SELECT data FROM docs WHERE pk = ? AND sk = ''" \
    --arg pk "Fn0WasmtimeVersionDoc" \
    '{requests: [
      {type: "execute", stmt: {sql: $sql, args: [{type: "text", value: $pk}]}},
      {type: "close"}
    ]}')"
  resp_file="$(mktemp)"
  http_code="$(curl -sS -o "$resp_file" -w '%{http_code}' \
    -X POST "${https_url}/v2/pipeline" \
    -H "Authorization: Bearer ${doc_db_token}" \
    -H "Content-Type: application/json" \
    --data-raw "$req")"
  if [[ "$http_code" != "200" ]]; then
    cat "$resp_file" >&2
    rm -f "$resp_file"
    echo "doc-db read failed (HTTP ${http_code})" >&2
    return 1
  fi
  jq -r '
    .results[0].response.result.rows[0][0]
    | if . == null then ""
      elif .type == "blob" then (.base64 | @base64d)
      else (.value // "")
      end
  ' <"$resp_file"
  rm -f "$resp_file"
}

# Calls control set_pending_fn0_wasmtime admin action. Idempotent.
control_set_pending_fn0_wasmtime() {
  local version="$1"
  local body resp outcome
  body="$(jq -nc --arg v "$version" '{version: $v}')"
  resp="$(control_admin_invoke set_pending_fn0_wasmtime "$body")"
  if [[ "$CONTROL_ADMIN_HTTP_CODE" != "200" ]]; then
    echo "set_pending_fn0_wasmtime HTTP ${CONTROL_ADMIN_HTTP_CODE}: ${resp}" >&2
    return 1
  fi
  outcome="$(jq -r 'if type == "string" then . else (keys[0]) end' <<<"$resp")"
  case "$outcome" in
    Ok) ;;
    Unauthorized) echo "admin token rejected (set_pending)" >&2; return 1 ;;
    *) echo "set_pending_fn0_wasmtime unexpected response: ${resp}" >&2; return 1 ;;
  esac
}

# Calls control promote_pending_fn0_wasmtime admin action. Idempotent.
# Echoes JSON {"old_active": "...", "new_active": "..."} on success;
# empty string on NoPending (noop).
control_promote_pending_fn0_wasmtime() {
  local resp outcome
  resp="$(control_admin_invoke promote_pending_fn0_wasmtime '{}')"
  if [[ "$CONTROL_ADMIN_HTTP_CODE" != "200" ]]; then
    echo "promote_pending_fn0_wasmtime HTTP ${CONTROL_ADMIN_HTTP_CODE}: ${resp}" >&2
    return 1
  fi
  outcome="$(jq -r '
    if type == "string" then .
    elif has("Ok") then "Ok"
    else (keys[0]) end' <<<"$resp")"
  case "$outcome" in
    Ok)
      jq -c '.Ok' <<<"$resp"
      ;;
    NoPending)
      echo ""
      ;;
    NoActiveVersion)
      echo "control reports no active version yet" >&2
      return 1
      ;;
    Unauthorized)
      echo "admin token rejected (promote)" >&2
      return 1
      ;;
    *)
      echo "promote_pending_fn0_wasmtime unexpected response: ${resp}" >&2
      return 1
      ;;
  esac
}
