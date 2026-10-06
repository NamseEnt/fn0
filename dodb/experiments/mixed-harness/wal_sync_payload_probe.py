import argparse
import csv
import hashlib
import json
import os
import platform
import random
import socket
import subprocess
import sys
import time
from pathlib import Path


DEFAULT_PAYLOAD_BYTES = "64,256,1024,4096,16384,65536"
DEFAULT_MODES = "append,ring"
DEFAULT_RING_BYTES = 67_108_864
DEFAULT_ITERATIONS = 100
DEFAULT_WARMUP_ITERATIONS = 20
DEFAULT_REPETITIONS = 3


def parse_arguments():
    parser = argparse.ArgumentParser()
    parser.add_argument("--payload-bytes", default=DEFAULT_PAYLOAD_BYTES)
    parser.add_argument("--iterations", type=int, default=DEFAULT_ITERATIONS)
    parser.add_argument(
        "--warmup-iterations", type=int, default=DEFAULT_WARMUP_ITERATIONS
    )
    parser.add_argument("--repetitions", type=int, default=DEFAULT_REPETITIONS)
    parser.add_argument("--modes", default=DEFAULT_MODES)
    parser.add_argument("--ring-bytes", type=int, default=DEFAULT_RING_BYTES)
    parser.add_argument("--data-root", required=True)
    parser.add_argument("--results", required=True)
    parser.add_argument("--seed", type=int, default=979_000_000)
    args = parser.parse_args()
    args.payload_sizes = parse_positive_sizes(args.payload_bytes)
    args.mode_names = parse_modes(args.modes)
    if args.iterations <= 0:
        parser.error("--iterations must be positive")
    if args.warmup_iterations < 0:
        parser.error("--warmup-iterations must not be negative")
    if args.repetitions <= 0:
        parser.error("--repetitions must be positive")
    if args.ring_bytes <= 0:
        parser.error("--ring-bytes must be positive")
    if "ring" in args.mode_names and max(args.payload_sizes) > args.ring_bytes:
        parser.error("every payload must fit in --ring-bytes")
    return args


def parse_positive_sizes(value):
    try:
        sizes = [int(piece.strip()) for piece in value.split(",")]
    except ValueError as error:
        raise argparse.ArgumentTypeError("payload sizes must be comma-separated integers") from error
    if not sizes or any(size <= 0 for size in sizes):
        raise argparse.ArgumentTypeError("payload sizes must all be positive")
    if len(set(sizes)) != len(sizes):
        raise argparse.ArgumentTypeError("payload sizes must not repeat")
    return sizes


def parse_modes(value):
    modes = [piece.strip() for piece in value.split(",") if piece.strip()]
    if not modes or any(mode not in {"append", "ring"} for mode in modes):
        raise argparse.ArgumentTypeError("modes must contain append and/or ring")
    if len(set(modes)) != len(modes):
        raise argparse.ArgumentTypeError("modes must not repeat")
    return modes


