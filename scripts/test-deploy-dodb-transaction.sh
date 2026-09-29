#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
transaction_script="${REPO_ROOT}/scripts/lib/dodb-deploy-transaction.sh"
temporary_dir="$(mktemp -d)"
cleanup() {
  rm -rf "${temporary_dir}"
}
trap cleanup EXIT

create_mock_commands() {
  local mock_bin="$1"
  mkdir -p "${mock_bin}"
  cat >"${mock_bin}/systemctl" <<'MOCK_SYSTEMCTL'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  is-active)
    [[ "$(cat "${DODB_TEST_STATE_FILE}")" == active ]]
    ;;
  restart)
    restart_count="$(cat "${DODB_TEST_RESTART_COUNT_FILE}")"
    printf '%s\n' "$((restart_count + 1))" >"${DODB_TEST_RESTART_COUNT_FILE}"
    binary_contents="$(cat "${DODB_BINARY_PATH}")"
    if [[ "${binary_contents}" == new && "${DODB_TEST_FAIL_NEW_RESTART:-0}" == 1 ]]; then
      printf 'inactive\n' >"${DODB_TEST_STATE_FILE}"
    else
      printf 'active\n' >"${DODB_TEST_STATE_FILE}"
    fi
    ;;
  *)
    exit 64
    ;;
esac
MOCK_SYSTEMCTL
  cat >"${mock_bin}/ss" <<'MOCK_SS'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$(cat "${DODB_TEST_STATE_FILE}")" == active ]]; then
  printf 'UNCONN 0 0 0.0.0.0:%s 0.0.0.0:*\n' "${DODB_UDP_PORT}"
fi
MOCK_SS
  chmod 0755 "${mock_bin}/systemctl" "${mock_bin}/ss"
}

run_success_case() {
  local case_dir="${temporary_dir}/success"
  local mock_bin="${case_dir}/bin"
  mkdir -p "${case_dir}/usr/local/bin"
  printf 'old\n' >"${case_dir}/usr/local/bin/dodb-server"
  printf 'new\n' >"${case_dir}/dodb-server.new"
  chmod 0755 "${case_dir}/usr/local/bin/dodb-server" "${case_dir}/dodb-server.new"
  printf 'active\n' >"${case_dir}/state"
  printf '0\n' >"${case_dir}/restart-count"
  create_mock_commands "${mock_bin}"
  PATH="${mock_bin}:${PATH}" \
    DODB_BINARY_PATH="${case_dir}/usr/local/bin/dodb-server" \
    DODB_CANDIDATE_PATH="${case_dir}/dodb-server.new" \
    DODB_BACKUP_DIR="${case_dir}/backups" \
    DODB_SERVICE_WAIT_SECONDS=1 \
    DODB_INSTALL_OWNER="$(id -un):$(id -gn)" \
    DODB_TEST_STATE_FILE="${case_dir}/state" \
    DODB_TEST_RESTART_COUNT_FILE="${case_dir}/restart-count" \
    DODB_UDP_PORT=18445 \
    bash "${transaction_script}"
  [[ "$(cat "${case_dir}/usr/local/bin/dodb-server")" == new ]]
  [[ "$(cat "${case_dir}/restart-count")" == 1 ]]
  [[ "$(cat "${case_dir}/state")" == active ]]
  [[ -f "$(find "${case_dir}/backups" -type f -print -quit)" ]]
}

run_rollback_case() {
  local case_dir="${temporary_dir}/rollback"
  local mock_bin="${case_dir}/bin"
  mkdir -p "${case_dir}/usr/local/bin"
  printf 'old\n' >"${case_dir}/usr/local/bin/dodb-server"
  printf 'new\n' >"${case_dir}/dodb-server.new"
  chmod 0755 "${case_dir}/usr/local/bin/dodb-server" "${case_dir}/dodb-server.new"
  printf 'active\n' >"${case_dir}/state"
  printf '0\n' >"${case_dir}/restart-count"
  create_mock_commands "${mock_bin}"
  if PATH="${mock_bin}:${PATH}" \
    DODB_BINARY_PATH="${case_dir}/usr/local/bin/dodb-server" \
    DODB_CANDIDATE_PATH="${case_dir}/dodb-server.new" \
    DODB_BACKUP_DIR="${case_dir}/backups" \
    DODB_SERVICE_WAIT_SECONDS=1 \
    DODB_INSTALL_OWNER="$(id -un):$(id -gn)" \
    DODB_TEST_STATE_FILE="${case_dir}/state" \
    DODB_TEST_RESTART_COUNT_FILE="${case_dir}/restart-count" \
    DODB_TEST_FAIL_NEW_RESTART=1 \
    DODB_UDP_PORT=18445 \
    bash "${transaction_script}"; then
    echo "expected the failed candidate deployment to return nonzero" >&2
    exit 1
  fi
  [[ "$(cat "${case_dir}/usr/local/bin/dodb-server")" == old ]]
  [[ "$(cat "${case_dir}/restart-count")" == 2 ]]
  [[ "$(cat "${case_dir}/state")" == active ]]
  [[ -f "$(find "${case_dir}/backups" -type f -print -quit)" ]]
}

run_success_case
run_rollback_case
echo "dodb deployment transaction success and rollback tests passed"
