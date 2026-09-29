import glob
import json
import math
import os
import statistics

base_dir = os.path.dirname(os.path.dirname(__file__))
raw_dir = os.path.join(base_dir, "raw")
rows = []
for result_path in sorted(glob.glob(os.path.join(raw_dir, "*.jsonl"))):
    if os.path.basename(result_path) == "process-metrics.jsonl":
        continue
    with open(result_path, encoding="utf-8") as result_file:
        for line in result_file:
            if line.strip():
                row = json.loads(line)
                row["result_file"] = os.path.basename(result_path)
                row["comparison_engine"] = "planned-blink" if row["git_commit"] == "0d310da32af5dfaa9df6469788998dacccebb841" else "exact-main"
                rows.append(row)

process_metrics = {}
with open(os.path.join(raw_dir, "process-metrics.jsonl"), encoding="utf-8") as metrics_file:
    for line in metrics_file:
        if line.strip():
            metric = json.loads(line)
            process_metrics[os.path.basename(metric["jsonl"])] = metric

assert len(rows) == 84, f"expected 84 result rows, got {len(rows)}"
assert all(row["sync_mode"] == "real" and row["wal_syncs_delta"] > 0 and row["wal_sync_nanos_total"] > 0 for row in rows)
assert all(row["errors"] == 0 and row["overloads"] == 0 and row["conflicts"] == 0 for row in rows)
assert all(row["successful_transactions"] == row["attempted_transactions"] for row in rows)
assert all(row["mutation_ops"] == row["successful_transactions"] * row["transaction_width"] for row in rows)
assert all(row["max_actual_group_requests"] <= 64 for row in rows)
run_order = []
with open(os.path.join(base_dir, "run-order.txt"), encoding="utf-8") as order_file:
    for line in order_file:
        if line.strip():
            event = json.loads(line)
            if event["event"] == "start":
                run_order.append(event)
assert len(run_order) == 84, f"expected 84 run starts, got {len(run_order)}"
assert all(event["tmpdir"] == "/bench/zfs/db" for event in run_order)
for scenario_index in range(14):
    for repetition in (1, 2, 3):
        pair = [event for event in run_order if event["scenario_index"] == scenario_index and event["repetition"] == repetition]
        expected_order = ["ExactMain", "Planned"] if repetition in (1, 3) else ["Planned", "ExactMain"]
        assert [event["engine"] for event in pair] == expected_order
        assert pair[0]["seed"] == pair[1]["seed"]

for row in rows:
    metric = process_metrics[row["result_file"]]
    row["rss_peak_mib"] = metric["rss_peak_sampled_kib"] / 1024
    row["sync_us"] = row["wal_sync_nanos_total"] / row["wal_syncs_delta"] / 1000
    row["wal_bytes_per_mutation"] = row["wal_bytes_delta"] / max(row["mutation_ops"], 1)
    row["page_images_per_mutation"] = row["page_images_delta"] / max(row["mutation_ops"], 1)

keys = ("writers", "transaction_width", "distribution")
grouped = {}
for row in rows:
    key = tuple(row[name] for name in keys) + (row["comparison_engine"],)
    grouped.setdefault(key, []).append(row)

summaries = []
for key, run_rows in sorted(grouped.items()):
    summary = {name: value for name, value in zip((*keys, "engine"), key)}
    for field in ("logical_tx_per_second", "mutation_ops_per_second", "e2e_p50_us", "e2e_p95_us", "e2e_p99_us", "cpu_utilization_percent_machine", "rss_peak_mib", "avg_group_requests", "max_actual_group_requests", "transactions_per_sync", "sync_us", "wal_bytes_per_mutation", "page_images_per_mutation"):
        summary[field] = statistics.mean(row[field] for row in run_rows)
    summary["successful_transactions_total"] = sum(row["successful_transactions"] for row in run_rows)
    summary["wal_syncs_total"] = sum(row["wal_syncs_delta"] for row in run_rows)
    summary["wal_sync_time_seconds_total"] = sum(row["wal_sync_nanos_total"] for row in run_rows) / 1_000_000_000
    summary["queue_wait_per_request_us"] = sum(row["queue_wait_nanos_total"] for row in run_rows) / max(sum(row["queued_requests"] for row in run_rows), 1) / 1000
    summary["collection_per_group_us"] = sum(row["collection_nanos_total"] for row in run_rows) / max(sum(row["groups"] for row in run_rows), 1) / 1000
    summary["wal_syncs_per_sec"] = sum(row["wal_syncs_delta"] for row in run_rows) / (sum(row["duration_ms"] for row in run_rows) / 1000)
    summary["total_errors"] = sum(row["errors"] for row in run_rows)
    summary["total_overloads"] = sum(row["overloads"] for row in run_rows)
    summary["total_conflicts"] = sum(row["conflicts"] for row in run_rows)
    summary["repetitions"] = len(run_rows)
    summaries.append(summary)

