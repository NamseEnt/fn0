import json
import statistics
import sys
from pathlib import Path


def median(rows, field):
    return statistics.median(row[field] for row in rows)


def read_runs(result_root):
    order = [
        json.loads(line)
        for line in (result_root / "run-order.jsonl").read_text().splitlines()
        if line.strip()
    ]
    grouped = {}
    for run in order:
        row_path = result_root / f'{run["run"]}.jsonl'
        records = [json.loads(line) for line in row_path.read_text().splitlines() if line.strip()]
        if len(records) != 1:
            raise SystemExit(f"expected one row in {row_path}, found {len(records)}")
        key = (
            run["writers"],
            run["width"],
            run["distribution"],
            run["repetition"],
            run["engine"],
        )
        grouped[key] = records[0]
        if records[0]["git_commit"] != run["monorepo_commit"]:
            raise SystemExit(f"source commit mismatch for {run['run']}")
    if len(order) != 54:
        raise SystemExit(f"expected 54 runs, found {len(order)}")
    return grouped


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: summarize_k_matrix.py RESULT_ROOT")
    result_root = Path(sys.argv[1])
    rows = read_runs(result_root)
    scenarios = (
        (16, 1, "uniform", "16 / 1 / uniform"),
        (16, 16, "uniform", "16 / 16 / uniform"),
        (64, 1, "uniform", "64 / 1 / uniform"),
        (64, 16, "uniform", "64 / 16 / uniform"),
        (64, 16, "compact", "64 / 16 / compact"),
        (64, 16, "spread", "64 / 16 / spread"),
    )
    print("| Writers / width / distribution | H1 tx/s | J tx/s | K tx/s | J/H1 | K/J | K/H1 |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    matrix = {}
    for writers, width, distribution, label in scenarios:
        values = {
            engine: [
                rows[(writers, width, distribution, repetition, engine)]
                for repetition in range(3)
            ]
            for engine in ("h1", "phase-j", "phase-k")
        }
        rates = {
            engine: median(group, "logical_tx_per_second")
            for engine, group in values.items()
        }
        ratios = {
            f"{candidate}/{baseline}": statistics.median(
                values[candidate][repetition]["logical_tx_per_second"]
                / values[baseline][repetition]["logical_tx_per_second"]
                for repetition in range(3)
            )
            for candidate, baseline in (
                ("phase-j", "h1"),
                ("phase-k", "phase-j"),
                ("phase-k", "h1"),
            )
        }
        matrix[label] = values
        print(
            f"| {label} | {rates['h1']:.1f} | {rates['phase-j']:.1f} | "
            f"{rates['phase-k']:.1f} | {ratios['phase-j/h1']:.3f}x | "
            f"{ratios['phase-k/phase-j']:.3f}x | {ratios['phase-k/h1']:.3f}x |"
        )
    print()
    print("| Writers / width / distribution | H1 p50/p95/p99 us | J p50/p95/p99 us | K p50/p95/p99 us |")
    print("| --- | ---: | ---: | ---: |")
    for _, _, _, label in scenarios:
        values = matrix[label]
        latencies = {}
        for engine, group in values.items():
            latencies[engine] = "/".join(
                str(round(median(group, f"e2e_{percentile}_us")))
                for percentile in ("p50", "p95", "p99")
            )
        print(
            f"| {label} | {latencies['h1']} | {latencies['phase-j']} | "
            f"{latencies['phase-k']} |"
        )
    primary = matrix["64 / 16 / uniform"]
    print()
    print("| Primary 64 / 16 / uniform metric (median) | H1 | J | K |")
    print("| --- | ---: | ---: | ---: |")
    fields = (
        ("writer_blocked_nanos_delta", "writer blocked ns"),
        ("materialization_total_nanos_delta", "materialization total ns"),
        ("materialization_cpu_nanos_delta", "materialization CPU ns"),
        ("materialization_data_write_nanos_delta", "data write ns"),
        ("materialization_data_sync_nanos_delta", "data sync ns"),
        ("materialization_checkpoint_nanos_delta", "checkpoint construction ns"),
        ("materialization_checkpoint_sync_nanos_delta", "checkpoint sync ns"),
        ("publish_pause_nanos_delta", "publish pause cumulative ns"),
        ("backpressure_nanos_delta", "backpressure ns"),
        ("backpressure_events_delta", "backpressure events"),
        ("materializations_delta", "materializations"),
        ("materialized_segments_delta", "segments materialized"),
        ("materialized_data_bytes_delta", "B-link data bytes materialized"),
        ("materialization_wal_bytes_reclaimed_delta", "WAL bytes reclaimed during materialization"),
        ("wal_bytes_retained", "WAL bytes retained at run end"),
        ("overlay_segments_peak", "peak overlay segments"),
    )
    for field, label in fields:
        print(
            f"| {label} | "
            + " | ".join(
                str(round(median(primary[engine], field)))
                for engine in ("h1", "phase-j", "phase-k")
            )
            + " |"
        )


if __name__ == "__main__":
    main()
