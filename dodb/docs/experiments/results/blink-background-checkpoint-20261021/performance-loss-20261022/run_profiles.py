import importlib.util
import json
import os
import signal
import subprocess
import threading
import time
from pathlib import Path


ROOT = Path(os.environ["DODB_LOSS_RUN_ROOT"])
SPEC = importlib.util.spec_from_file_location("loss_controller", ROOT / "controller/run_loss.py")
LOSS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(LOSS)
BASE = LOSS.BASE
BASE.validate_checkpoint_report = LOSS.validate_report
os.environ["DODB_CHECKPOINT_KEEP_DATABASES"] = "1"


class ProfileProcessLauncher:
    def __init__(self):
        self.case_directory = None
        self.profilers = []
        self.stop_snapshots = threading.Event()
        self.snapshot_thread = None

    def __getattr__(self, name):
        return getattr(subprocess, name)

    def Popen(self, command, **options):
        child = subprocess.Popen(command, **options)
        if str(command[0]) != str(BASE.UNCONDITIONAL_BINARY):
            return child
        self.stop_snapshots.clear()
        self.snapshot_thread = threading.Thread(target=self.record_descriptors, args=(child.pid,))
        self.snapshot_thread.start()
        cpu = ["perf", "record", "--quiet", "-m", "512", "-o", str(self.case_directory / "cpu.perf.data"),
               "-e", "cpu-clock:u", "-F", "99", "--call-graph", "dwarf,8192", "--clockid", "mono", "-p", str(child.pid)]
        self.launch(cpu, "cpu", False)
        syscalls = ["perf", "record", "--quiet", "-m", "1024", "-o", str(self.case_directory / "syscalls.perf.data")]
        for name in ["pwrite64", "fdatasync", "fsync"]:
            syscalls.extend(["-e", f"syscalls:sys_enter_{name}", "-e", f"syscalls:sys_exit_{name}"])
        syscalls.extend(["--clockid", "mono", "-p", str(child.pid)])
        self.launch(syscalls, "syscalls", True)
        return child

    def launch(self, arguments, name, privileged):
        stderr_path = self.case_directory / f"{name}.perf.stderr.log"
        pid_path = self.case_directory / f"{name}.perf.pid"
        if privileged:
            launcher = "import os,pathlib,sys;pathlib.Path(sys.argv[1]).write_text(str(os.getpid()));os.chmod(sys.argv[1],0o644);os.execvp(sys.argv[2],sys.argv[2:])"
            command = ["sudo", "-n", "python3", "-c", launcher, str(pid_path)] + arguments
        else:
            command = arguments
        with stderr_path.open("xb") as output:
            process = subprocess.Popen(command, stdout=output, stderr=subprocess.STDOUT)
        self.profilers.append((name, process, privileged, pid_path))

    def record_descriptors(self, process_id):
        samples = []
        previous = None
        while not self.stop_snapshots.is_set():
            descriptors = {}
            try:
                for path in Path(f"/proc/{process_id}/fd").iterdir():
                    try:
                        descriptors[path.name] = os.readlink(path)
                    except OSError:
                        pass
            except OSError:
                break
            if descriptors != previous:
                samples.append({"monotonic_nanos": time.monotonic_ns(), "unix_nanos": time.time_ns(), "descriptors": descriptors})
                previous = descriptors
            self.stop_snapshots.wait(0.1)
        (self.case_directory / "descriptor-snapshots.json").write_text(json.dumps(samples, indent=2) + "\n")

    def finish(self):
        results = []
        for name, process, privileged, pid_path in self.profilers:
            if process.poll() is None:
                if privileged:
                    assert pid_path.exists(), name
                    target = int(pid_path.read_text())
                    subprocess.run(["sudo", "-n", "kill", "-INT", str(target)], check=True)
                else:
                    process.send_signal(signal.SIGINT)
            exit_code = process.wait(timeout=15)
            assert exit_code in [0, 130, -signal.SIGINT], (name, exit_code)
            results.append({"profiler": name, "exit_code": exit_code})
        self.stop_snapshots.set()
        if self.snapshot_thread is not None:
            self.snapshot_thread.join(timeout=2)
            assert not self.snapshot_thread.is_alive()
        (self.case_directory / "profiler-results.json").write_text(json.dumps(results, indent=2) + "\n")
        self.profilers = []


def run():
    BASE.REPORTS.mkdir(parents=True, exist_ok=True)
    BASE.DATABASES.mkdir(parents=True, exist_ok=True)
    assert subprocess.check_output(["git", "-C", str(BASE.SOURCE), "rev-parse", "HEAD"], text=True).strip() == LOSS.EXPECTED_SOURCE
    assert not subprocess.check_output(["git", "-C", str(BASE.SOURCE), "status", "--porcelain"], text=True)
    hashes = {name: BASE.sha256_file(BASE.TARGET / name) for name in LOSS.EXPECTED_BINARIES}
    assert hashes == LOSS.EXPECTED_BINARIES
    plan = {"source_sha": LOSS.EXPECTED_SOURCE, "controller_sha": os.environ["DODB_LOSS_CONTROLLER_SHA"],
            "controller_sha256": BASE.sha256_file(ROOT / "controller/run_loss.py"),
            "base_controller_sha256": BASE.sha256_file(ROOT / "controller/run_matrix_base.py"),
            "profiler_controller_sha256": BASE.sha256_file(Path(__file__)),
            "binary_hashes": hashes, "path_orders": {"1": ["legacy-off", "legacy-sync-128m"]},
            "workloads": ["E"], "cells": 2, "measurement_ms": BASE.MEASUREMENT_MS,
            "warmup_ms": BASE.WARMUP_MS, "smoke": False, "diagnostic_only": True,
            "limits": {"max_process_group_rss_bytes": BASE.MAX_PROCESS_GROUP_RSS_BYTES,
                       "minimum_mem_available_bytes": BASE.MINIMUM_MEM_AVAILABLE_BYTES,
                       "maximum_database_bytes": BASE.MAXIMUM_DATABASE_BYTES,
                       "minimum_filesystem_available_bytes": BASE.MINIMUM_DISK_AVAILABLE_BYTES}}
    LOSS.write_json(ROOT / "matrix-plan.json", plan)
    LOSS.write_json(ROOT / "binary-hashes.json", hashes)
    launcher = ProfileProcessLauncher()
    BASE.subprocess = launcher
    completed = []
    for path_name in plan["path_orders"]["1"]:
        case_id = f"E-{path_name}-r1"
        launcher.case_directory = BASE.REPORTS / case_id
        backend, policy = LOSS.POLICIES[path_name]
        database_path = BASE.DATABASES / case_id
        report_path = BASE.REPORTS / f"{case_id}.json"
        command = BASE.unconditional_command(backend, database_path, report_path, policy)
        command.extend(["--blink-wal-layout", "legacy", "--blink-checkpoint-execution", "sync"])
        print(json.dumps({"starting_diagnostic": case_id}), flush=True)
        try:
            BASE.wait_for_case(command, case_id, database_path, report_path, "E", backend, policy)
        finally:
            launcher.finish()
        completed.append(case_id)
        (ROOT / "completed-cases.txt").write_text("\n".join(completed) + "\n")
        print(json.dumps({"completed_diagnostic": case_id}), flush=True)
    (ROOT / "matrix-complete.txt").write_text("2 profiled diagnostic cases completed; exclude timings from headline medians\n")


if __name__ == "__main__":
    run()
