import argparse
import importlib.util
import json
import math
import os
import select
import signal
import subprocess
import sys
import time
from pathlib import Path


parser = argparse.ArgumentParser()
parser.add_argument("--root", type=Path, required=True)
parser.add_argument("--stage", choices=["preflight", "main"], required=True)
parser.add_argument("--case")
options = parser.parse_args()
root = options.root.resolve()
source_root = Path("/bench/zfs/db/blink-checkpoint-costs-20261023")
source = source_root / "source"
scripts = Path(__file__).resolve().parent
stage_root = root / options.stage
stage_root.mkdir(parents=True, exist_ok=True)
os.environ.update({"PYTHONDONTWRITEBYTECODE": "1", "DODB_CHECKPOINT_RUN_ROOT": str(stage_root),
                   "DODB_CHECKPOINT_SMOKE": "1" if options.stage == "preflight" else "0"})
helper_path = source / "dodb/docs/experiments/results/blink-background-checkpoint-20261021/run_matrix.py"
specification = importlib.util.spec_from_file_location("checkpoint_helper", helper_path)
helper = importlib.util.module_from_spec(specification)
specification.loader.exec_module(helper)
helper.SOURCE = source
helper.SERVICE_BINARY = source_root / "bin/service_workload"
helper.UNCONDITIONAL_BINARY = source_root / "bin/unconditional_write_workload"
helper.MEASUREMENT_MS = 8000 if options.stage == "preflight" else 60000
helper.WARMUP_MS = 1000 if options.stage == "preflight" else 10000
source_sha = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
assert source_sha == "cc80d1fdc7a98792be1a739056a3013debf39268"
assert not subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip()
identities = json.loads((source_root / "binary-identities.json").read_text())
for name, identity in identities.items():
    assert helper.sha256_file(source_root / "bin" / name) == identity["sha256"]


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def wait_until(path, processes, timeout=70):
    deadline = time.monotonic() + timeout
    while not path.exists():
        for process in processes:
            if process.poll() is not None:
                raise RuntimeError(f"process exited before readiness: {process.args}")
        if time.monotonic() >= deadline:
            raise TimeoutError(str(path))
        time.sleep(0.05)


def terminate(process):
    if process is None or process.poll() is not None:
        return
    subprocess.run(["sudo", "-n", "kill", "-TERM", "--", f"-{process.pid}"], check=False)
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        subprocess.run(["sudo", "-n", "kill", "-KILL", "--", f"-{process.pid}"], check=False)
        process.wait(timeout=10)


