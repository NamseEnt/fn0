import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile
import time


ROOT = Path('/home/opc/tenant-density-harness-20261027')
RUN_NAME = 'shutdown-offcpu-b22638bdc-r1'
SOURCE_COMMIT = 'b22638bdcc4a0b22dad0606c3003d193cf41bbed'
EXPECTED_NATIVE_ARCHIVE_SHA256 = '9e6f7ea77ba2a9b2285b0d85ff90b00925688ab09a1c8e14def3f10f0deca2a5'
NATIVE_ARCHIVE = Path('/bench/zfs/db/product-build/rocksdb-source/librocksdb.a')
BINARIES = ('tenant_density_workload', 'memory_diagnostic_worker')


def digest(path):
    value = hashlib.sha256()
    with Path(path).open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            value.update(chunk)
    return value.hexdigest()


def read_json(path):
    return json.loads(Path(path).read_text())


def write_new_json(path, value):
    with Path(path).open('x') as output:
        json.dump(value, output, indent=2)
        output.write('\n')


def git(source, arguments):
    return subprocess.check_output(
        ['git', '-c', f'safe.directory={source}', '-C', str(source), *arguments],
        text=True,
    ).strip()


def cgroup_snapshot(group_root):
    snapshot = {}
    group_root = Path(group_root)
    cgroup_process_files = sorted(group_root.rglob('cgroup.procs'))
    for process_file in cgroup_process_files:
        case_path = process_file.parent
        fields = {}
        for name in (
            'cgroup.procs', 'memory.max', 'memory.high', 'memory.swap.max',
            'memory.current', 'memory.events', 'memory.stat', 'cpu.stat', 'io.stat',
        ):
            path = case_path / name
            try:
                value = path.read_text()[:8192]
                fields[name] = value.strip()
            except FileNotFoundError:
                fields[name] = {'status': 'unsupported'}
            except PermissionError as error:
                fields[name] = {'status': 'denied', 'errno': error.errno}
            except OSError as error:
                fields[name] = {'status': 'error', 'errno': error.errno}
        fields['group_empty'] = not str(fields.get('cgroup.procs', '')).strip()
        snapshot[str(case_path.relative_to(group_root))] = fields
    return snapshot


def swap_snapshots(value):
    snapshots = []
    if isinstance(value, dict):
        if 'memory.swap.max' in value or 'memory.swap.current' in value:
            snapshots.append(value)
        for nested in value.values():
            snapshots.extend(swap_snapshots(nested))
    elif isinstance(value, list):
        for nested in value:
            snapshots.extend(swap_snapshots(nested))
    return snapshots


