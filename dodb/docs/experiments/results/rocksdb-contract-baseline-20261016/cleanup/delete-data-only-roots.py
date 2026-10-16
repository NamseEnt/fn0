import json
import os
import shutil
from pathlib import Path

roots = {
    "/bench/zfs/db/fixed-group-smoke-data-d01da7ab": {"extensions": {".db", ".wal"}, "expected_files": 12},
    "/bench/zfs/db/product-baseline-data-e5ce8530": {"extensions": {".db", ".db-log", ".db-wal"}, "expected_files": 3},
    "/bench/zfs/db/wal-sync-payload-data-d01da7ab": {"extensions": {".bin"}, "expected_files": 36},
    "/bench/zfs/db/experiment-data": {"extensions": set(), "expected_files": 0},
    "/bench/zfs/db/oci-a1-borrowed-query16-datasets": {"extensions": set(), "expected_files": 0},
    "/bench/zfs/db/product-baseline-data-f893a005": {"extensions": set(), "expected_files": 0},
    "/bench/zfs/db/product-data-exact-f893a005-p1-p4": {"extensions": set(), "expected_files": 0},
    "/bench/zfs/db/product-data-turso-0.8.2-pre.2-f893a005": {"extensions": set(), "expected_files": 0},
    "/bench/zfs/db/product-data-turso-0.8.2-pre.2-smoke3": {"extensions": set(), "expected_files": 0},
    "/bench/zfs/db/product-data-turso-0.8.2-pre.2-tail-seedmatch-f893a005": {"extensions": set(), "expected_files": 0},
}

records = []
for root_string, specification in roots.items():
    root_path = Path(root_string)
    if root_path.is_symlink() or not root_path.is_dir():
        raise SystemExit(f"unsafe or missing root: {root_path}")
    files = []
    directories = []
    symlinks = []
    for current_root, directory_names, file_names in os.walk(root_path, topdown=True, followlinks=False):
        current_path = Path(current_root)
        for directory_name in list(directory_names):
            child_path = current_path / directory_name
            if child_path.is_symlink():
                symlinks.append(str(child_path))
                directory_names.remove(directory_name)
            else:
                directories.append(child_path)
        for file_name in file_names:
            file_path = current_path / file_name
            if file_path.is_symlink() or not file_path.is_file():
                symlinks.append(str(file_path))
                continue
            files.append(file_path)
    if symlinks:
        raise SystemExit(f"symlink or non-regular file under {root_path}: {symlinks[:10]}")
    if len(files) != specification["expected_files"]:
        raise SystemExit(f"unexpected file count under {root_path}: {len(files)}")
    unexpected = [str(file_path) for file_path in files if file_path.suffix not in specification["extensions"]]
    if unexpected:
        raise SystemExit(f"unexpected file types under {root_path}: {unexpected[:10]}")
    if not specification["extensions"] and files:
        raise SystemExit(f"expected an empty root: {root_path}")
    for file_path in files:
        stat_result = file_path.lstat()
        allocated_bytes = stat_result.st_blocks * 512
        logical_bytes = stat_result.st_size
        file_path.unlink()
        records.append({"root": root_string, "path": str(file_path), "logical_bytes": logical_bytes, "allocated_bytes": allocated_bytes, "result": "deleted"})
    for directory_path in sorted(directories, key=lambda path: len(path.parts), reverse=True):
        directory_path.rmdir()
    if files or directories:
        root_path.rmdir()
    print(json.dumps({"root": root_string, "file_count": len(files), "result": "deleted" if files or directories else "empty-root-kept"}, sort_keys=True))

for record in records:
    print(json.dumps(record, sort_keys=True))
