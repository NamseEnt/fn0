import argparse
import datetime
import json
import os
from pathlib import Path
import subprocess
import sys
import time


DEVICE_NAME = "sda"
MOUNT_DIRECTORY = Path("/bench/btrfs/db")
WORKING_DIRECTORY = MOUNT_DIRECTORY / "fsperf-working"
FIO_FILE = WORKING_DIRECTORY / "fio.dat"
RESULT_DIRECTORY = Path("/dev/shm/dodb-filesystem-performance/raw")
BINARY_PATH = Path("/home/opc/phase0-bench-filesystem-matrix")
SOURCE_DIRECTORY = Path("/home/opc/phase0-bench-source")


def read_device_stats():
    for line in Path("/proc/diskstats").read_text().splitlines():
        fields = line.split()
        if fields[2] == DEVICE_NAME:
            values = [int(value) for value in fields[3:]]
            return {
                "read_sectors": values[2],
                "written_sectors": values[6],
                "io_busy_ms": values[9],
            }
    raise RuntimeError(f"{DEVICE_NAME} is missing from /proc/diskstats")


def build_command(arguments, output_path):
    if arguments.kind == "fio-seq" or arguments.kind == "fio-rand":
        write_mode = "write" if arguments.kind == "fio-seq" else "randwrite"
        command = [
            "fio",
            "--name=wal-like-fdatasync",
            f"--filename={FIO_FILE}",
            f"--rw={write_mode}",
            "--bs=4k",
            "--ioengine=psync",
            "--iodepth=1",
            "--direct=0",
            "--size=268435456",
            "--time_based=1",
            "--runtime=20",
            "--fdatasync=1",
            "--group_reporting=1",
            "--allow_file_create=0",
        ]
        if arguments.kind == "fio-rand":
            command.extend(["--randrepeat=1", "--randseed=20260930", "--norandommap=1"])
        command.extend(["--output-format=json", f"--output={output_path}"])
        return command, os.environ.copy()

    if arguments.kind == "dodb":
        if arguments.width not in (1, 16):
            raise ValueError("dodb width must be 1 or 16")
        environment = os.environ.copy()
        environment["DODB_BENCH_DIR"] = str(WORKING_DIRECTORY / "data")
        command = [
            str(BINARY_PATH),
            "--suite",
            "write",
            "--engine",
            "parallel-blink",
            "--writers",
            "64",
            "--widths",
            str(arguments.width),
            "--distributions",
            "uniform",
            "--duration",
            "5s",
            "--warmup",
            "2s",
            "--repetitions",
            "1",
            "--cache-capacity",
            "256",
            "--working-set",
            "100000",
            "--key-size",
            "16",
            "--value-size",
            "64",
            "--group-limit",
            "64",
            "--group-bytes",
            "4194304",
            "--queue-capacity",
            "256",
            "--collection-delay",
            "0us",
            "--sync-mode",
            "real",
            "--tokio-workers",
            "2",
            "--blink-workers",
            "2",
            "--transaction-mode",
            "unconditional",
            "--seed",
            str(arguments.seed),
            "--output",
            str(output_path),
        ]
        return command, environment

    raise ValueError(f"unsupported run kind: {arguments.kind}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("kind", choices=("fio-seq", "fio-rand", "dodb"))
    parser.add_argument("filesystem")
    parser.add_argument("repetition", type=int)
    parser.add_argument("--width", type=int)
    parser.add_argument("--seed", type=int, default=979000000)
    arguments = parser.parse_args()

    RESULT_DIRECTORY.mkdir(parents=True, exist_ok=True)
    output_stem = arguments.filesystem.lower()
    if arguments.kind == "dodb":
        output_stem = f"{output_stem}-width{arguments.width}"
    output_stem = f"{output_stem}-{arguments.kind}-rep{arguments.repetition}"
    output_suffix = "jsonl" if arguments.kind == "dodb" else "json"
    output_path = RESULT_DIRECTORY / f"{output_stem}.{output_suffix}"
    log_path = RESULT_DIRECTORY / f"{output_stem}.log"
    metadata_path = RESULT_DIRECTORY / f"{output_stem}.meta.json"
    command, environment = build_command(arguments, output_path)
    stats_before = read_device_stats()
    started_at = datetime.datetime.now(datetime.timezone.utc).isoformat()
    started_monotonic = time.monotonic_ns()
    with log_path.open("wb") as log_file:
        completed = subprocess.run(
            command,
            cwd=SOURCE_DIRECTORY,
            env=environment,
            stdout=log_file,
            stderr=subprocess.STDOUT,
        )
    elapsed_nanos = time.monotonic_ns() - started_monotonic
    ended_at = datetime.datetime.now(datetime.timezone.utc).isoformat()
    stats_after = read_device_stats()
    metadata = {
        "filesystem": arguments.filesystem,
        "kind": arguments.kind,
        "repetition": arguments.repetition,
        "width": arguments.width,
        "seed": arguments.seed if arguments.kind == "dodb" else None,
        "started_at": started_at,
        "ended_at": ended_at,
        "elapsed_ms": elapsed_nanos / 1_000_000,
        "command": command,
        "exit_code": completed.returncode,
        "block_device": DEVICE_NAME,
        "block_stats_before": stats_before,
        "block_stats_after": stats_after,
        "block_stats_delta": {
            key: stats_after[key] - stats_before[key] for key in stats_before
        },
        "written_bytes": (stats_after["written_sectors"] - stats_before["written_sectors"]) * 512,
        "log_path": str(log_path),
        "output_path": str(output_path),
    }
    metadata_path.write_text(json.dumps(metadata, indent=2, sort_keys=True) + "\n")
    if completed.returncode != 0:
        print(f"run failed: {metadata_path}", file=sys.stderr)
        return completed.returncode
    print(metadata_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
