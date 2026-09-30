import csv
import json
import math
import statistics
from pathlib import Path


RESULT_DIRECTORY = Path(__file__).resolve().parents[1]
RAW_DIRECTORY = RESULT_DIRECTORY / "raw"
FILESYSTEM_RUNS = {
    "ext4": ("ext4",),
    "XFS": ("xfs",),
    "Btrfs": ("btrfs-pre", "btrfs-post"),
    "ZFS": ("zfs",),
}
EXPECTED_COMMIT = "08e82ec484bd62aeb659471a39f317b86cfb488f"


def read_json(path):
    return json.loads(path.read_text())


def run_metadata(filesystem, kind, repetition, width=None):
    output_stem = filesystem.lower()
    if kind == "dodb":
        output_stem = f"{output_stem}-width{width}"
    output_stem = f"{output_stem}-{kind}-rep{repetition}"
    return read_json(RAW_DIRECTORY / f"{output_stem}.meta.json")


def read_fio_run(filesystem, kind, repetition):
    path = RAW_DIRECTORY / f"{filesystem}-{kind}-rep{repetition}.json"
    report = read_json(path)
    job = report["jobs"][0]
    metadata = run_metadata(filesystem, kind, repetition)
    if job["error"] != 0:
        raise ValueError(f"fio error in {path}")
    if job["write"]["runtime"] < 19900:
        raise ValueError(f"fio runtime below 19.9 seconds in {path}")
    return job, metadata


def fio_sync_metric(job, statistic):
    return job["sync"]["lat_ns"][statistic]


def fio_write_percentile(job, percentile):
    return job["write"]["clat_ns"]["percentile"][percentile]


def dodb_run(filesystem, width, repetition):
    path = RAW_DIRECTORY / f"{filesystem}-width{width}-dodb-rep{repetition}.jsonl"
    rows = [json.loads(line) for line in path.read_text().splitlines() if line]
    if len(rows) != 1:
        raise ValueError(f"expected one dodb row in {path}, got {len(rows)}")
    row = rows[0]
    metadata = run_metadata(filesystem, "dodb", repetition, width)
    expected_fields = {
        "git_commit": EXPECTED_COMMIT,
        "engine": "parallel-blink",
        "sync_mode": "real",
        "writers": 64,
        "working_set": 100000,
        "key_size": 16,
        "value_size": 64,
        "transaction_width": width,
        "tokio_workers": 2,
        "blink_workers": 2,
        "errors": 0,
        "conflicts": 0,
        "overloads": 0,
    }
    for field_name, expected_value in expected_fields.items():
        if row.get(field_name) != expected_value:
            raise ValueError(f"{path}: {field_name} expected {expected_value!r}, got {row.get(field_name)!r}")
    return row, metadata


def dodb_samples(filesystem, width):
    samples = []
    for filesystem_run in FILESYSTEM_RUNS[filesystem]:
        for repetition in range(1, 6):
            row, metadata = dodb_run(filesystem_run, width, repetition)
            duration_seconds = row["duration_ms"] / 1000
            cpu_seconds = row["cpu_utilization_percent_one_core"] * duration_seconds / 100
            samples.append(
                {
                    "tx_per_second": row["logical_tx_per_second"],
                    "p50_us": row["e2e_p50_us"],
                    "p95_us": row["e2e_p95_us"],
                    "p99_us": row["e2e_p99_us"],
                    "cpu_seconds": cpu_seconds,
                    "wal_sync_count": row["wal_syncs_delta"],
                    "wal_sync_time_ms": row["wal_sync_nanos_total"] / 1_000_000,
                    "average_group_size": row["avg_group_requests"],
                    "groups_per_second": row["groups"] / duration_seconds,
                    "physical_execution_ms": row["physical_execution_nanos"] / 1_000_000,
                    "device_bytes_written": metadata["written_bytes"],
                    "device_busy_ms": metadata["block_stats_delta"]["io_busy_ms"],
                    "errors": row["errors"],
                    "metadata": metadata,
                }
            )
    return samples


