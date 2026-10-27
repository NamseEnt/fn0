import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path


EXPERIMENT = Path(os.environ["DODB_QUERYMIX_ROOT"])
STAGE = sys.argv[1]
ROOT = EXPERIMENT / STAGE
SMOKE = STAGE == "preflight"
DURATION_MS = 8_000 if SMOKE else 300_000
ROOT.mkdir(parents=True, exist_ok=True)
for name, target in [("source", EXPERIMENT / "source"), ("bin", EXPERIMENT / "bin")]:
    link_path = ROOT / name
    if link_path.exists() or link_path.is_symlink():
        if not link_path.is_symlink() or link_path.resolve() != target.resolve():
            raise RuntimeError(f"unexpected stage path already exists: {link_path}")
    else:
        link_path.symlink_to(target)
os.environ.update(
    {
        "DODB_CHECKPOINT_RUN_ROOT": str(ROOT),
        "DODB_CHECKPOINT_SMOKE": "1" if SMOKE else "0",
        "DODB_CHECKPOINT_MEASUREMENT_MS": str(DURATION_MS),
        "DODB_CHECKPOINT_WARMUP_MS": "1000" if SMOKE else "10000",
        "DODB_CHECKPOINT_KEEP_DATABASES": "1",
        "PYTHONDONTWRITEBYTECODE": "1",
    }
)
HELPER = EXPERIMENT / "source/dodb/docs/experiments/results/blink-background-checkpoint-20261021/run_matrix.py"
SPEC = importlib.util.spec_from_file_location("querymix_matrix_helper", HELPER)
BASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BASE)
BASE.CHILD_TIMEOUT_SECONDS = DURATION_MS // 1_000 + 240


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def counters_for(report):
    return report["counters"]


def archive_database(case):
    database = BASE.DATABASES / case
    case_root = BASE.REPORTS / case
    manifest = json.loads((case_root / "database-manifest.json").read_text())
    archive_directory = ROOT / "database-archives"
    archive_directory.mkdir(exist_ok=True)
    archive_path = archive_directory / f"{case}.tar.gz"
    temporary_path = archive_path.with_suffix(".incomplete")
    if archive_path.exists() or temporary_path.exists():
        raise RuntimeError(f"database archive path already exists: {archive_path}")
    with tarfile.open(temporary_path, "w:gz", compresslevel=1) as archive:
        archive.add(database, arcname=case)
    actual = {}
    with tarfile.open(temporary_path, "r:gz") as archive:
        for member in archive:
            if member.isdir():
                continue
            if not member.isfile() or not member.name.startswith(case + "/"):
                raise RuntimeError(f"unexpected database archive member: {member.name}")
            digest = hashlib.sha256()
            with archive.extractfile(member) as stream:
                while chunk := stream.read(1024 * 1024):
                    digest.update(chunk)
            actual[member.name[len(case) + 1 :]] = {"size_bytes": member.size, "sha256": digest.hexdigest()}
    expected = {entry["path"]: {"size_bytes": entry["size_bytes"], "sha256": entry["sha256"]} for entry in manifest}
    if actual != expected:
        raise RuntimeError(f"database archive verification failed: {case}")
    os.replace(temporary_path, archive_path)
    with archive_path.open("rb") as archive_file:
        os.fsync(archive_file.fileno())
    metadata = {
        "case": case,
        "path": str(archive_path),
        "sha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(),
        "compressed_bytes": archive_path.stat().st_size,
        "database_bytes": sum(entry["size_bytes"] for entry in manifest),
        "file_count": len(manifest),
        "verified": True,
    }
    write_json(archive_directory / f"{case}.json", metadata)
    shutil.rmtree(database)
    print(json.dumps({"database_preserved": case, **metadata}), flush=True)


