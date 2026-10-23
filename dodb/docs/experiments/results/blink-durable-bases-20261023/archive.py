import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while block := stream.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def archive(root, destination):
    source = root / "source"
    revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    assert not subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip()
    build_root = Path("/bench/zfs/db/dodb-rocksdb-build-runs")
    provenance = root / "build-provenance"
    provenance.mkdir(exist_ok=True)
    for build in build_root.glob(f"durable-bases-{revision[:9]}-*"):
        copied = provenance / build.name
        if copied.exists():
            for path in build.rglob("*"):
                if path.is_file():
                    assert sha256(path) == sha256(copied / path.relative_to(build))
        else:
            shutil.copytree(build, copied)
    excluded = {"source", "bin", "databases", "test-databases"}
    files = []
    for directory, children, names in os.walk(root, followlinks=False):
        relative = Path(directory).relative_to(root)
        children[:] = [name for name in children if name not in excluded and not (Path(directory) / name).is_symlink()]
        for name in names:
            path = Path(directory) / name
            if path.is_symlink() or name in {"evidence-file-sha256.json", "SHA256SUMS"}:
                continue
            if path.resolve() == destination.resolve():
                continue
            files.append(path)
    files.sort()
    inventory = [{"path": path.relative_to(root).as_posix(), "size_bytes": path.stat().st_size, "sha256": sha256(path)} for path in files]
    inventory_path = root / "evidence-file-sha256.json"
    inventory_path.write_text(json.dumps(inventory, indent=2) + "\n")
    with tarfile.open(destination, "w:gz") as output:
        for path in files + [inventory_path]:
            output.add(path, arcname=path.relative_to(root).as_posix(), recursive=False)
    with tarfile.open(destination, "r:gz") as archived:
        for entry in inventory:
            contents = archived.extractfile(entry["path"]).read()
            assert len(contents) == entry["size_bytes"]
            assert hashlib.sha256(contents).hexdigest() == entry["sha256"]
    summary = {"source_sha": revision, "files": len(inventory), "uncompressed_bytes": sum(entry["size_bytes"] for entry in inventory),
               "archive": destination.name, "archive_bytes": destination.stat().st_size, "sha256": sha256(destination),
               "inventory_verified": True, "excluded": sorted(excluded),
               "failed_databases_remain_on_host": True}
    destination.with_suffix(destination.suffix + ".json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    archive(Path(sys.argv[1]), Path(sys.argv[2]))
