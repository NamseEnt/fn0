import bisect
import json
import re
import statistics
import subprocess
import sys
from pathlib import Path


EVENT = re.compile(r"^\s*(\d+)\s+(\d+)\.(\d+):\s+syscalls:sys_(enter|exit)_(pwrite64|fdatasync|fsync):\s*(.*)$")


def union_nanos(intervals):
    total = 0
    previous_end = None
    for started, ended in sorted(intervals):
        total += max(0, ended - max(started, previous_end if previous_end is not None else started))
        previous_end = max(ended, previous_end if previous_end is not None else ended)
    return total


def decode(case):
    trace = case / "syscalls.txt"
    if not trace.exists():
        with trace.open("x") as output, (case / "syscalls.decode.stderr.log").open("x") as errors:
            subprocess.run(["sudo", "-n", "perf", "script", "--ns", "-i", str(case / "syscalls.perf.data"),
                            "-F", "trace:tid,time,event,trace"], stdout=output, stderr=errors, check=True)
    assert "lost" not in (case / "syscalls.decode.stderr.log").read_text().lower()
    assert "lost" not in (case / "syscalls.perf.stderr.log").read_text().lower()
    return trace


def analyze_case(case, offline=False):
    report = json.loads((case / "workload-report.raw.json").read_text())
    snapshots = json.loads((case / "descriptor-snapshots.json").read_text())
    offset = int(statistics.median(sample["unix_nanos"] - sample["monotonic_nanos"] for sample in snapshots))
    offset_range = max(sample["unix_nanos"] - sample["monotonic_nanos"] for sample in snapshots) - min(sample["unix_nanos"] - sample["monotonic_nanos"] for sample in snapshots)
    assert offset_range < 1_000_000
    times = [sample["monotonic_nanos"] for sample in snapshots]
    start = int(report["measurement_window_unix_nanos"]["start"]) - offset
    end = int(report["measurement_window_unix_nanos"]["last_client_completion"]) - offset
    pending = {}
    calls = []
    unpaired_inside = []
    first_time = None
    last_time = None
    trace_path = case / "syscalls.txt" if offline else decode(case)
    with trace_path.open() as trace:
        for line in trace:
            match = EVENT.match(line)
            assert match is not None, line
            thread, seconds, fraction, action, name, fields = match.groups()
            timestamp = int(seconds) * 1_000_000_000 + int(fraction.ljust(9, "0"))
            first_time = timestamp if first_time is None else min(first_time, timestamp)
            last_time = timestamp if last_time is None else max(last_time, timestamp)
            key = (thread, name)
            if action == "enter":
                assert key not in pending
                descriptor = str(int(re.search(r"fd:\s*(0x[0-9a-fA-F]+|\d+)", fields).group(1), 0))
                snapshot_index = bisect.bisect_right(times, timestamp) - 1
                assert snapshot_index >= 0
                path = snapshots[snapshot_index]["descriptors"].get(descriptor)
                count = int(re.search(r"count:\s*(0x[0-9a-fA-F]+|\d+)", fields).group(1), 0) if name == "pwrite64" else None
                pending[key] = {"start": timestamp, "syscall": name, "path": path, "requested_bytes": count}
            else:
                call = pending.pop(key, None)
                if call is None:
                    if start <= timestamp <= end:
                        unpaired_inside.append(line)
                    continue
                call["end"] = timestamp
                call["return"] = int(fields, 0)
                if start <= call["start"] and timestamp <= end:
                    assert call["path"] is not None, call
                    assert call["return"] == (call["requested_bytes"] if name == "pwrite64" else 0), call
                    call["category"] = "wal" if call["path"].endswith(".wal") else "database" if call["path"].endswith(".db") else "other"
                    calls.append(call)
    assert first_time < start < end < last_time
    assert not unpaired_inside
    assert not [call for call in pending.values() if start <= call["start"] <= end]
    assert calls and all(call["category"] != "other" for call in calls)
    metrics = report["blink_checkpoint_metrics"]
    events = [(int(event["started_unix_nanos"]) - offset, int(event["ended_unix_nanos"]) - offset)
              for event in metrics["checkpoint_events"] if event["succeeded"]
              and start <= int(event["started_unix_nanos"]) - offset
              and int(event["ended_unix_nanos"]) - offset <= end]
    summary = {}
    checkpoint_intervals = {}
    for call in calls:
        name = f"{call['category']}_{call['syscall']}"
        entry = summary.setdefault(name, {"count": 0, "bytes": 0, "syscall_seconds_sum": 0})
        entry["count"] += 1
        entry["bytes"] += call["return"] if call["syscall"] == "pwrite64" else 0
        entry["syscall_seconds_sum"] += (call["end"] - call["start"]) / 1_000_000_000
        for checkpoint_start, checkpoint_end in events:
            clipped_start = max(checkpoint_start, call["start"])
            clipped_end = min(checkpoint_end, call["end"])
            if clipped_end > clipped_start:
                checkpoint_intervals.setdefault(name, []).append((clipped_start, clipped_end))
    covered = union_nanos([interval for intervals in checkpoint_intervals.values() for interval in intervals])
    checkpoint_wall = sum(ended - started for started, ended in events)
    appended = metrics["wal_bytes_at_measurement_end"] - metrics["wal_bytes_at_measurement_start"] + sum(event["reclaimed_bytes"] + event["wal_bytes_after"] for event in metrics["checkpoint_events"])
    traced_wal = summary["wal_pwrite64"]["bytes"]
    verified = traced_wal == appended
    if not offline and (case / "cpu.perf.data").exists():
        with (case / "cpu.report.txt").open("w") as output, (case / "cpu.report.stderr.log").open("w") as errors:
            subprocess.run(["perf", "report", "--stdio", "--no-children", "--sort", "symbol", "--percent-limit", "0.5",
                            "--time", f"{start / 1_000_000_000:.9f},{end / 1_000_000_000:.9f}",
                            "-i", str(case / "cpu.perf.data")], stdout=output, stderr=errors, check=True)
    result = {"case": case.name, "diagnostic_only": True, "measurement_seconds": (end - start) / 1_000_000_000,
              "clock_offset_nanos": offset, "clock_offset_spread_nanos": offset_range,
              "trace_spans_measurement_window": True, "unpaired_measured_syscalls": 0, "syscalls": summary,
              "wal_append_conservation_verified": verified, "attribution_accepted": verified,
              "reported_wal_appended_bytes": appended, "traced_wal_bytes": traced_wal,
              "wal_capture_fraction": traced_wal / appended,
              "wal_bytes_per_transaction": traced_wal / metrics["successful_mutation_transactions"],
              "checkpoint_count": len(events), "checkpoint_wall_seconds": checkpoint_wall / 1_000_000_000,
              "checkpoint_syscall_union_seconds": covered / 1_000_000_000,
              "checkpoint_unattributed_wall_seconds": (checkpoint_wall - covered) / 1_000_000_000,
              "checkpoint_syscall_seconds_by_category": {name: union_nanos(intervals) / 1_000_000_000 for name, intervals in checkpoint_intervals.items()}}
    (case / "syscall-summary.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def run(root, offline=False):
    assert (root / "matrix-complete.txt").exists()
    cases = [analyze_case(path, offline) for path in sorted((root / "reports").iterdir()) if path.is_dir()]
    assert len(cases) == json.loads((root / "matrix-plan.json").read_text())["cells"]
    (root / "profile-analysis.json").write_text(json.dumps({"diagnostic_only": True, "all_attributions_accepted": all(case["attribution_accepted"] for case in cases), "cases": cases}, indent=2) + "\n")
    print(json.dumps(cases, indent=2))


if __name__ == "__main__":
    run(Path(sys.argv[1]), "--offline" in sys.argv[2:])
