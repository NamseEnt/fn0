#!/usr/bin/env bash
#
# scale-config.sh - Manage per-site ScaleConfig stored in DocDB (Turso).
#
# Prerequisites:
#   - Pulumi CLI logged in with access to the fn0Cloud stack
#   - curl, python3
#
# Commands:
#   get <site_name>              Print the scale config for the given site.
#   set <site_name> '<json>'     Validate and write a new scale config for the given site.
#
# The site_name must match the Pulumi name of the OciFn0WorkerSite component
# in infra/cloud/index.ts (currently "oci-fn0-worker-site").
#
# Fields (all required, integer >= 1):
#   instances_per_gb               Max instances per GB of host memory.
#   instances_per_core             Max instances per physical CPU core.
#   scale_out_threshold_percent    Add hosts when utilisation exceeds this % (1-100).
#   scale_in_threshold_percent     Remove hosts when utilisation drops below this % (1-100).
#                                  Must be less than scale_out_threshold_percent.
#   scale_out_cooldown_secs        Seconds to wait after a scale-out before another.
#   scale_in_cooldown_secs         Seconds to wait after a scale-in before another.
#   scale_in_threshold_ticks       Consecutive ticks below threshold before scale-in fires.
#   max_hosts                      Upper bound on running hosts.
#   min_hosts                      Lower bound on running hosts. Must be <= max_hosts.
#
# Examples:
#   ./scripts/scale-config.sh get oci-compute-vm
#
#   ./scripts/scale-config.sh set oci-compute-vm '{
#     "instances_per_gb": 4,
#     "instances_per_core": 1,
#     "scale_out_threshold_percent": 80,
#     "scale_in_threshold_percent": 40,
#     "scale_out_cooldown_secs": 60,
#     "scale_in_threshold_ticks": 3,
#     "scale_in_cooldown_secs": 300,
#     "max_hosts": 2,
#     "min_hosts": 2
#   }'
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PULUMI_DIR="$SCRIPT_DIR/../infra/cloud"

get_turso_jwt() {
  local api_token org_slug db_name jwt

  api_token=$(cd "$PULUMI_DIR" && pulumi config get turso:apiToken)
  org_slug=$(cd "$PULUMI_DIR" && pulumi config get fn0Cloud:tursoOrganizationSlug)

  db_name=$(curl -sf "https://api.turso.tech/v1/organizations/${org_slug}/databases" \
    -H "Authorization: Bearer ${api_token}" \
    | python3 -c "import sys,json; dbs=json.load(sys.stdin)['databases']; print(next(d['Name'] for d in dbs if d['Name'].startswith('fn0-doc-db')))")

  jwt=$(curl -sf "https://api.turso.tech/v1/organizations/${org_slug}/databases/${db_name}/auth/tokens" \
    -X POST \
    -H "Authorization: Bearer ${api_token}" \
    | python3 -c "import sys,json; print(json.load(sys.stdin)['jwt'])")

  TURSO_JWT="$jwt"
  TURSO_URL="https://${db_name}-${org_slug}.aws-ap-northeast-1.turso.io"
}

turso_pipeline() {
  local body="$1"
  curl -sf "${TURSO_URL}/v2/pipeline" \
    -H "Authorization: Bearer ${TURSO_JWT}" \
    -H "Content-Type: application/json" \
    -d "$body"
}

cmd_get() {
  local site_name="$1"
  get_turso_jwt

  local pk="scale-config:${site_name}"
  local body
  body=$(python3 -c "
import json, sys
pk = sys.argv[1]
payload = {
    'requests': [
        {'type': 'execute', 'stmt': {
            'sql': 'SELECT value FROM docs WHERE pk = ? AND sk = 0',
            'args': [{'type': 'text', 'value': pk}],
        }},
        {'type': 'close'},
    ]
}
print(json.dumps(payload))
" "$pk")

  local result
  result=$(turso_pipeline "$body")

  echo "$result" | python3 -c "
import sys, json
data = json.load(sys.stdin)
rows = data['results'][0]['response']['result']['rows']
if not rows:
    print('No scale config set for this site')
else:
    config = json.loads(rows[0][0]['value'])
    print(json.dumps(config, indent=2))
"
}

cmd_set() {
  local site_name="$1"
  local json_arg="${2:-}"
  if [ -z "$json_arg" ]; then
    echo "Error: JSON argument required. Run '$0' without arguments to see usage."
    exit 1
  fi

  python3 -c "
import json, sys
required = [
    'instances_per_gb', 'instances_per_core',
    'scale_out_threshold_percent', 'scale_in_threshold_percent',
    'scale_out_cooldown_secs', 'scale_in_threshold_ticks',
    'scale_in_cooldown_secs', 'max_hosts', 'min_hosts',
]
config = json.loads(sys.argv[1])
for key in required:
    if key not in config:
        print(f'Missing required field: {key}', file=sys.stderr)
        sys.exit(1)
    val = config[key]
    if not isinstance(val, int) or val < 1:
        print(f'{key} must be an integer >= 1, got: {val}', file=sys.stderr)
        sys.exit(1)
if config['scale_in_threshold_percent'] >= config['scale_out_threshold_percent']:
    print('scale_in_threshold_percent must be < scale_out_threshold_percent', file=sys.stderr)
    sys.exit(1)
if config['min_hosts'] > config['max_hosts']:
    print('min_hosts must be <= max_hosts', file=sys.stderr)
    sys.exit(1)
" "$json_arg"

  get_turso_jwt

  local pk="scale-config:${site_name}"
  local body
  body=$(python3 -c "
import json, sys
pk = sys.argv[1]
value = sys.argv[2]
payload = {
    'requests': [
        {'type': 'execute', 'stmt': {
            'sql': 'REPLACE INTO docs (pk, sk, value) VALUES (?, 0, ?)',
            'args': [
                {'type': 'text', 'value': pk},
                {'type': 'text', 'value': value},
            ],
        }},
        {'type': 'close'},
    ]
}
print(json.dumps(payload))
" "$pk" "$json_arg")

  turso_pipeline "$body" > /dev/null

  echo "Scale config for '${site_name}' updated."
  cmd_get "$site_name"
}

case "${1:-}" in
  get)
    if [ -z "${2:-}" ]; then
      echo "Error: site_name required. Usage: $0 get <site_name>"
      exit 1
    fi
    cmd_get "$2"
    ;;
  set)
    if [ -z "${2:-}" ]; then
      echo "Error: site_name required. Usage: $0 set <site_name> '<json>'"
      exit 1
    fi
    cmd_set "$2" "${3:-}"
    ;;
  *)
    sed -n '2,/^set -euo/{ /^#/s/^# \?//p }' "$0"
    exit 1
    ;;
esac
