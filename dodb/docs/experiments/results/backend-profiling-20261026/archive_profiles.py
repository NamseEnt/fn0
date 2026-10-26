import argparse
import hashlib
import json
import subprocess
import tarfile
from pathlib import Path


parser = argparse.ArgumentParser()
parser.add_argument("--root", type=Path, required=True)
options = parser.parse_args()
root = options.root.resolve()
assert (root / "main/complete.txt").exists()
assert len(list((root / "main").glob("*/accepted.json"))) == 8
assert len(list((root / "preflight").glob("*/accepted.json"))) == 4
assert (root / "main/profile-summary.json").exists()
source_root = Path("/bench/zfs/db/blink-checkpoint-costs-20261023")
source = source_root / "source"
source_sha = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
assert source_sha == "cc80d1fdc7a98792be1a739056a3013debf39268"
assert not subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True).strip()


def digest(path):
    checksum = hashlib.sha256()
    with path.open("rb") as source_file:
        for chunk in iter(lambda: source_file.read(1024 * 1024), b""):
            checksum.update(chunk)
    return checksum.hexdigest()


identities = json.loads((source_root / "binary-identities.json").read_text())
for name, identity in identities.items():
    assert digest(source_root / "bin" / name) == identity["sha256"]
(root / "final-identity.json").write_text(json.dumps({"source_sha": source_sha, "source_clean": True,
                                                    "binaries": identities}, indent=2) + "\n")
archive = root / "backend-profiling-20261026-evidence.tar.gz"
sidecar = archive.with_suffix(archive.suffix + ".json")
assert not archive.exists()
files = []
excluded = []
for path in sorted(root.rglob("*")):
    relative = path.relative_to(root)
    if "databases" in relative.parts:
        if path.is_file():
            excluded.append({"path": relative.as_posix(), "bytes": path.stat().st_size})
        continue
    if not path.is_file() or path in [archive, sidecar] or path.name == "archive-inventory.json":
        continue
    files.append({"path": relative.as_posix(), "bytes": path.stat().st_size, "sha256": digest(path)})
inventory = root / "archive-inventory.json"
inventory.write_text(json.dumps({"files": files, "excluded_databases_retained_on_host": excluded,
                                 "excluded_binaries": str(source_root / "bin")}, indent=2) + "\n")
with tarfile.open(archive, "w:gz", compresslevel=6) as output:
    output.add(inventory, arcname="archive-inventory.json", recursive=False)
    for entry in files:
        output.add(root / entry["path"], arcname=entry["path"], recursive=False)
metadata = {"archive": archive.name, "sha256": digest(archive), "bytes": archive.stat().st_size,
            "inventoried_files": len(files), "uncompressed_inventoried_bytes": sum(entry["bytes"] for entry in files),
            "excluded_database_files": len(excluded), "excluded_database_bytes": sum(entry["bytes"] for entry in excluded)}
sidecar.write_text(json.dumps(metadata, indent=2) + "\n")
print(json.dumps(metadata), flush=True)
