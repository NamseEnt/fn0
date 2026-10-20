import json
import os
import pathlib
import signal
import subprocess
import sys
import tempfile
import time


def process_state(process_id):
    result = subprocess.run(["ps", "-o", "stat=", "-p", str(process_id)], capture_output=True, text=True, check=False)
    return result.stdout.strip()


def live_group_members(process_group_id):
    result = subprocess.run(["ps", "-eo", "pid=,pgid=,stat="], capture_output=True, text=True, check=True)
    members = []
    for process_line in result.stdout.splitlines():
        process_fields = process_line.split()
        if len(process_fields) >= 3 and int(process_fields[1]) == process_group_id and not process_fields[2].startswith("Z"):
            members.append(int(process_fields[0]))
    return members


def terminate_group(process, grace_seconds=2):
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    deadline = time.monotonic() + grace_seconds
    while time.monotonic() < deadline and live_group_members(process.pid):
        time.sleep(0.1)
    if live_group_members(process.pid):
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    process.wait(timeout=5)


def wait_until_gone(process_id, timeout_seconds):
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        state = process_state(process_id)
        if not state or state.startswith("Z"):
            return state
        time.sleep(0.1)
    raise RuntimeError(f"process {process_id} remained live with state {process_state(process_id)}")


with tempfile.TemporaryDirectory(prefix="dodb-process-group-selfcheck-") as temporary_directory:
    child_id_path = pathlib.Path(temporary_directory) / "child.pid"
    parent_program = (
        "import pathlib, subprocess, sys, time; "
        "child=subprocess.Popen([sys.executable, '-c', 'import signal, time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(60)']); "
        "pathlib.Path(sys.argv[1]).write_text(str(child.pid)); "
        "time.sleep(60)"
    )
    target = subprocess.Popen([sys.executable, "-c", parent_program, str(child_id_path)], start_new_session=True)
    unrelated = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"], start_new_session=True)
    try:
        deadline = time.monotonic() + 5
        while not child_id_path.exists() and time.monotonic() < deadline:
            time.sleep(0.05)
        if not child_id_path.exists():
            raise RuntimeError("target grandchild did not start")
        child_process_id = int(child_id_path.read_text())
        terminate_group(target)
        child_state = wait_until_gone(child_process_id, 5)
        if unrelated.poll() is not None or not process_state(unrelated.pid):
            raise RuntimeError("terminating the target group affected the unrelated group")
        result = {
            "target_process_group_id": target.pid,
            "target_exit_status": target.returncode,
            "grandchild_process_id": child_process_id,
            "grandchild_final_state": child_state or "absent",
            "unrelated_process_group_id": unrelated.pid,
            "unrelated_survived_target_termination": True,
            "only_target_process_group_terminated": True,
        }
    finally:
        for process in (target, unrelated):
            if live_group_members(process.pid):
                terminate_group(process)

output = json.dumps(result, indent=2) + "\n"
pathlib.Path(__file__).with_suffix(".json").write_text(output)
print(output, end="")
