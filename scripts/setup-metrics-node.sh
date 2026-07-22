#!/usr/bin/env bash
# Stand up the self-hosted VictoriaMetrics node behind a Cloudflare Tunnel,
# with automatic incremental backups to R2 (#59). Run directly on the target
# x86_64 Debian machine as root. Idempotent; safe to re-run.
#
# Prefer scripts/setup-metrics-node-remote.sh, which pulls every input below
# from the pulumi stack and runs this script over ssh.
#
# Flow:
#   1. Install the pinned VictoriaMetrics single-node release (checksum
#      verified). It listens on localhost only, with every HTTP endpoint
#      behind basic auth (-httpAuth.* protects all endpoints since v1.86.0).
#   2. Install vmbackup/vmrestore from the matching vmutils release and a
#      10-minute systemd timer that snapshots the storage and incrementally
#      uploads it to the R2 backup bucket. On total disk loss the loss window
#      is one backup interval; outages that keep the disk are covered by the
#      workers' Alloy queue replay instead (#58).
#   3. Install cloudflared from the Cloudflare apt repo.
#   4. Create (or reuse) a remotely-managed tunnel named after --hostname,
#      point its ingress at the local VictoriaMetrics, and upsert the proxied
#      CNAME. The node is reachable only through Cloudflare, which terminates
#      TLS and provides DDoS protection; the machine needs no open inbound
#      ports.
#   5. Verify end-to-end: local health, unauthenticated rejection, a real
#      write -> query round trip through the public hostname, and one full
#      backup run.
#   6. Print the pulumi config the fn0Cloud stack needs.
#
# Usage:
#   sudo CLOUDFLARE_API_TOKEN=... \
#     FN0_METRICS_R2_ACCESS_KEY_ID=... \
#     FN0_METRICS_R2_SECRET_ACCESS_KEY=... \
#     ./setup-metrics-node.sh \
#     --hostname metrics.fn0.dev \
#     --account-id <cloudflare account id> \
#     --zone-id <cloudflare zone id> \
#     --r2-bucket <backup bucket name> \
#     [--retention 30d]
#
# The Cloudflare API token needs "Cloudflare One Connectors Write" (account
# scope) and "DNS Write" (zone scope). It is used only during setup; nothing
# on this machine stores it. The R2 credentials come from the fn0Cloud stack
# outputs metricsBackupR2AccessKeyId / metricsBackupR2SecretAccessKey and are
# stored in a root-only env file for the backup timer.
#
# Restore on a replacement machine: run this script first (it starts an empty
# node), then
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
VM_BASIC_AUTH_USERNAME="fn0"
VM_LISTEN_ADDR="127.0.0.1:8428"
CF_API="https://api.cloudflare.com/client/v4"

retention="30d"
public_hostname=""
account_id=""
zone_id=""
r2_bucket=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --hostname) public_hostname="$2"; shift 2 ;;
    --account-id) account_id="$2"; shift 2 ;;
    --zone-id) zone_id="$2"; shift 2 ;;
    --r2-bucket) r2_bucket="$2"; shift 2 ;;
    --retention) retention="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$public_hostname" || -z "$account_id" || -z "$zone_id" || -z "$r2_bucket" ]]; then
  echo "usage: sudo CLOUDFLARE_API_TOKEN=... FN0_METRICS_R2_ACCESS_KEY_ID=... FN0_METRICS_R2_SECRET_ACCESS_KEY=... $0 --hostname <fqdn> --account-id <id> --zone-id <id> --r2-bucket <bucket> [--retention 30d]" >&2
  exit 1
fi
: "${CLOUDFLARE_API_TOKEN:?CLOUDFLARE_API_TOKEN is required}"
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

echo "== 1/6 VictoriaMetrics ${VM_VERSION} =="

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

if [[ ! -s "$VM_PASSWORD_FILE" ]]; then
  openssl rand -base64 24 | tr -d '\n' > "$VM_PASSWORD_FILE"
fi
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
  -httpAuth.username=${VM_BASIC_AUTH_USERNAME} \\
  -httpAuth.password=file://${VM_PASSWORD_FILE}
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF_VM_UNIT

systemctl daemon-reload
systemctl enable victoria-metrics.service
systemctl restart victoria-metrics.service

echo "== 2/6 vmbackup + timer =="

if [[ -x /usr/local/bin/vmbackup-prod ]] \
  && /usr/local/bin/vmbackup-prod --version 2>&1 | grep -qF "${VM_VERSION}"; then
  echo "vmutils binaries already installed"
else
  vm_release_install "vmutils-linux-amd64-${VM_VERSION}" vmbackup-prod vmrestore-prod
fi

cat > "$VM_BACKUP_ENV_FILE" <<EOF_BACKUP_ENV
AWS_ACCESS_KEY_ID=${FN0_METRICS_R2_ACCESS_KEY_ID}
AWS_SECRET_ACCESS_KEY=${FN0_METRICS_R2_SECRET_ACCESS_KEY}
FN0_METRICS_BACKUP_S3_ENDPOINT=https://${account_id}.r2.cloudflarestorage.com
FN0_METRICS_BACKUP_DST=s3://${r2_bucket}/${public_hostname}/latest
EOF_BACKUP_ENV
chown "$VM_USER:$VM_USER" "$VM_BACKUP_ENV_FILE"
chmod 0600 "$VM_BACKUP_ENV_FILE"

