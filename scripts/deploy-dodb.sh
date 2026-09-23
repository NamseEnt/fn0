#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT

source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"

need pulumi
need jq
need oci
need ssh
need scp
need ssh-keygen
need python3

temporary_dir="$(mktemp -d)"
chmod 700 "${temporary_dir}"
session_id=""
tunnel_pid=""

cleanup() {
  if [[ -n "${tunnel_pid}" ]]; then
    kill "${tunnel_pid}" >/dev/null 2>&1 || true
    wait "${tunnel_pid}" >/dev/null 2>&1 || true
    echo ">> Bastion tunnel cleanup attempted"
  fi
  if [[ -n "${session_id}" ]]; then
    oci bastion session delete --session-id "${session_id}" --force >/dev/null 2>&1 || true
    echo ">> Bastion session cleanup attempted"
  fi
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

private_key_file="${temporary_dir}/worker-ssh-key"
public_key_file="${temporary_dir}/worker-ssh-key.pub"
pulumi_pick workerSshPrivateKey >"${private_key_file}"
chmod 600 "${private_key_file}"
ssh-keygen -y -f "${private_key_file}" >"${public_key_file}"
chmod 600 "${public_key_file}"

session_response="$(oci bastion session create-port-forwarding \
  --bastion-id "$(pulumi_pick workerBastionId)" \
  --target-private-ip "${dodb_private_ip}" \
  --target-port 22 \
  --ssh-public-key-file "${public_key_file}" \
  --session-ttl 3600 \
  --wait-for-state SUCCEEDED \
  --output json)"
work_request_id="$(jq -r '.data.id // empty' <<<"${session_response}")"
if [[ -z "${work_request_id}" ]]; then
  echo "OCI Bastion did not return a port-forwarding session work request ID" >&2
  exit 1
fi
work_request_json="$(oci bastion work-request get --work-request-id "${work_request_id}" --output json)"
session_id="$(jq -r '.data.resources[]? | select(."entity-type" == "SessionResource") | .identifier' <<<"${work_request_json}" | head -n 1)"
if [[ -z "${session_id}" ]]; then
  echo "OCI Bastion work request did not return a session ID" >&2
  exit 1
fi
echo ">> Created OCI Bastion port forwarding session ${session_id}"

session_json=""
session_state=""
attempt=0
while [[ "${attempt}" -lt 60 ]]; do
  session_json="$(oci bastion session get --session-id "${session_id}" --output json)"
  session_state="$(jq -r '.data."lifecycle-state" // .data.lifecycleState // empty' <<<"${session_json}")"
  if [[ "${session_state}" == "ACTIVE" ]]; then
    break
  fi
  if [[ "${session_state}" == "FAILED" || "${session_state}" == "DELETED" ]]; then
    echo "OCI Bastion session entered ${session_state}" >&2
    exit 1
  fi
  attempt=$((attempt + 1))
  sleep 5
done
if [[ "${session_state}" != "ACTIVE" ]]; then
  echo "OCI Bastion session did not become ACTIVE; current state=${session_state:-unknown}" >&2
  exit 1
fi

session_ssh_command="$(jq -r '.data."ssh-metadata".command // .data.sshMetadata.command // empty' <<<"${session_json}")"
if [[ -z "${session_ssh_command}" ]]; then
  echo "OCI Bastion session has no ssh-metadata.command" >&2
  exit 1
fi

local_port="$(python3 -c 'import socket; listener = socket.socket(); listener.bind(("127.0.0.1", 0)); print(listener.getsockname()[1]); listener.close()')"
if [[ ! "${local_port}" =~ ^[0-9]+$ ]] || [[ "${local_port}" -lt 1 ]] || [[ "${local_port}" -gt 65535 ]]; then
  echo "could not choose a valid local tunnel port: ${local_port}" >&2
  exit 1
fi

mapfile -d '' -t session_ssh_args < <(
  python3 -c '
import shlex
import sys

arguments = shlex.split(sys.argv[1])
for argument_index, argument in enumerate(arguments):
    arguments[argument_index] = argument.replace("<privateKey>", sys.argv[2]).replace("<localPort>", sys.argv[3])
for argument in arguments:
    sys.stdout.buffer.write(argument.encode() + b"\0")
' "${session_ssh_command}" "${private_key_file}" "${local_port}"
)
if [[ "${#session_ssh_args[@]}" -lt 2 || "${session_ssh_args[0]}" != "ssh" ]]; then
  echo "OCI Bastion returned an unsupported port-forwarding command" >&2
  exit 1
fi
forwarding_destination=""
for ((argument_index = 0; argument_index < ${#session_ssh_args[@]}; argument_index += 1)); do
  current_argument="${session_ssh_args[argument_index]}"
  if [[ "${current_argument}" == "-N" ]]; then
    no_remote_command=true
  fi
  if [[ "${current_argument}" == "-L" ]]; then
    argument_index=$((argument_index + 1))
    current_argument="${session_ssh_args[argument_index]:-}"
  elif [[ "${current_argument}" == -L* ]]; then
    current_argument="${current_argument#-L}"
  fi
  if [[ "${current_argument}" == "${local_port}:"* ]]; then
    forwarding_destination="${current_argument#"${local_port}:"}"
  fi
done
if [[ "${no_remote_command:-false}" != "true" ]]; then
  echo "OCI Bastion port-forwarding command is missing -N" >&2
  exit 1
fi
if [[ "${forwarding_destination}" != "${dodb_private_ip}:22" ]]; then
  echo "OCI Bastion command forwards to an unexpected destination: ${forwarding_destination:-missing}" >&2
  exit 1
fi
if [[ "${session_ssh_command}" == *"<privateKey>"* || "${session_ssh_command}" == *"<localPort>"* ]]; then
  echo "OCI Bastion command placeholders were not fully replaced" >&2
  exit 1
fi
session_ssh_args+=(-o ExitOnForwardFailure=yes)

tunnel_log="${temporary_dir}/bastion-tunnel.log"
"${session_ssh_args[@]}" >"${tunnel_log}" 2>&1 &
tunnel_pid="$!"
echo ">> Bastion session is ACTIVE; starting localhost port ${local_port} tunnel to ${dodb_private_ip}:22"

tunnel_ready=false
for attempt in $(seq 1 60); do
  if ! kill -0 "${tunnel_pid}" >/dev/null 2>&1; then
    cat "${tunnel_log}" >&2
    echo "Bastion port-forwarding tunnel exited before becoming ready" >&2
    exit 1
  fi
  if python3 -c 'import socket, sys; connection = socket.socket(); connection.settimeout(1); result = connection.connect_ex(("127.0.0.1", int(sys.argv[1]))); connection.close(); raise SystemExit(0 if result == 0 else 1)' "${local_port}"; then
    tunnel_ready=true
    break
  fi
  sleep 1
done
if [[ "${tunnel_ready}" != "true" ]]; then
  cat "${tunnel_log}" >&2
  echo "Bastion tunnel did not accept connections on 127.0.0.1:${local_port}" >&2
  exit 1
fi

temporary_known_hosts="${temporary_dir}/known_hosts"
touch "${temporary_known_hosts}"
chmod 600 "${temporary_known_hosts}"
ssh_options=(
  -i "${private_key_file}"
  -p "${local_port}"
  -o IdentitiesOnly=yes
  -o BatchMode=yes
  -o ConnectTimeout=10
  -o "UserKnownHostsFile=${temporary_known_hosts}"
  -o StrictHostKeyChecking=accept-new
)
scp_options=(
  -i "${private_key_file}"
  -P "${local_port}"
  -o IdentitiesOnly=yes
  -o BatchMode=yes
  -o ConnectTimeout=10
  -o "UserKnownHostsFile=${temporary_known_hosts}"
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
test "$(stat -c '%a:%U:%G' /etc/dodb/server.crt)" = "644:root:dodb"
test "$(stat -c '%a:%U:%G' /etc/dodb/server.key)" = "640:root:dodb"
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
