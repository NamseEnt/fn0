import importlib.util
import json
import os
import subprocess
from pathlib import Path


ROOT = Path(os.environ["DODB_LOSS_RUN_ROOT"])
SMOKE = os.environ.get("DODB_LOSS_SMOKE", "0") == "1"
BASE_PATH = Path(os.environ["DODB_LOSS_BASE_RUNNER"])
os.environ["DODB_CHECKPOINT_RUN_ROOT"] = str(ROOT)
os.environ["DODB_CHECKPOINT_SMOKE"] = "1" if SMOKE else "0"
os.environ["DODB_CHECKPOINT_MEASUREMENT_MS"] = "8000" if SMOKE else "60000"
os.environ["DODB_CHECKPOINT_WARMUP_MS"] = "1000" if SMOKE else "10000"
SPEC = importlib.util.spec_from_file_location("checkpoint_controller", BASE_PATH)
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)
EXPECTED_SOURCE = "d240416f0e0efafa7c08bb1e868387ebd0d894bc"
EXPECTED_BINARIES = {
    "service_workload": "09bd505d77ce0bcbc61ca46f7389373e60bc1ab1bf6b80a90077bed328418e40",
    "unconditional_write_workload": "a697a8c4479835e7fa54aaef4acd16025770193289dbaf63e5ada4fbec48e9cc",
}
POLICIES = {
    "minimal-native": ("minimal-native", "off"),
    "legacy-off": ("blink", "off"),
    "legacy-sync-128m": ("blink", "134217728"),
    "legacy-sync-512m": ("blink", "536870912"),
}


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def events_in_window(report):
    window = report["measurement_window_unix_nanos"]
    return [event for event in report["blink_checkpoint_metrics"].get("checkpoint_events", [])
            if event["succeeded"] and int(window["start"]) <= int(event["started_unix_nanos"])
            and int(event["ended_unix_nanos"]) <= int(window["last_client_completion"])]


def validate_report(report, command, source_revision, binary_digest):
    assert report["git_commit"] == source_revision == EXPECTED_SOURCE
    assert report["binary_sha256"] == binary_digest
    if "per_tenant" in report:
        assert report["scenario"] == "rw80"
        assert report["configuration"]["clients"] == 16
        assert abs(report["counters"]["read_fraction"] - 0.8) <= 0.005
        counters = report["counters"]
    else:
        counters = report["counters"]
    assert not counters["overloads"] and not counters["other_errors"]
    metrics = report["blink_checkpoint_metrics"]
    if "--blink-wal-layout" not in command:
        return
    assert report["blink_checkpoint_configuration"]["layout"] == "legacy"
    assert report["blink_checkpoint_configuration"]["execution"] == "sync"
    assert not metrics["checkpoints_failed"] and not metrics["failed_message"]
    assert not metrics["tasks_running"] and not metrics["retained_image_bytes"]
    assert not metrics["omitted_checkpoint_events"]
    policy = command[command.index("--blink-checkpoint-wal-bytes") + 1]
    if policy == "off":
        assert not metrics["checkpoints_started"]
    elif not SMOKE:
        assert len(events_in_window(report)) >= 1


def run():
    BASE.REPORTS.mkdir(parents=True, exist_ok=True)
    BASE.DATABASES.mkdir(parents=True, exist_ok=True)
    revision = subprocess.check_output(["git", "-C", str(BASE.SOURCE), "rev-parse", "HEAD"], text=True).strip()
    status = subprocess.check_output(["git", "-C", str(BASE.SOURCE), "status", "--porcelain"], text=True)
    assert revision == EXPECTED_SOURCE and not status
    hashes = {name: BASE.sha256_file(BASE.TARGET / name) for name in EXPECTED_BINARIES}
    assert hashes == EXPECTED_BINARIES
    paths = ["legacy-off", "legacy-sync-512m"] if SMOKE else list(POLICIES)
    orders = {repetition: paths[repetition - 1:] + paths[:repetition - 1]
              for repetition in range(1, 2 if SMOKE else 4)}
    plan = {
        "source_sha": revision, "controller_sha": os.environ["DODB_LOSS_CONTROLLER_SHA"],
        "controller_sha256": BASE.sha256_file(Path(__file__)),
        "base_controller_sha256": BASE.sha256_file(BASE_PATH),
        "binary_hashes": hashes, "path_orders": {str(repetition): order for repetition, order in orders.items()},
        "workloads": ["RW80", "E"], "cells": len(orders) * len(paths) * 2,
        "measurement_ms": BASE.MEASUREMENT_MS, "warmup_ms": BASE.WARMUP_MS, "smoke": SMOKE,
        "limits": {"max_process_group_rss_bytes": BASE.MAX_PROCESS_GROUP_RSS_BYTES,
                   "minimum_mem_available_bytes": BASE.MINIMUM_MEM_AVAILABLE_BYTES,
                   "maximum_database_bytes": BASE.MAXIMUM_DATABASE_BYTES,
                   "minimum_filesystem_available_bytes": BASE.MINIMUM_DISK_AVAILABLE_BYTES},
    }
    plan_path = ROOT / "matrix-plan.json"
    assert not plan_path.exists() or json.loads(plan_path.read_text()) == plan
    write_json(plan_path, plan)
    write_json(ROOT / "binary-hashes.json", hashes)
    completed_path = ROOT / "completed-cases.txt"
    completed = completed_path.read_text().splitlines() if completed_path.exists() else []
    prefix_path = ROOT / "input-prefix-fingerprints.json"
    prefixes = json.loads(prefix_path.read_text()) if prefix_path.exists() else []
    references = {entry["workload"]: entry["signature"] for entry in prefixes}
    BASE.validate_checkpoint_report = validate_report
    for repetition, order in orders.items():
        for path_name in order:
            backend, policy = POLICIES[path_name]
            for workload in plan["workloads"]:
                case_id = f"{workload}-{path_name}-r{repetition}"
                if case_id in completed:
                    continue
                database_path = BASE.DATABASES / case_id
                report_path = BASE.REPORTS / f"{case_id}.json"
                if workload == "RW80":
                    command = BASE.service_command(backend, database_path, report_path, policy)
                    command[command.index("--scenario") + 1] = "rw80"
                else:
                    command = BASE.unconditional_command(backend, database_path, report_path, policy)
                if backend == "blink":
                    command.extend(["--blink-wal-layout", "legacy", "--blink-checkpoint-execution", "sync"])
                print(json.dumps({"starting": case_id}), flush=True)
                BASE.wait_for_case(command, case_id, database_path, report_path, workload, backend, policy)
                report = json.loads(report_path.read_text())
                signature = BASE.input_prefix_signature(report, "A" if workload == "RW80" else "E")
                references.setdefault(workload, signature)
                prefixes.append({"case_id": case_id, "workload": workload, "signature": signature,
                                 "matches_first_case": signature == references[workload]})
                write_json(prefix_path, prefixes)
                assert signature == references[workload], case_id
                completed.append(case_id)
                completed_path.write_text("\n".join(completed) + "\n")
                throughput = report["throughput_per_second"]["api_calls"] if workload == "RW80" else report["timing"]["write_batches_per_second"]
                print(json.dumps({"completed": case_id, "throughput_per_second": throughput}), flush=True)
    (ROOT / "matrix-complete.txt").write_text(f"{len(completed)} cases completed and validated\n")


if __name__ == "__main__":
    run()
