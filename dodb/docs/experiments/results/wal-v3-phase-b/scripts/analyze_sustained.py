import glob
import json
import os
import sys

RESULTS = sys.argv[1]
RAW = os.path.join(RESULTS, "raw")
MIB = 1024 * 1024
PAGE_IMAGE_FRAME = 4156
FRAME_OVERHEAD = 52
COMMIT_FRAME = 68


def load(path):
    with open(path, encoding="utf-8") as input_file:
        return [json.loads(line) for line in input_file if line.strip()]


def nearest(monitor, unix_ms):
    return min(monitor, key=lambda row: abs(row["unix_ms"] - unix_ms))


def written_bytes(record):
    if "wal_page_image_records_delta" not in record:
        return record["successful_transactions"] * (PAGE_IMAGE_FRAME + COMMIT_FRAME)
    deltas = record.get("wal_page_delta_records_delta", 0)
    images = record.get("wal_page_image_records_delta", record["page_images_delta"])
    return (images * PAGE_IMAGE_FRAME + record.get("wal_page_delta_payload_bytes_delta", 0)
            + deltas * FRAME_OVERHEAD + record["successful_transactions"] * COMMIT_FRAME)


def sample(record, index):
    prefix = f"resource_sample_{index:02}"
    values = {key[len(prefix) + 1:]: value for key, value in record.items() if key.startswith(prefix + "_")}
    for key in ("wal_page_image_records", "wal_page_delta_records", "wal_syncs", "wal_sync_nanos"):
        values.setdefault(key, None)
    return values


def run_table(output, name):
    path = os.path.join(RAW, f"{name}.jsonl")
    if not os.path.exists(path) or not open(path).read().strip():
        output.append(f"### {name}\n\nno JSON row (run did not finish)\n")
        return None
    record = load(path)[0]
    assert record["engine"] == "planned-blink" and record["sync_mode"] == "real", path
    monitor = load(os.path.join(RAW, f"{name}.process-monitor.jsonl"))
    samples = [sample(record, index) for index in range(record["resource_sample_count"])]
    start = next(item for item in samples if item["label"] == "measurement_start")
    start_monitor = nearest(monitor, start["unix_ms"])
    windows = [item for item in samples if item["label"].startswith("window_")]
    threshold = record.get("checkpoint_wal_bytes_threshold", 0)
    transactions = record["successful_transactions"]
    output.append(f"### {name}\n")
    output.append(
        f"- tx/s {record['logical_tx_per_second']:,.0f}, p50 {record['e2e_p50_us']:,.0f} µs, "
        f"p95 {record['e2e_p95_us']:,.0f} µs, p99 {record['e2e_p99_us']:,.0f} µs, "
        f"{transactions:,} tx, errors {record['errors']}")
    output.append(
        f"- WAL bytes written per tx (from redo counters): {written_bytes(record) / transactions:,.1f}; "
        f"image records/tx {record.get('wal_page_image_records_delta', transactions) / transactions:.4f}, "
        f"delta records/tx {record.get('wal_page_delta_records_delta', 0) / transactions:.4f}")
    output.append(
        f"- peak RSS {max(row['rss_kib'] or 0 for row in monitor) / 1024:,.0f} MiB, "
        f"RSS at measurement start {start_monitor['rss_kib'] / 1024:,.0f} MiB, "
        f"min MemAvailable {min(row['mem_available_kib'] for row in monitor) / 1024:,.0f} MiB")
    if threshold:
        events = record["checkpoint_events"]
        output.append(
            f"- checkpoint threshold {threshold / MIB:,.0f} MiB: {record['checkpoint_count']} checkpoints in the "
            f"measurement, total {record['checkpoint_total_nanos'] / 1e9:,.2f} s, max {record['checkpoint_max_nanos'] / 1e6:,.0f} ms, "
            f"WAL reclaimed {record['checkpoint_wal_bytes_reclaimed'] / MIB:,.0f} MiB")
        output.append(f"- checkpoint events (offset:duration:WAL before): `{events}`")
    output.append("")
    header = "| Window | tx/s | p99 µs | WAL MiB (end) | RSS MiB (end) | dirty pages | delta ratio | image ratio | mean sync ms |"
    divider = "|---|---|---|---|---|---|---|---|---|"
    if threshold:
        header += " checkpoints | est. WAL B/tx |"
        divider += "---|---|"
    output.append(header)
    output.append(divider)
    previous = start
    rows = []
    for window_index, window in enumerate(windows):
        if f"window_{window_index:02}_logical_tx_per_second" not in record:
            break
        if record[f"window_{window_index:02}_span_seconds"] < 9.99:
            break
        monitor_row = nearest(monitor, window["unix_ms"])
        def change(key):
            if window[key] is None or previous[key] is None:
                return None
            return window[key] - previous[key]

        images = change("wal_page_image_records")
        deltas = change("wal_page_delta_records")
        syncs = change("wal_syncs")
        sync_nanos = change("wal_sync_nanos")
        window_tx = record[f"window_{window_index:02}_successful_transactions"]
        if images is None:
            images, deltas = window_tx, 0

        line = (f"| {window_index * 10}-{window_index * 10 + 10} s | "
                f"{record[f'window_{window_index:02}_logical_tx_per_second']:,.0f} | "
                f"{record[f'window_{window_index:02}_p99_us']:,.0f} | "
                f"{monitor_row['wal_bytes'] / MIB:,.0f} | {monitor_row['rss_kib'] / 1024:,.0f} | "
                f"{window['dirty_pages']:,} | {deltas / max(images + deltas, 1):.3f} | "
                f"{images / max(images + deltas, 1):.3f} | "
                + (f"{sync_nanos / syncs / 1e6:.2f} |" if syncs else "— |"))
        if threshold:
            estimate = (images * PAGE_IMAGE_FRAME + deltas * 159 + window_tx * COMMIT_FRAME) / max(window_tx, 1)
            line += f" {record.get(f'window_{window_index:02}_checkpoints', 0)} | {estimate:,.0f} |"
        output.append(line)
        rows.append((monitor_row, window))
        previous = window
    if rows and not threshold:
        wal_growth = rows[-1][0]["wal_bytes"] - start_monitor["wal_bytes"]
        rss_growth = (rows[-1][0]["rss_kib"] - start_monitor["rss_kib"]) * 1024
        output.append("")
        output.append(f"- 120 s: WAL growth {wal_growth / MIB:,.0f} MiB, RSS growth {rss_growth / MIB:,.0f} MiB")
    output.append("")
    return record


def main():
    output = ["# WAL v3 Phase B sustained and checkpoint tables\n"]
    output.append("## Sustained, no checkpoint (page-delta, 1,000,000 rows, width 1, uniform)\n")
    for name in sorted(glob.glob(os.path.join(RAW, "sustained-page-delta-nockpt-*-uniform.jsonl"))):
        run_table(output, os.path.basename(name)[:-len(".jsonl")])
    output.append("## Checkpoint control (16 writers, 1,000,000 rows, width 1, uniform)\n")
    output.append("Estimated WAL B/tx per window uses 4,156 B per image record, 159 B per delta record (mean measured frame) and 68 B per commit.\n")
    for name in sorted(glob.glob(os.path.join(RAW, "sustained-*-ckpt*m-*-uniform.jsonl"))):
        run_table(output, os.path.basename(name)[:-len(".jsonl")])
    with open(os.path.join(RESULTS, "sustained-tables.md"), "w", encoding="utf-8") as output_file:
        output_file.write("\n".join(output) + "\n")
    print("\n".join(output))


if __name__ == "__main__":
    main()
