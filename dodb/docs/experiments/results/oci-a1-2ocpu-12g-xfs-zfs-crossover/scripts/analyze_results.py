import csv
import html
import json
import math
import statistics
from pathlib import Path


RESULT_DIRECTORY = Path(__file__).resolve().parents[1]
STAGES = ("XFS-pre", "ZFS", "XFS-post")
FILESYSTEMS = ("XFS-pre", "ZFS", "XFS-post")
INITIAL_PAYLOAD_SIZES = (4096, 16384, 65536, 262144, 1048576)
TRANSACTION_WIDTHS = (1, 2, 4, 8, 16)
EXPECTED_COMMIT = "08e82ec484bd62aeb659471a39f317b86cfb488f"
EXPECTED_BINARY_SHA256 = "3756eac0557bbe0d87ba3b2ea46ed088df46aa7e8d79fae56f570d8eb52469b6"
COLORS = {"XFS-pre": "#2563eb", "ZFS": "#d97706", "XFS-post": "#059669"}


def read_json(path):
    return json.loads(path.read_text())


def summarize(values):
    mean_value = statistics.fmean(values)
    standard_deviation = statistics.stdev(values) if len(values) > 1 else 0.0
    coefficient = standard_deviation / mean_value * 100 if mean_value else 0.0
    return {
        "median": statistics.median(values),
        "mean": mean_value,
        "sample_sd": standard_deviation,
        "cv_percent": coefficient,
        "minimum": min(values),
        "maximum": max(values),
    }


def metadata_path(directory, stem):
    return directory / f"{stem}.meta.json"


def read_fio_samples(stage, payload_size):
    directory = RESULT_DIRECTORY / "fio"
    stage_prefix = stage.lower()
    samples = []
    for repetition in range(1, 6):
        stem = f"{stage_prefix}-payload{payload_size}-rep{repetition}"
        report = read_json(directory / f"{stem}.json")
        job = report["jobs"][0]
        metadata = read_json(metadata_path(directory, stem))
        if metadata["exit_code"] != 0 or job["error"] != 0:
            raise ValueError(f"fio failed: {stem}")
        if job["write"]["runtime"] < 19900:
            raise ValueError(f"fio runtime below 19.9 seconds: {stem}")
        if metadata["source_commit"] != EXPECTED_COMMIT or metadata["binary_sha256"] != EXPECTED_BINARY_SHA256:
            raise ValueError(f"fio provenance mismatch: {stem}")
        sync = job["sync"]["lat_ns"]
        runtime_seconds = job["write"]["runtime"] / 1000
        samples.append(
            {
                "stage": stage,
                "payload_size_bytes": payload_size,
                "repetition": repetition,
                "operations_per_second": job["write"]["iops"],
                "mb_per_second": job["write"]["bw_bytes"] / 1_000_000,
                "sync_mean_ms": sync["mean"] / 1_000_000,
                "sync_p50_ms": sync["percentile"]["50.000000"] / 1_000_000,
                "sync_p95_ms": sync["percentile"]["95.000000"] / 1_000_000,
                "sync_p99_ms": sync["percentile"]["99.000000"] / 1_000_000,
                "cpu_seconds": (job["usr_cpu"] + job["sys_cpu"]) * runtime_seconds / 100,
                "device_bytes_written": metadata["written_bytes"],
                "device_busy_ms": metadata["device_busy_ms"],
                "metadata": metadata,
            }
        )
    return samples


