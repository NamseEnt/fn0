import glob
import json
import math
import os
import statistics
import sys


RESULTS = sys.argv[1]
REAL_SYNC = sys.argv[2]
CROSSDB = sys.argv[3]
DISTRIBUTIONS = ("uniform", "same-leaf-heavy", "different-leaf-heavy")
REPORT_NAMES = {
    "uniform": "uniform",
    "same-leaf-heavy": "compact locality",
    "different-leaf-heavy": "spread locality",
}
CROSSDB_NAMES = {
    "uniform": "uniform",
    "same-leaf-heavy": "compact-locality",
    "different-leaf-heavy": "spread-locality",
}
VARIANTS = ("phase-a", "page-delta")
GATE_SCENARIOS = (0, 3, 6, 9)


def scenarios():
    scenario_list = []
    for writer_count in (16, 64):
        for transaction_width in (1, 16):
            for distribution in DISTRIBUTIONS:
                scenario_list.append((writer_count, transaction_width, distribution))
    for transaction_width in (1, 16):
        scenario_list.append((1, transaction_width, "uniform"))
    return scenario_list


def load_line(path):
    with open(path, encoding="utf-8") as input_file:
        lines = input_file.read().splitlines()
    if len(lines) != 1:
        raise SystemExit(f"{path}: expected one JSONL row, found {len(lines)}")
    return json.loads(lines[0])


def load_jsonl(path):
    if not os.path.exists(path):
        return []
    with open(path, encoding="utf-8") as input_file:
        return [json.loads(line) for line in input_file if line.strip()]


def geometric_mean(values):
    return math.exp(statistics.mean(math.log(value) for value in values))


def peak_rss_by_output(path):
    return {os.path.basename(row["jsonl"]): row.get("rss_peak_sampled_kib") for row in load_jsonl(path)}


def dodb_paths(pattern):
    paths = sorted(glob.glob(pattern))
    if len(paths) != 3:
        raise SystemExit(f"{pattern}: expected 3 repetitions, found {len(paths)}")
    return paths


def summarize_dodb(paths, rss_peaks, expected_engine="planned-blink"):
    rows = [load_line(path) for path in paths]
    for path, row in zip(paths, rows):
        if row["engine"] != expected_engine or row["sync_mode"] != "real":
            raise SystemExit(f"{path}: engine {row['engine']} sync {row['sync_mode']}")
    transactions = sum(row["successful_transactions"] for row in rows)
    delta_records = sum(row.get("wal_page_delta_records_delta", 0) for row in rows)
    image_records = sum(row.get("wal_page_image_records_delta", row["page_images_delta"]) for row in rows)
    peaks = [rss_peaks.get(os.path.basename(path)) for path in paths]
    fallback_keys = (
        "superblock", "not_requested", "page_image_format", "ineligible_commit",
        "first_touch", "no_base", "not_smaller",
    )
    return {
        "engine_field": sorted({row["engine"] for row in rows}),
        "sync_mode_field": sorted({row["sync_mode"] for row in rows}),
        "git_commit_field": sorted({row["git_commit"] for row in rows}),
        "tx_per_second_runs": [row["logical_tx_per_second"] for row in rows],
        "tx_per_second": statistics.mean(row["logical_tx_per_second"] for row in rows),
        "mutation_ops_per_second": statistics.mean(row["mutation_ops_per_second"] for row in rows),
        "p50_us": statistics.mean(row["e2e_p50_us"] for row in rows),
        "p95_us": statistics.mean(row["e2e_p95_us"] for row in rows),
        "p99_us": statistics.mean(row["e2e_p99_us"] for row in rows),
        "cpu_percent_one_core": statistics.mean(row["cpu_utilization_percent_one_core"] for row in rows),
        "peak_rss_mib": max(peak for peak in peaks if peak) / 1024 if any(peaks) else None,
        "wal_bytes_per_tx": sum(row["wal_bytes_delta"] for row in rows) / transactions,
        "page_image_records_per_tx": image_records / transactions,
        "page_delta_records_per_tx": delta_records / transactions,
        "page_delta_payload_bytes_per_record": sum(
            row.get("wal_page_delta_payload_bytes_delta", 0) for row in rows) / max(delta_records, 1),
        "page_delta_frame_bytes_per_record": (sum(
            row.get("wal_page_delta_payload_bytes_delta", 0) for row in rows) + 52 * delta_records)
        / max(delta_records, 1),
        "page_delta_spans_per_record": sum(
            row.get("wal_page_delta_spans_delta", 0) for row in rows) / max(delta_records, 1),
        "fallback_counts": {
            key: sum(row.get(f"wal_image_{key}_delta", 0) for row in rows) for key in fallback_keys
        },
        "tx_per_sync": transactions / sum(row["wal_syncs_delta"] for row in rows),
        "wal_syncs": sum(row["wal_syncs_delta"] for row in rows) / len(rows),
        "mean_sync_ms": sum(row["wal_sync_nanos_total"] for row in rows)
        / sum(row["wal_syncs_delta"] for row in rows) / 1e6,
        "wal_write_mib_per_second": statistics.mean(
            row["wal_bytes_delta"] / (row["duration_ms"] / 1000) / 2**20 for row in rows),
        "errors": sum(row["errors"] for row in rows),
        "overloads": sum(row["overloads"] for row in rows),
        "conflicts": sum(row["conflicts"] for row in rows),
        "superblock_images_emitted": sum(row.get("superblock_images_emitted", 0) for row in rows),
        "leaf_splits": sum(row.get("leaf_splits", 0) for row in rows),
    }


