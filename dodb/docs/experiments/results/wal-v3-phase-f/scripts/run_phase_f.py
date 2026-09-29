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
PHASE_ARGUMENTS = sys.argv[3:]
RAW = RESULTS / "raw"
RAW.mkdir(parents=True, exist_ok=True)
RUN_ORDER = RESULTS / "run-order.txt"
PROCESS_METRICS = RESULTS / "process-metrics.jsonl"
VARIANTS = {
    "phase-c": (pathlib.Path("/tmp/dodb-phase-c-target/release/phase0-bench"), None),
    "phase-d": (pathlib.Path("/tmp/dodb-phase-d-target/release/phase0-bench"), 2),
    "e0-churn": (pathlib.Path("/tmp/dodb-e-e0churn-target/release/phase0-bench"), 2),
    "phase-d-fp": (pathlib.Path("/tmp/dodb-e-d-fp-target/release/phase0-bench"), 2),
}
EXTRA_ARGUMENTS = {}
for extra_variant in json.loads(os.environ.get("PHASE_E_VARIANTS", "{}")).items():
    VARIANTS[extra_variant[0]] = (pathlib.Path(extra_variant[1][0]), extra_variant[1][1])
    EXTRA_ARGUMENTS[extra_variant[0]] = list(extra_variant[1][2]) if len(extra_variant[1]) > 2 else []
ROCKSDB_BINARY = pathlib.Path("/tmp/crossdb-rocksdb-target/release/rocksdb-bench")
ROCKSDB_SHA256 = "78f222f2214701a42b9f255b3b5044936e0dfb9be63bec1f0f01997e8ee9195a"
CROSSDB_NAMES = {
    "uniform": "uniform",
    "same-leaf-heavy": "compact-locality",
    "different-leaf-heavy": "spread-locality",
}
DATA_ROOT = pathlib.Path("/bench/zfs/db")
ENGINE = "planned-blink"
WORKING_SET = 100_000
BASE_SEED = 979_000_000
SEED_STRIDE = 1_009
LOW_MEMORY_KILL_KIB = 256 * 1024
SYNC_MODE = ["real"]
DISTRIBUTIONS = ("uniform", "same-leaf-heavy", "different-leaf-heavy")
REPETITIONS = 3
CPU_GATE_SCENARIOS = (9, 10, 11)
DURABLE_GATE_SCENARIOS = (0, 3, 6, 9, 10, 11)
PERF_SCENARIO = 9


def core_scenarios():
    scenario_list = []
    for writer_count in (16, 64):
        for transaction_width in (1, 16):
            for distribution in DISTRIBUTIONS:
                scenario_list.append((writer_count, transaction_width, distribution))
    for transaction_width in (1, 16):
        scenario_list.append((1, transaction_width, "uniform"))
    return scenario_list


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


