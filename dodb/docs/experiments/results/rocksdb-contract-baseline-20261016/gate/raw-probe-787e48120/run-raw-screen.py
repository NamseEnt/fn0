import csv
import hashlib
import json
import os
import shutil
import subprocess
import time
from pathlib import Path

root = Path("/bench/zfs/db/rocksdb-contract-baseline-20261016/raw-screen-r1")
source_path = Path("/bench/zfs/db/rocksdb-contract-baseline-20261016/source-787e48120/checkout")
workspace_path = source_path / "dodb"
binary_dir = Path("/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target/release")
source_sha = "5d772a7448b1b6c29a0f00f501bd51fd83be03b9"
binaries = {
    "raw-rocksdb": binary_dir / "unconditional_write_workload",
    "synchronous-core": binary_dir / "unconditional_write_workload",
    "rocksdb-service": binary_dir / "unconditional_write_workload",
    "blink": binary_dir / "unconditional_write_workload",
}
if root.exists():
    raise SystemExit(f"screen root already exists: {root}")
root.mkdir(parents=True)
runs_root = root / "runs"
runs_root.mkdir()

def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as file_handle:
        for chunk in iter(lambda: file_handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()

if subprocess.check_output(["git", "-C", str(source_path), "rev-parse", "HEAD"], text=True).strip() != source_sha:
    raise SystemExit("clean source SHA mismatch")
if subprocess.check_output(["git", "-C", str(source_path), "status", "--porcelain"], text=True).strip():
    raise SystemExit("source checkout is dirty")
binary_sha = sha256_file(binaries["raw-rocksdb"])
if any(sha256_file(binary_path) != binary_sha for binary_path in binaries.values()):
    raise SystemExit("backend workload binaries do not share one binary hash")
binary_sha_expected = "8ee11c9afeff6c3687643ad14d6f1c35597d984f988d4f8290935547566d8d48"
if binary_sha != binary_sha_expected:
    raise SystemExit(f"workload binary SHA mismatch: {binary_sha}")

memory = Path("/proc/meminfo").read_text()
mem_available_kb = int(next(line.split()[1] for line in memory.splitlines() if line.startswith("MemAvailable:")))
disk_usage = shutil.disk_usage("/bench/zfs/db")
if disk_usage.free < 20 * 1024**3:
    raise SystemExit(f"insufficient ZFS preflight space: {disk_usage.free}")
if mem_available_kb < 2 * 1024**2:
    raise SystemExit(f"insufficient available memory: {mem_available_kb} kB")
active_names = {"cargo", "service_workload", "unconditional_write_workload"}
active_processes = []
for process_path in Path("/proc").iterdir():
    if not process_path.name.isdigit() or int(process_path.name) in {os.getpid(), os.getppid()}:
        continue
    try:
        process_name = (process_path / "comm").read_text().strip()
        if process_name in active_names:
            active_processes.append((process_path.name, process_name))
    except OSError:
        pass
if active_processes:
    raise SystemExit(f"other build or workload processes are active: {active_processes}")
preflight = {
    "utc": subprocess.check_output(["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], text=True).strip(),
    "hostname": subprocess.check_output(["hostname"], text=True).strip(),
    "uname": subprocess.check_output(["uname", "-a"], text=True).strip(),
    "lscpu": subprocess.check_output(["lscpu"], text=True),
    "meminfo": memory,
    "mem_available_kb": mem_available_kb,
    "df": subprocess.check_output(["df", "-Pk", "/", "/bench/zfs/db"], text=True),
    "zfs": subprocess.check_output(["zfs", "list", "-Hp", "-o", "name,used,available,mountpoint", "dodbbench/db"], text=True),
    "source_sha": source_sha,
    "source_tree": subprocess.check_output(["git", "-C", str(source_path), "rev-parse", "HEAD^{tree}"], text=True).strip(),
    "binary_sha256": binary_sha,
    "binary_path": str(binaries["raw-rocksdb"]),
    "rocksdb_source_sha": subprocess.check_output(["git", "-C", "/bench/zfs/db/product-build/rocksdb-source", "rev-parse", "HEAD"], text=True).strip(),
    "rocksdb_archive_sha256": subprocess.check_output(["sha256sum", "/bench/zfs/db/product-build/rocksdb-source/librocksdb.a"], text=True).split()[0],
    "active_build_or_workload_processes": active_processes,
    "resident_dataset_payload_max_bytes": 64 * 256 * 16 * 512,
    "memory_exceed_axis": "deferred by user",
    "case_budget": {"warmup_ms": 1000, "measurement_ms": 5000, "timeout_seconds": 180, "minimum_zfs_free_bytes": 20 * 1024**3},
}
(root / "preflight.json").write_text(json.dumps(preflight, indent=2) + "\n")

slot_profiles_16 = [
    ("raw-rocksdb", 8),
    ("synchronous-core", 8),
    ("rocksdb-service", 16),
    ("blink", 8),
]
slot_profiles_64 = [
    ("raw-rocksdb", 8),
    ("synchronous-core", 8),
    ("blink", 8),
    ("rocksdb-service", 16),
    ("rocksdb-service", 32),
    ("rocksdb-service", 64),
]
case_results = []
prefix_fingerprints = {}
run_index = 0
batch_started = time.monotonic()
for width in (1, 16):
    for clients in (16, 64):
        profiles = slot_profiles_16 if clients == 16 else slot_profiles_64
        for repetition in range(1, 4):
            rotation = (repetition - 1) % len(profiles)
            rotated_profiles = profiles[rotation:] + profiles[:rotation]
            for backend, shared_slots in rotated_profiles:
                run_index += 1
                run_id = f"r{run_index:02d}-{backend}-c{clients}-w{width}-rep{repetition}-slots{shared_slots}"
                case_dir = runs_root / run_id
                database_path = case_dir / "db"
                report_path = case_dir / "report.json"
                output_path = case_dir / "output.txt"
                case_dir.mkdir()
                current_free = shutil.disk_usage("/bench/zfs/db").free
                if current_free < 20 * 1024**3:
                    raise SystemExit(f"per-case ZFS budget below 20 GiB before {run_id}: {current_free}")
                estimated_payload_bytes = clients * 256 * width * 512
                command = [
                    str(binaries[backend]),
                    "--backend", backend,
                    "--width", str(width),
                    "--clients", str(clients),
                    "--seed", "20261013",
                    "--value-size", "512",
                    "--warmup-ms", "1000",
                    "--measurement-ms", "5000",
                    "--call-cap", "100000000",
                    "--shared-blocking-slots", str(shared_slots),
                    "--data-dir", str(database_path),
                    "--report", str(report_path),
                ]
                if backend in {"synchronous-core", "rocksdb-service"}:
                    command.extend(["--revision-allocator", "reserved-ranges"])
                environment = dict(os.environ)
                environment.update({"DODB_SOURCE_SHA": source_sha, "DODB_BINARY_SHA256": binary_sha})
                (case_dir / "command.json").write_text(json.dumps({
                    "run_id": run_id,
                    "index": run_index,
                    "backend": backend,
                    "clients": clients,
                    "width": width,
                    "repetition": repetition,
                    "shared_blocking_slots": shared_slots,
                    "estimated_payload_bytes": estimated_payload_bytes,
                    "zfs_free_bytes_before": current_free,
                    "environment": {"DODB_SOURCE_SHA": source_sha, "DODB_BINARY_SHA256": binary_sha},
                    "argv": command,
                }, indent=2) + "\n")
                started = time.monotonic()
                try:
                    with output_path.open("w") as output_file:
                        completed = subprocess.run(command, cwd=workspace_path, env=environment, stdout=output_file, stderr=subprocess.STDOUT, timeout=180, check=False)
                except subprocess.TimeoutExpired as error:
                    (case_dir / "exit-status.txt").write_text("timeout\n")
                    raise SystemExit(f"run timeout; database and output preserved: {run_id}") from error
                elapsed_seconds = time.monotonic() - started
                (case_dir / "exit-status.txt").write_text(f"{completed.returncode}\n")
                if completed.returncode != 0 or not report_path.is_file():
                    raise SystemExit(f"run failed; database and output preserved: {run_id}, exit={completed.returncode}")
                report = json.loads(report_path.read_text())
                if report.get("source_sha") != source_sha or report.get("binary_sha256") != binary_sha:
                    raise SystemExit(f"report provenance mismatch; database preserved: {run_id}")
                configuration = report.get("configuration", {})
                if configuration.get("clients") != clients or configuration.get("width") != width or configuration.get("seed") != 20261013 or configuration.get("value_size_bytes") != 512:
                    raise SystemExit(f"workload configuration mismatch; database preserved: {run_id}")
                if configuration.get("raw_key_prefix_bytes") != 1 or configuration.get("canonical_document_key_encoding") is not True:
                    raise SystemExit(f"canonical input prefix configuration mismatch; database preserved: {run_id}")
                if backend in {"synchronous-core", "rocksdb-service"} and configuration.get("revision_allocator") != "ReservedRanges":
                    raise SystemExit(f"reserved-ranges allocator missing; database preserved: {run_id}")
                if backend == "rocksdb-service" and configuration.get("shared_blocking_slots") != shared_slots:
                    raise SystemExit(f"service slot count mismatch; database preserved: {run_id}")
                timing = report.get("timing", {})
                if timing.get("measurement_complete") is not True or timing.get("accepted_as_full_window") is not True or timing.get("termination_reason") != "requested-duration" or timing.get("requested_measurement_ms") != 5000:
                    raise SystemExit(f"measurement window incomplete; database preserved: {run_id}")
                counters = report.get("counters", {})
                if counters.get("other_errors") != 0 or counters.get("overloads") != 0 or counters.get("request_limit_reached") is not False:
                    raise SystemExit(f"counter error or cap reached; database preserved: {run_id}")
                final_validation = report.get("final_validation", {})
                reopen_validation = report.get("reopen_validation", {})
                if final_validation.get("passed") is not True or reopen_validation.get("passed") is not True:
                    raise SystemExit(f"final or reopen state oracle failed; database preserved: {run_id}")
                expected_rows = clients * 256 * width
                if final_validation.get("expected_rows") != expected_rows or final_validation.get("actual_rows") != expected_rows or reopen_validation.get("actual_rows") != expected_rows:
                    raise SystemExit(f"row count mismatch; database preserved: {run_id}")
                if final_validation.get("missing_rows") != 0 or final_validation.get("extra_rows") != 0 or final_validation.get("value_or_revision_mismatches") != 0:
                    raise SystemExit(f"final state mismatch; database preserved: {run_id}")
                if reopen_validation.get("missing_rows") != 0 or reopen_validation.get("extra_rows") != 0 or reopen_validation.get("value_or_revision_mismatches") != 0:
                    raise SystemExit(f"reopen state mismatch; database preserved: {run_id}")
                if backend == "raw-rocksdb":
                    if configuration.get("raw_has_logical_revision_occ_aba") is not False:
                        raise SystemExit(f"raw contract claim boundary missing: {run_id}")
                else:
                    if final_validation.get("same_transaction_revision_group_mismatches") != 0 or reopen_validation.get("same_transaction_revision_group_mismatches") != 0:
                        raise SystemExit(f"transaction revision grouping mismatch: {run_id}")
                if timing.get("actual_write_batches") != report.get("latency", {}).get("count") or report["latency"].get("sample_count") != timing.get("actual_write_batches"):
                    raise SystemExit(f"latency and acknowledged operation count mismatch: {run_id}")
                if counters.get("measurement_write_batches_acknowledged") != timing.get("actual_write_batches") or counters.get("acknowledged_keys") != timing.get("actual_write_batches") * width:
                    raise SystemExit(f"acknowledgment count mismatch: {run_id}")
                client_rows = report.get("per_client", [])
                if len(client_rows) != clients:
                    raise SystemExit(f"per-client report count mismatch: {run_id}")
                fingerprints = {}
                for client_row in client_rows:
                    client_index = client_row["client_index"]
                    if client_row.get("measurement_input_prefix_batches") != 100:
                        raise SystemExit(f"input prefix incomplete: {run_id} client={client_index}")
                    fingerprint = client_row.get("measurement_input_prefix_fingerprint_fnv1a64")
                    if not fingerprint:
                        raise SystemExit(f"input prefix fingerprint missing: {run_id} client={client_index}")
                    fingerprints[client_index] = fingerprint
                    prefix_key = (clients, width, repetition, client_index)
                    previous_fingerprint = prefix_fingerprints.setdefault(prefix_key, fingerprint)
                    if previous_fingerprint != fingerprint:
                        raise SystemExit(f"paired input prefix differs: {run_id} client={client_index}")
                process_metrics = report.get("process_metrics", {})
                if process_metrics.get("rss_sample_count", 0) < 2 or process_metrics.get("peak_sampled_rss_kb", 0) <= 0 or process_metrics.get("cpu_seconds") is None:
                    raise SystemExit(f"CPU/RSS measurement incomplete: {run_id}")
                disk_bytes = report.get("disk_usage_bytes")
                if not isinstance(disk_bytes, int) or disk_bytes <= 0:
                    raise SystemExit(f"disk usage measurement missing: {run_id}")
                run_summary = {
                    "run_id": run_id,
                    "index": run_index,
                    "backend": backend,
                    "clients": clients,
                    "width": width,
                    "repetition": repetition,
                    "shared_blocking_slots": shared_slots if backend == "rocksdb-service" else None,
                    "source_sha": source_sha,
                    "binary_sha256": binary_sha,
                    "duration_seconds": elapsed_seconds,
                    "write_batches": timing["actual_write_batches"],
                    "write_batches_per_second": timing["write_batches_per_second"],
                    "latency_p50_nanos": report["latency"]["p50_nanos"],
                    "latency_p95_nanos": report["latency"]["p95_nanos"],
                    "latency_p99_nanos": report["latency"]["p99_nanos"],
                    "latency_sample_count": report["latency"]["sample_count"],
                    "peak_sampled_rss_kb": process_metrics["peak_sampled_rss_kb"],
                    "cpu_seconds": process_metrics["cpu_seconds"],
                    "disk_usage_bytes": disk_bytes,
                    "input_prefix_fingerprints": fingerprints,
                    "final_validation": final_validation,
                    "reopen_validation": reopen_validation,
                }
                (case_dir / "validated-summary.json").write_text(json.dumps(run_summary, indent=2) + "\n")
                case_results.append(run_summary)
                print(f"pass {run_index}/60 {backend} clients={clients} width={width} rep={repetition} slots={shared_slots} batches={timing['actual_write_batches']} rps={timing['write_batches_per_second']:.1f} rss_kb={process_metrics['peak_sampled_rss_kb']}", flush=True)

if run_index != 60:
    raise SystemExit(f"unexpected run count: {run_index}")
expected_prefix_keys = {
    (clients, width, repetition, client_index)
    for width in (1, 16)
    for clients in (16, 64)
    for repetition in (1, 2, 3)
    for client_index in range(clients)
}
if set(prefix_fingerprints) != expected_prefix_keys:
    raise SystemExit("paired input prefix verification is incomplete")
fields = [
    "run_id", "index", "backend", "clients", "width", "repetition", "shared_blocking_slots",
    "source_sha", "binary_sha256", "duration_seconds", "write_batches", "write_batches_per_second",
    "latency_p50_nanos", "latency_p95_nanos", "latency_p99_nanos", "latency_sample_count",
    "peak_sampled_rss_kb", "cpu_seconds", "disk_usage_bytes",
]
with (root / "screen-summary.csv").open("w", newline="") as summary_file:
    writer = csv.DictWriter(summary_file, fieldnames=fields)
    writer.writeheader()
    for row in case_results:
        writer.writerow({field: row.get(field) for field in fields})
(root / "prefix-fingerprints.json").write_text(json.dumps({"keys": len(prefix_fingerprints), "passed": True}, indent=2) + "\n")
(root / "completion.json").write_text(json.dumps({
    "source_sha": source_sha,
    "binary_sha256": binary_sha,
    "run_count": len(case_results),
    "completed": True,
    "paired_prefix_fingerprint_groups": len(prefix_fingerprints),
    "batch_duration_seconds": time.monotonic() - batch_started,
    "memory_exceed_axis": "deferred by user",
    "performance_claim": "one 5-second screening repetition per case; diagnostic comparison only",
}, indent=2) + "\n")
print(json.dumps({"complete": True, "run_count": len(case_results), "source_sha": source_sha, "binary_sha256": binary_sha, "paired_prefix_groups": len(prefix_fingerprints)}, flush=True))
