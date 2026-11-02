import copy
import importlib.util
import json
from pathlib import Path
import unittest


EVIDENCE = Path(__file__).resolve().parent.parent
SPECIFICATION = importlib.util.spec_from_file_location('maintenance_under_test', Path(__file__).with_name('oci_maintenance.py'))
CONTROLLER = importlib.util.module_from_spec(SPECIFICATION)
SPECIFICATION.loader.exec_module(CONTROLLER)


class MaintenanceChecks(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.frontier, cls.reviewer, cls.capacity_review, cls.paths = CONTROLLER.dependencies()
        cls.report_path = next((EVIDENCE / 'resident-frontier' / 'runtime').rglob('confirm-n24-r1-blink.json'))
        cls.report = json.loads(cls.report_path.read_text())
        cls.command = json.loads(cls.report_path.with_name('confirm-n24-r1-blink-command.json').read_text())
        CONTROLLER.configure_reviewer(cls.reviewer)

    def test_fixed_matrix_and_backend_order(self):
        cells = CONTROLLER.sequence()
        self.assertEqual(len(cells), 6)
        self.assertEqual([cell['backend'] for cell in cells],
                         ['blink', 'minimal-native-shared', 'minimal-native-shared',
                          'blink', 'blink', 'minimal-native-shared'])
        self.assertEqual(len({cell['case'] for cell in cells}), 6)
        self.assertTrue(all(CONTROLLER.expected_cell(cell['case'])['duration_ms'] == 300000 for cell in cells))

    def test_reject_unplanned_population_repetition_and_phase(self):
        for case in ['maintenance-n28-r1-blink', 'maintenance-n24-r4-blink',
                     'confirm-n24-r1-blink', 'maintenance-n24-r1-other']:
            with self.subTest(case=case), self.assertRaises(AssertionError):
                CONTROLLER.expected_cell(case)

    def maintenance_command(self):
        command = copy.deepcopy(self.command)
        case = 'maintenance-n24-r1-blink'
        replacements = {
            '--duration-ms': '300000', '--report': str(CONTROLLER.RUN_DIRECTORY / (case + '.json')),
            '--data-dir': str(CONTROLLER.DATABASE_ROOT / case),
            '--backend-cgroup-root': str(CONTROLLER.CGROUP_ROOT / case)}
        for option, value in replacements.items():
            command['command'][command['command'].index(option) + 1] = value
        return command

    def test_exact_command_paths_and_duration(self):
        self.assertEqual(self.reviewer.parse_command(self.maintenance_command(), 'maintenance-n24-r1-blink')['duration_ms'], 300000)

    def test_reject_command_report_path_omission_or_wrong_recipe(self):
        for option in ['--report', '--duration-ms', '--data-dir', '--backend-memory-high-bytes', '--tenant-arrival-phase']:
            command = self.maintenance_command()
            command['command'][command['command'].index(option) + 1] = 'incorrect'
            with self.subTest(option=option), self.assertRaises(AssertionError):
                self.reviewer.parse_command(command, 'maintenance-n24-r1-blink')
        command = self.maintenance_command()
        report_option = command['command'].index('--report')
        del command['command'][report_option:report_option + 2]
        with self.assertRaises(AssertionError):
            self.reviewer.parse_command(command, 'maintenance-n24-r1-blink')

    def test_metrics_use_real_frozen_report_and_measurement_epoch(self):
        metrics = CONTROLLER.maintenance_metrics(self.report, self.capacity_review)
        self.assertGreater(metrics['periodic_checkpoint_count_increase'], 2)
        self.assertTrue(metrics['blink_multiple_cycles_observed'])
        self.assertEqual(metrics['offer_end_pending_requests'],
                         self.report['workload']['scheduled'] - self.report['workload']['completed_at_offer_end'])
        self.assertLess(metrics['native_serialized_bytes'], 8 * 1024**2)
        self.assertEqual(metrics['reference_cycle_count']['status'], 'unavailable')

    def test_short_report_cannot_be_accepted_as_long_maintenance(self):
        with self.assertRaisesRegex(AssertionError, 'duration_ms'):
            self.reviewer.validate_report(self.report, 'maintenance-n24-r1-blink', 'blink')

    def test_shared_report_does_not_supply_blink_cycle_counters(self):
        report_path = next((EVIDENCE / 'resident-frontier' / 'runtime').rglob('screen-n32-shared.json'))
        report = json.loads(report_path.read_text())
        metrics = CONTROLLER.maintenance_metrics(report, self.capacity_review)
        self.assertIsNone(metrics['periodic_checkpoint_count_increase'])
        self.assertIsNone(metrics['blink_multiple_cycles_observed'])

    def test_pair_fingerprint_mismatch_is_rejected(self):
        altered = copy.deepcopy(self.report)
        altered['workload']['request_fingerprint'] = 'incorrect'
        with self.assertRaises(AssertionError):
            self.capacity_review.compare_pair(self.report, altered)


if __name__ == '__main__':
    unittest.main()
