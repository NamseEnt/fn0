import json
import csv
import argparse
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent)
args = parser.parse_args()
root = args.root.resolve()
errors = []
run_errors = {}
def fail(run_id, message):
    run_errors.setdefault(run_id, []).append(message)

expected = {
    "09-get-main-btree-provenance-clean-r1": ("100%-read-get", "main-btree", 0, 1, 1),
    "02-get-planned-blink-r1": ("100%-read-get", "planned-blink", 0, 1, 1),
    "03-query16-planned-blink-r1": ("100%-read-query", "planned-blink", 0, 1, 1),
    "04-query16-main-btree-r1": ("100%-read-query", "main-btree", 0, 1, 1),
    "05-mixed50-w4-main-btree": ("mixed", "main-btree", 1, 1, 2),
    "06-mixed50-w4-planned-blink": ("mixed", "planned-blink", 1, 1, 2),
    "07-mixed50-w8-planned-blink": ("mixed", "planned-blink", 1, 1, 2),
    "08-mixed50-w8-main-btree": ("mixed", "main-btree", 1, 1, 2),
}
records = {}
binary = json.loads((root / "binary-provenance.json").read_text())
build = json.loads((root / "build-provenance.json").read_text())["configurations"]["candidate"]
expected_source_sha256 = build["phase0_bench_source_sha256"]
expected_cargo_lock_sha256 = build["cargo_lock_sha256"]
if binary.get("sha256") != build.get("binary_sha256"):
    for run_id in expected:
        fail(run_id, "binary provenance hash differs from build evidence")
if binary.get("source_sha") != build.get("source_sha"):
    for run_id in expected:
        fail(run_id, "binary provenance source SHA differs from build evidence")
