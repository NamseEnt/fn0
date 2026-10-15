import csv
import json
import statistics
from collections import defaultdict
from pathlib import Path


root = Path(__file__).resolve().parent
probe_root = root / "oci" / "overwrite"
manifest = json.loads((probe_root / "manifest.json").read_text())
assert len(manifest["completed"]) == manifest["planned_runs"] == 9
assert json.loads((probe_root / "complete.json").read_text())["passed"]
rows = []
prefixes_by_mode = {}
for run_name in manifest["completed"]:
    run_path = probe_root / run_name
    report = json.loads((run_path / "report.json").read_text())
    assert (run_path / "exit-status.txt").read_text().strip() == "0"
    assert report["validation"] and report["reopen_validation"]
    assert report["clients"] == report["initial_keys"] == 16
    assert report["value_bytes"] == 512 and report["shared_slots"] == 8
    assert report["revision_allocator"] == "reserved-ranges"
    assert report["latency"]["samples"] == report["calls"]
    if report["mode"] in prefixes_by_mode:
        assert prefixes_by_mode[report["mode"]] == report["client_prefixes"]
    prefixes_by_mode[report["mode"]] = report["client_prefixes"]
    row = {"run": run_name, "profile": run_name.rsplit("-rep", 1)[0],
           "calls": report["calls"], "calls_per_second": report["calls_per_second"],
           "p95_ms": report["latency"]["p95_ms"], "p99_ms": report["latency"]["p99_ms"],
           "native_read_us": None, "lock_wait_us": None, "shim_copy_us": None,
           "rust_copy_us": None, "decode_us": None, "copy_decode_us": None,
           "service_request_us": None, "user_commit_us": None}
    if report["research_metrics"]["enabled"]:
        metrics = report["research_metrics"]["tenants"]["tenant_1"]
        roles = metrics["write_reads"]["measurement"]
        role_name = "condition_key" if report["mode"] == "exists" else "mutation_only_key"
        active = roles[role_name]
        assert active["keys"] == active["found_keys"] == report["calls"]
        assert active["payload_bytes_fetched"] == report["calls"] * 512
        assert active["encoded_bytes_fetched"] == report["calls"] * 522
        assert sum(role["keys"] for role in roles.values()) == report["calls"]
        for field, metric_name in [("native_read_us", "native_get_for_update"),
                                   ("lock_wait_us", "native_lock_wait"), ("shim_copy_us", "shim_value_copy"),
                                   ("rust_copy_us", "rust_value_copy"), ("decode_us", "state_decode")]:
            assert active["timings"][metric_name]["count"] == report["calls"]
            row[field] = active["timings"][metric_name]["mean_ms"] * 1000
        row["copy_decode_us"] = row["shim_copy_us"] + row["rust_copy_us"] + row["decode_us"]
        row["service_request_us"] = metrics["measurement"]["service_request_duration"]["mean_ms"] * 1000
        row["user_commit_us"] = metrics["measurement"]["user_commit_duration"]["mean_ms"] * 1000
    rows.append(row)
with (root / "overwrite-summary.csv").open("w", newline="") as output:
    writer = csv.DictWriter(output, fieldnames=list(rows[0]), lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
groups = defaultdict(list)
for row in rows:
    groups[row["profile"]].append(row)
summary = {"source_sha": manifest["source_sha"], "binary_sha256": manifest["binary_sha256"],
           "runs": 9, "profiles": {}}
for profile, values in groups.items():
    assert len(values) == 3
    summary["profiles"][profile] = {}
    for metric_name in list(rows[0])[3:]:
        samples = [value[metric_name] for value in values if value[metric_name] is not None]
        summary["profiles"][profile][metric_name] = None if not samples else {
            "median": statistics.median(samples), "min": min(samples), "max": max(samples)}
(root / "overwrite-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
