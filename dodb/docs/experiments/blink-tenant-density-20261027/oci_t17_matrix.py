import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time


RECIPE_SHA256 = "5c2773034216a942486197a64400c70d8ac20fc22e78ac381ec39aba4a2b3bab"
POPULATIONS = [1, 8, 16, 24, 28, 32]
BACKENDS = ["blink-paged", "blink-resident", "minimal-native-shared"]
MINIMUM_ZFS_RESERVE_BYTES = 10 * 1024**3
MATRIX_FREE_BYTES = 47 * 1024**3
MEMORY_MAX_BYTES = 8 * 1024**3
MEMORY_HIGH_BYTES = 7 * 1024**3
APPLICATION_MEMORY_BYTES = 6 * 1024**3
TENANT_MEMORY_BYTES = 512 * 1024**2
BACKEND_DISK_BYTES = 32 * 1024**3
TENANT_DISK_BYTES = 2 * 1024**3
SHARED_RESOURCES = [
    "--rocksdb-shared-cache-bytes",
    str(64 * 1024**2),
    "--rocksdb-shared-wbm-bytes",
    str(32 * 1024**2),
    "--rocksdb-write-buffer-bytes",
    str(1024**2),
    "--rocksdb-flush-threads",
    "1",
    "--rocksdb-compaction-threads",
    "1",
    "--rocksdb-max-open-files",
    "64",
]
BLINK_RESOURCES = [
    "--blink-wal-layout",
    "segmented-bases-coalesced",
    "--blink-checkpoint-execution",
    "background",
    "--blink-wal-segment-bytes",
    "8192",
    "--blink-checkpoint-wal-bytes",
    str(1024**2),
]
ACTIVE_RUN_DIR = None


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as input_file:
        for chunk in iter(lambda: input_file.read(1024**2), b""):
            digest.update(chunk)
    return digest.hexdigest()


def canonical_sha256(value):
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def available_bytes(path):
    filesystem = os.statvfs(path)
    return filesystem.f_bavail * filesystem.f_frsize


def meminfo():
    values = {}
    for line in Path("/proc/meminfo").read_text().splitlines():
        name, value = line.split(":", 1)
        if name in ["MemTotal", "MemAvailable", "SwapFree"]:
            values[name] = int(value.split()[0]) * 1024
    return values


def arc_state():
    return {
        "arc_max": Path("/sys/module/zfs/parameters/zfs_arc_max").read_text().strip(),
        "arcstats": Path("/proc/spl/kstat/zfs/arcstats").read_text(),
        "arc_summary": subprocess.run(
            ["arc_summary"], check=True, capture_output=True, text=True
        ).stdout,
    }


def host_state(zfs_path):
    return {
        "timestamp_unix_seconds": time.time(),
        "zfs_free_bytes": available_bytes(zfs_path),
        "meminfo": meminfo(),
        "arc": arc_state(),
        "device_stat": Path("/sys/class/block/sda4/stat").read_text().strip(),
        "cpu_stat": Path("/proc/stat").read_text().splitlines()[:3],
    }


def active_workload_processes():
    active = []
    for process_path in Path("/proc").glob("[0-9]*"):
        executable_path = process_path / "exe"
        try:
            executable = Path(os.readlink(executable_path)).name
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        if executable in ["tenant_density_workload", "memory_diagnostic_worker"]:
            active.append({"pid": int(process_path.name), "executable": executable})
    return active


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    temporary.replace(path)


def append_progress(run_dir, outcome, accepted_populations, completed_cells):
    progress_path = run_dir / "progress.jsonl"
    event = {
        "event": "cell_completed",
        "label": outcome["label"],
        "accepted_populations": accepted_populations,
        "validated": outcome["validated"],
        "database_allocated_bytes": outcome["database_allocated_bytes"],
        "metrics": outcome["metrics"],
        "input_identity": outcome["input_identity"],
        "physical_read_coverage": outcome["physical_read_coverage"],
        "ended_unix_seconds": outcome["ended_unix_seconds"],
    }
    with progress_path.open("a") as progress_file:
        progress_file.write(json.dumps(event, sort_keys=True) + "\n")
        progress_file.flush()
        os.fsync(progress_file.fileno())
    write_json(
        run_dir / "progress.json",
        {
            "status": "running",
            "accepted_populations": accepted_populations,
            "completed_cells": completed_cells,
            "last_completed_cell": outcome["label"],
        },
    )


