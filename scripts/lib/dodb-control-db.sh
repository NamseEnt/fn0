# shellcheck shell=bash

if [[ -n "${__FN0_DODB_CONTROL_DB_LOADED:-}" ]]; then
  return 0
fi
__FN0_DODB_CONTROL_DB_LOADED=1
unset __FN0_DODB_CONTROL_DB_OPEN
source "${REPO_ROOT}/scripts/lib/bastion-port-forward.sh"

dodb_control_db_open() {
  if [[ -n "${__FN0_DODB_CONTROL_DB_OPEN:-}" ]]; then
    return 0
  fi

  local temporary_dir binary_dir bastion_session_key_file target_ssh_key_file
  local remote_nonce local_binary local_binary_sha remote_cache_dir cache_status remote_sha

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

  dodb_private_ip="$(pulumi_pick dodbPrivateIp)"
  bastion_id="$(pulumi_pick workerBastionId)"
  if [[ -z "$dodb_private_ip" || -z "$bastion_id" ]]; then
    echo "missing Pulumi outputs: dodbPrivateIp / workerBastionId" >&2
    return 1
  fi

  binary_dir="${temporary_dir}/bin"
  mkdir -p "$binary_dir"
  "${REPO_ROOT}/scripts/build-rust-linux-arm64-bin.sh" fn0-db-ops "$binary_dir" >&2

  bastion_session_key_file="${temporary_dir}/bastion-session-key"
  target_ssh_key_file="${temporary_dir}/target-ssh-key"
  pulumi_pick workerSshPrivateKey >"$bastion_session_key_file"
  pulumi_pick workerSshPrivateKey >"$target_ssh_key_file"
  chmod 600 "$bastion_session_key_file" "$target_ssh_key_file"

  bastion_port_forward_open control "$bastion_id" "$dodb_private_ip" 22 "$bastion_session_key_file" "$target_ssh_key_file"
  DODB_CONTROL_DB_SSH_OPTIONS=()
  while IFS= read -r -d '' ssh_option; do DODB_CONTROL_DB_SSH_OPTIONS+=("$ssh_option"); done < <(bastion_port_forward_ssh_options)
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
  bastion_port_forward_close
  if [[ -n "${DODB_CONTROL_DB_TEMP_DIR:-}" ]]; then
    rm -rf "$DODB_CONTROL_DB_TEMP_DIR"
  fi
  unset DODB_CONTROL_DB_REMOTE_BINARY
  unset DODB_CONTROL_DB_TEMP_DIR
  unset __FN0_DODB_CONTROL_DB_OPEN
}
