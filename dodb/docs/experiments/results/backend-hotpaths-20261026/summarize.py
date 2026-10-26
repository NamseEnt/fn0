import hashlib
import importlib.util
import json
import statistics
import sys
from pathlib import Path


helper_path = Path(__file__).resolve().parent.parent / "blink-background-checkpoint-20261021/performance-loss-20261022/analyze_loss.py"
if (Path(__file__).resolve().parent / "accounting-helper.py").is_file():
    helper_path = Path(__file__).resolve().parent / "accounting-helper.py"
if not helper_path.is_file():
    helper_path = Path("/bench/zfs/db/blink-checkpoint-costs-20261023/source/dodb/docs/experiments/results/blink-background-checkpoint-20261021/performance-loss-20261022/analyze_loss.py")
specification = importlib.util.spec_from_file_location("accounting", helper_path)
accounting = importlib.util.module_from_spec(specification)
specification.loader.exec_module(accounting)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def summarize(root):
    plan = json.loads((root / "matrix-plan.json").read_text())
    expected = {f"{workload}-{policy}-r{repetition}" for repetition, policies in plan["orders"].items()
                for policy in policies for workload in plan["workloads"]}
    assert (root / "matrix-complete.txt").is_file()
    assert set((root / "completed-cases.txt").read_text().splitlines()) == expected
    cells, prefixes, commands = [], {}, {}
    for case in sorted(expected):
        workload, remainder = case.split("-", 1)
        policy, repetition = remainder.rsplit("-r", 1)
        identity = plan["identities"][policy]
        blink = identity["backend"] == "blink"
        report_path, evidence = root / "reports" / f"{case}.json", root / "reports" / case
        report = json.loads(report_path.read_text())
        raw_path = evidence / "workload-report.raw.json"
        raw = json.loads(raw_path.read_text())
        command = json.loads((evidence / "command.json").read_text())
        arguments = command["argv"][1:]
        assert len(arguments) % 2 == 0
        configuration = dict(zip(arguments[::2], arguments[1::2]))
        assert int(configuration["--measurement-ms"]) == plan["duration_ms"]
        assert int(configuration["--warmup-ms"]) == plan["warmup_ms"]
        assert configuration["--backend"] == identity["backend"]
        excluded = {"--data-dir", "--report", "--backend", "--blink-checkpoint-wal-bytes",
                    "--blink-wal-layout", "--blink-checkpoint-execution", "--blink-wal-segment-bytes"}
        normalized = {key: value for key, value in configuration.items() if key not in excluded}
        commands.setdefault(workload, normalized)
        assert commands[workload] == normalized, case
        if blink:
            assert configuration["--blink-checkpoint-wal-bytes"] == "134217728"
            assert configuration["--blink-wal-layout"] == "segmented-bases-coalesced"
            assert configuration["--blink-checkpoint-execution"] == "background"
            assert configuration["--blink-wal-segment-bytes"] == "33554432"
        binary = "service_workload" if workload == "RW80" else "unconditional_write_workload"
        assert raw["git_commit"] == report["source_sha"] == command["source_sha"] == identity["source_sha"]
        assert raw["binary_sha256"] == command["binary_sha256"] == identity["binaries"][binary]
        assert digest(raw_path) == (evidence / "workload-report-raw-sha256.txt").read_text().strip()
        assert digest(report_path) == (evidence / "workload-report-sha256.txt").read_text().strip()
        assert {key: value for key, value in raw.items() if key != "source_sha"} == {
            key: value for key, value in report.items() if key != "source_sha"}
        controller = json.loads((evidence / "controller-result.json").read_text())
        samples = json.loads((evidence / "resource-samples.json").read_text())
        assert controller["exit_code"] == 0 and controller["stop_reason"] is None
        for observed, limit, relation in [
            ("peak_process_group_rss_bytes", "rss_limit_bytes", "below"),
            ("maximum_database_bytes", "disk_limit_bytes", "below"),
            ("minimum_mem_available_bytes", "minimum_memory_bytes", "above"),
            ("minimum_filesystem_available_bytes", "minimum_disk_bytes", "above"),
        ]:
            assert controller[observed] < plan[limit] if relation == "below" else controller[observed] >= plan[limit]
        assert report["run_status"] == "completed" and report["timing"]["measurement_complete"]
        assert report["timing"].get("accepted_as_full_window", True)
        accounting.check_latencies(report)
        signature = accounting.input_prefix(report, workload)
        assert len(signature) == (16 if workload == "RW80" else 64)
        prefixes.setdefault(workload, signature)
        assert prefixes[workload] == signature
        counters = report["counters"]
        assert counters["overloads"] == 0 and counters["other_errors"] == 0
        if workload == "RW80":
            assert report["scenario"] == "rw80" and abs(counters["read_fraction"] - 0.8) <= 0.005
            assert all(result["passed"] for result in report["validation"].values())
            assert all(result["passed"] for result in report["reopen_validation"].values())
            throughput, latency = report["throughput_per_second"]["api_calls"], counters["latency"]["workflow"]
        else:
            assert report["final_validation"]["passed"] and report["reopen_validation"]["passed"]
            assert report["configuration"]["clients"] == 64 and report["configuration"]["width"] == 16
            throughput, latency = report["timing"]["write_batches_per_second"], report["latency"]
        preserved = json.loads((root / "database-archives" / f"{case}.json").read_text())
        inventory = json.loads((evidence / "database-manifest.json").read_text())
        assert preserved["verified"] and preserved["case"] == case
        assert preserved["file_count"] == len(inventory)
        assert preserved["uncompressed_bytes"] == sum(entry["size_bytes"] for entry in inventory)
        cell = {"case": case, "policy": policy, "workload": workload, "repetition": int(repetition),
                "throughput_per_second": throughput,
                **{f"{percentile}_ms": accounting.latency_ms(latency, percentile) for percentile in ["p50", "p95", "p99"]},
                "p999_ms": accounting.histogram_percentile_ms(latency, 999, 1000),
                "maximum_ms": latency["maximum_nanos"] / 1e6 if "maximum_nanos" in latency else accounting.histogram_percentile_ms(latency, 1, 1),
                "reopen_and_validation_ms": report["reopen_and_validation_ms"],
                "peak_rss_bytes": controller["peak_process_group_rss_bytes"],
                "maximum_database_bytes": controller["maximum_database_bytes"],
                "maximum_wal_bytes": controller["maximum_wal_bytes"],
                "maximum_recovery_file_bytes": max(sample["recovery_file_bytes"] for sample in samples),
                "cpu_seconds": report["process_metrics"]["cpu_seconds"],
                "resource_samples": len(samples),
                "maximum_resource_sample_gap_seconds": max(later["elapsed_seconds"] - earlier["elapsed_seconds"]
                    for earlier, later in zip(samples, samples[1:]))}
        io = report["process_write_io"]
        assert io["before"]["available"] and io["after"]["available"]
        cell["process_write_request_bytes"] = io["after"]["wchar"] - io["before"]["wchar"]
        if blink:
            metrics = report["blink_checkpoint_metrics"]
            assert not any(metrics[key] for key in ["checkpoints_failed", "failed_message", "tasks_running", "retained_image_bytes", "omitted_checkpoint_events"])
            assert sum(metrics["transaction_group_histogram"]) == metrics["transaction_group_count"] == metrics["mutation_commit_groups"]
            window = report["measurement_window_unix_nanos"]
            events = [event for event in metrics["checkpoint_events"] if event["succeeded"]
                      and int(window["start"]) <= int(event["started_unix_nanos"])
                      and int(event["ended_unix_nanos"]) <= int(window["last_client_completion"])]
            minimum = 0 if plan["smoke"] else 10 if plan["duration_ms"] >= 300000 else 3
            assert len(events) >= minimum
            if not plan["smoke"]:
                assert sum(event["writes_during_checkpoint"] for event in events) > 0
            envelope = json.loads((evidence / "recovery-envelope.json").read_text())
            assert envelope["extra_bytes"] == plan["extra_recovery_bytes"]
            assert cell["maximum_recovery_file_bytes"] <= envelope["envelope_bytes"]
            transactions = metrics["successful_mutation_transactions"]
            wal_bytes = metrics["wal_bytes_at_measurement_end"] - metrics["wal_bytes_at_measurement_start"] + sum(
                event["reclaimed_bytes"] for event in metrics["checkpoint_events"])
            checkpoint_bytes = sum(event["checkpoint_bytes_written"] for event in metrics["checkpoint_events"])
            assert cell["process_write_request_bytes"] >= wal_bytes + checkpoint_bytes
            cell.update({"wal_bytes_per_transaction": wal_bytes / transactions,
                         "checkpoint_bytes_per_transaction": checkpoint_bytes / transactions,
                         "transactions_per_group": transactions / metrics["mutation_commit_groups"],
                         "measured_checkpoints": len(events), "capacity_wait_count": metrics["capacity_wait_count"]})
        if workload == "RW80":
            for category in ["get", "query16", "write_transaction_success"]:
                for percentile in ["p95", "p99"]:
                    cell[f"{category}_{percentile}_ms"] = accounting.latency_ms(counters["latency"][category], percentile)
        cells.append(cell)
    medians = {}
    for workload in plan["workloads"]:
        medians[workload] = {}
        for policy in plan["identities"]:
            selected = [cell for cell in cells if cell["workload"] == workload and cell["policy"] == policy]
            numeric = set(selected[0]) - {"case", "policy", "workload", "repetition"}
            metrics = {key: statistics.median(cell[key] for cell in selected) for key in numeric}
            metrics["throughput_range"] = [min(cell["throughput_per_second"] for cell in selected), max(cell["throughput_per_second"] for cell in selected)]
            metrics["worst_maximum_ms"] = max(cell["maximum_ms"] for cell in selected)
            medians[workload][policy] = metrics
    ratios, gates, paired = {}, {}, {}
    if not plan["smoke"]:
        for backend in ["blink", "rocksdb"]:
            ratios[backend], gates[backend], paired[backend] = {}, {}, {}
            for workload in plan["workloads"]:
                candidate, control = medians[workload][f"{backend}-candidate"], medians[workload][f"{backend}-control"]
                metrics = ["throughput_per_second", "p95_ms", "p99_ms", "p999_ms", "maximum_ms", "reopen_and_validation_ms", "peak_rss_bytes", "maximum_database_bytes"]
                if workload == "RW80":
                    metrics.extend(f"{category}_{percentile}_ms" for category in ["get", "query16", "write_transaction_success"] for percentile in ["p95", "p99"])
                ratios[backend][workload] = {metric: candidate[metric] / control[metric] for metric in metrics}
                gates[backend][f"{workload}_throughput_within_three_percent"] = ratios[backend][workload]["throughput_per_second"] >= 0.97
                for metric in metrics[1:]:
                    gates[backend][f"{workload}_{metric}_within_five_percent"] = ratios[backend][workload][metric] <= 1.05
                paired[backend][workload] = [
                    next(cell["throughput_per_second"] for cell in cells if cell["policy"] == f"{backend}-candidate" and cell["workload"] == workload and cell["repetition"] == repetition)
                    / next(cell["throughput_per_second"] for cell in cells if cell["policy"] == f"{backend}-control" and cell["workload"] == workload and cell["repetition"] == repetition)
                    for repetition in range(1, 4)]
            gates[backend]["target_at_least_five_percent"] = (
                ratios[backend]["RW80"]["throughput_per_second"] >= 1.05 or ratios[backend]["RW80"]["get_p99_ms"] <= 0.95
                if backend == "blink" else ratios[backend]["E"]["throughput_per_second"] >= 1.05)
    cross_backend = {workload: {metric: medians[workload]["blink-candidate"][metric] / medians[workload]["rocksdb-candidate"][metric]
                               for metric in ["throughput_per_second", "p95_ms", "p99_ms", "p999_ms", "maximum_ms", "reopen_and_validation_ms", "peak_rss_bytes", "maximum_database_bytes"]}
                     for workload in plan["workloads"]}
    result = {"validated": True, "smoke": plan["smoke"], "cells": cells, "medians": medians,
              "candidate_control_ratios": ratios, "paired_throughput_ratios": paired,
              "screening_gates": gates, "improved_blink_over_improved_rocksdb": cross_backend}
    (root / "matrix-summary.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps({key: value for key, value in result.items() if key != "cells"}, indent=2, sort_keys=True))
    return result


if __name__ == "__main__":
    summarize(Path(sys.argv[1]))
