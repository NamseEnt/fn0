import csv
import importlib.util
import json
import statistics
from collections import defaultdict
from pathlib import Path


result_root = Path(__file__).resolve().parent
runner_path = result_root.parents[3] / "scripts" / "run-rocksdb-read-write-admission.py"
module_spec = importlib.util.spec_from_file_location("attribution_runner", runner_path)
runner = importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(runner)
oci_root = result_root / "oci" / "measurements"
plan = json.loads((oci_root / "plan.json").read_text())
assert json.loads((oci_root / "complete.json").read_text())["runs"] == 60
run_paths = sorted((oci_root / "runs").glob("*/report.json"))
assert len(run_paths) == plan["performance_runs"] + plan["diagnostic_runs"] == 60
assert len(list((oci_root / "smoke").glob("*/checked.json"))) == 6
for smoke_path in sorted((oci_root / "smoke").glob("*/report.json")):
    report = json.loads(smoke_path.read_text())
    provenance = json.loads((smoke_path.parent / "provenance.json").read_text())
    assert provenance["binary_sha256_before"] == provenance["binary_sha256_after"] == plan["binary_sha256"]
    assert (smoke_path.parent / "exit-status.txt").read_text().strip() == "0"
    runner.verify_report(report, provenance["profile"].endswith("-on"))
rows = []
stage_rows = []
write_read_rows = []
outcome_rows = []
inputs = {}


def write_csv(path, values):
    with path.open("w", newline="") as output:
        writer = csv.DictWriter(output, fieldnames=list(values[0]), lineterminator="\n")
        writer.writeheader()
        writer.writerows(values)


for report_path in run_paths:
    report = json.loads(report_path.read_text())
    provenance = json.loads((report_path.parent / "provenance.json").read_text())
    assert provenance["binary_sha256_before"] == provenance["binary_sha256_after"] == plan["binary_sha256"]
    assert (report_path.parent / "exit-status.txt").read_text().strip() == "0"
    enabled = provenance["profile"].endswith("-on")
    runner.verify_report(report, enabled)
    configuration = report["configuration"]
    assert configuration["clients"] == 16
    assert configuration["initial_rows_per_tenant"] == 10000
    assert configuration["value_size_bytes"] == 512
    assert configuration["warmup_requested_ms"] == 1000
    assert configuration["measurement_requested_ms"] == 5000
    assert report["seed"] == 20261013
    expected = next(profile for profile in runner.PROFILES + runner.DIAGNOSTIC_PROFILES if profile[0] == provenance["profile"])
    assert report["backend"] == expected[1]
    if expected[1] == "rocksdb":
        rocksdb = configuration["rocksdb"]
        assert rocksdb["revision_allocator"] == expected[2]
        expected_admission = expected[3] if expected[3].startswith("split-") else f"shared-{expected[4]}"
        assert rocksdb["blocking_admission"] == expected_admission
        assert rocksdb["blocking_operation_limit"] == expected[4]
        assert rocksdb["read_blocking_slots"] == (2 if expected[3].startswith("split-") else expected[4])
        assert rocksdb["write_blocking_slots"] == (8 if expected[3].startswith("split-") else expected[4])
    assert configuration["read_stage_metrics_enabled"] is enabled
    assert configuration["detailed_write_metrics_enabled"] is enabled
    assert report["process_metrics"]["sampling_completeness"] == "complete"
    for tenant_name, tenant in report["per_tenant"].items():
        counters = tenant["counters"]
        categories = counters["write_categories"]
        assert all(category["attempts"] > 0 for category in categories.values())
        assert all(counters["condition_kinds"].get(name, 0) > 0 for name in ["Exists", "NotExists", "RevisionEquals"])
        expected_clients = 4 if report["scenario"] == "mt-mix" else 16
        prefixes = counters["client_input_prefixes"]
        assert len(prefixes) == expected_clients
        assert all(prefix["workflow_count"] == 100 for prefix in prefixes)
        input_key = (report["scenario"], tenant_name)
        fingerprint = {
            "schedule": tenant["schedule_fingerprint"],
            "clients": {str(prefix["client_index"]): prefix["fingerprint"] for prefix in prefixes},
        }
        if input_key in inputs:
            assert inputs[input_key] == fingerprint, (input_key, report_path)
        inputs[input_key] = fingerprint
        base = {"sequence": provenance["sequence"], "scenario": report["scenario"],
                "configuration": provenance["profile"], "repetition": provenance["repetition"],
                "tenant": tenant_name, "profile": tenant["profile"]}
        row = {**base, "api_per_second": tenant["throughput_per_second"]["api_calls"],
               "writes_per_second": tenant["throughput_per_second"]["successful_write_transactions"],
               "api_calls": counters["api_calls"], "successful_writes": counters["successful_write_transactions"],
               "read_percent": 100 * counters["read_fraction"], "write_percent": 100 * counters["write_fraction"],
               "rejects": counters["intentional_condition_rejections"], "conflicts": counters["natural_conflicts"],
               "other_errors": counters["other_errors"], "overloads": counters["overloads"],
               "cpu_one_core_percent": report["process_metrics"]["cpu_percent_of_one_core"],
               "rss_mib": report["process_metrics"]["peak_sampled_rss_kb"] / 1024}
        for category in ["get", "preparatory_get", "query16", "write_transaction_success", "workflow"]:
            latency = counters["latency"][category]
            row[category + "_count"] = latency["count"]
            row[category + "_samples"] = latency["sample_count"]
            for percentile in ["p50_ms", "p95_ms", "p99_ms"]:
                row[category + "_" + percentile] = latency[percentile]
        rows.append(row)
        if not enabled:
            continue
        metrics = report["rocksdb_research_metrics"]["tenants"][tenant_name]
        for kind, expected_count in [("get", counters["ordinary_gets"]),
                                     ("preparatory_get", counters["preparatory_gets"]),
                                     ("query16", counters["query16_calls"])]:
            applicable = ["request_to_permit_acquire_start", "semaphore_wait", "blocking_dispatch_wait",
                          "blocking_closure_duration", "blocking_completion_handoff", "tenant_database_lookup",
                          "service_request_total"]
            applicable += ["query_snapshot_iterator_setup", "query_iterator_calls_sum", "query_copy_decode_budget"] if kind == "query16" else ["rocksdb_get_call", "get_copy_decode"]
            for metric_name in applicable:
                assert metrics["read_stages"]["measurement"][kind][metric_name]["count"] == expected_count
        for metric_name, expected_count in [
            ("service_request_duration", counters["api_calls"]),
            ("read_semaphore_wait", counters["read_calls"]),
            ("write_semaphore_wait", counters["write_requests"]),
            ("blocking_dispatch_wait", counters["api_calls"]),
        ]:
            assert metrics["measurement"][metric_name]["count"] == expected_count
        outcome_rows.append({**base, **metrics["outcomes"]["measurement"]})
        for metric_name, histogram in metrics["measurement"].items():
            stage_rows.append({**base, "kind": "all", "metric": metric_name, **histogram})
        for kind, histograms in metrics["read_stages"]["measurement"].items():
            for metric_name, histogram in histograms.items():
                stage_rows.append({**base, "kind": kind, "metric": metric_name, **histogram})
        for role_name, role in metrics["write_reads"]["measurement"].items():
            for metric_name, histogram in role["timings"].items():
                write_read_rows.append({**base, "role": role_name, "metric": metric_name,
                                       "keys": role["keys"], "found_keys": role["found_keys"],
                                       "encoded_bytes_fetched": role["encoded_bytes_fetched"],
                                       "payload_bytes_fetched": role["payload_bytes_fetched"], **histogram})

