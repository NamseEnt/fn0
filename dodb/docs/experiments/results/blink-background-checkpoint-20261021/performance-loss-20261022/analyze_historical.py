import hashlib
import json
import statistics
from pathlib import Path

from analyze_loss import check_latencies


def run():
    destination = Path(__file__).resolve().parent / "evidence"
    destination.mkdir(exist_ok=True)
    reports = Path(__file__).resolve().parents[2] / "blink-wal-reclamation-20261020/oci/reports"
    cells = []
    for workload in ["A", "E"]:
        for backend in ["blink-off", "blink-128m"]:
            for repetition in [1, 2, 3]:
                case_id = f"{workload}-{backend}-r{repetition}"
                path = reports / case_id / "workload-report.raw.json"
                report = json.loads(path.read_text())
                command = json.loads((reports / case_id / "command.json").read_text())
                enriched = json.loads((reports / f"{case_id}.json").read_text())
                assert enriched["source_sha"] == command["source_sha"] == "876d5a1b4b7c0b2792efc27237fa3367cd8d1d9d"
                assert enriched["binary_sha256"] == command["binary_sha256"]
                assert {key: value for key, value in report.items() if key not in {"source_sha", "binary_sha256"}} == {key: value for key, value in enriched.items() if key not in {"source_sha", "binary_sha256"}}
                assert hashlib.sha256(path.read_bytes()).hexdigest() == (reports / case_id / "workload-report-raw-sha256.txt").read_text().strip()
                check_latencies(report)
                metrics = report["blink_checkpoint_metrics"]
                reclaimed = sum(event["reclaimed_bytes"] for event in metrics["checkpoint_events"])
                reinitialized = sum(event["wal_bytes_after"] for event in metrics["checkpoint_events"])
                appended = metrics["wal_bytes_at_measurement_end"] - metrics["wal_bytes_at_measurement_start"] + reclaimed + reinitialized
                transactions = metrics["successful_mutation_transactions"]
                groups = metrics["mutation_commit_groups"]
                timing = report["timing"]
                seconds = (timing["total_elapsed_ms"] if "total_elapsed_ms" in timing else timing["actual_admission_drain_and_last_completion_ms"] + timing["final_checkpoint_drain_ms"]) / 1000
                checkpoint_seconds = metrics["checkpoint_total_nanos"] / 1_000_000_000
                throughput = report["throughput_per_second"]["api_calls"] if workload == "A" else report["timing"]["write_batches_per_second"]
                cells.append({
                    "case": case_id, "workload": workload, "backend": backend,
                    "repetition": repetition, "raw_report_sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                    "original_relative_path": str(path.relative_to(reports.parents[5])),
                    "source_sha": command["source_sha"], "binary_sha256": command["binary_sha256"],
                    "historical_identity_scope": "controller and build provenance; no compile-time git_commit field",
                    "throughput_per_second": throughput, "total_seconds": seconds,
                    "checkpoint_seconds": checkpoint_seconds, "checkpoint_fraction": checkpoint_seconds / seconds,
                    "wal_appended_bytes": appended, "wal_bytes_per_transaction": appended / transactions,
                    "transactions_per_group": transactions / groups,
                    "throughput_excluding_checkpoint_wall": throughput * seconds / (seconds - checkpoint_seconds),
                })
    medians = {}
    for workload in ["A", "E"]:
        medians[workload] = {}
        for backend in ["blink-off", "blink-128m"]:
            values = [cell for cell in cells if cell["workload"] == workload and cell["backend"] == backend]
            medians[workload][backend] = {key: statistics.median(value[key] for value in values)
                                          for key in ["throughput_per_second", "checkpoint_seconds", "checkpoint_fraction",
                                                      "wal_bytes_per_transaction", "transactions_per_group", "throughput_excluding_checkpoint_wall"]}
    result = {"cells": cells, "medians": medians,
              "wal_accounting": "legacy appended bytes = ending retained bytes - starting retained bytes + reclaimed bytes + reset reinitialization bytes",
              "active_rate_limit": "excluding checkpoint wall is descriptive accounting, not a measured counterfactual or isolated native timing"}
    (destination / "historical-loss.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(medians, indent=2))


if __name__ == "__main__":
    run()
