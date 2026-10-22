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


def prefix(report, workload):
    if workload == "A":
        return sorted([
            [entry["client_index"], entry["profile"], entry["workflow_count"], entry["fingerprint"]]
            for tenant in report["per_tenant"].values()
            for entry in tenant["counters"]["client_input_prefixes"]
        ])
    return sorted([
        [entry["client_index"], entry["measurement_input_prefix_batches"], entry["measurement_input_prefix_fingerprint_fnv1a64"]]
        for entry in report["per_client"]
    ])


def summarize(root):
    plan = json.loads((root / "matrix-plan.json").read_text())
    binaries = json.loads((root / "binary-hashes.json").read_text())
    cells = []
    prefixes = {}
    for report_path in sorted((root / "reports").glob("*.json")):
        case = report_path.stem
        workload, remaining = case.split("-", 1)
        backend, repetition = remaining.rsplit("-r", 1)
        evidence = root / "reports" / case
        controller = json.loads((evidence / "controller-result.json").read_text())
        command = json.loads((evidence / "command.json").read_text())
        report = json.loads(report_path.read_text())
        raw_path = evidence / "workload-report.raw.json"
        raw = json.loads(raw_path.read_text())
        binary_name = "service_workload" if workload == "A" else "unconditional_write_workload"
        assert raw["git_commit"] == command["source_sha"] == plan["source_sha"], case
        assert raw["binary_sha256"] == command["binary_sha256"] == binaries[binary_name], case
        assert digest(raw_path) == (evidence / "workload-report-raw-sha256.txt").read_text().strip(), case
        assert digest(report_path) == (evidence / "workload-report-sha256.txt").read_text().strip(), case
        assert controller["exit_code"] == 0 and controller["stop_reason"] is None, case
        limits = plan["limits"]
        assert controller["peak_process_group_rss_bytes"] < limits["max_process_group_rss_bytes"], case
        assert controller["minimum_mem_available_bytes"] >= limits["minimum_mem_available_bytes"], case
        assert controller["maximum_database_bytes"] < limits["maximum_database_bytes"], case
        assert controller["minimum_filesystem_available_bytes"] >= limits["minimum_filesystem_available_bytes"], case
        assert report["run_status"] == "completed", case
        assert report["timing"]["measurement_complete"], case
        assert report["timing"].get("accepted_as_full_window", True), case
        check_latencies(report)
        signature = prefix(report, workload)
        if workload in prefixes:
            assert prefixes[workload] == signature, case
        prefixes[workload] = signature
        metrics = report.get("blink_checkpoint_metrics", {})
        manifest = json.loads((evidence / "database-manifest.json").read_text())
        if "segmented" in backend:
            assert any(entry["path"].endswith(".segments.manifest") for entry in manifest), case
            assert metrics["peak_tasks_running"] == 1, case
        elif backend.startswith("legacy"):
            assert not any(entry["path"].endswith(".segments.manifest") for entry in manifest), case
        start = int(report["measurement_window_unix_nanos"]["start"])
        end = int(report["measurement_window_unix_nanos"]["last_client_completion"])
        events = [event for event in metrics.get("checkpoint_events", []) if event["succeeded"]
                  and start <= int(event["started_unix_nanos"])
                  and int(event["ended_unix_nanos"]) <= end]
        if backend != "minimal-native":
            assert metrics["checkpoints_failed"] == 0 and metrics["failed_message"] is None, case
            assert metrics["tasks_running"] == 0 and metrics["retained_image_bytes"] == 0, case
            assert metrics["omitted_checkpoint_events"] == 0, case
            assert sum(metrics["transaction_group_histogram"]) == metrics["transaction_group_count"] == metrics["mutation_commit_groups"], case
            if not plan["smoke"]:
                assert len(events) >= 3, case
                if "background" in backend:
                    assert sum(event["writes_during_checkpoint"] for event in events) > 0, case
        if workload == "A":
            tenant = next(iter(report["per_tenant"].values()))
            counters = tenant["counters"]
            latency = counters["latency"]["workflow"]
            throughput = report["throughput_per_second"]["api_calls"]
            assert all(result["passed"] for result in report["validation"].values()), case
            assert all(result["passed"] for result in report["reopen_validation"].values()), case
        else:
            counters = report["counters"]
            latency = report["latency"]
            throughput = report["timing"]["write_batches_per_second"]
            assert report["final_validation"]["passed"] and report["reopen_validation"]["passed"], case
        assert counters["overloads"] == 0 and counters["other_errors"] == 0, case
        cells.append({
            "case": case, "workload": workload, "backend": backend, "repetition": int(repetition),
            "throughput_per_second": throughput,
            **{f"{percentile}_ms": latency[f"{percentile}_ms"] if f"{percentile}_ms" in latency else latency[f"{percentile}_nanos"] / 1_000_000 for percentile in ["p50", "p95", "p99"]},
            "measured_checkpoint_count": len(events),
            "measured_checkpoint_overlap_writes": sum(event["writes_during_checkpoint"] for event in events),
            "writes_during_checkpoint": metrics.get("writes_during_checkpoint", 0),
            "peak_retained_image_bytes": metrics.get("peak_retained_image_bytes", 0),
            "capacity_wait_count": metrics.get("capacity_wait_count", 0),
            "capacity_wait_nanos": metrics.get("capacity_wait_nanos", 0),
            "coordinator_pause_nanos": metrics.get("coordinator_pause_nanos", 0),
            "mutation_transactions": metrics.get("successful_mutation_transactions"),
            "mutation_commit_groups": metrics.get("mutation_commit_groups"),
            "mutation_path_wal_syncs": metrics.get("commit_wal_syncs"),
            "checkpoint_path_wal_syncs": metrics.get("checkpoint_wal_syncs"),
            "mutation_transactions_per_group": (metrics["successful_mutation_transactions"] / metrics["mutation_commit_groups"]) if metrics.get("mutation_commit_groups", 0) else None,
            "checkpoint_total_nanos": metrics.get("checkpoint_total_nanos", 0),
            "cpu_seconds": report.get("process_metrics", {}).get("cpu_seconds"),
            "peak_rss_bytes": controller["peak_process_group_rss_bytes"],
            "maximum_wal_bytes": controller["maximum_wal_bytes"],
            "maximum_database_bytes": controller["maximum_database_bytes"],
            "outcomes": {name: counters[name] for name in ["successful_mutations", "successful_write_transactions", "intentional_condition_rejections", "natural_conflicts", "overloads", "other_errors", "measurement_write_batches_acknowledged"] if name in counters},
            "source_sha": raw["git_commit"], "binary_sha256": raw["binary_sha256"],
        })
    assert len(cells) == plan["cells"], (len(cells), plan["cells"])
    expected = {f"{workload}-{backend}-r{repetition}" for repetition, paths in plan["path_orders"].items() for backend in paths for workload in ["A", "E"]}
    assert {cell["case"] for cell in cells} == expected
    assert (root / "matrix-complete.txt").exists()
    medians = {}
    for workload in ["A", "E"]:
        medians[workload] = {}
        for backend in sorted({cell["backend"] for cell in cells}):
            values = [cell for cell in cells if cell["workload"] == workload and cell["backend"] == backend]
            medians[workload][backend] = {
                key: statistics.median(cell[key] for cell in values)
                for key in ["throughput_per_second", "p50_ms", "p95_ms", "p99_ms", "peak_rss_bytes", "maximum_wal_bytes", "checkpoint_total_nanos", "coordinator_pause_nanos", "measured_checkpoint_overlap_writes"]
            }
            medians[workload][backend]["throughput_range"] = [
                min(cell["throughput_per_second"] for cell in values),
                max(cell["throughput_per_second"] for cell in values),
            ]
    gate = {}
    for workload, values in medians.items():
        synchronous = values["segmented-sync-128m"]
        background = values["segmented-background-128m"]
        gate[workload] = {
            "throughput_ratio_background_over_sync": background["throughput_per_second"] / synchronous["throughput_per_second"],
            "p99_ratio_background_over_sync": background["p99_ms"] / synchronous["p99_ms"],
        }
    gate["passed"] = not plan["smoke"] and all(
        value["throughput_ratio_background_over_sync"] >= 1.10 and value["p99_ratio_background_over_sync"] <= 1.0
        for value in list(gate.values())
    )
    result = {"source_sha": plan["source_sha"], "cells": cells, "medians": medians,
              "long_followup_gate": gate, "validation_passed": True, "smoke": plan["smoke"]}
    (root / "matrix-summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"medians": medians, "long_followup_gate": gate}, indent=2))


if __name__ == "__main__":
    summarize(Path(sys.argv[1]))
