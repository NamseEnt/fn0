import hashlib
import json
import os
import subprocess
from pathlib import Path


job_root = Path("/bench/zfs/db/dodb-wrapper-attribution-20261015")
build_root = Path("/bench/zfs/db/dodb-rocksdb-build-runs/wrapper-attribution-d56382ef2-tests-1")
source_sha = "d56382ef236542846b3bb836570566f835b2a0e3"
assert subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip() == source_sha
assert not subprocess.check_output(["git", "status", "--porcelain"], text=True).strip()
assert (build_root / "exit-status.txt").read_text().strip() == "0"
executables = {}
for line in (build_root / "cargo.log").read_text().splitlines():
    if not line.startswith("{"):
        continue
    event = json.loads(line)
    if event.get("reason") == "compiler-artifact" and event["profile"]["test"] and event.get("executable"):
        executables[event["target"]["name"]] = Path(event["executable"])
assert set(executables) == {"dodb_rocksdb", "service_workload", "read_contract", "transaction_contract", "research_metrics"}, executables
focused_root = job_root / "focused"
focused_root.mkdir(exist_ok=False)
temporary_root = focused_root / "tmp"
temporary_root.mkdir()
(focused_root / "db").mkdir()
env = dict(os.environ)
env["TMPDIR"] = str(temporary_root)
env["DODB_TEST_DATA_DIR"] = str(focused_root / "db")
manifest = {}
for target_name, executable in sorted(executables.items()):
    before_hash = hashlib.sha256(executable.read_bytes()).hexdigest()
    command = [str(executable), "--nocapture", "--test-threads=1"]
    (focused_root / (target_name + "-command.json")).write_text(json.dumps(command) + "\n")
    with (focused_root / (target_name + ".log")).open("w") as output:
        completed = subprocess.run(command, env=env, stdout=output, stderr=subprocess.STDOUT)
    after_hash = hashlib.sha256(executable.read_bytes()).hexdigest()
    manifest[target_name] = {"binary": str(executable), "sha256_before": before_hash,
                             "sha256_after": after_hash, "exit_status": completed.returncode}
    (focused_root / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps({"target": target_name, "exit_status": completed.returncode}), flush=True)
    assert before_hash == after_hash
    assert completed.returncode == 0, target_name
assert not subprocess.check_output(["git", "status", "--porcelain"], text=True).strip()
