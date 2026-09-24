#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
temporary_dir="$(mktemp -d)"
trap 'rm -rf "${temporary_dir}"' EXIT
mock_bin="${temporary_dir}/bin"
mock_repo="${temporary_dir}/repo"
state_dir="${temporary_dir}/state"
mkdir -p "${mock_bin}" "${mock_repo}/scripts/lib" "${mock_repo}/infra/cloud" "${state_dir}"
cp "${repo_root}/scripts/run-dodb-migration.sh" "${mock_repo}/scripts/run-dodb-migration.sh"
cp "${repo_root}/scripts/lib/pulumi-outputs.sh" "${mock_repo}/scripts/lib/pulumi-outputs.sh"
cp "${repo_root}/scripts/lib/bastion-port-forward.sh" "${mock_repo}/scripts/lib/bastion-port-forward.sh"
cp "${repo_root}/scripts/lib/dodb-control-db.sh" "${mock_repo}/scripts/lib/dodb-control-db.sh"
cat >"${mock_repo}/scripts/build-rust-linux-arm64-bin.sh" <<'BUILD'
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$2"
printf 'fake-arm64-binary-%s' "$1" >"$2/$1"
BUILD
chmod +x "${mock_repo}/scripts/build-rust-linux-arm64-bin.sh"

cat >"${mock_bin}/pulumi" <<'PULUMI'
#!/usr/bin/env bash
set -euo pipefail
name="${3:-}"
printf '%s\n' "$name" >>"${MOCK_STATE}/pulumi-outputs"
case "$name" in
  forteDbGroupToken) printf '%s' 'test-turso-token' ;;
  forteDbHostSuffix) printf '%s' 'example.test' ;;
  workerBastionId) printf '%s' 'bastion-test' ;;
  workerSshPrivateKey) printf '%s' 'test-private-key' ;;
  dodbPrivateIp) printf '%s' '10.0.0.7' ;;
  *) exit 3 ;;
esac
PULUMI

cat >"${mock_bin}/cargo" <<'CARGO'
#!/usr/bin/env bash
set -euo pipefail
python3 - "$@" <<'PY'
import json
import os
import sys
with open(os.path.join(os.environ["MOCK_STATE"], "cargo.json"), "w", encoding="utf-8") as output:
    json.dump({"args": sys.argv[1:], "token": os.environ.get("TURSO_GROUP_TOKEN"), "suffix": os.environ.get("TURSO_DB_HOST_SUFFIX")}, output)
print('{"projects":[],"rows":0,"bytes":0}')
PY
CARGO

cat >"${mock_bin}/oci" <<'OCI'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${MOCK_STATE}/oci-calls"
case "$*" in
  *"bastion session create-port-forwarding"*)
    display_name=""
    public_key_file=""
    while [[ "$#" -gt 0 ]]; do
      if [[ "$1" == --display-name ]]; then display_name="$2"; fi
      if [[ "$1" == --ssh-public-key-file ]]; then public_key_file="$2"; fi
      shift
    done
    create_count=0
    [[ ! -e "${MOCK_STATE}/create-count" ]] || create_count="$(cat "${MOCK_STATE}/create-count")"
    create_count=$((create_count + 1))
    printf '%s' "$create_count" >"${MOCK_STATE}/create-count"
    printf '%s\n' "$public_key_file" >"${MOCK_STATE}/ephemeral-public-path-${create_count}"
    cat "$public_key_file" >"${MOCK_STATE}/ephemeral-public-${create_count}"
    printf '%s' "$display_name" >"${MOCK_STATE}/display-name"
    case "${MOCK_CREATE_MODE:-work-request}" in
      work-request) printf '%s\n' '{"data":{"id":"work-1"}}' ;;
      missing) printf '%s\n' '{"data":{}}' ;;
      nonzero-created) printf '%s\n' '{"data":{}}'; exit 9 ;;
      none|none-nonzero) printf '%s\n' '{"data":{}}'; if [[ "${MOCK_CREATE_MODE}" == none-nonzero ]]; then exit 9; fi ;;
      delayed) printf '%s\n' '{"data":{}}' ;;
      multiple) printf '%s\n' '{"data":{}}' ;;
      *) printf '%s\n' '{"data":{}}' ;;
    esac
    ;;
  *"bastion work-request get"*) printf '%s\n' '{"data":{"resources":[{"entity-type":"SessionResource","identifier":"session-1"}]}}' ;;
  *"bastion session list"*)
    list_count=0
    [[ ! -e "${MOCK_STATE}/list-count" ]] || list_count="$(cat "${MOCK_STATE}/list-count")"
    list_count=$((list_count + 1))
    printf '%s' "$list_count" >"${MOCK_STATE}/list-count"
    case "${MOCK_CREATE_MODE:-work-request}" in
      none|none-nonzero) printf '%s\n' '{"data":[]}' ;;
      delayed)
        if [[ "$list_count" -lt 3 ]]; then printf '%s\n' '{"data":[]}'; else
          printf '{"data":[{"id":"session-1","display-name":"%s"}]}\n' "$(cat "${MOCK_STATE}/display-name")"
        fi
        ;;
      multiple)
        printf '{"data":[{"id":"session-1","display-name":"%s"},{"id":"session-2","display-name":"%s"}]}\n' "$(cat "${MOCK_STATE}/display-name")" "$(cat "${MOCK_STATE}/display-name")"
        ;;
      *) printf '{"data":[{"id":"session-1","display-name":"%s"}]}\n' "$(cat "${MOCK_STATE}/display-name")" ;;
    esac
    ;;
  *"bastion session get"*)
    display_name="$(cat "${MOCK_STATE}/display-name")"
    state="${MOCK_SESSION_STATE:-ACTIVE}"
    if [[ "${MOCK_NO_METADATA:-0}" == 1 ]]; then
      printf '{"data":{"id":"session-1","display-name":"%s","lifecycle-state":"%s"}}\n' "$display_name" "$state"
    else
      printf '{"data":{"id":"session-1","display-name":"%s","lifecycle-state":"%s","ssh-metadata":{"command":"ssh -N -L <localPort>:10.0.0.7:22 -i <privateKey> opc@bastion"}}}\n' "$display_name" "$state"
    fi
    ;;
  *"bastion session delete"*) printf '%s\n' "$*" >>"${MOCK_STATE}/session-deletes" ;;
  *) ;;
