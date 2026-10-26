import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path


EXPERIMENT = Path(os.environ["DODB_HOTPATH_ROOT"])
STAGE = sys.argv[1]
ROOT = EXPERIMENT / STAGE
CONTROL = Path("/bench/zfs/db/blink-checkpoint-costs-20261023")
SMOKE = STAGE == "preflight"
DURATION = 8000 if SMOKE else 60000 if STAGE == "short" else 300000
os.environ.update({"DODB_COST_RUN_ROOT": str(ROOT), "DODB_COST_CANDIDATE_ROOT": str(EXPERIMENT),
                   "DODB_COST_SMOKE": "1" if SMOKE else "0", "DODB_COST_DURATION_MS": str(DURATION),
                   "DODB_CHECKPOINT_KEEP_DATABASES": "1", "PYTHONDONTWRITEBYTECODE": "1"})
helper_path = EXPERIMENT / "source/dodb/docs/experiments/results/blink-checkpoint-costs-20261023/run_matrix.py"
specification = importlib.util.spec_from_file_location("previous_matrix", helper_path)
previous = importlib.util.module_from_spec(specification)
specification.loader.exec_module(previous)
helper = previous.BASE
POLICIES = {
    "blink-control": (CONTROL, "blink"),
    "rocksdb-control": (CONTROL, "minimal-native"),
    "blink-candidate": (EXPERIMENT, "blink"),
    "rocksdb-candidate": (EXPERIMENT, "minimal-native"),
}


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def archive_database(case, database):
    archive_directory = ROOT / "database-archives"
    archive_directory.mkdir(exist_ok=True)
    destination = archive_directory / f"{case}.tar.gz"
    metadata_path = archive_directory / f"{case}.json"
    if metadata_path.exists():
        metadata = json.loads(metadata_path.read_text())
        assert metadata["sha256"] == helper.sha256_file(destination)
        assert metadata["verified"]
        if database.exists():
            assert database.parent == ROOT / "databases"
            shutil.rmtree(database)
        return
    assert not destination.exists()
    assert database.parent == ROOT / "databases" and database.name == case
    inventory = json.loads((ROOT / "reports" / case / "database-manifest.json").read_text())
    temporary = destination.with_suffix(".gz.incomplete")
    assert not temporary.exists()
    with tarfile.open(temporary, "w:gz", compresslevel=1) as archive:
        archive.add(database, arcname=case)
    with temporary.open("rb") as stream:
        os.fsync(stream.fileno())
    actual = {}
    with tarfile.open(temporary, "r:gz") as archive:
        for member in archive:
            if member.isdir():
                continue
            assert member.isfile() and member.name.startswith(case + "/")
            digest = hashlib.sha256()
            with archive.extractfile(member) as stream:
                while chunk := stream.read(1024 * 1024):
                    digest.update(chunk)
            actual[member.name[len(case) + 1:]] = {"size_bytes": member.size, "sha256": digest.hexdigest()}
    expected = {entry["path"]: {"size_bytes": entry["size_bytes"], "sha256": entry["sha256"]} for entry in inventory}
    assert actual == expected
    os.replace(temporary, destination)
    metadata = {"case": case, "path": str(destination), "sha256": helper.sha256_file(destination),
                "compressed_bytes": destination.stat().st_size,
                "uncompressed_bytes": sum(entry["size_bytes"] for entry in inventory),
                "file_count": len(inventory), "verified": True}
    write_json(metadata_path, metadata)
    with metadata_path.open("rb") as stream:
        os.fsync(stream.fileno())
    shutil.rmtree(database)
    print(json.dumps({"database_preserved": case, **metadata}), flush=True)


