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
    with open(path, encoding="utf-8") as input_file:
        return [json.loads(line) for line in input_file if line.strip()]


def geometric_mean(values):
    return math.exp(statistics.mean(math.log(value) for value in values))


def dodb_rows(pattern):
    paths = sorted(glob.glob(pattern))
    if len(paths) != 3:
        raise SystemExit(f"{pattern}: expected 3 repetitions, found {len(paths)}")
    return [load_line(path) for path in paths]


def rss_by_output(process_metrics_path):
    peaks = {}
    for row in load_jsonl(process_metrics_path):
        peaks[os.path.basename(row["jsonl"])] = row.get("rss_peak_sampled_kib")
    return peaks


def summarize_dodb(rows, rss_peaks, name_pattern_prefix):
    transactions = [row["successful_transactions"] for row in rows]
    summary = {
        "engine_field": sorted({row["engine"] for row in rows}),
        "sync_mode_field": sorted({row["sync_mode"] for row in rows}),
        "git_commit_field": sorted({row["git_commit"] for row in rows}),
        "tx_per_second_runs": [row["logical_tx_per_second"] for row in rows],
        "tx_per_second": statistics.mean(row["logical_tx_per_second"] for row in rows),
        "mutation_ops_per_second": statistics.mean(row["mutation_ops_per_second"] for row in rows),
        "p50_us": statistics.mean(row["e2e_p50_us"] for row in rows),
        "p95_us": statistics.mean(row["e2e_p95_us"] for row in rows),
        "p99_us": statistics.mean(row["e2e_p99_us"] for row in rows),
        "wal_bytes_per_tx": sum(row["wal_bytes_delta"] for row in rows) / sum(transactions),
        "page_images_per_tx": sum(row["page_images_delta"] for row in rows) / sum(transactions),
        "wal_bytes_per_mutation": sum(row["wal_bytes_delta"] for row in rows)
        / sum(row["mutation_ops"] for row in rows),
        "tx_per_sync": sum(transactions) / sum(row["wal_syncs_delta"] for row in rows),
        "mean_sync_ms": sum(row["wal_sync_nanos_total"] for row in rows)
        / sum(row["wal_syncs_delta"] for row in rows) / 1e6,
        "wal_write_mib_per_second": statistics.mean(
            row["wal_bytes_delta"] / (row["duration_ms"] / 1000) / 2**20 for row in rows
        ),
        "cpu_percent_one_core": statistics.mean(row["cpu_utilization_percent_one_core"] for row in rows),
        "errors": sum(row["errors"] for row in rows),
        "overloads": sum(row["overloads"] for row in rows),
        "conflicts": sum(row["conflicts"] for row in rows),
        "superblock_images_emitted": sum(row.get("superblock_images_emitted", 0) for row in rows),
        "superblock_images_elided": sum(row.get("superblock_images_elided", 0) for row in rows),
        "leaf_splits": sum(row.get("leaf_splits", 0) for row in rows),
    }
    peaks = [rss_peaks.get(row_name) for row_name in name_pattern_prefix]
    summary["peak_rss_mib"] = max(peak for peak in peaks if peak) / 1024 if any(peaks) else None
    return summary


def crossdb_summary(engine_name, scenario_index, writer_count, transaction_width, distribution):
    paths = sorted(glob.glob(os.path.join(
        CROSSDB, "raw",
        f"{engine_name}-{scenario_index:02d}-rep*-w{writer_count}-width{transaction_width}-{CROSSDB_NAMES[distribution]}.jsonl",
    )))
    if len(paths) != 3:
        raise SystemExit(f"{engine_name} scenario {scenario_index}: expected 3 files, found {len(paths)}")
    rows = [load_line(path) for path in paths]
    return {
        "tx_per_second": statistics.mean(row["measured"]["logical_tx_per_second"] for row in rows),
        "mutation_ops_per_second": statistics.mean(row["measured"]["mutation_ops_per_second"] for row in rows),
        "p99_us": statistics.mean(row["measured"]["p99_us"] for row in rows),
        "cpu_percent_one_core": statistics.mean(row["cpu_utilization_percent_one_core"] for row in rows),
    }


