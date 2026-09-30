#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT

source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"
source "${REPO_ROOT}/scripts/lib/bastion-port-forward.sh"

need pulumi
need jq
need oci
need ssh
need scp
need curl

temporary_dir="$(mktemp -d)"
chmod 700 "${temporary_dir}"

cleanup() {
  bastion_port_forward_close
  rm -rf "${temporary_dir}"
}
trap cleanup EXIT

load_pulumi_outputs
require_pulumi_output \
  workerBastionId \
  workerSshPrivateKey \
  dodbPrivateIp \
  signyImageRefOutput \
  signyR2BucketName \
  signyR2StoragePrefix \
  signyR2Endpoint \
  signyR2AccessKeyId \
  signyR2SecretAccessKey \
  signyTunnelId \
  signyHostnameOutput \
  signyAccessClientId \
  signyAccessClientSecret \
  cloudflareAccountId \
  cloudflareOperatorApiToken

dodb_private_ip="$(pulumi_pick dodbPrivateIp)"
signy_image_ref="$(pulumi_pick signyImageRefOutput)"
signy_r2_bucket="$(pulumi_pick signyR2BucketName)"
signy_r2_prefix="$(pulumi_pick signyR2StoragePrefix)"
signy_r2_endpoint="$(pulumi_pick signyR2Endpoint)"
signy_r2_access_key_id="$(pulumi_pick signyR2AccessKeyId)"
signy_r2_secret_access_key="$(pulumi_pick signyR2SecretAccessKey)"
signy_tunnel_id="$(pulumi_pick signyTunnelId)"
signy_hostname="$(pulumi_pick signyHostnameOutput)"
signy_access_client_id="$(pulumi_pick signyAccessClientId)"
signy_access_client_secret="$(pulumi_pick signyAccessClientSecret)"
cloudflare_account_id="$(pulumi_pick cloudflareAccountId)"
cloudflare_api_token="$(pulumi_pick cloudflareOperatorApiToken)"

if [[ ! "${signy_image_ref}" =~ @sha256:[0-9a-f]{64}$ ]]; then
  echo "refusing to deploy Signy without a digest-pinned image reference" >&2
  exit 1
fi
if [[ "${signy_r2_prefix}" != "fn0/signy" || "${signy_hostname}" != "signy.fn0.dev" ]]; then
  echo "Pulumi Signy prefix or hostname differs from the approved production target" >&2
  exit 1
fi

tunnel_token="$(curl -fsS \
  -H "Authorization: Bearer ${cloudflare_api_token}" \
  -H 'Content-Type: application/json' \
  "https://api.cloudflare.com/client/v4/accounts/${cloudflare_account_id}/cfd_tunnel/${signy_tunnel_id}/token" \
  | jq -er '.result')"

signy_env_file="${temporary_dir}/signy.env"
tunnel_env_file="${temporary_dir}/signy-tunnel.env"
umask 077
cat >"${signy_env_file}" <<ENV
SIGNY_OBJECT_STORE_URL=s3://${signy_r2_bucket}/${signy_r2_prefix}
OBJECT_STORE_ENDPOINT=${signy_r2_endpoint}
OBJECT_STORE_REGION=auto
OBJECT_STORE_CONDITIONAL_PUT=etag
SIGNY_OBJECT_STORE_CATALOG_LOCKED=true
AWS_ACCESS_KEY_ID=${signy_r2_access_key_id}
AWS_SECRET_ACCESS_KEY=${signy_r2_secret_access_key}
SIGNY_LISTEN_ADDR=0.0.0.0:3100
SIGNY_LOG_FORMAT=json
RUST_LOG=signy=warn
SIGNY_MEMORY_BUDGET=2147483648
SIGNY_MEMORY_ACCOUNT_BYTES=1073741824
SIGNY_CACHE_MAX_BYTES=8589934592
SIGNY_MAX_WAL_BACKLOG_BYTES=1073741824
SIGNY_FLUSH_MAX_INTERVAL=60s
SIGNY_ORPHAN_GC_INTERVAL=1h
SIGNY_CATALOG_PRUNE_MIN_AGE=8d
SIGNY_MIN_FREE_DISK_BYTES=4294967296
SIGNY_RETENTION_INTERVAL=300s
SIGNY_RETENTION_GRACE_PERIOD=3600s
SIGNY_STARTUP_RETRY_BUDGET=300s
ENV
printf 'TUNNEL_TOKEN=%s\n' "${tunnel_token}" >"${tunnel_env_file}"
chmod 600 "${signy_env_file}" "${tunnel_env_file}"