def run_case(workload, backend, mode):
    case_id = f"{workload}-{backend}-{mode}"
    if options.case and options.case != case_id:
        return
    case_root = stage_root / case_id
    if (case_root / "accepted.json").exists():
        print(json.dumps({"already_accepted": case_id}), flush=True)
        return
    case_root.mkdir(exist_ok=False)
    database = stage_root / "databases" / case_id
    database.parent.mkdir(exist_ok=True)
    assert not database.exists()
    report_path = case_root / "workload-report.json"
    command = (helper.service_command(backend, database, report_path, "134217728") if workload == "RW80"
               else helper.unconditional_command(backend, database, report_path, "134217728"))
    if workload == "RW80":
        command[command.index("--scenario") + 1] = "rw80"
    if backend == "blink":
        command.extend(["--blink-wal-layout", "segmented-bases-coalesced", "--blink-checkpoint-execution",
                        "background", "--blink-wal-segment-bytes", "33554432"])
    binary_sha = helper.sha256_file(Path(command[0]))
    environment = os.environ.copy()
    environment.update({"DODB_SOURCE_SHA": source_sha, "DODB_BINARY_SHA256": binary_sha})
    write_json(case_root / "command.json", {"argv": command, "source_sha": source_sha, "binary_sha256": binary_sha,
                                           "mode": mode, "cwd": str(source), "epoch_ns": time.time_ns()})
    initial = helper.process_sample(os.getpgrp(), database)
    assert initial["mem_available_bytes"] >= helper.MINIMUM_MEM_AVAILABLE_BYTES, initial
    assert initial["filesystem_available_bytes"] >= helper.MINIMUM_DISK_AVAILABLE_BYTES, initial
    print(json.dumps({"starting": case_id, "stage": options.stage}), flush=True)
    child = None
    cpu = None
    waits = None
    handles = []
    samples = []
    started = time.monotonic()
    try:
        stdout = (case_root / "workload.stdout.log").open("wb")
        stderr = (case_root / "workload.stderr.log").open("wb")
        handles.extend([stdout, stderr])
        child = subprocess.Popen([sys.executable, str(scripts / "gate.py"), str(case_root), *command],
                                 cwd=source, env=environment, stdout=stdout, stderr=stderr, start_new_session=True)
        wait_until(case_root / "workload-pid.json", [child])
        assert json.loads((case_root / "workload-pid.json").read_text())["pid"] == child.pid
        if mode == "profile":
            waits_root = case_root / "waits"
            waits_root.mkdir()
            waits_log = (case_root / "waits.log").open("wb")
            handles.append(waits_log)
            waits_command = ["sudo", "-n", "/usr/bin/python3", str(scripts / "collect_waits.py"),
                             "--pid", str(child.pid), "--output", str(waits_root)]
            waits = subprocess.Popen(waits_command, stdout=waits_log, stderr=waits_log, start_new_session=True)
            wait_until(waits_root / "ready.json", [child, waits])
            control = case_root / "perf-control.fifo"
            acknowledgement = case_root / "perf-ack.fifo"
            os.mkfifo(control)
            os.mkfifo(acknowledgement)
            ack_descriptor = os.open(acknowledgement, os.O_RDONLY | os.O_NONBLOCK)
            perf_log = (case_root / "perf.log").open("wb")
            handles.append(perf_log)
            perf_command = ["sudo", "-n", "perf", "record", "-e", "cpu-clock", "-F", "99", "-k", "CLOCK_REALTIME",
                            "--call-graph", "dwarf,16384", "--max-size", "2G", "-m", "512", "-D", "-1",
                            "--control", f"fifo:{control},{acknowledgement}", "-p", str(child.pid),
                            "-o", str(case_root / "cpu.data")]
            write_json(case_root / "collectors.json", {"cpu": perf_command, "waits": waits_command})
            cpu = subprocess.Popen(perf_command, stdout=perf_log, stderr=perf_log, start_new_session=True)
            deadline = time.monotonic() + 15
            control_descriptor = None
            while control_descriptor is None:
                if cpu.poll() is not None or time.monotonic() >= deadline:
                    raise RuntimeError("perf control FIFO did not open")
                try:
                    control_descriptor = os.open(control, os.O_WRONLY | os.O_NONBLOCK)
                except OSError:
                    time.sleep(0.05)
            os.write(control_descriptor, b"enable\n")
            acknowledgement_bytes = b""
            while b"ack" not in acknowledgement_bytes:
                if cpu.poll() is not None or time.monotonic() >= deadline:
                    raise RuntimeError("perf did not acknowledge enable")
                if select.select([ack_descriptor], [], [], 0.1)[0]:
                    acknowledgement_bytes += os.read(ack_descriptor, 128)
            os.close(control_descriptor)
            os.close(ack_descriptor)
        release_ns = time.time_ns()
        (case_root / "release-workload").write_text(str(release_ns))
        while child.poll() is None:
            sample = helper.process_sample(child.pid, database)
            sample.update({"epoch_ns": time.time_ns(), "monotonic_ns": time.monotonic_ns()})
            sample["profiler_rss_bytes"] = sum(helper.process_group_stats(process.pid)[0]
                                                for process in [cpu, waits] if process is not None)
            samples.append(sample)
            stop_reason = None
            if sample["process_group_rss_bytes"] >= helper.MAX_PROCESS_GROUP_RSS_BYTES:
                stop_reason = "workload RSS guard"
            elif sample["mem_available_bytes"] < helper.MINIMUM_MEM_AVAILABLE_BYTES:
                stop_reason = "host memory reserve"
            elif sample["database_bytes"] >= helper.MAXIMUM_DATABASE_BYTES:
                stop_reason = "database size guard"
            elif sample["filesystem_available_bytes"] < helper.MINIMUM_DISK_AVAILABLE_BYTES:
                stop_reason = "filesystem reserve"
            elif time.monotonic() - started > 300:
                stop_reason = "workload timeout"
            elif cpu is not None and cpu.poll() is not None and child.poll() is None:
                stop_reason = "CPU collector exited before workload"
            elif waits is not None and waits.poll() is not None:
                stop_reason = "wait collector exited before workload"
            if stop_reason:
                raise RuntimeError(stop_reason)
            time.sleep(0.5)
        assert child.returncode == 0, child.returncode
        exit_ns = time.time_ns()
        if waits is not None:
            (case_root / "waits/stop").touch()
            assert waits.wait(timeout=30) == 0
            assert cpu.wait(timeout=30) == 0
            subprocess.run(["sudo", "-n", "chown", "-R", f"{os.getuid()}:{os.getgid()}", str(case_root)], check=True)
            control.unlink()
            acknowledgement.unlink()
        report = json.loads(report_path.read_text())
        helper.validate_latency_counts(report)
        helper.validate_checkpoint_report(report, command, source_sha, binary_sha)
        assert report["run_status"] == "completed"
        assert report["timing"]["measurement_complete"]
        assert report["timing"]["accepted_as_full_window"]
        assert helper.validation_passed(report.get("final_validation", report.get("validation")))
        assert helper.validation_passed(report["reopen_validation"])
        signature = helper.input_prefix_signature(report, "A" if workload == "RW80" else "E")
        signature_path = stage_root / f"{workload}-input-prefix.json"
        if signature_path.exists():
            assert json.loads(signature_path.read_text()) == signature
        else:
            write_json(signature_path, signature)
        if waits is not None:
            wait_report = json.loads((case_root / "waits/waits.json").read_text())
            assert wait_report["map_insert_failures"] == [0, 0], wait_report["map_insert_failures"]
        throughput = (report["throughput_per_second"]["api_calls"] if workload == "RW80"
                      else report["timing"]["write_batches_per_second"])
        result = {"case_id": case_id, "stage": options.stage, "source_sha": source_sha,
                  "binary_sha256": binary_sha, "throughput_per_second": throughput,
                  "release_epoch_ns": release_ns, "exit_epoch_ns": exit_ns,
                  "peak_workload_rss_bytes": max(sample["process_group_rss_bytes"] for sample in samples),
                  "peak_profiler_rss_bytes": max(sample["profiler_rss_bytes"] for sample in samples),
                  "maximum_database_bytes": max(sample["database_bytes"] for sample in samples),
                  "minimum_host_available_bytes": min(sample["mem_available_bytes"] for sample in samples),
                  "sample_count": len(samples), "elapsed_seconds": time.monotonic() - started}
        write_json(case_root / "accepted.json", result)
        print(json.dumps({"accepted": case_id, "throughput": throughput}), flush=True)
    finally:
        if (case_root / "waits").exists():
            (case_root / "waits/stop").touch()
        for process in [child, cpu, waits]:
            terminate(process)
        write_json(case_root / "resource-samples.json", samples)
        for handle in handles:
            handle.close()


if options.stage == "preflight":
    cases = [(workload, backend, "profile") for backend in ["blink", "minimal-native"] for workload in ["RW80", "E"]]
else:
    cases = [(workload, backend, mode) for mode, backends in [("control", ["blink", "minimal-native"]),
                                                             ("profile", ["minimal-native", "blink"])]
             for backend in backends for workload in ["RW80", "E"]]
write_json(stage_root / "run-plan.json", {"cases": cases, "measurement_ms": helper.MEASUREMENT_MS,
                                         "warmup_ms": helper.WARMUP_MS, "source_sha": source_sha,
                                         "binary_identities": identities,
                                         "script_sha256": {path.name: helper.sha256_file(path)
                                                           for path in scripts.glob("*.py")}})
for workload, backend, mode in cases:
    run_case(workload, backend, mode)
(stage_root / "complete.txt").write_text("requested cells accepted\n")