summary_by_key = {(row["writers"], row["transaction_width"], row["distribution"], row["engine"]): row for row in summaries}
ratios = []
for writers in (16, 64):
    for width in (1, 16):
        for distribution in ("uniform", "same-leaf-heavy", "different-leaf-heavy"):
            main_row = summary_by_key[(writers, width, distribution, "exact-main")]
            planned_row = summary_by_key[(writers, width, distribution, "planned-blink")]
            ratios.append({"writers": writers, "width": width, "distribution": distribution, "exact_main_tx_s": main_row["logical_tx_per_second"], "planned_tx_s": planned_row["logical_tx_per_second"], "exact_main_ops_s": main_row["mutation_ops_per_second"], "planned_ops_s": planned_row["mutation_ops_per_second"], "ratio": planned_row["mutation_ops_per_second"] / main_row["mutation_ops_per_second"]})

category_ratios = {}
for category in ("all", "width-1", "width-16", "writers-16", "writers-64", "uniform", "same-leaf-heavy", "different-leaf-heavy"):
    chosen = ratios
    if category == "width-1":
        chosen = [row for row in ratios if row["width"] == 1]
    elif category == "width-16":
        chosen = [row for row in ratios if row["width"] == 16]
    elif category == "writers-16":
        chosen = [row for row in ratios if row["writers"] == 16]
    elif category == "writers-64":
        chosen = [row for row in ratios if row["writers"] == 64]
    elif category in ("uniform", "same-leaf-heavy", "different-leaf-heavy"):
        chosen = [row for row in ratios if row["distribution"] == category]
    category_ratios[category] = math.exp(statistics.mean(math.log(row["ratio"]) for row in chosen))

old_raw = os.path.abspath(os.path.join(base_dir, "../oci-a1-2ocpu-12g-200g/dual-main-vs-planned/raw"))
old_groups = {}
for old_path in glob.glob(os.path.join(old_raw, "*.jsonl")):
    with open(old_path, encoding="utf-8") as old_file:
        for line in old_file:
            if line.strip():
                old_row = json.loads(line)
                if old_row.get("sync_mode") == "disabled" and old_row.get("writers") in (16, 64):
                    engine = "ExactMain" if "ExactMain" in os.path.basename(old_path) else "Planned"
                    old_key = (old_row["writers"], old_row["transaction_width"], old_row["distribution"], engine)
                    old_groups.setdefault(old_key, []).append(old_row["avg_group_requests"])

old_group_comparison = []
for writers in (16, 64):
    for width in (1, 16):
        for distribution in ("uniform", "same-leaf-heavy", "different-leaf-heavy"):
            for engine in ("ExactMain", "Planned"):
                old_key = (writers, width, distribution, engine)
                current_engine = "exact-main" if engine == "ExactMain" else "planned-blink"
                current = summary_by_key[(writers, width, distribution, current_engine)]["avg_group_requests"]
                old_values = old_groups.get(old_key, [])
                old_group_comparison.append({"writers": writers, "width": width, "distribution": distribution, "engine": engine, "sync_disabled_mean_group": statistics.mean(old_values) if old_values else None, "real_sync_mean_group": current})

