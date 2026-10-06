import csv
import json
import pathlib
import re
import statistics
import sys

results = pathlib.Path(sys.argv[1])
raw = results / "raw"
engine_order = ["main-btree", "parallel-blink", "rocksdb", "sqlite-wal", "turso-wal", "turso-mvcc"]
records = {}
for path in sorted(raw.glob("*.jsonl")):
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            record = json.loads(line)
            engine = record.get("engine")
            case = path.name.rsplit(f"-{engine}-rep", 1)[0]
            repetition = int(path.name.rsplit("-rep", 1)[1].split(".", 1)[0])
            measured = record.get("measured", record)
            read_success = record.get("successful_read_percent", measured.get("successful_read_percent", 0.0))
            write_success = record.get("successful_write_percent", measured.get("successful_write_percent", 0.0))
            attempted_read = record.get("attempted_read_percent", measured.get("attempted_read_percent", 0.0))
            values = {
                "total_ops_per_second": record.get("aggregate_ops_per_second", measured.get("total_operations_per_second", 0.0)),
                "read_ops_per_second": record.get("read_ops_per_second", measured.get("successful_read_operations_per_second", measured.get("read_operations_per_second", 0.0))),
                "write_tx_per_second": record.get("logical_tx_per_second", measured.get("write_transactions_per_second", 0.0)),
                "mutation_ops_per_second": record.get("mutation_ops_per_second", measured.get("mutation_ops_per_second", 0.0)),
                "read_p50_us": record.get("read_p50_us", measured.get("read_p50_us", 0.0)),
                "read_p95_us": record.get("read_p95_us", measured.get("read_p95_us", 0.0)),
                "read_p99_us": record.get("read_p99_us", measured.get("read_p99_us", 0.0)),
                "write_p50_us": record.get("write_p50_us", measured.get("write_p50_us", 0.0)),
                "write_p95_us": record.get("write_p95_us", measured.get("write_p95_us", 0.0)),
                "write_p99_us": record.get("write_p99_us", measured.get("write_p99_us", 0.0)),
                "attempted_read_percent": attempted_read,
                "successful_read_percent": read_success,
                "successful_write_percent": write_success,
                "errors": record.get("errors", measured.get("errors", measured.get("failed_transactions", 0))),
                "conflicts": record.get("conflicts", measured.get("conflicts", 0)),
                "retries": measured.get("retries", 0),
                "busy": measured.get("busy", 0),
                "busy_snapshot": measured.get("busy_snapshot", 0),
                "overloads": record.get("overloads"),
                "client_workers": record.get("client_workers", record.get("writers", 0)),
                "seed": record.get("seed", 0),
                "scenario_index": record.get("scenario_index", 0),
                "transaction_width": record.get("transaction_width", record.get("width", 1)),
                "read_percent": record.get("requested_read_percent", record.get("read_percent", 0)),
                "working_set": record.get("working_set", 0),
                "value_size": record.get("value_size", 0),
                "duration_ms": record.get("requested_duration_ms", record.get("duration_ms", 0)),
                "actual_wall_ms": record.get("duration_ms", 0),
                "warmup_ms": record.get("warmup_ms", 0),
                "git_commit": record.get("git_commit", ""),
                "logical_trace_prefix_hash": record.get("logical_trace_prefix_hash", ""),
            }
            records.setdefault((case, engine), []).append((repetition, values, path.name))

expected_cases = sorted({re.sub(r"-(main-btree|parallel-blink|rocksdb|sqlite-wal|turso-wal|turso-mvcc)-rep1\.jsonl$", "", path.name) for path in raw.glob("*-rep1.jsonl")})
rows = []
for case in expected_cases:
    for engine in engine_order:
        runs = sorted(records.get((case, engine), []))
        if len(runs) != 3:
            raise SystemExit(f"incomplete matrix: {case} {engine}: {len(runs)}/3 repetitions")
        spec_keys = ("client_workers", "scenario_index", "transaction_width", "read_percent", "working_set", "value_size", "warmup_ms", "duration_ms", "git_commit")
        specs = {tuple(run[1][key] for key in spec_keys) for run in runs}
        if len(specs) != 1:
            raise SystemExit(f"inconsistent scenario metadata: {case} {engine}")
        values = [run[1] for run in runs]
        row = {key: values[0][key] for key in ("client_workers", "transaction_width", "read_percent", "working_set", "value_size", "warmup_ms", "duration_ms", "git_commit")}
        row["logical_trace_prefix_hashes"] = ";".join(run["logical_trace_prefix_hash"] for run in values)
        row.update({"case": case, "engine": engine, "repetitions": len(runs)})
        metric_keys = [key for key in values[0] if key not in row and key not in ("seed", "scenario_index")]
        for key in metric_keys:
            metric_values = [run[key] for run in values if run[key] is not None]
            row[key] = statistics.median(metric_values) if metric_values else ""
        row["actual_read_write"] = f"{row['successful_read_percent']:.2f}/{row['successful_write_percent']:.2f}"
        rows.append(row)

for case in expected_cases:
    for repetition in (1, 2, 3):
        matched = [records[(case, engine)][repetition - 1][1] for engine in engine_order]
        if len({item["seed"] for item in matched}) != 1:
            raise SystemExit(f"seed mismatch across engines: {case} repetition {repetition}")
        if len({item["logical_trace_prefix_hash"] for item in matched}) != 1:
            raise SystemExit(f"logical workload trace mismatch: {case} repetition {repetition}")

csv_path = results / "mixed-summary.csv"
with csv_path.open("w", newline="", encoding="utf-8") as output:
    writer = csv.DictWriter(output, fieldnames=list(rows[0]))
    writer.writeheader()
    writer.writerows(rows)
print(f"cases={len(expected_cases)} engines={len(engine_order)} repetitions=3 rows={len(rows)} csv={csv_path}")
