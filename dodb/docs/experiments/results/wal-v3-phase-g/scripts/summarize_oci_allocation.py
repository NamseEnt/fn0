import csv
import json
import pathlib
import sys

results = pathlib.Path(sys.argv[1])
source = results / "raw/attribution"
records = [
    json.loads(path.read_text(encoding="utf-8").splitlines()[0])
    for path in sorted(source.glob("g0-disabled-00-rep*-w64-width16-uniform.jsonl"))
]
if len(records) < 3:
    raise SystemExit(f"expected at least three primary G0 allocation samples, found {len(records)}")

metrics = [
    "alloc_calls",
    "free_calls",
    "alloc_bytes",
    "free_bytes",
    "alloc_le_16",
    "alloc_le_32",
    "alloc_le_64",
    "alloc_le_128",
    "alloc_le_256",
    "alloc_le_1024",
    "alloc_le_4096",
    "alloc_gt_4096",
]
stage_names = sorted(
    key.removeprefix("churn_").removesuffix("_alloc_calls")
    for key in records[0]
    if key.startswith("churn_") and key.endswith("_alloc_calls")
)
output = results / "counters/oci-g0-stage-attribution.csv"
output.parent.mkdir(parents=True, exist_ok=True)
with output.open("w", newline="", encoding="utf-8") as destination:
    writer = csv.writer(destination)
    writer.writerow(["stage", "tx_count", *[f"{metric}_per_tx" for metric in metrics]])
    total_transactions = sum(record["successful_transactions"] for record in records)
    for stage_name in stage_names:
        aggregates = {
            metric: sum(record.get(f"churn_{stage_name}_{metric}", 0) for record in records)
            for metric in metrics
        }
        writer.writerow(
            [
                stage_name,
                total_transactions,
                *[f"{aggregates[metric] / total_transactions:.6f}" for metric in metrics],
            ]
        )
print(output)
