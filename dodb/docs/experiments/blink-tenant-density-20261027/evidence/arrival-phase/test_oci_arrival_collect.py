import hashlib
import importlib.util
import json
import io
import shutil
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


COLLECTOR_PATH = Path(__file__).with_name('oci_arrival_collect.py')
SPEC = importlib.util.spec_from_file_location('oci_arrival_collect_under_test', COLLECTOR_PATH)
COLLECTOR = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = COLLECTOR
SPEC.loader.exec_module(COLLECTOR)


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


class TrackingReader:
    def __init__(self, data):
        self.stream = io.BytesIO(data)
        self.read_sizes = []

    def __enter__(self):
        return self

    def __exit__(self, exception_type, exception, traceback):
        self.stream.close()

    def read(self, size):
        self.read_sizes.append(size)
        return self.stream.read(size)


class ArrivalCollectionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name) / 'harness'
        self.root.mkdir()
        self.source = self.root / 'source'
        self.source.mkdir()
        self.native = Path(self.temporary.name) / 'native' / 'librocksdb.a'
        self.native.parent.mkdir()
        self.native.write_bytes(b'native fixture payload')
        self.native_sha256 = COLLECTOR.digest(self.native)
        self.run_name = 'arrival-controls-fixture-r1'
        self.validation_run = 'arrival-fixture-r1'
        self.database_root = Path(self.temporary.name) / 'bench' / 'db' / self.run_name
        self.grouping_root = Path(self.temporary.name) / 'sysfs' / ('dodb-density-' + self.run_name)
        self.arrival_directory = self.root / 'arrival-phase'
        self.controls_directory = self.arrival_directory / self.run_name
        self.validation_directory = self.arrival_directory / self.validation_run
        self.attempt_name = 'arrival-control-collection-fixture-r1'
        self.case = 'tiny-synchronized-blink'
        self.case_group = self.grouping_root / self.case
        self.database_path = self.database_root / self.case
        self._create_groups()
        self.database_path.mkdir(parents=True)
        (self.database_path / 'CURRENT').write_text('preserved database fixture')
        self._create_validation()
        self._create_controls()
        self._create_helpers()
        self._create_launchers()
        COLLECTOR.ROOT = self.root
        COLLECTOR.SOURCE = self.source
        COLLECTOR.RUN_NAME = self.run_name
        COLLECTOR.CONTROL_LAUNCHER_PREFIX = 'arrival-controls-fixture-r'
        COLLECTOR.VALIDATION_RUN = self.validation_run
        COLLECTOR.DEFAULT_ATTEMPT = self.attempt_name
        COLLECTOR.SOURCE_COMMIT = 'cb0e0448270c1da3d9a214fb008ec01406d8e40d'
        COLLECTOR.EXPECTED_NATIVE_COMMIT = 'native-fixture'
        COLLECTOR.EXPECTED_NATIVE_SHA256 = self.native_sha256
        COLLECTOR.NATIVE_ARCHIVE = self.native
        COLLECTOR.DATABASE_ROOT = self.database_root
        COLLECTOR.GROUPING_ROOT = self.grouping_root
        self.git_patcher = patch.object(COLLECTOR, 'git', side_effect=self.fake_git)
        self.git_patcher.start()
        self.native_git_patcher = patch.object(COLLECTOR.subprocess, 'check_output', return_value='native-fixture\n')
        self.native_git_patcher.start()
        self.space_patcher = patch.object(COLLECTOR, 'collect_space', return_value={'fixture': {'available_bytes': 1}})
        self.space_patcher.start()

    def tearDown(self):
        self.space_patcher.stop()
        self.native_git_patcher.stop()
        self.git_patcher.stop()
        self.temporary.cleanup()

    def fake_git(self, arguments):
        if arguments == ['rev-parse', 'HEAD']:
            return COLLECTOR.SOURCE_COMMIT
        if arguments == ['status', '--porcelain']:
            return ''
        raise AssertionError(f'unexpected source git command {arguments}')

    def _write_group(self, path, root_group=False):
        path.mkdir(parents=True, exist_ok=True)
        values = {
            'memory.max': 'max' if root_group else str(COLLECTOR.MEMORY_MAX_BYTES),
            'memory.high': 'max' if root_group else str(COLLECTOR.MEMORY_HIGH_BYTES),
            'memory.swap.max': 'max' if root_group else '0',
            'memory.current': '0' if root_group else '4096',
            'cgroup.procs': '',
        }
        for name, value in values.items():
            (path / name).write_text(value + '\n')

    def _create_groups(self):
        self._write_group(self.grouping_root, root_group=True)
        self._write_group(self.case_group)
        for epoch in COLLECTOR.EPOCHS:
            self._write_group(self.case_group / epoch)

    def _create_validation(self):
        binaries = {}
        for binary_name in ['tenant_density_workload', 'memory_diagnostic_worker']:
            path = self.validation_directory / 'bin' / binary_name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(f'validated {binary_name}'.encode())
            binaries[binary_name] = {'sha256': COLLECTOR.digest(path), 'bytes': path.stat().st_size}
        outcomes = [
            {'name': 'tests', 'exit_code': 0, 'passing_tests': 85,
             'test_group_counts': [42, 9, 25, 5, 4]},
            {'name': 'clippy', 'exit_code': 0, 'same_17_existing_clippy_diagnostics': True},
            {'name': 'a1-link', 'exit_code': 0},
            {'name': 'default-link', 'exit_code': 0},
        ]
        self.validation_summary_path = self.validation_directory / 'arrival-validation-summary.json'
        write_json(self.validation_summary_path, {
            'status': 'completed',
            'source_commit': COLLECTOR.SOURCE_COMMIT,
            'controller_sha256': 'pending',
            'outcomes': outcomes,
            'default_link_was_last': True,
            'native_archive_sha256_before': self.native_sha256,
            'native_archive_sha256_after': self.native_sha256,
            'default_binaries': binaries,
        })
        for check_name in COLLECTOR.VALIDATION_CHECKS:
            build_directory = self.root / 'builds' / f'{self.validation_run}-{check_name}'
            build_directory.mkdir(parents=True)
            (build_directory / 'cargo.log').write_text(f'{check_name} fixture log\n')
            write_json(build_directory / 'provenance.json', {'source_commit': COLLECTOR.SOURCE_COMMIT})
            write_json(build_directory / 'status.json', {'exit_code': 0})

    def _create_controls(self):
        for binary_name in ['tenant_density_workload', 'memory_diagnostic_worker']:
            source_binary = self.validation_directory / 'bin' / binary_name
            destination = self.controls_directory / 'bin' / binary_name
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source_binary, destination)
        self.command_path = self.controls_directory / (self.case + '-command.json')
        data_directory = self.database_root / self.case
        group_path = self.grouping_root / self.case
        write_json(self.command_path, {
            'command': [
                '/bin/tenant_density_workload', '--data-dir', str(data_directory),
                '--backend-cgroup-root', str(group_path),
            ],
            'source_commit': COLLECTOR.SOURCE_COMMIT,
        })
        status = {
            'case': self.case,
            'exit_code': 1,
            'stop_reason': 'mocked_failure',
            'review_error': 'preserved failure reason',
            'owned_process_limits': [],
            'reduced': {'operational_pass': False, 'slo_pass': False},
        }
        write_json(self.controls_directory / (self.case + '-status.json'), status)
        (self.controls_directory / (self.case + '.log')).write_text('preserved cell log\n')
        write_json(self.controls_directory / (self.case + '-host-samples.json'), [])
        self.terminal_path = self.controls_directory / 'failure-receipt.json'
        write_json(self.terminal_path, {
            'completed': False,
            'source_commit': COLLECTOR.SOURCE_COMMIT,
            'plan_commit': 'd' * 40,
            'cases_attempted': 1,
            'outcomes': [{'case': self.case, 'exit_code': 1, 'review_error': 'preserved failure reason'}],
            'processes': [],
        })

    def _create_helpers(self):
        evidence_directory = COLLECTOR_PATH.parent
        for helper_name in ['oci_arrival_validate.py', 'oci_arrival_controls.py', 'arrival_review.py']:
            source_path = evidence_directory / helper_name
            destination_path = self.arrival_directory / helper_name
            destination_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source_path, destination_path)
        capacity_directory = evidence_directory.parent / 'capacity'
        for helper_name in ['oci_capacity.py', 'capacity_review.py']:
            shutil.copyfile(capacity_directory / helper_name, self.root / helper_name)
        validation = json.loads(self.validation_summary_path.read_text())
        validation['controller_sha256'] = COLLECTOR.digest(self.arrival_directory / 'oci_arrival_validate.py')
        write_json(self.validation_summary_path, validation)
        controls_provenance = {
            'source_commit': COLLECTOR.SOURCE_COMMIT,
            'run_name': self.run_name,
            'plan_commit': 'd' * 40,
            'validation_sha256': COLLECTOR.digest(self.validation_summary_path),
            'controller_sha256': COLLECTOR.digest(self.arrival_directory / 'oci_arrival_controls.py'),
            'reviewer_sha256': COLLECTOR.digest(self.arrival_directory / 'arrival_review.py'),
            'capacity_controller_sha256': COLLECTOR.digest(self.root / 'oci_capacity.py'),
            'capacity_reviewer_sha256': COLLECTOR.digest(self.root / 'capacity_review.py'),
        }
        write_json(self.controls_directory / 'provenance.json', controls_provenance)

    def _create_launchers(self):
        for run_name in [self.run_name, 'arrival-controls-fixture-r2', self.validation_run]:
            (self.root / f'{run_name}-launcher.log').write_text('launcher fixture\n')
            (self.root / f'{run_name}-pid.txt').write_text('987654321\n')

    def test_terminal_failure_is_archived_without_green_promotion_and_with_limits_manifest(self):
        receipt = COLLECTOR.collect(self.attempt_name)
        self.assertEqual(receipt['collection_status'], 'completed')
        self.assertEqual(receipt['controls_terminal_status'], 'failed')
        self.assertFalse(receipt['controls_success_preserved'])
        self.assertFalse(receipt['controls_validity']['completed_run_evidence_valid'])
        self.assertFalse(receipt['controls_validity']['accepted_run'])
        self.assertTrue(receipt['controls_validity']['grouping_root_memory_max_is_unbounded'])
        self.assertTrue(receipt['controls_validity']['cell_epoch_budgets_match'])
        self.assertEqual(receipt['database_preserved_case_count'], 1)
        self.assertTrue(receipt['controls_validity']['all_attempted_cell_databases_preserved'])
        self.assertFalse(receipt['controls_validity']['all_completed_cell_databases_preserved'])
        self.assertEqual(len(receipt['controls']['launchers']), 3)
        self.assertEqual(receipt['controls']['launcher_attempt_count'], 2)
        self.assertEqual(receipt['controls']['preflight_launch_attempts'][0]['actual_workload_cells'], 0)
        self.assertEqual(receipt['controls']['actual_cell_command_count'], 1)
        self.assertEqual(receipt['controls']['actual_cell_status_count'], 1)
        self.assertEqual((self.database_path / 'CURRENT').read_text(), 'preserved database fixture')
        groups = receipt['controls']['cgroups']['groups']
        root_group = groups[str(self.grouping_root)]
        self.assertEqual(root_group['memory.max'], 'max')
        self.assertEqual(groups[str(self.case_group)]['memory.max'], str(COLLECTOR.MEMORY_MAX_BYTES))
        self.assertEqual(groups[str(self.case_group / 'measurement')]['memory.high'], str(COLLECTOR.MEMORY_HIGH_BYTES))
        self.assertEqual(groups[str(self.case_group / 'measurement')]['memory.swap.max'], '0')
        archive_path = Path(receipt['archive']['path'])
        self.assertEqual(receipt['archive']['bytes'], archive_path.stat().st_size)
        self.assertEqual(receipt['archive']['sha256'], COLLECTOR.digest(archive_path))
        manifest = {record['relative_archive_path']: record for record in receipt['file_manifest']}
        self.assertIn(f'arrival-phase/{self.run_name}/failure-receipt.json', manifest)
        self.assertIn(f'arrival-phase/{self.validation_run}/bin/tenant_density_workload', manifest)
        self.assertIn('builds/arrival-fixture-r1-default-link/cargo.log', manifest)
        self.assertIn('arrival-phase/oci_arrival_collect.py', manifest)
        self.assertFalse(any('/db/' in archive_path for archive_path in manifest))
        with tarfile.open(archive_path, 'r:gz') as archive:
            self.assertEqual(set(archive.getnames()), set(manifest))
        sidecar = Path(self.root / self.attempt_name / 'collection.json')
        self.assertEqual(json.loads(sidecar.read_text())['archive']['sha256'], receipt['archive']['sha256'])

    def test_collection_attempt_namespace_is_never_overwritten(self):
        existing = self.root / self.attempt_name
        existing.mkdir()
        marker = existing / 'keep.txt'
        marker.write_text('immutable')
        with self.assertRaises(FileExistsError):
            COLLECTOR.collect(self.attempt_name)
        self.assertEqual(marker.read_text(), 'immutable')

    def test_initialization_failure_without_provenance_keeps_zero_cells_explicit(self):
        self.terminal_path.unlink()
        for path in self.controls_directory.glob('*-command.json'):
            path.unlink()
        for path in self.controls_directory.glob('*-status.json'):
            path.unlink()
        for suffix in ['.log', '-host-samples.json']:
            path = self.controls_directory / (self.case + suffix)
            if path.exists():
                path.unlink()
        shutil.rmtree(self.controls_directory / 'bin')
        (self.controls_directory / 'provenance.json').unlink()
        shutil.rmtree(self.case_group)
        shutil.rmtree(self.database_path)
        self.database_root.mkdir(parents=True, exist_ok=True)
        initialization_failure = self.controls_directory / 'initialization-failure-receipt.json'
        write_json(initialization_failure, {
            'completed': False,
            'source_commit': COLLECTOR.SOURCE_COMMIT,
            'plan_commit': 'e' * 40,
            'initialization_failure': {'type': 'fixture', 'message': 'before workloads'},
            'validation_sha256': COLLECTOR.digest(self.validation_summary_path),
            'source_validation': {'source_commit': COLLECTOR.SOURCE_COMMIT, 'clean': True},
            'runtime_limits': {'proc_soft': 65536, 'proc_hard': 524288},
        })

        receipt = COLLECTOR.collect('arrival-control-collection-init-fixture-r1')

        self.assertEqual(receipt['controls_terminal_status'], 'initialization_failed')
        self.assertFalse(receipt['controls_success_preserved'])
        self.assertEqual(len(receipt['controls']['launchers']), 3)
        self.assertEqual(receipt['controls']['statuses']['status_count'], 0)
        self.assertEqual(receipt['controls']['cgroups']['cell_count'], 0)
        self.assertTrue(receipt['controls']['cgroups']['grouping_root_memory_max_is_unbounded'])
        self.assertIsNone(receipt['controls']['cgroups']['cell_epoch_budgets_match'])
        self.assertEqual(receipt['controls']['runtime_limits']['proc_soft'], 65536)
        self.assertEqual(receipt['database_preserved_case_count'], 0)
        self.assertIsNone(receipt['controls_validity']['all_attempted_cell_databases_preserved'])
        self.assertFalse(receipt['controls_validity']['completed_run_evidence_valid'])

    def test_completed_evidence_remains_valid_when_one_cell_misses_slo(self):
        shutil.rmtree(self.case_group)
        shutil.rmtree(self.database_path)
        self.terminal_path.unlink()
        for path in self.controls_directory.glob('*-command.json'):
            path.unlink()
        for path in self.controls_directory.glob('*-status.json'):
            path.unlink()
        for suffix in ['.log', '-host-samples.json']:
            path = self.controls_directory / (self.case + suffix)
            if path.exists():
                path.unlink()
        cases = [f'cell-{cell_number:02d}' for cell_number in range(16)]
        outcomes = []
        for case in cases:
            cell_group = self.grouping_root / case
            self._write_group(cell_group)
            for epoch in COLLECTOR.EPOCHS:
                self._write_group(cell_group / epoch)
            database_path = self.database_root / case
            database_path.mkdir(parents=True)
            (database_path / 'CURRENT').write_text('preserved database fixture')
            write_json(self.controls_directory / f'{case}-command.json', {
                'command': [
                    '/bin/tenant_density_workload', '--data-dir', str(database_path),
                    '--backend-cgroup-root', str(cell_group),
                ],
                'source_commit': COLLECTOR.SOURCE_COMMIT,
            })
            slo_pass = case != 'cell-07'
            write_json(self.controls_directory / f'{case}-status.json', {
                'case': case,
                'exit_code': 0,
                'stop_reason': None,
                'review_error': None,
                'owned_process_limits': [],
                'reduced': {'operational_pass': True, 'slo_pass': slo_pass},
            })
            (self.controls_directory / f'{case}.log').write_text('completed cell fixture\n')
            write_json(self.controls_directory / f'{case}-host-samples.json', [])
            outcomes.append({'case': case, 'exit_code': 0, 'review_error': None})
        provenance_path = self.controls_directory / 'provenance.json'
        provenance = json.loads(provenance_path.read_text())
        provenance['plan_commit'] = COLLECTOR.FROZEN_PLAN_COMMIT
        write_json(provenance_path, provenance)
        write_json(self.controls_directory / 'completion.json', {
            'completed': True,
            'source_commit': COLLECTOR.SOURCE_COMMIT,
            'plan_commit': COLLECTOR.FROZEN_PLAN_COMMIT,
            'cases': 16,
            'outcomes': outcomes,
            'processes': [
                {'pid': 900000 + cell_number, 'start_time_ticks': cell_number + 1}
                for cell_number in range(16)
            ],
        })

        receipt = COLLECTOR.collect('arrival-control-collection-slo-miss-fixture-r1')

        self.assertTrue(receipt['controls_validity']['completed_run_evidence_valid'])
        self.assertFalse(receipt['controls_validity']['all_sixteen_cells_slo_pass'])
        self.assertEqual(receipt['controls_validity']['slo_miss_cases'], ['cell-07'])
        self.assertFalse(receipt['controls_validity']['accepted_run'])

    def test_sha256_reads_large_files_in_one_megabyte_chunks(self):
        payload = b'x' * (2 * 1024 * 1024 + 13)
        reader = TrackingReader(payload)
        with patch.object(Path, 'open', return_value=reader):
            actual = COLLECTOR.digest(Path('/mock/archive'))
        self.assertEqual(actual, hashlib.sha256(payload).hexdigest())
        self.assertTrue(reader.read_sizes)
        self.assertLessEqual(max(reader.read_sizes), 1024 * 1024)
        self.assertEqual(reader.read_sizes[0], 1024 * 1024)


if __name__ == '__main__':
    unittest.main()