target_ssh_key_file="${temporary_dir}/target-ssh-key"
pulumi_pick workerSshPrivateKey >"${target_ssh_key_file}"
chmod 600 "${target_ssh_key_file}"

bastion_port_forward_open signy-deploy "$(pulumi_pick workerBastionId)" "${dodb_private_ip}" 22
ssh_options=(
  -i "${target_ssh_key_file}" -p "${BASTION_LOCAL_PORT}" -o IdentitiesOnly=yes
  -o BatchMode=yes -o ConnectTimeout=10
  -o ServerAliveInterval=15 -o ServerAliveCountMax=3
  -o "UserKnownHostsFile=${BASTION_TEMP_DIR}/known_hosts"
  -o StrictHostKeyChecking=accept-new
)
scp_options=(
  -i "${target_ssh_key_file}" -P "${BASTION_LOCAL_PORT}" -o IdentitiesOnly=yes
  -o BatchMode=yes -o ConnectTimeout=10
  -o "UserKnownHostsFile=${BASTION_TEMP_DIR}/known_hosts"
  -o StrictHostKeyChecking=accept-new
)

scp "${scp_options[@]}" "${signy_env_file}" "${tunnel_env_file}" \
  opc@127.0.0.1:/tmp/

ssh "${ssh_options[@]}" opc@127.0.0.1 bash -s -- "${signy_image_ref}" <<'REMOTE_INSTALL'
set -euo pipefail

signy_image_ref="$1"
tunnel_image="docker.io/cloudflare/cloudflared:latest"

if ! command -v podman >/dev/null 2>&1; then
  sudo dnf install -y podman
fi

sudo podman pull "${signy_image_ref}"
image_architecture="$(sudo podman image inspect "${signy_image_ref}" --format '{{.Architecture}}')"
if [[ "${image_architecture}" != "arm64" && "${image_architecture}" != "aarch64" ]]; then
  echo "refusing to alter DODB VM: Signy image architecture is ${image_architecture}" >&2
  exit 1
fi

sudo podman pull "${tunnel_image}"
tunnel_image_id="$(sudo podman image inspect "${tunnel_image}" --format '{{.Id}}')"
tunnel_image_architecture="$(sudo podman image inspect "${tunnel_image}" --format '{{.Architecture}}')"
if [[ "${tunnel_image_architecture}" != "arm64" && "${tunnel_image_architecture}" != "aarch64" ]]; then
  echo "cloudflared image architecture is ${tunnel_image_architecture}" >&2
  exit 1
fi

sudo install -d -o 10001 -g 10001 -m 0750 /var/lib/signy
sudo install -d -o root -g root -m 0755 /etc/signy /etc/cloudflared
sudo install -o root -g root -m 0600 /tmp/signy.env /etc/signy/signy.env
sudo install -o root -g root -m 0600 /tmp/signy-tunnel.env /etc/cloudflared/fn0-signy-tunnel.env
rm -f /tmp/signy.env /tmp/signy-tunnel.env

sudo tee /etc/systemd/system/fn0-signy.service >/dev/null <<'EOF_SIGNY_UNIT'
[Unit]
Description=fn0 Signy telemetry store
After=network-online.target
Wants=network-online.target
RequiresMountsFor=/var/lib/signy

[Service]
Type=simple
Restart=always
RestartSec=5
TimeoutStopSec=45
MemoryMax=4G
MemoryHigh=2G
EnvironmentFile=/etc/signy/signy.env
ExecStartPre=-/usr/bin/podman rm -f fn0-signy
ExecStartPre=/usr/bin/podman pull ${SIGNY_IMAGE_REF}
ExecStart=/usr/bin/podman run --name fn0-signy --rm --security-opt label=disable --log-driver=k8s-file --log-opt max-size=100mb --memory=4g --memory-reservation=2g --env-file /etc/signy/signy.env -v /var/lib/signy:/var/lib/signy -p 127.0.0.1:3100:3100 ${SIGNY_IMAGE_REF}
ExecStop=/usr/bin/podman stop --time 30 fn0-signy

[Install]
WantedBy=multi-user.target
EOF_SIGNY_UNIT

sudo tee /etc/systemd/system/fn0-signy-tunnel.service >/dev/null <<EOF_TUNNEL_UNIT
[Unit]
Description=fn0 Signy Cloudflare Tunnel
After=network-online.target fn0-signy.service
Wants=network-online.target
Requires=fn0-signy.service

