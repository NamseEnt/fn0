import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(os.environ.get("DODB_CHECKPOINT_RUN_ROOT", "/bench/zfs/db/blink-background-checkpoint-20261022"))
SOURCE = ROOT / "source"
TARGET = ROOT / "bin"
SERVICE_BINARY = TARGET / "service_workload"
UNCONDITIONAL_BINARY = TARGET / "unconditional_write_workload"
REPORTS = ROOT / "reports"
DATABASES = ROOT / "databases"
SAMPLE_INTERVAL_SECONDS = 1.0
MAX_PROCESS_GROUP_RSS_BYTES = 3 * 1024**3
MINIMUM_MEM_AVAILABLE_BYTES = 1500 * 1024**2
MAXIMUM_DATABASE_BYTES = 4 * 1024**3
MINIMUM_DISK_AVAILABLE_BYTES = 10 * 1024**3
MEASUREMENT_MS = int(os.environ.get("DODB_CHECKPOINT_MEASUREMENT_MS", "30000"))
WARMUP_MS = int(os.environ.get("DODB_CHECKPOINT_WARMUP_MS", "10000"))
SMOKE = os.environ.get("DODB_CHECKPOINT_SMOKE", "0") == "1"
POLICIES = {
    "minimal-native": ("minimal-native", "off", "legacy", "sync"),
    "legacy-sync-128m": ("blink", "134217728", "legacy", "sync"),
    "segmented-sync-128m": ("blink", "134217728", "segmented", "sync"),
    "segmented-background-32m": ("blink", "33554432", "segmented", "background"),
    "segmented-background-128m": ("blink", "134217728", "segmented", "background"),
}
PATH_NAMES = list(POLICIES)
PATH_ORDERS = {
    repetition: PATH_NAMES[repetition - 1:] + PATH_NAMES[:repetition - 1]
    for repetition in range(1, 2 if SMOKE else 4)
}
CHILD_TIMEOUT_SECONDS = 150 if MEASUREMENT_MS <= 30000 else 480



def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as source_file:
        while chunk := source_file.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def directory_bytes(directory):
    if not directory.exists():
        return 0
    total_bytes = 0
    for file_path in directory.rglob("*"):
        if file_path.is_file():
            try:
                total_bytes += file_path.stat().st_size
            except FileNotFoundError:
                pass
    return total_bytes


def is_wal_file(file_path):
    return file_path.suffix == ".wal" or (
        file_path.suffix == ".log"
        and bool(file_path.stem)
        and file_path.stem.isascii()
        and file_path.stem.isdecimal()
    )


def wal_bytes(directory):
    if not directory.exists():
        return 0
    total_bytes = 0
    for file_path in directory.rglob("*"):
        if file_path.is_file() and is_wal_file(file_path):
            try:
                total_bytes += file_path.stat().st_size
            except FileNotFoundError:
                pass
    return total_bytes


def available_bytes(path):
    return shutil.disk_usage(path).free


def memory_available_bytes():
    for line in Path("/proc/meminfo").read_text().splitlines():
        if line.startswith("MemAvailable:"):
            return int(line.split()[1]) * 1024
    raise RuntimeError("MemAvailable is missing from /proc/meminfo")


def process_group_stats(process_group):
    resident_bytes = 0
    cpu_ticks = 0
    for process_path in Path("/proc").iterdir():
        if not process_path.name.isdigit():
            continue
        try:
            stat_fields = (process_path / "stat").read_text().rsplit(")", 1)[1].split()
            if int(stat_fields[2]) != process_group:
                continue
            cpu_ticks += int(stat_fields[11]) + int(stat_fields[12])
            for status_line in (process_path / "status").read_text().splitlines():
                if status_line.startswith("VmRSS:"):
                    resident_bytes += int(status_line.split()[1]) * 1024
                    break
        except (OSError, ValueError, IndexError):
            continue
    return resident_bytes, cpu_ticks / os.sysconf("SC_CLK_TCK")


def process_sample(process_group, database_path):
    resident_bytes, cpu_seconds = process_group_stats(process_group)
    return {
        "elapsed_monotonic_seconds": time.monotonic(),
        "process_group_rss_bytes": resident_bytes,
        "process_group_cpu_seconds": cpu_seconds,
        "mem_available_bytes": memory_available_bytes(),
        "database_bytes": directory_bytes(database_path),
        "wal_bytes": wal_bytes(database_path),
        "filesystem_available_bytes": available_bytes(ROOT),
    }


