import csv
import json
import tarfile
from pathlib import Path


result_root = Path(__file__).parent
archive_path = result_root / "oci" / "evidence-export.tar.gz"
reports = {}
with tarfile.open(archive_path, "r:gz") as archive:
    for member in archive.getmembers():
        parts = Path(member.name).parts
        if len(parts) < 8 or parts[-1] != "report.json" or "measure" not in parts:
            continue
        measure_index = parts.index("measure")
        scenario = parts[measure_index + 1]
        backend_mode = parts[measure_index + 2]
        backend = "rocksdb" if backend_mode.startswith("rocksdb-") else "blink"
        stream = archive.extractfile(member)
        if stream is None:
            raise RuntimeError(f"missing report contents: {member.name}")
        reports[(scenario, backend)] = json.loads(stream.read())


scenarios = ["rw95", "rw80", "rw50", "rw20", "mt-mix"]
for scenario in scenarios:
    for backend in ["blink", "rocksdb"]:
        report = reports[(scenario, backend)]
        if not all(item.get("passed") for item in report["validation"].values()):
            raise RuntimeError(f"independent replay validation failed: {scenario} {backend}")
        if not all(item.get("passed") for item in report["reopen_validation"].values()):
            raise RuntimeError(f"reopen validation failed: {scenario} {backend}")
        if report["configuration"]["research_metrics_enabled"]:
            raise RuntimeError(f"research metrics enabled: {scenario} {backend}")
    blink_report = reports[(scenario, "blink")]
    rocks_report = reports[(scenario, "rocksdb")]
    if set(blink_report["per_tenant"]) != set(rocks_report["per_tenant"]):
        raise RuntimeError(f"tenant schedule mismatch: {scenario}")
    for tenant_name in blink_report["per_tenant"]:
        blink_tenant = blink_report["per_tenant"][tenant_name]
        rocks_tenant = rocks_report["per_tenant"][tenant_name]
        if blink_tenant["schedule_fingerprint"] != rocks_tenant["schedule_fingerprint"]:
            raise RuntimeError(f"schedule fingerprint mismatch: {scenario} {tenant_name}")
        blink_prefixes = {
            item["client_index"]: item["fingerprint"]
            for item in blink_tenant["counters"]["client_input_prefixes"]
        }
        rocks_prefixes = {
            item["client_index"]: item["fingerprint"]
            for item in rocks_tenant["counters"]["client_input_prefixes"]
        }
        if not blink_prefixes or blink_prefixes != rocks_prefixes:
            raise RuntimeError(f"client input prefix mismatch: {scenario} {tenant_name}")


