import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys


ROOT = Path('/home/opc/tenant-density-harness-20261027')
SOURCE = ROOT / 'source'
RUN_NAME = 'shutdown-offcpu-b22638bdc-r1'
SOURCE_COMMIT = 'b22638bdcc4a0b22dad0606c3003d193cf41bbed'
DESIGN_PLAN_COMMIT = '02025ebfc8d0bfc6a4a703da23e45fd7e7268d12'
OLD_RUN_NAME = 'shutdown-b22638bdc-r1'
EXPECTED_NATIVE_ARCHIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
VALIDATION_NAME = 'shutdown-validation-summary.json'
NATIVE_ARCHIVE = Path('/bench/zfs/db/product-build/rocksdb-source/librocksdb.a')
OLD_CONTROLLER_NAME = 'oci_shutdown_diagnostic.py'


def git(source, arguments):
    return subprocess.check_output(
        ['git', '-c', f'safe.directory={source}', '-C', str(source), *arguments],
        text=True,
    ).strip()


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def load_module(name, path):
    import importlib.util

    sys.path.insert(0, str(Path(path).parent))
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


class OffCpuExperiment:
    def __init__(self, run_name, source_commit, plan_commit, observer_module, capacity_module,
                 validation, frozen_binaries, frozen_provenance):
        assert run_name == RUN_NAME
        assert source_commit == SOURCE_COMMIT
        assert len(plan_commit) == 40 and all(character in '0123456789abcdef' for character in plan_commit)
        assert git(SOURCE, ['rev-parse', 'HEAD']) == SOURCE_COMMIT
        assert not git(SOURCE, ['status', '--porcelain'])
        self.capacity = capacity_module
        self.observer_module = observer_module
        self.validation = validation
        self.frozen_binaries = frozen_binaries
        self.frozen_provenance = frozen_provenance
        self.native_archive_sha256_before = digest(NATIVE_ARCHIVE)
        assert self.native_archive_sha256_before == EXPECTED_NATIVE_ARCHIVE_SHA256
        assert validation['source_commit'] == SOURCE_COMMIT
        assert validation['passing_tests'] == 79
        assert validation['same_17_existing_clippy_diagnostics'] is True
        assert validation['native_archive_sha256_before'] == EXPECTED_NATIVE_ARCHIVE_SHA256
        assert validation['native_archive_sha256_after'] == EXPECTED_NATIVE_ARCHIVE_SHA256
        assert len(validation['outcomes']) == 4
        assert all(outcome['exit_code'] == 0 for outcome in validation['outcomes'])
        assert validation['outcomes'][-1]['name'] == 'shutdown-clean-default-link'
        assert self.frozen_provenance['source_commit'] == SOURCE_COMMIT
        self.source_status_before = git(SOURCE, ['status', '--porcelain'])
        self.old_controller_sha256 = capacity_module.digest(
            ROOT / OLD_CONTROLLER_NAME
        )
        self.trace_source = SOURCE / 'dodb/crates/dodb-rocksdb/src/bin/support/shutdown_trace.rs'
        self.trace_source_sha256 = capacity_module.digest(self.trace_source)
        expected_trace_sha256 = self.frozen_provenance['shutdown_trace_source_sha256']
        assert self.trace_source_sha256 == expected_trace_sha256
        self.source_identity = {
            'source_commit': SOURCE_COMMIT,
            'source_clean': True,
            'source_status_before': self.source_status_before,
            'trace_source_sha256': self.trace_source_sha256,
            'accepted_validation_sha256': capacity_module.digest(ROOT / VALIDATION_NAME),
            'accepted_validation_passing_tests': validation['passing_tests'],
            'accepted_validation_same_17_existing_clippy_diagnostics':
                validation['same_17_existing_clippy_diagnostics'],
            'native_archive_sha256_before': self.native_archive_sha256_before,
            'frozen_binary_identities': self.frozen_provenance['binaries'],
        }
        self.base = capacity_module.Experiment(run_name, source_commit, plan_commit)
        self.run_directory = self.base.run_directory
        self.database_directory = self.base.database_directory
        self.group_root = self.base.group_root
        self.outcomes = []
        self.observer_results = []
        self.capacity_module = capacity_module
        self.validation_sha256 = capacity_module.digest(ROOT / VALIDATION_NAME)
        self._verify_copied_binaries()
        provenance_path = self.run_directory / 'provenance.json'
        provenance = json.loads(provenance_path.read_text())
        provenance.update({
            'run_name': RUN_NAME,
            'invocation': {
                'run_name': run_name,
                'source_commit': source_commit,
                'controller_matrix_commit': plan_commit,
            },
            'controller_matrix_commit': plan_commit,
            'design_plan_commit': DESIGN_PLAN_COMMIT,
            'offcpu_controller_sha256': capacity_module.digest(Path(__file__)),
            'offcpu_observer_sha256': capacity_module.digest(Path(observer_module.__file__)),
            'offcpu_collector_sha256': capacity_module.digest(
                Path(__file__).with_name('oci_shutdown_offcpu_collect.py')
            ),
            'offcpu_reviewer_sha256': capacity_module.digest(
                Path(__file__).with_name('review_shutdown_offcpu.py')
            ),
            'accepted_controller_sha256': self.old_controller_sha256,
            'source_identity': self.source_identity,
            'accepted_validation_sha256': self.validation_sha256,
            'capability_probe_path': str(ROOT / 'capacity' / (RUN_NAME + '-capability-probe.json')),
            'runtime_sequence': [
                {'case': f'offcpu-e3-n5-r{repetition}-blink', 'backend': 'blink',
                 'duration_ms': 60000, 'sample_interval_ms': 1000,
                 'registered_tenants': 5, 'active_tenants': 5,
                 'initial_rows_per_tenant': 32768, 'per_tenant_rate': 50,
                 'backend_memory_max_bytes': 268435456,
                 'backend_memory_high_bytes': 201326592,
                 'backend_memory_swap_max_bytes': 0}
                for repetition in range(1, 4)
            ],
            'observer_limits': {
                'poll_interval_ms': 100, 'shutdown_pending_ms': 2000,
                'capture_interval_ms': 1000, 'max_captures_per_worker': 24,
                'max_threads_per_worker': 32, 'max_file_bytes': 1024,
                'max_status_bytes': 4096,
                'max_worker_output_bytes': 2097152,
                'max_total_output_bytes': 16777216,
                'max_worker_captured_file_bytes': 4194304,
                'max_errors_per_worker': 128,
                'max_observer_errors': 256,
                'max_observer_wall_seconds': 1230,
            },
            'scope': 'B-link-only E3 five-tenant shutdown wait attribution; no capacity or performance ranking',
        })
        self.capacity_module.write_json(provenance_path, provenance)
        self.capacity_module.guard(self.capacity_module.capacity(), before=True)
        self.assert_frozen_inputs()

    def _verify_copied_binaries(self):
        for name, frozen in self.frozen_binaries.items():
            copied = self.run_directory / 'bin' / name
            assert copied.is_file()
            assert copied.stat().st_size == frozen['bytes']
            assert self.capacity_module.digest(copied) == frozen['sha256']

    def assert_frozen_inputs(self):
        assert git(SOURCE, ['rev-parse', 'HEAD']) == SOURCE_COMMIT
        assert git(SOURCE, ['status', '--porcelain']) == self.source_status_before
        assert self.capacity_module.digest(self.trace_source) == self.trace_source_sha256
        assert self.capacity_module.digest(ROOT / VALIDATION_NAME) == self.validation_sha256
        self._verify_copied_binaries()

    def verify_native_archive_hash(self):
        native_archive_sha256_after = digest(NATIVE_ARCHIVE)
        assert native_archive_sha256_after == self.native_archive_sha256_before
        assert native_archive_sha256_after == EXPECTED_NATIVE_ARCHIVE_SHA256
        return native_archive_sha256_after

    def run_cell(self, case, repetition):
        assert case == f'offcpu-e3-n5-r{repetition}-blink'
        log_path = self.run_directory / (case + '.log')
        observation_path = self.run_directory / (case + '-offcpu.jsonl')
        observer = self.observer_module.OffCpuObserver(
            log_path,
            observation_path,
            SOURCE_COMMIT,
            self.group_root / case,
        )
        observer.start()
        outcome = None
        primary_error = None
        try:
            outcome = self.base.run_cell(
                case, 'E3', 'blink', 5, 60000, interval_ms=1000,
            )
        except Exception as error:
            primary_error = error
        finally:
            observer.stop_event.set()
            try:
                result = observer.stop()
            except Exception as observer_error:
                result = {
                    'observer_failure': f'{type(observer_error).__name__}:{observer_error}'[:512],
                    'workers': [],
                }
            self.capacity_module.write_json(
                self.run_directory / (case + '-offcpu-status.json'), result,
            )
            self.observer_results.append(result)
        if primary_error is not None:
            raise primary_error
        if result.get('observer_failure') or result.get('observer_errors'):
            raise AssertionError({'case': case, 'invalid_observer_result': result})
        assert outcome is not None
        assert outcome['reduced']['operational_pass'] is True, outcome
        report = json.loads((self.run_directory / (case + '.json')).read_text())
        assert report['status'] == 'completed'
        assert report['post_measurement_full_population_oracle']['passed'] is True
        assert report['reopened_full_population_oracle']['passed'] is True
        expected_workers = {
            (identity['pid'], identity['start_time_ticks'])
            for identity in report['memory_diagnostic']['processes']
            if identity['role'] == 'backend-process'
        }
        observation_records = [
            json.loads(line) for line in observation_path.read_text().splitlines()
        ]
        observed_workers = {
            (record['marker']['process_id'], record['marker']['process_start_time_ticks'])
            for record in observation_records
            if record.get('record_type') == 'shutdown_marker'
        }
        assert expected_workers and observed_workers == expected_workers, {
            'expected_workers': sorted(expected_workers),
            'observed_workers': sorted(observed_workers),
        }
        worker_results = {
            (worker['process_id'], worker['process_start_time_ticks']): worker
            for worker in result.get('workers', [])
        }
        assert set(worker_results) == expected_workers
        markers = {
            (record['marker']['process_id'], record['marker']['process_start_time_ticks']): record
            for record in observation_records
            if record.get('record_type') == 'shutdown_marker'
        }
        assert set(markers) == expected_workers
        for worker_key, worker in worker_results.items():
            marker_identity_valid = markers[worker_key]['identity_verification']['valid']
            finished_before_sampling = (
                worker['observation_status'] == 'finished_before_sampling'
                and worker['terminal_stage'] == 'async_server_shutdown_end'
                and worker['captures'] == 0
            )
            valid_sampled_identity = (
                worker['observation_status'] == 'sampled_then_finished'
                and worker['terminal_stage'] == 'async_server_shutdown_end'
                and worker['captures'] > 0
                and worker['identity_valid']
            )
            assert (
                worker['terminal_stage'] == 'async_server_shutdown_end'
                and worker['error_count'] == 0
                and (marker_identity_valid and worker['identity_valid']
                     or finished_before_sampling or valid_sampled_identity)
            ), {'worker': worker, 'marker': markers[worker_key]}
        assert self.capacity_module.process_identity_live(self.base.all_processes[-1]) is False
        self.outcomes.append(outcome)
        self.assert_frozen_inputs()
        return outcome

    def execute(self):
        for repetition in range(1, 4):
            case = f'offcpu-e3-n5-r{repetition}-blink'
            self.run_cell(case, repetition)
        assert len(self.outcomes) == 3
        assert all(outcome['reduced']['operational_pass'] for outcome in self.outcomes)
        self.assert_frozen_inputs()
        native_archive_sha256_after = self.verify_native_archive_hash()
        final = {
            'completed': True,
            'source_commit': SOURCE_COMMIT,
            'attempted_cells': len(self.base.outcomes),
            'completed_cells': sum(
                outcome.get('reduced', {}).get('operational_pass') is True
                for outcome in self.base.outcomes
            ),
            'cells': len(self.outcomes),
            'cases': [outcome['case'] for outcome in self.outcomes],
            'stall_reproduced': False,
            'interpretation': 'The earlier stall was not reproduced in this bounded B-link-only variant',
            'capacity_or_performance_ranking': None,
            'native_archive_sha256_before': self.native_archive_sha256_before,
            'native_archive_sha256_after': native_archive_sha256_after,
            'capacity_after': self.capacity_module.capacity(),
        }
        self.capacity_module.guard(final['capacity_after'])
        self.capacity_module.write_json(self.run_directory / 'completion.json', final)
        self.assert_frozen_inputs()
        return final