def run_rocksdb_one(scenario_index, repetition_index):
    writer_count, transaction_width, distribution = core_scenarios()[scenario_index]
    seed = BASE_SEED + scenario_index * SEED_STRIDE + repetition_index
    result_name = (f"confirm-rocksdb-{scenario_index:02d}-rep{repetition_index + 1}-"
                   f"w{writer_count}-width{transaction_width}-{CROSSDB_NAMES[distribution]}")
    output_path = RAW / f"{result_name}.jsonl"
    log_path = RAW / f"{result_name}.log"
    data_dir = DATA_ROOT / "rocksdb-phase-f"
    shutil.rmtree(data_dir, ignore_errors=True)
    if sha256(ROCKSDB_BINARY) != ROCKSDB_SHA256:
        raise SystemExit("RocksDB benchmark binary changed")
    command = [
        str(ROCKSDB_BINARY), "--pipelined", "off",
        "--mode", "bench",
        "--writers", str(writer_count),
        "--width", str(transaction_width),
        "--distribution", distribution,
        "--working-set", str(WORKING_SET),
        "--key-size", "16",
        "--value-size", "64",
        "--warmup-ms", "2000",
        "--duration-ms", "5000",
        "--window-ms", "0",
        "--monitor-ms", "1000",
        "--seed", str(seed),
        "--scenario-index", str(scenario_index),
        "--repetition", str(repetition_index + 1),
        "--data-dir", str(data_dir),
        "--output", str(output_path),
    ]
    start_line = {
        "event": "start",
        "phase": "confirm",
        "timestamp": utc_now(),
        "variant": "rocksdb",
        "scenario_index": scenario_index,
        "repetition": repetition_index + 1,
        "writers": writer_count,
        "width": transaction_width,
        "distribution": distribution,
        "seed": seed,
        "binary": str(ROCKSDB_BINARY),
        "binary_sha256": ROCKSDB_SHA256,
        "command": command,
    }
    log_order(start_line)
    outcome = run_monitored(command, os.environ.copy(), DATA_ROOT, log_path, None, data_dir)
    shutil.rmtree(data_dir, ignore_errors=True)
    if outcome["exit_code"] != 0:
        raise SystemExit(f"{result_name} failed; see {log_path}")
    record = json.loads(output_path.read_text(encoding="utf-8").splitlines()[0])
    effective = record["settings"]["effective"]
    if (record["engine"] != "rocksdb" or record["measured"]["errors"] != 0
            or not effective["write_options_sync"] or effective["write_options_disable_wal"]
            or effective["enable_pipelined_write"] or not record["verification"]["passed"]):
        raise SystemExit(f"{output_path}: RocksDB row failed validation")
    with PROCESS_METRICS.open("a", encoding="utf-8") as metrics_file:
        metrics_file.write(json.dumps({
            "timestamp": utc_now(),
            "phase": "confirm",
            "variant": "rocksdb",
            "scenario_index": scenario_index,
            "repetition": repetition_index + 1,
            "jsonl": str(output_path),
            **outcome,
        }, sort_keys=True) + "\n")
    completion_line = dict(start_line)
    del completion_line["command"]
    completion_line.update({"event": "complete", "timestamp": utc_now(), **outcome})
    log_order(completion_line)
    print(f"{result_name}: tx/s={record['measured']['logical_tx_per_second']:.0f} "
          f"p99={record['measured']['p99_us']:.0f}us", flush=True)


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


def bench_command(binary, parallel_workers, writer_count, transaction_width, distribution, duration, warmup,
                  seed, output_path, extra_arguments=()):
    command = [
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
        "--working-set", str(WORKING_SET),
        "--key-size", "16",
        "--value-size", "64",
        "--group-limit", "64",
        "--group-bytes", "4194304",
        "--queue-capacity", "256",
        "--collection-delay", "0us",
        "--transaction-mode", "unconditional",
        "--tokio-workers", "2",
        "--sync-mode", SYNC_MODE[0],
        "--seed", str(seed),
        "--output", str(output_path),
        *extra_arguments,
    ]
    if parallel_workers is not None:
        command += ["--parallel-workers", str(parallel_workers)]
    return command


