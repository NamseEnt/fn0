import json
import pathlib
import shutil
import sys
import tempfile
import unittest


SCRIPT_DIRECTORY = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import run_read_metrics_matrix as read_metrics_runner
from summarize_payload_batching import ValidationError, sha256_file


SOURCE_COMMIT = "a" * 40
SOURCE_BRANCH = "experiment/read-metrics-test"


def make_arguments(results_path, *extra_arguments):
    return read_metrics_runner.parse_arguments(
        ["--results", str(results_path), *extra_arguments]
    )


def make_raw_record(cell, checkout_sha=SOURCE_COMMIT, repetition=None):
    if repetition is None:
        repetition = cell["expected_raw_repetition"]
    record = {
        "record_type": "run",
        "engine": cell["expected_engine"],
        "git_commit": checkout_sha,
        "seed": cell["seed"],
        "suite": "read-scaling",
        "workload": {
            "get": "100%-read-get",
            "query": "100%-read-query",
        }[cell["read_kind"]],
        "readers": cell["readers"],
        "writers": 0,
        "tokio_workers": 2,
        "read_kind": cell["read_kind"],
        "transaction_width": 1,
        "distribution": cell["distribution"],
        "working_set": cell["working_set"],
        "key_size": cell["key_size"],
        "value_size": cell["value_size"],
        "read_limit": cell["read_limit"],
        "sync_mode": "real",
        "sync_contract": "durable-return",
        "collection_policy": cell["expected_collection_policy"],
        "blink_read_observational_metrics_enabled": cell[
            "expected_read_metrics_enabled"
        ],
        "blink_workers": cell.get("expected_blink_workers", 0),
        "warmup_ms": cell["warmup_ms"],
        "requested_duration_ms": cell["duration_ms"],
        "repetition": repetition,
        "errors": 0,
        "conflicts": 0,
        "overloads": 0,
        "successful_reads": 100,
        "successful_read_percent": 100.0,
        "attempted_read_percent": 100.0,
        "attempted_write_transactions": 0,
        "successful_write_transactions": 0,
        "read_operations_metric": 100
        if cell["expected_read_metrics_enabled"]
        else 0,
        f"{cell['read_kind']}_ops_per_second": 1000.0,
        "read_p99_us": 25.0,
    }
    return record