def read_dodb_samples(stage, width):
    directory = RESULT_DIRECTORY / "dodb"
    stage_prefix = stage.lower()
    samples = []
    for repetition in range(1, 6):
        stem = f"{stage_prefix}-width{width}-rep{repetition}"
        path = directory / f"{stem}.jsonl"
        rows = [json.loads(line) for line in path.read_text().splitlines() if line]
        if len(rows) != 1:
            raise ValueError(f"expected one dodb row in {path}, got {len(rows)}")
        row = rows[0]
        metadata = read_json(metadata_path(directory, stem))
        expected_values = {
            "git_commit": EXPECTED_COMMIT,
            "engine": "parallel-blink",
            "sync_mode": "real",
            "writers": 64,
            "working_set": 100000,
            "key_size": 16,
            "value_size": 64,
            "transaction_width": width,
            "transaction_mode": "unconditional",
            "errors": 0,
            "conflicts": 0,
            "overloads": 0,
        }
        for field_name, expected_value in expected_values.items():
            if row.get(field_name) != expected_value:
                raise ValueError(f"{path}: {field_name} expected {expected_value!r}, got {row.get(field_name)!r}")
        if metadata["exit_code"] != 0:
            raise ValueError(f"dodb command failed: {path}")
        if metadata["source_commit"] != EXPECTED_COMMIT or metadata["binary_sha256"] != EXPECTED_BINARY_SHA256:
            raise ValueError(f"dodb provenance mismatch: {path}")
        duration_seconds = row["duration_ms"] / 1000
        sync_count = row["wal_syncs_delta"]
        if sync_count <= 0:
            raise ValueError(f"missing WAL syncs: {path}")
        samples.append(
            {
                "stage": stage,
                "transaction_width": width,
                "repetition": repetition,
                "tx_per_second": row["logical_tx_per_second"],
                "p50_us": row["e2e_p50_us"],
                "p95_us": row["e2e_p95_us"],
                "p99_us": row["e2e_p99_us"],
                "cpu_seconds": row["cpu_utilization_percent_one_core"] * duration_seconds / 100,
                "wal_bytes": row["wal_bytes_delta"],
                "wal_sync_count": sync_count,
                "wal_sync_total_ms": row["wal_sync_nanos_total"] / 1_000_000,
                "wal_bytes_per_sync": row["wal_bytes_delta"] / sync_count,
                "wal_sync_mean_ms": row["wal_sync_nanos_total"] / sync_count / 1_000_000,
                "successful_transactions": row["successful_transactions"],
                "transactions_per_sync": row["successful_transactions"] / sync_count,
                "average_group_size": row["avg_group_requests"],
                "groups_per_second": row["groups"] / duration_seconds,
                "physical_execution_ms": row["physical_execution_nanos"] / 1_000_000,
                "planning_ms": row["planning_nanos"] / 1_000_000,
                "validation_ms": row["validation_nanos_total"] / 1_000_000,
                "publication_ms": row["publication_nanos_total"] / 1_000_000,
                "device_bytes_written": metadata["written_bytes"],
                "device_busy_ms": metadata["device_busy_ms"],
                "errors": row["errors"],
                "metadata": metadata,
            }
        )
    return samples


def available_payload_sizes():
    payload_sizes = set()
    for path in (RESULT_DIRECTORY / "fio").glob("*-payload*-rep1.json"):
        payload_sizes.add(int(path.stem.rsplit("payload", 1)[1].split("-rep", 1)[0]))
    return tuple(sorted(payload_sizes))


def load_all():
    payload_sizes = available_payload_sizes()
    fio = {stage: {size: read_fio_samples(stage, size) for size in payload_sizes} for stage in STAGES}
    dodb = {stage: {width: read_dodb_samples(stage, width) for width in TRANSACTION_WIDTHS} for stage in STAGES}
    return payload_sizes, fio, dodb