def fio_samples(filesystem, kind):
    samples = []
    for filesystem_run in FILESYSTEM_RUNS[filesystem]:
        for repetition in range(1, 6):
            job, metadata = read_fio_run(filesystem_run, kind, repetition)
            runtime_seconds = job["write"]["runtime"] / 1000
            sample = {
                "iops": job["write"]["iops"],
                "mb_per_second": job["write"]["bw_bytes"] / 1_000_000,
                "sync_mean_ms": fio_sync_metric(job, "mean") / 1_000_000,
                "sync_p50_ms": fio_sync_metric(job, "percentile")["50.000000"] / 1_000_000,
                "sync_p95_ms": fio_sync_metric(job, "percentile")["95.000000"] / 1_000_000,
                "sync_p99_ms": fio_sync_metric(job, "percentile")["99.000000"] / 1_000_000,
                "cpu_seconds": (job["usr_cpu"] + job["sys_cpu"]) * runtime_seconds / 100,
                "device_bytes_written": metadata["written_bytes"],
                "device_busy_ms": metadata["block_stats_delta"]["io_busy_ms"],
                "metadata": metadata,
            }
            if kind == "fio-rand":
                for percentile_name, percentile_key in (
                    ("write_p50_ms", "50.000000"),
                    ("write_p95_ms", "95.000000"),
                    ("write_p99_ms", "99.000000"),
                ):
                    sample[percentile_name] = fio_write_percentile(job, percentile_key) / 1_000_000
            samples.append(sample)
    return samples


def calculate_summary(values):
    mean_value = statistics.fmean(values)
    coefficient = statistics.stdev(values) / mean_value * 100 if len(values) > 1 and mean_value else 0.0
    return statistics.median(values), coefficient, mean_value, statistics.stdev(values) if len(values) > 1 else 0.0


def metric_cell(samples, metric_name, precision=2):
    median_value, coefficient, _, _ = calculate_summary([sample[metric_name] for sample in samples])
    return f"{median_value:,.{precision}f} ({coefficient:.1f}%)"


def filesystem_samples():
    values = {}
    for filesystem in FILESYSTEM_RUNS:
        values[filesystem] = {
            "fio-seq": fio_samples(filesystem, "fio-seq"),
            "fio-rand": fio_samples(filesystem, "fio-rand"),
            "dodb-width1": dodb_samples(filesystem, 1),
            "dodb-width16": dodb_samples(filesystem, 16),
        }
    return values


def summarize_rows(values, section_name, definitions):
    output = [f"## {section_name}", "", "| Metric | ext4 | XFS | Btrfs | ZFS |", "|---|---:|---:|---:|---:|"]
    for display_name, category, metric_name, unit, precision in definitions:
        row_values = []
        for filesystem in FILESYSTEM_RUNS:
            row_values.append(metric_cell(values[filesystem][category], metric_name, precision))
        output.append(f"| {display_name} ({unit}) | " + " | ".join(row_values) + " |")
    output.append("")
    return output


def pooled_variation(samples_a, samples_b):
    _, _, _, standard_deviation_a = calculate_summary(samples_a)
    _, _, _, standard_deviation_b = calculate_summary(samples_b)
    return math.sqrt((standard_deviation_a**2 + standard_deviation_b**2) / 2)


def compare_winners(values, metric_definition):
    display_name, category, metric_name, unit, precision, direction = metric_definition
    medians = {
        filesystem: calculate_summary([sample[metric_name] for sample in values[filesystem][category]])[0]
        for filesystem in FILESYSTEM_RUNS
    }
    ordered = sorted(medians, key=medians.get, reverse=(direction == "higher"))
    winner, runner_up = ordered[:2]
    winner_samples = values[winner][category]
    runner_samples = values[runner_up][category]
    variation = pooled_variation(
        [sample[metric_name] for sample in winner_samples],
        [sample[metric_name] for sample in runner_samples],
    )
    gap = abs(medians[winner] - medians[runner_up])
    larger_than_variation = gap > variation
    return {
        "display_name": display_name,
        "winner": winner,
        "runner_up": runner_up,
        "medians": medians,
        "gap": gap,
        "variation": variation,
        "larger_than_variation": larger_than_variation,
    }


