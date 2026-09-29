import json
import statistics
import sys
from pathlib import Path


def workload_name(record):
    if record["writers"] == 64 and record["transaction_width"] == 16:
        if record["distribution"] == "same-leaf-heavy":
            return "compact"
        if record["distribution"] == "different-leaf-heavy":
            return "spread"
    return record["distribution"]


def percentile(rows, field):
    return statistics.median(float(row.get(field, 0)) for row in rows)


def frame_count_per_tx(row):
    if row["engine"] == "logical-overlay-blink":
        return int(row["transaction_width"]) + 1
    frames = sum(
        int(row.get(field, 0))
        for field in (
            "wal_page_image_records_delta",
            "wal_page_delta_records_delta",
            "wal_committed_batches_delta",
        )
    )
    return frames / max(1, int(row["successful_transactions"]))


root = Path(sys.argv[1])
records = []
for path in sorted(root.glob("*.jsonl")):
    for line in path.read_text().splitlines():
        if line.strip():
            record = json.loads(line)
            if record.get("record_type") == "run":
                records.append(record)
if len(records) != 36:
    raise SystemExit(f"expected 36 durable run rows, got {len(records)}")

run_order = [
    json.loads(line)
    for line in (root / "run-order.jsonl").read_text().splitlines()
    if line.strip()
]
if len(run_order) != 36:
    raise SystemExit(f"expected 36 provenance rows, got {len(run_order)}")

expected_commits = {
    "parallel-blink": "29f4f4bbc3172e7ef8c09e7bfcfa00b3bb54e814",
    "logical-overlay-blink": "0ec2528a7513e507b2033aba095a937373e754b7",
}
expected_binaries = {
    "parallel-blink": "422a4635d67bd08fb0dcfc2839727838fe539cd28b5518fc8684b19e411f8b76",
    "logical-overlay-blink": "bcbf337ce4713b7294fac1d494f2ba853c132bdb6ee59f8fd588a60a62d2cd16",
}
for record in records:
    expected = expected_commits[record["engine"]]
    if record["git_commit"] != expected:
        raise SystemExit(f"unexpected source commit for {record['engine']}: {record['git_commit']}")
    if record["engine"] == "logical-overlay-blink":
        if record["benchmark_data_dir"] != "/bench/zfs/db/phase-j-data":
            raise SystemExit("J run did not use the requested ZFS benchmark directory")
    if record["build_mode"] != "release" or record["sync_mode"] != "real":
        raise SystemExit("durable matrix contains a non-release or non-real-sync row")
    if int(record.get("errors", 0)) != 0 or int(record["successful_transactions"]) != int(record["attempted_transactions"]):
        raise SystemExit(f"run {record['git_commit']} contains failed transactions")

for index, row in enumerate(run_order):
    engine = row["engine"]
    if row["source_commit"] != expected_commits[engine]:
        raise SystemExit(f"unexpected source provenance for run {row['run']}")
    if row["binary_sha256"] != expected_binaries[engine]:
        raise SystemExit(f"unexpected binary digest for run {row['run']}")
    if row["data_root"] != "/bench/zfs/db/phase-j-data":
        raise SystemExit(f"unexpected data root for run {row['run']}")
    if row["order"] != index:
        raise SystemExit(f"invalid interleave sequence at run {row['run']}")
    expected_engine = "parallel-blink" if index % 2 == 0 else "logical-overlay-blink"
    if row["engine"] != expected_engine:
        raise SystemExit(f"H1/J interleave order is invalid at run {row['run']}")

groups = {}
for record in records:
    key = (
        int(record["writers"]),
        int(record["transaction_width"]),
        workload_name(record),
        record["engine"],
    )
    groups.setdefault(key, []).append(record)

scenario_order = [
    (16, 1, "uniform"),
    (16, 16, "uniform"),
    (64, 1, "uniform"),
    (64, 16, "uniform"),
    (64, 16, "compact"),
    (64, 16, "spread"),
]
print("| Writers | Width | Distribution | H1 tx/s | J tx/s | J/H1 | H1 p50/p95/p99 us | J p50/p95/p99 us | H1/J CPU % | H1/J WAL B/tx | H1/J frames/tx | H1/J syncs | H1/J tx/sync | J mat. count | J mat. avg/max ms |")
print("|---:|---:|---|---:|---:|---:|---|---|---:|---:|---:|---:|---:|---:|---:|")
for writers, width, distribution in scenario_order:
    h1 = sorted(groups[(writers, width, distribution, "parallel-blink")], key=lambda row: row["repetition"])
    logical = sorted(groups[(writers, width, distribution, "logical-overlay-blink")], key=lambda row: row["repetition"])
    if len(h1) != 3 or len(logical) != 3:
        raise SystemExit(f"expected three paired repetitions for {writers}/{width}/{distribution}")
    h1_rate = statistics.median(float(row["logical_tx_per_second"]) for row in h1)
    logical_rate = statistics.median(float(row["logical_tx_per_second"]) for row in logical)
    ratios = [float(j["logical_tx_per_second"]) / float(h["logical_tx_per_second"]) for h, j in zip(h1, logical)]
    h1_latency = "/".join(f"{percentile(h1, f'e2e_p{p}_us'):.1f}" for p in (50, 95, 99))
    logical_latency = "/".join(f"{percentile(logical, f'e2e_p{p}_us'):.1f}" for p in (50, 95, 99))
    h1_bytes = statistics.median(float(row.get("planner_wal_bytes", 0)) / max(1, int(row["successful_transactions"])) for row in h1)
    logical_bytes = statistics.median(float(row.get("planner_wal_bytes", 0)) / max(1, int(row["successful_transactions"])) for row in logical)
    h1_frames = statistics.median(frame_count_per_tx(row) for row in h1)
    logical_frames = statistics.median(frame_count_per_tx(row) for row in logical)
    h1_cpu = percentile(h1, "cpu_utilization_percent_one_core")
    logical_cpu = percentile(logical, "cpu_utilization_percent_one_core")
    h1_syncs = statistics.median(int(row.get("wal_syncs_delta", 0)) for row in h1)
    logical_syncs = statistics.median(int(row.get("wal_syncs_delta", 0)) for row in logical)
    h1_tx_per_sync = statistics.median(
        int(row["successful_transactions"]) / max(1, int(row.get("wal_syncs_delta", 0)))
        for row in h1
    )
    logical_tx_per_sync = statistics.median(
        int(row["successful_transactions"]) / max(1, int(row.get("wal_syncs_delta", 0)))
        for row in logical
    )
    mat_count = statistics.median(int(row.get("materializations_delta", 0)) for row in logical)
    mat_average_ms = statistics.median(
        float(row.get("materialization_total_nanos_delta", 0))
        / max(1, int(row.get("materializations_delta", 0)))
        / 1e6
        for row in logical
    )
    mat_max_ms = statistics.median(float(row.get("materialization_max_nanos", 0)) / 1e6 for row in logical)
    print(f"| {writers} | {width} | {distribution} | {h1_rate:.1f} | {logical_rate:.1f} | {statistics.median(ratios):.3f}x | {h1_latency} | {logical_latency} | {h1_cpu:.1f}/{logical_cpu:.1f} | {h1_bytes:.1f}/{logical_bytes:.1f} | {h1_frames:.2f}/{logical_frames:.2f} | {h1_syncs:.0f}/{logical_syncs:.0f} | {h1_tx_per_sync:.2f}/{logical_tx_per_sync:.2f} | {mat_count:.0f} | {mat_average_ms:.2f}/{mat_max_ms:.2f} |")