def crossdb_summary(root, prefix, engine_name, scenario_index, writer_count, transaction_width, distribution):
    paths = sorted(glob.glob(os.path.join(
        root, "raw",
        f"{prefix}{engine_name}-{scenario_index:02d}-rep*-w{writer_count}-width{transaction_width}-{CROSSDB_NAMES[distribution]}.jsonl",
    )))
    if len(paths) != 3:
        return None
    rows = [load_line(path) for path in paths]
    for path, row in zip(paths, rows):
        if row["engine"] != engine_name:
            raise SystemExit(f"{path}: engine {row['engine']}")
        if row["measured"]["errors"] != 0:
            raise SystemExit(f"{path}: errors")
    return {
        "tx_per_second_runs": [row["measured"]["logical_tx_per_second"] for row in rows],
        "tx_per_second": statistics.mean(row["measured"]["logical_tx_per_second"] for row in rows),
        "mutation_ops_per_second": statistics.mean(row["measured"]["mutation_ops_per_second"] for row in rows),
        "p50_us": statistics.mean(row["measured"]["p50_us"] for row in rows),
        "p95_us": statistics.mean(row["measured"]["p95_us"] for row in rows),
        "p99_us": statistics.mean(row["measured"]["p99_us"] for row in rows),
        "cpu_percent_one_core": statistics.mean(row["cpu_utilization_percent_one_core"] for row in rows),
    }


def fmt(value, digits=0):
    if value is None:
        return "—"
    return f"{value:,.{digits}f}"


def scenario_label(writer_count, transaction_width, distribution):
    return f"{writer_count} | {transaction_width} | {REPORT_NAMES[distribution]}"


