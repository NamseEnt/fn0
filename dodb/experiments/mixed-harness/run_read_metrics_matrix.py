import argparse
import json
import math
import os
import pathlib
import statistics
import subprocess
import sys
from collections import defaultdict

from run_payload_batching_matrix import (
    BASE_SEED,
    SEED_STRIDE,
    RunnerError,
    environment_record,
    parse_string_list,
    record_event,
    require_clean_source,
    sha256_file,
    timestamp_utc,
    write_json,
)
from summarize_payload_batching import (
    ValidationError,
    load_json_lines,
    numeric_value,
    write_artifact_manifest,
)


RUNNER_NAME = "run_read_metrics_matrix.py"
SUMMARY_NAME = "read-metrics-summary.json"
DEFAULT_READ_KINDS = ("get", "query")
DEFAULT_VARIANTS = ("main-btree", "blink-metrics-on", "blink-metrics-off", "rocksdb")
READ_KIND_INDEX = {"get": 1, "query": 2}
PHASE0_TOKIO_WORKERS = 2
VARIANT_SPECS = {
    "main-btree": {
        "binary_key": "phase0-bench",
        "binary_env": "PHASE0_BINARY",
        "binary_kind": "phase0",
        "engine": "main-btree",
        "collection_policy": "native-main",
        "read_metrics_mode": "on",
        "blink_read_observational_metrics_enabled": True,
        "tokio_workers": PHASE0_TOKIO_WORKERS,
        "engine_options": (),
    },
    "blink-metrics-on": {
        "binary_key": "phase0-bench",
        "binary_env": "PHASE0_BINARY",
        "binary_kind": "phase0",
        "engine": "parallel-blink",
        "collection_policy": "current",
        "read_metrics_mode": "on",
        "blink_read_observational_metrics_enabled": True,
        "tokio_workers": PHASE0_TOKIO_WORKERS,
        "blink_workers": 2,
        "engine_options": ("--blink-collection-policy", "current", "--blink-workers", "2"),
    },
    "blink-metrics-off": {
        "binary_key": "phase0-read-metrics-off",
        "binary_env": "PHASE0_READ_METRICS_OFF_BINARY",
        "binary_kind": "phase0",
        "engine": "parallel-blink",
        "collection_policy": "current",
        "read_metrics_mode": "off",
        "blink_read_observational_metrics_enabled": False,
        "tokio_workers": PHASE0_TOKIO_WORKERS,
        "blink_workers": 2,
        "engine_options": ("--blink-collection-policy", "current", "--blink-workers", "2"),
    },
    "rocksdb": {
        "binary_key": "rocksdb-bench",
        "binary_env": "ROCKSDB_BINARY",
        "binary_kind": "rocksdb",
        "engine": "rocksdb",
        "collection_policy": "not-applicable",
        "read_metrics_mode": "not-applicable",
        "blink_read_observational_metrics_enabled": None,
        "engine_options": (),
    },
}


def positive_integer(value):
    try:
        parsed_value = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("value must be a positive integer") from error
    if parsed_value < 1:
        raise argparse.ArgumentTypeError("value must be a positive integer")
    return parsed_value


def nonnegative_integer(value):
    try:
        parsed_value = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError("value must be a nonnegative integer") from error
    if parsed_value < 0:
        raise argparse.ArgumentTypeError("value must be a nonnegative integer")
    return parsed_value


