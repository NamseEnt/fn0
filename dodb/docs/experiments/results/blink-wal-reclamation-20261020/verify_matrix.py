import hashlib
import json
import statistics
from pathlib import Path


ROOT = Path(__file__).resolve().parent
OCI = ROOT / "oci"
REPORTS = OCI / "reports"
POLICIES = {
    "minimal-native": {"backend": "minimal-native", "threshold": None},
    "blink-off": {"backend": "blink", "threshold": None},
    "blink-32m": {"backend": "blink", "threshold": 33_554_432},
    "blink-128m": {"backend": "blink", "threshold": 134_217_728},
}
PATH_ORDERS = {
    1: ["minimal-native", "blink-off", "blink-32m", "blink-128m"],
    2: ["blink-off", "blink-32m", "blink-128m", "minimal-native"],
    3: ["blink-32m", "blink-128m", "minimal-native", "blink-off"],
}


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as source_file:
        while chunk := source_file.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def validation_passed(value):
    if isinstance(value, dict):
        if "passed" in value:
            return value["passed"] is True
        return bool(value) and all(validation_passed(nested) for nested in value.values())
    if isinstance(value, list):
        return bool(value) and all(validation_passed(nested) for nested in value)
    return value is True


def validate_latency_counts(value, location="latency"):
    if isinstance(value, dict):
        if "count" in value and "sample_count" in value:
            if value["count"] != value["sample_count"]:
                raise ValueError(f"latency count mismatch at {location}: {value['count']} != {value['sample_count']}")
        for key, nested in value.items():
            validate_latency_counts(nested, f"{location}.{key}")
    elif isinstance(value, list):
        for index, nested in enumerate(value):
            validate_latency_counts(nested, f"{location}[{index}]")


def input_prefix_signature(report, workload):
    if workload == "A":
        prefixes = []
        for tenant_value in report["per_tenant"].values():
            for prefix in tenant_value["counters"]["client_input_prefixes"]:
                prefixes.append(
                    (
                        prefix["client_index"],
                        prefix["profile"],
                        prefix["workflow_count"],
                        prefix["fingerprint"],
                    )
                )
    else:
        prefixes = [
            (
                client["client_index"],
                client["measurement_input_prefix_batches"],
                client["measurement_input_prefix_fingerprint_fnv1a64"],
            )
            for client in report["per_client"]
        ]
    return sorted(prefixes)


def latency_summary(report, workload):
    if workload == "A":
        tenant = next(iter(report["per_tenant"].values()))
        latency = tenant["counters"]["latency"]
        return {
            name: {
                "count": sample["count"],
                "sample_count": sample["sample_count"],
                "p50_ms": sample["p50_ms"],
                "p95_ms": sample["p95_ms"],
                "p99_ms": sample["p99_ms"],
            }
            for name, sample in latency.items()
        }
    latency = report["latency"]
    return {
        "write_batch": {
            "count": latency["count"],
            "sample_count": latency["sample_count"],
            "p50_ms": latency["p50_nanos"] / 1_000_000,
            "p95_ms": latency["p95_nanos"] / 1_000_000,
            "p99_ms": latency["p99_nanos"] / 1_000_000,
        }
    }


def measurement_checkpoint_count(report, workload, events):
    if workload == "A":
        samples = report["process_metrics"]["resource_samples"]
        start_unix_nanos = int(samples[0]["elapsed_unix_nanos"])
        duration_nanos = int(report["timing"]["admission_ms"] * 1_000_000)
        end_unix_nanos = start_unix_nanos + duration_nanos
        completed_events = [
            event
            for event in events
            if event["succeeded"]
            and start_unix_nanos <= int(event["started_unix_nanos"])
            and int(event["ended_unix_nanos"]) <= end_unix_nanos
        ]
        return len(completed_events), "checkpoint event timestamps within the sampled measurement window"
    total_completed = sum(1 for event in events if event["succeeded"])
    conservative_count = max(0, total_completed - 1)
    return conservative_count, "completed events since the measurement reset minus one possible final coordinator group during drain"


