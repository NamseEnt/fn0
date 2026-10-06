import argparse
import hashlib
import json
import pathlib
import re
import statistics

from run_read_metrics_matrix import validate_matrix as validate_reads
from summarize_payload_batching import metric_value, validate_matrix as validate_mixed


READ_VARIANTS = ("main-btree", "main-btree-compact-wal", "blink-borrowed-pages")
MIXED_VARIANTS = ("main-btree", "main-btree-compact-wal", "parallel-blink-main-parity", "parallel-blink-main-parity-borrowed")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def load_json(path):
    return json.loads(path.read_text())


def paired_ratio(numerator, denominator):
    require(set(numerator) == set(denominator), "paired seed sets differ")
    ratios = [numerator[seed] / denominator[seed] for seed in sorted(numerator)]
    return {"ratios": ratios, "median": statistics.median(ratios), "min": min(ratios), "max": max(ratios)}


def validate_study(results, recorded_root, source_commit):
    provenance = (results / "build-proof" / "provenance.txt").read_text()
    for direction in ("before", "after"):
        require(f"source_sha_{direction}={source_commit}" in provenance, "build source mismatch")
        require(f"source_status_{direction}=clean" in provenance, "unclean build source")
    tests = (results / "build-proof" / "build-and-tests.log").read_text()
    require("181 passed; 0 failed; 4 ignored" in tests, "native storage tests failed")
    require("3 passed; 0 failed; 0 ignored" in tests, "native compact crash tests failed")
    binary_hashes = {}
    for line in (results / "build-proof" / "binary-sha256.txt").read_text().splitlines():
        binary_hash, binary_path = line.split(maxsplit=1)
        binary_hashes[pathlib.Path(binary_path).name] = binary_hash
    require(set(binary_hashes) == {"phase0-default", "phase0-borrowed"}, "unexpected build binaries")
    read_rates = {kind: {variant: {} for variant in READ_VARIANTS} for kind in ("get", "query")}
    read_metrics = {}
    mixed_rates = {clients: {variant: {} for variant in MIXED_VARIANTS} for clients in (4, 64)}
    mixed_metrics = {}
    resource_fields_present = {"cpu_utilization_percent_one_core": True, "rss_end_kib": True, "rss_hwm_kib": True}
    raw_hashes = {}
    case_counts = {}
    for matrix_name, is_read, expected_count, repetitions, warmup_ms, duration_ms in (
        ("read-smoke", True, 6, 1, 200, 300),
        ("mixed-smoke", False, 8, 1, 200, 300),
        ("read-c16", True, 18, 3, 2000, 10000),
        ("mixed-c4-c64", False, 24, 3, 2000, 5000),
    ):
        matrix_root = results / matrix_name
        validator = validate_reads if is_read else validate_mixed
        validator(matrix_root, recorded_results_root=recorded_root / matrix_name)
        config = load_json(matrix_root / "matrix-config.json")
        require(config["source_commit"] == source_commit, "matrix source mismatch")
        require(config["planned_count"] == expected_count, "matrix size mismatch")
        require(config["config"]["repetitions"] == repetitions, "matrix repetitions mismatch")
        environment = load_json(matrix_root / "environment.json")
        require(environment["architecture"] == "aarch64" and environment["logical_cpus"] == 2, "host shape mismatch")
        require(environment["hostname"] == "instance-20260923-1013", "host identity mismatch")
        require(environment["zfs_dataset"] == "dodbbench/db", "dataset mismatch")
        for expression in (r"sync\s+standard", r"recordsize\s+4K", r"compression\s+off"):
            require(re.search(expression, environment["zfs_properties"]), "ZFS property mismatch")
        starts = {}
        for line in (matrix_root / "run-order.jsonl").read_text().splitlines():
            event = json.loads(line)
            if event["event"] == "start":
                starts[event["cell_id"]] = event
        case_counts[matrix_name] = len(config["expected_cells"])
        for cell in config["expected_cells"]:
            path = matrix_root / "raw" / (cell["cell_id"] + ".jsonl")
            record = json.loads(path.read_text())
            require(record["git_commit"] == source_commit, "raw source mismatch")
            require(record["warmup_ms"] == warmup_ms and record["requested_duration_ms"] == duration_ms, "timing mismatch")
            compact = cell["variant"] == "main-btree-compact-wal"
            borrowed = cell["variant"] in ("blink-borrowed-pages", "parallel-blink-main-parity-borrowed")
            require(record["main_compact_wal_enabled"] == compact, "compact feature mismatch")
            require(record["blink_borrowed_page_views_enabled"] == borrowed, "borrowed feature mismatch")
            require(record["blink_read_observational_metrics_enabled"] is True, "disabled read metrics")
            binary_name = "phase0-borrowed" if borrowed else "phase0-default"
            require(starts[cell["cell_id"]]["binary_sha256"] == binary_hashes[binary_name], "run binary mismatch")
            for name in resource_fields_present:
                resource_fields_present[name] &= metric_value(record, name) is not None
            if is_read:
                for field_name in ("read_p50_us", "read_p95_us", "read_p99_us"):
                    require(record[field_name] > 0, f"missing {field_name}")
                if borrowed:
                    require(record["active_generation_pins"] == 0, "leaked generation pin")
                    require(record["generation_pins"] == record["successful_reads"], "pin count mismatch")
                    require(record["read_operations_metric"] == record["successful_reads"], "read metric mismatch")
                if cell["read_kind"] == "query":
                    require(record["returned_rows"] == 16 * record["successful_reads"], "Query(16) row count mismatch")
                if matrix_name == "read-c16":
                    read_rates[cell["read_kind"]][cell["variant"]][cell["seed"]] = record[cell["read_kind"] + "_ops_per_second"]
                    group = read_metrics.setdefault(f"{cell['read_kind']}-{cell['variant']}", {})
                    for name in ("read_p99_us", "cpu_utilization_percent_one_core"):
                        group.setdefault(name, []).append(metric_value(record, name))
            else:
                require(record["mixed_value_mode"] == "changing", "constant mixed values")
                if cell["variant"] != "main-btree":
                    require(record["wal_page_delta_records_delta"] > 0, "compact WAL emitted no deltas")
                    require(record["wal_page_image_records_delta"] == 0, "unexpected full page images in measured compact WAL")
                    require(record["wal_image_superblock_delta"] == 0, "unexpected metadata anchors in measured compact WAL")
                else:
                    require(record["wal_page_delta_records_delta"] == 0, "full-image control emitted deltas")
                    require(record["wal_page_image_records_delta"] > 0, "full-image control emitted no page images")
                for field_name in ("read_p50_us", "read_p95_us", "read_p99_us", "write_p50_us", "write_p95_us", "write_p99_us"):
                    require(record[field_name] > 0, f"missing {field_name}")
                if matrix_name == "mixed-c4-c64":
                    mixed_rates[cell["clients"]][cell["variant"]][cell["seed"]] = metric_value(record, "aggregate_ops_per_second")
                    group = mixed_metrics.setdefault(f"c{cell['clients']}-{cell['variant']}", {})
                    for metric_name in ("aggregate_ops_per_second", "avg_transactions_per_group", "write_p99_us", "read_p99_us", "cpu_utilization_percent_one_core"):
                        group.setdefault(metric_name, []).append(metric_value(record, metric_name))
                    group.setdefault("wal_bytes_per_successful_write", []).append(record["wal_bytes_delta"] / record["successful_write_transactions"])
            raw_hashes[str(path.relative_to(results))] = hashlib.sha256(path.read_bytes()).hexdigest()
    read_ratios = {}
    for kind, variants in read_rates.items():
        read_ratios[kind] = {
            "compact_main_over_main": paired_ratio(variants["main-btree-compact-wal"], variants["main-btree"]),
            "borrowed_blink_over_main": paired_ratio(variants["blink-borrowed-pages"], variants["main-btree"]),
            "borrowed_blink_over_compact_main": paired_ratio(variants["blink-borrowed-pages"], variants["main-btree-compact-wal"]),
        }
    mixed_ratios = {}
    for clients, variants in mixed_rates.items():
        mixed_ratios[str(clients)] = {
            "compact_main_over_main": paired_ratio(variants["main-btree-compact-wal"], variants["main-btree"]),
            "blink_over_compact_main": paired_ratio(variants["parallel-blink-main-parity"], variants["main-btree-compact-wal"]),
            "borrowed_blink_over_compact_main": paired_ratio(variants["parallel-blink-main-parity-borrowed"], variants["main-btree-compact-wal"]),
            "borrowed_blink_over_blink": paired_ratio(variants["parallel-blink-main-parity-borrowed"], variants["parallel-blink-main-parity"]),
        }
    summarized_metrics = {}
    for group_key, metrics in {**read_metrics, **mixed_metrics}.items():
        summarized_metrics[group_key] = {}
        for name, values in metrics.items():
            require(all(value is not None for value in values), f"missing mixed metric {name}")
            summarized_metrics[group_key][name] = {"values": values, "median": statistics.median(values)}
    return {
        "accepted": True,
        "source_commit": source_commit,
        "case_counts": case_counts,
        "binary_hashes": binary_hashes,
        "read_paired_ratios": read_ratios,
        "mixed_paired_ratios": mixed_ratios,
        "read_metrics": {key: value for key, value in summarized_metrics.items() if key in read_metrics},
        "mixed_metrics": {key: value for key, value in summarized_metrics.items() if key in mixed_metrics},
        "resource_fields_present_in_all_cases": resource_fields_present,
        "borrowed_blink_read_gate_passed": all(row["borrowed_blink_over_main"]["median"] >= 0.9 for row in read_ratios.values()),
        "raw_sha256": raw_hashes,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("results", type=pathlib.Path)
    parser.add_argument("--source-commit", required=True)
    parser.add_argument("--recorded-results-root", required=True, type=pathlib.Path)
    parser.add_argument("--output", required=True, type=pathlib.Path)
    arguments = parser.parse_args()
    result = validate_study(arguments.results.resolve(), arguments.recorded_results_root, arguments.source_commit)
    arguments.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({key: value for key, value in result.items() if key not in ("raw_sha256", "mixed_metrics", "read_metrics")}, indent=2))


if __name__ == "__main__":
    main()
