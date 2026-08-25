#!/usr/bin/env bash
# Stand up the self-hosted telemetry node behind a Cloudflare Tunnel: metrics in
# VictoriaMetrics (#59), logs and traces in loggytracy (#62). Run directly on the
# target x86_64 Debian machine as root.
# Ingest is OTLP only — the engine serves the Loki API for queries, not pushes.
# Idempotent; safe to re-run.
#
# Prefer scripts/setup-telemetry-node-remote.sh, which pulls every input below
# from the pulumi stack and runs this script over ssh.
#
# Two hostnames, one tunnel, two local listeners:
#
#   <metrics hostname>    -> 127.0.0.1:8428  VictoriaMetrics
#   <telemetry hostname>  -> 127.0.0.1:3100  loggytracy (ingest + query)
#
# The two backends are protected in different ways because they are built
# differently. VictoriaMetrics authenticates itself (-httpAuth.* covers every
# endpoint since v1.86.0), so the tunnel points straight at it. loggytracy has
# no TLS and no authentication at all — it reads X-Scope-OrgID and believes it —
# so a Cloudflare Access service token authenticates its callers at the edge and
# a Transform Rule overwrites the tenant header there. Both are created by the
# fn0Cloud stack, not here. That is why nothing on this machine terminates auth
# for loggytracy and no reverse proxy is installed: the edge is the gateway, and
# the listener is on loopback so nothing else can reach it.
#
# Durability is split the same way. VictoriaMetrics owns its data, so it takes
# incremental vmbackup snapshots to R2 every 10 minutes. loggytracy's data lives
# in R2 already — the local disk is a WAL plus a cache — so it has no backup
# cadence to design; losing the machine loses only the unflushed WAL window.
#
# Usage:
#   sudo CLOUDFLARE_API_TOKEN=... \
#     FN0_METRICS_PASSWORD=... \
#     FN0_METRICS_R2_ACCESS_KEY_ID=... \
#     FN0_METRICS_R2_SECRET_ACCESS_KEY=... \
#     FN0_TELEMETRY_R2_ACCESS_KEY_ID=... \
#     FN0_TELEMETRY_R2_SECRET_ACCESS_KEY=... \
#     FN0_TELEMETRY_ACCESS_CLIENT_ID=... \
#     FN0_TELEMETRY_ACCESS_CLIENT_SECRET=... \
#     ./setup-telemetry-node.sh \
#     --metrics-hostname metrics.fn0.dev \
#     --telemetry-hostname telemetry.fn0.dev \
#     --username fn0 \
#     --tenant fn0 \
#     --account-id <cloudflare account id> \
#     --zone-id <cloudflare zone id> \
#     --metrics-backup-bucket <bucket> \
#     --logs-traces-bucket <bucket> \
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
# loggytracy needs no equivalent: it restores its catalog from R2 on startup.
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

# Pinned by digest-bearing tag rather than `latest`: loggytracy's own deployment
# guide is explicit that `latest` is for typing, not for deployments, and a
# telemetry backend that silently changes version is one that cannot be told
# apart from the thing it is supposed to be observing.
LOGGYTRACY_IMAGE="ghcr.io/namse/loggytracy:ac59184e77cfcfa7f0fe1c2d8adee6845239c4f6"
LOGGYTRACY_DATA_DIR="/var/lib/loggytracy"
LOGGYTRACY_CONFIG_DIR="/etc/loggytracy"
LOGGYTRACY_ENV_FILE="${LOGGYTRACY_CONFIG_DIR}/loggytracy.env"
# The image runs as uid 10001 and a bind mount carries the host's numbers
# through untranslated, so the data directory has to be owned by that number.
LOGGYTRACY_UID=10001
LOGGYTRACY_PORT=3100

CF_API="https://api.cloudflare.com/client/v4"

retention="30d"
metrics_hostname=""
telemetry_hostname=""
basic_auth_username=""
tenant=""
account_id=""
zone_id=""
metrics_backup_bucket=""
logs_traces_bucket=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --metrics-hostname) metrics_hostname="$2"; shift 2 ;;
    --telemetry-hostname) telemetry_hostname="$2"; shift 2 ;;
    --username) basic_auth_username="$2"; shift 2 ;;
    --tenant) tenant="$2"; shift 2 ;;
    --account-id) account_id="$2"; shift 2 ;;
    --zone-id) zone_id="$2"; shift 2 ;;
    --metrics-backup-bucket) metrics_backup_bucket="$2"; shift 2 ;;
    --logs-traces-bucket) logs_traces_bucket="$2"; shift 2 ;;
    --retention) retention="$2"; shift 2 ;;
    *) echo "unknown argument: $1" >&2; exit 1 ;;
  esac
done

for required in metrics_hostname telemetry_hostname basic_auth_username \
  tenant account_id zone_id metrics_backup_bucket logs_traces_bucket; do
  if [[ -z "${!required}" ]]; then
    echo "missing required argument for ${required//_/-}; see the usage comment at the top of this script" >&2
    exit 1
  fi
