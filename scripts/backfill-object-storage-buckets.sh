#!/usr/bin/env bash
# Creates the per-project object-storage R2 bucket (fn0-object-storage-<id>)
# for every project already present in control's doc-db.
#
# New projects get their bucket on deploy via ensure_object_storage_bucket
# (control deploy action). This script backfills projects that were deployed
# before object storage existed. Idempotent — re-running skips buckets that
# already exist.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT

# shellcheck source=lib/pulumi-outputs.sh
source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"

need pulumi
need jq
need curl

load_pulumi_outputs
require_pulumi_output \
  controlDbUrl \
  forteDbGroupToken \
  staticAssetAccountId \
  staticAssetCloudflareApiToken

control_db_url="$(pulumi_pick controlDbUrl)"
control_db_url="${control_db_url%/}"
db_token="$(pulumi_pick forteDbGroupToken)"
account_id="$(pulumi_pick staticAssetAccountId)"
cf_token="$(pulumi_pick staticAssetCloudflareApiToken)"

echo ">> Listing projects from control doc-db"
query="$(jq -nc \
  --arg sql "SELECT pk FROM docs WHERE pk GLOB 'ProjectDoc/project_id=*'" \
  '{requests: [
    {type: "execute", stmt: {sql: $sql}},
    {type: "close"}
  ]}')"

resp_file="$(mktemp)"
http_code="$(curl -sS -o "$resp_file" -w '%{http_code}' \
  -X POST "${control_db_url}/v2/pipeline" \
  -H "Authorization: Bearer ${db_token}" \
  -H "Content-Type: application/json" \
  --data-raw "$query")"
if [[ "$http_code" != "200" ]]; then
  cat "$resp_file" >&2
  rm -f "$resp_file"
  echo "doc-db query failed (HTTP ${http_code})" >&2
  exit 1
fi

project_ids=()
while IFS= read -r project_id_line; do
  project_ids+=("$project_id_line")
done < <(jq -r '
  .results[0].response.result.rows[]?[0]
  | (.value // "")
  | sub("^ProjectDoc/project_id="; "")
  | select(. != "")
' <"$resp_file")
rm -f "$resp_file"

if [[ "${#project_ids[@]}" -eq 0 ]]; then
  echo ">> No projects found; nothing to backfill"
  exit 0
fi
echo ">> ${#project_ids[@]} project(s) found"

created=0
existing=0
for project_id in "${project_ids[@]}"; do
  bucket="fn0-object-storage-${project_id}"
  body="$(jq -nc --arg name "$bucket" '{name: $name, locationHint: "apac"}')"
  resp_file="$(mktemp)"
  http_code="$(curl -sS -o "$resp_file" -w '%{http_code}' \
    -X POST "https://api.cloudflare.com/client/v4/accounts/${account_id}/r2/buckets" \
    -H "Authorization: Bearer ${cf_token}" \
    -H "Content-Type: application/json" \
    --data "$body")"
  if [[ "$http_code" =~ ^2 ]]; then
    echo "   created ${bucket}"
    created=$((created + 1))
  elif grep -qiE 'already exists|already in use|duplicate' "$resp_file"; then
    echo "   exists  ${bucket}"
    existing=$((existing + 1))
  else
    cat "$resp_file" >&2
    rm -f "$resp_file"
    echo "create ${bucket} failed (HTTP ${http_code})" >&2
    exit 1
  fi
  rm -f "$resp_file"
done

echo ">> Done: ${created} created, ${existing} already existed"