def build_winner_report(values):
    definitions = [
        ("fio fdatasync IOPS", "fio-seq", "iops", "IOPS", 1, "higher"),
        ("fio fdatasync throughput", "fio-seq", "mb_per_second", "MB/s", 2, "higher"),
        ("fio fdatasync mean", "fio-seq", "sync_mean_ms", "ms", 3, "lower"),
        ("fio fdatasync p50", "fio-seq", "sync_p50_ms", "ms", 3, "lower"),
        ("fio fdatasync p95", "fio-seq", "sync_p95_ms", "ms", 3, "lower"),
        ("fio fdatasync p99", "fio-seq", "sync_p99_ms", "ms", 3, "lower"),
        ("4 KiB overwrite IOPS", "fio-rand", "iops", "IOPS", 1, "higher"),
        ("4 KiB overwrite write p50", "fio-rand", "write_p50_ms", "ms", 3, "lower"),
        ("4 KiB overwrite write p95", "fio-rand", "write_p95_ms", "ms", 3, "lower"),
        ("4 KiB overwrite write p99", "fio-rand", "write_p99_ms", "ms", 3, "lower"),
        ("dodb width1 throughput", "dodb-width1", "tx_per_second", "tx/s", 1, "higher"),
        ("dodb width1 p50", "dodb-width1", "p50_us", "us", 1, "lower"),
        ("dodb width1 p95", "dodb-width1", "p95_us", "us", 1, "lower"),
        ("dodb width1 p99", "dodb-width1", "p99_us", "us", 1, "lower"),
        ("dodb width16 throughput", "dodb-width16", "tx_per_second", "tx/s", 1, "higher"),
        ("dodb width16 p50", "dodb-width16", "p50_us", "us", 1, "lower"),
        ("dodb width16 p95", "dodb-width16", "p95_us", "us", 1, "lower"),
        ("dodb width16 p99", "dodb-width16", "p99_us", "us", 1, "lower"),
    ]
    results = [compare_winners(values, definition) for definition in definitions]
    output = [
        "## Fastest filesystem and repeat variation",
        "",
        "Fastest means highest IOPS/throughput or lowest latency. For the variation check, the absolute median gap between the top two filesystems is compared with the root mean square of their sample standard deviations. This is a descriptive comparison with five repetitions per stage, not a significance test.",
        "",
        "| Metric | Fastest median | Next median | Gap | Combined within-filesystem SD | Gap exceeds SD |",
        "|---|---:|---:|---:|---:|:---:|",
    ]
    for result, definition in zip(results, definitions):
        _, _, _, unit, precision, _ = definition
        winner_median = result["medians"][result["winner"]]
        runner_median = result["medians"][result["runner_up"]]
        output.append(
            f"| {result['display_name']} | {result['winner']} {winner_median:,.{precision}f} {unit} | {result['runner_up']} {runner_median:,.{precision}f} {unit} | {result['gap']:,.{precision}f} {unit} | {result['variation']:,.{precision}f} {unit} | {'Yes' if result['larger_than_variation'] else 'No'} |"
        )
    output.append("")
    slowest_counts = {filesystem: 0 for filesystem in FILESYSTEM_RUNS}
    slowest_metrics = {}
    for result, definition in zip(results, definitions):
        direction = definition[5]
        if direction == "higher":
            slowest = min(result["medians"], key=result["medians"].get)
        else:
            slowest = max(result["medians"], key=result["medians"].get)
        slowest_counts[slowest] += 1
        slowest_metrics.setdefault(slowest, []).append(result["display_name"])
    slowest_description = ", ".join(
        f"{filesystem} {count}/{len(results)}"
        for filesystem, count in slowest_counts.items()
        if count
    )
    slowest_detail = "; ".join(
        f"{filesystem}: {', '.join(metric_names)}"
        for filesystem, metric_names in slowest_metrics.items()
    )
    output.append(
        f"By median, the slowest filesystem across these direct throughput and latency metrics is {slowest_description}. The metric breakdown is {slowest_detail}."
    )
    output.append("")
    output.append("The `Btrfs` column in the aggregate tables pools five Btrfs-pre and five Btrfs-post repetitions. The separate drift table shows the change between those stages.")
    output.append("")
    return output, definitions, results