cat > /usr/local/bin/fn0-metrics-backup <<'EOF_BACKUP_SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

vm_addr="127.0.0.1:8428"
auth="fn0:$(cat /etc/victoria-metrics/basic-auth-password)"

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

echo "== 3/6 cloudflared =="

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

echo "== 4/6 tunnel + DNS =="

tunnel_name="fn0-metrics-${public_hostname}"
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
  "$(jq -n --arg host "$public_hostname" --arg service "http://${VM_LISTEN_ADDR}" \
    '{config: {ingress: [{hostname: $host, service: $service}, {service: "http_status:404"}]}}')" \
  >/dev/null

dns_record_body="$(jq -n --arg name "$public_hostname" --arg content "${tunnel_id}.cfargotunnel.com" \
  '{type: "CNAME", proxied: true, name: $name, content: $content}')"
dns_record_id="$(cf_api GET "/zones/${zone_id}/dns_records?type=CNAME&name=${public_hostname}" \
  | jq -r '.result[0].id // empty')"
if [[ -z "$dns_record_id" ]]; then
  cf_api POST "/zones/${zone_id}/dns_records" "$dns_record_body" >/dev/null
  echo "created CNAME ${public_hostname} -> ${tunnel_id}.cfargotunnel.com"
else
  cf_api PUT "/zones/${zone_id}/dns_records/${dns_record_id}" "$dns_record_body" >/dev/null
  echo "updated CNAME ${public_hostname} -> ${tunnel_id}.cfargotunnel.com"
fi

mkdir -p /etc/cloudflared
tunnel_env_file="/etc/cloudflared/fn0-metrics-tunnel.env"
printf 'TUNNEL_TOKEN=%s\n' "$tunnel_token" > "$tunnel_env_file"
chmod 0600 "$tunnel_env_file"

cat > /etc/systemd/system/fn0-metrics-tunnel.service <<EOF_TUNNEL_UNIT
[Unit]
Description=fn0 metrics cloudflare tunnel
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

systemctl daemon-reload
systemctl enable fn0-metrics-tunnel.service
systemctl restart fn0-metrics-tunnel.service

echo "== 5/6 verification =="

vm_auth="${VM_BASIC_AUTH_USERNAME}:$(cat "$VM_PASSWORD_FILE")"

for _ in $(seq 1 12); do
  if curl -fsS -u "$vm_auth" "http://${VM_LISTEN_ADDR}/health" >/dev/null 2>&1; then
    break
  fi
  sleep 5
done
curl -fsS -u "$vm_auth" "http://${VM_LISTEN_ADDR}/health" >/dev/null
echo "local health: ok"

unauth_code="$(curl -s -o /dev/null -w '%{http_code}' "http://${VM_LISTEN_ADDR}/health")"
if [[ "$unauth_code" != "401" ]]; then
  echo "expected 401 for unauthenticated request, got ${unauth_code}" >&2
  exit 1
fi
echo "unauthenticated rejection: ok"

write_ok=""
for _ in $(seq 1 24); do
  if curl -fsS -u "$vm_auth" -X POST \
    "https://${public_hostname}/api/v1/import/prometheus" \
    --data-binary "fn0_setup_verify{node=\"${public_hostname}\"} 1" >/dev/null 2>&1; then
    write_ok=1
    break
  fi
  sleep 5
done
if [[ -z "$write_ok" ]]; then
  echo "public write via https://${public_hostname} did not succeed within 2 minutes" >&2
  exit 1
fi
echo "public write: ok"

query_ok=""
for _ in $(seq 1 12); do
  found="$(curl -fsS -u "$vm_auth" \
    "https://${public_hostname}/api/v1/query?query=fn0_setup_verify" \
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
echo "public write -> query round trip: ok"

public_unauth_code="$(curl -s -o /dev/null -w '%{http_code}' "https://${public_hostname}/health")"
if [[ "$public_unauth_code" != "401" ]]; then
  echo "expected 401 for unauthenticated public request, got ${public_unauth_code}" >&2
  exit 1
fi
echo "public unauthenticated rejection: ok"

systemctl start fn0-metrics-backup.service
echo "backup to R2: ok"

echo "== 6/6 summary =="

cat <<EOF_SUMMARY

== done ==
remote_write url : https://${public_hostname}/api/v1/write
OTLP http base   : https://${public_hostname}/opentelemetry
query url        : https://${public_hostname}
basic auth user  : ${VM_BASIC_AUTH_USERNAME}
password file    : ${VM_PASSWORD_FILE} (on this machine, root-readable)
backup           : ${r2_bucket}/${public_hostname}/latest, every 10 minutes

pulumi config (run in infra/cloud on your workstation; fetch the password
from ${VM_PASSWORD_FILE} yourself — it is intentionally not printed here):
  pulumi config set fn0Cloud:metricsWriteUrl https://${public_hostname}/api/v1/write
  pulumi config set fn0Cloud:metricsQueryUrl https://${public_hostname}
  pulumi config set --secret fn0Cloud:metricsBasicAuthPassword <password>
EOF_SUMMARY
