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

if [ "$(id -u)" -ne 0 ]; then
  echo "ERROR: This script must be run as root (for user creation)."
  echo "Usage: sudo $0"
  exit 1
fi

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
  useradd -m -s /bin/bash "$DWS_USER"
  echo "User '$DWS_USER' created."
fi

if getent group docker &>/dev/null; then
  usermod -aG docker "$DWS_USER"
  echo "User '$DWS_USER' added to docker group."
else
  echo "WARNING: docker group does not exist. Create it after installing Docker:"
  echo "  groupadd docker && usermod -aG docker $DWS_USER"
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

mkdir -p "$SSH_DIR"
chmod 700 "$SSH_DIR"
touch "$AUTH_KEYS"
chmod 600 "$AUTH_KEYS"

if grep -qF "$SSH_PUBLIC_KEY" "$AUTH_KEYS" 2>/dev/null; then
  echo "SSH public key is already registered."
else
  echo "$SSH_PUBLIC_KEY" >> "$AUTH_KEYS"
  echo "SSH public key registered in $AUTH_KEYS"
fi

chown -R "$DWS_USER:$DWS_USER" "$SSH_DIR"

echo ""
echo "--- Step 3: Register host in doc-db ---"

TURSO_DB_URL=$(pulumi config get fn0Cloud:tursoDbUrl -s "$PULUMI_STACK" -C "$INFRA_DIR" 2>/dev/null) || {
  echo "ERROR: Failed to get tursoDbUrl from Pulumi config."
  exit 1
}

TURSO_DB_TOKEN=$(pulumi config get fn0Cloud:tursoDbToken -s "$PULUMI_STACK" -C "$INFRA_DIR" 2>/dev/null) || {
  echo "ERROR: Failed to get tursoDbToken from Pulumi config."
  exit 1
}

HTTP_URL="${TURSO_DB_URL/libsql:\/\//https://}"

VALUE=$(printf '{"addr":"%s","port":%d}' "$ADDR" "$QUIC_PORT")
PK="dws-host:$HOST_ID"

RESPONSE=$(curl -sf -X POST "${HTTP_URL}/v2/pipeline" \
  -H "Authorization: Bearer ${TURSO_DB_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "$(printf '{
    "requests": [
      {
        "type": "execute",
        "stmt": {
          "sql": "INSERT OR REPLACE INTO docs (pk, sk, value) VALUES (?, 0, ?)",
          "args": [
            {"type": "text", "value": "%s"},
            {"type": "text", "value": "%s"}
          ]
        }
      },
      {"type": "close"}
    ]
  }' "$PK" "$VALUE")"
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
