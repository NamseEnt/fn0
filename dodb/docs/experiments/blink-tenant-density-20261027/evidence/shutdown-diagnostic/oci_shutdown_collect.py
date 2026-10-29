import argparse
import json
from pathlib import Path
import subprocess
import sys
import tarfile

ROOT = Path('/home/opc/tenant-density-harness-20261027')
sys.path.insert(0, str(ROOT))
from oci_harness_build import digest
from oci_capacity import SOURCE, capacity, process_identity_live


SOURCE_COMMIT = 'b22638bdcc4a0b22dad0606c3003d193cf41bbed'
EXPECTED_NATIVE_ARCHIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
NATIVE_ARCHIVE = Path('/bench/zfs/db/product-build/rocksdb-source/librocksdb.a')
VALIDATION_NAME = 'shutdown-validation-summary.json'
TRACE_SOURCE = SOURCE / 'dodb/crates/dodb-rocksdb/src/bin/support/shutdown_trace.rs'
VALIDATION_CHECK_NAMES = [
    'shutdown-clean-tests', 'shutdown-clean-clippy',
    'shutdown-clean-a1-link', 'shutdown-clean-default-link',
]
BASELINE_CHECK_NAMES = [
    'observations-clean-tests', 'observations-clean-clippy',
    'observations-clean-a1-link', 'observations-clean-default-link',
]


def git(arguments):
    return subprocess.check_output(
        ['git', '-c', f'safe.directory={SOURCE}', '-C', str(SOURCE), *arguments], text=True
    ).strip()


def write_new_json(file_path, value):
    with file_path.open('x') as output:
        json.dump(value, output, indent=2)
        output.write('\n')