result = {"validity": {"rows": len(rows), "real_sync_rows": sum(row["sync_mode"] == "real" for row in rows), "errors": sum(row["errors"] for row in rows), "overloads": sum(row["overloads"] for row in rows), "conflicts": sum(row["conflicts"] for row in rows), "transaction_count_mismatches": sum(row["successful_transactions"] != row["attempted_transactions"] for row in rows), "mutation_width_mismatches": sum(row["mutation_ops"] != row["successful_transactions"] * row["transaction_width"] for row in rows), "database_root_mismatches": sum(event["tmpdir"] != "/bench/zfs/db" for event in run_order), "rotated_run_pairs": 42}, "scenarios": summaries, "ratios": ratios, "geometric_means": category_ratios, "sync_disabled_group_comparison": old_group_comparison, "group_distribution_percentiles": "unavailable: current benchmark exports average and maximum realized group size only; no per-group histogram or samples are exposed"}
with open(os.path.join(base_dir, "analysis.json"), "w", encoding="utf-8") as result_file:
    json.dump(result, result_file, indent=2, sort_keys=True)
    result_file.write("\n")

scenario_order = [(writers, width, distribution) for writers in (16, 64) for width in (1, 16) for distribution in ("uniform", "same-leaf-heavy", "different-leaf-heavy")] + [(1, width, "uniform") for width in (1, 16)]
with open(os.path.join(base_dir, "core-matrix.tsv"), "w", encoding="utf-8") as matrix_file:
    matrix_file.write("writers\twidth\tdistribution\tExactMain tx/s\tPlanned tx/s\tExactMain mutation ops/s\tPlanned mutation ops/s\tExactMain p50 us\tPlanned p50 us\tExactMain p95 us\tPlanned p95 us\tExactMain p99 us\tPlanned p99 us\tratio planned/main\n")
    for writers, width, distribution in scenario_order:
        main_row = summary_by_key[(writers, width, distribution, "exact-main")]
        planned_row = summary_by_key[(writers, width, distribution, "planned-blink")]
        ratio = planned_row["mutation_ops_per_second"] / main_row["mutation_ops_per_second"] if writers > 1 else "control"
        matrix_file.write("\t".join(map(str, (writers, width, distribution, round(main_row["logical_tx_per_second"]), round(planned_row["logical_tx_per_second"]), round(main_row["mutation_ops_per_second"]), round(planned_row["mutation_ops_per_second"]), round(main_row["e2e_p50_us"]), round(planned_row["e2e_p50_us"]), round(main_row["e2e_p95_us"]), round(planned_row["e2e_p95_us"]), round(main_row["e2e_p99_us"]), round(planned_row["e2e_p99_us"]), round(ratio, 4) if isinstance(ratio, float) else ratio))) + "\n")

with open(os.path.join(base_dir, "resource-metrics.tsv"), "w", encoding="utf-8") as metrics_file:
    metrics_file.write("writers\twidth\tdistribution\tengine\tCPU percent machine\tRSS peak MiB\tmean group\tmax group\ttx per sync\tmean sync us\tsyncs total\tsync time seconds\tsyncs per second\tqueue wait us/request\tcollection us/group\tWAL bytes/mutation\tpage images/mutation\tsuccessful tx\terrors\toverloads\tconflicts\n")
    for writers, width, distribution in scenario_order:
        for engine in ("exact-main", "planned-blink"):
            summary = summary_by_key[(writers, width, distribution, engine)]
            metrics_file.write("\t".join(map(str, (writers, width, distribution, engine, round(summary["cpu_utilization_percent_machine"], 2), round(summary["rss_peak_mib"], 1), round(summary["avg_group_requests"], 2), int(summary["max_actual_group_requests"]), round(summary["transactions_per_sync"], 2), round(summary["sync_us"], 1), summary["wal_syncs_total"], round(summary["wal_sync_time_seconds_total"], 3), round(summary["wal_syncs_per_sec"], 2), round(summary["queue_wait_per_request_us"], 1), round(summary["collection_per_group_us"], 1), round(summary["wal_bytes_per_mutation"], 1), round(summary["page_images_per_mutation"], 3), summary["successful_transactions_total"], summary["total_errors"], summary["total_overloads"], summary["total_conflicts"]))) + "\n")
print(json.dumps({"validity": result["validity"], "geometric_means": category_ratios, "core_matrix": os.path.join(base_dir, "core-matrix.tsv")}, indent=2))