def create_cgroup(path):
    path.mkdir()
    (path / "cgroup.subtree_control").write_text("+memory")
    if (path / "cgroup.procs").read_text().strip():
        raise RuntimeError(f"cgroup is not empty: {path}")


def remove_cgroup(path):
    for child in path.iterdir():
        if child.is_dir():
            remove_cgroup(child)
    if (path / "cgroup.procs").read_text().strip():
        raise RuntimeError(f"cgroup still has processes: {path}")
    path.rmdir()


def remove_failed_cgroup(path):
    if path.exists():
        remove_cgroup(path)


def checkpoint_cgroup_deltas(path):
    deltas = {}
    for group in [path, *[entry for entry in path.iterdir() if entry.is_dir()]]:
        event_path = group / "memory.events.local"
        if not event_path.exists():
            continue
        counters = {}
        for line in event_path.read_text().splitlines():
            name, value = line.split()
            counters[name] = int(value)
        deltas[str(group)] = counters
    return deltas


def cgroup_stop_reason(before, after):
    for group_path, final in after.items():
        initial = before.get(group_path, {})
        for counter in ["high", "max", "oom", "oom_kill"]:
            if final.get(counter, 0) > initial.get(counter, 0):
                return f"{counter} event in {group_path}"
    return None


def allocated_bytes(path):
    total = 0
    for root, directories, files in os.walk(path, followlinks=False):
        directories[:] = [
            name for name in directories if not (Path(root) / name).is_symlink()
        ]
        for filename in files:
            file_path = Path(root) / filename
            if file_path.is_symlink():
                raise RuntimeError(f"symlink in experiment database: {file_path}")
            total += file_path.stat().st_blocks * 512
    return total


def backend_arguments(backend):
    if backend == "blink-paged":
        return BLINK_RESOURCES + [
            "--blink-demand-paging",
            "on",
            "--blink-paged-cache-bytes",
            str(64 * 1024**2),
            "--blink-memory-limit-bytes",
            str(APPLICATION_MEMORY_BYTES),
            "--blink-memory-per-tenant-bytes",
            str(TENANT_MEMORY_BYTES),
            "--blink-disk-limit-bytes",
            str(BACKEND_DISK_BYTES),
            "--blink-disk-per-tenant-bytes",
            str(TENANT_DISK_BYTES),
        ]
    if backend == "blink-resident":
        return BLINK_RESOURCES
    if backend == "minimal-native-shared":
        return SHARED_RESOURCES
    raise ValueError(f"unknown backend {backend}")


def scenario_values(recipe, scenario_id):
    common = recipe["common"]
    selected = next(item for item in recipe["recipes"] if item["id"] == scenario_id)
    return {
        "arrival_phase": selected["arrival_phase"],
        "mutation_key_mode": selected["mutation_key_mode"],
        "initial_rows_per_tenant": selected.get(
            "cache_stress_all_active_tenants_rows",
            common["dataset"]["initial_rows_per_tenant"],
        ),
        "long_reader_retention_ms": selected.get("long_reader_retention_ms", 0),
        "profile": selected["profile"],
        "recipe": selected,
        "scenario_recipe_sha256": canonical_sha256(
            {"common": common, "recipe": selected}
        ),
    }