def prepare_run(arguments):
    assert arguments.run_name == RUN_NAME
    assert arguments.source_commit == SOURCE_COMMIT
    assert len(arguments.plan_commit) == 40 and all(
        character in '0123456789abcdef' for character in arguments.plan_commit
    )
    assert git(ROOT, ['rev-parse', 'HEAD']) == arguments.plan_commit
    observer_module = load_module(
        'offcpu_observer', Path(__file__).with_name('offcpu_observer.py'),
    )
    probe_path = ROOT / 'capacity' / (RUN_NAME + '-capability-probe.json')
    probe_path.parent.mkdir(parents=True, exist_ok=True)
    if probe_path.exists():
        probe = json.loads(probe_path.read_text())
        assert probe['probe_kind'] == 'owned_sleep_child_proc_read_capability'
        assert probe['probe_child_terminated'] is True
    else:
        probe = observer_module.run_probe(probe_path)
    if not probe['wait_fields_available']:
        failure_path = ROOT / 'capacity' / (RUN_NAME + '-probe-review-required.json')
        if not failure_path.exists():
            ROOT.joinpath('capacity').mkdir(parents=True, exist_ok=True)
            failure_path.write_text(json.dumps({
                'status': 'probe_review_required',
                'source_commit': SOURCE_COMMIT,
                'probe_path': str(probe_path),
                'reason': 'none of wchan, syscall, or stack produced a readable wait value',
            }, indent=2) + '\n')
        raise RuntimeError('wait field capability probe requires review; runtime stopped')
    validation_path = ROOT / VALIDATION_NAME
    validation = json.loads(validation_path.read_text())
    old_run_directory = ROOT / 'capacity' / OLD_RUN_NAME
    old_provenance = json.loads((old_run_directory / 'provenance.json').read_text())
    frozen_binaries = old_provenance['binaries']
    for binary_name, identity in frozen_binaries.items():
        frozen_path = old_run_directory / 'bin' / binary_name
        assert frozen_path.is_file()
        assert frozen_path.stat().st_size == identity['bytes']
        assert digest(frozen_path) == identity['sha256']
    capacity_module = load_module(
        'oci_capacity', ROOT / 'oci_capacity.py',
    )
    for binary_name, identity in frozen_binaries.items():
        built_path = ROOT / 'target/release' / binary_name
        assert digest(built_path) == identity['sha256']
    experiment = OffCpuExperiment(
        arguments.run_name, arguments.source_commit, arguments.plan_commit,
        observer_module, capacity_module, validation,
        frozen_binaries, old_provenance,
    )
    shutil.copy2(probe_path, experiment.run_directory / 'capability-probe.json')
    return experiment


