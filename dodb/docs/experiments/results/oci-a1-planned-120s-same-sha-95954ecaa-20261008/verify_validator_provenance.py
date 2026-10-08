import json
import shutil
import subprocess
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parent
run_id = "09-get-main-btree-provenance-clean-r1"
raw_path = Path("raw") / f"{run_id}.jsonl"
cases = [{"name": "valid", "mutation": None, "accepted_runs": 8, "passed": True}]
for field in ("source_dirty", "git_commit", "binary_sha256", "source_sha256", "cargo_lock_sha256"):
    cases.append({"name": f"missing-{field}", "mutation": {"kind": "remove", "field": field}, "accepted_runs": 7, "passed": False})
cases.extend([
    {"name": "dirty-source", "mutation": {"kind": "set", "field": "source_dirty", "value": True}, "accepted_runs": 7, "passed": False},
    {"name": "nonboolean-source-dirty", "mutation": {"kind": "set", "field": "source_dirty", "value": 0}, "accepted_runs": 7, "passed": False},
    {"name": "wrong-git-commit", "mutation": {"kind": "set", "field": "git_commit", "value": "0" * 40}, "accepted_runs": 7, "passed": False},
    {"name": "wrong-binary-hash", "mutation": {"kind": "set", "field": "binary_sha256", "value": "0" * 64}, "accepted_runs": 7, "passed": False},
    {"name": "wrong-source-hash", "mutation": {"kind": "set", "field": "source_sha256", "value": "0" * 64}, "accepted_runs": 7, "passed": False},
    {"name": "wrong-cargo-lock-hash", "mutation": {"kind": "set", "field": "cargo_lock_sha256", "value": "0" * 64}, "accepted_runs": 7, "passed": False},
])
results = []
with tempfile.TemporaryDirectory(prefix="dodb-validator-provenance-") as temporary_root:
    temporary_root = Path(temporary_root)
    for case in cases:
        case_root = temporary_root / case["name"]
        shutil.copytree(root, case_root, ignore=shutil.ignore_patterns("validator-provenance-tests.json"))
        if case["mutation"]:
            path = case_root / raw_path
            record = json.loads(path.read_text().splitlines()[0])
            mutation = case["mutation"]
            if mutation["kind"] == "remove":
                record.pop(mutation["field"], None)
            else:
                record[mutation["field"]] = mutation["value"]
            path.write_text(json.dumps(record, separators=(",", ":")) + "\n")
        completed = subprocess.run(["python3", str(root / "validate_results.py"), "--root", str(case_root)], capture_output=True, text=True)
        status_path = case_root / "validation.json"
        status = json.loads(status_path.read_text()) if status_path.exists() else {}
        actual = {
            "name": case["name"],
            "expected_accepted_runs": case["accepted_runs"],
            "accepted_runs": status.get("accepted_runs"),
            "expected_passed": case["passed"],
            "passed": status.get("passed"),
            "exit_code": completed.returncode,
        }
        actual["test_passed"] = actual["accepted_runs"] == case["accepted_runs"] and actual["passed"] is case["passed"] and ((completed.returncode == 0) is case["passed"])
        results.append(actual)
output = {"passed": all(result["test_passed"] for result in results), "case_count": len(results), "cases": results}
(root / "validator-provenance-tests.json").write_text(json.dumps(output, indent=2) + "\n")
print(json.dumps(output, sort_keys=True))
if not output["passed"]:
    raise SystemExit(1)
