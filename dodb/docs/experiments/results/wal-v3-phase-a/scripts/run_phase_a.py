import hashlib
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
PROCESS_METRICS = RESULTS / "process-metrics.jsonl"
BINARIES = {
    "old-v2": pathlib.Path("/tmp/dodb-zfs-exp-target/release/phase0-bench"),
    "old-v2-window": pathlib.Path("/tmp/dodb-crossdb-window-target/release/phase0-bench"),
    "retention-fixed-v2": pathlib.Path("/tmp/dodb-phase-a-target/release/phase0-bench"),
}
WORKING_DIRECTORIES = {
    "old-v2": pathlib.Path("/tmp/dodb-zfs-experiment"),
    "old-v2-window": pathlib.Path("/tmp/dodb-crossdb-window"),
    "retention-fixed-v2": pathlib.Path("/tmp/dodb-phase-a"),
}
DATA_ROOT = pathlib.Path("/bench/zfs/db")
ENGINE = "planned-blink"
WORKING_SET = 100_000
BASE_SEED = 979_000_000
SEED_STRIDE = 1_009
LOW_MEMORY_KILL_KIB = 256 * 1024
SHORT_SCENARIOS = ((0, 16), (6, 64))
REPETITIONS = 3


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


def mem_available_kib():
    with open("/proc/meminfo", encoding="utf-8") as meminfo_file:
        for meminfo_line in meminfo_file:
            if meminfo_line.startswith("MemAvailable:"):
                return int(meminfo_line.split()[1])
    return None


def file_sizes(data_dir):
    wal_bytes = 0
    data_bytes = 0
    allocated_bytes = 0
    for directory, _, files in os.walk(data_dir):
        for file_name in files:
            try:
                status = os.stat(os.path.join(directory, file_name))
            except FileNotFoundError:
                continue
            allocated_bytes += status.st_blocks * 512
            if file_name.endswith(".wal"):
                wal_bytes += status.st_size
            elif file_name.endswith(".db"):
                data_bytes += status.st_size
    return wal_bytes, data_bytes, allocated_bytes


def log_order(line):
    with RUN_ORDER.open("a", encoding="utf-8") as order_file:
        order_file.write(json.dumps(line, sort_keys=True) + "\n")


def check_data_root():
    mount_result = subprocess.run(
        ["findmnt", "-n", "-T", str(DATA_ROOT), "-o", "FSTYPE,SOURCE"],
        check=True,
        capture_output=True,
        text=True,
    )
    if mount_result.stdout.split() != ["zfs", "dodbbench/db"]:
        raise SystemExit(f"database root is not the expected ZFS dataset: {mount_result.stdout.strip()!r}")


def bench_command(binary, writer_count, transaction_width, distribution, working_set, duration, warmup,
                  seed, output_path, extra_arguments=()):
    return [
        str(binary),
        "--engine", ENGINE,
        "--suite", "write",
        "--writers", str(writer_count),
        "--widths", str(transaction_width),
        "--distributions", distribution,
        "--duration", duration,
        "--warmup", warmup,
        "--repetitions", "1",
        "--cache-capacity", "256",
        "--working-set", str(working_set),
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
        *extra_arguments,
    ]


