import json
import os
import subprocess
import sys
from pathlib import Path

candidate_paths = ["/bench/zfs/db/crossdb-10e3494e-oci/build/target-dodb", "/bench/zfs/db/crossdb-10e3494e-oci/build/target-rocksdb", "/bench/zfs/db/crossdb-1ac11e56-oci/build/target-dodb", "/bench/zfs/db/crossdb-1ac11e56-oci/build/target-rocksdb", "/bench/zfs/db/crossdb-33722487-oci/build/target-dodb", "/bench/zfs/db/crossdb-33722487-oci/build/target-rocksdb", "/bench/zfs/db/crossdb-3da68648-oci/build/target-dodb", "/bench/zfs/db/crossdb-3da68648-oci/build/target-rocksdb", "/bench/zfs/db/crossdb-4e35e99d-oci/build/target-dodb", "/bench/zfs/db/crossdb-4e35e99d-oci/build/target-rocksdb", "/bench/zfs/db/crossdb-82c2b6e1-oci/build/target-dodb", "/bench/zfs/db/crossdb-82c2b6e1-oci/build/target-rocksdb", "/bench/zfs/db/crossdb-aa83fab0-oci/build/target-dodb", "/bench/zfs/db/crossdb-aa83fab0-oci/build/target-rocksdb", "/bench/zfs/db/crossdb-be1c1786-oci/build/target-dodb", "/bench/zfs/db/crossdb-be1c1786-oci/build/target-rocksdb", "/bench/zfs/db/crossdb-c1ee7cbf-oci/build/target-dodb", "/bench/zfs/db/crossdb-c1ee7cbf-oci/build/target-rocksdb", "/bench/zfs/db/crossdb-d3f4958-oci/build/target-dodb", "/bench/zfs/db/crossdb-d3f4958-oci/build/target-rocksdb", "/bench/zfs/db/dodb-rocksdb-r1a-70105e0f871a25a9/target", "/bench/zfs/db/dodb-rocksdb-r1b1-2774bd1c3491134b/target", "/bench/zfs/db/dodb-rocksdb-r1b1-e59ae32117df13e2/target", "/bench/zfs/db/dodb-rocksdb-r1b2-558a9283c1a9/target", "/bench/zfs/db/dodb-rocksdb-r1b3a-551471d5/target", "/bench/zfs/db/experiment-build/b5aad16", "/bench/zfs/db/experiment-build/borrowed-page-b7c67715-target", "/bench/zfs/db/experiment-build/dodb-81f6ce268", "/bench/zfs/db/experiment-build/dodb-8a6f3dc", "/bench/zfs/db/oci-a1-borrowed-query16-4bc4d42e4d816f4a11428f5423d89a4c35494f9b/build-target-feature-on", "/bench/zfs/db/oci-a1-query-cursor-95954ecaa/target-base", "/bench/zfs/db/oci-a1-query-cursor-95954ecaa/target-candidate", "/bench/zfs/db/oci-phase0-followup-087406003b1edbe7e0a2991d54ae8e088e98baed/target", "/bench/zfs/db/oci-phase0-followup-4bc4d42e4d816f4a11428f5423d89a4c35494f9b/target", "/bench/zfs/db/oci-phase0-followup-ff864bf92461c9216e840d9eecf7bb04e52c41ce/target", "/bench/zfs/db/oci-phase0-measurement-5cee905cce6aa93d474112efdcaf561d31eadcf9/target", "/bench/zfs/db/oci-phase0-measurement-a277c01b5217a8cb8210b94c2be06131b6c4dc4e/target", "/bench/zfs/db/oci-phase0-measurement-aece5d4356f127d63181fd82613ead46bfb52aff/target", "/bench/zfs/db/product-build/dodb-product-target", "/bench/zfs/db/product-build/turso-0.8.2-pre.2-target"]
mode = sys.argv[1] if len(sys.argv) > 1 else "audit"
protected_prefixes = ("/bench/zfs/db/dodb-rocksdb-shared-cache/", "/bench/zfs/db/tooling/", "/bench/zfs/db/product-build/rocksdb-source/", "/bench/zfs/db/dodb-final-path-20261016/")

def inspect_target(target_value):
    target_path = Path(target_value)
    target_text = str(target_path)
    if not target_text.startswith("/bench/zfs/db/") or any(target_text.startswith(prefix) for prefix in protected_prefixes):
        return {"status": "skip", "path": target_text, "reason": "outside-old-target-scope"}
    if target_path.is_symlink() or not target_path.is_dir() or target_path.resolve(strict=True) != target_path:
        return {"status": "skip", "path": target_text, "reason": "missing-symlink-or-noncanonical"}
    immediate_names = {entry.name for entry in target_path.iterdir()}
    if not immediate_names.intersection({"CACHEDIR.TAG", ".rustc_info.json"}):
        return {"status": "skip", "path": target_text, "reason": "no-cargo-target-marker"}
    for directory_path, directory_names, file_names in os.walk(target_path, followlinks=False):
        directory_entry = Path(directory_path)
        if any((directory_entry / directory_name).is_symlink() for directory_name in directory_names):
            return {"status": "skip", "path": target_text, "reason": "symlink-descendant"}
        if any((directory_entry / file_name).is_symlink() for file_name in file_names):
            return {"status": "skip", "path": target_text, "reason": "symlink-descendant"}
    size_result = subprocess.run(["du", "-skx", "--", target_text], check=True, capture_output=True, text=True)
    allocated_kib = int(size_result.stdout.split()[0])
    return {"status": "safe", "path": target_text, "allocated_kib": allocated_kib, "reason": "old-cargo-build-target"}

if mode == "delete":
    process_result = subprocess.run(["ps", "-eo", "comm,args"], check=True, capture_output=True, text=True)
    active_matches = [line for line in process_result.stdout.splitlines()[1:] if any(token in line.lower() for token in ("cargo", "rustc", "service_workload", "crossdb", "rocksdb", "blink", "fixed-group", "product"))]
    if active_matches:
        raise SystemExit("active build or benchmark process found: " + " | ".join(active_matches))

for candidate_value in candidate_paths:
    result = inspect_target(candidate_value)
    if result["status"] == "safe" and mode == "delete":
        import shutil
        shutil.rmtree(result["path"])
        result["status"] = "deleted"
    print(json.dumps(result, sort_keys=True), flush=True)