def invocation(binary, backend, scenario, population, repetition, data_path, report_path, cgroup_path, source_commit, common):
    return [
        str(binary),
        "--backend",
        "blink" if backend.startswith("blink-") else "minimal-native-shared",
        "--data-dir",
        str(data_path),
        "--report",
        str(report_path),
        "--seed",
        str(common["dataset"]["seed"]),
        "--registered-tenants",
        str(population),
        "--active-tenants",
        str(population),
        "--opened-idle-tenants",
        "0",
        "--client-count",
        str(common["client_count_per_tenant"]),
        "--duration-ms",
        str(common["measurement_duration_ms"]),
        "--warmup-duration-ms",
        str(common["warmup_duration_ms"]),
        "--recovery-duration-ms",
        str(common["recovery_duration_ms"]),
        "--slo-window-ms",
        str(common["slo"]["window_ms"]),
        "--long-reader-retention-ms",
        str(scenario["long_reader_retention_ms"]),
        "--drop-page-cache-before-warmup",
        "on",
        "--mutation-key-mode",
        scenario["mutation_key_mode"],
        "--mutation-key-epoch",
        "0",
        "--initial-rows",
        str(scenario["initial_rows_per_tenant"]),
        "--value-bytes",
        str(common["dataset"]["value_bytes"]),
        "--topology",
        "fixed-per-tenant",
        "--tenant-arrival-phase",
        scenario["arrival_phase"],
        "--per-tenant-rate",
        str(common["arrival_rate_per_tenant_per_second"]),
        "--queue-bound",
        str(common["queue_bound_per_client"]),
        "--max-in-flight",
        str(common["maximum_in_flight_per_client"]),
        "--service-memory-admission-bytes",
        str(common["service_memory_admission_bytes"]),
        "--request-timeout-ms",
        "5000",
        "--drain-timeout-ms",
        "15000",
        "--profile",
        scenario["profile"],
        "--width",
        "mixed",
        "--backend-cgroup-root",
        str(cgroup_path),
        "--backend-memory-max-bytes",
        str(MEMORY_MAX_BYTES),
        "--backend-memory-high-bytes",
        str(MEMORY_HIGH_BYTES),
        "--pressure-sample-interval-ms",
        "5000",
    ] + backend_arguments(backend)


def slo_windows(report):
    workload = report.get("workload", {})
    phases = [("measurement", workload)]
    phase_workloads = workload.get("phase_workloads", {})
    for extension in phase_workloads.get("checkpoint_cycle_extensions", []):
        phases.append((extension.get("phase", "measurement-extension"), extension.get("workload", {})))
    recovery = phase_workloads.get("recovery_after_reopen")
    if recovery is not None:
        phases.append(("recovery", recovery))
    windows = []
    for phase_name, phase_report in phases:
        phase_windows = phase_report.get("slo_windows", [])
        if not phase_windows:
            raise RuntimeError(f"{phase_name} has no SLO windows")
        if not all(window.get("score", {}).get("passed") is True for window in phase_windows):
            raise RuntimeError(f"{phase_name} has a failed SLO window")
        windows.extend(
            {
                "phase": phase_name,
                **{key: value for key, value in window.items() if key != "score"},
                **window["score"],
            }
            for window in phase_windows
        )
    return windows