def validate_record(output_path, parallel_workers, transaction_width, writer_count):
    output_lines = output_path.read_text(encoding="utf-8").splitlines()
    if len(output_lines) != 1:
        raise SystemExit(f"expected exactly one JSONL row in {output_path}, found {len(output_lines)}")
    record = json.loads(output_lines[0])
    if record.get("engine") != ENGINE:
        raise SystemExit(f"{output_path}: engine is {record.get('engine')!r}, expected {ENGINE!r}")
    if record.get("sync_mode") != SYNC_MODE[0]:
        raise SystemExit(f"{output_path}: sync_mode is {record.get('sync_mode')!r}, expected {SYNC_MODE[0]!r}")
    if SYNC_MODE[0] == "real" and (record.get("wal_syncs_delta", 0) <= 0 or record.get("wal_sync_nanos_total", 0) <= 0):
        raise SystemExit(f"{output_path}: no positive WAL sync evidence")
    if record.get("wal_page_delta_records_delta", 0) <= 0:
        raise SystemExit(f"{output_path}: no page deltas")
    if record.get("errors", 0) != 0:
        raise SystemExit(f"{output_path}: benchmark reported errors")
    if parallel_workers is None:
        if "parallel_workers" in record:
            raise SystemExit(f"{output_path}: baseline binary unexpectedly reports parallel_workers")
    else:
        if record.get("parallel_workers") != parallel_workers:
            raise SystemExit(f"{output_path}: parallel_workers {record.get('parallel_workers')!r} != {parallel_workers}")
        multi_leaf = transaction_width > 1 or writer_count > 1
        if (parallel_workers > 0 and multi_leaf and record.get("parallel_groups_delta", 0) <= 0
                and record.get("parallel_skipped_small_group_delta", 0) <= 0):
            raise SystemExit(f"{output_path}: no parallel groups")
        if parallel_workers == 0 and record.get("parallel_groups_delta", 0) != 0:
            raise SystemExit(f"{output_path}: serial control ran parallel groups")
    return record


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def append_process_metrics(line):
    with PROCESS_METRICS.open("a", encoding="utf-8") as metrics_file:
        metrics_file.write(json.dumps(line, sort_keys=True) + "\n")


def run_core_one(phase, scenario_index, repetition_index, variant, duration="5s", warmup="2s"):
    writer_count, transaction_width, distribution = core_scenarios()[scenario_index]
    scenario_name = f"w{writer_count}-width{transaction_width}-{distribution}"
    seed = BASE_SEED + scenario_index * SEED_STRIDE + repetition_index
    result_name = f"{phase}-{SYNC_MODE[0]}-{scenario_index:02d}-rep{repetition_index + 1}-{variant}-{scenario_name}"
    output_path = RAW / f"{result_name}.jsonl"
    log_path = RAW / f"{result_name}.log"
    binary, parallel_workers = VARIANTS[variant]
    data_dir = DATA_ROOT / "dodb-phase-f-core"
    shutil.rmtree(data_dir, ignore_errors=True)
    data_dir.mkdir(parents=True)
    output_path.unlink(missing_ok=True)
    command = bench_command(binary, parallel_workers, writer_count, transaction_width, distribution, duration, warmup,
                            seed, output_path, EXTRA_ARGUMENTS.get(variant, []))
    environment = os.environ.copy()
    environment["TMPDIR"] = str(data_dir)
    start_line = {
        "event": "start", "phase": phase, "timestamp": utc_now(), "variant": variant,
        "scenario_index": scenario_index, "repetition": repetition_index + 1, "engine": ENGINE,
        "sync_mode": SYNC_MODE[0], "writers": writer_count, "width": transaction_width,
        "distribution": distribution, "seed": seed, "binary": str(binary), "binary_sha256": sha256(binary),
        "parallel_workers": parallel_workers, "tmpdir": str(data_dir), "command": command,
    }
    log_order(start_line)
    outcome = run_monitored(command, environment, pathlib.Path("/tmp"), log_path, None, data_dir)
    shutil.rmtree(data_dir, ignore_errors=True)
    if outcome["exit_code"] != 0:
        log_order({**start_line, "event": "failed", "timestamp": utc_now(), **outcome})
        raise SystemExit(f"{result_name} failed; see {log_path}")
    record = validate_record(output_path, parallel_workers, transaction_width, writer_count)
    append_process_metrics({
        "timestamp": utc_now(), "phase": phase, "sync_mode": SYNC_MODE[0], "variant": variant,
        "scenario_index": scenario_index, "repetition": repetition_index + 1, "jsonl": str(output_path), **outcome,
    })
    completion_line = dict(start_line)
    del completion_line["command"]
    completion_line.update({"event": "complete", "timestamp": utc_now(),
                            "successful_transactions": record.get("successful_transactions"), **outcome})
    log_order(completion_line)
    print(f"{result_name}: tx/s={record['logical_tx_per_second']:.0f} p99={record['e2e_p99_us']:.0f}us "
          f"par_groups={record.get('parallel_groups_delta')} fallback={record.get('parallel_fallback_groups_delta')}",
          flush=True)


