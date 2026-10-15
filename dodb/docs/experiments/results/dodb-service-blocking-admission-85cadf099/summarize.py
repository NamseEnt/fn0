import csv
import json
from pathlib import Path

root = Path(__file__).parent
runs = root / "oci" / "runs"
scenarios = ("rw95", "rw80", "rw50", "rw20", "mt-mix")
fields = (
    "run",
    "mode",
    "scenario",
    "tenant",
    "profile",
    "actual_read_percent",
    "actual_write_percent",
    "api_per_second",
    "successful_write_transactions_per_second",
    "ordinary_gets",
    "preparatory_gets",
    "get_request_count",
    "get_p50_ms",
    "get_p95_ms",
    "get_p99_ms",
    "get_latency_samples",
    "query16_request_count",
    "query16_p50_ms",
    "query16_p95_ms",
    "query16_p99_ms",
    "query16_latency_samples",
    "successful_write_p50_ms",
    "successful_write_p95_ms",
    "successful_write_p99_ms",
    "successful_write_request_count",
    "successful_write_latency_samples",
    "workflow_request_count",
    "workflow_p95_ms",
    "workflow_p99_ms",
    "workflow_latency_samples",
    "condition_rejections",
    "natural_conflicts",
    "overloads",
    "other_errors",
    "cpu_percent_one_core",
    "cpu_percent_host",
    "peak_sampled_rss_mib",
    "rss_sample_count",
    "sampling_completeness",
    "state_validation_passed",
    "reopen_validation_passed",
)
rows = []
for scenario in scenarios:
    pair_inputs = []
    for mode, run_name in (("shared", f"shared-{scenario}"), ("split", f"split-{scenario}")):
        report = json.load(open(runs / run_name / "report.json"))
        configuration = report["configuration"]
        rocksdb = configuration["rocksdb"]
        expected_mode = "shared-8" if mode == "shared" else "split-read-2-write-6"
        assert rocksdb["blocking_admission"] == expected_mode
        assert rocksdb["blocking_operation_limit"] == 8
        assert rocksdb["revision_range_size"] == 4096
        assert configuration["initial_rows_per_tenant"] == 10000
        assert configuration["value_size_bytes"] == 512
        assert configuration["clients"] == 16
        assert configuration["research_metrics_enabled"] is False
        assert configuration["read_stage_metrics_enabled"] is False
        assert all(item["passed"] for item in report["validation"].values())
        assert all(item["passed"] for item in report["reopen_validation"].values())
        inputs = {
            item["client_index"]: (item["profile"], item["fingerprint"], item["workflow_count"])
            for item in report["counters"]["client_input_prefixes"]
        }
        assert len(inputs) == 16
        pair_inputs.append(inputs)
        process = report["process_metrics"]
        assert process["sampling_completeness"] == "complete"
        assert process["rss_sample_count"] == 24
        if scenario == "mt-mix":
            entries = [
                (tenant, value["profile"], value["counters"], value["throughput_per_second"])
                for tenant, value in sorted(report["per_tenant"].items())
            ]
        else:
            entries = [("all", scenario, report["counters"], report["throughput_per_second"])]
        for tenant, profile, counters, throughput in entries:
            latency = counters["latency"]
            rows.append(
                {
                    "run": run_name,
                    "mode": mode,
                    "scenario": scenario,
                    "tenant": tenant,
                    "profile": profile,
                    "actual_read_percent": counters["read_fraction"] * 100,
                    "actual_write_percent": counters["write_fraction"] * 100,
                    "api_per_second": throughput["api_calls"],
                    "successful_write_transactions_per_second": throughput["successful_write_transactions"],
                    "ordinary_gets": counters["ordinary_gets"],
                    "preparatory_gets": counters["preparatory_gets"],
                    "get_request_count": latency["get"]["count"],
                    "get_p50_ms": latency["get"]["p50_ms"],
                    "get_p95_ms": latency["get"]["p95_ms"],
                    "get_p99_ms": latency["get"]["p99_ms"],
                    "get_latency_samples": latency["get"]["sample_count"],
                    "query16_p50_ms": latency["query16"]["p50_ms"],
                    "query16_p95_ms": latency["query16"]["p95_ms"],
                    "query16_p99_ms": latency["query16"]["p99_ms"],
                    "query16_latency_samples": latency["query16"]["sample_count"],
                    "query16_request_count": latency["query16"]["count"],
                    "successful_write_p50_ms": latency["write_transaction_success"]["p50_ms"],
                    "successful_write_p95_ms": latency["write_transaction_success"]["p95_ms"],
                    "successful_write_p99_ms": latency["write_transaction_success"]["p99_ms"],
                    "successful_write_request_count": latency["write_transaction_success"]["count"],
                    "successful_write_latency_samples": latency["write_transaction_success"]["sample_count"],
                    "workflow_request_count": latency["workflow"]["count"],
                    "workflow_p95_ms": latency["workflow"]["p95_ms"],
                    "workflow_p99_ms": latency["workflow"]["p99_ms"],
                    "workflow_latency_samples": latency["workflow"]["sample_count"],
                    "condition_rejections": counters["intentional_condition_rejections"],
                    "natural_conflicts": counters["natural_conflicts"],
                    "overloads": counters["overloads"],
                    "other_errors": counters["other_errors"],
                    "cpu_percent_one_core": process["cpu_percent_of_one_core"],
                    "cpu_percent_host": process["cpu_percent_of_online_machine"],
                    "peak_sampled_rss_mib": process["peak_sampled_rss_kb"] / 1024,
                    "rss_sample_count": process["rss_sample_count"],
                    "sampling_completeness": process["sampling_completeness"],
                    "state_validation_passed": all(item["passed"] for item in report["validation"].values()),
                    "reopen_validation_passed": all(item["passed"] for item in report["reopen_validation"].values()),
                }
            )
    assert pair_inputs[0] == pair_inputs[1], f"paired workload input mismatch in {scenario}"
with open(root / "results.csv", "w", newline="") as output:
    writer = csv.DictWriter(output, fieldnames=fields, lineterminator="\n")
    writer.writeheader()
    writer.writerows(rows)
print(f"rows={len(rows)}")
print("paired_input_fingerprints=matched for 5/5 scenarios")
