#!/usr/bin/env bash
#
# setup-dws.sh - Set up a Dedicated Worker Server with Cloudflare Tunnel.
#
# Prerequisites:
#   - Pulumi CLI logged in with access to the fn0Cloud stack
#   - Docker installed and running
#   - curl, python3
#
# Usage:
#   ./scripts/setup-dws.sh <HOST_ID>
#
# What it does:
#   1. Fetches worker image and S3 credentials from Pulumi stack output
#   2. Creates a Cloudflare Tunnel and configures ingress rules
#   3. Creates DNS CNAME records for HTTP and WebSocket
#   4. Installs cloudflared and starts the tunnel service
#   5. Pulls and runs the worker container
#   6. Registers the host in DocDB
#
# Environment variables:
#   HOST_ID          (required) Unique identifier for this DWS
#   PULUMI_STACK     Pulumi stack name (default: prod)
#   HTTP_PORT        Worker HTTP port (default: 8080)
#   WS_PORT          Worker WebSocket port (default: 10000)
#
set -euo pipefail

if [ -z "${1:-}" ]; then
  echo "Usage: $0 <HOST_ID>"
  exit 1
fi

HOST_ID="$1"
PULUMI_STACK="${PULUMI_STACK:-prod}"
HTTP_PORT="${HTTP_PORT:-8080}"
WS_PORT="${WS_PORT:-10000}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$SCRIPT_DIR/../infra/cloud"

echo "=== fn0 Dedicated Worker Server Setup ==="
echo "Host ID:       $HOST_ID"
echo "Pulumi Stack:  $PULUMI_STACK"
echo "HTTP Port:     $HTTP_PORT"
echo "WS Port:       $WS_PORT"
echo ""

# --- Step 1: Fetch config from Pulumi ---
echo "--- Step 1: Fetching configuration from Pulumi ---"

WORKER_IMAGE=$(cd "$INFRA_DIR" && pulumi stack output workerImage -s "$PULUMI_STACK")
CWASM_BUCKET=$(cd "$INFRA_DIR" && pulumi stack output cwasmBucket -s "$PULUMI_STACK")
S3_ENDPOINT=$(cd "$INFRA_DIR" && pulumi stack output s3Endpoint -s "$PULUMI_STACK")
S3_REGION=$(cd "$INFRA_DIR" && pulumi stack output s3Region -s "$PULUMI_STACK")
S3_ACCESS_KEY_ID=$(cd "$INFRA_DIR" && pulumi stack output s3AccessKeyId -s "$PULUMI_STACK" --show-secrets)
S3_SECRET_ACCESS_KEY=$(cd "$INFRA_DIR" && pulumi stack output s3SecretAccessKey -s "$PULUMI_STACK" --show-secrets)
WS_SECRET=$(cd "$INFRA_DIR" && pulumi stack output dwsWsSecret -s "$PULUMI_STACK" --show-secrets)

CF_ACCOUNT_ID=$(cd "$INFRA_DIR" && pulumi config get fn0Cloud:cloudflareAccountId -s "$PULUMI_STACK")
CF_ZONE_ID=$(cd "$INFRA_DIR" && pulumi config get fn0Cloud:cloudflareZoneId -s "$PULUMI_STACK")
CF_API_TOKEN=$(cd "$INFRA_DIR" && pulumi config get cloudflare:apiToken -s "$PULUMI_STACK")
DOMAIN=$(cd "$INFRA_DIR" && pulumi config get fn0Cloud:domain -s "$PULUMI_STACK")

TURSO_API_TOKEN=$(cd "$INFRA_DIR" && pulumi config get turso:apiToken -s "$PULUMI_STACK")
TURSO_ORG=$(cd "$INFRA_DIR" && pulumi config get fn0Cloud:tursoOrganizationSlug -s "$PULUMI_STACK")

echo "Worker image:  $WORKER_IMAGE"
echo "Domain:        $DOMAIN"
echo ""

