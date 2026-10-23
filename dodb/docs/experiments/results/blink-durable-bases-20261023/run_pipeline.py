import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(os.environ["DODB_BASE_RUN_ROOT"])
SOURCE = ROOT / "source"
REVISION = subprocess.check_output(["git", "-C", str(SOURCE), "rev-parse", "HEAD"], text=True).strip()
TARGET = Path("/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target/release")
DOCS = SOURCE / "dodb/docs/experiments/results/blink-durable-bases-20261023"
ENVIRONMENT = os.environ.copy()
ENVIRONMENT.update({"PYTHONDONTWRITEBYTECODE": "1", "DODB_BASE_SOURCE_SHA": REVISION})
STEPS = []


def run(name, command, environment=None):
    record = {"name": name, "argv": command, "source_sha": REVISION, "started": time.time()}
    STEPS.append(record)
    (ROOT / "pipeline-steps.json").write_text(json.dumps(STEPS, indent=2) + "\n")
    print(json.dumps({"starting": name}), flush=True)
    with (ROOT / f"{name}.log").open("wb") as stream:
        result = subprocess.run(command, cwd=SOURCE, env=environment or ENVIRONMENT, stdout=stream, stderr=subprocess.STDOUT)
    record.update({"exit_code": result.returncode, "ended": time.time()})
    (ROOT / "pipeline-steps.json").write_text(json.dumps(STEPS, indent=2) + "\n")
    if result.returncode:
        raise SystemExit(result.returncode)
    print(json.dumps({"completed": name}), flush=True)


def build(name, arguments):
    environment = ENVIRONMENT.copy()
    environment["DODB_BUILD_RUN_ID"] = f"durable-bases-{REVISION[:9]}-{name}"
    run(name, [sys.executable, str(DOCS / "run_build.py"), name, "bash", "dodb/scripts/run-oci-rocksdb-build.sh", *arguments], environment)


def matrix(name, smoke):
    root = ROOT / name
    root.mkdir(exist_ok=False)
    (root / "source").symlink_to(SOURCE)
    (root / "bin").symlink_to(ROOT / "bin")
    environment = ENVIRONMENT.copy()
    environment["DODB_BASE_RUN_ROOT"] = str(root)
    environment["DODB_BASE_SMOKE"] = "1" if smoke else "0"
    run(name, [sys.executable, str(DOCS / "run_matrix.py")], environment)


def main():
    assert not subprocess.check_output(["git", "-C", str(SOURCE), "status", "--porcelain"], text=True).strip()
    with (ROOT / "source-environment.txt").open("w") as stream:
        for command in [["git", "rev-parse", "HEAD"], ["git", "remote", "-v"], ["uname", "-a"], ["lscpu"],
                        ["findmnt", "-T", str(ROOT)], ["zfs", "get", "sync,compression,recordsize,logbias", "dodbbench/db"],
                        ["free", "-m"], ["df", "-h", str(ROOT)], ["rustc", "-vV"]]:
            stream.write(json.dumps(command) + "\n")
            stream.flush()
            subprocess.run(command, cwd=SOURCE, stdout=stream, stderr=subprocess.STDOUT, check=True)
    arguments = ["test", "--locked", "--release", "-j", "1", "-p", "dodb-storage", "-p", "dodb-server", "-p", "dodb-rocksdb",
                 "--lib", "--bin", "service_workload", "--bin", "unconditional_write_workload"]
    for name in ["background_checkpoint", "blink_wal_checkpoint", "background_checkpoint_sigkill", "segmented_checkpoint_sigkill",
                 "segmented_wal_sigkill", "phase2_recovery", "process_crash_recovery", "phase6_quic", "transaction_contract", "read_contract",
                 "contract_history", "durable_base_checkpoint", "durable_base_sigkill", "durable_base_background_sigkill"]:
        arguments.extend(["--test", name])
    arguments.append("--no-run")
    build("tests", arguments)
    build("workloads", ["build", "--locked", "--release", "-j", "1", "-p", "dodb-rocksdb", "--bin", "service_workload", "--bin", "unconditional_write_workload"])
    binary_directory = ROOT / "bin"
    binary_directory.mkdir(exist_ok=False)
    identities = {}
    for name in ["service_workload", "unconditional_write_workload"]:
        destination = binary_directory / name
        shutil.copy2(TARGET / name, destination)
        identities[name] = {"sha256": hashlib.sha256(destination.read_bytes()).hexdigest(), "source_sha": REVISION, "build_path": str(TARGET / name)}
    (ROOT / "binary-identities.json").write_text(json.dumps(identities, indent=2) + "\n")
    environment = ENVIRONMENT.copy()
    environment["DODB_BASE_TEST_BUILD_LOG"] = f"/bench/zfs/db/dodb-rocksdb-build-runs/durable-bases-{REVISION[:9]}-tests/cargo.log"
    run("validation", [sys.executable, str(DOCS / "run_validation.py")], environment)
    matrix("preflight", True)
    matrix("short", False)
    (ROOT / "short-stage-complete.txt").write_text("validation, preflight and short matrix passed\n")


if __name__ == "__main__":
    main()
