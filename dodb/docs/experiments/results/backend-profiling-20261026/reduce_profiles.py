import argparse
import collections
import gzip
import json
import math
import re
import subprocess
from pathlib import Path


parser = argparse.ArgumentParser()
parser.add_argument("--root", type=Path, required=True)
parser.add_argument("--flamegraph", type=Path, required=True)
parser.add_argument("--stage", choices=["preflight", "main"], default="main")
options = parser.parse_args()
root = options.root.resolve()
stage_root = root / options.stage
assert (stage_root / "complete.txt").exists()
tool_revision = subprocess.check_output(["git", "-C", str(options.flamegraph), "rev-parse", "HEAD"], text=True).strip()
assert tool_revision == "41fee1f99f9276008b7cd112fca19dc3ea84ac32"
patterns = {
    "page_encoding": r"encode_blink_page",
    "page_decoding": r"decode_blink_page",
    "tree_invariants": r"check_invariants",
    "immutable_base_loading": r"load_base_images|durable_bases::read_pack",
    "checkpoint_worker": r"BlinkCheckpointSnapshot>::run|BasePackJob>::run",
    "checkpoint_prepare_complete": r"prepare_segmented_checkpoint|complete_segmented_checkpoint",
    "blink_materialization": r"materializ",
    "blink_transaction_group": r"apply_transaction_group",
    "crc": r"crc32|crc32c|CRC32|Crc32",
    "allocator": r"^alloc::alloc::(alloc|alloc_zeroed|realloc|dealloc)(::|\s|$)|^(_int_malloc|_int_free|malloc|cfree|free|realloc|malloc_consolidate|unlink_chunk|sysmalloc|tcache[^ :]*|__libc_malloc|__libc_free)(\b|@)",
    "memory_copy": r"memcpy|memmove|memset|copy_nonoverlapping",
    "bpf_execution": r"bpf_prog_|bpf_get_|bpf_trace_|trace_call_bpf|__bpf_|htab_map_|stack_map_get_stackid",
    "futex": r"futex",
    "rocks_write": r"rocksdb::DBImpl::Write|rocksdb::WriteThread|rocksdb::PessimisticTransaction::Commit",
    "rocks_lock_or_get_for_update": r"rocksdb::.*(TryLock|LockBatch|LockTracker|PointLock|GetForUpdate)",
    "rocks_get_for_update": r"rocksdb::.*GetForUpdate",
    "rocks_try_lock": r"rocksdb::PessimisticTransaction::TryLock",
    "rocks_db_get": r"rocksdb::DBImpl::GetImpl",
    "rocks_memtable": r"rocksdb::.*(MemTable|SkipList|InsertInto)",
    "rocks_compaction": r"rocksdb::.*(CompactionJob|FlushJob|BackgroundCompaction|BackgroundFlush)",
    "driver_work": r"service_workload::execute_workflow|unconditional_write_workload::make|service_workload::make",
    "driver_validation": r"service_workload::validate|unconditional_write_workload::validate",
    "driver_payload": r"unconditional_write_workload::payload|service_workload::payload",
    "driver_mutation_construction": r"unconditional_write_workload::mutation_for_slot|service_workload::execute_workflow",
}
compiled_patterns = {name: re.compile(pattern, re.MULTILINE) for name, pattern in patterns.items()}
owner_pattern = re.compile(r"^<?(dodb_storage|dodb_server|dodb_rocksdb|dodb_core|rocksdb|service_workload|unconditional_write_workload)::")
header_pattern = re.compile(r"^(.*?)\s+(\d+)/(\d+)\s+(\d+)\.(\d+):\s+(\d+)\s+cpu-clock:\s*$")
frame_pattern = re.compile(r"^\s+([0-9a-f]+)\s+(.+)\s+\(([^()]*)\)\s*$")


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def add_values(target, value):
    for field in ["count", "total_ns", "errors", "bytes"]:
        target[field] += value[field]
    target["max_ns"] = max(target["max_ns"], value["max_ns"])