def rotated(variants, repetition_index, scenario_index):
    shift = (repetition_index + scenario_index) % len(variants)
    return list(variants[shift:]) + list(variants[:shift])


def run_perf_one(variant):
    writer_count, transaction_width, distribution = core_scenarios()[PERF_SCENARIO]
    if SYNC_MODE[0] != "real":
        raise SystemExit("perf runs use real sync")
    seed = BASE_SEED + PERF_SCENARIO * SEED_STRIDE
    result_name = f"perf-real-{PERF_SCENARIO:02d}-{variant}-w{writer_count}-width{transaction_width}-{distribution}"
    output_path = RAW / f"{result_name}.jsonl"
    log_path = RAW / f"{result_name}.log"
    perf_path = RESULTS / "perf" / f"{result_name}.perf.data"
    perf_path.parent.mkdir(parents=True, exist_ok=True)
    binary, parallel_workers = VARIANTS[variant]
    data_dir = DATA_ROOT / "dodb-phase-f-perf"
    shutil.rmtree(data_dir, ignore_errors=True)
    data_dir.mkdir(parents=True)
    output_path.unlink(missing_ok=True)
    command = bench_command(binary, parallel_workers, writer_count, transaction_width, distribution, "20s", "2s",
                            seed, output_path, ["--window-seconds", "10", *EXTRA_ARGUMENTS.get(variant, [])])
    environment = os.environ.copy()
    environment["TMPDIR"] = str(data_dir)
    log_order({"event": "start", "phase": "perf", "timestamp": utc_now(), "variant": variant,
               "binary": str(binary), "binary_sha256": sha256(binary), "command": command})
    with log_path.open("w", encoding="utf-8") as log_file:
        process = subprocess.Popen(command, cwd="/tmp", env=environment, stdout=log_file, stderr=subprocess.STDOUT)
        while "label=measurement_start" not in log_path.read_text(encoding="utf-8"):
            if process.poll() is not None:
                raise SystemExit(f"{result_name} exited before measurement")
            time.sleep(0.1)
        time.sleep(1)
        perf = subprocess.run(["sudo", "perf", "record", "-F", "499", "-g", "-p", str(process.pid),
                               "-o", str(perf_path), "--", "sleep", "15"], capture_output=True, text=True)
        exit_code = process.wait()
    subprocess.run(["sudo", "chown", f"{os.getuid()}:{os.getgid()}", str(perf_path)], check=True)
    shutil.rmtree(data_dir, ignore_errors=True)
    (RESULTS / "perf" / f"{result_name}.record.log").write_text(perf.stdout + perf.stderr, encoding="utf-8")
    if exit_code != 0 or perf.returncode != 0:
        raise SystemExit(f"{result_name} failed: bench {exit_code}, perf {perf.returncode}")
    record = validate_record(output_path, parallel_workers, transaction_width, writer_count)
    for report_name, arguments in (
        ("flat", ["--no-children", "--sort", "symbol", "--percent-limit", "0.3"]),
        ("flat-all", ["--no-children", "--sort", "symbol", "--percent-limit", "0"]),
        ("children", ["--children", "--sort", "symbol", "--percent-limit", "1"]),
        ("dso", ["--no-children", "--sort", "dso"]),
        ("comm", ["--no-children", "--sort", "comm"]),
    ):
        report = subprocess.run(["perf", "report", "-i", str(perf_path), "--stdio"] + arguments,
                                capture_output=True, text=True)
        (RESULTS / "perf" / f"{result_name}.report-{report_name}.txt").write_text(
            report.stdout + report.stderr, encoding="utf-8")
    log_order({"event": "complete", "phase": "perf", "timestamp": utc_now(), "variant": variant,
               "exit_code": exit_code})
    print(f"{result_name}: tx/s={record['logical_tx_per_second']:.0f}", flush=True)