def main(arguments):
    assert arguments.archive_name == 'shutdown-runtime-b22638bdc-r1'
    assert git(['rev-parse', 'HEAD']) == SOURCE_COMMIT
    assert not git(['status', '--porcelain'])
    validation_path = ROOT / VALIDATION_NAME
    validation = json.loads(validation_path.read_text())
    assert validation['source_commit'] == SOURCE_COMMIT
    assert validation['passing_tests'] == 79
    assert validation['same_17_existing_clippy_diagnostics']
    assert [outcome['name'] for outcome in validation['outcomes']] == VALIDATION_CHECK_NAMES
    assert all(outcome['exit_code'] == 0 for outcome in validation['outcomes'])
    assert validation['native_archive_sha256_before'] == EXPECTED_NATIVE_ARCHIVE_SHA256
    assert validation['native_archive_sha256_after'] == EXPECTED_NATIVE_ARCHIVE_SHA256
    validation_builds = {}
    for name in [*BASELINE_CHECK_NAMES, *VALIDATION_CHECK_NAMES]:
        build_directory = ROOT / 'builds' / name
        assert build_directory.is_dir()
        validation_builds[name] = {
            str(file_path.relative_to(ROOT)): digest(file_path)
            for file_path in sorted(build_directory.rglob('*')) if file_path.is_file()
        }
    native_archive_sha256 = digest(NATIVE_ARCHIVE)
    assert native_archive_sha256 == EXPECTED_NATIVE_ARCHIVE_SHA256
    postflight = {
        'collector_sha256': digest(Path(__file__)),
        'capacity': capacity(),
        'source_commit': SOURCE_COMMIT,
        'source_clean': True,
        'native_archive_sha256': native_archive_sha256,
        'native_archive_bytes': NATIVE_ARCHIVE.stat().st_size,
        'validation_sha256': digest(validation_path),
        'validation_passing_tests': validation['passing_tests'],
        'validation_same_17_existing_clippy_diagnostics': validation['same_17_existing_clippy_diagnostics'],
        'target_symlink': str((ROOT / 'target').readlink()),
        'resolved_target_directory': str((ROOT / 'target').resolve(strict=True)),
        'validation_build_hashes': validation_builds,
        'runs': {},
    }
    assert arguments.runs == ['shutdown-b22638bdc-r1']
    for run_name in arguments.runs:
        assert run_name == 'shutdown-b22638bdc-r1'
        directory = ROOT / 'capacity' / run_name
        completion_path = directory / 'completion.json'
        failure_path = directory / 'controller-failure.json'
        assert completion_path.exists() != failure_path.exists()
        provenance = json.loads((directory / 'provenance.json').read_text())
        assert provenance['source_commit'] == SOURCE_COMMIT
        assert provenance['shutdown_controller_sha256'] == digest(ROOT / 'oci_shutdown_diagnostic.py')
        assert provenance['shutdown_trace_source_sha256'] == digest(TRACE_SOURCE)
        assert provenance['shutdown_validation_sha256'] == digest(validation_path)
        assert provenance['native_archive_sha256_before'] == native_archive_sha256
        binaries = {
            name: {'sha256': digest(directory / 'bin' / name), 'bytes': (directory / 'bin' / name).stat().st_size}
            for name in provenance['binaries']
        }
        assert binaries == provenance['binaries']
        processes = {}
        databases = {}
        groups = {}
        statuses = sorted(directory.glob('*-status.json'))
        assert statuses
        for status_path in statuses:
            status = json.loads(status_path.read_text())
            assert status_path.name == status['case'] + '-status.json'
            report_path = directory / (status['case'] + '.json')
            assert digest(report_path) == status['report_sha256']
            report = json.loads(report_path.read_text())
            diagnostic = report['memory_diagnostic']
            assert diagnostic['source_revision'] == SOURCE_COMMIT
            assert diagnostic['processes']
            assert all(event['source_revision'] == SOURCE_COMMIT for event in diagnostic['worker_lifecycle'])
            assert all(event['harness_binary_sha256'] == binaries['tenant_density_workload']['sha256']
                       for event in diagnostic['worker_lifecycle'])
            assert all(event['worker_binary_sha256'] == binaries['memory_diagnostic_worker']['sha256']
                       for event in diagnostic['worker_lifecycle'])
            for identity in diagnostic['processes']:
                identity_key = (identity['pid'], identity['start_time_ticks'])
                processes[identity_key] = {
                    'pid': identity['pid'], 'start_time_ticks': identity['start_time_ticks'],
                    'role': identity['role'], 'same_live_process': process_identity_live(identity),
                }
            command_path = directory / (status['case'] + '-command.json')
            command = json.loads(command_path.read_text())['command']
            data_directory = Path(command[command.index('--data-dir') + 1])
            databases[status['case']] = {'path': str(data_directory), 'preserved': data_directory.is_dir()}
            assert databases[status['case']]['preserved']
            group = Path(report['backend_memory_pressure']['configuration']['cgroup_root'])
            assert group.is_dir()
            cgroup_files = sorted(group.rglob('cgroup.procs'))
            assert cgroup_files
            groups[status['case']] = {
                str(file_path.relative_to(group)): file_path.read_text().strip()
                for file_path in cgroup_files
            }
        assert all(not identity['same_live_process'] for identity in processes.values())
        assert all(not pids for group in groups.values() for pids in group.values())
        if completion_path.exists():
            completion = json.loads(completion_path.read_text())
            assert completion['completed'] and completion['source_commit'] == SOURCE_COMMIT
            assert completion['cases'] == len(statuses)
        else:
            failure = json.loads(failure_path.read_text())
            assert failure['completed_cells'] == len(statuses)
        postflight['runs'][run_name] = {
            'source_commit': provenance['source_commit'],
            'binaries': binaries,
            'processes': list(processes.values()),
            'databases': databases,
            'cgroup_processes': groups,
            'status_count': len(statuses),
            'completed': completion_path.exists(),
            'controller_failure': failure_path.exists(),
        }
    assert git(['rev-parse', 'HEAD']) == SOURCE_COMMIT
    assert not git(['status', '--porcelain'])
    assert digest(NATIVE_ARCHIVE) == native_archive_sha256
    extra_files = []
    for name in arguments.extra_files:
        relative_path = Path(name)
        assert not relative_path.is_absolute() and '..' not in relative_path.parts
        file_path = (ROOT / relative_path).resolve(strict=True)
        relative_path = file_path.relative_to(ROOT.resolve(strict=True))
        assert file_path.is_file()
        extra_files.append({
            'path': str(relative_path), 'sha256': digest(file_path),
            'bytes': file_path.stat().st_size,
        })
    postflight['extra_files'] = extra_files
    receipt_path = ROOT / (arguments.archive_name + '-postflight.json')
    archive_path = ROOT / (arguments.archive_name + '.tar.gz')
    assert not receipt_path.exists() and not archive_path.exists()
    write_new_json(receipt_path, postflight)
    with tarfile.open(archive_path, 'x:gz') as archive:
        archive.add(receipt_path, arcname='collection/' + receipt_path.name)
        archive.add(validation_path, arcname='validation/' + VALIDATION_NAME)
        for name in [*BASELINE_CHECK_NAMES, *VALIDATION_CHECK_NAMES]:
            build_directory = ROOT / 'builds' / name
            validation_namespace = 'baseline-builds' if name.startswith('observations-clean-') else 'builds'
            archive.add(build_directory, arcname='validation/' + validation_namespace + '/' + name)
        for record in extra_files:
            file_path = ROOT / record['path']
            archive.add(file_path, arcname='collection/launcher-evidence/' + record['path'])
        for run_name in arguments.runs:
            directory = ROOT / 'capacity' / run_name
            for file_path in sorted(directory.rglob('*')):
                relative = file_path.relative_to(directory)
                if file_path.is_file() and 'bin' not in relative.parts:
                    archive.add(file_path, arcname=str(Path('runtime') / run_name / relative))
    print(json.dumps({
        'archive': str(archive_path), 'archive_sha256': digest(archive_path),
        'bytes': archive_path.stat().st_size, 'runs': arguments.runs,
        'preserved_database_records': sum(len(run['databases']) for run in postflight['runs'].values()),
    }), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--runs', nargs='+', required=True)
    parser.add_argument('--archive-name', required=True)
    parser.add_argument('--extra-files', nargs='*', default=[])
    arguments = parser.parse_args()
    main(arguments)
