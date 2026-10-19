import csv
import hashlib
import json
import shutil
import time
from collections import defaultdict
from pathlib import Path

import argparse

parser = argparse.ArgumentParser()
parser.add_argument("--root", type=Path, required=True)
arguments = parser.parse_args()
screen_root = arguments.root.resolve()
runs_root = screen_root / "runs"
manifest_path = screen_root / "database-files.tsv"
logs_manifest_path = screen_root / "preserved-logs-manifest.tsv"
expected_source = "5d772a7448b1b6c29a0f00f501bd51fd83be03b9"
expected_binary = "8ee11c9afeff6c3687643ad14d6f1c35597d984f988d4f8290935547566d8d48"
run_directories = sorted(path for path in runs_root.iterdir() if path.is_dir())
if len(run_directories) != 60:
    raise SystemExit(f"expected 60 run directories, found {len(run_directories)}")

def sha256_path(path):
    digest = hashlib.sha256()
    with path.open("rb") as file_handle:
        for chunk in iter(lambda: file_handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()

database_files_by_run = defaultdict(dict)
with manifest_path.open(newline="") as manifest_file:
    for record in csv.DictReader(manifest_file, delimiter="\t"):
        database_files_by_run[record["run_id"]][record["relative_path"]] = record
logs_by_run = defaultdict(list)
with logs_manifest_path.open(newline="") as manifest_file:
    for record in csv.DictReader(manifest_file, delimiter="\t"):
        logs_by_run[record["run_id"]].append(record)

validated_databases = []
for run_directory in run_directories:
    run_id = run_directory.name
    report_path = run_directory / "report.json"
    command_path = run_directory / "command.json"
    validated_path = run_directory / "validated-summary.json"
    if not all(path.is_file() for path in (report_path, command_path, validated_path)):
        raise SystemExit(f"required validation artifact missing: {run_id}")
    command = json.loads(command_path.read_text())
    report = json.loads(report_path.read_text())
    validated = json.loads(validated_path.read_text())
    if command["run_id"] != run_id or report.get("source_sha") != expected_source or report.get("binary_sha256") != expected_binary:
        raise SystemExit(f"source or run identity mismatch: {run_id}")
    if command.get("environment", {}).get("DODB_SOURCE_SHA") != expected_source or command.get("environment", {}).get("DODB_BINARY_SHA256") != expected_binary:
        raise SystemExit(f"command provenance mismatch: {run_id}")
    if (run_directory / "exit-status.txt").read_text().strip() != "0":
        raise SystemExit(f"non-zero original exit status: {run_id}")
    timing = report.get("timing", {})
    if timing.get("measurement_complete") is not True or timing.get("accepted_as_full_window") is not True or timing.get("termination_reason") != "requested-duration" or timing.get("actual_admission_drain_and_last_completion_ms", 0) < 5000:
        raise SystemExit(f"full measurement window missing: {run_id}")
    if report["latency"].get("count") != report["timing"].get("actual_write_batches") or report["latency"].get("sample_count") != report["timing"].get("actual_write_batches"):
        raise SystemExit(f"latency and acknowledgment counts differ: {run_id}")
    for validation_name in ("final_validation", "reopen_validation"):
        state = report.get(validation_name, {})
        if state.get("passed") is not True or state.get("missing_rows") != 0 or state.get("extra_rows") != 0 or state.get("value_or_revision_mismatches") != 0:
            raise SystemExit(f"state validation failed: {run_id} {validation_name}")
    argv = command["argv"]
    options = dict(zip(argv[1::2], argv[2::2]))
    database_directory = Path(options["--data-dir"]).resolve()
    if database_directory.parent != run_directory.resolve() or database_directory.name != "db":
        raise SystemExit(f"database path outside expected run directory: {run_id}")
    recorded_files = database_files_by_run.get(run_id, {})
    if not recorded_files:
        raise SystemExit(f"database file manifest has no entries for: {run_id}")
    if database_directory.exists():
        actual_files = {str(path.relative_to(database_directory)): path for path in database_directory.rglob("*") if path.is_file()}
        if set(actual_files) != set(recorded_files):
            raise SystemExit(f"database file set differs from manifest: {run_id}")
        for relative_path, database_file in actual_files.items():
            expected = recorded_files[relative_path]
            if database_file.stat().st_size != int(expected["size_bytes"]) or sha256_path(database_file) != expected["sha256"]:
                raise SystemExit(f"database file differs from manifest: {run_id} {relative_path}")
        was_already_removed = False
        database_size = sum(path.stat().st_size for path in actual_files.values())
    else:
        was_already_removed = True
        database_size = sum(int(record["size_bytes"]) for record in recorded_files.values())
    recorded_logs = [record for record in recorded_files.values() if record["is_log"] == "True"]
    preserved_logs = logs_by_run.get(run_id, [])
    if len(recorded_logs) != len(preserved_logs):
        raise SystemExit(f"preserved LOG count differs from database manifest: {run_id}")
    for preserved_log in preserved_logs:
        preserved_path = screen_root / preserved_log["preserved_path"]
        if not preserved_path.is_file() or preserved_path.stat().st_size != int(preserved_log["size_bytes"]) or sha256_path(preserved_path) != preserved_log["sha256"]:
            raise SystemExit(f"preserved LOG content differs from manifest: {run_id}")
    validated_databases.append({
        "run_id": run_id,
        "database_path": str(database_directory),
        "removed_bytes": database_size,
        "already_removed_before_cleanup_manifest": was_already_removed,
    })

for database in validated_databases:
    database_directory = Path(database["database_path"])
    if database_directory.exists():
        shutil.rmtree(database_directory)

cleanup_manifest = {
    "scope": "60 completed and validated screening databases only",
    "validation_source_sha": expected_source,
    "validation_binary_sha256": expected_binary,
    "all_sixty_reports_and_reopened_state_checks_passed": True,
    "database_file_manifest_verified_for_remaining_databases": True,
    "preserved_log_content_verified": True,
    "deleted_database_count": len(validated_databases),
    "deleted_bytes_from_database_file_manifest": sum(database["removed_bytes"] for database in validated_databases),
    "already_removed_before_final_cleanup_manifest": [database["run_id"] for database in validated_databases if database["already_removed_before_cleanup_manifest"]],
    "preserved_log_count": sum(len(records) for records in logs_by_run.values()),
    "preserved_log_manifest": "preserved-logs-manifest.tsv",
    "database_file_manifest": "database-files.tsv",
    "deleted_databases": validated_databases,
    "utc_epoch_seconds": time.time(),
}
(screen_root / "database-cleanup-manifest.json").write_text(json.dumps(cleanup_manifest, indent=2) + "\n")
aggregate_path = screen_root / "screen-aggregate.json"
aggregate_rows = json.loads(aggregate_path.read_text())
for aggregate_row in aggregate_rows:
    if "median_raw_relative_throughput" in aggregate_row:
        aggregate_row["raw_to_backend_median_throughput_ratio"] = aggregate_row.pop("median_raw_relative_throughput")
aggregate_path.write_text(json.dumps(aggregate_rows, indent=2) + "\n")
with (screen_root / "screen-aggregate.csv").open("w", newline="") as aggregate_file:
    aggregate_writer = csv.DictWriter(aggregate_file, fieldnames=list(aggregate_rows[0]))
    aggregate_writer.writeheader()
    aggregate_writer.writerows(aggregate_rows)
print(json.dumps({
    "validated_cases": len(validated_databases),
    "deleted_databases": len(validated_databases),
    "already_removed_before_manifest": len(cleanup_manifest["already_removed_before_final_cleanup_manifest"]),
    "manifest_bytes_removed": cleanup_manifest["deleted_bytes_from_database_file_manifest"],
    "preserved_logs": cleanup_manifest["preserved_log_count"],
}, separators=(",", ":")))
