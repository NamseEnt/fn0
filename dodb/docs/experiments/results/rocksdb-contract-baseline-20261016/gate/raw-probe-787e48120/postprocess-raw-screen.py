import argparse
import csv
import hashlib
import json
import shutil
import statistics
import time
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--root", type=Path, required=True)
parser.add_argument("--delete-validated-databases", action="store_true")
arguments = parser.parse_args()
screen_root = arguments.root.resolve()
runs_root = screen_root / "runs"
expected_source = "5d772a7448b1b6c29a0f00f501bd51fd83be03b9"
expected_binary = "8ee11c9afeff6c3687643ad14d6f1c35597d984f988d4f8290935547566d8d48"
run_directories = sorted(path for path in runs_root.iterdir() if path.is_dir())
if len(run_directories) != 60:
    raise SystemExit(f"expected 60 run directories, found {len(run_directories)}")

def read_json(path):
    return json.loads(path.read_text())

def sha256_path(path):
    digest = hashlib.sha256()
    with path.open("rb") as file_handle:
        for chunk in iter(lambda: file_handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()

validated_rows = []
fingerprint_groups = {}
database_records = []
for run_directory in run_directories:
    report_path = run_directory / "report.json"
    command_path = run_directory / "command.json"
    status_path = run_directory / "exit-status.txt"
    validated_path = run_directory / "validated-summary.json"
    if not all(path.is_file() for path in (report_path, command_path, status_path, validated_path)):
        raise SystemExit(f"required artifact missing: {run_directory}")
    if status_path.read_text().strip() != "0":
        raise SystemExit(f"non-zero original exit status: {run_directory}")
    report = read_json(report_path)
    command = read_json(command_path)
    validated = read_json(validated_path)
    argv = command["argv"]
    options = dict(zip(argv[1::2], argv[2::2]))
    run_id = command["run_id"]
    backend = command["backend"]
    clients = command["clients"]
    width = command["width"]
    repetition = command["repetition"]
    shared_slots = command["shared_blocking_slots"] if backend == "rocksdb-service" else None
    if report.get("source_sha") != expected_source or report.get("binary_sha256") != expected_binary:
        raise SystemExit(f"provenance mismatch: {run_id}")
    if command.get("environment", {}).get("DODB_SOURCE_SHA") != expected_source or command.get("environment", {}).get("DODB_BINARY_SHA256") != expected_binary:
        raise SystemExit(f"command provenance mismatch: {run_id}")
    if options.get("--seed") != "20261013" or options.get("--value-size") != "512" or options.get("--measurement-ms") != "5000" or options.get("--warmup-ms") != "1000" or options.get("--call-cap") != "100000000":
        raise SystemExit(f"workload parameters mismatch: {run_id}")
    timing = report.get("timing", {})
    latency = report.get("latency", {})
    counters = report.get("counters", {})
    final_validation = report.get("final_validation", {})
    reopen_validation = report.get("reopen_validation", {})
    expected_rows = clients * 256 * width
    actual_batches = timing.get("actual_write_batches")
    if timing.get("measurement_complete") is not True or timing.get("accepted_as_full_window") is not True or timing.get("termination_reason") != "requested-duration" or timing.get("actual_admission_drain_and_last_completion_ms", 0) < 5000:
        raise SystemExit(f"incomplete measurement window: {run_id}")
    if actual_batches is None or actual_batches != latency.get("count") or actual_batches != latency.get("sample_count") or actual_batches != counters.get("measurement_write_batches_acknowledged") or counters.get("acknowledged_keys") != actual_batches * width:
        raise SystemExit(f"acknowledgment and latency count mismatch: {run_id}")
    if counters.get("other_errors") != 0 or counters.get("overloads") != 0 or counters.get("request_limit_reached") is not False:
        raise SystemExit(f"unexpected error or cap: {run_id}")
    for state_validation in (final_validation, reopen_validation):
        if state_validation.get("passed") is not True or state_validation.get("expected_rows", expected_rows) != expected_rows or state_validation.get("actual_rows") != expected_rows:
            raise SystemExit(f"final or reopen row validation failed: {run_id}")
        if state_validation.get("missing_rows") != 0 or state_validation.get("extra_rows") != 0 or state_validation.get("value_or_revision_mismatches") != 0:
            raise SystemExit(f"final or reopen state mismatch: {run_id}")
    if backend == "raw-rocksdb" and report.get("configuration", {}).get("raw_has_logical_revision_occ_aba") is not False:
        raise SystemExit(f"raw claim boundary missing: {run_id}")
    if backend != "raw-rocksdb":
        if final_validation.get("same_transaction_revision_group_mismatches") != 0 or reopen_validation.get("same_transaction_revision_group_mismatches") != 0:
            raise SystemExit(f"transaction revision grouping failed: {run_id}")
    client_rows = report.get("per_client", [])
    if len(client_rows) != clients:
        raise SystemExit(f"per-client records incomplete: {run_id}")
    for client_row in client_rows:
        if client_row.get("measurement_input_prefix_batches") != 100:
            raise SystemExit(f"input prefix incomplete: {run_id}")
        prefix_key = (clients, width, repetition, client_row["client_index"])
        fingerprint = client_row.get("measurement_input_prefix_fingerprint_fnv1a64")
        if not fingerprint or fingerprint_groups.setdefault(prefix_key, fingerprint) != fingerprint:
            raise SystemExit(f"paired input prefix mismatch: {run_id}")
    process_metrics = report.get("process_metrics", {})
    if process_metrics.get("rss_sample_count", 0) < 2 or process_metrics.get("peak_sampled_rss_kb", 0) <= 0 or process_metrics.get("cpu_seconds") is None:
        raise SystemExit(f"CPU/RSS evidence incomplete: {run_id}")
    data_directory = Path(options["--data-dir"]).resolve()
    if data_directory.parent != run_directory.resolve() or not data_directory.is_dir():
        raise SystemExit(f"database path outside expected run directory: {run_id}")
    database_bytes = 0
    for database_file in sorted(path for path in data_directory.rglob("*") if path.is_file()):
        relative_path = database_file.relative_to(data_directory)
        database_bytes += database_file.stat().st_size
        database_records.append({
            "run_id": run_id,
            "database_path": str(data_directory),
            "relative_path": str(relative_path),
            "size_bytes": database_file.stat().st_size,
            "sha256": sha256_path(database_file),
            "is_log": database_file.name.startswith("LOG"),
        })
    validated_rows.append({
        "run_id": run_id,
        "index": command["index"],
        "backend": backend,
        "clients": clients,
        "width": width,
        "repetition": repetition,
        "shared_blocking_slots": shared_slots,
        "source_sha": expected_source,
        "binary_sha256": expected_binary,
        "duration_seconds": process_metrics.get("elapsed_ms", 0) / 1000.0,
        "write_batches": actual_batches,
        "write_batches_per_second": timing["write_batches_per_second"],
        "latency_p50_nanos": latency["p50_nanos"],
        "latency_p95_nanos": latency["p95_nanos"],
        "latency_p99_nanos": latency["p99_nanos"],
        "latency_sample_count": latency["sample_count"],
        "peak_sampled_rss_kb": process_metrics["peak_sampled_rss_kb"],
        "cpu_seconds": process_metrics["cpu_seconds"],
        "disk_usage_bytes": report["disk_usage_bytes"],
        "final_validation": final_validation,
        "reopen_validation": reopen_validation,
    })

if len(fingerprint_groups) != 480:
    raise SystemExit(f"expected 480 paired prefix groups, found {len(fingerprint_groups)}")

summary_fields = [
    "run_id", "index", "backend", "clients", "width", "repetition", "shared_blocking_slots",
    "source_sha", "binary_sha256", "duration_seconds", "write_batches", "write_batches_per_second",
    "latency_p50_nanos", "latency_p95_nanos", "latency_p99_nanos", "latency_sample_count",
    "peak_sampled_rss_kb", "cpu_seconds", "disk_usage_bytes",
]
with (screen_root / "screen-summary.csv").open("w", newline="") as summary_file:
    writer = csv.DictWriter(summary_file, fieldnames=summary_fields)
    writer.writeheader()
    writer.writerows({field: row.get(field) for field in summary_fields} for row in validated_rows)

groups = {}
for row in validated_rows:
    group_key = (row["backend"], row["clients"], row["width"], row["shared_blocking_slots"])
    groups.setdefault(group_key, []).append(row)
performance_rows = []
for group_key, group_rows in sorted(groups.items()):
    if len(group_rows) != 3:
        raise SystemExit(f"expected three repetitions for {group_key}, found {len(group_rows)}")
    rates = [row["write_batches_per_second"] for row in group_rows]
    rss_values = [row["peak_sampled_rss_kb"] for row in group_rows]
    raw_key = ("raw-rocksdb", group_key[1], group_key[2], None)
    raw_rates = [row["write_batches_per_second"] for row in groups[raw_key]]
    performance_rows.append({
        "backend": group_key[0],
        "clients": group_key[1],
        "width": group_key[2],
        "shared_blocking_slots": group_key[3],
        "repetitions": len(group_rows),
        "median_batches_per_second": statistics.median(rates),
        "minimum_batches_per_second": min(rates),
        "maximum_batches_per_second": max(rates),
        "raw_to_backend_median_throughput_ratio": statistics.median(raw_rates) / statistics.median(rates),
        "median_peak_sampled_rss_kb": statistics.median(rss_values),
        "minimum_peak_sampled_rss_kb": min(rss_values),
        "maximum_peak_sampled_rss_kb": max(rss_values),
    })
(screen_root / "screen-aggregate.json").write_text(json.dumps(performance_rows, indent=2) + "\n")
aggregate_fields = list(performance_rows[0])
with (screen_root / "screen-aggregate.csv").open("w", newline="") as aggregate_file:
    writer = csv.DictWriter(aggregate_file, fieldnames=aggregate_fields)
    writer.writeheader()
    writer.writerows(performance_rows)

preserved_log_root = screen_root / "preserved-logs"
preserved_log_root.mkdir(exist_ok=True)
log_copies = []
for record in database_records:
    if not record["is_log"]:
        continue
    source_log = Path(record["database_path"]) / record["relative_path"]
    destination_log = preserved_log_root / record["run_id"] / record["relative_path"]
    destination_log.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source_log, destination_log)
    if destination_log.stat().st_size != record["size_bytes"] or sha256_path(destination_log) != record["sha256"]:
        raise SystemExit(f"preserved log verification failed: {record['run_id']} {record['relative_path']}")
    log_copies.append({**record, "preserved_path": str(destination_log.relative_to(screen_root))})

