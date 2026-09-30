import datetime
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import time


BASE = pathlib.Path("/bench/btrfs/db/oci-a1-scaling-20260930")
RAW = BASE / "raw"
DATA = BASE / "data"
UUID = "be3cac1e-6e47-445c-96d4-e5cf7278c97a"
MOUNT_POINT = "/bench/btrfs/db"
MOUNT_OPTIONS = "rw,noatime,seclabel,discard=async,space_cache=v2,subvolid=5,subvol=/"
SOURCE_SHA = "08e82ec484bd62aeb659471a39f317b86cfb488f"
ROCKSDB_SHA = "abeebd9630f11bd08c28b7bd43c7bdfc62050654"
DODB_BINARY = BASE / "target/release/phase0-bench"
ROCKSDB_BINARY = BASE / "rocksdb-target/release/rocksdb-bench"
RESULTS = RAW / "run-events.jsonl"
FIO_FILE = BASE / "fio/wal-like.dat"
FIO_SIZE = 256 * 1024 * 1024
BASE_SEED = 979_000_000
SEED_STRIDE = 1_009
WORKLOADS = (
    (0, "width1-uniform", 1, "uniform"),
    (1, "width16-uniform", 16, "uniform"),
    (2, "width16-compact", 16, "same-leaf-heavy"),
    (3, "width16-spread", 16, "different-leaf-heavy"),
)


def utc_now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def append_event(event):
    event["timestamp"] = utc_now()
    with RESULTS.open("a", encoding="utf-8") as result_file:
        result_file.write(json.dumps(event, sort_keys=True) + "\n")
    print(json.dumps(event, sort_keys=True), flush=True)


def output(command):
    return subprocess.run(command, check=True, capture_output=True, text=True).stdout.strip()


