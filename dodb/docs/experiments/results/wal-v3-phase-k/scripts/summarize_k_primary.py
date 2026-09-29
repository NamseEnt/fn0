#!/usr/bin/env python3
import json
import statistics
import sys
from pathlib import Path


def read_row(path):
    rows = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    if len(rows) != 1:
        raise SystemExit(f"expected one benchmark row in {path}, found {len(rows)}")
    return rows[0]


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: summarize_k_primary.py RESULT_ROOT")
    result_root = Path(sys.argv[1])
    run_order = [json.loads(line) for line in (result_root / "run-order.jsonl").read_text().splitlines()]
    grouped = {"phase-j": [], "phase-k": []}
    for run in run_order:
        grouped[run["engine"]].append(read_row(result_root / f'{run["run"]}.jsonl'))
    if not grouped["phase-j"] or len(grouped["phase-j"]) != len(grouped["phase-k"]):
        raise SystemExit("J and K repetition counts do not match")
    for engine, rows in grouped.items():
        throughput = statistics.median(row["logical_tx_per_second"] for row in rows)
        blocked = statistics.median(row["writer_blocked_nanos_delta"] for row in rows)
        materialization = statistics.median(row["materialization_total_nanos_delta"] for row in rows)
        publish = statistics.median(row["publish_pause_nanos_delta"] for row in rows)
        print(
            f"{engine}: median_tx_per_second={throughput:.2f} "
            f"writer_blocked_nanos={blocked:.0f} "
            f"materialization_total_nanos={materialization:.0f} "
            f"publish_pause_nanos={publish:.0f}"
        )
    j_blocked = statistics.median(row["writer_blocked_nanos_delta"] for row in grouped["phase-j"])
    k_blocked = statistics.median(row["writer_blocked_nanos_delta"] for row in grouped["phase-k"])
    reduction = 1.0 if j_blocked == 0 else 1.0 - k_blocked / j_blocked
    print(f"writer_blocked_reduction={reduction:.4%}")
    if reduction < 0.90:
        raise SystemExit("primary diagnostic failed the 90% writer-blocking reduction gate")


if __name__ == "__main__":
    main()
