#!/usr/bin/env bash
# Stand up the self-hosted metrics node behind a Cloudflare Tunnel:
# VictoriaMetrics (#59). Run directly on the target x86_64 Debian machine as
# root. Idempotent; safe to re-run.
#
# Prefer scripts/setup-telemetry-node-remote.sh, which pulls every input below
# from the pulumi stack and runs this script over ssh.
#
# One hostname, one tunnel, one local listener:
#
#   <metrics hostname>    -> 127.0.0.1:8428  VictoriaMetrics
#
# VictoriaMetrics authenticates itself (-httpAuth.* covers every endpoint since
# v1.86.0), so the tunnel points straight at it and no reverse proxy is
# installed. It owns its data, so it takes incremental vmbackup snapshots to R2
# every 10 minutes.
#
# Usage:
#   sudo CLOUDFLARE_API_TOKEN=... \
#     FN0_METRICS_PASSWORD=... \
#     FN0_METRICS_R2_ACCESS_KEY_ID=... \
#     FN0_METRICS_R2_SECRET_ACCESS_KEY=... \
#     ./setup-telemetry-node.sh \
#     --metrics-hostname metrics.fn0.dev \
#     --username fn0 \
#     --account-id <cloudflare account id> \
#     --zone-id <cloudflare zone id> \
#     --metrics-backup-bucket <bucket> \
#     [--retention 30d]
#
# Every input comes from the fn0Cloud stack, which owns them: the node stores no
# value it generated itself, so re-running converges it onto the stack and
# rebuilding the machine keeps the same credentials. The Cloudflare API token
# needs "Cloudflare One Connectors Write" (account scope) and "DNS Write" (zone
# scope); it is used only during setup and is not stored here.
#
# Restore VictoriaMetrics on a replacement machine: run this script first (it
# starts an empty node), then
#   systemctl stop victoria-metrics
#   sudo -u victoria-metrics bash -c 'set -a; . /etc/victoria-metrics/backup.env; \
#     vmrestore-prod -customS3Endpoint="$FN0_METRICS_BACKUP_S3_ENDPOINT" \
#     -src="$FN0_METRICS_BACKUP_DST" -storageDataPath=/var/lib/victoria-metrics'
#   systemctl start victoria-metrics
#
# Required tools: curl, jq, tar, sha256sum, openssl, apt-get, systemd.

set -euo pipefail

VM_VERSION="v1.148.0"
VM_USER="victoria-metrics"
VM_DATA_DIR="/var/lib/victoria-metrics"
VM_CONFIG_DIR="/etc/victoria-metrics"
VM_PASSWORD_FILE="${VM_CONFIG_DIR}/basic-auth-password"
VM_BACKUP_ENV_FILE="${VM_CONFIG_DIR}/backup.env"
VM_LISTEN_ADDR="127.0.0.1:8428"

CF_API="https://api.cloudflare.com/client/v4"

retention="30d"
metrics_hostname=""
basic_auth_username=""
account_id=""
zone_id=""
metrics_backup_bucket=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --metrics-hostname) metrics_hostname="$2"; shift 2 ;;
    --username) basic_auth_username="$2"; shift 2 ;;
    --account-id) account_id="$2"; shift 2 ;;
    --zone-id) zone_id="$2"; shift 2 ;;
    --metrics-backup-bucket) metrics_backup_bucket="$2"; shift 2 ;;
    --retention) retention="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

for required in metrics_hostname basic_auth_username \
  account_id zone_id metrics_backup_bucket; do
  if [[ -z "${!required}" ]]; then
    echo "missing required argument for ${required//_/-}; see the usage comment at the top of this script" >&2
    exit 1
  fi
done
: "${CLOUDFLARE_API_TOKEN:?CLOUDFLARE_API_TOKEN is required}"
: "${FN0_METRICS_PASSWORD:?FN0_METRICS_PASSWORD is required}"
: "${FN0_METRICS_R2_ACCESS_KEY_ID:?FN0_METRICS_R2_ACCESS_KEY_ID is required}"
: "${FN0_METRICS_R2_SECRET_ACCESS_KEY:?FN0_METRICS_R2_SECRET_ACCESS_KEY is required}"
if [[ "$(id -u)" -ne 0 ]]; then
  echo "run as root" >&2
  exit 1
fi
if [[ "$(uname -m)" != "x86_64" ]]; then
  echo "this script targets x86_64 (got $(uname -m))" >&2
  exit 1
fi
for tool in curl jq tar sha256sum openssl systemctl apt-get; do
  command -v "$tool" >/dev/null || { echo "missing required tool: $tool" >&2; exit 1; }
done

