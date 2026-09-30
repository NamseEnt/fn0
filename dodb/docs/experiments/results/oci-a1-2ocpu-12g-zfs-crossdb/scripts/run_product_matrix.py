import argparse
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
from datetime import datetime, timezone


parser = argparse.ArgumentParser()
parser.add_argument("--results", required=True, type=pathlib.Path)
parser.add_argument("--phase", choices=("smoke", "core", "cache-pressure", "all"), default="all")
parser.add_argument("--duration-ms", type=int, default=5_000)
parser.add_argument("--warmup-ms", type=int, default=2_000)
parser.add_argument("--repetitions", type=int, default=3)
parser.add_argument("--engine-filter", default="all")
parser.add_argument("--case-filter", default="all")
arguments = parser.parse_args()

results = arguments.results
raw = results / "raw"
raw.mkdir(parents=True, exist_ok=True)
order_path = results / "run-order.jsonl"
data_root = pathlib.Path("/bench/zfs/db")
phase0_binary = pathlib.Path(os.environ.get("PHASE0_BINARY", "/tmp/dodb-product-target/release/phase0-bench"))
sqlite_binary = pathlib.Path(os.environ.get("SQLITE_BINARY", "/tmp/sqlite-product-target/release/sqlite-bench"))
turso_binary = pathlib.Path(os.environ.get("TURSO_BINARY", "/tmp/turso-product-target/release/turso-bench"))
rocksdb_binary = pathlib.Path(os.environ.get("ROCKSDB_BINARY", "/tmp/rocksdb-product-target/release/rocksdb-bench"))
base_seed = 979_000_000
seed_stride = 1_009
resident_rows = 10_000
cache_pressure_rows = 250_000
cache_pages = 16_384
engine_names = ["main-btree", "parallel-blink", "sqlite-wal", "turso-wal", "turso-mvcc", "rocksdb"]


def workloads():
    cases = []
    for clients in (1, 4, 16, 64):
        cases.append((f"P1-get-c{clients}", "get", clients, 1, "uniform", 512, resident_rows, 100))
    for clients in (1, 4, 16):
        cases.append((f"P2-query16-c{clients}", "query", clients, 1, "uniform", 512, resident_rows, 100))
    for clients in (1, 4, 16, 64):
        cases.append((f"P3-write-w1-c{clients}", "write", clients, 1, "uniform", 512, resident_rows, 0))
    for width in (4, 8):
        for clients in (1, 4, 16, 64):
            cases.append((f"P4-tx-w{width}-c{clients}", "write", clients, width, "uniform", 512, resident_rows, 0))
        cases.append((f"P4-tx-w{width}-spread-c64", "write", 64, width, "different-leaf-heavy", 512, resident_rows, 0))
    for ratio, read_percent in (("95-5", 95), ("50-50", 50), ("20-80", 20)):
        for clients in (4, 16, 64):
            cases.append((f"P{'5' if ratio == '95-5' else '6' if ratio == '50-50' else '7'}-mixed-{ratio}-c{clients}", "mixed", clients, 1, "uniform", 512, resident_rows, read_percent))
    cases.append(("P8-hotspot-50-50-c16", "mixed", 16, 1, "hotspot", 512, resident_rows, 50))
    for distribution in ("uniform", "different-leaf-heavy", "same-leaf-heavy"):
        cases.append((f"P9-stress-w16-c64-{distribution}", "write", 64, 16, distribution, 64, resident_rows, 0))
    cases.extend([
        ("large-overflow-get-c16-2k", "get", 16, 1, "uniform", 2_048, resident_rows, 100),
        ("large-overflow-write-w1-c16-2k", "write", 16, 1, "uniform", 2_048, resident_rows, 0),
    ])
    pressure = [
        ("cache-pressure-get-c16", "get", 16, 1, "uniform", 512, cache_pressure_rows, 100),
        ("cache-pressure-query16-c16", "query", 16, 1, "uniform", 512, cache_pressure_rows, 100),
        ("cache-pressure-write-w1-c16", "write", 16, 1, "uniform", 512, cache_pressure_rows, 0),
        ("cache-pressure-mixed-50-50-c16", "mixed", 16, 1, "uniform", 512, cache_pressure_rows, 50),
    ]
    return cases, pressure