def variant_table(lines, title, entries):
    lines.append(f"## {title}\n")
    lines.append("| # | Writers | Width | Distribution | Variant | tx/s (rep1, rep2, rep3) | mean tx/s | mutation/s | p50 µs | p95 µs | p99 µs | CPU % | peak RSS MiB |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for entry in entries:
        for variant in VARIANTS:
            summary = entry[variant]
            lines.append(
                f"| {entry['scenario_index']} | {scenario_label(entry['writers'], entry['width'], entry['distribution'])} | {variant} | "
                + ", ".join(fmt(value) for value in summary["tx_per_second_runs"])
                + f" | {fmt(summary['tx_per_second'])} | {fmt(summary['mutation_ops_per_second'])} | {fmt(summary['p50_us'])} | "
                f"{fmt(summary['p95_us'])} | {fmt(summary['p99_us'])} | {fmt(summary['cpu_percent_one_core'])} | {fmt(summary['peak_rss_mib'])} |"
            )
    lines.append("")
    lines.append("| # | Writers | Width | Distribution | Variant | WAL B/tx | image rec/tx | delta rec/tx | delta frame B/rec | spans/rec | tx/sync | syncs/run | mean sync ms | WAL MiB/s | fallback (first touch / ineligible / not smaller / no base) |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for entry in entries:
        for variant in VARIANTS:
            summary = entry[variant]
            fallback = summary["fallback_counts"]
            lines.append(
                f"| {entry['scenario_index']} | {scenario_label(entry['writers'], entry['width'], entry['distribution'])} | {variant} | "
                f"{fmt(summary['wal_bytes_per_tx'], 1)} | {fmt(summary['page_image_records_per_tx'], 3)} | "
                f"{fmt(summary['page_delta_records_per_tx'], 3)} | {fmt(summary['page_delta_frame_bytes_per_record'], 1)} | "
                f"{fmt(summary['page_delta_spans_per_record'], 2)} | {fmt(summary['tx_per_sync'], 2)} | {fmt(summary['wal_syncs'])} | "
                f"{fmt(summary['mean_sync_ms'], 2)} | {fmt(summary['wal_write_mib_per_second'], 1)} | "
                f"{fallback['first_touch']} / {fallback['ineligible_commit']} / {fallback['not_smaller']} / {fallback['no_base']} |"
            )
    lines.append("")
    lines.append("| # | Writers | Width | Distribution | page-delta / phase-a tx/s | p99 phase-a → page-delta µs |")
    lines.append("|---|---|---|---|---|---|")
    for entry in entries:
        lines.append(
            f"| {entry['scenario_index']} | {scenario_label(entry['writers'], entry['width'], entry['distribution'])} | "
            f"{entry['ratio_vs_phase_a']:.3f} | {fmt(entry['phase-a']['p99_us'])} → {fmt(entry['page-delta']['p99_us'])} |"
        )
    lines.append("")


def phase_entries(phase, scenario_indexes, rss_peaks):
    entries = []
    for scenario_index in scenario_indexes:
        writer_count, transaction_width, distribution = scenarios()[scenario_index]
        name = f"w{writer_count}-width{transaction_width}-{distribution}"
        entry = {
            "scenario_index": scenario_index,
            "writers": writer_count,
            "width": transaction_width,
            "distribution": distribution,
        }
        for variant in VARIANTS:
            pattern = os.path.join(RESULTS, "raw", f"{phase}-{scenario_index:02d}-rep*-{variant}-{name}.jsonl")
            entry[variant] = summarize_dodb(dodb_paths(pattern), rss_peaks)
        entry["ratio_vs_phase_a"] = entry["page-delta"]["tx_per_second"] / entry["phase-a"]["tx_per_second"]
        entries.append(entry)
    return entries


def main():
    rss_peaks = peak_rss_by_output(os.path.join(RESULTS, "process-metrics.jsonl"))
    real_sync_peaks = peak_rss_by_output(os.path.join(REAL_SYNC, "process-metrics.jsonl"))
    lines = ["# WAL v3 Phase B tables\n"]
    analysis = {}

    gate = phase_entries("gate", GATE_SCENARIOS, rss_peaks)
    variant_table(lines, "First gate (4 scenarios, interleaved)", gate)
    gate_gm = geometric_mean([entry["ratio_vs_phase_a"] for entry in gate])
    lines.append(f"Gate geometric mean page-delta / phase-a: **{gate_gm:.3f}**\n")
    analysis["gate"] = {"entries": gate, "geometric_mean_ratio": gate_gm}

    matrix_available = glob.glob(os.path.join(RESULTS, "raw", "matrix-*.jsonl"))
    if matrix_available:
        matrix = phase_entries("matrix", range(len(scenarios())), rss_peaks)
        for entry in matrix:
            name = f"w{entry['writers']}-width{entry['width']}-{entry['distribution']}"
            main_pattern = os.path.join(REAL_SYNC, "raw", f"{entry['scenario_index']:02d}-rep*-exactmain-{name}.jsonl")
            entry["exact_main"] = summarize_dodb(dodb_paths(main_pattern), real_sync_peaks, "main-btree")
            entry["rocksdb_reused"] = crossdb_summary(
                CROSSDB, "", "rocksdb", entry["scenario_index"], entry["writers"], entry["width"], entry["distribution"])
            entry["rocksdb_confirm"] = crossdb_summary(
                RESULTS, "confirm-", "rocksdb", entry["scenario_index"], entry["writers"], entry["width"], entry["distribution"])
            entry["page_delta_vs_exact_main"] = entry["page-delta"]["tx_per_second"] / entry["exact_main"]["tx_per_second"]
            entry["phase_a_vs_exact_main"] = entry["phase-a"]["tx_per_second"] / entry["exact_main"]["tx_per_second"]
            if entry["rocksdb_reused"]:
                entry["page_delta_vs_rocksdb_reused"] = entry["page-delta"]["tx_per_second"] / entry["rocksdb_reused"]["tx_per_second"]
                entry["phase_a_vs_rocksdb_reused"] = entry["phase-a"]["tx_per_second"] / entry["rocksdb_reused"]["tx_per_second"]
        variant_table(lines, "Full matrix (14 scenarios, interleaved)", matrix)
        lines.append("## Reference comparisons (tx/s means)\n")
        lines.append("| # | Writers | Width | Distribution | page-delta | phase-a | ExactMain (reused) | RocksDB (reused, not interleaved) | PD / phase-a | PD / ExactMain | PD / RocksDB | phase-a / ExactMain | phase-a / RocksDB |")
        lines.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|")
        for entry in matrix:
            rocks = entry["rocksdb_reused"]
            lines.append(
                f"| {entry['scenario_index']} | {scenario_label(entry['writers'], entry['width'], entry['distribution'])} | "
                f"{fmt(entry['page-delta']['tx_per_second'])} | {fmt(entry['phase-a']['tx_per_second'])} | "
                f"{fmt(entry['exact_main']['tx_per_second'])} | {fmt(rocks['tx_per_second']) if rocks else '—'} | "
                f"{entry['ratio_vs_phase_a']:.3f} | {entry['page_delta_vs_exact_main']:.3f} | "
                f"{entry.get('page_delta_vs_rocksdb_reused', float('nan')):.3f} | {entry['phase_a_vs_exact_main']:.3f} | "
                f"{entry.get('phase_a_vs_rocksdb_reused', float('nan')):.3f} |"
            )
        lines.append("")
        multiwriter = [entry for entry in matrix if entry["writers"] > 1]
        categories = {
            "overall (12)": multiwriter,
            "writers 16": [entry for entry in multiwriter if entry["writers"] == 16],
            "writers 64": [entry for entry in multiwriter if entry["writers"] == 64],
            "width 1": [entry for entry in multiwriter if entry["width"] == 1],
            "width 16": [entry for entry in multiwriter if entry["width"] == 16],
            "uniform": [entry for entry in multiwriter if entry["distribution"] == "uniform"],
            "compact locality": [entry for entry in multiwriter if entry["distribution"] == "same-leaf-heavy"],
            "spread locality": [entry for entry in multiwriter if entry["distribution"] == "different-leaf-heavy"],
        }
        geometric_means = {}
        lines.append("## Geometric means over the 12 multiwriter scenarios\n")
        lines.append("| Category | PD / phase-a | PD / ExactMain | PD / RocksDB (reused) | phase-a / ExactMain | phase-a / RocksDB (reused) |")
        lines.append("|---|---|---|---|---|---|")
        for category, entries in categories.items():
            values = {
                "page_delta_vs_phase_a": geometric_mean([entry["ratio_vs_phase_a"] for entry in entries]),
                "page_delta_vs_exact_main": geometric_mean([entry["page_delta_vs_exact_main"] for entry in entries]),
                "page_delta_vs_rocksdb_reused": geometric_mean([entry["page_delta_vs_rocksdb_reused"] for entry in entries]),
                "phase_a_vs_exact_main": geometric_mean([entry["phase_a_vs_exact_main"] for entry in entries]),
                "phase_a_vs_rocksdb_reused": geometric_mean([entry["phase_a_vs_rocksdb_reused"] for entry in entries]),
            }
            geometric_means[category] = values
            lines.append(
                f"| {category} | {values['page_delta_vs_phase_a']:.3f} | {values['page_delta_vs_exact_main']:.3f} | "
                f"{values['page_delta_vs_rocksdb_reused']:.3f} | {values['phase_a_vs_exact_main']:.3f} | "
                f"{values['phase_a_vs_rocksdb_reused']:.3f} |"
            )
        lines.append("")
        analysis["matrix"] = {"entries": matrix, "geometric_means": geometric_means}

        confirmed = [entry for entry in matrix if entry["rocksdb_confirm"]]
        if confirmed:
            lines.append("## RocksDB confirmation (same session, rotated with page-delta)\n")
            lines.append("| # | Writers | Width | Distribution | page-delta tx/s (confirm) | RocksDB tx/s (confirm) | RocksDB p99 µs | page-delta p99 µs | PD / RocksDB (confirm) | RocksDB reused tx/s |")
            lines.append("|---|---|---|---|---|---|---|---|---|---|")
            confirm_ratios = []
            for entry in confirmed:
                name = f"w{entry['writers']}-width{entry['width']}-{entry['distribution']}"
                pattern = os.path.join(RESULTS, "raw", f"confirm-{entry['scenario_index']:02d}-rep*-page-delta-{name}.jsonl")
                page_delta = summarize_dodb(dodb_paths(pattern), rss_peaks)
                entry["page_delta_confirm"] = page_delta
                ratio = page_delta["tx_per_second"] / entry["rocksdb_confirm"]["tx_per_second"]
                entry["page_delta_vs_rocksdb_confirm"] = ratio
                confirm_ratios.append(ratio)
                lines.append(
                    f"| {entry['scenario_index']} | {scenario_label(entry['writers'], entry['width'], entry['distribution'])} | "
                    f"{fmt(page_delta['tx_per_second'])} | {fmt(entry['rocksdb_confirm']['tx_per_second'])} | "
                    f"{fmt(entry['rocksdb_confirm']['p99_us'])} | {fmt(page_delta['p99_us'])} | {ratio:.3f} | "
                    f"{fmt(entry['rocksdb_reused']['tx_per_second'])} |"
                )
            confirm_gm = geometric_mean(confirm_ratios)
            lines.append(f"\nGeometric mean PD / RocksDB over the confirmed scenarios: **{confirm_gm:.3f}**\n")
            analysis["rocksdb_confirm_geometric_mean"] = confirm_gm

    with open(os.path.join(RESULTS, "tables.md"), "w", encoding="utf-8") as output_file:
        output_file.write("\n".join(lines) + "\n")
    with open(os.path.join(RESULTS, "analysis.json"), "w", encoding="utf-8") as output_file:
        json.dump(analysis, output_file, indent=2, sort_keys=True)
    print("\n".join(lines))


if __name__ == "__main__":
    main()
