import json
import os
import pathlib
import subprocess
import sys
import time

source_directory = pathlib.Path("/bench/zfs/db/oci-a1-query-cursor-95954ecaa/candidate")
base_source_directory = pathlib.Path("/bench/zfs/db/oci-a1-query-cursor-95954ecaa/base")
result_directory = pathlib.Path("/bench/zfs/db/oci-a1-query-cursor-95954ecaa/results")
sys.path.insert(0, str(source_directory / "dodb/experiments/mixed-harness"))
sys.dont_write_bytecode = True
import run_read_metrics_matrix as matrix

source_sha = "95954ecaaae94757cc3705bc2aacf8c6bb78915f"
variants = ("main-btree", "blink-metrics-on", "blink-candidate")
matrix.VARIANT_SPECS["blink-candidate"] = {**matrix.VARIANT_SPECS["blink-metrics-on"], "binary_key": "phase0-candidate", "binary_env": "PHASE0_CANDIDATE_BINARY"}
run_orders = {
    1: ("main-btree", "blink-metrics-on", "blink-candidate"),
    2: ("blink-candidate", "main-btree", "blink-metrics-on"),
    3: ("blink-metrics-on", "blink-candidate", "main-btree"),
}


def sampled_process(command, cwd, environment, output_stream):
    if not command:
        raise matrix.RunnerError("refusing to execute an empty command")
    process = subprocess.Popen(command, cwd=cwd, env=environment, stdout=output_stream, stderr=subprocess.STDOUT)
    sample_records = []
    collection_failures = 0
    prior_sample_ns = None
    maximum_sample_gap_ns = 0
    while True:
        observed_ns = time.monotonic_ns()
        try:
            status_text = pathlib.Path(f"/proc/{process.pid}/status").read_text(encoding="utf-8")
            status_values = {}
            for status_line in status_text.splitlines():
                if status_line.startswith("VmRSS:"):
                    status_values["end"] = int(status_line.split()[1])
                elif status_line.startswith("VmHWM:"):
                    status_values["hwm"] = int(status_line.split()[1])
            if len(status_values) == 2:
                sample_records.append({"monotonic_ns": observed_ns, **status_values})
                if prior_sample_ns is not None:
                    maximum_sample_gap_ns = max(maximum_sample_gap_ns, observed_ns - prior_sample_ns)
                prior_sample_ns = observed_ns
            else:
                collection_failures += 1
        except OSError:
            if process.poll() is None:
                collection_failures += 1
        return_code = process.poll()
        if return_code is not None:
            break
        time.sleep(0.02)
    return_code = process.wait()
    if not sample_records:
        raise matrix.RunnerError(f"RSS sampling produced no successful samples for pid={process.pid}")
    process_rss = {
        "end": sample_records[-1]["end"],
        "hwm": max(sample["hwm"] for sample in sample_records),
        "sample_count": len(sample_records),
        "collection_failures": collection_failures,
        "maximum_sample_gap_ns": maximum_sample_gap_ns,
        "sampling_interval_target_ns": 20_000_000,
    }
    return return_code, process_rss


matrix.run_process_with_rss = sampled_process
original_build_command = matrix.build_command
original_validate_raw_record = matrix.validate_raw_record

def validate_source_record(record, cell, checkout_sha):
    source_sha = source_sha_candidate if cell["variant"] == "blink-candidate" else source_sha_base
    return original_validate_raw_record(record, cell, source_sha)

matrix.validate_raw_record = validate_source_record
source_sha_base = "4bc4d42e4d816f4a11428f5423d89a4c35494f9b"
source_sha_candidate = "95954ecaaae94757cc3705bc2aacf8c6bb78915f"


def build_command_with_zfs_tmp(arguments, cell, binary_path, output_path, data_directory):
    command, environment = original_build_command(arguments, cell, binary_path, output_path, data_directory)
    environment["TMPDIR"] = str(result_directory / "tmp")
    source = source_directory if cell["variant"] == "blink-candidate" else base_source_directory
    environment["GIT_DIR"] = str(source / ".git")
    environment["GIT_WORK_TREE"] = str(source)
    return command, environment


