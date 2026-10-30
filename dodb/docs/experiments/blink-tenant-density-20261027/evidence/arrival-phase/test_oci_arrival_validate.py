import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


HELPER_PATH = Path(__file__).with_name('oci_arrival_validate.py')
SPEC = importlib.util.spec_from_file_location('oci_arrival_validate_under_test', HELPER_PATH)
VALIDATOR = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = VALIDATOR
SPEC.loader.exec_module(VALIDATOR)


def cargo_test_log(counts):
    return ''.join(f'test result: ok. {count} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n' for count in counts)


def clippy_log(diagnostics):
    lines = []
    for diagnostic in diagnostics:
        lines.extend([
            f"warning: {diagnostic['message']}",
            f" --> {diagnostic['path']}:1:1",
            '  |',
            '1 | fn sample() {}',
            '  | ^^^^^^^^^^^^^',
            '',
        ])
    return '\n'.join(lines)


class ArrivalValidationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name) / 'harness-root'
        self.root.mkdir()
        (self.root / 'oci_harness_build.py').write_text('fake build helper fixture\n')
        self.source = self.root / 'source'
        self.source.mkdir()
        self.native = Path(self.temporary.name) / 'native'
        self.native.mkdir()
        self.native_archive = self.native / 'librocksdb.a'
        self.native_archive.write_bytes(b'pinned archive fixture')
        self.native_sha256 = VALIDATOR.digest(self.native_archive)
        target_directory = self.root / 'target-cache'
        target_directory.mkdir()
        (self.root / 'target').symlink_to(target_directory, target_is_directory=True)
        self.baseline_summary_path = self.root / 'shutdown-validation-summary.json'
        self.baseline_test_log = self.root / 'builds' / 'shutdown-clean-tests' / 'cargo.log'
        self.baseline_clippy_log = self.root / 'builds' / 'shutdown-clean-clippy' / 'cargo.log'
        self.baseline_test_log.parent.mkdir(parents=True)
        self.baseline_clippy_log.parent.mkdir(parents=True)
        self.baseline_test_log.write_text(cargo_test_log([42, 9, 19, 5, 4]))
        self.diagnostics = [
            {'message': f'baseline diagnostic {index}', 'path': f'dodb/crates/dodb-rocksdb/src/file_{index}.rs'}
            for index in range(17)
        ]
        baseline_clippy_text = clippy_log(self.diagnostics)
        self.baseline_clippy_log.write_text(baseline_clippy_text)
        self.diagnostics = VALIDATOR.clippy_diagnostics(baseline_clippy_text)
        self.baseline_summary_path.write_text(json.dumps({
            'source_commit': VALIDATOR.BASELINE_SOURCE,
            'passing_tests': 79,
            'same_17_existing_clippy_diagnostics': True,
            'baseline_clippy_diagnostics': self.diagnostics,
            'outcomes': [
                {'name': 'shutdown-clean-tests', 'exit_code': 0, 'test_group_counts': [42, 9, 19, 5, 4]},
                {'name': 'shutdown-clean-clippy', 'exit_code': 0},
            ],
        }))
        self.target_binaries = self.root / 'target' / 'release'
        self.target_binaries.mkdir()
        for binary_name in ['tenant_density_workload', 'memory_diagnostic_worker']:
            (self.target_binaries / binary_name).write_bytes(f'{binary_name} fixture'.encode())
        self.build_calls = []
        self.fail_next = False
        VALIDATOR.ROOT = self.root
        VALIDATOR.SOURCE = self.source
        VALIDATOR.NATIVE = self.native
        VALIDATOR.NATIVE_ARCHIVE = self.native_archive
        VALIDATOR.EXPECTED_NATIVE_SHA256 = self.native_sha256
        VALIDATOR.EXPECTED_NATIVE_COMMIT = 'native-fixture'
        VALIDATOR.BASELINE_SUMMARY = self.baseline_summary_path
        VALIDATOR.RUN_NAME = 'arrival-fixture-r1'
        VALIDATOR.SUMMARY_NAME = 'arrival-validation-summary.json'
        VALIDATOR.build = self.fake_build
        self.command_patcher = patch.object(VALIDATOR, 'command', side_effect=self.fake_command)
        self.command_patcher.start()

    def tearDown(self):
        self.command_patcher.stop()
        self.temporary.cleanup()

    def fake_command(self, arguments, cwd=None):
        if arguments == ['git', 'rev-parse', 'HEAD']:
            self.assertEqual(Path(cwd), self.source)
            return VALIDATOR.EXPECTED_SOURCE
        if arguments == ['git', 'status', '--porcelain']:
            self.assertEqual(Path(cwd), self.source)
            return ''
        if arguments == ['git', '-C', str(self.native), 'rev-parse', 'HEAD']:
            self.assertIsNone(cwd)
            return VALIDATOR.EXPECTED_NATIVE_COMMIT
        raise AssertionError(f'unexpected command: {arguments} cwd={cwd}')

    def fake_build(self, build_name, arguments):
        self.build_calls.append((build_name, arguments))
        build_directory = self.root / 'builds' / build_name
        build_directory.mkdir(parents=True, exist_ok=False)
        helper_sha256 = VALIDATOR.digest(self.root / 'oci_harness_build.py')
        provenance = {
            'controller_sha256': helper_sha256,
            'command': VALIDATOR.full_build_command(arguments),
            'cwd': str(self.source),
            'source_commit': VALIDATOR.EXPECTED_SOURCE,
            'source_status': '',
            'native_commit': VALIDATOR.EXPECTED_NATIVE_COMMIT,
            'native_archive_sha256': self.native_sha256,
            'toolchain': VALIDATOR.TOOLCHAIN,
            'environment': VALIDATOR.expected_environment(),
        }
        (build_directory / 'provenance.json').write_text(json.dumps(provenance))
        if arguments[0] == 'test':
            log_text = cargo_test_log([42, 9, 25, 5, 4])
        elif arguments[0] == 'clippy':
            log_text = self.baseline_clippy_log.read_text()
        else:
            log_text = ''
        (build_directory / 'cargo.log').write_text(log_text)
        exit_code = 1 if self.fail_next else 0
        self.fail_next = False
        (build_directory / 'status.json').write_text(json.dumps({'exit_code': exit_code}))
        return exit_code

    def test_preflight_uses_source_git_and_rejects_nonmatching_native_artifacts(self):
        identity = VALIDATOR.validate_source_and_native()
        self.assertEqual(identity['source_commit'], VALIDATOR.EXPECTED_SOURCE)
        self.assertEqual(identity['source_status'], '')
        self.assertEqual(identity['native_archive_sha256_before'], self.native_sha256)
        VALIDATOR.EXPECTED_NATIVE_SHA256 = '0' * 64
        with self.assertRaises(AssertionError):
            VALIDATOR.validate_source_and_native()

    def test_mocked_build_sequence_saves_provenance_and_default_binaries(self):
        summary = VALIDATOR.run_validation()
        self.assertEqual(summary['status'], 'completed')
        self.assertEqual([name for name, _ in self.build_calls], [f'arrival-fixture-r1-{name}' for name, _ in VALIDATOR.CHECKS])
        self.assertEqual(summary['outcomes'][0]['passing_tests'], 85)
        self.assertEqual(summary['outcomes'][1]['same_17_existing_clippy_diagnostics'], True)
        self.assertEqual(summary['outcomes'][-1]['name'], 'default-link')
        self.assertTrue(summary['default_link_was_last'])
        for binary in ['tenant_density_workload', 'memory_diagnostic_worker']:
            self.assertTrue((self.root / 'arrival-phase' / VALIDATOR.RUN_NAME / 'bin' / binary).is_file())
        self.assertEqual(len(summary['default_binaries']), 2)

    def test_first_failed_check_is_saved_and_no_later_check_runs(self):
        self.fail_next = True
        with self.assertRaisesRegex(AssertionError, 'validation check failed: tests'):
            VALIDATOR.run_validation()
        summary_path = self.root / 'arrival-phase' / VALIDATOR.RUN_NAME / VALIDATOR.SUMMARY_NAME
        summary = json.loads(summary_path.read_text())
        self.assertEqual(summary['status'], 'failed')
        self.assertEqual(len(summary['outcomes']), 1)
        self.assertEqual(summary['outcomes'][0]['name'], 'tests')
        self.assertEqual(summary['outcomes'][0]['exit_code'], 1)
        self.assertEqual(len(self.build_calls), 1)
        with self.assertRaisesRegex(AssertionError, 'terminal and cannot be retried'):
            VALIDATOR.run_validation()


if __name__ == '__main__':
    unittest.main()
