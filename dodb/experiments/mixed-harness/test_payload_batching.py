import json
import pathlib
import shutil
import sys
import tempfile
import unittest


SCRIPT_DIRECTORY = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIRECTORY))

from summarize_payload_batching import (
    ValidationError,
    sha256_file,
    validate_expected_matrix,
    validate_matrix,
    validate_raw_record,
    metric_value,
    write_summary,
)
from run_payload_batching_matrix import (
    VARIANT_SPECS,
    build_expected_cells,
    build_matrix_config,
    parse_arguments,
)


SOURCE_COMMIT = "a" * 40
VARIANTS = (
    ("main-btree", "main-btree", "native-main", "phase0-bench"),
    ("parallel-blink-current", "parallel-blink", "current", "phase0-bench"),
    ("parallel-blink-main-parity", "parallel-blink", "main-parity", "phase0-bench"),
    ("rocksdb", "rocksdb", "not-applicable", "rocksdb-bench"),
)


def sample_cells():
    cells = []
    for value_mode in ("constant", "changing"):
        for variant, expected_engine, collection_policy, binary_key in VARIANTS:
            repetition = 1
            cells.append(
                {
                    "cell_id": f"clients-4-mix-50-50-{value_mode}-{variant}-rep1",
                    "scenario_index": 4550,
                    "variant": variant,
                    "expected_engine": expected_engine,
                    "expected_collection_policy": collection_policy,
                    "binary_key": binary_key,
                    "binary_kind": "rocksdb" if variant == "rocksdb" else "phase0",
                    "clients": 4,
                    "read_percent": 50,
                    "write_percent": 50,
                    "value_mode": value_mode,
                    "repetition": repetition,
                    "expected_raw_repetition": 1 if variant == "rocksdb" else 0,
                    "seed": 983_590_950,
                    "transaction_width": 1,
                    "distribution": "uniform",
                    "working_set": 10_000,
                    "key_size": 16,
                    "value_size": 512,
                    "warmup_ms": 200,
                    "duration_ms": 500,
                    "cache_capacity": 16_384,
                    "read_limit": 16,
                }
            )
    return cells


def sample_raw_record(cell, trace_hash=None, policy=None):
    record = {
        "record_type": "crossdb-run" if cell["expected_engine"] == "rocksdb" else "run",
        "engine": cell["expected_engine"],
        "collection_policy": policy or cell["expected_collection_policy"],
        "client_workers": cell["clients"],
        "seed": cell["seed"],
        "transaction_width": cell["transaction_width"],
        "requested_read_percent": cell["read_percent"],
        "working_set": cell["working_set"],
        "key_size": cell["key_size"],
        "value_size": cell["value_size"],
        "distribution": cell["distribution"],
        "mixed_value_mode": cell["value_mode"],
        "mixed_value_generator": "legacy_constant_v1" if cell["value_mode"] == "constant" else "seeded_nonrepeating_v1",
        "logical_trace_prefix_operations": 1000,
        "logical_trace_prefix_hash": trace_hash or ("1" * 16 if cell["value_mode"] == "constant" else "2" * 16),
        "sync_contract": "durable-return",
        "git_commit": SOURCE_COMMIT,
        "warmup_ms": cell["warmup_ms"],
        "requested_duration_ms": cell["duration_ms"],
        "repetition": cell["expected_raw_repetition"],
        "operation": "mixed",
        "errors": 0,
        "conflicts": 0,
        "aggregate_ops_per_second": 1000.0,
        "read_ops_per_second": 500.0,
        "logical_tx_per_second": 500.0,
        "successful_read_percent": 50.0,
        "successful_write_percent": 50.0,
        "read_p50_us": 12.0,
        "read_p95_us": 25.0,
        "read_p99_us": 35.0,
        "write_p50_us": 40.0,
        "write_p95_us": 75.0,
        "write_p99_us": 100.0,
        "cpu_utilization_percent_one_core": 80.0,
        "avg_transactions_per_group": 3.0,
        "wal_syncs_delta": 20,
        "wal_bytes_delta": 2048,
        "wal_sync_nanos_total": 1_000_000,
        "page_images_delta": 8,
        "wal_page_delta_spans_delta": 4,
        "wal_page_delta_changed_bytes_delta": 128,
        "parallel_workers": 0,
    }
    if cell["expected_engine"] != "rocksdb":
        record["overloads"] = 0
        record["sync_mode"] = "real"
        if cell.get("expected_blink_workers") is not None:
            record["blink_workers"] = cell["expected_blink_workers"]
        if cell.get("expected_parallel_background_min_operations") is not None:
            record["parallel_background_min_operations"] = cell[
                "expected_parallel_background_min_operations"
            ]
        if cell.get("expected_blink_read_observational_metrics_enabled") is not None:
            record["blink_read_observational_metrics_enabled"] = cell[
                "expected_blink_read_observational_metrics_enabled"
            ]
        if cell.get("expected_parallel_background_worker_dispatches_metric") is True:
            record["parallel_background_worker_dispatches_delta"] = 6
            record["parallel_groups_delta"] = 12
    else:
        record["verification"] = {"passed": True}
        record["settings"] = {
            "effective": {
                "write_options_sync": True,
                "write_options_disable_wal": False,
            }
        }
    return record


