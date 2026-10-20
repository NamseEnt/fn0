import argparse
import hashlib
import json
import os
import shutil
import subprocess
import time
from pathlib import Path


SOURCE_SHA = "ee6cd94f1a4e7b2bab7af603bc8983bbf8184542"
SERVICE_BINARY_SHA256 = "7c8b35679df63402b3454067d720c53e1a194f5be6fe459eb7bec35862bdd4b5"
RAW_BINARY_SHA256 = "97b2b38717679bba4000a9eed2bf8c2435114e3d7fa5717e9727381e3586e3f4"
SCHEMA_VERSION = 1
SCHEMA_TEXT = "exact integer nanoseconds below 1024; 1024 equal-width sub-buckets per binary exponent thereafter; percentile is the bucket upper bound"


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
    if histogram["histogram_schema_version"] != SCHEMA_VERSION:
        raise ValueError("histogram schema version mismatch")
    if histogram["histogram_binning_schema"] != SCHEMA_TEXT:
        raise ValueError("histogram schema description mismatch")
    bins = histogram["nonzero_bins"]
    if bins != sorted(bins, key=lambda bin_entry: bin_entry[0]):
        raise ValueError("histogram bins are not sorted")
    if len({bin_entry[0] for bin_entry in bins}) != len(bins):
        raise ValueError("histogram contains duplicate bin indices")
    if sum(bin_entry[1] for bin_entry in bins) != histogram["count"]:
        raise ValueError("histogram bin sum differs from latency count")
    percentile_fields = ["p50_ms", "p95_ms", "p99_ms"] if "p50_ms" in histogram else ["p50_nanos", "p95_nanos", "p99_nanos"]
    for percentile, field_name in zip((50, 95, 99), percentile_fields):
        if histogram["count"] == 0:
            if histogram[field_name] is not None:
                raise ValueError(f"{field_name} must be null for an empty histogram")
            continue
        target_rank = max((histogram["count"] * percentile + 99) // 100, 1)
        if field_name.endswith("_ms"):
            target_rank = ((histogram["count"] - 1) * percentile + 99) // 100 + 1
        cumulative_count = 0
        upper_bound = 0
        for bucket_index, bucket_count in bins:
            cumulative_count += bucket_count
            if cumulative_count >= target_rank:
                upper_bound = histogram_upper_bound(bucket_index)
                break
        expected = upper_bound / 1_000_000 if field_name.endswith("_ms") else upper_bound
        if histogram[field_name] != expected:
            raise ValueError(f"{field_name} differs from sparse-bin percentile reconstruction")


def validate_histograms(value):
    if isinstance(value, dict):
        if value.get("storage", "").startswith("log1024"):
            validate_histogram(value)
        for nested_value in value.values():
            validate_histograms(nested_value)
    elif isinstance(value, list):
        for nested_value in value:
            validate_histograms(nested_value)


def validate_service_report(report, backend, scenario):
    configuration = report["configuration"]
    if report["backend"] != backend or report["scenario"] != scenario:
        raise ValueError("service backend or scenario mismatch")
    if configuration["clients"] != 16 or configuration["mutation_workload"] != "new-key":
        raise ValueError("service workload configuration mismatch")
    if configuration["include_width_16"] is not True:
        raise ValueError("width 16 was not enabled")
    if report["timing"]["accepted_as_full_window"] is not True:
        raise ValueError("service measurement did not complete the full window")
    for validation in list(report["validation"].values()) + list(report["reopen_validation"].values()):
        if validation["passed"] is not True:
            raise ValueError("service final or reopen oracle failed")
    if report["counters"]["other_errors"] != 0 or report["counters"]["verification_errors"]:
        raise ValueError("service recorded unexpected errors")
    if configuration["rocksdb"]["engine_health_interval_ms"] != 1_000:
        raise ValueError("service health interval mismatch")
    validate_histograms(report)
    health = report["engine_health"]
    if backend == "blink":
        if health.get("status") != "unsupported":
            raise ValueError("Blink health support was not reported as unsupported")
    else:
        if health.get("status") != "sampled" or health["sample_count"] < 3:
            raise ValueError("RocksDB health sampling did not cover the smoke interval")
        if health["coverage_complete"] is not True or health["truncated"] is not False:
            raise ValueError("RocksDB health sampling coverage is incomplete")
        if not health["samples"] or not health["samples"][0]["databases"]:
            raise ValueError("RocksDB health sample has no database observations")
        for sample in health["samples"]:
            for database in sample["databases"]:
                if database["status"] != "open" or len(database["properties"]) != 9:
                    raise ValueError("RocksDB property sample is incomplete")
                for prop in database["properties"].values():
                    if prop["status"] not in {"value", "unsupported", "get_int_property_unavailable", "native_property_error"}:
                        raise ValueError("RocksDB property status is not explicit")
                    if prop["status"] == "value" and prop["value"] is None:
                        raise ValueError("RocksDB property value is missing")
        if backend == "rocksdb-direct" and scenario == "mt-mix":
            database_names = [database["database"] for database in health["samples"][0]["databases"]]
            if len(database_names) != 4 or set(database_names) != {f"tenant_{tenant_number}" for tenant_number in range(1, 5)}:
                raise ValueError("core health observers are not uniquely mapped to all four tenants")
    return health


def validate_raw_report(report):
    configuration = report["configuration"]
    if report["backend"] != "RawRocksDb":
        raise ValueError("raw backend mismatch")
    if configuration["clients"] != 64 or configuration["width"] != 16:
        raise ValueError("raw smoke dimensions mismatch")
    if configuration["condition_key_lock_mode"] != "not-applicable-no-conditional-transactions":
        raise ValueError("raw condition lock mode was not reported as not applicable")
    if report["timing"]["measurement_complete"] is not True:
        raise ValueError("raw measurement did not complete")
    if report["counters"]["other_errors"] != 0 or report["counters"]["overloads"] != 0:
        raise ValueError("raw recorded unexpected errors")
    if report["final_validation"]["passed"] is not True or report["reopen_validation"]["passed"] is not True:
        raise ValueError("raw final or reopen oracle failed")
    health = report["engine_health"]
    if health.get("status") != "sampled" or health["sample_count"] < 3:
        raise ValueError("raw health sampling did not cover the smoke interval")
    if health["coverage_complete"] is not True or health["truncated"] is not False:
        raise ValueError("raw health sampling coverage is incomplete")
    for sample in health["samples"]:
        if len(sample["databases"]) != 1 or sample["databases"][0]["database"] != "raw_database":
            raise ValueError("raw health sample is not mapped to the raw database")
        database = sample["databases"][0]
        if database["status"] != "open" or len(database["properties"]) != 9:
            raise ValueError("raw property sample is incomplete")
        for prop in database["properties"].values():
            if prop["status"] not in {"value", "unsupported", "get_int_property_unavailable", "native_property_error"}:
                raise ValueError("raw property status is not explicit")
            if prop["status"] == "value" and prop["value"] is None:
                raise ValueError("raw property value is missing")
    validate_histograms(report)
    return health


def preserve_database(database_path, artifact_path):
    files = []
    for path in sorted(database_path.rglob("*")):
        if path.is_file():
            relative_path = path.relative_to(database_path)
            digest = sha256_file(path)
            record = {"path": str(relative_path), "size_bytes": path.stat().st_size, "sha256": digest}
            if path.name == "LOG":
                log_copy = artifact_path / "database-logs" / relative_path
                log_copy.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(path, log_copy)
                record["preserved_log_path"] = str(log_copy.relative_to(artifact_path))
            files.append(record)
    manifest = {"database_path": str(database_path), "file_count": len(files), "files": files}
    (artifact_path / "database-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    for path_record in files:
        if "preserved_log_path" in path_record:
            preserved_path = artifact_path / path_record["preserved_log_path"]
            if preserved_path.stat().st_size != path_record["size_bytes"] or sha256_file(preserved_path) != path_record["sha256"]:
                raise ValueError("preserved RocksDB LOG differs from its source")
    shutil.rmtree(database_path)
    manifest["database_removed_after_validation"] = True
    (artifact_path / "database-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


def run_case(binary_path, result_root, case_name, arguments, report_validator):
    artifact_path = result_root / case_name
    artifact_path.mkdir(parents=True, exist_ok=False)
    database_path = result_root / "databases" / case_name
    database_path.mkdir(parents=True, exist_ok=False)
    report_path = artifact_path / "report.json"
    stdout_path = artifact_path / "stdout.txt"
    stderr_path = artifact_path / "stderr.txt"
    command = [str(binary_path), *arguments, "--data-dir", str(database_path), "--report", str(report_path)]
    manifest = {"source_sha": SOURCE_SHA, "binary_sha256": sha256_file(binary_path), "command": command, "started_unix_seconds": time.time()}
    (artifact_path / "command.json").write_text(json.dumps(manifest, indent=2) + "\n")
    environment = os.environ.copy()
    environment["DODB_SOURCE_SHA"] = SOURCE_SHA
    environment["DODB_BINARY_SHA256"] = manifest["binary_sha256"]
    try:
        completed = subprocess.run(command, capture_output=True, text=True, timeout=180, env=environment)
    except subprocess.TimeoutExpired as error:
        stdout_path.write_text(error.stdout or "")
        stderr_path.write_text(error.stderr or "")
        manifest.update({"timed_out": True, "database_preserved": True, "finished_unix_seconds": time.time()})
        (artifact_path / "command.json").write_text(json.dumps(manifest, indent=2) + "\n")
        raise
    stdout_path.write_text(completed.stdout)
    stderr_path.write_text(completed.stderr)
    manifest.update({"exit_status": completed.returncode, "finished_unix_seconds": time.time()})
    if completed.returncode != 0:
        manifest["database_preserved"] = True
        (artifact_path / "command.json").write_text(json.dumps(manifest, indent=2) + "\n")
        raise RuntimeError(f"{case_name} exited {completed.returncode}; database preserved")
    report = json.loads(report_path.read_text())
    try:
        health = report_validator(report)
    except Exception as error:
        manifest["validation_error"] = str(error)
        manifest["database_preserved"] = True
        (artifact_path / "command.json").write_text(json.dumps(manifest, indent=2) + "\n")
        raise
    manifest["health_sample_count"] = health.get("sample_count", 0)
    manifest["validation"] = "passed"
    (artifact_path / "command.json").write_text(json.dumps(manifest, indent=2) + "\n")
    preserve_database(database_path, artifact_path)
    return manifest


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--service-binary", type=Path, required=True)
    parser.add_argument("--raw-binary", type=Path, required=True)
    parser.add_argument("--result-root", type=Path, required=True)
    args = parser.parse_args()
    if sha256_file(args.service_binary) != SERVICE_BINARY_SHA256:
        raise SystemExit("service workload binary hash mismatch")
    if sha256_file(args.raw_binary) != RAW_BINARY_SHA256:
        raise SystemExit("raw workload binary hash mismatch")
    args.result_root.mkdir(parents=True, exist_ok=False)
    active_processes = []
    process_listing = subprocess.run(["ps", "-eo", "comm=,args="], capture_output=True, text=True, check=True).stdout
    for line in process_listing.splitlines():
        fields = line.strip().split(None, 1)
        if not fields:
            continue
        process_name = fields[0]
        command_name = Path(fields[1].split()[0]).name if len(fields) == 2 else process_name
        if process_name in {"cargo", "rustc", "service_workload", "unconditional_write_workload"} or command_name in {"service_workload", "unconditional_write_workload"}:
            active_processes.append(line)
    if active_processes:
        raise SystemExit("preflight found an active build or workload process")
    available_memory_bytes = None
    for line in Path("/proc/meminfo").read_text().splitlines():
        if line.startswith("MemAvailable:"):
            available_memory_bytes = int(line.split()[1]) * 1_024
            break
    disk_usage = shutil.disk_usage(args.result_root)
    preflight = {
        "source_sha": SOURCE_SHA,
        "hostname": subprocess.run(["hostname"], capture_output=True, text=True, check=True).stdout.strip(),
        "architecture": os.uname().machine,
        "online_cpu_count": os.cpu_count(),
        "mem_available_bytes": available_memory_bytes,
        "filesystem_path": str(args.result_root),
        "filesystem_free_bytes": disk_usage.free,
        "active_build_or_workload_processes": active_processes,
        "service_binary_sha256": sha256_file(args.service_binary),
        "raw_binary_sha256": sha256_file(args.raw_binary),
    }
    (args.result_root / "preflight.json").write_text(json.dumps(preflight, indent=2) + "\n")
    cases = []
    service_base = [
        "--seed", "20261013", "--warmup-ms", "1000", "--measurement-ms", "5000",
        "--clients", "16", "--rows", "10000", "--value-size", "512",
        "--call-cap", "100000000", "--read-stage-metrics", "0",
        "--detailed-write-metrics", "0", "--engine-health-interval-ms", "1000",
        "--mutation-workload", "new-key", "--include-width-16", "1",
        "--latency-storage", "log1024", "--max-write-ledger", "5000000",
        "--require-full-window", "1", "--condition-key-lock-mode", "shared",
    ]
    for case_name, backend, additional in [
        ("blink-rw50-new-key-c16", "blink", []),
        ("rocksdb-service-rw50-new-key-c16-shared16", "rocksdb", ["--revision-allocator", "reserved-ranges", "--blocking-admission", "shared", "--shared-blocking-slots", "16"]),
        ("rocksdb-core-rw50-new-key-c16-shared16", "rocksdb-direct", ["--client-executor", "dedicated", "--revision-allocator", "reserved-ranges", "--shared-blocking-slots", "16"]),
        ("rocksdb-core-mt-mix-c16-shared16", "rocksdb-direct", ["--scenario", "mt-mix", "--client-executor", "dedicated", "--revision-allocator", "reserved-ranges", "--shared-blocking-slots", "16"]),
    ]:
        scenario = "mt-mix" if "mt-mix" in case_name else "rw50"
        arguments = ["--backend", backend, "--scenario", scenario, *service_base, *additional]
        cases.append(run_case(
            args.service_binary,
            args.result_root,
            case_name,
            arguments,
            lambda report, backend=backend, scenario=scenario: validate_service_report(report, backend, scenario),
        ))
    raw_arguments = [
        "--backend", "raw-rocksdb", "--width", "16", "--clients", "64",
        "--seed", "20261013", "--value-size", "512", "--warmup-ms", "1000",
        "--measurement-ms", "5000", "--call-cap", "100000000",
        "--validate-histogram", "0", "--revision-allocator", "reserved-ranges",
        "--condition-key-lock-mode", "shared", "--engine-health-interval-ms", "1000",
        "--shared-blocking-slots", "64",
    ]
    cases.append(run_case(args.raw_binary, args.result_root, "raw-unconditional-width16-c64", raw_arguments, validate_raw_report))
    result = {"source_sha": SOURCE_SHA, "case_count": len(cases), "cases": cases}
    (args.result_root / "completion.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
