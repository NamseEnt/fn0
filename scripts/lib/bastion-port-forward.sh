#!/usr/bin/env bash

if [[ -n "${__FN0_BASTION_PORT_FORWARD_LOADED:-}" ]]; then
  return 0
fi
__FN0_BASTION_PORT_FORWARD_LOADED=1

bastion_port_forward_open() {
  if [[ "$#" -ne 4 ]]; then
    echo "bastion_port_forward_open expects purpose, bastion ID, target IP, and target port" >&2
    return 2
  fi

  local purpose="$1"
  local bastion_id="$2"
  local target_private_ip="$3"
  local target_port="$4"
  local public_key_file create_response create_status work_request_id work_request_json
  local session_list_json matching_sessions session_json session_display_name session_state
  local session_ssh_command ssh_args_file session_argument current_argument forwarding_destination
  local no_remote_command argument_index attempt discovery_seconds discovery_interval
  local ssh_destination_index bastion_authenticated tunnel_tcp_ready
  local -a first_hop_options=()

  if [[ -n "${BASTION_SESSION_ID:-}" || -n "${BASTION_TUNNEL_PID:-}" ]]; then
    echo "Bastion port forward is already open" >&2
    return 2
  fi
  if [[ ! "$purpose" =~ ^[a-z0-9-]+$ || ! "$target_private_ip" =~ ^[0-9a-fA-F:.]+$ || ! "$target_port" =~ ^[0-9]+$ ]]; then
    echo "invalid Bastion purpose or target" >&2
    return 2
  fi

  BASTION_DISPLAY_NAME="fn0-dodb-${purpose}-$(python3 -c 'import secrets; print(secrets.token_hex(12))')"
  BASTION_SESSION_ID=""
  BASTION_TUNNEL_PID=""
  BASTION_LOCAL_PORT=""
  BASTION_SSH_ARGS=()
  BASTION_TEMP_DIR="$(mktemp -d)"
  chmod 700 "$BASTION_TEMP_DIR"
  local bastion_session_private_key_file="${BASTION_TEMP_DIR}/bastion-session-key"
  public_key_file="${bastion_session_private_key_file}.pub"
  ssh-keygen -q -t rsa -b 3072 -N '' -f "$bastion_session_private_key_file"
  chmod 600 "$bastion_session_private_key_file"
  chmod 600 "$public_key_file"
  BASTION_SESSION_KEY_FINGERPRINT="$(ssh-keygen -lf "$public_key_file" | awk '{print $2}')"

  create_status=0
  if create_response="$(oci bastion session create-port-forwarding \
    --bastion-id "$bastion_id" \
    --target-private-ip "$target_private_ip" \
    --target-port "$target_port" \
    --ssh-public-key-file "$public_key_file" \
    --session-ttl 3600 \
    --display-name "$BASTION_DISPLAY_NAME" \
    --wait-for-state SUCCEEDED \
    --max-wait-seconds 300 \
    --output json)"; then
    create_status=0
  else
    create_status=$?
    echo "OCI Bastion create returned status ${create_status}; checking for the uniquely named session" >&2
  fi

  if [[ "$create_status" -eq 0 ]]; then
    work_request_id="$(jq -r '.data.id // empty' <<<"$create_response" 2>/dev/null || true)"
    if [[ -n "$work_request_id" ]]; then
      if work_request_json="$(oci bastion work-request get --work-request-id "$work_request_id" --output json 2>/dev/null)"; then
        BASTION_SESSION_ID="$(jq -r '.data.resources[]? | select((."entity-type" == "SessionResource") or (."entityType" == "SessionResource")) | .identifier // empty' <<<"$work_request_json" | head -n 1)"
      fi
    fi
  fi

  discovery_seconds="${BASTION_DISCOVERY_SECONDS:-300}"
  if [[ ! "$discovery_seconds" =~ ^[0-9]+$ || "$discovery_seconds" -lt 1 || "$discovery_seconds" -gt 300 ]]; then
    echo "invalid Bastion discovery bound" >&2
    bastion_port_forward_close
    return 2
  fi
  discovery_interval="${BASTION_DISCOVERY_INTERVAL_SECONDS:-1}"
  if [[ ! "$discovery_interval" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo "invalid Bastion discovery polling interval" >&2
    bastion_port_forward_close
    return 2
  fi
  attempt=0
  while [[ -z "$BASTION_SESSION_ID" && "$attempt" -lt "$discovery_seconds" ]]; do
    if session_list_json="$(oci bastion session list --bastion-id "$bastion_id" --display-name "$BASTION_DISPLAY_NAME" --all --output json 2>/dev/null)"; then
      matching_sessions="$(jq -c --arg display_name "$BASTION_DISPLAY_NAME" '[.data[]? | select(."display-name" == $display_name or .displayName == $display_name)]' <<<"$session_list_json" 2>/dev/null || printf '[]')"
      case "$(jq 'length' <<<"$matching_sessions")" in
        0) ;;
        1) BASTION_SESSION_ID="$(jq -r '.[0].id // empty' <<<"$matching_sessions")" ;;
        *)
          echo "multiple OCI Bastion sessions matched unique display name ${BASTION_DISPLAY_NAME}" >&2
          while IFS= read -r session_id_to_delete; do
            [[ -n "$session_id_to_delete" ]] || continue
            oci bastion session delete --session-id "$session_id_to_delete" --force >/dev/null 2>&1 || true
          done < <(jq -r '.[].id // empty' <<<"$matching_sessions")
          bastion_port_forward_close
          return 1
          ;;
      esac
      if [[ -n "$BASTION_SESSION_ID" ]]; then break; fi
    fi
    attempt=$((attempt + 1))
    if [[ "$attempt" -lt "$discovery_seconds" ]]; then sleep "$discovery_interval"; fi
  done
  if [[ -z "$BASTION_SESSION_ID" ]]; then
    echo "OCI Bastion session was not discovered within ${discovery_seconds} seconds (create status ${create_status})" >&2
    bastion_port_forward_close
    return 1
  fi

  session_json="$(oci bastion session get --session-id "$BASTION_SESSION_ID" --output json)" || {
    bastion_port_forward_close
    return 1
  }
  session_display_name="$(jq -r '.data."display-name" // .data.displayName // empty' <<<"$session_json")"
  if [[ "$session_display_name" != "$BASTION_DISPLAY_NAME" ]]; then
    echo "discovered Bastion session display name does not match the generated name" >&2
    bastion_port_forward_close
    return 1
  fi

  session_state=""
  attempt=0
  while [[ "$attempt" -lt 60 ]]; do
    session_state="$(jq -r '.data."lifecycle-state" // .data.lifecycleState // empty' <<<"$session_json")"
    if [[ "$session_state" == "ACTIVE" ]]; then break; fi
    if [[ "$session_state" == "FAILED" || "$session_state" == "DELETED" ]]; then
      echo "OCI Bastion session entered ${session_state}" >&2
      bastion_port_forward_close
      return 1
    fi
    attempt=$((attempt + 1))
    if [[ "$attempt" -lt 60 ]]; then sleep 5; fi
    session_json="$(oci bastion session get --session-id "$BASTION_SESSION_ID" --output json)" || {
      bastion_port_forward_close
      return 1
    }
  done
  if [[ "$session_state" != "ACTIVE" ]]; then
    session_state="$(jq -r '.data."lifecycle-state" // .data.lifecycleState // empty' <<<"$session_json")"
  fi
  if [[ "$session_state" != "ACTIVE" ]]; then
    echo "OCI Bastion session did not become ACTIVE; current state=${session_state:-unknown}" >&2
    bastion_port_forward_close
    return 1
  fi
  BASTION_ACTIVE_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  printf 'Bastion session active: id=%s display_name=%s at=%s ephemeral_key_fingerprint=%s\n' \
    "$BASTION_SESSION_ID" "$BASTION_DISPLAY_NAME" "$BASTION_ACTIVE_AT" "$BASTION_SESSION_KEY_FINGERPRINT" >&2
  session_ssh_command="$(jq -r '.data."ssh-metadata".command // .data.sshMetadata.command // empty' <<<"$session_json")"
  if [[ -z "$session_ssh_command" ]]; then
    echo "OCI Bastion session has no ssh-metadata.command" >&2
    bastion_port_forward_close
    return 1
  fi

  BASTION_LOCAL_PORT="$(python3 -c 'import socket; listener = socket.socket(); listener.bind(("127.0.0.1", 0)); print(listener.getsockname()[1]); listener.close()')"
  if [[ ! "$BASTION_LOCAL_PORT" =~ ^[0-9]+$ || "$BASTION_LOCAL_PORT" -lt 1 || "$BASTION_LOCAL_PORT" -gt 65535 ]]; then
    echo "could not choose a valid local tunnel port: ${BASTION_LOCAL_PORT}" >&2
    bastion_port_forward_close
    return 1
  fi
  ssh_args_file="${BASTION_TEMP_DIR}/bastion-ssh-args"
  python3 -c '