assert len(inputs) == 8
assert sum(not json.loads((path.parent / "provenance.json").read_text())["profile"].endswith("-on") for path in run_paths) == 45
assert sum(json.loads((path.parent / "provenance.json").read_text())["profile"].endswith("-on") for path in run_paths) == 15
assert len(rows) == 96
assert len(outcome_rows) == 24
write_csv(result_root / "condition-summary.csv", rows)
write_csv(result_root / "stage-metrics.csv", stage_rows)
write_csv(result_root / "write-read-metrics.csv", write_read_rows)
write_csv(result_root / "outcomes.csv", outcome_rows)


def aggregate(values, group_fields, measurements):
    groups = defaultdict(list)
    for value in values:
        groups[tuple(value[field] for field in group_fields)].append(value)
    summarized = []
    for group_key, group_values in sorted(groups.items()):
        expected_repetitions = 1 if group_values[0]["configuration"].endswith("-on") else 3
        assert len(group_values) == expected_repetitions, group_key
        summary = dict(zip(group_fields, group_key))
        for measurement in measurements:
            samples = [value[measurement] for value in group_values if value[measurement] is not None]
            summary[measurement] = None if not samples else {
                "median": statistics.median(samples), "min": min(samples), "max": max(samples)}
        summarized.append(summary)
    return summarized