esac
OCI
cat >"${mock_bin}/ssh-keygen" <<'SSH_KEYGEN'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
  -q)
    private_key_file=""
    while [[ "$#" -gt 0 ]]; do
      if [[ "$1" == -f ]]; then private_key_file="$2"; fi
      shift
    done
    nonce="$(python3 -c 'import secrets; print(secrets.token_hex(24))')"
    printf 'fake-rsa-private-%s\n' "$nonce" >"$private_key_file"
    printf 'ssh-rsa fake-rsa-public-%s\n' "$nonce" >"${private_key_file}.pub"
    chmod 600 "$private_key_file"
    ;;
  -y)
    private_key_file="$3"
    cat "${private_key_file}.pub"
    ;;
  -lf)
    fingerprint="$(python3 - "$2" <<'PY'
import hashlib
import sys
print("SHA256:" + hashlib.sha256(open(sys.argv[1], "rb").read()).hexdigest())
PY
)"
    printf '3072 %s fake\n' "$fingerprint"
    ;;
  *) exit 2 ;;
esac
SSH_KEYGEN

cat >"${mock_bin}/ssh" <<'SSH'
#!/usr/bin/env python3
import gzip
import hashlib
import os
import shlex
import socket
import sys

arguments = sys.argv[1:]
state_dir = os.environ["MOCK_STATE"]
with open(os.path.join(state_dir, "ssh-argv"), "a", encoding="utf-8") as output:
    output.write(" ".join(arguments) + "\n")
if "-N" in arguments:
    print("Authenticated to mock-bastion using publickey", file=sys.stderr)
    forward_index = arguments.index("-L")
    local_port = int(arguments[forward_index + 1].split(":", 1)[0])
    server = socket.socket()
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind(("127.0.0.1", local_port))
    server.listen()
    while True:
        connection, _ = server.accept()
        connection.close()

destination_index = arguments.index("opc@127.0.0.1")
remote = arguments[destination_index + 1:]
if len(remote) == 1:
    remote = shlex.split(remote[0])
stdin_data = sys.stdin.buffer.read()
def record(name, value):
    with open(os.path.join(state_dir, name), "a", encoding="utf-8") as output:
        output.write(value + "\n")

if remote[:2] == ["bash", "-s"]:
    script = stdin_data.decode("utf-8")
    if remote[2:3] == ["--"] and 'binary_path="$1"' in script:
        expected_sha = remote[4]
        cache_sha_path = os.path.join(state_dir, "cache.sha")
        cache_sha = open(cache_sha_path, encoding="utf-8").read().strip() if os.path.exists(cache_sha_path) else ""
        record("cache-checks", expected_sha)
        if "/db-ops/" in remote[3]:
            print(f"HIT {expected_sha}")
        else:
            print(f"HIT {cache_sha}" if cache_sha == expected_sha else "MISS")
        raise SystemExit(0)
    if remote[2:3] == ["--"] and 'remote_binary="$1"' in script:
        record("remote-runs", script)
        raise SystemExit(0)
    if "Previous migration runner temporary paths" in script:
        record("stale-cleanup", "listed")
        raise SystemExit(0)
    if len(remote) == 6:
        record("remote-cleanups", "cleaned")
        raise SystemExit(0)
    raise SystemExit(4)

