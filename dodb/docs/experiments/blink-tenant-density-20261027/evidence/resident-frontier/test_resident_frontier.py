import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import sys
import tempfile
import types
import unittest
from unittest.mock import patch


EVIDENCE_DIRECTORY = Path(__file__).resolve().parent
CONTROLLER_SPEC = importlib.util.spec_from_file_location(
    'oci_resident_frontier_under_test', EVIDENCE_DIRECTORY / 'oci_resident_frontier.py')
CONTROLLER = importlib.util.module_from_spec(CONTROLLER_SPEC)
sys.modules[CONTROLLER_SPEC.name] = CONTROLLER
CONTROLLER_SPEC.loader.exec_module(CONTROLLER)
REVIEW_SPEC = importlib.util.spec_from_file_location(
    'resident_frontier_review_under_test', EVIDENCE_DIRECTORY / 'resident_frontier_review.py')
REVIEW = importlib.util.module_from_spec(REVIEW_SPEC)
sys.modules[REVIEW_SPEC.name] = REVIEW
REVIEW_SPEC.loader.exec_module(REVIEW)
ARRIVAL_MODULE = CONTROLLER.load_module(
    'arrival_controls_adapter_contract',
    EVIDENCE_DIRECTORY.parent / 'arrival-phase' / 'runtime' / 'arrival-phase' / 'oci_arrival_controls.py')


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def controls_review_fixture():
    case_results = {}
    for backend, suffix in [('blink', 'blink'), ('minimal-native-shared', 'shared')]:
        for round_number in range(1, 4):
            slo_pass = not (backend == 'blink' and round_number == 2)
            case_results[f'control-n28-r{round_number}-staggered-{suffix}'] = {
                'valid': True, 'operational_pass': True, 'slo_pass': slo_pass,
            }
    return {
        'source_commit': CONTROLLER.SOURCE_COMMIT,
        'provenance': {
            'plan_commit': CONTROLLER.CONTROLS_PLAN_COMMIT,
            'actual_controller_commit': CONTROLLER.CONTROLS_CONTROLLER_COMMIT,
        },
        'collection_validity': {'completed_run_evidence_valid': True, 'accepted_run': False},
        'cells': {
            'completed': True,
            'actual_unique_count': 16,
            'operationally_valid_count': 16,
            'tiny_slo_gate': True,
            'case_results': case_results,
        },
        'validation': {
            'source_commit': CONTROLLER.SOURCE_COMMIT,
            'passing_tests': 85,
            'same_17_existing_clippy_diagnostics': True,
            'default_link_was_last': True,
            'native_archive_sha256': CONTROLLER.EXPECTED_NATIVE_SHA256,
            'validation_binaries_valid': True,
            'runtime_binaries_match_validation': True,
            'default_binaries': {
                'tenant_density_workload': {'bytes': 10, 'sha256': 'a' * 64},
                'memory_diagnostic_worker': {'bytes': 11, 'sha256': 'b' * 64},
            },
        },
        'requests': {'expected_complete_cohort': 1_014_000,
                     'validated_successful_requests': 1_014_000},
        'oracles': {'validated_oracle_count': 32, 'expected_complete_cohort': 32},
        'databases': {'preserved_database_count': 16},
        'processes': {'all_recorded_processes_exited': True, 'owned_workload_child_count': 16},
        'cgroups': {'cgroup_count': 81, 'all_cgroup_process_lists_empty': True},
        'runtime_limits': {'new_soft': 65536, 'hard': 524288, 'preflight_child_exited': True},
    }


class FakeRunner:
    def __init__(self, behavior=None):
        self.behavior = behavior or {}
        self.calls = []

    def run_cell(self, case, backend, active_tenants, duration_ms, stage, repetition=None):
        self.calls.append({'case': case, 'backend': backend, 'active_tenants': active_tenants,
                           'duration_ms': duration_ms, 'stage': stage, 'repetition': repetition})
        outcome = self.behavior.get(case, {'operational_pass': True, 'slo_pass': True})
        if isinstance(outcome, Exception):
            raise outcome
        return {}, {'reduced': outcome}


class FakeCapacityRuntime:
    BASE_ARGUMENTS = []
    BACKEND_ARGUMENTS = {'blink': []}

    def capacity(self):
        return {'zfs_free_bytes': 20 * 1024**3,
                'memory_bytes': {'MemAvailable': 2 * 1024**3}}

    def guard(self, state, before=False):
        return None

    def prepare_group(self, group_path):
        Path(group_path).mkdir(parents=True, exist_ok=True)

    def kill_group(self, group_path):
        return None

    def process_identity_live(self, identity):
        return False

    def reduce_report(self, report, source_commit):
        return {'operational_pass': True, 'slo_pass': True}

    def write_json(self, path, value):
        write_json(path, value)


