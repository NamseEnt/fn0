import json
import os
import pathlib
import subprocess
import sys
import time
from datetime import datetime, timezone


RESULTS = pathlib.Path(sys.argv[1])
RAW = RESULTS / "raw"
RAW.mkdir(parents=True, exist_ok=True)
RUN_ORDER = RESULTS / "run-order.txt"
PROCESS_METRICS = RESULTS / "process-metrics.jsonl"
EXPERIMENT_BINARY = pathlib.Path("/tmp/dodb-zfs-exp-target/release/phase0-bench")
MAIN_BINARY = pathlib.Path("/tmp/dodb-zfs-main-target/release/phase0-bench")
EXPERIMENT_CWD = pathlib.Path("/tmp/dodb-zfs-experiment")
MAIN_CWD = pathlib.Path("/tmp/dodb-zfs-main")
DATA_ROOT = pathlib.Path("/bench/zfs/db")
WORKING_SET = 100_000
BASE_SEED = 979_000_000
SEED_STRIDE = 1_009
DISTRIBUTIONS = ("uniform", "same-leaf-heavy", "different-leaf-heavy")


def utc_now():
    return datetime.now(timezone.utc).isoformat()


def rss_kib(process_id):
    try:
        with open(f"/proc/{process_id}/status", encoding="utf-8") as status_file:
            for status_line in status_file:
                if status_line.startswith("VmRSS:"):
                    return int(status_line.split()[1])
    except FileNotFoundError:
        return None
    return None


def scenarios():
    scenario_list = []
    for writer_count in (16, 64):
        for transaction_width in (1, 16):
            for distribution in DISTRIBUTIONS:
                scenario_list.append((writer_count, transaction_width, distribution))
    for transaction_width in (1, 16):
        scenario_list.append((1, transaction_width, "uniform"))
    return scenario_list


def engine_order(repetition_index):
    if repetition_index % 2 == 0:
        return (
            ("ExactMain", MAIN_BINARY, MAIN_CWD),
            ("Planned", EXPERIMENT_BINARY, EXPERIMENT_CWD),
        )
    return (
        ("Planned", EXPERIMENT_BINARY, EXPERIMENT_CWD),
        ("ExactMain", MAIN_BINARY, MAIN_CWD),
    )


