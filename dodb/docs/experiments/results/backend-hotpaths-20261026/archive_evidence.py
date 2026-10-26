import hashlib
import json
import os
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
    for revision in ["e3dfd3601", identities["candidate"]["source_sha"][:9]]:
        for source in Path("/bench/zfs/db/dodb-rocksdb-build-runs").glob(f"durable-bases-{revision}-*"):
            destination = build_directory / source.name
            if not destination.exists():
                shutil.copytree(source, destination)
    archive_path = ROOT.parent / f"{ROOT.name}-evidence.tar.gz"
    assert not archive_path.exists()
    included, databases = [], []
    for path in sorted(ROOT.rglob("*")):
        relative = path.relative_to(ROOT)
        if relative.as_posix() == "evidence-inventory.json":
            continue
        if relative.parts[0] in {"source", "bin", "test-databases"} or "databases" in relative.parts:
            continue
        if path.is_symlink() or not path.is_file():
            continue
        if "database-archives" in relative.parts and path.suffix != ".json":
            metadata = json.loads(path.with_suffix("").with_suffix(".json").read_text())
            assert metadata["verified"] and metadata["sha256"] == digest(path)
            databases.append(metadata)
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