def validate_report(report, backend, scenario):
    if report.get("status") != "completed":
        raise RuntimeError(f"report status is {report.get('status')}")
    if report.get("post_measurement_full_population_oracle", {}).get("passed") is not True:
        raise RuntimeError("post-measurement full-population oracle failed")
    if report.get("reopened_full_population_oracle", {}).get("passed") is not True:
        raise RuntimeError("reopened full-population oracle failed")
    workload = report.get("workload", {})
    if workload.get("cell_valid") is not True or workload.get("valid_for_oracle") is not True:
        raise RuntimeError("workload correctness or operational validity failed")
    if workload.get("service_memory_admission_bytes") != 6 * 1024**3:
        raise RuntimeError("the fixed service memory admission budget was not applied")
    if workload.get("service_memory_admission_in_flight_limit") != 96:
        raise RuntimeError("the 6 GiB request admission ceiling was not applied")
    if report.get("host_cache_preconditioning", {}).get("status") != (
        "page_cache_dropped_before_fixed_warmup"
    ):
        raise RuntimeError("the fixed page-cache preconditioning pass was not recorded")
    if not report.get("host_cache_after_warmup", {}).get("arcstats"):
        raise RuntimeError("host cache state after warm-up is missing")
    windows = slo_windows(report)
    measurement_windows = [
        window
        for window in windows
        if window["phase"] == "measurement"
        or window["phase"].startswith("measurement-extension-")
    ]
    recovery_windows = [window for window in windows if window["phase"] == "recovery"]
    if len(measurement_windows) < 6:
        raise RuntimeError("the 30-minute measurement SLO windows are incomplete")
    if len(recovery_windows) != 6:
        raise RuntimeError("the 30-minute recovery SLO windows are incomplete")
    if any(window.get("duration_ms") != 300000 for window in windows):
        raise RuntimeError("an SLO window does not match the fixed 5-minute duration")
    if workload.get("slo_stop_reason") != "duration_completed":
        raise RuntimeError("measurement stopped before completing its fixed duration")
    recovery_report = workload.get("phase_workloads", {}).get("recovery_after_reopen", {})
    if recovery_report.get("slo_stop_reason") != "duration_completed":
        raise RuntimeError("recovery stopped before completing its fixed duration")
    all_phase_reports = [workload]
    all_phase_reports.extend(
        extension.get("workload", {})
        for extension in workload.get("phase_workloads", {}).get("checkpoint_cycle_extensions", [])
    )
    all_phase_reports.append(recovery_report)
    for phase_report in all_phase_reports:
        if phase_report.get("service_memory_admission_bytes") != 6 * 1024**3:
            raise RuntimeError("the fixed service memory admission budget was not applied in every phase")
        if phase_report.get("service_memory_admission_in_flight_limit") != 96:
            raise RuntimeError("the 6 GiB request admission ceiling was not applied in every phase")
    if any(
        extension.get("workload", {}).get("slo_stop_reason") != "duration_completed"
        for extension in workload.get("phase_workloads", {}).get("checkpoint_cycle_extensions", [])
    ):
        raise RuntimeError("checkpoint extension stopped before completing its fixed duration")
    if backend.startswith("blink-") and workload.get("checkpoint_cycle_gate_passed") is not True:
        raise RuntimeError("B-link checkpoint cycle gate failed")
    if backend.startswith("blink-"):
        debt_windows = list(workload.get("maintenance_debt_windows", []))
        for extension in workload.get("phase_workloads", {}).get("checkpoint_cycle_extensions", []):
            debt_windows.extend(extension.get("workload", {}).get("maintenance_debt_windows", []))
        if len(debt_windows) < 4 or any(not isinstance(debt, int) for debt in debt_windows[-4:]):
            raise RuntimeError("B-link checkpoint debt is missing from the final 20 minutes")
        if any(current > previous for previous, current in zip(debt_windows[-4:], debt_windows[-3:])):
            raise RuntimeError("B-link checkpoint debt increased during the final 20 minutes")
    if scenario["long_reader_retention_ms"] > 0:
        if report.get("long_reader_retention", {}).get("passed") is not True:
            raise RuntimeError("long-reader retention gate failed")
    pressure = report.get("backend_memory_pressure", {})
    if pressure.get("enabled") is not True or pressure.get("samples_truncated") is not False:
        raise RuntimeError("cgroup pressure evidence is absent or truncated")
    extension_names = [
        extension.get("phase")
        for extension in workload.get("phase_workloads", {}).get("checkpoint_cycle_extensions", [])
    ]
    for epoch_name in ["measurement", *extension_names, "recovery"]:
        epoch = pressure.get("epochs", {}).get(epoch_name, {})
        for boundary in ["initial", "final"]:
            snapshot = epoch.get(boundary, {})
            if snapshot.get("memory.max") != MEMORY_MAX_BYTES:
                raise RuntimeError(f"{epoch_name} cgroup memory.max differs from 8 GiB")
            if snapshot.get("memory.high") != MEMORY_HIGH_BYTES:
                raise RuntimeError(f"{epoch_name} cgroup memory.high differs from 7 GiB")
            if snapshot.get("memory.swap.max") != 0:
                raise RuntimeError(f"{epoch_name} cgroup swap limit differs from zero")
        if not isinstance(epoch.get("final", {}).get("memory.peak"), int):
            raise RuntimeError(f"{epoch_name} cgroup peak memory is missing")
    for epoch in pressure.get("epochs", {}).values():
        for counter in ["high", "max", "oom", "oom_kill"]:
            if epoch.get("local_event_delta", {}).get(counter, 0) != 0:
                raise RuntimeError(f"cgroup {counter} limit event occurred")
    return windows


def percentile(values, percentile_value):
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int((len(ordered) - 1) * percentile_value))]