[Service]
Type=simple
Restart=always
RestartSec=5
TimeoutStopSec=30
EnvironmentFile=/etc/cloudflared/fn0-signy-tunnel.env
ExecStartPre=-/usr/bin/podman rm -f fn0-signy-tunnel
ExecStart=/usr/bin/podman run --name fn0-signy-tunnel --rm --network host --log-driver=k8s-file --log-opt max-size=20mb --env-file /etc/cloudflared/fn0-signy-tunnel.env ${tunnel_image_id} tunnel --no-autoupdate run
ExecStop=/usr/bin/podman stop --time 20 fn0-signy-tunnel

[Install]
WantedBy=multi-user.target
EOF_TUNNEL_UNIT

sudo sed -i "s|\${SIGNY_IMAGE_REF}|${signy_image_ref}|g" /etc/systemd/system/fn0-signy.service
sudo systemctl daemon-reload
sudo systemctl enable --now fn0-signy.service

for attempt in $(seq 1 150); do
  if curl -fsS http://127.0.0.1:3100/ready >/tmp/fn0-signy-ready.json 2>/dev/null; then
    break
  fi
  if [[ "${attempt}" -eq 150 ]]; then
    sudo journalctl -u fn0-signy.service -n 100 --no-pager >&2 || true
    exit 1
  fi
  sleep 2
done

ready_body="$(curl -fsS http://127.0.0.1:3100/ready)"
metrics_body="$(curl -fsS http://127.0.0.1:3100/metrics)"
remote_healthy="$(awk '$1 == "signy_remote_healthy" {print $2; exit}' <<<"${metrics_body}")"
if [[ "${remote_healthy}" != "1" ]]; then
  echo "Signy R2 health verification failed" >&2
  exit 1
fi

startup_logs="$(sudo podman logs fn0-signy 2>&1)"
for expected_log in \
  'claimed the object-store writer epoch' \
  'restored object-store manifest' \
  'restored trace object-store manifest' \
  'restored metric object-store manifest' \
  'loaded per-tenant retention policies'; do
  if ! grep -Fq "${expected_log}" <<<"${startup_logs}"; then
    echo "Signy startup evidence missing: ${expected_log}" >&2
    exit 1
  fi
done
if grep -Eiq 'panicked at|thread .* panicked|panic occurred' <<<"${startup_logs}"; then
  echo "Signy startup logs contain a panic" >&2
  exit 1
fi

sudo systemctl enable --now fn0-signy-tunnel.service
sudo systemctl is-active --quiet fn0-signy.service
sudo systemctl is-enabled --quiet fn0-signy.service
sudo systemctl is-active --quiet fn0-signy-tunnel.service
sudo systemctl is-enabled --quiet fn0-signy-tunnel.service
sudo test "$(stat -c '%a:%U:%G' /etc/signy/signy.env)" = "600:root:root"
sudo test "$(stat -c '%a:%U:%G' /etc/cloudflared/fn0-signy-tunnel.env)" = "600:root:root"
sudo test "$(stat -c '%u:%g' /var/lib/signy)" = "10001:10001"

metric_names_file="$(mktemp)"
metric_query_file="$(mktemp)"
logs_query_file="$(mktemp)"
trap 'rm -f "${metric_names_file}" "${metric_query_file}" "${logs_query_file}"' EXIT
platform_metric=""
for attempt in $(seq 1 60); do
  if curl -fsS -H 'X-Tenant-Id: fn0' \
    'http://127.0.0.1:3100/signy/api/v1/metrics/names?start=-8d' >"${metric_names_file}"; then
    platform_metric="$(jq -sr '[.[]? | .name // empty][0] // empty' "${metric_names_file}")"
    if [[ -n "${platform_metric}" ]]; then
      break
    fi
  fi
  sleep 5
done
if [[ -z "${platform_metric}" ]]; then
  echo "no platform tenant metric was available for the post-start query" >&2
  exit 1
fi
curl -fsS -G -H 'X-Tenant-Id: fn0' \
  --data-urlencode "metric=${platform_metric}" \
  --data-urlencode 'start=-5m' \
  --data-urlencode 'step=30s' \
  http://127.0.0.1:3100/signy/api/v1/metrics/query >"${metric_query_file}"
curl -fsS -G -H 'X-Tenant-Id: fn0' \
  --data-urlencode 'start=-1h' \
  --data-urlencode 'limit=1' \
  http://127.0.0.1:3100/signy/api/v1/logs >"${logs_query_file}"