import shlex
import sys
arguments = shlex.split(sys.argv[1])
for argument_index, argument in enumerate(arguments):
    arguments[argument_index] = argument.replace("<privateKey>", sys.argv[2]).replace("<localPort>", sys.argv[3])
for argument in arguments:
    sys.stdout.buffer.write(argument.encode() + b"\0")
' "$session_ssh_command" "$bastion_session_private_key_file" "$BASTION_LOCAL_PORT" >"$ssh_args_file"
  while IFS= read -r -d '' session_argument; do BASTION_SSH_ARGS+=("$session_argument"); done <"$ssh_args_file"
  if [[ "${#BASTION_SSH_ARGS[@]}" -lt 2 || "${BASTION_SSH_ARGS[0]}" != ssh ]]; then
    echo "OCI Bastion returned an unsupported port-forwarding command" >&2
    bastion_port_forward_close
    return 1
  fi
  forwarding_destination=""
  no_remote_command=false
  for ((argument_index = 0; argument_index < ${#BASTION_SSH_ARGS[@]}; argument_index += 1)); do
    current_argument="${BASTION_SSH_ARGS[argument_index]}"
    if [[ "$current_argument" == -N ]]; then no_remote_command=true; fi
    if [[ "$current_argument" == -L ]]; then
      argument_index=$((argument_index + 1))
      current_argument="${BASTION_SSH_ARGS[argument_index]:-}"
    elif [[ "$current_argument" == -L* ]]; then
      current_argument="${current_argument#-L}"
    fi
    if [[ "$current_argument" == "${BASTION_LOCAL_PORT}:"* ]]; then
      forwarding_destination="${current_argument#"${BASTION_LOCAL_PORT}:"}"
    fi
  done
  if [[ "$no_remote_command" != true || "$forwarding_destination" != "${target_private_ip}:${target_port}" ]]; then
    echo "OCI Bastion command has an invalid port-forwarding target or mode" >&2
    bastion_port_forward_close
    return 1
  fi
  for current_argument in "${BASTION_SSH_ARGS[@]}"; do
    if [[ "$current_argument" == *'<privateKey>'* || "$current_argument" == *'<localPort>'* ]]; then
      echo "OCI Bastion command placeholders were not fully replaced" >&2
      bastion_port_forward_close
      return 1
    fi
  done

  ssh_destination_index=-1
  for ((argument_index = 0; argument_index < ${#BASTION_SSH_ARGS[@]}; argument_index += 1)); do
    if [[ "${BASTION_SSH_ARGS[argument_index]}" == *@* ]]; then ssh_destination_index="$argument_index"; fi
  done
  if [[ "$ssh_destination_index" -lt 1 ]]; then
    echo "OCI Bastion SSH command has no session destination" >&2
    bastion_port_forward_close
    return 1
  fi
  first_hop_options=(
    -v
    -o HostKeyAlgorithms=+ssh-rsa
    -o PubkeyAcceptedAlgorithms=+ssh-rsa
    -o IdentitiesOnly=yes
    -o BatchMode=yes
    -o ConnectTimeout=10
    -o ServerAliveInterval=15
    -o ServerAliveCountMax=3
    -o ExitOnForwardFailure=yes
  )
  BASTION_SSH_ARGS=(
    "${BASTION_SSH_ARGS[@]:0:ssh_destination_index}"
    "${first_hop_options[@]}"
    "${BASTION_SSH_ARGS[@]:ssh_destination_index}"
  )
  local known_hosts_file="${BASTION_TEMP_DIR}/known_hosts"
  local tunnel_log="${BASTION_TEMP_DIR}/bastion-tunnel.log"
  touch "$known_hosts_file"
  chmod 600 "$known_hosts_file"
  BASTION_SSH_ATTEMPT_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  printf 'Bastion SSH attempt: session_id=%s at=%s\n' "$BASTION_SESSION_ID" "$BASTION_SSH_ATTEMPT_AT" >&2
  "${BASTION_SSH_ARGS[@]}" >"$tunnel_log" 2>&1 &
  BASTION_TUNNEL_PID=$!
  bastion_authenticated=false
  tunnel_tcp_ready=false
  for attempt in $(seq 1 60); do
    if ! kill -0 "$BASTION_TUNNEL_PID" >/dev/null 2>&1; then
      grep -E 'Offering public key:|Server accepts key:|Authentications that can continue:|sign_and_send_pubkey|Authenticated to |Permission denied|Local forwarding listening' "$tunnel_log" >&2 || true
      echo "Bastion port-forwarding tunnel exited before becoming ready" >&2
      bastion_port_forward_close
      return 1
    fi
    if grep -q 'Authenticated to ' "$tunnel_log"; then bastion_authenticated=true; fi
    if python3 -c 'import socket, sys; connection = socket.socket(); connection.settimeout(1); result = connection.connect_ex(("127.0.0.1", int(sys.argv[1]))); connection.close(); raise SystemExit(0 if result == 0 else 1)' "$BASTION_LOCAL_PORT"; then
      tunnel_tcp_ready=true
    fi
    if [[ "$bastion_authenticated" == true && "$tunnel_tcp_ready" == true ]]; then
      printf 'Bastion first-hop authentication and local TCP forwarding passed for session %s\n' "$BASTION_SESSION_ID" >&2
      return 0
    fi
    sleep 1
  done
  grep -E 'Offering public key:|Server accepts key:|Authentications that can continue:|sign_and_send_pubkey|Authenticated to |Permission denied|Local forwarding listening' "$tunnel_log" >&2 || true
  echo "Bastion tunnel did not accept connections on 127.0.0.1:${BASTION_LOCAL_PORT}" >&2
  bastion_port_forward_close
  return 1
}

bastion_port_forward_close() {
  if [[ -n "${BASTION_TUNNEL_PID:-}" ]]; then
    kill "$BASTION_TUNNEL_PID" >/dev/null 2>&1 || true
    wait "$BASTION_TUNNEL_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "${BASTION_SESSION_ID:-}" ]]; then
    oci bastion session delete --session-id "$BASTION_SESSION_ID" --force >/dev/null 2>&1 || true
  fi
  if [[ -n "${BASTION_TEMP_DIR:-}" ]]; then rm -rf "$BASTION_TEMP_DIR"; fi
  unset BASTION_SESSION_ID BASTION_TUNNEL_PID BASTION_LOCAL_PORT
  unset BASTION_DISPLAY_NAME BASTION_TEMP_DIR BASTION_SSH_ARGS
  unset BASTION_SESSION_KEY_FINGERPRINT
  unset BASTION_ACTIVE_AT BASTION_SSH_ATTEMPT_AT
}
