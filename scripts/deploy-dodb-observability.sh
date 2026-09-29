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
  collectyImageRefOutput \
  signyHostnameOutput \
  signyAccessClientId \
  signyAccessClientSecret

dodb_private_ip="$(pulumi_pick dodbPrivateIp)"
collecty_image_ref="$(pulumi_pick collectyImageRefOutput)"
signy_url="https://$(pulumi_pick signyHostnameOutput)"
signy_access_client_id="$(pulumi_pick signyAccessClientId)"
signy_access_client_secret="$(pulumi_pick signyAccessClientSecret)"

for value in \
  "${collecty_image_ref}" \
  "${signy_url}" \
  "${signy_access_client_id}" \
  "${signy_access_client_secret}"; do
  if [[ ! "${value}" =~ ^[A-Za-z0-9._/:@+=-]+$ ]]; then
    echo "a Pulumi output contains characters unsupported by the collecty environment file" >&2
    exit 1
  fi
done

collecty_env_file="${temporary_dir}/collecty.env"
umask 077
cat >"${collecty_env_file}" <<ENV
COLLECTY_IMAGE_REF=${collecty_image_ref}
COLLECTY_LISTEN_ADDR=127.0.0.1:4318
COLLECTY_SIGNY_URL=${signy_url}
COLLECTY_SIGNY_ACCESS_CLIENT_ID=${signy_access_client_id}
COLLECTY_SIGNY_ACCESS_CLIENT_SECRET=${signy_access_client_secret}
COLLECTY_GENERATED_TELEMETRY_TENANT_ID=fn0
COLLECTY_HOST_METRICS_INTERVAL=60s
COLLECTY_HOST_METRICS_ROOT=/host
COLLECTY_QUEUE_MAX_BYTES=1GiB
COLLECTY_QUEUE_SEGMENT_BYTES=8MiB
COLLECTY_SEGMENT_MAX_AGE=2s
COLLECTY_RETRY_INITIAL=1s
COLLECTY_RETRY_MAX=30s
COLLECTY_SEND_TIMEOUT=30s
COLLECTY_REPORT_INTERVAL=60s
COLLECTY_ZSTD_LEVEL=3
COLLECTY_LOG=warn
ENV

target_ssh_key_file="${temporary_dir}/target-ssh-key"
pulumi_pick workerSshPrivateKey >"${target_ssh_key_file}"
chmod 600 "${target_ssh_key_file}"

bastion_port_forward_open observability "$(pulumi_pick workerBastionId)" "${dodb_private_ip}" 22
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

echo ">> Copying dodb collecty configuration through localhost:${BASTION_LOCAL_PORT}"
scp "${scp_options[@]}" "${collecty_env_file}" opc@127.0.0.1:/tmp/fn0-collecty.env.new

ssh "${ssh_options[@]}" opc@127.0.0.1 bash -s <<'REMOTE_INSTALL'
set -euo pipefail

if ! command -v podman >/dev/null 2>&1; then
  sudo dnf install -y podman
fi

sudo install -d -o 10002 -g 10002 -m 0750 /var/lib/collecty
sudo install -d -o root -g root -m 0755 /etc/fn0-collecty
sudo install -o root -g root -m 0600 /tmp/fn0-collecty.env.new /etc/fn0-collecty/env
rm -f /tmp/fn0-collecty.env.new

sudo tee /etc/systemd/system/fn0-collecty.service >/dev/null <<'EOF_COLLECTY_UNIT'
[Unit]
Description=fn0 collecty durable telemetry queue for dodb
After=network-online.target
Wants=network-online.target
RequiresMountsFor=/var/lib/collecty

[Service]
Type=simple
Restart=always
RestartSec=5
MemoryMax=256M
MemoryHigh=96M
EnvironmentFile=/etc/fn0-collecty/env
ExecStartPre=-/usr/bin/podman rm -f fn0-collecty
ExecStartPre=/usr/bin/podman pull ${COLLECTY_IMAGE_REF}
ExecStart=/usr/bin/podman run --name fn0-collecty --rm \
  --security-opt label=disable \
  --network host \
  --log-opt max-size=20mb \
  --env-file /etc/fn0-collecty/env \
  -v /var/lib/collecty:/var/lib/collecty \
  -v /:/host:ro,rslave \
  -v /proc:/host/proc:ro \
  -v /sys:/host/sys:ro \
  ${COLLECTY_IMAGE_REF}
ExecStop=/usr/bin/podman stop -t 30 fn0-collecty

[Install]
WantedBy=multi-user.target
EOF_COLLECTY_UNIT

sudo systemctl daemon-reload
sudo systemctl enable --now fn0-collecty.service
sudo systemctl is-active fn0-collecty.service
sudo systemctl is-enabled fn0-collecty.service
sudo test "$(stat -c '%a:%U:%G' /etc/fn0-collecty/env)" = "600:root:root"
sudo test "$(stat -c '%u:%g' /var/lib/collecty)" = "10002:10002"
REMOTE_INSTALL

echo ">> dodb collecty host telemetry is installed; dodb-server was not restarted"