def rotate(items, offset):
    offset %= len(items)
    return items[offset:] + items[:offset]


def record(value):
    with order_path.open("a", encoding="utf-8") as output_file:
        output_file.write(json.dumps(value, sort_keys=True) + "\n")


def command_output(*command):
    result = subprocess.run(command, check=True, text=True, capture_output=True)
    return result.stdout.strip()


def environment_record():
    cpuinfo = pathlib.Path("/proc/cpuinfo").read_text(encoding="utf-8")
    meminfo = pathlib.Path("/proc/meminfo").read_text(encoding="utf-8")
    lscpu = command_output("lscpu")
    cpu_model = next((line.split(":", 1)[1].strip() for line in lscpu.splitlines()
                      if line.startswith("Model name:")), "unknown")
    memory_total_kib = next((int(line.split()[1]) for line in meminfo.splitlines()
                             if line.startswith("MemTotal:")), None)
    return {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "hostname": os.uname().nodename,
        "architecture": os.uname().machine,
        "kernel": os.uname().release,
        "cpu_model": cpu_model,
        "logical_cpus": os.cpu_count(),
        "memory_total_kib": memory_total_kib,
        "rustc": command_output("rustc", "--version"),
        "git_commit": command_output("git", "rev-parse", "HEAD"),
        "git_branch": command_output("git", "branch", "--show-current"),
        "main_sha": command_output("git", "rev-parse", "origin/main"),
        "filesystem": command_output("findmnt", "-T", str(data_root), "-o", "SOURCE,FSTYPE,OPTIONS", "-n"),
        "pool_status": command_output("zpool", "status", "dodbbench"),
        "zfs_properties": command_output("zfs", "get", "sync,recordsize,compression,atime", "dodbbench/db"),
        "durability": "durable-return; real sync for dodb, WAL FULL for SQLite and Turso, synced RocksDB WriteBatch",
        "primary_working_set_rows": resident_rows,
        "cache_pressure_working_set_rows": cache_pressure_rows,
        "dodb_cache_pages": cache_pages,
    }


def engine_command(engine, case, seed, scenario_index, repetition):
    name, operation, clients, width, distribution, value_size, working_set, read_percent = case
    output_path = raw / f"{name}-{engine}-rep{repetition + 1}.jsonl"
    log_path = raw / f"{name}-{engine}-rep{repetition + 1}.log"
    if engine in ("main-btree", "parallel-blink"):
        command = [str(phase0_binary), "--engine", engine]
        if operation in ("get", "query"):
            command += ["--suite", "read", "--readers", str(clients), "--read-kinds", operation]
        elif operation == "write":
            command += ["--suite", "write", "--writers", str(clients), "--widths", str(width)]
        else:
            readers = max(1, round(clients * read_percent / 100))
            writers = clients - readers
            if writers == 0:
                writers, readers = 1, clients - 1
            command += ["--suite", "mixed", "--readers", str(readers), "--writers", str(writers), "--mixes", f"{read_percent}/{100 - read_percent}"]
        command += ["--distributions", distribution, "--duration", f"{arguments.duration_ms}ms",
                    "--warmup", f"{arguments.warmup_ms}ms", "--repetitions", "1",
                    "--cache-capacity", str(cache_pages), "--working-set", str(working_set),
                    "--key-size", "16", "--value-size", str(value_size), "--read-limit", "16",
                    "--sync-mode", "real", "--transaction-mode", "unconditional", "--seed", str(seed),
                    "--output", str(output_path)]
        environment = os.environ.copy()
        environment["DODB_BENCH_DIR"] = str(data_root / f"dodb-{engine}-{name}-rep{repetition + 1}")
    else:
        binary, options = {
            "sqlite-wal": (sqlite_binary, []),
            "turso-wal": (turso_binary, ["--journal", "wal"]),
            "turso-mvcc": (turso_binary, ["--journal", "mvcc", "--group-commit", "on"]),
            "rocksdb": (rocksdb_binary, ["--pipelined", "off"]),
        }[engine]
        data_dir = data_root / f"{engine}-{name}-rep{repetition + 1}"
        command = [str(binary), *options, "--mode", "bench", "--operation", operation,
                   "--read-percent", str(read_percent), "--read-limit", "16", "--writers", str(clients),
                   "--width", str(width), "--distribution", distribution, "--working-set", str(working_set),
                   "--key-size", "16", "--value-size", str(value_size), "--warmup-ms", str(arguments.warmup_ms),
                   "--duration-ms", str(arguments.duration_ms), "--seed", str(seed),
                   "--scenario-index", str(scenario_index), "--repetition", str(repetition + 1),
                   "--data-dir", str(data_dir), "--output", str(output_path)]
        environment = os.environ.copy()
    return command, environment, output_path, log_path


