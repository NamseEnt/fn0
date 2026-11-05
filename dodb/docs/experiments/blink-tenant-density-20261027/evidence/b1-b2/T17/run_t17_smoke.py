import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


SOURCE_COMMIT = "bfb735f04fdf78cbf35bdcc33324ab1efbc90b5b"
HARNESS_SHA256 = "4bf6cfa5e20d62e5a5a8734ee19b9b175790407597a49c8dbe79464d304f2595"
WORKER_SHA256 = "b37513b81e6d46eec4c8bed0e2bfec6ab74037f106b01e5d309268601b150e1b"
NATIVE_SHA256 = "9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5"
GIB = 1024**3


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as input_file:
        for chunk in iter(lambda: input_file.read(1024**2), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def allocated_bytes(path):
    total = 0
    for root, directories, files in os.walk(path, followlinks=False):
        directories[:] = [name for name in directories if not (Path(root) / name).is_symlink()]
        for filename in files:
            file_path = Path(root) / filename
            if file_path.is_symlink():
                raise RuntimeError(f"symlink in smoke database: {file_path}")
            total += file_path.stat().st_blocks * 512
    return total


def remove_cgroup(path):
    for child in sorted(path.iterdir(), key=lambda entry: len(entry.parts), reverse=True):
        if child.is_dir():
            if (child / "cgroup.procs").read_text().strip():
                raise RuntimeError(f"cgroup still has processes: {child}")
            child.rmdir()
    if (path / "cgroup.procs").read_text().strip():
        raise RuntimeError(f"smoke cgroup still has processes: {path}")
    path.rmdir()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--artifact-dir", type=Path, required=True)
    parser.add_argument("--zfs-path", type=Path, default=Path("/bench/zfs/db"))
    parser.add_argument("--run-dir", type=Path, required=True)
    parser.add_argument("--cgroup-path", type=Path, required=True)
    args = parser.parse_args()
    if os.geteuid() != 0:
        raise RuntimeError("T17 smoke requires root for page-cache and cgroup controls")
    if args.run_dir.exists() or args.cgroup_path.exists():
        raise RuntimeError("T17 smoke output or cgroup path already exists")
    harness = args.artifact_dir / "tenant_density_workload"
    worker = args.artifact_dir / "memory_diagnostic_worker"
    if sha256(harness) != HARNESS_SHA256 or sha256(worker) != WORKER_SHA256:
        raise RuntimeError("T17 smoke binaries do not match the preserved artifact manifest")
    native_path = Path("/bench/zfs/db/product-build/rocksdb-source/librocksdb.a")
    if sha256(native_path) != NATIVE_SHA256:
        raise RuntimeError("pinned RocksDB archive hash changed")
    args.run_dir.mkdir(parents=True)
    data_path = args.zfs_path / "databases" / f"t17-smoke-{args.run_dir.name}"
    if data_path.exists():
        raise RuntimeError(f"smoke database path already exists: {data_path}")
    data_path.mkdir(parents=True)
    args.cgroup_path.mkdir()
    (args.cgroup_path / "cgroup.subtree_control").write_text("+memory")
    arc_max_path = Path("/sys/module/zfs/parameters/zfs_arc_max")
    original_arc_max = arc_max_path.read_text().strip()
    report_path = args.run_dir / "report.json"
    log_path = args.run_dir / "process.log"
    command = [
        str(harness),
        "--backend", "blink",
        "--data-dir", str(data_path),
        "--report", str(report_path),
        "--seed", "20261027",
        "--registered-tenants", "1",
        "--active-tenants", "1",
        "--opened-idle-tenants", "0",
        "--client-count", "8",
        "--duration-ms", "300000",
        "--warmup-duration-ms", "300000",
        "--recovery-duration-ms", "300000",
        "--slo-window-ms", "300000",
        "--long-reader-retention-ms", "300000",
        "--drop-page-cache-before-warmup", "on",
        "--mutation-key-mode", "fixed",
        "--mutation-key-epoch", "0",
        "--initial-rows", "256",
        "--value-bytes", "512",
        "--topology", "fixed-per-tenant",
        "--tenant-arrival-phase", "synchronized",
        "--per-tenant-rate", "250",
        "--queue-bound", "8",
        "--max-in-flight", "8",
        "--service-memory-admission-bytes", str(6 * GIB),
        "--request-timeout-ms", "5000",
        "--drain-timeout-ms", "15000",
        "--profile", "read-heavy",
        "--width", "mixed",
        "--backend-cgroup-root", str(args.cgroup_path),
        "--backend-memory-max-bytes", str(8 * GIB),
        "--backend-memory-high-bytes", str(7 * GIB),
        "--pressure-sample-interval-ms", "5000",
        "--blink-wal-layout", "segmented-bases-coalesced",
        "--blink-checkpoint-execution", "background",
        "--blink-wal-segment-bytes", "8192",
        "--blink-checkpoint-wal-bytes", str(1024**2),
        "--blink-demand-paging", "on",
        "--blink-paged-cache-bytes", str(64 * 1024**2),
        "--blink-memory-limit-bytes", str(6 * GIB),
        "--blink-memory-per-tenant-bytes", str(512 * 1024**2),
        "--blink-disk-limit-bytes", str(32 * GIB),
        "--blink-disk-per-tenant-bytes", str(2 * GIB),
    ]
    write_json(
        args.run_dir / "command.json",
        {
            "source_commit": SOURCE_COMMIT,
            "harness_sha256": HARNESS_SHA256,
            "worker_sha256": WORKER_SHA256,
            "native_archive_sha256": NATIVE_SHA256,
            "scope": "five-minute cache, measurement, and reopen-recovery smoke; not a T17 comparison cell",
            "command": command,
        },
    )
    arc_max_path.write_text(str(2 * GIB))
    environment = os.environ.copy()
    environment["DODB_SOURCE_REVISION"] = SOURCE_COMMIT
    started = time.time()
    try:
        with log_path.open("x") as log_file:
            completed = subprocess.run(command, env=environment, stdout=log_file, stderr=subprocess.STDOUT, check=False)
        report = json.loads(report_path.read_text()) if report_path.exists() else None
        if completed.returncode != 0 or report is None:
            raise RuntimeError(f"smoke command failed with exit {completed.returncode}")
        workload = report.get("workload", {})
        recovery = workload.get("phase_workloads", {}).get("recovery_after_reopen", {})
        if report.get("status") != "completed":
            raise RuntimeError("smoke report status is not completed")
        if report.get("host_cache_preconditioning", {}).get("status") != "page_cache_dropped_before_fixed_warmup":
            raise RuntimeError("smoke did not record page-cache drop before fixed warm-up")
        if report.get("long_reader_retention", {}).get("passed") is not True:
            raise RuntimeError("smoke long-reader retention gate failed")
        if len(workload.get("slo_windows", [])) != 1 or len(recovery.get("slo_windows", [])) != 1:
            raise RuntimeError("smoke did not produce one fixed five-minute window per phase")
        if not all(window.get("score", {}).get("passed") is True for window in workload["slo_windows"] + recovery["slo_windows"]):
            raise RuntimeError("smoke SLO window failed")
        if report.get("post_measurement_full_population_oracle", {}).get("passed") is not True:
            raise RuntimeError("smoke post-measurement oracle failed")
        if report.get("reopened_full_population_oracle", {}).get("passed") is not True:
            raise RuntimeError("smoke reopen oracle failed")
        pressure = report.get("backend_memory_pressure", {})
        if pressure.get("enabled") is not True or pressure.get("samples_truncated") is not False:
            raise RuntimeError("smoke cgroup pressure evidence is missing or truncated")
        for epoch_name in ["measurement", "recovery"]:
            epoch = pressure.get("epochs", {}).get(epoch_name, {})
            if any(epoch.get("local_event_delta", {}).get(counter, 0) for counter in ["high", "max", "oom", "oom_kill"]):
                raise RuntimeError(f"smoke cgroup event occurred in {epoch_name}")
        database_bytes = allocated_bytes(data_path)
        if database_bytes > 32 * GIB:
            raise RuntimeError("smoke database exceeded the backend disk cap")
        outcome = {
            "status": "passed",
            "started_unix_seconds": started,
            "ended_unix_seconds": time.time(),
            "exit_code": completed.returncode,
            "report_sha256": sha256(report_path),
            "process_log_sha256": sha256(log_path),
            "input_manifest_sha256": sha256(Path(report["pre_execution_inputs_manifest"])),
            "database_allocated_bytes_before_cleanup": database_bytes,
            "zfs_free_bytes_before_cleanup": os.statvfs(args.zfs_path).f_bavail * os.statvfs(args.zfs_path).f_frsize,
            "measurement_slo_windows": len(workload["slo_windows"]),
            "recovery_slo_windows": len(recovery["slo_windows"]),
            "long_reader_retention": report["long_reader_retention"],
            "post_measurement_oracle": report["post_measurement_full_population_oracle"],
            "reopen_oracle": report["reopened_full_population_oracle"],
        }
        write_json(args.run_dir / "outcome.json", outcome)
        shutil.rmtree(data_path)
        outcome["successful_smoke_database_removed_after_evidence"] = True
        write_json(args.run_dir / "outcome.json", outcome)
        return 0
    except Exception as error:
        write_json(
            args.run_dir / "outcome.json",
            {
                "status": "failed",
                "reason": str(error),
                "started_unix_seconds": started,
                "ended_unix_seconds": time.time(),
                "database_preserved": data_path.exists(),
                "database_allocated_bytes": allocated_bytes(data_path) if data_path.exists() else None,
                "process_log_sha256": sha256(log_path) if log_path.exists() else None,
                "report_sha256": sha256(report_path) if report_path.exists() else None,
            },
        )
        raise
    finally:
        arc_max_path.write_text(original_arc_max)
        remove_cgroup(args.cgroup_path)


if __name__ == "__main__":
    raise SystemExit(main())