def write_summary_csv(payload_sizes, fio, dodb):
    rows = []
    for stage in STAGES:
        for payload_size in payload_sizes:
            for metric in (
                "operations_per_second",
                "mb_per_second",
                "sync_mean_ms",
                "sync_p50_ms",
                "sync_p95_ms",
                "sync_p99_ms",
                "cpu_seconds",
                "device_bytes_written",
                "device_busy_ms",
            ):
                values = [sample[metric] for sample in fio[stage][payload_size]]
                stats = summarize(values)
                rows.append({"stage": stage, "workload": "fio", "point": payload_size, "metric": metric, **stats})
        for width in TRANSACTION_WIDTHS:
            for metric in (
                "tx_per_second",
                "p50_us",
                "p95_us",
                "p99_us",
                "cpu_seconds",
                "wal_bytes",
                "wal_sync_count",
                "wal_sync_total_ms",
                "wal_bytes_per_sync",
                "wal_sync_mean_ms",
                "transactions_per_sync",
                "average_group_size",
                "groups_per_second",
                "physical_execution_ms",
                "planning_ms",
                "validation_ms",
                "publication_ms",
                "device_bytes_written",
                "device_busy_ms",
                "errors",
            ):
                values = [sample[metric] for sample in dodb[stage][width]]
                stats = summarize(values)
                rows.append({"stage": stage, "workload": "dodb", "point": width, "metric": metric, **stats})
    path = RESULT_DIRECTORY / "summary.csv"
    fieldnames = ("stage", "workload", "point", "metric", "median", "mean", "sample_sd", "cv_percent", "minimum", "maximum")
    with path.open("w", newline="") as file_handle:
        writer = csv.DictWriter(file_handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def write_run_order():
    rows = []
    for stage in STAGES:
        metadata_paths = list((RESULT_DIRECTORY / "fio").glob(f"{stage.lower()}-*.meta.json"))
        fio_metadata = [read_json(path) for path in metadata_paths]
        dodb_paths = list((RESULT_DIRECTORY / "dodb").glob(f"{stage.lower()}-*.meta.json"))
        dodb_metadata = [read_json(path) for path in dodb_paths]
        all_metadata = fio_metadata + dodb_metadata
        rows.append(
            {
                "order": len(rows) + 1,
                "stage": stage,
                "filesystem": "XFS" if stage.startswith("XFS") else "ZFS",
                "started_at": min(item["started_at"] for item in all_metadata),
                "ended_at": max(item["ended_at"] for item in all_metadata),
                "fio_runs": len(fio_metadata),
                "dodb_runs": len(dodb_metadata),
            }
        )
    with (RESULT_DIRECTORY / "run-order.csv").open("w", newline="") as file_handle:
        writer = csv.DictWriter(file_handle, fieldnames=("order", "stage", "filesystem", "started_at", "ended_at", "fio_runs", "dodb_runs"))
        writer.writeheader()
        writer.writerows(rows)


def cell(samples, metric, digits=2):
    values = [sample[metric] for sample in samples]
    stats = summarize(values)
    return f"{stats['median']:,.{digits}f} ({stats['cv_percent']:.1f}%)"


def metric_table(title, point_label, points, values, metric_definitions, digits=2):
    output = [f"## {title}", "", f"| {point_label} | XFS-pre | ZFS | XFS-post |", "|---:|---:|---:|---:|"]
    for point, metric, label, precision in metric_definitions:
        stage_cells = [cell(values[stage][point], metric, precision) for stage in FILESYSTEMS]
        output.append(f"| {label.format(point=point)} | " + " | ".join(stage_cells) + " |")
    output.append("")
    output.append("Cells show median (sample CV%). Each stage has five repetitions; XFS-pre and XFS-post are not pooled.")
    output.append("")
    return output


def write_chart(output_path, title, x_label, y_label, points, series, x_tick_labels=None):
    width = 920
    height = 540
    left = 92
    right = 230
    top = 74
    bottom = 98
    plot_width = width - left - right
    plot_height = height - top - bottom
    x_transforms = [math.log2(point) for point in points]
    x_min = min(x_transforms)
    x_max = max(x_transforms)
    if x_max == x_min:
        x_max = x_min + 1
    all_limits = []
    for values in series.values():
        for median_value, standard_deviation in values:
            all_limits.append(max(0.0, median_value - standard_deviation))
            all_limits.append(median_value + standard_deviation)
    y_min = min(all_limits) if all_limits else 0.0
    y_max = max(all_limits) if all_limits else 1.0
    span = y_max - y_min
    if span == 0:
        span = abs(y_max) or 1.0
    y_min = max(0.0, y_min - span * 0.08)
    y_max += span * 0.12

    def x_coordinate(point):
        return left + (math.log2(point) - x_min) / (x_max - x_min) * plot_width

    def y_coordinate(value):
        return top + (y_max - value) / (y_max - y_min) * plot_height

    pieces = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<rect width="100%" height="100%" fill="#ffffff"/>',
        f'<text x="{width / 2}" y="34" text-anchor="middle" font-family="sans-serif" font-size="21" font-weight="600">{html.escape(title)}</text>',
    ]
    for tick_index in range(6):
        value = y_min + (y_max - y_min) * tick_index / 5
        y_position = y_coordinate(value)
        pieces.append(f'<line x1="{left}" y1="{y_position:.2f}" x2="{left + plot_width}" y2="{y_position:.2f}" stroke="#e5e7eb"/>')
        pieces.append(f'<text x="{left - 12}" y="{y_position + 5:.2f}" text-anchor="end" font-family="sans-serif" font-size="12" fill="#4b5563">{value:,.2f}</text>')
    pieces.append(f'<line x1="{left}" y1="{top}" x2="{left}" y2="{top + plot_height}" stroke="#374151"/>')
    pieces.append(f'<line x1="{left}" y1="{top + plot_height}" x2="{left + plot_width}" y2="{top + plot_height}" stroke="#374151"/>')
    for point_index, point in enumerate(points):
        x_position = x_coordinate(point)
        label = x_tick_labels[point_index] if x_tick_labels else str(point)
        pieces.append(f'<line x1="{x_position:.2f}" y1="{top + plot_height}" x2="{x_position:.2f}" y2="{top + plot_height + 6}" stroke="#374151"/>')
        pieces.append(f'<text x="{x_position:.2f}" y="{top + plot_height + 24}" text-anchor="middle" font-family="sans-serif" font-size="12" fill="#4b5563">{html.escape(label)}</text>')
    pieces.append(f'<text x="{left + plot_width / 2}" y="{height - 34}" text-anchor="middle" font-family="sans-serif" font-size="14">{html.escape(x_label)}</text>')
    pieces.append(f'<text x="24" y="{top + plot_height / 2}" text-anchor="middle" transform="rotate(-90 24 {top + plot_height / 2})" font-family="sans-serif" font-size="14">{html.escape(y_label)}</text>')
    for series_index, (label, values) in enumerate(series.items()):
        color = COLORS[label]
        coordinates = [(x_coordinate(point), y_coordinate(median)) for point, (median, _) in zip(points, values)]
        path_data = " ".join(("M" if index == 0 else "L") + f" {x_position:.2f} {y_position:.2f}" for index, (x_position, y_position) in enumerate(coordinates))
        pieces.append(f'<path d="{path_data}" fill="none" stroke="{color}" stroke-width="2.5"/>')
        for point, (median_value, standard_deviation), (x_position, y_position) in zip(points, values, coordinates):
            lower_y = y_coordinate(max(0.0, median_value - standard_deviation))
            upper_y = y_coordinate(median_value + standard_deviation)
            pieces.append(f'<line x1="{x_position:.2f}" y1="{lower_y:.2f}" x2="{x_position:.2f}" y2="{upper_y:.2f}" stroke="{color}" stroke-width="1.2"/>')
            pieces.append(f'<line x1="{x_position - 4:.2f}" y1="{lower_y:.2f}" x2="{x_position + 4:.2f}" y2="{lower_y:.2f}" stroke="{color}"/>')
            pieces.append(f'<line x1="{x_position - 4:.2f}" y1="{upper_y:.2f}" x2="{x_position + 4:.2f}" y2="{upper_y:.2f}" stroke="{color}"/>')
            pieces.append(f'<circle cx="{x_position:.2f}" cy="{y_position:.2f}" r="4" fill="{color}"/>')
        legend_y = top + 28 + series_index * 24
        legend_x = left + plot_width + 20
        pieces.append(f'<line x1="{legend_x}" y1="{legend_y}" x2="{legend_x + 24}" y2="{legend_y}" stroke="{color}" stroke-width="2.5"/>')
        pieces.append(f'<text x="{legend_x + 32}" y="{legend_y + 5}" font-family="sans-serif" font-size="12" fill="#374151">{html.escape(label)} median ± SD</text>')
    pieces.append("</svg>")
    output_path.write_text("\n".join(pieces) + "\n")


