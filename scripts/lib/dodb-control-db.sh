# shellcheck shell=bash

if [[ -n "${__FN0_DODB_CONTROL_DB_LOADED:-}" ]]; then
  return 0
fi
__FN0_DODB_CONTROL_DB_LOADED=1
unset __FN0_DODB_CONTROL_DB_OPEN

dodb_control_db_open() {
  if [[ -n "${__FN0_DODB_CONTROL_DB_OPEN:-}" ]]; then
    return 0
  fi

  local temporary_dir binary_dir private_key_file public_key_file
  local session_response work_request_id work_request_json session_json
  local session_state attempt session_ssh_command local_port session_ssh_args_file
  local session_argument current_argument forwarding_destination tunnel_log
  local tunnel_ready remote_nonce local_binary local_binary_sha remote_cache_dir cache_status remote_sha

  need pulumi
  need jq
  need oci
  need ssh
  need gzip
  need sha256sum
  need ssh-keygen
  need python3

  temporary_dir="$(mktemp -d)"
  chmod 700 "$temporary_dir"
  DODB_CONTROL_DB_TEMP_DIR="$temporary_dir"
  DODB_CONTROL_DB_SESSION_ID=""
  DODB_CONTROL_DB_TUNNEL_PID=""

  dodb_private_ip="$(pulumi_pick dodbPrivateIp)"
  bastion_id="$(pulumi_pick workerBastionId)"
  if [[ -z "$dodb_private_ip" || -z "$bastion_id" ]]; then
    echo "missing Pulumi outputs: dodbPrivateIp / workerBastionId" >&2
    return 1
  fi

  binary_dir="${temporary_dir}/bin"
  mkdir -p "$binary_dir"
  "${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh" fn0-db-ops "$binary_dir" >&2

  private_key_file="${temporary_dir}/worker-ssh-key"
  public_key_file="${temporary_dir}/worker-ssh-key.pub"
  pulumi_pick workerSshPrivateKey >"$private_key_file"
  chmod 600 "$private_key_file"
  ssh-keygen -y -f "$private_key_file" >"$public_key_file"
  chmod 600 "$public_key_file"

  session_response="$(oci bastion session create-port-forwarding \
    --bastion-id "$bastion_id" \
    --target-private-ip "$dodb_private_ip" \
    --target-port 22 \
    --ssh-public-key-file "$public_key_file" \
    --session-ttl 3600 \
    --wait-for-state SUCCEEDED \
    --output json)"
  work_request_id="$(jq -r '.data.id // empty' <<<"$session_response")"
  if [[ -z "$work_request_id" ]]; then
    echo "OCI Bastion did not return a port-forwarding session work request ID" >&2
    return 1
  fi
  work_request_json="$(oci bastion work-request get --work-request-id "$work_request_id" --output json)"
  DODB_CONTROL_DB_SESSION_ID="$(jq -r '.data.resources[]? | select(."entity-type" == "SessionResource") | .identifier' <<<"$work_request_json" | head -n 1)"
  if [[ -z "$DODB_CONTROL_DB_SESSION_ID" ]]; then
    echo "OCI Bastion work request did not return a session ID" >&2
    return 1
  fi

  session_state=""
  attempt=0
  while [[ "$attempt" -lt 60 ]]; do
    session_json="$(oci bastion session get --session-id "$DODB_CONTROL_DB_SESSION_ID" --output json)"
    session_state="$(jq -r '.data."lifecycle-state" // .data.lifecycleState // empty' <<<"$session_json")"
    if [[ "$session_state" == "ACTIVE" ]]; then break; fi
    if [[ "$session_state" == "FAILED" || "$session_state" == "DELETED" ]]; then
      echo "OCI Bastion session entered $session_state" >&2
      return 1
    fi
    attempt=$((attempt + 1))
    sleep 5
  done
  if [[ "$session_state" != "ACTIVE" ]]; then
    echo "OCI Bastion session did not become ACTIVE" >&2
    return 1
  fi

  session_ssh_command="$(jq -r '.data."ssh-metadata".command // .data.sshMetadata.command // empty' <<<"$session_json")"
  if [[ -z "$session_ssh_command" ]]; then
    echo "OCI Bastion session has no ssh-metadata.command" >&2
    return 1
  fi
  local_port="$(python3 -c 'import socket; listener = socket.socket(); listener.bind(("127.0.0.1", 0)); print(listener.getsockname()[1]); listener.close()')"
  if [[ ! "$local_port" =~ ^[0-9]+$ ]] || [[ "$local_port" -lt 1 || "$local_port" -gt 65535 ]]; then
    echo "could not choose a valid local tunnel port: $local_port" >&2
    return 1
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
' "$session_ssh_command" "$private_key_file" "$local_port" >"$session_ssh_args_file"
  DODB_CONTROL_DB_SSH_ARGS=()
  while IFS= read -r -d '' session_argument; do
    DODB_CONTROL_DB_SSH_ARGS+=("$session_argument")
  done <"$session_ssh_args_file"
  if [[ "${#DODB_CONTROL_DB_SSH_ARGS[@]}" -lt 2 || "${DODB_CONTROL_DB_SSH_ARGS[0]}" != ssh ]]; then
    echo "OCI Bastion returned an unsupported port-forwarding command" >&2
    return 1
  fi
  forwarding_destination=""
  local no_remote_command=false
  for ((argument_index = 0; argument_index < ${#DODB_CONTROL_DB_SSH_ARGS[@]}; argument_index += 1)); do
    current_argument="${DODB_CONTROL_DB_SSH_ARGS[argument_index]}"
    if [[ "$current_argument" == -N ]]; then no_remote_command=true; fi
    if [[ "$current_argument" == -L ]]; then
      argument_index=$((argument_index + 1))
      current_argument="${DODB_CONTROL_DB_SSH_ARGS[argument_index]:-}"
    elif [[ "$current_argument" == -L* ]]; then
      current_argument="${current_argument#-L}"
    fi
    if [[ "$current_argument" == "${local_port}:"* ]]; then
      forwarding_destination="${current_argument#"${local_port}:"}"
    fi
  done
  if [[ "$no_remote_command" != true || "$forwarding_destination" != "${dodb_private_ip}:22" ]]; then
    echo "OCI Bastion command has an invalid port-forwarding target or mode" >&2
    return 1
  fi
  for current_argument in "${DODB_CONTROL_DB_SSH_ARGS[@]}"; do
    if [[ "$current_argument" == *'<privateKey>'* || "$current_argument" == *'<localPort>'* ]]; then
      echo "OCI Bastion command placeholders were not fully replaced" >&2
      return 1
    fi
  done
  DODB_CONTROL_DB_SSH_ARGS+=(-o ExitOnForwardFailure=yes)
  tunnel_log="${temporary_dir}/bastion-tunnel.log"
  "${DODB_CONTROL_DB_SSH_ARGS[@]}" >"$tunnel_log" 2>&1 &
  DODB_CONTROL_DB_TUNNEL_PID=$!

  tunnel_ready=false
  for attempt in $(seq 1 60); do
    if ! kill -0 "$DODB_CONTROL_DB_TUNNEL_PID" >/dev/null 2>&1; then
      cat "$tunnel_log" >&2
      echo "Bastion SSH tunnel exited before becoming ready" >&2
      return 1
    fi
    if python3 -c 'import socket, sys; connection = socket.socket(); connection.settimeout(1); result = connection.connect_ex(("127.0.0.1", int(sys.argv[1]))); connection.close(); raise SystemExit(0 if result == 0 else 1)' "$local_port"; then
      tunnel_ready=true
      break
    fi
    sleep 1
  done
  if [[ "$tunnel_ready" != true ]]; then
    cat "$tunnel_log" >&2
    echo "Bastion tunnel did not accept connections on 127.0.0.1:$local_port" >&2
    return 1
  fi

  DODB_CONTROL_DB_KNOWN_HOSTS="${temporary_dir}/known_hosts"
  touch "$DODB_CONTROL_DB_KNOWN_HOSTS"
  chmod 600 "$DODB_CONTROL_DB_KNOWN_HOSTS"
  DODB_CONTROL_DB_SSH_OPTIONS=(
    -i "$private_key_file" -p "$local_port" -o IdentitiesOnly=yes
    -o BatchMode=yes -o ConnectTimeout=10
    -o ServerAliveInterval=15 -o ServerAliveCountMax=3
    -o "UserKnownHostsFile=$DODB_CONTROL_DB_KNOWN_HOSTS"
    -o StrictHostKeyChecking=accept-new
  )
  remote_nonce="$(python3 -c 'import secrets; print(secrets.token_hex(12))')"
  local_binary="${binary_dir}/fn0-db-ops"
  local_binary_sha="$(sha256sum "${local_binary}" | awk '{print $1}')"
  remote_cache_dir="/home/opc/.cache/fn0/db-ops/${local_binary_sha}"
  DODB_CONTROL_DB_REMOTE_BINARY="${remote_cache_dir}/fn0-db-ops"
  cache_status="$(ssh "${DODB_CONTROL_DB_SSH_OPTIONS[@]}" opc@127.0.0.1 bash -s -- "${DODB_CONTROL_DB_REMOTE_BINARY}" "${local_binary_sha}" <<'REMOTE_CACHE_CHECK'
set -euo pipefail
binary_path="$1"
expected_sha="$2"
command -v gzip >/dev/null 2>&1 || { echo "remote gzip is required for fn0-db-ops transport" >&2; exit 1; }
command -v sha256sum >/dev/null 2>&1 || { echo "remote sha256sum is required for fn0-db-ops verification" >&2; exit 1; }
if [[ -f "$binary_path" ]]; then
  actual_sha="$(sha256sum "$binary_path" | awk '{print $1}')"
  if [[ "$actual_sha" == "$expected_sha" ]]; then
    printf 'HIT %s\n' "$actual_sha"
    exit 0
  fi
fi
printf 'MISS\n'
REMOTE_CACHE_CHECK
)"
  if [[ "${cache_status}" != "HIT ${local_binary_sha}" ]]; then
    {
      printf '%s\n%s\n' "${local_binary_sha}" "${remote_nonce}"
      gzip -c "${local_binary}"
    } | ssh "${DODB_CONTROL_DB_SSH_OPTIONS[@]}" opc@127.0.0.1 'bash -c '\''
set -euo pipefail
IFS= read -r expected_sha
IFS= read -r upload_nonce
[[ "${expected_sha}" =~ ^[0-9a-f]{64}$ ]]
[[ "${upload_nonce}" =~ ^[0-9a-f]{24}$ ]]
cache_dir="/home/opc/.cache/fn0/db-ops/${expected_sha}"
binary_path="${cache_dir}/fn0-db-ops"
partial_path="${cache_dir}/fn0-db-ops.${upload_nonce}.partial"
umask 077
mkdir -p "${cache_dir}"
trap "rm -f -- ${partial_path}" EXIT
gzip -dc >"${partial_path}"
printf "%s  %s\n" "${expected_sha}" "${partial_path}" | sha256sum -c -
chmod 0700 "${partial_path}"
mv -f -- "${partial_path}" "${binary_path}"
trap - EXIT
'\''' || {
      echo "compressed SSH streaming upload of fn0-db-ops failed" >&2
      return 1
    }
    remote_sha="$(printf '%s\n' "${local_binary_sha}" | ssh "${DODB_CONTROL_DB_SSH_OPTIONS[@]}" opc@127.0.0.1 'bash -c '\''
set -euo pipefail
IFS= read -r expected_sha
[[ "${expected_sha}" =~ ^[0-9a-f]{64}$ ]]
sha256sum "/home/opc/.cache/fn0/db-ops/${expected_sha}/fn0-db-ops"
'\''' | awk '{print $1}')"
    if [[ "${remote_sha}" != "${local_binary_sha}" ]]; then
      echo "remote fn0-db-ops cache SHA-256 mismatch" >&2
      return 1
    fi
  fi
  __FN0_DODB_CONTROL_DB_OPEN=1
}

dodb_control_db_call() {
  dodb_control_db_open
  local command_name="$1"
  shift
  # shellcheck disable=SC2029
  ssh "${DODB_CONTROL_DB_SSH_OPTIONS[@]}" opc@127.0.0.1 \
    "$DODB_CONTROL_DB_REMOTE_BINARY" "$command_name" "$@"
}

dodb_control_db_close() {
  if [[ -n "${DODB_CONTROL_DB_TUNNEL_PID:-}" ]]; then
    kill "$DODB_CONTROL_DB_TUNNEL_PID" >/dev/null 2>&1 || true
    wait "$DODB_CONTROL_DB_TUNNEL_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "${DODB_CONTROL_DB_SESSION_ID:-}" ]]; then
    oci bastion session delete --session-id "$DODB_CONTROL_DB_SESSION_ID" --force >/dev/null 2>&1 || true
  fi
  if [[ -n "${DODB_CONTROL_DB_TEMP_DIR:-}" ]]; then
    rm -rf "$DODB_CONTROL_DB_TEMP_DIR"
  fi
  unset DODB_CONTROL_DB_REMOTE_BINARY DODB_CONTROL_DB_SESSION_ID DODB_CONTROL_DB_TUNNEL_PID
  unset DODB_CONTROL_DB_TEMP_DIR DODB_CONTROL_DB_KNOWN_HOSTS DODB_CONTROL_DB_SSH_ARGS
  unset DODB_CONTROL_DB_SSH_OPTIONS
  unset __FN0_DODB_CONTROL_DB_OPEN
}