def new_values():
    return {"count": 0, "total_ns": 0, "max_ns": 0, "errors": 0, "bytes": 0}


def flamegraph(folded_path, title, count_name="samples", colors="hot"):
    svg_path = folded_path.with_suffix(".svg")
    with svg_path.open("wb") as output:
        subprocess.run(["perl", str(options.flamegraph / "flamegraph.pl"), "--title", title,
                        "--countname", count_name, "--colors", colors, "--width", "1600",
                        "--minwidth", "0.2", str(folded_path)], stdout=output, check=True)


def reduce_cpu(case_root, report, target_pid):
    output_root = case_root / "cpu"
    output_root.mkdir(exist_ok=True)
    script_path = output_root / "cpu.perf-script.txt.gz"
    if not script_path.exists():
        with (output_root / "perf-script.stderr.log").open("wb") as stderr:
            perf = subprocess.Popen(["sudo", "-n", "perf", "script", "-f", "--ns", "-F",
                                     "comm,pid,tid,time,event,period,ip,sym,dso", "-i", str(case_root / "cpu.data")],
                                    stdout=subprocess.PIPE, stderr=stderr)
            demangler = subprocess.Popen([str(Path.home() / ".cargo/bin/rustfilt")], stdin=perf.stdout, stdout=subprocess.PIPE)
            perf.stdout.close()
            with gzip.open(script_path, "wb", compresslevel=6) as destination:
                for chunk in iter(lambda: demangler.stdout.read(1024 * 1024), b""):
                    destination.write(chunk)
            assert demangler.wait() == 0
            assert perf.wait() == 0
    window = report["measurement_window_unix_nanos"]
    window_start = int(window["start"])
    window_end = int(window["last_client_completion"])
    checkpoints = [(int(event["started_unix_nanos"]), int(event["ended_unix_nanos"]))
                   for event in report.get("blink_checkpoint_metrics", {}).get("checkpoint_events", []) if event["succeeded"]]
    phase_names = ["measurement", "measurement-checkpoint-overlap", "measurement-outside-checkpoints",
                   "before-measurement", "after-measurement"]
    phases = {name: {"folded": collections.Counter(), "leaf": collections.Counter(), "inclusive": collections.Counter(),
                     "patterns": collections.Counter(), "allocator_owners": collections.Counter(), "samples": 0, "period_ns": 0,
                     "unknown_leaf_samples": 0, "frame_depths": [], "first_ns": None, "last_ns": None}
              for name in phase_names}
    sample = None
    unparsed_headers = []

    def consume(current):
        if current is None or current["pid"] != target_pid:
            return
        timestamp = current["timestamp_ns"]
        frames = current["frames"] or ["[missing_stack]"]
        if window_start <= timestamp <= window_end:
            checkpoint_overlap = any(start <= timestamp <= end for start, end in checkpoints)
            selections = ["measurement", "measurement-checkpoint-overlap" if checkpoint_overlap
                          else "measurement-outside-checkpoints"]
        else:
            selections = ["before-measurement" if timestamp < window_start else "after-measurement"]
        folded = ";".join([current["comm"], *reversed(frames)]).replace("\n", " ")
        joined = "\n".join(frames)
        matched = [name for name, pattern in compiled_patterns.items() if pattern.search(joined)]
        allocator_owner = "runtime_or_other"
        if "allocator" in matched:
            for frame in frames:
                owner = owner_pattern.match(frame)
                if owner:
                    allocator_owner = owner.group(1)
                    break
        for phase_name in selections:
            phase = phases[phase_name]
            phase["samples"] += 1
            phase["period_ns"] += current["period_ns"]
            phase["first_ns"] = timestamp if phase["first_ns"] is None else min(timestamp, phase["first_ns"])
            phase["last_ns"] = timestamp if phase["last_ns"] is None else max(timestamp, phase["last_ns"])
            phase["folded"][folded] += 1
            phase["leaf"][frames[0]] += 1
            phase["inclusive"].update(set(frames))
            phase["patterns"].update(matched)
            if "allocator" in matched:
                phase["allocator_owners"][allocator_owner] += 1
            phase["frame_depths"].append(len(frames))
            phase["unknown_leaf_samples"] += int("[unknown" in frames[0] or "missing_stack" in frames[0])

    with gzip.open(script_path, "rt", errors="replace") as script:
        for line in script:
            header = header_pattern.match(line.rstrip())
            if header:
                consume(sample)
                comm, process_id, thread_id, seconds, fraction, period = header.groups()
                sample = {"comm": comm, "pid": int(process_id), "tid": int(thread_id),
                          "timestamp_ns": int(seconds) * 10**9 + int(fraction.ljust(9, "0")),
                          "period_ns": int(period), "frames": []}
            elif line.startswith("\t") and sample is not None:
                frame = frame_pattern.match(line.rstrip())
                if frame:
                    address, symbol, dso = frame.groups()
                    if dso == "inlined":
                        symbol += " [inline]"
                    if symbol == "[unknown]":
                        symbol = f"[unknown:{Path(dso).name}]"
                    symbol = re.sub(r"\+0x[0-9a-f]+$", "", symbol).replace(";", ":")
                    if dso == "[kernel.kallsyms]":
                        symbol += " [kernel]"
                    sample["frames"].append(symbol)
            elif line.strip() and not line.startswith("#") and not line.startswith("\t"):
                unparsed_headers.append(line.strip())
    consume(sample)
    assert not unparsed_headers, unparsed_headers[:5]
    assert phases["measurement"]["samples"] > 100
    assert phases["measurement"]["last_ns"] >= window_end - 5 * 10**9
    assert phases["measurement"]["first_ns"] <= window_start + 5 * 10**9
    results = {}
    for phase_name, phase in phases.items():
        if not phase["samples"]:
            continue
        folded_path = output_root / f"{phase_name}.folded"
        folded_path.write_text("".join(f"{stack} {count}\n" for stack, count in sorted(phase["folded"].items())))
        assert sum(phase["folded"].values()) == phase["samples"]
        flamegraph(folded_path, f"{case_root.name}: {phase_name} CPU")
        denominator = phase["samples"]
        top = lambda counts: [{"symbol": symbol, "samples": count, "percent": count * 100 / denominator}
                              for symbol, count in counts.most_common(80)]
        results[phase_name] = {"samples": denominator, "sampled_cpu_seconds": phase["period_ns"] / 1e9,
                               "first_sample_epoch_ns": phase["first_ns"], "last_sample_epoch_ns": phase["last_ns"],
                               "unknown_leaf_percent": phase["unknown_leaf_samples"] * 100 / denominator,
                               "median_frame_depth": sorted(phase["frame_depths"])[denominator // 2],
                               "top_self": top(phase["leaf"]), "top_inclusive": top(phase["inclusive"]),
                               "allocator_nearest_project_caller": {owner: {"samples": count, "percent": count * 100 / denominator}
                                                                    for owner, count in phase["allocator_owners"].most_common()},
                               "inclusive_patterns": {name: {"samples": phase["patterns"][name],
                                                              "percent": phase["patterns"][name] * 100 / denominator}
                                                       for name in patterns}}
    write_json(output_root / "summary.json", results)
    return results


def reduce_waits(case_root, report):
    output_root = case_root / "waits"
    raw = json.loads((output_root / "waits.json").read_text())
    assert raw["map_insert_failures"] == [0, 0]
    offset = raw["realtime_minus_monotonic_ns"]
    offset_change = raw["final_realtime_minus_monotonic_ns"] - offset
    assert abs(offset_change) < 5_000_000
    window = report["measurement_window_unix_nanos"]
    first_second = math.ceil((int(window["start"]) - offset) / 1e9)
    final_second = math.floor((int(window["last_client_completion"]) - offset) / 1e9)
    in_window = lambda row: first_second <= row["second"] < final_second
    calls = collections.defaultdict(new_values)
    call_threads = collections.defaultdict(new_values)
    for row in raw["syscalls"]:
        if in_window(row):
            add_values(calls[row["syscall"]], row)
            add_values(call_threads[f"{row['syscall']}:{raw['thread_names'].get(str(row['thread_id']), 'unknown')}"], row)
    folded = {"offcpu": collections.Counter()}
    stack_totals = {name: collections.Counter() for name in folded}
    stack_failures_ns = 0
    total_ns = 0
    long_bucket_ns = 0
    for row in raw["waits"]:
        if not in_window(row):
            continue
        total_ns += row["total_ns"]
        stack_failures_ns += row["total_ns"] if row["stack_id"] < 0 else 0
        long_bucket_ns += row["total_ns"] if row["max_ns"] >= 10**9 else 0
        kind = "offcpu"
        stack = raw["kernel_stacks"][str(row["stack_id"])].copy()
        comm = raw["thread_names"].get(str(row["thread_id"]), "unknown")
        key = ";".join([comm, *reversed(stack)])
        folded[kind][key] += row["total_ns"]
        for symbol in set(stack):
            stack_totals[kind][symbol] += row["total_ns"]
    for kind, counts in folded.items():
        path = output_root / f"measurement-{kind}.folded"
        path.write_text("".join(f"{stack} {max(1, duration // 1000)}\n" for stack, duration in sorted(counts.items())))
        if counts:
            flamegraph(path, f"{case_root.name}: off-CPU kernel stacks, including scheduling and idle waits", "microseconds", "io")
    result = {"bucket_start_monotonic_second": first_second, "bucket_end_monotonic_second": final_second,
              "included_start_bucket_seconds": final_second - first_second,
              "boundary_policy": "Full start-second buckets; intervals can finish beyond a bucket/window. Long idle parks must not be attributed to request latency.",
              "state_policy": "All switch-out intervals pooled. The raw runnable flag means tracepoint prev_state == 0 and is not a complete runnable/sleep classification on this kernel, which also encodes preemption as R+.",
              "realtime_offset_change_ns": offset_change,
              "stack_capture_failure_ns": stack_failures_ns, "total_offcpu_ns": total_ns,
              "aggregate_ns_in_buckets_with_wait_at_least_one_second": long_bucket_ns,
              "syscalls": dict(calls), "syscalls_by_thread_name": dict(call_threads),
              "offcpu_totals_ns": {kind: sum(counts.values()) for kind, counts in folded.items()},
              "top_inclusive_kernel_stacks": {kind: [{"symbol": symbol, "total_ns": duration}
                                                       for symbol, duration in counts.most_common(60)]
                                              for kind, counts in stack_totals.items()}}
    write_json(output_root / "summary.json", result)
    return result


results = {}
for accepted in sorted(stage_root.glob("*/accepted.json")):
    case_root = accepted.parent
    report = json.loads((case_root / "workload-report.json").read_text())
    result = {"run": json.loads(accepted.read_text()), "timing": report["timing"],
              "reopen_and_validation_ms": report["reopen_and_validation_ms"],
              "checkpoint_metrics": report.get("blink_checkpoint_metrics"),
              "latency": report.get("latency"), "counters": report.get("counters"),
              "per_tenant": report.get("per_tenant"), "process_metrics": report.get("process_metrics")}
    if result["run"]["case_id"].endswith("profile"):
        target_pid = json.loads((case_root / "workload-pid.json").read_text())["pid"]
        result["cpu"] = reduce_cpu(case_root, report, target_pid)
        result["waits"] = reduce_waits(case_root, report)
    results[case_root.name] = result
    print(json.dumps({"reduced": case_root.name}), flush=True)
write_json(stage_root / "profile-summary.json", {"flamegraph_revision": tool_revision,
                                                 "inclusive_patterns_overlap": True, "cases": results})
