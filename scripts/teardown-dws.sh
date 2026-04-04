#!/usr/bin/env bash
#
# teardown-dws.sh - Remove a Dedicated Worker Server and its Cloudflare Tunnel.
# This is the exact reverse of setup-dws.sh.
#
# Usage:
#   ./scripts/teardown-dws.sh <HOST_ID>
#
set -euo pipefail

if [ -z "${1:-}" ]; then
  echo "Usage: $0 <HOST_ID>"
  exit 1
fi

HOST_ID="$1"
PULUMI_STACK="${PULUMI_STACK:-prod}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$SCRIPT_DIR/../infra/cloud"

echo "=== fn0 Dedicated Worker Server Teardown ==="
echo "Host ID: $HOST_ID"
echo ""

# --- Fetch config from Pulumi ---
CF_ACCOUNT_ID=$(cd "$INFRA_DIR" && pulumi config get fn0Cloud:cloudflareAccountId -s "$PULUMI_STACK")
CF_ZONE_ID=$(cd "$INFRA_DIR" && pulumi config get fn0Cloud:cloudflareZoneId -s "$PULUMI_STACK")
CF_API_TOKEN=$(cd "$INFRA_DIR" && pulumi config get cloudflare:apiToken -s "$PULUMI_STACK")
DOMAIN=$(cd "$INFRA_DIR" && pulumi config get fn0Cloud:domain -s "$PULUMI_STACK")
TURSO_API_TOKEN=$(cd "$INFRA_DIR" && pulumi config get turso:apiToken -s "$PULUMI_STACK")
TURSO_ORG=$(cd "$INFRA_DIR" && pulumi config get fn0Cloud:tursoOrganizationSlug -s "$PULUMI_STACK")

TUNNEL_NAME="dws-${HOST_ID}"
HTTP_HOSTNAME="${HOST_ID}-http.${DOMAIN}"
WS_HOSTNAME="${HOST_ID}-ws.${DOMAIN}"

# --- Step 7 reverse: Remove from DocDB ---
echo "--- Removing from DocDB ---"

DB_NAME=$(curl -sf "https://api.turso.tech/v1/organizations/${TURSO_ORG}/databases" \
  -H "Authorization: Bearer ${TURSO_API_TOKEN}" \
  | python3 -c "import sys,json; print(next(d['Name'] for d in json.load(sys.stdin)['databases'] if d['Name'].startswith('fn0-doc-db')))")

DB_TOKEN=$(curl -sf "https://api.turso.tech/v1/organizations/${TURSO_ORG}/databases/${DB_NAME}/auth/tokens" \
  -X POST \
  -H "Authorization: Bearer ${TURSO_API_TOKEN}" \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['jwt'])")

curl -sf "https://${DB_NAME}-${TURSO_ORG}.aws-ap-northeast-1.turso.io/v2/pipeline" \
  -H "Authorization: Bearer ${DB_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "$(python3 -c "
import json
print(json.dumps({
    'requests': [
        {'type': 'execute', 'stmt': {
            'sql': 'DELETE FROM docs WHERE pk = ?',
            'args': [{'type': 'text', 'value': 'dws-host:${HOST_ID}'}],
        }},
        {'type': 'close'},
    ]
}))
")" > /dev/null
echo "Removed: dws-host:${HOST_ID}"

# --- Step 6 reverse: Stop and remove worker container ---
echo "--- Stopping worker container ---"
docker stop fn0-worker 2>/dev/null && echo "Stopped" || echo "Not running"
docker rm fn0-worker 2>/dev/null && echo "Removed" || echo "Not found"

# --- Step 5 reverse: Stop and uninstall cloudflared ---
echo "--- Stopping cloudflared ---"
sudo systemctl stop cloudflared 2>/dev/null && echo "Service stopped" || echo "Service not running"
sudo systemctl disable cloudflared 2>/dev/null || true
sudo cloudflared service uninstall 2>/dev/null && echo "Service uninstalled" || echo "Service not installed"

# --- Step 4 reverse: Delete DNS CNAME records ---
echo "--- Deleting DNS records ---"

for HOSTNAME in "$HTTP_HOSTNAME" "$WS_HOSTNAME"; do
  RECORD_ID=$(curl -sf "https://api.cloudflare.com/client/v4/zones/${CF_ZONE_ID}/dns_records?type=CNAME&name=${HOSTNAME}" \
    -H "Authorization: Bearer ${CF_API_TOKEN}" \
    | python3 -c "import sys,json; r=json.load(sys.stdin)['result']; print(r[0]['id'] if r else '')")

  if [ -n "$RECORD_ID" ]; then
    curl -sf "https://api.cloudflare.com/client/v4/zones/${CF_ZONE_ID}/dns_records/${RECORD_ID}" \
      -X DELETE \
      -H "Authorization: Bearer ${CF_API_TOKEN}" > /dev/null
    echo "  Deleted: $HOSTNAME"
  else
    echo "  Not found: $HOSTNAME"
  fi
done

# --- Step 2/3 reverse: Delete Cloudflare Tunnel ---
echo "--- Deleting Cloudflare Tunnel ---"

TUNNEL_ID=$(curl -sf "https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT_ID}/tunnels?name=${TUNNEL_NAME}&is_deleted=false" \
  -H "Authorization: Bearer ${CF_API_TOKEN}" \
  | python3 -c "
import sys,json
tunnels = json.load(sys.stdin)['result']
active = [t for t in tunnels if t.get('deleted_at') is None]
print(active[0]['id'] if active else '')
")

if [ -n "$TUNNEL_ID" ]; then
  curl -sf "https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT_ID}/tunnels/${TUNNEL_ID}" \
    -X DELETE \
    -H "Authorization: Bearer ${CF_API_TOKEN}" \
    -H "Content-Type: application/json" \
    -d '{}' > /dev/null
  echo "Deleted: $TUNNEL_NAME ($TUNNEL_ID)"
else
  echo "Not found: $TUNNEL_NAME"
fi

echo ""
echo "=== Teardown complete ==="
