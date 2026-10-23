import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import time
from pathlib import Path


ROOT = Path(os.environ["DODB_COST_CANDIDATE_ROOT"])
SOURCE = ROOT / "source"


def digest(path):
    checksum = hashlib.sha256()
    with path.open("rb") as stream:
        while contents := stream.read(1024 * 1024):
            checksum.update(contents)
    return checksum.hexdigest()


def copy_verified(source, destination):
    destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.exists():
        assert digest(source) == digest(destination), destination
    else:
        shutil.copy2(source, destination)


def main():
    assert (ROOT / "all-stages-complete.txt").exists()
    followup_pid = int((ROOT / "followup-pid.txt").read_text())
    process_stat = Path(f"/proc/{followup_pid}/stat")
    for attempt in range(20):
        if not process_stat.exists() or process_stat.read_text().rsplit(")", 1)[1].split()[0] == "Z":
            break
        time.sleep(0.5)
    else:
        raise RuntimeError("follow-up process has not exited")
    plan = json.loads((ROOT / "short/matrix-plan.json").read_text())
    final_identity, checked_roots = {}, {}
    for policy, identity in plan["identities"].items():
        path_root = Path(identity["root"])
        if str(path_root) not in checked_roots:
            source = path_root / "source"
            revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
            clean = not subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip()
            binaries = {name: digest(path_root / "bin" / name) for name in identity["binaries"]}
            assert clean and revision == identity["source_sha"] and binaries == identity["binaries"]
            checked_roots[str(path_root)] = {"source_sha": revision, "binaries": binaries, "source_clean": clean}
        final_identity[policy] = checked_roots[str(path_root)]
    (ROOT / "final-identity.json").write_text(json.dumps(final_identity, indent=2) + "\n")
    previous_root = Path(plan["identities"]["previous"]["root"])
    for name in ["source-environment.txt", "binary-identities.json"]:
        copy_verified(previous_root / name, ROOT / "previous-provenance" / name)
    unchanged = ["dodb/crates/dodb-rocksdb", "dodb/crates/dodb-server", "dodb/crates/dodb-core",
                 "dodb/crates/dodb-service", "dodb/crates/dodb-protocol"]
    comparison = [plan["identities"]["previous"]["source_sha"], plan["identities"]["candidate"]["source_sha"]]
    subprocess.run(["git", "-C", str(SOURCE), "diff", "--exit-code", *comparison, "--", *unchanged], check=True)
    changed = subprocess.check_output(["git", "-C", str(SOURCE), "diff", "--name-only", *comparison,
                                       "--", "dodb/crates"], text=True).splitlines()
    (ROOT / "source-audit.json").write_text(json.dumps({"comparison": comparison, "unchanged_paths": unchanged,
                                                        "changed_crate_files": changed}, indent=2) + "\n")
    docs = SOURCE / "dodb/docs/experiments/results"
    copies = {
        "run_matrix.py": docs / "blink-checkpoint-costs-20261023/run_matrix.py",
        "pipeline-frozen.py": docs / "blink-checkpoint-costs-20261023/run_pipeline.py",
        "checkpoint_controller.py": docs / "blink-background-checkpoint-20261021/run_matrix.py",
        "analyze_loss.py": docs / "blink-background-checkpoint-20261021/performance-loss-20261022/analyze_loss.py",
        "run_validation.py": docs / "blink-durable-bases-20261023/run_validation.py",
        "run_build.py": docs / "blink-durable-bases-20261023/run_build.py",
        "monitored-build.py": docs / "blink-background-checkpoint-20261021/run_oci_build_monitored.py",
        "run-oci-rocksdb-build.sh": SOURCE / "dodb/scripts/run-oci-rocksdb-build.sh",
    }
    for name, source in copies.items():
        copy_verified(source, ROOT / "controller" / name)
    helper = docs / "blink-durable-bases-20261023/archive.py"
    specification = importlib.util.spec_from_file_location("base_archive", helper)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    destination = ROOT.parent / "blink-checkpoint-costs-20261023-evidence.tar.gz"
    assert not destination.exists()
    module.archive(ROOT, destination)


if __name__ == "__main__":
    main()
