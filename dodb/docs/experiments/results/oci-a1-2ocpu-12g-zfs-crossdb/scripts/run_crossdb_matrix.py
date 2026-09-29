import json
import os
import pathlib
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone


RESULTS = pathlib.Path(sys.argv[1])
PHASE = sys.argv[2]
RAW = RESULTS / "raw"
RAW.mkdir(parents=True, exist_ok=True)
RUN_ORDER = RESULTS / "run-order.txt"
TURSO_BINARY = "/tmp/crossdb-turso-target/release/turso-bench"
ROCKSDB_BINARY = "/tmp/crossdb-rocksdb-target/release/rocksdb-bench"
DODB_WINDOW_BINARY = "/tmp/dodb-crossdb-window-target/release/phase0-bench"
DODB_WINDOW_CWD = "/tmp/dodb-crossdb-window"
DATA_ROOT = pathlib.Path("/bench/zfs/db")
BASE_SEED = 979_000_000
SEED_STRIDE = 1_009
DISTRIBUTIONS = ("uniform", "same-leaf-heavy", "different-leaf-heavy")
REPORT_NAMES = {
    "uniform": "uniform",
    "same-leaf-heavy": "compact-locality",
    "different-leaf-heavy": "spread-locality",
}
GROUP_COMMIT_OFF_SCENARIOS = (0, 3, 6, 9)

ENGINES = {
    "turso-wal": [TURSO_BINARY, "--journal", "wal"],
    "turso-mvcc-gc": [TURSO_BINARY, "--journal", "mvcc", "--group-commit", "on"],
    "turso-mvcc-nogc": [TURSO_BINARY, "--journal", "mvcc", "--group-commit", "off"],
    "rocksdb": [ROCKSDB_BINARY, "--pipelined", "off"],
    "rocksdb-pipelined": [ROCKSDB_BINARY, "--pipelined", "on"],
}


def utc_now():
    return datetime.now(timezone.utc).isoformat()


def rss_kib(process_id):
    try:
        with open(f"/proc/{process_id}/status", encoding="utf-8") as status_file:
            for status_line in status_file:
                if status_line.startswith("VmRSS:"):
                    return int(status_line.split()[1])
    except (FileNotFoundError, ProcessLookupError):
        return None
    return None


def directory_usage(path):
    apparent = 0
    allocated = 0
    for directory, _, files in os.walk(path):
        for file_name in files:
            try:
                status = os.stat(os.path.join(directory, file_name))
            except FileNotFoundError:
                continue
            apparent += status.st_size
            allocated += status.st_blocks * 512
    return apparent, allocated


def core_scenarios():
    scenario_list = []
    for writer_count in (16, 64):
        for transaction_width in (1, 16):
            for distribution in DISTRIBUTIONS:
                scenario_list.append((writer_count, transaction_width, distribution))
    for transaction_width in (1, 16):
        scenario_list.append((1, transaction_width, "uniform"))
    return scenario_list


def rotate(engine_names, offset):
    offset %= len(engine_names)
    return engine_names[offset:] + engine_names[:offset]


def log_order(line):
    with RUN_ORDER.open("a", encoding="utf-8") as order_file:
        order_file.write(json.dumps(line, sort_keys=True) + "\n")


def run_process(command, environment, log_path, monitor_path, data_dir, cwd=None):
    peak_rss = 0
    started = time.monotonic()
    next_monitor = started + 1.0
    monitor_file = monitor_path.open("w", encoding="utf-8") if monitor_path else None
    with log_path.open("w", encoding="utf-8") as log_file:
        process = subprocess.Popen(
            command,
            stdout=log_file,
            stderr=subprocess.STDOUT,
            env=environment,
            cwd=cwd,
        )
        while process.poll() is None:
            current = rss_kib(process.pid)
            if current is not None:
                peak_rss = max(peak_rss, current)
            now = time.monotonic()
            if monitor_file and now >= next_monitor:
                next_monitor += 1.0
                apparent, allocated = directory_usage(data_dir)
                monitor_file.write(json.dumps({
                    "t_since_start_s": now - started,
                    "rss_kib": current,
                    "disk_apparent_bytes": apparent,
                    "disk_allocated_bytes": allocated,
                }) + "\n")
            time.sleep(0.1)
        exit_code = process.wait()
    if monitor_file:
        monitor_file.close()
    return exit_code, peak_rss, time.monotonic() - started