done
: "${CLOUDFLARE_API_TOKEN:?CLOUDFLARE_API_TOKEN is required}"
: "${FN0_METRICS_PASSWORD:?FN0_METRICS_PASSWORD is required}"
: "${FN0_METRICS_R2_ACCESS_KEY_ID:?FN0_METRICS_R2_ACCESS_KEY_ID is required}"
: "${FN0_METRICS_R2_SECRET_ACCESS_KEY:?FN0_METRICS_R2_SECRET_ACCESS_KEY is required}"
: "${FN0_TELEMETRY_R2_ACCESS_KEY_ID:?FN0_TELEMETRY_R2_ACCESS_KEY_ID is required}"
: "${FN0_TELEMETRY_R2_SECRET_ACCESS_KEY:?FN0_TELEMETRY_R2_SECRET_ACCESS_KEY is required}"
: "${FN0_TELEMETRY_ACCESS_CLIENT_ID:?FN0_TELEMETRY_ACCESS_CLIENT_ID is required}"
: "${FN0_TELEMETRY_ACCESS_CLIENT_SECRET:?FN0_TELEMETRY_ACCESS_CLIENT_SECRET is required}"
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

# A container that is already running has to be drained, not killed: loggytracy
# force-flushes acknowledged writes on SIGTERM, and its writer fencing expects
# the old instance to be fully gone before the next one claims the object-store
# prefix. `docker stop` inherits the --stop-timeout=-1 set at creation, so it
# waits rather than cutting the flush off at ten seconds.
container_replace() {
  local name="$1"
  shift
  if docker inspect "$name" >/dev/null 2>&1; then
    docker stop "$name" >/dev/null
    docker rm "$name" >/dev/null
  fi
  docker run -d --name "$name" "$@" >/dev/null
}

wait_for_loggytracy_ready() {
  for _ in $(seq 1 24); do
    if curl -fsS "http://127.0.0.1:${LOGGYTRACY_PORT}/ready" >/dev/null 2>&1; then
      return 0
    fi
    sleep 5
  done
  echo "loggytracy did not become ready within 2 minutes; docker logs loggytracy" >&2
  exit 1
}

echo "== 1/7 VictoriaMetrics ${VM_VERSION} =="

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

echo "== 2/7 vmbackup + timer =="

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

echo "== 3/7 docker =="

if ! command -v docker >/dev/null; then
  install -d -m 0755 /usr/share/keyrings
  curl -fsSL https://download.docker.com/linux/debian/gpg \
    -o /usr/share/keyrings/docker.asc
  chmod a+r /usr/share/keyrings/docker.asc
  echo "deb [arch=amd64 signed-by=/usr/share/keyrings/docker.asc] https://download.docker.com/linux/debian $(. /etc/os-release && echo "$VERSION_CODENAME") stable" \
    > /etc/apt/sources.list.d/docker.list
  apt-get update -qq
  apt-get install -y -qq docker-ce docker-ce-cli containerd.io
else
  echo "docker already installed"
fi

# The json-file driver rotates nothing by default, and a telemetry box that
# fills its own root filesystem with container logs is a poor advertisement.
# Set it on the daemon rather than per container so nothing has to remember.
if [[ ! -f /etc/docker/daemon.json ]]; then
  mkdir -p /etc/docker
  cat > /etc/docker/daemon.json <<'EOF_DOCKER_DAEMON'
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "100m",
    "max-file": "5"
  }
}
EOF_DOCKER_DAEMON
  systemctl restart docker
fi

echo "== 4/7 loggytracy =="

install -d -o "$LOGGYTRACY_UID" -g "$LOGGYTRACY_UID" -m 0750 "$LOGGYTRACY_DATA_DIR"
install -d -o root -g root -m 0700 "$LOGGYTRACY_CONFIG_DIR"

# OBJECT_STORE_CONDITIONAL_PUT=etag is not optional: every catalog commit is a
# compare-and-swap on one manifest object, and without conditional writes two
# commits overwrite each other silently. Startup runs a preflight that verifies
# a write which should be rejected is rejected, and refuses to start otherwise.
cat > "$LOGGYTRACY_ENV_FILE" <<EOF_LOGGYTRACY_ENV
LOGGYTRACY_OBJECT_STORE_URL=s3://${logs_traces_bucket}/loggytracy
OBJECT_STORE_ENDPOINT=https://${account_id}.r2.cloudflarestorage.com
OBJECT_STORE_REGION=auto
OBJECT_STORE_CONDITIONAL_PUT=etag
AWS_ACCESS_KEY_ID=${FN0_TELEMETRY_R2_ACCESS_KEY_ID}
AWS_SECRET_ACCESS_KEY=${FN0_TELEMETRY_R2_SECRET_ACCESS_KEY}
EOF_LOGGYTRACY_ENV
chmod 0600 "$LOGGYTRACY_ENV_FILE"

docker pull "$LOGGYTRACY_IMAGE" >/dev/null

