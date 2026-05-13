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
  local sql="INSERT INTO docs (pk, sk, data, version) VALUES (?, ?, ?, 0) ON CONFLICT(pk, sk) DO UPDATE SET data = excluded.data, version = docs.version + 1"
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

# forte_doc generates an i64 pk by mapping the value through
# `(value as u64).wrapping_add(2^63)` and then zero-padding to 20 digits.
# Reproduce that mapping here so the pk we INSERT matches what
# UserDocGet / control reads with.
__forte_doc_pk_i64() {
  python3 -c 'import sys; print(format(int(sys.argv[1]) + (1 << 63), "020d"))' "$1"
}

seed_user_doc() {
  local db_url="$1" db_token="$2"
  local github_id="$3" github_login="$4" project_id="$5"
  local pk_id created_at data
  pk_id="$(__forte_doc_pk_i64 "$github_id")"
  created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  data="$(jq -nc \
    --argjson gid "$github_id" \
    --arg login "$github_login" \
    --arg created "$created_at" \
    --arg pid "$project_id" \
    '{github_id:$gid, github_login:$login, created_at:$created, cli_tokens:[], web_sessions:[], projects:[{project_id:$pid, name:$pid}]}')"
  echo ">> seed UserDoc github_id=${github_id}"
  __upsert_doc "$db_url" "$db_token" "UserDoc/github_id=${pk_id}" "" "$data"
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
  echo ">> seed ProjectDoc project_id=${project_id}"
  __upsert_doc "$db_url" "$db_token" "ProjectDoc/project_id=${project_id}" "" "$data"
}

seed_fn0_wasmtime_version() {
  local db_url="$1" db_token="$2" version="$3"
  local data
  data="$(jq -nc --arg v "$version" '{active:$v, pending:null}')"
  echo ">> seed Fn0WasmtimeVersionDoc active=${version}"
  __upsert_doc "$db_url" "$db_token" "Fn0WasmtimeVersionDoc" "" "$data"
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
  # manifest_version must monotonically increase across runs; worker's
  # manifest_poller dedupes on it and would skip a new bundle otherwise.
  # code_version is already epoch-seconds at bootstrap time, so reuse it.
  local data
  data="$(jq -nc \
    --argjson mv "$code_version" \
    --arg pid "$project_id" \
    --argjson cv "$code_version" \
    --arg dom "$custom_domain" \
    '{manifest_version:$mv, project_manifests:{($pid):{code_version:$cv, custom_domain:$dom}}}')"
  echo ">> seed WorkerManifestDoc project=${project_id} domain=${custom_domain} manifest_version=${code_version}"
  __upsert_doc "$db_url" "$db_token" "WorkerManifestDoc" "" "$data"
}

seed_target_fn0_worker_config() {
  local db_url="$1" db_token="$2" image_ref="$3"
  local data
  data="$(jq -nc --arg r "$image_ref" '{image_ref:$r}')"
  echo ">> seed TargetFn0WorkerConfigDoc image_ref=${image_ref}"
  __upsert_doc "$db_url" "$db_token" "TargetFn0WorkerConfigDoc" "" "$data"
}
