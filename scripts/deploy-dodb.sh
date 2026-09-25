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
need ssh-keygen
need python3

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
  dodbPrivateIp

dodb_private_ip="$(pulumi_pick dodbPrivateIp)"

binary_dir="${temporary_dir}/bin"
mkdir -p "${binary_dir}"
"${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh" dodb-server "${binary_dir}"

target_ssh_key_file="${temporary_dir}/target-ssh-key"
pulumi_pick workerSshPrivateKey >"${target_ssh_key_file}"
chmod 600 "${target_ssh_key_file}"

bastion_port_forward_open deploy "$(pulumi_pick workerBastionId)" "${dodb_private_ip}" 22
ssh_options=(
  -i "${target_ssh_key_file}" -p "${BASTION_LOCAL_PORT}" -o IdentitiesOnly=yes
  -o BatchMode=yes -o ConnectTimeout=10
  -o ServerAliveInterval=15 -o ServerAliveCountMax=3
  -o "UserKnownHostsFile=${BASTION_TEMP_DIR}/known_hosts"
  -o StrictHostKeyChecking=accept-new
)
local_port="${BASTION_LOCAL_PORT}"
scp_options=(
  -i "${target_ssh_key_file}" -P "${local_port}" -o IdentitiesOnly=yes
  -o BatchMode=yes -o ConnectTimeout=10
  -o "UserKnownHostsFile=${BASTION_TEMP_DIR}/known_hosts"
  -o StrictHostKeyChecking=accept-new
)
echo ">> Copying dodb-server to ${dodb_private_ip} through localhost:${local_port}"
scp "${scp_options[@]}" "${binary_dir}/dodb-server" "opc@127.0.0.1:/tmp/dodb-server.new"

ssh "${ssh_options[@]}" opc@127.0.0.1 bash -s <<'REMOTE_INSTALL'
set -euo pipefail
sudo install -o root -g root -m 0755 /tmp/dodb-server.new /usr/local/bin/dodb-server
rm -f /tmp/dodb-server.new
sudo systemctl daemon-reload
sudo systemctl enable dodb.service
sudo systemctl restart dodb.service
sudo systemctl is-active dodb.service
REMOTE_INSTALL

ssh "${ssh_options[@]}" opc@127.0.0.1 bash -s <<'REMOTE_VERIFY'
set -euo pipefail
sudo systemctl is-active dodb.service
sudo systemctl is-enabled dodb.service
sudo systemctl status dodb.service --no-pager
if ! sudo ss -lun | grep -E '(:18445)([[:space:]]|$)'; then
  echo "UDP port 18445 is not listening" >&2
  exit 1
fi
test -d /var/lib/dodb
test "$(stat -c '%U:%G' /var/lib/dodb)" = "dodb:dodb"
test "$(sudo stat -c '%a:%U:%G' /etc/dodb/server.crt)" = "644:root:dodb"
test "$(sudo stat -c '%a:%U:%G' /etc/dodb/server.key)" = "640:root:dodb"
sudo openssl x509 -in /etc/dodb/server.crt -noout -ext subjectAltName
if sudo systemctl is-active --quiet firewalld; then
  if ! sudo firewall-cmd --query-port=18445/udp >/dev/null; then
    sudo firewall-cmd --permanent --add-port=18445/udp
    sudo firewall-cmd --reload
  fi
  sudo firewall-cmd --query-port=18445/udp
  if sudo firewall-cmd --query-port=18445/tcp >/dev/null; then
    echo "TCP port 18445 is already allowed; this deployment did not add that rule" >&2
  else
    echo "TCP port 18445 is not allowed"
  fi
else
  echo "firewalld is not active; no guest firewall rule was added"
fi
root_size_bytes="$(df -B1 --output=size / | tail -n 1 | tr -d ' ')"
if [ "${root_size_bytes}" -lt 100000000000 ]; then
  echo "root filesystem is not expanded to the 150 GB boot volume: ${root_size_bytes} bytes" >&2
  exit 1
fi
df -h /
df -h /var/lib/dodb
echo "Recent dodb journal errors:"
sudo journalctl -u dodb.service -p err --since "30 minutes ago" --no-pager
REMOTE_VERIFY

echo ">> dodb deployment and verification complete"
