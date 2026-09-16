#!/usr/bin/env bash

set -euo pipefail

if [[ "$(id -u)" -ne 0 ]]; then
  echo "run as root" >&2
  exit 1
fi

required_names=(
  SIGNY_IMAGE_REF
  SIGNY_R2_BUCKET
  SIGNY_R2_PREFIX
  SIGNY_R2_ENDPOINT
  SIGNY_R2_ACCESS_KEY_ID
  SIGNY_R2_SECRET_ACCESS_KEY
  CLOUDFLARE_API_TOKEN
  CLOUDFLARE_ACCOUNT_ID
  SIGNY_TUNNEL_ID
  SIGNY_HOSTNAME
)
for required_name in "${required_names[@]}"; do
  if [[ -z "${!required_name:-}" ]]; then
    echo "missing ${required_name}" >&2
    exit 1
  fi
done

if ! command -v docker >/dev/null; then
  apt-get update -qq
  apt-get install -y -qq docker.io
fi
systemctl enable --now docker.service

if ! command -v cloudflared >/dev/null; then
  install -d -m 0755 /usr/share/keyrings
  curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg -o /usr/share/keyrings/cloudflare-main.gpg
  printf '%s\n' 'deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] https://pkg.cloudflare.com/cloudflared any main' > /etc/apt/sources.list.d/cloudflared.list
  apt-get update -qq
  apt-get install -y -qq cloudflared
fi

install -d -o 10001 -g 10001 -m 0750 /var/lib/signy
install -d -o root -g root -m 0700 /etc/signy
install -d -o root -g root -m 0700 /etc/cloudflared

signy_env_file="/etc/signy/signy.env"
umask 0077
printf '%s\n' \
  "SIGNY_OBJECT_STORE_URL=s3://${SIGNY_R2_BUCKET}/${SIGNY_R2_PREFIX}" \
  "OBJECT_STORE_ENDPOINT=${SIGNY_R2_ENDPOINT}" \
  'OBJECT_STORE_REGION=auto' \
  'OBJECT_STORE_CONDITIONAL_PUT=etag' \
  'SIGNY_OBJECT_STORE_CATALOG_LOCKED=true' \
  "AWS_ACCESS_KEY_ID=${SIGNY_R2_ACCESS_KEY_ID}" \
  "AWS_SECRET_ACCESS_KEY=${SIGNY_R2_SECRET_ACCESS_KEY}" \
  'SIGNY_LISTEN_ADDR=0.0.0.0:3100' \
  'SIGNY_LOG_FORMAT=json' \
  'RUST_LOG=signy=warn' \
  'SIGNY_MEMORY_BUDGET=2147483648' \
  'SIGNY_MEMORY_ACCOUNT_BYTES=1073741824' \
  'SIGNY_CACHE_MAX_BYTES=8589934592' \
  'SIGNY_MAX_WAL_BACKLOG_BYTES=1073741824' \
  'SIGNY_FLUSH_MAX_INTERVAL=60s' \
  'SIGNY_ORPHAN_GC_INTERVAL=1h' \
  'SIGNY_CATALOG_PRUNE_MIN_AGE=8d' \
  'SIGNY_MIN_FREE_DISK_BYTES=4294967296' \
  'SIGNY_RETENTION_INTERVAL=300s' \
  'SIGNY_RETENTION_GRACE_PERIOD=3600s' \
  'SIGNY_STARTUP_RETRY_BUDGET=300s' > "$signy_env_file"
chmod 0600 "$signy_env_file"

docker pull "$SIGNY_IMAGE_REF"
docker rm -f signy >/dev/null 2>&1 || true
docker run -d --name signy --restart unless-stopped --stop-timeout=-1 \
  --log-driver json-file --log-opt max-size=100m --log-opt max-file=5 \
  --memory=4g --memory-reservation=2g \
  --env-file "$signy_env_file" \
  -v /var/lib/signy:/var/lib/signy \
  -p 127.0.0.1:3100:3100 \
  "$SIGNY_IMAGE_REF" >/dev/null

cloudflare_api() {
  local method="$1"
  local path="$2"
  curl -fsS -X "$method" \
    -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}" \
    -H 'Content-Type: application/json' \
    "https://api.cloudflare.com/client/v4${path}"
}

tunnel_token="$(cloudflare_api GET "/accounts/${CLOUDFLARE_ACCOUNT_ID}/cfd_tunnel/${SIGNY_TUNNEL_ID}/token" | jq -er '.result')"
tunnel_env_file="/etc/cloudflared/fn0-signy-tunnel.env"
printf 'TUNNEL_TOKEN=%s\n' "$tunnel_token" > "$tunnel_env_file"
chmod 0600 "$tunnel_env_file"

for old_unit in fn0-metrics-tunnel.service fn0-telemetry-tunnel.service fn0-metrics-backup.timer fn0-metrics-backup.service; do
  systemctl disable --now "$old_unit" >/dev/null 2>&1 || true
done
rm -f /etc/systemd/system/fn0-metrics-tunnel.service /etc/systemd/system/fn0-telemetry-tunnel.service
rm -f /etc/systemd/system/fn0-metrics-backup.timer /etc/systemd/system/fn0-metrics-backup.service
rm -f /etc/cloudflared/fn0-metrics-tunnel.env /etc/cloudflared/fn0-telemetry-tunnel.env

cloudflared_path="$(command -v cloudflared)"
cat > /etc/systemd/system/fn0-signy-tunnel.service <<EOF_TUNNEL
[Unit]
Description=fn0 Signy Cloudflare Tunnel
After=network-online.target
Wants=network-online.target

[Service]
Type=notify
TimeoutStartSec=0
EnvironmentFile=${tunnel_env_file}
ExecStart=${cloudflared_path} --no-autoupdate tunnel run
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF_TUNNEL
systemctl daemon-reload
systemctl enable --now fn0-signy-tunnel.service

for attempt in $(seq 1 60); do
  if curl -fsS http://127.0.0.1:3100/ready >/dev/null 2>&1; then
    break
  fi
  if [[ "$attempt" -eq 60 ]]; then
    docker logs --tail=50 signy >&2 || true
    exit 1
  fi
  sleep 2
done

ready_body="$(curl -fsS http://127.0.0.1:3100/ready)"
remote_healthy="$(curl -fsS http://127.0.0.1:3100/metrics | awk '$1 == "signy_remote_healthy" {print $2; exit}')"
if [[ "$remote_healthy" != 1 ]]; then
  echo "R2 health verification failed" >&2
  exit 1
fi

echo "signy image: ${SIGNY_IMAGE_REF}"
echo "signy ready: ${ready_body}"
echo "signy tenant policy: unchanged by installer; control owns project policies"
echo "signy R2 remote healthy: ${remote_healthy}"
echo "signy tunnel unit: active"