def decision_metrics(report):
    windows = slo_windows(report)
    workload = report["workload"]
    measurement_windows = [window for window in windows if window["phase"] != "recovery"]
    recovery_windows = [window for window in windows if window["phase"] == "recovery"]
    pressure_epochs = report["backend_memory_pressure"]["epochs"]
    return {
        "measurement_scheduled": workload.get("scheduled"),
        "measurement_successful": workload.get("successful"),
        "measurement_dropped": workload.get("dropped"),
        "measurement_elapsed_ms": workload.get("offered_load_elapsed_ms"),
        "measurement_successful_per_second": (
            workload["successful"] / (workload["offered_load_elapsed_ms"] / 1000)
            if isinstance(workload.get("successful"), (int, float))
            and isinstance(workload.get("offered_load_elapsed_ms"), (int, float))
            and workload["offered_load_elapsed_ms"] > 0
            else None
        ),
        "measurement_max_p99_read_ms": max(
            window["read_p99_upper_bound_ms"]
            for window in measurement_windows
        ),
        "measurement_max_p99_write_ms": max(
            window["write_p99_upper_bound_ms"]
            for window in measurement_windows
        ),
        "recovery_windows": len(recovery_windows),
        "recovery_max_p99_read_ms": max(
            window["read_p99_upper_bound_ms"] for window in recovery_windows
        ),
        "recovery_max_p99_write_ms": max(
            window["write_p99_upper_bound_ms"] for window in recovery_windows
        ),
        "measurement_cgroup_peak_memory_bytes": max(
            pressure_epochs[epoch_name]["final"]["memory.peak"]
            for epoch_name in [
                "measurement",
                *[
                    extension.get("phase")
                    for extension in workload.get("phase_workloads", {}).get("checkpoint_cycle_extensions", [])
                ],
            ]
        ),
        "recovery_cgroup_peak_memory_bytes": pressure_epochs["recovery"]["final"]["memory.peak"],
        "checkpoint_cycles": workload.get("successful_checkpoint_cycles"),
        "final_20_minute_checkpoint_debt_bytes": (
            [
                *workload.get("maintenance_debt_windows", []),
                *[
                    debt
                    for extension in workload.get("phase_workloads", {}).get("checkpoint_cycle_extensions", [])
                    for debt in extension.get("workload", {}).get("maintenance_debt_windows", [])
                ],
            ][-4:]
        ),
        "measured_duration_ms_including_extensions": workload.get(
            "measured_duration_ms_including_extensions"
        ),
    }


def physical_read_coverage(report, scenario, population, recipe):
    cache_snapshot = report.get("host_cache_after_warmup", {})
    meminfo_values = {}
    for line in cache_snapshot.get("meminfo", "").splitlines():
        name, value = line.split(":", 1)
        meminfo_values[name] = int(value.split()[0]) * 1024
    arc_values = {}
    for line in cache_snapshot.get("arcstats", "").splitlines()[2:]:
        fields = line.split()
        if len(fields) >= 3 and fields[0] in ["size", "c_max"]:
            arc_values[fields[0]] = int(fields[2])
    value_payload_bytes = (
        population
        * scenario["initial_rows_per_tenant"]
        * recipe["common"]["dataset"]["value_bytes"]
    )
    host_cache_bytes = arc_values.get("size", 0) + meminfo_values.get("Cached", 0)
    offer = report["workload"]
    before_read_bytes = offer["offer_boundary_host_device_before"].get("read_bytes")
    after_read_bytes = offer["offer_boundary_host_device_after"].get("read_bytes")
    device_read_delta = (
        after_read_bytes - before_read_bytes
        if isinstance(before_read_bytes, int) and isinstance(after_read_bytes, int)
        else None
    )
    coverage_ratio = (
        host_cache_bytes / value_payload_bytes if value_payload_bytes > 0 else None
    )
    physically_read_eligible = (
        coverage_ratio is not None
        and coverage_ratio < 0.25
        and device_read_delta is not None
        and device_read_delta > 0
    )
    return {
        "dataset_value_payload_bytes": value_payload_bytes,
        "arc_size_bytes": arc_values.get("size"),
        "linux_cached_bytes": meminfo_values.get("Cached"),
        "combined_cache_coverage_ratio": coverage_ratio,
        "host_device_read_delta_bytes": device_read_delta,
        "physical_read_gate": (
            "passed"
            if physically_read_eligible
            else "unverified_warm_cache_or_no_device_read_evidence"
        ),
        "device_counter_scope": "whole host device including unrelated I/O",
    }