def write_synthetic_matrix(results, mutation=None, drop_last_complete=False):
    results.mkdir(parents=True, exist_ok=True)
    raw_directory = results / "raw"
    raw_directory.mkdir()
    cells = sample_cells()
    config = {
        "schema_version": 1,
        "source_commit": SOURCE_COMMIT,
        "planned_count": len(cells),
        "config": {
            "clients": [4],
            "variants": [
                {
                    "name": variant[0],
                    "engine": variant[1],
                    "collection_policy": variant[2],
                    "binary_key": variant[3],
                }
                for variant in VARIANTS
            ],
            "value_modes": ["constant", "changing"],
            "mixes": ["50/50"],
            "repetitions": 1,
            "transaction_width": 1,
            "distribution": "uniform",
            "working_set": 10_000,
            "key_size": 16,
            "value_size": 512,
            "warmup_ms": 200,
            "duration_ms": 500,
            "cache_capacity": 16_384,
            "read_limit": 16,
            "base_seed": 979_000_000,
            "seed_stride": 1_009,
            "sync_mode": "real",
            "sync_contract": "durable-return",
        },
        "expected_cells": cells,
    }
    (results / "matrix-config.json").write_text(json.dumps(config), encoding="utf-8")
    (results / "environment.json").write_text(
        json.dumps({"git_commit": SOURCE_COMMIT}), encoding="utf-8"
    )
    events = []
    for cell_index, cell in enumerate(cells):
        record = sample_raw_record(cell)
        if mutation is not None and cell_index == 0:
            mutation(record)
        output_path = raw_directory / f"{cell['cell_id']}.jsonl"
        output_path.write_text(json.dumps(record) + "\n", encoding="utf-8")
        log_path = raw_directory / f"{cell['cell_id']}.log"
        log_path.write_text("synthetic benchmark output\n", encoding="utf-8")
        binary_hash = "b" * 64 if cell["binary_key"] == "phase0-bench" else "c" * 64
        event_identity = {
            "cell_id": cell["cell_id"],
            "variant": cell["variant"],
            "value_mode": cell["value_mode"],
            "clients": cell["clients"],
            "read_percent": cell["read_percent"],
            "write_percent": cell["write_percent"],
            "repetition": cell["repetition"],
            "seed": cell["seed"],
            "scenario_index": cell["scenario_index"],
            "checkout_sha": SOURCE_COMMIT,
        }
        started = {
            "event": "start",
            **event_identity,
            "binary_key": cell["binary_key"],
            "binary_sha256": binary_hash,
            "output": str(output_path.resolve()),
            "log": str(log_path.resolve()),
        }
        completed = {
            "event": "complete",
            **event_identity,
            "binary_key": cell["binary_key"],
            "binary_sha256": binary_hash,
            "exit_code": 0,
            "output": str(output_path.resolve()),
            "log": str(log_path.resolve()),
            "output_sha256": sha256_file(output_path),
            "log_sha256": sha256_file(log_path),
        }
        events.extend((started, completed))
    if drop_last_complete:
        events.pop()
    (results / "run-order.jsonl").write_text(
        "".join(json.dumps(event) + "\n" for event in events), encoding="utf-8"
    )
    return results