with (screen_root / "database-files.tsv").open("w") as manifest_file:
    manifest_file.write("run_id\tdatabase_path\trelative_path\tsize_bytes\tsha256\tis_log\n")
    for record in database_records:
        manifest_file.write("\t".join(str(record[key]) for key in ("run_id", "database_path", "relative_path", "size_bytes", "sha256", "is_log")) + "\n")
with (screen_root / "preserved-logs-manifest.tsv").open("w") as manifest_file:
    manifest_file.write("run_id\tdatabase_path\trelative_path\tsize_bytes\tsha256\tpreserved_path\n")
    for record in log_copies:
        manifest_file.write("\t".join(str(record[key]) for key in ("run_id", "database_path", "relative_path", "size_bytes", "sha256", "preserved_path")) + "\n")

original_completion = screen_root / "completion.json"
if original_completion.exists() and not (screen_root / "completion-original.json").exists():
    shutil.copy2(original_completion, screen_root / "completion-original.json")
original_data = read_json(screen_root / "completion-original.json") if (screen_root / "completion-original.json").exists() else {}
completion = {
    "source_sha": expected_source,
    "binary_sha256": expected_binary,
    "run_count": 60,
    "completed": True,
    "validated_case_count": len(validated_rows),
    "repetitions_per_profile": 3,
    "measurement_seconds_per_repetition": 5,
    "warmup_seconds_per_repetition": 1,
    "paired_prefix_fingerprint_groups": len(fingerprint_groups),
    "memory_exceed_axis": "deferred by user",
    "performance_claim": "short screening only; three 5-second repetitions per profile; not authoritative performance evidence",
    "original_final_print_failed": "Python 3.9 json.dumps rejected the flush keyword after all 60 reports and summaries had been written",
    "original_completion_performance_claim": original_data.get("performance_claim"),
    "database_files_manifest": "database-files.tsv",
    "preserved_logs_manifest": "preserved-logs-manifest.tsv",
}
(screen_root / "completion.json").write_text(json.dumps(completion, indent=2) + "\n")
(screen_root / "prefix-fingerprints.json").write_text(json.dumps({"keys": len(fingerprint_groups), "passed": True}, indent=2) + "\n")

