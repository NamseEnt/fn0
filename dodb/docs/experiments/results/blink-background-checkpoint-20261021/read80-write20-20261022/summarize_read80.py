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


def latency_ms(value, percentile):
    return value[f"{percentile}_ms"] if f"{percentile}_ms" in value else value[f"{percentile}_nanos"] / 1_000_000


def summarize(root):
    plan = json.loads((root / "matrix-plan.json").read_text())
    assert digest(root / "controller" / "run_read80.py") == plan["controller_sha256"]
    assert digest(root / "controller" / "run_matrix_base.py") == plan["base_controller_sha256"]
    expected = {
        f"RW80-{backend}-r{repetition}"
        for repetition, paths in plan["path_orders"].items() for backend in paths
    }
    assert set((root / "completed-cases.txt").read_text().splitlines()) == expected
    assert (root / "matrix-complete.txt").exists()
    cells = []
    reference_prefix = None
    for report_path in sorted((root / "reports").glob("*.json")):
        case_id = report_path.stem
        backend, repetition = case_id.removeprefix("RW80-").rsplit("-r", 1)
        evidence = root / "reports" / case_id
        command = json.loads((evidence / "command.json").read_text())
        controller = json.loads((evidence / "controller-result.json").read_text())
        raw_path = evidence / "workload-report.raw.json"
        raw = json.loads(raw_path.read_text())
        report = json.loads(report_path.read_text())
        assert raw["git_commit"] == command["source_sha"] == plan["source_sha"], case_id
        assert raw["binary_sha256"] == command["binary_sha256"] == plan["binary_hashes"]["service_workload"], case_id
        assert digest(raw_path) == (evidence / "workload-report-raw-sha256.txt").read_text().strip(), case_id
        assert digest(report_path) == (evidence / "workload-report-sha256.txt").read_text().strip(), case_id
        assert controller["exit_code"] == 0 and controller["stop_reason"] is None, case_id
        for observed, limit, lower_bound in [
            ("peak_process_group_rss_bytes", "max_process_group_rss_bytes", False),
            ("minimum_mem_available_bytes", "minimum_mem_available_bytes", True),
            ("maximum_database_bytes", "maximum_database_bytes", False),
            ("minimum_filesystem_available_bytes", "minimum_filesystem_available_bytes", True),
        ]:
            assert (controller[observed] >= plan["limits"][limit] if lower_bound else controller[observed] < plan["limits"][limit]), case_id
        assert report["run_status"] == "completed" and report["scenario"] == "rw80", case_id
        assert report["timing"]["measurement_complete"] and report["timing"]["accepted_as_full_window"], case_id
        assert report["configuration"]["clients"] == 16 and report["configuration"]["client_executor"] == "dedicated", case_id
        assert report["configuration"]["measurement_requested_ms"] == plan["measurement_ms"], case_id
        assert all(value["passed"] for value in report["validation"].values()), case_id
        assert all(value["passed"] for value in report["reopen_validation"].values()), case_id
        check_latencies(report)
        prefix = sorted([
            [entry["client_index"], entry["profile"], entry["workflow_count"], entry["fingerprint"]]
            for tenant in report["per_tenant"].values()
            for entry in tenant["counters"]["client_input_prefixes"]
        ])
        assert len(prefix) == 16 and all(entry[1] == "rw80" for entry in prefix), case_id
        if reference_prefix is None:
            reference_prefix = prefix
        assert prefix == reference_prefix, case_id
        counters = report["counters"]
        assert counters["overloads"] == 0 and counters["other_errors"] == 0, case_id
        assert abs(counters["read_fraction"] - 0.8) <= 0.005, case_id
        metrics = report["blink_checkpoint_metrics"]
        window = report["measurement_window_unix_nanos"]
        events = [event for event in metrics.get("checkpoint_events", []) if event["succeeded"]
                  and int(window["start"]) <= int(event["started_unix_nanos"])
                  and int(event["ended_unix_nanos"]) <= int(window["last_client_completion"])]
        if backend != "minimal-native":
            assert not metrics["checkpoints_failed"] and metrics["failed_message"] is None, case_id
            assert not metrics["tasks_running"] and not metrics["retained_image_bytes"], case_id
            assert not metrics["omitted_checkpoint_events"], case_id
            assert sum(metrics["transaction_group_histogram"]) == metrics["transaction_group_count"] == metrics["mutation_commit_groups"], case_id
            argv = command["argv"]
            effective = report["blink_checkpoint_configuration"]
            assert effective["layout"] == argv[argv.index("--blink-wal-layout") + 1], case_id
            assert effective["execution"] == argv[argv.index("--blink-checkpoint-execution") + 1], case_id
            if "background" in backend and events:
                assert sum(event["writes_during_checkpoint"] for event in events) > 0, case_id
        manifest = json.loads((evidence / "database-manifest.json").read_text())
        assert bool(any(entry["path"].endswith(".segments.manifest") for entry in manifest)) == ("segmented" in backend), case_id
        cell = {
            "case": case_id, "backend": backend, "repetition": int(repetition),
            **report["throughput_per_second"],
            "read_fraction": counters["read_fraction"], "write_fraction": counters["write_fraction"],
            "peak_rss_bytes": controller["peak_process_group_rss_bytes"],
            "maximum_wal_bytes": controller["maximum_wal_bytes"],
            "maximum_database_bytes": controller["maximum_database_bytes"],
            "cpu_seconds": report["process_metrics"]["cpu_seconds"],
            "measured_checkpoint_count": len(events),
            "measured_checkpoint_overlap_writes": sum(event["writes_during_checkpoint"] for event in events),
            "checkpoint_total_nanos": metrics.get("checkpoint_total_nanos", 0),
            "coordinator_pause_nanos": metrics.get("coordinator_pause_nanos", 0),
            "capacity_wait_count": metrics.get("capacity_wait_count", 0),
            "outcomes": {name: counters[name] for name in ["write_requests", "successful_write_transactions", "successful_mutations", "intentional_condition_rejections", "natural_conflicts", "overloads", "other_errors"]},
        }
        for category, latency in counters["latency"].items():
            for percentile in ["p50", "p95", "p99"]:
                cell[f"{category}_{percentile}_ms"] = latency_ms(latency, percentile)
        cells.append(cell)
    assert len(cells) == plan["cells"] and {cell["case"] for cell in cells} == expected
    medians = {}
    for backend in sorted({cell["backend"] for cell in cells}):
        values = [cell for cell in cells if cell["backend"] == backend]
        medians[backend] = {
            key: statistics.median(cell[key] for cell in values)
            for key in values[0] if key not in {"case", "backend", "repetition", "outcomes"}
        }
        medians[backend]["api_calls_range"] = [min(cell["api_calls"] for cell in values), max(cell["api_calls"] for cell in values)]
    reference = medians["minimal-native"]
    comparisons = {
        backend: {
            "api_calls_ratio_to_rocksdb": values["api_calls"] / reference["api_calls"],
            "workflow_p99_ratio_to_rocksdb": values["workflow_p99_ms"] / reference["workflow_p99_ms"],
        }
        for backend, values in medians.items()
    }
    result = {"plan": plan, "cells": cells, "medians": medians, "comparisons": comparisons, "validation_passed": True}
    (root / "matrix-summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"cases": len(cells), "medians": medians, "comparisons": comparisons}, indent=2))


if __name__ == "__main__":
    summarize(Path(sys.argv[1]))
