import argparse
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys
import time
from datetime import datetime, timezone

from summarize_payload_batching import ValidationError, validate_raw_record, write_artifact_manifest


BASE_SEED = 979_000_000
SEED_STRIDE = 1_009
DEFAULT_RESULTS = None
RUNNER_NAME = "run_payload_batching_matrix.py"
SUMMARIZER_NAME = "summarize_payload_batching.py"
TEST_NAME = "test_payload_batching.py"
DEFAULT_VARIANTS = (
    "main-btree",
    "parallel-blink-current",
    "parallel-blink-main-parity",
    "rocksdb",
)
VARIANT_SPECS = {
    "main-btree": {
        "binary_key": "phase0-bench",
        "binary_kind": "phase0",
        "engine": "main-btree",
        "collection_policy": "native-main",
        "engine_options": (),
    },
    "main-btree-compact-wal": {
        "binary_key": "phase0-bench",
        "binary_kind": "phase0",
        "engine": "main-btree",
        "collection_policy": "native-main",
        "engine_options": ("--main-compact-wal",),
    },
    "parallel-blink-current": {
        "binary_key": "phase0-bench",
        "binary_kind": "phase0",
        "engine": "parallel-blink",
        "collection_policy": "current",
        "engine_options": ("--blink-collection-policy", "current"),
    },
    "parallel-blink-main-parity": {
        "binary_key": "phase0-bench",
        "binary_kind": "phase0",
        "engine": "parallel-blink",
        "collection_policy": "main-parity",
        "engine_options": ("--blink-collection-policy", "main-parity"),
    },
    "parallel-blink-main-parity-borrowed": {
        "binary_key": "phase0-borrowed-pages",
        "binary_env": "PHASE0_BORROWED_PAGES_BINARY",
        "binary_kind": "phase0",
        "engine": "parallel-blink",
        "collection_policy": "main-parity",
        "blink_workers": 2,
        "engine_options": ("--blink-collection-policy", "main-parity", "--blink-workers", "2"),
    },
    "parallel-blink-main-parity-workers1": {
        "binary_key": "phase0-bench",
        "binary_kind": "phase0",
        "engine": "parallel-blink",
        "collection_policy": "main-parity",
        "engine_options": (
            "--blink-collection-policy",
            "main-parity",
            "--blink-workers",
            "1",
        ),
        "blink_workers": 1,
    },
    "parallel-blink-main-parity-adaptive32": {
        "binary_key": "phase0-bench",
        "binary_kind": "phase0",
        "engine": "parallel-blink",
        "collection_policy": "main-parity",
        "engine_options": (
            "--blink-collection-policy",
            "main-parity",
            "--blink-workers",
            "2",
            "--parallel-background-min-operations",
            "32",
        ),
        "blink_workers": 2,
        "parallel_background_min_operations": 32,
    },
    "planned-blink-main-parity": {
        "binary_key": "phase0-bench",
        "binary_kind": "phase0",
        "engine": "planned-blink",
        "collection_policy": "main-parity",
        "engine_options": ("--blink-collection-policy", "main-parity"),
    },
    "planned-blink-main-parity-workers0": {
        "binary_key": "phase0-bench",
        "binary_kind": "phase0",
        "engine": "planned-blink",
        "collection_policy": "main-parity",
        "engine_options": (
            "--blink-collection-policy",
            "main-parity",
            "--parallel-workers",
            "0",
        ),
    },
    "rocksdb": {
        "binary_key": "rocksdb-bench",
        "binary_kind": "rocksdb",
        "engine": "rocksdb",
        "collection_policy": "not-applicable",
        "engine_options": (),
    },
}


class RunnerError(RuntimeError):
    pass


def timestamp_utc():
    return datetime.now(timezone.utc).isoformat()