if arguments.delete_validated_databases:
    removed_records = []
    for run_directory in run_directories:
        command = read_json(run_directory / "command.json")
        argv = command["argv"]
        options = dict(zip(argv[1::2], argv[2::2]))
        database_directory = Path(options["--data-dir"]).resolve()
        if database_directory.parent != run_directory.resolve() or database_directory.name != "db" or not database_directory.is_dir():
            raise SystemExit(f"refusing unexpected database path: {database_directory}")
        database_size = sum(path.stat().st_size for path in database_directory.rglob("*") if path.is_file())
        expected_logs = [record for record in database_records if record["run_id"] == command["run_id"] and record["is_log"]]
        preserved_run_logs = [record for record in log_copies if record["run_id"] == command["run_id"]]
        if len(expected_logs) != len(preserved_run_logs):
            raise SystemExit(f"LOG preservation count mismatch for run: {command['run_id']}")
        shutil.rmtree(database_directory)
        removed_records.append({"run_id": command["run_id"], "database_path": str(database_directory), "removed_bytes": database_size})
    cleanup = {
        "deleted_after_all_60_reports_and_final_reopen_checks_passed": True,
        "deleted_database_count": len(removed_records),
        "deleted_bytes": sum(record["removed_bytes"] for record in removed_records),
        "preserved_log_files": len(log_copies),
        "removed_databases": removed_records,
        "utc_epoch_seconds": time.time(),
    }
    (screen_root / "database-cleanup-manifest.json").write_text(json.dumps(cleanup, indent=2) + "\n")

print(json.dumps({
    "validated_cases": len(validated_rows),
    "aggregate_groups": len(performance_rows),
    "paired_prefix_groups": len(fingerprint_groups),
    "database_files": len(database_records),
    "preserved_logs": len(log_copies),
    "deleted_databases": arguments.delete_validated_databases,
}, separators=(",", ":")))