def run_command(command, cwd=None):
    try:
        completed = subprocess.run(
            command,
            cwd=cwd,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    except OSError as error:
        return {"command": command, "error": str(error)}
    return {
        "command": command,
        "exit_code": completed.returncode,
        "stdout": completed.stdout.strip(),
        "stderr": completed.stderr.strip(),
    }


def git_state(repository_root):
    commit = run_command(["git", "rev-parse", "HEAD"], repository_root)
    status = run_command(
        ["git", "status", "--porcelain", "--untracked-files=all"], repository_root
    )
    return {
        "commit": commit.get("stdout"),
        "commit_error": commit.get("stderr") or commit.get("error"),
        "status_porcelain": status.get("stdout"),
        "status_error": status.get("stderr") or status.get("error"),
    }


def repository_root():
    root = Path(__file__).resolve().parents[3]
    response = run_command(["git", "rev-parse", "--show-toplevel"], root)
    if response.get("exit_code") == 0:
        return Path(response["stdout"])
    return root


def cpu_description():
    model = None
    cpu_path = Path("/proc/cpuinfo")
    if cpu_path.exists():
        for line in cpu_path.read_text(errors="replace").splitlines():
            if line.startswith("model name\t:") or line.startswith("Model\t:"):
                model = line.split(":", 1)[1].strip()
                break
    return {
        "logical_cpus": os.cpu_count(),
        "model": model or platform.processor() or None,
        "machine": platform.machine(),
    }


def zfs_description(data_root):
    mount = run_command(["findmnt", "-n", "-o", "SOURCE,FSTYPE,OPTIONS", "-T", str(data_root)])
    description = {"mount": mount.get("stdout"), "mount_error": mount.get("stderr") or mount.get("error")}
    if mount.get("exit_code") != 0:
        return description
    datasets = run_command(["zfs", "list", "-H", "-o", "name,mountpoint"])
    description["dataset_list_error"] = datasets.get("stderr") or datasets.get("error")
    if datasets.get("exit_code") != 0:
        return description
    root = data_root.resolve()
    candidates = []
    for line in datasets["stdout"].splitlines():
        columns = line.split("\t", 1)
        if len(columns) != 2 or columns[1] in {"-", "legacy"}:
            continue
        mount_path = Path(columns[1])
        try:
            root.relative_to(mount_path)
        except ValueError:
            continue
        candidates.append((len(str(mount_path)), columns[0], str(mount_path)))
    if not candidates:
        description["dataset"] = None
        return description
    _, dataset, dataset_mount = max(candidates)
    settings = run_command(
        [
            "zfs",
            "get",
            "-H",
            "-o",
            "property,value",
            "sync,recordsize,compression,atime",
            dataset,
        ]
    )
    description["dataset"] = dataset
    description["dataset_mount"] = dataset_mount
    description["settings"] = settings.get("stdout")
    description["settings_error"] = settings.get("stderr") or settings.get("error")
    return description


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def write_full(fd, payload, offset):
    written = 0
    while written < len(payload):
        count = os.pwrite(fd, payload[written:], offset + written)
        if count <= 0:
            raise OSError("pwrite returned no progress")
        written += count


def write_ring_full(fd, payload, offset, ring_bytes):
    payload_offset = 0
    ring_offset = offset
    while payload_offset < len(payload):
        available = ring_bytes - ring_offset
        segment_length = min(available, len(payload) - payload_offset)
        write_full(
            fd,
            payload[payload_offset : payload_offset + segment_length],
            ring_offset,
        )
        payload_offset += segment_length
        ring_offset = 0


def prefill_ring(fd, ring_bytes):
    chunk = bytes(min(1024 * 1024, ring_bytes))
    offset = 0
    while offset < ring_bytes:
        chunk_length = min(len(chunk), ring_bytes - offset)
        write_full(fd, chunk[:chunk_length], offset)
        offset += chunk_length
    os.fdatasync(fd)
    actual_size = os.fstat(fd).st_size
    if actual_size != ring_bytes:
        raise OSError(f"ring prefill size mismatch: {actual_size} != {ring_bytes}")


def timed_cycle(fd, mode, payload, ring_bytes, ring_offset):
    cycle_started = time.perf_counter_ns()
    write_started = time.perf_counter_ns()
    file_size = os.fstat(fd).st_size
    if mode == "append":
        write_full(fd, payload, file_size)
        next_ring_offset = ring_offset
    else:
        if file_size != ring_bytes:
            raise OSError(f"ring file size changed: {file_size} != {ring_bytes}")
        write_ring_full(fd, payload, ring_offset, ring_bytes)
        next_ring_offset = (ring_offset + len(payload)) % ring_bytes
    write_finished = time.perf_counter_ns()
    sync_started = time.perf_counter_ns()
    os.fdatasync(fd)
    sync_finished = time.perf_counter_ns()
    return (
        write_finished - write_started,
        sync_finished - sync_started,
        sync_finished - cycle_started,
        next_ring_offset,
    )


def percentile(values, fraction):
    ordered = sorted(values)
    position = round((len(ordered) - 1) * fraction)
    return ordered[position]


def summarize(values, prefix):
    return {
        f"{prefix}_ns_count": len(values),
        f"{prefix}_ns_sum": sum(values),
        f"{prefix}_ns_mean": sum(values) / len(values),
        f"{prefix}_ns_p50": percentile(values, 0.50),
        f"{prefix}_ns_p95": percentile(values, 0.95),
        f"{prefix}_ns_p99": percentile(values, 0.99),
    }


def run_case(args, mode, payload_size, repetition, environment):
    case_seed = (
        args.seed
        ^ payload_size
        ^ (repetition * 0x9E3779B97F4A7C15)
        ^ (0xA55A if mode == "append" else 0x5AA5)
    )
    payload = random.Random(case_seed).randbytes(payload_size)
    case_name = f"{mode}-payload-{payload_size}-rep-{repetition}"
    data_path = Path(args.data_root) / f"wal-sync-{case_name}.bin"
    flags = os.O_CREAT | os.O_EXCL | os.O_RDWR
    file_descriptor = os.open(data_path, flags, 0o600)
    try:
        if mode == "ring":
            prefill_ring(file_descriptor, args.ring_bytes)
        ring_offset = 0
        for _ in range(args.warmup_iterations):
            _, _, _, ring_offset = timed_cycle(
                file_descriptor, mode, payload, args.ring_bytes, ring_offset
            )
        initial_size_bytes = os.fstat(file_descriptor).st_size
        write_latencies = []
        sync_latencies = []
        cycle_latencies = []
        for _ in range(args.iterations):
            write_ns, sync_ns, cycle_ns, ring_offset = timed_cycle(
                file_descriptor, mode, payload, args.ring_bytes, ring_offset
            )
            write_latencies.append(write_ns)
            sync_latencies.append(sync_ns)
            cycle_latencies.append(cycle_ns)
        final_size_bytes = os.fstat(file_descriptor).st_size
        os.fdatasync(file_descriptor)
        case = {
            "record_type": "wal_sync_payload_storage_io_diagnostic",
            "interpretation_scope": "storage IO diagnostic; does not prove engine or WAL architecture behavior",
            "mode": mode,
            "payload_bytes": payload_size,
            "iterations": args.iterations,
            "warmup_iterations": args.warmup_iterations,
            "repetition": repetition,
            "seed": args.seed,
            "case_seed": case_seed,
            "ring_bytes": args.ring_bytes if mode == "ring" else None,
            "ring_prefill_bytes": args.ring_bytes if mode == "ring" else 0,
            "initial_size_bytes": initial_size_bytes,
            "final_size_bytes": final_size_bytes,
            "measured_payload_bytes": payload_size * args.iterations,
            "data_path": str(data_path),
            "write_timer_contract": "fstat plus full pwrite loop",
            "sync_timer_contract": "fdatasync",
            "cycle_timer_contract": "write timer plus fdatasync",
            "payload_generated_before_timing": True,
            "payload_sha256": hashlib.sha256(payload).hexdigest(),
            "environment": environment,
        }
        case.update(summarize(write_latencies, "write"))
        case.update(summarize(sync_latencies, "sync"))
        case.update(summarize(cycle_latencies, "cycle"))
        return case
    finally:
        os.close(file_descriptor)


def case_order(args):
    for repetition in range(args.repetitions):
        for mode_index in range(len(args.mode_names)):
            mode = args.mode_names[(mode_index + repetition) % len(args.mode_names)]
            sizes = args.payload_sizes
            rotation = (repetition + mode_index) % len(sizes)
            ordered_sizes = sizes[rotation:] + sizes[:rotation]
            for payload_size in ordered_sizes:
                yield mode, payload_size, repetition


def write_results(results_directory, records, metadata):
    raw_path = results_directory / "raw.jsonl"
    with raw_path.open("x", encoding="utf-8") as output:
        for record in records:
            output.write(json.dumps(record, sort_keys=True, separators=(",", ":")))
            output.write("\n")
        output.flush()
        os.fsync(output.fileno())
    metadata_path = results_directory / "environment.json"
    with metadata_path.open("x", encoding="utf-8") as output:
        json.dump(metadata, output, sort_keys=True, indent=2)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())
    summary_path = results_directory / "summary.csv"
    columns = list(records[0].keys())
    columns.remove("environment")
    with summary_path.open("x", encoding="utf-8", newline="") as output:
        writer = csv.DictWriter(output, fieldnames=columns, extrasaction="ignore")
        writer.writeheader()
        for record in records:
            writer.writerow(record)
        output.flush()
        os.fsync(output.fileno())
    manifest_paths = [raw_path, summary_path, metadata_path, Path(__file__).resolve()]
    manifest_path = results_directory / "SHA256SUMS"
    with manifest_path.open("x", encoding="utf-8") as output:
        for path in manifest_paths:
            output.write(f"{sha256_file(path)}  {path.name if path.parent == results_directory else 'wal_sync_payload_probe.py'}\n")
        output.flush()
        os.fsync(output.fileno())


