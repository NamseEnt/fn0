import json
import csv
from pathlib import Path

root = Path(__file__).resolve().parent
expected = {
    "01-get-main-btree-r1": ("100%-read-get", "main-btree", 0, 1, 1),
    "02-get-planned-blink-r1": ("100%-read-get", "planned-blink", 0, 1, 1),
    "03-query16-planned-blink-r1": ("100%-read-query", "planned-blink", 0, 1, 1),
    "04-query16-main-btree-r1": ("100%-read-query", "main-btree", 0, 1, 1),
    "05-mixed50-w4-main-btree": ("mixed", "main-btree", 1, 1, 2),
    "06-mixed50-w4-planned-blink": ("mixed", "planned-blink", 1, 1, 2),
    "07-mixed50-w8-planned-blink": ("mixed", "planned-blink", 1, 1, 2),
    "08-mixed50-w8-main-btree": ("mixed", "main-btree", 1, 1, 2),
}
errors = []
records = {}
for run_id, condition in expected.items():
    path = root / "raw" / f"{run_id}.jsonl"
    lines = [line for line in path.read_text().splitlines() if line.strip()] if path.exists() else []
    if len(lines) != 1:
        errors.append(f"{run_id}: expected one JSONL record, found {len(lines)}")
        continue
    record = json.loads(lines[0])
    records[run_id] = record
    workload, engine, writers, readers, client_count = condition
    expected_fields = {
        "git_commit": "95954ecaaae94757cc3705bc2aacf8c6bb78915f",
        "engine": engine,
        "workload": workload,
        "seed": 0xD0DB2026,
        "requested_duration_ms": 120000,
        "warmup_ms": 1000,
        "tokio_workers": 2,
        "working_set": 4096,
        "cache_capacity": 256,
        "key_size": 16,
        "value_size": 64,
        "distribution": "uniform",
        "sync_mode": "real",
        "blink_borrowed_page_views_enabled": False,
        "blink_read_observational_metrics_enabled": True,
        "parallel_workers": 0,
        "writers": writers,
        "readers": readers,
        "client_workers": client_count,
    }
    for field, value in expected_fields.items():
        if record.get(field) != value:
            errors.append(f"{run_id}: {field} expected {value!r}, got {record.get(field)!r}")
    if record.get("duration_ms", 0) < 120000:
        errors.append(f"{run_id}: measurement window shorter than 120000ms")
    if record.get("client_latest_completion_offset_ns", 0) < 120_000_000_000:
        errors.append(f"{run_id}: drain-inclusive client completion offset is missing or short")
    for field in ("errors", "overloads", "conflicts"):
        if record.get(field) != 0:
            errors.append(f"{run_id}: {field}={record.get(field)!r}")
    if record.get("client_aggregation_passed") is not True:
        errors.append(f"{run_id}: client aggregation did not pass")
    clients = record.get("clients")
    if not isinstance(clients, list) or len(clients) != client_count:
        errors.append(f"{run_id}: expected {client_count} client records")
    elif sum(client.get("successful_operations", 0) for client in clients) != record.get("successful_operations"):
        errors.append(f"{run_id}: client successful operation sum mismatch")
    elif sum(client.get("attempted_operations", 0) for client in clients) != record.get("attempted_operations"):
        errors.append(f"{run_id}: client attempted operation sum mismatch")
    if record.get("process_rss_status") != "complete":
        errors.append(f"{run_id}: RSS status is not complete")
    if record.get("process_rss_sample_count", 0) <= 0:
        errors.append(f"{run_id}: no positive RSS sample count")
    if record.get("process_rss_collection_failures") != 0:
        errors.append(f"{run_id}: RSS collection failures are nonzero")
    if record.get("process_rss_peak_observed_bytes", 0) <= 0:
        errors.append(f"{run_id}: RSS peak is not positive")
    if "client_completion_skew_ns" not in record:
        errors.append(f"{run_id}: client completion skew is missing")
    if engine == "planned-blink" and record.get("active_generation_pins") != 0:
        errors.append(f"{run_id}: active generation pins remain")
    if workload == "100%-read-get":
        if record.get("sampled_read_verification_status") != "passed":
            errors.append(f"{run_id}: Get sampled verification did not pass")
        if record.get("sampled_reads_checked", 0) <= 0:
            errors.append(f"{run_id}: no Get samples checked")
        for field in ("sampled_reads_failed", "sampled_reads_indeterminate", "verification_omitted_events"):
            if record.get(field) != 0:
                errors.append(f"{run_id}: {field}={record.get(field)!r}")
    if workload == "100%-read-query":
        if record.get("query_checked_requests") != record.get("successful_queries"):
            errors.append(f"{run_id}: Query request verification count mismatch")
        if record.get("query_checked_rows") != record.get("returned_rows"):
            errors.append(f"{run_id}: Query row verification count mismatch")
        if record.get("query_validation_failures") != 0:
            errors.append(f"{run_id}: Query response validation failures")
        if record.get("returned_rows") != record.get("successful_queries", 0) * 16:
            errors.append(f"{run_id}: Query did not return 16 checked rows per request")
        if record.get("query_client_aggregation_passed") is not True:
            errors.append(f"{run_id}: Query client aggregation did not pass")
        if record.get("successful_queries", 0) <= 0:
            errors.append(f"{run_id}: Query completed no requests")
        if record.get("parallel_worker_dispatches_delta") != 0:
            errors.append(f"{run_id}: unexpected parallel worker dispatches")
    if workload == "mixed":
        width = 4 if "w4" in run_id else 8
        if record.get("transaction_width") != width:
            errors.append(f"{run_id}: mixed width mismatch")
        if record.get("requested_read_percent") != 50 or record.get("requested_write_percent") != 50:
            errors.append(f"{run_id}: mixed requested ratio mismatch")
        roles = {client.get("role") for client in clients or []}
        if record.get("client_workers") != 2 or roles != {"writer", "reader"}:
            errors.append(f"{run_id}: mixed run must have one writer and one reader client")
        if record.get("sampled_read_verification_status") != "passed":
            errors.append(f"{run_id}: mixed revision-history verification did not pass")
        for field in ("sampled_reads_indeterminate", "sampled_reads_failed", "sampled_writes_ambiguous", "verification_omitted_events"):
            if record.get(field) != 0:
                errors.append(f"{run_id}: {field}={record.get(field)!r}")
        if record.get("verification_history_event_count", 0) > record.get("verification_event_limit", 0):
            errors.append(f"{run_id}: verification event limit exceeded")
        if record.get("verification_estimated_peak_memory_bytes", 0) > record.get("verification_memory_limit_bytes", 0):
            errors.append(f"{run_id}: verification memory limit exceeded")
        if not 49.0 <= record.get("successful_read_percent", 0) <= 51.0:
            errors.append(f"{run_id}: successful read ratio outside tolerance")
        for field in ("read_latency_samples", "write_latency_samples"):
            if record.get(field, 0) <= 0:
                errors.append(f"{run_id}: {field} is missing or empty")

    if record.get("parallel_worker_dispatches_delta") != 0:
        errors.append(f"{run_id}: parallel worker dispatch count was nonzero")
    if record.get("read_latency_samples", 0) <= 0:
        errors.append(f"{run_id}: read latency sample count is missing")
    if "--window-seconds" in (root / "commands.txt").read_text():
        errors.append("per-window collection was requested")