def create_charts(payload_sizes, fio, dodb):
    fio_charts = (
        ("operations_per_second", "Operations per second", "fio operations per second"),
        ("sync_mean_ms", "fdatasync mean latency", "milliseconds"),
        ("sync_p95_ms", "fdatasync p95 latency", "milliseconds"),
        ("sync_p99_ms", "fdatasync p99 latency", "milliseconds"),
    )
    for metric, title, y_label in fio_charts:
        series = {}
        for stage in FILESYSTEMS:
            series[stage] = [
                (summarize([sample[metric] for sample in fio[stage][payload_size]])["median"], summarize([sample[metric] for sample in fio[stage][payload_size]])["sample_sd"])
                for payload_size in payload_sizes
            ]
        write_chart(RESULT_DIRECTORY / "fio" / f"payload-{metric}.svg", title, "sync payload size (bytes, log2 scale)", y_label, payload_sizes, series)
    dodb_charts = (
        ("tx_per_second", "dodb throughput by transaction width", "transactions per second"),
        ("p50_us", "dodb p50 latency by transaction width", "microseconds"),
        ("p95_us", "dodb p95 latency by transaction width", "microseconds"),
        ("p99_us", "dodb p99 latency by transaction width", "microseconds"),
        ("wal_sync_mean_ms", "WAL sync mean by transaction width", "milliseconds"),
        ("wal_bytes_per_sync", "WAL bytes per sync by transaction width", "bytes per sync"),
        ("groups_per_second", "dodb groups per second by transaction width", "groups per second"),
    )
    for metric, title, y_label in dodb_charts:
        series = {}
        for stage in FILESYSTEMS:
            series[stage] = [
                (summarize([sample[metric] for sample in dodb[stage][width]])["median"], summarize([sample[metric] for sample in dodb[stage][width]])["sample_sd"])
                for width in TRANSACTION_WIDTHS
            ]
        write_chart(RESULT_DIRECTORY / "dodb" / f"width-{metric}.svg", title, "transaction width (log2 scale)", y_label, TRANSACTION_WIDTHS, series)