def main():
    ROOT.mkdir(parents=True, exist_ok=True)
    helper.REPORTS.mkdir(exist_ok=True)
    helper.DATABASES.mkdir(exist_ok=True)
    selected = [policy for policy in POLICIES if not SMOKE or policy.endswith("candidate")]
    identities = {}
    for policy in selected:
        path_root, backend = POLICIES[policy]
        source = path_root / "source"
        assert not subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip()
        declared = json.loads((path_root / "binary-identities.json").read_text())
        revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
        binaries = {name: helper.sha256_file(path_root / "bin" / name) for name in declared}
        assert all(binaries[name] == declared[name]["sha256"] and revision == declared[name]["source_sha"] for name in declared)
        identities[policy] = {"root": str(path_root), "backend": backend, "source_sha": revision, "binaries": binaries}
    orders = {repetition: selected[(repetition - 1) % len(selected):] + selected[:(repetition - 1) % len(selected)]
              for repetition in range(1, 2 if SMOKE else 4)}
    plan = {"identities": identities, "orders": orders, "workloads": ["RW80", "E"], "smoke": SMOKE,
            "duration_ms": DURATION, "warmup_ms": helper.WARMUP_MS,
            "controller_sha256": helper.sha256_file(Path(__file__)), "helper_sha256": helper.sha256_file(helper_path),
            "rss_limit_bytes": helper.MAX_PROCESS_GROUP_RSS_BYTES, "disk_limit_bytes": helper.MAXIMUM_DATABASE_BYTES,
            "minimum_memory_bytes": helper.MINIMUM_MEM_AVAILABLE_BYTES, "minimum_disk_bytes": helper.MINIMUM_DISK_AVAILABLE_BYTES,
            "extra_recovery_bytes": 512 * 1024**2, "database_preservation": "verified-lossless-archive-between-cells"}
    plan_path = ROOT / "matrix-plan.json"
    if plan_path.exists():
        assert json.loads(plan_path.read_text()) == json.loads(json.dumps(plan))
    write_json(plan_path, plan)
    completed_path = ROOT / "completed-cases.txt"
    completed = completed_path.read_text().splitlines() if completed_path.exists() else []
    prefixes_path = ROOT / "input-prefix-fingerprints.json"
    prefixes = json.loads(prefixes_path.read_text()) if prefixes_path.exists() else []
    references = {entry["workload"]: entry["signature"] for entry in prefixes}
    helper.process_sample = previous.process_sample
    helper.validate_checkpoint_report = previous.validate
    for repetition, order in orders.items():
        for policy in order:
            path_root, backend = POLICIES[policy]
            helper.SOURCE = path_root / "source"
            helper.SERVICE_BINARY = path_root / "bin/service_workload"
            helper.UNCONDITIONAL_BINARY = path_root / "bin/unconditional_write_workload"
            for workload in plan["workloads"]:
                case = f"{workload}-{policy}-r{repetition}"
                if case in completed:
                    continue
                database, report_path = helper.DATABASES / case, helper.REPORTS / f"{case}.json"
                if workload == "RW80":
                    command = helper.service_command(backend, database, report_path, "134217728")
                    command[command.index("--scenario") + 1] = "rw80"
                else:
                    command = helper.unconditional_command(backend, database, report_path, "134217728")
                if backend == "blink":
                    command.extend(["--blink-wal-layout", "segmented-bases-coalesced", "--blink-checkpoint-execution",
                                    "background", "--blink-wal-segment-bytes", "33554432"])
                print(json.dumps({"starting": case, "stage": STAGE}), flush=True)
                evidence = helper.REPORTS / case
                if evidence.exists():
                    result = json.loads((evidence / "controller-result.json").read_text())
                    assert result["exit_code"] == 0 and result["stop_reason"] is None
                    assert helper.sha256_file(report_path) == (evidence / "workload-report-sha256.txt").read_text().strip()
                    previous.ORIGINAL_VALIDATE(json.loads(report_path.read_text()), command,
                        identities[policy]["source_sha"], identities[policy]["binaries"][Path(command[0]).name])
                else:
                    helper.wait_for_case(command, case, database, report_path,
                                         "A" if workload == "RW80" else "E", backend, "134217728")
                report = json.loads(report_path.read_text())
                signature = helper.input_prefix_signature(report, "A" if workload == "RW80" else "E")
                references.setdefault(workload, signature)
                assert references[workload] == signature, case
                archive_database(case, database)
                prefixes.append({"case": case, "workload": workload, "signature": signature})
                write_json(prefixes_path, prefixes)
                completed.append(case)
                completed_path.write_text("\n".join(completed) + "\n")
                throughput = report["throughput_per_second"]["api_calls"] if workload == "RW80" else report["timing"]["write_batches_per_second"]
                print(json.dumps({"completed": case, "throughput": throughput, "stage": STAGE}), flush=True)
    for policy, identity in identities.items():
        path_root = Path(identity["root"])
        assert not subprocess.check_output(["git", "-C", str(path_root / "source"), "status", "--porcelain"], text=True).strip()
        assert all(helper.sha256_file(path_root / "bin" / name) == digest for name, digest in identity["binaries"].items())
    (ROOT / "matrix-complete.txt").write_text(f"{len(completed)} accepted cells\n")


if __name__ == "__main__":
    main()