# --stop-timeout=-1: on SIGTERM the engine stops accepting writes and
# force-flushes everything it has acknowledged. Docker's default cuts that off
# with a SIGKILL after ten seconds, which is the difference between a planned
# restart and losing the last few seconds of every tenant's logs. Set at
# creation so a plain `docker stop` inherits it.
#
# Only 3100 is published, and only on loopback. The gRPC listener is not
# exposed because everything reaching this node arrives as OTLP over HTTP
# through the tunnel, and a published port outranks the host firewall.
container_replace loggytracy \
  --restart unless-stopped \
  --stop-timeout=-1 \
  --env-file "$LOGGYTRACY_ENV_FILE" \
  -v "${LOGGYTRACY_DATA_DIR}:/var/lib/loggytracy" \
  -p "127.0.0.1:${LOGGYTRACY_PORT}:3100" \
  "$LOGGYTRACY_IMAGE"

wait_for_loggytracy_ready

# The engine reads no tenant allowlist from its environment: the pushed
# policies are the registry, and a tenant nobody pushed is refused with 403 on
# ingest and query alike. So onboarding belongs to standing the node up rather
# than to a separate manual step, and --retention is what the policy carries.
# The body is the whole policy and not a patch, so every limit left out stays
# unbounded — which is what this node had before it had policies at all.
policy_response="$(curl -s -w $'\n%{http_code}' -X PUT \
  -H 'Content-Type: application/json' \
  --data "$(jq -n --arg retention "$retention" '{retention: $retention}')" \
  "http://127.0.0.1:${LOGGYTRACY_PORT}/loggytracy/api/v1/admin/tenants/${tenant}/retention")"
if [[ "${policy_response##*$'\n'}" != "200" ]]; then
  echo "tenant policy push for ${tenant} returned ${policy_response##*$'\n'}" >&2
  echo "${policy_response%$'\n'*}" >&2
  exit 1
fi
echo "tenant ${tenant} onboarded with retention ${retention}"

echo "== 5/7 cloudflared =="

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

echo "== 6/7 tunnel + DNS =="

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
    --arg telemetry_host "$telemetry_hostname" \
    --arg telemetry_service "http://127.0.0.1:${LOGGYTRACY_PORT}" \
    '{config: {ingress: [
       {hostname: $metrics_host, service: $metrics_service},
       {hostname: $telemetry_host, service: $telemetry_service},
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
upsert_cname "$telemetry_hostname"

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

echo "== 7/7 verification =="

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

wait_for_loggytracy_ready
echo "loggytracy local ready: ok"

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

# This is the check that matters most on this node: loggytracy itself has no
# authentication, so if Access is not in front of the hostname then anyone who
# learns it can write and read every tenant's logs.
telemetry_unauth_code="$(curl -s -o /dev/null -w '%{http_code}' \
  "https://${telemetry_hostname}/loki/api/v1/labels")"
if [[ "$telemetry_unauth_code" != "401" && "$telemetry_unauth_code" != "403" ]]; then
  echo "expected 401/403 for unauthenticated telemetry query, got ${telemetry_unauth_code}" >&2
  echo "the Cloudflare Access application for ${telemetry_hostname} is missing or misconfigured" >&2
  exit 1
fi
echo "telemetry public unauthenticated rejection: ok"

telemetry_auth_ok=""
for _ in $(seq 1 24); do
  if curl -fsS \
    -H "CF-Access-Client-Id: ${FN0_TELEMETRY_ACCESS_CLIENT_ID}" \
    -H "CF-Access-Client-Secret: ${FN0_TELEMETRY_ACCESS_CLIENT_SECRET}" \
    "https://${telemetry_hostname}/loki/api/v1/labels" >/dev/null 2>&1; then
    telemetry_auth_ok=1
    break
  fi
  sleep 5
done
if [[ -z "$telemetry_auth_ok" ]]; then
  echo "authenticated telemetry query did not succeed within 2 minutes" >&2
  exit 1
fi
echo "telemetry service-token query: ok"

systemctl start fn0-metrics-backup.service
echo "metrics backup to R2: ok"

cat <<EOF_SUMMARY

== done ==
metrics remote_write : https://${metrics_hostname}/api/v1/write
metrics OTLP         : https://${metrics_hostname}/opentelemetry
metrics query        : https://${metrics_hostname}
metrics basic auth   : ${basic_auth_username} (password in ${VM_PASSWORD_FILE})
metrics backup       : ${metrics_backup_bucket}/${metrics_hostname}/latest, every 10 minutes

logs/traces ingest   : https://${telemetry_hostname} (OTLP /v1/logs, /v1/traces)
logs/traces query    : https://${telemetry_hostname} (Loki + Tempo APIs)
logs/traces auth     : Cloudflare Access service token; tenant ${tenant} stamped at the edge
logs/traces store    : s3://${logs_traces_bucket}/loggytracy
loggytracy image     : ${LOGGYTRACY_IMAGE}

These match the fn0Cloud stack outputs; nothing has to be copied back into pulumi.
EOF_SUMMARY
