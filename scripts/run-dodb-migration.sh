#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export REPO_ROOT

source "${REPO_ROOT}/scripts/lib/pulumi-outputs.sh"

usage() {
  cat <<'USAGE'
Usage:
  scripts/run-dodb-migration.sh inventory [migration options]
  scripts/run-dodb-migration.sh verify [migration options]
  scripts/run-dodb-migration.sh migrate --apply [migration options]

The migration binary runs on the dodb VM. The migrate command requires --apply.
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "help" ]]; then
  usage
  exit 0
fi
if [[ "$#" -eq 2 && "${2}" == "--help" ]]; then
  case "${1}" in inventory|verify|migrate) usage; exit 0 ;; esac
fi
if [[ "$#" -lt 1 ]]; then
  usage >&2
  exit 2
fi

command_name="$1"
shift
case "${command_name}" in
  inventory|verify|migrate) ;;
  *) echo "unsupported migration command: ${command_name}" >&2; usage >&2; exit 2 ;;
esac
if [[ "${command_name}" == "migrate" ]]; then
  apply_found=false
  for argument in "$@"; do
    if [[ "${argument}" == "--apply" ]]; then apply_found=true; fi
  done
  if [[ "${apply_found}" != "true" ]]; then
    echo "migrate writes to dodb and requires the explicit --apply flag" >&2
    exit 2
  fi
fi
if [[ "${command_name}" != "inventory" ]]; then
  for argument in "$@"; do
    case "${argument}" in
      --dodb-addr|--dodb-addr=*|--dodb-server-name|--dodb-server-name=*|--dodb-root-cert|--dodb-root-cert=*)
        echo "the remote runner fixes dodb to its local production endpoint and certificate" >&2
        exit 2
        ;;
    esac
  done
fi

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
remote_cleanup_needed=false

cleanup() {
  if [[ "${remote_cleanup_needed}" == "true" && "${tunnel_pid}" != "" ]]; then
    ssh "${ssh_options[@]}" opc@127.0.0.1 bash -s -- \
      "${remote_binary}" "${remote_env_file}" "${remote_argument_file}" "${remote_temp_dir}" >/dev/null 2>&1 <<'REMOTE_CLEANUP' || true
set -euo pipefail
rm -f "$1" "$2" "$3"
sudo rm -f "$4/server.crt" 2>/dev/null || true
rmdir "$4" 2>/dev/null || true
REMOTE_CLEANUP
    echo ">> Remote migration temporary files cleanup attempted" >&2
  fi
  if [[ -n "${tunnel_pid}" ]]; then
    kill "${tunnel_pid}" >/dev/null 2>&1 || true
    wait "${tunnel_pid}" >/dev/null 2>&1 || true
    echo ">> Bastion tunnel cleanup attempted" >&2
  fi
  if [[ -n "${session_id}" ]]; then
    oci bastion session delete --session-id "${session_id}" --force >/dev/null 2>&1 || true
    echo ">> Bastion session cleanup attempted" >&2
  fi
  rm -rf "${temporary_dir}"
}
trap cleanup EXIT

load_pulumi_outputs >&2
require_pulumi_output \
  forteDbGroupToken \
  forteDbHostSuffix \
  workerBastionId \
  workerSshPrivateKey \
  dodbPrivateIp
turso_group_token="$(pulumi_pick forteDbGroupToken)"
turso_db_host_suffix="$(pulumi_pick forteDbHostSuffix)"
dodb_private_ip="$(pulumi_pick dodbPrivateIp)"

binary_dir="${temporary_dir}/bin"
mkdir -p "${binary_dir}"
"${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh" fn0-db-migrate "${binary_dir}" >&2

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

session_json=""
session_state=""
attempt=0
while [[ "${attempt}" -lt 60 ]]; do
  session_json="$(oci bastion session get --session-id "${session_id}" --output json)"
  session_state="$(jq -r '.data."lifecycle-state" // .data.lifecycleState // empty' <<<"${session_json}")"
  if [[ "${session_state}" == "ACTIVE" ]]; then break; fi
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
session_ssh_args_file="${temporary_dir}/bastion-ssh-args"
python3 -c '
import shlex
import sys

arguments = shlex.split(sys.argv[1])
for argument_index, argument in enumerate(arguments):
    arguments[argument_index] = argument.replace("<privateKey>", sys.argv[2]).replace("<localPort>", sys.argv[3])
for argument in arguments:
    sys.stdout.buffer.write(argument.encode() + b"\0")
' "${session_ssh_command}" "${private_key_file}" "${local_port}" >"${session_ssh_args_file}"
session_ssh_args=()
while IFS= read -r -d '' session_argument; do session_ssh_args+=("${session_argument}"); done <"${session_ssh_args_file}"
if [[ "${#session_ssh_args[@]}" -lt 2 || "${session_ssh_args[0]}" != "ssh" ]]; then
  echo "OCI Bastion returned an unsupported port-forwarding command" >&2
  exit 1
