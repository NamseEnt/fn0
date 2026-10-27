import hashlib
import json
import os
import shutil
from pathlib import Path


root = Path(os.environ["DODB_HOTPATH_ROOT"])
destination_root = Path("/home/opc") / f"{root.name}-database-archives"
minimum_free = 3 * 1024**3
barrier = json.loads((root / "short-stage-barrier.json").read_text())
parent_state = Path(f"/proc/{barrier['pipeline_pid']}/stat").read_text().rsplit(")", 1)[1].split()[0]
assert parent_state in {"T", "t"}
assert not (root / "sustained").exists()


def digest(path):
    value = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            value.update(chunk)
    return value.hexdigest()


def sync_directory(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


records = []
for stage in ["preflight", "short"]:
    assert (root / stage / "matrix-complete.txt").is_file()
    destination_directory = destination_root / stage
    destination_directory.mkdir(parents=True, exist_ok=True)
    for metadata_path in sorted((root / stage / "database-archives").glob("*.json")):
        metadata = json.loads(metadata_path.read_text())
        source = root / stage / "database-archives" / f"{metadata['case']}.tar.gz"
        assert str(source) == metadata["path"] and metadata["verified"]
        destination = destination_directory / source.name
        if source.is_symlink():
            assert source.resolve() == destination.resolve() and digest(destination) == metadata["sha256"]
        else:
            available = shutil.disk_usage(destination_directory).free
            if available < minimum_free + metadata["compressed_bytes"]:
                records.append({"source": str(source), "relocated": False, "reason": "boot filesystem reserve"})
                continue
            assert not destination.exists() and digest(source) == metadata["sha256"]
            temporary = destination.with_suffix(".gz.incomplete")
            assert not temporary.exists()
            shutil.copy2(source, temporary)
            with temporary.open("rb") as stream:
                os.fsync(stream.fileno())
            assert digest(temporary) == metadata["sha256"]
            os.replace(temporary, destination)
            sync_directory(destination_directory)
            redirect = source.with_suffix(".gz.redirect")
            assert not redirect.exists()
            redirect.symlink_to(destination)
            os.replace(redirect, source)
            sync_directory(source.parent)
        records.append({"source": str(source), "destination": str(destination), "sha256": metadata["sha256"],
                        "size_bytes": metadata["compressed_bytes"], "relocated": True, "verified": True})
        (root / "archive-relocations.json").write_text(json.dumps(records, indent=2) + "\n")
        print(json.dumps(records[-1]), flush=True)
(root / "archive-relocations.json").write_text(json.dumps(records, indent=2) + "\n")
