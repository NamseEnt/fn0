import csv
import hashlib
import json
import os
import pathlib
import shutil
import socket
import subprocess
import sys
import time

source_path = pathlib.Path("/bench/zfs/db/condition-shared-lock-20261019/source-88b43098d")
screen_root = pathlib.Path("/bench/zfs/db/condition-shared-lock-20261019/ab-44-88b43098d")
report_root = screen_root / "reports"
manifest_root = screen_root / "manifests"
log_root = screen_root / "rocksdb-logs"
binary_path = pathlib.Path("/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target/release/service_workload")
source_sha = "88b43098dab8736363d8c36446bbede27e1b77e1"
scenario_names = ["rw95", "rw80", "rw50", "rw20", "mt-mix"]
condition_lock_modes = ["exclusive", "shared"]
per_case_database_budget_bytes = 1 << 30
matrix_evidence_budget_bytes = 512 << 20
minimum_free_bytes = 2 << 30


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as source_file:
        while True:
            chunk = source_file.read(1024 * 1024)
            if not chunk:
                break
            digest.update(chunk)
    return digest.hexdigest()


def truthy_validation(validation):
    return isinstance(validation, dict) and bool(validation) and all(
        isinstance(entry, dict) and entry.get("passed") is True
        for entry in validation.values()
    )


def width16_attempt_count(report):
    return sum(
        category.get("attempts", 0)
        for name, category in report["counters"]["write_categories"].items()
        if name.endswith("width_16")
    )


def make_jobs():
    jobs = []
    for repetition_number in range(1, 4):
        scenario_rotation = (repetition_number - 1) % len(scenario_names)
        rotated_scenarios = scenario_names[scenario_rotation:] + scenario_names[:scenario_rotation]
        for scenario_position, scenario_name in enumerate(rotated_scenarios):
            mode_rotation = (repetition_number + scenario_position) % len(condition_lock_modes)
            rotated_modes = condition_lock_modes[mode_rotation:] + condition_lock_modes[:mode_rotation]
            for condition_lock_mode in rotated_modes:
                jobs.append((scenario_name, 16, 16, repetition_number, condition_lock_mode, False))
    for repetition_number in range(1, 4):
        scenario_rotation = (repetition_number - 1) % 2
        clients32_scenarios = ["rw20", "mt-mix"]
        rotated_scenarios = clients32_scenarios[scenario_rotation:] + clients32_scenarios[:scenario_rotation]
        mode_rotation = (repetition_number - 1) % len(condition_lock_modes)
        rotated_modes = condition_lock_modes[mode_rotation:] + condition_lock_modes[:mode_rotation]
        for scenario_name in rotated_scenarios:
            for condition_lock_mode in rotated_modes:
                jobs.append((scenario_name, 32, 32, repetition_number, condition_lock_mode, False))
    for condition_lock_mode in condition_lock_modes:
        jobs.append(("rw20", 16, 16, 1, condition_lock_mode, True))
    if len(jobs) != 44:
        raise SystemExit(f"unexpected case count: {len(jobs)}")
    return jobs


def make_case_label(case_number, job):
    scenario_name, clients, slots, repetition_number, condition_lock_mode, diagnostic = job
    diagnostic_suffix = "-diagnostic" if diagnostic else ""
    return (
        f"c{clients}-{scenario_name}-r{repetition_number}-"
        f"{condition_lock_mode}-slots{slots}{diagnostic_suffix}-n{case_number:02d}"
    )