def write_probe(directory):
    probe_path = directory / ".btrfs-write-probe"
    file_descriptor = os.open(probe_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        os.write(file_descriptor, b"probe")
        os.fdatasync(file_descriptor)
    finally:
        os.close(file_descriptor)
    probe_path.unlink()
    directory_descriptor = os.open(directory, os.O_RDONLY)
    try:
        os.fsync(directory_descriptor)
    finally:
        os.close(directory_descriptor)


def capture_environment(stage, expected_ocpus):
    mount = output(["findmnt", "-n", "-o", "SOURCE,FSTYPE,TARGET,OPTIONS", MOUNT_POINT])
    source, filesystem, target, options = mount.split(maxsplit=3)
    uuid = output(["sudo", "-n", "blkid", "-s", "UUID", "-o", "value", source])
    logical_cpus = int(output(["nproc"]))
    affinity = sorted(os.sched_getaffinity(0))
    if uuid != UUID or filesystem != "btrfs" or target != MOUNT_POINT or options != MOUNT_OPTIONS:
        raise RuntimeError(f"Btrfs identity changed: {mount} UUID={uuid}")
    if logical_cpus != expected_ocpus or len(affinity) != expected_ocpus:
        raise RuntimeError(f"guest CPU mismatch: nproc={logical_cpus} affinity={affinity}")
    write_probe(BASE)
    snapshots = {
        "lsblk": output(["lsblk", "-f"]),
        "findmnt": output(["findmnt", "-no", "SOURCE,FSTYPE,TARGET,OPTIONS", MOUNT_POINT]),
        "btrfs_usage": output(["sudo", "-n", "btrfs", "filesystem", "usage", "-b", MOUNT_POINT]),
        "kernel": output(["uname", "-a"]),
        "lscpu": output(["lscpu"]),
        "cpuinfo": pathlib.Path("/proc/cpuinfo").read_text(encoding="utf-8"),
        "cpu_affinity": affinity,
        "nproc": logical_cpus,
        "fio_version": output(["fio", "--version"]),
        "filesystem_uuid": uuid,
        "block_device": source,
        "mount_point": target,
        "mount_options": options,
    }
    snapshot_path = RAW / f"environment-{stage}.json"
    snapshot_path.write_text(json.dumps(snapshots, indent=2) + "\n", encoding="utf-8")
    append_event({"event": "preflight", "stage": stage, "expected_ocpus": expected_ocpus, "snapshot": str(snapshot_path), "filesystem_uuid": uuid, "mount": mount, "nproc": logical_cpus, "cpu_affinity": affinity, "write_probe": "passed"})
    return snapshots


def block_stats(device):
    device_name = pathlib.Path(device).name
    stat_path = pathlib.Path("/sys/class/block") / device_name / "stat"
    fields = [int(value) for value in stat_path.read_text(encoding="utf-8").split()]
    return {"read_sectors": fields[2], "written_sectors": fields[6], "io_busy_ms": fields[9]}


def run(command, environment, output_path, stage, name, metadata):
    output_path.parent.mkdir(parents=True, exist_ok=True)
    existing_result = pathlib.Path(metadata["json_output"]) if metadata.get("json_output") else None
    if existing_result and existing_result.exists():
        existing_result.unlink()
    mount = output(["findmnt", "-n", "-o", "SOURCE", MOUNT_POINT])
    before_stats = block_stats(mount)
    started = time.monotonic()
    started_utc = utc_now()
    append_event({"event": "run_start", "stage": stage, "name": name, "started_at": started_utc, "command": command, "metadata": metadata, "block_stats_before": before_stats})
    with output_path.open("w", encoding="utf-8") as log_file:
        process = subprocess.run(command, env=environment, cwd=metadata.get("cwd"), stdout=log_file, stderr=subprocess.STDOUT)
    elapsed = time.monotonic() - started
    ended_utc = utc_now()
    after_stats = block_stats(mount)
    delta = {
        "disk_bytes_written": (after_stats["written_sectors"] - before_stats["written_sectors"]) * 512,
        "block_device_busy_ms": after_stats["io_busy_ms"] - before_stats["io_busy_ms"],
    }
    result_file = pathlib.Path(metadata["json_output"]) if metadata.get("json_output") else None
    record = {"event": "run_end", "stage": stage, "name": name, "started_at": started_utc, "ended_at": ended_utc, "elapsed_seconds": elapsed, "exit_code": process.returncode, "log": str(output_path), "block_stats_after": after_stats, **delta}
    if result_file and result_file.is_file():
        record["json_output"] = str(result_file)
        record["json_sha256"] = hashlib.sha256(result_file.read_bytes()).hexdigest()
    append_event(record)
    if process.returncode:
        raise RuntimeError(f"run failed: {name} exit={process.returncode}")
    if result_file and result_file.is_file() and metadata.get("engine") in {"parallel-blink", "rocksdb"}:
        rows = [json.loads(line) for line in result_file.read_text(encoding="utf-8").splitlines() if line.strip()]
        if len(rows) != 1:
            raise RuntimeError(f"expected one JSON result row for {name}, got {len(rows)}")
        row = rows[0]
        if metadata["engine"] == "parallel-blink":
            expected = {"git_commit": SOURCE_SHA, "engine": "parallel-blink", "sync_mode": "real"}
            actual = {key: row.get(key) for key in expected}
            if actual != expected:
                raise RuntimeError(f"dodb provenance/contract mismatch for {name}: {actual}")
        elif metadata["engine"] == "rocksdb":
            build = row.get("build", {})
            verification = row.get("verification", {})
            if build.get("commit") != ROCKSDB_SHA or build.get("tag") != "v11.8.1":
                raise RuntimeError(f"RocksDB build provenance mismatch for {name}: {build}")
            if row.get("sync_contract") != "durable-return" or verification.get("passed") is not True:
                raise RuntimeError(f"RocksDB durability/verification mismatch for {name}")
    if metadata.get("cleanup_data"):
        cleanup_path = pathlib.Path(metadata["cleanup_data"])
        if cleanup_path.exists():
            shutil.rmtree(cleanup_path)


def run_fio(stage):
    if not FIO_FILE.exists():
        file_descriptor = os.open(FIO_FILE, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        try:
            os.posix_fallocate(file_descriptor, 0, FIO_SIZE)
            os.fdatasync(file_descriptor)
        finally:
            os.close(file_descriptor)
        append_event({"event": "fio_file_preconditioned", "stage": stage, "path": str(FIO_FILE), "size_bytes": FIO_SIZE})
    json_output = RAW / f"fio-{stage}.json"
    log_output = RAW / f"fio-{stage}.log"
    command = [
        "fio", "--name=wal-like-fdatasync", f"--filename={FIO_FILE}", "--rw=write",
        "--bs=4k", "--ioengine=psync", f"--size={FIO_SIZE}", "--time_based=1",
        "--runtime=15", "--fdatasync=1", "--group_reporting=1", "--output-format=json",
        f"--output={json_output}",
    ]
    run(command, os.environ.copy(), log_output, stage, f"fio-{stage}", {"json_output": str(json_output), "file": str(FIO_FILE), "file_size_bytes": FIO_SIZE})


def dodb_command(stage, workload, repetition, expected_ocpus):
    scenario_index, workload_name, width, distribution = workload
    seed = BASE_SEED + scenario_index * SEED_STRIDE + repetition
    name = f"{stage}-dodb-{workload_name}-rep{repetition + 1}"
    data_directory = DATA / name
    json_output = RAW / f"{name}.jsonl"
    command = [
        str(DODB_BINARY), "--suite", "write", "--engine", "parallel-blink", "--writers", "64",
        "--widths", str(width), "--distributions", distribution, "--duration", "5s",
        "--warmup", "2s", "--repetitions", "1", "--cache-capacity", "256",
        "--working-set", "100000", "--key-size", "16", "--value-size", "64",
        "--group-limit", "64", "--group-bytes", "4194304", "--queue-capacity", "256",
        "--collection-delay", "0us", "--sync-mode", "real", "--tokio-workers", str(expected_ocpus),
        "--blink-workers", str(expected_ocpus), "--transaction-mode", "unconditional",
        "--seed", str(seed), "--output", str(json_output),
    ]
    environment = os.environ.copy()
    environment["DODB_BENCH_DIR"] = str(data_directory)
    environment["TMPDIR"] = str(BASE / "tmp")
    metadata = {"json_output": str(json_output), "cleanup_data": str(data_directory), "cwd": str(BASE / "fn0"), "engine": "parallel-blink", "git_commit": SOURCE_SHA, "seed": seed, "width": width, "distribution": distribution, "working_set": 100000, "writers": 64, "value_size": 64, "sync_mode": "real", "tokio_workers": expected_ocpus, "blink_workers": expected_ocpus}
    return name, command, environment, metadata


def rocksdb_command(stage, workload, repetition):
    scenario_index, workload_name, width, distribution = workload
    seed = BASE_SEED + scenario_index * SEED_STRIDE + repetition
    name = f"{stage}-rocksdb-{workload_name}-rep{repetition + 1}"
    data_directory = DATA / name
    json_output = RAW / f"{name}.jsonl"
    command = [
        str(ROCKSDB_BINARY), "--mode", "bench", "--writers", "64", "--width", str(width),
        "--distribution", distribution, "--working-set", "100000", "--key-size", "16",
        "--value-size", "64", "--warmup-ms", "2000", "--duration-ms", "5000",
        "--seed", str(seed), "--scenario-index", str(scenario_index), "--repetition", str(repetition + 1),
        "--data-dir", str(data_directory), "--output", str(json_output), "--pipelined", "off",
    ]
    metadata = {"json_output": str(json_output), "cleanup_data": str(data_directory), "engine": "rocksdb", "rocksdb_commit": ROCKSDB_SHA, "seed": seed, "width": width, "distribution": distribution, "working_set": 100000, "writers": 64, "value_size": 64, "sync_mode": "WriteOptions.sync=true"}
    return name, command, os.environ.copy(), metadata


def run_matrix(stage, expected_ocpus):
    workloads = WORKLOADS if stage != "2-post" else WORKLOADS[:2]
    for workload in workloads:
        scenario_index = workload[0]
        for repetition in range(3):
            engines = ("dodb", "rocksdb")
            if (scenario_index + repetition) % 2:
                engines = tuple(reversed(engines))
            for engine in engines:
                if engine == "dodb":
                    name, command, environment, metadata = dodb_command(stage, workload, repetition, expected_ocpus)
                else:
                    name, command, environment, metadata = rocksdb_command(stage, workload, repetition)
                run(command, environment, RAW / f"{name}.log", stage, name, metadata)


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: run_stage.py <2|4|6|2-post> <expected-ocpus>")
    stage = sys.argv[1]
    expected_ocpus = int(sys.argv[2])
    if stage not in {"2", "4", "6", "2-post"} or expected_ocpus not in {2, 4, 6}:
        raise SystemExit("invalid stage or OCPU count")
    if not DODB_BINARY.is_file() or not ROCKSDB_BINARY.is_file():
        raise RuntimeError("release benchmark binary is missing")
    capture_environment(stage, expected_ocpus)
    run_fio(stage)
    run_matrix(stage, expected_ocpus)
    append_event({"event": "stage_complete", "stage": stage, "ocpus": expected_ocpus})


if __name__ == "__main__":
    main()
