import argparse
import datetime
import hashlib
import json
import os
import pathlib
import shlex
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument("--output", required=True)
parser.add_argument("--log", required=True)
parser.add_argument("--data-dir", required=True)
parser.add_argument("--tmpdir", required=True)
parser.add_argument("--event", required=True)
args = parser.parse_args()

source = pathlib.Path("/bench/zfs/db/oci-a1-query-cursor-95954ecaa/candidate").resolve()
binary = pathlib.Path("/bench/zfs/db/oci-a1-query-cursor-95954ecaa/target-candidate/aarch64-unknown-linux-gnu/release/phase0-bench").resolve()
output = pathlib.Path(args.output).expanduser().resolve()
log = pathlib.Path(args.log).expanduser().resolve()
data_dir = pathlib.Path(args.data_dir).expanduser().resolve()
tmpdir = pathlib.Path(args.tmpdir).expanduser().resolve()
event_path = pathlib.Path(args.event).expanduser().resolve()
expected_commit = "95954ecaaae94757cc3705bc2aacf8c6bb78915f"
expected_binary_sha256 = "89c6e622b3f05c597a740353b30464bbc30cbe3e910195bfc545967a9dca4c87"
expected_source_sha256 = "bae7b7b6fe13223c6ef579715c773b87a843b1b8caeeff4659a67818d7387e90"
expected_lock_sha256 = "9702ecfb45d6edab98f5aa3e849ad6f456faed7f75d5b37bd9b545eb2d5d4da6"

for path in (output, log, data_dir, tmpdir, event_path):
    if not path.is_absolute():
        raise RuntimeError(f"path did not resolve to absolute path: {path}")
    try:
        path.relative_to(source)
    except ValueError:
        pass
    else:
        raise RuntimeError(f"output artifact path is inside the checkout: {path}")
    if path.exists():
        raise RuntimeError(f"refusing to overwrite existing path: {path}")

if subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip() != expected_commit:
    raise RuntimeError("source checkout SHA mismatch")
if subprocess.check_output(["git", "-C", str(source), "status", "--porcelain=v1"], text=True).strip():
    raise RuntimeError("source checkout is not clean")

def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as input_file:
        for chunk in iter(lambda: input_file.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()

binary_hash_before = sha256(binary)
source_hash_before = sha256(source / "dodb/crates/dodb-storage/src/bin/phase0-bench.rs")
lock_hash_before = sha256(source / "Cargo.lock")
if binary_hash_before != expected_binary_sha256:
    raise RuntimeError("verified binary SHA-256 mismatch")
if source_hash_before != expected_source_sha256 or lock_hash_before != expected_lock_sha256:
    raise RuntimeError("source file or Cargo.lock SHA-256 mismatch")

filesystem = subprocess.check_output(["findmnt", "-T", str(data_dir.parent), "-o", "TARGET,FSTYPE,SOURCE,OPTIONS", "-n"], text=True).strip()
if "zfs" not in filesystem.split() or not filesystem.startswith("/bench/zfs/db "):
    raise RuntimeError(f"measurement storage is not on the confirmed ZFS mount: {filesystem}")

output.parent.mkdir(parents=True, exist_ok=False)
log.parent.mkdir(parents=True, exist_ok=True)
data_dir.mkdir(parents=True, exist_ok=False)
tmpdir.mkdir(parents=True, exist_ok=False)
event_path.parent.mkdir(parents=True, exist_ok=True)
environment = os.environ.copy()
environment["DODB_BENCH_DIR"] = str(data_dir)
environment["TMPDIR"] = str(tmpdir)
command = [
    str(binary), "--engine", "main-btree", "--suite", "read", "--readers", "1",
    "--read-kinds", "get", "--distributions", "uniform", "--tokio-workers", "2",
    "--working-set", "4096", "--key-size", "16", "--value-size", "64",
    "--cache-capacity", "256", "--read-limit", "16", "--blink-collection-policy", "current",
    "--blink-workers", "2", "--parallel-workers", "0", "--group-limit", "64",
    "--group-bytes", "4194304", "--queue-capacity", "256", "--transaction-mode", "unconditional",
    "--sync-mode", "real", "--warmup", "1s", "--duration", "120s", "--repetitions", "1",
    "--seed", "0xd0db2026", "--output", str(output),
]
started_utc = datetime.datetime.now(datetime.timezone.utc).isoformat()
started_ns = time.monotonic_ns()
with log.open("xb") as log_file:
    process = subprocess.Popen(command, cwd=source, env=environment, stdout=log_file, stderr=subprocess.STDOUT)
    exit_code = process.wait()
ended_ns = time.monotonic_ns()
ended_utc = datetime.datetime.now(datetime.timezone.utc).isoformat()
records = [json.loads(line) for line in output.read_text().splitlines() if line.strip()] if output.exists() else []
event = {
    "run_id": "09-get-main-btree-provenance-clean-r1",
    "started_utc": started_utc,
    "ended_utc": ended_utc,
    "actual_elapsed_seconds": (ended_ns - started_ns) / 1_000_000_000,
    "exit_code": exit_code,
    "jsonl_records": len(records),
    "result_duration_ms": records[0].get("duration_ms", "") if len(records) == 1 else "",
    "state": "process-exited" if exit_code == 0 and len(records) == 1 else "invalid-output",
    "binary_sha256": sha256(binary),
    "command": command,
    "environment": {"DODB_BENCH_DIR": str(data_dir), "TMPDIR": str(tmpdir)},
    "source_commit_before": expected_commit,
    "source_status_before": "clean",
    "binary_sha256_before": binary_hash_before,
    "source_sha256_before": source_hash_before,
    "cargo_lock_sha256_before": lock_hash_before,
    "filesystem": filesystem,
    "output_path": str(output),
    "log_path": str(log),
}
event["source_commit_after"] = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
event["source_status_after"] = subprocess.check_output(["git", "-C", str(source), "status", "--porcelain=v1"], text=True).strip()
event["binary_sha256_after"] = sha256(binary)
event["source_sha256_after"] = sha256(source / "dodb/crates/dodb-storage/src/bin/phase0-bench.rs")
event["cargo_lock_sha256_after"] = sha256(source / "Cargo.lock")
with event_path.open("x") as event_file:
    event_file.write(json.dumps(event, sort_keys=True, indent=2) + "\n")
print(json.dumps(event, sort_keys=True))
if event["state"] != "process-exited" or event["source_commit_after"] != expected_commit or event["source_status_after"] or event["binary_sha256_after"] != expected_binary_sha256 or event["source_sha256_after"] != expected_source_sha256 or event["cargo_lock_sha256_after"] != expected_lock_sha256:
    raise SystemExit(1)
