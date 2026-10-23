import importlib.util
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(os.environ["DODB_COST_CANDIDATE_ROOT"])
SOURCE = ROOT / "source"
os.environ["DODB_BASE_RUN_ROOT"] = str(ROOT)
OLD_DOCS = SOURCE / "dodb/docs/experiments/results/blink-durable-bases-20261023"
DOCS = SOURCE / "dodb/docs/experiments/results/blink-checkpoint-costs-20261023"
SPEC = importlib.util.spec_from_file_location("base_pipeline", OLD_DOCS / "run_pipeline.py")
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)


def main():
    assert not subprocess.check_output(["git", "-C", str(SOURCE), "status", "--porcelain"], text=True).strip()
    arguments = ["test", "--locked", "--release", "-j", "1", "-p", "dodb-storage", "-p", "dodb-server", "-p", "dodb-rocksdb",
                 "--lib", "--bin", "service_workload", "--bin", "unconditional_write_workload"]
    for name in ["background_checkpoint", "blink_wal_checkpoint", "background_checkpoint_sigkill", "segmented_checkpoint_sigkill",
                 "segmented_wal_sigkill", "phase2_recovery", "process_crash_recovery", "phase6_quic", "transaction_contract", "read_contract",
                 "contract_history", "durable_base_checkpoint", "durable_base_sigkill", "durable_base_background_sigkill"]:
        arguments.extend(["--test", name])
    arguments.append("--no-run")
    BASE.build("tests", arguments)
    BASE.build("workloads", ["build", "--locked", "--release", "-j", "1", "-p", "dodb-rocksdb", "--bin", "service_workload", "--bin", "unconditional_write_workload"])
    binary_directory = ROOT / "bin"
    binary_directory.mkdir(exist_ok=False)
    identities = {}
    for name in ["service_workload", "unconditional_write_workload"]:
        destination = binary_directory / name
        shutil.copy2(BASE.TARGET / name, destination)
        identities[name] = {"sha256": BASE.hashlib.sha256(destination.read_bytes()).hexdigest(), "source_sha": BASE.REVISION}
    (ROOT / "binary-identities.json").write_text(json.dumps(identities, indent=2) + "\n")
    environment = BASE.ENVIRONMENT.copy()
    environment["DODB_BASE_TEST_BUILD_LOG"] = f"/bench/zfs/db/dodb-rocksdb-build-runs/durable-bases-{BASE.REVISION[:9]}-tests/cargo.log"
    BASE.run("validation", [sys.executable, str(OLD_DOCS / "run_validation.py")], environment)
    for stage, smoke in [("preflight", True), ("short", False)]:
        environment = BASE.ENVIRONMENT.copy()
        environment.update({"DODB_COST_CANDIDATE_ROOT": str(ROOT), "DODB_COST_RUN_ROOT": str(ROOT / stage),
                            "DODB_COST_SMOKE": "1" if smoke else "0"})
        BASE.run(stage, [sys.executable, str(DOCS / "run_matrix.py")], environment)
    (ROOT / "short-stage-complete.txt").write_text("functional validation, preflight and short matrix completed\n")


if __name__ == "__main__":
    main()
