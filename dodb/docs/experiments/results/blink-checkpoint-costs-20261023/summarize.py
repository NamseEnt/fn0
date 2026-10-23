import hashlib
import importlib.util
import json
import os
import statistics
import sys
from pathlib import Path


HELPER = Path(os.environ.get("DODB_COST_ACCOUNTING_HELPER", str(
    Path(__file__).parent.parent / "blink-background-checkpoint-20261021/performance-loss-20261022/analyze_loss.py")))
SPEC = importlib.util.spec_from_file_location("checkpoint_accounting", HELPER)
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def summarize(root):
    plan = json.loads((root / "matrix-plan.json").read_text())
    expected = {f"{workload}-{policy}-r{repetition}" for repetition, policies in plan["orders"].items()
                for policy in policies for workload in plan["workloads"]}
    assert (root / "matrix-complete.txt").exists()
    assert set((root / "completed-cases.txt").read_text().splitlines()) == expected
    cells, references, command_references = [], {}, {}
    for case in sorted(expected):
        workload, remainder = case.split("-", 1)
        policy, repetition = remainder.rsplit("-r", 1)
        report_path, evidence = root / "reports" / f"{case}.json", root / "reports" / case
        report = json.loads(report_path.read_text())
        raw_path = evidence / "workload-report.raw.json"
        raw = json.loads(raw_path.read_text())
        command = json.loads((evidence / "command.json").read_text())
        arguments = command["argv"][1:]
        assert len(arguments) % 2 == 0, case
        configuration = dict(zip(arguments[::2], arguments[1::2]))
        assert int(configuration["--measurement-ms"]) == plan["duration_ms"], case
        assert int(configuration["--warmup-ms"]) == plan["warmup_ms"], case
        excluded_arguments = {"--data-dir", "--report", "--backend", "--blink-checkpoint-wal-bytes",
                              "--blink-wal-layout", "--blink-checkpoint-execution", "--blink-wal-segment-bytes"}
        normalized_command = {key: value for key, value in configuration.items() if key not in excluded_arguments}
        command_references.setdefault(workload, normalized_command)
        assert normalized_command == command_references[workload], case
        if policy != "minimal-native":
            assert configuration["--backend"] == "blink"
            assert configuration["--blink-checkpoint-wal-bytes"] == "134217728"
            assert configuration["--blink-wal-layout"] == "segmented-bases-coalesced"
            assert configuration["--blink-checkpoint-execution"] == "background"
            assert configuration["--blink-wal-segment-bytes"] == "33554432"
        else:
            assert configuration["--backend"] == "minimal-native"
        controller = json.loads((evidence / "controller-result.json").read_text())
        samples = json.loads((evidence / "resource-samples.json").read_text())
        identity = plan["identities"][policy]
        binary = "service_workload" if workload == "RW80" else "unconditional_write_workload"
        assert raw["git_commit"] == report["source_sha"] == command["source_sha"] == identity["source_sha"], case
        assert raw["binary_sha256"] == command["binary_sha256"] == identity["binaries"][binary], case
        assert digest(raw_path) == (evidence / "workload-report-raw-sha256.txt").read_text().strip(), case
        assert digest(report_path) == (evidence / "workload-report-sha256.txt").read_text().strip(), case
        assert {key: value for key, value in raw.items() if key != "source_sha"} == {
            key: value for key, value in report.items() if key != "source_sha"}, case
        assert controller["exit_code"] == 0 and controller["stop_reason"] is None, case
        assert controller["peak_process_group_rss_bytes"] < plan["rss_limit_bytes"], case
        assert controller["maximum_database_bytes"] < plan["disk_limit_bytes"], case
        assert controller["minimum_mem_available_bytes"] >= plan["minimum_memory_bytes"], case
        assert controller["minimum_filesystem_available_bytes"] >= plan["minimum_disk_bytes"], case
        assert report["run_status"] == "completed" and report["timing"]["measurement_complete"], case
        assert report["timing"].get("accepted_as_full_window", True), case
        BASE.check_latencies(report)
        signature = BASE.input_prefix(report, workload)
        assert len(signature) == (16 if workload == "RW80" else 64), case
        references.setdefault(workload, signature)
        assert signature == references[workload], case
        counters = report["counters"]
        assert counters["overloads"] == 0 and counters["other_errors"] == 0, case
        if workload == "RW80":
            assert report["scenario"] == "rw80" and abs(counters["read_fraction"] - 0.8) <= 0.005, case
            assert all(value["passed"] for value in report["validation"].values()), case
            assert all(value["passed"] for value in report["reopen_validation"].values()), case
            throughput, latency = report["throughput_per_second"]["api_calls"], counters["latency"]["workflow"]
        else:
            assert report["final_validation"]["passed"] and report["reopen_validation"]["passed"], case
            assert report["configuration"]["clients"] == 64 and report["configuration"]["width"] == 16, case
            throughput, latency = report["timing"]["write_batches_per_second"], report["latency"]
        cell = {"case": case, "policy": policy, "workload": workload, "repetition": int(repetition),
                "throughput_per_second": throughput,
                **{f"{percentile}_ms": BASE.latency_ms(latency, percentile) for percentile in ["p50", "p95", "p99"]},
                "p999_ms": BASE.histogram_percentile_ms(latency, 999, 1000),
                "maximum_ms": latency["maximum_nanos"] / 1000000 if "maximum_nanos" in latency else BASE.histogram_percentile_ms(latency, 1, 1),
                "reopen_and_validation_ms": report["reopen_and_validation_ms"],
                "peak_rss_bytes": controller["peak_process_group_rss_bytes"],
                "maximum_database_bytes": controller["maximum_database_bytes"],
                "maximum_recovery_file_bytes": max(sample["recovery_file_bytes"] for sample in samples),
                "maximum_wal_bytes": controller["maximum_wal_bytes"],
                "cpu_seconds": report["process_metrics"]["cpu_seconds"],
                "resource_samples": len(samples),
                "maximum_resource_sample_gap_seconds": max(later["elapsed_seconds"] - earlier["elapsed_seconds"]
                    for earlier, later in zip(samples, samples[1:]))}
        io = report["process_write_io"]
        assert io["before"]["available"] and io["after"]["available"], case
        cell["process_write_request_bytes"] = io["after"]["wchar"] - io["before"]["wchar"]
        if policy != "minimal-native":
            metrics = report["blink_checkpoint_metrics"]
            assert not metrics["checkpoints_failed"] and not metrics["failed_message"], case
            assert not metrics["tasks_running"] and not metrics["retained_image_bytes"] and not metrics["omitted_checkpoint_events"], case
            assert sum(metrics["transaction_group_histogram"]) == metrics["transaction_group_count"] == metrics["mutation_commit_groups"], case
            window = report["measurement_window_unix_nanos"]
            events = [event for event in metrics["checkpoint_events"] if event["succeeded"]
                      and int(window["start"]) <= int(event["started_unix_nanos"])
                      and int(event["ended_unix_nanos"]) <= int(window["last_client_completion"])]
            minimum = 0 if plan["smoke"] else 10 if plan["duration_ms"] >= 300000 else 3
            assert len(events) >= minimum, case
            if not plan["smoke"]:
                assert sum(event["writes_during_checkpoint"] for event in events) > 0, case
            envelope = json.loads((evidence / "recovery-envelope.json").read_text())
            assert envelope["extra_bytes"] == plan["extra_recovery_bytes"], case
            assert cell["maximum_recovery_file_bytes"] <= envelope["envelope_bytes"], case
            transactions = metrics["successful_mutation_transactions"]
            wal_bytes = metrics["wal_bytes_at_measurement_end"] - metrics["wal_bytes_at_measurement_start"] + sum(
                event["reclaimed_bytes"] for event in metrics["checkpoint_events"])
            checkpoint_bytes = sum(event["checkpoint_bytes_written"] for event in metrics["checkpoint_events"])
            assert cell["process_write_request_bytes"] >= wal_bytes + checkpoint_bytes, case
            cell.update({"wal_bytes_per_transaction": wal_bytes / transactions,
                         "checkpoint_bytes_per_transaction": checkpoint_bytes / transactions,
                         "process_write_request_bytes_per_transaction": cell["process_write_request_bytes"] / transactions,
                         "transactions_per_group": transactions / metrics["mutation_commit_groups"],
                         "measured_checkpoints": len(events), "capacity_wait_count": metrics["capacity_wait_count"],
                         "coordinator_pause_seconds": metrics["coordinator_pause_nanos"] / 1000000000,
                         "full_base_snapshots": sum(event["full_base_snapshots"] for event in metrics["checkpoint_events"]),
                         **{f"checkpoint_{phase}_seconds": sum(event[f"{phase}_nanos"] for event in metrics["checkpoint_events"]) / 1000000000
                            for phase in ["preparation", "page_write", "data_sync", "metadata_sync", "completion"]}})
        if workload == "RW80":
            for category in ["get", "query16", "write_transaction_success"]:
                cell[f"{category}_p99_ms"] = BASE.latency_ms(counters["latency"][category], "p99")
                cell[f"{category}_p999_ms"] = BASE.histogram_percentile_ms(counters["latency"][category], 999, 1000)
        cells.append(cell)
    medians = {}
    for workload in plan["workloads"]:
        medians[workload] = {}
        for policy in plan["identities"]:
            selected = [cell for cell in cells if cell["workload"] == workload and cell["policy"] == policy]
            numeric = set(selected[0]) - {"case", "policy", "workload", "repetition"}
            medians[workload][policy] = {key: statistics.median(cell[key] for cell in selected) for key in numeric}
            medians[workload][policy]["throughput_range"] = [min(cell["throughput_per_second"] for cell in selected),
                                                               max(cell["throughput_per_second"] for cell in selected)]
            medians[workload][policy]["worst_maximum_ms"] = max(cell["maximum_ms"] for cell in selected)
    gates = {}
    if "previous" in plan["identities"]:
        candidate, previous = medians["E"]["candidate"], medians["E"]["previous"]
        gates["E_throughput_at_least_five_percent"] = candidate["throughput_per_second"] >= previous["throughput_per_second"] * 1.05
        gates["E_p999_at_least_twenty_five_percent_lower"] = candidate["p999_ms"] <= previous["p999_ms"] * 0.75
        for workload in plan["workloads"]:
            candidate, previous = medians[workload]["candidate"], medians[workload]["previous"]
            gates[f"{workload}_maximum_within_ten_percent"] = candidate["maximum_ms"] <= previous["maximum_ms"] * 1.10
            gates[f"{workload}_reopen_at_least_twenty_five_percent_lower"] = candidate["reopen_and_validation_ms"] <= previous["reopen_and_validation_ms"] * 0.75
    ratios = {workload: {policy: {metric: medians[workload]["candidate"][metric] / medians[workload][policy][metric]
                                for metric in ["throughput_per_second", "p99_ms", "p999_ms", "maximum_ms", "reopen_and_validation_ms", "peak_rss_bytes"]}
                         for policy in plan["identities"] if policy != "candidate"} for workload in plan["workloads"]}
    result = {"validated": True, "smoke": plan["smoke"], "cells": cells, "medians": medians,
              "candidate_ratios": ratios, "followup_gates": gates, "followup_allowed": bool(gates) and all(gates.values()) and not plan["smoke"]}
    (root / "matrix-summary.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({key: value for key, value in result.items() if key != "cells"}, indent=2))
    return result


if __name__ == "__main__":
    summarize(Path(sys.argv[1]))