fingerprint_maps = {}
for run_id in ("03-query16-planned-blink-r1", "04-query16-main-btree-r1"):
    record = records.get(run_id)
    if record is None or not record.get("clients"):
        continue
    fingerprint = record["clients"][0].get("query_input_fingerprint") or {}
    checkpoints = fingerprint.get("checkpoints") or []
    fingerprint_maps[run_id] = {item["request_count"]: item["fingerprint"] for item in checkpoints}
if len(fingerprint_maps) == 2:
    common_counts = sorted(set.intersection(*(set(values) for values in fingerprint_maps.values())))
    mismatches = [count for count in common_counts if len({values[count] for values in fingerprint_maps.values()}) != 1]
    if not common_counts:
        errors.append("Query fingerprint has no shared request-count checkpoint")
    if mismatches:
        errors.append(f"Query fingerprint mismatch at {mismatches}")
    (root / "query-fingerprint-validation.json").write_text(json.dumps({"passed": bool(common_counts) and not mismatches, "common_checkpoint_count": len(common_counts), "first_request_count": common_counts[0] if common_counts else None, "last_request_count": common_counts[-1] if common_counts else None, "mismatches": mismatches}, indent=2) + "\n")
else:
    errors.append("Both Query runs are required for fingerprint comparison")

binary = json.loads((root / "binary-provenance.json").read_text())
events = [json.loads(line) for line in (root / "runner-events.jsonl").read_text().splitlines() if line.strip()]
manifest = {row["run_id"]: row for row in csv.DictReader((root / "manifest.tsv").open(), delimiter="\t")}
for run_id in expected:
    candidates = [event for event in events if event.get("run_id") == run_id and event.get("exit_code") == 0 and event.get("jsonl_records") == 1 and event.get("binary_sha256") == binary["sha256"]]
    if not candidates and run_id == "01-get-main-btree-r1":
        candidates = [event for event in events if event.get("run_id") == run_id and event.get("exit_code") == 0 and event.get("binary_sha256") == binary["sha256"]]
    if not candidates:
        errors.append(f"{run_id}: no successful runner event with the recorded binary hash")
        continue
    event = candidates[-1]
    command = event.get("command", [])
    if not command or command[0] != binary["path"]:
        errors.append(f"{run_id}: runner binary path differs from provenance")
    if "--window-seconds" in command:
        errors.append(f"{run_id}: per-window collection was requested")
    if run_id.startswith(("05-", "06-", "07-", "08-")) and "--mixed-clients" in command:
        errors.append(f"{run_id}: accepted mixed command used a single mixed client")
    env = event.get("environment", {})
    if not env.get("DODB_BENCH_DIR", "").startswith("/bench/zfs/db/") or not env.get("TMPDIR", "").startswith("/bench/zfs/db/"):
        errors.append(f"{run_id}: DB or TMPDIR is outside the confirmed ZFS path")
    row = manifest[run_id]
    if env.get("DODB_BENCH_DIR") != row.get("data_dir") or env.get("TMPDIR") != row.get("tmpdir"):
        errors.append(f"{run_id}: runner environment differs from the manifest")
postflight = (root / "postflight.txt").read_text()
if "95954ecaaae94757cc3705bc2aacf8c6bb78915f" not in postflight or binary["sha256"] not in postflight:
    errors.append("post-run source SHA or binary hash does not match provenance")
if "--window-seconds" in (root / "commands.txt").read_text():
    errors.append("per-window collection was requested")

status = {"accepted_runs": len(records), "expected_runs": len(expected), "passed": not errors, "errors": errors}
(root / "validation.json").write_text(json.dumps(status, indent=2) + "\n")
print(json.dumps(status, sort_keys=True))
if errors:
    raise SystemExit(1)