def summarize_case(case_id, workload, backend_key, repetition):
    report_path = REPORTS / f"{case_id}.json"
    case_root = REPORTS / case_id
    controller_path = case_root / "controller-result.json"
    command_path = case_root / "command.json"
    manifest_path = case_root / "database-manifest.json"
    raw_report_path = case_root / "workload-report.raw.json"
    report = json.loads(report_path.read_text())
    controller = json.loads(controller_path.read_text())
    command = json.loads(command_path.read_text())
    policy = POLICIES[backend_key]
    errors = []
    if controller["exit_code"] != 0 or controller["stop_reason"] is not None or not controller["report_exists"]:
        errors.append("controller did not complete cleanly")
    if report.get("run_status") != "completed":
        errors.append("workload status was not completed")
    if not report.get("timing", {}).get("measurement_complete") or not report.get("timing", {}).get("accepted_as_full_window"):
        errors.append("full measurement window was not accepted")
    final_validation = report.get("final_validation", report.get("validation"))
    if not validation_passed(final_validation):
        errors.append("final validation failed")
    if not validation_passed(report.get("reopen_validation")):
        errors.append("reopen validation failed")
    if report.get("source_sha") != "876d5a1b4b7c0b2792efc27237fa3367cd8d1d9d":
        errors.append("report source SHA mismatch")
    expected_binary = json.loads((OCI / "binary-hashes.json").read_text())[
        "service_workload" if workload == "A" else "unconditional_write_workload"
    ]
    if report.get("binary_sha256") != expected_binary or command.get("binary_sha256") != expected_binary:
        errors.append("binary SHA mismatch")
    validate_latency_counts(report.get("latency", report.get("per_tenant", {})))
    metric = report.get("blink_checkpoint_metrics", {})
    expected_threshold = policy["threshold"]
    configuration = report.get("configuration", {})
    configured_blink_policy = configuration.get("blink", {}).get("wal_checkpoint_policy")
    if configured_blink_policy is None:
        configured_blink_policy = configuration.get("blink_checkpoint_policy")
    applied_policy = metric.get("policy") or configured_blink_policy.get("mode")
    applied_threshold = metric.get("threshold_bytes") or configured_blink_policy.get("threshold_bytes")
    if expected_threshold != applied_threshold:
        errors.append(f"applied WAL threshold mismatch: expected {expected_threshold}, got {applied_threshold}")
    if expected_threshold is None and applied_policy != "disabled":
        errors.append("disabled checkpoint policy mismatch")
    checkpoint_events = metric.get("checkpoint_events", [])
    measurement_checkpoints, checkpoint_window_method = measurement_checkpoint_count(report, workload, checkpoint_events)
    if expected_threshold is not None and measurement_checkpoints < 3:
        threshold_cell_status = "insufficient_repeated_measurement_checkpoints"
    elif expected_threshold is not None:
        threshold_cell_status = "repeated_measurement_checkpoints_observed"
    else:
        threshold_cell_status = "not_applicable"
    manifest = json.loads(manifest_path.read_text())
    if not manifest:
        errors.append("database manifest is empty")
    if sha256_file(raw_report_path) != (case_root / "workload-report-raw-sha256.txt").read_text().strip():
        errors.append("raw report SHA mismatch")
    if sha256_file(report_path) != (case_root / "workload-report-sha256.txt").read_text().strip():
        errors.append("enriched report SHA mismatch")
    if workload == "A":
        tenant = next(iter(report["per_tenant"].values()))
        throughput = report["throughput_per_second"]["api_calls"]
        counters = tenant["counters"]
        outcome = {
            "successful_mutations": counters["successful_mutations"],
            "successful_write_transactions": counters["successful_write_transactions"],
            "intentional_condition_rejections": counters["intentional_condition_rejections"],
            "natural_conflicts": counters["natural_conflicts"],
            "overloads": counters["overloads"],
            "other_errors": counters["other_errors"],
            "read_calls": counters["read_calls"],
        }
        latency = latency_summary(report, workload)
        cpu_seconds = report["process_metrics"].get("cpu_seconds")
        process_peak_rss_bytes = (report["process_metrics"].get("peak_sampled_rss_kb") or 0) * 1024
        drain_ms = report.get("timing", {}).get("drain_ms")
    else:
        throughput = report["timing"]["write_batches_per_second"]
        drain_ms = max(
            0,
            report["timing"]["actual_admission_drain_and_last_completion_ms"]
            - report["timing"]["requested_measurement_ms"],
        )
        outcome = {
            "measurement_write_batches_acknowledged": report["counters"]["measurement_write_batches_acknowledged"],
            "warmup_write_batches_acknowledged": report["counters"]["warmup_write_batches_acknowledged"],
            "overloads": report["counters"]["overloads"],
            "other_errors": report["counters"]["other_errors"],
        }
        latency = latency_summary(report, workload)
        cpu_seconds = report["process_metrics"].get("cpu_seconds")
        process_peak_rss_bytes = (report["process_metrics"].get("peak_sampled_rss_kb") or 0) * 1024
    average_group_size = None
    histogram = metric.get("transaction_group_histogram", [])
    histogram_count = metric.get("transaction_group_count", 0)
    if histogram_count:
        average_group_size = sum(index * count for index, count in enumerate(histogram)) / histogram_count
    result = {
        "case_id": case_id,
        "workload": workload,
        "backend_key": backend_key,
        "repetition": repetition,
        "source_sha": report.get("source_sha"),
        "binary_sha256": report.get("binary_sha256"),
        "blink_checkpoint_policy": applied_policy,
        "threshold_bytes": applied_threshold,
        "run_status": report.get("run_status"),
        "accepted_as_full_window": report.get("timing", {}).get("accepted_as_full_window"),
        "throughput_per_second": throughput,
        "latency": latency,
        "outcomes": outcome,
        "cpu_seconds": cpu_seconds,
        "process_peak_rss_bytes": process_peak_rss_bytes,
        "controller_peak_process_group_rss_bytes": controller["peak_process_group_rss_bytes"],
        "maximum_database_bytes": controller["maximum_database_bytes"],
        "maximum_wal_bytes": controller["maximum_wal_bytes"],
        "database_bytes_at_end": controller["database_bytes_before_removal"],
        "database_manifest_file_count": len(manifest),
        "checkpoint_started": metric.get("checkpoints_started", 0),
        "checkpoint_completed": metric.get("checkpoints_completed", 0),
        "checkpoint_failed": metric.get("checkpoints_failed", 0),
        "measurement_checkpoint_completed": measurement_checkpoints,
        "checkpoint_window_method": checkpoint_window_method,
        "threshold_cell_status": threshold_cell_status,
        "checkpoint_total_ms": metric.get("checkpoint_total_nanos", 0) / 1_000_000,
        "checkpoint_max_ms": metric.get("checkpoint_max_nanos", 0) / 1_000_000,
        "wal_bytes_at_measurement_start": metric.get("wal_bytes_at_measurement_start"),
        "wal_bytes_at_measurement_end": metric.get("wal_bytes_at_measurement_end"),
        "wal_bytes_max": metric.get("wal_bytes_max"),
        "threshold_excess_max_bytes": metric.get("threshold_excess_max_bytes"),
        "successful_mutation_transactions": metric.get("successful_mutation_transactions"),
        "mutation_commit_groups": metric.get("mutation_commit_groups"),
        "average_mutation_transactions_per_group": average_group_size,
        "transaction_group_histogram": histogram,
        "wal_syncs": metric.get("wal_syncs"),
        "commit_wal_syncs": metric.get("commit_wal_syncs"),
        "checkpoint_wal_syncs": metric.get("checkpoint_wal_syncs"),
        "drain_ms": drain_ms,
        "final_checkpoint_drain_ms": report.get("timing", {}).get("final_checkpoint_drain_ms"),
        "errors": errors,
        "input_prefix_signature": input_prefix_signature(report, workload),
    }
    return result


