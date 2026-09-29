import json
import pathlib
import sys

record_path = pathlib.Path(sys.argv[1])
record = json.loads(record_path.read_text(encoding="utf-8").splitlines()[0])
transaction_count = record["successful_transactions"]
sites = sorted({
    key.removeprefix("churn_").removesuffix("_alloc_calls")
    for key in record
    if key.startswith("churn_") and key.endswith("_alloc_calls")
})
print(f"scenario={record['workload']} successful_transactions={transaction_count}")
print(f"logical_tx_per_second={record['logical_tx_per_second']}")
print("stage,alloc_calls_per_tx,free_calls_per_tx,allocated_bytes_per_tx,freed_bytes_per_tx,arena_calls_per_tx,arena_bytes_per_tx")
for site in sites:
    metric_values = [
        record.get(f"churn_{site}_alloc_calls", 0),
        record.get(f"churn_{site}_free_calls", 0),
        record.get(f"churn_{site}_alloc_bytes", 0),
        record.get(f"churn_{site}_free_bytes", 0),
        record.get(f"churn_{site}_arena_alloc_calls", 0),
        record.get(f"churn_{site}_arena_alloc_bytes", 0),
    ]
    per_transaction = [value / transaction_count for value in metric_values]
    print(site + "," + ",".join(f"{value:.3f}" for value in per_transaction))