# --- Step 2: Create or reuse Cloudflare Tunnel ---
echo "--- Step 2: Cloudflare Tunnel ---"

TUNNEL_NAME="dws-${HOST_ID}"
HTTP_HOSTNAME="${HOST_ID}-http.${DOMAIN}"
WS_HOSTNAME="${HOST_ID}-ws.${DOMAIN}"

TUNNEL_ID=$(curl -sf "https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT_ID}/tunnels?name=${TUNNEL_NAME}&is_deleted=false" \
  -H "Authorization: Bearer ${CF_API_TOKEN}" \
  | python3 -c "
import sys,json
tunnels = json.load(sys.stdin)['result']
active = [t for t in tunnels if t.get('deleted_at') is None]
print(active[0]['id'] if active else '')
")

if [ -z "$TUNNEL_ID" ]; then
  TUNNEL_RESPONSE=$(curl -sf "https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT_ID}/tunnels" \
    -H "Authorization: Bearer ${CF_API_TOKEN}" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"${TUNNEL_NAME}\",\"tunnel_secret\":\"$(python3 -c 'import secrets,base64; print(base64.b64encode(secrets.token_bytes(32)).decode())')\"}")
  TUNNEL_ID=$(echo "$TUNNEL_RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['id'])")
  echo "Tunnel created: $TUNNEL_ID"
else
  echo "Tunnel exists: $TUNNEL_ID"
fi

TUNNEL_TOKEN=$(curl -sf "https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/token" \
  -H "Authorization: Bearer ${CF_API_TOKEN}" \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['result'])")
echo "Tunnel token obtained"

# --- Step 3: Configure tunnel ingress (PUT is idempotent) ---
echo "--- Step 3: Configuring tunnel ingress ---"

curl -sf "https://api.cloudflare.com/client/v4/accounts/${CF_ACCOUNT_ID}/cfd_tunnel/${TUNNEL_ID}/configurations" \
  -X PUT \
  -H "Authorization: Bearer ${CF_API_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "$(python3 -c "
import json
config = {
    'config': {
        'ingress': [
            {'hostname': '${HTTP_HOSTNAME}', 'service': 'http://localhost:${HTTP_PORT}'},
            {'hostname': '${WS_HOSTNAME}', 'service': 'http://localhost:${WS_PORT}'},
            {'service': 'http_status:404'}
        ]
    }
}
print(json.dumps(config))
")" > /dev/null

echo "Ingress: $HTTP_HOSTNAME -> :$HTTP_PORT, $WS_HOSTNAME -> :$WS_PORT"

# --- Step 4: Create DNS CNAME records (skip if exists) ---
echo "--- Step 4: DNS CNAME records ---"

EXISTING_RECORDS=$(curl -sf "https://api.cloudflare.com/client/v4/zones/${CF_ZONE_ID}/dns_records?type=CNAME" \
  -H "Authorization: Bearer ${CF_API_TOKEN}" \
  | python3 -c "import sys,json; print(' '.join(r['name'] for r in json.load(sys.stdin)['result']))")

for HOSTNAME in "$HTTP_HOSTNAME" "$WS_HOSTNAME"; do
  if echo "$EXISTING_RECORDS" | grep -qw "$HOSTNAME"; then
    echo "  DNS exists: $HOSTNAME"
  else
    curl -sf "https://api.cloudflare.com/client/v4/zones/${CF_ZONE_ID}/dns_records" \
      -H "Authorization: Bearer ${CF_API_TOKEN}" \
      -H "Content-Type: application/json" \
      -d "$(python3 -c "
import json
print(json.dumps({
    'type': 'CNAME',
    'name': '${HOSTNAME}',
    'content': '${TUNNEL_ID}.cfargotunnel.com',
    'proxied': True,
    'ttl': 1
}))
")" > /dev/null
    echo "  DNS created: $HOSTNAME"
  fi
done

# --- Step 5: Install cloudflared ---
echo "--- Step 5: Installing cloudflared ---"

if ! command -v cloudflared &>/dev/null; then
  ARCH=$(dpkg --print-architecture 2>/dev/null || echo "amd64")
  wget -q "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-${ARCH}.deb" -O /tmp/cloudflared.deb
  sudo dpkg -i /tmp/cloudflared.deb
  rm /tmp/cloudflared.deb
  echo "cloudflared installed"
else
  echo "cloudflared already installed"
fi

sudo cloudflared service install "$TUNNEL_TOKEN" 2>/dev/null || true
sudo systemctl enable cloudflared
sudo systemctl restart cloudflared
echo "cloudflared service started"

# --- Step 6: Run worker container ---
echo "--- Step 6: Running worker container ---"

docker pull "$WORKER_IMAGE"
docker stop fn0-worker 2>/dev/null || true
docker rm fn0-worker 2>/dev/null || true
docker run -d \
  --name fn0-worker \
  --network=host \
  --restart=unless-stopped \
  -e "CWASM_BUCKET=${CWASM_BUCKET}" \
  -e "S3_ENDPOINT=${S3_ENDPOINT}" \
  -e "S3_REGION=${S3_REGION}" \
  -e "AWS_ACCESS_KEY_ID=${S3_ACCESS_KEY_ID}" \
  -e "AWS_SECRET_ACCESS_KEY=${S3_SECRET_ACCESS_KEY}" \
  -e "HTTP_PORT=${HTTP_PORT}" \
  -e "WS_PORT=${WS_PORT}" \
  -e "WS_SECRET=${WS_SECRET}" \
  "$WORKER_IMAGE"
echo "Worker container started"

# --- Step 7: Register in DocDB ---
echo "--- Step 7: Registering host in DocDB ---"

DB_NAME=$(curl -sf "https://api.turso.tech/v1/organizations/${TURSO_ORG}/databases" \
  -H "Authorization: Bearer ${TURSO_API_TOKEN}" \
  | python3 -c "import sys,json; print(next(d['Name'] for d in json.load(sys.stdin)['databases'] if d['Name'].startswith('fn0-doc-db')))")

DB_TOKEN=$(curl -sf "https://api.turso.tech/v1/organizations/${TURSO_ORG}/databases/${DB_NAME}/auth/tokens" \
  -X POST \
  -H "Authorization: Bearer ${TURSO_API_TOKEN}" \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['jwt'])")

PK="dws-host:${HOST_ID}"
VALUE=$(python3 -c "
import json
print(json.dumps({
    'addr': '${WS_HOSTNAME}',
    'port': 443,
    'transport': 'websocket',
    'httpHost': '${HTTP_HOSTNAME}',
}))
")

curl -sf "https://${DB_NAME}-${TURSO_ORG}.aws-ap-northeast-1.turso.io/v2/pipeline" \
  -H "Authorization: Bearer ${DB_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "$(python3 -c "
import json, sys
payload = {
    'requests': [
        {'type': 'execute', 'stmt': {
            'sql': 'REPLACE INTO docs (pk, sk, value) VALUES (?, 0, ?)',
            'args': [
                {'type': 'text', 'value': '${PK}'},
                {'type': 'text', 'value': '${VALUE}'},
            ],
        }},
        {'type': 'close'},
    ]
}
print(json.dumps(payload))
")" > /dev/null

echo "Host registered: $PK"

echo ""
echo "=== Setup complete ==="
echo ""
echo "Worker:     $WORKER_IMAGE"
echo "HTTP:       https://$HTTP_HOSTNAME -> localhost:$HTTP_PORT"
echo "WebSocket:  wss://$WS_HOSTNAME -> localhost:$WS_PORT"
echo "Tunnel:     $TUNNEL_NAME ($TUNNEL_ID)"
