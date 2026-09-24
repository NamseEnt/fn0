#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
temporary_dir="$(mktemp -d)"
trap 'rm -rf "$temporary_dir"' EXIT

test_repo="${temporary_dir}/repo"
mkdir -p "${test_repo}/scripts/lib"
cp "${REPO_ROOT}/scripts/migrate-project-telemetry-policies.sh" "${test_repo}/scripts/"

cat >"${test_repo}/scripts/lib/pulumi-outputs.sh" <<'EOF'
need() {
  command -v "$1" >/dev/null 2>&1
}

load_pulumi_outputs() {
  PULUMI_OUTPUTS_JSON='{"signyUrl":"https://signy.example","signyAccessClientId":"id","signyAccessClientSecret":"secret"}'
}

pulumi_pick() {
  if [[ "$TEST_BACKEND" == dodb && ( "$1" == controlDbUrl || "$1" == forteDbGroupToken ) ]]; then
    printf '%s\n' "unexpected Turso output request: $1" >>"$TEST_CALL_LOG"
    return 1
  fi
  jq -r ".${1} // empty" <<<"$PULUMI_OUTPUTS_JSON"
}

require_pulumi_output() {
  local name
  for name in "$@"; do
    [[ -n "$(pulumi_pick "$name")" ]]
  done
}
EOF

cat >"${test_repo}/scripts/lib/control-db.sh" <<'EOF'
control_db_init() {
  CONTROL_DB_BACKEND="$TEST_BACKEND"
  CONTROL_DB_URL="https://control.example"
  CONTROL_DB_TOKEN="token"
}

control_db_scan() {
  local after_pk="$1" after_sk="$2" limit="$3" has_cursor="$4"
  printf 'scan %s %s %s %s\n' "$has_cursor" "$after_pk" "$after_sk" "$limit" >>"$TEST_CALL_LOG"
  jq -c --arg after_pk "$after_pk" --arg after_sk "$after_sk" --argjson limit "$limit" --arg has_cursor "$has_cursor" '{documents:(sort_by(.pk,.sk) | map(select($has_cursor != "true" or .pk > $after_pk or (.pk == $after_pk and .sk > $after_sk))) | .[:$limit])}' "$TEST_DOCUMENTS"
}

control_db_get_observed() {
  printf 'get %s %s\n' "$1" "$2" >>"$TEST_CALL_LOG"
  local encoded
  encoded="$(jq -r --arg pk "$1" --arg sk "$2" '.[] | select(.pk == $pk and .sk == $sk) | .data_base64' "$TEST_DOCUMENTS")"
  jq -nc --arg data "$encoded" '{found:true,revision:41,data_base64:$data}'
}
EOF

cat >"${temporary_dir}/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"$TEST_CALL_LOG"
if [[ "$*" == *"/v2/pipeline"* ]]; then
  output_file=""
  request=""
  previous=""
  for argument in "$@"; do
    if [[ "$previous" == -o ]]; then output_file="$argument"; fi
    if [[ "$previous" == --data-raw ]]; then request="$argument"; fi
    previous="$argument"
  done
  sql="$(jq -r '.requests[0].stmt.sql' <<<"$request")"
  case "$sql" in
    'SELECT COUNT(*)'*) response='{"results":[{"type":"ok","response":{"result":{"rows":[[{"value":"1"}]]}}}]}' ;;
    *"ProjectDoc/%"*) response="$(jq -nc --arg pk 'ProjectDoc/project-a' --arg data "$TEST_TURSO_PROJECT_DATA" '{results:[{type:"ok",response:{result:{rows:[[{value:$pk},{value:$data},{value:"1"}]]}}}]}')" ;;
    *"TelemetryPolicyOutboxDoc/%"*) response='{"results":[{"type":"ok","response":{"result":{"rows":[]}}}]}' ;;
    *) exit 1 ;;
  esac
  printf '%s\n' "$response" >"$output_file"
  printf '%s\n' '200'
else
  output_file=""
  previous=""
  for argument in "$@"; do
    if [[ "$previous" == -o ]]; then output_file="$argument"; fi
    previous="$argument"
  done
  printf '%s\n' "${TEST_SIGNY_POLICY:-{\"revision\":1,\"retention\":\"30d\",\"max_stored_bytes\":\"512MiB\"}}" >"$output_file"
  printf '%s\n' '200'
fi
EOF
chmod +x "${temporary_dir}/curl"

policy='{"revision":1,"base_retention":"30d","log_retention_override":null,"trace_retention_override":null,"metric_retention_override":null,"max_stored_bytes":"512MiB"}'
project_data="$(jq -nc --argjson policy "$policy" '{project_id:"project-a",telemetry_policy:$policy}')"
outbox_data="$(jq -nc --argjson policy "$policy" '{project_id:"project-a",policy_revision:1,policy:$policy,state:"applied"}')"
project_b64="$(printf '%s' "$project_data" | base64 | tr -d '\n')"
outbox_b64="$(printf '%s' "$outbox_data" | base64 | tr -d '\n')"
documents_file="${temporary_dir}/documents.json"
call_log="${temporary_dir}/calls.log"
export TEST_DOCUMENTS="$documents_file" TEST_CALL_LOG="$call_log"
export PATH="${temporary_dir}:$PATH"
export TEST_TURSO_PROJECT_DATA="$project_data"
export TEST_SIGNY_POLICY

run_telemetry() {
  TEST_BACKEND="$1" bash "${test_repo}/scripts/migrate-project-telemetry-policies.sh" "${@:2}"
}

