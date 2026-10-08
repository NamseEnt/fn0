import csv
import json
import math
import pathlib
import statistics

result_directory = pathlib.Path(__file__).resolve().parent / "results"
raw_directory = result_directory / "raw"
source_by_variant = {
    "main-btree": "4bc4d42e4d816f4a11428f5423d89a4c35494f9b",
    "blink-metrics-on": "4bc4d42e4d816f4a11428f5423d89a4c35494f9b",
    "blink-candidate": "95954ecaaae94757cc3705bc2aacf8c6bb78915f",
}
variant_binary = {
    "main-btree": "1de9bab43902c81770afa6fa466ecc19e4307d7c71444eb05232e51df17553c3",
    "blink-metrics-on": "1de9bab43902c81770afa6fa466ecc19e4307d7c71444eb05232e51df17553c3",
    "blink-candidate": "89c6e622b3f05c597a740353b30464bbc30cbe3e910195bfc545967a9dca4c87",
}
variants = ("main-btree", "blink-metrics-on", "blink-candidate")
variant_labels = {"main-btree": "A", "blink-metrics-on": "B", "blink-candidate": "C"}
records = {}
mask = (1 << 64) - 1
splitmix_increment = 0x9E3779B97F4A7C15
query_range_count = 136
query_range_count_rejection_threshold = ((-query_range_count) & mask) % query_range_count
query_starts_per_pk = 17
query_limit = 16


def splitmix64(state):
    value = (state + splitmix_increment) & mask
    value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & mask
    value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & mask
    return value ^ (value >> 31)


def trace_cursor_counts(seed, request_count):
    generator_state = seed ^ 0x20000000
    skipped_same_pk_entries = 0
    cursor_request_count = 0
    for _ in range(request_count):
        while True:
            generator_state = splitmix64(generator_state)
            if generator_state >= query_range_count_rejection_threshold:
                break
        range_index = generator_state % query_range_count
        start_index = range_index % query_starts_per_pk
        skipped_same_pk_entries += start_index
        cursor_request_count += start_index > 0
    return skipped_same_pk_entries, cursor_request_count


for workload in ("get", "query"):
    for repetition in range(1, 4):
        for variant in variants:
            cell_id = f"readers-1-{workload}-{variant}-rep{repetition}"
            path = raw_directory / f"{cell_id}.jsonl"
            rows = [json.loads(line) for line in path.read_text().splitlines() if line]
            if len(rows) != 1:
                raise ValueError(f"expected one raw result for {cell_id}")
            record = rows[0]
            if record.get("git_commit") != source_by_variant[variant] or record.get("source_dirty") is not False:
                raise ValueError(f"source provenance mismatch for {cell_id}")
            if record.get("binary_sha256") != variant_binary[variant]:
                raise ValueError(f"binary hash mismatch for {cell_id}")
            if record.get("engine") != ("main-btree" if variant == "main-btree" else "parallel-blink"):
                raise ValueError(f"engine mismatch for {cell_id}")
            if record.get("sync_mode") != "real" or record.get("tokio_workers") != 2 or record.get("readers") != 1:
                raise ValueError(f"execution configuration mismatch for {cell_id}")
            if record.get("working_set") != 4096 or record.get("cache_capacity") != 256 or record.get("key_size") != 16 or record.get("value_size") != 64:
                raise ValueError(f"workload configuration mismatch for {cell_id}")
            if record.get("requested_duration_ms") != 5000 or record.get("warmup_ms") != 1000 or record.get("read_limit") != 16:
                raise ValueError(f"duration or Query limit mismatch for {cell_id}")
            if record.get("seed") != 0xD0DB2025 + repetition:
                raise ValueError(f"seed mismatch for {cell_id}")
            if record.get("successful_reads", 0) < 1 or record.get("errors") != 0 or record.get("overloads") != 0 or record.get("conflicts") != 0:
                raise ValueError(f"operation completion gate failed for {cell_id}")
            if record.get("blink_borrowed_page_views_enabled") is not False:
                raise ValueError(f"borrowed-page option changed for {cell_id}")
            if variant != "main-btree":
                if record.get("blink_read_observational_metrics_enabled") is not True or record.get("read_operations_metric") != record.get("successful_reads"):
                    raise ValueError(f"Blink read observations missing for {cell_id}")
                if record.get("active_generation_pins") != 0:
                    raise ValueError(f"generation pin leak for {cell_id}")
            if record.get("process_rss_status") != "complete" or record.get("process_rss_sample_count", 0) < 1:
                raise ValueError(f"measured RSS window incomplete for {cell_id}")
            if workload == "get" and record.get("sampled_read_verification_status") != "passed":
                raise ValueError(f"Get value or revision verification failed for {cell_id}")
            if workload == "query":
                if record.get("query_checked_requests") != record.get("successful_reads") or record.get("query_validation_failures") != 0:
                    raise ValueError(f"Query response validation failed for {cell_id}")
                if record.get("query_client_aggregation_passed") is not True or record.get("client_aggregation_passed") is not True:
                    raise ValueError(f"Query client aggregation failed for {cell_id}")
            elapsed_ns = record["process_rss_window_end_offset_ns"] - record["process_rss_window_start_offset_ns"]
            if elapsed_ns <= 0:
                raise ValueError(f"invalid measured elapsed time for {cell_id}")
            record["recomputed_ops_per_second"] = record["successful_reads"] * 1_000_000_000 / elapsed_ns
            record["measured_elapsed_ns"] = elapsed_ns
            if workload == "query":
                record["skipped_same_pk_entries"] , record["cursor_request_count"] = trace_cursor_counts(record["seed"], record["successful_reads"])
                record["removed_post_cursor_comparison_clones"] = record["cursor_request_count"] * (query_limit + 1)
                record["removed_prior_entry_cursor_clones"] = record["skipped_same_pk_entries"]
            records[(workload, repetition, variant)] = record

