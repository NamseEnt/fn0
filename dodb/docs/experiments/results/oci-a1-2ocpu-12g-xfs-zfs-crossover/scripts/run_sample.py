import argparse
import datetime
import json
import os
from pathlib import Path
import subprocess
import time


DEVICE_NAME = "sda"
MOUNT_DIRECTORY = Path("/bench/btrfs/db")
WORKING_DIRECTORY = MOUNT_DIRECTORY / "xfs-zfs-crossover-working"
RESULT_ROOT = Path("/dev/shm/xfs-zfs-crossover/raw")
BINARY_PATH = Path("/var/oled/xfs-zfs-crossover/target/release/phase0-bench")
SOURCE_DIRECTORY = Path("/var/oled/xfs-zfs-crossover/source")
SOURCE_COMMIT = "08e82ec484bd62aeb659471a39f317b86cfb488f"
BINARY_SHA256 = "3756eac0557bbe0d87ba3b2ea46ed088df46aa7e8d79fae56f570d8eb52469b6"
FILE_SIZE_BYTES = 268435456
PAYLOAD_SIZES = (4096, 16384, 65536, 262144, 1048576)
TRANSACTION_WIDTHS = (1, 2, 4, 8, 16)


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


def stage_result_directory(stage, kind):
    directory = RESULT_ROOT / stage / kind
    directory.mkdir(parents=True, exist_ok=True)
    return directory


def prepare_stage(stage):
    if stage not in ("XFS-pre", "ZFS", "XFS-post"):
        raise ValueError(f"unsupported stage: {stage}")
    WORKING_DIRECTORY.mkdir(parents=True, exist_ok=True)
    for payload_size in PAYLOAD_SIZES:
        prepare_payload(payload_size)
    print(f"prepared {stage}: five fresh {FILE_SIZE_BYTES}-byte fio files")


def prepare_payload(payload_size):
    if payload_size < 4096 or payload_size > 1048576 or payload_size % 4096:
        raise ValueError(f"unsupported payload size: {payload_size}")
    WORKING_DIRECTORY.mkdir(parents=True, exist_ok=True)
    fio_path = WORKING_DIRECTORY / f"fio-payload-{payload_size}.dat"
    if fio_path.exists():
        raise FileExistsError(f"expected a fresh fio file: {fio_path}")
    command = ["fallocate", "-l", str(FILE_SIZE_BYTES), str(fio_path)]
    subprocess.run(command, check=True)
    print(f"prepared fresh {FILE_SIZE_BYTES}-byte file for {payload_size}-byte payload")


def build_fio_command(payload_size, output_path):
    fio_path = WORKING_DIRECTORY / f"fio-payload-{payload_size}.dat"
    return [
        "fio",
        "--name=wal-payload-fdatasync",
        f"--filename={fio_path}",
        "--rw=write",
        f"--bs={payload_size}",
        "--ioengine=psync",
        "--iodepth=1",
        "--direct=0",
        f"--size={FILE_SIZE_BYTES}",
        "--time_based=1",
        "--runtime=20",
        "--fdatasync=1",
        "--group_reporting=1",
        "--allow_file_create=0",
        "--output-format=json",
        f"--output={output_path}",
    ]


def build_dodb_command(width, seed, output_path):
    command = [
        str(BINARY_PATH),
        "--suite",
        "write",
        "--engine",
        "parallel-blink",
        "--writers",
        "64",
        "--widths",
        str(width),
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
        str(seed),
        "--output",
        str(output_path),
    ]
    return command


def run_sample(arguments):
    if arguments.kind == "fio":
        if arguments.payload_size < 4096 or arguments.payload_size > 1048576 or arguments.payload_size % 4096:
            raise ValueError(f"unsupported payload size: {arguments.payload_size}")
        stem = f"{arguments.stage.lower()}-payload{arguments.payload_size}-rep{arguments.repetition}"
        directory = stage_result_directory(arguments.stage, "fio")
        output_suffix = "json"
        command = build_fio_command(arguments.payload_size, directory / f"{stem}.{output_suffix}")
        environment = os.environ.copy()
    else:
        if arguments.width not in TRANSACTION_WIDTHS:
            raise ValueError(f"unsupported transaction width: {arguments.width}")
        stem = f"{arguments.stage.lower()}-width{arguments.width}-rep{arguments.repetition}"
        directory = stage_result_directory(arguments.stage, "dodb")
        output_suffix = "jsonl"
        output_path = directory / f"{stem}.{output_suffix}"
        command = build_dodb_command(arguments.width, arguments.seed, output_path)
        environment = os.environ.copy()
        environment["DODB_BENCH_DIR"] = str(WORKING_DIRECTORY / "data")

    output_path = directory / f"{stem}.{output_suffix}"
    log_path = directory / f"{stem}.log"
    metadata_path = directory / f"{stem}.meta.json"
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
        "stage": arguments.stage,
        "filesystem": "XFS" if arguments.stage.startswith("XFS") else "ZFS",
        "kind": arguments.kind,
        "payload_size_bytes": arguments.payload_size,
        "transaction_width": arguments.width,
        "repetition": arguments.repetition,
        "seed": arguments.seed if arguments.kind == "dodb" else None,
        "started_at": started_at,
        "ended_at": ended_at,
        "elapsed_ms": elapsed_nanos / 1_000_000,
        "source_commit": SOURCE_COMMIT,
        "binary_sha256": BINARY_SHA256,
        "command": command,
        "exit_code": completed.returncode,
        "block_device": DEVICE_NAME,
        "block_stats_before": stats_before,
        "block_stats_after": stats_after,
        "block_stats_delta": {
            key: stats_after[key] - stats_before[key] for key in stats_before
        },
        "written_bytes": (stats_after["written_sectors"] - stats_before["written_sectors"]) * 512,
        "device_busy_ms": stats_after["io_busy_ms"] - stats_before["io_busy_ms"],
        "output_path": str(output_path),
        "log_path": str(log_path),
    }
    metadata_path.write_text(json.dumps(metadata, indent=2, sort_keys=True) + "\n")
    if completed.returncode != 0:
        raise subprocess.CalledProcessError(completed.returncode, command)
    if not output_path.is_file() or output_path.stat().st_size == 0:
        raise RuntimeError(f"empty benchmark output: {output_path}")
    print(metadata_path)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("prepare", "prepare-payload", "run"))
    parser.add_argument("stage", choices=("XFS-pre", "ZFS", "XFS-post"))
    parser.add_argument("kind", nargs="?", choices=("fio", "dodb"))
    parser.add_argument("value", nargs="?", type=int)
    parser.add_argument("repetition", nargs="?", type=int)
    parser.add_argument("seed", nargs="?", type=int, default=0)
    arguments = parser.parse_args()
    if arguments.action == "prepare":
        prepare_stage(arguments.stage)
        return
    if arguments.action == "prepare-payload":
        if arguments.value is None:
            parser.error("prepare-payload requires a payload size")
        prepare_payload(arguments.value)
        return
    if arguments.kind == "fio":
        arguments.payload_size = arguments.value
        arguments.width = None
    elif arguments.kind == "dodb":
        arguments.payload_size = None
        arguments.width = arguments.value
    else:
        parser.error("run requires a benchmark kind")
    if arguments.value is None or arguments.repetition is None:
        parser.error("run requires a value and repetition")
    run_sample(arguments)


if __name__ == "__main__":
    main()