def main():
    expected_cases = []
    for repetition, backends in PATH_ORDERS.items():
        for backend_key in backends:
            for workload in ("A", "E"):
                expected_cases.append((f"{workload}-{backend_key}-r{repetition}", workload, backend_key, repetition))
    cases = [summarize_case(*entry) for entry in expected_cases]
    if len(cases) != 24:
        raise RuntimeError(f"expected 24 matrix cells, got {len(cases)}")
    errors = {case["case_id"]: case["errors"] for case in cases if case["errors"]}
    fingerprint_references = {}
    fingerprint_matches = {}
    for case in cases:
        reference = fingerprint_references.setdefault(case["workload"], case["input_prefix_signature"])
        fingerprint_matches[case["case_id"]] = case["input_prefix_signature"] == reference
    failed_fingerprints = [case_id for case_id, matches in fingerprint_matches.items() if not matches]
    checkpoints_short = [
        case["case_id"]
        for case in cases
        if case["threshold_bytes"] is not None and case["threshold_cell_status"] != "repeated_measurement_checkpoints_observed"
    ]
    aggregates = {}
    for workload in ("A", "E"):
        for backend_key in POLICIES:
            group = [case for case in cases if case["workload"] == workload and case["backend_key"] == backend_key]
            if not group:
                continue
            latency_category = "workflow" if workload == "A" else "write_batch"
            aggregates[f"{workload}-{backend_key}"] = {
                "median_throughput_per_second": statistics.median(case["throughput_per_second"] for case in group),
                "throughput_per_repetition": [case["throughput_per_second"] for case in group],
                "median_latency_ms": {
                    percentile: statistics.median(case["latency"][latency_category][f"{percentile}_ms"] for case in group)
                    for percentile in ("p50", "p95", "p99")
                },
                "latency_count_per_repetition": [case["latency"][latency_category]["count"] for case in group],
                "checkpoint_completed_per_repetition": [case["checkpoint_completed"] for case in group],
                "measurement_checkpoint_completed_per_repetition": [case["measurement_checkpoint_completed"] for case in group],
                "median_checkpoint_total_ms": statistics.median(case["checkpoint_total_ms"] for case in group),
                "median_mutation_group_size": statistics.median(case["average_mutation_transactions_per_group"] or 0 for case in group),
                "median_cpu_seconds": statistics.median(case["cpu_seconds"] or 0 for case in group),
                "median_peak_process_group_rss_mib": statistics.median(case["controller_peak_process_group_rss_bytes"] for case in group) / 1024**2,
                "median_maximum_database_mib": statistics.median(case["maximum_database_bytes"] for case in group) / 1024**2,
                "median_maximum_wal_mib": statistics.median(case["wal_bytes_max"] or case["maximum_wal_bytes"] for case in group) / 1024**2,
                "median_commit_wal_syncs": statistics.median(case["commit_wal_syncs"] or 0 for case in group),
                "median_checkpoint_wal_syncs": statistics.median(case["checkpoint_wal_syncs"] or 0 for case in group),
            }
    comparisons = {}
    for workload in ("A", "E"):
        reference = aggregates[f"{workload}-minimal-native"]
        checkpoint_off = aggregates[f"{workload}-blink-off"]
        comparisons[workload] = {
            "blink_off_vs_minimal_native": {
                "throughput_change_percent": (checkpoint_off["median_throughput_per_second"] / reference["median_throughput_per_second"] - 1) * 100,
                "p99_change_percent": (checkpoint_off["median_latency_ms"]["p99"] / reference["median_latency_ms"]["p99"] - 1) * 100,
            },
            "threshold_vs_blink_off": {},
            "threshold_vs_minimal_native": {},
        }
        for backend_key in ("blink-32m", "blink-128m"):
            threshold = aggregates[f"{workload}-{backend_key}"]
            comparisons[workload]["threshold_vs_blink_off"][backend_key] = {
                "throughput_change_percent": (threshold["median_throughput_per_second"] / checkpoint_off["median_throughput_per_second"] - 1) * 100,
                "p99_change_percent": (threshold["median_latency_ms"]["p99"] / checkpoint_off["median_latency_ms"]["p99"] - 1) * 100,
            }
            comparisons[workload]["threshold_vs_minimal_native"][backend_key] = {
                "throughput_change_percent": (threshold["median_throughput_per_second"] / reference["median_throughput_per_second"] - 1) * 100,
                "p99_change_percent": (threshold["median_latency_ms"]["p99"] / reference["median_latency_ms"]["p99"] - 1) * 100,
            }
    summary = {
        "source_sha": "876d5a1b4b7c0b2792efc27237fa3367cd8d1d9d",
        "binary_hashes": json.loads((OCI / "binary-hashes.json").read_text()),
        "expected_cells": 24,
        "observed_cells": len(cases),
        "all_cells_passed_controller_and_reopen_validation": not errors,
        "cell_errors": errors,
        "input_prefix_fingerprints_match_across_paths": not failed_fingerprints,
        "input_prefix_mismatch_cases": failed_fingerprints,
        "threshold_cells_insufficient": checkpoints_short,
        "threshold_measurement_checkpoint_method": {
            "A": "count checkpoint events whose start and end timestamps fall inside the first timestamped 30-second measurement sample window",
            "E": "count post-reset checkpoint completions conservatively minus one possible final queued coordinator group during drain; the runner uses 64 clients, one in-flight transaction per dedicated client, and a 64-request coordinator group",
        },
        "predeclared_path_orders": {str(key): value for key, value in PATH_ORDERS.items()},
        "aggregates": aggregates,
        "comparisons": comparisons,
        "cases": cases,
    }
    (OCI / "matrix-verification-final-v3.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps({key: summary[key] for key in ("expected_cells", "observed_cells", "all_cells_passed_controller_and_reopen_validation", "cell_errors", "input_prefix_fingerprints_match_across_paths", "input_prefix_mismatch_cases", "threshold_cells_insufficient", "aggregates", "comparisons")}, indent=2))
    if errors or failed_fingerprints:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
