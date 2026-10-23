import importlib.util
import json
import os
import subprocess
from pathlib import Path


ROOT = Path(os.environ["DODB_BASE_RUN_ROOT"])
SMOKE = os.environ.get("DODB_BASE_SMOKE", "0") == "1"
DURATION = int(os.environ.get("DODB_BASE_MEASUREMENT_MS", "8000" if SMOKE else "60000"))
EXPECTED_SOURCE = os.environ["DODB_BASE_SOURCE_SHA"]
os.environ["DODB_CHECKPOINT_RUN_ROOT"] = str(ROOT)
os.environ["DODB_CHECKPOINT_SMOKE"] = "1" if SMOKE else "0"
os.environ["DODB_CHECKPOINT_MEASUREMENT_MS"] = str(DURATION)
os.environ["DODB_CHECKPOINT_WARMUP_MS"] = "1000" if SMOKE else "10000"
BASE_PATH = Path(__file__).parent.parent / "blink-background-checkpoint-20261021" / "run_matrix.py"
SPEC = importlib.util.spec_from_file_location("checkpoint_controller", BASE_PATH)
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)
BASE.CHILD_TIMEOUT_SECONDS = max(180, DURATION // 1000 + 180)
POLICIES = {
    "segmented-sync": ("segmented", "sync"),
    "bases-sync": ("segmented-bases", "sync"),
    "coalesced-sync": ("segmented-bases-coalesced", "sync"),
    "coalesced-background": ("segmented-bases-coalesced", "background"),
}
PATH_NAMES = os.environ.get("DODB_BASE_PATHS", ",".join(POLICIES)).split(",")
MIN_CHECKPOINTS = 0 if SMOKE else 10 if DURATION >= 300000 else 3
ORIGINAL_SAMPLE = BASE.process_sample
ORIGINAL_VALIDATE = BASE.validate_checkpoint_report


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def process_sample(process_group, database_path):
    sample = ORIGINAL_SAMPLE(process_group, database_path)
    base_bytes = 0
    for path in database_path.rglob("*.pages"):
        try:
            base_bytes += path.stat().st_size
        except FileNotFoundError:
            pass
    sample["page_base_bytes"] = base_bytes
    sample["recovery_file_bytes"] = sample["wal_bytes"] + base_bytes
    return sample


def validate(report, command, source_revision, binary_digest):
    ORIGINAL_VALIDATE(report, command, source_revision, binary_digest)
    assert report["git_commit"] == source_revision == EXPECTED_SOURCE
    configuration = report["blink_checkpoint_configuration"]
    layout = command[command.index("--blink-wal-layout") + 1]
    assert configuration["layout"] == layout
    counters = report["process_write_io"]
    assert counters["before"]["available"] and counters["after"]["available"]
    assert counters["after"]["wchar"] > counters["before"]["wchar"]
    window = report["measurement_window_unix_nanos"]
    events = [event for event in report["blink_checkpoint_metrics"]["checkpoint_events"]
              if event["succeeded"] and int(window["start"]) <= int(event["started_unix_nanos"])
              and int(event["ended_unix_nanos"]) <= int(window["last_client_completion"])]
    assert len(events) >= MIN_CHECKPOINTS, len(events)
    case = Path(command[command.index("--report") + 1]).stem
    samples = json.loads((BASE.REPORTS / case / "resource-samples.json").read_text())
    assert max(sample["recovery_file_bytes"] for sample in samples) <= 512 * 1024**2
    if "per_tenant" in report:
        assert report["scenario"] == "rw80" and report["configuration"]["clients"] == 16


def run():
    BASE.REPORTS.mkdir(parents=True, exist_ok=True)
    BASE.DATABASES.mkdir(parents=True, exist_ok=True)
    revision = subprocess.check_output(["git", "-C", str(BASE.SOURCE), "rev-parse", "HEAD"], text=True).strip()
    assert revision == EXPECTED_SOURCE
    assert not subprocess.check_output(["git", "-C", str(BASE.SOURCE), "status", "--porcelain"], text=True).strip()
    hashes = {name: BASE.sha256_file(BASE.TARGET / name) for name in ["service_workload", "unconditional_write_workload"]}
    orders = {repetition: PATH_NAMES[repetition - 1:] + PATH_NAMES[:repetition - 1]
              for repetition in range(1, 2 if SMOKE else 4)}
    plan = {"source_sha": revision, "binary_hashes": hashes, "orders": orders,
            "workloads": ["RW80", "E"], "policies": POLICIES,
            "duration_ms": DURATION, "warmup_ms": BASE.WARMUP_MS,
            "minimum_checkpoints": MIN_CHECKPOINTS, "smoke": SMOKE,
            "controller_sha256": BASE.sha256_file(Path(__file__)),
            "base_controller_sha256": BASE.sha256_file(BASE_PATH),
            "limits": {"rss_bytes": BASE.MAX_PROCESS_GROUP_RSS_BYTES,
                       "recovery_file_bytes": 512 * 1024**2,
                       "database_bytes": BASE.MAXIMUM_DATABASE_BYTES,
                       "minimum_memory_bytes": BASE.MINIMUM_MEM_AVAILABLE_BYTES,
                       "minimum_free_disk_bytes": BASE.MINIMUM_DISK_AVAILABLE_BYTES}}
    plan_path = ROOT / "matrix-plan.json"
    if plan_path.exists():
        assert json.loads(plan_path.read_text()) == json.loads(json.dumps(plan))
    write_json(plan_path, plan)
    write_json(ROOT / "binary-hashes.json", hashes)
    completed_path = ROOT / "completed-cases.txt"
    completed = completed_path.read_text().splitlines() if completed_path.exists() else []
    prefix_path = ROOT / "input-prefix-fingerprints.json"
    prefixes = json.loads(prefix_path.read_text()) if prefix_path.exists() else []
    references = {entry["workload"]: entry["signature"] for entry in prefixes}
    BASE.process_sample = process_sample
    BASE.validate_checkpoint_report = validate
    for repetition, order in orders.items():
        for policy in order:
            layout, execution = POLICIES[policy]
            for workload in ["RW80", "E"]:
                case = f"{workload}-{policy}-r{repetition}"
                if case in completed:
                    continue
                database = BASE.DATABASES / case
                report_path = BASE.REPORTS / f"{case}.json"
                if workload == "RW80":
                    command = BASE.service_command("blink", database, report_path, "134217728")
                    command[command.index("--scenario") + 1] = "rw80"
                else:
                    command = BASE.unconditional_command("blink", database, report_path, "134217728")
                command.extend(["--blink-wal-layout", layout, "--blink-checkpoint-execution", execution,
                                "--blink-wal-segment-bytes", "33554432"])
                print(json.dumps({"starting": case}), flush=True)
                BASE.wait_for_case(command, case, database, report_path, workload, "blink", "134217728")
                report = json.loads(report_path.read_text())
                signature = BASE.input_prefix_signature(report, "A" if workload == "RW80" else "E")
                references.setdefault(workload, signature)
                assert signature == references[workload], case
                prefixes.append({"case": case, "workload": workload, "signature": signature})
                write_json(prefix_path, prefixes)
                completed.append(case)
                completed_path.write_text("\n".join(completed) + "\n")
                throughput = report["throughput_per_second"]["api_calls"] if workload == "RW80" else report["timing"]["write_batches_per_second"]
                print(json.dumps({"completed": case, "throughput": throughput}), flush=True)
    (ROOT / "matrix-complete.txt").write_text(f"{len(completed)} accepted cells\n")


if __name__ == "__main__":
    run()