summary = {
    "source_sha": plan["source_sha"], "binary_sha256": plan["binary_sha256"],
    "performance_runs": 45, "diagnostic_runs": 15, "smoke_runs": 6, "tenant_rows": len(rows),
    "input_groups_matched": len(inputs), "repetitions": 3,
    "conditions": aggregate([row for row in rows if row["configuration"].endswith("-off")], ["scenario", "configuration", "tenant"], [
        "api_per_second", "writes_per_second", "get_p95_ms", "get_p99_ms",
        "query16_p95_ms", "query16_p99_ms", "write_transaction_success_p95_ms",
        "write_transaction_success_p99_ms", "workflow_p95_ms", "cpu_one_core_percent", "rss_mib"]),
    "stages": aggregate(stage_rows, ["scenario", "configuration", "tenant", "kind", "metric"], ["mean_ms"]),
    "write_reads": aggregate(write_read_rows, ["scenario", "configuration", "tenant", "role", "metric"], ["mean_ms", "payload_bytes_fetched", "keys"]),
}
summary["diagnostic_conditions"] = aggregate(
    [row for row in rows if row["configuration"].endswith("-on")],
    ["scenario", "configuration", "tenant"],
    ["writes_per_second", "get_p95_ms", "get_p99_ms", "query16_p95_ms", "query16_p99_ms", "write_transaction_success_p99_ms"],
)
primary_ranges = []
for condition in summary["conditions"]:
    flattened = {field: condition[field] for field in ["scenario", "configuration", "tenant"]}
    for measurement, statistics_by_name in condition.items():
        if not isinstance(statistics_by_name, dict):
            continue
        for statistic_name, measured_value in statistics_by_name.items():
            flattened[f"{measurement}_{statistic_name}"] = measured_value
    primary_ranges.append(flattened)
write_csv(result_root / "median-ranges.csv", primary_ranges)
condition_index = {
    (condition["scenario"], condition["configuration"], condition["tenant"]): condition
    for condition in summary["conditions"]
}
comparisons = []
for (scenario, configuration, tenant), candidate in sorted(condition_index.items()):
    if configuration == "ranges-shared8-off":
        continue
    reference = condition_index[(scenario, "ranges-shared8-off", tenant)]
    ratios = {
        measurement: candidate[measurement]["median"] / reference[measurement]["median"]
        for measurement in ["writes_per_second", "get_p95_ms", "get_p99_ms", "query16_p95_ms", "query16_p99_ms", "write_transaction_success_p99_ms"]
    }
    screen = plan["decision_screen"]
    write_throughput_passed = ratios["writes_per_second"] >= screen["minimum_write_throughput_ratio"]
    write_p99_passed = ratios["write_transaction_success_p99_ms"] <= screen["maximum_write_p99_ratio"]
    reads_passed = all(ratios[measurement] < 1 for measurement in ["get_p95_ms", "get_p99_ms", "query16_p95_ms", "query16_p99_ms"])
    comparisons.append({
        "scenario": scenario, "configuration": configuration, "tenant": tenant,
        **ratios, "write_throughput_passed": write_throughput_passed,
        "write_p99_passed": write_p99_passed, "reads_passed": reads_passed,
        "screen_passed": write_throughput_passed and write_p99_passed and reads_passed,
    })
assert len(comparisons) == 16
write_csv(result_root / "comparison-ratios.csv", comparisons)
summary["decision_screen"] = plan["decision_screen"]
summary["comparisons"] = comparisons
(result_root / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
(result_root / "paired-input-validation.json").write_text(json.dumps({
    "passed": True, "performance_runs": 45, "diagnostic_runs": 15,
    "groups": {":".join(group_key): value for group_key, value in inputs.items()},
}, indent=2) + "\n")
print(json.dumps({"runs": 60, "smokes": 6, "tenant_rows": len(rows), "input_groups": len(inputs), "passed": True}))

rejected_root = result_root / "oci" / "measurements-capped-latencies"
rejected_reports = sorted((rejected_root / "runs").glob("*/report.json"))
affected = []
for report_path in rejected_reports:
    report = json.loads(report_path.read_text())
    for tenant_name, tenant in report["per_tenant"].items():
        for category, latency in tenant["counters"]["latency"].items():
            if latency["count"] > latency["sample_count"]:
                affected.append({
                    "run": report_path.parent.name, "tenant": tenant_name,
                    "category": category, "count": latency["count"],
                    "sample_count": latency["sample_count"],
                })
assert affected
(result_root / "rejected-attempt-summary.json").write_text(json.dumps({
    "accepted": False, "source_sha": "6331aae3113e9c4e97ddd3159aa4b9a9d0a92ac3",
    "complete_marker_present": (rejected_root / "complete.json").exists(),
    "reports_preserved": len(rejected_reports),
    "checked_runs_preserved": len(list((rejected_root / "runs").glob("*/checked.json"))),
    "smoke_reports_preserved": len(list((rejected_root / "smoke").glob("*/report.json"))),
    "affected_tenant_categories": affected,
    "reason": "biased capped latency replacement and truncated aggregate merging; stopped before acceptance",
}, indent=2) + "\n")