for run_id, condition in expected.items():
    path = root / "raw" / f"{run_id}.jsonl"
    lines = [line for line in path.read_text().splitlines() if line.strip()] if path.exists() else []
    if len(lines) != 1:
        fail(run_id, f"expected one JSONL record, found {len(lines)}")
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
    provenance_fields = {
        "source_dirty": False,
        "git_commit": "95954ecaaae94757cc3705bc2aacf8c6bb78915f",
        "binary_sha256": binary["sha256"],
        "source_sha256": expected_source_sha256,
        "cargo_lock_sha256": expected_cargo_lock_sha256,
    }
    for field, value in provenance_fields.items():
        if field not in record:
            fail(run_id, f"required provenance field {field} is missing")
        elif field == "source_dirty" and record[field] is not False:
            fail(run_id, "source_dirty must be the boolean false")
        elif record[field] != value:
            fail(run_id, f"{field} expected {value!r}, got {record[field]!r}")
    for field, value in expected_fields.items():
        if record.get(field) != value:
            fail(run_id, f"{field} expected {value!r}, got {record.get(field)!r}")
    if record.get("duration_ms", 0) < 120000:
        fail(run_id, f"measurement window shorter than 120000ms")
    if record.get("client_latest_completion_offset_ns", 0) < 120_000_000_000:
        fail(run_id, f"drain-inclusive client completion offset is missing or short")
    for field in ("errors", "overloads", "conflicts"):
        if record.get(field) != 0:
            fail(run_id, f"{field}={record.get(field)!r}")
    if record.get("client_aggregation_passed") is not True:
        fail(run_id, f"client aggregation did not pass")
    clients = record.get("clients")
    if not isinstance(clients, list) or len(clients) != client_count:
        fail(run_id, f"expected {client_count} client records")
    elif sum(client.get("successful_operations", 0) for client in clients) != record.get("successful_operations"):
        fail(run_id, f"client successful operation sum mismatch")
    elif sum(client.get("attempted_operations", 0) for client in clients) != record.get("attempted_operations"):
        fail(run_id, f"client attempted operation sum mismatch")
    if record.get("process_rss_status") != "complete":
        fail(run_id, f"RSS status is not complete")
    if record.get("process_rss_sample_count", 0) <= 0:
        fail(run_id, f"no positive RSS sample count")
    if record.get("process_rss_collection_failures") != 0:
        fail(run_id, f"RSS collection failures are nonzero")
    if record.get("process_rss_peak_observed_bytes", 0) <= 0:
        fail(run_id, f"RSS peak is not positive")
    if "client_completion_skew_ns" not in record:
        fail(run_id, f"client completion skew is missing")
    if engine == "planned-blink" and record.get("active_generation_pins") != 0:
        fail(run_id, f"active generation pins remain")
    if workload == "100%-read-get":
        if record.get("sampled_read_verification_status") != "passed":
            fail(run_id, f"Get sampled verification did not pass")
        if record.get("sampled_reads_checked", 0) <= 0:
            fail(run_id, f"no Get samples checked")
        for field in ("sampled_reads_failed", "sampled_reads_indeterminate", "verification_omitted_events"):
            if record.get(field) != 0:
                fail(run_id, f"{field}={record.get(field)!r}")
    if workload == "100%-read-query":
        if record.get("query_checked_requests") != record.get("successful_queries"):
            fail(run_id, f"Query request verification count mismatch")
        if record.get("query_checked_rows") != record.get("returned_rows"):
            fail(run_id, f"Query row verification count mismatch")
        if record.get("query_validation_failures") != 0:
            fail(run_id, f"Query response validation failures")
        if record.get("returned_rows") != record.get("successful_queries", 0) * 16:
            fail(run_id, f"Query did not return 16 checked rows per request")
        if record.get("query_client_aggregation_passed") is not True:
            fail(run_id, f"Query client aggregation did not pass")
        if record.get("successful_queries", 0) <= 0:
            fail(run_id, f"Query completed no requests")
        if record.get("parallel_worker_dispatches_delta") != 0:
            fail(run_id, f"unexpected parallel worker dispatches")
    if workload == "mixed":
        width = 4 if "w4" in run_id else 8
        if record.get("transaction_width") != width:
            fail(run_id, f"mixed width mismatch")
        if record.get("requested_read_percent") != 50 or record.get("requested_write_percent") != 50:
            fail(run_id, f"mixed requested ratio mismatch")
        roles = {client.get("role") for client in clients or []}
        if record.get("client_workers") != 2 or roles != {"writer", "reader"}:
            fail(run_id, f"mixed run must have one writer and one reader client")
        if record.get("sampled_read_verification_status") != "passed":
            fail(run_id, f"mixed revision-history verification did not pass")
        for field in ("sampled_reads_indeterminate", "sampled_reads_failed", "sampled_writes_ambiguous", "verification_omitted_events"):
            if record.get(field) != 0:
                fail(run_id, f"{field}={record.get(field)!r}")
        if record.get("verification_history_event_count", 0) > record.get("verification_event_limit", 0):
            fail(run_id, f"verification event limit exceeded")
        if record.get("verification_estimated_peak_memory_bytes", 0) > record.get("verification_memory_limit_bytes", 0):
            fail(run_id, f"verification memory limit exceeded")
        if not 49.0 <= record.get("successful_read_percent", 0) <= 51.0:
            fail(run_id, f"successful read ratio outside tolerance")
        for field in ("read_latency_samples", "write_latency_samples"):
            if record.get(field, 0) <= 0:
                fail(run_id, f"{field} is missing or empty")

    if record.get("parallel_worker_dispatches_delta") != 0:
        fail(run_id, f"parallel worker dispatch count was nonzero")
    if record.get("read_latency_samples", 0) <= 0:
        fail(run_id, f"read latency sample count is missing")
    if "--window-seconds" in (root / "commands.txt").read_text():
        fail(run_id, "per-window collection was requested")

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
        for run_id in ("03-query16-planned-blink-r1", "04-query16-main-btree-r1"):
            fail(run_id, "Query fingerprint has no shared request-count checkpoint")
    if mismatches:
        for run_id in ("03-query16-planned-blink-r1", "04-query16-main-btree-r1"):
            fail(run_id, f"Query fingerprint mismatch at {mismatches}")
    (root / "query-fingerprint-validation.json").write_text(json.dumps({"passed": bool(common_counts) and not mismatches, "common_checkpoint_count": len(common_counts), "first_request_count": common_counts[0] if common_counts else None, "last_request_count": common_counts[-1] if common_counts else None, "mismatches": mismatches}, indent=2) + "\n")
