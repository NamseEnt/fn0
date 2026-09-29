import glob
import json
import os
import statistics
import sys

RESULTS = sys.argv[1]
RAW = os.path.join(RESULTS, "raw")
SCENARIOS = {
    0: (16, 1, "uniform"),
    3: (16, 16, "uniform"),
    6: (64, 1, "uniform"),
    9: (64, 16, "uniform"),
    10: (64, 16, "same-leaf-heavy"),
    11: (64, 16, "different-leaf-heavy"),
}
LABELS = {"uniform": "uniform", "same-leaf-heavy": "compact", "different-leaf-heavy": "spread"}
COMPONENTS = (
    ("admission", "logical_admission_nanos"),
    ("planning", "planning_nanos"),
    ("physical mutation", "physical_mutation_nanos_total"),
    ("physical restamp", "physical_restamp_nanos_total"),
    ("page encode", "physical_page_encode_nanos_total"),
    ("physical other", None),
    ("dirty union", "dirty_union_nanos_total"),
    ("catalog construction", "catalog_construction_nanos"),
    ("WAL assembly", "wal_assembly_nanos_total"),
    ("WAL redo plan (delta encode)", "wal_redo_plan_nanos_total"),
    ("WAL frame encode", None),
    ("WAL write", "wal_group_write_nanos_total"),
    ("WAL sync", "wal_sync_nanos_total"),
    ("state install", "state_install_nanos_total"),
    ("generation publication", "generation_publication_nanos"),
    ("dirty tracking", "dirty_tracking_nanos_total"),
    ("unattributed", None),
)
DETAIL = (
    ("planner route", "planner_route_nanos"),
    ("leaf load clone (inside physical mutation)", "leaf_load_clone_nanos_total"),
    ("catalog state scan (inside catalog)", "catalog_state_scan_nanos_total"),
    ("catalog chunk clone (inside catalog)", "catalog_chunk_clone_nanos_total"),
    ("catalog map clone (inside catalog)", "catalog_map_clone_nanos_total"),
    ("catalog directory clone (inside catalog)", "catalog_directory_clone_nanos_total"),
    ("retired generation drop (inside publication)", "retired_generation_drop_nanos_total"),
    ("publication swap (inside publication)", "publication_swap_nanos_total"),
    ("WAL append total (plan + encode + write)", "wal_append_nanos_total"),
)


def load(path):
    with open(path, encoding="utf-8") as input_file:
        return json.loads(input_file.read().splitlines()[0])


def histogram(text):
    values = {}
    for item in filter(None, text.split(",")):
        bucket, count = item.split(":")
        values[int(bucket)] = values.get(int(bucket), 0) + int(count)
    return values


def merge_histograms(rows, key):
    merged = {}
    for row in rows:
        for bucket, count in histogram(row[key]).items():
            merged[bucket] = merged.get(bucket, 0) + count
    return merged


def histogram_stats(values):
    total = sum(values.values())
    if total == 0:
        return {"mean": 0, "p50": 0, "p95": 0, "max": 0}
    mean = sum(bucket * count for bucket, count in values.items()) / total

    def percentile(fraction):
        target = fraction * total
        running = 0
        for bucket in sorted(values):
            running += values[bucket]
            if running >= target:
                return bucket
        return max(values)

    return {"mean": mean, "p50": percentile(0.5), "p95": percentile(0.95), "max": max(values)}


def components(row):
    values = {}
    for name, key in COMPONENTS:
        if key:
            values[name] = row[key]
    values["physical other"] = max(0, row["physical_execution_nanos"] - row["physical_mutation_nanos_total"]
                                   - row["physical_restamp_nanos_total"] - row["physical_page_encode_nanos_total"])
    values["WAL frame encode"] = max(0, row["wal_group_encode_nanos_total"] - row["wal_redo_plan_nanos_total"])
    attributed = sum(value for value in values.values())
    values["unattributed"] = max(0, row["processing_nanos_total"] - attributed)
    return values


