import argparse
import hashlib
import json
import platform
import shlex
import subprocess
import time
from pathlib import Path


PROFILES = [
    ("serialized-8-on", "rocksdb", "serialized", "shared", 8, True),
    ("ranges-8-on", "rocksdb", "reserved-ranges", "shared", 8, True),
    ("ranges-16-on", "rocksdb", "reserved-ranges", "shared", 16, True),
    ("ranges-split-on", "rocksdb", "reserved-ranges", "split-read-write", 8, True),
    ("ranges-8-off", "rocksdb", "reserved-ranges", "shared", 8, False),
    ("blink-off", "blink", "serialized", "shared", 8, False),
]
SCENARIOS = ["rw95", "rw80", "rw50", "rw20", "mt-mix"]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def checked_output(command):
    return subprocess.check_output(command, text=True).strip()


def verify_report(report, metrics_enabled):
    for phase in ["validation", "reopen_validation"]:
        assert report[phase] and all(result["passed"] for result in report[phase].values()), phase
    metrics = report["rocksdb_research_metrics"]
    assert metrics["enabled"] is metrics_enabled
    for tenant_name, tenant in report["per_tenant"].items():
        counters = tenant["counters"]
        assert counters["other_errors"] == counters["overloads"] == 0
        assert not counters["request_limit_reached"]
        if not metrics_enabled:
            continue
        measured = metrics["tenants"][tenant_name]
        stages = measured["read_stages"]["measurement"]
        for kind, count in [
            ("get", counters["ordinary_gets"]),
            ("preparatory_get", counters["preparatory_gets"]),
            ("query16", counters["query16_calls"]),
        ]:
            assert stages[kind]["service_request_total"]["count"] == count, (tenant_name, kind)
        roles = measured["write_reads"]["measurement"]
        key_count = sum(role["keys"] for role in roles.values())
        assert key_count == measured["measurement"]["user_key_get_for_update_duration"]["count"]
        for role in roles.values():
            assert all(timing["count"] == role["keys"] for timing in role["timings"].values())
            assert role["encoded_bytes_fetched"] == role["payload_bytes_fetched"] + 10 * role["found_keys"]
        outcomes = measured["outcomes"]["measurement"]
        assert outcomes["user_commits"] == counters["successful_write_transactions"]
        assert outcomes["successful_writes"] == counters["successful_write_transactions"]
        assert outcomes["condition_rejections"] == counters["intentional_condition_rejections"] + counters["natural_conflicts"]
        if report["configuration"]["rocksdb"]["revision_allocator"] == "reserved-ranges":
            assert outcomes["user_transaction_counter_accesses"] == 0
        else:
            assert outcomes["user_transaction_counter_accesses"] == outcomes["user_commits"]


def run_case(binary, result_root, profile, scenario, repetition, sequence, smoke=False):
    profile_name, backend, allocator, admission, slots, enabled = profile
    run_name = f"{sequence:03d}-{scenario}-{profile_name}-rep{repetition}"
    run_path = result_root / ("smoke" if smoke else "runs") / run_name
    run_path.mkdir(parents=True, exist_ok=False)
    command = [
        str(binary), "--backend", backend, "--scenario", scenario,
        "--seed", "20261013", "--clients", "4" if smoke else "16",
        "--rows", "10000", "--value-size", "512", "--call-cap", "1000000",
        "--warmup-ms", "50" if smoke else "1000",
        "--measurement-ms", "100" if smoke else "5000",
        "--revision-allocator", allocator, "--blocking-admission", admission,
        "--research-metrics", str(int(enabled)), "--read-stage-metrics", str(int(enabled)),
        "--detailed-write-metrics", str(int(enabled)),
        "--data-dir", str(run_path / "db"), "--report", str(run_path / "report.json"),
    ]
    if admission == "shared":
        command.extend(["--shared-blocking-slots", str(slots)])
    (run_path / "command.txt").write_text(shlex.join(command) + "\n")
    before_hash = digest(binary)
    started = time.monotonic()
    with (run_path / "stdout.txt").open("w") as stdout:
        completed = subprocess.run(command, stdout=stdout, stderr=subprocess.STDOUT)
    (run_path / "exit-status.txt").write_text(str(completed.returncode) + "\n")
    provenance = {
        "profile": profile_name, "scenario": scenario, "repetition": repetition,
        "sequence": sequence, "elapsed_seconds": time.monotonic() - started,
        "binary_sha256_before": before_hash, "binary_sha256_after": digest(binary),
    }
    (run_path / "provenance.json").write_text(json.dumps(provenance, indent=2) + "\n")
    assert completed.returncode == 0, run_name
    assert provenance["binary_sha256_after"] == before_hash, run_name
    verify_report(json.loads((run_path / "report.json").read_text()), enabled)
    (run_path / "checked.json").write_text(json.dumps({"passed": True}) + "\n")
    print(json.dumps({"run": run_name, "smoke": smoke, "passed": True}), flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--result-root", required=True, type=Path)
    parser.add_argument("--source-sha", required=True)
    args = parser.parse_args()
    assert platform.machine() == "aarch64"
    assert checked_output(["hostname"]) == "instance-20260923-1013"
    assert checked_output(["git", "rev-parse", "HEAD"]) == args.source_sha
    assert not checked_output(["git", "status", "--porcelain"])
    assert checked_output(["findmnt", "-n", "-o", "FSTYPE", "-T", str(args.result_root.parent)]) == "zfs"
    args.result_root.mkdir(exist_ok=False)
    plan = {"source_sha": args.source_sha, "profiles": PROFILES, "scenarios": SCENARIOS,
            "repetitions": 3, "performance_runs": 90, "smoke_runs": 6,
            "binary_sha256": digest(args.binary)}
    (args.result_root / "plan.json").write_text(json.dumps(plan, indent=2) + "\n")
    commands = [
        ["date", "-u"], ["hostname"], ["uname", "-a"], ["lscpu"], ["free", "-h"],
        ["df", "-h", "/bench/zfs/db"], ["findmnt", "-T", "/bench/zfs/db"],
        ["rustup", "run", "1.97.1", "rustc", "-vV"],
        ["git", "status", "--short", "--branch"],
        ["git", "-C", "/bench/zfs/db/product-build/rocksdb-source", "rev-parse", "HEAD"],
        ["sha256sum", "/bench/zfs/db/product-build/rocksdb-source/librocksdb.a"],
    ]
    with (args.result_root / "host-and-source.txt").open("w") as output:
        for command in commands:
            output.write(shlex.join(command) + "\n")
            output.flush()
            subprocess.run(command, stdout=output, stderr=subprocess.STDOUT, check=True)
    for profile_index, profile in enumerate(PROFILES):
        run_case(args.binary, args.result_root, profile, "smoke", 0, profile_index + 1, True)
    sequence = 0
    for repetition in range(1, 4):
        for scenario_index, scenario in enumerate(SCENARIOS):
            offset = (scenario_index + 2 * (repetition - 1)) % len(PROFILES)
            order = PROFILES[offset:] + PROFILES[:offset]
            if repetition == 2:
                order = list(reversed(order))
            for profile in order:
                sequence += 1
                run_case(args.binary, args.result_root, profile, scenario, repetition, sequence)
    assert sequence == plan["performance_runs"]
    assert not checked_output(["git", "status", "--porcelain"])
    (args.result_root / "complete.json").write_text(json.dumps({"passed": True, "runs": sequence}) + "\n")


if __name__ == "__main__":
    main()
