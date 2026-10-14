import csv
import json
import sys
from pathlib import Path

runs_root = Path(sys.argv[1])
output_path = Path(sys.argv[2])
fieldnames = [
    "run",
    "variant",
    "scenario",
    "tenant",
    "profile",
    "api_per_second",
    "successful_write_transactions_per_second",
    "read_percent",
    "write_percent",
    "get_p50_ms",
    "get_p95_ms",
    "get_p99_ms",
    "query16_p50_ms",
    "query16_p95_ms",
    "query16_p99_ms",
    "write_p95_ms",
    "write_p99_ms",
    "workflow_p95_ms",
    "workflow_p99_ms",
    "intentional_condition_rejections",
    "natural_conflicts",
    "overloads",
    "other_errors",
    "cpu_percent_of_one_core",
    "cpu_percent_of_host",
    "peak_sampled_rss_mib",
    "rss_sample_count",
    "validation_passed",
    "reopen_validation_passed",
]
rows = []
expected_runs = {
    "baseline-rw95",
    "fast-rw95",
    "baseline-rw80",
    "fast-rw80",
    "baseline-rw50",
    "fast-rw50",
    "baseline-rw20",
    "fast-rw20",
    "baseline-mt-mix",
    "fast-mt-mix",
}
expected_binary_hashes = {
    "baseline": "10c12d6b540d0b6175c9dcf0e288c7ad716e20db5f43236bd00b2c9a82fd02d0",
    "fast": "725e5cb017ec3f038a70eee43772b787713cae9e3adba9a94a18b81e86393f72",
}
reports = sorted(runs_root.glob("*/report.json"))
actual_runs = {report_path.parent.name for report_path in reports}
if actual_runs != expected_runs:
    raise SystemExit(f"unexpected measurement run set: {sorted(actual_runs)}")
fingerprints = {}
for report_path in reports:
    report = json.loads(report_path.read_text())
    run_name = report_path.parent.name
    variant, scenario = run_name.split("-", 1)
    run_dir = report_path.parent
    if (run_dir / "exit-status.txt").read_text().strip() != "0":
        raise SystemExit(f"workload failed: {run_name}")
    before_hash = (run_dir / "binary-before.sha256").read_text().split()[0]
    after_hash = (run_dir / "binary-after.sha256").read_text().split()[0]
    if before_hash != after_hash:
        raise SystemExit(f"binary changed during run: {run_name}")
    if before_hash != expected_binary_hashes[variant]:
        raise SystemExit(f"unexpected binary hash: {run_name}")
    expected_sha = "de790c685dee9551170d6094c945ff03ca762ac4" if variant == "baseline" else "b4325ee98be07ed7efe10119bf4e7b7c997a2b25"
    if f"source_sha={expected_sha}" not in (run_dir / "command.txt").read_text():
        raise SystemExit(f"unexpected source SHA: {run_name}")
    configuration = report["configuration"]
    expected_scenario = "mt-mix" if scenario == "mt-mix" else scenario
    if report["backend"] != "rocksdb" or report["scenario"] != expected_scenario:
        raise SystemExit(f"wrong workload identity: {run_name}")
    if report["seed"] != 20261013 or configuration["warmup_requested_ms"] != 1000 or configuration["measurement_requested_ms"] != 5000:
        raise SystemExit(f"wrong timing or seed: {run_name}")
    if configuration["clients"] != 16 or configuration["initial_rows_per_tenant"] != 10000 or configuration["value_size_bytes"] != 512:
        raise SystemExit(f"wrong workload size: {run_name}")
    if configuration["research_metrics_enabled"] or configuration["read_stage_metrics_enabled"]:
        raise SystemExit(f"detailed measurement was enabled: {run_name}")
    if configuration["rocksdb"]["revision_allocator"] != "reserved-ranges" or configuration["rocksdb"]["revision_range_size"] != 4096 or configuration["rocksdb"]["blocking_operation_limit"] != 8:
        raise SystemExit(f"wrong RocksDB settings: {run_name}")
    if not all(item["passed"] for item in report["validation"].values()):
        raise SystemExit(f"workload validation failed: {run_name}")
    if not all(item["passed"] for item in report["reopen_validation"].values()):
        raise SystemExit(f"reopen validation failed: {run_name}")
    selected_fingerprints = {}
    if scenario == "mt-mix":
        for tenant_data in report["per_tenant"].values():
            for item in tenant_data["counters"]["client_input_prefixes"]:
                selected_fingerprints[item["client_index"]] = item["fingerprint"]
    else:
        for item in report["counters"]["client_input_prefixes"]:
            selected_fingerprints[item["client_index"]] = item["fingerprint"]
    fingerprints.setdefault(scenario, {})
    fingerprints[scenario][variant] = selected_fingerprints
    process = report["process_metrics"]
    if process["sampling_completeness"] != "complete" or process["rss_sample_count"] < 2:
        raise SystemExit(f"incomplete CPU/RSS sampling: {run_name}")
    process_values = {
        "cpu_percent_of_one_core": process["cpu_percent_of_one_core"],
        "cpu_percent_of_host": process["cpu_percent_of_online_machine"],
        "peak_sampled_rss_mib": process["peak_sampled_rss_kb"] / 1024,
        "rss_sample_count": process["rss_sample_count"],
    }
    entries = []
    if scenario == "mt-mix":
        for tenant, result in sorted(report["per_tenant"].items()):
            entries.append((tenant, result["profile"], result["counters"], result["throughput_per_second"]))
    else:
        entries.append(("all", scenario, report["counters"], report["throughput_per_second"]))
    for tenant, profile, counters, throughput in entries:
        latency = counters["latency"]
        get = latency["get"]
        query = latency["query16"]
        write = latency["write_transaction_success"]
        workflow = latency["workflow"]
        rows.append(
            {
                "run": run_name,
                "variant": variant,
                "scenario": scenario,
                "tenant": tenant,
                "profile": profile,
                "api_per_second": throughput["api_calls"],
                "successful_write_transactions_per_second": throughput["successful_write_transactions"],
                "read_percent": counters["read_fraction"] * 100,
                "write_percent": counters["write_fraction"] * 100,
                "get_p50_ms": get["p50_ms"],
                "get_p95_ms": get["p95_ms"],
                "get_p99_ms": get["p99_ms"],
                "query16_p50_ms": query["p50_ms"],
                "query16_p95_ms": query["p95_ms"],
                "query16_p99_ms": query["p99_ms"],
                "write_p95_ms": write["p95_ms"],
                "write_p99_ms": write["p99_ms"],
                "workflow_p95_ms": workflow["p95_ms"],
                "workflow_p99_ms": workflow["p99_ms"],
                "intentional_condition_rejections": counters["intentional_condition_rejections"],
                "natural_conflicts": counters["natural_conflicts"],
                "overloads": counters["overloads"],
                "other_errors": counters["other_errors"],
                **process_values,
                "validation_passed": all(item["passed"] for item in report["validation"].values()),
                "reopen_validation_passed": all(item["passed"] for item in report["reopen_validation"].values()),
            }
        )
for scenario, versions in fingerprints.items():
    if versions["baseline"] != versions["fast"]:
        raise SystemExit(f"client workflow inputs differ: {scenario}")
with output_path.open("w", newline="") as output_file:
    writer = csv.DictWriter(output_file, fieldnames=fieldnames)
    writer.writeheader()
    writer.writerows(rows)
print(f"validated_measurement_runs={len(reports)} passed={len(reports)} failed=0 not_reached=0")
print(f"result_rows={len(rows)} output={output_path}")