class FakeWorkloadProcess:
    def __init__(self, arguments, **kwargs):
        self.pid = __import__('os').getpid()
        self.returncode = 0
        report_path = Path(arguments[arguments.index('--report') + 1])
        write_json(report_path, {'workload': {'cell_valid': True},
                                 'memory_diagnostic': {'processes': []}})

    def poll(self):
        return 0

    def wait(self, timeout=None):
        return 0


class ResidentFrontierTests(unittest.TestCase):
    def test_actual_collected_controls_review_passes_entry_gate_and_binary_copy(self):
        controls_path = (EVIDENCE_DIRECTORY.parent / 'arrival-phase' / 'final-review.json')
        controls = CONTROLLER.validate_controls_review(
            controls_path, CONTROLLER.sha256_file(controls_path))
        self.assertFalse(controls['accepted_staggered_n28_baseline_by_backend']['blink'])
        self.assertTrue(controls['accepted_staggered_n28_baseline_by_backend']['minimal-native-shared'])
        self.assertEqual(
            [record['case'] for record in CONTROLLER.baseline_review_cells(controls, 'blink')],
            [f'control-n28-r{round_number}-staggered-blink' for round_number in range(1, 4)],
        )

        runtime_root = EVIDENCE_DIRECTORY.parent / 'arrival-phase' / 'runtime'
        validation_summary_path = (runtime_root / 'arrival-phase' / CONTROLLER.VALIDATION_RUN
                                   / 'arrival-validation-summary.json')
        original_root = CONTROLLER.ROOT
        CONTROLLER.ROOT = runtime_root
        try:
            with tempfile.TemporaryDirectory() as temporary:
                output_directory = Path(temporary) / CONTROLLER.RUN_NAME
                output_directory.mkdir()
                copied, summary = CONTROLLER.prepare_validated_binaries(
                    controls['review']['validation'], validation_summary_path, output_directory)
                self.assertTrue(summary['default_link_was_last'])
                self.assertEqual(set(copied), {'tenant_density_workload', 'memory_diagnostic_worker'})
                for binary_name, identity in copied.items():
                    binary_path = output_directory / 'bin' / binary_name
                    self.assertEqual(binary_path.stat().st_size, identity['bytes'])
                    self.assertEqual(CONTROLLER.sha256_file(binary_path), identity['sha256'])
        finally:
            CONTROLLER.ROOT = original_root

    def test_helpers_resolve_from_frozen_root_without_evidence_fallback(self):
        controls_path = EVIDENCE_DIRECTORY.parent / 'arrival-phase' / 'final-review.json'
        controls = CONTROLLER.validate_controls_review(
            controls_path, CONTROLLER.sha256_file(controls_path))
        runtime_helpers = (EVIDENCE_DIRECTORY.parent / 'arrival-phase' / 'runtime'
                           / 'arrival-phase')
        capacity_helpers = EVIDENCE_DIRECTORY.parent / 'capacity'
        arrival_names = ['oci_arrival_controls.py', 'arrival_review.py', 'oci_arrival_validate.py']
        capacity_names = ['oci_capacity.py', 'capacity_review.py']
        original_root = CONTROLLER.ROOT
        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = Path(temporary) / 'frozen-root'
            evidence_directory = temporary_root / 'resident-frontier'
            evidence_directory.mkdir(parents=True)
            for helper_name in arrival_names:
                shutil.copyfile(runtime_helpers / helper_name, temporary_root / helper_name)
            for helper_name in capacity_names:
                shutil.copyfile(capacity_helpers / helper_name, temporary_root / helper_name)
            CONTROLLER.ROOT = temporary_root
            try:
                paths = CONTROLLER.controls_helper_paths(controls, evidence_directory)
                self.assertEqual(set(paths), set(arrival_names + capacity_names))
                self.assertTrue(all(path.parent == temporary_root for path in paths.values()))
            finally:
                CONTROLLER.ROOT = original_root

    def test_resident_runner_uses_arrival_run_cell_contract_with_fake_child(self):
        with tempfile.TemporaryDirectory() as temporary:
            run_directory = Path(temporary) / 'run'
            database_directory = Path(temporary) / 'databases'
            group_root = Path(temporary) / 'groups'
            binaries = run_directory / 'bin'
            binaries.mkdir(parents=True)
            database_directory.mkdir()
            controls = types.SimpleNamespace(ArrivalExperiment=ARRIVAL_MODULE.ArrivalExperiment)
            runner = CONTROLLER.ResidentRunner(
                controls, FakeCapacityRuntime(), types.SimpleNamespace(
                    validate_report=lambda report, case, backend, source_commit: None),
                {'runtime_limits': {'proc_hard': 524288}, 'source_commit': CONTROLLER.SOURCE_COMMIT},
                run_directory, database_directory, group_root, {})
            process_pid = __import__('os').getpid()
            process_stat_path = f'/proc/{process_pid}/stat'
            original_read_text = Path.read_text

            def fake_read_text(path, *args, **kwargs):
                if str(path) == process_stat_path:
                    return '1 (fake) ' + ' '.join(['S', *(['0'] * 18), '123'])
                return original_read_text(path, *args, **kwargs)

            with patch.object(ARRIVAL_MODULE.subprocess, 'Popen', FakeWorkloadProcess), \
                    patch.object(ARRIVAL_MODULE, 'parse_proc_limits', return_value={
                        'soft': 65536, 'hard': 524288, 'units': 'files'}), \
                    patch.object(ARRIVAL_MODULE.Path, 'read_text', new=fake_read_text), \
                    patch.object(ARRIVAL_MODULE, 'process_tree', return_value=[
                        {'pid': process_pid, 'parent_pid': 1, 'start_time_ticks': 123}]):
                _, outcome = runner.run_cell('screen-n16-blink', 'blink', 16, 30000, 'screen')
            self.assertTrue(outcome['reduced']['operational_pass'])
            self.assertEqual(outcome['stage'], 'screen')
            command = json.loads((run_directory / 'screen-n16-blink-command.json').read_text())
            self.assertEqual(command['stage'], 'screen')
            self.assertEqual(command['population_role'], 'all_registered_tenants_active')

    def test_entry_requires_hash_and_accepts_operational_control_with_route_slo_split(self):
        with tempfile.TemporaryDirectory() as temporary:
            review_path = Path(temporary) / 'final-review.json'
            write_json(review_path, controls_review_fixture())
            review_hash = CONTROLLER.sha256_file(review_path)
            original_hash = hashlib.sha256(review_path.read_bytes()).hexdigest()
            with self.assertRaises(AssertionError):
                CONTROLLER.validate_controls_review(review_path, '0' * 64)
            accepted = CONTROLLER.validate_controls_review(review_path, review_hash)
            self.assertEqual(hashlib.sha256(review_path.read_bytes()).hexdigest(), original_hash)
        self.assertFalse(accepted['accepted_staggered_n28_baseline_by_backend']['blink'])
        self.assertTrue(accepted['accepted_staggered_n28_baseline_by_backend']['minimal-native-shared'])

    def test_entry_rejects_incomplete_operational_controls(self):
        with tempfile.TemporaryDirectory() as temporary:
            review_path = Path(temporary) / 'final-review.json'
            review = controls_review_fixture()
            review['collection_validity']['completed_run_evidence_valid'] = False
            write_json(review_path, review)
            with self.assertRaises(AssertionError):
                CONTROLLER.validate_controls_review(review_path, CONTROLLER.sha256_file(review_path))

    def test_screen_schedule_unions_counts_and_alternates_group_order(self):
        schedule = CONTROLLER.screen_schedule({'blink': False, 'minimal-native-shared': True})
        self.assertEqual([group['active_tenants'] for group in schedule], [16, 24, 32, 48, 64, 80, 96])
        self.assertEqual(schedule[0]['backend_order'], ['blink'])
        self.assertEqual(schedule[1]['backend_order'], ['blink'])
        self.assertEqual(schedule[2]['backend_order'], ['minimal-native-shared'])
        self.assertEqual(schedule[3]['backend_order'], ['minimal-native-shared'])
        self.assertEqual([group['first_backend'] for group in schedule], [
            'blink', 'minimal-native-shared', 'blink', 'minimal-native-shared',
            'blink', 'minimal-native-shared', 'blink',
        ])

    def test_screen_slo_miss_omits_only_that_routes_higher_count(self):
        state = CONTROLLER.make_frontier_state({'blink': False, 'minimal-native-shared': False})
        fake_runner = FakeRunner({'screen-n16-blink': {'operational_pass': True, 'slo_pass': False}})
        missed = CONTROLLER.execute_screen_schedule(
            fake_runner, state, {'blink': False, 'minimal-native-shared': False}, lambda updated: None)
        self.assertTrue(missed['blink'])
        self.assertFalse(missed['minimal-native-shared'])
        self.assertEqual([call['case'] for call in fake_runner.calls], [
            'screen-n16-blink', 'screen-n16-shared', 'screen-n24-shared',
        ])
        self.assertEqual(state['omitted_cases'], [{
            'case': 'screen-n24-blink', 'stage': 'screen',
            'reason': 'higher_screen_omitted_after_first_route_slo_miss',
        }])

    def test_operational_failure_stops_screen_schedule_immediately(self):
        state = CONTROLLER.make_frontier_state({'blink': False, 'minimal-native-shared': False})
        fake_runner = FakeRunner({'screen-n16-blink': RuntimeError('operational failure')})
        with self.assertRaisesRegex(RuntimeError, 'operational failure'):
            CONTROLLER.execute_screen_schedule(
                fake_runner, state, {'blink': False, 'minimal-native-shared': False}, lambda updated: None)
        self.assertEqual(len(fake_runner.calls), 1)

    def test_candidate_requires_three_fresh_slo_passing_repetitions(self):
        screen = {32: {'operational_pass': True, 'slo_pass': True}}
        three_passes = {'operational_pass': True, 'slo_pass': True}
        result = CONTROLLER.decide_route_acceptance(
            'blink', screen, {('candidate', 32): [three_passes] * 3}, False)
        self.assertEqual(result['accepted_count'], 32)
        self.assertEqual(result['accepted_from'], 'candidate_confirmed')

    def test_one_fallback_can_be_accepted_and_failed_fallback_reuses_control(self):
        screens = {
            16: {'operational_pass': True, 'slo_pass': True},
            24: {'operational_pass': True, 'slo_pass': True},
        }
        candidate_failure = [{'operational_pass': True, 'slo_pass': False}]
        fallback_passes = [{'operational_pass': True, 'slo_pass': True}] * 3
        accepted_fallback = CONTROLLER.decide_route_acceptance(
            'blink', screens,
            {('candidate', 24): candidate_failure, ('fallback', 16): fallback_passes}, False)
        self.assertEqual(accepted_fallback['accepted_count'], 16)
        self.assertEqual(accepted_fallback['accepted_from'], 'single_fallback_confirmed')
        fallback_failure = [{'operational_pass': True, 'slo_pass': False}]
        reused_baseline = CONTROLLER.decide_route_acceptance(
            'blink', screens,
            {('candidate', 24): candidate_failure, ('fallback', 16): fallback_failure}, True)
        self.assertEqual(reused_baseline['accepted_count'], 28)
        self.assertEqual(reused_baseline['accepted_from'], 'preserved_control_baseline')
        lower_screen_not_searched = CONTROLLER.decide_route_acceptance(
            'blink', {8: screens[16], 16: screens[16], 24: screens[24]},
            {('candidate', 24): candidate_failure, ('fallback', 16): fallback_failure}, False)
        self.assertEqual(lower_screen_not_searched['fallback_count'], 16)
        self.assertIsNone(lower_screen_not_searched['accepted_count'])

    def test_review_requires_inherited_soft_and_preserved_hard_open_file_limit(self):
        records = [
            {'role': 'owned-workload-process', 'limits': {'soft': 65536, 'hard': 524288, 'units': 'files'}},
            {'role': 'owned-descendant', 'limits': {'soft': 65536, 'hard': 524288, 'units': 'files'}},
        ]
        self.assertTrue(REVIEW.validate_owned_process_limits(records))
        records[1]['limits']['hard'] = 65536
        with self.assertRaises(AssertionError):
            REVIEW.validate_owned_process_limits(records)

    def test_review_validates_resident_cell_shape_and_observation_bounds(self):
        configuration = {
            'backend': 'blink', 'registered_tenants': 32, 'active_tenants': 32,
            'opened_idle_tenants': 0, 'initial_rows_per_tenant': 2048,
            'duration_ms': 30000, 'tenant_arrival_phase_selected': 'staggered',
            'tenant_arrival_phase_applies': True, 'tenant_arrival_phase': 'staggered',
            'client_count': 1, 'seed': 20261027, 'per_tenant_rate': 50,
            'queue_bound': 8, 'max_in_flight': 8, 'request_timeout_ms': 5000,
            'drain_timeout_ms': 15000, 'profile': 'mixed',
            'width': 'mixed 80/15/5 for 1/4/16', 'value_bytes': 512,
            'topology': 'fixed-per-tenant',
            'backend_memory_budget': {'max_bytes': 256 * 1024**2,
                                      'high_bytes': 192 * 1024**2,
                                      'sample_interval_ms': 5000},
            'blink_layout': {'layout': 'segmented-bases-coalesced',
                             'execution': 'background', 'segment_bytes': 256 * 1024},
            'checkpoint_policy': 'WalBytesThreshold { threshold_bytes: 1048576 }',
        }
        report = {
            'compiled_source_revision': REVIEW.SOURCE_COMMIT,
            'requested_source_revision': REVIEW.SOURCE_COMMIT,
            'effective_configuration': configuration,
            'status': 'completed',
            'workload': {
                'cell_valid': True, 'valid_for_oracle': True,
                'accounting': {'scheduled': True, 'admitted': True},
                'scheduled': 48000, 'admitted': 48000, 'completed': 48000,
                'successful': 48000, 'dropped': 0, 'timed_out': 0,
                'per_tenant': [{} for tenant_number in range(32)],
            },
            'post_measurement_full_population_oracle': {'passed': True},
            'reopened_full_population_oracle': {'passed': True},
            'backend_memory_pressure': {
                'samples_truncated': False, 'sample_limit': 4096, 'samples': [],
            },
            'memory_diagnostic': {
                'engine_health_reports': [{
                    'periodic_pressure': {'samples_truncated': False, 'sample_limit': 4096,
                                          'serialized_sample_bytes': 1024,
                                          'serialized_sample_byte_limit': 8 * 1024 * 1024,
                                          'samples': []},
                } for epoch_number in range(4)],
            },
        }
        self.assertEqual(REVIEW.validate_report(report, 'screen-n32-blink', 'blink')['duration_ms'], 30000)
        configuration['tenant_arrival_phase'] = 'synchronized'
        with self.assertRaises(AssertionError):
            REVIEW.validate_report(report, 'screen-n32-blink', 'blink')

    def test_review_requires_exact_screen_omissions_after_first_route_miss(self):
        baselines = {'blink': False, 'minimal-native-shared': False}
        state = CONTROLLER.make_frontier_state(baselines)
        state['controls_baseline'] = {
            backend: {'accepted': False, 'cases': []} for backend in CONTROLLER.BACKENDS
        }
        state['screen_results'] = {
            'blink': {'16': {'operational_pass': True, 'slo_pass': False,
                             'case': 'screen-n16-blink'}},
            'minimal-native-shared': {
                '16': {'operational_pass': True, 'slo_pass': True, 'case': 'screen-n16-shared'},
                '24': {'operational_pass': True, 'slo_pass': True, 'case': 'screen-n24-shared'},
            },
        }
        state['candidate_counts'] = {'blink': None, 'minimal-native-shared': 24}
        state['omitted_cases'] = [
            {'case': 'screen-n24-blink', 'stage': 'screen',
             'reason': 'higher_screen_omitted_after_first_route_slo_miss'},
            {'case': 'confirm-no-candidate-blink', 'stage': 'candidate_confirmation',
             'reason': 'route_has_no_passing_screen'},
        ]
        accepted_runs = [{'case': f'confirm-n24-r{round_number}-shared',
                          'backend': 'minimal-native-shared', 'active_tenants': 24,
                          'kind': 'confirm', 'valid': True, 'operational_pass': True,
                          'slo_pass': True} for round_number in range(1, 4)]
        reviewed = {
            'screen-n16-blink': {'case': 'screen-n16-blink', 'backend': 'blink', 'active_tenants': 16,
                                 'kind': 'screen', 'valid': True, 'operational_pass': True,
                                 'slo_pass': False},
            'screen-n16-shared': {'case': 'screen-n16-shared', 'backend': 'minimal-native-shared', 'active_tenants': 16,
                                  'kind': 'screen', 'valid': True, 'operational_pass': True,
                                  'slo_pass': True},
            'screen-n24-shared': {'case': 'screen-n24-shared', 'backend': 'minimal-native-shared', 'active_tenants': 24,
                                  'kind': 'screen', 'valid': True, 'operational_pass': True,
                                  'slo_pass': True},
        }
        reviewed.update({cell['case']: cell for cell in accepted_runs})
        state['confirmation_results']['minimal-native-shared']['candidate'] = [
            {'case': cell['case'], 'active_tenants': 24,
             'operational_pass': True, 'slo_pass': True} for cell in accepted_runs
        ]
        state['decisions'] = {
            'blink': REVIEW.route_decision('blink', reviewed, False),
            'minimal-native-shared': REVIEW.route_decision('minimal-native-shared', reviewed, False),
        }
        REVIEW.validate_frontier_decisions(state, reviewed)
        state['omitted_cases'].clear()
        with self.assertRaises(AssertionError):
            REVIEW.validate_frontier_decisions(state, reviewed)


if __name__ == '__main__':
    unittest.main()