cf_api() {
  local method="$1" path="$2" body="${3:-}"
  local args=(-sS -X "$method"
    -H "Authorization: Bearer ${CLOUDFLARE_API_TOKEN}"
    -H "Content-Type: application/json")
  if [[ -n "$body" ]]; then
    args+=(--data "$body")
  fi
  local response
  response="$(curl "${args[@]}" "${CF_API}${path}")"
  if ! jq -e '.success' <<<"$response" >/dev/null; then
    echo "cloudflare API ${method} ${path} failed: ${response}" >&2
    return 1
  fi
  printf '%s' "$response"
}

vm_release_install() {
  local tarball="$1"
  shift
  local download_dir
  download_dir="$(mktemp -d)"
  local base_url="https://github.com/VictoriaMetrics/VictoriaMetrics/releases/download/${VM_VERSION}"
  curl -fsSL -o "${download_dir}/${tarball}.tar.gz" "${base_url}/${tarball}.tar.gz"
  curl -fsSL -o "${download_dir}/checksums.txt" "${base_url}/${tarball}_checksums.txt"
  (cd "$download_dir" && sha256sum -c --ignore-missing checksums.txt)
  tar -xzf "${download_dir}/${tarball}.tar.gz" -C "$download_dir"
  local binary
  for binary in "$@"; do
    install -m 0755 "${download_dir}/${binary}" "/usr/local/bin/${binary}"
  done
  rm -rf "$download_dir"
}

echo "== 1/5 VictoriaMetrics ${VM_VERSION} =="

if [[ -x /usr/local/bin/victoria-metrics-prod ]] \
  && /usr/local/bin/victoria-metrics-prod --version 2>&1 | grep -qF "${VM_VERSION}"; then
  echo "victoria-metrics binary already installed"
else
  vm_release_install "victoria-metrics-linux-amd64-${VM_VERSION}" victoria-metrics-prod
fi

if ! id -u "$VM_USER" >/dev/null 2>&1; then
  useradd --system --home-dir "$VM_DATA_DIR" --shell /usr/sbin/nologin "$VM_USER"
fi
mkdir -p "$VM_DATA_DIR" "$VM_CONFIG_DIR"

printf '%s' "$FN0_METRICS_PASSWORD" > "$VM_PASSWORD_FILE"
chown -R "$VM_USER:$VM_USER" "$VM_DATA_DIR" "$VM_CONFIG_DIR"
chmod 0600 "$VM_PASSWORD_FILE"

cat > /etc/systemd/system/victoria-metrics.service <<EOF_VM_UNIT
[Unit]
Description=fn0 VictoriaMetrics metrics backend
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${VM_USER}
Group=${VM_USER}
ExecStart=/usr/local/bin/victoria-metrics-prod \\
  -httpListenAddr=${VM_LISTEN_ADDR} \\
  -storageDataPath=${VM_DATA_DIR} \\
  -retentionPeriod=${retention} \\
  -httpAuth.username=${basic_auth_username} \\
  -httpAuth.password=file://${VM_PASSWORD_FILE}
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF_VM_UNIT

systemctl daemon-reload
systemctl enable victoria-metrics.service
systemctl restart victoria-metrics.service

echo "== 2/5 vmbackup + timer =="

if [[ -x /usr/local/bin/vmbackup-prod ]] \
  && /usr/local/bin/vmbackup-prod --version 2>&1 | grep -qF "${VM_VERSION}"; then
  echo "vmutils binaries already installed"
else
  vm_release_install "vmutils-linux-amd64-${VM_VERSION}" vmbackup-prod vmrestore-prod
fi

cat > "$VM_BACKUP_ENV_FILE" <<EOF_BACKUP_ENV
AWS_ACCESS_KEY_ID=${FN0_METRICS_R2_ACCESS_KEY_ID}
AWS_SECRET_ACCESS_KEY=${FN0_METRICS_R2_SECRET_ACCESS_KEY}
FN0_METRICS_BASIC_AUTH_USERNAME=${basic_auth_username}
FN0_METRICS_BACKUP_S3_ENDPOINT=https://${account_id}.r2.cloudflarestorage.com
FN0_METRICS_BACKUP_DST=s3://${metrics_backup_bucket}/${metrics_hostname}/latest
EOF_BACKUP_ENV
chown "$VM_USER:$VM_USER" "$VM_BACKUP_ENV_FILE"
chmod 0600 "$VM_BACKUP_ENV_FILE"

cat > /usr/local/bin/fn0-metrics-backup <<'EOF_BACKUP_SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

vm_addr="127.0.0.1:8428"
auth="${FN0_METRICS_BASIC_AUTH_USERNAME}:$(cat /etc/victoria-metrics/basic-auth-password)"