if remote[:2] == ["bash", "-c"]:
    command = remote[2]
    if "gzip -dc" in command:
        if os.environ.get("MOCK_FAIL_UPLOAD") == "1":
            record("partial", "rejected")
            raise SystemExit(1)
        expected_sha, upload_nonce, compressed = stdin_data.split(b"\n", 2)
        binary = gzip.decompress(compressed)
        expected_sha = expected_sha.decode("ascii")
        actual_sha = hashlib.sha256(binary).hexdigest()
        if actual_sha != expected_sha:
            raise SystemExit(5)
        with open(os.path.join(state_dir, "cache.sha"), "w", encoding="utf-8") as output:
            output.write(actual_sha)
        with open(os.path.join(state_dir, "cache.binary"), "wb") as output:
            output.write(binary)
        with open(os.path.join(state_dir, "uploads"), "a", encoding="utf-8") as output:
            output.write(actual_sha + "\n")
        raise SystemExit(0)
    if "sha256sum" in command:
        cache_sha = open(os.path.join(state_dir, "cache.sha"), encoding="utf-8").read().strip()
        print(f"{cache_sha}  /home/opc/.cache/fn0/db-migrate/{cache_sha}/fn0-db-migrate")
        raise SystemExit(0)
    if "--help" in command:
        record("help", "--help")
        print("fn0-db-migrate usage")
        raise SystemExit(0)
    if "cat >" in command:
        upload_nonce, payload = stdin_data.split(b"\n", 1)
        name = "remote-env" if ".env." in command else "remote-args.json"
        with open(os.path.join(state_dir, name), "wb") as output:
            output.write(payload)
        raise SystemExit(0)

raise SystemExit(6)
SSH
chmod +x "${mock_bin}/pulumi" "${mock_bin}/cargo" "${mock_bin}/oci" "${mock_bin}/ssh-keygen" "${mock_bin}/ssh"

export MOCK_STATE="${state_dir}"
export BASTION_DISCOVERY_SECONDS=3
export BASTION_DISCOVERY_INTERVAL_SECONDS=0.01
export PATH="${mock_bin}:${PATH}"
inventory_output="$(bash "${mock_repo}/scripts/run-dodb-migration.sh" inventory --json --page-size 3)"
[[ "${inventory_output}" == '{"projects":[],"rows":0,"bytes":0}' ]]
[[ "$(cat "${state_dir}/pulumi-outputs")" == $'forteDbGroupToken\nforteDbHostSuffix' ]]
[[ ! -e "${state_dir}/oci-calls" ]]
python3 - "${state_dir}/cargo.json" <<'PY'
import json
import sys
data = json.load(open(sys.argv[1], encoding="utf-8"))
assert data["args"][-4:] == ["inventory", "--json", "--page-size", "3"]
assert data["token"] == "test-turso-token"
assert data["suffix"] == "example.test"
PY

bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null
[[ "$(cat "${state_dir}/pulumi-outputs")" == $'forteDbGroupToken\nforteDbHostSuffix\nworkerBastionId\nworkerSshPrivateKey\ndodbPrivateIp' ]]
[[ "$(wc -l <"${state_dir}/uploads" | tr -d '[:space:]')" == "1" ]]
[[ -e "${state_dir}/help" ]]
[[ -e "${state_dir}/stale-cleanup" ]]
[[ ! -e "${state_dir}/remote-runs" ]]
[[ "$(cat "${state_dir}/session-deletes")" == *"session-1"* ]]
if rg -q 'test-turso-token|example.test' "${state_dir}/oci-calls"; then exit 1; fi
python3 - "${state_dir}" <<'PY'
import os
import shlex
import sys
state_dir = sys.argv[1]
ssh_lines = open(os.path.join(state_dir, "ssh-argv"), encoding="utf-8").read().splitlines()
tunnel_arguments = next(shlex.split(line) for line in ssh_lines if "-N" in shlex.split(line))
target_arguments = next(shlex.split(line) for line in ssh_lines if "opc@127.0.0.1" in shlex.split(line))
assert "HostKeyAlgorithms=+ssh-rsa" in tunnel_arguments
assert "PubkeyAcceptedAlgorithms=+ssh-rsa" in tunnel_arguments
assert "IdentitiesOnly=yes" in tunnel_arguments
assert "BatchMode=yes" in tunnel_arguments
assert "ConnectTimeout=10" in tunnel_arguments
assert "ServerAliveInterval=15" in tunnel_arguments
assert "ServerAliveCountMax=3" in tunnel_arguments
assert "ExitOnForwardFailure=yes" in tunnel_arguments
assert "HostKeyAlgorithms=+ssh-rsa" not in target_arguments
assert "PubkeyAcceptedAlgorithms=+ssh-rsa" not in target_arguments
target_identity_index = target_arguments.index("-i") + 1
assert "target-ssh-key" in target_arguments[target_identity_index]
assert "bastion-session-key" not in target_arguments[target_identity_index]
bastion_identity_index = tunnel_arguments.index("-i") + 1
private_key_path = tunnel_arguments[bastion_identity_index]
assert "bastion-session-key" in private_key_path
assert not os.path.exists(private_key_path)
assert not os.path.exists(private_key_path + ".pub")
public_key_path = open(os.path.join(state_dir, "ephemeral-public-path-1"), encoding="utf-8").read().strip()
assert "bastion-session-key" in public_key_path
assert private_key_path == public_key_path[:-4]
assert not os.path.exists(public_key_path)
public_key = open(os.path.join(state_dir, "ephemeral-public-1"), encoding="utf-8").read()
assert public_key.startswith("ssh-rsa fake-rsa-public-")
assert "test-private-key" not in public_key
PY

bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null
[[ "$(wc -l <"${state_dir}/uploads" | tr -d '[:space:]')" == "1" ]]
if cmp -s "${state_dir}/ephemeral-public-1" "${state_dir}/ephemeral-public-2"; then exit 1; fi

for create_mode in missing nonzero-created delayed; do
  rm -f "${state_dir}/list-count"
  MOCK_CREATE_MODE="$create_mode" bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null
  [[ "$(cat "${state_dir}/session-deletes")" == *"session-1"* ]]
done
first_display_name="$(cat "${state_dir}/display-name")"
MOCK_CREATE_MODE=missing bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null
second_display_name="$(cat "${state_dir}/display-name")"
[[ "$first_display_name" != "$second_display_name" ]]
[[ "$first_display_name" =~ ^fn0-dodb-migration-[0-9a-f]{24}$ ]]
[[ "$second_display_name" =~ ^fn0-dodb-migration-[0-9a-f]{24}$ ]]
if MOCK_CREATE_MODE=none-nonzero bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null 2>&1; then exit 1; fi
if MOCK_CREATE_MODE=multiple bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null 2>&1; then exit 1; fi
rg -q -- "--session-id session-2" "${state_dir}/session-deletes"
if MOCK_SESSION_STATE=FAILED bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null 2>&1; then exit 1; fi
if MOCK_NO_METADATA=1 bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null 2>&1; then exit 1; fi

echo wrong >"${state_dir}/cache.sha"
bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null
[[ "$(wc -l <"${state_dir}/uploads" | tr -d '[:space:]')" == "2" ]]
echo corrupt >"${state_dir}/cache.sha"
if MOCK_FAIL_UPLOAD=1 bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null 2>&1; then exit 1; fi
[[ "$(cat "${state_dir}/cache.sha")" == "corrupt" ]]

(
  export REPO_ROOT="${mock_repo}"
  source "${mock_repo}/scripts/lib/pulumi-outputs.sh"
  source "${mock_repo}/scripts/lib/dodb-control-db.sh"
  PULUMI_OUTPUTS_JSON='{"dodbPrivateIp":"10.0.0.7","workerBastionId":"bastion-test","workerSshPrivateKey":"test-private-key"}'
  need() { command -v "$1" >/dev/null; }
  dodb_control_db_open
  [[ "${BASTION_SESSION_ID:-}" == session-1 ]]
  [[ "${BASTION_DISPLAY_NAME:-}" =~ ^fn0-dodb-control-[0-9a-f]{24}$ ]]
  dodb_control_db_close
  [[ -z "${BASTION_SESSION_ID:-}" ]]
)
[[ "$(tail -n 1 "${state_dir}/session-deletes")" == *"session-1"* ]]
[[ "$(cat "${state_dir}/display-name")" =~ ^fn0-dodb-control-[0-9a-f]{24}$ ]]

bash "${mock_repo}/scripts/run-dodb-migration.sh" verify --project-id abc --json >/dev/null
python3 - "${state_dir}" <<'PY'
import json
import os
import sys
state_dir = sys.argv[1]
arguments = json.loads(open(os.path.join(state_dir, "remote-args.json"), encoding="utf-8").read())
assert arguments == ["verify", "--project-id", "abc", "--json"]
env = open(os.path.join(state_dir, "remote-env"), encoding="utf-8").read()
assert "test-turso-token" in env
remote_runs = open(os.path.join(state_dir, "remote-runs"), encoding="utf-8").read()
assert "--dodb-addr" in remote_runs
ssh_argv = open(os.path.join(state_dir, "ssh-argv"), encoding="utf-8").read()
assert "test-turso-token" not in ssh_argv
PY

printf '%s\n' 'run-dodb-migration shell tests passed'
