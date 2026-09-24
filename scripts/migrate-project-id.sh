#!/usr/bin/env bash

set -euo pipefail
umask 077

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
temporary_dir="$(mktemp -d)"
chmod 700 "${temporary_dir}"
trap 'rm -rf "${temporary_dir}"' EXIT

if [[ "$#" -lt 1 ]]; then
  echo "usage: scripts/migrate-project-id.sh plan|apply|verify --old-id ID --new-id ID [--apply]" >&2
  exit 2
fi

command_name="$1"
shift
case "${command_name}" in
  plan|apply|verify) ;;
  *) echo "unsupported migration command: ${command_name}" >&2; exit 2 ;;
esac

if [[ "${command_name}" == "apply" ]]; then
  apply_found=false
  for argument in "$@"; do
    if [[ "${argument}" == "--apply" ]]; then apply_found=true; fi
  done
  if [[ "${apply_found}" != "true" ]]; then
    echo "apply requires the explicit --apply flag" >&2
    exit 2
  fi
fi

pulumi_dir="${PULUMI_DIR:-${REPO_ROOT}/infra/cloud}"
(cd "${pulumi_dir}" && pulumi stack output --show-secrets --json) >"${temporary_dir}/outputs.json"
org_slug="$(cd "${pulumi_dir}" && pulumi config get fn0Cloud:tursoOrganizationSlug)"
group_name="$(awk -F: '/^FN0_TURSO_GROUP_NAME:/ { sub(/^[[:space:]]+/, "", $2); print $2; exit }' "${REPO_ROOT}/fn0/control/env.yaml")"
[[ -n "${org_slug}" && -n "${group_name}" ]] || {
  echo "missing Turso management configuration" >&2
  exit 1
}
(cd "${pulumi_dir}" && pulumi config get turso:apiToken) >"${temporary_dir}/api-token"
chmod 600 "${temporary_dir}/api-token"

python3 - "${temporary_dir}/outputs.json" "${temporary_dir}/secrets.json" "${temporary_dir}/api-token" "${org_slug}" "${group_name}" <<'PY'
import json
import os
import sys

with open(sys.argv[1], encoding="utf-8") as source:
    outputs = json.load(source)
with open(sys.argv[3], encoding="utf-8") as token_source:
    api_token = token_source.read().strip()
secrets = {
    "api_token": api_token,
    "org_slug": sys.argv[4],
    "group_name": sys.argv[5],
    "outputs": outputs,
}
with open(sys.argv[2], "w", encoding="utf-8") as destination:
    json.dump(secrets, destination)
os.chmod(sys.argv[2], 0o600)
PY
rm -f "${temporary_dir}/outputs.json"
export FN0_PROJECT_ID_MIGRATION_SECRETS="${temporary_dir}/secrets.json"
python3 "${REPO_ROOT}/scripts/lib/project-id-migrate.py" "${command_name}" "$@"
