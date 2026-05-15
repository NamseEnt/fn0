# shellcheck shell=bash
# Seed fn0-control + worker-agent doc-db with the documents needed for the
# control plane to come online. Source-only.

if [[ -n "${__FN0_CONTROL_SEED_LOADED:-}" ]]; then
  return 0
fi
__FN0_CONTROL_SEED_LOADED=1

# ensure_docs_table <db_url> <db_token>
# Idempotently CREATE TABLE IF NOT EXISTS docs(...). Required for any DB that
# wasn't already seeded (i.e. fork-time first run).
ensure_docs_table() {
  local db_url="$1" db_token="$2"
  local https_url="${db_url/libsql:\/\//https://}"
  https_url="${https_url%/}"
  local sql="CREATE TABLE IF NOT EXISTS docs (pk TEXT, sk TEXT, data BLOB, version INTEGER NOT NULL DEFAULT 0, PRIMARY KEY (pk, sk))"
  local req
  req="$(jq -nc --arg sql "$sql" '{requests:[{type:"execute",stmt:{sql:$sql}},{type:"close"}]}')"
  local resp_file http_code
  resp_file="$(mktemp)"
  http_code="$(curl -sS -o "$resp_file" -w '%{http_code}' \
    -X POST "${https_url}/v2/pipeline" \
    -H "Authorization: Bearer ${db_token}" \
    -H "Content-Type: application/json" \
    --data "$req")"
  if [[ "$http_code" != "200" ]]; then
    echo "ensure_docs_table HTTP ${http_code}" >&2
    cat "$resp_file" >&2
    rm -f "$resp_file"
    return 1
  fi
  if jq -e '.results[0].type == "error"' <"$resp_file" >/dev/null 2>&1; then
    echo "ensure_docs_table SQL error:" >&2
    jq '.results[0]' <"$resp_file" >&2
    rm -f "$resp_file"
    return 1
  fi
  rm -f "$resp_file"
}

# __upsert_doc <db_url> <db_token> <pk> <sk> <json_data>
__upsert_doc() {
  local url="$1" token="$2" pk="$3" sk="$4" data="$5"
  local data_b64
  data_b64="$(printf '%s' "$data" | base64 | tr -d '\n')"
  local sql="INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, 0) ON CONFLICT(pk, sk) DO UPDATE SET data = excluded.data, version = docs.version + 1 WHERE docs.data IS NOT excluded.data"
  local https_url="${url/libsql:\/\//https://}"
  https_url="${https_url%/}"
  local req
  req="$(jq -nc \
    --arg sql "$sql" \
    --arg pk "$pk" \
    --arg sk "$sk" \
    --arg blob "$data_b64" \
    '{requests:[{type:"execute",stmt:{sql:$sql,args:[{type:"text",value:$pk},{type:"text",value:$sk},{type:"blob",base64:$blob}]}},{type:"close"}]}')"
  local resp_file http_code
  resp_file="$(mktemp)"
  http_code="$(curl -sS -o "$resp_file" -w '%{http_code}' \
    -X POST "${https_url}/v2/pipeline" \
    -H "Authorization: Bearer ${token}" \
    -H "Content-Type: application/json" \
    --data "$req")"
  if [[ "$http_code" != "200" ]]; then
    echo "doc upsert HTTP ${http_code} for pk=${pk}" >&2
    cat "$resp_file" >&2
    rm -f "$resp_file"
    return 1
  fi
  if jq -e '.results[0].type == "error"' <"$resp_file" >/dev/null 2>&1; then
    echo "doc upsert SQL error for pk=${pk}:" >&2
    jq '.results[0]' <"$resp_file" >&2
    rm -f "$resp_file"
    return 1
  fi
  rm -f "$resp_file"
}

# __insert_doc_if_missing <db_url> <db_token> <pk> <sk> <json_data>
__insert_doc_if_missing() {
  local url="$1" token="$2" pk="$3" sk="$4" data="$5"
  local data_b64
  data_b64="$(printf '%s' "$data" | base64 | tr -d '\n')"
  local sql="INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, 0) ON CONFLICT(pk, sk) DO NOTHING"
  local https_url="${url/libsql:\/\//https://}"
  https_url="${https_url%/}"
  local req
  req="$(jq -nc \
    --arg sql "$sql" \
    --arg pk "$pk" \
    --arg sk "$sk" \
    --arg blob "$data_b64" \
    '{requests:[{type:"execute",stmt:{sql:$sql,args:[{type:"text",value:$pk},{type:"text",value:$sk},{type:"blob",base64:$blob}]}},{type:"close"}]}')"
  local resp_file http_code
  resp_file="$(mktemp)"
  http_code="$(curl -sS -o "$resp_file" -w '%{http_code}' \
    -X POST "${https_url}/v2/pipeline" \
    -H "Authorization: Bearer ${token}" \
    -H "Content-Type: application/json" \
    --data "$req")"
  if [[ "$http_code" != "200" ]]; then
    echo "doc insert HTTP ${http_code} for pk=${pk}" >&2
    cat "$resp_file" >&2
    rm -f "$resp_file"
    return 1
  fi
  if jq -e '.results[0].type == "error"' <"$resp_file" >/dev/null 2>&1; then
    echo "doc insert SQL error for pk=${pk}:" >&2
    jq '.results[0]' <"$resp_file" >&2
    rm -f "$resp_file"
    return 1
  fi
  rm -f "$resp_file"
}

