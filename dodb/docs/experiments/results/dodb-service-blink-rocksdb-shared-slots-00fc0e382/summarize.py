import csv
import json
from pathlib import Path


result_root = Path(__file__).parent
oci_root = result_root / "oci" / "dodb-service-shared-slots-00fc0e382"
run_root = oci_root / "runs"
run_paths = sorted(run_root.glob("[0-9][0-9]-*/report.json"))
run_order = list(csv.DictReader((run_root / "run-order.csv").open()))
assert len(run_paths) == 15
assert len(run_order) == 15

columns = [
    "sequence",
    "scenario",
    "backend",
    "shared_slots",
    "tenant",
    "profile",
    "read_percent",
    "write_percent",
    "api_per_second",
    "successful_write_transactions_per_second",
    "get_p50_ms",
    "get_p95_ms",
    "get_p99_ms",
    "get_sample_count",
    "query16_p50_ms",
    "query16_p95_ms",
    "query16_p99_ms",
    "query16_sample_count",
    "write_p50_ms",
    "write_p95_ms",
    "write_p99_ms",
    "write_sample_count",
    "workflow_p50_ms",
    "workflow_p95_ms",
    "workflow_p99_ms",
    "workflow_sample_count",
    "preparatory_gets",
    "query16_calls",
    "intentional_rejections",
    "natural_conflicts",
    "overloads",
    "other_errors",
    "put_attempts",
    "delete_attempts",
    "conditional_width_1_attempts",
    "conditional_width_4_attempts",
    "conditional_width_8_attempts",
    "condition_kinds",
    "cpu_percent_one_core",
    "cpu_percent_host",
    "peak_sampled_rss_mib",
    "rss_sample_count",
    "rss_sampling_interval_ms",
    "rss_sampling_completeness",
]

summary_rows = []
pairing_result = {"run_count": len(run_paths), "scenarios": {}}
reports_by_scenario = {}

