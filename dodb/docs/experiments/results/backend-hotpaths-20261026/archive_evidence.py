import hashlib
import json
import os
import re
import shutil
import subprocess
import tarfile
from pathlib import Path


ROOT = Path(os.environ["DODB_HOTPATH_ROOT"])


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            value.update(chunk)
    return value.hexdigest()


def main():
    assert (ROOT / "measurement-complete.txt").is_file()
    control = Path("/bench/zfs/db/blink-checkpoint-costs-20261023")
    identities = {}
    for name, directory in [("control", control), ("candidate", ROOT)]:
        source = directory / "source"
        assert not subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip()
        revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
        declared = json.loads((directory / "binary-identities.json").read_text())
        assert all(digest(directory / "bin" / binary) == entry["sha256"] and entry["source_sha"] == revision
                   for binary, entry in declared.items())
        identities[name] = {"source_sha": revision, "binaries": declared}
    (ROOT / "final-identities.json").write_text(json.dumps(identities, indent=2) + "\n")
    build_directory = ROOT / "build-provenance"
    build_directory.mkdir(exist_ok=True)
    revisions = {record["source_sha"][:9] for record in json.loads((ROOT / "pipeline-steps.json").read_text())}
    for revision in sorted(revisions):
        for source in Path("/bench/zfs/db/dodb-rocksdb-build-runs").glob(f"durable-bases-{revision}-*"):
            destination = build_directory / source.name
            if not destination.exists():
                shutil.copytree(source, destination)
    for command_path in ROOT.glob("lint-command*.json"):
        lint = json.loads(command_path.read_text())
        source = Path("/bench/zfs/db/dodb-rocksdb-build-runs") / lint["build_run_id"]
        destination = build_directory / source.name
        if not destination.exists():
            shutil.copytree(source, destination)
    control_provenance = ROOT / "control-provenance"
    if not control_provenance.exists():
        shutil.copytree(control / "build-provenance", control_provenance)
    environments = {
        "control": control_provenance / f"durable-bases-{identities['control']['source_sha'][:9]}-workloads/environment.txt",
        "candidate": build_directory / f"durable-bases-{identities['candidate']['source_sha'][:9]}-workloads/environment.txt",
    }
    library_hashes = {}
    for label, environment_path in environments.items():
        environment = environment_path.read_text()
        assert "rustc 1.97.1 " in environment
        assert "rocksdb_source_sha=abeebd9630f11bd08c28b7bd43c7bdfc62050654\n" in environment
        library_hashes[label] = re.search(r"^([0-9a-f]{64})\s+\S*/librocksdb\.a$", environment, re.MULTILINE).group(1)
    assert library_hashes["control"] == library_hashes["candidate"]
    (ROOT / "native-library-identity.json").write_text(json.dumps({"rocksdb_source_sha": "abeebd9630f11bd08c28b7bd43c7bdfc62050654",
        "archive_sha256": library_hashes["candidate"], "same_control_candidate_archive": True}, indent=2) + "\n")
    archive_path = ROOT.parent / f"{ROOT.name}-evidence.tar.gz"
    assert not archive_path.exists()
    included, databases = [], []
    for metadata_path in sorted(ROOT.glob("*/database-archives/*.json")):
        metadata = json.loads(metadata_path.read_text())
        payload = Path(metadata["path"])
        assert metadata["verified"] and metadata["sha256"] == digest(payload)
        databases.append(metadata)
    for path in sorted(ROOT.rglob("*")):
        relative = path.relative_to(ROOT)
        if relative.as_posix() == "evidence-inventory.json":
            continue
        if relative.parts[0] in {"source", "bin", "test-databases"} or "databases" in relative.parts:
            continue
        if path.is_symlink() or not path.is_file():
            continue
        if "database-archives" in relative.parts and path.suffix != ".json":
            continue
        included.append({"path": relative.as_posix(), "size_bytes": path.stat().st_size, "sha256": digest(path)})
    inventory = {"files": included, "database_archives_preserved_on_host": databases}
    inventory_path = ROOT / "evidence-inventory.json"
    inventory_path.write_text(json.dumps(inventory, indent=2) + "\n")
    with tarfile.open(archive_path, "w:gz", compresslevel=6) as archive:
        archive.add(inventory_path, arcname="evidence-inventory.json")
        for entry in included:
            archive.add(ROOT / entry["path"], arcname=entry["path"], recursive=False)
    sidecar = {"path": str(archive_path), "sha256": digest(archive_path), "size_bytes": archive_path.stat().st_size,
               "inventoried_files": len(included), "inventoried_bytes": sum(entry["size_bytes"] for entry in included),
               "database_archives": len(databases), "database_archive_bytes": sum(entry["compressed_bytes"] for entry in databases),
               "identities": identities}
    archive_path.with_suffix(archive_path.suffix + ".json").write_text(json.dumps(sidecar, indent=2) + "\n")
    print(json.dumps(sidecar, indent=2))


if __name__ == "__main__":
    main()