def make_command(run_id, job, report_path, database_path):
    scenario_name, clients, slots, _, condition_lock_mode, diagnostic = job
    command = [
        str(binary_path),
        "--backend", "rocksdb",
        "--client-executor", "dedicated",
        "--scenario", scenario_name,
        "--data-dir", str(database_path),
        "--report", str(report_path),
        "--seed", "20261013",
        "--rows", "10000",
        "--value-size", "512",
        "--clients", str(clients),
        "--warmup-ms", "1000",
        "--measurement-ms", "5000",
        "--call-cap", "100000000",
        "--max-write-ledger", "5000000",
        "--require-full-window", "1",
        "--mutation-workload", "overwrite",
        "--overwrite-slots-per-client", "1024",
        "--latency-storage", "log1024",
        "--include-width-16", "1",
        "--revision-allocator", "reserved-ranges",
        "--shared-blocking-slots", str(slots),
        "--condition-key-lock-mode", condition_lock_mode,
    ]
    if diagnostic:
        command.extend([
            "--research-metrics", "1",
            "--read-stage-metrics", "1",
            "--detailed-write-metrics", "1",
        ])
    else:
        command.extend([
            "--research-metrics", "0",
            "--read-stage-metrics", "0",
            "--detailed-write-metrics", "0",
        ])
    return ["env", f"DODB_BUILD_RUN_ID={run_id}", "RUST_TEST_THREADS=1", *command]


def verify_report(report, job):
    scenario_name, clients, slots, _, condition_lock_mode, diagnostic = job
    configuration = report.get("configuration", {})
    timing = report.get("timing", {})
    counters = report.get("counters", {})
    rocksdb_config = configuration.get("rocksdb", {})
    if report.get("backend") != "rocksdb":
        raise ValueError("backend mismatch")
    if report.get("seed") != 20261013 or report.get("scenario") != scenario_name or configuration.get("clients") != clients:
        raise ValueError("seed or client count mismatch")
    if configuration.get("initial_rows_per_tenant") != 10000 or configuration.get("value_size_bytes") != 512:
        raise ValueError("input dimensions mismatch")
    if configuration.get("mutation_workload") != "overwrite" or configuration.get("overwrite_slots_per_client") != 1024:
        raise ValueError("overwrite input mismatch")
    if configuration.get("include_width_16") is not True or configuration.get("latency_storage") != "log1024":
        raise ValueError("width or latency storage mismatch")
    if rocksdb_config.get("revision_allocator") != "reserved-ranges":
        raise ValueError("revision allocator mismatch")
    if rocksdb_config.get("condition_key_lock_mode") != condition_lock_mode:
        raise ValueError("reported condition-key lock mode mismatch")
    if rocksdb_config.get("blocking_admission") != f"shared-{slots}":
        raise ValueError("admission slots mismatch")
    if configuration.get("research_metrics_enabled") is not diagnostic:
        raise ValueError("diagnostic mode mismatch")
    if configuration.get("measurement_requested_ms") != 5000 or configuration.get("warmup_requested_ms") != 1000:
        raise ValueError("duration mismatch")
    if timing.get("measurement_complete") is not True or timing.get("accepted_as_full_window") is not True:
        raise ValueError("measurement did not pass full-window gate")
    if timing.get("termination_reason") != "requested-duration" or timing.get("requested_measurement_ms") != 5000:
        raise ValueError("measurement stopped for an unexpected reason")
    if counters.get("request_limit_reached") is not False:
        raise ValueError("call cap reached")
    for counter_name in ["other_errors", "overloads"]:
        if counters.get(counter_name, 0) != 0:
            raise ValueError(f"unexpected {counter_name}: {counters[counter_name]}")
    if counters.get("verification_errors"):
        raise ValueError("verification errors were recorded")
    if counters.get("api_calls", 0) <= 0 or counters.get("successful_write_transactions", 0) <= 0:
        raise ValueError("no acknowledged transaction activity")
    if counters.get("successful_mutations", 0) <= 0 or counters.get("write_requests", 0) <= 0:
        raise ValueError("no successful mutation activity")
    if counters.get("intentional_condition_rejections", 0) <= 0:
        raise ValueError("expected conditional rejection activity is missing")
    if counters.get("natural_conflicts", 0) < 0:
        raise ValueError("invalid natural conflict count")
    if width16_attempt_count(report) <= 0:
        raise ValueError("width-16 path was not attempted")
    expected_tenant_count = 4 if scenario_name == "mt-mix" else 1
    if len(report.get("per_tenant", {})) != expected_tenant_count:
        raise ValueError("tenant count mismatch")
    if not truthy_validation(report.get("validation")) or not truthy_validation(report.get("reopen_validation")):
        raise ValueError("final or reopen validation failed")
    for tenant_name, validation in report["validation"].items():
        reopened_validation = report["reopen_validation"].get(tenant_name)
        if reopened_validation is None or validation.get("commit_identity_unique") is not True:
            raise ValueError("acknowledged commit identity validation is missing")
        if validation.get("commit_identity_entries") != reopened_validation.get("commit_identity_entries"):
            raise ValueError("reopened acknowledged commit count mismatch")
        if reopened_validation.get("commit_identity_unique") is not True:
            raise ValueError("reopened commit identity validation is missing")
    input_prefixes = counters.get("client_input_prefixes", [])
    if len(input_prefixes) != clients:
        raise ValueError("client input prefix count mismatch")
    if any(entry.get("prefix_limit") != 100 or entry.get("workflow_count") != 100 for entry in input_prefixes):
        raise ValueError("client input prefix limit mismatch")
    latency_counts = {
        "get": counters.get("ordinary_gets", 0),
        "preparatory_get": counters.get("preparatory_gets", 0),
        "query16": counters.get("query16_calls", 0),
        "condition_rejection": counters.get("intentional_condition_rejections", 0),
        "write_transaction_success": counters.get("successful_write_transactions", 0),
        "write_error": counters.get("write_requests", 0)
        - counters.get("successful_write_transactions", 0)
        - counters.get("intentional_condition_rejections", 0),
        "workflow": counters.get("api_calls", 0) - counters.get("preparatory_gets", 0),
    }
    expected_latency_kinds = {name for name, count in latency_counts.items() if count > 0}
    latency = counters.get("latency", {})
    if set(latency) != expected_latency_kinds:
        raise ValueError("latency kind set mismatch")
    for latency_name in expected_latency_kinds:
        latency_entry = latency[latency_name]
        if latency_entry.get("storage") != "log1024":
            raise ValueError(f"latency storage mismatch for {latency_name}")
        if latency_entry.get("sample_count") != latency_counts[latency_name] or latency_entry.get("count") != latency_counts[latency_name]:
            raise ValueError(f"latency sample count mismatch for {latency_name}")
    process_metrics = report.get("process_metrics", {})
    if process_metrics.get("rss_sample_count", 0) <= 0 or process_metrics.get("peak_sampled_rss_kb", 0) <= 0:
        raise ValueError("RSS sampling is incomplete")
    if process_metrics.get("cpu_seconds") is None or process_metrics.get("after", {}).get("complete") is not True:
        raise ValueError("CPU sampling is incomplete")
    return input_prefixes, expected_tenant_count