def run_crossdb(engine_name, scenario_index, repetition_index, writer_count, transaction_width,
                distribution, working_set, warmup_ms, duration_ms, window_ms, prefix):
    seed = BASE_SEED + scenario_index * SEED_STRIDE + repetition_index
    scenario_name = f"w{writer_count}-width{transaction_width}-{REPORT_NAMES[distribution]}"
    result_name = f"{prefix}{engine_name}-{scenario_index:02d}-rep{repetition_index + 1}-{scenario_name}"
    output_path = RAW / f"{result_name}.jsonl"
    log_path = RAW / f"{result_name}.log"
    data_dir = DATA_ROOT / engine_name
    shutil.rmtree(data_dir, ignore_errors=True)
    command = ENGINES[engine_name] + [
        "--mode", "bench",
        "--writers", str(writer_count),
        "--width", str(transaction_width),
        "--distribution", distribution,
        "--working-set", str(working_set),
        "--key-size", "16",
        "--value-size", "64",
        "--warmup-ms", str(warmup_ms),
        "--duration-ms", str(duration_ms),
        "--window-ms", str(window_ms),
        "--monitor-ms", "1000",
        "--seed", str(seed),
        "--scenario-index", str(scenario_index),
        "--repetition", str(repetition_index + 1),
        "--data-dir", str(data_dir),
        "--output", str(output_path),
    ]
    start_line = {
        "event": "start",
        "phase": PHASE,
        "timestamp": utc_now(),
        "engine": engine_name,
        "scenario_index": scenario_index,
        "repetition": repetition_index + 1,
        "writers": writer_count,
        "width": transaction_width,
        "distribution": distribution,
        "working_set": working_set,
        "seed": seed,
        "command": command,
        "data_dir": str(data_dir),
    }
    log_order(start_line)
    monitor_path = RAW / f"{result_name}.process-monitor.jsonl" if prefix else None
    exit_code, peak_rss, elapsed = run_process(
        command, os.environ.copy(), log_path, monitor_path, data_dir
    )
    shutil.rmtree(data_dir, ignore_errors=True)
    complete_line = dict(start_line)
    complete_line.update({
        "event": "complete",
        "timestamp": utc_now(),
        "exit_code": exit_code,
        "rss_peak_sampled_kib": peak_rss,
        "elapsed_seconds_including_startup": elapsed,
        "output": str(output_path),
    })
    del complete_line["command"]
    log_order(complete_line)
    print(f"{result_name}: exit={exit_code} elapsed={elapsed:.1f}s", flush=True)


def run_dodb_sustained(scenario_index, writer_count, prefix):
    seed = BASE_SEED + scenario_index * SEED_STRIDE
    result_name = f"{prefix}dodb-planned-{scenario_index:02d}-rep1-w{writer_count}-width1-uniform"
    output_path = RAW / f"{result_name}.jsonl"
    log_path = RAW / f"{result_name}.log"
    data_dir = DATA_ROOT / "dodb-planned"
    shutil.rmtree(data_dir, ignore_errors=True)
    data_dir.mkdir(parents=True)
    command = [
        DODB_WINDOW_BINARY,
        "--suite", "write",
        "--writers", str(writer_count),
        "--widths", "1",
        "--distributions", "uniform",
        "--duration", "120s",
        "--warmup", "10s",
        "--repetitions", "1",
        "--cache-capacity", "256",
        "--working-set", "1000000",
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
    environment["TMPDIR"] = str(data_dir)
    start_line = {
        "event": "start",
        "phase": PHASE,
        "timestamp": utc_now(),
        "engine": "dodb-planned",
        "scenario_index": scenario_index,
        "repetition": 1,
        "writers": writer_count,
        "width": 1,
        "distribution": "uniform",
        "working_set": 1_000_000,
        "seed": seed,
        "command": command,
        "data_dir": str(data_dir),
    }
    log_order(start_line)
    exit_code, peak_rss, elapsed = run_process(
        command, environment, log_path, RAW / f"{result_name}.process-monitor.jsonl", data_dir,
        cwd=DODB_WINDOW_CWD,
    )
    shutil.rmtree(data_dir, ignore_errors=True)
    complete_line = dict(start_line)
    complete_line.update({
        "event": "complete",
        "timestamp": utc_now(),
        "exit_code": exit_code,
        "rss_peak_sampled_kib": peak_rss,
        "elapsed_seconds_including_startup": elapsed,
        "output": str(output_path),
    })
    del complete_line["command"]
    log_order(complete_line)
    print(f"{result_name}: exit={exit_code} elapsed={elapsed:.1f}s", flush=True)


def core_matrix():
    primary = ["turso-wal", "turso-mvcc-gc", "rocksdb"]
    for scenario_index, (writer_count, transaction_width, distribution) in enumerate(core_scenarios()):
        for repetition_index in range(3):
            engines = primary + ["rocksdb-pipelined"]
            if scenario_index in GROUP_COMMIT_OFF_SCENARIOS:
                engines.append("turso-mvcc-nogc")
            engines = rotate(engines, repetition_index + scenario_index)
            for engine_name in engines:
                run_crossdb(engine_name, scenario_index, repetition_index, writer_count,
                            transaction_width, distribution, 100_000, 2_000, 5_000, 0, "")


def sustained():
    engine_names = ["dodb-planned", "turso-mvcc-gc", "rocksdb", "turso-wal"]
    for rotation_index, (scenario_index, writer_count) in enumerate(((14, 16), (15, 64))):
        for engine_name in rotate(engine_names, rotation_index):
            if engine_name == "dodb-planned":
                run_dodb_sustained(scenario_index, writer_count, "sustained-")
            else:
                run_crossdb(engine_name, scenario_index, 0, writer_count, 1, "uniform",
                            1_000_000, 10_000, 120_000, 10_000, "sustained-")


if PHASE == "core":
    core_matrix()
elif PHASE == "sustained":
    sustained()
else:
    raise SystemExit(f"unknown phase {PHASE}")
