import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path("/bench/zfs/db/blink-background-checkpoint-20261022-threshold")
SAMPLE_INTERVAL_SECONDS = 1.0
MAX_PROCESS_GROUP_RSS_BYTES = 3 * 1024**3
MINIMUM_MEM_AVAILABLE_BYTES = 1500 * 1024**2
MINIMUM_DISK_AVAILABLE_BYTES = 10 * 1024**3


def process_group_rss_bytes(process_group):
    total_bytes = 0
    for process_path in Path("/proc").iterdir():
        if not process_path.name.isdigit():
            continue
        try:
            fields = (process_path / "stat").read_text().rsplit(")", 1)[1].split()
            if int(fields[2]) != process_group:
                continue
            status_lines = (process_path / "status").read_text().splitlines()
            resident_kb = next(
                int(line.split()[1])
                for line in status_lines
                if line.startswith("VmRSS:")
            )
            total_bytes += resident_kb * 1024
        except (OSError, StopIteration, ValueError, IndexError):
            continue
    return total_bytes


def memory_available_bytes():
    for line in Path("/proc/meminfo").read_text().splitlines():
        if line.startswith("MemAvailable:"):
            return int(line.split()[1]) * 1024
    raise RuntimeError("MemAvailable is missing from /proc/meminfo")


def filesystem_available_bytes():
    return os.statvfs(ROOT).f_bavail * os.statvfs(ROOT).f_frsize


def main():
    ROOT.mkdir(parents=True, exist_ok=True)
    run_id = sys.argv[1]
    command = sys.argv[2:]
    run_root = ROOT / "builds" / run_id
    run_root.mkdir(parents=True, exist_ok=False)
    command_record = {"argv": command, "cwd": os.getcwd(), "run_id": run_id}
    (run_root / "command.json").write_text(json.dumps(command_record, indent=2) + "\n")
    initial_memory = memory_available_bytes()
    initial_disk = filesystem_available_bytes()
    if initial_memory < MINIMUM_MEM_AVAILABLE_BYTES:
        raise RuntimeError("host MemAvailable is below the configured reserve")
    if initial_disk < MINIMUM_DISK_AVAILABLE_BYTES:
        raise RuntimeError("filesystem free space is below the configured reserve")
    start_time = time.monotonic()
    with (run_root / "stdout-stderr.log").open("wb") as output_file:
        child = subprocess.Popen(
            command,
            stdout=output_file,
            stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        samples = []
        stop_reason = None
        while child.poll() is None:
            sample = {
                "elapsed_seconds": time.monotonic() - start_time,
                "process_group_rss_bytes": process_group_rss_bytes(child.pid),
                "mem_available_bytes": memory_available_bytes(),
                "filesystem_available_bytes": filesystem_available_bytes(),
            }
            samples.append(sample)
            if sample["process_group_rss_bytes"] >= MAX_PROCESS_GROUP_RSS_BYTES:
                stop_reason = "process-group RSS ceiling"
            elif sample["mem_available_bytes"] < MINIMUM_MEM_AVAILABLE_BYTES:
                stop_reason = "host MemAvailable reserve"
            elif sample["filesystem_available_bytes"] < MINIMUM_DISK_AVAILABLE_BYTES:
                stop_reason = "filesystem free-space reserve"
            if stop_reason is not None:
                os.killpg(child.pid, signal.SIGTERM)
                try:
                    child.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid, signal.SIGKILL)
                    child.wait()
                break
            time.sleep(SAMPLE_INTERVAL_SECONDS)
        exit_code = child.wait()
    samples.append(
        {
            "elapsed_seconds": time.monotonic() - start_time,
            "process_group_rss_bytes": process_group_rss_bytes(child.pid),
            "mem_available_bytes": memory_available_bytes(),
            "filesystem_available_bytes": filesystem_available_bytes(),
        }
    )
    (run_root / "resource-samples.json").write_text(json.dumps(samples, indent=2) + "\n")
    result = {
        "exit_code": exit_code,
        "stop_reason": stop_reason,
        "elapsed_seconds": time.monotonic() - start_time,
        "sample_count": len(samples),
        "peak_process_group_rss_bytes": max(sample["process_group_rss_bytes"] for sample in samples),
        "minimum_mem_available_bytes": min(sample["mem_available_bytes"] for sample in samples),
        "minimum_filesystem_available_bytes": min(sample["filesystem_available_bytes"] for sample in samples),
    }
    (run_root / "controller-result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
    if stop_reason is not None or exit_code != 0:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