if len(sys.argv) != 2:
    raise SystemExit("usage: python3 run-condition-lock-ab.py EXPECTED_BINARY_SHA256")

binary_sha_expected = sys.argv[1]
if len(binary_sha_expected) != 64:
    raise SystemExit("expected binary SHA-256 must be a 64-character hex digest")
source_head = subprocess.check_output(["git", "-C", str(source_path), "rev-parse", "HEAD"], text=True).strip()
source_tree = subprocess.check_output(["git", "-C", str(source_path), "rev-parse", "HEAD^{tree}"], text=True).strip()
source_status = subprocess.check_output(["git", "-C", str(source_path), "status", "--porcelain"], text=True).strip()
if source_head != source_sha or source_status:
    raise SystemExit(f"source checkout mismatch or dirty tree: {source_head}, status={source_status!r}")
if not binary_path.is_file():
    raise SystemExit(f"service workload binary is missing: {binary_path}")
binary_sha_before = sha256_file(binary_path)
if binary_sha_before != binary_sha_expected:
    raise SystemExit(f"binary SHA mismatch: {binary_sha_before}")
screen_root.mkdir(parents=True, exist_ok=False)
report_root.mkdir()
manifest_root.mkdir()
log_root.mkdir()
jobs = make_jobs()
disk_before = shutil.disk_usage(screen_root.parent)
cpu_count = int(subprocess.check_output(["getconf", "_NPROCESSORS_ONLN"], text=True).strip())
memory_available_kb = next(
    int(line.split()[1])
    for line in pathlib.Path("/proc/meminfo").read_text().splitlines()
    if line.startswith("MemAvailable:")
)
active_process_names = {"cargo", "rustc", "service_workload", "unconditional_write_workload"}
active_processes = []
for process_comm_path in pathlib.Path("/proc").glob("[0-9]*/comm"):
    process_name = process_comm_path.read_text().strip()
    if process_name in active_process_names:
        active_processes.append({"pid": process_comm_path.parent.name, "name": process_name})