for run_path in run_paths:
    run_id = run_path.parent.name
    sequence = run_id.split("-", 1)[0]
    report = json.load(run_path.open())
    assert int((run_path.parent / "exit-status.txt").read_text()) == 0
    assert report["seed"] == 20261013
    assert report["configuration"]["clients"] == 16
    assert report["configuration"]["initial_rows_per_tenant"] == 10000
    assert report["configuration"]["value_size_bytes"] == 512
    assert report["configuration"]["warmup_requested_ms"] == 1000
    assert report["configuration"]["measurement_requested_ms"] == 5000
    assert report["configuration"]["research_metrics_enabled"] is False
    assert report["configuration"]["read_stage_metrics_enabled"] is False
    assert all(entry["passed"] for entry in report["validation"].values())
    assert all(entry["passed"] for entry in report["reopen_validation"].values())
    assert report["rocksdb_research_metrics"]["enabled"] is False

    backend = report["backend"]
    shared_slots = report["configuration"]["rocksdb"]["blocking_operation_limit"]
    if backend == "blink":
        assert shared_slots == 0
        shared_slots = "n/a"
    else:
        assert shared_slots in (8, 16)
        assert report["configuration"]["rocksdb"]["revision_range_size"] == 4096
        assert report["configuration"]["rocksdb"]["blocking_admission"] == f"shared-{shared_slots}"

    scenario = report["scenario"]
    reports_by_scenario.setdefault(scenario, []).append(report)
    process_metrics = report["process_metrics"]
    assert process_metrics["sampling_completeness"] == "complete"
    assert process_metrics["rss_sample_count"] >= 2
    assert process_metrics["peak_sampled_rss_kb"] is not None

    for tenant_name, tenant_result in report["per_tenant"].items():
        counters = tenant_result["counters"]
        latencies = counters["latency"]
        categories = counters["write_categories"]
        assert counters["request_limit_reached"] is False
        assert counters["query16_calls"] > 0
        assert counters["preparatory_gets"] > 0
        assert all(counters["condition_kinds"].get(name, 0) > 0 for name in ("Exists", "NotExists", "RevisionEquals"))
        assert counters["overloads"] == 0
        assert counters["other_errors"] == 0
        assert all(categories[name]["attempts"] > 0 for name in categories)
        assert all(latencies[name]["sample_count"] > 0 for name in ("get", "query16", "write_transaction_success", "workflow"))
        summary_rows.append(
            {
                "sequence": sequence,
                "scenario": scenario,
                "backend": backend,
                "shared_slots": shared_slots,
                "tenant": tenant_name,
                "profile": tenant_result["profile"],
                "read_percent": 100 * counters["read_fraction"],
                "write_percent": 100 * counters["write_fraction"],
                "api_per_second": tenant_result["throughput_per_second"]["api_calls"],
                "successful_write_transactions_per_second": tenant_result["throughput_per_second"]["successful_write_transactions"],
                "get_p50_ms": latencies["get"]["p50_ms"],
                "get_p95_ms": latencies["get"]["p95_ms"],
                "get_p99_ms": latencies["get"]["p99_ms"],
                "get_sample_count": latencies["get"]["sample_count"],
                "query16_p50_ms": latencies["query16"]["p50_ms"],
                "query16_p95_ms": latencies["query16"]["p95_ms"],
                "query16_p99_ms": latencies["query16"]["p99_ms"],
                "query16_sample_count": latencies["query16"]["sample_count"],
                "write_p50_ms": latencies["write_transaction_success"]["p50_ms"],
                "write_p95_ms": latencies["write_transaction_success"]["p95_ms"],
                "write_p99_ms": latencies["write_transaction_success"]["p99_ms"],
                "write_sample_count": latencies["write_transaction_success"]["sample_count"],
                "workflow_p50_ms": latencies["workflow"]["p50_ms"],
                "workflow_p95_ms": latencies["workflow"]["p95_ms"],
                "workflow_p99_ms": latencies["workflow"]["p99_ms"],
                "workflow_sample_count": latencies["workflow"]["sample_count"],
                "preparatory_gets": counters["preparatory_gets"],
                "query16_calls": counters["query16_calls"],
                "intentional_rejections": counters["intentional_condition_rejections"],
                "natural_conflicts": counters["natural_conflicts"],
                "overloads": counters["overloads"],
                "other_errors": counters["other_errors"],
                "put_attempts": categories["put"]["attempts"],
                "delete_attempts": categories["delete"]["attempts"],
                "conditional_width_1_attempts": categories["conditional_width_1"]["attempts"],
                "conditional_width_4_attempts": categories["conditional_width_4"]["attempts"],
                "conditional_width_8_attempts": categories["conditional_width_8"]["attempts"],
                "condition_kinds": json.dumps(counters["condition_kinds"], sort_keys=True),
                "cpu_percent_one_core": process_metrics["cpu_percent_of_one_core"],
                "cpu_percent_host": process_metrics["cpu_percent_of_online_machine"],
                "peak_sampled_rss_mib": process_metrics["peak_sampled_rss_kb"] / 1024,
                "rss_sample_count": process_metrics["rss_sample_count"],
                "rss_sampling_interval_ms": process_metrics["sampling_interval_ms"],
                "rss_sampling_completeness": process_metrics["sampling_completeness"],
            }
        )

for scenario_name, scenario_reports in reports_by_scenario.items():
    schedule_fingerprints = {}
    client_prefix_fingerprints = {}
    for report in scenario_reports:
        for tenant_name, tenant_result in report["per_tenant"].items():
            tenant_key = (tenant_name, tenant_result["profile"])
            schedule_fingerprints.setdefault(tenant_key, set()).add(tenant_result["schedule_fingerprint"])
            prefix_map = {
                str(entry["client_index"]): entry["fingerprint"]
                for entry in tenant_result["counters"]["client_input_prefixes"]
                if entry["workflow_count"] >= 100
            }
            client_prefix_fingerprints.setdefault(tenant_key, []).append(prefix_map)
    assert all(len(values) == 1 for values in schedule_fingerprints.values()), (scenario_name, schedule_fingerprints)
    assert all(all(prefix_map == prefix_maps[0] for prefix_map in prefix_maps[1:]) for prefix_maps in client_prefix_fingerprints.values()), (scenario_name, client_prefix_fingerprints)
    pairing_result["scenarios"][scenario_name] = {
        "run_count": len(scenario_reports),
        "configuration_profiles": sorted(report["backend"] + ":" + str(report["configuration"]["rocksdb"]["blocking_operation_limit"]) for report in scenario_reports),
        "schedule_fingerprints_match": True,
        "first_100_client_input_prefixes_match": True,
    }

assert len(summary_rows) == 24
with (result_root / "condition-summary.csv").open("w", newline="") as output_file:
    writer = csv.DictWriter(output_file, fieldnames=columns)
    writer.writeheader()
    writer.writerows(summary_rows)
(result_root / "paired-input-validation.json").write_text(json.dumps(pairing_result, indent=2) + "\n")
print(json.dumps({"performance_runs": len(run_paths), "tenant_rows": len(summary_rows), "paired_inputs": pairing_result}, indent=2))
