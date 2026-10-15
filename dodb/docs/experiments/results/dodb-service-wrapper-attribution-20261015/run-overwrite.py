import hashlib
import json
import subprocess
from pathlib import Path


job_root = Path("/bench/zfs/db/dodb-wrapper-attribution-20261015")
source_sha = "4418f506b0e93ed5d4dde5590b291d38031b8bda"
assert subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip() == source_sha
assert not subprocess.check_output(["git", "status", "--porcelain"], text=True).strip()
binary = job_root / "binaries" / "wrapper_overwrite_probe-4418f506b"
binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
root = job_root / "overwrite"
root.mkdir(exist_ok=False)
profiles = [("put-on", "put", True), ("exists-on", "exists", True), ("put-off", "put", False)]
manifest = {"source_sha": source_sha, "binary_sha256": binary_hash, "planned_runs": 9, "completed": []}
prefixes_by_mode = {}
for repetition in range(1, 4):
    offset = repetition - 1
    for profile, mode, enabled in profiles[offset:] + profiles[:offset]:
        run_name = f"{profile}-rep{repetition}"
        run_path = root / run_name
        run_path.mkdir()
        command = ["/usr/bin/time", "-v", "-o", str(run_path / "process-time.txt"), str(binary),
                   "--data-dir", str(run_path / "db"), "--report", str(run_path / "report.json"),
                   "--mode", mode, "--metrics", str(int(enabled))]
        (run_path / "command.json").write_text(json.dumps(command, indent=2) + "\n")
        with (run_path / "stdout.txt").open("w") as output:
            completed = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT)
        (run_path / "exit-status.txt").write_text(str(completed.returncode) + "\n")
        assert completed.returncode == 0, run_name
        assert hashlib.sha256(binary.read_bytes()).hexdigest() == binary_hash
        report = json.loads((run_path / "report.json").read_text())
        assert report["validation"] and report["reopen_validation"]
        assert report["mode"] == mode and report["value_bytes"] == 512 and report["clients"] == 16
        assert report["research_metrics"]["enabled"] is enabled
        assert report["latency"]["samples"] == report["calls"]
        if mode in prefixes_by_mode:
            assert prefixes_by_mode[mode] == report["client_prefixes"]
        prefixes_by_mode[mode] = report["client_prefixes"]
        if enabled:
            metrics = report["research_metrics"]["tenants"]["tenant_1"]
            roles = metrics["write_reads"]["measurement"]
            role_name = "condition_key" if mode == "exists" else "mutation_only_key"
            active = roles[role_name]
            assert active["keys"] == active["found_keys"] == report["calls"]
            assert active["payload_bytes_fetched"] == 512 * report["calls"]
            assert active["encoded_bytes_fetched"] == 522 * report["calls"]
            assert sum(role["keys"] for role in roles.values()) == report["calls"]
            assert all(timing["count"] == report["calls"] for timing in active["timings"].values())
            assert metrics["measurement"]["service_request_duration"]["count"] == report["calls"]
            assert metrics["outcomes"]["measurement"]["user_commits"] == report["calls"]
            assert metrics["outcomes"]["measurement"]["user_transaction_counter_accesses"] == 0
        manifest["completed"].append(run_name)
        (root / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        print(json.dumps({"run": run_name, "passed": True}), flush=True)
assert len(manifest["completed"]) == 9
assert not subprocess.check_output(["git", "status", "--porcelain"], text=True).strip()
(root / "complete.json").write_text(json.dumps({"passed": True, "runs": 9}) + "\n")
