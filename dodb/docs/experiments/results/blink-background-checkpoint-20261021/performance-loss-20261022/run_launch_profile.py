import importlib.util
import json
import os
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


class LaunchProfiler:
    def __init__(self, directory):
        self.directory = directory
        self.snapshots = []
        self.finished = threading.Event()
        self.thread = None

    def __getattr__(self, name):
        return getattr(subprocess, name)

    def Popen(self, command, **options):
        if str(command[0]) != str(BASE.UNCONDITIONAL_BINARY):
            return subprocess.Popen(command, **options)
        wrapper = ["sudo", "-n", "perf", "record", "--quiet", "-m", "4096", "--clockid", "mono",
                   "-o", str(self.directory / "syscalls.perf.data")]
        for name in ["pwrite64", "fdatasync", "fsync"]:
            wrapper.extend(["-e", f"syscalls:sys_enter_{name}", "-e", f"syscalls:sys_exit_{name}"])
        wrapper.extend(["--", "sudo", "-n", "-u", "opc", "--", "env",
                        f"DODB_SOURCE_SHA={options['env']['DODB_SOURCE_SHA']}",
                        f"DODB_BINARY_SHA256={options['env']['DODB_BINARY_SHA256']}"] + command)
        (self.directory / "profiler-command.json").write_text(json.dumps(wrapper, indent=2) + "\n")
        child = subprocess.Popen(wrapper, **options)
        self.thread = threading.Thread(target=self.record_descriptors, args=(child.pid, str(command[0])))
        self.thread.start()
        return child

    def record_descriptors(self, parent_id, executable):
        target = None
        previous = None
        while not self.finished.is_set():
            if target is None:
                pending = [parent_id]
                while pending:
                    process_id = pending.pop()
                    try:
                        argv = Path(f"/proc/{process_id}/cmdline").read_bytes().split(b"\0")
                        if argv and argv[0].decode() == executable:
                            target = process_id
                            break
                        pending.extend(int(value) for value in Path(f"/proc/{process_id}/task/{process_id}/children").read_text().split())
                    except (OSError, ValueError):
                        pass
                if target is None:
                    self.finished.wait(0.01)
                    continue
                (self.directory / "profiled-process-id.txt").write_text(str(target) + "\n")
            descriptors = {}
            try:
                for path in Path(f"/proc/{target}/fd").iterdir():
                    try:
                        descriptors[path.name] = os.readlink(path)
                    except OSError:
                        pass
            except OSError:
                break
            if descriptors != previous:
                self.snapshots.append({"monotonic_nanos": time.monotonic_ns(), "unix_nanos": time.time_ns(), "descriptors": descriptors})
                previous = descriptors
            self.finished.wait(0.1)

    def finish(self):
        self.finished.set()
        if self.thread is not None:
            self.thread.join(timeout=2)
            assert not self.thread.is_alive()
        assert self.snapshots
        (self.directory / "descriptor-snapshots.json").write_text(json.dumps(self.snapshots, indent=2) + "\n")
        (self.directory / "syscalls.perf.stderr.log").write_bytes((self.directory / "workload.stderr.log").read_bytes())


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
            "profiler_controller_sha256": BASE.sha256_file(Path(__file__)), "profiler_controller": Path(__file__).name,
            "binary_hashes": hashes, "path_orders": {"1": ["legacy-sync-128m"]}, "workloads": ["E"], "cells": 1,
            "measurement_ms": BASE.MEASUREMENT_MS, "warmup_ms": BASE.WARMUP_MS, "smoke": False, "diagnostic_only": True,
            "method": "launch under inherited syscall tracing; benchmark executes as opc",
            "limits": {"max_process_group_rss_bytes": BASE.MAX_PROCESS_GROUP_RSS_BYTES,
                       "minimum_mem_available_bytes": BASE.MINIMUM_MEM_AVAILABLE_BYTES,
                       "maximum_database_bytes": BASE.MAXIMUM_DATABASE_BYTES,
                       "minimum_filesystem_available_bytes": BASE.MINIMUM_DISK_AVAILABLE_BYTES}}
    LOSS.write_json(ROOT / "matrix-plan.json", plan)
    LOSS.write_json(ROOT / "binary-hashes.json", hashes)
    case_id = "E-legacy-sync-128m-r1"
    directory = BASE.REPORTS / case_id
    launcher = LaunchProfiler(directory)
    BASE.subprocess = launcher
    command = BASE.unconditional_command("blink", BASE.DATABASES / case_id, BASE.REPORTS / f"{case_id}.json", "134217728")
    command.extend(["--blink-wal-layout", "legacy", "--blink-checkpoint-execution", "sync"])
    print(json.dumps({"starting_diagnostic": case_id}), flush=True)
    try:
        BASE.wait_for_case(command, case_id, BASE.DATABASES / case_id, BASE.REPORTS / f"{case_id}.json", "E", "blink", "134217728")
    finally:
        launcher.finish()
    (ROOT / "completed-cases.txt").write_text(case_id + "\n")
    (ROOT / "matrix-complete.txt").write_text("1 launch-traced diagnostic case completed; exclude from headline medians\n")
    print(json.dumps({"completed_diagnostic": case_id}), flush=True)


if __name__ == "__main__":
    run()