seed_project_doc() {
  local db_url="$1" db_token="$2"
  local project_id="$3" owner_github_id="$4" name="$5"
  local created_at data
  created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  data="$(jq -nc \
    --arg pid "$project_id" \
    --argjson owner "$owner_github_id" \
    --arg name "$name" \
    --arg created "$created_at" \
    '{project_id:$pid, owner_github_id:$owner, name:$name, created_at:$created}')"
  echo ">> seed ProjectDoc project_id=${project_id} (insert-only)"
  __insert_doc_if_missing "$db_url" "$db_token" "ProjectDoc/project_id=${project_id}" "" "$data"
}

seed_fn0_wasmtime_version() {
  local db_url="$1" db_token="$2" version="$3"
  local data
  data="$(jq -nc --arg v "$version" '{active:$v, pending:null}')"
  echo ">> seed Fn0WasmtimeVersionDoc active=${version} (insert-only)"
  __insert_doc_if_missing "$db_url" "$db_token" "Fn0WasmtimeVersionDoc" "" "$data"
}

seed_compiled_bundle() {
  local db_url="$1" db_token="$2" project_id="$3" code_version="$4" wasmtime="$5"
  local cv_padded
  printf -v cv_padded '%020d' "$code_version"
  local pk="CompiledBundleDoc/project_id=${project_id}&code_version=${cv_padded}"
  local data
  data="$(jq -nc \
    --arg pid "$project_id" \
    --argjson cv "$code_version" \
    --arg w "$wasmtime" \
    '{project_id:$pid, code_version:$cv, fn0_wasmtime_versions:[$w]}')"
  echo ">> seed CompiledBundleDoc project=${project_id} code_version=${code_version}"
  __upsert_doc "$db_url" "$db_token" "$pk" "" "$data"
}

seed_worker_manifest() {
  local db_url="$1" db_token="$2" project_id="$3" code_version="$4" custom_domain="$5"
  # Read existing manifest (if any), upsert this project's entry while
  # preserving every other project's entry, then write back. The previous
  # implementation overwrote the whole doc with a single-entry manifest, so
  # any second invocation evicted every other project from worker routing.
  # `bootstrap-fn0-control.sh` is called for redeploys too, so this MUST be
  # idempotent without dropping siblings.
  #
  # manifest_version must monotonically increase across runs; worker's
  # manifest_poller dedupes on it and would skip a new bundle otherwise.
  local https_url existing_data merged data
  https_url="${db_url/libsql:\/\//https://}"
  https_url="${https_url%/}"

  existing_data="$(__select_doc_data "$https_url" "$db_token" "WorkerManifestDoc" "")"
  if [[ -z "$existing_data" ]]; then
    existing_data='{"manifest_version":0,"project_manifests":{}}'
  fi

  merged="$(jq -c \
    --arg pid "$project_id" \
    --argjson cv "$code_version" \
    --arg dom "$custom_domain" \
    --argjson floor "$code_version" '
      .project_manifests[$pid] = {code_version:$cv, custom_domain:(if $dom == "" then null else $dom end)}
      | .manifest_version = ([(.manifest_version // 0) + 1, $floor] | max)
    ' <<<"$existing_data")"

  local mv
  mv="$(jq -r '.manifest_version' <<<"$merged")"
  echo ">> seed WorkerManifestDoc project=${project_id} domain=${custom_domain:-<none>} manifest_version=${mv} (siblings preserved)"
  __upsert_doc "$db_url" "$db_token" "WorkerManifestDoc" "" "$merged"
}

# __select_doc_data <https_url> <db_token> <pk> <sk>
# Echoes the doc's data column as utf-8 JSON, or empty string if no row.
__select_doc_data() {
  local url="$1" token="$2" pk="$3" sk="$4"
  local sql="SELECT data FROM docs WHERE pk = ? AND sk = ?"
  local req resp_file http_code
  req="$(jq -nc --arg sql "$sql" --arg pk "$pk" --arg sk "$sk" '{requests:[
    {type:"execute",stmt:{sql:$sql,args:[
      {type:"text",value:$pk},
      {type:"text",value:$sk}
    ]}},
    {type:"close"}
  ]}')"
  resp_file="$(mktemp)"
  http_code="$(curl -sS -o "$resp_file" -w '%{http_code}' \
    -X POST "${url}/v2/pipeline" \
    -H "Authorization: Bearer ${token}" \
    -H "Content-Type: application/json" \
    --data "$req")"
  if [[ "$http_code" != "200" ]] || ! jq -e '.results[0].type == "ok"' <"$resp_file" >/dev/null 2>&1; then
    cat "$resp_file" >&2
    rm -f "$resp_file"
    echo "__select_doc_data HTTP ${http_code}" >&2
    return 1
  fi
  local cell_type cell_value
  cell_type="$(jq -r '.results[0].response.result.rows[0][0].type // empty' <"$resp_file")"
  if [[ -z "$cell_type" ]]; then
    rm -f "$resp_file"
    return 0
  fi
  if [[ "$cell_type" == "blob" ]]; then
    # turso emits unpadded base64; macOS `base64 -d` silently drops trailing
    # bytes when input length isn't a multiple of 4, so pad with '=' first.
    jq -r '.results[0].response.result.rows[0][0].base64' <"$resp_file" \
      | awk '{ p=(4-length($0)%4)%4; s=""; for(i=0;i<p;i++) s=s"="; print $0 s }' \
      | base64 -d
  else
    jq -r '.results[0].response.result.rows[0][0].value' <"$resp_file"
  fi
  rm -f "$resp_file"
}

seed_target_fn0_worker_config() {
  local db_url="$1" db_token="$2" image_ref="$3"
  local data
  data="$(jq -nc --arg r "$image_ref" '{image_ref:$r}')"
  echo ">> seed TargetFn0WorkerConfigDoc image_ref=${image_ref}"
  __upsert_doc "$db_url" "$db_token" "TargetFn0WorkerConfigDoc" "" "$data"
}
