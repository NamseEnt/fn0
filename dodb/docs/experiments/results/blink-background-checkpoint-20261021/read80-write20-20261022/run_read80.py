import importlib.util
import json
import os
import subprocess
from pathlib import Path


EXPECTED_SOURCE = "d240416f0e0efafa7c08bb1e868387ebd0d894bc"
EXPECTED_BINARY = "09bd505d77ce0bcbc61ca46f7389373e60bc1ab1bf6b80a90077bed328418e40"
ROOT = Path(os.environ["DODB_READ80_RUN_ROOT"])
SMOKE = os.environ.get("DODB_READ80_SMOKE", "0") == "1"
BASE_PATH = Path(os.environ.get(
    "DODB_READ80_BASE_RUNNER", str(Path(__file__).resolve().parent.parent / "run_matrix.py")
))
os.environ["DODB_CHECKPOINT_RUN_ROOT"] = str(ROOT)
os.environ["DODB_CHECKPOINT_SMOKE"] = "1" if SMOKE else "0"
os.environ["DODB_CHECKPOINT_MEASUREMENT_MS"] = "8000" if SMOKE else "30000"
os.environ["DODB_CHECKPOINT_WARMUP_MS"] = "1000" if SMOKE else "10000"
SPEC = importlib.util.spec_from_file_location("checkpoint_controller", BASE_PATH)
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def measured_events(report):
    window = report["measurement_window_unix_nanos"]
    start = int(window["start"])
    end = int(window["last_client_completion"])
    return [
        event for event in report["blink_checkpoint_metrics"].get("checkpoint_events", [])
        if event["succeeded"] and start <= int(event["started_unix_nanos"])
        and int(event["ended_unix_nanos"]) <= end
    ]


def validate_report(report, command, source_revision, binary_digest):
    if report.get("git_commit") != source_revision or report.get("binary_sha256") != binary_digest:
        raise RuntimeError("raw implementation or executable identity mismatch")
    configuration = report["configuration"]
    if report["scenario"] != "rw80" or report["seed"] != 20261013:
        raise RuntimeError("scenario or seed mismatch")
    if configuration["clients"] != 16 or configuration["client_executor"] != "dedicated":
        raise RuntimeError("caller count or execution model mismatch")
    if configuration["value_size_bytes"] != 512 or configuration["latency_storage"] != "log1024":
        raise RuntimeError("value size or latency collector mismatch")
    for tenant in report["per_tenant"].values():
        counters = tenant["counters"]
        if counters["overloads"] or counters["other_errors"]:
            raise RuntimeError("request errors or overloads are present")
        if abs(counters["read_fraction"] - 0.8) > 0.005:
            raise RuntimeError("observed API-call mix is not read-80/write-20")
    if "--blink-checkpoint-execution" not in command:
        return
    execution = command[command.index("--blink-checkpoint-execution") + 1]
    layout = command[command.index("--blink-wal-layout") + 1]
    effective = report["blink_checkpoint_configuration"]
    if effective["execution"] != execution or effective["layout"] != layout:
        raise RuntimeError("effective checkpoint execution or layout mismatch")
    metrics = report["blink_checkpoint_metrics"]
    if metrics["checkpoints_failed"] or metrics["failed_message"]:
        raise RuntimeError("checkpoint failure is present")
    if metrics["tasks_running"] or metrics["retained_image_bytes"] or metrics["omitted_checkpoint_events"]:
        raise RuntimeError("checkpoint drain or event accounting failed")
    events = measured_events(report)
    if execution == "background" and events and not any(event["writes_during_checkpoint"] for event in events):
        raise RuntimeError("completed measured background checkpoints have no acknowledged overlap")


