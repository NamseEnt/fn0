import argparse
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path('/home/opc/tenant-density-harness-20261027')
sys.path.insert(0, str(ROOT))
from oci_harness_build import digest
import oci_capacity
from oci_capacity import Experiment, SOURCE, capacity, process_identity_live, write_json


SOURCE_COMMIT = 'b22638bdcc4a0b22dad0606c3003d193cf41bbed'
EXPECTED_NATIVE_ARCHIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
VALIDATION_NAME = 'shutdown-validation-summary.json'
NATIVE_ARCHIVE = Path('/bench/zfs/db/product-build/rocksdb-source/librocksdb.a')
TRACE_SOURCE = SOURCE / 'dodb/crates/dodb-rocksdb/src/bin/support/shutdown_trace.rs'
VALIDATION_CHECK_NAMES = [
    'shutdown-clean-tests', 'shutdown-clean-clippy',
    'shutdown-clean-a1-link', 'shutdown-clean-default-link',
]
BASE_REDUCE_REPORT = oci_capacity.reduce_report
TRACE_FIELDS = {'timestamp_unix_ns', 'process_id', 'process_start_time_ticks', 'source_revision', 'stage'}
REQUIRED_NORMAL_SHUTDOWN_STAGES = [
    'command_received_shutdown',
    'final_property_capture_start', 'final_property_capture_end',
    'async_server_shutdown_start', 'async_server_shutdown_end',
    'final_checkpoint_metrics_capture_start', 'final_checkpoint_metrics_capture_end',
    'server_drop_start', 'server_drop_end',
    'backend_drop_start', 'backend_drop_end',
    'health_report_write_start', 'health_report_write_end',
    'async_run_return_success', 'runtime_destruction_start', 'runtime_destruction_end',
]


def git(arguments):
    return subprocess.check_output(
        ['git', '-c', f'safe.directory={SOURCE}', '-C', str(SOURCE), *arguments], text=True
    ).strip()


