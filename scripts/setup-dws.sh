#!/usr/bin/env bash
set -euo pipefail

DWS_USER="fn0"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$SCRIPT_DIR/../infra/cloud"
PULUMI_STACK="${PULUMI_STACK:-prod}"
QUIC_PORT="${QUIC_PORT:-10000}"
HOST_ID="${HOST_ID:-$(hostname)}"
ADDR="${ADDR:-$(hostname)}"

echo "=== fn0 Dedicated Worker Server Setup ==="
echo "Host ID:      $HOST_ID"
echo "Address:      $ADDR"
echo "QUIC Port:    $QUIC_PORT"
echo "Pulumi Stack: $PULUMI_STACK"
echo ""

if ! command -v pulumi &>/dev/null; then
  echo "ERROR: pulumi CLI is not installed."
  echo "Install: https://www.pulumi.com/docs/install/"
  exit 1
fi

if ! command -v curl &>/dev/null; then
  echo "ERROR: curl is not installed."
  exit 1
fi

if ! command -v docker &>/dev/null; then
  echo "WARNING: docker is not installed. The worker runs as a Docker container."
  echo "Install Docker before HQ attempts to deploy the worker."
fi

echo "--- Step 1: Create user '$DWS_USER' ---"

if id "$DWS_USER" &>/dev/null; then
  echo "User '$DWS_USER' already exists."
else
  sudo useradd -m -s /bin/bash "$DWS_USER"
  echo "User '$DWS_USER' created."
fi

if getent group docker &>/dev/null; then
  sudo usermod -aG docker "$DWS_USER"
  echo "User '$DWS_USER' added to docker group."
else
  echo "WARNING: docker group does not exist. Create it after installing Docker:"
  echo "  sudo groupadd docker && sudo usermod -aG docker $DWS_USER"
fi

DWS_HOME=$(eval echo "~$DWS_USER")

echo ""
echo "--- Step 2: Register SSH public key ---"

SSH_PUBLIC_KEY=$(pulumi stack output dwsSshPublicKey -s "$PULUMI_STACK" -C "$INFRA_DIR" 2>/dev/null) || {
  echo "ERROR: Failed to get dwsSshPublicKey from Pulumi stack output."
  echo "Make sure you have run 'pulumi up' after adding the DWS SSH key."
  exit 1
}

SSH_DIR="$DWS_HOME/.ssh"
AUTH_KEYS="$SSH_DIR/authorized_keys"

sudo mkdir -p "$SSH_DIR"
sudo chmod 700 "$SSH_DIR"
sudo touch "$AUTH_KEYS"
sudo chmod 600 "$AUTH_KEYS"

if sudo grep -qF "$SSH_PUBLIC_KEY" "$AUTH_KEYS" 2>/dev/null; then
  echo "SSH public key is already registered."
else
  echo "$SSH_PUBLIC_KEY" | sudo tee -a "$AUTH_KEYS" > /dev/null
  echo "SSH public key registered in $AUTH_KEYS"
fi

sudo chown -R "$DWS_USER:$DWS_USER" "$SSH_DIR"

echo ""
echo "--- Step 3: Register host in doc-db ---"

TURSO_API_TOKEN=$(pulumi config get turso:apiToken -s "$PULUMI_STACK" -C "$INFRA_DIR" 2>/dev/null) || {
  echo "ERROR: Failed to get turso:apiToken from Pulumi config."
  exit 1
}

TURSO_ORG=$(pulumi config get fn0Cloud:tursoOrganizationSlug -s "$PULUMI_STACK" -C "$INFRA_DIR" 2>/dev/null) || {
  echo "ERROR: Failed to get tursoOrganizationSlug from Pulumi config."
  exit 1
}

DB_NAME=$(curl -sf "https://api.turso.tech/v1/organizations/${TURSO_ORG}/databases" \
  -H "Authorization: Bearer ${TURSO_API_TOKEN}" \
  | python3 -c "import sys,json; dbs=json.load(sys.stdin)['databases']; print(next(d['Name'] for d in dbs if d['Name'].startswith('fn0-doc-db')))") || {
  echo "ERROR: Failed to find doc-db database via Turso API."
  exit 1
}

DB_TOKEN=$(curl -sf "https://api.turso.tech/v1/organizations/${TURSO_ORG}/databases/${DB_NAME}/auth/tokens" \
  -X POST \
  -H "Authorization: Bearer ${TURSO_API_TOKEN}" \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['jwt'])") || {
  echo "ERROR: Failed to get database auth token."
  exit 1
}

HTTP_URL="https://${DB_NAME}-${TURSO_ORG}.aws-ap-northeast-1.turso.io"

VALUE=$(printf '{"addr":"%s","port":%d}' "$ADDR" "$QUIC_PORT")
PK="dws-host:$HOST_ID"

RESPONSE=$(curl -sf -X POST "${HTTP_URL}/v2/pipeline" \
  -H "Authorization: Bearer ${DB_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "$(python3 -c "
import json, sys
pk = sys.argv[1]
value = sys.argv[2]
payload = {
    'requests': [
        {'type': 'execute', 'stmt': {
            'sql': 'INSERT OR REPLACE INTO docs (pk, sk, value) VALUES (?, 0, ?)',
            'args': [
                {'type': 'text', 'value': pk},
                {'type': 'text', 'value': value},
            ],
        }},
        {'type': 'close'},
    ]
}
print(json.dumps(payload))
" "$PK" "$VALUE")"
) || {
  echo "ERROR: Failed to register host in doc-db."
  echo "Check your network and Turso credentials."
  exit 1
}

echo "Host registered: pk=$PK value=$VALUE"

echo ""
echo "=== Setup complete ==="
echo ""
echo "Make sure:"
echo "  1. Port $QUIC_PORT (UDP) is open for QUIC connections from HQ"
echo "  2. Port 22 (TCP) is open for SSH connections from HQ"
echo "  3. Docker is installed and running"
