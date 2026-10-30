import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock


HERE = Path(__file__).resolve().parent


def load_module(name, file_path):
    specification = importlib.util.spec_from_file_location(name, file_path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


REVIEW = load_module('arrival_review_under_test', HERE / 'arrival_review.py')
CONTROLS = load_module('arrival_controls_under_test', HERE / 'oci_arrival_controls.py')


def fake_report(case, mode, backend='blink'):
    expected = REVIEW.expected_cell(case)
    active = expected['active']
    registered = expected['registered']
    duration = expected['duration_ms']
    expected_per_tenant = duration * REVIEW.RATE // 1000
    rows = []
    for tenant_number in range(1, active + 1):
        active_tenant = tenant_number <= active
        rows.append({
            'tenant': tenant_number,
            'arrival_phase_offset_nanos': REVIEW.phase_offset_ns(tenant_number, REVIEW.RATE, active) if mode == 'staggered' else 0,
            'scheduled': expected_per_tenant if active_tenant else 0,
            'admitted': expected_per_tenant if active_tenant else 0,
            'completed': expected_per_tenant if active_tenant else 0,
            'successful': expected_per_tenant if active_tenant else 0,
            'dropped': 0,
            'request_count': expected_per_tenant if active_tenant else 0,
            'request_fingerprint': f'request-{tenant_number}',
            'operation_counts': [200, 100, 200] if active_tenant else [0, 0, 0],
            'transaction_counts': [200, 440, 44, 100, 100] if active_tenant else [0, 0, 0, 0, 0],
            'width_counts': [160, 30, 10] if active_tenant else [0, 0, 0],
        })
    mode_fingerprint = 'sync-schedule' if mode == 'synchronized' else 'staggered-schedule'
    total = expected_per_tenant * active
    return {
        'status': 'completed',
        'compiled_source_revision': REVIEW.SOURCE_COMMIT,
        'requested_source_revision': REVIEW.SOURCE_COMMIT,
        'effective_configuration': {
            'backend': backend,
            'registered_tenants': registered,
            'active_tenants': active,
            'opened_idle_tenants': expected['idle'],
            'initial_rows_per_tenant': expected['rows'],
            'duration_ms': duration,
            'tenant_arrival_phase_selected': mode,
            'tenant_arrival_phase_applies': True,
            'tenant_arrival_phase': mode,
            'client_count': 1,
            'seed': 20261027,
            'per_tenant_rate': REVIEW.RATE,
            'queue_bound': REVIEW.QUEUE_BOUND,
            'max_in_flight': REVIEW.MAX_IN_FLIGHT,
            'request_timeout_ms': REVIEW.REQUEST_TIMEOUT_MS,
            'drain_timeout_ms': REVIEW.DRAIN_TIMEOUT_MS,
            'backend_memory_budget': {
                'max_bytes': REVIEW.MEMORY_MAX_BYTES,
                'high_bytes': REVIEW.MEMORY_HIGH_BYTES,
                'sample_interval_ms': REVIEW.PRESSURE_INTERVAL_MS,
            },
        },
        'backend_memory_pressure': {'samples_truncated': False},
        'workload': {
            'cell_valid': True,
            'valid_for_oracle': True,
            'accounting': {'scheduled': True, 'completed': True},
            'scheduled': total,
            'admitted': total,
            'completed': total,
            'successful': total,
            'dropped': 0,
            'rejected_conflicts': 0,
            'other_error': 0,
            'timed_out': 0,
            'queued_at_end': 0,
            'in_flight_at_end': 0,
            'deterministic_fingerprints': {
                'input_fingerprint': 'same-input',
                'schedule_fingerprint': mode_fingerprint,
            },
            'request_count': total,
            'request_fingerprint': 'same-request',
            'scheduled_fingerprint': mode_fingerprint,
            'per_tenant': rows,
        },
        'post_measurement_full_population_oracle': {
            'passed': True, 'verified_seed_rows': registered * expected['rows'],
        },
        'reopened_full_population_oracle': {
            'passed': True, 'verified_seed_rows': registered * expected['rows'],
        },
    }


class ArrivalReviewTests(unittest.TestCase):
    def test_four_tenant_offsets_match_frozen_five_millisecond_spacing(self):
        self.assertEqual([REVIEW.phase_offset_ns(tenant_number, 50, 4)
                          for tenant_number in range(1, 5)],
                         [0, 5_000_000, 10_000_000, 15_000_000])

    def test_n28_staggered_offsets_use_integer_floor_formula(self):
        offsets = [REVIEW.phase_offset_ns(tenant_number, 50, 28)
                   for tenant_number in range(1, 29)]
        self.assertEqual(offsets[0], 0)
        self.assertEqual(offsets[-1], 19_285_714)
        self.assertEqual(offsets, sorted(offsets))

    def test_tiny_reports_validate_and_compare_across_modes(self):
        synchronized = fake_report('tiny-synchronized', 'synchronized')
        staggered = fake_report('tiny-staggered', 'staggered')
        REVIEW.validate_report(synchronized, 'tiny-synchronized', 'blink')
        REVIEW.validate_report(staggered, 'tiny-staggered', 'blink')
        comparison = REVIEW.compare_phases(synchronized, staggered)
        self.assertTrue(comparison['recipe_matches'])
        self.assertTrue(comparison['schedule_identity_differs'])
        self.assertEqual(comparison['expected_staggered_offsets_ns'], [0, 6_666_666, 13_333_333])
        reversed_comparison = REVIEW.compare_phases(staggered, synchronized)
        self.assertTrue(reversed_comparison['schedule_identity_differs'])

    def test_phase_review_rejects_offset_mismatch(self):
        report = fake_report('tiny-staggered', 'staggered')
        report['workload']['per_tenant'][1]['arrival_phase_offset_nanos'] += 1
        with self.assertRaises(AssertionError):
            REVIEW.validate_report(report, 'tiny-staggered', 'blink')

    def test_phase_review_records_request_differences_after_drops(self):
        synchronized = fake_report('tiny-synchronized', 'synchronized')
        staggered = fake_report('tiny-staggered', 'staggered')
        for report in [synchronized, staggered]:
            workload = report['workload']
            workload['dropped'] = 2
            workload['admitted'] -= 2
            workload['completed'] -= 2
            workload['successful'] -= 2
            workload['per_tenant'][0]['admitted'] -= 2
            workload['per_tenant'][0]['completed'] -= 2
            workload['per_tenant'][0]['successful'] -= 2
        staggered['workload']['request_fingerprint'] = 'different-after-drops'
        REVIEW.validate_report(synchronized, 'tiny-synchronized', 'blink')
        REVIEW.validate_report(staggered, 'tiny-staggered', 'blink')
        comparison = REVIEW.compare_phases(synchronized, staggered)
        self.assertTrue(comparison['either_mode_dropped_requests'])
        self.assertFalse(comparison['request_matches']['request_fingerprint'])

    def test_fixed_cell_order_and_backend_pair_parity(self):
        specs = list(CONTROLS.cell_sequence())
        self.assertEqual(len(specs), 8)
        self.assertEqual(specs[0]['backends'], ['blink', 'minimal-native-shared'])
        self.assertEqual(specs[1]['backends'], ['minimal-native-shared', 'blink'])
        self.assertEqual([spec['case'] for spec in specs[2:]], [
            'control-n28-r1-synchronized', 'control-n28-r1-staggered',
            'control-n28-r2-staggered', 'control-n28-r2-synchronized',
            'control-n28-r3-synchronized', 'control-n28-r3-staggered',
        ])
        self.assertEqual(2 * sum(spec['duration_ms'] * 50 * spec['active'] // 1000 for spec in specs), 1_014_000)

    def test_controller_soft_limit_preserves_hard_limit_and_owned_limit_contract(self):
        with mock.patch.object(CONTROLS.resource, 'getrlimit', side_effect=[(1024, 524288), (65536, 524288)]), \
                mock.patch.object(CONTROLS.resource, 'setrlimit') as set_limit, \
                mock.patch.object(CONTROLS, 'parse_proc_limits', return_value={
                    'soft': 65536, 'hard': 524288, 'units': 'files'}):
            identity = CONTROLS.set_controller_nofile_limit()
        set_limit.assert_called_once_with(CONTROLS.resource.RLIMIT_NOFILE, (65536, 524288))
        self.assertEqual(identity['original_soft'], 1024)
        self.assertEqual(identity['new_hard'], 524288)
        self.assertTrue(CONTROLS.validate_inherited_nofile_limits(
            {'soft': 65536, 'hard': 524288, 'units': 'files'}, 524288))
        with self.assertRaises(AssertionError):
            CONTROLS.validate_inherited_nofile_limits(
                {'soft': 1024, 'hard': 524288, 'units': 'files'}, 524288)

    def test_preflight_child_inherits_limits_and_is_reaped_before_cells(self):
        class FakeChild:
            pid = 123

            def wait(self, timeout):
                return 0

            def poll(self):
                return 0

        child_identity = {'pid': 123, 'parent_pid': CONTROLS.os.getpid(), 'start_time_ticks': 456}
        with mock.patch.object(CONTROLS.subprocess, 'Popen', return_value=FakeChild()) as launch_child, \
                mock.patch.object(CONTROLS, 'proc_identity', side_effect=[child_identity, FileNotFoundError()]), \
                mock.patch.object(CONTROLS, 'parse_proc_limits', return_value={
                    'soft': 65536, 'hard': 524288, 'units': 'files'}):
            result = CONTROLS.verify_preflight_child_limits(524288)
        launch_child.assert_called_once_with(
            [CONTROLS.sys.executable, '-c', 'import time; time.sleep(0.5)'],
            stdin=CONTROLS.subprocess.DEVNULL, stdout=CONTROLS.subprocess.DEVNULL,
            stderr=CONTROLS.subprocess.DEVNULL, start_new_session=True)
        self.assertEqual(result['role'], 'preflight-limit-child')
        self.assertEqual(result['start_time_ticks'], 456)
        self.assertEqual(result['limits']['soft'], 65536)
        self.assertFalse(result['same_live_process_after_exit'])

    def test_proc_tree_tracks_owned_children_and_open_file_limits(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            proc_root = Path(temporary_directory)
            for pid, parent_pid, start_time in [(10, 1, 100), (11, 10, 110), (12, 11, 120)]:
                process_directory = proc_root / str(pid)
                process_directory.mkdir()
                stat_fields = ['S', str(parent_pid), *(['0'] * 18)]
                stat_fields[19] = str(start_time)
                (process_directory / 'stat').write_text(f'{pid} (task) ' + ' '.join(stat_fields))
                (process_directory / 'limits').write_text(
                    'Limit Soft Limit Hard Limit Units\nMax open files 65536 524288 files\n')
            with mock.patch.object(CONTROLS, 'Path', side_effect=lambda value: proc_root if str(value) == '/proc' else Path(value)):
                tree = CONTROLS.process_tree(10)
                limits = CONTROLS.parse_proc_limits(11)
            self.assertEqual([identity['pid'] for identity in tree], [10, 11, 12])
            self.assertEqual(limits, {'soft': 65536, 'hard': 524288, 'units': 'files'})

    def test_fake_tiny_cells_review_without_mutating_reports(self):
        class FakeCapacityReviewer:
            def reduce_report(self, report, source_commit):
                self.assert_source(source_commit)
                return {'operational_pass': True, 'slo_pass': True}

            def compare_pair(self, left, right):
                return {'realized_comparison_available': True,
                        'same_mode': left['effective_configuration']['tenant_arrival_phase'] == right['effective_configuration']['tenant_arrival_phase']}

            def assert_source(self, source_commit):
                if source_commit != REVIEW.SOURCE_COMMIT:
                    raise AssertionError(source_commit)

        with tempfile.TemporaryDirectory() as temporary_directory:
            run_directory = Path(temporary_directory)
            (run_directory / 'provenance.json').write_text(json.dumps({'source_commit': REVIEW.SOURCE_COMMIT}))
            pairs = {}
            original_hashes = {}
            for mode in ['synchronized', 'staggered']:
                cell_names = []
                for backend, suffix in [('blink', '-blink'), ('minimal-native-shared', '-shared')]:
                    case = 'tiny-' + mode + suffix
                    report = fake_report('tiny-' + mode, mode, backend)
                    report_path = run_directory / (case + '.json')
                    report_path.write_text(json.dumps(report, sort_keys=True))
                    original_hashes[case] = CONTROLS.digest(report_path)
                    (run_directory / (case + '-status.json')).write_text(json.dumps({
                        'case': case, 'backend': backend,
                        'report_sha256': original_hashes[case],
                    }))
                    cell_names.append(case)
                pairs['tiny-' + mode] = cell_names
                (run_directory / ('tiny-' + mode + '-pair.json')).write_text(json.dumps({'cases': cell_names}))
            provenance_path = run_directory / 'provenance.json'
            provenance = json.loads(provenance_path.read_text())
            provenance['source_commit'] = REVIEW.SOURCE_COMMIT
            provenance_path.write_text(json.dumps(provenance))
            reviewed = REVIEW.review_directory(run_directory, FakeCapacityReviewer())
            self.assertEqual(len(reviewed['cases']), 4)
            self.assertEqual(len(reviewed['same_mode_pairs']), 2)
            self.assertEqual(len(reviewed['phase_pairs']), 2)
            for case, original_hash in original_hashes.items():
                self.assertEqual(CONTROLS.digest(run_directory / (case + '.json')), original_hash)


if __name__ == '__main__':
    unittest.main()