matrix.build_command = build_command_with_zfs_tmp
arguments = matrix.parse_arguments([
    "--results", str(result_directory),
    "--data-root", "/bench/zfs/db/oci-a1-query-cursor-95954ecaa/datasets",
    "--readers", "1",
    "--read-kinds", "get,query",
    "--variants", ",".join(variants),
    "--repetitions", "3",
    "--warmup-ms", "1000",
    "--duration-ms", "5000",
    "--working-set", "4096",
    "--key-size", "16",
    "--value-size", "64",
    "--read-limit", "16",
    "--cache-capacity", "256",
])
os.environ["PHASE0_BINARY"] = "/bench/zfs/db/oci-a1-query-cursor-95954ecaa/target-base/aarch64-unknown-linux-gnu/release/phase0-bench"
os.environ["PHASE0_CANDIDATE_BINARY"] = "/bench/zfs/db/oci-a1-query-cursor-95954ecaa/target-candidate/aarch64-unknown-linux-gnu/release/phase0-bench"
repository = source_directory
source_state_before = matrix.require_clean_source(repository, result_directory)
if source_state_before["commit"] != source_sha_candidate:
    raise matrix.RunnerError(f"candidate source SHA mismatch: {source_state_before['commit']}")
base_source_state_before = matrix.require_clean_source(base_source_directory, result_directory)
if base_source_state_before["commit"] != source_sha_base:
    raise matrix.RunnerError(f"base source SHA mismatch: {base_source_state_before['commit']}")
if arguments.results.joinpath("raw").exists() or arguments.results.joinpath("run-order.jsonl").exists():
    raise matrix.RunnerError("run output paths already exist")
if (arguments.data_root / arguments.results.name).exists():
    raise matrix.RunnerError("ZFS benchmark data namespace already exists")
arguments.data_root.mkdir(parents=True, exist_ok=True)
expected_cells = matrix.build_expected_cells(arguments, source_sha)
for cell in expected_cells:
    cell["seed"] = 0xD0DB2025 + cell["repetition"]
arguments.results.joinpath("raw").mkdir()
expected_variants = []
for variant in variants:
    variant_spec = matrix.VARIANT_SPECS[variant]
    expected_variants.append({
        "name": variant,
        "engine": variant_spec["engine"],
        "collection_policy": variant_spec["collection_policy"],
        "binary_key": variant_spec["binary_key"],
        "binary_env": variant_spec["binary_env"],
        "binary_kind": variant_spec["binary_kind"],
        "read_metrics_mode": variant_spec["read_metrics_mode"],
        "blink_read_observational_metrics_enabled": variant_spec["blink_read_observational_metrics_enabled"],
        "blink_workers": variant_spec.get("blink_workers"),
        "tokio_workers": variant_spec.get("tokio_workers"),
        "engine_options": list(variant_spec["engine_options"]),
    })
configuration = {
    "schema_version": 1,
    "created_at_utc": matrix.timestamp_utc(),
    "source_commit": source_sha,
    "source_branch": source_state_before["branch"],
    "planned_count": len(expected_cells),
    "config": {
        "readers": 1,
        "tokio_workers": 2,
        "reader_execution_models": {"phase0": "one async reader task on two Tokio workers", "rocksdb": "not used"},
        "read_kinds": ["get", "query"],
        "variants": expected_variants,
        "repetitions": 3,
        "warmup_ms": 1000,
        "duration_ms": 5000,
        "transaction_width": 1,
        "distribution": "uniform",
        "working_set": 4096,
        "key_size": 16,
        "value_size": 64,
        "read_limit": 16,
        "cache_capacity": 256,
        "sync_mode": "real",
        "sync_contract": "durable-return",
        "seeds": [0xD0DB2026, 0xD0DB2027, 0xD0DB2028],
        "data_root": str(arguments.data_root),
        "data_namespace": str(arguments.data_root / arguments.results.name),
        "requested_order_by_repetition": {str(repetition): list(order) for repetition, order in run_orders.items()},
    },
    "expected_cells": expected_cells,
}
matrix.write_json(arguments.results / "matrix-config.json", configuration)
matrix.write_json(arguments.results / "source-provenance.json", {"candidate_source_before": source_state_before, "base_source_before": base_source_state_before, "source_after": None})
matrix.write_json(arguments.results / "source-shas-by-variant.json", {"main-btree": source_sha_base, "blink-metrics-on": source_sha_base, "blink-candidate": source_sha_candidate})
matrix.write_json(arguments.results / "environment.json", matrix.environment_record(repository, arguments.data_root))
registry = {}
for variant in variants:
    variant_spec = matrix.VARIANT_SPECS[variant]
    probe_cell = next(cell for cell in expected_cells if cell["variant"] == variant)
    binary_path = matrix.binary_path_for(probe_cell)
    registry[variant] = {"path": str(binary_path), "sha256": matrix.sha256_file(binary_path), "environment_variable": variant_spec["binary_env"]}
matrix.write_json(arguments.results / "binary-provenance.json", registry)
completed_count = 0
for read_kind in ("get", "query"):
    for repetition in range(1, 4):
        for variant in run_orders[repetition]:
            cell = next(item for item in expected_cells if item["read_kind"] == read_kind and item["repetition"] == repetition and item["variant"] == variant)
            if not cell:
                raise matrix.RunnerError("planned execution cell is missing")
            matrix.execute_cell(arguments, arguments.results, repository, cell, {}, source_state_before)
            completed_count += 1