def collect(run_directory, output_directory, run_name):
    run_directory = Path(run_directory)
    output_directory = Path(output_directory)
    assert run_name == RUN_NAME
    assert run_directory.is_dir()
    output_directory.parent.mkdir(parents=True, exist_ok=True)
    output_directory.mkdir(exist_ok=False)
    provenance = read_json(run_directory / 'provenance.json')
    assert provenance['source_commit'] == SOURCE_COMMIT
    assert provenance['source_identity']['source_clean'] is True
    assert git(ROOT / 'source', ['rev-parse', 'HEAD']) == SOURCE_COMMIT
    assert not git(ROOT / 'source', ['status', '--porcelain'])
    completion_path = run_directory / 'completion.json'
    failure_path = run_directory / 'controller-failure.json'
    assert completion_path.exists() != failure_path.exists()
    completion = read_json(completion_path) if completion_path.exists() else None
    failure = read_json(failure_path) if failure_path.exists() else None
    probe = read_json(run_directory / 'capability-probe.json')
    assert probe['probe_child_terminated'] is True
    assert probe['wait_fields_available'] is True

    processes = {}
    databases = {}
    cases = []
    for status_path in sorted(run_directory.glob('offcpu-e3-n5-r*-blink-status.json')):
        status = read_json(status_path)
        case = status['case']
        report_path = run_directory / (case + '.json')
        log_path = run_directory / (case + '.log')
        observer_path = run_directory / (case + '-offcpu.jsonl')
        observer_status_path = run_directory / (case + '-offcpu-status.json')
        command_path = run_directory / (case + '-command.json')
        assert all(path.is_file() for path in (
            report_path, log_path, observer_status_path, command_path,
        ))
        report = read_json(report_path)
        assert report['memory_diagnostic']['source_revision'] == SOURCE_COMMIT
        assert report['memory_diagnostic']['harness_binary_sha256'] == \
            provenance['binaries']['tenant_density_workload']['sha256']
        assert digest(report_path) == status['report_sha256']
        assert report['effective_configuration']['backend_memory_budget']['max_bytes'] == 268435456
        assert report['effective_configuration']['backend_memory_budget']['high_bytes'] == 201326592
        pressure_snapshots = swap_snapshots(report.get('backend_memory_pressure', {}))
        assert pressure_snapshots
        assert all(snapshot.get('memory.swap.max') == 0 for snapshot in pressure_snapshots)
        assert all(snapshot.get('memory.swap.current') == 0 for snapshot in pressure_snapshots)
        for identity in report['memory_diagnostic']['processes']:
            key = (identity['pid'], identity['start_time_ticks'])
            processes[key] = {
                'pid': identity['pid'], 'start_time_ticks': identity['start_time_ticks'],
                'role': identity['role'], 'same_live_process': process_identity_live(identity),
            }
        command = read_json(command_path)['command']
        data_directory = Path(command[command.index('--data-dir') + 1])
        databases[case] = {'path': str(data_directory), 'preserved': data_directory.is_dir()}
        assert databases[case]['preserved']
        observer_status = read_json(observer_status_path)
        observer_workers_valid = bool(observer_status.get('workers')) and all(
            worker.get('error_count') == 0
            and worker.get('terminal_stage') == 'async_server_shutdown_end'
            and (
                worker.get('identity_valid') is True
                and worker.get('observation_status') == 'sampled_then_finished'
                and worker.get('captures', 0) > 0
                or worker.get('observation_status') == 'finished_before_sampling'
                and worker.get('captures', 0) == 0
            )
            for worker in observer_status.get('workers', [])
        )
        observer_valid = (
            not observer_status.get('observer_failure')
            and not observer_status.get('observer_errors')
            and observer_status.get('observer_error_count', 0) == 0
            and observer_path.is_file()
            and observer_workers_valid
        )
        cases.append({
            'case': case, 'exit_code': status['exit_code'],
            'operational_pass': status.get('reduced', {}).get('operational_pass'),
            'report_sha256': status['report_sha256'],
            'log_sha256': digest(log_path),
            'observer_jsonl_preserved': observer_path.is_file(),
            'observer_jsonl_sha256': digest(observer_path) if observer_path.is_file() else None,
            'observer_status_sha256': digest(observer_status_path),
            'observer_valid': observer_valid,
            'observer_failure': observer_status.get('observer_failure'),
            'observer_errors': observer_status.get('observer_errors', []),
            'database': databases[case],
        })
    if completion is not None:
        assert len(cases) == 3
        assert completion['completed'] is True and completion['cells'] == 3
        assert completion['attempted_cells'] == 3
        assert completion['completed_cells'] == 3
        assert all(case['operational_pass'] is True for case in cases)
    else:
        assert failure is not None
        assert failure['attempted_cells'] == len(cases)
        assert failure['completed_cells'] == sum(case['operational_pass'] is True for case in cases)
        assert len(cases) <= 3
    assert all(not value['same_live_process'] for value in processes.values())

    binaries = {}
    for name in BINARIES:
        path = run_directory / 'bin' / name
        expected = provenance['binaries'][name]
        binaries[name] = {
            'sha256': digest(path), 'bytes': path.stat().st_size,
            'matches_provenance': digest(path) == expected['sha256'],
        }
        assert binaries[name]['matches_provenance']
    native_archive_sha256_after = digest(NATIVE_ARCHIVE)
    assert native_archive_sha256_after == provenance['source_identity']['native_archive_sha256_before']
    assert native_archive_sha256_after == EXPECTED_NATIVE_ARCHIVE_SHA256
    groups = cgroup_snapshot(Path('/sys/fs/cgroup') / ('dodb-density-' + RUN_NAME))
    assert groups
    assert all(group['group_empty'] for group in groups.values())
    assert all(group['memory.swap.max'] == '0' for group in groups.values())
    capacity_module_path = ROOT / 'oci_capacity.py'
    import importlib.util
    import sys

    sys.path.insert(0, str(ROOT))
    spec = importlib.util.spec_from_file_location('oci_capacity', capacity_module_path)
    capacity_module = importlib.util.module_from_spec(spec)
    sys.modules['oci_capacity'] = capacity_module
    spec.loader.exec_module(capacity_module)
    capacity_after = capacity_module.capacity()
    capacity_module.guard(capacity_after)
    archive_path = output_directory / 'runtime.tar.gz'
    with tarfile.open(archive_path, 'x:gz') as archive:
        archive.add(run_directory, arcname=run_name)
    collection = {
        'collection_timestamp_unix_ns': time.time_ns(),
        'run_name': run_name,
        'source_commit': SOURCE_COMMIT,
        'source_clean': True,
        'completion_preserved': completion_path.exists(),
        'failure_preserved': failure_path.exists(),
        'attempted_cells': len(cases),
        'completed_cells': sum(case['operational_pass'] is True for case in cases),
        'all_observations_valid': all(case['observer_valid'] for case in cases),
        'valid_diagnostic_run': (
            completion is not None
            and all(case['operational_pass'] is True and case['observer_valid'] for case in cases)
        ),
        'cases': cases,
        'processes': list(processes.values()),
        'databases': databases,
        'groups': groups,
        'binaries': binaries,
        'native_archive_sha256_before': provenance['source_identity']['native_archive_sha256_before'],
        'native_archive_sha256_after': native_archive_sha256_after,
        'capability_probe_sha256': digest(run_directory / 'capability-probe.json'),
        'capacity_after': capacity_after,
        'runtime_archive': str(archive_path),
        'runtime_archive_sha256': digest(archive_path),
    }
    write_new_json(output_directory / 'collection.json', collection)
    return collection


def process_identity_live(identity):
    try:
        raw = (Path('/proc') / str(identity['pid']) / 'stat').read_text()
        return int(raw[raw.rfind(')') + 2:].split()[19]) == identity['start_time_ticks']
    except (FileNotFoundError, IndexError, ValueError):
        return False


def main():
    parser = argparse.ArgumentParser(description='Preserve and verify off-CPU shutdown runtime evidence')
    parser.add_argument('--run-name', required=True)
    parser.add_argument('--run-directory', type=Path)
    parser.add_argument('--output-directory', type=Path)
    arguments = parser.parse_args()
    run_directory = arguments.run_directory or ROOT / 'capacity' / arguments.run_name
    output_directory = arguments.output_directory or Path(__file__).with_name('runtime') / arguments.run_name
    result = collect(run_directory, output_directory, arguments.run_name)
    print(json.dumps({
        'run_name': result['run_name'], 'completed_cells': result['completed_cells'],
        'completion_preserved': result['completion_preserved'],
        'failure_preserved': result['failure_preserved'],
        'runtime_archive_sha256': result['runtime_archive_sha256'],
    }), flush=True)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
