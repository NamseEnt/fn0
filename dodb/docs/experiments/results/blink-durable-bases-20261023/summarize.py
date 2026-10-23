import hashlib
import importlib.util
import json
import math
import os
import statistics
import sys
from pathlib import Path


HELPER = Path(os.environ.get("DODB_BASE_ACCOUNTING_HELPER", str(Path(__file__).parent.parent / "blink-background-checkpoint-20261021/performance-loss-20261022/analyze_loss.py")))
SPEC = importlib.util.spec_from_file_location("loss_accounting", HELPER)
HELPERS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HELPERS)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def summarize(root):
    plan = json.loads((root / "matrix-plan.json").read_text())
    expected = {f"{workload}-{policy}-r{repetition}" for repetition, order in plan["orders"].items()
                for policy in order for workload in plan["workloads"]}
    assert set((root / "completed-cases.txt").read_text().splitlines()) == expected
    assert (root / "matrix-complete.txt").exists()
    cells = []
    references = {}
    for case in sorted(expected):
        report_path = root / "reports" / f"{case}.json"
        evidence = root / "reports" / case
        raw_path = evidence / "workload-report.raw.json"
        report = json.loads(report_path.read_text())
        raw = json.loads(raw_path.read_text())
        command = json.loads((evidence / "command.json").read_text())
        controller = json.loads((evidence / "controller-result.json").read_text())
        samples = json.loads((evidence / "resource-samples.json").read_text())
        workload, remaining = case.split("-", 1)
        policy, repetition = remaining.rsplit("-r", 1)
        binary = "service_workload" if workload == "RW80" else "unconditional_write_workload"
        assert raw["git_commit"] == command["source_sha"] == plan["source_sha"], case
        assert raw["binary_sha256"] == command["binary_sha256"] == plan["binary_hashes"][binary], case
        assert digest(raw_path) == (evidence / "workload-report-raw-sha256.txt").read_text().strip(), case
        assert digest(report_path) == (evidence / "workload-report-sha256.txt").read_text().strip(), case
        assert {key: value for key, value in raw.items() if key != "source_sha"} == {key: value for key, value in report.items() if key != "source_sha"}, case
        assert controller["exit_code"] == 0 and controller["stop_reason"] is None, case
        assert controller["peak_process_group_rss_bytes"] < plan["limits"]["rss_bytes"], case
        assert controller["maximum_database_bytes"] < plan["limits"]["database_bytes"], case
        assert controller["minimum_mem_available_bytes"] >= plan["limits"]["minimum_memory_bytes"], case
        assert controller["minimum_filesystem_available_bytes"] >= plan["limits"]["minimum_free_disk_bytes"], case
        envelope = json.loads((evidence / "recovery-envelope.json").read_text())
        assert envelope["extra_limit_bytes"] == plan["limits"]["recovery_extra_bytes"], case
        assert max(sample["recovery_file_bytes"] for sample in samples) <= envelope["required_recovery_envelope_bytes"], case
        assert report["run_status"] == "completed" and report["timing"]["measurement_complete"], case
        assert report["timing"].get("accepted_as_full_window", True), case
        HELPERS.check_latencies(report)
        signature = HELPERS.input_prefix(report, workload)
        assert len(signature) == (16 if workload == "RW80" else 64), case
        references.setdefault(workload, signature)
        assert references[workload] == signature, case
        counters = report["counters"]
        assert counters["overloads"] == 0 and counters["other_errors"] == 0, case
        if workload == "RW80":
            assert report["scenario"] == "rw80" and abs(counters["read_fraction"] - 0.8) <= 0.005, case
            assert all(value["passed"] for value in report["validation"].values()), case
            assert all(value["passed"] for value in report["reopen_validation"].values()), case
            throughput = report["throughput_per_second"]["api_calls"]
            latency = counters["latency"]["workflow"]
        else:
            assert report["final_validation"]["passed"] and report["reopen_validation"]["passed"], case
            assert report["configuration"]["clients"] == 64 and report["configuration"]["width"] == 16, case
            throughput = report["timing"]["write_batches_per_second"]
            latency = report["latency"]
        metrics = report["blink_checkpoint_metrics"]
        assert not metrics["checkpoints_failed"] and not metrics["failed_message"], case
        assert not metrics["tasks_running"] and not metrics["retained_image_bytes"], case
        assert not metrics["omitted_checkpoint_events"], case
        assert sum(metrics["transaction_group_histogram"]) == metrics["transaction_group_count"] == metrics["mutation_commit_groups"], case
        window = report["measurement_window_unix_nanos"]
        events = [event for event in metrics["checkpoint_events"] if event["succeeded"]
                  and int(window["start"]) <= int(event["started_unix_nanos"])
                  and int(event["ended_unix_nanos"]) <= int(window["last_client_completion"])]
        assert len(events) >= plan["minimum_checkpoints"], case
        if "background" in policy and not plan["smoke"]:
            assert sum(event["writes_during_checkpoint"] for event in events) > 0, case
        transactions = metrics["successful_mutation_transactions"]
        wal_bytes = metrics["wal_bytes_at_measurement_end"] - metrics["wal_bytes_at_measurement_start"] + sum(event["reclaimed_bytes"] for event in metrics["checkpoint_events"])
        checkpoint_bytes = sum(event["checkpoint_bytes_written"] for event in metrics["checkpoint_events"])
        process_io = report["process_write_io"]
        assert process_io["before"]["available"] and process_io["after"]["available"], case
        request_bytes = process_io["after"]["wchar"] - process_io["before"]["wchar"]
        assert request_bytes >= wal_bytes + checkpoint_bytes, case
        cell = {"case": case, "workload": workload, "policy": policy, "repetition": int(repetition),
                "throughput_per_second": throughput,
                **{f"{percentile}_ms": HELPERS.latency_ms(latency, percentile) for percentile in ["p50", "p95", "p99"]},
                "p999_ms": HELPERS.histogram_percentile_ms(latency, 999, 1000),
                "maximum_ms": latency["maximum_nanos"] / 1000000 if "maximum_nanos" in latency else HELPERS.histogram_percentile_ms(latency, 1, 1),
                "mutation_transactions": transactions, "wal_appended_bytes": wal_bytes,
                "wal_bytes_per_transaction": wal_bytes / transactions,
                "checkpoint_bytes": checkpoint_bytes, "checkpoint_bytes_per_transaction": checkpoint_bytes / transactions,
                "write_request_bytes": request_bytes, "write_request_bytes_per_transaction": request_bytes / transactions,
                "write_syscalls": process_io["after"]["syscw"] - process_io["before"]["syscw"],
                "full_base_snapshots": sum(event["full_base_snapshots"] for event in metrics["checkpoint_events"]),
                "base_bytes_written": sum(event["base_bytes_written"] for event in metrics["checkpoint_events"]),
                "checkpoint_write_requests": sum(event["checkpoint_write_requests"] for event in metrics["checkpoint_events"]),
                "checkpoint_seconds": metrics["checkpoint_total_nanos"] / 1000000000,
                "coordinator_pause_seconds": metrics["coordinator_pause_nanos"] / 1000000000,
                "measured_checkpoint_count": len(events),
                "checkpoint_overlap_writes": sum(event["writes_during_checkpoint"] for event in events),
                "transactions_per_group": transactions / metrics["mutation_commit_groups"],
                "capacity_wait_count": metrics["capacity_wait_count"],
                "peak_rss_bytes": controller["peak_process_group_rss_bytes"],
                "maximum_wal_bytes": controller["maximum_wal_bytes"],
                "maximum_base_bytes": max(sample["page_base_bytes"] for sample in samples),
                "maximum_recovery_file_bytes": max(sample["recovery_file_bytes"] for sample in samples),
                "live_checkpoint_image_bytes": envelope["live_checkpoint_image_bytes"],
                "required_recovery_envelope_bytes": envelope["required_recovery_envelope_bytes"],
                "maximum_database_bytes": controller["maximum_database_bytes"],
                "reopen_and_validation_ms": report["reopen_and_validation_ms"]}
        cells.append(cell)
    medians = {}
    excluded = {"case", "workload", "policy", "repetition"}
    for workload in plan["workloads"]:
        medians[workload] = {}
        for policy in sorted({cell["policy"] for cell in cells}):
            selected = [cell for cell in cells if cell["workload"] == workload and cell["policy"] == policy]
            medians[workload][policy] = {key: statistics.median(cell[key] for cell in selected) for key in selected[0] if key not in excluded}
            medians[workload][policy]["throughput_range"] = [min(cell["throughput_per_second"] for cell in selected), max(cell["throughput_per_second"] for cell in selected)]
            medians[workload][policy]["worst_individual_maximum_ms"] = max(cell["maximum_ms"] for cell in selected)
    candidates = [policy for policy in medians["E"] if policy != "segmented-sync"]
    scores = {policy: math.sqrt(math.prod(medians[workload][policy]["throughput_per_second"] / medians[workload]["segmented-sync"]["throughput_per_second"] for workload in plan["workloads"])) for policy in candidates}
    selected = max(candidates, key=lambda policy: scores[policy])
    ties = [policy for policy in candidates if scores[policy] >= scores[selected] - 0.01]
    selected = min(ties, key=lambda policy: medians["E"][policy]["p999_ms"])
    ratios = {workload: {key: medians[workload][selected][key] / medians[workload]["segmented-sync"][key]
                        for key in ["throughput_per_second", "wal_bytes_per_transaction", "write_request_bytes_per_transaction", "p999_ms", "maximum_ms", "reopen_and_validation_ms"]}
              for workload in plan["workloads"]}
    adoption = (not plan["smoke"] and plan["duration_ms"] >= 300000
                and ratios["E"]["throughput_per_second"] >= 1.10
                and ratios["RW80"]["throughput_per_second"] >= 0.97
                and ratios["E"]["wal_bytes_per_transaction"] <= 0.70
                and ratios["E"]["write_request_bytes_per_transaction"] <= 0.80
                and all(value["p999_ms"] <= 1.10 and value["maximum_ms"] <= 1.10 and value["reopen_and_validation_ms"] <= 1.25 for value in ratios.values()))
    summary = {"source_sha": plan["source_sha"], "validated": True, "duration_ms": plan["duration_ms"], "smoke": plan["smoke"],
               "cells": cells, "medians": medians, "selected_candidate": selected, "selection_scores": scores,
               "ratios": ratios, "adoption_gate_passed": adoption,
               "write_accounting_scope": "process write-request bytes, including metadata; not physical-device bytes"}
    (root / "matrix-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps({key: value for key, value in summary.items() if key != "cells"}, indent=2))


if __name__ == "__main__":
    summarize(Path(sys.argv[1]))