class PayloadBatchingSummaryTests(unittest.TestCase):
    def test_parallel_workers_one_variant_maps_engine_policy_and_raw_setting(self):
        variant_name = "parallel-blink-main-parity-workers1"
        arguments = parse_arguments(
            (
                "--results",
                "/tmp/payload-batching-workers1",
                "--clients",
                "4",
                "--variants",
                variant_name,
                "--value-modes",
                "changing",
                "--repetitions",
                "1",
            )
        )
        cell = build_expected_cells(arguments, SOURCE_COMMIT)[0]
        variant_spec = VARIANT_SPECS[variant_name]
        self.assertEqual(
            variant_spec["engine_options"],
            ("--blink-collection-policy", "main-parity", "--blink-workers", "1"),
        )
        matrix_config = {
            "config": {
                "clients": [4],
                "variants": [
                    {
                        "name": variant_name,
                        "engine": variant_spec["engine"],
                        "collection_policy": variant_spec["collection_policy"],
                        "binary_key": variant_spec["binary_key"],
                        "blink_workers": 1,
                    }
                ],
                "value_modes": ["changing"],
                "mixes": ["50/50"],
                "repetitions": 1,
                "transaction_width": 1,
                "distribution": "uniform",
                "working_set": 10_000,
                "key_size": 16,
                "value_size": 512,
                "warmup_ms": 2_000,
                "duration_ms": 5_000,
                "cache_capacity": 16_384,
                "read_limit": 16,
                "base_seed": 979_000_000,
                "seed_stride": 1_009,
                "sync_mode": "real",
                "sync_contract": "durable-return",
            }
        }
        validate_expected_matrix(matrix_config, [cell])
        record = sample_raw_record(cell)
        record["blink_workers"] = 1
        validated = validate_raw_record(record, cell, SOURCE_COMMIT)
        self.assertEqual(validated["raw_record"]["engine"], "parallel-blink")
        self.assertEqual(validated["raw_record"]["collection_policy"], "main-parity")

    def test_adaptive_background_variant_requires_runtime_threshold_and_metrics_mode(self):
        variant_name = "parallel-blink-main-parity-adaptive32"
        arguments = parse_arguments(
            (
                "--results",
                "/tmp/payload-batching-adaptive32",
                "--clients",
                "4",
                "--variants",
                variant_name,
                "--value-modes",
                "changing",
                "--repetitions",
                "1",
            )
        )
        cell = build_expected_cells(arguments, SOURCE_COMMIT)[0]
        variant_spec = VARIANT_SPECS[variant_name]
        self.assertEqual(
            variant_spec["engine_options"],
            (
                "--blink-collection-policy",
                "main-parity",
                "--blink-workers",
                "2",
                "--parallel-background-min-operations",
                "32",
            ),
        )
        self.assertEqual(cell["expected_parallel_background_min_operations"], 32)
        self.assertIs(cell["expected_blink_read_observational_metrics_enabled"], True)
        matrix_config = {
            "schema_version": 2,
            "config": {
                "clients": [4],
                "variants": [
                    {
                        "name": variant_name,
                        "engine": variant_spec["engine"],
                        "collection_policy": variant_spec["collection_policy"],
                        "binary_key": variant_spec["binary_key"],
                        "blink_workers": 2,
                        "parallel_background_min_operations": 32,
                        "blink_read_observational_metrics_enabled": True,
                    }
                ],
                "value_modes": ["changing"],
                "mixes": ["50/50"],
                "repetitions": 1,
                "transaction_width": 1,
                "distribution": "uniform",
                "working_set": 10_000,
                "key_size": 16,
                "value_size": 512,
                "warmup_ms": 2_000,
                "duration_ms": 5_000,
                "cache_capacity": 16_384,
                "read_limit": 16,
                "base_seed": 979_000_000,
                "seed_stride": 1_009,
                "sync_mode": "real",
                "sync_contract": "durable-return",
            },
        }
        validate_expected_matrix(matrix_config, [cell])
        raw_record = sample_raw_record(cell)
        raw_record.update(
            {
                "blink_workers": 2,
                "parallel_background_min_operations": 32,
                "blink_read_observational_metrics_enabled": True,
            }
        )
        validate_raw_record(raw_record, cell, SOURCE_COMMIT)
        self.assertEqual(
            metric_value(raw_record, "parallel_background_dispatches_per_parallel_group"),
            0.5,
        )
        raw_record.pop("parallel_background_worker_dispatches_delta")
        with self.assertRaisesRegex(
            ValidationError,
            "parallel_background_worker_dispatches_delta",
        ):
            validate_raw_record(raw_record, cell, SOURCE_COMMIT)
        raw_record["parallel_background_worker_dispatches_delta"] = 6
        raw_record.pop("parallel_background_min_operations")
        with self.assertRaisesRegex(ValidationError, "parallel_background_min_operations"):
            validate_raw_record(raw_record, cell, SOURCE_COMMIT)
        raw_record["parallel_background_min_operations"] = 32
        raw_record["blink_read_observational_metrics_enabled"] = False
        with self.assertRaisesRegex(ValidationError, "blink_read_observational_metrics_enabled"):
            validate_raw_record(raw_record, cell, SOURCE_COMMIT)

    def test_default_matrix_plans_48_runs_and_smoke_plans_eight_with_paired_seeds(self):
        default_arguments = parse_arguments(("--results", "/tmp/payload-batching-default"))
        default_cells = build_expected_cells(default_arguments, SOURCE_COMMIT)
        self.assertEqual(len(default_cells), 48)
        self.assertEqual(default_arguments.value_modes, ["constant", "changing"])
        self.assertEqual(default_arguments.clients, [4, 64])
        for cell in default_cells:
            expected_repetition = cell["repetition"] if cell["binary_kind"] == "rocksdb" else 0
            self.assertEqual(cell["expected_raw_repetition"], expected_repetition)

        smoke_arguments = parse_arguments(
            (
                "--results",
                "/tmp/payload-batching-smoke",
                "--clients",
                "4",
                "--repetitions",
                "1",
                "--duration-ms",
                "500",
                "--warmup-ms",
                "200",
            )
        )
        smoke_cells = build_expected_cells(smoke_arguments, SOURCE_COMMIT)
        self.assertEqual(len(smoke_cells), 8)
        seed_by_mode = {cell["value_mode"]: cell["seed"] for cell in smoke_cells}
        self.assertEqual(seed_by_mode["constant"], seed_by_mode["changing"])

    def test_generated_schema_two_default_and_worker_matrices_match_validator(self):
        configurations = (
            (
                parse_arguments(("--results", "/tmp/payload-batching-schema2-default")),
                48,
            ),
            (
                parse_arguments(
                    (
                        "--results",
                        "/tmp/payload-batching-schema2-workers",
                        "--variants",
                        "parallel-blink-main-parity-workers1,parallel-blink-main-parity-adaptive32",
                    )
                ),
                24,
            ),
        )
        for arguments, expected_count in configurations:
            with self.subTest(expected_count=expected_count):
                matrix_config = build_matrix_config(
                    arguments,
                    SOURCE_COMMIT,
                    "test-branch",
                    pathlib.Path("/bench/zfs/db"),
                )
                self.assertEqual(matrix_config["schema_version"], 2)
                self.assertEqual(matrix_config["planned_count"], expected_count)
                self.assertEqual(len(matrix_config["expected_cells"]), expected_count)
                validate_expected_matrix(
                    matrix_config,
                    matrix_config["expected_cells"],
                )
                rocks_cells = [
                    cell
                    for cell in matrix_config["expected_cells"]
                    if cell["binary_kind"] == "rocksdb"
                ]
                self.assertTrue(
                    all(
                        cell["expected_parallel_background_worker_dispatches_metric"] is False
                        for cell in rocks_cells
                    )
                )

    def test_accepts_complete_paired_matrix_and_writes_expected_rows(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            results = write_synthetic_matrix(pathlib.Path(temporary_directory) / "results")
            validated = validate_matrix(results)
            summary = write_summary(validated)
            self.assertEqual(summary["planned_count"], 8)
            self.assertEqual(summary["completed_count"], 8)
            self.assertEqual(len(summary["rows"]), 8)
            baseline_rows = [row for row in summary["rows"] if row["variant"] == "main-btree"]
            self.assertEqual(len(baseline_rows), 2)
            self.assertEqual(
                baseline_rows[0]["paired_ratios_vs_main_btree"]["aggregate_ops_per_second"]["median"],
                1.0,
            )
            self.assertEqual(
                summary["rows"][0]["metrics"]["wal_bytes_per_sync"]["median"],
                102.4,
            )
            self.assertIsNone(
                summary["rows"][0]["metrics"]["parallel_background_worker_dispatches_delta"][
                    "median"
                ]
            )
            self.assertIsNone(
                summary["rows"][0]["metrics"]["parallel_background_dispatches_per_parallel_group"][
                    "median"
                ]
            )

    def test_validates_relocated_results_without_rewriting_recorded_paths(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = pathlib.Path(temporary_directory)
            original = write_synthetic_matrix(root / "original")
            relocated = root / "relocated"
            shutil.copytree(original, relocated)
            recorded_order = (relocated / "run-order.jsonl").read_bytes()
            validated = validate_matrix(relocated, recorded_results_root=original)
            self.assertEqual(len(validated["validated_runs"]), 8)
            self.assertEqual((relocated / "run-order.jsonl").read_bytes(), recorded_order)
            with self.assertRaisesRegex(ValidationError, "escapes"):
                validate_matrix(relocated, recorded_results_root=root / "wrong")

    def test_rejects_missing_required_raw_field(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            results = write_synthetic_matrix(
                pathlib.Path(temporary_directory) / "results",
                mutation=lambda record: record.pop("mixed_value_mode"),
            )
            with self.assertRaisesRegex(ValidationError, "mixed_value_mode"):
                validate_matrix(results)
            self.assertFalse((results / "payload-batching-summary.json").exists())

    def test_rejects_variant_policy_mismatch(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            results = write_synthetic_matrix(
                pathlib.Path(temporary_directory) / "results",
                mutation=lambda record: record.update({"collection_policy": "current"}),
            )
            with self.assertRaisesRegex(ValidationError, "collection_policy mismatch"):
                validate_matrix(results)

    def test_rejects_incomplete_start_complete_matrix(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            results = write_synthetic_matrix(
                pathlib.Path(temporary_directory) / "results", drop_last_complete=True
            )
            with self.assertRaisesRegex(ValidationError, "missing complete events"):
                validate_matrix(results)

    def test_rejects_cross_variant_trace_mismatch(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            results = write_synthetic_matrix(
                pathlib.Path(temporary_directory) / "results",
                mutation=lambda record: record.update({"logical_trace_prefix_hash": "f" * 16}),
            )
            with self.assertRaisesRegex(ValidationError, "trace hash mismatch"):
                validate_matrix(results)


if __name__ == "__main__":
    unittest.main()
