#!/usr/bin/env bash
set -euo pipefail

binary_path="${DODB_BINARY_PATH:-/usr/local/bin/dodb-server}"
candidate_path="${DODB_CANDIDATE_PATH:-/tmp/dodb-server.new}"
backup_dir="${DODB_BACKUP_DIR:-/var/backups/dodb-server}"
service_name="${DODB_SERVICE_NAME:-dodb.service}"
udp_port="${DODB_UDP_PORT:-18445}"
wait_seconds="${DODB_SERVICE_WAIT_SECONDS:-30}"
install_owner="${DODB_INSTALL_OWNER:-root:root}"

if [[ ! "${wait_seconds}" =~ ^[1-9][0-9]*$ || "${wait_seconds}" -gt 300 ]]; then
  echo "DODB_SERVICE_WAIT_SECONDS must be between 1 and 300" >&2
  exit 2
fi
if [[ ! "${udp_port}" =~ ^[1-9][0-9]*$ || "${udp_port}" -gt 65535 ]]; then
  echo "DODB_UDP_PORT must be between 1 and 65535" >&2
  exit 2
fi
if [[ "${install_owner}" != *:* ]]; then
  echo "DODB_INSTALL_OWNER must be user:group" >&2
  exit 2
fi
if [[ ! -x "${binary_path}" ]]; then
  echo "current dodb-server binary is missing or not executable: ${binary_path}" >&2
  exit 1
fi
if [[ ! -s "${candidate_path}" ]]; then
  echo "new dodb-server binary is missing or empty: ${candidate_path}" >&2
  exit 1
fi

owner_user="${install_owner%%:*}"
owner_group="${install_owner#*:}"
old_sha256="$(sha256sum "${binary_path}" | awk '{print $1}')"
new_sha256="$(sha256sum "${candidate_path}" | awk '{print $1}')"
timestamp="$(date -u '+%Y%m%dT%H%M%SZ')"
install -d -o "${owner_user}" -g "${owner_group}" -m 0700 "${backup_dir}"
backup_path="$(mktemp "${backup_dir}/dodb-server.${timestamp}.XXXXXX")"
install -o "${owner_user}" -g "${owner_group}" -m 0700 "${binary_path}" "${backup_path}"
backup_sha256="$(sha256sum "${backup_path}" | awk '{print $1}')"
if [[ "${backup_sha256}" != "${old_sha256}" ]]; then
  echo "backup SHA-256 does not match the current production binary" >&2
  exit 1
fi
echo "current dodb-server SHA-256: ${old_sha256}"
echo "candidate dodb-server SHA-256: ${new_sha256}"
echo "rollback backup: ${backup_path}"

replacement_stage=""
replacement_started=0

wait_for_service() {
  local attempt
  for ((attempt = 0; attempt < wait_seconds; attempt += 1)); do
    if systemctl is-active --quiet "${service_name}" && \
      ss -H -lun 2>/dev/null | awk -v port=":${udp_port}" '$4 ~ (port "$" ) { found = 1 } END { exit found ? 0 : 1 }'; then
      return 0
    fi
    sleep 1
  done
  return 1
}

rollback_on_failure() {
  local exit_status=$?
  local rollback_stage
  trap - EXIT
  if [[ "${exit_status}" -ne 0 && "${replacement_started}" -eq 1 ]]; then
    set +e
    rollback_stage="$(mktemp "${binary_path}.rollback.XXXXXX")"
    if install -o "${owner_user}" -g "${owner_group}" -m 0755 "${backup_path}" "${rollback_stage}" && \
      mv -f "${rollback_stage}" "${binary_path}" && \
      systemctl restart "${service_name}" && \
      wait_for_service; then
      echo "dodb-server rollback verified; restored SHA-256: ${old_sha256}" >&2
      echo "rollback backup retained: ${backup_path}" >&2
    else
      echo "dodb-server rollback verification failed; backup retained: ${backup_path}" >&2
      exit_status=1
    fi
    rm -f "${rollback_stage}" "${replacement_stage}" "${candidate_path}"
  else
    rm -f "${replacement_stage}" "${candidate_path}"
  fi
  exit "${exit_status}"
}
trap rollback_on_failure EXIT

replacement_stage="$(mktemp "${binary_path}.new.XXXXXX")"
install -o "${owner_user}" -g "${owner_group}" -m 0755 "${candidate_path}" "${replacement_stage}"
replacement_started=1
mv -f "${replacement_stage}" "${binary_path}"
replacement_stage=""

if ! systemctl restart "${service_name}"; then
  echo "dodb.service restart failed" >&2
  exit 1
fi
if ! wait_for_service; then
  echo "dodb.service did not become active with UDP ${udp_port} listening within ${wait_seconds}s" >&2
  exit 1
fi

installed_sha256="$(sha256sum "${binary_path}" | awk '{print $1}')"
if [[ "${installed_sha256}" != "${new_sha256}" ]]; then
  echo "installed dodb-server SHA-256 does not match the candidate" >&2
  exit 1
fi

replacement_started=0
echo "current dodb-server SHA-256: ${old_sha256}"
echo "new dodb-server SHA-256: ${new_sha256}"
echo "rollback backup: ${backup_path}"
echo "dodb.service active and UDP ${udp_port} listening"