def write_matrix_fixture(results_path, recorded_root):
    arguments = make_arguments(
        results_path,
        "--readers",
        "2",
        "--read-kinds",
        "get",
        "--variants",
        "blink-metrics-on,blink-metrics-off",
        "--repetitions",
        "1",
        "--warmup-ms",
        "0",
        "--duration-ms",
        "10",
    )
    expected_cells = read_metrics_runner.build_expected_cells(arguments, SOURCE_COMMIT)
    variant_config = []
    for variant in arguments.variants:
        specification = read_metrics_runner.VARIANT_SPECS[variant]
        variant_config.append(
            {
                "name": variant,
                "engine": specification["engine"],
                "collection_policy": specification["collection_policy"],
                "binary_key": specification["binary_key"],
                "binary_env": specification["binary_env"],
                "binary_kind": specification["binary_kind"],
                "read_metrics_mode": specification["read_metrics_mode"],
                "blink_read_observational_metrics_enabled": specification[
                    "blink_read_observational_metrics_enabled"
                ],
                "blink_workers": specification.get("blink_workers"),
                "tokio_workers": specification.get("tokio_workers"),
                "engine_options": list(specification.get("engine_options", ())),
            }
        )
    config = {
        "schema_version": 1,
        "source_commit": SOURCE_COMMIT,
        "source_branch": SOURCE_BRANCH,
        "planned_count": len(expected_cells),
        "config": {
            "readers": arguments.readers,
            "read_kinds": arguments.read_kinds,
            "reader_execution_models": {
                "phase0": "one async reader task per reader on two Tokio workers",
                "rocksdb": "one OS thread per reader",
            },
            "variants": variant_config,
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
            "tokio_workers": 2,
            "sync_mode": "real",
            "sync_contract": "durable-return",
            "base_seed": arguments.base_seed,
            "seed_stride": read_metrics_runner.SEED_STRIDE,
            "data_root": str(arguments.data_root),
            "data_namespace": str(recorded_root / "data"),
        },
        "expected_cells": expected_cells,
    }
    results_path.mkdir(parents=True)
    raw_directory = results_path / "raw"
    raw_directory.mkdir()
    (results_path / "matrix-config.json").write_text(
        json.dumps(config), encoding="utf-8"
    )
    (results_path / "environment.json").write_text(
        json.dumps({"git_commit": SOURCE_COMMIT}), encoding="utf-8"
    )
    provenance_state = {
        "commit": SOURCE_COMMIT,
        "branch": SOURCE_BRANCH,
        "tracked_changes": [],
        "unrelated_untracked": [],
    }
    (results_path / "source-provenance.json").write_text(
        json.dumps(
            {
                "source_before": provenance_state,
                "source_after": provenance_state,
            }
        ),
        encoding="utf-8",
    )
    binary_paths = {
        "phase0-bench": "/tmp/phase0-bench",
        "phase0-read-metrics-off": "/tmp/phase0-read-metrics-off",
    }
    binary_provenance = {}
    for cell in expected_cells:
        binary_provenance[cell["binary_key"]] = {
            "path": binary_paths[cell["binary_key"]],
            "sha256": "b" * 64,
            "environment_variable": cell["binary_env"],
        }
    (results_path / "binary-provenance.json").write_text(
        json.dumps(binary_provenance), encoding="utf-8"
    )
    events = []
    for cell in expected_cells:
        raw_path = raw_directory / f"{cell['cell_id']}.jsonl"
        raw_path.write_text(json.dumps(make_raw_record(cell)) + "\n", encoding="utf-8")
        log_path = raw_directory / f"{cell['cell_id']}.log"
        log_path.write_text("synthetic read benchmark\n", encoding="utf-8")
        event_identity = read_metrics_runner.event_identity(
            cell, SOURCE_COMMIT, SOURCE_BRANCH
        )
        binary_sha = "b" * 64
        start_event = {
            "event": "start",
            **event_identity,
            "binary_key": cell["binary_key"],
            "binary_env": cell["binary_env"],
            "binary_path": binary_paths[cell["binary_key"]],
            "binary_sha256": binary_sha,
            "output": str(raw_path.resolve()),
            "log": str(log_path.resolve()),
            "read_metrics_mode": cell["read_metrics_mode"],
            "sync_mode": "real",
            "sync_contract": "durable-return",
        }
        complete_event = {
            "event": "complete",
            **event_identity,
            "binary_key": cell["binary_key"],
            "binary_env": cell["binary_env"],
            "binary_path": binary_paths[cell["binary_key"]],
            "binary_sha256": binary_sha,
            "exit_code": 0,
            "output": str(raw_path.resolve()),
            "log": str(log_path.resolve()),
            "output_sha256": sha256_file(raw_path),
            "log_sha256": sha256_file(log_path),
        }
        events.extend((start_event, complete_event))
    (results_path / "run-order.jsonl").write_text(
        "".join(json.dumps(event) + "\n" for event in events), encoding="utf-8"
    )
    return results_path


