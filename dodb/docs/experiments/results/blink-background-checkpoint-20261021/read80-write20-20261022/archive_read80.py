import hashlib
import json
import os
import sys
import tarfile
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def archive_roots(roots, archive):
    all_paths = []
    for root in roots:
        assert (root / "matrix-complete.txt").exists(), root
        assert json.loads((root / "matrix-summary.json").read_text())["validation_passed"], root
        paths = []
        for directory, subdirectories, filenames in os.walk(root, followlinks=False):
            subdirectories[:] = [name for name in subdirectories if name not in {"source", "bin", "databases", "controller"} and not (Path(directory) / name).is_symlink()]
            paths.extend(Path(directory) / name for name in filenames if name != "evidence-file-sha256.json" and not (Path(directory) / name).is_symlink())
        paths.extend(path for path in (root / "controller").rglob("*") if path.is_file() and not path.is_symlink())
        paths.sort()
        inventory_path = root / "evidence-file-sha256.json"
        inventory_path.write_text(json.dumps({str(path.relative_to(root)): digest(path) for path in paths}, indent=2) + "\n")
        all_paths.extend(paths + [inventory_path])
    with tarfile.open(archive, "x:gz") as output:
        for path in sorted(all_paths):
            output.add(path, arcname=str(path.relative_to(roots[0].parent)), recursive=False)
    record = {"archive": archive.name, "bytes": archive.stat().st_size, "files": len(all_paths), "sha256": digest(archive)}
    archive.with_suffix(archive.suffix + ".json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record))


if __name__ == "__main__":
    archive_roots([Path(value) for value in sys.argv[2:]], Path(sys.argv[1]))
