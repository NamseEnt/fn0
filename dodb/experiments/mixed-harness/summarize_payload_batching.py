import argparse
import csv
import hashlib
import json
import math
import pathlib
import statistics
import sys
from collections import defaultdict
from datetime import datetime, timezone
from itertools import product


SUMMARY_CSV_NAME = "payload-batching-summary.csv"
SUMMARY_JSON_NAME = "payload-batching-summary.json"
MANIFEST_NAME = "artifact-sha256.txt"
SUPPORTED_VARIANTS = {
    "main-btree": ("main-btree", "native-main", None),
    "parallel-blink-current": ("parallel-blink", "current", None),
    "parallel-blink-main-parity": ("parallel-blink", "main-parity", None),
    "parallel-blink-main-parity-workers1": ("parallel-blink", "main-parity", 1),
    "planned-blink-main-parity": ("planned-blink", "main-parity", None),
    "planned-blink-main-parity-workers0": ("planned-blink", "main-parity", None),
    "rocksdb": ("rocksdb", "not-applicable", None),
}
METRIC_NAMES = (
    "aggregate_ops_per_second",
    "read_ops_per_second",
    "write_tx_per_second",
    "read_p50_us",
    "read_p95_us",
    "read_p99_us",
    "write_p50_us",
    "write_p95_us",
    "write_p99_us",
    "successful_read_percent",
    "successful_write_percent",
    "errors",
    "conflicts",
    "overloads",
    "cpu_utilization_percent_one_core",
    "cpu_seconds",
    "rss_end_kib",
    "rss_hwm_kib",
    "avg_transactions_per_group",
    "avg_transactions_per_wal_sync",
    "wal_syncs_delta",
    "wal_bytes_delta",
    "wal_bytes_per_sync",
    "wal_sync_nanos_total",
    "wal_sync_average_us",
    "page_images_delta",
    "wal_page_image_records_delta",
    "wal_page_delta_spans_delta",
    "wal_page_delta_changed_bytes_delta",
    "superblock_images_emitted",
    "superblock_images_elided",
    "physical_execution_nanos",
    "planning_nanos",
    "mutations_planned",
    "coalesced_mutations",
    "parallel_workers",
    "blink_workers",
    "parallel_worker_dispatches_delta",
    "parallel_worker_nanos_total",
)
RATIO_METRICS = (
    "aggregate_ops_per_second",
    "read_ops_per_second",
    "write_tx_per_second",
)


class ValidationError(ValueError):
    pass


def is_lowercase_hex(value, expected_length):
    return (
        isinstance(value, str)
        and len(value) == expected_length
        and all(character in "0123456789abcdef" for character in value)
    )


