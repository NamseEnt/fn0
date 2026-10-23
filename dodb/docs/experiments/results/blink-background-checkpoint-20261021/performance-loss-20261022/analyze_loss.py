import hashlib
import json
import statistics
import sys
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def check_latencies(value):
    if isinstance(value, dict):
        if "count" in value and "sample_count" in value:
            assert value["count"] == value["sample_count"]
            if "nonzero_bins" in value:
                assert sum(entry[1] for entry in value["nonzero_bins"]) == value["count"]
        for nested in value.values():
            check_latencies(nested)
    elif isinstance(value, list):
        for nested in value:
            check_latencies(nested)


def latency_ms(latency, percentile):
    name = f"{percentile}_ms"
    return latency[name] if name in latency else latency[f"{percentile}_nanos"] / 1_000_000


def histogram_percentile_ms(latency, numerator, denominator):
    count = latency["count"]
    if "p99_nanos" in latency:
        target = max(1, (count * numerator + denominator - 1) // denominator)
    else:
        target = ((count - 1) * numerator + denominator - 1) // denominator + 1
    cumulative = 0
    for bucket_index, bucket_count in latency["nonzero_bins"]:
        cumulative += bucket_count
        if cumulative >= target:
            if bucket_index < 1024:
                upper = bucket_index
            else:
                relative = bucket_index - 1024
                exponent = 10 + relative // 1024
                upper = (1 << exponent) + (1 << (exponent - 10)) * (relative % 1024 + 1) - 1
            return upper / 1_000_000
    raise AssertionError("histogram percentile has no bucket")


def input_prefix(report, workload):
    if workload == "RW80":
        return sorted([
            [entry["client_index"], entry["profile"], entry["workflow_count"], entry["fingerprint"]]
            for tenant in report["per_tenant"].values()
            for entry in tenant["counters"]["client_input_prefixes"]
        ])
    return sorted([
        [entry["client_index"], entry["measurement_input_prefix_batches"], entry["measurement_input_prefix_fingerprint_fnv1a64"]]
        for entry in report["per_client"]
    ])


def analyze(root):
    plan = json.loads((root / "matrix-plan.json").read_text())
    assert digest(root / "controller/run_loss.py") == plan["controller_sha256"]
    assert digest(root / "controller/run_matrix_base.py") == plan["base_controller_sha256"]
    if plan.get("diagnostic_only"):
        assert digest(root / "controller" / plan.get("profiler_controller", "run_profiles.py")) == plan["profiler_controller_sha256"]
    expected = {f"{workload}-{backend}-r{repetition}" for repetition, paths in plan["path_orders"].items()
                for backend in paths for workload in plan["workloads"]}
    assert set((root / "completed-cases.txt").read_text().splitlines()) == expected
    assert (root / "matrix-complete.txt").exists()
    references = {}
    cells = []
    for report_path in sorted((root / "reports").glob("*.json")):
        case_id = report_path.stem
        workload, remaining = case_id.split("-", 1)
        backend, repetition = remaining.rsplit("-r", 1)
        evidence = root / "reports" / case_id
        report = json.loads(report_path.read_text())
        raw_path = evidence / "workload-report.raw.json"
        raw = json.loads(raw_path.read_text())
        assert {key: value for key, value in report.items() if key != "source_sha"} == {key: value for key, value in raw.items() if key != "source_sha"}, case_id
        assert report["source_sha"] == plan["source_sha"], case_id
        command = json.loads((evidence / "command.json").read_text())
        controller = json.loads((evidence / "controller-result.json").read_text())
        binary_name = "service_workload" if workload == "RW80" else "unconditional_write_workload"
        assert raw["git_commit"] == command["source_sha"] == plan["source_sha"], case_id
        assert raw["binary_sha256"] == command["binary_sha256"] == plan["binary_hashes"][binary_name], case_id
        assert digest(raw_path) == (evidence / "workload-report-raw-sha256.txt").read_text().strip(), case_id
        assert digest(report_path) == (evidence / "workload-report-sha256.txt").read_text().strip(), case_id
        assert controller["exit_code"] == 0 and controller["stop_reason"] is None, case_id
        assert controller["peak_process_group_rss_bytes"] < plan["limits"]["max_process_group_rss_bytes"], case_id
        assert controller["maximum_database_bytes"] < plan["limits"]["maximum_database_bytes"], case_id
        assert controller["minimum_mem_available_bytes"] >= plan["limits"]["minimum_mem_available_bytes"], case_id
        assert controller["minimum_filesystem_available_bytes"] >= plan["limits"]["minimum_filesystem_available_bytes"], case_id
        assert report["run_status"] == "completed" and report["timing"]["measurement_complete"], case_id
        assert report["timing"].get("accepted_as_full_window", True), case_id
        check_latencies(report)
        signature = input_prefix(report, workload)
        assert len(signature) == (16 if workload == "RW80" else 64), case_id
        references.setdefault(workload, signature)
        assert signature == references[workload], case_id
        counters = report["counters"]
        assert counters["overloads"] == 0 and counters["other_errors"] == 0, case_id
        argv = command["argv"]
        assert int(argv[argv.index("--measurement-ms") + 1]) == plan["measurement_ms"], case_id
        if workload == "RW80":
            assert report["scenario"] == "rw80" and abs(counters["read_fraction"] - 0.8) <= 0.005, case_id
            assert all(value["passed"] for value in report["validation"].values()), case_id
            assert all(value["passed"] for value in report["reopen_validation"].values()), case_id
            throughput = report["throughput_per_second"]["api_calls"]
            latency = counters["latency"]["workflow"]
        else:
            assert report["final_validation"]["passed"] and report["reopen_validation"]["passed"], case_id
            assert argv[argv.index("--width") + 1] == "16", case_id
            throughput = report["timing"]["write_batches_per_second"]
            latency = report["latency"]
        metrics = report["blink_checkpoint_metrics"]
        window = report["measurement_window_unix_nanos"]
        events = [event for event in metrics.get("checkpoint_events", []) if event["succeeded"]
                  and int(window["start"]) <= int(event["started_unix_nanos"])
                  and int(event["ended_unix_nanos"]) <= int(window["last_client_completion"])]
        appended = None
        bytes_per_transaction = None
        transactions_per_group = None
        if backend != "minimal-native":
            assert not metrics["checkpoints_failed"] and not metrics["failed_message"], case_id
            assert not metrics["tasks_running"] and not metrics["retained_image_bytes"], case_id
            assert not metrics["omitted_checkpoint_events"], case_id
            assert sum(metrics["transaction_group_histogram"]) == metrics["transaction_group_count"] == metrics["mutation_commit_groups"], case_id
            if backend == "legacy-off":
                assert not metrics["checkpoints_started"], case_id
            elif not plan["smoke"]:
                assert len(events) >= 1, case_id
            reclaimed = sum(event["reclaimed_bytes"] for event in metrics["checkpoint_events"])
            reinitialized = sum(event["wal_bytes_after"] for event in metrics["checkpoint_events"])
            appended = metrics["wal_bytes_at_measurement_end"] - metrics["wal_bytes_at_measurement_start"] + reclaimed + reinitialized
            bytes_per_transaction = appended / metrics["successful_mutation_transactions"]
            transactions_per_group = metrics["successful_mutation_transactions"] / metrics["mutation_commit_groups"]
        total_seconds = report["timing"].get("throughput_denominator_seconds", report["timing"].get("total_elapsed_ms", 0) / 1000)
        if not total_seconds:
            total_seconds = report["timing"]["admission_plus_final_completion_ms"] / 1000
        checkpoint_seconds = metrics.get("checkpoint_total_nanos", 0) / 1_000_000_000
        assert abs(histogram_percentile_ms(latency, 99, 100) - latency_ms(latency, "p99")) < 0.000001, case_id
        cell = {
            "case": case_id, "workload": workload, "backend": backend, "repetition": int(repetition),
            "throughput_per_second": throughput,
            **{f"{percentile}_ms": latency_ms(latency, percentile) for percentile in ["p50", "p95", "p99"]},
            "p999_ms": histogram_percentile_ms(latency, 999, 1000),
            "maximum_ms": latency["maximum_nanos"] / 1_000_000 if "maximum_nanos" in latency else histogram_percentile_ms(latency, 1, 1),
            "wal_appended_bytes": appended, "wal_bytes_per_transaction": bytes_per_transaction,
            "transactions_per_group": transactions_per_group,
            "checkpoint_seconds": checkpoint_seconds, "measured_checkpoint_count": len(events),
            "total_seconds": total_seconds,
            "checkpoint_fraction": checkpoint_seconds / total_seconds,
            "throughput_excluding_checkpoint_wall": throughput * total_seconds / (total_seconds - checkpoint_seconds),
            "cpu_seconds": report["process_metrics"]["cpu_seconds"],
            "peak_rss_bytes": controller["peak_process_group_rss_bytes"],
            "maximum_wal_bytes": controller["maximum_wal_bytes"],
            "maximum_database_bytes": controller["maximum_database_bytes"],
            "read_fraction": counters.get("read_fraction"),
            "overloads": counters["overloads"], "other_errors": counters["other_errors"],
        }
        if workload == "RW80":
            for category in ["get", "query16", "write_transaction_success"]:
                cell[f"{category}_p99_ms"] = latency_ms(counters["latency"][category], "p99")
        cells.append(cell)
    assert len(cells) == plan["cells"] and {cell["case"] for cell in cells} == expected
    medians = {}
    for workload in plan["workloads"]:
        medians[workload] = {}
        for backend in sorted({cell["backend"] for cell in cells}):
            values = [cell for cell in cells if cell["workload"] == workload and cell["backend"] == backend]
            medians[workload][backend] = {key: statistics.median(value[key] for value in values)
                                          for key in values[0] if key not in {"case", "workload", "backend", "repetition"}
                                          and values[0][key] is not None}
            medians[workload][backend]["throughput_range"] = [min(value["throughput_per_second"] for value in values), max(value["throughput_per_second"] for value in values)]
    result = {"source_sha": plan["source_sha"], "plan": plan, "cells": cells, "medians": medians, "validation_passed": True}
    (root / "matrix-summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"cells": len(cells), "medians": medians}, indent=2))


if __name__ == "__main__":
    analyze(Path(sys.argv[1]))
