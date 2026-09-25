#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
temporary_dir="$(mktemp -d)"
trap 'rm -rf "$temporary_dir"' EXIT

remote_binary="${temporary_dir}/fn0-db-ops"
capture_file="${temporary_dir}/arguments"
cat >"$remote_binary" <<'REMOTE_BINARY'
#!/usr/bin/env bash
printf '%s\0' "$@" >"$CAPTURE_FILE"
REMOTE_BINARY
chmod +x "$remote_binary"

export REPO_ROOT CAPTURE_FILE="$capture_file"
source "${REPO_ROOT}/scripts/lib/dodb-control-db.sh"
DODB_CONTROL_DB_REMOTE_BINARY="$remote_binary"
DODB_CONTROL_DB_REMOTE_CERT_PATH="${temporary_dir}/server.der"
DODB_CONTROL_DB_SSH_OPTIONS=(-o "test=value")
dodb_control_db_open() { :; }
ssh() {
  local remote_command=""
  for argument in "$@"; do remote_command="$argument"; done
  bash -c "$remote_command"
}

project_pk="ProjectDoc/project_id=00000001"
dodb_control_db_call get-observed --pk "$project_pk" --sk ""
python3 - "$capture_file" "$temporary_dir/server.der" "$project_pk" <<'PY'
import pathlib
import sys

arguments = pathlib.Path(sys.argv[1]).read_bytes().split(b"\0")[:-1]
expected = [b"--dodb-root-cert", sys.argv[2].encode(), b"get-observed", b"--pk", sys.argv[3].encode(), b"--sk", b""]
if arguments != expected:
    raise SystemExit(f"get-observed arguments changed: {arguments!r}")
PY

dodb_control_db_call put --pk "$project_pk" --sk ""
python3 - "$capture_file" "$temporary_dir/server.der" "$project_pk" <<'PY'
import pathlib
import sys

arguments = pathlib.Path(sys.argv[1]).read_bytes().split(b"\0")[:-1]
expected = [b"--dodb-root-cert", sys.argv[2].encode(), b"put", b"--pk", sys.argv[3].encode(), b"--sk", b""]
if arguments != expected:
    raise SystemExit(f"put arguments changed: {arguments!r}")
PY

printf '%s\n' 'dodb control database SSH argument tests passed'