def build_drift_report(values):
    metrics = [
        ("fio sequential IOPS", "fio-seq", "iops", "IOPS", 1),
        ("fio sequential fdatasync mean", "fio-seq", "sync_mean_ms", "ms", 3),
        ("fio sequential fdatasync p50", "fio-seq", "sync_p50_ms", "ms", 3),
        ("fio sequential fdatasync p99", "fio-seq", "sync_p99_ms", "ms", 3),
        ("4 KiB overwrite IOPS", "fio-rand", "iops", "IOPS", 1),
        ("dodb width1 throughput", "dodb-width1", "tx_per_second", "tx/s", 1),
        ("dodb width1 p99", "dodb-width1", "p99_us", "us", 1),
        ("dodb width16 throughput", "dodb-width16", "tx_per_second", "tx/s", 1),
        ("dodb width16 p99", "dodb-width16", "p99_us", "us", 1),
    ]
    output = [
        "## Btrfs pre/post drift",
        "",
        "Percent change is `(Btrfs-post median / Btrfs-pre median - 1) × 100`. Positive changes mean a higher metric; for latency, a positive change is slower.",
        "",
        "| Metric | Btrfs-pre median (CV) | Btrfs-post median (CV) | Post vs pre | Gap exceeds pooled SD |",
        "|---|---:|---:|---:|:---:|",
    ]
    for display_name, category, metric_name, unit, precision in metrics:
        pre_samples = values["Btrfs"][category][:5]
        post_samples = values["Btrfs"][category][5:]
        pre_median, _, _, _ = calculate_summary([sample[metric_name] for sample in pre_samples])
        post_median, _, _, _ = calculate_summary([sample[metric_name] for sample in post_samples])
        pre_cell = metric_cell(pre_samples, metric_name, precision)
        post_cell = metric_cell(post_samples, metric_name, precision)
        variation = pooled_variation(
            [sample[metric_name] for sample in pre_samples],
            [sample[metric_name] for sample in post_samples],
        )
        output.append(
            f"| {display_name} ({unit}) | {pre_cell} | {post_cell} | {(post_median / pre_median - 1) * 100:+.1f}% | {'Yes' if abs(post_median - pre_median) > variation else 'No'} |"
        )
    output.append("")
    return output


def build_run_order():
    stages = [
        ("Btrfs-pre", "Btrfs"),
        ("ext4", "ext4"),
        ("xfs", "XFS"),
        ("zfs", "ZFS"),
        ("Btrfs-post", "Btrfs"),
    ]
    rows = []
    for stage_name, filesystem_name in stages:
        metadata_paths = sorted(RAW_DIRECTORY.glob(f"{stage_name.lower()}-*.meta.json"))
        metadata = [read_json(path) for path in metadata_paths]
        if not metadata:
            raise ValueError(f"no raw run metadata for {stage_name}")
        rows.append(
            {
                "order": len(rows) + 1,
                "stage": stage_name,
                "filesystem": filesystem_name,
                "started_at": min(item["started_at"] for item in metadata),
                "ended_at": max(item["ended_at"] for item in metadata),
                "run_count": len(metadata),
            }
        )
    with (RESULT_DIRECTORY / "run-order.csv").open("w", newline="") as csv_file:
        field_names = ["order", "stage", "filesystem", "started_at", "ended_at", "run_count"]
        writer = csv.DictWriter(csv_file, fieldnames=field_names)
        writer.writeheader()
        writer.writerows(rows)


def write_csv_summary(values):
    output_path = RESULT_DIRECTORY / "summary.csv"
    with output_path.open("w", newline="") as csv_file:
        field_names = ["filesystem", "metric", "sample_count", "median", "coefficient_of_variation_percent", "mean", "sample_standard_deviation"]
        writer = csv.DictWriter(csv_file, fieldnames=field_names)
        writer.writeheader()
        for filesystem in FILESYSTEM_RUNS:
            categories = values[filesystem]
            for category, samples in categories.items():
                metric_names = [name for name, value in samples[0].items() if name != "metadata"]
                for metric_name in metric_names:
                    metric_values = [sample[metric_name] for sample in samples]
                    median_value, coefficient, mean_value, standard_deviation = calculate_summary(metric_values)
                    writer.writerow(
                        {
                            "filesystem": filesystem,
                            "metric": f"{category}.{metric_name}",
                            "sample_count": len(metric_values),
                            "median": median_value,
                            "coefficient_of_variation_percent": coefficient,
                            "mean": mean_value,
                            "sample_standard_deviation": standard_deviation,
                        }
                    )


