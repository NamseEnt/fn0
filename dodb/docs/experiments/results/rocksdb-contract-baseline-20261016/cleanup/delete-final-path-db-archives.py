import hashlib
import json
import subprocess
import sys
from pathlib import Path

archive_entries = [{"bytes": 14491544, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/001-rw95-serialized-final-after-rep1/db.tar.gz"}, {"bytes": 14175133, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/002-rw95-serialized-final-before-rep1/db.tar.gz"}, {"bytes": 27248172, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/003-rw95-reserved-ranges-final-before-rep1/db.tar.gz"}, {"bytes": 29597392, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/004-rw95-reserved-ranges-final-after-rep1/db.tar.gz"}, {"bytes": 13709907, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/005-rw80-serialized-final-before-rep1/db.tar.gz"}, {"bytes": 14739075, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/006-rw80-serialized-final-after-rep1/db.tar.gz"}, {"bytes": 34595188, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/007-rw80-reserved-ranges-final-after-rep1/db.tar.gz"}, {"bytes": 34427634, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/008-rw80-reserved-ranges-final-before-rep1/db.tar.gz"}, {"bytes": 15720329, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/009-rw50-serialized-final-after-rep1/db.tar.gz"}, {"bytes": 16267965, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/010-rw50-serialized-final-before-rep1/db.tar.gz"}, {"bytes": 38795461, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/011-rw50-reserved-ranges-final-before-rep1/db.tar.gz"}, {"bytes": 37129400, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/012-rw50-reserved-ranges-final-after-rep1/db.tar.gz"}, {"bytes": 16228206, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/013-rw20-serialized-final-before-rep1/db.tar.gz"}, {"bytes": 17297398, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/014-rw20-serialized-final-after-rep1/db.tar.gz"}, {"bytes": 41617476, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/015-rw20-reserved-ranges-final-after-rep1/db.tar.gz"}, {"bytes": 41007313, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/016-rw20-reserved-ranges-final-before-rep1/db.tar.gz"}, {"bytes": 33791206, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/017-mt-mix-serialized-final-after-rep1/db.tar.gz"}, {"bytes": 35194958, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/018-mt-mix-serialized-final-before-rep1/db.tar.gz"}, {"bytes": 45954533, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/019-mt-mix-reserved-ranges-final-before-rep1/db.tar.gz"}, {"bytes": 51235454, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/020-mt-mix-reserved-ranges-final-after-rep1/db.tar.gz"}, {"bytes": 2500323, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/smoke/001-smoke-serialized-final-before-rep0/db.tar.gz"}, {"bytes": 2488438, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/smoke/002-smoke-serialized-final-before-rep0/db.tar.gz"}, {"bytes": 2591315, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/smoke/003-smoke-serialized-final-after-rep0/db.tar.gz"}, {"bytes": 2580030, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/smoke/004-smoke-serialized-final-after-rep0/db.tar.gz"}, {"bytes": 2572454, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/smoke/005-smoke-reserved-ranges-final-before-rep0/db.tar.gz"}, {"bytes": 2531227, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/smoke/006-smoke-reserved-ranges-final-before-rep0/db.tar.gz"}, {"bytes": 2506579, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/smoke/007-smoke-reserved-ranges-final-after-rep0/db.tar.gz"}, {"bytes": 2412667, "path": "/bench/zfs/db/dodb-final-path-20261016/measurements/smoke/008-smoke-reserved-ranges-final-after-rep0/db.tar.gz"}]
retained_paths = ["/bench/zfs/db/dodb-final-path-20261016/measurements/runs/017-mt-mix-serialized-final-after-rep1/db.tar.gz", "/bench/zfs/db/dodb-final-path-20261016/measurements/runs/020-mt-mix-reserved-ranges-final-after-rep1/db.tar.gz"]
mode = sys.argv[1] if len(sys.argv) > 1 else "audit"

if mode == "delete":
    process_result = subprocess.run(["ps", "-eo", "comm,args"], check=True, capture_output=True, text=True)
    active_matches = [line for line in process_result.stdout.splitlines()[1:] if any(token in line.lower() for token in ("cargo", "rustc", "service_workload", "crossdb", "rocksdb", "blink", "fixed-group", "product"))]
    if active_matches:
        raise SystemExit("active build or benchmark process found: " + " | ".join(active_matches))

for archive_entry in archive_entries:
    archive_path = Path(archive_entry["path"])
    archive_text = str(archive_path)
    if not archive_text.startswith("/bench/zfs/db/dodb-final-path-20261016/measurements/") or archive_path.name != "db.tar.gz":
        print(json.dumps({"status": "skip", "path": archive_text, "reason": "outside-current-final-measurement-archive-scope"}), flush=True)
        continue
    if archive_path.is_symlink() or not archive_path.is_file() or archive_path.resolve(strict=True) != archive_path:
        print(json.dumps({"status": "skip", "path": archive_text, "reason": "missing-symlink-or-noncanonical"}), flush=True)
        continue
    file_stat = archive_path.stat()
    if file_stat.st_size != archive_entry["bytes"]:
        print(json.dumps({"status": "skip", "path": archive_text, "reason": "size-changed", "expected_bytes": archive_entry["bytes"], "actual_bytes": file_stat.st_size}), flush=True)
        continue
    hash_algorithm = hashlib.sha256()
    with archive_path.open("rb") as archive_file:
        while True:
            data_chunk = archive_file.read(1024 * 1024)
            if not data_chunk:
                break
            hash_algorithm.update(data_chunk)
    archive_hash = hash_algorithm.hexdigest()
    if archive_text in retained_paths:
        result_status = "retained"
    elif mode == "delete":
        result_status = "verified"
    else:
        result_status = "candidate"
    journal_record = {"status": result_status, "path": archive_text, "bytes": file_stat.st_size, "allocated_bytes": file_stat.st_blocks * 512, "sha256": archive_hash, "reason": "representative-current-final-archive" if archive_text in retained_paths else "completed-final-measurement-database-archive"}
    print(json.dumps(journal_record, sort_keys=True), flush=True)
    if mode == "delete" and archive_text not in retained_paths:
        archive_path.unlink()
        journal_record["status"] = "deleted"
        print(json.dumps(journal_record, sort_keys=True), flush=True)