def run_cell(args, recipe, binary, run_root, cgroup_base, scenario_id, population, repetition, backend, expected_hashes):
    scenario = scenario_values(recipe, scenario_id)
    label = f"n{population}-{scenario_id}-r{repetition}-{backend}"
    active_processes = active_workload_processes()
    if active_processes:
        raise RuntimeError(f"workload process was active before {label}: {active_processes}")
    cell_root = run_root / label
    cell_root.mkdir(parents=True, exist_ok=False)
    data_path = args.zfs_path / "databases" / label
    data_path.mkdir(parents=True, exist_ok=False)
    report_path = cell_root / "report.json"
    log_path = cell_root / "process.log"
    command_path = cell_root / "command.json"
    binary_manifest_path = cell_root / "binary-manifest.json"
    cgroup_path = cgroup_base / label
    create_cgroup(cgroup_path)
    before = host_state(args.zfs_path)
    if before["zfs_free_bytes"] < MATRIX_FREE_BYTES:
        raise RuntimeError(f"47 GiB gate failed before {label}: {before['zfs_free_bytes']}")
    if int(before["arc"]["arc_max"]) != 2 * 1024**3:
        raise RuntimeError(f"ZFS ARC maximum is not 2 GiB before {label}")
    if before["meminfo"]["MemAvailable"] < 1536 * 1024**2:
        raise RuntimeError(f"MemAvailable reserve failed before {label}")
    invocation_args = invocation(
        binary,
        backend,
        scenario,
        population,
        repetition,
        data_path,
        report_path,
        cgroup_path,
        args.source_commit,
        recipe["common"],
    )
    write_json(
        command_path,
        {
            "label": label,
            "source_commit": args.source_commit,
            "controller_sha256": sha256(Path(__file__).resolve()),
            "binary_sha256": sha256(binary),
            "worker_sha256": args.worker_sha256,
            "native_archive_sha256": args.native_sha256,
            "recipe_sha256": RECIPE_SHA256,
            "scenario_recipe_sha256": scenario["scenario_recipe_sha256"],
            "scenario": scenario,
            "command": invocation_args,
            "host_before": before,
            "scope": "T17 fixed-policy workload; warm-up includes the five-minute cache preconditioning pass",
        },
    )
    write_json(
        binary_manifest_path,
        {
            "source_commit": args.source_commit,
            "controller_sha256": sha256(Path(__file__).resolve()),
            "workload_sha256": sha256(binary),
            "worker_sha256": args.worker_sha256,
            "native_archive_sha256": args.native_sha256,
            "recipe_sha256": RECIPE_SHA256,
            "scenario_recipe_sha256": scenario["scenario_recipe_sha256"],
        },
    )
    environment = os.environ.copy()
    environment["DODB_SOURCE_REVISION"] = args.source_commit
    started = time.time()
    with log_path.open("x") as output:
        child = subprocess.Popen(
            invocation_args,
            env=environment,
            stdout=output,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        cgroup_initial = checkpoint_cgroup_deltas(cgroup_path)
        stop_reason = None
        while child.poll() is None:
            time.sleep(5)
            if available_bytes(args.zfs_path) < MINIMUM_ZFS_RESERVE_BYTES:
                stop_reason = "ZFS available space fell below 10 GiB"
            current_counters = checkpoint_cgroup_deltas(cgroup_path)
            stop_reason = stop_reason or cgroup_stop_reason(cgroup_initial, current_counters)
            if data_path.exists() and allocated_bytes(data_path) > BACKEND_DISK_BYTES:
                stop_reason = "backend allocated data exceeded the 32 GiB disk cap"
            if stop_reason:
                os.killpg(child.pid, signal.SIGTERM)
                try:
                    child.wait(timeout=30)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait()
                break
        exit_code = child.wait()
    remaining_processes = active_workload_processes()
    if remaining_processes:
        stop_reason = stop_reason or f"workload processes remained after {label}: {remaining_processes}"
    after = host_state(args.zfs_path)
    database_allocated_bytes = allocated_bytes(data_path) if data_path.exists() else None
    if database_allocated_bytes is not None and database_allocated_bytes > BACKEND_DISK_BYTES:
        stop_reason = stop_reason or "backend allocated data exceeded the 32 GiB disk cap"
    outcome = {
        "label": label,
        "exit_code": exit_code,
        "controller_stop_reason": stop_reason,
        "remaining_workload_processes": remaining_processes,
        "started_unix_seconds": started,
        "ended_unix_seconds": time.time(),
        "host_after": after,
        "database_allocated_bytes": database_allocated_bytes,
        "report_sha256": sha256(report_path) if report_path.exists() else None,
        "process_log_sha256": sha256(log_path),
    }
    if stop_reason:
        write_json(cell_root / "outcome.json", outcome)
        remove_failed_cgroup(cgroup_path)
        raise RuntimeError(f"{label} stopped: {stop_reason}")
    if exit_code != 0 or not report_path.exists():
        write_json(cell_root / "outcome.json", outcome)
        remove_failed_cgroup(cgroup_path)
        raise RuntimeError(f"{label} failed; exit={exit_code}, report={report_path.exists()}")
    report = json.loads(report_path.read_text())
    try:
        windows = validate_report(report, backend, scenario)
        input_manifest_path = Path(report["pre_execution_inputs_manifest"])
        if not input_manifest_path.is_file():
            raise RuntimeError("pre-execution input manifest is missing")
        manifest = json.loads(input_manifest_path.read_text())
        schedules = {}
        for phase in [
            "warmup",
            "measurement",
            "measurement_maximum_extension",
            "recovery",
        ]:
            phase_manifest = manifest[phase]
            if phase == "measurement_maximum_extension":
                phase_manifest = phase_manifest["fingerprint"]
            schedules[phase] = phase_manifest["ordered_request_schedule_sha256"]
        identity = {
            "seed_manifest_sha256": manifest["seed_manifest_sha256"],
            "ordered_request_schedule_sha256": schedules,
            "recipe_sha256": RECIPE_SHA256,
            "scenario_recipe_sha256": scenario["scenario_recipe_sha256"],
        }
        comparison_key = (population, scenario_id, repetition)
        expected = expected_hashes.setdefault(comparison_key, identity)
        if identity != expected:
            raise RuntimeError("input or schedule hash does not match paired backend")
        if windows[-1]["phase"] != "recovery":
            raise RuntimeError("30-minute recovery SLO windows are missing")
    except Exception as error:
        outcome["validation_error"] = str(error)
        write_json(cell_root / "outcome.json", outcome)
        remove_failed_cgroup(cgroup_path)
        raise RuntimeError(f"{label} failed validation: {error}") from error
    outcome["input_identity"] = identity
    outcome["metrics"] = decision_metrics(report)
    outcome["physical_read_coverage"] = physical_read_coverage(
        report, scenario, population, recipe
    )
    outcome["validated"] = True
    write_json(cell_root / "outcome.json", outcome)
    remove_cgroup(cgroup_path)
    shutil.rmtree(data_path)
    outcome["successful_database_removed_after_evidence"] = True
    write_json(cell_root / "outcome.json", outcome)
    return outcome


def main():
    global ACTIVE_RUN_DIR
    parser = argparse.ArgumentParser()
    parser.add_argument("--recipe", type=Path, required=True)
    parser.add_argument("--binary-dir", type=Path, required=True)
    parser.add_argument("--workload-sha256", required=True)
    parser.add_argument("--worker-sha256", required=True)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--native-sha256", required=True)
    parser.add_argument("--zfs-path", type=Path, default=Path("/bench/zfs/db"))
    parser.add_argument("--run-dir", type=Path, required=True)
    parser.add_argument("--cgroup-base", type=Path, required=True)
    args = parser.parse_args()
    ACTIVE_RUN_DIR = args.run_dir
    if os.geteuid() != 0:
        raise RuntimeError("T17 runner requires root for the frozen ARC, page-cache, and cgroup controls")
    if sha256(args.recipe) != RECIPE_SHA256:
        raise RuntimeError("stress recipe hash does not match the frozen matrix")
    if args.native_sha256 != "9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5":
        raise RuntimeError("pinned RocksDB archive hash does not match the frozen matrix")
    if available_bytes(args.zfs_path) < MATRIX_FREE_BYTES:
        raise RuntimeError("47 GiB free-space gate failed before execution")
    if args.run_dir.exists() or args.cgroup_base.exists():
        raise RuntimeError("T17 run or cgroup directory already exists")
    args.run_dir.mkdir(parents=True)
    args.cgroup_base.mkdir()
    (args.cgroup_base / "cgroup.subtree_control").write_text("+memory")
    recipe = json.loads(args.recipe.read_text())
    common = recipe["common"]
    if common["service_memory_admission_bytes"] != APPLICATION_MEMORY_BYTES:
        raise RuntimeError("recipe service memory admission differs from the frozen 6 GiB budget")
    if common["service_memory_request_budget_bytes"] != 64 * 1024**2:
        raise RuntimeError("recipe request reservation differs from the fixed 64 MiB budget")
    if common["blink_checkpoint"]["threshold_bytes"] != 1024**2:
        raise RuntimeError("recipe checkpoint threshold differs from the frozen 1 MiB setting")
    binary = args.binary_dir / "tenant_density_workload"
    worker = args.binary_dir / "memory_diagnostic_worker"
    if sha256(binary) != args.workload_sha256:
        raise RuntimeError("workload binary hash does not match the frozen matrix")
    if sha256(worker) != args.worker_sha256:
        raise RuntimeError("memory diagnostic worker hash does not match the frozen matrix")
    original_arc_max = Path("/sys/module/zfs/parameters/zfs_arc_max").read_text().strip()
    try:
        outcomes = []
        expected_hashes = {}
        accepted_populations = []
        for population_index, population in enumerate(POPULATIONS):
            backends = BACKENDS[population_index % len(BACKENDS) :] + BACKENDS[
                : population_index % len(BACKENDS)
            ]
            screening = "synchronized-arrival"
            for backend in backends:
                Path("/sys/module/zfs/parameters/zfs_arc_max").write_text(
                    str(2 * 1024**3)
                )
                if available_bytes(args.zfs_path) < MATRIX_FREE_BYTES:
                    raise RuntimeError("47 GiB gate failed before a screening cell")
                outcome = run_cell(
                    args,
                    recipe,
                    binary,
                    args.run_dir,
                    args.cgroup_base,
                    screening,
                    population,
                    0,
                    backend,
                    expected_hashes,
                )
                outcomes.append(outcome)
                append_progress(args.run_dir, outcome, accepted_populations, len(outcomes))
            accepted_populations.append(population)
        for population_index, population in enumerate(accepted_populations):
            for scenario_id in [
                "synchronized-arrival",
                "staggered-arrival",
                "fixed-key-churn",
                "growing-distinct-keys-with-tombstones",
                "twice-decoded-cache-live-data",
                "long-reader-retention",
            ]:
                for repetition in range(1, 4):
                    order_index = (population_index + repetition - 1) % len(BACKENDS)
                    backends = BACKENDS[order_index:] + BACKENDS[:order_index]
                    for backend in backends:
                        Path("/sys/module/zfs/parameters/zfs_arc_max").write_text(
                            str(2 * 1024**3)
                        )
                        if available_bytes(args.zfs_path) < MATRIX_FREE_BYTES:
                            raise RuntimeError("47 GiB gate failed before a stress cell")
                        outcome = run_cell(
                            args,
                            recipe,
                            binary,
                            args.run_dir,
                            args.cgroup_base,
                            scenario_id,
                            population,
                            repetition,
                            backend,
                            expected_hashes,
                        )
                        outcomes.append(outcome)
                        append_progress(args.run_dir, outcome, accepted_populations, len(outcomes))
        write_json(
            args.run_dir / "summary.json",
            {"status": "complete", "accepted_populations": accepted_populations, "outcomes": outcomes},
        )
        write_json(
            args.run_dir / "progress.json",
            {
                "status": "complete",
                "accepted_populations": accepted_populations,
                "completed_cells": len(outcomes),
            },
        )
    finally:
        Path("/sys/module/zfs/parameters/zfs_arc_max").write_text(original_arc_max)
        if (args.cgroup_base / "cgroup.procs").read_text().strip():
            raise RuntimeError("T17 root cgroup still has processes")
        args.cgroup_base.rmdir()
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        if ACTIVE_RUN_DIR is not None and ACTIVE_RUN_DIR.is_dir():
            write_json(
                ACTIVE_RUN_DIR / "controller-failure.json",
                {"error": str(error), "timestamp_unix_seconds": time.time()},
            )
            progress_path = ACTIVE_RUN_DIR / "progress.json"
            progress = json.loads(progress_path.read_text()) if progress_path.exists() else {}
            progress["status"] = "stopped"
            progress["stop_reason"] = str(error)
            write_json(progress_path, progress)
        print(f"T17 stopped: {error}", file=sys.stderr, flush=True)
        raise
