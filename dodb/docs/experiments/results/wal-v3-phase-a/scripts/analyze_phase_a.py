import json
import math
import pathlib
import statistics
import sys

RESULTS = pathlib.Path(sys.argv[1])
RAW = RESULTS / "raw"
MIB = 1024 * 1024


def load_record(path):
    record = json.loads(path.read_text(encoding="utf-8"))
    assert record["engine"] == "planned-blink", path
    assert record["sync_mode"] == "real", path
    assert record["errors"] == 0, path
    return record


def short_table(output):
    output.append("## Short A/B (5 s measure, 2 s warmup, working set 100,000)\n")
    output.append("| Writers | Variant | tx/s rep1 | rep2 | rep3 | mean | p99 µs mean | WAL B/tx | peak RSS MiB mean |")
    output.append("|---|---|---|---|---|---|---|---|---|")
    process_rows = [json.loads(line) for line in (RESULTS / "process-metrics.jsonl").read_text().splitlines()]
    peak_by_output = {pathlib.Path(row["jsonl"]).name: row["rss_peak_sampled_kib"] for row in process_rows}
    summary = {}
    for scenario_index, writers in ((0, 16), (6, 64)):
        for variant in ("old-v2", "retention-fixed-v2"):
            records = []
            peaks = []
            for repetition in (1, 2, 3):
                name = f"short-{scenario_index:02d}-rep{repetition}-{variant}-w{writers}-width1-uniform.jsonl"
                records.append(load_record(RAW / name))
                peaks.append(peak_by_output[name] / 1024)
            throughputs = [record["logical_tx_per_second"] for record in records]
            p99s = [record["e2e_p99_us"] for record in records]
            wal_per_tx = {record["wal_bytes_delta"] / record["successful_transactions"] for record in records}
            summary[(writers, variant)] = statistics.mean(throughputs)
            output.append(
                f"| {writers} | {variant} | " + " | ".join(f"{value:,.0f}" for value in throughputs)
                + f" | {statistics.mean(throughputs):,.0f} | {statistics.mean(p99s):,.0f} | "
                + ", ".join(f"{value:,.1f}" for value in sorted(wal_per_tx))
                + f" | {statistics.mean(peaks):,.0f} |"
            )
    output.append("")
    for writers in (16, 64):
        ratio = summary[(writers, "retention-fixed-v2")] / summary[(writers, "old-v2")]
        output.append(f"- {writers} writers: retention-fixed / old = {ratio:.3f}")
    output.append("")


def monitor_rows(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


def nearest_monitor(rows, unix_ms):
    return min(rows, key=lambda row: abs(row["unix_ms"] - unix_ms))


def sustained_table(output, name, internal_samples):
    record = load_record(RAW / f"{name}.jsonl")
    monitor = monitor_rows(RAW / f"{name}.process-monitor.jsonl")
    window_count = sum(1 for window_index in range(record["window_count"])
                       if record[f"window_{window_index:02}_span_seconds"] >= 9.99)
    wall_seconds = record["successful_transactions"] / record["logical_tx_per_second"]
    if internal_samples:
        start_ms = record["resource_sample_01_unix_ms"]
        assert record["resource_sample_01_label"] == "measurement_start"
    else:
        start_ms = record["timestamp_unix_ms"] - wall_seconds * 1000
    start = nearest_monitor(monitor, start_ms)
    output.append(f"### {name}\n")
    output.append(f"- tx/s {record['logical_tx_per_second']:,.0f}, p50 {record['e2e_p50_us']:,.0f} µs, "
                  f"p99 {record['e2e_p99_us']:,.0f} µs, successful {record['successful_transactions']:,}, "
                  f"errors {record['errors']}, WAL B/tx {record['wal_bytes_delta'] / record['successful_transactions']:,.1f}")
    output.append(f"- peak sampled RSS {max(row['rss_kib'] for row in monitor) / 1024:,.0f} MiB, "
                  f"min MemAvailable {min(row['mem_available_kib'] for row in monitor) / 1024:,.0f} MiB")
    if internal_samples:
        seeded = record["resource_sample_00_rss_kib"] / 1024
        output.append(f"- after seeding: RSS {seeded:,.0f} MiB, WAL {record['resource_sample_00_wal_bytes'] / MIB:,.0f} MiB, "
                      f"dirty pages {record['resource_sample_00_dirty_pages']:,}")
    output.append(f"- measurement start: RSS {start['rss_kib'] / 1024:,.0f} MiB, WAL {start['wal_bytes'] / MIB:,.0f} MiB\n")
    header = "| Window | tx/s | p50 µs | p99 µs | WAL end MiB | WAL growth MiB | RSS end MiB | RSS growth MiB |"
    divider = "|---|---|---|---|---|---|---|---|"
    if internal_samples:
        header += " dirty pages | retained recovery batches / images |"
        divider += "---|---|"
    output.append(header)
    output.append(divider)
    rows = []
    for window_index in range(window_count):
        end_ms = start_ms + (window_index + 1) * 10_000
        end = nearest_monitor(monitor, end_ms)
        line = (f"| {window_index * 10}-{window_index * 10 + 10} s | "
                f"{record[f'window_{window_index:02}_logical_tx_per_second']:,.0f} | "
                f"{record[f'window_{window_index:02}_p50_us']:,.0f} | "
                f"{record[f'window_{window_index:02}_p99_us']:,.0f} | "
                f"{end['wal_bytes'] / MIB:,.0f} | {(end['wal_bytes'] - start['wal_bytes']) / MIB:,.0f} | "
                f"{end['rss_kib'] / 1024:,.0f} | {(end['rss_kib'] - start['rss_kib']) / 1024:,.0f} |")
        if internal_samples:
            sample_index = window_index + 2
            prefix = f"resource_sample_{sample_index:02}"
            assert record[f"{prefix}_label"] == f"window_{window_index:02}_end"
            line += (f" {record[f'{prefix}_dirty_pages']:,} | "
                     f"{record[f'{prefix}_retained_recovery_batches']} / {record[f'{prefix}_retained_recovery_page_images']} |")
        output.append(line)
        rows.append((end['wal_bytes'] - start['wal_bytes'], end['rss_kib'] * 1024 - start['rss_kib'] * 1024))
    wal_growth, rss_growth = rows[-1]
    output.append("")
    output.append(f"- 120 s: WAL growth {wal_growth / MIB:,.0f} MiB, RSS growth {rss_growth / MIB:,.0f} MiB, "
                  f"RSS bytes per WAL byte {rss_growth / wal_growth:.3f}")
    later_wal = rows[-1][0] - rows[5][0]
    later_rss = rows[-1][1] - rows[5][1]
    output.append(f"- 60-120 s: WAL growth {later_wal / MIB:,.0f} MiB, RSS growth {later_rss / MIB:,.1f} MiB, "
                  f"RSS bytes per WAL byte {later_rss / later_wal:.4f}")
    output.append("")


def main():
    output = ["# WAL v3 Phase A tables\n"]
    short_table(output)
    output.append("## Sustained (working set 1,000,000, width 1, uniform, 10 s warmup, 120 s measure)\n")
    sustained_table(output, "sustained-retention-fixed-v2-14-rep1-w16-width1-uniform", True)
    sustained_table(output, "sustained-retention-fixed-v2-15-rep1-w64-width1-uniform", True)
    sustained_table(output, "sustained-old-v2-window-14-rep1-w16-width1-uniform", False)
    (RESULTS / "tables.md").write_text("\n".join(output) + "\n", encoding="utf-8")
    print("\n".join(output))


if __name__ == "__main__":
    main()
