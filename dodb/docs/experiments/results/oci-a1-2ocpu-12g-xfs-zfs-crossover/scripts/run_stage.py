import argparse
import csv
import json
from pathlib import Path
import subprocess
import sys


RESULT_DIRECTORY = Path(__file__).resolve().parents[1]
REMOTE_HOST = "opc@217.142.246.204"
SSH_KEY = Path.home() / "Downloads/ssh-key-2026-09-23.key"
REMOTE_HELPER = "/var/oled/xfs-zfs-crossover/run_sample.py"
REMOTE_RESULT_ROOT = "/dev/shm/xfs-zfs-crossover/raw"
SSH_BASE = ["ssh", "-i", str(SSH_KEY), "-o", "IdentitiesOnly=yes", REMOTE_HOST]
SCP_BASE = ["scp", "-i", str(SSH_KEY), "-o", "IdentitiesOnly=yes"]
PAYLOAD_SIZES = (4096, 16384, 65536, 262144, 1048576)
TRANSACTION_WIDTHS = (1, 2, 4, 8, 16)


def run_remote(arguments):
    completed = subprocess.run(
        SSH_BASE + ["python3", REMOTE_HELPER, *map(str, arguments)],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if completed.returncode != 0:
        sys.stderr.write(completed.stderr)
        sys.stderr.write(completed.stdout)
        raise subprocess.CalledProcessError(completed.returncode, completed.args)
    if completed.stdout.strip():
        print(completed.stdout.strip())


def copy_stage(stage):
    stage_path = f"{REMOTE_RESULT_ROOT}/{stage}"
    ssh_process = subprocess.Popen(
        SSH_BASE + ["tar", "-C", stage_path, "-czf", "-", "fio", "dodb"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    extraction = subprocess.run(
        ["tar", "-xzf", "-", "-C", str(RESULT_DIRECTORY)],
        stdin=ssh_process.stdout,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    ssh_process.stdout.close()
    ssh_error = ssh_process.stderr.read()
    ssh_return_code = ssh_process.wait()
    if extraction.returncode != 0 or ssh_return_code != 0:
        sys.stderr.buffer.write(extraction.stderr)
        sys.stderr.buffer.write(ssh_error)
        raise RuntimeError(f"failed to copy stage results for {stage}")
    metadata_count = sum(1 for path in (RESULT_DIRECTORY / "fio").glob(f"{stage.lower()}-*.meta.json"))
    metadata_count += sum(1 for path in (RESULT_DIRECTORY / "dodb").glob(f"{stage.lower()}-*.meta.json"))
    if metadata_count != 50:
        raise RuntimeError(f"expected 50 metadata files for {stage}, found {metadata_count}")


def write_run_order(stage):
    order_path = RESULT_DIRECTORY / "run-order.csv"
    rows = []
    if order_path.is_file():
        with order_path.open(newline="") as file_handle:
            rows.extend(csv.DictReader(file_handle))
    metadata_paths = list((RESULT_DIRECTORY / "fio").glob(f"{stage.lower()}-*.meta.json"))
    metadata_paths.extend((RESULT_DIRECTORY / "dodb").glob(f"{stage.lower()}-*.meta.json"))
    metadata = [json.loads(path.read_text()) for path in metadata_paths]
    started_at = min(item["started_at"] for item in metadata)
    ended_at = max(item["ended_at"] for item in metadata)
    rows.append(
        {
            "order": len(rows) + 1,
            "stage": stage,
            "filesystem": "XFS" if stage.startswith("XFS") else "ZFS",
            "started_at": started_at,
            "ended_at": ended_at,
            "fio_runs": 25,
            "dodb_runs": 25,
        }
    )
    with order_path.open("w", newline="") as file_handle:
        writer = csv.DictWriter(
            file_handle,
            fieldnames=("order", "stage", "filesystem", "started_at", "ended_at", "fio_runs", "dodb_runs"),
        )
        writer.writeheader()
        writer.writerows(rows)


def run_stage(stage):
    run_remote(["prepare", stage])
    for payload_size in PAYLOAD_SIZES:
        for repetition in range(1, 6):
            run_remote(["run", stage, "fio", payload_size, repetition])
        print(f"{stage}: fio payload {payload_size} bytes complete", flush=True)
    for width in TRANSACTION_WIDTHS:
        for repetition in range(1, 6):
            seed = 979200000 + width * 100 + repetition
            run_remote(["run", stage, "dodb", width, repetition, seed])
        print(f"{stage}: dodb width {width} complete", flush=True)
    copy_stage(stage)
    write_run_order(stage)
    print(f"{stage}: all 50 runs copied locally", flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("stage", choices=("XFS-pre", "ZFS", "XFS-post"))
    arguments = parser.parse_args()
    if not SSH_KEY.is_file():
        raise FileNotFoundError(f"SSH identity is missing: {SSH_KEY}")
    run_stage(arguments.stage)


if __name__ == "__main__":
    main()