printf 'signy_architecture=%s\n' "${image_architecture}"
printf 'signy_image=%s\n' "${signy_image_ref}"
printf 'signy_ready=%s\n' "${ready_body}"
printf 'signy_remote_healthy=%s\n' "${remote_healthy}"
printf 'startup_writer_epoch=claimed\n'
printf 'startup_catalog_restore=complete\n'
printf 'startup_logs_restore=complete\n'
printf 'startup_trace_restore=complete\n'
printf 'startup_metric_restore=complete\n'
printf 'startup_tenant_policy_load=complete\n'
printf 'tunnel_image=%s\n' "${tunnel_image_id}"
printf 'fn0-signy.service=active\n'
printf 'fn0-signy-tunnel.service=active\n'
printf 'platform_metric_query=%s\n' "${platform_metric}"
printf 'platform_metric_query_rows=%s\n' "$(wc -l <"${metric_query_file}" | tr -d ' ')"
printf 'platform_logs_query=success\n'
REMOTE_INSTALL

external_ready=""
tunnel_status=""
tunnel_connections="0"
attempt_number=0
while [[ "${attempt_number}" -lt 60 ]]; do
  tunnel_json="$(curl -fsS \
    -H "Authorization: Bearer ${cloudflare_api_token}" \
    -H 'Content-Type: application/json' \
    "https://api.cloudflare.com/client/v4/accounts/${cloudflare_account_id}/cfd_tunnel/${signy_tunnel_id}" 2>/dev/null || true)"
  tunnel_status="$(jq -r '.result.status // empty' <<<"${tunnel_json}" 2>/dev/null || true)"
  tunnel_connections="$(jq -r '.result.connections | length' <<<"${tunnel_json}" 2>/dev/null || printf '0')"
  external_ready="$(curl -fsS \
    -H "CF-Access-Client-Id: ${signy_access_client_id}" \
    -H "CF-Access-Client-Secret: ${signy_access_client_secret}" \
    "https://${signy_hostname}/ready" 2>/dev/null || true)"
  if [[ "${tunnel_status}" == "healthy" && "${tunnel_connections}" -gt 0 && -n "${external_ready}" ]]; then
    break
  fi
  attempt_number=$((attempt_number + 1))
  sleep 5
done
if [[ "${tunnel_status}" != "healthy" || "${tunnel_connections}" -lt 1 || -z "${external_ready}" ]]; then
  echo "existing Signy Tunnel did not become healthy with an external ready response" >&2
  echo "tunnel_status=${tunnel_status:-unknown} connections=${tunnel_connections}" >&2
  exit 1
fi

external_metrics="$(curl -fsS \
  -H "CF-Access-Client-Id: ${signy_access_client_id}" \
  -H "CF-Access-Client-Secret: ${signy_access_client_secret}" \
  "https://${signy_hostname}/metrics")"
external_remote_healthy="$(awk '$1 == "signy_remote_healthy" {print $2; exit}' <<<"${external_metrics}")"
if [[ "${external_remote_healthy}" != "1" ]]; then
  echo "external Signy R2 health verification failed" >&2
  exit 1
fi

external_metric_names="$(curl -fsS \
  -H "CF-Access-Client-Id: ${signy_access_client_id}" \
  -H "CF-Access-Client-Secret: ${signy_access_client_secret}" \
  -H 'X-Tenant-Id: fn0' \
  "https://${signy_hostname}/signy/api/v1/metrics/names?start=-8d")"
external_platform_metric="$(jq -sr '[.[]? | .name // empty][0] // empty' <<<"${external_metric_names}")"
if [[ -z "${external_platform_metric}" ]]; then
  echo "external platform metric discovery returned no fn0 metric" >&2
  exit 1
fi
curl -fsS -G \
  -H "CF-Access-Client-Id: ${signy_access_client_id}" \
  -H "CF-Access-Client-Secret: ${signy_access_client_secret}" \
  -H 'X-Tenant-Id: fn0' \
  --data-urlencode "metric=${external_platform_metric}" \
  --data-urlencode 'start=-5m' \
  --data-urlencode 'step=30s' \
  "https://${signy_hostname}/signy/api/v1/metrics/query" >/dev/null
curl -fsS -G \
  -H "CF-Access-Client-Id: ${signy_access_client_id}" \
  -H "CF-Access-Client-Secret: ${signy_access_client_secret}" \
  -H 'X-Tenant-Id: fn0' \
  --data-urlencode 'start=-1h' \
  --data-urlencode 'limit=1' \
  "https://${signy_hostname}/signy/api/v1/logs" >/dev/null

echo ">> DODB Signy deployment and localhost checks completed"
echo ">> existing Signy Tunnel status=${tunnel_status} connections=${tunnel_connections}"
echo ">> external Signy ready and R2 health verified"
echo ">> external platform metric query=${external_platform_metric}; logs query succeeded"