def run_one(scenario_index, repetition_index, scenario):
    writer_count, transaction_width, distribution = scenario
    scenario_name = f"w{writer_count}-width{transaction_width}-{distribution}"
    seed = BASE_SEED + scenario_index * SEED_STRIDE + repetition_index
    for engine_name, binary_path, working_directory in engine_order(repetition_index):
        result_name = f"{scenario_index:02d}-rep{repetition_index + 1}-{engine_name.lower()}-{scenario_name}"
        output_path = RAW / f"{result_name}.jsonl"
        log_path = RAW / f"{result_name}.log"
        command = [
            str(binary_path),
            "--suite", "write",
            "--writers", str(writer_count),
            "--widths", str(transaction_width),
            "--distributions", distribution,
            "--duration", "5s",
            "--warmup", "2s",
            "--repetitions", "1",
            "--cache-capacity", "256",
            "--working-set", str(WORKING_SET),
            "--key-size", "16",
            "--value-size", "64",
            "--group-limit", "64",
            "--group-bytes", "4194304",
            "--queue-capacity", "256",
            "--collection-delay", "0us",
            "--transaction-mode", "unconditional",
            "--tokio-workers", "2",
            "--sync-mode", "real",
            "--seed", str(seed),
            "--output", str(output_path),
        ]
        environment = os.environ.copy()
        environment["TMPDIR"] = str(DATA_ROOT)
        start_line = {
            "event": "start",
            "timestamp": utc_now(),
            "scenario_index": scenario_index,
            "repetition": repetition_index + 1,
            "engine": engine_name,
            "writers": writer_count,
            "width": transaction_width,
            "distribution": distribution,
            "seed": seed,
            "binary": str(binary_path),
            "output": str(output_path),
            "cwd": str(working_directory),
            "tmpdir": str(DATA_ROOT),
        }
        with RUN_ORDER.open("a", encoding="utf-8") as order_file:
            order_file.write(json.dumps(start_line, sort_keys=True) + "\n")
        start_rss = None
        peak_rss = 0
        last_rss = None
        process_started = time.monotonic()
        with log_path.open("w", encoding="utf-8") as log_file:
            process = subprocess.Popen(
                command,
                cwd=working_directory,
                env=environment,
                stdout=log_file,
                stderr=subprocess.STDOUT,
            )
            while process.poll() is None:
                current_rss = rss_kib(process.pid)
                if current_rss is not None:
                    if start_rss is None:
                        start_rss = current_rss
                    last_rss = current_rss
                    peak_rss = max(peak_rss, current_rss)
                time.sleep(0.05)
            exit_code = process.returncode
        process_seconds = time.monotonic() - process_started
        if exit_code != 0:
            with RUN_ORDER.open("a", encoding="utf-8") as order_file:
                order_file.write(json.dumps({**start_line, "event": "failed", "exit_code": exit_code, "timestamp": utc_now()}, sort_keys=True) + "\n")
            raise SystemExit(f"{engine_name} failed for {scenario_name}, repetition {repetition_index + 1}; see {log_path}")
        output_lines = output_path.read_text(encoding="utf-8").splitlines()
        if len(output_lines) != 1:
            raise SystemExit(f"expected exactly one JSONL row in {output_path}, found {len(output_lines)}")
        record = json.loads(output_lines[0])
        required_sync_fields = ("sync_mode", "wal_syncs_delta", "wal_sync_nanos_total")
        if any(field_name not in record for field_name in required_sync_fields):
            raise SystemExit(f"missing real-sync fields in {output_path}")
        if record["sync_mode"] != "real" or record["wal_syncs_delta"] <= 0 or record["wal_sync_nanos_total"] <= 0:
            raise SystemExit(f"invalid real-sync evidence in {output_path}")
        process_metrics = {
            "timestamp": utc_now(),
            "scenario_index": scenario_index,
            "repetition": repetition_index + 1,
            "engine": engine_name,
            "writers": writer_count,
            "width": transaction_width,
            "distribution": distribution,
            "seed": seed,
            "elapsed_seconds_including_startup": process_seconds,
            "rss_start_kib": start_rss,
            "rss_peak_sampled_kib": peak_rss or None,
            "rss_last_sampled_kib": last_rss,
            "jsonl": str(output_path),
        }
        with PROCESS_METRICS.open("a", encoding="utf-8") as metrics_file:
            metrics_file.write(json.dumps(process_metrics, sort_keys=True) + "\n")
        completion_line = {
            **start_line,
            "event": "complete",
            "timestamp": utc_now(),
            "elapsed_seconds_including_startup": process_seconds,
            "successful_transactions": record.get("successful_transactions"),
            "errors": record.get("errors"),
            "overloads": record.get("overloads"),
            "wal_syncs": record["wal_syncs_delta"],
            "transactions_per_sync": record.get("transactions_per_sync"),
            "rss_peak_sampled_kib": peak_rss or None,
        }
        with RUN_ORDER.open("a", encoding="utf-8") as order_file:
            order_file.write(json.dumps(completion_line, sort_keys=True) + "\n")


def main():
    mount_result = subprocess.run(
        ["findmnt", "-n", "-T", str(DATA_ROOT), "-o", "FSTYPE,SOURCE"],
        check=True,
        capture_output=True,
        text=True,
    )
    mount_fields = mount_result.stdout.split()
    if mount_fields != ["zfs", "dodbbench/db"]:
        raise SystemExit(f"database root is not the expected ZFS dataset: {mount_result.stdout.strip()!r}")
    for scenario_index, scenario in enumerate(scenarios()):
        for repetition_index in range(3):
            run_one(scenario_index, repetition_index, scenario)
    print(f"completed {len(scenarios())} scenarios x 3 repetitions x 2 engines")


if __name__ == "__main__":
    main()
