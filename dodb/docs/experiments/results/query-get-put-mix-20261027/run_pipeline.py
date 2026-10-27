import importlib.util
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(os.environ["DODB_QUERYMIX_ROOT"])
SOURCE = ROOT / "source"
os.environ["DODB_BASE_RUN_ROOT"] = str(ROOT)
OLD_DOCS = SOURCE / "dodb/docs/experiments/results/blink-durable-bases-20261023"
SPEC = importlib.util.spec_from_file_location("querymix_base_pipeline", OLD_DOCS / "run_pipeline.py")
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)


def build():
    assert not subprocess.check_output(["git", "-C", str(SOURCE), "status", "--porcelain"], text=True).strip()
    revision = subprocess.check_output(["git", "-C", str(SOURCE), "rev-parse", "HEAD"], text=True).strip()
    (ROOT / "source-revision.txt").write_text(revision + "\n")
    arguments = [
        "test",
        "--locked",
        "--release",
        "-j",
        "1",
        "-p",
        "dodb-rocksdb",
        "--bin",
        "service_workload",
        "query_mix_has_exact_api_shares_and_client_local_write_keys",
    ]
    BASE.build("querymix-schedule-test", arguments)
    BASE.build(
        "querymix-workload",
        ["build", "--locked", "--release", "-j", "1", "-p", "dodb-rocksdb", "--bin", "service_workload"],
    )
    binary_directory = ROOT / "bin"
    binary_directory.mkdir(exist_ok=False)
    source_binary = BASE.TARGET / "service_workload"
    destination = binary_directory / "service_workload"
    shutil.copy2(source_binary, destination)
    identities = {
        "service_workload": {
            "sha256": BASE.hashlib.sha256(destination.read_bytes()).hexdigest(),
            "source_sha": BASE.REVISION,
            "build_path": str(source_binary),
        }
    }
    (ROOT / "binary-identities.json").write_text(json.dumps(identities, indent=2) + "\n")
    (ROOT / "build-complete.txt").write_text("querymix schedule test and optimized workload build completed\n")


def measure():
    assert (ROOT / "build-complete.txt").is_file()
    environment = BASE.ENVIRONMENT.copy()
    environment["DODB_QUERYMIX_ROOT"] = str(ROOT)
    for stage in ["preflight", "sustained"]:
        BASE.run(stage, [sys.executable, str(ROOT / "source/dodb/docs/experiments/results/query-get-put-mix-20261027/run_matrix.py"), stage], environment)
        BASE.run(
            f"{stage}-summary",
            [sys.executable, str(ROOT / "source/dodb/docs/experiments/results/query-get-put-mix-20261027/summarize.py"), str(ROOT / stage)],
            environment,
        )
    (ROOT / "measurement-complete.txt").write_text("preflight and three-repeat sustained comparison completed\n")


if __name__ == "__main__":
    {"build": build, "measure": measure}[sys.argv[1]]()