zfs_mount = subprocess.check_output(
    ["findmnt", "-n", "-o", "SOURCE,FSTYPE,TARGET", "--target", str(screen_root.parent)],
    text=True,
).strip()
preflight = {
    "hostname": socket.gethostname(),
    "cpu_count": cpu_count,
    "memory_available_kb": memory_available_kb,
    "zfs_mount": zfs_mount,
    "active_build_or_workload_processes": active_processes,
    "source_sha": source_head,
    "source_tree": source_tree,
    "binary_path": str(binary_path),
    "binary_sha256": binary_sha_before,
    "planned_cases": len(jobs),
    "per_case_database_budget_bytes": per_case_database_budget_bytes,
    "matrix_evidence_budget_bytes": matrix_evidence_budget_bytes,
    "free_bytes_before": disk_before.free,
    "minimum_free_bytes": minimum_free_bytes,
    "estimated_maximum_live_database_bytes": per_case_database_budget_bytes,
    "estimated_matrix_evidence_bytes": matrix_evidence_budget_bytes,
}
(screen_root / "preflight.json").write_text(json.dumps(preflight, indent=2) + "\n")
if active_processes:
    raise SystemExit("active build or workload process found; preflight preserved")
if disk_before.free < minimum_free_bytes:
    raise SystemExit("available storage is below the approved preflight threshold")
