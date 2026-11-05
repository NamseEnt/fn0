import importlib.util
from pathlib import Path
import unittest


SCRIPT = Path(__file__).with_name("oci_t17_matrix.py")
SPEC = importlib.util.spec_from_file_location("oci_t17_matrix", SCRIPT)
MATRIX = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MATRIX)


def window(index, passed=True):
    return {
        "window_index": index,
        "duration_ms": 300000,
        "score": {
            "passed": passed,
            "read_p99_upper_bound_ms": 25.0,
            "write_p99_upper_bound_ms": 75.0,
            "rejection_rate": 0.0001,
        },
    }


class SloWindowTests(unittest.TestCase):
    def test_measurement_extension_and_recovery_windows_are_scored(self):
        report = {
            "workload": {
                "slo_windows": [window(0)],
                "phase_workloads": {
                    "checkpoint_cycle_extensions": [
                        {"phase": "measurement-extension-30", "workload": {"slo_windows": [window(0)]}}
                    ],
                    "recovery_after_reopen": {"slo_windows": [window(0)]},
                },
            }
        }

        windows = MATRIX.slo_windows(report)

        self.assertEqual([item["phase"] for item in windows], [
            "measurement",
            "measurement-extension-30",
            "recovery",
        ])
        self.assertTrue(all(item["passed"] for item in windows))
        self.assertEqual(MATRIX.decision_metrics(report)["recovery_windows"], 1)

    def test_report_requires_six_measurement_and_recovery_windows(self):
        report = {
            "status": "completed",
            "post_measurement_full_population_oracle": {"passed": True},
            "reopened_full_population_oracle": {"passed": True},
            "host_cache_preconditioning": {"status": "page_cache_dropped_before_fixed_warmup"},
            "host_cache_after_warmup": {"arcstats": "size 4 2147483648"},
            "backend_memory_pressure": {"enabled": True, "samples_truncated": False, "epochs": {}},
            "workload": {
                "cell_valid": True,
                "valid_for_oracle": True,
                "service_memory_admission_bytes": 6 * 1024**3,
                "service_memory_admission_in_flight_limit": 96,
                "slo_stop_reason": "duration_completed",
                "checkpoint_cycle_gate_passed": True,
                "successful_checkpoint_cycles": 20,
                "measured_duration_ms_including_extensions": 1800000,
                "slo_windows": [window(index) for index in range(6)],
                "phase_workloads": {
                    "recovery_after_reopen": {"slo_stop_reason": "duration_completed", "slo_windows": [window(index) for index in range(6)]}
                },
            },
        }

        windows = MATRIX.validate_report(
            report,
            "blink-paged",
            {"long_reader_retention_ms": 0},
        )

        metrics = MATRIX.decision_metrics(report)
        self.assertEqual(len(windows), 12)
        self.assertEqual(metrics["recovery_windows"], 6)
        self.assertEqual(metrics["recovery_max_p99_read_ms"], 25.0)

    def test_report_rejects_incomplete_recovery_windows(self):
        report = {
            "status": "completed",
            "post_measurement_full_population_oracle": {"passed": True},
            "reopened_full_population_oracle": {"passed": True},
            "host_cache_preconditioning": {"status": "page_cache_dropped_before_fixed_warmup"},
            "host_cache_after_warmup": {"arcstats": "size 4 2147483648"},
            "backend_memory_pressure": {"enabled": True, "samples_truncated": False, "epochs": {}},
            "workload": {
                "cell_valid": True,
                "valid_for_oracle": True,
                "service_memory_admission_bytes": 6 * 1024**3,
                "service_memory_admission_in_flight_limit": 96,
                "slo_stop_reason": "duration_completed",
                "checkpoint_cycle_gate_passed": True,
                "slo_windows": [window(index) for index in range(6)],
                "phase_workloads": {
                    "recovery_after_reopen": {"slo_windows": [window(0)]}
                },
            },
        }

        with self.assertRaisesRegex(RuntimeError, "30-minute recovery"):
            MATRIX.validate_report(report, "blink-resident", {"long_reader_retention_ms": 0})

    def test_failed_nested_window_is_rejected(self):
        report = {"workload": {"slo_windows": [window(0, passed=False)]}}

        with self.assertRaisesRegex(RuntimeError, "failed SLO window"):
            MATRIX.slo_windows(report)


if __name__ == "__main__":
    unittest.main()
