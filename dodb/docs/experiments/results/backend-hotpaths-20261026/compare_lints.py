import collections
import json
import os
import subprocess
import sys
from pathlib import Path


root = Path(os.environ["DODB_HOTPATH_ROOT"])
environment = os.environ.copy()
environment.update({"DODB_BASE_RUN_ROOT": str(root), "PYTHONDONTWRITEBYTECODE": "1"})
helper = root / "source/dodb/docs/experiments/results/blink-durable-bases-20261023/run_build.py"
sources = {"candidate": root / "source", "control": Path("/bench/zfs/db/blink-checkpoint-costs-20261023/source")}
diagnostics = {}
for label, source in sources.items():
    run_id = f"backend-hotpaths-3b94ee88e-clippy-{label}-json"
    environment["DODB_BUILD_RUN_ID"] = run_id
    command = [sys.executable, str(helper), f"lint-{label}-json", "bash", "dodb/scripts/run-oci-rocksdb-build.sh",
               "clippy", "--no-deps", "--locked", "--release", "-j", "1", "-p", "dodb-server", "-p", "dodb-rocksdb",
               "--lib", "--tests", "--message-format=json"]
    record = {"argv": command, "cwd": str(source), "build_run_id": run_id}
    (root / f"lint-command-{label}-json.json").write_text(json.dumps(record, indent=2) + "\n")
    with (root / f"lint-{label}-json.log").open("wb") as stream:
        subprocess.run(command, cwd=source, env=environment, stdout=stream, stderr=subprocess.STDOUT, check=True)
    cargo_log = Path("/bench/zfs/db/dodb-rocksdb-build-runs") / run_id / "cargo.log"
    entries = []
    for line in cargo_log.read_text().splitlines():
        if not line.startswith("{"):
            continue
        value = json.loads(line)
        if value.get("reason") != "compiler-message":
            continue
        message = value["message"]
        assert message["level"] != "error", message
        if message["level"] != "warning" or message.get("code") is None:
            continue
        primary = [span for span in message["spans"] if span["is_primary"]]
        entries.append({"code": message["code"]["code"], "message": message["message"],
                        "locations": [{"file": span["file_name"], "text": [entry["text"] for entry in span["text"]]} for span in primary]})
    diagnostics[label] = entries
candidate = collections.Counter(json.dumps(entry, sort_keys=True) for entry in diagnostics["candidate"])
control = collections.Counter(json.dumps(entry, sort_keys=True) for entry in diagnostics["control"])
new = list((candidate - control).elements())
result = {"new_diagnostics": [json.loads(entry) for entry in new], "candidate_count": sum(candidate.values()),
          "control_count": sum(control.values()), "diagnostics": diagnostics, "no_new_diagnostics": not new,
          "strict_result": "existing diagnostics remain; no lint suppressions or unrelated fixes"}
(root / "lint-comparison.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result, indent=2))
assert not new