snapshot="$(curl -fsS -u "$auth" -X POST "http://${vm_addr}/snapshot/create" | jq -r '.snapshot')"
cleanup() {
  curl -fsS -u "$auth" -X POST "http://${vm_addr}/snapshot/delete?snapshot=${snapshot}" >/dev/null
}
trap cleanup EXIT

/usr/local/bin/vmbackup-prod \
  -storageDataPath=/var/lib/victoria-metrics \
  -snapshotName="$snapshot" \
  -customS3Endpoint="$FN0_METRICS_BACKUP_S3_ENDPOINT" \
  -dst="$FN0_METRICS_BACKUP_DST"
EOF_BACKUP_SCRIPT
chmod 0755 /usr/local/bin/fn0-metrics-backup

cat > /etc/systemd/system/fn0-metrics-backup.service <<EOF_BACKUP_UNIT
[Unit]
Description=fn0 metrics backup to R2
After=victoria-metrics.service
Requires=victoria-metrics.service

[Service]
Type=oneshot
User=${VM_USER}
Group=${VM_USER}
EnvironmentFile=${VM_BACKUP_ENV_FILE}
ExecStart=/usr/local/bin/fn0-metrics-backup
EOF_BACKUP_UNIT

cat > /etc/systemd/system/fn0-metrics-backup.timer <<'EOF_BACKUP_TIMER'
[Unit]
Description=fn0 metrics backup every 10 minutes

[Timer]
OnCalendar=*:0/10
RandomizedDelaySec=60
Persistent=true

[Install]
WantedBy=timers.target
EOF_BACKUP_TIMER

systemctl daemon-reload
systemctl enable fn0-metrics-backup.timer
systemctl restart fn0-metrics-backup.timer

echo "== 3/5 cloudflared =="

if ! command -v cloudflared >/dev/null; then
  install -d -m 0755 /usr/share/keyrings
  curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg \
    -o /usr/share/keyrings/cloudflare-main.gpg
  echo "deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] https://pkg.cloudflare.com/cloudflared any main" \
    > /etc/apt/sources.list.d/cloudflared.list
  apt-get update -qq
  apt-get install -y -qq cloudflared
else
  echo "cloudflared already installed"
fi

echo "== 4/5 tunnel + DNS =="

# Named after the metrics hostname because that is what the tunnel was created
# as; renaming it would orphan the existing tunnel and its credentials rather
# than move them.
tunnel_name="fn0-metrics-${metrics_hostname}"
tunnel_id="$(cf_api GET "/accounts/${account_id}/cfd_tunnel?name=${tunnel_name}&is_deleted=false" \
  | jq -r '.result[0].id // empty')"
if [[ -z "$tunnel_id" ]]; then
  tunnel_id="$(cf_api POST "/accounts/${account_id}/cfd_tunnel" \
    "$(jq -n --arg name "$tunnel_name" '{name: $name, config_src: "cloudflare"}')" \
    | jq -r '.result.id')"
  echo "created tunnel ${tunnel_name} (${tunnel_id})"
else
  echo "reusing tunnel ${tunnel_name} (${tunnel_id})"
fi

tunnel_token="$(cf_api GET "/accounts/${account_id}/cfd_tunnel/${tunnel_id}/token" | jq -r '.result')"

cf_api PUT "/accounts/${account_id}/cfd_tunnel/${tunnel_id}/configurations" \
  "$(jq -n \
    --arg metrics_host "$metrics_hostname" \
    --arg metrics_service "http://${VM_LISTEN_ADDR}" \
    '{config: {ingress: [
       {hostname: $metrics_host, service: $metrics_service},
       {service: "http_status:404"}
     ]}}')" \
  >/dev/null

upsert_cname() {
  local hostname="$1"
  local body
  body="$(jq -n --arg name "$hostname" --arg content "${tunnel_id}.cfargotunnel.com" \
    '{type: "CNAME", proxied: true, name: $name, content: $content}')"
  local record_id
  record_id="$(cf_api GET "/zones/${zone_id}/dns_records?type=CNAME&name=${hostname}" \
    | jq -r '.result[0].id // empty')"
  if [[ -z "$record_id" ]]; then
    cf_api POST "/zones/${zone_id}/dns_records" "$body" >/dev/null
    echo "created CNAME ${hostname} -> ${tunnel_id}.cfargotunnel.com"
  else
    cf_api PUT "/zones/${zone_id}/dns_records/${record_id}" "$body" >/dev/null
    echo "updated CNAME ${hostname} -> ${tunnel_id}.cfargotunnel.com"
  fi
}

upsert_cname "$metrics_hostname"