def execute(engine, case, scenario_index, repetition):
    seed = base_seed + scenario_index * seed_stride + repetition
    command, environment, output_path, log_path = engine_command(engine, case, seed, scenario_index, repetition)
    started = datetime.now(timezone.utc).isoformat()
    binary_path = pathlib.Path(command[0])
    binary_hash = command_output("sha256sum", str(binary_path)).split()[0]
    record({"event": "start", "timestamp": started, "engine": engine, "case": case[0],
            "seed": seed, "scenario_index": scenario_index, "repetition": repetition + 1,
            "command": command, "binary_sha256": binary_hash, "output": str(output_path)})
    with log_path.open("w", encoding="utf-8") as log_file:
        process = subprocess.run(command, stdout=log_file, stderr=subprocess.STDOUT, env=environment)
    record({"event": "complete", "timestamp": datetime.now(timezone.utc).isoformat(),
            "engine": engine, "case": case[0], "seed": seed, "scenario_index": scenario_index,
            "repetition": repetition + 1, "exit_code": process.returncode,
            "output": str(output_path), "log": str(log_path)})
    if process.returncode != 0:
        raise SystemExit(f"benchmark failed: engine={engine}, case={case[0]}, repetition={repetition + 1}")
    if not output_path.exists():
        raise SystemExit(f"benchmark output missing: {output_path}")
    records = [json.loads(line) for line in output_path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if not records:
        raise SystemExit(f"benchmark output is empty: {output_path}")
    for result in records:
        verification = result.get("verification", {})
        if verification.get("passed") is False:
            raise SystemExit(f"benchmark data verification failed: engine={engine}, case={case[0]}")


core_cases, pressure_cases = workloads()
selected_cases = []
if arguments.phase in ("smoke", "core", "all"):
    selected_cases.extend(core_cases[:1] if arguments.phase == "smoke" else core_cases)
if arguments.phase in ("cache-pressure", "all"):
    selected_cases.extend(pressure_cases)
if arguments.case_filter != "all":
    allowed_cases = set(arguments.case_filter.split(","))
    selected_cases = [case for case in selected_cases if case[0] in allowed_cases]
    missing_cases = allowed_cases - {case[0] for case in selected_cases}
    if missing_cases:
        raise SystemExit(f"unknown or phase-excluded cases: {sorted(missing_cases)}")
selected_engines = engine_names if arguments.engine_filter == "all" else arguments.engine_filter.split(",")
results.mkdir(parents=True, exist_ok=True)
(results / "environment.json").write_text(json.dumps(environment_record(), indent=2) + "\n", encoding="utf-8")
for scenario_index, case in enumerate(selected_cases):
    for repetition in range(arguments.repetitions):
        for engine in rotate(selected_engines, scenario_index + repetition):
            execute(engine, case, scenario_index, repetition)

manifest_lines = []
for artifact in sorted(results.rglob("*")):
    if artifact.is_file() and artifact.name != "artifact-sha256.txt":
        digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
        manifest_lines.append(f"{digest}  {artifact.relative_to(results)}")
(results / "artifact-sha256.txt").write_text("\n".join(manifest_lines) + "\n", encoding="utf-8")
