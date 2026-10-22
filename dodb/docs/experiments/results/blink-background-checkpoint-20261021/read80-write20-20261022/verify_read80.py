import hashlib
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path


def digest(data):
    return hashlib.sha256(data).hexdigest()


def verify(archive):
    record = json.loads(archive.with_suffix(archive.suffix + ".json").read_text())
    assert archive.stat().st_size == record["bytes"]
    assert digest(archive.read_bytes()) == record["sha256"]
    results = []
    with tarfile.open(archive, "r:gz") as contents, tempfile.TemporaryDirectory(prefix="dodb-read80-review-") as temporary:
        members = contents.getmembers()
        assert len(members) == record["files"]
        extracted_files = {}
        for member in members:
            path = Path(member.name)
            assert member.isfile() and not path.is_absolute() and ".." not in path.parts
            assert member.name not in extracted_files
            data = contents.extractfile(member).read()
            extracted_files[member.name] = digest(data)
            target = Path(temporary) / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        for inventory in sorted(Path(temporary).glob("*/evidence-file-sha256.json")):
            root = inventory.parent
            prefix = root.name + "/"
            expected = json.loads(inventory.read_text())
            observed = {name.removeprefix(prefix): value for name, value in extracted_files.items() if name.startswith(prefix)}
            assert set(observed) == set(expected) | {inventory.name}
            assert all(observed[name] == value for name, value in expected.items())
            summary_path = root / "matrix-summary.json"
            original = summary_path.read_bytes()
            subprocess.run([sys.executable, str(Path(__file__).with_name("summarize_read80.py")), str(root)], capture_output=True, text=True, check=True)
            assert summary_path.read_bytes() == original
            summary = json.loads(original)
            results.append({"root": root.name, "cells": len(summary["cells"]), "smoke": summary["plan"]["smoke"], "inventory_verified": True, "offline_summary_matches": True})
    report = {"archive_sha256_verified": True, "files": record["files"], "roots": results}
    archive.with_name("archive-verification.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    verify(Path(sys.argv[1]))