mkdir -p /etc/cloudflared
tunnel_env_file="/etc/cloudflared/fn0-telemetry-tunnel.env"
printf 'TUNNEL_TOKEN=%s\n' "$tunnel_token" > "$tunnel_env_file"
chmod 0600 "$tunnel_env_file"

cat > /etc/systemd/system/fn0-telemetry-tunnel.service <<EOF_TUNNEL_UNIT
[Unit]
Description=fn0 telemetry cloudflare tunnel
After=network-online.target
Wants=network-online.target

[Service]
Type=notify
TimeoutStartSec=0
EnvironmentFile=${tunnel_env_file}
ExecStart=$(command -v cloudflared) --no-autoupdate tunnel run
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF_TUNNEL_UNIT

# The unit was called fn0-metrics-tunnel before this node grew logs and traces.
# Two units running the same tunnel token would both dial out and split traffic
# between them, so the old one has to go before the new one starts.
if systemctl list-unit-files fn0-metrics-tunnel.service >/dev/null 2>&1 \
  && [[ -f /etc/systemd/system/fn0-metrics-tunnel.service ]]; then
  systemctl disable --now fn0-metrics-tunnel.service >/dev/null 2>&1 || true
  rm -f /etc/systemd/system/fn0-metrics-tunnel.service
  rm -f /etc/cloudflared/fn0-metrics-tunnel.env
  echo "replaced fn0-metrics-tunnel.service"
fi

systemctl daemon-reload
systemctl enable fn0-telemetry-tunnel.service
systemctl restart fn0-telemetry-tunnel.service

echo "== 5/5 verification =="

vm_auth="${basic_auth_username}:$(cat "$VM_PASSWORD_FILE")"

for _ in $(seq 1 12); do
  if curl -fsS -u "$vm_auth" "http://${VM_LISTEN_ADDR}/health" >/dev/null 2>&1; then
    break
  fi
  sleep 5
done
curl -fsS -u "$vm_auth" "http://${VM_LISTEN_ADDR}/health" >/dev/null
echo "victoria-metrics local health: ok"

# /health and /ping are liveness probes that carry no data and stay exempt
# from -httpAuth.*, so the auth check has to probe a data endpoint.
unauth_code="$(curl -s -o /dev/null -w '%{http_code}' "http://${VM_LISTEN_ADDR}/api/v1/query?query=up")"
if [[ "$unauth_code" != "401" ]]; then
  echo "expected 401 for unauthenticated query, got ${unauth_code}" >&2
  exit 1
fi
echo "victoria-metrics unauthenticated rejection: ok"

write_ok=""
for _ in $(seq 1 24); do
  if curl -fsS -u "$vm_auth" -X POST \
    "https://${metrics_hostname}/api/v1/import/prometheus" \
    --data-binary "fn0_setup_verify{node=\"${metrics_hostname}\"} 1" >/dev/null 2>&1; then
    write_ok=1
    break
  fi
  sleep 5
done
if [[ -z "$write_ok" ]]; then
  echo "public write via https://${metrics_hostname} did not succeed within 2 minutes" >&2
  exit 1
fi
echo "metrics public write: ok"

query_ok=""
for _ in $(seq 1 12); do
  found="$(curl -fsS -u "$vm_auth" \
    "https://${metrics_hostname}/api/v1/query?query=fn0_setup_verify" \
    2>/dev/null | jq -r '.data.result | length' || echo 0)"
  if [[ "$found" -ge 1 ]]; then
    query_ok=1
    break
  fi
  sleep 5
done
if [[ -z "$query_ok" ]]; then
  echo "public query did not return the verification sample within 1 minute" >&2
  exit 1
fi
echo "metrics public write -> query round trip: ok"

public_unauth_code="$(curl -s -o /dev/null -w '%{http_code}' "https://${metrics_hostname}/api/v1/query?query=up")"
if [[ "$public_unauth_code" != "401" ]]; then
  echo "expected 401 for unauthenticated public metrics query, got ${public_unauth_code}" >&2
  exit 1
fi
echo "metrics public unauthenticated rejection: ok"

systemctl start fn0-metrics-backup.service
echo "metrics backup to R2: ok"

cat <<EOF_SUMMARY

== done ==
metrics remote_write : https://${metrics_hostname}/api/v1/write
metrics OTLP         : https://${metrics_hostname}/opentelemetry
metrics query        : https://${metrics_hostname}
metrics basic auth   : ${basic_auth_username} (password in ${VM_PASSWORD_FILE})
metrics backup       : ${metrics_backup_bucket}/${metrics_hostname}/latest, every 10 minutes

These match the fn0Cloud stack outputs; nothing has to be copied back into pulumi.
EOF_SUMMARY