jq -nc --arg project "$project_b64" --arg outbox "$outbox_b64" '[{pk:"ProjectDoc/project-a",sk:"",data_base64:$project},{pk:"TelemetryPolicyOutboxDoc/project_id=project-a",sk:"",data_base64:$outbox}]' >"$documents_file"
: >"$call_log"
result="$(run_telemetry dodb --check-schema)"
[[ "$(jq -r '.missing_policies' <<<"$result")" == 0 ]]
rg -q 'unexpected Turso output request' "$call_log" && exit 1

: >"$call_log"
result="$(run_telemetry turso --check-schema)"
[[ "$(jq -r '.projects' <<<"$result")" == 1 ]]
[[ "$(rg -c '/v2/pipeline' "$call_log")" == 3 ]]

missing_policy_data='{"project_id":"project-a"}'
missing_policy_b64="$(printf '%s' "$missing_policy_data" | base64 | tr -d '\n')"
jq -nc --arg project "$missing_policy_b64" '[{pk:"ProjectDoc/project-a",sk:"",data_base64:$project}]' >"$documents_file"
if run_telemetry dodb --check-schema >/dev/null 2>&1; then exit 1; fi

jq -nc --arg project "$project_b64" --arg outbox "$outbox_b64" '[{pk:"ProjectDoc/project-a",sk:"",data_base64:$project},{pk:"TelemetryPolicyOutboxDoc/project_id=project-a",sk:"",data_base64:$outbox}]' >"$documents_file"
result="$(run_telemetry dodb --check)"
[[ "$(jq -r '.missing_outboxes' <<<"$result")" == 0 ]]
[[ "$(jq -r '.unsettled_outboxes' <<<"$result")" == 0 ]]
plan="$(run_telemetry dodb)"
[[ "$(jq -r '.project_id' <<<"$plan" | tail -n 1)" == project-a ]]
[[ "$(jq -r '.revision' <<<"$plan" | tail -n 1)" == 41 ]]

jq -nc --arg project "$project_b64" '[{pk:"ProjectDoc/project-a",sk:"",data_base64:$project}]' >"$documents_file"
if run_telemetry dodb --check >/dev/null 2>&1; then exit 1; fi

jq -nc --arg project "$missing_policy_b64" '[{pk:"ProjectDoc/project-a",sk:"",data_base64:$project}]' >"$documents_file"
if run_telemetry dodb --check >/dev/null 2>&1; then exit 1; fi

jq -nc --arg project "$project_b64" --arg outbox "$outbox_b64" '[{pk:"ProjectDoc/project-a",sk:"",data_base64:$project},{pk:"TelemetryPolicyOutboxDoc/project_id=project-a",sk:"",data_base64:$outbox}]' >"$documents_file"
pending_outbox="$(jq -nc --argjson policy "$policy" '{project_id:"project-a",policy_revision:1,policy:$policy,state:"pending"}')"
pending_b64="$(printf '%s' "$pending_outbox" | base64 | tr -d '\n')"
jq -nc --arg project "$project_b64" --arg outbox "$pending_b64" '[{pk:"ProjectDoc/project-a",sk:"",data_base64:$project},{pk:"TelemetryPolicyOutboxDoc/project_id=project-a",sk:"",data_base64:$outbox}]' >"$documents_file"
if run_telemetry dodb --check >/dev/null 2>&1; then exit 1; fi

: >"$call_log"
if run_telemetry dodb --apply --backup-dir "${temporary_dir}/backup" >/dev/null 2>&1; then exit 1; fi
[[ ! -s "$call_log" ]]

jq -nc --arg encoded "$project_b64" '[range(0;501) as $index | {pk:("ProjectDoc/" + (("0000" + ($index|tostring))[-4:])),sk:"",data_base64:$encoded}]' >"$documents_file"
: >"$call_log"
result="$(run_telemetry dodb --check-schema)"
[[ "$(jq -r '.projects' <<<"$result")" == 501 ]]
[[ "$(rg -c '^scan ' "$call_log")" == 2 ]]
[[ "$(tail -n 1 "$call_log" | awk '{print $1, $2, $3}')" == 'scan true ProjectDoc/0499' ]]

jq -nc --arg project "$project_b64" --arg outbox "$outbox_b64" '[{pk:"ProjectDoc/project-a",sk:"",data_base64:$project}] + [range(0;501) as $index | {pk:("TelemetryPolicyOutboxDoc/project_id=" + (("0000" + ($index|tostring))[-4:])),sk:"",data_base64:$outbox}] + [{pk:"TelemetryPolicyOutboxDoc/project_id=project-a",sk:"",data_base64:$outbox}]' >"$documents_file"
: >"$call_log"
result="$(run_telemetry dodb --check)"
[[ "$(jq -r '.missing_outboxes' <<<"$result")" == 0 ]]
[[ "$(rg -c '^scan ' "$call_log")" == 2 ]]
[[ "$(tail -n 1 "$call_log" | awk '{print $1, $2, $3}')" == 'scan true TelemetryPolicyOutboxDoc/project_id=0498' ]]

jq -nc --arg project "$project_b64" --arg outbox "$outbox_b64" '[{pk:"ProjectDoc/project-a",sk:"",data_base64:$project},{pk:"TelemetryPolicyOutboxDoc/project_id=project-a",sk:"",data_base64:$outbox}]' >"$documents_file"
TEST_SIGNY_POLICY='{"retention":"30d","max_stored_bytes":"512MiB"}'
if run_telemetry dodb --check >/dev/null 2>&1; then exit 1; fi
unset TEST_SIGNY_POLICY

printf '%s\n' 'telemetry policy shell tests passed'