def main():
    parser = argparse.ArgumentParser(description='Run bounded B-link shutdown wait observations')
    parser.add_argument('--run-name', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--plan-commit', required=True)
    arguments = parser.parse_args()
    experiment = prepare_run(arguments)
    try:
        final = experiment.execute()
    except Exception as error:
        failure = {
            'status': 'failed_or_invalid',
            'reason': f'{type(error).__name__}:{error}'[:1024],
            'attempted_cells': len(experiment.base.outcomes),
            'completed_cells': sum(
                outcome.get('reduced', {}).get('operational_pass') is True
                for outcome in experiment.base.outcomes
            ),
            'accepted_observations': len(experiment.outcomes),
            'cells': len(experiment.outcomes),
            'capacity': experiment.capacity_module.capacity(),
            'observer_results': experiment.observer_results,
        }
        try:
            experiment.assert_frozen_inputs()
        except Exception as verification_error:
            failure['frozen_input_verification_error'] = \
                f'{type(verification_error).__name__}:{verification_error}'[:512]
        try:
            failure['native_archive_sha256_after'] = experiment.verify_native_archive_hash()
        except Exception as native_verification_error:
            failure['native_archive_verification_error'] = \
                f'{type(native_verification_error).__name__}:{native_verification_error}'[:512]
        experiment.capacity_module.write_json(
            experiment.run_directory / 'controller-failure.json', failure,
        )
        raise
    print(json.dumps({
        'run_name': arguments.run_name,
        'completed_cells': final['cells'],
        'source_commit': final['source_commit'],
        'stall_reproduced': final['stall_reproduced'],
    }), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
