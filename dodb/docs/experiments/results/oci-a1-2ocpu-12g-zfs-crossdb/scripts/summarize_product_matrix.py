import argparse
import csv
import json
import pathlib
import statistics


parser = argparse.ArgumentParser()
parser.add_argument("--results", action="append", required=True, type=pathlib.Path)
parser.add_argument("--replacement-results", action="append", default=[], type=pathlib.Path)
parser.add_argument("--replace-engine", action="append", default=[])
parser.add_argument("--output", required=True, type=pathlib.Path)
parser.add_argument("--csv-output", type=pathlib.Path)
arguments = parser.parse_args()

engine_names = ["main-btree", "parallel-blink", "sqlite-wal", "turso-wal", "turso-mvcc", "rocksdb"]


def category(case_name):
    if case_name.startswith(("P1-", "P2-", "P3-", "P4-", "P5-", "P6-", "P7-")):
        return "product-core"
    if case_name.startswith("P8-"):
        return "hotspot"
    if case_name.startswith("P9-"):
        return "architecture-stress"
    if case_name.startswith("cache-pressure-"):
        return "cache-pressure"
    if case_name.startswith("large-overflow-"):
        return "large-value-focused"
    return "unclassified"


def median(record, *keys):
    for key in keys:
        value = record.get(key)
        if isinstance(value, (int, float)):
            return value
    return None


def measured(record):
    return record.get("measured", record)


def throughput(case_name, record):
    values = measured(record)
    if case_name.startswith(("P1-get-", "large-overflow-get-", "cache-pressure-get-")):
        return median(values, "read_ops_per_second", "read_operations_per_second")
    if case_name.startswith(("P2-query", "cache-pressure-query")):
        return median(values, "query_ops_per_second", "read_operations_per_second")
    if "mixed-" in case_name:
        return median(values, "aggregate_ops_per_second", "logical_tx_per_second")
    return median(values, "logical_tx_per_second")


def number_list(records, selector):
    return [value for value in (selector(record) for record in records) if isinstance(value, (int, float))]


def counter_total(records, *keys):
    observed = []
    for record in records:
        value = median(measured(record), *keys)
        if value is None:
            value = median(record, *keys)
        if isinstance(value, (int, float)):
            observed.append(value)
    return sum(observed) if observed else None


grouped = {}
replacement_engines = set(arguments.replace_engine)
for results_path in arguments.results + arguments.replacement_results:
    is_replacement = results_path in arguments.replacement_results
    environment = json.loads((results_path / "environment.json").read_text(encoding="utf-8"))
    binary_hashes = {}
    for line in (results_path / "run-order.jsonl").read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        event = json.loads(line)
        if event.get("event") == "start":
            binary_hashes[(event["case"], event["engine"], event["repetition"])] = event.get("binary_sha256")
    for artifact in sorted((results_path / "raw").glob("*.jsonl")):
        artifact_name = artifact.stem.rsplit("-rep", 1)[0]
        repetition = int(artifact.stem.rsplit("-rep", 1)[1])
        engine = next((name for name in engine_names if artifact_name.endswith("-" + name)), None)
        if engine is None:
            continue
        if is_replacement and engine not in replacement_engines:
            continue
        if not is_replacement and engine in replacement_engines:
            continue
        case_name = artifact_name[: -(len(engine) + 1)]
        if "product-baseline-e5ce8530-rerun" in results_path.name and not case_name.startswith(("P1-", "P2-", "P3-", "P4-")):
            continue
        lines = [line for line in artifact.read_text(encoding="utf-8").splitlines() if line.strip()]
        if not lines:
            continue
        record = json.loads(lines[-1])
        binary_hash = binary_hashes.get((case_name, engine, repetition))
        grouped.setdefault((case_name, engine), []).append((record, artifact, environment, binary_hash))

summaries = []
for (case_name, engine), repetitions in sorted(grouped.items()):
    records = [entry[0] for entry in repetitions]
    rates = number_list(records, lambda record: throughput(case_name, record))
    metric_fields = {
        "read_ops_per_second": ("read_ops_per_second", "read_operations_per_second"),
        "write_tx_per_second": ("logical_tx_per_second",),
        "mutation_ops_per_second": ("mutation_ops_per_second",),
        "query_rows_per_second": ("returned_rows_per_second", "query_rows_per_second"),
        "p50_us": ("e2e_p50_us", "all_outcomes_p50_us", "p50_us"),
        "p95_us": ("e2e_p95_us", "all_outcomes_p95_us", "p95_us"),
        "p99_us": ("e2e_p99_us", "all_outcomes_p99_us", "p99_us"),
        "read_p50_us": ("read_p50_us",),
        "read_p95_us": ("read_p95_us",),
        "read_p99_us": ("read_p99_us",),
        "write_p50_us": ("write_p50_us",),
        "write_p95_us": ("write_p95_us",),
        "write_p99_us": ("write_p99_us",),
        "transactions_per_sync": ("transactions_per_sync",),
        "wal_bytes_delta": ("wal_bytes_delta",),
        "attempted_tx_per_second": ("attempted_tx_per_second",),
        "cpu_utilization_percent_machine": ("cpu_utilization_percent_machine",),
    }
    metrics = {}
    for output_name, keys in metric_fields.items():
        samples = []
        for record in records:
            sample = median(measured(record), *keys)
            if sample is None:
                sample = median(record, *keys)
            if isinstance(sample, (int, float)):
                samples.append(sample)
        metrics[output_name] = statistics.median(samples) if samples else None
    summaries.append({
        "case": case_name,
        "category": category(case_name),
        "engine": engine,
        "repetitions": len(records),
        "throughput_per_second": statistics.median(rates) if rates else None,
        "throughput_min_per_second": min(rates) if rates else None,
        "throughput_max_per_second": max(rates) if rates else None,
        "metrics": metrics,
        "errors_total": counter_total(records, "errors"),
        "conflicts_total": counter_total(records, "conflicts"),
        "retries_total": counter_total(records, "retries"),
        "wal_syncs_total": counter_total(records, "wal_syncs_delta"),
        "overloads_total": counter_total(records, "overloads"),
        "source_shas": sorted({entry[2]["git_commit"] for entry in repetitions}),
        "binary_sha256": sorted({entry[3] for entry in repetitions if entry[3]}),
        "raw_artifacts": [f"{entry[1].parent.parent.name}/raw/{entry[1].name}" for entry in repetitions],
    })

