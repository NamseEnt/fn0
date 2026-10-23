import importlib.util
import json
import os
import struct
import subprocess
from pathlib import Path


ROOT = Path(os.environ["DODB_COST_RUN_ROOT"])
SMOKE = os.environ.get("DODB_COST_SMOKE", "0") == "1"
DURATION = int(os.environ.get("DODB_COST_DURATION_MS", "8000" if SMOKE else "60000"))
PREVIOUS = Path(os.environ.get("DODB_COST_PREVIOUS_ROOT", "/bench/zfs/db/blink-durable-bases-20261023-primary-budget"))
CANDIDATE = Path(os.environ["DODB_COST_CANDIDATE_ROOT"])
POLICIES = os.environ.get("DODB_COST_PATHS", "previous,candidate,minimal-native").split(",")
os.environ.update({"DODB_CHECKPOINT_RUN_ROOT": str(ROOT), "DODB_CHECKPOINT_SMOKE": "1" if SMOKE else "0",
                   "DODB_CHECKPOINT_MEASUREMENT_MS": str(DURATION), "DODB_CHECKPOINT_WARMUP_MS": "1000" if SMOKE else "10000"})
HELPER = Path(__file__).parent.parent / "blink-background-checkpoint-20261021/run_matrix.py"
SPEC = importlib.util.spec_from_file_location("checkpoint_controller", HELPER)
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)
BASE.CHILD_TIMEOUT_SECONDS = max(180, DURATION // 1000 + 180)
ORIGINAL_SAMPLE = BASE.process_sample
ORIGINAL_VALIDATE = BASE.validate_checkpoint_report


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def process_sample(process_group, database):
    sample = ORIGINAL_SAMPLE(process_group, database)
    base_bytes = 0
    for pack in database.rglob("*.pages"):
        try:
            base_bytes += pack.stat().st_size
        except FileNotFoundError:
            pass
    sample.update({"page_base_bytes": base_bytes, "recovery_file_bytes": base_bytes + sample["wal_bytes"]})
    return sample


def validate(report, command, source_sha, binary_sha):
    ORIGINAL_VALIDATE(report, command, source_sha, binary_sha)
    if "--blink-wal-layout" not in command:
        return
    window = report["measurement_window_unix_nanos"]
    events = [event for event in report["blink_checkpoint_metrics"]["checkpoint_events"]
              if event["succeeded"] and int(window["start"]) <= int(event["started_unix_nanos"])
              and int(event["ended_unix_nanos"]) <= int(window["last_client_completion"])]
    minimum = 0 if SMOKE else 10 if DURATION >= 300000 else 3
    assert len(events) >= minimum, len(events)
    database = Path(command[command.index("--data-dir") + 1])
    live_bytes = 0
    for manifest_path in database.rglob("*.segments.manifest"):
        data = manifest_path.read_bytes()
        assert data[:8] == b"DBLKSWM1" and struct.unpack_from("<I", data, 8)[0] == 3
        live_bytes += struct.unpack_from("<Q", data, 88)[0] * 4096 + 4
    case = Path(command[command.index("--report") + 1]).stem
    samples = json.loads((BASE.REPORTS / case / "resource-samples.json").read_text())
    envelope = live_bytes + 512 * 1024**2
    assert max(sample["recovery_file_bytes"] for sample in samples) <= envelope, case
    write_json(BASE.REPORTS / case / "recovery-envelope.json", {
        "manifest_checkpoint_image_bytes": live_bytes, "extra_bytes": 512 * 1024**2, "envelope_bytes": envelope})


def main():
    ROOT.mkdir(parents=True, exist_ok=True)
    BASE.REPORTS.mkdir(exist_ok=True)
    BASE.DATABASES.mkdir(exist_ok=True)
    identities = {}
    for policy in POLICIES:
        path_root = PREVIOUS if policy == "previous" else CANDIDATE
        source = path_root / "source"
        assert not subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip()
        identities[policy] = {"root": str(path_root), "source_sha": subprocess.check_output(
            ["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip(),
            "binaries": {name: BASE.sha256_file(path_root / "bin" / name)
                         for name in ["service_workload", "unconditional_write_workload"]}}
    orders = {repetition: POLICIES[repetition - 1:] + POLICIES[:repetition - 1]
              for repetition in range(1, 2 if SMOKE else 4)}
    plan = {"identities": identities, "orders": orders, "workloads": ["RW80", "E"], "smoke": SMOKE,
            "duration_ms": DURATION, "warmup_ms": BASE.WARMUP_MS, "controller_sha256": BASE.sha256_file(Path(__file__)),
            "helper_sha256": BASE.sha256_file(HELPER), "rss_limit_bytes": BASE.MAX_PROCESS_GROUP_RSS_BYTES,
            "disk_limit_bytes": BASE.MAXIMUM_DATABASE_BYTES, "minimum_memory_bytes": BASE.MINIMUM_MEM_AVAILABLE_BYTES,
            "minimum_disk_bytes": BASE.MINIMUM_DISK_AVAILABLE_BYTES, "extra_recovery_bytes": 512 * 1024**2}
    plan_path = ROOT / "matrix-plan.json"
    if plan_path.exists():
        assert json.loads(plan_path.read_text()) == json.loads(json.dumps(plan))
    write_json(plan_path, plan)
    completed_path = ROOT / "completed-cases.txt"
    completed = completed_path.read_text().splitlines() if completed_path.exists() else []
    prefix_path = ROOT / "input-prefix-fingerprints.json"
    prefixes = json.loads(prefix_path.read_text()) if prefix_path.exists() else []
    references = {entry["workload"]: entry["signature"] for entry in prefixes}
    BASE.process_sample = process_sample
    BASE.validate_checkpoint_report = validate
    for repetition, order in orders.items():
        for policy in order:
            path_root = Path(identities[policy]["root"])
            BASE.SOURCE = path_root / "source"
            BASE.SERVICE_BINARY = path_root / "bin/service_workload"
            BASE.UNCONDITIONAL_BINARY = path_root / "bin/unconditional_write_workload"
            backend = "minimal-native" if policy == "minimal-native" else "blink"
            for workload in ["RW80", "E"]:
                case = f"{workload}-{policy}-r{repetition}"
                if case in completed:
                    continue
                database, report_path = BASE.DATABASES / case, BASE.REPORTS / f"{case}.json"
                if workload == "RW80":
                    command = BASE.service_command(backend, database, report_path, "134217728")
                    command[command.index("--scenario") + 1] = "rw80"
                else:
                    command = BASE.unconditional_command(backend, database, report_path, "134217728")
                if backend == "blink":
                    command.extend(["--blink-wal-layout", "segmented-bases-coalesced", "--blink-checkpoint-execution",
                                    "background", "--blink-wal-segment-bytes", "33554432"])
                print(json.dumps({"starting": case}), flush=True)
                BASE.wait_for_case(command, case, database, report_path, "A" if workload == "RW80" else "E", backend, "134217728")
                report = json.loads(report_path.read_text())
                signature = BASE.input_prefix_signature(report, "A" if workload == "RW80" else "E")
                references.setdefault(workload, signature)
                assert references[workload] == signature, case
                prefixes.append({"case": case, "workload": workload, "signature": signature})
                write_json(prefix_path, prefixes)
                completed.append(case)
                completed_path.write_text("\n".join(completed) + "\n")
                throughput = report["throughput_per_second"]["api_calls"] if workload == "RW80" else report["timing"]["write_batches_per_second"]
                print(json.dumps({"completed": case, "throughput": throughput}), flush=True)
    (ROOT / "matrix-complete.txt").write_text(f"{len(completed)} accepted cells\n")


if __name__ == "__main__":
    main()
