import hashlib
import json
import sys
import tarfile
from pathlib import Path


path = Path(sys.argv[1])
summary = json.loads(path.with_suffix(path.suffix + ".json").read_text())
assert hashlib.sha256(path.read_bytes()).hexdigest() == summary["sha256"]
with tarfile.open(path, "r:gz") as archive:
    inventory = json.load(archive.extractfile("evidence-file-sha256.json"))
    for entry in inventory:
        contents = archive.extractfile(entry["path"]).read()
        assert len(contents) == entry["size_bytes"]
        assert hashlib.sha256(contents).hexdigest() == entry["sha256"]
assert len(inventory) == summary["files"]
print(json.dumps({"verified": True, "files": len(inventory), "sha256": summary["sha256"]}, indent=2))