def run_monitored(command, environment, working_directory, log_path, monitor_path, data_dir):
    start_rss = None
    last_rss = None
    peak_rss = 0
    min_available = None
    killed_low_memory = False
    started = time.monotonic()
    next_monitor = started
    monitor_file = monitor_path.open("w", encoding="utf-8") if monitor_path else None
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
            available = mem_available_kib()
            if available is not None:
                min_available = available if min_available is None else min(min_available, available)
                if available < LOW_MEMORY_KILL_KIB:
                    killed_low_memory = True
                    process.kill()
            now = time.monotonic()
            if monitor_file and now >= next_monitor:
                next_monitor += 1.0
                wal_bytes, data_bytes, allocated_bytes = file_sizes(data_dir)
                monitor_file.write(json.dumps({
                    "unix_ms": int(time.time() * 1000),
                    "t_since_start_s": now - started,
                    "rss_kib": current_rss,
                    "mem_available_kib": available,
                    "wal_bytes": wal_bytes,
                    "data_bytes": data_bytes,
                    "allocated_bytes": allocated_bytes,
                }) + "\n")
                monitor_file.flush()
            time.sleep(0.05)
        exit_code = process.wait()
    if monitor_file:
        monitor_file.close()
    return {
        "exit_code": exit_code,
        "rss_start_kib": start_rss,
        "rss_peak_sampled_kib": peak_rss or None,
        "rss_last_sampled_kib": last_rss,
        "mem_available_min_kib": min_available,
        "killed_low_memory": killed_low_memory,
        "elapsed_seconds_including_startup": time.monotonic() - started,
    }


def validate_record(output_path):
    output_lines = output_path.read_text(encoding="utf-8").splitlines()
    if len(output_lines) != 1:
        raise SystemExit(f"expected exactly one JSONL row in {output_path}, found {len(output_lines)}")
    record = json.loads(output_lines[0])
    if record.get("engine") != ENGINE:
        raise SystemExit(f"{output_path}: engine is {record.get('engine')!r}, expected {ENGINE!r}")
    if record.get("sync_mode") != "real":
        raise SystemExit(f"{output_path}: sync_mode is {record.get('sync_mode')!r}, expected 'real'")
    if record.get("wal_syncs_delta", 0) <= 0 or record.get("wal_sync_nanos_total", 0) <= 0:
        raise SystemExit(f"{output_path}: no positive WAL sync evidence")
    if record.get("superblock_images_emitted", 0) + record.get("superblock_images_elided", 0) <= 0:
        raise SystemExit(f"{output_path}: no Blink planner superblock counters; engine path not exercised")
    return record


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_short_one(scenario_index, writer_count, repetition_index, variant):
    seed = BASE_SEED + scenario_index * SEED_STRIDE + repetition_index
    result_name = f"short-{scenario_index:02d}-rep{repetition_index + 1}-{variant}-w{writer_count}-width1-uniform"
    output_path = RAW / f"{result_name}.jsonl"
    log_path = RAW / f"{result_name}.log"
    binary = BINARIES[variant]
    data_dir = DATA_ROOT / "dodb-phase-a-short"
    shutil.rmtree(data_dir, ignore_errors=True)
    data_dir.mkdir(parents=True)
    command = bench_command(binary, writer_count, 1, "uniform", WORKING_SET, "5s", "2s", seed, output_path)
    environment = os.environ.copy()
    environment["TMPDIR"] = str(data_dir)
    start_line = {
        "event": "start",
        "phase": "short",
        "timestamp": utc_now(),
        "variant": variant,
        "scenario_index": scenario_index,
        "repetition": repetition_index + 1,
        "engine": ENGINE,
        "writers": writer_count,
        "width": 1,
        "distribution": "uniform",
        "seed": seed,
        "binary": str(binary),
        "binary_sha256": sha256(binary),
        "cwd": str(WORKING_DIRECTORIES[variant]),
        "tmpdir": str(data_dir),
        "command": command,
    }
    log_order(start_line)
    outcome = run_monitored(command, environment, WORKING_DIRECTORIES[variant], log_path, None, data_dir)
    shutil.rmtree(data_dir, ignore_errors=True)
    if outcome["exit_code"] != 0:
        log_order({**start_line, "event": "failed", "timestamp": utc_now(), **outcome})
        raise SystemExit(f"{result_name} failed; see {log_path}")
    record = validate_record(output_path)
    with PROCESS_METRICS.open("a", encoding="utf-8") as metrics_file:
        metrics_file.write(json.dumps({
            "timestamp": utc_now(),
            "phase": "short",
            "variant": variant,
            "scenario_index": scenario_index,
            "repetition": repetition_index + 1,
            "writers": writer_count,
            "seed": seed,
            "jsonl": str(output_path),
            **outcome,
        }, sort_keys=True) + "\n")
    completion_line = dict(start_line)
    del completion_line["command"]
    completion_line.update({
        "event": "complete",
        "timestamp": utc_now(),
        "successful_transactions": record.get("successful_transactions"),
        "errors": record.get("errors"),
        "overloads": record.get("overloads"),
        "wal_bytes_per_tx": record["wal_bytes_delta"] / max(record["successful_transactions"], 1),
        **outcome,
    })
    log_order(completion_line)
    print(f"{result_name}: tx/s={record['logical_tx_per_second']:.0f} "
          f"p99={record['e2e_p99_us']:.0f}us "
          f"wal_b_per_tx={record['wal_bytes_delta'] / max(record['successful_transactions'], 1):.1f}",
          flush=True)


