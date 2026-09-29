import json
import gzip
import math
import statistics
from collections import defaultdict
from pathlib import Path


artifact_root = Path(__file__).resolve().parents[1]
scenarios = [
    (64, 16, "uniform"),
    (64, 1, "uniform"),
    (64, 16, "same-leaf-heavy"),
    (64, 16, "different-leaf-heavy"),
    (16, 16, "uniform"),
]


def percentile(values, fraction):
    ordered = sorted(values)
    return ordered[math.ceil(fraction * len(ordered)) - 1] if ordered else 0


def summarize(values):
    return (
        statistics.fmean(values) if values else 0,
        percentile(values, 0.50),
        percentile(values, 0.95),
        max(values, default=0),
    )


def gather(sync_mode, writers, width, distribution):
    group_rows = []
    run_rows = []
    directory = artifact_root / "raw" / "locality" / sync_mode
    for path in sorted(directory.glob("*.groups.jsonl.gz")):
        with gzip.open(path, "rt", encoding="utf-8") as input_file:
            group_lines = input_file.read().splitlines()
        for line in group_lines:
            row = json.loads(line)
            if (
                row["writers"],
                row["width"],
                row["distribution"],
            ) == (writers, width, distribution):
                group_rows.append(row)
    for path in sorted(directory.glob("locality-*.jsonl")):
        if ".groups." in path.name or path.name == "run-order.jsonl":
            continue
        for line in path.read_text(encoding="utf-8").splitlines():
            row = json.loads(line)
            if row.get("record_type") == "run" and (
                row["writers"],
                row["transaction_width"],
                row["distribution"],
            ) == (writers, width, distribution):
                run_rows.append(row)
    return group_rows, run_rows


def report(sync_mode):
    print(f"## {sync_mode} sync\n")
    print(
        "| Scenario | Groups | Tx/group mean / p50 / p95 / max | Mutations/group mean / p50 / p95 / max | "
        "Boundaries/group mean | Leaves/group mean | Model A removable | "
        "Mutations/leaf mean / p50 / p95 / max | Tx/leaf mean / p50 / p95 / max | "
        "Leaves touched by 1 / 2 / 3+ tx | Page encodes/group | PageDelta/group |"
    )
    print("|---|---:|---|---|---:|---:|---:|---|---|---:|---:|---:|")
    for writers, width, distribution in scenarios:
        groups, runs = gather(sync_mode, writers, width, distribution)
        if not groups or len(runs) != 3:
            raise SystemExit(
                f"{sync_mode} {writers}w width{width} {distribution}: "
                f"groups={len(groups)} runs={len(runs)}"
            )
        tx_stats = summarize([row["successful_transactions"] for row in groups])
        mutation_stats = summarize([row["logical_mutations"] for row in groups])
        boundary_stats = summarize([row["boundary_materializations"] for row in groups])
        leaf_stats = summarize([row["distinct_touched_leaves"] for row in groups])
        mutation_leaf_stats = summarize(
            [value for row in groups for value in row["mutations_per_leaf"]]
        )
        transaction_leaf_stats = summarize(
            [value for row in groups for value in row["transactions_per_leaf"]]
        )
        boundary_total = sum(row["boundary_materializations"] for row in groups)
        leaves_total = sum(row["distinct_touched_leaves"] for row in groups)
        removable = (boundary_total - leaves_total) / boundary_total if boundary_total else 0
        touched = [sum(row["leaves_by_transaction_touch_count"][slot] for row in groups) for slot in range(3)]
        touched_total = sum(touched)
        touched_rates = [value / touched_total if touched_total else 0 for value in touched]
        page_encodes = statistics.fmean(row["page_encodes"] for row in groups)
        deltas = statistics.fmean(row["page_delta_records"] for row in groups)
        scenario_name = f"{writers}w width{width} {distribution}"
        print(
            f"| {scenario_name} | {len(groups)} | "
            f"{tx_stats[0]:.2f} / {tx_stats[1]} / {tx_stats[2]} / {tx_stats[3]} | "
            f"{mutation_stats[0]:.2f} / {mutation_stats[1]} / {mutation_stats[2]} / {mutation_stats[3]} | "
            f"{boundary_stats[0]:.2f} | {leaf_stats[0]:.2f} | {removable:.2%} | "
            f"{mutation_leaf_stats[0]:.2f} / {mutation_leaf_stats[1]} / {mutation_leaf_stats[2]} / {mutation_leaf_stats[3]} | "
            f"{transaction_leaf_stats[0]:.2f} / {transaction_leaf_stats[1]} / {transaction_leaf_stats[2]} / {transaction_leaf_stats[3]} | "
            f"{touched[0]} ({touched_rates[0]:.1%}) / {touched[1]} ({touched_rates[1]:.1%}) / "
            f"{touched[2]} ({touched_rates[2]:.1%}) | {page_encodes:.2f} | {deltas:.2f} |"
        )
        print(
            f"\n{scenario_name} {sync_mode}: requests/group "
            f"{summarize([row['requested_transactions'] for row in groups])}; "
            f"failed transactions/group "
            f"{statistics.fmean(row['failed_transactions'] for row in groups):.3f}; "
            f"unique keys/group {statistics.fmean(row['unique_keys'] for row in groups):.2f}; "
            f"successful tx/s median {statistics.median(row['logical_tx_per_second'] for row in runs):.0f}.\n"
        )


for mode in ("disabled", "real"):
    if (artifact_root / "raw" / "locality" / mode).exists():
        report(mode)
