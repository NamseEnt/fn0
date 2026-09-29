import json
import statistics
import sys
from pathlib import Path


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: summarize_k_amplification.py RESULT_ROOT")
    result_root = Path(sys.argv[1])
    order = [
        json.loads(line)
        for line in (result_root / "run-order.jsonl").read_text().splitlines()
        if line.strip()
    ]
    grouped = {}
    for run in order:
        records = [
            json.loads(line)
            for line in (result_root / f'{run["run"]}.jsonl').read_text().splitlines()
            if line.strip()
        ]
        if len(records) != 1:
            raise SystemExit(f"expected one benchmark row for {run['run']}")
        row = records[0]
        if row["git_commit"] != run["monorepo_commit"]:
            raise SystemExit(f"source commit mismatch for {run['run']}")
        key = (
            run["writers"],
            run["width"],
            run["distribution"],
            run["repetition"],
            run["engine"],
        )
        grouped[key] = row
    scenarios = (
        (16, 1, "uniform", "16 / 1 / uniform"),
        (16, 16, "uniform", "16 / 16 / uniform"),
        (64, 1, "uniform", "64 / 1 / uniform"),
        (64, 16, "uniform", "64 / 16 / uniform"),
        (64, 16, "compact", "64 / 16 / compact"),
        (64, 16, "spread", "64 / 16 / spread"),
    )
    print("| Workload | Mode | Logical user bytes | WAL bytes | WAL amp | Materialized B-link bytes | Physical amp | Known subtotal bytes | Known subtotal amp |")
    print("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for writers, width, distribution, label in scenarios:
        for engine in ("h1", "phase-j", "phase-k"):
            rows = [
                grouped[(writers, width, distribution, repetition, engine)]
                for repetition in range(3)
            ]
            logical_values = [
                row["mutation_ops"] * (row["key_size"] + row["value_size"])
                for row in rows
            ]
            logical = statistics.median(logical_values)
            wal = statistics.median(row["planner_wal_bytes"] for row in rows)
            physical_values = [row["materialized_data_bytes_delta"] for row in rows]
            if engine == "h1":
                physical = None
                subtotal = None
            else:
                physical = statistics.median(physical_values)
                subtotal = wal + physical
            physical_amp = "n/a" if physical is None else f"{physical / logical:.3f}x"
            subtotal_bytes = "n/a" if subtotal is None else f"{subtotal:.0f}"
            subtotal_amp = "n/a" if subtotal is None else f"{subtotal / logical:.3f}x"
            print(
                f"| {label} | {engine} | {logical:.0f} | {wal:.0f} | "
                f"{wal / logical:.3f}x | {physical if physical is not None else 'n/a'} | "
                f"{physical_amp} | {subtotal_bytes} | {subtotal_amp} |"
            )


if __name__ == "__main__":
    main()
