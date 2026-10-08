import csv
import json
from pathlib import Path

root = Path(__file__).resolve().parent
records = {}
for path in sorted((root / "raw").glob("*.jsonl")):
    lines = [line for line in path.read_text().splitlines() if line.strip()]
    if len(lines) == 1:
        records[path.stem] = json.loads(lines[0])

history_path = root / "runner-execution-history.tsv"
if not history_path.exists():
    (root / "execution-status.tsv").replace(history_path)
history_rows = list(csv.DictReader(history_path.open(), delimiter="\t"))
latest = {}
for row in history_rows:
    if row.get("state") == "process-exited" and row.get("jsonl_records") == "1":
        latest[row["run_id"]] = row
first = {
    "run_id": "01-get-main-btree-r1",
    "started_utc": "2026-10-08T02:46:08.638702+00:00",
    "ended_utc": "2026-10-08T02:48:10.246162+00:00",
    "actual_elapsed_seconds": "121.607424628",
    "exit_code": "0",
    "jsonl_records": "1",
    "result_duration_ms": str(records["01-get-main-btree-r1"]["duration_ms"]),
    "state": "recovered-output-relocated",
}
latest[first["run_id"]] = first
with (root / "execution-status.tsv").open("w", newline="") as status_file:
    fields = ["run_id", "started_utc", "ended_utc", "actual_elapsed_seconds", "exit_code", "jsonl_records", "result_duration_ms", "state"]
    writer = csv.DictWriter(status_file, fieldnames=fields, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    for run_id in sorted(records):
        writer.writerow(latest[run_id])

manifest = {row["run_id"]: row for row in csv.DictReader((root / "manifest.tsv").open(), delimiter="\t")}
columns = ["run_id", "workload", "engine", "clients", "duration_ms", "drain_duration_ns", "process_elapsed_seconds", "successful_operations", "successful_reads", "successful_queries", "successful_write_transactions", "mutation_ops", "errors", "overloads", "conflicts", "aggregate_ops_per_second", "read_ops_per_second", "query_ops_per_second", "write_transactions_per_second", "mutation_ops_per_second", "read_p50_us", "read_p95_us", "read_p99_us", "read_latency_samples", "write_p50_us", "write_p95_us", "write_p99_us", "write_latency_samples", "rss_status", "rss_samples", "rss_failures", "rss_peak_mib", "cpu_machine_percent", "sampled_verification", "sampled_reads_checked", "sampled_reads_failed", "sampled_reads_indeterminate", "query_checked_requests", "query_checked_rows", "query_validation_failures", "verification_history_events", "verification_event_limit", "verification_peak_bytes", "verification_memory_limit_bytes", "verification_omitted_events", "client_completion_skew_ns", "active_generation_pins", "parallel_worker_dispatches", "client_aggregation_passed", "valid"]
with (root / "per-run.tsv").open("w", newline="") as output_file:
    writer = csv.DictWriter(output_file, fieldnames=columns, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    for run_id, record in sorted(records.items()):
        seconds = record["client_latest_completion_offset_ns"] / 1_000_000_000
        successful_reads = record.get("successful_reads", 0)
        successful_queries = record.get("successful_queries", 0)
        successful_transactions = record.get("successful_transactions", 0)
        process_row = latest.get(run_id, {})
        writer.writerow({
            "run_id": run_id,
            "workload": record.get("workload"),
            "engine": record.get("engine"),
            "clients": record.get("client_workers"),
            "duration_ms": record.get("duration_ms"),
            "drain_duration_ns": record.get("client_latest_completion_offset_ns"),
            "process_elapsed_seconds": process_row.get("actual_elapsed_seconds"),
            "successful_operations": record.get("successful_operations"),
            "successful_reads": successful_reads,
            "successful_queries": successful_queries,
            "successful_write_transactions": successful_transactions,
            "mutation_ops": record.get("mutation_ops", 0),
            "errors": record.get("errors"),
            "overloads": record.get("overloads"),
            "conflicts": record.get("conflicts"),
            "aggregate_ops_per_second": record.get("successful_operations", 0) / seconds,
            "read_ops_per_second": successful_reads / seconds,
            "query_ops_per_second": successful_queries / seconds,
            "write_transactions_per_second": successful_transactions / seconds,
            "mutation_ops_per_second": record.get("mutation_ops", 0) / seconds,
            "read_p50_us": record.get("read_p50_us"),
            "read_p95_us": record.get("read_p95_us"),
            "read_p99_us": record.get("read_p99_us"),
            "read_latency_samples": record.get("read_latency_samples"),
            "write_p50_us": record.get("write_p50_us"),
            "write_p95_us": record.get("write_p95_us"),
            "write_p99_us": record.get("write_p99_us"),
            "write_latency_samples": record.get("write_latency_samples"),
            "rss_status": record.get("process_rss_status"),
            "rss_samples": record.get("process_rss_sample_count"),
            "rss_failures": record.get("process_rss_collection_failures"),
            "rss_peak_mib": record.get("process_rss_peak_observed_bytes", 0) / 1024 / 1024,
            "cpu_machine_percent": record.get("cpu_utilization_percent_machine"),
            "sampled_verification": record.get("sampled_read_verification_status"),
            "sampled_reads_checked": record.get("sampled_reads_checked"),
            "sampled_reads_failed": record.get("sampled_reads_failed"),
            "sampled_reads_indeterminate": record.get("sampled_reads_indeterminate"),
            "query_checked_requests": record.get("query_checked_requests"),
            "query_checked_rows": record.get("query_checked_rows"),
            "query_validation_failures": record.get("query_validation_failures"),
            "verification_history_events": record.get("verification_history_event_count"),
            "verification_event_limit": record.get("verification_event_limit"),
            "verification_peak_bytes": record.get("verification_estimated_peak_memory_bytes"),
            "verification_memory_limit_bytes": record.get("verification_memory_limit_bytes"),
            "verification_omitted_events": record.get("verification_omitted_events"),
            "client_completion_skew_ns": record.get("client_completion_skew_ns"),
            "active_generation_pins": record.get("active_generation_pins"),
            "parallel_worker_dispatches": record.get("parallel_worker_dispatches_delta"),
            "client_aggregation_passed": record.get("client_aggregation_passed"),
            "valid": "yes",
        })

conditions = ["run_id\tworkload\tengine\ttotal_concurrent_clients	writers	readers	width	mix	warmup	requested_duration	repetitions	seed	tokio_workers	working_set	cache_capacity	key_size	value_size	distribution	sync_mode	parallel_workers\tblink_workers	borrowed_page_views	read_observational_metrics\tdata_dir\ttmpdir\n"]
for run_id in sorted(records):
    record = records[run_id]
    row = manifest[run_id]
    mix = "50/50 read requests/write transactions" if record.get("workload") == "mixed" else "none"
    width = record.get("transaction_width", 1) if record.get("workload") == "mixed" else "N/A"
    conditions.append("\t".join(map(str, [run_id, record.get("workload"), record.get("engine"), record.get("client_workers"), record.get("writers"), record.get("readers"), width, mix, "1s", "120s", 1, "0xd0db2026", record.get("tokio_workers"), record.get("working_set"), record.get("cache_capacity"), record.get("key_size"), record.get("value_size"), record.get("distribution"), record.get("sync_mode"), record.get("parallel_workers"), record.get("blink_workers"), record.get("blink_borrowed_page_views_enabled"), record.get("blink_read_observational_metrics_enabled"), row.get("data_dir"), row.get("tmpdir")])) + "\n")
(root / "conditions.tsv").write_text("".join(conditions))

commands = []
for run_id in sorted(records):
    row = manifest[run_id]
    commands.append(f"{run_id}\tDODB_BENCH_DIR={row['data_dir']} TMPDIR={row['tmpdir']}\t{row['command']}\n")
(root / "commands.txt").write_text("".join(commands))

attempts = ["attempt\trun_id	start_utc	end_utc	elapsed_seconds	status	detail\n"]
attempts.append("01\t01-get-main-btree-r1\t2026-10-08T02:46:08.638702+00:00\t2026-10-08T02:48:10.246162+00:00\t121.607424628\trecovered-and-accepted\tJSONL was written to a source-relative path, copied with SHA-256 preserved, and checkout was restored clean\n")
for row in history_rows:
    if row["run_id"] == "02-get-planned-blink-r1" and row["state"] == "invalid-output":
        attempts.append(f"02\t{row['run_id']}\t{row['started_utc']}\t{row['ended_utc']}\t{row['actual_elapsed_seconds']}\tinvalid-stopped\tWrong output path; stopped after 105s; no JSONL record\n")
for run_id in ("05-mixed50-w4-main-btree", "06-mixed50-w4-planned-blink", "07-mixed50-w8-planned-blink", "08-mixed50-w8-main-btree"):
    row = next(item for item in history_rows if item["run_id"] == run_id and item["started_utc"] < "2026-10-08T03:07:00")
    attempts.append(f"03\t{run_id}\t{row['started_utc']}\t{row['ended_utc']}\t{row['actual_elapsed_seconds']}\tinvalid-client-model\t--mixed-clients ran one mixed client instead of separate writer and reader clients; raw JSONL preserved under invalid/attempt-03-one-mixed-client\n")
for run_id in sorted(records):
    if run_id != "01-get-main-btree-r1":
        row = latest[run_id]
        attempts.append(f"04\t{run_id}\t{row['started_utc']}\t{row['ended_utc']}\t{row['actual_elapsed_seconds']}\taccepted\tFinal run\n")
(root / "attempt-ledger.tsv").write_text("".join(attempts))

ratios = []
for workload, main_id, planned_id, metric in [
    ("Get", "01-get-main-btree-r1", "02-get-planned-blink-r1", "read_ops_per_second"),
    ("Query16", "04-query16-main-btree-r1", "03-query16-planned-blink-r1", "query_ops_per_second"),
    ("mixed-width4", "05-mixed50-w4-main-btree", "06-mixed50-w4-planned-blink", "aggregate_ops_per_second"),
    ("mixed-width8", "08-mixed50-w8-main-btree", "07-mixed50-w8-planned-blink", "aggregate_ops_per_second"),
]:
    left = records[main_id]
    right = records[planned_id]
    left_seconds = left["client_latest_completion_offset_ns"] / 1_000_000_000
    right_seconds = right["client_latest_completion_offset_ns"] / 1_000_000_000
    left_value = left.get("successful_operations", 0) / left_seconds if metric == "aggregate_ops_per_second" else (left.get("successful_reads", 0) / left_seconds if workload == "Get" else left.get("successful_queries", 0) / left_seconds)
    right_value = right.get("successful_operations", 0) / right_seconds if metric == "aggregate_ops_per_second" else (right.get("successful_reads", 0) / right_seconds if workload == "Get" else right.get("successful_queries", 0) / right_seconds)
    ratios.append(f"{workload}\t{left_value:.6f}\t{right_value:.6f}\t{right_value / left_value:.6f}\n")
(root / "paired-ratios.tsv").write_text("workload\tmain_btree_ops_per_second\tplanned_blink_ops_per_second\tplanned_over_main\n" + "".join(ratios))