def summarize(sync_mode, scenario_index):
    writers, width, distribution = SCENARIOS[scenario_index]
    pattern = os.path.join(RAW, f"attr-{sync_mode}-{scenario_index:02d}-rep*-page-delta-w{writers}-width{width}-{distribution}.jsonl")
    paths = sorted(glob.glob(pattern))
    if len(paths) != 3:
        raise SystemExit(f"{pattern}: {len(paths)} files")
    rows = [load(path) for path in paths]
    for path, row in zip(paths, rows):
        if row["engine"] != "planned-blink" or row["sync_mode"] != sync_mode or row["errors"] != 0:
            raise SystemExit(f"{path}: bad row")
    transactions = sum(row["successful_transactions"] for row in rows)
    mutations = sum(row["mutation_ops"] for row in rows)
    component_totals = {name: sum(components(row)[name] for row in rows) for name, _ in COMPONENTS}
    detail_totals = {name: sum(row[key] for row in rows) for name, key in DETAIL}
    processing = sum(row["processing_nanos_total"] for row in rows)
    duration = sum(row["duration_ms"] for row in rows) * 1e6
    deltas = sum(row["wal_page_delta_records_delta"] for row in rows)
    images = sum(row["wal_page_image_records_delta"] for row in rows)
    return {
        "scenario_index": scenario_index,
        "writers": writers,
        "width": width,
        "distribution": distribution,
        "sync_mode": sync_mode,
        "tx_per_second_runs": [row["logical_tx_per_second"] for row in rows],
        "tx_per_second": statistics.mean(row["logical_tx_per_second"] for row in rows),
        "mutation_ops_per_second": statistics.mean(row["mutation_ops_per_second"] for row in rows),
        "p50_us": statistics.mean(row["e2e_p50_us"] for row in rows),
        "p99_us": statistics.mean(row["e2e_p99_us"] for row in rows),
        "cpu_percent_one_core": statistics.mean(row["cpu_utilization_percent_one_core"] for row in rows),
        "coordinator_busy_fraction": processing / duration,
        "groups_per_second": sum(row["groups"] for row in rows) / (duration / 1e9),
        "tx_per_group": transactions / sum(row["groups"] for row in rows),
        "tx_per_sync": transactions / max(sum(row["wal_syncs_delta"] for row in rows), 1),
        "mean_sync_ms": sum(row["wal_sync_nanos_total"] for row in rows) / max(sum(row["wal_syncs_delta"] for row in rows), 1) / 1e6,
        "processing_ns_per_tx": processing / transactions,
        "processing_ns_per_mutation": processing / mutations,
        "component_ns_per_tx": {name: value / transactions for name, value in component_totals.items()},
        "component_ns_per_mutation": {name: value / mutations for name, value in component_totals.items()},
        "component_share": {name: value / processing for name, value in component_totals.items()},
        "detail_ns_per_tx": {name: value / transactions for name, value in detail_totals.items()},
        "mutations_per_tx": mutations / transactions,
        "page_encodes_per_tx": sum(row["leaf_encodes"] for row in rows) / transactions,
        "page_encodes_per_mutation": sum(row["leaf_encodes"] for row in rows) / mutations,
        "page_delta_records_per_tx": deltas / transactions,
        "page_image_records_per_tx": images / transactions,
        "delta_payload_bytes_per_tx": sum(row["wal_page_delta_payload_bytes_delta"] for row in rows) / transactions,
        "delta_payload_bytes_per_mutation": sum(row["wal_page_delta_payload_bytes_delta"] for row in rows) / mutations,
        "delta_spans_per_tx": sum(row["wal_page_delta_spans_delta"] for row in rows) / transactions,
        "delta_spans_per_mutation": sum(row["wal_page_delta_spans_delta"] for row in rows) / mutations,
        "wal_bytes_per_tx": sum(row["wal_bytes_delta"] for row in rows) / transactions,
        "leaf_splits": sum(row["leaf_splits"] for row in rows),
        "structural_transactions": sum(row["structural_transactions_delta"] for row in rows),
        "same_leaf_groups": sum(row["same_leaf_groups"] for row in rows),
        "leaf_groups": sum(row["leaf_groups"] for row in rows),
        "locality": {
            "mutations": histogram_stats(merge_histograms(rows, "tx_mutation_histogram")),
            "dirty_pages": histogram_stats(merge_histograms(rows, "tx_dirty_page_histogram")),
            "leaf_pages": histogram_stats(merge_histograms(rows, "tx_leaf_page_histogram")),
        },
    }