columns = [
    "scenario",
    "backend",
    "tenant",
    "target_read_pct",
    "actual_read_pct",
    "actual_write_pct",
    "api_calls",
    "api_per_second",
    "successful_write_transactions",
    "successful_write_transactions_per_second",
    "successful_write_p50_ms",
    "successful_write_p95_ms",
    "successful_write_p99_ms",
    "get_p95_ms",
    "get_p99_ms",
    "query16_p95_ms",
    "query16_p99_ms",
    "workflow_p50_ms",
    "workflow_p95_ms",
    "workflow_p99_ms",
    "preparatory_gets",
    "preparatory_get_p95_ms",
    "preparatory_get_p99_ms",
    "intentional_condition_rejections",
    "natural_conflicts",
    "overloads",
    "other_errors",
    "write_categories",
    "condition_kinds",
    "cpu_percent_of_one_core",
    "cpu_percent_of_machine",
    "peak_sampled_rss_kb",
    "rss_sample_count",
    "disk_bytes_before_shutdown",
    "disk_bytes_after_reopen_validation",
    "validation_passed",
    "reopen_validation_passed",
    "schedule_fingerprint",
    "client_input_prefixes_equal",
]
csv_path = result_root / "condition-summary.csv"
with csv_path.open("w", newline="") as output_file:
    writer = csv.DictWriter(output_file, fieldnames=columns, lineterminator="\n")
    writer.writeheader()
    for scenario in scenarios:
        for backend in ["blink", "rocksdb"]:
            report = reports[(scenario, backend)]
            target_percent = {
                "rw95": 95,
                "rw80": 80,
                "rw50": 50,
                "rw20": 20,
            }
            for tenant_name, tenant in report["per_tenant"].items():
                counters = tenant["counters"]
                latency = counters["latency"]
                successful_write = latency["write_transaction_success"]
                ordinary_get = latency["get"]
                query16 = latency["query16"]
                workflow = latency["workflow"]
                preparatory_get = latency["preparatory_get"]
                write_categories = counters["write_categories"]
                process = report["process_metrics"]
                blink_tenant = reports[(scenario, "blink")]["per_tenant"][tenant_name]
                rocks_tenant = reports[(scenario, "rocksdb")]["per_tenant"][tenant_name]
                blink_prefixes = {
                    item["client_index"]: item["fingerprint"]
                    for item in blink_tenant["counters"]["client_input_prefixes"]
                }
                rocks_prefixes = {
                    item["client_index"]: item["fingerprint"]
                    for item in rocks_tenant["counters"]["client_input_prefixes"]
                }
                row = {
                    "scenario": scenario,
                    "backend": backend,
                    "tenant": tenant_name,
                    "target_read_pct": target_percent.get(scenario, "tenant-specific"),
                    "actual_read_pct": round(counters["read_fraction"] * 100, 4),
                    "actual_write_pct": round(counters["write_fraction"] * 100, 4),
                    "api_calls": counters["api_calls"],
                    "api_per_second": round(tenant["throughput_per_second"]["api_calls"], 2),
                    "successful_write_transactions": counters["successful_write_transactions"],
                    "successful_write_transactions_per_second": round(tenant["throughput_per_second"]["successful_write_transactions"], 2),
                    "successful_write_p50_ms": successful_write["p50_ms"],
                    "successful_write_p95_ms": successful_write["p95_ms"],
                    "successful_write_p99_ms": successful_write["p99_ms"],
                    "get_p95_ms": ordinary_get["p95_ms"],
                    "get_p99_ms": ordinary_get["p99_ms"],
                    "query16_p95_ms": query16["p95_ms"],
                    "query16_p99_ms": query16["p99_ms"],
                    "workflow_p50_ms": workflow["p50_ms"],
                    "workflow_p95_ms": workflow["p95_ms"],
                    "workflow_p99_ms": workflow["p99_ms"],
                    "preparatory_gets": counters["preparatory_gets"],
                    "preparatory_get_p95_ms": preparatory_get["p95_ms"],
                    "preparatory_get_p99_ms": preparatory_get["p99_ms"],
                    "intentional_condition_rejections": counters["intentional_condition_rejections"],
                    "natural_conflicts": counters["natural_conflicts"],
                    "overloads": counters["overloads"],
                    "other_errors": counters["other_errors"],
                    "write_categories": json.dumps(write_categories, sort_keys=True),
                    "condition_kinds": json.dumps(counters["condition_kinds"], sort_keys=True),
                    "cpu_percent_of_one_core": report["process_metrics"]["cpu_percent_of_one_core"],
                    "cpu_percent_of_machine": report["process_metrics"]["cpu_percent_of_online_machine"],
                    "peak_sampled_rss_kb": process["peak_sampled_rss_kb"],
                    "rss_sample_count": process["rss_sample_count"],
                    "disk_bytes_before_shutdown": report["disk_usage_bytes"]["before_shutdown"],
                    "disk_bytes_after_reopen_validation": report["disk_usage_bytes"]["after_reopen_validation"],
                    "validation_passed": all(item.get("passed") for item in report["validation"].values()),
                    "reopen_validation_passed": all(item.get("passed") for item in report["reopen_validation"].values()),
                    "schedule_fingerprint": tenant["schedule_fingerprint"],
                    "client_input_prefixes_equal": blink_prefixes == rocks_prefixes,
                }
                if scenario == "mt-mix":
                    row["target_read_pct"] = round((95, 80, 50, 20)[int(tenant_name.split("_")[1]) - 1], 2)
                writer.writerow(row)


fingerprints_path = result_root / "paired-input-validation.json"
fingerprints = {}
for scenario in scenarios:
    fingerprints[scenario] = {}
    for tenant_name in reports[(scenario, "blink")]["per_tenant"]:
        blink_tenant = reports[(scenario, "blink")]["per_tenant"][tenant_name]
        rocks_tenant = reports[(scenario, "rocksdb")]["per_tenant"][tenant_name]
        blink_prefixes = {
            str(item["client_index"]): item["fingerprint"]
            for item in blink_tenant["counters"]["client_input_prefixes"]
        }
        rocks_prefixes = {
            str(item["client_index"]): item["fingerprint"]
            for item in rocks_tenant["counters"]["client_input_prefixes"]
        }
        fingerprints[scenario][tenant_name] = {
            "schedule_fingerprint": blink_tenant["schedule_fingerprint"],
            "client_prefixes_equal": blink_prefixes == rocks_prefixes,
            "client_prefix_count": len(blink_prefixes),
            "blink_client_prefixes": blink_prefixes,
            "rocksdb_client_prefixes": rocks_prefixes,
        }
fingerprints_path.write_text(json.dumps(fingerprints, indent=2) + "\n")