else:
    for run_id in ("03-query16-planned-blink-r1", "04-query16-main-btree-r1"):
        fail(run_id, "Both Query runs are required for fingerprint comparison")

events = [json.loads(line) for line in (root / "runner-events.jsonl").read_text().splitlines() if line.strip()]
manifest = {row["run_id"]: row for row in csv.DictReader((root / "manifest.tsv").open(), delimiter="\t")}
for run_id in expected:
    candidates = [event for event in events if event.get("run_id") == run_id and event.get("exit_code") == 0 and event.get("jsonl_records") == 1 and event.get("binary_sha256") == binary["sha256"]]
    if not candidates:
        fail(run_id, f"no successful runner event with the recorded binary hash")
        continue
    event = candidates[-1]
    command = event.get("command", [])
    if not command or command[0] != binary["path"]:
        fail(run_id, f"runner binary path differs from provenance")
    if event.get("actual_elapsed_seconds", 0) < 120:
        fail(run_id, f"runner elapsed time including drain is shorter than 120 seconds")
    if run_id == "09-get-main-btree-provenance-clean-r1":
        for field in ("source_status_before", "source_status_after"):
            if event.get(field) != "clean" and not (field == "source_status_after" and event.get(field) == ""):
                fail(run_id, f"{field} is not clean")
        for field in ("source_commit_before", "source_commit_after"):
            if event.get(field) != "95954ecaaae94757cc3705bc2aacf8c6bb78915f":
                fail(run_id, f"{field} does not match the measurement SHA")
        for field, expected_hash in (("binary_sha256_before", binary["sha256"]), ("binary_sha256_after", binary["sha256"]), ("source_sha256_before", expected_source_sha256), ("source_sha256_after", expected_source_sha256), ("cargo_lock_sha256_before", expected_cargo_lock_sha256), ("cargo_lock_sha256_after", expected_cargo_lock_sha256)):
            if event.get(field) != expected_hash:
                fail(run_id, f"{field} differs from build evidence")
        output_path = event.get("output_path", "")
        if not output_path.startswith("/bench/zfs/db/") or not output_path.startswith("/"):
            fail(run_id, "runner output path is not an absolute path under confirmed ZFS")
        if str(output_path).startswith(binary["path"].split("/target-")[0] + "/candidate/"):
            fail(run_id, "runner output path is inside the source checkout")
    if "--window-seconds" in command:
        fail(run_id, f"per-window collection was requested")
    if run_id.startswith(("05-", "06-", "07-", "08-")) and "--mixed-clients" in command:
        fail(run_id, f"accepted mixed command used a single mixed client")
    env = event.get("environment", {})
    if not env.get("DODB_BENCH_DIR", "").startswith("/bench/zfs/db/") or not env.get("TMPDIR", "").startswith("/bench/zfs/db/"):
        fail(run_id, f"DB or TMPDIR is outside the confirmed ZFS path")
    row = manifest[run_id]
    if env.get("DODB_BENCH_DIR") != row.get("data_dir") or env.get("TMPDIR") != row.get("tmpdir"):
        fail(run_id, f"runner environment differs from the manifest")
postflight = (root / "postflight.txt").read_text()
if "95954ecaaae94757cc3705bc2aacf8c6bb78915f" not in postflight or binary["sha256"] not in postflight:
    for run_id in expected:
        fail(run_id, "post-run source SHA or binary hash does not match provenance")
if "--window-seconds" in (root / "commands.txt").read_text():
    for run_id in expected:
        fail(run_id, "per-window collection was requested")

accepted_ids = [run_id for run_id in expected if run_id in records and not run_errors.get(run_id)]
rejected = {run_id: messages for run_id, messages in run_errors.items()}
errors.extend(f"{run_id}: {message}" for run_id, messages in run_errors.items() for message in messages)
status = {"accepted_runs": len(accepted_ids), "accepted_run_ids": accepted_ids, "expected_runs": len(expected), "passed": len(accepted_ids) == len(expected) and not errors, "errors": errors, "run_errors": rejected}
(root / "validation.json").write_text(json.dumps(status, indent=2) + "\n")
print(json.dumps(status, sort_keys=True))
if errors:
    raise SystemExit(1)