def sha256_file(path):
    digest = hashlib.sha256()
    with path.open("rb") as input_file:
        for chunk in iter(lambda: input_file.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json_lines(path):
    try:
        lines = [line for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
        return [json.loads(line) for line in lines]
    except (OSError, json.JSONDecodeError) as error:
        raise ValidationError(f"cannot read JSONL {path}: {error}") from error


def read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValidationError(f"cannot read JSON {path}: {error}") from error


def nested_value(record, names):
    for name in names:
        if name in record and record[name] is not None:
            return record[name]
    measured = record.get("measured")
    if isinstance(measured, dict):
        for name in names:
            if name in measured and measured[name] is not None:
                return measured[name]
    return None


def required_value(record, name, aliases=()):
    names = (name, *aliases)
    for candidate in names:
        if candidate in record and record[candidate] is not None:
            return record[candidate]
    raise ValidationError(f"raw record is missing required field {name}")


def numeric_value(record, names):
    value = nested_value(record, names)
    if value is None or isinstance(value, bool):
        return None
    if not isinstance(value, (int, float)) or not math.isfinite(float(value)):
        return None
    return float(value)


def assert_equal(record, field_name, expected, aliases=()):
    actual = required_value(record, field_name, aliases)
    if actual != expected:
        raise ValidationError(
            f"raw {field_name} mismatch: actual={actual!r}, expected={expected!r}"
        )


def validate_raw_record(record, cell, checkout_sha):
    if not isinstance(record, dict):
        raise ValidationError(f"raw record must be an object: {cell['cell_id']}")
    if record.get("record_type") not in (None, "run", "crossdb-run"):
        raise ValidationError(
            f"unexpected record_type for {cell['cell_id']}: {record.get('record_type')!r}"
        )
    assert_equal(record, "engine", cell["expected_engine"])
    assert_equal(record, "client_workers", cell["clients"], ("writers",))
    assert_equal(record, "seed", cell["seed"])
    assert_equal(record, "transaction_width", cell["transaction_width"], ("width",))
    assert_equal(
        record,
        "requested_read_percent",
        cell["read_percent"],
        ("read_percent",),
    )
    assert_equal(record, "working_set", cell["working_set"])
    assert_equal(record, "key_size", cell["key_size"])
    assert_equal(record, "value_size", cell["value_size"])
    assert_equal(record, "distribution", cell["distribution"])
    assert_equal(record, "mixed_value_mode", cell["value_mode"])
    expected_generator = (
        "legacy_constant_v1" if cell["value_mode"] == "constant" else "seeded_nonrepeating_v1"
    )
    assert_equal(record, "mixed_value_generator", expected_generator)
    assert_equal(record, "logical_trace_prefix_operations", 1000)
    trace_hash = required_value(record, "logical_trace_prefix_hash")
    if not is_lowercase_hex(trace_hash, 16):
        raise ValidationError(f"invalid logical_trace_prefix_hash for {cell['cell_id']}")
    assert_equal(record, "sync_contract", "durable-return")
    assert_equal(record, "collection_policy", cell["expected_collection_policy"])
    if cell.get("expected_blink_workers") is not None:
        assert_equal(record, "blink_workers", cell["expected_blink_workers"])
    assert_equal(record, "git_commit", checkout_sha)
    assert_equal(record, "warmup_ms", cell["warmup_ms"])
    assert_equal(
        record,
        "requested_duration_ms",
        cell["duration_ms"],
        ("duration_ms",),
    )
    assert_equal(record, "repetition", cell["expected_raw_repetition"])

    workload = record.get("workload", record.get("operation", record.get("suite")))
    if workload != "mixed":
        raise ValidationError(f"raw workload is not mixed for {cell['cell_id']}: {workload!r}")
    sync_mode = record.get("sync_mode")
    if cell["expected_engine"] != "rocksdb" and sync_mode != "real":
        raise ValidationError(f"raw sync_mode is not real for {cell['cell_id']}: {sync_mode!r}")

    verification = record.get("verification")
    if cell["expected_engine"] == "rocksdb":
        if not isinstance(verification, dict) or verification.get("passed") is not True:
            raise ValidationError(f"RocksDB data verification did not pass for {cell['cell_id']}")
        settings = record.get("settings")
        effective_settings = settings.get("effective") if isinstance(settings, dict) else None
        if not isinstance(effective_settings, dict):
            raise ValidationError(f"RocksDB effective settings are missing for {cell['cell_id']}")
        if effective_settings.get("write_options_sync") is not True:
            raise ValidationError(f"RocksDB WriteOptions sync is not enabled for {cell['cell_id']}")
        disable_wal = effective_settings.get(
            "write_options_disable_wal", effective_settings.get("disable_wal")
        )
        if disable_wal is not False:
            raise ValidationError(f"RocksDB WAL is disabled or unspecified for {cell['cell_id']}")
    elif isinstance(verification, dict) and verification.get("passed") is False:
        raise ValidationError(f"raw data verification failed for {cell['cell_id']}")

    gate_values = {}
    required_gates = ("errors", "conflicts")
    if cell["expected_engine"] != "rocksdb":
        required_gates = (*required_gates, "overloads")
    for gate_name in required_gates:
        gate_value = nested_value(record, (gate_name,))
        if isinstance(gate_value, bool) or not isinstance(gate_value, (int, float)):
            raise ValidationError(f"raw record is missing numeric gate field {gate_name}")
        if gate_value != 0:
            raise ValidationError(
                f"acceptance gate failed for {cell['cell_id']}: {gate_name}={gate_value}"
            )
        gate_values[gate_name] = gate_value
    if cell["expected_engine"] == "rocksdb":
        overload_value = nested_value(record, ("overloads",))
        if overload_value is not None and (
            isinstance(overload_value, bool)
            or not isinstance(overload_value, (int, float))
            or overload_value != 0
        ):
            raise ValidationError(
                f"RocksDB overload field must be absent, null, or zero for {cell['cell_id']}"
            )
        gate_values["overloads"] = None

    return {
        "cell_id": cell["cell_id"],
        "trace_hash": trace_hash,
        "gate_values": gate_values,
        "raw_record": record,
    }


def validate_event_identity(event, cell, source_commit, binary_hash=None):
    for field_name in (
        "cell_id",
        "variant",
        "value_mode",
        "clients",
        "read_percent",
        "write_percent",
        "repetition",
        "seed",
        "scenario_index",
    ):
        if event.get(field_name) != cell[field_name]:
            raise ValidationError(
                f"run event {field_name} mismatch for {cell['cell_id']}: "
                f"{event.get(field_name)!r} != {cell[field_name]!r}"
            )
    if event.get("checkout_sha") != source_commit:
        raise ValidationError(f"run event checkout SHA mismatch for {cell['cell_id']}")
    if binary_hash is not None and event.get("binary_sha256") != binary_hash:
        raise ValidationError(f"binary SHA mismatch for {cell['cell_id']}")


def validate_expected_matrix(config, expected_cells):
    matrix = config.get("config")
    if not isinstance(matrix, dict):
        raise ValidationError("matrix config is missing its configuration object")
    clients = matrix.get("clients")
    variants = matrix.get("variants")
    value_modes = matrix.get("value_modes")
    mixes = matrix.get("mixes")
    repetitions = matrix.get("repetitions")
    if not isinstance(clients, list) or not clients:
        raise ValidationError("matrix configuration is missing clients")
    if not isinstance(variants, list) or not variants:
        raise ValidationError("matrix configuration is missing variants")
    if not isinstance(value_modes, list) or not value_modes:
        raise ValidationError("matrix configuration is missing value_modes")
    if not isinstance(mixes, list) or not mixes:
        raise ValidationError("matrix configuration is missing mixes")
    if isinstance(repetitions, bool) or not isinstance(repetitions, int) or repetitions < 1:
        raise ValidationError("matrix configuration has invalid repetitions")
    if matrix.get("sync_mode") != "real" or matrix.get("sync_contract") != "durable-return":
        raise ValidationError("matrix configuration does not require real durable synchronization")
    base_seed = matrix.get("base_seed")
    seed_stride = matrix.get("seed_stride")
    if isinstance(base_seed, bool) or not isinstance(base_seed, int) or base_seed < 0:
        raise ValidationError("matrix configuration has invalid base_seed")
    if isinstance(seed_stride, bool) or not isinstance(seed_stride, int) or seed_stride < 1:
        raise ValidationError("matrix configuration has invalid seed_stride")

    variant_specs = {}
    for variant_entry in variants:
        if isinstance(variant_entry, str):
            variant_name = variant_entry
            variant_spec = None
        elif isinstance(variant_entry, dict) and isinstance(variant_entry.get("name"), str):
            variant_name = variant_entry["name"]
            variant_spec = variant_entry
        else:
            raise ValidationError("matrix configuration contains an invalid variant")
        if variant_name in variant_specs:
            raise ValidationError(f"matrix configuration repeats variant {variant_name}")
        if variant_name not in SUPPORTED_VARIANTS:
            raise ValidationError(f"matrix configuration has unsupported variant {variant_name}")
        variant_specs[variant_name] = variant_spec

    mix_pairs = []
    for mix in mixes:
        try:
            read_percent, write_percent = (int(part) for part in mix.split("/"))
        except (AttributeError, ValueError) as error:
            raise ValidationError(f"matrix configuration has invalid mix {mix!r}") from error
        if read_percent + write_percent != 100:
            raise ValidationError(f"matrix configuration mix does not sum to 100: {mix!r}")
        mix_pairs.append((read_percent, write_percent))

    configured_signatures = {
        (client_count, read_percent, write_percent, value_mode, variant_name, repetition)
        for client_count, (read_percent, write_percent), value_mode, variant_name, repetition in product(
            clients,
            mix_pairs,
            value_modes,
            variant_specs,
            range(1, repetitions + 1),
        )
    }
    observed_signatures = [
        (
            cell.get("clients"),
            cell.get("read_percent"),
            cell.get("write_percent"),
            cell.get("value_mode"),
            cell.get("variant"),
            cell.get("repetition"),
        )
        for cell in expected_cells
    ]
    if len(set(observed_signatures)) != len(observed_signatures):
        raise ValidationError("matrix expected_cells contains duplicate run signatures")
    if set(observed_signatures) != configured_signatures:
        raise ValidationError("matrix expected_cells do not match the configured matrix dimensions")

    seed_by_scenario = defaultdict(set)
    scenario_index_by_scenario = defaultdict(set)
    for cell in expected_cells:
        scenario_key = (
            cell["clients"],
            cell["read_percent"],
            cell["write_percent"],
            cell["repetition"],
        )
        seed_by_scenario[scenario_key].add(cell.get("seed"))
        scenario_index_by_scenario[scenario_key].add(cell.get("scenario_index"))
        for field_name in (
            "transaction_width",
            "distribution",
            "working_set",
            "key_size",
            "value_size",
            "warmup_ms",
            "duration_ms",
            "cache_capacity",
            "read_limit",
        ):
            if cell.get(field_name) != matrix.get(field_name):
                raise ValidationError(
                    f"expected cell {cell['cell_id']} {field_name} does not match matrix configuration"
                )
        expected_scenario_index = (
            cell["clients"] * 1000 + cell["read_percent"] * 10 + cell["write_percent"]
        )
        if cell.get("scenario_index") != expected_scenario_index:
            raise ValidationError(f"expected cell {cell['cell_id']} has an invalid scenario_index")
        expected_seed = base_seed + expected_scenario_index * seed_stride + cell["repetition"] - 1
        if cell.get("seed") != expected_seed:
            raise ValidationError(f"expected cell {cell['cell_id']} has an invalid seed")
        variant_spec = variant_specs[cell["variant"]]
        supported_engine, supported_policy, supported_blink_workers = SUPPORTED_VARIANTS[cell["variant"]]
        if cell.get("expected_engine") != supported_engine:
            raise ValidationError(f"expected cell {cell['cell_id']} has an unsupported engine mapping")
        if cell.get("expected_collection_policy") != supported_policy:
            raise ValidationError(f"expected cell {cell['cell_id']} has an unsupported policy mapping")
        if cell.get("expected_blink_workers") != supported_blink_workers:
            raise ValidationError(f"expected cell {cell['cell_id']} has an unsupported blink worker setting")
        if variant_spec is not None:
            for cell_field, spec_field in (
                ("expected_engine", "engine"),
                ("expected_collection_policy", "collection_policy"),
                ("binary_key", "binary_key"),
                ("expected_blink_workers", "blink_workers"),
            ):
                if cell.get(cell_field) != variant_spec.get(spec_field):
                    raise ValidationError(
                        f"expected cell {cell['cell_id']} {cell_field} does not match variant configuration"
                    )
    for scenario_key, scenario_seeds in seed_by_scenario.items():
        if len(scenario_seeds) != 1:
            raise ValidationError(f"paired variants and value modes do not share a seed for {scenario_key}")
        if len(scenario_index_by_scenario[scenario_key]) != 1:
            raise ValidationError(f"paired variants and value modes do not share a scenario index for {scenario_key}")


def verify_existing_manifest(results):
    manifest_path = results / MANIFEST_NAME
    if not manifest_path.exists():
        return
    for line_number, line in enumerate(manifest_path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        parts = line.split("  ", 1)
        if len(parts) != 2:
            raise ValidationError(f"invalid artifact manifest line {line_number}")
        expected_digest, relative_name = parts
        artifact = results / relative_name
        if not artifact.is_file() or sha256_file(artifact) != expected_digest:
            raise ValidationError(f"artifact checksum mismatch: {relative_name}")


def validate_matrix(results):
    results = pathlib.Path(results).resolve()
    verify_existing_manifest(results)
    config = read_json(results / "matrix-config.json")
    if config.get("schema_version") != 1:
        raise ValidationError("unsupported matrix config schema_version")
    expected_cells = config.get("expected_cells")
    if not isinstance(expected_cells, list) or not expected_cells:
        raise ValidationError("matrix config has no expected_cells")
    if config.get("planned_count") != len(expected_cells):
        raise ValidationError("matrix planned_count does not match expected_cells")
    validate_expected_matrix(config, expected_cells)
    source_commit = config.get("source_commit")
    if not is_lowercase_hex(source_commit, 40):
        raise ValidationError("matrix config is missing source_commit")
    expected_by_id = {}
    for cell in expected_cells:
        if not isinstance(cell, dict) or not cell.get("cell_id"):
            raise ValidationError("matrix config has an invalid expected cell")
        if cell["cell_id"] in expected_by_id:
            raise ValidationError(f"duplicate expected cell {cell['cell_id']}")
        expected_by_id[cell["cell_id"]] = cell

    event_path = results / "run-order.jsonl"
    events = load_json_lines(event_path)
    starts = {}
    completes = {}
    for event in events:
        if not isinstance(event, dict):
            raise ValidationError("run-order entry must be an object")
        event_type = event.get("event")
        cell_id = event.get("cell_id")
        if cell_id not in expected_by_id:
            raise ValidationError(f"run-order contains unexpected cell {cell_id!r}")
        cell = expected_by_id[cell_id]
        validate_event_identity(event, cell, source_commit)
        target = starts if event_type == "start" else completes if event_type == "complete" else None
        if target is None:
            if event_type in ("run-failed", "runner-failed"):
                continue
            raise ValidationError(f"unknown run-order event {event_type!r}")
        if cell_id in target:
            raise ValidationError(f"duplicate {event_type} event for {cell_id}")
        target[cell_id] = event

    if set(starts) != set(expected_by_id):
        missing_cells = sorted(set(expected_by_id) - set(starts))
        raise ValidationError(f"incomplete matrix: missing start events for {missing_cells}")
    if set(completes) != set(expected_by_id):
        missing_cells = sorted(set(expected_by_id) - set(completes))
        raise ValidationError(f"incomplete matrix: missing complete events for {missing_cells}")

    binary_hashes = {}
    validated_runs = []
    expected_raw_paths = set()
    expected_log_paths = set()
    for cell in expected_cells:
        cell_id = cell["cell_id"]
        started = starts[cell_id]
        completed = completes[cell_id]
        binary_key = started.get("binary_key")
        binary_hash = started.get("binary_sha256")
        if not isinstance(binary_key, str) or not is_lowercase_hex(binary_hash, 64):
            raise ValidationError(f"start event is missing binary provenance for {cell_id}")
        prior_hash = binary_hashes.setdefault(binary_key, binary_hash)
        if prior_hash != binary_hash:
            raise ValidationError(f"binary changed within matrix for {binary_key}")
        validate_event_identity(completed, cell, source_commit, binary_hash)
        if completed.get("exit_code") != 0:
            raise ValidationError(f"benchmark did not complete successfully for {cell_id}")
        output_name = started.get("output")
        if output_name != completed.get("output"):
            raise ValidationError(f"start/complete output path mismatch for {cell_id}")
        if started.get("log") != completed.get("log"):
            raise ValidationError(f"start/complete log path mismatch for {cell_id}")
        if started.get("binary_key") != completed.get("binary_key"):
            raise ValidationError(f"start/complete binary key mismatch for {cell_id}")
        output_path = pathlib.Path(output_name).resolve()
        raw_root = (results / "raw").resolve()
        if output_path.parent != raw_root:
            raise ValidationError(f"raw output path escapes raw directory for {cell_id}")
        expected_raw_paths.add(output_path)
        if not output_path.is_file():
            raise ValidationError(f"raw output is missing for {cell_id}")
        output_digest = sha256_file(output_path)
        if completed.get("output_sha256") != output_digest:
            raise ValidationError(f"raw output checksum mismatch for {cell_id}")
        log_path = pathlib.Path(started.get("log", "")).resolve()
        if log_path.parent != raw_root or not log_path.is_file():
            raise ValidationError(f"benchmark log is missing or outside raw directory for {cell_id}")
        expected_log_paths.add(log_path)
        if completed.get("log_sha256") != sha256_file(log_path):
            raise ValidationError(f"benchmark log checksum mismatch for {cell_id}")
        raw_records = load_json_lines(output_path)
        if len(raw_records) != 1:
            raise ValidationError(f"expected one raw record for {cell_id}, found {len(raw_records)}")
        validated = validate_raw_record(raw_records[0], cell, source_commit)
        validated_runs.append(
            {
                "cell": cell,
                "start": started,
                "complete": completed,
                "validation": validated,
            }
        )

    actual_raw_paths = {path.resolve() for path in (results / "raw").glob("*.jsonl")}
    if actual_raw_paths != expected_raw_paths:
        unexpected = sorted(str(path) for path in actual_raw_paths - expected_raw_paths)
        missing = sorted(str(path) for path in expected_raw_paths - actual_raw_paths)
        raise ValidationError(f"raw file set mismatch: unexpected={unexpected}, missing={missing}")
    actual_log_paths = {path.resolve() for path in (results / "raw").glob("*.log")}
    if actual_log_paths != expected_log_paths:
        unexpected = sorted(str(path) for path in actual_log_paths - expected_log_paths)
        missing = sorted(str(path) for path in expected_log_paths - actual_log_paths)
        raise ValidationError(f"log file set mismatch: unexpected={unexpected}, missing={missing}")

    trace_groups = defaultdict(list)
    mode_trace_groups = defaultdict(dict)
    for run in validated_runs:
        cell = run["cell"]
        raw_record = run["validation"]["raw_record"]
        trace_hash = run["validation"]["trace_hash"]
        trace_key = (
            cell["clients"],
            cell["read_percent"],
            cell["write_percent"],
            cell["value_mode"],
            cell["repetition"],
        )
        trace_groups[trace_key].append((cell["variant"], trace_hash))
        pair_key = (
            cell["clients"],
            cell["read_percent"],
            cell["write_percent"],
            cell["repetition"],
            cell["variant"],
        )
        mode_trace_groups[pair_key][cell["value_mode"]] = trace_hash
    for trace_key, variant_hashes in trace_groups.items():
        if len({trace_hash for _, trace_hash in variant_hashes}) != 1:
            raise ValidationError(f"trace hash mismatch across variants for {trace_key}")
    for mode_key, mode_hashes in mode_trace_groups.items():
        if "constant" in mode_hashes and "changing" in mode_hashes:
            if mode_hashes["constant"] == mode_hashes["changing"]:
                raise ValidationError(f"constant/changing trace hashes are identical for {mode_key}")

    environment = read_json(results / "environment.json")
    if environment.get("git_commit") != source_commit:
        raise ValidationError("environment checkout SHA does not match matrix source_commit")
    return {
        "results": results,
        "config": config,
        "environment": environment,
        "validated_runs": validated_runs,
    }


def successful_read_write_percentages(record):
    explicit_read = numeric_value(record, ("successful_read_percent",))
    explicit_write = numeric_value(record, ("successful_write_percent",))
    if explicit_read is not None and explicit_write is not None:
        return explicit_read, explicit_write
    read_count = numeric_value(
        record,
        ("successful_reads", "successful_read_operations", "read_operations"),
    )
    write_count = numeric_value(
        record,
        (
            "successful_write_transactions",
            "successful_transactions",
            "mutation_ops",
            "write_transactions",
        ),
    )
    if read_count is None or write_count is None or read_count + write_count <= 0:
        return explicit_read, explicit_write
    total = read_count + write_count
    return read_count * 100.0 / total, write_count * 100.0 / total


def metric_value(record, metric_name):
    aliases = {
        "aggregate_ops_per_second": ("aggregate_ops_per_second", "total_operations_per_second"),
        "read_ops_per_second": ("read_ops_per_second", "successful_read_operations_per_second", "read_operations_per_second"),
        "write_tx_per_second": ("logical_tx_per_second", "write_tx_per_second", "write_transactions_per_second"),
        "read_p50_us": ("read_p50_us",),
        "read_p95_us": ("read_p95_us",),
        "read_p99_us": ("read_p99_us",),
        "write_p50_us": ("write_p50_us",),
        "write_p95_us": ("write_p95_us",),
        "write_p99_us": ("write_p99_us",),
        "errors": ("errors",),
        "conflicts": ("conflicts",),
        "overloads": ("overloads",),
        "cpu_utilization_percent_one_core": ("cpu_utilization_percent_one_core",),
        "cpu_seconds": ("cpu_seconds",),
        "avg_transactions_per_group": ("avg_transactions_per_group",),
        "wal_syncs_delta": ("wal_syncs_delta", "syncs_delta"),
        "wal_bytes_delta": ("wal_bytes_delta",),
        "wal_sync_nanos_total": ("wal_sync_nanos_total",),
        "page_images_delta": ("page_images_delta",),
        "wal_page_image_records_delta": ("wal_page_image_records_delta",),
        "wal_page_delta_spans_delta": ("wal_page_delta_spans_delta",),
        "wal_page_delta_changed_bytes_delta": ("wal_page_delta_changed_bytes_delta",),
        "superblock_images_emitted": ("superblock_images_emitted",),
        "superblock_images_elided": ("superblock_images_elided",),
        "physical_execution_nanos": ("physical_execution_nanos",),
        "planning_nanos": ("planning_nanos",),
        "mutations_planned": ("mutations_planned",),
        "coalesced_mutations": ("coalesced_mutations",),
        "parallel_workers": ("parallel_workers",),
        "blink_workers": ("blink_workers",),
        "parallel_worker_dispatches_delta": ("parallel_worker_dispatches_delta", "parallel_worker_dispatches"),
        "parallel_worker_nanos_total": ("parallel_worker_nanos_total", "parallel_worker_nanos"),
    }
    if metric_name == "successful_read_percent":
        return successful_read_write_percentages(record)[0]
    if metric_name == "successful_write_percent":
        return successful_read_write_percentages(record)[1]
    if metric_name in ("rss_end_kib", "rss_hwm_kib"):
        rss = record.get("rss_kib")
        if isinstance(rss, dict):
            rss_key = "end" if metric_name == "rss_end_kib" else "hwm"
            value = rss.get(rss_key)
            if isinstance(value, (int, float)) and not isinstance(value, bool):
                return float(value)
        direct_names = (metric_name, "max_rss_kib" if metric_name == "rss_hwm_kib" else "rss_kib")
        return numeric_value(record, direct_names)
    if metric_name == "aggregate_ops_per_second":
        direct_value = numeric_value(record, aliases[metric_name])
        if direct_value is not None:
            return direct_value
        read_value = metric_value(record, "read_ops_per_second")
        write_value = metric_value(record, "write_tx_per_second")
        if read_value is not None and write_value is not None:
            return read_value + write_value
        return None
    if metric_name == "avg_transactions_per_wal_sync":
        direct_value = numeric_value(record, ("transactions_per_sync", "avg_transactions_per_wal_sync"))
        if direct_value is not None:
            return direct_value
        transactions = numeric_value(record, ("logical_transactions_metric", "logical_tx_count", "successful_transactions"))
        syncs = metric_value(record, "wal_syncs_delta")
        if transactions is not None and syncs not in (None, 0):
            return transactions / syncs
        return None
    if metric_name == "wal_bytes_per_sync":
        byte_count = metric_value(record, "wal_bytes_delta")
        syncs = metric_value(record, "wal_syncs_delta")
        if byte_count is not None and syncs not in (None, 0):
            return byte_count / syncs
        return None
    if metric_name == "wal_sync_average_us":
        sync_nanos = metric_value(record, "wal_sync_nanos_total")
        syncs = metric_value(record, "wal_syncs_delta")
        if sync_nanos is not None and syncs not in (None, 0):
            return sync_nanos / syncs / 1000.0
        return None
    return numeric_value(record, aliases.get(metric_name, (metric_name,)))


def median_range(values):
    present_values = [float(value) for value in values if value is not None]
    if not present_values:
        return {"median": None, "min": None, "max": None, "values": []}
    return {
        "median": statistics.median(present_values),
        "min": min(present_values),
        "max": max(present_values),
        "values": present_values,
    }


def build_summary(validated):
    config = validated["config"]
    grouped_runs = defaultdict(list)
    records_by_cell = {}
    for run in validated["validated_runs"]:
        cell = run["cell"]
        raw_record = run["validation"]["raw_record"]
        group_key = (
            cell["clients"],
            cell["read_percent"],
            cell["write_percent"],
            cell["value_mode"],
            cell["variant"],
        )
        grouped_runs[group_key].append(run)
        records_by_cell[cell["cell_id"]] = (cell, raw_record)

    baseline_values = defaultdict(dict)
    for cell, record in records_by_cell.values():
        if cell["variant"] == "main-btree":
            baseline_values[
                (
                    cell["clients"],
                    cell["read_percent"],
                    cell["write_percent"],
                    cell["value_mode"],
                    cell["repetition"],
                )
            ] = record

    summary_rows = []
    for group_key in sorted(grouped_runs):
        clients, read_percent, write_percent, value_mode, variant = group_key
        runs = sorted(grouped_runs[group_key], key=lambda run: run["cell"]["repetition"])
        first_cell = runs[0]["cell"]
        metric_summaries = {}
        for metric_name in METRIC_NAMES:
            metric_summaries[metric_name] = median_range(
                [metric_value(run["validation"]["raw_record"], metric_name) for run in runs]
            )
        paired_ratios = {}
        for metric_name in RATIO_METRICS:
            ratios = []
            for run in runs:
                cell = run["cell"]
                baseline = baseline_values.get(
                    (
                        cell["clients"],
                        cell["read_percent"],
                        cell["write_percent"],
                        cell["value_mode"],
                        cell["repetition"],
                    )
                )
                if baseline is None:
                    continue
                numerator = metric_value(run["validation"]["raw_record"], metric_name)
                denominator = metric_value(baseline, metric_name)
                if numerator is not None and denominator not in (None, 0):
                    ratios.append(numerator / denominator)
            paired_ratios[metric_name] = median_range(ratios)

        summary_rows.append(
            {
                "variant": variant,
                "engine": first_cell["expected_engine"],
                "collection_policy": first_cell["expected_collection_policy"],
                "clients": clients,
                "read_percent": read_percent,
                "write_percent": write_percent,
                "working_set": first_cell["working_set"],
                "transaction_width": first_cell["transaction_width"],
                "distribution": first_cell["distribution"],
                "key_size": first_cell["key_size"],
                "value_size": first_cell["value_size"],
                "value_mode": value_mode,
                "sync_contract": "durable-return",
                "warmup_ms": first_cell["warmup_ms"],
                "duration_ms": first_cell["duration_ms"],
                "git_commit": config["source_commit"],
                "repetitions": len(runs),
                "seeds": [run["cell"]["seed"] for run in runs],
                "trace_hashes": [run["validation"]["trace_hash"] for run in runs],
                "metrics": metric_summaries,
                "paired_ratios_vs_main_btree": paired_ratios,
                "runs": [
                    {
                        "cell_id": run["cell"]["cell_id"],
                        "start": run["start"],
                        "complete": run["complete"],
                        "raw_record": run["validation"]["raw_record"],
                    }
                    for run in runs
                ],
            }
        )

    return {
        "schema_version": 1,
        "accepted": True,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "analysis_scope": "Descriptive paired measurements; these records do not identify a causal mechanism.",
        "planned_count": config["planned_count"],
        "completed_count": len(validated["validated_runs"]),
        "source_commit": config["source_commit"],
        "matrix_config": config,
        "environment": validated["environment"],
        "rows": summary_rows,
    }


def flatten_csv_rows(summary):
    output_rows = []
    for summary_row in summary["rows"]:
        flat_row = {
            key: summary_row[key]
            for key in (
                "variant",
                "engine",
                "collection_policy",
                "clients",
                "read_percent",
                "write_percent",
                "working_set",
                "transaction_width",
                "distribution",
                "key_size",
                "value_size",
                "value_mode",
                "sync_contract",
                "warmup_ms",
                "duration_ms",
                "git_commit",
                "repetitions",
            )
        }
        flat_row["seeds"] = ";".join(str(seed) for seed in summary_row["seeds"])
        flat_row["trace_hashes"] = ";".join(summary_row["trace_hashes"])
        for metric_name, metric_summary in summary_row["metrics"].items():
            for statistic_name in ("median", "min", "max"):
                flat_row[f"{metric_name}_{statistic_name}"] = metric_summary[statistic_name]
        for metric_name, ratio_summary in summary_row["paired_ratios_vs_main_btree"].items():
            flat_row[f"paired_{metric_name}_ratio_median"] = ratio_summary["median"]
            flat_row[f"paired_{metric_name}_ratio_min"] = ratio_summary["min"]
            flat_row[f"paired_{metric_name}_ratio_max"] = ratio_summary["max"]
        output_rows.append(flat_row)
    return output_rows


def write_artifact_manifest(results):
    entries = []
    for artifact in sorted(results.rglob("*")):
        if artifact.is_file() and artifact.name != MANIFEST_NAME:
            entries.append(f"{sha256_file(artifact)}  {artifact.relative_to(results)}")
    manifest_path = results / MANIFEST_NAME
    manifest_path.write_text("\n".join(entries) + "\n", encoding="utf-8")


def write_summary(validated):
    results = validated["results"]
    summary = build_summary(validated)
    summary_path = results / SUMMARY_JSON_NAME
    summary_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    rows = flatten_csv_rows(summary)
    if not rows:
        raise ValidationError("accepted matrix produced no summary rows")
    csv_path = results / SUMMARY_CSV_NAME
    with csv_path.open("w", newline="", encoding="utf-8") as output_file:
        writer = csv.DictWriter(output_file, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    status = {
        "accepted": True,
        "planned_count": summary["planned_count"],
        "completed_count": summary["completed_count"],
        "summary_json": SUMMARY_JSON_NAME,
        "summary_csv": SUMMARY_CSV_NAME,
        "source_commit": summary["source_commit"],
    }
    (results / "matrix-status.json").write_text(
        json.dumps(status, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    write_artifact_manifest(results)
    return summary


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("results", type=pathlib.Path)
    arguments = parser.parse_args()
    try:
        validated = validate_matrix(arguments.results)
        summary = write_summary(validated)
    except ValidationError as error:
        print(f"payload batching matrix rejected: {error}", file=sys.stderr)
        return 1
    print(
        f"accepted={summary['completed_count']}/{summary['planned_count']} "
        f"rows={len(summary['rows'])} csv={arguments.results / SUMMARY_CSV_NAME}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