def parse_arguments(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument("--results", required=True, type=pathlib.Path)
    parser.add_argument(
        "--data-root",
        type=pathlib.Path,
        default=pathlib.Path(os.environ.get("BENCH_DATA_ROOT", "/bench/zfs/db")),
    )
    parser.add_argument("--readers", type=positive_integer, default=16)
    parser.add_argument("--read-kinds", "--read-kind", default=",".join(DEFAULT_READ_KINDS))
    parser.add_argument("--variants", default=",".join(DEFAULT_VARIANTS))
    parser.add_argument("--repetitions", type=positive_integer, default=3)
    parser.add_argument("--warmup-ms", type=nonnegative_integer, default=2_000)
    parser.add_argument("--duration-ms", type=positive_integer, default=5_000)
    parser.add_argument("--working-set", type=positive_integer, default=10_000)
    parser.add_argument("--key-size", type=positive_integer, default=16)
    parser.add_argument("--value-size", type=positive_integer, default=512)
    parser.add_argument("--read-limit", type=positive_integer, default=16)
    parser.add_argument("--cache-capacity", type=positive_integer, default=16_384)
    parser.add_argument("--base-seed", type=nonnegative_integer, default=BASE_SEED)
    arguments = parser.parse_args(argv)
    arguments.results = arguments.results.expanduser().resolve()
    arguments.data_root = arguments.data_root.expanduser().resolve()
    arguments.read_kinds = parse_string_list(arguments.read_kinds, "read-kinds")
    arguments.variants = parse_string_list(arguments.variants, "variants")
    unsupported_read_kinds = sorted(set(arguments.read_kinds) - set(DEFAULT_READ_KINDS))
    if unsupported_read_kinds:
        parser.error(f"unsupported read kinds: {unsupported_read_kinds}")
    unsupported_variants = sorted(set(arguments.variants) - set(VARIANT_SPECS))
    if unsupported_variants:
        parser.error(f"unsupported variants: {unsupported_variants}")
    return arguments


def build_expected_cells(arguments, source_commit):
    expected_cells = []
    for read_kind in arguments.read_kinds:
        scenario_index = arguments.readers * 1_000 + READ_KIND_INDEX[read_kind] * 100
        for repetition in range(1, arguments.repetitions + 1):
            seed = arguments.base_seed + scenario_index * SEED_STRIDE + repetition - 1
            if seed > (1 << 64) - 1:
                raise RunnerError(
                    f"seed exceeds u64 range for {read_kind} repetition {repetition}"
                )
            for variant in arguments.variants:
                variant_spec = VARIANT_SPECS[variant]
                cell_id = (
                    f"readers-{arguments.readers}-{read_kind}-{variant}-rep{repetition}"
                )
                expected_cells.append(
                    {
                        "cell_id": cell_id,
                        "scenario_index": scenario_index,
                        "variant": variant,
                        "expected_engine": variant_spec["engine"],
                        "expected_collection_policy": variant_spec["collection_policy"],
                        "expected_blink_workers": variant_spec.get("blink_workers"),
                        "expected_read_metrics_enabled": variant_spec[
                            "blink_read_observational_metrics_enabled"
                        ],
                        "expected_tokio_workers": (
                            PHASE0_TOKIO_WORKERS
                            if variant_spec["binary_kind"] == "phase0"
                            else None
                        ),
                        "read_metrics_mode": variant_spec["read_metrics_mode"],
                        "binary_key": variant_spec["binary_key"],
                        "binary_env": variant_spec["binary_env"],
                        "binary_kind": variant_spec["binary_kind"],
                        "readers": arguments.readers,
                        "read_kind": read_kind,
                        "repetition": repetition,
                        "expected_raw_repetition": (
                            0 if variant_spec["binary_kind"] == "phase0" else repetition
                        ),
                        "seed": seed,
                        "transaction_width": 1,
                        "distribution": "uniform",
                        "working_set": arguments.working_set,
                        "key_size": arguments.key_size,
                        "value_size": arguments.value_size,
                        "read_limit": arguments.read_limit,
                        "cache_capacity": arguments.cache_capacity,
                        "warmup_ms": arguments.warmup_ms,
                        "duration_ms": arguments.duration_ms,
                        "source_commit": source_commit,
                    }
                )
    return expected_cells


def binary_path_for(cell):
    configured_path = os.environ.get(cell["binary_env"])
    if not configured_path:
        raise RunnerError(f"required binary environment variable is missing: {cell['binary_env']}")
    binary_path = pathlib.Path(configured_path).expanduser().resolve()
    if not binary_path.is_file() or not os.access(binary_path, os.X_OK):
        raise RunnerError(f"benchmark executable is missing or not executable: {binary_path}")
    return binary_path


def build_command(arguments, cell, binary_path, output_path, data_directory):
    variant_spec = VARIANT_SPECS[cell["variant"]]
    if cell["binary_kind"] == "phase0":
        command = [
            str(binary_path),
            "--engine",
            cell["expected_engine"],
            "--suite",
            "read",
            "--readers",
            str(cell["readers"]),
            "--widths",
            "1",
            "--distributions",
            "uniform",
            "--read-kinds",
            cell["read_kind"],
            "--duration",
            f"{cell['duration_ms']}ms",
            "--warmup",
            f"{cell['warmup_ms']}ms",
            "--repetitions",
            "1",
            "--tokio-workers",
            str(PHASE0_TOKIO_WORKERS),
            "--cache-capacity",
            str(cell["cache_capacity"]),
            "--working-set",
            str(cell["working_set"]),
            "--key-size",
            str(cell["key_size"]),
            "--value-size",
            str(cell["value_size"]),
            "--read-limit",
            str(cell["read_limit"]),
            "--sync-mode",
            "real",
            "--seed",
            str(cell["seed"]),
            "--output",
            str(output_path),
            *variant_spec["engine_options"],
        ]
        environment = os.environ.copy()
        environment["DODB_BENCH_DIR"] = str(data_directory)
        return command, environment

    command = [
        str(binary_path),
        "--mode",
        "bench",
        "--operation",
        cell["read_kind"],
        "--read-limit",
        str(cell["read_limit"]),
        "--writers",
        str(cell["readers"]),
        "--width",
        "1",
        "--distribution",
        "uniform",
        "--working-set",
        str(cell["working_set"]),
        "--key-size",
        str(cell["key_size"]),
        "--value-size",
        str(cell["value_size"]),
        "--warmup-ms",
        str(cell["warmup_ms"]),
        "--duration-ms",
        str(cell["duration_ms"]),
        "--seed",
        str(cell["seed"]),
        "--scenario-index",
        str(cell["scenario_index"]),
        "--repetition",
        str(cell["repetition"]),
        "--data-dir",
        str(data_directory),
        "--output",
        str(output_path),
    ]
    return command, os.environ.copy()


def event_identity(cell, source_commit, source_branch):
    return {
        "cell_id": cell["cell_id"],
        "variant": cell["variant"],
        "readers": cell["readers"],
        "read_kind": cell["read_kind"],
        "repetition": cell["repetition"],
        "expected_raw_repetition": cell["expected_raw_repetition"],
        "seed": cell["seed"],
        "scenario_index": cell["scenario_index"],
        "checkout_sha": source_commit,
        "source_branch": source_branch,
        "read_metrics_mode": cell["read_metrics_mode"],
        "expected_tokio_workers": cell["expected_tokio_workers"],
        "execution_topology": (
            f"{cell['readers']} reader tasks on {PHASE0_TOKIO_WORKERS} Tokio workers"
            if cell["binary_kind"] == "phase0"
            else f"{cell['readers']} RocksDB reader OS threads"
        ),
    }


def validate_event_identity(event, cell, source_commit, source_branch):
    expected_identity = event_identity(cell, source_commit, source_branch)
    for field_name, expected_value in expected_identity.items():
        if event.get(field_name) != expected_value:
            raise ValidationError(
                f"run event {field_name} mismatch for {cell['cell_id']}"
            )


def raw_stem(cell):
    return cell["cell_id"]


def execute_cell(arguments, results, repository, cell, binary_registry, source_state_before):
    binary_path = binary_path_for(cell)
    binary_hash = sha256_file(binary_path)
    registered_binary = binary_registry.setdefault(
        cell["binary_key"], {"path": str(binary_path), "sha256": binary_hash}
    )
    if registered_binary != {"path": str(binary_path), "sha256": binary_hash}:
        raise RunnerError(f"binary changed within matrix: {cell['binary_key']}")

    raw_directory = results / "raw"
    raw_directory.mkdir(exist_ok=True)
    stem = raw_stem(cell)
    output_path = raw_directory / f"{stem}.jsonl"
    log_path = raw_directory / f"{stem}.log"
    data_directory = arguments.data_root / results.name / stem
    if output_path.exists() or log_path.exists() or data_directory.exists():
        raise RunnerError(f"run path already exists and will not be reused: {stem}")
    command, environment = build_command(
        arguments, cell, binary_path, output_path, data_directory
    )
    identity = event_identity(
        cell, source_state_before["commit"], source_state_before["branch"]
    )
    started = {
        "event": "start",
        **identity,
        "binary_key": cell["binary_key"],
        "binary_env": cell["binary_env"],
        "binary_path": str(binary_path),
        "binary_sha256": binary_hash,
        "command": command,
        "output": str(output_path.resolve()),
        "log": str(log_path.resolve()),
        "data_dir": str(data_directory),
        "read_metrics_mode": cell["read_metrics_mode"],
        "sync_mode": "real",
        "sync_contract": "durable-return",
    }
    record_event(results, started)
    try:
        with log_path.open("x", encoding="utf-8") as log_file:
            subprocess.run(
                command,
                cwd=repository,
                env=environment,
                stdout=log_file,
                stderr=subprocess.STDOUT,
                check=True,
            )
        raw_records = load_json_lines(output_path)
        if len(raw_records) != 1:
            raise ValidationError(
                f"expected one raw record for {cell['cell_id']}, found {len(raw_records)}"
            )
        validate_raw_record(raw_records[0], cell, source_state_before["commit"])
        completed = {
            "event": "complete",
            **identity,
            "binary_key": cell["binary_key"],
            "binary_env": cell["binary_env"],
            "binary_path": str(binary_path),
            "binary_sha256": binary_hash,
            "exit_code": 0,
            "output": str(output_path.resolve()),
            "log": str(log_path.resolve()),
            "output_sha256": sha256_file(output_path),
            "log_sha256": sha256_file(log_path),
        }
        record_event(results, completed)
    except BaseException as error:
        record_event(
            results,
            {
                "event": "run-failed",
                **identity,
                "failure": str(error),
                "failure_type": type(error).__name__,
                "timestamp_utc": timestamp_utc(),
            },
        )
        raise


def require_value(record, field_name, aliases=()):
    for candidate in (field_name, *aliases):
        if candidate in record and record[candidate] is not None:
            return record[candidate]
    raise ValidationError(f"raw record is missing required field {field_name}")


def assert_equal(record, field_name, expected, aliases=()):
    actual_value = require_value(record, field_name, aliases)
    if actual_value != expected:
        raise ValidationError(
            f"raw {field_name} mismatch: actual={actual_value!r}, expected={expected!r}"
        )


def assert_zero_gate(record, field_name, cell_id):
    gate_value = numeric_value(record, (field_name,))
    if gate_value is None:
        raise ValidationError(f"raw record is missing numeric gate field {field_name}")
    if gate_value != 0:
        raise ValidationError(f"acceptance gate failed for {cell_id}: {field_name}={gate_value}")


def effective_rocksdb_settings(record, cell_id):
    settings = record.get("settings")
    effective_settings = settings.get("effective") if isinstance(settings, dict) else None
    if not isinstance(effective_settings, dict):
        raise ValidationError(f"RocksDB effective settings are missing for {cell_id}")
    if effective_settings.get("write_options_sync") is not True:
        raise ValidationError(f"RocksDB WriteOptions sync is not enabled for {cell_id}")
    disable_wal = effective_settings.get(
        "write_options_disable_wal", effective_settings.get("disable_wal")
    )
    if disable_wal is not False:
        raise ValidationError(f"RocksDB WAL is disabled or unspecified for {cell_id}")


def validate_raw_record(record, cell, checkout_sha):
    if not isinstance(record, dict):
        raise ValidationError(f"raw record must be an object: {cell['cell_id']}")
    if record.get("record_type") not in ("run", "crossdb-run"):
        raise ValidationError(f"raw record_type is missing or invalid for {cell['cell_id']}")
    assert_equal(record, "engine", cell["expected_engine"])
    assert_equal(record, "seed", cell["seed"])
    assert_equal(record, "transaction_width", 1, ("width",))
    assert_equal(record, "distribution", cell["distribution"])
    assert_equal(record, "working_set", cell["working_set"])
    assert_equal(record, "key_size", cell["key_size"])
    assert_equal(record, "value_size", cell["value_size"])
    assert_equal(record, "read_limit", cell["read_limit"])
    assert_equal(record, "sync_contract", "durable-return")
    assert_equal(record, "git_commit", checkout_sha)
    assert_equal(record, "warmup_ms", cell["warmup_ms"])
    assert_equal(record, "requested_duration_ms", cell["duration_ms"])
    assert_equal(record, "repetition", cell["expected_raw_repetition"])
    if cell["binary_kind"] == "phase0":
        assert_equal(record, "suite", "read")
        assert_equal(record, "readers", cell["readers"])
        assert_equal(record, "writers", 0)
        assert_equal(record, "tokio_workers", PHASE0_TOKIO_WORKERS)
        assert_equal(record, "read_kind", cell["read_kind"])
        assert_equal(record, "collection_policy", cell["expected_collection_policy"])
        assert_equal(
            record,
            "blink_read_observational_metrics_enabled",
            cell["expected_read_metrics_enabled"],
        )
        if cell.get("expected_blink_workers") is not None:
            assert_equal(record, "blink_workers", cell["expected_blink_workers"])
        assert_equal(record, "sync_mode", "real")
    else:
        assert_equal(record, "operation", cell["read_kind"])
        assert_equal(record, "client_workers", cell["readers"], ("writers",))
        assert_equal(record, "collection_policy", "not-applicable")
        assert_equal(record, "read_key_generator_version", "key_only_preserving_schedule_v1")
        effective_rocksdb_settings(record, cell["cell_id"])
        verification = record.get("verification")
        if not isinstance(verification, dict) or verification.get("passed") is not True:
            raise ValidationError(f"RocksDB read verification did not pass for {cell['cell_id']}")
        if verification.get("comparison") != "full_value_hash_and_length":
            raise ValidationError(f"RocksDB did not verify full values for {cell['cell_id']}")
        if verification.get("sampled_keys", 0) < 1:
            raise ValidationError(f"RocksDB verified no sampled keys for {cell['cell_id']}")

    for gate_name in ("errors", "conflicts"):
        assert_zero_gate(record, gate_name, cell["cell_id"])
    if cell["binary_kind"] == "phase0":
        assert_zero_gate(record, "overloads", cell["cell_id"])

    success_count = numeric_value(
        record, ("successful_reads", "successful_read_operations")
    )
    if success_count is None or success_count < 1:
        raise ValidationError(f"raw record has no successful reads for {cell['cell_id']}")
    successful_read_percent = numeric_value(record, ("successful_read_percent",))
    successful_write_count = numeric_value(
        record, ("successful_write_transactions", "write_transactions")
    )
    if successful_read_percent != 100.0 or successful_write_count != 0:
        raise ValidationError(f"raw record contains non-read operations for {cell['cell_id']}")
    if cell["variant"] in ("blink-metrics-on", "blink-metrics-off"):
        read_operation_metric = numeric_value(record, ("read_operations_metric",))
        if read_operation_metric is None:
            raise ValidationError(
                f"raw record is missing read_operations_metric for {cell['cell_id']}"
            )
        expected_enabled = cell["expected_read_metrics_enabled"]
        if expected_enabled and read_operation_metric <= 0:
            raise ValidationError(
                f"Blink read metrics were inactive for {cell['cell_id']}"
            )
        if not expected_enabled and read_operation_metric != 0:
            raise ValidationError(
                f"Blink read metrics were active in disabled build for {cell['cell_id']}"
            )

    throughput_field = f"{cell['read_kind']}_ops_per_second"
    if cell["binary_kind"] == "phase0":
        throughput_value = numeric_value(record, (throughput_field,))
        read_p99_value = numeric_value(record, ("read_p99_us",))
    else:
        throughput_value = numeric_value(
            record,
            (
                "successful_read_operations_per_second",
                "read_operations_per_second",
                "total_operations_per_second",
            ),
        )
        read_p99_value = numeric_value(record, ("read_p99_us", "p99_us"))
    if throughput_value is None or throughput_value <= 0:
        raise ValidationError(f"raw throughput is missing or invalid for {cell['cell_id']}")
    if read_p99_value is None or read_p99_value < 0:
        raise ValidationError(f"raw read p99 is missing or invalid for {cell['cell_id']}")
    return record


def validate_expected_cells(config):
    matrix = config.get("config")
    if not isinstance(matrix, dict):
        raise ValidationError("matrix config is missing its configuration object")
    if config.get("schema_version") != 1:
        raise ValidationError("unsupported read metrics schema_version")
    if matrix.get("sync_mode") != "real" or matrix.get("sync_contract") != "durable-return":
        raise ValidationError("matrix does not require real durable synchronization")
    if matrix.get("tokio_workers") != PHASE0_TOKIO_WORKERS:
        raise ValidationError("matrix has an unexpected Tokio worker count")
    if matrix.get("reader_execution_models") != {
        "phase0": "one async reader task per reader on two Tokio workers",
        "rocksdb": "one OS thread per reader",
    }:
        raise ValidationError("matrix has invalid reader execution metadata")
    configured_readers = matrix.get("readers")
    configured_repetitions = matrix.get("repetitions")
    read_kinds = matrix.get("read_kinds")
    variant_entries = matrix.get("variants")
    if isinstance(configured_readers, bool) or not isinstance(configured_readers, int) or configured_readers < 1:
        raise ValidationError("matrix has invalid readers")
    if isinstance(configured_repetitions, bool) or not isinstance(configured_repetitions, int) or configured_repetitions < 1:
        raise ValidationError("matrix has invalid repetitions")
    if not isinstance(read_kinds, list) or not read_kinds or len(set(read_kinds)) != len(read_kinds):
        raise ValidationError("matrix has invalid read_kinds")
    if any(not isinstance(read_kind, str) for read_kind in read_kinds):
        raise ValidationError("matrix has invalid read_kinds")
    if not set(read_kinds) <= set(DEFAULT_READ_KINDS):
        raise ValidationError("matrix contains an unsupported read kind")
    if not isinstance(variant_entries, list) or not variant_entries:
        raise ValidationError("matrix has invalid variants")
    expected_cells = config.get("expected_cells")
    if not isinstance(expected_cells, list) or not expected_cells:
        raise ValidationError("matrix config has no expected_cells")
    if config.get("planned_count") != len(expected_cells):
        raise ValidationError("matrix planned_count does not match expected_cells")
    base_seed = matrix.get("base_seed")
    seed_stride = matrix.get("seed_stride")
    if isinstance(base_seed, bool) or not isinstance(base_seed, int) or base_seed < 0:
        raise ValidationError("matrix has invalid base_seed")
    if isinstance(seed_stride, bool) or not isinstance(seed_stride, int) or seed_stride < 1:
        raise ValidationError("matrix has invalid seed_stride")
    for field_name, minimum in (
        ("working_set", 1),
        ("key_size", 2),
        ("value_size", 1),
        ("read_limit", 1),
        ("cache_capacity", 1),
        ("duration_ms", 1),
        ("warmup_ms", 0),
    ):
        field_value = matrix.get(field_name)
        if isinstance(field_value, bool) or not isinstance(field_value, int) or field_value < minimum:
            raise ValidationError(f"matrix has invalid {field_name}")
    variant_by_name = {}
    for entry in variant_entries:
        if (
            not isinstance(entry, dict)
            or not isinstance(entry.get("name"), str)
            or entry.get("name") not in VARIANT_SPECS
        ):
            raise ValidationError("matrix contains an unsupported variant")
        variant_name = entry["name"]
        if variant_name in variant_by_name:
            raise ValidationError(f"matrix repeats variant {variant_name}")
        expected_spec = VARIANT_SPECS[variant_name]
        for field_name, spec_field in (
            ("engine", "engine"),
            ("collection_policy", "collection_policy"),
            ("binary_key", "binary_key"),
            ("binary_env", "binary_env"),
            ("binary_kind", "binary_kind"),
            ("read_metrics_mode", "read_metrics_mode"),
            (
                "blink_read_observational_metrics_enabled",
                "blink_read_observational_metrics_enabled",
            ),
            ("blink_workers", "blink_workers"),
            ("tokio_workers", "tokio_workers"),
            ("engine_options", "engine_options"),
        ):
            expected_value = expected_spec.get(spec_field)
            if field_name == "engine_options":
                expected_value = list(expected_value)
            if entry.get(field_name) != expected_value:
                raise ValidationError(f"matrix variant {variant_name} has invalid {field_name}")
        variant_by_name[variant_name] = expected_spec

    expected_signatures = {
        (read_kind, repetition, variant_name)
        for read_kind in read_kinds
        for repetition in range(1, configured_repetitions + 1)
        for variant_name in variant_by_name
    }
    observed_signatures = set()
    source_commit = config.get("source_commit")
    if not isinstance(source_commit, str) or len(source_commit) != 40 or any(
        character not in "0123456789abcdef" for character in source_commit
    ):
        raise ValidationError("matrix config is missing a valid source_commit")
    shared_dimensions = {
        "readers": configured_readers,
        "transaction_width": 1,
        "distribution": "uniform",
        "working_set": matrix.get("working_set"),
        "key_size": matrix.get("key_size"),
        "value_size": matrix.get("value_size"),
        "read_limit": matrix.get("read_limit"),
        "cache_capacity": matrix.get("cache_capacity"),
        "warmup_ms": matrix.get("warmup_ms"),
        "duration_ms": matrix.get("duration_ms"),
        "source_commit": source_commit,
    }
    for cell in expected_cells:
        if not isinstance(cell, dict):
            raise ValidationError("expected cell must be an object")
        signature = (cell.get("read_kind"), cell.get("repetition"), cell.get("variant"))
        if not isinstance(signature[0], str) or not isinstance(signature[2], str):
            raise ValidationError("expected cell has invalid read kind or variant")
        if signature in observed_signatures:
            raise ValidationError(f"matrix repeats expected cell signature {signature}")
        observed_signatures.add(signature)
        read_kind, repetition, variant_name = signature
        if variant_name not in variant_by_name or read_kind not in read_kinds:
            raise ValidationError(f"expected cell has an unconfigured variant or read kind: {signature}")
        if isinstance(repetition, bool) or not isinstance(repetition, int) or not 1 <= repetition <= configured_repetitions:
            raise ValidationError(f"expected cell has an invalid repetition: {signature}")
        scenario_index = configured_readers * 1_000 + READ_KIND_INDEX[read_kind] * 100
        expected_seed = base_seed + scenario_index * seed_stride + repetition - 1
        variant_spec = variant_by_name[variant_name]
        expected_values = {
            **shared_dimensions,
            "cell_id": f"readers-{configured_readers}-{read_kind}-{variant_name}-rep{repetition}",
            "scenario_index": scenario_index,
            "expected_engine": variant_spec["engine"],
            "expected_collection_policy": variant_spec["collection_policy"],
            "expected_blink_workers": variant_spec.get("blink_workers"),
            "expected_tokio_workers": (
                PHASE0_TOKIO_WORKERS if variant_spec["binary_kind"] == "phase0" else None
            ),
            "expected_read_metrics_enabled": variant_spec[
                "blink_read_observational_metrics_enabled"
            ],
            "read_metrics_mode": variant_spec["read_metrics_mode"],
            "binary_key": variant_spec["binary_key"],
            "binary_env": variant_spec["binary_env"],
            "binary_kind": variant_spec["binary_kind"],
            "expected_raw_repetition": (
                0 if variant_spec["binary_kind"] == "phase0" else repetition
            ),
            "seed": expected_seed,
        }
        for field_name, expected_value in expected_values.items():
            if cell.get(field_name) != expected_value:
                raise ValidationError(
                    f"expected cell {cell.get('cell_id')} has invalid {field_name}"
                )
    if observed_signatures != expected_signatures:
        raise ValidationError("expected cells do not match configured readers, kinds, variants, and repetitions")


def recorded_path(actual_results, raw_path, recorded_results_root):
    original_path = pathlib.Path(raw_path).resolve()
    original_root = (
        pathlib.Path(recorded_results_root).resolve()
        if recorded_results_root is not None
        else pathlib.Path(actual_results).resolve()
    )
    try:
        relative_path = original_path.relative_to(original_root)
    except ValueError as error:
        raise ValidationError(f"recorded artifact path escapes source root: {raw_path}") from error
    resolved_path = (pathlib.Path(actual_results) / relative_path).resolve()
    try:
        resolved_path.relative_to(pathlib.Path(actual_results).resolve())
    except ValueError as error:
        raise ValidationError(f"artifact path escapes results directory: {raw_path}") from error
    return resolved_path


def verify_artifact_manifest(results):
    manifest_path = results / "artifact-sha256.txt"
    if not manifest_path.exists():
        return
    expected_entries = {}
    for line_number, line in enumerate(manifest_path.read_text(encoding="utf-8").splitlines(), 1):
        parts = line.split("  ", 1)
        if len(parts) != 2:
            raise ValidationError(f"invalid artifact manifest line {line_number}")
        digest, relative_path = parts
        if len(digest) != 64 or any(character not in "0123456789abcdef" for character in digest):
            raise ValidationError(f"invalid artifact digest on manifest line {line_number}")
        if relative_path in expected_entries:
            raise ValidationError(f"duplicate artifact manifest path {relative_path}")
        expected_entries[relative_path] = digest
    actual_entries = {
        artifact.relative_to(results).as_posix(): sha256_file(artifact)
        for artifact in results.rglob("*")
        if artifact.is_file() and artifact != manifest_path
    }
    if expected_entries != actual_entries:
        raise ValidationError("artifact manifest does not match result files")


def validate_source_provenance(results, source_commit, source_branch):
    provenance = json.loads((results / "source-provenance.json").read_text(encoding="utf-8"))
    source_before = provenance.get("source_before")
    source_after = provenance.get("source_after")
    if not isinstance(source_before, dict) or not isinstance(source_after, dict):
        raise ValidationError("source provenance is missing before/after records")
    for source_state in (source_before, source_after):
        if source_state.get("commit") != source_commit or source_state.get("branch") != source_branch:
            raise ValidationError("source checkout changed during the matrix")
        if source_state.get("tracked_changes") or source_state.get("unrelated_untracked"):
            raise ValidationError("source checkout was not clean during the matrix")


def validate_matrix(results, recorded_results_root=None):
    results = pathlib.Path(results).resolve()
    verify_artifact_manifest(results)
    config = json.loads((results / "matrix-config.json").read_text(encoding="utf-8"))
    validate_expected_cells(config)
    expected_cells = config["expected_cells"]
    source_commit = config["source_commit"]
    source_branch = config.get("source_branch")
    cell_by_id = {cell["cell_id"]: cell for cell in expected_cells}
    events = load_json_lines(results / "run-order.jsonl")
    starts = {}
    completes = {}
    for event in events:
        event_type = event.get("event")
        cell_id = event.get("cell_id")
        if cell_id not in cell_by_id:
            raise ValidationError(f"run-order contains unexpected cell {cell_id!r}")
        cell = cell_by_id[cell_id]
        if event_type in ("start", "complete"):
            validate_event_identity(event, cell, source_commit, source_branch)
        destination = starts if event_type == "start" else completes if event_type == "complete" else None
        if destination is None:
            if event_type in ("run-failed", "runner-failed"):
                continue
            raise ValidationError(f"unknown run-order event {event_type!r}")
        if cell_id in destination:
            raise ValidationError(f"duplicate {event_type} event for {cell_id}")
        destination[cell_id] = event
    if set(starts) != set(cell_by_id) or set(completes) != set(cell_by_id):
        raise ValidationError("matrix is missing run start or complete events")

    binary_provenance_path = results / "binary-provenance.json"
    binary_provenance = json.loads(binary_provenance_path.read_text(encoding="utf-8"))
    expected_binary_keys = {cell["binary_key"] for cell in expected_cells}
    if set(binary_provenance) != expected_binary_keys:
        raise ValidationError("binary provenance keys do not match configured variants")
    binary_hashes = {}
    for binary_key, binary_record in binary_provenance.items():
        if not isinstance(binary_record, dict):
            raise ValidationError(f"binary provenance is invalid for {binary_key}")
        binary_hash = binary_record.get("sha256")
        if not isinstance(binary_hash, str) or len(binary_hash) != 64 or any(
            character not in "0123456789abcdef" for character in binary_hash
        ):
            raise ValidationError(f"binary provenance has an invalid SHA for {binary_key}")
        binary_hashes[binary_key] = binary_record

    validated_runs = []
    expected_raw_paths = set()
    expected_log_paths = set()
    for cell in expected_cells:
        cell_id = cell["cell_id"]
        started = starts[cell_id]
        completed = completes[cell_id]
        binary_key = cell["binary_key"]
        binary_record = binary_hashes[binary_key]
        binary_hash = started.get("binary_sha256")
        if (
            started.get("binary_key") != binary_key
            or started.get("binary_env") != cell["binary_env"]
            or started.get("binary_path") != binary_record.get("path")
            or started.get("binary_sha256") != binary_record.get("sha256")
            or binary_record.get("environment_variable") != cell["binary_env"]
        ):
            raise ValidationError(f"start event binary provenance mismatch for {cell_id}")
        if (
            completed.get("binary_key") != binary_key
            or completed.get("binary_env") != cell["binary_env"]
            or completed.get("binary_path") != binary_record.get("path")
            or completed.get("binary_sha256") != binary_hash
            or completed.get("exit_code") != 0
        ):
            raise ValidationError(f"benchmark did not complete consistently for {cell_id}")
        for path_field in ("output", "log"):
            if started.get(path_field) != completed.get(path_field):
                raise ValidationError(f"start/complete {path_field} path mismatch for {cell_id}")
        output_path = recorded_path(results, started.get("output", ""), recorded_results_root)
        log_path = recorded_path(results, started.get("log", ""), recorded_results_root)
        raw_root = (results / "raw").resolve()
        if output_path.parent != raw_root or not output_path.is_file():
            raise ValidationError(f"raw output is missing or outside raw directory for {cell_id}")
        if log_path.parent != raw_root or not log_path.is_file():
            raise ValidationError(f"benchmark log is missing or outside raw directory for {cell_id}")
        if completed.get("output_sha256") != sha256_file(output_path):
            raise ValidationError(f"raw output checksum mismatch for {cell_id}")
        if completed.get("log_sha256") != sha256_file(log_path):
            raise ValidationError(f"benchmark log checksum mismatch for {cell_id}")
        raw_records = load_json_lines(output_path)
        if len(raw_records) != 1:
            raise ValidationError(f"expected one raw record for {cell_id}, found {len(raw_records)}")
        validated_raw = validate_raw_record(raw_records[0], cell, source_commit)
        expected_raw_paths.add(output_path)
        expected_log_paths.add(log_path)
        validated_runs.append(
            {"cell": cell, "start": started, "complete": completed, "raw_record": validated_raw}
        )
    actual_raw_paths = {path.resolve() for path in (results / "raw").glob("*.jsonl")}
    actual_log_paths = {path.resolve() for path in (results / "raw").glob("*.log")}
    if actual_raw_paths != expected_raw_paths or actual_log_paths != expected_log_paths:
        raise ValidationError("raw/log file set does not match the configured matrix")
    environment = json.loads((results / "environment.json").read_text(encoding="utf-8"))
    if environment.get("git_commit") != source_commit:
        raise ValidationError("environment checkout SHA does not match matrix source_commit")
    validate_source_provenance(results, source_commit, source_branch)
    manifest_path = results / "manifest.json"
    if manifest_path.exists():
        accepted_manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        if (
            accepted_manifest.get("accepted") is not True
            or accepted_manifest.get("source_commit") != source_commit
            or accepted_manifest.get("planned_count") != config["planned_count"]
            or accepted_manifest.get("completed_count") != config["planned_count"]
            or accepted_manifest.get("binary_sha256") != binary_provenance
        ):
            raise ValidationError("accepted manifest provenance does not match matrix inputs")
    return {
        "results": results,
        "config": config,
        "environment": environment,
        "binary_sha256": binary_provenance,
        "validated_runs": validated_runs,
    }


def optional_rss_value(record, requested_metric):
    rss_values = record.get("rss_kib")
    if isinstance(rss_values, dict):
        nested_value = rss_values.get("end" if requested_metric == "end" else "hwm")
        if isinstance(nested_value, (int, float)) and not isinstance(nested_value, bool):
            return float(nested_value)
    matching_samples = []
    suffix = "_rss_kib"
    for field_name, field_value in record.items():
        if field_name.startswith("resource_sample_") and field_name.endswith(suffix):
            if isinstance(field_value, (int, float)) and not isinstance(field_value, bool):
                matching_samples.append((field_name, float(field_value)))
    if matching_samples:
        if requested_metric == "end":
            return matching_samples[-1][1]
        return max(value for _, value in matching_samples)
    direct_name = "rss_end_kib" if requested_metric == "end" else "rss_hwm_kib"
    direct_value = record.get(direct_name)
    if isinstance(direct_value, (int, float)) and not isinstance(direct_value, bool):
        return float(direct_value)
    return None


def median_range(values):
    numeric_values = [float(value) for value in values if value is not None and math.isfinite(float(value))]
    if not numeric_values:
        return {"median": None, "min": None, "max": None}
    return {
        "median": statistics.median(numeric_values),
        "min": min(numeric_values),
        "max": max(numeric_values),
    }


def metric_values(record, cell):
    if cell["binary_kind"] == "phase0":
        throughput = numeric_value(record, (f"{cell['read_kind']}_ops_per_second",))
        read_p99 = numeric_value(record, ("read_p99_us",))
    else:
        throughput = numeric_value(
            record,
            (
                "successful_read_operations_per_second",
                "read_operations_per_second",
                "total_operations_per_second",
            ),
        )
        read_p99 = numeric_value(record, ("read_p99_us", "p99_us"))
    return {
        "throughput_ops_per_second": throughput,
        "read_p99_us": read_p99,
        "cpu_utilization_percent_one_core": numeric_value(
            record, ("cpu_utilization_percent_one_core",)
        ),
        "rss_end_kib": optional_rss_value(record, "end"),
        "rss_hwm_kib": optional_rss_value(record, "hwm"),
        "read_operations_metric": numeric_value(record, ("read_operations_metric",)),
    }


def build_summary(validated):
    groups = defaultdict(list)
    for run in validated["validated_runs"]:
        cell = run["cell"]
        groups[(cell["readers"], cell["read_kind"], cell["variant"])].append(run)
    summary_rows = []
    for (reader_count, read_kind, variant), runs in sorted(groups.items()):
        first_cell = runs[0]["cell"]
        metric_data = {
            "throughput_ops_per_second": [],
            "read_p99_us": [],
            "cpu_utilization_percent_one_core": [],
            "rss_end_kib": [],
            "rss_hwm_kib": [],
            "read_operations_metric": [],
        }
        for run in runs:
            for metric_name, value in metric_values(run["raw_record"], run["cell"]).items():
                metric_data[metric_name].append(value)
        summary_rows.append(
            {
                "variant": variant,
                "engine": first_cell["expected_engine"],
                "collection_policy": first_cell["expected_collection_policy"],
                "read_metrics_mode": first_cell["read_metrics_mode"],
                "blink_read_observational_metrics_enabled": first_cell[
                    "expected_read_metrics_enabled"
                ],
                "reader_execution_model": (
                    f"{reader_count} reader tasks on {PHASE0_TOKIO_WORKERS} Tokio workers"
                    if first_cell["binary_kind"] == "phase0"
                    else f"{reader_count} RocksDB reader OS threads"
                ),
                "reader_execution_model": (
                    f"{reader_count} reader tasks on {PHASE0_TOKIO_WORKERS} Tokio workers"
                    if first_cell["binary_kind"] == "phase0"
                    else f"{reader_count} RocksDB reader OS threads"
                ),
                "readers": reader_count,
                "read_kind": read_kind,
                "working_set": first_cell["working_set"],
                "transaction_width": first_cell["transaction_width"],
                "distribution": first_cell["distribution"],
                "key_size": first_cell["key_size"],
                "value_size": first_cell["value_size"],
                "read_limit": first_cell["read_limit"],
                "sync_contract": "durable-return",
                "warmup_ms": first_cell["warmup_ms"],
                "duration_ms": first_cell["duration_ms"],
                "source_commit": validated["config"]["source_commit"],
                "repetitions": len(runs),
                "seeds": [run["cell"]["seed"] for run in runs],
                "metrics": {
                    metric_name: median_range(values)
                    for metric_name, values in metric_data.items()
                },
            }
        )
    return {
        "schema_version": 1,
        "accepted": True,
        "generated_at_utc": timestamp_utc(),
        "analysis_scope": (
            "Descriptive pure-read measurements. B-link metrics-on and metrics-off share the phase0 workload seed; "
            "phase0 uses reader tasks on two Tokio workers while RocksDB uses one OS thread per reader. "
            "The RocksDB driver uses an independent key stream, so cross-engine key-trace equality is not claimed."
        ),
        "planned_count": validated["config"]["planned_count"],
        "completed_count": len(validated["validated_runs"]),
        "source_commit": validated["config"]["source_commit"],
        "matrix_config": validated["config"],
        "environment": validated["environment"],
        "rows": summary_rows,
    }


def write_summary(validated):
    results = validated["results"]
    summary = build_summary(validated)
    write_json(results / SUMMARY_NAME, summary)
    status = {
        "accepted": True,
        "planned_count": summary["planned_count"],
        "completed_count": summary["completed_count"],
        "summary_json": SUMMARY_NAME,
        "source_commit": summary["source_commit"],
    }
    write_json(results / "matrix-status.json", status)
    write_json(
        results / "manifest.json",
        {
            "accepted": True,
            "planned_count": summary["planned_count"],
            "completed_count": summary["completed_count"],
            "source_commit": summary["source_commit"],
            "binary_sha256": validated["binary_sha256"],
            "results": str(results),
        },
    )
    write_artifact_manifest(results)
    verify_artifact_manifest(results)
    return summary


def write_failure_state(results, planned_count, completed_count, source_commit, error):
    write_json(
        results / "matrix-status.json",
        {
            "accepted": False,
            "planned_count": planned_count,
            "completed_count": completed_count,
            "source_commit": source_commit,
            "failure": str(error),
            "updated_at_utc": timestamp_utc(),
        },
    )
    write_json(
        results / "runner-failure.json",
        {"failure": str(error), "failure_type": type(error).__name__},
    )
    write_artifact_manifest(results)


def run_matrix(arguments):
    repository = pathlib.Path(__file__).resolve().parents[3]
    source_state_before = require_clean_source(repository, arguments.results)
    if arguments.results.exists():
        raise RunnerError(f"results directory must be fresh: {arguments.results}")
    data_namespace = arguments.data_root / arguments.results.name
    if data_namespace.exists():
        raise RunnerError(f"data namespace must be fresh: {data_namespace}")
    expected_cells = build_expected_cells(arguments, source_state_before["commit"])
    arguments.results.mkdir(parents=True)
    config = {
        "schema_version": 1,
        "created_at_utc": timestamp_utc(),
        "source_commit": source_state_before["commit"],
        "source_branch": source_state_before["branch"],
        "planned_count": len(expected_cells),
        "config": {
            "readers": arguments.readers,
            "tokio_workers": PHASE0_TOKIO_WORKERS,
            "reader_execution_models": {
                "phase0": "one async reader task per reader on two Tokio workers",
                "rocksdb": "one OS thread per reader",
            },
            "read_kinds": arguments.read_kinds,
            "variants": [
                {
                    "name": variant,
                    "engine": VARIANT_SPECS[variant]["engine"],
                    "collection_policy": VARIANT_SPECS[variant]["collection_policy"],
                    "binary_key": VARIANT_SPECS[variant]["binary_key"],
                    "binary_env": VARIANT_SPECS[variant]["binary_env"],
                    "binary_kind": VARIANT_SPECS[variant]["binary_kind"],
                    "read_metrics_mode": VARIANT_SPECS[variant]["read_metrics_mode"],
                    "blink_read_observational_metrics_enabled": VARIANT_SPECS[variant][
                        "blink_read_observational_metrics_enabled"
                    ],
                    "blink_workers": VARIANT_SPECS[variant].get("blink_workers"),
                    "tokio_workers": VARIANT_SPECS[variant].get("tokio_workers"),
                    "engine_options": list(VARIANT_SPECS[variant]["engine_options"]),
                }
                for variant in arguments.variants
            ],
            "repetitions": arguments.repetitions,
            "warmup_ms": arguments.warmup_ms,
            "duration_ms": arguments.duration_ms,
            "transaction_width": 1,
            "distribution": "uniform",
            "working_set": arguments.working_set,
            "key_size": arguments.key_size,
            "value_size": arguments.value_size,
            "read_limit": arguments.read_limit,
            "cache_capacity": arguments.cache_capacity,
            "sync_mode": "real",
            "sync_contract": "durable-return",
            "base_seed": arguments.base_seed,
            "seed_stride": SEED_STRIDE,
            "data_root": str(arguments.data_root),
            "data_namespace": str(data_namespace),
        },
        "expected_cells": expected_cells,
    }
    write_json(arguments.results / "matrix-config.json", config)
    write_json(
        arguments.results / "source-provenance.json",
        {"source_before": source_state_before, "source_after": None},
    )
    write_json(
        arguments.results / "environment.json",
        environment_record(repository, arguments.data_root),
    )
    write_json(
        arguments.results / "matrix-status.json",
        {
            "accepted": False,
            "planned_count": len(expected_cells),
            "completed_count": 0,
            "source_commit": source_state_before["commit"],
            "state": "running",
        },
    )

    binary_registry = {}
    binary_provenance = {}
    try:
        for variant in arguments.variants:
            variant_spec = VARIANT_SPECS[variant]
            probe_cell = next(
                cell for cell in expected_cells if cell["variant"] == variant
            )
            binary_path = binary_path_for(probe_cell)
            binary_hash = sha256_file(binary_path)
            prior_entry = binary_provenance.setdefault(
                variant_spec["binary_key"],
                {
                    "path": str(binary_path),
                    "sha256": binary_hash,
                    "environment_variable": variant_spec["binary_env"],
                },
            )
            if prior_entry["path"] != str(binary_path) or prior_entry["sha256"] != binary_hash:
                raise RunnerError(f"binary provenance differs for {variant}")
        write_json(arguments.results / "binary-provenance.json", binary_provenance)
        run_count = 0
        for read_kind in arguments.read_kinds:
            scenario_index = arguments.readers * 1_000 + READ_KIND_INDEX[read_kind] * 100
            for repetition in range(1, arguments.repetitions + 1):
                seed = arguments.base_seed + scenario_index * SEED_STRIDE + repetition - 1
                variant_rotation = (repetition + READ_KIND_INDEX[read_kind]) % len(
                    arguments.variants
                )
                ordered_variants = (
                    arguments.variants[variant_rotation:]
                    + arguments.variants[:variant_rotation]
                )
                for variant in ordered_variants:
                    cell = next(
                        candidate
                        for candidate in expected_cells
                        if candidate["read_kind"] == read_kind
                        and candidate["repetition"] == repetition
                        and candidate["variant"] == variant
                    )
                    execute_cell(
                        arguments,
                        arguments.results,
                        repository,
                        cell,
                        binary_registry,
                        source_state_before,
                    )
                    run_count += 1
        source_state_after = require_clean_source(repository, arguments.results)
        if source_state_after["commit"] != source_state_before["commit"]:
            raise RunnerError("source checkout changed during read metrics matrix")
        write_json(
            arguments.results / "source-provenance.json",
            {"source_before": source_state_before, "source_after": source_state_after},
        )
        for binary_key, binary_record in binary_registry.items():
            binary_path = pathlib.Path(binary_record["path"])
            if sha256_file(binary_path) != binary_record["sha256"]:
                raise RunnerError(f"binary changed during matrix: {binary_key}")
        validated = validate_matrix(arguments.results)
        summary = write_summary(validated)
    except BaseException as error:
        for accepted_path in (arguments.results / SUMMARY_NAME, arguments.results / "manifest.json"):
            if accepted_path.exists():
                accepted_path.unlink()
        event_path = arguments.results / "run-order.jsonl"
        completed_count = (
            sum(event.get("event") == "complete" for event in load_json_lines(event_path))
            if event_path.exists()
            else 0
        )
        write_failure_state(
            arguments.results,
            len(expected_cells),
            completed_count,
            source_state_before["commit"],
            error,
        )
        raise
    return summary


def main(argv=None):
    arguments = parse_arguments(argv)
    try:
        summary = run_matrix(arguments)
    except (RunnerError, ValidationError, OSError, subprocess.SubprocessError) as error:
        print(f"read metrics matrix failed: {error}", file=sys.stderr)
        return 1
    print(
        f"accepted={summary['completed_count']}/{summary['planned_count']} "
        f"rows={len(summary['rows'])} results={arguments.results}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