throughput_by_case = {
    summary["case"]: summary["throughput_per_second"]
    for summary in summaries
    if summary["engine"] == "main-btree"
}
for summary in summaries:
    main_throughput = throughput_by_case.get(summary["case"])
    if summary["engine"] == "parallel-blink" and main_throughput:
        summary["candidate_main_ratio"] = summary["throughput_per_second"] / main_throughput
    else:
        summary["candidate_main_ratio"] = None

arguments.output.parent.mkdir(parents=True, exist_ok=True)
arguments.output.write_text(json.dumps({"summaries": summaries}, indent=2) + "\n", encoding="utf-8")
if arguments.csv_output:
    csv_fields = [
        "case", "category", "engine", "throughput_unit", "throughput_per_second",
        "throughput_min_per_second", "throughput_max_per_second", "read_ops_per_second",
        "write_tx_per_second", "mutation_ops_per_second", "query_rows_per_second", "p50_us",
        "p95_us", "p99_us", "read_p50_us", "read_p95_us", "read_p99_us", "write_p50_us",
        "write_p95_us", "write_p99_us",
        "cpu_utilization_percent_machine", "errors", "conflicts", "retries", "overloads",
        "wal_syncs", "transactions_per_sync", "wal_bytes", "candidate_main_ratio", "repetitions",
        "source_sha", "binary_sha256", "raw_artifacts",
    ]
    arguments.csv_output.parent.mkdir(parents=True, exist_ok=True)
    with arguments.csv_output.open("w", encoding="utf-8", newline="") as output_file:
        writer = csv.DictWriter(output_file, fieldnames=csv_fields, lineterminator="\n")
        writer.writeheader()
        for summary in summaries:
            metrics = summary["metrics"]
            case_name = summary["case"]
            if case_name.startswith("P1-get-") or "-get-" in case_name:
                throughput_unit = "ops/s"
            elif case_name.startswith(("P2-query", "cache-pressure-query")):
                throughput_unit = "queries/s"
            elif "mixed-" in case_name:
                throughput_unit = "ops/s"
            else:
                throughput_unit = "tx/s"
            writer.writerow({
                "case": case_name,
                "category": summary["category"],
                "engine": summary["engine"],
                "throughput_unit": throughput_unit,
                "throughput_per_second": summary["throughput_per_second"],
                "throughput_min_per_second": summary["throughput_min_per_second"],
                "throughput_max_per_second": summary["throughput_max_per_second"],
                "read_ops_per_second": metrics["read_ops_per_second"],
                "write_tx_per_second": metrics["write_tx_per_second"],
                "mutation_ops_per_second": metrics["mutation_ops_per_second"],
                "query_rows_per_second": metrics["query_rows_per_second"],
                "p50_us": metrics["p50_us"],
                "p95_us": metrics["p95_us"],
                "p99_us": metrics["p99_us"],
                "read_p50_us": metrics["read_p50_us"],
                "read_p95_us": metrics["read_p95_us"],
                "read_p99_us": metrics["read_p99_us"],
                "write_p50_us": metrics["write_p50_us"],
                "write_p95_us": metrics["write_p95_us"],
                "write_p99_us": metrics["write_p99_us"],
                "cpu_utilization_percent_machine": metrics["cpu_utilization_percent_machine"],
                "errors": summary["errors_total"],
                "conflicts": summary["conflicts_total"],
                "retries": summary["retries_total"],
                "overloads": summary["overloads_total"],
                "wal_syncs": summary["wal_syncs_total"],
                "transactions_per_sync": metrics["transactions_per_sync"],
                "wal_bytes": metrics["wal_bytes_delta"],
                "candidate_main_ratio": summary["candidate_main_ratio"],
                "repetitions": summary["repetitions"],
                "source_sha": ";".join(summary["source_shas"]),
                "binary_sha256": ";".join(summary["binary_sha256"]),
                "raw_artifacts": ";".join(summary["raw_artifacts"]),
            })
