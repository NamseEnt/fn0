import importlib.util
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(os.environ["DODB_HOTPATH_ROOT"])
SOURCE = ROOT / "source"
os.environ.update({"DODB_BASE_RUN_ROOT": str(ROOT), "DODB_COST_CANDIDATE_ROOT": str(ROOT)})
previous_pipeline = SOURCE / "dodb/docs/experiments/results/blink-checkpoint-costs-20261023/run_pipeline.py"
specification = importlib.util.spec_from_file_location("previous_pipeline", previous_pipeline)
pipeline = importlib.util.module_from_spec(specification)
specification.loader.exec_module(pipeline)
build_helper = pipeline.BASE
if (ROOT / "pipeline-steps.json").is_file():
    build_helper.STEPS.extend(json.loads((ROOT / "pipeline-steps.json").read_text()))


def build():
    assert not subprocess.check_output(["git", "-C", str(SOURCE), "status", "--porcelain"], text=True).strip()
    arguments = ["test", "--locked", "--release", "-j", "1", "-p", "dodb-storage", "-p", "dodb-server", "-p", "dodb-rocksdb",
                 "--lib", "--bin", "service_workload", "--bin", "unconditional_write_workload"]
    for name in ["background_checkpoint", "blink_wal_checkpoint", "background_checkpoint_sigkill", "segmented_checkpoint_sigkill",
                 "segmented_wal_sigkill", "phase2_recovery", "process_crash_recovery", "phase6_quic", "transaction_contract", "read_contract",
                 "contract_history", "durable_base_checkpoint", "durable_base_sigkill", "durable_base_background_sigkill"]:
        arguments.extend(["--test", name])
    arguments.append("--no-run")
    build_helper.build("tests", arguments)
    build_helper.build("workloads", ["build", "--locked", "--release", "-j", "1", "-p", "dodb-rocksdb",
                                      "--bin", "service_workload", "--bin", "unconditional_write_workload"])
    binary_directory = ROOT / "bin"
    binary_directory.mkdir(exist_ok=False)
    identities = {}
    for name in ["service_workload", "unconditional_write_workload"]:
        destination = binary_directory / name
        shutil.copy2(build_helper.TARGET / name, destination)
        identities[name] = {"sha256": build_helper.hashlib.sha256(destination.read_bytes()).hexdigest(),
                            "source_sha": build_helper.REVISION}
    (ROOT / "binary-identities.json").write_text(json.dumps(identities, indent=2) + "\n")
    environment = build_helper.ENVIRONMENT.copy()
    environment["DODB_BASE_TEST_BUILD_LOG"] = (
        f"/bench/zfs/db/dodb-rocksdb-build-runs/durable-bases-{build_helper.REVISION[:9]}-tests/cargo.log")
    build_helper.run("validation", [sys.executable, str(build_helper.DOCS / "run_validation.py")], environment)
    (ROOT / "build-and-validation-complete.txt").write_text("frozen candidate build and functional validation completed\n")


def measure():
    assert (ROOT / "build-and-validation-complete.txt").is_file()
    scripts = Path(__file__).resolve().parent
    for stage in ["preflight", "short", "sustained"]:
        environment = build_helper.ENVIRONMENT.copy()
        environment["DODB_HOTPATH_ROOT"] = str(ROOT)
        build_helper.run(stage, [sys.executable, str(scripts / "run_matrix.py"), stage], environment)
        build_helper.run(f"{stage}-summary", [sys.executable, str(scripts / "summarize.py"), str(ROOT / stage)], environment)
        if stage == "short":
            summary = json.loads((ROOT / stage / "matrix-summary.json").read_text())
            (ROOT / "short-decision.json").write_text(json.dumps({
                "gates": summary["screening_gates"],
                "sustained_reason": "Functional and resource gates passed; sustained comparison reports both passing and failing performance gates.",
                "short_summary_sha256": build_helper.hashlib.sha256((ROOT / stage / "matrix-summary.json").read_bytes()).hexdigest(),
            }, indent=2) + "\n")
    (ROOT / "measurement-complete.txt").write_text("preflight, short and sustained evidence accepted\n")


if __name__ == "__main__":
    {"build": build, "measure": measure}[sys.argv[1]]()
