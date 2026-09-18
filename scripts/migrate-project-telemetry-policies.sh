#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT

source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"

mode="plan"
backup_dir=""

while (( $# > 0 )); do
  case "$1" in
    --apply)
      mode="apply"
      shift
      ;;
    --check)
      mode="check"
      shift
      ;;
    --check-schema)
      mode="check-schema"
      shift
      ;;
    --backup-dir)
      [[ $# -ge 2 ]] || { echo "--backup-dir requires a path" >&2; exit 1; }
      backup_dir="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ "$mode" == "apply" && -z "$backup_dir" ]]; then
  echo "--apply requires --backup-dir" >&2
  exit 1
fi

need curl
need jq
need base64

load_pulumi_outputs
require_pulumi_output controlDbUrl forteDbGroupToken signyUrl signyAccessClientId signyAccessClientSecret

control_db_url="$(pulumi_pick controlDbUrl)"
control_db_token="$(pulumi_pick forteDbGroupToken)"
signy_url="$(pulumi_pick signyUrl)"
signy_access_client_id="$(pulumi_pick signyAccessClientId)"
signy_access_client_secret="$(pulumi_pick signyAccessClientSecret)"
control_db_url="${control_db_url%/}"
signy_url="${signy_url%/}"

work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT
curl_args=(--connect-timeout 10 --max-time 30 --retry 2 --retry-delay 1 --retry-all-errors)

db_pipeline() {
  local request="$1"
  local response_file="$work_dir/db-response.json"
  local http_code
  http_code="$(curl "${curl_args[@]}" -sS -o "$response_file" -w '%{http_code}' \
    -X POST "${control_db_url}/v2/pipeline" \
    -H "Authorization: Bearer ${control_db_token}" \
    -H "Content-Type: application/json" \
    --data-raw "$request")"
  if [[ "$http_code" != "200" ]]; then
    jq . "$response_file" >&2 || true
    echo "control DB answered HTTP ${http_code}" >&2
    return 1
  fi
  if jq -e '.results[] | select(.type == "error")' "$response_file" >/dev/null; then
    jq '.results[] | select(.type == "error")' "$response_file" >&2
    return 1
  fi
  jq -c . "$response_file"
}

db_query() {
  local sql="$1"
  local request
  request="$(jq -nc --arg sql "$sql" '{requests:[{type:"execute",stmt:{sql:$sql}},{type:"close"}]}')"
  db_pipeline "$request"
}

signy_request() {
  local method="$1"
  local path="$2"
  local body="${3:-}"
  local response_file="$work_dir/signy-response.json"
  local http_code
  if [[ -n "$body" ]]; then
    http_code="$(curl "${curl_args[@]}" -sS -o "$response_file" -w '%{http_code}' \
      -X "$method" "${signy_url}/signy/api/v1/admin/${path}" \
      -H "CF-Access-Client-Id: ${signy_access_client_id}" \
      -H "CF-Access-Client-Secret: ${signy_access_client_secret}" \
      -H "Content-Type: application/json" \
      --data-raw "$body")"
  else
    http_code="$(curl "${curl_args[@]}" -sS -o "$response_file" -w '%{http_code}' \
      -X "$method" "${signy_url}/signy/api/v1/admin/${path}" \
      -H "CF-Access-Client-Id: ${signy_access_client_id}" \
      -H "CF-Access-Client-Secret: ${signy_access_client_secret}")"
  fi
  if [[ "$http_code" != "200" ]]; then
    cat "$response_file" >&2
    echo "Signy ${method} ${path} answered HTTP ${http_code}" >&2
    return 1
  fi
  jq -c . "$response_file"
}

normalize_signy_policy() {
  jq -c '{
    revision: ((.revision // 0) | if . < 1 then 1 else . end),
    base_retention: .retention,
    log_retention_override: (.log_retention // null),
    trace_retention_override: (.trace_retention // null),
    metric_retention_override: (.metric_retention // null),
    max_stored_bytes: (.max_stored_bytes // "unlimited")
  }'
}

policy_to_signy_body() {
  jq -c '{
    revision,
    retention: .base_retention,
    log_retention: .log_retention_override,
    trace_retention: .trace_retention_override,
    metric_retention: .metric_retention_override,
    max_stored_bytes
  }'
}

table_response="$(db_query "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'docs'")"
table_count="$(jq -r '.results[0].response.result.rows[0][0].value' <<<"$table_response")"
if [[ "$table_count" == "0" ]]; then
  jq -nc --arg mode "$mode" '{mode:$mode,projects:0,missing_policies:0,missing_outboxes:0,unsettled_outboxes:0,legacy_signy_policies:0}'
  if [[ "$mode" == "apply" ]]; then
    echo "control DB has no docs table" >&2
    exit 1
  fi
  exit 0
fi

project_response="$(db_query "SELECT pk, CAST(data AS TEXT), version FROM docs WHERE pk LIKE 'ProjectDoc/%' ORDER BY pk")"
outbox_response="$(db_query "SELECT pk, CAST(data AS TEXT), version FROM docs WHERE pk LIKE 'TelemetryPolicyOutboxDoc/%' ORDER BY pk")"

project_rows=()
while IFS= read -r project_row; do
  project_rows[${#project_rows[@]}]="$project_row"
done < <(jq -c '.results[0].response.result.rows[] | {pk:.[0].value,data:(.[1].value|fromjson),version:(.[2].value|tonumber)}' <<<"$project_response")
outbox_rows="$(jq -c '[.results[0].response.result.rows[] | {pk:.[0].value,data:(.[1].value|fromjson),version:(.[2].value|tonumber)}]' <<<"$outbox_response")"

inventory_file="$work_dir/inventory.jsonl"
: > "$inventory_file"

for project_row in "${project_rows[@]}"; do
  project_id="$(jq -r '.data.project_id' <<<"$project_row")"
  project_pk="$(jq -r '.pk' <<<"$project_row")"
  project_version="$(jq -r '.version' <<<"$project_row")"
  existing_policy="$(jq -c '.data.telemetry_policy // null' <<<"$project_row")"
  signy_original="$(signy_request GET "tenants/${project_id}/retention")"
  signy_policy="$(normalize_signy_policy <<<"$signy_original")"
  outbox_pk="TelemetryPolicyOutboxDoc/project_id=${project_id}"
  outbox_row="$(jq -c --arg pk "$outbox_pk" 'map(select(.pk == $pk))[0] // null' <<<"$outbox_rows")"
  if [[ "$existing_policy" != "null" && "$existing_policy" != "$signy_policy" ]]; then
    echo "ProjectDoc and Signy policy differ for ${project_id}" >&2
    exit 1
  fi
  if [[ "$outbox_row" != "null" ]]; then
    outbox_revision="$(jq -r '.data.policy_revision' <<<"$outbox_row")"
    policy_revision="$(jq -r '.revision' <<<"$signy_policy")"
    if (( outbox_revision > policy_revision )); then
      echo "outbox revision is newer than policy for ${project_id}" >&2
      exit 1
    fi
  fi
  jq -nc \
    --arg project_id "$project_id" \
    --arg project_pk "$project_pk" \
    --argjson project_version "$project_version" \
    --argjson project_data "$(jq -c '.data' <<<"$project_row")" \
    --argjson policy "$signy_policy" \
    --argjson signy_original "$signy_original" \
    --argjson outbox "$outbox_row" \
    '{project_id:$project_id,project_pk:$project_pk,project_version:$project_version,project_data:$project_data,policy:$policy,signy_original:$signy_original,outbox:$outbox}' \
    >> "$inventory_file"
done

inventory="$(jq -s '.' "$inventory_file")"
project_count="$(jq 'length' <<<"$inventory")"
missing_count="$(jq '[.[] | select(.project_data.telemetry_policy == null)] | length' <<<"$inventory")"
missing_outbox_count="$(jq '[.[] | select(.outbox == null)] | length' <<<"$inventory")"
unsettled_outbox_count="$(jq '[.[] | select(.outbox != null and (.outbox.data.state != "applied" or .outbox.data.policy_revision < .policy.revision))] | length' <<<"$inventory")"
legacy_signy_count="$(jq '[.[] | select(.signy_original.revision == null)] | length' <<<"$inventory")"

jq -nc \
  --arg mode "$mode" \
  --argjson projects "$project_count" \
  --argjson missing_policies "$missing_count" \
  --argjson missing_outboxes "$missing_outbox_count" \
  --argjson unsettled_outboxes "$unsettled_outbox_count" \
  --argjson legacy_signy_policies "$legacy_signy_count" \
  '{mode:$mode,projects:$projects,missing_policies:$missing_policies,missing_outboxes:$missing_outboxes,unsettled_outboxes:$unsettled_outboxes,legacy_signy_policies:$legacy_signy_policies}'

if [[ "$mode" == "check-schema" ]]; then
  if (( missing_count > 0 )); then
    exit 1
  fi
  exit 0
fi

if [[ "$mode" == "check" ]]; then
  if (( missing_count > 0 || missing_outbox_count > 0 || unsettled_outbox_count > 0 || legacy_signy_count > 0 )); then
    exit 1
  fi
  exit 0
fi

if [[ "$mode" == "plan" ]]; then
  jq -c '.[] | {project_id,project_version,policy,needs_project_update:(.project_data.telemetry_policy == null),needs_outbox:(.outbox == null)}' <<<"$inventory"
  exit 0
fi

mkdir -p "$backup_dir"
if [[ -n "$(find "$backup_dir" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
  echo "backup directory is not empty: ${backup_dir}" >&2
  exit 1
fi
printf '%s\n' "$inventory" > "$backup_dir/inventory.json"
printf '%s\n' "$project_response" > "$backup_dir/project-docs-response.json"
printf '%s\n' "$outbox_response" > "$backup_dir/outboxes-response.json"

inventory_rows=()
while IFS= read -r inventory_row; do
  inventory_rows[${#inventory_rows[@]}]="$inventory_row"
done < <(jq -c '.[]' <<<"$inventory")
for inventory_row in "${inventory_rows[@]}"; do
  project_id="$(jq -r '.project_id' <<<"$inventory_row")"
  if [[ "$(jq -r '.signy_original.revision == null' <<<"$inventory_row")" == "true" ]]; then
    continue
  fi
  requested_policy="$(jq -c '.policy' <<<"$inventory_row")"
  requested_body="$(policy_to_signy_body <<<"$requested_policy")"
  claimed_policy="$(signy_request PUT "project-tenants/${project_id}/retention" "$requested_body" | normalize_signy_policy)"
  if [[ "$claimed_policy" != "$requested_policy" ]]; then
    echo "Signy returned a different policy for ${project_id}" >&2
    exit 1
  fi
done

timestamp="$(date -u +'%Y-%m-%dT%H:%M:%SZ')"
requests='[{"type":"execute","stmt":{"sql":"BEGIN IMMEDIATE"}}]'
project_update_sql="UPDATE docs SET data = ?, version = version + 1 WHERE pk = ? AND sk = '' AND version = ? AND json_type(CAST(data AS TEXT), '$.telemetry_policy') IS NULL"
outbox_insert_sql="INSERT INTO docs (pk, sk, data, version) VALUES (?, '', ?, 0) ON CONFLICT(pk, sk) DO NOTHING"
outbox_update_sql="UPDATE docs SET data = ?, version = version + 1 WHERE pk = ? AND sk = '' AND version = ?"

for inventory_row in "${inventory_rows[@]}"; do
  project_id="$(jq -r '.project_id' <<<"$inventory_row")"
  policy="$(jq -c '.policy' <<<"$inventory_row")"
  project_pk="$(jq -r '.project_pk' <<<"$inventory_row")"
  project_version="$(jq -r '.project_version' <<<"$inventory_row")"
  if [[ "$(jq -r '.project_data.telemetry_policy == null' <<<"$inventory_row")" == "true" ]]; then
    updated_project="$(jq -c --argjson policy "$policy" '.project_data + {telemetry_policy:$policy}' <<<"$inventory_row")"
    updated_project_base64="$(printf '%s' "$updated_project" | base64 | tr -d '\n')"
    statement="$(jq -nc \
      --arg blob "$updated_project_base64" \
      --arg pk "$project_pk" \
      --arg sql "$project_update_sql" \
      --argjson version "$project_version" \
      '{type:"execute",stmt:{sql:$sql,args:[{type:"blob",base64:$blob},{type:"text",value:$pk},{type:"integer",value:($version|tostring)}]}}')"
    requests="$(jq -c --argjson statement "$statement" '. + [$statement]' <<<"$requests")"
  fi
  if [[ "$(jq -r '.outbox == null' <<<"$inventory_row")" == "true" ]]; then
    policy_revision="$(jq -r '.revision' <<<"$policy")"
    outbox_pk="TelemetryPolicyOutboxDoc/project_id=${project_id}"
    outbox_data="$(jq -nc \
      --arg project_id "$project_id" \
      --argjson policy_revision "$policy_revision" \
      --argjson policy "$policy" \
      --arg timestamp "$timestamp" \
      '{project_id:$project_id,policy_revision:$policy_revision,policy:$policy,state:"pending",attempts:0,last_error:null,pending_since:$timestamp,updated_at:$timestamp}')"
    outbox_base64="$(printf '%s' "$outbox_data" | base64 | tr -d '\n')"
    statement="$(jq -nc \
      --arg pk "$outbox_pk" \
      --arg blob "$outbox_base64" \
      --arg sql "$outbox_insert_sql" \
      '{type:"execute",stmt:{sql:$sql,args:[{type:"text",value:$pk},{type:"blob",base64:$blob}]}}')"
    requests="$(jq -c --argjson statement "$statement" '. + [$statement]' <<<"$requests")"
  elif [[ "$(jq -r '(.outbox.data.policy_revision < .policy.revision) or ((.outbox.data.policy_revision == .policy.revision) and (.outbox.data.policy == null))' <<<"$inventory_row")" == "true" ]]; then
    policy_revision="$(jq -r '.policy.revision' <<<"$inventory_row")"
    outbox_pk="$(jq -r '.outbox.pk' <<<"$inventory_row")"
    outbox_version="$(jq -r '.outbox.version' <<<"$inventory_row")"
    outbox_data="$(jq -c \
      --argjson policy_revision "$policy_revision" \
      --argjson policy "$policy" \
      --arg timestamp "$timestamp" \
      '.outbox.data + {policy_revision:$policy_revision,policy:$policy,state:"pending",last_error:null,pending_since:(.outbox.data.pending_since // $timestamp),updated_at:$timestamp}' \
      <<<"$inventory_row")"
    outbox_base64="$(printf '%s' "$outbox_data" | base64 | tr -d '\n')"
    statement="$(jq -nc \
      --arg pk "$outbox_pk" \
      --arg blob "$outbox_base64" \
      --arg sql "$outbox_update_sql" \
      --argjson version "$outbox_version" \
      '{type:"execute",stmt:{sql:$sql,args:[{type:"blob",base64:$blob},{type:"text",value:$pk},{type:"integer",value:($version|tostring)}]}}')"
    requests="$(jq -c --argjson statement "$statement" '. + [$statement]' <<<"$requests")"
  fi
done

requests="$(jq -c '. + [{type:"execute",stmt:{sql:"COMMIT"}},{type:"close"}]' <<<"$requests")"
migration_request="$(jq -nc --argjson requests "$requests" '{requests:$requests}')"
migration_response="$(db_pipeline "$migration_request")"
printf '%s\n' "$migration_request" > "$backup_dir/migration-request.json"
printf '%s\n' "$migration_response" > "$backup_dir/migration-response.json"

echo "migration applied; run --check after the telemetry policy outbox settles"
