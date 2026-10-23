import hashlib
import json
import os
import re
import subprocess
from collections import Counter
from pathlib import Path


ROOT = Path(os.environ["DODB_BASE_RUN_ROOT"])
SOURCE = ROOT / "source"
BUILD_LOG = Path(os.environ["DODB_BASE_TEST_BUILD_LOG"])
EXPECTED_SOURCE = os.environ["DODB_BASE_SOURCE_SHA"]
TARGET = Path("/bench/zfs/db/dodb-rocksdb-shared-cache/linux-aarch64-gnu/rust-1.97.1/release-crc/target/release").resolve()
OUTPUT = ROOT / "validation"
SELECTED = {"dodb_storage", "dodb_server", "dodb_rocksdb", "background_checkpoint", "blink_wal_checkpoint",
            "background_checkpoint_sigkill", "segmented_checkpoint_sigkill", "segmented_wal_sigkill",
            "phase2_recovery", "process_crash_recovery", "phase6_quic", "transaction_contract", "read_contract",
            "contract_history", "service_workload", "unconditional_write_workload", "durable_base_checkpoint",
            "durable_base_sigkill", "durable_base_background_sigkill"}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def main():
    OUTPUT.mkdir(exist_ok=True)
    revision = subprocess.check_output(["git", "-C", str(SOURCE), "rev-parse", "HEAD"], text=True).strip()
    assert revision == EXPECTED_SOURCE
    assert not subprocess.check_output(["git", "-C", str(SOURCE), "status", "--porcelain"], text=True).strip()
    binaries = {}
    for line in BUILD_LOG.read_text().splitlines():
        match = re.search(r"Executable .*\(([^)]+)\)", line)
        if not match:
            continue
        path = Path(match.group(1))
        path = (path if path.is_absolute() else SOURCE / path).resolve()
        name = path.name.rsplit("-", 1)[0]
        if name in SELECTED:
            assert TARGET in path.parents
            binaries[path.name] = (name, path)
    counts = Counter(name for name, path in binaries.values())
    assert set(counts) == SELECTED, SELECTED - set(counts)
    assert counts["durable_base_checkpoint"] == 2
    assert len(binaries) == len(SELECTED) + 1
    environment = os.environ.copy()
    temporary = ROOT / "test-databases"
    temporary.mkdir(exist_ok=True)
    environment["TMPDIR"] = str(temporary)
    environment["DODB_TEST_DATA_DIR"] = str(temporary)
    for name in ["phase2-crash-child", "process-crash-recovery-child"]:
        path = TARGET / name
        assert path.is_file()
        environment[f"CARGO_BIN_EXE_{name}"] = str(path)
    records_path = OUTPUT / "commands.json"
    records = json.loads(records_path.read_text()) if records_path.exists() else []
    for identity, (name, path) in binaries.items():
        previous = [record for record in records if record["identity"] == identity and record.get("exit_code") == 0]
        if previous:
            assert previous[-1]["source_sha"] == revision and previous[-1]["sha256"] == digest(path)
            continue
        attempt = sum(record["identity"] == identity for record in records)
        log_path = OUTPUT / f"{identity}-attempt-{attempt}.log"
        record = {"identity": identity, "name": name, "path": str(path), "sha256": digest(path),
                  "source_sha": revision, "argv": [str(path), "--test-threads=1"], "log": log_path.name}
        records.append(record)
        write_json(records_path, records)
        print(json.dumps({"starting": identity}), flush=True)
        with log_path.open("wb") as stream:
            result = subprocess.run(record["argv"], stdout=stream, stderr=subprocess.STDOUT, cwd=SOURCE, env=environment, timeout=600)
        record["exit_code"] = result.returncode
        summaries = re.findall(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", log_path.read_text(errors="replace"))
        assert summaries, identity
        record["counts"] = dict(zip(["passed", "failed", "ignored"], map(int, summaries[-1])))
        write_json(records_path, records)
        print(json.dumps(record), flush=True)
        if result.returncode:
            raise SystemExit(result.returncode)
    accepted = [next(record for record in reversed(records) if record["identity"] == identity and record.get("exit_code") == 0) for identity in binaries]
    summary = {"source_sha": revision, "binaries": len(accepted), "completed": True,
               "passed": sum(record["counts"]["passed"] for record in accepted),
               "ignored": sum(record["counts"]["ignored"] for record in accepted), "failed": 0}
    write_json(OUTPUT / "validation-complete.json", summary)
    print(json.dumps(summary), flush=True)


if __name__ == "__main__":
    main()