def main():
    check_data_root()
    if PHASE == "e0-counters":
        SYNC_MODE[0] = "disabled"
        for repetition_index in range(2):
            for scenario_index in (9, 6):
                run_core_one("counters", scenario_index, repetition_index, "e0-churn")
    elif PHASE == "probe":
        SYNC_MODE[0] = "disabled"
        for repetition_index in range(2):
            for scenario_index in (9, 10, 11):
                run_core_one("probe", scenario_index, repetition_index, PHASE_ARGUMENTS[0])
    elif PHASE == "counters":
        SYNC_MODE[0] = "disabled"
        variant = PHASE_ARGUMENTS[0]
        for repetition_index in range(2):
            for scenario_index in (9, 6):
                run_core_one("counters", scenario_index, repetition_index, variant)
    elif PHASE == "step":
        candidate = PHASE_ARGUMENTS[0]
        variants = tuple(PHASE_ARGUMENTS) if len(PHASE_ARGUMENTS) > 1 else (candidate, "phase-d")
        label = f"step{candidate}"
        for repetition_index in range(REPETITIONS):
            for sync_mode, scenario_index in (("disabled", 9), ("real", 9), ("real", 6)):
                SYNC_MODE[0] = sync_mode
                for variant in rotated(variants, repetition_index, scenario_index + (sync_mode == "real")):
                    run_core_one(label, scenario_index, repetition_index, variant)
    elif PHASE == "cpu-gate":
        SYNC_MODE[0] = "disabled"
        variants = tuple(PHASE_ARGUMENTS)
        for repetition_index in range(REPETITIONS):
            for scenario_index in CPU_GATE_SCENARIOS:
                for variant in rotated(variants, repetition_index, scenario_index):
                    run_core_one("cpu", scenario_index, repetition_index, variant)
    elif PHASE == "durable-gate":
        SYNC_MODE[0] = "real"
        variants = tuple(PHASE_ARGUMENTS)
        for repetition_index in range(REPETITIONS):
            for scenario_index in DURABLE_GATE_SCENARIOS:
                for variant in rotated(variants, repetition_index, scenario_index):
                    run_core_one("gate", scenario_index, repetition_index, variant)
    elif PHASE == "adaptive":
        SYNC_MODE[0] = "real"
        variants = tuple(PHASE_ARGUMENTS)
        for repetition_index in range(REPETITIONS):
            for scenario_index in (0, 6, 3, 9, 10):
                for variant in rotated(variants, repetition_index, scenario_index):
                    run_core_one("adaptive", scenario_index, repetition_index, variant)
    elif PHASE == "retention":
        SYNC_MODE[0] = "real"
        variants = tuple(PHASE_ARGUMENTS)
        for repetition_index in range(REPETITIONS):
            for scenario_index in DURABLE_GATE_SCENARIOS:
                for variant in rotated(variants, repetition_index, scenario_index):
                    run_core_one("retention", scenario_index, repetition_index, variant)
    elif PHASE == "matrix":
        SYNC_MODE[0] = "real"
        variants = tuple(PHASE_ARGUMENTS)
        for repetition_index in range(REPETITIONS):
            for scenario_index in range(len(core_scenarios())):
                for variant in rotated(variants, repetition_index, scenario_index):
                    run_core_one("matrix", scenario_index, repetition_index, variant)
    elif PHASE == "rocksdb":
        SYNC_MODE[0] = "real"
        variant = PHASE_ARGUMENTS[0]
        for repetition_index in range(REPETITIONS):
            for scenario_index in (6, 9):
                order = [variant, "rocksdb"]
                if (repetition_index + scenario_index) % 2 == 1:
                    order.reverse()
                for engine_name in order:
                    if engine_name == "rocksdb":
                        run_rocksdb_one(scenario_index, repetition_index)
                    else:
                        run_core_one("confirm", scenario_index, repetition_index, variant)
    elif PHASE == "perf":
        SYNC_MODE[0] = "real"
        for variant in PHASE_ARGUMENTS:
            run_perf_one(variant)
    else:
        raise SystemExit(f"unknown phase {PHASE}")


if __name__ == "__main__":
    main()
