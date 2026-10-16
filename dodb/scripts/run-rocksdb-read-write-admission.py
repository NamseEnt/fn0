import argparse
import importlib.util
import json
import platform
import shlex
import subprocess
from pathlib import Path


module_path = Path(__file__).with_name("run-rocksdb-wrapper-attribution.py")
module_spec = importlib.util.spec_from_file_location("wrapper_attribution", module_path)
attribution = importlib.util.module_from_spec(module_spec)
module_spec.loader.exec_module(attribution)
PROFILES = [
    ("ranges-shared8-off", "rocksdb", "reserved-ranges", "shared", 8, False),
    ("ranges-shared10-off", "rocksdb", "reserved-ranges", "shared", 10, False),
    ("ranges-split2-8-off", "rocksdb", "reserved-ranges", "split-read-2-write-8", 10, False),
]
DIAGNOSTIC_PROFILES = [(*profile[:5], True) for profile in PROFILES]
DIAGNOSTIC_PROFILES = [(profile[0].replace("-off", "-on"), *profile[1:]) for profile in DIAGNOSTIC_PROFILES]
SCENARIOS = attribution.SCENARIOS


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--result-root", required=True, type=Path)
    parser.add_argument("--source-sha", required=True)
    args = parser.parse_args()
    checked_output = attribution.checked_output
    assert platform.machine() == "aarch64"
    assert checked_output(["hostname"]) == "instance-20260923-1013"
    assert checked_output(["git", "rev-parse", "HEAD"]) == args.source_sha
    assert not checked_output(["git", "status", "--porcelain"])
    assert checked_output(["findmnt", "-n", "-o", "FSTYPE", "-T", str(args.result_root.parent)]) == "zfs"
    args.result_root.mkdir(exist_ok=False)
    plan = {
        "source_sha": args.source_sha, "profiles": PROFILES,
        "diagnostic_profiles": DIAGNOSTIC_PROFILES, "scenarios": SCENARIOS,
        "repetitions": 3, "diagnostic_repetitions": 1,
        "performance_runs": 45, "diagnostic_runs": 15, "smoke_runs": 6,
        "binary_sha256": attribution.digest(args.binary),
        "decision_screen": {
            "reference": "ranges-shared8-off",
            "minimum_write_throughput_ratio": 0.95,
            "maximum_write_p99_ratio": 1.10,
            "read_p95_and_p99_must_improve": True,
            "scope": "each tenant/profile group; descriptive medians of three runs, not statistical noninferiority",
            "default_promotion": False,
        },
    }
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
    for profile_index, profile in enumerate(PROFILES + DIAGNOSTIC_PROFILES):
        attribution.run_case(args.binary, args.result_root, profile, "smoke", 0, profile_index + 1, True)
    sequence = 0
    for repetition in range(1, 4):
        for scenario_index, scenario in enumerate(SCENARIOS):
            offset = (scenario_index + repetition - 1) % len(PROFILES)
            order = PROFILES[offset:] + PROFILES[:offset]
            if repetition == 2:
                order = list(reversed(order))
            for profile in order:
                sequence += 1
                attribution.run_case(args.binary, args.result_root, profile, scenario, repetition, sequence)
    assert sequence == plan["performance_runs"]
    for scenario_index, scenario in enumerate(SCENARIOS):
        offset = scenario_index % len(DIAGNOSTIC_PROFILES)
        for profile in DIAGNOSTIC_PROFILES[offset:] + DIAGNOSTIC_PROFILES[:offset]:
            sequence += 1
            attribution.run_case(args.binary, args.result_root, profile, scenario, 1, sequence)
    assert sequence == plan["performance_runs"] + plan["diagnostic_runs"]
    assert not checked_output(["git", "status", "--porcelain"])
    (args.result_root / "complete.json").write_text(json.dumps({"passed": True, "runs": sequence}) + "\n")


if __name__ == "__main__":
    main()