def run():
    BASE.REPORTS.mkdir(parents=True, exist_ok=True)
    BASE.DATABASES.mkdir(parents=True, exist_ok=True)
    source_revision = subprocess.check_output(
        ["git", "-C", str(BASE.SOURCE), "rev-parse", "HEAD"], text=True
    ).strip()
    source_status = subprocess.check_output(
        ["git", "-C", str(BASE.SOURCE), "status", "--porcelain"], text=True
    )
    binary_digest = BASE.sha256_file(BASE.SERVICE_BINARY)
    if source_revision != EXPECTED_SOURCE or source_status or binary_digest != EXPECTED_BINARY:
        raise RuntimeError("frozen implementation checkout or executable verification failed")
    paths = list(BASE.POLICIES)
    if SMOKE:
        paths = ["minimal-native", "segmented-sync-128m", "segmented-background-128m"]
    orders = {
        repetition: paths[repetition - 1:] + paths[:repetition - 1]
        for repetition in range(1, 2 if SMOKE else 4)
    }
    plan = {
        "source_sha": source_revision,
        "controller_sha": os.environ["DODB_READ80_CONTROLLER_SHA"],
        "controller_sha256": BASE.sha256_file(Path(__file__)),
        "base_controller_sha256": BASE.sha256_file(BASE_PATH),
        "binary_hashes": {"service_workload": binary_digest},
        "path_orders": {str(repetition): order for repetition, order in orders.items()},
        "workloads": ["RW80-read80-write20"],
        "repetitions": len(orders),
        "cells": len(paths) * len(orders),
        "measurement_ms": BASE.MEASUREMENT_MS,
        "warmup_ms": BASE.WARMUP_MS,
        "smoke": SMOKE,
        "limits": {
            "max_process_group_rss_bytes": BASE.MAX_PROCESS_GROUP_RSS_BYTES,
            "minimum_mem_available_bytes": BASE.MINIMUM_MEM_AVAILABLE_BYTES,
            "maximum_database_bytes": BASE.MAXIMUM_DATABASE_BYTES,
            "minimum_filesystem_available_bytes": BASE.MINIMUM_DISK_AVAILABLE_BYTES,
        },
    }
    plan_path = ROOT / "matrix-plan.json"
    if plan_path.exists() and json.loads(plan_path.read_text()) != plan:
        raise RuntimeError("existing plan differs from this execution")
    write_json(plan_path, plan)
    write_json(ROOT / "binary-hashes.json", plan["binary_hashes"])
    completed_path = ROOT / "completed-cases.txt"
    completed = completed_path.read_text().splitlines() if completed_path.exists() else []
    prefix_path = ROOT / "input-prefix-fingerprints.json"
    prefixes = json.loads(prefix_path.read_text()) if prefix_path.exists() else []
    reference = prefixes[0]["signature"] if prefixes else None
    BASE.validate_checkpoint_report = validate_report
    for repetition, order in orders.items():
        for path_name in order:
            case_id = f"RW80-{path_name}-r{repetition}"
            if case_id in completed:
                continue
            backend, policy, layout, execution = BASE.POLICIES[path_name]
            database_path = BASE.DATABASES / case_id
            report_path = BASE.REPORTS / f"{case_id}.json"
            command = BASE.service_command(backend, database_path, report_path, policy)
            command[command.index("--scenario") + 1] = "rw80"
            if backend == "blink":
                command.extend(["--blink-wal-layout", layout, "--blink-checkpoint-execution", execution])
                if layout == "segmented":
                    command.extend(["--blink-wal-segment-bytes", "33554432"])
            print(json.dumps({"starting": case_id}), flush=True)
            BASE.wait_for_case(command, case_id, database_path, report_path, "RW80", backend, policy)
            report = json.loads(report_path.read_text())
            signature = BASE.input_prefix_signature(report, "A")
            if reference is None:
                reference = signature
            prefixes.append({"case_id": case_id, "signature": signature, "matches_first_case": signature == reference})
            write_json(prefix_path, prefixes)
            if signature != reference:
                raise RuntimeError(f"fixed input prefixes differ: {case_id}")
            completed.append(case_id)
            completed_path.write_text("\n".join(completed) + "\n")
            print(json.dumps({"completed": case_id, "api_calls_per_second": report["throughput_per_second"]["api_calls"]}), flush=True)
    (ROOT / "matrix-complete.txt").write_text(f"{len(completed)} read-80/write-20 cases completed and validated\n")


if __name__ == "__main__":
    run()
