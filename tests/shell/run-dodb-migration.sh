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
cat >"${mock_repo}/scripts/build-rust-linux-arm64-bin.sh" <<'BUILD'
#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$2"
printf 'fake-arm64-migration-binary' >"$2/fn0-db-migrate"
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
  *"bastion session create-port-forwarding"*) printf '%s\n' '{"data":{"id":"work-1"}}' ;;
  *"bastion work-request get"*) printf '%s\n' '{"data":{"resources":[{"entity-type":"SessionResource","identifier":"session-1"}]}}' ;;
  *"bastion session get"*) printf '%s\n' '{"data":{"lifecycle-state":"ACTIVE","ssh-metadata":{"command":"ssh -N -L <localPort>:10.0.0.7:22 -i <privateKey> opc@bastion"}}}' ;;
  *) ;;
esac
OCI

cat >"${mock_bin}/ssh-keygen" <<'SSH_KEYGEN'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' 'ssh-ed25519 test'
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
if rg -q 'test-turso-token|example.test' "${state_dir}/oci-calls"; then exit 1; fi

bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null
[[ "$(wc -l <"${state_dir}/uploads" | tr -d '[:space:]')" == "1" ]]
echo wrong >"${state_dir}/cache.sha"
bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null
[[ "$(wc -l <"${state_dir}/uploads" | tr -d '[:space:]')" == "2" ]]
echo corrupt >"${state_dir}/cache.sha"
if MOCK_FAIL_UPLOAD=1 bash "${mock_repo}/scripts/run-dodb-migration.sh" transport-check >/dev/null 2>&1; then exit 1; fi
[[ "$(cat "${state_dir}/cache.sha")" == "corrupt" ]]

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