run_order = [
    {
        "case_number": case_number,
        "scenario": job[0],
        "clients": job[1],
        "slots": job[2],
        "repetition": job[3],
        "condition_key_lock_mode": job[4],
        "diagnostic": job[5],
    }
    for case_number, job in enumerate(jobs, 1)
]
(screen_root / "run-order.json").write_text(json.dumps(run_order, indent=2) + "\n")
field_names = [
    "case_number", "case_id", "scenario", "clients", "slots", "repetition",
    "condition_key_lock_mode", "diagnostic", "api_calls", "read_calls",
    "write_requests", "successful_transactions", "successful_mutations",
    "intentional_condition_rejections", "natural_conflicts", "throughput_api_calls_per_second",
    "throughput_successful_transactions_per_second", "latency_samples", "width16_attempts",
    "input_prefix_fingerprint", "tenant_fingerprints", "peak_rss_kb", "cpu_seconds",
    "database_bytes", "database_deleted", "passed",
]
performance_summary_path = screen_root / "summary-performance.csv"
diagnostic_summary_path = screen_root / "summary-diagnostic.csv"
paired_prefix_fingerprints = {}
with performance_summary_path.open("w", newline="") as performance_summary_file, diagnostic_summary_path.open("w", newline="") as diagnostic_summary_file:
    performance_writer = csv.DictWriter(performance_summary_file, fieldnames=field_names)
    diagnostic_writer = csv.DictWriter(diagnostic_summary_file, fieldnames=field_names)
    performance_writer.writeheader()
    diagnostic_writer.writeheader()
    performance_summary_file.flush()
    diagnostic_summary_file.flush()
    for case_number, job in enumerate(jobs, 1):
        case_label = make_case_label(case_number, job)
        run_id = f"condition-shared-lock-20261019-{case_label}-88b43098d"
        report_path = report_root / f"{case_label}.json"
        database_path = screen_root / f"db-{case_label}"
        stdout_path = screen_root / f"{case_label}.stdout.txt"
        command = make_command(run_id, job, report_path, database_path)
        (screen_root / f"{case_label}.command.json").write_text(json.dumps(command, indent=2) + "\n")
        case_preflight = {
            "case_number": case_number,
            "case_id": case_label,
            "source_sha": source_head,
            "binary_sha256": binary_sha_before,
            "free_bytes_before": shutil.disk_usage(screen_root.parent).free,
            "database_budget_bytes": per_case_database_budget_bytes,
            "database_path_absent": not database_path.exists(),
        }
        (screen_root / f"{case_label}.preflight.json").write_text(json.dumps(case_preflight, indent=2) + "\n")
        if database_path.exists() or shutil.disk_usage(screen_root.parent).free < minimum_free_bytes:
            raise SystemExit(f"case preflight failed; no run started: {case_label}")
        started = time.time()
        run_environment = dict(os.environ)
        run_environment.update({"DODB_BUILD_RUN_ID": run_id, "RUST_TEST_THREADS": "1"})
        with stdout_path.open("w") as stdout_file:
            try:
                completed = subprocess.run(
                    command,
                    cwd=source_path,
                    env=run_environment,
                    stdout=stdout_file,
                    stderr=subprocess.STDOUT,
                    check=False,
                    timeout=180,
                )
            except subprocess.TimeoutExpired as timeout_error:
                elapsed_seconds = time.time() - started
                (screen_root / f"{case_label}.exit-status.txt").write_text("timeout\n")
                raise SystemExit(
                    f"case timed out; preserved output and database: {case_label}, "
                    f"seconds={elapsed_seconds:.2f}"
                ) from timeout_error
        elapsed_seconds = time.time() - started
        (screen_root / f"{case_label}.exit-status.txt").write_text(f"{completed.returncode}\n")
        if completed.returncode != 0 or not report_path.is_file():
            raise SystemExit(f"case failed; preserved source artifacts and database: {case_label}, exit={completed.returncode}, seconds={elapsed_seconds:.2f}")
        report = json.loads(report_path.read_text())
        try:
            input_prefixes, expected_tenant_count = verify_report(report, job)
        except ValueError as validation_error:
            raise SystemExit(f"case validation failed; preserved artifacts: {case_label}: {validation_error}")
        current_prefixes = {str(entry["client_index"]): entry.get("fingerprint") for entry in input_prefixes}
        scenario_name, clients, _, repetition_number, condition_lock_mode, diagnostic = job
        prefix_pair_key = (scenario_name, clients, diagnostic)
        if prefix_pair_key in paired_prefix_fingerprints and paired_prefix_fingerprints[prefix_pair_key] != current_prefixes:
            raise SystemExit(f"paired input prefix mismatch; preserved artifacts: {case_label}")
        paired_prefix_fingerprints.setdefault(prefix_pair_key, current_prefixes)
        database_resolved = database_path.resolve(strict=True)
        if database_path.is_symlink() or screen_root.resolve() not in database_resolved.parents:
            raise SystemExit(f"unsafe database cleanup path; preserved artifacts: {case_label}")
        manifest_rows = []
        log_files = []
        metadata_hashes = []
        for database_file in sorted(database_resolved.rglob("*")):
            if database_file.is_symlink():
                raise SystemExit(f"database contains a symlink; preserved artifacts: {case_label}")
            if database_file.is_file():
                relative_path = database_file.relative_to(database_resolved).as_posix()
                file_hash = sha256_file(database_file)
                file_size = database_file.stat().st_size
                manifest_rows.append(f"{relative_path}\t{file_size}\t{file_hash}")
                if database_file.name.startswith("LOG"):
                    log_copy = log_root / case_label / relative_path
                    log_copy.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(database_file, log_copy)
                    if sha256_file(log_copy) != file_hash:
                        raise SystemExit(f"RocksDB LOG copy hash mismatch; preserved artifacts: {case_label}")
                    log_files.append({"relative_path": relative_path, "bytes": file_size, "sha256": file_hash})
                if database_file.name in {"CURRENT", "IDENTITY", "LOG", "MANIFEST-000001"} or database_file.suffix in {".sst", ".log"}:
                    metadata_hashes.append(f"{file_hash}  {relative_path}")
        if not log_files:
            raise SystemExit(f"RocksDB LOG content is missing; preserved database: {case_label}")
        manifest_path = manifest_root / f"{case_label}-files.tsv"
        manifest_path.write_text("relative_path\tbytes\tsha256\n" + "\n".join(manifest_rows) + "\n")
        (manifest_root / f"{case_label}-metadata-sha256.txt").write_text("\n".join(metadata_hashes) + "\n")
        (manifest_root / f"{case_label}-logs.json").write_text(json.dumps(log_files, indent=2) + "\n")
        database_bytes = sum(database_path_entry.stat().st_size for database_path_entry in database_resolved.rglob("*") if database_path_entry.is_file())
        if database_bytes > per_case_database_budget_bytes:
            raise SystemExit(f"database exceeded its preflight budget; preserved database: {case_label}")
        counters = report["counters"]
        tenant_fingerprints = sorted(
            entry.get("schedule_fingerprint", "")
            for entry in report.get("per_tenant", {}).values()
        )
        metrics = {
            "case_number": case_number,
            "case_id": case_label,
            "scenario": scenario_name,
            "clients": clients,
            "slots": job[2],
            "repetition": repetition_number,
            "condition_key_lock_mode": condition_lock_mode,
            "diagnostic": int(diagnostic),
            "api_calls": counters.get("api_calls", 0),
            "read_calls": counters.get("read_calls", 0),
            "write_requests": counters.get("write_requests", 0),
            "successful_transactions": counters.get("successful_write_transactions", 0),
            "successful_mutations": counters.get("successful_mutations", 0),
            "intentional_condition_rejections": counters.get("intentional_condition_rejections", 0),
            "natural_conflicts": counters.get("natural_conflicts", 0),
            "throughput_api_calls_per_second": report.get("throughput_per_second", {}).get("api_calls"),
            "throughput_successful_transactions_per_second": report.get("throughput_per_second", {}).get("successful_write_transactions"),
            "latency_samples": sum(entry.get("sample_count", 0) for entry in counters.get("latency", {}).values()),
            "width16_attempts": width16_attempt_count(report),
            "input_prefix_fingerprint": hashlib.sha256(json.dumps(current_prefixes, sort_keys=True).encode()).hexdigest(),
            "tenant_fingerprints": ";".join(tenant_fingerprints),
            "peak_rss_kb": report["process_metrics"]["peak_sampled_rss_kb"],
            "cpu_seconds": report["process_metrics"]["cpu_seconds"],
            "database_bytes": database_bytes,
            "database_deleted": True,
            "passed": True,
        }
        shutil.rmtree(database_resolved)
        summary_writer = diagnostic_writer if diagnostic else performance_writer
        summary_file = diagnostic_summary_file if diagnostic else performance_summary_file
        summary_writer.writerow(metrics)
        summary_file.flush()
        print(json.dumps({"completed": case_number, "total": len(jobs), "case_id": case_label, "elapsed_seconds": round(elapsed_seconds, 2), "api_calls": metrics["api_calls"], "rss_kb": metrics["peak_rss_kb"], "database_deleted": True}), flush=True)

binary_sha_after = sha256_file(binary_path)
if binary_sha_after != binary_sha_before:
    raise SystemExit(f"binary changed during A/B: before={binary_sha_before}, after={binary_sha_after}")
if subprocess.check_output(["git", "-C", str(source_path), "status", "--porcelain"], text=True).strip():
    raise SystemExit("source checkout became dirty during A/B")
disk_after = shutil.disk_usage(screen_root.parent)
(screen_root / "completion.json").write_text(json.dumps({
    "source_sha": source_head,
    "source_tree": source_tree,
    "binary_sha256_before": binary_sha_before,
    "binary_sha256_after": binary_sha_after,
    "planned_cases": len(jobs),
    "completed_performance_cases": sum(1 for _ in performance_summary_path.open()) - 1,
    "completed_diagnostic_cases": sum(1 for _ in diagnostic_summary_path.open()) - 1,
    "completed_cases": sum(1 for _ in performance_summary_path.open()) + sum(1 for _ in diagnostic_summary_path.open()) - 2,
    "all_cases_passed": True,
    "free_bytes_after": disk_after.free,
    "diagnostic_cases_separated_from_performance_summary": True,
}, indent=2) + "\n")
