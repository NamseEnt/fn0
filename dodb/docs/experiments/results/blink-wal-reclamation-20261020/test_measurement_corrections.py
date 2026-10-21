import tempfile
import unittest
from pathlib import Path

import run_short_matrix
import verify_matrix


class MeasurementCorrectionsTests(unittest.TestCase):
    def test_wal_size_includes_numeric_logs_and_excludes_diagnostic_logs(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            directory = Path(temporary_directory)
            files = {
                "tenant-1/000039.log": 37,
                "blink/tenant-1.wal": 19,
                "tenant-1/LOG": 101,
                "tenant-1/LOG.old.123": 103,
                "workload.stdout.log": 107,
                "wal-statistics.txt": 109,
                "１２３.log": 113,
            }
            for filename, size_bytes in files.items():
                path = directory / filename
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(bytes(size_bytes))
            self.assertEqual(run_short_matrix.wal_bytes(directory), 56)
            manifest = [{"path": name, "size_bytes": size} for name, size in files.items()]
            self.assertEqual(verify_matrix.retained_wal_bytes(manifest), 56)

    def test_nonzero_and_submillisecond_checkpoint_drain_reduce_throughput(self):
        timing = {
            "actual_admission_drain_and_last_completion_ms": 30_000,
            "actual_write_batches": 31_250,
            "final_checkpoint_drain_ms": 1_250,
        }
        self.assertEqual(verify_matrix.corrected_write_timing(timing), (31_250, 1_000))
        timing["final_checkpoint_drain_ms"] = 0.5
        elapsed_ms, throughput = verify_matrix.corrected_write_timing(timing)
        self.assertEqual(elapsed_ms, 30_000.5)
        self.assertLess(throughput, 31_250 / 30)

    def test_historical_sync_derivation_requires_complete_successful_events(self):
        metric = {
            "checkpoints_completed": 3,
            "checkpoint_wal_syncs": 3,
            "checkpoint_events": [{"succeeded": True}] * 3,
        }
        self.assertEqual(verify_matrix.corrected_checkpoint_syncs(metric), 6)
        for invalid_field, invalid_value in [
            ("checkpoints_failed", 1),
            ("omitted_checkpoint_events", 1),
            ("checkpoints_completed", 4),
            ("checkpoint_wal_syncs", 6),
        ]:
            with self.subTest(field=invalid_field):
                with self.assertRaises(ValueError):
                    verify_matrix.corrected_checkpoint_syncs({**metric, invalid_field: invalid_value})

    def test_short_drain_checkpoints_cannot_qualify_as_measurement_checkpoints(self):
        report = {"timing": {
            "requested_measurement_ms": 30_000,
            "actual_admission_drain_and_last_completion_ms": 30_010,
            "final_checkpoint_drain_ms": 0,
        }}
        short_events = [{"succeeded": True, "duration_nanos": 1_000_000}] * 4
        self.assertEqual(verify_matrix.measurement_checkpoint_count(report, "E", short_events)[0], 0)
        long_events = [{"succeeded": True, "duration_nanos": 100_000_000}] * 4
        self.assertEqual(verify_matrix.measurement_checkpoint_count(report, "E", long_events)[0], 3)

    def test_unavailable_peak_wal_stays_distinct_from_retained_manifest_bytes(self):
        for workload in ("A", "E"):
            case = verify_matrix.summarize_case(f"{workload}-minimal-native-r1", workload, "minimal-native", 1)
            self.assertFalse(case["errors"])
            self.assertIsNone(case["maximum_wal_bytes"])
            self.assertGreater(case["retained_wal_bytes_after_reopen"], 0)
            self.assertIsNone(case["checkpoint_wal_syncs"])


if __name__ == "__main__":
    unittest.main()