def rocksdb_summary(scenario_index):
    writers, width, _ = SCENARIOS[scenario_index]
    paths = sorted(glob.glob(os.path.join(RAW, f"confirm-rocksdb-{scenario_index:02d}-rep*-w{writers}-width{width}-uniform.jsonl")))
    dodb = sorted(glob.glob(os.path.join(RAW, f"confirm-real-{scenario_index:02d}-rep*-page-delta-w{writers}-width{width}-uniform.jsonl")))
    if len(paths) != 3 or len(dodb) != 3:
        return None
    rows = [load(path) for path in paths]
    dodb_rows = [load(path) for path in dodb]
    return {
        "rocksdb_tx_per_second": statistics.mean(row["measured"]["logical_tx_per_second"] for row in rows),
        "rocksdb_p99_us": statistics.mean(row["measured"]["p99_us"] for row in rows),
        "rocksdb_cpu_percent_one_core": statistics.mean(row["cpu_utilization_percent_one_core"] for row in rows),
        "dodb_tx_per_second": statistics.mean(row["logical_tx_per_second"] for row in dodb_rows),
        "dodb_p99_us": statistics.mean(row["e2e_p99_us"] for row in dodb_rows),
        "dodb_cpu_percent_one_core": statistics.mean(row["cpu_utilization_percent_one_core"] for row in dodb_rows),
    }


def fmt(value, digits=0):
    return f"{value:,.{digits}f}"


def label(summary):
    return f"{summary['writers']}w w{summary['width']} {LABELS[summary['distribution']]}"


