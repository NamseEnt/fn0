import csv
import json
import pathlib
import statistics
import sys

results = pathlib.Path(sys.argv[1])
raw = results / "raw"
rows = []
for path in sorted(raw.glob("g0-*.jsonl")):
    record = json.loads(path.read_text(encoding="utf-8").splitlines()[0])
    rows.append(record)
groups = {}
for record in rows:
    key = (
        record["sync_mode"],
        record["writers"],
        record["transaction_width"],
        record["distribution"],
    )
    groups.setdefault(key, []).append(record)

output = results / "g0-summary.csv"
with output.open("w", newline="", encoding="utf-8") as destination:
    writer = csv.writer(destination)
    writer.writerow(
        [
            "sync_mode",
            "writers",
            "width",
            "distribution",
            "runs",
            "tx_per_second_median",
            "tx_per_second_min",
            "tx_per_second_max",
            "p50_us_median",
            "p95_us_median",
            "p99_us_median",
        ]
    )
    for key, records in sorted(groups.items()):
        rates = [record["logical_tx_per_second"] for record in records]
        writer.writerow(
            [
                *key,
                len(records),
                f"{statistics.median(rates):.3f}",
                f"{min(rates):.3f}",
                f"{max(rates):.3f}",
                f"{statistics.median(record['e2e_p50_us'] for record in records):.3f}",
                f"{statistics.median(record['e2e_p95_us'] for record in records):.3f}",
                f"{statistics.median(record['e2e_p99_us'] for record in records):.3f}",
            ]
        )
print(output)