source_state_after = matrix.require_clean_source(repository, arguments.results)
base_source_state_after = matrix.require_clean_source(base_source_directory, result_directory)
if source_state_after["commit"] != source_sha_candidate or base_source_state_after["commit"] != source_sha_base:
    raise matrix.RunnerError("source changed during the benchmark matrix")
matrix.write_json(arguments.results / "source-provenance.json", {"candidate_source_before": source_state_before, "base_source_before": base_source_state_before, "candidate_source_after": source_state_after, "base_source_after": base_source_state_after})

raw_records = {}
for cell in expected_cells:
    raw_path = arguments.results / "raw" / f"{cell['cell_id']}.jsonl"
    record = matrix.load_json_lines(raw_path)[0]
    if record.get("successful_reads", 0) < 1:
        raise matrix.ValidationError(f"non-positive completion count in {cell['cell_id']}")
    if record.get("errors") != 0 or record.get("overloads") != 0 or record.get("conflicts") != 0:
        raise matrix.ValidationError(f"error, overload, or conflict gate failed in {cell['cell_id']}")
    if record.get("blink_borrowed_page_views_enabled") is not False:
        raise matrix.ValidationError(f"feature state mismatch in {cell['cell_id']}")
    if record.get("blink_read_observational_metrics_enabled") is not True:
        raise matrix.ValidationError(f"read observations are disabled in {cell['cell_id']}")
    if cell["expected_engine"] != "main-btree" and record.get("active_generation_pins") != 0:
        raise matrix.ValidationError(f"active generation pin remains in {cell['cell_id']}")
    process_rss = record.get("rss_kib")
    if not isinstance(process_rss, dict) or process_rss.get("sample_count", 0) < 1:
        raise matrix.ValidationError(f"incomplete process RSS sampling in {cell['cell_id']}")
    if cell["read_kind"] == "query":
        if record.get("query_checked_requests") != record.get("successful_reads"):
            raise matrix.ValidationError(f"Query request validation mismatch in {cell['cell_id']}")
        if record.get("query_validation_failures") != 0:
            raise matrix.ValidationError(f"Query response check failed in {cell['cell_id']}")
        if record.get("query_client_aggregation_passed") is not True or record.get("client_aggregation_passed") is not True:
            raise matrix.ValidationError(f"Query client aggregation failed in {cell['cell_id']}")
    if cell["read_kind"] == "get" and record.get("sampled_read_verification_status") != "passed":
        raise matrix.ValidationError(f"Get value and revision verification failed in {cell['cell_id']}")
    raw_records[(cell["read_kind"], cell["repetition"], cell["variant"])] = record
fingerprint_checks = []
for repetition in range(1, 4):
    fingerprint_maps = {}
    for variant in variants:
        record = raw_records[("query", repetition, variant)]
        client_records = record.get("clients")
        if not isinstance(client_records, list) or len(client_records) != 1:
            raise matrix.ValidationError(f"Query client detail missing for rep{repetition} {variant}")
        query_fingerprint = client_records[0].get("query_input_fingerprint")
        checkpoints = query_fingerprint.get("checkpoints") if isinstance(query_fingerprint, dict) else None
        if not isinstance(checkpoints, list):
            raise matrix.ValidationError(f"Query fingerprints missing for rep{repetition} {variant}")
        fingerprint_maps[variant] = {point["request_count"]: point["fingerprint"] for point in checkpoints}
    common_counts = set.intersection(*(set(fingerprint_maps[variant]) for variant in variants))
    if not common_counts:
        raise matrix.ValidationError(f"no same-length common Query input checkpoints for rep{repetition}")
    for request_count in sorted(common_counts):
        fingerprints = {variant: fingerprint_maps[variant][request_count] for variant in variants}
        if len(set(fingerprints.values())) != 1:
            raise matrix.ValidationError(f"Query input fingerprint mismatch at {request_count} requests in rep{repetition}")
        fingerprint_checks.append({"repetition": repetition, "seed": 0xD0DB2025 + repetition, "request_count": request_count, "fingerprint": next(iter(fingerprints.values()))})
status_record = {"accepted": True, "planned_count": 18, "completed_count": completed_count, "source_commit": source_sha, "state": "complete", "query_common_fingerprint_checkpoints": len(fingerprint_checks)}
matrix.write_json(arguments.results / "matrix-status.json", status_record)
matrix.write_json(arguments.results / "query-fingerprint-validation.json", {"passed": True, "checks": fingerprint_checks})
print(json.dumps(status_record, sort_keys=True))