def main():
    summaries = {(mode, index): summarize(mode, index) for mode in ("real", "disabled") for index in SCENARIOS}
    lines = ["# WAL v3 Phase C attribution tables\n"]
    lines.append("All rows: planned Blink with PageDelta (`cb7fb54`), 100,000 rows, 2 s warmup, 5 s measure, 3 repetitions. "
                 "`disabled` rows skip fsync and are a CPU control, not durable throughput.\n")
    lines.append("## Throughput\n")
    lines.append("| Scenario | sync | tx/s (runs) | mean tx/s | mutation/s | p50 µs | p99 µs | CPU % (one core) | coordinator busy | tx/group | tx/sync | mean sync ms |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for index in SCENARIOS:
        for mode in ("real", "disabled"):
            summary = summaries[(mode, index)]
            lines.append(
                f"| {label(summary)} | {mode} | " + ", ".join(fmt(value) for value in summary["tx_per_second_runs"])
                + f" | {fmt(summary['tx_per_second'])} | {fmt(summary['mutation_ops_per_second'])} | {fmt(summary['p50_us'])} | "
                f"{fmt(summary['p99_us'])} | {fmt(summary['cpu_percent_one_core'])} | {summary['coordinator_busy_fraction']:.1%} | "
                f"{summary['tx_per_group']:.1f} | {summary['tx_per_sync']:.1f} | {summary['mean_sync_ms']:.2f} |")
    lines.append("")
    for mode in ("real", "disabled"):
        for unit in ("tx", "mutation"):
            lines.append(f"## Coordinator time per {unit}, ns ({mode} sync)\n")
            lines.append("| Component | " + " | ".join(label(summaries[(mode, index)]) for index in SCENARIOS) + " |")
            lines.append("|---|" + "---|" * len(SCENARIOS))
            for name, _ in COMPONENTS:
                lines.append(f"| {name} | " + " | ".join(
                    fmt(summaries[(mode, index)][f"component_ns_per_{unit}"][name]) for index in SCENARIOS) + " |")
            lines.append("| **total (coordinator processing)** | " + " | ".join(
                fmt(summaries[(mode, index)][f"processing_ns_per_{unit}"]) for index in SCENARIOS) + " |")
            lines.append("")
        lines.append(f"## Share of coordinator processing time ({mode} sync)\n")
        lines.append("| Component | " + " | ".join(label(summaries[(mode, index)]) for index in SCENARIOS) + " |")
        lines.append("|---|" + "---|" * len(SCENARIOS))
        for name, _ in COMPONENTS:
            lines.append(f"| {name} | " + " | ".join(
                f"{summaries[(mode, index)]['component_share'][name]:.1%}" for index in SCENARIOS) + " |")
        lines.append("")
        lines.append(f"## Detail timers per tx, ns ({mode} sync)\n")
        lines.append("| Timer | " + " | ".join(label(summaries[(mode, index)]) for index in SCENARIOS) + " |")
        lines.append("|---|" + "---|" * len(SCENARIOS))
        for name, _ in DETAIL:
            lines.append(f"| {name} | " + " | ".join(
                fmt(summaries[(mode, index)]["detail_ns_per_tx"][name]) for index in SCENARIOS) + " |")
        lines.append("")
    lines.append("## Work counts per transaction (real sync)\n")
    lines.append("| Scenario | mutations/tx | page encodes/tx | encodes/mutation | deltas/tx | images/tx | delta payload B/tx | delta B/mutation | spans/tx | spans/mutation | WAL B/tx |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|---|")
    for index in SCENARIOS:
        summary = summaries[("real", index)]
        lines.append(
            f"| {label(summary)} | {summary['mutations_per_tx']:.2f} | {summary['page_encodes_per_tx']:.2f} | "
            f"{summary['page_encodes_per_mutation']:.3f} | {summary['page_delta_records_per_tx']:.3f} | "
            f"{summary['page_image_records_per_tx']:.4f} | {fmt(summary['delta_payload_bytes_per_tx'], 1)} | "
            f"{fmt(summary['delta_payload_bytes_per_mutation'], 1)} | {summary['delta_spans_per_tx']:.2f} | "
            f"{summary['delta_spans_per_mutation']:.2f} | {fmt(summary['wal_bytes_per_tx'], 1)} |")
    lines.append("")
    lines.append("## Page locality per transaction (real sync, all measured transactions)\n")
    lines.append("| Scenario | mutations mean / p50 / p95 | dirty pages mean / p50 / p95 / max | leaves mean / p50 / p95 | same-leaf mutations/tx | structural tx | leaf splits |")
    lines.append("|---|---|---|---|---|---|---|")
    for index in SCENARIOS:
        summary = summaries[("real", index)]
        locality = summary["locality"]
        lines.append(
            f"| {label(summary)} | {locality['mutations']['mean']:.2f} / {locality['mutations']['p50']} / {locality['mutations']['p95']} | "
            f"{locality['dirty_pages']['mean']:.2f} / {locality['dirty_pages']['p50']} / {locality['dirty_pages']['p95']} / {locality['dirty_pages']['max']} | "
            f"{locality['leaf_pages']['mean']:.2f} / {locality['leaf_pages']['p50']} / {locality['leaf_pages']['p95']} | "
            f"{locality['mutations']['mean'] - locality['leaf_pages']['mean']:.2f} | {summary['structural_transactions']} | {summary['leaf_splits']} |")
    lines.append("")
    lines.append("## Width 16 / width 1 scaling of per-transaction time (uniform)\n")
    lines.append("| Component | 16w real | 64w real | 16w no-sync | 64w no-sync |")
    lines.append("|---|---|---|---|---|")
    pairs = (("real", 0, 3), ("real", 6, 9), ("disabled", 0, 3), ("disabled", 6, 9))

    def ratio(mode, narrow, wide, name):
        narrow_value = summaries[(mode, narrow)]["component_ns_per_tx"][name]
        wide_value = summaries[(mode, wide)]["component_ns_per_tx"][name]
        return wide_value / narrow_value if narrow_value > 0 else float("nan")

    for name, _ in COMPONENTS:
        lines.append(f"| {name} | " + " | ".join(f"{ratio(mode, narrow, wide, name):.1f}×" for mode, narrow, wide in pairs) + " |")
    lines.append("| **total** | " + " | ".join(
        f"{summaries[(mode, wide)]['processing_ns_per_tx'] / summaries[(mode, narrow)]['processing_ns_per_tx']:.1f}×"
        for mode, narrow, wide in pairs) + " |")
    lines.append("| mutations | " + " | ".join(
        f"{summaries[(mode, wide)]['mutations_per_tx'] / summaries[(mode, narrow)]['mutations_per_tx']:.1f}×"
        for mode, narrow, wide in pairs) + " |")
    lines.append("")
    lines.append("## Real sync vs sync disabled\n")
    lines.append("| Scenario | real tx/s | no-sync tx/s | no-sync / real | real ns/tx | no-sync ns/tx | WAL sync ns/tx (real) | sync share of real coordinator time |")
    lines.append("|---|---|---|---|---|---|---|---|")
    for index in SCENARIOS:
        real = summaries[("real", index)]
        disabled = summaries[("disabled", index)]
        lines.append(
            f"| {label(real)} | {fmt(real['tx_per_second'])} | {fmt(disabled['tx_per_second'])} | "
            f"{disabled['tx_per_second'] / real['tx_per_second']:.2f} | {fmt(real['processing_ns_per_tx'])} | "
            f"{fmt(disabled['processing_ns_per_tx'])} | {fmt(real['component_ns_per_tx']['WAL sync'])} | "
            f"{real['component_share']['WAL sync']:.1%} |")
    lines.append("")
    rocksdb = {index: rocksdb_summary(index) for index in (6, 9)}
    if all(rocksdb.values()):
        lines.append("## RocksDB same-session check\n")
        lines.append("| Scenario | dodb tx/s | RocksDB tx/s | dodb / RocksDB | dodb p99 µs | RocksDB p99 µs | dodb CPU % | RocksDB CPU % |")
        lines.append("|---|---|---|---|---|---|---|---|")
        for index, values in rocksdb.items():
            writers, width, _ = SCENARIOS[index]
            lines.append(
                f"| {writers}w w{width} uniform | {fmt(values['dodb_tx_per_second'])} | {fmt(values['rocksdb_tx_per_second'])} | "
                f"{values['dodb_tx_per_second'] / values['rocksdb_tx_per_second']:.3f} | {fmt(values['dodb_p99_us'])} | "
                f"{fmt(values['rocksdb_p99_us'])} | {fmt(values['dodb_cpu_percent_one_core'])} | {fmt(values['rocksdb_cpu_percent_one_core'])} |")
        lines.append("")
    with open(os.path.join(RESULTS, "tables.md"), "w", encoding="utf-8") as output_file:
        output_file.write("\n".join(lines) + "\n")
    with open(os.path.join(RESULTS, "analysis.json"), "w", encoding="utf-8") as output_file:
        json.dump({"summaries": {f"{mode}-{index}": value for (mode, index), value in summaries.items()},
                   "rocksdb": rocksdb}, output_file, indent=2, sort_keys=True)
    print("\n".join(lines))


if __name__ == "__main__":
    main()