def validation_passed(value):
    if isinstance(value, dict):
        if "passed" in value:
            return value["passed"] is True
        return bool(value) and all(validation_passed(result) for result in value.values())
    return value is True


def validate_latency_counts(value, location="latency"):
    if isinstance(value, dict):
        if "count" in value and "sample_count" in value:
            if value["count"] != value["sample_count"]:
                raise RuntimeError(f"latency sample count mismatch at {location}: {value}")
        for key, nested in value.items():
            validate_latency_counts(nested, f"{location}.{key}")
    elif isinstance(value, list):
        for index, nested in enumerate(value):
            validate_latency_counts(nested, f"{location}[{index}]")


def input_prefix_signature(report_value, workload):
    if workload == "A":
        prefixes = []
        for tenant_value in report_value["per_tenant"].values():
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
                client_value["client_index"],
                client_value["measurement_input_prefix_batches"],
                client_value["measurement_input_prefix_fingerprint_fnv1a64"],
            )
            for client_value in report_value["per_client"]
        ]
    return sorted([list(prefix) for prefix in prefixes])


def wait_for_case(command, case_id, database_path, report_path, workload, backend, policy):
    case_root = REPORTS / case_id
    case_root.mkdir(parents=True, exist_ok=False)
    binary_digest = sha256_file(Path(command[0]))
    source_revision = subprocess.check_output(["git", "-C", str(SOURCE), "rev-parse", "HEAD"], text=True).strip()
    process_environment = os.environ.copy()
    process_environment["DODB_SOURCE_SHA"] = source_revision
    process_environment["DODB_BINARY_SHA256"] = binary_digest
    command_record = {
        "argv": [str(argument) for argument in command],
        "cwd": str(SOURCE),
        "case_id": case_id,
        "workload": workload,
        "backend": backend,
        "blink_checkpoint_policy": policy,
        "database_path": str(database_path),
        "report_path": str(report_path),
        "source_sha": source_revision,
        "binary_sha256": binary_digest,
        "limits": {
            "max_process_group_rss_bytes": MAX_PROCESS_GROUP_RSS_BYTES,
            "minimum_mem_available_bytes": MINIMUM_MEM_AVAILABLE_BYTES,
            "maximum_database_bytes": MAXIMUM_DATABASE_BYTES,
            "minimum_filesystem_available_bytes": MINIMUM_DISK_AVAILABLE_BYTES,
            "child_timeout_seconds": CHILD_TIMEOUT_SECONDS,
        },
    }
    (case_root / "command.json").write_text(json.dumps(command_record, indent=2) + "\n")
    database_path.parent.mkdir(parents=True, exist_ok=True)
    if database_path.exists():
        raise RuntimeError(f"case database path already exists: {database_path}")
    start_sample = process_sample(os.getpgrp(), database_path)
    if start_sample["mem_available_bytes"] < MINIMUM_MEM_AVAILABLE_BYTES:
        raise RuntimeError("host MemAvailable is below the configured reserve")
    if start_sample["filesystem_available_bytes"] < MINIMUM_DISK_AVAILABLE_BYTES:
        raise RuntimeError("benchmark filesystem is below the configured free-space reserve")
    start_time = time.monotonic()
    with (case_root / "workload.stdout.log").open("wb") as stdout_file, (case_root / "workload.stderr.log").open("wb") as stderr_file:
        child = subprocess.Popen(
            command,
            cwd=SOURCE,
            stdout=stdout_file,
            stderr=stderr_file,
            env=process_environment,
            start_new_session=True,
        )
        samples = []
        stop_reason = None
        while child.poll() is None:
            sample = process_sample(child.pid, database_path)
            sample["elapsed_seconds"] = time.monotonic() - start_time
            samples.append(sample)
            if sample["process_group_rss_bytes"] >= MAX_PROCESS_GROUP_RSS_BYTES:
                stop_reason = "process-group RSS ceiling"
            elif sample["mem_available_bytes"] < MINIMUM_MEM_AVAILABLE_BYTES:
                stop_reason = "host MemAvailable reserve"
            elif sample["database_bytes"] >= MAXIMUM_DATABASE_BYTES:
                stop_reason = "case database size ceiling"
            elif sample["filesystem_available_bytes"] < MINIMUM_DISK_AVAILABLE_BYTES:
                stop_reason = "filesystem free-space reserve"
            elif sample["elapsed_seconds"] >= CHILD_TIMEOUT_SECONDS:
                stop_reason = "child timeout"
            if stop_reason is not None:
                os.killpg(child.pid, signal.SIGTERM)
                try:
                    child.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait()
                break
            time.sleep(SAMPLE_INTERVAL_SECONDS)
        exit_code = child.wait()
    final_sample = process_sample(child.pid, database_path)
    final_sample["elapsed_seconds"] = time.monotonic() - start_time
    samples.append(final_sample)
    (case_root / "resource-samples.json").write_text(json.dumps(samples, indent=2) + "\n")
    run_result = {
        "case_id": case_id,
        "exit_code": exit_code,
        "stop_reason": stop_reason,
        "elapsed_seconds": time.monotonic() - start_time,
        "sample_count": len(samples),
        "peak_process_group_rss_bytes": max(sample["process_group_rss_bytes"] for sample in samples),
        "minimum_mem_available_bytes": min(sample["mem_available_bytes"] for sample in samples),
        "maximum_database_bytes": max(sample["database_bytes"] for sample in samples),
        "maximum_wal_bytes": max(sample["wal_bytes"] for sample in samples),
        "minimum_filesystem_available_bytes": min(sample["filesystem_available_bytes"] for sample in samples),
        "report_exists": report_path.is_file(),
    }
    (case_root / "controller-result.json").write_text(json.dumps(run_result, indent=2) + "\n")
    if stop_reason is not None or exit_code != 0 or not report_path.is_file():
        raise RuntimeError(f"case failed and database is preserved: {run_result}")
    report_value = json.loads(report_path.read_text())
    raw_report_path = case_root / "workload-report.raw.json"
    shutil.copy2(report_path, raw_report_path)
    report_value["source_sha"] = source_revision
    report_value["binary_sha256"] = binary_digest
    report_path.write_text(json.dumps(report_value, indent=2) + "\n")
    (case_root / "workload-report-raw-sha256.txt").write_text(sha256_file(raw_report_path) + "\n")
    (case_root / "workload-report-sha256.txt").write_text(sha256_file(report_path) + "\n")
    if report_value.get("run_status") != "completed":
        raise RuntimeError(f"workload status is not completed; database is preserved: {case_id}")
    validate_latency_counts(report_value)
    validate_checkpoint_report(report_value, command, source_revision, binary_digest)
    timing_value = report_value.get("timing", {})
    if timing_value.get("measurement_complete") is not True or timing_value.get("accepted_as_full_window") is False:
        raise RuntimeError(f"full measurement window was not accepted; database is preserved: {case_id}")
    final_validation = report_value.get("final_validation", report_value.get("validation"))
    if not validation_passed(final_validation):
        raise RuntimeError(f"final validation failed and database is preserved: {case_id}")
    reopen_validation = report_value.get("reopen_validation")
    if not validation_passed(reopen_validation):
        raise RuntimeError(f"reopen validation failed and database is preserved: {case_id}")
    manifest = []
    log_contents = []
    for file_path in sorted(database_path.rglob("*")):
        if not file_path.is_file():
            continue
        relative_path = file_path.relative_to(database_path).as_posix()
        manifest.append({"path": relative_path, "size_bytes": file_path.stat().st_size, "sha256": sha256_file(file_path)})
        if file_path.name == "LOG" or file_path.name.startswith("LOG."):
            preserved_log = case_root / "rocksdb-logs" / relative_path.replace("/", "__")
            preserved_log.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(file_path, preserved_log)
            log_contents.append({"path": relative_path, "preserved_path": str(preserved_log), "size_bytes": file_path.stat().st_size, "sha256": sha256_file(preserved_log)})
    (case_root / "database-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    (case_root / "rocksdb-log-manifest.json").write_text(json.dumps(log_contents, indent=2) + "\n")
    run_result["database_file_count"] = len(manifest)
    run_result["database_bytes_before_removal"] = sum(item["size_bytes"] for item in manifest)
    run_result["rocksdb_log_file_count"] = len(log_contents)
    (case_root / "controller-result.json").write_text(json.dumps(run_result, indent=2) + "\n")
    if os.environ.get("DODB_CHECKPOINT_KEEP_DATABASES", "0") != "1":
        shutil.rmtree(database_path)


def validate_checkpoint_report(report, command, source_revision, binary_digest):
    if report.get("git_commit") != source_revision:
        raise RuntimeError("raw build Git commit differs from the measurement checkout")
    if report.get("binary_sha256") != binary_digest:
        raise RuntimeError("raw executable SHA256 differs from the executed binary")
    for counters in [report.get("counters", {})] + [tenant.get("counters", {}) for tenant in report.get("per_tenant", {}).values()]:
        if counters.get("overloads", 0) or counters.get("other_errors", 0):
            raise RuntimeError(f"request errors are present: {counters}")
    if "--blink-checkpoint-execution" not in command:
        return
    execution = command[command.index("--blink-checkpoint-execution") + 1]
    layout = command[command.index("--blink-wal-layout") + 1]
    configuration = report["blink_checkpoint_configuration"]
    if configuration["execution"] != execution or configuration["layout"] != layout:
        raise RuntimeError("effective checkpoint execution/layout does not match the command")
    metrics = report["blink_checkpoint_metrics"]
    if metrics["checkpoints_failed"] or metrics["failed_message"]:
        raise RuntimeError("checkpoint failure is present")
    if metrics["tasks_running"] or metrics["retained_image_bytes"]:
        raise RuntimeError("checkpoint drain left retained work")
    if metrics["omitted_checkpoint_events"]:
        raise RuntimeError("checkpoint events were omitted")
    if not SMOKE:
        window = report["measurement_window_unix_nanos"]
        start = int(window["start"])
        end = int(window["last_client_completion"])
        events = [event for event in metrics["checkpoint_events"] if event["succeeded"]
            and int(event["started_unix_nanos"]) >= start and int(event["ended_unix_nanos"]) <= end]
        if len(events) < 3:
            raise RuntimeError(f"insufficient measured checkpoints: {len(events)}")
        if execution == "background" and not any(event["writes_during_checkpoint"] > 0 for event in events):
            raise RuntimeError("no completed checkpoint has acknowledged write overlap")


def service_command(backend, database_path, report_path, policy):
    command = [
        str(SERVICE_BINARY), "--backend", backend, "--client-executor", "dedicated",
        "--scenario", "rw20", "--data-dir", str(database_path), "--report", str(report_path),
        "--seed", "20261013", "--rows", "10000", "--value-size", "512", "--clients", "16",
        "--warmup-ms", str(WARMUP_MS), "--measurement-ms", str(MEASUREMENT_MS), "--call-cap", "100000000",
        "--max-write-ledger", "5000000", "--require-full-window", "1",
        "--mutation-workload", "overwrite", "--overwrite-slots-per-client", "1024",
        "--latency-storage", "log1024", "--include-width-16", "1",
        "--revision-allocator", "reserved-ranges", "--condition-key-lock-mode", "exclusive",
    ]
    if backend == "blink":
        command.extend(["--blink-checkpoint-wal-bytes", policy])
    return command


def unconditional_command(backend, database_path, report_path, policy):
    command = [
        str(UNCONDITIONAL_BINARY), "--backend", backend, "--width", "16", "--clients", "64",
        "--seed", "20261013", "--value-size", "512", "--warmup-ms", str(WARMUP_MS),
        "--measurement-ms", str(MEASUREMENT_MS), "--call-cap", "100000000",
        "--revision-allocator", "reserved-ranges", "--condition-key-lock-mode", "exclusive",
        "--engine-health-interval-ms", "0", "--data-dir", str(database_path),
        "--report", str(report_path),
    ]
    if backend == "blink":
        command.extend(["--blink-checkpoint-wal-bytes", policy])
    return command


def run_matrix():
    REPORTS.mkdir(parents=True, exist_ok=True)
    DATABASES.mkdir(parents=True, exist_ok=True)
    binary_hashes = {
        "service_workload": sha256_file(SERVICE_BINARY),
        "unconditional_write_workload": sha256_file(UNCONDITIONAL_BINARY),
    }
    binary_hash_path = ROOT / "binary-hashes.json"
    if binary_hash_path.exists():
        if json.loads(binary_hash_path.read_text()) != binary_hashes:
            raise RuntimeError("existing binary hashes differ from the built workloads")
    else:
        binary_hash_path.write_text(json.dumps(binary_hashes, indent=2) + "\n")
    matrix = {
        "source_sha": subprocess.check_output(["git", "-C", str(SOURCE), "rev-parse", "HEAD"], text=True).strip(),
        "binary_hashes": binary_hashes,
        "path_orders": {str(repetition): backends for repetition, backends in PATH_ORDERS.items()},
        "workloads": ["A-rw20", "E-width16-unconditional"],
        "repetitions": len(PATH_ORDERS),
        "cells": 2 * len(PATH_NAMES) * len(PATH_ORDERS),
        "measurement_ms": MEASUREMENT_MS,
        "warmup_ms": WARMUP_MS,
        "smoke": SMOKE,
        "limits": {
            "max_process_group_rss_bytes": MAX_PROCESS_GROUP_RSS_BYTES,
            "minimum_mem_available_bytes": MINIMUM_MEM_AVAILABLE_BYTES,
            "maximum_database_bytes": MAXIMUM_DATABASE_BYTES,
            "minimum_filesystem_available_bytes": MINIMUM_DISK_AVAILABLE_BYTES,
        },
    }
    matrix_path = ROOT / "matrix-plan.json"
    if matrix_path.exists():
        if json.loads(matrix_path.read_text()) != matrix:
            raise RuntimeError("existing matrix plan differs from this runner")
    else:
        matrix_path.write_text(json.dumps(matrix, indent=2) + "\n")
    completed_path = ROOT / "completed-cases.txt"
    completed_order = completed_path.read_text().splitlines() if completed_path.exists() else []
    completed_cases = set(completed_order)
    prefix_references = {"A": None, "E": None}
    prefix_records_path = ROOT / "input-prefix-fingerprints.json"
    prefix_records = json.loads(prefix_records_path.read_text()) if prefix_records_path.exists() else []
    for record in prefix_records:
        prefix_references[record["workload"]] = record["signature"]
    for completed_case in completed_cases:
        if prefix_references[completed_case.split("-", 1)[0]] is not None:
            continue
        completed_report = REPORTS / f"{completed_case}.json"
        if not completed_report.is_file():
            raise RuntimeError(f"completed case is missing its report: {completed_case}")
        completed_workload = completed_case.split("-", 1)[0]
        completed_signature = input_prefix_signature(json.loads(completed_report.read_text()), completed_workload)
        prefix_references[completed_workload] = completed_signature
        prefix_records.append({"case_id": completed_case, "workload": completed_workload, "signature": completed_signature, "matches_first_case": True})
        prefix_records_path.write_text(json.dumps(prefix_records, indent=2) + "\n")
    for repetition, backends in PATH_ORDERS.items():
        for backend_key in backends:
            backend, policy, layout, execution = POLICIES[backend_key]
            for workload in ("A", "E"):
                case_id = f"{workload}-{backend_key}-r{repetition}"
                if case_id in completed_cases:
                    continue
                database_path = DATABASES / case_id
                report_path = REPORTS / f"{case_id}.json"
                command = service_command(backend, database_path, report_path, policy) if workload == "A" else unconditional_command(backend, database_path, report_path, policy)
                if backend == "blink":
                    command.extend(["--blink-wal-layout", layout, "--blink-checkpoint-execution", execution])
                    if layout == "segmented":
                        command.extend(["--blink-wal-segment-bytes", "33554432"])
                wait_for_case(command, case_id, database_path, report_path, workload, backend, policy)
                report_value = json.loads(report_path.read_text())
                signature = input_prefix_signature(report_value, workload)
                if prefix_references[workload] is None:
                    prefix_references[workload] = signature
                prefix_records.append({"case_id": case_id, "workload": workload, "signature": signature, "matches_first_case": signature == prefix_references[workload]})
                prefix_records_path.write_text(json.dumps(prefix_records, indent=2) + "\n")
                completed_cases.add(case_id)
                completed_order.append(case_id)
                completed_path.write_text("\n".join(completed_order) + "\n")
    mismatched_prefixes = [record["case_id"] for record in prefix_records if not record["matches_first_case"]]
    if mismatched_prefixes:
        raise RuntimeError(f"input-prefix fingerprints differ from the first case: {mismatched_prefixes}")
    (ROOT / "matrix-complete.txt").write_text(f"{len(completed_cases)} cases completed and validated; input prefix fingerprints matched within each workload\n")


if __name__ == "__main__":
    try:
        run_matrix()
    except Exception as error:
        print(str(error), file=sys.stderr)
        raise
