import argparse
import hashlib
import json
import os
import pathlib
import signal
import shutil
import subprocess
import threading
import time


SOURCE_SHA = "ee6cd94f1a4e7b2bab7af603bc8983bbf8184542"
SERVICE_BINARY_SHA256 = "7c8b35679df63402b3454067d720c53e1a194f5be6fe459eb7bec35862bdd4b5"
RAW_BINARY_SHA256 = "97b2b38717679bba4000a9eed2bf8c2435114e3d7fa5717e9727381e3586e3f4"
SEED = 20261013
ROWS = 10_000
VALUE_SIZE = 512
WARMUP_MS = 10_000
MEASUREMENT_MS = 10_000
HEALTH_INTERVAL_MS = 1_000
CALL_CAP = 100_000_000
MAX_WRITE_LEDGER = 5_000_000
PER_CASE_RSS_BUDGET_BYTES = 3 << 30
PER_CASE_DATABASE_BUDGET_BYTES = 4 << 30
EVIDENCE_BUDGET_BYTES = 4 << 30
MINIMUM_AVAILABLE_MEMORY_BYTES = (3 << 30) + (1_500 << 20)
MINIMUM_RUNTIME_AVAILABLE_MEMORY_BYTES = 1_500 << 20
MINIMUM_FREE_DISK_BYTES = 10 << 30
HISTOGRAM_SCHEMA_VERSION = 1
HISTOGRAM_SCHEMA = "exact integer nanoseconds below 1024; 1024 equal-width sub-buckets per binary exponent thereafter; percentile is the bucket upper bound"
SCENARIOS = ["rw95", "rw80", "rw50", "rw20", "mt-mix"]
ALL_COHORTS = ["A", "B", "C", "D", "E"]
OVERWRITE_BUDGET_EVIDENCE = {
    "A": {"base_rows": 40_000, "clients": 16, "observed_rows": 96_920, "observed_peak_rss_bytes": 188_492 * 1024, "observed_collector_peak_bytes": 27_365_544, "observed_database_bytes": 317_062_847, "key_domain_per_client": 16_384, "source": "gate/rescreen-r3-results/screen-summary.csv rows 1-35; source 15762ba40"},
    "B": {"base_rows": 10_000, "clients": 16, "observed_rows": 50_441, "observed_peak_rss_bytes": 83_873_792, "observed_collector_peak_bytes": 13_646_160, "observed_database_bytes": 265_605_774, "key_domain_per_client": 16_384, "source": "gate/rescreen-r3-results/screen-summary.csv rw20/c16; source 15762ba40, cross-checked with condition-shared-lock-20261019/oci-evidence-88b43098d/ab-44-88b43098d"},
    "C": {"base_rows": 40_000, "clients": 32, "observed_rows": 100_685, "observed_peak_rss_bytes": 213_364 * 1024, "observed_collector_peak_bytes": 28_692_048, "observed_database_bytes": 360_747_297, "key_domain_per_client": 16_384, "source": "condition-shared-lock-20261019/oci-evidence-88b43098d/ab-44-88b43098d c32/rw20 and gate/rescreen-r3-results/screen-summary.csv c32/mt-mix; Blink RSS from source 15762ba40; direct-core c32 uses the same-caller-count service/engine bounds"},
    "E": {"base_rows": 16_384, "clients": 64, "observed_rows": 16_384, "observed_peak_rss_bytes": 374_212 * 1024, "observed_collector_peak_bytes": 30_932_992, "observed_database_bytes": 191_570_101, "key_domain_per_client": 256, "source": "engine-health/evidence-ee6/engine-health-smokes-ee6/raw-unconditional-width16-c64/report.json; source ee6cd94f"},
}
SERVICE_BINARY = pathlib.Path("/bench/zfs/db/condition-shared-lock-20261019/long-matrix-ee6-abce/frozen-ee6-88b43098d/service_workload")
RAW_BINARY = pathlib.Path("/bench/zfs/db/condition-shared-lock-20261019/long-matrix-ee6-abce/frozen-ee6-88b43098d/unconditional_write_workload")
SOURCE_PATH = pathlib.Path("/bench/zfs/db/condition-shared-lock-20261019/source-88b43098d")


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as file_handle:
        for chunk in iter(lambda: file_handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def service_variants(slots):
    return [
        {"backend": "blink", "allocator": "reserved-ranges", "lock_mode": "shared", "slots": None},
        {"backend": "rocksdb", "allocator": "reserved-ranges", "lock_mode": "shared", "slots": slots},
        {"backend": "rocksdb-direct", "allocator": "reserved-ranges", "lock_mode": "shared", "slots": slots},
    ]


def make_jobs():
    jobs = []
    overwrite_variants = [
        {"backend": "blink", "allocator": "reserved-ranges", "lock_mode": "shared", "slots": None},
        {"backend": "rocksdb", "allocator": "reserved-ranges", "lock_mode": "shared", "slots": 8},
        {"backend": "rocksdb", "allocator": "reserved-ranges", "lock_mode": "shared", "slots": 16},
        {"backend": "rocksdb-direct", "allocator": "reserved-ranges", "lock_mode": "shared", "slots": 16},
    ]
    for repetition in range(1, 4):
        scenario_offset = repetition - 1
        rotated_scenarios = SCENARIOS[scenario_offset:] + SCENARIOS[:scenario_offset]
        for scenario_index, scenario in enumerate(rotated_scenarios):
            rotation = (repetition + scenario_index - 1) % len(overwrite_variants)
            rotated_variants = overwrite_variants[rotation:] + overwrite_variants[:rotation]
            for variant in rotated_variants:
                jobs.append({"cohort": "A", "scenario": scenario, "clients": 16, "mutation_workload": "overwrite", "overwrite_slots": 1024, "width": None, "repetition": repetition, **variant})
    slot_comparison_variants = [
        {"backend": "rocksdb", "allocator": "serialized", "lock_mode": "exclusive", "slots": 8},
        {"backend": "rocksdb", "allocator": "reserved-ranges", "lock_mode": "exclusive", "slots": 8},
        {"backend": "rocksdb", "allocator": "reserved-ranges", "lock_mode": "exclusive", "slots": 16},
    ]
    for repetition in range(1, 4):
        rotation = repetition - 1
        rotated_variants = slot_comparison_variants[rotation:] + slot_comparison_variants[:rotation]
        for variant in rotated_variants:
            jobs.append({"cohort": "B", "scenario": "rw20", "clients": 16, "mutation_workload": "overwrite", "overwrite_slots": 1024, "width": None, "repetition": repetition, **variant})
    clients32_variants = [
        {"backend": "blink", "allocator": "reserved-ranges", "lock_mode": "shared", "slots": None},
        {"backend": "rocksdb", "allocator": "reserved-ranges", "lock_mode": "shared", "slots": 32},
        {"backend": "rocksdb-direct", "allocator": "reserved-ranges", "lock_mode": "shared", "slots": 32},
    ]
    for repetition in range(1, 4):
        rotation = repetition - 1
        rotated_variants = clients32_variants[rotation:] + clients32_variants[:rotation]
        for variant in rotated_variants:
            jobs.append({"cohort": "C", "scenario": "mt-mix", "clients": 32, "mutation_workload": "overwrite", "overwrite_slots": 1024, "width": None, "repetition": repetition, **variant})
    new_key_variants = service_variants(16)
    for repetition in range(1, 4):
        rotation = repetition - 1
        rotated_variants = new_key_variants[rotation:] + new_key_variants[:rotation]
        for variant in rotated_variants:
            jobs.append({"cohort": "D", "scenario": "rw50", "clients": 16, "mutation_workload": "new-key", "overwrite_slots": 1024, "width": None, "repetition": repetition, **variant})
    unconditional_backends = ["raw-rocksdb", "synchronous-core", "rocksdb-service", "blink"]
    for repetition in range(1, 4):
        for width_index, width in enumerate([1, 16]):
            rotation = (repetition + width_index - 1) % len(unconditional_backends)
            rotated_backends = unconditional_backends[rotation:] + unconditional_backends[:rotation]
            for backend in rotated_backends:
                slots = 64 if width == 1 else 32
                jobs.append({"cohort": "E", "scenario": None, "clients": 64, "mutation_workload": "strict-preseed-overwrite", "overwrite_slots": 256, "width": width, "repetition": repetition, "backend": backend, "allocator": "reserved-ranges", "lock_mode": "shared", "slots": slots if backend in {"rocksdb-service", "synchronous-core"} else None})
    if len(jobs) != 111:
        raise ValueError(f"unexpected matrix size: {len(jobs)}")
    for case_number, job in enumerate(jobs, 1):
        job["case_number"] = case_number
        scenario_tag = job["scenario"] if job["scenario"] is not None else f"w{job['width']}"
        job["case_id"] = f"{job['cohort']}-{scenario_tag}-c{job['clients']}-r{job['repetition']}-{job['backend']}-{job['allocator']}-{job['lock_mode']}-s{job['slots'] or 'na'}"
    return jobs


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
        raise ValueError("histogram schema mismatch")
    bins = histogram.get("nonzero_bins", [])
    if bins != sorted(bins, key=lambda bin_entry: bin_entry[0]) or len({entry[0] for entry in bins}) != len(bins):
        raise ValueError("histogram bins are not unique and sorted")
    if sum(entry[1] for entry in bins) != histogram.get("count"):
        raise ValueError("histogram count differs from sparse-bin count")
    count = histogram["count"]
    fields = ["p50_ms", "p95_ms", "p99_ms"] if "p50_ms" in histogram else ["p50_nanos", "p95_nanos", "p99_nanos"]
    for percentile, field_name in zip((50, 95, 99), fields):
        if count == 0:
            if histogram[field_name] is not None:
                raise ValueError("empty histogram has a non-null percentile")
            continue
        if field_name.endswith("_ms"):
            target_count = ((count - 1) * percentile + 99) // 100 + 1
        else:
            target_count = max((count * percentile + 99) // 100, 1)
        cumulative_count = 0
        upper_bound = 0
        for bucket_index, bucket_count in bins:
            cumulative_count += bucket_count
            if cumulative_count >= target_count:
                upper_bound = histogram_upper_bound(bucket_index)
                break
        expected = upper_bound / 1_000_000 if field_name.endswith("_ms") else upper_bound
        if histogram[field_name] != expected:
            raise ValueError(f"{field_name} does not match sparse-bin percentile")


def validate_all_histograms(value):
    if isinstance(value, dict):
        if value.get("storage", "").startswith("log1024"):
            validate_histogram(value)
        for child in value.values():
            validate_all_histograms(child)
    elif isinstance(value, list):
        for child in value:
            validate_all_histograms(child)


def source_tree_state():
    head = subprocess.run(["git", "-C", str(SOURCE_PATH), "rev-parse", "HEAD"], check=True, capture_output=True, text=True).stdout.strip()
    status = subprocess.run(["git", "-C", str(SOURCE_PATH), "status", "--porcelain"], check=True, capture_output=True, text=True).stdout.strip()
    return head, status


def process_inventory():
    active = []
    for command_path in pathlib.Path("/proc").glob("[0-9]*/cmdline"):
        try:
            first_argument = command_path.read_bytes().split(b"\0", 1)[0]
            name = pathlib.Path(os.fsdecode(first_argument)).name
        except (FileNotFoundError, ProcessLookupError):
            continue
        if name in {"cargo", "rustc", "service_workload", "unconditional_write_workload"}:
            active.append({"pid": command_path.parent.name, "name": name})
    return active


def read_host_sample():
    memory = {}
    for line in pathlib.Path("/proc/meminfo").read_text().splitlines():
        name, value, *unit = line.split()
        if name in {"MemTotal:", "MemAvailable:", "Cached:", "SwapFree:"}:
            memory[name.rstrip(":")] = int(value) * (1024 if unit and unit[0] == "kB" else 1)
    return {
        "unix_time_ns": time.time_ns(),
        "memory_bytes": memory,
        "vmstat": pathlib.Path("/proc/vmstat").read_text(),
        "diskstats": pathlib.Path("/proc/diskstats").read_text(),
    }


def sample_host_until_stopped(stop_event, destination):
    with destination.open("a") as output_file:
        while not stop_event.is_set():
            output_file.write(json.dumps(read_host_sample(), separators=(",", ":")) + "\n")
            output_file.flush()
            stop_event.wait(1)


def read_process_group_rss(process_group_id):
    total_rss_bytes = 0
    for stat_path in pathlib.Path("/proc").glob("[0-9]*/stat"):
        try:
            stat_text = stat_path.read_text()
            process_fields = stat_text[stat_text.rfind(")") + 2 :].split()
            if int(process_fields[2]) != process_group_id:
                continue
            for line in stat_path.with_name("status").read_text().splitlines():
                if line.startswith("VmRSS:"):
                    total_rss_bytes += int(line.split()[1]) * 1024
                    break
        except (FileNotFoundError, ProcessLookupError, ValueError, IndexError):
            continue
    return total_rss_bytes


def read_live_process_group_members(process_group_id):
    process_ids = []
    for stat_path in pathlib.Path("/proc").glob("[0-9]*/stat"):
        try:
            stat_text = stat_path.read_text()
            process_fields = stat_text[stat_text.rfind(")") + 2 :].split()
            if int(process_fields[2]) == process_group_id and process_fields[0] != "Z":
                process_ids.append(int(stat_path.parent.name))
        except (FileNotFoundError, ProcessLookupError, ValueError, IndexError):
            continue
    return process_ids


def signal_process_group(process_group_id, signal_number):
    try:
        os.killpg(process_group_id, signal_number)
    except ProcessLookupError:
        return False
    return True


def terminate_process_group(process, grace_seconds=5):
    process_group_id = process.pid
    signal_process_group(process_group_id, signal.SIGTERM)
    grace_deadline = time.monotonic() + grace_seconds
    while time.monotonic() < grace_deadline and read_live_process_group_members(process_group_id):
        time.sleep(0.1)
    if read_live_process_group_members(process_group_id):
        signal_process_group(process_group_id, signal.SIGKILL)
    process.wait()


def read_database_size(database_path):
    total_bytes = 0
    for file_path in database_path.rglob("*"):
        try:
            if file_path.is_file():
                total_bytes += file_path.stat().st_size
        except FileNotFoundError:
            continue
    return total_bytes


def read_live_case_sample(process_group_id, database_path, database_sample_due):
    host_sample = read_host_sample()
    return {
        "unix_time_ns": host_sample["unix_time_ns"],
        "process_group_rss_bytes": read_process_group_rss(process_group_id),
        "memory_bytes": host_sample["memory_bytes"],
        "database_bytes": read_database_size(database_path) if database_sample_due else None,
        "database_sampled": database_sample_due,
    }


def preserve_database(database_path, case_directory):
    files = []
    log_root = case_directory / "database-logs"
    for file_path in sorted(database_path.rglob("*")):
        if file_path.is_file():
            relative = file_path.relative_to(database_path)
            digest = sha256_file(file_path)
            file_record = {"path": str(relative), "size_bytes": file_path.stat().st_size, "sha256": digest}
            if file_path.name == "LOG":
                preserved_log = log_root / relative
                preserved_log.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(file_path, preserved_log)
                if sha256_file(preserved_log) != digest:
                    raise ValueError("preserved LOG hash mismatch")
                file_record["preserved_log_path"] = str(preserved_log.relative_to(case_directory))
            files.append(file_record)
    manifest = {"database_path": str(database_path), "file_count": len(files), "files": files}
    manifest_path = case_directory / "database-manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    if sum(record["size_bytes"] for record in files) > PER_CASE_DATABASE_BUDGET_BYTES:
        raise ValueError("database exceeded the per-case disk budget and was preserved")
    shutil.rmtree(database_path)
    manifest["database_removed_after_validation"] = True
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")


def service_input_fingerprint(report):
    entries = report["counters"]["client_input_prefixes"]
    if len(entries) != report["configuration"]["clients"]:
        raise ValueError("service client input prefix count mismatch")
    if any(entry.get("prefix_limit") != 100 or entry.get("workflow_count") != 100 for entry in entries):
        raise ValueError("service input prefix is incomplete")
    return [list(entry) for entry in sorted((entry["client_index"], entry["fingerprint"]) for entry in entries)]


def validate_health(report, backend, expected_database_count):
    health = report.get("engine_health", {})
    if backend == "blink":
        if health.get("status") != "unsupported":
            raise ValueError("Blink health support status mismatch")
        return health
    if health.get("status") != "sampled" or health.get("sampling_interval_ms") != HEALTH_INTERVAL_MS:
        raise ValueError("engine health sampling mode mismatch")
    minimum_health_samples = max(8, (MEASUREMENT_MS * 9 + 9_999) // 10_000)
    if health.get("coverage_complete") is not True or health.get("truncated") is not False or health.get("sample_count", 0) < minimum_health_samples:
        raise ValueError("engine health coverage is incomplete")
    samples = health.get("samples", [])
    sample_timestamps = [sample.get("timestamp_unix_ms") for sample in samples]
    if health.get("elapsed_ms", 0) < MEASUREMENT_MS - 1_500 or any(not isinstance(timestamp, int) for timestamp in sample_timestamps):
        raise ValueError("engine health timestamp coverage is incomplete")
    if sample_timestamps != sorted(sample_timestamps) or sample_timestamps[-1] - sample_timestamps[0] < MEASUREMENT_MS - 2_500:
        raise ValueError("engine health timestamps do not span the measurement window")
    expected_properties = {
        "estimate-pending-compaction-bytes",
        "num-running-compactions",
        "num-running-flushes",
        "mem-table-flush-pending",
        "compaction-pending",
        "is-write-stopped",
        "actual-delayed-write-rate",
        "block-cache-usage",
        "cur-size-all-mem-tables",
    }
    observed_database_names = None
    for sample in health.get("samples", []):
        databases = sample.get("databases", [])
        if len(databases) != expected_database_count:
            raise ValueError("engine health database count mismatch")
        database_names = [database.get("database") for database in databases]
        if len(set(database_names)) != expected_database_count or (observed_database_names is not None and database_names != observed_database_names):
            raise ValueError("engine health database names are duplicate or unstable")
        observed_database_names = database_names
        for database in databases:
            if database.get("status") != "open" or set(database.get("properties", {})) != expected_properties:
                raise ValueError("engine health property set is incomplete")
            for property_value in database["properties"].values():
                if property_value.get("status") not in {"value", "unsupported", "get_int_property_unavailable", "native_property_error"}:
                    raise ValueError("engine property status is not explicit")
                if property_value.get("status") == "value" and property_value.get("value") is None:
                    raise ValueError("engine property value is missing")
    return health


def verify_report(report, job):
    if job["cohort"] == "E":
        configuration = report.get("configuration", {})
        if report.get("benchmark_kind") != "unconditional_write_probe":
            raise ValueError("unconditional workload kind mismatch")
        if configuration.get("seed") != SEED or configuration.get("clients") != 64 or configuration.get("width") != job["width"]:
            raise ValueError("unconditional input dimensions mismatch")
        if configuration.get("value_size_bytes") != VALUE_SIZE or configuration.get("overwrite_slots_per_client") != 256:
            raise ValueError("unconditional slot or value size mismatch")
        if configuration.get("engine_health_interval_ms") != HEALTH_INTERVAL_MS:
            raise ValueError("unconditional health interval mismatch")
        if configuration.get("executor") != "one dedicated caller thread per client" or configuration.get("caller_threads") != 64:
            raise ValueError("unconditional caller executor mismatch")
        if configuration.get("wal_sync") is not True:
            raise ValueError("unconditional synchronous WAL configuration mismatch")
        if configuration.get("condition_key_lock_mode_requested") != job["lock_mode"]:
            raise ValueError("unconditional requested condition lock mode mismatch")
        if report["timing"].get("measurement_complete") is not True or report["timing"].get("requested_measurement_ms") != MEASUREMENT_MS:
            raise ValueError("unconditional full measurement window mismatch")
        if report["counters"].get("other_errors") != 0 or report["counters"].get("overloads") != 0:
            raise ValueError("unconditional errors were recorded")
        if report.get("final_validation", {}).get("passed") is not True or report.get("reopen_validation", {}).get("passed") is not True:
            raise ValueError("unconditional final or reopen oracle failed")
        per_client = report.get("per_client", [])
        if len(per_client) != 64 or any(entry.get("measurement_input_prefix_batches") != 100 for entry in per_client):
            raise ValueError("unconditional input prefix evidence is incomplete")
        expected_backend = {
            "raw-rocksdb": "RawRocksDb",
            "synchronous-core": "SynchronousCore",
            "rocksdb-service": "RocksDbService",
            "blink": "Blink",
        }[job["backend"]]
        if report.get("backend") != expected_backend:
            raise ValueError("unconditional backend mismatch")
        if job["backend"] in {"raw-rocksdb", "blink"}:
            expected_lock_mode = "not-applicable-no-conditional-transactions" if job["backend"] == "raw-rocksdb" else "not-applicable"
            if configuration.get("condition_key_lock_mode") != expected_lock_mode:
                raise ValueError("unconditional mode applicability mismatch")
        elif configuration.get("condition_key_lock_mode") != job["lock_mode"]:
            raise ValueError("unconditional condition lock mode mismatch")
        if job["backend"] == "rocksdb-service" and configuration.get("shared_blocking_slots") != job["slots"]:
            raise ValueError("unconditional service slots mismatch")
        if job["backend"] == "synchronous-core" and configuration.get("requested_shared_blocking_slots") != job["slots"]:
            raise ValueError("unconditional direct-core caller count mismatch")
        health_backend = "raw-rocksdb" if job["backend"] == "raw-rocksdb" else job["backend"]
        validate_health(report, health_backend, 1)
        validate_all_histograms(report)
        latency = report.get("latency", {})
        acknowledged_batches = report.get("counters", {}).get("measurement_write_batches_acknowledged")
        if latency.get("storage") != "log1024" or latency.get("count") != acknowledged_batches or latency.get("sample_count") != acknowledged_batches:
            raise ValueError("unconditional latency count differs from acknowledged batches")
        if report.get("timing", {}).get("actual_write_batches") != acknowledged_batches:
            raise ValueError("unconditional timing count differs from acknowledged batches")
        if report.get("counters", {}).get("acknowledged_keys") != acknowledged_batches * job["width"]:
            raise ValueError("unconditional acknowledged key count differs from batch width")
        return [list(entry) for entry in sorted((entry["client_index"], entry["measurement_input_prefix_fingerprint_fnv1a64"]) for entry in per_client)]
    configuration = report.get("configuration", {})
    rocksdb_config = configuration.get("rocksdb", {})
    expected_backend = {"blink": "blink", "rocksdb": "rocksdb", "rocksdb-direct": "rocksdb-direct"}[job["backend"]]
    if report.get("backend") != expected_backend or report.get("scenario") != job["scenario"]:
        raise ValueError("service backend or scenario mismatch")
    expected_api_boundary = "synchronous TenantDatabase core" if job["backend"] == "rocksdb-direct" else "DodbService::execute"
    if expected_api_boundary not in report.get("api_boundary", ""):
        raise ValueError("service API boundary mismatch")
    if report.get("seed") != SEED or configuration.get("clients") != job["clients"]:
        raise ValueError("service seed or client count mismatch")
    if configuration.get("initial_rows_per_tenant") != ROWS or configuration.get("value_size_bytes") != VALUE_SIZE:
        raise ValueError("service seed dimensions mismatch")
    if configuration.get("mutation_workload") != job["mutation_workload"] or configuration.get("overwrite_slots_per_client") != job["overwrite_slots"]:
        raise ValueError("service mutation workload mismatch")
    if configuration.get("include_width_16") is not True or configuration.get("latency_storage") != "log1024":
        raise ValueError("service width or histogram configuration mismatch")
    if configuration.get("client_executor") != "dedicated":
        raise ValueError("service client executor mismatch")
    if configuration.get("warmup_requested_ms") != WARMUP_MS or configuration.get("measurement_requested_ms") != MEASUREMENT_MS:
        raise ValueError("service timing configuration mismatch")
    if rocksdb_config.get("condition_key_lock_mode") != ("not-applicable" if job["backend"] == "blink" else job["lock_mode"]):
        raise ValueError("service condition-key lock mode mismatch")
    if rocksdb_config.get("revision_allocator") != job["allocator"]:
        raise ValueError("service revision allocator mismatch")
    if job["slots"] is not None and rocksdb_config.get("blocking_admission") != f"shared-{job['slots']}":
        raise ValueError("service blocking admission mismatch")
    if job["backend"] == "rocksdb-direct" and rocksdb_config.get("blocking_operation_limit") != job["slots"]:
        raise ValueError("direct-core caller limit mismatch")
    if configuration.get("research_metrics_enabled") is not False or configuration.get("read_stage_metrics_enabled") is not False or configuration.get("detailed_write_metrics_enabled") is not False:
        raise ValueError("detailed diagnostic metrics were enabled")
    timing = report.get("timing", {})
    if timing.get("measurement_complete") is not True or timing.get("accepted_as_full_window") is not True or timing.get("termination_reason") != "requested-duration":
        raise ValueError("service full measurement window failed")
    counters = report.get("counters", {})
    for counter_name in ["other_errors", "overloads"]:
        if counters.get(counter_name, 0) != 0:
            raise ValueError(f"service unexpected {counter_name}")
    if counters.get("verification_errors") or counters.get("request_limit_reached") is not False:
        raise ValueError("service verification or cap gate failed")
    if counters.get("successful_mutations", 0) == 0 or counters.get("intentional_condition_rejections", 0) == 0:
        raise ValueError("service success or intentional rejection path was not exercised")
    if sum(category.get("attempts", 0) for name, category in counters.get("write_categories", {}).items() if name.endswith("width_16")) == 0:
        raise ValueError("service width 16 path was not attempted")
    if not report.get("validation") or not report.get("reopen_validation") or any(not item.get("passed") for item in report["validation"].values()) or any(not item.get("passed") for item in report["reopen_validation"].values()):
        raise ValueError("service final or reopen oracle failed")
    expected_tenants = 4 if job["scenario"] == "mt-mix" else 1
    health = validate_health(report, job["backend"], expected_tenants)
    if job["backend"] == "rocksdb-direct" and job["scenario"] == "mt-mix":
        names = [database["database"] for database in health["samples"][0]["databases"]]
        if names != ["tenant_1", "tenant_2", "tenant_3", "tenant_4"]:
            raise ValueError("direct core health tenant labels mismatch")
    validate_all_histograms(report)
    latency_counts = {
        "get": counters.get("ordinary_gets", 0),
        "preparatory_get": counters.get("preparatory_gets", 0),
        "query16": counters.get("query16_calls", 0),
        "condition_rejection": counters.get("intentional_condition_rejections", 0),
        "write_transaction_success": counters.get("successful_write_transactions", 0),
        "write_error": counters.get("write_requests", 0) - counters.get("successful_write_transactions", 0) - counters.get("intentional_condition_rejections", 0),
        "workflow": counters.get("api_calls", 0) - counters.get("preparatory_gets", 0),
    }
    latency = counters.get("latency", {})
    expected_latency_kinds = {name for name, count in latency_counts.items() if count > 0}
    if set(latency) != expected_latency_kinds:
        raise ValueError("service latency kinds differ from workload counter kinds")
    for latency_name, expected_count in latency_counts.items():
        if expected_count and (latency[latency_name].get("storage") != "log1024" or latency[latency_name].get("sample_count") != expected_count or latency[latency_name].get("count") != expected_count):
            raise ValueError(f"service latency count mismatch for {latency_name}")
    return service_input_fingerprint(report)


def make_command(binary_paths, job, database_path, report_path):
    if job["cohort"] == "E":
        backend_binary = binary_paths["raw"]
        backend = job["backend"]
        command = [
            str(backend_binary), "--backend", backend, "--width", str(job["width"]),
            "--clients", "64", "--seed", str(SEED), "--value-size", str(VALUE_SIZE),
            "--warmup-ms", str(WARMUP_MS), "--measurement-ms", str(MEASUREMENT_MS),
            "--call-cap", str(CALL_CAP), "--validate-histogram", "0",
            "--revision-allocator", job["allocator"], "--condition-key-lock-mode", job["lock_mode"],
            "--engine-health-interval-ms", str(HEALTH_INTERVAL_MS),
            "--data-dir", str(database_path), "--report", str(report_path),
        ]
        if job["backend"] in {"rocksdb-service", "synchronous-core"}:
            command.extend(["--shared-blocking-slots", str(job["slots"])])
        return command
    command = [
        str(binary_paths["service"]), "--backend", job["backend"], "--scenario", job["scenario"],
        "--data-dir", str(database_path), "--report", str(report_path),
        "--seed", str(SEED), "--rows", str(ROWS), "--value-size", str(VALUE_SIZE),
        "--clients", str(job["clients"]), "--warmup-ms", str(WARMUP_MS),
        "--measurement-ms", str(MEASUREMENT_MS), "--call-cap", str(CALL_CAP),
        "--max-write-ledger", str(MAX_WRITE_LEDGER), "--require-full-window", "1",
        "--mutation-workload", job["mutation_workload"],
        "--overwrite-slots-per-client", str(job["overwrite_slots"]),
        "--latency-storage", "log1024", "--include-width-16", "1",
        "--revision-allocator", job["allocator"], "--condition-key-lock-mode", job["lock_mode"],
        "--engine-health-interval-ms", str(HEALTH_INTERVAL_MS),
        "--research-metrics", "0", "--read-stage-metrics", "0", "--detailed-write-metrics", "0",
    ]
    if job["backend"] in {"rocksdb", "rocksdb-direct"}:
        command.extend(["--shared-blocking-slots", str(job["slots"])])
    command.extend(["--client-executor", "dedicated"])
    return command


def projected_budgets(smoke_root):
    blink_report = json.loads((smoke_root / "blink-rw50-new-key-c16" / "report.json").read_text())
    raw_report = json.loads((smoke_root / "raw-unconditional-width16-c64" / "report.json").read_text())
    blink_rss = blink_report["process_metrics"]["peak_sampled_rss_kb"] * 1024
    blink_rss_samples = blink_report["process_metrics"]["rss_sample_count"]
    blink_mutations = blink_report["counters"]["successful_mutations"]
    blink_deletes = blink_report["counters"]["write_categories"]["delete"]["successes"]
    blink_new_key_mutations = blink_mutations - blink_deletes
    blink_database_bytes = blink_report["disk_usage_bytes"]["after_reopen_validation"]
    raw_rss = raw_report["process_metrics"]["peak_sampled_rss_kb"] * 1024
    raw_rss_samples = raw_report["process_metrics"]["rss_sample_count"]
    raw_database_bytes = raw_report["disk_usage_bytes"]
    smoke_total_run_ms = 6_000
    projected_total_run_ms = WARMUP_MS + MEASUREMENT_MS
    projection_multiplier = (projected_total_run_ms + 4_999) // 5_000
    new_key_smoke_reports = {
        "blink": blink_report,
        "rocksdb-service": json.loads((smoke_root / "rocksdb-service-rw50-new-key-c16-shared16" / "report.json").read_text()),
        "synchronous-core": json.loads((smoke_root / "rocksdb-core-rw50-new-key-c16-shared16" / "report.json").read_text()),
    }
    new_key_backend_projections = {}
    for backend_name, report in new_key_smoke_reports.items():
        counters = report["counters"]
        non_delete_mutations = counters["successful_mutations"] - counters["write_categories"]["delete"]["successes"]
        projected_insertions = (non_delete_mutations * projected_total_run_ms + 4_999) // 5_000
        peak_sampled_rss_bytes = report["process_metrics"]["peak_sampled_rss_kb"] * 1024
        collector_peak_bytes = report.get("collector_memory_phases", {}).get("estimated_simultaneous_collector_peak_bytes", 0)
        validation_state_bytes = (ROWS + projected_insertions) * 4_096
        sampled_rss_bytes = peak_sampled_rss_bytes * projection_multiplier
        collector_bytes = collector_peak_bytes * projection_multiplier
        projected_peak_bytes = sampled_rss_bytes + validation_state_bytes + collector_bytes + (512 << 20)
        if backend_name == "blink":
            smoke_database_bytes = report["disk_usage_bytes"]["after_reopen_validation"]
        else:
            smoke_database_bytes = report["disk_usage_bytes"]["after_reopen_validation"]
        projected_database_bytes = smoke_database_bytes * projection_multiplier
        new_key_backend_projections[backend_name] = {
            "smoke_measurement_ms": 5_000,
            "smoke_warmup_ms": report["configuration"]["warmup_requested_ms"],
            "projected_total_run_ms_including_warmup_and_measurement": projected_total_run_ms,
            "smoke_non_delete_mutations": non_delete_mutations,
            "projected_new_key_insertions": projected_insertions,
            "smoke_peak_sampled_rss_bytes": peak_sampled_rss_bytes,
            "whole_process_rss_projection_bytes": sampled_rss_bytes,
            "validation_expected_state_budget_bytes_at_4096_bytes_per_row": validation_state_bytes,
            "smoke_simultaneous_collector_peak_bytes": collector_peak_bytes,
            "collector_and_ledger_budget_projection_bytes": collector_bytes,
            "validation_runtime_reserve_bytes": 512 << 20,
            "projected_peak_bytes": projected_peak_bytes,
            "projected_peak_gib": projected_peak_bytes / (1 << 30),
            "per_case_rss_cap_bytes": PER_CASE_RSS_BUDGET_BYTES,
            "smoke_database_bytes": smoke_database_bytes,
            "projected_database_bytes": projected_database_bytes,
            "per_case_database_cap_bytes": PER_CASE_DATABASE_BUDGET_BYTES,
        }
    overwrite_cohort_projections = {}
    for cohort, evidence in OVERWRITE_BUDGET_EVIDENCE.items():
        if cohort == "E":
            projected_rows = evidence["observed_rows"]
            collector_multiplier = 1
            database_multiplier = 1
        else:
            observed_growth = max(0, evidence["observed_rows"] - evidence["base_rows"])
            projected_growth = observed_growth * projection_multiplier
            maximum_distinct_rows = evidence["base_rows"] + evidence["clients"] * evidence["key_domain_per_client"]
            projected_rows = min(evidence["base_rows"] + projected_growth, maximum_distinct_rows)
            collector_multiplier = projection_multiplier
            database_multiplier = projection_multiplier
        validation_state_bytes = projected_rows * 4_096
        collector_and_ledger_bytes = evidence["observed_collector_peak_bytes"] * collector_multiplier
        projected_peak_bytes = evidence["observed_peak_rss_bytes"] + validation_state_bytes + collector_and_ledger_bytes + (512 << 20)
        projected_database_bytes = evidence["observed_database_bytes"] * database_multiplier
        overwrite_cohort_projections[cohort] = {
            "evidence": evidence["source"],
            "projected_rows": projected_rows,
            "validation_expected_state_budget_bytes_at_4096_bytes_per_row": validation_state_bytes,
            "observed_peak_rss_bytes_not_scaled": evidence["observed_peak_rss_bytes"],
            "collector_and_ledger_projection_bytes": collector_and_ledger_bytes,
            "validation_runtime_reserve_bytes": 512 << 20,
            "projected_peak_bytes": projected_peak_bytes,
            "projected_peak_gib": projected_peak_bytes / (1 << 30),
            "observed_database_bytes": evidence["observed_database_bytes"],
            "projected_database_bytes": projected_database_bytes,
            "per_case_rss_cap_bytes": PER_CASE_RSS_BUDGET_BYTES,
            "per_case_database_cap_bytes": PER_CASE_DATABASE_BUDGET_BYTES,
        }
    projected_blink_peak_bytes = new_key_backend_projections["blink"]["projected_peak_bytes"]
    projected_blink_database_bytes = blink_database_bytes * projection_multiplier
    return {
        "smoke_measurement_ms": 5_000,
        "smoke_total_run_ms": smoke_total_run_ms,
        "projected_total_run_ms_including_warmup_and_measurement": projected_total_run_ms,
        "blink_new_key_smoke_peak_sampled_rss_bytes": blink_rss,
        "blink_new_key_smoke_rss_sample_count": blink_rss_samples,
        "blink_new_key_successful_mutations": blink_new_key_mutations,
        "blink_new_key_estimated_total_run_insertions": new_key_backend_projections["blink"]["projected_new_key_insertions"],
        "blink_expected_state_budget_bytes_at_4096_bytes_per_row": new_key_backend_projections["blink"]["validation_expected_state_budget_bytes_at_4096_bytes_per_row"],
        "blink_sampled_rss_linear_projection_bytes": new_key_backend_projections["blink"]["whole_process_rss_projection_bytes"],
        "blink_collector_and_ledger_projection_bytes": new_key_backend_projections["blink"]["collector_and_ledger_budget_projection_bytes"],
        "blink_validation_runtime_reserve_bytes": 512 << 20,
        "blink_projected_peak_bytes": projected_blink_peak_bytes,
        "blink_projected_database_bytes": projected_blink_database_bytes,
        "blink_per_case_rss_budget_bytes": PER_CASE_RSS_BUDGET_BYTES,
        "blink_per_case_database_budget_bytes": PER_CASE_DATABASE_BUDGET_BYTES,
        "new_key_backend_projections": new_key_backend_projections,
        "overwrite_cohort_projections": overwrite_cohort_projections,
        "raw_width16_c64_smoke_peak_sampled_rss_bytes": raw_rss,
        "raw_width16_c64_smoke_rss_sample_count": raw_rss_samples,
        "raw_width16_c64_smoke_database_bytes": raw_database_bytes,
        "evidence_budget_bytes": EVIDENCE_BUDGET_BYTES,
        "minimum_available_memory_bytes": MINIMUM_AVAILABLE_MEMORY_BYTES,
        "minimum_free_disk_bytes": MINIMUM_FREE_DISK_BYTES,
    }


def make_plan(smoke_root, selected_cohorts):
    jobs = make_jobs()
    budgets = projected_budgets(smoke_root)
    if budgets["blink_projected_database_bytes"] > PER_CASE_DATABASE_BUDGET_BYTES:
        raise ValueError("B-link new-key projection exceeds the per-case database budget")
    command_binaries = {"service": SERVICE_BINARY, "raw": RAW_BINARY}
    for job in jobs:
        expected_binary_name = "raw" if job["cohort"] == "E" else "service"
        command = make_command(command_binaries, job, pathlib.Path("/plan/database"), pathlib.Path("/plan/report.json"))
        if pathlib.Path(command[0]) != command_binaries[expected_binary_name]:
            raise ValueError(f"{job['case_id']} selected the wrong workload binary")
        if job["cohort"] != "E" and command[command.index("--client-executor") + 1] != "dedicated":
            raise ValueError(f"{job['case_id']} did not request a dedicated client executor")
        if "--engine-health-interval-ms" not in command or command[command.index("--engine-health-interval-ms") + 1] != str(HEALTH_INTERVAL_MS):
            raise ValueError(f"{job['case_id']} health interval differs from the matrix")
        job["binary_name"] = expected_binary_name
        job["executor"] = "one dedicated caller thread per client"
        job["durability"] = "not-applicable-no-RocksDB-WAL" if job["backend"] == "blink" else "syncWAL"
    budgets["new_key_projection_within_case_cap"] = all(
        estimate["projected_peak_bytes"] <= PER_CASE_RSS_BUDGET_BYTES
        for estimate in budgets["new_key_backend_projections"].values()
    )
    budgets["database_projection_within_case_cap"] = all(
        estimate["projected_database_bytes"] <= PER_CASE_DATABASE_BUDGET_BYTES
        for estimate in budgets["new_key_backend_projections"].values()
    )
    selected_jobs = [job for job in jobs if job["cohort"] in selected_cohorts]
    if selected_cohorts == ["D"]:
        selected_jobs = [job for job in selected_jobs if job["scenario"] == "rw50" and job["clients"] == 16 and job["repetition"] == 1]
        capacity_backends = {"blink", "rocksdb-service", "synchronous-core"}
        selected_budgets_fit = all(
            budgets["new_key_backend_projections"][backend]["projected_peak_bytes"] <= PER_CASE_RSS_BUDGET_BYTES
            and budgets["new_key_backend_projections"][backend]["projected_database_bytes"] <= PER_CASE_DATABASE_BUDGET_BYTES
            for backend in capacity_backends
        )
    else:
        selected_budgets_fit = all(
        budgets["overwrite_cohort_projections"][cohort]["projected_peak_bytes"] <= PER_CASE_RSS_BUDGET_BYTES
        and budgets["overwrite_cohort_projections"][cohort]["projected_database_bytes"] <= PER_CASE_DATABASE_BUDGET_BYTES
        for cohort in selected_cohorts
        if cohort in budgets["overwrite_cohort_projections"]
        ) and ("D" not in selected_cohorts)
    budgets["selected_cohorts_within_caps"] = selected_budgets_fit
    selected_case_ids = [job["case_id"] for job in selected_jobs]
    fingerprint_material = {
        "source_sha": SOURCE_SHA,
        "service_binary_sha256": SERVICE_BINARY_SHA256,
        "raw_binary_sha256": RAW_BINARY_SHA256,
        "parameters": {
            "seed": SEED,
            "rows": ROWS,
            "value_size": VALUE_SIZE,
            "warmup_ms": WARMUP_MS,
            "measurement_ms": MEASUREMENT_MS,
            "health_interval_ms": HEALTH_INTERVAL_MS,
            "call_cap": CALL_CAP,
            "max_write_ledger": MAX_WRITE_LEDGER,
            "per_case_rss_cap_bytes": PER_CASE_RSS_BUDGET_BYTES,
            "per_case_database_cap_bytes": PER_CASE_DATABASE_BUDGET_BYTES,
        },
        "selected_cohorts": selected_cohorts,
        "selected_case_ids": selected_case_ids,
        "all_case_definitions": jobs,
    }
    plan_fingerprint = hashlib.sha256(json.dumps(fingerprint_material, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
    if selected_cohorts == ["D"]:
        execution_budget_status = "capacity-only D rw50/c16 cases use 10s warmup plus 10s measurement; this does not complete the 30s D cohort"
    elif "D" in selected_cohorts:
        execution_budget_status = "blocked: standard D 30s new-key repetitions are outside this capacity-only plan"
    elif not selected_budgets_fit:
        execution_budget_status = "blocked: selected overwrite cohort projection exceeds a per-case cap"
    elif selected_cohorts == ALL_COHORTS:
        execution_budget_status = "blocked: complete matrix includes cohort D, which exceeds the projected RSS cap"
    else:
        execution_budget_status = "selected cohorts fit projected caps; cohort D remains unselected and pending"
    plan = {
        "experiment_kind": "new-key-capacity-only",
        "measurement_scope": "D rw50/c16, one case per Blink, RocksDB service Shared-16, and synchronous core Shared-16; 10s warmup plus 10s measurement; does not complete standard D 30s repetitions",
        "source_sha": SOURCE_SHA,
        "service_binary_sha256": SERVICE_BINARY_SHA256,
        "raw_binary_sha256": RAW_BINARY_SHA256,
        "plan_fingerprint": plan_fingerprint,
        "case_count": len(jobs),
        "selected_cohorts": selected_cohorts,
        "selected_case_count": len(selected_jobs),
        "selected_case_ids": selected_case_ids,
        "pending_cohorts": ALL_COHORTS if selected_cohorts == ["D"] else [cohort for cohort in ALL_COHORTS if cohort not in selected_cohorts],
        "expected_wall_time_seconds": len(jobs) * (WARMUP_MS + MEASUREMENT_MS) / 1_000,
        "selected_expected_wall_time_seconds": len(selected_jobs) * (WARMUP_MS + MEASUREMENT_MS) / 1_000,
        "resume_policy": "completed validated cases are journaled and never rerun; an interrupted in-progress case preserves its database and requires inspection before resume",
        "execution_policy": "all cases run serially; this plan-only controller does not start measurements without --execute",
        "execution_budget_status": execution_budget_status,
        "preflight_budgets": budgets,
        "jobs": jobs,
        "selected_jobs": selected_jobs,
    }
    return plan


def preflight(binary_paths, result_root):
    head, status = source_tree_state()
    if head != SOURCE_SHA or status:
        raise RuntimeError(f"source checkout mismatch or dirty source tree: {head}, {status!r}")
    expected_hashes = {"service": SERVICE_BINARY_SHA256, "raw": RAW_BINARY_SHA256}
    for name, path in binary_paths.items():
        if not path.is_file() or sha256_file(path) != expected_hashes[name]:
            raise RuntimeError(f"{name} binary hash mismatch")
    if process_inventory():
        raise RuntimeError("preflight found an active cargo, rustc, or workload process")
    if not pathlib.Path("/usr/bin/time").is_file():
        raise RuntimeError("/usr/bin/time is unavailable")
    memory = read_host_sample()["memory_bytes"]
    if memory.get("MemAvailable", 0) < MINIMUM_AVAILABLE_MEMORY_BYTES:
        raise RuntimeError("MemAvailable is below the approved per-case budget plus host reserve")
    disk_usage = shutil.disk_usage(result_root.parent)
    if disk_usage.free < MINIMUM_FREE_DISK_BYTES:
        raise RuntimeError("free space is below the ZFS measurement reserve")
    return {
        "source_sha": head,
        "source_status": status,
        "hostname": subprocess.run(["hostname"], check=True, capture_output=True, text=True).stdout.strip(),
        "architecture": os.uname().machine,
        "online_cpu_count": os.cpu_count(),
        "memory_bytes": memory,
        "filesystem_path": str(result_root.parent),
        "filesystem_free_bytes": disk_usage.free,
        "active_build_or_workload_processes": [],
        "time_binary": "/usr/bin/time",
        "binary_sha256": {name: sha256_file(path) for name, path in binary_paths.items()},
        "checked_unix_time_ns": time.time_ns(),
    }


def validation_group(job):
    if job["cohort"] == "E":
        return f"E-width{job['width']}"
    return f"service-{job['scenario']}-c{job['clients']}-{job['mutation_workload']}"


def run_case(job, result_root, binary_paths, journal):
    case_id = job["case_id"]
    host_state = read_host_sample()
    available_memory = host_state["memory_bytes"].get("MemAvailable", 0)
    free_disk = shutil.disk_usage(result_root).free
    if available_memory < MINIMUM_AVAILABLE_MEMORY_BYTES:
        raise RuntimeError("per-case MemAvailable preflight is below the required reserve")
    if free_disk < MINIMUM_FREE_DISK_BYTES:
        raise RuntimeError("per-case free-space preflight is below the ZFS reserve")
    case_directory = result_root / "cases" / case_id
    database_path = result_root / "databases" / case_id
    report_path = case_directory / "report.json"
    case_directory.mkdir(parents=True, exist_ok=False)
    database_path.mkdir(parents=True, exist_ok=False)
    command = make_command(binary_paths, job, database_path, report_path)
    case_manifest = {
        "case": job,
        "source_sha": SOURCE_SHA,
        "binary_sha256": sha256_file(binary_paths["raw" if job["cohort"] == "E" else "service"]),
        "binary_name": "raw" if job["cohort"] == "E" else "service",
        "executor": "one dedicated caller thread per client",
        "durability": "not-applicable-no-RocksDB-WAL" if job["backend"] == "blink" else "syncWAL",
        "command": command,
        "started_unix_time_ns": time.time_ns(),
        "database_path": str(database_path),
        "database_preserved_until_validation": True,
        "preflight": {"mem_available_bytes": available_memory, "free_disk_bytes": free_disk, "active_build_or_workload_processes": process_inventory()},
    }
    manifest_path = case_directory / "command.json"
    manifest_path.write_text(json.dumps(case_manifest, indent=2) + "\n")
    before_head, before_status = source_tree_state()
    if before_head != SOURCE_SHA or before_status:
        raise RuntimeError("source changed before case start")
    binary_name = "raw" if job["cohort"] == "E" else "service"
    if sha256_file(binary_paths[binary_name]) != case_manifest["binary_sha256"]:
        raise RuntimeError("workload binary changed before case start")
    if process_inventory():
        raise RuntimeError("another build or workload appeared before case start")
    host_sample_path = case_directory / "host-samples.jsonl"
    stop_event = threading.Event()
    host_sampler = threading.Thread(target=sample_host_until_stopped, args=(stop_event, host_sample_path), daemon=False)
    host_sampler.start()
    environment = os.environ.copy()
    environment["LC_ALL"] = "C"
    environment["DODB_SOURCE_SHA"] = SOURCE_SHA
    environment["DODB_BINARY_SHA256"] = case_manifest["binary_sha256"]
    environment["RUST_TEST_THREADS"] = "1"
    timed_command = ["/usr/bin/time", "-v", "-o", str(case_directory / "time-v.txt"), "env", *command]
    case_manifest["time_command"] = timed_command
    manifest_path.write_text(json.dumps(case_manifest, indent=2) + "\n")
    journal[case_id] = {"status": "running", "started_unix_time_ns": case_manifest["started_unix_time_ns"]}
    write_journal(result_root, journal)
    stdout_path = case_directory / "stdout.txt"
    stderr_path = case_directory / "stderr.txt"
    try:
        with stdout_path.open("w") as stdout_file, stderr_path.open("w") as stderr_file:
            process = subprocess.Popen(timed_command, stdout=stdout_file, stderr=stderr_file, env=environment, start_new_session=True)
            started_monotonic = time.monotonic()
            next_database_sample = started_monotonic
            live_samples = []
            failure_reason = None
            while process.poll() is None:
                time.sleep(1)
                if process.poll() is not None:
                    break
                current_monotonic = time.monotonic()
                database_sample_due = current_monotonic >= next_database_sample
                sample = read_live_case_sample(process.pid, database_path, database_sample_due)
                if database_sample_due:
                    next_database_sample = current_monotonic + 5
                live_samples.append(sample)
                with (case_directory / "live-budget-samples.jsonl").open("a") as live_output:
                    live_output.write(json.dumps(sample, separators=(",", ":")) + "\n")
                    live_output.flush()
                if sample["process_group_rss_bytes"] > PER_CASE_RSS_BUDGET_BYTES:
                    failure_reason = "process-group RSS exceeded per-case cap"
                elif sample["memory_bytes"].get("MemAvailable", 0) < MINIMUM_RUNTIME_AVAILABLE_MEMORY_BYTES:
                    failure_reason = "host MemAvailable fell below the runtime reserve"
                elif sample["database_bytes"] is not None and sample["database_bytes"] > PER_CASE_DATABASE_BUDGET_BYTES:
                    failure_reason = "database exceeded per-case disk cap"
                elif time.monotonic() - started_monotonic > 180:
                    failure_reason = "case exceeded 180-second timeout"
                if failure_reason:
                    terminate_process_group(process)
                    break
            completed_returncode = process.wait()
    except Exception as error:
        if "process" in locals() and read_live_process_group_members(process.pid):
            terminate_process_group(process)
        case_manifest.update({"controller_error": str(error), "database_preserved": True, "finished_unix_time_ns": time.time_ns()})
        manifest_path.write_text(json.dumps(case_manifest, indent=2) + "\n")
        journal[case_id] = {"status": "controller_error", "database_preserved": True, "error": str(error)}
        write_journal(result_root, journal)
        stop_event.set()
        host_sampler.join()
        raise
    finally:
        stop_event.set()
        host_sampler.join()
    if failure_reason:
        case_manifest.update({"watchdog_failure": failure_reason, "exit_status": completed_returncode, "database_preserved": True, "finished_unix_time_ns": time.time_ns(), "live_sample_count": len(live_samples)})
        manifest_path.write_text(json.dumps(case_manifest, indent=2) + "\n")
        journal[case_id] = {"status": "watchdog_failure", "reason": failure_reason, "database_preserved": True}
        write_journal(result_root, journal)
        raise RuntimeError(f"{case_id} stopped by watchdog: {failure_reason}; process group terminated and database preserved")
    case_manifest.update({"exit_status": completed_returncode, "finished_unix_time_ns": time.time_ns(), "live_sample_count": len(live_samples)})
    manifest_path.write_text(json.dumps(case_manifest, indent=2) + "\n")
    if completed_returncode != 0:
        journal[case_id] = {"status": "failed", "exit_status": completed_returncode, "database_preserved": True}
        write_journal(result_root, journal)
        raise RuntimeError(f"{case_id} exited {completed_returncode}; database preserved")
    with report_path.open() as report_file:
        report = json.load(report_file)
    input_fingerprint = verify_report(report, job)
    after_head, after_status = source_tree_state()
    if after_head != SOURCE_SHA or after_status:
        raise RuntimeError("source changed while the case was running")
    binary_name = "raw" if job["cohort"] == "E" else "service"
    if sha256_file(binary_paths[binary_name]) != case_manifest["binary_sha256"]:
        raise RuntimeError("workload binary changed while the case was running")
    group = validation_group(job)
    existing_fingerprint = journal.setdefault("input_fingerprints", {}).get(group)
    if existing_fingerprint is not None and existing_fingerprint != input_fingerprint:
        raise ValueError(f"input fingerprint differs from earlier cases in {group}")
    journal["input_fingerprints"][group] = input_fingerprint
    time_text = (case_directory / "time-v.txt").read_text(errors="replace")
    maximum_rss_line = next((line for line in time_text.splitlines() if "Maximum resident set size" in line), None)
    if maximum_rss_line is None:
        raise ValueError("whole-process maximum RSS is missing")
    maximum_rss_kb = int(maximum_rss_line.split(":", 1)[1].strip())
    if maximum_rss_kb * 1024 > PER_CASE_RSS_BUDGET_BYTES:
        raise ValueError("whole-process maximum RSS exceeded the per-case memory budget; database preserved")
    preserve_database(database_path, case_directory)
    case_manifest.update({"validated": True, "maximum_rss_kb": maximum_rss_kb, "database_removed_after_manifest": True})
    manifest_path.write_text(json.dumps(case_manifest, indent=2) + "\n")
    journal[case_id] = {"status": "complete", "maximum_rss_kb": maximum_rss_kb, "input_fingerprint_group": group}
    write_journal(result_root, journal)
    evidence_bytes = sum(path.stat().st_size for path in result_root.rglob("*") if path.is_file())
    if evidence_bytes > EVIDENCE_BUDGET_BYTES:
        raise RuntimeError("evidence artifacts exceeded the aggregate disk budget")
    return case_manifest


def recover_completed_case(job, result_root, binary_paths, journal):
    case_id = job["case_id"]
    case_directory = result_root / "cases" / case_id
    manifest_path = case_directory / "command.json"
    report_path = case_directory / "report.json"
    time_path = case_directory / "time-v.txt"
    if not all(path.is_file() for path in [manifest_path, report_path, time_path]):
        raise SystemExit(f"running case {case_id} lacks completed child artifacts; inspect and preserve its database")
    case_manifest = json.loads(manifest_path.read_text())
    if case_manifest.get("case") != job or case_manifest.get("source_sha") != SOURCE_SHA:
        raise SystemExit(f"running case {case_id} manifest does not match the selected source and plan")
    binary_name = "raw" if job["cohort"] == "E" else "service"
    if case_manifest.get("binary_sha256") != sha256_file(binary_paths[binary_name]):
        raise SystemExit(f"running case {case_id} binary hash differs from its manifest")
    if case_manifest.get("exit_status") != 0 or case_manifest.get("watchdog_failure"):
        raise SystemExit(f"running case {case_id} did not record a successful child exit; preserve its database")
    time_text = time_path.read_text(errors="replace")
    if "Exit status: 0" not in time_text:
        raise SystemExit(f"running case {case_id} lacks a successful whole-process exit record")
    with report_path.open() as report_file:
        report = json.load(report_file)
    if report.get("source_sha") != SOURCE_SHA or report.get("binary_sha256") != case_manifest["binary_sha256"]:
        raise SystemExit(f"running case {case_id} report provenance differs from its command manifest")
    input_fingerprint = verify_report(report, job)
    source_head, source_status = source_tree_state()
    if source_head != SOURCE_SHA or source_status:
        raise SystemExit("source tree is not the pinned clean checkout during completed-case recovery")
    database_path = result_root / "databases" / case_id
    if pathlib.Path(case_manifest.get("database_path", "")) != database_path:
        raise SystemExit(f"running case {case_id} database path differs from the selected result root")
    if not database_path.is_dir():
        raise SystemExit(f"running case {case_id} database is missing; preserve evidence and inspect")
    existing_fingerprint = journal.setdefault("input_fingerprints", {}).get(validation_group(job))
    if existing_fingerprint is not None and existing_fingerprint != input_fingerprint:
        raise SystemExit(f"running case {case_id} input prefix differs from prior completed cases")
    maximum_rss_line = next((line for line in time_text.splitlines() if "Maximum resident set size" in line), None)
    if maximum_rss_line is None:
        raise SystemExit(f"running case {case_id} lacks a whole-process maximum RSS record")
    maximum_rss_kb = int(maximum_rss_line.split(":", 1)[1].strip())
    if maximum_rss_kb * 1024 > PER_CASE_RSS_BUDGET_BYTES:
        raise SystemExit(f"running case {case_id} exceeded the whole-process RSS budget; preserve its database")
    group = validation_group(job)
    journal["input_fingerprints"][group] = input_fingerprint
    preserve_database(database_path, case_directory)
    case_manifest.update({
        "validated": True,
        "maximum_rss_kb": maximum_rss_kb,
        "database_removed_after_manifest": True,
        "recovered_after_completed_child_validation": True,
    })
    manifest_path.write_text(json.dumps(case_manifest, indent=2) + "\n")
    journal[case_id] = {
        "status": "complete",
        "maximum_rss_kb": maximum_rss_kb,
        "input_fingerprint_group": group,
        "recovered_after_completed_child_validation": True,
    }
    write_journal(result_root, journal)


def write_journal(result_root, journal):
    temporary_path = result_root / "journal.json.tmp"
    temporary_path.write_text(json.dumps(journal, indent=2) + "\n")
    temporary_path.replace(result_root / "journal.json")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--smoke-root", type=pathlib.Path, required=True)
    parser.add_argument("--result-root", type=pathlib.Path, required=True)
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--cohorts")
    args = parser.parse_args()
    if args.prepare_only == args.execute:
        raise SystemExit("choose exactly one of --prepare-only or --execute")
    if args.execute and args.cohorts is None:
        raise SystemExit("--execute requires an explicit --cohorts selection")
    selected_cohorts = ALL_COHORTS if args.cohorts is None else [cohort.strip() for cohort in args.cohorts.split(",") if cohort.strip()]
    if not selected_cohorts or len(set(selected_cohorts)) != len(selected_cohorts) or any(cohort not in ALL_COHORTS for cohort in selected_cohorts):
        raise SystemExit("--cohorts must be a unique comma-separated subset of A,B,C,D,E")
    selected_cohorts = [cohort for cohort in ALL_COHORTS if cohort in selected_cohorts]
    if selected_cohorts != ["D"]:
        raise SystemExit("this capacity controller only accepts --cohorts D")
    plan = make_plan(args.smoke_root, selected_cohorts)
    if args.prepare_only:
        print(json.dumps(plan, indent=2))
        return
    if not plan["preflight_budgets"]["selected_cohorts_within_caps"]:
        raise SystemExit("selected cohort projections exceed a per-case cap; do not start the matrix pending root review")
    binary_paths = {"service": SERVICE_BINARY, "raw": RAW_BINARY}
    preflight_report = preflight(binary_paths, args.result_root)
    if args.result_root.exists():
        journal_path = args.result_root / "journal.json"
        if not journal_path.is_file():
            raise SystemExit("existing result directory has no resume journal; refusing to start")
        journal = json.loads(journal_path.read_text())
        if journal.get("source_sha") != SOURCE_SHA or journal.get("plan_case_count") != len(plan["jobs"]):
            raise SystemExit("resume journal does not match this source and plan")
        if journal.get("plan_fingerprint") != plan["plan_fingerprint"] or journal.get("selected_cohorts") != selected_cohorts or journal.get("selected_case_count") != plan["selected_case_count"]:
            raise SystemExit("resume journal selection or plan fingerprint differs")
    else:
        args.result_root.mkdir(parents=True)
        journal = {
            "source_sha": SOURCE_SHA,
            "plan_case_count": len(plan["jobs"]),
            "selected_cohorts": selected_cohorts,
            "selected_case_count": plan["selected_case_count"],
            "selected_case_ids": plan["selected_case_ids"],
            "plan_fingerprint": plan["plan_fingerprint"],
            "pending_cohorts": plan["pending_cohorts"],
            "input_fingerprints": {},
        }
        (args.result_root / "plan.json").write_text(json.dumps(plan, indent=2) + "\n")
        (args.result_root / "preflight.json").write_text(json.dumps(preflight_report, indent=2) + "\n")
        write_journal(args.result_root, journal)
    jobs_by_id = {job["case_id"]: job for job in plan["selected_jobs"]}
    for case_id, entry in list(journal.items()):
        if case_id in jobs_by_id and entry.get("status") == "running":
            recover_completed_case(jobs_by_id[case_id], args.result_root, binary_paths, journal)
    if any(isinstance(value, dict) and value.get("status") == "running" for value in journal.values()):
        raise SystemExit("journal contains an interrupted running case; inspect its database before resuming")
    for case_id, entry in journal.items():
        if case_id in jobs_by_id and entry.get("status") == "complete":
            report_path = args.result_root / "cases" / case_id / "report.json"
            if not report_path.is_file() or (args.result_root / "databases" / case_id).exists():
                raise SystemExit(f"completed case {case_id} has incomplete evidence or a retained database")
    for job in plan["selected_jobs"]:
        status = journal.get(job["case_id"], {}).get("status")
        if status == "complete":
            continue
        if status is not None:
            raise SystemExit(f"case {job['case_id']} has non-resumable status {status}")
        run_case(job, args.result_root, binary_paths, journal)
    journal["selected_cohorts_complete"] = True
    journal["full_matrix_complete"] = not plan["pending_cohorts"]
    journal["status"] = "full-matrix-complete" if journal["full_matrix_complete"] else "selected-cohorts-complete"
    journal["finished_unix_time_ns"] = time.time_ns()
    write_journal(args.result_root, journal)
    print(json.dumps({"completed_cases": len(plan["selected_jobs"]), "plan_case_count": len(plan["jobs"]), "pending_cohorts": plan["pending_cohorts"], "result_root": str(args.result_root)}, indent=2))


if __name__ == "__main__":
    main()
