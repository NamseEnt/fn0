import json
import statistics
import sys
from pathlib import Path


ROOT = Path(sys.argv[1])
cases = (ROOT / "completed-cases.txt").read_text().splitlines()
results = {"stage": ROOT.name, "cases": [], "medians": {}}
measurements = {}
for case in cases:
    report = json.loads((ROOT / "reports" / f"{case}.json").read_text())
    controller = json.loads((ROOT / "reports" / case / "controller-result.json").read_text())
    counters = report["counters"]
    timing = report["timing"]
    if not report["validation"].get("tenant_1", {}).get("passed") or not report["reopen_validation"].get("tenant_1", {}).get("passed"):
        raise RuntimeError(f"final/reopen validation failed: {case}")
    if timing["measurement_complete"] is not True or timing["accepted_as_full_window"] is not True:
        raise RuntimeError(f"measurement window incomplete: {case}")
    latencies = counters["latency"]
    operation_counts = {
        "query16": counters["query16_calls"],
        "get": counters["ordinary_gets"],
        "put": counters["write_categories"]["put"]["successes"],
        "conditional_put": counters["write_categories"]["conditional_width_1"]["successes"],
    }
    denominator_seconds = timing["actual_admission_ms"] / 1_000
    metric = {
        "api_calls_per_second": counters["api_calls"] / denominator_seconds,
        "operations_per_second": {name: count / denominator_seconds for name, count in operation_counts.items()},
        "operation_counts": operation_counts,
        "query_rows_per_call": counters["query_rows_returned"] / counters["query16_calls"],
        "shares": {name: count / counters["api_calls"] for name, count in operation_counts.items()},
        "latencies_ms": {
            name: {key: value for key, value in latency.items() if key.endswith("_ms")}
            for name, latency in latencies.items()
        },
        "peak_process_group_rss_bytes": controller["peak_process_group_rss_bytes"],
        "minimum_mem_available_bytes": controller["minimum_mem_available_bytes"],
        "maximum_database_bytes": controller["maximum_database_bytes"],
        "reopen_and_validation_ms": report["reopen_and_validation_ms"],
        "database_archive": json.loads((ROOT / "database-archives" / f"{case}.json").read_text()),
        "source_sha": report["git_commit"],
        "binary_sha256": report["binary_sha256"],
        "resource_sample_count": controller["sample_count"],
    }
    result = {"case": case, "backend": case.rsplit("-r", 1)[0], **metric}
    results["cases"].append(result)
    measurements.setdefault(result["backend"], []).append(metric)

for backend, backend_runs in measurements.items():
    numeric_fields = ["api_calls_per_second", "peak_process_group_rss_bytes", "maximum_database_bytes", "reopen_and_validation_ms"]
    backend_summary = {}
    for field in numeric_fields:
        values = [run[field] for run in backend_runs]
        backend_summary[field] = {"median": statistics.median(values), "range": [min(values), max(values)]}
    for operation in ["query16", "get", "put", "conditional_put"]:
        backend_summary.setdefault("operations_per_second", {})[operation] = {
            "median": statistics.median([run["operations_per_second"][operation] for run in backend_runs]),
            "range": [min(run["operations_per_second"][operation] for run in backend_runs), max(run["operations_per_second"][operation] for run in backend_runs)],
        }
        for percentile in ["p50_ms", "p95_ms", "p99_ms"]:
            backend_summary.setdefault("latency_medians_ms", {}).setdefault(operation, {})[percentile] = statistics.median(
                [run["latencies_ms"][{"query16": "query16", "get": "get", "put": "put_success", "conditional_put": "conditional_put_success"}[operation]][percentile] for run in backend_runs]
            )
    results["medians"][backend] = backend_summary

(ROOT / "matrix-summary.json").write_text(json.dumps(results, indent=2) + "\n")
print(json.dumps(results["medians"], indent=2))
