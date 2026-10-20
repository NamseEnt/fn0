import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path("/bench/zfs/db/rocksdb-contract-baseline-20261016/minimal-native-baseline-20261020")
TARGET = Path("/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target/release")
SERVICE_BINARY = TARGET / "service_workload"
UNCONDITIONAL_BINARY = TARGET / "unconditional_write_workload"
REPORTS = ROOT / "reports"
DATABASES = ROOT / "databases"
SAMPLE_INTERVAL_SECONDS = 1.0
MAX_PROCESS_GROUP_RSS_BYTES = 3 * 1024**3
MINIMUM_MEM_AVAILABLE_BYTES = 1500 * 1024**2
MAXIMUM_DATABASE_BYTES = 4 * 1024**3
MINIMUM_DISK_AVAILABLE_BYTES = 10 * 1024**3
CHILD_TIMEOUT_SECONDS = 120


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as source_file:
        while chunk := source_file.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def available_bytes(path):
    return shutil.disk_usage(path).free


def memory_available_bytes():
    for line in Path("/proc/meminfo").read_text().splitlines():
        if line.startswith("MemAvailable:"):
            return int(line.split()[1]) * 1024
    raise RuntimeError("MemAvailable is missing from /proc/meminfo")


def process_group_rss_bytes(process_group):
    total_bytes = 0
    for process_path in Path("/proc").iterdir():
        if not process_path.name.isdigit():
            continue
        try:
            fields = (process_path / "stat").read_text().rsplit(")", 1)[1].split()
            if int(fields[2]) != process_group:
                continue
            status_lines = (process_path / "status").read_text().splitlines()
            resident_kb = next(
                int(line.split()[1])
                for line in status_lines
                if line.startswith("VmRSS:")
            )
            total_bytes += resident_kb * 1024
        except (OSError, StopIteration, ValueError, IndexError):
            continue
    return total_bytes


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


def validation_passed(value):
    if isinstance(value, dict):
        if "passed" in value:
            return value["passed"] is True
        return bool(value) and all(validation_passed(result) for result in value.values())
    return value is True


def process_sample(process_group, database_path):
    return {
        "elapsed_monotonic_seconds": time.monotonic(),
        "process_group_rss_bytes": process_group_rss_bytes(process_group),
        "mem_available_bytes": memory_available_bytes(),
        "database_bytes": directory_bytes(database_path),
        "filesystem_available_bytes": available_bytes(ROOT),
    }