def main():
    values = filesystem_samples()
    output = [
        "# Filesystem performance summary",
        "",
        "Each cell reports the median and sample coefficient of variation (CV = sample standard deviation / mean × 100%). fio and dodb use five repetitions per filesystem; aggregate Btrfs cells pool Btrfs-pre and Btrfs-post (ten repetitions).",
        "",
    ]
    requested_metrics = [
        ("fio fdatasync IOPS", "fio-seq", "iops", "IOPS", 1),
        ("fio fdatasync p50", "fio-seq", "sync_p50_ms", "ms", 3),
        ("fio fdatasync p99", "fio-seq", "sync_p99_ms", "ms", 3),
        ("4 KiB overwrite IOPS", "fio-rand", "iops", "IOPS", 1),
        ("dodb width1 throughput", "dodb-width1", "tx_per_second", "tx/s", 1),
        ("dodb width1 p99", "dodb-width1", "p99_us", "us", 1),
        ("dodb width16 throughput", "dodb-width16", "tx_per_second", "tx/s", 1),
        ("dodb width16 p99", "dodb-width16", "p99_us", "us", 1),
    ]
    output.extend(summarize_rows(values, "Requested comparison table", requested_metrics))
    detailed_metrics = [
        ("IOPS", "iops", "IOPS", 1),
        ("Throughput", "mb_per_second", "MB/s", 2),
        ("fdatasync mean", "sync_mean_ms", "ms", 3),
        ("fdatasync p50", "sync_p50_ms", "ms", 3),
        ("fdatasync p95", "sync_p95_ms", "ms", 3),
        ("fdatasync p99", "sync_p99_ms", "ms", 3),
        ("fio process CPU", "cpu_seconds", "CPU-s", 3),
        ("device bytes written", "device_bytes_written", "bytes", 0),
        ("device busy time", "device_busy_ms", "ms", 0),
    ]
    output.extend(summarize_rows(values, "fio sequential durable write", [(name, "fio-seq", key, unit, precision) for name, key, unit, precision in detailed_metrics]))
    random_metrics = [
        ("IOPS", "iops", "IOPS", 1),
        ("Throughput", "mb_per_second", "MB/s", 2),
        ("write p50", "write_p50_ms", "ms", 3),
        ("write p95", "write_p95_ms", "ms", 3),
        ("write p99", "write_p99_ms", "ms", 3),
        ("fdatasync mean", "sync_mean_ms", "ms", 3),
        ("fdatasync p50", "sync_p50_ms", "ms", 3),
        ("fdatasync p95", "sync_p95_ms", "ms", 3),
        ("fdatasync p99", "sync_p99_ms", "ms", 3),
        ("fio process CPU", "cpu_seconds", "CPU-s", 3),
        ("device bytes written", "device_bytes_written", "bytes", 0),
        ("device busy time", "device_busy_ms", "ms", 0),
    ]
    output.extend(summarize_rows(values, "fio 4 KiB random overwrite", [(name, "fio-rand", key, unit, precision) for name, key, unit, precision in random_metrics]))
    dodb_metrics = [
        ("Throughput", "tx_per_second", "tx/s", 1),
        ("p50", "p50_us", "us", 1),
        ("p95", "p95_us", "us", 1),
        ("p99", "p99_us", "us", 1),
        ("process CPU", "cpu_seconds", "CPU-s", 3),
        ("WAL sync count", "wal_sync_count", "count", 0),
        ("WAL sync time", "wal_sync_time_ms", "ms", 2),
        ("average group size", "average_group_size", "tx/group", 2),
        ("groups per second", "groups_per_second", "groups/s", 2),
        ("physical execution time", "physical_execution_ms", "ms", 2),
        ("device bytes written", "device_bytes_written", "bytes", 0),
        ("device busy time", "device_busy_ms", "ms", 0),
        ("errors", "errors", "count", 0),
    ]
    for width in (1, 16):
        category = f"dodb-width{width}"
        output.extend(summarize_rows(values, f"dodb width {width}", [(name, category, key, unit, precision) for name, key, unit, precision in dodb_metrics]))
    winner_report, winner_definitions, winner_results = build_winner_report(values)
    output.extend(winner_report)
    output.extend(build_drift_report(values))
    (RESULT_DIRECTORY / "summary-tables.md").write_text("\n".join(output))
    write_csv_summary(values)
    build_run_order()
    fastest_metrics = []
    slowest_counts = {filesystem: 0 for filesystem in FILESYSTEM_RUNS}
    slowest_metrics = []
    for result, definition in zip(winner_results, winner_definitions):
        _, _, _, unit, _, direction = definition
        if direction == "higher":
            slowest = min(result["medians"], key=result["medians"].get)
        else:
            slowest = max(result["medians"], key=result["medians"].get)
        slowest_counts[slowest] += 1
        fastest_metrics.append(
            {
                "metric": result["display_name"],
                "fastest_filesystem": result["winner"],
                "fastest_median": result["medians"][result["winner"]],
                "runner_up_filesystem": result["runner_up"],
                "runner_up_median": result["medians"][result["runner_up"]],
                "median_gap": result["gap"],
                "combined_within_filesystem_sd": result["variation"],
                "gap_exceeds_sd": result["larger_than_variation"],
                "unit": unit,
            }
        )
        slowest_metrics.append({"metric": result["display_name"], "slowest_filesystem": slowest})
    print(json.dumps({"fastest_by_metric": fastest_metrics, "slowest_metric_counts": slowest_counts, "slowest_by_metric": slowest_metrics}, indent=2))


if __name__ == "__main__":
    main()