def sha256_file(path):
    digest = hashlib.sha256()
    with pathlib.Path(path).open("rb") as input_file:
        for chunk in iter(lambda: input_file.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def command_output(*command):
    try:
        completed = subprocess.run(command, check=True, text=True, capture_output=True)
    except (OSError, subprocess.CalledProcessError):
        return None
    return completed.stdout.strip()


def process_rss_kib(process_id):
    try:
        status = pathlib.Path(f"/proc/{process_id}/status").read_text(encoding="utf-8")
    except OSError:
        return None
    values = {}
    for line in status.splitlines():
        if line.startswith("VmRSS:"):
            values["end"] = int(line.split()[1])
        elif line.startswith("VmHWM:"):
            values["hwm"] = int(line.split()[1])
    return values if len(values) == 2 else None


def run_process_with_rss(command, cwd, environment, stdout):
    process = subprocess.Popen(
        command,
        cwd=cwd,
        stdout=stdout,
        stderr=subprocess.STDOUT,
        env=environment,
    )
    final_sample = None
    peak_hwm_kib = 0
    while True:
        sample = process_rss_kib(process.pid)
        if sample is not None:
            final_sample = sample
            peak_hwm_kib = max(peak_hwm_kib, sample["hwm"])
        return_code = process.poll()
        if return_code is not None:
            break
        time.sleep(0.02)
    return_code = process.wait()
    if final_sample is None:
        raise RunnerError(f"unable to sample benchmark process memory: pid={process.pid}")
    return return_code, {"end": final_sample["end"], "hwm": peak_hwm_kib}


def parse_integer_list(value, name, minimum=1):
    try:
        parsed_values = [int(item.strip()) for item in value.split(",") if item.strip()]
    except ValueError as error:
        raise argparse.ArgumentTypeError(f"{name} must be a comma-separated integer list") from error
    if not parsed_values or any(item < minimum for item in parsed_values):
        raise argparse.ArgumentTypeError(f"{name} values must be at least {minimum}")
    if len(set(parsed_values)) != len(parsed_values):
        raise argparse.ArgumentTypeError(f"{name} must not contain duplicates")
    return parsed_values


def parse_string_list(value, name):
    parsed_values = [item.strip() for item in value.split(",") if item.strip()]
    if not parsed_values:
        raise argparse.ArgumentTypeError(f"{name} must not be empty")
    if len(set(parsed_values)) != len(parsed_values):
        raise argparse.ArgumentTypeError(f"{name} must not contain duplicates")
    return parsed_values


def parse_mixes(value):
    mixes = []
    for item in value.split(","):
        parts = item.strip().split("/")
        if len(parts) != 2:
            raise argparse.ArgumentTypeError("mixes must use read/write percentages such as 50/50")
        try:
            read_percent, write_percent = (int(part) for part in parts)
        except ValueError as error:
            raise argparse.ArgumentTypeError("mix percentages must be integers") from error
        if read_percent < 0 or write_percent < 0 or read_percent + write_percent != 100:
            raise argparse.ArgumentTypeError("each read/write mix must be nonnegative and sum to 100")
        mixes.append((read_percent, write_percent))
    if not mixes:
        raise argparse.ArgumentTypeError("at least one mix is required")
    if len(set(mixes)) != len(mixes):
        raise argparse.ArgumentTypeError("mixes must not contain duplicates")
    return mixes


def parse_arguments(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--results", required=True, type=pathlib.Path)
    parser.add_argument("--clients", default="4,64")
    parser.add_argument("--variants", default=",".join(DEFAULT_VARIANTS))
    parser.add_argument("--value-modes", default="constant,changing")
    parser.add_argument("--repetitions", type=int, default=3)
    parser.add_argument("--mixes", default="50/50")
    parser.add_argument("--duration-ms", type=int, default=5_000)
    parser.add_argument("--warmup-ms", type=int, default=2_000)
    parser.add_argument("--working-set", type=int, default=10_000)
    parser.add_argument("--transaction-width", type=int, default=1)
    parser.add_argument("--key-size", type=int, default=16)
    parser.add_argument("--value-size", type=int, default=512)
    parser.add_argument("--distribution", choices=("uniform",), default="uniform")
    parser.add_argument("--cache-capacity", type=int, default=16_384)
    parser.add_argument("--read-limit", type=int, default=16)
    parser.add_argument("--base-seed", type=int, default=BASE_SEED)
    arguments = parser.parse_args(argv)

    arguments.results = arguments.results.expanduser().resolve()
    arguments.clients = parse_integer_list(arguments.clients, "clients")
    arguments.variants = parse_string_list(arguments.variants, "variants")
    arguments.value_modes = parse_string_list(arguments.value_modes, "value-modes")
    arguments.mixes = parse_mixes(arguments.mixes)
    if arguments.repetitions < 1:
        parser.error("repetitions must be positive")
    if arguments.duration_ms < 1 or arguments.warmup_ms < 0:
        parser.error("duration-ms must be positive and warmup-ms must be nonnegative")
    for field_name in ("working_set", "transaction_width", "key_size", "value_size", "cache_capacity", "read_limit"):
        if getattr(arguments, field_name) < 1:
            parser.error(f"{field_name.replace('_', '-')} must be positive")
    if arguments.base_seed < 0:
        parser.error("base-seed must be nonnegative")
    unsupported_variants = sorted(set(arguments.variants) - set(VARIANT_SPECS))
    if unsupported_variants:
        parser.error(f"unsupported variants: {unsupported_variants}")
    unsupported_modes = sorted(set(arguments.value_modes) - {"constant", "changing"})
    if unsupported_modes:
        parser.error(f"unsupported value modes: {unsupported_modes}")
    return arguments


def git_output(repository, *arguments):
    try:
        completed = subprocess.run(
            ("git", *arguments),
            cwd=repository,
            check=True,
            text=True,
            capture_output=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise RunnerError(f"git command failed: {' '.join(arguments)}") from error
    return completed.stdout.strip()


def relative_repo_path(repository, path):
    try:
        return pathlib.Path(path).resolve().relative_to(repository.resolve()).as_posix()
    except ValueError:
        return None


def source_state(repository, output_path):
    commit = git_output(repository, "rev-parse", "HEAD")
    branch = git_output(repository, "branch", "--show-current")
    status = git_output(repository, "status", "--porcelain=v1", "--untracked-files=all")
    allowed_untracked = {
        f"dodb/experiments/mixed-harness/{RUNNER_NAME}",
        f"dodb/experiments/mixed-harness/{SUMMARIZER_NAME}",
        f"dodb/experiments/mixed-harness/{TEST_NAME}",
    }
    output_relative = relative_repo_path(repository, output_path)
    if output_relative is not None:
        allowed_untracked.add(output_relative)
    tracked_changes = []
    unrelated_untracked = []
    for line in status.splitlines():
        if not line:
            continue
        state = line[:2]
        changed_path = line[3:]
        if state == "??":
            is_result_artifact = changed_path.startswith("dodb/docs/experiments/results/")
            is_current_result = output_relative is not None and (
                changed_path == output_relative or changed_path.startswith(output_relative + "/")
            )
            if changed_path not in allowed_untracked and not is_result_artifact and not is_current_result:
                unrelated_untracked.append(changed_path)
        else:
            tracked_changes.append(line)
    return {
        "commit": commit,
        "branch": branch,
        "tracked_changes": tracked_changes,
        "unrelated_untracked": unrelated_untracked,
    }


def require_clean_source(repository, output_path):
    state = source_state(repository, output_path)
    if state["tracked_changes"]:
        raise RunnerError(f"tracked source changes are present: {state['tracked_changes']}")
    if state["unrelated_untracked"]:
        raise RunnerError(f"unrelated untracked source files are present: {state['unrelated_untracked']}")
    return state


def optional_read(path):
    try:
        return pathlib.Path(path).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None


def environment_record(repository, data_root):
    machine = os.uname()
    lscpu = command_output("lscpu")
    cpu_model = None
    if lscpu:
        cpu_model = next(
            (
                line.split(":", 1)[1].strip()
                for line in lscpu.splitlines()
                if line.startswith("Model name:")
            ),
            None,
        )
    meminfo = optional_read("/proc/meminfo")
    memory_total_kib = None
    if meminfo:
        memory_total_kib = next(
            (
                int(line.split()[1])
                for line in meminfo.splitlines()
                if line.startswith("MemTotal:")
            ),
            None,
        )
    source = source_state(repository, data_root)
    zfs_dataset = os.environ.get("ZFS_DATASET", "dodbbench/db")
    return {
        "timestamp_utc": timestamp_utc(),
        "hostname": machine.nodename,
        "architecture": machine.machine,
        "kernel": machine.release,
        "os_release": optional_read("/etc/os-release"),
        "cpu_model": cpu_model,
        "logical_cpus": os.cpu_count(),
        "memory_total_kib": memory_total_kib,
        "rustc": command_output("rustc", "--version"),
        "cargo": command_output("cargo", "--version"),
        "python": command_output(sys.executable, "--version"),
        "git_commit": source["commit"],
        "git_branch": source["branch"],
        "filesystem": command_output(
            "findmnt", "-T", str(data_root), "-o", "SOURCE,FSTYPE,OPTIONS", "-n"
        ),
        "zpool_status": command_output("zpool", "status", "dodbbench"),
        "zfs_dataset": zfs_dataset,
        "zfs_properties": command_output(
            "zfs", "get", "sync,recordsize,compression,atime", zfs_dataset
        ),
        "data_root": str(data_root),
    }


def build_expected_cells(arguments, source_commit):
    expected_cells = []
    run_pairs = [
        (variant, value_mode)
        for variant in arguments.variants
        for value_mode in arguments.value_modes
    ]
    for clients in arguments.clients:
        for read_percent, write_percent in arguments.mixes:
            scenario_index = clients * 1000 + read_percent * 10 + write_percent
            for repetition in range(1, arguments.repetitions + 1):
                seed = arguments.base_seed + scenario_index * SEED_STRIDE + repetition - 1
                if seed > (1 << 64) - 1:
                    raise RunnerError(f"seed exceeds u64 range for c{clients} repetition {repetition}")
                for variant, value_mode in run_pairs:
                    variant_spec = VARIANT_SPECS[variant]
                    cell_id = (
                        f"clients-{clients}-mix-{read_percent}-{write_percent}-"
                        f"{value_mode}-{variant}-rep{repetition}"
                    )
                    expected_cells.append(
                        {
                            "cell_id": cell_id,
                            "scenario_index": scenario_index,
                            "variant": variant,
                            "expected_engine": variant_spec["engine"],
                            "expected_collection_policy": variant_spec["collection_policy"],
                            "expected_blink_workers": variant_spec.get("blink_workers"),
                            "expected_parallel_background_min_operations": (
                                variant_spec.get("parallel_background_min_operations", 0)
                                if variant_spec["binary_kind"] == "phase0"
                                else None
                            ),
                            "expected_blink_read_observational_metrics_enabled": (
                                True if variant_spec["binary_kind"] == "phase0" else None
                            ),
                            "expected_parallel_background_worker_dispatches_metric": (
                                variant_spec["binary_kind"] == "phase0"
                            ),
                            "binary_key": variant_spec["binary_key"],
                            "binary_kind": variant_spec["binary_kind"],
                            "clients": clients,
                            "read_percent": read_percent,
                            "write_percent": write_percent,
                            "value_mode": value_mode,
                            "repetition": repetition,
                            "expected_raw_repetition": 0 if variant_spec["binary_kind"] == "phase0" else repetition,
                            "seed": seed,
                            "transaction_width": arguments.transaction_width,
                            "distribution": arguments.distribution,
                            "working_set": arguments.working_set,
                            "key_size": arguments.key_size,
                            "value_size": arguments.value_size,
                            "warmup_ms": arguments.warmup_ms,
                            "duration_ms": arguments.duration_ms,
                            "cache_capacity": arguments.cache_capacity,
                            "read_limit": arguments.read_limit,
                        }
                    )
    return expected_cells


def write_json(path, value):
    pathlib.Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def record_event(results, event):
    event_path = results / "run-order.jsonl"
    with event_path.open("a", encoding="utf-8") as output_file:
        output_file.write(json.dumps(event, sort_keys=True) + "\n")
        output_file.flush()
        os.fsync(output_file.fileno())


def binary_path_for(arguments, binary_kind, binary_env=None):
    if binary_env:
        binary_path = os.environ.get(binary_env)
        if not binary_path:
            raise RunnerError(f"required binary environment variable is missing: {binary_env}")
        return pathlib.Path(binary_path).expanduser().resolve()
    if binary_kind == "phase0":
        return pathlib.Path(
            os.environ.get("PHASE0_BINARY", "/tmp/dodb-product-target/release/phase0-bench")
        ).expanduser().resolve()
    return pathlib.Path(
        os.environ.get("ROCKSDB_BINARY", "/tmp/rocksdb-product-target/release/rocksdb-bench")
    ).expanduser().resolve()


def raw_stem(cell):
    return re.sub(r"[^a-zA-Z0-9._-]+", "-", cell["cell_id"])


def build_command(arguments, cell, binary, output_path, data_dir):
    variant_spec = VARIANT_SPECS[cell["variant"]]
    if cell["binary_kind"] == "phase0":
        command = [
            str(binary),
            "--engine",
            cell["expected_engine"],
            "--suite",
            "mixed",
            "--readers",
            "0",
            "--writers",
            str(cell["clients"]),
            "--widths",
            str(cell["transaction_width"]),
            "--mixes",
            f"{cell['read_percent']}/{cell['write_percent']}",
            "--mixed-clients",
            "--distributions",
            cell["distribution"],
            "--duration",
            f"{cell['duration_ms']}ms",
            "--warmup",
            f"{cell['warmup_ms']}ms",
            "--repetitions",
            "1",
            "--cache-capacity",
            str(cell["cache_capacity"]),
            "--working-set",
            str(cell["working_set"]),
            "--key-size",
            str(cell["key_size"]),
            "--value-size",
            str(cell["value_size"]),
            "--read-limit",
            str(cell["read_limit"]),
            "--sync-mode",
            "real",
            "--transaction-mode",
            "unconditional",
            "--mixed-value-mode",
            cell["value_mode"],
            "--seed",
            str(cell["seed"]),
            "--output",
            str(output_path),
            *variant_spec["engine_options"],
        ]
        environment = os.environ.copy()
        environment["DODB_BENCH_DIR"] = str(data_dir)
        return command, environment

    command = [
        str(binary),
        "--mode",
        "bench",
        "--operation",
        "mixed",
        "--read-percent",
        str(cell["read_percent"]),
        "--read-limit",
        str(cell["read_limit"]),
        "--writers",
        str(cell["clients"]),
        "--width",
        str(cell["transaction_width"]),
        "--distribution",
        cell["distribution"],
        "--working-set",
        str(cell["working_set"]),
        "--key-size",
        str(cell["key_size"]),
        "--value-size",
        str(cell["value_size"]),
        "--warmup-ms",
        str(cell["warmup_ms"]),
        "--duration-ms",
        str(cell["duration_ms"]),
        "--seed",
        str(cell["seed"]),
        "--scenario-index",
        str(cell["scenario_index"]),
        "--repetition",
        str(cell["repetition"]),
        "--data-dir",
        str(data_dir),
        "--output",
        str(output_path),
        "--mixed-value-mode",
        cell["value_mode"],
    ]
    return command, os.environ.copy()


def event_identity(cell, source_commit, source_branch):
    return {
        "cell_id": cell["cell_id"],
        "variant": cell["variant"],
        "value_mode": cell["value_mode"],
        "clients": cell["clients"],
        "read_percent": cell["read_percent"],
        "write_percent": cell["write_percent"],
        "repetition": cell["repetition"],
        "seed": cell["seed"],
        "scenario_index": cell["scenario_index"],
        "checkout_sha": source_commit,
        "source_branch": source_branch,
    }


def execute_cell(arguments, results, repository, cell, binary_registry, source_state_before):
    binary = binary_path_for(arguments, cell["binary_kind"], VARIANT_SPECS[cell["variant"]].get("binary_env"))
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise RunnerError(f"benchmark executable is missing or not executable: {binary}")
    binary_hash = sha256_file(binary)
    registered = binary_registry.setdefault(cell["binary_key"], {"path": str(binary), "sha256": binary_hash})
    if registered["path"] != str(binary) or registered["sha256"] != binary_hash:
        raise RunnerError(f"benchmark binary changed within matrix: {cell['binary_key']}")

    raw_directory = results / "raw"
    raw_directory.mkdir(exist_ok=True)
    stem = raw_stem(cell)
    output_path = raw_directory / f"{stem}.jsonl"
    log_path = raw_directory / f"{stem}.log"
    data_root = pathlib.Path(os.environ.get("BENCH_DATA_ROOT", "/bench/zfs/db")).expanduser().resolve()
    data_dir = data_root / results.name / stem
    if output_path.exists() or log_path.exists():
        raise RunnerError(f"run output already exists and will not be overwritten: {stem}")
    if data_dir.exists():
        raise RunnerError(f"benchmark data directory already exists and will not be reused: {data_dir}")

    command, environment = build_command(arguments, cell, binary, output_path, data_dir)
    identity = event_identity(cell, source_state_before["commit"], source_state_before["branch"])
    record_event(
        results,
        {
            "event": "start",
            **identity,
            "timestamp_utc": timestamp_utc(),
            "binary_key": cell["binary_key"],
            "binary_path": str(binary),
            "binary_sha256": binary_hash,
            "command": command,
            "output": str(output_path),
            "log": str(log_path),
            "data_dir": str(data_dir),
        },
    )
    data_dir.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open("x", encoding="utf-8") as log_file:
        return_code, process_rss = run_process_with_rss(
            command,
            repository,
            environment,
            log_file,
        )

    output_digest = None
    if output_path.is_file():
        output_lines = [line for line in output_path.read_text(encoding="utf-8").splitlines() if line.strip()]
        if len(output_lines) == 1:
            raw_record = json.loads(output_lines[0])
            raw_record["rss_kib"] = process_rss
            output_path.write_text(json.dumps(raw_record, sort_keys=True) + "\n", encoding="utf-8")
            output_digest = sha256_file(output_path)
    log_digest = sha256_file(log_path) if log_path.is_file() else None
    record_event(
        results,
        {
            "event": "complete",
            **identity,
            "timestamp_utc": timestamp_utc(),
            "binary_key": cell["binary_key"],
            "binary_path": str(binary),
            "binary_sha256": binary_hash,
            "exit_code": return_code,
            "output": str(output_path),
            "log": str(log_path),
            "output_sha256": output_digest,
            "log_sha256": log_digest,
            "process_rss_kib": process_rss,
        },
    )
    if return_code != 0:
        raise RunnerError(
            f"benchmark failed: variant={cell['variant']} mode={cell['value_mode']} "
            f"clients={cell['clients']} repetition={cell['repetition']} exit_code={return_code}"
        )
    if output_digest is None:
        raise RunnerError(f"benchmark output is missing: {output_path}")
    output_lines = [line for line in output_path.read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(output_lines) != 1:
        raise RunnerError(f"expected one raw result record at {output_path}, found {len(output_lines)}")
    try:
        record = json.loads(output_lines[0])
    except json.JSONDecodeError as error:
        raise RunnerError(f"invalid raw JSON at {output_path}: {error}") from error
    try:
        validate_raw_record(record, cell, source_state_before["commit"])
    except ValidationError as error:
        raise RunnerError(f"raw validation failed for {cell['cell_id']}: {error}") from error
    return output_path


def rotate(items, offset):
    if not items:
        return []
    normalized_offset = offset % len(items)
    return items[normalized_offset:] + items[:normalized_offset]


def verify_final_provenance(repository, results, source_state_before, binary_registry, arguments):
    source_state_after = require_clean_source(repository, results)
    if source_state_after["commit"] != source_state_before["commit"]:
        raise RunnerError(
            f"source HEAD changed during matrix: {source_state_before['commit']} -> {source_state_after['commit']}"
        )
    for binary_key, registered in binary_registry.items():
        binary_path = pathlib.Path(registered["path"])
        if not binary_path.is_file() or sha256_file(binary_path) != registered["sha256"]:
            raise RunnerError(f"benchmark binary changed during matrix: {binary_key}")


def write_failure_state(results, planned_count, completed_count, source_commit, error):
    status = {
        "accepted": False,
        "planned_count": planned_count,
        "completed_count": completed_count,
        "source_commit": source_commit,
        "failure": str(error),
        "updated_at_utc": timestamp_utc(),
    }
    write_json(results / "matrix-status.json", status)
    write_json(
        results / "runner-failure.json",
        {
            "failure": str(error),
            "failure_type": type(error).__name__,
            "timestamp_utc": timestamp_utc(),
        },
    )
    write_artifact_manifest(results)


def build_matrix_config(arguments, source_commit, source_branch, data_root):
    expected_cells = build_expected_cells(arguments, source_commit)
    return {
        "schema_version": 2,
        "created_at_utc": timestamp_utc(),
        "source_commit": source_commit,
        "source_branch": source_branch,
        "planned_count": len(expected_cells),
        "config": {
            "clients": arguments.clients,
            "variants": [
                {
                    "name": variant,
                    "engine": VARIANT_SPECS[variant]["engine"],
                    "collection_policy": VARIANT_SPECS[variant]["collection_policy"],
                    "binary_key": VARIANT_SPECS[variant]["binary_key"],
                    "blink_workers": VARIANT_SPECS[variant].get("blink_workers"),
                    "parallel_background_min_operations": (
                        VARIANT_SPECS[variant].get("parallel_background_min_operations", 0)
                        if VARIANT_SPECS[variant]["binary_kind"] == "phase0"
                        else None
                    ),
                    "blink_read_observational_metrics_enabled": (
                        True if VARIANT_SPECS[variant]["binary_kind"] == "phase0" else None
                    ),
                }
                for variant in arguments.variants
            ],
            "value_modes": arguments.value_modes,
            "mixes": [
                f"{read_percent}/{write_percent}"
                for read_percent, write_percent in arguments.mixes
            ],
            "repetitions": arguments.repetitions,
            "duration_ms": arguments.duration_ms,
            "warmup_ms": arguments.warmup_ms,
            "transaction_width": arguments.transaction_width,
            "distribution": arguments.distribution,
            "working_set": arguments.working_set,
            "key_size": arguments.key_size,
            "value_size": arguments.value_size,
            "cache_capacity": arguments.cache_capacity,
            "read_limit": arguments.read_limit,
            "sync_mode": "real",
            "sync_contract": "durable-return",
            "base_seed": arguments.base_seed,
            "seed_stride": SEED_STRIDE,
            "data_root": str(data_root),
        },
        "expected_cells": expected_cells,
    }


def run_matrix(arguments):
    repository = pathlib.Path(__file__).resolve().parents[3]
    results = arguments.results
    source_state_before = require_clean_source(repository, results)
    results.mkdir(parents=True, exist_ok=True)
    if any(results.iterdir()):
        raise RunnerError(f"results directory must be empty: {results}")

    data_root = pathlib.Path(os.environ.get("BENCH_DATA_ROOT", "/bench/zfs/db")).expanduser().resolve()
    config = build_matrix_config(
        arguments,
        source_state_before["commit"],
        source_state_before["branch"],
        data_root,
    )
    expected_cells = config["expected_cells"]
    write_json(results / "matrix-config.json", config)
    write_json(results / "environment.json", environment_record(repository, data_root))
    write_json(
        results / "matrix-status.json",
        {
            "accepted": False,
            "planned_count": len(expected_cells),
            "completed_count": 0,
            "source_commit": source_state_before["commit"],
            "state": "running",
        },
    )

    binary_registry = {}
    completed_count = 0
    run_pairs = [
        (variant, value_mode)
        for variant in arguments.variants
        for value_mode in arguments.value_modes
    ]
    try:
        for clients in arguments.clients:
            for read_percent, write_percent in arguments.mixes:
                scenario_index = clients * 1000 + read_percent * 10 + write_percent
                for repetition in range(1, arguments.repetitions + 1):
                    rotated_pairs = rotate(run_pairs, scenario_index + repetition - 1)
                    for variant, value_mode in rotated_pairs:
                        cell_id = (
                            f"clients-{clients}-mix-{read_percent}-{write_percent}-"
                            f"{value_mode}-{variant}-rep{repetition}"
                        )
                        cell = next(cell for cell in expected_cells if cell["cell_id"] == cell_id)
                        execute_cell(
                            arguments,
                            results,
                            repository,
                            cell,
                            binary_registry,
                            source_state_before,
                        )
                        completed_count += 1
        verify_final_provenance(repository, results, source_state_before, binary_registry, arguments)
        summarizer = pathlib.Path(__file__).with_name(SUMMARIZER_NAME)
        completed = subprocess.run(
            (sys.executable, str(summarizer), str(results)),
            cwd=repository,
            check=False,
        )
        if completed.returncode != 0:
            raise RunnerError(f"payload batching summarizer rejected results with exit code {completed.returncode}")
    except BaseException as error:
        write_failure_state(
            results,
            len(expected_cells),
            completed_count,
            source_state_before["commit"],
            error,
        )
        raise

    return config


def main(argv=None):
    arguments = parse_arguments(argv)
    try:
        config = run_matrix(arguments)
    except (RunnerError, OSError, subprocess.SubprocessError) as error:
        print(f"payload batching matrix failed: {error}", file=sys.stderr)
        return 1
    print(
        f"accepted={config['planned_count']}/{config['planned_count']} "
        f"results={arguments.results}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