def run_sustained_one(scenario_index, writer_count, variant):
    seed = BASE_SEED + scenario_index * SEED_STRIDE
    result_name = f"sustained-{variant}-{scenario_index:02d}-rep1-w{writer_count}-width1-uniform"
    output_path = RAW / f"{result_name}.jsonl"
    log_path = RAW / f"{result_name}.log"
    monitor_path = RAW / f"{result_name}.process-monitor.jsonl"
    binary = BINARIES[variant]
    data_dir = DATA_ROOT / "dodb-phase-a-sustained"
    shutil.rmtree(data_dir, ignore_errors=True)
    data_dir.mkdir(parents=True)
    extra_arguments = ("--window-seconds", "10") if variant == "retention-fixed-v2" else ()
    command = bench_command(binary, writer_count, 1, "uniform", 1_000_000, "120s", "10s", seed, output_path,
                            extra_arguments)
    environment = os.environ.copy()
    environment["TMPDIR"] = str(data_dir)
    start_line = {
        "event": "start",
        "phase": "sustained",
        "timestamp": utc_now(),
        "variant": variant,
        "scenario_index": scenario_index,
        "repetition": 1,
        "engine": ENGINE,
        "writers": writer_count,
        "width": 1,
        "distribution": "uniform",
        "working_set": 1_000_000,
        "seed": seed,
        "binary": str(binary),
        "binary_sha256": sha256(binary),
        "cwd": str(WORKING_DIRECTORIES[variant]),
        "tmpdir": str(data_dir),
        "command": command,
    }
    log_order(start_line)
    outcome = run_monitored(command, environment, WORKING_DIRECTORIES[variant], log_path, monitor_path, data_dir)
    shutil.rmtree(data_dir, ignore_errors=True)
    completion_line = dict(start_line)
    del completion_line["command"]
    completion_line.update({"event": "complete", "timestamp": utc_now(), **outcome})
    if outcome["exit_code"] == 0:
        record = validate_record(output_path)
        completion_line["successful_transactions"] = record.get("successful_transactions")
        completion_line["errors"] = record.get("errors")
    log_order(completion_line)
    print(f"{result_name}: exit={outcome['exit_code']} killed_low_memory={outcome['killed_low_memory']}",
          flush=True)


def main():
    check_data_root()
    if PHASE == "short":
        for repetition_index in range(REPETITIONS):
            variants = ("old-v2", "retention-fixed-v2")
            if repetition_index % 2 == 1:
                variants = tuple(reversed(variants))
            for scenario_index, writer_count in SHORT_SCENARIOS:
                for variant in variants:
                    run_short_one(scenario_index, writer_count, repetition_index, variant)
    elif PHASE == "sustained":
        for scenario_index, writer_count in ((14, 16), (15, 64)):
            run_sustained_one(scenario_index, writer_count, "retention-fixed-v2")
    elif PHASE == "sustained-control":
        run_sustained_one(14, 16, "old-v2-window")
    else:
        raise SystemExit(f"unknown phase {PHASE}")


if __name__ == "__main__":
    main()
