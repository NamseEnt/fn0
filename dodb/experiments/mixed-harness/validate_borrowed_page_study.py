import argparse
import hashlib
import json
import pathlib
import re
import statistics

from run_read_metrics_matrix import validate_matrix


VARIANTS = ("blink-metrics-on", "blink-borrowed-pages", "main-btree")
SOURCE_COMMIT = "b7c67715c2dc52d30fce43fcb5d885fedd070547"
RECORDED_RESULTS_ROOT = pathlib.Path("/bench/zfs/db/experiment-results/borrowed-page-b7c67715-accepted")


def require(condition, message):
    if not condition:
        raise ValueError(message)


def load_json(path):
    return json.loads(path.read_text())


def paired_ratio(numerators, denominators):
    require(set(numerators) == set(denominators), "paired seed sets differ")
    ratios = [numerators[seed] / denominators[seed] for seed in sorted(numerators)]
    return {"ratios": ratios, "median": statistics.median(ratios), "min": min(ratios), "max": max(ratios)}


def validate_study(results):
    build_hashes = {}
    for line in (results / "build-proof" / "binary-sha256.txt").read_text().splitlines():
        binary_hash, binary_path = line.split(maxsplit=1)
        build_hashes[pathlib.Path(binary_path).name] = binary_hash
    require(set(build_hashes) == {"phase0-default", "phase0-borrowed"}, "build hash set differs")
    provenance = (results / "build-proof" / "provenance.txt").read_text()
    for field_name in ("source_sha_before", "source_sha_after"):
        require(f"{field_name}={SOURCE_COMMIT}" in provenance, f"invalid {field_name}")
    for field_name in ("source_status_before", "source_status_after"):
        require(f"{field_name}=clean" in provenance, f"invalid {field_name}")
    native_tests = (results / "build-proof" / "build-and-tests.log").read_text()
    require("177 passed; 0 failed; 4 ignored" in native_tests, "native correctness suite did not pass")
    rates = {kind: {variant: {} for variant in VARIANTS} for kind in ("get", "query")}
    case_counts = {}
    raw_hashes = {}
    for matrix_name, repetitions, warmup_ms, duration_ms in (
        ("smoke", 1, 200, 300),
        ("read-c16", 3, 2000, 5000),
    ):
        matrix_root = results / matrix_name
        validate_matrix(matrix_root, recorded_results_root=RECORDED_RESULTS_ROOT / matrix_name)
        config = load_json(matrix_root / "matrix-config.json")
        require(config["source_commit"] == SOURCE_COMMIT, "matrix source mismatch")
        require(config["planned_count"] == 6 * repetitions, "matrix case count mismatch")
        require(config["config"]["repetitions"] == repetitions, "matrix repetitions mismatch")
        environment = load_json(matrix_root / "environment.json")
        require(environment["architecture"] == "aarch64" and environment["logical_cpus"] == 2, "host shape mismatch")
        require(environment["hostname"] == "instance-20260923-1013", "host identity mismatch")
        require(environment["zfs_dataset"] == "dodbbench/db", "dataset mismatch")
        for expression in (r"sync\s+standard", r"recordsize\s+4K", r"compression\s+off"):
            require(re.search(expression, environment["zfs_properties"]), "ZFS property mismatch")
        registry = load_json(matrix_root / "binary-provenance.json")
        require(registry["phase0-bench"]["sha256"] == build_hashes["phase0-default"], "control binary hash mismatch")
        require(registry["phase0-borrowed-pages"]["sha256"] == build_hashes["phase0-borrowed"], "candidate binary hash mismatch")
        case_counts[matrix_name] = len(config["expected_cells"])
        for cell in config["expected_cells"]:
            path = matrix_root / "raw" / (cell["cell_id"] + ".jsonl")
            record = json.loads(path.read_text())
            require(record["git_commit"] == SOURCE_COMMIT, "raw source mismatch")
            require(record["warmup_ms"] == warmup_ms and record["requested_duration_ms"] == duration_ms, "timing mismatch")
            borrowed = cell["variant"] == "blink-borrowed-pages"
            require(record["blink_borrowed_page_views_enabled"] == borrowed, "borrowed feature mismatch")
            require(record["blink_read_observational_metrics_enabled"] is True, "read metrics disabled")
            for field_name in ("read_p50_us", "read_p95_us", "read_p99_us"):
                require(record[field_name] > 0, f"invalid {field_name}")
            if cell["variant"] != "main-btree":
                require(record["active_generation_pins"] == 0, "leaked generation pin")
                require(record["generation_pins"] == record["successful_reads"], "pin count mismatch")
                require(record["read_operations_metric"] == record["successful_reads"], "read metric count mismatch")
            if cell["read_kind"] == "query":
                require(record["returned_rows"] == 16 * record["successful_reads"], "Query(16) row count mismatch")
            if matrix_name == "read-c16":
                rates[cell["read_kind"]][cell["variant"]][cell["seed"]] = record[cell["read_kind"] + "_ops_per_second"]
            raw_hashes[str(path.relative_to(results))] = hashlib.sha256(path.read_bytes()).hexdigest()
    profile_count = 0
    for read_kind in ("get", "query"):
        for mode in ("default", "borrowed"):
            profile_path = results / "profiles" / f"{read_kind}-{mode}.jsonl"
            record = json.loads(profile_path.read_text())
            require(record["git_commit"] == SOURCE_COMMIT, "profile source mismatch")
            require(record["errors"] == 0 and record["successful_reads"] > 0, "profile workload failed")
            require(record["blink_borrowed_page_views_enabled"] == (mode == "borrowed"), "profile feature mismatch")
            report = profile_path.with_suffix(".report.txt").read_text()
            require("Samples:" in report and re.search(r"dodb_storage.*blink", report), "profile has no Blink samples")
            compressed = profile_path.with_suffix(".perf.data.gz")
            require(compressed.stat().st_size > 0, "empty profile data")
            profile_count += 1
    ratios = {}
    for read_kind, variants in rates.items():
        ratios[read_kind] = {
            "candidate_over_control": paired_ratio(variants["blink-borrowed-pages"], variants["blink-metrics-on"]),
            "candidate_over_main": paired_ratio(variants["blink-borrowed-pages"], variants["main-btree"]),
            "control_over_main": paired_ratio(variants["blink-metrics-on"], variants["main-btree"]),
        }
    return {
        "accepted": True,
        "source_commit": SOURCE_COMMIT,
        "case_counts": case_counts,
        "profile_count": profile_count,
        "binary_hashes": build_hashes,
        "paired_ratios": ratios,
        "read_product_gate_passed": all(row["candidate_over_main"]["median"] >= 0.9 for row in ratios.values()),
        "raw_sha256": raw_hashes,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("results", type=pathlib.Path)
    parser.add_argument("--output", type=pathlib.Path)
    arguments = parser.parse_args()
    result = validate_study(arguments.results.resolve())
    encoded = json.dumps(result, indent=2) + "\n"
    if arguments.output:
        arguments.output.write_text(encoded)
    print(encoded)


if __name__ == "__main__":
    main()