def fmt(value, digits=0):
    if value is None:
        return "—"
    return f"{value:,.{digits}f}"


def core():
    new_rss = rss_by_output(os.path.join(RESULTS, "process-metrics.jsonl"))
    old_rss = rss_by_output(os.path.join(REAL_SYNC, "raw", "process-metrics.jsonl")) if os.path.exists(
        os.path.join(REAL_SYNC, "raw", "process-metrics.jsonl")) else rss_by_output(
        os.path.join(REAL_SYNC, "process-metrics.jsonl"))
    table = []
    for scenario_index, (writer_count, transaction_width, distribution) in enumerate(scenarios()):
        scenario_name = f"w{writer_count}-width{transaction_width}-{distribution}"
        blink_pattern = os.path.join(RESULTS, "raw", f"{scenario_index:02d}-rep*-planned-blink-{scenario_name}.jsonl")
        btree_pattern = os.path.join(REAL_SYNC, "raw", f"{scenario_index:02d}-rep*-planned-{scenario_name}.jsonl")
        main_pattern = os.path.join(REAL_SYNC, "raw", f"{scenario_index:02d}-rep*-exactmain-{scenario_name}.jsonl")
        blink_rows = dodb_rows(blink_pattern)
        btree_rows = dodb_rows(btree_pattern)
        main_rows = dodb_rows(main_pattern)
        blink = summarize_dodb(blink_rows, new_rss, [os.path.basename(path) for path in sorted(glob.glob(blink_pattern))])
        btree = summarize_dodb(btree_rows, old_rss, [os.path.basename(path) for path in sorted(glob.glob(btree_pattern))])
        exact_main = summarize_dodb(main_rows, old_rss, [os.path.basename(path) for path in sorted(glob.glob(main_pattern))])
        rocksdb = crossdb_summary("rocksdb", scenario_index, writer_count, transaction_width, distribution)
        turso_wal = crossdb_summary("turso-wal", scenario_index, writer_count, transaction_width, distribution)
        turso_mvcc = crossdb_summary("turso-mvcc-gc", scenario_index, writer_count, transaction_width, distribution)
        table.append({
            "scenario_index": scenario_index,
            "writers": writer_count,
            "width": transaction_width,
            "distribution": distribution,
            "planned_blink": blink,
            "experiment_main_btree": btree,
            "exact_main": exact_main,
            "rocksdb": rocksdb,
            "turso_wal": turso_wal,
            "turso_mvcc_gc": turso_mvcc,
            "ratio_vs_turso_wal": blink["tx_per_second"] / turso_wal["tx_per_second"],
            "ratio_vs_turso_mvcc_gc": blink["tx_per_second"] / turso_mvcc["tx_per_second"],
            "experiment_main_btree_vs_turso_wal": btree["tx_per_second"] / turso_wal["tx_per_second"],
            "experiment_main_btree_vs_turso_mvcc_gc": btree["tx_per_second"] / turso_mvcc["tx_per_second"],
            "ratio_vs_experiment_main_btree": blink["tx_per_second"] / btree["tx_per_second"],
            "ratio_vs_exact_main": blink["tx_per_second"] / exact_main["tx_per_second"],
            "ratio_vs_rocksdb": blink["tx_per_second"] / rocksdb["tx_per_second"],
            "experiment_main_btree_vs_rocksdb": btree["tx_per_second"] / rocksdb["tx_per_second"],
        })

    multiwriter = [row for row in table if row["writers"] > 1]
    categories = {
        "overall (12)": multiwriter,
        "writers 16": [row for row in multiwriter if row["writers"] == 16],
        "writers 64": [row for row in multiwriter if row["writers"] == 64],
        "width 1": [row for row in multiwriter if row["width"] == 1],
        "width 16": [row for row in multiwriter if row["width"] == 16],
        "uniform": [row for row in multiwriter if row["distribution"] == "uniform"],
        "compact locality": [row for row in multiwriter if row["distribution"] == "same-leaf-heavy"],
        "spread locality": [row for row in multiwriter if row["distribution"] == "different-leaf-heavy"],
    }
    geometric_means = {}
    for category, rows in categories.items():
        geometric_means[category] = {
            "planned_blink_vs_experiment_main_btree": geometric_mean([row["ratio_vs_experiment_main_btree"] for row in rows]),
            "planned_blink_vs_exact_main": geometric_mean([row["ratio_vs_exact_main"] for row in rows]),
            "planned_blink_vs_rocksdb": geometric_mean([row["ratio_vs_rocksdb"] for row in rows]),
            "experiment_main_btree_vs_rocksdb": geometric_mean([row["experiment_main_btree_vs_rocksdb"] for row in rows]),
            "planned_blink_vs_turso_wal": geometric_mean([row["ratio_vs_turso_wal"] for row in rows]),
            "planned_blink_vs_turso_mvcc_gc": geometric_mean([row["ratio_vs_turso_mvcc_gc"] for row in rows]),
            "experiment_main_btree_vs_turso_wal": geometric_mean([row["experiment_main_btree_vs_turso_wal"] for row in rows]),
            "experiment_main_btree_vs_turso_mvcc_gc": geometric_mean([row["experiment_main_btree_vs_turso_mvcc_gc"] for row in rows]),
            "experiment_main_btree_vs_exact_main": geometric_mean([
                row["experiment_main_btree"]["tx_per_second"] / row["exact_main"]["tx_per_second"] for row in rows
            ]),
        }

    validity_control = {}
    lines = []
    lines.append("## Successful logical tx/s (mean of 3 repetitions)\n")
    lines.append("| # | Writers | Width | Distribution | A planned-blink | B experiment main-btree | C ExactMain | D RocksDB | A/B | A/C | A/D |")
    lines.append("|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|")
    for row in table:
        lines.append(
            f"| {row['scenario_index']} | {row['writers']} | {row['width']} | {REPORT_NAMES[row['distribution']]} | "
            f"{fmt(row['planned_blink']['tx_per_second'])} | {fmt(row['experiment_main_btree']['tx_per_second'])} | "
            f"{fmt(row['exact_main']['tx_per_second'])} | {fmt(row['rocksdb']['tx_per_second'])} | "
            f"{row['ratio_vs_experiment_main_btree']:.3f} | {row['ratio_vs_exact_main']:.3f} | {row['ratio_vs_rocksdb']:.3f} |"
        )
    lines.append("")
    lines.append("## Successful mutation ops/s (mean of 3 repetitions)\n")
    lines.append("| # | Writers | Width | Distribution | A planned-blink | B experiment main-btree | C ExactMain | D RocksDB |")
    lines.append("|---:|---:|---:|---|---:|---:|---:|---:|")
    for row in table:
        lines.append(
            f"| {row['scenario_index']} | {row['writers']} | {row['width']} | {REPORT_NAMES[row['distribution']]} | "
            f"{fmt(row['planned_blink']['mutation_ops_per_second'])} | {fmt(row['experiment_main_btree']['mutation_ops_per_second'])} | "
            f"{fmt(row['exact_main']['mutation_ops_per_second'])} | {fmt(row['rocksdb']['mutation_ops_per_second'])} |"
        )
    lines.append("")
    lines.append("## Per-repetition planned-blink tx/s\n")
    lines.append("| # | Writers | Width | Distribution | rep 1 | rep 2 | rep 3 | min/max |")
    lines.append("|---:|---:|---:|---|---:|---:|---:|---:|")
    for row in table:
        runs = row["planned_blink"]["tx_per_second_runs"]
        lines.append(
            f"| {row['scenario_index']} | {row['writers']} | {row['width']} | {REPORT_NAMES[row['distribution']]} | "
            + " | ".join(fmt(value) for value in runs) + f" | {min(runs) / max(runs):.3f} |"
        )
    lines.append("")
    lines.append("## Geometric means over the 12 multiwriter scenarios\n")
    lines.append("| Category | A / B experiment main-btree | A / C ExactMain | A / D RocksDB | B / D (previous \"Planned\" / RocksDB) | B / C |")
    lines.append("|---|---:|---:|---:|---:|---:|")
    for category, values in geometric_means.items():
        lines.append(
            f"| {category} | {values['planned_blink_vs_experiment_main_btree']:.3f} | {values['planned_blink_vs_exact_main']:.3f} | "
            f"{values['planned_blink_vs_rocksdb']:.3f} | {values['experiment_main_btree_vs_rocksdb']:.3f} | "
            f"{values['experiment_main_btree_vs_exact_main']:.3f} |"
        )
    lines.append("")
    lines.append("## Geometric means against the Turso rows of the cross-DB run\n")
    lines.append("| Category | A / Turso WAL | A / Turso MVCC GC-on | B / Turso WAL (previously reported as Planned) | B / Turso MVCC GC-on (previously reported as Planned) |")
    lines.append("|---|---:|---:|---:|---:|")
    for category, values in geometric_means.items():
        lines.append(
            f"| {category} | {values['planned_blink_vs_turso_wal']:.3f} | {values['planned_blink_vs_turso_mvcc_gc']:.3f} | "
            f"{values['experiment_main_btree_vs_turso_wal']:.3f} | {values['experiment_main_btree_vs_turso_mvcc_gc']:.3f} |"
        )
    lines.append("")
    control_paths = sorted(glob.glob(os.path.join(RESULTS, "control-scenario0", "*.jsonl")))
    if control_paths:
        control_rows = [load_line(path) for path in control_paths]
        control_mean = statistics.mean(row["logical_tx_per_second"] for row in control_rows)
        sensitivity_ratios = []
        for row in multiwriter:
            blink_tx = control_mean if row["scenario_index"] == 0 else row["planned_blink"]["tx_per_second"]
            sensitivity_ratios.append((
                blink_tx / row["experiment_main_btree"]["tx_per_second"],
                blink_tx / row["exact_main"]["tx_per_second"],
                blink_tx / row["rocksdb"]["tx_per_second"],
            ))
        sensitivity = {
            "control_engine_fields": sorted({row["engine"] for row in control_rows}),
            "control_sync_mode_fields": sorted({row["sync_mode"] for row in control_rows}),
            "control_tx_per_second_runs": [row["logical_tx_per_second"] for row in control_rows],
            "control_tx_per_second_mean": control_mean,
            "overall_gm_vs_experiment_main_btree": geometric_mean([ratio[0] for ratio in sensitivity_ratios]),
            "overall_gm_vs_exact_main": geometric_mean([ratio[1] for ratio in sensitivity_ratios]),
            "overall_gm_vs_rocksdb": geometric_mean([ratio[2] for ratio in sensitivity_ratios]),
        }
        validity_control.update(sensitivity)
        lines.append("## Scenario 0 control (run after the matrix; not part of the primary table)\n")
        lines.append(
            f"Three extra 16-writer width-1 uniform runs with the matrix seeds gave "
            + ", ".join(fmt(value) for value in sensitivity["control_tx_per_second_runs"])
            + f" tx/s (mean {fmt(control_mean)}). Replacing only scenario 0 with this mean changes the 12-scenario GM to "
            f"{sensitivity['overall_gm_vs_experiment_main_btree']:.3f} vs experiment main-btree, {sensitivity['overall_gm_vs_exact_main']:.3f} vs ExactMain "
            f"and {sensitivity['overall_gm_vs_rocksdb']:.3f} vs RocksDB.\n"
        )
    lines.append("## Latency p50 / p95 / p99 µs (mean of repetition percentiles)\n")
    lines.append("| # | Writers | Width | Distribution | A planned-blink | B experiment main-btree | D RocksDB p99 |")
    lines.append("|---:|---:|---:|---|---:|---:|---:|")
    for row in table:
        blink = row["planned_blink"]
        btree = row["experiment_main_btree"]
        lines.append(
            f"| {row['scenario_index']} | {row['writers']} | {row['width']} | {REPORT_NAMES[row['distribution']]} | "
            f"{fmt(blink['p50_us'])} / {fmt(blink['p95_us'])} / {fmt(blink['p99_us'])} | "
            f"{fmt(btree['p50_us'])} / {fmt(btree['p95_us'])} / {fmt(btree['p99_us'])} | {fmt(row['rocksdb']['p99_us'])} |"
        )
    lines.append("")
    lines.append("## WAL and resources\n")
    lines.append("| # | Writers | Width | Distribution | A WAL B/tx | A images/tx | A tx/sync | A mean sync ms | A WAL MiB/s | B WAL B/tx | B images/tx | B tx/sync | B mean sync ms | B WAL MiB/s | A CPU % | B CPU % | A peak RSS MiB | B peak RSS MiB |")
    lines.append("|---:|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for row in table:
        blink = row["planned_blink"]
        btree = row["experiment_main_btree"]
        lines.append(
            f"| {row['scenario_index']} | {row['writers']} | {row['width']} | {REPORT_NAMES[row['distribution']]} | "
            f"{fmt(blink['wal_bytes_per_tx'])} | {blink['page_images_per_tx']:.3f} | {blink['tx_per_sync']:.2f} | {blink['mean_sync_ms']:.2f} | {blink['wal_write_mib_per_second']:.1f} | "
            f"{fmt(btree['wal_bytes_per_tx'])} | {btree['page_images_per_tx']:.3f} | {btree['tx_per_sync']:.2f} | {btree['mean_sync_ms']:.2f} | {btree['wal_write_mib_per_second']:.1f} | "
            f"{blink['cpu_percent_one_core']:.0f} | {btree['cpu_percent_one_core']:.0f} | {fmt(blink['peak_rss_mib'])} | {fmt(btree['peak_rss_mib'])} |"
        )
    lines.append("")
    validity = {
        "planned_blink_rows": sum(1 for _ in glob.glob(os.path.join(RESULTS, "raw", "??-rep*-planned-blink-*.jsonl"))),
        "planned_blink_engine_fields": sorted({value for row in table for value in row["planned_blink"]["engine_field"]}),
        "planned_blink_sync_mode_fields": sorted({value for row in table for value in row["planned_blink"]["sync_mode_field"]}),
        "experiment_main_btree_engine_fields": sorted({value for row in table for value in row["experiment_main_btree"]["engine_field"]}),
        "exact_main_engine_fields": sorted({value for row in table for value in row["exact_main"]["engine_field"]}),
        "planned_blink_errors": sum(row["planned_blink"]["errors"] for row in table),
        "planned_blink_overloads": sum(row["planned_blink"]["overloads"] for row in table),
        "planned_blink_conflicts": sum(row["planned_blink"]["conflicts"] for row in table),
        "planned_blink_superblock_images_emitted": sum(row["planned_blink"]["superblock_images_emitted"] for row in table),
        "planned_blink_superblock_images_elided": sum(row["planned_blink"]["superblock_images_elided"] for row in table),
        "planned_blink_measured_leaf_splits": sum(row["planned_blink"]["leaf_splits"] for row in table),
    }
    lines.append("## Validity\n")
    lines.append("```json")
    lines.append(json.dumps(validity, indent=2, sort_keys=True))
    lines.append("```")
    lines.append("")
    validity["scenario0_control"] = validity_control
    return table, geometric_means, validity, lines


WIDTH1_WAL_BYTES_PER_TX = 4224
WARMUP_MS = 10_000


def interpolate(monitor, key, unix_ms):
    before = [sample for sample in monitor if sample["unix_ms"] <= unix_ms]
    after = [sample for sample in monitor if sample["unix_ms"] >= unix_ms]
    if not before or not after:
        return None
    left = max(before, key=lambda sample: sample["unix_ms"])
    right = min(after, key=lambda sample: sample["unix_ms"])
    if right["unix_ms"] == left["unix_ms"]:
        return left[key]
    fraction = (unix_ms - left["unix_ms"]) / (right["unix_ms"] - left["unix_ms"])
    return left[key] + (right[key] - left[key]) * fraction


def crossing_time(monitor, key, value):
    for left, right in zip(monitor, monitor[1:]):
        if left[key] <= value <= right[key] and right[key] != left[key]:
            fraction = (value - left[key]) / (right[key] - left[key])
            return left["unix_ms"] + (right["unix_ms"] - left["unix_ms"]) * fraction
    return None


def process_start_ms(monitor):
    first = monitor[0]
    return first["unix_ms"] - first["t_since_start_s"] * 1000


def monitor_windows(monitor, measurement_start_ms, window_count):
    rows = []
    previous_wal = interpolate(monitor, "wal_bytes", measurement_start_ms)
    previous_rss = interpolate(monitor, "rss_kib", measurement_start_ms)
    for window_index in range(window_count):
        window_end_ms = measurement_start_ms + (window_index + 1) * 10_000
        wal_end = interpolate(monitor, "wal_bytes", window_end_ms)
        rss_end = interpolate(monitor, "rss_kib", window_end_ms)
        if wal_end is None or rss_end is None:
            break
        rows.append({
            "window_index": window_index,
            "wal_bytes_end": wal_end,
            "wal_growth_bytes": wal_end - previous_wal,
            "derived_tx_per_second": (wal_end - previous_wal) / WIDTH1_WAL_BYTES_PER_TX / 10.0,
            "rss_kib_end": rss_end,
            "rss_growth_kib": rss_end - previous_rss,
            "mem_available_kib_end": interpolate(monitor, "mem_available_kib", window_end_ms),
        })
        previous_wal = wal_end
        previous_rss = rss_end
    return rows


def sustained():
    output = {}
    lines = ["## 120-second sustained, planned-blink\n"]
    records = {}
    monitors = {}
    for writer_count, scenario_index in ((16, 14), (64, 15)):
        base = os.path.join(RESULTS, "raw", f"sustained-planned-blink-{scenario_index:02d}-rep1-w{writer_count}-width1-uniform")
        monitors[writer_count] = load_jsonl(base + ".process-monitor.jsonl")
        with open(base + ".jsonl", encoding="utf-8") as record_file:
            text = record_file.read().strip()
        records[writer_count] = json.loads(text) if text else None

    reference = records[16]
    reference_monitor = monitors[16]
    reference_start_ms = reference["timestamp_unix_ms"] - reference["duration_ms"]
    seed_wal_bytes = interpolate(reference_monitor, "wal_bytes", reference_start_ms - WARMUP_MS)
    alignment = {"seed_wal_bytes_estimate": seed_wal_bytes}

    for writer_count in (16, 64):
        record = records[writer_count]
        monitor = monitors[writer_count]
        started_ms = process_start_ms(monitor)
        seed_end_ms = crossing_time(monitor, "wal_bytes", seed_wal_bytes)
        if record is not None:
            measurement_start_ms = record["timestamp_unix_ms"] - record["duration_ms"]
            window_count = record["window_count"]
        else:
            measurement_start_ms = seed_end_ms + WARMUP_MS
            window_count = 12
        rows = monitor_windows(monitor, measurement_start_ms, window_count)
        for row in rows:
            if record is not None:
                row["tx_per_second"] = record[f"window_{row['window_index']:02d}_logical_tx_per_second"]
                row["p50_us"] = record[f"window_{row['window_index']:02d}_p50_us"]
                row["p99_us"] = record[f"window_{row['window_index']:02d}_p99_us"]
        last_sample = monitor[-1]
        summary = {
            "writers": writer_count,
            "completed": record is not None,
            "process_seconds": last_sample["t_since_start_s"],
            "seed_seconds": (seed_end_ms - started_ms) / 1000 if seed_end_ms else None,
            "measurement_start_s_since_process_start": (measurement_start_ms - started_ms) / 1000,
            "rss_kib_at_measurement_start": interpolate(monitor, "rss_kib", measurement_start_ms),
            "wal_bytes_at_measurement_start": interpolate(monitor, "wal_bytes", measurement_start_ms),
            "rss_kib_at_seed_end": interpolate(monitor, "rss_kib", seed_end_ms) if seed_end_ms else None,
            "peak_rss_kib": max(sample["rss_kib"] or 0 for sample in monitor),
            "max_wal_bytes": max(sample["wal_bytes"] for sample in monitor),
            "min_mem_available_kib": min(sample["mem_available_kib"] for sample in monitor),
            "windows": rows,
        }
        measured_wal_growth = sum(row["wal_growth_bytes"] for row in rows)
        measured_rss_growth = sum(row["rss_growth_kib"] for row in rows)
        summary["measured_wal_growth_bytes"] = measured_wal_growth
        summary["measured_rss_growth_kib"] = measured_rss_growth
        summary["measured_seconds_covered"] = 10 * len(rows)
        summary["rss_bytes_per_wal_byte"] = measured_rss_growth * 1024 / measured_wal_growth if measured_wal_growth else None
        summary["derived_tx_per_second"] = measured_wal_growth / WIDTH1_WAL_BYTES_PER_TX / (10 * len(rows)) if rows else None
        if record is not None:
            summary.update({
                "engine_field": record["engine"],
                "sync_mode_field": record["sync_mode"],
                "successful_transactions": record["successful_transactions"],
                "tx_per_second": record["logical_tx_per_second"],
                "p50_us": record["e2e_p50_us"],
                "p95_us": record["e2e_p95_us"],
                "p99_us": record["e2e_p99_us"],
                "errors": record["errors"],
                "wal_bytes_per_tx": record["wal_bytes_delta"] / record["successful_transactions"],
                "page_images_per_tx": record["page_images_delta"] / record["successful_transactions"],
                "tx_per_sync": record["successful_transactions"] / record["wal_syncs_delta"],
                "mean_sync_ms": record["wal_sync_nanos_total"] / record["wal_syncs_delta"] / 1e6,
                "cpu_percent_one_core": record["cpu_utilization_percent_one_core"],
                "wal_bytes_delta": record["wal_bytes_delta"],
            })
            deviations = [abs(row["derived_tx_per_second"] - row["tx_per_second"]) / row["tx_per_second"] for row in rows]
            alignment[f"w{writer_count}_derived_vs_measured_max_window_deviation"] = max(deviations)
            alignment[f"w{writer_count}_derived_vs_measured_total_deviation"] = abs(summary["derived_tx_per_second"] - summary["tx_per_second"]) / summary["tx_per_second"]
        output[writer_count] = summary

        lines.append(f"### {writer_count} writers\n")
        if record is not None:
            lines.append(
                f"Completed. engine `{record['engine']}`, sync `{record['sync_mode']}`, {fmt(summary['tx_per_second'])} tx/s over "
                f"{record['duration_ms'] / 1000:.1f} s, p50/p95/p99 {fmt(summary['p50_us'])} / {fmt(summary['p95_us'])} / {fmt(summary['p99_us'])} µs, "
                f"errors {record['errors']}, WAL {fmt(summary['wal_bytes_per_tx'])} B/tx, {summary['page_images_per_tx']:.3f} images/tx, "
                f"{summary['tx_per_sync']:.2f} tx/sync, mean sync {summary['mean_sync_ms']:.2f} ms, CPU {summary['cpu_percent_one_core']:.0f}%.\n"
            )
        else:
            lines.append(
                f"Killed by the runner's low-memory guard (MemAvailable < 256 MiB) {last_sample['t_since_start_s']:.1f} s after process start, "
                f"{(last_sample['unix_ms'] - measurement_start_ms) / 1000:.1f} s into the estimated measurement interval. phase0-bench writes its JSON row only at the end, "
                f"so tx/s below is derived from WAL growth at {WIDTH1_WAL_BYTES_PER_TX} B/tx; latency is unavailable.\n"
            )
        lines.append(
            f"Seeding 1,000,000 rows took about {fmt(summary['seed_seconds'], 1)} s and left RSS {fmt((summary['rss_kib_at_seed_end'] or 0) / 1024)} MiB. "
            f"Measurement started {summary['measurement_start_s_since_process_start']:.1f} s after process start with RSS "
            f"{fmt(summary['rss_kib_at_measurement_start'] / 1024)} MiB and WAL {fmt(summary['wal_bytes_at_measurement_start'] / 2**20)} MiB.\n"
        )
        lines.append("| Window | measured tx/s | derived tx/s (WAL growth / 4,224 B) | p50 µs | p99 µs | WAL at end (MiB) | WAL growth (MiB) | RSS at end (MiB) | RSS growth (MiB) | MemAvailable (MiB) |")
        lines.append("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
        for row in rows:
            lines.append(
                f"| {row['window_index'] * 10}-{row['window_index'] * 10 + 10} s | {fmt(row.get('tx_per_second'))} | {fmt(row['derived_tx_per_second'])} | "
                f"{fmt(row.get('p50_us'))} | {fmt(row.get('p99_us'))} | {fmt(row['wal_bytes_end'] / 2**20)} | {fmt(row['wal_growth_bytes'] / 2**20)} | "
                f"{fmt(row['rss_kib_end'] / 1024)} | {fmt(row['rss_growth_kib'] / 1024)} | {fmt((row['mem_available_kib_end'] or 0) / 1024)} |"
            )
        lines.append("")
        lines.append(
            f"Covered {summary['measured_seconds_covered']} s of measurement: WAL grew {fmt(measured_wal_growth / 2**20)} MiB and RSS grew "
            f"{fmt(measured_rss_growth / 1024)} MiB ({summary['rss_bytes_per_wal_byte']:.3f} RSS bytes per WAL byte). Peak sampled RSS "
            f"{fmt(summary['peak_rss_kib'] / 1024)} MiB, max WAL {fmt(summary['max_wal_bytes'] / 2**20)} MiB, minimum MemAvailable "
            f"{fmt(summary['min_mem_available_kib'] / 1024)} MiB.\n"
        )

    previous = {}
    for path in sorted(glob.glob(os.path.join(CROSSDB, "raw", "sustained-*.jsonl"))):
        if path.endswith(".process-monitor.jsonl"):
            continue
        record = load_line(path)
        name = os.path.basename(path)
        if name.startswith("sustained-dodb-planned-"):
            previous[f"experiment-main-btree|{record['writers']}"] = {
                "engine_field": record["engine"],
                "tx_per_second": record["logical_tx_per_second"],
                "p99_us": record["e2e_p99_us"],
                "wal_bytes_per_tx": record["wal_bytes_delta"] / record["successful_transactions"],
            }
        elif name.startswith("sustained-rocksdb-"):
            previous[f"rocksdb|{record['writers']}"] = {
                "tx_per_second": record["measured"]["logical_tx_per_second"],
                "p99_us": record["measured"]["p99_us"],
            }
    lines.append("### Sustained comparison (B and D reused from the cross-DB run)\n")
    lines.append("| Writers | A planned-blink tx/s | B experiment main-btree tx/s | D RocksDB tx/s | A/B | A/D | A p99 µs | B p99 µs | D p99 µs |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for writer_count, summary in sorted(output.items()):
        btree = previous[f"experiment-main-btree|{writer_count}"]
        rocksdb = previous[f"rocksdb|{writer_count}"]
        blink_tx = summary.get("tx_per_second") or summary["derived_tx_per_second"]
        marker = "" if summary["completed"] else f" (derived, {summary['measured_seconds_covered']} s before kill)"
        lines.append(
            f"| {writer_count} | {fmt(blink_tx)}{marker} | {fmt(btree['tx_per_second'])} | {fmt(rocksdb['tx_per_second'])} | "
            f"{blink_tx / btree['tx_per_second']:.3f} | {blink_tx / rocksdb['tx_per_second']:.3f} | "
            f"{fmt(summary.get('p99_us'))} | {fmt(btree['p99_us'])} | {fmt(rocksdb['p99_us'])} |"
        )
    lines.append("")
    lines.append("### Window alignment check\n")
    lines.append("```json")
    lines.append(json.dumps(alignment, indent=2, sort_keys=True))
    lines.append("```")
    lines.append("")
    return {"runs": output, "alignment": alignment}, previous, lines


def main():
    table, geometric_means, validity, core_lines = core()
    output = {"core": table, "geometric_means": geometric_means, "validity": validity}
    lines = ["# planned-blink durable baseline tables\n"] + core_lines
    if glob.glob(os.path.join(RESULTS, "raw", "sustained-planned-blink-*.jsonl")):
        windows_out, previous, sustained_lines = sustained()
        output["sustained"] = windows_out
        output["sustained_reused_references"] = previous
        lines += sustained_lines
    with open(os.path.join(RESULTS, "analysis.json"), "w", encoding="utf-8") as output_file:
        json.dump(output, output_file, indent=2, sort_keys=True)
    with open(os.path.join(RESULTS, "tables.md"), "w", encoding="utf-8") as output_file:
        output_file.write("\n".join(lines).rstrip("\n") + "\n")
    print(json.dumps({"geometric_means": geometric_means["overall (12)"], "validity": validity}, indent=2))


if __name__ == "__main__":
    main()
