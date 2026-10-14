import csv
import json
from pathlib import Path

root = Path(__file__).parent / "oci" / "runs"
run_names = ["rw95-on", "rw80-on", "rw50-on", "rw20-on", "mt-mix-on", "rw95-off"]
read_kinds = ["get", "preparatory_get", "query16"]
metric_names = [
    "request_to_permit_acquire_start",
    "semaphore_wait",
    "blocking_dispatch_wait",
    "blocking_closure_duration",
    "blocking_completion_handoff",
    "tenant_database_lookup",
    "rocksdb_get_call",
    "get_copy_decode",
    "query_snapshot_iterator_setup",
    "query_iterator_calls_sum",
    "query_copy_decode_budget",
    "response_processing",
    "service_request_total",
]

condition_fields = [
    "run",
    "scenario",
    "tenant",
    "profile",
    "api_calls",
    "api_per_second",
    "read_fraction",
    "write_fraction",
    "ordinary_gets",
    "preparatory_gets",
    "query16_calls",
    "successful_write_transactions",
    "condition_rejections",
    "overloads",
    "other_errors",
    "get_sample_count",
    "get_p50_ms",
    "get_p95_ms",
    "get_p99_ms",
    "query16_sample_count",
    "query16_p50_ms",
    "query16_p95_ms",
    "query16_p99_ms",
    "validation_passed",
    "reopen_validation_passed",
]
with (Path(__file__).parent / "condition-summary.csv").open("w", newline="") as output_file:
    writer = csv.DictWriter(output_file, fieldnames=condition_fields, lineterminator="\n")
    writer.writeheader()
    for run_name in run_names:
        report = json.loads((root / run_name / "report.json").read_text())
        for tenant_name, tenant in report["per_tenant"].items():
            counters = tenant["counters"]
            get_latency = counters["latency"]["get"]
            query_latency = counters["latency"]["query16"]
            writer.writerow({
                "run": run_name,
                "scenario": report["scenario"],
                "tenant": tenant_name,
                "profile": tenant["profile"],
                "api_calls": counters["api_calls"],
                "api_per_second": tenant["throughput_per_second"]["api_calls"],
                "read_fraction": counters["read_fraction"],
                "write_fraction": counters["write_fraction"],
                "ordinary_gets": counters["ordinary_gets"],
                "preparatory_gets": counters["preparatory_gets"],
                "query16_calls": counters["query16_calls"],
                "successful_write_transactions": counters["successful_write_transactions"],
                "condition_rejections": counters["intentional_condition_rejections"],
                "overloads": counters["overloads"],
                "other_errors": counters["other_errors"],
                "get_sample_count": get_latency["sample_count"],
                "get_p50_ms": get_latency["p50_ms"],
                "get_p95_ms": get_latency["p95_ms"],
                "get_p99_ms": get_latency["p99_ms"],
                "query16_sample_count": query_latency["sample_count"],
                "query16_p50_ms": query_latency["p50_ms"],
                "query16_p95_ms": query_latency["p95_ms"],
                "query16_p99_ms": query_latency["p99_ms"],
                "validation_passed": all(item["passed"] for item in report["validation"].values()),
                "reopen_validation_passed": all(item["passed"] for item in report["reopen_validation"].values()),
            })

stage_fields = [
    "run",
    "scenario",
    "tenant",
    "profile",
    "read_kind",
    "request_count",
    "driver_latency_sample_count",
    "driver_p50_ms",
    "driver_p95_ms",
    "driver_p99_ms",
    "metric",
    "metric_sample_count",
    "mean_ms",
    "p95_ms_approx",
    "p99_ms_approx",
]
with (Path(__file__).parent / "stage-metrics.csv").open("w", newline="") as output_file:
    writer = csv.DictWriter(output_file, fieldnames=stage_fields, lineterminator="\n")
    writer.writeheader()
    for run_name in run_names:
        report = json.loads((root / run_name / "report.json").read_text())
        for tenant_name, tenant in report["per_tenant"].items():
            counters = tenant["counters"]
            for read_kind in read_kinds:
                driver_latency = counters["latency"][read_kind]
                stage_metrics = report.get("rocksdb_research_metrics", {}).get("tenants", {}).get(tenant_name, {}).get("read_stages", {}).get("measurement", {}).get(read_kind, {})
                for metric_name in metric_names:
                    metric = stage_metrics.get(metric_name, {})
                    writer.writerow({
                        "run": run_name,
                        "scenario": report["scenario"],
                        "tenant": tenant_name,
                        "profile": tenant["profile"],
                        "read_kind": read_kind,
                        "request_count": driver_latency["count"],
                        "driver_latency_sample_count": driver_latency["sample_count"],
                        "driver_p50_ms": driver_latency["p50_ms"],
                        "driver_p95_ms": driver_latency["p95_ms"],
                        "driver_p99_ms": driver_latency["p99_ms"],
                        "metric": metric_name,
                        "metric_sample_count": metric.get("count", 0),
                        "mean_ms": metric.get("mean_ms"),
                        "p95_ms_approx": metric.get("p95_ms_approx"),
                        "p99_ms_approx": metric.get("p99_ms_approx"),
                    })
