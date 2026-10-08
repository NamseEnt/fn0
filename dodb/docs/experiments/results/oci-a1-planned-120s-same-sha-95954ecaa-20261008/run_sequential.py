import csv
import datetime
import hashlib
import json
import os
import pathlib
import shlex
import subprocess
import time

root = pathlib.Path(__file__).resolve().parent
remote_root = pathlib.Path("/bench/zfs/db/oci-a1-planned-120s-same-sha-95954ecaa-20261008")
source = pathlib.Path("/bench/zfs/db/oci-a1-query-cursor-95954ecaa/candidate")
binary = pathlib.Path("/bench/zfs/db/oci-a1-query-cursor-95954ecaa/target-candidate/aarch64-unknown-linux-gnu/release/phase0-bench")
manifest = list(csv.DictReader((root / "manifest.tsv").open(), delimiter="\t"))
status_path = root / "execution-status.tsv"
with status_path.open("w") as status_file:
    status_file.write("run_id\tstarted_utc\tended_utc\tactual_elapsed_seconds\texit_code\tjsonl_records\tresult_duration_ms\tstate\n")
for run in manifest:
    output_path = root / run["jsonl"]
    log_path = root / run["log"]
    data_dir = remote_root / "db" / run["run_id"]
    tmp_dir = remote_root / "tmp" / run["run_id"]
    if data_dir.exists() or output_path.exists() or log_path.exists():
        raise RuntimeError(f"refusing to overwrite existing path for {run['run_id']}")
    data_dir.mkdir(parents=True)
    tmp_dir.mkdir(parents=True)
    environment = os.environ.copy()
    environment["DODB_BENCH_DIR"] = str(data_dir)
    environment["TMPDIR"] = str(tmp_dir)
    command = shlex.split(run["command"])
    started_utc = datetime.datetime.now(datetime.timezone.utc).isoformat()
    started_ns = time.monotonic_ns()
    binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    with log_path.open("wb") as log_file:
        process = subprocess.Popen(command, cwd=source, env=environment, stdout=log_file, stderr=subprocess.STDOUT)
        exit_code = process.wait()
    ended_ns = time.monotonic_ns()
    ended_utc = datetime.datetime.now(datetime.timezone.utc).isoformat()
    elapsed_seconds = (ended_ns - started_ns) / 1_000_000_000
    jsonl_records = []
    if output_path.exists():
        jsonl_records = [json.loads(line) for line in output_path.read_text().splitlines() if line.strip()]
    record_duration_ms = jsonl_records[0].get("duration_ms", "") if len(jsonl_records) == 1 else ""
    state = "process-exited" if exit_code == 0 and len(jsonl_records) == 1 else "invalid-output"
    event = {"run_id": run["run_id"], "started_utc": started_utc, "ended_utc": ended_utc, "actual_elapsed_seconds": elapsed_seconds, "exit_code": exit_code, "jsonl_records": len(jsonl_records), "result_duration_ms": record_duration_ms, "state": state, "binary_sha256": binary_hash, "command": command, "environment": {"DODB_BENCH_DIR": str(data_dir), "TMPDIR": str(tmp_dir)}}
    with (root / "runner-events.jsonl").open("a") as event_file:
        event_file.write(json.dumps(event, sort_keys=True) + "\n")
    with status_path.open("a") as status_file:
        status_file.write(f"{run['run_id']}\t{started_utc}\t{ended_utc}\t{elapsed_seconds:.9f}\t{exit_code}\t{len(jsonl_records)}\t{record_duration_ms}\t{state}\n")
    print(json.dumps({"run_id": run["run_id"], "exit_code": exit_code, "jsonl_records": len(jsonl_records), "duration_ms": record_duration_ms, "actual_elapsed_seconds": elapsed_seconds, "binary_sha256": binary_hash}), flush=True)
    if state != "process-exited":
        break