def find_crossovers(points, left_values, right_values):
    crossings = []
    for index in range(1, len(points)):
        previous_difference = left_values[index - 1] - right_values[index - 1]
        current_difference = left_values[index] - right_values[index]
        if previous_difference * current_difference < 0:
            crossings.append({"lower_point": points[index - 1], "upper_point": points[index]})
    return crossings


def make_crossover_report(payload_sizes, fio, dodb):
    payload_report = {}
    for xfs_stage in ("XFS-pre", "XFS-post"):
        xfs_medians = [summarize([sample["operations_per_second"] for sample in fio[xfs_stage][size]])["median"] for size in payload_sizes]
        zfs_medians = [summarize([sample["operations_per_second"] for sample in fio["ZFS"][size]])["median"] for size in payload_sizes]
        payload_report[xfs_stage] = {
            "payload_bytes": list(payload_sizes),
            "xfs_operations_per_second": xfs_medians,
            "zfs_operations_per_second": zfs_medians,
            "crossing_intervals": find_crossovers(payload_sizes, xfs_medians, zfs_medians),
        }
    width_report = {}
    for xfs_stage in ("XFS-pre", "XFS-post"):
        xfs_medians = [summarize([sample["tx_per_second"] for sample in dodb[xfs_stage][width]])["median"] for width in TRANSACTION_WIDTHS]
        zfs_medians = [summarize([sample["tx_per_second"] for sample in dodb["ZFS"][width]])["median"] for width in TRANSACTION_WIDTHS]
        width_report[xfs_stage] = {
            "transaction_widths": list(TRANSACTION_WIDTHS),
            "xfs_tx_per_second": xfs_medians,
            "zfs_tx_per_second": zfs_medians,
            "crossing_intervals": find_crossovers(TRANSACTION_WIDTHS, xfs_medians, zfs_medians),
        }
    report = {"fio_operations_per_second": payload_report, "dodb_tx_per_second": width_report}
    (RESULT_DIRECTORY / "crossover-analysis.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def drift_rows(payload_sizes, fio, dodb):
    output = [
        "## XFS-pre and XFS-post drift",
        "",
        "Each value is the XFS-post median divided by the XFS-pre median. Positive percentages mean an increase in the metric; for latency metrics, that is slower.",
        "",
        "| Workload | Point | Metric | XFS-pre median | XFS-post median | Post / pre |",
        "|---|---:|---|---:|---:|---:|",
    ]
    for payload_size in payload_sizes:
        for metric, label in (("operations_per_second", "operations/s"), ("sync_mean_ms", "fdatasync mean ms"), ("sync_p50_ms", "fdatasync p50 ms"), ("sync_p95_ms", "fdatasync p95 ms"), ("sync_p99_ms", "fdatasync p99 ms")):
            pre = summarize([sample[metric] for sample in fio["XFS-pre"][payload_size]])["median"]
            post = summarize([sample[metric] for sample in fio["XFS-post"][payload_size]])["median"]
            output.append(f"| fio | {payload_size:,} B | {label} | {pre:,.4f} | {post:,.4f} | {post / pre:.4f} ({(post / pre - 1) * 100:+.1f}%) |")
    for width in TRANSACTION_WIDTHS:
        for metric, label in (("tx_per_second", "tx/s"), ("p50_us", "p50 us"), ("p95_us", "p95 us"), ("p99_us", "p99 us"), ("wal_sync_mean_ms", "WAL sync mean ms")):
            pre = summarize([sample[metric] for sample in dodb["XFS-pre"][width]])["median"]
            post = summarize([sample[metric] for sample in dodb["XFS-post"][width]])["median"]
            output.append(f"| dodb | {width} | {label} | {pre:,.4f} | {post:,.4f} | {post / pre:.4f} ({(post / pre - 1) * 100:+.1f}%) |")
    output.append("")
    return output


def variation_comparison_rows(payload_sizes, fio, dodb):
    output = [
        "## Median gaps and repeat variation",
        "",
        "This descriptive check compares the absolute gap between stage medians with the root mean square of the two stages' sample standard deviations. It is not a significance test.",
        "",
        "| Metric | Point | XFS stage median | ZFS median | Gap | Combined within-stage SD | Gap exceeds SD |",
        "|---|---:|---:|---:|---:|---:|:---:|",
    ]
    comparisons = []
    for payload_size in payload_sizes:
        for xfs_stage in ("XFS-pre", "XFS-post"):
            comparisons.append(("fio operations/s", f"{payload_size:,} B", xfs_stage, fio[xfs_stage][payload_size], fio["ZFS"][payload_size], "operations_per_second", "higher", 1))
    for width in TRANSACTION_WIDTHS:
        for xfs_stage in ("XFS-pre", "XFS-post"):
            comparisons.append(("dodb tx/s", str(width), xfs_stage, dodb[xfs_stage][width], dodb["ZFS"][width], "tx_per_second", "higher", 1))
            comparisons.append(("dodb WAL sync mean ms", str(width), xfs_stage, dodb[xfs_stage][width], dodb["ZFS"][width], "wal_sync_mean_ms", "lower", 3))
    for metric_label, point, xfs_stage, xfs_samples, zfs_samples, metric, direction, digits in comparisons:
        xfs_values = [sample[metric] for sample in xfs_samples]
        zfs_values = [sample[metric] for sample in zfs_samples]
        xfs_stats = summarize(xfs_values)
        zfs_stats = summarize(zfs_values)
        combined_sd = math.sqrt((xfs_stats["sample_sd"] ** 2 + zfs_stats["sample_sd"] ** 2) / 2)
        median_gap = abs(xfs_stats["median"] - zfs_stats["median"])
        output.append(
            f"| {metric_label} | {point} | {xfs_stage} {xfs_stats['median']:,.{digits}f} | {zfs_stats['median']:,.{digits}f} | {median_gap:,.{digits}f} | {combined_sd:,.{digits}f} | {'Yes' if median_gap > combined_sd else 'No'} |"
        )
    output.append("")
    return output


def interpretation_rows(payload_sizes, fio, dodb, crossover_report):
    output = ["## Observed crossover relationships", ""]
    fio_orders = []
    for xfs_stage in ("XFS-pre", "XFS-post"):
        xfs_medians = [summarize([sample["operations_per_second"] for sample in fio[xfs_stage][size]])["median"] for size in payload_sizes]
        zfs_medians = [summarize([sample["operations_per_second"] for sample in fio["ZFS"][size]])["median"] for size in payload_sizes]
        fio_orders.append(all(xfs_value > zfs_value for xfs_value, zfs_value in zip(xfs_medians, zfs_medians)))
    if all(fio_orders):
        output.append(
            f"Median fio operations/s favors XFS-pre and XFS-post at every measured payload from {min(payload_sizes):,} through {max(payload_sizes):,} bytes. No payload-size refinement was triggered because the median ordering did not reverse between adjacent points."
        )
    elif not any(item["crossing_intervals"] for item in crossover_report["fio_operations_per_second"].values()):
        output.append("The fio median operations/s ordering did not reverse at adjacent measured payload points; no payload-size refinement was triggered.")
    high_variation = []
    for stage in ("XFS-pre", "XFS-post"):
        for payload_size in payload_sizes:
            samples = fio[stage][payload_size]
            values = [sample["operations_per_second"] for sample in samples]
            stats = summarize(values)
            if stats["cv_percent"] >= 20:
                high_variation.append((stage, payload_size, stats["cv_percent"], values[0], statistics.median(values[1:])))
    if high_variation:
        details = "; ".join(
            f"{stage} {payload_size:,} B CV {coefficient:.1f}% (rep1 {first_value:,.1f} ops/s, reps2–5 median {remaining_median:,.1f})"
            for stage, payload_size, coefficient, first_value, remaining_median in high_variation
        )
        output.append(f"XFS fio repetition spread is high at these points: {details}. The median rank is therefore less stable there than at points with lower CV.")
    for xfs_stage in ("XFS-pre", "XFS-post"):
        intervals = crossover_report["dodb_tx_per_second"][xfs_stage]["crossing_intervals"]
        if not intervals:
            output.append(f"The dodb throughput medians did not cross for {xfs_stage} in the measured width range.")
            continue
        for interval in intervals:
            lower_width = interval["lower_point"]
            upper_width = interval["upper_point"]
            lower_stats = {}
            upper_stats = {}
            for width, destination in ((lower_width, lower_stats), (upper_width, upper_stats)):
                for stage in (xfs_stage, "ZFS"):
                    samples = dodb[stage][width]
                    destination[stage] = {
                        metric: summarize([sample[metric] for sample in samples])["median"]
                        for metric in ("tx_per_second", "wal_bytes_per_sync", "wal_sync_mean_ms", "transactions_per_sync", "groups_per_second")
                    }
            output.append(
                f"For {xfs_stage}, median throughput changes from ZFS ahead at width {lower_width} to XFS ahead at width {upper_width}. At the lower width, WAL bytes/sync are {lower_stats[xfs_stage]['wal_bytes_per_sync']:,.0f} for XFS and {lower_stats['ZFS']['wal_bytes_per_sync']:,.0f} for ZFS; WAL sync mean is {lower_stats[xfs_stage]['wal_sync_mean_ms']:.3f} ms for XFS and {lower_stats['ZFS']['wal_sync_mean_ms']:.3f} ms for ZFS. At the upper width, WAL bytes/sync are {upper_stats[xfs_stage]['wal_bytes_per_sync']:,.0f} for XFS and {upper_stats['ZFS']['wal_bytes_per_sync']:,.0f} for ZFS; WAL sync mean is {upper_stats[xfs_stage]['wal_sync_mean_ms']:.3f} ms for XFS and {upper_stats['ZFS']['wal_sync_mean_ms']:.3f} ms for ZFS."
            )
            output.append(
                f"At these widths, the WAL bytes/sync medians are similar across filesystems, while WAL sync mean and groups/s change ordering with throughput. This is consistent with dodb's observed sync/group dynamics being more informative than the fio payload-size ranking in this range; it does not establish a causal mechanism."
            )
    output.append("")
    return output


def write_summary_tables(payload_sizes, fio, dodb, crossover_report):
    output = [
        "# XFS and ZFS Crossover Results",
        "",
        "Cells report the median and sample coefficient of variation (CV = sample standard deviation / mean × 100%). Each stage uses five repetitions. XFS-pre and XFS-post remain separate.",
        "",
    ]
    fio_definitions = []
    for payload_size in payload_sizes:
        for metric, label, precision in (
            ("operations_per_second", "operations/s", 1),
            ("mb_per_second", "MB/s", 2),
            ("sync_mean_ms", "fdatasync mean ms", 3),
            ("sync_p50_ms", "fdatasync p50 ms", 3),
            ("sync_p95_ms", "fdatasync p95 ms", 3),
            ("sync_p99_ms", "fdatasync p99 ms", 3),
            ("cpu_seconds", "CPU seconds", 3),
            ("device_bytes_written", "device bytes written", 0),
            ("device_busy_ms", "device busy ms", 0),
        ):
            fio_definitions.append((payload_size, metric, f"{payload_size:,} B {label}", precision))
    output.extend(metric_table("fio payload-size sweep", "payload and metric", payload_sizes, fio, fio_definitions))
    dodb_definitions = []
    for width in TRANSACTION_WIDTHS:
        for metric, label, precision in (
            ("tx_per_second", "tx/s", 1),
            ("p50_us", "p50 us", 1),
            ("p95_us", "p95 us", 1),
            ("p99_us", "p99 us", 1),
            ("wal_bytes", "WAL bytes", 0),
            ("wal_sync_count", "WAL sync count", 0),
            ("wal_bytes_per_sync", "WAL bytes/sync", 1),
            ("wal_sync_mean_ms", "WAL sync mean ms", 3),
            ("transactions_per_sync", "successful tx/sync", 2),
            ("average_group_size", "average group size", 2),
            ("groups_per_second", "groups/s", 2),
            ("physical_execution_ms", "physical execution ms", 2),
            ("planning_ms", "planning ms", 2),
            ("validation_ms", "validation ms", 2),
            ("publication_ms", "publication ms", 2),
            ("cpu_seconds", "CPU seconds", 3),
            ("device_bytes_written", "device bytes written", 0),
            ("device_busy_ms", "device busy ms", 0),
            ("errors", "errors", 0),
        ):
            dodb_definitions.append((width, metric, f"width {width} {label}", precision))
    output.extend(metric_table("dodb transaction-width sweep", "width and metric", TRANSACTION_WIDTHS, dodb, dodb_definitions))
    output.extend(
        [
            "## WAL and grouping correlation table",
            "",
            "Cells show median (sample CV%). Successful transactions per sync are computed from `successful_transactions / wal_syncs_delta`; WAL bytes per sync and WAL sync mean are computed per run before summarizing.",
            "",
            "| Filesystem stage | Width | tx/s | WAL bytes/sync | WAL sync mean ms | Avg group | Groups/s |",
            "|---|---:|---:|---:|---:|---:|---:|",
        ]
    )
    for stage in STAGES:
        for width in TRANSACTION_WIDTHS:
            samples = dodb[stage][width]
            output.append(
                f"| {stage} | {width} | {cell(samples, 'tx_per_second', 1)} | {cell(samples, 'wal_bytes_per_sync', 1)} | {cell(samples, 'wal_sync_mean_ms', 3)} | {cell(samples, 'average_group_size', 2)} | {cell(samples, 'groups_per_second', 2)} |"
            )
    output.append("")
    output.extend(drift_rows(payload_sizes, fio, dodb))
    output.extend(variation_comparison_rows(payload_sizes, fio, dodb))
    output.extend(interpretation_rows(payload_sizes, fio, dodb, crossover_report))
    output.extend(
        [
            "## Crossover locations",
            "",
            "The initial fio sweep crosses only when the XFS-pre or XFS-post median operations-per-second ordering changes relative to ZFS between adjacent payload sizes. The dodb crossover is determined the same way from adjacent transaction widths. See `crossover-analysis.json` for the medians and intervals.",
            "",
        ]
    )
    for stage, data in crossover_report["fio_operations_per_second"].items():
        output.append(f"- fio {stage}: {data['crossing_intervals'] or 'no observed operations-per-second crossover in the measured payload range'}.")
    for stage, data in crossover_report["dodb_tx_per_second"].items():
        output.append(f"- dodb {stage}: {data['crossing_intervals'] or 'no observed throughput crossover in the measured width range'}.")
    output.extend(
        [
            "",
            "## Interpretation boundary",
            "",
            "The results describe these two filesystems, this OCI storage device, this binary, and these diagnostic workloads. Fio payload points and dodb widths are not equivalent operations. No fio median throughput ranking reversal was observed from 4 KiB through 1 MiB, while dodb throughput reversed between widths 4 and 8 at roughly 25–52 KiB per WAL sync. Therefore, this sweep does not support a simple standalone sync-payload crossover as the explanation for dodb's ranking change. At the dodb crossover, WAL sync mean and groups/s reverse in the same direction as throughput while WAL bytes/sync remain similar between filesystems; that association is descriptive and does not establish the mechanism.",
            "",
        ]
    )
    (RESULT_DIRECTORY / "summary-tables.md").write_text("\n".join(output))


def main():
    payload_sizes, fio, dodb = load_all()
    if len(payload_sizes) < len(INITIAL_PAYLOAD_SIZES):
        raise ValueError(f"missing fio payload points: {payload_sizes}")
    write_summary_csv(payload_sizes, fio, dodb)
    write_run_order()
    crossover_report = make_crossover_report(payload_sizes, fio, dodb)
    write_summary_tables(payload_sizes, fio, dodb, crossover_report)
    create_charts(payload_sizes, fio, dodb)
    print(json.dumps(crossover_report, indent=2))


if __name__ == "__main__":
    main()