def main():
    args = parse_arguments()
    data_root = Path(args.data_root).resolve()
    results_directory = Path(args.results).resolve()
    data_root.mkdir(parents=True, exist_ok=True)
    results_directory.parent.mkdir(parents=True, exist_ok=True)
    results_directory.mkdir(exist_ok=False)
    repository = repository_root()
    script_path = Path(__file__).resolve()
    script_sha_start = sha256_file(script_path)
    state_start = git_state(repository)
    environment_start = {
        "hostname": socket.gethostname(),
        "cpu": cpu_description(),
        "python_version": sys.version,
        "zfs": zfs_description(data_root),
        "source_git_commit_before": state_start["commit"],
        "source_git_status_before": state_start["status_porcelain"],
        "source_git_status_error_before": state_start["status_error"],
        "probe_script_sha256_before": script_sha_start,
        "data_root": str(data_root),
        "results_directory": str(results_directory),
    }
    records = []
    for mode, payload_size, repetition in case_order(args):
        print(
            f"case mode={mode} payload_bytes={payload_size} repetition={repetition}",
            flush=True,
        )
        records.append(run_case(args, mode, payload_size, repetition, environment_start))
    state_end = git_state(repository)
    script_sha_end = sha256_file(script_path)
    metadata = {
        "source_git_commit_before": state_start["commit"],
        "source_git_commit_after": state_end["commit"],
        "source_git_status_before": state_start["status_porcelain"],
        "source_git_status_after": state_end["status_porcelain"],
        "source_git_status_error_before": state_start["status_error"],
        "source_git_status_error_after": state_end["status_error"],
        "source_commit_stable": state_start["commit"] == state_end["commit"],
        "probe_script_sha256_before": script_sha_start,
        "probe_script_sha256_after": script_sha_end,
        "probe_script_stable": script_sha_start == script_sha_end,
        "hostname": environment_start["hostname"],
        "cpu": environment_start["cpu"],
        "python_version": sys.version,
        "zfs": environment_start["zfs"],
        "arguments": {
            "payload_bytes": args.payload_sizes,
            "iterations": args.iterations,
            "warmup_iterations": args.warmup_iterations,
            "repetitions": args.repetitions,
            "modes": args.mode_names,
            "ring_bytes": args.ring_bytes,
            "seed": args.seed,
        },
        "case_count": len(records),
        "interpretation_scope": "storage IO diagnostic; does not prove engine or WAL architecture behavior",
    }
    environment_end = dict(environment_start)
    environment_end.update(
        {
            "source_git_commit_after": state_end["commit"],
            "source_git_status_after": state_end["status_porcelain"],
            "probe_script_sha256_after": script_sha_end,
        }
    )
    for record in records:
        record["environment"] = environment_end
    write_results(results_directory, records, metadata)
    print(f"results={results_directory}", flush=True)


if __name__ == "__main__":
    main()
