#!/usr/bin/env python3

import json
import statistics
import sys
from collections import defaultdict
from pathlib import Path


EXPECTED_COMMIT = "42e53838e7ebabcd5f48ef1e015e0f71e2758668"
INPUT = Path(__file__).parents[1] / "prototype/raw/oci-a1-overlay.jsonl"


def median(rows, field):
    return statistics.median(row[field] for row in rows)


def main():
    path = Path(sys.argv[1]) if len(sys.argv) > 1 else INPUT
    rows = [json.loads(line) for line in path.read_text().splitlines() if line]
    commits = {row["git_commit"] for row in rows}
    if commits != {EXPECTED_COMMIT}:
        raise SystemExit(f"unexpected source commit set: {sorted(commits)}")

    by_kind = defaultdict(list)
    for row in rows:
        by_kind[row["record_type"]].append(row)

    print(f"source_commit={EXPECTED_COMMIT}")
    for segment_count in sorted(
        {row["segments_before_commit"] for row in by_kind["write_cpu"]}
    ):
        sample = [
            row
            for row in by_kind["write_cpu"]
            if row["segments_before_commit"] == segment_count
        ]
        print(
            "write_cpu",
            segment_count,
            f"ns_per_tx={median(sample, 'cpu_ns_per_tx'):.0f}",
            f"allocations_per_tx={median(sample, 'allocations_per_tx'):.2f}",
            f"bytes_per_tx={median(sample, 'allocated_bytes_per_tx'):.0f}",
        )

    for segment_count in sorted(
        {row["segments"] for row in by_kind["get"]}
    ):
        print(
            "get",
            segment_count,
            *[
                f"{case}={median([row for row in by_kind['get'] if row['segments'] == segment_count and row['case'] == case], 'prototype_over_base'):.3f}x"
                for case in ("newest", "oldest", "base_hit", "miss")
            ],
        )

    for segment_count in sorted(
        {row["segments"] for row in by_kind["range_read"]}
    ):
        print(
            "range_read",
            segment_count,
            *[
                f"{operation}={median([row for row in by_kind['range_read'] if row['segments'] == segment_count and row['operation'] == operation], 'prototype_over_base'):.2f}x"
                for operation in ("query", "scan")
            ],
        )

    correctness_runs = len(by_kind["correctness"])
    write_runs = len(by_kind["write_cpu"])
    read_runs = len(by_kind["get"]) + len(by_kind["range_read"])
    if (correctness_runs, write_runs, read_runs) != (3, 21, 108):
        raise SystemExit(
            f"unexpected record counts: correctness={correctness_runs}, "
            f"write={write_runs}, reads={read_runs}"
        )


if __name__ == "__main__":
    main()
