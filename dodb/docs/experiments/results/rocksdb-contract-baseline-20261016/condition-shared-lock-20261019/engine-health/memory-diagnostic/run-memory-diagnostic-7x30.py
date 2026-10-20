import argparse
import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path


SOURCE_SHA = "843f0ea1598a4fa87284d1e3ee421612f288b4c2"
SERVICE_BINARY_SHA256 = "8814fd1b3ff71b80df8fc4f02f11df68c9b280321f58114cbedcfb99e96e4022"
RAW_BINARY_SHA256 = "e0957ff46415ea3549ac50e9fc16ac1a2a4812ceede3cd0367715dc53ac7414d"
WORKER_BINARY_SHA256 = "01e26347790d75992b47634df6c83eaa5cc5482c442dbbfb7795d17eca30717d"
SERVICE_BINARY = Path("/bench/zfs/db/condition-shared-lock-20261019/engine-health/memory-diagnostic-843f0ea15/frozen-binaries/service_workload")
RAW_BINARY = Path("/bench/zfs/db/condition-shared-lock-20261019/engine-health/memory-diagnostic-843f0ea15/frozen-binaries/unconditional_write_workload")
WORKER_BINARY = Path("/bench/zfs/db/condition-shared-lock-20261019/engine-health/memory-diagnostic-843f0ea15/frozen-binaries/memory_diagnostic_worker")
SOURCE_PATH = Path("/bench/zfs/db/condition-shared-lock-20261019/source-b4b0652")
RESULT_ROOT = Path("/bench/zfs/db/condition-shared-lock-20261019/engine-health/memory-diagnostic-843f0ea15-7x30")
DATABASE_ROOT = Path("/bench/zfs/db/condition-shared-lock-20261019/engine-health/memory-diagnostic-843f0ea15-7x30/db")
DISK_ROOT = Path("/bench/zfs/db")
SEED = 20261013
ROWS = 10_000
VALUE_SIZE = 512
WARMUP_MS = 10_000
MEASUREMENT_MS = 30_000
HEALTH_INTERVAL_MS = 1_000
CALL_CAP = 100_000_000
MAX_WRITE_LEDGER = 5_000_000
RSS_CAP_BYTES = 3 << 30
DATABASE_CAP_BYTES = 12 << 30
MINIMUM_FREE_BYTES = 26 << 30
MINIMUM_AVAILABLE_BYTES = (3 << 30) + (1_500 << 20)
RUNTIME_AVAILABLE_RESERVE_BYTES = 1_500 << 20
CASE_TIMEOUT_SECONDS = 900
STARTUP_TIMEOUT_MS = 300_000
HISTOGRAM_SCHEMA_VERSION = 1
HISTOGRAM_SCHEMA = "exact integer nanoseconds below 1024; 1024 equal-width sub-buckets per binary exponent thereafter; percentile is the bucket upper bound"


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as file_handle:
        for chunk in iter(lambda: file_handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def histogram_upper_bound(bucket_index):
    if bucket_index < 1024:
        return bucket_index
    relative_index = bucket_index - 1024
    exponent = 10 + relative_index // 1024
    sub_bucket = relative_index % 1024
    exponent_base = 1 << exponent
    bucket_width = max(exponent_base // 1024, 1)
    return exponent_base + (sub_bucket + 1) * bucket_width - 1


def validate_histogram(histogram):
    if histogram.get("histogram_schema_version") != HISTOGRAM_SCHEMA_VERSION or histogram.get("histogram_binning_schema") != HISTOGRAM_SCHEMA:
        raise ValueError("latency histogram schema mismatch")
    bins = histogram.get("nonzero_bins", [])
    if bins != sorted(bins, key=lambda entry: entry[0]) or len({entry[0] for entry in bins}) != len(bins):
        raise ValueError("latency histogram bins are not unique and sorted")
    count = histogram.get("count", histogram.get("sample_count", 0))
    if sum(entry[1] for entry in bins) != count:
        raise ValueError("latency histogram bin count mismatch")
    percentile_fields = ["p50_ms", "p95_ms", "p99_ms"] if "p50_ms" in histogram else ["p50_nanos", "p95_nanos", "p99_nanos"]
    for percentile, field_name in zip((50, 95, 99), percentile_fields):
        if count == 0:
            if histogram.get(field_name) is not None:
                raise ValueError(f"{field_name} must be null for an empty histogram")
            continue
        if field_name.endswith("_ms"):
            target_rank = ((count - 1) * percentile + 99) // 100 + 1
        else:
            target_rank = max((count * percentile + 99) // 100, 1)
        cumulative = 0
        upper_bound = 0
        for bucket_index, bucket_count in bins:
            cumulative += bucket_count
            if cumulative >= target_rank:
                upper_bound = histogram_upper_bound(bucket_index)
                break
        expected = upper_bound / 1_000_000 if field_name.endswith("_ms") else upper_bound
        if histogram.get(field_name) != expected:
            raise ValueError(f"{field_name} differs from sparse-bin reconstruction")


def validate_histograms(value):
    if isinstance(value, dict):
        if value.get("storage", "").startswith("log1024"):
            validate_histogram(value)
        for nested_value in value.values():
            validate_histograms(nested_value)
    elif isinstance(value, list):
        for nested_value in value:
            validate_histograms(nested_value)


def selected_jobs():
    jobs = [
        {"case_id": "A-rw20-service-blink", "family": "A", "binary": "service", "backend": "blink"},
        {"case_id": "A-rw20-service-rocksdb16", "family": "A", "binary": "service", "backend": "rocksdb"},
        {"case_id": "A-rw20-service-core16", "family": "A", "binary": "service", "backend": "rocksdb-direct"},
    ]
    jobs.extend(
        {"case_id": f"E-width16-{backend}", "family": "E", "binary": "raw", "backend": backend}
        for backend in ["raw-rocksdb", "synchronous-core", "rocksdb-service", "blink"]
    )
    for job in jobs:
        if job["family"] == "A":
            command = [
                str(SERVICE_BINARY),
                "--backend", job["backend"],
                "--client-executor", "dedicated",
                "--scenario", "rw20",
                "--clients", "16",
                "--rows", str(ROWS),
                "--value-size", str(VALUE_SIZE),
                "--seed", str(SEED),
                "--warmup-ms", str(WARMUP_MS),
                "--measurement-ms", str(MEASUREMENT_MS),
                "--call-cap", str(CALL_CAP),
                "--mutation-workload", "overwrite",
                "--overwrite-slots-per-client", "1024",
                "--latency-storage", "log1024",
                "--max-write-ledger", str(MAX_WRITE_LEDGER),
                "--require-full-window", "1",
                "--include-width-16", "1",
                "--revision-allocator", "reserved-ranges",
                "--condition-key-lock-mode", "shared",
            ]
            if job["backend"] != "blink":
                command.extend(["--shared-blocking-slots", "16"])
            command.extend([
                "--engine-health-interval-ms", str(HEALTH_INTERVAL_MS),
                "--research-metrics", "0",
                "--read-stage-metrics", "0",
                "--detailed-write-metrics", "0",
                "--memory-diagnostic", "1",
                "--memory-worker-startup-timeout-ms", str(STARTUP_TIMEOUT_MS),
                "--source-revision", SOURCE_SHA,
            ])
        else:
            command = [
                str(RAW_BINARY),
                "--backend", job["backend"],
                "--width", "16",
                "--clients", "64",
                "--seed", str(SEED),
                "--value-size", str(VALUE_SIZE),
                "--warmup-ms", str(WARMUP_MS),
                "--measurement-ms", str(MEASUREMENT_MS),
                "--call-cap", str(CALL_CAP),
                "--validate-histogram", "0",
                "--revision-allocator", "reserved-ranges",
                "--condition-key-lock-mode", "shared",
                "--engine-health-interval-ms", str(HEALTH_INTERVAL_MS),
                "--shared-blocking-slots", "32",
                "--memory-diagnostic", "1",
                "--memory-worker-startup-timeout-ms", str(STARTUP_TIMEOUT_MS),
                "--source-revision", SOURCE_SHA,
            ]
        job["command"] = command
    return jobs


def read_host_state():
    values = {}
    for line in Path("/proc/meminfo").read_text().splitlines():
        name, raw_value = line.split(":", 1)
        fields = raw_value.split()
        if fields:
            values[name] = int(fields[0]) * 1024
    return {"unix_time_ns": time.time_ns(), "memory_bytes": values}


def read_group_processes(process_group_id):
    processes = []
    for stat_path in Path("/proc").glob("[0-9]*/stat"):
        try:
            stat_text = stat_path.read_text()
            fields = stat_text[stat_text.rfind(")") + 2 :].split()
            if int(fields[2]) != process_group_id or fields[0] == "Z":
                continue
            process_id = int(stat_path.parent.name)
            status_values = {}
            for line in stat_path.with_name("status").read_text().splitlines():
                if line.startswith(("VmRSS:", "VmSwap:")):
                    label, value = line.split(":", 1)
                    status_values[label] = int(value.split()[0]) * 1024
            rollup_values = {}
            for line in stat_path.with_name("smaps_rollup").read_text().splitlines():
                if line.startswith(("Rss:", "Pss:", "Anonymous:", "FilePmdMapped:", "Swap:")):
                    label, value = line.split(":", 1)
                    rollup_values[label] = int(value.split()[0]) * 1024
            processes.append({
                "pid": process_id,
                "role": "harness-process" if process_id == process_group_id else "child-process",
                "rss_bytes": status_values.get("VmRSS", 0),
                "pss_bytes": rollup_values.get("Pss", 0),
                "anonymous_bytes": rollup_values.get("Anonymous", 0),
                "swap_bytes": max(status_values.get("VmSwap", 0), rollup_values.get("Swap", 0)),
            })
        except (FileNotFoundError, ProcessLookupError, ValueError, IndexError):
            continue
    return processes


def database_bytes(database_path):
    total = 0
    for file_path in database_path.rglob("*"):
        try:
            if file_path.is_file():
                total += file_path.stat().st_size
        except FileNotFoundError:
            pass
    return total


def live_members(process_group_id):
    return [entry["pid"] for entry in read_group_processes(process_group_id)]


def process_inventory():
    matches = []
    for command_path in Path("/proc").glob("[0-9]*/cmdline"):
        process_id = int(command_path.parent.name)
        if process_id == os.getpid():
            continue
        try:
            arguments = [part.decode(errors="replace") for part in command_path.read_bytes().split(b"\0") if part]
        except FileNotFoundError:
            continue
        if not arguments:
            continue
        executable_name = Path(arguments[0]).name
        if executable_name in {"cargo", "rustc", "service_workload", "unconditional_write_workload", "memory_diagnostic_worker"}:
            matches.append({"pid": process_id, "command": arguments[:4]})
    return matches


def terminate_group(process):
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    grace_deadline = time.monotonic() + 5
    while time.monotonic() < grace_deadline and live_members(process.pid):
        time.sleep(0.1)
    if live_members(process.pid):
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    process.wait()


def validate_report(report, job):
    if job["family"] == "A":
        configuration = report["configuration"]
        if report.get("backend") != job["backend"] or report.get("scenario") != "rw20":
            raise ValueError("conditional backend/profile mismatch")
        if configuration.get("clients") != 16 or configuration.get("initial_rows_per_tenant") != ROWS:
            raise ValueError("conditional input dimensions mismatch")
        if configuration.get("client_executor") != "dedicated" or configuration.get("mutation_workload") != "overwrite":
            raise ValueError("conditional executor/workload mismatch")
        if configuration.get("overwrite_slots_per_client") != 1024 or configuration.get("include_width_16") is not True:
            raise ValueError("conditional overwrite input mismatch")
        if configuration.get("rocksdb", {}).get("condition_key_lock_mode") != "shared" and job["backend"] != "blink":
            raise ValueError("conditional lock mode mismatch")
        if report.get("counters", {}).get("other_errors") != 0 or report.get("counters", {}).get("verification_errors"):
            raise ValueError("conditional unexpected errors")
        validations = list(report.get("validation", {}).values()) + list(report.get("reopen_validation", {}).values())
        if not validations or any(result.get("passed") is not True for result in validations):
            raise ValueError("conditional final/reopen validation failed")
        timing = report.get("timing", {})
        if timing.get("requested_measurement_ms") != MEASUREMENT_MS or timing.get("accepted_as_full_window") is not True:
            raise ValueError("conditional measurement window mismatch")
        if report.get("counters", {}).get("latency", {}).get("workflow", {}).get("count", 0) <= 0:
            raise ValueError("conditional latency sample count is empty")
    else:
        configuration = report["configuration"]
        if report.get("benchmark_kind") != "unconditional_write_probe":
            raise ValueError("unconditional benchmark kind mismatch")
        if configuration.get("clients") != 64 or configuration.get("width") != 16 or configuration.get("seed") != SEED:
            raise ValueError("unconditional input dimensions mismatch")
        if configuration.get("value_size_bytes") != VALUE_SIZE or configuration.get("overwrite_slots_per_client") != 256:
            raise ValueError("unconditional key pool mismatch")
        if configuration.get("caller_threads") != 64 or configuration.get("executor") != "one dedicated caller thread per client":
            raise ValueError("unconditional caller configuration mismatch")
        if configuration.get("wal_sync") is not True or configuration.get("engine_health_interval_ms") != HEALTH_INTERVAL_MS:
            raise ValueError("unconditional durability/health mismatch")
        if report.get("counters", {}).get("other_errors") != 0 or report.get("counters", {}).get("overloads") != 0:
            raise ValueError("unconditional unexpected errors")
        if report.get("final_validation", {}).get("passed") is not True or report.get("reopen_validation", {}).get("passed") is not True:
            raise ValueError("unconditional final/reopen validation failed")
        timing = report.get("timing", {})
        if timing.get("requested_measurement_ms") != MEASUREMENT_MS or timing.get("measurement_complete") is not True:
            raise ValueError("unconditional measurement window mismatch")
        if report.get("latency", {}).get("count", 0) <= 0:
            raise ValueError("unconditional latency sample count is empty")
    validate_histograms(report)
    diagnostic = report.get("memory_only_diagnostic", {})
    if diagnostic.get("source_revision") != SOURCE_SHA or diagnostic.get("host_samples_truncated") is not False:
        raise ValueError("memory diagnostic provenance/host coverage mismatch")
    expected_binary_hash = SERVICE_BINARY_SHA256 if job["binary"] == "service" else RAW_BINARY_SHA256
    if diagnostic.get("harness_binary_sha256") != expected_binary_hash:
        raise ValueError("memory diagnostic binary provenance mismatch")
    if diagnostic.get("process_memory_samples_truncated") is not False:
        raise ValueError("process memory sample coverage is incomplete")
    workers = [entry for entry in diagnostic.get("processes", []) if entry.get("role") == "backend-process"]
    if len(workers) != 2:
        raise ValueError("initial and reopened backend identities are missing")
    lifecycle = diagnostic.get("worker_lifecycle", [])
    if sum(event.get("event") == "started" for event in lifecycle) != 2 or sum(event.get("event") == "stopped" for event in lifecycle) != 2:
        raise ValueError("worker lifecycle is incomplete")
    health_reports = diagnostic.get("engine_health_reports", [])
    if len(health_reports) != 2 or any(entry.get("shutdown_complete") is not True for entry in health_reports):
        raise ValueError("worker shutdown health report is incomplete")
    for entry in health_reports:
        health = entry.get("engine_health", {})
        if job["backend"] != "blink":
            if health.get("status") == "sampled":
                if health.get("coverage_complete") is not True or health.get("truncated") is not False or health.get("sample_count", 0) < 25:
                    raise ValueError("RocksDB engine-health coverage is incomplete")
                for sample in health.get("samples", []):
                    for database in sample.get("databases", []):
                        if not database.get("properties"):
                            raise ValueError("engine property sample is empty")
                        if any(prop.get("status") not in {"value", "unsupported", "get_int_property_unavailable", "native_property_error"} for prop in database["properties"].values()):
                            raise ValueError("engine property status is ambiguous")
            elif health.get("status") != "disabled":
                raise ValueError("RocksDB engine health has an unexpected state")
            shutdown_snapshot = entry.get("shutdown_property_snapshot", {})
            if shutdown_snapshot.get("databases"):
                for database in shutdown_snapshot["databases"]:
                    if database.get("status") != "open" or not database.get("properties"):
                        raise ValueError("shutdown property snapshot is incomplete")
                    if any(prop.get("status") not in {"value", "unsupported", "get_int_property_unavailable", "native_property_error"} for prop in database["properties"].values()):
                        raise ValueError("shutdown property status is ambiguous")
            else:
                raise ValueError("shutdown property snapshot is missing")
        elif health.get("status") != "unsupported" and entry.get("shutdown_property_snapshot", {}).get("status") != "unsupported":
            raise ValueError("Blink health support status mismatch")
    residency_scans = []

    def collect_residency_scans(value):
        if isinstance(value, dict):
            if "served_from_ram_for_second_bounded_scan" in value:
                residency_scans.append(value)
            else:
                for nested_value in value.values():
                    collect_residency_scans(nested_value)
        elif isinstance(value, list):
            for nested_value in value:
                collect_residency_scans(nested_value)

    collect_residency_scans(diagnostic.get("residency_scans"))
    expected_scan_count = report.get("configuration", {}).get("tenant_count", 1) if job["family"] == "A" else 1
    if len(residency_scans) != expected_scan_count:
        raise ValueError("bounded residency scan count mismatch")
    for scan in residency_scans:
        if scan.get("reopen_oracle_passed") is not True or scan.get("scan_fingerprints_match") is not True:
            raise ValueError("residency scan did not match the reopened oracle")
        if scan.get("same_backend_pid_and_starttime_before_and_after") is not True or scan.get("same_vdev_mapping_before_and_after") is not True:
            raise ValueError("residency scan process or vdev identity changed")
        if scan.get("second_scan_physical_read_delta_bytes") is None or scan.get("backend_swap_before_kb") is None or scan.get("backend_swap_after_kb") is None:
            raise ValueError("residency scan counter or swap observation is missing")
    return report


def preserve_and_remove_database(database_path, case_directory):
    files = []
    log_paths = []
    native_log_root = case_directory / "native-logs"
    for file_path in sorted(database_path.rglob("*")):
        if not file_path.is_file():
            continue
        relative_path = file_path.relative_to(database_path)
        record = {"path": str(relative_path), "size_bytes": file_path.stat().st_size, "sha256": sha256_file(file_path)}
        if file_path.name == "LOG" or file_path.name.startswith("LOG.old."):
            destination = native_log_root / relative_path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(file_path, destination)
            if sha256_file(destination) != record["sha256"] or destination.stat().st_size != record["size_bytes"]:
                raise ValueError(f"native log preservation mismatch: {relative_path}")
            record["preserved_log_path"] = str(destination.relative_to(case_directory))
            log_paths.append(str(relative_path))
        files.append(record)
    if not log_paths and not case_directory.name.endswith("blink"):
        raise ValueError("expected native LOG files were not preserved")
    manifest = {
        "database_path": str(database_path),
        "file_count": len(files),
        "files": files,
        "native_log_count": len(log_paths),
        "database_removed_after_validation": False,
    }
    manifest_path = case_directory / "database-manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    if sum(entry["size_bytes"] for entry in files) > DATABASE_CAP_BYTES:
        raise ValueError("database exceeded the approved cap; database preserved")
    shutil.rmtree(database_path)
    manifest["database_removed_after_validation"] = True
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")


def host_preflight():
    memory = read_host_state()["memory_bytes"]
    disk = shutil.disk_usage(DISK_ROOT)
    if disk.free < MINIMUM_FREE_BYTES:
        raise RuntimeError("ZFS free space is below 26 GiB")
    if memory.get("MemAvailable", 0) < MINIMUM_AVAILABLE_BYTES:
        raise RuntimeError("MemAvailable is below the 3 GiB cap plus 1.5 GiB startup reserve")
    active_processes = process_inventory()
    if active_processes:
        raise RuntimeError(f"preflight found active build/workload processes: {active_processes}")
    return {
        "timestamp_unix_ns": time.time_ns(),
        "hostname": subprocess.run(["hostname"], check=True, capture_output=True, text=True).stdout.strip(),
        "architecture": os.uname().machine,
        "online_cpu_count": os.cpu_count(),
        "memory_bytes": memory,
        "filesystem_path": str(DISK_ROOT),
        "filesystem_free_bytes": disk.free,
        "active_build_or_workload_processes": active_processes,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan-only", action="store_true")
    parser.add_argument("--run", action="store_true")
    arguments = parser.parse_args()
    if arguments.plan_only == arguments.run:
        raise SystemExit("choose exactly one of --plan-only or --run")
    if subprocess.run(["git", "-C", str(SOURCE_PATH), "rev-parse", "HEAD"], check=True, capture_output=True, text=True).stdout.strip() != SOURCE_SHA:
        raise RuntimeError("diagnostic source SHA mismatch")
    if subprocess.run(["git", "-C", str(SOURCE_PATH), "status", "--porcelain"], check=True, capture_output=True, text=True).stdout.strip():
        raise RuntimeError("diagnostic source checkout is dirty")
    expected_binaries = {
        "service_workload": (SERVICE_BINARY, SERVICE_BINARY_SHA256),
        "unconditional_write_workload": (RAW_BINARY, RAW_BINARY_SHA256),
        "memory_diagnostic_worker": (WORKER_BINARY, WORKER_BINARY_SHA256),
    }
    for binary_path, expected_hash in expected_binaries.values():
        if not binary_path.is_file() or sha256_file(binary_path) != expected_hash:
            raise RuntimeError(f"frozen binary SHA mismatch: {binary_path}")
    jobs = selected_jobs()
    plan = {
        "schema": "memory-diagnostic-7x30-v1",
        "source_sha": SOURCE_SHA,
        "binary_sha256": {name: sha256_file(path) for name, (path, _) in expected_binaries.items()},
        "jobs": jobs,
        "selected_job_count": len(jobs),
        "input": {"seed": SEED, "rows": ROWS, "value_size": VALUE_SIZE, "warmup_ms": WARMUP_MS, "measurement_ms": MEASUREMENT_MS, "health_interval_ms": HEALTH_INTERVAL_MS, "call_cap": CALL_CAP, "max_write_ledger": MAX_WRITE_LEDGER, "stage_diagnostics": False},
        "caps": {"process_group_rss_bytes": RSS_CAP_BYTES, "database_bytes": DATABASE_CAP_BYTES, "minimum_free_bytes": MINIMUM_FREE_BYTES, "minimum_available_before_start_bytes": MINIMUM_AVAILABLE_BYTES, "minimum_available_during_run_bytes": RUNTIME_AVAILABLE_RESERVE_BYTES, "case_timeout_seconds": CASE_TIMEOUT_SECONDS, "worker_startup_timeout_ms": STARTUP_TIMEOUT_MS},
        "scope": "memory-only parent/backend process and bounded residency diagnostics; not a replacement throughput ranking",
    }
    plan_fingerprint = hashlib.sha256(json.dumps(plan, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    plan["plan_fingerprint"] = plan_fingerprint
    if arguments.plan_only:
        preflight = host_preflight()
        output = {"plan": plan, "preflight": preflight}
        RESULT_ROOT.mkdir(parents=True, exist_ok=True)
        (RESULT_ROOT / "plan-only.json").write_text(json.dumps(output, indent=2) + "\n")
        print(json.dumps(output, separators=(",", ":")))
        return
    if not RESULT_ROOT.is_dir() or not (RESULT_ROOT / "plan-only.json").is_file():
        raise RuntimeError("plan-only evidence must exist before execution")
    saved_plan = json.loads((RESULT_ROOT / "plan-only.json").read_text())["plan"]
    if saved_plan.get("plan_fingerprint") != plan_fingerprint:
        raise RuntimeError("saved plan fingerprint mismatch")
    preflight = host_preflight()
    runtime = {
        "source_sha": SOURCE_SHA,
        "plan_fingerprint": plan_fingerprint,
        "preflight": preflight,
        "jobs": {},
        "started_unix_ns": time.time_ns(),
    }
    (RESULT_ROOT / "run-preflight.json").write_text(json.dumps(runtime, indent=2) + "\n")
    RESULT_ROOT.mkdir(parents=True, exist_ok=True)
    for job in jobs:
        case_id = job["case_id"]
        case_directory = RESULT_ROOT / case_id
        database_path = DATABASE_ROOT / case_id
        command = list(job["command"]) + ["--data-dir", str(database_path), "--report", str(case_directory / "report.json")]
        current_binary_path = SERVICE_BINARY if job["binary"] == "service" else RAW_BINARY
        expected_binary_hash = SERVICE_BINARY_SHA256 if job["binary"] == "service" else RAW_BINARY_SHA256
        if sha256_file(current_binary_path) != expected_binary_hash or sha256_file(WORKER_BINARY) != WORKER_BINARY_SHA256:
            raise RuntimeError("frozen binary changed before case start")
        active_processes = process_inventory()
        if active_processes:
            raise RuntimeError(f"another build/workload appeared before case start: {active_processes}")
        if case_directory.exists():
            command_manifest_path = case_directory / "command.json"
            report_path = case_directory / "report.json"
            if not command_manifest_path.is_file() or not report_path.is_file():
                raise RuntimeError(f"existing case lacks command/report evidence: {case_id}")
            command_manifest = json.loads(command_manifest_path.read_text())
            if command_manifest.get("command") != command or command_manifest.get("source_sha") != SOURCE_SHA or command_manifest.get("binary_sha256") != expected_binary_hash or command_manifest.get("plan_fingerprint") != plan_fingerprint or command_manifest.get("database_path") != str(database_path):
                raise RuntimeError(f"existing case provenance mismatch: {case_id}")
            case_status_path = case_directory / "status.txt"
            if case_status_path.is_file() and case_status_path.read_text().strip() == "complete":
                report = validate_report(json.loads(report_path.read_text()), job)
                if database_path.exists():
                    raise RuntimeError(f"completed case unexpectedly retains database: {case_id}")
                runtime["jobs"][case_id] = {"exit_status": 0, "watchdog_failure": None, "live_sample_count": command_manifest.get("live_sample_count"), "status": "complete", "resumed_existing_evidence": True}
                (RESULT_ROOT / "run-preflight.json").write_text(json.dumps(runtime, indent=2) + "\n")
                continue
            if case_status_path.exists() or command_manifest.get("exit_status") != 0 or command_manifest.get("watchdog_failure") is not None or not database_path.is_dir():
                raise RuntimeError(f"existing case is incomplete or failed and must remain preserved: {case_id}")
            expected_samples = command_manifest.get("live_sample_count")
            live_sample_path = case_directory / "live-budget-samples.jsonl"
            live_samples = [json.loads(line) for line in live_sample_path.read_text().splitlines() if line]
            if expected_samples != len(live_samples) or any(sample["process_group_rss_bytes"] > RSS_CAP_BYTES or sample["mem_available_bytes"] < RUNTIME_AVAILABLE_RESERVE_BYTES or (sample.get("database_bytes") is not None and sample["database_bytes"] > DATABASE_CAP_BYTES) for sample in live_samples):
                raise RuntimeError(f"existing case watchdog evidence is incomplete or over limit: {case_id}")
            if (case_directory / "stderr.log").read_text():
                raise RuntimeError(f"existing case has stderr output: {case_id}")
            report = validate_report(json.loads(report_path.read_text()), job)
            if sha256_file(current_binary_path) != expected_binary_hash or sha256_file(WORKER_BINARY) != WORKER_BINARY_SHA256:
                raise RuntimeError("frozen binary changed during recovered case validation")
            (case_directory / "report-validated.json").write_text(json.dumps({"passed": True, "actual_rows": report.get("validation", {}).get("tenant_1", {}).get("actual_rows", report.get("final_validation", {}).get("actual_rows")), "measurement_ms": MEASUREMENT_MS, "plan_fingerprint": plan_fingerprint, "revalidated_without_rerun": True}, indent=2) + "\n")
            preserve_and_remove_database(database_path, case_directory)
            case_status_path.write_text("complete\n")
            runtime["jobs"][case_id] = {"exit_status": 0, "watchdog_failure": None, "live_sample_count": expected_samples, "status": "complete", "revalidated_without_rerun": True}
            (RESULT_ROOT / "run-preflight.json").write_text(json.dumps(runtime, indent=2) + "\n")
            continue
        case_directory.mkdir(parents=True)
        if database_path.exists():
            raise RuntimeError(f"database path already exists: {database_path}")
        database_path.mkdir(parents=True)
        command_manifest = {
            "case": job,
            "command": command,
            "source_sha": SOURCE_SHA,
            "binary_sha256": sha256_file(current_binary_path),
            "plan_fingerprint": plan_fingerprint,
            "started_unix_ns": time.time_ns(),
            "database_path": str(database_path),
        }
        (case_directory / "command.json").write_text(json.dumps(command_manifest, indent=2) + "\n")
        env = os.environ.copy()
        env["LC_ALL"] = "C"
        stdout_path = case_directory / "stdout.log"
        stderr_path = case_directory / "stderr.log"
        with stdout_path.open("w") as stdout_file, stderr_path.open("w") as stderr_file:
            process = subprocess.Popen(command, stdout=stdout_file, stderr=stderr_file, start_new_session=True, env=env)
            started_monotonic = time.monotonic()
            next_database_sample = started_monotonic
            live_samples = []
            failure_reason = None
            while process.poll() is None:
                time.sleep(1)
                if process.poll() is not None:
                    break
                now = time.monotonic()
                sampled_processes = read_group_processes(process.pid)
                host = read_host_state()
                database_sampled = now >= next_database_sample
                current_database_bytes = database_bytes(database_path) if database_sampled else None
                if database_sampled:
                    next_database_sample = now + 5
                sample = {
                    "unix_time_ns": host["unix_time_ns"],
                    "process_group_id": process.pid,
                    "process_group_rss_bytes": sum(entry["rss_bytes"] for entry in sampled_processes),
                    "process_group_pss_bytes": sum(entry["pss_bytes"] for entry in sampled_processes),
                    "process_group_swap_bytes": sum(entry["swap_bytes"] for entry in sampled_processes),
                    "processes": sampled_processes,
                    "mem_available_bytes": host["memory_bytes"].get("MemAvailable", 0),
                    "database_bytes": current_database_bytes,
                    "database_sampled": database_sampled,
                }
                live_samples.append(sample)
                with (case_directory / "live-budget-samples.jsonl").open("a") as live_output:
                    live_output.write(json.dumps(sample, separators=(",", ":")) + "\n")
                    live_output.flush()
                if sample["process_group_rss_bytes"] > RSS_CAP_BYTES:
                    failure_reason = "process-group RSS exceeded 3 GiB cap"
                elif sample["mem_available_bytes"] < RUNTIME_AVAILABLE_RESERVE_BYTES:
                    failure_reason = "MemAvailable fell below 1.5 GiB runtime reserve"
                elif current_database_bytes is not None and current_database_bytes > DATABASE_CAP_BYTES:
                    failure_reason = "live database exceeded 12 GiB cap"
                elif now - started_monotonic > CASE_TIMEOUT_SECONDS:
                    failure_reason = "case exceeded 900 second timeout"
                if failure_reason:
                    terminate_group(process)
                    break
            exit_status = process.wait()
        command_manifest.update({"exit_status": exit_status, "finished_unix_ns": time.time_ns(), "live_sample_count": len(live_samples), "watchdog_failure": failure_reason})
        (case_directory / "command.json").write_text(json.dumps(command_manifest, indent=2) + "\n")
        runtime["jobs"][case_id] = {"exit_status": exit_status, "watchdog_failure": failure_reason, "live_sample_count": len(live_samples)}
        (RESULT_ROOT / "run-preflight.json").write_text(json.dumps(runtime, indent=2) + "\n")
        if failure_reason or exit_status != 0:
            (case_directory / "status.txt").write_text("failed\n")
            raise RuntimeError(f"{case_id} failed; database and original evidence preserved")
        report = validate_report(json.loads((case_directory / "report.json").read_text()), job)
        if sha256_file(current_binary_path) != expected_binary_hash or sha256_file(WORKER_BINARY) != WORKER_BINARY_SHA256:
            raise RuntimeError("frozen binary changed during case")
        if any(sample["process_group_rss_bytes"] > RSS_CAP_BYTES for sample in live_samples):
            raise ValueError(f"{case_id} process group exceeded RSS cap")
        (case_directory / "report-validated.json").write_text(json.dumps({"passed": True, "actual_rows": report.get("final_validation", {}).get("actual_rows"), "measurement_ms": MEASUREMENT_MS, "plan_fingerprint": plan_fingerprint}, indent=2) + "\n")
        preserve_and_remove_database(database_path, case_directory)
        (case_directory / "status.txt").write_text("complete\n")
        runtime["jobs"][case_id]["status"] = "complete"
        (RESULT_ROOT / "run-preflight.json").write_text(json.dumps(runtime, indent=2) + "\n")
    runtime["finished_unix_ns"] = time.time_ns()
    runtime["status"] = "complete"
    (RESULT_ROOT / "run-preflight.json").write_text(json.dumps(runtime, indent=2) + "\n")
    (RESULT_ROOT / "run-status.txt").write_text("complete\n")
    print(json.dumps({"status": "complete", "completed_cases": len(runtime["jobs"]), "plan_fingerprint": plan_fingerprint}, separators=(",", ":")))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"memory-diagnostic-7x30: {error}", file=sys.stderr)
        raise