def wait_for_case(command, case_id, database_path, report_path):
    case_root = REPORTS / case_id
    case_root.mkdir(parents=True, exist_ok=False)
    binary_digest = sha256_file(Path(command[0]))
    source_revision = subprocess.check_output(
        ["git", "-C", str(ROOT / "source"), "rev-parse", "HEAD"], text=True
    ).strip()
    process_environment = os.environ.copy()
    process_environment["DODB_SOURCE_SHA"] = source_revision
    process_environment["DODB_BINARY_SHA256"] = binary_digest
    command_record = {
        "argv": [str(argument) for argument in command],
        "cwd": str(ROOT / "source"),
        "case_id": case_id,
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
    child = subprocess.Popen(
        command,
        cwd=ROOT / "source",
        stdout=(case_root / "workload.stdout.log").open("wb"),
        stderr=(case_root / "workload.stderr.log").open("wb"),
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
        "minimum_filesystem_available_bytes": min(sample["filesystem_available_bytes"] for sample in samples),
        "report_exists": report_path.is_file(),
    }
    (case_root / "controller-result.json").write_text(json.dumps(run_result, indent=2) + "\n")
    if stop_reason is not None or exit_code != 0 or not report_path.is_file():
        raise RuntimeError(f"case failed and database is preserved: {run_result}")
    report_value = json.loads(report_path.read_text())
    (case_root / "workload-report-sha256.txt").write_text(sha256_file(report_path) + "\n")
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
        manifest.append({
            "path": relative_path,
            "size_bytes": file_path.stat().st_size,
            "sha256": sha256_file(file_path),
        })
        if file_path.name == "LOG" or file_path.name.startswith("LOG."):
            preserved_log = case_root / "rocksdb-logs" / relative_path.replace("/", "__")
            preserved_log.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(file_path, preserved_log)
            log_contents.append({
                "path": relative_path,
                "preserved_path": str(preserved_log),
                "size_bytes": file_path.stat().st_size,
                "sha256": sha256_file(preserved_log),
            })
    (case_root / "database-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    (case_root / "rocksdb-log-manifest.json").write_text(json.dumps(log_contents, indent=2) + "\n")
    run_result["database_file_count"] = len(manifest)
    run_result["database_bytes_before_removal"] = sum(item["size_bytes"] for item in manifest)
    run_result["rocksdb_log_file_count"] = len(log_contents)
    (case_root / "controller-result.json").write_text(json.dumps(run_result, indent=2) + "\n")
    shutil.rmtree(database_path)


def service_command(backend, case_id, database_path, report_path, clients, admission_slots):
    command = [
        SERVICE_BINARY,
        "--backend", backend,
        "--client-executor", "dedicated",
        "--scenario", "rw20",
        "--data-dir", database_path,
        "--report", report_path,
        "--seed", "20261013",
        "--rows", "10000",
        "--value-size", "512",
        "--clients", str(clients),
        "--warmup-ms", "10000",
        "--measurement-ms", "30000",
        "--call-cap", "100000000",
        "--max-write-ledger", "5000000",
        "--require-full-window", "1",
        "--mutation-workload", "overwrite",
        "--overwrite-slots-per-client", "1024",
        "--latency-storage", "log1024",
        "--include-width-16", "1",
        "--revision-allocator", "reserved-ranges",
        "--condition-key-lock-mode", "exclusive",
    ]
    if admission_slots is not None:
        command.extend(["--shared-blocking-slots", str(admission_slots)])
    return command


def unconditional_command(backend, database_path, report_path):
    command = [
        UNCONDITIONAL_BINARY,
        "--backend", backend,
        "--width", "16",
        "--clients", "64",
        "--seed", "20261013",
        "--value-size", "512",
        "--warmup-ms", "10000",
        "--measurement-ms", "30000",
        "--call-cap", "100000000",
        "--revision-allocator", "reserved-ranges",
        "--condition-key-lock-mode", "exclusive",
        "--engine-health-interval-ms", "0",
        "--data-dir", database_path,
        "--report", report_path,
    ]
    if backend == "rocksdb-service":
        command.extend(["--shared-blocking-slots", "32"])
    return command


def run_matrix():
    REPORTS.mkdir(parents=True, exist_ok=True)
    DATABASES.mkdir(parents=True, exist_ok=True)
    binary_hashes = {
        "service_workload": sha256_file(SERVICE_BINARY),
        "unconditional_write_workload": sha256_file(UNCONDITIONAL_BINARY),
    }
    (ROOT / "binary-hashes.json").write_text(json.dumps(binary_hashes, indent=2) + "\n")
    path_orders = {
        1: ["minimal-native", "rocksdb-direct", "blink"],
        2: ["rocksdb-direct", "blink", "minimal-native"],
        3: ["blink", "minimal-native", "rocksdb-direct"],
    }
    for repetition, backends in path_orders.items():
        for backend in backends:
            path_name = {
                "minimal-native": "minimal-native",
                "rocksdb-direct": "existing-core",
                "blink": "blink",
            }[backend]
            case_id = f"A-{path_name}-r{repetition}"
            database_path = DATABASES / case_id
            report_path = REPORTS / f"{case_id}.json"
            admission_slots = 16 if backend == "rocksdb-direct" else None
            command = service_command(
                backend, case_id, database_path, report_path, 16, admission_slots
            )
            wait_for_case(command, case_id, database_path, report_path)
    for repetition, backends in path_orders.items():
        for backend in backends:
            backend_name = {
                "minimal-native": "minimal-native",
                "rocksdb-direct": "synchronous-core",
                "blink": "blink",
            }[backend]
            case_id = f"E-{backend_name}-r{repetition}"
            database_path = DATABASES / case_id
            report_path = REPORTS / f"{case_id}.json"
            workload_backend = {
                "minimal-native": "minimal-native",
                "synchronous-core": "synchronous-core",
                "blink": "blink",
            }[backend_name]
            command = unconditional_command(workload_backend, database_path, report_path)
            wait_for_case(command, case_id, database_path, report_path)
    (ROOT / "matrix-complete.txt").write_text("18/18 cases completed and validated\n")


if __name__ == "__main__":
    try:
        run_matrix()
    except Exception as error:
        print(str(error), file=sys.stderr)
        raise