def validate_mix(report, case):
    if report.get("scenario") != "querymix":
        raise RuntimeError(f"wrong workload profile: {case}")
    counters = counters_for(report)
    if counters.get("other_errors") or counters.get("overloads") or counters.get("natural_conflicts"):
        raise RuntimeError(f"unexpected request errors: {case}")
    if counters.get("preparatory_gets") != 0 or counters.get("intentional_condition_rejections") != 0:
        raise RuntimeError(f"querymix contains unexpected preparatory reads or rejected conditions: {case}")
    categories = counters["write_categories"]
    attempts = {
        "query16": counters["query16_calls"],
        "get": counters["ordinary_gets"],
        "put": categories["put"]["attempts"],
        "conditional_put": categories["conditional_width_1"]["attempts"],
    }
    total = sum(attempts.values())
    expected_shares = {"query16": 0.40, "get": 0.30, "put": 0.15, "conditional_put": 0.15}
    observed_shares = {name: count / total for name, count in attempts.items()}
    if any(abs(observed_shares[name] - expected) > 0.01 for name, expected in expected_shares.items()):
        raise RuntimeError(f"operation mix drift exceeds 1 percentage point: {case}: {observed_shares}")
    if total != counters["api_calls"] or any(categories[name]["successes"] != categories[name]["attempts"] for name in ["put", "conditional_width_1"]):
        raise RuntimeError(f"operation counts or write successes do not match: {case}")
    if counters["query16_calls"] == 0 or counters["query_rows_returned"] < counters["query16_calls"]:
        raise RuntimeError(f"query returned too few rows: {case}")
    BASE.validate_latency_counts(counters["latency"])
    for name in ["query16", "get", "put_success", "conditional_put_success", "workflow"]:
        if counters["latency"].get(name, {}).get("count", 0) == 0:
            raise RuntimeError(f"latency category is missing: {case}: {name}")
    return attempts, observed_shares


def run_case(backend, repetition):
    case = f"{backend}-r{repetition}"
    database = BASE.DATABASES / case
    report_path = BASE.REPORTS / f"{case}.json"
    command = BASE.service_command(backend, database, report_path, "134217728")
    command[command.index("--scenario") + 1] = "querymix"
    if backend == "blink":
        command.extend(
            [
                "--blink-wal-layout",
                "segmented-bases-coalesced",
                "--blink-checkpoint-execution",
                "background",
                "--blink-wal-segment-bytes",
                "33554432",
            ]
        )
    BASE.wait_for_case(command, case, database, report_path, "A", backend, "134217728")
    report = json.loads(report_path.read_text())
    attempts, shares = validate_mix(report, case)
    signature = BASE.input_prefix_signature(report, "A")
    return case, report, attempts, shares, signature


def main():
    BASE.REPORTS.mkdir(exist_ok=False)
    BASE.DATABASES.mkdir(exist_ok=False)
    binaries = {"service_workload": BASE.sha256_file(BASE.SERVICE_BINARY)}
    source_sha = subprocess.check_output(["git", "-C", str(BASE.SOURCE), "rev-parse", "HEAD"], text=True).strip()
    repetitions = 1 if SMOKE else 3
    plan = {
        "stage": STAGE,
        "duration_ms": DURATION_MS,
        "warmup_ms": BASE.WARMUP_MS,
        "backends": ["blink", "minimal-native"],
        "order_by_repetition": {
            str(repetition): (["blink", "minimal-native"] if repetition % 2 else ["minimal-native", "blink"])
            for repetition in range(1, repetitions + 1)
        },
        "source_sha": source_sha,
        "binary_sha256": binaries,
        "expected_api_shares": {"query16": 0.40, "get": 0.30, "put": 0.15, "conditional_put": 0.15},
        "controller_sha256": BASE.sha256_file(Path(__file__)),
        "helper_sha256": BASE.sha256_file(HELPER),
    }
    write_json(ROOT / "matrix-plan.json", plan)
    completed = []
    prefixes = []
    reference_signature = None
    for repetition, backends in ((rep, plan["order_by_repetition"][str(rep)]) for rep in range(1, repetitions + 1)):
        for backend in backends:
            case, report, attempts, shares, signature = run_case(backend, repetition)
            if reference_signature is None:
                reference_signature = signature
            elif signature != reference_signature:
                raise RuntimeError(f"fixed per-client input prefix mismatch: {case}")
            prefixes.append({"case": case, "fingerprint": signature, "attempts": attempts, "shares": shares})
            archive_database(case)
            completed.append(case)
            write_json(ROOT / "input-prefix-fingerprints.json", prefixes)
            (ROOT / "completed-cases.txt").write_text("\n".join(completed) + "\n")
            print(json.dumps({"completed": case, "throughput": report["throughput_per_second"]["api_calls"], "shares": shares}), flush=True)
    (ROOT / "matrix-complete.txt").write_text(f"{len(completed)} accepted cells\n")


if __name__ == "__main__":
    main()
