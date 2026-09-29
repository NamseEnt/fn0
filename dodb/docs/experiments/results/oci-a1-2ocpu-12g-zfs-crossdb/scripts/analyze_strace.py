import json
import re
import sys

trace_path, probe_json_path, data_dir, label = sys.argv[1:5]

SYNC_CALLS = ("fsync", "fdatasync", "msync", "sync_file_range")
line_pattern = re.compile(r"^(?P<pid>\d+)\s+(?P<time>\d\d:\d\d:\d\d\.\d+)\s+(?P<body>.*)$")
call_pattern = re.compile(r"^(?P<name>[a-z_0-9]+)\((?P<args>.*)$")
resumed_pattern = re.compile(r"^<\.\.\. (?P<name>[a-z_0-9]+) resumed>(?P<rest>.*)$")
return_pattern = re.compile(r"=\s*(?P<ret>-?\d+)")
marker_pattern = re.compile(r"\.strace-marker-(?P<marker>[a-z0-9-]+)")

events = []
pending = {}
with open(trace_path, encoding="utf-8", errors="replace") as trace_file:
    for raw_line in trace_file:
        match = line_pattern.match(raw_line.rstrip("\n"))
        if not match:
            continue
        pid = match.group("pid")
        timestamp = match.group("time")
        body = match.group("body")
        resumed = resumed_pattern.match(body)
        if resumed:
            name = resumed.group("name")
            original = pending.pop((pid, name), "")
            full = original + resumed.group("rest")
        else:
            call = call_pattern.match(body)
            if not call:
                continue
            name = call.group("name")
            if body.endswith("<unfinished ...>"):
                pending[(pid, name)] = body[: -len("<unfinished ...>")]
                continue
            full = body
        if name in SYNC_CALLS:
            returned = return_pattern.search(full.rsplit(")", 1)[-1] if ")" in full else full)
            path_match = re.search(r"<([^>]*)>", full)
            events.append({
                "kind": "sync",
                "pid": pid,
                "time": timestamp,
                "call": name,
                "path": path_match.group(1) if path_match else None,
                "ret": int(returned.group("ret")) if returned else None,
                "line": f"{pid} {timestamp} {full}",
            })
        elif name in ("faccessat", "access", "faccessat2"):
            marker = marker_pattern.search(full)
            if marker:
                events.append({
                    "kind": "marker",
                    "pid": pid,
                    "time": timestamp,
                    "marker": marker.group("marker"),
                    "line": f"{pid} {timestamp} {full}",
                })

probe = None
with open(probe_json_path, encoding="utf-8") as probe_file:
    for probe_line in probe_file:
        record = json.loads(probe_line)
        if record.get("record_type") == "durability-probe":
            probe = record

transactions = probe["transactions"] if probe else []
results = []
for transaction in transactions:
    index = transaction["transaction"]
    begin = next((position for position, event in enumerate(events)
                  if event["kind"] == "marker" and event["marker"] == f"commit-begin-{index}"), None)
    end = next((position for position, event in enumerate(events)
                if event["kind"] == "marker" and event["marker"] == f"commit-returned-{index}"), None)
    syncs = []
    if begin is not None and end is not None:
        syncs = [event for event in events[begin + 1:end] if event["kind"] == "sync"]
    durable_syncs = [event for event in syncs
                     if event["ret"] == 0 and event["call"] in ("fsync", "fdatasync")
                     and event["path"] and event["path"].startswith(data_dir)]
    results.append({
        "transaction": index,
        "width": transaction["width"],
        "committed": transaction["committed"],
        "markers_found": begin is not None and end is not None,
        "sync_calls_between_begin_and_return": len(syncs),
        "successful_sync_calls_on_database_files": len(durable_syncs),
        "sync_paths": sorted({event["path"] for event in durable_syncs}),
        "sync_call_kinds": sorted({event["call"] for event in durable_syncs}),
        "verified": bool(transaction["committed"] and durable_syncs),
    })

all_verified = bool(results) and all(result["verified"] for result in results)
reopen_passed = bool(probe and probe.get("passed"))
verdict = "VERIFIED" if all_verified and reopen_passed else "DURABILITY CONTRACT NOT VERIFIED"
print(f"# {label} durability syscall probe")
print(f"verdict: {verdict}")
print(f"successful commits with a successful fsync/fdatasync on a database file under {data_dir}, "
      f"between the commit-begin marker and the commit-returned marker: "
      f"{sum(result['verified'] for result in results)}/{len(results)}")
print(f"close/reopen value verification passed: {reopen_passed} "
      f"(verified {probe.get('verified_values') if probe else None} of {probe.get('expected_values') if probe else None}, "
      f"rows {probe.get('row_count') if probe else None})")
print("")
print("## Per-transaction result")
for result in results:
    print(json.dumps(result, sort_keys=True))
print("")
print("## Effective settings reported by the probe")
print(json.dumps(probe.get("settings") if probe else None, sort_keys=True))
print("")
print("## Marker and sync events in order")
for event in events:
    print(event["line"])
