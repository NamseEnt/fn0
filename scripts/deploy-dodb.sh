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

cleanup() {
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
  dodbInstanceId \
  dodbPrivateIp \
  dodbServerName \
  dodbRootCertPem

dodb_server_name="$(pulumi_pick dodbServerName)"
dodb_private_ip="$(pulumi_pick dodbPrivateIp)"
if [[ "${dodb_server_name}" != "dodb.internal" ]]; then
  echo "unexpected dodb server name: ${dodb_server_name}" >&2
  exit 1
fi

binary_dir="${temporary_dir}/bin"
mkdir -p "${binary_dir}"
"${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh" dodb-server "${binary_dir}"

private_key_file="${temporary_dir}/worker-ssh-key"
public_key_file="${temporary_dir}/worker-ssh-key.pub"
pulumi_pick workerSshPrivateKey >"${private_key_file}"
chmod 600 "${private_key_file}"
ssh-keygen -y -f "${private_key_file}" >"${public_key_file}"
chmod 600 "${public_key_file}"

session_response="$(oci bastion session create-managed-ssh \
  --bastion-id "$(pulumi_pick workerBastionId)" \
  --target-resource-id "$(pulumi_pick dodbInstanceId)" \
  --target-os-username opc \
  --target-port 22 \
  --ssh-public-key-file "${public_key_file}" \
  --session-ttl 3600 \
  --wait-for-state SUCCEEDED \
  --output json)"
session_id="$(jq -r '.data.id // empty' <<<"${session_response}")"
if [[ -z "${session_id}" ]]; then
  echo "OCI Bastion did not return a managed SSH session ID" >&2
  exit 1
fi
echo ">> Created OCI Bastion managed SSH session ${session_id}"

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

mapfile -d '' -t session_ssh_args < <(
  python3 -c 'import shlex, sys; [sys.stdout.buffer.write(word.encode() + b"\0") for word in shlex.split(sys.argv[1])]' "${session_ssh_command}"
)
if [[ "${#session_ssh_args[@]}" -lt 2 || "${session_ssh_args[0]}" != "ssh" ]]; then
  echo "OCI Bastion returned an unsupported managed SSH command" >&2
  exit 1
fi
for argument_index in "${!session_ssh_args[@]}"; do
  if [[ "${session_ssh_args[argument_index]}" == "<privateKey>" ]]; then
    session_ssh_args[argument_index]="${private_key_file}"
  fi
done
session_target="${session_ssh_args[${#session_ssh_args[@]}-1]}"
if [[ "${session_target}" != "opc@${dodb_private_ip}" && "${session_target}" != "opc@${dodb_private_ip}:22" ]]; then
  echo "OCI Bastion command targets an unexpected host: ${session_target}" >&2
  exit 1
fi

scp_arguments=("${session_ssh_args[@]:1:${#session_ssh_args[@]}-2}")
echo ">> Bastion session is ACTIVE; copying dodb-server to ${dodb_private_ip}"
scp "${scp_arguments[@]}" "${binary_dir}/dodb-server" "${session_target}:/tmp/dodb-server.new"

ssh "${session_ssh_args[@]:1:${#session_ssh_args[@]}-1}" bash -s <<'REMOTE_INSTALL'
set -euo pipefail
sudo install -o root -g root -m 0755 /tmp/dodb-server.new /usr/local/bin/dodb-server
rm -f /tmp/dodb-server.new
sudo systemctl daemon-reload
sudo systemctl enable dodb.service
sudo systemctl restart dodb.service
sudo systemctl is-active dodb.service
REMOTE_INSTALL

ssh "${session_ssh_args[@]:1:${#session_ssh_args[@]}-1}" bash -s <<'REMOTE_VERIFY'
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