fingerprint_checks = []
for repetition in range(1, 4):
    checkpoint_maps = {}
    for variant in variants:
        record = records[("query", repetition, variant)]
        client_records = record.get("clients")
        if not isinstance(client_records, list) or len(client_records) != 1:
            raise ValueError(f"Query client record missing for rep{repetition} {variant}")
        fingerprint = client_records[0].get("query_input_fingerprint")
        checkpoints = fingerprint.get("checkpoints") if isinstance(fingerprint, dict) else None
        if not isinstance(checkpoints, list):
            raise ValueError(f"Query input checkpoints missing for rep{repetition} {variant}")
        checkpoint_maps[variant] = {checkpoint["request_count"]: checkpoint["fingerprint"] for checkpoint in checkpoints}
    common_counts = set.intersection(*(set(checkpoint_maps[variant]) for variant in variants))
    for request_count in sorted(common_counts):
        values = {variant: checkpoint_maps[variant][request_count] for variant in variants}
        if len(set(values.values())) != 1:
            raise ValueError(f"Query input fingerprint mismatch for rep{repetition} at {request_count}")
        fingerprint_checks.append({"repetition": repetition, "request_count": request_count, "fingerprints": values})

per_run_path = result_directory / "per-run.tsv"
with per_run_path.open("w", newline="") as stream:
    columns = ["workload", "repetition", "seed", "variant", "source_sha", "binary_sha256", "successful_reads", "elapsed_ns", "ops_per_second", "p50_us", "p95_us", "p99_us", "cpu_one_core_percent", "rss_peak_observed_mib", "rss_sample_count", "rss_external_hwm_mib", "input_fingerprint_checks"]
    writer = csv.DictWriter(stream, fieldnames=columns, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    for workload in ("get", "query"):
        for repetition in range(1, 4):
            for variant in variants:
                record = records[(workload, repetition, variant)]
                writer.writerow({"workload": workload, "repetition": repetition, "seed": hex(record["seed"]), "variant": variant_labels[variant], "source_sha": source_by_variant[variant], "binary_sha256": record["binary_sha256"], "successful_reads": record["successful_reads"], "elapsed_ns": record["measured_elapsed_ns"], "ops_per_second": f"{record['recomputed_ops_per_second']:.3f}", "p50_us": record["read_p50_us"], "p95_us": record["read_p95_us"], "p99_us": record["read_p99_us"], "cpu_one_core_percent": record["cpu_utilization_percent_one_core"], "rss_peak_observed_mib": f"{record['process_rss_peak_observed_bytes'] / (1024 * 1024):.3f}", "rss_sample_count": record["process_rss_sample_count"], "rss_external_hwm_mib": f"{record['rss_kib']['hwm'] / 1024:.3f}", "input_fingerprint_checks": len(fingerprint_checks) if workload == "query" else "not-applicable"})

ratio_path = result_directory / "paired-ratios.tsv"
with ratio_path.open("w", newline="") as stream:
    columns = ["workload", "repetition", "seed", "C/B", "B/A", "C/A"]
    writer = csv.DictWriter(stream, fieldnames=columns, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    for workload in ("get", "query"):
        for repetition in range(1, 4):
            rates = {variant: records[(workload, repetition, variant)]["recomputed_ops_per_second"] for variant in variants}
            writer.writerow({"workload": workload, "repetition": repetition, "seed": hex(records[(workload, repetition, "main-btree")]["seed"]), "C/B": f"{rates['blink-candidate'] / rates['blink-metrics-on']:.4f}x", "B/A": f"{rates['blink-metrics-on'] / rates['main-btree']:.4f}x", "C/A": f"{rates['blink-candidate'] / rates['main-btree']:.4f}x"})

summary_path = result_directory / "condition-summary.tsv"
with summary_path.open("w", newline="") as stream:
    columns = ["workload", "variant", "ops_per_second_median", "ops_per_second_range", "p50_us_median", "p50_us_range", "p95_us_median", "p95_us_range", "p99_us_median", "p99_us_range", "cpu_one_core_percent_median", "rss_peak_observed_mib_median", "rss_peak_observed_mib_range"]
    writer = csv.DictWriter(stream, fieldnames=columns, delimiter="\t", lineterminator="\n")
    writer.writeheader()
    for workload in ("get", "query"):
        for variant in variants:
            rows = [records[(workload, repetition, variant)] for repetition in range(1, 4)]
            rates = [row["recomputed_ops_per_second"] for row in rows]
            rss_values = [row["process_rss_peak_observed_bytes"] / (1024 * 1024) for row in rows]
            writer.writerow({"workload": workload, "variant": variant_labels[variant], "ops_per_second_median": f"{statistics.median(rates):.3f}", "ops_per_second_range": f"{min(rates):.3f}-{max(rates):.3f}", "p50_us_median": f"{statistics.median(row['read_p50_us'] for row in rows):.3f}", "p50_us_range": f"{min(row['read_p50_us'] for row in rows):.3f}-{max(row['read_p50_us'] for row in rows):.3f}", "p95_us_median": f"{statistics.median(row['read_p95_us'] for row in rows):.3f}", "p95_us_range": f"{min(row['read_p95_us'] for row in rows):.3f}-{max(row['read_p95_us'] for row in rows):.3f}", "p99_us_median": f"{statistics.median(row['read_p99_us'] for row in rows):.3f}", "p99_us_range": f"{min(row['read_p99_us'] for row in rows):.3f}-{max(row['read_p99_us'] for row in rows):.3f}", "cpu_one_core_percent_median": f"{statistics.median(row['cpu_utilization_percent_one_core'] for row in rows):.3f}", "rss_peak_observed_mib_median": f"{statistics.median(rss_values):.3f}", "rss_peak_observed_mib_range": f"{min(rss_values):.3f}-{max(rss_values):.3f}"})

cost_rows = []
for repetition in range(1, 4):
    for variant in ("blink-metrics-on", "blink-candidate"):
        row = records[("query", repetition, variant)]
        cost_rows.append({"repetition": repetition, "variant": variant_labels[variant], "successful_queries": row["successful_reads"], "cursor_queries": row["cursor_request_count"], "cursor_query_percent": row["cursor_request_count"] * 100 / row["successful_reads"], "same_pk_pre_cursor_entries": row["skipped_same_pk_entries"], "same_pk_pre_cursor_entries_per_query": row["skipped_same_pk_entries"] / row["successful_reads"], "baseline_decodes_avoided_lower_bound": row["skipped_same_pk_entries"], "baseline_documentkey_clones_avoided_lower_bound": row["removed_prior_entry_cursor_clones"] + row["removed_post_cursor_comparison_clones"], "post_cursor_comparison_clones_avoided": row["removed_post_cursor_comparison_clones"]})
(result_directory / "query-cursor-cost-diagnostic.json").write_text(json.dumps({"method": "reconstruct the deterministic measured QueryRange stream from the benchmark seed and request count; each QueryRange start index is the exact count of same-primary-key entries at or before its exclusive cursor", "query_range_count": query_range_count, "starts_per_pk": query_starts_per_pk, "limit": query_limit, "same_primary_key_pre_cursor_entries_per_query_expected": 8, "rows": cost_rows, "interpretation": "Counts are the same-PK portion. Entries earlier in the starting leaf from neighboring PKs, if present, add more baseline decode and cursor-clone work. The candidate validates skipped encoded keys without allocation, preserving malformed-key detection."}, indent=2) + "\n")
(result_directory / "query-fingerprint-validation.json").write_text(json.dumps({"passed": True, "common_checkpoint_count": len(fingerprint_checks), "checks": fingerprint_checks}, indent=2) + "\n")
print("paired ratios")
print(ratio_path.read_text())
print("condition summary")
print(summary_path.read_text())
print("cursor cost diagnostic")
print((result_directory / "query-cursor-cost-diagnostic.json").read_text())