class ReadMetricsMatrixTests(unittest.TestCase):
    def test_borrowed_page_variant_rejects_wrong_build_and_disabled_metrics(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            arguments = make_arguments(
                pathlib.Path(temporary_directory) / "results",
                "--variants", "blink-borrowed-pages,blink-metrics-on",
                "--read-kinds", "get",
                "--repetitions", "1",
            )
            cells = read_metrics_runner.build_expected_cells(arguments, SOURCE_COMMIT)
        candidate, baseline = cells
        record = make_raw_record(candidate)
        record["blink_borrowed_page_views_enabled"] = True
        read_metrics_runner.validate_raw_record(record, candidate, SOURCE_COMMIT)
        for field_name, value in (
            ("blink_borrowed_page_views_enabled", False),
            ("blink_read_observational_metrics_enabled", False),
            ("read_operations_metric", 0),
        ):
            mismatched_record = dict(record)
            mismatched_record[field_name] = value
            with self.subTest(field=field_name):
                with self.assertRaises(ValidationError):
                    read_metrics_runner.validate_raw_record(
                        mismatched_record, candidate, SOURCE_COMMIT
                    )
        record.pop("blink_borrowed_page_views_enabled")
        with self.assertRaises(ValidationError):
            read_metrics_runner.validate_raw_record(record, candidate, SOURCE_COMMIT)
        baseline_record = make_raw_record(baseline)
        baseline_record["blink_borrowed_page_views_enabled"] = True
        with self.assertRaises(ValidationError):
            read_metrics_runner.validate_raw_record(baseline_record, baseline, SOURCE_COMMIT)

    def test_default_matrix_has_24_cells_with_paired_modes_and_seeds(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            arguments = make_arguments(pathlib.Path(temporary_directory) / "results")
            cells = read_metrics_runner.build_expected_cells(arguments, SOURCE_COMMIT)

        self.assertEqual(len(cells), 24)
        self.assertEqual(
            {cell["read_metrics_mode"] for cell in cells},
            {"on", "off", "not-applicable"},
        )
        grouped_cells = {}
        for cell in cells:
            key = (cell["read_kind"], cell["repetition"])
            grouped_cells.setdefault(key, []).append(cell)
        self.assertEqual(len(grouped_cells), 6)
        for paired_cells in grouped_cells.values():
            self.assertEqual(len(paired_cells), 4)
            self.assertEqual(len({cell["seed"] for cell in paired_cells}), 1)
            self.assertEqual(
                {cell["variant"] for cell in paired_cells},
                set(read_metrics_runner.DEFAULT_VARIANTS),
            )
        self.assertEqual(
            len({cell["seed"] for cell in cells}),
            6,
        )

    def test_phase0_command_pins_two_tokio_workers(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            arguments = make_arguments(pathlib.Path(temporary_directory) / "results")
            cell = next(
                cell
                for cell in read_metrics_runner.build_expected_cells(
                    arguments, SOURCE_COMMIT
                )
                if cell["variant"] == "blink-metrics-off"
            )
            command, environment = read_metrics_runner.build_command(
                arguments,
                cell,
                pathlib.Path("/tmp/phase0-bench"),
                pathlib.Path("/tmp/read-metrics.jsonl"),
                pathlib.Path("/tmp/read-metrics-data"),
            )

        self.assertEqual(command[command.index("--tokio-workers") + 1], "2")
        self.assertEqual(command[command.index("--engine") + 1], "parallel-blink")
        self.assertEqual(command[command.index("--suite") + 1], "read")
        self.assertEqual(command[command.index("--readers") + 1], "16")
        self.assertEqual(environment["DODB_BENCH_DIR"], "/tmp/read-metrics-data")

    def test_phase0_read_contract_rejects_invalid_suite_and_workload(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            arguments = make_arguments(
                pathlib.Path(temporary_directory) / "results",
                "--readers",
                "2",
                "--read-kinds",
                "query",
                "--variants",
                "blink-metrics-on",
                "--repetitions",
                "1",
            )
            cell = read_metrics_runner.build_expected_cells(arguments, SOURCE_COMMIT)[0]

        record = make_raw_record(cell)
        self.assertIs(
            read_metrics_runner.validate_raw_record(record, cell, SOURCE_COMMIT), record
        )
        for field_name, bad_value in (
            ("suite", "read"),
            ("workload", "100%-read-get"),
            ("attempted_read_percent", 99.0),
            ("attempted_write_transactions", 1),
        ):
            mismatched_record = dict(record)
            mismatched_record[field_name] = bad_value
            with self.subTest(field=field_name):
                with self.assertRaises(ValidationError):
                    read_metrics_runner.validate_raw_record(
                        mismatched_record, cell, SOURCE_COMMIT
                    )

    def test_raw_metric_modes_accept_matching_records_and_reject_mismatch(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            arguments = make_arguments(
                pathlib.Path(temporary_directory) / "results",
                "--readers",
                "2",
                "--read-kinds",
                "get",
                "--repetitions",
                "1",
            )
            cells = read_metrics_runner.build_expected_cells(arguments, SOURCE_COMMIT)
        for variant in ("blink-metrics-on", "blink-metrics-off"):
            cell = next(cell for cell in cells if cell["variant"] == variant)
            record = make_raw_record(cell)
            self.assertIs(
                read_metrics_runner.validate_raw_record(record, cell, SOURCE_COMMIT),
                record,
            )
            mismatched_record = dict(record)
            mismatched_record["blink_read_observational_metrics_enabled"] = not cell[
                "expected_read_metrics_enabled"
            ]
            with self.subTest(variant=variant, field="mode"):
                with self.assertRaises(ValidationError):
                    read_metrics_runner.validate_raw_record(
                        mismatched_record, cell, SOURCE_COMMIT
                    )
            mismatched_record = dict(record)
            mismatched_record["read_operations_metric"] = (
                0 if cell["expected_read_metrics_enabled"] else 100
            )
            with self.subTest(variant=variant, field="counter"):
                with self.assertRaises(ValidationError):
                    read_metrics_runner.validate_raw_record(
                        mismatched_record, cell, SOURCE_COMMIT
                    )

    def test_raw_record_requires_exact_source_sha_and_phase0_repetition_zero(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            arguments = make_arguments(
                pathlib.Path(temporary_directory) / "results",
                "--readers",
                "2",
                "--read-kinds",
                "get",
                "--variants",
                "blink-metrics-on",
                "--repetitions",
                "1",
            )
            cell = read_metrics_runner.build_expected_cells(arguments, SOURCE_COMMIT)[0]
        record = make_raw_record(cell)
        self.assertEqual(cell["expected_raw_repetition"], 0)
        self.assertIs(
            read_metrics_runner.validate_raw_record(record, cell, SOURCE_COMMIT), record
        )
        with self.assertRaises(ValidationError):
            read_metrics_runner.validate_raw_record(
                make_raw_record(cell, checkout_sha="b" * 40), cell, SOURCE_COMMIT
            )
        with self.assertRaises(ValidationError):
            read_metrics_runner.validate_raw_record(
                make_raw_record(cell, repetition=1), cell, SOURCE_COMMIT
            )

    def test_missing_or_failed_read_acceptance_gates_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            arguments = make_arguments(
                pathlib.Path(temporary_directory) / "results",
                "--readers",
                "2",
                "--read-kinds",
                "get",
                "--variants",
                "blink-metrics-on",
                "--repetitions",
                "1",
            )
            cell = read_metrics_runner.build_expected_cells(arguments, SOURCE_COMMIT)[0]
        for field_name, bad_value in (
            ("errors", None),
            ("errors", 1),
            ("conflicts", 1),
            ("overloads", None),
            ("overloads", 1),
            ("successful_reads", 0),
        ):
            record = make_raw_record(cell)
            if bad_value is None:
                record.pop(field_name)
            else:
                record[field_name] = bad_value
            with self.subTest(field=field_name, value=bad_value):
                with self.assertRaises(ValidationError):
                    read_metrics_runner.validate_raw_record(
                        record, cell, SOURCE_COMMIT
                    )
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_path = pathlib.Path(temporary_directory)
            empty_results = write_matrix_fixture(
                temporary_path / "empty", temporary_path
            )
            matrix_config_path = empty_results / "matrix-config.json"
            matrix_config = json.loads(matrix_config_path.read_text(encoding="utf-8"))
            matrix_config["planned_count"] = 0
            matrix_config["expected_cells"] = []
            matrix_config_path.write_text(
                json.dumps(matrix_config), encoding="utf-8"
            )
            with self.assertRaises(ValidationError):
                read_metrics_runner.validate_matrix(empty_results)

    def test_copied_matrix_resolves_recorded_raw_paths(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_path = pathlib.Path(temporary_directory)
            original_results = write_matrix_fixture(
                temporary_path / "original", temporary_path
            )
            copied_results = temporary_path / "copied"
            shutil.copytree(original_results, copied_results)

            validated = read_metrics_runner.validate_matrix(
                copied_results, recorded_results_root=original_results
            )

        self.assertEqual(len(validated["validated_runs"]), 2)
        summary = read_metrics_runner.build_summary(validated)
        self.assertEqual(summary["completed_count"], 2)
        self.assertTrue(summary["accepted"])


if __name__ == "__main__":
    unittest.main()