fi
forwarding_destination=""
no_remote_command=false
for ((argument_index = 0; argument_index < ${#session_ssh_args[@]}; argument_index += 1)); do
  current_argument="${session_ssh_args[argument_index]}"
  if [[ "${current_argument}" == "-N" ]]; then no_remote_command=true; fi
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
if [[ "${no_remote_command}" != "true" || "${forwarding_destination}" != "${dodb_private_ip}:22" ]]; then
  echo "OCI Bastion command has an invalid port-forwarding target or mode" >&2
  exit 1
fi
for current_argument in "${session_ssh_args[@]}"; do
  if [[ "${current_argument}" == *"<privateKey>"* || "${current_argument}" == *"<localPort>"* ]]; then
    echo "OCI Bastion command placeholders were not fully replaced" >&2
    exit 1
  fi
done
session_ssh_args+=(-o ExitOnForwardFailure=yes)
tunnel_log="${temporary_dir}/bastion-tunnel.log"
"${session_ssh_args[@]}" >"${tunnel_log}" 2>&1 &
tunnel_pid="$!"

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

remote_nonce="$(python3 -c 'import secrets; print(secrets.token_hex(12))')"
remote_binary="/tmp/fn0-db-migrate"
remote_env_file="/tmp/fn0-db-migrate.env.${remote_nonce}"
remote_argument_file="/tmp/fn0-db-migrate.args.${remote_nonce}"
remote_temp_dir="/tmp/fn0-db-migrate.${remote_nonce}"
env_file="${temporary_dir}/migration.env"
TURSO_GROUP_TOKEN="${turso_group_token}" TURSO_DB_HOST_SUFFIX="${turso_db_host_suffix}" \
  python3 -c 'import os, shlex; print("export TURSO_GROUP_TOKEN=" + shlex.quote(os.environ["TURSO_GROUP_TOKEN"])); print("export TURSO_DB_HOST_SUFFIX=" + shlex.quote(os.environ["TURSO_DB_HOST_SUFFIX"]))' \
  >"${env_file}"
chmod 600 "${env_file}"
argument_file="${temporary_dir}/migration-args.json"
python3 - "${argument_file}" "${command_name}" "$@" <<'PY'
import json
import sys

with open(sys.argv[1], "w", encoding="utf-8") as argument_file:
    json.dump(sys.argv[2:], argument_file)
PY
chmod 600 "${argument_file}"

echo ">> Bastion Port Forwarding is ACTIVE; transferring migration runner inputs"
remote_cleanup_needed=true
scp "${scp_options[@]}" "${binary_dir}/fn0-db-migrate" "opc@127.0.0.1:${remote_binary}"
scp "${scp_options[@]}" "${env_file}" "opc@127.0.0.1:${remote_env_file}"
scp "${scp_options[@]}" "${argument_file}" "opc@127.0.0.1:${remote_argument_file}"

ssh "${ssh_options[@]}" opc@127.0.0.1 bash -s -- "${remote_binary}" "${remote_env_file}" "${remote_argument_file}" "${remote_temp_dir}" <<'REMOTE_RUNNER'
set -euo pipefail
remote_binary="$1"
remote_env_file="$2"
remote_argument_file="$3"
remote_temp_dir="$4"
cleanup_remote() {
  rm -f "$remote_binary" "$remote_env_file" "$remote_argument_file"
  sudo rm -f "$remote_temp_dir/server.crt" 2>/dev/null || true
  rmdir "$remote_temp_dir" 2>/dev/null || true
}
trap cleanup_remote EXIT
chmod 700 "$remote_binary"
chmod 600 "$remote_env_file" "$remote_argument_file"
umask 077
mkdir -m 700 "$remote_temp_dir"
sudo install -o opc -g opc -m 0600 /etc/dodb/server.crt "$remote_temp_dir/server.crt"
export DODB_ADDR=127.0.0.1:18445
export DODB_SERVER_NAME=dodb.internal
set -a
source "$remote_env_file"
set +a
python3 - "$remote_binary" "$remote_argument_file" "$remote_temp_dir/server.crt" <<'PY'
import json
import os
import sys

binary_path = sys.argv[1]
with open(sys.argv[2], encoding="utf-8") as argument_file:
    arguments = json.load(argument_file)
if arguments[0] in ("migrate", "verify"):
    arguments.extend(["--dodb-addr", "127.0.0.1:18445"])
    arguments.extend(["--dodb-server-name", "dodb.internal"])
    arguments.extend(["--dodb-root-cert", sys.argv[3]])
os.execv(binary_path, [binary_path, *arguments])
PY
REMOTE_RUNNER