class ShutdownExperiment(Experiment):
    def __init__(self, run_name, plan_commit):
        validation_path = ROOT / VALIDATION_NAME
        assert validation_path.is_file()
        validation = json.loads(validation_path.read_text())
        assert validation['source_commit'] == SOURCE_COMMIT
        assert validation['passing_tests'] == 79
        assert validation['baseline_passing_tests'] == 74
        assert validation['same_17_existing_clippy_diagnostics']
        assert validation['native_archive_sha256_before'] == EXPECTED_NATIVE_ARCHIVE_SHA256
        assert validation['native_archive_sha256_after'] == EXPECTED_NATIVE_ARCHIVE_SHA256
        outcomes = validation['outcomes']
        assert [outcome['name'] for outcome in outcomes] == VALIDATION_CHECK_NAMES
        assert all(outcome['exit_code'] == 0 for outcome in outcomes)
        assert outcomes[-1]['name'] == 'shutdown-clean-default-link'
        assert git(['rev-parse', 'HEAD']) == SOURCE_COMMIT
        assert not git(['status', '--porcelain'])
        super().__init__(run_name, SOURCE_COMMIT, plan_commit)
        self.native_archive_sha256 = digest(NATIVE_ARCHIVE)
        self.native_archive_bytes = NATIVE_ARCHIVE.stat().st_size
        self.target_link = (ROOT / 'target').readlink()
        self.target_directory = (ROOT / 'target').resolve(strict=True)
        self.validation_sha256 = digest(validation_path)
        self.trace_sha256 = digest(TRACE_SOURCE)
        self.trace_validation_failure = None
        provenance_path = self.run_directory / 'provenance.json'
        provenance = json.loads(provenance_path.read_text())
        self.binary_identities = provenance['binaries']
        provenance.update({
            'shutdown_controller_sha256': digest(Path(__file__)),
            'shutdown_trace_source_sha256': self.trace_sha256,
            'shutdown_validation_sha256': self.validation_sha256,
            'shutdown_validation_source_commit': validation['source_commit'],
            'shutdown_validation_passing_tests': validation['passing_tests'],
            'shutdown_validation_same_17_existing_clippy_diagnostics': validation['same_17_existing_clippy_diagnostics'],
            'native_archive_sha256_before': self.native_archive_sha256,
            'native_archive_bytes_before': self.native_archive_bytes,
            'target_symlink_before': str(self.target_link),
            'resolved_target_directory_before': str(self.target_directory),
            'shutdown_sequence': [
                {'case': 'tiny-e2-n1-ms1000', 'duration_ms': 2000, 'sample_interval_ms': 1000, 'first_backend': 'blink'},
                {'case': 'confirm-e3-n5-r1', 'duration_ms': 60000, 'sample_interval_ms': 1000, 'first_backend': 'minimal-native-shared'},
                {'case': 'confirm-e3-n5-r2', 'duration_ms': 60000, 'sample_interval_ms': 1000, 'first_backend': 'blink'},
                {'case': 'confirm-e3-n5-r3', 'duration_ms': 60000, 'sample_interval_ms': 1000, 'first_backend': 'minimal-native-shared'},
            ],
            'scope': 'Shutdown phase attribution only; unchanged workload, storage, deadlines, resource policy, and engine settings',
        })
        write_json(provenance_path, provenance)
        oci_capacity.reduce_report = self.review_report

    def review_report(self, report, source_commit):
        assert source_commit == SOURCE_COMMIT
        return BASE_REDUCE_REPORT(report, source_commit)

    def run_cell(self, case, stage, backend, tenants, duration_ms, interval_ms=1000, active=None, idle=0):
        outcome = super().run_cell(case, stage, backend, tenants, duration_ms, interval_ms, active, idle)
        assert outcome['reduced']['operational_pass'], outcome
        return outcome

    def assert_frozen_inputs(self):
        assert git(['rev-parse', 'HEAD']) == SOURCE_COMMIT
        assert not git(['status', '--porcelain'])
        target = ROOT / 'target'
        assert target.is_symlink() and target.readlink() == self.target_link
        assert target.resolve(strict=True) == self.target_directory
        assert NATIVE_ARCHIVE.stat().st_size == self.native_archive_bytes
        assert digest(TRACE_SOURCE) == self.trace_sha256

    def verify_native_archive_hash(self):
        assert NATIVE_ARCHIVE.stat().st_size == self.native_archive_bytes
        assert digest(NATIVE_ARCHIVE) == self.native_archive_sha256

    def verify_normal_shutdown_trace(self, case):
        report_path = self.run_directory / (case + '.json')
        report = json.loads(report_path.read_text())
        assert report['status'] == 'completed'
        assert 'partial_workload_counters' not in report
        diagnostic = report['memory_diagnostic']
        assert diagnostic['source_revision'] == SOURCE_COMMIT
        process_ids = {
            (identity['pid'], identity['start_time_ticks'])
            for identity in diagnostic['processes'] if identity['role'] == 'backend-process'
        }
        lifecycle_ids = {event['pid'] for event in diagnostic['worker_lifecycle']}
        assert process_ids and {pid for pid, _ in process_ids} <= lifecycle_ids
        assert all(event['source_revision'] == SOURCE_COMMIT for event in diagnostic['worker_lifecycle'])
        assert all(event['harness_binary_sha256'] == self.binary_identities['tenant_density_workload']['sha256']
                   for event in diagnostic['worker_lifecycle'])
        assert all(event['worker_binary_sha256'] == self.binary_identities['memory_diagnostic_worker']['sha256']
                   for event in diagnostic['worker_lifecycle'])
        events_by_identity = {}
        log_path = self.run_directory / (case + '.log')
        with log_path.open() as log_file:
            for line_number, line in enumerate(log_file, start=1):
                if '"timestamp_unix_ns"' not in line:
                    continue
                assert len(line.encode()) <= 4096
                event = json.loads(line)
                assert type(event) is dict and set(event) == TRACE_FIELDS
                assert event['source_revision'] == SOURCE_COMMIT
                assert type(event['timestamp_unix_ns']) is int
                identity_key = (event['process_id'], event['process_start_time_ticks'])
                assert identity_key in process_ids, (case, line_number, identity_key)
                events_by_identity.setdefault(identity_key, []).append(event)
        for identity_key in process_ids:
            identity_events = events_by_identity.get(identity_key, [])
            assert identity_events, (case, identity_key, 'missing shutdown trace')
            stages = [event['stage'] for event in identity_events]
            cursor = -1
            for required_stage in REQUIRED_NORMAL_SHUTDOWN_STAGES:
                try:
                    cursor = stages.index(required_stage, cursor + 1)
                except ValueError as error:
                    raise AssertionError((case, identity_key, required_stage, stages)) from error
            timestamps = [event['timestamp_unix_ns'] for event in identity_events]
            assert timestamps == sorted(timestamps)
            assert stages[-1] == 'runtime_destruction_end'

    def execute_shutdown_sequence(self):
        sequence = [
            ('tiny-e2-n1-ms1000', 'E2', 1, 2000),
            ('confirm-e3-n5-r1', 'E3', 5, 60000),
            ('confirm-e3-n5-r2', 'E3', 5, 60000),
            ('confirm-e3-n5-r3', 'E3', 5, 60000),
        ]
        for name, stage, tenants, duration_ms in sequence:
            outcomes = self.pair(name, stage, tenants, duration_ms, interval_ms=1000)
            assert len(outcomes) == 2
            assert all(outcome['reduced']['operational_pass'] for outcome in outcomes.values())
            assert (self.run_directory / (name + '-pair.json')).is_file()
            if name == 'tiny-e2-n1-ms1000':
                try:
                    for outcome in outcomes.values():
                        self.verify_normal_shutdown_trace(outcome['case'])
                except Exception as error:
                    self.trace_validation_failure = {
                        'pair': name,
                        'cases': [outcome['case'] for outcome in outcomes.values()],
                        'failing_case': outcome['case'],
                        'error': repr(error),
                    }
                    raise
            self.assert_frozen_inputs()
        assert len(self.outcomes) == 8
        assert all(outcome['reduced']['operational_pass'] for outcome in self.outcomes)
        self.verify_native_archive_hash()
        final = {
            'completed': True,
            'source_commit': SOURCE_COMMIT,
            'cases': len(self.outcomes),
            'pairs': 4,
            'tiny_pair_shutdown_trace_validated': True,
            'earlier_shutdown_failure_reproduced': False,
            'capacity_after': capacity(),
            'processes': [
                {'pid': identity['pid'], 'role': identity['role'],
                 'start_time_ticks': identity['start_time_ticks'],
                 'same_live_process': process_identity_live(identity)}
                for identity in self.all_processes
            ],
        }
        assert all(not identity['same_live_process'] for identity in final['processes'])
        self.assert_frozen_inputs()
        write_json(self.run_directory / 'completion.json', final)
        return final


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--run-name', required=True)
    parser.add_argument('--source-commit', required=True)
    parser.add_argument('--plan-commit', required=True)
    arguments = parser.parse_args()
    assert arguments.source_commit == SOURCE_COMMIT
    assert arguments.run_name == 'shutdown-b22638bdc-r1'
    experiment = ShutdownExperiment(arguments.run_name, arguments.plan_commit)
    try:
        final = experiment.execute_shutdown_sequence()
    except Exception as error:
        failure = {
            'reason': repr(error), 'completed_cells': len(experiment.outcomes),
            'capacity': capacity(),
        }
        if experiment.trace_validation_failure is not None:
            failure['failure_kind'] = 'shutdown_trace_validation'
            failure['last_operational_pass'] = bool(
                experiment.outcomes and experiment.outcomes[-1].get('reduced', {}).get('operational_pass')
            )
            failure['trace_validation'] = experiment.trace_validation_failure
        try:
            experiment.assert_frozen_inputs()
            experiment.verify_native_archive_hash()
        except Exception as verification_error:
            failure['frozen_input_verification_error'] = repr(verification_error)
        write_json(experiment.run_directory / 'controller-failure.json', failure)
        raise
    print(json.dumps({
        'run_name': arguments.run_name,
        'completed_cells': final['cases'],
        'pairs': final['pairs'],
        'source_commit': final['source_commit'],
        'earlier_shutdown_failure_reproduced': final['earlier_shutdown_failure_reproduced'],
    }), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
